//! Classifier Demo - Demonstrates rule-based and ensemble classification
//!
//! Run with: cargo run --example classifier_demo -p decy-ownership

use decy_ownership::{
    AllocationKind, ClassifierEvaluator, EnsembleClassifier, InferredOwnership,
    OwnershipClassifier, OwnershipFeaturesBuilder, RuleBasedClassifier, TrainingSample,
};

fn main() {
    println!("=== Decy Classifier Demo ===\n");

    // Demo 1: Rule-Based Classification
    println!("## Rule-Based Classifier\n");
    demo_rule_based();

    // Demo 2: Ensemble Classification
    println!("\n## Ensemble Classifier\n");
    demo_ensemble();

    // Demo 3: Evaluation Metrics
    println!("\n## Classifier Evaluation\n");
    demo_evaluation();

    println!("\n=== Demo Complete ===");
}

fn demo_rule_based() {
    let classifier = RuleBasedClassifier::new();

    println!("Classifier: {}", classifier.name());
    println!("Is trained: {}\n", classifier.is_trained());

    // Test various patterns
    let test_cases = vec![
        ("malloc allocation", AllocationKind::Malloc, 1, false, 0),
        ("stack allocation", AllocationKind::Stack, 0, false, 0),
        ("const parameter", AllocationKind::Parameter, 0, true, 0),
        ("mutable parameter", AllocationKind::Parameter, 0, false, 1),
    ];

    for (name, alloc, dealloc, is_const, writes) in test_cases {
        let features = OwnershipFeaturesBuilder::default()
            .pointer_depth(1)
            .allocation_site(alloc)
            .deallocation_count(dealloc)
            .const_qualified(is_const)
            .write_count(writes)
            .build();

        let result = classifier.classify(&features);
        println!(
            "  {} -> {:?} (confidence: {:.2})",
            name, result.prediction, result.confidence
        );
    }
}

fn demo_ensemble() {
    let mut ensemble = EnsembleClassifier::new("weighted_ensemble");

    // Add multiple classifiers with different weights
    ensemble.add_classifier(RuleBasedClassifier::new(), 1.0);
    ensemble.add_classifier(RuleBasedClassifier::new(), 0.5);

    println!("Ensemble: {}", ensemble.name());
    println!("Classifiers: {}", ensemble.classifier_count());
    println!("Is trained: {}\n", ensemble.is_trained());

    // Test ensemble classification
    let features = OwnershipFeaturesBuilder::default()
        .pointer_depth(1)
        .allocation_site(AllocationKind::Malloc)
        .deallocation_count(1)
        .build();

    let result = ensemble.classify(&features);
    println!(
        "Ensemble prediction: {:?} (confidence: {:.2})",
        result.prediction, result.confidence
    );
}

fn demo_evaluation() {
    let classifier = RuleBasedClassifier::new();

    // Create test samples
    let samples = vec![
        // Correct predictions
        TrainingSample::new(
            OwnershipFeaturesBuilder::default()
                .allocation_site(AllocationKind::Malloc)
                .deallocation_count(1)
                .build(),
            InferredOwnership::Owned,
            "test1.c",
            1,
        ),
        TrainingSample::new(
            OwnershipFeaturesBuilder::default()
                .const_qualified(true)
                .allocation_site(AllocationKind::Parameter)
                .build(),
            InferredOwnership::Borrowed,
            "test2.c",
            2,
        ),
        TrainingSample::new(
            OwnershipFeaturesBuilder::default()
                .write_count(1)
                .allocation_site(AllocationKind::Parameter)
                .build(),
            InferredOwnership::BorrowedMut,
            "test3.c",
            3,
        ),
        // This might be incorrect (depends on classifier)
        TrainingSample::new(
            OwnershipFeaturesBuilder::default()
                .pointer_depth(2)
                .build(),
            InferredOwnership::Owned, // Might be RawPointer
            "test4.c",
            4,
        ),
    ];

    let evaluator = ClassifierEvaluator::new(samples);
    let metrics = evaluator.evaluate(&classifier);

    println!("Evaluation on {} samples:", evaluator.sample_count());
    println!("  - Accuracy: {:.1}%", metrics.accuracy() * 100.0);
    println!("  - Correct: {}", metrics.correct);
    println!("  - Macro F1: {:.2}", metrics.macro_f1());

    // Per-class metrics
    for class in ["Owned", "Borrowed", "BorrowedMut"] {
        if metrics.true_positives.contains_key(class) || metrics.false_positives.contains_key(class)
        {
            println!(
                "  - {} precision: {:.2}, recall: {:.2}",
                class,
                metrics.precision(class),
                metrics.recall(class)
            );
        }
    }
}
