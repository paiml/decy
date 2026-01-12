//! Confidence threshold tuning for hybrid classification (DECY-ML-014).
//!
//! Provides utilities for finding the optimal confidence threshold for
//! deciding when to use ML predictions vs. falling back to rules.
//!
//! # Threshold Selection Strategy
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                  THRESHOLD TUNING PROCESS                       │
//! │                                                                 │
//! │  1. Collect validation data with ground truth                   │
//! │     ├─ ML predictions with confidence scores                    │
//! │     └─ Rule-based predictions                                   │
//! │                                                                 │
//! │  2. For each threshold candidate (0.1, 0.2, ..., 0.9):          │
//! │     ├─ If ML_confidence >= threshold: use ML                    │
//! │     └─ Else: use rules (fallback)                               │
//! │                                                                 │
//! │  3. Calculate metrics for each threshold:                       │
//! │     ├─ Accuracy (most important)                                │
//! │     ├─ F1 score (precision-recall balance)                      │
//! │     └─ Fallback rate (operational cost)                         │
//! │                                                                 │
//! │  4. Select optimal threshold based on criteria:                 │
//! │     ├─ Maximize accuracy                                        │
//! │     ├─ Or maximize F1                                           │
//! │     └─ Or balance accuracy vs fallback rate                     │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use crate::ml_features::InferredOwnership;
use serde::{Deserialize, Serialize};

/// A single validation sample for threshold tuning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSample {
    /// Ground truth ownership
    pub ground_truth: InferredOwnership,
    /// Rule-based prediction
    pub rule_prediction: InferredOwnership,
    /// ML prediction
    pub ml_prediction: InferredOwnership,
    /// ML confidence score (0.0 - 1.0)
    pub ml_confidence: f64,
}

impl ValidationSample {
    /// Create a new validation sample.
    pub fn new(
        ground_truth: InferredOwnership,
        rule_prediction: InferredOwnership,
        ml_prediction: InferredOwnership,
        ml_confidence: f64,
    ) -> Self {
        Self {
            ground_truth,
            rule_prediction,
            ml_prediction,
            ml_confidence: ml_confidence.clamp(0.0, 1.0),
        }
    }

    /// Check if rule prediction is correct.
    pub fn rule_correct(&self) -> bool {
        self.rule_prediction == self.ground_truth
    }

    /// Check if ML prediction is correct.
    pub fn ml_correct(&self) -> bool {
        self.ml_prediction == self.ground_truth
    }

    /// Get the hybrid prediction at a given threshold.
    pub fn hybrid_prediction(&self, threshold: f64) -> InferredOwnership {
        if self.ml_confidence >= threshold {
            self.ml_prediction
        } else {
            self.rule_prediction
        }
    }

    /// Check if hybrid prediction is correct at given threshold.
    pub fn hybrid_correct(&self, threshold: f64) -> bool {
        self.hybrid_prediction(threshold) == self.ground_truth
    }
}

/// Metrics at a specific threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdMetrics {
    /// Confidence threshold
    pub threshold: f64,
    /// Number of samples
    pub sample_count: usize,
    /// Accuracy (correct / total)
    pub accuracy: f64,
    /// Precision (true positives / predicted positives)
    pub precision: f64,
    /// Recall (true positives / actual positives)
    pub recall: f64,
    /// F1 score (harmonic mean of precision and recall)
    pub f1_score: f64,
    /// Fallback rate (samples using rules / total)
    pub fallback_rate: f64,
    /// ML usage rate (samples using ML / total)
    pub ml_usage_rate: f64,
}

impl ThresholdMetrics {
    /// Calculate metrics at a given threshold.
    pub fn calculate(samples: &[ValidationSample], threshold: f64) -> Self {
        if samples.is_empty() {
            return Self {
                threshold,
                sample_count: 0,
                accuracy: 0.0,
                precision: 0.0,
                recall: 0.0,
                f1_score: 0.0,
                fallback_rate: 1.0,
                ml_usage_rate: 0.0,
            };
        }

        let sample_count = samples.len();
        let mut correct = 0;
        let mut using_ml = 0;
        let mut using_rules = 0;

        // For multi-class, we'll use micro-averaged metrics
        let mut true_positives = 0;
        let mut false_positives = 0;
        let mut false_negatives = 0;

        for sample in samples {
            let prediction = sample.hybrid_prediction(threshold);
            let is_correct = prediction == sample.ground_truth;

            if is_correct {
                correct += 1;
                true_positives += 1;
            } else {
                // For micro-averaging: count as FP and FN
                false_positives += 1;
                false_negatives += 1;
            }

            if sample.ml_confidence >= threshold {
                using_ml += 1;
            } else {
                using_rules += 1;
            }
        }

        let accuracy = correct as f64 / sample_count as f64;
        let fallback_rate = using_rules as f64 / sample_count as f64;
        let ml_usage_rate = using_ml as f64 / sample_count as f64;

        // Micro-averaged precision and recall
        let precision = if true_positives + false_positives > 0 {
            true_positives as f64 / (true_positives + false_positives) as f64
        } else {
            0.0
        };

        let recall = if true_positives + false_negatives > 0 {
            true_positives as f64 / (true_positives + false_negatives) as f64
        } else {
            0.0
        };

        let f1_score = if precision + recall > 0.0 {
            2.0 * precision * recall / (precision + recall)
        } else {
            0.0
        };

        Self {
            threshold,
            sample_count,
            accuracy,
            precision,
            recall,
            f1_score,
            fallback_rate,
            ml_usage_rate,
        }
    }
}

/// Criteria for selecting the optimal threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionCriteria {
    /// Maximize accuracy
    MaxAccuracy,
    /// Maximize F1 score
    MaxF1,
    /// Balance accuracy and low fallback rate
    /// (weighted: 0.7 * accuracy + 0.3 * (1 - fallback_rate))
    BalancedAccuracyFallback,
    /// Minimize fallback while maintaining accuracy above baseline
    MinFallbackAboveBaseline,
}

impl std::fmt::Display for SelectionCriteria {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectionCriteria::MaxAccuracy => write!(f, "max-accuracy"),
            SelectionCriteria::MaxF1 => write!(f, "max-f1"),
            SelectionCriteria::BalancedAccuracyFallback => write!(f, "balanced"),
            SelectionCriteria::MinFallbackAboveBaseline => write!(f, "min-fallback"),
        }
    }
}

/// Result of threshold tuning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningResult {
    /// Optimal threshold
    pub optimal_threshold: f64,
    /// Selection criteria used
    pub criteria: String,
    /// Metrics at optimal threshold
    pub optimal_metrics: ThresholdMetrics,
    /// Metrics at all candidate thresholds
    pub all_thresholds: Vec<ThresholdMetrics>,
    /// Rule-only baseline accuracy
    pub baseline_accuracy: f64,
    /// ML-only accuracy (threshold = 0)
    pub ml_only_accuracy: f64,
    /// Improvement over baseline
    pub improvement_over_baseline: f64,
}

impl TuningResult {
    /// Generate markdown report.
    pub fn to_markdown(&self) -> String {
        let mut threshold_table = String::from(
            "| Threshold | Accuracy | F1 | Fallback Rate | ML Usage |\n|-----------|----------|----|--------------|---------|\n",
        );

        for m in &self.all_thresholds {
            threshold_table.push_str(&format!(
                "| {:.2} | {:.1}% | {:.3} | {:.1}% | {:.1}% |\n",
                m.threshold,
                m.accuracy * 100.0,
                m.f1_score,
                m.fallback_rate * 100.0,
                m.ml_usage_rate * 100.0,
            ));
        }

        format!(
            r#"## Threshold Tuning Report

### Optimal Configuration

| Parameter | Value |
|-----------|-------|
| **Optimal Threshold** | {:.2} |
| **Selection Criteria** | {} |
| **Accuracy** | {:.1}% |
| **F1 Score** | {:.3} |
| **Fallback Rate** | {:.1}% |

### Comparison to Baselines

| Method | Accuracy |
|--------|----------|
| Rules Only (baseline) | {:.1}% |
| ML Only (threshold=0) | {:.1}% |
| **Hybrid (optimal)** | **{:.1}%** |
| Improvement | {:+.1}% |

### All Thresholds

{}

### Recommendation

{}
"#,
            self.optimal_threshold,
            self.criteria,
            self.optimal_metrics.accuracy * 100.0,
            self.optimal_metrics.f1_score,
            self.optimal_metrics.fallback_rate * 100.0,
            self.baseline_accuracy * 100.0,
            self.ml_only_accuracy * 100.0,
            self.optimal_metrics.accuracy * 100.0,
            self.improvement_over_baseline * 100.0,
            threshold_table,
            if self.improvement_over_baseline > 0.0 {
                format!(
                    "✅ **ADOPT HYBRID**: {:.1}% accuracy improvement at threshold {:.2}",
                    self.improvement_over_baseline * 100.0,
                    self.optimal_threshold
                )
            } else {
                "❌ **KEEP RULES ONLY**: No improvement from ML enhancement".to_string()
            }
        )
    }
}

/// Threshold tuner for finding optimal confidence threshold.
#[derive(Debug, Clone)]
pub struct ThresholdTuner {
    /// Candidate thresholds to evaluate
    candidates: Vec<f64>,
    /// Selection criteria
    criteria: SelectionCriteria,
}

impl Default for ThresholdTuner {
    fn default() -> Self {
        Self::new()
    }
}

impl ThresholdTuner {
    /// Create tuner with default candidates (0.1 to 0.9 by 0.1).
    pub fn new() -> Self {
        Self {
            candidates: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.65, 0.7, 0.8, 0.9],
            criteria: SelectionCriteria::MaxAccuracy,
        }
    }

    /// Create tuner with custom candidate thresholds.
    pub fn with_candidates(candidates: Vec<f64>) -> Self {
        Self {
            candidates: candidates.into_iter().map(|t| t.clamp(0.0, 1.0)).collect(),
            criteria: SelectionCriteria::MaxAccuracy,
        }
    }

    /// Set selection criteria.
    pub fn with_criteria(mut self, criteria: SelectionCriteria) -> Self {
        self.criteria = criteria;
        self
    }

    /// Add a candidate threshold.
    pub fn add_candidate(&mut self, threshold: f64) {
        let t = threshold.clamp(0.0, 1.0);
        if !self.candidates.contains(&t) {
            self.candidates.push(t);
            self.candidates.sort_by(|a, b| a.partial_cmp(b).unwrap());
        }
    }

    /// Tune threshold using validation samples.
    pub fn tune(&self, samples: &[ValidationSample]) -> TuningResult {
        if samples.is_empty() {
            return TuningResult {
                optimal_threshold: 0.65,
                criteria: self.criteria.to_string(),
                optimal_metrics: ThresholdMetrics::calculate(&[], 0.65),
                all_thresholds: vec![],
                baseline_accuracy: 0.0,
                ml_only_accuracy: 0.0,
                improvement_over_baseline: 0.0,
            };
        }

        // Calculate baseline (rules only) accuracy
        let baseline_correct = samples.iter().filter(|s| s.rule_correct()).count();
        let baseline_accuracy = baseline_correct as f64 / samples.len() as f64;

        // Calculate ML-only accuracy (threshold = 0)
        let ml_only_correct = samples.iter().filter(|s| s.ml_correct()).count();
        let ml_only_accuracy = ml_only_correct as f64 / samples.len() as f64;

        // Calculate metrics at each threshold
        let all_thresholds: Vec<ThresholdMetrics> = self
            .candidates
            .iter()
            .map(|&t| ThresholdMetrics::calculate(samples, t))
            .collect();

        // Find optimal threshold based on criteria
        let optimal = self.select_optimal(&all_thresholds, baseline_accuracy);

        let improvement = optimal.accuracy - baseline_accuracy;

        TuningResult {
            optimal_threshold: optimal.threshold,
            criteria: self.criteria.to_string(),
            optimal_metrics: optimal.clone(),
            all_thresholds,
            baseline_accuracy,
            ml_only_accuracy,
            improvement_over_baseline: improvement,
        }
    }

    fn select_optimal(
        &self,
        metrics: &[ThresholdMetrics],
        baseline_accuracy: f64,
    ) -> ThresholdMetrics {
        if metrics.is_empty() {
            return ThresholdMetrics {
                threshold: 0.65,
                sample_count: 0,
                accuracy: 0.0,
                precision: 0.0,
                recall: 0.0,
                f1_score: 0.0,
                fallback_rate: 1.0,
                ml_usage_rate: 0.0,
            };
        }

        match self.criteria {
            SelectionCriteria::MaxAccuracy => metrics
                .iter()
                .max_by(|a, b| a.accuracy.partial_cmp(&b.accuracy).unwrap())
                .cloned()
                .unwrap(),

            SelectionCriteria::MaxF1 => metrics
                .iter()
                .max_by(|a, b| a.f1_score.partial_cmp(&b.f1_score).unwrap())
                .cloned()
                .unwrap(),

            SelectionCriteria::BalancedAccuracyFallback => {
                // Weighted score: 0.7 * accuracy + 0.3 * ml_usage
                metrics
                    .iter()
                    .max_by(|a, b| {
                        let score_a = 0.7 * a.accuracy + 0.3 * a.ml_usage_rate;
                        let score_b = 0.7 * b.accuracy + 0.3 * b.ml_usage_rate;
                        score_a.partial_cmp(&score_b).unwrap()
                    })
                    .cloned()
                    .unwrap()
            }

            SelectionCriteria::MinFallbackAboveBaseline => {
                // Filter to thresholds that maintain accuracy above baseline
                let above_baseline: Vec<_> = metrics
                    .iter()
                    .filter(|m| m.accuracy >= baseline_accuracy)
                    .collect();

                if above_baseline.is_empty() {
                    // Fall back to max accuracy
                    metrics
                        .iter()
                        .max_by(|a, b| a.accuracy.partial_cmp(&b.accuracy).unwrap())
                        .cloned()
                        .unwrap()
                } else {
                    // Select minimum fallback among those above baseline
                    above_baseline
                        .into_iter()
                        .min_by(|a, b| a.fallback_rate.partial_cmp(&b.fallback_rate).unwrap())
                        .cloned()
                        .unwrap()
                }
            }
        }
    }
}

/// Quick function to find optimal threshold from validation data.
pub fn find_optimal_threshold(samples: &[ValidationSample]) -> f64 {
    ThresholdTuner::new().tune(samples).optimal_threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // ValidationSample tests
    // ========================================================================

    #[test]
    fn validation_sample_new() {
        let sample = ValidationSample::new(
            InferredOwnership::Owned,
            InferredOwnership::Owned,
            InferredOwnership::Borrowed,
            0.8,
        );

        assert_eq!(sample.ground_truth, InferredOwnership::Owned);
        assert!(sample.rule_correct());
        assert!(!sample.ml_correct());
    }

    #[test]
    fn validation_sample_clamps_confidence() {
        let sample = ValidationSample::new(
            InferredOwnership::Owned,
            InferredOwnership::Owned,
            InferredOwnership::Owned,
            1.5, // Should be clamped to 1.0
        );

        assert!((sample.ml_confidence - 1.0).abs() < 0.001);

        let sample2 = ValidationSample::new(
            InferredOwnership::Owned,
            InferredOwnership::Owned,
            InferredOwnership::Owned,
            -0.5, // Should be clamped to 0.0
        );

        assert!((sample2.ml_confidence - 0.0).abs() < 0.001);
    }

    #[test]
    fn validation_sample_hybrid_prediction() {
        let sample = ValidationSample::new(
            InferredOwnership::Owned,
            InferredOwnership::Borrowed, // Rule says Borrowed
            InferredOwnership::Owned,    // ML says Owned
            0.7,                         // ML confidence
        );

        // At threshold 0.5, ML should be used (0.7 >= 0.5)
        assert_eq!(sample.hybrid_prediction(0.5), InferredOwnership::Owned);

        // At threshold 0.8, rules should be used (0.7 < 0.8)
        assert_eq!(sample.hybrid_prediction(0.8), InferredOwnership::Borrowed);
    }

    #[test]
    fn validation_sample_hybrid_correct() {
        let sample = ValidationSample::new(
            InferredOwnership::Owned,    // Ground truth: Owned
            InferredOwnership::Borrowed, // Rule wrong
            InferredOwnership::Owned,    // ML correct
            0.7,
        );

        // At threshold 0.5, ML is used (correct)
        assert!(sample.hybrid_correct(0.5));

        // At threshold 0.8, rules are used (wrong)
        assert!(!sample.hybrid_correct(0.8));
    }

    // ========================================================================
    // ThresholdMetrics tests
    // ========================================================================

    #[test]
    fn threshold_metrics_empty() {
        let metrics = ThresholdMetrics::calculate(&[], 0.5);
        assert_eq!(metrics.sample_count, 0);
        assert_eq!(metrics.accuracy, 0.0);
    }

    #[test]
    fn threshold_metrics_all_correct() {
        let samples = vec![
            ValidationSample::new(
                InferredOwnership::Owned,
                InferredOwnership::Owned,
                InferredOwnership::Owned,
                0.9,
            ),
            ValidationSample::new(
                InferredOwnership::Borrowed,
                InferredOwnership::Borrowed,
                InferredOwnership::Borrowed,
                0.8,
            ),
        ];

        let metrics = ThresholdMetrics::calculate(&samples, 0.5);
        assert_eq!(metrics.sample_count, 2);
        assert!((metrics.accuracy - 1.0).abs() < 0.001);
    }

    #[test]
    fn threshold_metrics_fallback_rate() {
        let samples = vec![
            ValidationSample::new(
                InferredOwnership::Owned,
                InferredOwnership::Owned,
                InferredOwnership::Owned,
                0.9, // Above 0.5
            ),
            ValidationSample::new(
                InferredOwnership::Borrowed,
                InferredOwnership::Borrowed,
                InferredOwnership::Borrowed,
                0.3, // Below 0.5
            ),
        ];

        let metrics = ThresholdMetrics::calculate(&samples, 0.5);
        assert!((metrics.fallback_rate - 0.5).abs() < 0.001); // 1/2 samples fallback
        assert!((metrics.ml_usage_rate - 0.5).abs() < 0.001); // 1/2 samples use ML
    }

    // ========================================================================
    // ThresholdTuner tests
    // ========================================================================

    #[test]
    fn threshold_tuner_default() {
        let tuner = ThresholdTuner::new();
        assert_eq!(tuner.candidates.len(), 10);
        assert!(tuner.candidates.contains(&0.65));
    }

    #[test]
    fn threshold_tuner_add_candidate() {
        let mut tuner = ThresholdTuner::new();
        tuner.add_candidate(0.55);
        assert!(tuner.candidates.contains(&0.55));
    }

    #[test]
    fn threshold_tuner_tune_empty() {
        let tuner = ThresholdTuner::new();
        let result = tuner.tune(&[]);
        assert!((result.optimal_threshold - 0.65).abs() < 0.001);
    }

    #[test]
    fn threshold_tuner_finds_optimal() {
        // Create samples where ML is better at high confidence
        let samples = vec![
            // High confidence ML predictions (correct)
            ValidationSample::new(
                InferredOwnership::Owned,
                InferredOwnership::Borrowed, // Rule wrong
                InferredOwnership::Owned,    // ML correct
                0.9,
            ),
            ValidationSample::new(
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,    // Rule wrong
                InferredOwnership::Borrowed, // ML correct
                0.85,
            ),
            // Low confidence ML predictions (wrong)
            ValidationSample::new(
                InferredOwnership::Owned,
                InferredOwnership::Owned,    // Rule correct
                InferredOwnership::Borrowed, // ML wrong
                0.4,
            ),
            ValidationSample::new(
                InferredOwnership::Borrowed,
                InferredOwnership::Borrowed, // Rule correct
                InferredOwnership::Owned,    // ML wrong
                0.3,
            ),
        ];

        let tuner = ThresholdTuner::new().with_criteria(SelectionCriteria::MaxAccuracy);
        let result = tuner.tune(&samples);

        // Optimal should be around 0.7-0.8 (use ML for high conf, rules for low)
        assert!(result.optimal_threshold >= 0.5);
        assert!(result.optimal_metrics.accuracy > 0.5);
    }

    #[test]
    fn threshold_tuner_selection_criteria() {
        let samples = vec![
            ValidationSample::new(
                InferredOwnership::Owned,
                InferredOwnership::Owned,
                InferredOwnership::Owned,
                0.9,
            ),
            ValidationSample::new(
                InferredOwnership::Borrowed,
                InferredOwnership::Borrowed,
                InferredOwnership::Borrowed,
                0.8,
            ),
        ];

        let max_acc = ThresholdTuner::new()
            .with_criteria(SelectionCriteria::MaxAccuracy)
            .tune(&samples);

        let max_f1 = ThresholdTuner::new()
            .with_criteria(SelectionCriteria::MaxF1)
            .tune(&samples);

        // Both should find optimal (all correct samples)
        assert!((max_acc.optimal_metrics.accuracy - 1.0).abs() < 0.001);
        assert!((max_f1.optimal_metrics.accuracy - 1.0).abs() < 0.001);
    }

    #[test]
    fn threshold_tuner_balanced_criteria() {
        // Create scenario where low threshold has high accuracy but more ML usage
        let mut samples = Vec::new();

        // Many high-confidence correct ML predictions
        for _ in 0..80 {
            samples.push(ValidationSample::new(
                InferredOwnership::Owned,
                InferredOwnership::Borrowed, // Rule wrong
                InferredOwnership::Owned,    // ML correct
                0.9,
            ));
        }

        // Some low-confidence where rules are correct
        for _ in 0..20 {
            samples.push(ValidationSample::new(
                InferredOwnership::Borrowed,
                InferredOwnership::Borrowed, // Rule correct
                InferredOwnership::Owned,    // ML wrong
                0.3,
            ));
        }

        let balanced = ThresholdTuner::new()
            .with_criteria(SelectionCriteria::BalancedAccuracyFallback)
            .tune(&samples);

        // Should find threshold that balances accuracy and ML usage
        assert!(balanced.optimal_metrics.accuracy > 0.7);
    }

    // ========================================================================
    // TuningResult tests
    // ========================================================================

    #[test]
    fn tuning_result_to_markdown() {
        let samples = vec![ValidationSample::new(
            InferredOwnership::Owned,
            InferredOwnership::Owned,
            InferredOwnership::Owned,
            0.9,
        )];

        let result = ThresholdTuner::new().tune(&samples);
        let md = result.to_markdown();

        assert!(md.contains("Threshold Tuning Report"));
        assert!(md.contains("Optimal Threshold"));
        assert!(md.contains("Accuracy"));
    }

    #[test]
    fn tuning_result_improvement() {
        // Scenario where ML improves over rules
        let samples = vec![
            ValidationSample::new(
                InferredOwnership::Owned,
                InferredOwnership::Borrowed, // Rule wrong
                InferredOwnership::Owned,    // ML correct
                0.9,
            ),
            ValidationSample::new(
                InferredOwnership::Borrowed,
                InferredOwnership::Borrowed, // Rule correct
                InferredOwnership::Borrowed, // ML correct
                0.8,
            ),
        ];

        let result = ThresholdTuner::new().tune(&samples);

        // Rules: 50% accuracy, Hybrid: 100% accuracy
        assert!((result.baseline_accuracy - 0.5).abs() < 0.001);
        assert!((result.optimal_metrics.accuracy - 1.0).abs() < 0.001);
        assert!(result.improvement_over_baseline > 0.0);
    }

    // ========================================================================
    // Convenience function tests
    // ========================================================================

    #[test]
    fn find_optimal_threshold_function() {
        let samples = vec![ValidationSample::new(
            InferredOwnership::Owned,
            InferredOwnership::Owned,
            InferredOwnership::Owned,
            0.9,
        )];

        let threshold = find_optimal_threshold(&samples);
        assert!((0.0..=1.0).contains(&threshold));
    }

    // ========================================================================
    // SelectionCriteria tests
    // ========================================================================

    #[test]
    fn selection_criteria_display() {
        assert_eq!(SelectionCriteria::MaxAccuracy.to_string(), "max-accuracy");
        assert_eq!(SelectionCriteria::MaxF1.to_string(), "max-f1");
        assert_eq!(
            SelectionCriteria::BalancedAccuracyFallback.to_string(),
            "balanced"
        );
        assert_eq!(
            SelectionCriteria::MinFallbackAboveBaseline.to_string(),
            "min-fallback"
        );
    }
}
