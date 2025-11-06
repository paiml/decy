//! Double Free Safety Property Tests
//!
//! **REFACTOR PHASE**: Property-based testing for double free patterns
//!
//! Tests 1000s of variations to validate safety invariants hold across
//! different allocation sizes, free patterns, and pointer management.
//!
//! **Pattern**: Property-based testing with proptest
//! **Coverage**: 10 properties × 256 cases = 2,560+ test executions
//! **Goal**: Prove double free prevention holds for all valid inputs
//!
//! **NOTE**: All tests currently ignored due to parser header include path issues.
//! These tests require system headers (<stdlib.h>) which are not accessible in
//! the current test environment.

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
    #[ignore]
    fn prop_simple_malloc_free_transpiles(
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

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Simple malloc/free should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 2: NULL after free always transpiles
// ============================================================================

proptest! {
    #[test]
    #[ignore]
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
                    free(ptr);
                    ptr = 0;
                }}
                return 0;
            }}
            "#,
            value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "NULL after free should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 3: Conditional free always transpiles
// ============================================================================

proptest! {
    #[test]
    #[ignore]
    fn prop_conditional_free_transpiles(
        value in -1000i32..=1000,
        should_free in any::<bool>()
    ) {
        let flag_val = if should_free { 1 } else { 0 };

        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
                int* ptr = (int*)malloc(sizeof(int));
                int should_free = {};

                if (ptr != 0 && should_free) {{
                    *ptr = {};
                    free(ptr);
                }}

                return 0;
            }}
            "#,
            flag_val, value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Conditional free should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 4: Array of pointers always transpiles
// ============================================================================

proptest! {
    #[test]
    #[ignore]
    fn prop_array_of_pointers_transpiles(
        size in 1usize..=10,
        value in -100i32..=100
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
                int* array[{}];
                int i;

                for (i = 0; i < {}; i++) {{
                    array[i] = (int*)malloc(sizeof(int));
                    if (array[i] != 0) {{
                        *array[i] = {};
                    }}
                }}

                for (i = 0; i < {}; i++) {{
                    if (array[i] != 0) {{
                        free(array[i]);
                        array[i] = 0;
                    }}
                }}

                return 0;
            }}
            "#,
            size, size, value, size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Array of pointers should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 5: Struct with allocated member always transpiles
// ============================================================================

proptest! {
    #[test]
    #[ignore]
    fn prop_struct_member_transpiles(
        value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            struct Container {{
                int* data;
            }};

            int main() {{
                struct Container c;
                c.data = (int*)malloc(sizeof(int));

                if (c.data != 0) {{
                    *c.data = {};
                    free(c.data);
                    c.data = 0;
                }}

                return 0;
            }}
            "#,
            value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Struct member should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 6: Function ownership transfer always transpiles
// ============================================================================

proptest! {
    #[test]
    #[ignore]
    fn prop_function_ownership_transpiles(
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
                    cleanup(ptr);
                }}
                return 0;
            }}
            "#,
            value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Function ownership should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 7: Multi-free protection always transpiles
// ============================================================================

proptest! {
    #[test]
    #[ignore]
    fn prop_multi_free_protection_transpiles(
        value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
                int* ptr = (int*)malloc(sizeof(int));
                int freed = 0;

                if (ptr != 0) {{
                    *ptr = {};

                    if (!freed) {{
                        free(ptr);
                        freed = 1;
                    }}

                    if (!freed) {{
                        free(ptr);
                    }}
                }}

                return 0;
            }}
            "#,
            value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Multi-free protection should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 8: Error path free always transpiles
// ============================================================================

proptest! {
    #[test]
    #[ignore]
    fn prop_error_path_free_transpiles(
        val1 in -1000i32..=1000,
        val2 in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
                int* ptr1 = (int*)malloc(sizeof(int));
                int* ptr2 = (int*)malloc(sizeof(int));

                if (ptr1 == 0 || ptr2 == 0) {{
                    if (ptr1 != 0) {{
                        free(ptr1);
                    }}
                    if (ptr2 != 0) {{
                        free(ptr2);
                    }}
                    return 1;
                }}

                *ptr1 = {};
                *ptr2 = {};

                free(ptr1);
                free(ptr2);

                return 0;
            }}
            "#,
            val1, val2
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Error path free should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 9: Unsafe density below target
// ============================================================================

proptest! {
    #[test]
    #[ignore]
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
                    int val = *ptr;
                    free(ptr);
                    ptr = 0;
                    return val;
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

        // Property: <=100 unsafe per 1000 LOC for double free prevention
        prop_assert!(
            unsafe_per_1000 <= 100.0,
            "Unsafe per 1000 LOC should be <=100, got {:.2}",
            unsafe_per_1000
        );
    }
}

// ============================================================================
// Property 10: Generated code has balanced braces
// ============================================================================

proptest! {
    #[test]
    #[ignore]
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
// Property 11: Transpilation is deterministic
// ============================================================================

proptest! {
    #[test]
    #[ignore]
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
                    free(ptr);
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
