//! DECY-089: FILE* to File transformation tests.
//!
//! Tests for transforming C file operations to Rust std::fs::File.

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirStatement, HirType};

/// Test fopen(filename, "r") → File::open().
/// C: FILE* f = fopen("test.txt", "r");
/// Rust: let f = std::fs::File::open("test.txt").ok();
#[test]
fn test_fopen_read_to_file_open() {
    let func = HirFunction::new_with_body(
        "read_file".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "f".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Void)), // FILE* is opaque
            initializer: Some(HirExpression::FunctionCall {
                function: "fopen".to_string(),
                arguments: vec![
                    HirExpression::StringLiteral("test.txt".to_string()),
                    HirExpression::StringLiteral("r".to_string()),
                ],
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should use File::open for read mode
    assert!(
        code.contains("File::open") || code.contains("std::fs::File::open"),
        "Expected File::open in:\n{}",
        code
    );
}

/// Test fopen(filename, "w") → File::create().
/// C: FILE* f = fopen("output.txt", "w");
/// Rust: let f = std::fs::File::create("output.txt").ok();
#[test]
fn test_fopen_write_to_file_create() {
    let func = HirFunction::new_with_body(
        "write_file".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "f".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Void)),
            initializer: Some(HirExpression::FunctionCall {
                function: "fopen".to_string(),
                arguments: vec![
                    HirExpression::StringLiteral("output.txt".to_string()),
                    HirExpression::StringLiteral("w".to_string()),
                ],
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should use File::create for write mode
    assert!(
        code.contains("File::create") || code.contains("std::fs::File::create"),
        "Expected File::create in:\n{}",
        code
    );
}

/// Test fclose(f) → drop(f) or RAII comment.
/// C: fclose(f);
/// Rust: drop(f); or // File closed by RAII
#[test]
fn test_fclose_to_drop_or_raii() {
    let func = HirFunction::new_with_body(
        "close_file".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "fclose".to_string(),
            arguments: vec![HirExpression::Variable("f".to_string())],
        })],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should have drop() or RAII
    assert!(
        code.contains("drop(") || code.contains("RAII"),
        "Expected drop() or RAII in:\n{}",
        code
    );
    // Should NOT have raw fclose() call in body
    let body_only = code.split('{').nth(1).unwrap_or("");
    assert!(
        !body_only.contains("fclose("),
        "Should not have fclose() call in body:\n{}",
        code
    );
}

/// Test fgetc(f) → file read byte.
/// C: int c = fgetc(f);
/// Rust: Uses Read trait
#[test]
fn test_fgetc_to_read() {
    let func = HirFunction::new_with_body(
        "read_char".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "c".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::FunctionCall {
                function: "fgetc".to_string(),
                arguments: vec![HirExpression::Variable("f".to_string())],
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should use Read trait or read method
    assert!(
        code.contains("Read") || code.contains(".read("),
        "Expected Read trait or .read() in:\n{}",
        code
    );
}

/// Test fprintf(f, fmt, ...) → write! macro.
/// C: fprintf(f, "Hello %s", name);
/// Rust: write!(f, ...) or writeln!
#[test]
fn test_fprintf_to_write_macro() {
    let func = HirFunction::new_with_body(
        "print_to_file".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "fprintf".to_string(),
            arguments: vec![
                HirExpression::Variable("f".to_string()),
                HirExpression::StringLiteral("Hello %s".to_string()),
                HirExpression::Variable("name".to_string()),
            ],
        })],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should use write! macro or Write trait
    assert!(
        code.contains("write!") || code.contains("Write"),
        "Expected write! macro or Write trait in:\n{}",
        code
    );
}

/// Test printf(fmt, ...) → print! macro.
/// C: printf("Hello %s", name);
/// Rust: print!("Hello {}", name);
#[test]
fn test_printf_to_print_macro() {
    let func = HirFunction::new_with_body(
        "print_message".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "printf".to_string(),
            arguments: vec![
                HirExpression::StringLiteral("Hello %s".to_string()),
                HirExpression::Variable("name".to_string()),
            ],
        })],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should use print! macro
    assert!(
        code.contains("print!"),
        "Expected print! macro in:\n{}",
        code
    );
}
