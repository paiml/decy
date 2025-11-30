//! Tests for realloc() → Vec::resize/reserve transformation (STDLIB-REALLOC validation)
//!
//! Reference: ISO C99 §7.20.3.4
//!
//! This module tests the transformation of C realloc() to Rust's Vec methods.
//! realloc() in C is unsafe and error-prone (can fail, may move memory).
//! Rust's Vec provides safe resizing with automatic reallocation.

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirStatement, HirType};

/// Test realloc(ptr, larger_size) → Vec::resize(new_size, default)
///
/// C: int* arr = malloc(10 * sizeof(int));
///    arr = realloc(arr, 20 * sizeof(int));
/// Rust: let mut arr = vec![0i32; 10];
///       arr.resize(20, 0i32);
///
/// Reference: ISO C99 §7.20.3.4
#[test]
fn test_realloc_expand_to_vec_resize() {
    // RED PHASE: This test should FAIL until we implement realloc handling
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
            HirStatement::Assignment {
                target: "arr".to_string(),
                value: HirExpression::Realloc {
                    pointer: Box::new(HirExpression::Variable("arr".to_string())),
                    new_size: Box::new(HirExpression::BinaryOp {
                        op: decy_hir::BinaryOperator::Multiply,
                        left: Box::new(HirExpression::IntLiteral(20)),
                        right: Box::new(HirExpression::Sizeof {
                            type_name: "int".to_string(),
                        }),
                    }),
                },
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Vec allocation followed by resize
    assert!(result.contains("let mut arr: Vec<i32> = vec![0i32; 10]"));
    assert!(result.contains("arr.resize(20, 0i32)"));

    // Critical: realloc() should NOT appear in generated code
    assert!(!result.contains("realloc"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test realloc(ptr, smaller_size) → Vec::truncate
///
/// C: int* arr = malloc(20 * sizeof(int));
///    arr = realloc(arr, 10 * sizeof(int));
/// Rust: let mut arr = vec![0i32; 20];
///       arr.truncate(10);
///
/// Reference: ISO C99 §7.20.3.4
#[test]
fn test_realloc_shrink_to_vec_truncate() {
    // RED PHASE: This test should FAIL until we implement realloc handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Vec(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Calloc {
                    count: Box::new(HirExpression::IntLiteral(20)),
                    element_type: Box::new(HirType::Int),
                }),
            },
            HirStatement::Assignment {
                target: "arr".to_string(),
                value: HirExpression::Realloc {
                    pointer: Box::new(HirExpression::Variable("arr".to_string())),
                    new_size: Box::new(HirExpression::BinaryOp {
                        op: decy_hir::BinaryOperator::Multiply,
                        left: Box::new(HirExpression::IntLiteral(10)),
                        right: Box::new(HirExpression::Sizeof {
                            type_name: "int".to_string(),
                        }),
                    }),
                },
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Vec allocation followed by truncate
    assert!(result.contains("let mut arr: Vec<i32> = vec![0i32; 20]"));
    // For shrinking, we might use resize with smaller size or truncate
    assert!(result.contains("arr.resize(10, 0i32)") || result.contains("arr.truncate(10)"));

    // Critical: realloc() should NOT appear in generated code
    assert!(!result.contains("realloc"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test realloc(NULL, size) → equivalent to malloc (Vec::with_capacity)
///
/// C: int* arr = realloc(NULL, 10 * sizeof(int));
/// Rust: let mut arr = vec![0i32; 10];
///
/// Reference: ISO C99 §7.20.3.4 - "If ptr is NULL, realloc behaves like malloc"
#[test]
fn test_realloc_null_to_vec() {
    // RED PHASE: This test should FAIL until we implement realloc handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Vec(Box::new(HirType::Int)),
            initializer: Some(HirExpression::Realloc {
                pointer: Box::new(HirExpression::NullLiteral),
                new_size: Box::new(HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Multiply,
                    left: Box::new(HirExpression::IntLiteral(10)),
                    right: Box::new(HirExpression::Sizeof {
                        type_name: "int".to_string(),
                    }),
                }),
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Vec allocation (realloc(NULL, size) → malloc)
    assert!(result.contains("let mut arr: Vec<i32> = vec![0i32; 10]"));

    // Critical: realloc() should NOT appear in generated code
    assert!(!result.contains("realloc"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test realloc(ptr, 0) → free (no code, RAII)
///
/// C: int* arr = malloc(10 * sizeof(int));
///    arr = realloc(arr, 0);
/// Rust: let mut arr = vec![0i32; 10];
///       drop(arr); // Actually implicit, just goes out of scope
///
/// Reference: ISO C99 §7.20.3.4 - "If size is 0, realloc behaves like free"
#[test]
fn test_realloc_zero_size_to_drop() {
    // RED PHASE: This test should FAIL until we implement realloc handling
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
            HirStatement::Assignment {
                target: "arr".to_string(),
                value: HirExpression::Realloc {
                    pointer: Box::new(HirExpression::Variable("arr".to_string())),
                    new_size: Box::new(HirExpression::IntLiteral(0)),
                },
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Vec allocation, then realloc(0) → RAII comment or clear()
    assert!(result.contains("let mut arr: Vec<i32> = vec![0i32; 10]"));
    // realloc(ptr, 0) could be: arr.clear() or just RAII comment
    assert!(
        result.contains("arr.clear()") || result.contains("// Memory deallocated automatically")
    );

    // Critical: realloc() should NOT appear in generated code
    assert!(!result.contains("realloc"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test realloc() with different types
///
/// C: float* arr = malloc(5 * sizeof(float));
///    arr = realloc(arr, 10 * sizeof(float));
/// Rust: let mut arr = vec![0.0f32; 5];
///       arr.resize(10, 0.0f32);
#[test]
fn test_realloc_float_array() {
    // RED PHASE: This test should FAIL until we implement realloc handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Vec(Box::new(HirType::Float)),
                initializer: Some(HirExpression::Calloc {
                    count: Box::new(HirExpression::IntLiteral(5)),
                    element_type: Box::new(HirType::Float),
                }),
            },
            HirStatement::Assignment {
                target: "arr".to_string(),
                value: HirExpression::Realloc {
                    pointer: Box::new(HirExpression::Variable("arr".to_string())),
                    new_size: Box::new(HirExpression::BinaryOp {
                        op: decy_hir::BinaryOperator::Multiply,
                        left: Box::new(HirExpression::IntLiteral(10)),
                        right: Box::new(HirExpression::Sizeof {
                            type_name: "float".to_string(),
                        }),
                    }),
                },
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Vec<f32> allocation and resize
    assert!(result.contains("let mut arr: Vec<f32> = vec![0.0f32; 5]"));
    assert!(result.contains("arr.resize(10, 0.0f32)"));

    // Critical: realloc() should NOT appear
    assert!(!result.contains("realloc"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test realloc() in conditional
///
/// C: if (need_more) { arr = realloc(arr, new_size); }
/// Rust: if need_more { arr.resize(new_count, default); }
#[test]
fn test_realloc_in_conditional() {
    // RED PHASE: This test should FAIL until we implement realloc handling
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
            HirStatement::If {
                condition: HirExpression::Variable("need_more".to_string()),
                then_block: vec![HirStatement::Assignment {
                    target: "arr".to_string(),
                    value: HirExpression::Realloc {
                        pointer: Box::new(HirExpression::Variable("arr".to_string())),
                        new_size: Box::new(HirExpression::BinaryOp {
                            op: decy_hir::BinaryOperator::Multiply,
                            left: Box::new(HirExpression::IntLiteral(20)),
                            right: Box::new(HirExpression::Sizeof {
                                type_name: "int".to_string(),
                            }),
                        }),
                    },
                }],
                else_block: None,
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: if block with resize
    // DECY-131: Non-boolean conditions are now wrapped with != 0
    assert!(result.contains("if (need_more) != 0"));
    assert!(result.contains("arr.resize(20, 0i32)"));

    // Critical: NO realloc() should appear
    assert!(!result.contains("realloc"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Verify unsafe block count remains 0
///
/// This is critical for the validation goal: <5 unsafe blocks per 1000 LOC
#[test]
fn test_realloc_transformation_unsafe_count() {
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
            HirStatement::Assignment {
                target: "arr".to_string(),
                value: HirExpression::Realloc {
                    pointer: Box::new(HirExpression::Variable("arr".to_string())),
                    new_size: Box::new(HirExpression::BinaryOp {
                        op: decy_hir::BinaryOperator::Multiply,
                        left: Box::new(HirExpression::IntLiteral(20)),
                        right: Box::new(HirExpression::Sizeof {
                            type_name: "int".to_string(),
                        }),
                    }),
                },
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Count unsafe blocks (should be 0)
    let unsafe_count = result.matches("unsafe").count();
    assert_eq!(
        unsafe_count, 0,
        "realloc → Vec transformation should not introduce unsafe blocks"
    );
}
