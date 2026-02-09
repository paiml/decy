//! DECY-194: Deterministic output guarantee tests.
//!
//! These tests verify that transpiling the same C code twice produces
//! byte-identical Rust output. This is critical for:
//! - Cache correctness (same input → same hash → same output)
//! - Reproducible builds
//! - CI/CD reliability
//!
//! These tests are append-only: never weaken or delete.

/// Helper: transpile C code twice and assert byte-identical output.
fn assert_deterministic(c_code: &str) {
    let result1 = decy_core::transpile(c_code).expect("First transpilation failed");
    let result2 = decy_core::transpile(c_code).expect("Second transpilation failed");
    assert_eq!(
        result1, result2,
        "Transpilation produced different output for same input:\n--- First ---\n{}\n--- Second ---\n{}",
        result1, result2
    );
}

#[test]
fn determinism_001_empty_main() {
    assert_deterministic("int main() { return 0; }");
}

#[test]
fn determinism_002_simple_function() {
    assert_deterministic("int add(int a, int b) { return a + b; }");
}

#[test]
fn determinism_003_multiple_functions() {
    assert_deterministic(
        r#"
        int square(int x) { return x * x; }
        int cube(int x) { return x * x * x; }
        int main() { return square(3) + cube(2); }
        "#,
    );
}

#[test]
fn determinism_004_variable_declarations() {
    assert_deterministic(
        r#"
        int main() {
            int x = 10;
            int y = 20;
            int z = x + y;
            return z;
        }
        "#,
    );
}

#[test]
fn determinism_005_if_else() {
    assert_deterministic(
        r#"
        int max(int a, int b) {
            if (a > b) {
                return a;
            } else {
                return b;
            }
        }
        "#,
    );
}

#[test]
fn determinism_006_while_loop() {
    assert_deterministic(
        r#"
        int sum_to_n(int n) {
            int total = 0;
            int i = 1;
            while (i <= n) {
                total = total + i;
                i = i + 1;
            }
            return total;
        }
        "#,
    );
}

#[test]
fn determinism_007_for_loop() {
    assert_deterministic(
        r#"
        int factorial(int n) {
            int result = 1;
            int i;
            for (i = 1; i <= n; i++) {
                result = result * i;
            }
            return result;
        }
        "#,
    );
}

#[test]
fn determinism_008_struct_definition() {
    assert_deterministic(
        r#"
        struct Point {
            int x;
            int y;
        };

        int distance_sq(struct Point p) {
            return p.x * p.x + p.y * p.y;
        }
        "#,
    );
}

#[test]
fn determinism_009_nested_control_flow() {
    assert_deterministic(
        r#"
        int classify(int n) {
            if (n > 0) {
                if (n > 100) {
                    return 2;
                } else {
                    return 1;
                }
            } else {
                return 0;
            }
        }
        "#,
    );
}

#[test]
fn determinism_010_switch_statement() {
    assert_deterministic(
        r#"
        int day_type(int day) {
            switch (day) {
                case 0: return 0;
                case 6: return 0;
                default: return 1;
            }
        }
        "#,
    );
}

#[test]
fn determinism_011_function_with_params() {
    assert_deterministic(
        r#"
        int clamp(int value, int min, int max) {
            if (value < min) return min;
            if (value > max) return max;
            return value;
        }
        "#,
    );
}

#[test]
fn determinism_012_arithmetic_expressions() {
    assert_deterministic(
        r#"
        int compute(int a, int b, int c) {
            return (a + b) * c - a / (b + 1);
        }
        "#,
    );
}

#[test]
fn determinism_013_multiple_runs_consistent() {
    let c_code = r#"
        int fibonacci(int n) {
            if (n <= 1) return n;
            int a = 0;
            int b = 1;
            int i;
            for (i = 2; i <= n; i++) {
                int temp = a + b;
                a = b;
                b = temp;
            }
            return b;
        }
    "#;

    // Run 10 times and verify all outputs are identical
    let first = decy_core::transpile(c_code).expect("transpilation failed");
    for run in 1..10 {
        let result = decy_core::transpile(c_code).expect("transpilation failed");
        assert_eq!(
            first, result,
            "Divergence detected on run {} of 10",
            run + 1
        );
    }
}

#[test]
fn determinism_014_comparison_operators() {
    assert_deterministic(
        r#"
        int compare(int a, int b) {
            if (a == b) return 0;
            if (a < b) return -1;
            if (a > b) return 1;
            if (a <= b) return -2;
            if (a >= b) return 2;
            if (a != b) return 3;
            return -99;
        }
        "#,
    );
}

#[test]
fn determinism_015_logical_operators() {
    assert_deterministic(
        r#"
        int check(int a, int b) {
            if (a > 0 && b > 0) return 1;
            if (a > 0 || b > 0) return 2;
            return 0;
        }
        "#,
    );
}
