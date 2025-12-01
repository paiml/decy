//! Training data collection and management (DECY-ML-010).
//!
//! Provides infrastructure for collecting, storing, and managing
//! labeled training data for ML-enhanced ownership inference.
//!
//! # Data Sources
//!
//! 1. **C Projects with Rust Ports**: Extract ground truth from
//!    established C→Rust migrations (Linux kernel, SQLite, curl)
//!
//! 2. **Compiler Error Feedback Loop (CITL)**: Learn from rustc
//!    errors on generated code to identify correct ownership patterns
//!
//! 3. **Synthetic Generation**: Use templates to generate labeled
//!    C→Rust pairs with known ownership patterns
//!
//! # Toyota Way Principles
//!
//! - **Genchi Genbutsu**: Ground truth from real-world code
//! - **Kaizen**: Continuous data collection improves model

use std::collections::HashMap;
use std::path::PathBuf;

use crate::ml_features::{AllocationKind, InferredOwnership, OwnershipFeaturesBuilder};
use crate::retraining_pipeline::TrainingSample;

/// Source of training data.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataSource {
    /// From open-source C→Rust migration (e.g., "rusqlite", "linux-rust").
    RustPort {
        /// Name of the project.
        project: String,
    },
    /// From compiler error feedback loop.
    CompilerFeedback {
        /// Error code (e.g., "E0382" for use after move).
        error_code: String,
    },
    /// Synthetically generated with known pattern.
    Synthetic {
        /// Pattern template name.
        template: String,
    },
    /// Manually annotated by human expert.
    HumanAnnotated {
        /// Annotator identifier.
        annotator: String,
    },
}

/// A labeled training sample with provenance metadata.
#[derive(Debug, Clone)]
pub struct LabeledSample {
    /// Core training sample (features + label).
    pub sample: TrainingSample,
    /// Where this sample came from.
    pub source: DataSource,
    /// Confidence in the label (0.0 - 1.0).
    pub label_confidence: f64,
    /// Original C code snippet.
    pub c_code: String,
    /// Expected Rust code snippet.
    pub rust_code: String,
    /// Additional metadata.
    pub metadata: HashMap<String, String>,
}

impl LabeledSample {
    /// Create a new labeled sample.
    pub fn new(
        sample: TrainingSample,
        source: DataSource,
        c_code: &str,
        rust_code: &str,
    ) -> Self {
        Self {
            sample,
            source,
            label_confidence: 1.0,
            c_code: c_code.to_string(),
            rust_code: rust_code.to_string(),
            metadata: HashMap::new(),
        }
    }

    /// Set label confidence.
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.label_confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Statistics about a training dataset.
#[derive(Debug, Clone, Default)]
pub struct DatasetStats {
    /// Total sample count.
    pub total_samples: usize,
    /// Samples per label.
    pub label_distribution: HashMap<String, usize>,
    /// Samples per source type.
    pub source_distribution: HashMap<String, usize>,
    /// Average label confidence.
    pub avg_confidence: f64,
    /// Minimum label confidence.
    pub min_confidence: f64,
    /// Maximum label confidence.
    pub max_confidence: f64,
}

impl DatasetStats {
    /// Check if dataset is balanced (no class has >3x samples of another).
    pub fn is_balanced(&self) -> bool {
        if self.label_distribution.is_empty() {
            return true;
        }

        let counts: Vec<usize> = self.label_distribution.values().copied().collect();
        let min_count = counts.iter().copied().min().unwrap_or(0);
        let max_count = counts.iter().copied().max().unwrap_or(0);

        min_count > 0 && max_count <= min_count * 3
    }

    /// Get the dominant label (most samples).
    pub fn dominant_label(&self) -> Option<String> {
        self.label_distribution
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(label, _)| label.clone())
    }
}

/// A collection of training samples with metadata.
#[derive(Debug, Clone, Default)]
pub struct TrainingDataset {
    /// All labeled samples.
    samples: Vec<LabeledSample>,
    /// Dataset name.
    name: String,
    /// Version identifier.
    version: String,
}

impl TrainingDataset {
    /// Create a new empty dataset.
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            samples: Vec::new(),
            name: name.to_string(),
            version: version.to_string(),
        }
    }

    /// Add a sample to the dataset.
    pub fn add(&mut self, sample: LabeledSample) {
        self.samples.push(sample);
    }

    /// Add multiple samples.
    pub fn add_all(&mut self, samples: impl IntoIterator<Item = LabeledSample>) {
        self.samples.extend(samples);
    }

    /// Get sample count.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Get all samples.
    pub fn samples(&self) -> &[LabeledSample] {
        &self.samples
    }

    /// Get samples by source.
    pub fn samples_by_source(&self, source_type: &str) -> Vec<&LabeledSample> {
        self.samples
            .iter()
            .filter(|s| source_type_name(&s.source) == source_type)
            .collect()
    }

    /// Get samples by label.
    pub fn samples_by_label(&self, label: InferredOwnership) -> Vec<&LabeledSample> {
        self.samples
            .iter()
            .filter(|s| s.sample.label == label)
            .collect()
    }

    /// Convert to training samples for the pipeline.
    pub fn to_training_samples(&self) -> Vec<TrainingSample> {
        self.samples.iter().map(|s| s.sample.clone()).collect()
    }

    /// Compute dataset statistics.
    pub fn stats(&self) -> DatasetStats {
        let mut label_dist = HashMap::new();
        let mut source_dist = HashMap::new();
        let mut confidence_sum = 0.0_f64;
        let mut min_conf: f64 = 1.0;
        let mut max_conf: f64 = 0.0;

        for sample in &self.samples {
            // Label distribution
            let label_key = format!("{:?}", sample.sample.label);
            *label_dist.entry(label_key).or_insert(0) += 1;

            // Source distribution
            let source_key = source_type_name(&sample.source);
            *source_dist.entry(source_key).or_insert(0) += 1;

            // Confidence stats
            confidence_sum += sample.label_confidence;
            min_conf = min_conf.min(sample.label_confidence);
            max_conf = max_conf.max(sample.label_confidence);
        }

        let avg_confidence = if self.samples.is_empty() {
            0.0
        } else {
            confidence_sum / self.samples.len() as f64
        };

        DatasetStats {
            total_samples: self.samples.len(),
            label_distribution: label_dist,
            source_distribution: source_dist,
            avg_confidence,
            min_confidence: if self.samples.is_empty() { 0.0 } else { min_conf },
            max_confidence: max_conf,
        }
    }

    /// Filter samples by confidence threshold.
    pub fn filter_by_confidence(&self, min_confidence: f64) -> TrainingDataset {
        let samples = self
            .samples
            .iter()
            .filter(|s| s.label_confidence >= min_confidence)
            .cloned()
            .collect();

        TrainingDataset {
            samples,
            name: self.name.clone(),
            version: self.version.clone(),
        }
    }

    /// Merge with another dataset.
    pub fn merge(&mut self, other: TrainingDataset) {
        self.samples.extend(other.samples);
    }

    /// Get dataset name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get dataset version.
    pub fn version(&self) -> &str {
        &self.version
    }
}

/// Get source type name for statistics.
fn source_type_name(source: &DataSource) -> String {
    match source {
        DataSource::RustPort { .. } => "RustPort".to_string(),
        DataSource::CompilerFeedback { .. } => "CompilerFeedback".to_string(),
        DataSource::Synthetic { .. } => "Synthetic".to_string(),
        DataSource::HumanAnnotated { .. } => "HumanAnnotated".to_string(),
    }
}

/// Configuration for synthetic data generation.
#[derive(Debug, Clone)]
pub struct SyntheticConfig {
    /// Number of samples to generate per pattern.
    pub samples_per_pattern: usize,
    /// Randomization seed for reproducibility.
    pub seed: u64,
    /// Whether to include edge cases.
    pub include_edge_cases: bool,
}

impl Default for SyntheticConfig {
    fn default() -> Self {
        Self {
            samples_per_pattern: 100,
            seed: 42,
            include_edge_cases: true,
        }
    }
}

/// Generator for synthetic training data.
pub struct SyntheticDataGenerator {
    config: SyntheticConfig,
}

impl SyntheticDataGenerator {
    /// Create a new generator with configuration.
    pub fn new(config: SyntheticConfig) -> Self {
        Self { config }
    }

    /// Generate samples for malloc/free → Box pattern.
    pub fn generate_malloc_box_samples(&self) -> Vec<LabeledSample> {
        let mut samples = Vec::new();

        // Basic malloc pattern
        for i in 0..self.config.samples_per_pattern {
            let features = OwnershipFeaturesBuilder::default()
                .pointer_depth(1)
                .allocation_site(AllocationKind::Malloc)
                .deallocation_count(1)
                .build();

            let sample = TrainingSample::new(
                features,
                InferredOwnership::Owned,
                &format!("malloc_box_{}.c", i),
                i as u32,
            );

            let c_code = format!(
                "int* ptr{} = (int*)malloc(sizeof(int));\n*ptr{} = {};\nfree(ptr{});",
                i, i, i, i
            );
            let rust_code = format!("let ptr{}: Box<i32> = Box::new({});", i, i);

            samples.push(LabeledSample::new(
                sample,
                DataSource::Synthetic {
                    template: "malloc_free_box".to_string(),
                },
                &c_code,
                &rust_code,
            ));
        }

        samples
    }

    /// Generate samples for array allocation → Vec pattern.
    pub fn generate_array_vec_samples(&self) -> Vec<LabeledSample> {
        let mut samples = Vec::new();

        for i in 0..self.config.samples_per_pattern {
            let size = (i % 10 + 1) * 10;
            let features = OwnershipFeaturesBuilder::default()
                .pointer_depth(1)
                .allocation_site(AllocationKind::Malloc)
                .has_size_param(true)
                .array_decay(true)
                .deallocation_count(1)
                .build();

            let sample = TrainingSample::new(
                features,
                InferredOwnership::Vec,
                &format!("array_vec_{}.c", i),
                i as u32,
            );
            let _ = size; // Used in c_code formatting

            let c_code = format!(
                "int* arr = (int*)malloc({} * sizeof(int));\nfor(int j = 0; j < {}; j++) arr[j] = j;\nfree(arr);",
                size, size
            );
            let rust_code = format!("let arr: Vec<i32> = (0..{}).collect();", size);

            samples.push(LabeledSample::new(
                sample,
                DataSource::Synthetic {
                    template: "array_vec".to_string(),
                },
                &c_code,
                &rust_code,
            ));
        }

        samples
    }

    /// Generate samples for const pointer → &T pattern.
    pub fn generate_const_ref_samples(&self) -> Vec<LabeledSample> {
        let mut samples = Vec::new();

        for i in 0..self.config.samples_per_pattern {
            let features = OwnershipFeaturesBuilder::default()
                .pointer_depth(1)
                .const_qualified(true)
                .build();

            let sample = TrainingSample::new(
                features,
                InferredOwnership::Borrowed,
                &format!("const_ref_{}.c", i),
                i as u32,
            );

            let c_code = format!("void process{}(const int* ptr) {{ printf(\"%d\", *ptr); }}", i);
            let rust_code = format!("fn process{}(ptr: &i32) {{ println!(\"{{}}\", ptr); }}", i);

            samples.push(LabeledSample::new(
                sample,
                DataSource::Synthetic {
                    template: "const_ref".to_string(),
                },
                &c_code,
                &rust_code,
            ));
        }

        samples
    }

    /// Generate samples for mutable pointer → &mut T pattern.
    pub fn generate_mut_ref_samples(&self) -> Vec<LabeledSample> {
        let mut samples = Vec::new();

        for i in 0..self.config.samples_per_pattern {
            let features = OwnershipFeaturesBuilder::default()
                .pointer_depth(1)
                .const_qualified(false)
                .write_count(1)
                .build();

            let sample = TrainingSample::new(
                features,
                InferredOwnership::BorrowedMut,
                &format!("mut_ref_{}.c", i),
                i as u32,
            );

            let c_code = format!("void increment{}(int* ptr) {{ (*ptr)++; }}", i);
            let rust_code = format!("fn increment{}(ptr: &mut i32) {{ *ptr += 1; }}", i);

            samples.push(LabeledSample::new(
                sample,
                DataSource::Synthetic {
                    template: "mut_ref".to_string(),
                },
                &c_code,
                &rust_code,
            ));
        }

        samples
    }

    /// Generate samples for slice parameter pattern.
    pub fn generate_slice_samples(&self) -> Vec<LabeledSample> {
        let mut samples = Vec::new();

        for i in 0..self.config.samples_per_pattern {
            let features = OwnershipFeaturesBuilder::default()
                .pointer_depth(1)
                .const_qualified(true)
                .array_decay(true)
                .has_size_param(true)
                .build();

            let sample = TrainingSample::new(
                features,
                InferredOwnership::Slice,
                &format!("slice_{}.c", i),
                i as u32,
            );

            let c_code = format!(
                "int sum{}(const int* arr, size_t len) {{ int s = 0; for(size_t j = 0; j < len; j++) s += arr[j]; return s; }}",
                i
            );
            let rust_code = format!(
                "fn sum{}(arr: &[i32]) -> i32 {{ arr.iter().sum() }}",
                i
            );

            samples.push(LabeledSample::new(
                sample,
                DataSource::Synthetic {
                    template: "slice".to_string(),
                },
                &c_code,
                &rust_code,
            ));
        }

        samples
    }

    /// Generate a complete synthetic dataset with all patterns.
    pub fn generate_full_dataset(&self) -> TrainingDataset {
        let mut dataset = TrainingDataset::new("synthetic", "1.0.0");

        dataset.add_all(self.generate_malloc_box_samples());
        dataset.add_all(self.generate_array_vec_samples());
        dataset.add_all(self.generate_const_ref_samples());
        dataset.add_all(self.generate_mut_ref_samples());
        dataset.add_all(self.generate_slice_samples());

        dataset
    }
}

/// Result of data collection.
#[derive(Debug)]
pub struct CollectionResult {
    /// Samples collected.
    pub samples_collected: usize,
    /// Errors encountered.
    pub errors: Vec<String>,
    /// Source path.
    pub source_path: Option<PathBuf>,
}

/// Collector for training data from various sources.
pub struct TrainingDataCollector {
    /// Collected samples.
    samples: Vec<LabeledSample>,
    /// Collection errors.
    errors: Vec<String>,
}

impl TrainingDataCollector {
    /// Create a new collector.
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Add synthetic data from generator.
    pub fn add_synthetic(&mut self, generator: &SyntheticDataGenerator) {
        let dataset = generator.generate_full_dataset();
        self.samples.extend(dataset.samples);
    }

    /// Record a collection error.
    pub fn record_error(&mut self, error: &str) {
        self.errors.push(error.to_string());
    }

    /// Get collection result.
    pub fn result(&self) -> CollectionResult {
        CollectionResult {
            samples_collected: self.samples.len(),
            errors: self.errors.clone(),
            source_path: None,
        }
    }

    /// Build the final dataset.
    pub fn build(self, name: &str, version: &str) -> TrainingDataset {
        let mut dataset = TrainingDataset::new(name, version);
        dataset.add_all(self.samples);
        dataset
    }

    /// Get sample count.
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Get error count.
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
}

impl Default for TrainingDataCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // DataSource tests
    // ========================================================================

    #[test]
    fn data_source_rust_port() {
        let source = DataSource::RustPort {
            project: "rusqlite".to_string(),
        };
        assert_eq!(source_type_name(&source), "RustPort");
    }

    #[test]
    fn data_source_compiler_feedback() {
        let source = DataSource::CompilerFeedback {
            error_code: "E0382".to_string(),
        };
        assert_eq!(source_type_name(&source), "CompilerFeedback");
    }

    #[test]
    fn data_source_synthetic() {
        let source = DataSource::Synthetic {
            template: "malloc_box".to_string(),
        };
        assert_eq!(source_type_name(&source), "Synthetic");
    }

    #[test]
    fn data_source_human_annotated() {
        let source = DataSource::HumanAnnotated {
            annotator: "expert1".to_string(),
        };
        assert_eq!(source_type_name(&source), "HumanAnnotated");
    }

    // ========================================================================
    // LabeledSample tests
    // ========================================================================

    #[test]
    fn labeled_sample_new() {
        let features = OwnershipFeaturesBuilder::default().build();
        let sample = TrainingSample::new(features, InferredOwnership::Owned, "test.c", 42);

        let labeled = LabeledSample::new(
            sample,
            DataSource::Synthetic {
                template: "test".to_string(),
            },
            "int* p = malloc(4);",
            "let p: Box<i32> = Box::new(0);",
        );

        assert!((labeled.label_confidence - 1.0).abs() < 0.001);
        assert!(labeled.c_code.contains("malloc"));
        assert!(labeled.rust_code.contains("Box"));
    }

    #[test]
    fn labeled_sample_with_confidence() {
        let features = OwnershipFeaturesBuilder::default().build();
        let sample = TrainingSample::new(features, InferredOwnership::Owned, "test.c", 1);

        let labeled = LabeledSample::new(
            sample,
            DataSource::Synthetic {
                template: "test".to_string(),
            },
            "",
            "",
        )
        .with_confidence(0.8);

        assert!((labeled.label_confidence - 0.8).abs() < 0.001);
    }

    #[test]
    fn labeled_sample_confidence_clamped() {
        let features = OwnershipFeaturesBuilder::default().build();
        let sample = TrainingSample::new(features, InferredOwnership::Owned, "test.c", 1);

        let labeled = LabeledSample::new(
            sample,
            DataSource::Synthetic {
                template: "test".to_string(),
            },
            "",
            "",
        )
        .with_confidence(1.5);

        assert!((labeled.label_confidence - 1.0).abs() < 0.001);
    }

    #[test]
    fn labeled_sample_with_metadata() {
        let features = OwnershipFeaturesBuilder::default().build();
        let sample = TrainingSample::new(features, InferredOwnership::Owned, "test.c", 1);

        let labeled = LabeledSample::new(
            sample,
            DataSource::Synthetic {
                template: "test".to_string(),
            },
            "",
            "",
        )
        .with_metadata("commit", "abc123");

        assert_eq!(labeled.metadata.get("commit"), Some(&"abc123".to_string()));
    }

    // ========================================================================
    // DatasetStats tests
    // ========================================================================

    #[test]
    fn dataset_stats_is_balanced_empty() {
        let stats = DatasetStats::default();
        assert!(stats.is_balanced());
    }

    #[test]
    fn dataset_stats_is_balanced_even() {
        let mut stats = DatasetStats::default();
        stats.label_distribution.insert("Owned".to_string(), 100);
        stats.label_distribution.insert("Borrowed".to_string(), 100);
        assert!(stats.is_balanced());
    }

    #[test]
    fn dataset_stats_is_balanced_imbalanced() {
        let mut stats = DatasetStats::default();
        stats.label_distribution.insert("Owned".to_string(), 100);
        stats.label_distribution.insert("Borrowed".to_string(), 10);
        assert!(!stats.is_balanced()); // 100 > 10 * 3
    }

    #[test]
    fn dataset_stats_dominant_label() {
        let mut stats = DatasetStats::default();
        stats.label_distribution.insert("Owned".to_string(), 100);
        stats.label_distribution.insert("Borrowed".to_string(), 50);
        assert_eq!(stats.dominant_label(), Some("Owned".to_string()));
    }

    // ========================================================================
    // TrainingDataset tests
    // ========================================================================

    #[test]
    fn training_dataset_new() {
        let dataset = TrainingDataset::new("test", "1.0.0");
        assert_eq!(dataset.name(), "test");
        assert_eq!(dataset.version(), "1.0.0");
        assert!(dataset.is_empty());
    }

    #[test]
    fn training_dataset_add() {
        let mut dataset = TrainingDataset::new("test", "1.0.0");

        let features = OwnershipFeaturesBuilder::default().build();
        let sample = TrainingSample::new(features, InferredOwnership::Owned, "test.c", 1);
        let labeled = LabeledSample::new(
            sample,
            DataSource::Synthetic {
                template: "test".to_string(),
            },
            "",
            "",
        );

        dataset.add(labeled);
        assert_eq!(dataset.len(), 1);
    }

    #[test]
    fn training_dataset_stats() {
        let mut dataset = TrainingDataset::new("test", "1.0.0");

        // Add 2 Owned samples
        for _ in 0..2 {
            let features = OwnershipFeaturesBuilder::default().build();
            let sample = TrainingSample::new(features, InferredOwnership::Owned, "test.c", 1);
            dataset.add(LabeledSample::new(
                sample,
                DataSource::Synthetic {
                    template: "test".to_string(),
                },
                "",
                "",
            ));
        }

        // Add 1 Borrowed sample
        let features = OwnershipFeaturesBuilder::default().build();
        let sample = TrainingSample::new(features, InferredOwnership::Borrowed, "test.c", 1);
        dataset.add(LabeledSample::new(
            sample,
            DataSource::Synthetic {
                template: "test".to_string(),
            },
            "",
            "",
        ));

        let stats = dataset.stats();
        assert_eq!(stats.total_samples, 3);
        assert_eq!(stats.label_distribution.get("Owned"), Some(&2));
        assert_eq!(stats.label_distribution.get("Borrowed"), Some(&1));
    }

    #[test]
    fn training_dataset_filter_by_confidence() {
        let mut dataset = TrainingDataset::new("test", "1.0.0");

        let features = OwnershipFeaturesBuilder::default().build();
        let sample = TrainingSample::new(features.clone(), InferredOwnership::Owned, "test.c", 1);
        dataset.add(
            LabeledSample::new(
                sample,
                DataSource::Synthetic {
                    template: "test".to_string(),
                },
                "",
                "",
            )
            .with_confidence(0.9),
        );

        let sample2 = TrainingSample::new(features, InferredOwnership::Owned, "test.c", 2);
        dataset.add(
            LabeledSample::new(
                sample2,
                DataSource::Synthetic {
                    template: "test".to_string(),
                },
                "",
                "",
            )
            .with_confidence(0.5),
        );

        let filtered = dataset.filter_by_confidence(0.8);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn training_dataset_to_training_samples() {
        let mut dataset = TrainingDataset::new("test", "1.0.0");

        let features = OwnershipFeaturesBuilder::default().build();
        let sample = TrainingSample::new(features, InferredOwnership::Owned, "test.c", 1);
        dataset.add(LabeledSample::new(
            sample,
            DataSource::Synthetic {
                template: "test".to_string(),
            },
            "",
            "",
        ));

        let samples = dataset.to_training_samples();
        assert_eq!(samples.len(), 1);
    }

    // ========================================================================
    // SyntheticDataGenerator tests
    // ========================================================================

    #[test]
    fn synthetic_generator_config_default() {
        let config = SyntheticConfig::default();
        assert_eq!(config.samples_per_pattern, 100);
        assert_eq!(config.seed, 42);
        assert!(config.include_edge_cases);
    }

    #[test]
    fn synthetic_generator_malloc_box() {
        let config = SyntheticConfig {
            samples_per_pattern: 10,
            ..Default::default()
        };
        let generator = SyntheticDataGenerator::new(config);
        let samples = generator.generate_malloc_box_samples();

        assert_eq!(samples.len(), 10);
        for sample in &samples {
            assert!(matches!(sample.sample.label, InferredOwnership::Owned));
            assert!(sample.c_code.contains("malloc"));
            assert!(sample.rust_code.contains("Box"));
        }
    }

    #[test]
    fn synthetic_generator_array_vec() {
        let config = SyntheticConfig {
            samples_per_pattern: 10,
            ..Default::default()
        };
        let generator = SyntheticDataGenerator::new(config);
        let samples = generator.generate_array_vec_samples();

        assert_eq!(samples.len(), 10);
        for sample in &samples {
            assert!(matches!(sample.sample.label, InferredOwnership::Vec));
            assert!(sample.rust_code.contains("Vec"));
        }
    }

    #[test]
    fn synthetic_generator_const_ref() {
        let config = SyntheticConfig {
            samples_per_pattern: 10,
            ..Default::default()
        };
        let generator = SyntheticDataGenerator::new(config);
        let samples = generator.generate_const_ref_samples();

        assert_eq!(samples.len(), 10);
        for sample in &samples {
            assert!(matches!(sample.sample.label, InferredOwnership::Borrowed));
            assert!(sample.c_code.contains("const"));
        }
    }

    #[test]
    fn synthetic_generator_mut_ref() {
        let config = SyntheticConfig {
            samples_per_pattern: 10,
            ..Default::default()
        };
        let generator = SyntheticDataGenerator::new(config);
        let samples = generator.generate_mut_ref_samples();

        assert_eq!(samples.len(), 10);
        for sample in &samples {
            assert!(matches!(sample.sample.label, InferredOwnership::BorrowedMut));
            assert!(sample.rust_code.contains("&mut"));
        }
    }

    #[test]
    fn synthetic_generator_slice() {
        let config = SyntheticConfig {
            samples_per_pattern: 10,
            ..Default::default()
        };
        let generator = SyntheticDataGenerator::new(config);
        let samples = generator.generate_slice_samples();

        assert_eq!(samples.len(), 10);
        for sample in &samples {
            assert!(matches!(sample.sample.label, InferredOwnership::Slice));
            assert!(sample.rust_code.contains("&[i32]"));
        }
    }

    #[test]
    fn synthetic_generator_full_dataset() {
        let config = SyntheticConfig {
            samples_per_pattern: 10,
            ..Default::default()
        };
        let generator = SyntheticDataGenerator::new(config);
        let dataset = generator.generate_full_dataset();

        // 5 patterns * 10 samples each = 50
        assert_eq!(dataset.len(), 50);

        let stats = dataset.stats();
        assert_eq!(stats.source_distribution.get("Synthetic"), Some(&50));
    }

    // ========================================================================
    // TrainingDataCollector tests
    // ========================================================================

    #[test]
    fn collector_new() {
        let collector = TrainingDataCollector::new();
        assert_eq!(collector.sample_count(), 0);
        assert_eq!(collector.error_count(), 0);
    }

    #[test]
    fn collector_add_synthetic() {
        let mut collector = TrainingDataCollector::new();
        let config = SyntheticConfig {
            samples_per_pattern: 10,
            ..Default::default()
        };
        let generator = SyntheticDataGenerator::new(config);

        collector.add_synthetic(&generator);

        assert_eq!(collector.sample_count(), 50);
    }

    #[test]
    fn collector_record_error() {
        let mut collector = TrainingDataCollector::new();
        collector.record_error("Failed to parse file");

        assert_eq!(collector.error_count(), 1);
        assert!(collector.result().errors[0].contains("parse"));
    }

    #[test]
    fn collector_build() {
        let mut collector = TrainingDataCollector::new();
        let config = SyntheticConfig {
            samples_per_pattern: 10,
            ..Default::default()
        };
        let generator = SyntheticDataGenerator::new(config);

        collector.add_synthetic(&generator);
        let dataset = collector.build("test", "1.0.0");

        assert_eq!(dataset.len(), 50);
        assert_eq!(dataset.name(), "test");
        assert_eq!(dataset.version(), "1.0.0");
    }

    #[test]
    fn collector_result() {
        let mut collector = TrainingDataCollector::new();
        let config = SyntheticConfig {
            samples_per_pattern: 10,
            ..Default::default()
        };
        let generator = SyntheticDataGenerator::new(config);

        collector.add_synthetic(&generator);
        let result = collector.result();

        assert_eq!(result.samples_collected, 50);
        assert!(result.errors.is_empty());
    }

    // ========================================================================
    // Integration tests
    // ========================================================================

    #[test]
    fn generate_1000_samples() {
        // This test verifies we can generate 1000+ samples for training
        let config = SyntheticConfig {
            samples_per_pattern: 200,
            ..Default::default()
        };
        let generator = SyntheticDataGenerator::new(config);
        let dataset = generator.generate_full_dataset();

        // 5 patterns * 200 samples = 1000
        assert!(dataset.len() >= 1000);

        let stats = dataset.stats();
        // Should be reasonably balanced (each label type has samples)
        assert!(stats.label_distribution.len() >= 4);
    }
}
