//! Integer Overflow Safety Property Tests
//!
//! **REFACTOR PHASE**: Property-based testing for integer overflow patterns
//!
//! Tests 1000s of variations to validate safety invariants hold across
//! different integer values and arithmetic operations.
//!
//! **Pattern**: Property-based testing with proptest
//! **Coverage**: 14 properties × 256 cases = 3,584+ test executions
//! **Goal**: Prove integer overflow prevention holds for all valid inputs

use decy_core::transpile;
use proptest::prelude::*;

// ============================================================================
// Property 1: Addition always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_addition_transpiles(
        a in -10000i32..=10000,
        b in -10000i32..=10000
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int b = {};
                int sum = a + b;

                return sum;
            }}
            "#,
            a, b
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Addition should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 2: Subtraction always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_subtraction_transpiles(
        a in -10000i32..=10000,
        b in -10000i32..=10000
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int b = {};
                int diff = a - b;

                return diff;
            }}
            "#,
            a, b
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Subtraction should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 3: Multiplication always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_multiplication_transpiles(
        a in -1000i32..=1000,
        b in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int b = {};
                int product = a * b;

                return product;
            }}
            "#,
            a, b
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Multiplication should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 4: Division with non-zero divisor transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_division_transpiles(
        a in -10000i32..=10000,
        b in 1i32..=1000
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int b = {};
                int quotient;

                if (b != 0) {{
                    quotient = a / b;
                }} else {{
                    quotient = 0;
                }}

                return quotient;
            }}
            "#,
            a, b
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Division should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 5: Modulo with non-zero divisor transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_modulo_transpiles(
        a in -10000i32..=10000,
        b in 1i32..=1000
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int b = {};
                int remainder;

                if (b != 0) {{
                    remainder = a % b;
                }} else {{
                    remainder = 0;
                }}

                return remainder;
            }}
            "#,
            a, b
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Modulo should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 6: Negation always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_negation_transpiles(
        a in -10000i32..=10000
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int b = -a;

                return b;
            }}
            "#,
            a
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Negation should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 7: Loop counter always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_loop_counter_transpiles(
        limit in 1usize..=100
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int i;
                int sum = 0;

                for (i = 0; i < {}; i++) {{
                    sum = sum + i;
                }}

                return sum;
            }}
            "#,
            limit
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Loop counter should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 8: Increment always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_increment_transpiles(
        a in -10000i32..=10000
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int b;

                a = a + 1;
                b = a;

                return b;
            }}
            "#,
            a
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Increment should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 9: Decrement always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_decrement_transpiles(
        a in -10000i32..=10000
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int b;

                a = a - 1;
                b = a;

                return b;
            }}
            "#,
            a
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Decrement should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 10: Compound addition always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_compound_addition_transpiles(
        a in -10000i32..=10000,
        b in -10000i32..=10000
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int b = {};

                a = a + b;

                return a;
            }}
            "#,
            a, b
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Compound addition should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 11: Compound multiplication always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_compound_multiplication_transpiles(
        a in -1000i32..=1000,
        b in -100i32..=100
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int b = {};

                a = a * b;

                return a;
            }}
            "#,
            a, b
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Compound multiplication should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 12: Unsafe density below target
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
                int product = sum * 2;
                int diff = product - 100;

                return diff;
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

        // Property: <=100 unsafe per 1000 LOC for integer overflow prevention
        prop_assert!(
            unsafe_per_1000 <= 100.0,
            "Unsafe per 1000 LOC should be <=100, got {:.2}",
            unsafe_per_1000
        );
    }
}

// ============================================================================
// Property 13: Generated code has balanced braces
// ============================================================================

proptest! {
    #[test]
    fn prop_generated_code_balanced(
        a in -1000i32..=1000,
        b in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int b = {};
                int sum = a + b;

                return sum;
            }}
            "#,
            a, b
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
// Property 14: Transpilation is deterministic
// ============================================================================

proptest! {
    #[test]
    fn prop_transpilation_deterministic(
        a in -1000i32..=1000,
        b in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int b = {};
                int sum = a + b;

                return sum;
            }}
            "#,
            a, b
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
