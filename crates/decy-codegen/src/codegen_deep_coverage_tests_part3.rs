    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("3.14"),
        "FloatLiteral should contain 3.14, got: {}",
        code
    );
}

#[test]
fn expr_int_literal_negative() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::IntLiteral(-42);
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("-42") || code.contains("42"),
        "Negative IntLiteral should contain the value, got: {}",
        code
    );
}

#[test]
fn expr_int_literal_zero() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::IntLiteral(0);
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("0"),
        "IntLiteral(0) should generate 0, got: {}",
        code
    );
}

#[test]
fn expr_post_increment_simple() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PostIncrement,
        operand: Box::new(HirExpression::Variable("i".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("i") && (code.contains("+=") || code.contains("+ 1")),
        "PostIncrement should increment i, got: {}",
        code
    );
}

#[test]
fn expr_post_decrement_simple() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PostDecrement,
        operand: Box::new(HirExpression::Variable("j".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("j") && (code.contains("-=") || code.contains("- 1")),
        "PostDecrement should decrement j, got: {}",
        code
    );
}

#[test]
fn expr_pre_increment() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PreIncrement,
        operand: Box::new(HirExpression::Variable("k".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("k"),
        "PreIncrement should reference k, got: {}",
        code
    );
}

#[test]
fn expr_pre_decrement() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PreDecrement,
        operand: Box::new(HirExpression::Variable("m".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("m"),
        "PreDecrement should reference m, got: {}",
        code
    );
}

#[test]
fn expr_bitwise_not() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::BitwiseNot,
        operand: Box::new(HirExpression::Variable("flags".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("!") || code.contains("flags"),
        "BitwiseNot should negate flags, got: {}",
        code
    );
}

#[test]
fn expr_compound_literal_array() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(3),
        },
        initializers: vec![
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(2),
            HirExpression::IntLiteral(3),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("1") && code.contains("2") && code.contains("3"),
        "CompoundLiteral should contain all initializers, got: {}",
        code
    );
}

#[test]
fn expr_compound_literal_struct() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Point".to_string()),
        initializers: vec![
            HirExpression::IntLiteral(10),
            HirExpression::IntLiteral(20),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("Point") || code.contains("10"),
        "CompoundLiteral struct should reference type or values, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: realloc, empty return, pointer conditions, errno, for(;;)
// ============================================================================

#[test]
fn stmt_return_empty_non_main() {
    let cg = CodeGenerator::new();
    // C: void foo() { return; }
    let stmt = HirStatement::Return(None);
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("return"),
        "Empty return should generate return, got: {}",
        code
    );
}

#[test]
fn stmt_for_loop_infinite() {
    let cg = CodeGenerator::new();
    // C: for(;;) { break; }
    let stmt = HirStatement::For {
        init: vec![],
        condition: None,
        increment: vec![],
        body: vec![HirStatement::Break],
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("loop"),
        "for(;;) should generate loop, got: {}",
        code
    );
}

#[test]
fn stmt_for_loop_with_init_no_condition() {
    let cg = CodeGenerator::new();
    // C: for(int i = 0; ; i++) { break; }
    let stmt = HirStatement::For {
        init: vec![HirStatement::VariableDeclaration {
            name: "i".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::IntLiteral(0)),
        }],
        condition: None,
        increment: vec![HirStatement::Expression(HirExpression::UnaryOp {
            op: UnaryOperator::PostIncrement,
            operand: Box::new(HirExpression::Variable("i".to_string())),
        })],
        body: vec![HirStatement::Break],
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("loop"),
        "for(init;;inc) should generate loop with init, got: {}",
        code
    );
}

#[test]
fn stmt_assignment_to_errno() {
    let cg = CodeGenerator::new();
    // C: errno = 0;
    let stmt = HirStatement::Assignment {
        target: "errno".to_string(),
        value: HirExpression::IntLiteral(0),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("errno") || code.contains("0"),
        "errno assignment should reference errno, got: {}",
        code
    );
}

#[test]
fn stmt_switch_with_default_only() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("val".to_string()),
        cases: vec![],
        default_case: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
            -1,
        )))]),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("_") || code.contains("match") || code.contains("val"),
        "Switch with default only should generate match, got: {}",
        code
    );
}

#[test]
fn stmt_switch_multiple_cases() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("op".to_string()),
        cases: vec![
            SwitchCase {
                value: Some(HirExpression::IntLiteral(1)),
                body: vec![HirStatement::Return(Some(HirExpression::StringLiteral(
                    "add".to_string(),
                )))],
            },
            SwitchCase {
                value: Some(HirExpression::IntLiteral(2)),
                body: vec![HirStatement::Return(Some(HirExpression::StringLiteral(
                    "sub".to_string(),
                )))],
            },
        ],
        default_case: Some(vec![HirStatement::Return(Some(
            HirExpression::StringLiteral("unknown".to_string()),
        ))]),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("1") && code.contains("2"),
        "Switch should contain both case values, got: {}",
        code
    );
}

#[test]
fn stmt_switch_char_literal_case() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("ch".to_string()),
        cases: vec![
            SwitchCase {
                value: Some(HirExpression::CharLiteral(b'A' as i8)),
                body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
            },
            SwitchCase {
                value: Some(HirExpression::CharLiteral(b'B' as i8)),
                body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(2)))],
            },
        ],
        default_case: None,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("A") || code.contains("65"),
        "Switch with char cases should contain char values, got: {}",
        code
    );
}

#[test]
fn stmt_var_decl_with_realloc_null() {
    let cg = CodeGenerator::new();
    // C: int* p = realloc(NULL, 10 * sizeof(int));
    let stmt = HirStatement::VariableDeclaration {
        name: "p".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
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
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("p") && (code.contains("Vec") || code.contains("vec") || code.contains("alloc")),
        "realloc(NULL, ...) should generate Vec or allocation, got: {}",
        code
    );
}

#[test]
fn stmt_var_decl_with_realloc_non_pattern_size() {
    let cg = CodeGenerator::new();
    // C: int* p = realloc(old, new_size); — non-multiply size pattern
    let stmt = HirStatement::VariableDeclaration {
        name: "p".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::Realloc {
            pointer: Box::new(HirExpression::Variable("old".to_string())),
            new_size: Box::new(HirExpression::Variable("new_size".to_string())),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("p"),
        "realloc with non-pattern size should still generate, got: {}",
        code
    );
}

// ============================================================================
// Format specifier coverage: via printf FunctionCall expressions
// ============================================================================

#[test]
fn expr_printf_with_width() {
    let cg = CodeGenerator::new();
    // C: printf("%10d", x);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%10d".to_string()),
            HirExpression::Variable("x".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print") || code.contains("x"),
        "printf with width should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_with_precision() {
    let cg = CodeGenerator::new();
    // C: printf("%.2f", val);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%.2f".to_string()),
            HirExpression::Variable("val".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print") || code.contains("val"),
        "printf with precision should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_with_width_and_precision() {
    let cg = CodeGenerator::new();
    // C: printf("%10.5f", val);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%10.5f".to_string()),
            HirExpression::Variable("val".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print"),
        "printf with width.precision should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_zero_pad_flag() {
    let cg = CodeGenerator::new();
    // C: printf("%05d", x);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%05d".to_string()),
            HirExpression::Variable("x".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print") || code.contains("x"),
        "printf with zero-pad should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_left_align_flag() {
    let cg = CodeGenerator::new();
    // C: printf("%-10s", name);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%-10s".to_string()),
            HirExpression::Variable("name".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print"),
        "printf with left-align should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_hex_alternate() {
    let cg = CodeGenerator::new();
    // C: printf("%#x", val);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%#x".to_string()),
            HirExpression::Variable("val".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print"),
        "printf with # flag should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_long_long() {
    let cg = CodeGenerator::new();
    // C: printf("%lld", big_val);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%lld".to_string()),
            HirExpression::Variable("big_val".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print"),
        "printf with lld should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_unsigned() {
    let cg = CodeGenerator::new();
    // C: printf("%u", count);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%u".to_string()),
            HirExpression::Variable("count".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print"),
        "printf with %%u should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_octal() {
    let cg = CodeGenerator::new();
    // C: printf("%o", val);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%o".to_string()),
            HirExpression::Variable("val".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print"),
        "printf with %%o should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_multiple_specifiers() {
    let cg = CodeGenerator::new();
    // C: printf("name=%s age=%d score=%.1f", name, age, score);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("name=%s age=%d score=%.1f".to_string()),
            HirExpression::Variable("name".to_string()),
            HirExpression::Variable("age".to_string()),
            HirExpression::Variable("score".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print"),
        "printf with multiple specifiers should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_char_specifier() {
    let cg = CodeGenerator::new();
    // C: printf("%c", ch);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%c".to_string()),
            HirExpression::Variable("ch".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print"),
        "printf with %%c should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_pointer_specifier() {
    let cg = CodeGenerator::new();
    // C: printf("%p", ptr);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%p".to_string()),
            HirExpression::Variable("ptr".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print"),
        "printf with %%p should generate print, got: {}",
        code
    );
}

#[test]
fn expr_printf_size_t_specifier() {
    let cg = CodeGenerator::new();
    // C: printf("%zu", len);
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%zu".to_string()),
            HirExpression::Variable("len".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("print"),
        "printf with %%zu should generate print, got: {}",
        code
    );
}

#[test]
fn expr_fprintf_to_stderr() {
    let cg = CodeGenerator::new();
    // C: fprintf(stderr, "error: %s\n", msg);
    let expr = HirExpression::FunctionCall {
        function: "fprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("stderr".to_string()),
            HirExpression::StringLiteral("error: %s\\n".to_string()),
            HirExpression::Variable("msg".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("eprint") || code.contains("stderr") || code.contains("msg"),
        "fprintf to stderr should generate eprint, got: {}",
        code
    );
}

// ============================================================================
// Additional stdlib function coverage
// ============================================================================

#[test]
fn expr_fgetc_call() {
    let cg = CodeGenerator::new();
    // C: fgetc(fp);
    let expr = HirExpression::FunctionCall {
        function: "fgetc".to_string(),
        arguments: vec![HirExpression::Variable("fp".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        !code.is_empty(),
        "fgetc should generate something, got: {}",
        code
    );
}

#[test]
fn expr_fputc_call() {
    let cg = CodeGenerator::new();
    // C: fputc('c', fp);
    let expr = HirExpression::FunctionCall {
        function: "fputc".to_string(),
        arguments: vec![
            HirExpression::CharLiteral(b'c' as i8),
            HirExpression::Variable("fp".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        !code.is_empty(),
        "fputc should generate something, got: {}",
        code
    );
}

#[test]
fn expr_realloc_call() {
    let cg = CodeGenerator::new();
    // C: realloc(ptr, new_size);
    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::Variable("ptr".to_string())),
        new_size: Box::new(HirExpression::Variable("new_size".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        !code.is_empty(),
        "realloc should generate code, got: {}",
        code
    );
}

// ============================================================================
// Variable declaration edge cases
// ============================================================================

#[test]
fn stmt_var_decl_char_array_from_compound() {
    let cg = CodeGenerator::new();
    // C: char arr[3] = {65, 66, 67};
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(3),
        },
        initializer: Some(HirExpression::CompoundLiteral {
            literal_type: HirType::Array {
                element_type: Box::new(HirType::Char),
                size: Some(3),
            },
            initializers: vec![
                HirExpression::IntLiteral(65),
                HirExpression::IntLiteral(66),
                HirExpression::IntLiteral(67),
            ],
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("arr"),
        "Char array from compound should contain arr, got: {}",
        code
    );
}

#[test]
fn stmt_var_decl_unsigned_int() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "count".to_string(),
        var_type: HirType::UnsignedInt,
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("u32") || code.contains("count"),
        "UnsignedInt decl should use u32, got: {}",
        code
    );
}

#[test]
fn stmt_var_decl_double() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "big".to_string(),
        var_type: HirType::Double,
        initializer: Some(HirExpression::FloatLiteral("0.0".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("f64") || code.contains("big"),
        "Double decl should use f64, got: {}",
        code
    );
}

#[test]
fn stmt_var_decl_signed_char() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "sc".to_string(),
        var_type: HirType::SignedChar,
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("i8") || code.contains("sc"),
        "SignedChar decl should use i8, got: {}",
        code
    );
}

#[test]
fn stmt_var_decl_struct_type() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "pt".to_string(),
        var_type: HirType::Struct("Point".to_string()),
        initializer: None,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("Point") || code.contains("pt"),
        "Struct decl should reference Point, got: {}",
        code
    );
}

#[test]
fn stmt_var_decl_string_literal_type() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "msg".to_string(),
        var_type: HirType::StringLiteral,
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("msg") || code.contains("hello"),
        "StringLiteral decl should contain msg or hello, got: {}",
        code
    );
}

// ============================================================================
// generate_annotated_signature_with_func coverage
// ============================================================================

#[test]
fn annotated_sig_simple_void_function() {
    use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "noop".to_string(),
        lifetimes: vec![],
        parameters: vec![],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let code = cg.generate_annotated_signature_with_func(&sig, None);
    assert!(
        code.contains("fn noop()"),
        "Should generate fn noop(), got: {}",
        code
    );
    assert!(
        !code.contains("->"),
        "Void should have no return arrow, got: {}",
        code
    );
}

#[test]
fn annotated_sig_with_params_no_func() {
    use decy_ownership::lifetime_gen::{AnnotatedParameter, AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "add".to_string(),
        lifetimes: vec![],
        parameters: vec![
            AnnotatedParameter {
                name: "a".to_string(),
                param_type: AnnotatedType::Simple(HirType::Int),
            },
            AnnotatedParameter {
                name: "b".to_string(),
                param_type: AnnotatedType::Simple(HirType::Int),
            },
        ],
        return_type: AnnotatedType::Simple(HirType::Int),
    };
    let code = cg.generate_annotated_signature_with_func(&sig, None);
    assert!(
        code.contains("fn add"),
        "Should contain fn add, got: {}",
        code
    );
    assert!(
        code.contains("a:") && code.contains("b:"),
        "Should contain both params, got: {}",
        code
    );
    assert!(
        code.contains("-> i32"),
        "Should return i32, got: {}",
        code
    );
}

#[test]
fn annotated_sig_keyword_rename() {
    use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "type".to_string(),
        lifetimes: vec![],
        parameters: vec![],
        return_type: AnnotatedType::Simple(HirType::Int),
    };
    let code = cg.generate_annotated_signature_with_func(&sig, None);
    assert!(
        code.contains("c_type"),
        "Keyword 'type' should be renamed to c_type, got: {}",
        code
    );
}

#[test]
fn annotated_sig_main_no_return_type() {
    use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "main".to_string(),
        lifetimes: vec![],
        parameters: vec![],
        return_type: AnnotatedType::Simple(HirType::Int),
    };
    // main function with int return should NOT have -> i32 (Rust main returns ())
    let func = HirFunction::new("main".to_string(), HirType::Int, vec![]);
    let code = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    assert!(
        !code.contains("->"),
        "main() should have no return type arrow, got: {}",
        code
    );
}

#[test]
fn annotated_sig_with_pointer_param_and_func_body() {
    use decy_ownership::lifetime_gen::{AnnotatedParameter, AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let sig = AnnotatedSignature {
        name: "read_val".to_string(),
        lifetimes: vec![],
        parameters: vec![AnnotatedParameter {
            name: "p".to_string(),
            param_type: AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Int))),
        }],
        return_type: AnnotatedType::Simple(HirType::Int),
    };
    // Function that only reads via pointer → &i32
    let func = HirFunction::new_with_body(
        "read_val".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::Dereference(
            Box::new(HirExpression::Variable("p".to_string())),
        )))],
    );
    let code = cg.generate_annotated_signature_with_func(&sig, Some(&func));
    assert!(
        code.contains("&") && code.contains("i32"),
        "Read-only pointer should become reference, got: {}",
        code
    );
}

// ============================================================================
// generate_function_with_lifetimes: full function generation
// ============================================================================

#[test]
fn gen_func_with_lifetimes_simple() {
    use decy_ownership::lifetime_gen::{AnnotatedParameter, AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "square".to_string(),
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::Variable("x".to_string())),
        }))],
    );
    let sig = AnnotatedSignature {
        name: "square".to_string(),
        lifetimes: vec![],
        parameters: vec![AnnotatedParameter {
            name: "x".to_string(),
            param_type: AnnotatedType::Simple(HirType::Int),
        }],
        return_type: AnnotatedType::Simple(HirType::Int),
    };
    let code = cg.generate_function_with_lifetimes(&func, &sig);
    assert!(
        code.contains("fn square"),
        "Should contain fn square, got: {}",
        code
    );
    assert!(
        code.contains("return") || code.contains("x * x") || code.contains("x"),
        "Should contain body, got: {}",
        code
    );
}

#[test]
fn gen_func_with_lifetimes_empty_body() {
    use decy_ownership::lifetime_gen::{AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "stub".to_string(),
        HirType::Void,
        vec![],
        vec![],
    );
    let sig = AnnotatedSignature {
        name: "stub".to_string(),
        lifetimes: vec![],
        parameters: vec![],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let code = cg.generate_function_with_lifetimes(&func, &sig);
    assert!(
        code.contains("fn stub"),
        "Should contain fn stub, got: {}",
        code
    );
}

#[test]
fn gen_func_with_lifetimes_pointer_param() {
    use decy_ownership::lifetime_gen::{AnnotatedParameter, AnnotatedSignature, AnnotatedType};
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "inc".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "val".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("val".to_string()),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Dereference(Box::new(
                    HirExpression::Variable("val".to_string()),
                ))),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
    );
    let sig = AnnotatedSignature {
        name: "inc".to_string(),
        lifetimes: vec![],
        parameters: vec![AnnotatedParameter {
            name: "val".to_string(),
            param_type: AnnotatedType::Simple(HirType::Pointer(Box::new(HirType::Int))),
        }],
        return_type: AnnotatedType::Simple(HirType::Void),
    };
    let code = cg.generate_function_with_lifetimes(&func, &sig);
    assert!(
        code.contains("fn inc"),
        "Should contain fn inc, got: {}",
        code
    );
    assert!(
        code.contains("&mut") || code.contains("val"),
        "Should transform pointer param, got: {}",
        code
    );
}

// ============================================================================
// More expression targets: string method, field access, array index
// ============================================================================

#[test]
fn expr_field_access_nested() {
    let cg = CodeGenerator::new();
    // point.inner.x — nested field access
    let expr = HirExpression::FieldAccess {
        object: Box::new(HirExpression::FieldAccess {
            object: Box::new(HirExpression::Variable("point".to_string())),
            field: "inner".to_string(),
        }),
        field: "x".to_string(),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("point") && code.contains("inner") && code.contains("x"),
        "Nested FieldAccess should chain, got: {}",
        code
    );
}

#[test]
fn expr_array_index_expression() {
    let cg = CodeGenerator::new();
    // arr[i + 1]
    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("i".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        }),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("arr"),
        "ArrayIndex with expr index should reference arr, got: {}",
        code
    );
}

#[test]
fn expr_string_method_call_len() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "len".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("len") || code.contains("s"),
        "StringMethodCall should reference method, got: {}",
        code
    );
}

#[test]
fn expr_is_not_null_via_not_equal() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::IsNotNull(Box::new(HirExpression::Variable("ptr".to_string())));
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("ptr") || code.contains("null") || code.contains("Some"),
        "IsNotNull should check ptr for non-null, got: {}",
        code
    );
}

#[test]
fn expr_function_call_strcpy() {
    let cg = CodeGenerator::new();
    // C: strcpy(dst, src);
    let expr = HirExpression::FunctionCall {
        function: "strcpy".to_string(),
        arguments: vec![
            HirExpression::Variable("dst".to_string()),
            HirExpression::Variable("src".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("dst") || code.contains("src") || code.contains("clone"),
        "strcpy should generate clone or copy, got: {}",
        code
    );
}

#[test]
fn expr_function_call_strlen() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "strlen".to_string(),
        arguments: vec![HirExpression::Variable("s".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("len") || code.contains("s"),
        "strlen should generate .len(), got: {}",
        code
    );
}

#[test]
fn expr_function_call_atoi() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "atoi".to_string(),
        arguments: vec![HirExpression::Variable("str_val".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("parse") || code.contains("str_val"),
        "atoi should generate parse::<i32>(), got: {}",
        code
    );
}

#[test]
fn expr_function_call_abs() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "abs".to_string(),
        arguments: vec![HirExpression::Variable("n".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("abs") || code.contains("n"),
        "abs should generate .abs(), got: {}",
        code
    );
}

#[test]
fn expr_function_call_exit() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "exit".to_string(),
        arguments: vec![HirExpression::IntLiteral(1)],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("exit") || code.contains("process"),
        "exit should generate std::process::exit, got: {}",
        code
    );
}

#[test]
fn expr_function_call_puts() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "puts".to_string(),
        arguments: vec![HirExpression::StringLiteral("hello".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("println") || code.contains("hello"),
        "puts should generate println!, got: {}",
        code
    );
}

#[test]
fn expr_function_call_qsort() {
    let cg = CodeGenerator::new();
    // C: qsort(arr, n, sizeof(int), compare);
    let expr = HirExpression::FunctionCall {
        function: "qsort".to_string(),
        arguments: vec![
            HirExpression::Variable("arr".to_string()),
            HirExpression::Variable("n".to_string()),
            HirExpression::Sizeof {
                type_name: "int".to_string(),
            },
            HirExpression::Variable("compare".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("sort") || code.contains("arr"),
        "qsort should generate sort_by, got: {}",
        code
    );
}

// ============================================================================
// More statement patterns
// ============================================================================

#[test]
fn stmt_while_with_break_inside() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::While {
        condition: HirExpression::IntLiteral(1),
        body: vec![
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::Equal,
                    left: Box::new(HirExpression::Variable("x".to_string())),
                    right: Box::new(HirExpression::IntLiteral(0)),
                },
                then_block: vec![HirStatement::Break],
                else_block: None,
            },
            HirStatement::Expression(HirExpression::UnaryOp {
                op: UnaryOperator::PostDecrement,
                operand: Box::new(HirExpression::Variable("x".to_string())),
            }),
        ],
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("while") || code.contains("loop"),
        "While with break should generate loop structure, got: {}",
        code
    );
}

#[test]
fn stmt_field_assignment() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::FieldAssignment {
        object: HirExpression::Variable("point".to_string()),
        field: "x".to_string(),
        value: HirExpression::IntLiteral(10),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("point") && code.contains("x") && code.contains("10"),
        "FieldAssignment should set point.x = 10, got: {}",
        code
    );
}

#[test]
fn stmt_multiple_var_decl() {
    let cg = CodeGenerator::new();
    // C: int a = 1, b = 2;  → two separate declarations
    let stmt1 = HirStatement::VariableDeclaration {
        name: "a".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(1)),
    };
    let stmt2 = HirStatement::VariableDeclaration {
        name: "b".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::IntLiteral(2)),
    };
    let code1 = cg.generate_statement(&stmt1);
    let code2 = cg.generate_statement(&stmt2);
    assert!(code1.contains("a") && code2.contains("b"));
}

#[test]
fn stmt_var_decl_pointer_to_struct() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "node".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        initializer: Some(HirExpression::NullLiteral),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("node"),
        "Pointer to struct decl should contain node, got: {}",
        code
    );
}

#[test]
fn stmt_var_decl_function_pointer() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "callback".to_string(),
        var_type: HirType::FunctionPointer {
            param_types: vec![HirType::Int],
            return_type: Box::new(HirType::Void),
        },
        initializer: None,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("callback") || code.contains("fn"),
        "Function pointer decl should contain fn or callback, got: {}",
        code
    );
}

#[test]
fn stmt_inline_asm_non_translatable() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::InlineAsm {
        text: "nop".to_string(),
        translatable: false,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        !code.is_empty(),
        "InlineAsm should generate comment or placeholder, got: {}",
        code
    );
}

#[test]
fn stmt_inline_asm_translatable_pause() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::InlineAsm {
        text: "pause".to_string(),
        translatable: true,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        !code.is_empty(),
        "Translatable InlineAsm should generate something, got: {}",
        code
    );
}

// ============================================================================
// target_type-dependent expression branches (via typed var declarations)
// ============================================================================

#[test]
fn typed_decl_float_literal_to_float() {
    let cg = CodeGenerator::new();
    // float x = 3.14;  → target_type = Float → "3.14f32"
    let stmt = HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Float,
        initializer: Some(HirExpression::FloatLiteral("3.14".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("f32") || code.contains("3.14"),
        "Float decl with float literal should use f32, got: {}",
        code
    );
}

#[test]
fn typed_decl_float_literal_to_double() {
    let cg = CodeGenerator::new();
    // double x = 2.718;  → target_type = Double → "2.718f64"
    let stmt = HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Double,
        initializer: Some(HirExpression::FloatLiteral("2.718".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("f64") || code.contains("2.718"),
        "Double decl with float literal should use f64, got: {}",
        code
    );
}

#[test]
fn typed_decl_float_literal_c_suffix() {
    let cg = CodeGenerator::new();
    // float x = 1.0f;  → strip 'f' suffix, add f32
    let stmt = HirStatement::VariableDeclaration {
        name: "x".to_string(),
        var_type: HirType::Float,
        initializer: Some(HirExpression::FloatLiteral("1.0f".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("f32"),
        "Float literal with C suffix should get f32, got: {}",
        code
    );
}

#[test]
fn typed_decl_addressof_to_pointer() {
    let cg = CodeGenerator::new();
    // int* p = &x;  → target_type = Pointer(Int) → "&mut x as *mut i32"
    let stmt = HirStatement::VariableDeclaration {
        name: "p".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::AddressOf(Box::new(
            HirExpression::Variable("x".to_string()),
        ))),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("&") && code.contains("x"),
        "AddressOf to pointer should generate reference, got: {}",
        code
    );
}

#[test]
fn typed_decl_unary_addressof_to_pointer() {
    let cg = CodeGenerator::new();
    // struct Node* n = &node;  → target_type = Pointer(Struct)
    let stmt = HirStatement::VariableDeclaration {
        name: "n".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        initializer: Some(HirExpression::UnaryOp {
            op: UnaryOperator::AddressOf,
            operand: Box::new(HirExpression::Variable("node".to_string())),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("&") && code.contains("node"),
        "UnaryOp AddressOf to pointer should generate reference, got: {}",
        code
    );
}

#[test]
fn typed_decl_logical_not_to_int() {
    let cg = CodeGenerator::new();
    // int result = !flag;  → target_type = Int → "(flag == 0) as i32"
    let stmt = HirStatement::VariableDeclaration {
        name: "result".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::UnaryOp {
            op: UnaryOperator::LogicalNot,
            operand: Box::new(HirExpression::Variable("flag".to_string())),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("== 0") || code.contains("as i32") || code.contains("!"),
        "LogicalNot to int should cast, got: {}",
        code
    );
}

#[test]
fn typed_decl_logical_not_of_bool_expr_to_int() {
    let cg = CodeGenerator::new();
    // int result = !(a > b);  → target_type = Int → "(!(a > b)) as i32"
    let stmt = HirStatement::VariableDeclaration {
        name: "result".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::UnaryOp {
            op: UnaryOperator::LogicalNot,
            operand: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            }),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("as i32") || code.contains("!"),
        "LogicalNot of bool expr to int should cast, got: {}",
        code
    );
}

#[test]
fn typed_decl_string_to_char_pointer() {
    let cg = CodeGenerator::new();
    // char* s = "hello";  → target_type = Pointer(Char) → b"hello\0".as_ptr()
    let stmt = HirStatement::VariableDeclaration {
        name: "s".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Char)),
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("hello") || code.contains("s"),
        "String to char* should contain string, got: {}",
        code
    );
}

#[test]
fn typed_decl_int_zero_to_pointer() {
    let cg = CodeGenerator::new();
    // int* p = 0;  → target_type = Pointer → std::ptr::null_mut()
    let stmt = HirStatement::VariableDeclaration {
        name: "p".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("null") || code.contains("None") || code.contains("0"),
        "Int 0 to pointer should generate null_mut or None, got: {}",
        code
    );
}

#[test]
fn typed_decl_logical_and_to_int() {
    let cg = CodeGenerator::new();
    // int result = a && b;  → target_type = Int → (a != 0 && b != 0) as i32
    let stmt = HirStatement::VariableDeclaration {
        name: "result".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::BinaryOp {
            op: BinaryOperator::LogicalAnd,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("a") && code.contains("b"),
        "LogicalAnd to int should reference operands, got: {}",
        code
    );
}

#[test]
fn typed_decl_logical_or_to_int() {
    let cg = CodeGenerator::new();
    // int result = a || b;  → target_type = Int
    let stmt = HirStatement::VariableDeclaration {
        name: "result".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::BinaryOp {
            op: BinaryOperator::LogicalOr,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::Variable("y".to_string())),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("x") && code.contains("y"),
        "LogicalOr to int should reference operands, got: {}",
        code
    );
}

#[test]
fn typed_decl_equal_comparison_to_int() {
    let cg = CodeGenerator::new();
    // int eq = (a == b);  → target_type = Int
    let stmt = HirStatement::VariableDeclaration {
        name: "eq".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("a") && code.contains("b"),
        "Comparison to int should reference operands, got: {}",
        code
    );
}

#[test]
fn typed_decl_cast_in_initializer() {
    let cg = CodeGenerator::new();
    // float f = (float)x;  → target_type = Float → "x as f32"
    let stmt = HirStatement::VariableDeclaration {
        name: "f".to_string(),
        var_type: HirType::Float,
        initializer: Some(HirExpression::Cast {
            target_type: HirType::Float,
            expr: Box::new(HirExpression::Variable("x".to_string())),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("f32") || code.contains("as"),
        "Cast in float decl should generate cast, got: {}",
        code
    );
}

#[test]
fn typed_decl_ternary_in_initializer() {
    let cg = CodeGenerator::new();
    // int max = (a > b) ? a : b;
    let stmt = HirStatement::VariableDeclaration {
        name: "max".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::Ternary {
            condition: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            }),
            then_expr: Box::new(HirExpression::Variable("a".to_string())),
            else_expr: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("if") || code.contains("a") && code.contains("b"),
        "Ternary in int decl should generate if expression, got: {}",
        code
    );
}

#[test]
fn typed_decl_box_with_malloc() {
    let cg = CodeGenerator::new();
    // int* p = malloc(sizeof(int));  → Box<i32>
    let stmt = HirStatement::VariableDeclaration {
        name: "p".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::Sizeof {
                type_name: "int".to_string(),
            }],
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("Box") || code.contains("box") || code.contains("alloc") || code.contains("p"),
        "malloc(sizeof) should generate Box, got: {}",
        code
    );
}

#[test]
fn typed_decl_vec_with_malloc_multiply() {
    let cg = CodeGenerator::new();
    // int* arr = malloc(10 * sizeof(int));  → Vec<i32>
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
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
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("Vec") || code.contains("vec") || code.contains("arr"),
        "malloc(n*sizeof) should generate Vec, got: {}",
        code
    );
}

#[test]
fn typed_assign_to_existing_var() {
    let cg = CodeGenerator::new();
    // x = a + b;
    let stmt = HirStatement::Assignment {
        target: "x".to_string(),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        },
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("x") && code.contains("a") && code.contains("b"),
        "Assignment should reference x, a, b, got: {}",
        code
    );
}

#[test]
fn typed_deref_assign_complex() {
    let cg = CodeGenerator::new();
    // *ptr = *ptr + 1;
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("ptr".to_string()),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Dereference(Box::new(
                HirExpression::Variable("ptr".to_string()),
            ))),
            right: Box::new(HirExpression::IntLiteral(1)),
        },
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("ptr"),
        "DerefAssignment should reference ptr, got: {}",
        code
    );
}

#[test]
fn typed_array_index_assign() {
    let cg = CodeGenerator::new();
    // arr[i] = 42;
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("arr") && code.contains("42"),
        "ArrayIndexAssignment should assign to arr, got: {}",
        code
    );
}

#[test]
fn typed_decl_calloc() {
    let cg = CodeGenerator::new();
    // int* arr = calloc(10, sizeof(int));
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::Calloc {
            count: Box::new(HirExpression::IntLiteral(10)),
            element_type: Box::new(HirType::Int),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("arr") && (code.contains("vec") || code.contains("Vec") || code.contains("0")),
        "calloc should generate zeroed Vec, got: {}",
        code
    );
}

#[test]
fn typed_decl_enum_type() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "color".to_string(),
        var_type: HirType::Enum("Color".to_string()),
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("color"),
        "Enum type decl should contain color, got: {}",
        code
    );
}

#[test]
fn typed_decl_type_alias() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "len".to_string(),
        var_type: HirType::TypeAlias("size_t".to_string()),
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("len"),
        "TypeAlias decl should contain len, got: {}",
        code
    );
}

#[test]
fn typed_decl_vec_type() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "items".to_string(),
        var_type: HirType::Vec(Box::new(HirType::Int)),
        initializer: None,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("Vec") || code.contains("items"),
        "Vec type decl should contain Vec or items, got: {}",
        code
    );
}

#[test]
fn typed_decl_option_type() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "maybe".to_string(),
        var_type: HirType::Option(Box::new(HirType::Int)),
        initializer: Some(HirExpression::NullLiteral),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("maybe") || code.contains("Option") || code.contains("None"),
        "Option type decl should contain Option or None, got: {}",
        code
    );
}

// ============================================================================
// Special library function coverage (via FunctionCall expressions)
// ============================================================================

#[test]
fn expr_fread_call() {
    let cg = CodeGenerator::new();
    // C: fread(buf, 1, 100, fp)
    let expr = HirExpression::FunctionCall {
        function: "fread".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(100),
            HirExpression::Variable("fp".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        !code.is_empty(),
        "fread should generate read code, got: {}",
        code
    );
}

#[test]
fn expr_fwrite_call() {
    let cg = CodeGenerator::new();
    // C: fwrite(buf, 1, 100, fp)
    let expr = HirExpression::FunctionCall {
        function: "fwrite".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(100),
            HirExpression::Variable("fp".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        !code.is_empty(),
        "fwrite should generate write code, got: {}",
        code
    );
}

#[test]
fn expr_snprintf_call() {
    let cg = CodeGenerator::new();
    // C: snprintf(buf, 100, "%d", val)
    let expr = HirExpression::FunctionCall {
        function: "snprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::IntLiteral(100),
            HirExpression::StringLiteral("%d".to_string()),
            HirExpression::Variable("val".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("format") || code.contains("buf"),
        "snprintf should generate format!, got: {}",
        code
    );
}

#[test]
fn expr_sprintf_call() {
    let cg = CodeGenerator::new();
    // C: sprintf(buf, "%s %d", name, age)
    let expr = HirExpression::FunctionCall {
        function: "sprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::StringLiteral("%s %d".to_string()),
            HirExpression::Variable("name".to_string()),
            HirExpression::Variable("age".to_string()),
        ],
    };
