/// Expression Statement Tests
/// Tests for DECY-065: Expression statement support in HIR
///
/// Tests that function calls and other expressions can be used as statements,
/// which is required for printf(), free(), and other side-effect functions.

use decy_hir::{HirExpression, HirStatement};

#[test]
fn test_hir_expression_statement_function_call() {
    // Test creating an expression statement with a function call
    // C: printf("Hello");
    // HIR: Expression(FunctionCall { function: "printf", arguments: [...] })

    let printf_call = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![HirExpression::StringLiteral("Hello".to_string())],
    };

    let stmt = HirStatement::Expression(printf_call);

    match stmt {
        HirStatement::Expression(expr) => {
            assert!(matches!(expr, HirExpression::FunctionCall { .. }));
        }
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_hir_parser_function_call_to_expression_statement() {
    // Test parser AST → HIR conversion for function call statements
    // This will verify that Statement::FunctionCall converts properly

    use decy_parser::parser::{Expression, Statement};

    let ast_stmt = Statement::FunctionCall {
        function: "free".to_string(),
        arguments: vec![Expression::Variable("ptr".to_string())],
    };

    let hir_stmt = HirStatement::from_ast_statement(&ast_stmt);

    // Should be an Expression statement, not Break
    match hir_stmt {
        HirStatement::Expression(HirExpression::FunctionCall { function, arguments }) => {
            assert_eq!(function, "free");
            assert_eq!(arguments.len(), 1);
        }
        _ => panic!(
            "Expected Expression(FunctionCall), got {:?}",
            hir_stmt
        ),
    }
}

#[test]
fn test_hir_expression_statement_with_return_value_discarded() {
    // Test that function calls with return values can be used as statements
    // C: strlen(s); // return value discarded
    // HIR: Expression(FunctionCall { ... })

    let strlen_call = HirExpression::FunctionCall {
        function: "strlen".to_string(),
        arguments: vec![HirExpression::Variable("s".to_string())],
    };

    let stmt = HirStatement::Expression(strlen_call);

    assert!(matches!(stmt, HirStatement::Expression(_)));
}
