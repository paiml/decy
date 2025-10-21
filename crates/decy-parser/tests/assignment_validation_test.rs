//! Assignment statement validation tests (DECY-042 RED phase)
//!
//! These tests target 7 missed mutants in assignment statement validation logic.
//! Current mutation testing shows gaps in extract_assignment_stmt validation:
//! - replace || with && in assignment checks
//! - replace == with != in operator validation
//!
//! Goal: Ensure assignment validation logic is thoroughly tested and would fail
//! if logical operators are mutated.
//!
//! References:
//! - Mutation testing report: 7 missed mutants in assignment validation
//! - cargo mutants report from 2025-10-14

use decy_parser::parser::{Expression, Statement};
use decy_parser::CParser;

#[test]
fn test_assignment_with_simple_value() {
    // Test: x = 42;
    // Ensures basic assignment is correctly parsed
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test() {
            int x;
            x = 42;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    // Find assignment statement
    let assignment = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::Assignment { .. }))
        .expect("Should have assignment statement");

    if let Statement::Assignment { target, value } = assignment {
        assert_eq!(target, "x", "Assignment target should be 'x'");
        assert!(
            matches!(value, Expression::IntLiteral(42)),
            "Assignment value should be integer literal 42"
        );
    }
}

#[test]
fn test_assignment_with_variable_value() {
    // Test: x = y;
    // Ensures variable assignment is validated correctly
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int y) {
            int x;
            x = y;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    let assignment = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::Assignment { .. }))
        .expect("Should have assignment");

    if let Statement::Assignment { target, value } = assignment {
        assert_eq!(target, "x");
        assert!(
            matches!(value, Expression::Variable(_)),
            "Value should be variable reference"
        );
    }
}

#[test]
fn test_assignment_with_expression() {
    // Test: x = a + b;
    // Ensures assignment with binary expression is validated
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int a, int b) {
            int x;
            x = a + b;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    let assignment = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::Assignment { .. }))
        .expect("Should have assignment");

    if let Statement::Assignment { target, value } = assignment {
        assert_eq!(target, "x");
        assert!(
            matches!(value, Expression::BinaryOp { .. }),
            "Value should be binary operation"
        );
    }
}

#[test]
fn test_multiple_assignments_in_sequence() {
    // Test: x = 1; y = 2; z = 3;
    // Ensures multiple assignments are all validated
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test() {
            int x;
            int y;
            int z;
            x = 1;
            y = 2;
            z = 3;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    let assignments: Vec<_> = func
        .body
        .iter()
        .filter(|stmt| matches!(stmt, Statement::Assignment { .. }))
        .collect();

    assert_eq!(
        assignments.len(),
        3,
        "Should have exactly 3 assignment statements"
    );

    // Verify each assignment
    let targets: Vec<String> = assignments
        .iter()
        .filter_map(|stmt| {
            if let Statement::Assignment { target, .. } = stmt {
                Some(target.clone())
            } else {
                None
            }
        })
        .collect();

    assert_eq!(targets, vec!["x", "y", "z"]);
}

#[test]
fn test_assignment_vs_declaration_with_initializer() {
    // Test: int x = 5; vs x = 5;
    // Ensures assignment validation correctly distinguishes from declaration
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test() {
            int x;
            int y = 10;
            x = 5;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    // Should have 2 declarations and 1 assignment
    let declarations: Vec<_> = func
        .body
        .iter()
        .filter(|stmt| matches!(stmt, Statement::VariableDeclaration { .. }))
        .collect();

    let assignments: Vec<_> = func
        .body
        .iter()
        .filter(|stmt| matches!(stmt, Statement::Assignment { .. }))
        .collect();

    assert_eq!(declarations.len(), 2, "Should have 2 variable declarations");
    assert_eq!(assignments.len(), 1, "Should have 1 assignment statement");

    // The assignment should be to 'x', not 'y'
    if let Statement::Assignment { target, .. } = assignments[0] {
        assert_eq!(target, "x", "Assignment should be to 'x'");
    }
}

#[test]
fn test_assignment_with_function_call() {
    // Test: x = malloc(size);
    // Ensures assignment validation handles function call RHS
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void* malloc(int);

        void test(int size) {
            void* x;
            x = malloc(size);
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    // Find the test function (not malloc prototype)
    let func = ast
        .functions()
        .iter()
        .find(|f| f.name == "test")
        .expect("Should find test function");

    let assignment = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::Assignment { .. }))
        .expect("Should have assignment");

    if let Statement::Assignment { target, value } = assignment {
        assert_eq!(target, "x");
        assert!(
            matches!(value, Expression::FunctionCall { .. }),
            "Value should be function call"
        );
    }
}

#[test]
fn test_assignment_with_dereference() {
    // Test: *ptr = value;
    // Ensures dereference assignments are validated separately from regular assignments
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int* ptr, int value) {
            *ptr = value;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    // This should be a DerefAssignment, not regular Assignment
    let deref_assignment = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::DerefAssignment { .. }))
        .expect("Should have dereference assignment");

    if let Statement::DerefAssignment { target, value } = deref_assignment {
        // Target is the inner expression being dereferenced (ptr, not *ptr)
        assert!(
            matches!(target, Expression::Variable(_)),
            "Target should be variable (the pointer being dereferenced)"
        );
        assert!(
            matches!(value, Expression::Variable(_)),
            "Value should be variable"
        );
    }
}

#[test]
fn test_assignment_with_array_index() {
    // Test: arr[i] = value;
    // Ensures array index assignments are validated separately
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int* arr, int i, int value) {
            arr[i] = value;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    // This should be an ArrayIndexAssignment
    let array_assignment = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::ArrayIndexAssignment { .. }))
        .expect("Should have array index assignment");

    if let Statement::ArrayIndexAssignment {
        array,
        index,
        value,
    } = array_assignment
    {
        assert!(
            matches!(**array, Expression::Variable(_)),
            "Array should be variable"
        );
        assert!(
            matches!(**index, Expression::Variable(_)),
            "Index should be variable"
        );
        assert!(
            matches!(value, Expression::Variable(_)),
            "Value should be variable"
        );
    }
}

#[test]
fn test_assignment_with_complex_lhs() {
    // Test: Verify validation logic correctly handles edge cases
    // This test ensures the validation checks would catch malformed assignments
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test() {
            int x;
            int y;
            x = 42;
            y = x;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    let assignments: Vec<_> = func
        .body
        .iter()
        .filter(|stmt| matches!(stmt, Statement::Assignment { .. }))
        .collect();

    // Both assignments should be present and valid
    assert_eq!(assignments.len(), 2);

    // First: x = 42
    if let Statement::Assignment { target, value } = assignments[0] {
        assert_eq!(target, "x");
        assert!(matches!(value, Expression::IntLiteral(_)));
    }

    // Second: y = x
    if let Statement::Assignment { target, value } = assignments[1] {
        assert_eq!(target, "y");
        assert!(matches!(value, Expression::Variable(_)));
    }
}

#[test]
fn test_assignment_validation_with_nested_expressions() {
    // Test: x = (a + b) * c;
    // Ensures validation handles nested expression structures
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test(int a, int b, int c) {
            int x;
            x = (a + b) * c;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");
    let func = &ast.functions()[0];

    let assignment = func
        .body
        .iter()
        .find(|stmt| matches!(stmt, Statement::Assignment { .. }))
        .expect("Should have assignment");

    if let Statement::Assignment { target, value } = assignment {
        assert_eq!(target, "x");

        // Should be multiplication with nested addition
        match value {
            Expression::BinaryOp { left, right, .. } => {
                // Left should be (a + b)
                assert!(
                    matches!(**left, Expression::BinaryOp { .. }),
                    "Left operand should be binary operation"
                );
                // Right should be c
                assert!(
                    matches!(**right, Expression::Variable(_)),
                    "Right operand should be variable"
                );
            }
            _ => panic!("Expected binary operation"),
        }
    }
}
