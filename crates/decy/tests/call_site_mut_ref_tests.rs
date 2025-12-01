//! DECY-117: Call site reference mutability tests.
//!
//! When calling a function that takes &mut T, the call site must pass &mut x not &x.
//!
//! C: swap(&x, &y)  where swap takes (int *a, int *b)
//! Expected Rust: swap(&mut x, &mut y)  where swap takes (&mut i32, &mut i32)

use decy_core::transpile;

/// Test that call site uses &mut when function expects mutable reference.
///
/// C: swap(&x, &y)
/// Expected: swap(&mut x, &mut y)
#[test]
fn test_call_site_mut_ref_for_swap() {
    let c_code = r#"
        void swap(int *a, int *b) {
            int temp = *a;
            *a = *b;
            *b = temp;
        }

        int main() {
            int x = 5;
            int y = 10;
            swap(&x, &y);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Function signature should have &mut params (possibly with lifetime)
    assert!(
        result.contains("a: &mut i32")
            || result.contains("a: &mut")
            || result.contains("a: &'a mut i32"),
        "Function should take &mut params\nGenerated:\n{}",
        result
    );

    // Call site should use &mut x, &mut y
    assert!(
        result.contains("swap(&mut x, &mut y)"),
        "Call site should use &mut references\nGenerated:\n{}",
        result
    );

    // Should NOT have mismatched &x when function expects &mut
    assert!(
        !result.contains("swap(&x, &y)"),
        "Should NOT have immutable refs at call site when function expects &mut\nGenerated:\n{}",
        result
    );
}

/// Test that call site uses &mut for multiple parameters.
///
/// C: increment_both(&a, &a)
/// Expected: increment_both(&mut a, &mut a)
#[test]
fn test_call_site_mut_ref_multiple_params() {
    let c_code = r#"
        void increment_both(int *a, int *b) {
            *a = *a + 1;
            *b = *b + 1;
        }

        int main() {
            int x = 0;
            increment_both(&x, &x);
            return x;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Call site should use &mut x, &mut x
    assert!(
        result.contains("increment_both(&mut x, &mut x)"),
        "Call site should use &mut for both params\nGenerated:\n{}",
        result
    );
}

/// Test that read-only pointer parameters still use & (not &mut).
///
/// C: print_value(&x)  where print_value takes (const int *p)
/// Expected: print_value(&x)  (immutable reference)
///
/// Note: This requires detecting 'const' qualifiers in the parser/HIR.
/// TODO: DECY-118 - Implement const pointer detection for immutable references
#[test]
#[ignore = "DECY-118: Const pointer detection not yet implemented"]
fn test_call_site_immut_ref_for_readonly() {
    let c_code = r#"
        int get_value(const int *p) {
            return *p;
        }

        int main() {
            int x = 42;
            int result = get_value(&x);
            return result;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Function should take & param (not &mut) since it's const
    assert!(
        result.contains("p: &i32") || result.contains("p: &"),
        "Function should take immutable ref for const param\nGenerated:\n{}",
        result
    );

    // Call site should use &x (not &mut x)
    assert!(
        result.contains("get_value(&x)"),
        "Call site should use immutable ref for const param\nGenerated:\n{}",
        result
    );
}

/// Test that generated code compiles without type errors.
#[test]
fn test_call_site_mut_ref_compiles() {
    let c_code = r#"
        void swap(int *a, int *b) {
            int temp = *a;
            *a = *b;
            *b = temp;
        }

        int main() {
            int x = 5;
            int y = 10;
            swap(&x, &y);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Try to compile the generated code
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
        "Generated code should compile without errors:\n{}\n\nStderr:\n{}",
        result,
        stderr
    );
}
