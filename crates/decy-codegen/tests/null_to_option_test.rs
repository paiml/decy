//! Tests for NULL → Option<T> transformation (TYPE-PTR-NULL validation)
//!
//! Reference: K&R §5.4, ISO C99 §7.17
//!
//! This module tests the transformation of C NULL pointer handling to Rust's Option<T>.
//! NULL in C is unsafe because it can cause null pointer dereferences.
//! Rust's Option<T> is safe because it forces explicit null checking at compile time.

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirStatement, HirType};

/// Test NULL pointer initialization → Option::None
///
/// C: int* p = NULL;
/// Rust: let p: Option<Box<i32>> = None;
///
/// Reference: K&R §5.4, ISO C99 §7.17
#[test]
fn test_null_pointer_initialization() {
    // RED PHASE: This test should FAIL until we implement NULL handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "p".to_string(),
            var_type: HirType::Option(Box::new(HirType::Box(Box::new(HirType::Int)))),
            initializer: Some(HirExpression::NullLiteral),
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected Rust output
    assert!(result.contains("let mut p: Option<Box<i32>> = None"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test NULL check with if statement → if let Some(p)
///
/// C: int* p = NULL; if (p) { ... }
/// Rust: let p: Option<Box<i32>> = None; if let Some(p) = p { ... }
///
/// Reference: K&R §5.4, ISO C99 §7.17
#[test]
fn test_null_check_if_statement() {
    // RED PHASE: This test should FAIL until we implement NULL checking
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "p".to_string(),
                var_type: HirType::Option(Box::new(HirType::Box(Box::new(HirType::Int)))),
                initializer: Some(HirExpression::NullLiteral),
            },
            HirStatement::If {
                condition: HirExpression::IsNotNull(Box::new(HirExpression::Variable(
                    "p".to_string(),
                ))),
                then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
                else_block: None,
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected Rust output with pattern matching
    assert!(result.contains("if let Some"));
    assert!(result.contains("= p"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test NULL comparison → is_none() / is_some()
///
/// C: if (p == NULL) { ... }
/// Rust: if p.is_none() { ... }
///
/// Reference: K&R §5.4, ISO C99 §7.17
#[test]
fn test_null_equality_comparison() {
    // RED PHASE: This test should FAIL until we implement NULL comparison
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "p".to_string(),
                var_type: HirType::Option(Box::new(HirType::Box(Box::new(HirType::Int)))),
                initializer: Some(HirExpression::NullLiteral),
            },
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Equal,
                    left: Box::new(HirExpression::Variable("p".to_string())),
                    right: Box::new(HirExpression::NullLiteral),
                },
                then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
                else_block: None,
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected Rust output
    assert!(result.contains("p.is_none()"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test NULL inequality → is_some()
///
/// C: if (p != NULL) { ... }
/// Rust: if p.is_some() { ... }
///
/// Reference: K&R §5.4, ISO C99 §7.17
#[test]
fn test_null_inequality_comparison() {
    // RED PHASE: This test should FAIL until we implement NULL comparison
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "p".to_string(),
                var_type: HirType::Option(Box::new(HirType::Box(Box::new(HirType::Int)))),
                initializer: Some(HirExpression::NullLiteral),
            },
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::NotEqual,
                    left: Box::new(HirExpression::Variable("p".to_string())),
                    right: Box::new(HirExpression::NullLiteral),
                },
                then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
                else_block: None,
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected Rust output
    assert!(result.contains("p.is_some()"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test NULL pointer function parameter → Option<Box<T>>
///
/// C: void func(int* p) { if (p) { ... } }
/// Rust: fn func(mut p: Option<Box<i32>>) { if let Some(p) = p { ... } }
///
/// Reference: K&R §5.4, ISO C99 §7.17
#[test]
fn test_null_pointer_parameter() {
    // RED PHASE: This test should FAIL until we implement NULL parameter handling
    let func = HirFunction::new_with_body(
        "func".to_string(),
        HirType::Void,
        vec![decy_hir::HirParameter::new(
            "p".to_string(),
            HirType::Option(Box::new(HirType::Box(Box::new(HirType::Int)))),
        )],
        vec![HirStatement::If {
            condition: HirExpression::IsNotNull(Box::new(HirExpression::Variable("p".to_string()))),
            then_block: vec![HirStatement::Return(None)],
            else_block: None,
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected Rust output
    assert!(result.contains("fn func(mut p: Option<Box<i32>>)"));
    assert!(result.contains("if let Some"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test multiple NULL pointers → multiple Option types
///
/// C: int* p1 = NULL; int* p2 = NULL;
/// Rust: let p1: Option<Box<i32>> = None; let p2: Option<Box<i32>> = None;
///
/// Reference: K&R §5.4, ISO C99 §7.17
#[test]
fn test_multiple_null_pointers() {
    // RED PHASE: This test should FAIL until we implement NULL handling
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "p1".to_string(),
                var_type: HirType::Option(Box::new(HirType::Box(Box::new(HirType::Int)))),
                initializer: Some(HirExpression::NullLiteral),
            },
            HirStatement::VariableDeclaration {
                name: "p2".to_string(),
                var_type: HirType::Option(Box::new(HirType::Box(Box::new(HirType::Int)))),
                initializer: Some(HirExpression::NullLiteral),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Expected Rust output
    assert!(result.contains("let mut p1: Option<Box<i32>> = None"));
    assert!(result.contains("let mut p2: Option<Box<i32>> = None"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Verify unsafe block count remains 0
///
/// This is critical for the validation goal: <5 unsafe blocks per 1000 LOC
#[test]
fn test_null_transformation_unsafe_count() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "p".to_string(),
            var_type: HirType::Option(Box::new(HirType::Box(Box::new(HirType::Int)))),
            initializer: Some(HirExpression::NullLiteral),
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Count unsafe blocks (should be 0)
    let unsafe_count = result.matches("unsafe").count();
    assert_eq!(
        unsafe_count, 0,
        "NULL → Option transformation should not introduce unsafe blocks"
    );
}
