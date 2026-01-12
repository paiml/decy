//! [RED] Tests for ML-enhanced ownership inference features.
//!
//! DECY-ML-001: OwnershipFeatures struct
//! DECY-ML-003: Ownership defect taxonomy

use crate::ml_features::*;
use decy_hir::HirExpression;

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
    assert_eq!(
        OwnershipDefect::PointerMisclassification.code(),
        "DECY-O-001"
    );
    assert_eq!(OwnershipDefect::LifetimeInferenceGap.code(), "DECY-O-002");
    assert_eq!(OwnershipDefect::DanglingPointerRisk.code(), "DECY-O-003");
    assert_eq!(OwnershipDefect::AliasViolation.code(), "DECY-O-004");
    assert_eq!(
        OwnershipDefect::UnsafeMinimizationFailure.code(),
        "DECY-O-005"
    );
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
    let d2 = d1;
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
    const _: () = assert!(OwnershipFeatures::DIMENSION > 0);
    // At minimum: syntactic(4) + semantic(4) + usage(4) = 12
    const _: () = assert!(OwnershipFeatures::DIMENSION >= 12);
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
        InferredOwnership::Owned,       // Box<T>
        InferredOwnership::Borrowed,    // &T
        InferredOwnership::BorrowedMut, // &mut T
        InferredOwnership::Shared,      // Rc<T> / Arc<T>
        InferredOwnership::RawPointer,  // *const T / *mut T
        InferredOwnership::Vec,         // Vec<T>
        InferredOwnership::Slice,       // &[T]
        InferredOwnership::SliceMut,    // &mut [T]
    ];
    assert_eq!(kinds.len(), 8);
}

#[test]
fn inferred_ownership_to_rust_type() {
    assert_eq!(InferredOwnership::Owned.to_rust_type("i32"), "Box<i32>");
    assert_eq!(InferredOwnership::Borrowed.to_rust_type("i32"), "&i32");
    assert_eq!(
        InferredOwnership::BorrowedMut.to_rust_type("i32"),
        "&mut i32"
    );
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

// ============================================================================
// DECY-ML-009: ERROR PATTERN LIBRARY TESTS
// ============================================================================

use crate::ml_features::{
    ErrorPattern, ErrorSeverity, OwnershipErrorKind, PatternLibrary, SuggestedFix,
};

#[test]
fn error_pattern_new() {
    let pattern = ErrorPattern::new(
        "malloc_without_box",
        OwnershipErrorKind::PointerMisclassification,
        "malloc result used as &T",
    );
    assert_eq!(pattern.id(), "malloc_without_box");
    assert_eq!(
        pattern.error_kind(),
        OwnershipErrorKind::PointerMisclassification
    );
}

#[test]
fn error_pattern_with_c_pattern() {
    let pattern = ErrorPattern::new(
        "test",
        OwnershipErrorKind::PointerMisclassification,
        "test pattern",
    )
    .with_c_pattern("int* p = malloc(sizeof(int));");

    assert_eq!(pattern.c_pattern(), Some("int* p = malloc(sizeof(int));"));
}

#[test]
fn error_pattern_with_rust_error() {
    let pattern = ErrorPattern::new(
        "test",
        OwnershipErrorKind::LifetimeInferenceGap,
        "missing lifetime",
    )
    .with_rust_error("E0106: missing lifetime specifier");

    assert_eq!(
        pattern.rust_error(),
        Some("E0106: missing lifetime specifier")
    );
}

#[test]
fn error_pattern_with_fix() {
    let fix = SuggestedFix::new("Use Box<T>", "Box::new(value)");
    let pattern = ErrorPattern::new("test", OwnershipErrorKind::PointerMisclassification, "test")
        .with_fix(fix);

    assert!(pattern.suggested_fix().is_some());
    assert_eq!(pattern.suggested_fix().unwrap().description(), "Use Box<T>");
}

#[test]
fn error_pattern_severity() {
    let pattern = ErrorPattern::new(
        "test",
        OwnershipErrorKind::DanglingPointerRisk,
        "use after free",
    )
    .with_severity(ErrorSeverity::Critical);

    assert_eq!(pattern.severity(), ErrorSeverity::Critical);
}

#[test]
fn error_pattern_curriculum_level() {
    let pattern = ErrorPattern::new(
        "test",
        OwnershipErrorKind::ArraySliceMismatch,
        "array vs slice",
    )
    .with_curriculum_level(2);

    assert_eq!(pattern.curriculum_level(), 2);
}

#[test]
fn suggested_fix_new() {
    let fix = SuggestedFix::new("Use reference", "&value");
    assert_eq!(fix.description(), "Use reference");
    assert_eq!(fix.code_template(), "&value");
}

#[test]
fn suggested_fix_with_confidence() {
    let fix = SuggestedFix::new("Use Box", "Box::new(x)").with_confidence(0.95);
    assert!((fix.confidence() - 0.95).abs() < f32::EPSILON);
}

#[test]
fn pattern_library_new() {
    let library = PatternLibrary::new();
    assert_eq!(library.len(), 0);
    assert!(library.is_empty());
}

#[test]
fn pattern_library_add_pattern() {
    let mut library = PatternLibrary::new();
    let pattern = ErrorPattern::new(
        "test_pattern",
        OwnershipErrorKind::MutabilityMismatch,
        "const vs mutable",
    );

    library.add(pattern);
    assert_eq!(library.len(), 1);
    assert!(!library.is_empty());
}

#[test]
fn pattern_library_get_by_id() {
    let mut library = PatternLibrary::new();
    library.add(ErrorPattern::new(
        "pattern_1",
        OwnershipErrorKind::ResourceLeakPattern,
        "memory leak",
    ));

    let found = library.get("pattern_1");
    assert!(found.is_some());
    assert_eq!(found.unwrap().id(), "pattern_1");
}

#[test]
fn pattern_library_get_by_error_kind() {
    let mut library = PatternLibrary::new();
    library.add(ErrorPattern::new(
        "p1",
        OwnershipErrorKind::AliasViolation,
        "alias 1",
    ));
    library.add(ErrorPattern::new(
        "p2",
        OwnershipErrorKind::AliasViolation,
        "alias 2",
    ));
    library.add(ErrorPattern::new(
        "p3",
        OwnershipErrorKind::LifetimeInferenceGap,
        "lifetime",
    ));

    let alias_patterns = library.get_by_error_kind(OwnershipErrorKind::AliasViolation);
    assert_eq!(alias_patterns.len(), 2);
}

#[test]
fn pattern_library_curriculum_ordered() {
    let mut library = PatternLibrary::new();
    library.add(
        ErrorPattern::new("hard", OwnershipErrorKind::AliasViolation, "hard pattern")
            .with_curriculum_level(3),
    );
    library.add(
        ErrorPattern::new("easy", OwnershipErrorKind::PointerMisclassification, "easy")
            .with_curriculum_level(1),
    );
    library.add(
        ErrorPattern::new("medium", OwnershipErrorKind::LifetimeInferenceGap, "medium")
            .with_curriculum_level(2),
    );

    let ordered = library.curriculum_ordered();
    assert_eq!(ordered.len(), 3);
    assert_eq!(ordered[0].id(), "easy");
    assert_eq!(ordered[1].id(), "medium");
    assert_eq!(ordered[2].id(), "hard");
}

#[test]
fn pattern_library_match_rust_error() {
    let mut library = PatternLibrary::new();
    library.add(
        ErrorPattern::new(
            "lifetime_err",
            OwnershipErrorKind::LifetimeInferenceGap,
            "lifetime",
        )
        .with_rust_error("E0106"),
    );
    library.add(
        ErrorPattern::new("borrow_err", OwnershipErrorKind::AliasViolation, "borrow")
            .with_rust_error("E0502"),
    );

    let matches = library.match_rust_error("E0106: missing lifetime specifier");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].id(), "lifetime_err");
}

#[test]
fn pattern_library_serialize() {
    let mut library = PatternLibrary::new();
    library.add(ErrorPattern::new(
        "test",
        OwnershipErrorKind::UnsafeMinimizationFailure,
        "unnecessary unsafe",
    ));

    let json = serde_json::to_string(&library).unwrap();
    assert!(json.contains("test"));
    assert!(json.contains("UnsafeMinimizationFailure"));
}

#[test]
fn pattern_library_deserialize() {
    let mut library = PatternLibrary::new();
    library.add(ErrorPattern::new(
        "deser_test",
        OwnershipErrorKind::DanglingPointerRisk,
        "dangling",
    ));

    let json = serde_json::to_string(&library).unwrap();
    let parsed: PatternLibrary = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.len(), 1);
    assert!(parsed.get("deser_test").is_some());
}

#[test]
fn ownership_error_kind_from_defect() {
    // OwnershipErrorKind should map to OwnershipDefect
    assert_eq!(
        OwnershipErrorKind::PointerMisclassification.to_defect(),
        OwnershipDefect::PointerMisclassification
    );
    assert_eq!(
        OwnershipErrorKind::DanglingPointerRisk.to_defect(),
        OwnershipDefect::DanglingPointerRisk
    );
}

// ============================================================================
// DECY-ML-007: CURRICULUM-ORDERED DEFAULT PATTERNS TESTS
// ============================================================================

use crate::ml_features::default_pattern_library;

#[test]
fn default_library_not_empty() {
    let library = default_pattern_library();
    assert!(!library.is_empty());
}

#[test]
fn default_library_has_all_error_kinds() {
    let library = default_pattern_library();

    // Should have at least one pattern for each error kind
    assert!(!library
        .get_by_error_kind(OwnershipErrorKind::PointerMisclassification)
        .is_empty());
    assert!(!library
        .get_by_error_kind(OwnershipErrorKind::LifetimeInferenceGap)
        .is_empty());
    assert!(!library
        .get_by_error_kind(OwnershipErrorKind::DanglingPointerRisk)
        .is_empty());
    assert!(!library
        .get_by_error_kind(OwnershipErrorKind::AliasViolation)
        .is_empty());
    assert!(!library
        .get_by_error_kind(OwnershipErrorKind::UnsafeMinimizationFailure)
        .is_empty());
    assert!(!library
        .get_by_error_kind(OwnershipErrorKind::ArraySliceMismatch)
        .is_empty());
    assert!(!library
        .get_by_error_kind(OwnershipErrorKind::ResourceLeakPattern)
        .is_empty());
    assert!(!library
        .get_by_error_kind(OwnershipErrorKind::MutabilityMismatch)
        .is_empty());
}

#[test]
fn default_library_curriculum_levels_ascending() {
    let library = default_pattern_library();
    let ordered = library.curriculum_ordered();

    // Verify levels are in ascending order
    let mut prev_level = 0u8;
    for pattern in ordered {
        assert!(
            pattern.curriculum_level() >= prev_level,
            "Curriculum levels should be ascending"
        );
        prev_level = pattern.curriculum_level();
    }
}

#[test]
fn default_library_has_c_patterns() {
    let library = default_pattern_library();

    // Most patterns should have C code examples
    let with_c_pattern = library.iter().filter(|p| p.c_pattern().is_some()).count();
    assert!(with_c_pattern > library.len() / 2);
}

#[test]
fn default_library_has_rust_errors() {
    let library = default_pattern_library();

    // Most patterns should have rustc error codes
    let with_rust_error = library.iter().filter(|p| p.rust_error().is_some()).count();
    assert!(with_rust_error > library.len() / 2);
}

#[test]
fn default_library_has_suggested_fixes() {
    let library = default_pattern_library();

    // Most patterns should have suggested fixes
    let with_fix = library
        .iter()
        .filter(|p| p.suggested_fix().is_some())
        .count();
    assert!(with_fix > library.len() / 2);
}

#[test]
fn default_library_malloc_pattern() {
    let library = default_pattern_library();

    // Should have the basic malloc → Box pattern
    let malloc_patterns = library.get_by_error_kind(OwnershipErrorKind::PointerMisclassification);
    let has_malloc = malloc_patterns
        .iter()
        .any(|p| p.c_pattern().is_some_and(|c| c.contains("malloc")));
    assert!(has_malloc, "Should have malloc misclassification pattern");
}

#[test]
fn default_library_lifetime_pattern() {
    let library = default_pattern_library();

    // Should have lifetime error patterns with E0106
    let matches = library.match_rust_error("E0106");
    assert!(!matches.is_empty(), "Should have E0106 lifetime pattern");
}

#[test]
fn default_library_use_after_free_pattern() {
    let library = default_pattern_library();

    // Should have use-after-free pattern
    let uaf_patterns = library.get_by_error_kind(OwnershipErrorKind::DanglingPointerRisk);
    let has_uaf = uaf_patterns.iter().any(|p| {
        p.description().to_lowercase().contains("free")
            || p.c_pattern().is_some_and(|c: &str| c.contains("free"))
    });
    assert!(has_uaf, "Should have use-after-free pattern");
}

#[test]
fn default_library_borrow_checker_pattern() {
    let library = default_pattern_library();

    // Should have borrow checker violation patterns with E0502 or E0499
    let e0502_matches = library.match_rust_error("E0502");
    let e0499_matches = library.match_rust_error("E0499");
    assert!(
        !e0502_matches.is_empty() || !e0499_matches.is_empty(),
        "Should have borrow checker error patterns"
    );
}

// ============================================================================
// ADDITIONAL COVERAGE TESTS
// ============================================================================

#[test]
fn inferred_ownership_to_rust_type_all_variants() {
    assert_eq!(InferredOwnership::Owned.to_rust_type("i32"), "Box<i32>");
    assert_eq!(InferredOwnership::Borrowed.to_rust_type("i32"), "&i32");
    assert_eq!(
        InferredOwnership::BorrowedMut.to_rust_type("i32"),
        "&mut i32"
    );
    assert_eq!(InferredOwnership::Shared.to_rust_type("i32"), "Rc<i32>");
    assert_eq!(
        InferredOwnership::RawPointer.to_rust_type("i32"),
        "*const i32"
    );
    assert_eq!(InferredOwnership::Vec.to_rust_type("i32"), "Vec<i32>");
    assert_eq!(InferredOwnership::Slice.to_rust_type("i32"), "&[i32]");
    assert_eq!(
        InferredOwnership::SliceMut.to_rust_type("i32"),
        "&mut [i32]"
    );
}

#[test]
fn ownership_prediction_is_confident() {
    let confident = OwnershipPrediction {
        kind: InferredOwnership::Owned,
        confidence: 0.8,
        fallback: None,
    };
    assert!(confident.is_confident());

    let not_confident = OwnershipPrediction {
        kind: InferredOwnership::Owned,
        confidence: 0.5,
        fallback: None,
    };
    assert!(!not_confident.is_confident());
}

#[test]
fn ownership_prediction_effective_ownership() {
    // Confident prediction uses kind
    let confident = OwnershipPrediction {
        kind: InferredOwnership::Owned,
        confidence: 0.8,
        fallback: Some(InferredOwnership::Borrowed),
    };
    assert_eq!(confident.effective_ownership(), InferredOwnership::Owned);

    // Not confident uses fallback
    let not_confident = OwnershipPrediction {
        kind: InferredOwnership::Owned,
        confidence: 0.5,
        fallback: Some(InferredOwnership::Borrowed),
    };
    assert_eq!(
        not_confident.effective_ownership(),
        InferredOwnership::Borrowed
    );

    // Not confident without fallback uses RawPointer
    let no_fallback = OwnershipPrediction {
        kind: InferredOwnership::Owned,
        confidence: 0.5,
        fallback: None,
    };
    assert_eq!(
        no_fallback.effective_ownership(),
        InferredOwnership::RawPointer
    );
}

#[test]
fn ownership_prediction_partial_eq() {
    let p1 = OwnershipPrediction {
        kind: InferredOwnership::Owned,
        confidence: 0.8,
        fallback: None,
    };
    let p2 = OwnershipPrediction {
        kind: InferredOwnership::Owned,
        confidence: 0.8,
        fallback: None,
    };
    assert_eq!(p1, p2);
}

#[test]
fn allocation_kind_default() {
    let kind: AllocationKind = Default::default();
    assert_eq!(kind, AllocationKind::Unknown);
}

#[test]
fn allocation_kind_copy_debug() {
    let kind = AllocationKind::Malloc;
    let copied = kind; // Copy trait
    assert_eq!(kind, copied);
    let debug = format!("{:?}", kind);
    assert!(debug.contains("Malloc"));
}

#[test]
fn severity_comparison() {
    assert!(Severity::Critical > Severity::High);
    assert!(Severity::High > Severity::Medium);
    assert!(Severity::Medium > Severity::Info);
}

#[test]
fn ownership_defect_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(OwnershipDefect::PointerMisclassification);
    set.insert(OwnershipDefect::LifetimeInferenceGap);
    assert!(set.contains(&OwnershipDefect::PointerMisclassification));
}

#[test]
fn feature_extractor_extracted_count() {
    let extractor = FeatureExtractor::new();
    assert_eq!(extractor.extracted_count(), 0);
}

#[test]
fn feature_extractor_extract_for_variable_pointer() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "p".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(4)],
            }),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_variable(&func, "p");
    assert!(features.is_some());
    let f = features.unwrap();
    assert_eq!(f.allocation_site, AllocationKind::Malloc);
}

#[test]
fn feature_extractor_extract_for_variable_calloc() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "p".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "calloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(10), HirExpression::IntLiteral(4)],
            }),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_variable(&func, "p");
    assert!(features.is_some());
    let f = features.unwrap();
    assert_eq!(f.allocation_site, AllocationKind::Calloc);
}

#[test]
fn feature_extractor_extract_for_variable_realloc() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "p".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "realloc".to_string(),
                arguments: vec![HirExpression::NullLiteral, HirExpression::IntLiteral(4)],
            }),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_variable(&func, "p");
    assert!(features.is_some());
    let f = features.unwrap();
    assert_eq!(f.allocation_site, AllocationKind::Realloc);
}

#[test]
fn error_severity_default() {
    let sev: ErrorSeverity = Default::default();
    assert_eq!(sev, ErrorSeverity::Info);
}

#[test]
fn inferred_ownership_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(InferredOwnership::Owned);
    set.insert(InferredOwnership::Borrowed);
    assert!(set.contains(&InferredOwnership::Owned));
}
