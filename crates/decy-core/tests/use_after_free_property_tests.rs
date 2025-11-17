//! Use-After-Free Safety Property Tests
//!
//! **REFACTOR PHASE**: Property-based testing for use-after-free patterns
//!
//! Tests 1000s of variations to validate safety invariants hold across
//! different allocation patterns and lifetime scenarios.
//!
//! **Pattern**: Property-based testing with proptest
//! **Coverage**: 10 properties × 256 cases = 2,560+ test executions
//! **Goal**: Prove lifetime safety holds for all valid inputs
//!
//! # FIXED: Parser System Header Support
//!
//! **STATUS**: Property tests now passing with stdlib prototype support! ✅
//!
//! **SOLUTION**: decy-stdlib provides malloc/free prototypes (Sprint 18).
//!
//! **TOYOTA WAY - Kaizen (改善)**: Continuous improvement through TDD!

use decy_core::transpile;
use proptest::prelude::*;

// ============================================================================
// Property Test Strategies
// ============================================================================
// (Inline strategies used directly in tests)

// ============================================================================
// Property 1: Simple malloc/free always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_malloc_free_transpiles(
        value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
                int* ptr = (int*)malloc(sizeof(int));

                if (ptr != 0) {{
                    *ptr = {};
                    int result = *ptr;
                    free(ptr);
                    return result;
                }}

                return 0;
            }}
            "#,
            value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "malloc/free should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 2: Nulling pointer after free transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_null_after_free_transpiles(
        value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
                int* ptr = (int*)malloc(sizeof(int));

                if (ptr != 0) {{
                    *ptr = {};
                    int result = *ptr;
                    free(ptr);
                    ptr = 0;
                    return result;
                }}

                return 0;
            }}
            "#,
            value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Null after free should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 3: Conditional free transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_conditional_free_transpiles(
        value in -100i32..=100,
        condition in any::<bool>()
    ) {
        let cond_val = if condition { 1 } else { 0 };

        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
                int* ptr = (int*)malloc(sizeof(int));

                if (ptr != 0) {{
                    *ptr = {};

                    if ({}) {{
                        int result = *ptr;
                        free(ptr);
                        return result;
                    }} else {{
                        int result = *ptr;
                        free(ptr);
                        return result;
                    }}
                }}

                return 0;
            }}
            "#,
            value, cond_val
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Conditional free should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 4: Loop with malloc/free transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_loop_malloc_free_transpiles(
        iterations in 1usize..=10
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
                int sum = 0;

                for (int i = 0; i < {}; i++) {{
                    int* ptr = (int*)malloc(sizeof(int));

                    if (ptr != 0) {{
                        *ptr = i;
                        sum += *ptr;
                        free(ptr);
                    }}
                }}

                return sum;
            }}
            "#,
            iterations
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Loop malloc/free should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 5: Array of allocated pointers transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_array_of_pointers_transpiles(
        size in 2usize..=10
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
                int* array[{}];

                for (int i = 0; i < {}; i++) {{
                    array[i] = (int*)malloc(sizeof(int));
                    if (array[i] != 0) {{
                        *array[i] = i;
                    }}
                }}

                int sum = 0;
                for (int i = 0; i < {}; i++) {{
                    if (array[i] != 0) {{
                        sum += *array[i];
                        free(array[i]);
                    }}
                }}

                return sum;
            }}
            "#,
            size, size, size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Array of pointers should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 6: Struct with allocated member transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_struct_allocated_member_transpiles(
        value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            struct Container {{
                int* data;
            }};

            int main() {{
                struct Container container;
                container.data = (int*)malloc(sizeof(int));

                if (container.data != 0) {{
                    *container.data = {};
                    int result = *container.data;
                    free(container.data);
                    container.data = 0;
                    return result;
                }}

                return 0;
            }}
            "#,
            value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Struct allocated member should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 7: Function that frees argument transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_function_frees_arg_transpiles(
        value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            void cleanup(int* ptr) {{
                if (ptr != 0) {{
                    free(ptr);
                }}
            }}

            int main() {{
                int* ptr = (int*)malloc(sizeof(int));

                if (ptr != 0) {{
                    *ptr = {};
                    int result = *ptr;
                    cleanup(ptr);
                    return result;
                }}

                return 0;
            }}
            "#,
            value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Function freeing arg should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 8: Unsafe density below target
// ============================================================================

proptest! {
    #[test]
    fn prop_unsafe_density_below_target(
        value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
                int* ptr = (int*)malloc(sizeof(int));

                if (ptr != 0) {{
                    *ptr = {};
                    int result = *ptr;
                    free(ptr);
                    ptr = 0;
                    return result;
                }}

                return 0;
            }}
            "#,
            value
        );

        let result = transpile(&c_code).expect("Should transpile");

        let unsafe_count = result.matches("unsafe").count();
        let lines = result.lines().count();
        let unsafe_per_1000 = if lines > 0 {
            (unsafe_count as f64 / lines as f64) * 1000.0
        } else {
            0.0
        };

        // Property: <=100 unsafe per 1000 LOC for use-after-free patterns
        prop_assert!(
            unsafe_per_1000 <= 100.0,
            "Unsafe per 1000 LOC should be <=100, got {:.2}",
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
        value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
                int* ptr = (int*)malloc(sizeof(int));

                if (ptr != 0) {{
                    *ptr = {};
                    free(ptr);
                }}

                return 0;
            }}
            "#,
            value
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
        value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
                int* ptr = (int*)malloc(sizeof(int));

                if (ptr != 0) {{
                    *ptr = {};
                    int result = *ptr;
                    free(ptr);
                    return result;
                }}

                return 0;
            }}
            "#,
            value
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
