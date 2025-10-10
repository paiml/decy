//! Unit tests for code generator (DECY-003 RED phase).
//!
//! These tests are intentionally failing to follow EXTREME TDD methodology.

use super::*;

#[cfg(test)]
mod tests {
    use super::*;
    use decy_hir::{HirFunction, HirParameter, HirType};

    #[test]
    fn test_generate_function_signature() {
        // RED PHASE: This test will FAIL
        let func = HirFunction::new("main".to_string(), HirType::Int, vec![]);

        let codegen = CodeGenerator::new();
        let signature = codegen.generate_signature(&func);

        assert_eq!(signature, "fn main() -> i32");
    }

    #[test]
    fn test_generate_function_with_parameters() {
        // RED PHASE: This test will FAIL
        let func = HirFunction::new(
            "add".to_string(),
            HirType::Int,
            vec![
                HirParameter::new("a".to_string(), HirType::Int),
                HirParameter::new("b".to_string(), HirType::Int),
            ],
        );

        let codegen = CodeGenerator::new();
        let signature = codegen.generate_signature(&func);

        assert_eq!(signature, "fn add(a: i32, b: i32) -> i32");
    }

    #[test]
    fn test_type_mapping_int_to_i32() {
        // RED PHASE: This test will FAIL
        assert_eq!(CodeGenerator::map_type(&HirType::Int), "i32");
    }

    #[test]
    fn test_type_mapping_float_to_f32() {
        // RED PHASE: This test will FAIL
        assert_eq!(CodeGenerator::map_type(&HirType::Float), "f32");
    }

    #[test]
    fn test_type_mapping_double_to_f64() {
        // RED PHASE: This test will FAIL
        assert_eq!(CodeGenerator::map_type(&HirType::Double), "f64");
    }

    #[test]
    fn test_type_mapping_void_to_unit() {
        // RED PHASE: This test will FAIL
        assert_eq!(CodeGenerator::map_type(&HirType::Void), "()");
    }

    #[test]
    fn test_type_mapping_char_to_u8() {
        // RED PHASE: This test will FAIL
        assert_eq!(CodeGenerator::map_type(&HirType::Char), "u8");
    }

    #[test]
    fn test_type_mapping_pointer() {
        // RED PHASE: This test will FAIL
        let ptr_type = HirType::Pointer(Box::new(HirType::Int));

        assert_eq!(CodeGenerator::map_type(&ptr_type), "*mut i32");
    }

    #[test]
    fn test_generate_void_function() {
        // RED PHASE: This test will FAIL
        let func = HirFunction::new("print_hello".to_string(), HirType::Void, vec![]);

        let codegen = CodeGenerator::new();
        let signature = codegen.generate_signature(&func);

        assert_eq!(signature, "fn print_hello()");
    }

    #[test]
    fn test_generate_complete_function() {
        // RED PHASE: This test will FAIL
        let func = HirFunction::new(
            "add".to_string(),
            HirType::Int,
            vec![
                HirParameter::new("a".to_string(), HirType::Int),
                HirParameter::new("b".to_string(), HirType::Int),
            ],
        );

        let codegen = CodeGenerator::new();
        let code = codegen.generate_function(&func);

        // Should generate a complete function with stub body
        assert!(code.contains("fn add(a: i32, b: i32) -> i32"));
        assert!(code.contains("{")); // Has body
        assert!(code.contains("}")); // Closes body
    }

    #[test]
    fn test_generate_return_statement() {
        // RED PHASE: This test will FAIL
        let codegen = CodeGenerator::new();
        let return_stmt = codegen.generate_return(&HirType::Int);

        // Should generate a default return for the type
        assert!(return_stmt.contains("return"));
        assert!(return_stmt.contains("0")); // Default i32 value
    }

    #[test]
    fn test_generate_function_with_pointer_parameter() {
        // RED PHASE: This test will FAIL
        let func = HirFunction::new(
            "process".to_string(),
            HirType::Void,
            vec![HirParameter::new(
                "data".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            )],
        );

        let codegen = CodeGenerator::new();
        let signature = codegen.generate_signature(&func);

        assert_eq!(signature, "fn process(data: *mut i32)");
    }

    #[test]
    fn test_end_to_end_simple_function() {
        // RED PHASE: This test will FAIL
        let func = HirFunction::new(
            "add".to_string(),
            HirType::Int,
            vec![
                HirParameter::new("a".to_string(), HirType::Int),
                HirParameter::new("b".to_string(), HirType::Int),
            ],
        );

        let codegen = CodeGenerator::new();
        let code = codegen.generate_function(&func);

        // Verify generated code has proper structure
        assert!(code.starts_with("fn add"));
        assert!(code.contains("a: i32"));
        assert!(code.contains("b: i32"));
        assert!(code.contains("-> i32"));

        // Should have a function body (even if stub)
        let open_braces = code.matches('{').count();
        let close_braces = code.matches('}').count();
        assert_eq!(open_braces, close_braces);
        assert!(open_braces > 0);
    }

    #[test]
    fn test_generated_code_compiles() {
        // RED PHASE: This test will FAIL
        // This is an integration test to ensure generated code is valid Rust
        let func = HirFunction::new("test_fn".to_string(), HirType::Int, vec![]);

        let codegen = CodeGenerator::new();
        let code = codegen.generate_function(&func);

        // The generated code should be syntactically valid
        // We'll verify this by checking it has all required parts
        assert!(code.contains("fn test_fn"));
        assert!(code.contains("-> i32"));
        assert!(code.contains("{"));
        assert!(code.contains("}"));
    }

    // DECY-004: Variable declaration tests (RED phase)
    #[test]
    fn test_generate_variable_declaration() {
        // RED PHASE: This test will FAIL
        use decy_hir::{HirExpression, HirStatement};

        let var_decl = HirStatement::VariableDeclaration {
            name: "x".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(5)),
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&var_decl);

        assert_eq!(code, "let mut x: i32 = 5;");
    }

    #[test]
    fn test_generate_variable_without_initializer() {
        // RED PHASE: This test will FAIL
        use decy_hir::HirStatement;

        let var_decl = HirStatement::VariableDeclaration {
            name: "y".to_string(),
            var_type: HirType::Float,
            initializer: None,
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&var_decl);

        // Uninitialized variables get default value
        assert_eq!(code, "let mut y: f32 = 0.0;");
    }

    #[test]
    fn test_generate_int_literal() {
        // RED PHASE: This test will FAIL
        use decy_hir::HirExpression;

        let expr = HirExpression::IntLiteral(42);

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        assert_eq!(code, "42");
    }

    #[test]
    fn test_generate_variable_reference() {
        // RED PHASE: This test will FAIL
        use decy_hir::HirExpression;

        let expr = HirExpression::Variable("x".to_string());

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        assert_eq!(code, "x");
    }

    #[test]
    fn test_generate_return_with_expression() {
        // RED PHASE: This test will FAIL
        use decy_hir::{HirExpression, HirStatement};

        let stmt = HirStatement::Return(Some(HirExpression::IntLiteral(0)));

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&stmt);

        assert_eq!(code, "return 0;");
    }

    #[test]
    fn test_generate_return_void() {
        // RED PHASE: This test will FAIL
        use decy_hir::HirStatement;

        let stmt = HirStatement::Return(None);

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&stmt);

        assert_eq!(code, "return;");
    }

    #[test]
    fn test_generate_function_with_body() {
        // RED PHASE: This test will FAIL
        use decy_hir::{HirExpression, HirStatement};

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

        let codegen = CodeGenerator::new();
        let code = codegen.generate_function(&func);

        assert!(code.contains("fn test() -> i32"));
        assert!(code.contains("let mut x: i32 = 5;"));
        assert!(code.contains("return x;"));
    }

    #[test]
    fn test_infer_mutability_default() {
        // RED PHASE: This test will FAIL
        // All C variables are mutable by default
        use decy_hir::{HirExpression, HirStatement};

        let var_decl = HirStatement::VariableDeclaration {
            name: "counter".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&var_decl);

        // Should generate "let mut" not "let"
        assert!(code.starts_with("let mut"));
    }

    #[test]
    fn test_end_to_end_variable_declaration() {
        // RED PHASE: This test will FAIL
        // Test complete flow: C code concept -> HIR -> Rust code
        use decy_hir::{HirExpression, HirStatement};

        // Simulates: int x = 5;
        let var_decl = HirStatement::VariableDeclaration {
            name: "x".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(5)),
        };

        let codegen = CodeGenerator::new();
        let rust_code = codegen.generate_statement(&var_decl);

        // Should produce: let mut x: i32 = 5;
        assert_eq!(rust_code, "let mut x: i32 = 5;");
    }

    // DECY-007: Binary expression generation tests (RED phase)

    #[test]
    fn test_generate_addition_expression() {
        // RED PHASE: This test will FAIL
        use decy_hir::{BinaryOperator, HirExpression};

        let expr = HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        assert_eq!(code, "a + b");
    }

    #[test]
    fn test_generate_comparison_expression() {
        // RED PHASE: This test will FAIL
        use decy_hir::{BinaryOperator, HirExpression};

        let expr = HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        assert_eq!(code, "x > 0");
    }

    #[test]
    fn test_generate_all_arithmetic_operators() {
        // RED PHASE: This test will FAIL
        use decy_hir::{BinaryOperator, HirExpression};

        let codegen = CodeGenerator::new();

        let ops_and_expected = vec![
            (BinaryOperator::Add, "a + b"),
            (BinaryOperator::Subtract, "a - b"),
            (BinaryOperator::Multiply, "a * b"),
            (BinaryOperator::Divide, "a / b"),
            (BinaryOperator::Modulo, "a % b"),
        ];

        for (op, expected) in ops_and_expected {
            let expr = HirExpression::BinaryOp {
                op,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            };

            let code = codegen.generate_expression(&expr);
            assert_eq!(code, expected);
        }
    }

    #[test]
    fn test_generate_all_comparison_operators() {
        // RED PHASE: This test will FAIL
        use decy_hir::{BinaryOperator, HirExpression};

        let codegen = CodeGenerator::new();

        let ops_and_expected = vec![
            (BinaryOperator::Equal, "a == b"),
            (BinaryOperator::NotEqual, "a != b"),
            (BinaryOperator::LessThan, "a < b"),
            (BinaryOperator::GreaterThan, "a > b"),
            (BinaryOperator::LessEqual, "a <= b"),
            (BinaryOperator::GreaterEqual, "a >= b"),
        ];

        for (op, expected) in ops_and_expected {
            let expr = HirExpression::BinaryOp {
                op,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            };

            let code = codegen.generate_expression(&expr);
            assert_eq!(code, expected);
        }
    }

    #[test]
    fn test_generate_nested_expressions() {
        // RED PHASE: This test will FAIL
        // (a + b) * c
        use decy_hir::{BinaryOperator, HirExpression};

        let expr = HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            }),
            right: Box::new(HirExpression::Variable("c".to_string())),
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        assert_eq!(code, "(a + b) * c");
    }

    #[test]
    fn test_generate_expression_in_variable_declaration() {
        // RED PHASE: This test will FAIL
        use decy_hir::{BinaryOperator, HirExpression, HirStatement};

        let var_decl = HirStatement::VariableDeclaration {
            name: "sum".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            }),
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&var_decl);

        assert_eq!(code, "let mut sum: i32 = a + b;");
    }

    #[test]
    fn test_operator_precedence_with_parentheses() {
        // RED PHASE: This test will FAIL
        // a * (b + c)
        use decy_hir::{BinaryOperator, HirExpression};

        let expr = HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("b".to_string())),
                right: Box::new(HirExpression::Variable("c".to_string())),
            }),
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        assert_eq!(code, "a * (b + c)");
    }

    // DECY-005: If/else statement generation tests (RED phase)

    #[test]
    fn test_generate_if_statement() {
        // RED PHASE: This test will FAIL
        use decy_hir::{BinaryOperator, HirExpression, HirStatement};

        let if_stmt = HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::IntLiteral(0)),
            },
            then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
            else_block: None,
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&if_stmt);

        assert!(code.contains("if x > 0"));
        assert!(code.contains("{"));
        assert!(code.contains("return 1;"));
        assert!(code.contains("}"));
    }

    #[test]
    fn test_generate_if_else_statement() {
        // RED PHASE: This test will FAIL
        use decy_hir::{BinaryOperator, HirExpression, HirStatement};

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

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&if_stmt);

        assert!(code.contains("if x == 0"));
        assert!(code.contains("} else {"));
        assert!(code.contains("return 1;"));
        assert!(code.contains("return -1;"));
    }

    #[test]
    fn test_generate_nested_if() {
        // RED PHASE: This test will FAIL
        use decy_hir::{BinaryOperator, HirExpression, HirStatement};

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

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&nested_if);

        // Should contain both if conditions
        assert!(code.contains("if x > 0"));
        assert!(code.contains("if x < 10"));
    }

    #[test]
    fn test_generate_if_with_multiple_statements() {
        // RED PHASE: This test will FAIL
        use decy_hir::{BinaryOperator, HirExpression, HirStatement};

        let if_stmt = HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::IntLiteral(0)),
            },
            then_block: vec![
                HirStatement::VariableDeclaration {
                    name: "y".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::IntLiteral(1)),
                },
                HirStatement::Return(Some(HirExpression::Variable("y".to_string()))),
            ],
            else_block: None,
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&if_stmt);

        assert!(code.contains("if x > 0"));
        assert!(code.contains("let mut y: i32 = 1;"));
        assert!(code.contains("return y;"));
    }

    #[test]
    fn test_end_to_end_if_else() {
        // RED PHASE: This test will FAIL
        // Test complete if/else in function context
        use decy_hir::{BinaryOperator, HirExpression, HirStatement};

        let func = HirFunction::new_with_body(
            "sign".to_string(),
            HirType::Int,
            vec![HirParameter::new("x".to_string(), HirType::Int)],
            vec![HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::GreaterThan,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::IntLiteral(0)),
                },
                then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
                else_block: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
                    -1,
                )))]),
            }],
        );

        let codegen = CodeGenerator::new();
        let code = codegen.generate_function(&func);

        assert!(code.contains("fn sign(x: i32) -> i32"));
        assert!(code.contains("if x > 0"));
        assert!(code.contains("} else {"));
        assert!(code.contains("return 1;"));
        assert!(code.contains("return -1;"));
    }
}
