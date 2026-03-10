//! ML Features Demo - Demonstrates feature extraction and classification
//!
//! Run with: cargo run --example ml_features_demo -p decy-ownership

use decy_ownership::{
    AllocationKind, InferredOwnership, OwnershipClassifier, OwnershipFeaturesBuilder,
    RuleBasedClassifier,
};

fn main() {
    println!("=== Decy ML Features Demo ===\n");

    // Demo 1: Feature Extraction
    println!("## Feature Extraction\n");
    demo_feature_extraction();

    // Demo 2: Rule-Based Classification
    println!("\n## Rule-Based Classification\n");
    demo_rule_based_classification();

    // Demo 3: Different Ownership Patterns
    println!("\n## Ownership Pattern Recognition\n");
    demo_ownership_patterns();

    println!("\n=== Demo Complete ===");
}

fn demo_feature_extraction() {
    // Create features for a malloc pointer
    let malloc_features = OwnershipFeaturesBuilder::default()
        .pointer_depth(1)
        .allocation_site(AllocationKind::Malloc)
        .deallocation_count(1)
        .build();

    println!("Malloc pointer features:");
    println!("  - Pointer depth: {}", malloc_features.pointer_depth);
    println!("  - Allocation: {:?}", malloc_features.allocation_site);
    println!("  - Deallocation count: {}", malloc_features.deallocation_count);

    // Convert to ML-ready vector
    let vector = malloc_features.to_vector();
    println!("  - Feature vector dimension: {}", vector.len());
    println!("  - First 5 values: {:?}", &vector[..5.min(vector.len())]);
}

fn demo_rule_based_classification() {
    let classifier = RuleBasedClassifier::new();

    // Classify a malloc pattern
    let features = OwnershipFeaturesBuilder::default()
        .pointer_depth(1)
        .allocation_site(AllocationKind::Malloc)
        .deallocation_count(1)
        .build();

    let result = classifier.classify(&features);

    println!("Input: malloc + free pattern");
    println!("Result:");
    println!("  - Ownership: {:?}", result.prediction);
    println!("  - Confidence: {:.2}", result.confidence);
}

fn demo_ownership_patterns() {
    let classifier = RuleBasedClassifier::new();

    let patterns = vec![
        (
            "malloc + free",
            OwnershipFeaturesBuilder::default()
                .pointer_depth(1)
                .allocation_site(AllocationKind::Malloc)
                .deallocation_count(1)
                .build(),
        ),
        (
            "const pointer parameter",
            OwnershipFeaturesBuilder::default()
                .pointer_depth(1)
                .const_qualified(true)
                .allocation_site(AllocationKind::Parameter)
                .build(),
        ),
        (
            "mutable pointer with writes",
            OwnershipFeaturesBuilder::default()
                .pointer_depth(1)
                .const_qualified(false)
                .write_count(1)
                .allocation_site(AllocationKind::Parameter)
                .build(),
        ),
        (
            "array allocation",
            OwnershipFeaturesBuilder::default()
                .pointer_depth(1)
                .allocation_site(AllocationKind::Malloc)
                .has_size_param(true)
                .array_decay(true)
                .deallocation_count(1)
                .build(),
        ),
        (
            "slice parameter",
            OwnershipFeaturesBuilder::default()
                .pointer_depth(1)
                .const_qualified(true)
                .array_decay(true)
                .has_size_param(true)
                .allocation_site(AllocationKind::Parameter)
                .build(),
        ),
    ];

    println!("| Pattern | Ownership | Confidence |");
    println!("|---------|-----------|------------|");

    for (name, features) in patterns {
        let result = classifier.classify(&features);
        let ownership_str = match result.prediction {
            InferredOwnership::Owned => "Box<T>",
            InferredOwnership::Borrowed => "&T",
            InferredOwnership::BorrowedMut => "&mut T",
            InferredOwnership::Vec => "Vec<T>",
            InferredOwnership::Slice => "&[T]",
            InferredOwnership::SliceMut => "&mut [T]",
            InferredOwnership::Shared => "Rc<T>",
            InferredOwnership::RawPointer => "*T",
        };
        println!("| {:<25} | {:<9} | {:.2} |", name, ownership_str, result.confidence);
    }
}
