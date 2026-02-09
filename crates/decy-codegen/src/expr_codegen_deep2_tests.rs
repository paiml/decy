//! Deep coverage tests (batch 2) for `generate_expression_with_target_type`
//! and `convert_format_specifiers`.
//!
//! Targets remaining uncovered branches: stdlib function calls (printf, fprintf,
//! strcpy, malloc, calloc, realloc, free, fopen, fclose, fgetc, fputc, fread,
//! fwrite, fputs, fork, exec*, wait*, WEXITSTATUS, WIFEXITED), StringMethodCall,
//! IsNotNull, CompoundLiteral edge cases, Cast chains, Ternary propagation,
//! PostIncrement/PreIncrement/PostDecrement/PreDecrement on pointers and
//! dereferenced pointers, bitwise ops with bool/unsigned, comparison chaining,
//! signed/unsigned mismatch, assignment expression, string literal to pointer,
//! variable renamed locals, variable special streams, Sizeof edge cases,
//! and remaining convert_format_specifiers branches.

use super::*;
use decy_hir::{BinaryOperator, HirExpression, HirType, UnaryOperator};

// ============================================================================
// Helpers
// ============================================================================

fn gen() -> CodeGenerator {
    CodeGenerator::new()
}

fn ctx() -> TypeContext {
    TypeContext::new()
}

fn expr_tt(expr: &HirExpression, ctx: &TypeContext, tt: Option<&HirType>) -> String {
    gen().generate_expression_with_target_type(expr, ctx, tt)
}

fn expr_no_tt(expr: &HirExpression, ctx: &TypeContext) -> String {
    gen().generate_expression_with_target_type(expr, ctx, None)
}

// ============================================================================
// FunctionCall: strlen with valid single arg
// ============================================================================

#[test]
fn strlen_single_arg_generates_len_cast() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "strlen".to_string(),
        arguments: vec![HirExpression::Variable("s".to_string())],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains(".len() as i32"),
        "strlen(s) should become s.len() as i32, got: {}",
        result
    );
}

#[test]
fn strlen_no_args_fallback() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "strlen".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("strlen()"),
        "strlen with no args should fall through, got: {}",
        result
    );
}

// ============================================================================
// FunctionCall: strcpy branches
// ============================================================================

#[test]
fn strcpy_simple_source_uses_to_string() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "strcpy".to_string(),
        arguments: vec![
            HirExpression::Variable("dest".to_string()),
            HirExpression::Variable("src".to_string()),
        ],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains(".to_string()"),
        "strcpy with simple var should use .to_string(), got: {}",
        result
    );
}

#[test]
fn strcpy_raw_pointer_source_uses_cstr() {
    let c = ctx();
    // Source that looks like a raw pointer (contains "(*")
    let expr = HirExpression::FunctionCall {
        function: "strcpy".to_string(),
        arguments: vec![
            HirExpression::Variable("dest".to_string()),
            HirExpression::Dereference(Box::new(HirExpression::Variable("ptr".to_string()))),
        ],
    };
    let result = expr_no_tt(&expr, &c);
    // Dereference generates (*ptr) which contains "(*" triggering raw pointer path
    assert!(
        result.contains("CStr") || result.contains("to_string"),
        "strcpy with raw pointer src should use CStr or to_string, got: {}",
        result
    );
}

#[test]
fn strcpy_wrong_arg_count_fallback() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "strcpy".to_string(),
        arguments: vec![HirExpression::Variable("x".to_string())],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("strcpy("),
        "strcpy with wrong args should fallback, got: {}",
        result
    );
}

// ============================================================================
// FunctionCall: malloc branches
// ============================================================================

#[test]
fn malloc_with_vec_int_target_non_multiply() {
    let c = ctx();
    let target = HirType::Vec(Box::new(HirType::Int));
    let expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(100)],
    };
    let result = expr_tt(&expr, &c, Some(&target));
    assert!(
        result.contains("Vec::<i32>::with_capacity"),
        "malloc(n) with Vec<i32> target should use with_capacity, got: {}",
        result
    );
}

#[test]
fn malloc_with_vec_target_multiply_pattern() {
    let c = ctx();
    let target = HirType::Vec(Box::new(HirType::Double));
    let expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::IntLiteral(8)),
        }],
    };
    let result = expr_tt(&expr, &c, Some(&target));
    assert!(
        result.contains("vec!["),
        "malloc(n*sizeof) with Vec target should use vec!, got: {}",
        result
    );
}

#[test]
fn malloc_with_pointer_struct_target() {
    let c = ctx();
    let target = HirType::Pointer(Box::new(HirType::Struct("Node".to_string())));
    let expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(64)],
    };
    let result = expr_tt(&expr, &c, Some(&target));
    assert!(
        result.contains("Box::into_raw(Box::<Node>::default())"),
        "malloc for struct ptr should use Box::into_raw, got: {}",
        result
    );
}

#[test]
fn malloc_with_pointer_int_multiply_target() {
    let c = ctx();
    let target = HirType::Pointer(Box::new(HirType::Int));
    let expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::IntLiteral(4)),
        }],
    };
    let result = expr_tt(&expr, &c, Some(&target));
    assert!(
        result.contains("Box::leak") && result.contains("as_mut_ptr"),
        "malloc(n*sizeof(int)) with *mut i32 target should use Box::leak, got: {}",
        result
    );
}

#[test]
fn malloc_with_pointer_int_single_target() {
    let c = ctx();
    let target = HirType::Pointer(Box::new(HirType::Int));
    let expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(4)],
    };
    let result = expr_tt(&expr, &c, Some(&target));
    assert!(
        result.contains("Box::leak") && result.contains("as_mut_ptr"),
        "malloc single allocation with ptr target should use Box::leak, got: {}",
        result
    );
}

#[test]
fn malloc_no_target_multiply_generates_vec_i32() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::IntLiteral(4)),
        }],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("vec![0i32;"),
        "malloc(n*sz) without target should generate vec![0i32;], got: {}",
        result
    );
}

#[test]
fn malloc_no_target_simple_generates_with_capacity() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![HirExpression::IntLiteral(256)],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("Vec::<u8>::with_capacity"),
        "malloc(n) without target should use Vec::<u8>::with_capacity, got: {}",
        result
    );
}

#[test]
fn malloc_no_args_generates_vec_new() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "malloc".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "Vec::new()");
}

// ============================================================================
// FunctionCall: calloc branches
// ============================================================================

#[test]
fn calloc_with_vec_target() {
    let c = ctx();
    let target = HirType::Vec(Box::new(HirType::Float));
    let expr = HirExpression::FunctionCall {
        function: "calloc".to_string(),
        arguments: vec![
            HirExpression::IntLiteral(10),
            HirExpression::IntLiteral(4),
        ],
    };
    let result = expr_tt(&expr, &c, Some(&target));
    assert!(
        result.contains("vec![0.0f32;"),
        "calloc with Vec<Float> target should use float default, got: {}",
        result
    );
}

#[test]
fn calloc_with_pointer_target() {
    let c = ctx();
    let target = HirType::Pointer(Box::new(HirType::Int));
    let expr = HirExpression::FunctionCall {
        function: "calloc".to_string(),
        arguments: vec![
            HirExpression::IntLiteral(10),
            HirExpression::IntLiteral(4),
        ],
    };
    let result = expr_tt(&expr, &c, Some(&target));
    assert!(
        result.contains("Box::leak") && result.contains("as_mut_ptr"),
        "calloc with ptr target should use Box::leak, got: {}",
        result
    );
}

#[test]
fn calloc_no_target_default() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "calloc".to_string(),
        arguments: vec![
            HirExpression::Variable("n".to_string()),
            HirExpression::IntLiteral(4),
        ],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("vec![0i32;"),
        "calloc default should produce vec![0i32;], got: {}",
        result
    );
}

#[test]
fn calloc_wrong_arg_count_fallback() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "calloc".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "Vec::new()");
}

// ============================================================================
// FunctionCall: realloc branches
// ============================================================================

#[test]
fn realloc_with_pointer_target_casts_return() {
    let c = ctx();
    let target = HirType::Pointer(Box::new(HirType::Int));
    let expr = HirExpression::FunctionCall {
        function: "realloc".to_string(),
        arguments: vec![
            HirExpression::Variable("ptr".to_string()),
            HirExpression::IntLiteral(200),
        ],
    };
    let result = expr_tt(&expr, &c, Some(&target));
    assert!(
        result.contains("realloc(") && result.contains("as *mut i32"),
        "realloc with ptr target should cast return, got: {}",
        result
    );
}

#[test]
fn realloc_without_target_no_cast() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "realloc".to_string(),
        arguments: vec![
            HirExpression::Variable("ptr".to_string()),
            HirExpression::IntLiteral(200),
        ],
    };
    let result = expr_no_tt(&expr, &c);
    // realloc always casts arg to *mut (), but without target it does not cast the return value
    assert!(
        result.contains("realloc(ptr as *mut (), 200)"),
        "realloc without target should not cast return, got: {}",
        result
    );
    // Verify no return type cast (no trailing "as *mut i32" etc.)
    assert!(
        !result.contains(") as *mut i32"),
        "realloc without target should not have return cast, got: {}",
        result
    );
}

#[test]
fn realloc_wrong_arg_count_returns_null_mut() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "realloc".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "std::ptr::null_mut()");
}

// ============================================================================
// FunctionCall: free
// ============================================================================

#[test]
fn free_single_arg_generates_drop() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "free".to_string(),
        arguments: vec![HirExpression::Variable("ptr".to_string())],
    };
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "drop(ptr)");
}

#[test]
fn free_no_args_fallback_comment() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "free".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("free()"),
        "free with no args should generate comment, got: {}",
        result
    );
}

// ============================================================================
// FunctionCall: fopen
// ============================================================================

#[test]
fn fopen_read_mode_generates_open() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fopen".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("test.txt".to_string()),
            HirExpression::StringLiteral("r".to_string()),
        ],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("File::open"),
        "fopen(\"r\") should use File::open, got: {}",
        result
    );
}

#[test]
fn fopen_write_mode_generates_create() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fopen".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("out.txt".to_string()),
            HirExpression::StringLiteral("w".to_string()),
        ],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("File::create"),
        "fopen(\"w\") should use File::create, got: {}",
        result
    );
}

#[test]
fn fopen_append_mode_generates_create() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fopen".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("log.txt".to_string()),
            HirExpression::StringLiteral("a".to_string()),
        ],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("File::create"),
        "fopen(\"a\") should use File::create, got: {}",
        result
    );
}

#[test]
fn fopen_wrong_arg_count_fallback() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fopen".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("fopen requires 2 args"),
        "fopen with wrong args should fallback, got: {}",
        result
    );
}

// ============================================================================
// FunctionCall: fclose
// ============================================================================

#[test]
fn fclose_single_arg_generates_drop() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fclose".to_string(),
        arguments: vec![HirExpression::Variable("fp".to_string())],
    };
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "drop(fp)");
}

#[test]
fn fclose_no_args_fallback() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fclose".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(result.contains("fclose()"));
}

// ============================================================================
// FunctionCall: fgetc / getc
// ============================================================================

#[test]
fn fgetc_generates_read_pattern() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fgetc".to_string(),
        arguments: vec![HirExpression::Variable("fp".to_string())],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("Read") && result.contains("read("),
        "fgetc should generate read pattern, got: {}",
        result
    );
}

#[test]
fn getc_generates_read_pattern() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "getc".to_string(),
        arguments: vec![HirExpression::Variable("fp".to_string())],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("Read"),
        "getc should generate read pattern, got: {}",
        result
    );
}

#[test]
fn fgetc_no_args_fallback() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fgetc".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(result.contains("fgetc requires 1 arg"));
}

// ============================================================================
// FunctionCall: fputc / putc
// ============================================================================

#[test]
fn fputc_generates_write_pattern() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fputc".to_string(),
        arguments: vec![
            HirExpression::Variable("ch".to_string()),
            HirExpression::Variable("fp".to_string()),
        ],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("Write") && result.contains("write("),
        "fputc should generate write pattern, got: {}",
        result
    );
}

#[test]
fn putc_generates_write_pattern() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "putc".to_string(),
        arguments: vec![
            HirExpression::Variable("ch".to_string()),
            HirExpression::Variable("fp".to_string()),
        ],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("Write"),
        "putc should generate write pattern, got: {}",
        result
    );
}

#[test]
fn fputc_wrong_args_fallback() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fputc".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(result.contains("fputc requires 2 args"));
}

// ============================================================================
// FunctionCall: fprintf
// ============================================================================

#[test]
fn fprintf_with_format_only() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("stderr".to_string()),
            HirExpression::StringLiteral("error\\n".to_string()),
        ],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("write!"),
        "fprintf should use write!, got: {}",
        result
    );
}

#[test]
fn fprintf_with_format_and_args() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fprintf".to_string(),
        arguments: vec![
            HirExpression::Variable("fp".to_string()),
            HirExpression::StringLiteral("val=%d\\n".to_string()),
            HirExpression::Variable("x".to_string()),
        ],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("write!") && result.contains("fp"),
        "fprintf with args should use write! with file, got: {}",
        result
    );
}

#[test]
fn fprintf_wrong_arg_count_fallback() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fprintf".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(result.contains("fprintf requires 2+ args"));
}

// ============================================================================
// FunctionCall: printf
// ============================================================================

#[test]
fn printf_format_only() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![HirExpression::StringLiteral("hello\\n".to_string())],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("print!("),
        "printf single arg should use print!, got: {}",
        result
    );
}

#[test]
fn printf_with_int_arg() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("%d\\n".to_string()),
            HirExpression::Variable("x".to_string()),
        ],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("print!(") && result.contains("x"),
        "printf with %d should include arg, got: {}",
        result
    );
}

#[test]
fn printf_no_args_generates_empty_print() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "printf".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "print!(\"\")");
}

// ============================================================================
// FunctionCall: fread / fwrite / fputs
// ============================================================================

#[test]
fn fread_generates_read_pattern() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fread".to_string(),
        arguments: vec![
            HirExpression::Variable("buf".to_string()),
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(256),
            HirExpression::Variable("fp".to_string()),
        ],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("read(") && result.contains("fp"),
        "fread should use read with file, got: {}",
        result
    );
}

#[test]
fn fread_wrong_args_fallback() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fread".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(result.contains("fread requires 4 args"));
}

#[test]
fn fwrite_generates_write_pattern() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fwrite".to_string(),
        arguments: vec![
            HirExpression::Variable("data".to_string()),
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(100),
            HirExpression::Variable("fp".to_string()),
        ],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("Write") && result.contains("write("),
        "fwrite should use write, got: {}",
        result
    );
}

#[test]
fn fwrite_wrong_args_fallback() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fwrite".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(result.contains("fwrite requires 4 args"));
}

#[test]
fn fputs_generates_write_all() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fputs".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("hello".to_string()),
            HirExpression::Variable("fp".to_string()),
        ],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("write_all"),
        "fputs should use write_all, got: {}",
        result
    );
}

#[test]
fn fputs_wrong_args_fallback() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fputs".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(result.contains("fputs requires 2 args"));
}

// ============================================================================
// FunctionCall: fork, exec, wait, WEXITSTATUS, etc.
// ============================================================================

#[test]
fn fork_generates_comment() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "fork".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("fork") && result.contains("0"),
        "fork should generate comment with 0, got: {}",
        result
    );
}

#[test]
fn execl_with_args_generates_command() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "execl".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("/bin/ls".to_string()),
            HirExpression::StringLiteral("ls".to_string()),
            HirExpression::StringLiteral("-la".to_string()),
            HirExpression::NullLiteral,
        ],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("Command::new"),
        "execl should use Command::new, got: {}",
        result
    );
}

#[test]
fn execl_single_arg_command_no_args() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "execlp".to_string(),
        arguments: vec![
            HirExpression::StringLiteral("/bin/ls".to_string()),
            HirExpression::StringLiteral("ls".to_string()),
            HirExpression::NullLiteral,
        ],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("Command::new"),
        "execlp should use Command::new, got: {}",
        result
    );
}

#[test]
fn exec_no_args_fallback() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "execv".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(result.contains("exec requires args"));
}

#[test]
fn waitpid_generates_child_wait() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "waitpid".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("child.wait()"),
        "waitpid should generate child.wait(), got: {}",
        result
    );
}

#[test]
fn wait_generates_child_wait() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "wait".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(result.contains("child.wait()"));
}

#[test]
fn wexitstatus_generates_code() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "WEXITSTATUS".to_string(),
        arguments: vec![HirExpression::Variable("status".to_string())],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("code().unwrap_or(-1)"),
        "WEXITSTATUS should generate .code().unwrap_or(-1), got: {}",
        result
    );
}

#[test]
fn wexitstatus_no_args_fallback() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "WEXITSTATUS".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(result.contains("WEXITSTATUS requires"));
}

#[test]
fn wifexited_generates_success() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "WIFEXITED".to_string(),
        arguments: vec![HirExpression::Variable("status".to_string())],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(result.contains("success()"));
}

#[test]
fn wifexited_no_args_fallback() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "WIFEXITED".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(result.contains("WIFEXITED requires"));
}

#[test]
fn wifsignaled_generates_signal_check() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "WIFSIGNALED".to_string(),
        arguments: vec![HirExpression::Variable("status".to_string())],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(result.contains("signal().is_some()"));
}

#[test]
fn wifsignaled_no_args_fallback() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "WIFSIGNALED".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(result.contains("WIFSIGNALED requires"));
}

#[test]
fn wtermsig_generates_signal_value() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "WTERMSIG".to_string(),
        arguments: vec![HirExpression::Variable("status".to_string())],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(result.contains("signal().unwrap_or(0)"));
}

#[test]
fn wtermsig_no_args_fallback() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "WTERMSIG".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(result.contains("WTERMSIG requires"));
}

// ============================================================================
// Variable: special streams and constants (DECY-239/241)
// ============================================================================

#[test]
fn variable_stderr_maps_to_io_stderr() {
    let c = ctx();
    let expr = HirExpression::Variable("stderr".to_string());
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "std::io::stderr()");
}

#[test]
fn variable_stdin_maps_to_io_stdin() {
    let c = ctx();
    let expr = HirExpression::Variable("stdin".to_string());
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "std::io::stdin()");
}

#[test]
fn variable_stdout_maps_to_io_stdout() {
    let c = ctx();
    let expr = HirExpression::Variable("stdout".to_string());
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "std::io::stdout()");
}

#[test]
fn variable_errno_maps_to_unsafe_errno() {
    let c = ctx();
    let expr = HirExpression::Variable("errno".to_string());
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "unsafe { ERRNO }");
}

#[test]
fn variable_erange_maps_to_constant() {
    let c = ctx();
    let expr = HirExpression::Variable("ERANGE".to_string());
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "34i32");
}

#[test]
fn variable_einval_maps_to_constant() {
    let c = ctx();
    let expr = HirExpression::Variable("EINVAL".to_string());
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "22i32");
}

#[test]
fn variable_enoent_maps_to_constant() {
    let c = ctx();
    let expr = HirExpression::Variable("ENOENT".to_string());
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "2i32");
}

#[test]
fn variable_eacces_maps_to_constant() {
    let c = ctx();
    let expr = HirExpression::Variable("EACCES".to_string());
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "13i32");
}

// ============================================================================
// Variable: Vec target returns directly (DECY-142)
// ============================================================================

#[test]
fn variable_with_vec_target_returns_directly() {
    let mut c = ctx();
    c.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let target = HirType::Vec(Box::new(HirType::Int));
    let expr = HirExpression::Variable("arr".to_string());
    let result = expr_tt(&expr, &c, Some(&target));
    assert_eq!(result, "arr", "Vec variable with Vec target should return directly");
}

// ============================================================================
// Variable: Box to raw pointer (DECY-115)
// ============================================================================

#[test]
fn box_variable_to_pointer_target_uses_box_into_raw() {
    let mut c = ctx();
    c.add_variable("node".to_string(), HirType::Box(Box::new(HirType::Int)));
    let target = HirType::Pointer(Box::new(HirType::Int));
    let expr = HirExpression::Variable("node".to_string());
    let result = expr_tt(&expr, &c, Some(&target));
    assert!(
        result.contains("Box::into_raw"),
        "Box var with Pointer target should use Box::into_raw, got: {}",
        result
    );
}

// ============================================================================
// Variable: Reference to pointer coercion (DECY-118/146)
// ============================================================================

#[test]
fn mutable_ref_array_to_pointer_uses_as_mut_ptr() {
    let mut c = ctx();
    c.add_variable(
        "arr".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(10),
            }),
            mutable: true,
        },
    );
    let target = HirType::Pointer(Box::new(HirType::Int));
    let expr = HirExpression::Variable("arr".to_string());
    let result = expr_tt(&expr, &c, Some(&target));
    assert!(
        result.contains("as_mut_ptr"),
        "Mutable ref array to ptr should use as_mut_ptr, got: {}",
        result
    );
}

#[test]
fn immutable_ref_array_to_pointer_uses_as_ptr() {
    let mut c = ctx();
    c.add_variable(
        "arr".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Array {
                element_type: Box::new(HirType::Int),
                size: Some(10),
            }),
            mutable: false,
        },
    );
    let target = HirType::Pointer(Box::new(HirType::Int));
    let expr = HirExpression::Variable("arr".to_string());
    let result = expr_tt(&expr, &c, Some(&target));
    assert!(
        result.contains("as_ptr"),
        "Immutable ref array to ptr should use as_ptr, got: {}",
        result
    );
}

#[test]
fn mutable_ref_vec_to_pointer_uses_as_mut_ptr() {
    let mut c = ctx();
    c.add_variable(
        "arr".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Vec(Box::new(HirType::Int))),
            mutable: true,
        },
    );
    let target = HirType::Pointer(Box::new(HirType::Int));
    let expr = HirExpression::Variable("arr".to_string());
    let result = expr_tt(&expr, &c, Some(&target));
    assert!(
        result.contains("as_mut_ptr"),
        "Mutable ref Vec to ptr should use as_mut_ptr, got: {}",
        result
    );
}

#[test]
fn mutable_ref_single_to_pointer_uses_as_star_mut() {
    let mut c = ctx();
    c.add_variable(
        "val".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        },
    );
    let target = HirType::Pointer(Box::new(HirType::Int));
    let expr = HirExpression::Variable("val".to_string());
    let result = expr_tt(&expr, &c, Some(&target));
    assert!(
        result.contains("as *mut _"),
        "&mut T to *mut T should cast, got: {}",
        result
    );
}

#[test]
fn immutable_ref_single_to_pointer_double_cast() {
    let mut c = ctx();
    c.add_variable(
        "val".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
    );
    let target = HirType::Pointer(Box::new(HirType::Int));
    let expr = HirExpression::Variable("val".to_string());
    let result = expr_tt(&expr, &c, Some(&target));
    assert!(
        result.contains("as *const _ as *mut _"),
        "&T to *mut T should double cast, got: {}",
        result
    );
}

// ============================================================================
// Variable: Array to pointer (DECY-211/244)
// ============================================================================

#[test]
fn array_to_pointer_same_type_uses_as_mut_ptr() {
    let mut c = ctx();
    c.add_variable(
        "buf".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Char),
            size: Some(256),
        },
    );
    let target = HirType::Pointer(Box::new(HirType::Char));
    let expr = HirExpression::Variable("buf".to_string());
    let result = expr_tt(&expr, &c, Some(&target));
    assert!(
        result.contains("as_mut_ptr"),
        "Array to ptr same type should use as_mut_ptr, got: {}",
        result
    );
}

#[test]
fn array_to_void_pointer_uses_as_mut_ptr_cast() {
    let mut c = ctx();
    c.add_variable(
        "buf".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    let target = HirType::Pointer(Box::new(HirType::Void));
    let expr = HirExpression::Variable("buf".to_string());
    let result = expr_tt(&expr, &c, Some(&target));
    assert!(
        result.contains("as_mut_ptr") && result.contains("as *mut ()"),
        "Array to void* should cast, got: {}",
        result
    );
}

// ============================================================================
// Variable: Pointer to pointer returns directly (DECY-148)
// ============================================================================

#[test]
fn pointer_variable_to_pointer_target_returns_directly() {
    let mut c = ctx();
    c.add_variable(
        "ptr".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    );
    let target = HirType::Pointer(Box::new(HirType::Int));
    let expr = HirExpression::Variable("ptr".to_string());
    let result = expr_tt(&expr, &c, Some(&target));
    assert_eq!(result, "ptr", "Pointer to pointer should return directly");
}

// ============================================================================
// Variable: renamed locals (DECY-245)
// ============================================================================

#[test]
fn renamed_local_uses_renamed_name() {
    let mut c = ctx();
    c.add_variable("count".to_string(), HirType::Int);
    c.add_renamed_local("count".to_string(), "local_count".to_string());
    let expr = HirExpression::Variable("count".to_string());
    let result = expr_no_tt(&expr, &c);
    assert_eq!(
        result, "local_count",
        "Renamed local should use renamed name, got: {}",
        result
    );
}

// ============================================================================
// StringLiteral with Pointer<Char> target (DECY-212)
// ============================================================================

#[test]
fn string_literal_to_pointer_char_generates_byte_string() {
    let c = ctx();
    let target = HirType::Pointer(Box::new(HirType::Char));
    let expr = HirExpression::StringLiteral("hello".to_string());
    let result = expr_tt(&expr, &c, Some(&target));
    assert!(
        result.contains("b\"hello\\0\"") && result.contains("as_ptr") && result.contains("*mut u8"),
        "String literal to char* should be byte string, got: {}",
        result
    );
}

#[test]
fn string_literal_no_target_stays_quoted() {
    let c = ctx();
    let expr = HirExpression::StringLiteral("hello".to_string());
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "\"hello\"");
}

// ============================================================================
// CharLiteral branches
// ============================================================================

#[test]
fn char_literal_null_generates_0u8() {
    let c = ctx();
    let expr = HirExpression::CharLiteral(0);
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "0u8");
}

#[test]
fn char_literal_printable_generates_byte_char() {
    let c = ctx();
    let expr = HirExpression::CharLiteral(65); // 'A'
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "b'A'");
}

#[test]
fn char_literal_space_generates_byte_space() {
    let c = ctx();
    let expr = HirExpression::CharLiteral(32); // ' '
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "b' '");
}

#[test]
fn char_literal_nonprintable_generates_numeric() {
    let c = ctx();
    let expr = HirExpression::CharLiteral(1); // SOH
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "1u8");
}

// ============================================================================
// IsNotNull expression
// ============================================================================

#[test]
fn is_not_null_generates_if_let_some() {
    let c = ctx();
    let expr = HirExpression::IsNotNull(Box::new(HirExpression::Variable("p".to_string())));
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("if let Some(_) = p"),
        "IsNotNull should generate if let Some, got: {}",
        result
    );
}

// ============================================================================
// StringMethodCall branches
// ============================================================================

#[test]
fn string_method_call_len_casts_to_i32() {
    let c = ctx();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("arr".to_string())),
        method: "len".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains(".len() as i32"),
        "len() should cast to i32, got: {}",
        result
    );
}

#[test]
fn string_method_call_no_args_non_len() {
    let c = ctx();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "is_empty".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "s.is_empty()");
}

#[test]
fn string_method_call_clone_into_adds_mut() {
    let c = ctx();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "clone_into".to_string(),
        arguments: vec![HirExpression::Variable("dest".to_string())],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("&mut dest"),
        "clone_into should add &mut, got: {}",
        result
    );
}

#[test]
fn string_method_call_with_args_non_clone_into() {
    let c = ctx();
    let expr = HirExpression::StringMethodCall {
        receiver: Box::new(HirExpression::Variable("s".to_string())),
        method: "push".to_string(),
        arguments: vec![HirExpression::CharLiteral(65)], // 'A'
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("s.push(b'A')"),
        "push method should pass arg directly, got: {}",
        result
    );
}

// ============================================================================
// Comparison: bool-to-int cast (DECY-191)
// ============================================================================

#[test]
fn comparison_with_int_target_casts_to_i32() {
    let c = ctx();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::GreaterThan,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let result = expr_tt(&expr, &c, Some(&HirType::Int));
    assert!(
        result.contains("as i32"),
        "Comparison with Int target should cast to i32, got: {}",
        result
    );
}

// ============================================================================
// Chained comparison (DECY-206): (a < b) < c
// ============================================================================

#[test]
fn chained_comparison_casts_bool_to_i32() {
    let c = ctx();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LessThan,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::LessThan,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }),
        right: Box::new(HirExpression::Variable("c".to_string())),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("as i32"),
        "Chained comparison should cast bool operand to i32, got: {}",
        result
    );
}

#[test]
fn chained_comparison_with_int_target() {
    let c = ctx();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::GreaterThan,
        left: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        }),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = expr_tt(&expr, &c, Some(&HirType::Int));
    assert!(
        result.contains("as i32"),
        "Chained comparison with int target should cast, got: {}",
        result
    );
}

// ============================================================================
// Signed/unsigned comparison mismatch (DECY-251)
// ============================================================================

#[test]
fn signed_unsigned_comparison_casts_to_i64() {
    let mut c = ctx();
    c.add_variable("s".to_string(), HirType::Int);
    c.add_variable("u".to_string(), HirType::UnsignedInt);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LessThan,
        left: Box::new(HirExpression::Variable("s".to_string())),
        right: Box::new(HirExpression::Variable("u".to_string())),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("as i64"),
        "Signed/unsigned comparison should cast to i64, got: {}",
        result
    );
}

#[test]
fn signed_unsigned_comparison_with_int_target() {
    let mut c = ctx();
    c.add_variable("s".to_string(), HirType::Int);
    c.add_variable("u".to_string(), HirType::UnsignedInt);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::GreaterEqual,
        left: Box::new(HirExpression::Variable("s".to_string())),
        right: Box::new(HirExpression::Variable("u".to_string())),
    };
    let result = expr_tt(&expr, &c, Some(&HirType::Int));
    assert!(
        result.contains("as i64") && result.contains("as i32"),
        "Signed/unsigned with int target should cast both, got: {}",
        result
    );
}

// ============================================================================
// Bitwise ops with bool operands (DECY-252)
// ============================================================================

#[test]
fn bitwise_and_with_bool_right_casts_to_i32() {
    let mut c = ctx();
    c.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::BitwiseAnd,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("y".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        }),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("as i32"),
        "Bitwise & with bool operand should cast, got: {}",
        result
    );
}

#[test]
fn bitwise_or_with_unsigned_and_bool_casts_back_to_u32() {
    let mut c = ctx();
    c.add_variable("flags".to_string(), HirType::UnsignedInt);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::BitwiseOr,
        left: Box::new(HirExpression::Variable("flags".to_string())),
        right: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::NotEqual,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("as u32"),
        "Bitwise | with unsigned + bool should cast to u32, got: {}",
        result
    );
}

// ============================================================================
// Arithmetic result cast to target type (DECY-204)
// ============================================================================

#[test]
fn int_add_int_with_float_target_casts_result() {
    let mut c = ctx();
    c.add_variable("a".to_string(), HirType::Int);
    c.add_variable("b".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let result = expr_tt(&expr, &c, Some(&HirType::Float));
    assert!(
        result.contains("as f32"),
        "int + int with Float target should cast to f32, got: {}",
        result
    );
}

#[test]
fn int_multiply_int_with_double_target_casts_result() {
    let mut c = ctx();
    c.add_variable("a".to_string(), HirType::Int);
    c.add_variable("b".to_string(), HirType::Int);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Multiply,
        left: Box::new(HirExpression::Variable("a".to_string())),
        right: Box::new(HirExpression::Variable("b".to_string())),
    };
    let result = expr_tt(&expr, &c, Some(&HirType::Double));
    assert!(
        result.contains("as f64"),
        "int * int with Double target should cast to f64, got: {}",
        result
    );
}

// ============================================================================
// Float + double mixed (DECY-204)
// ============================================================================

#[test]
fn float_plus_double_promotes_to_f64() {
    let mut c = ctx();
    c.add_variable("f".to_string(), HirType::Float);
    c.add_variable("d".to_string(), HirType::Double);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Variable("f".to_string())),
        right: Box::new(HirExpression::Variable("d".to_string())),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("as f64"),
        "float + double should promote float to f64, got: {}",
        result
    );
}

#[test]
fn double_minus_float_promotes_to_f64() {
    let mut c = ctx();
    c.add_variable("d".to_string(), HirType::Double);
    c.add_variable("f".to_string(), HirType::Float);
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Subtract,
        left: Box::new(HirExpression::Variable("d".to_string())),
        right: Box::new(HirExpression::Variable("f".to_string())),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("as f64"),
        "double - float should promote float to f64, got: {}",
        result
    );
}

// ============================================================================
// Assignment expression (DECY-195)
// ============================================================================

#[test]
fn assignment_expression_generates_block() {
    let c = ctx();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Assign,
        left: Box::new(HirExpression::Variable("x".to_string())),
        right: Box::new(HirExpression::IntLiteral(42)),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("__assign_tmp") && result.contains("42"),
        "Assignment expression should generate block with tmp, got: {}",
        result
    );
}

#[test]
fn assignment_expression_global_array_index() {
    let mut c = ctx();
    c.add_variable(
        "g_arr".to_string(),
        HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(10),
        },
    );
    c.add_global("g_arr".to_string());
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Assign,
        left: Box::new(HirExpression::ArrayIndex {
            array: Box::new(HirExpression::Variable("g_arr".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
        }),
        right: Box::new(HirExpression::IntLiteral(99)),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("unsafe") && result.contains("g_arr"),
        "Global array index assignment should be in unsafe, got: {}",
        result
    );
}

// ============================================================================
// Option comparison (DECY-144 / is_none / is_some)
// ============================================================================

#[test]
fn option_equal_null_is_none() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("p".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "p.is_none()");
}

#[test]
fn option_not_equal_null_is_some() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("p".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "p.is_some()");
}

#[test]
fn null_equal_option_is_none() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::NullLiteral),
        right: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "p.is_none()");
}

#[test]
fn null_not_equal_option_is_some() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Option(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::NullLiteral),
        right: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "p.is_some()");
}

// ============================================================================
// Pointer comparison with 0 (null pointer)
// ============================================================================

#[test]
fn pointer_equal_zero_becomes_null_mut() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("p".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("std::ptr::null_mut()"),
        "ptr == 0 should use null_mut(), got: {}",
        result
    );
}

#[test]
fn zero_not_equal_pointer_becomes_null_mut() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::IntLiteral(0)),
        right: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("std::ptr::null_mut()"),
        "0 != ptr should use null_mut(), got: {}",
        result
    );
}

// ============================================================================
// Vec null check (DECY-130)
// ============================================================================

#[test]
fn vec_equal_null_is_false() {
    let mut c = ctx();
    c.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Equal,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::NullLiteral),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("false"),
        "Vec == null should be false, got: {}",
        result
    );
}

#[test]
fn vec_not_equal_zero_is_true() {
    let mut c = ctx();
    c.add_variable("arr".to_string(), HirType::Vec(Box::new(HirType::Int)));
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::NotEqual,
        left: Box::new(HirExpression::Variable("arr".to_string())),
        right: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("true"),
        "Vec != 0 should be true, got: {}",
        result
    );
}

// ============================================================================
// Cast: Vec target over malloc unwraps cast (DECY-220)
// ============================================================================

#[test]
fn cast_over_malloc_with_vec_target_unwraps() {
    let c = ctx();
    let target = HirType::Vec(Box::new(HirType::Int));
    let expr = HirExpression::Cast {
        target_type: HirType::Pointer(Box::new(HirType::Int)),
        expr: Box::new(HirExpression::FunctionCall {
            function: "malloc".to_string(),
            arguments: vec![HirExpression::BinaryOp {
                op: BinaryOperator::Multiply,
                left: Box::new(HirExpression::Variable("n".to_string())),
                right: Box::new(HirExpression::IntLiteral(4)),
            }],
        }),
    };
    let result = expr_tt(&expr, &c, Some(&target));
    assert!(
        result.contains("vec!["),
        "Cast over malloc with Vec target should unwrap to vec!, got: {}",
        result
    );
}

// ============================================================================
// Cast: address-of to integer (DECY-208)
// ============================================================================

#[test]
fn cast_address_of_to_int_uses_pointer_chain() {
    let c = ctx();
    let expr = HirExpression::Cast {
        target_type: HirType::Int,
        expr: Box::new(HirExpression::AddressOf(Box::new(
            HirExpression::Variable("x".to_string()),
        ))),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("as *const _ as isize as i32"),
        "AddressOf to int should chain casts, got: {}",
        result
    );
}

#[test]
fn cast_binary_op_gets_parenthesized() {
    let c = ctx();
    let expr = HirExpression::Cast {
        target_type: HirType::Float,
        expr: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::IntLiteral(1)),
            right: Box::new(HirExpression::IntLiteral(2)),
        }),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("(1 + 2) as f32"),
        "Cast of binary op should parenthesize, got: {}",
        result
    );
}

// ============================================================================
// CompoundLiteral: Struct with partial init (DECY-133)
// ============================================================================

#[test]
fn compound_literal_struct_partial_init_uses_default() {
    let mut c = ctx();
    c.structs.insert(
        "Point".to_string(),
        vec![
            ("x".to_string(), HirType::Int),
            ("y".to_string(), HirType::Int),
            ("z".to_string(), HirType::Int),
        ],
    );
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Point".to_string()),
        initializers: vec![HirExpression::IntLiteral(10)],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("..Default::default()"),
        "Partial struct init should use Default, got: {}",
        result
    );
}

#[test]
fn compound_literal_struct_empty_init() {
    let c = ctx();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Foo".to_string()),
        initializers: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "Foo {}");
}

// ============================================================================
// CompoundLiteral: Array edge cases (DECY-257)
// ============================================================================

#[test]
fn compound_literal_array_single_init_repeats() {
    let c = ctx();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(5),
        },
        initializers: vec![HirExpression::IntLiteral(0)],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("[0; 5]"),
        "Single init with size should repeat, got: {}",
        result
    );
}

#[test]
fn compound_literal_array_partial_init_pads() {
    let c = ctx();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(4),
        },
        initializers: vec![HirExpression::IntLiteral(1), HirExpression::IntLiteral(2)],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("1, 2, 0i32, 0i32"),
        "Partial init should pad with defaults, got: {}",
        result
    );
}

#[test]
fn compound_literal_array_empty_with_size() {
    let c = ctx();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Double),
            size: Some(3),
        },
        initializers: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("[0.0f64; 3]"),
        "Empty array with size should use repeat syntax, got: {}",
        result
    );
}

#[test]
fn compound_literal_array_no_size_empty() {
    let c = ctx();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: None,
        },
        initializers: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "[]");
}

#[test]
fn compound_literal_other_type_generates_comment() {
    let c = ctx();
    let expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Int,
        initializers: vec![HirExpression::IntLiteral(42)],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("Compound literal"),
        "Unknown compound literal type should generate comment, got: {}",
        result
    );
}

// ============================================================================
// Ternary with target type propagation (DECY-213)
// ============================================================================

#[test]
fn ternary_propagates_target_type_to_branches() {
    let c = ctx();
    let target = HirType::Pointer(Box::new(HirType::Char));
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
        then_expr: Box::new(HirExpression::StringLiteral("yes".to_string())),
        else_expr: Box::new(HirExpression::StringLiteral("no".to_string())),
    };
    let result = expr_tt(&expr, &c, Some(&target));
    // Both branches should be converted to byte strings since target is *mut u8
    assert!(
        result.contains("b\"yes\\0\"") && result.contains("b\"no\\0\""),
        "Ternary branches should propagate target type, got: {}",
        result
    );
}

#[test]
fn ternary_non_boolean_condition_adds_not_eq_zero() {
    let c = ctx();
    let expr = HirExpression::Ternary {
        condition: Box::new(HirExpression::Variable("x".to_string())),
        then_expr: Box::new(HirExpression::IntLiteral(1)),
        else_expr: Box::new(HirExpression::IntLiteral(0)),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("!= 0"),
        "Non-boolean condition should add != 0, got: {}",
        result
    );
}

// ============================================================================
// PostIncrement / PreIncrement on pointers (DECY-253/255)
// ============================================================================

#[test]
fn post_increment_pointer_uses_wrapping_add() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("wrapping_add(1)"),
        "PostIncrement on pointer should use wrapping_add, got: {}",
        result
    );
}

#[test]
fn pre_increment_pointer_uses_wrapping_add() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PreIncrement {
        operand: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("wrapping_add(1)"),
        "PreIncrement on pointer should use wrapping_add, got: {}",
        result
    );
}

#[test]
fn post_decrement_pointer_uses_wrapping_sub() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PostDecrement {
        operand: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("wrapping_sub(1)"),
        "PostDecrement on pointer should use wrapping_sub, got: {}",
        result
    );
}

#[test]
fn pre_decrement_pointer_uses_wrapping_sub() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PreDecrement {
        operand: Box::new(HirExpression::Variable("p".to_string())),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("wrapping_sub(1)"),
        "PreDecrement on pointer should use wrapping_sub, got: {}",
        result
    );
}

// ============================================================================
// Post/PreIncrement on dereferenced pointers (DECY-255)
// ============================================================================

#[test]
fn post_increment_deref_pointer_unsafe() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("p".to_string()),
        ))),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("unsafe") && result.contains("*p"),
        "(*p)++ should generate unsafe deref, got: {}",
        result
    );
}

#[test]
fn pre_increment_deref_pointer_unsafe() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PreIncrement {
        operand: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("p".to_string()),
        ))),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("unsafe") && result.contains("*p"),
        "++(*p) should generate unsafe deref, got: {}",
        result
    );
}

#[test]
fn post_decrement_deref_pointer_unsafe() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PostDecrement {
        operand: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("p".to_string()),
        ))),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("unsafe") && result.contains("*p"),
        "(*p)-- should generate unsafe deref, got: {}",
        result
    );
}

#[test]
fn pre_decrement_deref_pointer_unsafe() {
    let mut c = ctx();
    c.add_variable("p".to_string(), HirType::Pointer(Box::new(HirType::Int)));
    let expr = HirExpression::PreDecrement {
        operand: Box::new(HirExpression::Dereference(Box::new(
            HirExpression::Variable("p".to_string()),
        ))),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("unsafe") && result.contains("*p"),
        "--(*p) should generate unsafe deref, got: {}",
        result
    );
}

// ============================================================================
// PostIncrement on &str (string iteration, DECY-138/158)
// ============================================================================

#[test]
fn post_increment_string_ref_generates_byte_access() {
    let mut c = ctx();
    c.add_variable("key".to_string(), HirType::StringReference);
    let expr = HirExpression::PostIncrement {
        operand: Box::new(HirExpression::Variable("key".to_string())),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("as_bytes()[0]") && result.contains("as u32"),
        "PostIncrement on &str should use as_bytes, got: {}",
        result
    );
}

// ============================================================================
// UnaryOp: LogicalNot with/without target (DECY-191)
// ============================================================================

#[test]
fn logical_not_bool_with_int_target_casts() {
    let c = ctx();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
    };
    let result = expr_tt(&expr, &c, Some(&HirType::Int));
    assert!(
        result.contains("as i32"),
        "!bool with Int target should cast, got: {}",
        result
    );
}

#[test]
fn logical_not_int_with_int_target_eq_zero_cast() {
    let c = ctx();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let result = expr_tt(&expr, &c, Some(&HirType::Int));
    assert!(
        result.contains("== 0") && result.contains("as i32"),
        "!int with Int target should use == 0 then cast, got: {}",
        result
    );
}

#[test]
fn logical_not_bool_without_target_no_cast() {
    let c = ctx();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        }),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.starts_with("!") && !result.contains("as i32"),
        "!bool without target should not cast, got: {}",
        result
    );
}

#[test]
fn logical_not_int_without_target_eq_zero_no_cast() {
    let c = ctx();
    let expr = HirExpression::UnaryOp {
        op: UnaryOperator::LogicalNot,
        operand: Box::new(HirExpression::Variable("x".to_string())),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("== 0") && !result.contains("as i32"),
        "!int without target should use == 0 without cast, got: {}",
        result
    );
}

// ============================================================================
// Dereference: string iteration index (DECY-134)
// ============================================================================

#[test]
fn dereference_string_iter_param_uses_index() {
    let mut c = ctx();
    c.add_variable("dest".to_string(), HirType::Pointer(Box::new(HirType::Char)));
    c.add_string_iter_param("dest".to_string(), "dest_idx".to_string());
    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("dest".to_string())));
    let result = expr_no_tt(&expr, &c);
    assert_eq!(
        result, "dest[dest_idx]",
        "Deref of string iter should use index, got: {result}",
    );
}

// ============================================================================
// Dereference: &str type (DECY-138)
// ============================================================================

#[test]
fn dereference_str_ref_uses_as_bytes() {
    let mut c = ctx();
    c.add_variable("s".to_string(), HirType::StringReference);
    let expr = HirExpression::Dereference(Box::new(HirExpression::Variable("s".to_string())));
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("as_bytes()[0] as i32"),
        "Deref of &str should use as_bytes, got: {}",
        result
    );
}

// ============================================================================
// Default function call: keyword renaming (DECY-241)
// ============================================================================

#[test]
fn function_named_write_gets_renamed() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "write".to_string(),
        arguments: vec![HirExpression::IntLiteral(1)],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("c_write("),
        "write function should be renamed to c_write, got: {}",
        result
    );
}

#[test]
fn function_named_read_gets_renamed() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "read".to_string(),
        arguments: vec![HirExpression::IntLiteral(0)],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("c_read("),
        "read function should be renamed to c_read, got: {}",
        result
    );
}

#[test]
fn function_named_type_gets_renamed() {
    let c = ctx();
    let expr = HirExpression::FunctionCall {
        function: "type".to_string(),
        arguments: vec![],
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("c_type("),
        "type function should be renamed to c_type, got: {}",
        result
    );
}

// ============================================================================
// convert_format_specifiers: remaining branches
// ============================================================================

#[test]
fn format_percent_percent_becomes_single_percent() {
    let result = CodeGenerator::convert_format_specifiers("%%");
    assert_eq!(result, "%");
}

#[test]
fn format_percent_d_simple() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%d"), "{}");
}

#[test]
fn format_percent_i_simple() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%i"), "{}");
}

#[test]
fn format_percent_u_simple() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%u"), "{}");
}

#[test]
fn format_percent_x_simple() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%x"), "{:x}");
}

#[test]
fn format_percent_x_upper_simple() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%X"), "{:X}");
}

#[test]
fn format_percent_o_simple() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%o"), "{:o}");
}

#[test]
fn format_percent_b_simple() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%b"), "{:b}");
}

#[test]
fn format_percent_f_simple() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%f"), "{}");
}

#[test]
fn format_percent_f_with_precision() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%.2f"), "{:.2}");
}

#[test]
fn format_percent_f_with_width_and_precision() {
    let result = CodeGenerator::convert_format_specifiers("%10.3f");
    assert_eq!(result, "{:10.3}");
}

#[test]
fn format_percent_f_with_width_no_precision() {
    let result = CodeGenerator::convert_format_specifiers("%10f");
    assert_eq!(result, "{:10}");
}

#[test]
fn format_percent_e_simple() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%e"), "{:e}");
}

#[test]
fn format_percent_e_upper() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%E"), "{:E}");
}

#[test]
fn format_percent_g_simple() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%g"), "{}");
}

#[test]
fn format_percent_g_upper() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%G"), "{}");
}

#[test]
fn format_percent_s_simple() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%s"), "{}");
}

#[test]
fn format_percent_s_with_width() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%20s"), "{:20}");
}

#[test]
fn format_percent_c_simple() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%c"), "{}");
}

#[test]
fn format_percent_p_simple() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%p"), "{:p}");
}

#[test]
fn format_unknown_specifier_kept_original() {
    // %Q is not a standard C format specifier
    let result = CodeGenerator::convert_format_specifiers("%Q");
    assert_eq!(result, "%Q");
}

#[test]
fn format_incomplete_at_end() {
    let result = CodeGenerator::convert_format_specifiers("hello%");
    assert!(
        result.contains("hello%"),
        "Incomplete format at end should keep original, got: {}",
        result
    );
}

#[test]
fn format_d_with_zero_pad_and_width() {
    let result = CodeGenerator::convert_format_specifiers("%05d");
    assert_eq!(result, "{:05}");
}

#[test]
fn format_x_with_zero_pad_and_width() {
    let result = CodeGenerator::convert_format_specifiers("%08x");
    assert_eq!(result, "{:08x}");
}

#[test]
fn format_x_upper_with_width() {
    let result = CodeGenerator::convert_format_specifiers("%4X");
    assert_eq!(result, "{:4X}");
}

#[test]
fn format_o_with_width() {
    let result = CodeGenerator::convert_format_specifiers("%6o");
    assert_eq!(result, "{:6o}");
}

#[test]
fn format_b_with_width() {
    let result = CodeGenerator::convert_format_specifiers("%8b");
    assert_eq!(result, "{:8b}");
}

#[test]
fn format_f_with_zero_pad_width_precision() {
    let result = CodeGenerator::convert_format_specifiers("%010.4f");
    assert_eq!(result, "{:010.4}");
}

#[test]
fn format_mixed_text_and_specifiers() {
    let result = CodeGenerator::convert_format_specifiers("x=%d, y=%f\\n");
    assert!(
        result.contains("x={}") && result.contains("y={}"),
        "Mixed text and specifiers should convert, got: {}",
        result
    );
}

#[test]
fn format_length_modifier_h_skipped() {
    let result = CodeGenerator::convert_format_specifiers("%hd");
    assert_eq!(result, "{}");
}

#[test]
fn format_length_modifier_hh_skipped() {
    let result = CodeGenerator::convert_format_specifiers("%hhd");
    assert_eq!(result, "{}");
}

#[test]
fn format_length_modifier_z_skipped() {
    let result = CodeGenerator::convert_format_specifiers("%zu");
    assert_eq!(result, "{}");
}

#[test]
fn format_length_modifier_j_skipped() {
    let result = CodeGenerator::convert_format_specifiers("%jd");
    assert_eq!(result, "{}");
}

#[test]
fn format_length_modifier_t_skipped() {
    let result = CodeGenerator::convert_format_specifiers("%td");
    assert_eq!(result, "{}");
}

#[test]
fn format_length_modifier_capital_l_skipped() {
    let result = CodeGenerator::convert_format_specifiers("%Lf");
    assert_eq!(result, "{}");
}

// ============================================================================
// NullLiteral
// ============================================================================

#[test]
fn null_literal_generates_none() {
    let c = ctx();
    let expr = HirExpression::NullLiteral;
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "None");
}

// ============================================================================
// Calloc HIR expression (not function call)
// ============================================================================

#[test]
fn calloc_hir_expr_signed_char_default() {
    let c = ctx();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(10)),
        element_type: Box::new(HirType::SignedChar),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("0i8"),
        "Calloc with SignedChar should use 0i8, got: {}",
        result
    );
}

#[test]
fn calloc_hir_expr_double_default() {
    let c = ctx();
    let expr = HirExpression::Calloc {
        count: Box::new(HirExpression::IntLiteral(5)),
        element_type: Box::new(HirType::Double),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("0.0f64"),
        "Calloc with Double should use 0.0f64, got: {}",
        result
    );
}

// ============================================================================
// Malloc HIR expression (not function call)
// ============================================================================

#[test]
fn malloc_hir_expr_multiply_generates_vec_with_capacity() {
    let c = ctx();
    let expr = HirExpression::Malloc {
        size: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::IntLiteral(4)),
        }),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("Vec::with_capacity"),
        "Malloc HIR with multiply should use Vec::with_capacity, got: {}",
        result
    );
}

#[test]
fn malloc_hir_expr_simple_generates_box_new() {
    let c = ctx();
    let expr = HirExpression::Malloc {
        size: Box::new(HirExpression::IntLiteral(4)),
    };
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "Box::new(0i32)");
}

// ============================================================================
// Realloc HIR expression
// ============================================================================

#[test]
fn realloc_hir_null_with_multiply_generates_vec() {
    let c = ctx();
    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::NullLiteral),
        new_size: Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::Variable("n".to_string())),
            right: Box::new(HirExpression::IntLiteral(4)),
        }),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("vec![0i32;"),
        "realloc(NULL, n*sz) should generate vec, got: {}",
        result
    );
}

#[test]
fn realloc_hir_null_simple_generates_vec_new() {
    let c = ctx();
    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::NullLiteral),
        new_size: Box::new(HirExpression::IntLiteral(100)),
    };
    let result = expr_no_tt(&expr, &c);
    assert_eq!(result, "Vec::new()");
}

#[test]
fn realloc_hir_non_null_returns_pointer() {
    let c = ctx();
    let expr = HirExpression::Realloc {
        pointer: Box::new(HirExpression::Variable("buf".to_string())),
        new_size: Box::new(HirExpression::IntLiteral(200)),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("buf"),
        "realloc non-null should return pointer, got: {}",
        result
    );
}

// ============================================================================
// Logical operators with bool operands (DECY-131)
// ============================================================================

#[test]
fn logical_and_with_bool_operands_no_conversion() {
    let c = ctx();
    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::LogicalAnd,
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
    };
    let result = expr_no_tt(&expr, &c);
    // Both operands are already boolean, so no != 0 conversion needed
    assert!(
        !result.contains("!= 0"),
        "Bool operands should not get != 0, got: {}",
        result
    );
}

// ============================================================================
// Sizeof edge cases (DECY-205/248)
// ============================================================================

#[test]
fn sizeof_known_variable_uses_size_of_val() {
    let mut c = ctx();
    c.add_variable("x".to_string(), HirType::Int);
    let expr = HirExpression::Sizeof {
        type_name: "x".to_string(),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("size_of_val(&x)"),
        "sizeof(variable) should use size_of_val, got: {}",
        result
    );
}

#[test]
fn sizeof_unknown_type_uses_size_of() {
    let c = ctx();
    let expr = HirExpression::Sizeof {
        type_name: "int".to_string(),
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("size_of::<"),
        "sizeof(type) should use size_of, got: {}",
        result
    );
}

// ============================================================================
// SliceIndex (DECY-070)
// ============================================================================

#[test]
fn slice_index_generates_usize_cast() {
    let c = ctx();
    let expr = HirExpression::SliceIndex {
        slice: Box::new(HirExpression::Variable("arr".to_string())),
        index: Box::new(HirExpression::Variable("i".to_string())),
        element_type: HirType::Int,
    };
    let result = expr_no_tt(&expr, &c);
    assert!(
        result.contains("arr[(i) as usize]"),
        "SliceIndex should generate usize cast, got: {}",
        result
    );
}

// ============================================================================
// Numeric type coercion: Float to int, unsigned
// ============================================================================

#[test]
fn float_var_to_int_target_casts_to_i32() {
    let mut c = ctx();
    c.add_variable("f".to_string(), HirType::Float);
    let expr = HirExpression::Variable("f".to_string());
    let result = expr_tt(&expr, &c, Some(&HirType::Int));
    assert!(
        result.contains("as i32"),
        "Float var with Int target should cast to i32, got: {}",
        result
    );
}

#[test]
fn double_var_to_unsigned_target_casts_to_u32() {
    let mut c = ctx();
    c.add_variable("d".to_string(), HirType::Double);
    let expr = HirExpression::Variable("d".to_string());
    let result = expr_tt(&expr, &c, Some(&HirType::UnsignedInt));
    assert!(
        result.contains("as u32"),
        "Double var with UnsignedInt target should cast to u32, got: {}",
        result
    );
}

#[test]
fn unsigned_int_to_float_target_casts_to_f32() {
    let mut c = ctx();
    c.add_variable("u".to_string(), HirType::UnsignedInt);
    let expr = HirExpression::Variable("u".to_string());
    let result = expr_tt(&expr, &c, Some(&HirType::Float));
    assert!(
        result.contains("as f32"),
        "UnsignedInt var with Float target should cast to f32, got: {}",
        result
    );
}

#[test]
fn unsigned_int_to_double_target_casts_to_f64() {
    let mut c = ctx();
    c.add_variable("u".to_string(), HirType::UnsignedInt);
    let expr = HirExpression::Variable("u".to_string());
    let result = expr_tt(&expr, &c, Some(&HirType::Double));
    assert!(
        result.contains("as f64"),
        "UnsignedInt var with Double target should cast to f64, got: {}",
        result
    );
}

#[test]
fn char_var_to_int_target_casts_to_i32() {
    let mut c = ctx();
    c.add_variable("ch".to_string(), HirType::Char);
    let expr = HirExpression::Variable("ch".to_string());
    let result = expr_tt(&expr, &c, Some(&HirType::Int));
    assert!(
        result.contains("as i32"),
        "Char var with Int target should cast to i32, got: {}",
        result
    );
}
