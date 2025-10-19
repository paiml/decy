//! Macro Expansion Examples - Function-Like Macros
//!
//! This example demonstrates DECY's transpilation of C function-like macros
//! to Rust inline functions.
//!
//! Reference: K&R §4.11, ISO C99 §6.10.3

use decy_codegen::CodeGenerator;
use decy_hir::HirMacroDefinition;

fn main() {
    println!("=== Macro Expansion: Function-Like Macros ===\n");

    let generator = CodeGenerator::new();

    // ANCHOR: single_param
    // Single parameter macros
    let sqr_def = HirMacroDefinition::new_function_like(
        "SQR".to_string(),
        vec!["x".to_string()],
        "((x)*(x))".to_string(),
    );

    let double_def = HirMacroDefinition::new_function_like(
        "DOUBLE".to_string(),
        vec!["x".to_string()],
        "((x)*2)".to_string(),
    );

    let sqr_rust = generator
        .generate_macro(&sqr_def)
        .expect("Failed to generate SQR");
    let double_rust = generator
        .generate_macro(&double_def)
        .expect("Failed to generate DOUBLE");

    println!("C:    #define SQR(x) ((x)*(x))");
    println!("Rust:\n{}\n", sqr_rust);

    println!("C:    #define DOUBLE(x) ((x)*2)");
    println!("Rust:\n{}\n", double_rust);

    assert!(sqr_rust.contains("fn sqr(x: i32) -> i32"));
    assert!(sqr_rust.contains("x * x"));
    assert!(sqr_rust.contains("#[inline]"));

    assert!(double_rust.contains("fn double(x: i32) -> i32"));
    assert!(double_rust.contains("x * 2"));
    // ANCHOR_END: single_param

    // ANCHOR: two_param
    // Two parameter macros
    let add_def = HirMacroDefinition::new_function_like(
        "ADD".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "((a)+(b))".to_string(),
    );

    let mul_def = HirMacroDefinition::new_function_like(
        "MUL".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "((a)*(b))".to_string(),
    );

    let add_rust = generator
        .generate_macro(&add_def)
        .expect("Failed to generate ADD");
    let mul_rust = generator
        .generate_macro(&mul_def)
        .expect("Failed to generate MUL");

    println!("C:    #define ADD(a, b) ((a)+(b))");
    println!("Rust:\n{}\n", add_rust);

    println!("C:    #define MUL(a, b) ((a)*(b))");
    println!("Rust:\n{}\n", mul_rust);

    assert!(add_rust.contains("fn add(a: i32, b: i32) -> i32"));
    assert!(add_rust.contains("a + b"));

    assert!(mul_rust.contains("fn mul(a: i32, b: i32) -> i32"));
    assert!(mul_rust.contains("a * b"));
    // ANCHOR_END: two_param

    // ANCHOR: three_param
    // Three parameter macros
    let add3_def = HirMacroDefinition::new_function_like(
        "ADD3".to_string(),
        vec!["a".to_string(), "b".to_string(), "c".to_string()],
        "((a)+(b)+(c))".to_string(),
    );

    let add3_rust = generator
        .generate_macro(&add3_def)
        .expect("Failed to generate ADD3");

    println!("C:    #define ADD3(a, b, c) ((a)+(b)+(c))");
    println!("Rust:\n{}\n", add3_rust);

    assert!(add3_rust.contains("fn add3(a: i32, b: i32, c: i32) -> i32"));
    assert!(add3_rust.contains("a + b + c"));
    // ANCHOR_END: three_param

    println!("\n✅ All function-like macro transformations verified!");
}
