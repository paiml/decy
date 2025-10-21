//! Code generation tests for string handling (DECY-025 RED phase)
//!
//! This test suite follows EXTREME TDD methodology - all tests should FAIL initially.
//! Tests verify that codegen correctly generates Rust string code from HIR.
//!
//! References:
//! - K&R §5.5: Character Pointers and Functions
//! - ISO C99 §7.21: String handling

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirType};

#[test]
fn test_codegen_string_literal_type() {
    // Test that HirType::StringLiteral generates &str
    let string_lit_type = HirType::StringLiteral;

    let rust_type = CodeGenerator::map_type(&string_lit_type);

    assert_eq!(rust_type, "&str", "StringLiteral should map to &str");
}

#[test]
fn test_codegen_owned_string_type() {
    // Test that HirType::OwnedString generates String
    let owned_string_type = HirType::OwnedString;

    let rust_type = CodeGenerator::map_type(&owned_string_type);

    assert_eq!(rust_type, "String", "OwnedString should map to String");
}

#[test]
fn test_codegen_string_reference_type() {
    // Test that HirType::StringReference generates &str
    let string_ref_type = HirType::StringReference;

    let rust_type = CodeGenerator::map_type(&string_ref_type);

    assert_eq!(rust_type, "&str", "StringReference should map to &str");
}

#[test]
fn test_codegen_string_literal_expression() {
    // Test that string literal expression generates Rust string literal
    let expr = HirExpression::StringLiteral {
        value: "Hello, world!".to_string(),
    };

    let rust_code = CodeGenerator::generate_expression(&expr);

    assert_eq!(
        rust_code, r#""Hello, world!""#,
        "Should generate quoted string literal"
    );
}

#[test]
fn test_codegen_strlen_to_len() {
    // Test that strlen(s) generates s.len()
    let strlen_expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable {
            name: "s".to_string(),
        }),
        method: "len".to_string(),
        arguments: vec![],
    };

    let rust_code = CodeGenerator::generate_expression(&strlen_expr);

    assert_eq!(rust_code, "s.len()", "strlen should become .len()");
}

#[test]
fn test_codegen_strcmp_to_equality() {
    // Test that strcmp(s1, s2) generates s1 == s2
    let strcmp_expr = HirExpression::BinaryOp {
        left: Box::new(HirExpression::Variable {
            name: "s1".to_string(),
        }),
        operator: decy_hir::BinaryOperator::Equal,
        right: Box::new(HirExpression::Variable {
            name: "s2".to_string(),
        }),
    };

    let rust_code = CodeGenerator::generate_expression(&strcmp_expr);

    assert_eq!(
        rust_code, "s1 == s2",
        "strcmp should become equality operator"
    );
}

#[test]
fn test_codegen_strcpy_to_clone_into() {
    // Test that strcpy(dst, src) generates src.clone_into(&mut dst)
    let strcpy_expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable {
            name: "src".to_string(),
        }),
        method: "clone_into".to_string(),
        arguments: vec![HirExpression::Variable {
            name: "dst".to_string(),
        }],
    };

    let rust_code = CodeGenerator::generate_expression(&strcpy_expr);

    assert_eq!(
        rust_code, "src.clone_into(&mut dst)",
        "strcpy should become clone_into"
    );
}

#[test]
fn test_codegen_strdup_to_to_string() {
    // Test that strdup(s) generates s.to_string()
    let strdup_expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable {
            name: "s".to_string(),
        }),
        method: "to_string".to_string(),
        arguments: vec![],
    };

    let rust_code = CodeGenerator::generate_expression(&strdup_expr);

    assert_eq!(
        rust_code, "s.to_string()",
        "strdup should become to_string()"
    );
}

#[test]
fn test_codegen_const_char_pointer_parameter() {
    // Test that const char* parameter generates &str
    use decy_hir::{HirFunction, HirParameter};

    let param = HirParameter::new("msg".to_string(), HirType::StringReference);

    let func = HirFunction {
        name: "print_message".to_string(),
        parameters: vec![param],
        return_type: HirType::Void,
        body: vec![],
    };

    let rust_code = CodeGenerator::generate_function(&func);

    assert!(
        rust_code.contains("msg: &str"),
        "const char* parameter should become &str"
    );
}

#[test]
fn test_codegen_owned_char_pointer_parameter() {
    // Test that char* (owned) parameter generates String
    use decy_hir::{HirFunction, HirParameter};

    let param = HirParameter::new("buffer".to_string(), HirType::OwnedString);

    let func = HirFunction {
        name: "process_buffer".to_string(),
        parameters: vec![param],
        return_type: HirType::Void,
        body: vec![],
    };

    let rust_code = CodeGenerator::generate_function(&func);

    assert!(
        rust_code.contains("buffer: String"),
        "owned char* parameter should become String"
    );
}
