//! Macro Expansion Examples - Operator Spacing
//!
//! This example demonstrates DECY's smart operator spacing:
//! - Binary operators: Add spaces (a+b → a + b)
//! - Unary operators: No spaces (-(x) → -x, not - x)
//!
//! Reference: K&R §4.11, ISO C99 §6.10.3

use decy_codegen::CodeGenerator;
use decy_hir::HirMacroDefinition;

fn main() {
    println!("=== Macro Expansion: Operator Spacing ===\n");

    let generator = CodeGenerator::new();

    // ANCHOR: operator_spacing
    // Binary operators get spaces
    let add_def = HirMacroDefinition::new_function_like(
        "ADD".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "((a)+(b))".to_string(), // No spaces in C
    );

    let add_rust = generator
        .generate_macro(&add_def)
        .expect("Failed to generate ADD");

    println!("Binary operators (add spaces):");
    println!("C:    #define ADD(a, b) ((a)+(b))");
    println!("Rust:\n{}\n", add_rust);
    assert!(add_rust.contains("a + b")); // Spaces added

    // Unary minus: No space
    let abs_def = HirMacroDefinition::new_function_like(
        "ABS".to_string(),
        vec!["x".to_string()],
        "((x)<0?-(x):(x))".to_string(),
    );

    let abs_rust = generator
        .generate_macro(&abs_def)
        .expect("Failed to generate ABS");

    println!("Unary operators (no spaces):");
    println!("C:    #define ABS(x) ((x)<0?-(x):(x))");
    println!("Rust:\n{}\n", abs_rust);
    assert!(abs_rust.contains("-x")); // No space after minus
    assert!(!abs_rust.contains("- x")); // Verify no space

    // Mixed operators
    let complex_def = HirMacroDefinition::new_function_like(
        "COMPLEX".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "((a)*2+(b)*3)".to_string(),
    );

    let complex_rust = generator
        .generate_macro(&complex_def)
        .expect("Failed to generate COMPLEX");

    println!("Mixed operators:");
    println!("C:    #define COMPLEX(a, b) ((a)*2+(b)*3)");
    println!("Rust:\n{}\n", complex_rust);
    assert!(complex_rust.contains("a * 2 + b * 3")); // All binary ops spaced
    // ANCHOR_END: operator_spacing

    println!("\n✅ Operator spacing verified!");
}
