//! Tests for C++ class code generation (DECY-202).

use super::*;
use decy_hir::HirClass;

#[test]
fn test_generate_class_simple_fields() {
    let codegen = CodeGenerator::new();

    let ast_class = decy_parser::parser::Class {
        name: "Point".to_string(),
        fields: vec![
            decy_parser::parser::StructField::new("x".to_string(), decy_parser::parser::Type::Int),
            decy_parser::parser::StructField::new("y".to_string(), decy_parser::parser::Type::Int),
        ],
        methods: vec![],
        constructor_params: vec![],
        has_destructor: false,
    };

    let hir_class = HirClass::from_ast_class(&ast_class);
    let code = codegen.generate_class(&hir_class);

    assert!(code.contains("pub struct Point"), "Should generate struct: {code}");
    assert!(code.contains("x: i32"), "Should have x field: {code}");
    assert!(code.contains("y: i32"), "Should have y field: {code}");
    assert!(code.contains("impl Point"), "Should generate impl block: {code}");
}

#[test]
fn test_generate_class_with_constructor() {
    let codegen = CodeGenerator::new();

    let ast_class = decy_parser::parser::Class {
        name: "Rect".to_string(),
        fields: vec![
            decy_parser::parser::StructField::new("w".to_string(), decy_parser::parser::Type::Int),
            decy_parser::parser::StructField::new("h".to_string(), decy_parser::parser::Type::Int),
        ],
        methods: vec![],
        constructor_params: vec![
            decy_parser::parser::Parameter::new("w".to_string(), decy_parser::parser::Type::Int),
            decy_parser::parser::Parameter::new("h".to_string(), decy_parser::parser::Type::Int),
        ],
        has_destructor: false,
    };

    let hir_class = HirClass::from_ast_class(&ast_class);
    let code = codegen.generate_class(&hir_class);

    assert!(code.contains("pub fn new(w: i32, h: i32) -> Self"), "Should have constructor: {code}");
    assert!(code.contains("w: w"), "Should map w param to w field: {code}");
    assert!(code.contains("h: h"), "Should map h param to h field: {code}");
}

#[test]
fn test_generate_class_with_destructor() {
    let codegen = CodeGenerator::new();

    let ast_class = decy_parser::parser::Class {
        name: "Resource".to_string(),
        fields: vec![decy_parser::parser::StructField::new(
            "handle".to_string(),
            decy_parser::parser::Type::Int,
        )],
        methods: vec![],
        constructor_params: vec![],
        has_destructor: true,
    };

    let hir_class = HirClass::from_ast_class(&ast_class);
    let code = codegen.generate_class(&hir_class);

    assert!(code.contains("impl Drop for Resource"), "Should have Drop impl: {code}");
    assert!(code.contains("fn drop(&mut self)"), "Should have drop method: {code}");
}

#[test]
fn test_generate_class_with_method() {
    let codegen = CodeGenerator::new();

    let method_func = decy_parser::parser::Function::new(
        "area".to_string(),
        decy_parser::parser::Type::Int,
        vec![],
    );

    let ast_class = decy_parser::parser::Class {
        name: "Square".to_string(),
        fields: vec![decy_parser::parser::StructField::new(
            "side".to_string(),
            decy_parser::parser::Type::Int,
        )],
        methods: vec![decy_parser::parser::Method {
            function: method_func,
            access: decy_parser::parser::AccessSpecifier::Public,
            is_const: true,
            is_static: false,
            is_virtual: false,
        }],
        constructor_params: vec![],
        has_destructor: false,
    };

    let hir_class = HirClass::from_ast_class(&ast_class);
    let code = codegen.generate_class(&hir_class);

    assert!(code.contains("pub fn area(&self") , "Const method should take &self: {code}");
    assert!(code.contains("-> i32"), "Should have return type: {code}");
}

#[test]
fn test_generate_class_no_drop_when_no_destructor() {
    let codegen = CodeGenerator::new();

    let ast_class = decy_parser::parser::Class {
        name: "Simple".to_string(),
        fields: vec![decy_parser::parser::StructField::new(
            "val".to_string(),
            decy_parser::parser::Type::Int,
        )],
        methods: vec![],
        constructor_params: vec![],
        has_destructor: false,
    };

    let hir_class = HirClass::from_ast_class(&ast_class);
    let code = codegen.generate_class(&hir_class);

    assert!(!code.contains("impl Drop"), "Should NOT have Drop impl without destructor: {code}");
}
