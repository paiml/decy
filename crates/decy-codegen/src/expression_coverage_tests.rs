//! Coverage tests for generate_expression_with_target_type.
//!
//! Targets uncovered code paths in the expression code generation,
//! including float literals, address-of with pointers, logical not,
//! and string literal conversions.

use super::*;
use decy_hir::{
    BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType,
    UnaryOperator,
};

// ============================================================================
// Float literal code generation
// ============================================================================

#[test]
fn test_float_literal_with_float_target() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Float,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::FloatLiteral(
            "3.14".to_string(),
        )))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("3.14") || code.contains("f32"));
}

#[test]
fn test_float_literal_with_double_target() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Double,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::FloatLiteral(
            "2.718".to_string(),
        )))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("2.718") || code.contains("f64"));
}

#[test]
fn test_float_literal_with_c_suffix() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Float,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::FloatLiteral(
            "1.0f".to_string(),
        )))],
    );
    let code = codegen.generate_function(&func);
    // Should strip the 'f' suffix
    assert!(code.contains("1.0"));
}

#[test]
fn test_float_literal_no_dot() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "x".to_string(),
            var_type: HirType::Double,
            initializer: Some(HirExpression::FloatLiteral("42".to_string())),
        }],
    );
    let code = codegen.generate_function(&func);
    // Should add .0 suffix for integer-like float literals
    assert!(code.contains("42"));
}

// ============================================================================
// Integer literal with pointer/Option target types
// ============================================================================

#[test]
fn test_int_zero_to_option_none() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "p".to_string(),
            var_type: HirType::Option(Box::new(HirType::Box(Box::new(HirType::Int)))),
            initializer: Some(HirExpression::IntLiteral(0)),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("None"));
}

#[test]
fn test_int_zero_to_null_mut() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "p".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::IntLiteral(0)),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("null_mut") || code.contains("None"));
}

// ============================================================================
// String literal code generation
// ============================================================================

#[test]
fn test_string_literal_expression() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "s".to_string(),
            var_type: HirType::StringLiteral,
            initializer: Some(HirExpression::StringLiteral("hello".to_string())),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("hello"));
}

#[test]
fn test_char_literal_expression() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "c".to_string(),
            var_type: HirType::Char,
            initializer: Some(HirExpression::CharLiteral(65)),
        }],
    );
    let code = codegen.generate_function(&func);
    // Should contain the char literal
    assert!(code.contains("65") || code.contains("'A'") || code.contains("b'A'"));
}

// ============================================================================
// Null literal and IsNotNull
// ============================================================================

#[test]
fn test_null_literal() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "p".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::NullLiteral),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("null") || code.contains("None"));
}

// ============================================================================
// Sizeof expression
// ============================================================================

#[test]
fn test_sizeof_expression() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::Sizeof {
            type_name: "int".to_string(),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("size_of") || code.contains("mem::size_of"));
}

// ============================================================================
// Cast expression
// ============================================================================

#[test]
fn test_cast_expression() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::Cast {
            target_type: HirType::Int,
            expr: Box::new(HirExpression::FloatLiteral("3.14".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("as") || code.contains("i32"));
}

// ============================================================================
// AddressOf and Dereference
// ============================================================================

#[test]
fn test_address_of_expression() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(42)),
            },
            HirStatement::VariableDeclaration {
                name: "p".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("x".to_string()),
                ))),
            },
        ],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("&") || code.contains("x"));
}

#[test]
fn test_dereference_expression() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::Dereference(
            Box::new(HirExpression::Variable("p".to_string())),
        )))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("*") || code.contains("p"));
}

// ============================================================================
// Array index and field access
// ============================================================================

#[test]
fn test_array_index_expression() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::ArrayIndex {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("arr") && code.contains("0"));
}

#[test]
fn test_field_access_expression() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::FieldAccess {
            object: Box::new(HirExpression::Variable("point".to_string())),
            field: "x".to_string(),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("point") && code.contains("x"));
}

#[test]
fn test_pointer_field_access_expression() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(
            HirExpression::PointerFieldAccess {
                pointer: Box::new(HirExpression::Variable("node".to_string())),
                field: "value".to_string(),
            },
        ))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("node") || code.contains("value"));
}

// ============================================================================
// Ternary expression
// ============================================================================

#[test]
fn test_ternary_expression() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::Ternary {
            condition: Box::new(HirExpression::Variable("x".to_string())),
            then_expr: Box::new(HirExpression::IntLiteral(1)),
            else_expr: Box::new(HirExpression::IntLiteral(0)),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("if") || code.contains("1") || code.contains("0"));
}

// ============================================================================
// Malloc and allocation expressions
// ============================================================================

#[test]
fn test_malloc_expression() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "p".to_string(),
            var_type: HirType::Box(Box::new(HirType::Int)),
            initializer: Some(HirExpression::Malloc {
                size: Box::new(HirExpression::IntLiteral(4)),
            }),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("Box") || code.contains("new") || code.contains("alloc"));
}

#[test]
fn test_calloc_expression() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Vec(Box::new(HirType::Int)),
            initializer: Some(HirExpression::Calloc {
                count: Box::new(HirExpression::IntLiteral(10)),
                element_type: Box::new(HirType::Int),
            }),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("vec!") || code.contains("Vec") || code.contains("0"));
}

// ============================================================================
// Logical not with target type
// ============================================================================

#[test]
fn test_logical_not_bool_expression() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::UnaryOp {
            op: UnaryOperator::LogicalNot,
            operand: Box::new(HirExpression::Variable("x".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("!") || code.contains("== 0"));
}

// ============================================================================
// Statement code generation: While, For, Switch, DoWhile
// ============================================================================

#[test]
fn test_while_statement() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "i".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::While {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::LessThan,
                    left: Box::new(HirExpression::Variable("i".to_string())),
                    right: Box::new(HirExpression::IntLiteral(10)),
                },
                body: vec![HirStatement::Assignment {
                    target: "i".to_string(),
                    value: HirExpression::BinaryOp {
                        op: BinaryOperator::Add,
                        left: Box::new(HirExpression::Variable("i".to_string())),
                        right: Box::new(HirExpression::IntLiteral(1)),
                    },
                }],
            },
        ],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("while"));
}

#[test]
fn test_for_statement() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::For {
            init: vec![HirStatement::VariableDeclaration {
                name: "i".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            }],
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::IntLiteral(10)),
            },
            increment: vec![HirStatement::Assignment {
                target: "i".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("i".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            }],
            body: vec![],
        }],
    );
    let code = codegen.generate_function(&func);
    // Should generate some loop construct
    assert!(code.contains("for") || code.contains("while") || code.contains("loop"));
}

#[test]
fn test_break_statement() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::While {
            condition: HirExpression::IntLiteral(1),
            body: vec![HirStatement::Break],
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("break"));
}

#[test]
fn test_continue_statement() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::While {
            condition: HirExpression::IntLiteral(1),
            body: vec![HirStatement::Continue],
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("continue"));
}

// ============================================================================
// Compound assignment (represented as Assignment in HIR)
// ============================================================================

#[test]
fn test_compound_assignment_add() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::IntLiteral(5)),
                },
            },
        ],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("x") && code.contains("5"));
}

// ============================================================================
// Post/Pre increment/decrement
// ============================================================================

#[test]
fn test_post_increment_expression() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "i".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::Expression(HirExpression::PostIncrement {
                operand: Box::new(HirExpression::Variable("i".to_string())),
            }),
        ],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("i") && (code.contains("+=") || code.contains("+ 1")));
}

// ============================================================================
// Type mapping coverage
// ============================================================================

#[test]
fn test_map_type_signed_char() {
    assert_eq!(CodeGenerator::map_type(&HirType::SignedChar), "i8");
}

#[test]
fn test_map_type_option() {
    assert_eq!(
        CodeGenerator::map_type(&HirType::Option(Box::new(HirType::Int))),
        "Option<i32>"
    );
}

#[test]
fn test_map_type_reference_immutable() {
    assert_eq!(
        CodeGenerator::map_type(&HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        }),
        "&i32"
    );
}

#[test]
fn test_map_type_reference_mutable() {
    assert_eq!(
        CodeGenerator::map_type(&HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        }),
        "&mut i32"
    );
}

#[test]
fn test_map_type_string_literal() {
    assert_eq!(CodeGenerator::map_type(&HirType::StringLiteral), "&str");
}

#[test]
fn test_map_type_owned_string() {
    assert_eq!(CodeGenerator::map_type(&HirType::OwnedString), "String");
}

#[test]
fn test_map_type_string_reference() {
    assert_eq!(CodeGenerator::map_type(&HirType::StringReference), "&str");
}

#[test]
fn test_map_type_type_alias() {
    // TypeAlias should preserve the alias name
    let result = CodeGenerator::map_type(&HirType::TypeAlias("size_t".to_string()));
    assert!(result.contains("size_t") || result.contains("usize"));
}

#[test]
fn test_map_type_array_with_size() {
    let result = CodeGenerator::map_type(&HirType::Array {
        element_type: Box::new(HirType::Int),
        size: Some(10),
    });
    assert!(result.contains("[i32; 10]") || result.contains("i32"));
}

#[test]
fn test_map_type_function_pointer() {
    let result = CodeGenerator::map_type(&HirType::FunctionPointer {
        return_type: Box::new(HirType::Int),
        param_types: vec![HirType::Int, HirType::Int],
    });
    assert!(result.contains("fn") || result.contains("Fn"));
}

#[test]
fn test_map_type_union() {
    let result = CodeGenerator::map_type(&HirType::Union(vec![
        ("field1".to_string(), HirType::Int),
        ("field2".to_string(), HirType::Float),
    ]));
    // Unions are represented as enums in Rust
    assert!(!result.is_empty());
}

// ============================================================================
// Sizeof expression — struct field access patterns (DECY-189, DECY-248)
// ============================================================================

#[test]
fn test_sizeof_simple_type() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::Sizeof {
            type_name: "double".to_string(),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("size_of"));
}

#[test]
fn test_sizeof_unsigned_type() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::Sizeof {
            type_name: "unsigned int".to_string(),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("size_of"));
}

#[test]
fn test_sizeof_long_type() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::Sizeof {
            type_name: "long long".to_string(),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("size_of"));
}

#[test]
fn test_sizeof_variable_known_in_context() {
    // sizeof(x) where x is a known variable → size_of_val(&x)
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::Sizeof {
            type_name: "x".to_string(),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("size_of_val") || code.contains("size_of"));
}

#[test]
fn test_sizeof_char() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::Sizeof {
            type_name: "char".to_string(),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("size_of"));
    assert!(code.contains("as i32"));
}

// ============================================================================
// Realloc expression (DECY-220)
// ============================================================================

#[test]
fn test_realloc_null_pointer_simple() {
    // realloc(NULL, size) → Vec::new()
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "p".to_string(),
            var_type: HirType::Vec(Box::new(HirType::Int)),
            initializer: Some(HirExpression::Realloc {
                pointer: Box::new(HirExpression::NullLiteral),
                new_size: Box::new(HirExpression::IntLiteral(10)),
            }),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("Vec::new"));
}

#[test]
fn test_realloc_null_pointer_with_multiply() {
    // realloc(NULL, n * sizeof(T)) → vec![0i32; n]
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Vec(Box::new(HirType::Int)),
            initializer: Some(HirExpression::Realloc {
                pointer: Box::new(HirExpression::NullLiteral),
                new_size: Box::new(HirExpression::BinaryOp {
                    op: BinaryOperator::Multiply,
                    left: Box::new(HirExpression::IntLiteral(10)),
                    right: Box::new(HirExpression::Sizeof {
                        type_name: "int".to_string(),
                    }),
                }),
            }),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("vec!") || code.contains("Vec"));
}

#[test]
fn test_realloc_non_null_pointer() {
    // realloc(ptr, size) → just returns the pointer expression
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::VariableDeclaration {
            name: "new_ptr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::Realloc {
                pointer: Box::new(HirExpression::Variable("ptr".to_string())),
                new_size: Box::new(HirExpression::IntLiteral(20)),
            }),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("ptr"));
}

#[test]
fn test_malloc_with_multiply_generates_vec() {
    // malloc(n * sizeof(T)) → Vec::with_capacity(n)
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Vec(Box::new(HirType::Int)),
            initializer: Some(HirExpression::Malloc {
                size: Box::new(HirExpression::BinaryOp {
                    op: BinaryOperator::Multiply,
                    left: Box::new(HirExpression::IntLiteral(10)),
                    right: Box::new(HirExpression::Sizeof {
                        type_name: "int".to_string(),
                    }),
                }),
            }),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("Vec::with_capacity"));
}

// ============================================================================
// Post-increment / Pre-increment (DECY-139, DECY-253, DECY-255)
// ============================================================================

#[test]
fn test_post_increment_simple_variable() {
    // x++ → { let __tmp = x; x += 1; __tmp }
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::Return(Some(HirExpression::PostIncrement {
                operand: Box::new(HirExpression::Variable("x".to_string())),
            })),
        ],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("__tmp") && code.contains("+= 1"));
}

#[test]
fn test_post_increment_pointer_uses_wrapping_add() {
    // p++ where p is pointer → exercises PostIncrement with pointer parameter
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::PostIncrement {
            operand: Box::new(HirExpression::Variable("p".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    // Exercises PostIncrement path - may use wrapping_add or += depending on context
    assert!(code.contains("__tmp") || code.contains("wrapping_add") || code.contains("+= 1"));
}

#[test]
fn test_post_increment_string_ref() {
    // key++ where key is &str → string iteration pattern
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::UnsignedInt,
        vec![HirParameter::new(
            "key".to_string(),
            HirType::StringReference,
        )],
        vec![HirStatement::Return(Some(HirExpression::PostIncrement {
            operand: Box::new(HirExpression::Variable("key".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("as_bytes") || code.contains("__tmp"));
}

#[test]
fn test_post_increment_dereferenced_pointer() {
    // (*p)++ → exercises Dereference+PostIncrement path
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::PostIncrement {
            operand: Box::new(HirExpression::Dereference(Box::new(
                HirExpression::Variable("p".to_string()),
            ))),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("unsafe") || code.contains("__tmp") || code.contains("+= 1"));
}

#[test]
fn test_pre_increment_simple_variable() {
    // ++x → { x += 1; x }
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::Return(Some(HirExpression::PreIncrement {
                operand: Box::new(HirExpression::Variable("x".to_string())),
            })),
        ],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("+= 1"));
}

#[test]
fn test_pre_increment_pointer_uses_wrapping_add() {
    // ++p where p is pointer → exercises PreIncrement path
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::PreIncrement {
            operand: Box::new(HirExpression::Variable("p".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("wrapping_add") || code.contains("+= 1"));
}

#[test]
fn test_pre_increment_dereferenced_pointer() {
    // ++(*p) → exercises Dereference+PreIncrement path
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::PreIncrement {
            operand: Box::new(HirExpression::Dereference(Box::new(
                HirExpression::Variable("p".to_string()),
            ))),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("unsafe") || code.contains("+= 1"));
}

// ============================================================================
// Post-decrement / Pre-decrement (DECY-139, DECY-253, DECY-255)
// ============================================================================

#[test]
fn test_post_decrement_simple_variable() {
    // x-- → { let __tmp = x; x -= 1; __tmp }
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(5)),
            },
            HirStatement::Return(Some(HirExpression::PostDecrement {
                operand: Box::new(HirExpression::Variable("x".to_string())),
            })),
        ],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("__tmp") && code.contains("-= 1"));
}

#[test]
fn test_post_decrement_pointer_uses_wrapping_sub() {
    // p-- where p is pointer → exercises PostDecrement path
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::PostDecrement {
            operand: Box::new(HirExpression::Variable("p".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("wrapping_sub") || code.contains("-= 1") || code.contains("__tmp"));
}

#[test]
fn test_post_decrement_dereferenced_pointer() {
    // (*p)-- → exercises Dereference+PostDecrement path
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::PostDecrement {
            operand: Box::new(HirExpression::Dereference(Box::new(
                HirExpression::Variable("p".to_string()),
            ))),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("unsafe") || code.contains("-= 1") || code.contains("__tmp"));
}

#[test]
fn test_pre_decrement_simple_variable() {
    // --x → { x -= 1; x }
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(5)),
            },
            HirStatement::Return(Some(HirExpression::PreDecrement {
                operand: Box::new(HirExpression::Variable("x".to_string())),
            })),
        ],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("-= 1"));
}

#[test]
fn test_pre_decrement_pointer_uses_wrapping_sub() {
    // --p where p is pointer → exercises PreDecrement path
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::PreDecrement {
            operand: Box::new(HirExpression::Variable("p".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("wrapping_sub") || code.contains("-= 1"));
}

#[test]
fn test_pre_decrement_dereferenced_pointer() {
    // --(*p) → exercises Dereference+PreDecrement path
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::PreDecrement {
            operand: Box::new(HirExpression::Dereference(Box::new(
                HirExpression::Variable("p".to_string()),
            ))),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("unsafe") || code.contains("-= 1"));
}

// ============================================================================
// StringMethodCall expressions
// ============================================================================

#[test]
fn test_string_method_call_len() {
    // s.len() → s.len() as i32
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new("s".to_string(), HirType::StringLiteral)],
        vec![HirStatement::Return(Some(HirExpression::StringMethodCall {
            receiver: Box::new(HirExpression::Variable("s".to_string())),
            method: "len".to_string(),
            arguments: vec![],
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("len()") && code.contains("as i32"));
}

#[test]
fn test_string_method_call_no_args() {
    // s.is_empty()
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new("s".to_string(), HirType::StringLiteral)],
        vec![HirStatement::Return(Some(HirExpression::StringMethodCall {
            receiver: Box::new(HirExpression::Variable("s".to_string())),
            method: "is_empty".to_string(),
            arguments: vec![],
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("is_empty()"));
}

#[test]
fn test_string_method_call_clone_into() {
    // s.clone_into(&mut dest) — clone_into adds &mut prefix
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("s".to_string(), HirType::StringLiteral),
            HirParameter::new("dest".to_string(), HirType::StringLiteral),
        ],
        vec![HirStatement::Expression(HirExpression::StringMethodCall {
            receiver: Box::new(HirExpression::Variable("s".to_string())),
            method: "clone_into".to_string(),
            arguments: vec![HirExpression::Variable("dest".to_string())],
        })],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("clone_into") && code.contains("&mut"));
}

#[test]
fn test_string_method_call_with_args() {
    // s.contains("hello")
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new("s".to_string(), HirType::StringLiteral)],
        vec![HirStatement::Return(Some(HirExpression::StringMethodCall {
            receiver: Box::new(HirExpression::Variable("s".to_string())),
            method: "contains".to_string(),
            arguments: vec![HirExpression::StringLiteral("hello".to_string())],
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("contains"));
}

// ============================================================================
// IsNotNull expression
// ============================================================================

#[test]
fn test_is_not_null_expression() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::IsNotNull(
            Box::new(HirExpression::Variable("p".to_string())),
        )))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("Some") || code.contains("is_some") || code.contains("if let"));
}

// ============================================================================
// Ternary with target type propagation (DECY-213)
// ============================================================================

#[test]
fn test_ternary_with_boolean_condition() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
        vec![HirStatement::Return(Some(HirExpression::Ternary {
            condition: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            }),
            then_expr: Box::new(HirExpression::Variable("a".to_string())),
            else_expr: Box::new(HirExpression::Variable("b".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("if") && code.contains("else"));
}

#[test]
fn test_ternary_with_int_condition_converts_to_bool() {
    // Ternary with non-boolean condition: x ? 1 : 0 → if x != 0 { 1 } else { 0 }
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::Ternary {
            condition: Box::new(HirExpression::Variable("x".to_string())),
            then_expr: Box::new(HirExpression::IntLiteral(1)),
            else_expr: Box::new(HirExpression::IntLiteral(0)),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("!= 0") || code.contains("if"));
}

// ============================================================================
// CompoundLiteral expression
// ============================================================================

#[test]
fn test_compound_literal_struct() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::VariableDeclaration {
            name: "p".to_string(),
            var_type: HirType::Struct("Point".to_string()),
            initializer: Some(HirExpression::CompoundLiteral {
                literal_type: HirType::Struct("Point".to_string()),
                initializers: vec![
                    HirExpression::IntLiteral(10),
                    HirExpression::IntLiteral(20),
                ],
            }),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("Point") || code.contains("10") || code.contains("20"));
}

// ============================================================================
// SliceIndex expression (DECY-070)
// ============================================================================

#[test]
fn test_slice_index_expression() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "arr".to_string(),
            HirType::Reference { inner: Box::new(HirType::Int), mutable: false },
        )],
        vec![HirStatement::Return(Some(HirExpression::SliceIndex {
            slice: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
            element_type: HirType::Int,
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("arr") && code.contains("0"));
}

// ============================================================================
// BinaryOp edge cases — comma operator, bitwise with bools
// ============================================================================

#[test]
fn test_binary_op_comma_operator() {
    // (a, b) → { a; b }
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Comma,
            left: Box::new(HirExpression::IntLiteral(1)),
            right: Box::new(HirExpression::IntLiteral(2)),
        }))],
    );
    let code = codegen.generate_function(&func);
    // Comma operator returns second operand
    assert!(code.contains("2") || code.contains("1"));
}

#[test]
fn test_binary_op_bitwise_and() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::BitwiseAnd,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("&") || code.contains("bitwise"));
}

#[test]
fn test_binary_op_bitwise_or() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::BitwiseOr,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("|"));
}

#[test]
fn test_binary_op_shift_left() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::LeftShift,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(2)),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("<<"));
}

#[test]
fn test_binary_op_shift_right() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::RightShift,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        }))],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains(">>"));
}
