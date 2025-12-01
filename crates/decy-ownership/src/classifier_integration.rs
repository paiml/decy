//! Classifier integration layer for transpilation pipeline.
//!
//! **Ticket**: DECY-182 - Wire RuleBasedClassifier into pipeline
//!
//! This module bridges the gap between the `OwnershipClassifier` trait system
//! and the existing transpilation pipeline that uses `OwnershipInference`.

use crate::classifier::{ClassifierPrediction, OwnershipClassifier, RuleBasedClassifier};
use crate::dataflow::DataflowGraph;
use crate::inference::{OwnershipInference, OwnershipKind};
use crate::ml_features::{AllocationKind, InferredOwnership, OwnershipFeatures, OwnershipFeaturesBuilder};
use decy_hir::{HirFunction, HirType};
use std::collections::HashMap;

/// Classify function variables using a rule-based classifier.
///
/// This is a convenience function that uses the default `RuleBasedClassifier`.
///
/// # Arguments
///
/// * `graph` - Dataflow graph from analysis
/// * `func` - HIR function being analyzed
///
/// # Returns
///
/// HashMap mapping variable names to ownership inferences.
pub fn classify_with_rules(
    graph: &DataflowGraph,
    func: &HirFunction,
) -> HashMap<String, OwnershipInference> {
    let classifier = RuleBasedClassifier::new();
    classify_function_variables(&classifier, graph, func)
}

/// Classify all pointer variables in a function using a classifier.
///
/// Extracts features from the dataflow graph and applies the classifier
/// to determine ownership for each pointer variable.
///
/// # Arguments
///
/// * `classifier` - Any implementation of `OwnershipClassifier`
/// * `graph` - Dataflow graph from analysis
/// * `func` - HIR function being analyzed
///
/// # Returns
///
/// HashMap mapping variable names to ownership inferences.
pub fn classify_function_variables(
    classifier: &dyn OwnershipClassifier,
    graph: &DataflowGraph,
    func: &HirFunction,
) -> HashMap<String, OwnershipInference> {
    let mut result = HashMap::new();

    // Extract features for each pointer parameter
    for param in func.parameters() {
        if is_pointer_type(param.param_type()) {
            let var_name = param.name();
            let features = extract_features_for_variable(var_name, graph, func);
            let prediction = classifier.classify(&features);

            let inference = prediction_to_inference(var_name, &prediction, classifier.name());
            result.insert(var_name.to_string(), inference);
        }
    }

    // Also check local variables in body that are pointers
    for var_name in graph.variables() {
        if !result.contains_key(var_name) {
            // DECY-183: Check if this variable is derived from an array
            // If so, use ArrayPointer kind for safe slice indexing
            if let Some(array_base) = graph.array_base_for(var_name) {
                let inference = OwnershipInference {
                    variable: var_name.clone(),
                    kind: OwnershipKind::ArrayPointer {
                        base_array: array_base.to_string(),
                        element_type: HirType::Int, // Default, could be improved
                        base_index: Some(0),
                    },
                    confidence: 0.95,
                    reason: format!(
                        "Classifier: array-derived pointer from {} (enables safe slice indexing)",
                        array_base
                    ),
                };
                result.insert(var_name.clone(), inference);
            } else {
                let features = extract_features_for_variable(var_name, graph, func);
                let prediction = classifier.classify(&features);

                let inference = prediction_to_inference(var_name, &prediction, classifier.name());
                result.insert(var_name.clone(), inference);
            }
        }
    }

    result
}

/// Extract features for a single variable from dataflow analysis.
///
/// This bridges the `DataflowGraph` analysis with `OwnershipFeatures` format
/// expected by classifiers.
pub fn extract_features_for_variable(
    var_name: &str,
    graph: &DataflowGraph,
    func: &HirFunction,
) -> OwnershipFeatures {
    let mut builder = OwnershipFeaturesBuilder::default();

    // Extract pointer depth from function parameters
    if let Some(param) = func.parameters().iter().find(|p| p.name() == var_name) {
        builder = builder.pointer_depth(calculate_pointer_depth(param.param_type()));
        builder = builder.const_qualified(is_const_type(param.param_type()));
    }

    // Extract features from dataflow graph using available methods
    // Check if variable is modified (indicates mutable borrow pattern)
    let is_modified = graph.is_modified(var_name);
    if is_modified {
        builder = builder.write_count(1);
    }

    // DECY-183: Check if pointer is derived from array (CRITICAL for safe slice indexing)
    // This detects patterns like: int arr[10]; int* p = arr;
    if graph.array_base_for(var_name).is_some() {
        builder = builder.array_decay(true);
        // Mark as array allocation since it's derived from array
        builder = builder.allocation_site(AllocationKind::Stack);
    }

    // Check for array parameter pattern
    if let Some(is_array) = graph.is_array_parameter(var_name) {
        if is_array {
            builder = builder.array_decay(true);
            // Array parameters come from function arguments
            builder = builder.allocation_site(AllocationKind::Parameter);
        }
    }

    // Check for size parameter pattern
    let has_size = has_size_parameter(var_name, func);
    builder = builder.has_size_param(has_size);

    // Check if it's a parameter (not locally allocated)
    let is_param = func.parameters().iter().any(|p| p.name() == var_name);
    if is_param {
        builder = builder.allocation_site(AllocationKind::Parameter);
    }

    builder.build()
}

/// Convert classifier prediction to ownership inference.
fn prediction_to_inference(
    var_name: &str,
    prediction: &ClassifierPrediction,
    classifier_name: &str,
) -> OwnershipInference {
    OwnershipInference {
        variable: var_name.to_string(),
        kind: inferred_to_ownership_kind(&prediction.prediction, var_name),
        confidence: prediction.confidence as f32,
        reason: format!(
            "{}: {:?} (confidence {:.0}%)",
            classifier_name,
            prediction.prediction,
            prediction.confidence * 100.0
        ),
    }
}

/// Convert `InferredOwnership` to `OwnershipKind`.
fn inferred_to_ownership_kind(inferred: &InferredOwnership, var_name: &str) -> OwnershipKind {
    match inferred {
        InferredOwnership::Owned => OwnershipKind::Owning,
        InferredOwnership::Borrowed => OwnershipKind::ImmutableBorrow,
        InferredOwnership::BorrowedMut => OwnershipKind::MutableBorrow,
        InferredOwnership::Vec => OwnershipKind::Owning, // Vec is also owning
        InferredOwnership::Slice => OwnershipKind::ArrayPointer {
            base_array: var_name.to_string(),
            element_type: HirType::Int, // Default, could be improved
            base_index: Some(0),
        },
        InferredOwnership::SliceMut => OwnershipKind::ArrayPointer {
            base_array: var_name.to_string(),
            element_type: HirType::Int,
            base_index: Some(0),
        },
        InferredOwnership::Shared => OwnershipKind::Unknown, // Rc not yet supported
        InferredOwnership::RawPointer => OwnershipKind::Unknown,
    }
}

/// Calculate pointer depth from HIR type.
fn calculate_pointer_depth(hir_type: &HirType) -> u8 {
    match hir_type {
        HirType::Pointer(inner) => 1 + calculate_pointer_depth(inner),
        HirType::Reference { inner, .. } => 1 + calculate_pointer_depth(inner),
        _ => 0,
    }
}

/// Check if a type is a pointer type.
fn is_pointer_type(hir_type: &HirType) -> bool {
    matches!(hir_type, HirType::Pointer(_) | HirType::Reference { .. })
}

/// Check if a type is const-qualified.
fn is_const_type(hir_type: &HirType) -> bool {
    match hir_type {
        HirType::Reference { mutable, .. } => !mutable,
        _ => false,
    }
}

/// Check if a parameter has an associated size parameter (array pattern).
fn has_size_parameter(var_name: &str, func: &HirFunction) -> bool {
    let params = func.parameters();

    // Look for parameter names that suggest size: len, size, count, n, num
    let size_names = ["len", "size", "count", "n", "num", "length"];

    for param in params {
        let param_name = param.name().to_lowercase();
        if size_names.iter().any(|s| param_name.contains(s)) {
            // Check if this size param is near the variable
            return true;
        }
    }

    // Also check for pattern: arr, arr_len
    let base_name = var_name.trim_end_matches("_ptr").trim_end_matches("_arr");
    params.iter().any(|p| {
        let name = p.name().to_lowercase();
        name == format!("{}_len", base_name)
            || name == format!("{}_size", base_name)
            || name == format!("{}len", base_name)
            || name == format!("{}size", base_name)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataflow::DataflowAnalyzer;
    use decy_hir::HirParameter;

    #[test]
    fn test_classify_with_rules_basic() {
        // DECY-182: Basic classification should work
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![HirParameter::new(
                "data".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            )],
            vec![],
        );

        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        let inferences = classify_with_rules(&graph, &func);

        // Should have an inference for 'data'
        assert!(inferences.contains_key("data"));
    }

    #[test]
    fn test_classify_function_variables_with_custom_classifier() {
        // DECY-182: Custom classifier should be used
        let classifier = RuleBasedClassifier::new();

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![HirParameter::new(
                "ptr".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            )],
            vec![],
        );

        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        let inferences = classify_function_variables(&classifier, &graph, &func);

        assert!(inferences.contains_key("ptr"));
        assert!(inferences["ptr"].confidence >= 0.0);
    }

    #[test]
    fn test_extract_features_for_parameter() {
        // DECY-182: Feature extraction should work for parameters
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![HirParameter::new(
                "arr".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            )],
            vec![],
        );

        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        let features = extract_features_for_variable("arr", &graph, &func);

        assert_eq!(features.pointer_depth, 1);
    }

    #[test]
    fn test_calculate_pointer_depth() {
        assert_eq!(calculate_pointer_depth(&HirType::Int), 0);
        assert_eq!(
            calculate_pointer_depth(&HirType::Pointer(Box::new(HirType::Int))),
            1
        );
        assert_eq!(
            calculate_pointer_depth(&HirType::Pointer(Box::new(HirType::Pointer(Box::new(
                HirType::Int
            ))))),
            2
        );
    }

    #[test]
    fn test_is_pointer_type() {
        assert!(!is_pointer_type(&HirType::Int));
        assert!(is_pointer_type(&HirType::Pointer(Box::new(HirType::Int))));
        assert!(is_pointer_type(&HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        }));
    }

    #[test]
    fn test_has_size_parameter() {
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![
                HirParameter::new("arr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
                HirParameter::new("len".to_string(), HirType::Int),
            ],
            vec![],
        );

        assert!(has_size_parameter("arr", &func));
    }

    #[test]
    fn test_has_size_parameter_named_pattern() {
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![
                HirParameter::new("data".to_string(), HirType::Pointer(Box::new(HirType::Int))),
                HirParameter::new("data_len".to_string(), HirType::Int),
            ],
            vec![],
        );

        assert!(has_size_parameter("data", &func));
    }

    #[test]
    fn test_prediction_to_inference() {
        let prediction = ClassifierPrediction::new(InferredOwnership::Borrowed, 0.85);

        let inference = prediction_to_inference("ptr", &prediction, "RuleBased");

        assert_eq!(inference.variable, "ptr");
        assert_eq!(inference.kind, OwnershipKind::ImmutableBorrow);
        assert!((inference.confidence - 0.85).abs() < 0.01);
        assert!(inference.reason.contains("RuleBased"));
    }

    #[test]
    fn test_inferred_to_ownership_kind_owned() {
        assert_eq!(
            inferred_to_ownership_kind(&InferredOwnership::Owned, "x"),
            OwnershipKind::Owning
        );
    }

    #[test]
    fn test_inferred_to_ownership_kind_borrowed() {
        assert_eq!(
            inferred_to_ownership_kind(&InferredOwnership::Borrowed, "x"),
            OwnershipKind::ImmutableBorrow
        );
    }

    #[test]
    fn test_inferred_to_ownership_kind_borrowed_mut() {
        assert_eq!(
            inferred_to_ownership_kind(&InferredOwnership::BorrowedMut, "x"),
            OwnershipKind::MutableBorrow
        );
    }
}
