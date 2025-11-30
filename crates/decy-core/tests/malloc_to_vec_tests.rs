//! DECY-087: malloc array to Vec<T> transformation tests.
//!
//! Tests for transforming C heap allocations to Rust Vec types.

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirStatement, HirType};

/// Test malloc(n * sizeof(int)) → Vec::with_capacity or vec![].
/// C: int* arr = malloc(n * sizeof(int));
/// Rust: let mut arr: Vec<i32> = vec![0i32; n as usize];
#[test]
fn test_malloc_array_to_vec() {
    let func = HirFunction::new_with_body(
        "allocate_array".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
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
        }],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should generate Vec type, not raw pointer
    assert!(
        code.contains("Vec<i32>"),
        "Expected Vec<i32> type in:\n{}",
        code
    );
    // Should NOT contain *mut
    assert!(
        !code.contains("*mut i32"),
        "Should not have raw pointer type in:\n{}",
        code
    );
}

/// Test calloc(n, sizeof(int)) → vec![0; n].
/// C: int* arr = calloc(n, sizeof(int));
/// Rust: let mut arr: Vec<i32> = vec![0i32; n as usize];
#[test]
fn test_calloc_to_vec_zeroed() {
    let func = HirFunction::new_with_body(
        "allocate_zeroed".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "calloc".to_string(),
                arguments: vec![
                    HirExpression::Variable("n".to_string()),
                    HirExpression::Sizeof {
                        type_name: "int".to_string(),
                    },
                ],
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should generate Vec type
    assert!(
        code.contains("Vec<i32>"),
        "Expected Vec<i32> type in:\n{}",
        code
    );
    // Should have zero initialization
    assert!(
        code.contains("vec![0i32;") || code.contains("vec![0;"),
        "Expected zero-initialized vec in:\n{}",
        code
    );
}

/// Test free(arr) → drop(arr) or RAII comment.
/// C: free(arr);
/// Rust: drop(arr); or // Memory deallocated by RAII
#[test]
fn test_free_to_drop_or_raii() {
    let func = HirFunction::new_with_body(
        "free_array".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Free {
            pointer: HirExpression::Variable("arr".to_string()),
        }],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should have RAII comment or drop()
    assert!(
        code.contains("RAII") || code.contains("drop("),
        "Expected RAII comment or drop() in:\n{}",
        code
    );
    // Should NOT have raw free() call
    assert!(
        !code.contains("free(arr)"),
        "Should not have raw free() call in:\n{}",
        code
    );
}

/// Test char* buffer malloc → Vec<u8>.
/// C: char* buf = malloc(100);
/// Rust: let mut buf: Vec<u8> = Vec::with_capacity(100);
#[test]
fn test_malloc_char_buffer_to_vec_u8() {
    let func = HirFunction::new_with_body(
        "allocate_buffer".to_string(),
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

    // Should generate Vec<u8> for char buffer
    assert!(
        code.contains("Vec<u8>") || code.contains("Vec::<u8>"),
        "Expected Vec<u8> type in:\n{}",
        code
    );
}

/// Test malloc for struct → Box<T> (not Vec).
/// C: struct Node* node = malloc(sizeof(struct Node));
/// Rust: let mut node: Box<Node> = Box::new(Node::default());
#[test]
fn test_malloc_struct_to_box() {
    let func = HirFunction::new_with_body(
        "allocate_node".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "node".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::Sizeof {
                    type_name: "struct Node".to_string(),
                }],
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should generate Box<Node> for single struct allocation
    assert!(
        code.contains("Box<Node>"),
        "Expected Box<Node> type in:\n{}",
        code
    );
    // Should use Box::new
    assert!(
        code.contains("Box::new"),
        "Expected Box::new() in:\n{}",
        code
    );
}

/// Test realloc → Vec::resize.
/// C: arr = realloc(arr, new_size * sizeof(int));
/// Rust: arr.resize(new_size, 0);
#[test]
fn test_realloc_to_vec_resize() {
    let func = HirFunction::new_with_body(
        "resize_array".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Assignment {
            target: "arr".to_string(),
            value: HirExpression::Realloc {
                pointer: Box::new(HirExpression::Variable("arr".to_string())),
                new_size: Box::new(HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Multiply,
                    left: Box::new(HirExpression::Variable("new_count".to_string())),
                    right: Box::new(HirExpression::Sizeof {
                        type_name: "int".to_string(),
                    }),
                }),
            },
        }],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should use resize method
    assert!(
        code.contains(".resize("),
        "Expected .resize() method in:\n{}",
        code
    );
}

/// Test array indexing on Vec remains safe.
/// C: arr[i] = 42;
/// Rust: arr[i as usize] = 42;
#[test]
fn test_vec_indexing_remains_safe() {
    let func = HirFunction::new_with_body(
        "index_array".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Vec(Box::new(HirType::Int)),
                initializer: None,
            },
            HirStatement::ArrayIndexAssignment {
                array: Box::new(HirExpression::Variable("arr".to_string())),
                index: Box::new(HirExpression::Variable("i".to_string())),
                value: HirExpression::IntLiteral(42),
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let code = codegen.generate_function(&func);

    // Should cast index to usize
    assert!(
        code.contains("as usize]"),
        "Expected usize cast for array index in:\n{}",
        code
    );
    // Should NOT have unsafe
    assert!(
        !code.contains("unsafe"),
        "Vec indexing should not need unsafe in:\n{}",
        code
    );
}
