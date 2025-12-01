//! DECY-169: Tests for malloc-to-Vec type consistency
//!
//! When malloc is transformed to Vec, the type annotation and expression
//! must both be Vec, not a mismatch of Vec type with raw pointer expression.
//!
//! Bug: `let mut result: Vec<u8> = Box::leak(...).as_mut_ptr();`
//! Fix: `let mut result: Vec<u8> = Vec::with_capacity(...);`

use decy_codegen::CodeGenerator;
use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType};

/// Create a code generator
fn create_generator() -> CodeGenerator {
    CodeGenerator::new()
}

#[test]
fn test_malloc_simple_size_generates_vec_expression() {
    // C code:
    // char* result = (char*)malloc(sb->length + 1);
    //
    // Expected Rust: let mut result: Vec<u8> = Vec::<u8>::with_capacity(... as usize);
    // NOT: let mut result: Vec<u8> = Box::leak(...).as_mut_ptr();
    //
    // The type annotation (Vec<u8>) must match the expression type (Vec)

    let gen = create_generator();

    // Simulate: char* result = malloc(size);
    let func = HirFunction::new_with_body(
        "test_func".to_string(),
        HirType::Pointer(Box::new(HirType::Char)),
        vec![HirParameter::new("size".to_string(), HirType::Int)],
        vec![
            // char* result = malloc(size);
            HirStatement::VariableDeclaration {
                name: "result".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Char)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::Variable("size".to_string())],
                }),
            },
            HirStatement::Return(Some(HirExpression::Variable("result".to_string()))),
        ],
    );

    let code = gen.generate_function(&func);

    println!("Generated: {}", code);

    // The generated code should NOT have Vec type with raw pointer expression
    assert!(
        !code.contains("Vec<u8> = Box::leak"),
        "Should not have type mismatch Vec = raw_ptr: {}",
        code
    );

    // Should either be:
    // 1. Vec<u8> = Vec::with_capacity(...) - type matches
    // 2. *mut u8 = Box::leak(...) - type matches
    // Both are valid, but type must match expression
    let has_vec_with_vec = code.contains("Vec<u8>") && code.contains("Vec::") && !code.contains("as_mut_ptr");
    let has_ptr_with_ptr = code.contains("*mut u8") && code.contains("as_mut_ptr");

    assert!(
        has_vec_with_vec || has_ptr_with_ptr,
        "Type and expression must match. Got: {}",
        code
    );
}

#[test]
fn test_malloc_array_pattern_generates_vec() {
    // C code:
    // int* arr = (int*)malloc(count * sizeof(int));
    //
    // Expected: let mut arr: Vec<i32> = vec![0i32; count as usize];

    let gen = create_generator();

    let func = HirFunction::new_with_body(
        "test_func".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new("count".to_string(), HirType::Int)],
        vec![
            // int* arr = malloc(count * sizeof(int));
            HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::BinaryOp {
                        op: BinaryOperator::Multiply,
                        left: Box::new(HirExpression::Variable("count".to_string())),
                        right: Box::new(HirExpression::Sizeof {
                            type_name: "int".to_string(),
                        }),
                    }],
                }),
            },
            HirStatement::Return(Some(HirExpression::Variable("arr".to_string()))),
        ],
    );

    let code = gen.generate_function(&func);

    println!("Generated: {}", code);

    // Should generate Vec allocation with matching types
    assert!(
        !code.contains("Vec<i32> = Box::leak"),
        "Should not have type mismatch: {}",
        code
    );

    // Should have vec! macro for array pattern
    if code.contains("Vec<i32>") {
        assert!(
            code.contains("vec!["),
            "Vec type should have vec! expression: {}",
            code
        );
    }
}
