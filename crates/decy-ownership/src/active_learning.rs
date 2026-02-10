//! Active learning for uncertain sample collection (DECY-ML-016).
//!
//! Implements uncertainty sampling to identify low-confidence predictions
//! that would benefit from human labeling to improve the model.
//!
//! # Active Learning Strategy
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                   ACTIVE LEARNING PIPELINE                      │
//! │                                                                 │
//! │  1. Classify samples with ML model                              │
//! │     ├─ High confidence → Use prediction                         │
//! │     └─ Low confidence  → Queue for labeling                     │
//! │                                                                 │
//! │  2. Selection strategies:                                       │
//! │     ├─ Uncertainty sampling (lowest confidence)                 │
//! │     ├─ Margin sampling (smallest margin between top 2)          │
//! │     └─ Entropy sampling (highest prediction entropy)            │
//! │                                                                 │
//! │  3. Human labels uncertain samples                              │
//! │                                                                 │
//! │  4. Retrain model with expanded dataset                         │
//! │                                                                 │
//! │  5. Repeat (continuous improvement cycle)                       │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Toyota Way: Kaizen (Continuous Improvement)
//!
//! Active learning embodies Kaizen by:
//! - Focusing labeling effort on most valuable samples
//! - Incrementally improving model accuracy
//! - Learning from failures (uncertain predictions)

use crate::ml_features::{InferredOwnership, OwnershipFeatures, OwnershipPrediction};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Strategy for selecting uncertain samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionStrategy {
    /// Select samples with lowest confidence
    UncertaintySampling,
    /// Select samples where top-2 predictions are close
    MarginSampling,
    /// Select samples with highest prediction entropy
    EntropySampling,
    /// Random sampling (baseline)
    Random,
}

impl std::fmt::Display for SelectionStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectionStrategy::UncertaintySampling => write!(f, "uncertainty"),
            SelectionStrategy::MarginSampling => write!(f, "margin"),
            SelectionStrategy::EntropySampling => write!(f, "entropy"),
            SelectionStrategy::Random => write!(f, "random"),
        }
    }
}

/// An uncertain sample queued for labeling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncertainSample {
    /// Unique identifier
    pub id: u64,
    /// Variable name being classified
    pub variable: String,
    /// Source file (if known)
    pub source_file: Option<String>,
    /// Source line (if known)
    pub source_line: Option<u32>,
    /// Feature vector
    pub features: OwnershipFeatures,
    /// ML prediction
    pub prediction: OwnershipPrediction,
    /// Uncertainty score (0.0 = certain, 1.0 = uncertain)
    pub uncertainty_score: f64,
    /// Selection strategy that chose this sample
    pub strategy: SelectionStrategy,
    /// Human-provided label (None if unlabeled)
    pub label: Option<InferredOwnership>,
    /// Timestamp when queued
    pub queued_at: u64,
    /// Timestamp when labeled (None if unlabeled)
    pub labeled_at: Option<u64>,
}

impl UncertainSample {
    /// Create a new uncertain sample.
    pub fn new(
        id: u64,
        variable: impl Into<String>,
        features: OwnershipFeatures,
        prediction: OwnershipPrediction,
        uncertainty_score: f64,
        strategy: SelectionStrategy,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id,
            variable: variable.into(),
            source_file: None,
            source_line: None,
            features,
            prediction,
            uncertainty_score: uncertainty_score.clamp(0.0, 1.0),
            strategy,
            label: None,
            queued_at: now,
            labeled_at: None,
        }
    }

    /// Set source location.
    pub fn with_source(mut self, file: impl Into<String>, line: u32) -> Self {
        self.source_file = Some(file.into());
        self.source_line = Some(line);
        self
    }

    /// Check if sample is labeled.
    pub fn is_labeled(&self) -> bool {
        self.label.is_some()
    }

    /// Apply a label.
    pub fn apply_label(&mut self, label: InferredOwnership) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        self.label = Some(label);
        self.labeled_at = Some(now);
    }

    /// Check if prediction matches label (for accuracy calculation).
    pub fn prediction_correct(&self) -> Option<bool> {
        self.label.map(|l| l == self.prediction.kind)
    }
}

/// Uncertainty calculator for predictions.
#[derive(Debug, Clone)]
pub struct UncertaintyCalculator {
    /// Confidence threshold below which samples are considered uncertain
    pub(crate) confidence_threshold: f64,
}

impl Default for UncertaintyCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl UncertaintyCalculator {
    /// Create with default thresholds.
    pub fn new() -> Self {
        Self {
            confidence_threshold: 0.65,
        }
    }

    /// Create with custom confidence threshold.
    pub fn with_confidence_threshold(threshold: f64) -> Self {
        Self {
            confidence_threshold: threshold.clamp(0.0, 1.0),
        }
    }

    /// Calculate uncertainty score using specified strategy.
    pub fn calculate(&self, prediction: &OwnershipPrediction, strategy: SelectionStrategy) -> f64 {
        match strategy {
            SelectionStrategy::UncertaintySampling => self.uncertainty_sampling(prediction),
            SelectionStrategy::MarginSampling => self.margin_sampling(prediction),
            SelectionStrategy::EntropySampling => self.entropy_sampling(prediction),
            SelectionStrategy::Random => rand_like_score(prediction),
        }
    }

    /// Uncertainty = 1 - confidence
    fn uncertainty_sampling(&self, prediction: &OwnershipPrediction) -> f64 {
        1.0 - (prediction.confidence as f64)
    }

    /// Margin between top prediction and fallback (if available)
    fn margin_sampling(&self, prediction: &OwnershipPrediction) -> f64 {
        // If fallback exists, use margin; otherwise use uncertainty
        if prediction.fallback.is_some() {
            // Assume fallback has slightly lower confidence
            let primary_conf = prediction.confidence as f64;
            let secondary_conf = primary_conf * 0.8; // Approximation
            let margin = primary_conf - secondary_conf;
            // Lower margin = higher uncertainty
            1.0 - margin.min(1.0)
        } else {
            self.uncertainty_sampling(prediction)
        }
    }

    /// Entropy-based uncertainty (simplified binary entropy)
    fn entropy_sampling(&self, prediction: &OwnershipPrediction) -> f64 {
        let p = prediction.confidence as f64;
        if p <= 0.0 || p >= 1.0 {
            return 0.0;
        }

        // Binary entropy: -p*log2(p) - (1-p)*log2(1-p)
        let entropy = -p * p.log2() - (1.0 - p) * (1.0 - p).log2();
        // Normalize to 0-1 (max entropy is 1.0 at p=0.5)
        entropy.min(1.0)
    }

    /// Check if prediction is uncertain.
    pub fn is_uncertain(&self, prediction: &OwnershipPrediction) -> bool {
        (prediction.confidence as f64) < self.confidence_threshold
    }
}

/// Simple deterministic "random" based on prediction properties.
fn rand_like_score(prediction: &OwnershipPrediction) -> f64 {
    // Use confidence and kind to generate pseudo-random score
    let kind_hash = match prediction.kind {
        InferredOwnership::Owned => 0.1,
        InferredOwnership::Borrowed => 0.2,
        InferredOwnership::BorrowedMut => 0.3,
        InferredOwnership::Shared => 0.4,
        InferredOwnership::RawPointer => 0.5,
        InferredOwnership::Vec => 0.6,
        InferredOwnership::Slice => 0.7,
        InferredOwnership::SliceMut => 0.8,
    };
    let conf_part = prediction.confidence as f64 * 0.3;
    ((kind_hash + conf_part) * 7.0) % 1.0
}

/// Active learning sample queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleQueue {
    /// Queued samples awaiting labeling
    pending: VecDeque<UncertainSample>,
    /// Labeled samples ready for training
    labeled: Vec<UncertainSample>,
    /// Selection strategy
    strategy: SelectionStrategy,
    /// Maximum pending queue size
    max_pending: usize,
    /// Next sample ID
    next_id: u64,
    /// Total samples processed
    total_processed: u64,
}

impl Default for SampleQueue {
    fn default() -> Self {
        Self::new(SelectionStrategy::UncertaintySampling)
    }
}

impl SampleQueue {
    /// Create a new sample queue.
    pub fn new(strategy: SelectionStrategy) -> Self {
        Self {
            pending: VecDeque::new(),
            labeled: Vec::new(),
            strategy,
            max_pending: 1000,
            next_id: 1,
            total_processed: 0,
        }
    }

    /// Set maximum pending queue size.
    pub fn with_max_pending(mut self, max: usize) -> Self {
        self.max_pending = max;
        self
    }

    /// Get selection strategy.
    pub fn strategy(&self) -> SelectionStrategy {
        self.strategy
    }

    /// Get pending count.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Get labeled count.
    pub fn labeled_count(&self) -> usize {
        self.labeled.len()
    }

    /// Get total processed.
    pub fn total_processed(&self) -> u64 {
        self.total_processed
    }

    /// Add an uncertain sample to the queue.
    pub fn enqueue(&mut self, mut sample: UncertainSample) -> u64 {
        sample.id = self.next_id;
        self.next_id += 1;
        self.total_processed += 1;

        // Maintain priority order (highest uncertainty first)
        let insert_pos = self
            .pending
            .iter()
            .position(|s| s.uncertainty_score < sample.uncertainty_score)
            .unwrap_or(self.pending.len());

        if insert_pos < self.max_pending {
            self.pending.insert(insert_pos, sample);

            // Remove lowest priority if over capacity
            if self.pending.len() > self.max_pending {
                self.pending.pop_back();
            }
        }

        self.next_id - 1
    }

    /// Get next sample for labeling (highest uncertainty).
    pub fn next_for_labeling(&mut self) -> Option<UncertainSample> {
        self.pending.pop_front()
    }

    /// Peek at next sample without removing.
    pub fn peek_next(&self) -> Option<&UncertainSample> {
        self.pending.front()
    }

    /// Get top N samples for batch labeling.
    pub fn batch_for_labeling(&mut self, n: usize) -> Vec<UncertainSample> {
        let mut batch = Vec::with_capacity(n);
        for _ in 0..n {
            if let Some(sample) = self.pending.pop_front() {
                batch.push(sample);
            } else {
                break;
            }
        }
        batch
    }

    /// Submit a labeled sample.
    pub fn submit_labeled(&mut self, sample: UncertainSample) {
        if sample.is_labeled() {
            self.labeled.push(sample);
        }
    }

    /// Get all labeled samples for training.
    pub fn get_labeled_samples(&self) -> &[UncertainSample] {
        &self.labeled
    }

    /// Take labeled samples (moves them out).
    pub fn take_labeled_samples(&mut self) -> Vec<UncertainSample> {
        std::mem::take(&mut self.labeled)
    }

    /// Clear all pending samples.
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }

    /// Get statistics.
    pub fn stats(&self) -> QueueStats {
        let labeled_correct = self
            .labeled
            .iter()
            .filter_map(|s| s.prediction_correct())
            .filter(|&c| c)
            .count();
        let labeled_total = self
            .labeled
            .iter()
            .filter(|s| s.prediction_correct().is_some())
            .count();

        QueueStats {
            pending: self.pending.len(),
            labeled: self.labeled.len(),
            total_processed: self.total_processed,
            avg_uncertainty: if self.pending.is_empty() {
                0.0
            } else {
                self.pending
                    .iter()
                    .map(|s| s.uncertainty_score)
                    .sum::<f64>()
                    / self.pending.len() as f64
            },
            prediction_accuracy: if labeled_total > 0 {
                labeled_correct as f64 / labeled_total as f64
            } else {
                0.0
            },
        }
    }
}

/// Statistics for the sample queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    /// Pending samples count
    pub pending: usize,
    /// Labeled samples count
    pub labeled: usize,
    /// Total samples processed
    pub total_processed: u64,
    /// Average uncertainty of pending samples
    pub avg_uncertainty: f64,
    /// Accuracy of predictions on labeled samples
    pub prediction_accuracy: f64,
}

/// Active learning manager.
#[derive(Debug)]
pub struct ActiveLearner {
    /// Uncertainty calculator
    calculator: UncertaintyCalculator,
    /// Sample queue
    queue: SampleQueue,
    /// Minimum uncertainty to queue
    min_uncertainty: f64,
}

impl Default for ActiveLearner {
    fn default() -> Self {
        Self::new()
    }
}

impl ActiveLearner {
    /// Create a new active learner.
    pub fn new() -> Self {
        Self {
            calculator: UncertaintyCalculator::new(),
            queue: SampleQueue::new(SelectionStrategy::UncertaintySampling),
            min_uncertainty: 0.35,
        }
    }

    /// Create with custom strategy.
    pub fn with_strategy(strategy: SelectionStrategy) -> Self {
        Self {
            calculator: UncertaintyCalculator::new(),
            queue: SampleQueue::new(strategy),
            min_uncertainty: 0.35,
        }
    }

    /// Set minimum uncertainty threshold.
    pub fn with_min_uncertainty(mut self, threshold: f64) -> Self {
        self.min_uncertainty = threshold.clamp(0.0, 1.0);
        self
    }

    /// Set confidence threshold.
    pub fn with_confidence_threshold(mut self, threshold: f64) -> Self {
        self.calculator = UncertaintyCalculator::with_confidence_threshold(threshold);
        self
    }

    /// Process a prediction and optionally queue for labeling.
    ///
    /// Returns the uncertainty score and whether it was queued.
    pub fn process_prediction(
        &mut self,
        variable: impl Into<String>,
        features: OwnershipFeatures,
        prediction: OwnershipPrediction,
    ) -> (f64, bool) {
        let strategy = self.queue.strategy();
        let uncertainty = self.calculator.calculate(&prediction, strategy);

        let queued = if uncertainty >= self.min_uncertainty {
            let sample =
                UncertainSample::new(0, variable, features, prediction, uncertainty, strategy);
            self.queue.enqueue(sample);
            true
        } else {
            false
        };

        (uncertainty, queued)
    }

    /// Get next sample for labeling.
    pub fn next_for_labeling(&mut self) -> Option<UncertainSample> {
        self.queue.next_for_labeling()
    }

    /// Get batch for labeling.
    pub fn batch_for_labeling(&mut self, n: usize) -> Vec<UncertainSample> {
        self.queue.batch_for_labeling(n)
    }

    /// Submit a labeled sample.
    pub fn submit_labeled(&mut self, sample: UncertainSample) {
        self.queue.submit_labeled(sample);
    }

    /// Get labeled samples for training.
    pub fn get_training_samples(&self) -> &[UncertainSample] {
        self.queue.get_labeled_samples()
    }

    /// Take labeled samples for training.
    pub fn take_training_samples(&mut self) -> Vec<UncertainSample> {
        self.queue.take_labeled_samples()
    }

    /// Get queue statistics.
    pub fn stats(&self) -> QueueStats {
        self.queue.stats()
    }

    /// Check if prediction is uncertain.
    pub fn is_uncertain(&self, prediction: &OwnershipPrediction) -> bool {
        self.calculator.is_uncertain(prediction)
    }

    /// Generate markdown report.
    pub fn to_markdown(&self) -> String {
        let stats = self.stats();

        format!(
            r#"## Active Learning Report

### Queue Status

| Metric | Value |
|--------|-------|
| Strategy | {} |
| Pending Samples | {} |
| Labeled Samples | {} |
| Total Processed | {} |
| Avg Uncertainty | {:.2} |
| Prediction Accuracy | {:.1}% |

### Configuration

| Parameter | Value |
|-----------|-------|
| Min Uncertainty | {:.2} |
| Confidence Threshold | {:.2} |
"#,
            self.queue.strategy(),
            stats.pending,
            stats.labeled,
            stats.total_processed,
            stats.avg_uncertainty,
            stats.prediction_accuracy * 100.0,
            self.min_uncertainty,
            self.calculator.confidence_threshold,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_prediction(kind: InferredOwnership, confidence: f32) -> OwnershipPrediction {
        OwnershipPrediction {
            kind,
            confidence,
            fallback: None,
        }
    }

    // ========================================================================
    // SelectionStrategy tests
    // ========================================================================

    #[test]
    fn selection_strategy_display() {
        assert_eq!(
            SelectionStrategy::UncertaintySampling.to_string(),
            "uncertainty"
        );
        assert_eq!(SelectionStrategy::MarginSampling.to_string(), "margin");
        assert_eq!(SelectionStrategy::EntropySampling.to_string(), "entropy");
        assert_eq!(SelectionStrategy::Random.to_string(), "random");
    }

    // ========================================================================
    // UncertaintyCalculator tests
    // ========================================================================

    #[test]
    fn uncertainty_calculator_default() {
        let calc = UncertaintyCalculator::new();
        assert!((calc.confidence_threshold - 0.65).abs() < 0.001);
    }

    #[test]
    fn uncertainty_sampling_high_confidence() {
        let calc = UncertaintyCalculator::new();
        let pred = make_prediction(InferredOwnership::Owned, 0.95);

        let score = calc.calculate(&pred, SelectionStrategy::UncertaintySampling);
        assert!((score - 0.05).abs() < 0.001); // 1 - 0.95 = 0.05
    }

    #[test]
    fn uncertainty_sampling_low_confidence() {
        let calc = UncertaintyCalculator::new();
        let pred = make_prediction(InferredOwnership::Owned, 0.3);

        let score = calc.calculate(&pred, SelectionStrategy::UncertaintySampling);
        assert!((score - 0.7).abs() < 0.001); // 1 - 0.3 = 0.7
    }

    #[test]
    fn entropy_sampling_mid_confidence() {
        let calc = UncertaintyCalculator::new();
        let pred = make_prediction(InferredOwnership::Owned, 0.5);

        let score = calc.calculate(&pred, SelectionStrategy::EntropySampling);
        // At p=0.5, entropy is maximum (1.0)
        assert!((score - 1.0).abs() < 0.001);
    }

    #[test]
    fn entropy_sampling_high_confidence() {
        let calc = UncertaintyCalculator::new();
        let pred = make_prediction(InferredOwnership::Owned, 0.95);

        let score = calc.calculate(&pred, SelectionStrategy::EntropySampling);
        // High confidence = low entropy
        assert!(score < 0.5);
    }

    #[test]
    fn is_uncertain_below_threshold() {
        let calc = UncertaintyCalculator::new(); // threshold = 0.65
        let pred = make_prediction(InferredOwnership::Owned, 0.5);
        assert!(calc.is_uncertain(&pred));
    }

    #[test]
    fn is_uncertain_above_threshold() {
        let calc = UncertaintyCalculator::new();
        let pred = make_prediction(InferredOwnership::Owned, 0.8);
        assert!(!calc.is_uncertain(&pred));
    }

    // ========================================================================
    // UncertainSample tests
    // ========================================================================

    #[test]
    fn uncertain_sample_new() {
        let features = OwnershipFeatures::default();
        let pred = make_prediction(InferredOwnership::Borrowed, 0.4);
        let sample = UncertainSample::new(
            1,
            "ptr",
            features,
            pred,
            0.6,
            SelectionStrategy::UncertaintySampling,
        );

        assert_eq!(sample.id, 1);
        assert_eq!(sample.variable, "ptr");
        assert!((sample.uncertainty_score - 0.6).abs() < 0.001);
        assert!(!sample.is_labeled());
    }

    #[test]
    fn uncertain_sample_apply_label() {
        let features = OwnershipFeatures::default();
        let pred = make_prediction(InferredOwnership::Borrowed, 0.4);
        let mut sample = UncertainSample::new(
            1,
            "ptr",
            features,
            pred,
            0.6,
            SelectionStrategy::UncertaintySampling,
        );

        sample.apply_label(InferredOwnership::Owned);

        assert!(sample.is_labeled());
        assert_eq!(sample.label, Some(InferredOwnership::Owned));
        assert!(sample.labeled_at.is_some());
    }

    #[test]
    fn uncertain_sample_prediction_correct() {
        let features = OwnershipFeatures::default();
        let pred = make_prediction(InferredOwnership::Borrowed, 0.4);
        let mut sample = UncertainSample::new(
            1,
            "ptr",
            features,
            pred,
            0.6,
            SelectionStrategy::UncertaintySampling,
        );

        // Before labeling
        assert!(sample.prediction_correct().is_none());

        // Label matches prediction
        sample.apply_label(InferredOwnership::Borrowed);
        assert_eq!(sample.prediction_correct(), Some(true));
    }

    #[test]
    fn uncertain_sample_prediction_incorrect() {
        let features = OwnershipFeatures::default();
        let pred = make_prediction(InferredOwnership::Borrowed, 0.4);
        let mut sample = UncertainSample::new(
            1,
            "ptr",
            features,
            pred,
            0.6,
            SelectionStrategy::UncertaintySampling,
        );

        // Label doesn't match prediction
        sample.apply_label(InferredOwnership::Owned);
        assert_eq!(sample.prediction_correct(), Some(false));
    }

    // ========================================================================
    // SampleQueue tests
    // ========================================================================

    #[test]
    fn sample_queue_new() {
        let queue = SampleQueue::new(SelectionStrategy::UncertaintySampling);
        assert_eq!(queue.pending_count(), 0);
        assert_eq!(queue.labeled_count(), 0);
    }

    #[test]
    fn sample_queue_enqueue() {
        let mut queue = SampleQueue::new(SelectionStrategy::UncertaintySampling);

        let sample = UncertainSample::new(
            0,
            "ptr",
            OwnershipFeatures::default(),
            make_prediction(InferredOwnership::Borrowed, 0.4),
            0.6,
            SelectionStrategy::UncertaintySampling,
        );

        let id = queue.enqueue(sample);
        assert_eq!(id, 1);
        assert_eq!(queue.pending_count(), 1);
    }

    #[test]
    fn sample_queue_priority_order() {
        let mut queue = SampleQueue::new(SelectionStrategy::UncertaintySampling);

        // Enqueue in wrong order
        for uncertainty in [0.3, 0.9, 0.5] {
            let sample = UncertainSample::new(
                0,
                "ptr",
                OwnershipFeatures::default(),
                make_prediction(InferredOwnership::Borrowed, 0.4),
                uncertainty,
                SelectionStrategy::UncertaintySampling,
            );
            queue.enqueue(sample);
        }

        // Should get highest uncertainty first
        let first = queue.next_for_labeling().unwrap();
        assert!((first.uncertainty_score - 0.9).abs() < 0.001);

        let second = queue.next_for_labeling().unwrap();
        assert!((second.uncertainty_score - 0.5).abs() < 0.001);
    }

    #[test]
    fn sample_queue_max_pending() {
        let mut queue =
            SampleQueue::new(SelectionStrategy::UncertaintySampling).with_max_pending(3);

        // Enqueue 5 samples
        for i in 0..5 {
            let sample = UncertainSample::new(
                0,
                format!("ptr{}", i),
                OwnershipFeatures::default(),
                make_prediction(InferredOwnership::Borrowed, 0.4),
                (i as f64 + 1.0) * 0.1,
                SelectionStrategy::UncertaintySampling,
            );
            queue.enqueue(sample);
        }

        // Should only have top 3 by uncertainty
        assert_eq!(queue.pending_count(), 3);
    }

    #[test]
    fn sample_queue_batch_labeling() {
        let mut queue = SampleQueue::new(SelectionStrategy::UncertaintySampling);

        for i in 0..5 {
            let sample = UncertainSample::new(
                0,
                format!("ptr{}", i),
                OwnershipFeatures::default(),
                make_prediction(InferredOwnership::Borrowed, 0.4),
                0.5,
                SelectionStrategy::UncertaintySampling,
            );
            queue.enqueue(sample);
        }

        let batch = queue.batch_for_labeling(3);
        assert_eq!(batch.len(), 3);
        assert_eq!(queue.pending_count(), 2);
    }

    #[test]
    fn sample_queue_submit_labeled() {
        let mut queue = SampleQueue::new(SelectionStrategy::UncertaintySampling);

        let mut sample = UncertainSample::new(
            1,
            "ptr",
            OwnershipFeatures::default(),
            make_prediction(InferredOwnership::Borrowed, 0.4),
            0.6,
            SelectionStrategy::UncertaintySampling,
        );
        sample.apply_label(InferredOwnership::Owned);

        queue.submit_labeled(sample);
        assert_eq!(queue.labeled_count(), 1);
    }

    #[test]
    fn sample_queue_stats() {
        let mut queue = SampleQueue::new(SelectionStrategy::UncertaintySampling);

        // Add pending
        let sample1 = UncertainSample::new(
            0,
            "ptr1",
            OwnershipFeatures::default(),
            make_prediction(InferredOwnership::Borrowed, 0.4),
            0.6,
            SelectionStrategy::UncertaintySampling,
        );
        queue.enqueue(sample1);

        // Add labeled (correct prediction)
        let mut sample2 = UncertainSample::new(
            0,
            "ptr2",
            OwnershipFeatures::default(),
            make_prediction(InferredOwnership::Borrowed, 0.4),
            0.5,
            SelectionStrategy::UncertaintySampling,
        );
        sample2.apply_label(InferredOwnership::Borrowed);
        queue.submit_labeled(sample2);

        let stats = queue.stats();
        assert_eq!(stats.pending, 1);
        assert_eq!(stats.labeled, 1);
        assert!((stats.prediction_accuracy - 1.0).abs() < 0.001); // 100% accurate
    }

    // ========================================================================
    // ActiveLearner tests
    // ========================================================================

    #[test]
    fn active_learner_new() {
        let learner = ActiveLearner::new();
        let stats = learner.stats();
        assert_eq!(stats.pending, 0);
        assert_eq!(stats.labeled, 0);
    }

    #[test]
    fn active_learner_process_uncertain() {
        let mut learner = ActiveLearner::new().with_min_uncertainty(0.3);

        let features = OwnershipFeatures::default();
        let pred = make_prediction(InferredOwnership::Borrowed, 0.4); // uncertainty = 0.6

        let (uncertainty, queued) = learner.process_prediction("ptr", features, pred);

        assert!((uncertainty - 0.6).abs() < 0.001);
        assert!(queued);
        assert_eq!(learner.stats().pending, 1);
    }

    #[test]
    fn active_learner_process_certain() {
        let mut learner = ActiveLearner::new().with_min_uncertainty(0.3);

        let features = OwnershipFeatures::default();
        let pred = make_prediction(InferredOwnership::Owned, 0.95); // uncertainty = 0.05

        let (uncertainty, queued) = learner.process_prediction("ptr", features, pred);

        assert!((uncertainty - 0.05).abs() < 0.001);
        assert!(!queued); // Not uncertain enough
        assert_eq!(learner.stats().pending, 0);
    }

    #[test]
    fn active_learner_labeling_workflow() {
        let mut learner = ActiveLearner::new().with_min_uncertainty(0.3);

        // Process uncertain prediction
        let features = OwnershipFeatures::default();
        let pred = make_prediction(InferredOwnership::Borrowed, 0.4);
        learner.process_prediction("ptr", features, pred);

        // Get for labeling
        let mut sample = learner.next_for_labeling().unwrap();
        assert_eq!(sample.variable, "ptr");

        // Apply label
        sample.apply_label(InferredOwnership::Owned);
        learner.submit_labeled(sample);

        // Check training samples
        let training = learner.get_training_samples();
        assert_eq!(training.len(), 1);
        assert_eq!(training[0].label, Some(InferredOwnership::Owned));
    }

    #[test]
    fn active_learner_to_markdown() {
        let learner = ActiveLearner::new();
        let md = learner.to_markdown();

        assert!(md.contains("Active Learning Report"));
        assert!(md.contains("Queue Status"));
        assert!(md.contains("Strategy"));
    }

    #[test]
    fn active_learner_with_strategy() {
        let learner = ActiveLearner::with_strategy(SelectionStrategy::EntropySampling);
        assert_eq!(learner.queue.strategy(), SelectionStrategy::EntropySampling);
    }

    #[test]
    fn active_learner_batch_labeling() {
        let mut learner = ActiveLearner::new().with_min_uncertainty(0.2);

        // Process multiple uncertain predictions
        for i in 0..5 {
            let features = OwnershipFeatures::default();
            let pred = make_prediction(InferredOwnership::Borrowed, 0.3 + (i as f32 * 0.05));
            learner.process_prediction(format!("ptr{}", i), features, pred);
        }

        let batch = learner.batch_for_labeling(3);
        assert_eq!(batch.len(), 3);
    }

    // ========================================================================
    // Additional coverage: margin sampling with fallback
    // ========================================================================

    #[test]
    fn margin_sampling_with_fallback() {
        let calc = UncertaintyCalculator::new();
        let pred = OwnershipPrediction {
            kind: InferredOwnership::Borrowed,
            confidence: 0.6,
            fallback: Some(InferredOwnership::Owned),
        };

        let score = calc.calculate(&pred, SelectionStrategy::MarginSampling);
        // With fallback: margin = 0.6 - 0.6*0.8 = 0.12, score = 1 - 0.12 = 0.88
        assert!(score > 0.5);
    }

    #[test]
    fn margin_sampling_without_fallback() {
        let calc = UncertaintyCalculator::new();
        let pred = make_prediction(InferredOwnership::Borrowed, 0.6);

        let score = calc.calculate(&pred, SelectionStrategy::MarginSampling);
        // Without fallback: falls through to uncertainty sampling: 1 - 0.6 = 0.4
        assert!((score - 0.4).abs() < 0.001);
    }

    // ========================================================================
    // Additional coverage: entropy boundary values
    // ========================================================================

    #[test]
    fn entropy_at_zero_confidence() {
        let calc = UncertaintyCalculator::new();
        let pred = make_prediction(InferredOwnership::Owned, 0.0);

        let score = calc.calculate(&pred, SelectionStrategy::EntropySampling);
        assert!((score - 0.0).abs() < 0.001); // p <= 0 returns 0
    }

    #[test]
    fn entropy_at_one_confidence() {
        let calc = UncertaintyCalculator::new();
        let pred = make_prediction(InferredOwnership::Owned, 1.0);

        let score = calc.calculate(&pred, SelectionStrategy::EntropySampling);
        assert!((score - 0.0).abs() < 0.001); // p >= 1 returns 0
    }

    // ========================================================================
    // Additional coverage: random sampling all InferredOwnership variants
    // ========================================================================

    #[test]
    fn random_sampling_all_ownership_kinds() {
        let calc = UncertaintyCalculator::new();
        let kinds = vec![
            InferredOwnership::Owned,
            InferredOwnership::Borrowed,
            InferredOwnership::BorrowedMut,
            InferredOwnership::Shared,
            InferredOwnership::RawPointer,
            InferredOwnership::Vec,
            InferredOwnership::Slice,
            InferredOwnership::SliceMut,
        ];

        for kind in kinds {
            let pred = make_prediction(kind, 0.5);
            let score = calc.calculate(&pred, SelectionStrategy::Random);
            assert!(score >= 0.0 && score <= 1.0);
        }
    }

    // ========================================================================
    // Additional coverage: UncertainSample with_source
    // ========================================================================

    #[test]
    fn uncertain_sample_with_source() {
        let features = OwnershipFeatures::default();
        let pred = make_prediction(InferredOwnership::Borrowed, 0.4);
        let sample = UncertainSample::new(
            1,
            "ptr",
            features,
            pred,
            0.6,
            SelectionStrategy::UncertaintySampling,
        )
        .with_source("test.c", 42);

        assert_eq!(sample.source_file, Some("test.c".to_string()));
        assert_eq!(sample.source_line, Some(42));
    }

    // ========================================================================
    // Additional coverage: SampleQueue edge cases
    // ========================================================================

    #[test]
    fn sample_queue_peek_next() {
        let mut queue = SampleQueue::new(SelectionStrategy::UncertaintySampling);

        assert!(queue.peek_next().is_none());

        let sample = UncertainSample::new(
            0,
            "ptr",
            OwnershipFeatures::default(),
            make_prediction(InferredOwnership::Borrowed, 0.4),
            0.6,
            SelectionStrategy::UncertaintySampling,
        );
        queue.enqueue(sample);

        assert!(queue.peek_next().is_some());
        assert_eq!(queue.pending_count(), 1); // Not consumed
    }

    #[test]
    fn sample_queue_clear_pending() {
        let mut queue = SampleQueue::new(SelectionStrategy::UncertaintySampling);

        for i in 0..3 {
            let sample = UncertainSample::new(
                0,
                format!("ptr{}", i),
                OwnershipFeatures::default(),
                make_prediction(InferredOwnership::Borrowed, 0.4),
                0.5,
                SelectionStrategy::UncertaintySampling,
            );
            queue.enqueue(sample);
        }

        assert_eq!(queue.pending_count(), 3);
        queue.clear_pending();
        assert_eq!(queue.pending_count(), 0);
    }

    #[test]
    fn sample_queue_take_labeled() {
        let mut queue = SampleQueue::new(SelectionStrategy::UncertaintySampling);

        let mut sample = UncertainSample::new(
            1,
            "ptr",
            OwnershipFeatures::default(),
            make_prediction(InferredOwnership::Borrowed, 0.4),
            0.6,
            SelectionStrategy::UncertaintySampling,
        );
        sample.apply_label(InferredOwnership::Owned);
        queue.submit_labeled(sample);

        assert_eq!(queue.labeled_count(), 1);
        let taken = queue.take_labeled_samples();
        assert_eq!(taken.len(), 1);
        assert_eq!(queue.labeled_count(), 0);
    }

    #[test]
    fn sample_queue_submit_unlabeled_rejected() {
        let mut queue = SampleQueue::new(SelectionStrategy::UncertaintySampling);

        let sample = UncertainSample::new(
            1,
            "ptr",
            OwnershipFeatures::default(),
            make_prediction(InferredOwnership::Borrowed, 0.4),
            0.6,
            SelectionStrategy::UncertaintySampling,
        );
        // Don't label it
        queue.submit_labeled(sample);
        assert_eq!(queue.labeled_count(), 0); // Rejected because unlabeled
    }

    #[test]
    fn sample_queue_batch_from_empty() {
        let mut queue = SampleQueue::new(SelectionStrategy::UncertaintySampling);
        let batch = queue.batch_for_labeling(5);
        assert!(batch.is_empty());
    }

    #[test]
    fn sample_queue_batch_partial() {
        let mut queue = SampleQueue::new(SelectionStrategy::UncertaintySampling);

        for i in 0..2 {
            let sample = UncertainSample::new(
                0,
                format!("ptr{}", i),
                OwnershipFeatures::default(),
                make_prediction(InferredOwnership::Borrowed, 0.4),
                0.5,
                SelectionStrategy::UncertaintySampling,
            );
            queue.enqueue(sample);
        }

        let batch = queue.batch_for_labeling(5);
        assert_eq!(batch.len(), 2); // Only 2 available
    }

    #[test]
    fn sample_queue_total_processed() {
        let mut queue = SampleQueue::new(SelectionStrategy::UncertaintySampling);
        assert_eq!(queue.total_processed(), 0);

        for i in 0..3 {
            let sample = UncertainSample::new(
                0,
                format!("ptr{}", i),
                OwnershipFeatures::default(),
                make_prediction(InferredOwnership::Borrowed, 0.4),
                0.5,
                SelectionStrategy::UncertaintySampling,
            );
            queue.enqueue(sample);
        }
        assert_eq!(queue.total_processed(), 3);
    }

    #[test]
    fn sample_queue_get_labeled_samples() {
        let queue = SampleQueue::new(SelectionStrategy::UncertaintySampling);
        assert!(queue.get_labeled_samples().is_empty());
    }

    // ========================================================================
    // Additional coverage: SampleQueue overflow (insert_pos >= max_pending)
    // ========================================================================

    #[test]
    fn sample_queue_overflow_low_priority_dropped() {
        let mut queue =
            SampleQueue::new(SelectionStrategy::UncertaintySampling).with_max_pending(2);

        // Add two high uncertainty samples
        for u in [0.9, 0.8] {
            let sample = UncertainSample::new(
                0,
                "ptr",
                OwnershipFeatures::default(),
                make_prediction(InferredOwnership::Borrowed, 0.4),
                u,
                SelectionStrategy::UncertaintySampling,
            );
            queue.enqueue(sample);
        }

        // Add very low uncertainty sample - should not be inserted
        let low = UncertainSample::new(
            0,
            "low",
            OwnershipFeatures::default(),
            make_prediction(InferredOwnership::Borrowed, 0.4),
            0.01,
            SelectionStrategy::UncertaintySampling,
        );
        queue.enqueue(low);

        assert_eq!(queue.pending_count(), 2);
        // The low priority sample should NOT be in the queue
        let first = queue.peek_next().unwrap();
        assert!((first.uncertainty_score - 0.9).abs() < 0.01);
    }

    // ========================================================================
    // Additional coverage: ActiveLearner with_confidence_threshold
    // ========================================================================

    #[test]
    fn active_learner_with_confidence_threshold() {
        let learner = ActiveLearner::new().with_confidence_threshold(0.8);
        let high = make_prediction(InferredOwnership::Owned, 0.75);
        assert!(learner.is_uncertain(&high)); // Below 0.8

        let very_high = make_prediction(InferredOwnership::Owned, 0.85);
        assert!(!learner.is_uncertain(&very_high)); // Above 0.8
    }

    #[test]
    fn active_learner_take_training_samples() {
        let mut learner = ActiveLearner::new().with_min_uncertainty(0.2);

        let features = OwnershipFeatures::default();
        let pred = make_prediction(InferredOwnership::Borrowed, 0.4);
        learner.process_prediction("ptr", features, pred);

        let mut sample = learner.next_for_labeling().unwrap();
        sample.apply_label(InferredOwnership::Owned);
        learner.submit_labeled(sample);

        let training = learner.take_training_samples();
        assert_eq!(training.len(), 1);
        assert!(learner.get_training_samples().is_empty()); // Moved out
    }

    // ========================================================================
    // Additional coverage: SampleQueue stats with no labeled
    // ========================================================================

    #[test]
    fn sample_queue_stats_empty() {
        let queue = SampleQueue::new(SelectionStrategy::UncertaintySampling);
        let stats = queue.stats();
        assert_eq!(stats.pending, 0);
        assert_eq!(stats.labeled, 0);
        assert_eq!(stats.total_processed, 0);
        assert!((stats.avg_uncertainty - 0.0).abs() < 0.001);
        assert!((stats.prediction_accuracy - 0.0).abs() < 0.001);
    }

    #[test]
    fn sample_queue_stats_with_incorrect_prediction() {
        let mut queue = SampleQueue::new(SelectionStrategy::UncertaintySampling);

        let mut sample = UncertainSample::new(
            0,
            "ptr",
            OwnershipFeatures::default(),
            make_prediction(InferredOwnership::Borrowed, 0.4),
            0.6,
            SelectionStrategy::UncertaintySampling,
        );
        sample.apply_label(InferredOwnership::Owned); // Different from prediction
        queue.submit_labeled(sample);

        let stats = queue.stats();
        assert!((stats.prediction_accuracy - 0.0).abs() < 0.001); // 0% accurate
    }

    // ========================================================================
    // Additional coverage: UncertaintyCalculator custom threshold
    // ========================================================================

    #[test]
    fn uncertainty_calculator_custom_threshold_clamp() {
        let calc = UncertaintyCalculator::with_confidence_threshold(2.0);
        assert!((calc.confidence_threshold - 1.0).abs() < 0.001);

        let calc2 = UncertaintyCalculator::with_confidence_threshold(-1.0);
        assert!((calc2.confidence_threshold - 0.0).abs() < 0.001);
    }

    // ========================================================================
    // Additional coverage: SampleQueue default
    // ========================================================================

    #[test]
    fn sample_queue_default() {
        let queue = SampleQueue::default();
        assert_eq!(queue.strategy(), SelectionStrategy::UncertaintySampling);
    }

    #[test]
    fn active_learner_default() {
        let learner = ActiveLearner::default();
        let stats = learner.stats();
        assert_eq!(stats.pending, 0);
    }

    // ========================================================================
    // Additional coverage: uncertain sample uncertainty clamp
    // ========================================================================

    #[test]
    fn uncertain_sample_clamp_high() {
        let sample = UncertainSample::new(
            1,
            "ptr",
            OwnershipFeatures::default(),
            make_prediction(InferredOwnership::Borrowed, 0.4),
            1.5, // Over 1.0
            SelectionStrategy::UncertaintySampling,
        );
        assert!((sample.uncertainty_score - 1.0).abs() < 0.001);
    }

    #[test]
    fn uncertain_sample_clamp_low() {
        let sample = UncertainSample::new(
            1,
            "ptr",
            OwnershipFeatures::default(),
            make_prediction(InferredOwnership::Borrowed, 0.4),
            -0.5, // Below 0.0
            SelectionStrategy::UncertaintySampling,
        );
        assert!((sample.uncertainty_score - 0.0).abs() < 0.001);
    }

    // ========================================================================
    // Additional coverage: ActiveLearner with_min_uncertainty clamp
    // ========================================================================

    #[test]
    fn active_learner_min_uncertainty_clamp() {
        let learner = ActiveLearner::new().with_min_uncertainty(2.0);
        // Should be clamped to 1.0
        let features = OwnershipFeatures::default();
        let pred = make_prediction(InferredOwnership::Borrowed, 0.01);
        let mut learner2 = learner;
        let (_, queued) = learner2.process_prediction("ptr", features, pred);
        // 1.0 - 0.01 = 0.99 < 1.0 (min_uncertainty), so NOT queued
        assert!(!queued);
    }

    // ========================================================================
    // Additional coverage: process_prediction with MarginSampling strategy
    // ========================================================================

    #[test]
    fn active_learner_process_with_margin_strategy() {
        let mut learner =
            ActiveLearner::with_strategy(SelectionStrategy::MarginSampling).with_min_uncertainty(0.2);

        let features = OwnershipFeatures::default();
        let pred = OwnershipPrediction {
            kind: InferredOwnership::Borrowed,
            confidence: 0.4,
            fallback: Some(InferredOwnership::Owned),
        };
        let (_, queued) = learner.process_prediction("ptr", features, pred);
        assert!(queued);
    }

    #[test]
    fn active_learner_process_with_entropy_strategy() {
        let mut learner =
            ActiveLearner::with_strategy(SelectionStrategy::EntropySampling).with_min_uncertainty(0.2);

        let features = OwnershipFeatures::default();
        let pred = make_prediction(InferredOwnership::Borrowed, 0.5);
        let (uncertainty, queued) = learner.process_prediction("ptr", features, pred);
        assert!((uncertainty - 1.0).abs() < 0.001); // max entropy at 0.5
        assert!(queued);
    }

    #[test]
    fn active_learner_process_with_random_strategy() {
        let mut learner =
            ActiveLearner::with_strategy(SelectionStrategy::Random).with_min_uncertainty(0.0);

        let features = OwnershipFeatures::default();
        let pred = make_prediction(InferredOwnership::Borrowed, 0.5);
        let (uncertainty, queued) = learner.process_prediction("ptr", features, pred);
        assert!(uncertainty >= 0.0 && uncertainty <= 1.0);
        assert!(queued);
    }
}
