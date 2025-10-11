//! Tests for struct and enum support in HIR (DECY-020 RED phase).

use super::*;

#[test]
fn test_create_hir_struct() {
    let fields = vec![
        HirStructField::new("x".to_string(), HirType::Int),
        HirStructField::new("y".to_string(), HirType::Int),
    ];

    let hir_struct = HirStruct::new("Point".to_string(), fields);

    assert_eq!(hir_struct.name(), "Point");
    assert_eq!(hir_struct.fields().len(), 2);
    assert_eq!(hir_struct.fields()[0].name(), "x");
    assert_eq!(hir_struct.fields()[0].field_type(), &HirType::Int);
}

#[test]
fn test_create_hir_enum() {
    let variants = vec![
        HirEnumVariant::new("Red".to_string(), None),
        HirEnumVariant::new("Green".to_string(), None),
        HirEnumVariant::new("Blue".to_string(), None),
    ];

    let hir_enum = HirEnum::new("Color".to_string(), variants);

    assert_eq!(hir_enum.name(), "Color");
    assert_eq!(hir_enum.variants().len(), 3);
    assert_eq!(hir_enum.variants()[0].name(), "Red");
}

#[test]
fn test_struct_field_access_expression() {
    let expr = HirExpression::FieldAccess {
        object: Box::new(HirExpression::Variable("point".to_string())),
        field: "x".to_string(),
    };

    match expr {
        HirExpression::FieldAccess { object, field } => {
            assert_eq!(field, "x");
            match *object {
                HirExpression::Variable(name) => assert_eq!(name, "point"),
                _ => panic!("Expected Variable"),
            }
        }
        _ => panic!("Expected FieldAccess"),
    }
}

#[test]
fn test_pointer_field_access_expression() {
    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("ptr".to_string())),
        field: "data".to_string(),
    };

    match expr {
        HirExpression::PointerFieldAccess { pointer, field } => {
            assert_eq!(field, "data");
            match *pointer {
                HirExpression::Variable(name) => assert_eq!(name, "ptr"),
                _ => panic!("Expected Variable"),
            }
        }
        _ => panic!("Expected PointerFieldAccess"),
    }
}

#[test]
fn test_struct_with_pointer_fields() {
    let fields = vec![
        HirStructField::new(
            "next".to_string(),
            HirType::Pointer(Box::new(HirType::Void)),
        ),
        HirStructField::new("value".to_string(), HirType::Int),
    ];

    let node_struct = HirStruct::new("Node".to_string(), fields);

    assert_eq!(node_struct.fields()[0].name(), "next");
    assert!(matches!(
        node_struct.fields()[0].field_type(),
        HirType::Pointer(_)
    ));
}

#[test]
fn test_enum_with_integer_values() {
    let variants = vec![
        HirEnumVariant::new("Low".to_string(), Some(0)),
        HirEnumVariant::new("Medium".to_string(), Some(1)),
        HirEnumVariant::new("High".to_string(), Some(2)),
    ];

    let priority_enum = HirEnum::new("Priority".to_string(), variants);

    assert_eq!(priority_enum.variants()[0].value(), Some(0));
    assert_eq!(priority_enum.variants()[1].value(), Some(1));
}

#[test]
fn test_struct_type_variant() {
    let struct_type = HirType::Struct("Point".to_string());

    match struct_type {
        HirType::Struct(name) => assert_eq!(name, "Point"),
        _ => panic!("Expected Struct type"),
    }
}

#[test]
fn test_enum_type_variant() {
    let enum_type = HirType::Enum("Color".to_string());

    match enum_type {
        HirType::Enum(name) => assert_eq!(name, "Color"),
        _ => panic!("Expected Enum type"),
    }
}
