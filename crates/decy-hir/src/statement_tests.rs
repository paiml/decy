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
}
