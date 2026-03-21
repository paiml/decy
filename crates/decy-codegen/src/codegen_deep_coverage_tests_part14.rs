
#[test]
fn stmt_context_break_and_continue() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let break_code = cg.generate_statement_with_context(
        &HirStatement::Break,
        Some("test"),
        &mut ctx,
        None,
    );
    assert_eq!(break_code, "break;");
    let continue_code = cg.generate_statement_with_context(
        &HirStatement::Continue,
        Some("test"),
        &mut ctx,
        None,
    );
    assert_eq!(continue_code, "continue;");
}

// =============================================================================
// Batch 44: For loop, errno, global assignment, return with target type
// =============================================================================

#[test]
fn stmt_context_for_loop_with_condition() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // for (int i = 0; i < 10; i++) → let mut i = 0i32; while i < 10 { ... i += 1; }
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
        increment: vec![HirStatement::Expression(HirExpression::UnaryOp {
            op: decy_hir::UnaryOperator::PostIncrement,
            operand: Box::new(HirExpression::Variable("i".to_string())),
        })],
        body: vec![HirStatement::Break],
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("let mut i") && code.contains("while"),
        "For loop should generate init + while: {}",
        code
    );
}

#[test]
fn stmt_context_for_loop_infinite() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // for (;;) → loop {}
    let stmt = HirStatement::For {
        init: vec![],
        condition: None,
        increment: vec![],
        body: vec![HirStatement::Break],
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("loop {"),
        "for(;;) should generate loop: {}",
        code
    );
}

#[test]
fn stmt_context_errno_assignment() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // errno = 0 → unsafe { ERRNO = 0; }
    let stmt = HirStatement::Assignment {
        target: "errno".to_string(),
        value: HirExpression::IntLiteral(0),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("unsafe") && code.contains("ERRNO"),
        "errno assignment → unsafe ERRNO: {}",
        code
    );
}

#[test]
fn stmt_context_global_assignment() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_global("count".to_string());
    let stmt = HirStatement::Assignment {
        target: "count".to_string(),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("unsafe") && code.contains("count = 42"),
        "Global assignment should be unsafe: {}",
        code
    );
}

#[test]
fn stmt_context_local_assignment() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Assignment {
        target: "x".to_string(),
        value: HirExpression::IntLiteral(7),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert_eq!(code, "x = 7;", "Local assignment: {}", code);
}

#[test]
fn stmt_context_return_with_target_type() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    // Return in a function with i32 return type
    let stmt = HirStatement::Return(Some(HirExpression::IntLiteral(0)));
    let code = cg.generate_statement_with_context(
        &stmt,
        Some("main"),
        &mut ctx,
        Some(&HirType::Int),
    );
    assert!(
        code.contains("return") || code.contains("0"),
        "Return with int: {}",
        code
    );
}

#[test]
fn stmt_context_return_void() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Return(None);
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("return"),
        "Void return: {}",
        code
    );
}

// =============================================================================
// Batch 45: Switch, DerefAssignment, ArrayIndexAssignment, Free, Expression,
//           InlineAsm, FieldAssignment statement types
// =============================================================================

#[test]
fn stmt_context_switch_int_with_char_cases() {
    // DECY-209/219: Switch on int with CharLiteral cases → numeric match patterns
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ch".to_string(), HirType::Int);
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("ch".to_string()),
        cases: vec![
            SwitchCase {
                value: Some(HirExpression::CharLiteral(b'0' as i8)),
                body: vec![
                    HirStatement::Expression(HirExpression::FunctionCall {
                        function: "handle_zero".to_string(),
                        arguments: vec![],
                    }),
                    HirStatement::Break,
                ],
            },
            SwitchCase {
                value: Some(HirExpression::CharLiteral(b'A' as i8)),
                body: vec![
                    HirStatement::Expression(HirExpression::FunctionCall {
                        function: "handle_a".to_string(),
                        arguments: vec![],
                    }),
                    HirStatement::Break,
                ],
            },
        ],
        default_case: Some(vec![HirStatement::Break]),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    // CharLiteral '0' = 48, 'A' = 65 as match patterns
    assert!(
        code.contains("48"),
        "Switch char→int pattern for '0': {}",
        code
    );
    assert!(
        code.contains("65"),
        "Switch char→int pattern for 'A': {}",
        code
    );
    assert!(code.contains("match ch"), "Switch match: {}", code);
    assert!(
        code.contains("handle_zero"),
        "Case body included: {}",
        code
    );
    // Break should be filtered out
    assert!(
        !code.contains("break"),
        "Break should be filtered: {}",
        code
    );
}

#[test]
fn stmt_context_switch_non_int_cases() {
    // Non-int switch → regular expression patterns
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![SwitchCase {
            value: Some(HirExpression::IntLiteral(1)),
            body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(10)))],
        }],
        default_case: None,
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(code.contains("match x"), "Match expression: {}", code);
    assert!(code.contains("1 =>"), "Case pattern: {}", code);
    assert!(code.contains("_ =>"), "Default case always present: {}", code);
}

#[test]
fn stmt_context_switch_with_default_body() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("v".to_string()),
        cases: vec![],
        default_case: Some(vec![
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "fallback".to_string(),
                arguments: vec![],
            }),
        ]),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(code.contains("_ =>"), "Default arm: {}", code);
    assert!(
        code.contains("fallback"),
        "Default body included: {}",
        code
    );
}

#[test]
fn stmt_context_deref_assign_struct_field() {
    // DECY-185: PointerFieldAccess target → no extra dereference
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::PointerFieldAccess {
            pointer: Box::new(HirExpression::Variable("sb".to_string())),
            field: "capacity".to_string(),
        },
        value: HirExpression::IntLiteral(100),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("capacity") && code.contains("100"),
        "Struct field deref: {}",
        code
    );
    // Should NOT have double dereference
    assert!(
        !code.contains("**"),
        "No double deref for field access: {}",
        code
    );
}

#[test]
fn stmt_context_deref_assign_array_index() {
    // DECY-254: ArrayIndex target → no extra dereference
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::ArrayIndex {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::Variable("i".to_string())),
        },
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("arr") && code.contains("42"),
        "Array index assignment: {}",
        code
    );
}

#[test]
fn stmt_context_deref_assign_raw_pointer() {
    // DECY-124: Variable target that is raw pointer → unsafe
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("ptr".to_string()),
        value: HirExpression::IntLiteral(99),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("unsafe"),
        "Raw pointer deref needs unsafe: {}",
        code
    );
    assert!(
        code.contains("*ptr") && code.contains("99"),
        "Deref write: {}",
        code
    );
}

#[test]
fn stmt_context_deref_assign_double_pointer() {
    // DECY-128: Dereference(Variable) where var is Reference to Pointer → unsafe
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "pp".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Pointer(Box::new(HirType::Int))),
            mutable: true,
        },
    );
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Dereference(Box::new(HirExpression::Variable("pp".to_string()))),
        value: HirExpression::IntLiteral(55),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("unsafe"),
        "Double pointer deref needs unsafe: {}",
        code
    );
    assert!(
        code.contains("55"),
        "Value written through double pointer: {}",
        code
    );
}

#[test]
fn stmt_context_deref_assign_plain_variable() {
    // Non-pointer variable → plain dereference, no unsafe
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("val".to_string(), HirType::Int);
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("val".to_string()),
        value: HirExpression::IntLiteral(7),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(code.contains("*val = 7;"), "Plain deref: {}", code);
    assert!(!code.contains("unsafe"), "No unsafe for plain var: {}", code);
}

#[test]
fn stmt_context_array_index_assign_local() {
    // Local array index assignment: arr[(i) as usize] = v
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
        value: HirExpression::IntLiteral(5),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("arr[") && code.contains("as usize") && code.contains("5"),
        "Local array index: {}",
        code
    );
    assert!(
        !code.contains("unsafe"),
        "Local array no unsafe: {}",
        code
    );
}

#[test]
fn stmt_context_array_index_assign_global() {
    // DECY-223: Global array → unsafe wrapper
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("BUFFER".to_string(), HirType::Array { element_type: Box::new(HirType::Char), size: Some(256) });
    ctx.add_global("BUFFER".to_string());
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("BUFFER".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
        value: HirExpression::CharLiteral(b'X' as i8),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("unsafe"),
        "Global array needs unsafe: {}",
        code
    );
    assert!(
        code.contains("BUFFER"),
        "Global array name: {}",
        code
    );
}

#[test]
fn stmt_context_array_index_assign_raw_pointer() {
    // DECY-165: Raw pointer array → unsafe pointer arithmetic
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("data".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("data".to_string())),
        index: Box::new(HirExpression::IntLiteral(3)),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("unsafe"),
        "Raw pointer index needs unsafe: {}",
        code
    );
    assert!(
        code.contains(".add("),
        "Pointer arithmetic with .add(): {}",
        code
    );
}

#[test]
fn stmt_context_array_index_int_to_char_coercion() {
    // DECY-210: Int value assigned to char array element → as u8
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("s".to_string(), HirType::Array { element_type: Box::new(HirType::Char), size: Some(10) });
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("s".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::IntLiteral(48)),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("as u8"),
        "Int→char coercion for array element: {}",
        code
    );
}

#[test]
fn stmt_context_field_assign_regular() {
    // Regular struct field assignment
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::FieldAssignment {
        object: HirExpression::Variable("point".to_string()),
        field: "x".to_string(),
        value: HirExpression::IntLiteral(10),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("point.x = 10;"),
        "Regular field assignment: {}",
        code
    );
}

#[test]
fn stmt_context_field_assign_pointer_object() {
    // DECY-119: Pointer object → unsafe deref
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
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("unsafe"),
        "Pointer field assign needs unsafe: {}",
        code
    );
    assert!(
        code.contains("(*node).value"),
        "Deref struct access: {}",
        code
    );
}

#[test]
fn stmt_context_field_assign_global_struct() {
    // DECY-261: Global struct → unsafe block
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("CONFIG".to_string(), HirType::Struct("Config".to_string()));
    ctx.add_global("CONFIG".to_string());
    let stmt = HirStatement::FieldAssignment {
        object: HirExpression::Variable("CONFIG".to_string()),
        field: "debug".to_string(),
        value: HirExpression::IntLiteral(1),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("unsafe"),
        "Global struct field needs unsafe: {}",
        code
    );
    assert!(
        code.contains("CONFIG.debug"),
        "Global struct field access: {}",
        code
    );
}

#[test]
fn stmt_context_field_assign_keyword_field() {
    // DECY-227: Reserved keyword in field name → escaped
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::FieldAssignment {
        object: HirExpression::Variable("obj".to_string()),
        field: "type".to_string(),
        value: HirExpression::IntLiteral(0),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("r#type"),
        "Keyword field escaped: {}",
        code
    );
}

#[test]
fn stmt_context_free_variable() {
    // free(ptr) → RAII comment
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Free {
        pointer: HirExpression::Variable("buffer".to_string()),
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("RAII") && code.contains("buffer"),
        "Free→RAII comment: {}",
        code
    );
}

#[test]
fn stmt_context_free_expression() {
    // free(ptr_expr) → RAII comment with generated expression
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Free {
        pointer: HirExpression::FieldAccess {
            object: Box::new(HirExpression::Variable("list".to_string())),
            field: "data".to_string(),
        },
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("RAII"),
        "Free expression→RAII: {}",
        code
    );
}

#[test]
fn stmt_context_expression_function_call() {
    // Expression statement: function call for side effects
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::Expression(HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![HirExpression::StringLiteral("hello".to_string())],
    });
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("print") && code.ends_with(";"),
        "Expression statement with semicolon: {}",
        code
    );
}

#[test]
fn stmt_context_inline_asm_translatable() {
    // DECY-197: Translatable inline assembly
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::InlineAsm {
        text: "nop".to_string(),
        translatable: true,
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("manual review required"),
        "Review comment: {}",
        code
    );
    assert!(
        code.contains("translatable to Rust intrinsics"),
        "Translatable hint: {}",
        code
    );
    assert!(code.contains("nop"), "Original asm text: {}", code);
}

#[test]
fn stmt_context_inline_asm_not_translatable() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let stmt = HirStatement::InlineAsm {
        text: "mov eax, 1".to_string(),
        translatable: false,
    };
    let code = cg.generate_statement_with_context(&stmt, Some("test"), &mut ctx, None);
    assert!(
        code.contains("manual review required"),
        "Review comment: {}",
        code
    );
    assert!(
        !code.contains("translatable"),
        "No translatable hint: {}",
        code
    );
    assert!(code.contains("mov eax, 1"), "Original asm: {}", code);
}

// =============================================================================
// Batch 46: BinaryOp deep branches — Assign, Option/NULL checks, strlen,
//           char coercions, comma, pointer arithmetic, logical operators
// =============================================================================

#[test]
fn expr_target_binary_assign_expression() {
    // DECY-195: Embedded assignment (c = getchar()) → block expression
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Assign,
        left: Box::new(HirExpression::Variable("c".to_string())),
        right: Box::new(HirExpression::FunctionCall {
            function: "getchar".to_string(),
            arguments: vec![],
        }),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("__assign_tmp") && code.contains("c ="),
        "Assign expression block: {}",
        code
    );
}

#[test]
fn expr_target_binary_assign_global_array_index_embedded() {
    // DECY-223: Assign to global array index → unsafe wrapper (embedded assign expr)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("GLOBAL_BUF".to_string(), HirType::Array { element_type: Box::new(HirType::Char), size: Some(256) });
    ctx.add_global("GLOBAL_BUF".to_string());
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Assign,
        left: Box::new(HirExpression::ArrayIndex {
            array: Box::new(HirExpression::Variable("GLOBAL_BUF".to_string())),
            index: Box::new(HirExpression::Variable("i".to_string())),
        }),
        right: Box::new(HirExpression::CharLiteral(b'X' as i8)),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("unsafe") && code.contains("GLOBAL_BUF"),
        "Global array assign in unsafe: {}",
        code
    );
}

#[test]
fn expr_target_binary_option_eq_null() {
    // Option var == NULL → .is_none()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("node".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("node".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("is_none()"),
        "Option == NULL → is_none: {}",
        code
    );
}

#[test]
fn expr_target_binary_option_ne_null() {
    // Option var != NULL → .is_some()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("ptr".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("is_some()"),
        "Option != NULL → is_some: {}",
        code
    );
}

#[test]
fn expr_target_binary_null_eq_option() {
    // NULL == Option var → .is_none()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("head".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::NullLiteral),
        right: Box::new(HirExpression::Variable("head".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("is_none()"),
        "NULL == Option → is_none: {}",
        code
    );
}

#[test]
fn expr_target_binary_ptr_eq_zero() {
    // ptr == 0 → ptr == std::ptr::null_mut()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("p".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("null_mut()"),
        "Pointer == 0 → null_mut: {}",
        code
    );
}

#[test]
fn expr_target_binary_zero_ne_ptr() {
    // 0 != ptr → std::ptr::null_mut() != ptr
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("q".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::Variable("q".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("null_mut()"),
        "0 != ptr → null_mut: {}",
        code
    );
}

#[test]
fn expr_target_binary_vec_eq_null() {
    // DECY-130: Vec == 0 → false (Vec never null)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("false"),
        "Vec == 0 → false: {}",
        code
    );
}

#[test]
fn expr_target_binary_vec_ne_null() {
    // DECY-130: Vec != NULL → true (Vec never null)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("buf".to_string(), HirType::Vec(Box::new(HirType::Char)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("buf".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("true"),
        "Vec != NULL → true: {}",
        code
    );
}

#[test]
fn expr_target_binary_box_eq_null() {
    // DECY-119: Box == 0 → false (Box never null)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("node".to_string(), HirType::Box(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("node".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("false"),
        "Box == 0 → false: {}",
        code
    );
}

#[test]
fn expr_target_binary_strlen_eq_zero() {
    // DECY-199: strlen(s) == 0 → s.is_empty()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("s".to_string())],
        }),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("is_empty()"),
        "strlen == 0 → is_empty: {}",
        code
    );
}

#[test]
fn expr_target_binary_strlen_ne_zero() {
    // strlen(s) != 0 → !s.is_empty()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("msg".to_string())],
        }),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("!") && code.contains("is_empty()"),
        "strlen != 0 → !is_empty: {}",
        code
    );
}

#[test]
fn expr_target_binary_zero_eq_strlen() {
    // 0 == strlen(s) → s.is_empty() (reversed)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::FunctionCall {
            function: "strlen".to_string(),
            arguments: vec![HirExpression::Variable("txt".to_string())],
        }),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("is_empty()"),
        "0 == strlen → is_empty: {}",
        code
    );
}

#[test]
fn expr_target_binary_int_ne_char_newline() {
    // DECY-198: int var != CharLiteral('\n') → cast char to i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ch".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("ch".to_string())),
        right: Box::new(HirExpression::CharLiteral(b'\n' as i8)),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("10i32"),
        "Char literal promoted to i32 (newline=10): {}",
        code
    );
}

#[test]
fn expr_target_binary_char_literal_cmp_int() {
    // CharLiteral('0') == int var → reversed char comparison
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("c".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::CharLiteral(b'0' as i8)),
        right: Box::new(HirExpression::Variable("c".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("48i32"),
        "Reversed char→i32 promotion ('0'=48): {}",
        code
    );
}

#[test]
fn expr_target_binary_int_add_char() {
    // DECY-210: int + char literal → cast char to i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("n".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("n".to_string())),
        right: Box::new(HirExpression::CharLiteral(b'0' as i8)),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("48i32"),
        "Int + char('0') arithmetic: {}",
        code
    );
}

#[test]
fn expr_target_binary_char_sub_int() {
    // char literal - int → reversed char arithmetic
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("offset".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::CharLiteral(b'z' as i8)),
        right: Box::new(HirExpression::Variable("offset".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("122i32"),
        "Char('z')→i32 minus int: {}",
        code
    );
}

#[test]
fn expr_target_binary_comma_operator() {
    // DECY-249: comma operator → block expression
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Comma,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::Variable("y".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("{ x; y }"),
        "Comma → block expression: {}",
        code
    );
}

#[test]
fn expr_target_binary_ptr_add() {
    // Pointer + int → .wrapping_add()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("p".to_string())),
        right: Box::new(HirExpression::IntLiteral(1)),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("wrapping_add"),
        "Pointer + int → wrapping_add: {}",
        code
    );
}

#[test]
fn expr_target_binary_ptr_sub_int() {
    // Pointer - int → .wrapping_sub()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("end".to_string(), HirType::Pointer(Box::new(HirType::Char)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("end".to_string())),
        right: Box::new(HirExpression::IntLiteral(3)),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("wrapping_sub"),
        "Pointer - int → wrapping_sub: {}",
        code
    );
}

#[test]
fn expr_target_binary_ptr_sub_ptr() {
    // Pointer - Pointer → unsafe offset_from
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("end".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    ctx.add_variable("start".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("end".to_string())),
        right: Box::new(HirExpression::Variable("start".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("offset_from") && code.contains("unsafe"),
        "Ptr - Ptr → unsafe offset_from: {}",
        code
    );
}

#[test]
fn expr_target_binary_logical_and_int_operands() {
    // DECY-131: a && b with int operands → (a != 0) && (b != 0)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalAnd,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::Variable("y".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("!= 0"),
        "Logical AND with int operands → != 0 checks: {}",
        code
    );
}

#[test]
fn expr_target_binary_logical_or_with_int_target() {
    // DECY-191: Logical OR with int target → cast result as i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalOr,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, Some(&HirType::Int));
    assert!(
        code.contains("as i32"),
        "Logical OR with int target → as i32: {}",
        code
    );
}

#[test]
fn expr_target_binary_logical_and_bool_operands() {
    // Logical AND with boolean expressions → no != 0 wrapping
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalAnd,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
        right: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(100)),
        }),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    // Boolean expressions should NOT get extra != 0
    assert!(
        code.contains("&&"),
        "Logical AND: {}",
        code
    );
}

#[test]
fn expr_target_variable_float_to_int_truncation() {
    // DECY-203: Float var with Int target → as i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ratio".to_string(), HirType::Float);
    let expr = HirExpression::Variable("ratio".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, Some(&HirType::Int));
    assert!(
        code.contains("as i32"),
        "Float→Int truncation: {}",
        code
    );
}

#[test]
fn expr_target_variable_float_to_uint() {
    // Float/Double → UnsignedInt → as u32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("val".to_string(), HirType::Double);
    let expr = HirExpression::Variable("val".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, Some(&HirType::UnsignedInt));
    assert!(
        code.contains("as u32"),
        "Double→UnsignedInt: {}",
        code
    );
}

#[test]
fn expr_target_variable_char_to_int_promotion() {
    // Char → Int → as i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ch".to_string(), HirType::Char);
    let expr = HirExpression::Variable("ch".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, Some(&HirType::Int));
    assert!(
        code.contains("as i32"),
        "Char→Int: {}",
        code
    );
}

#[test]
fn expr_target_variable_einval_enoent_eacces() {
    // DECY-241: errno constants
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let einval = HirExpression::Variable("EINVAL".to_string());
    let enoent = HirExpression::Variable("ENOENT".to_string());
    let eacces = HirExpression::Variable("EACCES".to_string());
    assert_eq!(cg.generate_expression_with_target_type(&einval, &mut ctx, None), "22i32");
    assert_eq!(cg.generate_expression_with_target_type(&enoent, &mut ctx, None), "2i32");
    assert_eq!(cg.generate_expression_with_target_type(&eacces, &mut ctx, None), "13i32");
}

#[test]
fn expr_target_variable_global_char_to_int() {
    // Global char→int with unsafe wrapper
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("FLAG".to_string(), HirType::Char);
    ctx.add_global("FLAG".to_string());
    let expr = HirExpression::Variable("FLAG".to_string());
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, Some(&HirType::Int));
    assert!(
        code.contains("unsafe") && code.contains("as i32"),
        "Global char→int with unsafe: {}",
        code
    );
}

#[test]
fn expr_target_variable_ref_immut_slice_to_ptr() {
    // DECY-146: Immutable reference to array → .as_ptr() with cast
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "data".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(10),
            }),
            mutable: false,
        },
    );
    let expr = HirExpression::Variable("data".to_string());
    let code = cg.generate_expression_with_target_type(
        &expr,
        &mut ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(
        code.contains("as_ptr()"),
        "Immut ref array → as_ptr: {}",
        code
    );
}

#[test]
fn expr_target_variable_ref_to_ptr_single() {
    // DECY-146: &mut T to *mut T → cast
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "item".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        },
    );
    let expr = HirExpression::Variable("item".to_string());
    let code = cg.generate_expression_with_target_type(
        &expr,
        &mut ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(
        code.contains("as *mut"),
        "Mutable ref→raw pointer cast: {}",
        code
    );
}

#[test]
fn expr_target_variable_immut_ref_to_ptr_single() {
    // &T to *mut T → as *const _ as *mut _
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "val".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
    );
    let expr = HirExpression::Variable("val".to_string());
    let code = cg.generate_expression_with_target_type(
        &expr,
        &mut ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(
        code.contains("as *const _") && code.contains("as *mut _"),
        "Immut ref→raw pointer double cast: {}",
        code
    );
}

// =============================================================================
// Batch 47: Annotated signature param transformations + deep statement paths
// =============================================================================

#[test]
fn sig_annotated_regular_char_ptr_param() {
    // Regular (non-const) char* → stays as pointer or reference (not &str)
    let func = HirFunction::new_with_body(
        "greet".to_string(),
        HirType::Void,
        vec![HirParameter::new("name".to_string(), HirType::Pointer(Box::new(HirType::Char)))],
        vec![],
    );
    let sig = make_annotated_sig(&func);
    let cg = CodeGenerator::new();
    let code = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    // Non-const char* should become &mut u8 (not &str)
    assert!(
        !code.contains("&str"),
        "Non-const char* should not become &str: {}",
        code
    );
}

#[test]
fn sig_annotated_void_ptr_stays_raw() {
    // DECY-168: void* → *mut ()
    let func = HirFunction::new_with_body(
        "process".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Pointer(Box::new(HirType::Void)),
        )],
        vec![],
    );
    let sig = make_annotated_sig(&func);
    let cg = CodeGenerator::new();
    let code = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    assert!(
        code.contains("*mut ()"),
        "void* stays as *mut (): {}",
        code
    );
}

#[test]
fn sig_annotated_ptr_arithmetic_stays_raw() {
    // DECY-123: Pointer used in arithmetic → stays raw pointer
    let func = HirFunction::new_with_body(
        "traverse".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
            // ptr++: UnaryOp increment on pointer → pointer arithmetic
            HirStatement::Assignment {
                target: "ptr".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("ptr".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
        ],
    );
    let sig = make_annotated_sig(&func);
    let cg = CodeGenerator::new();
    let code = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    assert!(
        code.contains("*mut i32"),
        "Pointer arithmetic → raw pointer: {}",
        code
    );
}

#[test]
fn sig_annotated_unsized_array_param() {
    // DECY-196: char arr[] → &mut [u8]
    let func = HirFunction::new_with_body(
        "fill".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "buf".to_string(),
            HirType::Array {
                element_type: Box::new(HirType::Char),
                size: None,
            },
        )],
        vec![],
    );
    let sig = make_annotated_sig(&func);
    let cg = CodeGenerator::new();
    let code = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    assert!(
        code.contains("&mut [u8]"),
        "Unsized array → &mut [slice]: {}",
        code
    );
}

#[test]
fn sig_annotated_main_no_return_type() {
    // int main() → fn main() (no return type)
    let func = HirFunction::new_with_body(
        "main".to_string(),
        HirType::Int,
        vec![],
        vec![],
    );
    let sig = make_annotated_sig(&func);
    let cg = CodeGenerator::new();
    let code = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    assert!(
        !code.contains("-> i32"),
        "main() should not have -> i32: {}",
        code
    );
    assert!(
        code.contains("fn main()"),
        "Should be fn main(): {}",
        code
    );
}

#[test]
fn sig_annotated_multiple_output_params_tuple() {
    // DECY-085: Multiple output params → tuple return
    let func = HirFunction::new_with_body(
        "get_dimensions".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("input".to_string(), HirType::Int),
            HirParameter::new(
                "width".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            ),
            HirParameter::new(
                "height".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            ),
        ],
        vec![
            // Write to *width (output)
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("width".to_string()),
                value: HirExpression::IntLiteral(800),
            },
            // Write to *height (output)
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("height".to_string()),
                value: HirExpression::IntLiteral(600),
            },
        ],
    );
    let sig = make_annotated_sig(&func);
    let cg = CodeGenerator::new();
    let code = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    assert!(
        code.contains("(i32, i32)"),
        "Multiple output params → tuple: {}",
        code
    );
}

#[test]
fn sig_annotated_regular_ptr_to_mut_ref() {
    // Regular pointer param without arithmetic → &mut T
    let func = HirFunction::new_with_body(
        "increment".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "val".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("val".to_string()),
                value: HirExpression::IntLiteral(1),
            },
        ],
    );
    let sig = make_annotated_sig(&func);
    let cg = CodeGenerator::new();
    let code = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    // Without pointer arithmetic, should become &mut i32
    assert!(
        code.contains("&mut i32") || code.contains("*mut i32"),
        "Regular ptr → &mut or *mut: {}",
        code
    );
}

#[test]
fn sig_annotated_ptr_null_check_stays_raw() {
    // DECY-137: Pointer compared to NULL → stays raw
    let func = HirFunction::new_with_body(
        "check_null".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::Equal,
                left: Box::new(HirExpression::Variable("ptr".to_string())),
                right: Box::new(HirExpression::IntLiteral(0)),
            },
            then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(-1)))],
            else_block: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))]),
        }],
    );
    let sig = make_annotated_sig(&func);
    let cg = CodeGenerator::new();
    let code = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    assert!(
        code.contains("*mut i32"),
        "Null-checked pointer stays raw: {}",
        code
    );
}

#[test]
fn sig_annotated_vec_return_detection() {
    // DECY-142: Function returning malloc'd array → Vec<T>
    let func = HirFunction::new_with_body(
        "create_buffer".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new("size".to_string(), HirType::Int)],
        vec![
            HirStatement::VariableDeclaration {
                name: "buf".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![HirExpression::BinaryOp {
                        op: BinaryOperator::Multiply,
                        left: Box::new(HirExpression::Variable("size".to_string())),
                        right: Box::new(HirExpression::Sizeof {
                            type_name: "int".to_string(),
                        }),
                    }],
                }),
            },
            HirStatement::Return(Some(HirExpression::Variable("buf".to_string()))),
        ],
    );
    let sig = make_annotated_sig(&func);
    let cg = CodeGenerator::new();
    let code = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    assert!(
        code.contains("Vec<i32>"),
        "Malloc'd return → Vec<i32>: {}",
        code
    );
}

#[test]
fn sig_annotated_non_void_return_type() {
    // Regular non-void return type
    let func = HirFunction::new_with_body(
        "add".to_string(),
        HirType::Double,
        vec![
            HirParameter::new("a".to_string(), HirType::Double),
            HirParameter::new("b".to_string(), HirType::Double),
        ],
        vec![],
    );
    let sig = make_annotated_sig(&func);
    let cg = CodeGenerator::new();
    let code = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    assert!(
        code.contains("-> f64"),
        "Non-void return type: {}",
        code
    );
}

// =============================================================================
// Batch 48: Mixed arithmetic, chained comparisons, signed/unsigned,
//           bitwise with bool, arithmetic result casting
// =============================================================================

#[test]
fn expr_target_binary_int_add_float() {
    // DECY-204: int + float → cast int to f32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("i".to_string(), HirType::Int);
    ctx.add_variable("f".to_string(), HirType::Float);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("i".to_string())),
        right: Box::new(HirExpression::Variable("f".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("as f32"),
        "Int + Float → int cast to f32: {}",
        code
    );
}

#[test]
fn expr_target_binary_double_sub_int() {
    // double - int → cast int to f64
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("d".to_string(), HirType::Double);
    ctx.add_variable("n".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("d".to_string())),
        right: Box::new(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("as f64"),
        "Double - Int → int cast to f64: {}",
        code
    );
}

#[test]
fn expr_target_binary_float_mul_double() {
    // float * double → cast float to f64
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("f".to_string(), HirType::Float);
    ctx.add_variable("d".to_string(), HirType::Double);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::Variable("f".to_string())),
        right: Box::new(HirExpression::Variable("d".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("as f64"),
        "Float * Double → float cast to f64: {}",
        code
    );
}

#[test]
fn expr_target_binary_chained_comparison() {
    // DECY-206: (x < y) < z → cast comparison to i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LessThan,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::Variable("y".to_string())),
        }),
        right: Box::new(HirExpression::Variable("z".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("as i32"),
        "Chained comparison casts to i32: {}",
        code
    );
}

#[test]
fn expr_target_binary_chained_comparison_with_int_target() {
    // Chained comparison with int target → final result also cast
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::GreaterThan,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("b".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        }),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, Some(&HirType::Int));
    assert!(
        code.contains("as i32"),
        "Chained comparison with int target: {}",
        code
    );
}

#[test]
fn expr_target_binary_signed_unsigned_comparison() {
    // DECY-251: int vs unsigned int comparison → cast both to i64
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("signed_val".to_string(), HirType::Int);
    ctx.add_variable("unsigned_val".to_string(), HirType::UnsignedInt);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LessThan,
        left: Box::new(HirExpression::Variable("signed_val".to_string())),
        right: Box::new(HirExpression::Variable("unsigned_val".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("as i64"),
        "Signed/unsigned comparison → i64 cast: {}",
        code
    );
}

#[test]
fn expr_target_binary_signed_unsigned_with_int_target() {
    // Signed/unsigned comparison with int target → also cast result to i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("a".to_string(), HirType::UnsignedInt);
    ctx.add_variable("b".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, Some(&HirType::Int));
    assert!(
        code.contains("as i64") && code.contains("as i32"),
        "Signed/unsigned + int target: {}",
        code
    );
}

#[test]
fn expr_target_binary_comparison_returns_bool_cast_to_int() {
    // DECY-191: Comparison with int target → cast to i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::GreaterThan,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, Some(&HirType::Int));
    assert!(
        code.contains("as i32"),
        "Comparison → int → as i32: {}",
        code
    );
}

#[test]
fn expr_target_binary_int_div_to_float_target() {
    // DECY-204: int / int with float target → cast result to f32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("a".to_string(), HirType::Int);
    ctx.add_variable("b".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Divide,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, Some(&HirType::Float));
    assert!(
        code.contains("as f32"),
        "Int/Int with float target → as f32: {}",
        code
    );
}

#[test]
fn expr_target_binary_int_mod_to_double_target() {
    // int % int with double target → cast result to f64
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    ctx.add_variable("y".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Modulo,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::Variable("y".to_string())),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, Some(&HirType::Double));
    assert!(
        code.contains("as f64"),
        "Int%Int with double target → as f64: {}",
        code
    );
}

#[test]
fn expr_target_binary_bitwise_and_with_bool() {
    // DECY-252: x & (y == 1) → cast bool to i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::BitwiseAnd,
        left: Box::new(HirExpression::Variable("flags".to_string())),
        right: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("mode".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        }),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("as i32"),
        "Bitwise AND with bool operand → cast: {}",
        code
    );
}

#[test]
fn expr_target_binary_bitwise_or_bool_and_unsigned() {
    // DECY-252: unsigned | (x != 0) → cast both
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("mask".to_string(), HirType::UnsignedInt);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::BitwiseOr,
        left: Box::new(HirExpression::Variable("mask".to_string())),
        right: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::NotEqual,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("as i32"),
        "Bitwise OR with unsigned + bool: {}",
        code
    );
}

#[test]
fn expr_target_binary_nested_parens() {
    // Nested binary ops get parenthesized
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
        right: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Subtract,
            left: Box::new(HirExpression::Variable("c".to_string())),
            right: Box::new(HirExpression::Variable("d".to_string())),
        }),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("(a + b)") && code.contains("(c - d)"),
        "Nested ops get parens: {}",
        code
    );
}

#[test]
fn expr_target_binary_ptr_field_access_cmp_zero() {
    // DECY-235: ptr->field == 0 where field is pointer → null_mut comparison
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("node".to_string(), HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))));
    // Register struct type for field type inference
    ctx.add_struct(&decy_hir::HirStruct::new(
        "Node".to_string(),
        vec![decy_hir::HirStructField::new(
            "next".to_string(),
            HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        )],
    ));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::PointerFieldAccess {
            pointer: Box::new(HirExpression::Variable("node".to_string())),
            field: "next".to_string(),
        }),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression_with_target_type(&expr, &mut ctx, None);
    assert!(
        code.contains("null_mut()"),
        "Ptr field == 0 → null_mut: {}",
        code
    );
}

// =============================================================================
// Batch 49: UnaryOp pointer variants, Dereference unsafe, FunctionCall transforms
// =============================================================================

#[test]
fn expr_context_post_inc_pointer() {
    // DECY-253: ptr++ → wrapping_add for pointers
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::PostIncrement,
        operand: Box::new(HirExpression::Variable("ptr".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("wrapping_add"),
        "Pointer post-inc → wrapping_add: {}",
        code
    );
}

#[test]
fn expr_context_post_inc_int() {
    // int++ → { let tmp = x; x += 1; tmp }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("i".to_string(), HirType::Int);
    let expr = HirExpression::UnaryOp {
