//! Hybrid Demo - Demonstrates hybrid ML + rules classification
//!
//! Run with: cargo run --example hybrid_demo -p decy-ownership

use decy_ownership::{
    AllocationKind, ClassificationMethod, HybridClassifier, HybridMetrics, NullModel,
    OwnershipFeaturesBuilder, OwnershipInference, OwnershipKind, DEFAULT_CONFIDENCE_THRESHOLD,
};

fn main() {
    println!("=== Decy Hybrid Classification Demo ===\n");

    // Demo 1: Basic Hybrid Classification
    println!("## Basic Classification\n");
    demo_basic_classification();

    // Demo 2: Confidence Thresholds
    println!("\n## Confidence Thresholds\n");
    demo_thresholds();

    // Demo 3: Classification Metrics
    println!("\n## Classification Metrics\n");
    demo_metrics();

    // Demo 4: ML Enable/Disable
    println!("\n## ML Enable/Disable\n");
    demo_ml_toggle();

    println!("\n=== Demo Complete ===");
}

fn demo_basic_classification() {
    let classifier = HybridClassifier::default();

    println!(
        "Default confidence threshold: {}",
        DEFAULT_CONFIDENCE_THRESHOLD
    );
    println!("ML enabled: {}\n", classifier.ml_enabled());

    // Create an OwnershipInference (simulating what the rule engine would produce)
    let inference = OwnershipInference {
        variable: "ptr".to_string(),
        kind: OwnershipKind::Owning,
        confidence: 0.9,
        reason: "malloc detected with corresponding free".to_string(),
    };

    let result = classifier.classify_rule_based(&inference);

    println!("malloc + free pattern:");
    println!("  - Ownership: {:?}", result.ownership);
    println!("  - Confidence: {:.2}", result.confidence);
    println!(
        "  - Method: {}",
        match result.method {
            ClassificationMethod::RuleBased => "Rule-based",
            ClassificationMethod::MachineLearning => "ML",
            ClassificationMethod::Fallback => "Fallback",
            ClassificationMethod::Hybrid => "Hybrid",
        }
    );
    println!("  - Used fallback: {}", result.used_fallback());
}

fn demo_thresholds() {
    // Test different thresholds
    let thresholds = vec![0.5, 0.65, 0.8, 0.95];

    let inference = OwnershipInference {
        variable: "ptr".to_string(),
        kind: OwnershipKind::ImmutableBorrow,
        confidence: 0.85,
        reason: "const pointer parameter".to_string(),
    };

    let features = OwnershipFeaturesBuilder::default()
        .pointer_depth(1)
        .const_qualified(true)
        .allocation_site(AllocationKind::Parameter)
        .build();

    let model = NullModel; // Null model returns 0.0 confidence

    println!("Testing const parameter pattern with different thresholds:\n");
    println!("| Threshold | Method | Ownership | Confidence |");
    println!("|-----------|--------|-----------|------------|");

    for threshold in thresholds {
        let mut classifier = HybridClassifier::with_threshold(threshold);
        classifier.enable_ml(); // Enable ML to test threshold behavior
        let result = classifier.classify_hybrid(&inference, &features, &model);

        println!(
            "| {:.2}      | {:<8} | {:?}  | {:.2}       |",
            threshold,
            match result.method {
                ClassificationMethod::RuleBased => "Rules",
                ClassificationMethod::MachineLearning => "ML",
                ClassificationMethod::Fallback => "Fallback",
                ClassificationMethod::Hybrid => "Hybrid",
            },
            result.ownership,
            result.confidence
        );
    }
}

fn demo_metrics() {
    let classifier = HybridClassifier::default();
    let mut metrics = HybridMetrics::new();

    // Classify several patterns to build up metrics
    let inferences = vec![
        OwnershipInference {
            variable: "ptr1".to_string(),
            kind: OwnershipKind::Owning,
            confidence: 0.95,
            reason: "malloc with free".to_string(),
        },
        OwnershipInference {
            variable: "ptr2".to_string(),
            kind: OwnershipKind::ImmutableBorrow,
            confidence: 0.85,
            reason: "const parameter".to_string(),
        },
        OwnershipInference {
            variable: "ptr3".to_string(),
            kind: OwnershipKind::MutableBorrow,
            confidence: 0.80,
            reason: "mutated parameter".to_string(),
        },
        OwnershipInference {
            variable: "ptr4".to_string(),
            kind: OwnershipKind::ArrayPointer {
                base_array: "arr".to_string(),
                element_type: decy_hir::HirType::Int,
                base_index: Some(0),
            },
            confidence: 0.90,
            reason: "array allocation".to_string(),
        },
        OwnershipInference {
            variable: "ptr5".to_string(),
            kind: OwnershipKind::Unknown,
            confidence: 0.30,
            reason: "unknown pattern".to_string(),
        },
    ];

    println!("Classifying {} patterns...\n", inferences.len());

    for inference in &inferences {
        let result = classifier.classify_rule_based(inference);
        metrics.record(&result);
    }

    println!("Hybrid Metrics:");
    println!("  - Total classifications: {}", metrics.total);
    println!("  - Rule-based used: {}", metrics.rule_based);
    println!("  - ML used: {}", metrics.ml_used);
    println!("  - Fallbacks: {}", metrics.fallback);
    println!("  - Fallback rate: {:.1}%", metrics.fallback_rate() * 100.0);
    println!(
        "  - Agreement rate: {:.1}%",
        metrics.agreement_rate() * 100.0
    );
}

fn demo_ml_toggle() {
    let mut classifier = HybridClassifier::default();

    let inference = OwnershipInference {
        variable: "ptr".to_string(),
        kind: OwnershipKind::Owning,
        confidence: 0.9,
        reason: "malloc detected".to_string(),
    };

    let features = OwnershipFeaturesBuilder::default()
        .pointer_depth(1)
        .allocation_site(AllocationKind::Malloc)
        .deallocation_count(1)
        .build();

    let model = NullModel;

    // ML disabled (default)
    println!("With ML disabled (default):");
    let result1 = classifier.classify_hybrid(&inference, &features, &model);
    println!(
        "  - Method: {:?}, Confidence: {:.2}",
        result1.method, result1.confidence
    );

    // ML enabled
    println!("\nWith ML enabled:");
    classifier.enable_ml();
    let result2 = classifier.classify_hybrid(&inference, &features, &model);
    println!(
        "  - Method: {:?}, Confidence: {:.2}",
        result2.method, result2.confidence
    );

    // Both should produce same ownership for clear-cut cases
    println!("\nSame result: {}", result1.ownership == result2.ownership);
}
