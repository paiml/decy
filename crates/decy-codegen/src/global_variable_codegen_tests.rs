//! RED Phase Tests for Global Variable Code Generation
//!
//! DECY-068: Codegen for Sprint 19 parser features (global variables)
//!
//! These tests are written FIRST (RED phase) and should FAIL initially.
//! They define the expected behavior for global variable code generation.

#![cfg(test)]

use crate::CodeGenerator;
use decy_hir::{HirConstant, HirExpression, HirType};

/// RED: Test code generation for static global variable
///
/// C code:
/// ```c
/// static int counter = 0;
/// ```
///
/// Expected Rust:
/// ```rust
/// static mut counter: i32 = 0;
/// ```
#[test]
fn test_codegen_static_global_variable() {
    let codegen = CodeGenerator::new();

    // Create a constant representing a global variable (we'll need a better structure)
    let global_var = HirConstant::new(
        "counter".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(0),
    );

    // This should generate: static mut counter: i32 = 0;
    let rust_code = codegen.generate_global_variable(&global_var, true, false, false);

    assert!(rust_code.contains("static mut counter"));
    assert!(rust_code.contains(": i32"));
    assert!(rust_code.contains("= 0"));
}

/// RED: Test code generation for extern global variable
///
/// C code:
/// ```c
/// extern int global_count;
/// ```
///
/// Expected Rust:
/// ```rust
/// extern "C" {
///     static global_count: i32;
/// }
/// ```
#[test]
fn test_codegen_extern_global_variable() {
    let codegen = CodeGenerator::new();

    let global_var = HirConstant::new(
        "global_count".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(0), // No initializer for extern
    );

    // is_static=false, is_extern=true, is_const=false
    let rust_code = codegen.generate_global_variable(&global_var, false, true, false);

    assert!(rust_code.contains("extern \"C\""));
    assert!(rust_code.contains("static global_count"));
    assert!(rust_code.contains(": i32"));
}

/// RED: Test code generation for const global variable
///
/// C code:
/// ```c
/// const int MAX_SIZE = 100;
/// ```
///
/// Expected Rust:
/// ```rust
/// const MAX_SIZE: i32 = 100;
/// ```
#[test]
fn test_codegen_const_global_variable() {
    let codegen = CodeGenerator::new();

    let global_var = HirConstant::new(
        "MAX_SIZE".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(100),
    );

    // is_static=false, is_extern=false, is_const=true
    let rust_code = codegen.generate_global_variable(&global_var, false, false, true);

    assert!(rust_code.contains("const MAX_SIZE"));
    assert!(rust_code.contains(": i32"));
    assert!(rust_code.contains("= 100"));
    assert!(!rust_code.contains("static"));
}

/// RED: Test code generation for static const global variable
///
/// C code:
/// ```c
/// static const double PI = 3.14159;
/// ```
///
/// Expected Rust:
/// ```rust
/// const PI: f64 = 3.14159;
/// ```
#[test]
fn test_codegen_static_const_global() {
    let codegen = CodeGenerator::new();

    let global_var = HirConstant::new(
        "PI".to_string(),
        HirType::Double,
        HirExpression::FloatLiteral(3.14159),
    );

    // is_static=true, is_extern=false, is_const=true
    // In Rust, static const → const (no need for static)
    let rust_code = codegen.generate_global_variable(&global_var, true, false, true);

    assert!(rust_code.contains("const PI"));
    assert!(rust_code.contains(": f64"));
    assert!(rust_code.contains("3.14159"));
}

/// RED: Test code generation for plain global variable (no storage class)
///
/// C code:
/// ```c
/// int global_flag = 1;
/// ```
///
/// Expected Rust:
/// ```rust
/// static mut global_flag: i32 = 1;
/// ```
#[test]
fn test_codegen_plain_global_variable() {
    let codegen = CodeGenerator::new();

    let global_var = HirConstant::new(
        "global_flag".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(1),
    );

    // is_static=false, is_extern=false, is_const=false
    // Plain global → static mut in Rust
    let rust_code = codegen.generate_global_variable(&global_var, false, false, false);

    assert!(rust_code.contains("static mut global_flag"));
    assert!(rust_code.contains(": i32"));
    assert!(rust_code.contains("= 1"));
}

/// RED: Test code generation for global pointer variable
///
/// C code:
/// ```c
/// static int* ptr = NULL;
/// ```
///
/// Expected Rust:
/// ```rust
/// static mut ptr: *mut i32 = std::ptr::null_mut();
/// ```
#[test]
fn test_codegen_global_pointer_variable() {
    let codegen = CodeGenerator::new();

    let global_var = HirConstant::new(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        HirExpression::IntLiteral(0), // Represents NULL
    );

    // is_static=true, is_extern=false, is_const=false
    let rust_code = codegen.generate_global_variable(&global_var, true, false, false);

    assert!(rust_code.contains("static mut ptr"));
    assert!(rust_code.contains("*mut i32"));
    assert!(rust_code.contains("null_mut()") || rust_code.contains("0"));
}

/// RED: Test code generation for global array variable
///
/// C code:
/// ```c
/// static int buffer[10];
/// ```
///
/// Expected Rust:
/// ```rust
/// static mut buffer: [i32; 10] = [0; 10];
/// ```
#[test]
fn test_codegen_global_array_variable() {
    let codegen = CodeGenerator::new();

    let global_var = HirConstant::new(
        "buffer".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
        HirExpression::IntLiteral(0), // Default initialization
    );

    // is_static=true, is_extern=false, is_const=false
    let rust_code = codegen.generate_global_variable(&global_var, true, false, false);

    assert!(rust_code.contains("static mut buffer"));
    assert!(rust_code.contains("[i32; 10]"));
    assert!(rust_code.contains("[0; 10]"));
}

/// RED: Test code generation for const string literal global
///
/// C code:
/// ```c
/// const char* message = "Hello, World!";
/// ```
///
/// Expected Rust:
/// ```rust
/// const message: &str = "Hello, World!";
/// ```
#[test]
fn test_codegen_const_string_global() {
    let codegen = CodeGenerator::new();

    let global_var = HirConstant::new(
        "message".to_string(),
        HirType::Pointer(Box::new(HirType::Char)),
        HirExpression::StringLiteral("Hello, World!".to_string()),
    );

    // is_static=false, is_extern=false, is_const=true
    let rust_code = codegen.generate_global_variable(&global_var, false, false, true);

    assert!(rust_code.contains("const message"));
    assert!(rust_code.contains("&str") || rust_code.contains("&'static str"));
    assert!(rust_code.contains("Hello, World!"));
}
