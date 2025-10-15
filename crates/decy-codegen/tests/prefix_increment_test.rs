//! Tests for pre-increment/decrement transformation (EXPR-UNARY-INCR validation)
//!
//! Reference: ISO C99 §6.5.3.1, K&R §2.8
//!
//! This module tests the transformation of C pre-increment/decrement operators to Rust.
//! Pre-increment/decrement must increment/decrement FIRST, then return the new value.

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType, UnaryOperator};

/// Test simple pre-increment
///
/// C: int test() {
///      int x = 5;
///      int y = ++x;
///      return y;
///    }
///
/// Rust: fn test() -> i32 {
///         let mut x: i32 = 5;
///         let mut y: i32 = { x += 1; x };
///         return y;
///       }
///
/// Reference: ISO C99 §6.5.3.1, K&R §2.8
#[test]
fn test_simple_pre_increment() {
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
                    op: UnaryOperator::PreIncrement,
                    operand: Box::new(HirExpression::Variable("x".to_string())),
                }),
            },
            HirStatement::Return(Some(HirExpression::Variable("y".to_string()))),
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify basic structure
    assert!(result.contains("let mut x"));
    assert!(result.contains("let mut y") || result.contains("let y"));

    // Verify pre-increment block expression: { x += 1; x }
    assert!(result.contains("x += 1") || result.contains("x = x + 1"));
    // Should NOT have tmp variable (that's for post-increment)
    assert!(!result.contains("let tmp"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test simple pre-decrement
///
/// C: int test() {
///      int x = 10;
///      int y = --x;
///      return y;
///    }
///
/// Rust: fn test() -> i32 {
///         let mut x: i32 = 10;
///         let mut y: i32 = { x -= 1; x };
///         return y;
///       }
///
/// Reference: ISO C99 §6.5.3.1, K&R §2.8
#[test]
fn test_simple_pre_decrement() {
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
                    op: UnaryOperator::PreDecrement,
                    operand: Box::new(HirExpression::Variable("x".to_string())),
                }),
            },
            HirStatement::Return(Some(HirExpression::Variable("y".to_string()))),
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify pre-decrement block expression: { x -= 1; x }
    assert!(result.contains("x -= 1") || result.contains("x = x - 1"));
    // Should NOT have tmp variable
    assert!(!result.contains("let tmp"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test pre-increment in expression
///
/// C: int test() {
///      int x = 5;
///      int y = ++x + 10;
///      return y;
///    }
///
/// Rust: fn test() -> i32 {
///         let mut x: i32 = 5;
///         let mut y: i32 = { x += 1; x } + 10;
///         return y;
///       }
///
/// Reference: ISO C99 §6.5.3.1, K&R §2.8
#[test]
fn test_pre_increment_in_expression() {
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
                initializer: Some(HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Add,
                    left: Box::new(HirExpression::UnaryOp {
                        op: UnaryOperator::PreIncrement,
                        operand: Box::new(HirExpression::Variable("x".to_string())),
                    }),
                    right: Box::new(HirExpression::IntLiteral(10)),
                }),
            },
            HirStatement::Return(Some(HirExpression::Variable("y".to_string()))),
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify block expression with addition
    assert!(result.contains("x += 1"));
    assert!(result.contains("+ 10"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test pre-increment on function parameter
///
/// C: int test(int n) {
///      return ++n;
///    }
///
/// Rust: fn test(mut n: i32) -> i32 {
///         return { n += 1; n };
///       }
///
/// Reference: ISO C99 §6.5.3.1, K&R §2.8
#[test]
fn test_pre_increment_parameter() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::UnaryOp {
            op: UnaryOperator::PreIncrement,
            operand: Box::new(HirExpression::Variable("n".to_string())),
        }))],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify parameter is mut
    assert!(result.contains("mut n: i32"));

    // Verify pre-increment in return
    assert!(result.contains("n += 1"));
    assert!(result.contains("return"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test multiple pre-increments
///
/// C: int test() {
///      int x = 0;
///      ++x;
///      ++x;
///      ++x;
///      return x;
///    }
///
/// Rust: fn test() -> i32 {
///         let mut x: i32 = 0;
///         { x += 1; x };
///         { x += 1; x };
///         { x += 1; x };
///         return x;
///       }
///
/// Reference: ISO C99 §6.5.3.1, K&R §2.8
#[test]
fn test_multiple_pre_increments() {
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
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::UnaryOp {
                    op: UnaryOperator::PreIncrement,
                    operand: Box::new(HirExpression::Variable("x".to_string())),
                },
            },
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::UnaryOp {
                    op: UnaryOperator::PreIncrement,
                    operand: Box::new(HirExpression::Variable("x".to_string())),
                },
            },
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::UnaryOp {
                    op: UnaryOperator::PreIncrement,
                    operand: Box::new(HirExpression::Variable("x".to_string())),
                },
            },
            HirStatement::Return(Some(HirExpression::Variable("x".to_string()))),
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify multiple increments
    let increment_count = result.matches("x += 1").count();
    assert_eq!(increment_count, 3, "Should have 3 increments");

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test pre-decrement in loop
///
/// C: void test() {
///      int n = 10;
///      while (--n > 0) {
///        // loop body
///      }
///    }
///
/// Rust: fn test() {
///         let mut n: i32 = 10;
///         while { n -= 1; n } > 0 {
///           // loop body
///         }
///       }
///
/// Reference: ISO C99 §6.5.3.1, K&R §2.8
#[test]
fn test_pre_decrement_in_loop() {
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
            HirStatement::While {
                condition: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::GreaterThan,
                    left: Box::new(HirExpression::UnaryOp {
                        op: UnaryOperator::PreDecrement,
                        operand: Box::new(HirExpression::Variable("n".to_string())),
                    }),
                    right: Box::new(HirExpression::IntLiteral(0)),
                },
                body: vec![],
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify while loop with pre-decrement condition
    assert!(result.contains("while"));
    assert!(result.contains("n -= 1"));
    assert!(result.contains("> 0"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test pre-increment with float type
///
/// C: float test() {
///      float x = 5.0;
///      float y = ++x;
///      return y;
///    }
///
/// Rust: fn test() -> f32 {
///         let mut x: f32 = 5.0;
///         let mut y: f32 = { x += 1; x };
///         return y;
///       }
///
/// Reference: ISO C99 §6.5.3.1, K&R §2.8
#[test]
fn test_pre_increment_float() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Float,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Float,
                initializer: Some(HirExpression::IntLiteral(5)),
            },
            HirStatement::VariableDeclaration {
                name: "y".to_string(),
                var_type: HirType::Float,
                initializer: Some(HirExpression::UnaryOp {
                    op: UnaryOperator::PreIncrement,
                    operand: Box::new(HirExpression::Variable("x".to_string())),
                }),
            },
            HirStatement::Return(Some(HirExpression::Variable("y".to_string()))),
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify float types
    assert!(result.contains("f32"));

    // Verify pre-increment works with floats
    assert!(result.contains("x += 1"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Verify unsafe block count remains 0
///
/// This is critical for the validation goal: <5 unsafe blocks per 1000 LOC
#[test]
fn test_pre_increment_unsafe_count() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(100)),
            },
            HirStatement::VariableDeclaration {
                name: "y".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::UnaryOp {
                    op: UnaryOperator::PreIncrement,
                    operand: Box::new(HirExpression::Variable("x".to_string())),
                }),
            },
            HirStatement::VariableDeclaration {
                name: "z".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::UnaryOp {
                    op: UnaryOperator::PreDecrement,
                    operand: Box::new(HirExpression::Variable("y".to_string())),
                }),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Count unsafe blocks (should be 0)
    let unsafe_count = result.matches("unsafe").count();
    assert_eq!(
        unsafe_count, 0,
        "Pre-increment/decrement transformation should not introduce unsafe blocks"
    );
}

/// Test that pre-increment returns NEW value (not old value like post-increment)
///
/// This test verifies semantic correctness: ++x should increment FIRST
///
/// C: int test() {
///      int x = 5;
///      int y = ++x;  // y should be 6, not 5
///      return y;
///    }
///
/// Expected: y = { x += 1; x } (increments first, returns 6)
/// NOT: y = { let tmp = x; x += 1; tmp } (would return 5)
#[test]
fn test_pre_increment_semantic_correctness() {
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
                    op: UnaryOperator::PreIncrement,
                    operand: Box::new(HirExpression::Variable("x".to_string())),
                }),
            },
            HirStatement::Return(Some(HirExpression::Variable("y".to_string()))),
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Should have increment followed by variable (not tmp)
    assert!(result.contains("x += 1"));

    // Should NOT create tmp variable (that's post-increment behavior)
    assert!(
        !result.contains("let tmp"),
        "Pre-increment should NOT use tmp variable (that's for post-increment)"
    );

    // The pattern should be: { x += 1; x } not { let tmp = x; x += 1; tmp }
    // We can verify this by checking that there's no tmp between the increment and the variable
    let increment_pos = result.find("x += 1").expect("Should have increment");
    let tmp_check = &result[increment_pos..];
    assert!(
        !tmp_check.contains("tmp"),
        "Pre-increment should return x directly after incrementing, not tmp"
    );
}
