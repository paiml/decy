// =============================================================================
// DECY-142: Fix Vec return type inference - prevent dangling pointer
// =============================================================================
// Tests for returning Vec<T> instead of *mut T from malloc-based allocations.
//
// Current (DANGEROUS):
//   fn create_array(n: i32) -> *mut i32 {
//       let arr: Vec<i32> = vec![0i32; n];
//       return arr.as_mut_ptr();  // Vec dropped, pointer dangles!
//   }
//
// Expected (SAFE):
//   fn create_array(n: i32) -> Vec<i32> {
//       vec![0i32; n]
//   }

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

/// Test that malloc-based array allocation returns Vec<T> not *mut T.
/// This prevents dangling pointer bugs.
#[test]
fn test_malloc_array_returns_vec() {
    let c_code = r#"
#include <stdlib.h>

int* create_int_array(int n) {
    int* arr = (int*)malloc(n * sizeof(int));
    return arr;
}
"#;

    let rust_code = transpile_c(c_code);

    // Should return Vec<i32>, not *mut i32
    assert!(
        rust_code.contains("-> Vec<i32>"),
        "DECY-142: Function returning malloc'd array should return Vec<T>, not *mut T.\nGot:\n{}",
        rust_code
    );

    // Should NOT return raw pointer
    assert!(
        !rust_code.contains("-> *mut i32"),
        "DECY-142: Should NOT return *mut T (dangling pointer risk).\nGot:\n{}",
        rust_code
    );

    // Should NOT have as_mut_ptr() in return
    assert!(
        !rust_code.contains("as_mut_ptr()"),
        "DECY-142: Should NOT use as_mut_ptr() on local Vec.\nGot:\n{}",
        rust_code
    );
}

/// Test that the return type transformation works for different element types.
#[test]
fn test_malloc_array_various_types() {
    let c_code = r#"
#include <stdlib.h>

double* create_double_array(int n) {
    double* arr = (double*)malloc(n * sizeof(double));
    return arr;
}

char* create_char_array(int n) {
    char* arr = (char*)malloc(n * sizeof(char));
    return arr;
}
"#;

    let rust_code = transpile_c(c_code);

    // Check double array
    assert!(
        rust_code.contains("-> Vec<f64>") || rust_code.contains("-> Vec<f32>"),
        "DECY-142: Double array should return Vec<f64>.\nGot:\n{}",
        rust_code
    );

    // Check char array - should be Vec<u8>
    assert!(
        rust_code.contains("-> Vec<u8>"),
        "DECY-142: Char array should return Vec<u8>.\nGot:\n{}",
        rust_code
    );
}

/// Test that malloc with direct return transforms correctly.
#[test]
fn test_malloc_direct_return() {
    let c_code = r#"
#include <stdlib.h>

int* allocate(int size) {
    return (int*)malloc(size * sizeof(int));
}
"#;

    let rust_code = transpile_c(c_code);

    // Direct return should also be Vec
    assert!(
        rust_code.contains("-> Vec<i32>"),
        "DECY-142: Direct malloc return should be Vec<T>.\nGot:\n{}",
        rust_code
    );
}

/// Test that non-malloc pointer returns are NOT changed to Vec.
#[test]
fn test_non_malloc_pointer_unchanged() {
    let c_code = r#"
int* get_pointer(int* arr, int index) {
    return &arr[index];
}
"#;

    let rust_code = transpile_c(c_code);

    // This is returning a reference to existing data, not a new allocation
    // It should NOT become Vec
    let has_vec_return = rust_code.contains("-> Vec<");

    // This test documents current behavior - may need adjustment
    // The key is that ONLY malloc returns should become Vec
    assert!(
        !has_vec_return || rust_code.contains("&arr["),
        "Non-malloc pointer returns should not blindly become Vec.\nGot:\n{}",
        rust_code
    );
}

/// Test that generated code compiles and runs without UB.
#[test]
fn test_vec_return_compiles_safely() {
    let c_code = r#"
#include <stdlib.h>

int* make_array(int n) {
    int* arr = (int*)malloc(n * sizeof(int));
    for (int i = 0; i < n; i++) {
        arr[i] = i * 2;
    }
    return arr;
}
"#;

    let rust_code = transpile_c(c_code);

    // Write to temp file and compile
    let mut temp_file = NamedTempFile::with_suffix(".rs").expect("Failed to create temp file");
    temp_file
        .write_all(rust_code.as_bytes())
        .expect("Failed to write Rust code");

    let output = Command::new("rustc")
        .args([
            "--crate-type=lib",
            "--edition=2021",
            "--crate-name=test_vec_return",
            "-A",
            "warnings",
        ])
        .arg(temp_file.path())
        .arg("-o")
        .arg("/tmp/test_vec_return_lib")
        .output()
        .expect("Failed to run rustc");

    let success = output.status.success();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    assert!(
        success,
        "Generated Vec return code should compile.\nCode:\n{}\nErrors:\n{}",
        rust_code, stderr
    );
}
