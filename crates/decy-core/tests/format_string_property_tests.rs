//! Format String Safety Property Tests
//!
//! **REFACTOR PHASE**: Property-based testing for format string patterns
//!
//! Tests 1000s of variations to validate safety invariants hold across
//! different format specifiers, argument values, and buffer sizes.
//!
//! **Pattern**: Property-based testing with proptest
//! **Coverage**: 12 properties × 256 cases = 3,072+ test executions
//! **Goal**: Prove format string safety holds for all valid inputs
//!
//! # FIXED: Parser System Header Support
//!
//! **STATUS**: Property tests now passing with stdlib prototype support! ✅
//!
//! **SOLUTION**: decy-stdlib provides stdio.h prototypes (Sprint 18).

use decy_core::transpile;
use proptest::prelude::*;

// ============================================================================
// Property Test Strategies
// ============================================================================
// (Inline strategies used directly in tests)

// ============================================================================
// Property 1: printf with integer always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_printf_integer_transpiles(
        value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            #include <stdio.h>

            int main() {{
                printf("Value: %d\n", {});
                return 0;
            }}
            "#,
            value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "printf with integer should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 2: printf with multiple integers always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_printf_multiple_integers_transpiles(
        a in -1000i32..=1000,
        b in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            #include <stdio.h>

            int main() {{
                printf("a=%d, b=%d, sum=%d\n", {}, {}, {});
                return 0;
            }}
            "#,
            a, b, a.wrapping_add(b)
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "printf with multiple integers should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 3: printf with float always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_printf_float_transpiles(
        value in -1000.0f64..=1000.0
    ) {
        let c_code = format!(
            r#"
            #include <stdio.h>

            int main() {{
                printf("Value: %f\n", {});
                return 0;
            }}
            "#,
            value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "printf with float should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 4: sprintf to buffer always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_sprintf_transpiles(
        buffer_size in 10usize..=200,
        value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            #include <stdio.h>

            int main() {{
                char buffer[{}];
                sprintf(buffer, "Value: %d", {});
                return 0;
            }}
            "#,
            buffer_size, value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "sprintf should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 5: snprintf with bounds always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_snprintf_transpiles(
        buffer_size in 10usize..=200,
        value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            #include <stdio.h>

            int main() {{
                char buffer[{}];
                snprintf(buffer, sizeof(buffer), "Value: %d", {});
                return 0;
            }}
            "#,
            buffer_size, value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "snprintf should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 6: printf with width specifier always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_printf_width_transpiles(
        width in 1usize..=50,
        value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            #include <stdio.h>

            int main() {{
                printf("%{}d\n", {});
                return 0;
            }}
            "#,
            width, value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "printf with width should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 7: printf with precision always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_printf_precision_transpiles(
        precision in 0usize..=10,
        value in -1000.0f64..=1000.0
    ) {
        let c_code = format!(
            r#"
            #include <stdio.h>

            int main() {{
                printf("%.{}f\n", {});
                return 0;
            }}
            "#,
            precision, value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "printf with precision should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 8: scanf with width specifier always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_scanf_width_transpiles(
        buffer_size in 5usize..=100
    ) {
        let width = buffer_size - 1; // Leave room for null terminator

        let c_code = format!(
            r#"
            #include <stdio.h>

            int main() {{
                char buffer[{}];
                scanf("%{}s", buffer);
                return 0;
            }}
            "#,
            buffer_size, width
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "scanf with width should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 9: printf with hex format always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_printf_hex_transpiles(
        value in 0u32..=0xFFFF
    ) {
        let c_code = format!(
            r#"
            #include <stdio.h>

            int main() {{
                printf("Hex: 0x%x\n", {});
                return 0;
            }}
            "#,
            value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "printf with hex should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 10: Unsafe density below target
// ============================================================================

proptest! {
    #[test]
    fn prop_unsafe_density_below_target(
        a in -1000i32..=1000,
        b in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            #include <stdio.h>

            int main() {{
                printf("Values: a=%d, b=%d\n", {}, {});

                char buffer[50];
                sprintf(buffer, "Sum: %d", {});

                return 0;
            }}
            "#,
            a, b, a.wrapping_add(b)
        );

        let result = transpile(&c_code).expect("Should transpile");

        let unsafe_count = result.matches("unsafe").count();
        let lines = result.lines().count();
        let unsafe_per_1000 = if lines > 0 {
            (unsafe_count as f64 / lines as f64) * 1000.0
        } else {
            0.0
        };

        // Property: <=30 unsafe per 1000 LOC for format strings
        prop_assert!(
            unsafe_per_1000 <= 30.0,
            "Unsafe per 1000 LOC should be <=30, got {:.2}",
            unsafe_per_1000
        );
    }
}

// ============================================================================
// Property 11: Generated code has balanced braces
// ============================================================================

proptest! {
    #[test]
    fn prop_generated_code_balanced(
        value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            #include <stdio.h>

            int main() {{
                printf("Value: %d\n", {});
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
// Property 12: Transpilation is deterministic
// ============================================================================

proptest! {
    #[test]
    fn prop_transpilation_deterministic(
        value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            #include <stdio.h>

            int main() {{
                printf("Value: %d\n", {});
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
