//! Additional coverage tests for training_data.rs.
//!
//! Targets uncovered branches including:
//! - TrainingDataset::samples_by_source with various source types
//! - TrainingDataset::samples_by_label with all label variants
//! - TrainingDataset::merge
//! - TrainingDataset::add_all
//! - DatasetStats::is_balanced with edge cases (min_count == 0)
//! - DatasetStats::dominant_label with empty distribution
//! - TrainingDataset::stats with varying confidences
//! - TrainingDataCollector::default
//! - SyntheticConfig custom values
//! - LabeledSample::with_confidence negative clamping

use crate::ml_features::{InferredOwnership, OwnershipFeaturesBuilder};
use crate::retraining_pipeline::TrainingSample;
use crate::training_data::*;

fn make_labeled_sample(label: InferredOwnership, confidence: f64) -> LabeledSample {
    let features = OwnershipFeaturesBuilder::default().build();
    let sample = TrainingSample::new(features, label, "test.c", 0);
    LabeledSample::new(
        sample,
        DataSource::Synthetic {
            template: "test".to_string(),
        },
        "int* p;",
        "let p: i32;",
    )
    .with_confidence(confidence)
}

// ============================================================================
// samples_by_source -- all source type variants
// ============================================================================

#[test]
fn test_samples_by_source_rust_port() {
    let mut dataset = TrainingDataset::new("test", "1.0");
    let features = OwnershipFeaturesBuilder::default().build();
    let sample = TrainingSample::new(features, InferredOwnership::Owned, "test.c", 0);
    dataset.add(LabeledSample::new(
        sample,
        DataSource::RustPort {
            project: "linux".to_string(),
        },
        "",
        "",
    ));
    let filtered = dataset.samples_by_source("RustPort");
    assert_eq!(filtered.len(), 1);
}

#[test]
fn test_samples_by_source_compiler_feedback() {
    let mut dataset = TrainingDataset::new("test", "1.0");
    let features = OwnershipFeaturesBuilder::default().build();
    let sample = TrainingSample::new(features, InferredOwnership::Borrowed, "test.c", 0);
    dataset.add(LabeledSample::new(
        sample,
        DataSource::CompilerFeedback {
            error_code: "E0382".to_string(),
        },
        "",
        "",
    ));
    let filtered = dataset.samples_by_source("CompilerFeedback");
    assert_eq!(filtered.len(), 1);
}

#[test]
fn test_samples_by_source_human_annotated() {
    let mut dataset = TrainingDataset::new("test", "1.0");
    let features = OwnershipFeaturesBuilder::default().build();
    let sample = TrainingSample::new(features, InferredOwnership::BorrowedMut, "test.c", 0);
    dataset.add(LabeledSample::new(
        sample,
        DataSource::HumanAnnotated {
            annotator: "expert1".to_string(),
        },
        "",
        "",
    ));
    let filtered = dataset.samples_by_source("HumanAnnotated");
    assert_eq!(filtered.len(), 1);
}

#[test]
fn test_samples_by_source_synthetic() {
    let mut dataset = TrainingDataset::new("test", "1.0");
    let features = OwnershipFeaturesBuilder::default().build();
    let sample = TrainingSample::new(features, InferredOwnership::Owned, "test.c", 0);
    dataset.add(LabeledSample::new(
        sample,
        DataSource::Synthetic {
            template: "test".to_string(),
        },
        "",
        "",
    ));
    let filtered = dataset.samples_by_source("Synthetic");
    assert_eq!(filtered.len(), 1);
}

#[test]
fn test_samples_by_source_no_match() {
    let mut dataset = TrainingDataset::new("test", "1.0");
    dataset.add(make_labeled_sample(InferredOwnership::Owned, 1.0));
    let filtered = dataset.samples_by_source("RustPort");
    assert!(filtered.is_empty());
}

// ============================================================================
// samples_by_label -- all label variants
// ============================================================================

#[test]
fn test_samples_by_label_owned() {
    let mut dataset = TrainingDataset::new("test", "1.0");
    dataset.add(make_labeled_sample(InferredOwnership::Owned, 1.0));
    dataset.add(make_labeled_sample(InferredOwnership::Borrowed, 1.0));
    let filtered = dataset.samples_by_label(InferredOwnership::Owned);
    assert_eq!(filtered.len(), 1);
}

#[test]
fn test_samples_by_label_borrowed() {
    let mut dataset = TrainingDataset::new("test", "1.0");
    dataset.add(make_labeled_sample(InferredOwnership::Borrowed, 1.0));
    let filtered = dataset.samples_by_label(InferredOwnership::Borrowed);
    assert_eq!(filtered.len(), 1);
}

#[test]
fn test_samples_by_label_borrowed_mut() {
    let mut dataset = TrainingDataset::new("test", "1.0");
    dataset.add(make_labeled_sample(InferredOwnership::BorrowedMut, 1.0));
    let filtered = dataset.samples_by_label(InferredOwnership::BorrowedMut);
    assert_eq!(filtered.len(), 1);
}

#[test]
fn test_samples_by_label_vec() {
    let mut dataset = TrainingDataset::new("test", "1.0");
    dataset.add(make_labeled_sample(InferredOwnership::Vec, 1.0));
    let filtered = dataset.samples_by_label(InferredOwnership::Vec);
    assert_eq!(filtered.len(), 1);
}

#[test]
fn test_samples_by_label_slice() {
    let mut dataset = TrainingDataset::new("test", "1.0");
    dataset.add(make_labeled_sample(InferredOwnership::Slice, 1.0));
    let filtered = dataset.samples_by_label(InferredOwnership::Slice);
    assert_eq!(filtered.len(), 1);
}

#[test]
fn test_samples_by_label_no_match() {
    let mut dataset = TrainingDataset::new("test", "1.0");
    dataset.add(make_labeled_sample(InferredOwnership::Owned, 1.0));
    let filtered = dataset.samples_by_label(InferredOwnership::Slice);
    assert!(filtered.is_empty());
}

// ============================================================================
// merge datasets
// ============================================================================

#[test]
fn test_merge_two_datasets() {
    let mut d1 = TrainingDataset::new("d1", "1.0");
    d1.add(make_labeled_sample(InferredOwnership::Owned, 1.0));
    d1.add(make_labeled_sample(InferredOwnership::Owned, 1.0));

    let mut d2 = TrainingDataset::new("d2", "1.0");
    d2.add(make_labeled_sample(InferredOwnership::Borrowed, 0.9));

    d1.merge(d2);
    assert_eq!(d1.len(), 3);
}

#[test]
fn test_merge_empty_dataset() {
    let mut d1 = TrainingDataset::new("d1", "1.0");
    d1.add(make_labeled_sample(InferredOwnership::Owned, 1.0));

    let d2 = TrainingDataset::new("d2", "1.0");
    d1.merge(d2);
    assert_eq!(d1.len(), 1);
}

// ============================================================================
// add_all
// ============================================================================

#[test]
fn test_add_all_from_vec() {
    let mut dataset = TrainingDataset::new("test", "1.0");
    let samples = vec![
        make_labeled_sample(InferredOwnership::Owned, 1.0),
        make_labeled_sample(InferredOwnership::Borrowed, 0.8),
        make_labeled_sample(InferredOwnership::Vec, 0.9),
    ];
    dataset.add_all(samples);
    assert_eq!(dataset.len(), 3);
}

// ============================================================================
// DatasetStats edge cases
// ============================================================================

#[test]
fn test_is_balanced_with_zero_count() {
    let mut stats = DatasetStats::default();
    stats.label_distribution.insert("Owned".to_string(), 0);
    stats.label_distribution.insert("Borrowed".to_string(), 10);
    // min_count is 0, so min_count > 0 is false, so not balanced
    assert!(!stats.is_balanced());
}

#[test]
fn test_is_balanced_with_single_class() {
    let mut stats = DatasetStats::default();
    stats.label_distribution.insert("Owned".to_string(), 100);
    // single class: min=100, max=100, 100 <= 100*3 = true, min > 0 = true
    assert!(stats.is_balanced());
}

#[test]
fn test_is_balanced_at_3x_boundary() {
    let mut stats = DatasetStats::default();
    stats.label_distribution.insert("Owned".to_string(), 30);
    stats.label_distribution.insert("Borrowed".to_string(), 10);
    // max(30) <= min(10) * 3 = 30, true -- exactly at boundary
    assert!(stats.is_balanced());
}

#[test]
fn test_is_balanced_just_over_3x() {
    let mut stats = DatasetStats::default();
    stats.label_distribution.insert("Owned".to_string(), 31);
    stats.label_distribution.insert("Borrowed".to_string(), 10);
    // max(31) <= min(10) * 3 = 30, false
    assert!(!stats.is_balanced());
}

#[test]
fn test_dominant_label_empty() {
    let stats = DatasetStats::default();
    assert!(stats.dominant_label().is_none());
}

#[test]
fn test_dominant_label_tie_returns_some() {
    let mut stats = DatasetStats::default();
    stats.label_distribution.insert("Owned".to_string(), 50);
    stats.label_distribution.insert("Borrowed".to_string(), 50);
    // Should return one of them (non-deterministic order in HashMap, but Some)
    assert!(stats.dominant_label().is_some());
}

// ============================================================================
// stats with varying confidences
// ============================================================================

#[test]
fn test_stats_confidence_range() {
    let mut dataset = TrainingDataset::new("test", "1.0");
    dataset.add(make_labeled_sample(InferredOwnership::Owned, 0.3));
    dataset.add(make_labeled_sample(InferredOwnership::Owned, 0.7));
    dataset.add(make_labeled_sample(InferredOwnership::Owned, 1.0));

    let stats = dataset.stats();
    assert_eq!(stats.total_samples, 3);
    assert!((stats.min_confidence - 0.3).abs() < 0.001);
    assert!((stats.max_confidence - 1.0).abs() < 0.001);
    // avg = (0.3 + 0.7 + 1.0) / 3 = 0.6667
    assert!((stats.avg_confidence - 0.6667).abs() < 0.01);
}

#[test]
fn test_stats_empty_dataset() {
    let dataset = TrainingDataset::new("empty", "1.0");
    let stats = dataset.stats();
    assert_eq!(stats.total_samples, 0);
    assert!((stats.avg_confidence - 0.0).abs() < 0.001);
    assert!((stats.min_confidence - 0.0).abs() < 0.001);
    assert!((stats.max_confidence - 0.0).abs() < 0.001);
}

#[test]
fn test_stats_source_distribution() {
    let mut dataset = TrainingDataset::new("test", "1.0");
    // Add samples from different sources
    let features = OwnershipFeaturesBuilder::default().build();
    let s1 = TrainingSample::new(features.clone(), InferredOwnership::Owned, "a.c", 0);
    dataset.add(LabeledSample::new(
        s1,
        DataSource::RustPort {
            project: "linux".to_string(),
        },
        "",
        "",
    ));
    let s2 = TrainingSample::new(features, InferredOwnership::Borrowed, "b.c", 1);
    dataset.add(LabeledSample::new(
        s2,
        DataSource::CompilerFeedback {
            error_code: "E0505".to_string(),
        },
        "",
        "",
    ));

    let stats = dataset.stats();
    assert_eq!(stats.source_distribution.get("RustPort"), Some(&1));
    assert_eq!(stats.source_distribution.get("CompilerFeedback"), Some(&1));
}

// ============================================================================
// LabeledSample confidence clamping
// ============================================================================

#[test]
fn test_confidence_clamped_below_zero() {
    let sample = make_labeled_sample(InferredOwnership::Owned, -0.5);
    assert!((sample.label_confidence - 0.0).abs() < 0.001);
}

#[test]
fn test_confidence_clamped_above_one() {
    let sample = make_labeled_sample(InferredOwnership::Owned, 2.5);
    assert!((sample.label_confidence - 1.0).abs() < 0.001);
}

#[test]
fn test_confidence_exact_boundaries() {
    let s0 = make_labeled_sample(InferredOwnership::Owned, 0.0);
    assert!((s0.label_confidence - 0.0).abs() < 0.001);
    let s1 = make_labeled_sample(InferredOwnership::Owned, 1.0);
    assert!((s1.label_confidence - 1.0).abs() < 0.001);
}

// ============================================================================
// LabeledSample with_metadata chaining
// ============================================================================

#[test]
fn test_with_metadata_multiple_keys() {
    let sample = make_labeled_sample(InferredOwnership::Owned, 1.0)
        .with_metadata("key1", "val1")
        .with_metadata("key2", "val2");
    assert_eq!(sample.metadata.get("key1"), Some(&"val1".to_string()));
    assert_eq!(sample.metadata.get("key2"), Some(&"val2".to_string()));
}

// ============================================================================
// TrainingDataCollector::default
// ============================================================================

#[test]
fn test_collector_default_trait() {
    let collector = TrainingDataCollector::default();
    assert_eq!(collector.sample_count(), 0);
    assert_eq!(collector.error_count(), 0);
}

// ============================================================================
// SyntheticConfig custom values
// ============================================================================

#[test]
fn test_synthetic_config_custom_values() {
    let config = SyntheticConfig {
        samples_per_pattern: 5,
        seed: 123,
        include_edge_cases: false,
    };
    assert_eq!(config.samples_per_pattern, 5);
    assert_eq!(config.seed, 123);
    assert!(!config.include_edge_cases);
}

// ============================================================================
// filter_by_confidence preserves name/version
// ============================================================================

#[test]
fn test_filter_by_confidence_preserves_metadata() {
    let mut dataset = TrainingDataset::new("my_dataset", "2.0");
    dataset.add(make_labeled_sample(InferredOwnership::Owned, 0.9));
    dataset.add(make_labeled_sample(InferredOwnership::Owned, 0.4));

    let filtered = dataset.filter_by_confidence(0.5);
    assert_eq!(filtered.name(), "my_dataset");
    assert_eq!(filtered.version(), "2.0");
    assert_eq!(filtered.len(), 1);
}

// ============================================================================
// to_training_samples preserves labels
// ============================================================================

#[test]
fn test_to_training_samples_preserves_labels() {
    let mut dataset = TrainingDataset::new("test", "1.0");
    dataset.add(make_labeled_sample(InferredOwnership::Owned, 1.0));
    dataset.add(make_labeled_sample(InferredOwnership::Borrowed, 1.0));
    dataset.add(make_labeled_sample(InferredOwnership::Vec, 1.0));

    let samples = dataset.to_training_samples();
    assert_eq!(samples.len(), 3);
    assert_eq!(samples[0].label, InferredOwnership::Owned);
    assert_eq!(samples[1].label, InferredOwnership::Borrowed);
    assert_eq!(samples[2].label, InferredOwnership::Vec);
}

// ============================================================================
// TrainingDataCollector::record_error and result
// ============================================================================

#[test]
fn test_collector_multiple_errors() {
    let mut collector = TrainingDataCollector::new();
    collector.record_error("error 1");
    collector.record_error("error 2");
    collector.record_error("error 3");
    assert_eq!(collector.error_count(), 3);
    let result = collector.result();
    assert_eq!(result.errors.len(), 3);
    assert!(result.source_path.is_none());
}

#[test]
fn test_collector_build_after_errors() {
    let mut collector = TrainingDataCollector::new();
    collector.record_error("some error");
    let config = SyntheticConfig {
        samples_per_pattern: 2,
        ..Default::default()
    };
    let gen = SyntheticDataGenerator::new(config);
    collector.add_synthetic(&gen);
    // 5 patterns * 2 samples = 10
    assert_eq!(collector.sample_count(), 10);
    let dataset = collector.build("errored", "1.0");
    assert_eq!(dataset.len(), 10);
}
