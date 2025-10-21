//! Parser tests for string handling (DECY-025 RED phase)
//!
//! This test suite follows EXTREME TDD methodology - all tests should FAIL initially.
//! Tests verify that the parser correctly identifies string literals vs buffers.
//!
//! References:
//! - K&R §5.5: Character Pointers and Functions
//! - ISO C99 §7.21: String handling

use decy_parser::CParser;

#[test]
fn test_string_literal_detected() {
    // Test that const char* with string literal is detected
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        const char* msg = "Hello, world!";
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.variables().len(), 1, "Should parse one variable");

    let var = &ast.variables()[0];
    assert_eq!(var.name(), "msg");
    assert!(
        var.is_string_literal(),
        "Should be recognized as string literal"
    );
}

#[test]
fn test_string_buffer_detected() {
    // Test that char* buffer allocated with malloc is detected
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        char* buffer = malloc(100);
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.variables().len(), 1);

    let var = &ast.variables()[0];
    assert_eq!(var.name(), "buffer");
    assert!(
        var.is_string_buffer(),
        "Should be recognized as owned string buffer"
    );
}

#[test]
fn test_strlen_function_call_detected() {
    // Test that strlen() function call is detected
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int len = strlen(str);
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    // Should detect strlen as a string operation function call
    assert_eq!(ast.variables().len(), 1);

    let var = &ast.variables()[0];
    assert_eq!(var.name(), "len");

    // Check the initializer expression
    let initializer = var.initializer().expect("Should have initializer");
    assert!(
        initializer.is_string_function_call(),
        "Should detect strlen as string function"
    );
    assert_eq!(
        initializer.string_function_name().unwrap(),
        "strlen"
    );
}

#[test]
fn test_strcmp_function_call_detected() {
    // Test that strcmp() function call is detected
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int result = strcmp(s1, s2);
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.variables().len(), 1);

    let var = &ast.variables()[0];
    let initializer = var.initializer().expect("Should have initializer");
    assert!(initializer.is_string_function_call());
    assert_eq!(initializer.string_function_name().unwrap(), "strcmp");
}

#[test]
fn test_strcpy_function_call_detected() {
    // Test that strcpy() function call is detected
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void copy_string(char* dst, const char* src) {
            strcpy(dst, src);
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.functions().len(), 1);

    let func = &ast.functions()[0];
    assert_eq!(func.name, "copy_string");

    // Check the function body for strcpy call
    let body = &func.body;
    assert_eq!(body.len(), 1, "Should have one statement");

    // First statement should be strcpy call
    assert!(
        body[0].is_string_function_call(),
        "Should detect strcpy as string function"
    );
}

#[test]
fn test_string_literal_in_function_parameter() {
    // Test that string literal passed as parameter is detected
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void greet() {
            printf("Hello");
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.functions().len(), 1);

    let func = &ast.functions()[0];
    let body = &func.body;

    // Should detect printf call with string literal argument
    assert!(body[0].is_function_call());
    let call_expr = body[0].as_function_call().expect("Should be function call");
    assert!(
        call_expr.has_string_literal_argument(),
        "Should detect string literal in arguments"
    );
}

#[test]
fn test_char_pointer_parameter_analysis() {
    // Test that char* parameters are analyzed for usage patterns
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int count_chars(const char* str) {
            return strlen(str);
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.functions().len(), 1);

    let func = &ast.functions()[0];
    assert_eq!(func.parameters.len(), 1);

    let param = &func.parameters[0];
    assert_eq!(param.name, "str");
    assert!(
        param.is_const_char_pointer(),
        "Should recognize const char* parameter"
    );
}

#[test]
fn test_strdup_function_call_detected() {
    // Test that strdup() function call is detected
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        char* copy = strdup(original);
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.variables().len(), 1);

    let var = &ast.variables()[0];
    let initializer = var.initializer().expect("Should have initializer");
    assert!(initializer.is_string_function_call());
    assert_eq!(initializer.string_function_name().unwrap(), "strdup");
}

#[test]
fn test_multiple_string_operations() {
    // Test that multiple string operations are detected
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void process_string(const char* input) {
            int len = strlen(input);
            char* copy = strdup(input);
            strcpy(copy, input);
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.functions().len(), 1);

    let func = &ast.functions()[0];
    let body = &func.body;

    // Should detect strlen, strdup, and strcpy
    assert!(body.iter().filter(|stmt| stmt.is_string_function_call()).count() >= 2,
            "Should detect multiple string function calls");
}
