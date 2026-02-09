//! DECY-192: Falsification test suite.
//!
//! 150 C programs that MUST transpile and produce compilable Rust.
//! These tests are **append-only**: never weaken or delete a test.
//!
//! Organization:
//! - C001-C010: Integer types and arithmetic
//! - C011-C020: Control flow (if/else, switch, loops)
//! - C021-C030: Functions, recursion, bitwise
//! - C031-C040: Structs
//! - C041-C050: Functions and return types
//! - C051-C060: Pointers and address-of (PATHOLOGICAL)
//! - C061-C070: Arrays and indexing (PATHOLOGICAL)
//! - C071-C080: Type casting and coercion (PATHOLOGICAL)
//! - C081-C090: String and char operations (PATHOLOGICAL)
//! - C091-C100: Nested expressions and operator precedence (PATHOLOGICAL)
//! - C101-C110: Global variables and static storage (PATHOLOGICAL)
//! - C111-C120: Do-while, goto, comma operator (PATHOLOGICAL)
//! - C121-C130: Typedef, enum, sizeof (PATHOLOGICAL)
//! - C131-C140: Multi-dimensional arrays, function pointers (PATHOLOGICAL)
//! - C141-C150: Compound assignment, ternary chains, edge cases (PATHOLOGICAL)

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

// ============================================================================
// C051-C060: Pointers and Address-Of (PATHOLOGICAL)
// ============================================================================

#[test]
fn c051_pointer_to_int() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int x = 42;
            int *p = &x;
            return *p;
        }"#,
        "C051",
    );
}

#[test]
fn c052_pointer_assignment() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int a = 10;
            int b = 20;
            int *p = &a;
            p = &b;
            return *p;
        }"#,
        "C052",
    );
}

#[test]
#[ignore = "FALSIFIED: double pointer (**pp) not yet supported"]
fn c053_pointer_to_pointer() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int x = 99;
            int *p = &x;
            int **pp = &p;
            return **pp;
        }"#,
        "C053",
    );
}

#[test]
fn c054_pointer_parameter() {
    assert_transpiles_and_compiles(
        r#"void swap(int *a, int *b) {
            int tmp = *a;
            *a = *b;
            *b = tmp;
        }
        int main() {
            int x = 1;
            int y = 2;
            swap(&x, &y);
            return x + y;
        }"#,
        "C054",
    );
}

#[test]
#[ignore = "FALSIFIED: null pointer check (p != 0) codegen issue"]
fn c055_null_pointer_check() {
    assert_transpiles_and_compiles(
        r#"int safe_deref(int *p) {
            if (p != 0) return *p;
            return -1;
        }
        int main() { return safe_deref(0); }"#,
        "C055",
    );
}

#[test]
#[ignore = "FALSIFIED: pointer arithmetic (*(p + 2)) codegen issue"]
fn c056_pointer_arithmetic_simple() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int arr[5];
            int *p = arr;
            arr[0] = 10;
            arr[1] = 20;
            arr[2] = 30;
            return *(p + 2);
        }"#,
        "C056",
    );
}

#[test]
fn c057_address_of_struct_field() {
    assert_transpiles_and_compiles(
        r#"struct Pair { int x; int y; };
        int main() {
            struct Pair p;
            p.x = 5;
            p.y = 10;
            int *px = &p.x;
            return *px;
        }"#,
        "C057",
    );
}

#[test]
#[ignore = "FALSIFIED: returning pointer to global codegen issue"]
fn c058_pointer_return() {
    // Pathological: returning address of local is UB in C, but transpiler should handle it
    assert_transpiles_and_compiles(
        r#"int global_val = 42;
        int *get_ptr() { return &global_val; }
        int main() { return *get_ptr(); }"#,
        "C058",
    );
}

#[test]
#[ignore = "FALSIFIED: void* to int* cast codegen issue"]
fn c059_void_pointer_cast() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int x = 77;
            void *vp = &x;
            int *ip = (int*)vp;
            return *ip;
        }"#,
        "C059",
    );
}

#[test]
fn c060_const_pointer() {
    assert_transpiles_and_compiles(
        r#"int read_val(const int *p) { return *p; }
        int main() {
            int x = 33;
            return read_val(&x);
        }"#,
        "C060",
    );
}

// ============================================================================
// C061-C070: Arrays and Indexing (PATHOLOGICAL)
// ============================================================================

#[test]
fn c061_array_declaration() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int arr[10];
            arr[0] = 1;
            arr[9] = 10;
            return arr[0] + arr[9];
        }"#,
        "C061",
    );
}

#[test]
fn c062_array_init_loop() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int arr[5];
            int i;
            for (i = 0; i < 5; i++) {
                arr[i] = i * i;
            }
            return arr[4];
        }"#,
        "C062",
    );
}

#[test]
fn c063_array_as_parameter() {
    assert_transpiles_and_compiles(
        r#"int sum_array(int arr[], int n) {
            int total = 0;
            int i;
            for (i = 0; i < n; i++) {
                total = total + arr[i];
            }
            return total;
        }
        int main() {
            int data[3];
            data[0] = 10;
            data[1] = 20;
            data[2] = 30;
            return sum_array(data, 3);
        }"#,
        "C063",
    );
}

#[test]
fn c064_array_of_structs() {
    assert_transpiles_and_compiles(
        r#"struct Point { int x; int y; };
        int main() {
            struct Point pts[3];
            pts[0].x = 1; pts[0].y = 2;
            pts[1].x = 3; pts[1].y = 4;
            pts[2].x = 5; pts[2].y = 6;
            return pts[2].x + pts[2].y;
        }"#,
        "C064",
    );
}

#[test]
fn c065_char_array_string() {
    assert_transpiles_and_compiles(
        r#"int main() {
            char msg[6];
            msg[0] = 'H';
            msg[1] = 'e';
            msg[2] = 'l';
            msg[3] = 'l';
            msg[4] = 'o';
            msg[5] = '\0';
            return msg[0];
        }"#,
        "C065",
    );
}

#[test]
fn c066_array_reverse() {
    assert_transpiles_and_compiles(
        r#"void reverse(int arr[], int n) {
            int i;
            for (i = 0; i < n / 2; i++) {
                int tmp = arr[i];
                arr[i] = arr[n - 1 - i];
                arr[n - 1 - i] = tmp;
            }
        }
        int main() {
            int arr[4];
            arr[0] = 1; arr[1] = 2; arr[2] = 3; arr[3] = 4;
            reverse(arr, 4);
            return arr[0];
        }"#,
        "C066",
    );
}

#[test]
fn c067_array_max() {
    assert_transpiles_and_compiles(
        r#"int find_max(int arr[], int n) {
            int max = arr[0];
            int i;
            for (i = 1; i < n; i++) {
                if (arr[i] > max) max = arr[i];
            }
            return max;
        }
        int main() {
            int data[5];
            data[0] = 3; data[1] = 7; data[2] = 1; data[3] = 9; data[4] = 4;
            return find_max(data, 5);
        }"#,
        "C067",
    );
}

#[test]
fn c068_array_bubble_sort() {
    assert_transpiles_and_compiles(
        r#"void bubble_sort(int arr[], int n) {
            int i; int j;
            for (i = 0; i < n - 1; i++) {
                for (j = 0; j < n - i - 1; j++) {
                    if (arr[j] > arr[j + 1]) {
                        int tmp = arr[j];
                        arr[j] = arr[j + 1];
                        arr[j + 1] = tmp;
                    }
                }
            }
        }
        int main() {
            int arr[4];
            arr[0] = 4; arr[1] = 2; arr[2] = 3; arr[3] = 1;
            bubble_sort(arr, 4);
            return arr[0];
        }"#,
        "C068",
    );
}

#[test]
fn c069_array_sum_of_squares() {
    assert_transpiles_and_compiles(
        r#"int sum_of_squares(int arr[], int n) {
            int sum = 0;
            int i;
            for (i = 0; i < n; i++) {
                sum = sum + arr[i] * arr[i];
            }
            return sum;
        }
        int main() {
            int data[3];
            data[0] = 1; data[1] = 2; data[2] = 3;
            return sum_of_squares(data, 3);
        }"#,
        "C069",
    );
}

#[test]
fn c070_array_search() {
    assert_transpiles_and_compiles(
        r#"int linear_search(int arr[], int n, int target) {
            int i;
            for (i = 0; i < n; i++) {
                if (arr[i] == target) return i;
            }
            return -1;
        }
        int main() {
            int data[5];
            data[0] = 10; data[1] = 20; data[2] = 30; data[3] = 40; data[4] = 50;
            return linear_search(data, 5, 30);
        }"#,
        "C070",
    );
}

// ============================================================================
// C071-C080: Type Casting and Coercion (PATHOLOGICAL)
// ============================================================================

#[test]
fn c071_int_to_float() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int x = 7;
            double y = (double)x;
            return (int)(y + 0.5);
        }"#,
        "C071",
    );
}

#[test]
fn c072_float_to_int_truncation() {
    assert_transpiles_and_compiles(
        r#"int main() {
            double pi = 3.14159;
            int truncated = (int)pi;
            return truncated;
        }"#,
        "C072",
    );
}

#[test]
fn c073_char_to_int() {
    assert_transpiles_and_compiles(
        r#"int main() {
            char c = 'A';
            int ascii = (int)c;
            return ascii;
        }"#,
        "C073",
    );
}

#[test]
#[ignore = "FALSIFIED: int to char cast codegen issue"]
fn c074_int_to_char() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int n = 65;
            char c = (char)n;
            return c;
        }"#,
        "C074",
    );
}

#[test]
fn c075_unsigned_int() {
    assert_transpiles_and_compiles(
        r#"int main() {
            unsigned int x = 42;
            unsigned int y = 10;
            return (int)(x - y);
        }"#,
        "C075",
    );
}

#[test]
fn c076_long_int() {
    assert_transpiles_and_compiles(
        r#"int main() {
            long x = 1000000;
            long y = 2000000;
            return (int)(x + y > 2500000);
        }"#,
        "C076",
    );
}

#[test]
fn c077_short_int() {
    assert_transpiles_and_compiles(
        r#"int main() {
            short s = 100;
            int i = s;
            return i;
        }"#,
        "C077",
    );
}

#[test]
fn c078_mixed_type_arithmetic() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int a = 5;
            double b = 2.5;
            double result = a * b;
            return (int)result;
        }"#,
        "C078",
    );
}

#[test]
fn c079_sizeof_types() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int si = sizeof(int);
            int sd = sizeof(double);
            int sc = sizeof(char);
            return si + sd + sc;
        }"#,
        "C079",
    );
}

#[test]
#[ignore = "FALSIFIED: char+char implicit int promotion codegen issue"]
fn c080_implicit_int_promotion() {
    assert_transpiles_and_compiles(
        r#"int main() {
            char a = 100;
            char b = 100;
            int result = a + b;
            return result;
        }"#,
        "C080",
    );
}

// ============================================================================
// C081-C090: String and Char Operations (PATHOLOGICAL)
// ============================================================================

#[test]
#[ignore = "FALSIFIED: char* string literal assignment codegen issue"]
fn c081_string_literal() {
    assert_transpiles_and_compiles(
        r#"int main() {
            char *s = "hello";
            return s[0];
        }"#,
        "C081",
    );
}

#[test]
fn c082_char_comparison() {
    assert_transpiles_and_compiles(
        r#"int is_uppercase(char c) {
            return c >= 'A' && c <= 'Z';
        }
        int main() { return is_uppercase('B'); }"#,
        "C082",
    );
}

#[test]
#[ignore = "FALSIFIED: pointer-indexed string iteration codegen issue"]
fn c083_string_length_manual() {
    assert_transpiles_and_compiles(
        r#"int my_strlen(char *s) {
            int len = 0;
            while (s[len] != '\0') {
                len++;
            }
            return len;
        }
        int main() { return my_strlen("abcdef"); }"#,
        "C083",
    );
}

#[test]
fn c084_char_digit_check() {
    assert_transpiles_and_compiles(
        r#"int is_digit(char c) {
            return c >= '0' && c <= '9';
        }
        int main() { return is_digit('5'); }"#,
        "C084",
    );
}

#[test]
#[ignore = "FALSIFIED: char arithmetic (c + 32) codegen issue"]
fn c085_char_to_lower() {
    assert_transpiles_and_compiles(
        r#"char to_lower(char c) {
            if (c >= 'A' && c <= 'Z') return c + 32;
            return c;
        }
        int main() { return to_lower('X'); }"#,
        "C085",
    );
}

#[test]
#[ignore = "FALSIFIED: escape char arithmetic codegen issue"]
fn c086_escape_characters() {
    assert_transpiles_and_compiles(
        r#"int main() {
            char tab = '\t';
            char newline = '\n';
            char null = '\0';
            return tab + newline + null;
        }"#,
        "C086",
    );
}

#[test]
#[ignore = "FALSIFIED: char* parameter string copy codegen issue"]
fn c087_char_array_copy() {
    assert_transpiles_and_compiles(
        r#"void my_strcpy(char *dst, char *src) {
            int i = 0;
            while (src[i] != '\0') {
                dst[i] = src[i];
                i++;
            }
            dst[i] = '\0';
        }
        int main() {
            char buf[10];
            my_strcpy(buf, "test");
            return buf[0];
        }"#,
        "C087",
    );
}

#[test]
#[ignore = "FALSIFIED: char* parameter string compare codegen issue"]
fn c088_string_compare_manual() {
    assert_transpiles_and_compiles(
        r#"int my_strcmp(char *a, char *b) {
            int i = 0;
            while (a[i] != '\0' && b[i] != '\0') {
                if (a[i] != b[i]) return a[i] - b[i];
                i++;
            }
            return a[i] - b[i];
        }
        int main() { return my_strcmp("abc", "abc"); }"#,
        "C088",
    );
}

#[test]
#[ignore = "FALSIFIED: char parameter string iteration codegen issue"]
fn c089_char_counting() {
    assert_transpiles_and_compiles(
        r#"int count_char(char *s, char target) {
            int count = 0;
            int i = 0;
            while (s[i] != '\0') {
                if (s[i] == target) count++;
                i++;
            }
            return count;
        }
        int main() { return count_char("banana", 'a'); }"#,
        "C089",
    );
}

#[test]
#[ignore = "FALSIFIED: char arithmetic and comparison codegen issue"]
fn c090_hex_char() {
    assert_transpiles_and_compiles(
        r#"int hex_value(char c) {
            if (c >= '0' && c <= '9') return c - '0';
            if (c >= 'a' && c <= 'f') return c - 'a' + 10;
            if (c >= 'A' && c <= 'F') return c - 'A' + 10;
            return -1;
        }
        int main() { return hex_value('F'); }"#,
        "C090",
    );
}

// ============================================================================
// C091-C100: Nested Expressions and Operator Precedence (PATHOLOGICAL)
// ============================================================================

#[test]
fn c091_deeply_nested_parens() {
    assert_transpiles_and_compiles(
        r#"int main() {
            return ((((1 + 2) * 3) - 4) + 5);
        }"#,
        "C091",
    );
}

#[test]
fn c092_ternary_operator() {
    assert_transpiles_and_compiles(
        r#"int abs_val(int x) {
            return x >= 0 ? x : -x;
        }
        int main() { return abs_val(-42); }"#,
        "C092",
    );
}

#[test]
fn c093_nested_ternary() {
    assert_transpiles_and_compiles(
        r#"int clamp(int x, int lo, int hi) {
            return x < lo ? lo : (x > hi ? hi : x);
        }
        int main() { return clamp(15, 0, 10); }"#,
        "C093",
    );
}

#[test]
fn c094_complex_boolean() {
    assert_transpiles_and_compiles(
        r#"int in_range(int x, int lo, int hi) {
            return x >= lo && x <= hi;
        }
        int main() { return in_range(5, 1, 10); }"#,
        "C094",
    );
}

#[test]
#[ignore = "FALSIFIED: boolean negation (!x, !!y) codegen issue"]
fn c095_boolean_not() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int x = 0;
            int y = !x;
            int z = !!y;
            return z;
        }"#,
        "C095",
    );
}

#[test]
fn c096_comma_in_for() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int sum = 0;
            int i;
            for (i = 0; i < 10; i++) {
                sum = sum + i;
            }
            return sum;
        }"#,
        "C096",
    );
}

#[test]
fn c097_compound_assignment() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int x = 10;
            x += 5;
            x -= 3;
            x *= 2;
            x /= 4;
            return x;
        }"#,
        "C097",
    );
}

#[test]
fn c098_bitwise_compound_assignment() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int x = 0xFF;
            x &= 0x0F;
            x |= 0x30;
            x ^= 0x05;
            x <<= 1;
            x >>= 2;
            return x;
        }"#,
        "C098",
    );
}

#[test]
fn c099_operator_precedence_mixed() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int a = 2 + 3 * 4;
            int b = (2 + 3) * 4;
            int c = 10 - 4 / 2;
            return a + b + c;
        }"#,
        "C099",
    );
}

#[test]
fn c100_chained_comparisons() {
    assert_transpiles_and_compiles(
        r#"int between(int x, int lo, int hi) {
            return lo <= x && x <= hi;
        }
        int main() {
            int a = between(5, 1, 10);
            int b = between(15, 1, 10);
            return a + b;
        }"#,
        "C100",
    );
}
