//! CLI contract tests for `decy transpile-project` command (DECY-050 RED phase)
//!
//! Following the ruchy/CLAUDE.md pattern for comprehensive CLI testing.
//!
//! Tests verify:
//! - Exit codes (0 = success, non-zero = failure)
//! - stdout/stderr separation
//! - Error messages include filenames
//! - Edge cases (empty dir, missing files, etc.)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper: Create decy command
fn decy_cmd() -> Command {
    Command::cargo_bin("decy").expect("Failed to find decy binary")
}

/// Helper: Create temp C file with content
fn create_c_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to write C file");
    path
}

// ============================================================================
// CLI CONTRACT TESTS: EXIT CODES
// ============================================================================

#[test]
fn cli_transpile_project_valid_dir_exits_zero() {
    let temp = TempDir::new().unwrap();
    create_c_file(&temp, "main.c", "int main() { return 0; }");

    let output_dir = temp.path().join("output");

    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(&output_dir)
        .assert()
        .success(); // Exit code 0
}

#[test]
fn cli_transpile_project_missing_dir_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let nonexistent = temp.path().join("does_not_exist");
    let output_dir = temp.path().join("output");

    decy_cmd()
        .arg("transpile-project")
        .arg(&nonexistent)
        .arg("-o")
        .arg(&output_dir)
        .assert()
        .failure() // Exit code != 0
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("No such"))
                .or(predicate::str::contains("does not exist")),
        );
}

#[test]
fn cli_transpile_project_no_output_dir_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    create_c_file(&temp, "main.c", "int main() { return 0; }");

    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        // Missing -o flag
        .assert()
        .failure();
}

// ============================================================================
// CLI CONTRACT TESTS: OUTPUT DIRECTORY CREATION
// ============================================================================

#[test]
fn cli_transpile_project_creates_output_dir() {
    let temp = TempDir::new().unwrap();
    create_c_file(&temp, "test.c", "int add(int a, int b) { return a + b; }");

    let output_dir = temp.path().join("rust_output");

    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(&output_dir)
        .assert()
        .success();

    assert!(output_dir.exists(), "Output directory should be created");
}

#[test]
fn cli_transpile_project_preserves_directory_structure() {
    let temp = TempDir::new().unwrap();

    // Create nested structure
    let src_dir = temp.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("main.c"), "int main() { return 0; }").unwrap();

    let lib_dir = temp.path().join("lib");
    fs::create_dir(&lib_dir).unwrap();
    fs::write(lib_dir.join("utils.c"), "int util() { return 1; }").unwrap();

    let output_dir = temp.path().join("output");

    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(&output_dir)
        .assert()
        .success();

    // Verify structure preserved
    assert!(output_dir.join("src").exists(), "src/ should exist in output");
    assert!(output_dir.join("lib").exists(), "lib/ should exist in output");
}

// ============================================================================
// CLI CONTRACT TESTS: FILE PROCESSING
// ============================================================================

#[test]
fn cli_transpile_project_generates_rust_files() {
    let temp = TempDir::new().unwrap();
    create_c_file(&temp, "test.c", "int value() { return 42; }");

    let output_dir = temp.path().join("output");

    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(&output_dir)
        .assert()
        .success();

    let rust_file = output_dir.join("test.rs");
    assert!(rust_file.exists(), "Should generate test.rs");

    let rust_content = fs::read_to_string(&rust_file).unwrap();
    assert!(rust_content.contains("fn value"), "Should contain Rust function");
}

#[test]
fn cli_transpile_project_with_multiple_files() {
    let temp = TempDir::new().unwrap();
    create_c_file(&temp, "file1.c", "int a() { return 1; }");
    create_c_file(&temp, "file2.c", "int b() { return 2; }");
    create_c_file(&temp, "file3.c", "int c() { return 3; }");

    let output_dir = temp.path().join("output");

    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(&output_dir)
        .assert()
        .success();

    assert!(output_dir.join("file1.rs").exists());
    assert!(output_dir.join("file2.rs").exists());
    assert!(output_dir.join("file3.rs").exists());
}

// ============================================================================
// CLI CONTRACT TESTS: CACHING
// ============================================================================

#[test]
fn cli_transpile_project_with_cache_flag() {
    let temp = TempDir::new().unwrap();
    create_c_file(&temp, "main.c", "int main() { return 0; }");

    let output_dir = temp.path().join("output");

    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(&output_dir)
        .arg("--cache")
        .assert()
        .success();
}

#[test]
fn cli_transpile_project_without_cache_flag() {
    let temp = TempDir::new().unwrap();
    create_c_file(&temp, "main.c", "int main() { return 0; }");

    let output_dir = temp.path().join("output");

    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(&output_dir)
        .arg("--no-cache")
        .assert()
        .success();
}

// ============================================================================
// CLI CONTRACT TESTS: EDGE CASES
// ============================================================================

#[test]
fn cli_transpile_project_empty_directory() {
    let temp = TempDir::new().unwrap();
    let output_dir = temp.path().join("output");

    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(&output_dir)
        .assert()
        .success() // Should succeed with no files to transpile
        .stdout(predicate::str::contains("0 files").or(predicate::str::contains("No C files")));
}

#[test]
fn cli_transpile_project_ignores_non_c_files() {
    let temp = TempDir::new().unwrap();
    create_c_file(&temp, "test.c", "int test() { return 1; }");
    create_c_file(&temp, "readme.txt", "This is not C code");
    create_c_file(&temp, "data.json", "{}");

    let output_dir = temp.path().join("output");

    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(&output_dir)
        .assert()
        .success();

    // Only test.c should be transpiled
    assert!(output_dir.join("test.rs").exists());
    assert!(!output_dir.join("readme.rs").exists());
    assert!(!output_dir.join("data.rs").exists());
}

#[test]
fn cli_transpile_project_with_syntax_errors() {
    let temp = TempDir::new().unwrap();
    create_c_file(&temp, "bad.c", "int main( { }"); // Malformed C

    let output_dir = temp.path().join("output");

    decy_cmd()
        .arg("transpile-project")
        .arg(temp.path())
        .arg("-o")
        .arg(&output_dir)
        .assert()
        .failure() // Should exit non-zero
        .stderr(
            predicate::str::contains("bad.c")
                .and(predicate::str::contains("error").or(predicate::str::contains("failed"))),
        );
}
