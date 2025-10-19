//! Macro Expansion Examples - Ternary Operator Transformation
//!
//! This example demonstrates DECY's transpilation of C ternary operators
//! in macros to Rust if-else expressions.
//!
//! Reference: K&R §4.11, ISO C99 §6.10.3

use decy_codegen::CodeGenerator;
use decy_hir::HirMacroDefinition;

fn main() {
    println!("=== Macro Expansion: Ternary Operator Transformation ===\n");

    let generator = CodeGenerator::new();

    // ANCHOR: max_macro
    // MAX macro with ternary operator
    let max_def = HirMacroDefinition::new_function_like(
        "MAX".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "((a)>(b)?(a):(b))".to_string(),
    );

    let max_rust = generator
        .generate_macro(&max_def)
        .expect("Failed to generate MAX");

    println!("C:    #define MAX(a, b) ((a)>(b)?(a):(b))");
    println!("Rust:\n{}\n", max_rust);

    assert!(max_rust.contains("fn max(a: i32, b: i32) -> i32"));
    assert!(max_rust.contains("if a > b { a } else { b }"));
    assert!(max_rust.contains("#[inline]"));
    // ANCHOR_END: max_macro

    // ANCHOR: min_macro
    // MIN macro with ternary operator
    let min_def = HirMacroDefinition::new_function_like(
        "MIN".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "((a)<(b)?(a):(b))".to_string(),
    );

    let min_rust = generator
        .generate_macro(&min_def)
        .expect("Failed to generate MIN");

    println!("C:    #define MIN(a, b) ((a)<(b)?(a):(b))");
    println!("Rust:\n{}\n", min_rust);

    assert!(min_rust.contains("fn min(a: i32, b: i32) -> i32"));
    assert!(min_rust.contains("if a < b { a } else { b }"));
    // ANCHOR_END: min_macro

    // ANCHOR: abs_macro
    // ABS macro with ternary operator and negation
    let abs_def = HirMacroDefinition::new_function_like(
        "ABS".to_string(),
        vec!["x".to_string()],
        "((x)<0?-(x):(x))".to_string(),
    );

    let abs_rust = generator
        .generate_macro(&abs_def)
        .expect("Failed to generate ABS");

    println!("C:    #define ABS(x) ((x)<0?-(x):(x))");
    println!("Rust:\n{}\n", abs_rust);

    assert!(abs_rust.contains("fn abs(x: i32) -> i32"));
    assert!(abs_rust.contains("if x < 0 { -x } else { x }"));
    // Note: Verifies unary minus is preserved without space: "-x" not "- x"
    assert!(abs_rust.contains("-x"));
    assert!(!abs_rust.contains("- x"));
    // ANCHOR_END: abs_macro

    println!("\n✅ All ternary operator transformations verified!");
}
