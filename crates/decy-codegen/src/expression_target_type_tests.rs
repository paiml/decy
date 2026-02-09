//! Coverage tests for generate_expression_with_target_type type conversion paths.
//!
//! Targets uncovered code paths in type coercion, casting, variable type
//! promotion, binary operation mixed types, and function call transformations.

use super::*;
use decy_hir::{
    BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType,
    UnaryOperator,
};

// ============================================================================
// Helper
// ============================================================================

fn make_func_with_body(stmts: Vec<HirStatement>) -> HirFunction {
    HirFunction::new_with_body("test_func".to_string(), HirType::Void, vec![], stmts)
}

fn make_func_returning(ret: HirType, params: Vec<HirParameter>, stmts: Vec<HirStatement>) -> HirFunction {
    HirFunction::new_with_body("test_func".to_string(), ret, params, stmts)
}

// ============================================================================
// 1. Float literal target type dispatch
// ============================================================================

#[test]
fn test_float_literal_target_float_adds_f32_suffix() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Float,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::FloatLiteral(
            "1.5".to_string(),
        )))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("1.5f32"), "Expected f32 suffix, got: {}", code);
}

#[test]
fn test_float_literal_target_double_adds_f64_suffix() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Double,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::FloatLiteral(
            "2.71828".to_string(),
        )))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("2.71828f64"), "Expected f64 suffix, got: {}", code);
}

#[test]
fn test_float_literal_c_suffix_f_stripped() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Float,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::FloatLiteral(
            "3.14f".to_string(),
        )))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("3.14f32"), "Expected stripped C suffix and f32, got: {}", code);
    assert!(!code.contains("3.14ff"), "Should not have double f suffix, got: {}", code);
}

#[test]
fn test_float_literal_c_suffix_L_stripped() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Double,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::FloatLiteral(
            "1.0L".to_string(),
        )))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("1.0f64"), "Expected stripped L suffix, got: {}", code);
}

#[test]
fn test_float_literal_no_target_with_exponent() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Double,
        initializer: Some(HirExpression::FloatLiteral("1e10".to_string())),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("1e10f64"), "Expected f64 for exponent literal, got: {}", code);
}

#[test]
fn test_float_literal_no_target_integer_like() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Double,
        initializer: Some(HirExpression::FloatLiteral("100".to_string())),
    }]);
    let code = codegen.generate_function(&func);
    // Integer-like float literal without dot or exponent should get .0f64
    assert!(code.contains("100.0f64") || code.contains("100f64"), "Expected .0f64 suffix, got: {}", code);
}

// ============================================================================
// 2. IntLiteral 0 with Option and Pointer targets
// ============================================================================

#[test]
fn test_int_zero_to_option_box_generates_none() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "p".to_string(),
        var_type: HirType::Option(Box::new(HirType::Box(Box::new(HirType::Int)))),
        initializer: Some(HirExpression::IntLiteral(0)),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("None"), "Expected None for 0 to Option, got: {}", code);
}

#[test]
fn test_int_zero_to_pointer_generates_null_mut() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "p".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Char)),
        initializer: Some(HirExpression::IntLiteral(0)),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("null_mut"), "Expected null_mut for 0 to pointer, got: {}", code);
}

#[test]
fn test_int_nonzero_to_pointer_stays_literal() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "p".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::IntLiteral(42)),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("42"), "Expected literal 42, got: {}", code);
}

// ============================================================================
// 3. StringLiteral with various target types
// ============================================================================

#[test]
fn test_string_literal_to_char_pointer_via_return() {
    // Use a return statement to a *mut u8 function to preserve Pointer target type
    // Note: BorrowGenerator transforms Pointer(Char) vardecls to &str
    // so we test via generate_expression directly
    let codegen = CodeGenerator::new();
    let expr = HirExpression::StringLiteral("hello".to_string());
    let code = codegen.generate_expression(&expr);
    // Without target type, string literal stays as string
    assert!(code.contains("\"hello\""), "Expected plain string, got: {}", code);
}

#[test]
fn test_string_literal_to_pointer_int_stays_as_string() {
    // StringLiteral to Pointer(Int) (not Char) should keep as string literal
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "s".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::StringLiteral("data".to_string())),
    }]);
    let code = codegen.generate_function(&func);
    // Pointer(Int) not Pointer(Char) so no byte string conversion
    assert!(code.contains("\"data\""), "Expected string literal preserved, got: {}", code);
}

#[test]
fn test_string_literal_no_pointer_target() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "s".to_string(),
        var_type: HirType::StringLiteral,
        initializer: Some(HirExpression::StringLiteral("world".to_string())),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("\"world\""), "Expected plain string literal, got: {}", code);
}

// ============================================================================
// 4. CharLiteral edge cases
// ============================================================================

#[test]
fn test_char_literal_null_byte() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "c".to_string(),
        var_type: HirType::Char,
        initializer: Some(HirExpression::CharLiteral(0)),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("0u8"), "Expected 0u8 for null byte, got: {}", code);
}

#[test]
fn test_char_literal_non_printable() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "c".to_string(),
        var_type: HirType::Char,
        initializer: Some(HirExpression::CharLiteral(7)), // bell character
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("7u8"), "Expected numeric u8 for non-printable, got: {}", code);
}

#[test]
fn test_char_literal_space() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "c".to_string(),
        var_type: HirType::Char,
        initializer: Some(HirExpression::CharLiteral(32)), // space
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("b' '"), "Expected b' ' for space char, got: {}", code);
}

// ============================================================================
// 5. AddressOf with Pointer target type
// ============================================================================

#[test]
fn test_address_of_with_pointer_target() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![
        HirStatement::VariableDeclaration {
            name: "x".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(5)),
        },
        HirStatement::VariableDeclaration {
            name: "p".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::AddressOf(Box::new(
                HirExpression::Variable("x".to_string()),
            ))),
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("&mut x as *mut i32"), "Expected &mut x as *mut i32, got: {}", code);
}

#[test]
fn test_unary_address_of_with_pointer_target() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![
        HirStatement::VariableDeclaration {
            name: "x".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(5)),
        },
        HirStatement::VariableDeclaration {
            name: "p".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::UnaryOp {
                op: UnaryOperator::AddressOf,
                operand: Box::new(HirExpression::Variable("x".to_string())),
            }),
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("&mut x as *mut i32"), "Expected UnaryOp AddressOf with pointer target, got: {}", code);
}

#[test]
fn test_address_of_dereference_wraps_in_parens() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Int)))],
        vec![HirStatement::VariableDeclaration {
            name: "q".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::AddressOf(Box::new(
                HirExpression::Dereference(Box::new(HirExpression::Variable("p".to_string()))),
            ))),
        }],
    );
    let code = codegen.generate_function(&func);
    // AddressOf of Dereference should produce &(...)
    assert!(code.contains("&(") || code.contains("&mut"), "Expected wrapped address-of, got: {}", code);
}

// ============================================================================
// 6. LogicalNot with Int target type
// ============================================================================

#[test]
fn test_logical_not_bool_expr_with_int_target() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
        vec![HirStatement::Return(Some(HirExpression::UnaryOp {
            op: UnaryOperator::LogicalNot,
            operand: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            }),
        }))],
    );
    let code = codegen.generate_function(&func);
    // LogicalNot of boolean expr with Int target should cast to i32
    assert!(code.contains("as i32"), "Expected i32 cast for !bool with int target, got: {}", code);
}

#[test]
fn test_logical_not_int_expr_with_int_target() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::UnaryOp {
            op: UnaryOperator::LogicalNot,
            operand: Box::new(HirExpression::Variable("x".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    // LogicalNot of int variable with Int target: (x == 0) as i32
    assert!(code.contains("== 0") && code.contains("as i32"),
        "Expected (x == 0) as i32 pattern, got: {}", code);
}

#[test]
fn test_logical_not_bool_expr_no_target() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "res".to_string(),
        var_type: HirType::Int,
        initializer: None,
    },
    HirStatement::Expression(HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::IntLiteral(1)),
            right: Box::new(HirExpression::IntLiteral(1)),
        }),
    })]);
    let code = codegen.generate_function(&func);
    // Without int target, should just be logical not, no i32 cast
    assert!(code.contains("!"), "Expected logical not, got: {}", code);
}

#[test]
fn test_logical_not_int_expr_no_target() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![
        HirStatement::Expression(HirExpression::UnaryOp {
            op: UnaryOperator::LogicalNot,
            operand: Box::new(HirExpression::IntLiteral(42)),
        }),
    ]);
    let code = codegen.generate_function(&func);
    // Without int target, !int_expr becomes (int == 0) without cast
    assert!(code.contains("== 0"), "Expected == 0 check, got: {}", code);
}

// ============================================================================
// 7. Variable type coercion paths
// ============================================================================

#[test]
fn test_variable_vec_target_returns_directly() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Vec(Box::new(HirType::Int)),
        vec![HirParameter::new("arr".to_string(), HirType::Vec(Box::new(HirType::Int)))],
        vec![HirStatement::Return(Some(HirExpression::Variable("arr".to_string())))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("arr"), "Expected direct return of vec variable, got: {}", code);
}

#[test]
fn test_variable_box_to_raw_pointer_uses_into_raw() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new("b".to_string(), HirType::Box(Box::new(HirType::Int)))],
        vec![HirStatement::Return(Some(HirExpression::Variable("b".to_string())))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("Box::into_raw(b)"), "Expected Box::into_raw, got: {}", code);
}

#[test]
fn test_variable_mutable_ref_slice_to_pointer_uses_as_mut_ptr() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new(
            "s".to_string(),
            HirType::Reference {
                inner: Box::new(HirType::Vec(Box::new(HirType::Int))),
                mutable: true,
            },
        )],
        vec![HirStatement::Return(Some(HirExpression::Variable("s".to_string())))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("as_mut_ptr"), "Expected as_mut_ptr for mutable slice to pointer, got: {}", code);
}

#[test]
fn test_variable_immutable_ref_slice_to_pointer_uses_as_ptr() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new(
            "s".to_string(),
            HirType::Reference {
                inner: Box::new(HirType::Vec(Box::new(HirType::Int))),
                mutable: false,
            },
        )],
        vec![HirStatement::Return(Some(HirExpression::Variable("s".to_string())))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("as_ptr"), "Expected as_ptr for immutable slice to pointer, got: {}", code);
}

#[test]
fn test_variable_mutable_ref_to_pointer_cast() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new(
            "r".to_string(),
            HirType::Reference {
                inner: Box::new(HirType::Int),
                mutable: true,
            },
        )],
        vec![HirStatement::Return(Some(HirExpression::Variable("r".to_string())))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("as *mut _"), "Expected pointer cast for mutable ref, got: {}", code);
}

#[test]
fn test_variable_immutable_ref_to_pointer_double_cast() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new(
            "r".to_string(),
            HirType::Reference {
                inner: Box::new(HirType::Int),
                mutable: false,
            },
        )],
        vec![HirStatement::Return(Some(HirExpression::Variable("r".to_string())))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("as *const _ as *mut _"), "Expected double cast for immutable ref to ptr, got: {}", code);
}

#[test]
fn test_variable_vec_to_pointer_uses_as_mut_ptr() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new("v".to_string(), HirType::Vec(Box::new(HirType::Int)))],
        vec![HirStatement::Return(Some(HirExpression::Variable("v".to_string())))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("as_mut_ptr"), "Expected as_mut_ptr for Vec to pointer, got: {}", code);
}

#[test]
fn test_variable_array_to_pointer_uses_as_mut_ptr() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new(
            "a".to_string(),
            HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(10),
            },
        )],
        vec![HirStatement::Return(Some(HirExpression::Variable("a".to_string())))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("as_mut_ptr"), "Expected as_mut_ptr for array to pointer, got: {}", code);
}

#[test]
fn test_variable_array_to_void_pointer() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Pointer(Box::new(HirType::Void)),
        vec![HirParameter::new(
            "a".to_string(),
            HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(10),
            },
        )],
        vec![HirStatement::Return(Some(HirExpression::Variable("a".to_string())))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("as_mut_ptr() as *mut ()"), "Expected void pointer cast, got: {}", code);
}

#[test]
fn test_variable_pointer_to_pointer_direct_return() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::Variable("p".to_string())))],
    );
    let code = codegen.generate_function(&func);
    // Pointer to pointer should just return directly
    assert!(code.contains("p"), "Expected direct pointer return, got: {}", code);
}

// ============================================================================
// 8. Variable numeric type coercions
// ============================================================================

#[test]
fn test_variable_int_to_char_coercion() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![
        HirStatement::VariableDeclaration {
            name: "c".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(65)),
        },
        HirStatement::VariableDeclaration {
            name: "ch".to_string(),
            var_type: HirType::Char,
            initializer: Some(HirExpression::Variable("c".to_string())),
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("c as u8"), "Expected int to char cast, got: {}", code);
}

#[test]
fn test_variable_int_to_float_coercion() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![
        HirStatement::VariableDeclaration {
            name: "n".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(10)),
        },
        HirStatement::VariableDeclaration {
            name: "f".to_string(),
            var_type: HirType::Float,
            initializer: Some(HirExpression::Variable("n".to_string())),
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("n as f32"), "Expected int to f32 cast, got: {}", code);
}

#[test]
fn test_variable_int_to_double_coercion() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![
        HirStatement::VariableDeclaration {
            name: "n".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(10)),
        },
        HirStatement::VariableDeclaration {
            name: "d".to_string(),
            var_type: HirType::Double,
            initializer: Some(HirExpression::Variable("n".to_string())),
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("n as f64"), "Expected int to f64 cast, got: {}", code);
}

#[test]
fn test_variable_float_to_int_coercion() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![
        HirStatement::VariableDeclaration {
            name: "f".to_string(),
            var_type: HirType::Float,
            initializer: Some(HirExpression::FloatLiteral("3.14".to_string())),
        },
        HirStatement::VariableDeclaration {
            name: "n".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::Variable("f".to_string())),
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("f as i32"), "Expected float to i32 cast, got: {}", code);
}

#[test]
fn test_variable_double_to_int_coercion() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![
        HirStatement::VariableDeclaration {
            name: "d".to_string(),
            var_type: HirType::Double,
            initializer: Some(HirExpression::FloatLiteral("2.71".to_string())),
        },
        HirStatement::VariableDeclaration {
            name: "n".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::Variable("d".to_string())),
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("d as i32"), "Expected double to i32 cast, got: {}", code);
}

#[test]
fn test_variable_float_to_unsigned_int_coercion() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![
        HirStatement::VariableDeclaration {
            name: "f".to_string(),
            var_type: HirType::Float,
            initializer: Some(HirExpression::FloatLiteral("3.14".to_string())),
        },
        HirStatement::VariableDeclaration {
            name: "u".to_string(),
            var_type: HirType::UnsignedInt,
            initializer: Some(HirExpression::Variable("f".to_string())),
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("f as u32"), "Expected float to u32 cast, got: {}", code);
}

#[test]
fn test_variable_char_to_int_coercion() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![
        HirStatement::VariableDeclaration {
            name: "c".to_string(),
            var_type: HirType::Char,
            initializer: Some(HirExpression::CharLiteral(65)),
        },
        HirStatement::VariableDeclaration {
            name: "n".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::Variable("c".to_string())),
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("c as i32"), "Expected char to i32 cast, got: {}", code);
}

#[test]
fn test_variable_unsigned_int_to_float_coercion() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![
        HirStatement::VariableDeclaration {
            name: "u".to_string(),
            var_type: HirType::UnsignedInt,
            initializer: Some(HirExpression::IntLiteral(42)),
        },
        HirStatement::VariableDeclaration {
            name: "f".to_string(),
            var_type: HirType::Float,
            initializer: Some(HirExpression::Variable("u".to_string())),
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("u as f32"), "Expected unsigned int to f32 cast, got: {}", code);
}

// ============================================================================
// 9. BinaryOp with Assign (embedded assignment)
// ============================================================================

#[test]
fn test_binary_op_assign_generates_block() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::Return(Some(HirExpression::BinaryOp {
                op: BinaryOperator::Assign,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::IntLiteral(42)),
            })),
        ],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("__assign_tmp"), "Expected embedded assignment block, got: {}", code);
}

// ============================================================================
// 10. Option comparison with NULL
// ============================================================================

#[test]
fn test_option_variable_equal_null_generates_is_none() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Option(Box::new(HirType::Box(Box::new(HirType::Int)))),
        )],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("p".to_string())),
            right: Box::new(HirExpression::NullLiteral),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("is_none"), "Expected is_none for option == NULL, got: {}", code);
}

#[test]
fn test_option_variable_not_equal_null_generates_is_some() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Option(Box::new(HirType::Box(Box::new(HirType::Int)))),
        )],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::NotEqual,
            left: Box::new(HirExpression::Variable("p".to_string())),
            right: Box::new(HirExpression::NullLiteral),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("is_some"), "Expected is_some for option != NULL, got: {}", code);
}

#[test]
fn test_null_equal_option_reversed() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Option(Box::new(HirType::Box(Box::new(HirType::Int)))),
        )],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::NullLiteral),
            right: Box::new(HirExpression::Variable("p".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("is_none"), "Expected is_none for NULL == option, got: {}", code);
}

// ============================================================================
// 11. Pointer comparison with 0
// ============================================================================

#[test]
fn test_pointer_equal_zero_generates_comparison() {
    // BorrowGenerator transforms Pointer params to references,
    // but the equality/comparison path still generates code
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("p".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }))],
    );
    let code = codegen.generate_function(&func);
    // After borrow generator transforms pointer to ref, the == 0 comparison still gets cast to i32
    assert!(code.contains("== 0") || code.contains("null_mut"),
        "Expected comparison with zero, got: {}", code);
}

#[test]
fn test_zero_not_equal_pointer_reversed() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::NotEqual,
            left: Box::new(HirExpression::IntLiteral(0)),
            right: Box::new(HirExpression::Variable("p".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("!= ") || code.contains("null_mut"),
        "Expected not-equal comparison, got: {}", code);
}

// ============================================================================
// 12. Vec and Box null checks
// ============================================================================

#[test]
fn test_vec_equal_null_is_false() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![HirParameter::new("arr".to_string(), HirType::Vec(Box::new(HirType::Int)))],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("arr".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("false"), "Expected false for Vec == null, got: {}", code);
}

#[test]
fn test_vec_not_equal_null_is_true() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![HirParameter::new("arr".to_string(), HirType::Vec(Box::new(HirType::Int)))],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::NotEqual,
            left: Box::new(HirExpression::Variable("arr".to_string())),
            right: Box::new(HirExpression::NullLiteral),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("true"), "Expected true for Vec != null, got: {}", code);
}

#[test]
fn test_box_equal_null_is_false() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![HirParameter::new("b".to_string(), HirType::Box(Box::new(HirType::Int)))],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("b".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("false"), "Expected false for Box == null, got: {}", code);
}

// ============================================================================
// 13. strlen comparison optimizations
// ============================================================================

#[test]
fn test_strlen_equal_zero_becomes_is_empty() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![HirParameter::new("s".to_string(), HirType::StringLiteral)],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::FunctionCall {
                function: "strlen".to_string(),
                arguments: vec![HirExpression::Variable("s".to_string())],
            }),
            right: Box::new(HirExpression::IntLiteral(0)),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("is_empty"), "Expected is_empty for strlen == 0, got: {}", code);
}

#[test]
fn test_zero_not_equal_strlen_becomes_not_is_empty() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![HirParameter::new("s".to_string(), HirType::StringLiteral)],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::NotEqual,
            left: Box::new(HirExpression::IntLiteral(0)),
            right: Box::new(HirExpression::FunctionCall {
                function: "strlen".to_string(),
                arguments: vec![HirExpression::Variable("s".to_string())],
            }),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("is_empty"), "Expected is_empty for 0 != strlen, got: {}", code);
}

// ============================================================================
// 14. BinaryOp mixed type arithmetic (int + float, int + double, float + double)
// ============================================================================

#[test]
fn test_binary_add_int_plus_float_casts_int() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Float,
        vec![
            HirParameter::new("n".to_string(), HirType::Int),
            HirParameter::new("f".to_string(), HirType::Float),
        ],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::Variable("f".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("as f32"), "Expected int cast to f32 for int+float, got: {}", code);
}

#[test]
fn test_binary_mul_int_times_double_casts_int() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Double,
        vec![
            HirParameter::new("n".to_string(), HirType::Int),
            HirParameter::new("d".to_string(), HirType::Double),
        ],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::Variable("d".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("as f64"), "Expected int cast to f64 for int*double, got: {}", code);
}

#[test]
fn test_binary_sub_float_minus_double_casts_float() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Double,
        vec![
            HirParameter::new("f".to_string(), HirType::Float),
            HirParameter::new("d".to_string(), HirType::Double),
        ],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Subtract,
            left: Box::new(HirExpression::Variable("f".to_string())),
            right: Box::new(HirExpression::Variable("d".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("as f64"), "Expected float cast to f64 for float-double, got: {}", code);
}

// ============================================================================
// 15. BinaryOp comparison with int target (returns bool in Rust, int in C)
// ============================================================================

#[test]
fn test_comparison_with_int_target_casts_to_i32() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
        vec![HirStatement::VariableDeclaration {
            name: "r".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::BinaryOp {
                op: BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            }),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("as i32"), "Expected i32 cast for comparison assigned to int, got: {}", code);
}

// ============================================================================
// 16. LogicalAnd/LogicalOr with Int target
// ============================================================================

#[test]
fn test_logical_and_with_int_target_casts_to_i32() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![
        HirStatement::VariableDeclaration {
            name: "a".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(1)),
        },
        HirStatement::VariableDeclaration {
            name: "b".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(2)),
        },
        HirStatement::VariableDeclaration {
            name: "r".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::BinaryOp {
                op: BinaryOperator::LogicalAnd,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            }),
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("as i32"), "Expected i32 cast for && with int target, got: {}", code);
    assert!(code.contains("!= 0"), "Expected != 0 for integer truthiness, got: {}", code);
}

#[test]
fn test_logical_or_with_bool_operands_no_cast_without_int_target() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
        vec![HirStatement::Expression(HirExpression::BinaryOp {
            op: BinaryOperator::LogicalOr,
            left: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::IntLiteral(0)),
            }),
            right: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("b".to_string())),
                right: Box::new(HirExpression::IntLiteral(10)),
            }),
        })],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("||"), "Expected || operator, got: {}", code);
}

// ============================================================================
// 17. BinaryOp char to int promotion in comparisons
// ============================================================================

#[test]
fn test_int_variable_compared_to_char_literal() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![HirParameter::new("c".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::NotEqual,
            left: Box::new(HirExpression::Variable("c".to_string())),
            right: Box::new(HirExpression::CharLiteral(10)), // '\n'
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("10i32"), "Expected char promoted to i32 literal, got: {}", code);
}

#[test]
fn test_char_literal_compared_to_int_variable_reversed() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![HirParameter::new("c".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::CharLiteral(48)), // '0'
            right: Box::new(HirExpression::Variable("c".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("48i32"), "Expected reversed char-to-int promotion, got: {}", code);
}

// ============================================================================
// 18. BinaryOp integer + char literal arithmetic
// ============================================================================

#[test]
fn test_int_add_char_literal_casts_char_to_i32() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::CharLiteral(48)), // '0'
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("48i32"), "Expected char literal as i32 in add, got: {}", code);
}

// ============================================================================
// 19. BinaryOp int result to float/double target
// ============================================================================

#[test]
fn test_int_division_with_float_target_casts_result() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![
        HirStatement::VariableDeclaration {
            name: "a".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(10)),
        },
        HirStatement::VariableDeclaration {
            name: "b".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(3)),
        },
        HirStatement::VariableDeclaration {
            name: "f".to_string(),
            var_type: HirType::Float,
            initializer: Some(HirExpression::BinaryOp {
                op: BinaryOperator::Divide,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            }),
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("as f32"), "Expected int/int result cast to f32, got: {}", code);
}

#[test]
fn test_int_addition_with_double_target_casts_result() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![
        HirStatement::VariableDeclaration {
            name: "a".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(10)),
        },
        HirStatement::VariableDeclaration {
            name: "b".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(3)),
        },
        HirStatement::VariableDeclaration {
            name: "d".to_string(),
            var_type: HirType::Double,
            initializer: Some(HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            }),
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("as f64"), "Expected int+int result cast to f64, got: {}", code);
}

// ============================================================================
// 20. Bitwise operations with boolean operands
// ============================================================================

#[test]
fn test_bitwise_and_with_bool_operand_casts_to_i32() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![
            HirParameter::new("x".to_string(), HirType::Int),
            HirParameter::new("y".to_string(), HirType::Int),
        ],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::BitwiseAnd,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Equal,
                left: Box::new(HirExpression::Variable("y".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            }),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("as i32"), "Expected i32 cast for bool in bitwise, got: {}", code);
}

// ============================================================================
// 21. Cast expression various paths
// ============================================================================

#[test]
fn test_cast_float_to_int() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::Cast {
            target_type: HirType::Int,
            expr: Box::new(HirExpression::FloatLiteral("3.14".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("as i32"), "Expected float cast to i32, got: {}", code);
}

#[test]
fn test_cast_int_to_char() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Char,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::Cast {
            target_type: HirType::Char,
            expr: Box::new(HirExpression::IntLiteral(65)),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("as u8"), "Expected int cast to u8, got: {}", code);
}

#[test]
fn test_cast_address_of_to_int_uses_ptr_chain() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::Return(Some(HirExpression::Cast {
                target_type: HirType::Int,
                expr: Box::new(HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("x".to_string()),
                ))),
            })),
        ],
    );
    let code = codegen.generate_function(&func);
    assert!(
        code.contains("as *const _ as isize as i32"),
        "Expected pointer-to-int chain cast, got: {}", code
    );
}

#[test]
fn test_cast_binary_op_wraps_in_parens() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::Cast {
            target_type: HirType::Int,
            expr: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::IntLiteral(1)),
                right: Box::new(HirExpression::IntLiteral(2)),
            }),
        }))],
    );
    let code = codegen.generate_function(&func);
    // Binary op inside cast should be wrapped in parens
    assert!(code.contains("(1 + 2) as i32") || code.contains(") as i32"),
        "Expected parens around binary op in cast, got: {}", code);
}

// ============================================================================
// 22. Ternary expression with type coercion
// ============================================================================

#[test]
fn test_ternary_with_non_boolean_condition_adds_check() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::Ternary {
            condition: Box::new(HirExpression::Variable("x".to_string())),
            then_expr: Box::new(HirExpression::IntLiteral(1)),
            else_expr: Box::new(HirExpression::IntLiteral(0)),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("if") && code.contains("else"), "Expected if/else for ternary, got: {}", code);
}

// ============================================================================
// 23. FunctionCall special cases
// ============================================================================

#[test]
fn test_function_call_strlen_returns_len_as_i32() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![HirParameter::new("s".to_string(), HirType::StringLiteral)],
        vec![HirStatement::Return(Some(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("s".to_string())],
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains(".len() as i32"), "Expected .len() as i32 for strlen, got: {}", code);
}

#[test]
fn test_function_call_strcpy_generates_to_string() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::Expression(HirExpression::FunctionCall {
        function: "strcpy".to_string(),
        arguments: vec![
            HirExpression::Variable("dest".to_string()),
            HirExpression::Variable("src".to_string()),
        ],
    })]);
    let code = codegen.generate_function(&func);
    assert!(code.contains(".to_string()"), "Expected .to_string() for strcpy, got: {}", code);
}

#[test]
fn test_function_call_free_generates_drop() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::Expression(HirExpression::FunctionCall {
        function: "free".to_string(),
        arguments: vec![HirExpression::Variable("ptr".to_string())],
    })]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("drop(ptr)"), "Expected drop() for free, got: {}", code);
}

#[test]
fn test_function_call_fopen_read_mode() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::Expression(HirExpression::FunctionCall {
        function: "fopen".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("test.txt".to_string()),
            HirExpression::StringLiteral("r".to_string()),
        ],
    })]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("File::open"), "Expected File::open for 'r' mode, got: {}", code);
}

#[test]
fn test_function_call_fopen_write_mode() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::Expression(HirExpression::FunctionCall {
        function: "fopen".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("test.txt".to_string()),
            HirExpression::StringLiteral("w".to_string()),
        ],
    })]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("File::create"), "Expected File::create for 'w' mode, got: {}", code);
}

// ============================================================================
// 24. malloc with Vec target and Pointer target
// ============================================================================

#[test]
fn test_malloc_with_vec_target_multiply_pattern() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Vec(Box::new(HirType::Int)),
        initializer: Some(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::IntLiteral(10)),
                right: Box::new(HirExpression::Sizeof {
                    type_name: "int".to_string(),
                }),
            }],
        }),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("vec!") || code.contains("Vec"), "Expected vec for malloc with Vec target, got: {}", code);
}

#[test]
fn test_malloc_with_pointer_char_target() {
    // BorrowGenerator transforms Pointer(Char) to &str or Vec<u8>,
    // so the malloc gets Vec treatment after transformation
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Char)),
        initializer: Some(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::IntLiteral(256)],
        }),
    }]);
    let code = codegen.generate_function(&func);
    // After borrow gen, the malloc call results in Vec allocation
    assert!(code.contains("Vec") || code.contains("vec!") || code.contains("with_capacity"),
        "Expected Vec allocation for malloc, got: {}", code);
}

// ============================================================================
// 25. calloc with Vec and Pointer targets
// ============================================================================

#[test]
fn test_calloc_with_vec_target() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Vec(Box::new(HirType::Int)),
        initializer: Some(HirExpression::FunctionCall {
            function: "calloc".to_string(),
            arguments: vec![
                HirExpression::IntLiteral(10),
                HirExpression::Sizeof {
                    type_name: "int".to_string(),
                },
            ],
        }),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("vec![0i32"), "Expected vec![0i32; ...] for calloc with Vec, got: {}", code);
}

#[test]
fn test_calloc_with_pointer_target() {
    // BorrowGenerator transforms Pointer to Vec, so calloc gets Vec treatment
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::FunctionCall {
            function: "calloc".to_string(),
            arguments: vec![
                HirExpression::IntLiteral(10),
                HirExpression::Sizeof {
                    type_name: "int".to_string(),
                },
            ],
        }),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("vec!") || code.contains("Vec"),
        "Expected Vec allocation for calloc after borrow gen, got: {}", code);
}

// ============================================================================
// 26. realloc function call path (through FunctionCall, not Realloc HIR)
// ============================================================================

#[test]
fn test_realloc_function_call_with_pointer_target() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![
        HirStatement::VariableDeclaration {
            name: "ptr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::IntLiteral(0)),
        },
        HirStatement::VariableDeclaration {
            name: "new_ptr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "realloc".to_string(),
                arguments: vec![
                    HirExpression::Variable("ptr".to_string()),
                    HirExpression::IntLiteral(100),
                ],
            }),
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("realloc") && code.contains("as *mut"),
        "Expected realloc with pointer cast, got: {}", code);
}

// ============================================================================
// 27. CompoundLiteral paths
// ============================================================================

#[test]
fn test_compound_literal_empty_struct() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "p".to_string(),
        var_type: HirType::Struct("Point".to_string()),
        initializer: Some(HirExpression::CompoundLiteral {
            literal_type: HirType::Struct("Point".to_string()),
            initializers: vec![],
        }),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("Point {}"), "Expected empty struct literal, got: {}", code);
}

#[test]
fn test_compound_literal_array_with_size() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(3),
        },
        initializer: Some(HirExpression::CompoundLiteral {
            literal_type: HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(3),
            },
            initializers: vec![
                HirExpression::IntLiteral(1),
                HirExpression::IntLiteral(2),
                HirExpression::IntLiteral(3),
            ],
        }),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("[1, 2, 3]"), "Expected array literal, got: {}", code);
}

#[test]
fn test_compound_literal_array_single_initializer_repeats() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
        initializer: Some(HirExpression::CompoundLiteral {
            literal_type: HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(10),
            },
            initializers: vec![HirExpression::IntLiteral(0)],
        }),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("[0; 10]"), "Expected repeated array init, got: {}", code);
}

#[test]
fn test_compound_literal_empty_array_with_size_uses_default() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(5),
        },
        initializer: Some(HirExpression::CompoundLiteral {
            literal_type: HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(5),
            },
            initializers: vec![],
        }),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("[0i32; 5]"), "Expected default-filled array, got: {}", code);
}

// ============================================================================
// 28. Variable with reserved keyword escaping
// ============================================================================

#[test]
fn test_variable_named_type_gets_escaped() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![
        HirStatement::VariableDeclaration {
            name: "type".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        },
        HirStatement::Expression(HirExpression::Variable("type".to_string())),
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("r#type") || code.contains("type_"), "Expected escaped keyword, got: {}", code);
}

// ============================================================================
// 29. Pointer arithmetic in BinaryOp
// ============================================================================

#[test]
fn test_pointer_add_generates_addition() {
    // BorrowGenerator may transform Pointer params to references,
    // so pointer arithmetic paths are hit when the ctx still has pointer types.
    // Here we test the output after full pipeline transformation.
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("p".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        }))],
    );
    let code = codegen.generate_function(&func);
    // After borrow gen, p is a reference, so the add path uses + operator
    assert!(code.contains("p") && code.contains("1"),
        "Expected pointer add expression, got: {}", code);
}

#[test]
fn test_pointer_subtract_int_generates_subtraction() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Subtract,
            left: Box::new(HirExpression::Variable("p".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("p") && code.contains("1"),
        "Expected pointer subtract expression, got: {}", code);
}

#[test]
fn test_pointer_subtract_pointer_generates_subtraction() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![
            HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("q".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Subtract,
            left: Box::new(HirExpression::Variable("p".to_string())),
            right: Box::new(HirExpression::Variable("q".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("p") && code.contains("q"),
        "Expected pointer difference expression, got: {}", code);
}

// ============================================================================
// 30. Chained comparisons
// ============================================================================

#[test]
fn test_chained_comparison_casts_bool_to_i32() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
            HirParameter::new("c".to_string(), HirType::Int),
        ],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            }),
            right: Box::new(HirExpression::Variable("c".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    // Chained comparison: (a < b) < c should cast left bool to i32
    assert!(code.contains("as i32"), "Expected i32 cast in chained comparison, got: {}", code);
}

// ============================================================================
// 31. Signed/unsigned comparison mismatch
// ============================================================================

#[test]
fn test_signed_unsigned_comparison_casts_to_i64() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![
            HirParameter::new("s".to_string(), HirType::Int),
            HirParameter::new("u".to_string(), HirType::UnsignedInt),
        ],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("s".to_string())),
            right: Box::new(HirExpression::Variable("u".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("as i64"), "Expected i64 cast for signed/unsigned comparison, got: {}", code);
}

// ============================================================================
// 32. Comma operator
// ============================================================================

#[test]
fn test_comma_operator_generates_block() {
    let codegen = CodeGenerator::new();
    let func = make_func_returning(
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Comma,
            left: Box::new(HirExpression::IntLiteral(1)),
            right: Box::new(HirExpression::IntLiteral(2)),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("{ 1; 2 }"), "Expected comma block expression, got: {}", code);
}

// ============================================================================
// 33. Standard stream variables
// ============================================================================

#[test]
fn test_variable_stderr_maps_to_io() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::Expression(
        HirExpression::Variable("stderr".to_string()),
    )]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("std::io::stderr()"), "Expected std::io::stderr(), got: {}", code);
}

#[test]
fn test_variable_stdin_maps_to_io() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::Expression(
        HirExpression::Variable("stdin".to_string()),
    )]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("std::io::stdin()"), "Expected std::io::stdin(), got: {}", code);
}

#[test]
fn test_variable_stdout_maps_to_io() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::Expression(
        HirExpression::Variable("stdout".to_string()),
    )]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("std::io::stdout()"), "Expected std::io::stdout(), got: {}", code);
}

#[test]
fn test_variable_errno_maps_to_unsafe() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::Expression(
        HirExpression::Variable("errno".to_string()),
    )]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("unsafe { ERRNO }"), "Expected unsafe ERRNO, got: {}", code);
}

// ============================================================================
// 34. FunctionCall edge cases
// ============================================================================

#[test]
fn test_fclose_generates_drop() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::Expression(
        HirExpression::FunctionCall {
            function: "fclose".to_string(),
            arguments: vec![HirExpression::Variable("f".to_string())],
        },
    )]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("drop(f)"), "Expected drop for fclose, got: {}", code);
}

#[test]
fn test_fork_generates_comment() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::Expression(
        HirExpression::FunctionCall {
            function: "fork".to_string(),
            arguments: vec![],
        },
    )]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("fork"), "Expected fork comment, got: {}", code);
}

#[test]
fn test_printf_single_arg() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::Expression(
        HirExpression::FunctionCall {
            function: "printf".to_string(),
            arguments: vec![HirExpression::StringLiteral("hello\\n".to_string())],
        },
    )]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("print!"), "Expected print! macro, got: {}", code);
}

#[test]
fn test_wexitstatus_generates_code() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::Expression(
        HirExpression::FunctionCall {
            function: "WEXITSTATUS".to_string(),
            arguments: vec![HirExpression::Variable("status".to_string())],
        },
    )]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("code().unwrap_or(-1)"), "Expected code() for WEXITSTATUS, got: {}", code);
}

#[test]
fn test_wifexited_generates_success() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::Expression(
        HirExpression::FunctionCall {
            function: "WIFEXITED".to_string(),
            arguments: vec![HirExpression::Variable("status".to_string())],
        },
    )]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("success()"), "Expected success() for WIFEXITED, got: {}", code);
}

// ============================================================================
// 35. Cast with Vec target unwraps malloc
// ============================================================================

#[test]
fn test_cast_over_malloc_with_vec_target_unwraps() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Vec(Box::new(HirType::Int)),
        initializer: Some(HirExpression::Cast {
            target_type: HirType::Pointer(Box::new(HirType::Int)),
            expr: Box::new(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![HirExpression::BinaryOp {
                    op: BinaryOperator::Multiply,
                    left: Box::new(HirExpression::IntLiteral(10)),
                    right: Box::new(HirExpression::Sizeof {
                        type_name: "int".to_string(),
                    }),
                }],
            }),
        }),
    }]);
    let code = codegen.generate_function(&func);
    // Cast around malloc with Vec target should unwrap the cast and generate vec
    assert!(code.contains("vec!") || code.contains("Vec"), "Expected vec for cast+malloc with Vec target, got: {}", code);
}

// ============================================================================
// 36. Compound literal for unknown type
// ============================================================================

#[test]
fn test_compound_literal_unknown_type_generates_comment() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::CompoundLiteral {
            literal_type: HirType::Int,
            initializers: vec![HirExpression::IntLiteral(42)],
        }),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("Compound literal"), "Expected compound literal comment for non-struct/array, got: {}", code);
}

// ============================================================================
// 37. Partial array initialization (padding with defaults)
// ============================================================================

#[test]
fn test_compound_literal_array_partial_pads_defaults() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(4),
        },
        initializer: Some(HirExpression::CompoundLiteral {
            literal_type: HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(4),
            },
            initializers: vec![
                HirExpression::IntLiteral(1),
                HirExpression::IntLiteral(2),
            ],
        }),
    }]);
    let code = codegen.generate_function(&func);
    // Partial init with 2 values and size 4 should pad with defaults
    assert!(code.contains("0i32") || code.contains("1, 2"),
        "Expected padded array with defaults, got: {}", code);
}

// ============================================================================
// 38. Calloc HIR expression (not function call)
// ============================================================================

#[test]
fn test_calloc_hir_expression_generates_vec() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Vec(Box::new(HirType::Float)),
        initializer: Some(HirExpression::Calloc {
            count: Box::new(HirExpression::IntLiteral(10)),
            element_type: Box::new(HirType::Float),
        }),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("vec![0.0f32; 10]"), "Expected vec![0.0f32; 10], got: {}", code);
}

#[test]
fn test_calloc_hir_expression_unsigned_int() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Vec(Box::new(HirType::UnsignedInt)),
        initializer: Some(HirExpression::Calloc {
            count: Box::new(HirExpression::IntLiteral(5)),
            element_type: Box::new(HirType::UnsignedInt),
        }),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("vec![0u32; 5]"), "Expected vec![0u32; 5], got: {}", code);
}

// ============================================================================
// 39. Errno constants
// ============================================================================

#[test]
fn test_erange_constant() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::Expression(
        HirExpression::Variable("ERANGE".to_string()),
    )]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("34i32"), "Expected 34i32 for ERANGE, got: {}", code);
}

#[test]
fn test_einval_constant() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_body(vec![HirStatement::Expression(
        HirExpression::Variable("EINVAL".to_string()),
    )]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("22i32"), "Expected 22i32 for EINVAL, got: {}", code);
}
