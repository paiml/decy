//! Training Demo - Demonstrates training data generation and model training
//!
//! Run with: cargo run --example training_demo -p decy-ownership

use decy_ownership::{
    ClassifierEvaluator, ClassifierTrainer, SyntheticConfig, SyntheticDataGenerator,
    TrainingConfig, TrainingDataCollector,
};

fn main() {
    println!("=== Decy Training Demo ===\n");

    // Demo 1: Synthetic Data Generation
    println!("## Synthetic Data Generation\n");
    let dataset = demo_synthetic_generation();

    // Demo 2: Dataset Statistics
    println!("\n## Dataset Statistics\n");
    demo_dataset_stats(&dataset);

    // Demo 3: Training Pipeline
    println!("\n## Training Pipeline\n");
    demo_training(&dataset);

    // Demo 4: Data Collection
    println!("\n## Data Collection\n");
    demo_data_collection();

    println!("\n=== Demo Complete ===");
}

fn demo_synthetic_generation() -> decy_ownership::TrainingDataset {
    // Configure synthetic data generator
    let config = SyntheticConfig {
        samples_per_pattern: 50, // 50 samples per pattern
        seed: 42,                // Reproducible
        include_edge_cases: true,
    };

    let generator = SyntheticDataGenerator::new(config);

    // Generate samples for each pattern
    println!("Generating synthetic training data...\n");

    let malloc_samples = generator.generate_malloc_box_samples();
    println!("  malloc/free -> Box<T>: {} samples", malloc_samples.len());

    let array_samples = generator.generate_array_vec_samples();
    println!("  array alloc -> Vec<T>: {} samples", array_samples.len());

    let const_samples = generator.generate_const_ref_samples();
    println!("  const ptr -> &T: {} samples", const_samples.len());

    let mut_samples = generator.generate_mut_ref_samples();
    println!("  mut ptr -> &mut T: {} samples", mut_samples.len());

    let slice_samples = generator.generate_slice_samples();
    println!("  slice param -> &[T]: {} samples", slice_samples.len());

    // Generate full dataset
    let dataset = generator.generate_full_dataset();
    println!("\nTotal samples: {}", dataset.len());

    dataset
}

fn demo_dataset_stats(dataset: &decy_ownership::TrainingDataset) {
    let stats = dataset.stats();

    println!("Total samples: {}", stats.total_samples);
    println!("Average confidence: {:.2}", stats.avg_confidence);
    println!("Min confidence: {:.2}", stats.min_confidence);
    println!("Max confidence: {:.2}", stats.max_confidence);

    println!("\nLabel distribution:");
    let mut labels: Vec<_> = stats.label_distribution.iter().collect();
    labels.sort_by(|a, b| b.1.cmp(a.1));
    for (label, count) in labels {
        let pct = (*count as f64 / stats.total_samples as f64) * 100.0;
        println!("  {}: {} ({:.1}%)", label, count, pct);
    }

    println!("\nSource distribution:");
    for (source, count) in &stats.source_distribution {
        println!("  {}: {}", source, count);
    }

    println!("\nDataset balanced: {}", stats.is_balanced());
    if let Some(dominant) = stats.dominant_label() {
        println!("Dominant label: {}", dominant);
    }
}

fn demo_training(dataset: &decy_ownership::TrainingDataset) {
    // Configure training
    let config = TrainingConfig {
        validation_split: 0.2,
        random_seed: 42,
        max_iterations: 100,
        early_stopping_patience: 10,
        min_improvement: 0.001,
    };

    println!("Training configuration:");
    println!("  - Validation split: {:.0}%", config.validation_split * 100.0);
    println!("  - Random seed: {}", config.random_seed);
    println!("  - Max iterations: {}", config.max_iterations);

    // Train rule-based classifier
    let trainer = ClassifierTrainer::new(config);
    let (classifier, result) = trainer.train_rule_based(dataset);

    println!("\nTraining result:");
    println!("  - Success: {}", result.success);
    println!("  - Duration: {:.3}s", result.duration_secs);
    println!("  - Iterations: {}", result.iterations);

    // Evaluate on full dataset
    let evaluator = ClassifierEvaluator::from_dataset(dataset);
    let metrics = evaluator.evaluate(&classifier);

    println!("\nEvaluation metrics:");
    println!("  - Accuracy: {:.1}%", metrics.accuracy() * 100.0);
    println!("  - Macro F1: {:.2}", metrics.macro_f1());
}

fn demo_data_collection() {
    let mut collector = TrainingDataCollector::new();

    // Add synthetic data
    let config = SyntheticConfig {
        samples_per_pattern: 20,
        seed: 123,
        include_edge_cases: true,
    };
    let generator = SyntheticDataGenerator::new(config);
    collector.add_synthetic(&generator);

    println!("Collected {} samples", collector.sample_count());

    // Simulate some errors
    collector.record_error("Failed to parse file: complex.c");
    collector.record_error("Unknown allocation pattern in legacy.c:42");

    println!("Recorded {} errors", collector.error_count());

    // Build final dataset
    let dataset = collector.build("demo_dataset", "1.0.0");

    println!("\nFinal dataset:");
    println!("  - Name: {}", dataset.name());
    println!("  - Version: {}", dataset.version());
    println!("  - Samples: {}", dataset.len());

    // Get collection result
    let result = TrainingDataCollector::new().result();
    println!("\nCollection result:");
    println!("  - Samples collected: {}", result.samples_collected);
    println!("  - Errors: {}", result.errors.len());
}
