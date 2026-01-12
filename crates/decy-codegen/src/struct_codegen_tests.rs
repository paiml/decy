//! Tests for struct and enum code generation (DECY-020).

use super::*;
use decy_hir::{HirEnum, HirEnumVariant, HirStruct, HirStructField, HirType};

#[test]
fn test_generate_simple_struct() {
    let codegen = CodeGenerator::new();

    let fields = vec![
        HirStructField::new("x".to_string(), HirType::Int),
        HirStructField::new("y".to_string(), HirType::Int),
    ];

    let point_struct = HirStruct::new("Point".to_string(), fields);
    let code = codegen.generate_struct(&point_struct);

    assert!(code.contains("struct Point"));
    assert!(code.contains("x: i32"));
    assert!(code.contains("y: i32"));
}

#[test]
fn test_generate_struct_with_derive() {
    let codegen = CodeGenerator::new();

    let fields = vec![HirStructField::new("value".to_string(), HirType::Int)];

    let simple_struct = HirStruct::new("Simple".to_string(), fields);
    let code = codegen.generate_struct(&simple_struct);

    // DECY-114: Struct now derives Default
    // DECY-225: Struct with only Copy types gets Copy derive
    assert!(code.contains("#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]"));
    assert!(code.contains("pub struct Simple"));
}

#[test]
fn test_generate_simple_enum() {
    let codegen = CodeGenerator::new();

    let variants = vec![
        HirEnumVariant::new("Red".to_string(), None),
        HirEnumVariant::new("Green".to_string(), None),
        HirEnumVariant::new("Blue".to_string(), None),
    ];

    let color_enum = HirEnum::new("Color".to_string(), variants);
    let code = codegen.generate_enum(&color_enum);

    // C enums are represented as type alias + constants for FFI compatibility
    assert!(code.contains("pub type Color = i32"));
    assert!(code.contains("pub const Red: i32"));
    assert!(code.contains("pub const Green: i32"));
    assert!(code.contains("pub const Blue: i32"));
}

#[test]
fn test_generate_enum_with_values() {
    let codegen = CodeGenerator::new();

    let variants = vec![
        HirEnumVariant::new("Low".to_string(), Some(0)),
        HirEnumVariant::new("High".to_string(), Some(10)),
    ];

    let priority_enum = HirEnum::new("Priority".to_string(), variants);
    let code = codegen.generate_enum(&priority_enum);

    // C enums with explicit values
    assert!(code.contains("pub const Low: i32 = 0"));
    assert!(code.contains("pub const High: i32 = 10"));
}

#[test]
fn test_generate_field_access() {
    use decy_hir::HirExpression;

    let codegen = CodeGenerator::new();

    let expr = HirExpression::FieldAccess {
        object: Box::new(HirExpression::Variable("point".to_string())),
        field: "x".to_string(),
    };

    let code = codegen.generate_expression(&expr);
    assert_eq!(code, "point.x");
}

#[test]
fn test_generate_pointer_field_access() {
    use decy_hir::HirExpression;

    let codegen = CodeGenerator::new();

    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("node".to_string())),
        field: "next".to_string(),
    };

    let code = codegen.generate_expression(&expr);
    // In Rust, ptr->field becomes ptr.field or (*ptr).field
    assert!(code.contains("node") && code.contains("next"));
}

#[test]
fn test_struct_with_box_field() {
    let codegen = CodeGenerator::new();

    let fields = vec![
        HirStructField::new("next".to_string(), HirType::Box(Box::new(HirType::Void))),
        HirStructField::new("value".to_string(), HirType::Int),
    ];

    let node_struct = HirStruct::new("Node".to_string(), fields);
    let code = codegen.generate_struct(&node_struct);

    assert!(code.contains("next: Box<()>"));
    assert!(code.contains("value: i32"));
}

#[test]
fn test_struct_with_reference_field() {
    let codegen = CodeGenerator::new();

    let fields = vec![HirStructField::new(
        "data".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
    )];

    let ref_struct = HirStruct::new("RefStruct".to_string(), fields);
    let code = codegen.generate_struct(&ref_struct);

    // Should have lifetime annotation
    assert!(code.contains("RefStruct<"));
    assert!(code.contains("data: &"));
}
