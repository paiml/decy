//! Tests for recursive multiply bug (P0-MULTIPLY-001).
//!
//! Verifies that `n * func(n-1)` generates correct Rust: `n * func(n - 1)`
//! NOT the buggy: `n - func(n - 1)`

use decy_codegen::CodeGenerator;
use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType};

/// Helper: Create test function
fn create_function(
    name: &str,
    params: Vec<HirParameter>,
    return_type: HirType,
    body: Vec<HirStatement>,
) -> HirFunction {
    HirFunction::new_with_body(name.to_string(), return_type, params, body)
}

// ============================================================================
// TEST 1: n * func(n-1) generates correct multiplication
// ============================================================================

#[test]
fn test_recursive_multiply_correct() {
    // int factorial(int n) { return n * factorial(n - 1); }
    let func = create_function(
        "factorial",
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        HirType::Int,
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::FunctionCall {
                function: "factorial".to_string(),
                arguments: vec![HirExpression::BinaryOp {
                    op: BinaryOperator::Subtract,
                    left: Box::new(HirExpression::Variable("n".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                }],
            }),
        }))],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    // Must contain multiplication operator
    assert!(
        code.contains("n * factorial"),
        "Should have n * factorial(...):\n{}",
        code
    );
    // Must NOT have subtraction instead of multiplication
    assert!(
        !code.contains("n - factorial"),
        "Should NOT have n - factorial (bug):\n{}",
        code
    );
}

// ============================================================================
// TEST 2: a * b generates correct multiplication
// ============================================================================

#[test]
fn test_simple_multiply_correct() {
    // int mul(int a, int b) { return a * b; }
    let func = create_function(
        "mul",
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
        HirType::Int,
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }))],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(code.contains("a * b"), "Should generate a * b:\n{}", code);
}

// ============================================================================
// TEST 3: Nested multiply: a * b * c
// ============================================================================

#[test]
fn test_nested_multiply_correct() {
    // int triple(int a, int b, int c) { return a * b * c; }
    let func = create_function(
        "triple",
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
            HirParameter::new("c".to_string(), HirType::Int),
        ],
        HirType::Int,
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            }),
            right: Box::new(HirExpression::Variable("c".to_string())),
        }))],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        code.contains("*") && code.matches('*').count() >= 2,
        "Should have two multiply operators:\n{}",
        code
    );
}

// ============================================================================
// TEST 4: Multiply with function call result
// ============================================================================

#[test]
fn test_multiply_func_result() {
    // int compute(int x) { return x * get_value(); }
    let func = create_function(
        "compute",
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        HirType::Int,
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::FunctionCall {
                function: "get_value".to_string(),
                arguments: vec![],
            }),
        }))],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        code.contains("x * get_value"),
        "Should have x * get_value():\n{}",
        code
    );
}
