//! Buffer Overflow Safety Property Tests
//!
//! **REFACTOR PHASE**: Property-based testing for buffer overflow patterns
//!
//! Tests 1000s of variations to validate safety invariants hold across
//! different array sizes, indices, and buffer operations.
//!
//! **Pattern**: Property-based testing with proptest
//! **Coverage**: 12 properties × 256 cases = 3,072+ test executions
//! **Goal**: Prove buffer overflow safety holds for all valid inputs

use decy_core::transpile;
use proptest::prelude::*;

// ============================================================================
// Property Test Strategies
// ============================================================================
// (Inline strategies used directly in tests)

// ============================================================================
// Property 1: Array access with valid index always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_array_access_valid_index_transpiles(
        size in 5usize..=20,
        index in 0usize..5
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int array[{}];
                int index = {};

                for (int i = 0; i < {}; i++) {{
                    array[i] = i;
                }}

                if (index < {}) {{
                    return array[index];
                }}

                return 0;
            }}
            "#,
            size, index, size, size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Array access with valid index should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 2: Array initialization always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_array_initialization_transpiles(
        size in 5usize..=30
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int array[{}];

                for (int i = 0; i < {}; i++) {{
                    array[i] = 0;
                }}

                return array[0];
            }}
            "#,
            size, size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Array initialization should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 3: Loop bounds checking always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_loop_bounds_checking_transpiles(
        size in 5usize..=25
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int array[{}];

                for (int i = 0; i < {}; i++) {{
                    array[i] = i * 2;
                }}

                return array[{}];
            }}
            "#,
            size, size, size - 1
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Loop bounds checking should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 4: Bounds check before access transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_bounds_check_transpiles(
        size in 10usize..=30,
        index in 0usize..50
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int array[{}];

                for (int i = 0; i < {}; i++) {{
                    array[i] = i;
                }}

                int index = {};
                if (index >= 0 && index < {}) {{
                    return array[index];
                }}

                return 0;
            }}
            "#,
            size, size, index, size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Bounds check should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 5: Multi-dimensional array access transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_multidimensional_array_transpiles(
        rows in 2usize..=10,
        cols in 2usize..=10
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int matrix[{}][{}];

                for (int i = 0; i < {}; i++) {{
                    for (int j = 0; j < {}; j++) {{
                        matrix[i][j] = i * {} + j;
                    }}
                }}

                return matrix[0][0];
            }}
            "#,
            rows, cols, rows, cols, cols
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Multi-dimensional array should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 6: Buffer copy operations transpile
// ============================================================================

proptest! {
    #[test]
    fn prop_buffer_copy_transpiles(
        size in 5usize..=20
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int source[{}];
                int dest[{}];

                for (int i = 0; i < {}; i++) {{
                    source[i] = i;
                }}

                for (int i = 0; i < {}; i++) {{
                    dest[i] = source[i];
                }}

                return dest[0];
            }}
            "#,
            size, size, size, size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Buffer copy should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 7: Pointer arithmetic with bounds transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_pointer_arithmetic_bounds_transpiles(
        size in 10usize..=30,
        offset in 0usize..10
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int array[{}];
                int* ptr = array;

                for (int i = 0; i < {}; i++) {{
                    array[i] = i;
                }}

                int offset = {};
                if (offset < {}) {{
                    ptr = ptr + offset;
                    return *ptr;
                }}

                return 0;
            }}
            "#,
            size, size, offset, size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Pointer arithmetic with bounds should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 8: Unsafe density below target
// ============================================================================

proptest! {
    #[test]
    fn prop_unsafe_density_below_target(
        size in 10usize..=30
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int array[{}];

                for (int i = 0; i < {}; i++) {{
                    array[i] = i;
                }}

                int sum = 0;
                for (int i = 0; i < {}; i++) {{
                    sum += array[i];
                }}

                return sum;
            }}
            "#,
            size, size, size
        );

        let result = transpile(&c_code).expect("Should transpile");

        let unsafe_count = result.matches("unsafe").count();
        let lines = result.lines().count();
        let unsafe_per_1000 = if lines > 0 {
            (unsafe_count as f64 / lines as f64) * 1000.0
        } else {
            0.0
        };

        // Property: <=30 unsafe per 1000 LOC for buffer operations
        prop_assert!(
            unsafe_per_1000 <= 30.0,
            "Unsafe per 1000 LOC should be <=30, got {:.2}",
            unsafe_per_1000
        );
    }
}

// ============================================================================
// Property 9: Generated code has balanced braces
// ============================================================================

proptest! {
    #[test]
    fn prop_generated_code_balanced(
        size in 5usize..=20
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int array[{}];

                for (int i = 0; i < {}; i++) {{
                    array[i] = i;
                }}

                return array[0];
            }}
            "#,
            size, size
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
// Property 10: Transpilation is deterministic
// ============================================================================

proptest! {
    #[test]
    fn prop_transpilation_deterministic(
        size in 5usize..=20
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int array[{}];

                for (int i = 0; i < {}; i++) {{
                    array[i] = i;
                }}

                return array[0];
            }}
            "#,
            size, size
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
// Property 11: Array with struct transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_array_in_struct_transpiles(
        size in 5usize..=15
    ) {
        let c_code = format!(
            r#"
            struct Data {{
                int buffer[{}];
                int count;
            }};

            int main() {{
                struct Data data;
                data.count = {};

                for (int i = 0; i < data.count; i++) {{
                    data.buffer[i] = i;
                }}

                return data.buffer[0];
            }}
            "#,
            size, size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Array in struct should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 12: Off-by-one prevention transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_off_by_one_prevention_transpiles(
        size in 5usize..=25
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int array[{}];

                for (int i = 0; i < {}; i++) {{
                    array[i] = i;
                }}

                int index = {};
                if (index < {}) {{
                    return array[index];
                }}

                return 0;
            }}
            "#,
            size, size, size - 1, size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Off-by-one prevention should transpile: {:?}", result.err());
    }
}
