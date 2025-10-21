//! Codegen tests for function pointer → fn type generation (DECY-024 RED phase)
//!
//! This test suite verifies that function pointers are correctly transpiled to Rust fn types.
//!
//! References:
//! - K&R §5.11: Pointers to Functions
//! - ISO C99 §6.7.5.3: Function declarators

use decy_codegen::CodeGenerator;
use decy_hir::{HirFunction, HirParameter, HirType, HirTypedef};

#[test]
fn test_function_pointer_simple_codegen() {
    // Test that simple function pointer generates fn type
    // C: int (*callback)(int) → Rust: fn(i32) -> i32
    let fn_ptr_type = HirType::FunctionPointer {
        param_types: vec![HirType::Int],
        return_type: Box::new(HirType::Int),
    };

    let rust_type = CodeGenerator::map_type(&fn_ptr_type);

    assert_eq!(rust_type, "fn(i32) -> i32");
}

#[test]
fn test_function_pointer_multiple_params_codegen() {
    // Test that function pointer with multiple params generates correctly
    // C: int (*add)(int, int) → Rust: fn(i32, i32) -> i32
    let fn_ptr_type = HirType::FunctionPointer {
        param_types: vec![HirType::Int, HirType::Int],
        return_type: Box::new(HirType::Int),
    };

    let rust_type = CodeGenerator::map_type(&fn_ptr_type);

    assert_eq!(rust_type, "fn(i32, i32) -> i32");
}

#[test]
fn test_function_pointer_void_return_codegen() {
    // Test that function pointer with void return generates fn() syntax
    // C: void (*handler)(int) → Rust: fn(i32)
    let fn_ptr_type = HirType::FunctionPointer {
        param_types: vec![HirType::Int],
        return_type: Box::new(HirType::Void),
    };

    let rust_type = CodeGenerator::map_type(&fn_ptr_type);

    assert_eq!(rust_type, "fn(i32)");
}

#[test]
fn test_function_pointer_no_params_codegen() {
    // Test that function pointer with no parameters generates correctly
    // C: int (*get_value)(void) → Rust: fn() -> i32
    let fn_ptr_type = HirType::FunctionPointer {
        param_types: vec![],
        return_type: Box::new(HirType::Int),
    };

    let rust_type = CodeGenerator::map_type(&fn_ptr_type);

    assert_eq!(rust_type, "fn() -> i32");
}

#[test]
fn test_function_pointer_typedef_codegen() {
    // Test that typedef with function pointer generates type alias
    // C: typedef int (*Callback)(int, int); → Rust: type Callback = fn(i32, i32) -> i32;
    let fn_ptr_type = HirType::FunctionPointer {
        param_types: vec![HirType::Int, HirType::Int],
        return_type: Box::new(HirType::Int),
    };

    let typedef = HirTypedef::new("Callback".to_string(), fn_ptr_type);
    let generator = CodeGenerator::new();

    let rust_code = generator
        .generate_typedef(&typedef)
        .expect("Failed to generate typedef");

    assert!(rust_code.contains("type Callback = fn(i32, i32) -> i32;"));
}

#[test]
fn test_function_pointer_with_float_types_codegen() {
    // Test that function pointer with float types generates correctly
    // C: float (*compute)(float, float) → Rust: fn(f32, f32) -> f32
    let fn_ptr_type = HirType::FunctionPointer {
        param_types: vec![HirType::Float, HirType::Float],
        return_type: Box::new(HirType::Float),
    };

    let rust_type = CodeGenerator::map_type(&fn_ptr_type);

    assert_eq!(rust_type, "fn(f32, f32) -> f32");
}

#[test]
fn test_function_pointer_with_pointer_params_codegen() {
    // Test that function pointer with pointer parameters generates correctly
    // C: void (*process)(int*, char*) → Rust: fn(*mut i32, *mut i8)
    let fn_ptr_type = HirType::FunctionPointer {
        param_types: vec![
            HirType::Pointer(Box::new(HirType::Int)),
            HirType::Pointer(Box::new(HirType::Char)),
        ],
        return_type: Box::new(HirType::Void),
    };

    let rust_type = CodeGenerator::map_type(&fn_ptr_type);

    assert_eq!(rust_type, "fn(*mut i32, *mut u8)");
}

#[test]
fn test_function_pointer_as_parameter_codegen() {
    // Test that function with function pointer parameter generates correctly
    let callback_type = HirType::FunctionPointer {
        param_types: vec![HirType::Int],
        return_type: Box::new(HirType::Int),
    };

    let params = vec![
        HirParameter::new("value".to_string(), HirType::Int),
        HirParameter::new("operation".to_string(), callback_type),
    ];

    let func = HirFunction::new("apply".to_string(), HirType::Void, params);

    let generator = CodeGenerator::new();
    let rust_code = generator.generate_function(&func);

    // Should contain function pointer parameter
    assert!(rust_code.contains("operation: fn(i32) -> i32"));
}

#[test]
fn test_callback_pattern_typedef_codegen() {
    // Test common callback pattern using typedef
    // typedef int (*Callback)(int, int);
    // int invoke(Callback cb, int a, int b);
    let callback_typedef = HirTypedef::new(
        "Callback".to_string(),
        HirType::FunctionPointer {
            param_types: vec![HirType::Int, HirType::Int],
            return_type: Box::new(HirType::Int),
        },
    );

    let generator = CodeGenerator::new();
    let typedef_code = generator
        .generate_typedef(&callback_typedef)
        .expect("Failed to generate typedef");

    assert!(typedef_code.contains("type Callback = fn(i32, i32) -> i32;"));
}

#[test]
fn test_function_pointer_return_type_complex() {
    // Test function pointer with double return type
    // C: double (*calc)(double, double) → Rust: fn(f64, f64) -> f64
    let fn_ptr_type = HirType::FunctionPointer {
        param_types: vec![HirType::Double, HirType::Double],
        return_type: Box::new(HirType::Double),
    };

    let rust_type = CodeGenerator::map_type(&fn_ptr_type);

    assert_eq!(rust_type, "fn(f64, f64) -> f64");
}
