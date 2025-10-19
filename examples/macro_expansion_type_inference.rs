//! Macro Expansion Examples - Type Inference
//!
//! This example demonstrates DECY's type inference for function-like macros.
//!
//! Reference: K&R §4.11, ISO C99 §6.10.3

use decy_codegen::CodeGenerator;
use decy_hir::HirMacroDefinition;

fn main() {
    println!("=== Macro Expansion: Type Inference ===\n");

    let generator = CodeGenerator::new();

    // ANCHOR: type_inference
    // Arithmetic expressions → i32
    let sqr_def = HirMacroDefinition::new_function_like(
        "SQR".to_string(),
        vec!["x".to_string()],
        "((x)*(x))".to_string(),
    );

    let sqr_rust = generator
        .generate_macro(&sqr_def)
        .expect("Failed to generate SQR");

    println!("Arithmetic expression:");
    println!("C:    #define SQR(x) ((x)*(x))");
    println!("Rust: {}\n", sqr_rust);
    assert!(sqr_rust.contains("-> i32"));

    // Comparison expressions → bool
    let is_positive_def = HirMacroDefinition::new_function_like(
        "IS_POSITIVE".to_string(),
        vec!["x".to_string()],
        "((x)>0)".to_string(),
    );

    let is_positive_rust = generator
        .generate_macro(&is_positive_def)
        .expect("Failed to generate IS_POSITIVE");

    println!("Comparison expression:");
    println!("C:    #define IS_POSITIVE(x) ((x)>0)");
    println!("Rust: {}\n", is_positive_rust);
    assert!(is_positive_rust.contains("-> bool"));

    // Logical expressions → bool
    let and_def = HirMacroDefinition::new_function_like(
        "AND".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "((a)&&(b))".to_string(),
    );

    let and_rust = generator
        .generate_macro(&and_def)
        .expect("Failed to generate AND");

    println!("Logical expression:");
    println!("C:    #define AND(a, b) ((a)&&(b))");
    println!("Rust: {}\n", and_rust);
    assert!(and_rust.contains("-> bool"));

    // Ternary expressions → i32 (type of branches, not condition)
    let max_def = HirMacroDefinition::new_function_like(
        "MAX".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "((a)>(b)?(a):(b))".to_string(),
    );

    let max_rust = generator
        .generate_macro(&max_def)
        .expect("Failed to generate MAX");

    println!("Ternary expression (returns values, not condition):");
    println!("C:    #define MAX(a, b) ((a)>(b)?(a):(b))");
    println!("Rust: {}\n", max_rust);
    assert!(max_rust.contains("-> i32")); // Not bool, even though contains comparison
    // ANCHOR_END: type_inference

    println!("\n✅ Type inference verified for all expression types!");
}
