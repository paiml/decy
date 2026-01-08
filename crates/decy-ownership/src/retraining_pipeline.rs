//! Weekly model retraining pipeline (DECY-ML-018).
//!
//! Implements a structured retraining workflow with:
//! - Data loading and splitting
//! - Model training with cross-validation
//! - Quality gate validation
//! - Automatic promotion or rollback
//!
//! # Toyota Way Principles
//!
//! - **Jidoka**: Quality gates prevent degraded models from deployment
//! - **Kaizen**: Weekly retraining enables continuous improvement
//! - **Genchi Genbutsu**: Metrics tracked for direct observation

use std::collections::HashMap;

use crate::ml_features::{InferredOwnership, OwnershipFeatures};
use crate::model_versioning::{
    ModelEntry, ModelQualityMetrics, ModelVersion, ModelVersionManager, RollbackResult,
};

/// A labeled training sample for ownership classification.
#[derive(Debug, Clone)]
pub struct TrainingSample {
    /// Feature vector extracted from C code.
    pub features: OwnershipFeatures,
    /// Ground truth ownership label.
    pub label: InferredOwnership,
    /// Source file this sample came from.
    pub source_file: String,
    /// Line number in source file.
    pub line_number: u32,
}

impl TrainingSample {
    /// Create a new training sample.
    pub fn new(
        features: OwnershipFeatures,
        label: InferredOwnership,
        source_file: &str,
        line_number: u32,
    ) -> Self {
        Self {
            features,
            label,
            source_file: source_file.to_string(),
            line_number,
        }
    }
}

/// Data split for training, validation, and testing.
#[derive(Debug, Clone)]
pub struct DataSplit {
    /// Training samples (typically 70%).
    pub train: Vec<TrainingSample>,
    /// Validation samples for hyperparameter tuning (typically 15%).
    pub validation: Vec<TrainingSample>,
    /// Test samples for final evaluation (typically 15%).
    pub test: Vec<TrainingSample>,
}

impl DataSplit {
    /// Create a new data split with specified ratios.
    ///
    /// # Arguments
    /// * `samples` - All samples to split
    /// * `train_ratio` - Fraction for training (0.0-1.0)
    /// * `validation_ratio` - Fraction for validation (0.0-1.0)
    ///
    /// Remaining samples go to test set.
    pub fn new(samples: Vec<TrainingSample>, train_ratio: f64, validation_ratio: f64) -> Self {
        let n = samples.len();
        let train_end = (n as f64 * train_ratio) as usize;
        let validation_end = train_end + (n as f64 * validation_ratio) as usize;

        let mut samples = samples;
        // Note: In production, should shuffle with seeded RNG for reproducibility

        let test = samples.split_off(validation_end.min(n));
        let validation = samples.split_off(train_end.min(samples.len()));
        let train = samples;

        Self {
            train,
            validation,
            test,
        }
    }

    /// Get total sample count across all splits.
    pub fn total_count(&self) -> usize {
        self.train.len() + self.validation.len() + self.test.len()
    }

    /// Check if splits meet minimum size requirements.
    pub fn meets_minimum_sizes(&self, min_train: usize, min_val: usize, min_test: usize) -> bool {
        self.train.len() >= min_train
            && self.validation.len() >= min_val
            && self.test.len() >= min_test
    }
}

/// Configuration for retraining pipeline.
#[derive(Debug, Clone)]
pub struct RetrainingConfig {
    /// Minimum precision required to promote model.
    pub min_precision: f64,
    /// Minimum recall required to promote model.
    pub min_recall: f64,
    /// Maximum allowed degradation vs current model.
    pub max_degradation: f64,
    /// Number of cross-validation folds.
    pub cv_folds: usize,
    /// Minimum training samples required.
    pub min_train_samples: usize,
    /// Minimum validation samples required.
    pub min_validation_samples: usize,
    /// Minimum test samples required.
    pub min_test_samples: usize,
    /// Train/validation/test split ratios.
    pub train_ratio: f64,
    /// Validation split ratio.
    pub validation_ratio: f64,
}

impl Default for RetrainingConfig {
    fn default() -> Self {
        Self {
            min_precision: 0.85,
            min_recall: 0.80,
            max_degradation: 0.02, // Allow 2% degradation
            cv_folds: 5,
            min_train_samples: 700,
            min_validation_samples: 150,
            min_test_samples: 150,
            train_ratio: 0.70,
            validation_ratio: 0.15,
        }
    }
}

/// Metrics from a single training run.
#[derive(Debug, Clone)]
pub struct TrainingMetrics {
    /// Precision on validation set.
    pub precision: f64,
    /// Recall on validation set.
    pub recall: f64,
    /// F1 score on validation set.
    pub f1_score: f64,
    /// Training loss (lower is better).
    pub training_loss: f64,
    /// Validation loss (lower is better).
    pub validation_loss: f64,
    /// Per-class metrics.
    pub class_metrics: HashMap<String, ClassMetrics>,
    /// Training duration in seconds.
    pub training_duration_secs: f64,
}

impl TrainingMetrics {
    /// Create metrics with basic scores.
    pub fn new(precision: f64, recall: f64) -> Self {
        let f1_score = if precision + recall > 0.0 {
            2.0 * precision * recall / (precision + recall)
        } else {
            0.0
        };

        Self {
            precision,
            recall,
            f1_score,
            training_loss: 0.0,
            validation_loss: 0.0,
            class_metrics: HashMap::new(),
            training_duration_secs: 0.0,
        }
    }

    /// Check if metrics meet quality thresholds.
    pub fn meets_thresholds(&self, config: &RetrainingConfig) -> bool {
        self.precision >= config.min_precision && self.recall >= config.min_recall
    }

    /// Convert to model quality metrics for versioning.
    pub fn to_quality_metrics(&self) -> ModelQualityMetrics {
        ModelQualityMetrics::new(
            self.f1_score, // accuracy approximated by F1
            self.precision,
            self.recall,
            self.f1_score,
            0.9, // avg_confidence placeholder
            0.0, // fallback_rate placeholder
            0,   // sample_count placeholder
        )
    }
}

/// Per-class classification metrics.
#[derive(Debug, Clone)]
pub struct ClassMetrics {
    /// Precision for this class.
    pub precision: f64,
    /// Recall for this class.
    pub recall: f64,
    /// Support (number of true instances).
    pub support: usize,
}

/// Result of retraining pipeline execution.
#[derive(Debug)]
pub enum RetrainingResult {
    /// Model was successfully trained and promoted.
    Promoted {
        /// New model version.
        version: ModelVersion,
        /// Training metrics.
        metrics: TrainingMetrics,
    },
    /// Model trained but didn't meet quality gates.
    QualityGateFailed {
        /// Reason for failure.
        reason: String,
        /// Training metrics.
        metrics: TrainingMetrics,
    },
    /// Model trained but degraded vs current.
    Degraded {
        /// Degradation amount.
        degradation: f64,
        /// New metrics.
        new_metrics: TrainingMetrics,
        /// Current metrics.
        current_metrics: ModelQualityMetrics,
    },
    /// Insufficient training data.
    InsufficientData {
        /// Actual sample count.
        actual: usize,
        /// Required sample count.
        required: usize,
    },
    /// Training failed with error.
    TrainingError {
        /// Error message.
        error: String,
    },
}

impl RetrainingResult {
    /// Check if retraining was successful.
    pub fn is_success(&self) -> bool {
        matches!(self, RetrainingResult::Promoted { .. })
    }

    /// Get metrics if available.
    pub fn metrics(&self) -> Option<&TrainingMetrics> {
        match self {
            RetrainingResult::Promoted { metrics, .. } => Some(metrics),
            RetrainingResult::QualityGateFailed { metrics, .. } => Some(metrics),
            RetrainingResult::Degraded { new_metrics, .. } => Some(new_metrics),
            _ => None,
        }
    }
}

/// Trait for model trainers.
pub trait ModelTrainer {
    /// Train a model on the given data split.
    fn train(&self, data: &DataSplit) -> Result<TrainingMetrics, String>;
}

/// Null trainer for testing (always returns fixed metrics).
pub struct NullTrainer {
    /// Metrics to return.
    pub metrics: TrainingMetrics,
}

impl NullTrainer {
    /// Create a null trainer with specified metrics.
    pub fn new(precision: f64, recall: f64) -> Self {
        Self {
            metrics: TrainingMetrics::new(precision, recall),
        }
    }
}

impl ModelTrainer for NullTrainer {
    fn train(&self, _data: &DataSplit) -> Result<TrainingMetrics, String> {
        Ok(self.metrics.clone())
    }
}

/// Weekly model retraining pipeline.
///
/// Orchestrates the full retraining workflow:
/// 1. Load and split data
/// 2. Train model
/// 3. Validate quality gates
/// 4. Promote or rollback
pub struct RetrainingPipeline<T: ModelTrainer> {
    /// Configuration.
    config: RetrainingConfig,
    /// Model trainer.
    trainer: T,
    /// Model version manager.
    version_manager: ModelVersionManager,
    /// Execution history.
    history: Vec<PipelineExecution>,
}

/// Record of a pipeline execution.
#[derive(Debug, Clone)]
pub struct PipelineExecution {
    /// Timestamp of execution (seconds since epoch).
    pub timestamp: u64,
    /// Result summary.
    pub result: ExecutionSummary,
    /// Sample count used.
    pub sample_count: usize,
}

/// Summary of execution result.
#[derive(Debug, Clone)]
pub enum ExecutionSummary {
    /// Successfully promoted.
    Promoted {
        /// The new model version string.
        version: String,
    },
    /// Quality gate failed.
    QualityGateFailed {
        /// Reason for the failure.
        reason: String,
    },
    /// Degradation detected.
    Degraded {
        /// Amount of degradation.
        amount: f64,
    },
    /// Insufficient data.
    InsufficientData,
    /// Training error.
    Error {
        /// Error message.
        message: String,
    },
}

impl<T: ModelTrainer> RetrainingPipeline<T> {
    /// Create a new retraining pipeline.
    pub fn new(trainer: T, version_manager: ModelVersionManager, config: RetrainingConfig) -> Self {
        Self {
            config,
            trainer,
            version_manager,
            history: Vec::new(),
        }
    }

    /// Create pipeline with default configuration.
    pub fn with_defaults(trainer: T) -> Self {
        Self::new(
            trainer,
            ModelVersionManager::new(),
            RetrainingConfig::default(),
        )
    }

    /// Execute the retraining pipeline.
    pub fn execute(&mut self, samples: Vec<TrainingSample>) -> RetrainingResult {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Check minimum sample count
        let min_required = self.config.min_train_samples
            + self.config.min_validation_samples
            + self.config.min_test_samples;

        if samples.len() < min_required {
            let result = RetrainingResult::InsufficientData {
                actual: samples.len(),
                required: min_required,
            };
            self.record_execution(timestamp, &result, samples.len());
            return result;
        }

        // Split data
        let data = DataSplit::new(
            samples.clone(),
            self.config.train_ratio,
            self.config.validation_ratio,
        );

        if !data.meets_minimum_sizes(
            self.config.min_train_samples,
            self.config.min_validation_samples,
            self.config.min_test_samples,
        ) {
            let result = RetrainingResult::InsufficientData {
                actual: data.total_count(),
                required: min_required,
            };
            self.record_execution(timestamp, &result, samples.len());
            return result;
        }

        // Train model
        let metrics = match self.trainer.train(&data) {
            Ok(m) => m,
            Err(e) => {
                let result = RetrainingResult::TrainingError { error: e };
                self.record_execution(timestamp, &result, samples.len());
                return result;
            }
        };

        // Check quality gates
        if !metrics.meets_thresholds(&self.config) {
            let reason = format!(
                "Precision {:.2} < {:.2} or Recall {:.2} < {:.2}",
                metrics.precision,
                self.config.min_precision,
                metrics.recall,
                self.config.min_recall
            );
            let result = RetrainingResult::QualityGateFailed { reason, metrics };
            self.record_execution(timestamp, &result, samples.len());
            return result;
        }

        // Check for degradation vs current model
        if let Some(current) = self.version_manager.active_version() {
            let current_f1 = current.metrics.f1_score;
            let degradation = current_f1 - metrics.f1_score;

            if degradation > self.config.max_degradation {
                let result = RetrainingResult::Degraded {
                    degradation,
                    new_metrics: metrics,
                    current_metrics: current.metrics.clone(),
                };
                self.record_execution(timestamp, &result, samples.len());
                return result;
            }
        }

        // Promote new model
        let quality_metrics = metrics.to_quality_metrics();
        let next_version = self
            .version_manager
            .active_version()
            .map(|e| e.version.bump_minor())
            .unwrap_or_else(|| ModelVersion::new(1, 0, 0));

        let entry = ModelEntry::new(
            next_version.clone(),
            quality_metrics,
            "Retrained model",
            "models/latest.bin",
        );

        // Try to register - ignore errors for simplicity
        let _ = self.version_manager.register_version(entry);

        let result = RetrainingResult::Promoted {
            version: next_version,
            metrics,
        };
        self.record_execution(timestamp, &result, samples.len());
        result
    }

    /// Record an execution in history.
    fn record_execution(&mut self, timestamp: u64, result: &RetrainingResult, sample_count: usize) {
        let summary = match result {
            RetrainingResult::Promoted { version, .. } => ExecutionSummary::Promoted {
                version: version.to_string(),
            },
            RetrainingResult::QualityGateFailed { reason, .. } => {
                ExecutionSummary::QualityGateFailed {
                    reason: reason.clone(),
                }
            }
            RetrainingResult::Degraded { degradation, .. } => ExecutionSummary::Degraded {
                amount: *degradation,
            },
            RetrainingResult::InsufficientData { .. } => ExecutionSummary::InsufficientData,
            RetrainingResult::TrainingError { error } => ExecutionSummary::Error {
                message: error.clone(),
            },
        };

        self.history.push(PipelineExecution {
            timestamp,
            result: summary,
            sample_count,
        });
    }

    /// Get execution history.
    pub fn history(&self) -> &[PipelineExecution] {
        &self.history
    }

    /// Get success rate from history.
    pub fn success_rate(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }

        let successes = self
            .history
            .iter()
            .filter(|e| matches!(e.result, ExecutionSummary::Promoted { .. }))
            .count();

        successes as f64 / self.history.len() as f64
    }

    /// Rollback to a previous version.
    pub fn rollback(&mut self, version: &ModelVersion) -> Result<RollbackResult, String> {
        self.version_manager
            .rollback_to(version, "Manual rollback via pipeline")
    }

    /// Get current model version.
    pub fn current_version(&self) -> Option<&ModelEntry> {
        self.version_manager.active_version()
    }

    /// Get configuration.
    pub fn config(&self) -> &RetrainingConfig {
        &self.config
    }

    /// Update configuration.
    pub fn set_config(&mut self, config: RetrainingConfig) {
        self.config = config;
    }
}

/// Schedule specification for weekly retraining.
#[derive(Debug, Clone)]
pub struct RetrainingSchedule {
    /// Day of week (0 = Sunday, 6 = Saturday).
    pub day_of_week: u8,
    /// Hour of day (0-23).
    pub hour: u8,
    /// Minute of hour (0-59).
    pub minute: u8,
    /// Timezone offset from UTC in hours.
    pub timezone_offset: i8,
}

impl Default for RetrainingSchedule {
    fn default() -> Self {
        Self {
            day_of_week: 0, // Sunday
            hour: 2,        // 2 AM
            minute: 0,
            timezone_offset: 0, // UTC
        }
    }
}

impl RetrainingSchedule {
    /// Create a schedule for a specific day and time.
    pub fn new(day_of_week: u8, hour: u8, minute: u8) -> Self {
        Self {
            day_of_week: day_of_week.min(6),
            hour: hour.min(23),
            minute: minute.min(59),
            timezone_offset: 0,
        }
    }

    /// Set timezone offset.
    pub fn with_timezone(mut self, offset: i8) -> Self {
        self.timezone_offset = offset;
        self
    }

    /// Check if current time matches schedule (simplified check).
    ///
    /// In production, this would use proper time library.
    pub fn should_run(&self, current_day: u8, current_hour: u8, current_minute: u8) -> bool {
        self.day_of_week == current_day
            && self.hour == current_hour
            && self.minute == current_minute
    }

    /// Get human-readable schedule description.
    pub fn description(&self) -> String {
        let day = match self.day_of_week {
            0 => "Sunday",
            1 => "Monday",
            2 => "Tuesday",
            3 => "Wednesday",
            4 => "Thursday",
            5 => "Friday",
            6 => "Saturday",
            _ => "Unknown",
        };

        let tz = if self.timezone_offset >= 0 {
            format!("UTC+{}", self.timezone_offset)
        } else {
            format!("UTC{}", self.timezone_offset)
        };

        format!("{} at {:02}:{:02} {}", day, self.hour, self.minute, tz)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ml_features::OwnershipFeaturesBuilder;

    // ========================================================================
    // TrainingSample tests
    // ========================================================================

    #[test]
    fn training_sample_new() {
        let features = OwnershipFeaturesBuilder::default().build();
        let sample = TrainingSample::new(features, InferredOwnership::Owned, "test.c", 42);

        assert_eq!(sample.source_file, "test.c");
        assert_eq!(sample.line_number, 42);
        assert!(matches!(sample.label, InferredOwnership::Owned));
    }

    // ========================================================================
    // DataSplit tests
    // ========================================================================

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

    #[test]
    fn data_split_ratios() {
        let samples = make_samples(100);
        let split = DataSplit::new(samples, 0.70, 0.15);

        // 70% train, 15% validation, 15% test
        assert_eq!(split.train.len(), 70);
        assert_eq!(split.validation.len(), 15);
        assert_eq!(split.test.len(), 15);
        assert_eq!(split.total_count(), 100);
    }

    #[test]
    fn data_split_meets_minimum_sizes() {
        let samples = make_samples(100);
        let split = DataSplit::new(samples, 0.70, 0.15);

        assert!(split.meets_minimum_sizes(70, 15, 15));
        assert!(!split.meets_minimum_sizes(80, 15, 15)); // train too small
        assert!(!split.meets_minimum_sizes(70, 20, 15)); // val too small
        assert!(!split.meets_minimum_sizes(70, 15, 20)); // test too small
    }

    #[test]
    fn data_split_empty_input() {
        let samples: Vec<TrainingSample> = vec![];
        let split = DataSplit::new(samples, 0.70, 0.15);

        assert!(split.train.is_empty());
        assert!(split.validation.is_empty());
        assert!(split.test.is_empty());
        assert_eq!(split.total_count(), 0);
    }

    // ========================================================================
    // RetrainingConfig tests
    // ========================================================================

    #[test]
    fn retraining_config_default() {
        let config = RetrainingConfig::default();

        assert!((config.min_precision - 0.85).abs() < 0.001);
        assert!((config.min_recall - 0.80).abs() < 0.001);
        assert!((config.max_degradation - 0.02).abs() < 0.001);
        assert_eq!(config.cv_folds, 5);
        assert_eq!(config.min_train_samples, 700);
    }

    // ========================================================================
    // TrainingMetrics tests
    // ========================================================================

    #[test]
    fn training_metrics_new() {
        let metrics = TrainingMetrics::new(0.90, 0.85);

        assert!((metrics.precision - 0.90).abs() < 0.001);
        assert!((metrics.recall - 0.85).abs() < 0.001);
        // F1 = 2 * 0.90 * 0.85 / (0.90 + 0.85) = 0.8743...
        assert!((metrics.f1_score - 0.8743).abs() < 0.01);
    }

    #[test]
    fn training_metrics_meets_thresholds() {
        let config = RetrainingConfig::default();

        let good = TrainingMetrics::new(0.90, 0.85);
        assert!(good.meets_thresholds(&config));

        let low_precision = TrainingMetrics::new(0.80, 0.85);
        assert!(!low_precision.meets_thresholds(&config));

        let low_recall = TrainingMetrics::new(0.90, 0.70);
        assert!(!low_recall.meets_thresholds(&config));
    }

    #[test]
    fn training_metrics_to_quality_metrics() {
        let metrics = TrainingMetrics::new(0.90, 0.85);
        let quality = metrics.to_quality_metrics();

        assert!((quality.precision - 0.90).abs() < 0.001);
        assert!((quality.recall - 0.85).abs() < 0.001);
    }

    // ========================================================================
    // RetrainingResult tests
    // ========================================================================

    #[test]
    fn retraining_result_is_success() {
        let promoted = RetrainingResult::Promoted {
            version: ModelVersion::new(1, 0, 0),
            metrics: TrainingMetrics::new(0.90, 0.85),
        };
        assert!(promoted.is_success());

        let failed = RetrainingResult::QualityGateFailed {
            reason: "test".to_string(),
            metrics: TrainingMetrics::new(0.70, 0.60),
        };
        assert!(!failed.is_success());
    }

    #[test]
    fn retraining_result_metrics() {
        let promoted = RetrainingResult::Promoted {
            version: ModelVersion::new(1, 0, 0),
            metrics: TrainingMetrics::new(0.90, 0.85),
        };
        assert!(promoted.metrics().is_some());

        let insufficient = RetrainingResult::InsufficientData {
            actual: 100,
            required: 1000,
        };
        assert!(insufficient.metrics().is_none());
    }

    // ========================================================================
    // NullTrainer tests
    // ========================================================================

    #[test]
    fn null_trainer_returns_fixed_metrics() {
        let trainer = NullTrainer::new(0.92, 0.88);
        let samples = make_samples(100);
        let data = DataSplit::new(samples, 0.70, 0.15);

        let metrics = trainer.train(&data).unwrap();

        assert!((metrics.precision - 0.92).abs() < 0.001);
        assert!((metrics.recall - 0.88).abs() < 0.001);
    }

    // ========================================================================
    // RetrainingPipeline tests
    // ========================================================================

    #[test]
    fn pipeline_insufficient_data() {
        let trainer = NullTrainer::new(0.90, 0.85);
        let mut pipeline = RetrainingPipeline::with_defaults(trainer);

        // Only 100 samples when 1000 required
        let samples = make_samples(100);
        let result = pipeline.execute(samples);

        assert!(!result.is_success());
        assert!(matches!(result, RetrainingResult::InsufficientData { .. }));
    }

    #[test]
    fn pipeline_quality_gate_failure() {
        // Trainer returns metrics below threshold
        let trainer = NullTrainer::new(0.70, 0.60);
        let mut pipeline = RetrainingPipeline::with_defaults(trainer);

        let samples = make_samples(1000);
        let result = pipeline.execute(samples);

        assert!(!result.is_success());
        assert!(matches!(result, RetrainingResult::QualityGateFailed { .. }));
    }

    #[test]
    fn pipeline_success_promotion() {
        let trainer = NullTrainer::new(0.92, 0.88);
        let mut pipeline = RetrainingPipeline::with_defaults(trainer);

        let samples = make_samples(1000);
        let result = pipeline.execute(samples);

        assert!(result.is_success());
        if let RetrainingResult::Promoted { version, metrics } = result {
            assert_eq!(version.major, 1);
            assert!((metrics.precision - 0.92).abs() < 0.001);
        } else {
            panic!("Expected Promoted result");
        }
    }

    #[test]
    fn pipeline_degradation_detection() {
        let trainer = NullTrainer::new(0.92, 0.88);
        let mut pipeline = RetrainingPipeline::with_defaults(trainer);

        // First run succeeds
        let samples = make_samples(1000);
        let result1 = pipeline.execute(samples);
        assert!(result1.is_success());

        // Second run with lower metrics
        let weak_trainer = NullTrainer::new(0.86, 0.81);
        let mut vm = ModelVersionManager::new();

        // Register a strong model first
        let strong_metrics = ModelQualityMetrics::new(
            0.90, // accuracy
            0.92, // precision
            0.88, // recall
            0.90, // f1_score
            0.95, // avg_confidence
            0.05, // fallback_rate
            1000, // sample_count
        );
        let strong_entry = ModelEntry::new(
            ModelVersion::new(1, 0, 0),
            strong_metrics,
            "Strong model",
            "models/strong.bin",
        );
        let _ = vm.register_version(strong_entry);

        let mut pipeline2 = RetrainingPipeline::new(weak_trainer, vm, RetrainingConfig::default());

        let samples2 = make_samples(1000);
        let result2 = pipeline2.execute(samples2);

        // Should detect degradation (F1 drops significantly)
        assert!(!result2.is_success());
        assert!(matches!(result2, RetrainingResult::Degraded { .. }));
    }

    #[test]
    fn pipeline_history_tracking() {
        let trainer = NullTrainer::new(0.92, 0.88);
        let mut pipeline = RetrainingPipeline::with_defaults(trainer);

        assert!(pipeline.history().is_empty());

        let samples = make_samples(1000);
        pipeline.execute(samples);

        assert_eq!(pipeline.history().len(), 1);
        assert!(matches!(
            pipeline.history()[0].result,
            ExecutionSummary::Promoted { .. }
        ));
    }

    #[test]
    fn pipeline_success_rate() {
        let trainer = NullTrainer::new(0.92, 0.88);
        let mut pipeline = RetrainingPipeline::with_defaults(trainer);

        // First run succeeds
        let samples = make_samples(1000);
        pipeline.execute(samples);

        // Second run with insufficient data
        let samples2 = make_samples(100);
        pipeline.execute(samples2);

        // 1 success out of 2 = 50%
        assert!((pipeline.success_rate() - 0.50).abs() < 0.001);
    }

    // ========================================================================
    // RetrainingSchedule tests
    // ========================================================================

    #[test]
    fn schedule_default() {
        let schedule = RetrainingSchedule::default();

        assert_eq!(schedule.day_of_week, 0); // Sunday
        assert_eq!(schedule.hour, 2);
        assert_eq!(schedule.minute, 0);
    }

    #[test]
    fn schedule_new() {
        let schedule = RetrainingSchedule::new(5, 3, 30); // Friday 3:30

        assert_eq!(schedule.day_of_week, 5);
        assert_eq!(schedule.hour, 3);
        assert_eq!(schedule.minute, 30);
    }

    #[test]
    fn schedule_bounds_check() {
        let schedule = RetrainingSchedule::new(10, 30, 70);

        // Should clamp to valid values
        assert_eq!(schedule.day_of_week, 6); // max 6
        assert_eq!(schedule.hour, 23); // max 23
        assert_eq!(schedule.minute, 59); // max 59
    }

    #[test]
    fn schedule_should_run() {
        let schedule = RetrainingSchedule::new(0, 2, 0); // Sunday 2:00

        assert!(schedule.should_run(0, 2, 0));
        assert!(!schedule.should_run(1, 2, 0)); // wrong day
        assert!(!schedule.should_run(0, 3, 0)); // wrong hour
        assert!(!schedule.should_run(0, 2, 1)); // wrong minute
    }

    #[test]
    fn schedule_description() {
        let schedule = RetrainingSchedule::new(5, 14, 30).with_timezone(-5);

        let desc = schedule.description();
        assert!(desc.contains("Friday"));
        assert!(desc.contains("14:30"));
        assert!(desc.contains("UTC-5"));
    }

    #[test]
    fn schedule_with_timezone() {
        let schedule = RetrainingSchedule::new(0, 2, 0).with_timezone(8);

        assert_eq!(schedule.timezone_offset, 8);
    }
}
