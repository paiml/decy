//! Tests for typedef code generation (DECY-023 RED phase).

use super::*;
use decy_hir::{HirType, HirTypedef};

#[test]
fn test_generate_simple_typedef() {
    // typedef int Integer;
    let codegen = CodeGenerator::new();

    let typedef = HirTypedef::new("Integer".to_string(), HirType::Int);

    let code = codegen.generate_typedef(&typedef);

    assert!(code.contains("type Integer = i32"));
    assert!(code.ends_with(';'));
}

#[test]
fn test_generate_pointer_typedef() {
    // typedef int* IntPtr;
    let codegen = CodeGenerator::new();

    let typedef = HirTypedef::new(
        "IntPtr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );

    let code = codegen.generate_typedef(&typedef);

    assert!(code.contains("type IntPtr = *mut i32"));
}

#[test]
fn test_generate_array_typedef() {
    // typedef int IntArray[10];
    let codegen = CodeGenerator::new();

    let typedef = HirTypedef::new(
        "IntArray".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );

    let code = codegen.generate_typedef(&typedef);

    assert!(code.contains("type IntArray = [i32; 10]"));
}

#[test]
fn test_generate_struct_typedef() {
    // typedef struct Point Point;
    // In Rust, this is just the struct name itself, no typedef needed
    let codegen = CodeGenerator::new();

    let typedef = HirTypedef::new("Point".to_string(), HirType::Struct("Point".to_string()));

    let code = codegen.generate_typedef(&typedef);

    // For struct typedefs where name matches underlying type, we can omit or generate a comment
    // typedef struct Point Point; → // type Point = Point; (redundant in Rust)
    assert!(code.contains("Point"));
}

#[test]
fn test_generate_multiple_typedefs() {
    let codegen = CodeGenerator::new();

    let typedef1 = HirTypedef::new("Integer".to_string(), HirType::Int);
    let typedef2 = HirTypedef::new("FloatType".to_string(), HirType::Float);

    let code1 = codegen.generate_typedef(&typedef1);
    let code2 = codegen.generate_typedef(&typedef2);

    assert!(code1.contains("type Integer = i32"));
    assert!(code2.contains("type FloatType = f32"));
}
