//! Integration tests for the Decy CLI tool
//!
//! Tests the complete transpilation pipeline on real C code examples.

use std::fs;
use std::path::Path;
use std::process::Command;

/// Test that the CLI can transpile a minimal C program
#[test]
fn test_transpile_minimal_c_program() {
    // Given: A minimal C program
    let example_path = "../../examples/simple/minimal.c";
    assert!(
        Path::new(example_path).exists(),
        "Example file {} should exist",
        example_path
    );

    let c_code = fs::read_to_string(example_path).expect("Failed to read example file");
    assert!(
        c_code.contains("int main()"),
        "Should contain main function"
    );

    // When: We transpile it using decy-core directly (CLI will use this)
    let result = decy_core::transpile(&c_code);

    // Then: Transpilation should succeed
    assert!(
        result.is_ok(),
        "Transpilation should succeed, got error: {:?}",
        result.err()
    );

    let rust_code = result.unwrap();

    // And: Generated Rust code should contain expected elements
    assert!(
        rust_code.contains("fn main"),
        "Should contain main function, got: {}",
        rust_code
    );

    // DECY-AUDIT-001 / REC-001: Compile the generated Rust code to verify it's valid
    // Write to temp file and compile with rustc
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("test_minimal.rs");
    fs::write(&temp_file, &rust_code).expect("Failed to write temp file");

    let compile_output = Command::new("rustc")
        .arg(&temp_file)
        .arg("--crate-type")
        .arg("bin")
        .arg("-o")
        .arg(temp_dir.join("test_minimal"))
        .output()
        .expect("Failed to run rustc");

    // Clean up temp files
    let _ = fs::remove_file(&temp_file);
    let _ = fs::remove_file(temp_dir.join("test_minimal"));

    assert!(
        compile_output.status.success(),
        "Generated Rust code should compile successfully. Compilation errors:\n{}",
        String::from_utf8_lossy(&compile_output.stderr)
    );
}

/// Test that the CLI can transpile simple arithmetic functions
#[test]
fn test_transpile_arithmetic_functions() {
    // Given: C code with arithmetic functions
    let example_path = "../../examples/simple/arithmetic.c";
    assert!(
        Path::new(example_path).exists(),
        "Example file {} should exist",
        example_path
    );

    let c_code = fs::read_to_string(example_path).expect("Failed to read example file");
    assert!(c_code.contains("int add"), "Should contain add function");
    assert!(
        c_code.contains("int multiply"),
        "Should contain multiply function"
    );

    // When: We transpile it
    let result = decy_core::transpile(&c_code);

    // Then: Transpilation should succeed
    assert!(
        result.is_ok(),
        "Transpilation should succeed, got error: {:?}",
        result.err()
    );

    let rust_code = result.unwrap();

    // And: Generated Rust code should contain both functions
    assert!(
        rust_code.contains("fn add"),
        "Should contain add function, got: {}",
        rust_code
    );
    assert!(
        rust_code.contains("fn multiply"),
        "Should contain multiply function, got: {}",
        rust_code
    );

    // And: Should use Rust types
    assert!(
        rust_code.contains("i32"),
        "Should use i32 type, got: {}",
        rust_code
    );

    // DECY-AUDIT-002 / REC-001: Compile as library (no main function)
    // Write to temp file and compile with rustc
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("test_arithmetic.rs");
    fs::write(&temp_file, &rust_code).expect("Failed to write temp file");

    let compile_output = Command::new("rustc")
        .arg(&temp_file)
        .arg("--crate-type")
        .arg("lib")
        .arg("-o")
        .arg(temp_dir.join("libtest_arithmetic.rlib"))
        .output()
        .expect("Failed to run rustc");

    // Clean up temp files
    let _ = fs::remove_file(&temp_file);
    let _ = fs::remove_file(temp_dir.join("libtest_arithmetic.rlib"));

    assert!(
        compile_output.status.success(),
        "Generated Rust code should compile as library. Compilation errors:\n{}",
        String::from_utf8_lossy(&compile_output.stderr)
    );
}

/// Test that the CLI binary can be invoked with --help
#[test]
fn test_cli_help() {
    // When: CLI is run with --help
    let output = Command::new("cargo")
        .args(["run", "-p", "decy", "--", "--help"])
        .output()
        .expect("Failed to run CLI");

    // Then: Should succeed and show help
    assert!(output.status.success(), "CLI should run successfully");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("decy") || stdout.contains("Usage"),
        "Should show help text, got: {}",
        stdout
    );
}

/// Test that the CLI can transpile a C file
#[test]
fn test_cli_transpile_file() {
    // Given: A simple C file
    let example_path = "../../examples/simple/minimal.c";

    // When: CLI transpiles it
    let output = Command::new("cargo")
        .args(["run", "-p", "decy", "--", "transpile", example_path])
        .output()
        .expect("Failed to run CLI");

    // Then: Should succeed
    assert!(
        output.status.success(),
        "CLI should transpile successfully, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // And: Should output Rust code
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("fn main"),
        "Should contain Rust main function, got: {}",
        stdout
    );
}

/// Test transpiling a file with control flow
#[test]
fn test_transpile_control_flow() {
    let c_code = r#"
        int max(int a, int b) {
            if (a > b) {
                return a;
            } else {
                return b;
            }
        }
    "#;

    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "Should transpile control flow");

    let rust_code = result.unwrap();
    assert!(rust_code.contains("fn max"), "Should contain max function");
    assert!(rust_code.contains("if"), "Should contain if statement");
}
