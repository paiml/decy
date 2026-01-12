//! Tests for strlen → .len() transformation (STDLIB-STRLEN validation)
//!
//! Reference: K&R §B3, ISO C99 §7.21.6.3
//!
//! This module tests the transformation of C strlen() to Rust's .len() method.
//! strlen(s) returns the length of a null-terminated string, which maps directly
//! to .len() on Rust strings.

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};

/// Test simple strlen call
///
/// C: size_t len = strlen(s);
///
/// Rust: let len: usize = s.len();
///
/// Reference: K&R §B3, ISO C99 §7.21.6.3
#[test]
fn test_simple_strlen() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "s".to_string(),
            HirType::Pointer(Box::new(HirType::Char)),
        )],
        vec![HirStatement::VariableDeclaration {
            name: "len".to_string(),
            var_type: HirType::Int, // size_t maps to usize in Rust
            initializer: Some(HirExpression::FunctionCall {
                function: "strlen".to_string(),
                arguments: vec![HirExpression::Variable("s".to_string())],
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify strlen is transformed to .len()
    assert!(
        result.contains("s.len()"),
        "Should transform strlen(s) to s.len()"
    );

    // Should NOT contain C strlen function
    assert!(
        !result.contains("strlen("),
        "Should not contain C strlen function"
    );

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test strlen in expression
///
/// C: if (strlen(s) > 10) { ... }
///
/// Rust: if s.len() > 10 { ... }
///
/// Reference: K&R §B3, ISO C99 §7.21.6.3
#[test]
fn test_strlen_in_expression() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "s".to_string(),
            HirType::Pointer(Box::new(HirType::Char)),
        )],
        vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::GreaterThan,
                left: Box::new(HirExpression::FunctionCall {
                    function: "strlen".to_string(),
                    arguments: vec![HirExpression::Variable("s".to_string())],
                }),
                right: Box::new(HirExpression::IntLiteral(10)),
            },
            then_block: vec![],
            else_block: None,
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify strlen is transformed in condition
    assert!(result.contains("s.len()"));
    assert!(result.contains("> 10"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test strlen in return statement
///
/// C: int get_length(char* s) {
///      return strlen(s);
///    }
///
/// Rust: fn get_length(s: &str) -> i32 {
///         return s.len() as i32;
///       }
///
/// Reference: K&R §B3, ISO C99 §7.21.6.3
#[test]
fn test_strlen_in_return() {
    let func = HirFunction::new_with_body(
        "get_length".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "s".to_string(),
            HirType::Pointer(Box::new(HirType::Char)),
        )],
        vec![HirStatement::Return(Some(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("s".to_string())],
        }))],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify strlen is transformed
    assert!(result.contains("s.len()"));
    assert!(result.contains("return"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test strlen with string literal
///
/// C: size_t len = strlen("hello");
///
/// Rust: let len = "hello".len();
///
/// Reference: K&R §B3, ISO C99 §7.21.6.3
#[test]
fn test_strlen_with_string_literal() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "len".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::FunctionCall {
                function: "strlen".to_string(),
                arguments: vec![HirExpression::StringLiteral("hello".to_string())],
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify strlen is transformed on string literal
    assert!(result.contains("\"hello\".len()") || result.contains(".len()"));
    assert!(!result.contains("strlen"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test multiple strlen calls
///
/// C: int total = strlen(s1) + strlen(s2);
///
/// Rust: let total = s1.len() + s2.len();
///
/// Reference: K&R §B3, ISO C99 §7.21.6.3
#[test]
fn test_multiple_strlen_calls() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("s1".to_string(), HirType::Pointer(Box::new(HirType::Char))),
            HirParameter::new("s2".to_string(), HirType::Pointer(Box::new(HirType::Char))),
        ],
        vec![HirStatement::VariableDeclaration {
            name: "total".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Add,
                left: Box::new(HirExpression::FunctionCall {
                    function: "strlen".to_string(),
                    arguments: vec![HirExpression::Variable("s1".to_string())],
                }),
                right: Box::new(HirExpression::FunctionCall {
                    function: "strlen".to_string(),
                    arguments: vec![HirExpression::Variable("s2".to_string())],
                }),
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify both strlen calls are transformed
    let len_count = result.matches(".len()").count();
    assert!(len_count >= 2, "Should have at least 2 .len() calls");

    // Should NOT contain strlen
    assert!(!result.contains("strlen"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test strlen in loop condition
///
/// C: for (size_t i = 0; i < strlen(s); i++) { ... }
///
/// Rust: let len = s.len();
///       for i in 0..len { ... }
///
/// Reference: K&R §B3, ISO C99 §7.21.6.3
#[test]
fn test_strlen_in_loop() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "s".to_string(),
            HirType::Pointer(Box::new(HirType::Char)),
        )],
        vec![HirStatement::For {
            init: vec![HirStatement::VariableDeclaration {
                name: "i".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            }],
            condition: HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::FunctionCall {
                    function: "strlen".to_string(),
                    arguments: vec![HirExpression::Variable("s".to_string())],
                }),
            },
            increment: vec![HirStatement::Assignment {
                target: "i".to_string(),
                value: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("i".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            }],
            body: vec![],
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify strlen is transformed
    assert!(result.contains(".len()"));
    assert!(!result.contains("strlen"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test strlen comparison with zero
///
/// C: if (strlen(s) == 0) { ... }
///
/// Rust: if s.is_empty() { ... } or if s.len() == 0 { ... }
///
/// Reference: K&R §B3, ISO C99 §7.21.6.3
#[test]
fn test_strlen_zero_comparison() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "s".to_string(),
            HirType::Pointer(Box::new(HirType::Char)),
        )],
        vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Equal,
                left: Box::new(HirExpression::FunctionCall {
                    function: "strlen".to_string(),
                    arguments: vec![HirExpression::Variable("s".to_string())],
                }),
                right: Box::new(HirExpression::IntLiteral(0)),
            },
            then_block: vec![],
            else_block: None,
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Accept either .is_empty() or .len() == 0
    assert!(result.contains(".is_empty()") || result.contains(".len() == 0"));
    assert!(!result.contains("strlen"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Verify unsafe block count remains 0
///
/// This is critical for the validation goal: <5 unsafe blocks per 1000 LOC
#[test]
fn test_strlen_transformation_unsafe_count() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("s1".to_string(), HirType::Pointer(Box::new(HirType::Char))),
            HirParameter::new("s2".to_string(), HirType::Pointer(Box::new(HirType::Char))),
        ],
        vec![
            HirStatement::VariableDeclaration {
                name: "len1".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::FunctionCall {
                    function: "strlen".to_string(),
                    arguments: vec![HirExpression::Variable("s1".to_string())],
                }),
            },
            HirStatement::VariableDeclaration {
                name: "len2".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::FunctionCall {
                    function: "strlen".to_string(),
                    arguments: vec![HirExpression::Variable("s2".to_string())],
                }),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Count unsafe blocks (should be 0)
    let unsafe_count = result.matches("unsafe").count();
    assert_eq!(
        unsafe_count, 0,
        "strlen → .len() transformation should not introduce unsafe blocks"
    );
}
