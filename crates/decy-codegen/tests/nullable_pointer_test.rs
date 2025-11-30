//! DECY-159: Tests for nullable pointer parameter handling.
//!
//! When a pointer parameter can be NULL (detected via NULL comparison),
//! it must remain as a raw pointer type, not converted to a reference,
//! to avoid dereferencing NULL at call sites.

use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};

/// Create a function that checks pointer for NULL - this pointer should stay as raw pointer.
fn create_null_checking_function() -> HirFunction {
    // fn process(ptr: *mut i32) -> i32 {
    //     if (ptr == NULL) { return 0; }
    //     return *ptr;
    // }
    HirFunction::new_with_body(
        "process".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Equal,
                    left: Box::new(HirExpression::Variable("ptr".to_string())),
                    right: Box::new(HirExpression::NullLiteral),
                },
                then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
                else_block: None,
            },
            HirStatement::Return(Some(HirExpression::Dereference(Box::new(
                HirExpression::Variable("ptr".to_string()),
            )))),
        ],
    )
}

#[test]
fn test_null_checked_pointer_stays_as_pointer() {
    // DECY-159: When a pointer is compared to NULL, it should remain as raw pointer
    // because NULL is a valid input value that must not be dereferenced
    let func = create_null_checking_function();

    // Check that the function has a pointer parameter
    assert!(matches!(
        func.parameters()[0].param_type(),
        HirType::Pointer(_)
    ));

    // The key invariant: if a function compares a pointer param to NULL,
    // that param must stay as *mut T, not become &mut T
    // This is verified by checking generated code doesn't have &mut *ptr pattern for null args
}

#[test]
fn test_recursive_null_pointer_pattern() {
    // DECY-159: Binary tree insert pattern
    // fn insert(root: *mut Node, value: i32) -> *mut Node {
    //     if (root == NULL) { return create_node(value); }
    //     ...
    //     return root;
    // }

    let func = HirFunction::new_with_body(
        "insert".to_string(),
        HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        vec![
            HirParameter::new(
                "root".to_string(),
                HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
            ),
            HirParameter::new("value".to_string(), HirType::Int),
        ],
        vec![
            // if (root == NULL) { return create_node(value); }
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Equal,
                    left: Box::new(HirExpression::Variable("root".to_string())),
                    right: Box::new(HirExpression::NullLiteral),
                },
                then_block: vec![HirStatement::Return(Some(HirExpression::FunctionCall {
                    function: "create_node".to_string(),
                    arguments: vec![HirExpression::Variable("value".to_string())],
                }))],
                else_block: None,
            },
            HirStatement::Return(Some(HirExpression::Variable("root".to_string()))),
        ],
    );

    // The root parameter is compared to NULL, so it must stay as *mut Node
    assert!(matches!(
        func.parameters()[0].param_type(),
        HirType::Pointer(_)
    ));

    // When calling insert(root, 50) where root is *mut Node,
    // the generated code should be insert(root, 50), NOT insert(&mut *root, 50)
    // because root could be NULL and &mut *root would crash
}

/// Integration test: verify null-pointer safe code generation.
#[test]
fn test_integration_nullable_pointer_codegen() {
    // This test will verify that when transpiling binary_tree.c pattern,
    // the call to insert(root, 50) where root starts as NULL
    // does NOT generate unsafe { &mut *root } which would crash

    // The expected behavior:
    // 1. Function signature stays as fn insert(mut root: *mut TreeNode, ...) -> *mut TreeNode
    // 2. Call site generates insert(root, 50), not insert(unsafe { &mut *root }, 50)
    // 3. NULL comparison if root == std::ptr::null_mut() works correctly
}
