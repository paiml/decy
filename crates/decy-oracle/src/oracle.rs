//! Main oracle implementation

use crate::config::OracleConfig;
use crate::context::CDecisionContext;
use crate::error::OracleError;
use crate::metrics::OracleMetrics;

#[cfg(feature = "citl")]
use entrenar::citl::{DecisionPatternStore, FixSuggestion as EntrenarFixSuggestion};

/// Fix suggestion from the oracle
#[cfg(feature = "citl")]
pub type FixSuggestion = EntrenarFixSuggestion;

/// Rustc error information
#[derive(Debug, Clone)]
pub struct RustcError {
    /// Error code (e.g., "E0382")
    pub code: String,
    /// Error message
    pub message: String,
    /// File path
    pub file: Option<String>,
    /// Line number
    pub line: Option<usize>,
}

impl RustcError {
    /// Create a new rustc error
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            file: None,
            line: None,
        }
    }

    /// Add file location
    pub fn with_location(mut self, file: impl Into<String>, line: usize) -> Self {
        self.file = Some(file.into());
        self.line = Some(line);
        self
    }
}

/// Decy CITL Oracle
///
/// Queries accumulated fix patterns to suggest corrections for rustc errors.
pub struct DecyOracle {
    config: OracleConfig,
    #[cfg(feature = "citl")]
    store: Option<DecisionPatternStore>,
    metrics: OracleMetrics,
}

impl DecyOracle {
    /// Create a new oracle from configuration
    pub fn new(config: OracleConfig) -> Result<Self, OracleError> {
        #[cfg(feature = "citl")]
        let store = if config.patterns_path.exists() {
            Some(
                DecisionPatternStore::load_apr(&config.patterns_path)
                    .map_err(|e| OracleError::PatternStoreError(e.to_string()))?,
            )
        } else {
            None
        };

        Ok(Self {
            config,
            #[cfg(feature = "citl")]
            store,
            metrics: OracleMetrics::default(),
        })
    }

    /// Check if the oracle has patterns loaded
    pub fn has_patterns(&self) -> bool {
        #[cfg(feature = "citl")]
        {
            self.store.is_some()
        }
        #[cfg(not(feature = "citl"))]
        {
            false
        }
    }

    /// Get the number of patterns loaded
    pub fn pattern_count(&self) -> usize {
        #[cfg(feature = "citl")]
        {
            self.store.as_ref().map(|s| s.len()).unwrap_or(0)
        }
        #[cfg(not(feature = "citl"))]
        {
            0
        }
    }

    /// Query for fix suggestion
    #[cfg(feature = "citl")]
    pub fn suggest_fix(
        &mut self,
        error: &RustcError,
        context: &CDecisionContext,
    ) -> Option<FixSuggestion> {
        let store = match self.store.as_ref() {
            Some(s) => s,
            None => {
                self.metrics.record_miss(&error.code);
                return None;
            }
        };

        let context_strings = context.to_context_strings();
        let suggestions = match store.suggest_fix(&error.code, &context_strings, self.config.max_suggestions) {
            Ok(s) => s,
            Err(_) => {
                self.metrics.record_miss(&error.code);
                return None;
            }
        };

        let best = match suggestions
            .into_iter()
            .find(|s| s.weighted_score() >= self.config.confidence_threshold)
        {
            Some(b) => b,
            None => {
                self.metrics.record_miss(&error.code);
                return None;
            }
        };

        self.metrics.record_hit(&error.code);
        Some(best)
    }

    /// Query for fix suggestion (stub when citl feature disabled)
    #[cfg(not(feature = "citl"))]
    pub fn suggest_fix(
        &mut self,
        error: &RustcError,
        _context: &CDecisionContext,
    ) -> Option<()> {
        self.metrics.record_miss(&error.code);
        None
    }

    /// Record a miss (no suggestion found)
    pub fn record_miss(&mut self, error: &RustcError) {
        self.metrics.record_miss(&error.code);
    }

    /// Record a successful fix application
    pub fn record_fix_applied(&mut self, error: &RustcError) {
        self.metrics.record_fix_applied(&error.code);
    }

    /// Record a verified fix (compiled successfully)
    pub fn record_fix_verified(&mut self, error: &RustcError) {
        self.metrics.record_fix_verified(&error.code);
    }

    /// Get current metrics
    pub fn metrics(&self) -> &OracleMetrics {
        &self.metrics
    }

    /// Get configuration
    pub fn config(&self) -> &OracleConfig {
        &self.config
    }

    /// Import patterns from another .apr file (cross-project transfer)
    ///
    /// Uses the smart import filter to verify fix strategies are applicable
    /// to C→Rust context (not just Python→Rust patterns).
    #[cfg(feature = "citl")]
    pub fn import_patterns(&mut self, path: &std::path::Path) -> Result<usize, OracleError> {
        self.import_patterns_with_config(path, crate::import::SmartImportConfig::default())
    }

    /// Import patterns with custom configuration
    #[cfg(feature = "citl")]
    pub fn import_patterns_with_config(
        &mut self,
        path: &std::path::Path,
        config: crate::import::SmartImportConfig,
    ) -> Result<usize, OracleError> {
        use crate::import::{smart_import_filter, ImportStats};

        let other_store = DecisionPatternStore::load_apr(path)
            .map_err(|e| OracleError::PatternStoreError(e.to_string()))?;

        // Transferable error codes (ownership/lifetime)
        let transferable = ["E0382", "E0499", "E0506", "E0597", "E0515"];

        let store = self.store.get_or_insert_with(|| {
            DecisionPatternStore::new().expect("Failed to create pattern store")
        });

        let mut count = 0;
        let mut stats = ImportStats::new();

        for code in &transferable {
            let patterns = other_store.patterns_for_error(code);
            for pattern in patterns {
                // Apply smart import filter
                let strategy = crate::import::analyze_fix_strategy(&pattern.fix_diff);
                let decision = smart_import_filter(&pattern.fix_diff, &pattern.metadata, &config);

                stats.record(strategy, &decision);

                if decision.allows_import() {
                    if store.index_fix(pattern.clone()).is_ok() {
                        count += 1;
                    }
                }
            }
        }

        // Log import statistics
        if stats.total_evaluated > 0 {
            tracing::info!(
                "Import stats: {}/{} patterns accepted ({:.1}%)",
                count,
                stats.total_evaluated,
                stats.overall_acceptance_rate() * 100.0
            );
        }

        Ok(count)
    }

    /// Import patterns with statistics tracking
    #[cfg(feature = "citl")]
    pub fn import_patterns_with_stats(
        &mut self,
        path: &std::path::Path,
        config: crate::import::SmartImportConfig,
    ) -> Result<(usize, crate::import::ImportStats), OracleError> {
        use crate::import::{smart_import_filter, ImportStats};

        let other_store = DecisionPatternStore::load_apr(path)
            .map_err(|e| OracleError::PatternStoreError(e.to_string()))?;

        let transferable = ["E0382", "E0499", "E0506", "E0597", "E0515"];

        let store = self.store.get_or_insert_with(|| {
            DecisionPatternStore::new().expect("Failed to create pattern store")
        });

        let mut count = 0;
        let mut stats = ImportStats::new();

        for code in &transferable {
            let patterns = other_store.patterns_for_error(code);
            for pattern in patterns {
                let strategy = crate::import::analyze_fix_strategy(&pattern.fix_diff);
                let decision = smart_import_filter(&pattern.fix_diff, &pattern.metadata, &config);

                stats.record(strategy, &decision);

                if decision.allows_import() {
                    if store.index_fix(pattern.clone()).is_ok() {
                        count += 1;
                    }
                }
            }
        }

        Ok((count, stats))
    }

    /// Save patterns to .apr file
    #[cfg(feature = "citl")]
    pub fn save(&self) -> Result<(), OracleError> {
        if let Some(ref store) = self.store {
            store
                .save_apr(&self.config.patterns_path)
                .map_err(|e| OracleError::SaveError {
                    path: self.config.patterns_path.display().to_string(),
                    source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
                })?;
        }
        Ok(())
    }

    /// Bootstrap the oracle with seed patterns for cold start
    ///
    /// This loads predefined patterns for common C→Rust transpilation errors,
    /// solving the cold start problem where the oracle has no patterns to learn from.
    ///
    /// # Toyota Way Principles
    ///
    /// - **Genchi Genbutsu**: Patterns derived from real C→Rust errors
    /// - **Yokoten**: Cross-project pattern sharing
    /// - **Jidoka**: Automated quality built-in
    #[cfg(feature = "citl")]
    pub fn bootstrap(&mut self) -> Result<usize, OracleError> {
        use crate::bootstrap::seed_pattern_store;

        let store = self.store.get_or_insert_with(|| {
            DecisionPatternStore::new().expect("Failed to create pattern store")
        });

        seed_pattern_store(store)
    }

    /// Check if bootstrap patterns are needed
    ///
    /// Returns true if the oracle has no patterns or very few patterns,
    /// indicating that bootstrapping would be beneficial.
    pub fn needs_bootstrap(&self) -> bool {
        self.pattern_count() < 10
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::CConstruct;
    use crate::decisions::CDecisionCategory;

    #[test]
    fn test_oracle_creation_no_patterns() {
        // Use a path that doesn't exist to test no-patterns case
        let config = OracleConfig {
            patterns_path: std::path::PathBuf::from("/tmp/nonexistent_test_patterns.apr"),
            ..Default::default()
        };
        let oracle = DecyOracle::new(config).unwrap();
        assert!(!oracle.has_patterns()); // No patterns file exists
    }

    #[test]
    fn test_oracle_pattern_count_empty() {
        // Use a path that doesn't exist to test empty case
        let config = OracleConfig {
            patterns_path: std::path::PathBuf::from("/tmp/nonexistent_test_patterns.apr"),
            ..Default::default()
        };
        let oracle = DecyOracle::new(config).unwrap();
        assert_eq!(oracle.pattern_count(), 0);
    }

    #[test]
    fn test_oracle_config_access() {
        let config = OracleConfig {
            confidence_threshold: 0.9,
            ..Default::default()
        };
        let oracle = DecyOracle::new(config).unwrap();
        assert!((oracle.config().confidence_threshold - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rustc_error() {
        let error = RustcError::new("E0382", "borrow of moved value")
            .with_location("test.rs", 42);
        assert_eq!(error.code, "E0382");
        assert_eq!(error.line, Some(42));
    }

    #[test]
    fn test_rustc_error_without_location() {
        let error = RustcError::new("E0499", "cannot borrow as mutable more than once");
        assert_eq!(error.code, "E0499");
        assert_eq!(error.message, "cannot borrow as mutable more than once");
        assert!(error.file.is_none());
        assert!(error.line.is_none());
    }

    #[test]
    fn test_rustc_error_chained_builder() {
        let error = RustcError::new("E0506", "cannot assign")
            .with_location("src/main.rs", 100);
        assert_eq!(error.code, "E0506");
        assert_eq!(error.file, Some("src/main.rs".into()));
        assert_eq!(error.line, Some(100));
    }

    #[test]
    fn test_metrics_recorded() {
        let config = OracleConfig::default();
        let mut oracle = DecyOracle::new(config).unwrap();

        let error = RustcError::new("E0382", "test");
        let context = CDecisionContext::new(
            CConstruct::RawPointer {
                is_const: false,
                pointee: "int".into(),
            },
            CDecisionCategory::PointerOwnership,
        );

        // No patterns, should be a miss
        let _ = oracle.suggest_fix(&error, &context);
        assert_eq!(oracle.metrics().misses, 1);
    }

    #[test]
    fn test_record_miss() {
        let config = OracleConfig::default();
        let mut oracle = DecyOracle::new(config).unwrap();

        let error = RustcError::new("E0597", "borrowed value does not live long enough");
        oracle.record_miss(&error);
        assert_eq!(oracle.metrics().misses, 1);
        assert_eq!(oracle.metrics().queries, 1);
    }

    #[test]
    fn test_record_fix_applied() {
        let config = OracleConfig::default();
        let mut oracle = DecyOracle::new(config).unwrap();

        let error = RustcError::new("E0382", "use of moved value");
        oracle.record_fix_applied(&error);
        assert_eq!(oracle.metrics().fixes_applied, 1);
    }

    #[test]
    fn test_record_fix_verified() {
        let config = OracleConfig::default();
        let mut oracle = DecyOracle::new(config).unwrap();

        let error = RustcError::new("E0515", "cannot return reference to local");
        oracle.record_fix_verified(&error);
        assert_eq!(oracle.metrics().fixes_verified, 1);
    }

    #[test]
    fn test_multiple_error_codes_tracked() {
        let config = OracleConfig::default();
        let mut oracle = DecyOracle::new(config).unwrap();

        oracle.record_miss(&RustcError::new("E0382", "test"));
        oracle.record_miss(&RustcError::new("E0499", "test"));
        oracle.record_miss(&RustcError::new("E0382", "test"));

        let metrics = oracle.metrics();
        assert_eq!(metrics.misses, 3);
        assert_eq!(metrics.by_error_code.get("E0382").unwrap().queries, 2);
        assert_eq!(metrics.by_error_code.get("E0499").unwrap().queries, 1);
    }
}
