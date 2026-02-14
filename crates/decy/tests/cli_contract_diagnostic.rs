//! CLI Contract Tests: Diagnostic Error Output Quality
//!
//! **Purpose**: Validate that parse errors produce rich, rustc-style diagnostics
//! with source locations, code snippets, explanatory notes, and fix suggestions.
//!
//! **Contract Specification**:
//! - Parse errors show `error[parse]:` header
//! - Parse errors show `--> file:line:col` location
//! - Parse errors show code snippet with `|` gutter
//! - Parse errors show `note:` and `help:` guidance
//! - Valid C produces no diagnostic output on stderr
//! - Multiple errors show all diagnostics

mod cli_testing_tools;

use cli_testing_tools::*;
use predicates::prelude::*;
use tempfile::TempDir;

// ============================================================================
// DIAGNOSTIC FORMAT: ERROR HEADER
// ============================================================================

#[test]
fn cli_diagnostic_contains_error_header() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad.c", "int main( { }");

    decy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("error[parse]"));
}

#[test]
fn cli_diagnostic_contains_error_message() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad.c", "int main( { }");

    decy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("expected"));
}

// ============================================================================
// DIAGNOSTIC FORMAT: SOURCE LOCATION
// ============================================================================

#[test]
fn cli_diagnostic_contains_location_arrow() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad.c", "int main( { }");

    decy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("-->"));
}

// ============================================================================
// DIAGNOSTIC FORMAT: CODE SNIPPET
// ============================================================================

#[test]
fn cli_diagnostic_contains_code_snippet() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad.c", "int main( { }");

    decy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("|"));
}

#[test]
fn cli_diagnostic_contains_caret() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad.c", "int main( { }");

    decy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("^"));
}

// ============================================================================
// DIAGNOSTIC FORMAT: NOTE AND HELP
// ============================================================================

#[test]
fn cli_diagnostic_contains_note() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad.c", "int main( { }");

    decy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("note:"));
}

#[test]
fn cli_diagnostic_contains_help() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad.c", "int main( { }");

    decy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("help:"));
}

// ============================================================================
// DIAGNOSTIC: VALID CODE PRODUCES NO DIAGNOSTICS
// ============================================================================

#[test]
fn cli_diagnostic_valid_code_no_error_header() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "good.c", VALID_C_CODE);

    decy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .success()
        .stderr(predicate::str::contains("error[parse]").not());
}

// ============================================================================
// DIAGNOSTIC: DIFFERENT ERROR TYPES
// ============================================================================

#[test]
fn cli_diagnostic_missing_semicolon() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "nosemi.c", "int main() { int x = 5 return 0; }");

    decy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("error["));
}

#[test]
fn cli_diagnostic_undeclared_identifier() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "undeclared.c",
        "int main() { return undefined_var; }",
    );

    decy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("error["));
}

// ============================================================================
// DIAGNOSTIC: MULTI-LINE SOURCE CONTEXT
// ============================================================================

#[test]
fn cli_diagnostic_multiline_source_shows_context() {
    let temp = TempDir::new().unwrap();
    let code = "int foo() {\n    return 0;\n}\n\nint main( {\n    return 0;\n}\n";
    let file = create_temp_file(&temp, "multiline.c", code);

    decy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("|"));
}
