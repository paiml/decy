// =============================================================================
// DECY-146: Transform slice-to-pointer assignments for array parameters
// =============================================================================
// When array parameters become slices (&[T]), any assignment to a raw pointer
// should use .as_ptr() or .as_mut_ptr() instead of direct assignment.
//
// C pattern:
//   void traverse(int* arr, int size) {
//       int* ptr = arr;  // arr is array param
//   }
//
// Expected Rust:
//   fn traverse(arr: &[i32], size: i32) {
//       let ptr = arr.as_ptr();  // NOT: let ptr: *mut i32 = arr;
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
            "--crate-name=test_slice_ptr",
            "-A",
            "warnings",
            "-o",
            "/tmp/decy_slice_ptr_test",
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

/// Test: Array parameter assigned to local pointer should use .as_ptr()
#[test]
fn test_slice_to_ptr_assignment() {
    let c_code = r#"
void traverse_array(int* arr, int size) {
    int* ptr = arr;
}
"#;

    let rust_code = transpile_c(c_code);

    // Should use .as_ptr() or .as_mut_ptr() for slice-to-pointer
    assert!(
        rust_code.contains(".as_ptr()") || rust_code.contains(".as_mut_ptr()"),
        "DECY-146: Slice-to-pointer assignment should use .as_ptr() or .as_mut_ptr().\nGot:\n{}",
        rust_code
    );

    // Should NOT have direct assignment like `let ptr: *mut i32 = arr;`
    assert!(
        !rust_code.contains("*mut i32 = arr"),
        "DECY-146: Should not assign slice directly to raw pointer.\nGot:\n{}",
        rust_code
    );
}

/// Test: The generated code should compile
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

    match compiles(&rust_code) {
        Ok(()) => {}
        Err(e) => panic!(
            "DECY-146: Generated code should compile.\nCode:\n{}\nErrors:\n{}",
            rust_code, e
        ),
    }
}

/// Test: Pointer arithmetic with slice (arr + offset)
#[test]
fn test_slice_pointer_arithmetic() {
    let c_code = r#"
int sum_array(int* arr, int size) {
    int sum = 0;
    int* end = arr + size;
    while (arr < end) {
        sum += *arr;
        arr++;
    }
    return sum;
}
"#;

    let rust_code = transpile_c(c_code);

    // Should handle arr + size for slices
    // Either use .as_ptr().add() or index-based approach
    assert!(
        rust_code.contains(".as_ptr()")
            || rust_code.contains("as_mut_ptr")
            || rust_code.contains("[")
            || !rust_code.contains("arr + "),
        "DECY-146: Slice pointer arithmetic should not use direct addition.\nGot:\n{}",
        rust_code
    );

    // The generated code should compile
    match compiles(&rust_code) {
        Ok(()) => {}
        Err(e) => panic!(
            "DECY-146: Generated code with pointer arithmetic should compile.\nCode:\n{}\nErrors:\n{}",
            rust_code, e
        ),
    }
}

/// Test: String pointer assignment (char* start = str)
#[test]
fn test_string_ptr_assignment() {
    let c_code = r#"
int string_length(char* str) {
    char* start = str;
    while (*str != '\0') {
        str++;
    }
    return str - start;
}
"#;

    let rust_code = transpile_c(c_code);

    // The generated code should compile
    match compiles(&rust_code) {
        Ok(()) => {}
        Err(e) => panic!(
            "DECY-146: String length function should compile.\nCode:\n{}\nErrors:\n{}",
            rust_code, e
        ),
    }
}

/// Test: increment_decrement.c should compile (E2E validation)
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
        .current_dir("/home/noah/src/decy")
        .output()
        .expect("Failed to run decy transpile");

    let rust_code = String::from_utf8_lossy(&output.stdout).to_string();

    match compiles(&rust_code) {
        Ok(()) => {}
        Err(e) => panic!(
            "DECY-146: increment_decrement.c should compile.\nCode:\n{}\nErrors:\n{}",
            rust_code, e
        ),
    }
}

/// Test: real_world_patterns.c should compile (E2E validation)
#[test]
fn test_real_world_patterns_compiles() {
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "decy",
            "--quiet",
            "--",
            "transpile",
            "examples/pointer_arithmetic/real_world_patterns.c",
        ])
        .current_dir("/home/noah/src/decy")
        .output()
        .expect("Failed to run decy transpile");

    let rust_code = String::from_utf8_lossy(&output.stdout).to_string();

    match compiles(&rust_code) {
        Ok(()) => {}
        Err(e) => panic!(
            "DECY-146: real_world_patterns.c should compile.\nCode:\n{}\nErrors:\n{}",
            rust_code, e
        ),
    }
}
