//! Tests for switch/case code generation (DECY-026 RED phase).
//!
//! Tests for generating Rust match expressions from C switch/case statements.

use super::*;
use decy_hir::{BinaryOperator, HirExpression, HirStatement, SwitchCase};

#[test]
fn test_generate_simple_switch() {
    // C: switch (x) { case 1: break; default: break; }
    // Rust: match x { 1 => {}, _ => {} }
    let switch_stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![SwitchCase {
            value: Some(HirExpression::IntLiteral(1)),
            body: vec![HirStatement::Break],
        }],
        default_case: Some(vec![HirStatement::Break]),
    };

    let codegen = CodeGenerator::new();
    let code = codegen.generate_statement(&switch_stmt);

    assert!(code.contains("match"));
    assert!(code.contains("x"));
    assert!(code.contains("1 =>"));
    assert!(code.contains("_ =>"));
}

#[test]
fn test_generate_switch_with_multiple_cases() {
    // C: switch (status) { case 0: return 1; case 1: return 2; default: return 0; }
    let switch_stmt = HirStatement::Switch {
        condition: HirExpression::Variable("status".to_string()),
        cases: vec![
            SwitchCase {
                value: Some(HirExpression::IntLiteral(0)),
                body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
            },
            SwitchCase {
                value: Some(HirExpression::IntLiteral(1)),
                body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(2)))],
            },
        ],
        default_case: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
            0,
        )))]),
    };

    let codegen = CodeGenerator::new();
    let code = codegen.generate_statement(&switch_stmt);

    assert!(code.contains("0 =>"));
    assert!(code.contains("1 =>"));
    assert!(code.contains("_ =>"));
}

#[test]
fn test_generate_switch_without_default() {
    // C: switch (x) { case 1: break; }
    // Rust: match x { 1 => {}, _ => {} } // Still need _ for exhaustiveness
    let switch_stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![SwitchCase {
            value: Some(HirExpression::IntLiteral(1)),
            body: vec![HirStatement::Break],
        }],
        default_case: None,
    };

    let codegen = CodeGenerator::new();
    let code = codegen.generate_statement(&switch_stmt);

    assert!(code.contains("match"));
    assert!(code.contains("1 =>"));
    // Rust requires exhaustive match, so we should add empty default
    assert!(code.contains("_ =>"));
}

#[test]
fn test_generate_switch_case_body() {
    // C: switch (x) { case 1: y = 10; break; }
    let switch_stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![SwitchCase {
            value: Some(HirExpression::IntLiteral(1)),
            body: vec![
                HirStatement::Assignment {
                    target: "y".to_string(),
                    value: HirExpression::IntLiteral(10),
                },
                HirStatement::Break,
            ],
        }],
        default_case: None,
    };

    let codegen = CodeGenerator::new();
    let code = codegen.generate_statement(&switch_stmt);

    assert!(code.contains("y = 10"));
}

#[test]
fn test_generate_switch_with_braces() {
    // Match arms should have braces for blocks
    let switch_stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![SwitchCase {
            value: Some(HirExpression::IntLiteral(1)),
            body: vec![HirStatement::Break],
        }],
        default_case: None,
    };

    let codegen = CodeGenerator::new();
    let code = codegen.generate_statement(&switch_stmt);

    // Match arms with blocks should have { }
    assert!(code.contains('{'));
    assert!(code.contains('}'));
}

#[test]
fn test_generate_switch_removes_break() {
    // Break statements in C switch should be removed in Rust match
    // (Rust match doesn't need explicit breaks)
    let switch_stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![SwitchCase {
            value: Some(HirExpression::IntLiteral(1)),
            body: vec![
                HirStatement::Assignment {
                    target: "y".to_string(),
                    value: HirExpression::IntLiteral(1),
                },
                HirStatement::Break,
            ],
        }],
        default_case: None,
    };

    let codegen = CodeGenerator::new();
    let code = codegen.generate_statement(&switch_stmt);

    // Should not contain "break;" in match arms
    assert!(!code.contains("break"));
}

#[test]
fn test_generate_switch_with_expression_condition() {
    // C: switch (x + 1) { case 1: break; }
    let switch_stmt = HirStatement::Switch {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        },
        cases: vec![SwitchCase {
            value: Some(HirExpression::IntLiteral(1)),
            body: vec![HirStatement::Break],
        }],
        default_case: None,
    };

    let codegen = CodeGenerator::new();
    let code = codegen.generate_statement(&switch_stmt);

    assert!(code.contains("match"));
    assert!(code.contains("x + 1"));
}

#[test]
fn test_generate_switch_formatting() {
    // Test that generated code is properly formatted
    let switch_stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![SwitchCase {
            value: Some(HirExpression::IntLiteral(1)),
            body: vec![HirStatement::Break],
        }],
        default_case: None,
    };

    let codegen = CodeGenerator::new();
    let code = codegen.generate_statement(&switch_stmt);

    // Should start with "match"
    assert!(code.starts_with("match"));
}

#[test]
fn test_generate_switch_with_return() {
    // C: switch (x) { case 1: return 42; default: return 0; }
    let switch_stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![SwitchCase {
            value: Some(HirExpression::IntLiteral(1)),
            body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(42)))],
        }],
        default_case: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
            0,
        )))]),
    };

    let codegen = CodeGenerator::new();
    let code = codegen.generate_statement(&switch_stmt);

    assert!(code.contains("return 42"));
    assert!(code.contains("return 0"));
}
