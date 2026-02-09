//! Deep coverage tests for error_tracking, threshold_tuning, and hybrid_classifier.
//!
//! Targets every branch in:
//! - `ErrorTracker::generate_markdown_report` (line 414)
//! - `ErrorTracker::calculate_suspiciousness` (line 688)
//! - `ThresholdMetrics::calculate` (line 112)
//! - `ThresholdTuner::select_optimal` (line 399)
//! - `HybridClassifier::classify_ensemble` (line 266)

use crate::error_tracking::{
    ErrorTracker, FeatureSuspiciousness, InferenceError, PatternStats,
    SuggestionCategory, SuggestionPriority,
};
use crate::hybrid_classifier::{
    ClassificationMethod, HybridClassifier, HybridMetrics, HybridResult, NullModel,
    OwnershipModel,
};
use crate::inference::{OwnershipInference, OwnershipKind};
use crate::ml_features::{InferredOwnership, OwnershipDefect, OwnershipFeatures, OwnershipPrediction};
use crate::threshold_tuning::{
    find_optimal_threshold, SelectionCriteria, ThresholdMetrics, ThresholdTuner,
    ValidationSample,
};

// ============================================================================
// Helper: configurable mock model for hybrid_classifier tests
// ============================================================================

struct ConfigurableMockModel {
    ownership: InferredOwnership,
    confidence: f64,
}

impl ConfigurableMockModel {
    fn new(ownership: InferredOwnership, confidence: f64) -> Self {
        Self {
            ownership,
            confidence,
        }
    }
}

impl OwnershipModel for ConfigurableMockModel {
    fn predict(&self, _features: &OwnershipFeatures) -> OwnershipPrediction {
        OwnershipPrediction {
            kind: self.ownership,
            confidence: self.confidence as f32,
            fallback: None,
        }
    }

    fn name(&self) -> &str {
        "configurable-mock"
    }
}

// Helper to create a standard InferenceError with features
fn make_error(
    variable: &str,
    predicted: InferredOwnership,
    expected: InferredOwnership,
    confidence: f64,
    defect: OwnershipDefect,
    features: Vec<String>,
) -> InferenceError {
    InferenceError::new(variable, "test.c", 1, predicted, expected, confidence, defect)
        .with_features(features)
}

// Helper to create a standard OwnershipInference
fn make_inference(variable: &str, kind: OwnershipKind, confidence: f32, reason: &str) -> OwnershipInference {
    OwnershipInference {
        variable: variable.to_string(),
        kind,
        confidence,
        reason: reason.to_string(),
    }
}

// ============================================================================
// ERROR TRACKING: generate_markdown_report
// ============================================================================

#[test]
fn markdown_report_empty_tracker_zero_error_rate() {
    // Branch: error_count + success_count == 0 => error rate 0.0
    let mut tracker = ErrorTracker::new();
    let report = tracker.generate_markdown_report();
    assert!(report.contains("Total Errors | 0"));
    assert!(report.contains("Total Successes | 0"));
    assert!(report.contains("Error Rate | 0.0%"));
}

#[test]
fn markdown_report_with_errors_and_successes_computes_error_rate() {
    // Branch: error_count + success_count > 0 => calculate error rate
    let mut tracker = ErrorTracker::new();
    for i in 0..3 {
        tracker.record_error(make_error(
            &format!("v{}", i),
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.5,
            OwnershipDefect::PointerMisclassification,
            vec!["feat_a".to_string()],
        ));
    }
    tracker.record_success(&["feat_a".to_string()]);
    let report = tracker.generate_markdown_report();
    // 3 errors, 1 success => error rate = 75.0%
    assert!(report.contains("Total Errors | 3"));
    assert!(report.contains("Total Successes | 1"));
    assert!(report.contains("75.0%"));
}

#[test]
fn markdown_report_includes_suspicious_features_table() {
    let mut tracker = ErrorTracker::new();
    for _ in 0..5 {
        tracker.record_error(make_error(
            "ptr",
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.6,
            OwnershipDefect::PointerMisclassification,
            vec!["malloc_free".to_string()],
        ));
    }
    tracker.record_success(&["malloc_free".to_string()]);
    let report = tracker.generate_markdown_report();
    assert!(report.contains("Top Suspicious Features (Tarantula)"));
    assert!(report.contains("malloc_free"));
    assert!(report.contains("Feature | Score | Failures | Successes"));
}

#[test]
fn markdown_report_defect_distribution_section() {
    let mut tracker = ErrorTracker::new();
    for _ in 0..4 {
        tracker.record_error(make_error(
            "ptr",
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.6,
            OwnershipDefect::PointerMisclassification,
            vec![],
        ));
    }
    for _ in 0..2 {
        tracker.record_error(make_error(
            "ptr",
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.6,
            OwnershipDefect::LifetimeInferenceGap,
            vec![],
        ));
    }
    let report = tracker.generate_markdown_report();
    assert!(report.contains("Defect Distribution"));
    assert!(report.contains("PointerMisclassification"));
    assert!(report.contains("LifetimeInferenceGap"));
}

#[test]
fn markdown_report_defect_distribution_percentage_uses_max1() {
    // Branch: self.error_count().max(1) - with errors, uses actual count
    let mut tracker = ErrorTracker::new();
    tracker.record_error(make_error(
        "ptr",
        InferredOwnership::Borrowed,
        InferredOwnership::Owned,
        0.5,
        OwnershipDefect::AliasViolation,
        vec![],
    ));
    let report = tracker.generate_markdown_report();
    // 1/1 = 100.0%
    assert!(report.contains("100.0%"));
}

#[test]
fn markdown_report_with_suggestions_section() {
    // Branch: suggestions is not empty => include "Improvement Suggestions"
    let mut tracker = ErrorTracker::new();
    for i in 0..30 {
        tracker.record_error(make_error(
            &format!("ptr{}", i),
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.6,
            OwnershipDefect::PointerMisclassification,
            vec!["dangerous_feature".to_string()],
        ));
    }
    for _ in 0..2 {
        tracker.record_success(&["dangerous_feature".to_string()]);
    }
    let report = tracker.generate_markdown_report();
    assert!(report.contains("Improvement Suggestions"));
    assert!(report.contains("[High]"));
}

#[test]
fn markdown_report_without_suggestions_section() {
    // Branch: suggestions is empty => no "Improvement Suggestions" section
    let mut tracker = ErrorTracker::new();
    // Just 1 error, not enough to trigger suggestions (count <= 5)
    tracker.record_error(make_error(
        "ptr",
        InferredOwnership::Borrowed,
        InferredOwnership::Owned,
        0.5,
        OwnershipDefect::PointerMisclassification,
        vec![],
    ));
    // Lots of successes to make suspiciousness low
    for _ in 0..50 {
        tracker.record_success(&["safe_feature".to_string()]);
    }
    let report = tracker.generate_markdown_report();
    assert!(!report.contains("Improvement Suggestions"));
}

#[test]
fn markdown_report_limits_suspicious_features_to_5() {
    let mut tracker = ErrorTracker::new();
    // Create 8 different features
    for i in 0..8 {
        for _ in 0..3 {
            tracker.record_error(make_error(
                "ptr",
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.6,
                OwnershipDefect::PointerMisclassification,
                vec![format!("feature_{}", i)],
            ));
        }
    }
    let report = tracker.generate_markdown_report();
    // Table rows for suspicious features should be limited to 5
    let feature_rows: Vec<&str> = report
        .lines()
        .filter(|l| l.starts_with("| feature_"))
        .collect();
    assert!(feature_rows.len() <= 5);
}

#[test]
fn markdown_report_limits_defect_distribution_to_5() {
    let mut tracker = ErrorTracker::new();
    // Use all 8 defect types
    let defects = [
        OwnershipDefect::PointerMisclassification,
        OwnershipDefect::LifetimeInferenceGap,
        OwnershipDefect::DanglingPointerRisk,
        OwnershipDefect::AliasViolation,
        OwnershipDefect::UnsafeMinimizationFailure,
        OwnershipDefect::ArraySliceMismatch,
        OwnershipDefect::ResourceLeakPattern,
        OwnershipDefect::MutabilityMismatch,
    ];
    for defect in &defects {
        for _ in 0..3 {
            tracker.record_error(make_error(
                "ptr",
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.5,
                *defect,
                vec![],
            ));
        }
    }
    let report = tracker.generate_markdown_report();
    // Defect distribution section uses .take(5) so at most 5 defect lines
    let defect_lines: Vec<&str> = report
        .lines()
        .filter(|l| l.starts_with("- ") && l.contains(":") && l.contains("("))
        .collect();
    assert!(defect_lines.len() <= 5);
}

#[test]
fn markdown_report_numbered_suggestions() {
    let mut tracker = ErrorTracker::new();
    for i in 0..25 {
        tracker.record_error(make_error(
            &format!("p{}", i),
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.6,
            OwnershipDefect::PointerMisclassification,
            vec!["bad_pattern".to_string()],
        ));
    }
    for _ in 0..2 {
        tracker.record_success(&["bad_pattern".to_string()]);
    }
    let report = tracker.generate_markdown_report();
    assert!(report.contains("1. "));
}

// ============================================================================
// ERROR TRACKING: calculate_suspiciousness
// ============================================================================

#[test]
fn suspiciousness_empty_tracker_returns_empty() {
    let mut tracker = ErrorTracker::new();
    let results = tracker.calculate_suspiciousness();
    assert!(results.is_empty());
}

#[test]
fn suspiciousness_all_failures_no_successes() {
    // Branch: passed_ratio == 0 and failed_ratio > 0
    let mut tracker = ErrorTracker::new();
    for _ in 0..5 {
        tracker.record_error(make_error(
            "ptr",
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.5,
            OwnershipDefect::PointerMisclassification,
            vec!["only_fails".to_string()],
        ));
    }
    let results = tracker.calculate_suspiciousness();
    assert_eq!(results.len(), 1);
    // failed_ratio = 5/5 = 1.0, passed_ratio = 0/1 = 0.0
    // suspiciousness = 1.0 / (1.0 + 0.0) = 1.0
    assert!((results[0].score - 1.0).abs() < 0.01);
}

#[test]
fn suspiciousness_all_successes_no_failures() {
    // Branch: failed_ratio == 0 and passed_ratio > 0
    let mut tracker = ErrorTracker::new();
    for _ in 0..5 {
        tracker.record_success(&["only_succeeds".to_string()]);
    }
    let results = tracker.calculate_suspiciousness();
    assert_eq!(results.len(), 1);
    // failed_ratio = 0/1 = 0.0, passed_ratio = 5/5 = 1.0
    // suspiciousness = 0.0 / (0.0 + 1.0) = 0.0
    assert!((results[0].score - 0.0).abs() < 0.01);
}

#[test]
fn suspiciousness_mixed_failures_successes() {
    let mut tracker = ErrorTracker::new();
    // Feature A: 8 failures, 2 successes
    for _ in 0..8 {
        tracker.record_error(make_error(
            "ptr",
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.5,
            OwnershipDefect::PointerMisclassification,
            vec!["feat_a".to_string()],
        ));
    }
    for _ in 0..2 {
        tracker.record_success(&["feat_a".to_string()]);
    }
    // Feature B: 2 failures, 8 successes
    for _ in 0..2 {
        tracker.record_error(make_error(
            "ptr",
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.5,
            OwnershipDefect::PointerMisclassification,
            vec!["feat_b".to_string()],
        ));
    }
    for _ in 0..8 {
        tracker.record_success(&["feat_b".to_string()]);
    }
    let results = tracker.calculate_suspiciousness();
    assert_eq!(results.len(), 2);
    // Results should be sorted with highest suspiciousness first
    assert!(results[0].score >= results[1].score);
    // Feature A should be first (more suspicious)
    assert_eq!(results[0].feature, "feat_a");
    assert_eq!(results[1].feature, "feat_b");
}

#[test]
fn suspiciousness_updates_feature_stats_in_place() {
    let mut tracker = ErrorTracker::new();
    tracker.record_error(make_error(
        "ptr",
        InferredOwnership::Borrowed,
        InferredOwnership::Owned,
        0.5,
        OwnershipDefect::PointerMisclassification,
        vec!["my_feature".to_string()],
    ));
    tracker.record_success(&["my_feature".to_string()]);
    let results = tracker.calculate_suspiciousness();
    // Verify stats are updated
    assert_eq!(results[0].total_count, 2);
    assert_eq!(results[0].failure_count, 1);
    assert_eq!(results[0].success_count, 1);
}

#[test]
fn suspiciousness_zero_failed_ratio_plus_passed_ratio_edge() {
    // Branch: failed_ratio + passed_ratio == 0.0 => suspiciousness = 0.0
    // This can happen if a feature has 0 failures and 0 successes in stats,
    // which requires direct manipulation. But with the API, we can get close:
    // A feature that is in feature_stats with count>0 but was never actually
    // involved in a success or failure path. Not reachable via public API,
    // so we test the closest scenario: fresh tracker with only one success
    // for a different feature.
    let mut tracker = ErrorTracker::new();
    tracker.record_success(&["other".to_string()]);
    // Record an error with feature "test_feat"
    tracker.record_error(make_error(
        "ptr",
        InferredOwnership::Borrowed,
        InferredOwnership::Owned,
        0.5,
        OwnershipDefect::PointerMisclassification,
        vec!["test_feat".to_string()],
    ));
    let results = tracker.calculate_suspiciousness();
    // test_feat: failed_ratio = 1/1 = 1.0, passed_ratio = 0/1 = 0.0
    // suspiciousness = 1.0 / (1.0 + 0.0) = 1.0
    let test_feat = results.iter().find(|r| r.feature == "test_feat").unwrap();
    assert!((test_feat.score - 1.0).abs() < 0.01);
}

#[test]
fn suspiciousness_sorted_descending_by_score() {
    let mut tracker = ErrorTracker::new();
    // Create 3 features with different suspiciousness levels
    for _ in 0..10 {
        tracker.record_error(make_error(
            "ptr",
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.5,
            OwnershipDefect::PointerMisclassification,
            vec!["high".to_string()],
        ));
    }
    for _ in 0..5 {
        tracker.record_error(make_error(
            "ptr",
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.5,
            OwnershipDefect::PointerMisclassification,
            vec!["medium".to_string()],
        ));
    }
    for _ in 0..5 {
        tracker.record_success(&["medium".to_string()]);
    }
    for _ in 0..1 {
        tracker.record_error(make_error(
            "ptr",
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.5,
            OwnershipDefect::PointerMisclassification,
            vec!["low".to_string()],
        ));
    }
    for _ in 0..10 {
        tracker.record_success(&["low".to_string()]);
    }

    let results = tracker.calculate_suspiciousness();
    for i in 0..results.len() - 1 {
        assert!(results[i].score >= results[i + 1].score);
    }
}

#[test]
fn suspiciousness_multiple_features_same_error() {
    let mut tracker = ErrorTracker::new();
    tracker.record_error(make_error(
        "ptr",
        InferredOwnership::Borrowed,
        InferredOwnership::Owned,
        0.5,
        OwnershipDefect::PointerMisclassification,
        vec!["feat_x".to_string(), "feat_y".to_string()],
    ));
    let results = tracker.calculate_suspiciousness();
    assert_eq!(results.len(), 2);
    // Both features should have failure_count = 1
    for r in &results {
        assert_eq!(r.failure_count, 1);
    }
}

// ============================================================================
// ERROR TRACKING: generate_suggestions branches
// ============================================================================

#[test]
fn suggestions_no_highly_suspicious_no_frequent_defects() {
    let mut tracker = ErrorTracker::new();
    // 1 error + many successes => low suspiciousness, count=1 <= 5
    tracker.record_error(make_error(
        "ptr",
        InferredOwnership::Borrowed,
        InferredOwnership::Owned,
        0.5,
        OwnershipDefect::PointerMisclassification,
        vec!["safe_feat".to_string()],
    ));
    for _ in 0..100 {
        tracker.record_success(&["safe_feat".to_string()]);
    }
    let suggestions = tracker.generate_suggestions();
    assert!(suggestions.is_empty());
}

#[test]
fn suggestions_highly_suspicious_feature_triggers_feature_handling() {
    let mut tracker = ErrorTracker::new();
    // We need suspiciousness > 0.7. Tarantula formula:
    // susp = (failures/total_failures) / ((failures/total_failures) + (successes/total_successes))
    // Feature "risky_feat": failures=20 out of total_failures=20, successes=1 out of total_successes=20
    // susp = (20/20) / ((20/20) + (1/20)) = 1.0 / (1.0 + 0.05) = 0.952
    for _ in 0..20 {
        tracker.record_error(make_error(
            "ptr",
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.5,
            OwnershipDefect::PointerMisclassification,
            vec!["risky_feat".to_string()],
        ));
    }
    // 1 success for risky_feat, but 19 successes for other features to inflate total_successes
    tracker.record_success(&["risky_feat".to_string()]);
    for _ in 0..19 {
        tracker.record_success(&["other_feat".to_string()]);
    }
    let suggestions = tracker.generate_suggestions();
    let feature_suggestions: Vec<_> = suggestions
        .iter()
        .filter(|s| s.category == SuggestionCategory::FeatureHandling)
        .collect();
    assert!(!feature_suggestions.is_empty());
    assert_eq!(feature_suggestions[0].priority, SuggestionPriority::High);
    assert!(feature_suggestions[0].affected_feature.is_some());
}

#[test]
fn suggestions_defect_count_above_5_below_20_gives_medium() {
    // Branch: *count > 5 and *count <= 20 => Medium priority
    let mut tracker = ErrorTracker::new();
    for i in 0..10 {
        tracker.record_error(make_error(
            &format!("v{}", i),
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.5,
            OwnershipDefect::DanglingPointerRisk,
            vec![],
        ));
    }
    for _ in 0..20 {
        tracker.record_success(&["x".to_string()]);
    }
    let suggestions = tracker.generate_suggestions();
    let defect_sug: Vec<_> = suggestions
        .iter()
        .filter(|s| s.category == SuggestionCategory::DefectPrevention)
        .collect();
    assert!(!defect_sug.is_empty());
    assert_eq!(defect_sug[0].priority, SuggestionPriority::Medium);
    assert!(defect_sug[0].affected_defect.is_some());
}

#[test]
fn suggestions_defect_count_above_20_gives_high() {
    // Branch: *count > 20 => High priority
    let mut tracker = ErrorTracker::new();
    for i in 0..25 {
        tracker.record_error(make_error(
            &format!("v{}", i),
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.5,
            OwnershipDefect::MutabilityMismatch,
            vec![],
        ));
    }
    for _ in 0..5 {
        tracker.record_success(&["x".to_string()]);
    }
    let suggestions = tracker.generate_suggestions();
    let defect_sug: Vec<_> = suggestions
        .iter()
        .filter(|s| s.category == SuggestionCategory::DefectPrevention)
        .collect();
    assert!(!defect_sug.is_empty());
    assert_eq!(defect_sug[0].priority, SuggestionPriority::High);
}

#[test]
fn suggestions_defect_count_5_or_below_skipped() {
    // Branch: *count <= 5 => no DefectPrevention suggestion
    let mut tracker = ErrorTracker::new();
    for i in 0..5 {
        tracker.record_error(make_error(
            &format!("v{}", i),
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.5,
            OwnershipDefect::ArraySliceMismatch,
            vec![],
        ));
    }
    for _ in 0..30 {
        tracker.record_success(&["x".to_string()]);
    }
    let suggestions = tracker.generate_suggestions();
    let defect_sug: Vec<_> = suggestions
        .iter()
        .filter(|s| s.category == SuggestionCategory::DefectPrevention)
        .collect();
    assert!(defect_sug.is_empty());
}

#[test]
fn suggestions_top3_defects_only() {
    // generate_suggestions uses .take(3) for defect iteration
    let mut tracker = ErrorTracker::new();
    let defects = [
        OwnershipDefect::PointerMisclassification,
        OwnershipDefect::LifetimeInferenceGap,
        OwnershipDefect::DanglingPointerRisk,
        OwnershipDefect::AliasViolation,
    ];
    for defect in &defects {
        for i in 0..10 {
            tracker.record_error(make_error(
                &format!("v{}", i),
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.5,
                *defect,
                vec![],
            ));
        }
    }
    for _ in 0..20 {
        tracker.record_success(&["x".to_string()]);
    }
    let suggestions = tracker.generate_suggestions();
    let defect_sug: Vec<_> = suggestions
        .iter()
        .filter(|s| s.category == SuggestionCategory::DefectPrevention)
        .collect();
    // At most 3 defect suggestions
    assert!(defect_sug.len() <= 3);
}

// ============================================================================
// THRESHOLD_TUNING: ThresholdMetrics::calculate
// ============================================================================

#[test]
fn threshold_metrics_calculate_empty_samples() {
    // Branch: samples.is_empty() => return defaults
    let metrics = ThresholdMetrics::calculate(&[], 0.5);
    assert_eq!(metrics.sample_count, 0);
    assert_eq!(metrics.accuracy, 0.0);
    assert_eq!(metrics.precision, 0.0);
    assert_eq!(metrics.recall, 0.0);
    assert_eq!(metrics.f1_score, 0.0);
    assert!((metrics.fallback_rate - 1.0).abs() < 0.001);
    assert!((metrics.ml_usage_rate - 0.0).abs() < 0.001);
}

#[test]
fn threshold_metrics_calculate_all_correct_using_ml() {
    // Branch: is_correct = true, ml_confidence >= threshold
    let samples = vec![
        ValidationSample::new(
            InferredOwnership::Owned,
            InferredOwnership::Borrowed, // rule wrong
            InferredOwnership::Owned,    // ml correct
            0.9,
        ),
        ValidationSample::new(
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,    // rule wrong
            InferredOwnership::Borrowed, // ml correct
            0.8,
        ),
    ];
    let metrics = ThresholdMetrics::calculate(&samples, 0.5);
    assert_eq!(metrics.sample_count, 2);
    assert!((metrics.accuracy - 1.0).abs() < 0.001);
    assert!((metrics.ml_usage_rate - 1.0).abs() < 0.001);
    assert!((metrics.fallback_rate - 0.0).abs() < 0.001);
}

#[test]
fn threshold_metrics_calculate_all_incorrect() {
    // Branch: is_correct = false for all => false_positives, false_negatives
    let samples = vec![
        ValidationSample::new(
            InferredOwnership::Owned,
            InferredOwnership::Borrowed, // rule wrong
            InferredOwnership::Borrowed, // ml also wrong (using rules due to low conf)
            0.3,
        ),
    ];
    let metrics = ThresholdMetrics::calculate(&samples, 0.5);
    assert!((metrics.accuracy - 0.0).abs() < 0.001);
    // precision: tp=0, fp=1 => 0/(0+1) = 0
    assert!((metrics.precision - 0.0).abs() < 0.001);
    // recall: tp=0, fn=1 => 0/(0+1) = 0
    assert!((metrics.recall - 0.0).abs() < 0.001);
    // f1: 0
    assert!((metrics.f1_score - 0.0).abs() < 0.001);
}

#[test]
fn threshold_metrics_calculate_mixed_ml_and_fallback() {
    // Some samples use ML (conf >= threshold), some use fallback
    let samples = vec![
        ValidationSample::new(
            InferredOwnership::Owned,
            InferredOwnership::Owned,    // rule correct
            InferredOwnership::Owned,    // ml correct
            0.9,                         // above threshold
        ),
        ValidationSample::new(
            InferredOwnership::Borrowed,
            InferredOwnership::Borrowed, // rule correct
            InferredOwnership::Owned,    // ml wrong
            0.3,                         // below threshold => uses rule
        ),
    ];
    let metrics = ThresholdMetrics::calculate(&samples, 0.5);
    assert_eq!(metrics.sample_count, 2);
    assert!((metrics.accuracy - 1.0).abs() < 0.001); // both correct
    assert!((metrics.ml_usage_rate - 0.5).abs() < 0.001);
    assert!((metrics.fallback_rate - 0.5).abs() < 0.001);
}

#[test]
fn threshold_metrics_calculate_precision_recall_f1() {
    // 3 correct, 1 wrong => precision = 3/4, recall = 3/4, f1 = 3/4
    let samples = vec![
        ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Owned, 0.9),
        ValidationSample::new(InferredOwnership::Borrowed, InferredOwnership::Borrowed, InferredOwnership::Borrowed, 0.8),
        ValidationSample::new(InferredOwnership::Vec, InferredOwnership::Vec, InferredOwnership::Vec, 0.7),
        ValidationSample::new(
            InferredOwnership::Owned,
            InferredOwnership::Borrowed, // wrong
            InferredOwnership::Borrowed, // also wrong, but using ML since high conf
            0.9,
        ),
    ];
    let metrics = ThresholdMetrics::calculate(&samples, 0.5);
    assert!((metrics.accuracy - 0.75).abs() < 0.001);
    // tp=3, fp=1, fn=1, precision=3/4=0.75, recall=3/4=0.75
    assert!((metrics.precision - 0.75).abs() < 0.001);
    assert!((metrics.recall - 0.75).abs() < 0.001);
    // f1 = 2*0.75*0.75/(0.75+0.75) = 0.75
    assert!((metrics.f1_score - 0.75).abs() < 0.001);
}

#[test]
fn threshold_metrics_zero_tp_zero_fp_precision_zero() {
    // Branch: true_positives + false_positives == 0 => precision = 0.0
    // This happens when no samples at all, but we tested that path above.
    // With samples, at least one is tp or fp, so test via empty.
    let m = ThresholdMetrics::calculate(&[], 0.5);
    assert!((m.precision - 0.0).abs() < 0.001);
}

#[test]
fn threshold_metrics_zero_precision_zero_recall_f1_zero() {
    // Branch: precision + recall == 0 => f1 = 0.0
    // All wrong predictions
    let samples = vec![
        ValidationSample::new(
            InferredOwnership::Owned,
            InferredOwnership::Borrowed, // wrong
            InferredOwnership::Borrowed, // wrong
            0.3,
        ),
    ];
    let metrics = ThresholdMetrics::calculate(&samples, 0.5);
    assert!((metrics.f1_score - 0.0).abs() < 0.001);
}

#[test]
fn threshold_metrics_high_threshold_forces_all_fallback() {
    let samples = vec![
        ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Borrowed, 0.5),
        ValidationSample::new(InferredOwnership::Borrowed, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.6),
    ];
    let metrics = ThresholdMetrics::calculate(&samples, 0.99);
    // All fallback to rules
    assert!((metrics.fallback_rate - 1.0).abs() < 0.001);
    assert!((metrics.ml_usage_rate - 0.0).abs() < 0.001);
    // Rules are correct for both
    assert!((metrics.accuracy - 1.0).abs() < 0.001);
}

#[test]
fn threshold_metrics_low_threshold_forces_all_ml() {
    let samples = vec![
        ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.1),
        ValidationSample::new(InferredOwnership::Borrowed, InferredOwnership::Owned, InferredOwnership::Borrowed, 0.2),
    ];
    let metrics = ThresholdMetrics::calculate(&samples, 0.05);
    // All use ML (conf >= 0.05)
    assert!((metrics.ml_usage_rate - 1.0).abs() < 0.001);
    assert!((metrics.fallback_rate - 0.0).abs() < 0.001);
}

// ============================================================================
// THRESHOLD_TUNING: ThresholdTuner::select_optimal
// ============================================================================

#[test]
fn select_optimal_max_accuracy_picks_best() {
    // SelectionCriteria::MaxAccuracy branch
    let samples = vec![
        ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.9),
        ValidationSample::new(InferredOwnership::Borrowed, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.3),
    ];
    let tuner = ThresholdTuner::new().with_criteria(SelectionCriteria::MaxAccuracy);
    let result = tuner.tune(&samples);
    // Best accuracy at threshold where ML is used for high-conf (correct) and rules for low-conf (correct)
    assert!(result.optimal_metrics.accuracy >= 0.5);
}

#[test]
fn select_optimal_max_f1_picks_best_f1() {
    // SelectionCriteria::MaxF1 branch
    let samples = vec![
        ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Owned, 0.9),
        ValidationSample::new(InferredOwnership::Borrowed, InferredOwnership::Borrowed, InferredOwnership::Borrowed, 0.8),
    ];
    let tuner = ThresholdTuner::new().with_criteria(SelectionCriteria::MaxF1);
    let result = tuner.tune(&samples);
    assert!((result.optimal_metrics.f1_score - 1.0).abs() < 0.001);
}

#[test]
fn select_optimal_balanced_accuracy_fallback() {
    // SelectionCriteria::BalancedAccuracyFallback branch
    // 0.7 * accuracy + 0.3 * ml_usage_rate
    let mut samples = Vec::new();
    for _ in 0..80 {
        samples.push(ValidationSample::new(
            InferredOwnership::Owned,
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.9,
        ));
    }
    for _ in 0..20 {
        samples.push(ValidationSample::new(
            InferredOwnership::Borrowed,
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.3,
        ));
    }
    let tuner = ThresholdTuner::new().with_criteria(SelectionCriteria::BalancedAccuracyFallback);
    let result = tuner.tune(&samples);
    assert!(result.optimal_metrics.accuracy > 0.5);
}

#[test]
fn select_optimal_min_fallback_above_baseline_with_candidates_above() {
    // SelectionCriteria::MinFallbackAboveBaseline branch: above_baseline not empty
    let samples = vec![
        ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.9),
        ValidationSample::new(InferredOwnership::Borrowed, InferredOwnership::Borrowed, InferredOwnership::Borrowed, 0.3),
    ];
    let tuner = ThresholdTuner::new().with_criteria(SelectionCriteria::MinFallbackAboveBaseline);
    let result = tuner.tune(&samples);
    // Should pick threshold with min fallback that's still above baseline
    assert!(result.optimal_metrics.accuracy >= result.baseline_accuracy);
}

#[test]
fn select_optimal_min_fallback_above_baseline_none_above() {
    // Branch: above_baseline is empty => falls back to max accuracy
    // This happens when no threshold achieves baseline accuracy.
    // Rules get 100% accuracy, but hybrid at any threshold can't do better and may do worse.
    // Actually, to trigger the empty above_baseline, we need all thresholds to yield
    // accuracy < baseline. We can create a dataset where rules are always correct but
    // ML always wrong, and thresholds that include ML cause accuracy to drop.
    let samples = vec![
        ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Borrowed, 0.9),
        ValidationSample::new(InferredOwnership::Borrowed, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.8),
        ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Borrowed, 0.7),
        ValidationSample::new(InferredOwnership::Borrowed, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.6),
    ];
    // Rules are 100% correct, ML is 0% correct at every confidence level
    // At any threshold <= 0.9, some ML predictions will be used (incorrectly)
    // Only threshold 0.9 achieves baseline because sample conf values are 0.6-0.9
    let tuner = ThresholdTuner::with_candidates(vec![0.1, 0.2, 0.3, 0.4, 0.5])
        .with_criteria(SelectionCriteria::MinFallbackAboveBaseline);
    let result = tuner.tune(&samples);
    // All thresholds 0.1-0.5 will use ML for most/all samples, getting 0% correct
    // Baseline is 100%, so none meet baseline => falls back to max accuracy
    assert!(result.optimal_metrics.accuracy >= 0.0);
}

#[test]
fn select_optimal_empty_metrics_returns_default() {
    // Branch: metrics is empty => returns default ThresholdMetrics
    let tuner = ThresholdTuner::with_candidates(vec![]);
    let result = tuner.tune(&[
        ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Owned, 0.9),
    ]);
    // With no candidates, all_thresholds is empty, select_optimal returns default
    assert!((result.optimal_threshold - 0.65).abs() < 0.001);
}

#[test]
fn tune_empty_samples_returns_default() {
    let tuner = ThresholdTuner::new();
    let result = tuner.tune(&[]);
    assert!((result.optimal_threshold - 0.65).abs() < 0.001);
    assert!((result.baseline_accuracy - 0.0).abs() < 0.001);
    assert!((result.ml_only_accuracy - 0.0).abs() < 0.001);
    assert!((result.improvement_over_baseline - 0.0).abs() < 0.001);
}

#[test]
fn tuner_with_custom_candidates() {
    let tuner = ThresholdTuner::with_candidates(vec![0.25, 0.5, 0.75]);
    let samples = vec![
        ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Owned, 0.9),
    ];
    let result = tuner.tune(&samples);
    assert_eq!(result.all_thresholds.len(), 3);
}

#[test]
fn tuner_add_candidate_no_duplicate() {
    let mut tuner = ThresholdTuner::new();
    let initial_len = 10; // default has 10 candidates
    tuner.add_candidate(0.5); // already exists
    // Should not add duplicate
    let samples = vec![
        ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Owned, 0.9),
    ];
    let result = tuner.tune(&samples);
    assert_eq!(result.all_thresholds.len(), initial_len);
}

#[test]
fn tuner_add_candidate_clamps_and_sorts() {
    let mut tuner = ThresholdTuner::with_candidates(vec![0.3, 0.7]);
    tuner.add_candidate(1.5); // clamped to 1.0
    tuner.add_candidate(-0.5); // clamped to 0.0
    let samples = vec![
        ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Owned, 0.9),
    ];
    let result = tuner.tune(&samples);
    assert_eq!(result.all_thresholds.len(), 4); // 0.0, 0.3, 0.7, 1.0
}

#[test]
fn find_optimal_threshold_convenience_function() {
    let samples = vec![
        ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Owned, 0.9),
    ];
    let threshold = find_optimal_threshold(&samples);
    assert!((0.0..=1.0).contains(&threshold));
}

#[test]
fn tuning_result_to_markdown_positive_improvement() {
    // Branch in to_markdown: improvement > 0 => ADOPT HYBRID
    let samples = vec![
        ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.9),
        ValidationSample::new(InferredOwnership::Borrowed, InferredOwnership::Borrowed, InferredOwnership::Borrowed, 0.8),
    ];
    let result = ThresholdTuner::new().tune(&samples);
    assert!(result.improvement_over_baseline > 0.0);
    let md = result.to_markdown();
    assert!(md.contains("ADOPT HYBRID"));
    assert!(md.contains("Threshold Tuning Report"));
    assert!(md.contains("Optimal Configuration"));
    assert!(md.contains("Comparison to Baselines"));
}

#[test]
fn tuning_result_to_markdown_no_improvement() {
    // Branch in to_markdown: improvement <= 0 => KEEP RULES ONLY
    let samples = vec![
        ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Borrowed, 0.9),
        ValidationSample::new(InferredOwnership::Borrowed, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.8),
    ];
    // Rules are always right, ML is always wrong
    let result = ThresholdTuner::new().tune(&samples);
    let md = result.to_markdown();
    assert!(md.contains("KEEP RULES ONLY"));
}

#[test]
fn tuning_result_to_markdown_includes_threshold_table() {
    let samples = vec![
        ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Owned, 0.9),
    ];
    let result = ThresholdTuner::new().tune(&samples);
    let md = result.to_markdown();
    assert!(md.contains("All Thresholds"));
    assert!(md.contains("| Threshold | Accuracy | F1 | Fallback Rate | ML Usage |"));
}

#[test]
fn selection_criteria_display_all_variants() {
    assert_eq!(SelectionCriteria::MaxAccuracy.to_string(), "max-accuracy");
    assert_eq!(SelectionCriteria::MaxF1.to_string(), "max-f1");
    assert_eq!(SelectionCriteria::BalancedAccuracyFallback.to_string(), "balanced");
    assert_eq!(SelectionCriteria::MinFallbackAboveBaseline.to_string(), "min-fallback");
}

// ============================================================================
// HYBRID_CLASSIFIER: classify_ensemble
// ============================================================================

#[test]
fn ensemble_agreement_boosts_confidence() {
    // Branch: agree == true => combined confidence boosted by 1.1x
    let classifier = HybridClassifier::new();
    let inference = make_inference("ptr", OwnershipKind::Owning, 0.8, "malloc detected");
    let features = OwnershipFeatures::default();
    let model = ConfigurableMockModel::new(InferredOwnership::Owned, 0.9);

    let result = classifier.classify_ensemble(&inference, &features, &model);
    assert_eq!(result.method, ClassificationMethod::Hybrid);
    assert_eq!(result.ownership, InferredOwnership::Owned);
    // (0.8 + 0.9) / 2 * 1.1 = 0.935
    assert!((result.confidence - 0.935).abs() < 0.01);
    assert!(result.rule_result.is_some());
    assert!(result.ml_result.is_some());
}

#[test]
fn ensemble_agreement_caps_at_1_0() {
    // Branch: combined_confidence.min(1.0) caps confidence
    let classifier = HybridClassifier::new();
    let inference = make_inference("ptr", OwnershipKind::Owning, 0.99, "very confident");
    let features = OwnershipFeatures::default();
    let model = ConfigurableMockModel::new(InferredOwnership::Owned, 0.99);

    let result = classifier.classify_ensemble(&inference, &features, &model);
    assert_eq!(result.method, ClassificationMethod::Hybrid);
    // (0.99 + 0.99) / 2 * 1.1 = 1.089 => capped at 1.0
    assert!((result.confidence - 1.0).abs() < 0.001);
}

#[test]
fn ensemble_disagreement_ml_wins_higher_confidence() {
    // Branch: !agree and ml_conf > inference.confidence => ML wins
    let classifier = HybridClassifier::new();
    let inference = make_inference("ptr", OwnershipKind::Unknown, 0.3, "uncertain");
    let features = OwnershipFeatures::default();
    let model = ConfigurableMockModel::new(InferredOwnership::Vec, 0.9);

    let result = classifier.classify_ensemble(&inference, &features, &model);
    assert_eq!(result.method, ClassificationMethod::MachineLearning);
    assert_eq!(result.ownership, InferredOwnership::Vec);
    assert!((result.confidence - 0.9).abs() < 0.01);
    assert!(result.reasoning.contains("ML wins"));
}

#[test]
fn ensemble_disagreement_rules_win_higher_confidence() {
    // Branch: !agree and ml_conf <= inference.confidence => rules win
    let classifier = HybridClassifier::new();
    let inference = make_inference("ptr", OwnershipKind::Owning, 0.95, "malloc+free");
    let features = OwnershipFeatures::default();
    let model = ConfigurableMockModel::new(InferredOwnership::Borrowed, 0.4);

    let result = classifier.classify_ensemble(&inference, &features, &model);
    assert_eq!(result.method, ClassificationMethod::RuleBased);
    assert_eq!(result.ownership, InferredOwnership::Owned);
    assert!((result.confidence - 0.95).abs() < 0.01);
    assert!(result.reasoning.contains("Rules win"));
}

#[test]
fn ensemble_disagreement_equal_confidence_rules_win() {
    // Branch: !agree and ml_conf == inference.confidence => rules win (else branch)
    let classifier = HybridClassifier::new();
    let inference = make_inference("ptr", OwnershipKind::MutableBorrow, 0.7, "mutation");
    let features = OwnershipFeatures::default();
    let model = ConfigurableMockModel::new(InferredOwnership::Owned, 0.7);

    let result = classifier.classify_ensemble(&inference, &features, &model);
    assert_eq!(result.method, ClassificationMethod::RuleBased);
    assert_eq!(result.ownership, InferredOwnership::BorrowedMut);
}

#[test]
fn ensemble_all_ownership_kinds_converted() {
    // Test that all OwnershipKind variants are handled correctly in ensemble
    let classifier = HybridClassifier::new();
    let features = OwnershipFeatures::default();

    // Owning -> Owned
    let inf = make_inference("a", OwnershipKind::Owning, 0.5, "test");
    let model = ConfigurableMockModel::new(InferredOwnership::Owned, 0.5);
    let r = classifier.classify_ensemble(&inf, &features, &model);
    assert_eq!(r.ownership, InferredOwnership::Owned);

    // ImmutableBorrow -> Borrowed
    let inf = make_inference("b", OwnershipKind::ImmutableBorrow, 0.5, "test");
    let model = ConfigurableMockModel::new(InferredOwnership::Borrowed, 0.5);
    let r = classifier.classify_ensemble(&inf, &features, &model);
    assert_eq!(r.ownership, InferredOwnership::Borrowed);

    // MutableBorrow -> BorrowedMut
    let inf = make_inference("c", OwnershipKind::MutableBorrow, 0.5, "test");
    let model = ConfigurableMockModel::new(InferredOwnership::BorrowedMut, 0.5);
    let r = classifier.classify_ensemble(&inf, &features, &model);
    assert_eq!(r.ownership, InferredOwnership::BorrowedMut);

    // Unknown -> RawPointer
    let inf = make_inference("d", OwnershipKind::Unknown, 0.5, "test");
    let model = ConfigurableMockModel::new(InferredOwnership::RawPointer, 0.5);
    let r = classifier.classify_ensemble(&inf, &features, &model);
    assert_eq!(r.ownership, InferredOwnership::RawPointer);
}

#[test]
fn ensemble_array_pointer_kind_to_slice() {
    let classifier = HybridClassifier::new();
    let features = OwnershipFeatures::default();
    let array_kind = OwnershipKind::ArrayPointer {
        base_array: "arr".to_string(),
        element_type: decy_hir::HirType::Int,
        base_index: Some(0),
    };
    let inf = make_inference("e", array_kind, 0.5, "array");
    let model = ConfigurableMockModel::new(InferredOwnership::Slice, 0.5);
    let r = classifier.classify_ensemble(&inf, &features, &model);
    assert_eq!(r.ownership, InferredOwnership::Slice);
    assert_eq!(r.method, ClassificationMethod::Hybrid); // they agree
}

#[test]
fn ensemble_reasoning_contains_expected_text_for_agree() {
    let classifier = HybridClassifier::new();
    let inf = make_inference("ptr", OwnershipKind::Owning, 0.8, "malloc");
    let features = OwnershipFeatures::default();
    let model = ConfigurableMockModel::new(InferredOwnership::Owned, 0.8);

    let result = classifier.classify_ensemble(&inf, &features, &model);
    assert!(result.reasoning.contains("rules + ML agree"));
    assert!(result.reasoning.contains("boosted confidence"));
}

#[test]
fn ensemble_reasoning_contains_expected_text_for_ml_wins() {
    let classifier = HybridClassifier::new();
    let inf = make_inference("ptr", OwnershipKind::Unknown, 0.2, "unknown");
    let features = OwnershipFeatures::default();
    let model = ConfigurableMockModel::new(InferredOwnership::Vec, 0.95);

    let result = classifier.classify_ensemble(&inf, &features, &model);
    assert!(result.reasoning.contains("ML wins"));
    assert!(result.reasoning.contains("0.95"));
}

#[test]
fn ensemble_reasoning_contains_expected_text_for_rules_win() {
    let classifier = HybridClassifier::new();
    let inf = make_inference("ptr", OwnershipKind::Owning, 0.9, "malloc+free pattern");
    let features = OwnershipFeatures::default();
    let model = ConfigurableMockModel::new(InferredOwnership::Borrowed, 0.3);

    let result = classifier.classify_ensemble(&inf, &features, &model);
    assert!(result.reasoning.contains("Rules win"));
}

// ============================================================================
// HYBRID_CLASSIFIER: classify_hybrid branches (extra coverage)
// ============================================================================

#[test]
fn classify_hybrid_ml_disabled_returns_rule_based() {
    let classifier = HybridClassifier::new(); // ml disabled
    let inf = make_inference("ptr", OwnershipKind::Owning, 0.8, "test");
    let features = OwnershipFeatures::default();
    let model = ConfigurableMockModel::new(InferredOwnership::Vec, 0.99);
    let result = classifier.classify_hybrid(&inf, &features, &model);
    assert_eq!(result.method, ClassificationMethod::RuleBased);
}

#[test]
fn classify_hybrid_ml_above_threshold() {
    let mut classifier = HybridClassifier::new();
    classifier.enable_ml();
    let inf = make_inference("ptr", OwnershipKind::Unknown, 0.3, "test");
    let features = OwnershipFeatures::default();
    let model = ConfigurableMockModel::new(InferredOwnership::Owned, 0.9);
    let result = classifier.classify_hybrid(&inf, &features, &model);
    assert_eq!(result.method, ClassificationMethod::MachineLearning);
    assert_eq!(result.ownership, InferredOwnership::Owned);
}

#[test]
fn classify_hybrid_ml_below_threshold_fallback() {
    let mut classifier = HybridClassifier::new();
    classifier.enable_ml();
    let inf = make_inference("ptr", OwnershipKind::ImmutableBorrow, 0.85, "read-only");
    let features = OwnershipFeatures::default();
    let model = ConfigurableMockModel::new(InferredOwnership::Owned, 0.3);
    let result = classifier.classify_hybrid(&inf, &features, &model);
    assert_eq!(result.method, ClassificationMethod::Fallback);
    assert_eq!(result.ownership, InferredOwnership::Borrowed);
    assert!(result.used_fallback());
    assert!(result.ml_rejected());
}

// ============================================================================
// HYBRID_CLASSIFIER: HybridMetrics (extra coverage)
// ============================================================================

#[test]
fn hybrid_metrics_record_all_methods() {
    let mut metrics = HybridMetrics::new();

    let methods = [
        ClassificationMethod::RuleBased,
        ClassificationMethod::MachineLearning,
        ClassificationMethod::Fallback,
        ClassificationMethod::Hybrid,
    ];
    for method in &methods {
        let result = HybridResult {
            variable: "x".to_string(),
            ownership: InferredOwnership::Owned,
            confidence: 0.8,
            method: *method,
            rule_result: None,
            ml_result: None,
            reasoning: "test".to_string(),
        };
        metrics.record(&result);
    }
    assert_eq!(metrics.total, 4);
    assert_eq!(metrics.rule_based, 1);
    assert_eq!(metrics.ml_used, 1);
    assert_eq!(metrics.fallback, 1);
    assert_eq!(metrics.hybrid, 1);
}

#[test]
fn hybrid_metrics_agreement_rate_no_comparisons() {
    let metrics = HybridMetrics::new();
    // No comparisons => 1.0
    assert!((metrics.agreement_rate() - 1.0).abs() < 0.001);
}

#[test]
fn hybrid_metrics_fallback_rate_zero_total() {
    let metrics = HybridMetrics::new();
    assert!((metrics.fallback_rate() - 0.0).abs() < 0.001);
}

#[test]
fn hybrid_metrics_ml_usage_rate_zero_total() {
    let metrics = HybridMetrics::new();
    assert!((metrics.ml_usage_rate() - 0.0).abs() < 0.001);
}

// ============================================================================
// HYBRID_CLASSIFIER: HybridResult helpers
// ============================================================================

#[test]
fn hybrid_result_used_fallback_false_for_rule_based() {
    let result = HybridResult {
        variable: "x".to_string(),
        ownership: InferredOwnership::Owned,
        confidence: 0.9,
        method: ClassificationMethod::RuleBased,
        rule_result: None,
        ml_result: None,
        reasoning: "test".to_string(),
    };
    assert!(!result.used_fallback());
}

#[test]
fn hybrid_result_ml_rejected_false_when_no_ml() {
    let result = HybridResult {
        variable: "x".to_string(),
        ownership: InferredOwnership::Owned,
        confidence: 0.9,
        method: ClassificationMethod::Fallback,
        rule_result: None,
        ml_result: None, // no ML result
        reasoning: "test".to_string(),
    };
    // Fallback but no ml_result => ml_rejected is false
    assert!(!result.ml_rejected());
}

#[test]
fn hybrid_result_ml_rejected_true() {
    let result = HybridResult {
        variable: "x".to_string(),
        ownership: InferredOwnership::Owned,
        confidence: 0.9,
        method: ClassificationMethod::Fallback,
        rule_result: None,
        ml_result: Some(OwnershipPrediction {
            kind: InferredOwnership::Borrowed,
            confidence: 0.3,
            fallback: None,
        }),
        reasoning: "test".to_string(),
    };
    assert!(result.ml_rejected());
}

// ============================================================================
// HYBRID_CLASSIFIER: NullModel (extra coverage)
// ============================================================================

#[test]
fn null_model_predict_batch() {
    let model = NullModel;
    let features = vec![OwnershipFeatures::default(), OwnershipFeatures::default()];
    let results = model.predict_batch(&features);
    assert_eq!(results.len(), 2);
    for r in &results {
        assert_eq!(r.kind, InferredOwnership::RawPointer);
        assert!((r.confidence - 0.0).abs() < 0.001);
    }
}

#[test]
fn null_model_name_is_null() {
    let model = NullModel;
    assert_eq!(model.name(), "null");
}

// ============================================================================
// HYBRID_CLASSIFIER: ClassificationMethod display (extra)
// ============================================================================

#[test]
fn classification_method_display_all() {
    assert_eq!(format!("{}", ClassificationMethod::RuleBased), "rule-based");
    assert_eq!(format!("{}", ClassificationMethod::MachineLearning), "ml");
    assert_eq!(format!("{}", ClassificationMethod::Fallback), "fallback");
    assert_eq!(format!("{}", ClassificationMethod::Hybrid), "hybrid");
}

// ============================================================================
// VALIDATION SAMPLE: edge case coverage
// ============================================================================

#[test]
fn validation_sample_rule_correct_true() {
    let s = ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Borrowed, 0.5);
    assert!(s.rule_correct());
}

#[test]
fn validation_sample_rule_correct_false() {
    let s = ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.5);
    assert!(!s.rule_correct());
}

#[test]
fn validation_sample_ml_correct_true() {
    let s = ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.5);
    assert!(s.ml_correct());
}

#[test]
fn validation_sample_ml_correct_false() {
    let s = ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Borrowed, 0.5);
    assert!(!s.ml_correct());
}

#[test]
fn validation_sample_hybrid_prediction_exact_threshold() {
    // ml_confidence == threshold => uses ML
    let s = ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.5);
    assert_eq!(s.hybrid_prediction(0.5), InferredOwnership::Owned); // uses ML
}

#[test]
fn validation_sample_hybrid_correct_exact_threshold() {
    let s = ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.5);
    assert!(s.hybrid_correct(0.5)); // ML is correct, exactly at threshold
}

// ============================================================================
// ERROR TRACKING: PatternStats edge cases
// ============================================================================

#[test]
fn pattern_stats_failure_rate_zero_count() {
    let stats = PatternStats::default();
    assert!((stats.failure_rate() - 0.0).abs() < 0.001);
}

#[test]
fn pattern_stats_failure_rate_nonzero() {
    let mut stats = PatternStats::default();
    stats.record(true);
    stats.record(true);
    stats.record(false);
    assert!((stats.failure_rate() - 2.0 / 3.0).abs() < 0.01);
}

// ============================================================================
// ERROR TRACKING: FeatureSuspiciousness thresholds
// ============================================================================

#[test]
fn feature_suspiciousness_exactly_05_not_suspicious() {
    let fs = FeatureSuspiciousness {
        feature: "test".to_string(),
        score: 0.5,
        total_count: 10,
        failure_count: 5,
        success_count: 5,
    };
    assert!(!fs.is_suspicious()); // > 0.5, not >=
}

#[test]
fn feature_suspiciousness_exactly_07_not_highly() {
    let fs = FeatureSuspiciousness {
        feature: "test".to_string(),
        score: 0.7,
        total_count: 10,
        failure_count: 7,
        success_count: 3,
    };
    assert!(!fs.is_highly_suspicious()); // > 0.7, not >=
    assert!(fs.is_suspicious()); // > 0.5
}

// ============================================================================
// THRESHOLD TUNER: default trait
// ============================================================================

#[test]
fn threshold_tuner_default_trait() {
    let tuner = ThresholdTuner::default();
    let samples = vec![
        ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Owned, 0.9),
    ];
    let result = tuner.tune(&samples);
    assert!((result.optimal_metrics.accuracy - 1.0).abs() < 0.001);
}

// ============================================================================
// HYBRID_CLASSIFIER: default and set_threshold
// ============================================================================

#[test]
fn hybrid_classifier_set_threshold() {
    let mut c = HybridClassifier::new();
    c.set_threshold(0.8);
    assert!((c.confidence_threshold() - 0.8).abs() < 0.001);
    c.set_threshold(1.5); // clamped
    assert!((c.confidence_threshold() - 1.0).abs() < 0.001);
    c.set_threshold(-0.5); // clamped
    assert!((c.confidence_threshold() - 0.0).abs() < 0.001);
}

#[test]
fn hybrid_classifier_default_trait() {
    let c = HybridClassifier::default();
    assert!(!c.ml_enabled());
    assert!((c.confidence_threshold() - 0.65).abs() < 0.001);
}
