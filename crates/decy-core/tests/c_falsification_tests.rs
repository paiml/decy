//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C001-C150: Systematic falsification of C language construct transpilation.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! Organization:
//! - C001-C015: Integer types and arithmetic (pathological)
//! - C016-C030: Control flow (pathological)
//! - C031-C055: Pointers and arrays (PATHOLOGICAL - key gap area)
//! - C056-C075: Structs, unions, enums
//! - C071-C090: Functions
//! - C091-C110: Standard library
//! - C111-C130: Preprocessor and advanced
//! - C131-C150: Real-world pathological patterns

use decy_core::ProjectContext;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper: Create a temporary C file with the given content.
fn create_temp_c_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).expect("Failed to write temp file");
    path
}

// ============================================================================
// C001-C015: Integer Types and Arithmetic (pathological edge cases)
// ============================================================================

#[test]
fn c001_integer_addition() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(&temp, "test.c", "int add(int a, int b) { return a + b; }");
    let context = ProjectContext::new();
    let result = decy_core::transpile_file(&file, &context);
    assert!(result.is_ok(), "C001: Integer addition should transpile");
    let output = result.unwrap();
    assert!(!output.rust_code.is_empty(), "C001: Output should not be empty");
    assert!(output.rust_code.contains("fn add"), "C001: Should contain function");
    assert!(
        !output.rust_code.contains("unsafe"),
        "C001: Should not need unsafe"
    );
}

#[test]
fn c002_integer_overflow() {
    let c_code = r#"
#include <limits.h>
int overflow() {
    int x = INT_MAX;
    int y = x + 1;
    return y;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C002: Integer overflow should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C002: Output should not be empty");
}

#[test]
fn c003_unsigned_underflow() {
    let c_code = r#"
unsigned int underflow() {
    unsigned int x = 0;
    unsigned int y = x - 1;
    return y;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C003: Unsigned underflow should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C003: Output should not be empty");
}

#[test]
fn c004_char_arithmetic() {
    let c_code = r#"
char next_char(char c) {
    return c + 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C004: Char arithmetic should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C004: Output should not be empty");
}

#[test]
fn c005_mixed_signed_unsigned_comparison() {
    let c_code = r#"
int mixed_compare(int a, unsigned int b) {
    if (a < 0) return -1;
    if ((unsigned int)a < b) return -1;
    if ((unsigned int)a > b) return 1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C005: Mixed signed/unsigned comparison should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C005: Output should not be empty");
}

#[test]
fn c006_bitwise_on_signed() {
    let c_code = r#"
int bitwise_signed(int a, int b) {
    int r1 = a & b;
    int r2 = a | b;
    int r3 = a ^ b;
    int r4 = ~a;
    return r1 + r2 + r3 + r4;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C006: Bitwise on signed should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C006: Output should not be empty");
}

#[test]
fn c007_integer_promotion() {
    let c_code = r#"
int promote(short a, short b) {
    return a + b;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C007: Integer promotion should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C007: Output should not be empty");
}

#[test]
fn c008_division_by_variable() {
    let c_code = r#"
int divide(int a, int b) {
    return a / b;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C008: Division by variable should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C008: Output should not be empty");
}

#[test]
fn c009_modulo_negative() {
    let c_code = r#"
int mod_neg(int a, int b) {
    return a % b;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C009: Modulo with negative should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C009: Output should not be empty");
}

#[test]
fn c010_shift_by_variable() {
    let c_code = r#"
int shift_var(int x, int n) {
    int left = x << n;
    int right = x >> n;
    return left + right;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C010: Shift by variable should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C010: Output should not be empty");
}

#[test]
fn c011_sizeof_arithmetic() {
    let c_code = r#"
int size_calc() {
    int x = 5;
    return sizeof(x + 3);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C011: Sizeof on expression should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C011: Output should not be empty");
}

#[test]
fn c012_comma_operator() {
    let c_code = r#"
int comma_op() {
    int a = (1, 2, 3);
    return a;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C012: Comma operator should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C012: Output should not be empty");
}

#[test]
fn c013_ternary_in_arithmetic() {
    let c_code = r#"
int ternary_calc(int x) {
    int result = x + (x > 0 ? 10 : -10);
    return result;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C013: Ternary in arithmetic should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C013: Output should not be empty");
}

#[test]
fn c014_pre_post_increment_in_expression() {
    let c_code = r#"
int inc_expr() {
    int x = 5;
    int a = ++x;
    int b = x--;
    return a + b;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C014: Pre/post increment in expression should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C014: Output should not be empty");
}

#[test]
fn c015_hex_octal_literals() {
    let c_code = r#"
int literals() {
    int hex = 0xFF;
    int oct = 077;
    int dec = 255;
    return hex + oct + dec;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C015: Hex/octal literals should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C015: Output should not be empty");
}

// ============================================================================
// C016-C030: Control Flow (pathological)
// ============================================================================

#[test]
fn c016_nested_if_else_chain() {
    let c_code = r#"
int classify(int x) {
    if (x > 100) {
        if (x > 200) {
            if (x > 300) {
                if (x > 400) {
                    if (x > 500) {
                        return 5;
                    }
                    return 4;
                }
                return 3;
            }
            return 2;
        }
        return 1;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C016: Nested if-else chain should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C016: Output should not be empty");
}

#[test]
fn c017_switch_fallthrough() {
    let c_code = r#"
int fallthrough(int x) {
    int result = 0;
    switch (x) {
        case 3: result += 3;
        case 2: result += 2;
        case 1: result += 1;
            break;
        default: result = -1;
    }
    return result;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C017: Switch fallthrough should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C017: Output should not be empty");
}

#[test]
fn c018_switch_on_enum() {
    let c_code = r#"
enum Color { RED, GREEN, BLUE };
int color_value(enum Color c) {
    switch (c) {
        case RED: return 0;
        case GREEN: return 1;
        case BLUE: return 2;
        default: return -1;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C018: Switch on enum should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C018: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c019_goto_forward() {
    let c_code = r#"
int forward_goto(int x) {
    if (x < 0) goto error;
    return x * 2;
error:
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C019: Forward goto should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C019: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c020_goto_backward_loop() {
    let c_code = r#"
int goto_loop(int n) {
    int i = 0;
    int sum = 0;
loop:
    if (i >= n) goto done;
    sum += i;
    i++;
    goto loop;
done:
    return sum;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C020: Backward goto should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C020: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c021_duffs_device() {
    let c_code = r#"
void duffs_copy(char *to, char *from, int count) {
    int n = (count + 7) / 8;
    switch (count % 8) {
    case 0: do { *to++ = *from++;
    case 7:      *to++ = *from++;
    case 6:      *to++ = *from++;
    case 5:      *to++ = *from++;
    case 4:      *to++ = *from++;
    case 3:      *to++ = *from++;
    case 2:      *to++ = *from++;
    case 1:      *to++ = *from++;
            } while (--n > 0);
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C021: Duff's device should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C021: Output should not be empty");
}

#[test]
fn c022_for_comma_init() {
    let c_code = r#"
int comma_for() {
    int sum = 0;
    int i, j;
    for (i = 0, j = 10; i < j; i++, j--) {
        sum += i + j;
    }
    return sum;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C022: For with comma init should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C022: Output should not be empty");
}

#[test]
fn c023_while_assignment_condition() {
    let c_code = r#"
int count_positive(int *arr, int n) {
    int count = 0;
    int i = 0;
    int val;
    while (i < n) {
        val = arr[i];
        if (val > 0) count++;
        i++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C023: While with assignment should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C023: Output should not be empty");
}

#[test]
fn c024_do_while_with_break() {
    let c_code = r#"
int find_first_neg(int *arr, int n) {
    int i = 0;
    do {
        if (arr[i] < 0) break;
        i++;
    } while (i < n);
    return i;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C024: Do-while with break should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C024: Output should not be empty");
}

#[test]
fn c025_nested_loops_break_continue() {
    let c_code = r#"
int nested_loops(int n) {
    int count = 0;
    int i = 0;
    while (i < n) {
        int j = 0;
        while (j < n) {
            if (j == i) { j++; continue; }
            if (i + j > n) break;
            count++;
            j++;
        }
        i++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C025: Nested loops with break/continue should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C025: Output should not be empty");
}

#[test]
fn c026_switch_inside_loop() {
    let c_code = r#"
int process(int *commands, int n) {
    int result = 0;
    int i = 0;
    while (i < n) {
        switch (commands[i]) {
            case 1: result += 10; break;
            case 2: result -= 5; break;
            case 3: result *= 2; break;
            default: break;
        }
        i++;
    }
    return result;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C026: Switch inside loop should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C026: Output should not be empty");
}

#[test]
fn c027_infinite_for_loop() {
    let c_code = r#"
int infinite_loop_break(int *arr, int max) {
    int i = 0;
    for (;;) {
        if (arr[i] == 0) return i;
        if (i >= max) return -1;
        i++;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C027: Infinite for loop should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C027: Output should not be empty");
}

#[test]
fn c028_empty_loop_body() {
    let c_code = r#"
int skip_whitespace(char *s) {
    int i = 0;
    while (s[i] == ' ') i++;
    return i;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C028: Empty loop body should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C028: Output should not be empty");
}

#[test]
fn c029_multiple_return_paths() {
    let c_code = r#"
int categorize(int x) {
    if (x < 0) return -1;
    if (x == 0) return 0;
    if (x < 10) return 1;
    if (x < 100) return 2;
    if (x < 1000) return 3;
    return 4;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C029: Multiple return paths should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C029: Output should not be empty");
}

#[test]
fn c030_early_return_nested_scope() {
    let c_code = r#"
int early_return(int *arr, int n) {
    int i = 0;
    while (i < n) {
        if (arr[i] < 0) {
            if (arr[i] == -1) {
                return -1;
            }
            return arr[i];
        }
        i++;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C030: Early return from nested scope should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C030: Output should not be empty");
}

// ============================================================================
// C031-C055: Pointers and Arrays (PATHOLOGICAL - key gap area)
// ============================================================================

#[test]
fn c031_basic_pointer_dereference() {
    let c_code = r#"
int deref(int *p) {
    return *p;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C031: Basic pointer dereference should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C031: Output should not be empty");
}

#[test]
fn c032_pointer_arithmetic() {
    let c_code = r#"
int get_nth(int *arr, int n) {
    int *p = arr + n;
    return *p;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C032: Pointer arithmetic should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C032: Output should not be empty");
}

#[test]
fn c033_pointer_to_pointer() {
    let c_code = r#"
int deref_pp(int **pp) {
    return **pp;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C033: Pointer-to-pointer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C033: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c034_triple_pointer() {
    let c_code = r#"
int deref_ppp(int ***ppp) {
    return ***ppp;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C034: Triple pointer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C034: Output should not be empty");
}

#[test]
fn c035_void_pointer_generic() {
    let c_code = r#"
void* identity(void *ptr) {
    return ptr;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C035: void* generic should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C035: Output should not be empty");
}

#[test]
fn c036_void_pointer_cast() {
    let c_code = r#"
int get_int(void *ptr) {
    int *ip = (int *)ptr;
    return *ip;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C036: void* cast should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C036: Output should not be empty");
}

#[test]
fn c037_array_decay_to_pointer() {
    let c_code = r#"
int first_element(int arr[]) {
    return arr[0];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C037: Array decay should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C037: Output should not be empty");
}

#[test]
fn c038_pointer_comparison() {
    let c_code = r#"
int count_elements(int *start, int *end) {
    int count = 0;
    int *p = start;
    while (p < end) {
        count++;
        p = p + 1;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C038: Pointer comparison should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C038: Output should not be empty");
}

#[test]
fn c039_null_pointer_check() {
    let c_code = r#"
int safe_deref(int *p) {
    if (p == 0) return -1;
    return *p;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C039: NULL pointer check should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C039: Output should not be empty");
}

#[test]
fn c040_pointer_subtraction() {
    let c_code = r#"
int distance(int *start, int *end) {
    return end - start;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C040: Pointer subtraction should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C040: Output should not be empty");
}

#[test]
fn c041_array_of_pointers() {
    let c_code = r#"
int deref_first(int *arr[], int n) {
    if (n > 0) return *arr[0];
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C041: Array of pointers should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C041: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c042_pointer_to_array() {
    let c_code = r#"
int get_elem(int (*p)[10], int i) {
    return (*p)[i];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C042: Pointer to array should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C042: Output should not be empty");
}

#[test]
fn c043_function_pointer() {
    let c_code = r#"
int apply(int (*f)(int), int x) {
    return f(x);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C043: Function pointer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C043: Output should not be empty");
}

#[test]
fn c044_function_pointer_array() {
    let c_code = r#"
typedef int (*op_func)(int, int);
int dispatch(op_func *table, int idx, int a, int b) {
    return table[idx](a, b);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C044: Function pointer array should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C044: Output should not be empty");
}

#[test]
fn c045_pointer_aliasing() {
    let c_code = r#"
void swap_via_alias(int *a, int *b) {
    int tmp = *a;
    *a = *b;
    *b = tmp;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C045: Pointer aliasing should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C045: Output should not be empty");
}

#[test]
fn c046_restrict_pointer() {
    let c_code = r#"
void copy_restrict(int * restrict dst, const int * restrict src, int n) {
    int i = 0;
    while (i < n) {
        dst[i] = src[i];
        i++;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C046: Restrict pointer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C046: Output should not be empty");
}

#[test]
fn c047_pointer_to_struct_member() {
    let c_code = r#"
struct Point { int x; int y; };
int get_x(struct Point *p) {
    return p->x;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C047: Pointer to struct member should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C047: Output should not be empty");
}

#[test]
fn c048_linked_list_traversal() {
    let c_code = r#"
struct Node { int val; struct Node *next; };
int count_nodes(struct Node *head) {
    int count = 0;
    struct Node *cur = head;
    while (cur != 0) {
        count++;
        cur = cur->next;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C048: Linked list traversal should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C048: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c049_realloc_pattern() {
    let c_code = r#"
#include <stdlib.h>
int *grow_array(int *arr, int old_size, int new_size) {
    int *new_arr = (int *)realloc(arr, new_size * sizeof(int));
    return new_arr;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C049: Realloc pattern should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C049: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c050_calloc_vs_malloc_memset() {
    let c_code = r#"
#include <stdlib.h>
#include <string.h>
int *alloc_calloc(int n) {
    return (int *)calloc(n, sizeof(int));
}
int *alloc_malloc(int n) {
    int *p = (int *)malloc(n * sizeof(int));
    memset(p, 0, n * sizeof(int));
    return p;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C050: Calloc/malloc+memset should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C050: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c051_pointer_cast_integer() {
    let c_code = r#"
#include <stdint.h>
uintptr_t ptr_to_int(void *p) {
    return (uintptr_t)p;
}
void *int_to_ptr(uintptr_t val) {
    return (void *)val;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C051: Pointer-integer cast should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C051: Output should not be empty");
}

#[test]
fn c052_const_pointer_vs_pointer_to_const() {
    let c_code = r#"
int read_only(const int *p) {
    return *p;
}
int fixed_pointer(int * const p) {
    *p = 42;
    return *p;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C052: Const pointer variants should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C052: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c053_double_free_detection() {
    let c_code = r#"
#include <stdlib.h>
void double_free() {
    int *p = (int *)malloc(sizeof(int));
    free(p);
    free(p);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C053: Double free should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C053: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c054_use_after_free_detection() {
    let c_code = r#"
#include <stdlib.h>
int use_after_free() {
    int *p = (int *)malloc(sizeof(int));
    *p = 42;
    free(p);
    return *p;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C054: Use after free should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C054: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c055_buffer_overflow_detection() {
    let c_code = r#"
void overflow() {
    int arr[5];
    int i;
    for (i = 0; i <= 10; i++) {
        arr[i] = i;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C055: Buffer overflow should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C055: Output should not be empty");
}

// ============================================================================
// C056-C070: Structs, Unions, Enums
// ============================================================================

#[test]
fn c056_basic_struct() {
    let c_code = r#"
struct Point {
    int x;
    int y;
};
int main() { struct Point p; p.x = 1; p.y = 2; return p.x + p.y; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C056: Basic struct should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C056: Output should not be empty");
}

#[test]
fn c057_nested_struct() {
    let c_code = r#"
struct Inner { int a; };
struct Outer { struct Inner inner; int b; };
int get_inner_a(struct Outer *o) { return o->inner.a; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C057: Nested struct should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C057: Output should not be empty");
}

#[test]
fn c058_self_referential_struct() {
    let c_code = r#"
struct ListNode {
    int data;
    struct ListNode *next;
};
int get_data(struct ListNode *n) { return n->data; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C058: Self-referential struct should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C058: Output should not be empty");
}

#[test]
fn c059_union_type() {
    let c_code = r#"
union Value {
    int i;
    float f;
    char c;
};
int get_int(union Value *v) { return v->i; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C059: Union type should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C059: Output should not be empty");
}

#[test]
fn c060_tagged_union() {
    let c_code = r#"
enum Tag { INT_TAG, FLOAT_TAG };
struct TaggedValue {
    enum Tag tag;
    union {
        int i;
        float f;
    } value;
};
int get_value(struct TaggedValue *tv) {
    if (tv->tag == INT_TAG) return tv->value.i;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C060: Tagged union should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C060: Output should not be empty");
}

#[test]
fn c061_bit_fields() {
    let c_code = r#"
struct Flags {
    unsigned int read : 1;
    unsigned int write : 1;
    unsigned int execute : 1;
    unsigned int reserved : 5;
};
int is_readable(struct Flags *f) { return f->read; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C061: Bit fields should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C061: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c062_flexible_array_member() {
    let c_code = r#"
struct Buffer {
    int size;
    char data[];
};
int get_size(struct Buffer *b) { return b->size; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C062: Flexible array member should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C062: Output should not be empty");
}

#[test]
fn c063_anonymous_struct() {
    let c_code = r#"
struct Container {
    struct {
        int x;
        int y;
    } pos;
    int id;
};
int get_id(struct Container *c) { return c->id; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C063: Anonymous struct should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C063: Output should not be empty");
}

#[test]
fn c064_struct_with_function_pointer() {
    let c_code = r#"
struct Handler {
    int (*process)(int);
    int id;
};
int call_handler(struct Handler *h, int val) { return h->process(val); }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C064: Struct with function pointer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C064: Output should not be empty");
}

#[test]
fn c065_designated_initializer() {
    let c_code = r#"
struct Config {
    int width;
    int height;
    int depth;
};
struct Config make_config() {
    struct Config c;
    c.width = 800;
    c.height = 600;
    c.depth = 32;
    return c;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C065: Struct initialization should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C065: Output should not be empty");
}

#[test]
fn c066_struct_copy_vs_pointer() {
    let c_code = r#"
struct Pair { int a; int b; };
struct Pair copy_pair(struct Pair p) {
    struct Pair result;
    result.a = p.a;
    result.b = p.b;
    return result;
}
int sum_pair(struct Pair *p) { return p->a + p->b; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C066: Struct copy vs pointer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C066: Output should not be empty");
}

#[test]
fn c067_struct_padding() {
    let c_code = r#"
struct Padded {
    char a;
    int b;
    char c;
    double d;
};
int get_size() { return sizeof(struct Padded); }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C067: Struct padding should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C067: Output should not be empty");
}

#[test]
fn c068_enum_explicit_values() {
    let c_code = r#"
enum Priority {
    LOW = 0,
    MEDIUM = 5,
    HIGH = 10,
    CRITICAL = 100
};
int get_priority_val(enum Priority p) { return p; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C068: Enum with explicit values should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C068: Output should not be empty");
}

#[test]
fn c069_enum_arithmetic() {
    let c_code = r#"
enum Direction { NORTH = 0, EAST = 1, SOUTH = 2, WEST = 3 };
int opposite(enum Direction d) {
    return (d + 2) % 4;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C069: Enum arithmetic should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C069: Output should not be empty");
}

#[test]
fn c070_typedef_struct_pattern() {
    let c_code = r#"
typedef struct {
    int x;
    int y;
} Vec2;
Vec2 add_vec(Vec2 a, Vec2 b) {
    Vec2 result;
    result.x = a.x + b.x;
    result.y = a.y + b.y;
    return result;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C070: Typedef struct should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C070: Output should not be empty");
}

// ============================================================================
// C071-C090: Functions
// ============================================================================

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c071_variadic_function() {
    let c_code = r#"
#include <stdarg.h>
int sum_args(int count, ...) {
    va_list args;
    va_start(args, count);
    int total = 0;
    int i;
    for (i = 0; i < count; i++) {
        total += va_arg(args, int);
    }
    va_end(args);
    return total;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C071: Variadic function should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C071: Output should not be empty");
}

#[test]
fn c072_static_function() {
    let c_code = r#"
static int helper() { return 42; }
int public_func() { return helper(); }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C072: Static function should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C072: Output should not be empty");
}

#[test]
fn c073_inline_function() {
    let c_code = r#"
static inline int square(int x) { return x * x; }
int calc() { return square(5); }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C073: Inline function should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C073: Output should not be empty");
}

#[test]
fn c074_recursive_function() {
    let c_code = r#"
int factorial(int n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C074: Recursive function should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C074: Output should not be empty");
}

#[test]
fn c075_mutual_recursion() {
    let c_code = r#"
int is_even(int n);
int is_odd(int n);
int is_even(int n) {
    if (n == 0) return 1;
    return is_odd(n - 1);
}
int is_odd(int n) {
    if (n == 0) return 0;
    return is_even(n - 1);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C075: Mutual recursion should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C075: Output should not be empty");
}

#[test]
fn c076_callback_pattern() {
    let c_code = r#"
typedef int (*callback_t)(int);
int with_callback(callback_t cb, int x) {
    return cb(x);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C076: Callback pattern should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C076: Output should not be empty");
}

#[test]
fn c077_qsort_comparator() {
    let c_code = r#"
int compare_int(const void *a, const void *b) {
    int ia = *(const int *)a;
    int ib = *(const int *)b;
    return ia - ib;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C077: Qsort comparator should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C077: Output should not be empty");
}

#[test]
fn c078_function_returning_struct() {
    let c_code = r#"
struct Result { int value; int error; };
struct Result make_ok(int val) {
    struct Result r;
    r.value = val;
    r.error = 0;
    return r;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C078: Function returning struct should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C078: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c079_return_pointer_to_local() {
    let c_code = r#"
int *bad_return() {
    int local = 42;
    return &local;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C079: Return pointer to local should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C079: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c080_kr_style_function() {
    let c_code = r#"
int add(a, b)
    int a;
    int b;
{
    return a + b;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C080: K&R style function should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C080: Output should not be empty");
}

#[test]
fn c081_void_function() {
    let c_code = r#"
void do_nothing() {}
void set_value(int *p, int val) { *p = val; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C081: Void function should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C081: Output should not be empty");
}

#[test]
fn c082_many_parameters() {
    let c_code = r#"
int many_params(int a, int b, int c, int d, int e, int f, int g, int h, int i) {
    return a + b + c + d + e + f + g + h + i;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C082: Many parameters should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C082: Output should not be empty");
}

#[test]
fn c083_nested_function_calls() {
    let c_code = r#"
int double_val(int x) { return x * 2; }
int add_one(int x) { return x + 1; }
int composed(int x) { return double_val(add_one(double_val(x))); }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C083: Nested function calls should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C083: Output should not be empty");
}

#[test]
fn c084_side_effects_in_args() {
    let c_code = r#"
int add(int a, int b) { return a + b; }
int with_side_effects() {
    int x = 1;
    int result = add(x++, x++);
    return result;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C084: Side effects in args should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C084: Output should not be empty");
}

#[test]
fn c085_static_local_variable() {
    let c_code = r#"
int counter() {
    static int count = 0;
    count++;
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C085: Static local variable should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C085: Output should not be empty");
}

#[test]
fn c086_register_variable() {
    let c_code = r#"
int fast_sum(int *arr, int n) {
    register int sum = 0;
    register int i;
    for (i = 0; i < n; i++) {
        sum += arr[i];
    }
    return sum;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C086: Register variable should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C086: Output should not be empty");
}

#[test]
fn c087_extern_variable() {
    let c_code = r#"
extern int global_count;
int get_count() { return global_count; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C087: Extern variable should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C087: Output should not be empty");
}

#[test]
fn c088_global_variable_init() {
    let c_code = r#"
int global_val = 100;
int get_global() { return global_val; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C088: Global variable init should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C088: Output should not be empty");
}

#[test]
fn c089_const_global_array() {
    let c_code = r#"
const int primes[5] = {2, 3, 5, 7, 11};
int get_prime(int i) { return primes[i]; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C089: Const global array should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C089: Output should not be empty");
}

#[test]
fn c090_string_literal_global() {
    let c_code = r#"
const char *greeting = "hello";
const char *get_greeting() { return greeting; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C090: String literal global should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C090: Output should not be empty");
}

// ============================================================================
// C091-C110: Standard Library
// ============================================================================

#[test]
fn c091_malloc_free_basic() {
    let c_code = r#"
#include <stdlib.h>
int *alloc_int(int val) {
    int *p = (int *)malloc(sizeof(int));
    *p = val;
    return p;
}
void free_int(int *p) {
    free(p);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C091: malloc/free basic should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C091: Output should not be empty");
}

#[test]
fn c092_malloc_array() {
    let c_code = r#"
#include <stdlib.h>
int *alloc_array(int n) {
    int *arr = (int *)malloc(n * sizeof(int));
    return arr;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C092: malloc array should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C092: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c093_strcpy() {
    let c_code = r#"
#include <string.h>
void copy_str(char *dst, const char *src) {
    strcpy(dst, src);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C093: strcpy should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C093: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c094_strcmp() {
    let c_code = r#"
#include <string.h>
int are_equal(const char *a, const char *b) {
    return strcmp(a, b) == 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C094: strcmp should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C094: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c095_strcat() {
    let c_code = r#"
#include <string.h>
void append(char *dst, const char *src) {
    strcat(dst, src);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C095: strcat should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C095: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c096_strtok() {
    let c_code = r#"
#include <string.h>
int count_tokens(char *str, const char *delim) {
    int count = 0;
    char *tok = strtok(str, delim);
    while (tok != 0) {
        count++;
        tok = strtok(0, delim);
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C096: strtok should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C096: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c097_memcpy() {
    let c_code = r#"
#include <string.h>
void copy_mem(void *dst, const void *src, int n) {
    memcpy(dst, src, n);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C097: memcpy should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C097: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c098_memset() {
    let c_code = r#"
#include <string.h>
void zero_mem(void *ptr, int n) {
    memset(ptr, 0, n);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C098: memset should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C098: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c099_file_io() {
    let c_code = r#"
#include <stdio.h>
int read_first_byte(const char *filename) {
    FILE *f = fopen(filename, "r");
    if (f == 0) return -1;
    int ch = fgetc(f);
    fclose(f);
    return ch;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C099: File I/O should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C099: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c100_printf_format() {
    let c_code = r#"
#include <stdio.h>
void print_info(const char *name, int age) {
    printf("Name: %s, Age: %d\n", name, age);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C100: printf format should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C100: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c101_scanf_input() {
    let c_code = r#"
#include <stdio.h>
int read_int() {
    int val;
    scanf("%d", &val);
    return val;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C101: scanf should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C101: Output should not be empty");
}

#[test]
fn c102_assert_macro() {
    let c_code = r#"
#include <assert.h>
int safe_div(int a, int b) {
    assert(b != 0);
    return a / b;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C102: assert macro should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C102: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c103_errno_checking() {
    let c_code = r#"
#include <errno.h>
#include <stdlib.h>
long safe_strtol(const char *str) {
    errno = 0;
    long val = strtol(str, 0, 10);
    if (errno != 0) return -1;
    return val;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C103: errno checking should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C103: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c104_signal_handling() {
    let c_code = r#"
#include <signal.h>
volatile int got_signal = 0;
void handler(int sig) {
    got_signal = 1;
}
void setup_handler() {
    signal(SIGINT, handler);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C104: Signal handling should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C104: Output should not be empty");
}

#[test]
#[ignore = "FALSIFIED: setjmp/longjmp not supported"]
fn c105_setjmp_longjmp() {
    let c_code = r#"
#include <setjmp.h>
jmp_buf env;
int try_operation() {
    if (setjmp(env) == 0) {
        return 1;
    } else {
        return -1;
    }
}
void fail_operation() {
    longjmp(env, 1);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C105: setjmp/longjmp should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C105: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c106_atexit_handler() {
    let c_code = r#"
#include <stdlib.h>
void cleanup() {}
int setup() {
    atexit(cleanup);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C106: atexit should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C106: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c107_qsort_usage() {
    let c_code = r#"
#include <stdlib.h>
int cmp(const void *a, const void *b) {
    return *(const int *)a - *(const int *)b;
}
void sort_array(int *arr, int n) {
    qsort(arr, n, sizeof(int), cmp);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C107: qsort usage should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C107: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c108_bsearch_usage() {
    let c_code = r#"
#include <stdlib.h>
int cmp(const void *a, const void *b) {
    return *(const int *)a - *(const int *)b;
}
int *find_in_sorted(int *arr, int n, int key) {
    return (int *)bsearch(&key, arr, n, sizeof(int), cmp);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C108: bsearch should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C108: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c109_atoi_strtol() {
    let c_code = r#"
#include <stdlib.h>
int parse_int(const char *str) {
    return atoi(str);
}
long parse_long(const char *str) {
    return strtol(str, 0, 10);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C109: atoi/strtol should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C109: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c110_math_functions() {
    let c_code = r#"
#include <math.h>
double hypotenuse(double a, double b) {
    return sqrt(a * a + b * b);
}
double power(double base, double exp) {
    return pow(base, exp);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C110: Math functions should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C110: Output should not be empty");
}

// ============================================================================
// C111-C130: Preprocessor and Advanced
// ============================================================================

#[test]
fn c111_define_constant() {
    let c_code = r#"
#define MAX_SIZE 100
int get_max() { return MAX_SIZE; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C111: #define constant should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C111: Output should not be empty");
}

#[test]
fn c112_define_function_macro() {
    let c_code = r#"
#define MAX(a, b) ((a) > (b) ? (a) : (b))
int max_val(int x, int y) { return MAX(x, y); }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C112: Function-like macro should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C112: Output should not be empty");
}

#[test]
fn c113_conditional_compilation() {
    let c_code = r#"
#define DEBUG 1
int get_mode() {
#ifdef DEBUG
    return 1;
#else
    return 0;
#endif
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C113: Conditional compilation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C113: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c114_stringification() {
    let c_code = r#"
#define STR(x) #x
const char *get_name() { return STR(hello); }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C114: Stringification should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C114: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c115_token_pasting() {
    let c_code = r#"
#define CONCAT(a, b) a##b
int CONCAT(my, func)() { return 42; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C115: Token pasting should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C115: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c116_variadic_macro() {
    let c_code = r#"
#include <stdio.h>
#define LOG(fmt, ...) printf(fmt, __VA_ARGS__)
void log_info(int code) { LOG("code: %d\n", code); }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C116: Variadic macro should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C116: Output should not be empty");
}

#[test]
fn c117_include_guard() {
    let c_code = r#"
#ifndef MY_HEADER_H
#define MY_HEADER_H
int guarded_func() { return 1; }
#endif
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C117: Include guard should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C117: Output should not be empty");
}

#[test]
fn c118_volatile_variable() {
    let c_code = r#"
volatile int flag = 0;
int check_flag() { return flag; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C118: Volatile variable should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C118: Output should not be empty");
}

#[test]
fn c119_restrict_keyword() {
    let c_code = r#"
void add_arrays(int * restrict a, const int * restrict b, int n) {
    int i;
    for (i = 0; i < n; i++) {
        a[i] += b[i];
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C119: Restrict keyword should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C119: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c120_static_assert() {
    let c_code = r#"
_Static_assert(sizeof(int) >= 4, "int must be at least 4 bytes");
int test() { return 0; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C120: _Static_assert should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C120: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c121_compound_literal() {
    let c_code = r#"
struct Point { int x; int y; };
struct Point *make_point() {
    return &(struct Point){ .x = 1, .y = 2 };
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C121: Compound literal should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C121: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c122_designated_initializer_c99() {
    let c_code = r#"
struct Config { int a; int b; int c; };
struct Config make() {
    struct Config cfg = { .c = 3, .a = 1, .b = 2 };
    return cfg;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C122: Designated initializer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C122: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c123_variable_length_array() {
    let c_code = r#"
int sum_vla(int n) {
    int arr[n];
    int i;
    for (i = 0; i < n; i++) arr[i] = i;
    int total = 0;
    for (i = 0; i < n; i++) total += arr[i];
    return total;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C123: VLA should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C123: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c124_generic_keyword() {
    let c_code = r#"
#define type_name(x) _Generic((x), \
    int: "int", \
    float: "float", \
    double: "double", \
    default: "other")
int test() { return 0; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C124: _Generic should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C124: Output should not be empty");
}

#[test]
#[ignore = "FALSIFIED: _Alignof/_Alignas (C11) not supported"]
fn c125_alignof_alignas() {
    let c_code = r#"
#include <stdalign.h>
int get_alignment() {
    return alignof(double);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C125: _Alignof should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C125: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c126_atomic() {
    let c_code = r#"
#include <stdatomic.h>
_Atomic int counter = 0;
void increment() {
    atomic_fetch_add(&counter, 1);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C126: _Atomic should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C126: Output should not be empty");
}

#[test]
#[ignore = "FALSIFIED: _Complex number type not supported"]
fn c127_complex_number() {
    let c_code = r#"
#include <complex.h>
double complex make_complex(double r, double i) {
    return r + i * I;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C127: Complex number should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C127: Output should not be empty");
}

#[test]
fn c128_bool_type() {
    let c_code = r#"
#include <stdbool.h>
bool is_positive(int x) {
    return x > 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C128: _Bool type should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C128: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c129_inline_assembly() {
    let c_code = r#"
int read_timestamp() {
    unsigned int lo, hi;
    __asm__ __volatile__ ("rdtsc" : "=a"(lo), "=d"(hi));
    return lo;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C129: Inline assembly should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C129: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c130_pragma_directives() {
    let c_code = r#"
#pragma once
#pragma pack(push, 1)
struct Packed { char a; int b; };
#pragma pack(pop)
int test() { return sizeof(struct Packed); }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C130: Pragma directives should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C130: Output should not be empty");
}

// ============================================================================
// C131-C150: Real-World Pathological Patterns
// ============================================================================

#[test]
fn c131_linked_list_operations() {
    let c_code = r#"
struct Node { int data; struct Node *next; };
int list_sum(struct Node *head) {
    int sum = 0;
    struct Node *cur = head;
    while (cur != 0) {
        sum += cur->data;
        cur = cur->next;
    }
    return sum;
}
int list_length(struct Node *head) {
    int len = 0;
    struct Node *cur = head;
    while (cur != 0) {
        len++;
        cur = cur->next;
    }
    return len;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C131: Linked list operations should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C131: Output should not be empty");
}

#[test]
fn c132_binary_tree_operations() {
    let c_code = r#"
struct TreeNode {
    int value;
    struct TreeNode *left;
    struct TreeNode *right;
};
int tree_height(struct TreeNode *root) {
    if (root == 0) return 0;
    int lh = tree_height(root->left);
    int rh = tree_height(root->right);
    if (lh > rh) return lh + 1;
    return rh + 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C132: Binary tree operations should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C132: Output should not be empty");
}

#[test]
fn c133_hash_table_impl() {
    let c_code = r#"
#define TABLE_SIZE 256
struct Entry { int key; int value; struct Entry *next; };
struct HashTable { struct Entry *buckets[TABLE_SIZE]; };
int hash(int key) { return key % TABLE_SIZE; }
int ht_get(struct HashTable *ht, int key) {
    int idx = hash(key);
    struct Entry *e = ht->buckets[idx];
    while (e != 0) {
        if (e->key == key) return e->value;
        e = e->next;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C133: Hash table should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C133: Output should not be empty");
}

#[test]
fn c134_circular_buffer() {
    let c_code = r#"
#define BUF_SIZE 16
struct CircBuf {
    int data[BUF_SIZE];
    int head;
    int tail;
    int count;
};
int cb_push(struct CircBuf *cb, int val) {
    if (cb->count >= BUF_SIZE) return -1;
    cb->data[cb->tail] = val;
    cb->tail = (cb->tail + 1) % BUF_SIZE;
    cb->count++;
    return 0;
}
int cb_pop(struct CircBuf *cb) {
    if (cb->count <= 0) return -1;
    int val = cb->data[cb->head];
    cb->head = (cb->head + 1) % BUF_SIZE;
    cb->count--;
    return val;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C134: Circular buffer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C134: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c135_memory_pool() {
    let c_code = r#"
#include <stdlib.h>
struct Pool {
    char *memory;
    int offset;
    int capacity;
};
struct Pool *pool_create(int cap) {
    struct Pool *p = (struct Pool *)malloc(sizeof(struct Pool));
    p->memory = (char *)malloc(cap);
    p->offset = 0;
    p->capacity = cap;
    return p;
}
void *pool_alloc(struct Pool *p, int size) {
    if (p->offset + size > p->capacity) return 0;
    void *ptr = p->memory + p->offset;
    p->offset += size;
    return ptr;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C135: Memory pool should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C135: Output should not be empty");
}

#[test]
fn c136_reference_counting() {
    let c_code = r#"
struct RefCounted {
    int refcount;
    int data;
};
void retain(struct RefCounted *obj) {
    obj->refcount++;
}
int release(struct RefCounted *obj) {
    obj->refcount--;
    return obj->refcount;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C136: Reference counting should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C136: Output should not be empty");
}

#[test]
fn c137_observer_pattern() {
    let c_code = r#"
typedef void (*observer_fn)(int);
struct Subject {
    observer_fn observers[10];
    int count;
};
void notify(struct Subject *s, int event) {
    int i;
    for (i = 0; i < s->count; i++) {
        s->observers[i](event);
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C137: Observer pattern should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C137: Output should not be empty");
}

#[test]
fn c138_state_machine_function_pointers() {
    let c_code = r#"
typedef int (*state_fn)(int);
int state_idle(int input) { return input > 0 ? 1 : 0; }
int state_running(int input) { return input == 0 ? 0 : 1; }
int run_machine(int initial_state, int input) {
    state_fn states[2];
    states[0] = state_idle;
    states[1] = state_running;
    return states[initial_state](input);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C138: State machine should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C138: Output should not be empty");
}

#[test]
#[ignore = "FALSIFIED: setjmp/longjmp coroutine pattern not supported"]
fn c139_coroutine_setjmp() {
    let c_code = r#"
#include <setjmp.h>
jmp_buf main_ctx, co_ctx;
int co_value;
void coroutine() {
    co_value = 1;
    longjmp(main_ctx, 1);
    co_value = 2;
    longjmp(main_ctx, 1);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C139: Coroutine via setjmp should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C139: Output should not be empty");
}

#[test]
fn c140_opaque_pointer() {
    let c_code = r#"
struct OpaqueImpl;
typedef struct OpaqueImpl* Handle;
int use_handle(Handle h);
int test() { return 0; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C140: Opaque pointer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C140: Output should not be empty");
}

#[test]
fn c141_vtable_polymorphism() {
    let c_code = r#"
struct VTable {
    int (*area)(void *self);
    int (*perimeter)(void *self);
};
struct Shape {
    struct VTable *vtable;
};
int get_area(struct Shape *s) {
    return s->vtable->area(s);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C141: VTable polymorphism should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C141: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c142_errno_goto_cleanup() {
    let c_code = r#"
#include <stdlib.h>
int process() {
    int *a = (int *)malloc(10 * sizeof(int));
    if (!a) goto fail;
    int *b = (int *)malloc(20 * sizeof(int));
    if (!b) goto cleanup_a;
    int *c = (int *)malloc(30 * sizeof(int));
    if (!c) goto cleanup_b;

    free(c);
cleanup_b:
    free(b);
cleanup_a:
    free(a);
fail:
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C142: Goto cleanup should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C142: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c143_thread_local_storage() {
    let c_code = r#"
__thread int tls_counter = 0;
int get_tls_counter() { return tls_counter; }
void inc_tls_counter() { tls_counter++; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C143: Thread-local storage should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C143: Output should not be empty");
}

#[test]
#[ignore = "FALSIFIED: atomic operations (stdatomic.h) not supported"]
fn c144_atomic_operations() {
    let c_code = r#"
#include <stdatomic.h>
atomic_int shared_counter = 0;
void atomic_inc() {
    atomic_fetch_add(&shared_counter, 1);
}
int atomic_get() {
    return atomic_load(&shared_counter);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C144: Atomic operations should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C144: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c145_memory_mapped_io() {
    let c_code = r#"
#define GPIO_BASE 0x40000000
volatile unsigned int *gpio = (volatile unsigned int *)GPIO_BASE;
void set_pin(int pin) {
    *gpio |= (1 << pin);
}
void clear_pin(int pin) {
    *gpio &= ~(1 << pin);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C145: Memory-mapped I/O should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C145: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c146_packed_struct() {
    let c_code = r#"
struct __attribute__((packed)) PackedData {
    char type;
    int value;
    short flags;
};
int get_value(struct PackedData *p) { return p->value; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C146: Packed struct should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C146: Output should not be empty");
}

#[test]
fn c147_type_punning_union() {
    let c_code = r#"
union Pun {
    int i;
    float f;
};
float int_bits_to_float(int bits) {
    union Pun p;
    p.i = bits;
    return p.f;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C147: Type punning via union should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C147: Output should not be empty");
}

#[test]
// UN-FALSIFIED: transpiler improvements resolved this test case
fn c148_computed_goto() {
    let c_code = r#"
int dispatch(int op, int a, int b) {
    static void *table[] = { &&op_add, &&op_sub, &&op_mul };
    if (op < 0 || op > 2) return -1;
    goto *table[op];
op_add: return a + b;
op_sub: return a - b;
op_mul: return a * b;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C148: Computed goto should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C148: Output should not be empty");
}

#[test]
#[ignore = "FALSIFIED: nested functions (GCC extension) not supported"]
fn c149_nested_functions() {
    let c_code = r#"
int outer(int x) {
    int inner(int y) {
        return x + y;
    }
    return inner(10);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C149: Nested functions should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C149: Output should not be empty");
}

#[test]
fn c150_boolean_negation_edge_cases() {
    let c_code = r#"
int bool_edge(int x) {
    int a = !0;
    int b = !1;
    int c = !!x;
    int d = !(-1);
    int e = !(x > 0);
    return a + b + c + d + e;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C150: Boolean negation edge cases should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C150: Output should not be empty");
}
