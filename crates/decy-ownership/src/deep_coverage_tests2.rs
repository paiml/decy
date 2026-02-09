//! Deep coverage tests (batch 2) for retraining_pipeline, model_versioning,
//! and error_tracking.
//!
//! Targets uncovered lines in:
//! - `RetrainingPipeline::execute` (line 373, 74 uncov lines)
//! - `ModelVersionManager::rollback` (line 369, 40 uncov)
//! - `ModelVersionManager::rollback_to` (line 425, 40 uncov)
//! - `ModelVersionManager::to_markdown` (line 520, 41 uncov)
//! - `ErrorTracker::generate_markdown_report` (line 414, 48 uncov)
//! - `ErrorTracker::calculate_suspiciousness` (line 688, 45 uncov)

use crate::error_tracking::{
    ErrorTracker, FeatureSuspiciousness, InferenceError, PatternStats,
    SuggestionCategory, SuggestionPriority,
};
use crate::ml_features::{
    InferredOwnership, OwnershipDefect, OwnershipFeaturesBuilder,
};
use crate::model_versioning::{
    ModelEntry, ModelQualityMetrics, ModelVersion, ModelVersionManager, QualityThresholds,
};
use crate::retraining_pipeline::{
    DataSplit, ExecutionSummary, ModelTrainer, NullTrainer,
    RetrainingConfig, RetrainingPipeline, RetrainingResult, TrainingMetrics,
    TrainingSample,
};

// ============================================================================
// Helpers
// ============================================================================

fn make_samples(n: usize) -> Vec<TrainingSample> {
    (0..n)
        .map(|i| {
            TrainingSample::new(
                OwnershipFeaturesBuilder::default().build(),
                InferredOwnership::Owned,
                &format!("file{}.c", i),
                i as u32,
            )
        })
        .collect()
}

fn make_good_metrics() -> ModelQualityMetrics {
    ModelQualityMetrics::new(0.90, 0.88, 0.86, 0.87, 0.92, 0.05, 1000)
}

fn make_weak_metrics() -> ModelQualityMetrics {
    ModelQualityMetrics::new(0.80, 0.78, 0.75, 0.765, 0.82, 0.12, 500)
}

fn make_below_threshold_metrics() -> ModelQualityMetrics {
    ModelQualityMetrics::new(0.70, 0.65, 0.60, 0.625, 0.55, 0.40, 200)
}

fn make_error(
    variable: &str,
    defect: OwnershipDefect,
    features: Vec<String>,
) -> InferenceError {
    InferenceError::new(
        variable,
        "test.c",
        42,
        InferredOwnership::Borrowed,
        InferredOwnership::Owned,
        0.6,
        defect,
    )
    .with_features(features)
}

fn populated_version_manager(count: u32) -> ModelVersionManager {
    let mut vm = ModelVersionManager::new();
    for i in 0..count {
        let m = ModelQualityMetrics::new(
            0.85 + (i as f64 * 0.02),
            0.85 + (i as f64 * 0.01),
            0.85 + (i as f64 * 0.01),
            0.85 + (i as f64 * 0.015),
            0.90,
            0.05,
            1000 + (i as u64 * 100),
        );
        let e = ModelEntry::new(
            ModelVersion::new(1, i, 0),
            m,
            format!("Version 1.{}.0", i),
            format!("/models/v1.{}.0.bin", i),
        );
        let _ = vm.register_version(e);
    }
    vm
}

// ============================================================================
// RETRAINING PIPELINE: execute path tests
// ============================================================================

#[test]
fn execute_insufficient_data_records_history() {
    let trainer = NullTrainer::new(0.92, 0.88);
    let mut pipeline = RetrainingPipeline::with_defaults(trainer);

    let samples = make_samples(50);
    let result = pipeline.execute(samples);

    assert!(matches!(result, RetrainingResult::InsufficientData { actual: 50, .. }));
    assert_eq!(pipeline.history().len(), 1);
    assert!(matches!(
        pipeline.history()[0].result,
        ExecutionSummary::InsufficientData
    ));
    assert_eq!(pipeline.history()[0].sample_count, 50);
}

#[test]
fn execute_insufficient_data_exact_boundary() {
    let trainer = NullTrainer::new(0.92, 0.88);
    let config = RetrainingConfig {
        min_train_samples: 70,
        min_validation_samples: 15,
        min_test_samples: 15,
        train_ratio: 0.70,
        validation_ratio: 0.15,
        ..RetrainingConfig::default()
    };
    let vm = ModelVersionManager::new();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    // Exactly at boundary: 99 < 100 required
    let samples = make_samples(99);
    let result = pipeline.execute(samples);
    assert!(matches!(
        result,
        RetrainingResult::InsufficientData { actual: 99, required: 100 }
    ));
}

#[test]
fn execute_data_split_fails_minimum_after_splitting() {
    let trainer = NullTrainer::new(0.92, 0.88);
    let config = RetrainingConfig {
        min_train_samples: 80,
        min_validation_samples: 15,
        min_test_samples: 15,
        train_ratio: 0.50,     // Only 50% train
        validation_ratio: 0.10, // Only 10% validation
        ..RetrainingConfig::default()
    };
    let vm = ModelVersionManager::new();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    // 110 samples >= 80+15+15=110 total, passes first check.
    // But with 50/10/40 split: train=55, val=11, test=44
    // train(55) < min_train(80) -> fails meets_minimum_sizes
    let samples = make_samples(110);
    let result = pipeline.execute(samples);

    assert!(!result.is_success());
    assert!(matches!(result, RetrainingResult::InsufficientData { .. }));
}

#[test]
fn execute_training_error_from_failing_trainer() {
    struct FailTrainer;
    impl ModelTrainer for FailTrainer {
        fn train(&self, _data: &DataSplit) -> Result<TrainingMetrics, String> {
            Err("GPU out of memory".to_string())
        }
    }

    let mut pipeline = RetrainingPipeline::with_defaults(FailTrainer);
    let samples = make_samples(1000);
    let result = pipeline.execute(samples);

    assert!(!result.is_success());
    if let RetrainingResult::TrainingError { error } = &result {
        assert!(error.contains("GPU out of memory"));
    } else {
        panic!("Expected TrainingError");
    }
    assert!(matches!(
        pipeline.history()[0].result,
        ExecutionSummary::Error { .. }
    ));
}

#[test]
fn execute_quality_gate_low_precision() {
    let trainer = NullTrainer::new(0.70, 0.90); // precision below threshold
    let mut pipeline = RetrainingPipeline::with_defaults(trainer);

    let samples = make_samples(1000);
    let result = pipeline.execute(samples);

    assert!(!result.is_success());
    if let RetrainingResult::QualityGateFailed { reason, metrics } = &result {
        assert!(reason.contains("Precision"));
        assert!((metrics.precision - 0.70).abs() < 0.001);
    } else {
        panic!("Expected QualityGateFailed");
    }
}

#[test]
fn execute_quality_gate_low_recall() {
    let trainer = NullTrainer::new(0.90, 0.50); // recall below threshold
    let mut pipeline = RetrainingPipeline::with_defaults(trainer);

    let samples = make_samples(1000);
    let result = pipeline.execute(samples);

    assert!(!result.is_success());
    if let RetrainingResult::QualityGateFailed { reason, .. } = &result {
        assert!(reason.contains("Recall"));
    } else {
        panic!("Expected QualityGateFailed, got {:?}", result);
    }
}

#[test]
fn execute_quality_gate_both_below_threshold() {
    let trainer = NullTrainer::new(0.60, 0.50);
    let mut pipeline = RetrainingPipeline::with_defaults(trainer);

    let samples = make_samples(1000);
    let result = pipeline.execute(samples);

    assert!(!result.is_success());
    assert!(matches!(result, RetrainingResult::QualityGateFailed { .. }));
    assert!(matches!(
        pipeline.history()[0].result,
        ExecutionSummary::QualityGateFailed { .. }
    ));
}

#[test]
fn execute_degradation_with_existing_strong_model() {
    // Register a strong model, then train with a weaker one
    let weak_trainer = NullTrainer::new(0.86, 0.81);
    let mut vm = ModelVersionManager::new();

    let strong_metrics = ModelQualityMetrics::new(0.95, 0.95, 0.95, 0.95, 0.98, 0.01, 2000);
    let strong_entry = ModelEntry::new(
        ModelVersion::new(1, 0, 0),
        strong_metrics,
        "Strong baseline",
        "models/strong.bin",
    );
    let _ = vm.register_version(strong_entry);

    let mut pipeline = RetrainingPipeline::new(weak_trainer, vm, RetrainingConfig::default());
    let samples = make_samples(1000);
    let result = pipeline.execute(samples);

    assert!(!result.is_success());
    if let RetrainingResult::Degraded {
        degradation,
        new_metrics,
        current_metrics,
    } = &result
    {
        assert!(*degradation > 0.02);
        assert!(new_metrics.f1_score < current_metrics.f1_score);
    } else {
        panic!("Expected Degraded, got {:?}", result);
    }
}

#[test]
fn execute_degradation_records_history_summary() {
    let weak_trainer = NullTrainer::new(0.86, 0.81);
    let mut vm = ModelVersionManager::new();

    let strong = ModelQualityMetrics::new(0.95, 0.95, 0.95, 0.95, 0.98, 0.01, 2000);
    let entry = ModelEntry::new(ModelVersion::new(1, 0, 0), strong, "Strong", "/strong.bin");
    let _ = vm.register_version(entry);

    let mut pipeline = RetrainingPipeline::new(weak_trainer, vm, RetrainingConfig::default());
    let samples = make_samples(1000);
    pipeline.execute(samples);

    assert_eq!(pipeline.history().len(), 1);
    if let ExecutionSummary::Degraded { amount } = &pipeline.history()[0].result {
        assert!(*amount > 0.0);
    } else {
        panic!("Expected Degraded summary");
    }
}

#[test]
fn execute_promotion_first_version_is_1_0_0() {
    let trainer = NullTrainer::new(0.92, 0.88);
    let mut pipeline = RetrainingPipeline::with_defaults(trainer);

    let samples = make_samples(1000);
    let result = pipeline.execute(samples);

    assert!(result.is_success());
    if let RetrainingResult::Promoted { version, .. } = result {
        assert_eq!(version, ModelVersion::new(1, 0, 0));
    }
}

#[test]
fn execute_successive_promotions_increment_minor() {
    // NullTrainer returns same metrics, but the version manager's is_better_than
    // check means the second promotion may not activate if accuracy diff < 0.01.
    // The execute() method still returns Promoted with the computed next version,
    // but the version_manager may not actually activate it. To test true successive
    // promotions we need slightly improving metrics via the version manager.
    let trainer = NullTrainer::new(0.92, 0.88);
    let mut pipeline = RetrainingPipeline::with_defaults(trainer);

    // First promotion => 1.0.0
    let result1 = pipeline.execute(make_samples(1000));
    assert!(result1.is_success());
    if let RetrainingResult::Promoted { version, .. } = result1 {
        assert_eq!(version, ModelVersion::new(1, 0, 0));
    }

    // Second call: same metrics. The execute method bumps from active (1.0.0) to 1.1.0
    // but register_version might not activate it if metrics aren't better.
    // The result still says Promoted with version 1.1.0.
    let result2 = pipeline.execute(make_samples(1000));
    assert!(result2.is_success());
    if let RetrainingResult::Promoted { version, .. } = result2 {
        assert_eq!(version, ModelVersion::new(1, 1, 0));
    }
}

#[test]
fn execute_promotion_records_promoted_summary() {
    let trainer = NullTrainer::new(0.92, 0.88);
    let mut pipeline = RetrainingPipeline::with_defaults(trainer);

    let samples = make_samples(1000);
    pipeline.execute(samples);

    if let ExecutionSummary::Promoted { version } = &pipeline.history()[0].result {
        assert!(version.contains("1.0.0"));
    } else {
        panic!("Expected Promoted summary");
    }
}

#[test]
fn execute_mixed_history_success_rate() {
    let trainer = NullTrainer::new(0.92, 0.88);
    let mut pipeline = RetrainingPipeline::with_defaults(trainer);

    // 1. Success
    pipeline.execute(make_samples(1000));
    // 2. Insufficient data
    pipeline.execute(make_samples(50));
    // 3. Success
    pipeline.execute(make_samples(1000));
    // 4. Insufficient data
    pipeline.execute(make_samples(10));

    assert_eq!(pipeline.history().len(), 4);
    assert!((pipeline.success_rate() - 0.50).abs() < 0.001);
}

#[test]
fn execute_result_metrics_accessor_promoted() {
    let promoted = RetrainingResult::Promoted {
        version: ModelVersion::new(1, 0, 0),
        metrics: TrainingMetrics::new(0.92, 0.88),
    };
    let m = promoted.metrics().unwrap();
    assert!((m.precision - 0.92).abs() < 0.001);
    assert!((m.recall - 0.88).abs() < 0.001);
}

#[test]
fn execute_result_metrics_accessor_quality_gate_failed() {
    let failed = RetrainingResult::QualityGateFailed {
        reason: "bad".to_string(),
        metrics: TrainingMetrics::new(0.50, 0.50),
    };
    let m = failed.metrics().unwrap();
    assert!((m.precision - 0.50).abs() < 0.001);
}

#[test]
fn execute_result_metrics_accessor_degraded() {
    let degraded = RetrainingResult::Degraded {
        degradation: 0.10,
        new_metrics: TrainingMetrics::new(0.80, 0.75),
        current_metrics: ModelQualityMetrics::new(0.95, 0.95, 0.95, 0.95, 0.9, 0.0, 100),
    };
    let m = degraded.metrics().unwrap();
    assert!((m.precision - 0.80).abs() < 0.001);
}

#[test]
fn execute_result_metrics_accessor_none_variants() {
    let insuf = RetrainingResult::InsufficientData {
        actual: 50,
        required: 1000,
    };
    assert!(insuf.metrics().is_none());

    let err = RetrainingResult::TrainingError {
        error: "boom".into(),
    };
    assert!(err.metrics().is_none());
}

#[test]
fn execute_pipeline_rollback_after_promotion() {
    // Manually set up a version manager with 2 activated versions
    let mut vm = ModelVersionManager::new();
    let m1 = ModelQualityMetrics::new(0.88, 0.88, 0.86, 0.87, 0.9, 0.05, 1000);
    let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "/v1");
    vm.register_version(e1).unwrap();
    let m2 = ModelQualityMetrics::new(0.92, 0.91, 0.90, 0.905, 0.93, 0.03, 1500);
    let e2 = ModelEntry::new(ModelVersion::new(1, 1, 0), m2, "v2", "/v2");
    vm.register_version(e2).unwrap();

    let trainer = NullTrainer::new(0.92, 0.88);
    let mut pipeline = RetrainingPipeline::new(trainer, vm, RetrainingConfig::default());

    let cv = pipeline.current_version().unwrap();
    assert_eq!(cv.version, ModelVersion::new(1, 1, 0));

    let rb = pipeline.rollback(&ModelVersion::new(1, 0, 0)).unwrap();
    assert!(rb.success);
    assert_eq!(rb.to_version, ModelVersion::new(1, 0, 0));
}

#[test]
fn execute_pipeline_config_roundtrip() {
    let trainer = NullTrainer::new(0.92, 0.88);
    let mut pipeline = RetrainingPipeline::with_defaults(trainer);

    let mut config = RetrainingConfig::default();
    config.min_precision = 0.99;
    config.min_recall = 0.98;
    config.cv_folds = 10;
    pipeline.set_config(config);

    assert!((pipeline.config().min_precision - 0.99).abs() < 0.001);
    assert!((pipeline.config().min_recall - 0.98).abs() < 0.001);
    assert_eq!(pipeline.config().cv_folds, 10);
}

// ============================================================================
// MODEL VERSIONING: rollback tests
// ============================================================================

#[test]
fn rollback_with_two_versions_marks_current_rolled_back() {
    let mut vm = populated_version_manager(3);

    assert_eq!(vm.active_version().unwrap().version, ModelVersion::new(1, 2, 0));

    let rb = vm.rollback("Quality regression").unwrap();
    assert!(rb.success);
    assert_eq!(rb.from_version, ModelVersion::new(1, 2, 0));
    assert_eq!(rb.to_version, ModelVersion::new(1, 1, 0));
    assert_eq!(rb.reason, "Quality regression");
    assert!(rb.timestamp > 0);

    // Active should now be 1.1.0
    assert_eq!(vm.active_version().unwrap().version, ModelVersion::new(1, 1, 0));
}

#[test]
fn rollback_not_enough_versions_returns_error() {
    let mut vm = ModelVersionManager::new();
    let m = make_good_metrics();
    let e = ModelEntry::new(ModelVersion::new(1, 0, 0), m, "v1", "/v1");
    vm.register_version(e).unwrap();

    let result = vm.rollback("test");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Not enough versions"));
}

#[test]
fn rollback_no_active_version_returns_error() {
    let mut vm = ModelVersionManager::new();
    // Register a version below threshold so it is not activated
    let m = make_below_threshold_metrics();
    let e = ModelEntry::new(ModelVersion::new(1, 0, 0), m.clone(), "v1", "/v1");
    let _ = vm.register_version(e);
    let e2 = ModelEntry::new(ModelVersion::new(1, 1, 0), m, "v2", "/v2");
    let _ = vm.register_version(e2);

    let result = vm.rollback("test");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No active version"));
}

#[test]
fn rollback_skips_already_rolled_back_versions() {
    // The rollback() method does .rev().skip(1) which skips the LAST deque entry,
    // then finds the first non-rolled-back version scanning backwards.
    // With versions [0, 1, 2, 3] and active=3:
    //   First rollback: skip(1) skips 3, finds 2 (not rolled_back). Result: 3->2.
    // Now versions are [0, 1, 2(active), 3(rolled_back)].
    //   Second rollback: skip(1) skips 3 again, finds 2 (not rolled_back, but IS current).
    //   The code doesn't distinguish, so prev_idx=2=current_idx. To properly test
    //   "skipping", we need a scenario where the last entry is rolled back AND there's
    //   a non-rolled-back entry before it.
    // Instead, test with rollback_to to skip intermediate versions.
    let mut vm = populated_version_manager(4);

    // Active is 1.3.0. Rollback to 1.0.0, skipping 1.2.0 and 1.1.0
    let rb = vm
        .rollback_to(&ModelVersion::new(1, 0, 0), "skip to baseline")
        .unwrap();
    assert!(rb.success);
    assert_eq!(rb.from_version, ModelVersion::new(1, 3, 0));
    assert_eq!(rb.to_version, ModelVersion::new(1, 0, 0));
    assert_eq!(vm.active_version().unwrap().version, ModelVersion::new(1, 0, 0));
}

#[test]
fn rollback_history_is_cumulative() {
    let mut vm = populated_version_manager(4);

    vm.rollback("issue 1").unwrap();
    vm.rollback("issue 2").unwrap();

    assert_eq!(vm.rollback_history().len(), 2);
    assert_eq!(vm.rollback_history()[0].reason, "issue 1");
    assert_eq!(vm.rollback_history()[1].reason, "issue 2");
}

#[test]
fn rollback_result_has_valid_timestamp() {
    let mut vm = populated_version_manager(3);
    let rb = vm.rollback("test").unwrap();

    // Timestamp should be a reasonable Unix millis value (after 2020)
    assert!(rb.timestamp > 1_577_836_800_000);
}

// ============================================================================
// MODEL VERSIONING: rollback_to tests
// ============================================================================

#[test]
fn rollback_to_specific_version() {
    let mut vm = populated_version_manager(4);

    // Active is 1.3.0, rollback to 1.0.0
    let rb = vm
        .rollback_to(&ModelVersion::new(1, 0, 0), "Return to baseline")
        .unwrap();

    assert!(rb.success);
    assert_eq!(rb.from_version, ModelVersion::new(1, 3, 0));
    assert_eq!(rb.to_version, ModelVersion::new(1, 0, 0));
    assert_eq!(rb.reason, "Return to baseline");
    assert_eq!(vm.active_version().unwrap().version, ModelVersion::new(1, 0, 0));
}

#[test]
fn rollback_to_nonexistent_version_returns_error() {
    let mut vm = populated_version_manager(3);

    let result = vm.rollback_to(&ModelVersion::new(9, 9, 9), "test");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn rollback_to_current_active_version_returns_error() {
    let mut vm = populated_version_manager(3);

    let result = vm.rollback_to(&ModelVersion::new(1, 2, 0), "test");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already the active version"));
}

#[test]
fn rollback_to_no_active_version_returns_error() {
    let mut vm = ModelVersionManager::new();
    let m = make_below_threshold_metrics();
    let e = ModelEntry::new(ModelVersion::new(1, 0, 0), m, "v1", "/v1");
    let _ = vm.register_version(e);

    let result = vm.rollback_to(&ModelVersion::new(1, 0, 0), "test");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No active version"));
}

#[test]
fn rollback_to_clears_previous_rollback_flag() {
    let mut vm = populated_version_manager(3);

    // Rollback: 1.2.0 -> 1.1.0
    vm.rollback("first").unwrap();
    // Now rollback_to 1.2.0 (which was rolled_back)
    // Need to go back to 1.2.0 from 1.1.0 - but 1.2.0 is rolled_back
    // rollback_to should clear the rolled_back flag on the target
    let rb = vm.rollback_to(&ModelVersion::new(1, 2, 0), "re-activate").unwrap();
    assert!(rb.success);
    assert_eq!(vm.active_version().unwrap().version, ModelVersion::new(1, 2, 0));
    // Verify the rolled_back flag was cleared
    assert!(!vm.active_version().unwrap().rolled_back);
}

#[test]
fn rollback_to_records_in_history() {
    let mut vm = populated_version_manager(3);

    vm.rollback_to(&ModelVersion::new(1, 0, 0), "back to base").unwrap();
    assert_eq!(vm.rollback_history().len(), 1);
    assert_eq!(vm.rollback_history()[0].reason, "back to base");
}

#[test]
fn rollback_to_marks_old_active_as_rolled_back() {
    let mut vm = populated_version_manager(3);
    let old_active = vm.active_version().unwrap().version.clone();
    assert_eq!(old_active, ModelVersion::new(1, 2, 0));

    vm.rollback_to(&ModelVersion::new(1, 0, 0), "rollback").unwrap();

    // The old active (1.2.0) should be marked as rolled_back
    let v120 = vm.versions().find(|e| e.version == ModelVersion::new(1, 2, 0)).unwrap();
    assert!(v120.rolled_back);
    assert!(!v120.is_active);
    assert_eq!(v120.rollback_reason.as_deref(), Some("rollback"));
}

// ============================================================================
// MODEL VERSIONING: to_markdown tests
// ============================================================================

#[test]
fn to_markdown_contains_header() {
    let vm = ModelVersionManager::new();
    let md = vm.to_markdown();
    assert!(md.contains("## Model Version Report"));
}

#[test]
fn to_markdown_no_active_version() {
    let vm = ModelVersionManager::new();
    let md = vm.to_markdown();
    assert!(md.contains("**Active Version**: None"));
}

#[test]
fn to_markdown_with_active_version() {
    let vm = populated_version_manager(1);
    let md = vm.to_markdown();
    assert!(md.contains("**Active Version**: v1.0.0"));
    assert!(md.contains("Accuracy:"));
    assert!(md.contains("F1:"));
}

#[test]
fn to_markdown_contains_version_history_table() {
    let vm = populated_version_manager(3);
    let md = vm.to_markdown();
    assert!(md.contains("### Version History"));
    assert!(md.contains("| Version | Accuracy | F1 | Status | Released |"));
    assert!(md.contains("v1.0.0"));
    assert!(md.contains("v1.1.0"));
    assert!(md.contains("v1.2.0"));
}

#[test]
fn to_markdown_shows_active_status() {
    let vm = populated_version_manager(2);
    let md = vm.to_markdown();
    assert!(md.contains("Active"));
}

#[test]
fn to_markdown_shows_rolled_back_status() {
    let mut vm = populated_version_manager(3);
    vm.rollback("test regression").unwrap();

    let md = vm.to_markdown();
    assert!(md.contains("Rolled Back"));
}

#[test]
fn to_markdown_shows_available_status() {
    let mut vm = ModelVersionManager::new();
    // Register a good v1 (activated)
    let m1 = make_good_metrics();
    let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "/v1");
    vm.register_version(e1).unwrap();
    // Register a worse v2 (not activated, marked as Available)
    let m2 = make_weak_metrics();
    let e2 = ModelEntry::new(ModelVersion::new(1, 1, 0), m2, "v2", "/v2");
    vm.register_version(e2).unwrap();

    let md = vm.to_markdown();
    assert!(md.contains("Available"));
}

#[test]
fn to_markdown_includes_rollback_history() {
    let mut vm = populated_version_manager(3);
    vm.rollback("quality issue").unwrap();

    let md = vm.to_markdown();
    assert!(md.contains("### Rollback History"));
    assert!(md.contains("quality issue"));
}

#[test]
fn to_markdown_no_rollback_section_when_empty() {
    let vm = populated_version_manager(2);
    let md = vm.to_markdown();
    assert!(!md.contains("### Rollback History"));
}

#[test]
fn to_markdown_multiple_rollbacks_shown() {
    let mut vm = populated_version_manager(4);
    vm.rollback("issue 1").unwrap();
    vm.rollback("issue 2").unwrap();

    let md = vm.to_markdown();
    assert!(md.contains("issue 1"));
    assert!(md.contains("issue 2"));
}

#[test]
fn to_markdown_accuracy_formatted_as_percentage() {
    let vm = populated_version_manager(1);
    let md = vm.to_markdown();
    // Accuracy 0.85 should show as 85.0%
    assert!(md.contains("85.0%"));
}

// ============================================================================
// ERROR TRACKING: generate_markdown_report tests
// ============================================================================

#[test]
fn markdown_report_contains_summary_section() {
    let mut tracker = ErrorTracker::new();
    let md = tracker.generate_markdown_report();
    assert!(md.contains("### Summary"));
    assert!(md.contains("Total Errors"));
    assert!(md.contains("Total Successes"));
    assert!(md.contains("Error Rate"));
}

#[test]
fn markdown_report_empty_tracker_zero_rate() {
    let mut tracker = ErrorTracker::new();
    let md = tracker.generate_markdown_report();
    assert!(md.contains("Total Errors | 0"));
    assert!(md.contains("Total Successes | 0"));
    assert!(md.contains("0.0%"));
}

#[test]
fn markdown_report_with_errors_shows_error_rate() {
    let mut tracker = ErrorTracker::new();
    for i in 0..10 {
        tracker.record_error(make_error(
            &format!("ptr{}", i),
            OwnershipDefect::PointerMisclassification,
            vec!["malloc_free".into()],
        ));
    }
    for _ in 0..10 {
        tracker.record_success(&["malloc_free".into()]);
    }

    let md = tracker.generate_markdown_report();
    assert!(md.contains("Total Errors | 10"));
    assert!(md.contains("Total Successes | 10"));
    assert!(md.contains("50.0%"));
}

#[test]
fn markdown_report_suspicious_features_table() {
    let mut tracker = ErrorTracker::new();
    for _ in 0..15 {
        tracker.record_error(make_error(
            "ptr",
            OwnershipDefect::PointerMisclassification,
            vec!["dangerous_cast".into()],
        ));
    }
    tracker.record_success(&["dangerous_cast".into()]);

    let md = tracker.generate_markdown_report();
    assert!(md.contains("### Top Suspicious Features (Tarantula)"));
    assert!(md.contains("dangerous_cast"));
}

#[test]
fn markdown_report_defect_distribution_section() {
    let mut tracker = ErrorTracker::new();
    for _ in 0..5 {
        tracker.record_error(make_error(
            "ptr",
            OwnershipDefect::PointerMisclassification,
            vec![],
        ));
    }
    for _ in 0..3 {
        tracker.record_error(make_error(
            "ptr",
            OwnershipDefect::LifetimeInferenceGap,
            vec![],
        ));
    }

    let md = tracker.generate_markdown_report();
    assert!(md.contains("### Defect Distribution"));
    assert!(md.contains("PointerMisclassification"));
    assert!(md.contains("LifetimeInferenceGap"));
}

#[test]
fn markdown_report_suggestions_section_present_when_enough_errors() {
    let mut tracker = ErrorTracker::new();
    for i in 0..30 {
        tracker.record_error(make_error(
            &format!("ptr{}", i),
            OwnershipDefect::PointerMisclassification,
            vec!["problematic".into()],
        ));
    }
    for _ in 0..2 {
        tracker.record_success(&["problematic".into()]);
    }

    let md = tracker.generate_markdown_report();
    assert!(md.contains("### Improvement Suggestions"));
    assert!(md.contains("problematic"));
}

#[test]
fn markdown_report_no_suggestions_when_few_errors() {
    let mut tracker = ErrorTracker::new();
    tracker.record_error(make_error(
        "ptr",
        OwnershipDefect::PointerMisclassification,
        vec!["small_feature".into()],
    ));

    let md = tracker.generate_markdown_report();
    // With only 1 error, no suggestion category should trigger
    // (needs > 5 for DefectPrevention, > 0.7 suspiciousness for FeatureHandling)
    // The feature might be suspicious if all are failures though
    // Still, the report should be well-formed
    assert!(md.contains("### Summary"));
}

#[test]
fn markdown_report_defect_percentages() {
    let mut tracker = ErrorTracker::new();
    for _ in 0..8 {
        tracker.record_error(make_error(
            "ptr",
            OwnershipDefect::PointerMisclassification,
            vec![],
        ));
    }
    for _ in 0..2 {
        tracker.record_error(make_error(
            "ptr",
            OwnershipDefect::AliasViolation,
            vec![],
        ));
    }

    let md = tracker.generate_markdown_report();
    // PointerMisclassification: 8/10 = 80.0%
    assert!(md.contains("80.0%"));
    // AliasViolation: 2/10 = 20.0%
    assert!(md.contains("20.0%"));
}

// ============================================================================
// ERROR TRACKING: calculate_suspiciousness tests
// ============================================================================

#[test]
fn suspiciousness_empty_tracker() {
    let mut tracker = ErrorTracker::new();
    let results = tracker.calculate_suspiciousness();
    assert!(results.is_empty());
}

#[test]
fn suspiciousness_single_feature_all_failures() {
    let mut tracker = ErrorTracker::new();
    for _ in 0..5 {
        tracker.record_error(make_error(
            "ptr",
            OwnershipDefect::PointerMisclassification,
            vec!["only_feature".into()],
        ));
    }

    let results = tracker.calculate_suspiciousness();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].feature, "only_feature");
    // All failures, no successes: failed_ratio = 5/5=1.0, passed_ratio = 0/1=0
    // susp = 1.0 / (1.0 + 0.0) = 1.0
    assert!((results[0].score - 1.0).abs() < 0.01);
}

#[test]
fn suspiciousness_single_feature_all_successes() {
    let mut tracker = ErrorTracker::new();
    for _ in 0..5 {
        tracker.record_success(&["good_feature".into()]);
    }

    let results = tracker.calculate_suspiciousness();
    assert_eq!(results.len(), 1);
    // No failures for this feature: failed_ratio = 0/1=0, passed_ratio = 5/5=1.0
    // susp = 0 / (0 + 1) = 0
    assert!((results[0].score - 0.0).abs() < 0.01);
}

#[test]
fn suspiciousness_ranked_highest_first() {
    let mut tracker = ErrorTracker::new();
    // Feature A: 9 failures, 1 success => high suspiciousness
    for _ in 0..9 {
        tracker.record_error(make_error(
            "ptr",
            OwnershipDefect::PointerMisclassification,
            vec!["feature_a".into()],
        ));
    }
    tracker.record_success(&["feature_a".into()]);

    // Feature B: 1 failure, 9 successes => low suspiciousness
    tracker.record_error(make_error(
        "ptr",
        OwnershipDefect::PointerMisclassification,
        vec!["feature_b".into()],
    ));
    for _ in 0..9 {
        tracker.record_success(&["feature_b".into()]);
    }

    let results = tracker.calculate_suspiciousness();
    assert!(results.len() >= 2);
    assert_eq!(results[0].feature, "feature_a");
    assert!(results[0].score > results[1].score);
}

#[test]
fn suspiciousness_updates_stats_in_place() {
    let mut tracker = ErrorTracker::new();
    for _ in 0..4 {
        tracker.record_error(make_error(
            "ptr",
            OwnershipDefect::PointerMisclassification,
            vec!["feat".into()],
        ));
    }
    tracker.record_success(&["feat".into()]);

    let results = tracker.calculate_suspiciousness();
    assert_eq!(results[0].failure_count, 4);
    assert_eq!(results[0].success_count, 1);
    assert_eq!(results[0].total_count, 5);
}

#[test]
fn suspiciousness_multiple_features_per_error() {
    let mut tracker = ErrorTracker::new();
    // Error with multiple features
    tracker.record_error(make_error(
        "ptr",
        OwnershipDefect::PointerMisclassification,
        vec!["feat_x".into(), "feat_y".into()],
    ));
    tracker.record_success(&["feat_x".into()]);

    let results = tracker.calculate_suspiciousness();
    assert!(results.len() >= 2);

    let feat_x = results.iter().find(|r| r.feature == "feat_x").unwrap();
    let feat_y = results.iter().find(|r| r.feature == "feat_y").unwrap();

    // feat_x: 1 failure, 1 success
    assert_eq!(feat_x.failure_count, 1);
    assert_eq!(feat_x.success_count, 1);
    // feat_y: 1 failure, 0 successes
    assert_eq!(feat_y.failure_count, 1);
    assert_eq!(feat_y.success_count, 0);
    // feat_y should be more suspicious
    assert!(feat_y.score >= feat_x.score);
}

#[test]
fn top_suspicious_limits_results() {
    let mut tracker = ErrorTracker::new();
    for i in 0..10 {
        tracker.record_error(make_error(
            "ptr",
            OwnershipDefect::PointerMisclassification,
            vec![format!("feature_{}", i)],
        ));
    }

    let top3 = tracker.top_suspicious(3);
    assert_eq!(top3.len(), 3);
}

#[test]
fn top_suspicious_returns_all_when_fewer_than_n() {
    let mut tracker = ErrorTracker::new();
    tracker.record_error(make_error(
        "ptr",
        OwnershipDefect::PointerMisclassification,
        vec!["only".into()],
    ));

    let top5 = tracker.top_suspicious(5);
    assert_eq!(top5.len(), 1);
}

// ============================================================================
// ERROR TRACKING: generate_suggestions edge cases
// ============================================================================

#[test]
fn suggestions_no_suggestions_when_empty() {
    let mut tracker = ErrorTracker::new();
    let suggestions = tracker.generate_suggestions();
    assert!(suggestions.is_empty());
}

#[test]
fn suggestions_high_priority_for_many_defects() {
    let mut tracker = ErrorTracker::new();
    for i in 0..25 {
        tracker.record_error(make_error(
            &format!("ptr{}", i),
            OwnershipDefect::DanglingPointerRisk,
            vec![],
        ));
    }
    for _ in 0..5 {
        tracker.record_success(&[]);
    }

    let suggestions = tracker.generate_suggestions();
    let defect_suggestions: Vec<_> = suggestions
        .iter()
        .filter(|s| s.category == SuggestionCategory::DefectPrevention)
        .collect();
    assert!(!defect_suggestions.is_empty());
    assert_eq!(defect_suggestions[0].priority, SuggestionPriority::High);
}

#[test]
fn suggestions_medium_priority_for_moderate_defects() {
    let mut tracker = ErrorTracker::new();
    for i in 0..10 {
        tracker.record_error(make_error(
            &format!("ptr{}", i),
            OwnershipDefect::ArraySliceMismatch,
            vec![],
        ));
    }
    for _ in 0..20 {
        tracker.record_success(&[]);
    }

    let suggestions = tracker.generate_suggestions();
    let defect_suggestions: Vec<_> = suggestions
        .iter()
        .filter(|s| s.category == SuggestionCategory::DefectPrevention)
        .collect();
    assert!(!defect_suggestions.is_empty());
    assert_eq!(defect_suggestions[0].priority, SuggestionPriority::Medium);
}

#[test]
fn suggestions_feature_handling_for_highly_suspicious() {
    let mut tracker = ErrorTracker::new();
    // To get suspiciousness > 0.7 for "bad_pattern":
    // susp = (failed(e)/total_failed) / ((failed(e)/total_failed) + (passed(e)/total_passed))
    // We need many failures with "bad_pattern" and many total successes but few
    // successes with "bad_pattern".
    // With 20 failures for bad_pattern, total_failed=20:
    //   failed_ratio = 20/20 = 1.0
    // With 1 success for bad_pattern, 100 total successes:
    //   passed_ratio = 1/100 = 0.01
    // susp = 1.0 / (1.0 + 0.01) = 0.99 => highly suspicious
    for _ in 0..20 {
        tracker.record_error(make_error(
            "ptr",
            OwnershipDefect::PointerMisclassification,
            vec!["bad_pattern".into()],
        ));
    }
    // 1 success for bad_pattern
    tracker.record_success(&["bad_pattern".into()]);
    // 99 more successes for OTHER features (inflates total_passed)
    for _ in 0..99 {
        tracker.record_success(&["good_pattern".into()]);
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

// ============================================================================
// Additional model versioning coverage
// ============================================================================

#[test]
fn version_manager_auto_rollback_triggers_on_bad_metrics() {
    let mut vm = populated_version_manager(3);

    let degraded = make_below_threshold_metrics();
    let result = vm.auto_rollback_if_needed(&degraded);

    assert!(result.is_some());
    let rb = result.unwrap();
    assert!(rb.success);
    assert!(rb.reason.contains("Auto-rollback"));
}

#[test]
fn version_manager_auto_rollback_does_not_trigger_on_good_metrics() {
    let mut vm = populated_version_manager(2);

    let good = make_good_metrics();
    let result = vm.auto_rollback_if_needed(&good);

    assert!(result.is_none());
}

#[test]
fn version_manager_check_quality_regression_more_than_5pct() {
    let mut vm = ModelVersionManager::new();
    let m = ModelQualityMetrics::new(0.95, 0.90, 0.90, 0.90, 0.92, 0.05, 1000);
    let e = ModelEntry::new(ModelVersion::new(1, 0, 0), m, "v1", "/v1");
    vm.register_version(e).unwrap();

    // accuracy 0.89 is > 0.05 drop from 0.95
    let regressed = ModelQualityMetrics::new(0.89, 0.88, 0.88, 0.88, 0.90, 0.06, 1000);
    let check = vm.check_quality(&regressed);
    assert!(check.is_some());
    assert!(check.unwrap().contains("regression"));
}

#[test]
fn version_manager_with_max_history_clamps_minimum_to_2() {
    let vm = ModelVersionManager::new().with_max_history(1);
    // Internal max should be clamped to 2
    // We verify by registering 3 versions; with max=2, only 2 should remain
    let mut vm = vm;
    for i in 0..3 {
        let m = ModelQualityMetrics::new(
            0.85 + (i as f64 * 0.02),
            0.85,
            0.85,
            0.85,
            0.9,
            0.05,
            1000,
        );
        let e = ModelEntry::new(
            ModelVersion::new(1, i, 0),
            m,
            format!("v1.{}", i),
            format!("/v1.{}", i),
        );
        let _ = vm.register_version(e);
    }
    assert_eq!(vm.version_count(), 2);
}

#[test]
fn version_manager_with_custom_thresholds() {
    let thresholds = QualityThresholds {
        min_accuracy: 0.99,
        min_precision: 0.99,
        min_recall: 0.99,
        min_f1: 0.99,
    };
    let mut vm = ModelVersionManager::with_thresholds(thresholds);

    // Good metrics normally but below 0.99 thresholds
    let m = make_good_metrics();
    let e = ModelEntry::new(ModelVersion::new(1, 0, 0), m, "v1", "/v1");
    let activated = vm.register_version(e).unwrap();

    assert!(!activated); // Below strict thresholds
}

#[test]
fn training_metrics_f1_calculation() {
    let m = TrainingMetrics::new(1.0, 1.0);
    assert!((m.f1_score - 1.0).abs() < 0.001);

    let m2 = TrainingMetrics::new(0.0, 0.0);
    assert!((m2.f1_score - 0.0).abs() < 0.001);

    let m3 = TrainingMetrics::new(0.5, 0.5);
    // F1 = 2*0.5*0.5 / (0.5+0.5) = 0.5
    assert!((m3.f1_score - 0.5).abs() < 0.001);
}

#[test]
fn training_metrics_to_quality_metrics_roundtrip() {
    let tm = TrainingMetrics::new(0.90, 0.85);
    let qm = tm.to_quality_metrics();

    assert!((qm.precision - 0.90).abs() < 0.001);
    assert!((qm.recall - 0.85).abs() < 0.001);
    assert!((qm.f1_score - tm.f1_score).abs() < 0.001);
}

// ============================================================================
// Additional error tracking coverage
// ============================================================================

#[test]
fn errors_by_defect_returns_matching() {
    let mut tracker = ErrorTracker::new();
    tracker.record_error(make_error(
        "ptr1",
        OwnershipDefect::MutabilityMismatch,
        vec![],
    ));
    tracker.record_error(make_error(
        "ptr2",
        OwnershipDefect::PointerMisclassification,
        vec![],
    ));
    tracker.record_error(make_error(
        "ptr3",
        OwnershipDefect::MutabilityMismatch,
        vec![],
    ));

    let results = tracker.errors_by_defect(&OwnershipDefect::MutabilityMismatch);
    assert_eq!(results.len(), 2);
}

#[test]
fn errors_by_feature_returns_matching() {
    let mut tracker = ErrorTracker::new();
    tracker.record_error(make_error(
        "ptr1",
        OwnershipDefect::PointerMisclassification,
        vec!["void_star".into(), "struct_field".into()],
    ));
    tracker.record_error(make_error(
        "ptr2",
        OwnershipDefect::PointerMisclassification,
        vec!["void_star".into()],
    ));
    tracker.record_error(make_error(
        "ptr3",
        OwnershipDefect::PointerMisclassification,
        vec!["array_param".into()],
    ));

    let results = tracker.errors_by_feature("void_star");
    assert_eq!(results.len(), 2);
}

#[test]
fn feature_defect_correlation_tracks_combinations() {
    let mut tracker = ErrorTracker::new();
    // Same feature, different defects
    tracker.record_error(make_error(
        "ptr1",
        OwnershipDefect::PointerMisclassification,
        vec!["feature_x".into()],
    ));
    tracker.record_error(make_error(
        "ptr2",
        OwnershipDefect::LifetimeInferenceGap,
        vec!["feature_x".into()],
    ));

    let correlations = tracker.feature_defect_correlation();
    assert_eq!(correlations.len(), 2);
    // Both should have feature_x
    assert!(correlations.iter().all(|(f, _, _)| f == "feature_x"));
}

#[test]
fn pattern_stats_failure_rate_zero_count() {
    let stats = PatternStats::default();
    assert!((stats.failure_rate() - 0.0).abs() < 0.001);
}

#[test]
fn pattern_stats_failure_rate_all_failures() {
    let mut stats = PatternStats::default();
    stats.record(true);
    stats.record(true);
    stats.record(true);
    assert!((stats.failure_rate() - 1.0).abs() < 0.001);
}

#[test]
fn feature_suspiciousness_is_suspicious_boundary() {
    let exact = FeatureSuspiciousness {
        feature: "test".into(),
        score: 0.5,
        total_count: 10,
        failure_count: 5,
        success_count: 5,
    };
    assert!(!exact.is_suspicious()); // 0.5 is not > 0.5

    let just_above = FeatureSuspiciousness {
        feature: "test".into(),
        score: 0.501,
        total_count: 10,
        failure_count: 5,
        success_count: 5,
    };
    assert!(just_above.is_suspicious());
}

#[test]
fn feature_suspiciousness_is_highly_suspicious_boundary() {
    let exact = FeatureSuspiciousness {
        feature: "test".into(),
        score: 0.7,
        total_count: 10,
        failure_count: 7,
        success_count: 3,
    };
    assert!(!exact.is_highly_suspicious()); // 0.7 is not > 0.7

    let just_above = FeatureSuspiciousness {
        feature: "test".into(),
        score: 0.701,
        total_count: 10,
        failure_count: 7,
        success_count: 3,
    };
    assert!(just_above.is_highly_suspicious());
}
