//! Property tests for unary operators (DECY-035 REFACTOR phase)
//!
//! These property tests verify invariants for unary operator parsing across
//! randomly generated test cases.
//!
//! References:
//! - K&R §2.10: Unary Operators
//! - ISO C99 §6.5.3: Unary operators

use decy_parser::parser::{Expression, UnaryOperator};
use decy_parser::CParser;
use proptest::prelude::*;

prop_compose! {
    /// Generate a valid C identifier (avoiding keywords)
    fn valid_identifier()(s in "[a-z][a-z0-9_]{0,10}") -> String {
        const C_KEYWORDS: &[&str] = &[
            "if", "else", "for", "while", "do", "switch", "case", "default",
            "break", "continue", "return", "goto",
            "int", "char", "float", "double", "void", "long", "short", "signed", "unsigned",
            "struct", "union", "enum", "typedef",
            "const", "volatile", "static", "extern", "auto", "register",
            "sizeof", "typeof", "asm", "inline", "restrict",
        ];

        if C_KEYWORDS.contains(&s.as_str()) {
            format!("{}_var", s)
        } else {
            s
        }
    }
}

prop_compose! {
    /// Generate unary minus expression
    fn unary_minus_expr()(var_name in valid_identifier()) -> (String, String) {
        let c_code = format!(
            r#"
            int negate(int {}) {{
                return -{};
            }}
            "#,
            var_name, var_name
        );
        (c_code, var_name)
    }
}

prop_compose! {
    /// Generate logical NOT expression
    fn logical_not_expr()(var_name in valid_identifier()) -> (String, String) {
        let c_code = format!(
            r#"
            int logical_not(int {}) {{
                return !{};
            }}
            "#,
            var_name, var_name
        );
        (c_code, var_name)
    }
}

prop_compose! {
    /// Generate bitwise NOT expression
    fn bitwise_not_expr()(var_name in valid_identifier()) -> (String, String) {
        let c_code = format!(
            r#"
            int bitwise_not(int {}) {{
                return ~{};
            }}
            "#,
            var_name, var_name
        );
        (c_code, var_name)
    }
}

prop_compose! {
    /// Generate address-of expression
    fn address_of_expr()(var_name in valid_identifier()) -> (String, String) {
        let c_code = format!(
            r#"
            int* address_of(int {}) {{
                return &{};
            }}
            "#,
            var_name, var_name
        );
        (c_code, var_name)
    }
}

proptest! {
    #[test]
    fn property_unary_minus_never_panics((c_code, _) in unary_minus_expr()) {
        let parser = CParser::new().expect("Parser creation failed");
        let _ = parser.parse(&c_code);
    }

    #[test]
    fn property_unary_minus_parses_correctly((c_code, _var_name) in unary_minus_expr()) {
        let parser = CParser::new().expect("Parser creation failed");
        let ast = parser.parse(&c_code).expect("Should parse valid C code");

        prop_assert_eq!(ast.functions().len(), 1);

        let func = &ast.functions()[0];
        if let decy_parser::parser::Statement::Return(Some(expr)) = &func.body[0] {
            prop_assert!(
                matches!(
                    expr,
                    Expression::UnaryOp {
                        op: UnaryOperator::Minus,
                        ..
                    }
                ),
                "Should be unary minus operator"
            );
        }
    }

    #[test]
    fn property_logical_not_never_panics((c_code, _) in logical_not_expr()) {
        let parser = CParser::new().expect("Parser creation failed");
        let _ = parser.parse(&c_code);
    }

    #[test]
    fn property_logical_not_parses_correctly((c_code, _var_name) in logical_not_expr()) {
        let parser = CParser::new().expect("Parser creation failed");
        let ast = parser.parse(&c_code).expect("Should parse valid C code");

        let func = &ast.functions()[0];
        if let decy_parser::parser::Statement::Return(Some(expr)) = &func.body[0] {
            prop_assert!(
                matches!(
                    expr,
                    Expression::UnaryOp {
                        op: UnaryOperator::LogicalNot,
                        ..
                    }
                ),
                "Should be logical NOT operator"
            );
        }
    }

    #[test]
    fn property_bitwise_not_never_panics((c_code, _) in bitwise_not_expr()) {
        let parser = CParser::new().expect("Parser creation failed");
        let _ = parser.parse(&c_code);
    }

    #[test]
    fn property_bitwise_not_parses_correctly((c_code, _var_name) in bitwise_not_expr()) {
        let parser = CParser::new().expect("Parser creation failed");
        let ast = parser.parse(&c_code).expect("Should parse valid C code");

        let func = &ast.functions()[0];
        if let decy_parser::parser::Statement::Return(Some(expr)) = &func.body[0] {
            prop_assert!(
                matches!(
                    expr,
                    Expression::UnaryOp {
                        op: UnaryOperator::BitwiseNot,
                        ..
                    }
                ),
                "Should be bitwise NOT operator"
            );
        }
    }

    #[test]
    fn property_address_of_never_panics((c_code, _) in address_of_expr()) {
        let parser = CParser::new().expect("Parser creation failed");
        let _ = parser.parse(&c_code);
    }

    #[test]
    fn property_address_of_parses_correctly((c_code, _var_name) in address_of_expr()) {
        let parser = CParser::new().expect("Parser creation failed");
        let ast = parser.parse(&c_code).expect("Should parse valid C code");

        let func = &ast.functions()[0];
        if let decy_parser::parser::Statement::Return(Some(expr)) = &func.body[0] {
            prop_assert!(
                matches!(
                    expr,
                    Expression::UnaryOp {
                        op: UnaryOperator::AddressOf,
                        ..
                    }
                ),
                "Should be address-of operator"
            );
        }
    }

    #[test]
    fn property_all_unary_ops_deterministic(
        _var_name in valid_identifier()
    ) {
        let c_code = r#"
            void test(int x) {
                int a = -x;
                int b = !x;
                int c = ~x;
            }
            "#.to_string();

        let parser1 = CParser::new().expect("Parser creation failed");
        let parser2 = CParser::new().expect("Parser creation failed");

        let ast1 = parser1.parse(&c_code);
        let ast2 = parser2.parse(&c_code);

        prop_assert_eq!(ast1.is_ok(), ast2.is_ok(), "Parsing should be deterministic");

        if let (Ok(ast1), Ok(ast2)) = (ast1, ast2) {
            prop_assert_eq!(
                ast1.functions().len(),
                ast2.functions().len(),
                "Should parse same number of functions"
            );
        }
    }

    #[test]
    fn property_unary_operators_preserve_operand_type(
        var_name in valid_identifier()
    ) {
        // Test that unary operators work with variable operands
        let c_code = format!(
            r#"
            int test(int {}) {{
                return -{};
            }}
            "#,
            var_name, var_name
        );

        let parser = CParser::new().expect("Parser creation failed");
        let ast = parser.parse(&c_code).expect("Should parse");

        let func = &ast.functions()[0];
        if let decy_parser::parser::Statement::Return(Some(Expression::UnaryOp { operand, .. })) = &func.body[0] {
            // Operand should be a variable
            prop_assert!(
                matches!(**operand, Expression::Variable(_)),
                "Operand should be preserved as variable"
            );
        }
    }

    #[test]
    fn property_nested_unary_ops(
        var_name in valid_identifier()
    ) {
        // Test nested unary operations: !!x
        let c_code = format!(
            r#"
            int test(int {}) {{
                return !!{};
            }}
            "#,
            var_name, var_name
        );

        let parser = CParser::new().expect("Parser creation failed");
        let ast = parser.parse(&c_code).expect("Should parse nested unary ops");

        let func = &ast.functions()[0];
        if let decy_parser::parser::Statement::Return(Some(Expression::UnaryOp { op, operand })) = &func.body[0] {
            prop_assert_eq!(*op, UnaryOperator::LogicalNot, "Outer should be !");

            // Inner should also be UnaryOp
            prop_assert!(
                matches!(**operand, Expression::UnaryOp { .. }),
                "Inner should also be UnaryOp"
            );
        }
    }
}
