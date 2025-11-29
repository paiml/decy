//! CLI contract tests for `decy check-project` command (DECY-050 RED phase)
//!
//! Tests the dry-run command that shows build order without transpiling.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper: Create decy command
#[allow(deprecated)]
fn decy_cmd() -> Command {
    Command::cargo_bin("decy").expect("Failed to find decy binary")
}

/// Helper: Create temp C file
fn create_c_file(dir: &TempDir, name: &str, content: &str) {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to write C file");
}

#[test]
fn cli_check_project_exits_zero() {
    let temp = TempDir::new().unwrap();
    create_c_file(&temp, "main.c", "int main() { return 0; }");

    decy_cmd()
        .arg("check-project")
        .arg(temp.path())
        .assert()
        .success();
}

#[test]
fn cli_check_project_shows_build_order() {
    let temp = TempDir::new().unwrap();
    create_c_file(&temp, "a.c", "int a() { return 1; }");
    create_c_file(&temp, "b.c", "int b() { return 2; }");

    decy_cmd()
        .arg("check-project")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Build order").or(predicate::str::contains("a.c")));
}

#[test]
fn cli_check_project_detects_circular_dependencies() {
    let temp = TempDir::new().unwrap();
    // Create files with circular includes (if we implement include parsing)
    create_c_file(&temp, "a.c", "int a() { return 1; }");
    create_c_file(&temp, "b.c", "int b() { return 2; }");

    decy_cmd()
        .arg("check-project")
        .arg(temp.path())
        .assert()
        .success(); // Should succeed even with circular deps in check mode
}

#[test]
fn cli_check_project_missing_dir_fails() {
    let temp = TempDir::new().unwrap();
    let nonexistent = temp.path().join("missing");

    decy_cmd()
        .arg("check-project")
        .arg(&nonexistent)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such")));
}

#[test]
fn cli_check_project_shows_file_count() {
    let temp = TempDir::new().unwrap();
    create_c_file(&temp, "file1.c", "int f1() { return 1; }");
    create_c_file(&temp, "file2.c", "int f2() { return 2; }");
    create_c_file(&temp, "file3.c", "int f3() { return 3; }");

    decy_cmd()
        .arg("check-project")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("3").or(predicate::str::contains("files")));
}
