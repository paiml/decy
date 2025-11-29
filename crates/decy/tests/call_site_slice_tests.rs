//! DECY-116: Call site slice transformation tests.
//!
//! When a function is transformed to take a slice parameter (instead of ptr + len),
//! the call sites must also be transformed to pass &array instead of (array, len).
//!
//! C: sum_array(nums, 5)  where sum_array takes (int* arr, int len)
//! Expected Rust: sum_array(&nums)  where sum_array takes (&[i32])

use decy_core::transpile;

/// Test that call site transforms array + len to slice reference.
///
/// C: sum_array(nums, 5)
/// Expected: sum_array(&nums)
#[test]
fn test_call_site_array_to_slice() {
    let c_code = r#"
        int sum_array(int* arr, int len) {
            int sum = 0;
            for (int i = 0; i < len; i++) {
                sum = sum + arr[i];
            }
            return sum;
        }

        int main() {
            int nums[5];
            nums[0] = 1;
            nums[1] = 2;
            nums[2] = 3;
            nums[3] = 4;
            nums[4] = 5;
            int result = sum_array(nums, 5);
            return result;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Function should take slice parameter
    assert!(
        result.contains("arr: &[i32]") || result.contains("arr: &mut [i32]"),
        "Function should take slice param\nGenerated:\n{}",
        result
    );

    // Call site should NOT have two arguments (array + length)
    assert!(
        !result.contains("sum_array(nums, 5)") && !result.contains("sum_array(mut nums, 5)"),
        "Call site should NOT have (array, len) syntax\nGenerated:\n{}",
        result
    );

    // Call site should pass slice reference
    assert!(
        result.contains("sum_array(&nums)") || result.contains("sum_array(&mut nums)"),
        "Call site should pass &nums or &mut nums\nGenerated:\n{}",
        result
    );
}

/// Test that code compiles after call site transformation.
#[test]
fn test_call_site_slice_compiles() {
    let c_code = r#"
        int sum_array(int* arr, int len) {
            int sum = 0;
            for (int i = 0; i < len; i++) {
                sum = sum + arr[i];
            }
            return sum;
        }

        int main() {
            int nums[5];
            int result = sum_array(nums, 5);
            return result;
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

/// Test multiple slice function calls.
#[test]
fn test_multiple_slice_calls() {
    let c_code = r#"
        int sum(int* arr, int n) {
            int total = 0;
            for (int i = 0; i < n; i++) {
                total = total + arr[i];
            }
            return total;
        }

        int max(int* arr, int n) {
            int m = arr[0];
            for (int i = 1; i < n; i++) {
                if (arr[i] > m) {
                    m = arr[i];
                }
            }
            return m;
        }

        int main() {
            int data[3];
            data[0] = 10;
            data[1] = 20;
            data[2] = 30;
            int s = sum(data, 3);
            int m = max(data, 3);
            return s + m;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Both calls should use slice syntax
    assert!(
        result.contains("sum(&data)") || result.contains("sum(&mut data)"),
        "sum call should pass slice\nGenerated:\n{}",
        result
    );
    assert!(
        result.contains("max(&data)") || result.contains("max(&mut data)"),
        "max call should pass slice\nGenerated:\n{}",
        result
    );
}
