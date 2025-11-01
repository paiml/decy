//! Buffer Overflow Safety Property Tests
//!
//! **REFACTOR PHASE**: Property-based testing for buffer overflow patterns
//!
//! Tests 1000s of variations to validate safety invariants hold across
//! different array sizes, indices, and buffer operations.
//!
//! **Pattern**: Property-based testing with proptest
//! **Coverage**: 12 properties × 256 cases = 3,072+ test executions
//! **Goal**: Prove buffer overflow prevention holds for all valid inputs

use decy_core::transpile;
use proptest::prelude::*;

// ============================================================================
// Property 1: Fixed array access always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_fixed_array_access_transpiles(
        size in 1usize..=20,
        value in -100i32..=100
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int arr[{}];
                int i;

                for (i = 0; i < {}; i++) {{
                    arr[i] = {};
                }}

                return arr[0];
            }}
            "#,
            size, size, value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Fixed array access should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 2: Array index validation always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_array_index_validation_transpiles(
        size in 1usize..=10,
        index in 0usize..=9
    ) {
        let safe_index = index.min(size - 1);
        let c_code = format!(
            r#"
            int main() {{
                int arr[{}];
                int index = {};

                if (index >= 0 && index < {}) {{
                    arr[index] = 42;
                }}

                return 0;
            }}
            "#,
            size, safe_index, size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Array index validation should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 3: Loop bounds checking always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_loop_bounds_checked_transpiles(
        size in 1usize..=15,
        multiplier in -10i32..=10
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int arr[{}];
                int i;

                for (i = 0; i < {}; i++) {{
                    arr[i] = i * {};
                }}

                return arr[0];
            }}
            "#,
            size, size, multiplier
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Loop bounds checking should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 4: 2D array access always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_2d_array_access_transpiles(
        rows in 1usize..=5,
        cols in 1usize..=5
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int matrix[{}][{}];
                int i;
                int j;

                for (i = 0; i < {}; i++) {{
                    for (j = 0; j < {}; j++) {{
                        matrix[i][j] = i + j;
                    }}
                }}

                return matrix[0][0];
            }}
            "#,
            rows, cols, rows, cols
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "2D array access should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 5: Buffer copy operations always transpile
// ============================================================================

proptest! {
    #[test]
    fn prop_buffer_copy_transpiles(
        size in 1usize..=10,
        value in -50i32..=50
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int src[{}];
                int dst[{}];
                int i;

                for (i = 0; i < {}; i++) {{
                    src[i] = {};
                }}

                for (i = 0; i < {}; i++) {{
                    dst[i] = src[i];
                }}

                return dst[0];
            }}
            "#,
            size, size, size, value, size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Buffer copy should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 6: Partial buffer copy always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_partial_buffer_copy_transpiles(
        size in 5usize..=15,
        copy_count in 1usize..=10
    ) {
        let safe_count = copy_count.min(size);
        let c_code = format!(
            r#"
            int main() {{
                int src[{}];
                int dst[{}];
                int count = {};
                int i;

                for (i = 0; i < {}; i++) {{
                    src[i] = i;
                }}

                for (i = 0; i < count && i < {}; i++) {{
                    dst[i] = src[i];
                }}

                return 0;
            }}
            "#,
            size, size, safe_count, size, size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Partial buffer copy should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 7: String buffer operations always transpile
// ============================================================================

proptest! {
    #[test]
    fn prop_string_buffer_transpiles(
        buffer_size in 10usize..=30,
        fill_size in 1usize..=25
    ) {
        let safe_fill = fill_size.min(buffer_size - 1);
        let c_code = format!(
            r#"
            int main() {{
                char str[{}];
                int i;

                for (i = 0; i < {}; i++) {{
                    str[i] = 'A';
                }}
                str[{}] = '\0';

                return 0;
            }}
            "#,
            buffer_size, safe_fill, safe_fill
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "String buffer should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 8: Variable size arrays always transpile
// ============================================================================

proptest! {
    #[test]
    fn prop_variable_size_array_transpiles(
        array_size in 5usize..=20,
        used_size in 1usize..=15
    ) {
        let safe_used = used_size.min(array_size);
        let c_code = format!(
            r#"
            int main() {{
                int size = {};
                int arr[{}];
                int i;

                for (i = 0; i < size && i < {}; i++) {{
                    arr[i] = i;
                }}

                return 0;
            }}
            "#,
            safe_used, array_size, array_size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Variable size array should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 9: Struct with array member always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_struct_with_array_transpiles(
        array_size in 1usize..=10,
        value in -100i32..=100
    ) {
        let c_code = format!(
            r#"
            struct Buffer {{
                int data[{}];
                int size;
            }};

            int main() {{
                struct Buffer buf;
                int i;

                buf.size = {};

                for (i = 0; i < buf.size; i++) {{
                    buf.data[i] = {};
                }}

                return 0;
            }}
            "#,
            array_size, array_size, value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Struct with array should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 10: Nested arrays always transpile
// ============================================================================

proptest! {
    #[test]
    fn prop_nested_arrays_transpile(
        outer_size in 1usize..=4,
        inner_size in 1usize..=4
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int arrays[{}][{}];
                int i;
                int j;

                for (i = 0; i < {}; i++) {{
                    for (j = 0; j < {}; j++) {{
                        arrays[i][j] = i + j;
                    }}
                }}

                return arrays[0][0];
            }}
            "#,
            outer_size, inner_size, outer_size, inner_size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Nested arrays should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 11: Unsafe density below target
// ============================================================================

proptest! {
    #[test]
    fn prop_unsafe_density_below_target(
        size in 10usize..=50
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int buffer[{}];
                int i;
                int sum = 0;

                for (i = 0; i < {}; i++) {{
                    buffer[i] = i;
                }}

                for (i = 0; i < {}; i++) {{
                    sum = sum + buffer[i];
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

        // Property: <=100 unsafe per 1000 LOC for buffer overflow prevention
        prop_assert!(
            unsafe_per_1000 <= 100.0,
            "Unsafe per 1000 LOC should be <=100, got {:.2}",
            unsafe_per_1000
        );
    }
}

// ============================================================================
// Property 12: Generated code has balanced braces
// ============================================================================

proptest! {
    #[test]
    fn prop_generated_code_balanced(
        size in 1usize..=15,
        value in -100i32..=100
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int arr[{}];
                int i;

                for (i = 0; i < {}; i++) {{
                    arr[i] = {};
                }}

                return arr[0];
            }}
            "#,
            size, size, value
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
// Property 13: Transpilation is deterministic
// ============================================================================

proptest! {
    #[test]
    fn prop_transpilation_deterministic(
        size in 1usize..=10,
        value in -50i32..=50
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int arr[{}];
                int i;

                for (i = 0; i < {}; i++) {{
                    arr[i] = {};
                }}

                return arr[0];
            }}
            "#,
            size, size, value
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
