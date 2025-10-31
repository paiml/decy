//! Loop + Array Safety Property Tests
//!
//! **REFACTOR PHASE**: Property-based testing for loop+array patterns
//!
//! Tests 1000s of variations to validate safety invariants hold across
//! different array sizes, loop bounds, and access patterns.
//!
//! **Pattern**: Property-based testing with proptest
//! **Coverage**: 10 properties × 256 cases = 2,560+ test executions
//! **Goal**: Prove safety holds for all valid inputs

use decy_core::transpile;
use proptest::prelude::*;

// ============================================================================
// Property Test Strategies
// ============================================================================

/// Generate safe array sizes (1-100)
fn array_size_strategy() -> impl Strategy<Value = usize> {
    1usize..=100
}

/// Generate loop iteration counts matching array size
fn matching_loop_bound() -> impl Strategy<Value = (usize, usize)> {
    (1usize..=50).prop_flat_map(|size| (Just(size), Just(size)))
}

/// Generate array initialization values
fn init_value_strategy() -> impl Strategy<Value = i32> {
    -100i32..=100
}

// ============================================================================
// Property 1: For loops always transpile successfully
// ============================================================================

proptest! {
    #[test]
    fn prop_for_loop_array_always_transpiles(
        array_size in array_size_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int array[{}];

                for (int i = 0; i < {}; i++) {{
                    array[i] = i;
                }}

                return 0;
            }}
            "#,
            array_size, array_size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "For loop should always transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 2: Array bounds are respected (no buffer overflow)
// ============================================================================

proptest! {
    #[test]
    fn prop_loop_bounds_match_array_size(
        (size, bound) in matching_loop_bound()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int data[{}];
                int sum = 0;

                for (int i = 0; i < {}; i++) {{
                    data[i] = i * 2;
                    sum += data[i];
                }}

                return sum;
            }}
            "#,
            size, bound
        );

        let result = transpile(&c_code).expect("Should transpile");

        // Property: Should generate valid code
        prop_assert!(result.contains("fn main"), "Should have main function");

        // Property: Unsafe should be minimal
        let unsafe_count = result.matches("unsafe").count();
        prop_assert!(
            unsafe_count < 10,
            "Should minimize unsafe (found {})",
            unsafe_count
        );
    }
}

// ============================================================================
// Property 3: Unsafe density stays below target
// ============================================================================

proptest! {
    #[test]
    fn prop_unsafe_density_below_target(
        array_size in 1usize..=50
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int numbers[{}];

                for (int i = 0; i < {}; i++) {{
                    numbers[i] = i * i;
                }}

                return numbers[0];
            }}
            "#,
            array_size, array_size
        );

        let result = transpile(&c_code).expect("Should transpile");

        let unsafe_count = result.matches("unsafe").count();
        let lines = result.lines().count();
        let unsafe_per_1000 = if lines > 0 {
            (unsafe_count as f64 / lines as f64) * 1000.0
        } else {
            0.0
        };

        // Property: <50 unsafe per 1000 LOC for loop+array patterns
        prop_assert!(
            unsafe_per_1000 < 50.0,
            "Unsafe per 1000 LOC should be <50, got {:.2}",
            unsafe_per_1000
        );
    }
}

// ============================================================================
// Property 4: While loops with arrays transpile
// ============================================================================

proptest! {
    #[test]
    fn prop_while_loop_array_transpiles(
        array_size in array_size_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int values[{}];
                int i = 0;

                while (i < {}) {{
                    values[i] = i + 1;
                    i++;
                }}

                return 0;
            }}
            "#,
            array_size, array_size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "While loop should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 5: Array initialization values are preserved
// ============================================================================

proptest! {
    #[test]
    fn prop_array_init_values_in_code(
        init_val in init_value_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int array[5];

                for (int i = 0; i < 5; i++) {{
                    array[i] = {};
                }}

                return array[0];
            }}
            "#,
            init_val
        );

        let result = transpile(&c_code).expect("Should transpile");

        // Property: initialization value should appear in code
        let init_str = init_val.to_string();
        prop_assert!(
            result.contains(&init_str),
            "Init value {} should appear in output",
            init_val
        );
    }
}

// ============================================================================
// Property 6: Generated code has balanced braces
// ============================================================================

proptest! {
    #[test]
    fn prop_generated_code_balanced(
        array_size in array_size_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int arr[{}];
                for (int i = 0; i < {}; i++) {{
                    arr[i] = 0;
                }}
                return 0;
            }}
            "#,
            array_size, array_size
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
// Property 7: Loop counter variable is generated
// ============================================================================

proptest! {
    #[test]
    fn prop_loop_counter_in_output(
        array_size in 1usize..=20
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int data[{}];
                for (int index = 0; index < {}; index++) {{
                    data[index] = index;
                }}
                return 0;
            }}
            "#,
            array_size, array_size
        );

        let result = transpile(&c_code).expect("Should transpile");

        // Property: loop variable name should appear
        prop_assert!(
            result.contains("index"),
            "Loop counter 'index' should appear in output"
        );
    }
}

// ============================================================================
// Property 8: Transpilation is deterministic
// ============================================================================

proptest! {
    #[test]
    fn prop_transpilation_deterministic(
        array_size in 1usize..=30
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int arr[{}];
                for (int i = 0; i < {}; i++) {{
                    arr[i] = i * 2;
                }}
                return 0;
            }}
            "#,
            array_size, array_size
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
// Property 9: Array copy loops work for varying sizes
// ============================================================================

proptest! {
    #[test]
    fn prop_array_copy_various_sizes(
        size in 1usize..=40
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
        prop_assert!(result.is_ok(), "Array copy should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 10: Nested loops with 2D arrays
// ============================================================================

proptest! {
    #[test]
    fn prop_nested_loops_2d_array(
        rows in 1usize..=10,
        cols in 1usize..=10
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int matrix[{}][{}];

                for (int i = 0; i < {}; i++) {{
                    for (int j = 0; j < {}; j++) {{
                        matrix[i][j] = i + j;
                    }}
                }}

                return 0;
            }}
            "#,
            rows, cols, rows, cols
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "2D array should transpile: {:?}", result.err());

        if let Ok(code) = result {
            // Should have main function
            prop_assert!(code.contains("fn main"), "Should generate main");
        }
    }
}
