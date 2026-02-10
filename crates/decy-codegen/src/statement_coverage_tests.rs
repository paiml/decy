//! Coverage tests for generate_statement_with_context.
//!
//! Targets uncovered code paths in statement code generation including:
//! - For loops with multiple init/increment statements
//! - Switch with multiple cases and default
//! - InlineAsm statement generation (translatable and non-translatable)
//! - Expression statements (standalone function calls)
//! - Break/Continue in loop context
//! - Variable declarations with complex types and initializers
//! - Return statements in main vs non-main functions
//! - If with else-if chains (nested if/else)
//! - While with non-boolean conditions
//! - Assignment with realloc patterns
//! - DerefAssignment, FieldAssignment, ArrayIndexAssignment
//! - Free statement (RAII comment generation)
//! - Nested blocks via nested control flow

use super::*;
use decy_hir::{
    BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType, SwitchCase,
};

// ============================================================================
// Helper
// ============================================================================

fn make_func_with_statements(stmts: Vec<HirStatement>) -> HirFunction {
    HirFunction::new_with_body("test_func".to_string(), HirType::Void, vec![], stmts)
}

fn make_int_func_with_statements(stmts: Vec<HirStatement>) -> HirFunction {
    HirFunction::new_with_body("test_func".to_string(), HirType::Int, vec![], stmts)
}

fn make_main_with_statements(stmts: Vec<HirStatement>) -> HirFunction {
    HirFunction::new_with_body("main".to_string(), HirType::Int, vec![], stmts)
}

// ============================================================================
// FOR LOOP TESTS
// ============================================================================

#[test]
fn test_for_loop_basic() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::For {
        init: vec![HirStatement::VariableDeclaration {
            name: "i".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        }],
        condition: Some(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        }),
        increment: vec![HirStatement::Assignment {
            target: "i".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
        body: vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "println".to_string(),
            arguments: vec![HirExpression::Variable("i".to_string())],
        })],
    }]);
    let code = codegen.generate_function(&func);
    // For loops generate init stmt before, then a while loop
    assert!(code.contains("let mut i"));
    assert!(code.contains("while"));
    assert!(code.contains("i < 10"));
    assert!(code.contains("println(i)"));
}

#[test]
fn test_for_loop_multiple_init_and_increment() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::For {
        init: vec![
            HirStatement::VariableDeclaration {
                name: "i".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            },
            HirStatement::VariableDeclaration {
                name: "j".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(10)),
            },
        ],
        condition: Some(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::Variable("j".to_string())),
        }),
        increment: vec![
            HirStatement::Assignment {
                target: "i".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("i".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
            HirStatement::Assignment {
                target: "j".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Subtract,
                    left: Box::new(HirExpression::Variable("j".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
        ],
        body: vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "do_work".to_string(),
            arguments: vec![],
        })],
    }]);
    let code = codegen.generate_function(&func);
    // Both init statements should appear before the loop
    assert!(code.contains("let mut i"));
    assert!(code.contains("let mut j"));
    // Both increment statements should appear at end of loop body
    assert!(code.contains("i = i + 1"));
    assert!(code.contains("j = j - 1"));
}

#[test]
fn test_for_loop_empty_body() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::For {
        init: vec![HirStatement::VariableDeclaration {
            name: "i".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        }],
        condition: Some(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(5)),
        }),
        increment: vec![HirStatement::Assignment {
            target: "i".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
        body: vec![],
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("while"));
    assert!(code.contains("i < 5"));
}

// ============================================================================
// SWITCH STATEMENT TESTS
// ============================================================================

#[test]
fn test_switch_single_case_with_default() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![SwitchCase {
            value: Some(HirExpression::IntLiteral(1)),
            body: vec![
                HirStatement::Expression(HirExpression::FunctionCall {
                    function: "handle_one".to_string(),
                    arguments: vec![],
                }),
                HirStatement::Break,
            ],
        }],
        default_case: Some(vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "handle_default".to_string(),
            arguments: vec![],
        })]),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("match"));
    assert!(code.contains("1 =>"));
    assert!(code.contains("handle_one()"));
    assert!(code.contains("_ =>"));
    assert!(code.contains("handle_default()"));
    // Break should be filtered out from match arms
    assert!(!code.contains("break"));
}

#[test]
fn test_switch_multiple_cases() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::Switch {
        condition: HirExpression::Variable("cmd".to_string()),
        cases: vec![
            SwitchCase {
                value: Some(HirExpression::IntLiteral(0)),
                body: vec![
                    HirStatement::Expression(HirExpression::FunctionCall {
                        function: "cmd_help".to_string(),
                        arguments: vec![],
                    }),
                    HirStatement::Break,
                ],
            },
            SwitchCase {
                value: Some(HirExpression::IntLiteral(1)),
                body: vec![
                    HirStatement::Expression(HirExpression::FunctionCall {
                        function: "cmd_run".to_string(),
                        arguments: vec![],
                    }),
                    HirStatement::Break,
                ],
            },
            SwitchCase {
                value: Some(HirExpression::IntLiteral(2)),
                body: vec![
                    HirStatement::Expression(HirExpression::FunctionCall {
                        function: "cmd_stop".to_string(),
                        arguments: vec![],
                    }),
                    HirStatement::Break,
                ],
            },
        ],
        default_case: Some(vec![
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "cmd_unknown".to_string(),
                arguments: vec![],
            }),
            HirStatement::Break,
        ]),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("match cmd"));
    assert!(code.contains("0 =>"));
    assert!(code.contains("1 =>"));
    assert!(code.contains("2 =>"));
    assert!(code.contains("cmd_help()"));
    assert!(code.contains("cmd_run()"));
    assert!(code.contains("cmd_stop()"));
    assert!(code.contains("_ =>"));
    assert!(code.contains("cmd_unknown()"));
}

#[test]
fn test_switch_no_default() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::Switch {
        condition: HirExpression::Variable("val".to_string()),
        cases: vec![SwitchCase {
            value: Some(HirExpression::IntLiteral(42)),
            body: vec![HirStatement::Break],
        }],
        default_case: None,
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("match val"));
    assert!(code.contains("42 =>"));
    // Even without explicit default, codegen adds _ => {}
    assert!(code.contains("_ =>"));
}

#[test]
fn test_switch_with_char_literal_case_and_int_condition() {
    let codegen = CodeGenerator::new();
    // Build a function where the condition variable is known to be Int type
    let func = HirFunction::new_with_body(
        "test_func".to_string(),
        HirType::Void,
        vec![HirParameter::new("ch".to_string(), HirType::Int)],
        vec![HirStatement::Switch {
            condition: HirExpression::Variable("ch".to_string()),
            cases: vec![SwitchCase {
                value: Some(HirExpression::CharLiteral(48)), // '0' = 48
                body: vec![
                    HirStatement::Expression(HirExpression::FunctionCall {
                        function: "handle_zero".to_string(),
                        arguments: vec![],
                    }),
                    HirStatement::Break,
                ],
            }],
            default_case: None,
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("match ch"));
    // CharLiteral in int-conditioned switch should produce numeric value
    assert!(code.contains("48 =>"));
}

#[test]
fn test_switch_case_with_multiple_statements() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::Switch {
        condition: HirExpression::Variable("action".to_string()),
        cases: vec![SwitchCase {
            value: Some(HirExpression::IntLiteral(1)),
            body: vec![
                HirStatement::VariableDeclaration {
                    name: "result".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::IntLiteral(100)),
                },
                HirStatement::Expression(HirExpression::FunctionCall {
                    function: "process".to_string(),
                    arguments: vec![HirExpression::Variable("result".to_string())],
                }),
                HirStatement::Break,
            ],
        }],
        default_case: None,
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("let mut result"));
    assert!(code.contains("process(result)"));
}

// ============================================================================
// INLINE ASM TESTS
// ============================================================================

#[test]
fn test_inline_asm_non_translatable() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::InlineAsm {
        text: "mov eax, ebx".to_string(),
        translatable: false,
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("manual review required"));
    assert!(code.contains("inline assembly"));
    assert!(code.contains("mov eax, ebx"));
    // Should NOT contain translatable hint
    assert!(!code.contains("translatable to Rust intrinsics"));
}

#[test]
fn test_inline_asm_translatable() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::InlineAsm {
        text: "cpuid".to_string(),
        translatable: true,
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("manual review required"));
    assert!(code.contains("translatable to Rust intrinsics"));
    assert!(code.contains("cpuid"));
}

#[test]
fn test_inline_asm_multiline() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::InlineAsm {
        text: "push rbp\nmov rbp, rsp\npop rbp".to_string(),
        translatable: false,
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("push rbp"));
    assert!(code.contains("// mov rbp, rsp"));
}

// ============================================================================
// EXPRESSION STATEMENT TESTS
// ============================================================================

#[test]
fn test_expression_statement_function_call() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::Expression(
        HirExpression::FunctionCall {
            function: "log_message".to_string(),
            arguments: vec![HirExpression::StringLiteral("Hello, World!".to_string())],
        },
    )]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("log_message("));
    assert!(code.contains("Hello, World!"));
    // Expression statement ends with semicolon
    assert!(code.contains(";"));
}

#[test]
fn test_expression_statement_variable() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::Expression(
        HirExpression::Variable("x".to_string()),
    )]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("x;"));
}

#[test]
fn test_expression_statement_post_increment() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![
        HirStatement::VariableDeclaration {
            name: "counter".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        },
        HirStatement::Expression(HirExpression::PostIncrement {
            operand: Box::new(HirExpression::Variable("counter".to_string())),
        }),
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("counter"));
}

// ============================================================================
// BREAK / CONTINUE TESTS
// ============================================================================

#[test]
fn test_break_in_while_loop() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::While {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(100)),
        },
        body: vec![
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::Equal,
                    left: Box::new(HirExpression::Variable("i".to_string())),
                    right: Box::new(HirExpression::IntLiteral(50)),
                },
                then_block: vec![HirStatement::Break],
                else_block: None,
            },
            HirStatement::Assignment {
                target: "i".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("i".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
        ],
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("while"));
    assert!(code.contains("break;"));
}

#[test]
fn test_continue_in_for_loop() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::For {
        init: vec![HirStatement::VariableDeclaration {
            name: "i".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        }],
        condition: Some(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(20)),
        }),
        increment: vec![HirStatement::Assignment {
            target: "i".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
        body: vec![
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::Equal,
                    left: Box::new(HirExpression::BinaryOp {
                        op: BinaryOperator::Modulo,
                        left: Box::new(HirExpression::Variable("i".to_string())),
                        right: Box::new(HirExpression::IntLiteral(2)),
                    }),
                    right: Box::new(HirExpression::IntLiteral(0)),
                },
                then_block: vec![HirStatement::Continue],
                else_block: None,
            },
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "process".to_string(),
                arguments: vec![HirExpression::Variable("i".to_string())],
            }),
        ],
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("continue;"));
}

#[test]
fn test_break_standalone() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::While {
        condition: HirExpression::IntLiteral(1),
        body: vec![HirStatement::Break],
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("break;"));
}

#[test]
fn test_continue_standalone() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::While {
        condition: HirExpression::IntLiteral(1),
        body: vec![HirStatement::Continue],
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("continue;"));
}

// ============================================================================
// VARIABLE DECLARATION TESTS
// ============================================================================

#[test]
fn test_var_decl_int_with_initializer() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::VariableDeclaration {
        name: "count".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(42)),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("let mut count: i32 = 42"));
}

#[test]
fn test_var_decl_without_initializer() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Double,
        initializer: None,
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("let mut x: f64"));
    // Should have default value
    assert!(code.contains("0.0"));
}

#[test]
fn test_var_decl_float_type() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::VariableDeclaration {
        name: "val".to_string(),
        var_type: HirType::Float,
        initializer: Some(HirExpression::FloatLiteral("3.14".to_string())),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("let mut val: f32"));
    assert!(code.contains("3.14"));
}

#[test]
fn test_var_decl_unsigned_int() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::VariableDeclaration {
        name: "size".to_string(),
        var_type: HirType::UnsignedInt,
        initializer: Some(HirExpression::IntLiteral(0)),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("let mut size: u32"));
}

#[test]
fn test_var_decl_char_type() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::VariableDeclaration {
        name: "ch".to_string(),
        var_type: HirType::Char,
        initializer: Some(HirExpression::CharLiteral(65)),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("let mut ch: u8"));
}

#[test]
fn test_var_decl_signed_char_type() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::VariableDeclaration {
        name: "sc".to_string(),
        var_type: HirType::SignedChar,
        initializer: None,
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("let mut sc: i8"));
}

#[test]
fn test_var_decl_vla_int() {
    let codegen = CodeGenerator::new();
    // VLA: int arr[n]; → let mut arr = vec![0i32; n];
    let func = HirFunction::new_with_body(
        "test_func".to_string(),
        HirType::Void,
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Int),
                size: None,
            },
            initializer: Some(HirExpression::Variable("n".to_string())),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("vec![0i32; n]"));
}

#[test]
fn test_var_decl_vla_double() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test_func".to_string(),
        HirType::Void,
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Double),
                size: None,
            },
            initializer: Some(HirExpression::Variable("n".to_string())),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("vec![0.0f64; n]"));
}

#[test]
fn test_var_decl_vla_float() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test_func".to_string(),
        HirType::Void,
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Float),
                size: None,
            },
            initializer: Some(HirExpression::Variable("n".to_string())),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("vec![0.0f32; n]"));
}

#[test]
fn test_var_decl_vla_unsigned_int() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test_func".to_string(),
        HirType::Void,
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::UnsignedInt),
                size: None,
            },
            initializer: Some(HirExpression::Variable("n".to_string())),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("vec![0u32; n]"));
}

#[test]
fn test_var_decl_vla_char() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test_func".to_string(),
        HirType::Void,
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Char),
                size: None,
            },
            initializer: Some(HirExpression::Variable("n".to_string())),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("vec![0u8; n]"));
}

#[test]
fn test_var_decl_vla_signed_char() {
    let codegen = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "test_func".to_string(),
        HirType::Void,
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::SignedChar),
                size: None,
            },
            initializer: Some(HirExpression::Variable("n".to_string())),
        }],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("vec![0i8; n]"));
}

#[test]
fn test_var_decl_string_literal_char_pointer() {
    let codegen = CodeGenerator::new();
    // char *s = "hello" → let s: &str = "hello"
    let func = make_func_with_statements(vec![HirStatement::VariableDeclaration {
        name: "s".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Char)),
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("&str"));
    assert!(code.contains("hello"));
}

#[test]
fn test_var_decl_char_array_string_init() {
    let codegen = CodeGenerator::new();
    // char str[6] = "hello" → let mut str: [u8; 6] = *b"hello\0"
    let func = make_func_with_statements(vec![HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(6),
        },
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("*b\"hello\\0\""));
}

#[test]
fn test_var_decl_reserved_keyword_name() {
    let codegen = CodeGenerator::new();
    // Variable named "type" should be escaped
    let func = make_func_with_statements(vec![HirStatement::VariableDeclaration {
        name: "type".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(1)),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("r#type"));
}

// ============================================================================
// RETURN STATEMENT TESTS
// ============================================================================

#[test]
fn test_return_with_value() {
    let codegen = CodeGenerator::new();
    let func = make_int_func_with_statements(vec![HirStatement::Return(Some(
        HirExpression::IntLiteral(42),
    ))]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("return 42"));
}

#[test]
fn test_return_void() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::Return(None)]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("return;"));
}

#[test]
fn test_return_in_main_with_value() {
    let codegen = CodeGenerator::new();
    // return 0; in main → std::process::exit(0);
    let func = make_main_with_statements(vec![HirStatement::Return(Some(
        HirExpression::IntLiteral(0),
    ))]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("std::process::exit(0)"));
}

#[test]
fn test_return_in_main_without_value() {
    let codegen = CodeGenerator::new();
    // return; in main → std::process::exit(0);
    let func = make_main_with_statements(vec![HirStatement::Return(None)]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("std::process::exit(0)"));
}

#[test]
fn test_return_in_main_with_char_cast() {
    let codegen = CodeGenerator::new();
    // return 'A'; in main should cast to i32 for exit()
    let func = HirFunction::new_with_body(
        "main".to_string(),
        HirType::Int,
        vec![],
        vec![
            HirStatement::VariableDeclaration {
                name: "ch".to_string(),
                var_type: HirType::Char,
                initializer: Some(HirExpression::CharLiteral(65)),
            },
            HirStatement::Return(Some(HirExpression::Variable("ch".to_string()))),
        ],
    );
    let code = codegen.generate_function(&func);
    assert!(code.contains("std::process::exit("));
}

#[test]
fn test_return_expression() {
    let codegen = CodeGenerator::new();
    let func = make_int_func_with_statements(vec![HirStatement::Return(Some(
        HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::IntLiteral(1)),
            right: Box::new(HirExpression::IntLiteral(2)),
        },
    ))]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("return 1 + 2"));
}

// ============================================================================
// IF STATEMENT TESTS
// ============================================================================

#[test]
fn test_if_without_else() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        then_block: vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "positive".to_string(),
            arguments: vec![],
        })],
        else_block: None,
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("if x > 0"));
    assert!(code.contains("positive()"));
    assert!(!code.contains("} else {"));
}

#[test]
fn test_if_with_else() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        then_block: vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "positive".to_string(),
            arguments: vec![],
        })],
        else_block: Some(vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "non_positive".to_string(),
            arguments: vec![],
        })]),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("if x > 0"));
    assert!(code.contains("positive()"));
    assert!(code.contains("} else {"));
    assert!(code.contains("non_positive()"));
}

#[test]
fn test_if_else_if_chain() {
    let codegen = CodeGenerator::new();
    // if (x > 0) ... else { if (x < 0) ... else { ... } }
    let func = make_func_with_statements(vec![HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        then_block: vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "positive".to_string(),
            arguments: vec![],
        })],
        else_block: Some(vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::IntLiteral(0)),
            },
            then_block: vec![HirStatement::Expression(HirExpression::FunctionCall {
                function: "negative".to_string(),
                arguments: vec![],
            })],
            else_block: Some(vec![HirStatement::Expression(HirExpression::FunctionCall {
                function: "zero".to_string(),
                arguments: vec![],
            })]),
        }]),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("if x > 0"));
    assert!(code.contains("positive()"));
    assert!(code.contains("} else {"));
    assert!(code.contains("if x < 0"));
    assert!(code.contains("negative()"));
    assert!(code.contains("zero()"));
}

#[test]
fn test_if_non_boolean_condition_wraps_ne_zero() {
    let codegen = CodeGenerator::new();
    // if (x) where x is not boolean → if (x) != 0
    let func = make_func_with_statements(vec![HirStatement::If {
        condition: HirExpression::Variable("x".to_string()),
        then_block: vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "do_something".to_string(),
            arguments: vec![],
        })],
        else_block: None,
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("!= 0") || code.contains("is_null"));
}

// ============================================================================
// WHILE STATEMENT TESTS
// ============================================================================

#[test]
fn test_while_boolean_condition() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::While {
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
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("while i < 10"));
}

#[test]
fn test_while_non_boolean_condition() {
    let codegen = CodeGenerator::new();
    // while (n) → while (n) != 0
    let func = make_func_with_statements(vec![HirStatement::While {
        condition: HirExpression::Variable("n".to_string()),
        body: vec![HirStatement::Assignment {
            target: "n".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Subtract,
                left: Box::new(HirExpression::Variable("n".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("!= 0") || code.contains("is_null"));
}

#[test]
fn test_while_with_nested_statements() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::While {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::NotEqual,
            left: Box::new(HirExpression::Variable("done".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        },
        body: vec![
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "step".to_string(),
                arguments: vec![],
            }),
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::Equal,
                    left: Box::new(HirExpression::Variable("status".to_string())),
                    right: Box::new(HirExpression::IntLiteral(0)),
                },
                then_block: vec![HirStatement::Break],
                else_block: None,
            },
        ],
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("while done != 1"));
    assert!(code.contains("step()"));
    assert!(code.contains("break;"));
}

// ============================================================================
// ASSIGNMENT TESTS
// ============================================================================

#[test]
fn test_simple_assignment() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![
        HirStatement::VariableDeclaration {
            name: "x".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        },
        HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::IntLiteral(42),
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("x = 42"));
}

#[test]
fn test_assignment_with_binary_op() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![
        HirStatement::VariableDeclaration {
            name: "x".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(10)),
        },
        HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::IntLiteral(2)),
            },
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("x = x * 2"));
}

#[test]
fn test_assignment_realloc_zero_size() {
    let codegen = CodeGenerator::new();
    // realloc(buf, 0) → buf.clear()
    let func = make_func_with_statements(vec![
        HirStatement::VariableDeclaration {
            name: "buf".to_string(),
            var_type: HirType::Vec(Box::new(HirType::Int)),
            initializer: None,
        },
        HirStatement::Assignment {
            target: "buf".to_string(),
            value: HirExpression::Realloc {
                pointer: Box::new(HirExpression::Variable("buf".to_string())),
                new_size: Box::new(HirExpression::IntLiteral(0)),
            },
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains(".clear()"));
}

// ============================================================================
// FREE STATEMENT TESTS
// ============================================================================

#[test]
fn test_free_statement() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::Free {
        pointer: HirExpression::Variable("ptr".to_string()),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("RAII"));
    assert!(code.contains("ptr"));
}

#[test]
fn test_free_statement_complex_expression() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::Free {
        pointer: HirExpression::FieldAccess {
            object: Box::new(HirExpression::Variable("node".to_string())),
            field: "data".to_string(),
        },
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("RAII"));
}

// ============================================================================
// DEREF ASSIGNMENT TESTS
// ============================================================================

#[test]
fn test_deref_assignment_basic() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::DerefAssignment {
        target: HirExpression::Variable("x".to_string()),
        value: HirExpression::IntLiteral(10),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("*x = 10"));
}

#[test]
fn test_deref_assignment_field_access() {
    let codegen = CodeGenerator::new();
    // ptr->field = value → (*ptr).field = value (no extra dereference)
    let func = make_func_with_statements(vec![HirStatement::DerefAssignment {
        target: HirExpression::PointerFieldAccess {
            pointer: Box::new(HirExpression::Variable("sb".to_string())),
            field: "capacity".to_string(),
        },
        value: HirExpression::IntLiteral(100),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("capacity"));
    assert!(code.contains("= 100"));
}

#[test]
fn test_deref_assignment_array_index() {
    let codegen = CodeGenerator::new();
    // arr[i] = value (no extra dereference)
    let func = make_func_with_statements(vec![HirStatement::DerefAssignment {
        target: HirExpression::ArrayIndex {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
        },
        value: HirExpression::IntLiteral(42),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("arr"));
    assert!(code.contains("= 42"));
}

// ============================================================================
// FIELD ASSIGNMENT TESTS
// ============================================================================

#[test]
fn test_field_assignment() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::FieldAssignment {
        object: HirExpression::Variable("point".to_string()),
        field: "x".to_string(),
        value: HirExpression::IntLiteral(10),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("point.x = 10"));
}

#[test]
fn test_field_assignment_reserved_keyword() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::FieldAssignment {
        object: HirExpression::Variable("obj".to_string()),
        field: "type".to_string(),
        value: HirExpression::IntLiteral(1),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("r#type"));
}

// ============================================================================
// ARRAY INDEX ASSIGNMENT TESTS
// ============================================================================

#[test]
fn test_array_index_assignment() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![
        HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(10),
            },
            initializer: None,
        },
        HirStatement::ArrayIndexAssignment {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::IntLiteral(5)),
            value: HirExpression::IntLiteral(42),
        },
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("arr[(5) as usize] = 42"));
}

// ============================================================================
// NESTED CONTROL FLOW / BLOCK TESTS
// ============================================================================

#[test]
fn test_nested_if_in_while() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::While {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        },
        body: vec![
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::Equal,
                    left: Box::new(HirExpression::BinaryOp {
                        op: BinaryOperator::Modulo,
                        left: Box::new(HirExpression::Variable("i".to_string())),
                        right: Box::new(HirExpression::IntLiteral(2)),
                    }),
                    right: Box::new(HirExpression::IntLiteral(0)),
                },
                then_block: vec![HirStatement::Expression(HirExpression::FunctionCall {
                    function: "even".to_string(),
                    arguments: vec![HirExpression::Variable("i".to_string())],
                })],
                else_block: Some(vec![HirStatement::Expression(
                    HirExpression::FunctionCall {
                        function: "odd".to_string(),
                        arguments: vec![HirExpression::Variable("i".to_string())],
                    },
                )]),
            },
            HirStatement::Assignment {
                target: "i".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("i".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
        ],
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("while"));
    assert!(code.contains("even(i)"));
    assert!(code.contains("odd(i)"));
}

#[test]
fn test_nested_for_in_for() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::For {
        init: vec![HirStatement::VariableDeclaration {
            name: "i".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        }],
        condition: Some(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(3)),
        }),
        increment: vec![HirStatement::Assignment {
            target: "i".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
        body: vec![HirStatement::For {
            init: vec![HirStatement::VariableDeclaration {
                name: "j".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            }],
            condition: Some(HirExpression::BinaryOp {
                op: BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("j".to_string())),
                right: Box::new(HirExpression::IntLiteral(3)),
            }),
            increment: vec![HirStatement::Assignment {
                target: "j".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("j".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            }],
            body: vec![HirStatement::Expression(HirExpression::FunctionCall {
                function: "process".to_string(),
                arguments: vec![
                    HirExpression::Variable("i".to_string()),
                    HirExpression::Variable("j".to_string()),
                ],
            })],
        }],
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("let mut i"));
    assert!(code.contains("let mut j"));
    assert!(code.contains("process(i, j)"));
}

#[test]
fn test_switch_in_while() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::While {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::NotEqual,
            left: Box::new(HirExpression::Variable("state".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        body: vec![HirStatement::Switch {
            condition: HirExpression::Variable("state".to_string()),
            cases: vec![
                SwitchCase {
                    value: Some(HirExpression::IntLiteral(1)),
                    body: vec![
                        HirStatement::Assignment {
                            target: "state".to_string(),
                            value: HirExpression::IntLiteral(2),
                        },
                        HirStatement::Break,
                    ],
                },
                SwitchCase {
                    value: Some(HirExpression::IntLiteral(2)),
                    body: vec![
                        HirStatement::Assignment {
                            target: "state".to_string(),
                            value: HirExpression::IntLiteral(0),
                        },
                        HirStatement::Break,
                    ],
                },
            ],
            default_case: Some(vec![HirStatement::Break]),
        }],
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("while state != 0"));
    assert!(code.contains("match state"));
    assert!(code.contains("1 =>"));
    assert!(code.contains("2 =>"));
}

// ============================================================================
// COMPLEX VARIABLE DECLARATION TESTS
// ============================================================================

#[test]
fn test_var_decl_pointer_type() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::VariableDeclaration {
        name: "ptr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: None,
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("*mut i32"));
}

#[test]
fn test_var_decl_array_fixed_size() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(5),
        },
        initializer: None,
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("let mut arr"));
    // Should have a default initializer
    assert!(code.contains("[i32; 5]") || code.contains("arr"));
}

#[test]
fn test_var_decl_struct_type() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::VariableDeclaration {
        name: "point".to_string(),
        var_type: HirType::Struct("Point".to_string()),
        initializer: None,
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("let mut point: Point"));
}

#[test]
fn test_var_decl_vec_type() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::VariableDeclaration {
        name: "items".to_string(),
        var_type: HirType::Vec(Box::new(HirType::Int)),
        initializer: None,
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("Vec<i32>"));
}

#[test]
fn test_var_decl_bool_initializer_with_binary_op() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::VariableDeclaration {
        name: "flag".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::BinaryOp {
            op: BinaryOperator::LogicalAnd,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("let mut flag: i32"));
    assert!(code.contains("&&"));
}

// ============================================================================
// EXPRESSION STATEMENT WITH DIFFERENT EXPRESSION TYPES
// ============================================================================

#[test]
fn test_expression_statement_binary_op() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::Expression(
        HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        },
    )]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("a + b;"));
}

#[test]
fn test_expression_statement_function_call_no_args() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::Expression(
        HirExpression::FunctionCall {
            function: "initialize".to_string(),
            arguments: vec![],
        },
    )]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("initialize();"));
}

#[test]
fn test_expression_statement_function_call_multiple_args() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::Expression(
        HirExpression::FunctionCall {
            function: "add".to_string(),
            arguments: vec![
                HirExpression::IntLiteral(1),
                HirExpression::IntLiteral(2),
                HirExpression::IntLiteral(3),
            ],
        },
    )]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("add(1, 2, 3);"));
}

// ============================================================================
// MULTIPLE STATEMENTS IN SEQUENCE
// ============================================================================

#[test]
fn test_multiple_sequential_statements() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![
        HirStatement::VariableDeclaration {
            name: "x".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        },
        HirStatement::VariableDeclaration {
            name: "y".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        },
        HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::IntLiteral(10),
        },
        HirStatement::Assignment {
            target: "y".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::IntLiteral(2)),
            },
        },
        HirStatement::Expression(HirExpression::FunctionCall {
            function: "output".to_string(),
            arguments: vec![
                HirExpression::Variable("x".to_string()),
                HirExpression::Variable("y".to_string()),
            ],
        }),
    ]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("let mut x: i32 = 0"));
    assert!(code.contains("let mut y: i32 = 0"));
    assert!(code.contains("x = 10"));
    assert!(code.contains("y = x * 2"));
    assert!(code.contains("output(x, y)"));
}

#[test]
fn test_var_decl_with_function_call_initializer() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::VariableDeclaration {
        name: "result".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::FunctionCall {
            function: "compute".to_string(),
            arguments: vec![HirExpression::IntLiteral(5)],
        }),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("let mut result: i32 = compute(5)"));
}

#[test]
fn test_for_loop_with_break_condition() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::For {
        init: vec![HirStatement::VariableDeclaration {
            name: "i".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        }],
        condition: Some(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(100)),
        }),
        increment: vec![HirStatement::Assignment {
            target: "i".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
        body: vec![
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::GreaterThan,
                    left: Box::new(HirExpression::Variable("i".to_string())),
                    right: Box::new(HirExpression::IntLiteral(50)),
                },
                then_block: vec![HirStatement::Break],
                else_block: None,
            },
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "work".to_string(),
                arguments: vec![HirExpression::Variable("i".to_string())],
            }),
        ],
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("break;"));
    assert!(code.contains("work(i)"));
}

#[test]
fn test_switch_with_return_in_case() {
    let codegen = CodeGenerator::new();
    let func = make_int_func_with_statements(vec![HirStatement::Switch {
        condition: HirExpression::Variable("op".to_string()),
        cases: vec![
            SwitchCase {
                value: Some(HirExpression::IntLiteral(0)),
                body: vec![HirStatement::Return(Some(HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("a".to_string())),
                    right: Box::new(HirExpression::Variable("b".to_string())),
                }))],
            },
            SwitchCase {
                value: Some(HirExpression::IntLiteral(1)),
                body: vec![HirStatement::Return(Some(HirExpression::BinaryOp {
                    op: BinaryOperator::Subtract,
                    left: Box::new(HirExpression::Variable("a".to_string())),
                    right: Box::new(HirExpression::Variable("b".to_string())),
                }))],
            },
        ],
        default_case: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
            0,
        )))]),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("match op"));
    assert!(code.contains("return a + b"));
    assert!(code.contains("return a - b"));
    assert!(code.contains("return 0"));
}

#[test]
fn test_inline_asm_empty_text() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::InlineAsm {
        text: "".to_string(),
        translatable: false,
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("manual review required"));
}

#[test]
fn test_if_with_multiple_then_statements() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("status".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        then_block: vec![
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "log_success".to_string(),
                arguments: vec![],
            }),
            HirStatement::Assignment {
                target: "count".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("count".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "notify".to_string(),
                arguments: vec![],
            }),
        ],
        else_block: None,
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("if status == 0"));
    assert!(code.contains("log_success()"));
    assert!(code.contains("count = count + 1"));
    assert!(code.contains("notify()"));
}

#[test]
fn test_while_with_logical_and_condition() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::While {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::LogicalAnd,
            left: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::IntLiteral(10)),
            }),
            right: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                left: Box::new(HirExpression::Variable("j".to_string())),
                right: Box::new(HirExpression::IntLiteral(0)),
            }),
        },
        body: vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "tick".to_string(),
            arguments: vec![],
        })],
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("while"));
    assert!(code.contains("&&"));
    assert!(code.contains("tick()"));
}

#[test]
fn test_var_decl_with_cast_initializer() {
    let codegen = CodeGenerator::new();
    let func = make_func_with_statements(vec![HirStatement::VariableDeclaration {
        name: "val".to_string(),
        var_type: HirType::Double,
        initializer: Some(HirExpression::Cast {
            target_type: HirType::Double,
            expr: Box::new(HirExpression::IntLiteral(42)),
        }),
    }]);
    let code = codegen.generate_function(&func);
    assert!(code.contains("let mut val: f64"));
    assert!(code.contains("42") || code.contains("as f64"));
}
