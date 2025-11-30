//! DECY-090: Read/Write trait usage tests.
//!
//! Tests for generating Rust Read/Write trait usage from C file operations.

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirStatement, HirType};

/// Test fread() → .read() method.
/// C: fread(buf, size, count, file);
/// Rust: file.read(&mut buf)?; (uses Read trait)
#[test]
fn test_fread_generates_read_trait() {
    let func = HirFunction::new_with_body(
        "read_data".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "fread".to_string(),
            arguments: vec![
                HirExpression::Variable("buf".to_string()),
                HirExpression::IntLiteral(1),
                HirExpression::IntLiteral(100),
                HirExpression::Variable("file".to_string()),
            ],
        })],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should use Read trait
    assert!(
        code.contains("Read") || code.contains(".read("),
        "Expected Read trait or .read() in:\n{}",
        code
    );
}

/// Test fwrite() → .write() method.
/// C: fwrite(data, size, count, file);
/// Rust: file.write(&data)?; (uses Write trait)
#[test]
fn test_fwrite_generates_write_trait() {
    let func = HirFunction::new_with_body(
        "write_data".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "fwrite".to_string(),
            arguments: vec![
                HirExpression::Variable("data".to_string()),
                HirExpression::IntLiteral(1),
                HirExpression::IntLiteral(100),
                HirExpression::Variable("file".to_string()),
            ],
        })],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should use Write trait or write method
    assert!(
        code.contains("Write") || code.contains(".write("),
        "Expected Write trait or .write() in:\n{}",
        code
    );
}

/// Test fputc() → .write() with single byte.
/// C: fputc(c, file);
/// Rust: file.write(&[c as u8])?;
#[test]
fn test_fputc_generates_write() {
    let func = HirFunction::new_with_body(
        "write_char".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "fputc".to_string(),
            arguments: vec![
                HirExpression::Variable("c".to_string()),
                HirExpression::Variable("file".to_string()),
            ],
        })],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should use Write trait
    assert!(
        code.contains("Write") || code.contains(".write("),
        "Expected Write trait or .write() in:\n{}",
        code
    );
}

/// Test fgetc() → .read() with single byte.
/// C: int c = fgetc(file);
/// Rust: let mut buf = [0u8; 1]; file.read(&mut buf)?;
#[test]
fn test_fgetc_generates_read() {
    let func = HirFunction::new_with_body(
        "read_char".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "c".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::FunctionCall {
                function: "fgetc".to_string(),
                arguments: vec![HirExpression::Variable("file".to_string())],
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should use Read trait
    assert!(
        code.contains("Read") || code.contains(".read("),
        "Expected Read trait or .read() in:\n{}",
        code
    );
}

/// Test fputs() → .write_all() for strings.
/// C: fputs(str, file);
/// Rust: file.write_all(str.as_bytes())?;
#[test]
fn test_fputs_generates_write_all() {
    let func = HirFunction::new_with_body(
        "write_string".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "fputs".to_string(),
            arguments: vec![
                HirExpression::Variable("str".to_string()),
                HirExpression::Variable("file".to_string()),
            ],
        })],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should use Write trait (write_all or write)
    assert!(
        code.contains("Write") || code.contains(".write"),
        "Expected Write trait or .write in:\n{}",
        code
    );
}

/// Test getc() → .read() (alias for fgetc).
/// C: int c = getc(file);
/// Rust: Same as fgetc
#[test]
fn test_getc_generates_read() {
    let func = HirFunction::new_with_body(
        "read_getc".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "c".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::FunctionCall {
                function: "getc".to_string(),
                arguments: vec![HirExpression::Variable("file".to_string())],
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should use Read trait
    assert!(
        code.contains("Read") || code.contains(".read("),
        "Expected Read trait or .read() in:\n{}",
        code
    );
}

/// Test putc() → .write() (alias for fputc).
/// C: putc(c, file);
/// Rust: Same as fputc
#[test]
fn test_putc_generates_write() {
    let func = HirFunction::new_with_body(
        "write_putc".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "putc".to_string(),
            arguments: vec![
                HirExpression::Variable("c".to_string()),
                HirExpression::Variable("file".to_string()),
            ],
        })],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should use Write trait
    assert!(
        code.contains("Write") || code.contains(".write("),
        "Expected Write trait or .write() in:\n{}",
        code
    );
}
