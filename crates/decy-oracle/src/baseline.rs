//! Baseline measurement for ownership inference quality (DECY-ML-005).
//!
//! Measures "compiles on first try" rate with 95% confidence interval
//! against a test corpus of C files.

use serde::{Deserialize, Serialize};

/// Baseline measurement results.
///
/// Records the "compiles on first try" rate and confidence interval
/// from running transpilation on a test corpus.
///
/// # Example
///
/// ```
/// use decy_oracle::baseline::BaselineMetrics;
///
/// let metrics = BaselineMetrics::new(85, 100);
/// assert_eq!(metrics.first_try_rate(), 0.85);
/// assert!(metrics.confidence_interval().0 < 0.85);
/// assert!(metrics.confidence_interval().1 > 0.85);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetrics {
    /// Total files in corpus
    corpus_size: u64,
    /// Files that compiled on first try
    first_try_successes: u64,
    /// Files that eventually compiled (with iterations)
    eventual_successes: u64,
    /// Average iterations needed for success
    average_iterations: f64,
    /// 95% confidence interval lower bound
    ci_lower: f64,
    /// 95% confidence interval upper bound
    ci_upper: f64,
}

impl BaselineMetrics {
    /// Target "compiles on first try" rate (85%).
    pub const TARGET_RATE: f64 = 0.85;

    /// Create new baseline metrics from success count and total.
    pub fn new(first_try_successes: u64, corpus_size: u64) -> Self {
        let (ci_lower, ci_upper) = wilson_score_interval(first_try_successes, corpus_size, 0.95);

        Self {
            corpus_size,
            first_try_successes,
            eventual_successes: first_try_successes, // Default to same
            average_iterations: 1.0,
            ci_lower,
            ci_upper,
        }
    }

    /// Create metrics with full iteration data.
    pub fn with_iterations(
        first_try_successes: u64,
        eventual_successes: u64,
        total_iterations: u64,
        corpus_size: u64,
    ) -> Self {
        let (ci_lower, ci_upper) = wilson_score_interval(first_try_successes, corpus_size, 0.95);

        let average_iterations = if eventual_successes == 0 {
            0.0
        } else {
            total_iterations as f64 / eventual_successes as f64
        };

        Self {
            corpus_size,
            first_try_successes,
            eventual_successes,
            average_iterations,
            ci_lower,
            ci_upper,
        }
    }

    /// Get corpus size.
    pub fn corpus_size(&self) -> u64 {
        self.corpus_size
    }

    /// Get number of first-try successes.
    pub fn first_try_successes(&self) -> u64 {
        self.first_try_successes
    }

    /// Get number of eventual successes.
    pub fn eventual_successes(&self) -> u64 {
        self.eventual_successes
    }

    /// Calculate "compiles on first try" rate (0.0 - 1.0).
    pub fn first_try_rate(&self) -> f64 {
        if self.corpus_size == 0 {
            return 0.0;
        }
        self.first_try_successes as f64 / self.corpus_size as f64
    }

    /// Get 95% confidence interval as (lower, upper).
    pub fn confidence_interval(&self) -> (f64, f64) {
        (self.ci_lower, self.ci_upper)
    }

    /// Get average iterations needed for success.
    pub fn average_iterations(&self) -> f64 {
        self.average_iterations
    }

    /// Check if first-try rate meets target.
    pub fn meets_target(&self) -> bool {
        self.first_try_rate() >= Self::TARGET_RATE
    }

    /// Check if confidence interval excludes target (significantly below).
    pub fn significantly_below_target(&self) -> bool {
        self.ci_upper < Self::TARGET_RATE
    }

    /// Check if confidence interval includes target (not significantly different).
    pub fn includes_target(&self) -> bool {
        self.ci_lower <= Self::TARGET_RATE && self.ci_upper >= Self::TARGET_RATE
    }

    /// Format as markdown report.
    pub fn to_markdown(&self) -> String {
        let status = if self.meets_target() {
            "PASSED"
        } else if self.significantly_below_target() {
            "FAILED (significantly below target)"
        } else {
            "PENDING (includes target in CI)"
        };

        format!(
            r#"## Baseline Measurement Report

| Metric | Value |
|--------|-------|
| Corpus Size | {} |
| First-Try Successes | {} |
| First-Try Rate | {:.1}% |
| 95% CI | [{:.1}%, {:.1}%] |
| Target Rate | {:.1}% |
| Average Iterations | {:.2} |

### Status: {}
"#,
            self.corpus_size,
            self.first_try_successes,
            self.first_try_rate() * 100.0,
            self.ci_lower * 100.0,
            self.ci_upper * 100.0,
            Self::TARGET_RATE * 100.0,
            self.average_iterations,
            status
        )
    }
}

/// Calculate Wilson score confidence interval for a proportion.
///
/// This is more accurate than the normal approximation for small samples
/// or extreme proportions (near 0 or 1).
///
/// # Arguments
/// * `successes` - Number of successes
/// * `total` - Total trials
/// * `confidence` - Confidence level (e.g., 0.95 for 95%)
///
/// # Returns
/// Tuple of (lower_bound, upper_bound)
pub fn wilson_score_interval(successes: u64, total: u64, confidence: f64) -> (f64, f64) {
    if total == 0 {
        return (0.0, 1.0);
    }

    let n = total as f64;
    let p = successes as f64 / n;

    // Z-score for confidence level (using 1.96 for 95%)
    let z = match confidence {
        c if (c - 0.90).abs() < 0.01 => 1.645,
        c if (c - 0.95).abs() < 0.01 => 1.96,
        c if (c - 0.99).abs() < 0.01 => 2.576,
        _ => 1.96, // Default to 95%
    };

    let z2 = z * z;

    // Wilson score formula
    let denominator = 1.0 + z2 / n;
    let center = (p + z2 / (2.0 * n)) / denominator;
    let margin = (z / denominator) * ((p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt());

    let lower = (center - margin).max(0.0);
    let upper = (center + margin).min(1.0);

    (lower, upper)
}

/// Result of measuring a single file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMeasurement {
    /// File path
    pub path: String,
    /// Whether compilation succeeded on first try
    pub first_try_success: bool,
    /// Whether compilation eventually succeeded
    pub eventual_success: bool,
    /// Number of iterations needed
    pub iterations: u32,
    /// Error codes encountered (if any)
    pub error_codes: Vec<String>,
}

impl FileMeasurement {
    /// Create a successful first-try measurement.
    pub fn first_try_success(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            first_try_success: true,
            eventual_success: true,
            iterations: 1,
            error_codes: Vec::new(),
        }
    }

    /// Create a measurement that succeeded after iterations.
    pub fn success_after(path: impl Into<String>, iterations: u32, errors: Vec<String>) -> Self {
        Self {
            path: path.into(),
            first_try_success: iterations == 1,
            eventual_success: true,
            iterations,
            error_codes: errors,
        }
    }

    /// Create a failed measurement.
    pub fn failure(path: impl Into<String>, iterations: u32, errors: Vec<String>) -> Self {
        Self {
            path: path.into(),
            first_try_success: false,
            eventual_success: false,
            iterations,
            error_codes: errors,
        }
    }
}

/// Aggregate file measurements into baseline metrics.
pub fn aggregate_measurements(measurements: &[FileMeasurement]) -> BaselineMetrics {
    let corpus_size = measurements.len() as u64;

    let first_try_successes = measurements
        .iter()
        .filter(|m| m.first_try_success)
        .count() as u64;

    let eventual_successes = measurements.iter().filter(|m| m.eventual_success).count() as u64;

    let total_iterations: u64 = measurements
        .iter()
        .filter(|m| m.eventual_success)
        .map(|m| m.iterations as u64)
        .sum();

    BaselineMetrics::with_iterations(
        first_try_successes,
        eventual_successes,
        total_iterations,
        corpus_size,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // BaselineMetrics tests
    // ========================================================================

    #[test]
    fn baseline_metrics_new() {
        let metrics = BaselineMetrics::new(85, 100);
        assert_eq!(metrics.corpus_size(), 100);
        assert_eq!(metrics.first_try_successes(), 85);
    }

    #[test]
    fn baseline_metrics_first_try_rate() {
        let metrics = BaselineMetrics::new(85, 100);
        assert!((metrics.first_try_rate() - 0.85).abs() < 0.001);
    }

    #[test]
    fn baseline_metrics_empty_corpus() {
        let metrics = BaselineMetrics::new(0, 0);
        assert_eq!(metrics.first_try_rate(), 0.0);
    }

    #[test]
    fn baseline_metrics_meets_target() {
        let passing = BaselineMetrics::new(85, 100);
        assert!(passing.meets_target());

        let failing = BaselineMetrics::new(80, 100);
        assert!(!failing.meets_target());
    }

    #[test]
    fn baseline_metrics_confidence_interval_exists() {
        let metrics = BaselineMetrics::new(85, 100);
        let (lower, upper) = metrics.confidence_interval();

        // CI should bracket the point estimate
        assert!(lower < 0.85);
        assert!(upper > 0.85);
        assert!(lower >= 0.0);
        assert!(upper <= 1.0);
    }

    #[test]
    fn baseline_metrics_ci_narrows_with_larger_samples() {
        let small = BaselineMetrics::new(17, 20);
        let large = BaselineMetrics::new(850, 1000);

        let (small_lo, small_hi) = small.confidence_interval();
        let (large_lo, large_hi) = large.confidence_interval();

        let small_width = small_hi - small_lo;
        let large_width = large_hi - large_lo;

        // Larger sample should have narrower CI
        assert!(large_width < small_width);
    }

    #[test]
    fn baseline_metrics_significantly_below_target() {
        // Very low rate with large sample - CI upper bound below target
        let metrics = BaselineMetrics::new(500, 1000);
        assert!(metrics.significantly_below_target());

        // Rate that includes target
        let close = BaselineMetrics::new(840, 1000);
        assert!(!close.significantly_below_target());
    }

    #[test]
    fn baseline_metrics_with_iterations() {
        let metrics = BaselineMetrics::with_iterations(80, 95, 150, 100);

        assert_eq!(metrics.first_try_successes(), 80);
        assert_eq!(metrics.eventual_successes(), 95);
        assert!((metrics.average_iterations() - 1.578).abs() < 0.01);
    }

    #[test]
    fn baseline_metrics_to_markdown() {
        let metrics = BaselineMetrics::new(85, 100);
        let md = metrics.to_markdown();

        assert!(md.contains("Baseline Measurement Report"));
        assert!(md.contains("| Corpus Size | 100 |"));
        assert!(md.contains("| First-Try Successes | 85 |"));
        assert!(md.contains("PASSED"));
    }

    // ========================================================================
    // Wilson score interval tests
    // ========================================================================

    #[test]
    fn wilson_score_empty() {
        let (lower, upper) = wilson_score_interval(0, 0, 0.95);
        assert_eq!(lower, 0.0);
        assert_eq!(upper, 1.0);
    }

    #[test]
    fn wilson_score_all_success() {
        let (lower, upper) = wilson_score_interval(100, 100, 0.95);
        assert!(lower > 0.95);
        assert!((upper - 1.0).abs() < 1e-10);
    }

    #[test]
    fn wilson_score_all_failure() {
        let (lower, upper) = wilson_score_interval(0, 100, 0.95);
        assert_eq!(lower, 0.0);
        assert!(upper < 0.05);
    }

    #[test]
    fn wilson_score_typical_case() {
        // 85% success rate with 100 samples
        let (lower, upper) = wilson_score_interval(85, 100, 0.95);

        // Should be roughly [0.77, 0.91]
        assert!(lower > 0.75 && lower < 0.80);
        assert!(upper > 0.89 && upper < 0.93);
    }

    // ========================================================================
    // FileMeasurement tests
    // ========================================================================

    #[test]
    fn file_measurement_first_try_success() {
        let m = FileMeasurement::first_try_success("test.c");
        assert!(m.first_try_success);
        assert!(m.eventual_success);
        assert_eq!(m.iterations, 1);
        assert!(m.error_codes.is_empty());
    }

    #[test]
    fn file_measurement_success_after_iterations() {
        let m = FileMeasurement::success_after("test.c", 3, vec!["E0382".to_string()]);
        assert!(!m.first_try_success);
        assert!(m.eventual_success);
        assert_eq!(m.iterations, 3);
        assert_eq!(m.error_codes.len(), 1);
    }

    #[test]
    fn file_measurement_failure() {
        let m = FileMeasurement::failure("test.c", 5, vec!["E0382".to_string(), "E0499".to_string()]);
        assert!(!m.first_try_success);
        assert!(!m.eventual_success);
        assert_eq!(m.iterations, 5);
        assert_eq!(m.error_codes.len(), 2);
    }

    // ========================================================================
    // Aggregation tests
    // ========================================================================

    #[test]
    fn aggregate_empty() {
        let metrics = aggregate_measurements(&[]);
        assert_eq!(metrics.corpus_size(), 0);
        assert_eq!(metrics.first_try_rate(), 0.0);
    }

    #[test]
    fn aggregate_all_first_try() {
        let measurements = vec![
            FileMeasurement::first_try_success("a.c"),
            FileMeasurement::first_try_success("b.c"),
            FileMeasurement::first_try_success("c.c"),
        ];
        let metrics = aggregate_measurements(&measurements);

        assert_eq!(metrics.corpus_size(), 3);
        assert_eq!(metrics.first_try_successes(), 3);
        assert!((metrics.first_try_rate() - 1.0).abs() < 0.001);
    }

    #[test]
    fn aggregate_mixed_results() {
        let measurements = vec![
            FileMeasurement::first_try_success("a.c"),
            FileMeasurement::first_try_success("b.c"),
            FileMeasurement::success_after("c.c", 2, vec!["E0382".to_string()]),
            FileMeasurement::success_after("d.c", 3, vec!["E0499".to_string()]),
            FileMeasurement::failure("e.c", 5, vec!["E0515".to_string()]),
        ];
        let metrics = aggregate_measurements(&measurements);

        assert_eq!(metrics.corpus_size(), 5);
        assert_eq!(metrics.first_try_successes(), 2);
        assert_eq!(metrics.eventual_successes(), 4);
        assert!((metrics.first_try_rate() - 0.4).abs() < 0.001);
        // (1 + 1 + 2 + 3) / 4 = 1.75
        assert!((metrics.average_iterations() - 1.75).abs() < 0.001);
    }

    #[test]
    fn aggregate_all_failures() {
        let measurements = vec![
            FileMeasurement::failure("a.c", 5, vec![]),
            FileMeasurement::failure("b.c", 5, vec![]),
        ];
        let metrics = aggregate_measurements(&measurements);

        assert_eq!(metrics.corpus_size(), 2);
        assert_eq!(metrics.first_try_successes(), 0);
        assert_eq!(metrics.eventual_successes(), 0);
        assert!((metrics.first_try_rate() - 0.0).abs() < 0.001);
        assert!((metrics.average_iterations() - 0.0).abs() < 0.001);
    }
}
