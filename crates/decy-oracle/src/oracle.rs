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
        let store = self.store.as_ref()?;

        let context_strings = context.to_context_strings();
        let suggestions = store
            .suggest_fix(&error.code, &context_strings, self.config.max_suggestions)
            .ok()?;

        let best = suggestions
            .into_iter()
            .find(|s| s.weighted_score() >= self.config.confidence_threshold)?;

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
    #[cfg(feature = "citl")]
    pub fn import_patterns(&mut self, path: &std::path::Path) -> Result<usize, OracleError> {
        let other_store = DecisionPatternStore::load_apr(path)
            .map_err(|e| OracleError::PatternStoreError(e.to_string()))?;

        // Transferable error codes (ownership/lifetime)
        let transferable = ["E0382", "E0499", "E0506", "E0597", "E0515"];

        let store = self.store.get_or_insert_with(|| {
            DecisionPatternStore::new().expect("Failed to create pattern store")
        });

        let mut count = 0;
        for code in &transferable {
            if let Ok(patterns) = other_store.patterns_for_error(code) {
                for pattern in patterns {
                    if store.index_fix(pattern.clone()).is_ok() {
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::CConstruct;
    use crate::decisions::CDecisionCategory;

    #[test]
    fn test_oracle_creation() {
        let config = OracleConfig::default();
        let oracle = DecyOracle::new(config).unwrap();
        assert!(!oracle.has_patterns()); // No patterns file exists
    }

    #[test]
    fn test_rustc_error() {
        let error = RustcError::new("E0382", "borrow of moved value")
            .with_location("test.rs", 42);
        assert_eq!(error.code, "E0382");
        assert_eq!(error.line, Some(42));
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
}
