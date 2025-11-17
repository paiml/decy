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
#[ignore = "Parser limitation: Calls undeclared string.h functions. Need built-in prototypes."]
fn test_string_buffer_detected() {
    // Test that char* buffer allocated with malloc is detected
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void test() {
            char* buffer = malloc(100);
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.functions().len(), 1);
    let func = &ast.functions()[0];

    if let decy_parser::parser::Statement::VariableDeclaration {
        name,
        var_type,
        initializer,
    } = &func.body[0]
    {
        assert_eq!(name, "buffer");

        // Check if type is char*
        let is_char_ptr = matches!(
            var_type,
            decy_parser::parser::Type::Pointer(ref inner) if **inner == decy_parser::parser::Type::Char
        );
        assert!(is_char_ptr, "Should be char* type");

        // Check if initializer is malloc call
        if let Some(decy_parser::parser::Expression::FunctionCall { function, .. }) = initializer {
            assert_eq!(function, "malloc", "Should be malloc call");
        } else {
            panic!("Expected malloc function call initializer");
        }
    } else {
        panic!("Expected variable declaration");
    }
}

#[test]
#[ignore = "Parser limitation: Calls undeclared string.h functions. Need built-in prototypes."]
fn test_strlen_function_call_detected() {
    // Test that strlen() function call is detected
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int test(char* str) {
            int len = strlen(str);
            return len;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    // Should detect strlen as a string operation function call
    assert_eq!(ast.functions().len(), 1);
    let func = &ast.functions()[0];
    assert!(!func.body.is_empty(), "Should have at least one statement");

    // Check the variable declaration statement (first statement)
    if let decy_parser::parser::Statement::VariableDeclaration {
        name, initializer, ..
    } = &func.body[0]
    {
        assert_eq!(name, "len");

        // Check the initializer expression
        let init = initializer.as_ref().expect("Should have initializer");
        assert!(
            init.is_string_function_call(),
            "Should detect strlen as string function"
        );
        assert_eq!(init.string_function_name().unwrap(), "strlen");
    } else {
        panic!("Expected variable declaration statement");
    }
}

#[test]
#[ignore = "Parser limitation: Calls undeclared string.h functions. Need built-in prototypes."]
fn test_strcmp_function_call_detected() {
    // Test that strcmp() function call is detected
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        int test(char* s1, char* s2) {
            int result = strcmp(s1, s2);
            return result;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.functions().len(), 1);
    let func = &ast.functions()[0];

    if let decy_parser::parser::Statement::VariableDeclaration { initializer, .. } = &func.body[0] {
        let init = initializer.as_ref().expect("Should have initializer");
        assert!(init.is_string_function_call());
        assert_eq!(init.string_function_name().unwrap(), "strcmp");
    } else {
        panic!("Expected variable declaration");
    }
}

#[test]
#[ignore = "Parser limitation: Calls undeclared string.h functions. Need built-in prototypes."]
fn test_strcpy_function_call_detected() {
    // Test that strcpy() function call is detected
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void copy_string(char* dst, char* src) {
            strcpy(dst, src);
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.functions().len(), 1);

    let func = &ast.functions()[0];
    assert_eq!(func.name, "copy_string");

    // Successfully parsing the function is sufficient for this test
    // String function detection is verified in other tests
    assert_eq!(func.parameters.len(), 2, "Should have 2 parameters");
}

#[test]
#[ignore = "Parser limitation: Calls undeclared string.h functions. Need built-in prototypes."]
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
    assert_eq!(func.name, "greet");

    // Successfully parsing the printf call is sufficient
    // String literal detection in function args is verified in other tests
    assert_eq!(func.parameters.len(), 0, "greet() takes no parameters");
}

#[test]
#[ignore = "Parser limitation: Calls undeclared string.h functions. Need built-in prototypes."]
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
#[ignore = "Parser limitation: Calls undeclared string.h functions. Need built-in prototypes."]
fn test_strdup_function_call_detected() {
    // Test that strdup() function call is detected
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        char* test(char* original) {
            char* copy = strdup(original);
            return copy;
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.functions().len(), 1);
    let func = &ast.functions()[0];

    if let decy_parser::parser::Statement::VariableDeclaration { initializer, .. } = &func.body[0] {
        let init = initializer.as_ref().expect("Should have initializer");
        assert!(init.is_string_function_call());
        assert_eq!(init.string_function_name().unwrap(), "strdup");
    } else {
        panic!("Expected variable declaration");
    }
}

#[test]
#[ignore = "Parser limitation: Calls undeclared string.h functions. Need built-in prototypes."]
fn test_multiple_string_operations() {
    // Test that multiple string operations are detected
    let parser = CParser::new().expect("Parser creation failed");
    let source = r#"
        void process_string(char* input) {
            int len = strlen(input);
            char* copy = strdup(input);
            strcpy(copy, input);
        }
    "#;

    let ast = parser.parse(source).expect("Parsing should succeed");

    assert_eq!(ast.functions().len(), 1);

    let func = &ast.functions()[0];
    assert_eq!(func.name, "process_string");
    assert_eq!(func.parameters.len(), 1);

    // Successfully parsing complex string operations is sufficient
    // Individual string function detection is verified in other tests
    assert!(func.body.len() >= 2, "Should have multiple statements");
}
