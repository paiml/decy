//! DECY-118: Local pointer variable to reference transformation tests.
//!
//! When a local variable is declared as a pointer and initialized with &x,
//! it should be transformed to a reference type.
//!
//! C: int *p = &x;
//! Expected Rust: let p = &mut x;  (NOT: let p: *mut i32 = &x;)

use decy_core::transpile;

/// Test that local pointer initialized with address-of becomes reference.
///
/// C: int *p = &x;
/// Expected: let p = &mut x;  or  let p: &mut i32 = &mut x;
#[test]
fn test_local_pointer_to_ref() {
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

    // Should NOT have raw pointer assignment
    assert!(
        !result.contains(": *mut i32 = &"),
        "Should NOT assign &x to *mut i32\nGenerated:\n{}",
        result
    );

    // Should use reference type
    assert!(
        result.contains("&mut x") || result.contains("& mut x"),
        "Should use &mut reference\nGenerated:\n{}",
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
