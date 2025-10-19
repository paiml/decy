//! RED phase tests for function-like macro expansion (DECY-098d)
//!
//! These tests verify that function-like macros are properly transformed
//! to Rust inline functions during code generation.
//!
//! Reference: K&R §4.11, ISO C99 §6.10.3
//!
//! Status: GREEN phase - implementation complete, tests should PASS

use decy_codegen::CodeGenerator;
use decy_hir::HirMacroDefinition;

#[test]

fn test_single_parameter_expression_macro() {
    // #define SQR(x) ((x) * (x)) → fn sqr(x: i32) -> i32 { x * x }
    let macro_def = HirMacroDefinition::new_function_like(
        "SQR".to_string(),
        vec!["x".to_string()],
        "((x)*(x))".to_string(),
    );

    let generator = CodeGenerator::new();
    let rust_code = generator
        .generate_macro(&macro_def)
        .expect("Failed to generate");

    // Should generate an inline function
    assert!(rust_code.contains("fn sqr"));
    assert!(rust_code.contains("x: i32"));
    assert!(rust_code.contains("-> i32"));
    assert!(rust_code.contains("x * x"));
}

#[test]

fn test_two_parameter_expression_macro() {
    // #define MAX(a, b) ((a) > (b) ? (a) : (b)) → fn max(a: i32, b: i32) -> i32 { if a > b { a } else { b } }
    let macro_def = HirMacroDefinition::new_function_like(
        "MAX".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "((a)>(b)?(a):(b))".to_string(),
    );

    let generator = CodeGenerator::new();
    let rust_code = generator
        .generate_macro(&macro_def)
        .expect("Failed to generate");

    // Should generate an inline function with ternary → if-else
    assert!(rust_code.contains("fn max"));
    assert!(rust_code.contains("a: i32"));
    assert!(rust_code.contains("b: i32"));
    assert!(rust_code.contains("-> i32"));
    assert!(rust_code.contains("if") && rust_code.contains("else"));
}

#[test]

fn test_three_parameter_expression_macro() {
    // #define ADD3(a, b, c) ((a) + (b) + (c)) → fn add3(a: i32, b: i32, c: i32) -> i32 { a + b + c }
    let macro_def = HirMacroDefinition::new_function_like(
        "ADD3".to_string(),
        vec!["a".to_string(), "b".to_string(), "c".to_string()],
        "((a)+(b)+(c))".to_string(),
    );

    let generator = CodeGenerator::new();
    let rust_code = generator
        .generate_macro(&macro_def)
        .expect("Failed to generate");

    assert!(rust_code.contains("fn add3"));
    assert!(rust_code.contains("a: i32"));
    assert!(rust_code.contains("b: i32"));
    assert!(rust_code.contains("c: i32"));
    assert!(rust_code.contains("a + b + c"));
}

#[test]

fn test_macro_name_converted_to_snake_case() {
    // #define IS_POSITIVE(x) ((x) > 0) → fn is_positive(x: i32) -> bool
    let macro_def = HirMacroDefinition::new_function_like(
        "IS_POSITIVE".to_string(),
        vec!["x".to_string()],
        "((x)>0)".to_string(),
    );

    let generator = CodeGenerator::new();
    let rust_code = generator
        .generate_macro(&macro_def)
        .expect("Failed to generate");

    // Macro name should be converted to snake_case for Rust function
    assert!(rust_code.contains("fn is_positive"));
    assert!(!rust_code.contains("IS_POSITIVE")); // Should not keep SCREAMING_SNAKE_CASE
}

#[test]

fn test_arithmetic_expression_macro() {
    // #define DOUBLE(x) ((x) * 2) → fn double(x: i32) -> i32 { x * 2 }
    let macro_def = HirMacroDefinition::new_function_like(
        "DOUBLE".to_string(),
        vec!["x".to_string()],
        "((x)*2)".to_string(),
    );

    let generator = CodeGenerator::new();
    let rust_code = generator
        .generate_macro(&macro_def)
        .expect("Failed to generate");

    assert!(rust_code.contains("fn double"));
    assert!(rust_code.contains("x * 2"));
}

#[test]

fn test_comparison_expression_macro() {
    // #define IS_ZERO(x) ((x) == 0) → fn is_zero(x: i32) -> bool { x == 0 }
    let macro_def = HirMacroDefinition::new_function_like(
        "IS_ZERO".to_string(),
        vec!["x".to_string()],
        "((x)==0)".to_string(),
    );

    let generator = CodeGenerator::new();
    let rust_code = generator
        .generate_macro(&macro_def)
        .expect("Failed to generate");

    assert!(rust_code.contains("fn is_zero"));
    assert!(rust_code.contains("-> bool"));
    assert!(rust_code.contains("x == 0"));
}

#[test]

fn test_parentheses_preserved_in_expression() {
    // #define ABS(x) ((x) < 0 ? -(x) : (x)) → fn abs(x: i32) -> i32 { if x < 0 { -x } else { x } }
    let macro_def = HirMacroDefinition::new_function_like(
        "ABS".to_string(),
        vec!["x".to_string()],
        "((x)<0?-(x):(x))".to_string(),
    );

    let generator = CodeGenerator::new();
    let rust_code = generator
        .generate_macro(&macro_def)
        .expect("Failed to generate");

    assert!(rust_code.contains("fn abs"));
    assert!(rust_code.contains("if") && rust_code.contains("else"));
    assert!(rust_code.contains("-x"));
}

#[test]

fn test_inline_attribute_added() {
    // All generated functions should have #[inline] for performance
    let macro_def = HirMacroDefinition::new_function_like(
        "IDENTITY".to_string(),
        vec!["x".to_string()],
        "(x)".to_string(),
    );

    let generator = CodeGenerator::new();
    let rust_code = generator
        .generate_macro(&macro_def)
        .expect("Failed to generate");

    assert!(rust_code.contains("#[inline]"));
}

#[test]

fn test_multiple_use_of_same_parameter() {
    // #define SUM_TWICE(x) ((x) + (x)) → fn sum_twice(x: i32) -> i32 { x + x }
    // Note: This should potentially warn about multiple evaluation
    let macro_def = HirMacroDefinition::new_function_like(
        "SUM_TWICE".to_string(),
        vec!["x".to_string()],
        "((x)+(x))".to_string(),
    );

    let generator = CodeGenerator::new();
    let rust_code = generator
        .generate_macro(&macro_def)
        .expect("Failed to generate");

    assert!(rust_code.contains("fn sum_twice"));
    assert!(rust_code.contains("x + x"));
}

#[test]

fn test_logical_expression_macro() {
    // #define AND(a, b) ((a) && (b)) → fn and(a: bool, b: bool) -> bool { a && b }
    let macro_def = HirMacroDefinition::new_function_like(
        "AND".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "((a)&&(b))".to_string(),
    );

    let generator = CodeGenerator::new();
    let rust_code = generator
        .generate_macro(&macro_def)
        .expect("Failed to generate");

    assert!(rust_code.contains("fn and"));
    assert!(rust_code.contains("bool"));
    assert!(rust_code.contains("a && b"));
}
