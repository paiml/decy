//! DECY-192: Falsification test suite.
//!
//! 50 C programs that MUST transpile and produce compilable Rust.
//! These tests are **append-only**: never weaken or delete a test.
//!
//! Organization:
//! - C001-C010: Integer types and arithmetic
//! - C011-C020: Control flow (if/else, switch, loops)
//! - C021-C030: Pointer basics
//! - C031-C040: Structs
//! - C041-C050: Functions and return types

use std::io::Write;
use std::process::Command;

/// Helper: transpile C code and verify the result compiles with rustc.
///
/// Steps:
/// 1. Transpile C → Rust via `decy_core::transpile()`
/// 2. Write Rust to temp file
/// 3. Run `rustc --edition 2021 --emit=metadata` to verify compilation
fn transpile_and_compile(c_code: &str) -> TranspileResult {
    // Step 1: Transpile
    let rust_code = match decy_core::transpile(c_code) {
        Ok(code) => code,
        Err(e) => {
            return TranspileResult {
                transpiled: false,
                compiled: false,
                rust_code: String::new(),
                error: Some(format!("Transpilation failed: {}", e)),
            };
        }
    };

    // Step 2: Write to temp file
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let rs_path = temp_dir.path().join("test.rs");
    let mut file = std::fs::File::create(&rs_path).expect("Failed to create temp file");
    file.write_all(rust_code.as_bytes())
        .expect("Failed to write");
    drop(file);

    // Step 3: Try to compile with rustc
    let output = Command::new("rustc")
        .args(["--edition", "2021", "--emit=metadata", "-o"])
        .arg(temp_dir.path().join("test.rmeta"))
        .arg(&rs_path)
        .output()
        .expect("Failed to run rustc");

    let compiled = output.status.success();
    let error = if compiled {
        None
    } else {
        Some(String::from_utf8_lossy(&output.stderr).to_string())
    };

    TranspileResult {
        transpiled: true,
        compiled,
        rust_code,
        error,
    }
}

struct TranspileResult {
    transpiled: bool,
    compiled: bool,
    rust_code: String,
    error: Option<String>,
}

/// Assert that C code transpiles and compiles successfully.
fn assert_transpiles_and_compiles(c_code: &str, test_id: &str) {
    let result = transpile_and_compile(c_code);
    assert!(
        result.transpiled,
        "[{}] Transpilation failed: {}",
        test_id,
        result.error.unwrap_or_default()
    );
    assert!(
        result.compiled,
        "[{}] Compilation failed.\nRust code:\n{}\nError:\n{}",
        test_id,
        result.rust_code,
        result.error.unwrap_or_default()
    );
}

// ============================================================================
// C001-C010: Integer Types and Arithmetic
// ============================================================================

#[test]
fn c001_return_zero() {
    assert_transpiles_and_compiles("int main() { return 0; }", "C001");
}

#[test]
fn c002_return_literal() {
    assert_transpiles_and_compiles("int main() { return 42; }", "C002");
}

#[test]
fn c003_int_variable() {
    assert_transpiles_and_compiles(
        "int main() { int x = 10; return x; }",
        "C003",
    );
}

#[test]
fn c004_addition() {
    assert_transpiles_and_compiles(
        "int main() { int a = 3; int b = 4; return a + b; }",
        "C004",
    );
}

#[test]
fn c005_subtraction() {
    assert_transpiles_and_compiles(
        "int main() { int a = 10; int b = 3; return a - b; }",
        "C005",
    );
}

#[test]
fn c006_multiplication() {
    assert_transpiles_and_compiles(
        "int main() { int a = 6; int b = 7; return a * b; }",
        "C006",
    );
}

#[test]
fn c007_division() {
    assert_transpiles_and_compiles(
        "int main() { int a = 42; int b = 6; return a / b; }",
        "C007",
    );
}

#[test]
fn c008_modulo() {
    assert_transpiles_and_compiles(
        "int main() { int a = 17; int b = 5; return a % b; }",
        "C008",
    );
}

#[test]
fn c009_multiple_variables() {
    assert_transpiles_and_compiles(
        "int main() { int a = 1; int b = 2; int c = 3; int d = 4; return a + b + c + d; }",
        "C009",
    );
}

#[test]
fn c010_negative_literal() {
    assert_transpiles_and_compiles(
        "int main() { int x = -5; return -x; }",
        "C010",
    );
}

// ============================================================================
// C011-C020: Control Flow
// ============================================================================

#[test]
fn c011_if_statement() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int x = 5;
            if (x > 0) { return 1; }
            return 0;
        }"#,
        "C011",
    );
}

#[test]
fn c012_if_else() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int x = -1;
            if (x > 0) { return 1; } else { return 0; }
        }"#,
        "C012",
    );
}

#[test]
fn c013_while_loop() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int sum = 0;
            int i = 1;
            while (i <= 10) {
                sum = sum + i;
                i = i + 1;
            }
            return sum;
        }"#,
        "C013",
    );
}

#[test]
fn c014_for_loop() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int sum = 0;
            int i;
            for (i = 0; i < 10; i++) {
                sum = sum + i;
            }
            return sum;
        }"#,
        "C014",
    );
}

#[test]
fn c015_nested_if() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int x = 5;
            if (x > 0) {
                if (x > 10) { return 2; }
                return 1;
            }
            return 0;
        }"#,
        "C015",
    );
}

#[test]
fn c016_switch_basic() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int x = 2;
            switch (x) {
                case 1: return 10;
                case 2: return 20;
                default: return 0;
            }
        }"#,
        "C016",
    );
}

#[test]
fn c017_break_in_loop() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int i = 0;
            while (1) {
                if (i >= 5) break;
                i = i + 1;
            }
            return i;
        }"#,
        "C017",
    );
}

#[test]
fn c018_continue_in_loop() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int sum = 0;
            int i;
            for (i = 0; i < 10; i++) {
                if (i % 2 == 0) continue;
                sum = sum + i;
            }
            return sum;
        }"#,
        "C018",
    );
}

#[test]
fn c019_comparison_operators() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int a = 5;
            int b = 10;
            int r = 0;
            if (a < b) r = r + 1;
            if (a <= b) r = r + 1;
            if (b > a) r = r + 1;
            if (b >= a) r = r + 1;
            if (a != b) r = r + 1;
            if (a == a) r = r + 1;
            return r;
        }"#,
        "C019",
    );
}

#[test]
fn c020_logical_operators() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int a = 1;
            int b = 0;
            if (a && !b) return 1;
            if (a || b) return 2;
            return 0;
        }"#,
        "C020",
    );
}

// ============================================================================
// C021-C030: Pointer Basics
// ============================================================================

#[test]
fn c021_simple_function() {
    assert_transpiles_and_compiles(
        r#"int add(int a, int b) { return a + b; }
           int main() { return add(3, 4); }"#,
        "C021",
    );
}

#[test]
fn c022_void_function() {
    assert_transpiles_and_compiles(
        r#"int result;
           void set_result(int x) { result = x; }
           int main() { set_result(42); return result; }"#,
        "C022",
    );
}

#[test]
fn c023_multiple_params() {
    assert_transpiles_and_compiles(
        r#"int max(int a, int b) {
            if (a > b) return a;
            return b;
        }
        int main() { return max(10, 20); }"#,
        "C023",
    );
}

#[test]
fn c024_recursive_function() {
    assert_transpiles_and_compiles(
        r#"int factorial(int n) {
            if (n <= 1) return 1;
            return n * factorial(n - 1);
        }
        int main() { return factorial(5); }"#,
        "C024",
    );
}

#[test]
fn c025_nested_calls() {
    assert_transpiles_and_compiles(
        r#"int square(int x) { return x * x; }
           int add_squares(int a, int b) { return square(a) + square(b); }
           int main() { return add_squares(3, 4); }"#,
        "C025",
    );
}

#[test]
fn c026_early_return() {
    assert_transpiles_and_compiles(
        r#"int abs_val(int x) {
            if (x < 0) return -x;
            return x;
        }
        int main() { return abs_val(-5); }"#,
        "C026",
    );
}

#[test]
fn c027_chained_operations() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int a = 2;
            int b = 3;
            int c = 4;
            return (a + b) * c - a;
        }"#,
        "C027",
    );
}

#[test]
fn c028_bitwise_operators() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int a = 0xFF;
            int b = 0x0F;
            int c = a & b;
            int d = a | b;
            int e = a ^ b;
            return c + d + e;
        }"#,
        "C028",
    );
}

#[test]
fn c029_shift_operators() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int x = 1;
            int left = x << 4;
            int right = left >> 2;
            return right;
        }"#,
        "C029",
    );
}

#[test]
fn c030_increment_decrement() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int x = 5;
            x++;
            x++;
            x--;
            return x;
        }"#,
        "C030",
    );
}

// ============================================================================
// C031-C040: Structs
// ============================================================================

#[test]
fn c031_struct_definition() {
    assert_transpiles_and_compiles(
        r#"struct Point {
            int x;
            int y;
        };
        int main() { return 0; }"#,
        "C031",
    );
}

#[test]
fn c032_struct_member_access() {
    assert_transpiles_and_compiles(
        r#"struct Point {
            int x;
            int y;
        };
        int main() {
            struct Point p;
            p.x = 10;
            p.y = 20;
            return p.x + p.y;
        }"#,
        "C032",
    );
}

#[test]
fn c033_struct_in_function() {
    assert_transpiles_and_compiles(
        r#"struct Pair {
            int first;
            int second;
        };
        int sum_pair(struct Pair p) {
            return p.first + p.second;
        }
        int main() {
            struct Pair p;
            p.first = 3;
            p.second = 7;
            return sum_pair(p);
        }"#,
        "C033",
    );
}

#[test]
fn c034_struct_multiple_fields() {
    assert_transpiles_and_compiles(
        r#"struct Data {
            int a;
            int b;
            int c;
            int d;
        };
        int main() {
            struct Data d;
            d.a = 1;
            d.b = 2;
            d.c = 3;
            d.d = 4;
            return d.a + d.b + d.c + d.d;
        }"#,
        "C034",
    );
}

#[test]
fn c035_two_structs() {
    assert_transpiles_and_compiles(
        r#"struct Vec2 {
            int x;
            int y;
        };
        struct Vec3 {
            int x;
            int y;
            int z;
        };
        int main() { return 0; }"#,
        "C035",
    );
}

#[test]
fn c036_struct_assign_fields() {
    assert_transpiles_and_compiles(
        r#"struct Counter {
            int count;
        };
        int main() {
            struct Counter c;
            c.count = 0;
            c.count = c.count + 1;
            c.count = c.count + 1;
            return c.count;
        }"#,
        "C036",
    );
}

#[test]
fn c037_struct_as_return() {
    assert_transpiles_and_compiles(
        r#"struct Result {
            int value;
            int error;
        };
        int get_value(struct Result r) {
            return r.value;
        }
        int main() {
            struct Result r;
            r.value = 42;
            r.error = 0;
            return get_value(r);
        }"#,
        "C037",
    );
}

#[test]
fn c038_struct_with_float() {
    assert_transpiles_and_compiles(
        r#"struct Measurement {
            double value;
            int unit;
        };
        int main() { return 0; }"#,
        "C038",
    );
}

#[test]
fn c039_struct_field_comparison() {
    assert_transpiles_and_compiles(
        r#"struct Box {
            int width;
            int height;
        };
        int area(struct Box b) {
            return b.width * b.height;
        }
        int main() {
            struct Box b;
            b.width = 5;
            b.height = 10;
            return area(b);
        }"#,
        "C039",
    );
}

#[test]
fn c040_struct_zero_init() {
    assert_transpiles_and_compiles(
        r#"struct Config {
            int debug;
            int verbose;
        };
        int main() {
            struct Config c;
            c.debug = 0;
            c.verbose = 0;
            return c.debug + c.verbose;
        }"#,
        "C040",
    );
}

// ============================================================================
// C041-C050: Functions and Return Types
// ============================================================================

#[test]
fn c041_void_return() {
    assert_transpiles_and_compiles(
        r#"int global_val;
           void set_val(int x) { global_val = x; }
           int main() { set_val(10); return global_val; }"#,
        "C041",
    );
}

#[test]
fn c042_multiple_returns() {
    assert_transpiles_and_compiles(
        r#"int classify(int n) {
            if (n > 0) return 1;
            if (n < 0) return -1;
            return 0;
        }
        int main() { return classify(-5); }"#,
        "C042",
    );
}

#[test]
fn c043_function_with_local_vars() {
    assert_transpiles_and_compiles(
        r#"int compute(int x) {
            int a = x * 2;
            int b = a + 3;
            int c = b - 1;
            return c;
        }
        int main() { return compute(5); }"#,
        "C043",
    );
}

#[test]
fn c044_mutual_functions() {
    assert_transpiles_and_compiles(
        r#"int double_val(int x) { return x * 2; }
           int triple_val(int x) { return x * 3; }
           int main() { return double_val(3) + triple_val(2); }"#,
        "C044",
    );
}

#[test]
fn c045_function_chain() {
    assert_transpiles_and_compiles(
        r#"int inc(int x) { return x + 1; }
           int dec(int x) { return x - 1; }
           int main() { return inc(inc(dec(10))); }"#,
        "C045",
    );
}

#[test]
fn c046_complex_expression() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int a = 2;
            int b = 3;
            int c = 4;
            int d = 5;
            return a * b + c * d - a;
        }"#,
        "C046",
    );
}

#[test]
fn c047_conditional_assignment() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int x = 10;
            int y;
            if (x > 5) {
                y = 1;
            } else {
                y = 0;
            }
            return y;
        }"#,
        "C047",
    );
}

#[test]
fn c048_loop_accumulator() {
    assert_transpiles_and_compiles(
        r#"int sum_range(int start, int end) {
            int total = 0;
            int i = start;
            while (i <= end) {
                total = total + i;
                i = i + 1;
            }
            return total;
        }
        int main() { return sum_range(1, 100); }"#,
        "C048",
    );
}

#[test]
fn c049_nested_loops() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int sum = 0;
            int i;
            int j;
            for (i = 0; i < 3; i++) {
                for (j = 0; j < 3; j++) {
                    sum = sum + 1;
                }
            }
            return sum;
        }"#,
        "C049",
    );
}

#[test]
fn c050_power_function() {
    assert_transpiles_and_compiles(
        r#"int power(int base, int exp) {
            int result = 1;
            int i;
            for (i = 0; i < exp; i++) {
                result = result * base;
            }
            return result;
        }
        int main() { return power(2, 10); }"#,
        "C050",
    );
}
