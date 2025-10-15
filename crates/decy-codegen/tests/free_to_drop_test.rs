//! Tests for free() → automatic drop transformation (STDLIB-FREE validation)
//!
//! Reference: K&R §8.7, ISO C99 §7.20.3.2
//!
//! This module tests the transformation of C free() to Rust's automatic drop.
//! free() in C is unsafe and error-prone (double-free, use-after-free).
//! Rust's RAII (Resource Acquisition Is Initialization) handles memory
//! cleanup automatically when values go out of scope.

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirStatement, HirType};

/// Test free(ptr) → automatic drop (eliminated from code)
///
/// C: int* p = malloc(sizeof(int)); free(p);
/// Rust: let p = Box::new(0i32); // Automatic drop, no explicit free
///
/// Reference: K&R §8.7, ISO C99 §7.20.3.2
#[test]
fn test_free_eliminated_for_box() {
    // RED PHASE: This test should FAIL until we implement free handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "p".to_string(),
                var_type: HirType::Box(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Malloc {
                    size: Box::new(HirExpression::Sizeof {
                        type_name: "int".to_string(),
                    }),
                }),
            },
            HirStatement::Free {
                pointer: HirExpression::Variable("p".to_string()),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Box allocation without explicit free
    assert!(result.contains("let mut p: Box<i32> = Box::new(0i32)"));

    // Critical: free() should NOT appear in generated code
    assert!(!result.contains("free"));
    assert!(!result.contains("drop(p)"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test free() with Vec → automatic drop
///
/// C: int* arr = malloc(n * sizeof(int)); free(arr);
/// Rust: let arr = Vec::with_capacity(n); // Automatic drop
///
/// Reference: K&R §8.7, ISO C99 §7.20.3.2
#[test]
fn test_free_eliminated_for_vec() {
    // RED PHASE: This test should FAIL until we implement free handling
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
                var_type: HirType::Vec(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Malloc {
                    size: Box::new(HirExpression::BinaryOp {
                        op: decy_hir::BinaryOperator::Multiply,
                        left: Box::new(HirExpression::Variable("n".to_string())),
                        right: Box::new(HirExpression::Sizeof {
                            type_name: "int".to_string(),
                        }),
                    }),
                }),
            },
            HirStatement::Free {
                pointer: HirExpression::Variable("arr".to_string()),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Vec allocation without explicit free
    assert!(result.contains("let mut arr: Vec<i32> = Vec::with_capacity(n)"));

    // Critical: free() should NOT appear in generated code
    assert!(!result.contains("free"));
    assert!(!result.contains("drop(arr)"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test free() on NULL pointer → eliminated (safe in C, unnecessary in Rust)
///
/// C: free(NULL); // Safe no-op in C
/// Rust: // No code generated
///
/// Reference: ISO C99 §7.20.3.2 - "If ptr is NULL, no action occurs"
#[test]
fn test_free_null_pointer_eliminated() {
    // RED PHASE: This test should FAIL until we implement free handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Free {
            pointer: HirExpression::NullLiteral,
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Critical: free(NULL) should generate only a comment
    assert!(!result.contains("drop"));

    // Function body should only contain RAII comment
    assert!(result.contains("RAII") || result.contains("deallocated automatically"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test multiple free() calls → all eliminated
///
/// C: int* p1 = malloc(...); int* p2 = malloc(...); free(p1); free(p2);
/// Rust: let p1 = Box::new(...); let p2 = Box::new(...); // Automatic drops
///
/// Reference: K&R §8.7
#[test]
fn test_multiple_free_eliminated() {
    // RED PHASE: This test should FAIL until we implement free handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "p1".to_string(),
                var_type: HirType::Box(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Malloc {
                    size: Box::new(HirExpression::Sizeof {
                        type_name: "int".to_string(),
                    }),
                }),
            },
            HirStatement::VariableDeclaration {
                name: "p2".to_string(),
                var_type: HirType::Box(Box::new(HirType::Float)),
                initializer: Some(HirExpression::Malloc {
                    size: Box::new(HirExpression::Sizeof {
                        type_name: "float".to_string(),
                    }),
                }),
            },
            HirStatement::Free {
                pointer: HirExpression::Variable("p1".to_string()),
            },
            HirStatement::Free {
                pointer: HirExpression::Variable("p2".to_string()),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Box allocations without free
    assert!(result.contains("let mut p1: Box<i32> = Box::new(0i32)"));
    assert!(result.contains("let mut p2: Box<f32> = Box::new(0.0f32)"));

    // Critical: NO free() calls should appear
    assert!(!result.contains("free"));
    assert!(!result.contains("drop"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test free() in conditional block → eliminated
///
/// C: if (condition) { free(p); }
/// Rust: if condition { } // No explicit free, automatic drop
///
/// Reference: K&R §8.7
#[test]
fn test_free_in_conditional_eliminated() {
    // RED PHASE: This test should FAIL until we implement free handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "p".to_string(),
                var_type: HirType::Box(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Malloc {
                    size: Box::new(HirExpression::Sizeof {
                        type_name: "int".to_string(),
                    }),
                }),
            },
            HirStatement::If {
                condition: HirExpression::Variable("condition".to_string()),
                then_block: vec![HirStatement::Free {
                    pointer: HirExpression::Variable("p".to_string()),
                }],
                else_block: None,
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: if block should be empty or minimal
    assert!(result.contains("if condition"));

    // Critical: NO free() or drop() should appear
    assert!(!result.contains("free"));
    assert!(!result.contains("drop(p)"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test free() before return → eliminated (RAII handles cleanup)
///
/// C: int* p = malloc(...); int result = compute(p); free(p); return result;
/// Rust: let p = Box::new(...); let result = compute(&p); result // Auto drop
///
/// Reference: K&R §8.7
#[test]
fn test_free_before_return_eliminated() {
    // RED PHASE: This test should FAIL until we implement free handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "p".to_string(),
                var_type: HirType::Box(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Malloc {
                    size: Box::new(HirExpression::Sizeof {
                        type_name: "int".to_string(),
                    }),
                }),
            },
            HirStatement::VariableDeclaration {
                name: "result".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(42)),
            },
            HirStatement::Free {
                pointer: HirExpression::Variable("p".to_string()),
            },
            HirStatement::Return(Some(HirExpression::Variable("result".to_string()))),
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Box allocation, computation, return (no free)
    assert!(result.contains("let mut p: Box<i32> = Box::new(0i32)"));
    assert!(result.contains("let mut result: i32 = 42"));
    assert!(result.contains("return result"));

    // Critical: NO free() should appear
    assert!(!result.contains("free"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Verify unsafe block count remains 0
///
/// This is critical for the validation goal: <5 unsafe blocks per 1000 LOC
#[test]
fn test_free_transformation_unsafe_count() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "p".to_string(),
                var_type: HirType::Box(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Malloc {
                    size: Box::new(HirExpression::Sizeof {
                        type_name: "int".to_string(),
                    }),
                }),
            },
            HirStatement::Free {
                pointer: HirExpression::Variable("p".to_string()),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Count unsafe blocks (should be 0)
    let unsafe_count = result.matches("unsafe").count();
    assert_eq!(
        unsafe_count, 0,
        "free → drop transformation should not introduce unsafe blocks"
    );
}

/// Test RAII documentation comment generation
///
/// We should generate a comment explaining that free() is eliminated
/// due to Rust's RAII (Resource Acquisition Is Initialization).
#[test]
fn test_free_generates_raii_comment() {
    // RED PHASE: This test should FAIL until we implement free handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "p".to_string(),
                var_type: HirType::Box(Box::new(HirType::Int)),
                initializer: Some(HirExpression::Malloc {
                    size: Box::new(HirExpression::Sizeof {
                        type_name: "int".to_string(),
                    }),
                }),
            },
            HirStatement::Free {
                pointer: HirExpression::Variable("p".to_string()),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Comment explaining RAII
    assert!(
        result.contains("RAII")
            || result.contains("deallocated automatically")
            || result.contains("Memory for"),
        "Expected RAII comment explaining memory deallocation, but found:\n{}",
        result
    );
}
