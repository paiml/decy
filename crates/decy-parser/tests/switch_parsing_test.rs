//! Parser tests for switch/case statements (DECY-038 RED phase).
//!
//! This test suite follows EXTREME TDD methodology - all tests should FAIL initially.
//! Tests verify that the parser correctly extracts switch/case statements from C AST.
//!
//! References:
//! - K&R §3.4: Switch
//! - ISO C99 §6.8.4.2: The switch statement

use decy_parser::parser::Statement;
use decy_parser::CParser;

#[test]
fn test_parse_simple_switch() {
    // Test basic switch with one case and default
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int test_switch(int x) {
            switch (x) {
                case 1:
                    return 10;
                default:
                    return 0;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.functions().len(), 1, "Should parse one function");

    let func = &ast.functions()[0];
    assert_eq!(func.name, "test_switch");
    assert_eq!(
        func.body.len(),
        1,
        "Function should have one statement (switch)"
    );

    // Verify switch statement
    match &func.body[0] {
        Statement::Switch {
            condition,
            cases,
            default_case,
        } => {
            // Verify condition is a variable reference
            assert!(matches!(
                condition,
                decy_parser::parser::Expression::Variable(_)
            ));

            // Verify one case
            assert_eq!(cases.len(), 1, "Should have one case");

            // Verify default case exists
            assert!(default_case.is_some(), "Should have default case");
        }
        _ => panic!("Expected Switch statement, got {:?}", func.body[0]),
    }
}

#[test]
fn test_parse_switch_with_multiple_cases() {
    // Test switch with multiple cases
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int classify(int status) {
            switch (status) {
                case 0:
                    return 1;
                case 1:
                    return 2;
                case 2:
                    return 3;
                default:
                    return 0;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];
    assert_eq!(func.body.len(), 1);

    match &func.body[0] {
        Statement::Switch {
            cases,
            default_case,
            ..
        } => {
            assert_eq!(cases.len(), 3, "Should have three cases");
            assert!(default_case.is_some(), "Should have default case");
        }
        _ => panic!("Expected Switch statement"),
    }
}

#[test]
fn test_parse_switch_with_default_only() {
    // Test switch with only default case (unusual but valid)
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int always_zero(int x) {
            switch (x) {
                default:
                    return 0;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];

    match &func.body[0] {
        Statement::Switch {
            cases,
            default_case,
            ..
        } => {
            assert_eq!(cases.len(), 0, "Should have no cases");
            assert!(default_case.is_some(), "Should have default case");
        }
        _ => panic!("Expected Switch statement"),
    }
}

#[test]
fn test_parse_switch_with_fallthrough() {
    // Test switch with fallthrough (no break between cases)
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int categorize(int value) {
            int result = 0;
            switch (value) {
                case 1:
                case 2:
                case 3:
                    result = 1;
                    break;
                case 4:
                    result = 2;
                    break;
                default:
                    result = 0;
            }
            return result;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];

    // Function should have multiple statements (variable decl, switch, return)
    assert!(func.body.len() >= 2, "Should have multiple statements");

    // Find the switch statement
    let switch_stmt = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::Switch { .. }));

    assert!(switch_stmt.is_some(), "Should find switch statement");

    match switch_stmt.unwrap() {
        Statement::Switch { cases, .. } => {
            // Should have multiple cases (some empty for fallthrough)
            assert!(cases.len() >= 2, "Should have multiple cases");
        }
        _ => panic!("Expected Switch statement"),
    }
}

#[test]
fn test_parse_switch_with_complex_condition() {
    // Test switch with expression as condition
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int test_expr(int a, int b) {
            switch (a + b) {
                case 0:
                    return 1;
                case 10:
                    return 2;
                default:
                    return 0;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];

    match &func.body[0] {
        Statement::Switch {
            condition, cases, ..
        } => {
            // Condition should be a binary operation (a + b)
            assert!(matches!(
                condition,
                decy_parser::parser::Expression::BinaryOp { .. }
            ));

            assert_eq!(cases.len(), 2, "Should have two cases");
        }
        _ => panic!("Expected Switch statement"),
    }
}

#[test]
fn test_parse_nested_switch() {
    // Test nested switch statements
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int nested(int x, int y) {
            switch (x) {
                case 1:
                    switch (y) {
                        case 10:
                            return 100;
                        default:
                            return 10;
                    }
                case 2:
                    return 2;
                default:
                    return 0;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];

    match &func.body[0] {
        Statement::Switch { cases, .. } => {
            assert!(cases.len() >= 2, "Outer switch should have multiple cases");

            // First case should contain a nested switch
            // (Implementation detail: check case body for nested switch)
            assert!(!cases[0].body.is_empty(), "First case should have body");
        }
        _ => panic!("Expected Switch statement"),
    }
}

#[test]
fn test_parse_switch_with_multiple_statements_per_case() {
    // Test case with multiple statements before break
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int multi_stmt(int x) {
            int result = 0;
            switch (x) {
                case 1:
                    result = 1;
                    result = result + 10;
                    break;
                case 2:
                    result = 2;
                    break;
                default:
                    result = 0;
            }
            return result;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];

    // Find switch statement
    let switch_stmt = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::Switch { .. }));

    match switch_stmt.unwrap() {
        Statement::Switch { cases, .. } => {
            assert!(cases.len() >= 2, "Should have multiple cases");

            // First case should have multiple statements
            assert!(
                cases[0].body.len() >= 2,
                "First case should have multiple statements"
            );
        }
        _ => panic!("Expected Switch statement"),
    }
}

#[test]
fn test_parse_switch_without_default() {
    // Test switch without default case (valid in C)
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void process(int code) {
            switch (code) {
                case 1:
                    return;
                case 2:
                    return;
            }
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];

    match &func.body[0] {
        Statement::Switch {
            cases,
            default_case,
            ..
        } => {
            assert_eq!(cases.len(), 2, "Should have two cases");
            assert!(default_case.is_none(), "Should not have default case");
        }
        _ => panic!("Expected Switch statement"),
    }
}
