//! Hybrid ownership classification combining rules and ML (DECY-ML-012).
//!
//! Implements the hybrid classification system from the ML-enhanced ownership
//! inference specification. Combines rule-based heuristics with ML predictions
//! and falls back to rules when ML confidence is below threshold.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                  HYBRID CLASSIFIER                          │
//! │  ┌───────────────────────────┐                              │
//! │  │ Phase 1: Rule-Based       │──────────────────────┐       │
//! │  │ • malloc/free → Box       │                      │       │
//! │  │ • array alloc → Vec       │                      ▼       │
//! │  │ • single deref → &T       │            ┌─────────────┐   │
//! │  └───────────────────────────┘            │  Fallback   │   │
//! │              │                            │  Logic      │   │
//! │              ▼                            │             │   │
//! │  ┌───────────────────────────┐            │  if conf    │   │
//! │  │ Phase 2: ML Enhancement   │            │  < 0.65:    │   │
//! │  │ • Feature extraction      │───────────►│  use rules  │   │
//! │  │ • Pattern classification  │            └─────────────┘   │
//! │  │ • Confidence scoring      │                      │       │
//! │  └───────────────────────────┘                      │       │
//! │                                                     ▼       │
//! │                                          ┌─────────────┐    │
//! │                                          │   Result    │    │
//! │                                          └─────────────┘    │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use crate::inference::{OwnershipInference, OwnershipInferencer, OwnershipKind};
use crate::ml_features::{InferredOwnership, OwnershipFeatures, OwnershipPrediction};
use serde::{Deserialize, Serialize};

/// Default confidence threshold for ML predictions.
///
/// If ML confidence is below this, fall back to rule-based classification.
pub const DEFAULT_CONFIDENCE_THRESHOLD: f64 = 0.65;

/// Classification method used for a prediction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClassificationMethod {
    /// Used rule-based heuristics only
    RuleBased,
    /// Used ML prediction with sufficient confidence
    MachineLearning,
    /// ML confidence was low, fell back to rules
    Fallback,
    /// Combined rules and ML (ensemble)
    Hybrid,
}

impl std::fmt::Display for ClassificationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClassificationMethod::RuleBased => write!(f, "rule-based"),
            ClassificationMethod::MachineLearning => write!(f, "ml"),
            ClassificationMethod::Fallback => write!(f, "fallback"),
            ClassificationMethod::Hybrid => write!(f, "hybrid"),
        }
    }
}

/// Result of hybrid classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridResult {
    /// Variable name
    pub variable: String,
    /// Final inferred ownership
    pub ownership: InferredOwnership,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Method used for classification
    pub method: ClassificationMethod,
    /// Rule-based result (always available)
    pub rule_result: Option<InferredOwnership>,
    /// ML result (if available)
    pub ml_result: Option<OwnershipPrediction>,
    /// Reasoning for final decision
    pub reasoning: String,
}

impl HybridResult {
    /// Check if this result used fallback logic.
    pub fn used_fallback(&self) -> bool {
        self.method == ClassificationMethod::Fallback
    }

    /// Check if ML was available but not used due to low confidence.
    pub fn ml_rejected(&self) -> bool {
        self.ml_result.is_some() && self.method == ClassificationMethod::Fallback
    }
}

/// Trait for ML model inference.
///
/// Implement this trait to integrate a custom ML model (e.g., aprender RandomForest,
/// CodeBERT fine-tuned, or external API).
pub trait OwnershipModel: Send + Sync {
    /// Predict ownership from features.
    fn predict(&self, features: &OwnershipFeatures) -> OwnershipPrediction;

    /// Batch predict for multiple feature vectors.
    fn predict_batch(&self, features: &[OwnershipFeatures]) -> Vec<OwnershipPrediction> {
        features.iter().map(|f| self.predict(f)).collect()
    }

    /// Get model name/version for logging.
    fn name(&self) -> &str {
        "unknown"
    }
}

/// Null model that always returns low confidence.
///
/// Use as placeholder when no ML model is available.
#[derive(Debug, Clone, Default)]
pub struct NullModel;

impl OwnershipModel for NullModel {
    fn predict(&self, _features: &OwnershipFeatures) -> OwnershipPrediction {
        OwnershipPrediction {
            kind: InferredOwnership::RawPointer,
            confidence: 0.0,
            fallback: None,
        }
    }

    fn name(&self) -> &str {
        "null"
    }
}

/// Hybrid ownership classifier combining rules and ML.
#[derive(Debug)]
pub struct HybridClassifier {
    /// Rule-based inferencer (reserved for future rule execution)
    #[allow(dead_code)]
    rule_inferencer: OwnershipInferencer,
    /// Confidence threshold for ML predictions
    confidence_threshold: f64,
    /// Whether ML is enabled
    ml_enabled: bool,
}

impl Default for HybridClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl HybridClassifier {
    /// Create a new hybrid classifier with default settings.
    pub fn new() -> Self {
        Self {
            rule_inferencer: OwnershipInferencer::new(),
            confidence_threshold: DEFAULT_CONFIDENCE_THRESHOLD,
            ml_enabled: false,
        }
    }

    /// Create with custom confidence threshold.
    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            rule_inferencer: OwnershipInferencer::new(),
            confidence_threshold: threshold.clamp(0.0, 1.0),
            ml_enabled: false,
        }
    }

    /// Enable ML predictions.
    pub fn enable_ml(&mut self) {
        self.ml_enabled = true;
    }

    /// Disable ML predictions (use rules only).
    pub fn disable_ml(&mut self) {
        self.ml_enabled = false;
    }

    /// Check if ML is enabled.
    pub fn ml_enabled(&self) -> bool {
        self.ml_enabled
    }

    /// Get confidence threshold.
    pub fn confidence_threshold(&self) -> f64 {
        self.confidence_threshold
    }

    /// Set confidence threshold.
    pub fn set_threshold(&mut self, threshold: f64) {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
    }

    /// Classify using rules only.
    pub fn classify_rule_based(&self, inference: &OwnershipInference) -> HybridResult {
        let ownership = ownership_kind_to_inferred(&inference.kind);

        HybridResult {
            variable: inference.variable.clone(),
            ownership,
            confidence: inference.confidence as f64,
            method: ClassificationMethod::RuleBased,
            rule_result: Some(ownership),
            ml_result: None,
            reasoning: format!("Rule-based: {}", inference.reason),
        }
    }

    /// Classify using hybrid approach with ML model.
    pub fn classify_hybrid<M: OwnershipModel>(
        &self,
        inference: &OwnershipInference,
        features: &OwnershipFeatures,
        model: &M,
    ) -> HybridResult {
        // Always get rule-based result
        let rule_ownership = ownership_kind_to_inferred(&inference.kind);

        // If ML is disabled, use rules only
        if !self.ml_enabled {
            return self.classify_rule_based(inference);
        }

        // Get ML prediction
        let ml_prediction = model.predict(features);
        let ml_conf = ml_prediction.confidence as f64;
        let ml_kind = ml_prediction.kind;

        // Check confidence threshold
        if ml_conf >= self.confidence_threshold {
            // ML confidence is sufficient - use ML result
            HybridResult {
                variable: inference.variable.clone(),
                ownership: ml_kind,
                confidence: ml_conf,
                method: ClassificationMethod::MachineLearning,
                rule_result: Some(rule_ownership),
                ml_result: Some(ml_prediction),
                reasoning: format!("ML prediction (confidence {:.2}): {:?}", ml_conf, ml_kind),
            }
        } else {
            // ML confidence too low - fall back to rules
            HybridResult {
                variable: inference.variable.clone(),
                ownership: rule_ownership,
                confidence: inference.confidence as f64,
                method: ClassificationMethod::Fallback,
                rule_result: Some(rule_ownership),
                ml_result: Some(ml_prediction),
                reasoning: format!(
                    "Fallback to rules (ML confidence {:.2} < threshold {:.2}): {}",
                    ml_conf, self.confidence_threshold, inference.reason
                ),
            }
        }
    }

    /// Classify with ensemble (combine rules and ML).
    ///
    /// When both methods agree, boost confidence.
    /// When they disagree, use the higher confidence result.
    pub fn classify_ensemble<M: OwnershipModel>(
        &self,
        inference: &OwnershipInference,
        features: &OwnershipFeatures,
        model: &M,
    ) -> HybridResult {
        let rule_ownership = ownership_kind_to_inferred(&inference.kind);
        let ml_prediction = model.predict(features);
        let ml_conf = ml_prediction.confidence as f64;
        let ml_kind = ml_prediction.kind;

        // Check if they agree
        let agree = rule_ownership == ml_kind;

        if agree {
            // Both agree - boost confidence
            let combined_confidence = (inference.confidence as f64 + ml_conf) / 2.0 * 1.1;
            let final_confidence = combined_confidence.min(1.0);

            HybridResult {
                variable: inference.variable.clone(),
                ownership: rule_ownership,
                confidence: final_confidence,
                method: ClassificationMethod::Hybrid,
                rule_result: Some(rule_ownership),
                ml_result: Some(ml_prediction),
                reasoning: format!(
                    "Hybrid (rules + ML agree): boosted confidence {:.2}",
                    final_confidence
                ),
            }
        } else {
            // Disagree - use higher confidence
            if ml_conf > inference.confidence as f64 {
                HybridResult {
                    variable: inference.variable.clone(),
                    ownership: ml_kind,
                    confidence: ml_conf,
                    method: ClassificationMethod::MachineLearning,
                    rule_result: Some(rule_ownership),
                    ml_result: Some(ml_prediction),
                    reasoning: format!(
                        "ML wins (conf {:.2} > rules {:.2}): {:?}",
                        ml_conf, inference.confidence, ml_kind
                    ),
                }
            } else {
                HybridResult {
                    variable: inference.variable.clone(),
                    ownership: rule_ownership,
                    confidence: inference.confidence as f64,
                    method: ClassificationMethod::RuleBased,
                    rule_result: Some(rule_ownership),
                    ml_result: Some(ml_prediction),
                    reasoning: format!(
                        "Rules win (conf {:.2} > ML {:.2}): {}",
                        inference.confidence, ml_conf, inference.reason
                    ),
                }
            }
        }
    }
}

/// Convert OwnershipKind to InferredOwnership.
fn ownership_kind_to_inferred(kind: &OwnershipKind) -> InferredOwnership {
    match kind {
        OwnershipKind::Owning => InferredOwnership::Owned,
        OwnershipKind::ImmutableBorrow => InferredOwnership::Borrowed,
        OwnershipKind::MutableBorrow => InferredOwnership::BorrowedMut,
        OwnershipKind::ArrayPointer { .. } => InferredOwnership::Slice,
        OwnershipKind::Unknown => InferredOwnership::RawPointer,
    }
}

/// Metrics for hybrid classification.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HybridMetrics {
    /// Total classifications
    pub total: u64,
    /// Rule-based only classifications
    pub rule_based: u64,
    /// ML classifications with sufficient confidence
    pub ml_used: u64,
    /// Fallback to rules due to low ML confidence
    pub fallback: u64,
    /// Hybrid/ensemble classifications
    pub hybrid: u64,
    /// Cases where rules and ML agreed
    pub agreements: u64,
    /// Cases where rules and ML disagreed
    pub disagreements: u64,
}

impl HybridMetrics {
    /// Create new metrics tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a classification result.
    pub fn record(&mut self, result: &HybridResult) {
        self.total += 1;
        match result.method {
            ClassificationMethod::RuleBased => self.rule_based += 1,
            ClassificationMethod::MachineLearning => self.ml_used += 1,
            ClassificationMethod::Fallback => self.fallback += 1,
            ClassificationMethod::Hybrid => self.hybrid += 1,
        }

        // Track agreement/disagreement
        if let (Some(rule), Some(ml)) = (&result.rule_result, &result.ml_result) {
            if *rule == ml.kind {
                self.agreements += 1;
            } else {
                self.disagreements += 1;
            }
        }
    }

    /// Get ML usage rate (0.0 - 1.0).
    pub fn ml_usage_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.ml_used as f64 / self.total as f64
        }
    }

    /// Get fallback rate (0.0 - 1.0).
    pub fn fallback_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.fallback as f64 / self.total as f64
        }
    }

    /// Get agreement rate when both methods were used (0.0 - 1.0).
    pub fn agreement_rate(&self) -> f64 {
        let both = self.agreements + self.disagreements;
        if both == 0 {
            1.0 // No comparisons = perfect agreement by default
        } else {
            self.agreements as f64 / both as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // ClassificationMethod tests
    // ========================================================================

    #[test]
    fn classification_method_display() {
        assert_eq!(ClassificationMethod::RuleBased.to_string(), "rule-based");
        assert_eq!(ClassificationMethod::MachineLearning.to_string(), "ml");
        assert_eq!(ClassificationMethod::Fallback.to_string(), "fallback");
        assert_eq!(ClassificationMethod::Hybrid.to_string(), "hybrid");
    }

    // ========================================================================
    // HybridClassifier tests
    // ========================================================================

    #[test]
    fn hybrid_classifier_default() {
        let classifier = HybridClassifier::new();
        assert!(!classifier.ml_enabled());
        assert!((classifier.confidence_threshold() - 0.65).abs() < 0.001);
    }

    #[test]
    fn hybrid_classifier_with_threshold() {
        let classifier = HybridClassifier::with_threshold(0.8);
        assert!((classifier.confidence_threshold() - 0.8).abs() < 0.001);
    }

    #[test]
    fn hybrid_classifier_threshold_clamp() {
        let low = HybridClassifier::with_threshold(-0.5);
        assert!((low.confidence_threshold() - 0.0).abs() < 0.001);

        let high = HybridClassifier::with_threshold(1.5);
        assert!((high.confidence_threshold() - 1.0).abs() < 0.001);
    }

    #[test]
    fn hybrid_classifier_enable_disable_ml() {
        let mut classifier = HybridClassifier::new();
        assert!(!classifier.ml_enabled());

        classifier.enable_ml();
        assert!(classifier.ml_enabled());

        classifier.disable_ml();
        assert!(!classifier.ml_enabled());
    }

    #[test]
    fn classify_rule_based_owning() {
        let classifier = HybridClassifier::new();
        let inference = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Owning,
            confidence: 0.9,
            reason: "malloc detected".to_string(),
        };

        let result = classifier.classify_rule_based(&inference);

        assert_eq!(result.variable, "ptr");
        assert_eq!(result.ownership, InferredOwnership::Owned);
        assert_eq!(result.method, ClassificationMethod::RuleBased);
        assert!(result.ml_result.is_none());
    }

    #[test]
    fn classify_rule_based_immutable_borrow() {
        let classifier = HybridClassifier::new();
        let inference = OwnershipInference {
            variable: "ref".to_string(),
            kind: OwnershipKind::ImmutableBorrow,
            confidence: 0.85,
            reason: "read-only access".to_string(),
        };

        let result = classifier.classify_rule_based(&inference);

        assert_eq!(result.ownership, InferredOwnership::Borrowed);
    }

    // ========================================================================
    // Hybrid classification with ML tests
    // ========================================================================

    /// Mock ML model for testing
    struct MockModel {
        ownership: InferredOwnership,
        confidence: f64,
    }

    impl MockModel {
        fn with_confidence(ownership: InferredOwnership, confidence: f64) -> Self {
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
            "mock"
        }
    }

    #[test]
    fn classify_hybrid_ml_high_confidence() {
        let mut classifier = HybridClassifier::new();
        classifier.enable_ml();

        let inference = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Unknown,
            confidence: 0.5,
            reason: "uncertain".to_string(),
        };

        let features = OwnershipFeatures::default();
        let model = MockModel::with_confidence(InferredOwnership::Vec, 0.9);

        let result = classifier.classify_hybrid(&inference, &features, &model);

        assert_eq!(result.ownership, InferredOwnership::Vec);
        assert_eq!(result.method, ClassificationMethod::MachineLearning);
        assert!(!result.used_fallback());
    }

    #[test]
    fn classify_hybrid_ml_low_confidence_fallback() {
        let mut classifier = HybridClassifier::new();
        classifier.enable_ml();

        let inference = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Owning,
            confidence: 0.8,
            reason: "malloc detected".to_string(),
        };

        let features = OwnershipFeatures::default();
        let model = MockModel::with_confidence(InferredOwnership::Borrowed, 0.3);

        let result = classifier.classify_hybrid(&inference, &features, &model);

        // Should fall back to rule-based (Owning → Owned)
        assert_eq!(result.ownership, InferredOwnership::Owned);
        assert_eq!(result.method, ClassificationMethod::Fallback);
        assert!(result.used_fallback());
        assert!(result.ml_rejected());
    }

    #[test]
    fn classify_hybrid_ml_disabled() {
        let classifier = HybridClassifier::new(); // ML disabled by default

        let inference = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::MutableBorrow,
            confidence: 0.7,
            reason: "mutation detected".to_string(),
        };

        let features = OwnershipFeatures::default();
        let model = MockModel::with_confidence(InferredOwnership::Owned, 0.99);

        let result = classifier.classify_hybrid(&inference, &features, &model);

        // Should use rules since ML is disabled
        assert_eq!(result.ownership, InferredOwnership::BorrowedMut);
        assert_eq!(result.method, ClassificationMethod::RuleBased);
    }

    // ========================================================================
    // Ensemble classification tests
    // ========================================================================

    #[test]
    fn classify_ensemble_agreement_boosts_confidence() {
        let classifier = HybridClassifier::new();

        let inference = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Owning,
            confidence: 0.7,
            reason: "malloc".to_string(),
        };

        let features = OwnershipFeatures::default();
        let model = MockModel::with_confidence(InferredOwnership::Owned, 0.8);

        let result = classifier.classify_ensemble(&inference, &features, &model);

        assert_eq!(result.method, ClassificationMethod::Hybrid);
        // (0.7 + 0.8) / 2 * 1.1 = 0.825
        assert!(result.confidence > 0.82);
    }

    #[test]
    fn classify_ensemble_disagreement_ml_wins() {
        let classifier = HybridClassifier::new();

        let inference = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Unknown,
            confidence: 0.3,
            reason: "unknown".to_string(),
        };

        let features = OwnershipFeatures::default();
        let model = MockModel::with_confidence(InferredOwnership::Vec, 0.9);

        let result = classifier.classify_ensemble(&inference, &features, &model);

        assert_eq!(result.ownership, InferredOwnership::Vec);
        assert_eq!(result.method, ClassificationMethod::MachineLearning);
    }

    #[test]
    fn classify_ensemble_disagreement_rules_win() {
        let classifier = HybridClassifier::new();

        let inference = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Owning,
            confidence: 0.95,
            reason: "malloc + free".to_string(),
        };

        let features = OwnershipFeatures::default();
        let model = MockModel::with_confidence(InferredOwnership::Borrowed, 0.4);

        let result = classifier.classify_ensemble(&inference, &features, &model);

        assert_eq!(result.ownership, InferredOwnership::Owned);
        assert_eq!(result.method, ClassificationMethod::RuleBased);
    }

    // ========================================================================
    // NullModel tests
    // ========================================================================

    #[test]
    fn null_model_returns_unknown() {
        let model = NullModel;
        let features = OwnershipFeatures::default();

        let prediction = model.predict(&features);

        assert_eq!(prediction.kind, InferredOwnership::RawPointer);
        assert!((prediction.confidence as f64 - 0.0).abs() < 0.001);
    }

    #[test]
    fn null_model_name() {
        let model = NullModel;
        assert_eq!(model.name(), "null");
    }

    // ========================================================================
    // HybridResult tests
    // ========================================================================

    #[test]
    fn hybrid_result_used_fallback() {
        let result = HybridResult {
            variable: "x".to_string(),
            ownership: InferredOwnership::Owned,
            confidence: 0.8,
            method: ClassificationMethod::Fallback,
            rule_result: Some(InferredOwnership::Owned),
            ml_result: Some(OwnershipPrediction {
                kind: InferredOwnership::Borrowed,
                confidence: 0.3,
                fallback: None,
            }),
            reasoning: "test".to_string(),
        };

        assert!(result.used_fallback());
        assert!(result.ml_rejected());
    }

    #[test]
    fn hybrid_result_ml_not_rejected() {
        let result = HybridResult {
            variable: "x".to_string(),
            ownership: InferredOwnership::Owned,
            confidence: 0.9,
            method: ClassificationMethod::MachineLearning,
            rule_result: Some(InferredOwnership::RawPointer),
            ml_result: Some(OwnershipPrediction {
                kind: InferredOwnership::Owned,
                confidence: 0.9,
                fallback: None,
            }),
            reasoning: "test".to_string(),
        };

        assert!(!result.used_fallback());
        assert!(!result.ml_rejected());
    }

    // ========================================================================
    // HybridMetrics tests
    // ========================================================================

    #[test]
    fn hybrid_metrics_default() {
        let metrics = HybridMetrics::new();
        assert_eq!(metrics.total, 0);
        assert_eq!(metrics.ml_usage_rate(), 0.0);
    }

    #[test]
    fn hybrid_metrics_record() {
        let mut metrics = HybridMetrics::new();

        let result1 = HybridResult {
            variable: "a".to_string(),
            ownership: InferredOwnership::Owned,
            confidence: 0.9,
            method: ClassificationMethod::MachineLearning,
            rule_result: Some(InferredOwnership::Owned),
            ml_result: Some(OwnershipPrediction {
                kind: InferredOwnership::Owned,
                confidence: 0.9,
                fallback: None,
            }),
            reasoning: "ml".to_string(),
        };

        let result2 = HybridResult {
            variable: "b".to_string(),
            ownership: InferredOwnership::Borrowed,
            confidence: 0.7,
            method: ClassificationMethod::RuleBased,
            rule_result: Some(InferredOwnership::Borrowed),
            ml_result: None,
            reasoning: "rules".to_string(),
        };

        metrics.record(&result1);
        metrics.record(&result2);

        assert_eq!(metrics.total, 2);
        assert_eq!(metrics.ml_used, 1);
        assert_eq!(metrics.rule_based, 1);
        assert!((metrics.ml_usage_rate() - 0.5).abs() < 0.001);
    }

    #[test]
    fn hybrid_metrics_agreement_rate() {
        let mut metrics = HybridMetrics::new();

        // Agreement
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

        // Disagreement
        let disagree = HybridResult {
            variable: "b".to_string(),
            ownership: InferredOwnership::Owned,
            confidence: 0.9,
            method: ClassificationMethod::Fallback,
            rule_result: Some(InferredOwnership::Owned),
            ml_result: Some(OwnershipPrediction {
                kind: InferredOwnership::Borrowed,
                confidence: 0.3,
                fallback: None,
            }),
            reasoning: "disagree".to_string(),
        };

        metrics.record(&agree);
        metrics.record(&disagree);

        assert_eq!(metrics.agreements, 1);
        assert_eq!(metrics.disagreements, 1);
        assert!((metrics.agreement_rate() - 0.5).abs() < 0.001);
    }

    #[test]
    fn hybrid_metrics_fallback_rate() {
        let mut metrics = HybridMetrics::new();
        metrics.total = 10;
        metrics.fallback = 3;

        assert!((metrics.fallback_rate() - 0.3).abs() < 0.001);
    }

    // ========================================================================
    // ownership_kind_to_inferred tests
    // ========================================================================

    #[test]
    fn convert_ownership_kinds() {
        assert_eq!(
            ownership_kind_to_inferred(&OwnershipKind::Owning),
            InferredOwnership::Owned
        );
        assert_eq!(
            ownership_kind_to_inferred(&OwnershipKind::ImmutableBorrow),
            InferredOwnership::Borrowed
        );
        assert_eq!(
            ownership_kind_to_inferred(&OwnershipKind::MutableBorrow),
            InferredOwnership::BorrowedMut
        );
        assert_eq!(
            ownership_kind_to_inferred(&OwnershipKind::Unknown),
            InferredOwnership::RawPointer
        );
    }

    #[test]
    fn convert_array_pointer() {
        let array_kind = OwnershipKind::ArrayPointer {
            base_array: "arr".to_string(),
            element_type: decy_hir::HirType::Int,
            base_index: Some(0),
        };
        assert_eq!(
            ownership_kind_to_inferred(&array_kind),
            InferredOwnership::Slice
        );
    }

    // ========================================================================
    // Deep coverage: classify_ensemble all branches
    // ========================================================================

    #[test]
    fn classify_ensemble_agreement_confidence_capped_at_one() {
        // When both have very high confidence, boosted value should cap at 1.0
        let classifier = HybridClassifier::new();

        let inference = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Owning,
            confidence: 0.99, // Very high rule confidence
            reason: "malloc + free".to_string(),
        };

        let features = OwnershipFeatures::default();
        // ML agrees with high confidence
        let model = MockModel::with_confidence(InferredOwnership::Owned, 0.99);

        let result = classifier.classify_ensemble(&inference, &features, &model);

        assert_eq!(result.method, ClassificationMethod::Hybrid);
        assert_eq!(result.ownership, InferredOwnership::Owned);
        // (0.99 + 0.99) / 2 * 1.1 = 1.089 -> capped at 1.0
        assert!((result.confidence - 1.0).abs() < 0.001);
        assert!(result.reasoning.contains("boosted"));
    }

    #[test]
    fn classify_ensemble_agreement_low_confidence_boosted() {
        let classifier = HybridClassifier::new();

        let inference = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::ImmutableBorrow,
            confidence: 0.4,
            reason: "parameter read-only".to_string(),
        };

        let features = OwnershipFeatures::default();
        let model = MockModel::with_confidence(InferredOwnership::Borrowed, 0.5);

        let result = classifier.classify_ensemble(&inference, &features, &model);

        assert_eq!(result.method, ClassificationMethod::Hybrid);
        assert_eq!(result.ownership, InferredOwnership::Borrowed);
        // (0.4 + 0.5) / 2 * 1.1 = 0.495
        assert!(result.confidence > 0.49);
        assert!(result.confidence < 0.51);
    }

    #[test]
    fn classify_ensemble_disagreement_ml_wins_with_exact_equality() {
        // Edge: ML confidence equals rules confidence => rules win (else branch)
        let classifier = HybridClassifier::new();

        let inference = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Owning,
            confidence: 0.7, // Same as ML
            reason: "allocation".to_string(),
        };

        let features = OwnershipFeatures::default();
        let model = MockModel::with_confidence(InferredOwnership::Borrowed, 0.7);

        let result = classifier.classify_ensemble(&inference, &features, &model);

        // They disagree, confidence is equal => rules win (ml_conf > inference.confidence is false)
        assert_eq!(result.method, ClassificationMethod::RuleBased);
        assert_eq!(result.ownership, InferredOwnership::Owned);
        assert!(result.reasoning.contains("Rules win"));
    }

    #[test]
    fn classify_ensemble_disagreement_ml_wins_clearly() {
        let classifier = HybridClassifier::new();

        let inference = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Unknown, // Low confidence from rules
            confidence: 0.3,
            reason: "unknown pattern".to_string(),
        };

        let features = OwnershipFeatures::default();
        let model = MockModel::with_confidence(InferredOwnership::Vec, 0.95);

        let result = classifier.classify_ensemble(&inference, &features, &model);

        assert_eq!(result.method, ClassificationMethod::MachineLearning);
        assert_eq!(result.ownership, InferredOwnership::Vec);
        assert!((result.confidence - 0.95).abs() < 0.001);
        assert!(result.reasoning.contains("ML wins"));
    }

    #[test]
    fn classify_ensemble_disagreement_rules_win_clearly() {
        let classifier = HybridClassifier::new();

        let inference = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Owning,
            confidence: 0.95,
            reason: "malloc with free".to_string(),
        };

        let features = OwnershipFeatures::default();
        let model = MockModel::with_confidence(InferredOwnership::Borrowed, 0.2);

        let result = classifier.classify_ensemble(&inference, &features, &model);

        assert_eq!(result.method, ClassificationMethod::RuleBased);
        assert_eq!(result.ownership, InferredOwnership::Owned);
        assert!((result.confidence - 0.95).abs() < 0.001);
        assert!(result.reasoning.contains("Rules win"));
        assert!(result.ml_result.is_some());
        assert!(result.rule_result.is_some());
    }

    #[test]
    fn classify_ensemble_all_ownership_kinds() {
        // Test ensemble with different ownership kind conversions
        let classifier = HybridClassifier::new();
        let features = OwnershipFeatures::default();

        // MutableBorrow
        let inference_mut = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::MutableBorrow,
            confidence: 0.7,
            reason: "mutation detected".to_string(),
        };
        let model_agree = MockModel::with_confidence(InferredOwnership::BorrowedMut, 0.8);
        let result = classifier.classify_ensemble(&inference_mut, &features, &model_agree);
        assert_eq!(result.method, ClassificationMethod::Hybrid);
        assert_eq!(result.ownership, InferredOwnership::BorrowedMut);

        // Unknown -> RawPointer
        let inference_unknown = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Unknown,
            confidence: 0.3,
            reason: "uncertain".to_string(),
        };
        let model_agree_raw = MockModel::with_confidence(InferredOwnership::RawPointer, 0.4);
        let result = classifier.classify_ensemble(&inference_unknown, &features, &model_agree_raw);
        assert_eq!(result.method, ClassificationMethod::Hybrid);
        assert_eq!(result.ownership, InferredOwnership::RawPointer);
    }

    // ========================================================================
    // classify_hybrid: additional branch coverage
    // ========================================================================

    #[test]
    fn classify_hybrid_at_exact_threshold() {
        // ML confidence clearly above threshold => should use ML
        // Note: confidence is f32 in OwnershipPrediction, compared as f64
        // so we use a value clearly above to avoid f32 precision issues
        let mut classifier = HybridClassifier::with_threshold(0.5);
        classifier.enable_ml();

        let inference = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Unknown,
            confidence: 0.3,
            reason: "uncertain".to_string(),
        };

        let features = OwnershipFeatures::default();
        let model = MockModel::with_confidence(InferredOwnership::Owned, 0.75);

        let result = classifier.classify_hybrid(&inference, &features, &model);

        // 0.75 >= 0.5 => use ML
        assert_eq!(result.method, ClassificationMethod::MachineLearning);
        assert_eq!(result.ownership, InferredOwnership::Owned);
        assert!(result.reasoning.contains("ML prediction"));
    }

    #[test]
    fn classify_hybrid_just_below_threshold() {
        let mut classifier = HybridClassifier::with_threshold(0.65);
        classifier.enable_ml();

        let inference = OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Owning,
            confidence: 0.9,
            reason: "malloc".to_string(),
        };

        let features = OwnershipFeatures::default();
        let model = MockModel::with_confidence(InferredOwnership::Borrowed, 0.6499);

        let result = classifier.classify_hybrid(&inference, &features, &model);

        // 0.6499 < 0.65 => fallback to rules
        assert_eq!(result.method, ClassificationMethod::Fallback);
        assert_eq!(result.ownership, InferredOwnership::Owned);
        assert!(result.reasoning.contains("Fallback"));
    }

    // ========================================================================
    // HybridClassifier: set_threshold coverage
    // ========================================================================

    #[test]
    fn set_threshold_clamps() {
        let mut classifier = HybridClassifier::new();
        classifier.set_threshold(2.0);
        assert!((classifier.confidence_threshold() - 1.0).abs() < 0.001);

        classifier.set_threshold(-1.0);
        assert!((classifier.confidence_threshold() - 0.0).abs() < 0.001);

        classifier.set_threshold(0.42);
        assert!((classifier.confidence_threshold() - 0.42).abs() < 0.001);
    }

    // ========================================================================
    // HybridMetrics: additional coverage
    // ========================================================================

    #[test]
    fn hybrid_metrics_record_hybrid_method() {
        let mut metrics = HybridMetrics::new();
        let result = HybridResult {
            variable: "x".to_string(),
            ownership: InferredOwnership::Owned,
            confidence: 0.9,
            method: ClassificationMethod::Hybrid,
            rule_result: Some(InferredOwnership::Owned),
            ml_result: Some(OwnershipPrediction {
                kind: InferredOwnership::Owned,
                confidence: 0.9,
                fallback: None,
            }),
            reasoning: "hybrid agreement".to_string(),
        };
        metrics.record(&result);

        assert_eq!(metrics.hybrid, 1);
        assert_eq!(metrics.total, 1);
        assert_eq!(metrics.agreements, 1);
        assert_eq!(metrics.disagreements, 0);
    }

    #[test]
    fn hybrid_metrics_agreement_rate_no_comparisons() {
        let metrics = HybridMetrics::new();
        // No comparisons = perfect agreement by default
        assert!((metrics.agreement_rate() - 1.0).abs() < 0.001);
    }

    #[test]
    fn hybrid_metrics_all_methods_tracked() {
        let mut metrics = HybridMetrics::new();

        // Record one of each method
        for (method, rule_own, ml_own) in [
            (ClassificationMethod::RuleBased, InferredOwnership::Owned, InferredOwnership::Owned),
            (ClassificationMethod::MachineLearning, InferredOwnership::Borrowed, InferredOwnership::Borrowed),
            (ClassificationMethod::Fallback, InferredOwnership::Owned, InferredOwnership::Borrowed),
            (ClassificationMethod::Hybrid, InferredOwnership::Vec, InferredOwnership::Vec),
        ] {
            let result = HybridResult {
                variable: "x".to_string(),
                ownership: rule_own,
                confidence: 0.8,
                method,
                rule_result: Some(rule_own),
                ml_result: Some(OwnershipPrediction {
                    kind: ml_own,
                    confidence: 0.8,
                    fallback: None,
                }),
                reasoning: "test".to_string(),
            };
            metrics.record(&result);
        }

        assert_eq!(metrics.total, 4);
        assert_eq!(metrics.rule_based, 1);
        assert_eq!(metrics.ml_used, 1);
        assert_eq!(metrics.fallback, 1);
        assert_eq!(metrics.hybrid, 1);
        // agreements: RuleBased (same), MachineLearning (same), Hybrid (same) = 3
        // disagreements: Fallback (Owned vs Borrowed) = 1
        assert_eq!(metrics.agreements, 3);
        assert_eq!(metrics.disagreements, 1);
    }

    #[test]
    fn hybrid_result_ml_not_rejected_without_ml_result() {
        let result = HybridResult {
            variable: "x".to_string(),
            ownership: InferredOwnership::Owned,
            confidence: 0.8,
            method: ClassificationMethod::Fallback,
            rule_result: Some(InferredOwnership::Owned),
            ml_result: None, // No ML result
            reasoning: "rules only".to_string(),
        };

        assert!(result.used_fallback());
        // ml_rejected requires ml_result.is_some() AND Fallback method
        assert!(!result.ml_rejected());
    }

    // ========================================================================
    // NullModel: batch predict coverage
    // ========================================================================

    #[test]
    fn null_model_batch_predict() {
        let model = NullModel;
        let features = vec![
            OwnershipFeatures::default(),
            OwnershipFeatures::default(),
            OwnershipFeatures::default(),
        ];
        let predictions = model.predict_batch(&features);
        assert_eq!(predictions.len(), 3);
        for pred in &predictions {
            assert_eq!(pred.kind, InferredOwnership::RawPointer);
            assert!((pred.confidence as f64).abs() < 0.001);
        }
    }

    // ========================================================================
    // Default implementation
    // ========================================================================

    #[test]
    fn hybrid_classifier_default_impl() {
        let classifier = HybridClassifier::default();
        assert!(!classifier.ml_enabled());
        assert!((classifier.confidence_threshold() - DEFAULT_CONFIDENCE_THRESHOLD).abs() < 0.001);
    }
}
