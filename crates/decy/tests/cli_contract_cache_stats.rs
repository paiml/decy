//! CLI contract tests for `decy cache-stats` command (DECY-050 RED phase)
//!
//! Tests the cache statistics command that shows hit/miss rates.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper: Create decy command
fn decy_cmd() -> Command {
    Command::cargo_bin("decy").expect("Failed to find decy binary")
}

/// Helper: Create temp C file
fn create_c_file(dir: &TempDir, name: &str, content: &str) {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to write C file");
}

#[test]
fn cli_cache_stats_exits_zero() {
    let temp = TempDir::new().unwrap();

    decy_cmd()
        .arg("cache-stats")
        .arg(temp.path())
        .assert()
        .success();
}

#[test]
fn cli_cache_stats_shows_metrics() {
    let temp = TempDir::new().unwrap();

    decy_cmd()
        .arg("cache-stats")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Cache")
                .or(predicate::str::contains("hits"))
                .or(predicate::str::contains("misses")),
        );
}

#[test]
fn cli_cache_stats_empty_cache() {
    let temp = TempDir::new().unwrap();

    decy_cmd()
        .arg("cache-stats")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("0")
                .or(predicate::str::contains("empty"))
                .or(predicate::str::contains("No cache")),
        );
}

#[test]
fn cli_cache_stats_after_transpilation() {
    let temp = TempDir::new().unwrap();
    create_c_file(&temp, "main.c", "int main() { return 0; }");

    let output_dir = temp.path().join("output");

    // First transpile with cache
    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(&output_dir)
        .arg("--cache")
        .assert()
        .success();

    // Then check stats
    decy_cmd()
        .arg("cache-stats")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("1").or(predicate::str::contains("files")));
}

#[test]
fn cli_cache_stats_missing_dir_fails() {
    let temp = TempDir::new().unwrap();
    let nonexistent = temp.path().join("missing");

    decy_cmd()
        .arg("cache-stats")
        .arg(&nonexistent)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such")));
}
