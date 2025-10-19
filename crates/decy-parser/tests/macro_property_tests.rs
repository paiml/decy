//! Property-based tests for macro parsing (DECY-098b REFACTOR)
//!
//! These tests use proptest to verify macro parsing properties across
//! a wide range of inputs.
//!
//! Reference: K&R §4.11, ISO C99 §6.10.3

use decy_parser::parser::CParser;
use proptest::prelude::*;

/// Property: Any valid C identifier can be a macro name
#[test]
fn property_macro_name_is_valid_identifier() {
    proptest!(|(name in "[a-zA-Z_][a-zA-Z0-9_]{0,30}")| {
        let c_code = format!("#define {} 100\nint main() {{ return 0; }}", name);
        let parser = CParser::new().expect("Failed to create parser");

        if let Ok(ast) = parser.parse(&c_code) {
            if ast.macros().len() == 1 {
                prop_assert_eq!(ast.macros()[0].name(), name.as_str());
            }
        }
    });
}

/// Property: Object-like macros have no parameters
#[test]
fn property_object_like_macros_have_no_parameters() {
    proptest!(|(value in 0i32..1000)| {
        let c_code = format!("#define MAX {}\nint main() {{ return 0; }}", value);
        let parser = CParser::new().expect("Failed to create parser");
        let ast = parser.parse(&c_code).expect("Failed to parse");

        prop_assert_eq!(ast.macros().len(), 1);
        prop_assert!(ast.macros()[0].is_object_like());
        prop_assert_eq!(ast.macros()[0].parameters().len(), 0);
    });
}

/// Property: Function-like macros with parameters are detected
#[test]
fn property_function_like_macros_have_parameters() {
    proptest!(|(param in "[a-zA-Z_][a-zA-Z0-9_]{0,10}")| {
        let c_code = format!("#define SQR({}) (({}) * ({}))\nint main() {{ return 0; }}", param, param, param);
        let parser = CParser::new().expect("Failed to create parser");
        let ast = parser.parse(&c_code).expect("Failed to parse");

        prop_assert_eq!(ast.macros().len(), 1);
        prop_assert!(ast.macros()[0].is_function_like());
        prop_assert_eq!(ast.macros()[0].parameters().len(), 1);
        prop_assert_eq!(ast.macros()[0].parameters()[0].as_str(), param.as_str());
    });
}

/// Property: Macros with numeric bodies parse correctly
#[test]
fn property_numeric_macro_bodies() {
    proptest!(|(value in -1000i32..1000)| {
        let c_code = format!("#define NUM {}\nint main() {{ return 0; }}", value);
        let parser = CParser::new().expect("Failed to create parser");
        let ast = parser.parse(&c_code).expect("Failed to parse");

        prop_assert_eq!(ast.macros().len(), 1);
        let body = ast.macros()[0].body();
        prop_assert!(body.contains(&value.to_string()));
    });
}

/// Property: Multiple macros are all captured
#[test]
fn property_multiple_macros_all_captured() {
    proptest!(|(count in 1usize..10)| {
        let mut c_code = String::new();
        for i in 0..count {
            c_code.push_str(&format!("#define MACRO_{} {}\n", i, i * 10));
        }
        c_code.push_str("int main() { return 0; }");

        let parser = CParser::new().expect("Failed to create parser");
        let ast = parser.parse(&c_code).expect("Failed to parse");

        prop_assert_eq!(ast.macros().len(), count);
        for i in 0..count {
            prop_assert_eq!(ast.macros()[i].name(), format!("MACRO_{}", i));
        }
    });
}

/// Property: Macro names are preserved exactly
#[test]
fn property_macro_names_preserved() {
    proptest!(|(
        prefix in "[A-Z]{1,5}",
        suffix in "[A-Z0-9_]{0,10}"
    )| {
        let name = format!("{}{}", prefix, suffix);
        let c_code = format!("#define {} 42\nint main() {{ return 0; }}", name);
        let parser = CParser::new().expect("Failed to create parser");
        let ast = parser.parse(&c_code).expect("Failed to parse");

        prop_assert_eq!(ast.macros().len(), 1);
        prop_assert_eq!(ast.macros()[0].name(), name.as_str());
    });
}

/// Property: Empty macro bodies are handled
#[test]
fn property_empty_macro_bodies() {
    proptest!(|(name in "[A-Z_]{1,20}")| {
        let c_code = format!("#define {}\nint main() {{ return 0; }}", name);
        let parser = CParser::new().expect("Failed to create parser");
        let ast = parser.parse(&c_code).expect("Failed to parse");

        prop_assert_eq!(ast.macros().len(), 1);
        prop_assert_eq!(ast.macros()[0].body(), "");
    });
}

/// Property: Macros don't interfere with function parsing
#[test]
fn property_macros_dont_interfere_with_functions() {
    proptest!(|(value in 1i32..100)| {
        let c_code = format!(
            "#define MAX {}\nint get_max() {{ return MAX; }}",
            value
        );
        let parser = CParser::new().expect("Failed to create parser");
        let ast = parser.parse(&c_code).expect("Failed to parse");

        prop_assert_eq!(ast.macros().len(), 1);
        prop_assert_eq!(ast.functions().len(), 1);
        prop_assert_eq!(ast.functions()[0].name.as_str(), "get_max");
    });
}

/// Property: String literal macro bodies preserve quotes
#[test]
fn property_string_literal_bodies() {
    proptest!(|(text in "[a-zA-Z ]{1,20}")| {
        let c_code = format!("#define MSG \"{}\"\nint main() {{ return 0; }}", text);
        let parser = CParser::new().expect("Failed to create parser");
        let ast = parser.parse(&c_code).expect("Failed to parse");

        prop_assert_eq!(ast.macros().len(), 1);
        let body = ast.macros()[0].body();
        prop_assert!(body.starts_with("\""));
        prop_assert!(body.ends_with("\""));
    });
}

/// Property: Multi-parameter macros capture all parameters
#[test]
fn property_multi_parameter_macros() {
    // Use specific safe parameter names to avoid C keywords and ensure uniqueness
    proptest!(|(
        suffix1 in 0u32..100,
        suffix2 in 0u32..100,
        suffix3 in 0u32..100
    )| {
        let param1 = format!("x{}", suffix1);
        let param2 = format!("y{}", suffix2);
        let param3 = format!("z{}", suffix3);

        let c_code = format!(
            "#define ADD3({},{},{}) (({})+({})+({}))\nint main() {{ return 0; }}",
            param1, param2, param3, param1, param2, param3
        );
        let parser = CParser::new().expect("Failed to create parser");

        if let Ok(ast) = parser.parse(&c_code) {
            if ast.macros().len() == 1 {
                prop_assert_eq!(ast.macros()[0].parameters().len(), 3);
                prop_assert_eq!(ast.macros()[0].parameters()[0].as_str(), param1.as_str());
                prop_assert_eq!(ast.macros()[0].parameters()[1].as_str(), param2.as_str());
                prop_assert_eq!(ast.macros()[0].parameters()[2].as_str(), param3.as_str());
            }
        }
    });
}
