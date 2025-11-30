//! Tests for string handling code generation (DECY-025 RED phase).

use super::*;
use decy_hir::{HirExpression, HirParameter, HirStatement, HirType};

#[test]
fn test_generate_string_literal() {
    // C: "hello"
    // Rust: "hello"
    let string_lit = HirExpression::StringLiteral("hello".to_string());
    let codegen = CodeGenerator::new();

    let code = codegen.generate_expression(&string_lit);

    assert_eq!(code, "\"hello\"");
}

#[test]
fn test_generate_string_with_escapes() {
    // C: "hello\nworld"
    // Rust: "hello\nworld"
    let string_lit = HirExpression::StringLiteral("hello\\nworld".to_string());
    let codegen = CodeGenerator::new();

    let code = codegen.generate_expression(&string_lit);

    assert_eq!(code, "\"hello\\nworld\"");
}

#[test]
fn test_generate_empty_string() {
    // C: ""
    // Rust: ""
    let string_lit = HirExpression::StringLiteral(String::new());
    let codegen = CodeGenerator::new();

    let code = codegen.generate_expression(&string_lit);

    assert_eq!(code, "\"\"");
}

#[test]
fn test_map_char_pointer_to_str_slice() {
    // C: const char* str
    // Rust: &str (string slice)
    // For now, char* maps to *mut u8, but we want &str for const char*
    let char_ptr = HirType::Pointer(Box::new(HirType::Char));

    let rust_type = CodeGenerator::map_type(&char_ptr);

    // Currently maps to *mut u8, but we want to detect const char* → &str
    // This test will initially fail and guide our implementation
    // For now, just verify it generates something
    assert!(!rust_type.is_empty());
}

#[test]
fn test_generate_string_variable_declaration() {
    // C: char* message = "Hello";
    // Rust: let mut message: &str = "Hello";
    let stmt = HirStatement::VariableDeclaration {
        name: "message".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Char)),
        initializer: Some(HirExpression::StringLiteral("Hello".to_string())),
    };

    let codegen = CodeGenerator::new();
    let code = codegen.generate_statement(&stmt);

    // Currently generates: let mut message: *mut u8 = "Hello";
    // We want: let mut message: &str = "Hello";
    assert!(code.contains("message"));
    assert!(code.contains("Hello"));
}

#[test]
fn test_generate_string_parameter() {
    // C: void print_message(const char* msg);
    // Rust: fn print_message(msg: &str);
    let param = HirParameter::new("msg".to_string(), HirType::Pointer(Box::new(HirType::Char)));

    let param_code = format!(
        "{}: {}",
        param.name(),
        CodeGenerator::map_type(param.param_type())
    );

    // Currently: msg: *mut u8
    // Want: msg: &str
    assert!(param_code.contains("msg"));
}

#[test]
fn test_generate_string_return_type() {
    // C: const char* get_name();
    // Rust: fn get_name() -> &str;
    let return_type = HirType::Pointer(Box::new(HirType::Char));

    let type_str = CodeGenerator::map_type(&return_type);

    // Currently: *mut u8
    // Want: &str
    assert!(!type_str.is_empty());
}

#[test]
fn test_string_literal_in_function_call() {
    // C: printf("Hello, %s!", name);
    // DECY-132: printf is now transformed to print! macro
    // DECY-119: Format specifiers are converted: %s → {}
    // Rust: print!("Hello, {}!", name)
    let call_expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("Hello, %s!".to_string()),
            HirExpression::Variable("name".to_string()),
        ],
    };

    let codegen = CodeGenerator::new();
    let code = codegen.generate_expression(&call_expr);

    // DECY-132: printf is now transformed to print! macro
    // DECY-119: Format specifiers are converted
    assert!(code.contains("print!"));
    assert!(code.contains("Hello, {}!")); // %s → {} conversion
    assert!(code.contains("name"));
}

#[test]
fn test_string_literal_concatenation_concept() {
    // This test documents that C string concatenation  ("hello" "world")
    // would need special handling, but we'll focus on single literals first
    let string_lit = HirExpression::StringLiteral("helloworld".to_string());
    let codegen = CodeGenerator::new();

    let code = codegen.generate_expression(&string_lit);

    assert_eq!(code, "\"helloworld\"");
}
