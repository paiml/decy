//! DECY-086: Fixed-size array translation tests.
//!
//! Tests for transforming C fixed-size arrays to Rust [T; N] arrays.

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};

/// Test basic fixed-size int array declaration.
/// C: int arr[10];
/// Rust: let mut arr: [i32; 10] = [0i32; 10];
#[test]
fn test_fixed_size_int_array_declaration() {
    let func = HirFunction::new_with_body(
        "test_array".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(10),
            },
            initializer: None,
        }],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should generate [i32; 10] type
    assert!(
        code.contains("[i32; 10]"),
        "Expected [i32; 10] in:\n{}",
        code
    );
    // Should have default initialization
    assert!(
        code.contains("[0i32; 10]"),
        "Expected [0i32; 10] initialization in:\n{}",
        code
    );
}

/// Test fixed-size char array declaration.
/// C: char buf[100];
/// Rust: let mut buf: [u8; 100] = [0u8; 100];
#[test]
fn test_fixed_size_char_array_declaration() {
    let func = HirFunction::new_with_body(
        "test_char_array".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "buf".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Char),
                size: Some(100),
            },
            initializer: None,
        }],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    assert!(
        code.contains("[u8; 100]"),
        "Expected [u8; 100] in:\n{}",
        code
    );
    assert!(
        code.contains("[0u8; 100]"),
        "Expected [0u8; 100] initialization in:\n{}",
        code
    );
}

/// Test fixed-size float array declaration.
/// C: float values[5];
/// Rust: let mut values: [f32; 5] = [0.0f32; 5];
#[test]
fn test_fixed_size_float_array_declaration() {
    let func = HirFunction::new_with_body(
        "test_float_array".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "values".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Float),
                size: Some(5),
            },
            initializer: None,
        }],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    assert!(
        code.contains("[f32; 5]"),
        "Expected [f32; 5] in:\n{}",
        code
    );
    assert!(
        code.contains("[0.0f32; 5]"),
        "Expected [0.0f32; 5] initialization in:\n{}",
        code
    );
}

/// Test array parameter becomes slice reference in signature.
/// C: void process(int arr[10], int len)
/// Rust: fn process(arr: &[i32])  (length inferred from slice)
#[test]
fn test_fixed_size_array_parameter_becomes_slice() {
    let func = HirFunction::new(
        "process".to_string(),
        HirType::Void,
        vec![
            HirParameter::new(
                "arr".to_string(),
                HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: Some(10),
                },
            ),
            HirParameter::new("len".to_string(), HirType::Int),
        ],
    );

    let codegen = CodeGenerator::new();
    let sig = codegen.generate_signature(&func);

    // Array parameters should become slice references, not [T; N]
    assert!(
        sig.contains("&[i32]") || sig.contains("&mut [i32]"),
        "Expected slice reference in signature:\n{}",
        sig
    );
}

/// Test array indexing in function body.
/// C: arr[i] = 42;
/// Rust: arr[i as usize] = 42;
#[test]
fn test_array_index_assignment() {
    let func = HirFunction::new_with_body(
        "test_index".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Array {
                    element_type: Box::new(HirType::Int),
                    size: Some(10),
                },
                initializer: None,
            },
            HirStatement::ArrayIndexAssignment {
                array: Box::new(HirExpression::Variable("arr".to_string())),
                index: Box::new(HirExpression::Variable("i".to_string())),
                value: HirExpression::IntLiteral(42),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should cast index to usize for safe indexing
    assert!(
        code.contains("as usize]"),
        "Expected usize cast for array index in:\n{}",
        code
    );
}

/// Test array return type mapping.
/// C: int[5] (conceptual)
/// Rust: [i32; 5]
#[test]
fn test_array_type_mapping() {
    let array_type = HirType::Array {
        element_type: Box::new(HirType::Int),
        size: Some(5),
    };

    let rust_type = CodeGenerator::map_type(&array_type);

    assert_eq!(rust_type, "[i32; 5]", "Type mapping failed");
}

/// Test nested/multi-dimensional array type mapping.
/// C: int matrix[3][4];
/// Rust: [[i32; 4]; 3]
#[test]
fn test_multidimensional_array_type_mapping() {
    let inner_array = HirType::Array {
        element_type: Box::new(HirType::Int),
        size: Some(4),
    };
    let outer_array = HirType::Array {
        element_type: Box::new(inner_array),
        size: Some(3),
    };

    let rust_type = CodeGenerator::map_type(&outer_array);

    assert_eq!(rust_type, "[[i32; 4]; 3]", "Nested array type mapping failed");
}

/// Test double array declaration.
/// C: double coords[3];
/// Rust: let mut coords: [f64; 3] = [0.0f64; 3];
#[test]
fn test_fixed_size_double_array_declaration() {
    let func = HirFunction::new_with_body(
        "test_double_array".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "coords".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Double),
                size: Some(3),
            },
            initializer: None,
        }],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    assert!(
        code.contains("[f64; 3]"),
        "Expected [f64; 3] in:\n{}",
        code
    );
    assert!(
        code.contains("[0.0f64; 3]"),
        "Expected [0.0f64; 3] initialization in:\n{}",
        code
    );
}
