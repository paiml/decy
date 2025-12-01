//! [RED] Tests for ML-enhanced ownership inference features.
//!
//! DECY-ML-001: OwnershipFeatures struct
//! DECY-ML-003: Ownership defect taxonomy

use crate::ml_features::*;

// ============================================================================
// DECY-ML-003: OWNERSHIP DEFECT TAXONOMY TESTS
// ============================================================================

#[test]
fn defect_category_has_all_eight_variants() {
    // Spec: 8 defect categories (DECY-O-001 through DECY-O-008)
    let categories = [
        OwnershipDefect::PointerMisclassification,
        OwnershipDefect::LifetimeInferenceGap,
        OwnershipDefect::DanglingPointerRisk,
        OwnershipDefect::AliasViolation,
        OwnershipDefect::UnsafeMinimizationFailure,
        OwnershipDefect::ArraySliceMismatch,
        OwnershipDefect::ResourceLeakPattern,
        OwnershipDefect::MutabilityMismatch,
    ];
    assert_eq!(categories.len(), 8);
}

#[test]
fn defect_category_code_mapping() {
    assert_eq!(OwnershipDefect::PointerMisclassification.code(), "DECY-O-001");
    assert_eq!(OwnershipDefect::LifetimeInferenceGap.code(), "DECY-O-002");
    assert_eq!(OwnershipDefect::DanglingPointerRisk.code(), "DECY-O-003");
    assert_eq!(OwnershipDefect::AliasViolation.code(), "DECY-O-004");
    assert_eq!(OwnershipDefect::UnsafeMinimizationFailure.code(), "DECY-O-005");
    assert_eq!(OwnershipDefect::ArraySliceMismatch.code(), "DECY-O-006");
    assert_eq!(OwnershipDefect::ResourceLeakPattern.code(), "DECY-O-007");
    assert_eq!(OwnershipDefect::MutabilityMismatch.code(), "DECY-O-008");
}

#[test]
fn defect_category_display() {
    let defect = OwnershipDefect::PointerMisclassification;
    let display = format!("{}", defect);
    assert!(display.contains("DECY-O-001"));
}

#[test]
fn defect_category_debug() {
    let defect = OwnershipDefect::AliasViolation;
    let debug = format!("{:?}", defect);
    assert!(debug.contains("AliasViolation"));
}

#[test]
fn defect_category_clone_eq() {
    let d1 = OwnershipDefect::ResourceLeakPattern;
    let d2 = d1.clone();
    assert_eq!(d1, d2);
}

#[test]
fn defect_category_from_code() {
    assert_eq!(
        OwnershipDefect::from_code("DECY-O-001"),
        Some(OwnershipDefect::PointerMisclassification)
    );
    assert_eq!(
        OwnershipDefect::from_code("DECY-O-008"),
        Some(OwnershipDefect::MutabilityMismatch)
    );
    assert_eq!(OwnershipDefect::from_code("INVALID"), None);
}

#[test]
fn defect_category_description() {
    let defect = OwnershipDefect::PointerMisclassification;
    assert!(!defect.description().is_empty());
    assert!(defect.description().contains("Owning") || defect.description().contains("owning"));
}

#[test]
fn defect_severity_levels() {
    // Critical defects that cause memory unsafety
    assert!(OwnershipDefect::DanglingPointerRisk.severity() >= Severity::Critical);
    assert!(OwnershipDefect::AliasViolation.severity() >= Severity::Critical);

    // High severity - incorrect but may compile
    assert!(OwnershipDefect::PointerMisclassification.severity() >= Severity::High);

    // Medium - suboptimal but safe
    assert!(OwnershipDefect::UnsafeMinimizationFailure.severity() >= Severity::Medium);
}

// ============================================================================
// DECY-ML-001: OWNERSHIP FEATURES STRUCT TESTS
// ============================================================================

#[test]
fn allocation_kind_variants() {
    let kinds = [
        AllocationKind::Malloc,
        AllocationKind::Calloc,
        AllocationKind::Realloc,
        AllocationKind::Stack,
        AllocationKind::Static,
        AllocationKind::Parameter,
        AllocationKind::Unknown,
    ];
    assert!(kinds.len() >= 5);
}

#[test]
fn ownership_features_default() {
    let features = OwnershipFeatures::default();
    assert_eq!(features.pointer_depth, 0);
    assert!(!features.is_const);
    assert_eq!(features.read_count, 0);
    assert_eq!(features.write_count, 0);
}

#[test]
fn ownership_features_dimension_constant() {
    // Spec: 142 dimensions for batch processing
    assert!(OwnershipFeatures::DIMENSION > 0);
    // At minimum: syntactic(4) + semantic(4) + usage(4) = 12
    assert!(OwnershipFeatures::DIMENSION >= 12);
}

#[test]
fn ownership_features_to_vector() {
    let features = OwnershipFeatures::default();
    let vec = features.to_vector();
    assert_eq!(vec.len(), OwnershipFeatures::DIMENSION);
}

#[test]
fn ownership_features_builder() {
    let features = OwnershipFeatures::builder()
        .pointer_depth(2)
        .const_qualified(true)
        .allocation_site(AllocationKind::Malloc)
        .build();

    assert_eq!(features.pointer_depth, 2);
    assert!(features.is_const);
    assert_eq!(features.allocation_site, AllocationKind::Malloc);
}

#[test]
fn ownership_features_syntactic() {
    let features = OwnershipFeatures::builder()
        .pointer_depth(1)
        .const_qualified(false)
        .array_decay(true)
        .has_size_param(true)
        .build();

    assert_eq!(features.pointer_depth, 1);
    assert!(features.is_array_decay);
    assert!(features.has_size_param);
}

#[test]
fn ownership_features_semantic() {
    let features = OwnershipFeatures::builder()
        .allocation_site(AllocationKind::Calloc)
        .deallocation_count(1)
        .alias_count(2)
        .escape_scope(true)
        .build();

    assert_eq!(features.allocation_site, AllocationKind::Calloc);
    assert_eq!(features.deallocation_count, 1);
    assert_eq!(features.alias_count, 2);
    assert!(features.escape_scope);
}

#[test]
fn ownership_features_usage_patterns() {
    let features = OwnershipFeatures::builder()
        .read_count(10)
        .write_count(5)
        .arithmetic_ops(2)
        .null_checks(3)
        .build();

    assert_eq!(features.read_count, 10);
    assert_eq!(features.write_count, 5);
    assert_eq!(features.arithmetic_ops, 2);
    assert_eq!(features.null_checks, 3);
}

#[test]
fn ownership_features_clone() {
    let f1 = OwnershipFeatures::builder()
        .pointer_depth(3)
        .read_count(100)
        .build();
    let f2 = f1.clone();

    assert_eq!(f1.pointer_depth, f2.pointer_depth);
    assert_eq!(f1.read_count, f2.read_count);
}

#[test]
fn ownership_features_debug() {
    let features = OwnershipFeatures::default();
    let debug = format!("{:?}", features);
    assert!(debug.contains("OwnershipFeatures"));
}

// ============================================================================
// INFERRED OWNERSHIP KIND TESTS
// ============================================================================

#[test]
fn inferred_ownership_variants() {
    let kinds = [
        InferredOwnership::Owned,      // Box<T>
        InferredOwnership::Borrowed,   // &T
        InferredOwnership::BorrowedMut, // &mut T
        InferredOwnership::Shared,     // Rc<T> / Arc<T>
        InferredOwnership::RawPointer, // *const T / *mut T
        InferredOwnership::Vec,        // Vec<T>
        InferredOwnership::Slice,      // &[T]
        InferredOwnership::SliceMut,   // &mut [T]
    ];
    assert_eq!(kinds.len(), 8);
}

#[test]
fn inferred_ownership_to_rust_type() {
    assert_eq!(InferredOwnership::Owned.to_rust_type("i32"), "Box<i32>");
    assert_eq!(InferredOwnership::Borrowed.to_rust_type("i32"), "&i32");
    assert_eq!(InferredOwnership::BorrowedMut.to_rust_type("i32"), "&mut i32");
    assert_eq!(InferredOwnership::Vec.to_rust_type("i32"), "Vec<i32>");
    assert_eq!(InferredOwnership::Slice.to_rust_type("i32"), "&[i32]");
}

#[test]
fn inferred_ownership_requires_unsafe() {
    assert!(!InferredOwnership::Owned.requires_unsafe());
    assert!(!InferredOwnership::Borrowed.requires_unsafe());
    assert!(InferredOwnership::RawPointer.requires_unsafe());
}

#[test]
fn inferred_ownership_confidence() {
    // Ownership with confidence score
    let result = OwnershipPrediction {
        kind: InferredOwnership::Owned,
        confidence: 0.95,
        fallback: None,
    };
    assert!(result.confidence > 0.65); // Above threshold
    assert!(result.is_confident());
}

#[test]
fn inferred_ownership_fallback() {
    let result = OwnershipPrediction {
        kind: InferredOwnership::Borrowed,
        confidence: 0.50, // Below threshold
        fallback: Some(InferredOwnership::RawPointer),
    };
    assert!(!result.is_confident());
    assert!(result.fallback.is_some());
}

// ============================================================================
// SERIALIZATION TESTS
// ============================================================================

#[test]
fn defect_category_serialize() {
    let defect = OwnershipDefect::PointerMisclassification;
    let json = serde_json::to_string(&defect).unwrap();
    assert!(json.contains("PointerMisclassification"));
}

#[test]
fn ownership_features_serialize() {
    let features = OwnershipFeatures::builder()
        .pointer_depth(1)
        .const_qualified(true)
        .build();
    let json = serde_json::to_string(&features).unwrap();
    assert!(json.contains("pointer_depth"));
}

#[test]
fn ownership_prediction_serialize() {
    let pred = OwnershipPrediction {
        kind: InferredOwnership::Owned,
        confidence: 0.9,
        fallback: None,
    };
    let json = serde_json::to_string(&pred).unwrap();
    let parsed: OwnershipPrediction = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.kind, InferredOwnership::Owned);
}

// ============================================================================
// DECY-ML-002: FEATURE EXTRACTION FROM HIR TESTS
// ============================================================================

use crate::ml_features::FeatureExtractor;
use decy_hir::{HirFunction, HirParameter, HirStatement, HirType};

fn make_function(
    name: &str,
    params: Vec<HirParameter>,
    body: Vec<HirStatement>,
    ret: HirType,
) -> HirFunction {
    HirFunction::new_with_body(name.to_string(), ret, params, body)
}

fn make_param(name: &str, ty: HirType) -> HirParameter {
    HirParameter::new(name.to_string(), ty)
}

#[test]
fn feature_extractor_new() {
    let extractor = FeatureExtractor::new();
    assert_eq!(extractor.extracted_count(), 0);
}

#[test]
fn feature_extractor_extract_pointer_depth() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().pointer_depth, 1);
}

#[test]
fn feature_extractor_extract_double_pointer_depth() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param(
            "ptr",
            HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Int)))),
        )],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().pointer_depth, 2);
}

#[test]
fn feature_extractor_const_reference() {
    let extractor = FeatureExtractor::new();
    // Use immutable reference (const) to test is_const detection
    let func = make_function(
        "test",
        vec![make_param(
            "ptr",
            HirType::Reference {
                inner: Box::new(HirType::Int),
                mutable: false,
            },
        )],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert!(features.unwrap().is_const);
}

#[test]
fn feature_extractor_array_decay_pattern() {
    let extractor = FeatureExtractor::new();
    // Pattern: fn(int* arr, int len) suggests array decay
    let func = make_function(
        "test",
        vec![
            make_param("arr", HirType::Pointer(Box::new(HirType::Int))),
            make_param("len", HirType::Int),
        ],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "arr");
    assert!(features.is_some());
    assert!(features.unwrap().is_array_decay);
}

#[test]
fn feature_extractor_has_size_param() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![
            make_param("buffer", HirType::Pointer(Box::new(HirType::Char))),
            make_param("size", HirType::Int),
        ],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "buffer");
    assert!(features.is_some());
    assert!(features.unwrap().has_size_param);
}

#[test]
fn feature_extractor_allocation_site_malloc() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "ptr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(decy_hir::HirExpression::Malloc {
                size: Box::new(decy_hir::HirExpression::IntLiteral(4)),
            }),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_variable(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().allocation_site, AllocationKind::Malloc);
}

#[test]
fn feature_extractor_allocation_site_parameter() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().allocation_site, AllocationKind::Parameter);
}

#[test]
fn feature_extractor_deallocation_count() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::Free {
            pointer: decy_hir::HirExpression::Variable("ptr".to_string()),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().deallocation_count, 1);
}

#[test]
fn feature_extractor_null_checks() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::If {
            condition: decy_hir::HirExpression::IsNotNull(Box::new(
                decy_hir::HirExpression::Variable("ptr".to_string()),
            )),
            then_block: vec![],
            else_block: None,
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().null_checks, 1);
}

#[test]
fn feature_extractor_non_pointer_returns_none() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("x", HirType::Int)],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "x");
    assert!(features.is_none());
}

#[test]
fn feature_extractor_unknown_variable_returns_none() {
    let extractor = FeatureExtractor::new();
    let func = make_function("test", vec![], vec![], HirType::Void);

    let features = extractor.extract_for_parameter(&func, "nonexistent");
    assert!(features.is_none());
}

#[test]
fn feature_extractor_extract_all_pointers() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![
            make_param("a", HirType::Pointer(Box::new(HirType::Int))),
            make_param("b", HirType::Int),
            make_param("c", HirType::Pointer(Box::new(HirType::Char))),
        ],
        vec![],
        HirType::Void,
    );

    let all_features = extractor.extract_all(&func);
    assert_eq!(all_features.len(), 2); // Only pointer parameters
}
