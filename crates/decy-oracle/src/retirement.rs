//! Pattern retirement policy for oracle hygiene
//!
//! Prunes obsolete patterns that are rarely used or superseded.
//! Implements Kaizen principle - continuous improvement.
//!
//! # References
//! - training-oracle-spec.md §3.3.3: Pattern Retirement Policy (Kaizen Enhancement)
//! - Gemini Review: "define a Retirement Policy for patterns"

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Retirement decision for a pattern
#[derive(Debug, Clone, PartialEq)]
pub enum RetirementDecision {
    /// Keep the pattern active
    Keep,
    /// Retire the pattern with reason
    Retire(RetirementReason),
    /// Archive the pattern (keep for analysis, don't use for suggestions)
    Archive(RetirementReason),
}

impl RetirementDecision {
    /// Check if pattern should be removed from active use
    pub fn should_remove(&self) -> bool {
        matches!(
            self,
            RetirementDecision::Retire(_) | RetirementDecision::Archive(_)
        )
    }

    /// Get the reason if retiring
    pub fn reason(&self) -> Option<&RetirementReason> {
        match self {
            RetirementDecision::Retire(r) | RetirementDecision::Archive(r) => Some(r),
            RetirementDecision::Keep => None,
        }
    }
}

/// Reason for retiring a pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RetirementReason {
    /// Pattern rarely used (< threshold uses in evaluation window)
    LowUsage {
        uses: usize,
        threshold: usize,
        window_days: u32,
    },
    /// Pattern has high failure rate
    HighFailureRate { success_rate: f32, threshold: f32 },
    /// Pattern superseded by better alternative
    Superseded {
        better_pattern_id: String,
        improvement: f32,
    },
    /// Manually deprecated by maintainer
    ManualDeprecation { reason: String },
}

impl std::fmt::Display for RetirementReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RetirementReason::LowUsage {
                uses,
                threshold,
                window_days,
            } => {
                write!(
                    f,
                    "Low usage: {} uses in {} days (threshold: {})",
                    uses, window_days, threshold
                )
            }
            RetirementReason::HighFailureRate {
                success_rate,
                threshold,
            } => {
                write!(
                    f,
                    "High failure rate: {:.1}% (threshold: {:.1}%)",
                    success_rate * 100.0,
                    threshold * 100.0
                )
            }
            RetirementReason::Superseded {
                better_pattern_id,
                improvement,
            } => {
                write!(
                    f,
                    "Superseded by {} (+{:.1}% success)",
                    better_pattern_id,
                    improvement * 100.0
                )
            }
            RetirementReason::ManualDeprecation { reason } => {
                write!(f, "Manually deprecated: {}", reason)
            }
        }
    }
}

/// Usage statistics for a pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStats {
    /// Pattern identifier
    pub pattern_id: String,
    /// Error code this pattern addresses
    pub error_code: String,
    /// Total number of times pattern was used
    pub total_uses: usize,
    /// Uses within the evaluation window
    pub uses_in_window: usize,
    /// Successful applications
    pub successes: usize,
    /// Failed applications
    pub failures: usize,
    /// Timestamp of last use
    pub last_used: Option<SystemTime>,
    /// ID of a better pattern if one exists
    pub superseded_by: Option<String>,
}

impl PatternStats {
    /// Create new pattern stats
    pub fn new(pattern_id: impl Into<String>, error_code: impl Into<String>) -> Self {
        Self {
            pattern_id: pattern_id.into(),
            error_code: error_code.into(),
            total_uses: 0,
            uses_in_window: 0,
            successes: 0,
            failures: 0,
            last_used: None,
            superseded_by: None,
        }
    }

    /// Record a use of the pattern
    pub fn record_use(&mut self, success: bool) {
        self.total_uses += 1;
        self.uses_in_window += 1;
        self.last_used = Some(SystemTime::now());

        if success {
            self.successes += 1;
        } else {
            self.failures += 1;
        }
    }

    /// Get success rate
    pub fn success_rate(&self) -> f32 {
        let total = self.successes + self.failures;
        if total == 0 {
            0.0
        } else {
            self.successes as f32 / total as f32
        }
    }

    /// Reset window usage (called at window boundary)
    pub fn reset_window(&mut self) {
        self.uses_in_window = 0;
    }

    /// Mark as superseded by another pattern
    pub fn mark_superseded(&mut self, better_pattern_id: impl Into<String>) {
        self.superseded_by = Some(better_pattern_id.into());
    }
}

/// Configuration for pattern retirement policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetirementConfig {
    /// Minimum uses to keep pattern (default: 5)
    pub min_usage_threshold: usize,
    /// Minimum success rate to keep pattern (default: 0.3)
    pub min_success_rate: f32,
    /// Evaluation window in days (default: 30)
    pub evaluation_window_days: u32,
    /// Improvement threshold to consider pattern superseded (default: 0.1)
    pub supersede_improvement_threshold: f32,
    /// Whether to archive instead of delete
    pub archive_instead_of_delete: bool,
}

impl Default for RetirementConfig {
    fn default() -> Self {
        Self {
            min_usage_threshold: 5,
            min_success_rate: 0.3,
            evaluation_window_days: 30,
            supersede_improvement_threshold: 0.1,
            archive_instead_of_delete: true,
        }
    }
}

/// Pattern retirement policy
pub struct PatternRetirementPolicy {
    config: RetirementConfig,
}

impl PatternRetirementPolicy {
    /// Create new policy with default configuration
    pub fn new() -> Self {
        Self {
            config: RetirementConfig::default(),
        }
    }

    /// Create policy with custom configuration
    pub fn with_config(config: RetirementConfig) -> Self {
        Self { config }
    }

    /// Get configuration
    pub fn config(&self) -> &RetirementConfig {
        &self.config
    }

    /// Evaluate whether a pattern should be retired
    pub fn evaluate(&self, stats: &PatternStats) -> RetirementDecision {
        // Criterion 1: Low usage
        if stats.uses_in_window < self.config.min_usage_threshold {
            let reason = RetirementReason::LowUsage {
                uses: stats.uses_in_window,
                threshold: self.config.min_usage_threshold,
                window_days: self.config.evaluation_window_days,
            };
            return if self.config.archive_instead_of_delete {
                RetirementDecision::Archive(reason)
            } else {
                RetirementDecision::Retire(reason)
            };
        }

        // Criterion 2: High failure rate
        if stats.success_rate() < self.config.min_success_rate && stats.total_uses >= 5 {
            let reason = RetirementReason::HighFailureRate {
                success_rate: stats.success_rate(),
                threshold: self.config.min_success_rate,
            };
            return if self.config.archive_instead_of_delete {
                RetirementDecision::Archive(reason)
            } else {
                RetirementDecision::Retire(reason)
            };
        }

        // Criterion 3: Superseded by better pattern
        if let Some(ref better_id) = stats.superseded_by {
            let reason = RetirementReason::Superseded {
                better_pattern_id: better_id.clone(),
                improvement: self.config.supersede_improvement_threshold,
            };
            return RetirementDecision::Archive(reason);
        }

        RetirementDecision::Keep
    }

    /// Evaluate multiple patterns and return retirement decisions
    pub fn evaluate_batch(&self, stats_list: &[PatternStats]) -> Vec<(String, RetirementDecision)> {
        stats_list
            .iter()
            .map(|stats| (stats.pattern_id.clone(), self.evaluate(stats)))
            .collect()
    }

    /// Find patterns that should be retired
    pub fn find_retireable<'a>(&self, stats_list: &'a [PatternStats]) -> Vec<&'a PatternStats> {
        stats_list
            .iter()
            .filter(|stats| self.evaluate(stats).should_remove())
            .collect()
    }
}

impl Default for PatternRetirementPolicy {
    fn default() -> Self {
        Self::new()
    }
}

/// Results of a retirement sweep
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetirementSweepResult {
    /// Total patterns evaluated
    pub total_evaluated: usize,
    /// Patterns kept
    pub kept: usize,
    /// Patterns retired due to low usage
    pub retired_low_usage: usize,
    /// Patterns retired due to high failure rate
    pub retired_high_failure: usize,
    /// Patterns retired due to being superseded
    pub retired_superseded: usize,
    /// Patterns archived (instead of deleted)
    pub archived: usize,
    /// Pattern IDs that were retired
    pub retired_ids: Vec<String>,
}

impl RetirementSweepResult {
    /// Create new empty result
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a decision
    pub fn record(&mut self, pattern_id: &str, decision: &RetirementDecision) {
        self.total_evaluated += 1;
        match decision {
            RetirementDecision::Keep => {
                self.kept += 1;
            }
            RetirementDecision::Retire(reason) | RetirementDecision::Archive(reason) => {
                self.retired_ids.push(pattern_id.to_string());
                if matches!(decision, RetirementDecision::Archive(_)) {
                    self.archived += 1;
                }
                match reason {
                    RetirementReason::LowUsage { .. } => self.retired_low_usage += 1,
                    RetirementReason::HighFailureRate { .. } => self.retired_high_failure += 1,
                    RetirementReason::Superseded { .. } => self.retired_superseded += 1,
                    RetirementReason::ManualDeprecation { .. } => {}
                }
            }
        }
    }

    /// Get total retired count
    pub fn total_retired(&self) -> usize {
        self.retired_low_usage + self.retired_high_failure + self.retired_superseded
    }

    /// Get retirement rate
    pub fn retirement_rate(&self) -> f32 {
        if self.total_evaluated == 0 {
            0.0
        } else {
            self.total_retired() as f32 / self.total_evaluated as f32
        }
    }
}

/// Run a retirement sweep on pattern statistics
pub fn run_retirement_sweep(
    stats_list: &[PatternStats],
    policy: &PatternRetirementPolicy,
) -> RetirementSweepResult {
    let mut result = RetirementSweepResult::new();

    for stats in stats_list {
        let decision = policy.evaluate(stats);
        result.record(&stats.pattern_id, &decision);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // RetirementReason Tests
    // ============================================================================

    #[test]
    fn test_retirement_reason_display_low_usage() {
        let reason = RetirementReason::LowUsage {
            uses: 2,
            threshold: 5,
            window_days: 30,
        };
        let display = format!("{}", reason);
        assert!(display.contains("Low usage"));
        assert!(display.contains("2 uses"));
    }

    #[test]
    fn test_retirement_reason_display_high_failure() {
        let reason = RetirementReason::HighFailureRate {
            success_rate: 0.2,
            threshold: 0.3,
        };
        let display = format!("{}", reason);
        assert!(display.contains("High failure rate"));
        assert!(display.contains("20.0%"));
    }

    #[test]
    fn test_retirement_reason_display_superseded() {
        let reason = RetirementReason::Superseded {
            better_pattern_id: "pattern-123".into(),
            improvement: 0.15,
        };
        let display = format!("{}", reason);
        assert!(display.contains("Superseded"));
        assert!(display.contains("pattern-123"));
    }

    // ============================================================================
    // RetirementDecision Tests
    // ============================================================================

    #[test]
    fn test_retirement_decision_should_remove() {
        assert!(!RetirementDecision::Keep.should_remove());
        assert!(RetirementDecision::Retire(RetirementReason::LowUsage {
            uses: 0,
            threshold: 5,
            window_days: 30
        })
        .should_remove());
        assert!(RetirementDecision::Archive(RetirementReason::LowUsage {
            uses: 0,
            threshold: 5,
            window_days: 30
        })
        .should_remove());
    }

    #[test]
    fn test_retirement_decision_reason() {
        assert!(RetirementDecision::Keep.reason().is_none());

        let reason = RetirementReason::LowUsage {
            uses: 0,
            threshold: 5,
            window_days: 30,
        };
        let decision = RetirementDecision::Retire(reason.clone());
        assert!(decision.reason().is_some());
    }

    // ============================================================================
    // PatternStats Tests
    // ============================================================================

    #[test]
    fn test_pattern_stats_new() {
        let stats = PatternStats::new("pat-1", "E0382");
        assert_eq!(stats.pattern_id, "pat-1");
        assert_eq!(stats.error_code, "E0382");
        assert_eq!(stats.total_uses, 0);
        assert_eq!(stats.success_rate(), 0.0);
    }

    #[test]
    fn test_pattern_stats_record_use() {
        let mut stats = PatternStats::new("pat-1", "E0382");

        stats.record_use(true);
        stats.record_use(true);
        stats.record_use(false);

        assert_eq!(stats.total_uses, 3);
        assert_eq!(stats.successes, 2);
        assert_eq!(stats.failures, 1);
        assert!((stats.success_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_pattern_stats_reset_window() {
        let mut stats = PatternStats::new("pat-1", "E0382");
        stats.record_use(true);
        stats.record_use(true);

        assert_eq!(stats.uses_in_window, 2);
        stats.reset_window();
        assert_eq!(stats.uses_in_window, 0);
        assert_eq!(stats.total_uses, 2); // Total unchanged
    }

    #[test]
    fn test_pattern_stats_mark_superseded() {
        let mut stats = PatternStats::new("pat-1", "E0382");
        assert!(stats.superseded_by.is_none());

        stats.mark_superseded("pat-2");
        assert_eq!(stats.superseded_by, Some("pat-2".into()));
    }

    // ============================================================================
    // RetirementConfig Tests
    // ============================================================================

    #[test]
    fn test_retirement_config_default() {
        let config = RetirementConfig::default();
        assert_eq!(config.min_usage_threshold, 5);
        assert!((config.min_success_rate - 0.3).abs() < f32::EPSILON);
        assert_eq!(config.evaluation_window_days, 30);
        assert!(config.archive_instead_of_delete);
    }

    // ============================================================================
    // PatternRetirementPolicy Tests
    // ============================================================================

    #[test]
    fn test_policy_keeps_active_pattern() {
        let policy = PatternRetirementPolicy::new();
        let mut stats = PatternStats::new("pat-1", "E0382");

        // Add enough uses with good success rate
        for _ in 0..10 {
            stats.record_use(true);
        }

        let decision = policy.evaluate(&stats);
        assert_eq!(decision, RetirementDecision::Keep);
    }

    #[test]
    fn test_policy_retires_low_usage() {
        let policy = PatternRetirementPolicy::new();
        let mut stats = PatternStats::new("pat-1", "E0382");

        // Only 2 uses (below threshold of 5)
        stats.record_use(true);
        stats.record_use(true);

        let decision = policy.evaluate(&stats);
        assert!(decision.should_remove());
        assert!(matches!(
            decision.reason(),
            Some(RetirementReason::LowUsage { .. })
        ));
    }

    #[test]
    fn test_policy_retires_high_failure() {
        let policy = PatternRetirementPolicy::new();
        let mut stats = PatternStats::new("pat-1", "E0382");

        // 10 uses, 20% success rate (below threshold of 30%)
        stats.record_use(true);
        stats.record_use(true);
        for _ in 0..8 {
            stats.record_use(false);
        }
        // Force uses_in_window to be above threshold
        stats.uses_in_window = 10;

        let decision = policy.evaluate(&stats);
        assert!(decision.should_remove());
        assert!(matches!(
            decision.reason(),
            Some(RetirementReason::HighFailureRate { .. })
        ));
    }

    #[test]
    fn test_policy_retires_superseded() {
        let policy = PatternRetirementPolicy::new();
        let mut stats = PatternStats::new("pat-1", "E0382");

        // Good stats but superseded
        for _ in 0..10 {
            stats.record_use(true);
        }
        stats.mark_superseded("pat-2");

        let decision = policy.evaluate(&stats);
        assert!(decision.should_remove());
        assert!(matches!(
            decision.reason(),
            Some(RetirementReason::Superseded { .. })
        ));
    }

    #[test]
    fn test_policy_evaluate_batch() {
        let policy = PatternRetirementPolicy::new();

        let mut stats1 = PatternStats::new("pat-1", "E0382");
        for _ in 0..10 {
            stats1.record_use(true);
        }

        let mut stats2 = PatternStats::new("pat-2", "E0382");
        stats2.record_use(true);

        let batch = vec![stats1, stats2];
        let decisions = policy.evaluate_batch(&batch);

        assert_eq!(decisions.len(), 2);
        assert_eq!(decisions[0].1, RetirementDecision::Keep);
        assert!(decisions[1].1.should_remove());
    }

    #[test]
    fn test_policy_find_retireable() {
        let policy = PatternRetirementPolicy::new();

        let mut stats1 = PatternStats::new("pat-1", "E0382");
        for _ in 0..10 {
            stats1.record_use(true);
        }

        let mut stats2 = PatternStats::new("pat-2", "E0382");
        stats2.record_use(true);

        let batch = vec![stats1, stats2];
        let retireable = policy.find_retireable(&batch);

        assert_eq!(retireable.len(), 1);
        assert_eq!(retireable[0].pattern_id, "pat-2");
    }

    // ============================================================================
    // RetirementSweepResult Tests
    // ============================================================================

    #[test]
    fn test_sweep_result_new() {
        let result = RetirementSweepResult::new();
        assert_eq!(result.total_evaluated, 0);
        assert_eq!(result.kept, 0);
    }

    #[test]
    fn test_sweep_result_record() {
        let mut result = RetirementSweepResult::new();

        result.record("pat-1", &RetirementDecision::Keep);
        result.record(
            "pat-2",
            &RetirementDecision::Retire(RetirementReason::LowUsage {
                uses: 0,
                threshold: 5,
                window_days: 30,
            }),
        );
        result.record(
            "pat-3",
            &RetirementDecision::Archive(RetirementReason::HighFailureRate {
                success_rate: 0.1,
                threshold: 0.3,
            }),
        );

        assert_eq!(result.total_evaluated, 3);
        assert_eq!(result.kept, 1);
        assert_eq!(result.retired_low_usage, 1);
        assert_eq!(result.retired_high_failure, 1);
        assert_eq!(result.archived, 1);
        assert_eq!(result.retired_ids.len(), 2);
    }

    #[test]
    fn test_sweep_result_total_retired() {
        let mut result = RetirementSweepResult::new();
        result.retired_low_usage = 3;
        result.retired_high_failure = 2;
        result.retired_superseded = 1;

        assert_eq!(result.total_retired(), 6);
    }

    #[test]
    fn test_sweep_result_retirement_rate() {
        let mut result = RetirementSweepResult::new();
        result.total_evaluated = 10;
        result.retired_low_usage = 2;
        result.retired_high_failure = 1;

        assert!((result.retirement_rate() - 0.3).abs() < 0.01);
    }

    // ============================================================================
    // run_retirement_sweep Tests
    // ============================================================================

    #[test]
    fn test_run_retirement_sweep() {
        let policy = PatternRetirementPolicy::new();

        let mut stats1 = PatternStats::new("pat-1", "E0382");
        for _ in 0..10 {
            stats1.record_use(true);
        }

        let mut stats2 = PatternStats::new("pat-2", "E0382");
        stats2.record_use(true);

        let mut stats3 = PatternStats::new("pat-3", "E0382");
        for _ in 0..10 {
            stats3.record_use(false);
        }
        stats3.uses_in_window = 10;

        let batch = vec![stats1, stats2, stats3];
        let result = run_retirement_sweep(&batch, &policy);

        assert_eq!(result.total_evaluated, 3);
        assert_eq!(result.kept, 1);
        assert_eq!(result.total_retired(), 2);
    }

    // ============================================================================
    // Spec Compliance Tests
    // ============================================================================

    #[test]
    fn test_spec_low_usage_threshold() {
        // Spec: Retire patterns with < 5 uses in 30 days
        let config = RetirementConfig::default();
        assert_eq!(config.min_usage_threshold, 5);
        assert_eq!(config.evaluation_window_days, 30);
    }

    #[test]
    fn test_spec_high_failure_threshold() {
        // Spec: Retire patterns with success_rate < 0.3
        let config = RetirementConfig::default();
        assert!((config.min_success_rate - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_spec_superseded_archived() {
        // Spec: Superseded patterns should be archived (not deleted)
        let policy = PatternRetirementPolicy::new();
        let mut stats = PatternStats::new("pat-1", "E0382");

        for _ in 0..10 {
            stats.record_use(true);
        }
        stats.mark_superseded("pat-2");

        let decision = policy.evaluate(&stats);
        assert!(matches!(decision, RetirementDecision::Archive(_)));
    }

    // ============================================================================
    // COVERAGE: ManualDeprecation Display, with_config, config(), Retire branches
    // ============================================================================

    #[test]
    fn test_manual_deprecation_display() {
        let reason = RetirementReason::ManualDeprecation {
            reason: "API changed".to_string(),
        };
        let display = format!("{}", reason);
        assert!(
            display.contains("Manually deprecated"),
            "Got: {}",
            display
        );
        assert!(display.contains("API changed"), "Got: {}", display);
    }

    #[test]
    fn test_policy_with_config() {
        let config = RetirementConfig {
            min_usage_threshold: 10,
            min_success_rate: 0.5,
            evaluation_window_days: 60,
            supersede_improvement_threshold: 0.2,
            archive_instead_of_delete: false,
        };
        let policy = PatternRetirementPolicy::with_config(config);
        assert_eq!(policy.config().min_usage_threshold, 10);
        assert_eq!(policy.config().evaluation_window_days, 60);
        assert!(!policy.config().archive_instead_of_delete);
    }

    #[test]
    fn test_policy_config_accessor() {
        let policy = PatternRetirementPolicy::new();
        let config = policy.config();
        assert_eq!(config.min_usage_threshold, 5);
        assert!((config.min_success_rate - 0.3).abs() < 0.01);
        assert!(config.archive_instead_of_delete);
    }

    #[test]
    fn test_policy_default_impl() {
        let policy = PatternRetirementPolicy::default();
        assert_eq!(policy.config().min_usage_threshold, 5);
    }

    #[test]
    fn test_evaluate_low_usage_retire_not_archive() {
        let config = RetirementConfig {
            archive_instead_of_delete: false,
            ..Default::default()
        };
        let policy = PatternRetirementPolicy::with_config(config);
        let stats = PatternStats::new("pat-1", "E0382");
        // 0 uses in window < threshold 5 → Retire (not Archive)
        let decision = policy.evaluate(&stats);
        assert!(
            matches!(decision, RetirementDecision::Retire(RetirementReason::LowUsage { .. })),
            "Expected Retire(LowUsage), got: {:?}",
            decision
        );
    }

    #[test]
    fn test_evaluate_high_failure_retire_not_archive() {
        let config = RetirementConfig {
            archive_instead_of_delete: false,
            ..Default::default()
        };
        let policy = PatternRetirementPolicy::with_config(config);
        let mut stats = PatternStats::new("pat-1", "E0382");
        // Need >= 5 total uses and success_rate < 0.3
        for _ in 0..5 {
            stats.record_use(true);
        }
        stats.uses_in_window = 10; // above threshold
        stats.successes = 1;
        stats.failures = 9;
        stats.total_uses = 10;
        let decision = policy.evaluate(&stats);
        assert!(
            matches!(decision, RetirementDecision::Retire(RetirementReason::HighFailureRate { .. })),
            "Expected Retire(HighFailureRate), got: {:?}",
            decision
        );
    }

    #[test]
    fn test_sweep_result_manual_deprecation() {
        let mut result = RetirementSweepResult::default();
        let decision = RetirementDecision::Retire(RetirementReason::ManualDeprecation {
            reason: "Obsolete".to_string(),
        });
        result.record("pat-1", &decision);
        assert_eq!(result.total_evaluated, 1);
        assert_eq!(result.retired_ids.len(), 1);
        assert_eq!(result.retired_ids[0], "pat-1");
        // ManualDeprecation doesn't increment specific counters
        assert_eq!(result.retired_low_usage, 0);
        assert_eq!(result.retired_high_failure, 0);
        assert_eq!(result.retired_superseded, 0);
    }

    #[test]
    fn test_sweep_result_retirement_rate_zero() {
        let result = RetirementSweepResult::default();
        assert_eq!(result.retirement_rate(), 0.0);
    }

    #[test]
    fn test_sweep_result_record_superseded() {
        let mut result = RetirementSweepResult::default();
        let decision = RetirementDecision::Archive(RetirementReason::Superseded {
            better_pattern_id: "pat-new".to_string(),
            improvement: 0.15,
        });
        result.record("pat-old", &decision);
        assert_eq!(result.total_evaluated, 1);
        assert_eq!(result.retired_superseded, 1);
        assert_eq!(result.archived, 1);
        assert_eq!(result.retired_ids.len(), 1);
    }

    #[test]
    fn test_sweep_result_record_archive_low_usage() {
        let mut result = RetirementSweepResult::default();
        let decision = RetirementDecision::Archive(RetirementReason::LowUsage {
            uses: 1,
            threshold: 5,
            window_days: 30,
        });
        result.record("pat-stale", &decision);
        assert_eq!(result.retired_low_usage, 1);
        assert_eq!(result.archived, 1);
    }
}
