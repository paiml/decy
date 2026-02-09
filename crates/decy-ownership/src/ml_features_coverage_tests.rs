//! Targeted coverage tests for ml_features.rs
//!
//! Covers branches and paths not exercised by ml_features_tests.rs,
//! including: allocation_kind_to_f32 variants, compute_pointer_depth for Box/Reference,
//! detect_array_decay edge cases, count_free_in_stmt in nested blocks,
//! check_escape, count_stmt_accesses for DerefAssignment, null check via BinaryOp,
//! expr_uses_var for all expression variants, ErrorPattern occurrence tracking,
//! PatternLibrary get_mut/iter, and OwnershipErrorKind to_defect for all variants.

use crate::ml_features::*;
use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType,
               UnaryOperator};

// ============================================================================
// HELPERS
// ============================================================================

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

// ============================================================================
// to_vector: allocation_kind_to_f32 BRANCH COVERAGE
// ============================================================================

#[test]
fn to_vector_allocation_kind_calloc() {
    let features = OwnershipFeatures {
        allocation_site: AllocationKind::Calloc,
        ..Default::default()
    };
    let vec = features.to_vector();
    assert_eq!(vec.len(), OwnershipFeatures::DIMENSION);
    // allocation kind is at index 4 (after 4 syntactic features)
    assert!((vec[4] - 2.0).abs() < f32::EPSILON, "Calloc should map to 2.0");
}

#[test]
fn to_vector_allocation_kind_realloc() {
    let features = OwnershipFeatures {
        allocation_site: AllocationKind::Realloc,
        ..Default::default()
    };
    let vec = features.to_vector();
    assert!((vec[4] - 3.0).abs() < f32::EPSILON, "Realloc should map to 3.0");
}

#[test]
fn to_vector_allocation_kind_stack() {
    let features = OwnershipFeatures {
        allocation_site: AllocationKind::Stack,
        ..Default::default()
    };
    let vec = features.to_vector();
    assert!((vec[4] - 4.0).abs() < f32::EPSILON, "Stack should map to 4.0");
}

#[test]
fn to_vector_allocation_kind_static() {
    let features = OwnershipFeatures {
        allocation_site: AllocationKind::Static,
        ..Default::default()
    };
    let vec = features.to_vector();
    assert!((vec[4] - 5.0).abs() < f32::EPSILON, "Static should map to 5.0");
}

#[test]
fn to_vector_allocation_kind_parameter() {
    let features = OwnershipFeatures {
        allocation_site: AllocationKind::Parameter,
        ..Default::default()
    };
    let vec = features.to_vector();
    assert!((vec[4] - 6.0).abs() < f32::EPSILON, "Parameter should map to 6.0");
}

#[test]
fn to_vector_allocation_kind_unknown() {
    let features = OwnershipFeatures {
        allocation_site: AllocationKind::Unknown,
        ..Default::default()
    };
    let vec = features.to_vector();
    assert!((vec[4] - 0.0).abs() < f32::EPSILON, "Unknown should map to 0.0");
}

#[test]
fn to_vector_allocation_kind_malloc() {
    let features = OwnershipFeatures {
        allocation_site: AllocationKind::Malloc,
        ..Default::default()
    };
    let vec = features.to_vector();
    assert!((vec[4] - 1.0).abs() < f32::EPSILON, "Malloc should map to 1.0");
}

#[test]
fn to_vector_all_features_populated() {
    let features = OwnershipFeatures {
        pointer_depth: 3,
        is_const: true,
        is_array_decay: true,
        has_size_param: true,
        allocation_site: AllocationKind::Realloc,
        deallocation_count: 2,
        alias_count: 4,
        escape_scope: true,
        read_count: 100,
        write_count: 50,
        arithmetic_ops: 7,
        null_checks: 3,
    };
    let vec = features.to_vector();
    assert_eq!(vec.len(), OwnershipFeatures::DIMENSION);
    // Syntactic
    assert!((vec[0] - 3.0).abs() < f32::EPSILON); // pointer_depth
    assert!((vec[1] - 1.0).abs() < f32::EPSILON); // is_const = true
    assert!((vec[2] - 1.0).abs() < f32::EPSILON); // is_array_decay = true
    assert!((vec[3] - 1.0).abs() < f32::EPSILON); // has_size_param = true
    // Semantic
    assert!((vec[4] - 3.0).abs() < f32::EPSILON); // Realloc
    assert!((vec[5] - 2.0).abs() < f32::EPSILON); // deallocation_count
    assert!((vec[6] - 4.0).abs() < f32::EPSILON); // alias_count
    assert!((vec[7] - 1.0).abs() < f32::EPSILON); // escape_scope = true
    // Usage patterns
    assert!((vec[8] - 100.0).abs() < f32::EPSILON); // read_count
    assert!((vec[9] - 50.0).abs() < f32::EPSILON);  // write_count
    assert!((vec[10] - 7.0).abs() < f32::EPSILON);  // arithmetic_ops
    assert!((vec[11] - 3.0).abs() < f32::EPSILON);  // null_checks
    // Padding should be zeros
    for i in 12..OwnershipFeatures::DIMENSION {
        assert!((vec[i] - 0.0).abs() < f32::EPSILON, "Padding at index {} should be 0.0", i);
    }
}

// ============================================================================
// compute_pointer_depth: Box and Reference branches
// ============================================================================

#[test]
fn feature_extractor_pointer_depth_box_type() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Box(Box::new(HirType::Int)))],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().pointer_depth, 1);
}

#[test]
fn feature_extractor_pointer_depth_box_nested() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param(
            "ptr",
            HirType::Box(Box::new(HirType::Box(Box::new(HirType::Int)))),
        )],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().pointer_depth, 2);
}

#[test]
fn feature_extractor_pointer_depth_mutable_reference() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param(
            "ptr",
            HirType::Reference {
                inner: Box::new(HirType::Int),
                mutable: true,
            },
        )],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    let f = features.unwrap();
    assert_eq!(f.pointer_depth, 1);
    // mutable reference should NOT be const
    assert!(!f.is_const);
}

#[test]
fn feature_extractor_pointer_depth_reference_to_pointer() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param(
            "ptr",
            HirType::Reference {
                inner: Box::new(HirType::Pointer(Box::new(HirType::Int))),
                mutable: false,
            },
        )],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().pointer_depth, 2);
}

// ============================================================================
// is_pointer_like: Vec type branch
// ============================================================================

#[test]
fn feature_extractor_vec_type_is_pointer_like() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("arr", HirType::Vec(Box::new(HirType::Int)))],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "arr");
    assert!(features.is_some(), "Vec type should be treated as pointer-like");
}

// ============================================================================
// is_const_type: non-Reference returns false
// ============================================================================

#[test]
fn feature_extractor_is_const_pointer_type() {
    // A plain Pointer type is not detected as const (only Reference{mutable:false} is)
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert!(!features.unwrap().is_const, "Pointer type should not be const");
}

// ============================================================================
// detect_array_decay: various naming patterns and edge cases
// ============================================================================

#[test]
fn feature_extractor_array_decay_count_name() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![
            make_param("data", HirType::Pointer(Box::new(HirType::Int))),
            make_param("count", HirType::Int),
        ],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "data");
    assert!(features.is_some());
    assert!(features.unwrap().is_array_decay, "'count' should trigger array decay");
}

#[test]
fn feature_extractor_array_decay_num_name() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![
            make_param("items", HirType::Pointer(Box::new(HirType::Int))),
            make_param("num_items", HirType::Int),
        ],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "items");
    assert!(features.is_some());
    assert!(features.unwrap().is_array_decay, "'num_items' should trigger array decay");
}

#[test]
fn feature_extractor_array_decay_n_name() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![
            make_param("arr", HirType::Pointer(Box::new(HirType::Int))),
            make_param("n", HirType::Int),
        ],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "arr");
    assert!(features.is_some());
    assert!(features.unwrap().is_array_decay, "'n' should trigger array decay");
}

#[test]
fn feature_extractor_array_decay_unsigned_int_size_param() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![
            make_param("buf", HirType::Pointer(Box::new(HirType::Char))),
            make_param("size", HirType::UnsignedInt),
        ],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "buf");
    assert!(features.is_some());
    assert!(features.unwrap().is_array_decay, "UnsignedInt size param should trigger array decay");
}

#[test]
fn feature_extractor_no_array_decay_last_param() {
    // Last parameter with no successor should not detect array decay
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert!(!features.unwrap().is_array_decay, "Last param should not detect array decay");
}

#[test]
fn feature_extractor_no_array_decay_non_int_next() {
    // Next param is not Int/UnsignedInt
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![
            make_param("ptr", HirType::Pointer(Box::new(HirType::Int))),
            make_param("len", HirType::Float),
        ],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert!(!features.unwrap().is_array_decay, "Float next param should not trigger array decay");
}

#[test]
fn feature_extractor_no_array_decay_non_size_name() {
    // Next param is Int but name doesn't match size patterns
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![
            make_param("ptr", HirType::Pointer(Box::new(HirType::Int))),
            make_param("flags", HirType::Int),
        ],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert!(!features.unwrap().is_array_decay, "'flags' should not trigger array decay");
}

#[test]
fn feature_extractor_no_array_decay_non_pointer_current() {
    // Current param is Reference, not Pointer (detect_array_decay requires Pointer)
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![
            make_param("ptr", HirType::Reference {
                inner: Box::new(HirType::Int),
                mutable: false,
            }),
            make_param("len", HirType::Int),
        ],
        vec![],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert!(!features.unwrap().is_array_decay, "Reference type should not trigger array decay");
}

// ============================================================================
// classify_allocation: Calloc/Realloc HirExpression variants
// ============================================================================

#[test]
fn feature_extractor_allocation_calloc_expr() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::Calloc {
                count: Box::new(HirExpression::IntLiteral(10)),
                element_type: Box::new(HirType::Int),
            }),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_variable(&func, "arr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().allocation_site, AllocationKind::Calloc);
}

#[test]
fn feature_extractor_allocation_realloc_expr() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::Realloc {
                pointer: Box::new(HirExpression::Variable("old".to_string())),
                new_size: Box::new(HirExpression::IntLiteral(20)),
            }),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_variable(&func, "arr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().allocation_site, AllocationKind::Realloc);
}

#[test]
fn feature_extractor_allocation_unknown_expr() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "ptr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::Variable("other".to_string())),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_variable(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().allocation_site, AllocationKind::Unknown);
}

#[test]
fn feature_extractor_allocation_no_initializer() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "ptr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: None,
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_variable(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().allocation_site, AllocationKind::Unknown);
}

// ============================================================================
// extract_for_variable: non-pointer and missing variable
// ============================================================================

#[test]
fn feature_extractor_variable_non_pointer_returns_none() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "x".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(42)),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_variable(&func, "x");
    assert!(features.is_none(), "Non-pointer variable should return None");
}

#[test]
fn feature_extractor_variable_not_found_returns_none() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "x".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: None,
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_variable(&func, "nonexistent");
    assert!(features.is_none(), "Missing variable should return None");
}

// ============================================================================
// count_free_in_stmt: nested blocks (If with else, While, For)
// ============================================================================

#[test]
fn feature_extractor_free_in_if_then_block() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::If {
            condition: HirExpression::Variable("flag".to_string()),
            then_block: vec![HirStatement::Free {
                pointer: HirExpression::Variable("ptr".to_string()),
            }],
            else_block: None,
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().deallocation_count, 1);
}

#[test]
fn feature_extractor_free_in_if_else_block() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::If {
            condition: HirExpression::Variable("flag".to_string()),
            then_block: vec![],
            else_block: Some(vec![HirStatement::Free {
                pointer: HirExpression::Variable("ptr".to_string()),
            }]),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().deallocation_count, 1);
}

#[test]
fn feature_extractor_free_in_both_if_branches() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::If {
            condition: HirExpression::Variable("flag".to_string()),
            then_block: vec![HirStatement::Free {
                pointer: HirExpression::Variable("ptr".to_string()),
            }],
            else_block: Some(vec![HirStatement::Free {
                pointer: HirExpression::Variable("ptr".to_string()),
            }]),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().deallocation_count, 2);
}

#[test]
fn feature_extractor_free_in_while_loop() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::While {
            condition: HirExpression::Variable("flag".to_string()),
            body: vec![HirStatement::Free {
                pointer: HirExpression::Variable("ptr".to_string()),
            }],
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().deallocation_count, 1);
}

#[test]
fn feature_extractor_free_in_for_loop() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::For {
            init: vec![],
            condition: HirExpression::Variable("flag".to_string()),
            increment: vec![],
            body: vec![HirStatement::Free {
                pointer: HirExpression::Variable("ptr".to_string()),
            }],
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().deallocation_count, 1);
}

#[test]
fn feature_extractor_free_not_matching_var() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::Free {
            pointer: HirExpression::Variable("other".to_string()),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().deallocation_count, 0);
}

#[test]
fn feature_extractor_free_in_non_matching_stmt() {
    // A statement type that is not Free, If, While, or For
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::Break],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().deallocation_count, 0);
}

// ============================================================================
// check_escape: variable returned from function
// ============================================================================

#[test]
fn feature_extractor_escape_via_return() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::Return(Some(HirExpression::Variable(
            "ptr".to_string(),
        )))],
        HirType::Pointer(Box::new(HirType::Int)),
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert!(features.unwrap().escape_scope, "Returned variable should escape scope");
}

#[test]
fn feature_extractor_no_escape_return_other_var() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::Return(Some(HirExpression::Variable(
            "other".to_string(),
        )))],
        HirType::Pointer(Box::new(HirType::Int)),
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert!(!features.unwrap().escape_scope, "Non-matching return should not escape");
}

#[test]
fn feature_extractor_no_escape_return_none() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::Return(None)],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert!(!features.unwrap().escape_scope, "Return(None) should not cause escape");
}

// ============================================================================
// count_stmt_accesses: Assignment and DerefAssignment branches
// ============================================================================

#[test]
fn feature_extractor_assignment_target_is_var() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::Assignment {
            target: "ptr".to_string(),
            value: HirExpression::Variable("other".to_string()),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    let f = features.unwrap();
    assert_eq!(f.write_count, 1, "Assignment to var should count as write");
    assert_eq!(f.read_count, 0, "Assignment from other should not count as read for ptr");
}

#[test]
fn feature_extractor_assignment_value_uses_var() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::Assignment {
            target: "other".to_string(),
            value: HirExpression::Variable("ptr".to_string()),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    let f = features.unwrap();
    assert_eq!(f.write_count, 0);
    // Read counts: 1 for value usage + 1 for the extra read check (target != var_name && expr_uses_var)
    assert!(f.read_count >= 1, "Assignment value using var should count reads");
}

#[test]
fn feature_extractor_deref_assignment_target_uses_var() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("ptr".to_string()),
            value: HirExpression::IntLiteral(42),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    let f = features.unwrap();
    assert_eq!(f.write_count, 1, "DerefAssignment to var should count as write");
    assert_eq!(f.read_count, 1, "DerefAssignment target also counts as read");
}

#[test]
fn feature_extractor_deref_assignment_value_uses_var() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("other".to_string()),
            value: HirExpression::Variable("ptr".to_string()),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    let f = features.unwrap();
    assert_eq!(f.write_count, 0, "DerefAssignment to other should not count as write for ptr");
    assert_eq!(f.read_count, 1, "DerefAssignment value using var should count as read");
}

#[test]
fn feature_extractor_deref_assignment_neither_uses_var() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("a".to_string()),
            value: HirExpression::Variable("b".to_string()),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    let f = features.unwrap();
    assert_eq!(f.write_count, 0);
    assert_eq!(f.read_count, 0);
}

// ============================================================================
// count_stmt_accesses: If with condition using var and else block
// ============================================================================

#[test]
fn feature_extractor_if_condition_reads_var() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::If {
            condition: HirExpression::Variable("ptr".to_string()),
            then_block: vec![],
            else_block: None,
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert!(features.unwrap().read_count >= 1, "If condition using var should count as read");
}

#[test]
fn feature_extractor_if_else_block_accesses() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::If {
            condition: HirExpression::IntLiteral(1),
            then_block: vec![],
            else_block: Some(vec![HirStatement::Assignment {
                target: "ptr".to_string(),
                value: HirExpression::IntLiteral(0),
            }]),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().write_count, 1, "Assignment in else block should count");
}

// ============================================================================
// count_stmt_accesses: non-matching statement returns (0, 0)
// ============================================================================

#[test]
fn feature_extractor_break_no_accesses() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::Break],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    let f = features.unwrap();
    assert_eq!(f.read_count, 0);
    assert_eq!(f.write_count, 0);
}

// ============================================================================
// count_null_checks_in_stmt: While loop with null check
// ============================================================================

#[test]
fn feature_extractor_null_check_in_while_condition() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::While {
            condition: HirExpression::IsNotNull(Box::new(HirExpression::Variable(
                "ptr".to_string(),
            ))),
            body: vec![],
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().null_checks, 1, "While condition null check should count");
}

#[test]
fn feature_extractor_null_check_nested_in_while_body() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::While {
            condition: HirExpression::IntLiteral(1),
            body: vec![HirStatement::If {
                condition: HirExpression::IsNotNull(Box::new(HirExpression::Variable(
                    "ptr".to_string(),
                ))),
                then_block: vec![],
                else_block: None,
            }],
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().null_checks, 1, "Nested null check in while body should count");
}

#[test]
fn feature_extractor_null_check_in_if_else() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::If {
            condition: HirExpression::IntLiteral(1),
            then_block: vec![],
            else_block: Some(vec![HirStatement::If {
                condition: HirExpression::IsNotNull(Box::new(HirExpression::Variable(
                    "ptr".to_string(),
                ))),
                then_block: vec![],
                else_block: None,
            }]),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().null_checks, 1, "Null check in else block should count");
}

// ============================================================================
// is_null_check: BinaryOp with NullLiteral on left and right
// ============================================================================

#[test]
fn feature_extractor_null_check_binary_op_null_right() {
    // ptr != NULL
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::NotEqual,
                left: Box::new(HirExpression::Variable("ptr".to_string())),
                right: Box::new(HirExpression::NullLiteral),
            },
            then_block: vec![],
            else_block: None,
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().null_checks, 1, "ptr != NULL should count as null check");
}

#[test]
fn feature_extractor_null_check_binary_op_null_left() {
    // NULL == ptr
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::Equal,
                left: Box::new(HirExpression::NullLiteral),
                right: Box::new(HirExpression::Variable("ptr".to_string())),
            },
            then_block: vec![],
            else_block: None,
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().null_checks, 1, "NULL == ptr should count as null check");
}

#[test]
fn feature_extractor_null_check_binary_op_no_null() {
    // ptr != other (not a null check)
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::NotEqual,
                left: Box::new(HirExpression::Variable("ptr".to_string())),
                right: Box::new(HirExpression::Variable("other".to_string())),
            },
            then_block: vec![],
            else_block: None,
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().null_checks, 0, "Non-null comparison should not count");
}

// ============================================================================
// expr_uses_var: various expression types
// ============================================================================

#[test]
fn feature_extractor_expr_uses_var_dereference() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::Dereference(Box::new(HirExpression::Variable(
                "ptr".to_string(),
            ))),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert!(features.unwrap().read_count >= 1, "Dereference of var should count as read");
}

#[test]
fn feature_extractor_expr_uses_var_address_of() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::AddressOf(Box::new(HirExpression::Variable(
                "ptr".to_string(),
            ))),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert!(features.unwrap().read_count >= 1, "AddressOf var should count as read");
}

#[test]
fn feature_extractor_expr_uses_var_unary_op() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::UnaryOp {
                op: UnaryOperator::Minus,
                operand: Box::new(HirExpression::Variable("ptr".to_string())),
            },
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert!(features.unwrap().read_count >= 1, "UnaryOp using var should count as read");
}

#[test]
fn feature_extractor_expr_uses_var_array_index() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::ArrayIndex {
                array: Box::new(HirExpression::Variable("ptr".to_string())),
                index: Box::new(HirExpression::IntLiteral(0)),
            },
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert!(features.unwrap().read_count >= 1, "ArrayIndex array using var should count as read");
}

#[test]
fn feature_extractor_expr_uses_var_array_index_in_index() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("idx", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::ArrayIndex {
                array: Box::new(HirExpression::Variable("arr".to_string())),
                index: Box::new(HirExpression::Variable("idx".to_string())),
            },
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "idx");
    assert!(features.is_some());
    assert!(features.unwrap().read_count >= 1, "ArrayIndex index using var should count as read");
}

#[test]
fn feature_extractor_expr_uses_var_function_call_arg() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::FunctionCall {
                function: "process".to_string(),
                arguments: vec![
                    HirExpression::IntLiteral(1),
                    HirExpression::Variable("ptr".to_string()),
                ],
            },
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert!(features.unwrap().read_count >= 1, "FunctionCall arg using var should count as read");
}

#[test]
fn feature_extractor_expr_uses_var_binary_op_left() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("ptr".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert!(features.unwrap().read_count >= 1, "BinaryOp left using var should count as read");
}

#[test]
fn feature_extractor_expr_uses_var_binary_op_right() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::IntLiteral(1)),
                right: Box::new(HirExpression::Variable("ptr".to_string())),
            },
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert!(features.unwrap().read_count >= 1, "BinaryOp right using var should count as read");
}

#[test]
fn feature_extractor_expr_uses_var_not_matching() {
    // Expression types that do not use the variable (IntLiteral, etc.)
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::IntLiteral(42),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert_eq!(features.unwrap().read_count, 0, "IntLiteral should not count as var usage");
}

#[test]
fn feature_extractor_expr_uses_var_is_not_null() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![make_param("ptr", HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::IsNotNull(Box::new(HirExpression::Variable(
                "ptr".to_string(),
            ))),
        }],
        HirType::Void,
    );

    let features = extractor.extract_for_parameter(&func, "ptr");
    assert!(features.is_some());
    assert!(features.unwrap().read_count >= 1, "IsNotNull using var should count as read");
}

// ============================================================================
// OwnershipDefect: Display for all variants
// ============================================================================

#[test]
fn ownership_defect_display_all_variants() {
    let variants = [
        OwnershipDefect::PointerMisclassification,
        OwnershipDefect::LifetimeInferenceGap,
        OwnershipDefect::DanglingPointerRisk,
        OwnershipDefect::AliasViolation,
        OwnershipDefect::UnsafeMinimizationFailure,
        OwnershipDefect::ArraySliceMismatch,
        OwnershipDefect::ResourceLeakPattern,
        OwnershipDefect::MutabilityMismatch,
    ];
    for defect in &variants {
        let display = format!("{}", defect);
        assert!(display.contains(defect.code()), "Display should contain code for {:?}", defect);
        assert!(
            display.contains(defect.description()),
            "Display should contain description for {:?}",
            defect
        );
    }
}

// ============================================================================
// OwnershipDefect: description for all variants
// ============================================================================

#[test]
fn ownership_defect_description_all_variants() {
    assert!(!OwnershipDefect::LifetimeInferenceGap.description().is_empty());
    assert!(!OwnershipDefect::DanglingPointerRisk.description().is_empty());
    assert!(!OwnershipDefect::AliasViolation.description().is_empty());
    assert!(!OwnershipDefect::UnsafeMinimizationFailure.description().is_empty());
    assert!(!OwnershipDefect::ArraySliceMismatch.description().is_empty());
    assert!(!OwnershipDefect::ResourceLeakPattern.description().is_empty());
    assert!(!OwnershipDefect::MutabilityMismatch.description().is_empty());
}

// ============================================================================
// OwnershipDefect: severity for all variants
// ============================================================================

#[test]
fn ownership_defect_severity_all_variants() {
    assert_eq!(OwnershipDefect::DanglingPointerRisk.severity(), Severity::Critical);
    assert_eq!(OwnershipDefect::AliasViolation.severity(), Severity::Critical);
    assert_eq!(OwnershipDefect::PointerMisclassification.severity(), Severity::High);
    assert_eq!(OwnershipDefect::LifetimeInferenceGap.severity(), Severity::High);
    assert_eq!(OwnershipDefect::MutabilityMismatch.severity(), Severity::High);
    assert_eq!(OwnershipDefect::UnsafeMinimizationFailure.severity(), Severity::Medium);
    assert_eq!(OwnershipDefect::ArraySliceMismatch.severity(), Severity::Medium);
    assert_eq!(OwnershipDefect::ResourceLeakPattern.severity(), Severity::Medium);
}

// ============================================================================
// OwnershipDefect: from_code for all variants
// ============================================================================

#[test]
fn ownership_defect_from_code_all_variants() {
    assert_eq!(OwnershipDefect::from_code("DECY-O-001"), Some(OwnershipDefect::PointerMisclassification));
    assert_eq!(OwnershipDefect::from_code("DECY-O-002"), Some(OwnershipDefect::LifetimeInferenceGap));
    assert_eq!(OwnershipDefect::from_code("DECY-O-003"), Some(OwnershipDefect::DanglingPointerRisk));
    assert_eq!(OwnershipDefect::from_code("DECY-O-004"), Some(OwnershipDefect::AliasViolation));
    assert_eq!(OwnershipDefect::from_code("DECY-O-005"), Some(OwnershipDefect::UnsafeMinimizationFailure));
    assert_eq!(OwnershipDefect::from_code("DECY-O-006"), Some(OwnershipDefect::ArraySliceMismatch));
    assert_eq!(OwnershipDefect::from_code("DECY-O-007"), Some(OwnershipDefect::ResourceLeakPattern));
    assert_eq!(OwnershipDefect::from_code("DECY-O-008"), Some(OwnershipDefect::MutabilityMismatch));
    assert_eq!(OwnershipDefect::from_code("DECY-O-009"), None);
    assert_eq!(OwnershipDefect::from_code(""), None);
}

// ============================================================================
// OwnershipErrorKind: to_defect for ALL variants
// ============================================================================

#[test]
fn ownership_error_kind_to_defect_all_variants() {
    assert_eq!(
        OwnershipErrorKind::PointerMisclassification.to_defect(),
        OwnershipDefect::PointerMisclassification
    );
    assert_eq!(
        OwnershipErrorKind::LifetimeInferenceGap.to_defect(),
        OwnershipDefect::LifetimeInferenceGap
    );
    assert_eq!(
        OwnershipErrorKind::DanglingPointerRisk.to_defect(),
        OwnershipDefect::DanglingPointerRisk
    );
    assert_eq!(
        OwnershipErrorKind::AliasViolation.to_defect(),
        OwnershipDefect::AliasViolation
    );
    assert_eq!(
        OwnershipErrorKind::UnsafeMinimizationFailure.to_defect(),
        OwnershipDefect::UnsafeMinimizationFailure
    );
    assert_eq!(
        OwnershipErrorKind::ArraySliceMismatch.to_defect(),
        OwnershipDefect::ArraySliceMismatch
    );
    assert_eq!(
        OwnershipErrorKind::ResourceLeakPattern.to_defect(),
        OwnershipDefect::ResourceLeakPattern
    );
    assert_eq!(
        OwnershipErrorKind::MutabilityMismatch.to_defect(),
        OwnershipDefect::MutabilityMismatch
    );
}

// ============================================================================
// OwnershipPrediction: PartialEq edge cases
// ============================================================================

#[test]
fn ownership_prediction_partial_eq_different_kind() {
    let p1 = OwnershipPrediction {
        kind: InferredOwnership::Owned,
        confidence: 0.8,
        fallback: None,
    };
    let p2 = OwnershipPrediction {
        kind: InferredOwnership::Borrowed,
        confidence: 0.8,
        fallback: None,
    };
    assert_ne!(p1, p2, "Different kinds should not be equal");
}

#[test]
fn ownership_prediction_partial_eq_different_confidence() {
    let p1 = OwnershipPrediction {
        kind: InferredOwnership::Owned,
        confidence: 0.8,
        fallback: None,
    };
    let p2 = OwnershipPrediction {
        kind: InferredOwnership::Owned,
        confidence: 0.5,
        fallback: None,
    };
    assert_ne!(p1, p2, "Different confidences should not be equal");
}

#[test]
fn ownership_prediction_at_threshold_boundary() {
    // Exactly at threshold
    let at_threshold = OwnershipPrediction {
        kind: InferredOwnership::Owned,
        confidence: OwnershipPrediction::CONFIDENCE_THRESHOLD,
        fallback: Some(InferredOwnership::Borrowed),
    };
    assert!(at_threshold.is_confident(), "At threshold should be confident");
    assert_eq!(at_threshold.effective_ownership(), InferredOwnership::Owned);
}

#[test]
fn ownership_prediction_just_below_threshold() {
    let below = OwnershipPrediction {
        kind: InferredOwnership::Owned,
        confidence: OwnershipPrediction::CONFIDENCE_THRESHOLD - 0.01,
        fallback: Some(InferredOwnership::Borrowed),
    };
    assert!(!below.is_confident(), "Below threshold should not be confident");
    assert_eq!(below.effective_ownership(), InferredOwnership::Borrowed);
}

// ============================================================================
// InferredOwnership: requires_unsafe for all variants
// ============================================================================

#[test]
fn inferred_ownership_requires_unsafe_all_variants() {
    assert!(!InferredOwnership::Owned.requires_unsafe());
    assert!(!InferredOwnership::Borrowed.requires_unsafe());
    assert!(!InferredOwnership::BorrowedMut.requires_unsafe());
    assert!(!InferredOwnership::Shared.requires_unsafe());
    assert!(InferredOwnership::RawPointer.requires_unsafe());
    assert!(!InferredOwnership::Vec.requires_unsafe());
    assert!(!InferredOwnership::Slice.requires_unsafe());
    assert!(!InferredOwnership::SliceMut.requires_unsafe());
}

// ============================================================================
// ErrorPattern: occurrence tracking
// ============================================================================

#[test]
fn error_pattern_record_occurrence() {
    let mut pattern = ErrorPattern::new(
        "test",
        OwnershipErrorKind::PointerMisclassification,
        "test pattern",
    );
    assert_eq!(pattern.occurrence_count(), 0);

    pattern.record_occurrence();
    assert_eq!(pattern.occurrence_count(), 1);

    pattern.record_occurrence();
    pattern.record_occurrence();
    assert_eq!(pattern.occurrence_count(), 3);
}

#[test]
fn error_pattern_description_accessor() {
    let pattern = ErrorPattern::new(
        "test-id",
        OwnershipErrorKind::AliasViolation,
        "Multiple mutable aliases detected",
    );
    assert_eq!(pattern.description(), "Multiple mutable aliases detected");
}

// ============================================================================
// PatternLibrary: get_mut
// ============================================================================

#[test]
fn pattern_library_get_mut_existing() {
    let mut library = PatternLibrary::new();
    library.add(ErrorPattern::new(
        "mutable_test",
        OwnershipErrorKind::ResourceLeakPattern,
        "test",
    ));

    let pattern = library.get_mut("mutable_test");
    assert!(pattern.is_some());
    let pattern = pattern.unwrap();
    pattern.record_occurrence();
    assert_eq!(pattern.occurrence_count(), 1);
}

#[test]
fn pattern_library_get_mut_missing() {
    let mut library = PatternLibrary::new();
    assert!(library.get_mut("nonexistent").is_none());
}

// ============================================================================
// PatternLibrary: get missing
// ============================================================================

#[test]
fn pattern_library_get_missing() {
    let library = PatternLibrary::new();
    assert!(library.get("nonexistent").is_none());
}

// ============================================================================
// PatternLibrary: match_rust_error with no matches
// ============================================================================

#[test]
fn pattern_library_match_rust_error_no_match() {
    let mut library = PatternLibrary::new();
    library.add(
        ErrorPattern::new("test", OwnershipErrorKind::LifetimeInferenceGap, "test")
            .with_rust_error("E0106"),
    );

    let matches = library.match_rust_error("E9999: totally unknown error");
    assert!(matches.is_empty(), "No patterns should match unknown error");
}

#[test]
fn pattern_library_match_rust_error_pattern_without_rust_error() {
    // Patterns without rust_error set should not match
    let mut library = PatternLibrary::new();
    library.add(ErrorPattern::new(
        "no_rust_err",
        OwnershipErrorKind::ResourceLeakPattern,
        "test",
    ));

    let matches = library.match_rust_error("E0308");
    assert!(matches.is_empty(), "Patterns without rust_error should not match");
}

// ============================================================================
// PatternLibrary: iter
// ============================================================================

#[test]
fn pattern_library_iter() {
    let mut library = PatternLibrary::new();
    library.add(ErrorPattern::new(
        "p1",
        OwnershipErrorKind::AliasViolation,
        "alias",
    ));
    library.add(ErrorPattern::new(
        "p2",
        OwnershipErrorKind::DanglingPointerRisk,
        "dangling",
    ));

    let count = library.iter().count();
    assert_eq!(count, 2);
}

// ============================================================================
// PatternLibrary: get_by_error_kind no matches
// ============================================================================

#[test]
fn pattern_library_get_by_error_kind_empty() {
    let library = PatternLibrary::new();
    let results = library.get_by_error_kind(OwnershipErrorKind::AliasViolation);
    assert!(results.is_empty());
}

// ============================================================================
// ErrorSeverity: ordering
// ============================================================================

#[test]
fn error_severity_ordering() {
    assert!(ErrorSeverity::Critical > ErrorSeverity::Error);
    assert!(ErrorSeverity::Error > ErrorSeverity::Warning);
    assert!(ErrorSeverity::Warning > ErrorSeverity::Info);
}

// ============================================================================
// ErrorSeverity: Debug/Clone
// ============================================================================

#[test]
fn error_severity_debug_clone() {
    let sev = ErrorSeverity::Critical;
    let cloned = sev;
    assert_eq!(sev, cloned);
    let debug = format!("{:?}", sev);
    assert!(debug.contains("Critical"));
}

// ============================================================================
// Severity: Debug/Clone/Serialize
// ============================================================================

#[test]
fn severity_serialize_deserialize() {
    let sev = Severity::High;
    let json = serde_json::to_string(&sev).unwrap();
    let parsed: Severity = serde_json::from_str(&json).unwrap();
    assert_eq!(sev, parsed);
}

#[test]
fn severity_debug_clone() {
    let sev = Severity::Info;
    let cloned = sev;
    assert_eq!(sev, cloned);
    let debug = format!("{:?}", sev);
    assert!(debug.contains("Info"));
}

// ============================================================================
// AllocationKind: Serialize/Deserialize
// ============================================================================

#[test]
fn allocation_kind_serialize_deserialize() {
    let kinds = [
        AllocationKind::Malloc,
        AllocationKind::Calloc,
        AllocationKind::Realloc,
        AllocationKind::Stack,
        AllocationKind::Static,
        AllocationKind::Parameter,
        AllocationKind::Unknown,
    ];
    for kind in &kinds {
        let json = serde_json::to_string(kind).unwrap();
        let parsed: AllocationKind = serde_json::from_str(&json).unwrap();
        assert_eq!(*kind, parsed, "AllocationKind {:?} should round-trip through JSON", kind);
    }
}

// ============================================================================
// InferredOwnership: Serialize/Deserialize
// ============================================================================

#[test]
fn inferred_ownership_serialize_deserialize() {
    let kinds = [
        InferredOwnership::Owned,
        InferredOwnership::Borrowed,
        InferredOwnership::BorrowedMut,
        InferredOwnership::Shared,
        InferredOwnership::RawPointer,
        InferredOwnership::Vec,
        InferredOwnership::Slice,
        InferredOwnership::SliceMut,
    ];
    for kind in &kinds {
        let json = serde_json::to_string(kind).unwrap();
        let parsed: InferredOwnership = serde_json::from_str(&json).unwrap();
        assert_eq!(*kind, parsed, "InferredOwnership {:?} should round-trip through JSON", kind);
    }
}

// ============================================================================
// OwnershipErrorKind: Serialize/Deserialize
// ============================================================================

#[test]
fn ownership_error_kind_serialize_deserialize() {
    let kinds = [
        OwnershipErrorKind::PointerMisclassification,
        OwnershipErrorKind::LifetimeInferenceGap,
        OwnershipErrorKind::DanglingPointerRisk,
        OwnershipErrorKind::AliasViolation,
        OwnershipErrorKind::UnsafeMinimizationFailure,
        OwnershipErrorKind::ArraySliceMismatch,
        OwnershipErrorKind::ResourceLeakPattern,
        OwnershipErrorKind::MutabilityMismatch,
    ];
    for kind in &kinds {
        let json = serde_json::to_string(kind).unwrap();
        let parsed: OwnershipErrorKind = serde_json::from_str(&json).unwrap();
        assert_eq!(*kind, parsed, "OwnershipErrorKind {:?} should round-trip through JSON", kind);
    }
}

// ============================================================================
// SuggestedFix: default confidence
// ============================================================================

#[test]
fn suggested_fix_default_confidence() {
    let fix = SuggestedFix::new("desc", "template");
    assert!((fix.confidence() - 0.5).abs() < f32::EPSILON, "Default confidence should be 0.5");
}

// ============================================================================
// ErrorPattern: builder methods chaining
// ============================================================================

#[test]
fn error_pattern_full_builder_chain() {
    let fix = SuggestedFix::new("Use Box", "Box::new(x)").with_confidence(0.9);
    let pattern = ErrorPattern::new(
        "full-test",
        OwnershipErrorKind::UnsafeMinimizationFailure,
        "Unsafe block can be eliminated",
    )
    .with_c_pattern("*(ptr + i) = val;")
    .with_rust_error("E0133")
    .with_fix(fix)
    .with_severity(ErrorSeverity::Warning)
    .with_curriculum_level(3);

    assert_eq!(pattern.id(), "full-test");
    assert_eq!(pattern.error_kind(), OwnershipErrorKind::UnsafeMinimizationFailure);
    assert_eq!(pattern.description(), "Unsafe block can be eliminated");
    assert_eq!(pattern.c_pattern(), Some("*(ptr + i) = val;"));
    assert_eq!(pattern.rust_error(), Some("E0133"));
    assert!(pattern.suggested_fix().is_some());
    assert!((pattern.suggested_fix().unwrap().confidence() - 0.9).abs() < f32::EPSILON);
    assert_eq!(pattern.severity(), ErrorSeverity::Warning);
    assert_eq!(pattern.curriculum_level(), 3);
    assert_eq!(pattern.occurrence_count(), 0);
}

// ============================================================================
// default_pattern_library: specific pattern existence checks
// ============================================================================

#[test]
fn default_library_has_minimum_patterns() {
    let library = default_pattern_library();
    // The source code adds at least 16 patterns
    assert!(library.len() >= 16, "Default library should have at least 16 patterns, got {}", library.len());
}

#[test]
fn default_library_patterns_have_descriptions() {
    let library = default_pattern_library();
    for pattern in library.iter() {
        assert!(!pattern.description().is_empty(), "Pattern {} should have description", pattern.id());
    }
}

#[test]
fn default_library_patterns_have_valid_ids() {
    let library = default_pattern_library();
    for pattern in library.iter() {
        assert!(!pattern.id().is_empty(), "Pattern should have non-empty id");
    }
}

#[test]
fn default_library_level_5_patterns_exist() {
    let library = default_pattern_library();
    let ordered = library.curriculum_ordered();
    let max_level = ordered.iter().map(|p| p.curriculum_level()).max().unwrap_or(0);
    assert!(max_level >= 5, "Should have expert-level (5) patterns");
}

#[test]
fn default_library_self_referential_pattern() {
    let library = default_pattern_library();
    let alias_patterns = library.get_by_error_kind(OwnershipErrorKind::AliasViolation);
    let has_self_ref = alias_patterns.iter().any(|p| {
        p.description().to_lowercase().contains("self-referential")
            || p.id().contains("self-referential")
    });
    assert!(has_self_ref, "Should have self-referential struct pattern");
}

#[test]
fn default_library_interior_mutability_pattern() {
    let library = default_pattern_library();
    let mut_patterns = library.get_by_error_kind(OwnershipErrorKind::MutabilityMismatch);
    let has_interior = mut_patterns.iter().any(|p| {
        p.description().to_lowercase().contains("interior")
            || p.id().contains("interior")
    });
    assert!(has_interior, "Should have interior mutability pattern");
}

// ============================================================================
// extract_all: mixed parameters
// ============================================================================

#[test]
fn feature_extractor_extract_all_no_pointers() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![
            make_param("x", HirType::Int),
            make_param("y", HirType::Float),
        ],
        vec![],
        HirType::Void,
    );

    let all = extractor.extract_all(&func);
    assert!(all.is_empty(), "No pointer params means empty result");
}

#[test]
fn feature_extractor_extract_all_with_body_accesses() {
    let extractor = FeatureExtractor::new();
    let func = make_function(
        "test",
        vec![
            make_param("data", HirType::Pointer(Box::new(HirType::Int))),
            make_param("len", HirType::Int),
        ],
        vec![
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::Dereference(Box::new(HirExpression::Variable(
                    "data".to_string(),
                ))),
            },
            HirStatement::Free {
                pointer: HirExpression::Variable("data".to_string()),
            },
        ],
        HirType::Void,
    );

    let all = extractor.extract_all(&func);
    assert_eq!(all.len(), 1);
    let (name, features) = &all[0];
    assert_eq!(name, "data");
    assert!(features.read_count >= 1);
    assert_eq!(features.deallocation_count, 1);
    assert!(features.is_array_decay, "data + len should trigger array decay");
}

// ============================================================================
// FeatureExtractor: Default trait
// ============================================================================

#[test]
fn feature_extractor_default() {
    let extractor = FeatureExtractor::default();
    assert_eq!(extractor.extracted_count(), 0);
}

// ============================================================================
// PatternLibrary: add replaces existing
// ============================================================================

#[test]
fn pattern_library_add_replaces_existing_id() {
    let mut library = PatternLibrary::new();
    library.add(ErrorPattern::new(
        "dup",
        OwnershipErrorKind::AliasViolation,
        "first version",
    ));
    library.add(ErrorPattern::new(
        "dup",
        OwnershipErrorKind::DanglingPointerRisk,
        "second version",
    ));

    assert_eq!(library.len(), 1, "Same ID should replace, not duplicate");
    let pattern = library.get("dup").unwrap();
    assert_eq!(pattern.error_kind(), OwnershipErrorKind::DanglingPointerRisk);
    assert_eq!(pattern.description(), "second version");
}
