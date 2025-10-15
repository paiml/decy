//! Tests for nested struct transformation (TYPE-STRUCT-NESTED validation)
//!
//! Reference: K&R §6.2, ISO C99 §6.7.2.1
//!
//! This module tests the transformation of C nested structs to Rust structs.
//! Nested structs allow structs to contain other structs as fields, enabling
//! composition and complex data structures.

use decy_codegen::CodeGenerator;
use decy_hir::{
    HirExpression, HirFunction, HirParameter, HirStatement, HirStruct, HirStructField, HirType,
};

/// Test simple nested struct with separate definitions
///
/// C: struct Point { int x; int y; };
///    struct Rectangle {
///        struct Point top_left;
///        struct Point bottom_right;
///    };
///
/// Rust: struct Point { x: i32, y: i32 }
///       struct Rectangle {
///           top_left: Point,
///           bottom_right: Point
///       }
///
/// Reference: K&R §6.2, ISO C99 §6.7.2.1
#[test]
fn test_simple_nested_struct_definition() {
    let codegen = CodeGenerator::new();

    // Define Point struct
    let point_fields = vec![
        HirStructField::new("x".to_string(), HirType::Int),
        HirStructField::new("y".to_string(), HirType::Int),
    ];
    let point_struct = HirStruct::new("Point".to_string(), point_fields);

    // Define Rectangle struct with Point fields
    let rectangle_fields = vec![
        HirStructField::new("top_left".to_string(), HirType::Struct("Point".to_string())),
        HirStructField::new(
            "bottom_right".to_string(),
            HirType::Struct("Point".to_string()),
        ),
    ];
    let rectangle_struct = HirStruct::new("Rectangle".to_string(), rectangle_fields);

    let point_result = codegen.generate_struct(&point_struct);
    let rect_result = codegen.generate_struct(&rectangle_struct);

    // Verify Point struct is generated correctly
    assert!(
        point_result.contains("struct Point"),
        "Should generate Point struct"
    );
    assert!(point_result.contains("x: i32"), "Should have x: i32 field");
    assert!(point_result.contains("y: i32"), "Should have y: i32 field");

    // Verify Rectangle struct uses Point type (not "struct Point")
    assert!(
        rect_result.contains("struct Rectangle"),
        "Should generate Rectangle struct"
    );
    assert!(
        rect_result.contains("top_left: Point"),
        "Should use Point type without 'struct' keyword"
    );
    assert!(
        rect_result.contains("bottom_right: Point"),
        "Should use Point type for second field"
    );

    // Should NOT contain C-style "struct Point" in field types
    // (Note: may contain "struct Rectangle" for the struct definition itself)
    let field_section = rect_result.split("struct Rectangle").nth(1).unwrap_or("");
    assert!(
        !field_section.contains("struct Point"),
        "Should not use C-style 'struct Point' in Rust field types"
    );

    // Verify no unsafe blocks
    assert!(!point_result.contains("unsafe"));
    assert!(!rect_result.contains("unsafe"));
}

/// Test nested struct member access
///
/// C: struct Rectangle r;
///    int width = r.bottom_right.x - r.top_left.x;
///
/// Rust: let r: Rectangle;
///       let width = r.bottom_right.x - r.top_left.x;
///
/// Reference: K&R §6.2, ISO C99 §6.7.2.1
#[test]
fn test_nested_struct_member_access() {
    let func = HirFunction::new_with_body(
        "rect_width".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "r".to_string(),
            HirType::Struct("Rectangle".to_string()),
        )],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::Subtract,
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::FieldAccess {
                    object: Box::new(HirExpression::Variable("r".to_string())),
                    field: "bottom_right".to_string(),
                }),
                field: "x".to_string(),
            }),
            right: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::FieldAccess {
                    object: Box::new(HirExpression::Variable("r".to_string())),
                    field: "top_left".to_string(),
                }),
                field: "x".to_string(),
            }),
        }))],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    println!("Generated code:\n{}", result);

    // Verify nested member access
    assert!(
        result.contains("r.bottom_right.x"),
        "Should generate nested member access"
    );
    assert!(
        result.contains("r.top_left.x"),
        "Should generate second nested access"
    );

    // Should use dot notation (not arrow)
    // Note: Function signature may contain -> for return type, so check only the body
    let body_start = result.find('{').unwrap_or(0);
    let body = &result[body_start..];
    assert!(
        !body.contains("->"),
        "Should use dot notation, not arrow in function body"
    );

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test nested struct with pointer member access
///
/// C: struct Rectangle* r;
///    int width = r->bottom_right.x - r->top_left.x;
///
/// Rust: let r: &Rectangle;
///       let width = r.bottom_right.x - r.top_left.x;
///
/// Reference: K&R §6.2, ISO C99 §6.7.2.1
#[test]
fn test_nested_struct_pointer_member_access() {
    let func = HirFunction::new_with_body(
        "rect_width".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "r".to_string(),
            HirType::Pointer(Box::new(HirType::Struct("Rectangle".to_string()))),
        )],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::Subtract,
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::PointerFieldAccess {
                    pointer: Box::new(HirExpression::Variable("r".to_string())),
                    field: "bottom_right".to_string(),
                }),
                field: "x".to_string(),
            }),
            right: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::PointerFieldAccess {
                    pointer: Box::new(HirExpression::Variable("r".to_string())),
                    field: "top_left".to_string(),
                }),
                field: "x".to_string(),
            }),
        }))],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify nested access through pointer
    assert!(
        result.contains("bottom_right") && result.contains("x"),
        "Should access nested field through pointer"
    );
    assert!(
        result.contains("top_left"),
        "Should access second nested field"
    );

    // Verify no unsafe blocks (pointer access should be safe with references)
    assert!(!result.contains("unsafe"));
}

/// Test deeply nested structs (3 levels)
///
/// C: struct Point { int x; int y; };
///    struct Rectangle { struct Point top_left; struct Point bottom_right; };
///    struct Canvas { struct Rectangle bounds; int color; };
///
/// Rust: struct Point { x: i32, y: i32 }
///       struct Rectangle { top_left: Point, bottom_right: Point }
///       struct Canvas { bounds: Rectangle, color: i32 }
///
/// Reference: K&R §6.2, ISO C99 §6.7.2.1
#[test]
fn test_deeply_nested_structs() {
    let codegen = CodeGenerator::new();

    let canvas_fields = vec![
        HirStructField::new(
            "bounds".to_string(),
            HirType::Struct("Rectangle".to_string()),
        ),
        HirStructField::new("color".to_string(), HirType::Int),
    ];
    let canvas_struct = HirStruct::new("Canvas".to_string(), canvas_fields);

    let result = codegen.generate_struct(&canvas_struct);

    // Verify deeply nested struct definition
    assert!(
        result.contains("struct Canvas"),
        "Should generate Canvas struct"
    );
    assert!(
        result.contains("bounds: Rectangle"),
        "Should use Rectangle type"
    );
    assert!(result.contains("color: i32"), "Should have color field");

    // Should not use C-style "struct Rectangle"
    let field_section = result.split("struct Canvas").nth(1).unwrap_or("");
    assert!(
        !field_section.contains("struct Rectangle"),
        "Should not use 'struct' keyword in field type"
    );

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test deeply nested member access (3 levels)
///
/// C: struct Canvas c;
///    int x = c.bounds.top_left.x;
///
/// Rust: let c: Canvas;
///       let x = c.bounds.top_left.x;
///
/// Reference: K&R §6.2, ISO C99 §6.7.2.1
#[test]
fn test_deeply_nested_member_access() {
    let func = HirFunction::new_with_body(
        "get_canvas_x".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "c".to_string(),
            HirType::Struct("Canvas".to_string()),
        )],
        vec![HirStatement::Return(Some(HirExpression::FieldAccess {
            object: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::FieldAccess {
                    object: Box::new(HirExpression::Variable("c".to_string())),
                    field: "bounds".to_string(),
                }),
                field: "top_left".to_string(),
            }),
            field: "x".to_string(),
        }))],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify deeply nested member access
    assert!(
        result.contains("c.bounds.top_left.x"),
        "Should generate deeply nested member access"
    );

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test nested struct with mixed primitive and struct fields
///
/// C: struct Data {
///        int id;
///        struct Point location;
///        int size;
///    };
///
/// Rust: struct Data {
///         id: i32,
///         location: Point,
///         size: i32
///       }
///
/// Reference: K&R §6.2, ISO C99 §6.7.2.1
#[test]
fn test_nested_struct_with_mixed_fields() {
    let codegen = CodeGenerator::new();

    let data_fields = vec![
        HirStructField::new("id".to_string(), HirType::Int),
        HirStructField::new("location".to_string(), HirType::Struct("Point".to_string())),
        HirStructField::new("size".to_string(), HirType::Int),
    ];
    let data_struct = HirStruct::new("Data".to_string(), data_fields);

    let result = codegen.generate_struct(&data_struct);

    // Verify field ordering is preserved
    let id_pos = result.find("id").unwrap();
    let location_pos = result.find("location").unwrap();
    let size_pos = result.find("size").unwrap();

    assert!(
        id_pos < location_pos && location_pos < size_pos,
        "Should preserve field order"
    );

    // Verify correct types
    assert!(result.contains("id: i32"), "Should have id: i32");
    assert!(
        result.contains("location: Point"),
        "Should have location: Point"
    );
    assert!(result.contains("size: i32"), "Should have size: i32");

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test nested struct with array of structs
///
/// C: struct Polygon {
///        struct Point vertices[10];
///        int count;
///    };
///
/// Rust: struct Polygon {
///         vertices: [Point; 10],
///         count: i32
///       }
///
/// Reference: K&R §6.2, ISO C99 §6.7.2.1
#[test]
fn test_nested_struct_with_array() {
    let codegen = CodeGenerator::new();

    let polygon_fields = vec![
        HirStructField::new(
            "vertices".to_string(),
            HirType::Array {
                element_type: Box::new(HirType::Struct("Point".to_string())),
                size: Some(10),
            },
        ),
        HirStructField::new("count".to_string(), HirType::Int),
    ];
    let polygon_struct = HirStruct::new("Polygon".to_string(), polygon_fields);

    let result = codegen.generate_struct(&polygon_struct);

    // Verify array of struct type
    assert!(
        result.contains("vertices: [Point; 10]"),
        "Should have array of Point structs"
    );
    assert!(result.contains("count: i32"), "Should have count field");

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test nested struct with pointer to struct
///
/// C: struct Node {
///        int data;
///        struct Node* next;
///    };
///
/// Rust: struct Node {
///         data: i32,
///         next: Option<Box<Node>>
///       }
///
/// Reference: K&R §6.2, ISO C99 §6.7.2.1
#[test]
fn test_self_referential_struct() {
    let codegen = CodeGenerator::new();

    let node_fields = vec![
        HirStructField::new("data".to_string(), HirType::Int),
        HirStructField::new(
            "next".to_string(),
            HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        ),
    ];
    let node_struct = HirStruct::new("Node".to_string(), node_fields);

    let result = codegen.generate_struct(&node_struct);

    // Verify struct definition
    assert!(result.contains("struct Node"), "Should define Node struct");
    assert!(result.contains("data: i32"), "Should have data field");

    // Verify self-referential pointer (may be Box or raw pointer depending on implementation)
    assert!(result.contains("next:"), "Should have next field");

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Verify unsafe block count remains 0
///
/// This is critical for the validation goal: <5 unsafe blocks per 1000 LOC
#[test]
fn test_nested_struct_transformation_unsafe_count() {
    let codegen = CodeGenerator::new();

    let point_fields = vec![
        HirStructField::new("x".to_string(), HirType::Int),
        HirStructField::new("y".to_string(), HirType::Int),
    ];
    let point_struct = HirStruct::new("Point".to_string(), point_fields);

    let rectangle_fields = vec![
        HirStructField::new("top_left".to_string(), HirType::Struct("Point".to_string())),
        HirStructField::new(
            "bottom_right".to_string(),
            HirType::Struct("Point".to_string()),
        ),
    ];
    let rectangle_struct = HirStruct::new("Rectangle".to_string(), rectangle_fields);

    let point_result = codegen.generate_struct(&point_struct);
    let rect_result = codegen.generate_struct(&rectangle_struct);

    // Combine results
    let combined = format!("{}\n{}", point_result, rect_result);

    // Count unsafe blocks (should be 0)
    let unsafe_count = combined.matches("unsafe").count();
    assert_eq!(
        unsafe_count, 0,
        "Nested struct transformation should not introduce unsafe blocks"
    );
}
