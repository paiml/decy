//! Unit tests for code generator (DECY-003 RED phase).
//!
//! These tests are intentionally failing to follow EXTREME TDD methodology.

use super::*;

#[cfg(test)]
mod tests {
    use super::*;
    use decy_hir::{
        BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType,
    };

    #[test]
    fn test_generate_function_signature() {
        // DECY-AUDIT-001: Updated for main function special handling
        // main with int return type becomes fn main() (no return type)
        let func = HirFunction::new("main".to_string(), HirType::Int, vec![]);

        let codegen = CodeGenerator::new();
        let signature = codegen.generate_signature(&func);

        assert_eq!(signature, "fn main()");
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

        assert_eq!(signature, "fn add(mut a: i32, mut b: i32) -> i32");
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
        assert!(code.contains("fn add(mut a: i32, mut b: i32) -> i32"));
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

        // DECY-111: Pointer params now become references
        // Since this function has no body (read-only), it gets immutable ref
        assert_eq!(signature, "fn process(data: &i32)");
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

        // Uninitialized variables get default value with type suffix
        assert_eq!(code, "let mut y: f32 = 0.0f32;");
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

        assert!(code.contains("fn sign(mut x: i32) -> i32"));
        assert!(code.contains("if x > 0"));
        assert!(code.contains("} else {"));
        assert!(code.contains("return 1;"));
        assert!(code.contains("return -1;"));
    }

    // DECY-006: While loop tests (RED phase)

    #[test]
    fn test_generate_while_loop() {
        // RED PHASE: This test will FAIL
        let while_stmt = HirStatement::While {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::IntLiteral(10)),
            },
            body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&while_stmt);

        assert!(code.contains("while x < 10"));
        assert!(code.contains("{"));
        assert!(code.contains("return 1;"));
        assert!(code.contains("}"));
    }

    #[test]
    fn test_generate_break_statement() {
        // RED PHASE: This test will FAIL
        let break_stmt = HirStatement::Break;

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&break_stmt);

        assert_eq!(code, "break;");
    }

    #[test]
    fn test_generate_continue_statement() {
        // RED PHASE: This test will FAIL
        let continue_stmt = HirStatement::Continue;

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&continue_stmt);

        assert_eq!(code, "continue;");
    }

    #[test]
    fn test_generate_while_with_break() {
        // RED PHASE: This test will FAIL
        let while_stmt = HirStatement::While {
            condition: HirExpression::IntLiteral(1),
            body: vec![
                HirStatement::VariableDeclaration {
                    name: "x".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::IntLiteral(5)),
                },
                HirStatement::Break,
            ],
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&while_stmt);

        assert!(code.contains("while 1"));
        assert!(code.contains("let mut x: i32 = 5;"));
        assert!(code.contains("break;"));
    }

    #[test]
    fn test_end_to_end_while_loop() {
        // RED PHASE: This test will FAIL
        let func = HirFunction::new_with_body(
            "count_to_ten".to_string(),
            HirType::Void,
            vec![],
            vec![
                HirStatement::VariableDeclaration {
                    name: "i".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::IntLiteral(0)),
                },
                HirStatement::While {
                    condition: HirExpression::BinaryOp {
                        op: BinaryOperator::LessThan,
                        left: Box::new(HirExpression::Variable("i".to_string())),
                        right: Box::new(HirExpression::IntLiteral(10)),
                    },
                    body: vec![HirStatement::Continue],
                },
            ],
        );

        let codegen = CodeGenerator::new();
        let code = codegen.generate_function(&func);

        assert!(code.contains("fn count_to_ten()"));
        assert!(code.contains("let mut i: i32 = 0;"));
        assert!(code.contains("while i < 10"));
        assert!(code.contains("continue;"));
    }

    // DECY-008: Pointer operation tests (RED phase)

    #[test]
    fn test_generate_dereference() {
        // RED PHASE: This test will FAIL
        use decy_hir::HirExpression;

        let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("ptr".to_string())));

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        assert_eq!(code, "*ptr");
    }

    #[test]
    fn test_generate_address_of() {
        // RED PHASE: This test will FAIL
        use decy_hir::HirExpression;

        let expr = HirExpression::AddressOf(Box::new(HirExpression::Variable("x".to_string())));

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        assert_eq!(code, "&x");
    }

    #[test]
    fn test_generate_nested_dereference() {
        // RED PHASE: This test will FAIL
        use decy_hir::HirExpression;

        // **ptr_ptr
        let expr = HirExpression::Dereference(Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("ptr_ptr".to_string()),
        ))));

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        assert_eq!(code, "**ptr_ptr");
    }

    #[test]
    fn test_generate_address_of_dereference() {
        // RED PHASE: This test will FAIL
        use decy_hir::HirExpression;

        // &(*ptr)
        let expr = HirExpression::AddressOf(Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("ptr".to_string()),
        ))));

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        assert_eq!(code, "&(*ptr)");
    }

    #[test]
    fn test_generate_pointer_in_variable_declaration() {
        // RED PHASE: This test will FAIL
        use decy_hir::{HirExpression, HirStatement};

        let var_decl = HirStatement::VariableDeclaration {
            name: "val".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::Dereference(Box::new(
                HirExpression::Variable("ptr".to_string()),
            ))),
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&var_decl);

        assert_eq!(code, "let mut val: i32 = *ptr;");
    }

    // DECY-009: Function call expression tests (RED phase)

    #[test]
    fn test_generate_function_call_no_args() {
        // RED PHASE: This test will FAIL
        use decy_hir::HirExpression;

        let expr = HirExpression::FunctionCall {
            function: "foo".to_string(),
            arguments: vec![],
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        assert_eq!(code, "foo()");
    }

    #[test]
    fn test_generate_function_call_one_arg() {
        // RED PHASE: This test will FAIL
        use decy_hir::HirExpression;

        let expr = HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::IntLiteral(10)],
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        assert_eq!(code, "malloc(10)");
    }

    #[test]
    fn test_generate_function_call_multiple_args() {
        // RED PHASE: This test will FAIL
        use decy_hir::HirExpression;

        let expr = HirExpression::FunctionCall {
            function: "add".to_string(),
            arguments: vec![
                HirExpression::Variable("x".to_string()),
                HirExpression::Variable("y".to_string()),
            ],
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        assert_eq!(code, "add(x, y)");
    }

    #[test]
    fn test_generate_nested_function_call() {
        // RED PHASE: This test will FAIL
        // outer(inner())
        use decy_hir::HirExpression;

        let expr = HirExpression::FunctionCall {
            function: "outer".to_string(),
            arguments: vec![HirExpression::FunctionCall {
                function: "inner".to_string(),
                arguments: vec![],
            }],
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        assert_eq!(code, "outer(inner())");
    }

    #[test]
    fn test_generate_function_call_in_variable_declaration() {
        // RED PHASE: This test will FAIL
        use decy_hir::{HirExpression, HirStatement};

        let var_decl = HirStatement::VariableDeclaration {
            name: "ptr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(40)],
            }),
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&var_decl);

        assert_eq!(code, "let mut ptr: *mut i32 = malloc(40);");
    }

    // DECY-009 Phase 2: Assignment statement tests (RED phase)

    #[test]
    fn test_generate_assignment_simple() {
        // RED PHASE: This test will FAIL
        use decy_hir::{HirExpression, HirStatement};

        let assign_stmt = HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::IntLiteral(42),
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&assign_stmt);

        assert_eq!(code, "x = 42;");
    }

    #[test]
    fn test_generate_assignment_with_variable() {
        // RED PHASE: This test will FAIL
        use decy_hir::{HirExpression, HirStatement};

        let assign_stmt = HirStatement::Assignment {
            target: "result".to_string(),
            value: HirExpression::Variable("temp".to_string()),
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&assign_stmt);

        assert_eq!(code, "result = temp;");
    }

    #[test]
    fn test_generate_assignment_with_malloc() {
        // RED PHASE: This test will FAIL
        // ptr = malloc(100)
        use decy_hir::{HirExpression, HirStatement};

        let assign_stmt = HirStatement::Assignment {
            target: "ptr".to_string(),
            value: HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(100)],
            },
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&assign_stmt);

        assert_eq!(code, "ptr = malloc(100);");
    }

    #[test]
    fn test_generate_assignment_with_expression() {
        // RED PHASE: This test will FAIL
        // x = a + b
        use decy_hir::{BinaryOperator, HirExpression, HirStatement};

        let assign_stmt = HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            },
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&assign_stmt);

        assert_eq!(code, "x = a + b;");
    }

    #[test]
    fn test_generate_assignment_with_dereference() {
        // RED PHASE: This test will FAIL
        // *ptr = 5
        use decy_hir::{HirExpression, HirStatement};

        let assign_stmt = HirStatement::Assignment {
            target: "ptr".to_string(),
            value: HirExpression::Dereference(Box::new(HirExpression::Variable(
                "other_ptr".to_string(),
            ))),
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&assign_stmt);

        assert_eq!(code, "ptr = *other_ptr;");
    }

    #[test]
    fn test_generate_assignment_in_function() {
        // RED PHASE: This test will FAIL
        use decy_hir::{HirExpression, HirStatement};

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                HirStatement::VariableDeclaration {
                    name: "x".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::IntLiteral(0)),
                },
                HirStatement::Assignment {
                    target: "x".to_string(),
                    value: HirExpression::IntLiteral(42),
                },
            ],
        );

        let codegen = CodeGenerator::new();
        let code = codegen.generate_function(&func);

        assert!(code.contains("fn test()"));
        assert!(code.contains("let mut x: i32 = 0;"));
        assert!(code.contains("x = 42;"));
    }
}

#[test]
fn test_type_mapping_box_int() {
    let box_type = HirType::Box(Box::new(HirType::Int));
    assert_eq!(CodeGenerator::map_type(&box_type), "Box<i32>");
}

#[test]
fn test_type_mapping_box_char() {
    let box_type = HirType::Box(Box::new(HirType::Char));
    assert_eq!(CodeGenerator::map_type(&box_type), "Box<u8>");
}

#[test]
fn test_generate_box_variable_declaration() {
    use decy_analyzer::patterns::PatternDetector;

    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "ptr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(100)],
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let detector = PatternDetector::new();
    let candidates = detector.find_box_candidates(&func);
    let code = codegen.generate_function_with_box_transform(&func, &candidates);

    assert!(code.contains("Box<i32>"));
    assert!(code.contains("Box::new"));
    assert!(!code.contains("*mut"));
}

// DECY-019: Vec code generation tests (RED phase)
// These tests are disabled until HirType::Vec and Vec transform methods are implemented

#[test]
#[ignore = "RED phase - waiting for HirType::Vec implementation"]
fn test_type_mapping_vec_int() {
    // RED PHASE: Waiting for full implementation
    // Test that HirType::Vec maps to Vec<T>
    // Uncomment when ready:
    // use decy_hir::HirType;
    // let vec_type = HirType::Vec(Box::new(HirType::Int));
    // assert_eq!(CodeGenerator::map_type(&vec_type), "Vec<i32>");
}

#[test]
#[ignore = "RED phase - waiting for HirType::Vec implementation"]
fn test_type_mapping_vec_char() {
    // RED PHASE: Waiting for full implementation
    // Uncomment when ready:
    // use decy_hir::HirType;
    // let vec_type = HirType::Vec(Box::new(HirType::Char));
    // assert_eq!(CodeGenerator::map_type(&vec_type), "Vec<u8>");
}

#[test]
#[ignore = "RED phase - waiting for HirType::Vec implementation"]
fn test_type_mapping_vec_double() {
    // RED PHASE: Waiting for full implementation
    // Uncomment when ready:
    // use decy_hir::HirType;
    // let vec_type = HirType::Vec(Box::new(HirType::Double));
    // assert_eq!(CodeGenerator::map_type(&vec_type), "Vec<f64>");
}

#[test]
#[ignore = "RED phase - waiting for generate_function_with_vec_transform implementation"]
fn test_generate_vec_with_capacity_literal() {
    // RED PHASE: Waiting for full implementation
    // Pattern: int* arr = malloc(10 * sizeof(int)); → let mut arr: Vec<i32> = Vec::with_capacity(10);
    // Uncomment when ready:
    /*
    use decy_analyzer::patterns::PatternDetector;

    let capacity = HirExpression::IntLiteral(10);
    let sizeof_expr = HirExpression::IntLiteral(4); // sizeof(int)
    let size_expr = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(capacity),
        right: Box::new(sizeof_expr),
    };

    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![size_expr],
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let detector = PatternDetector::new();
    let candidates = detector.find_vec_candidates(&func);
    let code = codegen.generate_function_with_vec_transform(&func, &candidates);

    // Should transform to Vec<i32> with capacity
    assert!(code.contains("Vec<i32>"));
    assert!(code.contains("Vec::with_capacity(10)"));
    assert!(!code.contains("*mut"));
    assert!(!code.contains("malloc"));
    */
}

#[test]
#[ignore = "RED phase - waiting for full implementation"]
fn test_generate_vec_with_capacity_variable() {
    // RED PHASE: This test will FAIL
    // Pattern: int* arr = malloc(n * sizeof(int)); → let mut arr: Vec<i32> = Vec::with_capacity(n);
    /*
    use decy_analyzer::patterns::PatternDetector;

    let capacity = HirExpression::Variable("n".to_string());
    let sizeof_expr = HirExpression::IntLiteral(4);
    let size_expr = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(capacity),
        right: Box::new(sizeof_expr),
    };

    let func = HirFunction::new_with_body(
        "allocate_array".to_string(),
        HirType::Void,
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![size_expr],
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let detector = PatternDetector::new();
    let candidates = detector.find_vec_candidates(&func);
    let code = codegen.generate_function_with_vec_transform(&func, &candidates);

    assert!(code.contains("Vec<i32>"));
    assert!(code.contains("Vec::with_capacity(n)"));
    assert!(!code.contains("*mut"));
    */
}

#[test]
fn test_generate_multiple_vec_allocations() {
    // RED PHASE: This test will FAIL
    // Test that multiple Vec allocations are all transformed correctly
    use decy_analyzer::patterns::PatternDetector;

    let size1 = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::IntLiteral(5)),
        right: Box::new(HirExpression::IntLiteral(4)),
    };

    let size2 = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::IntLiteral(20)),
        right: Box::new(HirExpression::IntLiteral(8)),
    };

    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "arr1".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![size1],
                }),
            },
            HirStatement::VariableDeclaration {
                name: "arr2".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Double)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![size2],
                }),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let detector = PatternDetector::new();
    let candidates = detector.find_vec_candidates(&func);
    let code = codegen.generate_function_with_vec_transform(&func, &candidates);

    assert!(code.contains("arr1"));
    assert!(code.contains("Vec<i32>"));
    assert!(code.contains("Vec::with_capacity(5)"));
    assert!(code.contains("arr2"));
    assert!(code.contains("Vec<f64>"));
    assert!(code.contains("Vec::with_capacity(20)"));
}

#[test]
fn test_vec_and_box_together() {
    // RED PHASE: This test will FAIL
    // Test that Vec and Box transformations coexist in the same function
    use decy_analyzer::patterns::PatternDetector;

    let vec_size = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::IntLiteral(10)),
        right: Box::new(HirExpression::IntLiteral(4)),
    };

    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            // Box candidate: single element
            HirStatement::VariableDeclaration {
                name: "single".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::IntLiteral(4)],
                }),
            },
            // Vec candidate: array
            HirStatement::VariableDeclaration {
                name: "array".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![vec_size],
                }),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let detector = PatternDetector::new();
    let box_candidates = detector.find_box_candidates(&func);
    let vec_candidates = detector.find_vec_candidates(&func);

    // Apply both transformations
    let code = codegen.generate_function_with_box_and_vec_transform(
        &func,
        &box_candidates,
        &vec_candidates,
    );

    // Box transformation
    assert!(code.contains("single"));
    assert!(code.contains("Box<i32>"));

    // Vec transformation
    assert!(code.contains("array"));
    assert!(code.contains("Vec<i32>"));
    assert!(code.contains("Vec::with_capacity(10)"));
}

#[test]
fn test_vec_type_inference_from_pointer() {
    // RED PHASE: This test will FAIL
    // Test that element type is correctly inferred from pointer type
    use decy_analyzer::patterns::PatternDetector;

    let test_cases = vec![
        (HirType::Int, "Vec<i32>"),
        (HirType::Char, "Vec<u8>"),
        (HirType::Float, "Vec<f32>"),
        (HirType::Double, "Vec<f64>"),
    ];

    for (element_type, expected_vec_type) in test_cases {
        let size_expr = HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::IntLiteral(10)),
            right: Box::new(HirExpression::IntLiteral(4)),
        };

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Pointer(Box::new(element_type)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![size_expr],
                }),
            }],
        );

        let codegen = CodeGenerator::new();
        let detector = PatternDetector::new();
        let candidates = detector.find_vec_candidates(&func);
        let code = codegen.generate_function_with_vec_transform(&func, &candidates);

        assert!(
            code.contains(expected_vec_type),
            "Expected {} in generated code, got: {}",
            expected_vec_type,
            code
        );
    }
}

#[test]
fn test_vec_no_transform_for_non_malloc() {
    // RED PHASE: This test will FAIL
    // Test that non-malloc allocations are not transformed
    use decy_analyzer::patterns::PatternDetector;

    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "custom_alloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(40)],
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let detector = PatternDetector::new();
    let candidates = detector.find_vec_candidates(&func);

    // Should find no Vec candidates
    assert_eq!(candidates.len(), 0);

    let code = codegen.generate_function_with_vec_transform(&func, &candidates);

    // Should keep raw pointer since it's not a malloc pattern
    assert!(code.contains("*mut i32"));
    assert!(!code.contains("Vec<"));
}

// DECY-AUDIT-001: Main function signature should be () not -> i32 (Gemini audit finding)
#[test]
fn test_main_function_special_signature() {
    // DECY-AUDIT-001 RED PHASE: This test will FAIL
    // C's int main() should become Rust's fn main() (not fn main() -> i32)
    let func = HirFunction::new("main".to_string(), HirType::Int, vec![]);

    let codegen = CodeGenerator::new();
    let signature = codegen.generate_signature(&func);

    // Rust's entry point must be fn main(), NOT fn main() -> i32
    assert_eq!(signature, "fn main()");
}

#[test]
fn test_main_function_with_return_becomes_exit() {
    // DECY-AUDIT-001 RED PHASE: This test will FAIL
    // return N in main should become std::process::exit(N)
    use decy_hir::{HirExpression, HirStatement};

    let func = HirFunction::new_with_body(
        "main".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    assert!(code.contains("fn main()")); // No return type
    assert!(!code.contains("-> i32")); // Must NOT have -> i32
    assert!(code.contains("std::process::exit(0)")); // return 0 becomes exit
}
