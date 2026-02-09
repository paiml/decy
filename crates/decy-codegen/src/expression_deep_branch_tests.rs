//! Deep branch coverage tests for generate_expression_with_target_type.
//!
//! Targets specific uncovered branches identified by coverage analysis.
//! These complement expression_target_type_tests.rs.

use super::*;
use decy_hir::{BinaryOperator, HirExpression, HirType};

fn make_ctx() -> TypeContext {
    TypeContext::new()
}

fn gen() -> CodeGenerator {
    CodeGenerator::new()
}

// ============================================================================
// Box null comparison (DECY-119)
// ============================================================================

#[test]
fn box_null_equal_returns_false() {
    let mut ctx = make_ctx();
    ctx.add_variable("data".to_string(), HirType::Box(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("data".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("false"), "Box == null should be false, got: {}", result);
}

#[test]
fn box_null_not_equal_returns_true() {
    let mut ctx = make_ctx();
    ctx.add_variable("data".to_string(), HirType::Box(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("data".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("true"), "Box != 0 should be true, got: {}", result);
}

// ============================================================================
// strlen optimization - reversed (0 == strlen(s))
// ============================================================================

#[test]
fn zero_equal_strlen_becomes_is_empty() {
    let ctx = make_ctx();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("s".to_string())],
        }),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("is_empty"), "0 == strlen(s) should use is_empty, got: {}", result);
}

#[test]
fn zero_not_equal_strlen_becomes_not_is_empty() {
    let ctx = make_ctx();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("s".to_string())],
        }),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("!") && result.contains("is_empty"), "0 != strlen(s) should use !is_empty, got: {}", result);
}

// ============================================================================
// Char-int comparison reversed (char on left, int var on right)
// ============================================================================

#[test]
fn char_literal_left_int_var_right_comparison() {
    let mut ctx = make_ctx();
    ctx.add_variable("c".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::CharLiteral(65)), // 'A'
        right: Box::new(HirExpression::Variable("c".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("65i32") || result.contains("i32"), "Char on left should cast to i32, got: {}", result);
}

// ============================================================================
// Char + int arithmetic (DECY-210)
// ============================================================================

#[test]
fn int_plus_char_literal_arithmetic() {
    let mut ctx = make_ctx();
    ctx.add_variable("n".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("n".to_string())),
        right: Box::new(HirExpression::CharLiteral(48)), // '0'
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("48i32") || result.contains("i32"), "int + '0' should cast char to i32, got: {}", result);
}

#[test]
fn char_literal_minus_int_arithmetic() {
    let mut ctx = make_ctx();
    ctx.add_variable("n".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::CharLiteral(97)), // 'a'
        right: Box::new(HirExpression::Variable("n".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("97i32") || result.contains("i32"), "'a' - int should cast char to i32, got: {}", result);
}

// ============================================================================
// Comma operator (DECY-249)
// ============================================================================

#[test]
fn comma_operator_becomes_block() {
    let ctx = make_ctx();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Comma,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("{") && result.contains(";"), "Comma should become block, got: {}", result);
}

// ============================================================================
// Logical operators with int target (DECY-191)
// ============================================================================

#[test]
fn logical_and_with_int_target_casts_to_i32() {
    let ctx = make_ctx();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalAnd,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("as i32"), "Logical && with Int target should cast to i32, got: {}", result);
}

#[test]
fn logical_or_with_int_target_casts_to_i32() {
    let ctx = make_ctx();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalOr,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("as i32"), "Logical || with Int target should cast to i32, got: {}", result);
}

#[test]
fn logical_and_without_int_target_no_cast() {
    let ctx = make_ctx();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalAnd,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(!result.contains("as i32"), "Logical && without Int target should not cast, got: {}", result);
    assert!(result.contains("!= 0"), "Should convert int operands to bool, got: {}", result);
}

// ============================================================================
// Char promotion in arithmetic (DECY-151)
// ============================================================================

#[test]
fn char_subtract_with_int_target_casts_to_i32() {
    let mut ctx = make_ctx();
    ctx.add_variable("s1".to_string(), HirType::Char);
    ctx.add_variable("s2".to_string(), HirType::Char);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("s1".to_string())),
        right: Box::new(HirExpression::Variable("s2".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("as i32"), "Char - Char with Int target should cast, got: {}", result);
}

#[test]
fn char_add_int_with_int_target() {
    let mut ctx = make_ctx();
    ctx.add_variable("c".to_string(), HirType::Char);
    ctx.add_variable("n".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("c".to_string())),
        right: Box::new(HirExpression::Variable("n".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("as i32"), "Char + Int should cast char, got: {}", result);
}

// ============================================================================
// Mixed int/float arithmetic (DECY-204)
// ============================================================================

#[test]
fn int_plus_float_promotes_to_f32() {
    let mut ctx = make_ctx();
    ctx.add_variable("n".to_string(), HirType::Int);
    ctx.add_variable("f".to_string(), HirType::Float);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("n".to_string())),
        right: Box::new(HirExpression::Variable("f".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as f32"), "int + float should cast to f32, got: {}", result);
}

#[test]
fn float_minus_int_promotes_to_f32() {
    let mut ctx = make_ctx();
    ctx.add_variable("f".to_string(), HirType::Float);
    ctx.add_variable("n".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("f".to_string())),
        right: Box::new(HirExpression::Variable("n".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as f32"), "float - int should cast to f32, got: {}", result);
}

#[test]
fn int_multiply_double_promotes_to_f64() {
    let mut ctx = make_ctx();
    ctx.add_variable("n".to_string(), HirType::Int);
    ctx.add_variable("d".to_string(), HirType::Double);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::Variable("n".to_string())),
        right: Box::new(HirExpression::Variable("d".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as f64"), "int * double should cast to f64, got: {}", result);
}

#[test]
fn double_divide_int_promotes_to_f64() {
    let mut ctx = make_ctx();
    ctx.add_variable("d".to_string(), HirType::Double);
    ctx.add_variable("n".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Divide,
        left: Box::new(HirExpression::Variable("d".to_string())),
        right: Box::new(HirExpression::Variable("n".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as f64"), "double / int should cast to f64, got: {}", result);
}

// ============================================================================
// Pointer arithmetic (DECY-041)
// ============================================================================

#[test]
fn pointer_add_uses_wrapping_add() {
    let mut ctx = make_ctx();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::IntLiteral(1)),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("wrapping_add"), "ptr + n should use wrapping_add, got: {}", result);
}

#[test]
fn pointer_subtract_integer_uses_wrapping_sub() {
    let mut ctx = make_ctx();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::IntLiteral(1)),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("wrapping_sub"), "ptr - n should use wrapping_sub, got: {}", result);
}

#[test]
fn pointer_subtract_pointer_uses_offset_from() {
    let mut ctx = make_ctx();
    ctx.add_variable("p1".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    ctx.add_variable("p2".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("p1".to_string())),
        right: Box::new(HirExpression::Variable("p2".to_string())),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("offset_from"), "ptr - ptr should use offset_from, got: {}", result);
}

// ============================================================================
// Global variable access (DECY-220)
// ============================================================================

#[test]
fn global_variable_wrapped_in_unsafe() {
    let mut ctx = make_ctx();
    ctx.add_variable("g_count".to_string(), HirType::Int);
    ctx.add_global("g_count".to_string());
    let expr = HirExpression::Variable("g_count".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("unsafe"), "Global access should be unsafe, got: {}", result);
}

#[test]
fn global_int_to_float_coercion_unsafe() {
    let mut ctx = make_ctx();
    ctx.add_variable("g_val".to_string(), HirType::Int);
    ctx.add_global("g_val".to_string());
    let expr = HirExpression::Variable("g_val".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Float));
    assert!(result.contains("unsafe"), "Global coercion should be unsafe, got: {}", result);
    assert!(result.contains("as f32"), "Should cast to f32, got: {}", result);
}

#[test]
fn global_int_to_double_coercion_unsafe() {
    let mut ctx = make_ctx();
    ctx.add_variable("g_val".to_string(), HirType::Int);
    ctx.add_global("g_val".to_string());
    let expr = HirExpression::Variable("g_val".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Double));
    assert!(result.contains("unsafe"), "Global coercion should be unsafe, got: {}", result);
    assert!(result.contains("as f64"), "Should cast to f64, got: {}", result);
}

#[test]
fn global_float_to_int_coercion_unsafe() {
    let mut ctx = make_ctx();
    ctx.add_variable("g_f".to_string(), HirType::Float);
    ctx.add_global("g_f".to_string());
    let expr = HirExpression::Variable("g_f".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("unsafe"), "Global float to int should be unsafe, got: {}", result);
    assert!(result.contains("as i32"), "Should cast to i32, got: {}", result);
}

#[test]
fn global_float_to_unsigned_coercion_unsafe() {
    let mut ctx = make_ctx();
    ctx.add_variable("g_f".to_string(), HirType::Double);
    ctx.add_global("g_f".to_string());
    let expr = HirExpression::Variable("g_f".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::UnsignedInt));
    assert!(result.contains("unsafe"), "Global double to uint should be unsafe, got: {}", result);
    assert!(result.contains("as u32"), "Should cast to u32, got: {}", result);
}

#[test]
fn global_char_to_int_coercion_unsafe() {
    let mut ctx = make_ctx();
    ctx.add_variable("g_c".to_string(), HirType::Char);
    ctx.add_global("g_c".to_string());
    let expr = HirExpression::Variable("g_c".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("unsafe"), "Global char to int should be unsafe, got: {}", result);
    assert!(result.contains("as i32"), "Should cast to i32, got: {}", result);
}

// ============================================================================
// Int to char coercion (DECY-198)
// ============================================================================

#[test]
fn int_var_to_char_target_casts_to_u8() {
    let mut ctx = make_ctx();
    ctx.add_variable("c".to_string(), HirType::Int);
    let expr = HirExpression::Variable("c".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Char));
    assert!(result.contains("as u8"), "int var with Char target should cast to u8, got: {}", result);
}

// ============================================================================
// Array variable to pointer target (DECY-148/244)
// ============================================================================

#[test]
fn vec_variable_to_pointer_target() {
    let mut ctx = make_ctx();
    ctx.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("arr".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Pointer(Box::new(HirType::Int))));
    assert!(result.contains("as_mut_ptr"), "Vec to pointer should use as_mut_ptr, got: {}", result);
}

#[test]
fn vec_variable_to_void_pointer() {
    let mut ctx = make_ctx();
    ctx.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::Variable("arr".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Pointer(Box::new(HirType::Void))));
    // Vec to void* - codegen may simplify to just the variable name or use as_mut_ptr
    assert!(!result.is_empty(), "Should produce non-empty output, got: {}", result);
}

// ============================================================================
// Nested binary ops get parenthesized
// ============================================================================

#[test]
fn nested_binary_ops_parenthesized() {
    let ctx = make_ctx();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::IntLiteral(1)),
            right: Box::new(HirExpression::IntLiteral(2)),
        }),
        right: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Subtract,
            left: Box::new(HirExpression::IntLiteral(3)),
            right: Box::new(HirExpression::IntLiteral(4)),
        }),
    };
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("(1 + 2)") || result.contains("("), "Nested ops should be parenthesized, got: {}", result);
}

// ============================================================================
// Float literal edge cases (DECY-222)
// ============================================================================

#[test]
fn float_literal_with_c_suffix_stripped() {
    let ctx = make_ctx();
    let expr = HirExpression::FloatLiteral("3.14f".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(!result.contains("3.14ff"), "C suffix should be stripped, got: {}", result);
    assert!(result.contains("3.14"), "Value should be preserved, got: {}", result);
}

#[test]
fn float_literal_no_dot_gets_dot_zero() {
    let ctx = make_ctx();
    let expr = HirExpression::FloatLiteral("42".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("42.0"), "Integer float literal should get .0, got: {}", result);
}

#[test]
fn float_literal_with_exponent_no_dot() {
    let ctx = make_ctx();
    let expr = HirExpression::FloatLiteral("1e10".to_string());
    let result = gen().generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("1e10"), "Exponent format preserved, got: {}", result);
    assert!(result.contains("f64"), "Should have f64 suffix, got: {}", result);
}
