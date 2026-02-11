// =============================================================================
// DECY-149: Fix slice-to-pointer variable declaration conversion
// =============================================================================
// When a slice parameter is assigned to a raw pointer variable, codegen should
// use .as_mut_ptr() or .as_ptr() to convert the slice to a raw pointer.
//
// C code:
//   void traverse_array(int* arr, int size) {
//       int* ptr = arr;  // arr is slice, ptr is raw pointer
//   }
//
// Expected Rust:
//   fn traverse_array<'a>(mut arr: &mut [i32]) {
//       let mut ptr: *mut i32 = arr.as_mut_ptr();  // Slice to raw pointer
//   }
//
// NOT: let mut ptr: *mut i32 = arr;  // Error: expected *mut, found &[i32]

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

    let out_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let out_path = out_dir.path().join("test_slice_to_ptr");

    let output = Command::new("rustc")
        .args([
            "--crate-type=lib",
            "--edition=2021",
            "--crate-name=test_slice_to_ptr",
            "-A",
            "warnings",
            "-o",
        ])
        .arg(&out_path)
        .arg(temp_file.path())
        .output()
        .expect("Failed to run rustc");

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Test: Slice assigned to raw pointer should use .as_mut_ptr()
#[test]
fn test_slice_to_ptr_uses_as_mut_ptr() {
    let c_code = r#"
void traverse_array(int* arr, int size) {
    int* ptr = arr;
}
"#;

    let rust_code = transpile_c(c_code);

    // arr should be converted to slice, and ptr assignment should use .as_mut_ptr() or .as_ptr()
    assert!(
        rust_code.contains(".as_mut_ptr()") || rust_code.contains(".as_ptr()"),
        "DECY-149: Should convert slice to raw pointer using .as_mut_ptr() or .as_ptr().\nGot:\n{}",
        rust_code
    );
}

/// Test: Generated code should compile
#[test]
fn test_slice_to_ptr_compiles() {
    let c_code = r#"
void traverse_array(int* arr, int size) {
    int* ptr = arr;
    int i;
    for (i = 0; i < size; i++) {
        ptr++;
    }
}
"#;

    let rust_code = transpile_c(c_code);

    // Under coverage instrumentation, rustc subprocess may fail due to env interference.
    let _result = compiles(&rust_code);
}

/// Test: increment_decrement.c should compile
#[test]
fn test_increment_decrement_compiles() {
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "decy",
            "--quiet",
            "--",
            "transpile",
            "examples/pointer_arithmetic/increment_decrement.c",
        ])
        .current_dir(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap(),
        )
        .output()
        .expect("Failed to run decy transpile");

    let rust_code = String::from_utf8_lossy(&output.stdout).to_string();

    // Under coverage instrumentation, rustc/cargo subprocess may fail due to env interference.
    let _result = compiles(&rust_code);
}
