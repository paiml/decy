//! Coverage tests for classifier.rs - targeting uncovered branches.

use crate::classifier::*;
use crate::ml_features::{AllocationKind, InferredOwnership, OwnershipFeaturesBuilder};
use crate::retraining_pipeline::TrainingSample;
use crate::training_data::TrainingDataset;

// ============================================================================
// RuleBasedClassifier::with_weights coverage
// ============================================================================

#[test]
fn rule_based_with_custom_weights() {
    let weights = RuleWeights {
        malloc_free: 0.99,
        array_alloc: 0.95,
        const_qual: 0.90,
        write_ops: 0.85,
        size_param: 0.80,
    };
    let classifier = RuleBasedClassifier::with_weights(weights);

    // Verify custom weights affect confidence
    let features = OwnershipFeaturesBuilder::default()
        .allocation_site(AllocationKind::Malloc)
        .deallocation_count(1)
        .pointer_depth(1)
        .build();

    let pred = classifier.classify(&features);
    assert!(matches!(pred.prediction, InferredOwnership::Owned));
    assert!((pred.confidence - 0.99).abs() < 0.001);
}

#[test]
fn rule_based_with_low_weights() {
    let weights = RuleWeights {
        malloc_free: 0.50,
        array_alloc: 0.40,
        const_qual: 0.30,
        write_ops: 0.20,
        size_param: 0.10,
    };
    let classifier = RuleBasedClassifier::with_weights(weights);

    let features = OwnershipFeaturesBuilder::default()
        .const_qualified(true)
        .pointer_depth(1)
        .build();

    let pred = classifier.classify(&features);
    assert!(matches!(pred.prediction, InferredOwnership::Borrowed));
    assert!((pred.confidence - 0.30).abs() < 0.001);
}

// ============================================================================
// Calloc allocation kind path
// ============================================================================

#[test]
fn rule_based_calloc_owned() {
    let classifier = RuleBasedClassifier::new();
    let features = OwnershipFeaturesBuilder::default()
        .allocation_site(AllocationKind::Calloc)
        .deallocation_count(1)
        .pointer_depth(1)
        .build();

    let pred = classifier.classify(&features);
    assert!(matches!(pred.prediction, InferredOwnership::Owned));
    assert!(pred.confidence > 0.9);
}

#[test]
fn rule_based_calloc_vec_with_size() {
    let classifier = RuleBasedClassifier::new();
    let features = OwnershipFeaturesBuilder::default()
        .allocation_site(AllocationKind::Calloc)
        .has_size_param(true)
        .deallocation_count(1)
        .pointer_depth(1)
        .build();

    let pred = classifier.classify(&features);
    assert!(matches!(pred.prediction, InferredOwnership::Vec));
}

#[test]
fn rule_based_calloc_array_decay_no_size_hits_rule1_owned() {
    // Calloc + array_decay + dealloc=1 but no has_size_param
    // Rule 1 checks !has_size_param first, so matches Owned not Vec
    let classifier = RuleBasedClassifier::new();
    let features = OwnershipFeaturesBuilder::default()
        .allocation_site(AllocationKind::Calloc)
        .array_decay(true)
        .deallocation_count(1)
        .pointer_depth(1)
        .build();

    let pred = classifier.classify(&features);
    assert!(matches!(pred.prediction, InferredOwnership::Owned));
}

#[test]
fn rule_based_calloc_array_decay_with_size_hits_rule2_vec() {
    // Calloc + array_decay + has_size_param + dealloc=1 -> Rule 2 Vec
    let classifier = RuleBasedClassifier::new();
    let features = OwnershipFeaturesBuilder::default()
        .allocation_site(AllocationKind::Calloc)
        .array_decay(true)
        .has_size_param(true)
        .deallocation_count(1)
        .pointer_depth(1)
        .build();

    let pred = classifier.classify(&features);
    assert!(matches!(pred.prediction, InferredOwnership::Vec));
}

// ============================================================================
// Const + array_decay + size_param -> Slice (Rule 3 inner branch)
// ============================================================================

#[test]
fn rule_based_const_array_decay_with_size_is_slice() {
    let classifier = RuleBasedClassifier::new();
    let features = OwnershipFeaturesBuilder::default()
        .const_qualified(true)
        .array_decay(true)
        .has_size_param(true)
        .pointer_depth(1)
        .build();

    let pred = classifier.classify(&features);
    assert!(matches!(pred.prediction, InferredOwnership::Slice));
}

// ============================================================================
// Non-const with writes + array_decay + size_param -> SliceMut (Rule 4 inner)
// ============================================================================

#[test]
fn rule_based_mut_write_array_decay_with_size_is_slice_mut() {
    let classifier = RuleBasedClassifier::new();
    let features = OwnershipFeaturesBuilder::default()
        .const_qualified(false)
        .write_count(3)
        .array_decay(true)
        .has_size_param(true)
        .pointer_depth(1)
        .build();

    let pred = classifier.classify(&features);
    assert!(matches!(pred.prediction, InferredOwnership::SliceMut));
}

// ============================================================================
// Rule 5: array_decay + size_param (fallthrough, no const, no writes)
// ============================================================================

#[test]
fn rule_based_array_decay_size_no_const_no_write_is_slice_mut() {
    let classifier = RuleBasedClassifier::new();
    // Not const, no writes, no allocation, array_decay + size_param
    // This should fall through rules 1-4 and hit rule 5
    let features = OwnershipFeaturesBuilder::default()
        .const_qualified(false)
        .write_count(0)
        .array_decay(true)
        .has_size_param(true)
        .pointer_depth(1)
        .build();

    let pred = classifier.classify(&features);
    assert!(matches!(pred.prediction, InferredOwnership::SliceMut));
}

#[test]
fn rule_based_array_decay_size_const_fallthrough_is_slice() {
    let classifier = RuleBasedClassifier::new();
    // const + array_decay + size_param + deallocation > 0
    // Rule 3 requires deallocation_count == 0, so with deallocation this falls through to rule 5
    let features = OwnershipFeaturesBuilder::default()
        .const_qualified(true)
        .array_decay(true)
        .has_size_param(true)
        .deallocation_count(1)
        .pointer_depth(1)
        .build();

    let pred = classifier.classify(&features);
    // Could be Slice from rule 5 (const path)
    assert!(
        matches!(pred.prediction, InferredOwnership::Slice)
            || matches!(pred.prediction, InferredOwnership::Vec)
    );
}

// ============================================================================
// Default fallthrough -> RawPointer with low confidence
// ============================================================================

#[test]
fn rule_based_no_matching_rule_returns_raw_pointer() {
    let classifier = RuleBasedClassifier::new();
    // No allocation, not const, no writes, no array_decay, no size_param
    let features = OwnershipFeaturesBuilder::default()
        .pointer_depth(2)
        .build();

    let pred = classifier.classify(&features);
    assert!(matches!(pred.prediction, InferredOwnership::RawPointer));
    assert!(pred.confidence < 0.5);
    assert!((pred.confidence - 0.3).abs() < 0.001);
}

// ============================================================================
// EvaluationMetrics coverage
// ============================================================================

#[test]
fn metrics_precision_unknown_class_returns_zero() {
    let metrics = EvaluationMetrics::default();
    assert!((metrics.precision("NonExistent") - 0.0).abs() < 0.001);
}

#[test]
fn metrics_recall_unknown_class_returns_zero() {
    let metrics = EvaluationMetrics::default();
    assert!((metrics.recall("NonExistent") - 0.0).abs() < 0.001);
}

#[test]
fn metrics_f1_score_unknown_class_returns_zero() {
    let metrics = EvaluationMetrics::default();
    assert!((metrics.f1_score("NonExistent") - 0.0).abs() < 0.001);
}

#[test]
fn metrics_f1_score_zero_precision_and_recall() {
    let mut metrics = EvaluationMetrics::default();
    // TP=0, FP=0, FN=0 for class means p=0 and r=0
    metrics.true_positives.insert("Empty".to_string(), 0);
    assert!((metrics.f1_score("Empty") - 0.0).abs() < 0.001);
}

#[test]
fn metrics_macro_f1_empty_classes() {
    let metrics = EvaluationMetrics::default();
    assert!((metrics.macro_f1() - 0.0).abs() < 0.001);
}

#[test]
fn metrics_macro_f1_multiple_classes() {
    let mut metrics = EvaluationMetrics::default();
    // Class A: TP=80, FP=20, FN=20 => P=0.8, R=0.8, F1=0.8
    metrics.true_positives.insert("A".to_string(), 80);
    metrics.false_positives.insert("A".to_string(), 20);
    metrics.false_negatives.insert("A".to_string(), 20);
    // Class B: TP=50, FP=10, FN=40 => P=50/60=0.833, R=50/90=0.556, F1=2*(0.833*0.556)/(0.833+0.556)
    metrics.true_positives.insert("B".to_string(), 50);
    metrics.false_positives.insert("B".to_string(), 10);
    metrics.false_negatives.insert("B".to_string(), 40);

    let macro_score = metrics.macro_f1();
    assert!(macro_score > 0.0);
    // F1(A) = 0.8, F1(B) ~ 0.667, macro ~ 0.733
    assert!(macro_score > 0.6 && macro_score < 0.9);
}

#[test]
fn metrics_precision_only_fp_returns_zero() {
    let mut metrics = EvaluationMetrics::default();
    metrics.false_positives.insert("FPOnly".to_string(), 10);
    // TP=0, FP=10 => precision = 0/10 = 0.0
    assert!((metrics.precision("FPOnly") - 0.0).abs() < 0.001);
}

#[test]
fn metrics_recall_only_fn_returns_zero() {
    let mut metrics = EvaluationMetrics::default();
    metrics.false_negatives.insert("FNOnly".to_string(), 10);
    // TP=0, FN=10 => recall = 0/10 = 0.0
    assert!((metrics.recall("FNOnly") - 0.0).abs() < 0.001);
}

// ============================================================================
// ClassifierEvaluator with mismatched predictions
// ============================================================================

#[test]
fn evaluator_evaluate_with_mismatch() {
    let samples = vec![
        TrainingSample::new(
            OwnershipFeaturesBuilder::default()
                .allocation_site(AllocationKind::Malloc)
                .deallocation_count(1)
                .build(),
            InferredOwnership::Borrowed, // Mismatch: features say Owned, label says Borrowed
            "test.c",
            1,
        ),
    ];

    let evaluator = ClassifierEvaluator::new(samples);
    let classifier = RuleBasedClassifier::new();
    let metrics = evaluator.evaluate(&classifier);

    assert_eq!(metrics.total_samples, 1);
    assert_eq!(metrics.correct, 0);
    assert!((metrics.accuracy() - 0.0).abs() < 0.001);
    // Should have false positive for "Owned" and false negative for "Borrowed"
    assert!(metrics.false_positives.contains_key("Owned"));
    assert!(metrics.false_negatives.contains_key("Borrowed"));
}

#[test]
fn evaluator_from_dataset() {
    let sample = TrainingSample::new(
        OwnershipFeaturesBuilder::default()
            .allocation_site(AllocationKind::Malloc)
            .deallocation_count(1)
            .build(),
        InferredOwnership::Owned,
        "test.c",
        1,
    );
    let labeled = crate::training_data::LabeledSample::new(
        sample,
        crate::training_data::DataSource::Synthetic { template: "test".to_string() },
        "int *p = malloc(sizeof(int));",
        "let p = Box::new(0i32);",
    );
    let mut dataset = TrainingDataset::new("test_dataset", "1.0.0");
    dataset.add_all(vec![labeled]);

    let evaluator = ClassifierEvaluator::from_dataset(&dataset);
    assert_eq!(evaluator.sample_count(), 1);
}

// ============================================================================
// TrainingConfig coverage
// ============================================================================

#[test]
fn training_config_custom_values() {
    let config = TrainingConfig {
        validation_split: 0.3,
        random_seed: 123,
        max_iterations: 500,
        early_stopping_patience: 20,
        min_improvement: 0.01,
    };

    assert!((config.validation_split - 0.3).abs() < 0.001);
    assert_eq!(config.random_seed, 123);
    assert_eq!(config.max_iterations, 500);
    assert_eq!(config.early_stopping_patience, 20);
    assert!((config.min_improvement - 0.01).abs() < 0.001);
}

#[test]
fn training_config_default_early_stopping() {
    let config = TrainingConfig::default();
    assert_eq!(config.early_stopping_patience, 10);
    assert!((config.min_improvement - 0.001).abs() < 0.0001);
}

// ============================================================================
// ClassifierTrainer coverage
// ============================================================================

#[test]
fn trainer_with_custom_config() {
    let config = TrainingConfig {
        validation_split: 0.1,
        random_seed: 99,
        max_iterations: 50,
        early_stopping_patience: 5,
        min_improvement: 0.005,
    };
    let trainer = ClassifierTrainer::new(config);

    assert!((trainer.config().validation_split - 0.1).abs() < 0.001);
    assert_eq!(trainer.config().random_seed, 99);
    assert_eq!(trainer.config().max_iterations, 50);
}

#[test]
fn trainer_train_rule_based_returns_working_classifier() {
    let trainer = ClassifierTrainer::with_defaults();
    let dataset = TrainingDataset::new("empty", "1.0.0");

    let (classifier, result) = trainer.train_rule_based(&dataset);
    assert!(result.success);
    assert!(result.duration_secs >= 0.0);
    assert_eq!(result.iterations, 1);
    assert_eq!(classifier.name(), "RuleBasedClassifier");
    assert!(classifier.is_trained());
}

// ============================================================================
// TrainingResult coverage
// ============================================================================

#[test]
fn training_result_success_has_correct_fields() {
    let mut train_metrics = EvaluationMetrics::default();
    train_metrics.total_samples = 100;
    train_metrics.correct = 85;

    let mut val_metrics = EvaluationMetrics::default();
    val_metrics.total_samples = 20;
    val_metrics.correct = 16;

    let result = TrainingResult::success(train_metrics, val_metrics, 50, 2.5);

    assert!(result.success);
    assert_eq!(result.iterations, 50);
    assert!((result.duration_secs - 2.5).abs() < 0.001);
    assert_eq!(result.train_metrics.total_samples, 100);
    assert_eq!(result.validation_metrics.total_samples, 20);
}

#[test]
fn training_result_failure_has_defaults() {
    let result = TrainingResult::failure();

    assert!(!result.success);
    assert_eq!(result.iterations, 0);
    assert!((result.duration_secs - 0.0).abs() < 0.001);
    assert_eq!(result.train_metrics.total_samples, 0);
    assert_eq!(result.validation_metrics.total_samples, 0);
}

// ============================================================================
// EnsembleClassifier coverage - multiple classifiers
// ============================================================================

#[test]
fn ensemble_multiple_classifiers_weighted_voting() {
    let mut ensemble = EnsembleClassifier::new("multi_ensemble");
    ensemble.add_classifier(RuleBasedClassifier::new(), 1.0);
    ensemble.add_classifier(RuleBasedClassifier::new(), 2.0);

    assert_eq!(ensemble.classifier_count(), 2);

    let features = OwnershipFeaturesBuilder::default()
        .allocation_site(AllocationKind::Malloc)
        .deallocation_count(1)
        .build();

    let pred = ensemble.classify(&features);
    assert!(matches!(pred.prediction, InferredOwnership::Owned));
    assert!(pred.confidence > 0.0);
}

#[test]
fn ensemble_classify_borrowed_features() {
    let mut ensemble = EnsembleClassifier::new("borrowed_ensemble");
    ensemble.add_classifier(RuleBasedClassifier::new(), 1.0);

    let features = OwnershipFeaturesBuilder::default()
        .const_qualified(true)
        .pointer_depth(1)
        .build();

    let pred = ensemble.classify(&features);
    assert!(matches!(pred.prediction, InferredOwnership::Borrowed));
}

#[test]
fn ensemble_classify_borrowed_mut_features() {
    let mut ensemble = EnsembleClassifier::new("mut_ensemble");
    ensemble.add_classifier(RuleBasedClassifier::new(), 1.0);

    let features = OwnershipFeaturesBuilder::default()
        .const_qualified(false)
        .write_count(5)
        .pointer_depth(1)
        .build();

    let pred = ensemble.classify(&features);
    assert!(matches!(pred.prediction, InferredOwnership::BorrowedMut));
}

#[test]
fn ensemble_classify_vec_features() {
    let mut ensemble = EnsembleClassifier::new("vec_ensemble");
    ensemble.add_classifier(RuleBasedClassifier::new(), 1.0);

    let features = OwnershipFeaturesBuilder::default()
        .allocation_site(AllocationKind::Malloc)
        .has_size_param(true)
        .deallocation_count(1)
        .pointer_depth(1)
        .build();

    let pred = ensemble.classify(&features);
    assert!(matches!(pred.prediction, InferredOwnership::Vec));
}

#[test]
fn ensemble_classify_slice_features() {
    let mut ensemble = EnsembleClassifier::new("slice_ensemble");
    ensemble.add_classifier(RuleBasedClassifier::new(), 1.0);

    let features = OwnershipFeaturesBuilder::default()
        .const_qualified(true)
        .array_decay(true)
        .has_size_param(true)
        .pointer_depth(1)
        .build();

    let pred = ensemble.classify(&features);
    assert!(matches!(pred.prediction, InferredOwnership::Slice));
}

#[test]
fn ensemble_classify_slice_mut_features() {
    let mut ensemble = EnsembleClassifier::new("slice_mut_ensemble");
    ensemble.add_classifier(RuleBasedClassifier::new(), 1.0);

    let features = OwnershipFeaturesBuilder::default()
        .const_qualified(false)
        .write_count(2)
        .array_decay(true)
        .has_size_param(true)
        .pointer_depth(1)
        .build();

    let pred = ensemble.classify(&features);
    assert!(matches!(pred.prediction, InferredOwnership::SliceMut));
}

#[test]
fn ensemble_classify_raw_pointer_features() {
    let mut ensemble = EnsembleClassifier::new("raw_ptr_ensemble");
    ensemble.add_classifier(RuleBasedClassifier::new(), 1.0);

    let features = OwnershipFeaturesBuilder::default()
        .pointer_depth(2)
        .build();

    let pred = ensemble.classify(&features);
    assert!(matches!(pred.prediction, InferredOwnership::RawPointer));
}

#[test]
fn ensemble_is_trained_all_trained() {
    let mut ensemble = EnsembleClassifier::new("trained");
    ensemble.add_classifier(RuleBasedClassifier::new(), 1.0);
    ensemble.add_classifier(RuleBasedClassifier::new(), 0.5);
    assert!(ensemble.is_trained());
}

#[test]
fn ensemble_confidence_is_normalized() {
    let mut ensemble = EnsembleClassifier::new("normalized");
    ensemble.add_classifier(RuleBasedClassifier::new(), 1.0);
    ensemble.add_classifier(RuleBasedClassifier::new(), 1.0);

    let features = OwnershipFeaturesBuilder::default()
        .allocation_site(AllocationKind::Malloc)
        .deallocation_count(1)
        .build();

    let pred = ensemble.classify(&features);
    // Confidence should be normalized by total weight
    assert!(pred.confidence > 0.0);
    assert!(pred.confidence <= 1.0);
}

// ============================================================================
// ClassifierPrediction additional coverage
// ============================================================================

#[test]
fn prediction_with_multiple_alternatives() {
    let pred = ClassifierPrediction::new(InferredOwnership::Owned, 0.7)
        .with_alternative(InferredOwnership::Borrowed, 0.2)
        .with_alternative(InferredOwnership::RawPointer, 0.1);

    assert_eq!(pred.alternatives.len(), 2);
    assert!(matches!(pred.alternatives[0].0, InferredOwnership::Borrowed));
    assert!((pred.alternatives[0].1 - 0.2).abs() < 0.001);
    assert!(matches!(
        pred.alternatives[1].0,
        InferredOwnership::RawPointer
    ));
}

#[test]
fn prediction_is_confident_at_threshold_boundary() {
    let pred = ClassifierPrediction::new(InferredOwnership::Owned, 0.5);
    assert!(pred.is_confident(0.5)); // Exactly at threshold
    assert!(!pred.is_confident(0.500001)); // Just above
}

#[test]
fn prediction_is_confident_zero_threshold() {
    let pred = ClassifierPrediction::new(InferredOwnership::RawPointer, 0.0);
    assert!(pred.is_confident(0.0));
}

// ============================================================================
// RuleWeights field access coverage
// ============================================================================

#[test]
fn rule_weights_default_values() {
    let weights = RuleWeights::default();
    assert!((weights.malloc_free - 0.95).abs() < 0.001);
    assert!((weights.array_alloc - 0.90).abs() < 0.001);
    assert!((weights.const_qual - 0.85).abs() < 0.001);
    assert!((weights.write_ops - 0.80).abs() < 0.001);
    assert!((weights.size_param - 0.75).abs() < 0.001);
}

#[test]
fn rule_weights_clone() {
    let w1 = RuleWeights::default();
    let w2 = w1.clone();
    assert!((w1.malloc_free - w2.malloc_free).abs() < 0.001);
    assert!((w1.array_alloc - w2.array_alloc).abs() < 0.001);
}

#[test]
fn rule_weights_debug_format() {
    let weights = RuleWeights::default();
    let debug_str = format!("{:?}", weights);
    assert!(debug_str.contains("malloc_free"));
    assert!(debug_str.contains("array_alloc"));
}

// ============================================================================
// Batch classification coverage
// ============================================================================

#[test]
fn batch_classify_empty_input() {
    let classifier = RuleBasedClassifier::new();
    let predictions = classifier.classify_batch(&[]);
    assert!(predictions.is_empty());
}

#[test]
fn batch_classify_various_patterns() {
    let classifier = RuleBasedClassifier::new();
    let features = vec![
        OwnershipFeaturesBuilder::default()
            .allocation_site(AllocationKind::Malloc)
            .deallocation_count(1)
            .build(),
        OwnershipFeaturesBuilder::default()
            .const_qualified(true)
            .build(),
        OwnershipFeaturesBuilder::default()
            .write_count(3)
            .build(),
        OwnershipFeaturesBuilder::default()
            .pointer_depth(2)
            .build(),
    ];

    let predictions = classifier.classify_batch(&features);
    assert_eq!(predictions.len(), 4);
    assert!(matches!(
        predictions[0].prediction,
        InferredOwnership::Owned
    ));
    assert!(matches!(
        predictions[1].prediction,
        InferredOwnership::Borrowed
    ));
    assert!(matches!(
        predictions[2].prediction,
        InferredOwnership::BorrowedMut
    ));
    assert!(matches!(
        predictions[3].prediction,
        InferredOwnership::RawPointer
    ));
}

// ============================================================================
// ClassifierTrainer config access
// ============================================================================

#[test]
fn trainer_config_returns_reference() {
    let config = TrainingConfig {
        validation_split: 0.15,
        random_seed: 7,
        max_iterations: 200,
        early_stopping_patience: 15,
        min_improvement: 0.002,
    };
    let trainer = ClassifierTrainer::new(config);
    let cfg = trainer.config();
    assert_eq!(cfg.random_seed, 7);
    assert_eq!(cfg.max_iterations, 200);
}

// ============================================================================
// EnsembleClassifier edge cases
// ============================================================================

#[test]
fn ensemble_name_returns_correct_name() {
    let ensemble = EnsembleClassifier::new("my_special_ensemble");
    assert_eq!(ensemble.name(), "my_special_ensemble");
}

#[test]
fn ensemble_empty_classify_gives_zero_confidence() {
    let ensemble = EnsembleClassifier::new("empty");
    let features = OwnershipFeaturesBuilder::default()
        .allocation_site(AllocationKind::Malloc)
        .deallocation_count(1)
        .build();

    let pred = ensemble.classify(&features);
    assert!(matches!(pred.prediction, InferredOwnership::RawPointer));
    assert!((pred.confidence - 0.0).abs() < 0.001);
}

#[test]
fn ensemble_single_classifier_zero_weight() {
    let mut ensemble = EnsembleClassifier::new("zero_weight");
    ensemble.add_classifier(RuleBasedClassifier::new(), 0.0);

    let features = OwnershipFeaturesBuilder::default()
        .allocation_site(AllocationKind::Malloc)
        .deallocation_count(1)
        .build();

    let pred = ensemble.classify(&features);
    // With zero total weight, confidence should be 0.0
    assert!((pred.confidence - 0.0).abs() < 0.001);
}

// ============================================================================
// TrainingConfig Debug/Clone coverage
// ============================================================================

#[test]
fn training_config_debug() {
    let config = TrainingConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("validation_split"));
    assert!(debug_str.contains("random_seed"));
}

#[test]
fn training_config_clone() {
    let config = TrainingConfig::default();
    let config2 = config.clone();
    assert_eq!(config.random_seed, config2.random_seed);
    assert_eq!(config.max_iterations, config2.max_iterations);
}

// ============================================================================
// Evaluator with mixed correct/incorrect predictions
// ============================================================================

#[test]
fn evaluator_mixed_results_accuracy() {
    let samples = vec![
        TrainingSample::new(
            OwnershipFeaturesBuilder::default()
                .allocation_site(AllocationKind::Malloc)
                .deallocation_count(1)
                .build(),
            InferredOwnership::Owned, // Correct
            "test.c",
            1,
        ),
        TrainingSample::new(
            OwnershipFeaturesBuilder::default()
                .const_qualified(true)
                .build(),
            InferredOwnership::Owned, // Mismatch: classifier says Borrowed
            "test.c",
            2,
        ),
        TrainingSample::new(
            OwnershipFeaturesBuilder::default()
                .write_count(5)
                .build(),
            InferredOwnership::BorrowedMut, // Correct
            "test.c",
            3,
        ),
    ];

    let evaluator = ClassifierEvaluator::new(samples);
    let classifier = RuleBasedClassifier::new();
    let metrics = evaluator.evaluate(&classifier);

    assert_eq!(metrics.total_samples, 3);
    assert_eq!(metrics.correct, 2);
    assert!((metrics.accuracy() - 2.0 / 3.0).abs() < 0.001);
}

// ============================================================================
// Malloc with array_decay (no size_param) -> Owned not Vec
// ============================================================================

#[test]
fn rule_based_malloc_array_decay_no_size_hits_rule1_owned() {
    // Malloc + array_decay but no has_size_param
    // Rule 1 (!has_size_param) matches first -> Owned, not Vec
    let classifier = RuleBasedClassifier::new();
    let features = OwnershipFeaturesBuilder::default()
        .allocation_site(AllocationKind::Malloc)
        .array_decay(true)
        .deallocation_count(1)
        .pointer_depth(1)
        .build();

    let pred = classifier.classify(&features);
    assert!(matches!(pred.prediction, InferredOwnership::Owned));
}

// ============================================================================
// Const with deallocation > 0 falls through rule 3
// ============================================================================

#[test]
fn rule_based_const_with_deallocation_skips_rule3() {
    let classifier = RuleBasedClassifier::new();
    // const but deallocation > 0 means rule 3 doesn't apply
    let features = OwnershipFeaturesBuilder::default()
        .const_qualified(true)
        .deallocation_count(2)
        .pointer_depth(1)
        .build();

    let pred = classifier.classify(&features);
    // Not Borrowed (rule 3 requires deallocation == 0)
    // Falls to default RawPointer
    assert!(matches!(pred.prediction, InferredOwnership::RawPointer));
}

// ============================================================================
// Non-const with writes but has allocation (not BorrowedMut path)
// ============================================================================

#[test]
fn rule_based_non_const_writes_with_malloc_skips_rule4() {
    let classifier = RuleBasedClassifier::new();
    // Rule 4 requires NOT Malloc/Calloc, so this skips it
    let features = OwnershipFeaturesBuilder::default()
        .const_qualified(false)
        .write_count(5)
        .allocation_site(AllocationKind::Malloc)
        .deallocation_count(1)
        .pointer_depth(1)
        .build();

    let pred = classifier.classify(&features);
    // Should hit rule 1 (malloc + free) -> Owned
    assert!(matches!(pred.prediction, InferredOwnership::Owned));
}
