//! CITL-based error tracking for ownership inference (DECY-ML-015).
//!
//! Integrates with entrenar's CITL module to track and analyze
//! ownership inference errors using Tarantula fault localization.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                   ERROR TRACKING PIPELINE                       │
//! │                                                                 │
//! │  Ownership        ┌──────────────┐     ┌────────────────┐      │
//! │  Inference  ─────►│ Error        │────►│ CITL           │      │
//! │  Result           │ Collector    │     │ Analysis       │      │
//! │                   └──────────────┘     └────────────────┘      │
//! │                          │                    │                 │
//! │                          ▼                    ▼                 │
//! │                   ┌──────────────┐     ┌────────────────┐      │
//! │                   │ Error        │     │ Suspiciousness │      │
//! │                   │ Database     │     │ Scores         │      │
//! │                   └──────────────┘     └────────────────┘      │
//! │                                               │                 │
//! │                                               ▼                 │
//! │                                        ┌────────────────┐      │
//! │                                        │ Improvement    │      │
//! │                                        │ Suggestions    │      │
//! │                                        └────────────────┘      │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Toyota Way: Hansei (Reflection)
//!
//! Error tracking embodies Hansei by:
//! - Reflecting on every inference failure
//! - Identifying root causes through CITL analysis
//! - Using insights to improve the model

use crate::ml_features::{InferredOwnership, OwnershipDefect};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An ownership inference error record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceError {
    /// Unique error ID
    pub id: u64,
    /// Variable name
    pub variable: String,
    /// Source file
    pub source_file: String,
    /// Source line
    pub source_line: u32,
    /// Predicted ownership
    pub predicted: InferredOwnership,
    /// Expected ownership (ground truth)
    pub expected: InferredOwnership,
    /// Confidence of prediction
    pub confidence: f64,
    /// C features present in context
    pub c_features: Vec<String>,
    /// Rust error code (if compilation failed)
    pub rust_error: Option<String>,
    /// Defect category
    pub defect: OwnershipDefect,
    /// Timestamp
    pub timestamp: u64,
}

impl InferenceError {
    /// Create a new inference error.
    pub fn new(
        variable: impl Into<String>,
        source_file: impl Into<String>,
        source_line: u32,
        predicted: InferredOwnership,
        expected: InferredOwnership,
        confidence: f64,
        defect: OwnershipDefect,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id: 0,
            variable: variable.into(),
            source_file: source_file.into(),
            source_line,
            predicted,
            expected,
            confidence: confidence.clamp(0.0, 1.0),
            c_features: Vec::new(),
            rust_error: None,
            defect,
            timestamp: now,
        }
    }

    /// Add C features to the error.
    pub fn with_features(mut self, features: Vec<String>) -> Self {
        self.c_features = features;
        self
    }

    /// Add Rust error code.
    pub fn with_rust_error(mut self, error: impl Into<String>) -> Self {
        self.rust_error = Some(error.into());
        self
    }
}

/// Statistics for a specific error pattern.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternStats {
    /// Number of occurrences
    pub count: u64,
    /// Number of times pattern led to failure
    pub failure_count: u64,
    /// Number of times pattern led to success
    pub success_count: u64,
    /// Tarantula suspiciousness score
    pub suspiciousness: f64,
}

impl PatternStats {
    /// Calculate failure rate.
    pub fn failure_rate(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.failure_count as f64 / self.count as f64
        }
    }

    /// Update with new outcome.
    pub fn record(&mut self, is_failure: bool) {
        self.count += 1;
        if is_failure {
            self.failure_count += 1;
        } else {
            self.success_count += 1;
        }
    }
}

/// Suspiciousness score for a C feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSuspiciousness {
    /// Feature name
    pub feature: String,
    /// Tarantula suspiciousness score (0.0 - 1.0)
    pub score: f64,
    /// Total occurrences
    pub total_count: u64,
    /// Failure count
    pub failure_count: u64,
    /// Success count
    pub success_count: u64,
}

impl FeatureSuspiciousness {
    /// Check if feature is suspicious (score > 0.5).
    pub fn is_suspicious(&self) -> bool {
        self.score > 0.5
    }

    /// Check if feature is highly suspicious (score > 0.7).
    pub fn is_highly_suspicious(&self) -> bool {
        self.score > 0.7
    }
}

/// Error tracker using CITL-style analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTracker {
    /// All recorded errors
    errors: Vec<InferenceError>,
    /// Pattern statistics by C feature
    feature_stats: HashMap<String, PatternStats>,
    /// Pattern statistics by defect category
    defect_stats: HashMap<String, PatternStats>,
    /// Pattern statistics by (feature, defect) combination
    feature_defect_stats: HashMap<(String, String), PatternStats>,
    /// Total successes (for Tarantula calculation)
    total_successes: u64,
    /// Total failures (for Tarantula calculation)
    total_failures: u64,
    /// Next error ID
    next_id: u64,
}

impl Default for ErrorTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorTracker {
    /// Create a new error tracker.
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            feature_stats: HashMap::new(),
            defect_stats: HashMap::new(),
            feature_defect_stats: HashMap::new(),
            total_successes: 0,
            total_failures: 0,
            next_id: 1,
        }
    }

    /// Record an inference error.
    pub fn record_error(&mut self, mut error: InferenceError) {
        error.id = self.next_id;
        self.next_id += 1;
        self.total_failures += 1;

        // Update feature stats
        for feature in &error.c_features {
            self.feature_stats
                .entry(feature.clone())
                .or_default()
                .record(true);
        }

        // Update defect stats
        let defect_key = format!("{:?}", error.defect);
        self.defect_stats
            .entry(defect_key.clone())
            .or_default()
            .record(true);

        // Update feature-defect combination stats
        for feature in &error.c_features {
            self.feature_defect_stats
                .entry((feature.clone(), defect_key.clone()))
                .or_default()
                .record(true);
        }

        self.errors.push(error);
    }

    /// Record a successful inference (no error).
    pub fn record_success(&mut self, c_features: &[String]) {
        self.total_successes += 1;

        for feature in c_features {
            self.feature_stats
                .entry(feature.clone())
                .or_default()
                .record(false);
        }
    }

    /// Get all errors.
    pub fn errors(&self) -> &[InferenceError] {
        &self.errors
    }

    /// Get error count.
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get total successes.
    pub fn success_count(&self) -> u64 {
        self.total_successes
    }

    /// Calculate Tarantula suspiciousness for all features.
    ///
    /// Tarantula formula:
    /// suspiciousness = (failed(e) / total_failed) / ((failed(e) / total_failed) + (passed(e) / total_passed))
    pub fn calculate_suspiciousness(&mut self) -> Vec<FeatureSuspiciousness> {
        let total_failed = self.total_failures.max(1) as f64;
        let total_passed = self.total_successes.max(1) as f64;

        let mut results = Vec::new();

        for (feature, stats) in &mut self.feature_stats {
            let failed_ratio = stats.failure_count as f64 / total_failed;
            let passed_ratio = stats.success_count as f64 / total_passed;

            let suspiciousness = if failed_ratio + passed_ratio > 0.0 {
                failed_ratio / (failed_ratio + passed_ratio)
            } else {
                0.0
            };

            stats.suspiciousness = suspiciousness;

            results.push(FeatureSuspiciousness {
                feature: feature.clone(),
                score: suspiciousness,
                total_count: stats.count,
                failure_count: stats.failure_count,
                success_count: stats.success_count,
            });
        }

        // Sort by suspiciousness (highest first)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        results
    }

    /// Get top N most suspicious features.
    pub fn top_suspicious(&mut self, n: usize) -> Vec<FeatureSuspiciousness> {
        let all = self.calculate_suspiciousness();
        all.into_iter().take(n).collect()
    }

    /// Get errors by defect category.
    pub fn errors_by_defect(&self, defect: &OwnershipDefect) -> Vec<&InferenceError> {
        let defect_key = format!("{:?}", defect);
        self.errors
            .iter()
            .filter(|e| format!("{:?}", e.defect) == defect_key)
            .collect()
    }

    /// Get errors by C feature.
    pub fn errors_by_feature(&self, feature: &str) -> Vec<&InferenceError> {
        self.errors
            .iter()
            .filter(|e| e.c_features.contains(&feature.to_string()))
            .collect()
    }

    /// Get defect distribution.
    pub fn defect_distribution(&self) -> HashMap<String, u64> {
        let mut dist = HashMap::new();
        for error in &self.errors {
            *dist.entry(format!("{:?}", error.defect)).or_insert(0) += 1;
        }
        dist
    }

    /// Get feature distribution among errors.
    pub fn feature_distribution(&self) -> HashMap<String, u64> {
        let mut dist = HashMap::new();
        for error in &self.errors {
            for feature in &error.c_features {
                *dist.entry(feature.clone()).or_insert(0) += 1;
            }
        }
        dist
    }

    /// Get correlation between features and defects.
    pub fn feature_defect_correlation(&self) -> Vec<(String, String, u64)> {
        self.feature_defect_stats
            .iter()
            .map(|((f, d), stats)| (f.clone(), d.clone(), stats.failure_count))
            .collect()
    }

    /// Generate improvement suggestions based on analysis.
    pub fn generate_suggestions(&mut self) -> Vec<ImprovementSuggestion> {
        let suspicious = self.top_suspicious(5);
        let defect_dist = self.defect_distribution();

        let mut suggestions = Vec::new();

        // Suggest improvements for highly suspicious features
        for fs in suspicious {
            if fs.is_highly_suspicious() {
                suggestions.push(ImprovementSuggestion {
                    priority: SuggestionPriority::High,
                    category: SuggestionCategory::FeatureHandling,
                    description: format!(
                        "Improve handling of '{}' (suspiciousness: {:.2}, {} failures)",
                        fs.feature, fs.score, fs.failure_count
                    ),
                    affected_feature: Some(fs.feature),
                    affected_defect: None,
                });
            }
        }

        // Suggest improvements for common defects
        let mut defects: Vec<_> = defect_dist.into_iter().collect();
        defects.sort_by(|a, b| b.1.cmp(&a.1));

        for (defect, count) in defects.iter().take(3) {
            if *count > 5 {
                suggestions.push(ImprovementSuggestion {
                    priority: if *count > 20 {
                        SuggestionPriority::High
                    } else {
                        SuggestionPriority::Medium
                    },
                    category: SuggestionCategory::DefectPrevention,
                    description: format!(
                        "Address {} defect category ({} occurrences)",
                        defect, count
                    ),
                    affected_feature: None,
                    affected_defect: Some(defect.clone()),
                });
            }
        }

        suggestions
    }

    /// Generate markdown report.
    pub fn generate_markdown_report(&mut self) -> String {
        let suspicious = self.top_suspicious(10);
        let defect_dist = self.defect_distribution();
        let suggestions = self.generate_suggestions();

        let mut report = String::from("## Error Tracking Report (CITL Analysis)\n\n");

        // Summary
        report.push_str(&format!(
            "### Summary\n\n\
            | Metric | Value |\n\
            |--------|-------|\n\
            | Total Errors | {} |\n\
            | Total Successes | {} |\n\
            | Error Rate | {:.1}% |\n\n",
            self.error_count(),
            self.success_count(),
            if self.error_count() + self.success_count() as usize > 0 {
                (self.error_count() as f64 / (self.error_count() + self.success_count() as usize) as f64) * 100.0
            } else {
                0.0
            }
        ));

        // Suspicious features
        report.push_str("### Top Suspicious Features (Tarantula)\n\n");
        report.push_str("| Feature | Score | Failures | Successes |\n");
        report.push_str("|---------|-------|----------|----------|\n");
        for fs in suspicious.iter().take(5) {
            report.push_str(&format!(
                "| {} | {:.2} | {} | {} |\n",
                fs.feature, fs.score, fs.failure_count, fs.success_count
            ));
        }
        report.push('\n');

        // Defect distribution
        report.push_str("### Defect Distribution\n\n");
        let mut defects: Vec<_> = defect_dist.into_iter().collect();
        defects.sort_by(|a, b| b.1.cmp(&a.1));
        for (defect, count) in defects.iter().take(5) {
            report.push_str(&format!("- {}: {} ({:.1}%)\n", defect, count,
                (*count as f64 / self.error_count().max(1) as f64) * 100.0));
        }
        report.push('\n');

        // Suggestions
        if !suggestions.is_empty() {
            report.push_str("### Improvement Suggestions\n\n");
            for (i, s) in suggestions.iter().enumerate() {
                report.push_str(&format!(
                    "{}. **[{:?}]** {}\n",
                    i + 1,
                    s.priority,
                    s.description
                ));
            }
        }

        report
    }
}

/// Priority level for improvement suggestions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionPriority {
    /// Critical - fix immediately
    Critical,
    /// High - fix soon
    High,
    /// Medium - plan to fix
    Medium,
    /// Low - nice to have
    Low,
}

/// Category of improvement suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionCategory {
    /// Improve handling of specific C feature
    FeatureHandling,
    /// Prevent specific defect category
    DefectPrevention,
    /// Add training data for pattern
    TrainingData,
    /// Adjust model configuration
    Configuration,
}

/// An improvement suggestion based on error analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementSuggestion {
    /// Priority level
    pub priority: SuggestionPriority,
    /// Category
    pub category: SuggestionCategory,
    /// Description
    pub description: String,
    /// Affected C feature (if any)
    pub affected_feature: Option<String>,
    /// Affected defect category (if any)
    pub affected_defect: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // InferenceError tests
    // ========================================================================

    #[test]
    fn inference_error_new() {
        let error = InferenceError::new(
            "ptr",
            "test.c",
            42,
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.6,
            OwnershipDefect::PointerMisclassification,
        );

        assert_eq!(error.variable, "ptr");
        assert_eq!(error.source_file, "test.c");
        assert_eq!(error.source_line, 42);
    }

    #[test]
    fn inference_error_with_features() {
        let error = InferenceError::new(
            "ptr",
            "test.c",
            42,
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.6,
            OwnershipDefect::PointerMisclassification,
        )
        .with_features(vec!["malloc_free".to_string(), "pointer_arithmetic".to_string()]);

        assert_eq!(error.c_features.len(), 2);
    }

    #[test]
    fn inference_error_with_rust_error() {
        let error = InferenceError::new(
            "ptr",
            "test.c",
            42,
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.6,
            OwnershipDefect::PointerMisclassification,
        )
        .with_rust_error("E0382");

        assert_eq!(error.rust_error, Some("E0382".to_string()));
    }

    // ========================================================================
    // PatternStats tests
    // ========================================================================

    #[test]
    fn pattern_stats_default() {
        let stats = PatternStats::default();
        assert_eq!(stats.count, 0);
        assert_eq!(stats.failure_rate(), 0.0);
    }

    #[test]
    fn pattern_stats_record() {
        let mut stats = PatternStats::default();
        stats.record(true);  // failure
        stats.record(false); // success
        stats.record(true);  // failure

        assert_eq!(stats.count, 3);
        assert_eq!(stats.failure_count, 2);
        assert_eq!(stats.success_count, 1);
        assert!((stats.failure_rate() - 0.666).abs() < 0.01);
    }

    // ========================================================================
    // FeatureSuspiciousness tests
    // ========================================================================

    #[test]
    fn feature_suspiciousness_thresholds() {
        let low = FeatureSuspiciousness {
            feature: "test".to_string(),
            score: 0.3,
            total_count: 10,
            failure_count: 3,
            success_count: 7,
        };
        assert!(!low.is_suspicious());
        assert!(!low.is_highly_suspicious());

        let medium = FeatureSuspiciousness {
            feature: "test".to_string(),
            score: 0.6,
            total_count: 10,
            failure_count: 6,
            success_count: 4,
        };
        assert!(medium.is_suspicious());
        assert!(!medium.is_highly_suspicious());

        let high = FeatureSuspiciousness {
            feature: "test".to_string(),
            score: 0.8,
            total_count: 10,
            failure_count: 8,
            success_count: 2,
        };
        assert!(high.is_suspicious());
        assert!(high.is_highly_suspicious());
    }

    // ========================================================================
    // ErrorTracker tests
    // ========================================================================

    #[test]
    fn error_tracker_new() {
        let tracker = ErrorTracker::new();
        assert_eq!(tracker.error_count(), 0);
        assert_eq!(tracker.success_count(), 0);
    }

    #[test]
    fn error_tracker_record_error() {
        let mut tracker = ErrorTracker::new();

        let error = InferenceError::new(
            "ptr",
            "test.c",
            42,
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.6,
            OwnershipDefect::PointerMisclassification,
        )
        .with_features(vec!["malloc_free".to_string()]);

        tracker.record_error(error);

        assert_eq!(tracker.error_count(), 1);
        assert_eq!(tracker.errors()[0].id, 1);
    }

    #[test]
    fn error_tracker_record_success() {
        let mut tracker = ErrorTracker::new();

        tracker.record_success(&["malloc_free".to_string()]);

        assert_eq!(tracker.success_count(), 1);
        assert!(tracker.feature_stats.contains_key("malloc_free"));
    }

    #[test]
    fn error_tracker_calculate_suspiciousness() {
        let mut tracker = ErrorTracker::new();

        // Feature A: 8 failures, 2 successes (high suspiciousness)
        for _ in 0..8 {
            let error = InferenceError::new(
                "ptr",
                "test.c",
                42,
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.6,
                OwnershipDefect::PointerMisclassification,
            )
            .with_features(vec!["feature_a".to_string()]);
            tracker.record_error(error);
        }
        for _ in 0..2 {
            tracker.record_success(&["feature_a".to_string()]);
        }

        // Feature B: 2 failures, 8 successes (low suspiciousness)
        for _ in 0..2 {
            let error = InferenceError::new(
                "ptr",
                "test.c",
                42,
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.6,
                OwnershipDefect::PointerMisclassification,
            )
            .with_features(vec!["feature_b".to_string()]);
            tracker.record_error(error);
        }
        for _ in 0..8 {
            tracker.record_success(&["feature_b".to_string()]);
        }

        let suspicious = tracker.calculate_suspiciousness();

        // Feature A should be more suspicious than Feature B
        let feature_a = suspicious.iter().find(|s| s.feature == "feature_a").unwrap();
        let feature_b = suspicious.iter().find(|s| s.feature == "feature_b").unwrap();

        assert!(feature_a.score > feature_b.score);
    }

    #[test]
    fn error_tracker_top_suspicious() {
        let mut tracker = ErrorTracker::new();

        // Add errors with different features
        for (i, feature) in ["a", "b", "c"].iter().enumerate() {
            for _ in 0..(i + 1) * 3 {
                let error = InferenceError::new(
                    "ptr",
                    "test.c",
                    42,
                    InferredOwnership::Borrowed,
                    InferredOwnership::Owned,
                    0.6,
                    OwnershipDefect::PointerMisclassification,
                )
                .with_features(vec![feature.to_string()]);
                tracker.record_error(error);
            }
        }

        let top = tracker.top_suspicious(2);
        assert_eq!(top.len(), 2);
    }

    #[test]
    fn error_tracker_errors_by_defect() {
        let mut tracker = ErrorTracker::new();

        // Add two different defect types
        let error1 = InferenceError::new(
            "ptr1",
            "test.c",
            1,
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.6,
            OwnershipDefect::PointerMisclassification,
        );
        let error2 = InferenceError::new(
            "ptr2",
            "test.c",
            2,
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.6,
            OwnershipDefect::LifetimeInferenceGap,
        );
        let error3 = InferenceError::new(
            "ptr3",
            "test.c",
            3,
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.6,
            OwnershipDefect::PointerMisclassification,
        );

        tracker.record_error(error1);
        tracker.record_error(error2);
        tracker.record_error(error3);

        let pointer_errors = tracker.errors_by_defect(&OwnershipDefect::PointerMisclassification);
        assert_eq!(pointer_errors.len(), 2);
    }

    #[test]
    fn error_tracker_errors_by_feature() {
        let mut tracker = ErrorTracker::new();

        let error1 = InferenceError::new(
            "ptr1",
            "test.c",
            1,
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.6,
            OwnershipDefect::PointerMisclassification,
        )
        .with_features(vec!["malloc_free".to_string()]);

        let error2 = InferenceError::new(
            "ptr2",
            "test.c",
            2,
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.6,
            OwnershipDefect::PointerMisclassification,
        )
        .with_features(vec!["pointer_arithmetic".to_string()]);

        tracker.record_error(error1);
        tracker.record_error(error2);

        let malloc_errors = tracker.errors_by_feature("malloc_free");
        assert_eq!(malloc_errors.len(), 1);
    }

    #[test]
    fn error_tracker_defect_distribution() {
        let mut tracker = ErrorTracker::new();

        for _ in 0..3 {
            tracker.record_error(InferenceError::new(
                "ptr",
                "test.c",
                1,
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.6,
                OwnershipDefect::PointerMisclassification,
            ));
        }
        for _ in 0..2 {
            tracker.record_error(InferenceError::new(
                "ptr",
                "test.c",
                1,
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.6,
                OwnershipDefect::LifetimeInferenceGap,
            ));
        }

        let dist = tracker.defect_distribution();
        assert_eq!(dist.get("PointerMisclassification"), Some(&3));
        assert_eq!(dist.get("LifetimeInferenceGap"), Some(&2));
    }

    #[test]
    fn error_tracker_generate_suggestions() {
        let mut tracker = ErrorTracker::new();

        // Add many errors for a highly suspicious feature
        for _ in 0..25 {
            let error = InferenceError::new(
                "ptr",
                "test.c",
                42,
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.6,
                OwnershipDefect::PointerMisclassification,
            )
            .with_features(vec!["problematic_feature".to_string()]);
            tracker.record_error(error);
        }

        // Add a few successes
        for _ in 0..5 {
            tracker.record_success(&["problematic_feature".to_string()]);
        }

        let suggestions = tracker.generate_suggestions();
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn error_tracker_generate_markdown_report() {
        let mut tracker = ErrorTracker::new();

        let error = InferenceError::new(
            "ptr",
            "test.c",
            42,
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.6,
            OwnershipDefect::PointerMisclassification,
        )
        .with_features(vec!["malloc_free".to_string()]);

        tracker.record_error(error);
        tracker.record_success(&["other_feature".to_string()]);

        let md = tracker.generate_markdown_report();
        assert!(md.contains("Error Tracking Report"));
        assert!(md.contains("CITL Analysis"));
        assert!(md.contains("Tarantula"));
    }

    // ========================================================================
    // ImprovementSuggestion tests
    // ========================================================================

    #[test]
    fn improvement_suggestion_structure() {
        let suggestion = ImprovementSuggestion {
            priority: SuggestionPriority::High,
            category: SuggestionCategory::FeatureHandling,
            description: "Test suggestion".to_string(),
            affected_feature: Some("malloc_free".to_string()),
            affected_defect: None,
        };

        assert_eq!(suggestion.priority, SuggestionPriority::High);
        assert_eq!(suggestion.category, SuggestionCategory::FeatureHandling);
    }
}
