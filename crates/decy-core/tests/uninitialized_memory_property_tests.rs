//! Uninitialized Memory Safety Property Tests
//!
//! **REFACTOR PHASE**: Property-based testing for initialization patterns
//!
//! Tests 1000s of variations to validate safety invariants hold across
//! different initialization values and patterns.
//!
//! **Pattern**: Property-based testing with proptest
//! **Coverage**: 11 properties × 256 cases = 2,816+ test executions
//! **Goal**: Prove initialization safety holds for all valid inputs

use decy_core::transpile;
use proptest::prelude::*;

// ============================================================================
// Property Test Strategies
// ============================================================================
// (Inline strategies used directly in tests)

// ============================================================================
// Property 1: Initialized local variable always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_initialized_local_transpiles(
        value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int value = {};
                return value;
            }}
            "#,
            value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Initialized local should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 2: Initialized array always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_initialized_array_transpiles(
        size in 1usize..=20,
        value in -100i32..=100
    ) {
        let initializer = (0..size).map(|_| value.to_string()).collect::<Vec<_>>().join(", ");

        let c_code = format!(
            r#"
            int main() {{
                int array[{}] = {{{}}};
                return array[0];
            }}
            "#,
            size, initializer
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Initialized array should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 3: Zero-initialized array always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_zero_initialized_array_transpiles(
        size in 1usize..=30
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int array[{}] = {{0}};
                return array[0];
            }}
            "#,
            size
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Zero-initialized array should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 4: Loop-initialized array always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_loop_initialized_array_transpiles(
        size in 1usize..=25
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

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Loop-initialized array should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 5: Initialized struct always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_initialized_struct_transpiles(
        x in -1000i32..=1000,
        y in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            struct Point {{
                int x;
                int y;
            }};

            int main() {{
                struct Point p = {{{}, {}}};
                return p.x + p.y;
            }}
            "#,
            x, y
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Initialized struct should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 6: Field-initialized struct always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_field_initialized_struct_transpiles(
        x in -1000i32..=1000,
        y in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            struct Point {{
                int x;
                int y;
            }};

            int main() {{
                struct Point p;
                p.x = {};
                p.y = {};
                return p.x + p.y;
            }}
            "#,
            x, y
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Field-initialized struct should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 7: Conditional initialization always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_conditional_initialization_transpiles(
        true_val in -1000i32..=1000,
        false_val in -1000i32..=1000,
        condition in any::<bool>()
    ) {
        let cond_val = if condition { 1 } else { 0 };

        let c_code = format!(
            r#"
            int main() {{
                int value;

                if ({}) {{
                    value = {};
                }} else {{
                    value = {};
                }}

                return value;
            }}
            "#,
            cond_val, true_val, false_val
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Conditional initialization should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 8: Static variable initialization always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_static_initialization_transpiles(
        init_val in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            int main() {{
                static int counter = {};
                counter++;
                return counter;
            }}
            "#,
            init_val
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Static initialization should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 9: Unsafe density below target
// ============================================================================

proptest! {
    #[test]
    fn prop_unsafe_density_below_target(
        a in -1000i32..=1000,
        b in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int b = {};
                int sum = a + b;

                int array[3] = {{1, 2, 3}};
                int product = array[0] * array[1];

                return sum + product;
            }}
            "#,
            a, b
        );

        let result = transpile(&c_code).expect("Should transpile");

        let unsafe_count = result.matches("unsafe").count();
        let lines = result.lines().count();
        let unsafe_per_1000 = if lines > 0 {
            (unsafe_count as f64 / lines as f64) * 1000.0
        } else {
            0.0
        };

        // Property: <=50 unsafe per 1000 LOC for initialization
        prop_assert!(
            unsafe_per_1000 <= 50.0,
            "Unsafe per 1000 LOC should be <=50, got {:.2}",
            unsafe_per_1000
        );
    }
}

// ============================================================================
// Property 10: Generated code has balanced braces
// ============================================================================

proptest! {
    #[test]
    fn prop_generated_code_balanced(
        value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int value = {};
                return value;
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
    fn prop_transpilation_deterministic(
        value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int value = {};
                return value;
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
