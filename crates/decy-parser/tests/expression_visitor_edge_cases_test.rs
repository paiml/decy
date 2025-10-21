//! Edge case tests for expression visitors (DECY-040 RED phase)
//!
//! These tests target missed mutants in expression visitor match arms.
//! Goal: Improve mutation score by catching match arm deletions.
//!
//! Mutation testing gaps addressed:
//! - Delete match arm CXCursor_CallExpr
//! - Delete match arm CXCursor_DeclRefExpr
//! - Delete match arm CXCursor_IntegerLiteral
//! - Delete match arm CXCursor_BinaryOperator
//!
//! References:
//! - Mutation testing report: 9 missed mutants in expression visitors
//! - cargo mutants report from 2025-10-14

use decy_parser::parser::{Expression, Statement};
use decy_parser::CParser;

#[test]
fn test_if_condition_with_function_call() {
    // Test: if (isValid()) { ... }
    // This ensures CXCursor_CallExpr match arm is tested in if condition visitor
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int isValid();

        void test() {
            if (isValid()) {
                int x;
                x = 1;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    // Find the test function (not the prototype)
    let func = ast
        .functions()
        .iter()
        .find(|f| f.name == "test")
        .expect("Should find test function");

    if let Statement::If {
        condition,
        then_block,
        ..
    } = &func.body[0]
    {
        // Condition should be a function call
        assert!(
            matches!(condition, Expression::FunctionCall { .. }),
            "If condition should be function call, got {:?}",
            condition
        );
        assert!(!then_block.is_empty(), "Then block should have statements");
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_if_condition_with_variable_reference() {
    // Test: if (flag) { ... }
    // This ensures CXCursor_DeclRefExpr match arm is tested
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int flag) {
            if (flag) {
                int x;
                x = 1;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::Variable(_)),
            "If condition should be variable reference, got {:?}",
            condition
        );
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_if_condition_with_integer_literal() {
    // Test: if (1) { ... }
    // This ensures CXCursor_IntegerLiteral match arm is tested
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test() {
            if (1) {
                int x;
                x = 1;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::IntLiteral(_)),
            "If condition should be integer literal, got {:?}",
            condition
        );
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_if_condition_with_binary_operation() {
    // Test: if (a > b) { ... }
    // This ensures CXCursor_BinaryOperator match arm is tested
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int a, int b) {
            if (a > b) {
                int x;
                x = 1;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::BinaryOp { .. }),
            "If condition should be binary operation, got {:?}",
            condition
        );
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_while_condition_with_function_call() {
    // Test: while (hasNext()) { ... }
    // Ensures function call match arm tested in while visitor
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int hasNext();

        void test() {
            while (hasNext()) {
                int x;
                x = 1;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    // Find the test function (not the prototype)
    let func = ast
        .functions()
        .iter()
        .find(|f| f.name == "test")
        .expect("Should find test function");

    if let Statement::While { condition, body } = &func.body[0] {
        assert!(
            matches!(condition, Expression::FunctionCall { .. }),
            "While condition should be function call, got {:?}",
            condition
        );
        assert!(!body.is_empty(), "While body should have statements");
    } else {
        panic!("Expected While statement");
    }
}

#[test]
fn test_while_condition_with_comparison() {
    // Test: while (i < n) { ... }
    // Ensures binary operator match arm tested in while visitor
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int n) {
            int i;
            i = 0;
            while (i < n) {
                i = i + 1;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    // While statement should be third (after declarations and initialization)
    let while_stmt = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::While { .. }));

    assert!(while_stmt.is_some(), "Should have while statement");

    if let Some(Statement::While { condition, .. }) = while_stmt {
        assert!(
            matches!(condition, Expression::BinaryOp { .. }),
            "While condition should be comparison, got {:?}",
            condition
        );
    }
}

#[test]
fn test_for_loop_with_variable_in_condition() {
    // Test: for (int i = 0; i < n; i++) { ... }
    // Ensures variable reference in for condition is captured
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int n) {
            int i;
            for (i = 0; i < n; i = i + 1) {
                int x;
                x = i;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    // Find the for statement
    let for_stmt = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::For { .. }));

    assert!(for_stmt.is_some(), "Should have for statement");

    if let Some(Statement::For {
        condition: Some(cond),
        ..
    }) = for_stmt
    {
        // Condition should involve variable references
        match cond {
            Expression::BinaryOp { left, right, .. } => {
                // At least one operand should be a variable
                let has_variable = matches!(**left, Expression::Variable(_))
                    || matches!(**right, Expression::Variable(_));
                assert!(has_variable, "For condition should reference variables");
            }
            _ => panic!("Expected binary operation in for condition"),
        }
    }
}

#[test]
fn test_for_loop_increment_with_assignment() {
    // Test: for (i = 0; i < 10; i = i + 1) { ... }
    // Ensures increment expression is captured
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test() {
            int i;
            for (i = 0; i < 10; i = i + 1) {
                int x;
                x = 1;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    let for_stmt = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::For { .. }));

    assert!(for_stmt.is_some(), "Should have for statement");

    if let Some(Statement::For { increment, .. }) = for_stmt {
        assert!(
            increment.is_some(),
            "For loop should have increment expression"
        );
    }
}

#[test]
fn test_nested_if_with_different_expression_types() {
    // Test: if (a) { if (b()) { if (c > d) { ... } } }
    // Ensures all match arms tested in nested contexts
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int b();

        void test(int a, int c, int d) {
            if (a) {
                if (b()) {
                    if (c > d) {
                        int x;
                        x = 1;
                    }
                }
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    // Find the test function (not the prototype)
    let func = ast
        .functions()
        .iter()
        .find(|f| f.name == "test")
        .expect("Should find test function");

    // Outer if: variable
    if let Statement::If {
        condition: outer_cond,
        then_block: outer_then,
        ..
    } = &func.body[0]
    {
        assert!(
            matches!(outer_cond, Expression::Variable(_)),
            "Outer if should have variable condition"
        );

        // Middle if: function call
        if let Statement::If {
            condition: mid_cond,
            then_block: mid_then,
            ..
        } = &outer_then[0]
        {
            assert!(
                matches!(mid_cond, Expression::FunctionCall { .. }),
                "Middle if should have function call condition"
            );

            // Inner if: binary operation
            if let Statement::If {
                condition: inner_cond,
                ..
            } = &mid_then[0]
            {
                assert!(
                    matches!(inner_cond, Expression::BinaryOp { .. }),
                    "Inner if should have binary operation condition"
                );
            } else {
                panic!("Expected inner if statement");
            }
        } else {
            panic!("Expected middle if statement");
        }
    } else {
        panic!("Expected outer if statement");
    }
}

#[test]
fn test_expression_with_dereference_in_condition() {
    // Test: if (*ptr) { ... }
    // Ensures dereference operator tested in if condition
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int* ptr) {
            if (*ptr) {
                int x;
                x = 1;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::Dereference(_)),
            "If condition should be dereference, got {:?}",
            condition
        );
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn test_expression_with_array_index_in_condition() {
    // Test: if (arr[i]) { ... }
    // Ensures array indexing tested in if condition
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int* arr, int i) {
            if (arr[i]) {
                int x;
                x = 1;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    if let Statement::If { condition, .. } = &func.body[0] {
        assert!(
            matches!(condition, Expression::ArrayIndex { .. }),
            "If condition should be array index, got {:?}",
            condition
        );
    } else {
        panic!("Expected If statement");
    }
}
