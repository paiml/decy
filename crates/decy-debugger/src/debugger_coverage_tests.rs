//! Coverage expansion tests for decy-debugger
//!
//! Targets uncovered lines in:
//! - visualize_ast.rs: visualize_c_ast (which calls format_function, format_statement, format_expression)
//! - visualize_hir.rs: visualize_hir
//!
//! These tests exercise the public API which drives all the internal formatting code.
//! We test with both use_colors=true and use_colors=false to cover all color branches.

use crate::visualize_ast::visualize_c_ast;
use crate::visualize_hir::visualize_hir;
use crate::Debugger;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

// ============================================================================
// visualize_c_ast: No-color path (exercises all formatting without ANSI codes)
// ============================================================================

#[test]
fn test_ast_simple_return_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ return 0; }}").unwrap();

    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Decy Debugger: C AST Visualization"));
    assert!(output.contains("File:"));
    assert!(output.contains("Size:"));
    assert!(output.contains("Source Code"));
    assert!(output.contains("Abstract Syntax Tree"));
    assert!(output.contains("Function: main"));
    assert!(output.contains("Return"));
    assert!(output.contains("Statistics"));
    assert!(output.contains("Functions:"));
    assert!(output.contains("Total statements:"));
}

#[test]
fn test_ast_simple_return_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ return 0; }}").unwrap();

    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    // Output should still contain the function name and relevant content,
    // albeit wrapped in ANSI color codes
    assert!(!output.is_empty());
    assert!(output.contains("main"));
}

#[test]
fn test_ast_nonexistent_file() {
    let result = visualize_c_ast(Path::new("/tmp/nonexistent_coverage_test_abc.c"), false);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Failed to read file"));
}

// ============================================================================
// Exercise format_function branches through full pipeline
// ============================================================================

#[test]
fn test_ast_function_with_parameters_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int add(int a, int b) {{ return a + b; }}").unwrap();

    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Function: add"));
    assert!(output.contains("Parameters:"));
    assert!(output.contains("Body:"));
}

#[test]
fn test_ast_function_with_parameters_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int add(int a, int b) {{ return a + b; }}").unwrap();

    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("add"));
}

#[test]
fn test_ast_function_three_params_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(
        temp,
        "int sum3(int a, int b, int c) {{ return a + b + c; }}"
    )
    .unwrap();

    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Parameters:"));
    // Multiple parameters should be comma-separated
    assert!(output.contains(", "));
}

#[test]
fn test_ast_function_three_params_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(
        temp,
        "int sum3(int a, int b, int c) {{ return a + b + c; }}"
    )
    .unwrap();

    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
}

#[test]
fn test_ast_empty_function_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "void noop() {{ }}").unwrap();

    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Function: noop"));
    // Empty body should not show Body: label
}

#[test]
fn test_ast_empty_function_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "void noop() {{ }}").unwrap();

    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
}

#[test]
fn test_ast_void_return_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "void nothing() {{ return; }}").unwrap();

    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    // void return should show "Return (void)"
    assert!(output.contains("Return"));
}

#[test]
fn test_ast_void_return_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "void nothing() {{ return; }}").unwrap();

    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Return"));
}

// ============================================================================
// Exercise format_statement branches through full pipeline
// ============================================================================

#[test]
fn test_ast_variable_declaration_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ int x = 42; return x; }}").unwrap();

    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("VarDecl"));
}

#[test]
fn test_ast_variable_declaration_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ int x = 42; return x; }}").unwrap();

    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
}

#[test]
fn test_ast_variable_declaration_no_init_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ int y; y = 5; return y; }}").unwrap();

    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
}

#[test]
fn test_ast_assignment_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ int x = 0; x = 42; return x; }}").unwrap();

    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Assignment"));
}

#[test]
fn test_ast_assignment_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ int x = 0; x = 42; return x; }}").unwrap();

    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
}

#[test]
fn test_ast_if_no_else_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ int x = 1; if (x) {{ return 1; }} return 0; }}").unwrap();

    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("If"));
    assert!(output.contains("then:"));
}

#[test]
fn test_ast_if_no_else_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ int x = 1; if (x) {{ return 1; }} return 0; }}").unwrap();

    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
}

#[test]
fn test_ast_if_with_else_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(
        temp,
        "int main() {{ int x = 5; if (x) {{ return 1; }} else {{ return 0; }} }}"
    )
    .unwrap();

    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("If"));
    assert!(output.contains("else:"));
}

#[test]
fn test_ast_if_with_else_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(
        temp,
        "int main() {{ int x = 5; if (x) {{ return 1; }} else {{ return 0; }} }}"
    )
    .unwrap();

    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
}

#[test]
fn test_ast_while_loop_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(
        temp,
        "int main() {{ int x = 10; while (x) {{ x = x - 1; }} return 0; }}"
    )
    .unwrap();

    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("While"));
}

#[test]
fn test_ast_while_loop_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(
        temp,
        "int main() {{ int x = 10; while (x) {{ x = x - 1; }} return 0; }}"
    )
    .unwrap();

    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
}

#[test]
fn test_ast_for_loop_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(
        temp,
        "int main() {{ int i; for (i = 0; i < 10; i = i + 1) {{ }} return 0; }}"
    )
    .unwrap();

    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("For") || output.contains("for"));
}

#[test]
fn test_ast_for_loop_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(
        temp,
        "int main() {{ int i; for (i = 0; i < 10; i = i + 1) {{ }} return 0; }}"
    )
    .unwrap();

    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
}

// ============================================================================
// Exercise format_expression branches through full pipeline
// ============================================================================

#[test]
fn test_ast_binary_expression_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ return 3 + 4; }}").unwrap();

    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("3"));
    assert!(output.contains("4"));
}

#[test]
fn test_ast_binary_expression_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ return 3 + 4; }}").unwrap();

    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
}

#[test]
fn test_ast_function_call_expression_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(
        temp,
        "int helper(int x) {{ return x; }}\nint main() {{ return helper(42); }}"
    )
    .unwrap();

    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    // Should contain the function call info
    assert!(output.contains("helper") || output.contains("Function"));
}

#[test]
fn test_ast_function_call_expression_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(
        temp,
        "int helper(int x) {{ return x; }}\nint main() {{ return helper(42); }}"
    )
    .unwrap();

    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
}

#[test]
fn test_ast_nested_binary_expression() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ return (1 + 2) * (3 - 4); }}").unwrap();

    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
}

#[test]
fn test_ast_variable_expression_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ int x = 5; return x; }}").unwrap();

    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
}

#[test]
fn test_ast_variable_expression_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ int x = 5; return x; }}").unwrap();

    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
}

// ============================================================================
// Multiple function statistics
// ============================================================================

#[test]
fn test_ast_statistics_multiple_functions_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int f1() {{ return 1; }}").unwrap();
    writeln!(temp, "int f2() {{ return 2; }}").unwrap();
    writeln!(temp, "int f3(int a) {{ return a; }}").unwrap();

    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Statistics"));
    assert!(output.contains("Functions:"));
    assert!(output.contains("Total statements:"));
}

#[test]
fn test_ast_statistics_multiple_functions_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int f1() {{ return 1; }}").unwrap();
    writeln!(temp, "int f2() {{ return 2; }}").unwrap();

    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
}

// ============================================================================
// Multiline source preview
// ============================================================================

#[test]
fn test_ast_multiline_source_preview_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{").unwrap();
    writeln!(temp, "  int a = 1;").unwrap();
    writeln!(temp, "  int b = 2;").unwrap();
    writeln!(temp, "  return a + b;").unwrap();
    writeln!(temp, "}}").unwrap();

    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    // Line numbers in the source preview
    assert!(output.contains("  1 "));
    assert!(output.contains("  2 "));
    assert!(output.contains("  3 "));
    assert!(output.contains("  4 "));
    assert!(output.contains("  5 "));
}

#[test]
fn test_ast_multiline_source_preview_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{").unwrap();
    writeln!(temp, "  int a = 1;").unwrap();
    writeln!(temp, "  return a;").unwrap();
    writeln!(temp, "}}").unwrap();

    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_empty());
}

// ============================================================================
// Complex integration: all statement types together
// ============================================================================

#[test]
fn test_ast_all_statement_types_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int complex(int n) {{").unwrap();
    writeln!(temp, "  int result = 0;").unwrap();
    writeln!(temp, "  int i;").unwrap();
    writeln!(temp, "  if (n) {{").unwrap();
    writeln!(temp, "    result = 1;").unwrap();
    writeln!(temp, "  }} else {{").unwrap();
    writeln!(temp, "    result = 0;").unwrap();
    writeln!(temp, "  }}").unwrap();
    writeln!(temp, "  for (i = 0; i < n; i = i + 1) {{").unwrap();
    writeln!(temp, "    result = result + i;").unwrap();
    writeln!(temp, "  }}").unwrap();
    writeln!(temp, "  while (result) {{").unwrap();
    writeln!(temp, "    result = result - 1;").unwrap();
    writeln!(temp, "  }}").unwrap();
    writeln!(temp, "  return result;").unwrap();
    writeln!(temp, "}}").unwrap();

    let result = visualize_c_ast(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Function: complex"));
    assert!(output.contains("VarDecl") || output.contains("VariableDeclaration"));
    assert!(output.contains("Assignment") || output.contains("assignment"));
}

#[test]
fn test_ast_all_statement_types_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int complex(int n) {{").unwrap();
    writeln!(temp, "  int result = 0;").unwrap();
    writeln!(temp, "  int i;").unwrap();
    writeln!(temp, "  if (n) {{").unwrap();
    writeln!(temp, "    result = 1;").unwrap();
    writeln!(temp, "  }} else {{").unwrap();
    writeln!(temp, "    result = 0;").unwrap();
    writeln!(temp, "  }}").unwrap();
    writeln!(temp, "  for (i = 0; i < n; i = i + 1) {{").unwrap();
    writeln!(temp, "    result = result + i;").unwrap();
    writeln!(temp, "  }}").unwrap();
    writeln!(temp, "  while (result) {{").unwrap();
    writeln!(temp, "    result = result - 1;").unwrap();
    writeln!(temp, "  }}").unwrap();
    writeln!(temp, "  return result;").unwrap();
    writeln!(temp, "}}").unwrap();

    let result = visualize_c_ast(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("complex"));
}

// ============================================================================
// visualize_hir: comprehensive coverage
// ============================================================================

#[test]
fn test_hir_simple_function_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int add(int a, int b) {{ return a + b; }}").unwrap();

    let result = visualize_hir(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("HIR Visualization"));
    assert!(output.contains("File:"));
    assert!(output.contains("HIR Functions:"));
    assert!(output.contains("add"));
    assert!(output.contains("Return type:"));
    assert!(output.contains("Parameters: 2"));
    assert!(output.contains("Body statements:"));
}

#[test]
fn test_hir_simple_function_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int add(int a, int b) {{ return a + b; }}").unwrap();

    let result = visualize_hir(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("add"));
    assert!(output.contains("HIR Visualization"));
}

#[test]
fn test_hir_no_params_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ return 0; }}").unwrap();

    let result = visualize_hir(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("main"));
    assert!(output.contains("Parameters: 0"));
}

#[test]
fn test_hir_no_params_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ return 0; }}").unwrap();

    let result = visualize_hir(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("main"));
}

#[test]
fn test_hir_multiple_functions_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int f1() {{ return 1; }}").unwrap();
    writeln!(temp, "int f2() {{ return 2; }}").unwrap();

    let result = visualize_hir(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("HIR Functions: 2"));
}

#[test]
fn test_hir_multiple_functions_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int f1() {{ return 1; }}").unwrap();
    writeln!(temp, "int f2() {{ return 2; }}").unwrap();

    let result = visualize_hir(temp.path(), true);
    assert!(result.is_ok());
}

#[test]
fn test_hir_parameter_details_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int sum(int x, int y) {{ return x + y; }}").unwrap();

    let result = visualize_hir(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("- x :"));
    assert!(output.contains("- y :"));
}

#[test]
fn test_hir_parameter_details_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int sum(int x, int y) {{ return x + y; }}").unwrap();

    let result = visualize_hir(temp.path(), true);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("- x :"));
    assert!(output.contains("- y :"));
}

#[test]
fn test_hir_nonexistent_file_error() {
    let result = visualize_hir(Path::new("/tmp/nonexistent_hir_coverage_test.c"), false);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Failed to read file"));
}

#[test]
fn test_hir_body_statements_count() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{").unwrap();
    writeln!(temp, "  int a = 1;").unwrap();
    writeln!(temp, "  int b = 2;").unwrap();
    writeln!(temp, "  return a + b;").unwrap();
    writeln!(temp, "}}").unwrap();

    let result = visualize_hir(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Body statements:"));
}

#[test]
fn test_hir_void_function_no_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "void noop() {{ return; }}").unwrap();

    let result = visualize_hir(temp.path(), false);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("noop"));
}

#[test]
fn test_hir_void_function_with_colors() {
    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "void noop() {{ return; }}").unwrap();

    let result = visualize_hir(temp.path(), true);
    assert!(result.is_ok());
}

// ============================================================================
// Debugger struct: method delegation
// ============================================================================

#[test]
fn test_debugger_default() {
    let debugger = Debugger::default();
    assert!(!debugger.verbose);
    assert!(!debugger.colored);
}

#[test]
fn test_debugger_new() {
    let debugger = Debugger::new();
    assert!(!debugger.verbose);
    assert!(debugger.colored);
}

#[test]
fn test_debugger_visualize_c_ast_no_colors() {
    let mut debugger = Debugger::new();
    debugger.colored = false;

    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ return 0; }}").unwrap();

    let result = debugger.visualize_c_ast(temp.path());
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Decy Debugger: C AST Visualization"));
}

#[test]
fn test_debugger_visualize_c_ast_with_colors() {
    let debugger = Debugger::new(); // colored=true by default

    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ return 0; }}").unwrap();

    let result = debugger.visualize_c_ast(temp.path());
    assert!(result.is_ok());
}

#[test]
fn test_debugger_visualize_hir_no_colors() {
    let mut debugger = Debugger::new();
    debugger.colored = false;

    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ return 0; }}").unwrap();

    let result = debugger.visualize_hir(temp.path());
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("HIR Visualization"));
}

#[test]
fn test_debugger_visualize_hir_with_colors() {
    let debugger = Debugger::new();

    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ return 0; }}").unwrap();

    let result = debugger.visualize_hir(temp.path());
    assert!(result.is_ok());
}

#[test]
fn test_debugger_visualize_ownership_error_path() {
    let debugger = Debugger::new();
    let result = debugger.visualize_ownership(Path::new("/tmp/nonexistent_owner_cov_test.c"));
    assert!(result.is_err());
}

#[test]
fn test_debugger_visualize_ownership_no_colors() {
    let mut debugger = Debugger::new();
    debugger.colored = false;

    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ return 0; }}").unwrap();

    let result = debugger.visualize_ownership(temp.path());
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Ownership Analysis"));
}

#[test]
fn test_debugger_visualize_ownership_with_colors() {
    let debugger = Debugger::new();

    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "int main() {{ return 0; }}").unwrap();

    let result = debugger.visualize_ownership(temp.path());
    assert!(result.is_ok());
}

#[test]
fn test_debugger_step_through_ok() {
    let debugger = Debugger::new();
    let path = Path::new("/tmp/step_test.c");
    let result = debugger.step_through(path);
    assert!(result.is_ok());
}

#[test]
fn test_debugger_verbose_step_through() {
    let mut debugger = Debugger::new();
    debugger.verbose = true;
    let path = Path::new("/tmp/step_verbose_test.c");
    let result = debugger.step_through(path);
    assert!(result.is_ok());
}

#[test]
fn test_debugger_debug_trait() {
    let debugger = Debugger::new();
    let debug_str = format!("{:?}", debugger);
    assert!(debug_str.contains("Debugger"));
}
