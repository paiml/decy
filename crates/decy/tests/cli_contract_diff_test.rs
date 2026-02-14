//! CLI Contract Tests: `decy diff-test`
//!
//! **Purpose**: Validate differential testing (S5) CLI subcommand
//! **Layer 4**: CLI expectation testing (black-box validation)
//!
//! **Contract Specification**:
//! - Exit code 0: C and Rust produce identical stdout + exit code
//! - Exit code 1: Divergence detected OR compilation failure
//! - stdout: PASS/FAIL report
//! - stderr: Error messages for failures
//!
//! **Pattern**: Following ruchy's CLI testing approach (CLAUDE.md)

mod cli_testing_tools;

use cli_testing_tools::*;
use predicates::prelude::*;
use tempfile::TempDir;

const RETURN_ONLY_C: &str = "int main() { return 0; }";

const INVALID_C: &str = "int main( { }";

// ============================================================================
// CLI CONTRACT TESTS: EXIT CODES
// ============================================================================

#[test]
fn cli_diff_test_valid_file_exits_zero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.c", RETURN_ONLY_C);

    decy_cmd()
        .arg("diff-test")
        .arg(&file)
        .assert()
        .success();
}

#[test]
fn cli_diff_test_missing_file_exits_nonzero() {
    decy_cmd()
        .arg("diff-test")
        .arg("nonexistent_file.c")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn cli_diff_test_invalid_c_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad.c", INVALID_C);

    decy_cmd()
        .arg("diff-test")
        .arg(&file)
        .assert()
        .failure();
}

// ============================================================================
// CLI CONTRACT TESTS: STDOUT/STDERR
// ============================================================================

#[test]
fn cli_diff_test_pass_reports_pass() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.c", RETURN_ONLY_C);

    decy_cmd()
        .arg("diff-test")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("PASS"));
}

#[test]
fn cli_diff_test_missing_file_shows_error() {
    decy_cmd()
        .arg("diff-test")
        .arg("missing.c")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

// ============================================================================
// CLI CONTRACT TESTS: FLAGS
// ============================================================================

#[test]
fn cli_diff_test_timeout_flag() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.c", RETURN_ONLY_C);

    decy_cmd()
        .arg("diff-test")
        .arg(&file)
        .arg("--timeout")
        .arg("10")
        .assert()
        .success();
}

#[test]
fn cli_diff_test_help_flag() {
    decy_cmd()
        .arg("diff-test")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Differential test"));
}

#[test]
fn cli_diff_test_requires_input_file() {
    decy_cmd()
        .arg("diff-test")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}
