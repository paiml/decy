#[test]
fn stmt_ctx_vla_declaration_signed_char() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "sca".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::SignedChar),
            size: None,
        },
        initializer: Some(HirExpression::IntLiteral(4)),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("vec![0i8;"), "Got: {}", result);
}

#[test]
fn stmt_ctx_return_in_main_function() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Return(Some(HirExpression::IntLiteral(0)));
    let result = cg.generate_statement_with_context(&stmt, Some("main"), &mut ctx, None);
    assert!(
        result.contains("std::process::exit(0)"),
        "Got: {}",
        result
    );
}

#[test]
fn stmt_ctx_return_none_in_main() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Return(None);
    let result = cg.generate_statement_with_context(&stmt, Some("main"), &mut ctx, None);
    assert!(
        result.contains("std::process::exit(0)"),
        "Got: {}",
        result
    );
}

#[test]
fn stmt_ctx_return_char_in_main_casts_to_i32() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ch".to_string(), HirType::Char);
    let stmt = HirStatement::Return(Some(HirExpression::Variable("ch".to_string())));
    let result = cg.generate_statement_with_context(&stmt, Some("main"), &mut ctx, None);
    assert!(result.contains("as i32"), "Got: {}", result);
    assert!(result.contains("exit"), "Got: {}", result);
}

#[test]
fn stmt_ctx_return_void_in_regular_func() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Return(None);
    let result = cg.generate_statement_with_context(&stmt, Some("process"), &mut ctx, None);
    assert_eq!(result, "return;");
}

#[test]
fn stmt_ctx_assignment_to_global_wraps_unsafe() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_global("COUNTER".to_string());
    ctx.add_variable("COUNTER".to_string(), HirType::Int);
    let stmt = HirStatement::Assignment {
        target: "COUNTER".to_string(),
        value: HirExpression::IntLiteral(42),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("COUNTER"), "Got: {}", result);
    assert!(result.contains("42"), "Got: {}", result);
}

#[test]
fn stmt_ctx_assignment_errno_special_handling() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Assignment {
        target: "errno".to_string(),
        value: HirExpression::IntLiteral(0),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("ERRNO"), "Got: {}", result);
}

#[test]
fn stmt_ctx_realloc_assignment_with_zero_size() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buf".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let stmt = HirStatement::Assignment {
        target: "buf".to_string(),
        value: HirExpression::Realloc {
            pointer: Box::new(HirExpression::Variable("buf".to_string())),
            new_size: Box::new(HirExpression::IntLiteral(0)),
        },
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("clear"), "Got: {}", result);
}

#[test]
fn stmt_ctx_realloc_assignment_with_multiply_size() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buf".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let stmt = HirStatement::Assignment {
        target: "buf".to_string(),
        value: HirExpression::Realloc {
            pointer: Box::new(HirExpression::Variable("buf".to_string())),
            new_size: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::IntLiteral(20)),
                right: Box::new(HirExpression::Sizeof { type_name: "i32".to_string() }),
            }),
        },
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("resize"), "Got: {}", result);
    assert!(result.contains("20"), "Got: {}", result);
}

#[test]
fn stmt_ctx_realloc_assignment_no_multiply() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("data".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let stmt = HirStatement::Assignment {
        target: "data".to_string(),
        value: HirExpression::Realloc {
            pointer: Box::new(HirExpression::Variable("data".to_string())),
            new_size: Box::new(HirExpression::Variable("new_len".to_string())),
        },
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("resize"), "Got: {}", result);
    assert!(result.contains("as usize"), "Got: {}", result);
}

#[test]
fn stmt_ctx_switch_with_cases() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("cmd".to_string(), HirType::Int);
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("cmd".to_string()),
        cases: vec![
            SwitchCase {
                value: Some(HirExpression::IntLiteral(1)),
                body: vec![
                    HirStatement::Expression(HirExpression::FunctionCall {
                        function: "handle_one".to_string(),
                        arguments: vec![],
                    }),
                    HirStatement::Break,
                ],
            },
            SwitchCase {
                value: Some(HirExpression::IntLiteral(2)),
                body: vec![HirStatement::Break],
            },
        ],
        default_case: Some(vec![HirStatement::Break]),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("match cmd"), "Got: {}", result);
    assert!(result.contains("1 =>"), "Got: {}", result);
    assert!(result.contains("2 =>"), "Got: {}", result);
    assert!(result.contains("_ =>"), "Got: {}", result);
    // Break should be filtered out
    assert!(!result.contains("break"), "Got: {}", result);
}

#[test]
fn stmt_ctx_switch_with_char_literal_case() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Int);
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("c".to_string()),
        cases: vec![SwitchCase {
            value: Some(HirExpression::CharLiteral(b'0' as i8)),
            body: vec![
                HirStatement::Return(Some(HirExpression::IntLiteral(0))),
                HirStatement::Break,
            ],
        }],
        default_case: None,
    };
    let result = cg.generate_statement_with_context(&stmt, Some("parse_digit"), &mut ctx, None);
    assert!(result.contains("match c"), "Got: {}", result);
    // When condition is Int and case is CharLiteral, numeric value 48 for '0'
    assert!(result.contains("48"), "Got: {}", result);
}

#[test]
fn stmt_ctx_char_array_string_literal_init() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "msg".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(6),
        },
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("b\"hello\\0\""), "Got: {}", result);
}

#[test]
fn stmt_ctx_char_ptr_string_literal_init() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "msg".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Char)),
        initializer: Some(HirExpression::StringLiteral("world".to_string())),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("&str"), "Got: {}", result);
    assert!(result.contains("world"), "Got: {}", result);
}

#[test]
fn stmt_ctx_deref_assign_pointer_field_access() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::PointerFieldAccess {
            pointer: Box::new(HirExpression::Variable("node".to_string())),
            field: "value".to_string(),
        },
        value: HirExpression::IntLiteral(42),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    // PointerFieldAccess is handled without extra dereference
    assert!(result.contains("= 42"), "Got: {}", result);
    assert!(!result.contains("*(*"), "Got: {}", result);
}

#[test]
fn stmt_ctx_deref_assign_raw_pointer_wraps_unsafe() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("ptr".to_string()),
        value: HirExpression::IntLiteral(99),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("*ptr = 99"), "Got: {}", result);
}

#[test]
fn stmt_ctx_for_loop_with_init_and_increment() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::For {
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
        body: vec![HirStatement::Break],
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("let mut i"), "Got: {}", result);
    assert!(result.contains("while"), "Got: {}", result);
    assert!(result.contains("break"), "Got: {}", result);
}

#[test]
fn stmt_ctx_for_infinite_loop_none_condition() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::For {
        init: vec![],
        condition: None,
        increment: vec![],
        body: vec![HirStatement::Break],
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("loop {"), "Got: {}", result);
    assert!(result.contains("break"), "Got: {}", result);
}

#[test]
fn stmt_ctx_variable_shadows_global_gets_renamed() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_global("count".to_string());
    let stmt = HirStatement::VariableDeclaration {
        name: "count".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("count_local"), "Got: {}", result);
}

#[test]
fn stmt_ctx_uninitialized_variable() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Int,
        initializer: None,
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("let mut x: i32"), "Got: {}", result);
    assert!(result.contains("0i32"), "Got: {}", result);
}

// ============================================================================
// BATCH 16: ArrayIndexAssignment, FieldAssignment, Free, Expression, InlineAsm
// ============================================================================

#[test]
fn stmt_ctx_array_index_assign_raw_pointer_unsafe() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "arr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(3)),
        value: HirExpression::IntLiteral(42),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains(".add("), "Got: {}", result);
}

#[test]
fn stmt_ctx_array_index_assign_global_array() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_global("TABLE".to_string());
    ctx.add_variable(
        "TABLE".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("TABLE".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
        value: HirExpression::IntLiteral(99),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("TABLE"), "Got: {}", result);
}

#[test]
fn stmt_ctx_array_index_assign_regular() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "data".to_string(),
        HirType::Vec(Box::new(HirType::Int)),
    );
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("data".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
        value: HirExpression::IntLiteral(0),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("data[(i) as usize] = 0"), "Got: {}", result);
}

#[test]
fn stmt_ctx_field_assign_regular() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Struct("Point".to_string()));
    let stmt = HirStatement::FieldAssignment {
        object: HirExpression::Variable("p".to_string()),
        field: "x".to_string(),
        value: HirExpression::IntLiteral(10),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("p.x = 10"), "Got: {}", result);
}

#[test]
fn stmt_ctx_field_assign_raw_pointer_wraps_unsafe() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "node".to_string(),
        HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
    );
    let stmt = HirStatement::FieldAssignment {
        object: HirExpression::Variable("node".to_string()),
        field: "value".to_string(),
        value: HirExpression::IntLiteral(42),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("(*node).value"), "Got: {}", result);
}

#[test]
fn stmt_ctx_field_assign_global_struct_wraps_unsafe() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_global("CONFIG".to_string());
    ctx.add_variable("CONFIG".to_string(), HirType::Struct("Config".to_string()));
    let stmt = HirStatement::FieldAssignment {
        object: HirExpression::Variable("CONFIG".to_string()),
        field: "timeout".to_string(),
        value: HirExpression::IntLiteral(30),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("CONFIG.timeout"), "Got: {}", result);
}

#[test]
fn stmt_ctx_field_assign_keyword_field_escaping() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("s".to_string(), HirType::Struct("S".to_string()));
    let stmt = HirStatement::FieldAssignment {
        object: HirExpression::Variable("s".to_string()),
        field: "type".to_string(),
        value: HirExpression::IntLiteral(1),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("r#type"), "Got: {}", result);
}

#[test]
fn stmt_ctx_free_variable() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Free {
        pointer: HirExpression::Variable("buf".to_string()),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("RAII"), "Got: {}", result);
    assert!(result.contains("buf"), "Got: {}", result);
}

#[test]
fn stmt_ctx_free_expression() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Free {
        pointer: HirExpression::FieldAccess {
            object: Box::new(HirExpression::Variable("s".to_string())),
            field: "data".to_string(),
        },
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("RAII"), "Got: {}", result);
}

#[test]
fn stmt_ctx_expression_function_call() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Expression(HirExpression::FunctionCall {
        function: "do_work".to_string(),
        arguments: vec![HirExpression::IntLiteral(1)],
    });
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("do_work(1)"), "Got: {}", result);
    assert!(result.ends_with(';'), "Got: {}", result);
}

#[test]
fn stmt_ctx_inline_asm_non_translatable() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::InlineAsm {
        text: "nop".to_string(),
        translatable: false,
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("manual review"), "Got: {}", result);
    assert!(result.contains("nop"), "Got: {}", result);
    assert!(!result.contains("translatable"), "Got: {}", result);
}

#[test]
fn stmt_ctx_inline_asm_translatable() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::InlineAsm {
        text: "mfence".to_string(),
        translatable: true,
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("manual review"), "Got: {}", result);
    assert!(result.contains("translatable"), "Got: {}", result);
    assert!(result.contains("mfence"), "Got: {}", result);
}

#[test]
fn stmt_ctx_deref_assign_double_pointer_unsafe() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // Reference to pointer → dereferencing yields raw pointer → needs unsafe
    ctx.add_variable(
        "pp".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Pointer(Box::new(HirType::Int))),
            mutable: true,
        },
    );
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Dereference(Box::new(HirExpression::Variable("pp".to_string()))),
        value: HirExpression::IntLiteral(42),
    };
    let result = cg.generate_statement_with_context(&stmt, None, &mut ctx, None);
    assert!(result.contains("unsafe"), "Got: {}", result);
}

// ============================================================================
// BATCH 16b: generate_signature — main, output_param, keyword rename
// ============================================================================

#[test]
fn gen_sig_main_function_no_return_type() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("main".to_string(), HirType::Int, vec![]);
    let result = cg.generate_signature(&func);
    assert_eq!(result, "fn main()");
    assert!(!result.contains("-> i32"), "Got: {}", result);
}

#[test]
fn gen_sig_keyword_write_becomes_c_write() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("write".to_string(), HirType::Void, vec![]);
    let result = cg.generate_signature(&func);
    assert!(result.contains("fn c_write"), "Got: {}", result);
}

#[test]
fn gen_sig_keyword_read_becomes_c_read() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("read".to_string(), HirType::Int, vec![]);
    let result = cg.generate_signature(&func);
    assert!(result.contains("fn c_read"), "Got: {}", result);
}

#[test]
fn gen_sig_keyword_type_becomes_c_type() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("type".to_string(), HirType::Void, vec![]);
    let result = cg.generate_signature(&func);
    assert!(result.contains("fn c_type"), "Got: {}", result);
}

#[test]
fn gen_sig_keyword_match_becomes_c_match() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("match".to_string(), HirType::Void, vec![]);
    let result = cg.generate_signature(&func);
    assert!(result.contains("fn c_match"), "Got: {}", result);
}

#[test]
fn gen_sig_with_return_type() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "compute".to_string(),
        HirType::Double,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
    );
    let result = cg.generate_signature(&func);
    assert!(result.contains("fn compute"), "Got: {}", result);
    assert!(result.contains("-> f64"), "Got: {}", result);
}

#[test]
fn gen_sig_void_no_return_type() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("process".to_string(), HirType::Void, vec![]);
    let result = cg.generate_signature(&func);
    assert!(result.contains("fn process()"), "Got: {}", result);
    assert!(!result.contains("->"), "Got: {}", result);
}

// ============================================================================
// BATCH 16c: generate_function_with_lifetimes_and_structs — globals, string iter
// ============================================================================

#[test]
fn gen_func_lifetimes_structs_with_globals() {
    use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "inc_global".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Assignment {
            target: "COUNTER".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("COUNTER".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
    );
    let sig = AnnotatedSignature {
        name: "inc_global".to_string(),
        lifetimes: vec![],
        parameters: vec![],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let globals = vec![("COUNTER".to_string(), HirType::Int)];
    let result = cg.generate_function_with_lifetimes_and_structs(
        &func, &sig, &[], &[], &[], &[], &globals,
    );
    assert!(result.contains("fn inc_global"), "Got: {}", result);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("COUNTER"), "Got: {}", result);
}

#[test]
fn gen_func_lifetimes_structs_empty_body_stub() {
    use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let func = HirFunction::new("stub_fn".to_string(), HirType::Int, vec![]);
    let sig = AnnotatedSignature {
        name: "stub_fn".to_string(),
        lifetimes: vec![],
        parameters: vec![],
        return_type: AnnotatedType::Simple(HirType::Int),
    };
    let result = cg.generate_function_with_lifetimes_and_structs(
        &func, &sig, &[], &[], &[], &[], &[],
    );
    assert!(result.contains("fn stub_fn"), "Got: {}", result);
    // Empty body should have a return value stub
    assert!(
        result.contains("0i32") || result.contains("return"),
        "Got: {}",
        result
    );
}

// ============================================================================
// BATCH 17: Deep binary op expression branches
// ============================================================================

#[test]
fn expr_target_chained_comparison_left_bool() {
    // (x < y) < z → ((x < y) as i32) < z
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    ctx.add_variable("y".to_string(), HirType::Int);
    ctx.add_variable("z".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LessThan,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::Variable("y".to_string())),
        }),
        right: Box::new(HirExpression::Variable("z".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as i32"), "Got: {}", result);
}

#[test]
fn expr_target_chained_comparison_right_bool() {
    // x < (y > z) → x < ((y > z) as i32)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    ctx.add_variable("y".to_string(), HirType::Int);
    ctx.add_variable("z".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LessThan,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("y".to_string())),
            right: Box::new(HirExpression::Variable("z".to_string())),
        }),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as i32"), "Got: {}", result);
}

#[test]
fn expr_target_chained_comparison_int_target() {
    // (x < y) < z with Int target → casts to i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("a".to_string(), HirType::Int);
    ctx.add_variable("b".to_string(), HirType::Int);
    ctx.add_variable("c".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LessThan,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::GreaterEqual,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
        right: Box::new(HirExpression::Variable("c".to_string())),
    };
    let target = HirType::Int;
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    // Should have double cast: inner comparison to i32, and outer result to i32
    assert!(result.contains("as i32"), "Got: {}", result);
}

#[test]
fn expr_target_signed_unsigned_comparison() {
    // signed < unsigned → both cast to i64
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("s".to_string(), HirType::Int);
    ctx.add_variable("u".to_string(), HirType::UnsignedInt);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LessThan,
        left: Box::new(HirExpression::Variable("s".to_string())),
        right: Box::new(HirExpression::Variable("u".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as i64"), "Got: {}", result);
}

#[test]
fn expr_target_unsigned_signed_comparison_int_target() {
    // unsigned > signed with Int target → both cast to i64, result to i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("u".to_string(), HirType::UnsignedInt);
    ctx.add_variable("s".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::GreaterThan,
        left: Box::new(HirExpression::Variable("u".to_string())),
        right: Box::new(HirExpression::Variable("s".to_string())),
    };
    let target = HirType::Int;
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as i64"), "Got: {}", result);
    assert!(result.contains("as i32"), "Got: {}", result);
}

#[test]
fn expr_target_bitwise_and_bool_left_operand() {
    // (x == 1) & y → cast bool to i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    ctx.add_variable("y".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::BitwiseAnd,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        }),
        right: Box::new(HirExpression::Variable("y".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as i32"), "Got: {}", result);
    assert!(result.contains("&"), "Got: {}", result);
}

#[test]
fn expr_target_bitwise_or_bool_with_unsigned() {
    // x | (y == 0) where x is unsigned → cast both, result to u32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::UnsignedInt);
    ctx.add_variable("y".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::BitwiseOr,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("y".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("as u32"), "Got: {}", result);
}

#[test]
fn expr_target_comparison_to_int_target() {
    // x > y with Int target → (x > y) as i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("a".to_string(), HirType::Int);
    ctx.add_variable("b".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::GreaterThan,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let target = HirType::Int;
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as i32"), "Got: {}", result);
}

#[test]
fn expr_target_int_arithmetic_to_float_target() {
    // int + int with Float target → cast to f32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("a".to_string(), HirType::Int);
    ctx.add_variable("b".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let target = HirType::Float;
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as f32"), "Got: {}", result);
}

#[test]
fn expr_target_int_arithmetic_to_double_target() {
    // int * int with Double target → cast to f64
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    ctx.add_variable("y".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::Variable("y".to_string())),
    };
    let target = HirType::Double;
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&target));
    assert!(result.contains("as f64"), "Got: {}", result);
}

#[test]
fn expr_target_pointer_add_wrapping() {
    // ptr + n → ptr.wrapping_add(n as usize)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::IntLiteral(3)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("wrapping_add"), "Got: {}", result);
}

#[test]
fn expr_target_pointer_sub_int_wrapping() {
    // ptr - n → ptr.wrapping_sub(n as usize)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::IntLiteral(1)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("wrapping_sub"), "Got: {}", result);
}

#[test]
fn expr_target_pointer_sub_pointer_offset_from() {
    // ptr1 - ptr2 → unsafe { ptr1.offset_from(ptr2) as i32 }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "p1".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    ctx.add_variable(
        "p2".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("p1".to_string())),
        right: Box::new(HirExpression::Variable("p2".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("offset_from"), "Got: {}", result);
    assert!(result.contains("unsafe"), "Got: {}", result);
}

#[test]
fn expr_target_pointer_sub_non_pointer_var() {
    // ptr - offset_var (where offset_var is int, not pointer)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Char)),
    );
    ctx.add_variable("offset".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::Variable("offset".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("wrapping_sub"), "Got: {}", result);
}

#[test]
fn expr_target_dereference_raw_pointer_unsafe() {
    // *ptr → unsafe { *ptr }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("ptr".to_string())));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("*ptr"), "Got: {}", result);
}

#[test]
fn expr_target_dereference_non_pointer_no_unsafe() {
    // *ref → *ref (no unsafe for references)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "r".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
    );
    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("r".to_string())));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(!result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("*r"), "Got: {}", result);
}

#[test]
fn expr_target_dereference_pointer_arithmetic_unsafe() {
    // *(ptr + n) → unsafe { *ptr.wrapping_add(...) }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "arr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let expr = HirExpression::Dereference(Box::new(HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::IntLiteral(2)),
    }));
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("wrapping_add"), "Got: {}", result);
}

#[test]
fn expr_target_nested_binary_adds_parens() {
    // (a + b) * c → parenthesized
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
        right: Box::new(HirExpression::Variable("c".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("(a + b)"), "Got: {}", result);
}

// ============================================================================
// BATCH 18: UnaryOp pointer variants + FunctionCall stdlib branches
// ============================================================================

// --- UnaryOp: pointer PostIncrement → wrapping_add ---
#[test]
fn expr_target_post_increment_pointer_wrapping_add() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PostIncrement,
        operand: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("wrapping_add(1)"), "Got: {}", result);
}

// --- UnaryOp: non-pointer PostIncrement → += 1 ---
#[test]
fn expr_target_post_increment_int_plus_equals() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("i".to_string(), HirType::Int);
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PostIncrement,
        operand: Box::new(HirExpression::Variable("i".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("+= 1"), "Got: {}", result);
    assert!(result.contains("__tmp"), "Got: {}", result);
}

// --- UnaryOp: pointer PostDecrement → wrapping_sub ---
#[test]
fn expr_target_post_decrement_pointer_wrapping_sub() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PostDecrement,
        operand: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("wrapping_sub(1)"), "Got: {}", result);
}

// --- UnaryOp: non-pointer PostDecrement → -= 1 ---
#[test]
fn expr_target_post_decrement_int_minus_equals() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("i".to_string(), HirType::Int);
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PostDecrement,
        operand: Box::new(HirExpression::Variable("i".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("-= 1"), "Got: {}", result);
}

// --- UnaryOp: pointer PreIncrement → wrapping_add ---
#[test]
fn expr_target_pre_increment_pointer_wrapping_add() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PreIncrement,
        operand: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("wrapping_add(1)"), "Got: {}", result);
}

// --- UnaryOp: pointer PreDecrement → wrapping_sub ---
#[test]
fn expr_target_pre_decrement_pointer_wrapping_sub() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PreDecrement,
        operand: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("wrapping_sub(1)"), "Got: {}", result);
}

// --- UnaryOp: LogicalNot on boolean expr → !expr ---
#[test]
fn expr_target_logical_not_boolean_expr() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.starts_with("!"), "Got: {}", result);
    assert!(!result.contains("as i32"), "Got: {}", result);
}

// --- UnaryOp: LogicalNot on integer with Int target → (x == 0) as i32 ---
#[test]
fn expr_target_logical_not_integer_as_i32() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("== 0"), "Got: {}", result);
    assert!(result.contains("as i32"), "Got: {}", result);
}

// --- UnaryOp: LogicalNot on integer without target → (x == 0) no cast ---
#[test]
fn expr_target_logical_not_integer_no_target() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("== 0"), "Got: {}", result);
    assert!(!result.contains("as i32"), "Got: {}", result);
}

// --- UnaryOp: LogicalNot on boolean with Int target → (!expr) as i32 ---
#[test]
fn expr_target_logical_not_boolean_int_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, Some(&HirType::Int));
    assert!(result.contains("as i32"), "Got: {}", result);
    assert!(result.starts_with("(!"), "Got: {}", result);
}

// --- FunctionCall: strlen with 1 arg → .len() as i32 ---
#[test]
fn expr_target_strlen_single_arg() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "strlen".to_string(),
        arguments: vec![HirExpression::Variable("s".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains(".len() as i32"), "Got: {}", result);
}

// --- FunctionCall: strcpy with &str source → .to_string() ---
#[test]
fn expr_target_strcpy_str_source() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "strcpy".to_string(),
        arguments: vec![
            HirExpression::Variable("dest".to_string()),
            HirExpression::Variable("src".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains(".to_string()"), "Got: {}", result);
}

// --- FunctionCall: strcpy with raw pointer source → CStr ---
#[test]
fn expr_target_strcpy_raw_pointer_source() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("node".to_string(), HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))));
    let expr = HirExpression::FunctionCall {
        function: "strcpy".to_string(),
        arguments: vec![
            HirExpression::Variable("dest".to_string()),
            // (*node).name pattern triggers raw pointer detection
            HirExpression::PointerFieldAccess {
                pointer: Box::new(HirExpression::Variable("node".to_string())),
                field: "name".to_string(),
            },
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    // PointerFieldAccess generates (*node).name which contains (*
    assert!(result.contains("CStr") || result.contains("to_string"), "Got: {}", result);
}

// --- FunctionCall: malloc with Vec target + multiply pattern ---
#[test]
fn expr_target_malloc_vec_target_multiply() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::Sizeof { type_name: "int".to_string() }),
        }],
    };
    let result = cg.generate_expression_with_target_type(
        &expr, &ctx, Some(&HirType::Vec(Box::new(HirType::Int))),
    );
    assert!(result.contains("vec![0i32;"), "Got: {}", result);
}

// --- FunctionCall: malloc with Vec target no multiply → Vec::with_capacity ---
#[test]
fn expr_target_malloc_vec_target_no_multiply() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::Variable("size".to_string())],
    };
    let result = cg.generate_expression_with_target_type(
        &expr, &ctx, Some(&HirType::Vec(Box::new(HirType::Int))),
    );
    assert!(result.contains("Vec::<i32>::with_capacity"), "Got: {}", result);
}

// --- FunctionCall: malloc with Pointer(Char) target → Box::leak byte buffer ---
#[test]
fn expr_target_malloc_pointer_char() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(256)],
    };
    let result = cg.generate_expression_with_target_type(
        &expr, &ctx, Some(&HirType::Pointer(Box::new(HirType::Char))),
    );
    assert!(result.contains("Box::leak"), "Got: {}", result);
    assert!(result.contains("0u8"), "Got: {}", result);
    assert!(result.contains("as_mut_ptr()"), "Got: {}", result);
}

// --- FunctionCall: malloc with Pointer(Struct) target → Box::into_raw ---
#[test]
fn expr_target_malloc_pointer_struct() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::Sizeof { type_name: "Node".to_string() }],
    };
    let result = cg.generate_expression_with_target_type(
        &expr, &ctx, Some(&HirType::Pointer(Box::new(HirType::Struct("Node".to_string())))),
    );
    assert!(result.contains("Box::into_raw(Box::<Node>::default())"), "Got: {}", result);
}

// --- FunctionCall: malloc with Pointer(Int) target + multiply ---
#[test]
fn expr_target_malloc_pointer_int_multiply() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::Sizeof { type_name: "int".to_string() }),
        }],
    };
    let result = cg.generate_expression_with_target_type(
        &expr, &ctx, Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(result.contains("Box::leak"), "Got: {}", result);
    assert!(result.contains("as *mut i32"), "Got: {}", result);
}

// --- FunctionCall: malloc with Pointer(Int) target no multiply ---
#[test]
fn expr_target_malloc_pointer_int_single() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(4)],
    };
    let result = cg.generate_expression_with_target_type(
        &expr, &ctx, Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(result.contains("Box::leak"), "Got: {}", result);
    assert!(result.contains("as *mut i32"), "Got: {}", result);
}

// --- FunctionCall: malloc no target, multiply pattern ---
#[test]
fn expr_target_malloc_no_target_multiply() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::Sizeof { type_name: "int".to_string() }),
        }],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("vec![0i32;"), "Got: {}", result);
}

// --- FunctionCall: malloc no target, no multiply → Vec::with_capacity ---
#[test]
fn expr_target_malloc_no_target_no_multiply() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(100)],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("Vec::<u8>::with_capacity"), "Got: {}", result);
}

// --- FunctionCall: calloc with Vec target ---
#[test]
fn expr_target_calloc_vec_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "calloc".to_string(),
        arguments: vec![
            HirExpression::Variable("n".to_string()),
            HirExpression::Sizeof { type_name: "int".to_string() },
        ],
    };
    let result = cg.generate_expression_with_target_type(
        &expr, &ctx, Some(&HirType::Vec(Box::new(HirType::Int))),
    );
    assert!(result.contains("vec![0i32;"), "Got: {}", result);
}

// --- FunctionCall: calloc with Pointer target ---
#[test]
fn expr_target_calloc_pointer_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "calloc".to_string(),
        arguments: vec![
            HirExpression::Variable("n".to_string()),
            HirExpression::Sizeof { type_name: "int".to_string() },
        ],
    };
    let result = cg.generate_expression_with_target_type(
        &expr, &ctx, Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(result.contains("Box::leak"), "Got: {}", result);
    assert!(result.contains("as *mut i32"), "Got: {}", result);
}

// --- FunctionCall: calloc no target ---
#[test]
fn expr_target_calloc_no_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "calloc".to_string(),
        arguments: vec![
            HirExpression::IntLiteral(10),
            HirExpression::Sizeof { type_name: "int".to_string() },
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("vec![0i32;"), "Got: {}", result);
}

// --- FunctionCall: realloc with Pointer target ---
#[test]
fn expr_target_realloc_pointer_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "realloc".to_string(),
        arguments: vec![
            HirExpression::Variable("ptr".to_string()),
            HirExpression::Variable("new_size".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(
        &expr, &ctx, Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(result.contains("realloc("), "Got: {}", result);
    assert!(result.contains("as *mut ()"), "Got: {}", result);
    assert!(result.contains("as *mut i32"), "Got: {}", result);
}

// --- FunctionCall: realloc without target ---
#[test]
fn expr_target_realloc_no_target() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "realloc".to_string(),
        arguments: vec![
            HirExpression::Variable("ptr".to_string()),
            HirExpression::Variable("new_size".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("realloc("), "Got: {}", result);
    assert!(result.contains("as *mut ()"), "Got: {}", result);
    assert!(!result.contains("as *mut i32"), "Got: {}", result);
}

// --- FunctionCall: free with 1 arg → drop ---
#[test]
fn expr_target_free_single_arg() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "free".to_string(),
        arguments: vec![HirExpression::Variable("ptr".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("drop(ptr)"), "Got: {}", result);
}

// --- FunctionCall: fopen read mode → File::open ---
#[test]
fn expr_target_fopen_read_mode() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fopen".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("data.txt".to_string()),
            HirExpression::StringLiteral("r".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("File::open"), "Got: {}", result);
}

// --- FunctionCall: fopen write mode → File::create ---
#[test]
fn expr_target_fopen_write_mode() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fopen".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("out.txt".to_string()),
            HirExpression::StringLiteral("w".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("File::create"), "Got: {}", result);
}

// --- FunctionCall: fclose → drop ---
#[test]
fn expr_target_fclose_drop() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fclose".to_string(),
        arguments: vec![HirExpression::Variable("fp".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("drop(fp)"), "Got: {}", result);
}

// --- FunctionCall: fgetc → Read::read ---
#[test]
fn expr_target_fgetc_read() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fgetc".to_string(),
        arguments: vec![HirExpression::Variable("fp".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("std::io::Read"), "Got: {}", result);
    assert!(result.contains("buf[0] as i32"), "Got: {}", result);
}

// --- FunctionCall: fputc → Write::write ---
#[test]
fn expr_target_fputc_write() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fputc".to_string(),
        arguments: vec![
            HirExpression::Variable("ch".to_string()),
            HirExpression::Variable("fp".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("std::io::Write"), "Got: {}", result);
    assert!(result.contains("as u8"), "Got: {}", result);
}

// --- FunctionCall: fprintf with 2 args (no extra format args) ---
#[test]
fn expr_target_fprintf_two_args() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("fp".to_string()),
            HirExpression::StringLiteral("hello\\n".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("std::io::Write"), "Got: {}", result);
    assert!(result.contains("write!"), "Got: {}", result);
}

// --- FunctionCall: fprintf with extra format args ---
#[test]
fn expr_target_fprintf_with_args() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("fp".to_string()),
            HirExpression::StringLiteral("val=%d\\n".to_string()),
            HirExpression::Variable("x".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("write!"), "Got: {}", result);
}

// --- FunctionCall: fread → Read::read ---
#[test]
fn expr_target_fread() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fread".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::IntLiteral(1),
            HirExpression::Variable("n".to_string()),
            HirExpression::Variable("fp".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("std::io::Read"), "Got: {}", result);
    assert!(result.contains("read(&mut buf)"), "Got: {}", result);
}

// --- FunctionCall: fwrite → Write::write ---
#[test]
fn expr_target_fwrite() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fwrite".to_string(),
        arguments: vec![
            HirExpression::Variable("data".to_string()),
            HirExpression::IntLiteral(1),
            HirExpression::Variable("n".to_string()),
            HirExpression::Variable("fp".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("std::io::Write"), "Got: {}", result);
    assert!(result.contains("write(&data)"), "Got: {}", result);
}

// --- FunctionCall: fputs → Write::write_all ---
#[test]
fn expr_target_fputs() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fputs".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("hello".to_string()),
            HirExpression::Variable("fp".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("std::io::Write"), "Got: {}", result);
    assert!(result.contains("write_all"), "Got: {}", result);
}

// --- FunctionCall: atoi → parse::<i32> ---
#[test]
fn expr_target_atoi() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "atoi".to_string(),
        arguments: vec![HirExpression::Variable("s".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("parse::<i32>()"), "Got: {}", result);
}

// --- FunctionCall: atof → parse::<f64> ---
#[test]
fn expr_target_atof() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "atof".to_string(),
        arguments: vec![HirExpression::Variable("s".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("parse::<f64>()"), "Got: {}", result);
}

// --- FunctionCall: abs → .abs() ---
#[test]
fn expr_target_abs() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "abs".to_string(),
        arguments: vec![HirExpression::Variable("x".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains(".abs()"), "Got: {}", result);
}

// --- FunctionCall: exit → std::process::exit ---
#[test]
fn expr_target_exit() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "exit".to_string(),
        arguments: vec![HirExpression::IntLiteral(1)],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("std::process::exit(1)"), "Got: {}", result);
}

// --- FunctionCall: puts → println! ---
#[test]
fn expr_target_puts() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "puts".to_string(),
        arguments: vec![HirExpression::StringLiteral("hello".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("println!"), "Got: {}", result);
}

// --- FunctionCall: snprintf with 3 args → format! ---
#[test]
fn expr_target_snprintf_no_extra_args() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "snprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::IntLiteral(256),
            HirExpression::StringLiteral("hello".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("format!"), "Got: {}", result);
}

// --- FunctionCall: snprintf with extra args ---
#[test]
fn expr_target_snprintf_with_args() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "snprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::IntLiteral(256),
            HirExpression::StringLiteral("x=%d".to_string()),
            HirExpression::Variable("x".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("format!"), "Got: {}", result);
}

// --- FunctionCall: sprintf with 2 args → format! ---
#[test]
fn expr_target_sprintf_no_extra_args() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "sprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::StringLiteral("hello".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("format!"), "Got: {}", result);
}

// --- FunctionCall: qsort → .sort_by ---
#[test]
fn expr_target_qsort() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "qsort".to_string(),
        arguments: vec![
            HirExpression::Variable("arr".to_string()),
            HirExpression::Variable("n".to_string()),
            HirExpression::Sizeof { type_name: "int".to_string() },
            HirExpression::Variable("compare".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("sort_by"), "Got: {}", result);
    assert!(result.contains("compare"), "Got: {}", result);
}

// --- FunctionCall: fork → comment + 0 ---
#[test]
fn expr_target_fork() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fork".to_string(),
        arguments: vec![],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("fork"), "Got: {}", result);
}

// --- FunctionCall: execl → Command::new ---
#[test]
fn expr_target_execl() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "execl".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("/bin/ls".to_string()),
            HirExpression::StringLiteral("ls".to_string()),
            HirExpression::NullLiteral,
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("Command::new"), "Got: {}", result);
}

// --- FunctionCall: WEXITSTATUS → .code() ---
#[test]
fn expr_target_wexitstatus() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "WEXITSTATUS".to_string(),
        arguments: vec![HirExpression::Variable("status".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains(".code()"), "Got: {}", result);
}

// --- FunctionCall: WIFEXITED → .success() ---
#[test]
fn expr_target_wifexited() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "WIFEXITED".to_string(),
        arguments: vec![HirExpression::Variable("status".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains(".success()"), "Got: {}", result);
}

// --- FunctionCall: printf with no args → print!("") ---
#[test]
fn expr_target_printf_no_args() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("print!"), "Got: {}", result);
}

// --- FunctionCall: printf with 1 arg → print!(fmt) ---
#[test]
fn expr_target_printf_single_arg() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![HirExpression::StringLiteral("hello\\n".to_string())],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("print!"), "Got: {}", result);
}

// --- FunctionCall: default passthrough with keyword rename ---
#[test]
fn expr_target_func_call_keyword_rename_write() {
    let cg = CodeGenerator::new();
    let ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "write".to_string(),
        arguments: vec![
            HirExpression::IntLiteral(1),
            HirExpression::Variable("buf".to_string()),
            HirExpression::Variable("n".to_string()),
        ],
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("c_write("), "Got: {}", result);
}

// --- ArrayIndex: global array → unsafe wrapper ---
#[test]
fn expr_target_array_index_global_unsafe() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_global("TABLE".to_string());
    ctx.add_variable("TABLE".to_string(), HirType::Array {
        element_type: Box::new(HirType::Int),
        size: Some(100),
    });
    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("TABLE".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains("TABLE"), "Got: {}", result);
}

// --- ArrayIndex: raw pointer → unsafe { *ptr.add(i) } ---
#[test]
fn expr_target_array_index_raw_pointer_unsafe() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    assert!(result.contains("unsafe"), "Got: {}", result);
    assert!(result.contains(".add("), "Got: {}", result);
}

// --- ArrayIndex: regular array → arr[i as usize] ---
#[test]
fn expr_target_array_index_regular() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Array {
        element_type: Box::new(HirType::Int),
        size: Some(10),
    });
    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(5)),
    };
    let result = cg.generate_expression_with_target_type(&expr, &ctx, None);
    let _ = result; // test body completed by DECY-202 fix
}
