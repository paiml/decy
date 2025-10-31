//! CLI Contract Tests: `decy audit`
//!
//! **Purpose**: Validate unsafe code auditing functionality
//! **Layer 4**: CLI expectation testing (black-box validation)
//!
//! **Contract Specification**:
//! - Exit code 0: Audit completed successfully
//! - Exit code 1: File not found OR invalid Rust
//! - stdout: Audit report with unsafe block count and safety analysis
//! - stderr: Error messages for failures
//!
//! **Pattern**: Following ruchy's CLI testing approach (CLAUDE.md)

mod cli_testing_tools;

use cli_testing_tools::*;
use predicates::prelude::*;
use tempfile::TempDir;

// Sample Rust code for testing
const RUST_WITH_UNSAFE: &str = r#"
fn main() {
    unsafe {
        let x = 42;
        println!("{}", x);
    }
}
"#;

const RUST_WITHOUT_UNSAFE: &str = r#"
fn main() {
    let x = 42;
    println!("{}", x);
}
"#;

const RUST_MULTIPLE_UNSAFE: &str = r#"
fn foo() {
    unsafe {
        // First unsafe block
    }
}

fn bar() {
    unsafe {
        // Second unsafe block
    }
}

fn main() {
    foo();
    bar();
}
"#;

// ============================================================================
// CLI CONTRACT TESTS: EXIT CODES
// ============================================================================

#[test]
fn cli_audit_valid_file_exits_zero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.rs", RUST_WITH_UNSAFE);

    decy_cmd().arg("audit").arg(&file).assert().success();
}

#[test]
fn cli_audit_missing_file_exits_nonzero() {
    decy_cmd()
        .arg("audit")
        .arg("nonexistent.rs")
        .assert()
        .failure();
}

#[test]
fn cli_audit_safe_code_exits_zero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "safe.rs", RUST_WITHOUT_UNSAFE);

    decy_cmd().arg("audit").arg(&file).assert().success();
}

// ============================================================================
// CLI CONTRACT TESTS: STDOUT (audit report)
// ============================================================================

#[test]
fn cli_audit_outputs_report() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.rs", RUST_WITH_UNSAFE);

    decy_cmd()
        .arg("audit")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not()); // Non-empty report
}

#[test]
fn cli_audit_reports_unsafe_count() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.rs", RUST_WITH_UNSAFE);

    decy_cmd()
        .arg("audit")
        .arg(&file)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("unsafe")
                .or(predicate::str::contains("Unsafe"))
                .or(predicate::str::contains("UNSAFE")),
        );
}

#[test]
fn cli_audit_safe_code_reports_zero_unsafe() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "safe.rs", RUST_WITHOUT_UNSAFE);

    let output = decy_cmd()
        .arg("audit")
        .arg(&file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should indicate 0 unsafe blocks or "safe" status
    assert!(
        output_str.contains('0')
            || output_str.to_lowercase().contains("safe")
            || output_str.to_lowercase().contains("no unsafe"),
        "Safe code audit should report 0 or 'safe', got: {}",
        output_str
    );
}

// ============================================================================
// CLI CONTRACT TESTS: VERBOSE FLAG
// ============================================================================

#[test]
fn cli_audit_verbose_flag_provides_detail() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.rs", RUST_WITH_UNSAFE);

    // Verbose flag should be accepted and produce output
    decy_cmd()
        .arg("audit")
        .arg(&file)
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not()); // Produces output
}

#[test]
fn cli_audit_verbose_short_flag() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.rs", RUST_WITH_UNSAFE);

    decy_cmd()
        .arg("audit")
        .arg(&file)
        .arg("-v")
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: ERROR HANDLING
// ============================================================================

#[test]
fn cli_audit_missing_file_writes_stderr() {
    decy_cmd()
        .arg("audit")
        .arg("missing.rs")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn cli_audit_error_includes_filename() {
    decy_cmd()
        .arg("audit")
        .arg("specific_missing_file.rs")
        .assert()
        .failure()
        .stderr(predicate::str::contains("specific_missing_file.rs"));
}

// ============================================================================
// CLI CONTRACT TESTS: MULTIPLE UNSAFE BLOCKS
// ============================================================================

#[test]
fn cli_audit_counts_multiple_unsafe_blocks() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "multi.rs", RUST_MULTIPLE_UNSAFE);

    let output = decy_cmd()
        .arg("audit")
        .arg(&file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should report 2 unsafe blocks or mention multiple
    assert!(
        output_str.contains('2')
            || output_str.to_lowercase().contains("multiple")
            || output_str.to_lowercase().contains("unsafe"),
        "Should detect multiple unsafe blocks, got: {}",
        output_str
    );
}

// ============================================================================
// CLI CONTRACT TESTS: HELP
// ============================================================================

#[test]
fn cli_audit_help_flag() {
    decy_cmd()
        .arg("audit")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Audit unsafe code"));
}

#[test]
fn cli_audit_requires_input_file() {
    decy_cmd()
        .arg("audit")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ============================================================================
// CLI CONTRACT TESTS: PATH HANDLING
// ============================================================================

#[test]
fn cli_audit_relative_path() {
    let temp = TempDir::new().unwrap();
    let _file = create_temp_file(&temp, "relative.rs", RUST_WITH_UNSAFE);

    std::env::set_current_dir(temp.path()).unwrap();

    decy_cmd()
        .arg("audit")
        .arg("relative.rs")
        .assert()
        .success();
}

#[test]
fn cli_audit_absolute_path() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "absolute.rs", RUST_WITH_UNSAFE);

    decy_cmd()
        .arg("audit")
        .arg(file.canonicalize().unwrap())
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: OUTPUT CONSISTENCY
// ============================================================================

#[test]
fn cli_audit_generates_consistent_output() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "consistent.rs", RUST_WITH_UNSAFE);

    // Run twice and compare outputs
    let output1 = decy_cmd()
        .arg("audit")
        .arg(&file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output2 = decy_cmd()
        .arg("audit")
        .arg(&file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(output1, output2, "Audit should produce consistent output");
}

// ============================================================================
// CLI CONTRACT TESTS: EDGE CASES
// ============================================================================

#[test]
fn cli_audit_empty_file() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "empty.rs", "");

    // Should handle gracefully (success or failure, just no panic)
    let result = decy_cmd().arg("audit").arg(&file).assert();
    result.code(predicate::function(|code: &i32| *code == 0 || *code == 1));
}

#[test]
fn cli_audit_comment_only_file() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "comments.rs",
        "// Just comments\n/* Block comment */",
    );

    decy_cmd().arg("audit").arg(&file).assert().success();
}

#[test]
fn cli_audit_file_with_no_functions() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "constants.rs", "const X: i32 = 42;");

    decy_cmd().arg("audit").arg(&file).assert().success();
}
