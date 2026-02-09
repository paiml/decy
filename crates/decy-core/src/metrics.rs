//! Compile success rate metrics for transpilation pipeline.
//!
//! **Ticket**: DECY-181 - Add compile success rate metrics
//!
//! This module tracks compile success rates to measure progress toward
//! the 80% single-shot compile target.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metrics for tracking compile success rate.
///
/// Tracks the number of successful and failed compilations,
/// along with error codes for analysis.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompileMetrics {
    /// Total transpilation attempts
    total_attempts: u64,

    /// Successful first-try compilations
    successes: u64,

    /// Failed compilations
    failures: u64,

    /// Error code histogram for failure analysis
    error_counts: HashMap<String, u64>,
}

impl CompileMetrics {
    /// Create a new empty metrics collector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful compilation.
    pub fn record_success(&mut self) {
        self.total_attempts += 1;
        self.successes += 1;
    }

    /// Record a failed compilation with error message.
    ///
    /// The error message is parsed to extract the error code (e.g., "E0308").
    pub fn record_failure(&mut self, error_message: &str) {
        self.total_attempts += 1;
        self.failures += 1;

        // Extract error code from message (e.g., "E0308: mismatched types" -> "E0308")
        let error_code = extract_error_code(error_message);
        *self.error_counts.entry(error_code).or_insert(0) += 1;
    }

    /// Get total number of transpilation attempts.
    pub fn total_attempts(&self) -> u64 {
        self.total_attempts
    }

    /// Get number of successful compilations.
    pub fn successes(&self) -> u64 {
        self.successes
    }

    /// Get number of failed compilations.
    pub fn failures(&self) -> u64 {
        self.failures
    }

    /// Calculate success rate as a value between 0.0 and 1.0.
    pub fn success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            self.successes as f64 / self.total_attempts as f64
        }
    }

    /// Check if the success rate meets a target threshold.
    ///
    /// # Arguments
    /// * `target` - Target rate as a value between 0.0 and 1.0 (e.g., 0.80 for 80%)
    pub fn meets_target(&self, target: f64) -> bool {
        self.success_rate() >= target
    }

    /// Get the error code histogram for failure analysis.
    pub fn error_histogram(&self) -> &HashMap<String, u64> {
        &self.error_counts
    }

    /// Reset all metrics to zero.
    pub fn reset(&mut self) {
        self.total_attempts = 0;
        self.successes = 0;
        self.failures = 0;
        self.error_counts.clear();
    }

    /// Generate a markdown report of the metrics.
    pub fn to_markdown(&self) -> String {
        let rate_pct = self.success_rate() * 100.0;
        let target_status = if self.meets_target(0.80) {
            "✅ PASS"
        } else {
            "❌ FAIL"
        };

        let mut report = format!(
            "## Compile Success Rate Metrics\n\n\
             | Metric | Value |\n\
             |--------|-------|\n\
             | Total Attempts | {} |\n\
             | Successes | {} |\n\
             | Failures | {} |\n\
             | Success Rate | {:.1}% |\n\
             | Target (80%) | {} |\n",
            self.total_attempts, self.successes, self.failures, rate_pct, target_status
        );

        if !self.error_counts.is_empty() {
            report.push_str("\n### Error Breakdown\n\n");
            report.push_str("| Error Code | Count |\n");
            report.push_str("|------------|-------|\n");

            let mut sorted_errors: Vec<_> = self.error_counts.iter().collect();
            sorted_errors.sort_by(|a, b| b.1.cmp(a.1));

            for (code, count) in sorted_errors {
                report.push_str(&format!("| {} | {} |\n", code, count));
            }
        }

        report
    }

    /// Serialize metrics to JSON for CI integration.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Extract error code from Rust compiler error message.
///
/// # Examples
/// - "E0308: mismatched types" -> "E0308"
/// - "error[E0502]: cannot borrow" -> "E0502"
/// - "some unknown error" -> "UNKNOWN"
fn extract_error_code(message: &str) -> String {
    // Pattern: E followed by 4 digits
    if let Some(start) = message.find('E') {
        let rest = &message[start..];
        if rest.len() >= 5 && rest[1..5].chars().all(|c| c.is_ascii_digit()) {
            return rest[..5].to_string();
        }
    }

    // Try pattern with brackets: error[E0308]
    if let Some(bracket_start) = message.find("[E") {
        let rest = &message[bracket_start + 1..];
        if let Some(bracket_end) = rest.find(']') {
            return rest[..bracket_end].to_string();
        }
    }

    "UNKNOWN".to_string()
}

/// Result of transpilation with verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranspilationResult {
    /// Generated Rust code
    pub rust_code: String,

    /// Whether the generated code compiles
    pub compiles: bool,

    /// Compilation errors if any
    pub errors: Vec<String>,

    /// Clippy warnings count
    pub warnings: usize,
}

impl TranspilationResult {
    /// Create a successful result.
    pub fn success(rust_code: String) -> Self {
        Self {
            rust_code,
            compiles: true,
            errors: Vec::new(),
            warnings: 0,
        }
    }

    /// Create a failed result with errors.
    pub fn failure(rust_code: String, errors: Vec<String>) -> Self {
        Self {
            rust_code,
            compiles: false,
            errors,
            warnings: 0,
        }
    }
}

/// DECY-191: Per-tier metrics for corpus convergence measurement.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TierMetrics {
    /// Tier name (e.g., "chapter-1", "P0", "P1")
    pub tier_name: String,
    /// Number of C files in this tier
    pub total_files: u64,
    /// Files that transpiled successfully
    pub transpile_success: u64,
    /// Files that compiled after transpilation
    pub compile_success: u64,
    /// Files that failed transpilation
    pub transpile_failures: u64,
    /// Files that transpiled but failed compilation
    pub compile_failures: u64,
}

impl TierMetrics {
    /// Create new tier metrics for a named tier.
    pub fn new(name: &str) -> Self {
        Self {
            tier_name: name.to_string(),
            ..Default::default()
        }
    }

    /// Transpilation success rate (0.0 to 1.0).
    pub fn transpile_rate(&self) -> f64 {
        if self.total_files == 0 {
            0.0
        } else {
            self.transpile_success as f64 / self.total_files as f64
        }
    }

    /// Compile success rate (0.0 to 1.0).
    pub fn compile_rate(&self) -> f64 {
        if self.total_files == 0 {
            0.0
        } else {
            self.compile_success as f64 / self.total_files as f64
        }
    }
}

/// DECY-191: Convergence report across all tiers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConvergenceReport {
    /// Per-tier metrics
    pub tiers: Vec<TierMetrics>,
}

impl ConvergenceReport {
    /// Create a new empty convergence report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add tier metrics to the report.
    pub fn add_tier(&mut self, tier: TierMetrics) {
        self.tiers.push(tier);
    }

    /// Overall transpilation rate across all tiers.
    pub fn overall_transpile_rate(&self) -> f64 {
        let total: u64 = self.tiers.iter().map(|t| t.total_files).sum();
        let success: u64 = self.tiers.iter().map(|t| t.transpile_success).sum();
        if total == 0 {
            0.0
        } else {
            success as f64 / total as f64
        }
    }

    /// Overall compile rate across all tiers.
    pub fn overall_compile_rate(&self) -> f64 {
        let total: u64 = self.tiers.iter().map(|t| t.total_files).sum();
        let success: u64 = self.tiers.iter().map(|t| t.compile_success).sum();
        if total == 0 {
            0.0
        } else {
            success as f64 / total as f64
        }
    }

    /// Generate a markdown table of convergence results.
    pub fn to_markdown(&self) -> String {
        let mut report = String::new();
        report.push_str("## Corpus Convergence Report\n\n");
        report.push_str("| Tier | Files | Transpile | Compile | Transpile Rate | Compile Rate |\n");
        report.push_str("|------|-------|-----------|---------|----------------|-------------|\n");

        for tier in &self.tiers {
            report.push_str(&format!(
                "| {} | {} | {} | {} | {:.1}% | {:.1}% |\n",
                tier.tier_name,
                tier.total_files,
                tier.transpile_success,
                tier.compile_success,
                tier.transpile_rate() * 100.0,
                tier.compile_rate() * 100.0,
            ));
        }

        let total_files: u64 = self.tiers.iter().map(|t| t.total_files).sum();
        let total_transpile: u64 = self.tiers.iter().map(|t| t.transpile_success).sum();
        let total_compile: u64 = self.tiers.iter().map(|t| t.compile_success).sum();

        report.push_str(&format!(
            "| **Total** | **{}** | **{}** | **{}** | **{:.1}%** | **{:.1}%** |\n",
            total_files,
            total_transpile,
            total_compile,
            self.overall_transpile_rate() * 100.0,
            self.overall_compile_rate() * 100.0,
        ));

        report
    }
}

/// DECY-195: Record of a semantic equivalence divergence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceDivergence {
    /// Path to the C file
    pub file: String,
    /// Expected output (from gcc)
    pub expected_stdout: String,
    /// Actual output (from transpiled Rust)
    pub actual_stdout: String,
    /// Expected exit code (from gcc)
    pub expected_exit: i32,
    /// Actual exit code (from transpiled Rust)
    pub actual_exit: i32,
}

/// DECY-195: Metrics for semantic equivalence validation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EquivalenceMetrics {
    /// Number of files tested
    pub total_files: u64,
    /// Files with matching output and exit code
    pub equivalent: u64,
    /// Files with divergent behavior
    pub divergent: u64,
    /// Files that could not be compiled (either C or Rust)
    pub errors: u64,
    /// Detailed divergence records
    pub divergences: Vec<EquivalenceDivergence>,
}

impl EquivalenceMetrics {
    /// Create new empty equivalence metrics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Equivalence rate (0.0 to 1.0).
    pub fn equivalence_rate(&self) -> f64 {
        if self.total_files == 0 {
            0.0
        } else {
            self.equivalent as f64 / self.total_files as f64
        }
    }

    /// Generate a markdown report.
    pub fn to_markdown(&self) -> String {
        let mut report = String::new();
        report.push_str("## Semantic Equivalence Report\n\n");
        report.push_str(&format!(
            "| Metric | Value |\n\
             |--------|-------|\n\
             | Total Files | {} |\n\
             | Equivalent | {} |\n\
             | Divergent | {} |\n\
             | Errors | {} |\n\
             | Equivalence Rate | {:.1}% |\n",
            self.total_files,
            self.equivalent,
            self.divergent,
            self.errors,
            self.equivalence_rate() * 100.0,
        ));

        if !self.divergences.is_empty() {
            report.push_str("\n### Divergences\n\n");
            for d in &self.divergences {
                report.push_str(&format!(
                    "- **{}**: exit {} vs {} | stdout differs: {}\n",
                    d.file,
                    d.expected_exit,
                    d.actual_exit,
                    d.expected_stdout != d.actual_stdout,
                ));
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_error_code_standard() {
        assert_eq!(extract_error_code("E0308: mismatched types"), "E0308");
    }

    #[test]
    fn test_extract_error_code_with_brackets() {
        assert_eq!(extract_error_code("error[E0502]: cannot borrow"), "E0502");
    }

    #[test]
    fn test_extract_error_code_unknown() {
        assert_eq!(extract_error_code("some random error"), "UNKNOWN");
    }

    #[test]
    fn test_extract_error_code_partial() {
        assert_eq!(extract_error_code("E03"), "UNKNOWN");
    }

    // TierMetrics tests

    #[test]
    fn test_tier_metrics_new() {
        let tier = TierMetrics::new("chapter-1");
        assert_eq!(tier.tier_name, "chapter-1");
        assert_eq!(tier.total_files, 0);
        assert_eq!(tier.transpile_success, 0);
    }

    #[test]
    fn test_tier_metrics_transpile_rate_empty() {
        let tier = TierMetrics::new("empty");
        assert_eq!(tier.transpile_rate(), 0.0);
    }

    #[test]
    fn test_tier_metrics_transpile_rate() {
        let mut tier = TierMetrics::new("test");
        tier.total_files = 10;
        tier.transpile_success = 8;
        assert!((tier.transpile_rate() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_tier_metrics_compile_rate_empty() {
        let tier = TierMetrics::new("empty");
        assert_eq!(tier.compile_rate(), 0.0);
    }

    #[test]
    fn test_tier_metrics_compile_rate() {
        let mut tier = TierMetrics::new("test");
        tier.total_files = 10;
        tier.compile_success = 7;
        assert!((tier.compile_rate() - 0.7).abs() < 0.001);
    }

    // ConvergenceReport tests

    #[test]
    fn test_convergence_report_new_empty() {
        let report = ConvergenceReport::new();
        assert!(report.tiers.is_empty());
        assert_eq!(report.overall_transpile_rate(), 0.0);
        assert_eq!(report.overall_compile_rate(), 0.0);
    }

    #[test]
    fn test_convergence_report_add_tier() {
        let mut report = ConvergenceReport::new();
        let mut tier = TierMetrics::new("tier1");
        tier.total_files = 10;
        tier.transpile_success = 9;
        tier.compile_success = 7;
        report.add_tier(tier);
        assert_eq!(report.tiers.len(), 1);
    }

    #[test]
    fn test_convergence_report_overall_rates() {
        let mut report = ConvergenceReport::new();
        let mut t1 = TierMetrics::new("t1");
        t1.total_files = 10;
        t1.transpile_success = 8;
        t1.compile_success = 6;
        let mut t2 = TierMetrics::new("t2");
        t2.total_files = 10;
        t2.transpile_success = 10;
        t2.compile_success = 8;
        report.add_tier(t1);
        report.add_tier(t2);

        assert!((report.overall_transpile_rate() - 0.9).abs() < 0.001);
        assert!((report.overall_compile_rate() - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_convergence_report_to_markdown() {
        let mut report = ConvergenceReport::new();
        let mut tier = TierMetrics::new("chapter-1");
        tier.total_files = 20;
        tier.transpile_success = 18;
        tier.compile_success = 15;
        report.add_tier(tier);

        let md = report.to_markdown();
        assert!(md.contains("Corpus Convergence Report"));
        assert!(md.contains("chapter-1"));
        assert!(md.contains("20"));
        assert!(md.contains("18"));
        assert!(md.contains("15"));
        assert!(md.contains("Total"));
    }

    // EquivalenceMetrics tests

    #[test]
    fn test_equivalence_metrics_new_empty() {
        let metrics = EquivalenceMetrics::new();
        assert_eq!(metrics.total_files, 0);
        assert_eq!(metrics.equivalence_rate(), 0.0);
    }

    #[test]
    fn test_equivalence_metrics_rate() {
        let mut metrics = EquivalenceMetrics::new();
        metrics.total_files = 20;
        metrics.equivalent = 18;
        metrics.divergent = 2;
        assert!((metrics.equivalence_rate() - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_equivalence_metrics_to_markdown() {
        let mut metrics = EquivalenceMetrics::new();
        metrics.total_files = 10;
        metrics.equivalent = 8;
        metrics.divergent = 1;
        metrics.errors = 1;

        let md = metrics.to_markdown();
        assert!(md.contains("Semantic Equivalence Report"));
        assert!(md.contains("10"));
        assert!(md.contains("80.0%"));
    }

    #[test]
    fn test_equivalence_metrics_to_markdown_with_divergences() {
        let mut metrics = EquivalenceMetrics::new();
        metrics.total_files = 5;
        metrics.equivalent = 4;
        metrics.divergent = 1;
        metrics.errors = 0;
        metrics.divergences.push(EquivalenceDivergence {
            file: "test.c".to_string(),
            expected_stdout: "hello".to_string(),
            actual_stdout: "world".to_string(),
            expected_exit: 0,
            actual_exit: 1,
        });

        let md = metrics.to_markdown();
        assert!(md.contains("Divergences"));
        assert!(md.contains("test.c"));
        assert!(md.contains("exit 0 vs 1"));
    }

    // CompileMetrics tests for uncovered paths

    #[test]
    fn test_compile_metrics_new_empty() {
        let metrics = CompileMetrics::new();
        assert_eq!(metrics.success_rate(), 0.0);
        assert_eq!(metrics.total_attempts(), 0);
        assert!(metrics.error_histogram().is_empty());
    }

    #[test]
    fn test_compile_metrics_record_success() {
        let mut metrics = CompileMetrics::new();
        metrics.record_success();
        metrics.record_success();
        assert!((metrics.success_rate() - 1.0).abs() < 0.001);
        assert_eq!(metrics.successes(), 2);
    }

    #[test]
    fn test_compile_metrics_record_failure() {
        let mut metrics = CompileMetrics::new();
        metrics.record_success();
        metrics.record_failure("E0308: mismatched types");
        metrics.record_failure("E0502: cannot borrow");
        assert!((metrics.success_rate() - (1.0 / 3.0)).abs() < 0.01);
        assert_eq!(metrics.failures(), 2);
    }

    #[test]
    fn test_compile_metrics_error_histogram() {
        let mut metrics = CompileMetrics::new();
        metrics.record_failure("E0308: mismatched");
        metrics.record_failure("E0308: mismatched again");
        metrics.record_failure("E0502: borrow");
        let hist = metrics.error_histogram();
        assert_eq!(hist.get("E0308"), Some(&2));
        assert_eq!(hist.get("E0502"), Some(&1));
    }

    #[test]
    fn test_compile_metrics_meets_target() {
        let mut metrics = CompileMetrics::new();
        for _ in 0..8 {
            metrics.record_success();
        }
        for _ in 0..2 {
            metrics.record_failure("E0308: test");
        }
        assert!(metrics.meets_target(0.80));
        assert!(!metrics.meets_target(0.90));
    }

    #[test]
    fn test_compile_metrics_to_markdown() {
        let mut metrics = CompileMetrics::new();
        metrics.record_success();
        metrics.record_failure("E0308: test");
        let md = metrics.to_markdown();
        assert!(md.contains("Compile Success Rate"));
        assert!(md.contains("50.0%"));
        assert!(md.contains("E0308"));
    }

    #[test]
    fn test_compile_metrics_to_markdown_passing() {
        let mut metrics = CompileMetrics::new();
        for _ in 0..10 {
            metrics.record_success();
        }
        let md = metrics.to_markdown();
        assert!(md.contains("PASS"));
        assert!(md.contains("100.0%"));
    }

    #[test]
    fn test_compile_metrics_reset() {
        let mut metrics = CompileMetrics::new();
        metrics.record_success();
        metrics.record_failure("E0308: test");
        metrics.reset();
        assert_eq!(metrics.total_attempts(), 0);
        assert_eq!(metrics.successes(), 0);
        assert_eq!(metrics.failures(), 0);
        assert!(metrics.error_histogram().is_empty());
    }
}
