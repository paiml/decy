//! Tests for switch/case statements in HIR (DECY-026 RED phase).
//!
//! Tests for representing C switch/case statements in HIR,
//! with plans to transpile to Rust match expressions.

use super::*;

#[test]
fn test_create_switch_statement() {
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

    match switch_stmt {
        HirStatement::Switch {
            condition,
            cases,
            default_case,
        } => {
            assert!(matches!(condition, HirExpression::Variable(_)));
            assert_eq!(cases.len(), 1);
            assert!(default_case.is_some());
        }
        _ => panic!("Expected Switch statement"),
    }
}

#[test]
fn test_switch_with_multiple_cases() {
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

    match switch_stmt {
        HirStatement::Switch { cases, .. } => {
            assert_eq!(cases.len(), 2);
        }
        _ => panic!("Expected Switch statement"),
    }
}

#[test]
fn test_switch_without_default() {
    // C: switch (x) { case 1: break; }
    // Default case is optional
    let switch_stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![SwitchCase {
            value: Some(HirExpression::IntLiteral(1)),
            body: vec![HirStatement::Break],
        }],
        default_case: None,
    };

    match switch_stmt {
        HirStatement::Switch { default_case, .. } => {
            assert!(default_case.is_none());
        }
        _ => panic!("Expected Switch statement"),
    }
}

#[test]
fn test_switch_case_with_multiple_statements() {
    // C: switch (x) { case 1: y = 1; z = 2; break; }
    let switch_stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![SwitchCase {
            value: Some(HirExpression::IntLiteral(1)),
            body: vec![
                HirStatement::Assignment {
                    target: "y".to_string(),
                    value: HirExpression::IntLiteral(1),
                },
                HirStatement::Assignment {
                    target: "z".to_string(),
                    value: HirExpression::IntLiteral(2),
                },
                HirStatement::Break,
            ],
        }],
        default_case: None,
    };

    match switch_stmt {
        HirStatement::Switch { cases, .. } => {
            assert_eq!(cases[0].body.len(), 3);
        }
        _ => panic!("Expected Switch statement"),
    }
}

#[test]
fn test_switch_case_structure() {
    // Test SwitchCase structure directly
    let case = SwitchCase {
        value: Some(HirExpression::IntLiteral(42)),
        body: vec![HirStatement::Break],
    };

    assert!(case.value.is_some());
    assert_eq!(case.body.len(), 1);
}

#[test]
fn test_switch_default_case_structure() {
    // Default case has no value
    let default = SwitchCase {
        value: None,
        body: vec![HirStatement::Break],
    };

    assert!(default.value.is_none());
}

#[test]
fn test_switch_with_nested_if() {
    // C: switch (x) { case 1: if (y) { break; } break; }
    let switch_stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![SwitchCase {
            value: Some(HirExpression::IntLiteral(1)),
            body: vec![
                HirStatement::If {
                    condition: HirExpression::Variable("y".to_string()),
                    then_block: vec![HirStatement::Break],
                    else_block: None,
                },
                HirStatement::Break,
            ],
        }],
        default_case: None,
    };

    match switch_stmt {
        HirStatement::Switch { cases, .. } => {
            assert!(matches!(cases[0].body[0], HirStatement::If { .. }));
        }
        _ => panic!("Expected Switch statement"),
    }
}

#[test]
fn test_switch_clone() {
    let switch_stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![SwitchCase {
            value: Some(HirExpression::IntLiteral(1)),
            body: vec![HirStatement::Break],
        }],
        default_case: None,
    };

    let cloned = switch_stmt.clone();
    assert_eq!(switch_stmt, cloned);
}

#[test]
fn test_switch_with_expression_condition() {
    // C: switch (x + y) { case 1: break; }
    let switch_stmt = HirStatement::Switch {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::Variable("y".to_string())),
        },
        cases: vec![SwitchCase {
            value: Some(HirExpression::IntLiteral(1)),
            body: vec![HirStatement::Break],
        }],
        default_case: None,
    };

    match switch_stmt {
        HirStatement::Switch { condition, .. } => {
            assert!(matches!(condition, HirExpression::BinaryOp { .. }));
        }
        _ => panic!("Expected Switch statement"),
    }
}

#[test]
fn test_switch_fallthrough_pattern() {
    // C: switch (x) { case 1: case 2: break; }
    // Represented as multiple cases with empty bodies, then final case with break
    // This is a simplified representation - full fallthrough needs additional analysis
    let switch_stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![
            SwitchCase {
                value: Some(HirExpression::IntLiteral(1)),
                body: vec![], // Fallthrough to next case
            },
            SwitchCase {
                value: Some(HirExpression::IntLiteral(2)),
                body: vec![HirStatement::Break],
            },
        ],
        default_case: None,
    };

    match switch_stmt {
        HirStatement::Switch { cases, .. } => {
            assert_eq!(cases.len(), 2);
            assert!(cases[0].body.is_empty()); // First case falls through
        }
        _ => panic!("Expected Switch statement"),
    }
}
