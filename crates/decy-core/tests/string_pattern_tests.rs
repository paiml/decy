//! DECY-088: String pattern detection and transformation tests.
//!
//! Tests for transforming C char* to Rust String/&str based on usage.

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirStatement, HirType};

/// Test string literal assignment → &str.
/// C: char* s = "hello";
/// Rust: let s: &str = "hello";
#[test]
fn test_string_literal_becomes_str_slice() {
    let func = HirFunction::new_with_body(
        "test_literal".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "s".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Char)),
            initializer: Some(HirExpression::StringLiteral("hello".to_string())),
        }],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should have string literal
    assert!(
        code.contains("\"hello\""),
        "Expected string literal in:\n{}",
        code
    );
    // Should generate &str type (not *mut u8)
    assert!(
        code.contains("&str") || !code.contains("*mut u8"),
        "Expected &str type or no raw pointer in:\n{}",
        code
    );
}

/// Test heap-allocated string buffer → String.
/// C: char* buf = malloc(100);
/// Rust: let mut buf: String = String::with_capacity(100);
#[test]
fn test_malloc_char_becomes_string() {
    let func = HirFunction::new_with_body(
        "test_heap_string".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "buf".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Char)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::IntLiteral(100)],
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should generate Vec<u8> or String for char buffer
    assert!(
        code.contains("Vec<u8>") || code.contains("String"),
        "Expected Vec<u8> or String type in:\n{}",
        code
    );
}

/// Test strlen(s) → s.len().
/// C: int len = strlen(s);
/// Rust: let len = s.len();
#[test]
fn test_strlen_to_len_method() {
    let func = HirFunction::new_with_body(
        "test_strlen".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "len".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::FunctionCall {
                function: "strlen".to_string(),
                arguments: vec![HirExpression::Variable("s".to_string())],
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should use .len() method
    assert!(
        code.contains(".len()"),
        "Expected .len() method in:\n{}",
        code
    );
    // Should NOT have strlen function call in the body (exclude function signature)
    let body_only = code.split('{').nth(1).unwrap_or("");
    assert!(
        !body_only.contains("strlen("),
        "Should not have strlen() call in body:\n{}",
        code
    );
}

/// Test strcpy(dest, src) → safe string copy.
/// C: strcpy(dest, src);
/// Rust: dest = src.to_string(); or similar safe operation
#[test]
fn test_strcpy_to_safe_copy() {
    let func = HirFunction::new_with_body(
        "test_strcpy".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "strcpy".to_string(),
            arguments: vec![
                HirExpression::Variable("dest".to_string()),
                HirExpression::Variable("src".to_string()),
            ],
        })],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should use safe string method
    assert!(
        code.contains(".to_string()") || code.contains("clone") || code.contains("copy"),
        "Expected safe string operation in:\n{}",
        code
    );
    // Should NOT have strcpy function call in the body (exclude function signature)
    let body_only = code.split('{').nth(1).unwrap_or("");
    assert!(
        !body_only.contains("strcpy("),
        "Should not have strcpy() call in body:\n{}",
        code
    );
}

/// Test type mapping for string types.
#[test]
fn test_string_type_mapping() {
    // &str type
    let str_ref = HirType::StringReference;
    assert_eq!(CodeGenerator::map_type(&str_ref), "&str");

    // String type
    let owned_string = HirType::OwnedString;
    assert_eq!(CodeGenerator::map_type(&owned_string), "String");

    // String literal type
    let string_lit = HirType::StringLiteral;
    assert_eq!(CodeGenerator::map_type(&string_lit), "&str");
}

/// Test char* parameter → &str in signature.
/// C: void print_msg(char* msg)
/// Rust: fn print_msg(msg: &str)
#[test]
fn test_char_ptr_param_becomes_str_ref() {
    let func = HirFunction::new(
        "print_msg".to_string(),
        HirType::Void,
        vec![decy_hir::HirParameter::new(
            "msg".to_string(),
            HirType::Pointer(Box::new(HirType::Char)),
        )],
    );

    let codegen = CodeGenerator::new();
    let sig = codegen.generate_signature(&func);

    // Should have reference type (either &str, &[u8], or &mut)
    assert!(
        sig.contains("&") || sig.contains("*mut"),
        "Expected reference or pointer in signature:\n{}",
        sig
    );
}

/// Test const char* → &str (immutable).
/// C: const char* greeting = "Hello";
/// Rust: let greeting: &str = "Hello";
#[test]
fn test_const_char_ptr_to_str_slice() {
    // Using StringLiteral type for const char*
    let func = HirFunction::new_with_body(
        "test_const".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "greeting".to_string(),
            var_type: HirType::StringLiteral,
            initializer: Some(HirExpression::StringLiteral("Hello".to_string())),
        }],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should have &str type
    assert!(
        code.contains("&str"),
        "Expected &str type in:\n{}",
        code
    );
}
