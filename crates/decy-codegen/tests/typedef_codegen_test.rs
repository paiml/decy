//! Codegen tests for typedef → type alias generation (DECY-023 RED phase)
//!
//! This test suite verifies that typedefs are correctly transpiled to Rust type aliases.
//!
//! References:
//! - K&R §6.7: Type Names
//! - ISO C99 §6.7.7: Type definitions

use decy_codegen::CodeGenerator;
use decy_hir::{HirType, HirTypedef};

#[test]
fn test_typedef_simple_int_codegen() {
    // Test that simple typedef generates Rust type alias
    let typedef = HirTypedef::new("MyInt".to_string(), HirType::Int);
    let generator = CodeGenerator::new();

    let rust_code = generator
        .generate_typedef(&typedef)
        .expect("Failed to generate typedef");

    assert!(
        rust_code.contains("type MyInt = i32;"),
        "Expected 'type MyInt = i32;', got: {}",
        rust_code
    );
}

#[test]
fn test_typedef_float_codegen() {
    // Test that float typedef generates correct alias
    let typedef = HirTypedef::new("MyFloat".to_string(), HirType::Float);
    let generator = CodeGenerator::new();

    let rust_code = generator
        .generate_typedef(&typedef)
        .expect("Failed to generate typedef");

    assert!(
        rust_code.contains("type MyFloat = f32;"),
        "Expected 'type MyFloat = f32;', got: {}",
        rust_code
    );
}

#[test]
fn test_typedef_double_codegen() {
    // Test that double typedef generates f64
    let typedef = HirTypedef::new("MyDouble".to_string(), HirType::Double);
    let generator = CodeGenerator::new();

    let rust_code = generator
        .generate_typedef(&typedef)
        .expect("Failed to generate typedef");

    assert!(
        rust_code.contains("type MyDouble = f64;"),
        "Expected 'type MyDouble = f64;', got: {}",
        rust_code
    );
}

#[test]
fn test_typedef_pointer_codegen() {
    // Test that pointer typedef generates correct type
    let typedef = HirTypedef::new(
        "IntPtr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let generator = CodeGenerator::new();

    let rust_code = generator
        .generate_typedef(&typedef)
        .expect("Failed to generate typedef");

    assert!(
        rust_code.contains("type IntPtr = *mut i32;"),
        "Expected 'type IntPtr = *mut i32;', got: {}",
        rust_code
    );
}

#[test]
fn test_typedef_char_pointer_codegen() {
    // Test that char* typedef generates string-appropriate type
    let typedef = HirTypedef::new(
        "String".to_string(),
        HirType::Pointer(Box::new(HirType::Char)),
    );
    let generator = CodeGenerator::new();

    let rust_code = generator
        .generate_typedef(&typedef)
        .expect("Failed to generate typedef");

    // Should generate either *const i8/u8 or &str depending on context
    assert!(
        rust_code.contains("type String") && (rust_code.contains("i8") || rust_code.contains("u8")),
        "Expected string-related type, got: {}",
        rust_code
    );
}

#[test]
fn test_typedef_multiple_codegen() {
    // Test that multiple typedefs generate correctly
    let typedef1 = HirTypedef::new("Int32".to_string(), HirType::Int);
    let typedef2 = HirTypedef::new("Float32".to_string(), HirType::Float);

    let generator = CodeGenerator::new();

    let rust1 = generator
        .generate_typedef(&typedef1)
        .expect("Failed to generate typedef1");
    let rust2 = generator
        .generate_typedef(&typedef2)
        .expect("Failed to generate typedef2");

    assert!(rust1.contains("type Int32 = i32;"));
    assert!(rust2.contains("type Float32 = f32;"));
}

#[test]
fn test_typedef_struct_codegen() {
    // Test that struct typedef generates correct alias
    let typedef = HirTypedef::new(
        "Point".to_string(),
        HirType::Struct("Point".to_string()),
    );
    let generator = CodeGenerator::new();

    let rust_code = generator
        .generate_typedef(&typedef)
        .expect("Failed to generate typedef");

    assert!(
        rust_code.contains("type Point") && rust_code.contains("Point"),
        "Expected struct typedef, got: {}",
        rust_code
    );
}

#[test]
fn test_typedef_function_pointer_codegen() {
    // Test that function pointer typedef generates fn type
    let typedef = HirTypedef::new(
        "Callback".to_string(),
        HirType::FunctionPointer {
            param_types: vec![HirType::Int, HirType::Int],
            return_type: Box::new(HirType::Int),
        },
    );
    let generator = CodeGenerator::new();

    let rust_code = generator
        .generate_typedef(&typedef)
        .expect("Failed to generate typedef");

    assert!(
        rust_code.contains("type Callback = fn(i32, i32) -> i32;"),
        "Expected 'type Callback = fn(i32, i32) -> i32;', got: {}",
        rust_code
    );
}

#[test]
#[ignore]
fn test_typedef_array_codegen() {
    // RED: Test that array typedef generates correct type
    let typedef = HirTypedef::new(
        "IntArray".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    let generator = CodeGenerator::new();

    let rust_code = generator
        .generate_typedef(&typedef)
        .expect("Failed to generate typedef");

    assert!(
        rust_code.contains("type IntArray = [i32; 10];"),
        "Expected 'type IntArray = [i32; 10];', got: {}",
        rust_code
    );
}

#[test]
fn test_typedef_visibility() {
    // Test that generated typedefs have public visibility
    let typedef = HirTypedef::new("MyInt".to_string(), HirType::Int);
    let generator = CodeGenerator::new();

    let rust_code = generator
        .generate_typedef(&typedef)
        .expect("Failed to generate typedef");

    assert!(
        rust_code.contains("pub type MyInt"),
        "Expected public typedef, got: {}",
        rust_code
    );
}
