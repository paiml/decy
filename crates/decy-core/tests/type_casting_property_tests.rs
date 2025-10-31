//! Type Casting Safety Property Tests
//!
//! **REFACTOR PHASE**: Property-based testing for type casting patterns
//!
//! Tests 1000s of variations to validate safety invariants hold across
//! different value ranges, cast types, and conversion patterns.
//!
//! **Pattern**: Property-based testing with proptest
//! **Coverage**: 8 properties × 256 cases = 2,048+ test executions
//! **Goal**: Prove type safety holds for all valid inputs

use decy_core::transpile;
use proptest::prelude::*;

// ============================================================================
// Property Test Strategies
// ============================================================================

/// Generate safe integer values (-1000 to 1000)
fn int_value_strategy() -> impl Strategy<Value = i32> {
    -1000i32..=1000
}

/// Generate unsigned values (0 to 1000)
fn uint_value_strategy() -> impl Strategy<Value = u32> {
    0u32..=1000
}

/// Generate char-range values (0-127)
fn char_value_strategy() -> impl Strategy<Value = i32> {
    0i32..=127
}

// ============================================================================
// Property 1: Int to char cast always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_int_to_char_cast_transpiles(
        value in int_value_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int value = {};
                char ch = (char)value;
                return ch;
            }}
            "#,
            value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Int to char cast should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 2: Char to int cast always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_char_to_int_cast_transpiles(
        value in char_value_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                char ch = {};
                int value = (int)ch;
                return value;
            }}
            "#,
            value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Char to int cast should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 3: Unsigned to signed conversion
// ============================================================================

proptest! {
    #[test]
    fn prop_unsigned_to_signed_transpiles(
        value in uint_value_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                unsigned int u = {}U;
                int s = (int)u;
                return s;
            }}
            "#,
            value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Unsigned to signed cast should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 4: Pointer casts transpile
// ============================================================================

proptest! {
    #[test]
    fn prop_pointer_casts_transpile(
        value in int_value_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int val = {};
                int* iptr = &val;
                void* vptr = (void*)iptr;
                int* back = (int*)vptr;
                return *back;
            }}
            "#,
            value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Pointer casts should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 5: Unsafe density below target
// ============================================================================

proptest! {
    #[test]
    fn prop_unsafe_density_below_target(
        value in int_value_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int i = {};
                char c = (char)i;
                unsigned int u = (unsigned int)i;
                int result = (int)c + (int)u;
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

        // Property: <150 unsafe per 1000 LOC for type casts
        prop_assert!(
            unsafe_per_1000 < 150.0,
            "Unsafe per 1000 LOC should be <150, got {:.2}",
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
        value in int_value_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int value = {};
                char ch = (char)value;
                return ch;
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
// Property 7: Transpilation is deterministic
// ============================================================================

proptest! {
    #[test]
    fn prop_transpilation_deterministic(
        value in int_value_strategy()
    ) {
        let c_code = format!(
            r#"
            int main() {{
                int i = {};
                char c = (char)i;
                return c;
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
// Property 8: Enum conversions transpile
// ============================================================================

proptest! {
    #[test]
    fn prop_enum_conversions_transpile(
        value in 0i32..=2
    ) {
        let c_code = format!(
            r#"
            enum Color {{
                RED = 0,
                GREEN = 1,
                BLUE = 2
            }};

            int main() {{
                enum Color c = {};
                int val = (int)c;
                return val;
            }}
            "#,
            value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Enum conversion should transpile: {:?}", result.err());
    }
}
