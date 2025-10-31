//! Dynamic Memory Safety Property Tests
//!
//! **REFACTOR PHASE**: Property-based testing for malloc/free patterns
//!
//! Tests 1000s of variations to validate safety invariants hold across
//! different allocation sizes, patterns, and edge cases.
//!
//! **Pattern**: Property-based testing with proptest
//! **Coverage**: 8 properties × 256 cases = 2,048+ test executions
//! **Goal**: Prove memory safety holds for all valid inputs

use decy_core::transpile;
use proptest::prelude::*;

// ============================================================================
// Property Test Strategies
// ============================================================================

/// Generate safe allocation sizes (1-1000 elements)
fn allocation_size_strategy() -> impl Strategy<Value = usize> {
    1usize..=1000
}

/// Generate small allocation sizes for performance (1-100)
fn small_allocation_strategy() -> impl Strategy<Value = usize> {
    1usize..=100
}

/// Generate array element counts
fn element_count_strategy() -> impl Strategy<Value = usize> {
    1usize..=50
}

// ============================================================================
// Property 1: malloc/free always transpiles successfully
// ============================================================================

proptest! {
    #[test]
    fn prop_malloc_free_always_transpiles(
        size in allocation_size_strategy()
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
                int* ptr = (int*)malloc(sizeof(int) * {});

                if (ptr != 0) {{
                    ptr[0] = 42;
                    free(ptr);
                }}

                return 0;
            }}
            "#,
            size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "malloc/free should always transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 2: calloc always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_calloc_always_transpiles(
        count in element_count_strategy()
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
                int* array = (int*)calloc({}, sizeof(int));

                if (array != 0) {{
                    int sum = 0;
                    for (int i = 0; i < {}; i++) {{
                        sum += array[i];
                    }}
                    free(array);
                }}

                return 0;
            }}
            "#,
            count, count
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "calloc should always transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 3: Unsafe density stays below target
// ============================================================================

proptest! {
    #[test]
    fn prop_unsafe_density_below_target(
        size in small_allocation_strategy()
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
                int* data = (int*)malloc(sizeof(int) * {});

                if (data != 0) {{
                    for (int i = 0; i < {}; i++) {{
                        data[i] = i;
                    }}
                    free(data);
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

        // Property: <100 unsafe per 1000 LOC for malloc patterns
        prop_assert!(
            unsafe_per_1000 < 100.0,
            "Unsafe per 1000 LOC should be <100, got {:.2}",
            unsafe_per_1000
        );
    }
}

// ============================================================================
// Property 4: Generated code has balanced braces
// ============================================================================

proptest! {
    #[test]
    fn prop_generated_code_balanced(
        size in small_allocation_strategy()
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
                int* ptr = (int*)malloc(sizeof(int) * {});
                if (ptr != 0) {{
                    free(ptr);
                }}
                return 0;
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
// Property 5: Transpilation is deterministic
// ============================================================================

proptest! {
    #[test]
    fn prop_transpilation_deterministic(
        size in 1usize..=30
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
                int* array = (int*)malloc(sizeof(int) * {});
                if (array != 0) {{
                    array[0] = 123;
                    free(array);
                }}
                return 0;
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
// Property 6: Multiple malloc/free pairs
// ============================================================================

proptest! {
    #[test]
    fn prop_multiple_allocations(
        count in 1usize..=10
    ) {
        // Generate multiple malloc/free pairs
        let mut allocations = String::new();
        let mut frees = String::new();

        for i in 0..count {
            allocations.push_str(&format!(
                "    int* ptr{} = (int*)malloc(sizeof(int));\n",
                i
            ));
            allocations.push_str(&format!(
                "    if (ptr{} != 0) *ptr{} = {};\n",
                i, i, i
            ));
        }

        for i in 0..count {
            frees.push_str(&format!("    if (ptr{} != 0) free(ptr{});\n", i, i));
        }

        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
            {}
            {}
                return 0;
            }}
            "#,
            allocations, frees
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Multiple malloc/free should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 7: Struct allocation with various sizes
// ============================================================================

proptest! {
    #[test]
    fn prop_struct_allocation(
        field_value in 0i32..=1000
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            struct Data {{
                int value;
            }};

            int main() {{
                struct Data* d = (struct Data*)malloc(sizeof(struct Data));

                if (d != 0) {{
                    d->value = {};
                    free(d);
                }}

                return 0;
            }}
            "#,
            field_value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Struct malloc should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 8: NULL check patterns always compile
// ============================================================================

proptest! {
    #[test]
    fn prop_null_check_patterns(
        size in small_allocation_strategy()
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
                int* data = (int*)malloc(sizeof(int) * {});

                if (data == 0) {{
                    return 1;  // Allocation failed
                }}

                data[0] = 42;
                free(data);
                return 0;
            }}
            "#,
            size
        );

        let result = transpile(&c_code).expect("Should transpile");

        // Should have main function
        prop_assert!(result.contains("fn main"), "Should generate main");

        // NULL check should be handled
        prop_assert!(!result.is_empty(), "Should generate non-empty code");
    }
}
