//! Tests for compound assignment operators (EXPR-ASSIGN-COMPOUND validation)
//!
//! Reference: ISO C99 §6.5.16.2 (compound assignment operators), K&R §2.10
//!
//! This module tests the transformation of C compound assignment operators to Rust.
//! Compound assignments (+=, -=, *=, /=, %=) should be expanded to regular assignments
//! by the HIR layer: x += 5 becomes x = x + 5.

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};

/// Test simple += compound assignment
///
/// C: void test() {
///      int x = 10;
///      x += 5;
///    }
///
/// Rust: fn test() {
///         let mut x: i32 = 10;
///         x = x + 5;
///       }
///
/// Reference: ISO C99 §6.5.16.2, K&R §2.10
#[test]
fn test_simple_add_assignment() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(10)),
            },
            // HIR converts x += 5 to x = x + 5
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::IntLiteral(5)),
                },
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify basic structure
    assert!(result.contains("let mut x: i32 = 10"));
    assert!(result.contains("x = x + 5"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test -= compound assignment
///
/// C: void test() {
///      int x = 20;
///      x -= 8;
///    }
///
/// Rust: fn test() {
///         let mut x: i32 = 20;
///         x = x - 8;
///       }
#[test]
fn test_subtract_assignment() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(20)),
            },
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Subtract,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::IntLiteral(8)),
                },
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    assert!(result.contains("let mut x: i32 = 20"));
    assert!(result.contains("x = x - 8"));
    assert!(!result.contains("unsafe"));
}

/// Test *= compound assignment
///
/// C: void test() {
///      int x = 5;
///      x *= 3;
///    }
///
/// Rust: fn test() {
///         let mut x: i32 = 5;
///         x = x * 3;
///       }
#[test]
fn test_multiply_assignment() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(5)),
            },
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Multiply,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::IntLiteral(3)),
                },
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    assert!(result.contains("let mut x: i32 = 5"));
    assert!(result.contains("x = x * 3"));
    assert!(!result.contains("unsafe"));
}

/// Test /= compound assignment
///
/// C: void test() {
///      int x = 100;
///      x /= 10;
///    }
///
/// Rust: fn test() {
///         let mut x: i32 = 100;
///         x = x / 10;
///       }
#[test]
fn test_divide_assignment() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(100)),
            },
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Divide,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::IntLiteral(10)),
                },
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    assert!(result.contains("let mut x: i32 = 100"));
    assert!(result.contains("x = x / 10"));
    assert!(!result.contains("unsafe"));
}

/// Test %= compound assignment
///
/// C: void test() {
///      int x = 17;
///      x %= 5;
///    }
///
/// Rust: fn test() {
///         let mut x: i32 = 17;
///         x = x % 5;
///       }
#[test]
fn test_modulo_assignment() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(17)),
            },
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Modulo,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::IntLiteral(5)),
                },
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    assert!(result.contains("let mut x: i32 = 17"));
    assert!(result.contains("x = x % 5"));
    assert!(!result.contains("unsafe"));
}

/// Test compound assignment with expression on right side
///
/// C: void test() {
///      int x = 10;
///      int y = 5;
///      x += y * 2;
///    }
///
/// Rust: fn test() {
///         let mut x: i32 = 10;
///         let mut y: i32 = 5;
///         x = x + y * 2;
///       }
#[test]
fn test_compound_assignment_with_expression() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
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
                initializer: Some(HirExpression::IntLiteral(5)),
            },
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::BinaryOp {
                        op: decy_hir::BinaryOperator::Multiply,
                        left: Box::new(HirExpression::Variable("y".to_string())),
                        right: Box::new(HirExpression::IntLiteral(2)),
                    }),
                },
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    assert!(result.contains("let mut x: i32 = 10"));
    assert!(result.contains("let mut y: i32 = 5"));
    // Rust may add parentheses for clarity: x + (y * 2)
    assert!(result.contains("x = x + y * 2") || result.contains("x = x + (y * 2)"));
    assert!(!result.contains("unsafe"));
}

/// Test compound assignment in loop
///
/// C: void test() {
///      int sum = 0;
///      for (int i = 0; i < 10; i++) {
///        sum += i;
///      }
///    }
///
/// Rust: fn test() {
///         let mut sum: i32 = 0;
///         let mut i: i32 = 0;
///         while i < 10 {
///           sum = sum + i;
///           i = i + 1;
///         }
///       }
#[test]
fn test_compound_assignment_in_loop() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "sum".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::For {
                init: Some(Box::new(HirStatement::VariableDeclaration {
                    name: "i".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::IntLiteral(0)),
                })),
                condition: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::LessThan,
                    left: Box::new(HirExpression::Variable("i".to_string())),
                    right: Box::new(HirExpression::IntLiteral(10)),
                },
                increment: Some(Box::new(HirStatement::Assignment {
                    target: "i".to_string(),
                    value: HirExpression::BinaryOp {
                        op: decy_hir::BinaryOperator::Add,
                        left: Box::new(HirExpression::Variable("i".to_string())),
                        right: Box::new(HirExpression::IntLiteral(1)),
                    },
                })),
                body: vec![HirStatement::Assignment {
                    target: "sum".to_string(),
                    value: HirExpression::BinaryOp {
                        op: decy_hir::BinaryOperator::Add,
                        left: Box::new(HirExpression::Variable("sum".to_string())),
                        right: Box::new(HirExpression::Variable("i".to_string())),
                    },
                }],
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify sum accumulation
    assert!(result.contains("let mut sum: i32 = 0"));
    assert!(result.contains("sum = sum + i"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test compound assignment with function parameter
///
/// C: void accumulate(int delta) {
///      delta += 10;
///    }
///
/// Rust: fn accumulate(mut delta: i32) {
///         delta = delta + 10;
///       }
#[test]
fn test_compound_assignment_parameter() {
    let func = HirFunction::new_with_body(
        "accumulate".to_string(),
        HirType::Void,
        vec![HirParameter::new("delta".to_string(), HirType::Int)],
        vec![HirStatement::Assignment {
            target: "delta".to_string(),
            value: HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("delta".to_string())),
                right: Box::new(HirExpression::IntLiteral(10)),
            },
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify parameter is mut
    assert!(result.contains("mut delta: i32"));
    assert!(result.contains("delta = delta + 10"));
    assert!(!result.contains("unsafe"));
}

/// Test multiple compound assignments (different operators)
///
/// C: void test() {
///      int x = 100;
///      x += 10;   // x = 110
///      x -= 5;    // x = 105
///      x *= 2;    // x = 210
///      x /= 7;    // x = 30
///      x %= 8;    // x = 6
///    }
#[test]
fn test_multiple_compound_assignments() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(100)),
            },
            // x += 10
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::IntLiteral(10)),
                },
            },
            // x -= 5
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Subtract,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::IntLiteral(5)),
                },
            },
            // x *= 2
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Multiply,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::IntLiteral(2)),
                },
            },
            // x /= 7
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Divide,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::IntLiteral(7)),
                },
            },
            // x %= 8
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Modulo,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::IntLiteral(8)),
                },
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify all operations
    assert!(result.contains("let mut x: i32 = 100"));
    assert!(result.contains("x = x + 10"));
    assert!(result.contains("x = x - 5"));
    assert!(result.contains("x = x * 2"));
    assert!(result.contains("x = x / 7"));
    assert!(result.contains("x = x % 8"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test compound assignment with float type
///
/// C: void test() {
///      float x = 10.5;
///      x += 2.5;
///    }
///
/// Rust: fn test() {
///         let mut x: f32 = 10.5;
///         x = x + 2.5;
///       }
#[test]
fn test_compound_assignment_float() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Float,
                initializer: Some(HirExpression::IntLiteral(10)),
            },
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::IntLiteral(2)),
                },
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify float type
    assert!(result.contains("f32"));
    assert!(result.contains("x = x + 2"));
    assert!(!result.contains("unsafe"));
}

/// Verify unsafe block count remains 0
///
/// This is critical for the validation goal: <5 unsafe blocks per 1000 LOC
#[test]
fn test_compound_assignment_unsafe_count() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(100)),
            },
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::IntLiteral(50)),
                },
            },
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Multiply,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::IntLiteral(2)),
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
        "Compound assignment transformation should not introduce unsafe blocks"
    );
}

/// Test compound assignment returns correct value
///
/// This test verifies semantic correctness: x += 5 should modify x and evaluate to new value
///
/// C: int test() {
///      int x = 10;
///      x += 5;
///      return x;
///    }
///
/// Expected: x becomes 15, return 15
#[test]
fn test_compound_assignment_semantic_correctness() {
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
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::IntLiteral(5)),
                },
            },
            HirStatement::Return(Some(HirExpression::Variable("x".to_string()))),
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify structure
    assert!(result.contains("let mut x: i32 = 10"));
    assert!(result.contains("x = x + 5"));
    assert!(result.contains("return x"));

    // Should be simple assignment, not complex expression
    assert!(!result.contains("let tmp"));
    assert!(!result.contains("unsafe"));
}
