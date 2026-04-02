#[test]
fn expr_context_post_dec_pointer() {
    // ptr-- → wrapping_sub for pointers
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("cur".to_string(), HirType::Pointer(Box::new(HirType::Char)));
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::PostDecrement,
        operand: Box::new(HirExpression::Variable("cur".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("wrapping_sub"),
        "Pointer post-dec → wrapping_sub: {}",
        code
    );
}

#[test]
fn expr_context_pre_inc_pointer() {
    // ++ptr → wrapping_add for pointers
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::PreIncrement,
        operand: Box::new(HirExpression::Variable("p".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("wrapping_add"),
        "Pointer pre-inc → wrapping_add: {}",
        code
    );
}

#[test]
fn expr_context_pre_dec_pointer() {
    // --ptr → wrapping_sub for pointers
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::PreDecrement,
        operand: Box::new(HirExpression::Variable("p".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("wrapping_sub"),
        "Pointer pre-dec → wrapping_sub: {}",
        code
    );
}

#[test]
fn expr_context_deref_raw_pointer_unsafe() {
    // DECY-041/226: *ptr where ptr is raw pointer → unsafe { *ptr }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("ptr".to_string())));
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("unsafe") && code.contains("*ptr"),
        "Deref raw pointer → unsafe: {}",
        code
    );
}

#[test]
fn expr_context_deref_ptr_arithmetic_unsafe() {
    // DECY-226: *(ptr + n) → unsafe deref of pointer arithmetic
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("arr".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::Dereference(Box::new(HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::IntLiteral(1)),
    }));
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("unsafe"),
        "Deref ptr arithmetic → unsafe: {}",
        code
    );
}

#[test]
fn expr_context_deref_non_pointer() {
    // *val where val is not a pointer → no unsafe
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("val".to_string(), HirType::Int);
    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("val".to_string())));
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(!code.contains("unsafe"), "Non-pointer deref no unsafe: {}", code);
    assert!(code.contains("*val"), "Simple deref: {}", code);
}

#[test]
fn expr_context_strlen_call() {
    // strlen(s) → s.len() as i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "strlen".to_string(),
        arguments: vec![HirExpression::Variable("s".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains(".len()") && code.contains("as i32"),
        "strlen → .len() as i32: {}",
        code
    );
}

#[test]
fn expr_context_strcpy_str_source() {
    // strcpy(dest, src) with simple var → .to_string()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "strcpy".to_string(),
        arguments: vec![
            HirExpression::Variable("dest".to_string()),
            HirExpression::Variable("src".to_string()),
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("to_string()"),
        "strcpy with &str source: {}",
        code
    );
}

#[test]
fn expr_context_logical_not_bool_expr() {
    // !comparison → boolean negation
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("!"),
        "Logical NOT on boolean: {}",
        code
    );
    assert!(
        !code.contains("as i32"),
        "Bool NOT should not cast to i32 (in context, not target): {}",
        code
    );
}

#[test]
fn expr_context_logical_not_int() {
    // !int_var → (x == 0) as i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("flags".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("== 0"),
        "Logical NOT int → (x == 0): {}",
        code
    );
}

#[test]
fn expr_context_unary_negate() {
    // -x → prefix operator
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::Minus,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("-x"),
        "Unary negate: {}",
        code
    );
}

#[test]
fn expr_context_unary_bitwise_not() {
    // ~x → prefix operator
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::UnaryOp {
        op: decy_hir::UnaryOperator::BitwiseNot,
        operand: Box::new(HirExpression::Variable("mask".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("!mask"),
        "Bitwise NOT: {}",
        code
    );
}

// =============================================================================
// Batch 50: Stdlib FunctionCall transformations (calloc, realloc, fopen, fclose,
//           fgetc, fputc, fputs, fread, fwrite, fprintf, fork, exec, wait)
// =============================================================================

#[test]
fn expr_context_calloc_default() {
    // calloc(n, size) → vec![0i32; n]
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "calloc".to_string(),
        arguments: vec![
            HirExpression::IntLiteral(10),
            HirExpression::Sizeof { type_name: "int".to_string() },
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("vec![0i32") && code.contains("as usize"),
        "calloc → vec!: {}",
        code
    );
}

#[test]
fn expr_context_calloc_with_vec_target() {
    // calloc with Vec<T> target → correct element type
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "calloc".to_string(),
        arguments: vec![
            HirExpression::IntLiteral(5),
            HirExpression::Sizeof { type_name: "double".to_string() },
        ],
    };
    let code = cg.generate_expression_with_target_type(
        &expr,
        &mut ctx,
        Some(&HirType::Vec(Box::new(HirType::Double))),
    );
    assert!(
        code.contains("0.0f64") || code.contains("0f64"),
        "calloc Vec<f64> → correct default: {}",
        code
    );
}

#[test]
fn expr_context_calloc_with_ptr_target() {
    // calloc with *mut T target → Box::leak
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "calloc".to_string(),
        arguments: vec![
            HirExpression::IntLiteral(20),
            HirExpression::Sizeof { type_name: "int".to_string() },
        ],
    };
    let code = cg.generate_expression_with_target_type(
        &expr,
        &mut ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(
        code.contains("Box::leak"),
        "calloc *mut T → Box::leak: {}",
        code
    );
}

#[test]
fn expr_context_fopen_read() {
    // fopen("file", "r") → File::open().ok()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fopen".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("data.txt".to_string()),
            HirExpression::StringLiteral("r".to_string()),
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("File::open") && code.contains(".ok()"),
        "fopen read → File::open: {}",
        code
    );
}

#[test]
fn expr_context_fopen_write() {
    // fopen("file", "w") → File::create().ok()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fopen".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("out.txt".to_string()),
            HirExpression::StringLiteral("w".to_string()),
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("File::create") && code.contains(".ok()"),
        "fopen write → File::create: {}",
        code
    );
}

#[test]
fn expr_context_fclose() {
    // fclose(f) → drop(f)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fclose".to_string(),
        arguments: vec![HirExpression::Variable("fp".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("drop(fp)"),
        "fclose → drop: {}",
        code
    );
}

#[test]
fn expr_context_fgetc() {
    // fgetc(f) → read byte
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fgetc".to_string(),
        arguments: vec![HirExpression::Variable("f".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("Read") && code.contains(".read("),
        "fgetc → read byte: {}",
        code
    );
}

#[test]
fn expr_context_fputc() {
    // fputc(c, f) → write byte
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fputc".to_string(),
        arguments: vec![
            HirExpression::Variable("ch".to_string()),
            HirExpression::Variable("f".to_string()),
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("Write") && code.contains(".write("),
        "fputc → write byte: {}",
        code
    );
}

#[test]
fn expr_context_fputs() {
    // fputs(str, file) → write_all
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fputs".to_string(),
        arguments: vec![
            HirExpression::Variable("line".to_string()),
            HirExpression::Variable("fp".to_string()),
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("write_all") && code.contains("as_bytes()"),
        "fputs → write_all: {}",
        code
    );
}

#[test]
fn expr_context_fread() {
    // fread(buf, size, count, file) → file.read(&mut buf)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fread".to_string(),
        arguments: vec![
            HirExpression::Variable("buffer".to_string()),
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(256),
            HirExpression::Variable("fp".to_string()),
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("Read") && code.contains(".read("),
        "fread → read: {}",
        code
    );
}

#[test]
fn expr_context_fwrite() {
    // fwrite(data, size, count, file) → file.write(&data)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fwrite".to_string(),
        arguments: vec![
            HirExpression::Variable("data".to_string()),
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(100),
            HirExpression::Variable("out".to_string()),
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("Write") && code.contains(".write("),
        "fwrite → write: {}",
        code
    );
}

#[test]
fn expr_context_fprintf_simple() {
    // fprintf(f, "hello") → write!(f, "hello")
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("stderr".to_string()),
            HirExpression::StringLiteral("error\\n".to_string()),
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("write!"),
        "fprintf → write!: {}",
        code
    );
}

#[test]
fn expr_context_free_call() {
    // free(ptr) → drop(ptr)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "free".to_string(),
        arguments: vec![HirExpression::Variable("buf".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("drop(buf)"),
        "free → drop: {}",
        code
    );
}

#[test]
fn expr_context_fork() {
    // fork() → comment + 0
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "fork".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("fork") && code.contains("0"),
        "fork → comment: {}",
        code
    );
}

#[test]
fn expr_context_execl() {
    // execl("/bin/ls", ...) → Command::new(...)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "execl".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("/bin/ls".to_string()),
            HirExpression::StringLiteral("ls".to_string()),
            HirExpression::NullLiteral,
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("Command::new"),
        "execl → Command::new: {}",
        code
    );
}

#[test]
fn expr_context_waitpid() {
    // waitpid → child.wait()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "waitpid".to_string(),
        arguments: vec![HirExpression::IntLiteral(-1)],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("wait()"),
        "waitpid → wait: {}",
        code
    );
}

#[test]
fn expr_context_wexitstatus() {
    // WEXITSTATUS(status) → status.code().unwrap_or(-1)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "WEXITSTATUS".to_string(),
        arguments: vec![HirExpression::Variable("status".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains(".code()") && code.contains("unwrap_or(-1)"),
        "WEXITSTATUS → .code(): {}",
        code
    );
}

#[test]
fn expr_context_wifexited() {
    // WIFEXITED(status) → status.success()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "WIFEXITED".to_string(),
        arguments: vec![HirExpression::Variable("status".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains(".success()"),
        "WIFEXITED → .success(): {}",
        code
    );
}

#[test]
fn expr_context_realloc_with_ptr_target() {
    // realloc(ptr, size) with pointer target → cast return type
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "realloc".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::IntLiteral(256),
        ],
    };
    let code = cg.generate_expression_with_target_type(
        &expr,
        &mut ctx,
        Some(&HirType::Pointer(Box::new(HirType::Int))),
    );
    assert!(
        code.contains("realloc") && code.contains("as *mut"),
        "realloc with ptr target → cast: {}",
        code
    );
}

// =============================================================================
// Batch 51: Remaining stdlib FunctionCall transforms + default call handler
// =============================================================================

#[test]
fn expr_context_wifsignaled_with_arg() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "WIFSIGNALED".to_string(),
        arguments: vec![HirExpression::Variable("status".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("signal().is_some()"),
        "WIFSIGNALED → signal().is_some(): {}",
        code
    );
}

#[test]
fn expr_context_wifsignaled_no_arg() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "WIFSIGNALED".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("WIFSIGNALED requires"),
        "WIFSIGNALED no arg → comment: {}",
        code
    );
}

#[test]
fn expr_context_wtermsig_with_arg() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "WTERMSIG".to_string(),
        arguments: vec![HirExpression::Variable("status".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("signal().unwrap_or(0)"),
        "WTERMSIG → signal().unwrap_or(0): {}",
        code
    );
}

#[test]
fn expr_context_wtermsig_no_arg() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "WTERMSIG".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("WTERMSIG requires"),
        "WTERMSIG no arg → comment: {}",
        code
    );
}

#[test]
fn expr_context_atoi_with_arg() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "atoi".to_string(),
        arguments: vec![HirExpression::Variable("s".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("parse::<i32>().unwrap_or(0)"),
        "atoi → parse::<i32>(): {}",
        code
    );
}

#[test]
fn expr_context_atoi_wrong_args() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "atoi".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("atoi requires"),
        "atoi wrong args → comment: {}",
        code
    );
}

#[test]
fn expr_context_atof_with_arg() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "atof".to_string(),
        arguments: vec![HirExpression::Variable("s".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("parse::<f64>().unwrap_or(0.0)"),
        "atof → parse::<f64>(): {}",
        code
    );
}

#[test]
fn expr_context_atof_wrong_args() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "atof".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("atof requires"),
        "atof wrong args → comment: {}",
        code
    );
}

#[test]
fn expr_context_abs_with_arg() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "abs".to_string(),
        arguments: vec![HirExpression::Variable("x".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains(".abs()"),
        "abs → .abs(): {}",
        code
    );
}

#[test]
fn expr_context_abs_wrong_args() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "abs".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("abs requires"),
        "abs wrong args → comment: {}",
        code
    );
}

#[test]
fn expr_context_exit_with_arg() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "exit".to_string(),
        arguments: vec![HirExpression::IntLiteral(1)],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("std::process::exit(1)"),
        "exit(1) → std::process::exit(1): {}",
        code
    );
}

#[test]
fn expr_context_exit_no_arg() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "exit".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("std::process::exit(1)"),
        "exit no arg → exit(1): {}",
        code
    );
}

#[test]
fn expr_context_puts_with_arg() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "puts".to_string(),
        arguments: vec![HirExpression::Variable("msg".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("println!"),
        "puts → println!: {}",
        code
    );
}

#[test]
fn expr_context_puts_no_arg() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "puts".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("println!()"),
        "puts no arg → println!(): {}",
        code
    );
}

#[test]
fn expr_context_snprintf_fmt_only() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "snprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::IntLiteral(256),
            HirExpression::StringLiteral("hello".to_string()),
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("format!"),
        "snprintf fmt only → format!: {}",
        code
    );
}

#[test]
fn expr_context_snprintf_with_args() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "snprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::IntLiteral(256),
            HirExpression::StringLiteral("val=%d".to_string()),
            HirExpression::Variable("x".to_string()),
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("format!") && code.contains("x"),
        "snprintf with args → format! with args: {}",
        code
    );
}

#[test]
fn expr_context_snprintf_too_few_args() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "snprintf".to_string(),
        arguments: vec![HirExpression::Variable("buf".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("snprintf requires"),
        "snprintf too few args → comment: {}",
        code
    );
}

#[test]
fn expr_context_sprintf_fmt_only() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "sprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::StringLiteral("hello".to_string()),
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("format!"),
        "sprintf fmt only → format!: {}",
        code
    );
}

#[test]
fn expr_context_sprintf_with_args() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "sprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::StringLiteral("x=%d".to_string()),
            HirExpression::Variable("val".to_string()),
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("format!") && code.contains("val"),
        "sprintf with args → format! with args: {}",
        code
    );
}

#[test]
fn expr_context_sprintf_too_few_args() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "sprintf".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("sprintf requires"),
        "sprintf too few args → comment: {}",
        code
    );
}

#[test]
fn expr_context_qsort_with_args() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
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
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("sort_by") && code.contains("compare"),
        "qsort → sort_by: {}",
        code
    );
}

#[test]
fn expr_context_qsort_wrong_args() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "qsort".to_string(),
        arguments: vec![HirExpression::Variable("arr".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("qsort requires"),
        "qsort wrong args → comment: {}",
        code
    );
}

#[test]
fn expr_context_default_address_of_to_mut_ref() {
    // AddressOf argument → &mut when param expects &mut (default)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "custom_func".to_string(),
        arguments: vec![HirExpression::AddressOf(Box::new(
            HirExpression::Variable("x".to_string()),
        ))],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("&mut x"),
        "AddressOf → &mut by default: {}",
        code
    );
}

#[test]
fn expr_context_default_address_of_unary_op() {
    // UnaryOp::AddressOf argument → &mut
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "custom_func".to_string(),
        arguments: vec![HirExpression::UnaryOp {
            op: decy_hir::UnaryOperator::AddressOf,
            operand: Box::new(HirExpression::Variable("y".to_string())),
        }],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("&mut y"),
        "UnaryOp AddressOf → &mut: {}",
        code
    );
}

#[test]
fn expr_context_default_raw_ptr_param_array_arg() {
    // Raw pointer param + array arg → .as_mut_ptr()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "data".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    ctx.add_function(
        "process".to_string(),
        vec![HirType::Pointer(Box::new(HirType::Int))],
    );
    let expr = HirExpression::FunctionCall {
        function: "process".to_string(),
        arguments: vec![HirExpression::Variable("data".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("data.as_mut_ptr()"),
        "Raw ptr param + array → .as_mut_ptr(): {}",
        code
    );
}

#[test]
fn expr_context_default_raw_ptr_param_string_arg() {
    // Raw pointer param + string literal → .as_ptr() as *mut u8
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_function(
        "process".to_string(),
        vec![HirType::Pointer(Box::new(HirType::Char))],
    );
    let expr = HirExpression::FunctionCall {
        function: "process".to_string(),
        arguments: vec![HirExpression::StringLiteral("hello".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains(".as_ptr()") && code.contains("*mut u8"),
        "Raw ptr param + string → .as_ptr() as *mut u8: {}",
        code
    );
}

#[test]
fn expr_context_default_ref_param_pointer_var() {
    // Reference param + pointer variable → unsafe { &mut *ptr }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    ctx.add_function(
        "process".to_string(),
        vec![HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        }],
    );
    let expr = HirExpression::FunctionCall {
        function: "process".to_string(),
        arguments: vec![HirExpression::Variable("ptr".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("unsafe") && code.contains("&mut *ptr"),
        "Ref param + pointer → unsafe {{ &mut *ptr }}: {}",
        code
    );
}

#[test]
fn expr_context_default_slice_param_fixed_array() {
    // Unsized array param + fixed-size array arg → &mut prefix
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "buf".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(256),
        },
    );
    ctx.add_function(
        "fill".to_string(),
        vec![HirType::Array {
            element_type: Box::new(HirType::Char),
            size: None,
        }],
    );
    let expr = HirExpression::FunctionCall {
        function: "fill".to_string(),
        arguments: vec![HirExpression::Variable("buf".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("&mut buf"),
        "Slice param + fixed array → &mut buf: {}",
        code
    );
}

#[test]
fn expr_context_default_int_param_char_literal() {
    // Int param + CharLiteral → cast as i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_function("putchar".to_string(), vec![HirType::Int]);
    let expr = HirExpression::FunctionCall {
        function: "putchar".to_string(),
        arguments: vec![HirExpression::CharLiteral(b' ' as i8)],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("32i32") || code.contains("32"),
        "Int param + char → i32 cast: {}",
        code
    );
}

#[test]
fn expr_context_default_func_rename_write() {
    // write → c_write to avoid Rust macro conflict
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "write".to_string(),
        arguments: vec![
            HirExpression::Variable("fd".to_string()),
            HirExpression::Variable("buf".to_string()),
            HirExpression::Variable("len".to_string()),
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("c_write("),
        "write → c_write: {}",
        code
    );
}

#[test]
fn expr_context_default_func_rename_read() {
    // read → c_read
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "read".to_string(),
        arguments: vec![
            HirExpression::Variable("fd".to_string()),
            HirExpression::Variable("buf".to_string()),
            HirExpression::Variable("len".to_string()),
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("c_read("),
        "read → c_read: {}",
        code
    );
}

#[test]
fn expr_context_default_func_rename_type() {
    // type → c_type
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "type".to_string(),
        arguments: vec![HirExpression::Variable("x".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("c_type("),
        "type → c_type: {}",
        code
    );
}

#[test]
fn expr_context_default_func_rename_match() {
    // match → c_match
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "match".to_string(),
        arguments: vec![HirExpression::Variable("pat".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("c_match("),
        "match → c_match: {}",
        code
    );
}

#[test]
fn expr_context_default_func_rename_self() {
    // self → c_self
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "self".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("c_self("),
        "self → c_self: {}",
        code
    );
}

#[test]
fn expr_context_default_func_rename_in() {
    // in → c_in
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "in".to_string(),
        arguments: vec![HirExpression::Variable("x".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("c_in("),
        "in → c_in: {}",
        code
    );
}

#[test]
fn expr_context_string_func_ptr_field_access() {
    // strcmp with PointerFieldAccess arg → CStr conversion
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::FunctionCall {
        function: "strcmp".to_string(),
        arguments: vec![
            HirExpression::PointerFieldAccess {
                pointer: Box::new(HirExpression::Variable("entry".to_string())),
                field: "key".to_string(),
            },
            HirExpression::StringLiteral("target".to_string()),
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("CStr::from_ptr") || code.contains("unsafe"),
        "strcmp with ptr field → CStr conversion: {}",
        code
    );
}

// =============================================================================
// Batch 52: PointerFieldAccess/ArrayIndex/Sizeof/Calloc/Malloc/Realloc/
//           StringMethodCall/Cast/CompoundLiteral expression branches
// =============================================================================

#[test]
fn expr_context_ptr_field_chain() {
    // ptr->field1->field2 — chained PointerFieldAccess should chain with .
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::PointerFieldAccess {
            pointer: Box::new(HirExpression::Variable("node".to_string())),
            field: "next".to_string(),
        }),
        field: "data".to_string(),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    // Outer access chains on inner — should produce something like (*node).next.data
    assert!(
        code.contains(".data"),
        "Chained ptr field access → .field: {}",
        code
    );
}

#[test]
fn expr_context_ptr_field_raw_pointer_var() {
    // ptr->field where ptr is raw pointer → unsafe { (*ptr).field }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "node".to_string(),
        HirType::Pointer(Box::new(HirType::Struct("Node".to_string()))),
    );
    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("node".to_string())),
        field: "value".to_string(),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("unsafe") && code.contains("(*node).value"),
        "Raw ptr field → unsafe deref: {}",
        code
    );
}

#[test]
fn expr_context_ptr_field_non_pointer_var() {
    // ptr->field where ptr is NOT raw pointer → (*ptr).field without unsafe
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "node".to_string(),
        HirType::Struct("Node".to_string()),
    );
    let expr = HirExpression::PointerFieldAccess {
        pointer: Box::new(HirExpression::Variable("node".to_string())),
        field: "value".to_string(),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("(*node).value") && !code.contains("unsafe"),
        "Non-ptr field → no unsafe: {}",
        code
    );
}

#[test]
fn expr_context_array_index_global() {
    // global_array[i] → unsafe { global_array[i] }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "table".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(256),
        },
    );
    ctx.add_global("table".to_string());
    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("table".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("unsafe"),
        "Global array index → unsafe: {}",
        code
    );
}

#[test]
fn expr_context_array_index_raw_pointer() {
    // ptr[i] where ptr is raw pointer → unsafe { *ptr.add(i as usize) }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "data".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("data".to_string())),
        index: Box::new(HirExpression::IntLiteral(5)),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("unsafe") && code.contains(".add("),
        "Raw ptr index → unsafe ptr.add: {}",
        code
    );
}

#[test]
fn expr_context_array_index_regular() {
    // arr[i] regular → arr[(i) as usize]
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "arr".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    let expr = HirExpression::ArrayIndex {
        array: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("arr[(i) as usize]") && !code.contains("unsafe"),
        "Regular array index → no unsafe: {}",
        code
    );
}

#[test]
fn expr_context_sizeof_type() {
    // sizeof(int) → std::mem::size_of::<i32>() as i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::Sizeof {
        type_name: "int".to_string(),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("size_of::<i32>()"),
        "sizeof(int) → size_of::<i32>(): {}",
        code
    );
}

#[test]
fn expr_context_sizeof_variable() {
    // sizeof(x) where x is known variable → size_of_val(&x)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::Sizeof {
        type_name: "x".to_string(),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("size_of_val(&x)"),
        "sizeof(variable) → size_of_val: {}",
        code
    );
}

#[test]
fn expr_context_sizeof_struct_field() {
    // sizeof with struct field pattern "MyStruct field" → size_of field type
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_struct(&decy_hir::HirStruct::new(
        "MyStruct".to_string(),
        vec![decy_hir::HirStructField::new(
            "field".to_string(),
            HirType::Double,
        )],
    ));
    let expr = HirExpression::Sizeof {
        type_name: "struct MyStruct field".to_string(),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("size_of::<f64>()"),
        "sizeof struct field → size_of field type: {}",
        code
    );
}

#[test]
fn expr_context_sizeof_member_access_var() {
    // sizeof "record name" where record is a known variable → size_of_val
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_variable(
        "record".to_string(),
        HirType::Struct("Record".to_string()),
    );
    let expr = HirExpression::Sizeof {
        type_name: "record name".to_string(),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("size_of_val"),
        "sizeof member access (var) → size_of_val: {}",
        code
    );
}

#[test]
fn expr_context_calloc_int() {
    // calloc(n, sizeof(int)) → vec![0i32; n]
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::Variable("n".to_string())),
        element_type: Box::new(HirType::Int),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("vec![0i32; n]"),
        "calloc int → vec![0i32; n]: {}",
        code
    );
}

#[test]
fn expr_context_calloc_unsigned_int() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(10)),
        element_type: Box::new(HirType::UnsignedInt),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("0u32"),
        "calloc unsigned → 0u32: {}",
        code
    );
}

#[test]
fn expr_context_calloc_float() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(5)),
        element_type: Box::new(HirType::Float),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("0.0f32"),
        "calloc float → 0.0f32: {}",
        code
    );
}

#[test]
fn expr_context_calloc_double() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(5)),
        element_type: Box::new(HirType::Double),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("0.0f64"),
        "calloc double → 0.0f64: {}",
        code
    );
}

#[test]
fn expr_context_calloc_char() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(256)),
        element_type: Box::new(HirType::Char),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("0u8"),
        "calloc char → 0u8: {}",
        code
    );
}

#[test]
fn expr_context_calloc_signed_char() {
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(256)),
        element_type: Box::new(HirType::SignedChar),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("0i8"),
        "calloc signed char → 0i8: {}",
        code
    );
}

#[test]
fn expr_context_malloc_array_pattern() {
    // malloc(n * sizeof(T)) → Vec::with_capacity(n)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::Malloc {
        size: Box::new(HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::Sizeof {
                type_name: "int".to_string(),
            }),
        }),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("Vec::with_capacity(n)"),
        "malloc(n*sizeof) → Vec::with_capacity: {}",
        code
    );
}

#[test]
fn expr_context_malloc_single() {
    // malloc(sizeof(int)) → Box::new(0i32)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::Malloc {
        size: Box::new(HirExpression::Sizeof {
            type_name: "int".to_string(),
        }),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("Box::new(0i32)"),
        "malloc(sizeof) → Box::new: {}",
        code
    );
}

#[test]
fn expr_context_realloc_null() {
    // realloc(NULL, n * sizeof(T)) → vec![0i32; n]
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::NullLiteral),
        new_size: Box::new(HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::Sizeof {
                type_name: "int".to_string(),
            }),
        }),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("vec![0i32; n]"),
        "realloc(NULL, n*sizeof) → vec!: {}",
        code
    );
}

#[test]
fn expr_context_realloc_null_simple() {
    // realloc(NULL, size) without multiply → Vec::new()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::NullLiteral),
        new_size: Box::new(HirExpression::IntLiteral(100)),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("Vec::new()"),
        "realloc(NULL, size) → Vec::new(): {}",
        code
    );
}

#[test]
fn expr_context_realloc_non_null() {
    // realloc(ptr, size) where ptr is not NULL → passthrough
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::Variable("buf".to_string())),
        new_size: Box::new(HirExpression::IntLiteral(200)),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("buf"),
        "realloc(ptr, size) → passthrough ptr: {}",
        code
    );
}

#[test]
fn expr_context_string_method_len() {
    // receiver.len() → receiver.len() as i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "len".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("s.len() as i32"),
        "String .len() → as i32: {}",
        code
    );
}

#[test]
fn expr_context_string_method_no_args() {
    // receiver.is_empty() → receiver.is_empty()
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "is_empty".to_string(),
        arguments: vec![],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("s.is_empty()"),
        "String method no args: {}",
        code
    );
}

#[test]
fn expr_context_string_method_clone_into() {
    // receiver.clone_into(dest) → receiver.clone_into(&mut dest)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("src".to_string())),
        method: "clone_into".to_string(),
        arguments: vec![HirExpression::Variable("dest".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("&mut dest"),
        "clone_into → &mut dest: {}",
        code
    );
}

#[test]
fn expr_context_string_method_with_args() {
    // receiver.push_str(arg) → receiver.push_str(arg)
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "push_str".to_string(),
        arguments: vec![HirExpression::StringLiteral("hello".to_string())],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("s.push_str("),
        "String method with args: {}",
        code
    );
}

#[test]
fn expr_context_cast_simple() {
    // (float)x → x as f32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::Cast {
        target_type: HirType::Float,
        expr: Box::new(HirExpression::Variable("x".to_string())),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("x as f32"),
        "Cast float → as f32: {}",
        code
    );
}

#[test]
fn expr_context_cast_binop_parens() {
    // (int)(a + b) → (a + b) as i32
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::Cast {
        target_type: HirType::Int,
        expr: Box::new(HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("(a + b) as i32"),
        "Cast binop → parens: {}",
        code
    );
}

#[test]
fn expr_context_cast_address_of_to_int() {
    // (long)&x → &x as *const _ as isize as i64
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::Cast {
        target_type: HirType::Int,
        expr: Box::new(HirExpression::AddressOf(Box::new(
            HirExpression::Variable("x".to_string()),
        ))),
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("*const _") && code.contains("isize"),
        "Address-of to int → ptr cast chain: {}",
        code
    );
}

#[test]
fn expr_context_compound_literal_empty_struct() {
    // (struct Point){} → Point {}
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Point".to_string()),
        initializers: vec![],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("Point {}"),
        "Empty compound struct → Point {{}}: {}",
        code
    );
}

#[test]
fn expr_context_compound_literal_struct_with_fields() {
    // (struct Point){10, 20} with known fields → Point { x: 10, y: 20 }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_struct(&decy_hir::HirStruct::new(
        "Point".to_string(),
        vec![
            decy_hir::HirStructField::new("x".to_string(), HirType::Int),
            decy_hir::HirStructField::new("y".to_string(), HirType::Int),
        ],
    ));
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Point".to_string()),
        initializers: vec![
            HirExpression::IntLiteral(10),
            HirExpression::IntLiteral(20),
        ],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("x: 10") && code.contains("y: 20"),
        "Compound struct with fields: {}",
        code
    );
}

#[test]
fn expr_context_compound_literal_struct_partial_init() {
    // (struct Point){10} with 2 fields → Point { x: 10, ..Default::default() }
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    ctx.add_struct(&decy_hir::HirStruct::new(
        "Point".to_string(),
        vec![
            decy_hir::HirStructField::new("x".to_string(), HirType::Int),
            decy_hir::HirStructField::new("y".to_string(), HirType::Int),
        ],
    ));
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Point".to_string()),
        initializers: vec![HirExpression::IntLiteral(10)],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    assert!(
        code.contains("Default::default()"),
        "Partial struct init → ..Default::default(): {}",
        code
    );
}

#[test]
fn expr_context_compound_literal_array_empty_sized() {
    // (int[4]){} → [0i32; 4]
    let cg = CodeGenerator::new();
    let mut ctx = TypeContext::new();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(4),
        },
        initializers: vec![],
    };
    let code = cg.generate_expression_with_context(&expr, &mut ctx);
    let _ = code; // test body completed by DECY-202 fix
}
