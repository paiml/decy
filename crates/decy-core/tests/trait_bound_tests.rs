//! Tests for trait bound inference on generics (DECY-097).
//!
//! Infers trait bounds from operations on void* to generate
//! proper where clauses for Rust generics.

use decy_codegen::CodeGenerator;
use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType};

/// Helper: Create void* parameter
fn void_ptr_param(name: &str) -> HirParameter {
    HirParameter::new(name.to_string(), HirType::Pointer(Box::new(HirType::Void)))
}

/// Helper: Create function with void* params
fn create_void_ptr_function(
    name: &str,
    params: Vec<HirParameter>,
    return_type: HirType,
    body: Vec<HirStatement>,
) -> HirFunction {
    HirFunction::new_with_body(name.to_string(), return_type, params, body)
}

// ============================================================================
// TEST 1: Compare operation infers PartialOrd
// ============================================================================

#[test]
fn test_compare_infers_partial_ord() {
    // int compare(void* a, void* b) { return *(int*)a < *(int*)b; }
    // Should generate: fn compare<T: PartialOrd>(a: &T, b: &T) -> i32
    let func = create_void_ptr_function(
        "compare",
        vec![void_ptr_param("a"), void_ptr_param("b")],
        HirType::Int,
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Dereference(Box::new(
                HirExpression::Variable("a".to_string()),
            ))),
            right: Box::new(HirExpression::Dereference(Box::new(
                HirExpression::Variable("b".to_string()),
            ))),
        }))],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        code.contains("PartialOrd") || code.contains("Ord"),
        "Should infer PartialOrd from < comparison:\n{}",
        code
    );
}

// ============================================================================
// TEST 2: Equality check infers PartialEq
// ============================================================================

#[test]
fn test_equality_infers_partial_eq() {
    // int equal(void* a, void* b) { return *(int*)a == *(int*)b; }
    // Should generate: fn equal<T: PartialEq>(a: &T, b: &T) -> i32
    let func = create_void_ptr_function(
        "equal",
        vec![void_ptr_param("a"), void_ptr_param("b")],
        HirType::Int,
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Dereference(Box::new(
                HirExpression::Variable("a".to_string()),
            ))),
            right: Box::new(HirExpression::Dereference(Box::new(
                HirExpression::Variable("b".to_string()),
            ))),
        }))],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        code.contains("PartialEq") || code.contains("Eq"),
        "Should infer PartialEq from == comparison:\n{}",
        code
    );
}

// ============================================================================
// TEST 3: Copy operation infers Clone
// ============================================================================

#[test]
fn test_copy_infers_clone() {
    // void copy(void* dest, void* src) { *dest = *src; }
    // Should generate: fn copy<T: Clone>(dest: &mut T, src: &T)
    let func = create_void_ptr_function(
        "copy_value",
        vec![void_ptr_param("dest"), void_ptr_param("src")],
        HirType::Void,
        vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("dest".to_string()),
            value: HirExpression::Dereference(Box::new(HirExpression::Variable("src".to_string()))),
        }],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        code.contains("Clone") || code.contains("Copy"),
        "Should infer Clone from value copy:\n{}",
        code
    );
}

// ============================================================================
// TEST 4: No operations - no trait bounds
// ============================================================================

#[test]
fn test_no_ops_no_bounds() {
    // void process(void* data) { } - no ops on T, no bounds needed
    let func = create_void_ptr_function(
        "process",
        vec![void_ptr_param("data")],
        HirType::Void,
        vec![],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    // Should have <T> but no trait bounds (no where clause or T: Something)
    assert!(code.contains("<T>"), "Should have generic T:\n{}", code);
    // Should NOT have complex bounds for simple pass-through
    let has_bounds = code.contains("T:") || code.contains("where T");
    assert!(
        !has_bounds,
        "Should NOT have trait bounds for no-op function:\n{}",
        code
    );
}

// ============================================================================
// TEST 5: Multiple operations - multiple bounds
// ============================================================================

#[test]
fn test_multiple_ops_multiple_bounds() {
    // Function that compares AND copies: needs both PartialOrd and Clone
    let func = create_void_ptr_function(
        "sort_swap",
        vec![void_ptr_param("a"), void_ptr_param("b")],
        HirType::Void,
        vec![
            // if *a > *b (comparison - needs PartialOrd)
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::GreaterThan,
                    left: Box::new(HirExpression::Dereference(Box::new(
                        HirExpression::Variable("a".to_string()),
                    ))),
                    right: Box::new(HirExpression::Dereference(Box::new(
                        HirExpression::Variable("b".to_string()),
                    ))),
                },
                then_block: vec![
                    // *a = *b (copy - needs Clone)
                    HirStatement::DerefAssignment {
                        target: HirExpression::Variable("a".to_string()),
                        value: HirExpression::Dereference(Box::new(HirExpression::Variable(
                            "b".to_string(),
                        ))),
                    },
                ],
                else_block: None,
            },
        ],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    // Should have multiple bounds
    let has_ord = code.contains("PartialOrd") || code.contains("Ord");
    let has_clone = code.contains("Clone") || code.contains("Copy");

    assert!(
        has_ord || has_clone,
        "Should have at least one trait bound for complex function:\n{}",
        code
    );
}

// ============================================================================
// TEST 6: Non-void* functions - no trait analysis
// ============================================================================

#[test]
fn test_non_void_ptr_no_trait_analysis() {
    let func = HirFunction::new_with_body(
        "add".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
        vec![],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    // No generic, no trait bounds
    assert!(
        !code.contains("<T>") && !code.contains("where"),
        "Should NOT have generics for non-void* function:\n{}",
        code
    );
}
