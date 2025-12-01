//! Ownership and lifetime inference for C-to-Rust conversion.
//!
//! **CRITICAL COMPONENT**: This is the most important module for quality transpilation.
//! Infers Rust ownership patterns and lifetime annotations from C pointer usage.
//!
//! # ML-Enhanced Features (DECY-ML-001, DECY-ML-003)
//!
//! This crate includes ML-enhanced ownership inference features:
//! - [`OwnershipFeatures`]: 142-dimension feature vector for batch ML processing
//! - [`OwnershipDefect`]: 8-category defect taxonomy (DECY-O-001 through DECY-O-008)
//! - [`InferredOwnership`]: Predicted Rust ownership kinds
//! - [`OwnershipPrediction`]: Ownership with confidence score and fallback

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

pub mod ab_testing;
pub mod active_learning;
pub mod array_slice;
pub mod borrow_gen;
pub mod classifier;
pub mod classifier_integration;
pub mod dataflow;
pub mod error_tracking;
pub mod hybrid_classifier;
pub mod inference;
pub mod lifetime;
pub mod lifetime_gen;
pub mod ml_features;
pub mod model_versioning;
pub mod retraining_pipeline;
pub mod struct_lifetime;
pub mod threshold_tuning;
pub mod training_data;

// Re-export ML feature types at crate root for convenience
pub use ml_features::{
    default_pattern_library, AllocationKind, ErrorPattern, ErrorSeverity, FeatureExtractor,
    InferredOwnership, OwnershipDefect, OwnershipErrorKind, OwnershipFeatures,
    OwnershipFeaturesBuilder, OwnershipPrediction, PatternLibrary, Severity, SuggestedFix,
};

// Re-export hybrid classifier types
pub use hybrid_classifier::{
    ClassificationMethod, HybridClassifier, HybridMetrics, HybridResult, NullModel, OwnershipModel,
    DEFAULT_CONFIDENCE_THRESHOLD,
};

// Re-export A/B testing types (DECY-ML-013)
pub use ab_testing::{
    ABExperiment, ABTestRunner, AssignmentStrategy, TestObservation, TestVariant, VariantMetrics,
};

// Re-export threshold tuning types (DECY-ML-014)
pub use threshold_tuning::{
    find_optimal_threshold, SelectionCriteria, ThresholdMetrics, ThresholdTuner, TuningResult,
    ValidationSample,
};

// Re-export model versioning types (DECY-ML-017)
pub use model_versioning::{
    ModelEntry, ModelQualityMetrics, ModelVersion, ModelVersionManager, QualityThresholds,
    RollbackResult,
};

// Re-export active learning types (DECY-ML-016)
pub use active_learning::{
    ActiveLearner, QueueStats, SampleQueue, SelectionStrategy, UncertainSample,
    UncertaintyCalculator,
};

// Re-export error tracking types (DECY-ML-015)
pub use error_tracking::{
    ErrorTracker, FeatureSuspiciousness, ImprovementSuggestion, InferenceError, PatternStats,
    SuggestionCategory, SuggestionPriority,
};

// Re-export retraining pipeline types (DECY-ML-018)
pub use retraining_pipeline::{
    DataSplit, ModelTrainer, NullTrainer, PipelineExecution, RetrainingConfig,
    RetrainingPipeline, RetrainingResult, RetrainingSchedule, TrainingMetrics, TrainingSample,
};

// Re-export training data types (DECY-ML-010)
pub use training_data::{
    CollectionResult, DataSource, DatasetStats, LabeledSample, SyntheticConfig,
    SyntheticDataGenerator, TrainingDataCollector, TrainingDataset,
};

// Re-export classifier types (DECY-ML-011)
pub use classifier::{
    ClassifierEvaluator, ClassifierPrediction, ClassifierTrainer, EnsembleClassifier,
    EvaluationMetrics, OwnershipClassifier, RuleBasedClassifier, RuleWeights, TrainingConfig,
    TrainingResult,
};

// Re-export inference types for hybrid classifier examples
pub use inference::{OwnershipInference, OwnershipKind};

// Re-export classifier integration types (DECY-182)
pub use classifier_integration::{
    classify_function_variables, classify_with_rules, extract_features_for_variable,
};

#[cfg(test)]
mod ml_features_tests;
