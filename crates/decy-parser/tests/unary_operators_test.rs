//! Parser tests for unary operators (DECY-035 RED phase)
//!
//! This test suite follows EXTREME TDD methodology - tests should initially FAIL or PASS
//! depending on current implementation state.
//!
//! Root cause (from DECY-027 real-world validation):
//! - -x becomes x (negation lost)
//! - !condition may be lost
//! - ~bits (bitwise NOT) not handled
//! - &variable (address-of) incomplete
//!
//! References:
//! - K&R §2.10: Unary Operators
//! - ISO C99 §6.5.3: Unary operators

use decy_parser::parser::{Expression, UnaryOperator};
use decy_parser::CParser;

#[test]
fn test_parse_unary_minus() {
    // Test unary minus: -x
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int negate(int x) {
            return -x;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.functions().len(), 1, "Should parse one function");

    let func = &ast.functions()[0];
    assert_eq!(func.name, "negate");

    // Check the return statement
    if let decy_parser::parser::Statement::Return(Some(expr)) = &func.body[0] {
        match expr {
            Expression::UnaryOp { op, operand } => {
                assert_eq!(
                    *op,
                    UnaryOperator::Minus,
                    "Operator should be unary minus"
                );

                // Operand should be a variable reference to "x"
                assert!(
                    matches!(**operand, Expression::Variable(_)),
                    "Operand should be a variable"
                );
            }
            _ => panic!("Expected UnaryOp expression for -x, got {:?}", expr),
        }
    } else {
        panic!("Expected return statement with expression");
    }
}

#[test]
fn test_parse_logical_not() {
    // Test logical NOT: !x
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int logical_not(int condition) {
            return !condition;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];
    assert_eq!(func.name, "logical_not");

    if let decy_parser::parser::Statement::Return(Some(expr)) = &func.body[0] {
        match expr {
            Expression::UnaryOp { op, operand } => {
                assert_eq!(
                    *op,
                    UnaryOperator::LogicalNot,
                    "Operator should be logical NOT"
                );

                assert!(
                    matches!(**operand, Expression::Variable(_)),
                    "Operand should be a variable"
                );
            }
            _ => panic!("Expected UnaryOp expression for !x, got {:?}", expr),
        }
    } else {
        panic!("Expected return statement");
    }
}

#[test]
fn test_parse_bitwise_not() {
    // Test bitwise NOT: ~x
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int bitwise_not(int bits) {
            return ~bits;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];
    assert_eq!(func.name, "bitwise_not");

    if let decy_parser::parser::Statement::Return(Some(expr)) = &func.body[0] {
        // This will likely fail initially - BitwiseNot not in UnaryOperator enum
        match expr {
            Expression::UnaryOp { op, .. } => {
                // Check if BitwiseNot exists (will fail if not implemented)
                assert!(
                    format!("{:?}", op).contains("BitwiseNot"),
                    "Operator should be bitwise NOT (~), got {:?}",
                    op
                );
            }
            _ => panic!(
                "Expected UnaryOp expression for ~bits, got {:?}",
                expr
            ),
        }
    } else {
        panic!("Expected return statement");
    }
}

#[test]
fn test_parse_address_of_operator() {
    // Test address-of: &x
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int* get_address(int x) {
            return &x;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];
    assert_eq!(func.name, "get_address");

    if let decy_parser::parser::Statement::Return(Some(expr)) = &func.body[0] {
        // This will likely fail - AddressOf not in Expression enum
        // Check for either AddressOf variant or UnaryOp with AddressOf operator
        let is_address_of = match expr {
            Expression::UnaryOp { op, .. } => {
                format!("{:?}", op).contains("AddressOf")
            }
            _ => format!("{:?}", expr).contains("AddressOf"),
        };

        assert!(
            is_address_of,
            "Expected AddressOf expression for &x, got {:?}",
            expr
        );
    } else {
        panic!("Expected return statement");
    }
}

#[test]
fn test_unary_vs_binary_minus() {
    // Test that parser distinguishes unary minus from binary subtraction
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int test_minus(int a, int b) {
            int unary = -a;      // Unary minus
            int binary = b - a;  // Binary subtraction
            return unary;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];
    assert_eq!(
        func.body.len(),
        3,
        "Should have 3 statements (2 decls + return)"
    );

    // First variable: unary = -a (unary minus)
    if let decy_parser::parser::Statement::VariableDeclaration {
        name,
        initializer: Some(expr),
        ..
    } = &func.body[0]
    {
        assert_eq!(name, "unary");
        assert!(
            matches!(expr, Expression::UnaryOp { .. }),
            "First should be UnaryOp (-a), got {:?}",
            expr
        );
    } else {
        panic!("Expected first variable declaration");
    }

    // Second variable: binary = b - a (binary subtraction)
    if let decy_parser::parser::Statement::VariableDeclaration {
        name,
        initializer: Some(expr),
        ..
    } = &func.body[1]
    {
        assert_eq!(name, "binary");
        assert!(
            matches!(expr, Expression::BinaryOp { .. }),
            "Second should be BinaryOp (b - a), got {:?}",
            expr
        );
    } else {
        panic!("Expected second variable declaration");
    }
}

#[test]
fn test_unary_minus_in_expression() {
    // Test unary minus in complex expression: -(a + b)
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int negate_sum(int a, int b) {
            return -(a + b);
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];

    if let decy_parser::parser::Statement::Return(Some(expr)) = &func.body[0] {
        match expr {
            Expression::UnaryOp { op, operand } => {
                assert_eq!(
                    *op,
                    UnaryOperator::Minus,
                    "Outer operator should be unary minus"
                );

                // Operand should be a binary expression (a + b)
                assert!(
                    matches!(**operand, Expression::BinaryOp { .. }),
                    "Operand should be BinaryOp (a + b), got {:?}",
                    operand
                );
            }
            _ => panic!("Expected UnaryOp wrapping BinaryOp, got {:?}", expr),
        }
    } else {
        panic!("Expected return statement");
    }
}

#[test]
fn test_double_negation() {
    // Test double negation: !!x (logical NOT twice)
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int double_not(int x) {
            return !!x;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];

    if let decy_parser::parser::Statement::Return(Some(expr)) = &func.body[0] {
        match expr {
            Expression::UnaryOp { op, operand } => {
                assert_eq!(*op, UnaryOperator::LogicalNot, "Outer should be !");

                // Inner should also be UnaryOp with LogicalNot
                assert!(
                    matches!(
                        **operand,
                        Expression::UnaryOp {
                            op: UnaryOperator::LogicalNot,
                            ..
                        }
                    ),
                    "Inner should also be UnaryOp (!), got {:?}",
                    operand
                );
            }
            _ => panic!("Expected nested UnaryOp, got {:?}", expr),
        }
    } else {
        panic!("Expected return statement");
    }
}

#[test]
fn test_unary_operators_in_assignment() {
    // Test unary operators in assignment context
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test_assignment(int x) {
            int neg = -x;
            int not_val = !x;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];
    assert_eq!(func.body.len(), 2, "Should have 2 variable declarations");

    // First: neg = -x
    if let decy_parser::parser::Statement::VariableDeclaration {
        initializer: Some(expr),
        ..
    } = &func.body[0]
    {
        assert!(
            matches!(
                expr,
                Expression::UnaryOp {
                    op: UnaryOperator::Minus,
                    ..
                }
            ),
            "First should be unary minus"
        );
    }

    // Second: not_val = !x
    if let decy_parser::parser::Statement::VariableDeclaration {
        initializer: Some(expr),
        ..
    } = &func.body[1]
    {
        assert!(
            matches!(
                expr,
                Expression::UnaryOp {
                    op: UnaryOperator::LogicalNot,
                    ..
                }
            ),
            "Second should be logical NOT"
        );
    }
}
