//! CLI Contract Tests for `decy oracle query`
//!
//! DECY-110: Oracle query command for pattern lookup
//!
//! Per unified spec Section 6.7, this command queries the oracle
//! for fix patterns for specific rustc error codes.
//!
//! CLI: decy oracle query --error E0308 --context "let x: &mut i32 = ptr"

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper: Create decy command
fn decy_cmd() -> Command {
    Command::cargo_bin("decy").expect("Failed to find decy binary")
}

// ============================================================================
// CLI CONTRACT TESTS: EXIT CODES
// ============================================================================

#[test]
fn cli_oracle_query_valid_error_code_exits_zero() {
    decy_cmd()
        .arg("oracle")
        .arg("query")
        .arg("--error")
        .arg("E0308")
        .assert()
        .success(); // Exit code 0
}

#[test]
fn cli_oracle_query_with_context_exits_zero() {
    decy_cmd()
        .arg("oracle")
        .arg("query")
        .arg("--error")
        .arg("E0382")
        .arg("--context")
        .arg("let x = value; use(x);")
        .assert()
        .success();
}

#[test]
fn cli_oracle_query_missing_error_arg_exits_nonzero() {
    decy_cmd()
        .arg("oracle")
        .arg("query")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--error"));
}

#[test]
fn cli_oracle_query_invalid_error_format_exits_nonzero() {
    decy_cmd()
        .arg("oracle")
        .arg("query")
        .arg("--error")
        .arg("INVALID")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("E")
                .or(predicate::str::contains("error"))
                .or(predicate::str::contains("invalid")),
        );
}

// ============================================================================
// CLI CONTRACT TESTS: COMMON ERROR CODES
// ============================================================================

#[test]
fn cli_oracle_query_e0308_type_mismatch() {
    decy_cmd()
        .arg("oracle")
        .arg("query")
        .arg("--error")
        .arg("E0308")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("E0308")
                .or(predicate::str::contains("type"))
                .or(predicate::str::contains("mismatch")),
        );
}

#[test]
fn cli_oracle_query_e0133_unsafe_required() {
    decy_cmd()
        .arg("oracle")
        .arg("query")
        .arg("--error")
        .arg("E0133")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("E0133")
                .or(predicate::str::contains("unsafe"))
                .or(predicate::str::contains("block")),
        );
}

#[test]
fn cli_oracle_query_e0382_use_after_move() {
    decy_cmd()
        .arg("oracle")
        .arg("query")
        .arg("--error")
        .arg("E0382")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("E0382")
                .or(predicate::str::contains("move"))
                .or(predicate::str::contains("borrow")),
        );
}

#[test]
fn cli_oracle_query_e0499_multiple_mut_borrows() {
    decy_cmd()
        .arg("oracle")
        .arg("query")
        .arg("--error")
        .arg("E0499")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("E0499")
                .or(predicate::str::contains("mutable"))
                .or(predicate::str::contains("borrow")),
        );
}

#[test]
fn cli_oracle_query_e0597_lifetime() {
    decy_cmd()
        .arg("oracle")
        .arg("query")
        .arg("--error")
        .arg("E0597")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("E0597")
                .or(predicate::str::contains("lifetime"))
                .or(predicate::str::contains("live")),
        );
}

// ============================================================================
// CLI CONTRACT TESTS: OUTPUT FORMAT
// ============================================================================

#[test]
fn cli_oracle_query_shows_pattern_count() {
    decy_cmd()
        .arg("oracle")
        .arg("query")
        .arg("--error")
        .arg("E0308")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("pattern")
                .or(predicate::str::contains("Pattern"))
                .or(predicate::str::contains("found"))
                .or(predicate::str::contains("Found")),
        );
}

#[test]
fn cli_oracle_query_shows_fix_diff() {
    decy_cmd()
        .arg("oracle")
        .arg("query")
        .arg("--error")
        .arg("E0308")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("-")
                .or(predicate::str::contains("+"))
                .or(predicate::str::contains("fix"))
                .or(predicate::str::contains("Fix")),
        );
}

#[test]
fn cli_oracle_query_shows_description() {
    decy_cmd()
        .arg("oracle")
        .arg("query")
        .arg("--error")
        .arg("E0308")
        .assert()
        .success()
        .stdout(
            predicate::str::is_empty()
                .not()
                .and(predicate::str::contains("cast").or(predicate::str::contains("type"))),
        );
}

// ============================================================================
// CLI CONTRACT TESTS: NO PATTERNS CASE
// ============================================================================

#[test]
fn cli_oracle_query_unknown_error_shows_message() {
    decy_cmd()
        .arg("oracle")
        .arg("query")
        .arg("--error")
        .arg("E9999")
        .assert()
        .success() // Still succeeds, just shows "no patterns"
        .stdout(
            predicate::str::contains("No pattern")
                .or(predicate::str::contains("no pattern"))
                .or(predicate::str::contains("not found"))
                .or(predicate::str::contains("0 pattern")),
        );
}

// ============================================================================
// CLI CONTRACT TESTS: JSON OUTPUT
// ============================================================================

#[test]
fn cli_oracle_query_json_format() {
    decy_cmd()
        .arg("oracle")
        .arg("query")
        .arg("--error")
        .arg("E0308")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("{").and(predicate::str::contains("}")));
}

// ============================================================================
// CLI CONTRACT TESTS: HELP
// ============================================================================

#[test]
fn cli_oracle_query_help_shows_usage() {
    decy_cmd()
        .arg("oracle")
        .arg("query")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--error"))
        .stdout(predicate::str::contains("query"));
}

#[test]
fn cli_oracle_query_help_shows_context_option() {
    decy_cmd()
        .arg("oracle")
        .arg("query")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--context"));
}

#[test]
fn cli_oracle_query_help_shows_format_option() {
    decy_cmd()
        .arg("oracle")
        .arg("query")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--format"));
}
