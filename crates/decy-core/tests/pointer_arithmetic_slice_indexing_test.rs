//! Integration test for pointer arithmetic → slice indexing transformation (DECY-070).
//!
//! Tests the complete pipeline: C parsing → HIR → ownership inference → transformation → codegen.
//! Verifies that pointer arithmetic on array-derived pointers generates safe slice indexing
//! with 0 unsafe blocks.

use decy_core::transpile;

/// Test stack array with pointer arithmetic.
///
/// C code with stack array and pointer arithmetic should generate
/// safe Rust code with slice indexing, not unsafe pointer arithmetic.
///
/// C: int arr[10];
///    int* p = arr;
///    return *(p + 1);
///
/// Expected Rust: let arr = [0; 10];
///                let p = &arr[..];
///                return p[1];  // OR arr[1]
#[test]
fn test_stack_array_pointer_arithmetic_to_slice_index() {
    let c_code = r#"
        int get_next() {
            int arr[10];
            int* p = arr;
            return *(p + 1);
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Verify NO unsafe blocks
    assert!(
        !result.contains("unsafe"),
        "Pointer arithmetic should NOT generate unsafe blocks.\nGenerated code:\n{}",
        result
    );

    // Verify no unsafe pointer methods
    assert!(
        !result.contains("wrapping_add"),
        "Should not use wrapping_add (unsafe pointer arithmetic).\nGenerated code:\n{}",
        result
    );
    assert!(
        !result.contains("offset"),
        "Should not use offset/offset_from (unsafe pointer arithmetic).\nGenerated code:\n{}",
        result
    );

    // Verify slice indexing pattern (either arr[1] or p[1])
    // The transformation should convert *(p + 1) to safe indexing
    assert!(
        result.contains("[") && result.contains("]"),
        "Should use safe slice indexing syntax.\nGenerated code:\n{}",
        result
    );
}

/// Test pointer from array initialization with pointer arithmetic.
///
/// C: int arr[10];
///    int* ptr = arr;
///    return *(ptr + 5);
///
/// Expected Rust: let arr = [0; 10];
///                let ptr = &arr[..];
///                return ptr[5];  // OR arr[5]
#[test]
fn test_array_init_pointer_arithmetic_to_slice_index() {
    let c_code = r#"
        int get_element() {
            int arr[10];
            int* ptr = arr;
            return *(ptr + 5);
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Verify NO unsafe blocks
    assert!(
        !result.contains("unsafe"),
        "Array pointer arithmetic should NOT generate unsafe blocks.\nGenerated code:\n{}",
        result
    );

    // Verify no unsafe pointer methods
    assert!(
        !result.contains("wrapping_add"),
        "Should not use wrapping_add.\nGenerated code:\n{}",
        result
    );
}

/// Test pointer arithmetic with subtraction.
///
/// C: int arr[10];
///    return *(arr - 2);  // Note: simplified to avoid compound expression
///
/// Expected Rust: Safe slice indexing with bounds checking
#[test]
fn test_pointer_subtraction_to_slice_index() {
    let c_code = r#"
        int get_prev() {
            int arr[10];
            return *(arr - 2);
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Verify NO unsafe blocks
    assert!(
        !result.contains("unsafe"),
        "Pointer subtraction should NOT generate unsafe blocks.\nGenerated code:\n{}",
        result
    );

    assert!(
        !result.contains("wrapping_sub"),
        "Should not use wrapping_sub.\nGenerated code:\n{}",
        result
    );
}

/// Test multiple pointer arithmetic operations.
///
/// C: int arr[10];
///    int a = *(arr + 1);
///    int b = *(arr + 5);
///    return a + b;
///
/// Expected Rust: Safe slice indexing for both operations
#[test]
fn test_multiple_pointer_arithmetic_operations() {
    let c_code = r#"
        int sum_elements() {
            int arr[10];
            int a = *(arr + 1);
            int b = *(arr + 5);
            return a + b;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Verify NO unsafe blocks
    assert!(
        !result.contains("unsafe"),
        "Multiple pointer arithmetic should NOT generate unsafe blocks.\nGenerated code:\n{}",
        result
    );

    // Count unsafe operations (should be 0)
    let unsafe_count = result.matches("unsafe").count();
    assert_eq!(
        unsafe_count, 0,
        "Should have 0 unsafe blocks, found {}.\nGenerated code:\n{}",
        unsafe_count, result
    );
}

/// Test pointer arithmetic with variable offset.
///
/// C: int arr[10];
///    int offset = 3;
///    return *(arr + offset);
///
/// Expected Rust: return arr[offset as usize];  // Safe runtime bounds check
#[test]
fn test_pointer_arithmetic_with_variable_offset() {
    let c_code = r#"
        int get_at_offset() {
            int arr[10];
            int offset = 3;
            return *(arr + offset);
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Verify NO unsafe blocks
    assert!(
        !result.contains("unsafe"),
        "Variable offset pointer arithmetic should NOT generate unsafe blocks.\nGenerated code:\n{}",
        result
    );
}

/// Verify unsafe block count metric for DECY mission: <5 unsafe per 1000 LOC
#[test]
fn test_pointer_arithmetic_unsafe_count_metric() {
    // Generate multiple functions with pointer arithmetic
    let c_code = r#"
        int test1() {
            int arr[10];
            return *(arr + 1);
        }

        int test2() {
            int arr[20];
            return *(arr + 5);
        }

        int test3() {
            int arr[15];
            int* p = arr;
            return *(p + 3);
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

    // CRITICAL: Verify <5 unsafe per 1000 LOC (target: 0 for pointer arithmetic)
    assert!(
        unsafe_per_1000 < 5.0,
        "Must achieve <5 unsafe blocks per 1000 LOC. Got {:.2} unsafe per 1000 LOC",
        unsafe_per_1000
    );

    // STRETCH GOAL: 0 unsafe for pointer arithmetic
    assert_eq!(
        unsafe_count, 0,
        "Pointer arithmetic transformation should achieve 0 unsafe blocks. Found {}",
        unsafe_count
    );
}
