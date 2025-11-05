//! Tests for SliceIndex HIR expression.
//!
//! DECY-069: SliceIndex is a safe alternative to pointer arithmetic for array access.
//! Instead of generating unsafe pointer arithmetic (ptr + offset), we generate
//! safe slice indexing (arr[index]).

use crate::{HirExpression, HirType};

// ============================================================================
// DECY-069 RED PHASE: SliceIndex Expression Tests
// ============================================================================

#[test]
#[ignore = "DECY-069 RED: SliceIndex variant not yet implemented"]
fn test_slice_index_creation() {
    // Create SliceIndex expression: arr[5]
    let slice_index = HirExpression::SliceIndex {
        slice: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(5)),
        element_type: HirType::Int,
    };

    // Should be able to create and match
    if let HirExpression::SliceIndex {
        slice,
        index,
        element_type,
    } = slice_index
    {
        assert!(matches!(*slice, HirExpression::Variable(_)));
        assert!(matches!(*index, HirExpression::IntLiteral(5)));
        assert_eq!(element_type, HirType::Int);
    } else {
        panic!("Expected SliceIndex variant");
    }
}

#[test]
#[ignore = "DECY-069 RED: SliceIndex variant not yet implemented"]
fn test_slice_index_debug_format() {
    // SliceIndex should have Debug implementation
    let slice_index = HirExpression::SliceIndex {
        slice: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
        element_type: HirType::Int,
    };

    let debug_str = format!("{:?}", slice_index);

    // Debug output should contain key information
    assert!(
        debug_str.contains("SliceIndex"),
        "Debug output should mention SliceIndex"
    );
    assert!(
        debug_str.contains("arr") || debug_str.contains("Variable"),
        "Debug output should show slice variable"
    );
}

#[test]
#[ignore = "DECY-069 RED: SliceIndex variant not yet implemented"]
fn test_slice_index_distinguishes_from_array_index() {
    // SliceIndex and ArrayIndex are different variants
    let slice_index = HirExpression::SliceIndex {
        slice: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
        element_type: HirType::Int,
    };

    let array_index = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
    };

    // They should be different types
    assert!(
        !matches!(slice_index, HirExpression::ArrayIndex { .. }),
        "SliceIndex should not be ArrayIndex"
    );
    assert!(
        !matches!(array_index, HirExpression::SliceIndex { .. }),
        "ArrayIndex should not be SliceIndex"
    );
}

#[test]
#[ignore = "DECY-069 RED: SliceIndex variant not yet implemented"]
fn test_slice_index_with_expression_index() {
    // SliceIndex can use expression as index: arr[i + 1]
    let slice_index = HirExpression::SliceIndex {
        slice: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::BinaryOp {
            op: crate::BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        }),
        element_type: HirType::Int,
    };

    // Should match and extract components
    if let HirExpression::SliceIndex { index, .. } = slice_index {
        assert!(
            matches!(*index, HirExpression::BinaryOp { .. }),
            "Index should be BinaryOp"
        );
    } else {
        panic!("Expected SliceIndex");
    }
}

#[test]
#[ignore = "DECY-069 RED: SliceIndex variant not yet implemented"]
fn test_slice_index_element_type_preserved() {
    // SliceIndex preserves element type for codegen
    let int_slice = HirExpression::SliceIndex {
        slice: Box::new(HirExpression::Variable("int_arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
        element_type: HirType::Int,
    };

    let float_slice = HirExpression::SliceIndex {
        slice: Box::new(HirExpression::Variable("float_arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
        element_type: HirType::Float,
    };

    // Element types should be preserved
    if let HirExpression::SliceIndex { element_type, .. } = int_slice {
        assert_eq!(element_type, HirType::Int);
    }

    if let HirExpression::SliceIndex { element_type, .. } = float_slice {
        assert_eq!(element_type, HirType::Float);
    }
}
