// =============================================================================
// DECY-150: Fix array index type mismatch in reverse-style calculations
// =============================================================================
// When calculating array indices like `buffer[size - 1 - i]`, the generated
// Rust code must use consistent types (usize throughout).
//
// C code:
//   void buffer_reverse(int* buffer, int size) {
//       buffer[size - 1 - i] = temp;
//   }
//
// Expected Rust (after BorrowGenerator replaces size with buffer.len()):
//   buffer[buffer.len() - 1 - i as usize] = temp;
//
// NOT:
//   buffer[(buffer.len() as i32 - 1) - i as usize] = temp;  // Type mismatch!

use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

/// Helper to transpile C code and return the generated Rust
fn transpile_c(c_code: &str) -> String {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(c_code.as_bytes())
        .expect("Failed to write C code");

    let output = Command::new("cargo")
        .args(["run", "-p", "decy", "--quiet", "--", "transpile"])
        .arg(temp_file.path())
        .output()
        .expect("Failed to run decy transpile");

    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Helper to check if generated Rust compiles
fn compiles(rust_code: &str) -> Result<(), String> {
    let mut temp_file = NamedTempFile::with_suffix(".rs").expect("Failed to create temp file");
    temp_file
        .write_all(rust_code.as_bytes())
        .expect("Failed to write Rust code");

    let output = Command::new("rustc")
        .args([
            "--crate-type=lib",
            "--edition=2021",
            "--crate-name=test_array_index",
            "-A",
            "warnings",
            "-o",
            "/tmp/decy_array_index_test",
        ])
        .arg(temp_file.path())
        .output()
        .expect("Failed to run rustc");

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Test: Array index with subtraction should use consistent usize types
#[test]
fn test_array_index_reverse_pattern_compiles() {
    let c_code = r#"
void buffer_reverse(int* buffer, int size) {
    int i;
    int temp;
    for (i = 0; i < size / 2; i = i + 1) {
        temp = buffer[i];
        buffer[i] = buffer[size - 1 - i];
        buffer[size - 1 - i] = temp;
    }
}
"#;

    let rust_code = transpile_c(c_code);

    match compiles(&rust_code) {
        Ok(()) => {}
        Err(e) => panic!(
            "DECY-150: Array index reverse pattern should compile.\nCode:\n{}\nErrors:\n{}",
            rust_code, e
        ),
    }
}

/// Test: Array index should not mix i32 and usize in calculations
#[test]
fn test_array_index_no_i32_minus_usize() {
    let c_code = r#"
void buffer_access(int* buffer, int size) {
    int i;
    for (i = 0; i < size; i = i + 1) {
        buffer[size - 1 - i] = 0;
    }
}
"#;

    let rust_code = transpile_c(c_code);

    // Should NOT have "as i32 - 1) - i as usize" pattern (mixing types)
    assert!(
        !rust_code.contains("as i32 - 1) - i as usize"),
        "DECY-150: Should not mix i32 and usize in index calculation.\nGot:\n{}",
        rust_code
    );
}

/// Test: buffer_ops.c should compile
#[test]
fn test_buffer_ops_compiles() {
    // Get workspace root from CARGO_MANIFEST_DIR
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let workspace_root = std::path::Path::new(&manifest_dir)
        .parent()
        .expect("Failed to get parent")
        .parent()
        .expect("Failed to get workspace root");

    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "decy",
            "--quiet",
            "--",
            "transpile",
            "examples/real-world/buffer_ops.c",
        ])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to run decy transpile");

    let rust_code = String::from_utf8_lossy(&output.stdout).to_string();

    match compiles(&rust_code) {
        Ok(()) => {}
        Err(e) => panic!(
            "DECY-150: buffer_ops.c should compile.\nCode:\n{}\nErrors:\n{}",
            rust_code, e
        ),
    }
}
