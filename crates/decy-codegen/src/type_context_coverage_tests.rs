//! Additional coverage tests for TypeContext methods, CodeGenerator helpers,
//! and untested branches in helper functions.

use super::*;
use decy_hir::{HirStruct, HirStructField};

// ============================================================================
// struct_has_default: large array field and unknown struct
// ============================================================================

#[test]
fn struct_has_default_with_large_array_returns_false() {
    let mut ctx = TypeContext::new();
    let struct_def = HirStruct::new(
        "BigArray".to_string(),
        vec![HirStructField::new(
            "data".to_string(),
            HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(64), // > 32
            },
        )],
    );
    ctx.add_struct(&struct_def);
    assert!(!ctx.struct_has_default("BigArray"));
}

#[test]
fn struct_has_default_with_small_array_returns_true() {
    let mut ctx = TypeContext::new();
    let struct_def = HirStruct::new(
        "SmallArray".to_string(),
        vec![HirStructField::new(
            "data".to_string(),
            HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(16), // <= 32
            },
        )],
    );
    ctx.add_struct(&struct_def);
    assert!(ctx.struct_has_default("SmallArray"));
}

#[test]
fn struct_has_default_exactly_32_returns_true() {
    let mut ctx = TypeContext::new();
    let struct_def = HirStruct::new(
        "Exact32".to_string(),
        vec![HirStructField::new(
            "data".to_string(),
            HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(32), // exactly 32 — should pass
            },
        )],
    );
    ctx.add_struct(&struct_def);
    assert!(ctx.struct_has_default("Exact32"));
}

#[test]
fn struct_has_default_exactly_33_returns_false() {
    let mut ctx = TypeContext::new();
    let struct_def = HirStruct::new(
        "Over32".to_string(),
        vec![HirStructField::new(
            "data".to_string(),
            HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(33), // 33 > 32
            },
        )],
    );
    ctx.add_struct(&struct_def);
    assert!(!ctx.struct_has_default("Over32"));
}

#[test]
fn struct_has_default_unknown_struct_returns_false() {
    let ctx = TypeContext::new();
    assert!(!ctx.struct_has_default("NonExistent"));
}

#[test]
fn struct_has_default_no_size_array_returns_true() {
    let mut ctx = TypeContext::new();
    let struct_def = HirStruct::new(
        "FlexArray".to_string(),
        vec![HirStructField::new(
            "data".to_string(),
            HirType::Array {
                element_type: Box::new(HirType::Int),
                size: None, // No size — not > 32
            },
        )],
    );
    ctx.add_struct(&struct_def);
    assert!(ctx.struct_has_default("FlexArray"));
}

// ============================================================================
// get_field_type: through Pointer, Box, Reference types
// ============================================================================

#[test]
fn get_field_type_through_pointer_to_struct() {
    let mut ctx = TypeContext::new();
    let struct_def = HirStruct::new(
        "Node".to_string(),
        vec![
            HirStructField::new("value".to_string(), HirType::Int),
            HirStructField::new("next".to_string(), HirType::Pointer(Box::new(HirType::Void))),
        ],
    );
    ctx.add_struct(&struct_def);
    ctx.add_variable(
        "node_ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
    );

    let obj = HirExpression::Variable("node_ptr".to_string());
    assert_eq!(ctx.get_field_type(&obj, "value"), Some(HirType::Int));
}

#[test]
fn get_field_type_through_box_to_struct() {
    let mut ctx = TypeContext::new();
    let struct_def = HirStruct::new(
        "Boxed".to_string(),
        vec![HirStructField::new("data".to_string(), HirType::Double)],
    );
    ctx.add_struct(&struct_def);
    ctx.add_variable(
        "boxed".to_string(),
        HirType::Box(Box::new(HirType::Struct("Boxed".to_string()))),
    );

    let obj = HirExpression::Variable("boxed".to_string());
    assert_eq!(ctx.get_field_type(&obj, "data"), Some(HirType::Double));
}

#[test]
fn get_field_type_through_reference_to_struct() {
    let mut ctx = TypeContext::new();
    let struct_def = HirStruct::new(
        "Ref".to_string(),
        vec![HirStructField::new("count".to_string(), HirType::Int)],
    );
    ctx.add_struct(&struct_def);
    ctx.add_variable(
        "ref_var".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Struct("Ref".to_string())),
            mutable: false,
        },
    );

    let obj = HirExpression::Variable("ref_var".to_string());
    assert_eq!(ctx.get_field_type(&obj, "count"), Some(HirType::Int));
}

#[test]
fn get_field_type_pointer_to_non_struct_returns_none() {
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "int_ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );

    let obj = HirExpression::Variable("int_ptr".to_string());
    assert_eq!(ctx.get_field_type(&obj, "field"), None);
}

#[test]
fn get_field_type_box_non_struct_returns_none() {
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "boxed_int".to_string(),
        HirType::Box(Box::new(HirType::Int)),
    );

    let obj = HirExpression::Variable("boxed_int".to_string());
    assert_eq!(ctx.get_field_type(&obj, "field"), None);
}

#[test]
fn get_field_type_reference_non_struct_returns_none() {
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ref_int".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
    );

    let obj = HirExpression::Variable("ref_int".to_string());
    assert_eq!(ctx.get_field_type(&obj, "field"), None);
}

#[test]
fn get_field_type_non_variable_expression_returns_none() {
    let ctx = TypeContext::new();
    let expr = HirExpression::IntLiteral(42);
    assert_eq!(ctx.get_field_type(&expr, "field"), None);
}

#[test]
fn get_field_type_non_struct_type_returns_none() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);

    let obj = HirExpression::Variable("x".to_string());
    assert_eq!(ctx.get_field_type(&obj, "field"), None);
}

// ============================================================================
// is_boolean_expression: LogicalNot and non-LogicalNot UnaryOp
// ============================================================================

#[test]
fn is_boolean_expression_logical_not() {
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    assert!(CodeGenerator::is_boolean_expression(&expr));
}

#[test]
fn is_boolean_expression_minus_not_boolean() {
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::Minus,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    assert!(!CodeGenerator::is_boolean_expression(&expr));
}

#[test]
fn is_boolean_expression_all_comparison_ops() {
    let ops = [
        BinaryOperator::NotEqual,
        BinaryOperator::LessThan,
        BinaryOperator::GreaterThan,
        BinaryOperator::LessEqual,
        BinaryOperator::GreaterEqual,
        BinaryOperator::LogicalOr,
    ];

    for op in ops {
        let expr = HirExpression::BinaryOp {
            op,
            left: Box::new(HirExpression::IntLiteral(1)),
            right: Box::new(HirExpression::IntLiteral(2)),
        };
        assert!(
            CodeGenerator::is_boolean_expression(&expr),
            "Expected {:?} to be boolean",
            op
        );
    }
}

#[test]
fn is_boolean_expression_arithmetic_not_boolean() {
    let ops = [
        BinaryOperator::Add,
        BinaryOperator::Subtract,
        BinaryOperator::Multiply,
        BinaryOperator::Divide,
    ];

    for op in ops {
        let expr = HirExpression::BinaryOp {
            op,
            left: Box::new(HirExpression::IntLiteral(1)),
            right: Box::new(HirExpression::IntLiteral(2)),
        };
        assert!(
            !CodeGenerator::is_boolean_expression(&expr),
            "Expected {:?} to not be boolean",
            op
        );
    }
}

// ============================================================================
// is_malloc_expression: Calloc, Malloc variant, Cast wrapping
// ============================================================================

#[test]
fn is_malloc_expression_malloc_variant() {
    let expr = HirExpression::Malloc {
        size: Box::new(HirExpression::IntLiteral(100)),
    };
    assert!(CodeGenerator::is_malloc_expression(&expr));
}

#[test]
fn is_malloc_expression_calloc_variant() {
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(10)),
        element_type: Box::new(HirType::Int),
    };
    assert!(CodeGenerator::is_malloc_expression(&expr));
}

#[test]
fn is_malloc_expression_calloc_function_call() {
    let expr = HirExpression::FunctionCall {
        function: "calloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(10), HirExpression::IntLiteral(4)],
    };
    assert!(CodeGenerator::is_malloc_expression(&expr));
}

#[test]
fn is_malloc_expression_cast_wrapping_malloc() {
    // (int*)malloc(sizeof(int))
    let expr = HirExpression::Cast {
        expr: Box::new(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::IntLiteral(4)],
        }),
        target_type: HirType::Pointer(Box::new(HirType::Int)),
    };
    assert!(CodeGenerator::is_malloc_expression(&expr));
}

#[test]
fn is_malloc_expression_non_malloc_returns_false() {
    let expr = HirExpression::Variable("ptr".to_string());
    assert!(!CodeGenerator::is_malloc_expression(&expr));
}

// ============================================================================
// get_string_deref_var: string dereference detection
// ============================================================================

#[test]
fn get_string_deref_var_direct_deref_string_ref() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("str".to_string(), HirType::StringReference);

    let expr =
        HirExpression::Dereference(Box::new(HirExpression::Variable("str".to_string())));
    assert_eq!(
        CodeGenerator::get_string_deref_var(&expr, &ctx),
        Some("str".to_string())
    );
}

#[test]
fn get_string_deref_var_direct_deref_string_literal() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("s".to_string(), HirType::StringLiteral);

    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("s".to_string())));
    assert_eq!(
        CodeGenerator::get_string_deref_var(&expr, &ctx),
        Some("s".to_string())
    );
}

#[test]
fn get_string_deref_var_deref_non_string_returns_none() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));

    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("p".to_string())));
    assert_eq!(CodeGenerator::get_string_deref_var(&expr, &ctx), None);
}

#[test]
fn get_string_deref_var_comparison_with_zero() {
    // *str != 0
    let mut ctx = TypeContext::new();
    ctx.add_variable("str".to_string(), HirType::StringReference);

    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("str".to_string()),
        ))),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    assert_eq!(
        CodeGenerator::get_string_deref_var(&expr, &ctx),
        Some("str".to_string())
    );
}

#[test]
fn get_string_deref_var_zero_on_left_side() {
    // 0 == *str
    let mut ctx = TypeContext::new();
    ctx.add_variable("str".to_string(), HirType::StringReference);

    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("str".to_string()),
        ))),
    };
    assert_eq!(
        CodeGenerator::get_string_deref_var(&expr, &ctx),
        Some("str".to_string())
    );
}

#[test]
fn get_string_deref_var_non_zero_comparison_returns_none() {
    // *str != 1 (not checking for null terminator)
    let mut ctx = TypeContext::new();
    ctx.add_variable("str".to_string(), HirType::StringReference);

    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("str".to_string()),
        ))),
        right: Box::new(HirExpression::IntLiteral(1)),
    };
    assert_eq!(CodeGenerator::get_string_deref_var(&expr, &ctx), None);
}

#[test]
fn get_string_deref_var_non_comparison_op_returns_none() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("str".to_string(), HirType::StringReference);

    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("str".to_string()),
        ))),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    assert_eq!(CodeGenerator::get_string_deref_var(&expr, &ctx), None);
}

#[test]
fn get_string_deref_var_non_deref_expression_returns_none() {
    let ctx = TypeContext::new();
    let expr = HirExpression::IntLiteral(42);
    assert_eq!(CodeGenerator::get_string_deref_var(&expr, &ctx), None);
}

#[test]
fn get_string_deref_var_deref_non_variable_returns_none() {
    let ctx = TypeContext::new();
    // *42 — dereference of a non-variable
    let expr =
        HirExpression::Dereference(Box::new(HirExpression::IntLiteral(42)));
    assert_eq!(CodeGenerator::get_string_deref_var(&expr, &ctx), None);
}

#[test]
fn get_string_deref_var_deref_unknown_variable_returns_none() {
    let ctx = TypeContext::new();
    // *unknown — variable not in context
    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable(
        "unknown".to_string(),
    )));
    assert_eq!(CodeGenerator::get_string_deref_var(&expr, &ctx), None);
}

// ============================================================================
// TypeContext: global variable tracking (DECY-220)
// ============================================================================

#[test]
fn type_context_global_variable_tracking() {
    let mut ctx = TypeContext::new();
    assert!(!ctx.is_global("counter"));

    ctx.add_global("counter".to_string());
    assert!(ctx.is_global("counter"));
    assert!(!ctx.is_global("other"));
}

// ============================================================================
// TypeContext: renamed locals (DECY-245)
// ============================================================================

#[test]
fn type_context_renamed_locals() {
    let mut ctx = TypeContext::new();
    assert!(ctx.get_renamed_local("x").is_none());

    ctx.add_renamed_local("x".to_string(), "x_local".to_string());
    assert_eq!(ctx.get_renamed_local("x"), Some(&"x_local".to_string()));
    assert!(ctx.get_renamed_local("y").is_none());
}

// ============================================================================
// TypeContext: string iteration params (DECY-134)
// ============================================================================

#[test]
fn type_context_string_iter_params() {
    let mut ctx = TypeContext::new();
    assert!(ctx.get_string_iter_index("dest").is_none());

    ctx.add_string_iter_param("dest".to_string(), "dest_idx".to_string());
    assert_eq!(
        ctx.get_string_iter_index("dest"),
        Some(&"dest_idx".to_string())
    );
    assert!(ctx.is_string_iter_param("dest"));
    assert!(!ctx.is_string_iter_param("src"));
}

// ============================================================================
// TypeContext: string iteration functions (DECY-134b)
// ============================================================================

#[test]
fn type_context_string_iter_funcs() {
    let mut ctx = TypeContext::new();
    assert!(ctx.get_string_iter_func("strcpy").is_none());

    ctx.add_string_iter_func("strcpy".to_string(), vec![(0, true), (1, false)]);
    let params = ctx.get_string_iter_func("strcpy").unwrap();
    assert_eq!(params.len(), 2);
    assert_eq!(params[0], (0, true));
    assert_eq!(params[1], (1, false));
}

// ============================================================================
// TypeContext: function signatures (DECY-117) and slice args (DECY-116)
// ============================================================================

#[test]
fn type_context_function_param_types() {
    let mut ctx = TypeContext::new();
    ctx.add_function(
        "sum_array".to_string(),
        vec![
            HirType::Pointer(Box::new(HirType::Int)),
            HirType::Int,
        ],
    );

    assert_eq!(
        ctx.get_function_param_type("sum_array", 0),
        Some(&HirType::Pointer(Box::new(HirType::Int)))
    );
    assert_eq!(
        ctx.get_function_param_type("sum_array", 1),
        Some(&HirType::Int)
    );
    assert_eq!(ctx.get_function_param_type("sum_array", 2), None);
    assert_eq!(ctx.get_function_param_type("other", 0), None);
}

#[test]
fn type_context_slice_func_args() {
    let mut ctx = TypeContext::new();
    ctx.add_slice_func_args("process".to_string(), vec![(0, 1)]);

    let mappings = ctx.get_slice_func_len_indices("process").unwrap();
    assert_eq!(mappings.len(), 1);
    assert_eq!(mappings[0], (0, 1));

    assert!(ctx.get_slice_func_len_indices("other").is_none());
}

// ============================================================================
// TypeContext: is_pointer, is_option, is_vec
// ============================================================================

#[test]
fn type_context_is_pointer() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    ctx.add_variable("val".to_string(), HirType::Int);

    assert!(ctx.is_pointer("ptr"));
    assert!(!ctx.is_pointer("val"));
    assert!(!ctx.is_pointer("unknown"));
}

#[test]
fn type_context_is_option() {
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "opt".to_string(),
        HirType::Option(Box::new(HirType::Int)),
    );
    ctx.add_variable("val".to_string(), HirType::Int);

    assert!(ctx.is_option("opt"));
    assert!(!ctx.is_option("val"));
}

#[test]
fn type_context_is_vec() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("v".to_string(), HirType::Vec(Box::new(HirType::Int)));
    ctx.add_variable("val".to_string(), HirType::Int);

    assert!(ctx.is_vec("v"));
    assert!(!ctx.is_vec("val"));
}

// ============================================================================
// infer_expression_type: literal types (DECY-204)
// ============================================================================

#[test]
fn infer_expression_type_float_literal() {
    let ctx = TypeContext::new();
    let expr = HirExpression::FloatLiteral("3.14".to_string());
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Double));
}

#[test]
fn infer_expression_type_char_literal() {
    let ctx = TypeContext::new();
    let expr = HirExpression::CharLiteral(b'a' as i8);
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Char));
}

#[test]
fn infer_expression_type_int_literal() {
    let ctx = TypeContext::new();
    let expr = HirExpression::IntLiteral(42);
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Int));
}

// ============================================================================
// infer_expression_type: dereference through Box and Reference
// ============================================================================

#[test]
fn infer_deref_box_type() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("b".to_string(), HirType::Box(Box::new(HirType::Double)));

    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("b".to_string())));
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Double));
}

#[test]
fn infer_deref_reference_type() {
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "r".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Float),
            mutable: false,
        },
    );

    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("r".to_string())));
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Float));
}

#[test]
fn infer_deref_vec_type() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("v".to_string(), HirType::Vec(Box::new(HirType::Int)));

    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("v".to_string())));
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Int));
}

// ============================================================================
// infer_expression_type: ArrayIndex through Reference and Vec
// ============================================================================

#[test]
fn infer_array_index_reference_vec() {
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "slice".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Vec(Box::new(HirType::Double))),
            mutable: false,
        },
    );

    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("slice".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
    };
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Double));
}

#[test]
fn infer_array_index_reference_array() {
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "arr_ref".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Array {
                element_type: Box::new(HirType::Char),
                size: Some(10),
            }),
            mutable: false,
        },
    );

    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("arr_ref".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
    };
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Char));
}

#[test]
fn infer_array_index_vec_direct() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("v".to_string(), HirType::Vec(Box::new(HirType::Float)));

    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("v".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
    };
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Float));
}

// ============================================================================
// infer_expression_type: BinaryOp type promotion
// ============================================================================

#[test]
fn infer_binary_op_float_promotion() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Float);
    ctx.add_variable("y".to_string(), HirType::Int);

    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::Variable("y".to_string())),
    };
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Float));
}

#[test]
fn infer_binary_op_double_promotion() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Double);
    ctx.add_variable("y".to_string(), HirType::Float);

    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::Variable("y".to_string())),
    };
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Double));
}

#[test]
fn infer_binary_op_comparison_returns_int() {
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LessThan,
        left: Box::new(HirExpression::IntLiteral(1)),
        right: Box::new(HirExpression::IntLiteral(2)),
    };
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Int));
}

#[test]
fn infer_binary_op_bitwise_returns_int() {
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::BitwiseAnd,
        left: Box::new(HirExpression::IntLiteral(0xFF)),
        right: Box::new(HirExpression::IntLiteral(0x0F)),
    };
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Int));
}

// ============================================================================
// infer_expression_type: FieldAccess and PointerFieldAccess
// ============================================================================

#[test]
fn infer_field_access_struct() {
    let mut ctx = TypeContext::new();
    let struct_def = HirStruct::new(
        "Point".to_string(),
        vec![HirStructField::new("x".to_string(), HirType::Double)],
    );
    ctx.add_struct(&struct_def);
    ctx.add_variable("p".to_string(), HirType::Struct("Point".to_string()));

    let expr = HirExpression::FieldAccess {
        object: Box::new(HirExpression::Variable("p".to_string())),
        field: "x".to_string(),
    };
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Double));
}

#[test]
fn infer_pointer_field_access_through_pointer() {
    let mut ctx = TypeContext::new();
    let struct_def = HirStruct::new(
        "Node".to_string(),
        vec![HirStructField::new("val".to_string(), HirType::Int)],
    );
    ctx.add_struct(&struct_def);
    ctx.add_variable(
        "node".to_string(),
        HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
    );

    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("node".to_string())),
        field: "val".to_string(),
    };
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Int));
}

#[test]
fn infer_pointer_field_access_through_box() {
    let mut ctx = TypeContext::new();
    let struct_def = HirStruct::new(
        "Config".to_string(),
        vec![HirStructField::new("enabled".to_string(), HirType::Int)],
    );
    ctx.add_struct(&struct_def);
    ctx.add_variable(
        "cfg".to_string(),
        HirType::Box(Box::new(HirType::Struct("Config".to_string()))),
    );

    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("cfg".to_string())),
        field: "enabled".to_string(),
    };
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Int));
}

#[test]
fn infer_pointer_field_access_through_reference() {
    let mut ctx = TypeContext::new();
    let struct_def = HirStruct::new(
        "Data".to_string(),
        vec![HirStructField::new("size".to_string(), HirType::Int)],
    );
    ctx.add_struct(&struct_def);
    ctx.add_variable(
        "d".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Struct("Data".to_string())),
            mutable: true,
        },
    );

    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("d".to_string())),
        field: "size".to_string(),
    };
    assert_eq!(ctx.infer_expression_type(&expr), Some(HirType::Int));
}

#[test]
fn infer_pointer_field_access_non_struct_returns_none() {
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);

    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("x".to_string())),
        field: "val".to_string(),
    };
    assert_eq!(ctx.infer_expression_type(&expr), None);
}

// ============================================================================
// infer_expression_type: unknown expression
// ============================================================================

#[test]
fn infer_expression_type_string_literal() {
    let ctx = TypeContext::new();
    let expr = HirExpression::StringLiteral("hello".to_string());
    // StringLiteral falls through to the _ arm
    assert_eq!(ctx.infer_expression_type(&expr), None);
}

// ============================================================================
// escape_rust_keyword
// ============================================================================

#[test]
fn escape_rust_keyword_reserved() {
    assert_eq!(escape_rust_keyword("type"), "r#type");
    assert_eq!(escape_rust_keyword("fn"), "r#fn");
    assert_eq!(escape_rust_keyword("match"), "r#match");
    assert_eq!(escape_rust_keyword("yield"), "r#yield");
}

#[test]
fn escape_rust_keyword_non_reserved() {
    assert_eq!(escape_rust_keyword("count"), "count");
    assert_eq!(escape_rust_keyword("data"), "data");
}

// ============================================================================
// get_field_type_from_type helper
// ============================================================================

#[test]
fn get_field_type_from_type_non_struct_returns_none() {
    let ctx = TypeContext::new();
    assert_eq!(ctx.get_field_type_from_type(&HirType::Int, "field"), None);
}

#[test]
fn get_field_type_from_type_struct_not_registered_returns_none() {
    let ctx = TypeContext::new();
    assert_eq!(
        ctx.get_field_type_from_type(&HirType::Struct("Foo".to_string()), "bar"),
        None
    );
}
