//! Macro Expansion Examples - Parentheses Cleanup
//!
//! This example demonstrates DECY's removal of unnecessary parentheses
//! from C macro bodies while preserving operator precedence.
//!
//! Reference: K&R §4.11, ISO C99 §6.10.3

use decy_codegen::CodeGenerator;
use decy_hir::HirMacroDefinition;

fn main() {
    println!("=== Macro Expansion: Parentheses Cleanup ===\n");

    let generator = CodeGenerator::new();

    // ANCHOR: parens_cleanup
    // C macros use defensive parentheses
    let sqr_def = HirMacroDefinition::new_function_like(
        "SQR".to_string(),
        vec!["x".to_string()],
        "((x)*(x))".to_string(), // Many parens in C
    );

    let sqr_rust = generator
        .generate_macro(&sqr_def)
        .expect("Failed to generate SQR");

    println!("Parentheses cleanup:");
    println!("C:    #define SQR(x) ((x)*(x))  // Defensive parens");
    println!("Rust:\n{}\n", sqr_rust);
    assert!(sqr_rust.contains("x * x")); // Clean, no extra parens

    // Complex expression with many parentheses
    let complex_def = HirMacroDefinition::new_function_like(
        "COMPLEX".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "(((a)+(b))*2)".to_string(),
    );

    let complex_rust = generator
        .generate_macro(&complex_def)
        .expect("Failed to generate COMPLEX");

    println!("Complex expression:");
    println!("C:    #define COMPLEX(a, b) (((a)+(b))*2)");
    println!("Rust:\n{}\n", complex_rust);
    // Outer parens removed, but operator precedence preserved
    assert!(complex_rust.contains("a + b * 2"));

    // Ternary with parentheses
    let max_def = HirMacroDefinition::new_function_like(
        "MAX".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "((a)>(b)?(a):(b))".to_string(),
    );

    let max_rust = generator
        .generate_macro(&max_def)
        .expect("Failed to generate MAX");

    println!("Ternary expression:");
    println!("C:    #define MAX(a, b) ((a)>(b)?(a):(b))");
    println!("Rust:\n{}\n", max_rust);
    // Clean if-else, no parens around variables
    assert!(max_rust.contains("if a > b { a } else { b }"));
    // ANCHOR_END: parens_cleanup

    println!("\n✅ Parentheses cleanup verified!");
}
