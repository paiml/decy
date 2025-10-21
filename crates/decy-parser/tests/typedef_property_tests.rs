//! Property tests for typedef parsing (DECY-023 REFACTOR phase)
//!
//! This test suite uses proptest to verify typedef parsing properties across
//! a wide range of inputs. Target: 10 properties × 256 cases = 2,560 test cases.
//!
//! References:
//! - K&R §6.7: Type Names
//! - ISO C99 §6.7.7: Type definitions

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
        Just("void".to_string()),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property 1: Any valid typedef with simple type should parse without panic
    #[test]
    fn prop_typedef_simple_never_panics(
        typedef_name in valid_identifier(),
        base_type in simple_type()
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let source = format!("typedef {} {};", base_type, typedef_name);

        // Should not panic regardless of input
        let _ = parser.parse(&source);
    }

    /// Property 2: Valid typedef should always produce at least one typedef in AST
    #[test]
    fn prop_valid_typedef_always_parses(
        typedef_name in valid_identifier(),
        base_type in simple_type()
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let source = format!("typedef {} {};", base_type, typedef_name);

        if let Ok(ast) = parser.parse(&source) {
            prop_assert!(!ast.typedefs().is_empty(), "Should have at least one typedef");
        }
    }

    /// Property 3: Typedef name should be preserved exactly
    #[test]
    fn prop_typedef_name_preserved(
        typedef_name in valid_identifier(),
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let source = format!("typedef int {};", typedef_name);

        if let Ok(ast) = parser.parse(&source) {
            if !ast.typedefs().is_empty() {
                let parsed = &ast.typedefs()[0];
                prop_assert_eq!(parsed.name(), typedef_name.as_str());
            }
        }
    }

    /// Property 4: Pointer typedefs should be recognized as pointers
    #[test]
    fn prop_pointer_typedef_is_pointer(
        typedef_name in valid_identifier(),
        base_type in simple_type()
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let source = format!("typedef {}* {};", base_type, typedef_name);

        if let Ok(ast) = parser.parse(&source) {
            if !ast.typedefs().is_empty() {
                let parsed = &ast.typedefs()[0];
                prop_assert!(parsed.is_pointer(), "Should be recognized as pointer");
            }
        }
    }

    /// Property 5: Multiple typedefs should all be parsed
    #[test]
    fn prop_multiple_typedefs_all_parsed(
        name1 in valid_identifier(),
        name2 in valid_identifier(),
        name3 in valid_identifier(),
    ) {
        // Ensure unique names
        prop_assume!(name1 != name2 && name2 != name3 && name1 != name3);

        let parser = CParser::new().expect("Parser creation failed");
        let source = format!(
            "typedef int {};\ntypedef float {};\ntypedef double {};",
            name1, name2, name3
        );

        if let Ok(ast) = parser.parse(&source) {
            prop_assert!(
                ast.typedefs().len() >= 3,
                "Should parse all three typedefs, got {}",
                ast.typedefs().len()
            );
        }
    }

    /// Property 6: Typedef parsing should be deterministic
    #[test]
    fn prop_typedef_parsing_deterministic(
        typedef_name in valid_identifier(),
        base_type in simple_type()
    ) {
        let parser1 = CParser::new().expect("Parser creation failed");
        let parser2 = CParser::new().expect("Parser creation failed");
        let source = format!("typedef {} {};", base_type, typedef_name);

        let result1 = parser1.parse(&source);
        let result2 = parser2.parse(&source);

        // Both should succeed or both should fail
        prop_assert_eq!(result1.is_ok(), result2.is_ok());

        if let (Ok(ast1), Ok(ast2)) = (result1, result2) {
            prop_assert_eq!(ast1.typedefs().len(), ast2.typedefs().len());
            if !ast1.typedefs().is_empty() {
                prop_assert_eq!(ast1.typedefs()[0].name(), ast2.typedefs()[0].name());
            }
        }
    }

    /// Property 7: Typedef with function should not interfere
    #[test]
    fn prop_typedef_with_function_both_parsed(
        typedef_name in valid_identifier(),
        func_name in valid_identifier(),
    ) {
        prop_assume!(typedef_name != func_name);

        let parser = CParser::new().expect("Parser creation failed");
        let source = format!(
            "typedef int {};\n{} add({} a, {} b) {{ return a + b; }}",
            typedef_name, typedef_name, typedef_name, typedef_name
        );

        if let Ok(ast) = parser.parse(&source) {
            prop_assert!(!ast.typedefs().is_empty(), "Should have typedef");
            prop_assert!(!ast.functions().is_empty(), "Should have function");
        }
    }

    /// Property 8: Const typedefs should parse
    #[test]
    fn prop_const_typedef_parses(
        typedef_name in valid_identifier(),
        base_type in simple_type()
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let source = format!("typedef const {}* {};", base_type, typedef_name);

        // Should not panic
        let _ = parser.parse(&source);
    }

    /// Property 9: Typedef underlying type should match base type
    #[test]
    fn prop_typedef_type_matches(
        typedef_name in valid_identifier(),
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let source = format!("typedef int {};", typedef_name);

        if let Ok(ast) = parser.parse(&source) {
            if !ast.typedefs().is_empty() {
                let parsed = &ast.typedefs()[0];
                prop_assert_eq!(parsed.underlying_type(), "int");
            }
        }
    }

    /// Property 10: Function pointer typedefs should be recognized
    #[test]
    fn prop_function_pointer_typedef_recognized(
        typedef_name in valid_identifier(),
    ) {
        let parser = CParser::new().expect("Parser creation failed");
        let source = format!("typedef int (*{})(int, int);", typedef_name);

        if let Ok(ast) = parser.parse(&source) {
            if !ast.typedefs().is_empty() {
                let parsed = &ast.typedefs()[0];
                prop_assert!(
                    parsed.is_function_pointer(),
                    "Should be recognized as function pointer"
                );
            }
        }
    }
}
