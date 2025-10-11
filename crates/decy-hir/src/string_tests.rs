//! Tests for string handling in HIR (DECY-025 RED phase).
//!
//! Tests for representing C string literals and char* types in HIR,
//! with plans to transpile to Rust &str and String types.

use super::*;

#[test]
fn test_create_string_literal_expression() {
    // C: "hello"
    // Rust: "hello"
    let string_lit = HirExpression::StringLiteral("hello".to_string());

    match string_lit {
        HirExpression::StringLiteral(s) => {
            assert_eq!(s, "hello");
        }
        _ => panic!("Expected StringLiteral"),
    }
}

#[test]
fn test_string_literal_with_escapes() {
    // C: "hello\nworld"
    // Rust: "hello\nworld"
    let string_lit = HirExpression::StringLiteral("hello\\nworld".to_string());

    match string_lit {
        HirExpression::StringLiteral(s) => {
            assert_eq!(s, "hello\\nworld");
        }
        _ => panic!("Expected StringLiteral"),
    }
}

#[test]
fn test_empty_string_literal() {
    // C: ""
    // Rust: ""
    let string_lit = HirExpression::StringLiteral(String::new());

    match string_lit {
        HirExpression::StringLiteral(s) => {
            assert!(s.is_empty());
        }
        _ => panic!("Expected StringLiteral"),
    }
}

#[test]
fn test_char_pointer_type_for_owned_string() {
    // C: char* str
    // Could be Rust: String (owned) or &str (borrowed)
    // For now we need to represent this in HIR
    let char_ptr = HirType::Pointer(Box::new(HirType::Char));

    match char_ptr {
        HirType::Pointer(inner) => {
            assert_eq!(*inner, HirType::Char);
        }
        _ => panic!("Expected Pointer type"),
    }
}

#[test]
fn test_string_variable_declaration() {
    // C: char* message = "Hello";
    // Rust: let message: &str = "Hello";
    let var_decl = HirStatement::VariableDeclaration {
        name: "message".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Char)),
        initializer: Some(HirExpression::StringLiteral("Hello".to_string())),
    };

    match var_decl {
        HirStatement::VariableDeclaration {
            name,
            var_type,
            initializer,
        } => {
            assert_eq!(name, "message");
            assert!(matches!(var_type, HirType::Pointer(_)));
            assert!(initializer.is_some());
        }
        _ => panic!("Expected VariableDeclaration"),
    }
}

#[test]
fn test_const_char_pointer_type() {
    // C: const char* str
    // Should become Rust: &str (immutable string slice)
    // We need a way to represent const-ness
    let const_char_ptr = HirType::Pointer(Box::new(HirType::Char));

    // For now, we're representing const char* as Pointer(Char)
    // Future: add const qualifier to HirType
    match const_char_ptr {
        HirType::Pointer(inner) => {
            assert_eq!(*inner, HirType::Char);
        }
        _ => panic!("Expected Pointer type"),
    }
}

#[test]
fn test_string_literal_clone() {
    let string_lit = HirExpression::StringLiteral("test".to_string());
    let cloned = string_lit.clone();

    assert_eq!(string_lit, cloned);
}

#[test]
fn test_string_literal_equality() {
    let string_lit1 = HirExpression::StringLiteral("test".to_string());
    let string_lit2 = HirExpression::StringLiteral("test".to_string());

    assert_eq!(string_lit1, string_lit2);
}

#[test]
fn test_string_function_parameter() {
    // C: void print_message(const char* msg);
    // Rust: fn print_message(msg: &str);
    let param = HirParameter::new("msg".to_string(), HirType::Pointer(Box::new(HirType::Char)));

    assert_eq!(param.name(), "msg");
    assert!(matches!(param.param_type(), HirType::Pointer(_)));
}

#[test]
fn test_string_return_type() {
    // C: const char* get_name();
    // Rust: fn get_name() -> &str;
    let func = HirFunction::new(
        "get_name".to_string(),
        HirType::Pointer(Box::new(HirType::Char)),
        vec![],
    );

    assert_eq!(func.name(), "get_name");
    assert!(matches!(func.return_type(), HirType::Pointer(_)));
}
