//! Deep coverage tests for `transform_expression_recursive_with_length` and
//! `transform_expression_with_length_replacement` in borrow_gen.rs.
//!
//! These tests exercise every match arm in the recursive expression
//! transformation, targeting the 80 uncovered lines (591-875).

use super::*;
use crate::inference::{OwnershipInference, OwnershipKind};
use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType, UnaryOperator};
use std::collections::HashMap;

// ============================================================================
// Helpers
// ============================================================================

fn empty_inferences() -> HashMap<String, OwnershipInference> {
    HashMap::new()
}

fn empty_length_params() -> HashMap<String, String> {
    HashMap::new()
}

fn length_params_with(length_var: &str, array_var: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert(length_var.to_string(), array_var.to_string());
    m
}

fn array_pointer_inference(var: &str, base: &str) -> (String, OwnershipInference) {
    (
        var.to_string(),
        OwnershipInference {
            variable: var.to_string(),
            kind: OwnershipKind::ArrayPointer {
                base_array: base.to_string(),
                element_type: HirType::Int,
                base_index: None,
            },
            confidence: 0.9,
            reason: "test".to_string(),
        },
    )
}

/// Build a minimal HirFunction with a single expression statement in body.
fn func_with_body_expr(expr: HirExpression) -> HirFunction {
    HirFunction::new_with_body(
        "test_fn".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Expression(expr)],
    )
}

/// Build a function with params and body statements.
fn func_with_params_and_body(
    params: Vec<HirParameter>,
    body: Vec<HirStatement>,
) -> HirFunction {
    HirFunction::new_with_body("test_fn".to_string(), HirType::Void, params, body)
}

// ============================================================================
// Section 1: Dereference transformation
// ============================================================================

#[test]
fn deep_deref_simple_variable() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("x".to_string())));
    let func = func_with_body_expr(expr);
    let result = gen.transform_function(&func, &empty_inferences());
    let body = result.body();
    assert_eq!(body.len(), 1);
    if let HirStatement::Expression(HirExpression::Dereference(inner)) = &body[0] {
        assert!(matches!(&**inner, HirExpression::Variable(n) if n == "x"));
    } else {
        panic!("Expected Dereference expression");
    }
}

#[test]
fn deep_deref_with_length_replacement() {
    // Dereference of a variable that is a length param => Dereference(len_call)
    let gen = BorrowGenerator::new();
    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("n".to_string())));
    let length_map = length_params_with("n", "arr");
    let _func = func_with_params_and_body(vec![], vec![HirStatement::Expression(expr)]);
    // We need to test through transform_function, but length params come from array detection.
    // Instead, test the private method directly via the wrapper.
    let result = gen.transform_expression_with_length_replacement(
        &HirExpression::Dereference(Box::new(HirExpression::Variable("n".to_string()))),
        &empty_inferences(),
        &length_map,
    );
    // The inner variable "n" should be replaced with arr.len() call,
    // and then wrapped in Dereference
    if let HirExpression::Dereference(inner) = &result {
        if let HirExpression::StringMethodCall {
            receiver, method, ..
        } = &**inner
        {
            assert_eq!(method, "len");
            assert!(matches!(&**receiver, HirExpression::Variable(n) if n == "arr"));
        } else {
            panic!("Expected StringMethodCall inside Dereference, got {:?}", inner);
        }
    } else {
        panic!("Expected Dereference, got {:?}", result);
    }
}

// ============================================================================
// Section 2: AddressOf transformation
// ============================================================================

#[test]
fn deep_address_of_simple() {
    let gen = BorrowGenerator::new();
    let result = gen.transform_expression_with_length_replacement(
        &HirExpression::AddressOf(Box::new(HirExpression::Variable("x".to_string()))),
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::AddressOf(inner) = &result {
        assert!(matches!(&**inner, HirExpression::Variable(n) if n == "x"));
    } else {
        panic!("Expected AddressOf");
    }
}

#[test]
fn deep_address_of_with_length_replacement() {
    let gen = BorrowGenerator::new();
    let length_map = length_params_with("n", "arr");
    let result = gen.transform_expression_with_length_replacement(
        &HirExpression::AddressOf(Box::new(HirExpression::Variable("n".to_string()))),
        &empty_inferences(),
        &length_map,
    );
    if let HirExpression::AddressOf(inner) = &result {
        assert!(matches!(&**inner, HirExpression::StringMethodCall { method, .. } if method == "len"));
    } else {
        panic!("Expected AddressOf");
    }
}

// ============================================================================
// Section 3: UnaryOp transformation
// ============================================================================

#[test]
fn deep_unary_minus() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::Minus,
        operand: Box::new(HirExpression::IntLiteral(42)),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::UnaryOp { op, operand } = &result {
        assert!(matches!(op, UnaryOperator::Minus));
        assert!(matches!(&**operand, HirExpression::IntLiteral(42)));
    } else {
        panic!("Expected UnaryOp");
    }
}

#[test]
fn deep_unary_logical_not_with_length() {
    let gen = BorrowGenerator::new();
    let length_map = length_params_with("n", "arr");
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("n".to_string())),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &length_map,
    );
    if let HirExpression::UnaryOp { op, operand } = &result {
        assert!(matches!(op, UnaryOperator::LogicalNot));
        assert!(matches!(&**operand, HirExpression::StringMethodCall { method, .. } if method == "len"));
    } else {
        panic!("Expected UnaryOp");
    }
}

#[test]
fn deep_unary_bitwise_not() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::BitwiseNot,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::UnaryOp { op, .. } = &result {
        assert!(matches!(op, UnaryOperator::BitwiseNot));
    } else {
        panic!("Expected UnaryOp");
    }
}

// ============================================================================
// Section 4: BinaryOp transformation
// ============================================================================

#[test]
fn deep_binary_add_simple() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::IntLiteral(1)),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::BinaryOp { op, left, right } = &result {
        assert!(matches!(op, BinaryOperator::Add));
        assert!(matches!(&**left, HirExpression::Variable(n) if n == "x"));
        assert!(matches!(&**right, HirExpression::IntLiteral(1)));
    } else {
        panic!("Expected BinaryOp");
    }
}

#[test]
fn deep_binary_subtract_with_length() {
    let gen = BorrowGenerator::new();
    let length_map = length_params_with("n", "arr");
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("n".to_string())),
        right: Box::new(HirExpression::IntLiteral(1)),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &length_map,
    );
    if let HirExpression::BinaryOp { op, left, right } = &result {
        assert!(matches!(op, BinaryOperator::Subtract));
        // left should be replaced with arr.len()
        assert!(matches!(&**left, HirExpression::StringMethodCall { method, .. } if method == "len"));
        assert!(matches!(&**right, HirExpression::IntLiteral(1)));
    } else {
        panic!("Expected BinaryOp");
    }
}

#[test]
fn deep_binary_multiply() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::IntLiteral(2)),
        right: Box::new(HirExpression::IntLiteral(3)),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::BinaryOp { op, .. } = &result {
        assert!(matches!(op, BinaryOperator::Multiply));
    } else {
        panic!("Expected BinaryOp");
    }
}

#[test]
fn deep_binary_equal() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    assert!(matches!(&result, HirExpression::BinaryOp { op: BinaryOperator::Equal, .. }));
}

// ============================================================================
// Section 5: FunctionCall transformation
// ============================================================================

#[test]
fn deep_function_call_no_args() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "foo".to_string(),
        arguments: vec![],
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::FunctionCall {
        function,
        arguments,
    } = &result
    {
        assert_eq!(function, "foo");
        assert!(arguments.is_empty());
    } else {
        panic!("Expected FunctionCall");
    }
}

#[test]
fn deep_function_call_with_args() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "bar".to_string(),
        arguments: vec![
            HirExpression::Variable("x".to_string()),
            HirExpression::IntLiteral(42),
        ],
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::FunctionCall {
        function,
        arguments,
    } = &result
    {
        assert_eq!(function, "bar");
        assert_eq!(arguments.len(), 2);
    } else {
        panic!("Expected FunctionCall");
    }
}

#[test]
fn deep_function_call_with_length_arg() {
    let gen = BorrowGenerator::new();
    let length_map = length_params_with("n", "arr");
    let expr = HirExpression::FunctionCall {
        function: "process".to_string(),
        arguments: vec![
            HirExpression::Variable("arr".to_string()),
            HirExpression::Variable("n".to_string()),
        ],
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &length_map,
    );
    if let HirExpression::FunctionCall { arguments, .. } = &result {
        assert_eq!(arguments.len(), 2);
        // Second arg "n" should become arr.len()
        assert!(matches!(
            &arguments[1],
            HirExpression::StringMethodCall { method, .. } if method == "len"
        ));
    } else {
        panic!("Expected FunctionCall");
    }
}

// ============================================================================
// Section 6: FieldAccess transformation
// ============================================================================

#[test]
fn deep_field_access_simple() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::FieldAccess {
        object: Box::new(HirExpression::Variable("obj".to_string())),
        field: "x".to_string(),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::FieldAccess { object, field } = &result {
        assert!(matches!(&**object, HirExpression::Variable(n) if n == "obj"));
        assert_eq!(field, "x");
    } else {
        panic!("Expected FieldAccess");
    }
}

#[test]
fn deep_field_access_with_length_object() {
    let gen = BorrowGenerator::new();
    let length_map = length_params_with("n", "arr");
    let expr = HirExpression::FieldAccess {
        object: Box::new(HirExpression::Variable("n".to_string())),
        field: "value".to_string(),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &length_map,
    );
    if let HirExpression::FieldAccess { object, field } = &result {
        assert!(matches!(&**object, HirExpression::StringMethodCall { method, .. } if method == "len"));
        assert_eq!(field, "value");
    } else {
        panic!("Expected FieldAccess");
    }
}

// ============================================================================
// Section 7: PointerFieldAccess transformation
// ============================================================================

#[test]
fn deep_pointer_field_access_simple() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("ptr".to_string())),
        field: "data".to_string(),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::PointerFieldAccess { pointer, field } = &result {
        assert!(matches!(&**pointer, HirExpression::Variable(n) if n == "ptr"));
        assert_eq!(field, "data");
    } else {
        panic!("Expected PointerFieldAccess");
    }
}

#[test]
fn deep_pointer_field_access_with_length() {
    let gen = BorrowGenerator::new();
    let length_map = length_params_with("n", "arr");
    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("n".to_string())),
        field: "count".to_string(),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &length_map,
    );
    if let HirExpression::PointerFieldAccess { pointer, field } = &result {
        assert!(matches!(&**pointer, HirExpression::StringMethodCall { method, .. } if method == "len"));
        assert_eq!(field, "count");
    } else {
        panic!("Expected PointerFieldAccess");
    }
}

// ============================================================================
// Section 8: ArrayIndex transformation
// ============================================================================

#[test]
fn deep_array_index_simple() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::ArrayIndex { array, index } = &result {
        assert!(matches!(&**array, HirExpression::Variable(n) if n == "arr"));
        assert!(matches!(&**index, HirExpression::IntLiteral(0)));
    } else {
        panic!("Expected ArrayIndex");
    }
}

#[test]
fn deep_array_index_with_length_in_index() {
    let gen = BorrowGenerator::new();
    let length_map = length_params_with("n", "data");
    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::Variable("n".to_string())),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &length_map,
    );
    if let HirExpression::ArrayIndex { array, index } = &result {
        assert!(matches!(&**array, HirExpression::Variable(n) if n == "arr"));
        // index "n" becomes data.len()
        assert!(matches!(&**index, HirExpression::StringMethodCall { method, .. } if method == "len"));
    } else {
        panic!("Expected ArrayIndex");
    }
}

#[test]
fn deep_array_index_variable_index() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("buf".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    assert!(matches!(&result, HirExpression::ArrayIndex { .. }));
}

// ============================================================================
// Section 9: Cast transformation
// ============================================================================

#[test]
fn deep_cast_int_to_float() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::Cast {
        expr: Box::new(HirExpression::Variable("x".to_string())),
        target_type: HirType::Float,
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::Cast {
        expr: inner,
        target_type,
    } = &result
    {
        assert!(matches!(&**inner, HirExpression::Variable(n) if n == "x"));
        assert_eq!(*target_type, HirType::Float);
    } else {
        panic!("Expected Cast");
    }
}

#[test]
fn deep_cast_with_length_replacement() {
    let gen = BorrowGenerator::new();
    let length_map = length_params_with("n", "arr");
    let expr = HirExpression::Cast {
        expr: Box::new(HirExpression::Variable("n".to_string())),
        target_type: HirType::Int,
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &length_map,
    );
    if let HirExpression::Cast { expr: inner, .. } = &result {
        assert!(matches!(&**inner, HirExpression::StringMethodCall { method, .. } if method == "len"));
    } else {
        panic!("Expected Cast");
    }
}

// ============================================================================
// Section 10: CompoundLiteral transformation
// ============================================================================

#[test]
fn deep_compound_literal() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Point".to_string()),
        initializers: vec![HirExpression::IntLiteral(10), HirExpression::IntLiteral(20)],
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::CompoundLiteral {
        literal_type,
        initializers,
    } = &result
    {
        assert_eq!(*literal_type, HirType::Struct("Point".to_string()));
        assert_eq!(initializers.len(), 2);
    } else {
        panic!("Expected CompoundLiteral");
    }
}

#[test]
fn deep_compound_literal_with_length_init() {
    let gen = BorrowGenerator::new();
    let length_map = length_params_with("n", "arr");
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Data".to_string()),
        initializers: vec![
            HirExpression::Variable("n".to_string()),
            HirExpression::IntLiteral(0),
        ],
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &length_map,
    );
    if let HirExpression::CompoundLiteral { initializers, .. } = &result {
        assert_eq!(initializers.len(), 2);
        assert!(matches!(
            &initializers[0],
            HirExpression::StringMethodCall { method, .. } if method == "len"
        ));
    } else {
        panic!("Expected CompoundLiteral");
    }
}

// ============================================================================
// Section 11: IsNotNull transformation
// ============================================================================

#[test]
fn deep_is_not_null() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::IsNotNull(Box::new(HirExpression::Variable("ptr".to_string())));
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::IsNotNull(inner) = &result {
        assert!(matches!(&**inner, HirExpression::Variable(n) if n == "ptr"));
    } else {
        panic!("Expected IsNotNull");
    }
}

// ============================================================================
// Section 12: Calloc transformation
// ============================================================================

#[test]
fn deep_calloc_simple() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(10)),
        element_type: Box::new(HirType::Int),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::Calloc {
        count,
        element_type,
    } = &result
    {
        assert!(matches!(&**count, HirExpression::IntLiteral(10)));
        assert_eq!(**element_type, HirType::Int);
    } else {
        panic!("Expected Calloc");
    }
}

#[test]
fn deep_calloc_with_length_count() {
    let gen = BorrowGenerator::new();
    let length_map = length_params_with("n", "arr");
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::Variable("n".to_string())),
        element_type: Box::new(HirType::Int),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &length_map,
    );
    if let HirExpression::Calloc { count, .. } = &result {
        assert!(matches!(&**count, HirExpression::StringMethodCall { method, .. } if method == "len"));
    } else {
        panic!("Expected Calloc");
    }
}

// ============================================================================
// Section 13: Malloc transformation
// ============================================================================

#[test]
fn deep_malloc_simple() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::Malloc {
        size: Box::new(HirExpression::IntLiteral(100)),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::Malloc { size } = &result {
        assert!(matches!(&**size, HirExpression::IntLiteral(100)));
    } else {
        panic!("Expected Malloc");
    }
}

#[test]
fn deep_malloc_with_length_size() {
    let gen = BorrowGenerator::new();
    let length_map = length_params_with("n", "arr");
    let expr = HirExpression::Malloc {
        size: Box::new(HirExpression::Variable("n".to_string())),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &length_map,
    );
    if let HirExpression::Malloc { size } = &result {
        assert!(matches!(&**size, HirExpression::StringMethodCall { method, .. } if method == "len"));
    } else {
        panic!("Expected Malloc");
    }
}

// ============================================================================
// Section 14: Realloc transformation
// ============================================================================

#[test]
fn deep_realloc_simple() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::Variable("buf".to_string())),
        new_size: Box::new(HirExpression::IntLiteral(200)),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::Realloc { pointer, new_size } = &result {
        assert!(matches!(&**pointer, HirExpression::Variable(n) if n == "buf"));
        assert!(matches!(&**new_size, HirExpression::IntLiteral(200)));
    } else {
        panic!("Expected Realloc");
    }
}

#[test]
fn deep_realloc_with_length_in_new_size() {
    let gen = BorrowGenerator::new();
    let length_map = length_params_with("n", "arr");
    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::Variable("ptr".to_string())),
        new_size: Box::new(HirExpression::Variable("n".to_string())),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &length_map,
    );
    if let HirExpression::Realloc { new_size, .. } = &result {
        assert!(matches!(&**new_size, HirExpression::StringMethodCall { method, .. } if method == "len"));
    } else {
        panic!("Expected Realloc");
    }
}

// ============================================================================
// Section 15: StringMethodCall transformation
// ============================================================================

#[test]
fn deep_string_method_call() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "len".to_string(),
        arguments: vec![],
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::StringMethodCall {
        receiver,
        method,
        arguments,
    } = &result
    {
        assert!(matches!(&**receiver, HirExpression::Variable(n) if n == "s"));
        assert_eq!(method, "len");
        assert!(arguments.is_empty());
    } else {
        panic!("Expected StringMethodCall");
    }
}

#[test]
fn deep_string_method_call_with_args() {
    let gen = BorrowGenerator::new();
    let length_map = length_params_with("n", "arr");
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "split".to_string(),
        arguments: vec![HirExpression::Variable("n".to_string())],
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &length_map,
    );
    if let HirExpression::StringMethodCall { arguments, .. } = &result {
        assert_eq!(arguments.len(), 1);
        assert!(matches!(
            &arguments[0],
            HirExpression::StringMethodCall { method, .. } if method == "len"
        ));
    } else {
        panic!("Expected StringMethodCall");
    }
}

// ============================================================================
// Section 16: SliceIndex transformation
// ============================================================================

#[test]
fn deep_slice_index() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::SliceIndex {
        slice: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(3)),
        element_type: HirType::Int,
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::SliceIndex {
        slice,
        index,
        element_type,
    } = &result
    {
        assert!(matches!(&**slice, HirExpression::Variable(n) if n == "arr"));
        assert!(matches!(&**index, HirExpression::IntLiteral(3)));
        assert_eq!(*element_type, HirType::Int);
    } else {
        panic!("Expected SliceIndex");
    }
}

#[test]
fn deep_slice_index_with_length_in_index() {
    let gen = BorrowGenerator::new();
    let length_map = length_params_with("n", "data");
    let expr = HirExpression::SliceIndex {
        slice: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::Variable("n".to_string())),
        element_type: HirType::Float,
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &length_map,
    );
    if let HirExpression::SliceIndex { index, .. } = &result {
        assert!(matches!(&**index, HirExpression::StringMethodCall { method, .. } if method == "len"));
    } else {
        panic!("Expected SliceIndex");
    }
}

// ============================================================================
// Section 17: PostIncrement / PreIncrement / PostDecrement / PreDecrement
// ============================================================================

#[test]
fn deep_post_increment() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Variable("i".to_string())),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::PostIncrement { operand } = &result {
        assert!(matches!(&**operand, HirExpression::Variable(n) if n == "i"));
    } else {
        panic!("Expected PostIncrement");
    }
}

#[test]
fn deep_pre_increment() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::PreIncrement {
        operand: Box::new(HirExpression::Variable("j".to_string())),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::PreIncrement { operand } = &result {
        assert!(matches!(&**operand, HirExpression::Variable(n) if n == "j"));
    } else {
        panic!("Expected PreIncrement");
    }
}

#[test]
fn deep_post_decrement() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::PostDecrement {
        operand: Box::new(HirExpression::Variable("k".to_string())),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::PostDecrement { operand } = &result {
        assert!(matches!(&**operand, HirExpression::Variable(n) if n == "k"));
    } else {
        panic!("Expected PostDecrement");
    }
}

#[test]
fn deep_pre_decrement() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::PreDecrement {
        operand: Box::new(HirExpression::Variable("m".to_string())),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::PreDecrement { operand } = &result {
        assert!(matches!(&**operand, HirExpression::Variable(n) if n == "m"));
    } else {
        panic!("Expected PreDecrement");
    }
}

// ============================================================================
// Section 18: Ternary transformation
// ============================================================================

#[test]
fn deep_ternary_simple() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::Variable("flag".to_string())),
        then_expr: Box::new(HirExpression::IntLiteral(1)),
        else_expr: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::Ternary {
        condition,
        then_expr,
        else_expr,
    } = &result
    {
        assert!(matches!(&**condition, HirExpression::Variable(n) if n == "flag"));
        assert!(matches!(&**then_expr, HirExpression::IntLiteral(1)));
        assert!(matches!(&**else_expr, HirExpression::IntLiteral(0)));
    } else {
        panic!("Expected Ternary");
    }
}

#[test]
fn deep_ternary_with_length_in_condition() {
    let gen = BorrowGenerator::new();
    let length_map = length_params_with("n", "arr");
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::Variable("n".to_string())),
        then_expr: Box::new(HirExpression::IntLiteral(1)),
        else_expr: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &length_map,
    );
    if let HirExpression::Ternary { condition, .. } = &result {
        assert!(matches!(&**condition, HirExpression::StringMethodCall { method, .. } if method == "len"));
    } else {
        panic!("Expected Ternary");
    }
}

#[test]
fn deep_ternary_with_length_in_branches() {
    let gen = BorrowGenerator::new();
    let length_map = length_params_with("n", "arr");
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::IntLiteral(1)),
        then_expr: Box::new(HirExpression::Variable("n".to_string())),
        else_expr: Box::new(HirExpression::Variable("n".to_string())),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &length_map,
    );
    if let HirExpression::Ternary {
        then_expr,
        else_expr,
        ..
    } = &result
    {
        assert!(matches!(&**then_expr, HirExpression::StringMethodCall { method, .. } if method == "len"));
        assert!(matches!(&**else_expr, HirExpression::StringMethodCall { method, .. } if method == "len"));
    } else {
        panic!("Expected Ternary");
    }
}

// ============================================================================
// Section 19: Leaf expressions (pass-through)
// ============================================================================

#[test]
fn deep_leaf_int_literal() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::IntLiteral(42);
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    assert!(matches!(&result, HirExpression::IntLiteral(42)));
}

#[test]
fn deep_leaf_float_literal() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::FloatLiteral("3.14".to_string());
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    assert!(matches!(&result, HirExpression::FloatLiteral(s) if s == "3.14"));
}

#[test]
fn deep_leaf_string_literal() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::StringLiteral("hello".to_string());
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    assert!(matches!(&result, HirExpression::StringLiteral(s) if s == "hello"));
}

#[test]
fn deep_leaf_char_literal() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::CharLiteral(65);
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    assert!(matches!(&result, HirExpression::CharLiteral(65)));
}

#[test]
fn deep_leaf_variable_no_replacement() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::Variable("x".to_string());
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    assert!(matches!(&result, HirExpression::Variable(n) if n == "x"));
}

#[test]
fn deep_leaf_sizeof() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::Sizeof {
        type_name: "int".to_string(),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    assert!(matches!(&result, HirExpression::Sizeof { type_name } if type_name == "int"));
}

#[test]
fn deep_leaf_null_literal() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::NullLiteral;
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    assert!(matches!(&result, HirExpression::NullLiteral));
}

// ============================================================================
// Section 20: Pointer arithmetic to SliceIndex (DECY-070)
// ============================================================================

#[test]
fn deep_deref_add_array_pointer_to_slice_index() {
    let gen = BorrowGenerator::new();
    let mut inferences = HashMap::new();
    let (name, inf) = array_pointer_inference("arr", "arr");
    inferences.insert(name, inf);

    // *(arr + 2) should become SliceIndex { arr, 2 }
    let expr = HirExpression::Dereference(Box::new(HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::IntLiteral(2)),
    }));
    let result = gen.transform_expression_recursive_with_length(
        &expr,
        &inferences,
        &empty_length_params(),
    );
    if let HirExpression::SliceIndex {
        slice,
        index,
        element_type,
    } = &result
    {
        assert!(matches!(&**slice, HirExpression::Variable(n) if n == "arr"));
        assert!(matches!(&**index, HirExpression::IntLiteral(2)));
        assert_eq!(*element_type, HirType::Int);
    } else {
        panic!("Expected SliceIndex from deref-add pattern, got {:?}", result);
    }
}

#[test]
fn deep_deref_subtract_array_pointer_to_slice_index() {
    let gen = BorrowGenerator::new();
    let mut inferences = HashMap::new();
    let (name, inf) = array_pointer_inference("arr", "arr");
    inferences.insert(name, inf);

    // *(arr - 1) should become SliceIndex { arr, 1 }
    let expr = HirExpression::Dereference(Box::new(HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::IntLiteral(1)),
    }));
    let result = gen.transform_expression_recursive_with_length(
        &expr,
        &inferences,
        &empty_length_params(),
    );
    assert!(matches!(&result, HirExpression::SliceIndex { .. }));
}

#[test]
fn deep_deref_add_non_array_pointer_stays_deref() {
    let gen = BorrowGenerator::new();
    // No inference for "ptr", so it stays as dereference
    let expr = HirExpression::Dereference(Box::new(HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::IntLiteral(1)),
    }));
    let result = gen.transform_expression_recursive_with_length(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    // Should remain Dereference since no ArrayPointer inference
    assert!(matches!(&result, HirExpression::Dereference(_)));
}

#[test]
fn deep_deref_multiply_no_transform() {
    let gen = BorrowGenerator::new();
    let mut inferences = HashMap::new();
    let (name, inf) = array_pointer_inference("arr", "arr");
    inferences.insert(name, inf);

    // *(arr * 2) should NOT transform to SliceIndex (multiply is not pointer arithmetic)
    let expr = HirExpression::Dereference(Box::new(HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::IntLiteral(2)),
    }));
    let result = gen.transform_expression_recursive_with_length(
        &expr,
        &inferences,
        &empty_length_params(),
    );
    assert!(matches!(&result, HirExpression::Dereference(_)));
}

// ============================================================================
// Section 21: Length variable replacement
// ============================================================================

#[test]
fn deep_variable_length_replaced() {
    let gen = BorrowGenerator::new();
    let length_map = length_params_with("len", "data");
    let expr = HirExpression::Variable("len".to_string());
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &length_map,
    );
    if let HirExpression::StringMethodCall {
        receiver,
        method,
        arguments,
    } = &result
    {
        assert_eq!(method, "len");
        assert!(arguments.is_empty());
        assert!(matches!(&**receiver, HirExpression::Variable(n) if n == "data"));
    } else {
        panic!("Expected StringMethodCall for length replacement");
    }
}

#[test]
fn deep_variable_not_in_length_map() {
    let gen = BorrowGenerator::new();
    let length_map = length_params_with("len", "data");
    let expr = HirExpression::Variable("other".to_string());
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &length_map,
    );
    assert!(matches!(&result, HirExpression::Variable(n) if n == "other"));
}

// ============================================================================
// Section 22: Nested/complex expressions
// ============================================================================

#[test]
fn deep_nested_binary_in_array_index() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        }),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::ArrayIndex { index, .. } = &result {
        assert!(matches!(&**index, HirExpression::BinaryOp { op: BinaryOperator::Add, .. }));
    } else {
        panic!("Expected ArrayIndex");
    }
}

#[test]
fn deep_nested_deref_in_field_access() {
    let gen = BorrowGenerator::new();
    let expr = HirExpression::FieldAccess {
        object: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("ptr".to_string()),
        ))),
        field: "value".to_string(),
    };
    let result = gen.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    if let HirExpression::FieldAccess { object, field } = &result {
        assert!(matches!(&**object, HirExpression::Dereference(_)));
        assert_eq!(field, "value");
    } else {
        panic!("Expected FieldAccess");
    }
}

#[test]
fn deep_deref_add_with_length_in_offset() {
    let gen = BorrowGenerator::new();
    let mut inferences = HashMap::new();
    let (name, inf) = array_pointer_inference("arr", "arr");
    inferences.insert(name, inf);
    let length_map = length_params_with("n", "arr");

    // *(arr + n) => SliceIndex { arr, arr.len() }
    let expr = HirExpression::Dereference(Box::new(HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::Variable("n".to_string())),
    }));
    let result = gen.transform_expression_recursive_with_length(
        &expr,
        &inferences,
        &length_map,
    );
    if let HirExpression::SliceIndex { index, .. } = &result {
        // The index "n" should be replaced with arr.len()
        assert!(matches!(&**index, HirExpression::StringMethodCall { method, .. } if method == "len"));
    } else {
        panic!("Expected SliceIndex with len() replacement, got {:?}", result);
    }
}
