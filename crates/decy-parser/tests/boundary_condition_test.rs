//! Boundary condition and counter tests (DECY-043 RED phase)
//!
//! These tests target 7 missed mutants related to boundary conditions and counters:
//! - 4 counter mutants (replace += with -= in visitor counters)
//! - 3 boundary mutants (replace > with >= in boundary checks)
//!
//! Goal: Ensure boundary conditions and counter arithmetic are thoroughly tested
//! and would fail if operators are mutated.
//!
//! References:
//! - Mutation testing report: 7 missed mutants (4 counter + 3 boundary)
//! - cargo mutants report from 2025-10-14

use decy_parser::parser::{Expression, Statement};
use decy_parser::CParser;

#[test]
fn test_loop_counter_increment() {
    // Test: for (i = 0; i < 10; i = i + 1)
    // Ensures counter increment is correctly handled
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test() {
            int i;
            for (i = 0; i < 10; i = i + 1) {
                int x;
                x = i;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    let for_stmt = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::For { .. }))
        .expect("Should have for statement");

    if let Statement::For { increment, .. } = for_stmt {
        assert!(increment.is_some(), "For loop should have increment");

        // Increment should be an assignment statement: i = i + 1
        if let Some(stmt) = increment {
            assert!(
                matches!(**stmt, Statement::Assignment { .. }),
                "Expected increment to be assignment statement"
            );
        }
    }
}

#[test]
fn test_loop_boundary_less_than() {
    // Test: i < 10
    // Ensures < boundary check is not mutated to <=
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test() {
            int i;
            for (i = 0; i < 10; i = i + 1) {
                int x;
                x = i;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    let for_stmt = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::For { .. }))
        .expect("Should have for statement");

    if let Statement::For {
        condition: Some(cond),
        ..
    } = for_stmt
    {
        // Condition should be i < 10
        match cond {
            Expression::BinaryOp { op, .. } => {
                assert!(
                    matches!(op, decy_parser::parser::BinaryOperator::LessThan),
                    "Should use < operator, not <="
                );
            }
            _ => panic!("Expected binary operation for condition"),
        }
    }
}

#[test]
fn test_loop_boundary_less_than_or_equal() {
    // Test: i <= n
    // Ensures <= boundary check is not mutated to <
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int n) {
            int i;
            for (i = 1; i <= n; i = i + 1) {
                int x;
                x = i;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    let for_stmt = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::For { .. }))
        .expect("Should have for statement");

    if let Statement::For {
        condition: Some(cond),
        ..
    } = for_stmt
    {
        match cond {
            Expression::BinaryOp { op, .. } => {
                assert!(
                    matches!(op, decy_parser::parser::BinaryOperator::LessEqual),
                    "Should use <= operator, not <"
                );
            }
            _ => panic!("Expected binary operation"),
        }
    }
}

#[test]
fn test_while_loop_boundary_greater_than() {
    // Test: while (count > 0)
    // Ensures > boundary is not mutated to >=
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int count) {
            while (count > 0) {
                count = count - 1;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    if let Statement::While { condition, .. } = &func.body[0] {
        match condition {
            Expression::BinaryOp { op, .. } => {
                assert!(
                    matches!(op, decy_parser::parser::BinaryOperator::GreaterThan),
                    "Should use > operator, not >="
                );
            }
            _ => panic!("Expected binary operation"),
        }
    }
}

#[test]
fn test_array_index_boundary_zero() {
    // Test: i = 0; (starting boundary)
    // Ensures array indexing starts at correct boundary
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int* arr, int size) {
            int i;
            for (i = 0; i < size; i = i + 1) {
                arr[i] = 0;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    let for_stmt = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::For { .. }))
        .expect("Should have for statement");

    if let Statement::For { init, .. } = for_stmt {
        assert!(init.is_some(), "For loop should have init");

        // Init should be assignment: i = 0 (starting at boundary 0)
        if let Some(stmt) = init {
            if let Statement::Assignment { value, .. } = &**stmt {
                assert!(
                    matches!(value, Expression::IntLiteral(0)),
                    "Array indexing should start at 0"
                );
            }
        }
    }
}

#[test]
fn test_off_by_one_boundary() {
    // Test: for (i = 0; i < size; i++)
    // vs for (i = 0; i <= size; i++)
    // Ensures correct boundary handling to avoid off-by-one errors
    let parser = CParser::new().expect("Parser creation failed");

    // Correct version: i < size
    let source_correct = r#"
        void test(int* arr, int size) {
            int i;
            for (i = 0; i < size; i = i + 1) {
                arr[i] = 0;
            }
        }
    "#;

    let ast = parser.parse(source_correct).expect("Should parse");
    let func = &ast.functions()[0];

    let for_stmt = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::For { .. }))
        .expect("Should have for statement");

    if let Statement::For {
        condition: Some(cond),
        ..
    } = for_stmt
    {
        match cond {
            Expression::BinaryOp { op, .. } => {
                // Should be <, not <=
                assert!(
                    matches!(op, decy_parser::parser::BinaryOperator::LessThan),
                    "Array loop should use < to avoid off-by-one"
                );
            }
            _ => panic!("Expected comparison"),
        }
    }
}

#[test]
fn test_counter_decrement_boundary() {
    // Test: count = count - 1
    // Ensures counter decrement is correctly parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int count) {
            while (count > 0) {
                count = count - 1;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    if let Statement::While { body, .. } = &func.body[0] {
        // Body should have count = count - 1
        let assignment = body
            .iter()
            .find(|stmt| matches!(stmt, Statement::Assignment { .. }))
            .expect("Should have assignment");

        if let Statement::Assignment { value, .. } = assignment {
            match value {
                Expression::BinaryOp { op, .. } => {
                    assert!(
                        matches!(op, decy_parser::parser::BinaryOperator::Subtract),
                        "Should use subtraction for decrement"
                    );
                }
                _ => panic!("Expected binary operation"),
            }
        }
    }
}

#[test]
fn test_range_boundaries_inclusive_vs_exclusive() {
    // Test: 1 to n inclusive (i <= n) vs 0 to n exclusive (i < n)
    // Ensures correct distinction between inclusive and exclusive boundaries
    let parser = CParser::new().expect("Parser creation failed");

    // Inclusive: for (i = 1; i <= n; i++)
    let source_inclusive = r#"
        void test_inclusive(int n) {
            int i;
            for (i = 1; i <= n; i = i + 1) {
                int x;
                x = i;
            }
        }
    "#;

    let ast = parser.parse(source_inclusive).expect("Should parse");
    let func = &ast.functions()[0];

    let for_stmt = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::For { .. }))
        .expect("Should have for statement");

    if let Statement::For {
        condition: Some(cond),
        ..
    } = for_stmt
    {
        match cond {
            Expression::BinaryOp { op, .. } => {
                assert!(
                    matches!(op, decy_parser::parser::BinaryOperator::LessEqual),
                    "Inclusive range should use <="
                );
            }
            _ => panic!("Expected comparison"),
        }
    }
}

#[test]
fn test_nested_loop_multiple_counters() {
    // Test: Multiple loop counters with different boundaries
    // Ensures each counter is independently tracked
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int m, int n) {
            int i;
            int j;
            for (i = 0; i < m; i = i + 1) {
                for (j = 0; j < n; j = j + 1) {
                    int x;
                    x = i + j;
                }
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    // Outer for loop
    let outer_for = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::For { .. }))
        .expect("Should have outer for");

    if let Statement::For { body, .. } = outer_for {
        // Inner for loop
        let inner_for = body
            .iter()
            .find(|stmt| matches!(stmt, Statement::For { .. }))
            .expect("Should have inner for");

        // Both loops should exist with proper structure
        assert!(
            matches!(inner_for, Statement::For { .. }),
            "Nested loop structure should be preserved"
        );
    }
}

#[test]
fn test_boundary_condition_with_equality() {
    // Test: if (count == 0) vs if (count < 0)
    // Ensures boundary equality checks are distinct from comparisons
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int count) {
            if (count == 0) {
                int x;
                x = 1;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    if let Statement::If { condition, .. } = &func.body[0] {
        match condition {
            Expression::BinaryOp { op, .. } => {
                assert!(
                    matches!(op, decy_parser::parser::BinaryOperator::Equal),
                    "Should use == for equality boundary check"
                );
            }
            _ => panic!("Expected binary operation"),
        }
    }
}
