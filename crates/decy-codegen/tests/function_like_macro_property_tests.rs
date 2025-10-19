//! Property-based tests for function-like macro expansion (DECY-098d REFACTOR)
//!
//! These tests use proptest to verify function-like macro expansion properties
//! across a wide range of inputs.
//!
//! Reference: K&R §4.11, ISO C99 §6.10.3

use decy_codegen::CodeGenerator;
use decy_hir::HirMacroDefinition;
use proptest::prelude::*;

/// Property: All function-like macros generate valid inline functions
#[test]
fn property_generates_inline_functions() {
    proptest!(|(param in "[a-z]{1,3}")| {
        let macro_def = HirMacroDefinition::new_function_like(
            "FUNC".to_string(),
            vec![param.clone()],
            format!("(({})*(2))", param)
        );

        let generator = CodeGenerator::new();
        let result = generator.generate_macro(&macro_def);

        prop_assert!(result.is_ok());
        let rust_code = result.unwrap();
        prop_assert!(rust_code.contains("#[inline]"));
        prop_assert!(rust_code.contains("fn "));
    });
}

/// Property: Macro names are converted to snake_case
#[test]
fn property_name_converted_to_snake_case() {
    proptest!(|(name in "[A-Z][A-Z_]{0,9}")| {
        // Ensure at least one uppercase letter to test conversion
        let macro_def = HirMacroDefinition::new_function_like(
            name.clone(),
            vec!["x".to_string()],
            "(x)".to_string()
        );

        let generator = CodeGenerator::new();
        let result = generator.generate_macro(&macro_def);

        prop_assert!(result.is_ok());
        let rust_code = result.unwrap();
        // Should contain lowercase version
        let lowercase = name.to_lowercase();
        prop_assert!(rust_code.contains(&lowercase));

        // If name contains uppercase letters, shouldn't contain original uppercase
        if name != lowercase {
            let fn_name = format!("fn {}", name);
            prop_assert!(!rust_code.contains(&fn_name));
        }
    });
}

/// Property: Single parameter macros always have one parameter
#[test]
fn property_single_param_has_one_param() {
    proptest!(|(param in "[a-z]{1,5}")| {
        let macro_def = HirMacroDefinition::new_function_like(
            "FUNC".to_string(),
            vec![param.clone()],
            format!("({})", param)
        );

        let generator = CodeGenerator::new();
        let result = generator.generate_macro(&macro_def);

        prop_assert!(result.is_ok());
        let rust_code = result.unwrap();
        // Should contain the parameter with type
        let param_decl = format!("{}: i32", param);
        prop_assert!(rust_code.contains(&param_decl));
    });
}

/// Property: Multiple parameter macros preserve parameter order
#[test]
fn property_multi_param_preserves_order() {
    proptest!(|(suffix1 in 0u32..100, suffix2 in 0u32..100)| {
        let param1 = format!("x{}", suffix1);
        let param2 = format!("y{}", suffix2);

        let macro_def = HirMacroDefinition::new_function_like(
            "FUNC".to_string(),
            vec![param1.clone(), param2.clone()],
            format!("(({})+({}))", param1, param2)
        );

        let generator = CodeGenerator::new();
        let result = generator.generate_macro(&macro_def);

        prop_assert!(result.is_ok());
        let rust_code = result.unwrap();

        // Find positions of parameters
        let pos1 = rust_code.find(&param1);
        let pos2 = rust_code.find(&param2);

        // Both should exist and param1 should come before param2 in signature
        prop_assert!(pos1.is_some());
        prop_assert!(pos2.is_some());
        prop_assert!(pos1.unwrap() < pos2.unwrap());
    });
}

/// Property: Arithmetic expressions generate i32 return type
#[test]
fn property_arithmetic_returns_i32() {
    proptest!(|(param in "[a-z]{1,3}", multiplier in 1i32..10)| {
        let macro_def = HirMacroDefinition::new_function_like(
            "FUNC".to_string(),
            vec![param.clone()],
            format!("(({})*({}))", param, multiplier)
        );

        let generator = CodeGenerator::new();
        let result = generator.generate_macro(&macro_def);

        prop_assert!(result.is_ok());
        let rust_code = result.unwrap();
        prop_assert!(rust_code.contains("-> i32"));
    });
}

/// Property: Comparison expressions generate bool return type
#[test]
fn property_comparison_returns_bool() {
    proptest!(|(param in "[a-z]{1,3}", value in 0i32..100)| {
        let macro_def = HirMacroDefinition::new_function_like(
            "FUNC".to_string(),
            vec![param.clone()],
            format!("(({})<({}))", param, value)
        );

        let generator = CodeGenerator::new();
        let result = generator.generate_macro(&macro_def);

        prop_assert!(result.is_ok());
        let rust_code = result.unwrap();
        prop_assert!(rust_code.contains("-> bool"));
    });
}

/// Property: Logical operators generate bool return type
#[test]
fn property_logical_returns_bool() {
    proptest!(|(param1 in "[a-z]{1,2}", param2 in "[a-z]{1,2}")| {
        let macro_def = HirMacroDefinition::new_function_like(
            "FUNC".to_string(),
            vec![param1.clone(), param2.clone()],
            format!("(({})||({}))", param1, param2)
        );

        let generator = CodeGenerator::new();
        let result = generator.generate_macro(&macro_def);

        prop_assert!(result.is_ok());
        let rust_code = result.unwrap();
        prop_assert!(rust_code.contains("-> bool"));
    });
}

/// Property: Ternary operators generate i32 return type (return values, not condition)
#[test]
fn property_ternary_returns_i32() {
    proptest!(|(param1 in "[a-z]{1,2}", param2 in "[a-z]{1,2}")| {
        let macro_def = HirMacroDefinition::new_function_like(
            "FUNC".to_string(),
            vec![param1.clone(), param2.clone()],
            format!("(({})<({})?({}):({})", param1, param2, param1, param2)
        );

        let generator = CodeGenerator::new();
        let result = generator.generate_macro(&macro_def);

        prop_assert!(result.is_ok());
        let rust_code = result.unwrap();
        prop_assert!(rust_code.contains("-> i32"));
    });
}

/// Property: Ternary operators transform to if-else
#[test]
fn property_ternary_becomes_if_else() {
    proptest!(|(param1 in "[a-z]{1,2}", param2 in "[a-z]{1,2}")| {
        let macro_def = HirMacroDefinition::new_function_like(
            "FUNC".to_string(),
            vec![param1.clone(), param2.clone()],
            format!("(({})<({})?({}):({})", param1, param2, param1, param2)
        );

        let generator = CodeGenerator::new();
        let result = generator.generate_macro(&macro_def);

        prop_assert!(result.is_ok());
        let rust_code = result.unwrap();
        prop_assert!(rust_code.contains("if"));
        prop_assert!(rust_code.contains("else"));
    });
}

/// Property: Generated functions always have inline attribute
#[test]
fn property_always_has_inline() {
    proptest!(|(
        name in "[A-Z]{1,5}",
        param in "[a-z]{1,3}",
        value in 1i32..10
    )| {
        let macro_def = HirMacroDefinition::new_function_like(
            name,
            vec![param.clone()],
            format!("(({})*({}))", param, value)
        );

        let generator = CodeGenerator::new();
        let result = generator.generate_macro(&macro_def);

        prop_assert!(result.is_ok());
        let rust_code = result.unwrap();
        prop_assert!(rust_code.starts_with("#[inline]"));
    });
}
