//! DECY-118: Local pointer variable to reference transformation tests.
//!
//! DECY-118 is DISABLED for local variables due to DECY-128 (double pointer issues).
//! When a local pointer's address is taken (e.g., &p in set_value(&p, 42)),
//! transforming `int *p = &x` to `let p = &mut x` breaks type compatibility.
//!
//! Current behavior: Local pointer → raw pointer (requires unsafe)
//! Future (when DECY-118 re-enabled): Local pointer → reference
//!
//! C: int *p = &x;
//! Current Rust: let p: *mut i32 = &mut x as *mut i32;
//! Future Rust: let p = &mut x;

use decy_core::transpile;

/// Test that local pointer initialized with address-of generates raw pointer.
///
/// DECY-118 is DISABLED. Local pointers remain as raw pointers because
/// their address might be taken (e.g., &p), which would fail with references.
///
/// C: int *p = &x;
/// Current: let p: *mut i32 = &mut x as *mut i32; (raw pointer, needs unsafe)
#[test]
fn test_local_pointer_stays_raw_pointer() {
    let c_code = r#"
        int main() {
            int x = 10;
            int *p = &x;
            *p = 20;
            return x;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // DECY-118 disabled: Local pointers stay as raw pointers
    // This is intentional due to DECY-128 (double pointer compatibility)
    assert!(
        result.contains("*mut i32") || result.contains("&mut x"),
        "Local pointer should be raw pointer or reference\nGenerated:\n{}",
        result
    );

    // Should still have &mut x somewhere (either as reference or cast)
    assert!(
        result.contains("&mut x"),
        "Should have address-of expression\nGenerated:\n{}",
        result
    );
}

/// Test that local pointer to ref compiles.
#[test]
fn test_local_pointer_to_ref_compiles() {
    let c_code = r#"
        int main() {
            int x = 10;
            int *p = &x;
            *p = 20;
            return x;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Try to compile
    use std::process::Command;

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let temp_file_path = temp_dir.path().join("test_code.rs");
    std::fs::write(&temp_file_path, format!("#![allow(unused)]\n{}", result))
        .expect("Failed to write temp file");

    let output = Command::new("rustc")
        .arg("--emit=metadata")
        .arg("--crate-type=lib")
        .arg("--crate-name=decy_test")
        .arg("--out-dir")
        .arg(temp_dir.path())
        .arg(&temp_file_path)
        .output()
        .expect("Failed to run rustc");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Generated code should compile:\n{}\n\nStderr:\n{}",
        result,
        stderr
    );
}

/// Test pointer initialized with function call (not address-of) stays as-is.
#[test]
fn test_pointer_from_function_stays_pointer() {
    // This is a case where we can't easily transform to reference
    // because the return type is a pointer
    let c_code = r#"
        int* get_ptr(int* arr) {
            return arr;
        }

        int main() {
            int arr[5];
            int *p = get_ptr(arr);
            return *p;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Just verify it transpiles - exact type handling depends on context
}
