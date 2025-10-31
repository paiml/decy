//! Pointer Arithmetic Safety Property Tests
//!
//! **REFACTOR PHASE**: Property-based testing for pointer arithmetic patterns
//!
//! Tests 1000s of variations to validate safety invariants hold across
//! different array sizes, offsets, and pointer operations.
//!
//! **Pattern**: Property-based testing with proptest
//! **Coverage**: 10 properties × 256 cases = 2,560+ test executions
//! **Goal**: Prove pointer safety holds for all valid inputs

use decy_core::transpile;
use proptest::prelude::*;

// ============================================================================
// Property Test Strategies
// ============================================================================

/// Generate safe array sizes (1-100 elements)
fn array_size_strategy() -> impl Strategy<Value = usize> {
    1usize..=100
}

/// Generate safe offsets (0-50)
fn offset_strategy() -> impl Strategy<Value = usize> {
    0usize..=50
}

/// Generate valid array indices
fn valid_index_strategy(max: usize) -> impl Strategy<Value = usize> {
    0usize..max
}

// ============================================================================
// Property 1: Pointer increment always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_pointer_increment_transpiles(
        array_size in array_size_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int array[{}];
                int* ptr = array;

                for (int i = 0; i < {}; i++) {{
                    *ptr = i;
                    ptr++;
                }}

                return 0;
            }}
            "#,
            array_size, array_size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Pointer increment should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 2: Pointer addition with offset
// ============================================================================

proptest! {
    #[test]
    fn prop_pointer_addition_transpiles(
        size in 10usize..=50,
        offset in 0usize..=9
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int array[{}];
                int* ptr = array;

                int value = *(ptr + {});

                return value;
            }}
            "#,
            size, offset
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Pointer addition should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 3: Array traversal with pointer
// ============================================================================

proptest! {
    #[test]
    fn prop_array_traversal_transpiles(
        size in array_size_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int array[{}];
                int* ptr = array;
                int sum = 0;

                for (int i = 0; i < {}; i++) {{
                    sum += *ptr;
                    ptr++;
                }}

                return sum;
            }}
            "#,
            size, size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Array traversal should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 4: Pointer indexing (ptr[i])
// ============================================================================

proptest! {
    #[test]
    fn prop_pointer_indexing_transpiles(
        size in 10usize..=50,
        index in 0usize..=9
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int array[{}];
                int* ptr = array;

                int value = ptr[{}];

                return value;
            }}
            "#,
            size, index
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Pointer indexing should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 5: Unsafe density stays below target
// ============================================================================

proptest! {
    #[test]
    fn prop_unsafe_density_below_target(
        size in 10usize..=30
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int data[{}];
                int* ptr = data;

                for (int i = 0; i < {}; i++) {{
                    *ptr = i;
                    ptr++;
                }}

                return 0;
            }}
            "#,
            size, size
        );

        let result = transpile(&c_code).expect("Should transpile");

        let unsafe_count = result.matches("unsafe").count();
        let lines = result.lines().count();
        let unsafe_per_1000 = if lines > 0 {
            (unsafe_count as f64 / lines as f64) * 1000.0
        } else {
            0.0
        };

        // Property: <250 unsafe per 1000 LOC for pointer patterns
        prop_assert!(
            unsafe_per_1000 < 250.0,
            "Unsafe per 1000 LOC should be <250, got {:.2}",
            unsafe_per_1000
        );
    }
}

// ============================================================================
// Property 6: Generated code has balanced braces
// ============================================================================

proptest! {
    #[test]
    fn prop_generated_code_balanced(
        size in array_size_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int array[{}];
                int* ptr = array;
                int value = *ptr;
                return value;
            }}
            "#,
            size
        );

        let result = transpile(&c_code).expect("Should transpile");

        let open_braces = result.matches('{').count();
        let close_braces = result.matches('}').count();

        prop_assert_eq!(
            open_braces, close_braces,
            "Braces should be balanced: {} open, {} close",
            open_braces, close_braces
        );
    }
}

// ============================================================================
// Property 7: Transpilation is deterministic
// ============================================================================

proptest! {
    #[test]
    fn prop_transpilation_deterministic(
        size in 1usize..=30
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int array[{}];
                int* ptr = array;
                *ptr = 42;
                return *ptr;
            }}
            "#,
            size
        );

        // Transpile twice
        let result1 = transpile(&c_code).expect("Should transpile (1)");
        let result2 = transpile(&c_code).expect("Should transpile (2)");

        // Property: identical inputs produce identical outputs
        prop_assert_eq!(
            result1, result2,
            "Transpilation should be deterministic"
        );
    }
}

// ============================================================================
// Property 8: Pointer comparison always works
// ============================================================================

proptest! {
    #[test]
    fn prop_pointer_comparison_transpiles(
        size in array_size_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int array[{}];
                int* start = array;
                int* end = &array[{}];
                int* current = start;
                int count = 0;

                while (current < end) {{
                    count++;
                    current++;
                }}

                return count;
            }}
            "#,
            size, size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Pointer comparison should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 9: Pointer difference calculation
// ============================================================================

proptest! {
    #[test]
    fn prop_pointer_difference_transpiles(
        size in 20usize..=50,
        offset1 in 0usize..=10,
        offset2 in 11usize..=19
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int array[{}];
                int* ptr1 = &array[{}];
                int* ptr2 = &array[{}];

                int distance = ptr2 - ptr1;

                return distance;
            }}
            "#,
            size, offset1, offset2
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Pointer difference should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 10: Backwards traversal with pointer
// ============================================================================

proptest! {
    #[test]
    fn prop_backwards_traversal_transpiles(
        size in array_size_strategy()
    ) {
        let last_index = if size > 0 { size - 1 } else { 0 };

        let c_code = format!(
            r#"
            int main() {{
                int array[{}];
                int* ptr = &array[{}];
                int sum = 0;

                for (int i = 0; i < {}; i++) {{
                    sum += *ptr;
                    ptr--;
                }}

                return sum;
            }}
            "#,
            size, last_index, size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Backwards traversal should transpile: {:?}", result.err());
    }
}
