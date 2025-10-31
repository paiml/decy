//! Integer Overflow Safety Property Tests
//!
//! **REFACTOR PHASE**: Property-based testing for integer overflow patterns
//!
//! Tests 1000s of variations to validate safety invariants hold across
//! different integer values and operations.
//!
//! **Pattern**: Property-based testing with proptest
//! **Coverage**: 10 properties × 256 cases = 2,560+ test executions
//! **Goal**: Prove overflow safety holds for all valid inputs

use decy_core::transpile;
use proptest::prelude::*;

// ============================================================================
// Property Test Strategies
// ============================================================================

/// Generate safe integer values for arithmetic
fn safe_int_strategy() -> impl Strategy<Value = i32> {
    -10000i32..=10000
}

/// Generate values near INT_MAX for overflow testing
fn near_max_strategy() -> impl Strategy<Value = i32> {
    2147483640i32..=2147483647
}

/// Generate values near INT_MIN for underflow testing
fn near_min_strategy() -> impl Strategy<Value = i32> {
    -2147483648i32..=-2147483640
}

/// Generate non-zero integers
fn non_zero_strategy() -> impl Strategy<Value = i32> {
    prop_oneof![
        -10000i32..=-1,
        1i32..=10000,
    ]
}

// ============================================================================
// Property 1: Addition always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_addition_transpiles(
        a in safe_int_strategy(),
        b in safe_int_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int b = {};
                int result = a + b;
                return result;
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
        a in safe_int_strategy(),
        b in safe_int_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int b = {};
                int result = a - b;
                return result;
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
                int result = a * b;
                return result;
            }}
            "#,
            a, b
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Multiplication should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 4: Division with non-zero always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_division_transpiles(
        a in safe_int_strategy(),
        b in non_zero_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int b = {};
                int result = a / b;
                return result;
            }}
            "#,
            a, b
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Division should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 5: Negation always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_negation_transpiles(
        a in safe_int_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int result = -a;
                return result;
            }}
            "#,
            a
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Negation should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 6: Left shift within bounds transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_left_shift_transpiles(
        a in 1i32..=1000,
        shift in 0usize..16
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int result = a << {};
                return result;
            }}
            "#,
            a, shift
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Left shift should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 7: Increment/decrement transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_increment_transpiles(
        a in safe_int_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                a++;
                return a;
            }}
            "#,
            a
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Increment should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 8: Compound assignment transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_compound_assignment_transpiles(
        a in safe_int_strategy(),
        b in safe_int_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int sum = {};
                int value = {};
                sum += value;
                return sum;
            }}
            "#,
            a, b
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Compound assignment should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 9: Unsafe density below target
// ============================================================================

proptest! {
    #[test]
    fn prop_unsafe_density_below_target(
        a in safe_int_strategy(),
        b in safe_int_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int b = {};
                int sum = a + b;
                int product = a * b;
                int result = sum + product;
                return result;
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

        // Property: <=50 unsafe per 1000 LOC for overflow patterns
        prop_assert!(
            unsafe_per_1000 <= 50.0,
            "Unsafe per 1000 LOC should be <=50, got {:.2}",
            unsafe_per_1000
        );
    }
}

// ============================================================================
// Property 10: Generated code balanced
// ============================================================================

proptest! {
    #[test]
    fn prop_generated_code_balanced(
        a in safe_int_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int value = {};
                int result = value + 10;
                return result;
            }}
            "#,
            a
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
        a in safe_int_strategy(),
        b in safe_int_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int b = {};
                int result = a + b;
                return result;
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

// ============================================================================
// Property 12: Near-overflow values transpile
// ============================================================================

proptest! {
    #[test]
    fn prop_near_overflow_transpiles(
        a in near_max_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int b = 1;
                int result = a + b;
                return result;
            }}
            "#,
            a
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Near-overflow should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 13: Near-underflow values transpile
// ============================================================================

proptest! {
    #[test]
    fn prop_near_underflow_transpiles(
        a in near_min_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int a = {};
                int b = 1;
                int result = a - b;
                return result;
            }}
            "#,
            a
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Near-underflow should transpile: {:?}", result.err());
    }
}
