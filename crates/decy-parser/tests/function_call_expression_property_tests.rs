//! Property tests for function call expressions (DECY-036 REFACTOR phase)
//!
//! These property tests verify invariants for function call parsing across
//! randomly generated test cases.
//!
//! References:
//! - K&R §5.4: Pointers and Functions
//! - ISO C99 §6.5.2.2: Function calls

use decy_parser::parser::{Expression, Statement};
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
            "sizeof", "typeof",
        ];

        if C_KEYWORDS.contains(&s.as_str()) {
            format!("{}_var", s)
        } else {
            s
        }
    }
}

prop_compose! {
    /// Generate function call in variable initializer
    fn function_call_in_initializer()(
        (var_name, param_name) in (valid_identifier(), valid_identifier())
            .prop_filter("Names must differ", |(v, p)| v != p)
    ) -> (String, String) {
        let c_code = format!(
            r#"
            void* helper(int {});

            void* test(int {}) {{
                void* {} = helper({});
                return {};
            }}
            "#,
            param_name, param_name, var_name, param_name, var_name
        );
        (c_code, var_name)
    }
}

prop_compose! {
    /// Generate function call in assignment
    fn function_call_in_assignment()(
        (var_name, param_name) in (valid_identifier(), valid_identifier())
            .prop_filter("Names must differ", |(v, p)| v != p)
    ) -> (String, String) {
        let c_code = format!(
            r#"
            void* helper(int {});

            void* test(int {}) {{
                void* {};
                {} = helper({});
                return {};
            }}
            "#,
            param_name, param_name, var_name, var_name, param_name, var_name
        );
        (c_code, var_name)
    }
}

prop_compose! {
    /// Generate function call in return statement
    fn function_call_in_return()(
        param_name in valid_identifier()
    ) -> String {
        format!(
            r#"
            void* helper(int {});

            void* test(int {}) {{
                return helper({});
            }}
            "#,
            param_name, param_name, param_name
        )
    }
}

prop_compose! {
    /// Generate nested function calls
    fn nested_function_calls()(
        param_name in valid_identifier()
    ) -> String {
        format!(
            r#"
            int inner(int {});
            int outer(int {});

            int test(int {}) {{
                return outer(inner({}));
            }}
            "#,
            param_name, param_name, param_name, param_name
        )
    }
}

proptest! {
    #[test]
    fn property_function_call_in_initializer_never_panics(
        (c_code, _) in function_call_in_initializer()
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let _ = parser.parse(&c_code);
    }

    #[test]
    fn property_function_call_in_initializer_parses_correctly(
        (c_code, var_name) in function_call_in_initializer()
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let ast = parser.parse(&c_code).expect("Should parse valid C code");

        prop_assert!(!ast.functions().is_empty(), "Should have at least test function");

        // Find the test function
        let func = ast.functions().iter().find(|f| f.name == "test").expect("Should have test function");

        // First statement should be VariableDeclaration with FunctionCall initializer
        if let Statement::VariableDeclaration { name, initializer, .. } = &func.body[0] {
            prop_assert_eq!(name, &var_name, "Variable name should match");
            prop_assert!(initializer.is_some(), "Should have initializer");

            if let Some(Expression::FunctionCall { function, .. }) = initializer {
                prop_assert_eq!(function, "helper", "Function name should be helper");
            }
        }
    }

    #[test]
    fn property_function_call_in_assignment_never_panics(
        (c_code, _) in function_call_in_assignment()
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let _ = parser.parse(&c_code);
    }

    #[test]
    fn property_function_call_in_assignment_parses_correctly(
        (c_code, var_name) in function_call_in_assignment()
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let ast = parser.parse(&c_code).expect("Should parse valid C code");

        // Find the test function
        let func = ast.functions().iter().find(|f| f.name == "test").expect("Should have test function");

        // Should have at least 2 statements (declaration + assignment)
        prop_assert!(func.body.len() >= 2, "Should have declaration and assignment");

        // Second statement should be Assignment with FunctionCall
        if let Statement::Assignment { target, value } = &func.body[1] {
            prop_assert_eq!(target, &var_name, "Assignment target should match");

            if let Expression::FunctionCall { function, .. } = value {
                prop_assert_eq!(function, "helper", "Function name should be helper");
            } else {
                return Err(proptest::test_runner::TestCaseError::fail(
                    "Assignment value should be FunctionCall"
                ));
            }
        } else {
            return Err(proptest::test_runner::TestCaseError::fail(
                "Second statement should be Assignment"
            ));
        }
    }

    #[test]
    fn property_function_call_in_return_never_panics(
        c_code in function_call_in_return()
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let _ = parser.parse(&c_code);
    }

    #[test]
    fn property_function_call_in_return_parses_correctly(
        c_code in function_call_in_return()
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let ast = parser.parse(&c_code).expect("Should parse valid C code");

        // Find the test function
        let func = ast.functions().iter().find(|f| f.name == "test").expect("Should have test function");

        if let Statement::Return(Some(Expression::FunctionCall { function, .. })) = &func.body[0] {
            prop_assert_eq!(function, "helper", "Function name should be helper");
        } else {
            return Err(proptest::test_runner::TestCaseError::fail(
                "Return should contain FunctionCall"
            ));
        }
    }

    #[test]
    fn property_nested_function_calls_never_panic(
        c_code in nested_function_calls()
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let _ = parser.parse(&c_code);
    }

    #[test]
    fn property_nested_function_calls_parse_correctly(
        c_code in nested_function_calls()
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let ast = parser.parse(&c_code).expect("Should parse valid C code");

        // Find the test function
        let func = ast.functions().iter().find(|f| f.name == "test").expect("Should have test function");

        if let Statement::Return(Some(Expression::FunctionCall { function, arguments })) = &func.body[0] {
            prop_assert_eq!(function, "outer", "Outer function name should be outer");
            prop_assert_eq!(arguments.len(), 1, "Should have 1 argument");

            // Inner should also be FunctionCall
            if let Expression::FunctionCall { function: inner, .. } = &arguments[0] {
                prop_assert_eq!(inner, "inner", "Inner function name should be inner");
            } else {
                return Err(proptest::test_runner::TestCaseError::fail(
                    "Argument should be FunctionCall"
                ));
            }
        }
    }

    #[test]
    fn property_function_call_parsing_deterministic(
        param_name in valid_identifier()
    ) {
        let c_code = format!(
            r#"
            void* helper(int {});

            void* test(int {}) {{
                return helper({});
            }}
            "#,
            param_name, param_name, param_name
        );

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
    fn property_function_call_preserves_argument_count(
        (arg1, arg2, arg3) in (valid_identifier(), valid_identifier(), valid_identifier())
            .prop_filter("All args must be unique", |(a1, a2, a3)| {
                a1 != a2 && a2 != a3 && a1 != a3
            })
    ) {
        // Test with 0, 1, 2, and 3 arguments
        for (args_str, expected_count) in [
            ("", 0),
            (&arg1[..], 1),
            (&format!("{}, {}", arg1, arg2)[..], 2),
            (&format!("{}, {}, {}", arg1, arg2, arg3)[..], 3),
        ] {
            let c_code = format!(
                r#"
                void* helper();
                void* helper1(int);
                void* helper2(int, int);
                void* helper3(int, int, int);

                void* test(int {}, int {}, int {}) {{
                    return helper{}({});
                }}
                "#,
                arg1, arg2, arg3, expected_count, args_str
            );

            let parser = CParser::new().expect("Parser creation failed");
            let ast = parser.parse(&c_code).expect("Should parse");

            // Find the test function
            let func = ast.functions().iter().find(|f| f.name == "test").expect("Should have test function");

            if let Statement::Return(Some(Expression::FunctionCall { arguments, .. })) = &func.body[0] {
                prop_assert_eq!(
                    arguments.len(),
                    expected_count,
                    "Argument count should match for {} args",
                    expected_count
                );
            }
        }
    }

    #[test]
    fn property_malloc_with_sizeof_pattern(
        var_name in valid_identifier(),
        type_name in prop::sample::select(vec!["int", "char", "float", "double"])
    ) {
        let c_code = format!(
            r#"
            void* malloc(unsigned long);

            void* test() {{
                void* {} = malloc(sizeof({}));
                return {};
            }}
            "#,
            var_name, type_name, var_name
        );

        let parser = CParser::new().expect("Parser creation failed");
        let ast = parser.parse(&c_code).expect("Should parse malloc with sizeof");

        // Find the test function
        let func = ast.functions().iter().find(|f| f.name == "test").expect("Should have test function");

        if let Statement::VariableDeclaration { initializer: Some(Expression::FunctionCall { function, arguments }), .. } = &func.body[0] {
            prop_assert_eq!(function, "malloc", "Should be malloc");
            prop_assert_eq!(arguments.len(), 1, "malloc should have 1 argument");

            // Argument should be sizeof
            prop_assert!(
                matches!(arguments[0], Expression::Sizeof { .. }),
                "Argument should be sizeof expression"
            );
        }
    }
}
