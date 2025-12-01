# ML-Enhanced Ownership Inference

Decy includes ML-enhanced ownership inference that combines rule-based heuristics with machine learning for improved accuracy.

## Overview

The ML system operates in a hybrid mode:
1. **Rule-based classifier** provides deterministic baseline
2. **ML classifier** handles complex patterns
3. **Fallback logic** uses rules when ML confidence is low (<0.65)

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Feature Extraction                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
│  │ Syntactic   │  │ Semantic    │  │ Context         │ │
│  │ - ptr depth │  │ - malloc    │  │ - func name     │ │
│  │ - const     │  │ - free      │  │ - var name      │ │
│  │ - array     │  │ - aliases   │  │ - types         │ │
│  └─────────────┘  └─────────────┘  └─────────────────┘ │
│                         │                               │
│                         ▼                               │
│              ┌───────────────────────┐                  │
│              │  Hybrid Classifier    │                  │
│              │  ┌─────────┐ ┌─────┐  │                  │
│              │  │ Rules   │ │ ML  │  │                  │
│              │  └────┬────┘ └──┬──┘  │                  │
│              │       │   ▼     │     │                  │
│              │       └─►Merge◄─┘     │                  │
│              └───────────┬───────────┘                  │
│                          │                              │
│                          ▼                              │
│              Ownership Prediction + Confidence          │
└─────────────────────────────────────────────────────────┘
```

## Feature Extraction

### OwnershipFeatures (142 dimensions)

```rust,ignore
use decy_ownership::{OwnershipFeatures, OwnershipFeaturesBuilder, AllocationKind};

let features = OwnershipFeaturesBuilder::default()
    .pointer_depth(1)                          // int* vs int**
    .const_qualified(true)                     // const modifier
    .allocation_site(AllocationKind::Malloc)   // malloc, stack, etc.
    .deallocation_count(1)                     // free() calls
    .has_size_param(true)                      // (ptr, size) pattern
    .array_decay(true)                         // T[] → T*
    .write_count(0)                            // write operations
    .read_count(5)                             // read operations
    .build();

// Convert to ML-ready vector
let vector = features.to_vector();  // Vec<f32> with 142 dimensions
```

### Key Feature Categories

| Category | Features | Purpose |
|----------|----------|---------|
| **Syntactic** | pointer_depth, is_const, is_array_decay | Surface-level patterns |
| **Semantic** | allocation_site, deallocation_count, alias_count | Dataflow analysis |
| **Usage** | read_count, write_count, arithmetic_ops | How pointer is used |
| **Context** | function_name_embedding, surrounding_types | Semantic context |

## Hybrid Classification

### Using the Hybrid Classifier

```rust,ignore
use decy_ownership::{
    HybridClassifier, OwnershipModel, NullModel,
    OwnershipFeaturesBuilder, AllocationKind,
};

// Create classifier (uses rule-based by default)
let mut classifier = HybridClassifier::default();

// Build features
let features = OwnershipFeaturesBuilder::default()
    .allocation_site(AllocationKind::Malloc)
    .deallocation_count(1)
    .pointer_depth(1)
    .build();

// Classify
let result = classifier.classify(&features);

println!("Prediction: {:?}", result.ownership);
println!("Confidence: {:.2}", result.confidence);
println!("Method: {:?}", result.method);  // RuleBased, ML, or Ensemble
```

### Classification Results

```rust,ignore
use decy_ownership::InferredOwnership;

// Possible ownership kinds
match result.ownership {
    InferredOwnership::Owned => println!("Box<T>"),
    InferredOwnership::Borrowed => println!("&T"),
    InferredOwnership::BorrowedMut => println!("&mut T"),
    InferredOwnership::Vec => println!("Vec<T>"),
    InferredOwnership::Slice => println!("&[T]"),
    InferredOwnership::SliceMut => println!("&mut [T]"),
    InferredOwnership::Shared => println!("Rc<T> or Arc<T>"),
    InferredOwnership::RawPointer => println!("*const T or *mut T"),
}
```

## Training Data Collection

### Synthetic Data Generation

```rust,ignore
use decy_ownership::{
    SyntheticDataGenerator, SyntheticConfig, TrainingDataset,
};

// Configure generator
let config = SyntheticConfig {
    samples_per_pattern: 200,  // 200 samples per pattern
    seed: 42,                   // Reproducible
    include_edge_cases: true,
};

// Generate dataset
let generator = SyntheticDataGenerator::new(config);
let dataset = generator.generate_full_dataset();

println!("Generated {} samples", dataset.len());

// Check dataset statistics
let stats = dataset.stats();
println!("Label distribution: {:?}", stats.label_distribution);
println!("Is balanced: {}", stats.is_balanced());
```

### Synthetic Patterns

| Pattern | C Code | Rust Type | Label |
|---------|--------|-----------|-------|
| malloc/free | `int* p = malloc(4); free(p);` | `Box<i32>` | Owned |
| array alloc | `int* arr = malloc(n * sizeof(int));` | `Vec<i32>` | Vec |
| const ptr | `void f(const int* p)` | `&i32` | Borrowed |
| mutable ptr | `void f(int* p) { *p = 0; }` | `&mut i32` | BorrowedMut |
| array + size | `void f(const int* arr, size_t n)` | `&[i32]` | Slice |

## Classifier Training

### Rule-Based Baseline

```rust,ignore
use decy_ownership::{
    RuleBasedClassifier, OwnershipClassifier,
    ClassifierEvaluator, TrainingDataset,
};

// Create rule-based classifier (always "trained")
let classifier = RuleBasedClassifier::new();

// Evaluate on dataset
let evaluator = ClassifierEvaluator::from_dataset(&dataset);
let metrics = evaluator.evaluate(&classifier);

println!("Accuracy: {:.2}%", metrics.accuracy() * 100.0);
println!("Macro F1: {:.2}", metrics.macro_f1());
println!("Owned precision: {:.2}", metrics.precision("Owned"));
println!("Owned recall: {:.2}", metrics.recall("Owned"));
```

### Training Pipeline

```rust,ignore
use decy_ownership::{
    ClassifierTrainer, TrainingConfig, TrainingDataset,
};

// Configure training
let config = TrainingConfig {
    validation_split: 0.2,
    random_seed: 42,
    max_iterations: 100,
    early_stopping_patience: 10,
    min_improvement: 0.001,
};

// Train classifier
let trainer = ClassifierTrainer::new(config);
let (classifier, result) = trainer.train_rule_based(&dataset);

if result.success {
    println!("Training completed in {:.2}s", result.duration_secs);
    println!("Validation accuracy: {:.2}%",
             result.validation_metrics.accuracy() * 100.0);
}
```

## Model Versioning

### Version Management

```rust,ignore
use decy_ownership::{
    ModelVersionManager, ModelVersion, ModelQualityMetrics, ModelEntry,
};

let mut manager = ModelVersionManager::new();

// Register a new model version
let metrics = ModelQualityMetrics::new(
    0.92,  // accuracy
    0.90,  // precision
    0.88,  // recall
    0.89,  // f1_score
    0.85,  // avg_confidence
    0.05,  // fallback_rate
    1000,  // sample_count
);

let entry = ModelEntry::new(
    ModelVersion::new(1, 0, 0),
    metrics,
    "Initial model",
    "models/v1.0.0.bin",
);

let activated = manager.register_version(entry)?;
println!("Model activated: {}", activated);
```

### Rollback on Degradation

```rust,ignore
// Check if rollback is needed
if let Some(current) = manager.active_version() {
    if new_metrics.f1_score < current.metrics.f1_score - 0.02 {
        // Rollback to previous version
        let result = manager.rollback("Performance degradation")?;
        println!("Rolled back from {} to {}",
                 result.from_version, result.to_version);
    }
}
```

## Active Learning

### Collecting Uncertain Samples

```rust,ignore
use decy_ownership::{
    ActiveLearner, SelectionStrategy, UncertainSample,
};

// Create active learner
let mut learner = ActiveLearner::new(SelectionStrategy::Uncertainty);

// Record prediction
let sample = UncertainSample::new(
    "ptr",              // variable name
    "source.c",         // file
    42,                 // line
    features.clone(),
    0.55,               // confidence (uncertain!)
);
learner.record_sample(sample);

// Get samples for labeling (lowest confidence)
let to_label = learner.get_samples_for_labeling(10);
println!("Need {} samples labeled", to_label.len());
```

## Error Tracking (CITL)

### Recording Inference Errors

```rust,ignore
use decy_ownership::{
    ErrorTracker, InferenceError, OwnershipDefect, InferredOwnership,
};

let mut tracker = ErrorTracker::new();

// Record an error
let error = InferenceError::new(
    "ptr",
    "source.c",
    42,
    InferredOwnership::Borrowed,  // predicted
    InferredOwnership::Owned,     // actual
    0.6,
    OwnershipDefect::PointerMisclassification,
).with_features(vec!["malloc_free".to_string()]);

tracker.record_error(error);
```

### Tarantula Fault Localization

```rust,ignore
// Calculate suspiciousness of features
let suspicious = tracker.top_suspicious(5);

for fs in suspicious {
    println!("Feature '{}': score={:.2}, failures={}",
             fs.feature, fs.score, fs.failure_count);
}

// Generate improvement suggestions
let suggestions = tracker.generate_suggestions();
for suggestion in suggestions {
    println!("[{:?}] {}", suggestion.priority, suggestion.description);
}
```

## Retraining Pipeline

### Weekly Retraining

```rust,ignore
use decy_ownership::{
    RetrainingPipeline, RetrainingConfig, NullTrainer,
    TrainingSample, RetrainingResult,
};

// Configure pipeline
let config = RetrainingConfig {
    min_precision: 0.85,
    min_recall: 0.80,
    max_degradation: 0.02,  // Max 2% regression allowed
    min_train_samples: 700,
    ..Default::default()
};

// Create pipeline
let trainer = NullTrainer::new(0.90, 0.85);
let mut pipeline = RetrainingPipeline::with_defaults(trainer);

// Execute retraining
let samples: Vec<TrainingSample> = collect_samples();
let result = pipeline.execute(samples);

match result {
    RetrainingResult::Promoted { version, metrics } => {
        println!("Model promoted to {}", version);
        println!("F1 score: {:.2}", metrics.f1_score);
    }
    RetrainingResult::QualityGateFailed { reason, .. } => {
        println!("Quality gate failed: {}", reason);
    }
    RetrainingResult::Degraded { degradation, .. } => {
        println!("Model degraded by {:.1}%", degradation * 100.0);
    }
    _ => {}
}
```

## A/B Testing

### Comparing Rule-Based vs ML

```rust,ignore
use decy_ownership::{
    ABTestRunner, ABExperiment, TestVariant, AssignmentStrategy,
};

// Create experiment
let experiment = ABExperiment::new("rules_vs_ml")
    .with_control(TestVariant::new("rules", "Rule-based classifier"))
    .with_treatment(TestVariant::new("ml", "ML classifier"));

// Run test
let mut runner = ABTestRunner::new(experiment, AssignmentStrategy::Random);

// Record observations
runner.record_observation("rules", true, 0.95);   // correct, confident
runner.record_observation("ml", true, 0.88);      // correct, less confident
runner.record_observation("rules", false, 0.60);  // incorrect

// Analyze results
let analysis = runner.analyze();
println!("Control accuracy: {:.2}%", analysis.control_accuracy * 100.0);
println!("Treatment accuracy: {:.2}%", analysis.treatment_accuracy * 100.0);
println!("P-value: {:.4}", analysis.p_value);
println!("Significant: {}", analysis.is_significant);
```

## Running Examples

To run ML feature examples:

```bash
# Run all examples
cargo run --example ml_features_demo -p decy-ownership

# Run specific examples
cargo run --example classifier_demo -p decy-ownership
cargo run --example training_demo -p decy-ownership
cargo run --example hybrid_demo -p decy-ownership
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DECY_ML_ENABLED` | `true` | Enable ML classification |
| `DECY_ML_THRESHOLD` | `0.65` | Confidence threshold for ML |
| `DECY_ML_MODEL_PATH` | `~/.decy/models` | Model storage path |

### Quality Thresholds

```rust,ignore
use decy_ownership::QualityThresholds;

let thresholds = QualityThresholds {
    min_accuracy: 0.85,
    min_precision: 0.80,
    min_recall: 0.75,
    min_f1_score: 0.80,
    max_degradation: 0.02,
};
```

## Next Steps

- [Oracle Integration (CITL)](./oracle.md) - Compiler-in-the-loop learning
- [Debugging](./debugging.md) - Inspecting ownership inference
