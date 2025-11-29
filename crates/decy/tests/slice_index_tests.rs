//! DECY-113: Slice indexing with proper usize conversion tests.
//!
//! These tests verify that slice indices are properly cast to usize in generated Rust code.
//!
//! C: arr[i] where i is int
//! Expected: arr[i as usize]
//!
//! C: *(arr + i) where i is int
//! Expected: arr[i as usize]

use decy_core::transpile;

/// Test that array/slice indexing with i32 variable adds `as usize` cast.
///
/// C: sum = sum + arr[i];  // where i is int
/// Expected Rust: sum = sum + arr[i as usize];
#[test]
fn test_slice_index_i32_to_usize() {
    let c_code = r#"
        int sum(int* arr, int len) {
            int total = 0;
            for (int i = 0; i < len; i++) {
                total = total + arr[i];
            }
            return total;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should have `as usize` cast for the slice index
    assert!(
        result.contains("as usize]"),
        "Should cast index to usize\nGenerated:\n{}",
        result
    );

    // Should NOT have bare i32 index
    assert!(
        !result.contains("[i]") && !result.contains("[ i ]"),
        "Should NOT have bare i32 index\nGenerated:\n{}",
        result
    );
}

/// Test that pointer arithmetic is converted to proper slice indexing.
///
/// C: *(arr + i) where i is int
/// Expected Rust: arr[i as usize]
#[test]
fn test_pointer_arithmetic_to_slice_index() {
    let c_code = r#"
        int get_element(int* arr, int index) {
            return *(arr + index);
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should convert pointer arithmetic to slice indexing with usize
    assert!(
        result.contains("as usize]") || result.contains("[index as usize]"),
        "Should convert to slice indexing with usize cast\nGenerated:\n{}",
        result
    );

    // Should NOT have unsafe pointer arithmetic
    assert!(
        !result.contains(".add(") && !result.contains(".offset("),
        "Should NOT use unsafe pointer arithmetic\nGenerated:\n{}",
        result
    );
}

/// Test that loop bounds work correctly with slice.len().
///
/// C: for (int i = 0; i < len; i++)
/// Expected Rust: while i < arr.len() as i32 { ... }
///
/// Or alternatively:
/// Expected Rust: for i in 0..arr.len() { ... }
#[test]
fn test_loop_bound_with_slice_len() {
    let c_code = r#"
        int count_nonzero(int* arr, int len) {
            int count = 0;
            for (int i = 0; i < len; i++) {
                if (arr[i] != 0) {
                    count = count + 1;
                }
            }
            return count;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Loop should work with .len() comparison (either cast comparison or range-based)
    assert!(
        result.contains(".len()") || result.contains("0.."),
        "Should use .len() for loop bound\nGenerated:\n{}",
        result
    );
}

/// Test that the generated code actually compiles (integration test).
///
/// The transpiled code must pass rustc compilation.
#[test]
fn test_slice_index_code_compiles() {
    let c_code = r#"
        int sum(int* arr, int len) {
            int total = 0;
            for (int i = 0; i < len; i++) {
                total = total + arr[i];
            }
            return total;
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

/// Test multiple index accesses in same statement.
///
/// C: int avg = (arr[0] + arr[len-1]) / 2;
/// Expected: let avg = (arr[0_usize] + arr[(len-1) as usize]) / 2;
#[test]
fn test_multiple_index_accesses() {
    let c_code = r#"
        int first_last_avg(int* arr, int len) {
            int result = (arr[0] + arr[len - 1]) / 2;
            return result;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should have usize casts for both accesses
    // Note: literal 0 might not need cast, but len-1 definitely does
    assert!(
        result.contains("as usize") || result.contains("0usize"),
        "Should have usize casts for indices\nGenerated:\n{}",
        result
    );
}

/// Test that function calls with array arguments work correctly.
///
/// C: sum_array(nums, 5)  where nums is int[]
/// Expected: sum_array(&nums)  // pass as slice reference
///
/// Note: This is a complex call-site transformation that requires:
/// 1. Detecting when calling a function that takes slice params
/// 2. Transforming array args to slice references
/// 3. Removing length args
/// TODO: DECY-116 - Implement call site transformation for array-to-slice
#[test]
#[ignore = "DECY-116: Call site transformation not yet implemented"]
fn test_array_to_slice_call_site() {
    let c_code = r#"
        int sum(int* arr, int len) {
            int total = 0;
            for (int i = 0; i < len; i++) {
                total = total + arr[i];
            }
            return total;
        }

        int main() {
            int nums[5];
            int result = sum(nums, 5);
            return result;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // The call site should pass array as slice
    // This could be &nums or nums.as_slice() or similar
    // Key: should NOT have two arguments (array + length)
    assert!(
        !result.contains("sum(nums, 5)") && !result.contains("sum(mut nums, 5)"),
        "Should NOT have (array, length) at call site\nGenerated:\n{}",
        result
    );
}

/// Test 2D array indexing with proper casts.
///
/// C: matrix[i][j] where i, j are int
/// Expected: matrix[i as usize][j as usize]
#[test]
fn test_2d_array_index_casts() {
    let c_code = r#"
        int get_element(int matrix[3][3], int row, int col) {
            return matrix[row][col];
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should cast both indices to usize
    // Check for "as usize]" pattern appearing twice
    let usize_count = result.matches("as usize]").count();
    assert!(
        usize_count >= 2 || result.contains("row as usize") && result.contains("col as usize"),
        "Should cast both row and col to usize\nGenerated:\n{}",
        result
    );
}
