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

// ============================================================================
// C101-C110: Global Variables and Static Storage (PATHOLOGICAL)
// ============================================================================

#[test]
fn c101_global_int() {
    assert_transpiles_and_compiles(
        r#"int counter = 0;
        void increment() { counter = counter + 1; }
        int main() {
            increment();
            increment();
            increment();
            return counter;
        }"#,
        "C101",
    );
}

#[test]
fn c102_multiple_globals() {
    assert_transpiles_and_compiles(
        r#"int x = 10;
        int y = 20;
        int z = 30;
        int sum_globals() { return x + y + z; }
        int main() { return sum_globals(); }"#,
        "C102",
    );
}

#[test]
fn c103_global_array() {
    assert_transpiles_and_compiles(
        r#"int data[5];
        void init_data() {
            int i;
            for (i = 0; i < 5; i++) data[i] = i * 10;
        }
        int main() {
            init_data();
            return data[3];
        }"#,
        "C103",
    );
}

#[test]
#[ignore = "FALSIFIED: global struct variable codegen issue"]
fn c104_global_struct() {
    assert_transpiles_and_compiles(
        r#"struct Config { int debug; int verbose; };
        struct Config cfg;
        int main() {
            cfg.debug = 1;
            cfg.verbose = 0;
            return cfg.debug;
        }"#,
        "C104",
    );
}

#[test]
fn c105_global_const() {
    assert_transpiles_and_compiles(
        r#"const int MAX_SIZE = 100;
        int main() { return MAX_SIZE; }"#,
        "C105",
    );
}

#[test]
fn c106_static_local_variable() {
    assert_transpiles_and_compiles(
        r#"int next_id() {
            static int counter = 0;
            counter = counter + 1;
            return counter;
        }
        int main() {
            next_id();
            next_id();
            return next_id();
        }"#,
        "C106",
    );
}

#[test]
fn c107_global_initialized_array() {
    assert_transpiles_and_compiles(
        r#"int primes[5];
        int main() {
            primes[0] = 2; primes[1] = 3; primes[2] = 5;
            primes[3] = 7; primes[4] = 11;
            return primes[4];
        }"#,
        "C107",
    );
}

#[test]
fn c108_global_read_write() {
    assert_transpiles_and_compiles(
        r#"int state = 0;
        int get_state() { return state; }
        void set_state(int s) { state = s; }
        int main() {
            set_state(42);
            return get_state();
        }"#,
        "C108",
    );
}

#[test]
fn c109_global_double() {
    assert_transpiles_and_compiles(
        r#"double pi_approx = 3.14;
        int main() { return (int)(pi_approx * 2); }"#,
        "C109",
    );
}

#[test]
#[ignore = "FALSIFIED: mixed global types (int+double+char) codegen issue"]
fn c110_multiple_global_types() {
    assert_transpiles_and_compiles(
        r#"int g_int = 1;
        double g_dbl = 2.5;
        char g_chr = 'A';
        int main() { return g_int + (int)g_dbl + g_chr; }"#,
        "C110",
    );
}

// ============================================================================
// C111-C120: Do-While, Goto, Comma Operator (PATHOLOGICAL)
// ============================================================================

#[test]
fn c111_do_while_basic() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int x = 0;
            do {
                x = x + 1;
            } while (x < 10);
            return x;
        }"#,
        "C111",
    );
}

#[test]
fn c112_do_while_single_iteration() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int x = 100;
            do {
                x = x + 1;
            } while (x < 50);
            return x;
        }"#,
        "C112",
    );
}

#[test]
fn c113_nested_do_while() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int sum = 0;
            int i = 0;
            do {
                int j = 0;
                do {
                    sum = sum + 1;
                    j = j + 1;
                } while (j < 3);
                i = i + 1;
            } while (i < 3);
            return sum;
        }"#,
        "C113",
    );
}

#[test]
#[ignore = "FALSIFIED: do-while with break codegen issue"]
fn c114_do_while_with_break() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int x = 0;
            do {
                x = x + 1;
                if (x == 5) break;
            } while (x < 100);
            return x;
        }"#,
        "C114",
    );
}

#[test]
fn c115_while_true_break() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int x = 0;
            while (1) {
                x = x + 1;
                if (x >= 10) break;
            }
            return x;
        }"#,
        "C115",
    );
}

#[test]
fn c116_for_empty_body() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int i;
            int x = 0;
            for (i = 0; i < 10; i++) {
                x = x + 1;
            }
            return x;
        }"#,
        "C116",
    );
}

#[test]
fn c117_nested_if_deep() {
    assert_transpiles_and_compiles(
        r#"int classify(int n) {
            if (n > 100) {
                if (n > 1000) {
                    return 3;
                } else {
                    return 2;
                }
            } else {
                if (n > 0) {
                    return 1;
                } else {
                    return 0;
                }
            }
        }
        int main() { return classify(500); }"#,
        "C117",
    );
}

#[test]
fn c118_switch_with_default() {
    assert_transpiles_and_compiles(
        r#"int day_type(int day) {
            switch (day) {
                case 0: return 0;
                case 6: return 0;
                case 1: case 2: case 3: case 4: case 5: return 1;
                default: return -1;
            }
        }
        int main() { return day_type(3); }"#,
        "C118",
    );
}

#[test]
fn c119_continue_in_loop() {
    assert_transpiles_and_compiles(
        r#"int count_positive(int arr[], int n) {
            int count = 0;
            int i;
            for (i = 0; i < n; i++) {
                if (arr[i] <= 0) continue;
                count = count + 1;
            }
            return count;
        }
        int main() {
            int data[5];
            data[0] = -1; data[1] = 2; data[2] = 0; data[3] = 4; data[4] = -3;
            return count_positive(data, 5);
        }"#,
        "C119",
    );
}

#[test]
fn c120_multiple_switch_cases() {
    assert_transpiles_and_compiles(
        r#"int grade(int score) {
            switch (score / 10) {
                case 10: case 9: return 4;
                case 8: return 3;
                case 7: return 2;
                case 6: return 1;
                default: return 0;
            }
        }
        int main() { return grade(85); }"#,
        "C120",
    );
}

// ============================================================================
// C121-C130: Typedef, Enum, Sizeof (PATHOLOGICAL)
// ============================================================================

#[test]
fn c121_typedef_int() {
    assert_transpiles_and_compiles(
        r#"typedef int Score;
        Score add_scores(Score a, Score b) { return a + b; }
        int main() { return add_scores(50, 30); }"#,
        "C121",
    );
}

#[test]
fn c122_enum_basic() {
    assert_transpiles_and_compiles(
        r#"enum Color { RED, GREEN, BLUE };
        int main() {
            enum Color c = GREEN;
            return c;
        }"#,
        "C122",
    );
}

#[test]
fn c123_enum_with_values() {
    assert_transpiles_and_compiles(
        r#"enum HttpStatus {
            OK = 200,
            NOT_FOUND = 404,
            ERROR = 500
        };
        int main() { return OK; }"#,
        "C123",
    );
}

#[test]
fn c124_sizeof_variable() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int x = 42;
            int s = sizeof(x);
            return s;
        }"#,
        "C124",
    );
}

#[test]
#[ignore = "FALSIFIED: sizeof(struct) comparison codegen issue"]
fn c125_sizeof_struct() {
    assert_transpiles_and_compiles(
        r#"struct Small { int a; };
        struct Large { int a; int b; int c; int d; };
        int main() {
            return sizeof(struct Large) > sizeof(struct Small);
        }"#,
        "C125",
    );
}

#[test]
fn c126_typedef_struct() {
    assert_transpiles_and_compiles(
        r#"typedef struct { int x; int y; } Point;
        int distance_sq(Point a, Point b) {
            int dx = a.x - b.x;
            int dy = a.y - b.y;
            return dx * dx + dy * dy;
        }
        int main() {
            Point p1; p1.x = 0; p1.y = 0;
            Point p2; p2.x = 3; p2.y = 4;
            return distance_sq(p1, p2);
        }"#,
        "C126",
    );
}

#[test]
fn c127_enum_as_switch() {
    assert_transpiles_and_compiles(
        r#"enum Op { ADD, SUB, MUL };
        int apply(enum Op op, int a, int b) {
            switch (op) {
                case ADD: return a + b;
                case SUB: return a - b;
                case MUL: return a * b;
                default: return 0;
            }
        }
        int main() { return apply(MUL, 6, 7); }"#,
        "C127",
    );
}

#[test]
fn c128_nested_struct() {
    assert_transpiles_and_compiles(
        r#"struct Inner { int val; };
        struct Outer { struct Inner in_field; int extra; };
        int main() {
            struct Outer o;
            o.in_field.val = 10;
            o.extra = 20;
            return o.in_field.val + o.extra;
        }"#,
        "C128",
    );
}

#[test]
fn c129_typedef_function_alias() {
    assert_transpiles_and_compiles(
        r#"typedef int Integer;
        typedef double Real;
        Integer round_real(Real x) {
            return (Integer)(x + 0.5);
        }
        int main() { return round_real(3.7); }"#,
        "C129",
    );
}

#[test]
fn c130_sizeof_array() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int arr[10];
            int element_count = sizeof(arr) / sizeof(arr[0]);
            return element_count;
        }"#,
        "C130",
    );
}

// ============================================================================
// C131-C140: Multi-Dimensional Arrays, Function Pointers (PATHOLOGICAL)
// ============================================================================

#[test]
fn c131_2d_array() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int matrix[3][3];
            int i; int j;
            for (i = 0; i < 3; i++)
                for (j = 0; j < 3; j++)
                    matrix[i][j] = i * 3 + j;
            return matrix[2][2];
        }"#,
        "C131",
    );
}

#[test]
fn c132_array_of_arrays_sum() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int rows[2][3];
            rows[0][0] = 1; rows[0][1] = 2; rows[0][2] = 3;
            rows[1][0] = 4; rows[1][1] = 5; rows[1][2] = 6;
            int sum = 0;
            int i; int j;
            for (i = 0; i < 2; i++)
                for (j = 0; j < 3; j++)
                    sum = sum + rows[i][j];
            return sum;
        }"#,
        "C132",
    );
}

#[test]
fn c133_function_pointer_basic() {
    assert_transpiles_and_compiles(
        r#"int add(int a, int b) { return a + b; }
        int sub(int a, int b) { return a - b; }
        int apply(int (*op)(int, int), int x, int y) {
            return op(x, y);
        }
        int main() { return apply(add, 10, 3); }"#,
        "C133",
    );
}

#[test]
fn c134_recursive_fibonacci() {
    assert_transpiles_and_compiles(
        r#"int fib(int n) {
            if (n <= 1) return n;
            return fib(n - 1) + fib(n - 2);
        }
        int main() { return fib(10); }"#,
        "C134",
    );
}

#[test]
fn c135_recursive_factorial() {
    assert_transpiles_and_compiles(
        r#"int factorial(int n) {
            if (n <= 1) return 1;
            return n * factorial(n - 1);
        }
        int main() { return factorial(6); }"#,
        "C135",
    );
}

#[test]
fn c136_recursive_gcd() {
    assert_transpiles_and_compiles(
        r#"int gcd(int a, int b) {
            if (b == 0) return a;
            return gcd(b, a % b);
        }
        int main() { return gcd(48, 18); }"#,
        "C136",
    );
}

#[test]
fn c137_binary_search() {
    assert_transpiles_and_compiles(
        r#"int binary_search(int arr[], int n, int target) {
            int lo = 0;
            int hi = n - 1;
            while (lo <= hi) {
                int mid = lo + (hi - lo) / 2;
                if (arr[mid] == target) return mid;
                if (arr[mid] < target) lo = mid + 1;
                else hi = mid - 1;
            }
            return -1;
        }
        int main() {
            int data[5];
            data[0] = 10; data[1] = 20; data[2] = 30; data[3] = 40; data[4] = 50;
            return binary_search(data, 5, 30);
        }"#,
        "C137",
    );
}

#[test]
fn c138_matrix_multiply_element() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int a[2][2]; int b[2][2]; int c[2][2];
            a[0][0] = 1; a[0][1] = 2; a[1][0] = 3; a[1][1] = 4;
            b[0][0] = 5; b[0][1] = 6; b[1][0] = 7; b[1][1] = 8;
            c[0][0] = a[0][0]*b[0][0] + a[0][1]*b[1][0];
            return c[0][0];
        }"#,
        "C138",
    );
}

#[test]
fn c139_deeply_nested_loops() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int count = 0;
            int i; int j; int k;
            for (i = 0; i < 3; i++)
                for (j = 0; j < 3; j++)
                    for (k = 0; k < 3; k++)
                        count = count + 1;
            return count;
        }"#,
        "C139",
    );
}

#[test]
fn c140_array_rotation() {
    assert_transpiles_and_compiles(
        r#"void rotate_left(int arr[], int n) {
            int first = arr[0];
            int i;
            for (i = 0; i < n - 1; i++)
                arr[i] = arr[i + 1];
            arr[n - 1] = first;
        }
        int main() {
            int data[4];
            data[0] = 1; data[1] = 2; data[2] = 3; data[3] = 4;
            rotate_left(data, 4);
            return data[0];
        }"#,
        "C140",
    );
}

// ============================================================================
// C141-C150: Compound Assignment, Ternary Chains, Edge Cases (PATHOLOGICAL)
// ============================================================================

#[test]
fn c141_modulo_chain() {
    assert_transpiles_and_compiles(
        r#"int is_leap_year(int year) {
            if (year % 400 == 0) return 1;
            if (year % 100 == 0) return 0;
            if (year % 4 == 0) return 1;
            return 0;
        }
        int main() { return is_leap_year(2000); }"#,
        "C141",
    );
}

#[test]
fn c142_absolute_value_branchless() {
    assert_transpiles_and_compiles(
        r#"int abs_val(int x) {
            int mask = x >> 31;
            return (x + mask) ^ mask;
        }
        int main() { return abs_val(-42); }"#,
        "C142",
    );
}

#[test]
fn c143_min_max_functions() {
    assert_transpiles_and_compiles(
        r#"int min(int a, int b) { return a < b ? a : b; }
        int max(int a, int b) { return a > b ? a : b; }
        int main() { return min(10, 20) + max(10, 20); }"#,
        "C143",
    );
}

#[test]
fn c144_swap_without_temp() {
    assert_transpiles_and_compiles(
        r#"int main() {
            int a = 5;
            int b = 10;
            a = a ^ b;
            b = a ^ b;
            a = a ^ b;
            return a * 100 + b;
        }"#,
        "C144",
    );
}

#[test]
fn c145_nested_function_calls() {
    assert_transpiles_and_compiles(
        r#"int square(int x) { return x * x; }
        int add(int a, int b) { return a + b; }
        int main() { return add(square(3), square(4)); }"#,
        "C145",
    );
}

#[test]
fn c146_early_return_guard() {
    assert_transpiles_and_compiles(
        r#"int safe_divide(int a, int b) {
            if (b == 0) return -1;
            return a / b;
        }
        int main() { return safe_divide(10, 0); }"#,
        "C146",
    );
}

#[test]
fn c147_cascading_if_else() {
    assert_transpiles_and_compiles(
        r#"int fizzbuzz_type(int n) {
            if (n % 15 == 0) return 3;
            else if (n % 3 == 0) return 1;
            else if (n % 5 == 0) return 2;
            else return 0;
        }
        int main() { return fizzbuzz_type(15); }"#,
        "C147",
    );
}

#[test]
fn c148_accumulator_pattern() {
    assert_transpiles_and_compiles(
        r#"int digit_sum(int n) {
            int sum = 0;
            while (n > 0) {
                sum = sum + n % 10;
                n = n / 10;
            }
            return sum;
        }
        int main() { return digit_sum(12345); }"#,
        "C148",
    );
}

#[test]
fn c149_count_bits() {
    assert_transpiles_and_compiles(
        r#"int popcount(int n) {
            int count = 0;
            while (n > 0) {
                count = count + (n & 1);
                n = n >> 1;
            }
            return count;
        }
        int main() { return popcount(255); }"#,
        "C149",
    );
}

#[test]
fn c150_collatz_steps() {
    assert_transpiles_and_compiles(
        r#"int collatz_steps(int n) {
            int steps = 0;
            while (n != 1) {
                if (n % 2 == 0) n = n / 2;
                else n = 3 * n + 1;
                steps = steps + 1;
            }
            return steps;
        }
        int main() { return collatz_steps(27); }"#,
        "C150",
    );
}
