//! Property tests for pointer field access parsing (DECY-034 REFACTOR phase)
//!
//! These property tests verify invariants for ptr->field parsing across
//! randomly generated test cases.
//!
//! References:
//! - K&R §6.2: Structures and Functions
//! - ISO C99 §6.5.2.3: Structure and union members

use decy_parser::parser::Expression;
use decy_parser::CParser;
use proptest::prelude::*;

prop_compose! {
    /// Generate a valid C identifier (struct/field/variable name)
    fn valid_identifier()(s in "[a-z][a-z0-9_]{0,10}") -> String {
        s
    }
}

prop_compose! {
    /// Generate a pointer field access expression
    fn pointer_field_access()(
        var_name in valid_identifier(),
        field_name in valid_identifier(),
    ) -> (String, String, String) {
        let c_code = format!(
            r#"
            struct Data {{
                int {};
            }};

            int test(struct Data* {}) {{
                return {}->FIELD;
            }}
            "#,
            field_name,
            var_name,
            var_name
        ).replace("FIELD", &field_name);

        (c_code, var_name, field_name)
    }
}

proptest! {
    #[test]
    fn property_pointer_field_access_never_panics(
        (c_code, _, _) in pointer_field_access()
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        // Should never panic, even if parsing fails
        let _ = parser.parse(&c_code);
    }

    #[test]
    fn property_pointer_field_access_parses_correctly(
        (c_code, _var_name, field_name) in pointer_field_access()
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let ast = parser.parse(&c_code).expect("Should parse valid C code");

        prop_assert_eq!(ast.functions().len(), 1, "Should parse one function");

        let func = &ast.functions()[0];
        prop_assert!(!func.body.is_empty(), "Function should have body");

        // Check that we have a return statement with pointer field access
        if let decy_parser::parser::Statement::Return(Some(expr)) = &func.body[0] {
            match expr {
                Expression::PointerFieldAccess { pointer, field } => {
                    prop_assert!(
                        matches!(**pointer, Expression::Variable(_)),
                        "Pointer should be a variable"
                    );
                    prop_assert_eq!(field, &field_name, "Field name should match");
                }
                _ => return Err(proptest::test_runner::TestCaseError::fail(
                    "Expected PointerFieldAccess"
                )),
            }
        }
    }

    #[test]
    fn property_field_names_preserved(
        (c_code, _, field_name) in pointer_field_access()
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let ast = parser.parse(&c_code).expect("Should parse");

        let func = &ast.functions()[0];
        if let decy_parser::parser::Statement::Return(Some(Expression::PointerFieldAccess { field, .. })) = &func.body[0] {
            prop_assert_eq!(field, &field_name, "Field name must be preserved exactly");
        }
    }

    #[test]
    fn property_arrow_vs_dot_distinction(
        _var_name in valid_identifier(),
        field_name in valid_identifier(),
    ) {
        // Test that we distinguish -> from .
        let c_ptr = format!(
            r#"
            struct Data {{ int {}; }};
            int test_ptr(struct Data* ptr) {{ return ptr->{}; }}
            "#,
            field_name, field_name
        );

        let c_direct = format!(
            r#"
            struct Data {{ int {}; }};
            int test_direct(struct Data obj) {{ return obj.{}; }}
            "#,
            field_name, field_name
        );

        let parser = CParser::new().expect("Parser creation failed");

        let ast_ptr = parser.parse(&c_ptr).expect("Should parse ptr->");
        let ast_direct = parser.parse(&c_direct).expect("Should parse obj.");

        // Pointer access should be PointerFieldAccess
        let func_ptr = &ast_ptr.functions()[0];
        if let decy_parser::parser::Statement::Return(Some(expr)) = &func_ptr.body[0] {
            prop_assert!(
                matches!(expr, Expression::PointerFieldAccess { .. }),
                "ptr->field should be PointerFieldAccess"
            );
        }

        // Direct access should be FieldAccess
        let func_direct = &ast_direct.functions()[0];
        if let decy_parser::parser::Statement::Return(Some(expr)) = &func_direct.body[0] {
            prop_assert!(
                matches!(expr, Expression::FieldAccess { .. }),
                "obj.field should be FieldAccess"
            );
        }
    }

    #[test]
    fn property_nested_access_uses_correct_operators(
        _var_name in valid_identifier(),
        field1 in valid_identifier(),
        field2 in valid_identifier(),
    ) {
        // Test r->field1.field2 uses correct operators
        let c_code = format!(
            r#"
            struct Inner {{ int {}; }};
            struct Outer {{ struct Inner {}; }};
            int test(struct Outer* r) {{ return r->{}.{}; }}
            "#,
            field2, field1, field1, field2
        );

        let parser = CParser::new().expect("Parser creation failed");
        let ast = parser.parse(&c_code).expect("Should parse nested access");

        let func = &ast.functions()[0];
        if let decy_parser::parser::Statement::Return(Some(expr)) = &func.body[0] {
            // Outermost should be FieldAccess (for .field2)
            match expr {
                Expression::FieldAccess { object, field } => {
                    prop_assert_eq!(field, &field2, "Outer field should be field2");

                    // Inner should be PointerFieldAccess (for r->field1)
                    prop_assert!(
                        matches!(**object, Expression::PointerFieldAccess { .. }),
                        "Inner access should be PointerFieldAccess"
                    );
                }
                _ => return Err(proptest::test_runner::TestCaseError::fail(
                    "Outer access should be FieldAccess"
                )),
            }
        }
    }

    #[test]
    fn property_chained_pointer_access(
        _var_name in valid_identifier(),
        field1 in valid_identifier(),
        field2 in valid_identifier(),
    ) {
        // Test ptr->field1->field2 (both arrows)
        let c_code = format!(
            r#"
            struct Inner {{ int value; }};
            struct Outer {{ struct Inner* {}; }};
            struct Container {{ struct Outer* {}; }};
            int test(struct Container* ptr) {{ return ptr->{}->{}.value; }}
            "#,
            field1, field2, field2, field1
        );

        let parser = CParser::new().expect("Parser creation failed");
        if let Ok(ast) = parser.parse(&c_code) {
            let func = &ast.functions()[0];
            if !func.body.is_empty() {
                if let decy_parser::parser::Statement::Return(Some(expr)) = &func.body[0] {
                    // All accesses in chain should be identifiable
                    prop_assert!(
                        matches!(expr, Expression::FieldAccess { .. } | Expression::PointerFieldAccess { .. }),
                        "Chained access should parse as field access"
                    );
                }
            }
        }
    }

    #[test]
    fn property_deterministic_parsing(
        (c_code, _, _) in pointer_field_access()
    ) {
        let parser1 = CParser::new().expect("Parser creation failed");
        let parser2 = CParser::new().expect("Parser creation failed");

        let ast1 = parser1.parse(&c_code);
        let ast2 = parser2.parse(&c_code);

        // Both should succeed or both should fail
        prop_assert_eq!(ast1.is_ok(), ast2.is_ok(), "Parsing should be deterministic");

        if let (Ok(ast1), Ok(ast2)) = (ast1, ast2) {
            prop_assert_eq!(
                ast1.functions().len(),
                ast2.functions().len(),
                "Should parse same number of functions"
            );
        }
    }
}
