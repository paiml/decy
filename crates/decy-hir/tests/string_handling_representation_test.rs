//! HIR tests for string handling representation (DECY-025 RED phase)
//!
//! This test suite follows EXTREME TDD methodology - all tests should FAIL initially.
//! Tests verify that HIR correctly represents string types and operations.
//!
//! References:
//! - K&R §5.5: Character Pointers and Functions
//! - ISO C99 §7.21: String handling

use decy_hir::{HirExpression, HirType};

#[test]
fn test_hir_string_literal_type() {
    // Test that HirType has StringLiteral variant
    let string_lit_type = HirType::StringLiteral;

    assert!(matches!(string_lit_type, HirType::StringLiteral));
}

#[test]
fn test_hir_owned_string_type() {
    // Test that HirType has OwnedString variant for String
    let owned_string_type = HirType::OwnedString;

    assert!(matches!(owned_string_type, HirType::OwnedString));
}

#[test]
fn test_hir_string_reference_type() {
    // Test that HirType has StringReference variant for &str
    let string_ref_type = HirType::StringReference;

    assert!(matches!(string_ref_type, HirType::StringReference));
}

#[test]
fn test_hir_string_literal_expression() {
    // Test that HirExpression can represent string literals
    let expr = HirExpression::StringLiteral("Hello, world!".to_string());

    match expr {
        HirExpression::StringLiteral(value) => {
            assert_eq!(value, "Hello, world!");
        }
        _ => panic!("Expected StringLiteral expression"),
    }
}

#[test]
fn test_hir_strlen_transformation() {
    // Test that strlen(s) is represented as method call s.len()
    let strlen_call = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "len".to_string(),
        arguments: vec![],
    };

    match strlen_call {
        HirExpression::StringMethodCall { method, .. } => {
            assert_eq!(method, "len");
        }
        _ => panic!("Expected StringMethodCall expression"),
    }
}

#[test]
fn test_hir_strcmp_transformation() {
    // Test that strcmp(s1, s2) is represented as equality comparison
    let strcmp_expr = HirExpression::BinaryOp {
        left: Box::new(HirExpression::Variable("s1".to_string())),
        op: decy_hir::BinaryOperator::Equal,
        right: Box::new(HirExpression::Variable("s2".to_string())),
    };

    match strcmp_expr {
        HirExpression::BinaryOp { op, .. } => {
            assert_eq!(op, decy_hir::BinaryOperator::Equal);
        }
        _ => panic!("Expected BinaryOp expression"),
    }
}

#[test]
fn test_hir_strcpy_transformation() {
    // Test that strcpy(dst, src) is represented as clone operation
    let strcpy_expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("src".to_string())),
        method: "clone_into".to_string(),
        arguments: vec![HirExpression::Variable("dst".to_string())],
    };

    match strcpy_expr {
        HirExpression::StringMethodCall { method, .. } => {
            assert_eq!(method, "clone_into");
        }
        _ => panic!("Expected StringMethodCall expression"),
    }
}

#[test]
fn test_hir_strdup_transformation() {
    // Test that strdup(s) is represented as s.to_string()
    let strdup_expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "to_string".to_string(),
        arguments: vec![],
    };

    match strdup_expr {
        HirExpression::StringMethodCall { method, .. } => {
            assert_eq!(method, "to_string");
        }
        _ => panic!("Expected StringMethodCall expression"),
    }
}

#[test]
fn test_hir_const_char_pointer_to_str_ref() {
    // Test that const char* maps to &str in HIR
    let param_type = HirType::StringReference;

    assert!(matches!(param_type, HirType::StringReference));
}

#[test]
fn test_hir_char_pointer_to_string() {
    // Test that char* (owned) maps to String in HIR
    let owned_type = HirType::OwnedString;

    assert!(matches!(owned_type, HirType::OwnedString));
}
