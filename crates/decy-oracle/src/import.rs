//! Smart import filter for cross-project pattern transfer
//!
//! Filters imported patterns by fix strategy applicability, not just error code.
//! Python ownership issues differ from C issues (reference counting vs pointer aliasing).
//!
//! # References
//! - training-oracle-spec.md §3.1.2: Smart Import Filter (Yokoten Enhancement)
//! - Gemini Review: "smart import is better than blind bulk import"

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Fix strategy type derived from diff analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FixStrategy {
    /// Add .clone() call
    AddClone,
    /// Add borrow (&T or &mut T)
    AddBorrow,
    /// Add lifetime annotation
    AddLifetime,
    /// Wrap in Option<T>
    WrapInOption,
    /// Wrap in Result<T, E>
    WrapInResult,
    /// Add explicit type annotation
    AddTypeAnnotation,
    /// Unknown or complex strategy
    Unknown,
}

/// Decision for importing a pattern
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportDecision {
    /// Accept pattern for import
    Accept,
    /// Accept with warning message
    AcceptWithWarning(String),
    /// Reject pattern with reason
    Reject(String),
}

impl ImportDecision {
    /// Check if decision allows import
    pub fn allows_import(&self) -> bool {
        matches!(
            self,
            ImportDecision::Accept | ImportDecision::AcceptWithWarning(_)
        )
    }
}

/// Statistics for import operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImportStats {
    /// Patterns accepted by strategy
    pub accepted_by_strategy: HashMap<FixStrategy, usize>,
    /// Patterns rejected by strategy
    pub rejected_by_strategy: HashMap<FixStrategy, usize>,
    /// Patterns accepted with warnings
    pub warnings: usize,
    /// Total patterns evaluated
    pub total_evaluated: usize,
}

impl ImportStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an import decision
    pub fn record(&mut self, strategy: FixStrategy, decision: &ImportDecision) {
        self.total_evaluated += 1;
        match decision {
            ImportDecision::Accept => {
                *self.accepted_by_strategy.entry(strategy).or_insert(0) += 1;
            }
            ImportDecision::AcceptWithWarning(_) => {
                *self.accepted_by_strategy.entry(strategy).or_insert(0) += 1;
                self.warnings += 1;
            }
            ImportDecision::Reject(_) => {
                *self.rejected_by_strategy.entry(strategy).or_insert(0) += 1;
            }
        }
    }

    /// Get acceptance rate for a strategy
    pub fn acceptance_rate(&self, strategy: FixStrategy) -> f32 {
        let accepted = self
            .accepted_by_strategy
            .get(&strategy)
            .copied()
            .unwrap_or(0);
        let rejected = self
            .rejected_by_strategy
            .get(&strategy)
            .copied()
            .unwrap_or(0);
        let total = accepted + rejected;
        if total == 0 {
            0.0
        } else {
            accepted as f32 / total as f32
        }
    }

    /// Get overall acceptance rate
    pub fn overall_acceptance_rate(&self) -> f32 {
        let accepted: usize = self.accepted_by_strategy.values().sum();
        if self.total_evaluated == 0 {
            0.0
        } else {
            accepted as f32 / self.total_evaluated as f32
        }
    }
}

/// Smart import filter configuration
#[derive(Debug, Clone)]
pub struct SmartImportConfig {
    /// Source language of patterns (for context-aware filtering)
    pub source_language: SourceLanguage,
    /// Minimum confidence threshold for patterns
    pub min_confidence: f32,
    /// Allow patterns with warnings
    pub allow_warnings: bool,
}

impl Default for SmartImportConfig {
    fn default() -> Self {
        Self {
            source_language: SourceLanguage::Python,
            min_confidence: 0.5,
            allow_warnings: true,
        }
    }
}

/// Source language for imported patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceLanguage {
    Python,
    C,
    Cpp,
    Other,
}

/// Analyze fix diff to determine strategy type
pub fn analyze_fix_strategy(fix_diff: &str) -> FixStrategy {
    // Pattern matching on common fix patterns
    // Order matters: more specific patterns first

    // Clone patterns
    if fix_diff.contains(".clone()") || fix_diff.contains(".to_owned()") {
        return FixStrategy::AddClone;
    }

    // Lifetime patterns (check before borrow since 'a appears in both)
    if fix_diff.contains("<'a>")
        || fix_diff.contains("'static")
        || fix_diff.contains("'_")
        || (fix_diff.contains("'a") && fix_diff.contains("fn "))
    {
        return FixStrategy::AddLifetime;
    }

    // Borrow patterns - check for borrow additions in function signatures
    // Look for patterns like ": &String" or ": &mut Vec" or "(x: &"
    if fix_diff.contains(": &mut ")
        || fix_diff.contains(": &")
        || fix_diff.contains("(&self)")
        || fix_diff.contains("(&mut self)")
        || fix_diff.contains("(x: &")
        || fix_diff.contains("(y: &")
        || fix_diff.contains("(z: &")
        || (fix_diff.contains("&") && fix_diff.contains("+ fn"))
    {
        return FixStrategy::AddBorrow;
    }

    // Option patterns
    if fix_diff.contains("Option<")
        || fix_diff.contains("Some(")
        || fix_diff.contains(".unwrap()")
        || fix_diff.contains(".is_none()")
        || fix_diff.contains(".is_some()")
    {
        return FixStrategy::WrapInOption;
    }

    // Result patterns
    if fix_diff.contains("Result<") || fix_diff.contains("Ok(") || fix_diff.contains("Err(") {
        return FixStrategy::WrapInResult;
    }

    // Type annotation patterns (only if no borrow/option/result)
    if fix_diff.contains(": i32")
        || fix_diff.contains(": String")
        || (fix_diff.contains(": ") && !fix_diff.contains(": &"))
    {
        return FixStrategy::AddTypeAnnotation;
    }

    FixStrategy::Unknown
}

/// Evaluate whether a pattern should be imported based on fix strategy
pub fn smart_import_filter(
    fix_diff: &str,
    metadata: &HashMap<String, String>,
    config: &SmartImportConfig,
) -> ImportDecision {
    let strategy = analyze_fix_strategy(fix_diff);

    match strategy {
        FixStrategy::AddClone => {
            // Clone semantics differ: Python shallow copy vs Rust deep clone
            if config.source_language == SourceLanguage::Python {
                if let Some(construct) = metadata.get("source_construct") {
                    if construct.contains("list") || construct.contains("dict") {
                        return ImportDecision::Reject(
                            "Python collection copy != Rust clone".to_string(),
                        );
                    }
                }
            }
            ImportDecision::Accept
        }
        FixStrategy::AddBorrow => {
            // Borrow semantics are largely universal
            ImportDecision::Accept
        }
        FixStrategy::AddLifetime => {
            // Lifetime patterns transfer well
            ImportDecision::Accept
        }
        FixStrategy::WrapInOption => {
            // Python None vs C NULL have different semantics
            if config.source_language == SourceLanguage::Python {
                // Check if pattern handles C NULL pointer checks or uses idiomatic Option methods
                let has_null_handling = fix_diff.contains("NULL")
                    || fix_diff.contains("nullptr")
                    || fix_diff.contains("null")
                    || fix_diff.contains(".is_none()")
                    || fix_diff.contains(".is_some()")
                    || fix_diff.contains(".unwrap_or");

                if has_null_handling {
                    ImportDecision::Accept
                } else {
                    ImportDecision::AcceptWithWarning(
                        "Verify NULL handling for C context".to_string(),
                    )
                }
            } else {
                ImportDecision::Accept
            }
        }
        FixStrategy::WrapInResult => {
            // Error handling patterns are largely universal
            ImportDecision::Accept
        }
        FixStrategy::AddTypeAnnotation => {
            // Type annotation patterns depend on type system differences
            if config.source_language == SourceLanguage::Python {
                ImportDecision::AcceptWithWarning("Verify type mapping for C context".to_string())
            } else {
                ImportDecision::Accept
            }
        }
        FixStrategy::Unknown => ImportDecision::Reject("Unknown fix strategy".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // RED PHASE TESTS - These should FAIL until implementation is complete
    // ============================================================================

    // ============ FixStrategy Analysis Tests ============

    #[test]
    fn test_analyze_strategy_add_clone() {
        let diff = "- let x = value;\n+ let x = value.clone();";
        assert_eq!(analyze_fix_strategy(diff), FixStrategy::AddClone);
    }

    #[test]
    fn test_analyze_strategy_to_owned() {
        let diff = "- let s = str_slice;\n+ let s = str_slice.to_owned();";
        assert_eq!(analyze_fix_strategy(diff), FixStrategy::AddClone);
    }

    #[test]
    fn test_analyze_strategy_add_borrow() {
        let diff = "- fn foo(x: String)\n+ fn foo(x: &String)";
        assert_eq!(analyze_fix_strategy(diff), FixStrategy::AddBorrow);
    }

    #[test]
    fn test_analyze_strategy_add_mut_borrow() {
        let diff = "- fn foo(x: Vec<i32>)\n+ fn foo(x: &mut Vec<i32>)";
        assert_eq!(analyze_fix_strategy(diff), FixStrategy::AddBorrow);
    }

    #[test]
    fn test_analyze_strategy_add_lifetime() {
        let diff = "- fn foo(x: &str) -> &str\n+ fn foo<'a>(x: &'a str) -> &'a str";
        assert_eq!(analyze_fix_strategy(diff), FixStrategy::AddLifetime);
    }

    #[test]
    fn test_analyze_strategy_wrap_option() {
        let diff = "- let x: *const T\n+ let x: Option<&T>";
        assert_eq!(analyze_fix_strategy(diff), FixStrategy::WrapInOption);
    }

    #[test]
    fn test_analyze_strategy_wrap_result() {
        let diff = "- fn foo() -> i32\n+ fn foo() -> Result<i32, Error>";
        assert_eq!(analyze_fix_strategy(diff), FixStrategy::WrapInResult);
    }

    #[test]
    fn test_analyze_strategy_unknown() {
        let diff = "- some random change\n+ another random change";
        assert_eq!(analyze_fix_strategy(diff), FixStrategy::Unknown);
    }

    // ============ Import Decision Tests ============

    #[test]
    fn test_import_decision_allows_import() {
        assert!(ImportDecision::Accept.allows_import());
        assert!(ImportDecision::AcceptWithWarning("warning".into()).allows_import());
        assert!(!ImportDecision::Reject("reason".into()).allows_import());
    }

    // ============ Smart Import Filter Tests ============

    #[test]
    fn test_smart_filter_accepts_borrow_from_python() {
        let diff = "- fn foo(x: String)\n+ fn foo(x: &String)";
        let metadata = HashMap::new();
        let config = SmartImportConfig {
            source_language: SourceLanguage::Python,
            ..Default::default()
        };

        let decision = smart_import_filter(diff, &metadata, &config);
        assert_eq!(decision, ImportDecision::Accept);
    }

    #[test]
    fn test_smart_filter_rejects_python_list_clone() {
        let diff = "- let x = lst;\n+ let x = lst.clone();";
        let mut metadata = HashMap::new();
        metadata.insert("source_construct".into(), "list_copy".into());
        let config = SmartImportConfig {
            source_language: SourceLanguage::Python,
            ..Default::default()
        };

        let decision = smart_import_filter(diff, &metadata, &config);
        assert!(matches!(decision, ImportDecision::Reject(_)));
    }

    #[test]
    fn test_smart_filter_accepts_clone_without_list_context() {
        let diff = "- let x = value;\n+ let x = value.clone();";
        let metadata = HashMap::new();
        let config = SmartImportConfig {
            source_language: SourceLanguage::Python,
            ..Default::default()
        };

        let decision = smart_import_filter(diff, &metadata, &config);
        assert_eq!(decision, ImportDecision::Accept);
    }

    #[test]
    fn test_smart_filter_warns_on_option_without_null() {
        let diff = "- let x = value\n+ let x = Some(value)";
        let metadata = HashMap::new();
        let config = SmartImportConfig {
            source_language: SourceLanguage::Python,
            ..Default::default()
        };

        let decision = smart_import_filter(diff, &metadata, &config);
        assert!(matches!(decision, ImportDecision::AcceptWithWarning(_)));
    }

    #[test]
    fn test_smart_filter_accepts_option_with_null() {
        let diff = "- if (ptr == NULL)\n+ if ptr.is_none()";
        let metadata = HashMap::new();
        let config = SmartImportConfig {
            source_language: SourceLanguage::Python,
            ..Default::default()
        };

        let decision = smart_import_filter(diff, &metadata, &config);
        // NULL in diff means it's applicable to C context
        assert!(decision.allows_import());
    }

    #[test]
    fn test_smart_filter_rejects_unknown_strategy() {
        let diff = "random gibberish change";
        let metadata = HashMap::new();
        let config = SmartImportConfig::default();

        let decision = smart_import_filter(diff, &metadata, &config);
        assert!(matches!(decision, ImportDecision::Reject(_)));
    }

    #[test]
    fn test_smart_filter_accepts_lifetime_from_any_source() {
        let diff = "- fn foo(x: &str)\n+ fn foo<'a>(x: &'a str)";
        let metadata = HashMap::new();

        // Python source
        let config_py = SmartImportConfig {
            source_language: SourceLanguage::Python,
            ..Default::default()
        };
        assert_eq!(
            smart_import_filter(diff, &metadata, &config_py),
            ImportDecision::Accept
        );

        // C source
        let config_c = SmartImportConfig {
            source_language: SourceLanguage::C,
            ..Default::default()
        };
        assert_eq!(
            smart_import_filter(diff, &metadata, &config_c),
            ImportDecision::Accept
        );
    }

    // ============ Import Stats Tests ============

    #[test]
    fn test_import_stats_new() {
        let stats = ImportStats::new();
        assert_eq!(stats.total_evaluated, 0);
        assert_eq!(stats.warnings, 0);
    }

    #[test]
    fn test_import_stats_record_accept() {
        let mut stats = ImportStats::new();
        stats.record(FixStrategy::AddBorrow, &ImportDecision::Accept);

        assert_eq!(stats.total_evaluated, 1);
        assert_eq!(
            stats.accepted_by_strategy.get(&FixStrategy::AddBorrow),
            Some(&1)
        );
    }

    #[test]
    fn test_import_stats_record_reject() {
        let mut stats = ImportStats::new();
        stats.record(
            FixStrategy::AddClone,
            &ImportDecision::Reject("reason".into()),
        );

        assert_eq!(stats.total_evaluated, 1);
        assert_eq!(
            stats.rejected_by_strategy.get(&FixStrategy::AddClone),
            Some(&1)
        );
    }

    #[test]
    fn test_import_stats_record_warning() {
        let mut stats = ImportStats::new();
        stats.record(
            FixStrategy::WrapInOption,
            &ImportDecision::AcceptWithWarning("warning".into()),
        );

        assert_eq!(stats.total_evaluated, 1);
        assert_eq!(stats.warnings, 1);
        assert_eq!(
            stats.accepted_by_strategy.get(&FixStrategy::WrapInOption),
            Some(&1)
        );
    }

    #[test]
    fn test_import_stats_acceptance_rate() {
        let mut stats = ImportStats::new();
        // 3 accepts, 1 reject for AddBorrow
        stats.record(FixStrategy::AddBorrow, &ImportDecision::Accept);
        stats.record(FixStrategy::AddBorrow, &ImportDecision::Accept);
        stats.record(FixStrategy::AddBorrow, &ImportDecision::Accept);
        stats.record(
            FixStrategy::AddBorrow,
            &ImportDecision::Reject("reason".into()),
        );

        let rate = stats.acceptance_rate(FixStrategy::AddBorrow);
        assert!((rate - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_import_stats_overall_acceptance_rate() {
        let mut stats = ImportStats::new();
        stats.record(FixStrategy::AddBorrow, &ImportDecision::Accept);
        stats.record(FixStrategy::AddClone, &ImportDecision::Accept);
        stats.record(
            FixStrategy::Unknown,
            &ImportDecision::Reject("reason".into()),
        );

        let rate = stats.overall_acceptance_rate();
        assert!((rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_import_stats_empty_acceptance_rate() {
        let stats = ImportStats::new();
        assert_eq!(stats.acceptance_rate(FixStrategy::AddBorrow), 0.0);
        assert_eq!(stats.overall_acceptance_rate(), 0.0);
    }

    // ============ Expected Acceptance Rates from Spec ============

    #[test]
    fn test_expected_acceptance_rates_add_borrow() {
        // Spec says AddBorrow should have 95% acceptance
        // This is a property test that will guide implementation
        let mut stats = ImportStats::new();
        let config = SmartImportConfig {
            source_language: SourceLanguage::Python,
            ..Default::default()
        };

        // Simulate typical borrow patterns
        let borrow_diffs = [
            "- fn foo(x: String)\n+ fn foo(x: &String)",
            "- fn bar(y: Vec<i32>)\n+ fn bar(y: &Vec<i32>)",
            "- fn baz(z: T)\n+ fn baz(z: &mut T)",
        ];

        for diff in &borrow_diffs {
            let decision = smart_import_filter(diff, &HashMap::new(), &config);
            stats.record(FixStrategy::AddBorrow, &decision);
        }

        // All should be accepted
        assert!(
            stats.acceptance_rate(FixStrategy::AddBorrow) >= 0.95,
            "AddBorrow should have >=95% acceptance rate, got {}",
            stats.acceptance_rate(FixStrategy::AddBorrow)
        );
    }

    #[test]
    fn test_expected_acceptance_rates_add_lifetime() {
        // Spec says AddLifetime should have 90% acceptance
        let mut stats = ImportStats::new();
        let config = SmartImportConfig {
            source_language: SourceLanguage::Python,
            ..Default::default()
        };

        let lifetime_diffs = [
            "- fn foo(x: &str)\n+ fn foo<'a>(x: &'a str)",
            "- struct Foo { x: &str }\n+ struct Foo<'a> { x: &'a str }",
        ];

        for diff in &lifetime_diffs {
            let decision = smart_import_filter(diff, &HashMap::new(), &config);
            stats.record(FixStrategy::AddLifetime, &decision);
        }

        assert!(
            stats.acceptance_rate(FixStrategy::AddLifetime) >= 0.90,
            "AddLifetime should have >=90% acceptance rate"
        );
    }
}
