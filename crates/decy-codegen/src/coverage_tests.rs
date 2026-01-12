//! Coverage improvement tests for CodeGenerator (DECY-COVERAGE)
//!
//! Tests for helper functions to increase coverage to 95%.

use super::*;
use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType};

// ============================================================================
// TypeContext Tests
// ============================================================================

#[test]
fn test_type_context_new() {
    let ctx = TypeContext::new();
    assert!(ctx.get_type("any").is_none());
    assert!(!ctx.is_pointer("any"));
    assert!(!ctx.is_option("any"));
    assert!(!ctx.is_vec("any"));
}

#[test]
fn test_type_context_add_variable() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    assert!(ctx.get_type("x").is_some());
    assert_eq!(ctx.get_type("x").unwrap(), &HirType::Int);
}

#[test]
fn test_type_context_is_pointer() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    ctx.add_variable("x".to_string(), HirType::Int);
    assert!(ctx.is_pointer("ptr"));
    assert!(!ctx.is_pointer("x"));
}

#[test]
fn test_type_context_is_option() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("opt".to_string(), HirType::Option(Box::new(HirType::Int)));
    ctx.add_variable("x".to_string(), HirType::Int);
    assert!(ctx.is_option("opt"));
    assert!(!ctx.is_option("x"));
}

#[test]
fn test_type_context_is_vec() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("vec".to_string(), HirType::Vec(Box::new(HirType::Int)));
    ctx.add_variable("x".to_string(), HirType::Int);
    assert!(ctx.is_vec("vec"));
    assert!(!ctx.is_vec("x"));
}

#[test]
fn test_type_context_from_function() {
    let params = vec![HirParameter::new("a".to_string(), HirType::Int)];
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        params,
        vec![HirStatement::Return(None)],
    );

    let ctx = TypeContext::from_function(&func);
    assert!(ctx.get_type("a").is_some());
}

#[test]
fn test_type_context_add_renamed_local() {
    let mut ctx = TypeContext::new();
    ctx.add_renamed_local("old".to_string(), "new".to_string());
    assert_eq!(ctx.get_renamed_local("old"), Some(&"new".to_string()));
    assert_eq!(ctx.get_renamed_local("unknown"), None);
}

#[test]
fn test_type_context_add_global() {
    let mut ctx = TypeContext::new();
    ctx.add_global("GLOBAL_VAR".to_string());
    assert!(ctx.is_global("GLOBAL_VAR"));
    assert!(!ctx.is_global("local_var"));
}

#[test]
fn test_type_context_string_iter_func() {
    let mut ctx = TypeContext::new();
    ctx.add_string_iter_func("iter_func".to_string(), vec![(0, true)]);
    assert!(ctx.get_string_iter_func("iter_func").is_some());
    assert!(ctx.get_string_iter_func("unknown").is_none());
}

#[test]
fn test_type_context_string_iter_param() {
    let mut ctx = TypeContext::new();
    ctx.add_string_iter_param("str_param".to_string(), "i".to_string());
    assert!(ctx.is_string_iter_param("str_param"));
    assert!(!ctx.is_string_iter_param("other"));
    assert_eq!(
        ctx.get_string_iter_index("str_param"),
        Some(&"i".to_string())
    );
}

#[test]
fn test_type_context_add_function() {
    let mut ctx = TypeContext::new();
    ctx.add_function("foo".to_string(), vec![HirType::Int, HirType::Float]);
    assert_eq!(ctx.get_function_param_type("foo", 0), Some(&HirType::Int));
    assert_eq!(ctx.get_function_param_type("foo", 1), Some(&HirType::Float));
    assert_eq!(ctx.get_function_param_type("foo", 2), None);
    assert_eq!(ctx.get_function_param_type("bar", 0), None);
}

#[test]
fn test_type_context_slice_func_args() {
    let mut ctx = TypeContext::new();
    ctx.add_slice_func_args("process".to_string(), vec![(0, 1)]);
    assert!(ctx.get_slice_func_len_indices("process").is_some());
    assert!(ctx.get_slice_func_len_indices("unknown").is_none());
}

// ============================================================================
// CodeGenerator map_type Tests
// ============================================================================

#[test]
fn test_map_type_primitives() {
    assert_eq!(CodeGenerator::map_type(&HirType::Void), "()");
    assert_eq!(CodeGenerator::map_type(&HirType::Int), "i32");
    assert_eq!(CodeGenerator::map_type(&HirType::Float), "f32");
    assert_eq!(CodeGenerator::map_type(&HirType::Double), "f64");
    assert_eq!(CodeGenerator::map_type(&HirType::Char), "u8");
    assert_eq!(CodeGenerator::map_type(&HirType::UnsignedInt), "u32");
    assert_eq!(CodeGenerator::map_type(&HirType::SignedChar), "i8");
}

#[test]
fn test_map_type_pointers() {
    let ptr_int = HirType::Pointer(Box::new(HirType::Int));
    assert!(CodeGenerator::map_type(&ptr_int).contains("i32"));
}

#[test]
fn test_map_type_arrays() {
    let arr = HirType::Array {
        element_type: Box::new(HirType::Int),
        size: Some(10),
    };
    let result = CodeGenerator::map_type(&arr);
    assert!(result.contains("i32"));
    assert!(result.contains("10"));

    // Unsized array
    let unsized_arr = HirType::Array {
        element_type: Box::new(HirType::Int),
        size: None,
    };
    let unsized_result = CodeGenerator::map_type(&unsized_arr);
    assert!(unsized_result.contains("i32"));
}

#[test]
fn test_map_type_vec() {
    let vec_type = HirType::Vec(Box::new(HirType::Int));
    assert!(CodeGenerator::map_type(&vec_type).contains("Vec"));
}

#[test]
fn test_map_type_option() {
    let opt_type = HirType::Option(Box::new(HirType::Int));
    assert!(CodeGenerator::map_type(&opt_type).contains("Option"));
}

#[test]
fn test_map_type_box() {
    let box_type = HirType::Box(Box::new(HirType::Int));
    assert!(CodeGenerator::map_type(&box_type).contains("Box"));
}

#[test]
fn test_map_type_struct() {
    let struct_type = HirType::Struct("Point".to_string());
    assert_eq!(CodeGenerator::map_type(&struct_type), "Point");
}

#[test]
fn test_map_type_reference() {
    let ref_type = HirType::Reference {
        inner: Box::new(HirType::Int),
        mutable: false,
    };
    assert!(CodeGenerator::map_type(&ref_type).contains("&"));
    assert!(CodeGenerator::map_type(&ref_type).contains("i32"));

    let mut_ref = HirType::Reference {
        inner: Box::new(HirType::Int),
        mutable: true,
    };
    assert!(CodeGenerator::map_type(&mut_ref).contains("&mut"));
}

#[test]
fn test_map_type_reference_to_vec_becomes_slice() {
    // &Vec<T> → &[T]
    let ref_vec = HirType::Reference {
        inner: Box::new(HirType::Vec(Box::new(HirType::Int))),
        mutable: false,
    };
    let result = CodeGenerator::map_type(&ref_vec);
    assert!(result.contains("&[i32]"));

    // &mut Vec<T> → &mut [T]
    let mut_ref_vec = HirType::Reference {
        inner: Box::new(HirType::Vec(Box::new(HirType::Int))),
        mutable: true,
    };
    let mut_result = CodeGenerator::map_type(&mut_ref_vec);
    assert!(mut_result.contains("&mut [i32]"));
}

#[test]
fn test_map_type_function_pointer() {
    let fn_ptr = HirType::FunctionPointer {
        param_types: vec![HirType::Int, HirType::Int],
        return_type: Box::new(HirType::Int),
    };
    let result = CodeGenerator::map_type(&fn_ptr);
    assert!(result.contains("fn("));
    assert!(result.contains("i32"));

    let void_fn = HirType::FunctionPointer {
        param_types: vec![],
        return_type: Box::new(HirType::Void),
    };
    let void_result = CodeGenerator::map_type(&void_fn);
    assert!(void_result.contains("fn()"));
}

#[test]
fn test_map_type_strings() {
    assert_eq!(CodeGenerator::map_type(&HirType::StringLiteral), "&str");
    assert_eq!(CodeGenerator::map_type(&HirType::OwnedString), "String");
    assert_eq!(CodeGenerator::map_type(&HirType::StringReference), "&str");
}

#[test]
fn test_map_type_enum() {
    let enum_type = HirType::Enum("Color".to_string());
    assert_eq!(CodeGenerator::map_type(&enum_type), "Color");
}

#[test]
fn test_map_type_union() {
    let union_type = HirType::Union(vec![
        ("field1".to_string(), HirType::Int),
        ("field2".to_string(), HirType::Float),
    ]);
    let result = CodeGenerator::map_type(&union_type);
    assert!(result.contains("Union"));
}

#[test]
fn test_map_type_type_alias() {
    let alias = HirType::TypeAlias("size_t".to_string());
    assert_eq!(CodeGenerator::map_type(&alias), "size_t");
}

// ============================================================================
// CodeGenerator generate_expression Tests
// ============================================================================

#[test]
fn test_generate_expression_literals() {
    let gen = CodeGenerator::new();

    let int_lit = HirExpression::IntLiteral(42);
    assert_eq!(gen.generate_expression(&int_lit), "42");

    let float_lit = HirExpression::FloatLiteral("3.14".to_string());
    let result = gen.generate_expression(&float_lit);
    assert!(result.contains("3.14"));

    let char_lit = HirExpression::CharLiteral(65); // ASCII 'A'
    assert!(
        gen.generate_expression(&char_lit).contains("65")
            || gen.generate_expression(&char_lit).contains("b'A'")
    );
}

#[test]
fn test_generate_expression_variable() {
    let gen = CodeGenerator::new();
    let var = HirExpression::Variable("foo".to_string());
    assert_eq!(gen.generate_expression(&var), "foo");
}

#[test]
fn test_generate_expression_binary_ops() {
    let gen = CodeGenerator::new();

    let add = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::IntLiteral(1)),
        right: Box::new(HirExpression::IntLiteral(2)),
    };
    assert!(gen.generate_expression(&add).contains("+"));

    let sub = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::IntLiteral(5)),
        right: Box::new(HirExpression::IntLiteral(3)),
    };
    assert!(gen.generate_expression(&sub).contains("-"));
}

#[test]
fn test_generate_expression_unary_ops() {
    let gen = CodeGenerator::new();

    let negate = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::Minus,
        operand: Box::new(HirExpression::IntLiteral(42)),
    };
    assert!(gen.generate_expression(&negate).contains("-"));

    // LogicalNot on int literal generates "(val == 0)" rather than "!"
    let logical_not = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::IntLiteral(1)),
    };
    let not_result = gen.generate_expression(&logical_not);
    assert!(not_result.contains("== 0"), "Got: {}", not_result);
}

#[test]
fn test_generate_expression_address_of() {
    let gen = CodeGenerator::new();
    let addr = HirExpression::AddressOf(Box::new(HirExpression::Variable("x".to_string())));
    assert!(gen.generate_expression(&addr).contains("&"));
}

#[test]
fn test_generate_expression_dereference() {
    let gen = CodeGenerator::new();
    let deref = HirExpression::Dereference(Box::new(HirExpression::Variable("ptr".to_string())));
    assert!(gen.generate_expression(&deref).contains("*"));
}

#[test]
fn test_generate_expression_function_call() {
    let gen = CodeGenerator::new();
    let call = HirExpression::FunctionCall {
        function: "test_func".to_string(),
        arguments: vec![HirExpression::IntLiteral(1)],
    };
    let result = gen.generate_expression(&call);
    assert!(result.contains("test_func"));
}

#[test]
fn test_generate_expression_array_index() {
    let gen = CodeGenerator::new();
    let index = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(5)),
    };
    let result = gen.generate_expression(&index);
    assert!(result.contains("arr"));
    assert!(result.contains("5"));
}

#[test]
fn test_generate_expression_cast() {
    let gen = CodeGenerator::new();
    let cast = HirExpression::Cast {
        expr: Box::new(HirExpression::IntLiteral(42)),
        target_type: HirType::Float,
    };
    let result = gen.generate_expression(&cast);
    assert!(result.contains("as"));
}

// ============================================================================
// CodeGenerator generate_statement Tests
// ============================================================================

#[test]
fn test_generate_statement_variable_declaration() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(42)),
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("let"));
    assert!(result.contains("x"));
}

#[test]
fn test_generate_statement_assignment() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::Assignment {
        target: "x".to_string(),
        value: HirExpression::IntLiteral(10),
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("x"));
    assert!(result.contains("10"));
}

#[test]
fn test_generate_statement_return() {
    let gen = CodeGenerator::new();

    let ret_val = HirStatement::Return(Some(HirExpression::IntLiteral(0)));
    let result = gen.generate_statement(&ret_val);
    assert!(result.contains("return"));
    assert!(result.contains("0"));

    let ret_void = HirStatement::Return(None);
    let result = gen.generate_statement(&ret_void);
    assert!(result.contains("return"));
}

#[test]
fn test_generate_statement_break_continue() {
    let gen = CodeGenerator::new();

    let break_stmt = HirStatement::Break;
    assert!(gen.generate_statement(&break_stmt).contains("break"));

    let continue_stmt = HirStatement::Continue;
    assert!(gen.generate_statement(&continue_stmt).contains("continue"));
}

// ============================================================================
// Binary Operator String Tests
// ============================================================================

#[test]
fn test_binary_operator_to_string() {
    use super::CodeGenerator;

    let ops_and_expected = [
        (BinaryOperator::Add, "+"),
        (BinaryOperator::Subtract, "-"),
        (BinaryOperator::Multiply, "*"),
        (BinaryOperator::Divide, "/"),
        (BinaryOperator::Modulo, "%"),
        (BinaryOperator::Equal, "=="),
        (BinaryOperator::NotEqual, "!="),
        (BinaryOperator::LessThan, "<"),
        (BinaryOperator::GreaterThan, ">"),
        (BinaryOperator::LessEqual, "<="),
        (BinaryOperator::GreaterEqual, ">="),
        (BinaryOperator::LogicalAnd, "&&"),
        (BinaryOperator::LogicalOr, "||"),
        (BinaryOperator::BitwiseAnd, "&"),
        (BinaryOperator::BitwiseOr, "|"),
        (BinaryOperator::BitwiseXor, "^"),
        (BinaryOperator::LeftShift, "<<"),
        (BinaryOperator::RightShift, ">>"),
    ];

    for (op, expected) in ops_and_expected {
        let result = CodeGenerator::binary_operator_to_string(&op);
        assert_eq!(
            result, expected,
            "Operator {:?} should map to {}",
            op, expected
        );
    }
}

// ============================================================================
// Unary Operator String Tests
// ============================================================================

#[test]
fn test_unary_operator_to_string() {
    use decy_hir::UnaryOperator;

    let ops_and_expected = [
        (UnaryOperator::Minus, "-"),
        (UnaryOperator::LogicalNot, "!"),
        (UnaryOperator::BitwiseNot, "!"), // Bitwise not maps to ! in some contexts
    ];

    for (op, expected) in ops_and_expected {
        let result = CodeGenerator::unary_operator_to_string(&op);
        assert!(
            result.contains(expected) || result == expected,
            "Operator {:?} should contain {}",
            op,
            expected
        );
    }
}

// ============================================================================
// Default Value Tests
// ============================================================================

#[test]
fn test_default_value_for_type() {
    let defaults = [
        (HirType::Int, "0"),
        (HirType::Float, "0.0"),
        (HirType::Char, "0"),
        (HirType::Void, "()"),
        (HirType::Double, "0.0"),
        (HirType::UnsignedInt, "0"),
    ];

    for (ty, expected) in defaults {
        let result = CodeGenerator::default_value_for_type(&ty);
        assert!(
            result.contains(expected),
            "Type {:?} should have default containing {}",
            ty,
            expected
        );
    }
}

// ============================================================================
// Helper Function Tests
// ============================================================================

#[test]
fn test_is_malloc_expression() {
    let malloc_call = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(100)],
    };
    assert!(CodeGenerator::is_malloc_expression(&malloc_call));

    let other_call = HirExpression::FunctionCall {
        function: "foo".to_string(),
        arguments: vec![],
    };
    assert!(!CodeGenerator::is_malloc_expression(&other_call));
}

#[test]
fn test_is_boolean_expression() {
    let cmp_expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::IntLiteral(1)),
        right: Box::new(HirExpression::IntLiteral(1)),
    };
    assert!(CodeGenerator::is_boolean_expression(&cmp_expr));

    let logical_expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalAnd,
        left: Box::new(HirExpression::IntLiteral(1)),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    assert!(CodeGenerator::is_boolean_expression(&logical_expr));

    let int_expr = HirExpression::IntLiteral(42);
    assert!(!CodeGenerator::is_boolean_expression(&int_expr));
}

// ============================================================================
// Struct Field Tests
// ============================================================================

#[test]
fn test_type_context_add_struct() {
    let mut ctx = TypeContext::new();
    let struct_def = decy_hir::HirStruct::new(
        "Point".to_string(),
        vec![
            decy_hir::HirStructField::new("x".to_string(), HirType::Int),
            decy_hir::HirStructField::new("y".to_string(), HirType::Int),
        ],
    );
    ctx.add_struct(&struct_def);
    assert!(ctx.struct_has_default("Point"));
}

#[test]
fn test_type_context_get_field_type() {
    let mut ctx = TypeContext::new();
    let struct_def = decy_hir::HirStruct::new(
        "Point".to_string(),
        vec![
            decy_hir::HirStructField::new("x".to_string(), HirType::Int),
            decy_hir::HirStructField::new("y".to_string(), HirType::Float),
        ],
    );
    ctx.add_struct(&struct_def);
    ctx.add_variable("p".to_string(), HirType::Struct("Point".to_string()));

    let obj = HirExpression::Variable("p".to_string());
    assert_eq!(ctx.get_field_type(&obj, "x"), Some(HirType::Int));
    assert_eq!(ctx.get_field_type(&obj, "y"), Some(HirType::Float));
    assert_eq!(ctx.get_field_type(&obj, "z"), None);
}

// ============================================================================
// TypeContext infer_expression_type Tests
// ============================================================================

#[test]
fn test_infer_type_dereference_pointer() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let deref = HirExpression::Dereference(Box::new(HirExpression::Variable("ptr".to_string())));
    assert_eq!(ctx.infer_expression_type(&deref), Some(HirType::Int));
}

#[test]
fn test_infer_type_dereference_box() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("boxed".to_string(), HirType::Box(Box::new(HirType::Float)));
    let deref = HirExpression::Dereference(Box::new(HirExpression::Variable("boxed".to_string())));
    assert_eq!(ctx.infer_expression_type(&deref), Some(HirType::Float));
}

#[test]
fn test_infer_type_dereference_reference() {
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ref_var".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Char),
            mutable: false,
        },
    );
    let deref =
        HirExpression::Dereference(Box::new(HirExpression::Variable("ref_var".to_string())));
    assert_eq!(ctx.infer_expression_type(&deref), Some(HirType::Char));
}

#[test]
fn test_infer_type_dereference_vec() {
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "vec_var".to_string(),
        HirType::Vec(Box::new(HirType::Double)),
    );
    let deref =
        HirExpression::Dereference(Box::new(HirExpression::Variable("vec_var".to_string())));
    assert_eq!(ctx.infer_expression_type(&deref), Some(HirType::Double));
}

#[test]
fn test_infer_type_dereference_unknown() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let deref = HirExpression::Dereference(Box::new(HirExpression::Variable("x".to_string())));
    assert_eq!(ctx.infer_expression_type(&deref), None);
}

#[test]
fn test_infer_type_array_index_array() {
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "arr".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    let index = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
    };
    assert_eq!(ctx.infer_expression_type(&index), Some(HirType::Int));
}

#[test]
fn test_infer_type_array_index_pointer() {
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Float)),
    );
    let index = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("ptr".to_string())),
        index: Box::new(HirExpression::IntLiteral(1)),
    };
    assert_eq!(ctx.infer_expression_type(&index), Some(HirType::Float));
}

#[test]
fn test_infer_type_array_index_slice() {
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "slice".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Vec(Box::new(HirType::Char))),
            mutable: false,
        },
    );
    let index = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("slice".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
    };
    assert_eq!(ctx.infer_expression_type(&index), Some(HirType::Char));
}

#[test]
fn test_infer_type_array_index_ref_array() {
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ref_arr".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Array {
                element_type: Box::new(HirType::Double),
                size: Some(5),
            }),
            mutable: true,
        },
    );
    let index = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("ref_arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(2)),
    };
    assert_eq!(ctx.infer_expression_type(&index), Some(HirType::Double));
}

#[test]
fn test_infer_type_array_index_vec() {
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "vec".to_string(),
        HirType::Vec(Box::new(HirType::UnsignedInt)),
    );
    let index = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("vec".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
    };
    assert_eq!(
        ctx.infer_expression_type(&index),
        Some(HirType::UnsignedInt)
    );
}

#[test]
fn test_infer_type_array_index_unknown() {
    let ctx = TypeContext::new();
    let index = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("unknown".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
    };
    assert_eq!(ctx.infer_expression_type(&index), None);
}

#[test]
fn test_infer_type_field_access() {
    let mut ctx = TypeContext::new();
    let struct_def = decy_hir::HirStruct::new(
        "Point".to_string(),
        vec![
            decy_hir::HirStructField::new("x".to_string(), HirType::Int),
            decy_hir::HirStructField::new("y".to_string(), HirType::Float),
        ],
    );
    ctx.add_struct(&struct_def);
    ctx.add_variable("p".to_string(), HirType::Struct("Point".to_string()));

    let access = HirExpression::FieldAccess {
        object: Box::new(HirExpression::Variable("p".to_string())),
        field: "x".to_string(),
    };
    assert_eq!(ctx.infer_expression_type(&access), Some(HirType::Int));
}

#[test]
fn test_infer_type_pointer_field_access() {
    let mut ctx = TypeContext::new();
    let struct_def = decy_hir::HirStruct::new(
        "Node".to_string(),
        vec![decy_hir::HirStructField::new(
            "value".to_string(),
            HirType::Int,
        )],
    );
    ctx.add_struct(&struct_def);
    ctx.add_variable(
        "node_ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
    );

    let access = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("node_ptr".to_string())),
        field: "value".to_string(),
    };
    assert_eq!(ctx.infer_expression_type(&access), Some(HirType::Int));
}

#[test]
fn test_infer_type_box_field_access() {
    let mut ctx = TypeContext::new();
    let struct_def = decy_hir::HirStruct::new(
        "Data".to_string(),
        vec![decy_hir::HirStructField::new(
            "count".to_string(),
            HirType::UnsignedInt,
        )],
    );
    ctx.add_struct(&struct_def);
    ctx.add_variable(
        "boxed".to_string(),
        HirType::Box(Box::new(HirType::Struct("Data".to_string()))),
    );

    let access = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("boxed".to_string())),
        field: "count".to_string(),
    };
    assert_eq!(
        ctx.infer_expression_type(&access),
        Some(HirType::UnsignedInt)
    );
}

#[test]
fn test_infer_type_ref_field_access() {
    let mut ctx = TypeContext::new();
    let struct_def = decy_hir::HirStruct::new(
        "Info".to_string(),
        vec![decy_hir::HirStructField::new(
            "flag".to_string(),
            HirType::Char,
        )],
    );
    ctx.add_struct(&struct_def);
    ctx.add_variable(
        "ref_info".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Struct("Info".to_string())),
            mutable: false,
        },
    );

    let access = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("ref_info".to_string())),
        field: "flag".to_string(),
    };
    assert_eq!(ctx.infer_expression_type(&access), Some(HirType::Char));
}

#[test]
fn test_infer_type_binary_add_double() {
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::IntLiteral(1)),
        right: Box::new(HirExpression::FloatLiteral("3.14".to_string())),
    };
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Double));
}

#[test]
fn test_infer_type_binary_comparison() {
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::IntLiteral(1)),
        right: Box::new(HirExpression::IntLiteral(2)),
    };
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Int));
}

#[test]
fn test_infer_type_binary_logical_and() {
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalAnd,
        left: Box::new(HirExpression::IntLiteral(1)),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Int));
}

#[test]
fn test_infer_type_binary_bitwise() {
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::BitwiseXor,
        left: Box::new(HirExpression::IntLiteral(0xFF)),
        right: Box::new(HirExpression::IntLiteral(0x0F)),
    };
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Int));
}

#[test]
fn test_infer_type_binary_shift() {
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LeftShift,
        left: Box::new(HirExpression::IntLiteral(1)),
        right: Box::new(HirExpression::IntLiteral(4)),
    };
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Int));
}

#[test]
fn test_infer_type_literals() {
    let ctx = TypeContext::new();
    assert_eq!(
        ctx.infer_expression_type(&HirExpression::IntLiteral(42)),
        Some(HirType::Int)
    );
    assert_eq!(
        ctx.infer_expression_type(&HirExpression::FloatLiteral("3.14".to_string())),
        Some(HirType::Double)
    );
    assert_eq!(
        ctx.infer_expression_type(&HirExpression::CharLiteral(65)),
        Some(HirType::Char)
    );
}

// ============================================================================
// CodeGenerator escape_rust_keyword Tests
// ============================================================================

#[test]
fn test_escape_rust_keyword_type() {
    let escaped = super::escape_rust_keyword("type");
    assert_eq!(escaped, "r#type");
}

#[test]
fn test_escape_rust_keyword_fn() {
    let escaped = super::escape_rust_keyword("fn");
    assert_eq!(escaped, "r#fn");
}

#[test]
fn test_escape_rust_keyword_match() {
    let escaped = super::escape_rust_keyword("match");
    assert_eq!(escaped, "r#match");
}

#[test]
fn test_escape_rust_keyword_not_keyword() {
    let escaped = super::escape_rust_keyword("my_variable");
    assert_eq!(escaped, "my_variable");
}

#[test]
fn test_escape_rust_keyword_reserved() {
    let escaped = super::escape_rust_keyword("abstract");
    assert_eq!(escaped, "r#abstract");
}

// ============================================================================
// More Statement Type Tests
// ============================================================================

#[test]
fn test_generate_if_else_statement() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::IntLiteral(1),
        then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
        else_block: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
            0,
        )))]),
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("if"));
    assert!(result.contains("else"));
}

#[test]
fn test_generate_while_statement() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::While {
        condition: HirExpression::IntLiteral(1),
        body: vec![HirStatement::Break],
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("while") || result.contains("loop"));
}

#[test]
fn test_generate_expression_statement() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::Expression(HirExpression::FunctionCall {
        function: "print".to_string(),
        arguments: vec![],
    });
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("print"));
}

// ============================================================================
// Additional Expression Tests
// ============================================================================

#[test]
fn test_generate_string_literal() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::StringLiteral("hello".to_string());
    let result = gen.generate_expression(&expr);
    assert!(result.contains("hello"));
}

#[test]
fn test_generate_member_access() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::FieldAccess {
        object: Box::new(HirExpression::Variable("obj".to_string())),
        field: "field".to_string(),
    };
    let result = gen.generate_expression(&expr);
    assert!(result.contains("obj"));
    assert!(result.contains("field"));
}

#[test]
fn test_generate_pointer_field_access() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("ptr".to_string())),
        field: "value".to_string(),
    };
    let result = gen.generate_expression(&expr);
    assert!(result.contains("ptr") || result.contains("value"));
}

#[test]
fn test_generate_sizeof() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::Sizeof {
        type_name: "int".to_string(),
    };
    let result = gen.generate_expression(&expr);
    assert!(result.contains("size_of") || result.contains("mem::") || result.contains("4"));
}

#[test]
fn test_generate_ternary() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::IntLiteral(1)),
        then_expr: Box::new(HirExpression::IntLiteral(10)),
        else_expr: Box::new(HirExpression::IntLiteral(20)),
    };
    let result = gen.generate_expression(&expr);
    assert!(result.contains("if") || result.contains("10") || result.contains("20"));
}

// ============================================================================
// More Operator Tests
// ============================================================================

#[test]
fn test_generate_multiply() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::IntLiteral(3)),
        right: Box::new(HirExpression::IntLiteral(4)),
    };
    let result = gen.generate_expression(&expr);
    assert!(result.contains("*"));
}

#[test]
fn test_generate_divide() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Divide,
        left: Box::new(HirExpression::IntLiteral(10)),
        right: Box::new(HirExpression::IntLiteral(2)),
    };
    let result = gen.generate_expression(&expr);
    assert!(result.contains("/"));
}

#[test]
fn test_generate_modulo() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Modulo,
        left: Box::new(HirExpression::IntLiteral(7)),
        right: Box::new(HirExpression::IntLiteral(3)),
    };
    let result = gen.generate_expression(&expr);
    assert!(result.contains("%"));
}

#[test]
fn test_generate_bitwise_and() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::BitwiseAnd,
        left: Box::new(HirExpression::IntLiteral(0xFF)),
        right: Box::new(HirExpression::IntLiteral(0x0F)),
    };
    let result = gen.generate_expression(&expr);
    assert!(result.contains("&"));
}

#[test]
fn test_generate_bitwise_or() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::BitwiseOr,
        left: Box::new(HirExpression::IntLiteral(0x01)),
        right: Box::new(HirExpression::IntLiteral(0x02)),
    };
    let result = gen.generate_expression(&expr);
    assert!(result.contains("|"));
}

#[test]
fn test_generate_left_shift() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LeftShift,
        left: Box::new(HirExpression::IntLiteral(1)),
        right: Box::new(HirExpression::IntLiteral(4)),
    };
    let result = gen.generate_expression(&expr);
    assert!(result.contains("<<"));
}

#[test]
fn test_generate_right_shift() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::RightShift,
        left: Box::new(HirExpression::IntLiteral(16)),
        right: Box::new(HirExpression::IntLiteral(2)),
    };
    let result = gen.generate_expression(&expr);
    assert!(result.contains(">>"));
}

// ============================================================================
// More Type Mapping Tests
// ============================================================================

#[test]
fn test_map_type_signed_char() {
    assert_eq!(CodeGenerator::map_type(&HirType::SignedChar), "i8");
}

#[test]
fn test_map_type_nested_pointer() {
    let nested = HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Int))));
    let result = CodeGenerator::map_type(&nested);
    assert!(result.contains("*") || result.contains("mut"));
}

#[test]
fn test_map_type_nested_box() {
    let nested = HirType::Box(Box::new(HirType::Box(Box::new(HirType::Float))));
    let result = CodeGenerator::map_type(&nested);
    assert!(result.contains("Box"));
}

#[test]
fn test_map_type_option_of_pointer() {
    let opt_ptr = HirType::Option(Box::new(HirType::Pointer(Box::new(HirType::Char))));
    let result = CodeGenerator::map_type(&opt_ptr);
    assert!(result.contains("Option"));
}

#[test]
fn test_map_type_vec_of_structs() {
    let vec_struct = HirType::Vec(Box::new(HirType::Struct("Node".to_string())));
    let result = CodeGenerator::map_type(&vec_struct);
    assert!(result.contains("Vec") && result.contains("Node"));
}

// ============================================================================
// Additional Assignment Tests
// ============================================================================

#[test]
fn test_generate_assignment_with_expression() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::Assignment {
        target: "x".to_string(),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(5)),
        },
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("x") && result.contains("+") && result.contains("5"));
}

// ============================================================================
// For Loop Tests
// ============================================================================

#[test]
fn test_generate_for_loop() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::For {
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
        increment: vec![HirStatement::Expression(HirExpression::PreIncrement {
            operand: Box::new(HirExpression::Variable("i".to_string())),
        })],
        body: vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "print".to_string(),
            arguments: vec![],
        })],
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("for") || result.contains("while") || result.contains("loop"));
}

// ============================================================================
// Increment/Decrement Tests
// ============================================================================

#[test]
fn test_generate_pre_increment() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::PreIncrement {
        operand: Box::new(HirExpression::Variable("i".to_string())),
    };
    let result = gen.generate_expression(&expr);
    assert!(result.contains("i") || result.contains("+") || result.contains("1"));
}

#[test]
fn test_generate_post_increment() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Variable("j".to_string())),
    };
    let result = gen.generate_expression(&expr);
    assert!(result.contains("j") || result.contains("+") || result.contains("1"));
}

#[test]
fn test_generate_pre_decrement() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::PreDecrement {
        operand: Box::new(HirExpression::Variable("k".to_string())),
    };
    let result = gen.generate_expression(&expr);
    assert!(result.contains("k") || result.contains("-") || result.contains("1"));
}

#[test]
fn test_generate_post_decrement() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::PostDecrement {
        operand: Box::new(HirExpression::Variable("n".to_string())),
    };
    let result = gen.generate_expression(&expr);
    assert!(result.contains("n") || result.contains("-") || result.contains("1"));
}

// ============================================================================
// Malloc/Calloc Expression Tests
// ============================================================================

#[test]
fn test_generate_malloc_expression() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::Malloc {
        size: Box::new(HirExpression::IntLiteral(100)),
    };
    let result = gen.generate_expression(&expr);
    assert!(result.contains("Box") || result.contains("Vec") || result.contains("alloc"));
}

#[test]
fn test_generate_calloc_expression() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(10)),
        element_type: Box::new(HirType::Int),
    };
    let result = gen.generate_expression(&expr);
    assert!(result.contains("vec!") || result.contains("Vec") || result.contains("0"));
}

#[test]
fn test_generate_realloc_expression() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::Variable("ptr".to_string())),
        new_size: Box::new(HirExpression::IntLiteral(200)),
    };
    let result = gen.generate_expression(&expr);
    assert!(result.contains("ptr") || result.contains("resize") || result.contains("reserve"));
}

// ============================================================================
// Null Literal Tests
// ============================================================================

#[test]
fn test_generate_null_literal() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::NullLiteral;
    let result = gen.generate_expression(&expr);
    assert!(result.contains("None") || result.contains("null") || result.contains("ptr"));
}

#[test]
fn test_generate_is_not_null() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::IsNotNull(Box::new(HirExpression::Variable("ptr".to_string())));
    let result = gen.generate_expression(&expr);
    assert!(result.contains("is_some") || result.contains("ptr") || result.contains("!= None"));
}

// ============================================================================
// SliceIndex Expression Tests
// ============================================================================

#[test]
fn test_generate_slice_index() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::SliceIndex {
        slice: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(5)),
        element_type: HirType::Int,
    };
    let result = gen.generate_expression(&expr);
    assert!(result.contains("arr") || result.contains("5"));
}

// ============================================================================
// CompoundLiteral Tests
// ============================================================================

#[test]
fn test_generate_compound_literal() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Point".to_string()),
        initializers: vec![HirExpression::IntLiteral(10), HirExpression::IntLiteral(20)],
    };
    let result = gen.generate_expression(&expr);
    assert!(result.contains("Point") || result.contains("10") || result.contains("20"));
}

// ============================================================================
// Expression Generation with Target Type (DECY-COVERAGE-2)
// ============================================================================

#[test]
fn test_int_literal_to_option_target() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::IntLiteral(0);
    let target = HirType::Option(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("None"));
}

#[test]
fn test_int_literal_to_pointer_target() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::IntLiteral(0);
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("null_mut"));
}

#[test]
fn test_float_literal_with_float_target() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("3.14f".to_string());
    let target = HirType::Float;
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("f32"));
}

#[test]
fn test_float_literal_with_double_target() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("3.14".to_string());
    let target = HirType::Double;
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("f64"));
}

#[test]
fn test_float_literal_no_decimal() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("42".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains(".0") || result.contains("f64"));
}

#[test]
fn test_address_of_to_pointer_target() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::AddressOf(Box::new(HirExpression::Variable("x".to_string())));
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("&mut") && result.contains("as"));
}

#[test]
fn test_address_of_dereference() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::AddressOf(Box::new(HirExpression::Dereference(Box::new(
        HirExpression::Variable("ptr".to_string()),
    ))));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("&("));
}

#[test]
fn test_unary_address_of_to_pointer() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::AddressOf,
        operand: Box::new(HirExpression::Variable("val".to_string())),
    };
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("&mut"));
}

#[test]
fn test_logical_not_int_target() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("flag".to_string())),
    };
    let target = HirType::Int;
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("== 0") && result.contains("i32"));
}

#[test]
fn test_logical_not_boolean_expr() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    // Comparison is a boolean expression
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("!"));
}

#[test]
fn test_string_literal_to_pointer() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::StringLiteral("hello".to_string());
    let target = HirType::Pointer(Box::new(HirType::Char));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("b\"") && result.contains("as_ptr"));
}

#[test]
fn test_char_literal_null() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::CharLiteral(0i8);
    let result = gen.generate_expression(&expr);
    assert!(result.contains("0u8"));
}

#[test]
fn test_char_literal_non_printable() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::CharLiteral(7i8); // bell character
    let result = gen.generate_expression(&expr);
    assert!(result.contains("u8"));
}

// ============================================================================
// Variable Expression with Coercions (DECY-COVERAGE-3)
// ============================================================================

#[test]
fn test_variable_stderr() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::Variable("stderr".to_string());
    let result = gen.generate_expression(&expr);
    assert!(result.contains("std::io::stderr"));
}

#[test]
fn test_variable_stdin() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::Variable("stdin".to_string());
    let result = gen.generate_expression(&expr);
    assert!(result.contains("std::io::stdin"));
}

#[test]
fn test_variable_stdout() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::Variable("stdout".to_string());
    let result = gen.generate_expression(&expr);
    assert!(result.contains("std::io::stdout"));
}

#[test]
fn test_variable_errno() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::Variable("errno".to_string());
    let result = gen.generate_expression(&expr);
    assert!(result.contains("ERRNO"));
}

#[test]
fn test_variable_erange() {
    let gen = CodeGenerator::new();
    let expr = HirExpression::Variable("ERANGE".to_string());
    let result = gen.generate_expression(&expr);
    assert!(result.contains("34"));
}

#[test]
fn test_variable_vec_to_vec_target() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("arr".to_string());
    let target = HirType::Vec(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert_eq!(result, "arr");
}

#[test]
fn test_variable_box_to_pointer() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("node".to_string(), HirType::Box(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("node".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("Box::into_raw"));
}

#[test]
fn test_variable_reference_vec_to_pointer() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "slice".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Vec(Box::new(HirType::Int))),
            mutable: true,
        },
    );
    let expr = HirExpression::Variable("slice".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as_mut_ptr"));
}

#[test]
fn test_variable_immutable_slice_to_pointer() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "slice".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Vec(Box::new(HirType::Int))),
            mutable: false,
        },
    );
    let expr = HirExpression::Variable("slice".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as_ptr"));
}

#[test]
fn test_variable_mutable_ref_to_pointer() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "x".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        },
    );
    let expr = HirExpression::Variable("x".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as *mut _"));
}

#[test]
fn test_variable_immutable_ref_to_pointer() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "x".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
    );
    let expr = HirExpression::Variable("x".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as *const"));
}

#[test]
fn test_variable_vec_to_pointer() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("arr".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as_mut_ptr"));
}

#[test]
fn test_variable_array_to_pointer() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "arr".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    let expr = HirExpression::Variable("arr".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as_mut_ptr"));
}

#[test]
fn test_variable_array_to_void_pointer() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "arr".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    let expr = HirExpression::Variable("arr".to_string());
    let target = HirType::Pointer(Box::new(HirType::Void));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as *mut ()"));
}

#[test]
fn test_variable_pointer_to_pointer() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("ptr".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert_eq!(result, "ptr");
}

#[test]
fn test_variable_int_to_char() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Int);
    let expr = HirExpression::Variable("c".to_string());
    let target = HirType::Char;
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as u8"));
}

#[test]
fn test_variable_int_to_float() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("i".to_string(), HirType::Int);
    let expr = HirExpression::Variable("i".to_string());
    let target = HirType::Float;
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as f32"));
}

#[test]
fn test_variable_int_to_double() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("i".to_string(), HirType::Int);
    let expr = HirExpression::Variable("i".to_string());
    let target = HirType::Double;
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as f64"));
}

#[test]
fn test_variable_float_to_int() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("f".to_string(), HirType::Float);
    let expr = HirExpression::Variable("f".to_string());
    let target = HirType::Int;
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as i32"));
}

#[test]
fn test_variable_double_to_uint() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("d".to_string(), HirType::Double);
    let expr = HirExpression::Variable("d".to_string());
    let target = HirType::UnsignedInt;
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as u32"));
}

#[test]
fn test_variable_char_to_int() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Char);
    let expr = HirExpression::Variable("c".to_string());
    let target = HirType::Int;
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as i32"));
}

#[test]
fn test_variable_global_access() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_global("GLOBAL_VAR".to_string());
    let expr = HirExpression::Variable("GLOBAL_VAR".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("unsafe"));
}

// ============================================================================
// Binary Operations with Special Cases (DECY-COVERAGE-4)
// ============================================================================

#[test]
fn test_binary_assign_expression() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Assign,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::IntLiteral(42)),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("__assign_tmp"));
}

#[test]
fn test_binary_assign_global_array() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_global("GLOBAL_ARR".to_string());
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Assign,
        left: Box::new(HirExpression::ArrayIndex {
            array: Box::new(HirExpression::Variable("GLOBAL_ARR".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
        }),
        right: Box::new(HirExpression::IntLiteral(99)),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("unsafe"));
}

#[test]
fn test_option_equals_null() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("opt".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("opt".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("is_none"));
}

#[test]
fn test_option_not_equals_null() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("opt".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("opt".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("is_some"));
}

#[test]
fn test_null_equals_option() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("opt".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::NullLiteral),
        right: Box::new(HirExpression::Variable("opt".to_string())),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("is_none"));
}

#[test]
fn test_pointer_equals_zero() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("null_mut"));
}

#[test]
fn test_zero_equals_pointer() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::Variable("ptr".to_string())),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("null_mut"));
}

#[test]
fn test_vec_equals_null() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("false"));
}

#[test]
fn test_vec_not_equals_null() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("true"));
}

#[test]
fn test_box_equals_null() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("node".to_string(), HirType::Box(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("node".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("false"));
}

#[test]
fn test_box_not_equals_null() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("node".to_string(), HirType::Box(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("node".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("true"));
}

#[test]
fn test_strlen_equals_zero() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("s".to_string())],
        }),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("is_empty"));
}

#[test]
fn test_strlen_not_equals_zero() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("s".to_string())],
        }),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("!") && result.contains("is_empty"));
}

#[test]
fn test_zero_equals_strlen() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("s".to_string())],
        }),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("is_empty"));
}

// ============================================================================
// Macro Generation Tests (DECY-COVERAGE-MACRO)
// ============================================================================

#[test]
fn test_codegen_generate_macro_object_like() {
    use decy_hir::HirMacroDefinition;
    let gen = CodeGenerator::new();
    let macro_def = HirMacroDefinition::new_object_like("MAX".to_string(), "100".to_string());
    let result = gen.generate_macro(&macro_def);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("MAX"));
}

#[test]
fn test_codegen_generate_macro_function_like() {
    use decy_hir::HirMacroDefinition;
    let gen = CodeGenerator::new();
    let macro_def = HirMacroDefinition::new_function_like(
        "DOUBLE".to_string(),
        vec!["x".to_string()],
        "((x) * 2)".to_string(),
    );
    let result = gen.generate_macro(&macro_def);
    assert!(result.is_ok());
}

#[test]
fn test_codegen_generate_macro_two_params() {
    use decy_hir::HirMacroDefinition;
    let gen = CodeGenerator::new();
    let macro_def = HirMacroDefinition::new_function_like(
        "ADD".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "((a) + (b))".to_string(),
    );
    let result = gen.generate_macro(&macro_def);
    assert!(result.is_ok());
}

// ============================================================================
// Function Generation Tests (DECY-COVERAGE-FUNC)
// ============================================================================

#[test]
fn test_generate_function_void_return() {
    let gen = CodeGenerator::new();
    let func = HirFunction::new_with_body("empty_func".to_string(), HirType::Void, vec![], vec![]);
    let result = gen.generate_function(&func);
    assert!(result.contains("fn empty_func"));
}

#[test]
fn test_generate_function_with_return_value() {
    let gen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "get_value".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(42)))],
    );
    let result = gen.generate_function(&func);
    assert!(result.contains("-> i32"));
    assert!(result.contains("42"));
}

#[test]
fn test_generate_function_with_pointer_param() {
    let gen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![],
    );
    let result = gen.generate_function(&func);
    assert!(result.contains("data"));
}

#[test]
fn test_generate_function_with_multiple_params() {
    let gen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "calc".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
            HirParameter::new("c".to_string(), HirType::Int),
        ],
        vec![HirStatement::Return(Some(HirExpression::Variable(
            "a".to_string(),
        )))],
    );
    let result = gen.generate_function(&func);
    assert!(result.contains("a: i32"));
    assert!(result.contains("b: i32"));
    assert!(result.contains("c: i32"));
}

#[test]
fn test_generate_function_with_array_param() {
    let gen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "process_arr".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "arr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)), // Arrays decay to pointers
        )],
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
    );
    let result = gen.generate_function(&func);
    assert!(result.contains("arr"));
}

// ============================================================================
// Complex Statement Tests (DECY-COVERAGE-STMT)
// ============================================================================

#[test]
fn test_generate_switch_statement() {
    use decy_hir::SwitchCase;
    let gen = CodeGenerator::new();
    let switch_stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![
            SwitchCase {
                value: Some(HirExpression::IntLiteral(1)),
                body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(10)))],
            },
            SwitchCase {
                value: Some(HirExpression::IntLiteral(2)),
                body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(20)))],
            },
        ],
        default_case: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
            0,
        )))]),
    };
    let result = gen.generate_statement(&switch_stmt);
    assert!(result.contains("match"));
}

#[test]
fn test_generate_switch_without_default() {
    use decy_hir::SwitchCase;
    let gen = CodeGenerator::new();
    let switch_stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![SwitchCase {
            value: Some(HirExpression::IntLiteral(1)),
            body: vec![HirStatement::Break],
        }],
        default_case: None,
    };
    let result = gen.generate_statement(&switch_stmt);
    assert!(result.contains("match") || result.contains("if"));
}

#[test]
fn test_generate_while_with_complex_condition() {
    let gen = CodeGenerator::new();
    let while_stmt = HirStatement::While {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::LogicalAnd,
            left: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::IntLiteral(0)),
            }),
            right: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("y".to_string())),
                right: Box::new(HirExpression::IntLiteral(100)),
            }),
        },
        body: vec![HirStatement::Break],
    };
    let result = gen.generate_statement(&while_stmt);
    assert!(result.contains("while"));
}

#[test]
fn test_generate_nested_for_loop() {
    let gen = CodeGenerator::new();
    let inner_loop = HirStatement::For {
        init: vec![HirStatement::VariableDeclaration {
            name: "j".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        }],
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("j".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        },
        increment: vec![],
        body: vec![HirStatement::Break],
    };
    let outer_loop = HirStatement::For {
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
        increment: vec![],
        body: vec![inner_loop],
    };
    let result = gen.generate_statement(&outer_loop);
    assert!(result.contains("for") || result.contains("while"));
}

// ============================================================================
// Type Inference Tests (DECY-COVERAGE-INFER)
// ============================================================================

#[test]
fn test_infer_expression_type_binary_comparison() {
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LessThan,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::IntLiteral(5)),
    };
    let result = ctx.infer_expression_type(&expr);
    assert!(result.is_some());
}

#[test]
fn test_infer_expression_type_logical() {
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalAnd,
        left: Box::new(HirExpression::IntLiteral(1)),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = ctx.infer_expression_type(&expr);
    assert!(result.is_some());
}

#[test]
fn test_infer_expression_type_ternary() {
    let ctx = TypeContext::new();
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::IntLiteral(1)),
        then_expr: Box::new(HirExpression::IntLiteral(10)),
        else_expr: Box::new(HirExpression::IntLiteral(20)),
    };
    // Ternary is not yet handled in type inference
    let result = ctx.infer_expression_type(&expr);
    assert!(result.is_none());
}

#[test]
fn test_infer_expression_type_sizeof() {
    let ctx = TypeContext::new();
    let expr = HirExpression::Sizeof {
        type_name: "int".to_string(),
    };
    // Sizeof is not yet handled in type inference
    let result = ctx.infer_expression_type(&expr);
    assert!(result.is_none());
}

#[test]
fn test_infer_expression_type_string_literal() {
    let ctx = TypeContext::new();
    let expr = HirExpression::StringLiteral("hello".to_string());
    // StringLiteral is not yet handled in type inference
    let result = ctx.infer_expression_type(&expr);
    assert!(result.is_none());
}

// ============================================================================
// Format String Conversion Tests (DECY-COVERAGE-FORMAT)
// ============================================================================

#[test]
fn test_format_specifier_lu() {
    let gen = CodeGenerator::new();
    let call = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("Value: %lu\n".to_string()),
            HirExpression::Variable("x".to_string()),
        ],
    };
    let ctx = TypeContext::new();
    let result = gen.generate_expression_with_target_type(&call, &ctx, None);
    assert!(!result.contains("%lu") || result.contains("print!"));
}

#[test]
fn test_format_specifier_ld() {
    let gen = CodeGenerator::new();
    let call = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("Value: %ld\n".to_string()),
            HirExpression::Variable("x".to_string()),
        ],
    };
    let ctx = TypeContext::new();
    let result = gen.generate_expression_with_target_type(&call, &ctx, None);
    assert!(!result.contains("%ld") || result.contains("print!"));
}

#[test]
fn test_format_specifier_x() {
    let gen = CodeGenerator::new();
    let call = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("Hex: %x\n".to_string()),
            HirExpression::Variable("x".to_string()),
        ],
    };
    let ctx = TypeContext::new();
    let result = gen.generate_expression_with_target_type(&call, &ctx, None);
    assert!(!result.contains("%x") || result.contains("print!") || result.contains(":x"));
}

#[test]
fn test_format_specifier_p() {
    let gen = CodeGenerator::new();
    let call = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("Ptr: %p\n".to_string()),
            HirExpression::Variable("ptr".to_string()),
        ],
    };
    let ctx = TypeContext::new();
    let result = gen.generate_expression_with_target_type(&call, &ctx, None);
    assert!(result.contains("print!") || result.contains("ptr"));
}

// ============================================================================
// Expression Target Type Tests (DECY-COVERAGE-EXPR-TARGET)
// ============================================================================

#[test]
fn test_expr_float_literal_double_target() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("3.14".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Double));
    assert!(result.contains("f64"));
}

#[test]
fn test_expr_float_literal_float_target() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("3.14f".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Float));
    assert!(result.contains("f32"));
}

#[test]
fn test_expr_zero_to_option_type() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::IntLiteral(0);
    let option_type = HirType::Option(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&option_type));
    assert!(result.contains("None"));
}

#[test]
fn test_expr_zero_to_pointer_type() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::IntLiteral(0);
    let ptr_type = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&ptr_type));
    assert!(result.contains("null_mut"));
}

#[test]
fn test_expr_address_of_to_pointer() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::AddressOf(Box::new(HirExpression::Variable("y".to_string())));
    let ptr_type = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&ptr_type));
    assert!(result.contains("&mut") || result.contains("y"));
}

#[test]
fn test_expr_xor_operator() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::BitwiseXor,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("^"));
}

#[test]
fn test_expr_less_equal() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LessEqual,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("<="));
}

#[test]
fn test_expr_greater_equal() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::GreaterEqual,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains(">="));
}

#[test]
fn test_expr_logical_or() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalOr,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("||"));
}

// ============================================================================
// Special Variable Names Tests (DECY-COVERAGE-VARS)
// ============================================================================

#[test]
fn test_var_stderr() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Variable("stderr".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("std::io::stderr()"));
}

#[test]
fn test_var_stdin() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Variable("stdin".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("std::io::stdin()"));
}

#[test]
fn test_var_stdout() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Variable("stdout".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("std::io::stdout()"));
}

#[test]
fn test_var_errno() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Variable("errno".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("ERRNO"));
}

#[test]
fn test_var_erange() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Variable("ERANGE".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("34i32"));
}

#[test]
fn test_var_einval() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Variable("EINVAL".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("22i32"));
}

#[test]
fn test_var_enoent() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Variable("ENOENT".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("2i32"));
}

#[test]
fn test_var_eacces() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::Variable("EACCES".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("13i32"));
}

// ============================================================================
// Float Literal Tests (DECY-COVERAGE-FLOAT)
// ============================================================================

#[test]
fn test_float_literal_f_suffix() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("3.14f".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("3.14f64"));
}

#[test]
fn test_float_literal_uppercase_f_suffix() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("2.71F".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("2.71f64"));
}

#[test]
fn test_float_literal_target_float() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("1.5".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Float));
    assert!(result.contains("f32"));
}

#[test]
fn test_float_literal_target_double() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("1.5".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Double));
    assert!(result.contains("f64"));
}

#[test]
fn test_float_literal_with_exponent() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("1e10".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("1e10f64"));
}

#[test]
fn test_float_literal_integer_form() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    // When float literal is just an integer without decimal point
    let expr = HirExpression::FloatLiteral("42".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("42.0f64"));
}

// ============================================================================
// Address Of Expression Tests (DECY-COVERAGE-ADDR)
// ============================================================================

#[test]
fn test_addressof_with_pointer_target() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::AddressOf(Box::new(HirExpression::Variable("x".to_string())));
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("&mut x as"));
}

#[test]
fn test_addressof_with_dereference() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::AddressOf(Box::new(HirExpression::Dereference(Box::new(
        HirExpression::Variable("p".to_string()),
    ))));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("&("));
}

#[test]
fn test_unaryop_addressof_with_pointer_target() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::AddressOf,
        operand: Box::new(HirExpression::Variable("y".to_string())),
    };
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("&mut y as"));
}

// ============================================================================
// Logical Not Expression Tests (DECY-COVERAGE-NOT)
// ============================================================================

#[test]
fn test_logical_not_bool_to_int() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    // !bool_expr with int target
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("as i32"));
}

#[test]
fn test_logical_not_int_to_int() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    // !int_expr with int target
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("== 0"));
}

#[test]
fn test_logical_not_binary_op_parens() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
    };
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("!"));
}

// ============================================================================
// String Literal to Pointer Tests (DECY-COVERAGE-STR)
// ============================================================================

#[test]
fn test_string_literal_to_char_pointer() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::StringLiteral("hello".to_string());
    let target = HirType::Pointer(Box::new(HirType::Char));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("b\"hello\\0\""));
    assert!(result.contains("as_ptr"));
}

#[test]
fn test_string_literal_with_escape() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::StringLiteral("hello\\nworld".to_string());
    let target = HirType::Pointer(Box::new(HirType::Char));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as_ptr"));
}

// ============================================================================
// Char Literal Tests (DECY-COVERAGE-CHAR)
// ============================================================================

#[test]
fn test_char_literal_nul_byte() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::CharLiteral(0);
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "0u8");
}

#[test]
fn test_char_literal_ascii_printable() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::CharLiteral(b'A' as i8);
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("b'A'"));
}

#[test]
fn test_char_literal_space_char() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::CharLiteral(b' ' as i8);
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("b' '"));
}

#[test]
fn test_char_literal_nonprintable_char() {
    let gen = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::CharLiteral(7); // Bell character
    let result = gen.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("7u8"));
}

// ============================================================================
// Type Coercion Tests (DECY-COVERAGE-COERCE)
// ============================================================================

#[test]
fn test_int_to_float_coercion() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("n".to_string(), HirType::Int);
    let expr = HirExpression::Variable("n".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Float));
    assert!(result.contains("as f32"));
}

#[test]
fn test_int_to_double_coercion() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("n".to_string(), HirType::Int);
    let expr = HirExpression::Variable("n".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Double));
    assert!(result.contains("as f64"));
}

#[test]
fn test_float_to_int_coercion() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("f".to_string(), HirType::Float);
    let expr = HirExpression::Variable("f".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("as i32"));
}

#[test]
fn test_double_to_int_coercion() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("d".to_string(), HirType::Double);
    let expr = HirExpression::Variable("d".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("as i32"));
}

#[test]
fn test_float_to_unsigned_int_coercion() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("f".to_string(), HirType::Float);
    let expr = HirExpression::Variable("f".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::UnsignedInt));
    assert!(result.contains("as u32"));
}

#[test]
fn test_int_to_char_coercion() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Int);
    let expr = HirExpression::Variable("c".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Char));
    assert!(result.contains("as u8"));
}

#[test]
fn test_unsigned_int_to_float_coercion() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("u".to_string(), HirType::UnsignedInt);
    let expr = HirExpression::Variable("u".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Float));
    assert!(result.contains("as f32"));
}

// ============================================================================
// Box to Raw Pointer Tests (DECY-COVERAGE-BOX)
// ============================================================================

#[test]
fn test_box_to_raw_pointer() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("b".to_string(), HirType::Box(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("b".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("Box::into_raw"));
}

// ============================================================================
// Reference to Pointer Coercion Tests (DECY-COVERAGE-REF)
// ============================================================================

#[test]
fn test_mutable_ref_to_pointer() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "r".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        },
    );
    let expr = HirExpression::Variable("r".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as *mut"));
}

#[test]
fn test_immutable_ref_to_pointer() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "r".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
    );
    let expr = HirExpression::Variable("r".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as *const"));
}

// ============================================================================
// Vec to Pointer Tests (DECY-COVERAGE-VEC)
// ============================================================================

#[test]
fn test_vec_to_pointer() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("v".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("v".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as_mut_ptr"));
}

#[test]
fn test_vec_to_vec_no_conversion() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("v".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("v".to_string());
    let target = HirType::Vec(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert_eq!(result, "v");
}

// ============================================================================
// Array to Pointer Tests (DECY-COVERAGE-ARR)
// ============================================================================

#[test]
fn test_array_to_pointer() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "arr".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    let expr = HirExpression::Variable("arr".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as_mut_ptr"));
}

#[test]
fn test_array_to_void_pointer() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "arr".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    let expr = HirExpression::Variable("arr".to_string());
    let target = HirType::Pointer(Box::new(HirType::Void));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as *mut ()"));
}

// ============================================================================
// Slice Reference Tests (DECY-COVERAGE-SLICE)
// ============================================================================

#[test]
fn test_mutable_slice_to_pointer() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "s".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Vec(Box::new(HirType::Int))),
            mutable: true,
        },
    );
    let expr = HirExpression::Variable("s".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as_mut_ptr"));
}

#[test]
fn test_immutable_slice_to_pointer() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "s".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Vec(Box::new(HirType::Int))),
            mutable: false,
        },
    );
    let expr = HirExpression::Variable("s".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as_ptr"));
}

// ============================================================================
// Global Variable Tests (DECY-COVERAGE-GLOBAL)
// ============================================================================

#[test]
fn test_global_int_to_float() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("g".to_string(), HirType::Int);
    ctx.add_global("g".to_string());
    let expr = HirExpression::Variable("g".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Float));
    assert!(result.contains("unsafe"));
    assert!(result.contains("as f32"));
}

#[test]
fn test_global_int_to_double() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("g".to_string(), HirType::Int);
    ctx.add_global("g".to_string());
    let expr = HirExpression::Variable("g".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Double));
    assert!(result.contains("unsafe"));
    assert!(result.contains("as f64"));
}

#[test]
fn test_global_float_to_int() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("g".to_string(), HirType::Float);
    ctx.add_global("g".to_string());
    let expr = HirExpression::Variable("g".to_string());
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("unsafe"));
    assert!(result.contains("as i32"));
}

// ============================================================================
// Pointer to Pointer Tests (DECY-COVERAGE-PTR)
// ============================================================================

#[test]
fn test_pointer_to_pointer_no_conversion() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("p".to_string());
    let target = HirType::Pointer(Box::new(HirType::Int));
    let result = gen.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert_eq!(result, "p");
}

// ============================================================================
// VLA (Variable Length Array) Tests (DECY-COVERAGE-VLA)
// ============================================================================

#[test]
fn test_vla_to_vec_int() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None, // VLA indicator
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("vec!"));
    assert!(result.contains("0i32"));
}

#[test]
fn test_vla_to_vec_unsigned() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::UnsignedInt),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("0u32"));
}

#[test]
fn test_vla_to_vec_float() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Float),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("0.0f32"));
}

#[test]
fn test_vla_to_vec_double() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Double),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("0.0f64"));
}

#[test]
fn test_vla_to_vec_char() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Char),
            size: None,
        },
        initializer: Some(HirExpression::Variable("len".to_string())),
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("0u8"));
}

#[test]
fn test_vla_to_vec_signed_char() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::SignedChar),
            size: None,
        },
        initializer: Some(HirExpression::Variable("len".to_string())),
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("0i8"));
}

// ============================================================================
// Malloc/Calloc Statement Tests (DECY-COVERAGE-MALLOC)
// ============================================================================

#[test]
fn test_malloc_to_box() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "node".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::Sizeof {
                type_name: "Node".to_string(),
            }),
        }),
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("Box"));
}

#[test]
fn test_malloc_array_to_vec() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::Variable("n".to_string())),
                right: Box::new(HirExpression::Sizeof {
                    type_name: "int".to_string(),
                }),
            }),
        }),
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("Vec"));
}

// ============================================================================
// String Literal Initialization Tests (DECY-COVERAGE-STR-INIT)
// ============================================================================

#[test]
fn test_char_pointer_string_init() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "msg".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Char)),
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("&str"));
    assert!(result.contains("\"hello\""));
}

#[test]
fn test_char_pointer_array_string_init() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "msgs".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Pointer(Box::new(HirType::Char))),
            size: Some(2),
        },
        initializer: Some(HirExpression::CompoundLiteral {
            literal_type: HirType::Void,
            initializers: vec![
                HirExpression::StringLiteral("foo".to_string()),
                HirExpression::StringLiteral("bar".to_string()),
            ],
        }),
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("[&str;"));
}

// ============================================================================
// Local Variable Shadowing Tests (DECY-COVERAGE-SHADOW)
// ============================================================================

#[test]
fn test_local_shadows_global() {
    let gen = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_global("counter".to_string());
    ctx.add_variable("counter".to_string(), HirType::Int);

    let stmt = HirStatement::VariableDeclaration {
        name: "counter".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    let result = gen.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("counter_local"));
}

// ============================================================================
// Escaped Keyword Tests (DECY-COVERAGE-KEYWORD)
// ============================================================================

#[test]
fn test_reserved_keyword_type_var() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "type".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("r#type"));
}

#[test]
fn test_reserved_keyword_match_var() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "match".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("r#match"));
}

// ============================================================================
// Return Statement Tests (DECY-COVERAGE-RETURN)
// ============================================================================

#[test]
fn test_return_void() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::Return(None);
    let result = gen.generate_statement(&stmt);
    assert_eq!(result, "return;");
}

#[test]
fn test_return_value() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::Return(Some(HirExpression::IntLiteral(42)));
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("42"));
}

// ============================================================================
// DerefAssignment Statement Tests (DECY-COVERAGE-DEREF-ASSIGN)
// ============================================================================

#[test]
fn test_deref_assignment() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("ptr".to_string()),
        value: HirExpression::IntLiteral(42),
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("42"));
}

// ============================================================================
// ArrayIndexAssignment Statement Tests (DECY-COVERAGE-ARR-ASSIGN)
// ============================================================================

#[test]
fn test_array_index_assignment() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
        value: HirExpression::IntLiteral(100),
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("arr"));
    assert!(result.contains("100"));
}

// ============================================================================
// FieldAssignment Statement Tests (DECY-COVERAGE-FIELD-ASSIGN)
// ============================================================================

#[test]
fn test_field_assignment() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::FieldAssignment {
        object: HirExpression::Variable("point".to_string()),
        field: "x".to_string(),
        value: HirExpression::IntLiteral(10),
    };
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("point"));
    assert!(result.contains("x"));
    assert!(result.contains("10"));
}

// ============================================================================
// Free Statement Tests (DECY-COVERAGE-FREE)
// ============================================================================

#[test]
fn test_free_statement() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::Free {
        pointer: HirExpression::Variable("ptr".to_string()),
    };
    let result = gen.generate_statement(&stmt);
    // Free generates a comment about RAII deallocation
    assert!(result.contains("RAII") || result.contains("deallocated"));
}

// ============================================================================
// Expression Statement Tests (DECY-COVERAGE-EXPR-STMT)
// ============================================================================

#[test]
fn test_expression_statement() {
    let gen = CodeGenerator::new();
    let stmt = HirStatement::Expression(HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![HirExpression::StringLiteral("Hello".to_string())],
    });
    let result = gen.generate_statement(&stmt);
    assert!(result.contains("printf") || result.contains("print"));
}

// ============================================================================
