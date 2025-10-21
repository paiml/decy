//! Unit tests for function call expressions in assignments (DECY-036 RED phase)
//!
//! These tests verify that function calls are correctly parsed when used in:
//! - Variable initializers: int* ptr = malloc(sizeof(int));
//! - Assignment statements: ptr = malloc(size);
//! - Return statements: return malloc(n);
//!
//! References:
//! - K&R §5.4: Pointers and Functions
//! - ISO C99 §6.5.2.2: Function calls

use decy_parser::parser::{Expression, Statement};
use decy_parser::CParser;

#[test]
fn test_parse_function_call_in_initializer() {
    // Test: int* ptr = malloc(sizeof(int));
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int* create_int() {
            int* ptr = malloc(sizeof(int));
            return ptr;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];
    assert_eq!(func.name, "create_int");

    // First statement should be a variable declaration with malloc initializer
    if let Statement::VariableDeclaration { name, initializer, .. } = &func.body[0] {
        assert_eq!(name, "ptr", "Variable name should be 'ptr'");

        assert!(
            initializer.is_some(),
            "Variable should have initializer (malloc call)"
        );

        if let Some(expr) = initializer {
            // Initializer should be a FunctionCall expression
            match expr {
                Expression::FunctionCall { function, arguments } => {
                    assert_eq!(function, "malloc", "Function name should be 'malloc'");
                    assert_eq!(arguments.len(), 1, "malloc should have 1 argument");

                    // Argument should be sizeof expression
                    assert!(
                        matches!(arguments[0], Expression::Sizeof { .. }),
                        "Argument should be sizeof expression"
                    );
                }
                _ => panic!("Expected FunctionCall expression in initializer, got {:?}", expr),
            }
        }
    } else {
        panic!("First statement should be VariableDeclaration, got {:?}", func.body[0]);
    }
}

#[test]
fn test_parse_malloc_with_sizeof() {
    // Test: struct Node* node = malloc(sizeof(struct Node));
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        struct Node {
            int value;
            struct Node* next;
        };

        struct Node* create_node() {
            struct Node* node = malloc(sizeof(struct Node));
            return node;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];
    assert_eq!(func.name, "create_node");

    // Check malloc(sizeof(struct Node)) pattern
    if let Statement::VariableDeclaration { initializer, .. } = &func.body[0] {
        assert!(
            initializer.is_some(),
            "Variable should have malloc initializer"
        );

        if let Some(Expression::FunctionCall { function, arguments }) = initializer {
            assert_eq!(function, "malloc");
            assert_eq!(arguments.len(), 1);

            if let Expression::Sizeof { type_name } = &arguments[0] {
                assert_eq!(type_name, "struct Node", "sizeof should have struct type");
            } else {
                panic!("malloc argument should be sizeof expression");
            }
        } else {
            panic!("Initializer should be FunctionCall");
        }
    } else {
        panic!("First statement should be VariableDeclaration");
    }
}

#[test]
fn test_parse_function_call_in_assignment() {
    // Test: Simple assignment with function call (not dereference)
    // BUG: x = malloc(size); is not being parsed/extracted
    // Expected: VariableDeclaration, Assignment, Return
    // Actual: VariableDeclaration, Return (Assignment missing!)
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void* allocate(int size) {
            void* x;
            x = malloc(size);
            return x;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];
    assert_eq!(func.name, "allocate");

    // Function should have statements: declaration, assignment, return
    assert!(
        func.body.len() >= 2,
        "Function should have at least 2 statements (declaration + assignment), found {}",
        func.body.len()
    );

    // Second statement should be Assignment with malloc on RHS
    if let Statement::Assignment { target, value } = &func.body[1] {
        assert_eq!(target, "x", "Assignment target should be 'x'");

        // Value should be malloc function call
        match value {
            Expression::FunctionCall { function, arguments } => {
                assert_eq!(function, "malloc", "Function should be malloc");
                assert_eq!(arguments.len(), 1, "malloc should have 1 argument");
            }
            _ => panic!("Assignment value should be FunctionCall, got {:?}", value),
        }
    } else {
        panic!("Second statement should be Assignment with malloc call, got {:?}", func.body[1]);
    }
}

#[test]
fn test_parse_function_call_in_return() {
    // Test: return malloc(n);
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int* allocate(int n) {
            return malloc(n * sizeof(int));
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];
    assert_eq!(func.name, "allocate");

    // Return statement should have malloc call
    if let Statement::Return(Some(expr)) = &func.body[0] {
        match expr {
            Expression::FunctionCall { function, .. } => {
                assert_eq!(function, "malloc", "Return value should be malloc call");
            }
            _ => panic!("Return expression should be FunctionCall, got {:?}", expr),
        }
    } else {
        panic!("First statement should be Return with malloc call");
    }
}

#[test]
fn test_parse_nested_function_calls() {
    // Test: strlen(strdup(s))
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int copy_length(char* s) {
            return strlen(strdup(s));
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    let func = &ast.functions()[0];
    assert_eq!(func.name, "copy_length");

    if let Statement::Return(Some(expr)) = &func.body[0] {
        // Outer call should be strlen
        if let Expression::FunctionCall { function, arguments } = expr {
            assert_eq!(function, "strlen", "Outer function should be strlen");
            assert_eq!(arguments.len(), 1, "strlen should have 1 argument");

            // Inner call should be strdup
            if let Expression::FunctionCall { function: inner_func, .. } = &arguments[0] {
                assert_eq!(inner_func, "strdup", "Inner function should be strdup");
            } else {
                panic!("Argument should be FunctionCall (strdup)");
            }
        } else {
            panic!("Return should be FunctionCall");
        }
    } else {
        panic!("Should have return statement");
    }
}
