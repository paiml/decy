//! Integration tests for array parameter → slice signature transformation (DECY-072).
//!
//! Tests the complete pipeline: C parsing → ownership inference → signature transformation → codegen.
//! Verifies that C array parameters are transformed to safe Rust slice parameters.

use decy_core::transpile;

/// Test basic array parameter transformation: (int* arr, int len) → &[i32]
///
/// C: void process(int* arr, int len) { }
/// Rust: fn process(arr: &[i32]) { }
#[test]
// DECY-072 GREEN: Test now active
fn test_transform_array_parameter_to_slice() {
    let c_code = r#"
        void process(int* arr, int len) {
            // Empty function for signature testing
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should transform to slice parameter
    assert!(
        result.contains("arr: &[i32]"),
        "Should transform (int* arr, int len) to arr: &[i32]\nGenerated:\n{}",
        result
    );

    // Length parameter should be removed (slice includes length)
    assert!(
        !result.contains("len: i32") && !result.contains("len: usize"),
        "Should remove redundant length parameter\nGenerated:\n{}",
        result
    );

    // Should NOT use unsafe
    assert!(
        !result.contains("unsafe"),
        "Should generate safe code\nGenerated:\n{}",
        result
    );
}

/// Test mutable array parameter: (int* arr, int len) with mutations → &mut [i32]
///
/// C: void fill(int* arr, int len) { arr[0] = 1; }
/// Rust: fn fill(arr: &mut [i32]) { arr[0] = 1; }
#[test]
// DECY-072 GREEN: Test now active
fn test_transform_mutable_array_parameter() {
    let c_code = r#"
        void fill(int* arr, int len) {
            arr[0] = 1;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should transform to mutable slice parameter
    assert!(
        result.contains("arr: &mut [i32]"),
        "Should transform to arr: &mut [i32] for mutable access\nGenerated:\n{}",
        result
    );

    // Should NOT use unsafe
    assert!(
        !result.contains("unsafe"),
        "Should generate safe code\nGenerated:\n{}",
        result
    );
}

/// Test array parameter with length usage in body: len → arr.len()
///
/// C: int sum(int* arr, int len) { for(int i=0; i<len; i++) { } }
/// Rust: fn sum(arr: &[i32]) -> i32 { for i in 0..arr.len() { } }
#[test]
// DECY-072 GREEN: Test now active
fn test_transform_length_usage_in_body() {
    let c_code = r#"
        int sum(int* arr, int len) {
            int total = 0;
            for (int i = 0; i < len; i++) {
                total += arr[i];
            }
            return total;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should transform to slice parameter
    assert!(
        result.contains("arr: &[i32]"),
        "Should use slice parameter\nGenerated:\n{}",
        result
    );

    // Length usage should be transformed to arr.len()
    assert!(
        result.contains("arr.len()"),
        "Should use arr.len() instead of len parameter\nGenerated:\n{}",
        result
    );

    // Should NOT use unsafe
    assert!(
        !result.contains("unsafe"),
        "Should generate safe code\nGenerated:\n{}",
        result
    );
}

/// Test char array parameter: (char* buf, int size) → &[u8]
///
/// C: void process_buffer(char* buf, int size) { }
/// Rust: fn process_buffer(buf: &[u8]) { }
#[test]
// DECY-072 GREEN: Test now active
fn test_transform_char_array_parameter() {
    let c_code = r#"
        void process_buffer(char* buf, int size) {
            buf[0] = 65;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should transform to mutable u8 slice
    assert!(
        result.contains("buf: &mut [u8]"),
        "Should transform char* to &mut [u8]\nGenerated:\n{}",
        result
    );

    // Should NOT use unsafe
    assert!(
        !result.contains("unsafe"),
        "Should generate safe code\nGenerated:\n{}",
        result
    );
}

/// Test multiple array parameters
///
/// C: void merge(int* arr1, int len1, int* arr2, int len2) { }
/// Rust: fn merge(arr1: &[i32], arr2: &[i32]) { }
#[test]
// DECY-072 GREEN: Test now active
fn test_transform_multiple_array_parameters() {
    let c_code = r#"
        void merge(int* arr1, int len1, int* arr2, int len2) {
            // Empty for signature testing
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should transform both array parameters
    assert!(
        result.contains("arr1: &[i32]"),
        "Should transform arr1 to slice\nGenerated:\n{}",
        result
    );
    assert!(
        result.contains("arr2: &[i32]"),
        "Should transform arr2 to slice\nGenerated:\n{}",
        result
    );

    // Both length parameters should be removed
    assert!(
        !result.contains("len1") && !result.contains("len2"),
        "Should remove both length parameters\nGenerated:\n{}",
        result
    );

    // Should NOT use unsafe
    assert!(
        !result.contains("unsafe"),
        "Should generate safe code\nGenerated:\n{}",
        result
    );
}

/// Test array parameter with return value
///
/// C: int first_element(int* arr, int len) { return arr[0]; }
/// Rust: fn first_element(arr: &[i32]) -> i32 { arr[0] }
#[test]
// DECY-072 GREEN: Test now active
fn test_transform_array_parameter_with_return() {
    let c_code = r#"
        int first_element(int* arr, int len) {
            return arr[0];
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should transform to slice parameter
    assert!(
        result.contains("arr: &[i32]"),
        "Should use slice parameter\nGenerated:\n{}",
        result
    );

    // Should use safe array indexing
    assert!(
        result.contains("arr[0]"),
        "Should use safe indexing\nGenerated:\n{}",
        result
    );

    // Should NOT use unsafe
    assert!(
        !result.contains("unsafe"),
        "Should generate safe code\nGenerated:\n{}",
        result
    );
}

/// Test that non-array pointers are NOT transformed
///
/// C: void process(int* ptr) { }
/// Rust: fn process(ptr: *mut i32) { }  (should remain raw pointer)
#[test]
fn test_no_transform_non_array_pointer() {
    let c_code = r#"
        void process(int* ptr) {
            // Single pointer without length - not an array
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should NOT transform to slice (no length parameter)
    assert!(
        !result.contains(": &[i32]"),
        "Should NOT transform single pointer to slice\nGenerated:\n{}",
        result
    );

    // Should keep as raw pointer
    assert!(
        result.contains("ptr: *mut i32") || result.contains("ptr: *const i32"),
        "Should keep as raw pointer\nGenerated:\n{}",
        result
    );
}

/// Test unsafe block count metric
#[test]
// DECY-072 GREEN: Test now active
fn test_array_parameter_transformation_unsafe_count() {
    let c_code = r#"
        void process1(int* arr, int len) { arr[0] = 1; }
        void process2(char* buf, int size) { buf[0] = 'A'; }
        int sum(int* arr, int len) {
            int total = 0;
            for (int i = 0; i < len; i++) {
                total += arr[i];
            }
            return total;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Count unsafe blocks
    let unsafe_count = result.matches("unsafe").count();
    let loc = result.lines().count();
    let unsafe_per_1000 = (unsafe_count as f64 / loc as f64) * 1000.0;

    println!("Unsafe count: {}", unsafe_count);
    println!("Lines of code: {}", loc);
    println!("Unsafe per 1000 LOC: {:.2}", unsafe_per_1000);

    // CRITICAL: Verify <5 unsafe per 1000 LOC (target: 0 for array params)
    assert!(
        unsafe_per_1000 < 5.0,
        "Must achieve <5 unsafe blocks per 1000 LOC. Got {:.2}",
        unsafe_per_1000
    );

    // STRETCH GOAL: 0 unsafe for array parameters
    assert_eq!(
        unsafe_count, 0,
        "Array parameter transformation should achieve 0 unsafe blocks. Found {}",
        unsafe_count
    );
}
