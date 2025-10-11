//! Tests for array indexing support in HIR (DECY-021 RED phase).

use super::*;

#[test]
fn test_create_array_index_expression() {
    // arr[i] should be represented as ArrayIndex { array, index }
    let array_expr = HirExpression::Variable("arr".to_string());
    let index_expr = HirExpression::Variable("i".to_string());

    let array_index = HirExpression::ArrayIndex {
        array: Box::new(array_expr),
        index: Box::new(index_expr),
    };

    match array_index {
        HirExpression::ArrayIndex { array, index } => match (*array, *index) {
            (HirExpression::Variable(arr_name), HirExpression::Variable(idx_name)) => {
                assert_eq!(arr_name, "arr");
                assert_eq!(idx_name, "i");
            }
            _ => panic!("Expected Variable expressions"),
        },
        _ => panic!("Expected ArrayIndex expression"),
    }
}

#[test]
fn test_array_index_with_literal() {
    // arr[0] - array access with integer literal
    let array_expr = HirExpression::Variable("arr".to_string());
    let index_expr = HirExpression::IntLiteral(0);

    let array_index = HirExpression::ArrayIndex {
        array: Box::new(array_expr),
        index: Box::new(index_expr),
    };

    match array_index {
        HirExpression::ArrayIndex { array, index } => {
            assert!(matches!(*array, HirExpression::Variable(_)));
            assert_eq!(*index, HirExpression::IntLiteral(0));
        }
        _ => panic!("Expected ArrayIndex expression"),
    }
}

#[test]
fn test_nested_array_index() {
    // matrix[i][j] - nested array indexing
    let inner_array = HirExpression::Variable("matrix".to_string());
    let i_index = HirExpression::Variable("i".to_string());

    let first_index = HirExpression::ArrayIndex {
        array: Box::new(inner_array),
        index: Box::new(i_index),
    };

    let j_index = HirExpression::Variable("j".to_string());
    let nested_index = HirExpression::ArrayIndex {
        array: Box::new(first_index),
        index: Box::new(j_index),
    };

    // Verify nested structure
    match nested_index {
        HirExpression::ArrayIndex { array, .. } => {
            assert!(matches!(*array, HirExpression::ArrayIndex { .. }));
        }
        _ => panic!("Expected nested ArrayIndex"),
    }
}

#[test]
fn test_array_index_with_expression() {
    // arr[i + 1] - array access with complex expression
    let array_expr = HirExpression::Variable("arr".to_string());

    let index_expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("i".to_string())),
        right: Box::new(HirExpression::IntLiteral(1)),
    };

    let array_index = HirExpression::ArrayIndex {
        array: Box::new(array_expr),
        index: Box::new(index_expr),
    };

    match array_index {
        HirExpression::ArrayIndex { index, .. } => {
            assert!(matches!(*index, HirExpression::BinaryOp { .. }));
        }
        _ => panic!("Expected ArrayIndex with BinaryOp"),
    }
}

#[test]
fn test_array_index_in_assignment() {
    // arr[i] = 5; - array indexing on left side of assignment
    let target_expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
    };

    // Note: This will require extending HirStatement::Assignment to support
    // expressions as targets, not just variable names
    // For now, just verify the expression can be created
    assert!(matches!(target_expr, HirExpression::ArrayIndex { .. }));
}

#[test]
fn test_pointer_array_index() {
    // ptr[i] where ptr is a pointer - should still use ArrayIndex
    let array_expr = HirExpression::Variable("ptr".to_string());
    let index_expr = HirExpression::Variable("i".to_string());

    let array_index = HirExpression::ArrayIndex {
        array: Box::new(array_expr),
        index: Box::new(index_expr),
    };

    assert!(matches!(array_index, HirExpression::ArrayIndex { .. }));
}

#[test]
fn test_array_type_declaration() {
    // int arr[10]; - fixed-size array type
    let array_type = HirType::Array {
        element_type: Box::new(HirType::Int),
        size: Some(10),
    };

    match array_type {
        HirType::Array { element_type, size } => {
            assert_eq!(*element_type, HirType::Int);
            assert_eq!(size, Some(10));
        }
        _ => panic!("Expected Array type"),
    }
}

#[test]
fn test_unsized_array_type() {
    // int arr[]; - unsized array (common in function parameters)
    let array_type = HirType::Array {
        element_type: Box::new(HirType::Int),
        size: None,
    };

    match array_type {
        HirType::Array { element_type, size } => {
            assert_eq!(*element_type, HirType::Int);
            assert_eq!(size, None);
        }
        _ => panic!("Expected Array type"),
    }
}
