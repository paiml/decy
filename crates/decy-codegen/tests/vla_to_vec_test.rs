//! Tests for VLA (Variable-Length Array) → Vec transformation (TYPE-ARRAY-VLA validation)
//!
//! Reference: ISO C99 §6.7.5.2 (VLA - variable length arrays)
//!
//! This module tests the transformation of C99 variable-length arrays to Rust's Vec.
//! VLAs in C are dangerous (stack overflow, no bounds checking) and should become
//! heap-allocated Vec in Rust for safety.

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirStatement, HirType};

/// Test simple VLA with variable size → Vec
///
/// C: int n = 10;
///    int arr[n];
///
/// Rust: let n = 10;
///       let arr = vec![0i32; n];
///
/// Reference: ISO C99 §6.7.5.2
#[test]
fn test_simple_vla_to_vec() {
    // RED PHASE: This test should FAIL until we implement VLA → Vec transformation
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "n".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(10)),
            },
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: None, // VLA: size determined at runtime
                },
                initializer: Some(HirExpression::Variable("n".to_string())),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Vec allocation with runtime size
    assert!(result.contains("let mut n: i32 = 10"));
    assert!(result.contains("let mut arr = vec![0i32; n]"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test VLA with expression as size → Vec
///
/// C: int n = 5;
///    int arr[n * 2];
///
/// Rust: let n = 5;
///       let arr = vec![0i32; n * 2];
#[test]
fn test_vla_with_expression_size() {
    // RED PHASE: This test should FAIL until we implement VLA handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "n".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(5)),
            },
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: None,
                },
                initializer: Some(HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Multiply,
                    left: Box::new(HirExpression::Variable("n".to_string())),
                    right: Box::new(HirExpression::IntLiteral(2)),
                }),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Vec with expression
    assert!(result.contains("let mut arr = vec![0i32; n * 2]"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test VLA with function parameter as size
///
/// C: void allocate_array(int size) {
///      int arr[size];
///    }
///
/// Rust: fn allocate_array(mut size: i32) {
///         let arr = vec![0i32; size];
///       }
#[test]
fn test_vla_with_parameter_size() {
    // RED PHASE: This test should FAIL until we implement VLA handling
    let func = HirFunction::new_with_body(
        "allocate_array".to_string(),
        HirType::Void,
        vec![decy_hir::HirParameter::new(
            "size".to_string(),
            HirType::Int,
        )],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Int),
                size: None,
            },
            initializer: Some(HirExpression::Variable("size".to_string())),
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Vec with parameter
    assert!(result.contains("fn allocate_array(mut size: i32)"));
    assert!(result.contains("let mut arr = vec![0i32; size]"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test VLA with different types (float)
///
/// C: int n = 5;
///    float arr[n];
///
/// Rust: let n = 5;
///       let arr = vec![0.0f32; n];
#[test]
fn test_vla_float_array() {
    // RED PHASE: This test should FAIL until we implement VLA handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "n".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(5)),
            },
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Array {
                    element_type: Box::new(HirType::Float),
                    size: None,
                },
                initializer: Some(HirExpression::Variable("n".to_string())),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Vec<f32>
    assert!(result.contains("let mut arr = vec![0.0f32; n]"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test VLA with usage (array indexing)
///
/// C: int n = 10;
///    int arr[n];
///    arr[0] = 42;
///
/// Rust: let n = 10;
///       let mut arr = vec![0i32; n];
///       arr[0] = 42;
#[test]
fn test_vla_with_usage() {
    // RED PHASE: This test should FAIL until we implement VLA handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "n".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(10)),
            },
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: None,
                },
                initializer: Some(HirExpression::Variable("n".to_string())),
            },
            HirStatement::ArrayIndexAssignment {
                array: Box::new(HirExpression::Variable("arr".to_string())),
                index: Box::new(HirExpression::IntLiteral(0)),
                value: HirExpression::IntLiteral(42),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Vec with assignment
    assert!(result.contains("let mut arr = vec![0i32; n]"));
    // DECY-150: Accept either arr[0], arr[0 as usize], or arr[(0) as usize]
    assert!(
        (result.contains("arr[0") || result.contains("arr[(0)")) && result.contains("= 42"),
        "Expected arr[...] = 42 in output, got: {}",
        result
    );

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test that fixed-size arrays are NOT transformed to Vec
///
/// C: int arr[10];  // Fixed size, NOT a VLA
///
/// Rust: let arr: [i32; 10] = [0; 10];  // Should remain fixed-size
#[test]
fn test_fixed_size_array_not_vec() {
    // This test verifies we don't break existing fixed-size array handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(10), // Fixed size
            },
            initializer: None,
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Fixed-size array (NOT Vec)
    assert!(result.contains("[i32; 10]"));
    assert!(!result.contains("vec!"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Verify unsafe block count remains 0
///
/// This is critical for the validation goal: <5 unsafe blocks per 1000 LOC
#[test]
fn test_vla_transformation_unsafe_count() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "n".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(100)),
            },
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: None,
                },
                initializer: Some(HirExpression::Variable("n".to_string())),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Count unsafe blocks (should be 0)
    let unsafe_count = result.matches("unsafe").count();
    assert_eq!(
        unsafe_count, 0,
        "VLA → Vec transformation should not introduce unsafe blocks"
    );
}
