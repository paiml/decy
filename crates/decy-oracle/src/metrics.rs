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
}
