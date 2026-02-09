//! Tests for transform_expression_recursive_with_length and
//! transform_expression_with_length_replacement in borrow_gen.
//!
//! Exercises every branch of the recursive expression transformation,
//! including pointer arithmetic to SliceIndex, length parameter replacement,
//! and recursive child transformation for all HirExpression variants.

use super::*;
use crate::inference::{OwnershipInference, OwnershipKind};
use decy_hir::{BinaryOperator, HirExpression, HirType, UnaryOperator};
use std::collections::HashMap;

// ============================================================================
// Helper: create BorrowGenerator + empty context
// ============================================================================

fn gen() -> BorrowGenerator {
    BorrowGenerator::new()
}

fn empty_inferences() -> HashMap<String, OwnershipInference> {
    HashMap::new()
}

fn empty_length_params() -> HashMap<String, String> {
    HashMap::new()
}

fn array_pointer_inference(
    var: &str,
    base_array: &str,
    element_type: HirType,
) -> (String, OwnershipInference) {
    (
        var.to_string(),
        OwnershipInference {
            variable: var.to_string(),
            kind: OwnershipKind::ArrayPointer {
                base_array: base_array.to_string(),
                element_type,
                base_index: None,
            },
            confidence: 0.9,
            reason: "Array pointer inference for test".to_string(),
        },
    )
}

// ============================================================================
// Leaf expressions: no transformation needed
// ============================================================================

#[test]
fn test_leaf_int_literal_unchanged() {
    let g = gen();
    let expr = HirExpression::IntLiteral(42);
    let result = g.transform_expression_recursive_with_length(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    assert_eq!(result, HirExpression::IntLiteral(42));
}

#[test]
fn test_leaf_float_literal_unchanged() {
    let g = gen();
    let expr = HirExpression::FloatLiteral("3.14".to_string());
    let result = g.transform_expression_recursive_with_length(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    assert_eq!(result, HirExpression::FloatLiteral("3.14".to_string()));
}

#[test]
fn test_leaf_string_literal_unchanged() {
    let g = gen();
    let expr = HirExpression::StringLiteral("hello".to_string());
    let result = g.transform_expression_recursive_with_length(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    assert_eq!(result, HirExpression::StringLiteral("hello".to_string()));
}

#[test]
fn test_leaf_char_literal_unchanged() {
    let g = gen();
    let expr = HirExpression::CharLiteral(b'a' as i8);
    let result = g.transform_expression_recursive_with_length(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    assert_eq!(result, HirExpression::CharLiteral(b'a' as i8));
}

#[test]
fn test_leaf_variable_unchanged_no_length_mapping() {
    let g = gen();
    let expr = HirExpression::Variable("x".to_string());
    let result = g.transform_expression_recursive_with_length(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    assert_eq!(result, HirExpression::Variable("x".to_string()));
}

#[test]
fn test_leaf_sizeof_unchanged() {
    let g = gen();
    let expr = HirExpression::Sizeof {
        type_name: "int".to_string(),
    };
    let result = g.transform_expression_recursive_with_length(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    assert_eq!(
        result,
        HirExpression::Sizeof {
            type_name: "int".to_string()
        }
    );
}

#[test]
fn test_leaf_null_literal_unchanged() {
    let g = gen();
    let expr = HirExpression::NullLiteral;
    let result = g.transform_expression_recursive_with_length(
        &expr,
        &empty_inferences(),
        &empty_length_params(),
    );
    assert_eq!(result, HirExpression::NullLiteral);
}

// ============================================================================
// Pointer arithmetic to SliceIndex (DECY-070)
// ============================================================================

#[test]
fn test_deref_add_to_slice_index() {
    // *(arr + i) with arr as ArrayPointer => arr[i] (SliceIndex)
    let g = gen();
    let mut inferences = HashMap::new();
    let (k, v) = array_pointer_inference("arr", "arr", HirType::Int);
    inferences.insert(k, v);

    let expr = HirExpression::Dereference(Box::new(HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::Variable("i".to_string())),
    }));

    let result =
        g.transform_expression_recursive_with_length(&expr, &inferences, &empty_length_params());

    match result {
        HirExpression::SliceIndex {
            slice,
            index,
            element_type,
        } => {
            assert_eq!(*slice, HirExpression::Variable("arr".to_string()));
            assert_eq!(*index, HirExpression::Variable("i".to_string()));
            assert_eq!(element_type, HirType::Int);
        }
        other => panic!("Expected SliceIndex, got {:?}", other),
    }
}

#[test]
fn test_deref_subtract_to_slice_index() {
    // *(arr - i) with arr as ArrayPointer => SliceIndex
    let g = gen();
    let mut inferences = HashMap::new();
    let (k, v) = array_pointer_inference("arr", "arr", HirType::Char);
    inferences.insert(k, v);

    let expr = HirExpression::Dereference(Box::new(HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::IntLiteral(1)),
    }));

    let result =
        g.transform_expression_recursive_with_length(&expr, &inferences, &empty_length_params());

    match result {
        HirExpression::SliceIndex {
            element_type, ..
        } => {
            assert_eq!(element_type, HirType::Char);
        }
        other => panic!("Expected SliceIndex, got {:?}", other),
    }
}

#[test]
fn test_deref_add_non_array_pointer_no_transform() {
    // *(ptr + i) where ptr is not ArrayPointer - no transformation
    let g = gen();
    let mut inferences = HashMap::new();
    inferences.insert(
        "ptr".to_string(),
        OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::ImmutableBorrow,
            confidence: 0.8,
            reason: "not array".to_string(),
        },
    );

    let expr = HirExpression::Dereference(Box::new(HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::IntLiteral(1)),
    }));

    let result =
        g.transform_expression_recursive_with_length(&expr, &inferences, &empty_length_params());

    // Should fall through to the normal Dereference arm
    assert!(matches!(result, HirExpression::Dereference(_)));
}

#[test]
fn test_deref_add_unknown_variable_no_transform() {
    // *(unknown + i) where unknown is not in inferences
    let g = gen();
    let expr = HirExpression::Dereference(Box::new(HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("unknown".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    }));

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &empty_length_params());

    assert!(matches!(result, HirExpression::Dereference(_)));
}

#[test]
fn test_deref_multiply_no_transform() {
    // *(a * b) is not pointer arithmetic (Multiply op), falls through
    let g = gen();
    let expr = HirExpression::Dereference(Box::new(HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    }));

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &empty_length_params());

    assert!(matches!(result, HirExpression::Dereference(_)));
}

#[test]
fn test_deref_non_binary_inner_no_transform() {
    // *(variable) without BinaryOp inside, just a plain dereference
    let g = gen();
    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("x".to_string())));

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &empty_length_params());

    assert!(matches!(result, HirExpression::Dereference(_)));
}

// ============================================================================
// Length parameter replacement (DECY-072)
// ============================================================================

#[test]
fn test_length_param_replaced_with_len_call() {
    // When a variable is in length_params_to_remove, it gets replaced with arr.len()
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("n".to_string(), "data".to_string());

    let expr = HirExpression::Variable("n".to_string());
    // Call transform_expression_with_length_replacement which checks length params first
    let result = g.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &length_params,
    );

    match result {
        HirExpression::StringMethodCall {
            receiver,
            method,
            arguments,
        } => {
            assert_eq!(*receiver, HirExpression::Variable("data".to_string()));
            assert_eq!(method, "len");
            assert!(arguments.is_empty());
        }
        other => panic!("Expected StringMethodCall len(), got {:?}", other),
    }
}

#[test]
fn test_length_param_not_replaced_when_not_in_map() {
    // Variable not in the length_params map stays unchanged
    let g = gen();
    let length_params = HashMap::new();

    let expr = HirExpression::Variable("n".to_string());
    let result = g.transform_expression_with_length_replacement(
        &expr,
        &empty_inferences(),
        &length_params,
    );

    assert_eq!(result, HirExpression::Variable("n".to_string()));
}

// ============================================================================
// Pointer arithmetic + length replacement combined (DECY-070 + DECY-072)
// ============================================================================

#[test]
fn test_deref_add_with_length_replacement_in_index() {
    // *(arr + n) where n is a length param -> arr[data.len()]
    let g = gen();
    let mut inferences = HashMap::new();
    let (k, v) = array_pointer_inference("arr", "arr", HirType::Int);
    inferences.insert(k, v);

    let mut length_params = HashMap::new();
    length_params.insert("n".to_string(), "data".to_string());

    let expr = HirExpression::Dereference(Box::new(HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::Variable("n".to_string())),
    }));

    let result =
        g.transform_expression_recursive_with_length(&expr, &inferences, &length_params);

    match result {
        HirExpression::SliceIndex {
            slice,
            index,
            element_type,
        } => {
            assert_eq!(*slice, HirExpression::Variable("arr".to_string()));
            // The index (n) should be replaced with data.len()
            match *index {
                HirExpression::StringMethodCall {
                    ref receiver,
                    ref method,
                    ref arguments,
                } => {
                    assert_eq!(**receiver, HirExpression::Variable("data".to_string()));
                    assert_eq!(method, "len");
                    assert!(arguments.is_empty());
                }
                ref other => panic!("Expected len() call in index, got {:?}", other),
            }
            assert_eq!(element_type, HirType::Int);
        }
        other => panic!("Expected SliceIndex with len() index, got {:?}", other),
    }
}

// ============================================================================
// Recursive child expression transformation
// ============================================================================

#[test]
fn test_binary_op_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("len".to_string(), "arr".to_string());

    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("len".to_string())),
        right: Box::new(HirExpression::IntLiteral(1)),
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::BinaryOp { op, left, right } => {
            assert_eq!(op, BinaryOperator::Add);
            // left should be replaced with arr.len()
            match *left {
                HirExpression::StringMethodCall { ref method, .. } => {
                    assert_eq!(method, "len");
                }
                ref other => panic!("Expected len() call, got {:?}", other),
            }
            assert_eq!(*right, HirExpression::IntLiteral(1));
        }
        other => panic!("Expected BinaryOp, got {:?}", other),
    }
}

#[test]
fn test_unary_op_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("n".to_string(), "arr".to_string());

    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::Minus,
        operand: Box::new(HirExpression::Variable("n".to_string())),
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::UnaryOp { op, operand } => {
            assert_eq!(op, UnaryOperator::Minus);
            assert!(matches!(*operand, HirExpression::StringMethodCall { .. }));
        }
        other => panic!("Expected UnaryOp, got {:?}", other),
    }
}

#[test]
fn test_address_of_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("sz".to_string(), "buf".to_string());

    let expr = HirExpression::AddressOf(Box::new(HirExpression::Variable("sz".to_string())));

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::AddressOf(inner) => {
            assert!(matches!(*inner, HirExpression::StringMethodCall { .. }));
        }
        other => panic!("Expected AddressOf, got {:?}", other),
    }
}

#[test]
fn test_dereference_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("sz".to_string(), "buf".to_string());

    // *sz where sz is a length param (not a pointer arithmetic pattern)
    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("sz".to_string())));

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::Dereference(inner) => {
            // Inner goes through transform_expression_with_length_replacement
            // which replaces sz -> buf.len()
            assert!(matches!(*inner, HirExpression::StringMethodCall { .. }));
        }
        other => panic!("Expected Dereference, got {:?}", other),
    }
}

#[test]
fn test_function_call_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("n".to_string(), "arr".to_string());

    let expr = HirExpression::FunctionCall {
        function: "process".to_string(),
        arguments: vec![
            HirExpression::Variable("n".to_string()),
            HirExpression::IntLiteral(0),
        ],
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::FunctionCall {
            function,
            arguments,
        } => {
            assert_eq!(function, "process");
            assert_eq!(arguments.len(), 2);
            assert!(matches!(
                arguments[0],
                HirExpression::StringMethodCall { .. }
            ));
            assert_eq!(arguments[1], HirExpression::IntLiteral(0));
        }
        other => panic!("Expected FunctionCall, got {:?}", other),
    }
}

#[test]
fn test_field_access_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("obj".to_string(), "arr".to_string());

    let expr = HirExpression::FieldAccess {
        object: Box::new(HirExpression::Variable("obj".to_string())),
        field: "count".to_string(),
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::FieldAccess { object, field } => {
            assert_eq!(field, "count");
            assert!(matches!(*object, HirExpression::StringMethodCall { .. }));
        }
        other => panic!("Expected FieldAccess, got {:?}", other),
    }
}

#[test]
fn test_pointer_field_access_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("ptr".to_string(), "arr".to_string());

    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("ptr".to_string())),
        field: "value".to_string(),
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::PointerFieldAccess { pointer, field } => {
            assert_eq!(field, "value");
            assert!(matches!(*pointer, HirExpression::StringMethodCall { .. }));
        }
        other => panic!("Expected PointerFieldAccess, got {:?}", other),
    }
}

#[test]
fn test_array_index_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("idx".to_string(), "arr".to_string());

    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("data".to_string())),
        index: Box::new(HirExpression::Variable("idx".to_string())),
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::ArrayIndex { array, index } => {
            assert_eq!(*array, HirExpression::Variable("data".to_string()));
            assert!(matches!(*index, HirExpression::StringMethodCall { .. }));
        }
        other => panic!("Expected ArrayIndex, got {:?}", other),
    }
}

#[test]
fn test_cast_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("n".to_string(), "arr".to_string());

    let expr = HirExpression::Cast {
        expr: Box::new(HirExpression::Variable("n".to_string())),
        target_type: HirType::Int,
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::Cast {
            expr: cast_expr,
            target_type,
        } => {
            assert_eq!(target_type, HirType::Int);
            assert!(matches!(*cast_expr, HirExpression::StringMethodCall { .. }));
        }
        other => panic!("Expected Cast, got {:?}", other),
    }
}

#[test]
fn test_compound_literal_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("n".to_string(), "arr".to_string());

    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Int,
        initializers: vec![
            HirExpression::Variable("n".to_string()),
            HirExpression::IntLiteral(10),
        ],
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::CompoundLiteral {
            literal_type,
            initializers,
        } => {
            assert_eq!(literal_type, HirType::Int);
            assert_eq!(initializers.len(), 2);
            assert!(matches!(
                initializers[0],
                HirExpression::StringMethodCall { .. }
            ));
            assert_eq!(initializers[1], HirExpression::IntLiteral(10));
        }
        other => panic!("Expected CompoundLiteral, got {:?}", other),
    }
}

#[test]
fn test_is_not_null_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("ptr".to_string(), "arr".to_string());

    let expr = HirExpression::IsNotNull(Box::new(HirExpression::Variable("ptr".to_string())));

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::IsNotNull(inner) => {
            assert!(matches!(*inner, HirExpression::StringMethodCall { .. }));
        }
        other => panic!("Expected IsNotNull, got {:?}", other),
    }
}

#[test]
fn test_calloc_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("n".to_string(), "arr".to_string());

    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::Variable("n".to_string())),
        element_type: Box::new(HirType::Int),
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::Calloc {
            count,
            element_type,
        } => {
            assert!(matches!(*count, HirExpression::StringMethodCall { .. }));
            assert_eq!(*element_type, HirType::Int);
        }
        other => panic!("Expected Calloc, got {:?}", other),
    }
}

#[test]
fn test_malloc_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("n".to_string(), "arr".to_string());

    let expr = HirExpression::Malloc {
        size: Box::new(HirExpression::Variable("n".to_string())),
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::Malloc { size } => {
            assert!(matches!(*size, HirExpression::StringMethodCall { .. }));
        }
        other => panic!("Expected Malloc, got {:?}", other),
    }
}

#[test]
fn test_realloc_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("n".to_string(), "arr".to_string());
    length_params.insert("ptr".to_string(), "buf".to_string());

    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::Variable("ptr".to_string())),
        new_size: Box::new(HirExpression::Variable("n".to_string())),
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::Realloc { pointer, new_size } => {
            assert!(matches!(*pointer, HirExpression::StringMethodCall { .. }));
            assert!(matches!(*new_size, HirExpression::StringMethodCall { .. }));
        }
        other => panic!("Expected Realloc, got {:?}", other),
    }
}

#[test]
fn test_string_method_call_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("s".to_string(), "arr".to_string());

    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "to_string".to_string(),
        arguments: vec![HirExpression::Variable("s".to_string())],
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::StringMethodCall {
            receiver,
            method,
            arguments,
        } => {
            assert_eq!(method, "to_string");
            assert!(matches!(*receiver, HirExpression::StringMethodCall { .. }));
            assert!(matches!(
                arguments[0],
                HirExpression::StringMethodCall { .. }
            ));
        }
        other => panic!("Expected StringMethodCall, got {:?}", other),
    }
}

#[test]
fn test_slice_index_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("idx".to_string(), "arr".to_string());

    let expr = HirExpression::SliceIndex {
        slice: Box::new(HirExpression::Variable("data".to_string())),
        index: Box::new(HirExpression::Variable("idx".to_string())),
        element_type: HirType::Int,
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::SliceIndex {
            slice,
            index,
            element_type,
        } => {
            assert_eq!(*slice, HirExpression::Variable("data".to_string()));
            assert!(matches!(*index, HirExpression::StringMethodCall { .. }));
            assert_eq!(element_type, HirType::Int);
        }
        other => panic!("Expected SliceIndex, got {:?}", other),
    }
}

// ============================================================================
// Increment/decrement expressions (DECY-139)
// ============================================================================

#[test]
fn test_post_increment_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("x".to_string(), "arr".to_string());

    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::PostIncrement { operand } => {
            assert!(matches!(*operand, HirExpression::StringMethodCall { .. }));
        }
        other => panic!("Expected PostIncrement, got {:?}", other),
    }
}

#[test]
fn test_pre_increment_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("x".to_string(), "arr".to_string());

    let expr = HirExpression::PreIncrement {
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::PreIncrement { operand } => {
            assert!(matches!(*operand, HirExpression::StringMethodCall { .. }));
        }
        other => panic!("Expected PreIncrement, got {:?}", other),
    }
}

#[test]
fn test_post_decrement_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("x".to_string(), "arr".to_string());

    let expr = HirExpression::PostDecrement {
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::PostDecrement { operand } => {
            assert!(matches!(*operand, HirExpression::StringMethodCall { .. }));
        }
        other => panic!("Expected PostDecrement, got {:?}", other),
    }
}

#[test]
fn test_pre_decrement_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("x".to_string(), "arr".to_string());

    let expr = HirExpression::PreDecrement {
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::PreDecrement { operand } => {
            assert!(matches!(*operand, HirExpression::StringMethodCall { .. }));
        }
        other => panic!("Expected PreDecrement, got {:?}", other),
    }
}

// ============================================================================
// Ternary expression (DECY-192)
// ============================================================================

#[test]
fn test_ternary_recursive_transformation() {
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("n".to_string(), "arr".to_string());

    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::Variable("n".to_string())),
        then_expr: Box::new(HirExpression::Variable("n".to_string())),
        else_expr: Box::new(HirExpression::IntLiteral(0)),
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            assert!(matches!(*condition, HirExpression::StringMethodCall { .. }));
            assert!(matches!(*then_expr, HirExpression::StringMethodCall { .. }));
            assert_eq!(*else_expr, HirExpression::IntLiteral(0));
        }
        other => panic!("Expected Ternary, got {:?}", other),
    }
}

// ============================================================================
// Nested expression trees
// ============================================================================

#[test]
fn test_deeply_nested_binary_ops_with_length_replacement() {
    // (a + (b + n)) where n is a length param
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("n".to_string(), "arr".to_string());

    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("b".to_string())),
            right: Box::new(HirExpression::Variable("n".to_string())),
        }),
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::BinaryOp { right, .. } => {
            match *right {
                HirExpression::BinaryOp { right: inner_right, .. } => {
                    assert!(
                        matches!(*inner_right, HirExpression::StringMethodCall { .. }),
                        "Deeply nested 'n' should be replaced with arr.len()"
                    );
                }
                other => panic!("Expected inner BinaryOp, got {:?}", other),
            }
        }
        other => panic!("Expected BinaryOp, got {:?}", other),
    }
}

#[test]
fn test_function_call_with_nested_expr_arg() {
    // process(arr[n]) where n is a length param
    let g = gen();
    let mut length_params = HashMap::new();
    length_params.insert("n".to_string(), "arr".to_string());

    let expr = HirExpression::FunctionCall {
        function: "process".to_string(),
        arguments: vec![HirExpression::ArrayIndex {
            array: Box::new(HirExpression::Variable("data".to_string())),
            index: Box::new(HirExpression::Variable("n".to_string())),
        }],
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &length_params);

    match result {
        HirExpression::FunctionCall { arguments, .. } => {
            match &arguments[0] {
                HirExpression::ArrayIndex { index, .. } => {
                    assert!(matches!(**index, HirExpression::StringMethodCall { .. }));
                }
                other => panic!("Expected ArrayIndex arg, got {:?}", other),
            }
        }
        other => panic!("Expected FunctionCall, got {:?}", other),
    }
}

// ============================================================================
// No transformation when maps are empty
// ============================================================================

#[test]
fn test_binary_op_no_transformation_when_empty() {
    let g = gen();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &empty_length_params());

    assert_eq!(
        result,
        HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }
    );
}

#[test]
fn test_function_call_no_transformation_when_empty() {
    let g = gen();
    let expr = HirExpression::FunctionCall {
        function: "foo".to_string(),
        arguments: vec![
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(2),
        ],
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &empty_length_params());

    assert_eq!(
        result,
        HirExpression::FunctionCall {
            function: "foo".to_string(),
            arguments: vec![
                HirExpression::IntLiteral(1),
                HirExpression::IntLiteral(2),
            ],
        }
    );
}

#[test]
fn test_cast_no_transformation_when_empty() {
    let g = gen();
    let expr = HirExpression::Cast {
        expr: Box::new(HirExpression::IntLiteral(42)),
        target_type: HirType::Double,
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &empty_length_params());

    assert_eq!(
        result,
        HirExpression::Cast {
            expr: Box::new(HirExpression::IntLiteral(42)),
            target_type: HirType::Double,
        }
    );
}

#[test]
fn test_ternary_no_transformation_when_empty() {
    let g = gen();
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::IntLiteral(1)),
        then_expr: Box::new(HirExpression::IntLiteral(2)),
        else_expr: Box::new(HirExpression::IntLiteral(3)),
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &empty_length_params());

    assert_eq!(
        result,
        HirExpression::Ternary {
            condition: Box::new(HirExpression::IntLiteral(1)),
            then_expr: Box::new(HirExpression::IntLiteral(2)),
            else_expr: Box::new(HirExpression::IntLiteral(3)),
        }
    );
}

// ============================================================================
// Edge case: deref of binary op where left is not Variable
// ============================================================================

#[test]
fn test_deref_add_left_not_variable_no_slice_transform() {
    // *(func_call() + i) -- left is not a Variable, so no SliceIndex
    let g = gen();
    let expr = HirExpression::Dereference(Box::new(HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::FunctionCall {
            function: "get_ptr".to_string(),
            arguments: vec![],
        }),
        right: Box::new(HirExpression::IntLiteral(0)),
    }));

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &empty_length_params());

    // Should be a Dereference (not SliceIndex) because left is not Variable
    assert!(matches!(result, HirExpression::Dereference(_)));
}

// ============================================================================
// Edge case: compound literal with empty initializers
// ============================================================================

#[test]
fn test_compound_literal_empty_initializers() {
    let g = gen();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Point".to_string()),
        initializers: vec![],
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &empty_length_params());

    match result {
        HirExpression::CompoundLiteral {
            literal_type,
            initializers,
        } => {
            assert_eq!(literal_type, HirType::Struct("Point".to_string()));
            assert!(initializers.is_empty());
        }
        other => panic!("Expected CompoundLiteral, got {:?}", other),
    }
}

// ============================================================================
// Edge case: function call with empty arguments
// ============================================================================

#[test]
fn test_function_call_no_args() {
    let g = gen();
    let expr = HirExpression::FunctionCall {
        function: "getchar".to_string(),
        arguments: vec![],
    };

    let result =
        g.transform_expression_recursive_with_length(&expr, &empty_inferences(), &empty_length_params());

    match result {
        HirExpression::FunctionCall {
            function,
            arguments,
        } => {
            assert_eq!(function, "getchar");
            assert!(arguments.is_empty());
        }
        other => panic!("Expected FunctionCall, got {:?}", other),
    }
}
