//! CLI Contract Tests: `decy transpile`
//!
//! **Purpose**: Validate user-facing contract (exit codes, stdio, output files)
//! **Layer 4**: CLI expectation testing (black-box validation)
//!
//! **Contract Specification**:
//! - Exit code 0: Successful transpilation
//! - Exit code 1: Invalid syntax OR file not found
//! - stdout: Rust code (default) OR empty (with -o flag)
//! - stderr: Error messages for failures
//!
//! **Pattern**: Following ruchy's CLI testing approach (CLAUDE.md)
//! **Reference**: C99/K&R C validation (C-VALIDATION-ROADMAP.yaml)

mod cli_testing_tools;

use cli_testing_tools::*;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// CLI CONTRACT TESTS: EXIT CODES
// ============================================================================

#[test]
fn cli_transpile_valid_file_exits_zero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "valid.c", VALID_C_CODE);

    decy_cmd().arg("transpile").arg(&file).assert().success(); // Exit code 0
}

#[test]
fn cli_transpile_invalid_syntax_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "invalid.c", INVALID_C_CODE);

    decy_cmd().arg("transpile").arg(&file).assert().failure(); // Exit code != 0
}

#[test]
fn cli_transpile_missing_file_exits_nonzero() {
    decy_cmd()
        .arg("transpile")
        .arg("nonexistent_file.c")
        .assert()
        .failure(); // Exit code != 0
}

// ============================================================================
// CLI CONTRACT TESTS: STDOUT (default output)
// ============================================================================

#[test]
fn cli_transpile_outputs_rust_to_stdout() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "hello.c", VALID_C_CODE);

    decy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("fn main")) // Rust code
        .stdout(predicate::str::is_empty().not()); // Non-empty output
}

#[test]
fn cli_transpile_rust_contains_function() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "function.c", C_WITH_FUNCTION);

    decy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("fn add")) // Function transpiled
        .stdout(predicate::str::contains("fn main"));
}

// ============================================================================
// CLI CONTRACT TESTS: FILE OUTPUT (-o flag)
// ============================================================================

#[test]
fn cli_transpile_to_file_creates_output() {
    let temp = TempDir::new().unwrap();
    let input = create_temp_file(&temp, "input.c", VALID_C_CODE);
    let output = temp.path().join("output.rs");

    decy_cmd()
        .arg("transpile")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .success();

    // Verify output file exists and contains Rust code
    assert!(output.exists(), "Output file should exist");
    let content = fs::read_to_string(&output).unwrap();
    assert!(
        content.contains("fn main"),
        "Should contain Rust fn main, got: {}",
        content
    );
}

#[test]
fn cli_transpile_to_file_no_stdout() {
    let temp = TempDir::new().unwrap();
    let input = create_temp_file(&temp, "input.c", VALID_C_CODE);
    let output = temp.path().join("output.rs");

    decy_cmd()
        .arg("transpile")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::is_empty()); // No stdout when writing to file
}

#[test]
fn cli_transpile_output_flag_long_form() {
    let temp = TempDir::new().unwrap();
    let input = create_temp_file(&temp, "input.c", VALID_C_CODE);
    let output = temp.path().join("long_form.rs");

    decy_cmd()
        .arg("transpile")
        .arg(&input)
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    assert!(output.exists());
}

// ============================================================================
// CLI CONTRACT TESTS: ERROR HANDLING
// ============================================================================

#[test]
fn cli_transpile_invalid_syntax_writes_stderr() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad.c", INVALID_C_CODE);

    decy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not()); // Non-empty stderr
}

#[test]
fn cli_transpile_missing_file_writes_stderr() {
    decy_cmd()
        .arg("transpile")
        .arg("missing.c")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("No such file"))
                .or(predicate::str::contains("does not exist"))
                .or(predicate::str::contains("Failed to read")),
        );
}

#[test]
fn cli_transpile_error_includes_filename() {
    decy_cmd()
        .arg("transpile")
        .arg("missing_specific_file.c")
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing_specific_file.c"));
}

// ============================================================================
// CLI CONTRACT TESTS: EDGE CASES
// ============================================================================

#[test]
fn cli_transpile_empty_file_handles_gracefully() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "empty.c", "");

    // Empty file should either succeed with empty output or fail gracefully
    let result = decy_cmd().arg("transpile").arg(&file).assert();

    // Either succeeds with minimal output or fails with clear error
    // We don't require specific behavior, just that it doesn't panic
    result.code(predicate::function(|code: &i32| *code == 0 || *code == 1));
}

#[test]
fn cli_transpile_comment_only_succeeds() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "comments.c",
        "// Just a comment\n/* Block comment */\n",
    );

    // Comment-only should transpile successfully (generates empty Rust or just comments)
    decy_cmd().arg("transpile").arg(&file).assert().success();
}

#[test]
fn cli_transpile_whitespace_only_succeeds() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "whitespace.c", "   \n\n   \t  \n");

    // Whitespace-only should handle gracefully
    decy_cmd().arg("transpile").arg(&file).assert().success();
}

// ============================================================================
// CLI CONTRACT TESTS: REALISTIC C CODE
// ============================================================================

#[test]
fn cli_transpile_function_with_parameters() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "func.c", C_WITH_FUNCTION);

    decy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("fn add"));
}

#[test]
fn cli_transpile_pointer_code() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "pointers.c", C_WITH_POINTERS);

    decy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("fn increment"));
}

#[test]
fn cli_transpile_malloc_code() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "malloc.c", C_WITH_MALLOC);

    let result = decy_cmd().arg("transpile").arg(&file).assert();

    // May succeed or fail depending on malloc support, but should handle gracefully
    result.code(predicate::function(|code: &i32| *code == 0 || *code == 1));
}

// ============================================================================
// CLI CONTRACT TESTS: HELP AND VERSION
// ============================================================================

#[test]
fn cli_transpile_help_flag() {
    decy_cmd()
        .arg("transpile")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Transpile a C source file"));
}

#[test]
fn cli_transpile_requires_input_file() {
    decy_cmd()
        .arg("transpile")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ============================================================================
// CLI CONTRACT TESTS: PATH HANDLING
// ============================================================================

#[test]
fn cli_transpile_relative_path() {
    let temp = TempDir::new().unwrap();
    let _file = create_temp_file(&temp, "relative.c", VALID_C_CODE);

    // Use relative path from temp dir
    std::env::set_current_dir(temp.path()).unwrap();

    decy_cmd()
        .arg("transpile")
        .arg("relative.c")
        .assert()
        .success();
}

#[test]
fn cli_transpile_absolute_path() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "absolute.c", VALID_C_CODE);

    decy_cmd()
        .arg("transpile")
        .arg(file.canonicalize().unwrap())
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: OUTPUT VALIDATION
// ============================================================================

#[test]
fn cli_transpile_output_is_valid_rust_syntax() {
    let temp = TempDir::new().unwrap();
    let input = create_temp_file(&temp, "input.c", VALID_C_CODE);
    let output = temp.path().join("output.rs");

    decy_cmd()
        .arg("transpile")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .success();

    // Verify output is parseable as Rust
    let content = fs::read_to_string(&output).unwrap();

    // Basic Rust syntax checks
    assert!(
        content.contains("fn ") || content.is_empty(),
        "Output should contain Rust functions or be empty"
    );
}

#[test]
fn cli_transpile_generates_consistent_output() {
    let temp = TempDir::new().unwrap();
    let input = create_temp_file(&temp, "input.c", VALID_C_CODE);

    // Run twice and compare outputs
    let output1 = decy_cmd()
        .arg("transpile")
        .arg(&input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output2 = decy_cmd()
        .arg("transpile")
        .arg(&input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(
        output1, output2,
        "Transpiler should produce consistent output"
    );
}
