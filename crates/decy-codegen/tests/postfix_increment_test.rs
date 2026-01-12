//! Tests for post-increment/decrement transformation (EXPR-POSTFIX-INCR validation)
//!
//! Reference: ISO C99 §6.5.2.4, K&R §2.8
//!
//! This module tests the transformation of C post-increment/decrement operators
//! to Rust expressions. Post-increment returns the old value before incrementing.

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType, UnaryOperator};

/// Test simple post-increment (x++)
///
/// C: int test() {
///      int x = 5;
///      int y = x++;
///      return y;
///    }
///
/// Rust: fn test() -> i32 {
///         let mut x = 5;
///         let y = { let tmp = x; x += 1; tmp };
///         return y;
///       }
///
/// Reference: K&R §2.8, ISO C99 §6.5.2.4
#[test]
#[ignore = "RED phase: post-increment block generation not yet implemented"]
fn test_simple_post_increment() {
    // RED PHASE: This test should FAIL until we implement post-increment
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(5)),
            },
            HirStatement::VariableDeclaration {
                name: "y".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::UnaryOp {
                    op: UnaryOperator::PostIncrement,
                    operand: Box::new(HirExpression::Variable("x".to_string())),
                }),
            },
            HirStatement::Return(Some(HirExpression::Variable("y".to_string()))),
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Block with tmp variable
    assert!(result.contains("let mut x"));
    assert!(result.contains("let mut y") || result.contains("let y"));
    assert!(result.contains("let tmp"));
    assert!(result.contains("x += 1") || result.contains("x = x + 1"));

    // Post-increment returns OLD value
    assert!(result.contains("tmp"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test simple post-decrement (x--)
///
/// C: int test() {
///      int x = 10;
///      int y = x--;
///      return y;
///    }
///
/// Rust: fn test() -> i32 {
///         let mut x = 10;
///         let y = { let tmp = x; x -= 1; tmp };
///         return y;
///       }
#[test]
#[ignore = "RED phase: post-decrement block generation not yet implemented"]
fn test_simple_post_decrement() {
    // RED PHASE: This test should FAIL until we implement post-decrement
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(10)),
            },
            HirStatement::VariableDeclaration {
                name: "y".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::UnaryOp {
                    op: UnaryOperator::PostDecrement,
                    operand: Box::new(HirExpression::Variable("x".to_string())),
                }),
            },
            HirStatement::Return(Some(HirExpression::Variable("y".to_string()))),
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Block with tmp variable and decrement
    assert!(result.contains("let mut x"));
    assert!(result.contains("let mut y") || result.contains("let y"));
    assert!(result.contains("let tmp"));
    assert!(result.contains("x -= 1") || result.contains("x = x - 1"));
    assert!(result.contains("tmp"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test post-increment in expression
///
/// C: int test() {
///      int x = 5;
///      return x++ + 10;
///    }
///
/// Rust: fn test() -> i32 {
///         let mut x = 5;
///         return { let tmp = x; x += 1; tmp } + 10;
///       }
#[test]
#[ignore = "RED phase: post-increment block generation not yet implemented"]
fn test_post_increment_in_expression() {
    // RED PHASE: This test should FAIL until we implement post-increment
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(5)),
            },
            HirStatement::Return(Some(HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Add,
                left: Box::new(HirExpression::UnaryOp {
                    op: UnaryOperator::PostIncrement,
                    operand: Box::new(HirExpression::Variable("x".to_string())),
                }),
                right: Box::new(HirExpression::IntLiteral(10)),
            })),
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Block expression within addition
    assert!(result.contains("let tmp"));
    assert!(result.contains("x += 1") || result.contains("x = x + 1"));
    assert!(result.contains("+ 10"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test post-increment with function parameter
///
/// C: int test(int n) {
///      return n++;
///    }
///
/// Rust: fn test(mut n: i32) -> i32 {
///         { let tmp = n; n += 1; tmp }
///       }
#[test]
#[ignore = "RED phase: post-increment block generation not yet implemented"]
fn test_post_increment_parameter() {
    // RED PHASE: This test should FAIL until we implement post-increment
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::UnaryOp {
            op: UnaryOperator::PostIncrement,
            operand: Box::new(HirExpression::Variable("n".to_string())),
        }))],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Parameter must be mutable for post-increment
    assert!(result.contains("mut n"));
    assert!(result.contains("let tmp"));
    assert!(result.contains("n += 1") || result.contains("n = n + 1"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test multiple post-increments in sequence
///
/// C: int test() {
///      int x = 1;
///      int a = x++;
///      int b = x++;
///      return a + b;
///    }
///
/// Rust: fn test() -> i32 {
///         let mut x = 1;
///         let a = { let tmp = x; x += 1; tmp };
///         let b = { let tmp = x; x += 1; tmp };
///         return a + b;
///       }
#[test]
#[ignore = "RED phase: post-increment block generation not yet implemented"]
fn test_multiple_post_increments() {
    // RED PHASE: This test should FAIL until we implement post-increment
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(1)),
            },
            HirStatement::VariableDeclaration {
                name: "a".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::UnaryOp {
                    op: UnaryOperator::PostIncrement,
                    operand: Box::new(HirExpression::Variable("x".to_string())),
                }),
            },
            HirStatement::VariableDeclaration {
                name: "b".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::UnaryOp {
                    op: UnaryOperator::PostIncrement,
                    operand: Box::new(HirExpression::Variable("x".to_string())),
                }),
            },
            HirStatement::Return(Some(HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            })),
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Each post-increment creates its own block
    // Should contain "let tmp" twice
    let tmp_count = result.matches("let tmp").count();
    assert!(tmp_count >= 2, "Should have at least 2 tmp variables");

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test post-decrement in loop
///
/// C: int countdown(int n) {
///      int sum = 0;
///      while (n > 0) {
///        sum += n--;
///      }
///      return sum;
///    }
///
/// Rust: fn countdown(mut n: i32) -> i32 {
///         let mut sum = 0;
///         while n > 0 {
///           sum += { let tmp = n; n -= 1; tmp };
///         }
///         return sum;
///       }
#[test]
#[ignore = "RED phase: post-decrement block generation not yet implemented"]
fn test_post_decrement_in_loop() {
    // RED PHASE: This test should FAIL until we implement post-decrement
    let func = HirFunction::new_with_body(
        "countdown".to_string(),
        HirType::Int,
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![
            HirStatement::VariableDeclaration {
                name: "sum".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::While {
                condition: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::GreaterThan,
                    left: Box::new(HirExpression::Variable("n".to_string())),
                    right: Box::new(HirExpression::IntLiteral(0)),
                },
                body: vec![HirStatement::Assignment {
                    target: "sum".to_string(),
                    value: HirExpression::BinaryOp {
                        op: decy_hir::BinaryOperator::Add,
                        left: Box::new(HirExpression::Variable("sum".to_string())),
                        right: Box::new(HirExpression::UnaryOp {
                            op: UnaryOperator::PostDecrement,
                            operand: Box::new(HirExpression::Variable("n".to_string())),
                        }),
                    },
                }],
            },
            HirStatement::Return(Some(HirExpression::Variable("sum".to_string()))),
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Post-decrement in loop body
    assert!(result.contains("fn countdown(mut n: i32) -> i32"));
    assert!(result.contains("while n > 0"));
    assert!(result.contains("let tmp"));
    assert!(result.contains("n -= 1") || result.contains("n = n - 1"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test post-increment with different types (float)
///
/// C: float test() {
///      float x = 1.5;
///      return x++;
///    }
///
/// Rust: fn test() -> f32 {
///         let mut x = 1.5f32;
///         { let tmp = x; x += 1.0; tmp }
///       }
#[test]
#[ignore = "RED phase: post-increment float block generation not yet implemented"]
fn test_post_increment_float() {
    // RED PHASE: This test should FAIL until we implement post-increment
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Float,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Float,
                initializer: Some(HirExpression::IntLiteral(1)), // Will be converted to float
            },
            HirStatement::Return(Some(HirExpression::UnaryOp {
                op: UnaryOperator::PostIncrement,
                operand: Box::new(HirExpression::Variable("x".to_string())),
            })),
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected: Float post-increment
    assert!(result.contains("let mut x"));
    assert!(result.contains("let tmp"));
    // Float increment should add 1.0
    assert!(result.contains("+= 1") || result.contains("+ 1"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Verify unsafe block count remains 0
///
/// This is critical for the validation goal: <5 unsafe blocks per 1000 LOC
#[test]
fn test_post_increment_unsafe_count() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::VariableDeclaration {
                name: "a".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::UnaryOp {
                    op: UnaryOperator::PostIncrement,
                    operand: Box::new(HirExpression::Variable("x".to_string())),
                }),
            },
            HirStatement::VariableDeclaration {
                name: "b".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::UnaryOp {
                    op: UnaryOperator::PostDecrement,
                    operand: Box::new(HirExpression::Variable("x".to_string())),
                }),
            },
            HirStatement::Return(Some(HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            })),
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Count unsafe blocks (should be 0)
    let unsafe_count = result.matches("unsafe").count();
    assert_eq!(
        unsafe_count, 0,
        "Post-increment/decrement should not introduce unsafe blocks"
    );
}
