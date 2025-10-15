//! Tests for #define constant transformation (PREP-DEFINE-CONST validation)
//!
//! Reference: K&R §4.11, ISO C99 §6.10.3
//!
//! This module tests the transformation of C #define macro constants to Rust const declarations.
//! #define constants are compile-time text substitutions in C, which map naturally to Rust's
//! const declarations with compile-time type inference and evaluation.

use decy_codegen::CodeGenerator;
use decy_hir::{HirConstant, HirExpression, HirType};

/// Test simple integer #define constant
///
/// C: #define MAX 100
///
/// Rust: const MAX: i32 = 100;
///
/// Reference: K&R §4.11, ISO C99 §6.10.3.2
#[test]
fn test_simple_integer_define() {
    let codegen = CodeGenerator::new();

    let constant = HirConstant::new(
        "MAX".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(100),
    );

    let result = codegen.generate_constant(&constant);

    // Verify const declaration
    assert!(
        result.contains("const MAX: i32 = 100"),
        "Should generate const declaration"
    );

    // Should end with semicolon
    assert!(result.trim().ends_with(';'), "Should end with semicolon");

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test negative integer #define constant
///
/// C: #define MIN -100
///
/// Rust: const MIN: i32 = -100;
///
/// Reference: K&R §4.11, ISO C99 §6.10.3.2
#[test]
fn test_negative_integer_define() {
    let codegen = CodeGenerator::new();

    let constant = HirConstant::new(
        "MIN".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(-100),
    );

    let result = codegen.generate_constant(&constant);

    // Verify negative constant
    assert!(result.contains("const MIN: i32 = -100"));
    assert!(!result.contains("unsafe"));
}

/// Test string #define constant
///
/// C: #define MSG "Hello, World!"
///
/// Rust: const MSG: &str = "Hello, World!";
///
/// Reference: K&R §4.11, ISO C99 §6.10.3.2
#[test]
fn test_string_define() {
    let codegen = CodeGenerator::new();

    let constant = HirConstant::new(
        "MSG".to_string(),
        HirType::Pointer(Box::new(HirType::Char)), // char* → &str
        HirExpression::StringLiteral("Hello, World!".to_string()),
    );

    let result = codegen.generate_constant(&constant);

    // Verify string constant (should be &str, not String)
    assert!(
        result.contains("const MSG: &str = \"Hello, World!\""),
        "Should generate const &str declaration"
    );
    assert!(!result.contains("unsafe"));
}

/// Test hexadecimal #define constant
///
/// C: #define FLAGS 0xFF
///
/// Rust: const FLAGS: i32 = 0xFF;
///
/// Reference: K&R §4.11, ISO C99 §6.10.3.2
#[test]
fn test_hex_define() {
    let codegen = CodeGenerator::new();

    let constant = HirConstant::new(
        "FLAGS".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(0xFF),
    );

    let result = codegen.generate_constant(&constant);

    // Verify hex constant (can be decimal 255 or hex 0xFF in Rust)
    assert!(
        result.contains("const FLAGS: i32 = 255") || result.contains("const FLAGS: i32 = 0xFF"),
        "Should generate const with value 255 or 0xFF"
    );
    assert!(!result.contains("unsafe"));
}

/// Test expression #define constant
///
/// C: #define SIZE (10 * 20)
///
/// Rust: const SIZE: i32 = 10 * 20;
///
/// Note: Rust will evaluate this at compile-time
///
/// Reference: K&R §4.11, ISO C99 §6.10.3.2
#[test]
fn test_expression_define() {
    let codegen = CodeGenerator::new();

    let constant = HirConstant::new(
        "SIZE".to_string(),
        HirType::Int,
        HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::Multiply,
            left: Box::new(HirExpression::IntLiteral(10)),
            right: Box::new(HirExpression::IntLiteral(20)),
        },
    );

    let result = codegen.generate_constant(&constant);

    // Verify expression constant (can be evaluated or kept as expression)
    assert!(
        result.contains("const SIZE: i32 ="),
        "Should generate const declaration"
    );
    // Accept either "200" (evaluated) or "10 * 20" (expression)
    assert!(
        result.contains("200") || (result.contains("10") && result.contains("20")),
        "Should contain value or expression"
    );
    assert!(!result.contains("unsafe"));
}

/// Test #define with underscore naming convention
///
/// C: #define MAX_BUFFER_SIZE 1024
///
/// Rust: const MAX_BUFFER_SIZE: i32 = 1024;
///
/// Reference: K&R §4.11, ISO C99 §6.10.3.2
#[test]
fn test_define_with_underscores() {
    let codegen = CodeGenerator::new();

    let constant = HirConstant::new(
        "MAX_BUFFER_SIZE".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(1024),
    );

    let result = codegen.generate_constant(&constant);

    // Verify constant preserves naming (SCREAMING_SNAKE_CASE)
    assert!(
        result.contains("const MAX_BUFFER_SIZE: i32 = 1024"),
        "Should preserve constant naming"
    );
    assert!(!result.contains("unsafe"));
}

/// Test multiple #define constants
///
/// C: #define WIDTH 800
///    #define HEIGHT 600
///
/// Rust: const WIDTH: i32 = 800;
///       const HEIGHT: i32 = 600;
///
/// Reference: K&R §4.11, ISO C99 §6.10.3.2
#[test]
fn test_multiple_defines() {
    let codegen = CodeGenerator::new();

    let constant1 = HirConstant::new(
        "WIDTH".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(800),
    );
    let constant2 = HirConstant::new(
        "HEIGHT".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(600),
    );

    let result1 = codegen.generate_constant(&constant1);
    let result2 = codegen.generate_constant(&constant2);

    // Verify both constants are generated correctly
    assert!(result1.contains("const WIDTH: i32 = 800"));
    assert!(result2.contains("const HEIGHT: i32 = 600"));
    assert!(!result1.contains("unsafe"));
    assert!(!result2.contains("unsafe"));
}

/// Test #define constant in function context (usage)
///
/// C: #define MAX 100
///    int x = MAX;
///
/// Rust: const MAX: i32 = 100;
///       let x: i32 = MAX;
///
/// Reference: K&R §4.11, ISO C99 §6.10.3.2
#[test]
fn test_define_constant_usage() {
    let codegen = CodeGenerator::new();

    // First, generate the constant
    let constant = HirConstant::new(
        "MAX".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(100),
    );
    let const_result = codegen.generate_constant(&constant);

    // Verify constant is generated
    assert!(const_result.contains("const MAX: i32 = 100"));

    // Usage in variable declaration is just Variable("MAX")
    // which should be generated as "MAX" in Rust code
    let _usage = HirExpression::Variable("MAX".to_string());

    // We'll verify this compiles with the constant in scope
    assert!(!const_result.contains("unsafe"));
}

/// Test zero-valued #define constant
///
/// C: #define NULL_VALUE 0
///
/// Rust: const NULL_VALUE: i32 = 0;
///
/// Reference: K&R §4.11, ISO C99 §6.10.3.2
#[test]
fn test_zero_define() {
    let codegen = CodeGenerator::new();

    let constant = HirConstant::new(
        "NULL_VALUE".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(0),
    );

    let result = codegen.generate_constant(&constant);

    // Verify zero constant
    assert!(result.contains("const NULL_VALUE: i32 = 0"));
    assert!(!result.contains("unsafe"));
}

/// Test #define constant with addition expression
///
/// C: #define OFFSET (BASE + 10)
///
/// Rust: const OFFSET: i32 = BASE + 10;
///
/// Note: This assumes BASE is also a const
///
/// Reference: K&R §4.11, ISO C99 §6.10.3.2
#[test]
fn test_define_with_const_reference() {
    let codegen = CodeGenerator::new();

    let constant = HirConstant::new(
        "OFFSET".to_string(),
        HirType::Int,
        HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("BASE".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        },
    );

    let result = codegen.generate_constant(&constant);

    // Verify constant expression with reference to another constant
    assert!(
        result.contains("const OFFSET: i32 ="),
        "Should generate const declaration"
    );
    assert!(
        result.contains("BASE") && result.contains("10"),
        "Should reference BASE constant"
    );
    assert!(!result.contains("unsafe"));
}

/// Verify unsafe block count remains 0
///
/// This is critical for the validation goal: <5 unsafe blocks per 1000 LOC
#[test]
fn test_define_const_transformation_unsafe_count() {
    let codegen = CodeGenerator::new();

    let constant1 = HirConstant::new(
        "MAX".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(100),
    );
    let constant2 = HirConstant::new(
        "MSG".to_string(),
        HirType::Pointer(Box::new(HirType::Char)),
        HirExpression::StringLiteral("Hello".to_string()),
    );

    let result1 = codegen.generate_constant(&constant1);
    let result2 = codegen.generate_constant(&constant2);

    // Combine results
    let combined = format!("{}\n{}", result1, result2);

    // Count unsafe blocks (should be 0)
    let unsafe_count = combined.matches("unsafe").count();
    assert_eq!(
        unsafe_count, 0,
        "#define → const transformation should not introduce unsafe blocks"
    );
}
