//! DECY-165: Tests for struct pointer field indexing
//!
//! When a struct has a pointer field (e.g., `char* data`), indexing that field
//! like `sb->data[i]` should generate safe pointer arithmetic:
//! `unsafe { *(*sb).data.add(i as usize) }`
//!
//! NOT the invalid: `(*sb).data[(i) as usize]` (can't index raw pointers with [])

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};

/// Helper to create a code generator
fn create_generator() -> CodeGenerator {
    CodeGenerator::new()
}

#[test]
fn test_struct_pointer_field_indexing_uses_unsafe_add() {
    // C code:
    // typedef struct { char* data; } StringBuilder;
    // void test(StringBuilder* sb) { sb->data[0] = 'a'; }
    //
    // Expected Rust:
    // unsafe { *(*sb).data.add(0) } = b'a';

    // Create function: void test(StringBuilder* sb) { sb->data[0] = 'a'; }
    let test_fn = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "sb".to_string(),
            HirType::Pointer(Box::new(HirType::Struct("StringBuilder".to_string()))),
        )],
        vec![
            // sb->data[0] = 'a';
            HirStatement::ArrayIndexAssignment {
                array: Box::new(HirExpression::PointerFieldAccess {
                    pointer: Box::new(HirExpression::Variable("sb".to_string())),
                    field: "data".to_string(),
                }),
                index: Box::new(HirExpression::IntLiteral(0)),
                value: HirExpression::CharLiteral(b'a' as i8),
            },
        ],
    );

    let gen = create_generator();
    let rust_code = gen.generate_function(&test_fn);

    println!("Generated code:\n{}", rust_code);

    // The generated code should contain unsafe pointer arithmetic, NOT array indexing
    // Should use .add() or .wrapping_add() for pointer arithmetic
    assert!(
        rust_code.contains(".add(") || rust_code.contains(".wrapping_add("),
        "Struct pointer field indexing should use .add() or .wrapping_add(), got:\n{}",
        rust_code
    );

    // Should NOT use array indexing syntax on raw pointer
    assert!(
        !rust_code.contains(".data[(0)"),
        "Should not use array index syntax on raw pointer, got:\n{}",
        rust_code
    );

    // Should be wrapped in unsafe
    assert!(
        rust_code.contains("unsafe"),
        "Raw pointer access should be wrapped in unsafe, got:\n{}",
        rust_code
    );
}

#[test]
fn test_struct_pointer_field_read_uses_unsafe_add() {
    // C code:
    // typedef struct { char* data; size_t length; } StringBuilder;
    // char test(StringBuilder* sb) { return sb->data[sb->length - 1]; }
    //
    // Expected Rust: reads from pointer should also use unsafe .add()

    // Create function that reads from the pointer field with an index
    let test_fn = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Char,
        vec![HirParameter::new(
            "sb".to_string(),
            HirType::Pointer(Box::new(HirType::Struct("StringBuilder".to_string()))),
        )],
        vec![
            // return sb->data[sb->length - 1];
            HirStatement::Return(Some(HirExpression::ArrayIndex {
                array: Box::new(HirExpression::PointerFieldAccess {
                    pointer: Box::new(HirExpression::Variable("sb".to_string())),
                    field: "data".to_string(),
                }),
                index: Box::new(HirExpression::BinaryOp {
                    left: Box::new(HirExpression::PointerFieldAccess {
                        pointer: Box::new(HirExpression::Variable("sb".to_string())),
                        field: "length".to_string(),
                    }),
                    op: decy_hir::BinaryOperator::Subtract,
                    right: Box::new(HirExpression::IntLiteral(1)),
                }),
            })),
        ],
    );

    let gen = create_generator();
    let rust_code = gen.generate_function(&test_fn);

    println!("Generated code:\n{}", rust_code);

    // Should use pointer arithmetic
    assert!(
        rust_code.contains(".add(") || rust_code.contains(".wrapping_add("),
        "Struct pointer field indexing should use pointer arithmetic, got:\n{}",
        rust_code
    );

    // Should be wrapped in unsafe
    assert!(
        rust_code.contains("unsafe"),
        "Raw pointer access should be wrapped in unsafe, got:\n{}",
        rust_code
    );
}

#[test]
fn test_nested_pointer_indexing() {
    // C code:
    // typedef struct { int** matrix; } Matrix;
    // int test(Matrix* m) { return m->matrix[0][1]; }
    //
    // Both indexing levels should use unsafe pointer arithmetic

    // Create function that does double indexing
    let test_fn = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "m".to_string(),
            HirType::Pointer(Box::new(HirType::Struct("Matrix".to_string()))),
        )],
        vec![
            // return m->matrix[0][1];
            HirStatement::Return(Some(HirExpression::ArrayIndex {
                array: Box::new(HirExpression::ArrayIndex {
                    array: Box::new(HirExpression::PointerFieldAccess {
                        pointer: Box::new(HirExpression::Variable("m".to_string())),
                        field: "matrix".to_string(),
                    }),
                    index: Box::new(HirExpression::IntLiteral(0)),
                }),
                index: Box::new(HirExpression::IntLiteral(1)),
            })),
        ],
    );

    let gen = create_generator();
    let rust_code = gen.generate_function(&test_fn);

    println!("Generated code:\n{}", rust_code);

    // Should use pointer arithmetic (at least once, ideally twice)
    assert!(
        rust_code.contains(".add(") || rust_code.contains(".wrapping_add("),
        "Nested pointer indexing should use pointer arithmetic, got:\n{}",
        rust_code
    );

    // Should be wrapped in unsafe
    assert!(
        rust_code.contains("unsafe"),
        "Raw pointer access should be wrapped in unsafe, got:\n{}",
        rust_code
    );
}
