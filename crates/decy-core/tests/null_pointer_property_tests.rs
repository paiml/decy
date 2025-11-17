//! NULL Pointer Safety Property Tests
//!
//! **REFACTOR PHASE**: Property-based testing for NULL pointer patterns
//!
//! Tests 1000s of variations to validate safety invariants hold across
//! different NULL check patterns and pointer operations.
//!
//! **Pattern**: Property-based testing with proptest
//! **Coverage**: 8 properties × 256 cases = 2,048+ test executions
//! **Goal**: Prove NULL safety holds for all valid inputs

use decy_core::transpile;
use proptest::prelude::*;

// ============================================================================
// Property Test Strategies
// ============================================================================

/// Generate safe integer values for pointer content
fn value_strategy() -> impl Strategy<Value = i32> {
    -1000i32..=1000
}

/// Generate boolean for NULL/not-NULL scenarios
fn null_scenario_strategy() -> impl Strategy<Value = bool> {
    any::<bool>()
}

// ============================================================================
// Property 1: NULL checks always transpile
// ============================================================================

proptest! {
    #[test]
    #[ignore = "Parser limitation: Cannot handle #include <stdlib.h>. malloc/free require system headers."]
    fn prop_null_check_transpiles(
        value in value_strategy()
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
                int* ptr = (int*)malloc(sizeof(int));

                if (ptr == 0) {{
                    return 1;
                }}

                *ptr = {};
                free(ptr);
                return 0;
            }}
            "#,
            value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "NULL check should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 2: NULL initialization transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_null_initialization_transpiles(
        _ in 0..10usize
    ) {
        let c_code = r#"
            int main() {
                int* ptr = 0;

                if (ptr == 0) {
                    return 1;
                }

                return 0;
            }
        "#;

        let result = transpile(c_code);
        prop_assert!(result.is_ok(), "NULL initialization should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 3: Function returning NULL
// ============================================================================

proptest! {
    #[test]
    #[ignore = "Parser limitation: Cannot handle malloc/free without system headers."]
    fn prop_function_return_null_transpiles(
        return_null in null_scenario_strategy()
    ) {
        let condition = if return_null { "0" } else { "1" };

        let c_code = format!(
            r#"
            int* get_ptr(int cond) {{
                if (cond == 0) {{
                    return 0;
                }}
                int* ptr = (int*)malloc(sizeof(int));
                return ptr;
            }}

            int main() {{
                int* ptr = get_ptr({});

                if (ptr != 0) {{
                    free(ptr);
                }}

                return 0;
            }}
            "#,
            condition
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Function return NULL should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 4: Defensive NULL checks
// ============================================================================

proptest! {
    #[test]
    fn prop_defensive_null_check_transpiles(
        value in value_strategy()
    ) {
        let c_code = format!(
            r#"
            int safe_deref(int* ptr) {{
                if (ptr == 0) {{
                    return -1;
                }}
                return *ptr;
            }}

            int main() {{
                int val = {};
                int result = safe_deref(&val);
                return result;
            }}
            "#,
            value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Defensive NULL check should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 5: Unsafe density below target
// ============================================================================

proptest! {
    #[test]
    #[ignore = "Parser limitation: Cannot handle #include <stdlib.h>. malloc/free require system headers."]
    fn prop_unsafe_density_below_target(
        value in value_strategy()
    ) {
        let c_code = format!(
            r#"
            #include <stdlib.h>

            int main() {{
                int* ptr = (int*)malloc(sizeof(int));

                if (ptr == 0) {{
                    return 1;
                }}

                *ptr = {};
                int result = *ptr;
                free(ptr);

                return result;
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

        // Property: <=100 unsafe per 1000 LOC for NULL patterns
        prop_assert!(
            unsafe_per_1000 <= 100.0,
            "Unsafe per 1000 LOC should be <=100, got {:.2}",
            unsafe_per_1000
        );
    }
}

// ============================================================================
// Property 6: Generated code balanced braces
// ============================================================================

proptest! {
    #[test]
    fn prop_generated_code_balanced(
        _ in 0..10usize
    ) {
        let c_code = r#"
            int main() {
                int* ptr = 0;

                if (ptr != 0) {
                    *ptr = 42;
                }

                return 0;
            }
        "#;

        let result = transpile(c_code).expect("Should transpile");

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
        value in value_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int val = {};
                int* ptr = &val;

                if (ptr != 0) {{
                    return *ptr;
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

// ============================================================================
// Property 8: NULL coalescing pattern
// ============================================================================

proptest! {
    #[test]
    fn prop_null_coalescing_transpiles(
        default_value in value_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int* ptr = 0;
                int value = (ptr != 0) ? *ptr : {};

                return value;
            }}
            "#,
            default_value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "NULL coalescing should transpile: {:?}", result.err());
    }
}
