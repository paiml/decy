//! Oracle metrics for observability

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metrics for oracle usage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OracleMetrics {
    /// Total queries made
    pub queries: u64,

    /// Queries with suggestions (hits)
    pub hits: u64,

    /// Queries without suggestions (misses)
    pub misses: u64,

    /// Fixes successfully applied
    pub fixes_applied: u64,

    /// Fixes that compiled successfully
    pub fixes_verified: u64,

    /// Patterns captured for learning
    pub patterns_captured: u64,

    /// Per-error-code breakdown
    pub by_error_code: HashMap<String, ErrorCodeMetrics>,
}

/// Per-error-code metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorCodeMetrics {
    pub queries: u64,
    pub hits: u64,
    pub fixes_applied: u64,
    pub fixes_verified: u64,
}

impl OracleMetrics {
    /// Calculate hit rate (0.0-1.0)
    pub fn hit_rate(&self) -> f64 {
        if self.queries == 0 {
            0.0
        } else {
            self.hits as f64 / self.queries as f64
        }
    }

    /// Calculate fix success rate (0.0-1.0)
    pub fn fix_success_rate(&self) -> f64 {
        if self.fixes_applied == 0 {
            0.0
        } else {
            self.fixes_verified as f64 / self.fixes_applied as f64
        }
    }

    /// Record a query hit
    pub fn record_hit(&mut self, error_code: &str) {
        self.queries += 1;
        self.hits += 1;
        self.by_error_code
            .entry(error_code.to_string())
            .or_default()
            .queries += 1;
        self.by_error_code
            .entry(error_code.to_string())
            .or_default()
            .hits += 1;
    }

    /// Record a query miss
    pub fn record_miss(&mut self, error_code: &str) {
        self.queries += 1;
        self.misses += 1;
        self.by_error_code
            .entry(error_code.to_string())
            .or_default()
            .queries += 1;
    }

    /// Record a fix application
    pub fn record_fix_applied(&mut self, error_code: &str) {
        self.fixes_applied += 1;
        self.by_error_code
            .entry(error_code.to_string())
            .or_default()
            .fixes_applied += 1;
    }

    /// Record a verified fix
    pub fn record_fix_verified(&mut self, error_code: &str) {
        self.fixes_verified += 1;
        self.by_error_code
            .entry(error_code.to_string())
            .or_default()
            .fixes_verified += 1;
    }

    /// Record a pattern capture
    pub fn record_pattern_captured(&mut self) {
        self.patterns_captured += 1;
    }

    /// Export as Prometheus metrics
    pub fn to_prometheus(&self) -> String {
        format!(
            r#"# HELP decy_oracle_queries_total Total oracle queries
# TYPE decy_oracle_queries_total counter
decy_oracle_queries_total {}

# HELP decy_oracle_hits_total Oracle hits
# TYPE decy_oracle_hits_total counter
decy_oracle_hits_total {}

# HELP decy_oracle_hit_rate Current hit rate
# TYPE decy_oracle_hit_rate gauge
decy_oracle_hit_rate {}

# HELP decy_oracle_fixes_applied_total Fixes applied
# TYPE decy_oracle_fixes_applied_total counter
decy_oracle_fixes_applied_total {}

# HELP decy_oracle_fixes_verified_total Fixes verified
# TYPE decy_oracle_fixes_verified_total counter
decy_oracle_fixes_verified_total {}

# HELP decy_oracle_fix_success_rate Fix success rate
# TYPE decy_oracle_fix_success_rate gauge
decy_oracle_fix_success_rate {}
"#,
            self.queries,
            self.hits,
            self.hit_rate(),
            self.fixes_applied,
            self.fixes_verified,
            self.fix_success_rate()
        )
    }

    /// Export as JSON (for CI integration)
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Check if metrics meet CI thresholds
    pub fn meets_ci_thresholds(&self, min_hit_rate: f64, min_fix_rate: f64) -> bool {
        // No queries = passes (nothing to measure)
        if self.queries == 0 {
            return true;
        }
        // Check hit rate threshold
        if self.hit_rate() < min_hit_rate {
            return false;
        }
        // Only check fix rate if fixes were attempted
        if self.fixes_applied > 0 && self.fix_success_rate() < min_fix_rate {
            return false;
        }
        true
    }
}

/// CI report output format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CIReport {
    /// Summary metrics
    pub metrics: OracleMetrics,
    /// Hit rate percentage
    pub hit_rate_pct: f64,
    /// Fix success rate percentage
    pub fix_success_rate_pct: f64,
    /// Whether CI thresholds were met
    pub passed: bool,
    /// Threshold configuration
    pub thresholds: CIThresholds,
}

/// CI threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CIThresholds {
    pub min_hit_rate: f64,
    pub min_fix_rate: f64,
}

impl Default for CIThresholds {
    fn default() -> Self {
        Self {
            min_hit_rate: 0.5, // 50% hit rate
            min_fix_rate: 0.8, // 80% fix success rate
        }
    }
}

impl CIReport {
    /// Create a CI report from metrics
    pub fn from_metrics(metrics: OracleMetrics, thresholds: CIThresholds) -> Self {
        let passed = metrics.meets_ci_thresholds(thresholds.min_hit_rate, thresholds.min_fix_rate);
        Self {
            hit_rate_pct: metrics.hit_rate() * 100.0,
            fix_success_rate_pct: metrics.fix_success_rate() * 100.0,
            metrics,
            passed,
            thresholds,
        }
    }

    /// Export as JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Export as markdown
    pub fn to_markdown(&self) -> String {
        format!(
            r#"## Oracle CI Report

| Metric | Value |
|--------|-------|
| Queries | {} |
| Hits | {} |
| Misses | {} |
| Hit Rate | {:.1}% |
| Fixes Applied | {} |
| Fixes Verified | {} |
| Fix Success Rate | {:.1}% |
| Patterns Captured | {} |

### Status: {}

Thresholds: Hit Rate >= {:.0}%, Fix Rate >= {:.0}%
"#,
            self.metrics.queries,
            self.metrics.hits,
            self.metrics.misses,
            self.hit_rate_pct,
            self.metrics.fixes_applied,
            self.metrics.fixes_verified,
            self.fix_success_rate_pct,
            self.metrics.patterns_captured,
            if self.passed {
                "✅ PASSED"
            } else {
                "❌ FAILED"
            },
            self.thresholds.min_hit_rate * 100.0,
            self.thresholds.min_fix_rate * 100.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_default() {
        let metrics = OracleMetrics::default();
        assert_eq!(metrics.queries, 0);
        assert_eq!(metrics.hit_rate(), 0.0);
    }

    #[test]
    fn test_hit_rate() {
        let mut metrics = OracleMetrics::default();
        metrics.record_hit("E0382");
        metrics.record_hit("E0382");
        metrics.record_miss("E0308");

        assert_eq!(metrics.queries, 3);
        assert_eq!(metrics.hits, 2);
        assert!((metrics.hit_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_by_error_code() {
        let mut metrics = OracleMetrics::default();
        metrics.record_hit("E0382");
        metrics.record_hit("E0382");
        metrics.record_miss("E0499");

        assert_eq!(metrics.by_error_code["E0382"].hits, 2);
        assert_eq!(metrics.by_error_code["E0499"].queries, 1);
        assert_eq!(metrics.by_error_code["E0499"].hits, 0);
    }

    #[test]
    fn test_fix_success_rate() {
        let mut metrics = OracleMetrics::default();
        metrics.record_fix_applied("E0382");
        metrics.record_fix_applied("E0382");
        metrics.record_fix_verified("E0382");

        assert_eq!(metrics.fixes_applied, 2);
        assert_eq!(metrics.fixes_verified, 1);
        assert!((metrics.fix_success_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_to_json() {
        let mut metrics = OracleMetrics::default();
        metrics.record_hit("E0382");
        let json = metrics.to_json();
        assert!(json.contains("\"queries\": 1"));
        assert!(json.contains("\"hits\": 1"));
    }

    #[test]
    fn test_meets_ci_thresholds_no_queries() {
        let metrics = OracleMetrics::default();
        // No queries = passes
        assert!(metrics.meets_ci_thresholds(0.5, 0.8));
    }

    #[test]
    fn test_meets_ci_thresholds_pass() {
        let mut metrics = OracleMetrics::default();
        metrics.record_hit("E0382");
        metrics.record_hit("E0382");
        metrics.record_fix_applied("E0382");
        metrics.record_fix_verified("E0382");

        // 100% hit rate, 100% fix rate - should pass
        assert!(metrics.meets_ci_thresholds(0.5, 0.8));
    }

    #[test]
    fn test_meets_ci_thresholds_fail() {
        let mut metrics = OracleMetrics::default();
        metrics.record_miss("E0382");
        metrics.record_miss("E0382");

        // 0% hit rate - should fail
        assert!(!metrics.meets_ci_thresholds(0.5, 0.8));
    }

    #[test]
    fn test_ci_report_from_metrics() {
        let mut metrics = OracleMetrics::default();
        metrics.record_hit("E0382");
        metrics.record_fix_applied("E0382");
        metrics.record_fix_verified("E0382");

        let report = CIReport::from_metrics(metrics, CIThresholds::default());
        assert!(report.passed);
        assert!((report.hit_rate_pct - 100.0).abs() < 0.01);
        assert!((report.fix_success_rate_pct - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_ci_report_to_markdown() {
        let mut metrics = OracleMetrics::default();
        metrics.record_hit("E0382");

        let report = CIReport::from_metrics(metrics, CIThresholds::default());
        let md = report.to_markdown();
        assert!(md.contains("Oracle CI Report"));
        assert!(md.contains("| Queries | 1 |"));
        assert!(md.contains("PASSED"));
    }

    #[test]
    fn test_ci_thresholds_default() {
        let thresholds = CIThresholds::default();
        assert!((thresholds.min_hit_rate - 0.5).abs() < 0.01);
        assert!((thresholds.min_fix_rate - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_record_pattern_captured() {
        let mut metrics = OracleMetrics::default();
        assert_eq!(metrics.patterns_captured, 0);
        metrics.record_pattern_captured();
        assert_eq!(metrics.patterns_captured, 1);
        metrics.record_pattern_captured();
        assert_eq!(metrics.patterns_captured, 2);
    }

    #[test]
    fn test_to_prometheus() {
        let mut metrics = OracleMetrics::default();
        metrics.record_hit("E0382");
        metrics.record_fix_applied("E0382");
        metrics.record_fix_verified("E0382");
        metrics.record_pattern_captured();

        let prom = metrics.to_prometheus();
        assert!(prom.contains("decy_oracle_queries_total 1"), "Got: {}", prom);
        assert!(prom.contains("decy_oracle_hits_total 1"), "Got: {}", prom);
        assert!(prom.contains("decy_oracle_hit_rate 1"), "Got: {}", prom);
        assert!(prom.contains("decy_oracle_fixes_applied_total 1"), "Got: {}", prom);
        assert!(prom.contains("decy_oracle_fixes_verified_total 1"), "Got: {}", prom);
        assert!(prom.contains("decy_oracle_fix_success_rate 1"), "Got: {}", prom);
    }

    #[test]
    fn test_meets_ci_thresholds_fix_rate_fail() {
        let mut metrics = OracleMetrics::default();
        metrics.record_hit("E0382");
        metrics.record_fix_applied("E0382");
        metrics.record_fix_applied("E0382");
        // 0 verified of 2 applied = 0% fix rate
        // Hit rate is 100%, fix rate is 0% — should fail on fix rate
        assert!(!metrics.meets_ci_thresholds(0.5, 0.8));
    }

    #[test]
    fn test_ci_report_to_json() {
        let mut metrics = OracleMetrics::default();
        metrics.record_hit("E0382");
        let report = CIReport::from_metrics(metrics, CIThresholds::default());
        let json = report.to_json();
        assert!(json.contains("hit_rate_pct"), "Got: {}", json);
        assert!(json.contains("passed"), "Got: {}", json);
    }

    #[test]
    fn test_ci_report_to_markdown_failed() {
        let mut metrics = OracleMetrics::default();
        metrics.record_miss("E0382");
        metrics.record_miss("E0308");
        // 0% hit rate — should fail
        let report = CIReport::from_metrics(metrics, CIThresholds::default());
        assert!(!report.passed);
        let md = report.to_markdown();
        assert!(md.contains("FAILED"), "Got: {}", md);
    }
}
