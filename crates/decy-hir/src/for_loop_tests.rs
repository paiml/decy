//! Tests for for loop support in HIR (DECY-022 RED phase).

use super::*;

#[test]
fn test_create_for_loop_statement() {
    // for(int i = 0; i < 10; i++) { sum += i; }
    let init = HirStatement::VariableDeclaration {
        name: "i".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(0)),
    };

    let condition = HirExpression::BinaryOp {
        op: BinaryOperator::LessThan,
        left: Box::new(HirExpression::Variable("i".to_string())),
        right: Box::new(HirExpression::IntLiteral(10)),
    };

    let increment = HirStatement::Assignment {
        target: "i".to_string(),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        },
    };

    let body = vec![HirStatement::Assignment {
        target: "sum".to_string(),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("sum".to_string())),
            right: Box::new(HirExpression::Variable("i".to_string())),
        },
    }];

    let for_loop = HirStatement::For {
        init: vec![init],
        condition,
        increment: vec![increment],
        body,
    };

    match for_loop {
        HirStatement::For {
            init,
            condition,
            increment,
            body,
        } => {
            assert!(!init.is_empty());
            assert!(!increment.is_empty());
            assert_eq!(body.len(), 1);
            assert!(matches!(condition, HirExpression::BinaryOp { .. }));
        }
        _ => panic!("Expected For statement"),
    }
}

#[test]
fn test_for_loop_without_init() {
    // for(; i < 10; i++) { }
    let condition = HirExpression::BinaryOp {
        op: BinaryOperator::LessThan,
        left: Box::new(HirExpression::Variable("i".to_string())),
        right: Box::new(HirExpression::IntLiteral(10)),
    };

    let increment = HirStatement::Assignment {
        target: "i".to_string(),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        },
    };

    let for_loop = HirStatement::For {
        init: vec![],
        condition,
        increment: vec![increment],
        body: vec![],
    };

    match for_loop {
        HirStatement::For { init, .. } => {
            assert!(init.is_empty());
        }
        _ => panic!("Expected For statement"),
    }
}

#[test]
fn test_for_loop_without_increment() {
    // for(int i = 0; i < 10; ) { i++; }
    let init = HirStatement::VariableDeclaration {
        name: "i".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(0)),
    };

    let condition = HirExpression::BinaryOp {
        op: BinaryOperator::LessThan,
        left: Box::new(HirExpression::Variable("i".to_string())),
        right: Box::new(HirExpression::IntLiteral(10)),
    };

    let body = vec![HirStatement::Assignment {
        target: "i".to_string(),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        },
    }];

    let for_loop = HirStatement::For {
        init: vec![init],
        condition,
        increment: vec![],
        body,
    };

    match for_loop {
        HirStatement::For { increment, .. } => {
            assert!(increment.is_empty());
        }
        _ => panic!("Expected For statement"),
    }
}

#[test]
fn test_for_loop_infinite() {
    // for(;;) { break; }
    let for_loop = HirStatement::For {
        init: vec![],
        condition: HirExpression::IntLiteral(1), // true in C
        increment: vec![],
        body: vec![HirStatement::Break],
    };

    match for_loop {
        HirStatement::For {
            init,
            increment,
            body,
            ..
        } => {
            assert!(init.is_empty());
            assert!(increment.is_empty());
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected For statement"),
    }
}

#[test]
fn test_nested_for_loops() {
    // for(int i = 0; i < 10; i++) {
    //     for(int j = 0; j < 10; j++) {
    //         matrix[i][j] = 0;
    //     }
    // }
    let inner_loop = HirStatement::For {
        init: vec![HirStatement::VariableDeclaration {
            name: "j".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        }],
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("j".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        },
        increment: vec![HirStatement::Assignment {
            target: "j".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("j".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
        body: vec![],
    };

    let outer_loop = HirStatement::For {
        init: vec![HirStatement::VariableDeclaration {
            name: "i".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        }],
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        },
        increment: vec![HirStatement::Assignment {
            target: "i".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
        body: vec![inner_loop],
    };

    match outer_loop {
        HirStatement::For { body, .. } => {
            assert_eq!(body.len(), 1);
            assert!(matches!(body[0], HirStatement::For { .. }));
        }
        _ => panic!("Expected For statement"),
    }
}

#[test]
fn test_for_loop_with_break_continue() {
    // for(int i = 0; i < 10; i++) {
    //     if (i == 5) break;
    //     if (i % 2 == 0) continue;
    // }
    let for_loop = HirStatement::For {
        init: vec![HirStatement::VariableDeclaration {
            name: "i".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        }],
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        },
        increment: vec![HirStatement::Assignment {
            target: "i".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
        body: vec![
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::Equal,
                    left: Box::new(HirExpression::Variable("i".to_string())),
                    right: Box::new(HirExpression::IntLiteral(5)),
                },
                then_block: vec![HirStatement::Break],
                else_block: None,
            },
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::Equal,
                    left: Box::new(HirExpression::BinaryOp {
                        op: BinaryOperator::Modulo,
                        left: Box::new(HirExpression::Variable("i".to_string())),
                        right: Box::new(HirExpression::IntLiteral(2)),
                    }),
                    right: Box::new(HirExpression::IntLiteral(0)),
                },
                then_block: vec![HirStatement::Continue],
                else_block: None,
            },
        ],
    };

    match for_loop {
        HirStatement::For { body, .. } => {
            assert_eq!(body.len(), 2);
        }
        _ => panic!("Expected For statement"),
    }
}
