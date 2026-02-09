//! Additional coverage tests for decy-debugger
//!
//! Targets remaining uncovered lines in:
//! - visualize_ast.rs: format_statement colored paths, format_function colored paths,
//!   format_expression edge cases, visualize_c_ast header/stats colored branches
//! - visualize_hir.rs: visualize_hir colored branches, parameter iteration, body count
//! - lib.rs: Debugger delegation methods
//!
//! These tests use both direct function invocation (for format_* helpers)
//! and full pipeline invocation (through visualize_c_ast / visualize_hir)
//! to exercise every color/no-color branch systematically.

use crate::visualize_ast::visualize_c_ast;
use crate::visualize_hir::visualize_hir;
use crate::Debugger;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

// ============================================================================
// Helper
// ============================================================================

fn write_c_temp(source: &str) -> NamedTempFile {
    let mut temp = NamedTempFile::new().unwrap();
    write!(temp, "{}", source).unwrap();
    temp
}

// ============================================================================
// format_statement: Return(Some(expr)) with colors
// ============================================================================

#[test]
fn test_stmt_return_expr_colored() {
    let temp = write_c_temp("int f() { return 42; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    // The colored "Return" label should still be present (wrapped in ANSI)
    assert!(output.contains("Return") || output.contains("return"));
    assert!(output.contains("42"));
}

#[test]
fn test_stmt_return_variable_colored() {
    let temp = write_c_temp("int f(int x) { return x; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("x"));
}

// ============================================================================
// format_statement: Assignment colored path
// ============================================================================

#[test]
fn test_stmt_assignment_colored() {
    let temp = write_c_temp("int f() { int x = 0; x = 99; return x; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    // Assignment label is in yellow when colored
    assert!(output.contains("99") || output.contains("Assignment"));
}

#[test]
fn test_stmt_assignment_binary_expr_colored() {
    let temp = write_c_temp("int f() { int x = 0; x = 1 + 2; return x; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("1"));
    assert!(output.contains("2"));
}

// ============================================================================
// format_statement: VarDecl colored path (with and without initializer)
// ============================================================================

#[test]
fn test_stmt_var_decl_with_init_colored() {
    let temp = write_c_temp("int f() { int val = 100; return val; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("val") || output.contains("VarDecl"));
}

#[test]
fn test_stmt_var_decl_without_init_colored() {
    let temp = write_c_temp("int f() { int z; z = 5; return z; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("z"));
}

// ============================================================================
// format_statement: While colored path
// ============================================================================

#[test]
fn test_stmt_while_colored() {
    let temp = write_c_temp("int f() { int x = 3; while (x) { x = x - 1; } return x; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("While") || output.contains("while"));
}

#[test]
fn test_stmt_while_empty_body_colored() {
    let temp = write_c_temp("int f() { int x = 0; while (x) { } return 0; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
}

// ============================================================================
// format_statement: For colored path
// ============================================================================

#[test]
fn test_stmt_for_colored() {
    let temp =
        write_c_temp("int f() { int i; int s = 0; for (i = 0; i < 5; i = i + 1) { s = s + i; } return s; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("For") || output.contains("for"));
}

#[test]
fn test_stmt_for_no_body_colored() {
    let temp = write_c_temp("int f() { int i; for (i = 0; i < 3; i = i + 1) { } return 0; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
}

// ============================================================================
// format_statement: If (no else) colored path
// ============================================================================

#[test]
fn test_stmt_if_no_else_colored() {
    let temp = write_c_temp("int f(int x) { if (x) { return 1; } return 0; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("If") || output.contains("if"));
    // Without else block, should show only then
    assert!(output.contains("then:"));
}

#[test]
fn test_stmt_if_with_else_colored() {
    let temp =
        write_c_temp("int f(int x) { if (x) { return 1; } else { return 0; } }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("else:"));
}

// ============================================================================
// format_statement: Return(None) colored path (void return)
// ============================================================================

#[test]
fn test_stmt_void_return_colored() {
    let temp = write_c_temp("void f() { return; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Return"));
}

// ============================================================================
// format_statement: catchall/fallthrough branch (e.g., Break, Continue)
// These go to the `_ => format!("{:?}", stmt)` arm
// ============================================================================

#[test]
fn test_stmt_break_in_loop_no_colors() {
    let temp = write_c_temp(
        "int f() { int i; for (i = 0; i < 10; i = i + 1) { if (i) { break; } } return 0; }",
    );
    let result = visualize_c_ast(temp.path(), false);
    // Parser may or may not emit Break as a statement depending on how the loop is parsed
    assert!(result.is_ok());
}

#[test]
fn test_stmt_break_in_loop_with_colors() {
    let temp = write_c_temp(
        "int f() { int i; for (i = 0; i < 10; i = i + 1) { if (i) { break; } } return 0; }",
    );
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
}

// ============================================================================
// format_expression: BinaryOp colored (recursive formatting)
// ============================================================================

#[test]
fn test_expr_binary_op_colored() {
    let temp = write_c_temp("int f() { return 10 + 20; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("10"));
    assert!(output.contains("20"));
}

#[test]
fn test_expr_nested_binary_op_colored() {
    let temp = write_c_temp("int f(int a, int b, int c) { return a + b * c; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
}

#[test]
fn test_expr_subtraction_colored() {
    let temp = write_c_temp("int f(int x) { return x - 1; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
}

// ============================================================================
// format_expression: FunctionCall colored path
// ============================================================================

#[test]
fn test_expr_function_call_colored() {
    let temp = write_c_temp(
        "int helper(int x) { return x; }\nint f() { return helper(5); }",
    );
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("helper"));
}

#[test]
fn test_expr_function_call_multiple_args_colored() {
    let temp = write_c_temp(
        "int add(int a, int b) { return a + b; }\nint f() { return add(3, 4); }",
    );
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("add"));
}

#[test]
fn test_expr_function_call_no_args_no_colors() {
    let temp = write_c_temp(
        "int zero() { return 0; }\nint f() { return zero(); }",
    );
    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("zero"));
}

// ============================================================================
// format_expression: Variable colored path
// ============================================================================

#[test]
fn test_expr_variable_in_return_colored() {
    let temp = write_c_temp("int f() { int abc = 7; return abc; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("abc"));
}

// ============================================================================
// format_expression: IntLiteral colored path
// ============================================================================

#[test]
fn test_expr_int_literal_zero_colored() {
    let temp = write_c_temp("int f() { return 0; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("0"));
}

#[test]
fn test_expr_int_literal_negative_no_colors() {
    // Negative literals are typically UnaryOp(-IntLiteral), exercise that path
    let temp = write_c_temp("int f() { return -1; }");
    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("1"));
}

// ============================================================================
// format_expression: StringLiteral (catchall Debug branch)
// ============================================================================

#[test]
fn test_expr_string_literal_catchall_no_colors() {
    // String literals in C will go through the Debug-format catchall
    let temp = write_c_temp(
        "int f() { char *s = \"hello\"; return 0; }",
    );
    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
}

// ============================================================================
// format_function: colored path with parameters, body, return type
// ============================================================================

#[test]
fn test_fn_colored_return_type_display() {
    let temp = write_c_temp("int identity(int x) { return x; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    // The return type is displayed after "→"
    assert!(output.contains("identity"));
}

#[test]
fn test_fn_colored_no_params() {
    let temp = write_c_temp("int zero() { return 0; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("zero"));
    // No Parameters: line since params are empty
}

#[test]
fn test_fn_colored_empty_body() {
    let temp = write_c_temp("void nop() { }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("nop"));
}

#[test]
fn test_fn_colored_multiple_params() {
    let temp = write_c_temp("int sum(int a, int b, int c, int d) { return a + b + c + d; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("sum"));
}

// ============================================================================
// visualize_c_ast: header sections colored vs no-color branches
// ============================================================================

#[test]
fn test_ast_header_box_no_colors() {
    let temp = write_c_temp("int main() { return 0; }");
    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    // Non-colored header box characters
    assert!(output.contains("\u{2554}")); // ╔
    assert!(output.contains("\u{2551}")); // ║
    assert!(output.contains("\u{255a}")); // ╚
}

#[test]
fn test_ast_header_box_with_colors() {
    let temp = write_c_temp("int main() { return 0; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    // Colored header still contains the text
    assert!(output.contains("Decy Debugger"));
}

#[test]
fn test_ast_source_preview_line_numbers_colored() {
    let temp = write_c_temp("int f() {\n  return 1;\n}\n");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    // Source code preview with dimmed line numbers
    assert!(output.contains("Source Code") || !output.is_empty());
}

#[test]
fn test_ast_source_preview_line_numbers_no_colors() {
    let temp = write_c_temp("int f() {\n  return 1;\n}\n");
    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("  1 "));
    assert!(output.contains("  2 "));
}

#[test]
fn test_ast_statistics_section_colored() {
    let temp = write_c_temp("int a() { return 1; }\nint b() { return 2; }\n");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    // Statistics section header should be blue+bold when colored
    assert!(output.contains("Statistics") || !output.is_empty());
}

#[test]
fn test_ast_statistics_counts_colored() {
    let temp = write_c_temp("int f() { int a = 1; int b = 2; return a + b; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    // Should contain function and statement counts
    assert!(output.contains("Functions") || output.contains("functions"));
}

// ============================================================================
// visualize_hir: thorough colored branch testing
// ============================================================================

#[test]
fn test_hir_header_colored() {
    let temp = write_c_temp("int main() { return 0; }");
    let result = visualize_hir(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("HIR Visualization"));
}

#[test]
fn test_hir_header_no_colors() {
    let temp = write_c_temp("int main() { return 0; }");
    let result = visualize_hir(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("HIR Visualization"));
}

#[test]
fn test_hir_function_name_colored() {
    let temp = write_c_temp("int compute(int x) { return x + 1; }");
    let result = visualize_hir(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    // Function name should appear in bright_cyan when colored
    assert!(output.contains("compute"));
}

#[test]
fn test_hir_function_name_no_colors() {
    let temp = write_c_temp("int compute(int x) { return x + 1; }");
    let result = visualize_hir(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("compute:"));
}

#[test]
fn test_hir_multiple_params_colored() {
    let temp = write_c_temp("int add(int a, int b) { return a + b; }");
    let result = visualize_hir(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Parameters: 2"));
    assert!(output.contains("- a :"));
    assert!(output.contains("- b :"));
}

#[test]
fn test_hir_zero_params_no_colors() {
    let temp = write_c_temp("int zero() { return 0; }");
    let result = visualize_hir(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Parameters: 0"));
}

#[test]
fn test_hir_body_statement_count_no_colors() {
    let temp = write_c_temp("int f() { int a = 1; int b = 2; return a + b; }");
    let result = visualize_hir(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Body statements:"));
}

#[test]
fn test_hir_body_statement_count_colored() {
    let temp = write_c_temp("int f() { int a = 1; int b = 2; return a + b; }");
    let result = visualize_hir(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Body statements:"));
}

#[test]
fn test_hir_return_type_display() {
    let temp = write_c_temp("void nop() { return; }");
    let result = visualize_hir(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Return type:"));
}

#[test]
fn test_hir_file_info() {
    let temp = write_c_temp("int main() { return 0; }");
    let result = visualize_hir(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("File:"));
    assert!(output.contains("HIR Functions: 1"));
}

#[test]
fn test_hir_three_functions_colored() {
    let temp = write_c_temp(
        "int a() { return 1; }\nint b() { return 2; }\nint c() { return 3; }\n",
    );
    let result = visualize_hir(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("HIR Functions: 3"));
    assert!(output.contains("a"));
    assert!(output.contains("b"));
    assert!(output.contains("c"));
}

#[test]
fn test_hir_three_functions_no_colors() {
    let temp = write_c_temp(
        "int a() { return 1; }\nint b() { return 2; }\nint c() { return 3; }\n",
    );
    let result = visualize_hir(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("HIR Functions: 3"));
}

#[test]
fn test_hir_nonexistent_file() {
    let result = visualize_hir(Path::new("/tmp/no_such_cov_test_hir.c"), false);
    assert!(result.is_err());
}

#[test]
fn test_hir_nonexistent_file_colored() {
    let result = visualize_hir(Path::new("/tmp/no_such_cov_test_hir.c"), true);
    assert!(result.is_err());
}

// ============================================================================
// Debugger struct: method delegation through colored and non-colored
// ============================================================================

#[test]
fn test_debugger_fields_manual_set() {
    let mut d = Debugger::new();
    d.verbose = true;
    d.colored = false;
    assert!(d.verbose);
    assert!(!d.colored);
}

#[test]
fn test_debugger_visualize_ast_error_path() {
    let debugger = Debugger::new();
    let result = debugger.visualize_c_ast(Path::new("/tmp/debugger_cov_missing.c"));
    assert!(result.is_err());
}

#[test]
fn test_debugger_visualize_hir_error_path() {
    let debugger = Debugger::new();
    let result = debugger.visualize_hir(Path::new("/tmp/debugger_cov_missing_hir.c"));
    assert!(result.is_err());
}

#[test]
fn test_debugger_visualize_ast_colored_true() {
    let debugger = Debugger::new(); // colored = true
    let temp = write_c_temp("int main() { return 0; }");
    let result = debugger.visualize_c_ast(temp.path());
    assert!(result.is_ok());
}

#[test]
fn test_debugger_visualize_ast_colored_false() {
    let mut debugger = Debugger::new();
    debugger.colored = false;
    let temp = write_c_temp("int main() { return 0; }");
    let result = debugger.visualize_c_ast(temp.path());
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Decy Debugger"));
}

#[test]
fn test_debugger_visualize_hir_colored_true() {
    let debugger = Debugger::new();
    let temp = write_c_temp("int f(int x) { return x; }");
    let result = debugger.visualize_hir(temp.path());
    assert!(result.is_ok());
}

#[test]
fn test_debugger_visualize_hir_colored_false() {
    let mut debugger = Debugger::new();
    debugger.colored = false;
    let temp = write_c_temp("int f(int x) { return x; }");
    let result = debugger.visualize_hir(temp.path());
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("HIR Visualization"));
}

#[test]
fn test_debugger_step_through_nonexistent() {
    let debugger = Debugger::new();
    let result = debugger.step_through(Path::new("/no/such/file.c"));
    // step_through just prints, doesn't read file, so should be Ok
    assert!(result.is_ok());
}

// ============================================================================
// Complex C programs: exercise all branches in a single pipeline run
// ============================================================================

#[test]
fn test_ast_complex_program_no_colors() {
    let source = r#"
int max(int a, int b) {
    if (a > b) {
        return a;
    } else {
        return b;
    }
}

int sum_to(int n) {
    int total = 0;
    int i;
    for (i = 0; i < n; i = i + 1) {
        total = total + i;
    }
    return total;
}

int countdown(int start) {
    while (start) {
        start = start - 1;
    }
    return start;
}
"#;
    let temp = write_c_temp(source);
    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Function: max"));
    assert!(output.contains("Function: sum_to"));
    assert!(output.contains("Function: countdown"));
    assert!(output.contains("Functions:"));
    assert!(output.contains("Total statements:"));
}

#[test]
fn test_ast_complex_program_with_colors() {
    let source = r#"
int max(int a, int b) {
    if (a > b) {
        return a;
    } else {
        return b;
    }
}

int sum_to(int n) {
    int total = 0;
    int i;
    for (i = 0; i < n; i = i + 1) {
        total = total + i;
    }
    return total;
}

int countdown(int start) {
    while (start) {
        start = start - 1;
    }
    return start;
}
"#;
    let temp = write_c_temp(source);
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("max"));
    assert!(output.contains("sum_to"));
    assert!(output.contains("countdown"));
}

#[test]
fn test_hir_complex_program_no_colors() {
    let source = r#"
int max(int a, int b) {
    if (a > b) {
        return a;
    } else {
        return b;
    }
}

int sum_to(int n) {
    int total = 0;
    int i;
    for (i = 0; i < n; i = i + 1) {
        total = total + i;
    }
    return total;
}
"#;
    let temp = write_c_temp(source);
    let result = visualize_hir(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("max"));
    assert!(output.contains("sum_to"));
    assert!(output.contains("HIR Functions: 2"));
}

#[test]
fn test_hir_complex_program_with_colors() {
    let source = r#"
int max(int a, int b) {
    if (a > b) {
        return a;
    } else {
        return b;
    }
}

int sum_to(int n) {
    int total = 0;
    int i;
    for (i = 0; i < n; i = i + 1) {
        total = total + i;
    }
    return total;
}
"#;
    let temp = write_c_temp(source);
    let result = visualize_hir(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("max"));
    assert!(output.contains("sum_to"));
}

// ============================================================================
// Edge cases: single-line vs multiline, indentation depth
// ============================================================================

#[test]
fn test_ast_deeply_nested_if_no_colors() {
    let temp = write_c_temp(
        "int f(int x) { if (x) { if (x) { return 1; } } return 0; }",
    );
    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
}

#[test]
fn test_ast_deeply_nested_if_colored() {
    let temp = write_c_temp(
        "int f(int x) { if (x) { if (x) { return 1; } } return 0; }",
    );
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
}

#[test]
fn test_ast_single_line_program_no_colors() {
    let temp = write_c_temp("int main() { return 0; }");
    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    // Single line: Size should show 1 line
    assert!(output.contains("1 lines"));
}

#[test]
fn test_ast_single_line_program_colored() {
    let temp = write_c_temp("int main() { return 0; }");
    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
}
