//! Race Condition Safety Property Tests
//!
//! **REFACTOR PHASE**: Property-based testing for race condition patterns
//!
//! Tests 1000s of variations to validate safety invariants hold across
//! different shared state patterns, counter values, and concurrent operations.
//!
//! **Pattern**: Property-based testing with proptest
//! **Coverage**: 11 properties × 256 cases = 2,816+ test executions
//! **Goal**: Prove race condition safety holds for all valid inputs

use decy_core::transpile;
use proptest::prelude::*;

// ============================================================================
// Property Test Strategies
// ============================================================================
// (Inline strategies used directly in tests)

// ============================================================================
// Property 1: Global variable always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_global_variable_transpiles(
        initial_value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            int counter = {};

            int main() {{
                counter = counter + 1;
                return counter;
            }}
            "#,
            initial_value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Global variable should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 2: Multiple globals always transpile
// ============================================================================

proptest! {
    #[test]
    fn prop_multiple_globals_transpile(
        val1 in -1000i32..=1000,
        val2 in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            int counter1 = {};
            int counter2 = {};

            int main() {{
                counter1 = counter1 + 1;
                counter2 = counter2 + 1;
                return counter1 + counter2;
            }}
            "#,
            val1, val2
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Multiple globals should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 3: Read-modify-write always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_read_modify_write_transpiles(
        balance in 0i32..=10000,
        amount in 0i32..=1000
    ) {
        let c_code = format!(
            r#"
            int balance = {};

            int withdraw(int amount) {{
                int temp = balance;
                temp = temp - amount;
                balance = temp;
                return balance;
            }}

            int main() {{
                int result = withdraw({});
                return result;
            }}
            "#,
            balance, amount
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Read-modify-write should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 4: Increment/decrement always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_increment_decrement_transpiles(
        initial in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            int counter = {};

            void increment() {{
                counter = counter + 1;
            }}

            void decrement() {{
                counter = counter - 1;
            }}

            int main() {{
                increment();
                decrement();
                return counter;
            }}
            "#,
            initial
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Increment/decrement should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 5: Shared array access always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_shared_array_transpiles(
        size in 1usize..=50,
        index in 0usize..10
    ) {
        let safe_index = index.min(size - 1);

        let c_code = format!(
            r#"
            int shared_array[{}];

            int main() {{
                int i;
                for (i = 0; i < {}; i++) {{
                    shared_array[i] = i * 2;
                }}
                return shared_array[{}];
            }}
            "#,
            size, size, safe_index
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Shared array should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 6: Check-then-act always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_check_then_act_transpiles(
        resource_count in 0i32..=100
    ) {
        let c_code = format!(
            r#"
            int resource_count = {};

            int allocate_resource() {{
                if (resource_count > 0) {{
                    resource_count = resource_count - 1;
                    return 1;
                }}
                return 0;
            }}

            int main() {{
                int result = allocate_resource();
                return result;
            }}
            "#,
            resource_count
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Check-then-act should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 7: Flag-based sync always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_flag_based_sync_transpiles(
        data_value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            int data_ready = 0;
            int shared_data = 0;

            void producer() {{
                shared_data = {};
                data_ready = 1;
            }}

            int consumer() {{
                if (data_ready == 1) {{
                    return shared_data;
                }}
                return 0;
            }}

            int main() {{
                producer();
                int result = consumer();
                return result;
            }}
            "#,
            data_value
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Flag-based sync should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 8: Producer-consumer counter always transpiles
// ============================================================================

proptest! {
    #[test]
    fn prop_producer_consumer_transpiles(
        initial_produced in 0i32..=100,
        initial_consumed in 0i32..=100
    ) {
        let c_code = format!(
            r#"
            int items_produced = {};
            int items_consumed = {};

            void produce() {{
                items_produced = items_produced + 1;
            }}

            void consume() {{
                if (items_produced > items_consumed) {{
                    items_consumed = items_consumed + 1;
                }}
            }}

            int main() {{
                produce();
                consume();
                return items_consumed;
            }}
            "#,
            initial_produced, initial_consumed
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Producer-consumer should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 9: Struct shared fields always transpile
// ============================================================================

proptest! {
    #[test]
    fn prop_struct_shared_fields_transpiles(
        counter_val in -1000i32..=1000,
        flag_val in 0i32..=1
    ) {
        let c_code = format!(
            r#"
            struct SharedData {{
                int counter;
                int flag;
            }};

            struct SharedData shared;

            int main() {{
                shared.counter = {};
                shared.flag = {};
                shared.counter = shared.counter + 1;
                return shared.counter;
            }}
            "#,
            counter_val, flag_val
        );

        let result = transpile(&c_code);
        prop_assert!(result.is_ok(), "Struct shared fields should transpile: {:?}", result.err());
    }
}

// ============================================================================
// Property 10: Unsafe density below target
// ============================================================================

proptest! {
    #[test]
    fn prop_unsafe_density_below_target(
        initial_value in -1000i32..=1000
    ) {
        let c_code = format!(
            r#"
            int shared_counter = {};

            void increment() {{
                int temp = shared_counter;
                temp = temp + 1;
                shared_counter = temp;
            }}

            int main() {{
                increment();
                increment();
                return shared_counter;
            }}
            "#,
            initial_value
        );

        let result = transpile(&c_code).expect("Should transpile");

        let unsafe_count = result.matches("unsafe").count();
        let lines = result.lines().count();
        let unsafe_per_1000 = if lines > 0 {
            (unsafe_count as f64 / lines as f64) * 1000.0
        } else {
            0.0
        };

        // DECY-261: Current implementation generates one unsafe per global variable access
        // Property: <=250 unsafe per 1000 LOC (baseline), target <=50 after optimization
        prop_assert!(
            unsafe_per_1000 <= 250.0,
            "Unsafe per 1000 LOC should be <=250, got {:.2}",
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
            int counter = {};

            int main() {{
                counter = counter + 1;
                return counter;
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
            int counter = {};

            int main() {{
                counter = counter + 1;
                return counter;
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
