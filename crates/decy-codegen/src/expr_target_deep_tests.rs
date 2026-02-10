//! Deep coverage tests for `generate_expression_with_target_type` and
//! `generate_statement_with_context` -- targeting the biggest uncovered branches.
//!
//! Focus areas:
//! - Variable with target type coercions (Box→raw, Ref→raw, Vec→raw, Array→raw)
//! - Numeric type coercions (int→float, float→int, char→int)
//! - BinaryOp branches: Option/pointer null checks, Vec/Box null checks,
//!   strlen==0, char-int comparisons, char+int arithmetic, comma operator,
//!   pointer arithmetic, logical operators with int operands, char promotion,
//!   mixed float/int arithmetic, chained comparisons, signed/unsigned mismatch,
//!   bitwise with boolean operands, arithmetic result cast to float target
//! - Statement branches: VLA declarations, malloc struct/array, return in main,
//!   return with void, if/while with pointer condition, switch with char cases,
//!   deref assignment, realloc assignment, for loop, global variable assignment,
//!   renamed locals shadowing globals, errno assignment

use super::*;
use decy_hir::{
    BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType, SwitchCase,
    UnaryOperator,
};

// ============================================================================
// Helpers
// ============================================================================

fn cg() -> CodeGenerator {
    CodeGenerator::new()
}

fn ctx() -> TypeContext {
    TypeContext::new()
}

fn void_func(stmts: Vec<HirStatement>) -> HirFunction {
    HirFunction::new_with_body("test_func".to_string(), HirType::Void, vec![], stmts)
}

fn typed_func(ret: HirType, params: Vec<HirParameter>, stmts: Vec<HirStatement>) -> HirFunction {
    HirFunction::new_with_body("test_func".to_string(), ret, params, stmts)
}

fn main_func(stmts: Vec<HirStatement>) -> HirFunction {
    HirFunction::new_with_body("main".to_string(), HirType::Int, vec![], stmts)
}

fn int_var(name: &str) -> HirExpression {
    HirExpression::Variable(name.to_string())
}

fn int_lit(v: i32) -> HirExpression {
    HirExpression::IntLiteral(v)
}

fn binop(op: BinaryOperator, l: HirExpression, r: HirExpression) -> HirExpression {
    HirExpression::BinaryOp {
        op,
        left: Box::new(l),
        right: Box::new(r),
    }
}

// ============================================================================
// 1. Variable with Vec target type returns directly
// ============================================================================

#[test]
fn variable_with_vec_target_returns_directly() {
    let mut c = ctx();
    c.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let result = cg().generate_expression_with_target_type(
        &int_var("arr"),
        &c,
        Some(&HirType::Vec(Box::new(HirType::Int))),
    );
    assert_eq!(result, "arr");
}

// ============================================================================
// 2. Variable with Pointer target and Box type → Box::into_raw
// ============================================================================

#[test]
fn variable_box_to_raw_pointer_uses_into_raw() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Box(Box::new(HirType::Int)));
    let result = cg().generate_expression_with_target_type(
        &int_var("p"),
        &c,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(
        result.contains("Box::into_raw(p)"),
        "Expected Box::into_raw, got: {}",
        result
    );
}

// ============================================================================
// 3. Reference mutable slice → as_mut_ptr()
// ============================================================================

#[test]
fn mutable_ref_slice_to_pointer_uses_as_mut_ptr() {
    let mut c = ctx();
    c.add_variable(
        "s".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Vec(Box::new(HirType::Int))),
            mutable: true,
        },
    );
    let result = cg().generate_expression_with_target_type(
        &int_var("s"),
        &c,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(
        result.contains("as_mut_ptr"),
        "Expected as_mut_ptr, got: {}",
        result
    );
}

// ============================================================================
// 4. Immutable ref slice → as_ptr() with cast
// ============================================================================

#[test]
fn immutable_ref_slice_to_pointer_uses_as_ptr() {
    let mut c = ctx();
    c.add_variable(
        "s".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Vec(Box::new(HirType::Int))),
            mutable: false,
        },
    );
    let result = cg().generate_expression_with_target_type(
        &int_var("s"),
        &c,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(
        result.contains("as_ptr"),
        "Expected as_ptr, got: {}",
        result
    );
}

// ============================================================================
// 5. Mutable single reference to pointer
// ============================================================================

#[test]
fn mutable_ref_single_to_pointer_casts() {
    let mut c = ctx();
    c.add_variable(
        "r".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        },
    );
    let result = cg().generate_expression_with_target_type(
        &int_var("r"),
        &c,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(
        result.contains("as *mut _"),
        "Expected *mut cast, got: {}",
        result
    );
}

// ============================================================================
// 6. Immutable single reference to pointer
// ============================================================================

#[test]
fn immutable_ref_single_to_pointer_double_casts() {
    let mut c = ctx();
    c.add_variable(
        "r".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
    );
    let result = cg().generate_expression_with_target_type(
        &int_var("r"),
        &c,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(
        result.contains("as *const _ as *mut _"),
        "Expected double cast, got: {}",
        result
    );
}

// ============================================================================
// 7. Vec<T> to *mut T → as_mut_ptr()
// ============================================================================

#[test]
fn vec_to_pointer_uses_as_mut_ptr() {
    let mut c = ctx();
    c.add_variable("v".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let result = cg().generate_expression_with_target_type(
        &int_var("v"),
        &c,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(
        result.contains("as_mut_ptr"),
        "Expected as_mut_ptr, got: {}",
        result
    );
}

// ============================================================================
// 8. Array to *mut T → as_mut_ptr()
// ============================================================================

#[test]
fn array_to_pointer_uses_as_mut_ptr() {
    let mut c = ctx();
    c.add_variable(
        "a".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    let result = cg().generate_expression_with_target_type(
        &int_var("a"),
        &c,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(
        result.contains("as_mut_ptr"),
        "Expected as_mut_ptr, got: {}",
        result
    );
}

// ============================================================================
// 9. Array to *mut void
// ============================================================================

#[test]
fn array_to_void_pointer_casts() {
    let mut c = ctx();
    c.add_variable(
        "a".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(5),
        },
    );
    let result = cg().generate_expression_with_target_type(
        &int_var("a"),
        &c,
        Some(&HirType::Pointer(Box::new(HirType::Void))),
    );
    assert!(
        result.contains("as *mut ()"),
        "Expected void pointer cast, got: {}",
        result
    );
}

// ============================================================================
// 10. Pointer to pointer - stays as-is
// ============================================================================

#[test]
fn pointer_to_pointer_returns_directly() {
    let mut c = ctx();
    c.add_variable(
        "p".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let result = cg().generate_expression_with_target_type(
        &int_var("p"),
        &c,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    // Raw pointer stays as raw pointer
    assert_eq!(result, "p");
}

// ============================================================================
// 11. Int variable to Char target → as u8
// ============================================================================

#[test]
fn int_to_char_target_casts_to_u8() {
    let mut c = ctx();
    c.add_variable("c".to_string(), HirType::Int);
    let result = cg().generate_expression_with_target_type(
        &int_var("c"),
        &c,
        Some(&HirType::Char),
    );
    assert!(
        result.contains("as u8"),
        "Expected as u8 cast, got: {}",
        result
    );
}

// ============================================================================
// 12. Int to Float target → as f32
// ============================================================================

#[test]
fn int_to_float_target_casts_to_f32() {
    let mut c = ctx();
    c.add_variable("n".to_string(), HirType::Int);
    let result = cg().generate_expression_with_target_type(
        &int_var("n"),
        &c,
        Some(&HirType::Float),
    );
    assert!(
        result.contains("as f32"),
        "Expected as f32 cast, got: {}",
        result
    );
}

// ============================================================================
// 13. Int to Double target → as f64
// ============================================================================

#[test]
fn int_to_double_target_casts_to_f64() {
    let mut c = ctx();
    c.add_variable("n".to_string(), HirType::Int);
    let result = cg().generate_expression_with_target_type(
        &int_var("n"),
        &c,
        Some(&HirType::Double),
    );
    assert!(
        result.contains("as f64"),
        "Expected as f64 cast, got: {}",
        result
    );
}

// ============================================================================
// 14. Float to Int target → as i32
// ============================================================================

#[test]
fn float_to_int_target_casts_to_i32() {
    let mut c = ctx();
    c.add_variable("f".to_string(), HirType::Float);
    let result = cg().generate_expression_with_target_type(
        &int_var("f"),
        &c,
        Some(&HirType::Int),
    );
    assert!(
        result.contains("as i32"),
        "Expected as i32 cast, got: {}",
        result
    );
}

// ============================================================================
// 15. Double to UnsignedInt target → as u32
// ============================================================================

#[test]
fn double_to_unsigned_int_target_casts_to_u32() {
    let mut c = ctx();
    c.add_variable("d".to_string(), HirType::Double);
    let result = cg().generate_expression_with_target_type(
        &int_var("d"),
        &c,
        Some(&HirType::UnsignedInt),
    );
    assert!(
        result.contains("as u32"),
        "Expected as u32 cast, got: {}",
        result
    );
}

// ============================================================================
// 16. Char to Int target → as i32
// ============================================================================

#[test]
fn char_to_int_target_casts_to_i32() {
    let mut c = ctx();
    c.add_variable("ch".to_string(), HirType::Char);
    let result = cg().generate_expression_with_target_type(
        &int_var("ch"),
        &c,
        Some(&HirType::Int),
    );
    assert!(
        result.contains("as i32"),
        "Expected as i32 cast, got: {}",
        result
    );
}

// ============================================================================
// 17. Global variable access wraps in unsafe
// ============================================================================

#[test]
fn global_variable_access_wraps_unsafe() {
    let mut c = ctx();
    c.add_variable("g".to_string(), HirType::Int);
    c.add_global("g".to_string());
    let result = cg().generate_expression_with_target_type(&int_var("g"), &c, None);
    assert!(
        result.contains("unsafe"),
        "Expected unsafe wrapper, got: {}",
        result
    );
}

// ============================================================================
// 18. Global int to float wraps in unsafe
// ============================================================================

#[test]
fn global_int_to_float_wraps_unsafe() {
    let mut c = ctx();
    c.add_variable("g".to_string(), HirType::Int);
    c.add_global("g".to_string());
    let result = cg().generate_expression_with_target_type(
        &int_var("g"),
        &c,
        Some(&HirType::Float),
    );
    assert!(
        result.contains("unsafe"),
        "Expected unsafe for global, got: {}",
        result
    );
    assert!(
        result.contains("as f32"),
        "Expected f32 cast, got: {}",
        result
    );
}

// ============================================================================
// 19. BinaryOp Assign → embedded assignment block
// ============================================================================

#[test]
fn binary_assign_generates_tmp_block() {
    let c = ctx();
    let expr = binop(
        BinaryOperator::Assign,
        int_var("x"),
        int_lit(42),
    );
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("__assign_tmp"),
        "Expected assignment tmp block, got: {}",
        result
    );
}

// ============================================================================
// 20. Option variable == NULL → is_none()
// ============================================================================

#[test]
fn option_eq_null_becomes_is_none() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = binop(
        BinaryOperator::Equal,
        int_var("p"),
        HirExpression::NullLiteral,
    );
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("is_none"),
        "Expected is_none, got: {}",
        result
    );
}

// ============================================================================
// 21. Option variable != NULL → is_some()
// ============================================================================

#[test]
fn option_ne_null_becomes_is_some() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = binop(
        BinaryOperator::NotEqual,
        int_var("p"),
        HirExpression::NullLiteral,
    );
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("is_some"),
        "Expected is_some, got: {}",
        result
    );
}

// ============================================================================
// 22. NULL == Option → is_none() (reversed)
// ============================================================================

#[test]
fn null_eq_option_reversed_becomes_is_none() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = binop(
        BinaryOperator::Equal,
        HirExpression::NullLiteral,
        int_var("p"),
    );
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("is_none"),
        "Expected is_none, got: {}",
        result
    );
}

// ============================================================================
// 23. Pointer variable == 0 → ptr == null_mut()
// ============================================================================

#[test]
fn pointer_eq_zero_becomes_null_mut_comparison() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = binop(BinaryOperator::Equal, int_var("p"), int_lit(0));
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("std::ptr::null_mut()"),
        "Expected null_mut comparison, got: {}",
        result
    );
}

// ============================================================================
// 24. 0 == pointer (reversed) → null_mut comparison
// ============================================================================

#[test]
fn zero_eq_pointer_reversed_becomes_null_mut() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = binop(BinaryOperator::Equal, int_lit(0), int_var("p"));
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("std::ptr::null_mut()"),
        "Expected null_mut comparison, got: {}",
        result
    );
}

// ============================================================================
// 25. Vec == 0 → false (Vec never null)
// ============================================================================

#[test]
fn vec_eq_zero_becomes_false() {
    let mut c = ctx();
    c.add_variable("v".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = binop(BinaryOperator::Equal, int_var("v"), int_lit(0));
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("false"),
        "Expected false for Vec null check, got: {}",
        result
    );
}

// ============================================================================
// 26. Vec != NULL → true (Vec never null)
// ============================================================================

#[test]
fn vec_ne_null_becomes_true() {
    let mut c = ctx();
    c.add_variable("v".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = binop(
        BinaryOperator::NotEqual,
        int_var("v"),
        HirExpression::NullLiteral,
    );
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("true"),
        "Expected true for Vec null check, got: {}",
        result
    );
}

// ============================================================================
// 27. Box == 0 → false (Box never null)
// ============================================================================

#[test]
fn box_eq_zero_becomes_false() {
    let mut c = ctx();
    c.add_variable("b".to_string(), HirType::Box(Box::new(HirType::Int)));
    let expr = binop(BinaryOperator::Equal, int_var("b"), int_lit(0));
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("false"),
        "Expected false for Box null check, got: {}",
        result
    );
}

// ============================================================================
// 28. strlen(s) == 0 → s.is_empty()
// ============================================================================

#[test]
fn strlen_eq_zero_becomes_is_empty() {
    let c = ctx();
    let expr = binop(
        BinaryOperator::Equal,
        HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![int_var("s")],
        },
        int_lit(0),
    );
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("is_empty"),
        "Expected is_empty, got: {}",
        result
    );
}

// ============================================================================
// 29. 0 != strlen(s) → !s.is_empty()
// ============================================================================

#[test]
fn zero_ne_strlen_becomes_not_is_empty() {
    let c = ctx();
    let expr = binop(
        BinaryOperator::NotEqual,
        int_lit(0),
        HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![int_var("s")],
        },
    );
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("!s.is_empty()"),
        "Expected !s.is_empty(), got: {}",
        result
    );
}

// ============================================================================
// 30. Char literal comparison with int var → i32 literal
// ============================================================================

#[test]
fn int_var_compared_to_char_literal_uses_i32() {
    let mut c = ctx();
    c.add_variable("c".to_string(), HirType::Int);
    let expr = binop(
        BinaryOperator::NotEqual,
        int_var("c"),
        HirExpression::CharLiteral(b'\n' as i8),
    );
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("10i32"),
        "Expected 10i32 for '\\n' comparison, got: {}",
        result
    );
}

// ============================================================================
// 31. Char literal on left compared to int var
// ============================================================================

#[test]
fn char_literal_compared_to_int_var_reversed() {
    let mut c = ctx();
    c.add_variable("c".to_string(), HirType::Int);
    let expr = binop(
        BinaryOperator::Equal,
        HirExpression::CharLiteral(b'A' as i8),
        int_var("c"),
    );
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("65i32"),
        "Expected 65i32 for 'A' comparison, got: {}",
        result
    );
}

// ============================================================================
// 32. Integer + char literal arithmetic
// ============================================================================

#[test]
fn int_plus_char_literal_uses_i32_cast() {
    let mut c = ctx();
    c.add_variable("n".to_string(), HirType::Int);
    let expr = binop(
        BinaryOperator::Add,
        int_var("n"),
        HirExpression::CharLiteral(b'0' as i8),
    );
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("48i32"),
        "Expected 48i32 for '0', got: {}",
        result
    );
}

// ============================================================================
// 33. Char literal - int (reversed)
// ============================================================================

#[test]
fn char_literal_minus_int_uses_i32_cast() {
    let mut c = ctx();
    c.add_variable("n".to_string(), HirType::Int);
    let expr = binop(
        BinaryOperator::Subtract,
        HirExpression::CharLiteral(b'9' as i8),
        int_var("n"),
    );
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("57i32"),
        "Expected 57i32 for '9', got: {}",
        result
    );
}

// ============================================================================
// 34. Comma operator → block expression
// ============================================================================

#[test]
fn comma_operator_generates_block() {
    let c = ctx();
    let expr = binop(BinaryOperator::Comma, int_lit(1), int_lit(2));
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("{ 1; 2 }"),
        "Expected block expression, got: {}",
        result
    );
}

// ============================================================================
// 35. Pointer addition → wrapping_add
// ============================================================================

#[test]
fn pointer_add_uses_wrapping_add() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = binop(BinaryOperator::Add, int_var("p"), int_lit(3));
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("wrapping_add"),
        "Expected wrapping_add, got: {}",
        result
    );
}

// ============================================================================
// 36. Pointer subtraction → wrapping_sub
// ============================================================================

#[test]
fn pointer_sub_integer_uses_wrapping_sub() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = binop(BinaryOperator::Subtract, int_var("p"), int_lit(1));
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("wrapping_sub"),
        "Expected wrapping_sub, got: {}",
        result
    );
}

// ============================================================================
// 37. Pointer - pointer → offset_from
// ============================================================================

#[test]
fn pointer_minus_pointer_uses_offset_from() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    c.add_variable("q".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = binop(BinaryOperator::Subtract, int_var("p"), int_var("q"));
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("offset_from"),
        "Expected offset_from, got: {}",
        result
    );
}

// ============================================================================
// 38. Logical AND with int operands → != 0 coercion
// ============================================================================

#[test]
fn logical_and_with_int_operands_adds_bool_coercion() {
    let mut c = ctx();
    c.add_variable("a".to_string(), HirType::Int);
    c.add_variable("b".to_string(), HirType::Int);
    let expr = binop(BinaryOperator::LogicalAnd, int_var("a"), int_var("b"));
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("!= 0"),
        "Expected != 0 coercion, got: {}",
        result
    );
    assert!(
        result.contains("&&"),
        "Expected && operator, got: {}",
        result
    );
}

// ============================================================================
// 39. Logical OR with int target → as i32 cast
// ============================================================================

#[test]
fn logical_or_with_int_target_casts_result() {
    let mut c = ctx();
    c.add_variable("a".to_string(), HirType::Int);
    c.add_variable("b".to_string(), HirType::Int);
    let expr = binop(BinaryOperator::LogicalOr, int_var("a"), int_var("b"));
    let result = cg().generate_expression_with_target_type(&expr, &c, Some(&HirType::Int));
    assert!(
        result.contains("as i32"),
        "Expected as i32 for logical op with int target, got: {}",
        result
    );
}

// ============================================================================
// 40. Char promotion in arithmetic with Int target
// ============================================================================

#[test]
fn char_subtraction_with_int_target_promotes() {
    let mut c = ctx();
    c.add_variable("a".to_string(), HirType::Char);
    c.add_variable("b".to_string(), HirType::Char);
    let expr = binop(BinaryOperator::Subtract, int_var("a"), int_var("b"));
    let result = cg().generate_expression_with_target_type(&expr, &c, Some(&HirType::Int));
    assert!(
        result.contains("as i32"),
        "Expected i32 promotion, got: {}",
        result
    );
}

// ============================================================================
// 41. Mixed int + float arithmetic → f32 cast
// ============================================================================

#[test]
fn int_plus_float_casts_int_to_f32() {
    let mut c = ctx();
    c.add_variable("n".to_string(), HirType::Int);
    c.add_variable("f".to_string(), HirType::Float);
    let expr = binop(BinaryOperator::Add, int_var("n"), int_var("f"));
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("as f32"),
        "Expected f32 promotion, got: {}",
        result
    );
}

// ============================================================================
// 42. Mixed int + double arithmetic → f64 cast
// ============================================================================

#[test]
fn int_plus_double_casts_int_to_f64() {
    let mut c = ctx();
    c.add_variable("n".to_string(), HirType::Int);
    c.add_variable("d".to_string(), HirType::Double);
    let expr = binop(BinaryOperator::Multiply, int_var("n"), int_var("d"));
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("as f64"),
        "Expected f64 promotion, got: {}",
        result
    );
}

// ============================================================================
// 43. Mixed float + double → f64 cast
// ============================================================================

#[test]
fn float_plus_double_promotes_float_to_f64() {
    let mut c = ctx();
    c.add_variable("f".to_string(), HirType::Float);
    c.add_variable("d".to_string(), HirType::Double);
    let expr = binop(BinaryOperator::Add, int_var("f"), int_var("d"));
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("as f64"),
        "Expected f64 promotion, got: {}",
        result
    );
}

// ============================================================================
// 44. Comparison returns bool, cast to i32 with Int target
// ============================================================================

#[test]
fn comparison_with_int_target_casts_to_i32() {
    let c = ctx();
    let expr = binop(BinaryOperator::GreaterThan, int_lit(5), int_lit(3));
    let result = cg().generate_expression_with_target_type(&expr, &c, Some(&HirType::Int));
    assert!(
        result.contains("as i32"),
        "Expected as i32 for comparison result, got: {}",
        result
    );
}

// ============================================================================
// 45. Chained comparison: (x < y) < z
// ============================================================================

#[test]
fn chained_comparison_casts_inner_to_i32() {
    let mut c = ctx();
    c.add_variable("x".to_string(), HirType::Int);
    c.add_variable("y".to_string(), HirType::Int);
    c.add_variable("z".to_string(), HirType::Int);
    let inner = binop(BinaryOperator::LessThan, int_var("x"), int_var("y"));
    let expr = binop(BinaryOperator::LessThan, inner, int_var("z"));
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    // The inner comparison should be cast to i32 before outer comparison
    assert!(
        result.contains("as i32)"),
        "Expected inner comparison cast to i32, got: {}",
        result
    );
}

// ============================================================================
// 46. Signed/unsigned comparison mismatch → i64 cast
// ============================================================================

#[test]
fn signed_unsigned_comparison_casts_to_i64() {
    let mut c = ctx();
    c.add_variable("s".to_string(), HirType::Int);
    c.add_variable("u".to_string(), HirType::UnsignedInt);
    let expr = binop(BinaryOperator::LessThan, int_var("s"), int_var("u"));
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("as i64"),
        "Expected i64 cast for mixed sign, got: {}",
        result
    );
}

// ============================================================================
// 47. Int arithmetic result with Float target → as f32
// ============================================================================

#[test]
fn int_arithmetic_with_float_target_casts_result() {
    let mut c = ctx();
    c.add_variable("a".to_string(), HirType::Int);
    c.add_variable("b".to_string(), HirType::Int);
    let expr = binop(BinaryOperator::Divide, int_var("a"), int_var("b"));
    let result = cg().generate_expression_with_target_type(&expr, &c, Some(&HirType::Float));
    assert!(
        result.contains("as f32"),
        "Expected as f32 cast of result, got: {}",
        result
    );
}

// ============================================================================
// 48. Bitwise AND with bool operand → i32 cast
// ============================================================================

#[test]
fn bitwise_and_with_bool_casts_to_i32() {
    let mut c = ctx();
    c.add_variable("x".to_string(), HirType::Int);
    // Inner comparison produces bool
    let cmp = binop(BinaryOperator::Equal, int_var("x"), int_lit(1));
    let expr = binop(BinaryOperator::BitwiseAnd, int_lit(3), cmp);
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("as i32"),
        "Expected i32 cast for bool in bitwise, got: {}",
        result
    );
}

// ============================================================================
// 49. LogicalNot on boolean with Int target → (!expr) as i32
// ============================================================================

#[test]
fn logical_not_bool_with_int_target_casts() {
    let mut c = ctx();
    c.add_variable("x".to_string(), HirType::Int);
    let cmp = binop(BinaryOperator::GreaterThan, int_var("x"), int_lit(0));
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(cmp),
    };
    let result = cg().generate_expression_with_target_type(&expr, &c, Some(&HirType::Int));
    assert!(
        result.contains("as i32"),
        "Expected as i32, got: {}",
        result
    );
}

// ============================================================================
// 50. LogicalNot on int (non-bool) with Int target → (x == 0) as i32
// ============================================================================

#[test]
fn logical_not_int_with_int_target_uses_eq_zero() {
    let mut c = ctx();
    c.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(int_var("x")),
    };
    let result = cg().generate_expression_with_target_type(&expr, &c, Some(&HirType::Int));
    assert!(
        result.contains("== 0"),
        "Expected == 0 pattern, got: {}",
        result
    );
    assert!(
        result.contains("as i32"),
        "Expected as i32, got: {}",
        result
    );
}

// ============================================================================
// 51. LogicalNot on bool without target → !expr
// ============================================================================

#[test]
fn logical_not_bool_without_target_is_plain_not() {
    let mut c = ctx();
    c.add_variable("x".to_string(), HirType::Int);
    let cmp = binop(BinaryOperator::GreaterThan, int_var("x"), int_lit(0));
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(cmp),
    };
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.starts_with('!'),
        "Expected plain !, got: {}",
        result
    );
    assert!(
        !result.contains("as i32"),
        "Should not have i32 cast without target, got: {}",
        result
    );
}

// ============================================================================
// 52. StringLiteral with Pointer(Char) target → byte string
// ============================================================================

#[test]
fn string_literal_to_char_pointer_generates_byte_string() {
    let c = ctx();
    let expr = HirExpression::StringLiteral("hello".to_string());
    let result = cg().generate_expression_with_target_type(
        &expr,
        &c,
        Some(&HirType::Pointer(Box::new(HirType::Char))),
    );
    assert!(
        result.contains("b\"hello\\0\""),
        "Expected byte string, got: {}",
        result
    );
    assert!(
        result.contains("as_ptr"),
        "Expected as_ptr call, got: {}",
        result
    );
}

// ============================================================================
// 53. CharLiteral '\0' → 0u8
// ============================================================================

#[test]
fn char_literal_null_becomes_zero_u8() {
    let c = ctx();
    let expr = HirExpression::CharLiteral(0);
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert_eq!(result, "0u8");
}

// ============================================================================
// 54. CharLiteral printable → b'x'
// ============================================================================

#[test]
fn char_literal_printable_becomes_byte_literal() {
    let c = ctx();
    let expr = HirExpression::CharLiteral(b'x' as i8);
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert_eq!(result, "b'x'");
}

// ============================================================================
// 55. CharLiteral non-printable → Xu8
// ============================================================================

#[test]
fn char_literal_nonprintable_becomes_numeric_u8() {
    let c = ctx();
    let expr = HirExpression::CharLiteral(7); // BEL character
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert_eq!(result, "7u8");
}

// ============================================================================
// 56. Variable mapping: stderr/stdin/stdout/errno
// ============================================================================

#[test]
fn stderr_maps_to_std_io() {
    let c = ctx();
    let result = cg().generate_expression_with_target_type(
        &int_var("stderr"),
        &c,
        None,
    );
    assert_eq!(result, "std::io::stderr()");
}

#[test]
fn stdin_maps_to_std_io() {
    let c = ctx();
    let result = cg().generate_expression_with_target_type(
        &int_var("stdin"),
        &c,
        None,
    );
    assert_eq!(result, "std::io::stdin()");
}

#[test]
fn stdout_maps_to_std_io() {
    let c = ctx();
    let result = cg().generate_expression_with_target_type(
        &int_var("stdout"),
        &c,
        None,
    );
    assert_eq!(result, "std::io::stdout()");
}

#[test]
fn errno_maps_to_unsafe_global() {
    let c = ctx();
    let result = cg().generate_expression_with_target_type(
        &int_var("errno"),
        &c,
        None,
    );
    assert_eq!(result, "unsafe { ERRNO }");
}

#[test]
fn erange_maps_to_constant() {
    let c = ctx();
    let result = cg().generate_expression_with_target_type(
        &int_var("ERANGE"),
        &c,
        None,
    );
    assert_eq!(result, "34i32");
}

// ============================================================================
// STATEMENT COVERAGE: VLA declaration
// ============================================================================

#[test]
fn vla_declaration_generates_vec_macro() {
    let func = void_func(vec![HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        },
        initializer: Some(int_var("n")),
    }]);
    let code = cg().generate_function(&func);
    assert!(
        code.contains("vec![0i32;"),
        "Expected vec! macro for VLA, got: {}",
        code
    );
}

#[test]
fn vla_declaration_double_type() {
    let func = void_func(vec![HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Double),
            size: None,
        },
        initializer: Some(int_var("n")),
    }]);
    let code = cg().generate_function(&func);
    assert!(
        code.contains("vec![0.0f64;"),
        "Expected vec! with f64, got: {}",
        code
    );
}

#[test]
fn vla_declaration_char_type() {
    let func = void_func(vec![HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Char),
            size: None,
        },
        initializer: Some(int_var("size")),
    }]);
    let code = cg().generate_function(&func);
    assert!(
        code.contains("vec![0u8;"),
        "Expected vec! with u8, got: {}",
        code
    );
}

#[test]
fn vla_declaration_unsigned_int_type() {
    let func = void_func(vec![HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::UnsignedInt),
            size: None,
        },
        initializer: Some(int_var("n")),
    }]);
    let code = cg().generate_function(&func);
    assert!(
        code.contains("vec![0u32;"),
        "Expected vec! with u32, got: {}",
        code
    );
}

#[test]
fn vla_declaration_signed_char_type() {
    let func = void_func(vec![HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::SignedChar),
            size: None,
        },
        initializer: Some(int_var("n")),
    }]);
    let code = cg().generate_function(&func);
    assert!(
        code.contains("vec![0i8;"),
        "Expected vec! with i8, got: {}",
        code
    );
}

// ============================================================================
// STATEMENT COVERAGE: Return in main → exit
// ============================================================================

#[test]
fn return_in_main_generates_exit() {
    let func = main_func(vec![HirStatement::Return(Some(int_lit(0)))]);
    let code = cg().generate_function(&func);
    assert!(
        code.contains("std::process::exit(0)"),
        "Expected exit in main, got: {}",
        code
    );
}

#[test]
fn return_void_in_main_generates_exit_zero() {
    let func = main_func(vec![HirStatement::Return(None)]);
    let code = cg().generate_function(&func);
    assert!(
        code.contains("std::process::exit(0)"),
        "Expected exit(0) for void return in main, got: {}",
        code
    );
}

#[test]
fn return_in_non_main_generates_return() {
    let func = typed_func(
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(int_lit(42)))],
    );
    let code = cg().generate_function(&func);
    assert!(
        code.contains("return 42"),
        "Expected return 42, got: {}",
        code
    );
}

#[test]
fn return_void_in_non_main_generates_return_semicolon() {
    let func = void_func(vec![HirStatement::Return(None)]);
    let code = cg().generate_function(&func);
    assert!(
        code.contains("return;"),
        "Expected return;, got: {}",
        code
    );
}

// ============================================================================
// STATEMENT COVERAGE: If with pointer condition → !ptr.is_null()
// ============================================================================

#[test]
fn if_with_pointer_condition_uses_is_null() {
    let func = void_func(vec![
        HirStatement::VariableDeclaration {
            name: "p".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: None,
        },
        HirStatement::If {
            condition: int_var("p"),
            then_block: vec![HirStatement::Break],
            else_block: None,
        },
    ]);
    let code = cg().generate_function(&func);
    assert!(
        code.contains("is_null"),
        "Expected is_null for pointer condition, got: {}",
        code
    );
}

// ============================================================================
// STATEMENT COVERAGE: While with non-bool condition → != 0
// ============================================================================

#[test]
fn while_with_int_condition_adds_neq_zero() {
    let func = void_func(vec![
        HirStatement::VariableDeclaration {
            name: "n".to_string(),
            var_type: HirType::Int,
            initializer: Some(int_lit(10)),
        },
        HirStatement::While {
            condition: int_var("n"),
            body: vec![HirStatement::Break],
        },
    ]);
    let code = cg().generate_function(&func);
    assert!(
        code.contains("!= 0"),
        "Expected != 0 for int while condition, got: {}",
        code
    );
}

// ============================================================================
// STATEMENT COVERAGE: Switch with cases
// ============================================================================

#[test]
fn switch_generates_match_with_cases() {
    let func = void_func(vec![HirStatement::Switch {
        condition: int_var("x"),
        cases: vec![
            SwitchCase {
                value: Some(int_lit(1)),
                body: vec![HirStatement::Break],
            },
            SwitchCase {
                value: Some(int_lit(2)),
                body: vec![HirStatement::Break],
            },
        ],
        default_case: Some(vec![HirStatement::Break]),
    }]);
    let code = cg().generate_function(&func);
    assert!(
        code.contains("match"),
        "Expected match for switch, got: {}",
        code
    );
    assert!(
        code.contains("_ =>"),
        "Expected default case, got: {}",
        code
    );
}

// ============================================================================
// STATEMENT COVERAGE: For loop
// ============================================================================

#[test]
fn for_loop_generates_init_and_while() {
    let func = void_func(vec![HirStatement::For {
        init: vec![HirStatement::VariableDeclaration {
            name: "i".to_string(),
            var_type: HirType::Int,
            initializer: Some(int_lit(0)),
        }],
        condition: Some(binop(BinaryOperator::LessThan, int_var("i"), int_lit(10))),
        increment: vec![HirStatement::Assignment {
            target: "i".to_string(),
            value: binop(BinaryOperator::Add, int_var("i"), int_lit(1)),
        }],
        body: vec![HirStatement::Continue],
    }]);
    let code = cg().generate_function(&func);
    assert!(
        code.contains("while"),
        "Expected while for for-loop, got: {}",
        code
    );
    assert!(
        code.contains("let mut i"),
        "Expected init before loop, got: {}",
        code
    );
}

// ============================================================================
// STATEMENT COVERAGE: Assignment to global → unsafe
// ============================================================================

#[test]
fn assignment_to_global_wraps_unsafe() {
    let _func = void_func(vec![
        HirStatement::Assignment {
            target: "g".to_string(),
            value: int_lit(42),
        },
    ]);
    // We need to use generate_statement_with_context directly for global tracking
    let mut c = ctx();
    c.add_variable("g".to_string(), HirType::Int);
    c.add_global("g".to_string());
    let result = cg().generate_statement_with_context(
        &HirStatement::Assignment {
            target: "g".to_string(),
            value: int_lit(42),
        },
        Some("test_func"),
        &mut c,
        None,
    );
    assert!(
        result.contains("unsafe"),
        "Expected unsafe for global assignment, got: {}",
        result
    );
}

// ============================================================================
// STATEMENT COVERAGE: errno assignment → unsafe { ERRNO = ... }
// ============================================================================

#[test]
fn errno_assignment_generates_unsafe_global() {
    let mut c = ctx();
    let result = cg().generate_statement_with_context(
        &HirStatement::Assignment {
            target: "errno".to_string(),
            value: int_lit(0),
        },
        Some("test_func"),
        &mut c,
        None,
    );
    assert!(
        result.contains("unsafe") && result.contains("ERRNO"),
        "Expected unsafe ERRNO assignment, got: {}",
        result
    );
}

// ============================================================================
// STATEMENT COVERAGE: Realloc(ptr, 0) → clear
// ============================================================================

#[test]
fn realloc_zero_size_generates_clear() {
    let mut c = ctx();
    c.add_variable("v".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let result = cg().generate_statement_with_context(
        &HirStatement::Assignment {
            target: "v".to_string(),
            value: HirExpression::Realloc {
                pointer: Box::new(int_var("v")),
                new_size: Box::new(int_lit(0)),
            },
        },
        Some("test_func"),
        &mut c,
        None,
    );
    assert!(
        result.contains(".clear()"),
        "Expected clear for realloc 0, got: {}",
        result
    );
}

// ============================================================================
// STATEMENT COVERAGE: DerefAssignment with pointer field access
// ============================================================================

#[test]
fn deref_assignment_pointer_field_no_extra_deref() {
    let mut c = ctx();
    c.add_variable("s".to_string(), HirType::Pointer(Box::new(HirType::Struct("MyStruct".to_string()))));
    let result = cg().generate_statement_with_context(
        &HirStatement::DerefAssignment {
            target: HirExpression::PointerFieldAccess {
                pointer: Box::new(int_var("s")),
                field: "x".to_string(),
            },
            value: int_lit(10),
        },
        Some("test_func"),
        &mut c,
        None,
    );
    // Should NOT have extra * prefix since PointerFieldAccess is handled specially
    assert!(
        !result.starts_with("*(*"),
        "Should not double-deref, got: {}",
        result
    );
}

// ============================================================================
// STATEMENT COVERAGE: Variable shadows global → renamed
// ============================================================================

#[test]
fn variable_shadowing_global_gets_renamed() {
    let mut c = ctx();
    c.add_variable("count".to_string(), HirType::Int);
    c.add_global("count".to_string());
    let result = cg().generate_statement_with_context(
        &HirStatement::VariableDeclaration {
            name: "count".to_string(),
            var_type: HirType::Int,
            initializer: Some(int_lit(0)),
        },
        Some("test_func"),
        &mut c,
        None,
    );
    assert!(
        result.contains("count_local"),
        "Expected renamed variable, got: {}",
        result
    );
}

// ============================================================================
// STATEMENT COVERAGE: Char array init from string literal → byte string
// ============================================================================

#[test]
fn char_array_from_string_literal_generates_byte_string() {
    let func = void_func(vec![HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(10),
        },
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    }]);
    let code = cg().generate_function(&func);
    assert!(
        code.contains("*b\"hello\\0\""),
        "Expected byte string init, got: {}",
        code
    );
}

// ============================================================================
// STATEMENT COVERAGE: UnsignedInt to Float global cast
// ============================================================================

#[test]
fn unsigned_int_to_double_global_wraps_unsafe() {
    let mut c = ctx();
    c.add_variable("g".to_string(), HirType::UnsignedInt);
    c.add_global("g".to_string());
    let result = cg().generate_expression_with_target_type(
        &int_var("g"),
        &c,
        Some(&HirType::Double),
    );
    assert!(
        result.contains("unsafe") && result.contains("as f64"),
        "Expected unsafe + f64 cast, got: {}",
        result
    );
}

// ============================================================================
// EXPRESSION: AddressOf with Dereference inside → &(expr)
// ============================================================================

#[test]
fn address_of_dereference_wraps_in_parens() {
    let c = ctx();
    let expr = HirExpression::AddressOf(Box::new(HirExpression::Dereference(Box::new(
        int_var("p"),
    ))));
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("&("),
        "Expected &(expr) for address-of dereference, got: {}",
        result
    );
}

// ============================================================================
// EXPRESSION: UnaryOp AddressOf with pointer target
// ============================================================================

#[test]
fn unary_address_of_with_pointer_target_casts() {
    let c = ctx();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::AddressOf,
        operand: Box::new(int_var("x")),
    };
    let result = cg().generate_expression_with_target_type(
        &expr,
        &c,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(
        result.contains("&mut x as *mut i32"),
        "Expected raw pointer cast, got: {}",
        result
    );
}

// ============================================================================
// EXPRESSION: Bitwise with unsigned + bool → u32 result cast
// ============================================================================

#[test]
fn bitwise_or_unsigned_with_bool_casts_result_u32() {
    let mut c = ctx();
    c.add_variable("x".to_string(), HirType::UnsignedInt);
    let cmp = binop(BinaryOperator::Equal, int_lit(1), int_lit(1));
    let expr = binop(BinaryOperator::BitwiseOr, int_var("x"), cmp);
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("as u32"),
        "Expected u32 result cast, got: {}",
        result
    );
}

// ============================================================================
// EXPRESSION: Int arithmetic result → Double target cast
// ============================================================================

#[test]
fn int_arithmetic_with_double_target_casts_to_f64() {
    let mut c = ctx();
    c.add_variable("a".to_string(), HirType::Int);
    c.add_variable("b".to_string(), HirType::Int);
    let expr = binop(BinaryOperator::Add, int_var("a"), int_var("b"));
    let result = cg().generate_expression_with_target_type(&expr, &c, Some(&HirType::Double));
    assert!(
        result.contains("as f64"),
        "Expected f64 cast of int result, got: {}",
        result
    );
}

// ============================================================================
// STATEMENT: If with else block
// ============================================================================

#[test]
fn if_else_generates_both_branches() {
    let func = void_func(vec![HirStatement::If {
        condition: binop(BinaryOperator::GreaterThan, int_lit(5), int_lit(3)),
        then_block: vec![HirStatement::Return(None)],
        else_block: Some(vec![HirStatement::Break]),
    }]);
    let code = cg().generate_function(&func);
    assert!(
        code.contains("} else {"),
        "Expected else block, got: {}",
        code
    );
}

// ============================================================================
// STATEMENT: Break and Continue
// ============================================================================

#[test]
fn break_generates_break() {
    let mut c = ctx();
    let result = cg().generate_statement_with_context(
        &HirStatement::Break,
        Some("test_func"),
        &mut c,
        None,
    );
    assert_eq!(result, "break;");
}

#[test]
fn continue_generates_continue() {
    let mut c = ctx();
    let result = cg().generate_statement_with_context(
        &HirStatement::Continue,
        Some("test_func"),
        &mut c,
        None,
    );
    assert_eq!(result, "continue;");
}

// ============================================================================
// EXPRESSION: Global char to int wraps in unsafe
// ============================================================================

#[test]
fn global_char_to_int_wraps_unsafe() {
    let mut c = ctx();
    c.add_variable("g".to_string(), HirType::Char);
    c.add_global("g".to_string());
    let result = cg().generate_expression_with_target_type(
        &int_var("g"),
        &c,
        Some(&HirType::Int),
    );
    assert!(
        result.contains("unsafe") && result.contains("as i32"),
        "Expected unsafe + i32 cast, got: {}",
        result
    );
}

// ============================================================================
// EXPRESSION: LogicalNot on int without target → (x == 0)
// ============================================================================

#[test]
fn logical_not_int_without_target_is_eq_zero() {
    let mut c = ctx();
    c.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(int_var("x")),
    };
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("== 0"),
        "Expected == 0, got: {}",
        result
    );
    assert!(
        !result.contains("as i32"),
        "Should not cast without Int target, got: {}",
        result
    );
}

// ============================================================================
// EXPRESSION: LogicalNot on binary op wraps in parens
// ============================================================================

#[test]
fn logical_not_on_binary_op_adds_parens() {
    let mut c = ctx();
    c.add_variable("x".to_string(), HirType::Int);
    let binexpr = binop(BinaryOperator::GreaterThan, int_var("x"), int_lit(0));
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(binexpr),
    };
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("!("),
        "Expected parenthesized inner, got: {}",
        result
    );
}

// ============================================================================
// STATEMENT: Switch with char case on int condition → numeric pattern
// ============================================================================

#[test]
fn switch_char_case_on_int_condition_uses_numeric() {
    let func = void_func(vec![
        HirStatement::VariableDeclaration {
            name: "ch".to_string(),
            var_type: HirType::Int,
            initializer: Some(int_lit(65)),
        },
        HirStatement::Switch {
            condition: int_var("ch"),
            cases: vec![SwitchCase {
                value: Some(HirExpression::CharLiteral(b'A' as i8)),
                body: vec![HirStatement::Break],
            }],
            default_case: None,
        },
    ]);
    let code = cg().generate_function(&func);
    // Should use 65 (numeric) not b'A' in match pattern
    assert!(
        code.contains("65 =>"),
        "Expected numeric pattern for char on int switch, got: {}",
        code
    );
}

// ============================================================================
// STATEMENT: DerefAssignment with pointer variable → unsafe deref
// ============================================================================

#[test]
fn deref_assignment_raw_pointer_wraps_unsafe() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let result = cg().generate_statement_with_context(
        &HirStatement::DerefAssignment {
            target: int_var("p"),
            value: int_lit(42),
        },
        Some("test_func"),
        &mut c,
        None,
    );
    assert!(
        result.contains("unsafe"),
        "Expected unsafe for ptr deref, got: {}",
        result
    );
    assert!(
        result.contains("*p = 42"),
        "Expected *p = 42, got: {}",
        result
    );
}

// ============================================================================
// EXPRESSION: Renamed local variable
// ============================================================================

#[test]
fn renamed_local_uses_renamed_name() {
    let mut c = ctx();
    c.add_variable("count".to_string(), HirType::Int);
    c.add_global("count".to_string());
    c.add_renamed_local("count".to_string(), "count_local".to_string());
    let result = cg().generate_expression_with_target_type(&int_var("count"), &c, None);
    // Should use the renamed local, not the global
    assert!(
        result.contains("count_local"),
        "Expected renamed local, got: {}",
        result
    );
}

// ============================================================================
// EXPRESSION: Global array index assign in embedded assignment
// ============================================================================

#[test]
fn embedded_assign_global_array_wraps_unsafe() {
    let mut c = ctx();
    c.add_variable("arr".to_string(), HirType::Array {
        element_type: Box::new(HirType::Int),
        size: Some(10),
    });
    c.add_global("arr".to_string());
    let expr = binop(
        BinaryOperator::Assign,
        HirExpression::ArrayIndex {
            array: Box::new(int_var("arr")),
            index: Box::new(int_lit(0)),
        },
        int_lit(42),
    );
    let result = cg().generate_expression_with_target_type(&expr, &c, None);
    assert!(
        result.contains("unsafe"),
        "Expected unsafe for global array assign, got: {}",
        result
    );
}

// ============================================================================
// STATEMENT: Realloc with multiply pattern → resize
// ============================================================================

#[test]
fn realloc_with_multiply_generates_resize() {
    let mut c = ctx();
    c.add_variable("v".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let result = cg().generate_statement_with_context(
        &HirStatement::Assignment {
            target: "v".to_string(),
            value: HirExpression::Realloc {
                pointer: Box::new(int_var("v")),
                new_size: Box::new(binop(
                    BinaryOperator::Multiply,
                    int_lit(20),
                    HirExpression::Sizeof {
                        type_name: "int".to_string(),
                    },
                )),
            },
        },
        Some("test_func"),
        &mut c,
        None,
    );
    assert!(
        result.contains(".resize("),
        "Expected resize for realloc, got: {}",
        result
    );
}

// ============================================================================
// STATEMENT: Realloc non-multiply → fallback resize
// ============================================================================

#[test]
fn realloc_without_multiply_generates_fallback_resize() {
    let mut c = ctx();
    c.add_variable("v".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let result = cg().generate_statement_with_context(
        &HirStatement::Assignment {
            target: "v".to_string(),
            value: HirExpression::Realloc {
                pointer: Box::new(int_var("v")),
                new_size: Box::new(int_lit(100)),
            },
        },
        Some("test_func"),
        &mut c,
        None,
    );
    assert!(
        result.contains(".resize(") && result.contains("as usize"),
        "Expected resize with usize cast, got: {}",
        result
    );
}

// ============================================================================
// STATEMENT: While with pointer condition → !ptr.is_null()
// ============================================================================

#[test]
fn while_with_pointer_condition_uses_is_null() {
    let func = void_func(vec![
        HirStatement::VariableDeclaration {
            name: "p".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: None,
        },
        HirStatement::While {
            condition: int_var("p"),
            body: vec![HirStatement::Break],
        },
    ]);
    let code = cg().generate_function(&func);
    assert!(
        code.contains("is_null"),
        "Expected is_null for pointer while condition, got: {}",
        code
    );
}

// ============================================================================
// STATEMENT: DerefAssignment with double pointer (Reference to Pointer)
// ============================================================================

#[test]
fn deref_double_pointer_wraps_unsafe() {
    let mut c = ctx();
    c.add_variable(
        "pp".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Pointer(Box::new(HirType::Int))),
            mutable: true,
        },
    );
    let result = cg().generate_statement_with_context(
        &HirStatement::DerefAssignment {
            target: HirExpression::Dereference(Box::new(int_var("pp"))),
            value: int_lit(42),
        },
        Some("test_func"),
        &mut c,
        None,
    );
    assert!(
        result.contains("unsafe"),
        "Expected unsafe for double ptr deref, got: {}",
        result
    );
}
