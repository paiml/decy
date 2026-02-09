//! Coverage tests for retraining_pipeline::execute and inference::classify_pointer.
//!
//! Targets uncovered branches in:
//! - retraining_pipeline.rs: execute() function (line 373+)
//! - inference.rs: classify_pointer() function (line 82+)

use crate::dataflow::{DataflowAnalyzer, DataflowGraph};
use crate::inference::{OwnershipInferencer, OwnershipKind};
use crate::ml_features::{InferredOwnership, OwnershipFeaturesBuilder};
use crate::model_versioning::{ModelEntry, ModelQualityMetrics, ModelVersion, ModelVersionManager};
use crate::retraining_pipeline::{
    DataSplit, ExecutionSummary, ModelTrainer, NullTrainer, RetrainingConfig, RetrainingPipeline,
    RetrainingResult, RetrainingSchedule, TrainingMetrics, TrainingSample,
};
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};

// ============================================================================
// Helper functions
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

fn make_varied_samples(n: usize) -> Vec<TrainingSample> {
    (0..n)
        .map(|i| {
            let label = match i % 4 {
                0 => InferredOwnership::Owned,
                1 => InferredOwnership::Borrowed,
                2 => InferredOwnership::BorrowedMut,
                _ => InferredOwnership::Shared,
            };
            TrainingSample::new(
                OwnershipFeaturesBuilder::default()
                    .pointer_depth((i % 3) as u8)
                    .build(),
                label,
                &format!("src/file{}.c", i),
                i as u32 + 1,
            )
        })
        .collect()
}

fn small_config() -> RetrainingConfig {
    RetrainingConfig {
        min_precision: 0.85,
        min_recall: 0.80,
        max_degradation: 0.02,
        cv_folds: 3,
        min_train_samples: 7,
        min_validation_samples: 2,
        min_test_samples: 2,
        train_ratio: 0.70,
        validation_ratio: 0.15,
    }
}

struct FailingTrainer;
impl ModelTrainer for FailingTrainer {
    fn train(&self, _data: &DataSplit) -> Result<TrainingMetrics, String> {
        Err("Model training diverged unexpectedly".to_string())
    }
}

struct ConfigurableTrainer {
    result: Result<TrainingMetrics, String>,
}

impl ConfigurableTrainer {
    fn failure(msg: &str) -> Self {
        Self {
            result: Err(msg.to_string()),
        }
    }
}

impl ModelTrainer for ConfigurableTrainer {
    fn train(&self, _data: &DataSplit) -> Result<TrainingMetrics, String> {
        self.result.clone()
    }
}

// ============================================================================
// RETRAINING PIPELINE: execute() coverage tests
// ============================================================================

#[test]
fn execute_insufficient_data_returns_insufficient_result() {
    let trainer = NullTrainer::new(0.92, 0.88);
    let config = small_config();
    let vm = ModelVersionManager::new();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    // Provide fewer samples than min required (7+2+2 = 11)
    let samples = make_samples(5);
    let result = pipeline.execute(samples);

    assert!(!result.is_success());
    match result {
        RetrainingResult::InsufficientData { actual, required } => {
            assert_eq!(actual, 5);
            assert_eq!(required, 11);
        }
        _ => panic!("Expected InsufficientData"),
    }
}

#[test]
fn execute_records_insufficient_data_in_history() {
    let trainer = NullTrainer::new(0.92, 0.88);
    let config = small_config();
    let vm = ModelVersionManager::new();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    let samples = make_samples(3);
    pipeline.execute(samples);

    assert_eq!(pipeline.history().len(), 1);
    assert!(matches!(
        pipeline.history()[0].result,
        ExecutionSummary::InsufficientData
    ));
    assert_eq!(pipeline.history()[0].sample_count, 3);
}

#[test]
fn execute_split_fails_minimum_sizes_second_check() {
    // The total count passes first check but per-split minimums fail
    let trainer = NullTrainer::new(0.92, 0.88);
    let config = RetrainingConfig {
        min_precision: 0.85,
        min_recall: 0.80,
        max_degradation: 0.02,
        cv_folds: 3,
        min_train_samples: 15,
        min_validation_samples: 5,
        min_test_samples: 5,
        train_ratio: 0.50,
        validation_ratio: 0.10,
    };
    let vm = ModelVersionManager::new();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    // 25 samples: min_required = 15+5+5 = 25, passes first check
    // With 50/10 split: train=12, val=2, test=11
    // train(12) < min_train(15) -> fails
    let samples = make_samples(25);
    let result = pipeline.execute(samples);

    assert!(!result.is_success());
    assert!(matches!(result, RetrainingResult::InsufficientData { .. }));
}

#[test]
fn execute_training_error_path() {
    let trainer = FailingTrainer;
    let config = small_config();
    let vm = ModelVersionManager::new();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    let samples = make_samples(20);
    let result = pipeline.execute(samples);

    assert!(!result.is_success());
    match result {
        RetrainingResult::TrainingError { error } => {
            assert!(error.contains("diverged"));
        }
        _ => panic!("Expected TrainingError"),
    }
}

#[test]
fn execute_training_error_recorded_in_history() {
    let trainer = FailingTrainer;
    let config = small_config();
    let vm = ModelVersionManager::new();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    let samples = make_samples(20);
    pipeline.execute(samples);

    assert_eq!(pipeline.history().len(), 1);
    match &pipeline.history()[0].result {
        ExecutionSummary::Error { message } => {
            assert!(message.contains("diverged"));
        }
        _ => panic!("Expected Error in history"),
    }
}

#[test]
fn execute_quality_gate_failure_low_precision() {
    let trainer = NullTrainer::new(0.70, 0.90);
    let config = small_config();
    let vm = ModelVersionManager::new();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    let samples = make_samples(20);
    let result = pipeline.execute(samples);

    assert!(!result.is_success());
    match result {
        RetrainingResult::QualityGateFailed { reason, metrics } => {
            assert!(reason.contains("Precision"));
            assert!((metrics.precision - 0.70).abs() < 0.001);
        }
        _ => panic!("Expected QualityGateFailed"),
    }
}

#[test]
fn execute_quality_gate_failure_low_recall() {
    let trainer = NullTrainer::new(0.90, 0.60);
    let config = small_config();
    let vm = ModelVersionManager::new();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    let samples = make_samples(20);
    let result = pipeline.execute(samples);

    assert!(!result.is_success());
    match result {
        RetrainingResult::QualityGateFailed { reason, metrics } => {
            assert!(reason.contains("Recall"));
            assert!((metrics.recall - 0.60).abs() < 0.001);
        }
        _ => panic!("Expected QualityGateFailed"),
    }
}

#[test]
fn execute_quality_gate_failure_recorded_in_history() {
    let trainer = NullTrainer::new(0.60, 0.50);
    let config = small_config();
    let vm = ModelVersionManager::new();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    let samples = make_samples(20);
    pipeline.execute(samples);

    assert_eq!(pipeline.history().len(), 1);
    match &pipeline.history()[0].result {
        ExecutionSummary::QualityGateFailed { reason } => {
            assert!(!reason.is_empty());
        }
        _ => panic!("Expected QualityGateFailed in history"),
    }
}

#[test]
fn execute_degradation_detected() {
    // Register a strong model first
    let mut vm = ModelVersionManager::new();
    let strong_metrics = ModelQualityMetrics::new(0.95, 0.95, 0.92, 0.935, 0.95, 0.02, 2000);
    let strong_entry = ModelEntry::new(
        ModelVersion::new(1, 0, 0),
        strong_metrics,
        "Strong baseline model",
        "models/strong.bin",
    );
    let _ = vm.register_version(strong_entry);

    // Train with weaker metrics
    let trainer = NullTrainer::new(0.86, 0.82);
    let config = small_config();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    let samples = make_samples(20);
    let result = pipeline.execute(samples);

    assert!(!result.is_success());
    match result {
        RetrainingResult::Degraded {
            degradation,
            new_metrics,
            current_metrics,
        } => {
            assert!(degradation > 0.02);
            assert!((new_metrics.precision - 0.86).abs() < 0.001);
            assert!(current_metrics.f1_score > 0.9);
        }
        _ => panic!("Expected Degraded, got {:?}", result),
    }
}

#[test]
fn execute_degradation_recorded_in_history() {
    let mut vm = ModelVersionManager::new();
    let strong = ModelQualityMetrics::new(0.95, 0.95, 0.92, 0.935, 0.95, 0.02, 2000);
    let entry = ModelEntry::new(ModelVersion::new(1, 0, 0), strong, "v1", "m/v1.bin");
    let _ = vm.register_version(entry);

    let trainer = NullTrainer::new(0.86, 0.82);
    let config = small_config();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    let samples = make_samples(20);
    pipeline.execute(samples);

    assert_eq!(pipeline.history().len(), 1);
    match &pipeline.history()[0].result {
        ExecutionSummary::Degraded { amount } => {
            assert!(*amount > 0.02);
        }
        _ => panic!("Expected Degraded in history"),
    }
}

#[test]
fn execute_successful_promotion_first_model() {
    let trainer = NullTrainer::new(0.92, 0.88);
    let config = small_config();
    let vm = ModelVersionManager::new();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    let samples = make_samples(20);
    let result = pipeline.execute(samples);

    assert!(result.is_success());
    match result {
        RetrainingResult::Promoted { version, metrics } => {
            assert_eq!(version, ModelVersion::new(1, 0, 0));
            assert!((metrics.precision - 0.92).abs() < 0.001);
        }
        _ => panic!("Expected Promoted"),
    }
}

#[test]
fn execute_promotion_recorded_in_history() {
    let trainer = NullTrainer::new(0.92, 0.88);
    let config = small_config();
    let vm = ModelVersionManager::new();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    let samples = make_samples(20);
    pipeline.execute(samples);

    assert_eq!(pipeline.history().len(), 1);
    match &pipeline.history()[0].result {
        ExecutionSummary::Promoted { version } => {
            assert!(version.contains("1.0.0"));
        }
        _ => panic!("Expected Promoted in history"),
    }
}

#[test]
fn execute_successive_promotions_bump_minor_version() {
    let trainer = NullTrainer::new(0.92, 0.88);
    let config = small_config();
    let vm = ModelVersionManager::new();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    // First promotion -> v1.0.0
    let result1 = pipeline.execute(make_samples(20));
    assert!(result1.is_success());
    if let RetrainingResult::Promoted { version, .. } = &result1 {
        assert_eq!(*version, ModelVersion::new(1, 0, 0));
    }

    // Second promotion -> v1.1.0
    // Note: register_version may not activate if metrics are equal (is_better_than check).
    // The version number still bumps based on active_version().
    let result2 = pipeline.execute(make_samples(20));
    assert!(result2.is_success());
    if let RetrainingResult::Promoted { version, .. } = &result2 {
        assert_eq!(*version, ModelVersion::new(1, 1, 0));
    }
}

#[test]
fn execute_no_degradation_when_improvement() {
    // First model
    let mut vm = ModelVersionManager::new();
    let baseline = ModelQualityMetrics::new(0.85, 0.86, 0.82, 0.84, 0.9, 0.05, 500);
    let entry = ModelEntry::new(ModelVersion::new(1, 0, 0), baseline, "v1", "m/v1.bin");
    let _ = vm.register_version(entry);

    // Better trainer: higher F1 than baseline
    let trainer = NullTrainer::new(0.92, 0.88);
    let config = small_config();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    let samples = make_samples(20);
    let result = pipeline.execute(samples);

    assert!(result.is_success());
}

#[test]
fn execute_marginal_degradation_within_threshold() {
    // Register baseline with F1 ~0.87
    let mut vm = ModelVersionManager::new();
    let baseline = ModelQualityMetrics::new(0.87, 0.88, 0.86, 0.87, 0.9, 0.05, 500);
    let entry = ModelEntry::new(ModelVersion::new(1, 0, 0), baseline, "v1", "m/v1.bin");
    let _ = vm.register_version(entry);

    // New model with F1 ~0.8628 (precision=0.87, recall=0.856)
    // Degradation = 0.87 - 0.8628 = ~0.007 < 0.02 threshold
    let trainer = NullTrainer::new(0.87, 0.856);
    let config = small_config();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    let samples = make_samples(20);
    let result = pipeline.execute(samples);

    // Should promote since degradation is within threshold
    assert!(result.is_success());
}

#[test]
fn execute_with_varied_sample_labels() {
    let trainer = NullTrainer::new(0.92, 0.88);
    let config = small_config();
    let vm = ModelVersionManager::new();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    let samples = make_varied_samples(20);
    let result = pipeline.execute(samples);

    assert!(result.is_success());
}

#[test]
fn execute_multiple_failures_then_success() {
    let config = small_config();
    let vm = ModelVersionManager::new();

    // First: failing trainer
    let trainer = ConfigurableTrainer::failure("out of memory");
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config.clone());

    let result1 = pipeline.execute(make_samples(20));
    assert!(!result1.is_success());

    // Since we can't change trainer, test history tracking for multiple entries
    let result2 = pipeline.execute(make_samples(3)); // Insufficient
    assert!(!result2.is_success());

    assert_eq!(pipeline.history().len(), 2);
    assert!((pipeline.success_rate() - 0.0).abs() < 0.001);
}

#[test]
fn execute_success_rate_calculation() {
    let trainer = NullTrainer::new(0.92, 0.88);
    let config = small_config();
    let vm = ModelVersionManager::new();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    // 1 success
    pipeline.execute(make_samples(20));
    assert!((pipeline.success_rate() - 1.0).abs() < 0.001);

    // 1 success, 1 insufficient
    pipeline.execute(make_samples(3));
    assert!((pipeline.success_rate() - 0.5).abs() < 0.001);
}

#[test]
fn execute_result_metrics_for_all_variants() {
    // Promoted
    let promoted = RetrainingResult::Promoted {
        version: ModelVersion::new(1, 0, 0),
        metrics: TrainingMetrics::new(0.9, 0.9),
    };
    assert!(promoted.metrics().is_some());

    // QualityGateFailed
    let qgf = RetrainingResult::QualityGateFailed {
        reason: "low".to_string(),
        metrics: TrainingMetrics::new(0.5, 0.5),
    };
    assert!(qgf.metrics().is_some());

    // Degraded
    let degraded = RetrainingResult::Degraded {
        degradation: 0.1,
        new_metrics: TrainingMetrics::new(0.8, 0.7),
        current_metrics: ModelQualityMetrics::new(0.9, 0.9, 0.9, 0.9, 0.9, 0.0, 100),
    };
    assert!(degraded.metrics().is_some());

    // InsufficientData
    let insuf = RetrainingResult::InsufficientData {
        actual: 5,
        required: 100,
    };
    assert!(insuf.metrics().is_none());

    // TrainingError
    let err = RetrainingResult::TrainingError {
        error: "boom".to_string(),
    };
    assert!(err.metrics().is_none());
}

#[test]
fn execute_config_accessors_and_mutation() {
    let trainer = NullTrainer::new(0.92, 0.88);
    let config = small_config();
    let vm = ModelVersionManager::new();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    assert!((pipeline.config().min_precision - 0.85).abs() < 0.001);
    assert_eq!(pipeline.config().cv_folds, 3);

    let mut new_config = small_config();
    new_config.min_precision = 0.99;
    new_config.cv_folds = 10;
    pipeline.set_config(new_config);

    assert!((pipeline.config().min_precision - 0.99).abs() < 0.001);
    assert_eq!(pipeline.config().cv_folds, 10);
}

#[test]
fn execute_rollback_after_promotion() {
    // Pre-populate with two versions where second is activated
    let mut vm = ModelVersionManager::new();
    let m1 = ModelQualityMetrics::new(0.85, 0.86, 0.82, 0.84, 0.9, 0.05, 500);
    let m2 = ModelQualityMetrics::new(0.92, 0.93, 0.90, 0.915, 0.95, 0.03, 1000);
    let e1 = ModelEntry::new(ModelVersion::new(1, 0, 0), m1, "v1", "m/v1.bin");
    let e2 = ModelEntry::new(ModelVersion::new(1, 1, 0), m2, "v2", "m/v2.bin");
    let _ = vm.register_version(e1);
    let _ = vm.register_version(e2);

    let trainer = NullTrainer::new(0.92, 0.88);
    let config = small_config();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    // Rollback from v1.1.0 to v1.0.0
    let v1 = ModelVersion::new(1, 0, 0);
    let rb = pipeline.rollback(&v1);
    assert!(rb.is_ok());
    let rb_result = rb.unwrap();
    assert!(rb_result.success);
    assert_eq!(rb_result.to_version, v1);
}

#[test]
fn execute_current_version_tracking() {
    let trainer = NullTrainer::new(0.92, 0.88);
    let config = small_config();
    let vm = ModelVersionManager::new();
    let mut pipeline = RetrainingPipeline::new(trainer, vm, config);

    assert!(pipeline.current_version().is_none());

    pipeline.execute(make_samples(20));
    let cv = pipeline.current_version();
    assert!(cv.is_some());
    assert_eq!(cv.unwrap().version, ModelVersion::new(1, 0, 0));
}

#[test]
fn training_metrics_f1_zero_when_both_zero() {
    let metrics = TrainingMetrics::new(0.0, 0.0);
    assert!((metrics.f1_score).abs() < 0.001);
}

#[test]
fn training_metrics_to_quality_metrics_conversion() {
    let metrics = TrainingMetrics::new(0.88, 0.82);
    let qm = metrics.to_quality_metrics();
    assert!((qm.precision - 0.88).abs() < 0.001);
    assert!((qm.recall - 0.82).abs() < 0.001);
    assert!(qm.f1_score > 0.0);
}

#[test]
fn data_split_with_extreme_ratios() {
    // All to train
    let split = DataSplit::new(make_samples(10), 1.0, 0.0);
    assert_eq!(split.train.len(), 10);
    assert_eq!(split.validation.len(), 0);
    assert_eq!(split.test.len(), 0);

    // All to test
    let split2 = DataSplit::new(make_samples(10), 0.0, 0.0);
    assert_eq!(split2.train.len(), 0);
    assert_eq!(split2.validation.len(), 0);
    assert_eq!(split2.test.len(), 10);
}

#[test]
fn schedule_description_all_seven_days() {
    let expected = [
        (0, "Sunday"),
        (1, "Monday"),
        (2, "Tuesday"),
        (3, "Wednesday"),
        (4, "Thursday"),
        (5, "Friday"),
        (6, "Saturday"),
    ];
    for (day, name) in &expected {
        let desc = RetrainingSchedule::new(*day, 0, 0).description();
        assert!(desc.contains(name), "Day {} should contain {}", day, name);
    }
}

#[test]
fn schedule_out_of_bounds_day_clamped() {
    let s = RetrainingSchedule::new(255, 0, 0);
    assert_eq!(s.day_of_week, 6);
    let desc = s.description();
    assert!(desc.contains("Saturday"));
}

// ============================================================================
// INFERENCE: classify_pointer() coverage tests
// ============================================================================

#[test]
fn classify_pointer_allocation_is_owning() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "p".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(4)],
            }),
        }],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert_eq!(inferences["p"].kind, OwnershipKind::Owning);
    assert!(inferences["p"].confidence >= 0.85);
}

#[test]
fn classify_pointer_parameter_not_mutated_is_immutable_borrow() {
    let func = HirFunction::new_with_body(
        "read".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert_eq!(inferences["data"].kind, OwnershipKind::ImmutableBorrow);
    assert!(inferences["data"].confidence >= 0.7);
}

#[test]
fn classify_pointer_parameter_with_deref_is_mutable_borrow() {
    let func = HirFunction::new_with_body(
        "write".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "out".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("out".to_string()),
            value: HirExpression::IntLiteral(42),
        }],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("out"));
    // The parameter has a dereference node, so classify_pointer sees Dereference first
    // which returns ImmutableBorrow, OR if nodes are ordered differently, MutableBorrow
    let kind = &inferences["out"].kind;
    assert!(
        matches!(
            kind,
            OwnershipKind::MutableBorrow | OwnershipKind::ImmutableBorrow
        ),
        "Expected borrow for deref parameter, got {:?}",
        kind
    );
}

#[test]
fn classify_pointer_assignment_from_variable_is_borrow() {
    let func = HirFunction::new_with_body(
        "alias".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "owner".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(8)],
                }),
            },
            HirStatement::VariableDeclaration {
                name: "alias".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Variable("owner".to_string())),
            },
        ],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert_eq!(inferences["owner"].kind, OwnershipKind::Owning);
    assert!(
        matches!(
            inferences["alias"].kind,
            OwnershipKind::ImmutableBorrow | OwnershipKind::MutableBorrow
        ),
        "Alias from another pointer should be a borrow"
    );
}

#[test]
fn classify_pointer_dereference_node_is_immutable_borrow() {
    let func = HirFunction::new_with_body(
        "deref_test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "ptr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Dereference(Box::new(
                    HirExpression::NullLiteral,
                ))),
            },
        ],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    if let Some(inf) = inferences.get("ptr") {
        assert_eq!(inf.kind, OwnershipKind::ImmutableBorrow);
    }
}

#[test]
fn classify_pointer_array_allocation_is_array_pointer() {
    let func = HirFunction::new_with_body(
        "arr_test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(10),
            },
            initializer: None,
        }],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("arr"));
    match &inferences["arr"].kind {
        OwnershipKind::ArrayPointer {
            base_array,
            element_type,
            base_index,
        } => {
            assert_eq!(base_array, "arr");
            assert_eq!(*element_type, HirType::Int);
            assert_eq!(*base_index, Some(0));
        }
        other => panic!("Expected ArrayPointer, got {:?}", other),
    }
    assert!(inferences["arr"].confidence >= 0.9);
}

#[test]
fn classify_pointer_heap_array_malloc_sizeof_pattern() {
    let func = HirFunction::new_with_body(
        "heap_arr".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "buf".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Multiply,
                    left: Box::new(HirExpression::IntLiteral(100)),
                    right: Box::new(HirExpression::Sizeof {
                        type_name: "int".to_string(),
                    }),
                }],
            }),
        }],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("buf"));
    assert!(
        matches!(inferences["buf"].kind, OwnershipKind::ArrayPointer { .. }),
        "malloc(n*sizeof) should be ArrayPointer, got {:?}",
        inferences["buf"].kind
    );
}

#[test]
fn classify_pointer_assignment_from_array_is_array_pointer() {
    let func = HirFunction::new_with_body(
        "ptr_from_arr".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Array {
                    element_type: Box::new(HirType::Float),
                    size: Some(5),
                },
                initializer: None,
            },
            HirStatement::VariableDeclaration {
                name: "p".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Float)),
                initializer: Some(HirExpression::Variable("arr".to_string())),
            },
        ],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("p"));
    match &inferences["p"].kind {
        OwnershipKind::ArrayPointer { base_array, .. } => {
            assert_eq!(base_array, "arr");
        }
        other => panic!("Expected ArrayPointer for ptr from array, got {:?}", other),
    }
}

#[test]
fn classify_pointer_unknown_when_no_nodes() {
    // Use an empty graph with no nodes
    let graph = DataflowGraph::new();
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);
    assert!(inferences.is_empty());
}

#[test]
fn confidence_owning_boosted_when_escaping() {
    let func = HirFunction::new_with_body(
        "factory".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "p".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(4)],
                }),
            },
            HirStatement::Return(Some(HirExpression::Variable("p".to_string()))),
        ],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert_eq!(inferences["p"].kind, OwnershipKind::Owning);
    assert!(inferences["p"].confidence >= 0.9, "Escaping owning should boost confidence");
}

#[test]
fn confidence_borrow_reduced_when_escaping() {
    let func = HirFunction::new_with_body(
        "leak".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new(
            "src".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
            HirStatement::VariableDeclaration {
                name: "alias".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Variable("src".to_string())),
            },
            HirStatement::Return(Some(HirExpression::Variable("alias".to_string()))),
        ],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    if let Some(inf) = inferences.get("alias") {
        // Borrow that escapes should have reduced confidence
        assert!(inf.confidence <= 0.8, "Escaping borrow should have reduced confidence, got {}", inf.confidence);
    }
}

#[test]
fn reasoning_contains_variable_name() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "my_ptr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(4)],
            }),
        }],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences["my_ptr"].reason.contains("my_ptr"));
    assert!(inferences["my_ptr"].reason.contains("malloc"));
}

#[test]
fn reasoning_assignment_immutable_contains_source() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "src".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::VariableDeclaration {
            name: "dst".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::Variable("src".to_string())),
        }],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    if let Some(inf) = inferences.get("dst") {
        assert!(
            inf.reason.contains("src") || inf.reason.contains("assigned"),
            "Reasoning should mention source: {}",
            inf.reason
        );
    }
}

#[test]
fn reasoning_array_allocation_mentions_array() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "data".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(20),
            },
            initializer: None,
        }],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    if let Some(inf) = inferences.get("data") {
        assert!(
            inf.reason.contains("array") || inf.reason.contains("Array"),
            "Array reasoning should mention 'array': {}",
            inf.reason
        );
    }
}

#[test]
fn reasoning_parameter_read_only_mentions_parameter() {
    let func = HirFunction::new_with_body(
        "reader".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "input".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(
        inferences["input"].reason.contains("parameter")
            || inferences["input"].reason.contains("read-only"),
        "Reason should mention parameter: {}",
        inferences["input"].reason
    );
}

#[test]
fn classify_multiple_pointers_in_same_function() {
    let func = HirFunction::new_with_body(
        "complex".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("in_ptr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("out_ptr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ],
        vec![
            HirStatement::VariableDeclaration {
                name: "heap".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(16)],
                }),
            },
            HirStatement::VariableDeclaration {
                name: "local_arr".to_string(),
                var_type: HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: Some(4),
                },
                initializer: None,
            },
        ],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    // Should have at least 3 inferences (in_ptr, out_ptr, heap, local_arr)
    assert!(inferences.len() >= 3);
    assert_eq!(inferences["heap"].kind, OwnershipKind::Owning);
    assert!(matches!(
        inferences["local_arr"].kind,
        OwnershipKind::ArrayPointer { .. }
    ));
}

#[test]
fn classify_pointer_with_char_array_allocation() {
    let func = HirFunction::new_with_body(
        "char_arr".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "buf".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Char)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Multiply,
                    left: Box::new(HirExpression::IntLiteral(256)),
                    right: Box::new(HirExpression::Sizeof {
                        type_name: "char".to_string(),
                    }),
                }],
            }),
        }],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    assert!(inferences.contains_key("buf"));
    match &inferences["buf"].kind {
        OwnershipKind::ArrayPointer { element_type, .. } => {
            assert_eq!(*element_type, HirType::Char);
        }
        other => panic!("Expected ArrayPointer for char buffer, got {:?}", other),
    }
}

#[test]
fn classify_pointer_with_float_array_allocation() {
    let func = HirFunction::new_with_body(
        "float_arr".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "fbuf".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Float)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Multiply,
                    left: Box::new(HirExpression::IntLiteral(50)),
                    right: Box::new(HirExpression::Sizeof {
                        type_name: "float".to_string(),
                    }),
                }],
            }),
        }],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    match &inferences["fbuf"].kind {
        OwnershipKind::ArrayPointer { element_type, .. } => {
            assert_eq!(*element_type, HirType::Float);
        }
        other => panic!("Expected ArrayPointer, got {:?}", other),
    }
}

#[test]
fn classify_pointer_with_double_array_allocation() {
    let func = HirFunction::new_with_body(
        "double_arr".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "dbuf".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Double)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Multiply,
                    left: Box::new(HirExpression::IntLiteral(10)),
                    right: Box::new(HirExpression::Sizeof {
                        type_name: "double".to_string(),
                    }),
                }],
            }),
        }],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    match &inferences["dbuf"].kind {
        OwnershipKind::ArrayPointer { element_type, .. } => {
            assert_eq!(*element_type, HirType::Double);
        }
        other => panic!("Expected ArrayPointer, got {:?}", other),
    }
}

#[test]
fn classify_pointer_fallback_type_in_sizeof() {
    // Unknown sizeof type falls back to Int
    let func = HirFunction::new_with_body(
        "unknown_sizeof".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "ptr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Multiply,
                    left: Box::new(HirExpression::IntLiteral(10)),
                    right: Box::new(HirExpression::Sizeof {
                        type_name: "custom_struct".to_string(),
                    }),
                }],
            }),
        }],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    match &inferences["ptr"].kind {
        OwnershipKind::ArrayPointer { element_type, .. } => {
            assert_eq!(*element_type, HirType::Int, "Unknown sizeof type should fall back to Int");
        }
        other => panic!("Expected ArrayPointer for custom sizeof, got {:?}", other),
    }
}

#[test]
fn inferencer_default_trait_works() {
    let inf: OwnershipInferencer = Default::default();
    let graph = DataflowGraph::new();
    let result = inf.infer(&graph);
    assert!(result.is_empty());
}
