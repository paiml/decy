//! A/B testing framework for ownership inference (DECY-ML-013).
//!
//! Enables controlled comparison between rule-based and ML-enhanced
//! ownership classification to measure improvement and prevent regression.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    A/B TESTING FRAMEWORK                        │
//! │  ┌───────────────┐         ┌───────────────┐                   │
//! │  │   Control     │         │   Treatment   │                   │
//! │  │ (Rules Only)  │ ◄─────► │ (Hybrid ML)   │                   │
//! │  └───────────────┘         └───────────────┘                   │
//! │          │                         │                            │
//! │          ▼                         ▼                            │
//! │  ┌───────────────────────────────────────────┐                  │
//! │  │           Metrics Collector               │                  │
//! │  │  • Accuracy   • Confidence   • Latency    │                  │
//! │  └───────────────────────────────────────────┘                  │
//! │                         │                                       │
//! │                         ▼                                       │
//! │  ┌───────────────────────────────────────────┐                  │
//! │  │           Statistical Analysis            │                  │
//! │  │  • Chi-squared   • Effect size           │                  │
//! │  └───────────────────────────────────────────┘                  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use crate::hybrid_classifier::{ClassificationMethod, HybridResult};
use crate::ml_features::InferredOwnership;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A/B test variant identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TestVariant {
    /// Control group: rule-based only
    Control,
    /// Treatment group: hybrid ML-enhanced
    Treatment,
}

impl std::fmt::Display for TestVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestVariant::Control => write!(f, "control"),
            TestVariant::Treatment => write!(f, "treatment"),
        }
    }
}

/// Single observation from an A/B test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestObservation {
    /// Test variant used
    pub variant: TestVariant,
    /// Variable name being classified
    pub variable: String,
    /// Predicted ownership
    pub predicted: InferredOwnership,
    /// Ground truth (if available)
    pub ground_truth: Option<InferredOwnership>,
    /// Confidence score
    pub confidence: f64,
    /// Classification method used
    pub method: ClassificationMethod,
    /// Classification latency
    pub latency: Duration,
    /// Whether prediction was correct (if ground truth available)
    pub correct: Option<bool>,
}

impl TestObservation {
    /// Create observation from hybrid result.
    pub fn from_result(
        variant: TestVariant,
        result: &HybridResult,
        ground_truth: Option<InferredOwnership>,
        latency: Duration,
    ) -> Self {
        let correct = ground_truth.as_ref().map(|gt| *gt == result.ownership);

        Self {
            variant,
            variable: result.variable.clone(),
            predicted: result.ownership,
            ground_truth,
            confidence: result.confidence,
            method: result.method,
            latency,
            correct,
        }
    }
}

/// Metrics for a single test variant.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VariantMetrics {
    /// Total observations
    pub count: u64,
    /// Correct predictions (where ground truth available)
    pub correct: u64,
    /// Total with ground truth
    pub with_ground_truth: u64,
    /// Sum of confidence scores
    pub confidence_sum: f64,
    /// Sum of latencies (in microseconds)
    pub latency_sum_us: u64,
    /// Predictions by ownership type
    pub by_ownership: HashMap<String, u64>,
    /// Predictions by method
    pub by_method: HashMap<String, u64>,
}

impl VariantMetrics {
    /// Create new metrics tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an observation.
    pub fn record(&mut self, obs: &TestObservation) {
        self.count += 1;
        self.confidence_sum += obs.confidence;
        self.latency_sum_us += obs.latency.as_micros() as u64;

        // Track ownership distribution
        *self
            .by_ownership
            .entry(format!("{:?}", obs.predicted))
            .or_insert(0) += 1;

        // Track method distribution
        *self.by_method.entry(obs.method.to_string()).or_insert(0) += 1;

        // Track accuracy (if ground truth available)
        if let Some(correct) = obs.correct {
            self.with_ground_truth += 1;
            if correct {
                self.correct += 1;
            }
        }
    }

    /// Get accuracy rate (0.0 - 1.0).
    pub fn accuracy(&self) -> f64 {
        if self.with_ground_truth == 0 {
            0.0
        } else {
            self.correct as f64 / self.with_ground_truth as f64
        }
    }

    /// Get average confidence.
    pub fn avg_confidence(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.confidence_sum / self.count as f64
        }
    }

    /// Get average latency in microseconds.
    pub fn avg_latency_us(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.latency_sum_us as f64 / self.count as f64
        }
    }
}

/// A/B test experiment tracker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABExperiment {
    /// Experiment name/ID
    pub name: String,
    /// Experiment description
    pub description: String,
    /// Control group metrics
    pub control: VariantMetrics,
    /// Treatment group metrics
    pub treatment: VariantMetrics,
    /// Start timestamp (Unix millis)
    pub started_at: u64,
    /// End timestamp (Unix millis, 0 if ongoing)
    pub ended_at: u64,
}

impl ABExperiment {
    /// Create a new experiment.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            name: name.into(),
            description: description.into(),
            control: VariantMetrics::new(),
            treatment: VariantMetrics::new(),
            started_at: now,
            ended_at: 0,
        }
    }

    /// Record an observation.
    pub fn record(&mut self, obs: &TestObservation) {
        match obs.variant {
            TestVariant::Control => self.control.record(obs),
            TestVariant::Treatment => self.treatment.record(obs),
        }
    }

    /// End the experiment.
    pub fn end(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.ended_at = now;
    }

    /// Check if experiment is active.
    pub fn is_active(&self) -> bool {
        self.ended_at == 0
    }

    /// Get total observations across both variants.
    pub fn total_observations(&self) -> u64 {
        self.control.count + self.treatment.count
    }

    /// Calculate accuracy lift (treatment - control).
    pub fn accuracy_lift(&self) -> f64 {
        self.treatment.accuracy() - self.control.accuracy()
    }

    /// Calculate confidence lift.
    pub fn confidence_lift(&self) -> f64 {
        self.treatment.avg_confidence() - self.control.avg_confidence()
    }

    /// Calculate latency difference (negative = treatment faster).
    pub fn latency_diff_us(&self) -> f64 {
        self.treatment.avg_latency_us() - self.control.avg_latency_us()
    }

    /// Check if treatment is statistically better (simplified chi-squared).
    ///
    /// Returns (is_significant, p_value_estimate).
    pub fn is_treatment_better(&self) -> (bool, f64) {
        // Simplified significance test
        // In practice, use proper chi-squared or Bayesian analysis

        let control_correct = self.control.correct as f64;
        let control_wrong = (self.control.with_ground_truth - self.control.correct) as f64;
        let treatment_correct = self.treatment.correct as f64;
        let treatment_wrong = (self.treatment.with_ground_truth - self.treatment.correct) as f64;

        // Not enough data
        if self.control.with_ground_truth < 30 || self.treatment.with_ground_truth < 30 {
            return (false, 1.0);
        }

        // Calculate chi-squared statistic
        let total = control_correct + control_wrong + treatment_correct + treatment_wrong;
        if total == 0.0 {
            return (false, 1.0);
        }

        let row_total_control = control_correct + control_wrong;
        let row_total_treatment = treatment_correct + treatment_wrong;
        let col_total_correct = control_correct + treatment_correct;
        let col_total_wrong = control_wrong + treatment_wrong;

        // Expected values
        let e_cc = (row_total_control * col_total_correct) / total;
        let e_cw = (row_total_control * col_total_wrong) / total;
        let e_tc = (row_total_treatment * col_total_correct) / total;
        let e_tw = (row_total_treatment * col_total_wrong) / total;

        // Chi-squared calculation
        let chi_sq = if e_cc > 0.0 && e_cw > 0.0 && e_tc > 0.0 && e_tw > 0.0 {
            ((control_correct - e_cc).powi(2) / e_cc)
                + ((control_wrong - e_cw).powi(2) / e_cw)
                + ((treatment_correct - e_tc).powi(2) / e_tc)
                + ((treatment_wrong - e_tw).powi(2) / e_tw)
        } else {
            0.0
        };

        // Approximate p-value (df=1)
        // chi_sq > 3.84 => p < 0.05
        // chi_sq > 6.63 => p < 0.01
        let p_value = if chi_sq > 6.63 {
            0.01
        } else if chi_sq > 3.84 {
            0.05
        } else {
            0.5
        };

        let is_significant = p_value < 0.05 && self.treatment.accuracy() > self.control.accuracy();

        (is_significant, p_value)
    }

    /// Generate markdown report.
    pub fn to_markdown(&self) -> String {
        let (is_better, p_value) = self.is_treatment_better();
        let status = if !self.is_active() {
            "COMPLETED"
        } else {
            "ACTIVE"
        };

        let recommendation = if is_better {
            "✅ ADOPT TREATMENT - Statistically significant improvement"
        } else if self.total_observations() < 100 {
            "⏳ INSUFFICIENT DATA - Need more observations"
        } else {
            "❌ KEEP CONTROL - No significant improvement"
        };

        format!(
            r#"## A/B Test Report: {}

**Status**: {} | **Description**: {}

### Summary

| Metric | Control | Treatment | Lift |
|--------|---------|-----------|------|
| Observations | {} | {} | - |
| Accuracy | {:.1}% | {:.1}% | {:+.1}% |
| Avg Confidence | {:.2} | {:.2} | {:+.2} |
| Avg Latency | {:.0}μs | {:.0}μs | {:+.0}μs |

### Statistical Analysis

- **Chi-squared p-value**: {:.3}
- **Treatment better?**: {}
- **Recommendation**: {}

### Control Group Distribution

{}

### Treatment Group Distribution

{}
"#,
            self.name,
            status,
            self.description,
            self.control.count,
            self.treatment.count,
            self.control.accuracy() * 100.0,
            self.treatment.accuracy() * 100.0,
            self.accuracy_lift() * 100.0,
            self.control.avg_confidence(),
            self.treatment.avg_confidence(),
            self.confidence_lift(),
            self.control.avg_latency_us(),
            self.treatment.avg_latency_us(),
            self.latency_diff_us(),
            p_value,
            if is_better { "Yes" } else { "No" },
            recommendation,
            self.format_distribution(&self.control),
            self.format_distribution(&self.treatment),
        )
    }

    fn format_distribution(&self, metrics: &VariantMetrics) -> String {
        let mut lines = Vec::new();
        for (ownership, count) in &metrics.by_ownership {
            let pct = if metrics.count > 0 {
                (*count as f64 / metrics.count as f64) * 100.0
            } else {
                0.0
            };
            lines.push(format!("- {}: {} ({:.1}%)", ownership, count, pct));
        }
        if lines.is_empty() {
            "- No data".to_string()
        } else {
            lines.join("\n")
        }
    }
}

/// Simple A/B test runner for evaluating classification.
#[derive(Debug)]
pub struct ABTestRunner {
    /// Current experiment
    experiment: ABExperiment,
    /// Assignment strategy
    strategy: AssignmentStrategy,
    /// Random seed for reproducibility
    seed: u64,
    /// Counter for round-robin
    counter: u64,
}

/// Strategy for assigning samples to variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentStrategy {
    /// Alternate between control and treatment
    RoundRobin,
    /// Random assignment (50/50)
    Random,
    /// All to control (baseline measurement)
    AllControl,
    /// All to treatment (full rollout)
    AllTreatment,
}

impl ABTestRunner {
    /// Create a new test runner.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        strategy: AssignmentStrategy,
    ) -> Self {
        Self {
            experiment: ABExperiment::new(name, description),
            strategy,
            seed: 42,
            counter: 0,
        }
    }

    /// Set random seed for reproducible assignments.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Get the next variant assignment.
    pub fn assign(&mut self) -> TestVariant {
        let variant = match self.strategy {
            AssignmentStrategy::RoundRobin => {
                if self.counter % 2 == 0 {
                    TestVariant::Control
                } else {
                    TestVariant::Treatment
                }
            }
            AssignmentStrategy::Random => {
                // Simple LCG for deterministic "random"
                self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                if self.seed % 2 == 0 {
                    TestVariant::Control
                } else {
                    TestVariant::Treatment
                }
            }
            AssignmentStrategy::AllControl => TestVariant::Control,
            AssignmentStrategy::AllTreatment => TestVariant::Treatment,
        };
        self.counter += 1;
        variant
    }

    /// Record a classification result.
    pub fn record(
        &mut self,
        variant: TestVariant,
        result: &HybridResult,
        ground_truth: Option<InferredOwnership>,
        latency: Duration,
    ) {
        let obs = TestObservation::from_result(variant, result, ground_truth, latency);
        self.experiment.record(&obs);
    }

    /// Time and record a classification.
    pub fn timed_record<F>(
        &mut self,
        variant: TestVariant,
        classify_fn: F,
        ground_truth: Option<InferredOwnership>,
    ) -> HybridResult
    where
        F: FnOnce() -> HybridResult,
    {
        let start = Instant::now();
        let result = classify_fn();
        let latency = start.elapsed();

        self.record(variant, &result, ground_truth, latency);
        result
    }

    /// End the experiment and get report.
    pub fn finish(&mut self) -> String {
        self.experiment.end();
        self.experiment.to_markdown()
    }

    /// Get current experiment state.
    pub fn experiment(&self) -> &ABExperiment {
        &self.experiment
    }

    /// Get mutable experiment for direct recording.
    pub fn experiment_mut(&mut self) -> &mut ABExperiment {
        &mut self.experiment
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // TestVariant tests
    // ========================================================================

    #[test]
    fn test_variant_display() {
        assert_eq!(TestVariant::Control.to_string(), "control");
        assert_eq!(TestVariant::Treatment.to_string(), "treatment");
    }

    // ========================================================================
    // VariantMetrics tests
    // ========================================================================

    #[test]
    fn variant_metrics_default() {
        let metrics = VariantMetrics::new();
        assert_eq!(metrics.count, 0);
        assert_eq!(metrics.accuracy(), 0.0);
        assert_eq!(metrics.avg_confidence(), 0.0);
    }

    #[test]
    fn variant_metrics_record() {
        let mut metrics = VariantMetrics::new();

        let obs = TestObservation {
            variant: TestVariant::Control,
            variable: "ptr".to_string(),
            predicted: InferredOwnership::Owned,
            ground_truth: Some(InferredOwnership::Owned),
            confidence: 0.9,
            method: ClassificationMethod::RuleBased,
            latency: Duration::from_micros(100),
            correct: Some(true),
        };

        metrics.record(&obs);

        assert_eq!(metrics.count, 1);
        assert_eq!(metrics.with_ground_truth, 1);
        assert_eq!(metrics.correct, 1);
        assert!((metrics.accuracy() - 1.0).abs() < 0.001);
        assert!((metrics.avg_confidence() - 0.9).abs() < 0.001);
    }

    #[test]
    fn variant_metrics_accuracy() {
        let mut metrics = VariantMetrics::new();

        // 3 correct, 1 wrong
        for (correct, gt) in [(true, true), (true, true), (true, true), (false, false)] {
            let obs = TestObservation {
                variant: TestVariant::Control,
                variable: "x".to_string(),
                predicted: InferredOwnership::Owned,
                ground_truth: Some(if gt {
                    InferredOwnership::Owned
                } else {
                    InferredOwnership::Borrowed
                }),
                confidence: 0.8,
                method: ClassificationMethod::RuleBased,
                latency: Duration::from_micros(50),
                correct: Some(correct),
            };
            metrics.record(&obs);
        }

        assert_eq!(metrics.with_ground_truth, 4);
        assert_eq!(metrics.correct, 3);
        assert!((metrics.accuracy() - 0.75).abs() < 0.001);
    }

    // ========================================================================
    // ABExperiment tests
    // ========================================================================

    #[test]
    fn ab_experiment_new() {
        let exp = ABExperiment::new("test-001", "Testing hybrid vs rules");
        assert!(exp.is_active());
        assert_eq!(exp.name, "test-001");
        assert_eq!(exp.total_observations(), 0);
    }

    #[test]
    fn ab_experiment_end() {
        let mut exp = ABExperiment::new("test", "desc");
        assert!(exp.is_active());
        exp.end();
        assert!(!exp.is_active());
    }

    #[test]
    fn ab_experiment_record() {
        let mut exp = ABExperiment::new("test", "desc");

        let obs_control = TestObservation {
            variant: TestVariant::Control,
            variable: "a".to_string(),
            predicted: InferredOwnership::Owned,
            ground_truth: Some(InferredOwnership::Owned),
            confidence: 0.8,
            method: ClassificationMethod::RuleBased,
            latency: Duration::from_micros(100),
            correct: Some(true),
        };

        let obs_treatment = TestObservation {
            variant: TestVariant::Treatment,
            variable: "b".to_string(),
            predicted: InferredOwnership::Borrowed,
            ground_truth: Some(InferredOwnership::Borrowed),
            confidence: 0.9,
            method: ClassificationMethod::MachineLearning,
            latency: Duration::from_micros(150),
            correct: Some(true),
        };

        exp.record(&obs_control);
        exp.record(&obs_treatment);

        assert_eq!(exp.control.count, 1);
        assert_eq!(exp.treatment.count, 1);
        assert_eq!(exp.total_observations(), 2);
    }

    #[test]
    fn ab_experiment_lift_calculation() {
        let mut exp = ABExperiment::new("test", "desc");

        // Control: 70% accuracy, 0.7 confidence
        for i in 0..10 {
            let correct = i < 7;
            let obs = TestObservation {
                variant: TestVariant::Control,
                variable: format!("c{}", i),
                predicted: InferredOwnership::Owned,
                ground_truth: Some(if correct {
                    InferredOwnership::Owned
                } else {
                    InferredOwnership::Borrowed
                }),
                confidence: 0.7,
                method: ClassificationMethod::RuleBased,
                latency: Duration::from_micros(100),
                correct: Some(correct),
            };
            exp.record(&obs);
        }

        // Treatment: 90% accuracy, 0.9 confidence
        for i in 0..10 {
            let correct = i < 9;
            let obs = TestObservation {
                variant: TestVariant::Treatment,
                variable: format!("t{}", i),
                predicted: InferredOwnership::Owned,
                ground_truth: Some(if correct {
                    InferredOwnership::Owned
                } else {
                    InferredOwnership::Borrowed
                }),
                confidence: 0.9,
                method: ClassificationMethod::MachineLearning,
                latency: Duration::from_micros(150),
                correct: Some(correct),
            };
            exp.record(&obs);
        }

        // Lift = 90% - 70% = 20%
        assert!((exp.accuracy_lift() - 0.2).abs() < 0.001);
        assert!((exp.confidence_lift() - 0.2).abs() < 0.001);
    }

    #[test]
    fn ab_experiment_to_markdown() {
        let exp = ABExperiment::new("test-001", "Hybrid vs Rules");
        let md = exp.to_markdown();

        assert!(md.contains("A/B Test Report: test-001"));
        assert!(md.contains("ACTIVE"));
        assert!(md.contains("Hybrid vs Rules"));
    }

    // ========================================================================
    // ABTestRunner tests
    // ========================================================================

    #[test]
    fn ab_runner_round_robin() {
        let mut runner = ABTestRunner::new("test", "desc", AssignmentStrategy::RoundRobin);

        assert_eq!(runner.assign(), TestVariant::Control);
        assert_eq!(runner.assign(), TestVariant::Treatment);
        assert_eq!(runner.assign(), TestVariant::Control);
        assert_eq!(runner.assign(), TestVariant::Treatment);
    }

    #[test]
    fn ab_runner_all_control() {
        let mut runner = ABTestRunner::new("test", "desc", AssignmentStrategy::AllControl);

        for _ in 0..10 {
            assert_eq!(runner.assign(), TestVariant::Control);
        }
    }

    #[test]
    fn ab_runner_all_treatment() {
        let mut runner = ABTestRunner::new("test", "desc", AssignmentStrategy::AllTreatment);

        for _ in 0..10 {
            assert_eq!(runner.assign(), TestVariant::Treatment);
        }
    }

    #[test]
    fn ab_runner_random_deterministic() {
        let mut runner1 =
            ABTestRunner::new("test", "desc", AssignmentStrategy::Random).with_seed(42);
        let mut runner2 =
            ABTestRunner::new("test", "desc", AssignmentStrategy::Random).with_seed(42);

        // Same seed should produce same sequence
        for _ in 0..10 {
            assert_eq!(runner1.assign(), runner2.assign());
        }
    }

    #[test]
    fn ab_runner_finish_generates_report() {
        let mut runner = ABTestRunner::new("exp-001", "Testing", AssignmentStrategy::RoundRobin);

        let result = HybridResult {
            variable: "ptr".to_string(),
            ownership: InferredOwnership::Owned,
            confidence: 0.9,
            method: ClassificationMethod::MachineLearning,
            rule_result: Some(InferredOwnership::Owned),
            ml_result: None,
            reasoning: "test".to_string(),
        };

        runner.record(
            TestVariant::Control,
            &result,
            Some(InferredOwnership::Owned),
            Duration::from_micros(100),
        );

        let report = runner.finish();
        assert!(report.contains("exp-001"));
        assert!(report.contains("COMPLETED"));
    }

    // ========================================================================
    // Statistical tests
    // ========================================================================

    #[test]
    fn ab_experiment_insufficient_data() {
        let mut exp = ABExperiment::new("test", "desc");

        // Only 5 samples - not enough
        for i in 0..5 {
            let obs = TestObservation {
                variant: TestVariant::Control,
                variable: format!("c{}", i),
                predicted: InferredOwnership::Owned,
                ground_truth: Some(InferredOwnership::Owned),
                confidence: 0.8,
                method: ClassificationMethod::RuleBased,
                latency: Duration::from_micros(100),
                correct: Some(true),
            };
            exp.record(&obs);
        }

        let (is_better, p_value) = exp.is_treatment_better();
        assert!(!is_better);
        assert!((p_value - 1.0).abs() < 0.001); // Insufficient data returns p=1.0
    }

    #[test]
    fn ab_experiment_significant_improvement() {
        let mut exp = ABExperiment::new("test", "desc");

        // Control: 50% accuracy (30 samples)
        for i in 0..30 {
            let correct = i < 15;
            let obs = TestObservation {
                variant: TestVariant::Control,
                variable: format!("c{}", i),
                predicted: InferredOwnership::Owned,
                ground_truth: Some(if correct {
                    InferredOwnership::Owned
                } else {
                    InferredOwnership::Borrowed
                }),
                confidence: 0.5,
                method: ClassificationMethod::RuleBased,
                latency: Duration::from_micros(100),
                correct: Some(correct),
            };
            exp.record(&obs);
        }

        // Treatment: 90% accuracy (30 samples)
        for i in 0..30 {
            let correct = i < 27;
            let obs = TestObservation {
                variant: TestVariant::Treatment,
                variable: format!("t{}", i),
                predicted: InferredOwnership::Owned,
                ground_truth: Some(if correct {
                    InferredOwnership::Owned
                } else {
                    InferredOwnership::Borrowed
                }),
                confidence: 0.9,
                method: ClassificationMethod::MachineLearning,
                latency: Duration::from_micros(150),
                correct: Some(correct),
            };
            exp.record(&obs);
        }

        let (is_better, p_value) = exp.is_treatment_better();
        assert!(is_better);
        assert!(p_value < 0.05);
    }

    #[test]
    fn ab_test_zero_observations_returns_not_significant() {
        // Both groups have zero samples → with_ground_truth < 30 → early return (false, 1.0)
        let exp = ABExperiment::new("empty_test", "No data");
        let (is_better, p_value) = exp.is_treatment_better();
        assert!(!is_better);
        assert!((p_value - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ab_test_variant_metrics_default_trait() {
        let metrics = VariantMetrics::default();
        assert_eq!(metrics.count, 0);
        assert_eq!(metrics.accuracy(), 0.0);
        assert_eq!(metrics.avg_confidence(), 0.0);
    }

    fn make_obs(variant: TestVariant, correct: Option<bool>) -> TestObservation {
        TestObservation {
            variant,
            variable: "ptr".to_string(),
            predicted: InferredOwnership::Owned,
            ground_truth: Some(InferredOwnership::Owned),
            confidence: 0.8,
            method: ClassificationMethod::RuleBased,
            latency: Duration::from_micros(100),
            correct,
        }
    }

    #[test]
    fn ab_test_sufficient_data_both_correct() {
        // Both groups fully correct → chi_sq = 0 → not significant
        let mut exp = ABExperiment::new("equal", "Both equally good");
        for _ in 0..40 {
            exp.record(&make_obs(TestVariant::Control, Some(true)));
            exp.record(&make_obs(TestVariant::Treatment, Some(true)));
        }
        let (is_better, _p_value) = exp.is_treatment_better();
        assert!(!is_better, "Equal groups should not show treatment as better");
    }

    #[test]
    fn ab_test_total_zero_early_return() {
        // Manually set with_ground_truth >= 30 but all zeros (both groups have 0 correct/wrong)
        let mut exp = ABExperiment::new("zero_total", "Zero data case");
        // Need at least 30 with_ground_truth per group, but force total correct+wrong = 0
        // by having 30 observations with ground_truth=None
        for _ in 0..30 {
            let mut obs = make_obs(TestVariant::Control, None);
            obs.ground_truth = None;
            exp.record(&obs);
        }
        for _ in 0..30 {
            let mut obs = make_obs(TestVariant::Treatment, None);
            obs.ground_truth = None;
            exp.record(&obs);
        }
        // with_ground_truth will be 0, so this triggers the < 30 check
        let (is_better, p_value) = exp.is_treatment_better();
        assert!(!is_better);
        assert!((p_value - 1.0).abs() < f64::EPSILON);
    }
}
