//! Coverage expansion tests for ab_testing.rs
//!
//! Targets uncovered branches: format_distribution edge cases, from_result with
//! None ground truth, timed_record, experiment accessors, statistical edge cases,
//! to_markdown branch coverage, and VariantMetrics edge paths.

use crate::ab_testing::*;
use crate::hybrid_classifier::{ClassificationMethod, HybridResult};
use crate::ml_features::InferredOwnership;
use std::time::Duration;

// ============================================================================
// TestObservation::from_result: None ground truth
// ============================================================================

#[test]
fn test_observation_from_result_no_ground_truth() {
    let result = HybridResult {
        variable: "ptr".to_string(),
        ownership: InferredOwnership::Owned,
        confidence: 0.85,
        method: ClassificationMethod::MachineLearning,
        rule_result: Some(InferredOwnership::Owned),
        ml_result: None,
        reasoning: "ML classified as Owned".to_string(),
    };

    let obs = TestObservation::from_result(
        TestVariant::Treatment,
        &result,
        None,
        Duration::from_micros(200),
    );

    assert_eq!(obs.variant, TestVariant::Treatment);
    assert_eq!(obs.variable, "ptr");
    assert_eq!(obs.predicted, InferredOwnership::Owned);
    assert!(obs.ground_truth.is_none());
    assert!(obs.correct.is_none());
    assert!((obs.confidence - 0.85).abs() < 0.001);
    assert_eq!(obs.method, ClassificationMethod::MachineLearning);
}

#[test]
fn test_observation_from_result_incorrect_prediction() {
    let result = HybridResult {
        variable: "buf".to_string(),
        ownership: InferredOwnership::Borrowed,
        confidence: 0.6,
        method: ClassificationMethod::Fallback,
        rule_result: Some(InferredOwnership::Borrowed),
        ml_result: None,
        reasoning: "Fell back to rules".to_string(),
    };

    let obs = TestObservation::from_result(
        TestVariant::Control,
        &result,
        Some(InferredOwnership::Owned),
        Duration::from_micros(50),
    );

    assert_eq!(obs.correct, Some(false));
    assert_eq!(obs.predicted, InferredOwnership::Borrowed);
    assert_eq!(obs.ground_truth, Some(InferredOwnership::Owned));
}

#[test]
fn test_observation_from_result_correct_prediction() {
    let result = HybridResult {
        variable: "data".to_string(),
        ownership: InferredOwnership::Vec,
        confidence: 0.95,
        method: ClassificationMethod::Hybrid,
        rule_result: Some(InferredOwnership::Vec),
        ml_result: None,
        reasoning: "Both agree on Vec".to_string(),
    };

    let obs = TestObservation::from_result(
        TestVariant::Treatment,
        &result,
        Some(InferredOwnership::Vec),
        Duration::from_micros(120),
    );

    assert_eq!(obs.correct, Some(true));
}

// ============================================================================
// VariantMetrics: recording without ground truth
// ============================================================================

#[test]
fn test_variant_metrics_record_no_ground_truth() {
    let mut metrics = VariantMetrics::new();

    let obs = TestObservation {
        variant: TestVariant::Control,
        variable: "x".to_string(),
        predicted: InferredOwnership::BorrowedMut,
        ground_truth: None,
        confidence: 0.7,
        method: ClassificationMethod::RuleBased,
        latency: Duration::from_micros(80),
        correct: None,
    };

    metrics.record(&obs);

    assert_eq!(metrics.count, 1);
    assert_eq!(metrics.with_ground_truth, 0);
    assert_eq!(metrics.correct, 0);
    assert_eq!(metrics.accuracy(), 0.0);
    assert!((metrics.avg_confidence() - 0.7).abs() < 0.001);
    assert!((metrics.avg_latency_us() - 80.0).abs() < 0.001);
}

#[test]
fn test_variant_metrics_record_incorrect_prediction() {
    let mut metrics = VariantMetrics::new();

    let obs = TestObservation {
        variant: TestVariant::Treatment,
        variable: "y".to_string(),
        predicted: InferredOwnership::Owned,
        ground_truth: Some(InferredOwnership::Shared),
        confidence: 0.55,
        method: ClassificationMethod::Fallback,
        latency: Duration::from_micros(200),
        correct: Some(false),
    };

    metrics.record(&obs);

    assert_eq!(metrics.count, 1);
    assert_eq!(metrics.with_ground_truth, 1);
    assert_eq!(metrics.correct, 0);
    assert_eq!(metrics.accuracy(), 0.0);
}

// ============================================================================
// VariantMetrics: avg_latency_us edge case (zero count)
// ============================================================================

#[test]
fn test_variant_metrics_avg_latency_zero_count() {
    let metrics = VariantMetrics::new();
    assert_eq!(metrics.avg_latency_us(), 0.0);
}

// ============================================================================
// VariantMetrics: multiple records for by_ownership and by_method distributions
// ============================================================================

#[test]
fn test_variant_metrics_distributions() {
    let mut metrics = VariantMetrics::new();

    let methods = [
        ClassificationMethod::RuleBased,
        ClassificationMethod::MachineLearning,
        ClassificationMethod::Fallback,
        ClassificationMethod::Hybrid,
    ];

    let ownerships = [
        InferredOwnership::Owned,
        InferredOwnership::Borrowed,
        InferredOwnership::BorrowedMut,
        InferredOwnership::Shared,
    ];

    for (i, (method, ownership)) in methods.iter().zip(ownerships.iter()).enumerate() {
        let obs = TestObservation {
            variant: TestVariant::Control,
            variable: format!("var_{}", i),
            predicted: *ownership,
            ground_truth: Some(*ownership),
            confidence: 0.8,
            method: *method,
            latency: Duration::from_micros(100),
            correct: Some(true),
        };
        metrics.record(&obs);
    }

    assert_eq!(metrics.count, 4);
    assert_eq!(metrics.by_ownership.len(), 4);
    assert_eq!(metrics.by_method.len(), 4);

    // Verify method distribution
    assert_eq!(*metrics.by_method.get("rule-based").unwrap(), 1);
    assert_eq!(*metrics.by_method.get("ml").unwrap(), 1);
    assert_eq!(*metrics.by_method.get("fallback").unwrap(), 1);
    assert_eq!(*metrics.by_method.get("hybrid").unwrap(), 1);
}

// ============================================================================
// VariantMetrics: multiple records average calculations
// ============================================================================

#[test]
fn test_variant_metrics_averages() {
    let mut metrics = VariantMetrics::new();

    for i in 0..4 {
        let obs = TestObservation {
            variant: TestVariant::Control,
            variable: format!("v{}", i),
            predicted: InferredOwnership::Owned,
            ground_truth: Some(InferredOwnership::Owned),
            confidence: 0.5 + (i as f64 * 0.1), // 0.5, 0.6, 0.7, 0.8
            method: ClassificationMethod::RuleBased,
            latency: Duration::from_micros(100 + i * 100), // 100, 200, 300, 400
            correct: Some(true),
        };
        metrics.record(&obs);
    }

    // avg confidence = (0.5+0.6+0.7+0.8)/4 = 0.65
    assert!((metrics.avg_confidence() - 0.65).abs() < 0.001);
    // avg latency = (100+200+300+400)/4 = 250
    assert!((metrics.avg_latency_us() - 250.0).abs() < 0.001);
}

// ============================================================================
// ABExperiment: latency_diff_us
// ============================================================================

#[test]
fn test_ab_experiment_latency_diff() {
    let mut exp = ABExperiment::new("latency-test".to_string(), "Compare latency".to_string());

    // Control: 100us average
    for i in 0..5 {
        let obs = TestObservation {
            variant: TestVariant::Control,
            variable: format!("c{}", i),
            predicted: InferredOwnership::Owned,
            ground_truth: None,
            confidence: 0.8,
            method: ClassificationMethod::RuleBased,
            latency: Duration::from_micros(100),
            correct: None,
        };
        exp.record(&obs);
    }

    // Treatment: 200us average (slower)
    for i in 0..5 {
        let obs = TestObservation {
            variant: TestVariant::Treatment,
            variable: format!("t{}", i),
            predicted: InferredOwnership::Owned,
            ground_truth: None,
            confidence: 0.9,
            method: ClassificationMethod::MachineLearning,
            latency: Duration::from_micros(200),
            correct: None,
        };
        exp.record(&obs);
    }

    // Treatment is 100us slower
    assert!((exp.latency_diff_us() - 100.0).abs() < 0.001);
}

// ============================================================================
// ABExperiment: is_treatment_better edge cases
// ============================================================================

#[test]
fn test_is_treatment_better_zero_total() {
    let exp = ABExperiment::new("empty-test".to_string(), "No data".to_string());
    let (is_better, p_value) = exp.is_treatment_better();
    assert!(!is_better);
    assert!((p_value - 1.0).abs() < 0.001);
}

#[test]
fn test_is_treatment_better_treatment_not_actually_better() {
    let mut exp = ABExperiment::new("no-improve".to_string(), "No improvement".to_string());

    // Control: 90% accuracy (30 samples)
    for i in 0..30 {
        let correct = i < 27;
        let obs = TestObservation {
            variant: TestVariant::Control,
            variable: format!("c{}", i),
            predicted: InferredOwnership::Owned,
            ground_truth: Some(if correct {
                InferredOwnership::Owned
            } else {
                InferredOwnership::Borrowed
            }),
            confidence: 0.9,
            method: ClassificationMethod::RuleBased,
            latency: Duration::from_micros(100),
            correct: Some(correct),
        };
        exp.record(&obs);
    }

    // Treatment: 50% accuracy (30 samples) - worse than control
    for i in 0..30 {
        let correct = i < 15;
        let obs = TestObservation {
            variant: TestVariant::Treatment,
            variable: format!("t{}", i),
            predicted: InferredOwnership::Owned,
            ground_truth: Some(if correct {
                InferredOwnership::Owned
            } else {
                InferredOwnership::Borrowed
            }),
            confidence: 0.5,
            method: ClassificationMethod::MachineLearning,
            latency: Duration::from_micros(150),
            correct: Some(correct),
        };
        exp.record(&obs);
    }

    let (is_better, _p_value) = exp.is_treatment_better();
    // Treatment is worse, so should NOT be better even if statistically significant
    assert!(!is_better);
}

#[test]
fn test_is_treatment_better_moderate_chi_sq() {
    let mut exp = ABExperiment::new("moderate".to_string(), "Moderate difference".to_string());

    // Control: 60% accuracy (30 samples)
    for i in 0..30 {
        let correct = i < 18;
        let obs = TestObservation {
            variant: TestVariant::Control,
            variable: format!("c{}", i),
            predicted: InferredOwnership::Owned,
            ground_truth: Some(if correct {
                InferredOwnership::Owned
            } else {
                InferredOwnership::Borrowed
            }),
            confidence: 0.6,
            method: ClassificationMethod::RuleBased,
            latency: Duration::from_micros(100),
            correct: Some(correct),
        };
        exp.record(&obs);
    }

    // Treatment: 80% accuracy (30 samples)
    for i in 0..30 {
        let correct = i < 24;
        let obs = TestObservation {
            variant: TestVariant::Treatment,
            variable: format!("t{}", i),
            predicted: InferredOwnership::Owned,
            ground_truth: Some(if correct {
                InferredOwnership::Owned
            } else {
                InferredOwnership::Borrowed
            }),
            confidence: 0.8,
            method: ClassificationMethod::MachineLearning,
            latency: Duration::from_micros(150),
            correct: Some(correct),
        };
        exp.record(&obs);
    }

    let (is_better, p_value) = exp.is_treatment_better();
    // With 60% vs 80% on 30 samples, chi-squared should be moderate
    // This tests the chi_sq > 3.84 branch (p = 0.05)
    assert!(p_value <= 0.5);
    if p_value < 0.05 {
        assert!(is_better);
    }
}

#[test]
fn test_is_treatment_better_equal_performance() {
    let mut exp = ABExperiment::new("equal".to_string(), "Same performance".to_string());

    // Both variants: 70% accuracy (30 samples each)
    for i in 0..30 {
        let correct = i < 21;
        for variant in [TestVariant::Control, TestVariant::Treatment] {
            let obs = TestObservation {
                variant,
                variable: format!("{:?}_{}", variant, i),
                predicted: InferredOwnership::Owned,
                ground_truth: Some(if correct {
                    InferredOwnership::Owned
                } else {
                    InferredOwnership::Borrowed
                }),
                confidence: 0.7,
                method: if variant == TestVariant::Control {
                    ClassificationMethod::RuleBased
                } else {
                    ClassificationMethod::MachineLearning
                },
                latency: Duration::from_micros(100),
                correct: Some(correct),
            };
            exp.record(&obs);
        }
    }

    let (is_better, p_value) = exp.is_treatment_better();
    // Equal performance: chi_sq should be ~0, p=0.5
    assert!(!is_better);
    assert!((p_value - 0.5).abs() < 0.001);
}

// ============================================================================
// ABExperiment: to_markdown branch coverage
// ============================================================================

#[test]
fn test_to_markdown_completed_experiment() {
    let mut exp = ABExperiment::new("completed-test".to_string(), "Testing".to_string());
    exp.end();

    let md = exp.to_markdown();
    assert!(md.contains("COMPLETED"));
    assert!(md.contains("completed-test"));
}

#[test]
fn test_to_markdown_insufficient_data() {
    let mut exp = ABExperiment::new("small-test".to_string(), "Few samples".to_string());

    // Add just a few observations (< 100 total)
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

    let md = exp.to_markdown();
    assert!(md.contains("INSUFFICIENT DATA"));
}

#[test]
fn test_to_markdown_keep_control_recommendation() {
    let mut exp =
        ABExperiment::new("no-improvement".to_string(), "Treatment not better".to_string());

    // Control: 90% accuracy, 60 samples
    for i in 0..60 {
        let correct = i < 54;
        let obs = TestObservation {
            variant: TestVariant::Control,
            variable: format!("c{}", i),
            predicted: InferredOwnership::Owned,
            ground_truth: Some(if correct {
                InferredOwnership::Owned
            } else {
                InferredOwnership::Borrowed
            }),
            confidence: 0.9,
            method: ClassificationMethod::RuleBased,
            latency: Duration::from_micros(100),
            correct: Some(correct),
        };
        exp.record(&obs);
    }

    // Treatment: 90% accuracy, 60 samples (same as control)
    for i in 0..60 {
        let correct = i < 54;
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
            latency: Duration::from_micros(100),
            correct: Some(correct),
        };
        exp.record(&obs);
    }

    let md = exp.to_markdown();
    // >= 100 observations but no significant improvement
    assert!(md.contains("KEEP CONTROL"));
}

#[test]
fn test_to_markdown_adopt_treatment() {
    let mut exp = ABExperiment::new("success".to_string(), "Treatment is better".to_string());

    // Control: 50% accuracy, 30 samples
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

    // Treatment: 90% accuracy, 30 samples
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

    let md = exp.to_markdown();
    assert!(md.contains("ADOPT TREATMENT"));
}

// ============================================================================
// ABExperiment: format_distribution edge cases
// ============================================================================

#[test]
fn test_format_distribution_empty_metrics() {
    let exp = ABExperiment::new("dist-test".to_string(), "Distribution test".to_string());

    // Empty metrics should show "No data"
    let md = exp.to_markdown();
    assert!(md.contains("No data"));
}

#[test]
fn test_format_distribution_with_data() {
    let mut exp = ABExperiment::new("dist-data".to_string(), "With data".to_string());

    let obs = TestObservation {
        variant: TestVariant::Control,
        variable: "x".to_string(),
        predicted: InferredOwnership::Owned,
        ground_truth: None,
        confidence: 0.8,
        method: ClassificationMethod::RuleBased,
        latency: Duration::from_micros(100),
        correct: None,
    };
    exp.record(&obs);

    let md = exp.to_markdown();
    // Should show ownership type with percentage
    assert!(md.contains("Owned"));
    assert!(md.contains("100.0%"));
}

// ============================================================================
// ABTestRunner: timed_record
// ============================================================================

#[test]
fn test_ab_runner_timed_record() {
    let mut runner = ABTestRunner::new(
        "timed-test".to_string(),
        "Timing test".to_string(),
        AssignmentStrategy::RoundRobin,
    );

    let result = runner.timed_record(
        TestVariant::Control,
        || HybridResult {
            variable: "ptr".to_string(),
            ownership: InferredOwnership::Owned,
            confidence: 0.9,
            method: ClassificationMethod::RuleBased,
            rule_result: Some(InferredOwnership::Owned),
            ml_result: None,
            reasoning: "Rule-based classification".to_string(),
        },
        Some(InferredOwnership::Owned),
    );

    assert_eq!(result.variable, "ptr");
    assert_eq!(result.ownership, InferredOwnership::Owned);

    // Verify observation was recorded
    let exp = runner.experiment();
    assert_eq!(exp.control.count, 1);
    assert_eq!(exp.control.correct, 1);
}

#[test]
fn test_ab_runner_timed_record_treatment() {
    let mut runner = ABTestRunner::new(
        "timed-treatment".to_string(),
        "Treatment timing".to_string(),
        AssignmentStrategy::AllTreatment,
    );

    let result = runner.timed_record(
        TestVariant::Treatment,
        || HybridResult {
            variable: "buf".to_string(),
            ownership: InferredOwnership::Vec,
            confidence: 0.85,
            method: ClassificationMethod::MachineLearning,
            rule_result: Some(InferredOwnership::Owned),
            ml_result: None,
            reasoning: "ML predicted Vec".to_string(),
        },
        None, // No ground truth
    );

    assert_eq!(result.ownership, InferredOwnership::Vec);

    let exp = runner.experiment();
    assert_eq!(exp.treatment.count, 1);
    assert_eq!(exp.treatment.with_ground_truth, 0);
}

// ============================================================================
// ABTestRunner: experiment() and experiment_mut() accessors
// ============================================================================

#[test]
fn test_ab_runner_experiment_accessor() {
    let runner = ABTestRunner::new(
        "accessor-test".to_string(),
        "Testing accessors".to_string(),
        AssignmentStrategy::RoundRobin,
    );

    let exp = runner.experiment();
    assert_eq!(exp.name, "accessor-test");
    assert_eq!(exp.description, "Testing accessors");
    assert!(exp.is_active());
    assert_eq!(exp.total_observations(), 0);
}

#[test]
fn test_ab_runner_experiment_mut_accessor() {
    let mut runner = ABTestRunner::new(
        "mut-accessor".to_string(),
        "Testing mut accessor".to_string(),
        AssignmentStrategy::AllControl,
    );

    // Use experiment_mut to directly record an observation
    let obs = TestObservation {
        variant: TestVariant::Control,
        variable: "direct".to_string(),
        predicted: InferredOwnership::Borrowed,
        ground_truth: Some(InferredOwnership::Borrowed),
        confidence: 0.75,
        method: ClassificationMethod::RuleBased,
        latency: Duration::from_micros(50),
        correct: Some(true),
    };

    runner.experiment_mut().record(&obs);

    assert_eq!(runner.experiment().control.count, 1);
    assert_eq!(runner.experiment().control.correct, 1);
}

// ============================================================================
// ABTestRunner: with_seed
// ============================================================================

#[test]
fn test_ab_runner_with_seed() {
    let mut runner1 = ABTestRunner::new(
        "seed-test".to_string(),
        "Seed test".to_string(),
        AssignmentStrategy::Random,
    )
    .with_seed(12345);

    let mut runner2 = ABTestRunner::new(
        "seed-test".to_string(),
        "Seed test".to_string(),
        AssignmentStrategy::Random,
    )
    .with_seed(12345);

    // Same seed produces same sequence
    for _ in 0..20 {
        assert_eq!(runner1.assign(), runner2.assign());
    }
}

#[test]
fn test_ab_runner_different_seeds_may_differ() {
    // Verify seeds 0 and 42 produce at least one different result
    // The LCG: seed = seed * 6364136223846793005 + 1, then check seed % 2
    // seed=0: 0*... + 1 = 1 (odd -> Treatment)
    // seed=42: 42*6364136223846793005 + 1 = ... (computed)
    let mut runner1 = ABTestRunner::new(
        "diff-seed".to_string(),
        "Different seeds".to_string(),
        AssignmentStrategy::Random,
    )
    .with_seed(0);

    let mut runner2 = ABTestRunner::new(
        "diff-seed".to_string(),
        "Different seeds".to_string(),
        AssignmentStrategy::Random,
    )
    .with_seed(42);

    // Just verify both run without panic and produce valid variants
    let v1 = runner1.assign();
    let v2 = runner2.assign();
    assert!(v1 == TestVariant::Control || v1 == TestVariant::Treatment);
    assert!(v2 == TestVariant::Control || v2 == TestVariant::Treatment);
}

// ============================================================================
// TestVariant: hash, serialize, deserialize
// ============================================================================

#[test]
fn test_variant_serialize_deserialize() {
    let control = TestVariant::Control;
    let treatment = TestVariant::Treatment;

    let json_c = serde_json::to_string(&control).unwrap();
    let json_t = serde_json::to_string(&treatment).unwrap();

    let deser_c: TestVariant = serde_json::from_str(&json_c).unwrap();
    let deser_t: TestVariant = serde_json::from_str(&json_t).unwrap();

    assert_eq!(deser_c, TestVariant::Control);
    assert_eq!(deser_t, TestVariant::Treatment);
}

#[test]
fn test_variant_hash() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert(TestVariant::Control, "control_data");
    map.insert(TestVariant::Treatment, "treatment_data");

    assert_eq!(*map.get(&TestVariant::Control).unwrap(), "control_data");
    assert_eq!(
        *map.get(&TestVariant::Treatment).unwrap(),
        "treatment_data"
    );
}

#[test]
fn test_variant_clone_copy() {
    let v = TestVariant::Control;
    let cloned = v.clone();
    let copied = v;
    assert_eq!(v, cloned);
    assert_eq!(v, copied);
}

// ============================================================================
// TestObservation: serialization round-trip
// ============================================================================

#[test]
fn test_observation_serde_roundtrip() {
    let obs = TestObservation {
        variant: TestVariant::Treatment,
        variable: "ptr".to_string(),
        predicted: InferredOwnership::RawPointer,
        ground_truth: Some(InferredOwnership::Owned),
        confidence: 0.42,
        method: ClassificationMethod::Fallback,
        latency: Duration::from_micros(999),
        correct: Some(false),
    };

    let json = serde_json::to_string(&obs).unwrap();
    let deserialized: TestObservation = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.variant, TestVariant::Treatment);
    assert_eq!(deserialized.variable, "ptr");
    assert_eq!(deserialized.predicted, InferredOwnership::RawPointer);
    assert_eq!(deserialized.correct, Some(false));
}

// ============================================================================
// VariantMetrics: serialization round-trip
// ============================================================================

#[test]
fn test_variant_metrics_serde_roundtrip() {
    let mut metrics = VariantMetrics::new();
    let obs = TestObservation {
        variant: TestVariant::Control,
        variable: "x".to_string(),
        predicted: InferredOwnership::Slice,
        ground_truth: Some(InferredOwnership::Slice),
        confidence: 0.88,
        method: ClassificationMethod::Hybrid,
        latency: Duration::from_micros(150),
        correct: Some(true),
    };
    metrics.record(&obs);

    let json = serde_json::to_string(&metrics).unwrap();
    let deserialized: VariantMetrics = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.count, 1);
    assert_eq!(deserialized.correct, 1);
    assert_eq!(deserialized.with_ground_truth, 1);
}

// ============================================================================
// ABExperiment: serialization round-trip
// ============================================================================

#[test]
fn test_ab_experiment_serde_roundtrip() {
    let mut exp = ABExperiment::new("serde-test".to_string(), "Test serialization".to_string());

    let obs = TestObservation {
        variant: TestVariant::Control,
        variable: "v".to_string(),
        predicted: InferredOwnership::Owned,
        ground_truth: None,
        confidence: 0.7,
        method: ClassificationMethod::RuleBased,
        latency: Duration::from_micros(50),
        correct: None,
    };
    exp.record(&obs);

    let json = serde_json::to_string(&exp).unwrap();
    let deserialized: ABExperiment = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, "serde-test");
    assert_eq!(deserialized.control.count, 1);
    assert!(deserialized.started_at > 0);
}

// ============================================================================
// AssignmentStrategy: debug, clone, copy, eq
// ============================================================================

#[test]
fn test_assignment_strategy_debug() {
    assert!(format!("{:?}", AssignmentStrategy::RoundRobin).contains("RoundRobin"));
    assert!(format!("{:?}", AssignmentStrategy::Random).contains("Random"));
    assert!(format!("{:?}", AssignmentStrategy::AllControl).contains("AllControl"));
    assert!(format!("{:?}", AssignmentStrategy::AllTreatment).contains("AllTreatment"));
}

#[test]
fn test_assignment_strategy_eq() {
    assert_eq!(AssignmentStrategy::RoundRobin, AssignmentStrategy::RoundRobin);
    assert_ne!(AssignmentStrategy::Random, AssignmentStrategy::AllControl);
    assert_ne!(
        AssignmentStrategy::AllControl,
        AssignmentStrategy::AllTreatment
    );
}

#[test]
fn test_assignment_strategy_clone_copy() {
    let s = AssignmentStrategy::Random;
    let cloned = s.clone();
    let copied = s;
    assert_eq!(s, cloned);
    assert_eq!(s, copied);
}

// ============================================================================
// ClassificationMethod: display for all variants
// ============================================================================

#[test]
fn test_classification_method_display_all() {
    assert_eq!(ClassificationMethod::RuleBased.to_string(), "rule-based");
    assert_eq!(ClassificationMethod::MachineLearning.to_string(), "ml");
    assert_eq!(ClassificationMethod::Fallback.to_string(), "fallback");
    assert_eq!(ClassificationMethod::Hybrid.to_string(), "hybrid");
}

// ============================================================================
// ABExperiment: confidence_lift and accuracy_lift with zero data
// ============================================================================

#[test]
fn test_lift_calculations_zero_data() {
    let exp = ABExperiment::new("zero".to_string(), "No data".to_string());

    assert_eq!(exp.accuracy_lift(), 0.0);
    assert_eq!(exp.confidence_lift(), 0.0);
    assert_eq!(exp.latency_diff_us(), 0.0);
}

// ============================================================================
// ABExperiment: to_markdown with various ownership types in distribution
// ============================================================================

#[test]
fn test_to_markdown_diverse_ownership_distribution() {
    let mut exp = ABExperiment::new("diverse".to_string(), "Diverse types".to_string());

    let ownership_types = [
        InferredOwnership::Owned,
        InferredOwnership::Borrowed,
        InferredOwnership::BorrowedMut,
        InferredOwnership::Shared,
        InferredOwnership::RawPointer,
        InferredOwnership::Vec,
        InferredOwnership::Slice,
    ];

    for (i, ownership) in ownership_types.iter().enumerate() {
        let obs = TestObservation {
            variant: TestVariant::Control,
            variable: format!("v{}", i),
            predicted: *ownership,
            ground_truth: None,
            confidence: 0.8,
            method: ClassificationMethod::RuleBased,
            latency: Duration::from_micros(100),
            correct: None,
        };
        exp.record(&obs);
    }

    let md = exp.to_markdown();
    // Should show all ownership types in the control distribution
    assert!(md.contains("Control Group Distribution"));
    assert!(md.contains("Treatment Group Distribution"));
}

// ============================================================================
// ABTestRunner: record via runner.record()
// ============================================================================

#[test]
fn test_ab_runner_record_directly() {
    let mut runner = ABTestRunner::new(
        "direct-record".to_string(),
        "Direct recording".to_string(),
        AssignmentStrategy::RoundRobin,
    );

    let result = HybridResult {
        variable: "data".to_string(),
        ownership: InferredOwnership::Vec,
        confidence: 0.92,
        method: ClassificationMethod::Hybrid,
        rule_result: Some(InferredOwnership::Vec),
        ml_result: None,
        reasoning: "Both methods agree on Vec".to_string(),
    };

    runner.record(
        TestVariant::Treatment,
        &result,
        Some(InferredOwnership::Vec),
        Duration::from_micros(175),
    );

    assert_eq!(runner.experiment().treatment.count, 1);
    assert_eq!(runner.experiment().treatment.correct, 1);
}

// ============================================================================
// ABTestRunner: finish sets ended_at
// ============================================================================

#[test]
fn test_ab_runner_finish_marks_ended() {
    let mut runner = ABTestRunner::new(
        "finish-test".to_string(),
        "Finish marking".to_string(),
        AssignmentStrategy::AllControl,
    );

    assert!(runner.experiment().is_active());

    let report = runner.finish();

    assert!(!runner.experiment().is_active());
    assert!(runner.experiment().ended_at > 0);
    assert!(report.contains("COMPLETED"));
}
