//! Comprehensive coverage tests for decy-codegen lib.rs
//!
//! Targets uncovered branches in:
//! - map_type (all HirType variants)
//! - generate_expression (all expression variants)
//! - generate_statement (all statement variants)
//! - generate_struct, generate_enum, generate_typedef, generate_constant
//! - generate_global_variable
//! - generate_return (all type variants)
//! - generate_macro (object-like and function-like)
//! - helper functions (escape_rust_keyword, default_value_for_type, etc.)

use decy_hir::{
    BinaryOperator, HirConstant, HirEnum, HirEnumVariant, HirExpression, HirFunction,
    HirMacroDefinition, HirParameter, HirStatement, HirStruct, HirStructField, HirType,
    HirTypedef, SwitchCase, UnaryOperator,
};

use crate::CodeGenerator;

// ============================================================================
// MAP_TYPE TESTS - All HirType variants
// ============================================================================

#[test]
fn map_type_void() {
    assert_eq!(CodeGenerator::map_type(&HirType::Void), "()");
}

#[test]
fn map_type_int() {
    assert_eq!(CodeGenerator::map_type(&HirType::Int), "i32");
}

#[test]
fn map_type_unsigned_int() {
    assert_eq!(CodeGenerator::map_type(&HirType::UnsignedInt), "u32");
}

#[test]
fn map_type_float() {
    assert_eq!(CodeGenerator::map_type(&HirType::Float), "f32");
}

#[test]
fn map_type_double() {
    assert_eq!(CodeGenerator::map_type(&HirType::Double), "f64");
}

#[test]
fn map_type_char() {
    assert_eq!(CodeGenerator::map_type(&HirType::Char), "u8");
}

#[test]
fn map_type_signed_char() {
    assert_eq!(CodeGenerator::map_type(&HirType::SignedChar), "i8");
}

#[test]
fn map_type_pointer_int() {
    assert_eq!(
        CodeGenerator::map_type(&HirType::Pointer(Box::new(HirType::Int))),
        "*mut i32"
    );
}

#[test]
fn map_type_box_int() {
    assert_eq!(
        CodeGenerator::map_type(&HirType::Box(Box::new(HirType::Int))),
        "Box<i32>"
    );
}

#[test]
fn map_type_vec_int() {
    assert_eq!(
        CodeGenerator::map_type(&HirType::Vec(Box::new(HirType::Int))),
        "Vec<i32>"
    );
}

#[test]
fn map_type_option_int() {
    assert_eq!(
        CodeGenerator::map_type(&HirType::Option(Box::new(HirType::Int))),
        "Option<i32>"
    );
}

#[test]
fn map_type_reference_immutable() {
    assert_eq!(
        CodeGenerator::map_type(&HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        }),
        "&i32"
    );
}

#[test]
fn map_type_reference_mutable() {
    assert_eq!(
        CodeGenerator::map_type(&HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        }),
        "&mut i32"
    );
}

#[test]
fn map_type_reference_to_vec_immutable_slice() {
    // &Vec<T> -> &[T]
    assert_eq!(
        CodeGenerator::map_type(&HirType::Reference {
            inner: Box::new(HirType::Vec(Box::new(HirType::Int))),
            mutable: false,
        }),
        "&[i32]"
    );
}

#[test]
fn map_type_reference_to_vec_mutable_slice() {
    // &mut Vec<T> -> &mut [T]
    assert_eq!(
        CodeGenerator::map_type(&HirType::Reference {
            inner: Box::new(HirType::Vec(Box::new(HirType::Int))),
            mutable: true,
        }),
        "&mut [i32]"
    );
}

#[test]
fn map_type_struct() {
    assert_eq!(
        CodeGenerator::map_type(&HirType::Struct("MyStruct".to_string())),
        "MyStruct"
    );
}

#[test]
fn map_type_enum() {
    assert_eq!(
        CodeGenerator::map_type(&HirType::Enum("MyEnum".to_string())),
        "MyEnum"
    );
}

#[test]
fn map_type_array_sized() {
    assert_eq!(
        CodeGenerator::map_type(&HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        }),
        "[i32; 10]"
    );
}

#[test]
fn map_type_array_unsized() {
    assert_eq!(
        CodeGenerator::map_type(&HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        }),
        "[i32]"
    );
}

#[test]
fn map_type_function_pointer() {
    assert_eq!(
        CodeGenerator::map_type(&HirType::FunctionPointer {
            param_types: vec![HirType::Int, HirType::Int],
            return_type: Box::new(HirType::Int),
        }),
        "fn(i32, i32) -> i32"
    );
}

#[test]
fn map_type_function_pointer_void_return() {
    assert_eq!(
        CodeGenerator::map_type(&HirType::FunctionPointer {
            param_types: vec![HirType::Int],
            return_type: Box::new(HirType::Void),
        }),
        "fn(i32)"
    );
}

#[test]
fn map_type_string_literal() {
    assert_eq!(CodeGenerator::map_type(&HirType::StringLiteral), "&str");
}

#[test]
fn map_type_owned_string() {
    assert_eq!(CodeGenerator::map_type(&HirType::OwnedString), "String");
}

#[test]
fn map_type_string_reference() {
    assert_eq!(CodeGenerator::map_type(&HirType::StringReference), "&str");
}

#[test]
fn map_type_union() {
    assert_eq!(
        CodeGenerator::map_type(&HirType::Union(vec![])),
        "/* Union type */"
    );
}

#[test]
fn map_type_type_alias() {
    assert_eq!(
        CodeGenerator::map_type(&HirType::TypeAlias("size_t".to_string())),
        "size_t"
    );
}

// ============================================================================
// GENERATE_EXPRESSION TESTS - Various expression types
// ============================================================================

#[test]
fn expr_int_literal() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.generate_expression(&HirExpression::IntLiteral(42)), "42");
}

#[test]
fn expr_float_literal_with_dot() {
    let gen = CodeGenerator::new();
    assert_eq!(
        gen.generate_expression(&HirExpression::FloatLiteral("3.14".to_string())),
        "3.14f64"
    );
}

#[test]
fn expr_float_literal_without_dot() {
    let gen = CodeGenerator::new();
    assert_eq!(
        gen.generate_expression(&HirExpression::FloatLiteral("42".to_string())),
        "42.0f64"
    );
}

#[test]
fn expr_float_literal_with_c_suffix() {
    let gen = CodeGenerator::new();
    // C uses 'f' suffix for float
    assert_eq!(
        gen.generate_expression(&HirExpression::FloatLiteral("3.14f".to_string())),
        "3.14f64"
    );
}

#[test]
fn expr_string_literal() {
    let gen = CodeGenerator::new();
    assert_eq!(
        gen.generate_expression(&HirExpression::StringLiteral("hello".to_string())),
        "\"hello\""
    );
}

#[test]
fn expr_char_literal_printable() {
    let gen = CodeGenerator::new();
    assert_eq!(
        gen.generate_expression(&HirExpression::CharLiteral(b'a' as i8)),
        "b'a'"
    );
}

#[test]
fn expr_char_literal_null() {
    let gen = CodeGenerator::new();
    assert_eq!(
        gen.generate_expression(&HirExpression::CharLiteral(0)),
        "0u8"
    );
}

#[test]
fn expr_char_literal_non_printable() {
    let gen = CodeGenerator::new();
    assert_eq!(
        gen.generate_expression(&HirExpression::CharLiteral(1)),
        "1u8"
    );
}

#[test]
fn expr_variable() {
    let gen = CodeGenerator::new();
    assert_eq!(
        gen.generate_expression(&HirExpression::Variable("x".to_string())),
        "x"
    );
}

#[test]
fn expr_variable_stderr() {
    let gen = CodeGenerator::new();
    assert_eq!(
        gen.generate_expression(&HirExpression::Variable("stderr".to_string())),
        "std::io::stderr()"
    );
}

#[test]
fn expr_variable_stdin() {
    let gen = CodeGenerator::new();
    assert_eq!(
        gen.generate_expression(&HirExpression::Variable("stdin".to_string())),
        "std::io::stdin()"
    );
}

#[test]
fn expr_variable_stdout() {
    let gen = CodeGenerator::new();
    assert_eq!(
        gen.generate_expression(&HirExpression::Variable("stdout".to_string())),
        "std::io::stdout()"
    );
}

#[test]
fn expr_variable_errno() {
    let gen = CodeGenerator::new();
    assert_eq!(
        gen.generate_expression(&HirExpression::Variable("errno".to_string())),
        "unsafe { ERRNO }"
    );
}

#[test]
fn expr_variable_erange() {
    let gen = CodeGenerator::new();
    assert_eq!(
        gen.generate_expression(&HirExpression::Variable("ERANGE".to_string())),
        "34i32"
    );
}

#[test]
fn expr_variable_einval() {
    let gen = CodeGenerator::new();
    assert_eq!(
        gen.generate_expression(&HirExpression::Variable("EINVAL".to_string())),
        "22i32"
    );
}

#[test]
fn expr_variable_enoent() {
    let gen = CodeGenerator::new();
    assert_eq!(
        gen.generate_expression(&HirExpression::Variable("ENOENT".to_string())),
        "2i32"
    );
}

#[test]
fn expr_variable_eacces() {
    let gen = CodeGenerator::new();
    assert_eq!(
        gen.generate_expression(&HirExpression::Variable("EACCES".to_string())),
        "13i32"
    );
}

#[test]
fn expr_null_literal() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.generate_expression(&HirExpression::NullLiteral), "None");
}

#[test]
fn expr_is_not_null() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::IsNotNull(Box::new(
        HirExpression::Variable("p".to_string()),
    )));
    assert_eq!(result, "if let Some(_) = p");
}

#[test]
fn expr_binary_add() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::IntLiteral(1)),
        right: Box::new(HirExpression::IntLiteral(2)),
    });
    assert_eq!(result, "1 + 2");
}

#[test]
fn expr_binary_comma() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::BinaryOp {
        op: BinaryOperator::Comma,
        left: Box::new(HirExpression::IntLiteral(1)),
        right: Box::new(HirExpression::IntLiteral(2)),
    });
    assert_eq!(result, "{ 1; 2 }");
}

#[test]
fn expr_binary_assign_embedded() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::BinaryOp {
        op: BinaryOperator::Assign,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::IntLiteral(42)),
    });
    assert!(result.contains("__assign_tmp"));
    assert!(result.contains("42"));
}

#[test]
fn expr_nested_binary_ops_parenthesized() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::IntLiteral(2)),
            right: Box::new(HirExpression::IntLiteral(3)),
        }),
        right: Box::new(HirExpression::IntLiteral(4)),
    });
    assert!(result.contains("(2 * 3)"));
    assert!(result.contains("+ 4"));
}

#[test]
fn expr_dereference() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::Dereference(Box::new(
        HirExpression::Variable("x".to_string()),
    )));
    assert_eq!(result, "*x");
}

#[test]
fn expr_address_of() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::AddressOf(Box::new(
        HirExpression::Variable("x".to_string()),
    )));
    assert_eq!(result, "&x");
}

#[test]
fn expr_address_of_dereference() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::AddressOf(Box::new(
        HirExpression::Dereference(Box::new(HirExpression::Variable("x".to_string()))),
    )));
    assert_eq!(result, "&(*x)");
}

#[test]
fn expr_unary_minus() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::UnaryOp {
        op: UnaryOperator::Minus,
        operand: Box::new(HirExpression::IntLiteral(5)),
    });
    assert_eq!(result, "-5");
}

#[test]
fn expr_unary_bitwise_not() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::UnaryOp {
        op: UnaryOperator::BitwiseNot,
        operand: Box::new(HirExpression::IntLiteral(5)),
    });
    assert_eq!(result, "!5");
}

#[test]
fn expr_unary_logical_not_on_int() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    });
    assert!(result.contains("== 0"));
}

#[test]
fn expr_unary_logical_not_on_boolean() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::IntLiteral(1)),
            right: Box::new(HirExpression::IntLiteral(2)),
        }),
    });
    assert!(result.starts_with("!"));
}

#[test]
fn expr_post_increment() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Variable("x".to_string())),
    });
    assert!(result.contains("__tmp"));
    assert!(result.contains("x += 1"));
}

#[test]
fn expr_pre_increment() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::PreIncrement {
        operand: Box::new(HirExpression::Variable("x".to_string())),
    });
    assert!(result.contains("x += 1"));
}

#[test]
fn expr_post_decrement() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::PostDecrement {
        operand: Box::new(HirExpression::Variable("x".to_string())),
    });
    assert!(result.contains("__tmp"));
    assert!(result.contains("x -= 1"));
}

#[test]
fn expr_pre_decrement() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::PreDecrement {
        operand: Box::new(HirExpression::Variable("x".to_string())),
    });
    assert!(result.contains("x -= 1"));
}

#[test]
fn expr_field_access() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FieldAccess {
        object: Box::new(HirExpression::Variable("obj".to_string())),
        field: "name".to_string(),
    });
    assert_eq!(result, "obj.name");
}

#[test]
fn expr_pointer_field_access() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("ptr".to_string())),
        field: "name".to_string(),
    });
    assert!(result.contains("ptr"));
    assert!(result.contains("name"));
}

#[test]
fn expr_pointer_field_access_chained() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::PointerFieldAccess {
            pointer: Box::new(HirExpression::Variable("ptr".to_string())),
            field: "next".to_string(),
        }),
        field: "data".to_string(),
    });
    // Chained -> should use dot notation
    assert!(result.contains(".data"));
}

#[test]
fn expr_array_index() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(5)),
    });
    assert!(result.contains("arr"));
    assert!(result.contains("5"));
    assert!(result.contains("as usize"));
}

#[test]
fn expr_slice_index() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::SliceIndex {
        slice: Box::new(HirExpression::Variable("data".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
        element_type: HirType::Int,
    });
    assert!(result.contains("data"));
    assert!(result.contains("i"));
    assert!(result.contains("as usize"));
}

#[test]
fn expr_sizeof_int() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::Sizeof {
        type_name: "int".to_string(),
    });
    assert!(result.contains("std::mem::size_of::<i32>()"));
    assert!(result.contains("as i32"));
}

#[test]
fn expr_sizeof_struct() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::Sizeof {
        type_name: "struct Data".to_string(),
    });
    assert!(result.contains("Data"));
    assert!(result.contains("size_of"));
}

#[test]
fn expr_calloc() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(10)),
        element_type: Box::new(HirType::Int),
    });
    assert!(result.contains("vec!"));
    assert!(result.contains("0i32"));
    assert!(result.contains("10"));
}

#[test]
fn expr_calloc_unsigned_int() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(5)),
        element_type: Box::new(HirType::UnsignedInt),
    });
    assert!(result.contains("0u32"));
}

#[test]
fn expr_calloc_float() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(5)),
        element_type: Box::new(HirType::Float),
    });
    assert!(result.contains("0.0f32"));
}

#[test]
fn expr_calloc_double() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(5)),
        element_type: Box::new(HirType::Double),
    });
    assert!(result.contains("0.0f64"));
}

#[test]
fn expr_calloc_char() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(5)),
        element_type: Box::new(HirType::Char),
    });
    assert!(result.contains("0u8"));
}

#[test]
fn expr_calloc_signed_char() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(5)),
        element_type: Box::new(HirType::SignedChar),
    });
    assert!(result.contains("0i8"));
}

#[test]
fn expr_malloc_single() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::Malloc {
        size: Box::new(HirExpression::IntLiteral(4)),
    });
    assert!(result.contains("Box::new(0i32)"));
}

#[test]
fn expr_malloc_array_pattern() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::Malloc {
        size: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::IntLiteral(4)),
        }),
    });
    assert!(result.contains("Vec::with_capacity"));
}

#[test]
fn expr_realloc_null_pointer() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::Realloc {
        pointer: Box::new(HirExpression::NullLiteral),
        new_size: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::IntLiteral(10)),
            right: Box::new(HirExpression::IntLiteral(4)),
        }),
    });
    assert!(result.contains("vec!"));
}

#[test]
fn expr_realloc_null_simple_size() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::Realloc {
        pointer: Box::new(HirExpression::NullLiteral),
        new_size: Box::new(HirExpression::IntLiteral(100)),
    });
    assert_eq!(result, "Vec::new()");
}

#[test]
fn expr_realloc_existing_pointer() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::Realloc {
        pointer: Box::new(HirExpression::Variable("ptr".to_string())),
        new_size: Box::new(HirExpression::IntLiteral(200)),
    });
    assert_eq!(result, "ptr");
}

#[test]
fn expr_string_method_call_len() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "len".to_string(),
        arguments: vec![],
    });
    assert_eq!(result, "s.len() as i32");
}

#[test]
fn expr_string_method_call_no_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "is_empty".to_string(),
        arguments: vec![],
    });
    assert_eq!(result, "s.is_empty()");
}

#[test]
fn expr_string_method_call_with_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "push".to_string(),
        arguments: vec![HirExpression::IntLiteral(42)],
    });
    assert_eq!(result, "s.push(42)");
}

#[test]
fn expr_string_method_call_clone_into() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("src".to_string())),
        method: "clone_into".to_string(),
        arguments: vec![HirExpression::Variable("dst".to_string())],
    });
    assert!(result.contains("&mut dst"));
}

#[test]
fn expr_cast_int() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::Cast {
        target_type: HirType::Int,
        expr: Box::new(HirExpression::Variable("x".to_string())),
    });
    assert_eq!(result, "x as i32");
}

#[test]
fn expr_cast_binary_op_parenthesized() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::Cast {
        target_type: HirType::Float,
        expr: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::IntLiteral(1)),
            right: Box::new(HirExpression::IntLiteral(2)),
        }),
    });
    assert!(result.contains("(1 + 2) as f32"));
}

#[test]
fn expr_compound_literal_struct_empty() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Point".to_string()),
        initializers: vec![],
    });
    assert_eq!(result, "Point {}");
}

#[test]
fn expr_compound_literal_struct_with_values() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Point".to_string()),
        initializers: vec![HirExpression::IntLiteral(10), HirExpression::IntLiteral(20)],
    });
    assert!(result.contains("Point"));
    assert!(result.contains("10"));
    assert!(result.contains("20"));
}

#[test]
fn expr_compound_literal_array_empty_sized() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(5),
        },
        initializers: vec![],
    });
    assert!(result.contains("[0i32; 5]"));
}

#[test]
fn expr_compound_literal_array_empty_unsized() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        },
        initializers: vec![],
    });
    assert_eq!(result, "[]");
}

#[test]
fn expr_compound_literal_array_single_init() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
        initializers: vec![HirExpression::IntLiteral(0)],
    });
    assert!(result.contains("[0; 10]"));
}

#[test]
fn expr_compound_literal_array_with_values() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        },
        initializers: vec![HirExpression::IntLiteral(1), HirExpression::IntLiteral(2)],
    });
    assert!(result.contains("[1, 2]"));
}

#[test]
fn expr_compound_literal_other_type() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::CompoundLiteral {
        literal_type: HirType::Int,
        initializers: vec![HirExpression::IntLiteral(42)],
    });
    assert!(result.contains("Compound literal"));
}

#[test]
fn expr_ternary() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::Ternary {
        condition: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
        then_expr: Box::new(HirExpression::Variable("a".to_string())),
        else_expr: Box::new(HirExpression::Variable("b".to_string())),
    });
    assert!(result.contains("if"));
    assert!(result.contains("else"));
}

#[test]
fn expr_ternary_non_boolean_condition() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::Ternary {
        condition: Box::new(HirExpression::Variable("flag".to_string())),
        then_expr: Box::new(HirExpression::IntLiteral(1)),
        else_expr: Box::new(HirExpression::IntLiteral(0)),
    });
    assert!(result.contains("!= 0"));
}

// ============================================================================
// FUNCTION CALL EXPRESSIONS - Standard library functions
// ============================================================================

#[test]
fn expr_strlen() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "strlen".to_string(),
        arguments: vec![HirExpression::Variable("s".to_string())],
    });
    assert!(result.contains("s.len() as i32"));
}

#[test]
fn expr_strcpy() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "strcpy".to_string(),
        arguments: vec![
            HirExpression::Variable("dest".to_string()),
            HirExpression::Variable("src".to_string()),
        ],
    });
    assert!(result.contains("to_string()"));
}

#[test]
fn expr_free() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "free".to_string(),
        arguments: vec![HirExpression::Variable("ptr".to_string())],
    });
    assert_eq!(result, "drop(ptr)");
}

#[test]
fn expr_free_no_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "free".to_string(),
        arguments: vec![],
    });
    assert_eq!(result, "/* free() */");
}

#[test]
fn expr_fopen_read() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "fopen".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("test.txt".to_string()),
            HirExpression::StringLiteral("r".to_string()),
        ],
    });
    assert!(result.contains("File::open"));
}

#[test]
fn expr_fopen_write() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "fopen".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("test.txt".to_string()),
            HirExpression::StringLiteral("w".to_string()),
        ],
    });
    assert!(result.contains("File::create"));
}

#[test]
fn expr_fclose() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "fclose".to_string(),
        arguments: vec![HirExpression::Variable("f".to_string())],
    });
    assert_eq!(result, "drop(f)");
}

#[test]
fn expr_fgetc() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "fgetc".to_string(),
        arguments: vec![HirExpression::Variable("f".to_string())],
    });
    assert!(result.contains("Read"));
    assert!(result.contains("buf"));
}

#[test]
fn expr_fputc() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "fputc".to_string(),
        arguments: vec![
            HirExpression::Variable("c".to_string()),
            HirExpression::Variable("f".to_string()),
        ],
    });
    assert!(result.contains("Write"));
    assert!(result.contains("write"));
}

#[test]
fn expr_printf_simple() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![HirExpression::StringLiteral("hello\\n".to_string())],
    });
    assert!(result.contains("print!"));
}

#[test]
fn expr_printf_empty() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![],
    });
    assert_eq!(result, "print!(\"\")");
}

#[test]
fn expr_fork() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "fork".to_string(),
        arguments: vec![],
    });
    assert!(result.contains("fork"));
    assert!(result.contains("0"));
}

#[test]
fn expr_wait() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "wait".to_string(),
        arguments: vec![HirExpression::Variable("status".to_string())],
    });
    assert!(result.contains("wait"));
}

#[test]
fn expr_wexitstatus() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "WEXITSTATUS".to_string(),
        arguments: vec![HirExpression::Variable("status".to_string())],
    });
    assert!(result.contains("code()"));
}

#[test]
fn expr_wifexited() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "WIFEXITED".to_string(),
        arguments: vec![HirExpression::Variable("status".to_string())],
    });
    assert!(result.contains("success()"));
}

#[test]
fn expr_wifsignaled() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "WIFSIGNALED".to_string(),
        arguments: vec![HirExpression::Variable("status".to_string())],
    });
    assert!(result.contains("signal()"));
}

#[test]
fn expr_wtermsig() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "WTERMSIG".to_string(),
        arguments: vec![HirExpression::Variable("status".to_string())],
    });
    assert!(result.contains("signal()"));
}

#[test]
fn expr_fread() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "fread".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(100),
            HirExpression::Variable("f".to_string()),
        ],
    });
    assert!(result.contains("Read"));
}

#[test]
fn expr_fwrite() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "fwrite".to_string(),
        arguments: vec![
            HirExpression::Variable("data".to_string()),
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(100),
            HirExpression::Variable("f".to_string()),
        ],
    });
    assert!(result.contains("Write"));
}

#[test]
fn expr_fputs() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "fputs".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("hello".to_string()),
            HirExpression::Variable("f".to_string()),
        ],
    });
    assert!(result.contains("write_all"));
}

#[test]
fn expr_function_call_write_renamed() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "write".to_string(),
        arguments: vec![HirExpression::IntLiteral(1)],
    });
    assert!(result.contains("c_write"));
}

#[test]
fn expr_function_call_read_renamed() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "read".to_string(),
        arguments: vec![HirExpression::IntLiteral(1)],
    });
    assert!(result.contains("c_read"));
}

#[test]
fn expr_calloc_function_call() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "calloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(10), HirExpression::IntLiteral(4)],
    });
    assert!(result.contains("vec!"));
}

#[test]
fn expr_calloc_no_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "calloc".to_string(),
        arguments: vec![],
    });
    assert_eq!(result, "Vec::new()");
}

#[test]
fn expr_malloc_no_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![],
    });
    assert_eq!(result, "Vec::new()");
}

#[test]
fn expr_realloc_function_call() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "realloc".to_string(),
        arguments: vec![
            HirExpression::Variable("ptr".to_string()),
            HirExpression::IntLiteral(200),
        ],
    });
    assert!(result.contains("realloc"));
}

#[test]
fn expr_realloc_no_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "realloc".to_string(),
        arguments: vec![],
    });
    assert!(result.contains("null_mut"));
}

// ============================================================================
// GENERATE_STATEMENT TESTS
// ============================================================================

#[test]
fn stmt_variable_declaration_int() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(42)),
    });
    assert!(result.contains("let mut x: i32 = 42;"));
}

#[test]
fn stmt_variable_declaration_no_init() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Int,
        initializer: None,
    });
    assert!(result.contains("let mut x: i32 = 0i32;"));
}

#[test]
fn stmt_variable_declaration_string_literal() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::VariableDeclaration {
        name: "s".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Char)),
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    });
    assert!(result.contains("&str"));
    assert!(result.contains("\"hello\""));
}

#[test]
fn stmt_return_some() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::Return(Some(HirExpression::IntLiteral(0))));
    assert_eq!(result, "return 0;");
}

#[test]
fn stmt_return_none() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::Return(None));
    assert_eq!(result, "return;");
}

#[test]
fn stmt_break() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.generate_statement(&HirStatement::Break), "break;");
}

#[test]
fn stmt_continue() {
    let gen = CodeGenerator::new();
    assert_eq!(
        gen.generate_statement(&HirStatement::Continue),
        "continue;"
    );
}

#[test]
fn stmt_if_simple() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
        else_block: None,
    });
    assert!(result.contains("if x > 0 {"));
    assert!(result.contains("return 1;"));
}

#[test]
fn stmt_if_else() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
        else_block: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
            1,
        )))]),
    });
    assert!(result.contains("if"));
    assert!(result.contains("} else {"));
}

#[test]
fn stmt_if_non_boolean_condition() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::If {
        condition: HirExpression::Variable("flag".to_string()),
        then_block: vec![HirStatement::Break],
        else_block: None,
    });
    assert!(result.contains("!= 0"));
}

#[test]
fn stmt_while() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::While {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        },
        body: vec![HirStatement::Expression(HirExpression::PostIncrement {
            operand: Box::new(HirExpression::Variable("i".to_string())),
        })],
    });
    assert!(result.contains("while i < 10 {"));
}

#[test]
fn stmt_for_loop() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::For {
        init: vec![HirStatement::VariableDeclaration {
            name: "i".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        }],
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        },
        increment: vec![HirStatement::Assignment {
            target: "i".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
        body: vec![HirStatement::Break],
    });
    assert!(result.contains("let mut i: i32 = 0;"));
    assert!(result.contains("while i < 10 {"));
}

#[test]
fn stmt_switch() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![
            SwitchCase {
                value: Some(HirExpression::IntLiteral(1)),
                body: vec![
                    HirStatement::Expression(HirExpression::FunctionCall {
                        function: "printf".to_string(),
                        arguments: vec![HirExpression::StringLiteral("one".to_string())],
                    }),
                    HirStatement::Break,
                ],
            },
            SwitchCase {
                value: Some(HirExpression::IntLiteral(2)),
                body: vec![
                    HirStatement::Expression(HirExpression::FunctionCall {
                        function: "printf".to_string(),
                        arguments: vec![HirExpression::StringLiteral("two".to_string())],
                    }),
                    HirStatement::Break,
                ],
            },
        ],
        default_case: Some(vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "printf".to_string(),
            arguments: vec![HirExpression::StringLiteral("other".to_string())],
        })]),
    });
    assert!(result.contains("match x {"));
    assert!(result.contains("1 =>"));
    assert!(result.contains("2 =>"));
    assert!(result.contains("_ =>"));
}

#[test]
fn stmt_assignment() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::Assignment {
        target: "x".to_string(),
        value: HirExpression::IntLiteral(42),
    });
    assert_eq!(result, "x = 42;");
}

#[test]
fn stmt_deref_assignment() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::DerefAssignment {
        target: HirExpression::Variable("x".to_string()),
        value: HirExpression::IntLiteral(42),
    });
    assert!(result.contains("*x = 42;"));
}

#[test]
fn stmt_deref_assignment_pointer_field_access() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::DerefAssignment {
        target: HirExpression::PointerFieldAccess {
            pointer: Box::new(HirExpression::Variable("ptr".to_string())),
            field: "data".to_string(),
        },
        value: HirExpression::IntLiteral(42),
    });
    // PointerFieldAccess in DerefAssignment doesn't need extra dereference
    assert!(result.contains("= 42;"));
    assert!(!result.contains("**"));
}

#[test]
fn stmt_deref_assignment_field_access() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::DerefAssignment {
        target: HirExpression::FieldAccess {
            object: Box::new(HirExpression::Variable("obj".to_string())),
            field: "data".to_string(),
        },
        value: HirExpression::IntLiteral(42),
    });
    assert!(result.contains("obj.data = 42;"));
}

#[test]
fn stmt_deref_assignment_array_index() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::DerefAssignment {
        target: HirExpression::ArrayIndex {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
        },
        value: HirExpression::IntLiteral(42),
    });
    assert!(result.contains("arr"));
    assert!(result.contains("= 42;"));
}

#[test]
fn stmt_array_index_assignment() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(5)),
        value: HirExpression::IntLiteral(99),
    });
    assert!(result.contains("arr"));
    assert!(result.contains("as usize"));
    assert!(result.contains("99"));
}

#[test]
fn stmt_field_assignment() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::FieldAssignment {
        object: HirExpression::Variable("obj".to_string()),
        field: "name".to_string(),
        value: HirExpression::IntLiteral(42),
    });
    assert!(result.contains("obj.name = 42;"));
}

#[test]
fn stmt_free() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::Free {
        pointer: HirExpression::Variable("ptr".to_string()),
    });
    assert!(result.contains("RAII"));
    assert!(result.contains("ptr"));
}

#[test]
fn stmt_expression() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::Expression(HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![HirExpression::StringLiteral("hi".to_string())],
    }));
    assert!(result.contains("print!"));
}

#[test]
fn stmt_inline_asm() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::InlineAsm {
        text: "mov eax, 0".to_string(),
        translatable: false,
    });
    assert!(result.contains("manual review"));
    assert!(result.contains("mov eax, 0"));
}

#[test]
fn stmt_inline_asm_translatable() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::InlineAsm {
        text: "nop".to_string(),
        translatable: true,
    });
    assert!(result.contains("translatable"));
}

#[test]
fn stmt_assignment_realloc_zero_size() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::Assignment {
        target: "buf".to_string(),
        value: HirExpression::Realloc {
            pointer: Box::new(HirExpression::Variable("buf".to_string())),
            new_size: Box::new(HirExpression::IntLiteral(0)),
        },
    });
    assert!(result.contains("clear"));
}

#[test]
fn stmt_assignment_realloc_with_size() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::Assignment {
        target: "buf".to_string(),
        value: HirExpression::Realloc {
            pointer: Box::new(HirExpression::Variable("buf".to_string())),
            new_size: Box::new(HirExpression::IntLiteral(200)),
        },
    });
    assert!(result.contains("resize"));
}

#[test]
fn stmt_assignment_errno() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::Assignment {
        target: "errno".to_string(),
        value: HirExpression::IntLiteral(22),
    });
    assert!(result.contains("unsafe"));
    assert!(result.contains("ERRNO"));
}

// ============================================================================
// GENERATE_RETURN TESTS
// ============================================================================

#[test]
fn return_void() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.generate_return(&HirType::Void), "");
}

#[test]
fn return_int() {
    let gen = CodeGenerator::new();
    assert!(gen.generate_return(&HirType::Int).contains("return 0"));
}

#[test]
fn return_unsigned_int() {
    let gen = CodeGenerator::new();
    assert!(gen
        .generate_return(&HirType::UnsignedInt)
        .contains("return 0"));
}

#[test]
fn return_float() {
    let gen = CodeGenerator::new();
    assert!(gen.generate_return(&HirType::Float).contains("return 0.0"));
}

#[test]
fn return_double() {
    let gen = CodeGenerator::new();
    assert!(gen
        .generate_return(&HirType::Double)
        .contains("return 0.0"));
}

#[test]
fn return_char() {
    let gen = CodeGenerator::new();
    assert!(gen.generate_return(&HirType::Char).contains("return 0"));
}

#[test]
fn return_signed_char() {
    let gen = CodeGenerator::new();
    assert!(gen
        .generate_return(&HirType::SignedChar)
        .contains("return 0"));
}

#[test]
fn return_pointer() {
    let gen = CodeGenerator::new();
    assert!(gen
        .generate_return(&HirType::Pointer(Box::new(HirType::Int)))
        .contains("null_mut"));
}

#[test]
fn return_box() {
    let gen = CodeGenerator::new();
    assert!(gen
        .generate_return(&HirType::Box(Box::new(HirType::Int)))
        .contains("Box::new"));
}

#[test]
fn return_vec() {
    let gen = CodeGenerator::new();
    assert!(gen
        .generate_return(&HirType::Vec(Box::new(HirType::Int)))
        .contains("Vec::new"));
}

#[test]
fn return_option() {
    let gen = CodeGenerator::new();
    assert!(gen
        .generate_return(&HirType::Option(Box::new(HirType::Int)))
        .contains("None"));
}

#[test]
fn return_reference() {
    let gen = CodeGenerator::new();
    assert_eq!(
        gen.generate_return(&HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        }),
        ""
    );
}

#[test]
fn return_struct() {
    let gen = CodeGenerator::new();
    assert!(gen
        .generate_return(&HirType::Struct("MyStruct".to_string()))
        .contains("MyStruct::default()"));
}

#[test]
fn return_enum() {
    let gen = CodeGenerator::new();
    assert!(gen
        .generate_return(&HirType::Enum("MyEnum".to_string()))
        .contains("MyEnum::default()"));
}

#[test]
fn return_array_sized() {
    let gen = CodeGenerator::new();
    let result = gen.generate_return(&HirType::Array {
        element_type: Box::new(HirType::Int),
        size: Some(3),
    });
    assert!(result.contains("[0i32; 3]"));
}

#[test]
fn return_array_unsized() {
    let gen = CodeGenerator::new();
    assert_eq!(
        gen.generate_return(&HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        }),
        ""
    );
}

#[test]
fn return_function_pointer() {
    let gen = CodeGenerator::new();
    assert_eq!(
        gen.generate_return(&HirType::FunctionPointer {
            param_types: vec![],
            return_type: Box::new(HirType::Void),
        }),
        ""
    );
}

#[test]
fn return_string_literal() {
    let gen = CodeGenerator::new();
    assert!(gen
        .generate_return(&HirType::StringLiteral)
        .contains("\"\""));
}

#[test]
fn return_owned_string() {
    let gen = CodeGenerator::new();
    assert!(gen
        .generate_return(&HirType::OwnedString)
        .contains("String::new"));
}

#[test]
fn return_string_reference() {
    let gen = CodeGenerator::new();
    assert!(gen
        .generate_return(&HirType::StringReference)
        .contains("\"\""));
}

#[test]
fn return_type_alias_size_t() {
    let gen = CodeGenerator::new();
    assert!(gen
        .generate_return(&HirType::TypeAlias("size_t".to_string()))
        .contains("0usize"));
}

#[test]
fn return_type_alias_ssize_t() {
    let gen = CodeGenerator::new();
    assert!(gen
        .generate_return(&HirType::TypeAlias("ssize_t".to_string()))
        .contains("0isize"));
}

#[test]
fn return_type_alias_ptrdiff_t() {
    let gen = CodeGenerator::new();
    assert!(gen
        .generate_return(&HirType::TypeAlias("ptrdiff_t".to_string()))
        .contains("0isize"));
}

#[test]
fn return_type_alias_other() {
    let gen = CodeGenerator::new();
    assert!(gen
        .generate_return(&HirType::TypeAlias("custom_t".to_string()))
        .contains("return 0;"));
}

// ============================================================================
// GENERATE_STRUCT TESTS
// ============================================================================

#[test]
fn struct_simple() {
    let gen = CodeGenerator::new();
    let s = HirStruct::new(
        "Point".to_string(),
        vec![
            HirStructField::new("x".to_string(), HirType::Int),
            HirStructField::new("y".to_string(), HirType::Int),
        ],
    );
    let result = gen.generate_struct(&s);
    assert!(result.contains("pub struct Point"));
    assert!(result.contains("pub x: i32"));
    assert!(result.contains("pub y: i32"));
    assert!(result.contains("Default"));
    assert!(result.contains("Copy")); // All primitive fields -> Copy
}

#[test]
fn struct_with_float_no_eq() {
    let gen = CodeGenerator::new();
    let s = HirStruct::new(
        "FloatPoint".to_string(),
        vec![
            HirStructField::new("x".to_string(), HirType::Float),
            HirStructField::new("y".to_string(), HirType::Float),
        ],
    );
    let result = gen.generate_struct(&s);
    assert!(result.contains("PartialEq"));
    assert!(!result.contains(", Eq"));
}

#[test]
fn struct_with_large_array_no_default() {
    let gen = CodeGenerator::new();
    let s = HirStruct::new(
        "Buffer".to_string(),
        vec![HirStructField::new(
            "data".to_string(),
            HirType::Array {
                element_type: Box::new(HirType::Char),
                size: Some(1024),
            },
        )],
    );
    let result = gen.generate_struct(&s);
    assert!(!result.contains("Default"));
}

#[test]
fn struct_with_reference_has_lifetime() {
    let gen = CodeGenerator::new();
    let s = HirStruct::new(
        "RefStruct".to_string(),
        vec![HirStructField::new(
            "data".to_string(),
            HirType::Reference {
                inner: Box::new(HirType::Int),
                mutable: false,
            },
        )],
    );
    let result = gen.generate_struct(&s);
    assert!(result.contains("<'a>"));
}

#[test]
fn struct_with_flexible_array_member() {
    let gen = CodeGenerator::new();
    let s = HirStruct::new(
        "FlexStruct".to_string(),
        vec![
            HirStructField::new("size".to_string(), HirType::Int),
            HirStructField::new(
                "data".to_string(),
                HirType::Array {
                    element_type: Box::new(HirType::Char),
                    size: None,
                },
            ),
        ],
    );
    let result = gen.generate_struct(&s);
    assert!(result.contains("Vec<u8>"));
}

#[test]
fn struct_with_pointer_field() {
    let gen = CodeGenerator::new();
    let s = HirStruct::new(
        "Node".to_string(),
        vec![
            HirStructField::new("data".to_string(), HirType::Int),
            HirStructField::new(
                "next".to_string(),
                HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
            ),
        ],
    );
    let result = gen.generate_struct(&s);
    assert!(result.contains("*mut Node"));
}

#[test]
fn struct_field_with_reserved_keyword() {
    let gen = CodeGenerator::new();
    let s = HirStruct::new(
        "KeywordStruct".to_string(),
        vec![HirStructField::new("type".to_string(), HirType::Int)],
    );
    let result = gen.generate_struct(&s);
    assert!(result.contains("r#type"));
}

// ============================================================================
// GENERATE_ENUM TESTS
// ============================================================================

#[test]
fn enum_simple() {
    let gen = CodeGenerator::new();
    let e = HirEnum::new(
        "Color".to_string(),
        vec![
            HirEnumVariant::new("RED".to_string(), Some(0)),
            HirEnumVariant::new("GREEN".to_string(), Some(1)),
            HirEnumVariant::new("BLUE".to_string(), Some(2)),
        ],
    );
    let result = gen.generate_enum(&e);
    assert!(result.contains("pub type Color = i32;"));
    assert!(result.contains("pub const RED: i32 = 0;"));
    assert!(result.contains("pub const GREEN: i32 = 1;"));
    assert!(result.contains("pub const BLUE: i32 = 2;"));
}

#[test]
fn enum_auto_incrementing() {
    let gen = CodeGenerator::new();
    let e = HirEnum::new(
        "Days".to_string(),
        vec![
            HirEnumVariant::new("MON".to_string(), Some(1)),
            HirEnumVariant::new("TUE".to_string(), None), // Should be 2
            HirEnumVariant::new("WED".to_string(), None), // Should be 3
        ],
    );
    let result = gen.generate_enum(&e);
    assert!(result.contains("pub const MON: i32 = 1;"));
    assert!(result.contains("pub const TUE: i32 = 2;"));
    assert!(result.contains("pub const WED: i32 = 3;"));
}

#[test]
fn enum_empty_name() {
    let gen = CodeGenerator::new();
    let e = HirEnum::new(
        "".to_string(),
        vec![HirEnumVariant::new("VALUE".to_string(), Some(42))],
    );
    let result = gen.generate_enum(&e);
    // Empty name should not produce "pub type = i32;"
    assert!(!result.contains("pub type  = i32;"));
    assert!(result.contains("pub const VALUE: i32 = 42;"));
}

// ============================================================================
// GENERATE_TYPEDEF TESTS
// ============================================================================

#[test]
fn typedef_simple() {
    let gen = CodeGenerator::new();
    let td = HirTypedef::new("Integer".to_string(), HirType::Int);
    let result = gen.generate_typedef(&td).unwrap();
    assert!(result.contains("pub type Integer = i32;"));
}

#[test]
fn typedef_pointer() {
    let gen = CodeGenerator::new();
    let td = HirTypedef::new(
        "IntPtr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let result = gen.generate_typedef(&td).unwrap();
    assert!(result.contains("pub type IntPtr = *mut i32;"));
}

#[test]
fn typedef_redundant_struct() {
    let gen = CodeGenerator::new();
    let td = HirTypedef::new("Node".to_string(), HirType::Struct("Node".to_string()));
    let result = gen.generate_typedef(&td).unwrap();
    assert!(result.contains("redundant"));
}

#[test]
fn typedef_size_t() {
    let gen = CodeGenerator::new();
    let td = HirTypedef::new("size_t".to_string(), HirType::UnsignedInt);
    let result = gen.generate_typedef(&td).unwrap();
    assert!(result.contains("pub type size_t = usize;"));
}

#[test]
fn typedef_ssize_t() {
    let gen = CodeGenerator::new();
    let td = HirTypedef::new("ssize_t".to_string(), HirType::Int);
    let result = gen.generate_typedef(&td).unwrap();
    assert!(result.contains("pub type ssize_t = isize;"));
}

#[test]
fn typedef_ptrdiff_t() {
    let gen = CodeGenerator::new();
    let td = HirTypedef::new("ptrdiff_t".to_string(), HirType::Int);
    let result = gen.generate_typedef(&td).unwrap();
    assert!(result.contains("pub type ptrdiff_t = isize;"));
}

#[test]
fn typedef_array_fixed_size() {
    let gen = CodeGenerator::new();
    let td = HirTypedef::new(
        "IntArray".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    let result = gen.generate_typedef(&td).unwrap();
    assert!(result.contains("pub type IntArray = [i32; 10];"));
}

#[test]
fn typedef_array_assertion_pattern() {
    let gen = CodeGenerator::new();
    let td = HirTypedef::new(
        "_assert".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Char),
            size: None,
        },
    );
    let result = gen.generate_typedef(&td).unwrap();
    assert!(result.contains("Compile-time assertion"));
}

// ============================================================================
// GENERATE_CONSTANT TESTS
// ============================================================================

#[test]
fn constant_int() {
    let gen = CodeGenerator::new();
    let c = HirConstant::new("MAX".to_string(), HirType::Int, HirExpression::IntLiteral(100));
    let result = gen.generate_constant(&c);
    assert!(result.contains("const MAX: i32 = 100;"));
}

#[test]
fn constant_string() {
    let gen = CodeGenerator::new();
    let c = HirConstant::new(
        "MSG".to_string(),
        HirType::Pointer(Box::new(HirType::Char)),
        HirExpression::StringLiteral("hello".to_string()),
    );
    let result = gen.generate_constant(&c);
    assert!(result.contains("const MSG: &str = \"hello\";"));
}

// ============================================================================
// GENERATE_GLOBAL_VARIABLE TESTS
// ============================================================================

#[test]
fn global_static_mutable() {
    let gen = CodeGenerator::new();
    let g = HirConstant::new(
        "counter".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(0),
    );
    let result = gen.generate_global_variable(&g, true, false, false);
    assert!(result.contains("static mut counter: i32 = 0;"));
}

#[test]
fn global_extern() {
    let gen = CodeGenerator::new();
    let g = HirConstant::new(
        "global_var".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(0),
    );
    let result = gen.generate_global_variable(&g, false, true, false);
    assert!(result.contains("extern \"C\""));
    assert!(result.contains("static global_var: i32;"));
}

#[test]
fn global_const() {
    let gen = CodeGenerator::new();
    let g = HirConstant::new(
        "MAX_SIZE".to_string(),
        HirType::Int,
        HirExpression::IntLiteral(1024),
    );
    let result = gen.generate_global_variable(&g, true, false, true);
    assert!(result.contains("const MAX_SIZE: i32 = 1024;"));
}

#[test]
fn global_const_string() {
    let gen = CodeGenerator::new();
    let g = HirConstant::new(
        "GREETING".to_string(),
        HirType::Pointer(Box::new(HirType::Char)),
        HirExpression::StringLiteral("hello".to_string()),
    );
    let result = gen.generate_global_variable(&g, true, false, true);
    assert!(result.contains("const GREETING: &str"));
}

#[test]
fn global_array_init() {
    let gen = CodeGenerator::new();
    let g = HirConstant::new(
        "buffer".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
        HirExpression::IntLiteral(0),
    );
    let result = gen.generate_global_variable(&g, true, false, false);
    assert!(result.contains("[0i32; 10]"));
}

#[test]
fn global_pointer_null() {
    let gen = CodeGenerator::new();
    let g = HirConstant::new(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        HirExpression::IntLiteral(0),
    );
    let result = gen.generate_global_variable(&g, true, false, false);
    assert!(result.contains("null_mut"));
}

// ============================================================================
// GENERATE_MACRO TESTS
// ============================================================================

#[test]
fn macro_object_like_int() {
    let gen = CodeGenerator::new();
    let m = HirMacroDefinition::new_object_like("MAX".to_string(), "100".to_string());
    let result = gen.generate_macro(&m).unwrap();
    assert!(result.contains("const MAX: i32 = 100;"));
}

#[test]
fn macro_object_like_empty() {
    let gen = CodeGenerator::new();
    let m = HirMacroDefinition::new_object_like("EMPTY".to_string(), "".to_string());
    let result = gen.generate_macro(&m).unwrap();
    assert!(result.contains("// Empty macro: EMPTY"));
}

#[test]
fn macro_object_like_string() {
    let gen = CodeGenerator::new();
    let m =
        HirMacroDefinition::new_object_like("GREETING".to_string(), "\"Hello\"".to_string());
    let result = gen.generate_macro(&m).unwrap();
    assert!(result.contains("&str"));
}

#[test]
fn macro_object_like_char() {
    let gen = CodeGenerator::new();
    let m = HirMacroDefinition::new_object_like("NEWLINE".to_string(), "'\\n'".to_string());
    let result = gen.generate_macro(&m).unwrap();
    assert!(result.contains("char"));
}

#[test]
fn macro_object_like_float() {
    let gen = CodeGenerator::new();
    let m = HirMacroDefinition::new_object_like("PI".to_string(), "3.14159".to_string());
    let result = gen.generate_macro(&m).unwrap();
    assert!(result.contains("f64"));
}

#[test]
fn macro_object_like_hex() {
    let gen = CodeGenerator::new();
    let m = HirMacroDefinition::new_object_like("FLAGS".to_string(), "0xFF".to_string());
    let result = gen.generate_macro(&m).unwrap();
    assert!(result.contains("i32"));
    assert!(result.contains("0xFF"));
}

#[test]
fn macro_object_like_octal() {
    let gen = CodeGenerator::new();
    let m = HirMacroDefinition::new_object_like("PERMS".to_string(), "0755".to_string());
    let result = gen.generate_macro(&m).unwrap();
    assert!(result.contains("0755"));
}

#[test]
fn macro_function_like() {
    let gen = CodeGenerator::new();
    let m = HirMacroDefinition::new_function_like(
        "SQR".to_string(),
        vec!["x".to_string()],
        "((x) * (x))".to_string(),
    );
    let result = gen.generate_macro(&m).unwrap();
    assert!(result.contains("#[inline]"));
    assert!(result.contains("fn sqr"));
    assert!(result.contains("x: i32"));
}

#[test]
fn macro_function_like_ternary() {
    let gen = CodeGenerator::new();
    let m = HirMacroDefinition::new_function_like(
        "MAX".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "((a)>(b)?(a):(b))".to_string(),
    );
    let result = gen.generate_macro(&m).unwrap();
    assert!(result.contains("if"));
    assert!(result.contains("else"));
}

// ============================================================================
// DEFAULT_VALUE_FOR_TYPE TESTS
// ============================================================================

#[test]
fn default_void() {
    assert_eq!(CodeGenerator::default_value_for_type(&HirType::Void), "()");
}

#[test]
fn default_int() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Int),
        "0i32"
    );
}

#[test]
fn default_unsigned_int() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::UnsignedInt),
        "0u32"
    );
}

#[test]
fn default_float() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Float),
        "0.0f32"
    );
}

#[test]
fn default_double() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Double),
        "0.0f64"
    );
}

#[test]
fn default_char() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Char),
        "0u8"
    );
}

#[test]
fn default_signed_char() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::SignedChar),
        "0i8"
    );
}

#[test]
fn default_pointer() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Pointer(Box::new(HirType::Int))),
        "std::ptr::null_mut()"
    );
}

#[test]
fn default_box() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Box(Box::new(HirType::Int))),
        "Box::new(0i32)"
    );
}

#[test]
fn default_vec() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Vec(Box::new(HirType::Int))),
        "Vec::new()"
    );
}

#[test]
fn default_option() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Option(Box::new(HirType::Int))),
        "None"
    );
}

#[test]
fn default_struct() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Struct("Point".to_string())),
        "Point::default()"
    );
}

#[test]
fn default_enum() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Enum("Color".to_string())),
        "Color::default()"
    );
}

#[test]
fn default_function_pointer() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::FunctionPointer {
            param_types: vec![],
            return_type: Box::new(HirType::Void),
        }),
        "None"
    );
}

#[test]
fn default_string_literal() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::StringLiteral),
        "\"\""
    );
}

#[test]
fn default_owned_string() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::OwnedString),
        "String::new()"
    );
}

#[test]
fn default_string_reference() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::StringReference),
        "\"\""
    );
}

#[test]
fn default_type_alias_size_t() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::TypeAlias("size_t".to_string())),
        "0usize"
    );
}

#[test]
fn default_type_alias_ssize_t() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::TypeAlias("ssize_t".to_string())),
        "0isize"
    );
}

#[test]
fn default_type_alias_other() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::TypeAlias("custom".to_string())),
        "0"
    );
}

#[test]
fn default_array_sized() {
    assert_eq!(
        CodeGenerator::default_value_for_type(&HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(5),
        }),
        "[0i32; 5]"
    );
}

// ============================================================================
// MAP_SIZEOF_TYPE TESTS
// ============================================================================

#[test]
fn sizeof_type_int() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.map_sizeof_type("int"), "i32");
}

#[test]
fn sizeof_type_short() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.map_sizeof_type("short"), "i16");
}

#[test]
fn sizeof_type_long() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.map_sizeof_type("long"), "i64");
}

#[test]
fn sizeof_type_long_long() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.map_sizeof_type("long long"), "i64");
}

#[test]
fn sizeof_type_unsigned_int() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.map_sizeof_type("unsigned int"), "u32");
}

#[test]
fn sizeof_type_unsigned_short() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.map_sizeof_type("unsigned short"), "u16");
}

#[test]
fn sizeof_type_unsigned_long() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.map_sizeof_type("unsigned long"), "u64");
}

#[test]
fn sizeof_type_unsigned_long_long() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.map_sizeof_type("unsigned long long"), "u64");
}

#[test]
fn sizeof_type_unsigned_char() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.map_sizeof_type("unsigned char"), "u8");
}

#[test]
fn sizeof_type_signed_char() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.map_sizeof_type("signed char"), "i8");
}

#[test]
fn sizeof_type_float() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.map_sizeof_type("float"), "f32");
}

#[test]
fn sizeof_type_double() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.map_sizeof_type("double"), "f64");
}

#[test]
fn sizeof_type_char() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.map_sizeof_type("char"), "u8");
}

#[test]
fn sizeof_type_void() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.map_sizeof_type("void"), "()");
}

#[test]
fn sizeof_type_char_pointer() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.map_sizeof_type("char*"), "*mut u8");
}

#[test]
fn sizeof_type_int_pointer() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.map_sizeof_type("int*"), "*mut i32");
}

#[test]
fn sizeof_type_void_pointer() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.map_sizeof_type("void*"), "*mut ()");
}

#[test]
fn sizeof_type_struct() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.map_sizeof_type("struct Data"), "Data");
}

#[test]
fn sizeof_type_custom() {
    let gen = CodeGenerator::new();
    assert_eq!(gen.map_sizeof_type("MyCustomType"), "MyCustomType");
}

// ============================================================================
// GENERATE_FUNCTION TESTS
// ============================================================================

#[test]
fn function_void_empty() {
    let gen = CodeGenerator::new();
    let func = HirFunction::new("test".to_string(), HirType::Void, vec![]);
    let result = gen.generate_function(&func);
    assert!(result.contains("fn test()"));
    assert!(result.contains("{"));
    assert!(result.contains("}"));
}

#[test]
fn function_with_return_type() {
    let gen = CodeGenerator::new();
    let func = HirFunction::new("get_value".to_string(), HirType::Int, vec![]);
    let result = gen.generate_function(&func);
    assert!(result.contains("fn get_value()"));
    assert!(result.contains("return 0"));
}

#[test]
fn function_main_no_return_type() {
    let gen = CodeGenerator::new();
    let func = HirFunction::new("main".to_string(), HirType::Int, vec![]);
    let result = gen.generate_function(&func);
    // main should not have -> i32
    assert!(result.contains("fn main()"));
    assert!(!result.contains("-> i32"));
}

#[test]
fn function_with_body() {
    let gen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "add".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }))],
    );
    let result = gen.generate_function(&func);
    assert!(result.contains("fn add"));
    assert!(result.contains("return a + b;"));
}

#[test]
fn function_main_return_becomes_exit() {
    let gen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "main".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
    );
    let result = gen.generate_function(&func);
    assert!(result.contains("std::process::exit(0)"));
}

#[test]
fn function_main_empty_return_becomes_exit_zero() {
    let gen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "main".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(None)],
    );
    let result = gen.generate_function(&func);
    assert!(result.contains("std::process::exit(0)"));
}

// ============================================================================
// ESCAPE_RUST_KEYWORD TESTS
// ============================================================================

#[test]
fn escape_keyword_type() {
    let result = super::escape_rust_keyword("type");
    assert_eq!(result, "r#type");
}

#[test]
fn escape_keyword_fn() {
    let result = super::escape_rust_keyword("fn");
    assert_eq!(result, "r#fn");
}

#[test]
fn escape_keyword_match() {
    let result = super::escape_rust_keyword("match");
    assert_eq!(result, "r#match");
}

#[test]
fn escape_keyword_self() {
    let result = super::escape_rust_keyword("self");
    assert_eq!(result, "r#self");
}

#[test]
fn escape_keyword_return() {
    let result = super::escape_rust_keyword("return");
    assert_eq!(result, "r#return");
}

#[test]
fn escape_non_keyword() {
    let result = super::escape_rust_keyword("my_variable");
    assert_eq!(result, "my_variable");
}

#[test]
fn escape_keyword_abstract() {
    // Reserved for future use
    let result = super::escape_rust_keyword("abstract");
    assert_eq!(result, "r#abstract");
}

// ============================================================================
// BINARY OPERATOR STRING TESTS
// ============================================================================

#[test]
fn binary_ops_all_variants() {
    assert_eq!(CodeGenerator::binary_operator_to_string(&BinaryOperator::Add), "+");
    assert_eq!(CodeGenerator::binary_operator_to_string(&BinaryOperator::Subtract), "-");
    assert_eq!(CodeGenerator::binary_operator_to_string(&BinaryOperator::Multiply), "*");
    assert_eq!(CodeGenerator::binary_operator_to_string(&BinaryOperator::Divide), "/");
    assert_eq!(CodeGenerator::binary_operator_to_string(&BinaryOperator::Modulo), "%");
    assert_eq!(CodeGenerator::binary_operator_to_string(&BinaryOperator::Equal), "==");
    assert_eq!(CodeGenerator::binary_operator_to_string(&BinaryOperator::NotEqual), "!=");
    assert_eq!(CodeGenerator::binary_operator_to_string(&BinaryOperator::LessThan), "<");
    assert_eq!(CodeGenerator::binary_operator_to_string(&BinaryOperator::GreaterThan), ">");
    assert_eq!(CodeGenerator::binary_operator_to_string(&BinaryOperator::LessEqual), "<=");
    assert_eq!(CodeGenerator::binary_operator_to_string(&BinaryOperator::GreaterEqual), ">=");
    assert_eq!(CodeGenerator::binary_operator_to_string(&BinaryOperator::LogicalAnd), "&&");
    assert_eq!(CodeGenerator::binary_operator_to_string(&BinaryOperator::LogicalOr), "||");
    assert_eq!(CodeGenerator::binary_operator_to_string(&BinaryOperator::LeftShift), "<<");
    assert_eq!(CodeGenerator::binary_operator_to_string(&BinaryOperator::RightShift), ">>");
    assert_eq!(CodeGenerator::binary_operator_to_string(&BinaryOperator::BitwiseAnd), "&");
    assert_eq!(CodeGenerator::binary_operator_to_string(&BinaryOperator::BitwiseOr), "|");
    assert_eq!(CodeGenerator::binary_operator_to_string(&BinaryOperator::BitwiseXor), "^");
    assert_eq!(CodeGenerator::binary_operator_to_string(&BinaryOperator::Assign), "=");
    assert_eq!(CodeGenerator::binary_operator_to_string(&BinaryOperator::Comma), ",");
}

// ============================================================================
// IS_BOOLEAN_EXPRESSION TESTS
// ============================================================================

#[test]
fn is_boolean_comparison() {
    assert!(CodeGenerator::is_boolean_expression(&HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::IntLiteral(1)),
        right: Box::new(HirExpression::IntLiteral(2)),
    }));
}

#[test]
fn is_boolean_logical_and() {
    assert!(CodeGenerator::is_boolean_expression(&HirExpression::BinaryOp {
        op: BinaryOperator::LogicalAnd,
        left: Box::new(HirExpression::IntLiteral(1)),
        right: Box::new(HirExpression::IntLiteral(2)),
    }));
}

#[test]
fn is_not_boolean_add() {
    assert!(!CodeGenerator::is_boolean_expression(&HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::IntLiteral(1)),
        right: Box::new(HirExpression::IntLiteral(2)),
    }));
}

#[test]
fn is_boolean_logical_not() {
    assert!(CodeGenerator::is_boolean_expression(&HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::IntLiteral(1)),
    }));
}

#[test]
fn is_not_boolean_variable() {
    assert!(!CodeGenerator::is_boolean_expression(&HirExpression::Variable("x".to_string())));
}

// ============================================================================
// FORMAT SPECIFIER TESTS
// ============================================================================

#[test]
fn format_percent_d() {
    let result = CodeGenerator::convert_c_format_to_rust("\"%d\"");
    assert_eq!(result, "\"{}\"");
}

#[test]
fn format_percent_s() {
    let result = CodeGenerator::convert_c_format_to_rust("\"%s\"");
    assert_eq!(result, "\"{}\"");
}

#[test]
fn format_percent_f() {
    let result = CodeGenerator::convert_c_format_to_rust("\"%f\"");
    assert_eq!(result, "\"{}\"");
}

#[test]
fn format_percent_x() {
    let result = CodeGenerator::convert_c_format_to_rust("\"%x\"");
    assert_eq!(result, "\"{:x}\"");
}

#[test]
fn format_percent_upper_x() {
    let result = CodeGenerator::convert_c_format_to_rust("\"%X\"");
    assert_eq!(result, "\"{:X}\"");
}

#[test]
fn format_percent_o() {
    let result = CodeGenerator::convert_c_format_to_rust("\"%o\"");
    assert_eq!(result, "\"{:o}\"");
}

#[test]
fn format_percent_p() {
    let result = CodeGenerator::convert_c_format_to_rust("\"%p\"");
    assert_eq!(result, "\"{:p}\"");
}

#[test]
fn format_percent_e() {
    let result = CodeGenerator::convert_c_format_to_rust("\"%e\"");
    assert_eq!(result, "\"{:e}\"");
}

#[test]
fn format_percent_upper_e() {
    let result = CodeGenerator::convert_c_format_to_rust("\"%E\"");
    assert_eq!(result, "\"{:E}\"");
}

#[test]
fn format_percent_c() {
    let result = CodeGenerator::convert_c_format_to_rust("\"%c\"");
    assert_eq!(result, "\"{}\"");
}

#[test]
fn format_percent_percent() {
    let result = CodeGenerator::convert_c_format_to_rust("\"%%\"");
    assert_eq!(result, "\"%\"");
}

#[test]
fn format_width_specifier() {
    let result = CodeGenerator::convert_c_format_to_rust("\"%10d\"");
    assert_eq!(result, "\"{:10}\"");
}

#[test]
fn format_zero_padded() {
    let result = CodeGenerator::convert_c_format_to_rust("\"%02X\"");
    assert_eq!(result, "\"{:02X}\"");
}

#[test]
fn format_precision_float() {
    let result = CodeGenerator::convert_c_format_to_rust("\"%.2f\"");
    assert_eq!(result, "\"{:.2}\"");
}

#[test]
fn format_non_string_passthrough() {
    let result = CodeGenerator::convert_c_format_to_rust("some_var");
    assert_eq!(result, "some_var");
}

// ============================================================================
// IS_STRING_TERNARY TESTS
// ============================================================================

#[test]
fn is_string_ternary_true() {
    assert!(CodeGenerator::is_string_ternary(&HirExpression::Ternary {
        condition: Box::new(HirExpression::IntLiteral(1)),
        then_expr: Box::new(HirExpression::StringLiteral("yes".to_string())),
        else_expr: Box::new(HirExpression::StringLiteral("no".to_string())),
    }));
}

#[test]
fn is_string_ternary_false_not_ternary() {
    assert!(!CodeGenerator::is_string_ternary(&HirExpression::IntLiteral(0)));
}

#[test]
fn is_string_ternary_false_non_string_branches() {
    assert!(!CodeGenerator::is_string_ternary(&HirExpression::Ternary {
        condition: Box::new(HirExpression::IntLiteral(1)),
        then_expr: Box::new(HirExpression::IntLiteral(1)),
        else_expr: Box::new(HirExpression::IntLiteral(0)),
    }));
}

// ============================================================================
// VLA VARIABLE DECLARATION TESTS
// ============================================================================

#[test]
fn stmt_vla_declaration_int() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    });
    assert!(result.contains("vec!"));
    assert!(result.contains("0i32"));
}

#[test]
fn stmt_vla_declaration_unsigned() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::UnsignedInt),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    });
    assert!(result.contains("0u32"));
}

#[test]
fn stmt_vla_declaration_float() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Float),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    });
    assert!(result.contains("0.0f32"));
}

#[test]
fn stmt_vla_declaration_double() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Double),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    });
    assert!(result.contains("0.0f64"));
}

#[test]
fn stmt_vla_declaration_char() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Char),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    });
    assert!(result.contains("0u8"));
}

#[test]
fn stmt_vla_declaration_signed_char() {
    let gen = CodeGenerator::new();
    let result = gen.generate_statement(&HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::SignedChar),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    });
    assert!(result.contains("0i8"));
}

// ============================================================================
// CODEGEN DEFAULT TRAIT TEST
// ============================================================================

#[test]
fn codegen_default() {
    let gen = CodeGenerator::default();
    // Just verify default creates a working instance
    assert_eq!(gen.generate_expression(&HirExpression::IntLiteral(1)), "1");
}

// ============================================================================
// UNSAFE BLOCK/STMT HELPERS
// ============================================================================

#[test]
fn unsafe_block_format() {
    let result = CodeGenerator::unsafe_block("*ptr", "pointer is valid");
    assert!(result.contains("SAFETY: pointer is valid"));
    assert!(result.contains("unsafe { *ptr }"));
}

#[test]
fn unsafe_stmt_format() {
    let result = CodeGenerator::unsafe_stmt("*ptr = 42", "pointer is valid");
    assert!(result.contains("SAFETY: pointer is valid"));
    assert!(result.contains("unsafe { *ptr = 42; }"));
}

// ============================================================================
// FIND_STRING_FORMAT_POSITIONS TESTS
// ============================================================================

#[test]
fn find_string_positions_simple() {
    let positions = CodeGenerator::find_string_format_positions("\"%s %d %s\"");
    assert_eq!(positions, vec![0, 2]);
}

#[test]
fn find_string_positions_no_string() {
    let positions = CodeGenerator::find_string_format_positions("\"%d %f\"");
    assert!(positions.is_empty());
}

#[test]
fn find_string_positions_percent_percent() {
    let positions = CodeGenerator::find_string_format_positions("\"%%s\"");
    // %% is literal percent, s is just a char
    assert!(positions.is_empty());
}

// ============================================================================
// SWITCH WITH CHAR LITERAL CASE TESTS
// ============================================================================

#[test]
fn stmt_switch_with_char_literal_case() {
    let gen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test_switch".to_string(),
        HirType::Int,
        vec![HirParameter::new("c".to_string(), HirType::Int)],
        vec![HirStatement::Switch {
            condition: HirExpression::Variable("c".to_string()),
            cases: vec![SwitchCase {
                value: Some(HirExpression::CharLiteral(b'0' as i8)),
                body: vec![
                    HirStatement::Return(Some(HirExpression::IntLiteral(0))),
                    HirStatement::Break,
                ],
            }],
            default_case: Some(vec![HirStatement::Return(Some(
                HirExpression::IntLiteral(-1),
            ))]),
        }],
    );
    let result = gen.generate_function(&func);
    // The char literal should be converted to numeric value (48 for '0')
    assert!(result.contains("48"));
}

// ============================================================================
// EXEC FUNCTION CALLS
// ============================================================================

#[test]
fn expr_execl_with_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "execl".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("/bin/ls".to_string()),
            HirExpression::StringLiteral("ls".to_string()),
            HirExpression::StringLiteral("-l".to_string()),
            HirExpression::NullLiteral,
        ],
    });
    assert!(result.contains("Command::new"));
    assert!(result.contains(".arg"));
}

#[test]
fn expr_execl_no_extra_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "execl".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("/bin/ls".to_string()),
            HirExpression::StringLiteral("ls".to_string()),
            HirExpression::NullLiteral,
        ],
    });
    assert!(result.contains("Command::new"));
}

#[test]
fn expr_exec_no_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "execl".to_string(),
        arguments: vec![],
    });
    assert!(result.contains("exec requires args"));
}

#[test]
fn expr_waitpid() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "waitpid".to_string(),
        arguments: vec![HirExpression::IntLiteral(0)],
    });
    assert!(result.contains("wait"));
}

// ============================================================================
// FPRINTF FUNCTION CALL
// ============================================================================

#[test]
fn expr_fprintf_simple() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "fprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("stderr".to_string()),
            HirExpression::StringLiteral("error\\n".to_string()),
        ],
    });
    assert!(result.contains("write!"));
}

#[test]
fn expr_fprintf_with_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "fprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("stderr".to_string()),
            HirExpression::StringLiteral("error: %d\\n".to_string()),
            HirExpression::IntLiteral(42),
        ],
    });
    assert!(result.contains("write!"));
}

#[test]
fn expr_fprintf_no_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "fprintf".to_string(),
        arguments: vec![],
    });
    assert!(result.contains("fprintf requires 2+ args"));
}

// ============================================================================
// NO-ARG VARIANTS OF FUNCTIONS
// ============================================================================

#[test]
fn expr_fopen_no_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "fopen".to_string(),
        arguments: vec![],
    });
    assert!(result.contains("fopen requires 2 args"));
}

#[test]
fn expr_fclose_no_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "fclose".to_string(),
        arguments: vec![],
    });
    assert!(result.contains("fclose"));
}

#[test]
fn expr_fgetc_no_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "fgetc".to_string(),
        arguments: vec![],
    });
    assert!(result.contains("fgetc requires 1 arg"));
}

#[test]
fn expr_fputc_no_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "fputc".to_string(),
        arguments: vec![],
    });
    assert!(result.contains("fputc requires 2 args"));
}

#[test]
fn expr_fread_no_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "fread".to_string(),
        arguments: vec![],
    });
    assert!(result.contains("fread requires 4 args"));
}

#[test]
fn expr_fwrite_no_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "fwrite".to_string(),
        arguments: vec![],
    });
    assert!(result.contains("fwrite requires 4 args"));
}

#[test]
fn expr_fputs_no_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "fputs".to_string(),
        arguments: vec![],
    });
    assert!(result.contains("fputs requires 2 args"));
}

#[test]
fn expr_wexitstatus_no_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "WEXITSTATUS".to_string(),
        arguments: vec![],
    });
    assert!(result.contains("WEXITSTATUS requires status arg"));
}

#[test]
fn expr_wifexited_no_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "WIFEXITED".to_string(),
        arguments: vec![],
    });
    assert!(result.contains("WIFEXITED requires status arg"));
}

#[test]
fn expr_wifsignaled_no_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "WIFSIGNALED".to_string(),
        arguments: vec![],
    });
    assert!(result.contains("WIFSIGNALED requires status arg"));
}

#[test]
fn expr_wtermsig_no_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "WTERMSIG".to_string(),
        arguments: vec![],
    });
    assert!(result.contains("WTERMSIG requires status arg"));
}

#[test]
fn expr_strlen_invalid_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "strlen".to_string(),
        arguments: vec![
            HirExpression::Variable("a".to_string()),
            HirExpression::Variable("b".to_string()),
        ],
    });
    // Invalid strlen call fallback
    assert!(result.contains("strlen(a, b)"));
}

#[test]
fn expr_strcpy_invalid_args() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FunctionCall {
        function: "strcpy".to_string(),
        arguments: vec![],
    });
    assert!(result.contains("strcpy()"));
}

// ============================================================================
// BOX TRANSFORMER ACCESS
// ============================================================================

#[test]
fn box_transformer_accessible() {
    let gen = CodeGenerator::new();
    let _bt = gen.box_transformer();
    // Just verifying the accessor works
}

// ============================================================================
// STRUCT WITH NO COPY (Box field)
// ============================================================================

#[test]
fn struct_with_box_field_no_copy() {
    let gen = CodeGenerator::new();
    let s = HirStruct::new(
        "BoxStruct".to_string(),
        vec![HirStructField::new(
            "data".to_string(),
            HirType::Box(Box::new(HirType::Int)),
        )],
    );
    let result = gen.generate_struct(&s);
    assert!(!result.contains("Copy"));
    assert!(result.contains("Default"));
}

// ============================================================================
// COMPOUND LITERAL PARTIAL INITIALIZATION
// ============================================================================

#[test]
fn expr_compound_literal_array_partial_init() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(5),
        },
        initializers: vec![HirExpression::IntLiteral(1), HirExpression::IntLiteral(2)],
    });
    // Partial: 2 values for size 5 array, should pad with defaults
    assert!(result.contains("1"));
    assert!(result.contains("2"));
    assert!(result.contains("0i32"));
}

// ============================================================================
// LOGICAL OPERATORS WITH INTEGER OPERANDS
// ============================================================================

#[test]
fn expr_logical_and_with_int_operands() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::BinaryOp {
        op: BinaryOperator::LogicalAnd,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::Variable("y".to_string())),
    });
    assert!(result.contains("!= 0"));
    assert!(result.contains("&&"));
}

#[test]
fn expr_logical_or_with_boolean_operands() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::BinaryOp {
        op: BinaryOperator::LogicalOr,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::IntLiteral(1)),
            right: Box::new(HirExpression::IntLiteral(1)),
        }),
        right: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::NotEqual,
            left: Box::new(HirExpression::IntLiteral(2)),
            right: Box::new(HirExpression::IntLiteral(3)),
        }),
    });
    assert!(result.contains("||"));
    // Boolean operands shouldn't get != 0 wrapping
    assert!(!result.contains("!= 0"));
}

// ============================================================================
// VARIABLE RESERVED KEYWORD IN FIELD ACCESS
// ============================================================================

#[test]
fn expr_field_access_reserved_keyword() {
    let gen = CodeGenerator::new();
    let result = gen.generate_expression(&HirExpression::FieldAccess {
        object: Box::new(HirExpression::Variable("obj".to_string())),
        field: "type".to_string(),
    });
    assert!(result.contains("r#type"));
}
