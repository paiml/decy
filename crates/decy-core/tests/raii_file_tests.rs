//! DECY-091: RAII file handling verification tests.
//!
//! Tests verifying files are automatically closed via RAII without explicit close calls.

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirStatement, HirType};

/// Test fclose is not required - generates RAII comment or drop.
/// C: fclose(f);
/// Rust: Either drop(f) or RAII comment (no fclose call)
#[test]
fn test_fclose_generates_raii_not_fclose() {
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

    // Should NOT have fclose in the body
    let body = code.split('{').nth(1).unwrap_or("");
    assert!(
        !body.contains("fclose("),
        "Should not generate fclose() call in:\n{}",
        code
    );
    // Should have RAII or drop
    assert!(
        code.contains("drop(") || code.contains("RAII"),
        "Expected drop() or RAII comment in:\n{}",
        code
    );
}

/// Test free() is transformed to RAII comment (memory cleanup).
/// C: free(ptr);
/// Rust: // Memory deallocated by RAII
#[test]
fn test_free_generates_raii_comment() {
    let func = HirFunction::new_with_body(
        "free_memory".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Free {
            pointer: HirExpression::Variable("ptr".to_string()),
        }],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should have RAII comment
    assert!(
        code.contains("RAII") || code.contains("deallocated"),
        "Expected RAII comment in:\n{}",
        code
    );
    // Should NOT have free() call
    let body = code.split('{').nth(1).unwrap_or("");
    assert!(
        !body.contains("free("),
        "Should not generate free() call in:\n{}",
        code
    );
}

/// Test generated File uses Option for error handling (RAII-safe).
/// C: FILE* f = fopen("test", "r");
/// Rust: let f = File::open("test").ok(); // Option<File> - auto-drops
#[test]
fn test_fopen_returns_option_for_raii() {
    let func = HirFunction::new_with_body(
        "open_file".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "f".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Void)),
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

    // Should use File::open or File::create
    assert!(
        code.contains("File::open") || code.contains("File::create"),
        "Expected File::open or File::create in:\n{}",
        code
    );
    // Should return Option (using .ok())
    assert!(
        code.contains(".ok()"),
        "Expected .ok() for Option-based RAII in:\n{}",
        code
    );
}

/// Test Box uses RAII - no explicit deallocation needed.
/// C: struct Node* n = malloc(sizeof(Node)); ... free(n);
/// Rust: let n: Box<Node> = Box::new(...); // auto-drops
#[test]
fn test_box_allocation_uses_raii() {
    let func = HirFunction::new_with_body(
        "allocate_node".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "n".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::Sizeof {
                        type_name: "struct Node".to_string(),
                    }],
                }),
            },
            HirStatement::Free {
                pointer: HirExpression::Variable("n".to_string()),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should use Box type
    assert!(
        code.contains("Box<Node>"),
        "Expected Box<Node> type in:\n{}",
        code
    );
    // Should have RAII comment instead of explicit deallocation
    assert!(
        code.contains("RAII"),
        "Expected RAII comment for memory cleanup in:\n{}",
        code
    );
}

/// Test Vec allocation uses RAII.
/// C: int* arr = malloc(n * sizeof(int)); ... free(arr);
/// Rust: let arr: Vec<i32> = ...; // auto-drops
#[test]
fn test_vec_allocation_uses_raii() {
    let func = HirFunction::new_with_body(
        "allocate_array".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::BinaryOp {
                        op: decy_hir::BinaryOperator::Multiply,
                        left: Box::new(HirExpression::Variable("n".to_string())),
                        right: Box::new(HirExpression::Sizeof {
                            type_name: "int".to_string(),
                        }),
                    }],
                }),
            },
            HirStatement::Free {
                pointer: HirExpression::Variable("arr".to_string()),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should use Vec type
    assert!(
        code.contains("Vec<i32>"),
        "Expected Vec<i32> type in:\n{}",
        code
    );
    // Should have RAII comment
    assert!(code.contains("RAII"), "Expected RAII comment in:\n{}", code);
}

/// Test no explicit fclose needed after file operations.
/// Full workflow: open, read, close → open, read, auto-drop
#[test]
fn test_complete_file_workflow_raii() {
    let func = HirFunction::new_with_body(
        "read_file_workflow".to_string(),
        HirType::Void,
        vec![],
        vec![
            // FILE* f = fopen("test", "r");
            HirStatement::VariableDeclaration {
                name: "f".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Void)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "fopen".to_string(),
                    arguments: vec![
                        HirExpression::StringLiteral("test.txt".to_string()),
                        HirExpression::StringLiteral("r".to_string()),
                    ],
                }),
            },
            // int c = fgetc(f);
            HirStatement::VariableDeclaration {
                name: "c".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::FunctionCall {
                    function: "fgetc".to_string(),
                    arguments: vec![HirExpression::Variable("f".to_string())],
                }),
            },
            // fclose(f);
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "fclose".to_string(),
                arguments: vec![HirExpression::Variable("f".to_string())],
            }),
        ],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Verify complete RAII workflow
    assert!(
        code.contains("File::open"),
        "Expected File::open in:\n{}",
        code
    );
    // No fclose in body
    let body = code.split('{').nth(1).unwrap_or("");
    assert!(
        !body.contains("fclose("),
        "Should not have fclose() in body:\n{}",
        code
    );
}
