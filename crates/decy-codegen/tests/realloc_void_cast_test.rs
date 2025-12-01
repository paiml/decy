//! DECY-171: Tests for void* casting in realloc calls
//!
//! When calling realloc with a typed pointer (*mut u8), we need to:
//! 1. Cast the argument to *mut ()
//! 2. Cast the return value to the target type
//!
//! C: sb->data = (char*)realloc(sb->data, sb->capacity);
//! Rust: sb.data = realloc((*sb).data as *mut (), (*sb).capacity) as *mut u8;

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};

/// Create a code generator
fn create_generator() -> CodeGenerator {
    CodeGenerator::new()
}

#[test]
fn test_realloc_casts_typed_pointer_argument() {
    // C code:
    // void resize_buffer(char** data, int new_size) {
    //     *data = (char*)realloc(*data, new_size);
    // }
    //
    // Expected Rust:
    // realloc(*data as *mut (), new_size) as *mut u8
    //
    // The typed pointer (*mut u8) must be cast to *mut () for realloc

    let gen = create_generator();

    let func = HirFunction::new_with_body(
        "resize_buffer".to_string(),
        HirType::Void,
        vec![
            HirParameter::new(
                "data".to_string(),
                HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Char)))),
            ),
            HirParameter::new("new_size".to_string(), HirType::Int),
        ],
        vec![
            // *data = (char*)realloc(*data, new_size);
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("data".to_string()),
                value: HirExpression::Cast {
                    target_type: HirType::Pointer(Box::new(HirType::Char)),
                    expr: Box::new(HirExpression::FunctionCall {
                        function: "realloc".to_string(),
                        arguments: vec![
                            HirExpression::Dereference(Box::new(HirExpression::Variable(
                                "data".to_string(),
                            ))),
                            HirExpression::Variable("new_size".to_string()),
                        ],
                    }),
                },
            },
        ],
    );

    let code = gen.generate_function(&func);

    println!("Generated: {}", code);

    // The first argument should be cast to *mut ()
    assert!(
        code.contains("as *mut ()") || code.contains("as *mut()"),
        "realloc argument should be cast to *mut (). Got: {}",
        code
    );
}

#[test]
fn test_realloc_casts_return_value() {
    // C code:
    // void resize_buffer(char** data, int new_size) {
    //     *data = (char*)realloc(*data, new_size);
    // }
    //
    // Expected: realloc returns *mut (), needs cast to *mut u8

    let gen = create_generator();

    let func = HirFunction::new_with_body(
        "resize_buffer".to_string(),
        HirType::Void,
        vec![
            HirParameter::new(
                "data".to_string(),
                HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Char)))),
            ),
            HirParameter::new("new_size".to_string(), HirType::Int),
        ],
        vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("data".to_string()),
            value: HirExpression::Cast {
                target_type: HirType::Pointer(Box::new(HirType::Char)),
                expr: Box::new(HirExpression::FunctionCall {
                    function: "realloc".to_string(),
                    arguments: vec![
                        HirExpression::Dereference(Box::new(HirExpression::Variable(
                            "data".to_string(),
                        ))),
                        HirExpression::Variable("new_size".to_string()),
                    ],
                }),
            },
        }],
    );

    let code = gen.generate_function(&func);

    println!("Generated: {}", code);

    // Return value should be cast to the target pointer type
    assert!(
        code.contains("as *mut u8"),
        "realloc return should be cast to target type. Got: {}",
        code
    );
}

#[test]
fn test_realloc_without_explicit_cast_gets_cast() {
    // Even when C doesn't have explicit cast, we need them in Rust
    // C: *data = realloc(*data, size);  // implicit cast in C
    // Rust needs explicit casts both ways

    let gen = create_generator();

    let func = HirFunction::new_with_body(
        "resize_buffer".to_string(),
        HirType::Void,
        vec![
            HirParameter::new(
                "data".to_string(),
                HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Char)))),
            ),
            HirParameter::new("new_size".to_string(), HirType::Int),
        ],
        vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("data".to_string()),
            // No explicit cast in HIR - realloc called directly
            value: HirExpression::FunctionCall {
                function: "realloc".to_string(),
                arguments: vec![
                    HirExpression::Dereference(Box::new(HirExpression::Variable(
                        "data".to_string(),
                    ))),
                    HirExpression::Variable("new_size".to_string()),
                ],
            },
        }],
    );

    let code = gen.generate_function(&func);

    println!("Generated: {}", code);

    // Even without explicit cast in HIR, realloc should add casts
    assert!(
        code.contains("as *mut ()") || code.contains("as *mut()"),
        "realloc should cast argument to *mut (). Got: {}",
        code
    );
}
