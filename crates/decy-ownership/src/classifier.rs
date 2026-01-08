//! ML classifier infrastructure for ownership inference (DECY-ML-011).
//!
//! Provides infrastructure for training and using ML classifiers
//! for ownership pattern recognition. Designed to integrate with
//! external ML libraries (e.g., aprender for RandomForest).
//!
//! # Architecture
//!
//! The classifier system uses a trait-based design:
//! - [`OwnershipClassifier`]: Core trait for all classifiers
//! - [`RuleBasedClassifier`]: Baseline deterministic classifier
//! - [`EnsembleClassifier`]: Combines multiple classifiers
//!
//! # Training Flow
//!
//! 1. Collect labeled data via [`TrainingDataset`](crate::training_data::TrainingDataset)
//! 2. Train classifier using [`ClassifierTrainer`]
//! 3. Validate using [`ClassifierEvaluator`]
//! 4. Deploy via [`HybridClassifier`](crate::hybrid_classifier::HybridClassifier)

use std::collections::HashMap;

use crate::ml_features::{AllocationKind, InferredOwnership, OwnershipFeatures};
use crate::retraining_pipeline::TrainingSample;
use crate::training_data::TrainingDataset;

/// A classification prediction with confidence.
#[derive(Debug, Clone)]
pub struct ClassifierPrediction {
    /// Predicted ownership kind.
    pub prediction: InferredOwnership,
    /// Confidence score (0.0 - 1.0).
    pub confidence: f64,
    /// Alternative predictions with confidences.
    pub alternatives: Vec<(InferredOwnership, f64)>,
}

impl ClassifierPrediction {
    /// Create a new prediction.
    pub fn new(prediction: InferredOwnership, confidence: f64) -> Self {
        Self {
            prediction,
            confidence,
            alternatives: Vec::new(),
        }
    }

    /// Add an alternative prediction.
    pub fn with_alternative(mut self, kind: InferredOwnership, confidence: f64) -> Self {
        self.alternatives.push((kind, confidence));
        self
    }

    /// Check if prediction is confident (above threshold).
    pub fn is_confident(&self, threshold: f64) -> bool {
        self.confidence >= threshold
    }
}

/// Core trait for ownership classifiers.
pub trait OwnershipClassifier: Send + Sync {
    /// Classify a feature vector.
    fn classify(&self, features: &OwnershipFeatures) -> ClassifierPrediction;

    /// Classify multiple samples (batch prediction).
    fn classify_batch(&self, features: &[OwnershipFeatures]) -> Vec<ClassifierPrediction> {
        features.iter().map(|f| self.classify(f)).collect()
    }

    /// Get classifier name.
    fn name(&self) -> &str;

    /// Check if classifier is trained.
    fn is_trained(&self) -> bool;
}

/// Rule-based baseline classifier.
///
/// Uses deterministic rules based on feature patterns:
/// - `malloc + free` → `Owned`
/// - `malloc + size_param + array_decay` → `Vec`
/// - `const` → `Borrowed`
/// - `write_count > 0` → `BorrowedMut`
/// - `array_decay + size_param` → `Slice`
#[derive(Debug, Clone, Default)]
pub struct RuleBasedClassifier {
    /// Rule weights for confidence scoring.
    weights: RuleWeights,
}

/// Weights for rule-based confidence scoring.
#[derive(Debug, Clone)]
pub struct RuleWeights {
    /// Weight for malloc/free pattern.
    pub malloc_free: f64,
    /// Weight for array allocation pattern.
    pub array_alloc: f64,
    /// Weight for const qualifier.
    pub const_qual: f64,
    /// Weight for write operations.
    pub write_ops: f64,
    /// Weight for size parameter.
    pub size_param: f64,
}

impl Default for RuleWeights {
    fn default() -> Self {
        Self {
            malloc_free: 0.95,
            array_alloc: 0.90,
            const_qual: 0.85,
            write_ops: 0.80,
            size_param: 0.75,
        }
    }
}

impl RuleBasedClassifier {
    /// Create a new rule-based classifier.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom weights.
    pub fn with_weights(weights: RuleWeights) -> Self {
        Self { weights }
    }
}

impl OwnershipClassifier for RuleBasedClassifier {
    fn classify(&self, features: &OwnershipFeatures) -> ClassifierPrediction {
        // Rule 1: malloc + free → Owned (Box)
        if matches!(
            features.allocation_site,
            AllocationKind::Malloc | AllocationKind::Calloc
        ) && features.deallocation_count > 0
            && !features.has_size_param
        {
            return ClassifierPrediction::new(InferredOwnership::Owned, self.weights.malloc_free);
        }

        // Rule 2: malloc + size + array_decay → Vec
        if matches!(
            features.allocation_site,
            AllocationKind::Malloc | AllocationKind::Calloc
        ) && (features.has_size_param || features.is_array_decay)
            && features.deallocation_count > 0
        {
            return ClassifierPrediction::new(InferredOwnership::Vec, self.weights.array_alloc);
        }

        // Rule 3: const pointer → Borrowed (&T)
        if features.is_const && features.deallocation_count == 0 {
            // Check for slice pattern
            if features.is_array_decay && features.has_size_param {
                return ClassifierPrediction::new(
                    InferredOwnership::Slice,
                    self.weights.size_param,
                );
            }
            return ClassifierPrediction::new(InferredOwnership::Borrowed, self.weights.const_qual);
        }

        // Rule 4: non-const with writes → BorrowedMut (&mut T)
        if !features.is_const
            && features.write_count > 0
            && features.deallocation_count == 0
            && !matches!(
                features.allocation_site,
                AllocationKind::Malloc | AllocationKind::Calloc
            )
        {
            // Check for mutable slice pattern
            if features.is_array_decay && features.has_size_param {
                return ClassifierPrediction::new(
                    InferredOwnership::SliceMut,
                    self.weights.size_param,
                );
            }
            return ClassifierPrediction::new(
                InferredOwnership::BorrowedMut,
                self.weights.write_ops,
            );
        }

        // Rule 5: array decay with size → Slice
        if features.is_array_decay && features.has_size_param {
            let ownership = if features.is_const {
                InferredOwnership::Slice
            } else {
                InferredOwnership::SliceMut
            };
            return ClassifierPrediction::new(ownership, self.weights.size_param);
        }

        // Default: low confidence RawPointer
        ClassifierPrediction::new(InferredOwnership::RawPointer, 0.3)
    }

    fn name(&self) -> &str {
        "RuleBasedClassifier"
    }

    fn is_trained(&self) -> bool {
        true // Always trained (deterministic)
    }
}

/// Evaluation metrics for a classifier.
#[derive(Debug, Clone, Default)]
pub struct EvaluationMetrics {
    /// True positives per class.
    pub true_positives: HashMap<String, usize>,
    /// False positives per class.
    pub false_positives: HashMap<String, usize>,
    /// False negatives per class.
    pub false_negatives: HashMap<String, usize>,
    /// Total samples evaluated.
    pub total_samples: usize,
    /// Correct predictions.
    pub correct: usize,
}

impl EvaluationMetrics {
    /// Compute overall accuracy.
    pub fn accuracy(&self) -> f64 {
        if self.total_samples == 0 {
            return 0.0;
        }
        self.correct as f64 / self.total_samples as f64
    }

    /// Compute precision for a class.
    pub fn precision(&self, class: &str) -> f64 {
        let tp = *self.true_positives.get(class).unwrap_or(&0) as f64;
        let fp = *self.false_positives.get(class).unwrap_or(&0) as f64;

        if tp + fp == 0.0 {
            return 0.0;
        }
        tp / (tp + fp)
    }

    /// Compute recall for a class.
    pub fn recall(&self, class: &str) -> f64 {
        let tp = *self.true_positives.get(class).unwrap_or(&0) as f64;
        let fn_ = *self.false_negatives.get(class).unwrap_or(&0) as f64;

        if tp + fn_ == 0.0 {
            return 0.0;
        }
        tp / (tp + fn_)
    }

    /// Compute F1 score for a class.
    pub fn f1_score(&self, class: &str) -> f64 {
        let p = self.precision(class);
        let r = self.recall(class);

        if p + r == 0.0 {
            return 0.0;
        }
        2.0 * p * r / (p + r)
    }

    /// Compute macro-averaged F1 score.
    pub fn macro_f1(&self) -> f64 {
        let classes: Vec<_> = self.true_positives.keys().collect();
        if classes.is_empty() {
            return 0.0;
        }

        let sum: f64 = classes.iter().map(|c| self.f1_score(c)).sum();
        sum / classes.len() as f64
    }
}

/// Classifier evaluator.
pub struct ClassifierEvaluator {
    /// Test samples.
    samples: Vec<TrainingSample>,
}

impl ClassifierEvaluator {
    /// Create evaluator from test samples.
    pub fn new(samples: Vec<TrainingSample>) -> Self {
        Self { samples }
    }

    /// Create from dataset (uses all samples).
    pub fn from_dataset(dataset: &TrainingDataset) -> Self {
        Self {
            samples: dataset.to_training_samples(),
        }
    }

    /// Evaluate a classifier.
    pub fn evaluate(&self, classifier: &dyn OwnershipClassifier) -> EvaluationMetrics {
        let mut metrics = EvaluationMetrics {
            total_samples: self.samples.len(),
            ..Default::default()
        };

        for sample in &self.samples {
            let prediction = classifier.classify(&sample.features);
            let predicted_class = format!("{:?}", prediction.prediction);
            let actual_class = format!("{:?}", sample.label);

            if prediction.prediction == sample.label {
                metrics.correct += 1;
                *metrics.true_positives.entry(actual_class).or_insert(0) += 1;
            } else {
                *metrics
                    .false_positives
                    .entry(predicted_class.clone())
                    .or_insert(0) += 1;
                *metrics.false_negatives.entry(actual_class).or_insert(0) += 1;
            }
        }

        metrics
    }

    /// Get sample count.
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }
}

/// Configuration for classifier training.
#[derive(Debug, Clone)]
pub struct TrainingConfig {
    /// Validation split ratio.
    pub validation_split: f64,
    /// Random seed for reproducibility.
    pub random_seed: u64,
    /// Maximum training iterations.
    pub max_iterations: usize,
    /// Early stopping patience.
    pub early_stopping_patience: usize,
    /// Minimum improvement for early stopping.
    pub min_improvement: f64,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            validation_split: 0.2,
            random_seed: 42,
            max_iterations: 100,
            early_stopping_patience: 10,
            min_improvement: 0.001,
        }
    }
}

/// Result of classifier training.
#[derive(Debug)]
pub struct TrainingResult {
    /// Training succeeded.
    pub success: bool,
    /// Final training metrics.
    pub train_metrics: EvaluationMetrics,
    /// Final validation metrics.
    pub validation_metrics: EvaluationMetrics,
    /// Number of iterations completed.
    pub iterations: usize,
    /// Training duration in seconds.
    pub duration_secs: f64,
}

impl TrainingResult {
    /// Create a successful result.
    pub fn success(
        train_metrics: EvaluationMetrics,
        validation_metrics: EvaluationMetrics,
        iterations: usize,
        duration_secs: f64,
    ) -> Self {
        Self {
            success: true,
            train_metrics,
            validation_metrics,
            iterations,
            duration_secs,
        }
    }

    /// Create a failed result.
    pub fn failure() -> Self {
        Self {
            success: false,
            train_metrics: EvaluationMetrics::default(),
            validation_metrics: EvaluationMetrics::default(),
            iterations: 0,
            duration_secs: 0.0,
        }
    }
}

/// Trainer for classifiers.
pub struct ClassifierTrainer {
    config: TrainingConfig,
}

impl ClassifierTrainer {
    /// Create a new trainer with configuration.
    pub fn new(config: TrainingConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(TrainingConfig::default())
    }

    /// Train a rule-based classifier (returns pre-built classifier).
    ///
    /// Note: Rule-based classifier doesn't require training.
    pub fn train_rule_based(
        &self,
        _dataset: &TrainingDataset,
    ) -> (RuleBasedClassifier, TrainingResult) {
        let start = std::time::Instant::now();
        let classifier = RuleBasedClassifier::new();
        let duration = start.elapsed().as_secs_f64();

        let result = TrainingResult::success(
            EvaluationMetrics::default(),
            EvaluationMetrics::default(),
            1,
            duration,
        );

        (classifier, result)
    }

    /// Get training configuration.
    pub fn config(&self) -> &TrainingConfig {
        &self.config
    }
}

/// Ensemble classifier combining multiple classifiers.
pub struct EnsembleClassifier {
    /// Component classifiers with weights.
    classifiers: Vec<(Box<dyn OwnershipClassifier>, f64)>,
    /// Ensemble name.
    name: String,
}

impl EnsembleClassifier {
    /// Create a new ensemble.
    pub fn new(name: &str) -> Self {
        Self {
            classifiers: Vec::new(),
            name: name.to_string(),
        }
    }

    /// Add a classifier with weight.
    pub fn add_classifier<C: OwnershipClassifier + 'static>(&mut self, classifier: C, weight: f64) {
        self.classifiers.push((Box::new(classifier), weight));
    }

    /// Get number of classifiers in ensemble.
    pub fn classifier_count(&self) -> usize {
        self.classifiers.len()
    }
}

impl OwnershipClassifier for EnsembleClassifier {
    fn classify(&self, features: &OwnershipFeatures) -> ClassifierPrediction {
        if self.classifiers.is_empty() {
            return ClassifierPrediction::new(InferredOwnership::RawPointer, 0.0);
        }

        // Collect weighted votes
        let mut votes: HashMap<String, f64> = HashMap::new();

        for (classifier, weight) in &self.classifiers {
            let prediction = classifier.classify(features);
            let key = format!("{:?}", prediction.prediction);
            *votes.entry(key.clone()).or_insert(0.0) += weight * prediction.confidence;
        }

        // Find highest voted class
        let total_weight: f64 = self.classifiers.iter().map(|(_, w)| w).sum();
        let (best_class, best_score) = votes
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(k, v)| (k.clone(), *v))
            .unwrap_or_else(|| ("RawPointer".to_string(), 0.0));

        // Convert back to InferredOwnership
        let prediction = match best_class.as_str() {
            "Owned" => InferredOwnership::Owned,
            "Borrowed" => InferredOwnership::Borrowed,
            "BorrowedMut" => InferredOwnership::BorrowedMut,
            "Vec" => InferredOwnership::Vec,
            "Slice" => InferredOwnership::Slice,
            "SliceMut" => InferredOwnership::SliceMut,
            "Shared" => InferredOwnership::Shared,
            _ => InferredOwnership::RawPointer,
        };

        let confidence = if total_weight > 0.0 {
            best_score / total_weight
        } else {
            0.0
        };

        ClassifierPrediction::new(prediction, confidence)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_trained(&self) -> bool {
        self.classifiers.iter().all(|(c, _)| c.is_trained())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ml_features::OwnershipFeaturesBuilder;

    // ========================================================================
    // ClassifierPrediction tests
    // ========================================================================

    #[test]
    fn prediction_new() {
        let pred = ClassifierPrediction::new(InferredOwnership::Owned, 0.9);

        assert!(matches!(pred.prediction, InferredOwnership::Owned));
        assert!((pred.confidence - 0.9).abs() < 0.001);
        assert!(pred.alternatives.is_empty());
    }

    #[test]
    fn prediction_with_alternative() {
        let pred = ClassifierPrediction::new(InferredOwnership::Owned, 0.9)
            .with_alternative(InferredOwnership::Borrowed, 0.1);

        assert_eq!(pred.alternatives.len(), 1);
    }

    #[test]
    fn prediction_is_confident() {
        let pred = ClassifierPrediction::new(InferredOwnership::Owned, 0.9);

        assert!(pred.is_confident(0.8));
        assert!(!pred.is_confident(0.95));
    }

    // ========================================================================
    // RuleBasedClassifier tests
    // ========================================================================

    #[test]
    fn rule_based_malloc_owned() {
        let classifier = RuleBasedClassifier::new();
        let features = OwnershipFeaturesBuilder::default()
            .allocation_site(AllocationKind::Malloc)
            .deallocation_count(1)
            .pointer_depth(1)
            .build();

        let pred = classifier.classify(&features);

        assert!(matches!(pred.prediction, InferredOwnership::Owned));
        assert!(pred.confidence > 0.9);
    }

    #[test]
    fn rule_based_array_vec() {
        let classifier = RuleBasedClassifier::new();
        let features = OwnershipFeaturesBuilder::default()
            .allocation_site(AllocationKind::Malloc)
            .has_size_param(true)
            .deallocation_count(1)
            .pointer_depth(1)
            .build();

        let pred = classifier.classify(&features);

        assert!(matches!(pred.prediction, InferredOwnership::Vec));
    }

    #[test]
    fn rule_based_const_borrowed() {
        let classifier = RuleBasedClassifier::new();
        let features = OwnershipFeaturesBuilder::default()
            .const_qualified(true)
            .pointer_depth(1)
            .build();

        let pred = classifier.classify(&features);

        assert!(matches!(pred.prediction, InferredOwnership::Borrowed));
    }

    #[test]
    fn rule_based_mut_borrowed() {
        let classifier = RuleBasedClassifier::new();
        let features = OwnershipFeaturesBuilder::default()
            .const_qualified(false)
            .write_count(1)
            .pointer_depth(1)
            .build();

        let pred = classifier.classify(&features);

        assert!(matches!(pred.prediction, InferredOwnership::BorrowedMut));
    }

    #[test]
    fn rule_based_slice() {
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

    #[test]
    fn rule_based_unknown() {
        let classifier = RuleBasedClassifier::new();
        let features = OwnershipFeaturesBuilder::default().pointer_depth(1).build();

        let pred = classifier.classify(&features);

        // Default to RawPointer with low confidence
        assert!(matches!(pred.prediction, InferredOwnership::RawPointer));
        assert!(pred.confidence < 0.5);
    }

    #[test]
    fn rule_based_name() {
        let classifier = RuleBasedClassifier::new();
        assert_eq!(classifier.name(), "RuleBasedClassifier");
    }

    #[test]
    fn rule_based_is_trained() {
        let classifier = RuleBasedClassifier::new();
        assert!(classifier.is_trained());
    }

    #[test]
    fn rule_based_batch_classify() {
        let classifier = RuleBasedClassifier::new();
        let features = vec![
            OwnershipFeaturesBuilder::default()
                .allocation_site(AllocationKind::Malloc)
                .deallocation_count(1)
                .build(),
            OwnershipFeaturesBuilder::default()
                .const_qualified(true)
                .build(),
        ];

        let predictions = classifier.classify_batch(&features);

        assert_eq!(predictions.len(), 2);
        assert!(matches!(
            predictions[0].prediction,
            InferredOwnership::Owned
        ));
        assert!(matches!(
            predictions[1].prediction,
            InferredOwnership::Borrowed
        ));
    }

    // ========================================================================
    // EvaluationMetrics tests
    // ========================================================================

    #[test]
    fn metrics_accuracy() {
        let mut metrics = EvaluationMetrics::default();
        metrics.total_samples = 100;
        metrics.correct = 80;

        assert!((metrics.accuracy() - 0.8).abs() < 0.001);
    }

    #[test]
    fn metrics_accuracy_empty() {
        let metrics = EvaluationMetrics::default();
        assert!((metrics.accuracy() - 0.0).abs() < 0.001);
    }

    #[test]
    fn metrics_precision() {
        let mut metrics = EvaluationMetrics::default();
        metrics.true_positives.insert("Owned".to_string(), 80);
        metrics.false_positives.insert("Owned".to_string(), 20);

        assert!((metrics.precision("Owned") - 0.8).abs() < 0.001);
    }

    #[test]
    fn metrics_recall() {
        let mut metrics = EvaluationMetrics::default();
        metrics.true_positives.insert("Owned".to_string(), 80);
        metrics.false_negatives.insert("Owned".to_string(), 20);

        assert!((metrics.recall("Owned") - 0.8).abs() < 0.001);
    }

    #[test]
    fn metrics_f1_score() {
        let mut metrics = EvaluationMetrics::default();
        metrics.true_positives.insert("Owned".to_string(), 80);
        metrics.false_positives.insert("Owned".to_string(), 20);
        metrics.false_negatives.insert("Owned".to_string(), 20);

        // Precision = 80/100 = 0.8, Recall = 80/100 = 0.8
        // F1 = 2 * 0.8 * 0.8 / (0.8 + 0.8) = 0.8
        assert!((metrics.f1_score("Owned") - 0.8).abs() < 0.001);
    }

    // ========================================================================
    // ClassifierEvaluator tests
    // ========================================================================

    #[test]
    fn evaluator_new() {
        let samples = vec![TrainingSample::new(
            OwnershipFeaturesBuilder::default().build(),
            InferredOwnership::Owned,
            "test.c",
            1,
        )];

        let evaluator = ClassifierEvaluator::new(samples);
        assert_eq!(evaluator.sample_count(), 1);
    }

    #[test]
    fn evaluator_evaluate() {
        let samples = vec![
            TrainingSample::new(
                OwnershipFeaturesBuilder::default()
                    .allocation_site(AllocationKind::Malloc)
                    .deallocation_count(1)
                    .build(),
                InferredOwnership::Owned,
                "test.c",
                1,
            ),
            TrainingSample::new(
                OwnershipFeaturesBuilder::default()
                    .const_qualified(true)
                    .build(),
                InferredOwnership::Borrowed,
                "test.c",
                2,
            ),
        ];

        let evaluator = ClassifierEvaluator::new(samples);
        let classifier = RuleBasedClassifier::new();
        let metrics = evaluator.evaluate(&classifier);

        assert_eq!(metrics.total_samples, 2);
        assert_eq!(metrics.correct, 2);
        assert!((metrics.accuracy() - 1.0).abs() < 0.001);
    }

    // ========================================================================
    // TrainingConfig tests
    // ========================================================================

    #[test]
    fn training_config_default() {
        let config = TrainingConfig::default();

        assert!((config.validation_split - 0.2).abs() < 0.001);
        assert_eq!(config.random_seed, 42);
        assert_eq!(config.max_iterations, 100);
    }

    // ========================================================================
    // TrainingResult tests
    // ========================================================================

    #[test]
    fn training_result_success() {
        let result = TrainingResult::success(
            EvaluationMetrics::default(),
            EvaluationMetrics::default(),
            10,
            1.5,
        );

        assert!(result.success);
        assert_eq!(result.iterations, 10);
        assert!((result.duration_secs - 1.5).abs() < 0.001);
    }

    #[test]
    fn training_result_failure() {
        let result = TrainingResult::failure();

        assert!(!result.success);
        assert_eq!(result.iterations, 0);
    }

    // ========================================================================
    // ClassifierTrainer tests
    // ========================================================================

    #[test]
    fn trainer_with_defaults() {
        let trainer = ClassifierTrainer::with_defaults();
        assert!((trainer.config().validation_split - 0.2).abs() < 0.001);
    }

    #[test]
    fn trainer_train_rule_based() {
        let trainer = ClassifierTrainer::with_defaults();
        let dataset = crate::training_data::TrainingDataset::new("test", "1.0.0");

        let (classifier, result) = trainer.train_rule_based(&dataset);

        assert!(result.success);
        assert!(classifier.is_trained());
    }

    // ========================================================================
    // EnsembleClassifier tests
    // ========================================================================

    #[test]
    fn ensemble_new() {
        let ensemble = EnsembleClassifier::new("test_ensemble");

        assert_eq!(ensemble.name(), "test_ensemble");
        assert_eq!(ensemble.classifier_count(), 0);
    }

    #[test]
    fn ensemble_add_classifier() {
        let mut ensemble = EnsembleClassifier::new("test");
        ensemble.add_classifier(RuleBasedClassifier::new(), 1.0);

        assert_eq!(ensemble.classifier_count(), 1);
    }

    #[test]
    fn ensemble_classify_empty() {
        let ensemble = EnsembleClassifier::new("empty");
        let features = OwnershipFeaturesBuilder::default().build();

        let pred = ensemble.classify(&features);

        assert!(matches!(pred.prediction, InferredOwnership::RawPointer));
        assert!((pred.confidence - 0.0).abs() < 0.001);
    }

    #[test]
    fn ensemble_classify_single() {
        let mut ensemble = EnsembleClassifier::new("single");
        ensemble.add_classifier(RuleBasedClassifier::new(), 1.0);

        let features = OwnershipFeaturesBuilder::default()
            .allocation_site(AllocationKind::Malloc)
            .deallocation_count(1)
            .build();

        let pred = ensemble.classify(&features);

        assert!(matches!(pred.prediction, InferredOwnership::Owned));
    }

    #[test]
    fn ensemble_is_trained() {
        let mut ensemble = EnsembleClassifier::new("test");
        // Empty ensemble is vacuously trained (all 0 classifiers are trained)
        assert!(ensemble.is_trained());

        ensemble.add_classifier(RuleBasedClassifier::new(), 1.0);
        assert!(ensemble.is_trained()); // Rule-based always trained
    }

    // ========================================================================
    // Integration tests
    // ========================================================================

    #[test]
    fn full_training_pipeline() {
        // Create synthetic dataset
        let config = crate::training_data::SyntheticConfig {
            samples_per_pattern: 20,
            ..Default::default()
        };
        let generator = crate::training_data::SyntheticDataGenerator::new(config);
        let dataset = generator.generate_full_dataset();

        // Train classifier
        let trainer = ClassifierTrainer::with_defaults();
        let (classifier, result) = trainer.train_rule_based(&dataset);

        assert!(result.success);

        // Evaluate
        let evaluator = ClassifierEvaluator::from_dataset(&dataset);
        let metrics = evaluator.evaluate(&classifier);

        // Should have high accuracy on synthetic data
        assert!(metrics.accuracy() > 0.8);
    }
}
