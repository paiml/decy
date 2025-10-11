//! Property tests for string handling in HIR (DECY-025 REFACTOR phase).

use super::*;
use proptest::prelude::*;

proptest! {
    /// Property: String literals can be created with any valid string
    #[test]
    fn property_string_literal_creation(s in ".*") {
        let string_lit = HirExpression::StringLiteral(s.clone());

        match string_lit {
            HirExpression::StringLiteral(content) => {
                prop_assert_eq!(content, s);
            }
            _ => prop_assert!(false, "Expected StringLiteral"),
        }
    }

    /// Property: String literals with same content are equal
    #[test]
    fn property_string_literal_equality(s in ".*") {
        let lit1 = HirExpression::StringLiteral(s.clone());
        let lit2 = HirExpression::StringLiteral(s);

        prop_assert_eq!(lit1, lit2);
    }

    /// Property: Cloning string literals preserves content
    #[test]
    fn property_string_literal_clone_preserves_content(s in ".*") {
        let original = HirExpression::StringLiteral(s);
        let cloned = original.clone();

        prop_assert_eq!(original, cloned);
    }

    /// Property: Empty strings are valid
    #[test]
    fn property_empty_string_valid(_seed in 0u32..100) {
        let empty = HirExpression::StringLiteral(String::new());

        match empty {
            HirExpression::StringLiteral(s) => {
                prop_assert!(s.is_empty());
            }
            _ => prop_assert!(false, "Expected StringLiteral"),
        }
    }

    /// Property: String literals in function calls preserve content
    #[test]
    fn property_string_in_function_call(s in ".*", func_name in "[a-z_][a-z0-9_]{0,10}") {
        let call = HirExpression::FunctionCall {
            function: func_name.clone(),
            arguments: vec![HirExpression::StringLiteral(s.clone())],
        };

        match call {
            HirExpression::FunctionCall { function, arguments } => {
                prop_assert_eq!(function, func_name);
                prop_assert_eq!(arguments.len(), 1);
                match &arguments[0] {
                    HirExpression::StringLiteral(content) => {
                        prop_assert_eq!(content, &s);
                    }
                    _ => prop_assert!(false, "Expected StringLiteral in arguments"),
                }
            }
            _ => prop_assert!(false, "Expected FunctionCall"),
        }
    }

    /// Property: Variable declarations with string literals preserve both
    #[test]
    fn property_var_decl_with_string(
        var_name in "[a-z_][a-z0-9_]{0,10}",
        string_val in ".*"
    ) {
        let decl = HirStatement::VariableDeclaration {
            name: var_name.clone(),
            var_type: HirType::Pointer(Box::new(HirType::Char)),
            initializer: Some(HirExpression::StringLiteral(string_val.clone())),
        };

        match decl {
            HirStatement::VariableDeclaration { name, initializer, .. } => {
                prop_assert_eq!(name, var_name);
                match initializer {
                    Some(HirExpression::StringLiteral(s)) => {
                        prop_assert_eq!(s, string_val);
                    }
                    _ => prop_assert!(false, "Expected StringLiteral initializer"),
                }
            }
            _ => prop_assert!(false, "Expected VariableDeclaration"),
        }
    }
}
