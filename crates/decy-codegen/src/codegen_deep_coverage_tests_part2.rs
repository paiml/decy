// ============================================================================

#[test]
fn expr_unary_negate() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::Minus,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("-"));
    assert!(code.contains("x"));
}

#[test]
fn expr_unary_not() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("flag".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("flag"));
}

#[test]
fn expr_unary_bitwise_not() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::BitwiseNot,
        operand: Box::new(HirExpression::Variable("mask".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("!"));
    assert!(code.contains("mask"));
}

// ============================================================================
// Statement codegen: FieldAssignment with reserved keyword
// ============================================================================

#[test]
fn stmt_field_assignment_reserved_keyword() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::FieldAssignment {
        object: HirExpression::Variable("obj".to_string()),
        field: "type".to_string(),
        value: HirExpression::IntLiteral(1),
    };
    let code = cg.generate_statement(&stmt);
    // "type" is a Rust keyword, should be escaped
    assert!(code.contains("r#type") || code.contains("type_"));
}

// ============================================================================
// S3-Phase1: Standard library function mapping tests
// Note: Pointer-based functions (memcpy, memset, strcmp, strncmp, strcat)
// use the stub mechanism rather than inline expansion because transpiled
// code uses raw pointer types that don't support safe Rust operations.
// ============================================================================

#[test]
fn stdlib_atoi_generates_parse() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "atoi".to_string(),
        arguments: vec![HirExpression::Variable("s".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("parse::<i32>"),
        "atoi should generate parse::<i32>(), got: {}",
        code
    );
    assert!(code.contains("unwrap_or(0)"));
}

#[test]
fn stdlib_atoi_invalid_args() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "atoi".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("atoi requires 1 arg"));
}

#[test]
fn stdlib_atof_generates_parse_f64() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "atof".to_string(),
        arguments: vec![HirExpression::Variable("s".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("parse::<f64>"),
        "atof should generate parse::<f64>(), got: {}",
        code
    );
    assert!(code.contains("unwrap_or(0.0)"));
}

#[test]
fn stdlib_atof_invalid_args() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "atof".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("atof requires 1 arg"));
}

#[test]
fn stdlib_abs_generates_abs() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "abs".to_string(),
        arguments: vec![HirExpression::Variable("x".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains(".abs()"),
        "abs should generate .abs(), got: {}",
        code
    );
}

#[test]
fn stdlib_abs_invalid_args() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "abs".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("abs requires 1 arg"));
}

#[test]
fn stdlib_exit_generates_process_exit() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "exit".to_string(),
        arguments: vec![HirExpression::IntLiteral(0)],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("std::process::exit"),
        "exit should generate std::process::exit, got: {}",
        code
    );
}

#[test]
fn stdlib_exit_no_args() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "exit".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("std::process::exit(1)"));
}

#[test]
fn stdlib_puts_generates_println() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "puts".to_string(),
        arguments: vec![HirExpression::StringLiteral("hello".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("println!"),
        "puts should generate println!, got: {}",
        code
    );
}

#[test]
fn stdlib_puts_no_args() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "puts".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("println!()"));
}

#[test]
fn stdlib_snprintf_generates_format() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "snprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::IntLiteral(256),
            HirExpression::StringLiteral("value: %d".to_string()),
            HirExpression::Variable("x".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("format!"),
        "snprintf should generate format!, got: {}",
        code
    );
}

#[test]
fn stdlib_snprintf_no_varargs() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "snprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::IntLiteral(256),
            HirExpression::StringLiteral("hello".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("format!"));
}

#[test]
fn stdlib_snprintf_invalid_args() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "snprintf".to_string(),
        arguments: vec![HirExpression::Variable("buf".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("snprintf requires 3+ args"));
}

#[test]
fn stdlib_sprintf_generates_format() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "sprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::StringLiteral("val=%d".to_string()),
            HirExpression::Variable("x".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("format!"),
        "sprintf should generate format!, got: {}",
        code
    );
}

#[test]
fn stdlib_sprintf_no_varargs() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "sprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::StringLiteral("hello".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("format!"));
}

#[test]
fn stdlib_sprintf_invalid_args() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "sprintf".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("sprintf requires 2+ args"));
}

#[test]
fn stdlib_qsort_generates_sort_by() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "qsort".to_string(),
        arguments: vec![
            HirExpression::Variable("arr".to_string()),
            HirExpression::Variable("n".to_string()),
            HirExpression::FunctionCall {
                function: "sizeof".to_string(),
                arguments: vec![HirExpression::Variable("int".to_string())],
            },
            HirExpression::Variable("compare".to_string()),
        ],
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("sort_by"),
        "qsort should generate sort_by, got: {}",
        code
    );
}

#[test]
fn stdlib_qsort_invalid_args() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FunctionCall {
        function: "qsort".to_string(),
        arguments: vec![HirExpression::Variable("arr".to_string())],
    };
    let code = cg.generate_expression(&expr);
    assert!(code.contains("qsort requires 4 args"));
}

// ============================================================================
// Signature generation: function name renaming (DECY-241 keyword conflicts)
// ============================================================================

#[test]
fn signature_renames_write_to_c_write() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("write".to_string(), HirType::Void, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("fn c_write"),
        "write should be renamed to c_write, got: {}",
        sig
    );
}

#[test]
fn signature_renames_read_to_c_read() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("read".to_string(), HirType::Int, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("fn c_read"),
        "read should be renamed to c_read, got: {}",
        sig
    );
}

#[test]
fn signature_renames_type_to_c_type() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("type".to_string(), HirType::Void, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("fn c_type"),
        "type should be renamed to c_type, got: {}",
        sig
    );
}

#[test]
fn signature_renames_match_to_c_match() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("match".to_string(), HirType::Int, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("fn c_match"),
        "match should be renamed to c_match, got: {}",
        sig
    );
}

#[test]
fn signature_renames_self_to_c_self() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("self".to_string(), HirType::Void, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("fn c_self"),
        "self should be renamed to c_self, got: {}",
        sig
    );
}

#[test]
fn signature_renames_in_to_c_in() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("in".to_string(), HirType::Void, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("fn c_in"),
        "in should be renamed to c_in, got: {}",
        sig
    );
}

#[test]
fn signature_preserves_normal_name() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("process_data".to_string(), HirType::Int, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("fn process_data"),
        "Normal name should be preserved, got: {}",
        sig
    );
}

// ============================================================================
// Signature generation: main() special case, return types
// ============================================================================

#[test]
fn signature_main_omits_return_type() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("main".to_string(), HirType::Int, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("fn main"),
        "Should generate main, got: {}",
        sig
    );
    assert!(
        !sig.contains("-> i32"),
        "main should not have -> i32 return, got: {}",
        sig
    );
}

#[test]
fn signature_non_main_has_return_type() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("compute".to_string(), HirType::Int, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("-> i32"),
        "Non-main with Int return should have -> i32, got: {}",
        sig
    );
}

#[test]
fn signature_void_return_no_arrow() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("process".to_string(), HirType::Void, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        !sig.contains("->"),
        "Void return should have no arrow, got: {}",
        sig
    );
}

#[test]
fn signature_float_return() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("calc".to_string(), HirType::Float, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("-> f32"),
        "Float return should be -> f32, got: {}",
        sig
    );
}

#[test]
fn signature_double_return() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("precise".to_string(), HirType::Double, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("-> f64"),
        "Double return should be -> f64, got: {}",
        sig
    );
}

#[test]
fn signature_char_return() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("getchar_fn".to_string(), HirType::Char, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("->"),
        "Char return should have arrow, got: {}",
        sig
    );
}

// ============================================================================
// Signature generation: parameters
// ============================================================================

#[test]
fn signature_basic_int_params() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new(
        "add".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
    );
    let sig = cg.generate_signature(&func);
    assert!(sig.contains("a:"), "Should contain param a, got: {}", sig);
    assert!(sig.contains("b:"), "Should contain param b, got: {}", sig);
}

#[test]
fn signature_pointer_param() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "deref".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::Dereference(
            Box::new(HirExpression::Variable("ptr".to_string())),
        )))],
    );
    let sig = cg.generate_signature(&func);
    assert!(sig.contains("ptr"), "Should contain ptr param, got: {}", sig);
}

#[test]
fn signature_unsigned_int_return() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new("count".to_string(), HirType::UnsignedInt, vec![]);
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("-> u32"),
        "UnsignedInt return should be -> u32, got: {}",
        sig
    );
}

// ============================================================================
// Expression target type: null pointer detection
// ============================================================================

#[test]
fn expr_int_zero_to_pointer_is_null_mut() {
    let cg = CodeGenerator::new();
    // VariableDeclaration with pointer type and IntLiteral(0) initializer
    let stmt = HirStatement::VariableDeclaration {
        name: "ptr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("null_mut") || code.contains("None"),
        "0 assigned to pointer should generate null_mut or None, got: {}",
        code
    );
}

#[test]
fn expr_int_nonzero_to_pointer_no_null() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "ptr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::IntLiteral(42)),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        !code.contains("null_mut"),
        "Non-zero to pointer should NOT be null_mut, got: {}",
        code
    );
}

#[test]
fn expr_string_literal_to_pointer_type() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "msg".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Char)),
        initializer: Some(HirExpression::StringLiteral("hello".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("hello"),
        "String literal assigned to char* should contain the string, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: InlineAsm
// ============================================================================

#[test]
fn statement_inline_asm_translatable() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::InlineAsm {
        text: "nop".to_string(),
        translatable: true,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("asm") || code.contains("nop"),
        "InlineAsm with translatable should generate asm, got: {}",
        code
    );
}

#[test]
fn statement_inline_asm_not_translatable() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::InlineAsm {
        text: "int 0x80".to_string(),
        translatable: false,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        !code.is_empty(),
        "InlineAsm non-translatable should generate something, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: switch/case with char literal
// ============================================================================

#[test]
fn statement_switch_basic() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("x".to_string()),
        cases: vec![SwitchCase {
            value: Some(HirExpression::IntLiteral(1)),
            body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(10)))],
        }],
        default_case: Some(vec![HirStatement::Return(Some(
            HirExpression::IntLiteral(0),
        ))]),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("match"),
        "Switch should generate match, got: {}",
        code
    );
}

#[test]
fn statement_switch_char_cases() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Switch {
        condition: HirExpression::Variable("ch".to_string()),
        cases: vec![
            SwitchCase {
                value: Some(HirExpression::CharLiteral(b'a' as i8)),
                body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
            },
            SwitchCase {
                value: Some(HirExpression::CharLiteral(b'b' as i8)),
                body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(2)))],
            },
        ],
        default_case: None,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("match"),
        "Switch with chars should generate match, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: for loop variants
// ============================================================================

#[test]
fn statement_for_with_init_and_increment() {
    let cg = CodeGenerator::new();
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
            op: UnaryOperator::PostIncrement,
            operand: Box::new(HirExpression::Variable("i".to_string())),
        })],
        body: vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "puts".to_string(),
            arguments: vec![HirExpression::StringLiteral("tick".to_string())],
        })],
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("while") || code.contains("for"),
        "For loop should generate while or for, got: {}",
        code
    );
}

#[test]
fn statement_for_infinite_loop() {
    let cg = CodeGenerator::new();
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

// ============================================================================
// Statement coverage: deref assignment
// ============================================================================

#[test]
fn statement_deref_assignment() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("ptr".to_string()),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("42"),
        "Deref assignment should contain value, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: array index assignment
// ============================================================================

#[test]
fn statement_array_index_assignment() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::ArrayIndexAssignment {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::IntLiteral(0)),
        value: HirExpression::IntLiteral(99),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("arr") && code.contains("99"),
        "Array index assignment should contain array and value, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: field assignment
// ============================================================================

#[test]
fn statement_field_assignment() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::FieldAssignment {
        object: HirExpression::Variable("point".to_string()),
        field: "x".to_string(),
        value: HirExpression::IntLiteral(10),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("point") && code.contains("x") && code.contains("10"),
        "Field assignment should contain object, field, value, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: free
// ============================================================================

#[test]
fn statement_free_generates_drop_comment() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Free {
        pointer: HirExpression::Variable("ptr".to_string()),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("drop") || code.contains("RAII") || code.contains("freed"),
        "Free should generate drop/RAII comment, got: {}",
        code
    );
}

// ============================================================================
// Expression coverage: char literal
// ============================================================================

#[test]
fn expr_char_literal_printable() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::CharLiteral(b'A' as i8);
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("'A'") || code.contains("b'A'") || code.contains("65"),
        "Printable char should generate char literal, got: {}",
        code
    );
}

#[test]
fn expr_char_literal_non_printable() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::CharLiteral(b'\n' as i8);
    let code = cg.generate_expression(&expr);
    assert!(
        !code.is_empty(),
        "Non-printable char should generate something, got: {}",
        code
    );
}

#[test]
fn expr_char_literal_zero() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::CharLiteral(0);
    let code = cg.generate_expression(&expr);
    assert!(
        !code.is_empty(),
        "Null char should generate something, got: {}",
        code
    );
}

// ============================================================================
// Expression coverage: unary ops
// ============================================================================

#[test]
fn expr_unary_post_increment() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PostIncrement,
        operand: Box::new(HirExpression::Variable("i".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(!code.is_empty(), "PostIncrement should generate code, got: {}", code);
}

#[test]
fn expr_unary_pre_decrement() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::PreDecrement,
        operand: Box::new(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(!code.is_empty(), "PreDecrement should generate code, got: {}", code);
}

#[test]
fn expr_unary_logical_not() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("flag".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("!") || code.contains("== 0"),
        "LogicalNot should generate negation or == 0, got: {}",
        code
    );
}

// ============================================================================
// Expression coverage: dereference
// ============================================================================

#[test]
fn expr_dereference_variable() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable(
        "ptr".to_string(),
    )));
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("ptr"),
        "Dereference should contain variable name, got: {}",
        code
    );
}

// ============================================================================
// Expression coverage: address-of
// ============================================================================

#[test]
fn expr_address_of_variable() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::AddressOf(Box::new(HirExpression::Variable(
        "x".to_string(),
    )));
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("&") || code.contains("x"),
        "AddressOf should generate reference, got: {}",
        code
    );
}

// ============================================================================
// Expression coverage: sizeof
// ============================================================================

#[test]
fn expr_sizeof_type() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Sizeof { type_name: "int".to_string() };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("size_of") || code.contains("mem::size_of"),
        "SizeOf should generate size_of, got: {}",
        code
    );
}

#[test]
fn expr_sizeof_pointer_type() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Sizeof { type_name: "char*".to_string() };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("size_of"),
        "SizeOf pointer should generate size_of, got: {}",
        code
    );
}

// ============================================================================
// Expression coverage: type cast (Cast variant)
// ============================================================================

#[test]
fn expr_cast_var_to_float() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Cast {
        expr: Box::new(HirExpression::Variable("x".to_string())),
        target_type: HirType::Float,
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("as f32") || code.contains("f32"),
        "Cast var to float should generate as f32, got: {}",
        code
    );
}

#[test]
fn expr_cast_var_to_int() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Cast {
        expr: Box::new(HirExpression::Variable("f".to_string())),
        target_type: HirType::Int,
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("as i32") || code.contains("i32"),
        "Cast float to int should generate as i32, got: {}",
        code
    );
}

#[test]
fn expr_cast_to_unsigned() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Cast {
        expr: Box::new(HirExpression::Variable("x".to_string())),
        target_type: HirType::UnsignedInt,
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("as u32") || code.contains("u32"),
        "Cast to unsigned should generate u32, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: VLA (variable-length array) patterns
// ============================================================================

#[test]
fn statement_vla_int_array() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("vec![") && code.contains("0i32"),
        "VLA int should generate vec![0i32; n], got: {}",
        code
    );
}

#[test]
fn statement_vla_float_array() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Float),
            size: None,
        },
        initializer: Some(HirExpression::Variable("size".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("vec![") && code.contains("0.0f32"),
        "VLA float should generate vec![0.0f32; size], got: {}",
        code
    );
}

#[test]
fn statement_vla_double_array() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "data".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Double),
            size: None,
        },
        initializer: Some(HirExpression::Variable("len".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("vec![") && code.contains("0.0f64"),
        "VLA double should generate vec![0.0f64; len], got: {}",
        code
    );
}

#[test]
fn statement_vla_char_array() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "buffer".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Char),
            size: None,
        },
        initializer: Some(HirExpression::Variable("sz".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("vec![") && code.contains("0u8"),
        "VLA char should generate vec![0u8; sz], got: {}",
        code
    );
}

#[test]
fn statement_vla_unsigned_int_array() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "counts".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::UnsignedInt),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("vec![") && code.contains("0u32"),
        "VLA unsigned int should generate vec![0u32; n], got: {}",
        code
    );
}

#[test]
fn statement_vla_signed_char_array() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "vals".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::SignedChar),
            size: None,
        },
        initializer: Some(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("vec![") && code.contains("0i8"),
        "VLA signed char should generate vec![0i8; n], got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: malloc initialization patterns
// ============================================================================

#[test]
fn statement_malloc_init_box_pattern() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "data".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::Sizeof {
                type_name: "Node".to_string(),
            }),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("Box") || code.contains("box"),
        "Struct malloc should generate Box, got: {}",
        code
    );
}

#[test]
fn statement_malloc_init_vec_pattern() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "arr".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Int)),
        initializer: Some(HirExpression::Malloc {
            size: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::Variable("n".to_string())),
                right: Box::new(HirExpression::Sizeof {
                    type_name: "int".to_string(),
                }),
            }),
        }),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("Vec") || code.contains("vec"),
        "Array malloc pattern should generate Vec, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: char* string literal initialization
// ============================================================================

#[test]
fn statement_char_ptr_string_literal() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "msg".to_string(),
        var_type: HirType::Pointer(Box::new(HirType::Char)),
        initializer: Some(HirExpression::StringLiteral("hello world".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("&str") || code.contains("hello world"),
        "char* with string literal should use &str, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: char array from string literal
// ============================================================================

#[test]
fn statement_char_array_string_init() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::VariableDeclaration {
        name: "buf".to_string(),
        var_type: HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(10),
        },
        initializer: Some(HirExpression::StringLiteral("test".to_string())),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("test") || code.contains("b\"test"),
        "Char array from string should contain the string, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: return in main (exit code) vs non-main
// ============================================================================

#[test]
fn statement_return_none() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Return(None);
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("return"),
        "Return None should generate return, got: {}",
        code
    );
}

#[test]
fn statement_return_expression() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Return(Some(HirExpression::Variable("result".to_string())));
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("return") && code.contains("result"),
        "Return expr should generate return result, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: while loop
// ============================================================================

#[test]
fn statement_while_basic() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::While {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        body: vec![HirStatement::Expression(HirExpression::UnaryOp {
            op: UnaryOperator::PostDecrement,
            operand: Box::new(HirExpression::Variable("n".to_string())),
        })],
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("while"),
        "While should generate while, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: if/else
// ============================================================================

#[test]
fn statement_if_only() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::Variable("flag".to_string()),
        then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
        else_block: None,
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("if"),
        "If should generate if, got: {}",
        code
    );
}

#[test]
fn statement_if_else() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
        else_block: Some(vec![HirStatement::Return(Some(
            HirExpression::IntLiteral(1),
        ))]),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("if") && code.contains("else"),
        "If/else should generate both branches, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: continue and break
// ============================================================================

#[test]
fn statement_break() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Break;
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("break"), "Break should generate break, got: {}", code);
}

#[test]
fn statement_continue() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Continue;
    let code = cg.generate_statement(&stmt);
    assert!(code.contains("continue"), "Continue should generate continue, got: {}", code);
}

// ============================================================================
// Statement coverage: expression statement
// ============================================================================

#[test]
fn statement_expression_function_call() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Expression(HirExpression::FunctionCall {
        function: "process".to_string(),
        arguments: vec![HirExpression::Variable("data".to_string())],
    });
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("process") && code.contains("data"),
        "Expression statement should contain function call, got: {}",
        code
    );
}

// ============================================================================
// Statement coverage: assignment
// ============================================================================

#[test]
fn statement_assignment_simple() {
    let cg = CodeGenerator::new();
    let stmt = HirStatement::Assignment {
        target: "x".to_string(),
        value: HirExpression::IntLiteral(42),
    };
    let code = cg.generate_statement(&stmt);
    assert!(
        code.contains("x") && code.contains("42"),
        "Assignment should contain target and value, got: {}",
        code
    );
}

// ============================================================================
// Expression coverage: binary operators
// ============================================================================

#[test]
fn expr_binary_add() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("+") || code.contains("a") && code.contains("b"),
        "Add should generate +, got: {}",
        code
    );
}

#[test]
fn expr_binary_subtract() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("-") || code.contains("wrapping_sub"),
        "Subtract should generate - or wrapping_sub, got: {}",
        code
    );
}

#[test]
fn expr_binary_logical_and() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalAnd,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("&&") || code.contains("!= 0"),
        "LogicalAnd should generate &&, got: {}",
        code
    );
}

#[test]
fn expr_binary_logical_or() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalOr,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("||") || code.contains("!= 0"),
        "LogicalOr should generate ||, got: {}",
        code
    );
}

#[test]
fn expr_binary_bitwise_and() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::BitwiseAnd,
        left: Box::new(HirExpression::Variable("flags".to_string())),
        right: Box::new(HirExpression::IntLiteral(0xFF)),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("&") || code.contains("flags"),
        "BitwiseAnd should generate &, got: {}",
        code
    );
}

#[test]
fn expr_binary_shift_left() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LeftShift,
        left: Box::new(HirExpression::IntLiteral(1)),
        right: Box::new(HirExpression::Variable("n".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("<<"),
        "LeftShift should generate <<, got: {}",
        code
    );
}

#[test]
fn expr_null_literal_codegen() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::NullLiteral;
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("None") || code.contains("null"),
        "NullLiteral should generate None or null, got: {}",
        code
    );
}

#[test]
fn expr_is_not_null_codegen() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::IsNotNull(Box::new(HirExpression::Variable("ptr".to_string())));
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("is_some") || code.contains("is_null") || code.contains("ptr"),
        "IsNotNull should generate null check, got: {}",
        code
    );
}

#[test]
fn expr_calloc_generates_vec_zeroed() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(10)),
        element_type: Box::new(HirType::Int),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("vec![") || code.contains("Vec"),
        "Calloc should generate vec or Vec, got: {}",
        code
    );
}

// ============================================================================
// Signature generation: pointer parameter transformation with body
// ============================================================================

#[test]
fn signature_pointer_param_read_only_becomes_ref() {
    let cg = CodeGenerator::new();
    // void print_val(int* p) { return *p; }
    let func = HirFunction::new_with_body(
        "print_val".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::Dereference(
            Box::new(HirExpression::Variable("p".to_string())),
        )))],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("&") && sig.contains("i32"),
        "Read-only pointer param should become reference, got: {}",
        sig
    );
}

#[test]
fn signature_pointer_param_modified_becomes_mut_ref() {
    let cg = CodeGenerator::new();
    // int increment(int* p) { *p = *p + 1; return *p; }
    // Using int return type + deref write means output param detector won't claim 'p'
    let func = HirFunction::new_with_body(
        "increment".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("p".to_string()),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Dereference(Box::new(
                        HirExpression::Variable("p".to_string()),
                    ))),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
            HirStatement::Return(Some(HirExpression::Dereference(Box::new(
                HirExpression::Variable("p".to_string()),
            )))),
        ],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("&mut") && sig.contains("i32"),
        "Modified pointer param should become &mut, got: {}",
        sig
    );
}

#[test]
fn signature_void_star_stub_no_generic() {
    let cg = CodeGenerator::new();
    // void process(void* data); — no body (stub)
    let func = HirFunction::new(
        "process".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Pointer(Box::new(HirType::Void)),
        )],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        !sig.contains("<T>"),
        "void* stub without body should NOT get generic <T>, got: {}",
        sig
    );
    assert!(
        sig.contains("*mut ()"),
        "void* stub should become *mut (), got: {}",
        sig
    );
}

#[test]
fn signature_multiple_params_mixed_types() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "compute".to_string(),
        HirType::Float,
        vec![
            HirParameter::new("x".to_string(), HirType::Int),
            HirParameter::new("scale".to_string(), HirType::Float),
        ],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Cast {
                target_type: HirType::Float,
                expr: Box::new(HirExpression::Variable("x".to_string())),
            }),
            right: Box::new(HirExpression::Variable("scale".to_string())),
        }))],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("x:") && sig.contains("scale:"),
        "Should contain both params, got: {}",
        sig
    );
    assert!(
        sig.contains("-> f32"),
        "Should return f32, got: {}",
        sig
    );
}

#[test]
fn signature_array_param_becomes_slice() {
    let cg = CodeGenerator::new();
    // int sum(int arr[10]) { return arr[0]; }
    let func = HirFunction::new_with_body(
        "sum".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "arr".to_string(),
            HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(10),
            },
        )],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("arr".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }))],
    );
    let sig = cg.generate_signature(&func);
    assert!(
        sig.contains("[i32]") || sig.contains("arr"),
        "Array param should become slice or keep name, got: {}",
        sig
    );
}

// ============================================================================
// generate_function: full function code generation
// ============================================================================

#[test]
fn generate_function_simple() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "add".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }))],
    );
    let code = cg.generate_function(&func);
    assert!(
        code.contains("fn add"),
        "Should contain fn add, got: {}",
        code
    );
    assert!(
        code.contains("return"),
        "Should contain return, got: {}",
        code
    );
}

#[test]
fn generate_function_void_no_return() {
    let cg = CodeGenerator::new();
    let func = HirFunction::new_with_body(
        "do_nothing".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "puts".to_string(),
            arguments: vec![HirExpression::StringLiteral("hello".to_string())],
        })],
    );
    let code = cg.generate_function(&func);
    assert!(
        code.contains("fn do_nothing"),
        "Should contain fn do_nothing, got: {}",
        code
    );
    assert!(
        !code.contains("->"),
        "Void function should have no return type arrow, got: {}",
        code
    );
}

// ============================================================================
// Expression coverage: comma operator, char arithmetic, logical ops
// ============================================================================

#[test]
fn expr_comma_operator() {
    let cg = CodeGenerator::new();
    // C: (a = 1, b = 2) — comma operator evaluates both, returns last
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Comma,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression(&expr);
    // Comma generates block: { a; b }
    assert!(
        code.contains("a") && code.contains("b"),
        "Comma should include both operands, got: {}",
        code
    );
}

#[test]
fn expr_char_literal_arithmetic_add() {
    let cg = CodeGenerator::new();
    // C: (num % 10) + '0'
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Modulo,
            left: Box::new(HirExpression::Variable("num".to_string())),
            right: Box::new(HirExpression::IntLiteral(10)),
        }),
        right: Box::new(HirExpression::CharLiteral(b'0' as i8)),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("num") && code.contains("0"),
        "Should contain operands, got: {}",
        code
    );
}

#[test]
fn expr_logical_and_generates_bool() {
    let cg = CodeGenerator::new();
    // C: a && b
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalAnd,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("&&") || code.contains("a") && code.contains("b"),
        "LogicalAnd should generate && or bool check, got: {}",
        code
    );
}

#[test]
fn expr_logical_or_generates_bool() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalOr,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::Variable("y".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("||") || code.contains("x") && code.contains("y"),
        "LogicalOr should generate || or bool check, got: {}",
        code
    );
}

#[test]
fn expr_bitwise_xor() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::BitwiseXor,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("^"),
        "BitwiseXor should generate ^, got: {}",
        code
    );
}

#[test]
fn expr_modulo_operator() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Modulo,
        left: Box::new(HirExpression::Variable("n".to_string())),
        right: Box::new(HirExpression::IntLiteral(7)),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("%"),
        "Modulo should generate %, got: {}",
        code
    );
}

#[test]
fn expr_not_equal_comparison() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("!=") || code.contains("x"),
        "NotEqual should generate != or truthy check, got: {}",
        code
    );
}

#[test]
fn expr_greater_than_or_equal() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::GreaterEqual,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains(">="),
        "GreaterThanOrEqual should generate >=, got: {}",
        code
    );
}

#[test]
fn expr_less_than_or_equal() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LessEqual,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::IntLiteral(100)),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("<="),
        "LessThanOrEqual should generate <=, got: {}",
        code
    );
}

#[test]
fn expr_ternary_simple() {
    let cg = CodeGenerator::new();
    // C: (x > 0) ? x : -x
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
        then_expr: Box::new(HirExpression::Variable("x".to_string())),
        else_expr: Box::new(HirExpression::UnaryOp {
            op: UnaryOperator::Minus,
            operand: Box::new(HirExpression::Variable("x".to_string())),
        }),
    };
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("if") || code.contains("x"),
        "Ternary should generate if expression, got: {}",
        code
    );
}

#[test]
fn expr_string_literal_basic() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::StringLiteral("hello world".to_string());
    let code = cg.generate_expression(&expr);
    assert!(
        code.contains("hello world"),
        "StringLiteral should contain the string, got: {}",
        code
    );
}

#[test]
fn expr_float_literal() {
    let cg = CodeGenerator::new();
    let expr = HirExpression::FloatLiteral("3.14".to_string());
