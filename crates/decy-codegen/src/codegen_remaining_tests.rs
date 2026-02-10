//! Remaining coverage tests for `generate_expression_with_target_type` and
//! `generate_statement_with_context` -- targeting branches not covered by existing tests.
//!
//! Focus areas:
//! - FunctionCall default arm: slice mappings, address-of args, string iter funcs,
//!   raw pointer params, ref params, slice params, int params with char literal,
//!   string params with PointerFieldAccess, function renaming
//! - calloc with Pointer target, realloc function call path
//! - FieldAccess, PointerFieldAccess chaining
//! - ArrayIndex with global/raw pointer, SliceIndex
//! - Sizeof branches (member access, struct field, variable)
//! - NullLiteral, IsNotNull, Calloc/Malloc/Realloc expression paths
//! - StringMethodCall with args / clone_into
//! - Cast with Vec target wrapping malloc, address-of with integer target
//! - CompoundLiteral (struct with partial init, array partial init, other type)
//! - PostIncrement on string reference and deref pointer
//! - PreIncrement/PreDecrement on deref pointer
//! - Ternary with non-boolean condition
//! - Statement: realloc NULL, string iter assignment, ArrayIndexAssignment,
//!   FieldAssignment, Free, Expression, InlineAsm

use super::*;
use decy_hir::{
    BinaryOperator, HirExpression, HirFunction, HirStatement, HirType, UnaryOperator,
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

fn var(name: &str) -> HirExpression {
    HirExpression::Variable(name.to_string())
}

fn ilit(v: i32) -> HirExpression {
    HirExpression::IntLiteral(v)
}

fn make_func(stmts: Vec<HirStatement>) -> HirFunction {
    HirFunction::new_with_body("test_func".to_string(), HirType::Void, vec![], stmts)
}

fn expr_tt(expr: &HirExpression, c: &TypeContext, tt: Option<&HirType>) -> String {
    cg().generate_expression_with_target_type(expr, c, tt)
}

fn stmt_ctx(
    stmt: &HirStatement,
    c: &mut TypeContext,
    fname: Option<&str>,
    ret: Option<&HirType>,
) -> String {
    cg().generate_statement_with_context(stmt, fname, c, ret)
}

// ============================================================================
// 1. calloc with Vec target generates vec! with correct element default
// ============================================================================

#[test]
fn calloc_with_vec_char_target() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "calloc".to_string(),
        arguments: vec![ilit(100), ilit(1)],
    };
    let result = expr_tt(&expr, &c, Some(&HirType::Vec(Box::new(HirType::Char))));
    assert!(
        result.contains("vec![0u8;"),
        "calloc Vec<Char> should use 0u8, got: {}",
        result
    );
}

// ============================================================================
// 2. calloc with Pointer target generates Box::leak
// ============================================================================

#[test]
fn calloc_with_pointer_target() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "calloc".to_string(),
        arguments: vec![ilit(10), ilit(4)],
    };
    let result = expr_tt(&expr, &c, Some(&HirType::Pointer(Box::new(HirType::Int))));
    assert!(
        result.contains("Box::leak") && result.contains("as_mut_ptr"),
        "calloc Pointer target should use Box::leak, got: {}",
        result
    );
}

// ============================================================================
// 3. calloc with no target (default path)
// ============================================================================

#[test]
fn calloc_default_no_target() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "calloc".to_string(),
        arguments: vec![ilit(5), ilit(4)],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("vec![0i32; 5 as usize]"),
        "calloc default should use vec![0i32; n], got: {}",
        result
    );
}

// ============================================================================
// 4. calloc with wrong arg count
// ============================================================================

#[test]
fn calloc_wrong_args_returns_vec_new() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "calloc".to_string(),
        arguments: vec![],
    };
    let result = expr_tt(&expr, &c, None);
    assert_eq!(result, "Vec::new()");
}

// ============================================================================
// 5. realloc function call with pointer target
// ============================================================================

#[test]
fn realloc_func_call_with_pointer_target() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "realloc".to_string(),
        arguments: vec![var("p"), ilit(200)],
    };
    let result = expr_tt(
        &expr,
        &c,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(
        result.contains("realloc(") && result.contains("as *mut i32"),
        "realloc with ptr target should cast return, got: {}",
        result
    );
}

// ============================================================================
// 6. realloc function call no target
// ============================================================================

#[test]
fn realloc_func_call_no_target() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "realloc".to_string(),
        arguments: vec![var("p"), ilit(100)],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("realloc(") && result.contains("as *mut ()"),
        "realloc without target should cast arg to *mut (), got: {}",
        result
    );
}

// ============================================================================
// 7. realloc function call wrong args
// ============================================================================

#[test]
fn realloc_func_call_wrong_args() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "realloc".to_string(),
        arguments: vec![],
    };
    let result = expr_tt(&expr, &c, None);
    assert_eq!(result, "std::ptr::null_mut()");
}

// ============================================================================
// 8. free function call
// ============================================================================

#[test]
fn free_func_call_generates_drop() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "free".to_string(),
        arguments: vec![var("ptr")],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("drop(ptr)"),
        "free should generate drop, got: {}",
        result
    );
}

// ============================================================================
// 9. free with no args
// ============================================================================

#[test]
fn free_no_args_generates_comment() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "free".to_string(),
        arguments: vec![],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("free()"),
        "free no args should generate comment, got: {}",
        result
    );
}

// ============================================================================
// 10. printf with no args
// ============================================================================

#[test]
fn printf_no_args_generates_empty_print() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![],
    };
    let result = expr_tt(&expr, &c, None);
    assert_eq!(result, "print!(\"\")");
}

// ============================================================================
// 11. printf single format string (no extra args)
// ============================================================================

#[test]
fn printf_format_only() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![HirExpression::StringLiteral("hello\\n".to_string())],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("print!("),
        "printf should generate print!, got: {}",
        result
    );
}

// ============================================================================
// 12. Default function call with renamed function (write/read/type/match/self)
// ============================================================================

#[test]
fn func_call_write_renamed_to_c_write() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "write".to_string(),
        arguments: vec![ilit(1)],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("c_write("),
        "write should be renamed to c_write, got: {}",
        result
    );
}

#[test]
fn func_call_read_renamed_to_c_read() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "read".to_string(),
        arguments: vec![ilit(0)],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("c_read("),
        "read should be renamed to c_read, got: {}",
        result
    );
}

#[test]
fn func_call_type_renamed_to_c_type() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "type".to_string(),
        arguments: vec![],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("c_type("),
        "type should be renamed to c_type, got: {}",
        result
    );
}

#[test]
fn func_call_match_renamed_to_c_match() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "match".to_string(),
        arguments: vec![],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("c_match("),
        "match should be renamed to c_match, got: {}",
        result
    );
}

#[test]
fn func_call_self_renamed_to_c_self() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "self".to_string(),
        arguments: vec![],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("c_self("),
        "self should be renamed to c_self, got: {}",
        result
    );
}

#[test]
fn func_call_in_renamed_to_c_in() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "in".to_string(),
        arguments: vec![],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("c_in("),
        "in should be renamed to c_in, got: {}",
        result
    );
}

// ============================================================================
// 13. Default function call with address-of argument
// ============================================================================

#[test]
fn func_call_address_of_arg_generates_mut_ref() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "custom_func".to_string(),
        arguments: vec![HirExpression::AddressOf(Box::new(var("x")))],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("&mut x"),
        "AddressOf arg should generate &mut, got: {}",
        result
    );
}

#[test]
fn func_call_unary_address_of_arg_generates_mut_ref() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "custom_func".to_string(),
        arguments: vec![HirExpression::UnaryOp {
            op: UnaryOperator::AddressOf,
            operand: Box::new(var("y")),
        }],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("&mut y"),
        "UnaryOp AddressOf arg should generate &mut, got: {}",
        result
    );
}

// ============================================================================
// 14. FieldAccess expression
// ============================================================================

#[test]
fn field_access_generates_dot_notation() {
    let c = ctx();
    let expr = HirExpression::FieldAccess {
        object: Box::new(var("obj")),
        field: "name".to_string(),
    };
    let result = expr_tt(&expr, &c, None);
    assert_eq!(result, "obj.name");
}

// ============================================================================
// 15. PointerFieldAccess chained
// ============================================================================

#[test]
fn pointer_field_access_chained() {
    let c = ctx();
    // ptr->a->b generates chained dot notation
    let inner = HirExpression::PointerFieldAccess {
        pointer: Box::new(var("node")),
        field: "next".to_string(),
    };
    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(inner),
        field: "data".to_string(),
    };
    let result = expr_tt(&expr, &c, None);
    // Chained field access should not double-deref
    assert!(
        result.contains(".next.data"),
        "Chained ptr field access should use dots, got: {}",
        result
    );
}

#[test]
fn pointer_field_access_raw_pointer_wraps_unsafe() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(var("p")),
        field: "val".to_string(),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("unsafe"),
        "Raw pointer field access should be unsafe, got: {}",
        result
    );
    assert!(
        result.contains("(*p).val"),
        "Should deref pointer for field access, got: {}",
        result
    );
}

// ============================================================================
// 16. ArrayIndex with raw pointer → unsafe pointer arithmetic
// ============================================================================

#[test]
fn array_index_raw_pointer_uses_unsafe_add() {
    let mut c = ctx();
    c.add_variable("arr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::ArrayIndex {
        array: Box::new(var("arr")),
        index: Box::new(ilit(3)),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("unsafe") && result.contains(".add("),
        "Raw pointer array index should use unsafe .add, got: {}",
        result
    );
}

// ============================================================================
// 17. ArrayIndex with global array → unsafe wrapper
// ============================================================================

#[test]
fn array_index_global_wraps_unsafe() {
    let mut c = ctx();
    c.add_variable(
        "g_arr".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    c.add_global("g_arr".to_string());
    let expr = HirExpression::ArrayIndex {
        array: Box::new(var("g_arr")),
        index: Box::new(ilit(5)),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("unsafe"),
        "Global array index should wrap in unsafe, got: {}",
        result
    );
}

// ============================================================================
// 18. SliceIndex expression
// ============================================================================

#[test]
fn slice_index_generates_safe_indexing() {
    let c = ctx();
    let expr = HirExpression::SliceIndex {
        slice: Box::new(var("data")),
        index: Box::new(ilit(2)),
        element_type: HirType::Int,
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("data[(2) as usize]"),
        "SliceIndex should generate safe indexing, got: {}",
        result
    );
}

// ============================================================================
// 19. Sizeof with known variable → size_of_val
// ============================================================================

#[test]
fn sizeof_variable_uses_size_of_val() {
    let mut c = ctx();
    c.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::Sizeof {
        type_name: "x".to_string(),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("size_of_val"),
        "sizeof(var) should use size_of_val, got: {}",
        result
    );
}

// ============================================================================
// 20. Sizeof with type → size_of
// ============================================================================

#[test]
fn sizeof_type_uses_size_of() {
    let c = ctx();
    let expr = HirExpression::Sizeof {
        type_name: "int".to_string(),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("size_of::<"),
        "sizeof(type) should use size_of, got: {}",
        result
    );
}

// ============================================================================
// 21. NullLiteral → None
// ============================================================================

#[test]
fn null_literal_generates_none() {
    let c = ctx();
    let result = expr_tt(&HirExpression::NullLiteral, &c, None);
    assert_eq!(result, "None");
}

// ============================================================================
// 22. IsNotNull expression
// ============================================================================

#[test]
fn is_not_null_generates_if_let() {
    let c = ctx();
    let expr = HirExpression::IsNotNull(Box::new(var("p")));
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("if let Some(_) = p"),
        "IsNotNull should generate if let Some, got: {}",
        result
    );
}

// ============================================================================
// 23. Calloc expression (not function call)
// ============================================================================

#[test]
fn calloc_expr_int_type() {
    let c = ctx();
    let expr = HirExpression::Calloc {
        count: Box::new(ilit(10)),
        element_type: Box::new(HirType::Int),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("vec![0i32; 10]"),
        "Calloc expr should generate vec!, got: {}",
        result
    );
}

#[test]
fn calloc_expr_unsigned_int_type() {
    let c = ctx();
    let expr = HirExpression::Calloc {
        count: Box::new(ilit(5)),
        element_type: Box::new(HirType::UnsignedInt),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("vec![0u32; 5]"),
        "Calloc UnsignedInt should use 0u32, got: {}",
        result
    );
}

#[test]
fn calloc_expr_float_type() {
    let c = ctx();
    let expr = HirExpression::Calloc {
        count: Box::new(ilit(3)),
        element_type: Box::new(HirType::Float),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("vec![0.0f32; 3]"),
        "Calloc Float should use 0.0f32, got: {}",
        result
    );
}

#[test]
fn calloc_expr_double_type() {
    let c = ctx();
    let expr = HirExpression::Calloc {
        count: Box::new(ilit(4)),
        element_type: Box::new(HirType::Double),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("vec![0.0f64; 4]"),
        "Calloc Double should use 0.0f64, got: {}",
        result
    );
}

#[test]
fn calloc_expr_char_type() {
    let c = ctx();
    let expr = HirExpression::Calloc {
        count: Box::new(ilit(256)),
        element_type: Box::new(HirType::Char),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("vec![0u8; 256]"),
        "Calloc Char should use 0u8, got: {}",
        result
    );
}

#[test]
fn calloc_expr_signed_char_type() {
    let c = ctx();
    let expr = HirExpression::Calloc {
        count: Box::new(ilit(8)),
        element_type: Box::new(HirType::SignedChar),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("vec![0i8; 8]"),
        "Calloc SignedChar should use 0i8, got: {}",
        result
    );
}

// ============================================================================
// 24. Malloc expression with multiply → Vec::with_capacity
// ============================================================================

#[test]
fn malloc_expr_multiply_generates_vec_capacity() {
    let c = ctx();
    let expr = HirExpression::Malloc {
        size: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(var("n")),
            right: Box::new(ilit(4)),
        }),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("Vec::with_capacity(n)"),
        "Malloc multiply should use Vec::with_capacity, got: {}",
        result
    );
}

// ============================================================================
// 25. Malloc expression without multiply → Box::new
// ============================================================================

#[test]
fn malloc_expr_single_generates_box_new() {
    let c = ctx();
    let expr = HirExpression::Malloc {
        size: Box::new(ilit(4)),
    };
    let result = expr_tt(&expr, &c, None);
    assert_eq!(result, "Box::new(0i32)");
}

// ============================================================================
// 26. Realloc expression with NULL pointer and multiply → vec!
// ============================================================================

#[test]
fn realloc_expr_null_multiply_generates_vec() {
    let c = ctx();
    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::NullLiteral),
        new_size: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(ilit(10)),
            right: Box::new(ilit(4)),
        }),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("vec![0i32; 10]"),
        "Realloc NULL with multiply should generate vec!, got: {}",
        result
    );
}

// ============================================================================
// 27. Realloc expression with NULL pointer no multiply → Vec::new
// ============================================================================

#[test]
fn realloc_expr_null_no_multiply_generates_vec_new() {
    let c = ctx();
    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::NullLiteral),
        new_size: Box::new(ilit(100)),
    };
    let result = expr_tt(&expr, &c, None);
    assert_eq!(result, "Vec::new()");
}

// ============================================================================
// 28. Realloc expression with non-NULL pointer → passthrough
// ============================================================================

#[test]
fn realloc_expr_non_null_returns_pointer() {
    let c = ctx();
    let expr = HirExpression::Realloc {
        pointer: Box::new(var("buf")),
        new_size: Box::new(ilit(200)),
    };
    let result = expr_tt(&expr, &c, None);
    assert_eq!(result, "buf");
}

// ============================================================================
// 29. StringMethodCall with len → as i32 cast
// ============================================================================

#[test]
fn string_method_call_len_casts_i32() {
    let c = ctx();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(var("s")),
        method: "len".to_string(),
        arguments: vec![],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("s.len() as i32"),
        "StringMethodCall len should cast to i32, got: {}",
        result
    );
}

// ============================================================================
// 30. StringMethodCall with non-len method → no cast
// ============================================================================

#[test]
fn string_method_call_is_empty_no_cast() {
    let c = ctx();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(var("s")),
        method: "is_empty".to_string(),
        arguments: vec![],
    };
    let result = expr_tt(&expr, &c, None);
    assert_eq!(result, "s.is_empty()");
}

// ============================================================================
// 31. StringMethodCall clone_into adds &mut
// ============================================================================

#[test]
fn string_method_call_clone_into_adds_mut_ref() {
    let c = ctx();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(var("src")),
        method: "clone_into".to_string(),
        arguments: vec![var("dest")],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("&mut dest"),
        "clone_into should add &mut to arg, got: {}",
        result
    );
}

// ============================================================================
// 32. StringMethodCall with regular args → no &mut
// ============================================================================

#[test]
fn string_method_call_regular_args() {
    let c = ctx();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(var("s")),
        method: "push".to_string(),
        arguments: vec![ilit(42)],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("s.push(42)"),
        "Regular method call should pass args directly, got: {}",
        result
    );
}

// ============================================================================
// 33. Cast with Vec target wrapping malloc → unwrap cast
// ============================================================================

#[test]
fn cast_vec_target_unwraps_malloc_cast() {
    let c = ctx();
    let expr = HirExpression::Cast {
        target_type: HirType::Pointer(Box::new(HirType::Int)),
        expr: Box::new(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(var("n")),
                right: Box::new(ilit(4)),
            }],
        }),
    };
    let result = expr_tt(&expr, &c, Some(&HirType::Vec(Box::new(HirType::Int))));
    // Should skip the cast and generate vec! directly
    assert!(
        result.contains("vec!["),
        "Cast around malloc with Vec target should unwrap, got: {}",
        result
    );
}

// ============================================================================
// 34. Cast address-of to integer type → pointer chain cast
// ============================================================================

#[test]
fn cast_address_of_to_int_uses_pointer_chain() {
    let c = ctx();
    let expr = HirExpression::Cast {
        target_type: HirType::Int,
        expr: Box::new(HirExpression::AddressOf(Box::new(var("x")))),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("as *const _ as isize as i32"),
        "AddressOf to int should use pointer chain, got: {}",
        result
    );
}

// ============================================================================
// 35. Cast with binary op expression wraps in parens
// ============================================================================

#[test]
fn cast_binary_op_wraps_in_parens() {
    let c = ctx();
    let expr = HirExpression::Cast {
        target_type: HirType::Float,
        expr: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(ilit(1)),
            right: Box::new(ilit(2)),
        }),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("(1 + 2) as f32"),
        "Cast should wrap binop in parens, got: {}",
        result
    );
}

// ============================================================================
// 36. CompoundLiteral struct with partial init → Default
// ============================================================================

#[test]
fn compound_literal_struct_partial_init_uses_default() {
    let mut c = ctx();
    c.structs.insert(
        "Point".to_string(),
        vec![
            ("x".to_string(), HirType::Int),
            ("y".to_string(), HirType::Int),
            ("z".to_string(), HirType::Int),
        ],
    );
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Point".to_string()),
        initializers: vec![ilit(10)],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("..Default::default()"),
        "Partial struct init should include Default, got: {}",
        result
    );
}

// ============================================================================
// 37. CompoundLiteral struct empty → Struct {}
// ============================================================================

#[test]
fn compound_literal_struct_empty() {
    let c = ctx();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Empty".to_string()),
        initializers: vec![],
    };
    let result = expr_tt(&expr, &c, None);
    assert_eq!(result, "Empty {}");
}

// ============================================================================
// 38. CompoundLiteral array with size and single init → repeat
// ============================================================================

#[test]
fn compound_literal_array_single_init_repeats() {
    let c = ctx();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(5),
        },
        initializers: vec![ilit(0)],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("[0; 5]"),
        "Single init array should repeat, got: {}",
        result
    );
}

// ============================================================================
// 39. CompoundLiteral array partial init → pad with defaults
// ============================================================================

#[test]
fn compound_literal_array_partial_init_pads() {
    let c = ctx();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(4),
        },
        initializers: vec![ilit(1), ilit(2)],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("[1, 2, 0i32, 0i32]"),
        "Partial array init should pad with defaults, got: {}",
        result
    );
}

// ============================================================================
// 40. CompoundLiteral empty array with size → default fill
// ============================================================================

#[test]
fn compound_literal_array_empty_with_size() {
    let c = ctx();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Double),
            size: Some(3),
        },
        initializers: vec![],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("[0.0f64; 3]"),
        "Empty array with size should default-fill, got: {}",
        result
    );
}

// ============================================================================
// 41. CompoundLiteral unsized array with no inits
// ============================================================================

#[test]
fn compound_literal_array_unsized_empty() {
    let c = ctx();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        },
        initializers: vec![],
    };
    let result = expr_tt(&expr, &c, None);
    assert_eq!(result, "[]");
}

// ============================================================================
// 42. CompoundLiteral other type → comment
// ============================================================================

#[test]
fn compound_literal_other_type_generates_comment() {
    let c = ctx();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Int,
        initializers: vec![ilit(42)],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("Compound literal"),
        "Other compound literal should generate comment, got: {}",
        result
    );
}

// ============================================================================
// 43. PostIncrement on string reference → byte access pattern
// ============================================================================

#[test]
fn post_increment_string_ref_generates_byte_access() {
    let mut c = ctx();
    c.add_variable("key".to_string(), HirType::StringReference);
    let expr = HirExpression::PostIncrement {
        operand: Box::new(var("key")),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("as_bytes()[0]") && result.contains("&key[1..]"),
        "PostIncrement on &str should generate byte access, got: {}",
        result
    );
}

// ============================================================================
// 44. PostIncrement on deref pointer → unsafe block
// ============================================================================

#[test]
fn post_increment_deref_pointer_wraps_unsafe() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Dereference(Box::new(var("p")))),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("unsafe { *p }") && result.contains("unsafe { *p += 1 }"),
        "PostIncrement deref ptr should use unsafe, got: {}",
        result
    );
}

// ============================================================================
// 45. PreIncrement on deref pointer → unsafe block
// ============================================================================

#[test]
fn pre_increment_deref_pointer_wraps_unsafe() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PreIncrement {
        operand: Box::new(HirExpression::Dereference(Box::new(var("p")))),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("unsafe { *p += 1 }") && result.contains("unsafe { *p }"),
        "PreIncrement deref ptr should use unsafe, got: {}",
        result
    );
}

// ============================================================================
// 46. PostDecrement on deref pointer → unsafe block
// ============================================================================

#[test]
fn post_decrement_deref_pointer_wraps_unsafe() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PostDecrement {
        operand: Box::new(HirExpression::Dereference(Box::new(var("p")))),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("unsafe { *p }") && result.contains("unsafe { *p -= 1 }"),
        "PostDecrement deref ptr should use unsafe, got: {}",
        result
    );
}

// ============================================================================
// 47. PreDecrement on deref pointer → unsafe block
// ============================================================================

#[test]
fn pre_decrement_deref_pointer_wraps_unsafe() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PreDecrement {
        operand: Box::new(HirExpression::Dereference(Box::new(var("p")))),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("unsafe { *p -= 1 }") && result.contains("unsafe { *p }"),
        "PreDecrement deref ptr should use unsafe, got: {}",
        result
    );
}

// ============================================================================
// 48. Ternary with non-boolean condition → != 0
// ============================================================================

#[test]
fn ternary_non_boolean_cond_adds_neq_zero() {
    let c = ctx();
    let expr = HirExpression::Ternary {
        condition: Box::new(var("x")),
        then_expr: Box::new(ilit(1)),
        else_expr: Box::new(ilit(0)),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("!= 0"),
        "Ternary non-bool cond should add != 0, got: {}",
        result
    );
    assert!(
        result.contains("if") && result.contains("else"),
        "Ternary should generate if/else, got: {}",
        result
    );
}

// ============================================================================
// 49. Ternary with boolean condition → no != 0
// ============================================================================

#[test]
fn ternary_boolean_cond_no_neq_zero() {
    let c = ctx();
    let cond = HirExpression::BinaryOp {
        op: BinaryOperator::GreaterThan,
        left: Box::new(ilit(5)),
        right: Box::new(ilit(3)),
    };
    let expr = HirExpression::Ternary {
        condition: Box::new(cond),
        then_expr: Box::new(ilit(1)),
        else_expr: Box::new(ilit(0)),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        !result.contains("!= 0"),
        "Ternary bool cond should not add != 0, got: {}",
        result
    );
}

// ============================================================================
// 50. Ternary propagates target_type to branches
// ============================================================================

#[test]
fn ternary_propagates_target_type_to_branches() {
    let c = ctx();
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(ilit(1)),
            right: Box::new(ilit(1)),
        }),
        then_expr: Box::new(HirExpression::StringLiteral("yes".to_string())),
        else_expr: Box::new(HirExpression::StringLiteral("no".to_string())),
    };
    let result = expr_tt(
        &expr,
        &c,
        Some(&HirType::Pointer(Box::new(HirType::Char))),
    );
    // Both branches should have byte string conversion
    assert!(
        result.contains("b\"yes\\0\"") && result.contains("b\"no\\0\""),
        "Ternary should propagate target type to branches, got: {}",
        result
    );
}

// ============================================================================
// 51. Statement: Realloc assignment with NULL pointer → resize from 0
// ============================================================================

#[test]
fn stmt_realloc_null_pointer_generates_resize() {
    let mut c = ctx();
    c.add_variable("v".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let result = stmt_ctx(
        &HirStatement::Assignment {
            target: "v".to_string(),
            value: HirExpression::Realloc {
                pointer: Box::new(HirExpression::NullLiteral),
                new_size: Box::new(HirExpression::BinaryOp {
                    op: BinaryOperator::Multiply,
                    left: Box::new(ilit(10)),
                    right: Box::new(ilit(4)),
                }),
            },
        },
        &mut c,
        Some("test_func"),
        None,
    );
    assert!(
        result.contains(".resize("),
        "Realloc NULL should generate resize, got: {}",
        result
    );
}

// ============================================================================
// 52. Statement: ArrayIndexAssignment with raw pointer → unsafe
// ============================================================================

#[test]
fn stmt_array_index_assign_raw_pointer_unsafe() {
    let mut c = ctx();
    c.add_variable("buf".to_string(), HirType::Pointer(Box::new(HirType::Char)));
    let result = stmt_ctx(
        &HirStatement::ArrayIndexAssignment {
            array: Box::new(var("buf")),
            index: Box::new(ilit(0)),
            value: ilit(65),
        },
        &mut c,
        Some("test_func"),
        None,
    );
    assert!(
        result.contains("unsafe") && result.contains(".add("),
        "Raw pointer array assign should use unsafe .add, got: {}",
        result
    );
}

// ============================================================================
// 53. Statement: ArrayIndexAssignment with global array → unsafe wrapper
// ============================================================================

#[test]
fn stmt_array_index_assign_global_unsafe() {
    let mut c = ctx();
    c.add_variable(
        "g_arr".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    c.add_global("g_arr".to_string());
    let result = stmt_ctx(
        &HirStatement::ArrayIndexAssignment {
            array: Box::new(var("g_arr")),
            index: Box::new(ilit(0)),
            value: ilit(42),
        },
        &mut c,
        Some("test_func"),
        None,
    );
    assert!(
        result.contains("unsafe {"),
        "Global array index assign should wrap unsafe, got: {}",
        result
    );
}

// ============================================================================
// 54. Statement: ArrayIndexAssignment int-to-char coercion
// ============================================================================

#[test]
fn stmt_array_index_assign_int_to_char_coercion() {
    let mut c = ctx();
    c.add_variable(
        "buf".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(100),
        },
    );
    c.add_variable("n".to_string(), HirType::Int);
    let result = stmt_ctx(
        &HirStatement::ArrayIndexAssignment {
            array: Box::new(var("buf")),
            index: Box::new(ilit(0)),
            value: var("n"),
        },
        &mut c,
        Some("test_func"),
        None,
    );
    assert!(
        result.contains("as u8"),
        "Int-to-char array assign should cast to u8, got: {}",
        result
    );
}

// ============================================================================
// 55. Statement: FieldAssignment with raw pointer → unsafe
// ============================================================================

#[test]
fn stmt_field_assign_raw_pointer_unsafe() {
    let mut c = ctx();
    c.add_variable(
        "node".to_string(),
        HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
    );
    let result = stmt_ctx(
        &HirStatement::FieldAssignment {
            object: var("node"),
            field: "data".to_string(),
            value: ilit(42),
        },
        &mut c,
        Some("test_func"),
        None,
    );
    assert!(
        result.contains("unsafe"),
        "Raw pointer field assign should be unsafe, got: {}",
        result
    );
    assert!(
        result.contains("(*node).data"),
        "Should deref pointer for field access, got: {}",
        result
    );
}

// ============================================================================
// 56. Statement: FieldAssignment with global struct → unsafe
// ============================================================================

#[test]
fn stmt_field_assign_global_struct_unsafe() {
    let mut c = ctx();
    c.add_variable(
        "config".to_string(),
        HirType::Struct("Config".to_string()),
    );
    c.add_global("config".to_string());
    let result = stmt_ctx(
        &HirStatement::FieldAssignment {
            object: var("config"),
            field: "value".to_string(),
            value: ilit(99),
        },
        &mut c,
        Some("test_func"),
        None,
    );
    assert!(
        result.contains("unsafe {"),
        "Global struct field assign should be unsafe, got: {}",
        result
    );
}

// ============================================================================
// 57. Statement: Regular FieldAssignment
// ============================================================================

#[test]
fn stmt_field_assign_regular() {
    let mut c = ctx();
    c.add_variable(
        "obj".to_string(),
        HirType::Struct("Obj".to_string()),
    );
    let result = stmt_ctx(
        &HirStatement::FieldAssignment {
            object: var("obj"),
            field: "x".to_string(),
            value: ilit(10),
        },
        &mut c,
        Some("test_func"),
        None,
    );
    assert_eq!(result, "obj.x = 10;");
}

// ============================================================================
// 58. Statement: Free → RAII comment
// ============================================================================

#[test]
fn stmt_free_generates_raii_comment() {
    let mut c = ctx();
    let result = stmt_ctx(
        &HirStatement::Free {
            pointer: var("ptr"),
        },
        &mut c,
        Some("test_func"),
        None,
    );
    assert!(
        result.contains("RAII") && result.contains("ptr"),
        "Free should generate RAII comment, got: {}",
        result
    );
}

// ============================================================================
// 59. Statement: Free with complex expression
// ============================================================================

#[test]
fn stmt_free_complex_expression() {
    let mut c = ctx();
    let result = stmt_ctx(
        &HirStatement::Free {
            pointer: HirExpression::FieldAccess {
                object: Box::new(var("node")),
                field: "data".to_string(),
            },
        },
        &mut c,
        Some("test_func"),
        None,
    );
    assert!(
        result.contains("RAII") && result.contains("node.data"),
        "Free complex expr should generate RAII comment, got: {}",
        result
    );
}

// ============================================================================
// 60. Statement: Expression statement
// ============================================================================

#[test]
fn stmt_expression_generates_semicolon() {
    let mut c = ctx();
    let result = stmt_ctx(
        &HirStatement::Expression(HirExpression::FunctionCall {
            function: "custom".to_string(),
            arguments: vec![ilit(42)],
        }),
        &mut c,
        Some("test_func"),
        None,
    );
    assert!(
        result.contains("custom(42);"),
        "Expression statement should end with semicolon, got: {}",
        result
    );
}

// ============================================================================
// 61. Statement: InlineAsm non-translatable
// ============================================================================

#[test]
fn stmt_inline_asm_non_translatable() {
    let mut c = ctx();
    let result = stmt_ctx(
        &HirStatement::InlineAsm {
            text: "nop".to_string(),
            translatable: false,
        },
        &mut c,
        Some("test_func"),
        None,
    );
    assert!(
        result.contains("manual review"),
        "InlineAsm should note manual review, got: {}",
        result
    );
    assert!(
        result.contains("nop"),
        "InlineAsm should include asm text, got: {}",
        result
    );
    assert!(
        !result.contains("translatable"),
        "Non-translatable should not mention translatable, got: {}",
        result
    );
}

// ============================================================================
// 62. Statement: InlineAsm translatable
// ============================================================================

#[test]
fn stmt_inline_asm_translatable() {
    let mut c = ctx();
    let result = stmt_ctx(
        &HirStatement::InlineAsm {
            text: "bswap eax".to_string(),
            translatable: true,
        },
        &mut c,
        Some("test_func"),
        None,
    );
    assert!(
        result.contains("manual review"),
        "InlineAsm should note manual review, got: {}",
        result
    );
    assert!(
        result.contains("translatable to Rust intrinsics"),
        "Translatable asm should note potential, got: {}",
        result
    );
}

// ============================================================================
// 63. Statement: DerefAssignment with ArrayIndex target → no extra deref
// ============================================================================

#[test]
fn stmt_deref_assign_array_index_no_extra_deref() {
    let mut c = ctx();
    let result = stmt_ctx(
        &HirStatement::DerefAssignment {
            target: HirExpression::ArrayIndex {
                array: Box::new(var("arr")),
                index: Box::new(ilit(0)),
            },
            value: ilit(42),
        },
        &mut c,
        Some("test_func"),
        None,
    );
    assert!(
        !result.starts_with("*"),
        "ArrayIndex deref assign should not add extra *, got: {}",
        result
    );
    assert!(
        result.contains("arr[(0) as usize] = 42;"),
        "Should generate direct array assignment, got: {}",
        result
    );
}

// ============================================================================
// 64. Statement: DerefAssignment with FieldAccess target → no extra deref
// ============================================================================

#[test]
fn stmt_deref_assign_field_access_no_extra_deref() {
    let mut c = ctx();
    let result = stmt_ctx(
        &HirStatement::DerefAssignment {
            target: HirExpression::FieldAccess {
                object: Box::new(var("obj")),
                field: "x".to_string(),
            },
            value: ilit(10),
        },
        &mut c,
        Some("test_func"),
        None,
    );
    assert!(
        result.contains("obj.x = 10;"),
        "FieldAccess deref assign should not add extra *, got: {}",
        result
    );
}

// ============================================================================
// 65. Int literal 0 with Option target → None
// ============================================================================

#[test]
fn int_literal_zero_option_target_becomes_none() {
    let c = ctx();
    let result = expr_tt(
        &ilit(0),
        &c,
        Some(&HirType::Option(Box::new(HirType::Int))),
    );
    assert_eq!(result, "None");
}

// ============================================================================
// 66. Int literal 0 with Pointer target → null_mut
// ============================================================================

#[test]
fn int_literal_zero_pointer_target_becomes_null_mut() {
    let c = ctx();
    let result = expr_tt(
        &ilit(0),
        &c,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert_eq!(result, "std::ptr::null_mut()");
}

// ============================================================================
// 67. Float literal without target with dot → f64 suffix
// ============================================================================

#[test]
fn float_literal_no_target_with_dot() {
    let c = ctx();
    let expr = HirExpression::FloatLiteral("3.14".to_string());
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("3.14f64"),
        "Float literal with dot should get f64 suffix, got: {}",
        result
    );
}

// ============================================================================
// 68. Float literal without target without dot → .0f64
// ============================================================================

#[test]
fn float_literal_no_target_integer_form() {
    let c = ctx();
    let expr = HirExpression::FloatLiteral("42".to_string());
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("42.0f64"),
        "Float literal without dot should get .0f64, got: {}",
        result
    );
}

// ============================================================================
// 69. AddressOf with Pointer target → raw pointer cast
// ============================================================================

#[test]
fn address_of_with_pointer_target_casts_to_raw() {
    let c = ctx();
    let expr = HirExpression::AddressOf(Box::new(var("x")));
    let result = expr_tt(
        &expr,
        &c,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(
        result.contains("&mut x as *mut i32"),
        "AddressOf with pointer target should cast, got: {}",
        result
    );
}

// ============================================================================
// 70. AddressOf without target (no dereference inside)
// ============================================================================

#[test]
fn address_of_simple_no_target() {
    let c = ctx();
    let expr = HirExpression::AddressOf(Box::new(var("x")));
    let result = expr_tt(&expr, &c, None);
    assert_eq!(result, "&x");
}

// ============================================================================
// 71. EINVAL/ENOENT/EACCES constant mapping
// ============================================================================

#[test]
fn einval_maps_to_constant() {
    let c = ctx();
    let result = expr_tt(&var("EINVAL"), &c, None);
    assert_eq!(result, "22i32");
}

#[test]
fn enoent_maps_to_constant() {
    let c = ctx();
    let result = expr_tt(&var("ENOENT"), &c, None);
    assert_eq!(result, "2i32");
}

#[test]
fn eacces_maps_to_constant() {
    let c = ctx();
    let result = expr_tt(&var("EACCES"), &c, None);
    assert_eq!(result, "13i32");
}

// ============================================================================
// 72. Return in main with char expression → exit with cast
// ============================================================================

#[test]
fn return_char_in_main_casts_to_i32() {
    let mut c = ctx();
    c.add_variable("ch".to_string(), HirType::Char);
    let result = stmt_ctx(
        &HirStatement::Return(Some(var("ch"))),
        &mut c,
        Some("main"),
        Some(&HirType::Int),
    );
    assert!(
        result.contains("as i32"),
        "Return char in main should cast to i32, got: {}",
        result
    );
    assert!(
        result.contains("exit"),
        "Return in main should use exit, got: {}",
        result
    );
}

// ============================================================================
// 73. Dereference raw pointer → unsafe block
// ============================================================================

#[test]
fn dereference_raw_pointer_wraps_unsafe() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::Dereference(Box::new(var("p")));
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("unsafe"),
        "Deref raw pointer should be unsafe, got: {}",
        result
    );
}

// ============================================================================
// 74. Dereference pointer arithmetic → unsafe
// ============================================================================

#[test]
fn dereference_pointer_arithmetic_wraps_unsafe() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::Dereference(Box::new(HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(var("p")),
        right: Box::new(ilit(1)),
    }));
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("unsafe"),
        "Deref ptr arithmetic should be unsafe, got: {}",
        result
    );
}

// ============================================================================
// 75. UnaryOp Minus generates prefix minus
// ============================================================================

#[test]
fn unary_minus_generates_prefix() {
    let c = ctx();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::Minus,
        operand: Box::new(var("x")),
    };
    let result = expr_tt(&expr, &c, None);
    assert_eq!(result, "-x");
}

// ============================================================================
// 76. UnaryOp BitwiseNot generates prefix !
// ============================================================================

#[test]
fn unary_bitwise_not_generates_prefix_bang() {
    let c = ctx();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::BitwiseNot,
        operand: Box::new(var("x")),
    };
    let result = expr_tt(&expr, &c, None);
    assert_eq!(result, "!x");
}

// ============================================================================
// 77. UnaryOp LogicalNot in context (not target_type path) on boolean → !expr
// ============================================================================

#[test]
fn unary_logical_not_boolean_in_context() {
    let c = ctx();
    let inner = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(ilit(1)),
        right: Box::new(ilit(1)),
    };
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(inner),
    };
    let result = cg().generate_expression_with_context(&expr, &c);
    assert!(
        result.starts_with("!"),
        "LogicalNot on bool in context should be !, got: {}",
        result
    );
}

// ============================================================================
// 78. UnaryOp LogicalNot in context on integer → (x == 0)
// ============================================================================

#[test]
fn unary_logical_not_integer_in_context() {
    let c = ctx();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(var("n")),
    };
    let result = cg().generate_expression_with_context(&expr, &c);
    // generate_expression_with_context calls with None target, so the specific
    // LogicalNot arm at line 1070 applies: no Int target → (x == 0) without i32 cast
    assert!(
        result.contains("== 0") && !result.contains("as i32"),
        "LogicalNot on int without target should be (n == 0) no cast, got: {}",
        result
    );
}

// ============================================================================
// 79. PostIncrement on pointer type → wrapping_add
// ============================================================================

#[test]
fn post_increment_pointer_uses_wrapping_add() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PostIncrement {
        operand: Box::new(var("p")),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("wrapping_add(1)"),
        "PostIncrement ptr should use wrapping_add, got: {}",
        result
    );
}

// ============================================================================
// 80. PreDecrement on pointer type → wrapping_sub
// ============================================================================

#[test]
fn pre_decrement_pointer_uses_wrapping_sub() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PreDecrement {
        operand: Box::new(var("p")),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("wrapping_sub(1)"),
        "PreDecrement ptr should use wrapping_sub, got: {}",
        result
    );
}

// ============================================================================
// 81. Sizeof with struct field pattern
// ============================================================================

#[test]
fn sizeof_struct_field_looks_up_type() {
    let mut c = ctx();
    c.structs.insert(
        "Record".to_string(),
        vec![
            ("name".to_string(), HirType::Pointer(Box::new(HirType::Char))),
            ("age".to_string(), HirType::Int),
        ],
    );
    let expr = HirExpression::Sizeof {
        type_name: "struct Record age".to_string(),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("size_of::<i32>()"),
        "sizeof struct field should look up type, got: {}",
        result
    );
}

// ============================================================================
// 82. Pointer field access from FieldAccess (chaining)
// ============================================================================

#[test]
fn pointer_field_access_from_field_access() {
    let c = ctx();
    let inner = HirExpression::FieldAccess {
        object: Box::new(var("obj")),
        field: "ptr".to_string(),
    };
    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(inner),
        field: "val".to_string(),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("obj.ptr.val"),
        "FieldAccess -> PointerFieldAccess should chain, got: {}",
        result
    );
}

// ============================================================================
// 83. malloc function call with char pointer target → byte buffer
// ============================================================================

#[test]
fn malloc_func_call_char_pointer_target() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![ilit(256)],
    };
    let result = expr_tt(
        &expr,
        &c,
        Some(&HirType::Pointer(Box::new(HirType::Char))),
    );
    assert!(
        result.contains("vec![0u8;") && result.contains("as_mut_ptr"),
        "malloc char pointer should allocate byte buffer, got: {}",
        result
    );
}

// ============================================================================
// 84. malloc function call with no args → Vec::new
// ============================================================================

#[test]
fn malloc_func_call_no_args() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![],
    };
    let result = expr_tt(&expr, &c, None);
    assert_eq!(result, "Vec::new()");
}

// ============================================================================
// 85. wait function call
// ============================================================================

#[test]
fn wait_func_call_generates_child_wait() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "wait".to_string(),
        arguments: vec![var("status")],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("child.wait()"),
        "wait should generate child.wait(), got: {}",
        result
    );
}

// ============================================================================
// 86. exec with no args → comment
// ============================================================================

#[test]
fn exec_no_args_generates_comment() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "execl".to_string(),
        arguments: vec![],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("exec requires args"),
        "exec no args should generate comment, got: {}",
        result
    );
}

// ============================================================================
// 87. fprintf with wrong arg count
// ============================================================================

#[test]
fn fprintf_wrong_args_generates_fallback() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fprintf".to_string(),
        arguments: vec![],
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("fprintf requires"),
        "fprintf no args should generate error, got: {}",
        result
    );
}

// ============================================================================
// 88. Escape reserved keyword in variable name
// ============================================================================

#[test]
fn variable_reserved_keyword_escaped() {
    let c = ctx();
    // "type" is a reserved Rust keyword
    let result = expr_tt(&var("type"), &c, None);
    assert!(
        result.contains("r#type"),
        "Reserved keyword should be escaped, got: {}",
        result
    );
}

// ============================================================================
// 89. VLA with Float element type
// ============================================================================

#[test]
fn vla_float_type_uses_f32_default() {
    let func = make_func(vec![HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Float),
            size: None,
        },
        initializer: Some(var("n")),
    }]);
    let code = cg().generate_function(&func);
    assert!(
        code.contains("vec![0.0f32;"),
        "VLA Float should use 0.0f32, got: {}",
        code
    );
}

// ============================================================================
// 90. Pointer sub with non-variable integer expression → wrapping_sub
// ============================================================================

#[test]
fn pointer_sub_expression_uses_wrapping_sub() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(var("p")),
        // Non-variable (literal) on right side
        right: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(ilit(1)),
            right: Box::new(ilit(2)),
        }),
    };
    let result = expr_tt(&expr, &c, None);
    assert!(
        result.contains("wrapping_sub"),
        "Ptr sub with expression should use wrapping_sub, got: {}",
        result
    );
}
