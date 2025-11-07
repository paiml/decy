//! Array Parameter to Slice Transformation Tests
//!
//! **RED PHASE**: DECY-072 - Transform C array parameters to safe Rust slices
//!
//! Tests the transformation of C function signatures with array parameters
//! to idiomatic Rust slice parameters.
//!
//! **Pattern**: EXTREME TDD - Test-First Development
//! **Sprint**: 21 - Array Parameter Detection
//!
//! **Transformations**:
//! - `void process(int* arr, int len)` → `fn process(arr: &[i32])`
//! - `void modify(int* arr, int len)` → `fn modify(arr: &mut [i32])`
//! - Length parameter removed (redundant with slice)
//! - Function body uses `arr.len()` instead of length param

use decy_core::transpile;

// ============================================================================
// RED PHASE: Test 1 - Basic array parameter with length
// ============================================================================

#[test]
fn test_array_parameter_transforms_to_slice() {
    let c_code = r#"
        void process(int* arr, int len) {
            for (int i = 0; i < len; i++) {
                arr[i] = arr[i] * 2;
            }
        }
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Should transform to slice signature
    assert!(
        rust_code.contains("arr: &mut [i32]"),
        "Should transform array parameter to mutable slice:\n{}",
        rust_code
    );

    // Should NOT have redundant length parameter
    assert!(
        !rust_code.contains("len: i32") && !rust_code.contains("len: usize"),
        "Should not have redundant length parameter:\n{}",
        rust_code
    );

    // Function body should use arr.len()
    assert!(
        rust_code.contains("arr.len()"),
        "Should use arr.len() instead of len parameter:\n{}",
        rust_code
    );
}

// ============================================================================
// RED PHASE: Test 2 - Read-only array parameter (immutable slice)
// ============================================================================

#[test]
fn test_readonly_array_parameter_transforms_to_immutable_slice() {
    let c_code = r#"
        int sum(const int* arr, int count) {
            int total = 0;
            for (int i = 0; i < count; i++) {
                total += arr[i];
            }
            return total;
        }
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Should transform to immutable slice
    assert!(
        rust_code.contains("arr: &[i32]"),
        "Should transform const array parameter to immutable slice:\n{}",
        rust_code
    );

    // Should NOT have redundant count parameter
    assert!(
        !rust_code.contains("count: i32") && !rust_code.contains("count: usize"),
        "Should not have redundant count parameter:\n{}",
        rust_code
    );
}

// ============================================================================
// RED PHASE: Test 3 - Char buffer with size
// ============================================================================

#[test]
fn test_char_buffer_transforms_to_u8_slice() {
    let c_code = r#"
        void fill_buffer(char* buf, size_t size) {
            for (size_t i = 0; i < size; i++) {
                buf[i] = 'A';
            }
        }
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Should transform to u8 slice
    assert!(
        rust_code.contains("buf: &mut [u8]"),
        "Should transform char buffer to mutable u8 slice:\n{}",
        rust_code
    );

    // Should NOT have redundant size parameter
    assert!(
        !rust_code.contains("size:"),
        "Should not have redundant size parameter:\n{}",
        rust_code
    );
}

// ============================================================================
// RED PHASE: Test 4 - Multiple array parameters
// ============================================================================

#[test]
fn test_multiple_array_parameters_transform_correctly() {
    let c_code = r#"
        void merge(int* dest, int dest_len, const int* src, int src_len) {
            for (int i = 0; i < src_len && i < dest_len; i++) {
                dest[i] = src[i];
            }
        }
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Should have both slices
    assert!(
        rust_code.contains("dest: &mut [i32]"),
        "Should transform dest to mutable slice:\n{}",
        rust_code
    );
    assert!(
        rust_code.contains("src: &[i32]"),
        "Should transform src to immutable slice:\n{}",
        rust_code
    );

    // Should NOT have redundant length parameters
    assert!(
        !rust_code.contains("dest_len:") && !rust_code.contains("src_len:"),
        "Should not have redundant length parameters:\n{}",
        rust_code
    );
}

// ============================================================================
// RED PHASE: Test 5 - Array parameter without length NOT transformed
// ============================================================================

#[test]
fn test_pointer_without_length_not_transformed_to_slice() {
    let c_code = r#"
        void process_single(int* ptr) {
            *ptr = 42;
        }
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Should NOT transform to slice (no length parameter)
    assert!(
        !rust_code.contains("&[i32]") && !rust_code.contains("&mut [i32]"),
        "Should not transform single pointer to slice:\n{}",
        rust_code
    );

    // Should use &mut i32 or *mut i32 instead
    assert!(
        rust_code.contains("&mut i32") || rust_code.contains("*mut i32"),
        "Should use reference or raw pointer for single pointer:\n{}",
        rust_code
    );
}

// ============================================================================
// RED PHASE: Test 6 - Function body length references updated
// ============================================================================

#[test]
fn test_length_parameter_references_replaced_with_len_method() {
    let c_code = r#"
        int calculate(int* numbers, int count) {
            if (count == 0) {
                return 0;
            }
            int sum = 0;
            for (int i = 0; i < count; i++) {
                sum += numbers[i];
            }
            return count * 2;
        }
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // All references to count should be replaced with numbers.len()
    let count_occurrences =
        rust_code.matches(" count").count() + rust_code.matches("count ").count();
    assert_eq!(
        count_occurrences, 0,
        "Should not have any 'count' variable references in body:\n{}",
        rust_code
    );

    // Should have multiple uses of .len()
    let len_occurrences = rust_code.matches(".len()").count();
    assert!(
        len_occurrences >= 3,
        "Should have at least 3 uses of .len():\n{}",
        rust_code
    );
}

// ============================================================================
// RED PHASE: Test 7 - Array parameter with arithmetic (not transformed)
// ============================================================================

#[test]
fn test_array_with_pointer_arithmetic_not_transformed() {
    let c_code = r#"
        void process_with_arithmetic(int* arr, int len) {
            int* ptr = arr + 5;
            *ptr = 10;
        }
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Should NOT transform to slice if pointer arithmetic detected
    // (from DECY-071: has_pointer_arithmetic should return false for array params)
    // But if arithmetic is used, it should not be detected as array parameter
    // This test verifies the negative case
    assert!(
        !rust_code.contains("&[i32]") && !rust_code.contains("&mut [i32]"),
        "Should not transform to slice when pointer arithmetic present:\n{}",
        rust_code
    );
}

// ============================================================================
// RED PHASE: Test 8 - Integration: Full function generation
// ============================================================================

#[test]
fn test_full_function_with_array_slice_transformation() {
    let c_code = r#"
        void normalize(float* values, int n) {
            float sum = 0.0;
            for (int i = 0; i < n; i++) {
                sum += values[i];
            }
            float avg = sum / n;
            for (int i = 0; i < n; i++) {
                values[i] = values[i] - avg;
            }
        }
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Verify slice signature
    assert!(
        rust_code.contains("fn normalize(values: &mut [f32])")
            || rust_code.contains("pub fn normalize(values: &mut [f32])"),
        "Should have slice signature:\n{}",
        rust_code
    );

    // Verify no 'n' parameter
    assert!(
        !rust_code.contains(", n:"),
        "Should not have n parameter:\n{}",
        rust_code
    );

    // Verify .len() usage
    assert!(
        rust_code.contains("values.len()"),
        "Should use values.len():\n{}",
        rust_code
    );
}
