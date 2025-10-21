//! Unit tests for HirMacroDefinition
//!
//! Reference: K&R §4.11, ISO C99 §6.10.3

use crate::HirMacroDefinition;

#[test]
fn test_object_like_macro_creation() {
    // #define MAX 100
    let macro_def = HirMacroDefinition::new_object_like("MAX".to_string(), "100".to_string());

    assert_eq!(macro_def.name(), "MAX");
    assert_eq!(macro_def.body(), "100");
    assert!(macro_def.is_object_like());
    assert!(!macro_def.is_function_like());
    assert!(macro_def.parameters().is_empty());
}

#[test]
fn test_function_like_macro_creation_single_param() {
    // #define SQR(x) ((x) * (x))
    let macro_def = HirMacroDefinition::new_function_like(
        "SQR".to_string(),
        vec!["x".to_string()],
        "((x) * (x))".to_string(),
    );

    assert_eq!(macro_def.name(), "SQR");
    assert_eq!(macro_def.body(), "((x) * (x))");
    assert!(macro_def.is_function_like());
    assert!(!macro_def.is_object_like());
    assert_eq!(macro_def.parameters(), &["x"]);
}

#[test]
fn test_function_like_macro_creation_multiple_params() {
    // #define MAX(a, b) ((a) > (b) ? (a) : (b))
    let macro_def = HirMacroDefinition::new_function_like(
        "MAX".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "((a) > (b) ? (a) : (b))".to_string(),
    );

    assert_eq!(macro_def.name(), "MAX");
    assert_eq!(macro_def.body(), "((a) > (b) ? (a) : (b))");
    assert!(macro_def.is_function_like());
    assert!(!macro_def.is_object_like());
    assert_eq!(macro_def.parameters(), &["a", "b"]);
}

#[test]
fn test_macro_with_empty_body() {
    // #define EMPTY
    let macro_def = HirMacroDefinition::new_object_like("EMPTY".to_string(), "".to_string());

    assert_eq!(macro_def.name(), "EMPTY");
    assert_eq!(macro_def.body(), "");
    assert!(macro_def.is_object_like());
}

#[test]
fn test_macro_with_complex_body() {
    // #define SWAP(a, b) { typeof(a) tmp = a; a = b; b = tmp; }
    let body = "{ typeof(a) tmp = a; a = b; b = tmp; }";
    let macro_def = HirMacroDefinition::new_function_like(
        "SWAP".to_string(),
        vec!["a".to_string(), "b".to_string()],
        body.to_string(),
    );

    assert_eq!(macro_def.name(), "SWAP");
    assert_eq!(macro_def.body(), body);
    assert_eq!(macro_def.parameters().len(), 2);
}

#[test]
fn test_macro_clone() {
    let macro_def1 = HirMacroDefinition::new_object_like("PI".to_string(), "3.14".to_string());
    let macro_def2 = macro_def1.clone();

    assert_eq!(macro_def1, macro_def2);
    assert_eq!(macro_def1.name(), macro_def2.name());
    assert_eq!(macro_def1.body(), macro_def2.body());
}

#[test]
fn test_macro_debug_formatting() {
    let macro_def = HirMacroDefinition::new_object_like("DBG".to_string(), "1".to_string());
    let debug_str = format!("{:?}", macro_def);

    assert!(debug_str.contains("HirMacroDefinition"));
    assert!(debug_str.contains("DBG"));
}

#[test]
fn test_macro_equality() {
    let macro_def1 = HirMacroDefinition::new_function_like(
        "MIN".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "((a) < (b) ? (a) : (b))".to_string(),
    );

    let macro_def2 = HirMacroDefinition::new_function_like(
        "MIN".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "((a) < (b) ? (a) : (b))".to_string(),
    );

    let macro_def3 = HirMacroDefinition::new_function_like(
        "MAX".to_string(), // Different name
        vec!["a".to_string(), "b".to_string()],
        "((a) < (b) ? (a) : (b))".to_string(),
    );

    assert_eq!(macro_def1, macro_def2);
    assert_ne!(macro_def1, macro_def3);
}

#[test]
fn test_macro_with_variadic_notation() {
    // Note: This test documents that we store __VA_ARGS__ as a string
    // Actual variadic macro support would require parser changes
    // #define LOG(fmt, ...) printf(fmt, __VA_ARGS__)
    let macro_def = HirMacroDefinition::new_function_like(
        "LOG".to_string(),
        vec!["fmt".to_string(), "...".to_string()],
        "printf(fmt, __VA_ARGS__)".to_string(),
    );

    assert_eq!(macro_def.parameters(), &["fmt", "..."]);
    assert!(macro_def.body().contains("__VA_ARGS__"));
}

#[test]
fn test_macro_with_stringification() {
    // #define STR(x) #x
    let macro_def = HirMacroDefinition::new_function_like(
        "STR".to_string(),
        vec!["x".to_string()],
        "#x".to_string(),
    );

    assert_eq!(macro_def.name(), "STR");
    assert_eq!(macro_def.body(), "#x");
}

#[test]
fn test_macro_with_token_pasting() {
    // #define CONCAT(a, b) a##b
    let macro_def = HirMacroDefinition::new_function_like(
        "CONCAT".to_string(),
        vec!["a".to_string(), "b".to_string()],
        "a##b".to_string(),
    );

    assert_eq!(macro_def.name(), "CONCAT");
    assert_eq!(macro_def.body(), "a##b");
}
