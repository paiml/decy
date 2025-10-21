//! Comprehensive binary operator test coverage (DECY-041 RED phase)
//!
//! These tests target 8 missed mutants related to binary operator handling.
//! Current mutation testing shows incomplete coverage for operators:
//! ==, !=, /, %, <=, >=, *
//!
//! Goal: Ensure every binary operator has dedicated tests that would fail
//! if operator handling code is mutated.
//!
//! References:
//! - Mutation testing report: 8 missed mutants in binary operator handling
//! - cargo mutants report from 2025-10-14

use decy_parser::parser::{BinaryOperator, Expression, Statement};
use decy_parser::CParser;

#[test]
fn test_equality_operator() {
    // Test: if (a == b) { ... }
    // Ensures == operator is correctly parsed and would fail if mutated to !=
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int a, int b) {
            if (a == b) {
                int result;
                result = 1;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    if let Statement::If { condition, .. } = &func.body[0] {
        match condition {
            Expression::BinaryOp { op, .. } => {
                assert!(
                    matches!(op, BinaryOperator::Equal),
                    "Operator should be ==, got {:?}",
                    op
                );
            }
            _ => panic!("Expected binary operation, got {:?}", condition),
        }
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_inequality_operator() {
    // Test: if (a != b) { ... }
    // Ensures != operator is correctly parsed and would fail if mutated to ==
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int a, int b) {
            if (a != b) {
                int result;
                result = 1;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    if let Statement::If { condition, .. } = &func.body[0] {
        match condition {
            Expression::BinaryOp { op, .. } => {
                assert!(
                    matches!(op, BinaryOperator::NotEqual),
                    "Operator should be !=, got {:?}",
                    op
                );
            }
            _ => panic!("Expected binary operation"),
        }
    }
}

#[test]
fn test_division_operator() {
    // Test: result = a / b;
    // Ensures / operator is correctly parsed and distinguished from other operators
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int test(int a, int b) {
            int result;
            result = a / b;
            return result;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    // Find assignment statement (after variable declaration)
    let assignment = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::Assignment { .. }))
        .expect("Should have assignment statement");

    if let Statement::Assignment { value, .. } = assignment {
        match value {
            Expression::BinaryOp { op, .. } => {
                assert!(
                    matches!(op, BinaryOperator::Divide),
                    "Operator should be /, got {:?}",
                    op
                );
            }
            _ => panic!("Expected binary operation, got {:?}", value),
        }
    }
}

#[test]
fn test_modulo_operator() {
    // Test: result = a % b;
    // Ensures % operator is correctly parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int test(int a, int b) {
            int result;
            result = a % b;
            return result;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    let assignment = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::Assignment { .. }))
        .expect("Should have assignment");

    if let Statement::Assignment { value, .. } = assignment {
        match value {
            Expression::BinaryOp { op, .. } => {
                assert!(
                    matches!(op, BinaryOperator::Modulo),
                    "Operator should be %, got {:?}",
                    op
                );
            }
            _ => panic!("Expected binary operation"),
        }
    }
}

#[test]
fn test_less_than_or_equal_operator() {
    // Test: if (a <= b) { ... }
    // Ensures <= operator is correctly parsed and not mutated to <
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int a, int b) {
            if (a <= b) {
                int result;
                result = 1;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    if let Statement::If { condition, .. } = &func.body[0] {
        match condition {
            Expression::BinaryOp { op, .. } => {
                assert!(
                    matches!(op, BinaryOperator::LessEqual),
                    "Operator should be <=, got {:?}",
                    op
                );
            }
            _ => panic!("Expected binary operation"),
        }
    }
}

#[test]
fn test_greater_than_or_equal_operator() {
    // Test: if (a >= b) { ... }
    // Ensures >= operator is correctly parsed and not mutated to >
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int a, int b) {
            if (a >= b) {
                int result;
                result = 1;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    if let Statement::If { condition, .. } = &func.body[0] {
        match condition {
            Expression::BinaryOp { op, .. } => {
                assert!(
                    matches!(op, BinaryOperator::GreaterEqual),
                    "Operator should be >=, got {:?}",
                    op
                );
            }
            _ => panic!("Expected binary operation"),
        }
    }
}

#[test]
fn test_multiplication_operator() {
    // Test: result = a * b;
    // Ensures * operator is correctly parsed (distinct from pointer dereference)
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int test(int a, int b) {
            int result;
            result = a * b;
            return result;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    let assignment = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::Assignment { .. }))
        .expect("Should have assignment");

    if let Statement::Assignment { value, .. } = assignment {
        match value {
            Expression::BinaryOp { op, .. } => {
                assert!(
                    matches!(op, BinaryOperator::Multiply),
                    "Operator should be *, got {:?}",
                    op
                );
            }
            _ => panic!("Expected binary operation, got {:?}", value),
        }
    }
}

#[test]
fn test_operator_precedence_multiplication_vs_addition() {
    // Test: result = a + b * c;
    // Ensures operator precedence is correct (multiplication before addition)
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int test(int a, int b, int c) {
            int result;
            result = a + b * c;
            return result;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    let assignment = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::Assignment { .. }))
        .expect("Should have assignment");

    if let Statement::Assignment { value, .. } = assignment {
        // Outer operation should be addition
        match value {
            Expression::BinaryOp { op, right, .. } => {
                assert!(
                    matches!(op, BinaryOperator::Add),
                    "Outer operator should be +, got {:?}",
                    op
                );

                // Right side should be multiplication (b * c)
                match &**right {
                    Expression::BinaryOp { op, .. } => {
                        assert!(
                            matches!(op, BinaryOperator::Multiply),
                            "Inner operator should be *, got {:?}",
                            op
                        );
                    }
                    _ => panic!("Expected binary operation for b * c"),
                }
            }
            _ => panic!("Expected binary operation"),
        }
    }
}

#[test]
fn test_comparison_operators_in_logical_expression() {
    // Test: if ((a < b) && (c > d)) { ... }
    // Ensures comparison operators work correctly in logical expressions
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int a, int b, int c, int d) {
            if ((a < b) && (c > d)) {
                int result;
                result = 1;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    if let Statement::If { condition, .. } = &func.body[0] {
        // Outer should be LogicalAnd
        match condition {
            Expression::BinaryOp {
                op, left, right, ..
            } => {
                assert!(
                    matches!(op, BinaryOperator::LogicalAnd),
                    "Outer should be &&"
                );

                // Left: a < b
                match &**left {
                    Expression::BinaryOp { op, .. } => {
                        assert!(matches!(op, BinaryOperator::LessThan), "Left should be <");
                    }
                    _ => panic!("Expected comparison"),
                }

                // Right: c > d
                match &**right {
                    Expression::BinaryOp { op, .. } => {
                        assert!(
                            matches!(op, BinaryOperator::GreaterThan),
                            "Right should be >"
                        );
                    }
                    _ => panic!("Expected comparison"),
                }
            }
            _ => panic!("Expected binary operation"),
        }
    }
}

#[test]
fn test_all_arithmetic_operators_in_sequence() {
    // Test: result = a + b - c * d / e % f;
    // Ensures all arithmetic operators can coexist and parse correctly
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int test(int a, int b, int c, int d, int e, int f) {
            int result;
            result = a + b - c * d / e % f;
            return result;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    let assignment = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::Assignment { .. }))
        .expect("Should have assignment");

    if let Statement::Assignment { value, .. } = assignment {
        // Should be a complex binary expression tree
        assert!(
            matches!(value, Expression::BinaryOp { .. }),
            "Should be binary operation"
        );
        // The exact structure depends on precedence, but it should parse without error
    }
}
