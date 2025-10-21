//! Parser tests for pointer field access (ptr->field) - DECY-034 RED phase
//!
//! This test suite follows EXTREME TDD methodology - all tests should FAIL initially.
//! Tests verify that the parser correctly extracts pointer field access (C's -> operator).
//!
//! Root cause (from DECY-027 real-world validation):
//! - head->next becomes head (field access lost)
//! - r->bottom_right.x becomes incorrect nested dereferences
//!
//! References:
//! - K&R §6.2: Structures and Functions
//! - ISO C99 §6.5.2.3: Structure and union members

use decy_parser::parser::Expression;
use decy_parser::CParser;

#[test]
fn test_parse_simple_pointer_field_access() {
    // Test basic ptr->field pattern
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        struct Node {
            int value;
        };

        int get_value(struct Node* node) {
            return node->value;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.functions().len(), 1, "Should parse one function");

    let func = &ast.functions()[0];
    assert_eq!(func.name, "get_value");

    // Check the return statement
    if let decy_parser::parser::Statement::Return(Some(expr)) = &func.body[0] {
        // Should be Expression::PointerFieldAccess
        match expr {
            Expression::PointerFieldAccess { pointer, field } => {
                // Verify pointer is a variable reference to "node"
                assert!(
                    matches!(**pointer, Expression::Variable(_)),
                    "Pointer should be a variable"
                );

                // Verify field name is "value"
                assert_eq!(field, "value", "Field name should be 'value'");
            }
            _ => panic!(
                "Expected PointerFieldAccess expression, got {:?}",
                expr
            ),
        }
    } else {
        panic!("Expected return statement with expression");
    }
}

#[test]
fn test_parse_nested_pointer_field_access() {
    // Test nested access: ptr->field1.field2
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        struct Point {
            int x;
            int y;
        };

        struct Rectangle {
            struct Point top_left;
            struct Point bottom_right;
        };

        int get_x(struct Rectangle* r) {
            return r->bottom_right.x;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];
    assert_eq!(func.name, "get_x");

    // Check the return statement contains nested field access
    if let decy_parser::parser::Statement::Return(Some(expr)) = &func.body[0] {
        // Should be FieldAccess wrapping PointerFieldAccess
        match expr {
            Expression::FieldAccess { object, field } => {
                assert_eq!(field, "x", "Final field should be 'x'");

                // Object should be PointerFieldAccess(r, bottom_right)
                assert!(
                    matches!(**object, Expression::PointerFieldAccess { .. }),
                    "Object should be PointerFieldAccess"
                );
            }
            _ => panic!("Expected nested FieldAccess, got {:?}", expr),
        }
    } else {
        panic!("Expected return statement");
    }
}

#[test]
fn test_pointer_vs_direct_field_access() {
    // Test that parser distinguishes ptr->field from obj.field
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        struct Data {
            int value;
        };

        int compare(struct Data obj, struct Data* ptr) {
            int a = obj.value;    // Direct field access
            int b = ptr->value;   // Pointer field access
            return a + b;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];
    assert_eq!(func.body.len(), 3, "Should have 3 statements (2 decls + return)");

    // First variable: obj.value (direct field access)
    if let decy_parser::parser::Statement::VariableDeclaration {
        initializer: Some(expr),
        ..
    } = &func.body[0]
    {
        assert!(
            matches!(expr, Expression::FieldAccess { .. }),
            "First access should be FieldAccess (obj.value)"
        );
    }

    // Second variable: ptr->value (pointer field access)
    if let decy_parser::parser::Statement::VariableDeclaration {
        initializer: Some(expr),
        ..
    } = &func.body[1]
    {
        assert!(
            matches!(expr, Expression::PointerFieldAccess { .. }),
            "Second access should be PointerFieldAccess (ptr->value)"
        );
    }
}

#[test]
fn test_parse_pointer_field_in_assignment() {
    // Test ptr->field in assignment context
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        struct Node {
            int value;
            struct Node* next;
        };

        void traverse(struct Node* head) {
            head = head->next;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];
    assert_eq!(func.name, "traverse");

    // Check assignment statement
    if let decy_parser::parser::Statement::Assignment { value, .. } = &func.body[0] {
        // Right side should be pointer field access
        assert!(
            matches!(value, Expression::PointerFieldAccess { .. }),
            "Assignment value should be PointerFieldAccess"
        );
    } else {
        panic!("Expected assignment statement");
    }
}

#[test]
fn test_parse_chained_pointer_field_access() {
    // Test chaining: ptr->next->value
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        struct Node {
            int value;
            struct Node* next;
        };

        int get_next_value(struct Node* node) {
            return node->next->value;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];

    if let decy_parser::parser::Statement::Return(Some(expr)) = &func.body[0] {
        // Should be PointerFieldAccess wrapping PointerFieldAccess
        match expr {
            Expression::PointerFieldAccess { pointer, field } => {
                assert_eq!(field, "value", "Final field should be 'value'");

                // Pointer should itself be a PointerFieldAccess
                assert!(
                    matches!(**pointer, Expression::PointerFieldAccess { .. }),
                    "Pointer should be another PointerFieldAccess (node->next)"
                );
            }
            _ => panic!("Expected chained PointerFieldAccess, got {:?}", expr),
        }
    }
}
