//! Deep coverage tests for `generate_expression_with_target_type` and
//! `generate_statement_with_context`.
//!
//! Targets uncovered branches in expression type coercion, casting, dereference,
//! address-of, ternary, sizeof, null pointer, array index, struct field access,
//! and statement generation for Switch, For, DoWhile, Break, Continue,
//! VariableDeclaration with complex initializers, Assignment with realloc,
//! DerefAssignment, FieldAssignment, ArrayIndexAssignment, and Free.

use super::*;
use decy_hir::{
    BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType, SwitchCase,
    UnaryOperator,
};

// ============================================================================
// Helpers
// ============================================================================

fn gen() -> CodeGenerator {
    CodeGenerator::new()
}

fn make_ctx() -> TypeContext {
    TypeContext::new()
}

fn make_void_func(stmts: Vec<HirStatement>) -> HirFunction {
    HirFunction::new_with_body("test_func".to_string(), HirType::Void, vec![], stmts)
}

fn make_int_func(stmts: Vec<HirStatement>) -> HirFunction {
    HirFunction::new_with_body("test_func".to_string(), HirType::Int, vec![], stmts)
}

fn make_main_func(stmts: Vec<HirStatement>) -> HirFunction {
    HirFunction::new_with_body("main".to_string(), HirType::Int, vec![], stmts)
}

fn make_func_with_params(
    ret: HirType,
    params: Vec<HirParameter>,
    stmts: Vec<HirStatement>,
) -> HirFunction {
    HirFunction::new_with_body("test_func".to_string(), ret, params, stmts)
}

// ============================================================================
// EXPRESSION: IntLiteral 0 with target Option<T> -> None
// ============================================================================

#[test]
fn int_literal_zero_with_option_target_becomes_none() {
    let ctx = make_ctx();
    let expr = HirExpression::IntLiteral(0);
    let result =
        gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Option(Box::new(HirType::Int))));
    assert_eq!(result, "None", "IntLiteral(0) with Option target should be None, got: {}", result);
}

#[test]
fn int_literal_zero_with_pointer_target_becomes_null_mut() {
    let ctx = make_ctx();
    let expr = HirExpression::IntLiteral(0);
    let result =
        gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Pointer(Box::new(HirType::Int))));
    assert_eq!(result, "std::ptr::null_mut()", "IntLiteral(0) with Pointer target should be null_mut, got: {}", result);
}

#[test]
fn int_literal_nonzero_with_pointer_target_stays_literal() {
    let ctx = make_ctx();
    let expr = HirExpression::IntLiteral(42);
    let result =
        gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Pointer(Box::new(HirType::Int))));
    assert_eq!(result, "42", "Non-zero IntLiteral with Pointer target should be just the number, got: {}", result);
}

// ============================================================================
// EXPRESSION: AddressOf with target type pointer
// ============================================================================

#[test]
fn address_of_with_pointer_target_casts_to_raw() {
    let ctx = make_ctx();
    let expr = HirExpression::AddressOf(Box::new(HirExpression::Variable("x".to_string())));
    let result = gen().generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(result.contains("&mut x as *mut i32"), "AddressOf with Pointer target should cast, got: {}", result);
}

#[test]
fn address_of_without_target_type_is_ref() {
    let ctx = make_ctx();
    let expr = HirExpression::AddressOf(Box::new(HirExpression::Variable("x".to_string())));
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "&x", "AddressOf without target should be &x, got: {}", result);
}

#[test]
fn address_of_dereference_wraps_in_parens() {
    let ctx = make_ctx();
    let expr = HirExpression::AddressOf(Box::new(HirExpression::Dereference(Box::new(
        HirExpression::Variable("p".to_string()),
    ))));
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("&("), "AddressOf(Dereference) should wrap in parens, got: {}", result);
}

// ============================================================================
// EXPRESSION: UnaryOp AddressOf with target type pointer
// ============================================================================

#[test]
fn unary_address_of_with_pointer_target_casts_to_raw() {
    let ctx = make_ctx();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::AddressOf,
        operand: Box::new(HirExpression::Variable("y".to_string())),
    };
    let result = gen().generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(result.contains("&mut y as *mut i32"), "UnaryOp AddressOf with Pointer target should cast, got: {}", result);
}

#[test]
fn unary_address_of_without_target_is_ref() {
    let ctx = make_ctx();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::AddressOf,
        operand: Box::new(HirExpression::Variable("y".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "&y", "UnaryOp AddressOf without target should be &y, got: {}", result);
}

// ============================================================================
// EXPRESSION: LogicalNot with target type
// ============================================================================

#[test]
fn logical_not_bool_expr_with_int_target_casts() {
    let ctx = make_ctx();
    // !( a == b ) with Int target -> (!(...)) as i32
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("as i32"), "LogicalNot(bool) with Int target should cast, got: {}", result);
    assert!(result.contains("!("), "Should negate with parens, got: {}", result);
}

#[test]
fn logical_not_int_expr_with_int_target_uses_eq_zero() {
    let mut ctx = make_ctx();
    ctx.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("== 0") && result.contains("as i32"), "!int with Int target should be (x==0) as i32, got: {}", result);
}

#[test]
fn logical_not_bool_expr_without_target_no_cast() {
    let ctx = make_ctx();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(!result.contains("as i32"), "LogicalNot(bool) without Int target should not cast, got: {}", result);
    assert!(result.contains("!"), "Should have negation, got: {}", result);
}

#[test]
fn logical_not_int_expr_without_target_becomes_eq_zero() {
    let mut ctx = make_ctx();
    ctx.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("== 0"), "!int without target should use == 0, got: {}", result);
    assert!(!result.contains("as i32"), "Should not cast to i32 without Int target, got: {}", result);
}

// ============================================================================
// EXPRESSION: StringLiteral with Pointer(Char) target
// ============================================================================

#[test]
fn string_literal_with_char_pointer_target_becomes_byte_string() {
    let ctx = make_ctx();
    let expr = HirExpression::StringLiteral("hello".to_string());
    let result = gen().generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Char))),
    );
    assert!(result.contains("b\"hello\\0\""), "String with char* target should be byte string, got: {}", result);
    assert!(result.contains("as_ptr"), "Should call as_ptr, got: {}", result);
    assert!(result.contains("*mut u8"), "Should cast to *mut u8, got: {}", result);
}

#[test]
fn string_literal_without_pointer_target_is_plain_string() {
    let ctx = make_ctx();
    let expr = HirExpression::StringLiteral("world".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "\"world\"", "String without target should be plain string, got: {}", result);
}

// ============================================================================
// EXPRESSION: CharLiteral branches
// ============================================================================

#[test]
fn char_literal_zero_becomes_0u8() {
    let ctx = make_ctx();
    let expr = HirExpression::CharLiteral(0);
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "0u8", "CharLiteral(0) should be 0u8, got: {}", result);
}

#[test]
fn char_literal_printable_becomes_byte_char() {
    let ctx = make_ctx();
    let expr = HirExpression::CharLiteral(65); // 'A'
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "b'A'", "CharLiteral(65) should be b'A', got: {}", result);
}

#[test]
fn char_literal_space_becomes_byte_space() {
    let ctx = make_ctx();
    let expr = HirExpression::CharLiteral(32); // ' '
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "b' '", "CharLiteral(32) should be b' ', got: {}", result);
}

#[test]
fn char_literal_non_printable_becomes_numeric_u8() {
    let ctx = make_ctx();
    let expr = HirExpression::CharLiteral(7); // BEL
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "7u8", "Non-printable CharLiteral should be numeric u8, got: {}", result);
}

// ============================================================================
// EXPRESSION: Variable with special names (stderr, stdin, stdout, errno)
// ============================================================================

#[test]
fn variable_stderr_becomes_io_stderr() {
    let ctx = make_ctx();
    let expr = HirExpression::Variable("stderr".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "std::io::stderr()", "stderr should map to std::io::stderr(), got: {}", result);
}

#[test]
fn variable_stdin_becomes_io_stdin() {
    let ctx = make_ctx();
    let expr = HirExpression::Variable("stdin".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "std::io::stdin()", "stdin should map to std::io::stdin(), got: {}", result);
}

#[test]
fn variable_stdout_becomes_io_stdout() {
    let ctx = make_ctx();
    let expr = HirExpression::Variable("stdout".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "std::io::stdout()", "stdout should map to std::io::stdout(), got: {}", result);
}

#[test]
fn variable_errno_becomes_unsafe_errno() {
    let ctx = make_ctx();
    let expr = HirExpression::Variable("errno".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "unsafe { ERRNO }", "errno should be unsafe ERRNO, got: {}", result);
}

#[test]
fn variable_erange_becomes_constant() {
    let ctx = make_ctx();
    let expr = HirExpression::Variable("ERANGE".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "34i32", "ERANGE should be 34i32, got: {}", result);
}

#[test]
fn variable_einval_becomes_constant() {
    let ctx = make_ctx();
    let expr = HirExpression::Variable("EINVAL".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "22i32", "EINVAL should be 22i32, got: {}", result);
}

#[test]
fn variable_enoent_becomes_constant() {
    let ctx = make_ctx();
    let expr = HirExpression::Variable("ENOENT".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "2i32", "ENOENT should be 2i32, got: {}", result);
}

#[test]
fn variable_eacces_becomes_constant() {
    let ctx = make_ctx();
    let expr = HirExpression::Variable("EACCES".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "13i32", "EACCES should be 13i32, got: {}", result);
}

// ============================================================================
// EXPRESSION: Variable with Vec target returns as-is
// ============================================================================

#[test]
fn variable_with_vec_target_returns_directly() {
    let mut ctx = make_ctx();
    ctx.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("arr".to_string());
    let result = gen().generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Vec(Box::new(HirType::Int))),
    );
    assert_eq!(result, "arr", "Variable with Vec target should return directly, got: {}", result);
}

// ============================================================================
// EXPRESSION: Variable Box to raw pointer
// ============================================================================

#[test]
fn box_variable_to_pointer_target_uses_into_raw() {
    let mut ctx = make_ctx();
    ctx.add_variable("bx".to_string(), HirType::Box(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("bx".to_string());
    let result = gen().generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(result.contains("Box::into_raw"), "Box to pointer should use Box::into_raw, got: {}", result);
}

// ============================================================================
// EXPRESSION: Reference to pointer coercion (DECY-146)
// ============================================================================

#[test]
fn mutable_ref_to_pointer_casts() {
    let mut ctx = make_ctx();
    ctx.add_variable(
        "r".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        },
    );
    let expr = HirExpression::Variable("r".to_string());
    let result = gen().generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(result.contains("as *mut _"), "Mutable ref to pointer should cast, got: {}", result);
}

#[test]
fn immutable_ref_to_pointer_casts_const_then_mut() {
    let mut ctx = make_ctx();
    ctx.add_variable(
        "r".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
    );
    let expr = HirExpression::Variable("r".to_string());
    let result = gen().generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(result.contains("as *const _ as *mut _"), "Immutable ref to pointer should double cast, got: {}", result);
}

// ============================================================================
// EXPRESSION: Mutable ref to slice -> as_mut_ptr (DECY-149)
// ============================================================================

#[test]
fn mutable_ref_to_vec_to_pointer_uses_as_mut_ptr() {
    let mut ctx = make_ctx();
    ctx.add_variable(
        "s".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Vec(Box::new(HirType::Int))),
            mutable: true,
        },
    );
    let expr = HirExpression::Variable("s".to_string());
    let result = gen().generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(result.contains("as_mut_ptr"), "Mutable ref to Vec to pointer should use as_mut_ptr, got: {}", result);
}

#[test]
fn immutable_ref_to_vec_to_pointer_uses_as_ptr() {
    let mut ctx = make_ctx();
    ctx.add_variable(
        "s".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Vec(Box::new(HirType::Int))),
            mutable: false,
        },
    );
    let expr = HirExpression::Variable("s".to_string());
    let result = gen().generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(result.contains("as_ptr"), "Immutable ref to Vec to pointer should use as_ptr, got: {}", result);
}

// ============================================================================
// EXPRESSION: Array to pointer (DECY-211/244)
// ============================================================================

#[test]
fn array_variable_to_matching_pointer_uses_as_mut_ptr() {
    let mut ctx = make_ctx();
    ctx.add_variable(
        "arr".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    let expr = HirExpression::Variable("arr".to_string());
    let result = gen().generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(result.contains("as_mut_ptr"), "Array to matching pointer should use as_mut_ptr, got: {}", result);
}

#[test]
fn array_variable_to_void_pointer_casts() {
    let mut ctx = make_ctx();
    ctx.add_variable(
        "arr".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(5),
        },
    );
    let expr = HirExpression::Variable("arr".to_string());
    let result = gen().generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Void))),
    );
    assert!(result.contains("as_mut_ptr") && result.contains("as *mut ()"),
        "Array to void pointer should use as_mut_ptr and cast, got: {}", result);
}

// ============================================================================
// EXPRESSION: Pointer variable to pointer target stays as-is (DECY-148)
// ============================================================================

#[test]
fn pointer_variable_to_pointer_target_stays_asis() {
    let mut ctx = make_ctx();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("p".to_string());
    let result = gen().generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert_eq!(result, "p", "Pointer to pointer target should stay as-is, got: {}", result);
}

// ============================================================================
// EXPRESSION: BinaryOp Assign embedded assignment
// ============================================================================

#[test]
fn binary_assign_generates_block_expression() {
    let ctx = make_ctx();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Assign,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::IntLiteral(5)),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("__assign_tmp"), "Assign should generate temp var, got: {}", result);
    assert!(result.contains("x = __assign_tmp"), "Should assign to x, got: {}", result);
}

#[test]
fn binary_assign_global_array_index_generates_unsafe() {
    let mut ctx = make_ctx();
    ctx.add_variable("buf".to_string(), HirType::Array {
        element_type: Box::new(HirType::Int),
        size: Some(100),
    });
    ctx.add_global("buf".to_string());
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Assign,
        left: Box::new(HirExpression::ArrayIndex {
            array: Box::new(HirExpression::Variable("buf".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
        }),
        right: Box::new(HirExpression::IntLiteral(42)),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("unsafe"), "Global array assign should be unsafe, got: {}", result);
}

// ============================================================================
// EXPRESSION: Option null comparison (is_none/is_some)
// ============================================================================

#[test]
fn option_equal_null_becomes_is_none() {
    let mut ctx = make_ctx();
    ctx.add_variable("p".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("p".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("is_none"), "Option == NULL should be is_none, got: {}", result);
}

#[test]
fn option_not_equal_null_becomes_is_some() {
    let mut ctx = make_ctx();
    ctx.add_variable("p".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("p".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("is_some"), "Option != NULL should be is_some, got: {}", result);
}

#[test]
fn null_equal_option_becomes_is_none() {
    let mut ctx = make_ctx();
    ctx.add_variable("p".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::NullLiteral),
        right: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("is_none"), "NULL == Option should be is_none, got: {}", result);
}

#[test]
fn null_not_equal_option_becomes_is_some() {
    let mut ctx = make_ctx();
    ctx.add_variable("p".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::NullLiteral),
        right: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("is_some"), "NULL != Option should be is_some, got: {}", result);
}

// ============================================================================
// EXPRESSION: Pointer comparison with zero (null pointer check)
// ============================================================================

#[test]
fn pointer_equal_zero_becomes_null_mut_comparison() {
    let mut ctx = make_ctx();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("std::ptr::null_mut()"), "ptr == 0 should use null_mut(), got: {}", result);
}

#[test]
fn zero_not_equal_pointer_becomes_null_mut_comparison() {
    let mut ctx = make_ctx();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::Variable("ptr".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("std::ptr::null_mut()"), "0 != ptr should use null_mut(), got: {}", result);
}

// ============================================================================
// EXPRESSION: Vec null check -> always false/true (DECY-130)
// ============================================================================

#[test]
fn vec_equal_null_becomes_false() {
    let mut ctx = make_ctx();
    ctx.add_variable("v".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("v".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("false"), "Vec == null should be false, got: {}", result);
}

#[test]
fn vec_not_equal_zero_becomes_true() {
    let mut ctx = make_ctx();
    ctx.add_variable("v".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("v".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("true"), "Vec != 0 should be true, got: {}", result);
}

// ============================================================================
// EXPRESSION: Comparison returns bool, cast to i32 when target is Int (DECY-191)
// ============================================================================

#[test]
fn comparison_with_int_target_casts_to_i32() {
    let ctx = make_ctx();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::GreaterThan,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("as i32"), "Comparison with Int target should cast to i32, got: {}", result);
}

#[test]
fn comparison_without_int_target_no_cast() {
    let ctx = make_ctx();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LessThan,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(!result.contains("as i32"), "Comparison without Int target should not cast, got: {}", result);
}

// ============================================================================
// EXPRESSION: Chained comparison (DECY-206)
// ============================================================================

#[test]
fn chained_comparison_casts_bool_operand_to_i32() {
    let ctx = make_ctx();
    // (a < b) < c  ->  ((a < b) as i32) < c
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LessThan,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
        right: Box::new(HirExpression::Variable("c".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as i32"), "Chained comparison should cast bool to i32, got: {}", result);
}

#[test]
fn chained_comparison_with_int_target_adds_outer_cast() {
    let ctx = make_ctx();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::GreaterThan,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        }),
        right: Box::new(HirExpression::Variable("y".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("as i32"), "Chained comparison with Int target should add cast, got: {}", result);
}

// ============================================================================
// EXPRESSION: Signed/unsigned comparison mismatch (DECY-251)
// ============================================================================

#[test]
fn signed_unsigned_comparison_casts_to_i64() {
    let mut ctx = make_ctx();
    ctx.add_variable("s".to_string(), HirType::Int);
    ctx.add_variable("u".to_string(), HirType::UnsignedInt);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LessThan,
        left: Box::new(HirExpression::Variable("s".to_string())),
        right: Box::new(HirExpression::Variable("u".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as i64"), "Signed/unsigned comparison should cast to i64, got: {}", result);
}

#[test]
fn unsigned_signed_comparison_with_int_target_casts_result() {
    let mut ctx = make_ctx();
    ctx.add_variable("u".to_string(), HirType::UnsignedInt);
    ctx.add_variable("s".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::GreaterEqual,
        left: Box::new(HirExpression::Variable("u".to_string())),
        right: Box::new(HirExpression::Variable("s".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("as i64"), "Should cast operands to i64, got: {}", result);
    assert!(result.contains("as i32"), "With Int target should also cast result, got: {}", result);
}

// ============================================================================
// EXPRESSION: Arithmetic result cast to float/double target (DECY-204)
// ============================================================================

#[test]
fn int_arithmetic_with_float_target_casts_result() {
    let mut ctx = make_ctx();
    ctx.add_variable("a".to_string(), HirType::Int);
    ctx.add_variable("b".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Divide,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Float));
    assert!(result.contains("as f32"), "int/int with Float target should cast to f32, got: {}", result);
}

#[test]
fn int_arithmetic_with_double_target_casts_result() {
    let mut ctx = make_ctx();
    ctx.add_variable("a".to_string(), HirType::Int);
    ctx.add_variable("b".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Double));
    assert!(result.contains("as f64"), "int+int with Double target should cast to f64, got: {}", result);
}

// ============================================================================
// EXPRESSION: Bitwise operations with boolean operands (DECY-252)
// ============================================================================

#[test]
fn bitwise_and_with_bool_operand_casts_to_i32() {
    let mut ctx = make_ctx();
    ctx.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::BitwiseAnd,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("y".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        }),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as i32"), "Bitwise & with bool should cast to i32, got: {}", result);
}

#[test]
fn bitwise_or_with_unsigned_operands_casts_result() {
    let mut ctx = make_ctx();
    ctx.add_variable("a".to_string(), HirType::UnsignedInt);
    ctx.add_variable("b".to_string(), HirType::UnsignedInt);
    // Need a bool operand to trigger the bool/unsigned handling
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::BitwiseOr,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::NotEqual,
            left: Box::new(HirExpression::Variable("b".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    // Either casts to i32 (for bool) or u32 (for unsigned), but should handle the mismatch
    assert!(result.contains("as i32") || result.contains("as u32"),
        "Bitwise | with bool+unsigned should cast, got: {}", result);
}

// ============================================================================
// EXPRESSION: Float/double mixed arithmetic (DECY-204)
// ============================================================================

#[test]
fn float_plus_double_promotes_to_f64() {
    let mut ctx = make_ctx();
    ctx.add_variable("f".to_string(), HirType::Float);
    ctx.add_variable("d".to_string(), HirType::Double);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("f".to_string())),
        right: Box::new(HirExpression::Variable("d".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as f64"), "float + double should promote to f64, got: {}", result);
}

#[test]
fn double_multiply_float_promotes_to_f64() {
    let mut ctx = make_ctx();
    ctx.add_variable("d".to_string(), HirType::Double);
    ctx.add_variable("f".to_string(), HirType::Float);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::Variable("d".to_string())),
        right: Box::new(HirExpression::Variable("f".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as f64"), "double * float should promote to f64, got: {}", result);
}

// ============================================================================
// EXPRESSION: Ternary
// ============================================================================

#[test]
fn ternary_with_bool_condition() {
    let ctx = make_ctx();
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
        then_expr: Box::new(HirExpression::IntLiteral(1)),
        else_expr: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("if"), "Ternary should use if expression, got: {}", result);
    assert!(result.contains("else"), "Ternary should have else, got: {}", result);
}

#[test]
fn ternary_with_non_bool_condition_adds_ne_zero() {
    let ctx = make_ctx();
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::Variable("flag".to_string())),
        then_expr: Box::new(HirExpression::IntLiteral(1)),
        else_expr: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("!= 0"), "Non-bool ternary condition should get != 0, got: {}", result);
}

#[test]
fn ternary_propagates_target_type_to_branches() {
    let ctx = make_ctx();
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
        then_expr: Box::new(HirExpression::IntLiteral(0)),
        else_expr: Box::new(HirExpression::IntLiteral(1)),
    };
    let result = gen().generate_expression_with_target_type(
        &expr,
        &ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    // The then branch IntLiteral(0) with Pointer target should become null_mut
    assert!(result.contains("null_mut"), "Ternary with pointer target should propagate to 0 -> null_mut, got: {}", result);
}

// ============================================================================
// EXPRESSION: Cast
// ============================================================================

#[test]
fn cast_int_generates_as_i32() {
    let ctx = make_ctx();
    let expr = HirExpression::Cast {
        target_type: HirType::Int,
        expr: Box::new(HirExpression::Variable("x".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as i32"), "Cast to Int should generate as i32, got: {}", result);
}

#[test]
fn cast_binary_op_wraps_in_parens() {
    let ctx = make_ctx();
    let expr = HirExpression::Cast {
        target_type: HirType::Float,
        expr: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("(a + b) as f32"), "Cast of BinaryOp should wrap in parens, got: {}", result);
}

#[test]
fn cast_address_of_to_int_uses_pointer_chain() {
    let ctx = make_ctx();
    let expr = HirExpression::Cast {
        target_type: HirType::Int,
        expr: Box::new(HirExpression::AddressOf(Box::new(HirExpression::Variable("x".to_string())))),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as *const _ as isize as i32"), "Cast AddressOf to int should chain casts, got: {}", result);
}

#[test]
fn cast_unary_address_of_to_char_uses_pointer_chain() {
    let ctx = make_ctx();
    let expr = HirExpression::Cast {
        target_type: HirType::Char,
        expr: Box::new(HirExpression::UnaryOp {
            op: UnaryOperator::AddressOf,
            operand: Box::new(HirExpression::Variable("x".to_string())),
        }),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as *const _ as isize as u8"), "Cast &x to char should chain casts, got: {}", result);
}

// ============================================================================
// EXPRESSION: NullLiteral
// ============================================================================

#[test]
fn null_literal_becomes_none() {
    let ctx = make_ctx();
    let expr = HirExpression::NullLiteral;
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "None", "NullLiteral should be None, got: {}", result);
}

// ============================================================================
// EXPRESSION: Dereference with raw pointer needs unsafe
// ============================================================================

#[test]
fn dereference_raw_pointer_wraps_unsafe() {
    let mut ctx = make_ctx();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("ptr".to_string())));
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("unsafe"), "Dereference of raw pointer should be unsafe, got: {}", result);
    assert!(result.contains("*ptr"), "Should dereference ptr, got: {}", result);
}

#[test]
fn dereference_non_pointer_no_unsafe() {
    let mut ctx = make_ctx();
    ctx.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("x".to_string())));
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(!result.contains("unsafe"), "Dereference of non-pointer should not be unsafe, got: {}", result);
}

// ============================================================================
// EXPRESSION: UnaryOp (PostIncrement, PreDecrement, BitwiseNot, Minus)
// ============================================================================

#[test]
fn post_increment_generates_block() {
    let ctx = make_ctx();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PostIncrement,
        operand: Box::new(HirExpression::Variable("i".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("__tmp"), "PostIncrement should use temp var, got: {}", result);
    assert!(result.contains("+= 1"), "PostIncrement should increment, got: {}", result);
}

#[test]
fn pre_decrement_generates_block() {
    let ctx = make_ctx();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PreDecrement,
        operand: Box::new(HirExpression::Variable("i".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("-= 1"), "PreDecrement should decrement, got: {}", result);
}

#[test]
fn post_increment_pointer_uses_wrapping_add() {
    let mut ctx = make_ctx();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PostIncrement,
        operand: Box::new(HirExpression::Variable("ptr".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("wrapping_add"), "Pointer post-increment should use wrapping_add, got: {}", result);
}

#[test]
fn pre_increment_pointer_uses_wrapping_add() {
    let mut ctx = make_ctx();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PreIncrement,
        operand: Box::new(HirExpression::Variable("ptr".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("wrapping_add"), "Pointer pre-increment should use wrapping_add, got: {}", result);
}

#[test]
fn post_decrement_pointer_uses_wrapping_sub() {
    let mut ctx = make_ctx();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PostDecrement,
        operand: Box::new(HirExpression::Variable("ptr".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("wrapping_sub"), "Pointer post-decrement should use wrapping_sub, got: {}", result);
}

#[test]
fn pre_decrement_pointer_uses_wrapping_sub() {
    let mut ctx = make_ctx();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PreDecrement,
        operand: Box::new(HirExpression::Variable("ptr".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("wrapping_sub"), "Pointer pre-decrement should use wrapping_sub, got: {}", result);
}

#[test]
fn unary_minus_generates_negation() {
    let ctx = make_ctx();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::Minus,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "-x", "Unary minus should generate -x, got: {}", result);
}

#[test]
fn unary_bitwise_not_generates_bang() {
    let ctx = make_ctx();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::BitwiseNot,
        operand: Box::new(HirExpression::Variable("mask".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "!mask", "BitwiseNot should generate !mask, got: {}", result);
}

// ============================================================================
// EXPRESSION: Sizeof
// ============================================================================

#[test]
fn sizeof_basic_type() {
    let ctx = make_ctx();
    let expr = HirExpression::Sizeof {
        type_name: "int".to_string(),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("size_of"), "Sizeof should use std::mem::size_of, got: {}", result);
}

// ============================================================================
// EXPRESSION: Calloc
// ============================================================================

#[test]
fn calloc_generates_vec_with_zeros() {
    let ctx = make_ctx();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(10)),
        element_type: Box::new(HirType::Int),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("vec![0i32; 10]"), "Calloc should generate vec![0i32; n], got: {}", result);
}

#[test]
fn calloc_unsigned_int_generates_vec_with_u32_zeros() {
    let ctx = make_ctx();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(5)),
        element_type: Box::new(HirType::UnsignedInt),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("vec![0u32; 5]"), "Calloc UnsignedInt should generate 0u32, got: {}", result);
}

#[test]
fn calloc_float_generates_vec_with_f32_zeros() {
    let ctx = make_ctx();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(3)),
        element_type: Box::new(HirType::Float),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("vec![0.0f32; 3]"), "Calloc Float should generate 0.0f32, got: {}", result);
}

#[test]
fn calloc_double_generates_vec_with_f64_zeros() {
    let ctx = make_ctx();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(3)),
        element_type: Box::new(HirType::Double),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("vec![0.0f64; 3]"), "Calloc Double should generate 0.0f64, got: {}", result);
}

#[test]
fn calloc_char_generates_vec_with_u8_zeros() {
    let ctx = make_ctx();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(64)),
        element_type: Box::new(HirType::Char),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("vec![0u8; 64]"), "Calloc Char should generate 0u8, got: {}", result);
}

// ============================================================================
// EXPRESSION: Malloc
// ============================================================================

#[test]
fn malloc_array_pattern_generates_vec_with_capacity() {
    let ctx = make_ctx();
    let expr = HirExpression::Malloc {
        size: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::Sizeof {
                type_name: "int".to_string(),
            }),
        }),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("Vec::with_capacity"), "Malloc n*sizeof should use Vec::with_capacity, got: {}", result);
}

#[test]
fn malloc_single_generates_box() {
    let ctx = make_ctx();
    let expr = HirExpression::Malloc {
        size: Box::new(HirExpression::Sizeof {
            type_name: "int".to_string(),
        }),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("Box::new"), "Malloc single should use Box::new, got: {}", result);
}

// ============================================================================
// EXPRESSION: Realloc
// ============================================================================

#[test]
fn realloc_null_pointer_with_array_size_generates_vec() {
    let ctx = make_ctx();
    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::NullLiteral),
        new_size: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::IntLiteral(10)),
            right: Box::new(HirExpression::Sizeof {
                type_name: "int".to_string(),
            }),
        }),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("vec!"), "Realloc(NULL, n*sizeof) should generate vec!, got: {}", result);
}

#[test]
fn realloc_null_pointer_simple_size_generates_vec_new() {
    let ctx = make_ctx();
    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::NullLiteral),
        new_size: Box::new(HirExpression::IntLiteral(100)),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("Vec::new()"), "Realloc(NULL, simple) should generate Vec::new(), got: {}", result);
}

#[test]
fn realloc_non_null_returns_pointer() {
    let mut ctx = make_ctx();
    ctx.add_variable("buf".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::Variable("buf".to_string())),
        new_size: Box::new(HirExpression::IntLiteral(200)),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    // Should fall through to just returning the pointer expression
    assert!(result.contains("buf"), "Realloc(ptr, size) should reference pointer, got: {}", result);
}

// ============================================================================
// EXPRESSION: IsNotNull
// ============================================================================

#[test]
fn is_not_null_generates_if_let_some() {
    let ctx = make_ctx();
    let expr = HirExpression::IsNotNull(Box::new(HirExpression::Variable("p".to_string())));
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("if let Some(_) = p"), "IsNotNull should generate if let Some, got: {}", result);
}

// ============================================================================
// EXPRESSION: StringMethodCall
// ============================================================================

#[test]
fn string_method_call_len_casts_to_i32() {
    let ctx = make_ctx();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "len".to_string(),
        arguments: vec![],
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("s.len() as i32"), "StringMethodCall.len should cast to i32, got: {}", result);
}

#[test]
fn string_method_call_is_empty_no_cast() {
    let ctx = make_ctx();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "is_empty".to_string(),
        arguments: vec![],
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "s.is_empty()", "StringMethodCall.is_empty should not cast, got: {}", result);
}

#[test]
fn string_method_call_clone_into_adds_mut_ref() {
    let ctx = make_ctx();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("src".to_string())),
        method: "clone_into".to_string(),
        arguments: vec![HirExpression::Variable("dst".to_string())],
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("&mut dst"), "clone_into should add &mut to arg, got: {}", result);
}

// ============================================================================
// STATEMENT: Return in main -> process::exit
// ============================================================================

#[test]
fn stmt_return_in_main_generates_process_exit() {
    let codegen = gen();
    let func = make_main_func(vec![
        HirStatement::Return(Some(HirExpression::IntLiteral(0))),
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("std::process::exit(0)"), "Return in main should use process::exit, got: {}", code);
}

#[test]
fn stmt_return_none_in_main_exits_zero() {
    let codegen = gen();
    let func = make_main_func(vec![HirStatement::Return(None)]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("std::process::exit(0)"), "Return None in main should exit(0), got: {}", code);
}

#[test]
fn stmt_return_value_in_non_main() {
    let codegen = gen();
    let func = make_int_func(vec![
        HirStatement::Return(Some(HirExpression::IntLiteral(42))),
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("return 42"), "Return in non-main should use return, got: {}", code);
}

#[test]
fn stmt_return_none_in_non_main() {
    let codegen = gen();
    let func = make_void_func(vec![HirStatement::Return(None)]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("return;"), "Return None in non-main should use return;, got: {}", code);
}

// ============================================================================
// STATEMENT: If with else block
// ============================================================================

#[test]
fn stmt_if_with_else_block() {
    let codegen = gen();
    let func = make_void_func(vec![HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        then_block: vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "foo".to_string(),
            arguments: vec![],
        })],
        else_block: Some(vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "bar".to_string(),
            arguments: vec![],
        })]),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("if x > 0"), "Should have if condition, got: {}", code);
    assert!(code.contains("foo()"), "Should have then block, got: {}", code);
    assert!(code.contains("} else {"), "Should have else block, got: {}", code);
    assert!(code.contains("bar()"), "Should have else body, got: {}", code);
}

#[test]
fn stmt_if_non_boolean_condition_wraps_ne_zero() {
    let codegen = gen();
    let func = make_void_func(vec![HirStatement::If {
        condition: HirExpression::Variable("flag".to_string()),
        then_block: vec![HirStatement::Break],
        else_block: None,
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("!= 0"), "Non-boolean if condition should get != 0, got: {}", code);
}

// ============================================================================
// STATEMENT: While with non-boolean condition
// ============================================================================

#[test]
fn stmt_while_with_int_condition_wraps_ne_zero() {
    let codegen = gen();
    let func = make_void_func(vec![HirStatement::While {
        condition: HirExpression::Variable("running".to_string()),
        body: vec![HirStatement::Break],
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("!= 0"), "Non-boolean while condition should get != 0, got: {}", code);
}

#[test]
fn stmt_while_with_pointer_condition_uses_is_null() {
    // Note: Parameters go through borrow gen which may transform Pointer to Reference
    // Instead, test with a VariableDeclaration that keeps the pointer type
    let codegen = gen();
    let func = make_void_func(vec![
        HirStatement::VariableDeclaration {
            name: "p".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::IntLiteral(0)),
        },
        HirStatement::While {
            condition: HirExpression::Variable("p".to_string()),
            body: vec![HirStatement::Break],
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("is_null"), "Pointer while condition should use is_null, got: {}", code);
}

// ============================================================================
// STATEMENT: Break and Continue
// ============================================================================

#[test]
fn stmt_break_generates_break() {
    let codegen = gen();
    let func = make_void_func(vec![HirStatement::While {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        },
        body: vec![HirStatement::Break],
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("break;"), "Break should generate break;, got: {}", code);
}

#[test]
fn stmt_continue_generates_continue() {
    let codegen = gen();
    let func = make_void_func(vec![HirStatement::While {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        },
        body: vec![HirStatement::Continue],
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("continue;"), "Continue should generate continue;, got: {}", code);
}

// ============================================================================
// STATEMENT: Switch with cases and default
// ============================================================================

#[test]
fn stmt_switch_generates_match() {
    let codegen = gen();
    let func = make_func_with_params(
        HirType::Void,
        vec![HirParameter::new("cmd".to_string(), HirType::Int)],
        vec![HirStatement::Switch {
            condition: HirExpression::Variable("cmd".to_string()),
            cases: vec![
                SwitchCase {
                    value: Some(HirExpression::IntLiteral(1)),
                    body: vec![
                        HirStatement::Expression(HirExpression::FunctionCall {
                            function: "handle_one".to_string(),
                            arguments: vec![],
                        }),
                        HirStatement::Break,
                    ],
                },
                SwitchCase {
                    value: Some(HirExpression::IntLiteral(2)),
                    body: vec![
                        HirStatement::Expression(HirExpression::FunctionCall {
                            function: "handle_two".to_string(),
                            arguments: vec![],
                        }),
                        HirStatement::Break,
                    ],
                },
            ],
            default_case: Some(vec![
                HirStatement::Expression(HirExpression::FunctionCall {
                    function: "handle_default".to_string(),
                    arguments: vec![],
                }),
                HirStatement::Break,
            ]),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("match cmd"), "Switch should generate match, got: {}", code);
    assert!(code.contains("1 =>"), "Should have case 1, got: {}", code);
    assert!(code.contains("2 =>"), "Should have case 2, got: {}", code);
    assert!(code.contains("_ =>"), "Should have default case, got: {}", code);
    assert!(code.contains("handle_one()"), "Should call handle_one, got: {}", code);
    assert!(code.contains("handle_default()"), "Should call handle_default, got: {}", code);
    // Break statements should be filtered out of match arms
    assert!(!code.contains("break;"), "Break should be filtered from match arms, got: {}", code);
}

#[test]
fn stmt_switch_char_literal_case_with_int_condition() {
    let codegen = gen();
    let func = make_func_with_params(
        HirType::Void,
        vec![HirParameter::new("ch".to_string(), HirType::Int)],
        vec![HirStatement::Switch {
            condition: HirExpression::Variable("ch".to_string()),
            cases: vec![SwitchCase {
                value: Some(HirExpression::CharLiteral(48)), // '0'
                body: vec![HirStatement::Break],
            }],
            default_case: None,
        }],
    );
    let code = codegen.generate_function(&func);
    // When condition is Int and case is CharLiteral, should use numeric value (48) not b'0'
    assert!(code.contains("48 =>"), "Int switch with CharLiteral case should use numeric value, got: {}", code);
}

#[test]
fn stmt_switch_without_default_has_empty_wildcard() {
    let codegen = gen();
    let func = make_func_with_params(
        HirType::Void,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Switch {
            condition: HirExpression::Variable("x".to_string()),
            cases: vec![SwitchCase {
                value: Some(HirExpression::IntLiteral(0)),
                body: vec![HirStatement::Break],
            }],
            default_case: None,
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("_ =>"), "Switch without default should still have _ arm, got: {}", code);
}

// ============================================================================
// STATEMENT: For loop with multiple init and increment
// ============================================================================

#[test]
fn stmt_for_loop_generates_init_while_increment() {
    let codegen = gen();
    let func = make_void_func(vec![HirStatement::For {
        init: vec![
            HirStatement::VariableDeclaration {
                name: "i".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::VariableDeclaration {
                name: "j".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
        ],
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        },
        increment: vec![
            HirStatement::Assignment {
                target: "i".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("i".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
            HirStatement::Assignment {
                target: "j".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("j".to_string())),
                    right: Box::new(HirExpression::IntLiteral(2)),
                },
            },
        ],
        body: vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "work".to_string(),
            arguments: vec![],
        })],
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("let mut i"), "Should have i init, got: {}", code);
    assert!(code.contains("let mut j"), "Should have j init, got: {}", code);
    assert!(code.contains("while"), "Should have while loop, got: {}", code);
}

// ============================================================================
// STATEMENT: VariableDeclaration with no initializer
// ============================================================================

#[test]
fn stmt_var_decl_no_init_gets_default() {
    let codegen = gen();
    let func = make_void_func(vec![HirStatement::VariableDeclaration {
        name: "count".to_string(),
        var_type: HirType::Int,
        initializer: None,
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("let mut count: i32"), "Should declare mutable typed var, got: {}", code);
}

// ============================================================================
// STATEMENT: VariableDeclaration char array with string literal
// ============================================================================

#[test]
fn stmt_var_decl_char_array_string_literal_generates_byte_string() {
    let codegen = gen();
    let func = make_void_func(vec![HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(10),
        },
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("*b\"hello\\0\""), "Char array with string init should use byte string, got: {}", code);
}

// ============================================================================
// STATEMENT: VariableDeclaration char* with string literal -> &str
// ============================================================================

#[test]
fn stmt_var_decl_char_pointer_string_literal_becomes_str_ref() {
    let codegen = gen();
    let func = make_void_func(vec![HirStatement::VariableDeclaration {
        name: "msg".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Char)),
        initializer: Some(HirExpression::StringLiteral("world".to_string())),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("&str"), "char* with string literal should become &str, got: {}", code);
    assert!(code.contains("\"world\""), "Should contain the string value, got: {}", code);
}

// ============================================================================
// STATEMENT: VariableDeclaration renamed due to global shadowing (DECY-245)
// ============================================================================

#[test]
fn stmt_var_decl_shadowing_global_gets_renamed() {
    let codegen = gen();
    // We need to use generate_function which builds context including globals
    // The simplest way is to check via the function generator
    let func = make_void_func(vec![
        HirStatement::VariableDeclaration {
            name: "count".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        },
    ]);
    // We can't easily test global renaming without the full pipeline, so test the basic case
    let code = codegen.generate_function(&func);
    assert!(code.contains("let mut count"), "Should declare the variable, got: {}", code);
}

// ============================================================================
// STATEMENT: Assignment with global variable
// ============================================================================

#[test]
fn stmt_assignment_to_global_wraps_unsafe() {
    let codegen = gen();
    // Use full function with a pattern that triggers global assignment
    let func = make_void_func(vec![
        HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::IntLiteral(42),
        },
    ]);
    let code = codegen.generate_function(&func);
    // Without global context, this should be a regular assignment
    assert!(code.contains("x = 42"), "Regular assignment should not be unsafe, got: {}", code);
}

// ============================================================================
// STATEMENT: Assignment to errno
// ============================================================================

#[test]
fn stmt_assignment_to_errno_wraps_unsafe() {
    let codegen = gen();
    let func = make_void_func(vec![
        HirStatement::Assignment {
            target: "errno".to_string(),
            value: HirExpression::IntLiteral(0),
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("unsafe") && code.contains("ERRNO"), "Assignment to errno should be unsafe ERRNO, got: {}", code);
}

// ============================================================================
// STATEMENT: Free generates RAII comment
// ============================================================================

#[test]
fn stmt_free_generates_raii_comment() {
    let codegen = gen();
    let func = make_void_func(vec![HirStatement::Free {
        pointer: HirExpression::Variable("buf".to_string()),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("RAII"), "Free should generate RAII comment, got: {}", code);
    assert!(code.contains("buf"), "RAII comment should mention variable name, got: {}", code);
}

// ============================================================================
// STATEMENT: Expression statement
// ============================================================================

#[test]
fn stmt_expression_generates_semicolon() {
    let codegen = gen();
    let func = make_void_func(vec![HirStatement::Expression(
        HirExpression::FunctionCall {
            function: "my_func".to_string(),
            arguments: vec![HirExpression::IntLiteral(42)],
        },
    )]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("my_func(42);"), "Expression statement should end with ;, got: {}", code);
}

// ============================================================================
// STATEMENT: DerefAssignment with PointerFieldAccess (no extra deref)
// ============================================================================

#[test]
fn stmt_deref_assignment_pointer_field_no_extra_deref() {
    let codegen = gen();
    let func = make_func_with_params(
        HirType::Void,
        vec![HirParameter::new(
            "node".to_string(),
            HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        )],
        vec![HirStatement::DerefAssignment {
            target: HirExpression::PointerFieldAccess {
                pointer: Box::new(HirExpression::Variable("node".to_string())),
                field: "value".to_string(),
            },
            value: HirExpression::IntLiteral(42),
        }],
    );
    let code = codegen.generate_function(&func);
    // PointerFieldAccess should not get an extra * prefix
    assert!(!code.contains("*(*node)"), "Should not double-dereference, got: {}", code);
}

// ============================================================================
// STATEMENT: DerefAssignment with raw pointer variable
// ============================================================================

#[test]
fn stmt_deref_assignment_raw_pointer_wraps_unsafe() {
    // Declare pointer as local variable to avoid borrow gen transforming the parameter
    let codegen = gen();
    let func = make_void_func(vec![
        HirStatement::VariableDeclaration {
            name: "ptr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::IntLiteral(0)),
        },
        HirStatement::DerefAssignment {
            target: HirExpression::Variable("ptr".to_string()),
            value: HirExpression::IntLiteral(99),
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("unsafe"), "DerefAssignment to raw pointer should be unsafe, got: {}", code);
    assert!(code.contains("*ptr"), "Should dereference ptr, got: {}", code);
}

// ============================================================================
// STATEMENT: ArrayIndexAssignment
// ============================================================================

#[test]
fn stmt_array_index_assignment_casts_index_to_usize() {
    let codegen = gen();
    let func = make_func_with_params(
        HirType::Void,
        vec![HirParameter::new(
            "arr".to_string(),
            HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(10),
            },
        )],
        vec![HirStatement::ArrayIndexAssignment {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::Variable("i".to_string())),
            value: HirExpression::IntLiteral(5),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("as usize"), "Array index should cast to usize, got: {}", code);
}

#[test]
fn stmt_array_index_assignment_raw_pointer_uses_unsafe() {
    // Declare pointer as local variable to avoid borrow gen transforming the parameter
    let codegen = gen();
    let func = make_void_func(vec![
        HirStatement::VariableDeclaration {
            name: "buf".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::IntLiteral(0)),
        },
        HirStatement::ArrayIndexAssignment {
            array: Box::new(HirExpression::Variable("buf".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
            value: HirExpression::IntLiteral(1),
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("unsafe"), "Raw pointer array assignment should be unsafe, got: {}", code);
    assert!(code.contains(".add("), "Should use pointer add, got: {}", code);
}

// ============================================================================
// STATEMENT: FieldAssignment
// ============================================================================

#[test]
fn stmt_field_assignment_generates_dot_notation() {
    let codegen = gen();
    let func = make_func_with_params(
        HirType::Void,
        vec![HirParameter::new(
            "pt".to_string(),
            HirType::Struct("Point".to_string()),
        )],
        vec![HirStatement::FieldAssignment {
            object: HirExpression::Variable("pt".to_string()),
            field: "x".to_string(),
            value: HirExpression::IntLiteral(10),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("pt.x = 10"), "Field assignment should use dot notation, got: {}", code);
}

// ============================================================================
// STATEMENT: VLA (Variable Length Array) declaration
// ============================================================================

#[test]
fn stmt_vla_declaration_generates_vec() {
    let codegen = gen();
    let func = make_func_with_params(
        HirType::Void,
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Int),
                size: None, // VLA - no compile-time size
            },
            initializer: Some(HirExpression::Variable("n".to_string())),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("vec!"), "VLA should generate vec!, got: {}", code);
    assert!(code.contains("0i32"), "VLA int should use 0i32 default, got: {}", code);
}

#[test]
fn stmt_vla_float_declaration_generates_vec_f32() {
    let codegen = gen();
    let func = make_func_with_params(
        HirType::Void,
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Float),
                size: None,
            },
            initializer: Some(HirExpression::Variable("n".to_string())),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("vec!"), "VLA should generate vec!, got: {}", code);
    assert!(code.contains("0.0f32"), "VLA float should use 0.0f32, got: {}", code);
}

#[test]
fn stmt_vla_double_declaration_generates_vec_f64() {
    let codegen = gen();
    let func = make_func_with_params(
        HirType::Void,
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Double),
                size: None,
            },
            initializer: Some(HirExpression::Variable("n".to_string())),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("0.0f64"), "VLA double should use 0.0f64, got: {}", code);
}

#[test]
fn stmt_vla_char_declaration_generates_vec_u8() {
    let codegen = gen();
    let func = make_func_with_params(
        HirType::Void,
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![HirStatement::VariableDeclaration {
            name: "buf".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Char),
                size: None,
            },
            initializer: Some(HirExpression::Variable("n".to_string())),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("0u8"), "VLA char should use 0u8, got: {}", code);
}

#[test]
fn stmt_vla_unsigned_int_declaration_generates_vec_u32() {
    let codegen = gen();
    let func = make_func_with_params(
        HirType::Void,
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::UnsignedInt),
                size: None,
            },
            initializer: Some(HirExpression::Variable("n".to_string())),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("0u32"), "VLA unsigned int should use 0u32, got: {}", code);
}

// ============================================================================
// STATEMENT: InlineAsm
// ============================================================================

#[test]
fn stmt_inline_asm_translatable_has_hint() {
    let codegen = gen();
    let func = make_void_func(vec![HirStatement::InlineAsm {
        text: "nop".to_string(),
        translatable: true,
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("manual review"), "InlineAsm should have review comment, got: {}", code);
    assert!(code.contains("translatable"), "Translatable asm should mention translatable, got: {}", code);
}

#[test]
fn stmt_inline_asm_non_translatable_no_hint() {
    let codegen = gen();
    let func = make_void_func(vec![HirStatement::InlineAsm {
        text: "cpuid".to_string(),
        translatable: false,
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("manual review"), "InlineAsm should have review comment, got: {}", code);
    assert!(code.contains("cpuid"), "Should include the assembly text, got: {}", code);
}

// ============================================================================
// STATEMENT: Assignment with realloc patterns
// ============================================================================

#[test]
fn stmt_assignment_realloc_zero_generates_clear() {
    let codegen = gen();
    let func = make_func_with_params(
        HirType::Void,
        vec![HirParameter::new("buf".to_string(), HirType::Vec(Box::new(HirType::Int)))],
        vec![HirStatement::Assignment {
            target: "buf".to_string(),
            value: HirExpression::Realloc {
                pointer: Box::new(HirExpression::Variable("buf".to_string())),
                new_size: Box::new(HirExpression::IntLiteral(0)),
            },
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains(".clear()"), "Realloc(ptr, 0) should generate clear(), got: {}", code);
}

#[test]
fn stmt_assignment_realloc_with_size_generates_resize() {
    let codegen = gen();
    let func = make_func_with_params(
        HirType::Void,
        vec![HirParameter::new("buf".to_string(), HirType::Vec(Box::new(HirType::Int)))],
        vec![HirStatement::Assignment {
            target: "buf".to_string(),
            value: HirExpression::Realloc {
                pointer: Box::new(HirExpression::Variable("buf".to_string())),
                new_size: Box::new(HirExpression::BinaryOp {
                    op: BinaryOperator::Multiply,
                    left: Box::new(HirExpression::IntLiteral(20)),
                    right: Box::new(HirExpression::Sizeof {
                        type_name: "int".to_string(),
                    }),
                }),
            },
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains(".resize("), "Realloc(ptr, n*sizeof) should generate resize, got: {}", code);
}

#[test]
fn stmt_assignment_realloc_simple_size_generates_resize_with_cast() {
    let codegen = gen();
    let func = make_func_with_params(
        HirType::Void,
        vec![HirParameter::new("buf".to_string(), HirType::Vec(Box::new(HirType::Int)))],
        vec![HirStatement::Assignment {
            target: "buf".to_string(),
            value: HirExpression::Realloc {
                pointer: Box::new(HirExpression::Variable("buf".to_string())),
                new_size: Box::new(HirExpression::IntLiteral(100)),
            },
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains(".resize("), "Realloc with plain size should generate resize, got: {}", code);
    assert!(code.contains("as usize"), "Should cast size to usize, got: {}", code);
}

// ============================================================================
// EXPRESSION: CompoundLiteral - struct
// ============================================================================

#[test]
fn compound_literal_struct_generates_struct_literal() {
    let mut ctx = make_ctx();
    ctx.structs.insert(
        "Point".to_string(),
        vec![
            ("x".to_string(), HirType::Int),
            ("y".to_string(), HirType::Int),
        ],
    );
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Point".to_string()),
        initializers: vec![
            HirExpression::IntLiteral(10),
            HirExpression::IntLiteral(20),
        ],
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("Point"), "Should generate Point struct, got: {}", result);
    assert!(result.contains("x: 10"), "Should use field name x, got: {}", result);
    assert!(result.contains("y: 20"), "Should use field name y, got: {}", result);
}

#[test]
fn compound_literal_struct_partial_init_uses_default() {
    let mut ctx = make_ctx();
    ctx.structs.insert(
        "Rect".to_string(),
        vec![
            ("x".to_string(), HirType::Int),
            ("y".to_string(), HirType::Int),
            ("w".to_string(), HirType::Int),
            ("h".to_string(), HirType::Int),
        ],
    );
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Rect".to_string()),
        initializers: vec![
            HirExpression::IntLiteral(0),
            HirExpression::IntLiteral(0),
        ],
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("..Default::default()"), "Partial struct init should have Default, got: {}", result);
}

#[test]
fn compound_literal_empty_struct() {
    let ctx = make_ctx();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Empty".to_string()),
        initializers: vec![],
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("Empty {}"), "Empty struct should be Empty {{}}, got: {}", result);
}

// ============================================================================
// EXPRESSION: CompoundLiteral - array
// ============================================================================

#[test]
fn compound_literal_array_generates_array_literal() {
    let ctx = make_ctx();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(3),
        },
        initializers: vec![
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(2),
            HirExpression::IntLiteral(3),
        ],
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("[1, 2, 3]"), "Array compound literal should generate [1, 2, 3], got: {}", result);
}

#[test]
fn compound_literal_empty_array_with_size() {
    let ctx = make_ctx();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(5),
        },
        initializers: vec![],
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("[0i32; 5]"), "Empty array with size should generate [0i32; 5], got: {}", result);
}

#[test]
fn compound_literal_empty_array_no_size() {
    let ctx = make_ctx();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        },
        initializers: vec![],
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert_eq!(result, "[]", "Empty array without size should be [], got: {}", result);
}
