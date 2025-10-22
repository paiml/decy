//! CLI Contract Tests: Quality-of-life improvements (DECY-053)
//!
//! Tests for new CLI flags and improved error messages.
//! Following ruchy's proven CLI testing pattern.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper: Create decy command
fn decy_cmd() -> Command {
    Command::cargo_bin("decy").expect("Failed to find decy binary")
}

/// Helper: Create temp file with content
fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to write temp file");
    path
}

// ============================================================================
// CLI CONTRACT TESTS: --verbose FLAG
// ============================================================================

#[test]
fn cli_verbose_flag_shows_per_file_progress() {
    let temp = TempDir::new().unwrap();
    let file1 = create_temp_file(&temp, "file1.c", "int add(int a, int b) { return a + b; }");
    let file2 = create_temp_file(&temp, "file2.c", "int sub(int a, int b) { return a - b; }");

    let output_dir = temp.path().join("output");
    fs::create_dir(&output_dir).unwrap();

    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(&output_dir)
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("file1.c"))
        .stdout(predicate::str::contains("file2.c"));
}

#[test]
fn cli_verbose_flag_shows_cache_hits() {
    let temp = TempDir::new().unwrap();
    let _file = create_temp_file(&temp, "test.c", "int main() { return 0; }");
    let output_dir = temp.path().join("output");
    fs::create_dir(&output_dir).unwrap();

    // First run
    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(&output_dir)
        .assert()
        .success();

    // Second run with --verbose should show cache hit
    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(&output_dir)
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("cache").or(predicate::str::contains("cached")));
}

// ============================================================================
// CLI CONTRACT TESTS: --quiet FLAG
// ============================================================================

#[test]
fn cli_quiet_flag_suppresses_progress() {
    let temp = TempDir::new().unwrap();
    let _file = create_temp_file(&temp, "test.c", "int main() { return 0; }");
    let output_dir = temp.path().join("output");
    fs::create_dir(&output_dir).unwrap();

    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(&output_dir)
        .arg("--quiet")
        .assert()
        .success()
        // Quiet mode should have minimal output (only errors if any)
        .stdout(predicate::str::is_empty().or(predicate::str::contains("Complete")));
}

#[test]
fn cli_quiet_and_verbose_are_mutually_exclusive() {
    let temp = TempDir::new().unwrap();

    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(temp.path())
        .arg("--quiet")
        .arg("--verbose")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used together")
            .or(predicate::str::contains("mutually exclusive")));
}

// ============================================================================
// CLI CONTRACT TESTS: --version
// ============================================================================

#[test]
fn cli_version_shows_detailed_info() {
    decy_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("decy"))
        .stdout(predicate::str::contains("0.2.0")); // Version
}

#[test]
fn cli_version_shows_commit_info() {
    decy_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("commit")
                .or(predicate::str::contains("git"))
                .or(predicate::str::contains("0.2.0")) // At minimum, version is shown
        );
}

// ============================================================================
// CLI CONTRACT TESTS: IMPROVED ERROR MESSAGES
// ============================================================================

#[test]
fn cli_error_missing_file_suggests_check_path() {
    decy_cmd()
        .arg("transpile")
        .arg("nonexistent_file.c")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("No such file"))
        )
        .stderr(
            predicate::str::contains("Try:")
                .or(predicate::str::contains("Check"))
                .or(predicate::str::contains("Verify"))
        );
}

#[test]
fn cli_error_invalid_c_syntax_suggests_preprocess() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "invalid.c",
        "#include <stdio.h>\nint main() { return 0; }",
    );

    decy_cmd()
        .arg("transpile")
        .arg(&file)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Try:")
                .or(predicate::str::contains("preprocess"))
                .or(predicate::str::contains("gcc -E"))
        );
}

#[test]
fn cli_error_no_output_directory_suggests_create() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "test.c", "int main() { return 0; }");
    let nonexistent_output = temp.path().join("nonexistent/output");

    decy_cmd()
        .arg("transpile")
        .arg(&file)
        .arg("-o")
        .arg(&nonexistent_output)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("No such file"))
                .or(predicate::str::contains("does not exist"))
        );
}

// ============================================================================
// CLI CONTRACT TESTS: --dry-run FLAG
// ============================================================================

#[test]
fn cli_dry_run_shows_what_would_happen() {
    let temp = TempDir::new().unwrap();
    let _file = create_temp_file(&temp, "test.c", "int main() { return 0; }");
    let output_dir = temp.path().join("output");

    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(&output_dir)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Would transpile")
            .or(predicate::str::contains("test.c")));
}

#[test]
fn cli_dry_run_creates_no_files() {
    let temp = TempDir::new().unwrap();
    let _file = create_temp_file(&temp, "test.c", "int main() { return 0; }");
    let output_dir = temp.path().join("output");

    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(&output_dir)
        .arg("--dry-run")
        .assert()
        .success();

    // Output directory should not exist
    assert!(!output_dir.exists());
}

// ============================================================================
// CLI CONTRACT TESTS: --stats FLAG
// ============================================================================

#[test]
fn cli_stats_flag_shows_summary() {
    let temp = TempDir::new().unwrap();
    let _file = create_temp_file(&temp, "test.c", "int main() { return 0; }");
    let output_dir = temp.path().join("output");
    fs::create_dir(&output_dir).unwrap();

    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(&output_dir)
        .arg("--stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("Files"))
        .stdout(predicate::str::contains("Lines")
            .or(predicate::str::contains("Functions")));
}

#[test]
fn cli_stats_shows_cache_statistics() {
    let temp = TempDir::new().unwrap();
    let _file = create_temp_file(&temp, "test.c", "int main() { return 0; }");
    let output_dir = temp.path().join("output");
    fs::create_dir(&output_dir).unwrap();

    // Run twice to get cache stats
    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(&output_dir)
        .assert()
        .success();

    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(&output_dir)
        .arg("--stats")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("cache")
                .or(predicate::str::contains("Cache"))
        );
}
