//! Tests for calloc → Vec transformation (STDLIB-CALLOC validation)
//!
//! Reference: ISO C99 §7.20.3.1
//!
//! This module tests the transformation of C calloc() to Rust's Vec.
//! calloc() in C allocates zero-initialized memory, which is unsafe.
//! Rust's vec![0; n] is safe and provides the same zero-initialization.

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirStatement, HirType};

/// Test calloc(n, sizeof(int)) → vec![0i32; n]
///
/// C: int* arr = calloc(n, sizeof(int));
/// Rust: let arr = vec![0i32; n];
///
/// Reference: ISO C99 §7.20.3.1
#[test]
fn test_calloc_int_array() {
    // RED PHASE: This test should FAIL until we implement calloc handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Vec(Box::new(HirType::Int)),
            initializer: Some(HirExpression::Calloc {
                count: Box::new(HirExpression::IntLiteral(10)),
                element_type: Box::new(HirType::Int),
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected Rust output
    assert!(result.contains("let mut arr: Vec<i32> = vec![0i32; 10]"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test calloc with variable count → vec![0; n]
///
/// C: int* arr = calloc(n, sizeof(int));
/// Rust: let arr = vec![0i32; n];
///
/// Reference: ISO C99 §7.20.3.1
#[test]
fn test_calloc_variable_count() {
    // RED PHASE: This test should FAIL until we implement calloc handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "n".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(20)),
            },
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Vec(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Calloc {
                    count: Box::new(HirExpression::Variable("n".to_string())),
                    element_type: Box::new(HirType::Int),
                }),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected Rust output
    assert!(result.contains("let mut arr: Vec<i32> = vec![0i32; n]"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test calloc for different types (float, double, char)
///
/// C: float* arr = calloc(5, sizeof(float));
/// Rust: let arr = vec![0.0f32; 5];
///
/// Reference: ISO C99 §7.20.3.1
#[test]
fn test_calloc_float_array() {
    // RED PHASE: This test should FAIL until we implement calloc handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Vec(Box::new(HirType::Float)),
            initializer: Some(HirExpression::Calloc {
                count: Box::new(HirExpression::IntLiteral(5)),
                element_type: Box::new(HirType::Float),
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected Rust output
    assert!(result.contains("let mut arr: Vec<f32> = vec![0.0f32; 5]"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test calloc for char array
///
/// C: char* str = calloc(100, sizeof(char));
/// Rust: let str = vec![0u8; 100];
///
/// Reference: ISO C99 §7.20.3.1
#[test]
fn test_calloc_char_array() {
    // RED PHASE: This test should FAIL until we implement calloc handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "str".to_string(),
            var_type: HirType::Vec(Box::new(HirType::Char)),
            initializer: Some(HirExpression::Calloc {
                count: Box::new(HirExpression::IntLiteral(100)),
                element_type: Box::new(HirType::Char),
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected Rust output
    assert!(result.contains("let mut str: Vec<u8> = vec![0u8; 100]"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test calloc followed by usage (read/write)
///
/// C: int* arr = calloc(n, sizeof(int)); arr[0] = 42;
/// Rust: let mut arr = vec![0i32; n]; arr[0] = 42;
///
/// Reference: ISO C99 §7.20.3.1
#[test]
fn test_calloc_with_usage() {
    // RED PHASE: This test should FAIL until we implement calloc handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Vec(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Calloc {
                    count: Box::new(HirExpression::IntLiteral(10)),
                    element_type: Box::new(HirType::Int),
                }),
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

    // Expected Rust output with mutable declaration
    assert!(result.contains("let mut arr: Vec<i32> = vec![0i32; 10]"));
    // Accept either arr[0] = 42 or arr[0 as usize] = 42 (the latter is more correct Rust)
    assert!(result.contains("arr[0") && result.contains("= 42"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Verify unsafe block count remains 0
///
/// This is critical for the validation goal: <5 unsafe blocks per 1000 LOC
#[test]
fn test_calloc_transformation_unsafe_count() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Vec(Box::new(HirType::Int)),
            initializer: Some(HirExpression::Calloc {
                count: Box::new(HirExpression::IntLiteral(10)),
                element_type: Box::new(HirType::Int),
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Count unsafe blocks (should be 0)
    let unsafe_count = result.matches("unsafe").count();
    assert_eq!(
        unsafe_count, 0,
        "calloc → Vec transformation should not introduce unsafe blocks"
    );
}

/// Test zero-initialization verification
///
/// C: int* arr = calloc(3, sizeof(int)); // All elements are 0
/// Rust: let arr = vec![0i32; 3]; // All elements are 0
///
/// Reference: ISO C99 §7.20.3.1 - "The space is initialized to all bits zero"
#[test]
fn test_calloc_zero_initialization() {
    // RED PHASE: This test should FAIL until we implement calloc handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Vec(Box::new(HirType::Int)),
            initializer: Some(HirExpression::Calloc {
                count: Box::new(HirExpression::IntLiteral(3)),
                element_type: Box::new(HirType::Int),
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: vec![0i32; 3] which guarantees zero-initialization
    assert!(result.contains("vec![0i32; 3]"));

    // Should NOT use Vec::with_capacity (which doesn't zero-initialize)
    assert!(!result.contains("with_capacity"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}
