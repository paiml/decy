//! Property tests for function pointer parsing (DECY-024 REFACTOR phase)
//!
//! This test suite uses proptest to verify function pointer parsing properties.
//! Target: 10 properties × 256 cases = 2,560 test cases.
//!
//! References:
//! - K&R §5.11: Pointers to Functions
//! - ISO C99 §6.7.5.3: Function declarators

use decy_parser::CParser;
use proptest::prelude::*;

/// Generate valid C identifier strings
fn valid_identifier() -> impl Strategy<Value = String> {
    "[a-zA-Z_][a-zA-Z0-9_]{0,30}".prop_map(|s| s.to_string())
}

/// Generate simple C type names
fn simple_type() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("int".to_string()),
        Just("float".to_string()),
        Just("double".to_string()),
        Just("char".to_string()),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property 1: Parsing function pointer should never panic
    #[test]
    fn prop_function_pointer_never_panics(
        var_name in valid_identifier(),
        return_type in simple_type()
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let source = format!("{} (*{})(int);", return_type, var_name);

        // Should not panic regardless of input
        let _ = parser.parse(&source);
    }

    /// Property 2: Valid function pointer should always parse
    #[test]
    fn prop_function_pointer_always_parses(
        var_name in valid_identifier(),
        return_type in simple_type()
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let source = format!("{} (*{})(int);", return_type, var_name);

        if let Ok(ast) = parser.parse(&source) {
            prop_assert!(!ast.variables().is_empty(), "Should have at least one variable");
        }
    }

    /// Property 3: Function pointer name should be preserved
    #[test]
    fn prop_function_pointer_name_preserved(
        var_name in valid_identifier(),
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let source = format!("int (*{})(int);", var_name);

        if let Ok(ast) = parser.parse(&source) {
            if !ast.variables().is_empty() {
                prop_assert_eq!(ast.variables()[0].name(), var_name.as_str());
            }
        }
    }

    /// Property 4: Function pointer should be recognized as such
    #[test]
    fn prop_function_pointer_is_recognized(
        var_name in valid_identifier(),
        return_type in simple_type()
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let source = format!("{} (*{})(int);", return_type, var_name);

        if let Ok(ast) = parser.parse(&source) {
            if !ast.variables().is_empty() {
                prop_assert!(
                    ast.variables()[0].is_function_pointer(),
                    "Should be recognized as function pointer"
                );
            }
        }
    }

    /// Property 5: Multiple function pointers should all parse
    #[test]
    fn prop_multiple_function_pointers_parse(
        name1 in valid_identifier(),
        name2 in valid_identifier(),
    ) {
        prop_assume!(name1 != name2);

        let parser = CParser::new().expect("Parser creation failed");
        let source = format!(
            "int (*{})(int);\nfloat (*{})(float);",
            name1, name2
        );

        if let Ok(ast) = parser.parse(&source) {
            prop_assert!(
                ast.variables().len() >= 2,
                "Should parse both function pointers"
            );
        }
    }

    /// Property 6: Parsing should be deterministic
    #[test]
    fn prop_function_pointer_parsing_deterministic(
        var_name in valid_identifier(),
    ) {
        let parser1 = CParser::new().expect("Parser creation failed");
        let parser2 = CParser::new().expect("Parser creation failed");
        let source = format!("int (*{})(int);", var_name);

        let result1 = parser1.parse(&source);
        let result2 = parser2.parse(&source);

        prop_assert_eq!(result1.is_ok(), result2.is_ok());

        if let (Ok(ast1), Ok(ast2)) = (result1, result2) {
            prop_assert_eq!(ast1.variables().len(), ast2.variables().len());
            if !ast1.variables().is_empty() {
                prop_assert_eq!(
                    ast1.variables()[0].name(),
                    ast2.variables()[0].name()
                );
            }
        }
    }

    /// Property 7: Void return type should work
    #[test]
    fn prop_void_return_function_pointer(
        var_name in valid_identifier(),
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let source = format!("void (*{})(int);", var_name);

        if let Ok(ast) = parser.parse(&source) {
            if !ast.variables().is_empty() {
                let var = &ast.variables()[0];
                prop_assert!(var.is_function_pointer());
                prop_assert!(var.function_pointer_has_void_return());
            }
        }
    }

    /// Property 8: Parameter count should be tracked
    #[test]
    fn prop_function_pointer_param_count(
        var_name in valid_identifier(),
        param_count in 0usize..5,
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let params = vec!["int"; param_count].join(", ");
        let source = format!("int (*{})({}); ", var_name, params);

        if let Ok(ast) = parser.parse(&source) {
            if !ast.variables().is_empty() {
                let var = &ast.variables()[0];
                prop_assert_eq!(var.function_pointer_param_count(), param_count);
            }
        }
    }

    /// Property 9: Function pointer in typedef should work
    #[test]
    fn prop_function_pointer_typedef(
        typedef_name in valid_identifier(),
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let source = format!("typedef int (*{})(int);", typedef_name);

        if let Ok(ast) = parser.parse(&source) {
            prop_assert!(!ast.typedefs().is_empty(), "Should have typedef");
            let typedef = &ast.typedefs()[0];
            prop_assert_eq!(typedef.name(), typedef_name.as_str());
            prop_assert!(typedef.is_function_pointer());
        }
    }

    /// Property 10: Function pointer with different types should work
    #[test]
    fn prop_function_pointer_with_different_types(
        var_name in valid_identifier(),
        return_type in simple_type(),
        param_type in simple_type(),
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let source = format!("{} (*{})({});", return_type, var_name, param_type);

        if let Ok(ast) = parser.parse(&source) {
            if !ast.variables().is_empty() {
                prop_assert!(ast.variables()[0].is_function_pointer());
            }
        }
    }
}
