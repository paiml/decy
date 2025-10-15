//! End-to-end integration test for switch → match transformation (STMT-SWITCH validation)
//!
//! Reference: ISO C99 §6.8.4.2 (switch statement)

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType, SwitchCase};

/// Test simple switch statement with single case
///
/// C: int test_switch(int x) {
///      switch (x) {
///        case 1:
///          return 10;
///        default:
///          return 0;
///      }
///    }
///
/// Rust: fn test_switch(mut x: i32) -> i32 {
///         match x {
///           1 => { return 10; },
///           _ => { return 0; }
///         }
///       }
#[test]
fn test_simple_switch_statement() {
    let func = HirFunction::new_with_body(
        "test_switch".to_string(),
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Switch {
            condition: HirExpression::Variable("x".to_string()),
            cases: vec![SwitchCase {
                value: Some(HirExpression::IntLiteral(1)),
                body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(10)))],
            }],
            default_case: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
                0,
            )))]),
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify function signature
    assert!(result.contains("fn test_switch(mut x: i32) -> i32"));

    // Verify match statement
    assert!(result.contains("match x"));
    assert!(result.contains("1 =>"));
    assert!(result.contains("return 10"));
    assert!(result.contains("_ =>"));
    assert!(result.contains("return 0"));

    // Critical: no break statements (Rust doesn't need them)
    assert!(!result.contains("break"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test switch with multiple cases
///
/// C: int classify(int status) {
///      switch (status) {
///        case 0:
///          return 100;
///        case 1:
///          return 200;
///        case 2:
///          return 300;
///        default:
///          return -1;
///      }
///    }
#[test]
fn test_switch_with_multiple_cases() {
    let func = HirFunction::new_with_body(
        "classify".to_string(),
        HirType::Int,
        vec![HirParameter::new("status".to_string(), HirType::Int)],
        vec![HirStatement::Switch {
            condition: HirExpression::Variable("status".to_string()),
            cases: vec![
                SwitchCase {
                    value: Some(HirExpression::IntLiteral(0)),
                    body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(100)))],
                },
                SwitchCase {
                    value: Some(HirExpression::IntLiteral(1)),
                    body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(200)))],
                },
                SwitchCase {
                    value: Some(HirExpression::IntLiteral(2)),
                    body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(300)))],
                },
            ],
            default_case: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
                -1,
            )))]),
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify all cases are present
    assert!(result.contains("0 =>"));
    assert!(result.contains("return 100"));
    assert!(result.contains("1 =>"));
    assert!(result.contains("return 200"));
    assert!(result.contains("2 =>"));
    assert!(result.contains("return 300"));
    assert!(result.contains("_ =>"));
    assert!(result.contains("return -1"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test switch without default case (Rust requires exhaustive match)
///
/// C: void handle_one(int x) {
///      switch (x) {
///        case 1:
///          return;
///      }
///    }
///
/// Rust should add empty default case for exhaustiveness
#[test]
fn test_switch_without_default_adds_wildcard() {
    let func = HirFunction::new_with_body(
        "handle_one".to_string(),
        HirType::Void,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Switch {
            condition: HirExpression::Variable("x".to_string()),
            cases: vec![SwitchCase {
                value: Some(HirExpression::IntLiteral(1)),
                body: vec![HirStatement::Return(None)],
            }],
            default_case: None, // No default in C
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify match statement has wildcard for exhaustiveness
    assert!(result.contains("match x"));
    assert!(result.contains("1 =>"));
    assert!(result.contains("_ =>")); // Rust requires this for exhaustiveness

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test switch with statements in case body
///
/// C: void process(int x) {
///      int y;
///      switch (x) {
///        case 1:
///          y = 10;
///          break;
///        case 2:
///          y = 20;
///          break;
///        default:
///          y = 0;
///          break;
///      }
///    }
#[test]
fn test_switch_with_case_body_statements() {
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![
            HirStatement::VariableDeclaration {
                name: "y".to_string(),
                var_type: HirType::Int,
                initializer: None,
            },
            HirStatement::Switch {
                condition: HirExpression::Variable("x".to_string()),
                cases: vec![
                    SwitchCase {
                        value: Some(HirExpression::IntLiteral(1)),
                        body: vec![
                            HirStatement::Assignment {
                                target: "y".to_string(),
                                value: HirExpression::IntLiteral(10),
                            },
                            HirStatement::Break,
                        ],
                    },
                    SwitchCase {
                        value: Some(HirExpression::IntLiteral(2)),
                        body: vec![
                            HirStatement::Assignment {
                                target: "y".to_string(),
                                value: HirExpression::IntLiteral(20),
                            },
                            HirStatement::Break,
                        ],
                    },
                ],
                default_case: Some(vec![
                    HirStatement::Assignment {
                        target: "y".to_string(),
                        value: HirExpression::IntLiteral(0),
                    },
                    HirStatement::Break,
                ]),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify variable declaration
    assert!(result.contains("let mut y: i32"));

    // Verify match with assignments
    assert!(result.contains("match x"));
    assert!(result.contains("1 =>"));
    assert!(result.contains("y = 10"));
    assert!(result.contains("2 =>"));
    assert!(result.contains("y = 20"));
    assert!(result.contains("_ =>"));
    assert!(result.contains("y = 0"));

    // Break statements should be removed
    assert!(!result.contains("break"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test switch with expression as condition
///
/// C: int test(int x) {
///      switch (x + 1) {
///        case 2:
///          return 42;
///        default:
///          return 0;
///      }
///    }
#[test]
fn test_switch_with_expression_condition() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Switch {
            condition: HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
            cases: vec![SwitchCase {
                value: Some(HirExpression::IntLiteral(2)),
                body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(42)))],
            }],
            default_case: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
                0,
            )))]),
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify expression in match condition
    assert!(result.contains("match x + 1"));
    assert!(result.contains("2 =>"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Verify unsafe block count remains 0
///
/// This is critical for the validation goal: <5 unsafe blocks per 1000 LOC
#[test]
fn test_switch_transformation_unsafe_count() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Switch {
            condition: HirExpression::Variable("x".to_string()),
            cases: vec![
                SwitchCase {
                    value: Some(HirExpression::IntLiteral(1)),
                    body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(10)))],
                },
                SwitchCase {
                    value: Some(HirExpression::IntLiteral(2)),
                    body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(20)))],
                },
            ],
            default_case: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
                0,
            )))]),
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Count unsafe blocks (should be 0)
    let unsafe_count = result.matches("unsafe").count();
    assert_eq!(
        unsafe_count, 0,
        "switch → match transformation should not introduce unsafe blocks"
    );
}
