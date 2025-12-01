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
}
