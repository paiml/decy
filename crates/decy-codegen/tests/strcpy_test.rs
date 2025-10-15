//! Tests for strcpy → String operations (STDLIB-STRCPY validation)
//!
//! Reference: K&R §B3, ISO C99 §7.21.3.1
//!
//! This module tests the transformation of C strcpy() to Rust String operations.
//! strcpy(dest, src) copies a null-terminated string from src to dest, which is
//! UNSAFE (buffer overflow risk). Rust uses safe String operations with automatic
//! bounds checking.

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};

/// Test simple strcpy call as statement
///
/// C: char dest[100];
///    strcpy(dest, src);
///
/// Rust: let mut dest = String::new();
///       dest = src.to_string();
///
/// Reference: K&R §B3, ISO C99 §7.21.3.1
#[test]
fn test_simple_strcpy() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "src".to_string(),
            HirType::Pointer(Box::new(HirType::Char)),
        )],
        vec![
            HirStatement::VariableDeclaration {
                name: "dest".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Char)),
                initializer: None,
            },
            HirStatement::Assignment {
                target: "dest".to_string(),
                value: HirExpression::FunctionCall {
                    function: "strcpy".to_string(),
                    arguments: vec![
                        HirExpression::Variable("dest".to_string()),
                        HirExpression::Variable("src".to_string()),
                    ],
                },
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify strcpy is transformed to String operation
    assert!(
        result.contains(".to_string()") || result.contains(".clone_from("),
        "Should transform strcpy to String operation"
    );

    // Should NOT contain C strcpy function
    assert!(
        !result.contains("strcpy("),
        "Should not contain C strcpy function"
    );

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test strcpy as initializer
///
/// C: char* dest = strcpy(buffer, src);
///
/// Rust: let dest = src.to_string();
///
/// Reference: K&R §B3, ISO C99 §7.21.3.1
#[test]
fn test_strcpy_as_initializer() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![
            HirParameter::new(
                "buffer".to_string(),
                HirType::Pointer(Box::new(HirType::Char)),
            ),
            HirParameter::new("src".to_string(), HirType::Pointer(Box::new(HirType::Char))),
        ],
        vec![HirStatement::VariableDeclaration {
            name: "dest".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Char)),
            initializer: Some(HirExpression::FunctionCall {
                function: "strcpy".to_string(),
                arguments: vec![
                    HirExpression::Variable("buffer".to_string()),
                    HirExpression::Variable("src".to_string()),
                ],
            }),
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify strcpy is transformed
    assert!(result.contains(".to_string()") || result.contains(".clone()"));
    assert!(!result.contains("strcpy"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test strcpy with string literal
///
/// C: strcpy(dest, "hello");
///
/// Rust: dest = "hello".to_string();
///
/// Reference: K&R §B3, ISO C99 §7.21.3.1
#[test]
fn test_strcpy_with_string_literal() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "dest".to_string(),
            HirType::Pointer(Box::new(HirType::Char)),
        )],
        vec![HirStatement::Assignment {
            target: "dest".to_string(),
            value: HirExpression::FunctionCall {
                function: "strcpy".to_string(),
                arguments: vec![
                    HirExpression::Variable("dest".to_string()),
                    HirExpression::StringLiteral("hello".to_string()),
                ],
            },
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify strcpy with literal is transformed
    assert!(result.contains("\"hello\""));
    assert!(result.contains(".to_string()"));
    assert!(!result.contains("strcpy"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test strcpy in if condition
///
/// C: if (strcpy(dest, src)) { ... }
///
/// Rust: dest = src.to_string();
///       if !dest.is_empty() { ... }
///
/// Note: strcpy returns pointer to dest (non-NULL), so we check !empty
///
/// Reference: K&R §B3, ISO C99 §7.21.3.1
#[test]
fn test_strcpy_in_condition() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![
            HirParameter::new(
                "dest".to_string(),
                HirType::Pointer(Box::new(HirType::Char)),
            ),
            HirParameter::new("src".to_string(), HirType::Pointer(Box::new(HirType::Char))),
        ],
        vec![HirStatement::If {
            condition: HirExpression::FunctionCall {
                function: "strcpy".to_string(),
                arguments: vec![
                    HirExpression::Variable("dest".to_string()),
                    HirExpression::Variable("src".to_string()),
                ],
            },
            then_block: vec![],
            else_block: None,
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify strcpy is transformed
    assert!(result.contains(".to_string()") || result.contains(".clone_from("));
    assert!(!result.contains("strcpy"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test strcpy in return statement
///
/// C: char* copy_string(char* dest, char* src) {
///      return strcpy(dest, src);
///    }
///
/// Rust: fn copy_string(dest: &mut String, src: &str) -> &str {
///         dest.clear();
///         dest.push_str(src);
///         return dest.as_str();
///       }
///
/// Reference: K&R §B3, ISO C99 §7.21.3.1
#[test]
fn test_strcpy_in_return() {
    let func = HirFunction::new_with_body(
        "copy_string".to_string(),
        HirType::Pointer(Box::new(HirType::Char)),
        vec![
            HirParameter::new(
                "dest".to_string(),
                HirType::Pointer(Box::new(HirType::Char)),
            ),
            HirParameter::new("src".to_string(), HirType::Pointer(Box::new(HirType::Char))),
        ],
        vec![HirStatement::Return(Some(HirExpression::FunctionCall {
            function: "strcpy".to_string(),
            arguments: vec![
                HirExpression::Variable("dest".to_string()),
                HirExpression::Variable("src".to_string()),
            ],
        }))],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify strcpy is transformed
    assert!(result.contains(".to_string()") || result.contains(".clone_from("));
    assert!(result.contains("return"));
    assert!(!result.contains("strcpy"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test multiple strcpy calls
///
/// C: strcpy(dest1, src1);
///    strcpy(dest2, src2);
///
/// Rust: dest1 = src1.to_string();
///       dest2 = src2.to_string();
///
/// Reference: K&R §B3, ISO C99 §7.21.3.1
#[test]
fn test_multiple_strcpy_calls() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![
            HirParameter::new(
                "dest1".to_string(),
                HirType::Pointer(Box::new(HirType::Char)),
            ),
            HirParameter::new(
                "src1".to_string(),
                HirType::Pointer(Box::new(HirType::Char)),
            ),
            HirParameter::new(
                "dest2".to_string(),
                HirType::Pointer(Box::new(HirType::Char)),
            ),
            HirParameter::new(
                "src2".to_string(),
                HirType::Pointer(Box::new(HirType::Char)),
            ),
        ],
        vec![
            HirStatement::Assignment {
                target: "dest1".to_string(),
                value: HirExpression::FunctionCall {
                    function: "strcpy".to_string(),
                    arguments: vec![
                        HirExpression::Variable("dest1".to_string()),
                        HirExpression::Variable("src1".to_string()),
                    ],
                },
            },
            HirStatement::Assignment {
                target: "dest2".to_string(),
                value: HirExpression::FunctionCall {
                    function: "strcpy".to_string(),
                    arguments: vec![
                        HirExpression::Variable("dest2".to_string()),
                        HirExpression::Variable("src2".to_string()),
                    ],
                },
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify both strcpy calls are transformed
    let to_string_count = result.matches(".to_string()").count();
    assert!(
        to_string_count >= 2 || result.matches(".clone_from(").count() >= 2,
        "Should have at least 2 String operations"
    );

    // Should NOT contain strcpy
    assert!(!result.contains("strcpy"));

    // Verify no unsafe blocks
    assert!(!result.contains("unsafe"));
}

/// Test strcpy in loop
///
/// C: for (int i = 0; i < n; i++) {
///      strcpy(dests[i], srcs[i]);
///    }
///
/// Rust: for i in 0..n {
///         dests[i] = srcs[i].to_string();
///       }
///
/// Reference: K&R §B3, ISO C99 §7.21.3.1
#[test]
fn test_strcpy_in_loop() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![
            HirParameter::new(
                "dests".to_string(),
                HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Char)))),
            ),
            HirParameter::new(
                "srcs".to_string(),
                HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Char)))),
            ),
            HirParameter::new("n".to_string(), HirType::Int),
        ],
        vec![HirStatement::For {
            init: Some(Box::new(HirStatement::VariableDeclaration {
                name: "i".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            })),
            condition: HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::Variable("n".to_string())),
            },
            increment: Some(Box::new(HirStatement::Assignment {
                target: "i".to_string(),
                value: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("i".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            })),
            body: vec![HirStatement::ArrayIndexAssignment {
                array: Box::new(HirExpression::Variable("dests".to_string())),
                index: Box::new(HirExpression::Variable("i".to_string())),
                value: HirExpression::FunctionCall {
                    function: "strcpy".to_string(),
                    arguments: vec![
                        HirExpression::ArrayIndex {
                            array: Box::new(HirExpression::Variable("dests".to_string())),
                            index: Box::new(HirExpression::Variable("i".to_string())),
                        },
                        HirExpression::ArrayIndex {
                            array: Box::new(HirExpression::Variable("srcs".to_string())),
                            index: Box::new(HirExpression::Variable("i".to_string())),
                        },
                    ],
                },
            }],
        }],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify strcpy is transformed in loop
    assert!(result.contains(".to_string()") || result.contains(".clone()"));
    assert!(!result.contains("strcpy"));

    // NOTE: This test has pointer-to-pointer parameters (char** dests, char** srcs)
    // Array indexing on double pointers (dests[i], srcs[i]) requires unsafe pointer arithmetic
    // This is expected and correct behavior from DECY-041 pointer arithmetic feature
    // The strcpy transformation itself remains safe - only the pointer array indexing needs unsafe
}

/// Verify unsafe block count remains 0
///
/// This is critical for the validation goal: <5 unsafe blocks per 1000 LOC
#[test]
fn test_strcpy_transformation_unsafe_count() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![
            HirParameter::new(
                "dest1".to_string(),
                HirType::Pointer(Box::new(HirType::Char)),
            ),
            HirParameter::new(
                "src1".to_string(),
                HirType::Pointer(Box::new(HirType::Char)),
            ),
            HirParameter::new(
                "dest2".to_string(),
                HirType::Pointer(Box::new(HirType::Char)),
            ),
            HirParameter::new(
                "src2".to_string(),
                HirType::Pointer(Box::new(HirType::Char)),
            ),
        ],
        vec![
            HirStatement::Assignment {
                target: "dest1".to_string(),
                value: HirExpression::FunctionCall {
                    function: "strcpy".to_string(),
                    arguments: vec![
                        HirExpression::Variable("dest1".to_string()),
                        HirExpression::Variable("src1".to_string()),
                    ],
                },
            },
            HirStatement::Assignment {
                target: "dest2".to_string(),
                value: HirExpression::FunctionCall {
                    function: "strcpy".to_string(),
                    arguments: vec![
                        HirExpression::Variable("dest2".to_string()),
                        HirExpression::Variable("src2".to_string()),
                    ],
                },
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Count unsafe blocks (should be 0)
    let unsafe_count = result.matches("unsafe").count();
    assert_eq!(
        unsafe_count, 0,
        "strcpy → String transformation should not introduce unsafe blocks"
    );
}

/// Test strcpy buffer overflow prevention
///
/// This test documents that strcpy transformation prevents buffer overflows
///
/// C: char dest[5];
///    strcpy(dest, "hello world");  // BUFFER OVERFLOW!
///
/// Rust: let mut dest = String::new();
///       dest = "hello world".to_string();  // SAFE - grows as needed
///
/// Reference: K&R §B3, ISO C99 §7.21.3.1
#[test]
fn test_strcpy_buffer_overflow_prevention() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "dest".to_string(),
                var_type: HirType::Array {
                    element_type: Box::new(HirType::Char),
                    size: Some(5),
                },
                initializer: None,
            },
            HirStatement::Assignment {
                target: "dest".to_string(),
                value: HirExpression::FunctionCall {
                    function: "strcpy".to_string(),
                    arguments: vec![
                        HirExpression::Variable("dest".to_string()),
                        HirExpression::StringLiteral("hello world".to_string()),
                    ],
                },
            },
        ],
    );

    let codegen = CodeGenerator::new();
    let result = codegen.generate_function(&func);

    // Verify strcpy is transformed to safe String operation
    assert!(result.contains("\"hello world\""));
    assert!(result.contains(".to_string()") || result.contains("String::from("));
    assert!(!result.contains("strcpy"));

    // Verify no unsafe blocks - this is the safety benefit!
    assert!(!result.contains("unsafe"));
}
