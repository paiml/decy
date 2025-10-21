//! Property-based tests for macro expansion (DECY-098c TDD-Refactor)
//!
//! These tests use proptest to verify macro expansion properties across
//! a wide range of inputs.
//!
//! Reference: K&R §4.11, ISO C99 §6.10.3

use decy_codegen::CodeGenerator;
use decy_hir::HirMacroDefinition;
use proptest::prelude::*;

/// Property: All numeric constants generate valid const declarations
#[test]
fn property_numeric_constants_generate_valid_code() {
    proptest!(|(value in -1000i32..1000)| {
        let macro_def = HirMacroDefinition::new_object_like(
            "NUM".to_string(),
            value.to_string()
        );

        let generator = CodeGenerator::new();
        let result = generator.generate_macro(&macro_def);

        prop_assert!(result.is_ok());
        let rust_code = result.unwrap();
        prop_assert!(rust_code.contains("const NUM"));
        prop_assert!(rust_code.contains("i32"));
        prop_assert!(rust_code.contains(&value.to_string()));
    });
}

/// Property: Float constants are always typed as f64
#[test]
fn property_float_constants_typed_as_f64() {
    proptest!(|(value in -1000.0f64..1000.0)| {
        let macro_def = HirMacroDefinition::new_object_like(
            "FLOAT".to_string(),
            value.to_string()
        );

        let generator = CodeGenerator::new();
        let result = generator.generate_macro(&macro_def);

        prop_assert!(result.is_ok());
        let rust_code = result.unwrap();
        prop_assert!(rust_code.contains("f64"));
    });
}

/// Property: String constants are always typed as &str
#[test]
fn property_string_constants_typed_as_str() {
    proptest!(|(text in "[a-zA-Z0-9 ]{1,20}")| {
        let macro_def = HirMacroDefinition::new_object_like(
            "STR".to_string(),
            format!("\"{}\"", text)
        );

        let generator = CodeGenerator::new();
        let result = generator.generate_macro(&macro_def);

        prop_assert!(result.is_ok());
        let rust_code = result.unwrap();
        prop_assert!(rust_code.contains("&str"));
        let expected = format!("\"{}\"", text);
        prop_assert!(rust_code.contains(&expected));
    });
}

/// Property: Macro names are preserved exactly
#[test]
fn property_macro_names_preserved() {
    proptest!(|(name in "[A-Z_][A-Z0-9_]{0,20}")| {
        let macro_def = HirMacroDefinition::new_object_like(
            name.clone(),
            "42".to_string()
        );

        let generator = CodeGenerator::new();
        let result = generator.generate_macro(&macro_def);

        prop_assert!(result.is_ok());
        let rust_code = result.unwrap();
        prop_assert!(rust_code.contains(&name));
    });
}

/// Property: Hex constants are recognized and preserved
#[test]
fn property_hex_constants_recognized() {
    proptest!(|(value in 0u32..256)| {
        let hex_str = format!("0x{:X}", value);
        let macro_def = HirMacroDefinition::new_object_like(
            "HEX".to_string(),
            hex_str.clone()
        );

        let generator = CodeGenerator::new();
        let result = generator.generate_macro(&macro_def);

        prop_assert!(result.is_ok());
        let rust_code = result.unwrap();
        prop_assert!(rust_code.contains(&hex_str) || rust_code.contains(&value.to_string()));
    });
}

/// Property: Empty macros always generate valid output
#[test]
fn property_empty_macros_valid() {
    proptest!(|(name in "[A-Z_]{1,20}")| {
        let macro_def = HirMacroDefinition::new_object_like(
            name.clone(),
            "".to_string()
        );

        let generator = CodeGenerator::new();
        let result = generator.generate_macro(&macro_def);

        prop_assert!(result.is_ok());
        let rust_code = result.unwrap();
        // Should either be a comment or valid Rust code
        prop_assert!(rust_code.contains(&name) || rust_code.starts_with("//"));
    });
}

/// Property: Negative numbers are handled correctly
#[test]
fn property_negative_numbers_handled() {
    proptest!(|(value in -1000i32..-1)| {
        let macro_def = HirMacroDefinition::new_object_like(
            "NEG".to_string(),
            value.to_string()
        );

        let generator = CodeGenerator::new();
        let result = generator.generate_macro(&macro_def);

        prop_assert!(result.is_ok());
        let rust_code = result.unwrap();
        prop_assert!(rust_code.contains("= -") || rust_code.contains(&value.to_string()));
    });
}

/// Property: Character literals are typed as char
#[test]
fn property_char_literals_typed_as_char() {
    proptest!(|(c in "[a-z]")| {
        let char_literal = format!("'{}'", c);
        let macro_def = HirMacroDefinition::new_object_like(
            "CHAR".to_string(),
            char_literal.clone()
        );

        let generator = CodeGenerator::new();
        let result = generator.generate_macro(&macro_def);

        prop_assert!(result.is_ok());
        let rust_code = result.unwrap();
        prop_assert!(rust_code.contains("char"));
        prop_assert!(rust_code.contains(&char_literal));
    });
}

/// Property: Multiple macros generate independent declarations
#[test]
fn property_multiple_macros_independent() {
    proptest!(|(value1 in 0i32..100, value2 in 0i32..100)| {
        let macro1 = HirMacroDefinition::new_object_like(
            "FIRST".to_string(),
            value1.to_string()
        );
        let macro2 = HirMacroDefinition::new_object_like(
            "SECOND".to_string(),
            value2.to_string()
        );

        let generator = CodeGenerator::new();
        let rust1 = generator.generate_macro(&macro1).expect("Failed to generate first");
        let rust2 = generator.generate_macro(&macro2).expect("Failed to generate second");

        // Each should be independent
        prop_assert!(rust1.contains("FIRST"));
        prop_assert!(!rust1.contains("SECOND"));
        prop_assert!(rust2.contains("SECOND"));
        prop_assert!(!rust2.contains("FIRST"));
    });
}

/// Property: Function-like macros generate inline functions (DECY-098d)
#[test]
fn property_function_like_macros_generate_functions() {
    proptest!(|(param in "[a-z]{1,5}")| {
        let macro_def = HirMacroDefinition::new_function_like(
            "FUNC".to_string(),
            vec![param.clone()],
            format!("({})", param)
        );

        let generator = CodeGenerator::new();
        let result = generator.generate_macro(&macro_def);

        // Should now succeed since function-like macros are implemented (DECY-098d)
        prop_assert!(result.is_ok());
        let rust_code = result.unwrap();
        prop_assert!(rust_code.contains("#[inline]"));
        prop_assert!(rust_code.contains("fn "));
    });
}
