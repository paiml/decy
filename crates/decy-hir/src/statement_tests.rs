//! Unit tests for HIR statements (DECY-004 RED phase).
//!
//! These tests are intentionally failing to follow EXTREME TDD methodology.

use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hir_variable_declaration_creation() {
        // RED PHASE: This test will FAIL until we define HirStatement
        let var_decl = HirStatement::VariableDeclaration {
            name: "x".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(5)),
        };

        if let HirStatement::VariableDeclaration {
            name,
            var_type,
            initializer,
        } = &var_decl
        {
            assert_eq!(name, "x");
            assert_eq!(var_type, &HirType::Int);
            assert!(initializer.is_some());
        } else {
            panic!("Expected VariableDeclaration");
        }
    }

    #[test]
    fn test_hir_variable_declaration_without_initializer() {
        // RED PHASE: This test will FAIL
        let var_decl = HirStatement::VariableDeclaration {
            name: "y".to_string(),
            var_type: HirType::Float,
            initializer: None,
        };

        if let HirStatement::VariableDeclaration {
            name,
            var_type,
            initializer,
        } = &var_decl
        {
            assert_eq!(name, "y");
            assert_eq!(var_type, &HirType::Float);
            assert!(initializer.is_none());
        } else {
            panic!("Expected VariableDeclaration");
        }
    }

    #[test]
    fn test_hir_expression_int_literal() {
        // RED PHASE: This test will FAIL
        let expr = HirExpression::IntLiteral(42);

        if let HirExpression::IntLiteral(val) = expr {
            assert_eq!(val, 42);
        } else {
            panic!("Expected IntLiteral");
        }
    }

    #[test]
    fn test_hir_expression_variable_reference() {
        // RED PHASE: This test will FAIL
        let expr = HirExpression::Variable("x".to_string());

        if let HirExpression::Variable(name) = &expr {
            assert_eq!(name, "x");
        } else {
            panic!("Expected Variable");
        }
    }

    #[test]
    fn test_hir_function_with_body() {
        // RED PHASE: This test will FAIL
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
                HirStatement::Return(Some(HirExpression::Variable("x".to_string()))),
            ],
        );

        assert_eq!(func.name(), "test");
        assert_eq!(func.body().len(), 2);
    }

    #[test]
    fn test_hir_return_statement() {
        // RED PHASE: This test will FAIL
        let stmt = HirStatement::Return(Some(HirExpression::IntLiteral(0)));

        if let HirStatement::Return(expr) = &stmt {
            assert!(expr.is_some());
        } else {
            panic!("Expected Return statement");
        }
    }

    #[test]
    fn test_hir_return_void() {
        // RED PHASE: This test will FAIL
        let stmt = HirStatement::Return(None);

        if let HirStatement::Return(expr) = &stmt {
            assert!(expr.is_none());
        } else {
            panic!("Expected Return statement");
        }
    }

    #[test]
    fn test_variable_declaration_clone() {
        // RED PHASE: This test will FAIL
        let var_decl = HirStatement::VariableDeclaration {
            name: "x".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(5)),
        };

        let cloned = var_decl.clone();
        assert_eq!(var_decl, cloned);
    }

    #[test]
    fn test_expression_clone() {
        // RED PHASE: This test will FAIL
        let expr = HirExpression::IntLiteral(42);
        let cloned = expr.clone();
        assert_eq!(expr, cloned);
    }

    #[test]
    fn test_statement_debug() {
        // RED PHASE: This test will FAIL
        let stmt = HirStatement::Return(Some(HirExpression::IntLiteral(0)));
        let debug_str = format!("{:?}", stmt);
        assert!(debug_str.contains("Return"));
    }

    #[test]
    fn test_variable_with_pointer_type() {
        // RED PHASE: This test will FAIL
        let var_decl = HirStatement::VariableDeclaration {
            name: "ptr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: None,
        };

        if let HirStatement::VariableDeclaration { name, var_type, .. } = &var_decl {
            assert_eq!(name, "ptr");
            assert!(matches!(var_type, HirType::Pointer(_)));
        } else {
            panic!("Expected VariableDeclaration");
        }
    }

    // DECY-007: Binary expression tests (RED phase)

    #[test]
    fn test_binary_expression_addition() {
        // RED PHASE: This test will FAIL
        let expr = HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        };

        if let HirExpression::BinaryOp { op, left, right } = &expr {
            assert!(matches!(op, BinaryOperator::Add));
            assert!(matches!(**left, HirExpression::Variable(_)));
            assert!(matches!(**right, HirExpression::Variable(_)));
        } else {
            panic!("Expected BinaryOp");
        }
    }

    #[test]
    fn test_binary_expression_comparison() {
        // RED PHASE: This test will FAIL
        let expr = HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        };

        if let HirExpression::BinaryOp { op, .. } = &expr {
            assert!(matches!(op, BinaryOperator::GreaterThan));
        } else {
            panic!("Expected BinaryOp");
        }
    }

    #[test]
    fn test_binary_operator_variants() {
        // RED PHASE: This test will FAIL
        let operators = [
            BinaryOperator::Add,
            BinaryOperator::Subtract,
            BinaryOperator::Multiply,
            BinaryOperator::Divide,
            BinaryOperator::Modulo,
            BinaryOperator::Equal,
            BinaryOperator::NotEqual,
            BinaryOperator::LessThan,
            BinaryOperator::GreaterThan,
            BinaryOperator::LessEqual,
            BinaryOperator::GreaterEqual,
        ];

        assert_eq!(operators.len(), 11);
    }

    #[test]
    fn test_nested_binary_expressions() {
        // RED PHASE: This test will FAIL
        // (a + b) * c
        let expr = HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            }),
            right: Box::new(HirExpression::Variable("c".to_string())),
        };

        if let HirExpression::BinaryOp { op, left, .. } = &expr {
            assert!(matches!(op, BinaryOperator::Multiply));
            assert!(matches!(**left, HirExpression::BinaryOp { .. }));
        } else {
            panic!("Expected BinaryOp");
        }
    }

    #[test]
    fn test_binary_expression_clone() {
        // RED PHASE: This test will FAIL
        let expr = HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::IntLiteral(1)),
            right: Box::new(HirExpression::IntLiteral(2)),
        };

        let cloned = expr.clone();
        assert_eq!(expr, cloned);
    }

    // DECY-005: If/else statement tests (RED phase)

    #[test]
    fn test_hir_if_statement_creation() {
        // RED PHASE: This test will FAIL
        let if_stmt = HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::IntLiteral(0)),
            },
            then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
            else_block: None,
        };

        if let HirStatement::If {
            condition,
            then_block,
            else_block,
        } = &if_stmt
        {
            assert!(matches!(condition, HirExpression::BinaryOp { .. }));
            assert_eq!(then_block.len(), 1);
            assert!(else_block.is_none());
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn test_hir_if_else_statement() {
        // RED PHASE: This test will FAIL
        let if_stmt = HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::Equal,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::IntLiteral(0)),
            },
            then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
            else_block: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
                -1,
            )))]),
        };

        if let HirStatement::If {
            condition,
            then_block,
            else_block,
        } = &if_stmt
        {
            assert!(matches!(condition, HirExpression::BinaryOp { .. }));
            assert_eq!(then_block.len(), 1);
            assert!(else_block.is_some());
            assert_eq!(else_block.as_ref().unwrap().len(), 1);
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn test_nested_if_statements() {
        // RED PHASE: This test will FAIL
        let nested_if = HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::IntLiteral(0)),
            },
            then_block: vec![HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::LessThan,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::IntLiteral(10)),
                },
                then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
                else_block: None,
            }],
            else_block: None,
        };

        if let HirStatement::If { then_block, .. } = &nested_if {
            assert_eq!(then_block.len(), 1);
            assert!(matches!(then_block[0], HirStatement::If { .. }));
        } else {
            panic!("Expected If statement");
        }
    }

    #[test]
    fn test_if_statement_clone() {
        // RED PHASE: This test will FAIL
        let if_stmt = HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::IntLiteral(0)),
            },
            then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
            else_block: None,
        };

        let cloned = if_stmt.clone();
        assert_eq!(if_stmt, cloned);
    }

    #[test]
    fn test_if_statement_debug() {
        // RED PHASE: This test will FAIL
        let if_stmt = HirStatement::If {
            condition: HirExpression::Variable("x".to_string()),
            then_block: vec![],
            else_block: None,
        };

        let debug_str = format!("{:?}", if_stmt);
        assert!(debug_str.contains("If"));
    }

    // DECY-006: While loop tests (RED phase)

    #[test]
    fn test_hir_while_loop_creation() {
        // RED PHASE: This test will FAIL
        let while_stmt = HirStatement::While {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::IntLiteral(10)),
            },
            body: vec![HirStatement::VariableDeclaration {
                name: "i".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            }],
        };

        if let HirStatement::While { condition, body } = &while_stmt {
            assert!(matches!(condition, HirExpression::BinaryOp { .. }));
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected While statement");
        }
    }

    #[test]
    fn test_hir_break_statement() {
        // RED PHASE: This test will FAIL
        let break_stmt = HirStatement::Break;

        assert!(matches!(break_stmt, HirStatement::Break));
    }

    #[test]
    fn test_hir_continue_statement() {
        // RED PHASE: This test will FAIL
        let continue_stmt = HirStatement::Continue;

        assert!(matches!(continue_stmt, HirStatement::Continue));
    }

    #[test]
    fn test_while_with_break() {
        // RED PHASE: This test will FAIL
        let while_stmt = HirStatement::While {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::Equal,
                left: Box::new(HirExpression::IntLiteral(1)),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
            body: vec![HirStatement::Break],
        };

        if let HirStatement::While { body, .. } = &while_stmt {
            assert_eq!(body.len(), 1);
            assert!(matches!(body[0], HirStatement::Break));
        } else {
            panic!("Expected While statement");
        }
    }

    #[test]
    fn test_while_statement_clone() {
        // RED PHASE: This test will FAIL
        let while_stmt = HirStatement::While {
            condition: HirExpression::Variable("x".to_string()),
            body: vec![HirStatement::Continue],
        };

        let cloned = while_stmt.clone();
        assert_eq!(while_stmt, cloned);
    }

    // DECY-008: Pointer operation tests (RED phase)

    #[test]
    fn test_dereference_expression() {
        // RED PHASE: This test will FAIL
        let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("ptr".to_string())));

        if let HirExpression::Dereference(inner) = &expr {
            assert!(matches!(**inner, HirExpression::Variable(_)));
        } else {
            panic!("Expected Dereference");
        }
    }

    #[test]
    fn test_address_of_expression() {
        // RED PHASE: This test will FAIL
        let expr = HirExpression::AddressOf(Box::new(HirExpression::Variable("x".to_string())));

        if let HirExpression::AddressOf(inner) = &expr {
            assert!(matches!(**inner, HirExpression::Variable(_)));
        } else {
            panic!("Expected AddressOf");
        }
    }

    #[test]
    fn test_dereference_clone() {
        // RED PHASE: This test will FAIL
        let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("ptr".to_string())));
        let cloned = expr.clone();
        assert_eq!(expr, cloned);
    }

    #[test]
    fn test_address_of_clone() {
        // RED PHASE: This test will FAIL
        let expr = HirExpression::AddressOf(Box::new(HirExpression::Variable("x".to_string())));
        let cloned = expr.clone();
        assert_eq!(expr, cloned);
    }

    #[test]
    fn test_nested_dereference() {
        // RED PHASE: This test will FAIL
        // **ptr_ptr (dereference twice)
        let expr = HirExpression::Dereference(Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("ptr_ptr".to_string()),
        ))));

        if let HirExpression::Dereference(inner) = &expr {
            assert!(matches!(**inner, HirExpression::Dereference(_)));
        } else {
            panic!("Expected nested Dereference");
        }
    }
}
