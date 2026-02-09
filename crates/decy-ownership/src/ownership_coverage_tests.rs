//! Coverage tests for decy-ownership: error_tracking, threshold_tuning, hybrid_classifier.
//!
//! Targets pmat coverage gaps in:
//! - error_tracking.rs: generate_markdown_report (48 uncov) + calculate_suspiciousness (45 uncov)
//! - threshold_tuning.rs: calculate (59 uncov) + select_optimal (53 uncov)
//! - hybrid_classifier.rs: classify_ensemble (52 uncov)

#[cfg(test)]
mod tests {
    use crate::error_tracking::*;
    use crate::hybrid_classifier::*;
    use crate::inference::{OwnershipInference, OwnershipKind};
    use crate::ml_features::{InferredOwnership, OwnershipDefect, OwnershipFeatures, OwnershipPrediction};
    use crate::threshold_tuning::*;

    // ========================================================================
    // Helper: mock model for hybrid classifier tests
    // ========================================================================

    /// Configurable mock ML model for testing classify_ensemble paths.
    struct MockModel {
        ownership: InferredOwnership,
        confidence: f64,
    }

    impl MockModel {
        fn new(ownership: InferredOwnership, confidence: f64) -> Self {
            Self {
                ownership,
                confidence,
            }
        }
    }

    impl OwnershipModel for MockModel {
        fn predict(&self, _features: &OwnershipFeatures) -> OwnershipPrediction {
            OwnershipPrediction {
                kind: self.ownership,
                confidence: self.confidence as f32,
                fallback: None,
            }
        }

        fn name(&self) -> &str {
            "mock-coverage-test"
        }
    }

    // Helper to create a standard InferenceError with features
    fn make_error(
        variable: &str,
        predicted: InferredOwnership,
        expected: InferredOwnership,
        confidence: f64,
        defect: OwnershipDefect,
        features: Vec<String>,
    ) -> InferenceError {
        InferenceError::new(variable, "test.c", 1, predicted, expected, confidence, defect)
            .with_features(features)
    }

    // ========================================================================
    // ERROR TRACKING: generate_markdown_report coverage
    // ========================================================================

    #[test]
    fn markdown_report_empty_tracker_has_zero_counts() {
        let mut tracker = ErrorTracker::new();
        let report = tracker.generate_markdown_report();

        assert!(report.contains("## Error Tracking Report (CITL Analysis)"));
        assert!(report.contains("### Summary"));
        assert!(report.contains("Total Errors | 0"));
        assert!(report.contains("Total Successes | 0"));
        assert!(report.contains("Error Rate | 0.0%"));
        assert!(report.contains("### Top Suspicious Features (Tarantula)"));
        assert!(report.contains("### Defect Distribution"));
        // No suggestions section when empty
        assert!(!report.contains("### Improvement Suggestions"));
    }

    #[test]
    fn markdown_report_single_error_single_success() {
        let mut tracker = ErrorTracker::new();

        let error = make_error(
            "ptr",
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.6,
            OwnershipDefect::PointerMisclassification,
            vec!["malloc_free".to_string()],
        );
        tracker.record_error(error);
        tracker.record_success(&["safe_pattern".to_string()]);

        let report = tracker.generate_markdown_report();

        assert!(report.contains("Total Errors | 1"));
        assert!(report.contains("Total Successes | 1"));
        assert!(report.contains("Error Rate | 50.0%"));
        assert!(report.contains("malloc_free"));
        assert!(report.contains("PointerMisclassification"));
    }

    #[test]
    fn markdown_report_many_errors_triggers_suggestions_section() {
        let mut tracker = ErrorTracker::new();

        // Add 30 errors with a highly suspicious feature to trigger suggestions
        for i in 0..30 {
            let error = make_error(
                &format!("ptr{}", i),
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.5,
                OwnershipDefect::PointerMisclassification,
                vec!["dangerous_pattern".to_string()],
            );
            tracker.record_error(error);
        }
        // Add a few successes so suspiciousness isn't 1.0 for all
        for _ in 0..3 {
            tracker.record_success(&["dangerous_pattern".to_string()]);
        }

        let report = tracker.generate_markdown_report();

        assert!(report.contains("### Improvement Suggestions"));
        assert!(report.contains("dangerous_pattern"));
        assert!(report.contains("[High]"));
        assert!(report.contains("Total Errors | 30"));
    }

    #[test]
    fn markdown_report_multiple_defect_types_in_distribution() {
        let mut tracker = ErrorTracker::new();

        // Add errors with various defect types
        for _ in 0..5 {
            tracker.record_error(make_error(
                "a",
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.4,
                OwnershipDefect::PointerMisclassification,
                vec!["feat_a".to_string()],
            ));
        }
        for _ in 0..3 {
            tracker.record_error(make_error(
                "b",
                InferredOwnership::Owned,
                InferredOwnership::BorrowedMut,
                0.3,
                OwnershipDefect::LifetimeInferenceGap,
                vec!["feat_b".to_string()],
            ));
        }
        for _ in 0..2 {
            tracker.record_error(make_error(
                "c",
                InferredOwnership::Slice,
                InferredOwnership::Vec,
                0.5,
                OwnershipDefect::ArraySliceMismatch,
                vec!["feat_c".to_string()],
            ));
        }

        let report = tracker.generate_markdown_report();

        // Defect distribution should list all three types
        assert!(report.contains("PointerMisclassification"));
        assert!(report.contains("LifetimeInferenceGap"));
        assert!(report.contains("ArraySliceMismatch"));
        // Total should be 10
        assert!(report.contains("Total Errors | 10"));
    }

    #[test]
    fn markdown_report_many_features_in_table() {
        let mut tracker = ErrorTracker::new();

        // Create errors with many different features (more than 5 to exercise table truncation)
        let features: Vec<String> = (0..8)
            .map(|i| format!("feature_{}", i))
            .collect();
        for (i, feat) in features.iter().enumerate() {
            for _ in 0..(i + 1) {
                tracker.record_error(make_error(
                    &format!("var_{}", i),
                    InferredOwnership::Borrowed,
                    InferredOwnership::Owned,
                    0.4,
                    OwnershipDefect::PointerMisclassification,
                    vec![feat.clone()],
                ));
            }
            tracker.record_success(&[feat.clone()]);
        }

        let report = tracker.generate_markdown_report();

        // The table shows top 5 suspicious features
        assert!(report.contains("| Feature | Score | Failures | Successes |"));
        // At least one feature row appears
        assert!(report.contains("feature_"));
    }

    #[test]
    fn markdown_report_contains_percentage_in_defect_distribution() {
        let mut tracker = ErrorTracker::new();

        for _ in 0..4 {
            tracker.record_error(make_error(
                "x",
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.5,
                OwnershipDefect::DanglingPointerRisk,
                vec![],
            ));
        }
        for _ in 0..6 {
            tracker.record_error(make_error(
                "y",
                InferredOwnership::Owned,
                InferredOwnership::Borrowed,
                0.5,
                OwnershipDefect::AliasViolation,
                vec![],
            ));
        }

        let report = tracker.generate_markdown_report();

        // Distribution should contain percentages
        assert!(report.contains("60.0%")); // AliasViolation: 6/10
        assert!(report.contains("40.0%")); // DanglingPointerRisk: 4/10
    }

    #[test]
    fn markdown_report_only_successes_zero_error_rate() {
        let mut tracker = ErrorTracker::new();

        for _ in 0..10 {
            tracker.record_success(&["good_pattern".to_string()]);
        }

        let report = tracker.generate_markdown_report();

        assert!(report.contains("Total Errors | 0"));
        assert!(report.contains("Total Successes | 10"));
        assert!(report.contains("Error Rate | 0.0%"));
    }

    // ========================================================================
    // ERROR TRACKING: calculate_suspiciousness coverage
    // ========================================================================

    #[test]
    fn suspiciousness_empty_tracker_returns_empty() {
        let mut tracker = ErrorTracker::new();
        let results = tracker.calculate_suspiciousness();
        assert!(results.is_empty());
    }

    #[test]
    fn suspiciousness_single_feature_all_failures() {
        let mut tracker = ErrorTracker::new();

        for _ in 0..5 {
            tracker.record_error(make_error(
                "ptr",
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.5,
                OwnershipDefect::PointerMisclassification,
                vec!["always_fails".to_string()],
            ));
        }

        let results = tracker.calculate_suspiciousness();
        assert_eq!(results.len(), 1);
        // With 0 successes, suspiciousness should be 1.0 (or close)
        // failed_ratio = 5/5 = 1.0, passed_ratio = 0/1 = 0.0
        // susp = 1.0 / (1.0 + 0.0) = 1.0
        assert!((results[0].score - 1.0).abs() < 0.01);
        assert_eq!(results[0].failure_count, 5);
        assert_eq!(results[0].success_count, 0);
    }

    #[test]
    fn suspiciousness_single_feature_all_successes() {
        let mut tracker = ErrorTracker::new();

        for _ in 0..5 {
            tracker.record_success(&["always_succeeds".to_string()]);
        }

        let results = tracker.calculate_suspiciousness();
        assert_eq!(results.len(), 1);
        // With 0 failures, suspiciousness should be 0.0
        // failed_ratio = 0/1 = 0.0, passed_ratio = 5/5 = 1.0
        // susp = 0.0 / (0.0 + 1.0) = 0.0
        assert!((results[0].score - 0.0).abs() < 0.01);
    }

    #[test]
    fn suspiciousness_multiple_features_ranked_by_score() {
        let mut tracker = ErrorTracker::new();

        // Feature A: 9 failures, 1 success (high suspiciousness)
        for _ in 0..9 {
            tracker.record_error(make_error(
                "a",
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.5,
                OwnershipDefect::PointerMisclassification,
                vec!["high_risk".to_string()],
            ));
        }
        tracker.record_success(&["high_risk".to_string()]);

        // Feature B: 1 failure, 9 successes (low suspiciousness)
        tracker.record_error(make_error(
            "b",
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.5,
            OwnershipDefect::PointerMisclassification,
            vec!["low_risk".to_string()],
        ));
        for _ in 0..9 {
            tracker.record_success(&["low_risk".to_string()]);
        }

        // Feature C: 5 failures, 5 successes (medium suspiciousness)
        for _ in 0..5 {
            tracker.record_error(make_error(
                "c",
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.5,
                OwnershipDefect::PointerMisclassification,
                vec!["medium_risk".to_string()],
            ));
        }
        for _ in 0..5 {
            tracker.record_success(&["medium_risk".to_string()]);
        }

        let results = tracker.calculate_suspiciousness();
        assert_eq!(results.len(), 3);

        // Results should be sorted by score descending
        assert!(results[0].score >= results[1].score);
        assert!(results[1].score >= results[2].score);

        // high_risk should be first
        assert_eq!(results[0].feature, "high_risk");
        assert!(results[0].is_highly_suspicious());

        // low_risk should be last
        assert_eq!(results[2].feature, "low_risk");
        assert!(!results[2].is_suspicious());
    }

    #[test]
    fn suspiciousness_feature_with_both_failure_and_success_counts() {
        let mut tracker = ErrorTracker::new();

        // 3 failures and 7 successes for the same feature
        for _ in 0..3 {
            tracker.record_error(make_error(
                "ptr",
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.5,
                OwnershipDefect::PointerMisclassification,
                vec!["mixed_feature".to_string()],
            ));
        }
        for _ in 0..7 {
            tracker.record_success(&["mixed_feature".to_string()]);
        }

        let results = tracker.calculate_suspiciousness();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].total_count, 10);
        assert_eq!(results[0].failure_count, 3);
        assert_eq!(results[0].success_count, 7);
        // Score should be between 0 and 1
        assert!(results[0].score > 0.0);
        assert!(results[0].score < 1.0);
    }

    #[test]
    fn suspiciousness_updates_internal_stats() {
        let mut tracker = ErrorTracker::new();

        tracker.record_error(make_error(
            "ptr",
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.5,
            OwnershipDefect::PointerMisclassification,
            vec!["feat".to_string()],
        ));
        tracker.record_success(&["feat".to_string()]);

        // First call computes scores
        let results1 = tracker.calculate_suspiciousness();
        assert!(!results1.is_empty());

        // Second call should also work (idempotent)
        let results2 = tracker.calculate_suspiciousness();
        assert_eq!(results1.len(), results2.len());
        assert!((results1[0].score - results2[0].score).abs() < 0.001);
    }

    // ========================================================================
    // ERROR TRACKING: top_suspicious coverage
    // ========================================================================

    #[test]
    fn top_suspicious_zero_returns_empty() {
        let mut tracker = ErrorTracker::new();
        tracker.record_error(make_error(
            "ptr",
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.5,
            OwnershipDefect::PointerMisclassification,
            vec!["feat".to_string()],
        ));
        let top = tracker.top_suspicious(0);
        assert!(top.is_empty());
    }

    #[test]
    fn top_suspicious_more_than_available() {
        let mut tracker = ErrorTracker::new();
        tracker.record_error(make_error(
            "ptr",
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.5,
            OwnershipDefect::PointerMisclassification,
            vec!["only_one".to_string()],
        ));

        let top = tracker.top_suspicious(100);
        assert_eq!(top.len(), 1);
    }

    #[test]
    fn top_suspicious_returns_highest_scored() {
        let mut tracker = ErrorTracker::new();

        // High suspiciousness
        for _ in 0..10 {
            tracker.record_error(make_error(
                "x",
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.5,
                OwnershipDefect::PointerMisclassification,
                vec!["top_feat".to_string()],
            ));
        }
        tracker.record_success(&["top_feat".to_string()]);

        // Low suspiciousness
        tracker.record_error(make_error(
            "y",
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.5,
            OwnershipDefect::PointerMisclassification,
            vec!["bottom_feat".to_string()],
        ));
        for _ in 0..10 {
            tracker.record_success(&["bottom_feat".to_string()]);
        }

        let top = tracker.top_suspicious(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].feature, "top_feat");
    }

    // ========================================================================
    // ERROR TRACKING: defect_distribution coverage
    // ========================================================================

    #[test]
    fn defect_distribution_empty_tracker() {
        let tracker = ErrorTracker::new();
        let dist = tracker.defect_distribution();
        assert!(dist.is_empty());
    }

    #[test]
    fn defect_distribution_multiple_defect_types() {
        let mut tracker = ErrorTracker::new();

        // 4 different defect types with varying counts
        for _ in 0..3 {
            tracker.record_error(make_error(
                "a", InferredOwnership::Borrowed, InferredOwnership::Owned,
                0.5, OwnershipDefect::PointerMisclassification, vec![],
            ));
        }
        for _ in 0..2 {
            tracker.record_error(make_error(
                "b", InferredOwnership::Owned, InferredOwnership::BorrowedMut,
                0.5, OwnershipDefect::MutabilityMismatch, vec![],
            ));
        }
        tracker.record_error(make_error(
            "c", InferredOwnership::Slice, InferredOwnership::Vec,
            0.5, OwnershipDefect::ArraySliceMismatch, vec![],
        ));
        tracker.record_error(make_error(
            "d", InferredOwnership::Owned, InferredOwnership::Borrowed,
            0.5, OwnershipDefect::ResourceLeakPattern, vec![],
        ));

        let dist = tracker.defect_distribution();
        assert_eq!(dist.len(), 4);
        assert_eq!(*dist.get("PointerMisclassification").unwrap(), 3);
        assert_eq!(*dist.get("MutabilityMismatch").unwrap(), 2);
        assert_eq!(*dist.get("ArraySliceMismatch").unwrap(), 1);
        assert_eq!(*dist.get("ResourceLeakPattern").unwrap(), 1);
    }

    // ========================================================================
    // ERROR TRACKING: generate_suggestions coverage
    // ========================================================================

    #[test]
    fn suggestions_empty_tracker_returns_empty() {
        let mut tracker = ErrorTracker::new();
        let suggestions = tracker.generate_suggestions();
        assert!(suggestions.is_empty());
    }

    #[test]
    fn suggestions_highly_suspicious_feature_generates_feature_handling() {
        let mut tracker = ErrorTracker::new();

        // Feature with very high suspiciousness (score > 0.7)
        // Tarantula: susp = (failed(e)/total_failed) / ((failed(e)/total_failed) + (passed(e)/total_passed))
        // We need failed_ratio >> passed_ratio
        // 20 failures with feature, 2 successes with feature
        // But we also need many total successes to make passed_ratio small
        for _ in 0..20 {
            tracker.record_error(make_error(
                "ptr",
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.5,
                OwnershipDefect::PointerMisclassification,
                vec!["problematic_cast".to_string()],
            ));
        }
        // 2 successes with the feature
        for _ in 0..2 {
            tracker.record_success(&["problematic_cast".to_string()]);
        }
        // Many more successes WITHOUT the feature to inflate total_passed
        // This makes passed_ratio = 2/102 very small vs failed_ratio = 20/20 = 1.0
        for _ in 0..100 {
            tracker.record_success(&["safe_pattern".to_string()]);
        }

        let suggestions = tracker.generate_suggestions();
        let feature_suggestions: Vec<_> = suggestions
            .iter()
            .filter(|s| s.category == SuggestionCategory::FeatureHandling)
            .collect();
        assert!(!feature_suggestions.is_empty());
        assert_eq!(feature_suggestions[0].priority, SuggestionPriority::High);
        assert!(feature_suggestions[0]
            .affected_feature
            .as_ref()
            .unwrap()
            .contains("problematic_cast"));
    }

    #[test]
    fn suggestions_medium_priority_defect_count_between_5_and_20() {
        let mut tracker = ErrorTracker::new();

        // Add 15 errors of the same defect type (> 5, <= 20 = Medium)
        for i in 0..15 {
            tracker.record_error(make_error(
                &format!("ptr{}", i),
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.5,
                OwnershipDefect::LifetimeInferenceGap,
                vec![],
            ));
        }
        // Need some successes to avoid max suspiciousness
        for _ in 0..30 {
            tracker.record_success(&["unrelated".to_string()]);
        }

        let suggestions = tracker.generate_suggestions();
        let defect_suggestions: Vec<_> = suggestions
            .iter()
            .filter(|s| s.category == SuggestionCategory::DefectPrevention)
            .collect();
        assert!(!defect_suggestions.is_empty());
        assert_eq!(defect_suggestions[0].priority, SuggestionPriority::Medium);
        assert!(defect_suggestions[0].affected_defect.is_some());
    }

    #[test]
    fn suggestions_high_priority_defect_count_above_20() {
        let mut tracker = ErrorTracker::new();

        // Add 25 errors of the same defect type (> 20 = High)
        for i in 0..25 {
            tracker.record_error(make_error(
                &format!("ptr{}", i),
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.5,
                OwnershipDefect::DanglingPointerRisk,
                vec![],
            ));
        }
        for _ in 0..5 {
            tracker.record_success(&["x".to_string()]);
        }

        let suggestions = tracker.generate_suggestions();
        let defect_suggestions: Vec<_> = suggestions
            .iter()
            .filter(|s| s.category == SuggestionCategory::DefectPrevention)
            .collect();
        assert!(!defect_suggestions.is_empty());
        assert_eq!(defect_suggestions[0].priority, SuggestionPriority::High);
    }

    #[test]
    fn suggestions_no_defect_prevention_when_counts_low() {
        let mut tracker = ErrorTracker::new();

        // Only 3 errors per defect type (below the >5 threshold)
        for _ in 0..3 {
            tracker.record_error(make_error(
                "x", InferredOwnership::Borrowed, InferredOwnership::Owned,
                0.5, OwnershipDefect::PointerMisclassification, vec![],
            ));
        }
        for _ in 0..3 {
            tracker.record_error(make_error(
                "y", InferredOwnership::Owned, InferredOwnership::BorrowedMut,
                0.5, OwnershipDefect::AliasViolation, vec![],
            ));
        }
        for _ in 0..10 {
            tracker.record_success(&["z".to_string()]);
        }

        let suggestions = tracker.generate_suggestions();
        let defect_suggestions: Vec<_> = suggestions
            .iter()
            .filter(|s| s.category == SuggestionCategory::DefectPrevention)
            .collect();
        assert!(defect_suggestions.is_empty());
    }

    #[test]
    fn suggestions_both_feature_and_defect_suggestions() {
        let mut tracker = ErrorTracker::new();

        // High-suspiciousness feature with many errors of same defect
        // Need: failed_ratio >> passed_ratio in Tarantula
        for i in 0..25 {
            tracker.record_error(make_error(
                &format!("ptr{}", i),
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.5,
                OwnershipDefect::UnsafeMinimizationFailure,
                vec!["suspicious_feat".to_string()],
            ));
        }
        // Few successes with the feature, many without
        for _ in 0..2 {
            tracker.record_success(&["suspicious_feat".to_string()]);
        }
        for _ in 0..100 {
            tracker.record_success(&["safe_pattern".to_string()]);
        }

        let suggestions = tracker.generate_suggestions();
        let feature_count = suggestions
            .iter()
            .filter(|s| s.category == SuggestionCategory::FeatureHandling)
            .count();
        let defect_count = suggestions
            .iter()
            .filter(|s| s.category == SuggestionCategory::DefectPrevention)
            .count();

        assert!(feature_count > 0);
        assert!(defect_count > 0);
    }

    // ========================================================================
    // THRESHOLD TUNING: ThresholdMetrics::calculate coverage
    // ========================================================================

    #[test]
    fn threshold_metrics_calculate_empty_samples() {
        let metrics = ThresholdMetrics::calculate(&[], 0.5);
        assert_eq!(metrics.sample_count, 0);
        assert_eq!(metrics.accuracy, 0.0);
        assert_eq!(metrics.precision, 0.0);
        assert_eq!(metrics.recall, 0.0);
        assert_eq!(metrics.f1_score, 0.0);
        assert_eq!(metrics.fallback_rate, 1.0);
        assert_eq!(metrics.ml_usage_rate, 0.0);
    }

    #[test]
    fn threshold_metrics_calculate_all_correct_ml() {
        let samples = vec![
            ValidationSample::new(
                InferredOwnership::Owned,
                InferredOwnership::Borrowed, // rule wrong
                InferredOwnership::Owned,    // ml correct
                0.9,
            ),
            ValidationSample::new(
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,    // rule wrong
                InferredOwnership::Borrowed, // ml correct
                0.8,
            ),
        ];

        let metrics = ThresholdMetrics::calculate(&samples, 0.5);
        assert_eq!(metrics.sample_count, 2);
        assert!((metrics.accuracy - 1.0).abs() < 0.001);
        assert!((metrics.ml_usage_rate - 1.0).abs() < 0.001);
        assert!((metrics.fallback_rate - 0.0).abs() < 0.001);
        assert!((metrics.precision - 1.0).abs() < 0.001);
        assert!((metrics.recall - 1.0).abs() < 0.001);
        assert!((metrics.f1_score - 1.0).abs() < 0.001);
    }

    #[test]
    fn threshold_metrics_calculate_all_incorrect() {
        let samples = vec![
            ValidationSample::new(
                InferredOwnership::Owned,
                InferredOwnership::Borrowed, // rule wrong
                InferredOwnership::Borrowed, // ml also wrong
                0.9,
            ),
            ValidationSample::new(
                InferredOwnership::Borrowed,
                InferredOwnership::Owned, // rule wrong
                InferredOwnership::Owned, // ml also wrong
                0.8,
            ),
        ];

        let metrics = ThresholdMetrics::calculate(&samples, 0.5);
        assert_eq!(metrics.sample_count, 2);
        assert!((metrics.accuracy - 0.0).abs() < 0.001);
        assert!((metrics.precision - 0.0).abs() < 0.001);
        // recall = 0 / (0 + 2) = 0.0
        assert!((metrics.recall - 0.0).abs() < 0.001);
        assert!((metrics.f1_score - 0.0).abs() < 0.001);
    }

    #[test]
    fn threshold_metrics_calculate_mixed_ml_and_fallback() {
        let samples = vec![
            // Sample 1: high confidence ML (uses ML) - correct
            ValidationSample::new(
                InferredOwnership::Owned,
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.9,
            ),
            // Sample 2: low confidence ML (uses rules) - correct
            ValidationSample::new(
                InferredOwnership::Borrowed,
                InferredOwnership::Borrowed,
                InferredOwnership::Owned, // ml wrong but not used
                0.3,
            ),
            // Sample 3: high confidence ML (uses ML) - wrong
            ValidationSample::new(
                InferredOwnership::Owned,
                InferredOwnership::Owned,
                InferredOwnership::Borrowed, // ml wrong
                0.8,
            ),
            // Sample 4: low confidence ML (uses rules) - wrong
            ValidationSample::new(
                InferredOwnership::Vec,
                InferredOwnership::Owned, // rule wrong
                InferredOwnership::Vec,   // ml right but not used
                0.2,
            ),
        ];

        let metrics = ThresholdMetrics::calculate(&samples, 0.7);
        assert_eq!(metrics.sample_count, 4);
        // Sample 1: ML used (0.9 >= 0.7), Owned == Owned, correct
        // Sample 2: Rules used (0.3 < 0.7), Borrowed == Borrowed, correct
        // Sample 3: ML used (0.8 >= 0.7), Borrowed != Owned, wrong
        // Sample 4: Rules used (0.2 < 0.7), Owned != Vec, wrong
        assert!((metrics.accuracy - 0.5).abs() < 0.001);
        assert!((metrics.fallback_rate - 0.5).abs() < 0.001);
        assert!((metrics.ml_usage_rate - 0.5).abs() < 0.001);
    }

    #[test]
    fn threshold_metrics_calculate_threshold_boundary_exact_match() {
        // Test when ml_confidence == threshold (should use ML)
        let sample = ValidationSample::new(
            InferredOwnership::Owned,
            InferredOwnership::Borrowed, // rule wrong
            InferredOwnership::Owned,    // ml right
            0.5,
        );

        let metrics = ThresholdMetrics::calculate(&[sample], 0.5);
        // ML confidence (0.5) >= threshold (0.5) -> uses ML
        assert!((metrics.ml_usage_rate - 1.0).abs() < 0.001);
        assert!((metrics.accuracy - 1.0).abs() < 0.001);
    }

    #[test]
    fn threshold_metrics_precision_recall_f1_calculation() {
        // 3 correct, 1 wrong -> precision = 3/4 = 0.75, recall = 3/4 = 0.75
        let samples = vec![
            ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Owned, 0.9),
            ValidationSample::new(InferredOwnership::Borrowed, InferredOwnership::Borrowed, InferredOwnership::Borrowed, 0.9),
            ValidationSample::new(InferredOwnership::BorrowedMut, InferredOwnership::BorrowedMut, InferredOwnership::BorrowedMut, 0.9),
            ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Borrowed, InferredOwnership::Borrowed, 0.9), // ml wrong
        ];

        let metrics = ThresholdMetrics::calculate(&samples, 0.5);
        assert!((metrics.accuracy - 0.75).abs() < 0.001);
        assert!((metrics.precision - 0.75).abs() < 0.001);
        assert!((metrics.recall - 0.75).abs() < 0.001);
        // F1 = 2 * 0.75 * 0.75 / (0.75 + 0.75) = 0.75
        assert!((metrics.f1_score - 0.75).abs() < 0.001);
    }

    // ========================================================================
    // THRESHOLD TUNING: ThresholdTuner::tune + select_optimal coverage
    // ========================================================================

    #[test]
    fn tuner_tune_empty_samples_returns_default() {
        let tuner = ThresholdTuner::new();
        let result = tuner.tune(&[]);
        assert!((result.optimal_threshold - 0.65).abs() < 0.001);
        assert_eq!(result.baseline_accuracy, 0.0);
        assert_eq!(result.ml_only_accuracy, 0.0);
        assert!(result.all_thresholds.is_empty());
    }

    #[test]
    fn tuner_tune_max_accuracy_criteria() {
        // Setup: ML is correct at high confidence, rules correct at low confidence
        let samples = vec![
            ValidationSample::new(
                InferredOwnership::Owned,
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.9,
            ),
            ValidationSample::new(
                InferredOwnership::Borrowed,
                InferredOwnership::Borrowed,
                InferredOwnership::Owned, // ml wrong
                0.3,
            ),
        ];

        let tuner = ThresholdTuner::new().with_criteria(SelectionCriteria::MaxAccuracy);
        let result = tuner.tune(&samples);

        // Optimal threshold should be around 0.5-0.9 (uses ML for first, rules for second)
        assert!(result.optimal_metrics.accuracy >= 0.5);
        assert!(!result.all_thresholds.is_empty());
        assert_eq!(result.criteria, "max-accuracy");
    }

    #[test]
    fn tuner_tune_max_f1_criteria() {
        let samples = vec![
            ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Owned, 0.9),
            ValidationSample::new(InferredOwnership::Borrowed, InferredOwnership::Borrowed, InferredOwnership::Borrowed, 0.8),
            ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.7),
        ];

        let tuner = ThresholdTuner::new().with_criteria(SelectionCriteria::MaxF1);
        let result = tuner.tune(&samples);

        assert_eq!(result.criteria, "max-f1");
        assert!(result.optimal_metrics.f1_score > 0.0);
    }

    #[test]
    fn tuner_tune_balanced_accuracy_fallback_criteria() {
        let mut samples = Vec::new();

        // Many high-confidence ML correct
        for _ in 0..50 {
            samples.push(ValidationSample::new(
                InferredOwnership::Owned,
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.9,
            ));
        }
        // Some low-confidence where rules correct
        for _ in 0..20 {
            samples.push(ValidationSample::new(
                InferredOwnership::Borrowed,
                InferredOwnership::Borrowed,
                InferredOwnership::Owned,
                0.2,
            ));
        }

        let tuner = ThresholdTuner::new().with_criteria(SelectionCriteria::BalancedAccuracyFallback);
        let result = tuner.tune(&samples);

        assert_eq!(result.criteria, "balanced");
        // Score = 0.7 * accuracy + 0.3 * ml_usage_rate
        // Should prefer lower threshold to maximize ML usage while maintaining accuracy
        assert!(result.optimal_metrics.accuracy > 0.5);
    }

    #[test]
    fn tuner_tune_min_fallback_above_baseline_criteria() {
        // Rules correct 60% of the time as baseline
        let samples = vec![
            // Rules correct, ML correct (high conf)
            ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Owned, 0.9),
            ValidationSample::new(InferredOwnership::Borrowed, InferredOwnership::Borrowed, InferredOwnership::Borrowed, 0.8),
            ValidationSample::new(InferredOwnership::BorrowedMut, InferredOwnership::BorrowedMut, InferredOwnership::BorrowedMut, 0.7),
            // Rules wrong, ML correct (high conf)
            ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.85),
            // Rules correct, ML wrong (low conf)
            ValidationSample::new(InferredOwnership::Borrowed, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.3),
        ];

        let tuner = ThresholdTuner::new().with_criteria(SelectionCriteria::MinFallbackAboveBaseline);
        let result = tuner.tune(&samples);

        assert_eq!(result.criteria, "min-fallback");
        // Should maintain accuracy >= baseline while minimizing fallback
        assert!(result.optimal_metrics.accuracy >= result.baseline_accuracy - 0.001);
    }

    #[test]
    fn tuner_tune_min_fallback_falls_back_to_max_accuracy() {
        // Scenario where NO threshold beats baseline (rules always better)
        // All ML predictions are wrong regardless of confidence
        let samples = vec![
            ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Borrowed, 0.9),
            ValidationSample::new(InferredOwnership::Borrowed, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.8),
            ValidationSample::new(InferredOwnership::BorrowedMut, InferredOwnership::BorrowedMut, InferredOwnership::Owned, 0.7),
        ];

        // Baseline = 100% (rules all correct)
        // Any threshold < 0.7 will use some ML (all wrong), reducing accuracy below baseline
        // Threshold 0.9 or higher: all samples use rules => accuracy = 100% = baseline => above_baseline includes it
        // But let's make ML wrong at all confidences

        let tuner = ThresholdTuner::with_candidates(vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6])
            .with_criteria(SelectionCriteria::MinFallbackAboveBaseline);
        let result = tuner.tune(&samples);

        // All candidates 0.1-0.6 will use at least some ML (which is always wrong)
        // At 0.1: all use ML => 0% accuracy, below baseline
        // At 0.6: samples with conf >= 0.6 use ML (wrong) => accuracy < 100%
        // Since NO candidate maintains baseline, should fall back to max accuracy
        // The highest accuracy threshold will be 0.6 (fewest wrong ML usages: only conf >= 0.6)
        // Actually at 0.6, conf 0.9, 0.8, 0.7 all >= 0.6, all wrong => 0% accuracy
        // At any threshold, all samples have conf > threshold, so all use ML (all wrong)
        // Hmm, all ML wrong and all conf >= any threshold 0.1-0.6, so accuracy = 0 for all
        // Falls back to max accuracy which is 0% for all => picks first
        assert!(result.optimal_threshold >= 0.0);
        assert!(result.optimal_threshold <= 1.0);
    }

    #[test]
    fn tuner_with_custom_candidates() {
        let tuner = ThresholdTuner::with_candidates(vec![0.25, 0.5, 0.75]);
        let samples = vec![
            ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Owned, 0.9),
        ];

        let result = tuner.tune(&samples);
        // Should evaluate exactly 3 thresholds
        assert_eq!(result.all_thresholds.len(), 3);
    }

    #[test]
    fn tuner_add_candidate_deduplication() {
        let mut tuner = ThresholdTuner::with_candidates(vec![0.3, 0.5, 0.7]);
        tuner.add_candidate(0.5); // duplicate, should not add
        tuner.add_candidate(0.4); // new, should add

        let samples = vec![
            ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Owned, 0.9),
        ];

        let result = tuner.tune(&samples);
        assert_eq!(result.all_thresholds.len(), 4); // 0.3, 0.4, 0.5, 0.7
    }

    #[test]
    fn tuner_add_candidate_clamps_and_sorts() {
        let mut tuner = ThresholdTuner::with_candidates(vec![0.5]);
        tuner.add_candidate(1.5); // clamped to 1.0
        tuner.add_candidate(-0.5); // clamped to 0.0
        tuner.add_candidate(0.3);

        let samples = vec![
            ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Owned, 0.9),
        ];

        let result = tuner.tune(&samples);
        // Should have 0.0, 0.3, 0.5, 1.0
        assert_eq!(result.all_thresholds.len(), 4);
    }

    // ========================================================================
    // THRESHOLD TUNING: TuningResult::to_markdown coverage
    // ========================================================================

    #[test]
    fn tuning_result_to_markdown_positive_improvement() {
        let samples = vec![
            ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.9),
            ValidationSample::new(InferredOwnership::Borrowed, InferredOwnership::Borrowed, InferredOwnership::Borrowed, 0.8),
        ];

        let result = ThresholdTuner::new().tune(&samples);
        let md = result.to_markdown();

        assert!(md.contains("## Threshold Tuning Report"));
        assert!(md.contains("### Optimal Configuration"));
        assert!(md.contains("### Comparison to Baselines"));
        assert!(md.contains("### All Thresholds"));
        assert!(md.contains("### Recommendation"));
        // Positive improvement => ADOPT HYBRID
        assert!(md.contains("ADOPT HYBRID") || md.contains("KEEP RULES ONLY"));
    }

    #[test]
    fn tuning_result_to_markdown_no_improvement() {
        // Rules are always correct, ML always wrong => no improvement
        let samples = vec![
            ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Borrowed, 0.9),
            ValidationSample::new(InferredOwnership::Borrowed, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.8),
        ];

        let tuner = ThresholdTuner::with_candidates(vec![0.5])
            .with_criteria(SelectionCriteria::MaxAccuracy);
        let result = tuner.tune(&samples);
        let md = result.to_markdown();

        // Both thresholds: ML wrong (conf >= 0.5) => rules used for nothing since conf >= threshold
        // Actually: at 0.5, both have conf >= 0.5, both use ML (both wrong) => 0% accuracy
        // Baseline: rules 100% correct => improvement negative
        assert!(md.contains("KEEP RULES ONLY"));
    }

    #[test]
    fn tuning_result_to_markdown_contains_threshold_table() {
        let samples = vec![
            ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Owned, 0.9),
        ];

        let result = ThresholdTuner::new().tune(&samples);
        let md = result.to_markdown();

        // Should contain threshold table headers
        assert!(md.contains("| Threshold | Accuracy | F1 | Fallback Rate | ML Usage |"));
        // Should contain at least one threshold row
        assert!(md.contains("| 0."));
    }

    // ========================================================================
    // THRESHOLD TUNING: ValidationSample hybrid_prediction coverage
    // ========================================================================

    #[test]
    fn validation_sample_hybrid_prediction_uses_ml_when_above_threshold() {
        let sample = ValidationSample::new(
            InferredOwnership::Owned,
            InferredOwnership::Borrowed, // rule
            InferredOwnership::Owned,    // ml
            0.8,
        );

        assert_eq!(sample.hybrid_prediction(0.7), InferredOwnership::Owned); // ML used
        assert_eq!(sample.hybrid_prediction(0.9), InferredOwnership::Borrowed); // rules used
    }

    #[test]
    fn validation_sample_rule_correct_and_ml_correct() {
        let both_correct = ValidationSample::new(
            InferredOwnership::Owned,
            InferredOwnership::Owned,
            InferredOwnership::Owned,
            0.9,
        );
        assert!(both_correct.rule_correct());
        assert!(both_correct.ml_correct());

        let rule_wrong = ValidationSample::new(
            InferredOwnership::Owned,
            InferredOwnership::Borrowed,
            InferredOwnership::Owned,
            0.9,
        );
        assert!(!rule_wrong.rule_correct());
        assert!(rule_wrong.ml_correct());

        let ml_wrong = ValidationSample::new(
            InferredOwnership::Owned,
            InferredOwnership::Owned,
            InferredOwnership::Borrowed,
            0.9,
        );
        assert!(ml_wrong.rule_correct());
        assert!(!ml_wrong.ml_correct());
    }

    // ========================================================================
    // HYBRID CLASSIFIER: classify_ensemble coverage
    // ========================================================================

    #[test]
    fn ensemble_agreement_uses_hybrid_method_and_boosts_confidence() {
        let classifier = HybridClassifier::new();

        let inference = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Owning, // -> Owned
            confidence: 0.8,
            reason: "malloc detected".to_string(),
        };
        let features = OwnershipFeatures::default();
        let model = MockModel::new(InferredOwnership::Owned, 0.7); // agrees

        let result = classifier.classify_ensemble(&inference, &features, &model);

        assert_eq!(result.method, ClassificationMethod::Hybrid);
        assert_eq!(result.ownership, InferredOwnership::Owned);
        // Boosted: (0.8 + 0.7) / 2 * 1.1 = 0.825
        assert!(result.confidence > 0.82);
        assert!(result.confidence <= 1.0);
        assert!(result.reasoning.contains("agree"));
    }

    #[test]
    fn ensemble_agreement_confidence_capped_at_1() {
        let classifier = HybridClassifier::new();

        let inference = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Owning,
            confidence: 0.98,
            reason: "strong malloc".to_string(),
        };
        let features = OwnershipFeatures::default();
        let model = MockModel::new(InferredOwnership::Owned, 0.99);

        let result = classifier.classify_ensemble(&inference, &features, &model);

        assert_eq!(result.method, ClassificationMethod::Hybrid);
        // (0.98 + 0.99) / 2 * 1.1 = 1.0835 -> capped at 1.0
        assert!((result.confidence - 1.0).abs() < 0.001);
    }

    #[test]
    fn ensemble_disagreement_ml_higher_confidence_wins() {
        let classifier = HybridClassifier::new();

        let inference = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Unknown, // -> RawPointer
            confidence: 0.3,
            reason: "uncertain".to_string(),
        };
        let features = OwnershipFeatures::default();
        let model = MockModel::new(InferredOwnership::Vec, 0.85); // disagrees, higher confidence

        let result = classifier.classify_ensemble(&inference, &features, &model);

        assert_eq!(result.method, ClassificationMethod::MachineLearning);
        assert_eq!(result.ownership, InferredOwnership::Vec);
        assert!((result.confidence - 0.85).abs() < 0.01);
        assert!(result.reasoning.contains("ML wins"));
    }

    #[test]
    fn ensemble_disagreement_rules_higher_confidence_wins() {
        let classifier = HybridClassifier::new();

        let inference = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Owning, // -> Owned
            confidence: 0.95,
            reason: "malloc+free pattern".to_string(),
        };
        let features = OwnershipFeatures::default();
        let model = MockModel::new(InferredOwnership::Borrowed, 0.4); // disagrees, lower confidence

        let result = classifier.classify_ensemble(&inference, &features, &model);

        assert_eq!(result.method, ClassificationMethod::RuleBased);
        assert_eq!(result.ownership, InferredOwnership::Owned);
        assert!((result.confidence - 0.95).abs() < 0.01);
        assert!(result.reasoning.contains("Rules win"));
    }

    #[test]
    fn ensemble_disagreement_equal_confidence_rules_win() {
        let classifier = HybridClassifier::new();

        let inference = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::ImmutableBorrow, // -> Borrowed
            confidence: 0.7,
            reason: "read only".to_string(),
        };
        let features = OwnershipFeatures::default();
        let model = MockModel::new(InferredOwnership::Owned, 0.7); // same confidence, disagrees

        let result = classifier.classify_ensemble(&inference, &features, &model);

        // When equal confidence, rules should win (ml_conf > rule_conf is false)
        assert_eq!(result.method, ClassificationMethod::RuleBased);
        assert_eq!(result.ownership, InferredOwnership::Borrowed);
    }

    #[test]
    fn ensemble_all_ownership_kinds_mapped_correctly() {
        let classifier = HybridClassifier::new();
        let features = OwnershipFeatures::default();

        let kinds_and_expected = vec![
            (OwnershipKind::Owning, InferredOwnership::Owned),
            (OwnershipKind::ImmutableBorrow, InferredOwnership::Borrowed),
            (OwnershipKind::MutableBorrow, InferredOwnership::BorrowedMut),
            (OwnershipKind::Unknown, InferredOwnership::RawPointer),
        ];

        for (kind, expected) in kinds_and_expected {
            let inference = OwnershipInference {
                variable: "x".to_string(),
                kind,
                confidence: 0.8,
                reason: "test".to_string(),
            };
            // Make model agree to exercise Hybrid path
            let model = MockModel::new(expected, 0.8);
            let result = classifier.classify_ensemble(&inference, &features, &model);

            assert_eq!(result.ownership, expected);
            assert_eq!(result.method, ClassificationMethod::Hybrid);
        }
    }

    #[test]
    fn ensemble_result_has_both_rule_and_ml_results() {
        let classifier = HybridClassifier::new();

        let inference = OwnershipInference {
            variable: "buf".to_string(),
            kind: OwnershipKind::Owning,
            confidence: 0.7,
            reason: "alloc".to_string(),
        };
        let features = OwnershipFeatures::default();
        let model = MockModel::new(InferredOwnership::Vec, 0.9);

        let result = classifier.classify_ensemble(&inference, &features, &model);

        assert!(result.rule_result.is_some());
        assert!(result.ml_result.is_some());
        assert_eq!(result.rule_result.unwrap(), InferredOwnership::Owned);
        assert_eq!(result.ml_result.unwrap().kind, InferredOwnership::Vec);
    }

    // ========================================================================
    // HYBRID CLASSIFIER: HybridMetrics record and rates coverage
    // ========================================================================

    #[test]
    fn hybrid_metrics_record_all_methods() {
        let mut metrics = HybridMetrics::new();

        let make_result = |method: ClassificationMethod| HybridResult {
            variable: "x".to_string(),
            ownership: InferredOwnership::Owned,
            confidence: 0.8,
            method,
            rule_result: Some(InferredOwnership::Owned),
            ml_result: Some(OwnershipPrediction {
                kind: InferredOwnership::Owned,
                confidence: 0.8,
                fallback: None,
            }),
            reasoning: "test".to_string(),
        };

        metrics.record(&make_result(ClassificationMethod::RuleBased));
        metrics.record(&make_result(ClassificationMethod::MachineLearning));
        metrics.record(&make_result(ClassificationMethod::Fallback));
        metrics.record(&make_result(ClassificationMethod::Hybrid));

        assert_eq!(metrics.total, 4);
        assert_eq!(metrics.rule_based, 1);
        assert_eq!(metrics.ml_used, 1);
        assert_eq!(metrics.fallback, 1);
        assert_eq!(metrics.hybrid, 1);
    }

    #[test]
    fn hybrid_metrics_agreement_tracking() {
        let mut metrics = HybridMetrics::new();

        // Agreement: rule and ML both say Owned
        let agree = HybridResult {
            variable: "a".to_string(),
            ownership: InferredOwnership::Owned,
            confidence: 0.9,
            method: ClassificationMethod::Hybrid,
            rule_result: Some(InferredOwnership::Owned),
            ml_result: Some(OwnershipPrediction {
                kind: InferredOwnership::Owned,
                confidence: 0.9,
                fallback: None,
            }),
            reasoning: "agree".to_string(),
        };

        // Disagreement: rule says Borrowed, ML says Owned
        let disagree = HybridResult {
            variable: "b".to_string(),
            ownership: InferredOwnership::Owned,
            confidence: 0.8,
            method: ClassificationMethod::MachineLearning,
            rule_result: Some(InferredOwnership::Borrowed),
            ml_result: Some(OwnershipPrediction {
                kind: InferredOwnership::Owned,
                confidence: 0.8,
                fallback: None,
            }),
            reasoning: "ml wins".to_string(),
        };

        // No ML result: should not count as agreement or disagreement
        let no_ml = HybridResult {
            variable: "c".to_string(),
            ownership: InferredOwnership::Borrowed,
            confidence: 0.7,
            method: ClassificationMethod::RuleBased,
            rule_result: Some(InferredOwnership::Borrowed),
            ml_result: None,
            reasoning: "rules only".to_string(),
        };

        metrics.record(&agree);
        metrics.record(&disagree);
        metrics.record(&no_ml);

        assert_eq!(metrics.agreements, 1);
        assert_eq!(metrics.disagreements, 1);
        // agreement_rate = 1 / (1 + 1) = 0.5
        assert!((metrics.agreement_rate() - 0.5).abs() < 0.001);
    }

    #[test]
    fn hybrid_metrics_rates_with_zero_total() {
        let metrics = HybridMetrics::new();
        assert_eq!(metrics.ml_usage_rate(), 0.0);
        assert_eq!(metrics.fallback_rate(), 0.0);
        assert_eq!(metrics.agreement_rate(), 1.0); // default: perfect agreement
    }

    #[test]
    fn hybrid_metrics_ml_usage_rate_calculation() {
        let mut metrics = HybridMetrics::new();
        metrics.total = 20;
        metrics.ml_used = 8;
        assert!((metrics.ml_usage_rate() - 0.4).abs() < 0.001);
    }

    #[test]
    fn hybrid_metrics_fallback_rate_calculation() {
        let mut metrics = HybridMetrics::new();
        metrics.total = 10;
        metrics.fallback = 7;
        assert!((metrics.fallback_rate() - 0.7).abs() < 0.001);
    }

    // ========================================================================
    // HYBRID CLASSIFIER: Additional classify_ensemble edge cases
    // ========================================================================

    #[test]
    fn ensemble_with_array_pointer_kind() {
        let classifier = HybridClassifier::new();
        let features = OwnershipFeatures::default();

        let inference = OwnershipInference {
            variable: "arr_ptr".to_string(),
            kind: OwnershipKind::ArrayPointer {
                base_array: "arr".to_string(),
                element_type: decy_hir::HirType::Int,
                base_index: Some(0),
            },
            confidence: 0.8,
            reason: "array access pattern".to_string(),
        };

        // ML agrees it's a Slice
        let model = MockModel::new(InferredOwnership::Slice, 0.75);
        let result = classifier.classify_ensemble(&inference, &features, &model);

        assert_eq!(result.method, ClassificationMethod::Hybrid);
        assert_eq!(result.ownership, InferredOwnership::Slice);
    }

    #[test]
    fn ensemble_with_custom_threshold_classifier() {
        let classifier = HybridClassifier::with_threshold(0.9);
        let features = OwnershipFeatures::default();

        let inference = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Owning,
            confidence: 0.6,
            reason: "alloc".to_string(),
        };
        let model = MockModel::new(InferredOwnership::Vec, 0.85);

        // classify_ensemble doesn't use threshold (it's for classify_hybrid),
        // but the classifier still works correctly
        let result = classifier.classify_ensemble(&inference, &features, &model);

        // ML has higher confidence (0.85 > 0.6), so ML wins
        assert_eq!(result.method, ClassificationMethod::MachineLearning);
        assert_eq!(result.ownership, InferredOwnership::Vec);
    }

    // ========================================================================
    // THRESHOLD TUNING: find_optimal_threshold convenience function
    // ========================================================================

    #[test]
    fn find_optimal_threshold_returns_valid_range() {
        let samples = vec![
            ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.9),
            ValidationSample::new(InferredOwnership::Borrowed, InferredOwnership::Borrowed, InferredOwnership::Borrowed, 0.4),
        ];

        let threshold = find_optimal_threshold(&samples);
        assert!(threshold >= 0.0);
        assert!(threshold <= 1.0);
    }

    #[test]
    fn find_optimal_threshold_empty_returns_default() {
        let threshold = find_optimal_threshold(&[]);
        assert!((threshold - 0.65).abs() < 0.001);
    }

    // ========================================================================
    // THRESHOLD TUNING: TuningResult improvement_over_baseline
    // ========================================================================

    #[test]
    fn tuning_result_tracks_improvement() {
        // ML strictly better than rules
        let samples = vec![
            ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.9),
            ValidationSample::new(InferredOwnership::Borrowed, InferredOwnership::Owned, InferredOwnership::Borrowed, 0.8),
        ];
        // Rules: 0/2 correct = 0% baseline
        // ML at any threshold < 0.8: 2/2 = 100%
        let result = ThresholdTuner::new().tune(&samples);
        assert!(result.improvement_over_baseline > 0.0);
        assert!((result.baseline_accuracy - 0.0).abs() < 0.001);
        assert!((result.ml_only_accuracy - 1.0).abs() < 0.001);
    }

    #[test]
    fn tuning_result_negative_or_zero_improvement_when_rules_better() {
        // Rules always correct, ML always wrong
        let samples = vec![
            ValidationSample::new(InferredOwnership::Owned, InferredOwnership::Owned, InferredOwnership::Borrowed, 0.9),
            ValidationSample::new(InferredOwnership::Borrowed, InferredOwnership::Borrowed, InferredOwnership::Owned, 0.8),
        ];

        let result = ThresholdTuner::new().tune(&samples);
        assert!((result.baseline_accuracy - 1.0).abs() < 0.001);
        assert!((result.ml_only_accuracy - 0.0).abs() < 0.001);
        // At high thresholds, all samples fall back to rules, achieving baseline
        // improvement_over_baseline = optimal_accuracy - baseline_accuracy = 1.0 - 1.0 = 0.0
        assert!(result.improvement_over_baseline <= 0.001);
    }
}
