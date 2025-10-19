//! Macro Expansion Examples - Name Conversion
//!
//! This example demonstrates DECY's naming convention conversion:
//! - Constants: Keep SCREAMING_SNAKE_CASE
//! - Functions: Convert to snake_case
//!
//! Reference: K&R §4.11, ISO C99 §6.10.3

use decy_codegen::CodeGenerator;
use decy_hir::HirMacroDefinition;

fn main() {
    println!("=== Macro Expansion: Name Conversion ===\n");

    let generator = CodeGenerator::new();

    // ANCHOR: name_conversion
    // Object-like macros: Keep SCREAMING_SNAKE_CASE
    let max_const = HirMacroDefinition::new_object_like("MAX_VALUE".to_string(), "100".to_string());

    let max_const_rust = generator
        .generate_macro(&max_const)
        .expect("Failed to generate MAX_VALUE");

    println!("Constant macro (keeps SCREAMING_SNAKE_CASE):");
    println!("C:    #define MAX_VALUE 100");
    println!("Rust: {}\n", max_const_rust);
    assert!(max_const_rust.contains("const MAX_VALUE"));

    // Function-like macros: Convert to snake_case
    let is_positive = HirMacroDefinition::new_function_like(
        "IS_POSITIVE".to_string(),
        vec!["x".to_string()],
        "((x)>0)".to_string(),
    );

    let is_positive_rust = generator
        .generate_macro(&is_positive)
        .expect("Failed to generate IS_POSITIVE");

    println!("Function macro (converts to snake_case):");
    println!("C:    #define IS_POSITIVE(x) ((x)>0)");
    println!("Rust:\n{}\n", is_positive_rust);
    assert!(is_positive_rust.contains("fn is_positive"));
    assert!(!is_positive_rust.contains("IS_POSITIVE")); // Original name not present

    // More complex name conversion
    let get_max_value = HirMacroDefinition::new_function_like(
        "GET_MAX_VALUE".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "((a)>(b)?(a):(b))".to_string(),
    );

    let get_max_value_rust = generator
        .generate_macro(&get_max_value)
        .expect("Failed to generate GET_MAX_VALUE");

    println!("Complex name conversion:");
    println!("C:    #define GET_MAX_VALUE(a, b) ((a)>(b)?(a):(b))");
    println!("Rust:\n{}\n", get_max_value_rust);
    assert!(get_max_value_rust.contains("fn get_max_value"));
    // ANCHOR_END: name_conversion

    println!("\n✅ Name conversion verified for constants and functions!");
}
