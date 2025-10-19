//! RED phase tests for constant macro expansion (DECY-098c)
//!
//! These tests verify that object-like macros (constants) are properly
//! transformed to Rust const declarations during code generation.
//!
//! Reference: K&R §4.11, ISO C99 §6.10.3
//!
//! Status: GREEN phase - implementation complete, tests should PASS

use decy_codegen::CodeGenerator;
use decy_hir::HirMacroDefinition;

#[test]
fn test_expand_simple_numeric_constant() {
    // #define MAX 100 → const MAX: i32 = 100;
    let macro_def = HirMacroDefinition::new_object_like("MAX".to_string(), "100".to_string());

    let generator = CodeGenerator::new();
    let rust_code = generator
        .generate_macro(&macro_def)
        .expect("Failed to generate");

    assert!(rust_code.contains("const MAX"));
    assert!(rust_code.contains("i32"));
    assert!(rust_code.contains("= 100"));
}

#[test]
fn test_expand_float_constant() {
    // #define PI 3.14159 → const PI: f64 = 3.14159;
    let macro_def = HirMacroDefinition::new_object_like("PI".to_string(), "3.14159".to_string());

    let generator = CodeGenerator::new();
    let rust_code = generator
        .generate_macro(&macro_def)
        .expect("Failed to generate");

    assert!(rust_code.contains("const PI"));
    assert!(rust_code.contains("f64"));
    assert!(rust_code.contains("= 3.14159"));
}

#[test]
fn test_expand_string_constant() {
    // #define GREETING "Hello" → const GREETING: &str = "Hello";
    let macro_def =
        HirMacroDefinition::new_object_like("GREETING".to_string(), "\"Hello\"".to_string());

    let generator = CodeGenerator::new();
    let rust_code = generator
        .generate_macro(&macro_def)
        .expect("Failed to generate");

    assert!(rust_code.contains("const GREETING"));
    assert!(rust_code.contains("&str"));
    assert!(rust_code.contains("\"Hello\""));
}

#[test]
fn test_expand_negative_constant() {
    // #define MIN_VALUE -100 → const MIN_VALUE: i32 = -100;
    let macro_def =
        HirMacroDefinition::new_object_like("MIN_VALUE".to_string(), "-100".to_string());

    let generator = CodeGenerator::new();
    let rust_code = generator
        .generate_macro(&macro_def)
        .expect("Failed to generate");

    assert!(rust_code.contains("const MIN_VALUE"));
    assert!(rust_code.contains("i32"));
    assert!(rust_code.contains("= -100"));
}

#[test]
fn test_expand_hex_constant() {
    // #define FLAGS 0xFF → const FLAGS: i32 = 0xFF;
    let macro_def = HirMacroDefinition::new_object_like("FLAGS".to_string(), "0xFF".to_string());

    let generator = CodeGenerator::new();
    let rust_code = generator
        .generate_macro(&macro_def)
        .expect("Failed to generate");

    assert!(rust_code.contains("const FLAGS"));
    assert!(rust_code.contains("0xFF") || rust_code.contains("255"));
}

#[test]
fn test_expand_multiple_constants() {
    // Multiple #define statements should each generate const declarations
    let macro1 = HirMacroDefinition::new_object_like("MAX".to_string(), "100".to_string());
    let macro2 = HirMacroDefinition::new_object_like("MIN".to_string(), "0".to_string());

    let generator = CodeGenerator::new();
    let rust1 = generator
        .generate_macro(&macro1)
        .expect("Failed to generate");
    let rust2 = generator
        .generate_macro(&macro2)
        .expect("Failed to generate");

    assert!(rust1.contains("const MAX"));
    assert!(rust2.contains("const MIN"));
}

#[test]
fn test_expand_empty_macro() {
    // #define EMPTY → Empty macros might be used for conditional compilation
    let macro_def = HirMacroDefinition::new_object_like("EMPTY".to_string(), "".to_string());

    let generator = CodeGenerator::new();
    let result = generator.generate_macro(&macro_def);

    // Empty macros should either generate nothing or a comment
    assert!(result.is_ok());
}

#[test]
fn test_expand_char_constant() {
    // #define NEWLINE '\n' → const NEWLINE: char = '\n';
    let macro_def = HirMacroDefinition::new_object_like("NEWLINE".to_string(), "'\\n'".to_string());

    let generator = CodeGenerator::new();
    let rust_code = generator
        .generate_macro(&macro_def)
        .expect("Failed to generate");

    assert!(rust_code.contains("const NEWLINE"));
    assert!(rust_code.contains("char") || rust_code.contains("'\\n'"));
}

#[test]
fn test_expand_boolean_constant() {
    // #define DEBUG 1 → const DEBUG: bool = true; (or i32 = 1)
    let macro_def = HirMacroDefinition::new_object_like("DEBUG".to_string(), "1".to_string());

    let generator = CodeGenerator::new();
    let rust_code = generator
        .generate_macro(&macro_def)
        .expect("Failed to generate");

    assert!(rust_code.contains("const DEBUG"));
    // Could be bool or i32, both valid
    assert!(rust_code.contains("= 1") || rust_code.contains("= true"));
}

#[test]
fn test_naming_convention_preserved() {
    // Macro names should be preserved in SCREAMING_SNAKE_CASE
    let macro_def =
        HirMacroDefinition::new_object_like("BUFFER_SIZE".to_string(), "1024".to_string());

    let generator = CodeGenerator::new();
    let rust_code = generator
        .generate_macro(&macro_def)
        .expect("Failed to generate");

    assert!(rust_code.contains("BUFFER_SIZE"));
    assert!(!rust_code.contains("buffer_size")); // Should NOT convert to snake_case
}
