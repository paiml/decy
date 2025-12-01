//! DECY-170: Tests for correct 'as usize' precedence in Vec allocation
//!
//! When generating Vec::with_capacity(size as usize), the size expression
//! must be wrapped in parentheses if it's a binary operation:
//!
//! Bug: Vec::with_capacity(x + 1 as usize) - parsed as x + (1 as usize)
//! Fix: Vec::with_capacity((x + 1) as usize) - correct precedence

use decy_codegen::CodeGenerator;
use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType};

/// Create a code generator
fn create_generator() -> CodeGenerator {
    CodeGenerator::new()
}

#[test]
fn test_malloc_addition_size_has_correct_precedence() {
    // C code:
    // char* result = (char*)malloc(len + 1);
    //
    // Expected Rust: Vec::<u8>::with_capacity((len + 1) as usize);
    // NOT: Vec::<u8>::with_capacity(len + 1 as usize);
    //
    // The latter would parse as len + (1 as usize) causing u32 + usize error

    let gen = create_generator();

    // Simulate: char* result = malloc(len + 1);
    let func = HirFunction::new_with_body(
        "test_func".to_string(),
        HirType::Pointer(Box::new(HirType::Char)),
        vec![HirParameter::new("len".to_string(), HirType::Int)],
        vec![
            // char* result = malloc(len + 1);
            HirStatement::VariableDeclaration {
                name: "result".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Char)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::BinaryOp {
                        op: BinaryOperator::Add,
                        left: Box::new(HirExpression::Variable("len".to_string())),
                        right: Box::new(HirExpression::IntLiteral(1)),
                    }],
                }),
            },
            HirStatement::Return(Some(HirExpression::Variable("result".to_string()))),
        ],
    );

    let code = gen.generate_function(&func);

    println!("Generated: {}", code);

    // The size expression must be wrapped in parentheses
    // Check for correct precedence pattern: (expression) as usize
    assert!(
        code.contains("((len + 1)) as usize") || code.contains("((len + 1) as usize)"),
        "Addition in malloc size should be wrapped in parens for correct 'as' precedence. Got: {}",
        code
    );

    // Should NOT have the incorrect precedence pattern
    assert!(
        !code.contains("len + 1 as usize"),
        "Should NOT have 'x + 1 as usize' (wrong precedence). Got: {}",
        code
    );
}

#[test]
fn test_malloc_subtraction_size_has_correct_precedence() {
    // C code:
    // char* buf = (char*)malloc(size - offset);
    //
    // Expected: with_capacity((size - offset) as usize)
    // NOT: with_capacity(size - offset as usize)

    let gen = create_generator();

    let func = HirFunction::new_with_body(
        "test_func".to_string(),
        HirType::Pointer(Box::new(HirType::Char)),
        vec![
            HirParameter::new("size".to_string(), HirType::Int),
            HirParameter::new("offset".to_string(), HirType::Int),
        ],
        vec![
            HirStatement::VariableDeclaration {
                name: "buf".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Char)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::BinaryOp {
                        op: BinaryOperator::Subtract,
                        left: Box::new(HirExpression::Variable("size".to_string())),
                        right: Box::new(HirExpression::Variable("offset".to_string())),
                    }],
                }),
            },
            HirStatement::Return(Some(HirExpression::Variable("buf".to_string()))),
        ],
    );

    let code = gen.generate_function(&func);

    println!("Generated: {}", code);

    // Check for correct precedence
    assert!(
        code.contains("((size - offset)) as usize") || code.contains("((size - offset) as usize)"),
        "Subtraction in malloc size should be wrapped in parens. Got: {}",
        code
    );
}
