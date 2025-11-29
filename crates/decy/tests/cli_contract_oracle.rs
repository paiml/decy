//! CLI contract tests for `decy oracle` commands (DECY-104)
//!
//! Tests the oracle management commands: seed, stats, retire, validate.
//!
//! Note: These tests require the `oracle` feature to be enabled.
//! Without the feature, commands should fail gracefully with a helpful message.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper: Create decy command
#[allow(deprecated)]
fn decy_cmd() -> Command {
    Command::cargo_bin("decy").expect("Failed to find decy binary")
}

// ============================================================================
// ORACLE STATS COMMAND
// ============================================================================

#[test]
fn cli_oracle_stats_exits_zero() {
    decy_cmd().arg("oracle").arg("stats").assert().success();
}

#[test]
fn cli_oracle_stats_default_format_markdown() {
    decy_cmd()
        .arg("oracle")
        .arg("stats")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Oracle CI Report").or(predicate::str::contains("oracle")),
        );
}

#[test]
fn cli_oracle_stats_json_format() {
    decy_cmd()
        .arg("oracle")
        .arg("stats")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("{").and(predicate::str::contains("}")));
}

#[test]
fn cli_oracle_stats_prometheus_format() {
    decy_cmd()
        .arg("oracle")
        .arg("stats")
        .arg("--format")
        .arg("prometheus")
        .assert()
        .success()
        .stdout(predicate::str::contains("decy_oracle"));
}

// ============================================================================
// ORACLE RETIRE COMMAND
// ============================================================================

#[test]
fn cli_oracle_retire_dry_run_exits_zero() {
    decy_cmd()
        .arg("oracle")
        .arg("retire")
        .arg("--dry-run")
        .assert()
        .success();
}

#[test]
fn cli_oracle_retire_dry_run_shows_thresholds() {
    decy_cmd()
        .arg("oracle")
        .arg("retire")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("DRY RUN"))
        .stdout(predicate::str::contains("Min uses"))
        .stdout(predicate::str::contains("success rate"));
}

#[test]
fn cli_oracle_retire_without_dry_run() {
    decy_cmd()
        .arg("oracle")
        .arg("retire")
        .assert()
        .success()
        .stdout(predicate::str::contains("Pattern Retirement Analysis"));
}

// ============================================================================
// ORACLE SEED COMMAND
// ============================================================================

#[test]
fn cli_oracle_seed_missing_file_fails() {
    let temp = TempDir::new().unwrap();
    let nonexistent = temp.path().join("missing.apr");

    decy_cmd()
        .arg("oracle")
        .arg("seed")
        .arg("--from")
        .arg(&nonexistent)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn cli_oracle_seed_requires_from_arg() {
    decy_cmd()
        .arg("oracle")
        .arg("seed")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ============================================================================
// ORACLE VALIDATE COMMAND
// ============================================================================

#[test]
fn cli_oracle_validate_missing_dir_fails() {
    let temp = TempDir::new().unwrap();
    let nonexistent = temp.path().join("missing_corpus");

    decy_cmd()
        .arg("oracle")
        .arg("validate")
        .arg(&nonexistent)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn cli_oracle_validate_empty_corpus() {
    let temp = TempDir::new().unwrap();

    decy_cmd()
        .arg("oracle")
        .arg("validate")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Found 0 C files"));
}

#[test]
fn cli_oracle_validate_with_c_files() {
    let temp = TempDir::new().unwrap();
    std::fs::write(temp.path().join("test.c"), "int main() { return 0; }").unwrap();

    decy_cmd()
        .arg("oracle")
        .arg("validate")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Found 1 C files"))
        .stdout(predicate::str::contains("Validation Results"));
}

#[test]
fn cli_oracle_validate_requires_corpus_arg() {
    decy_cmd()
        .arg("oracle")
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ============================================================================
// ORACLE HELP
// ============================================================================

#[test]
fn cli_oracle_help_shows_subcommands() {
    decy_cmd()
        .arg("oracle")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("seed"))
        .stdout(predicate::str::contains("stats"))
        .stdout(predicate::str::contains("retire"))
        .stdout(predicate::str::contains("validate"));
}
