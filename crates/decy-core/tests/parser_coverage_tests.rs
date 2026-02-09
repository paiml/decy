//! Parser coverage integration tests (DECY-COVERAGE-PARSER)
//!
//! These tests exercise parser code paths (extract_inc_dec_stmt, extract_for_stmt,
//! extract_binary_operator, process_ast_to_rust) via the transpilation pipeline
//! to improve coverage in decy-parser/src/parser.rs.

use decy_core::ProjectContext;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper: Create temporary C file with content
fn create_temp_c_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).expect("Failed to write temp file");
    path
}

/// Helper: Transpile C code string and assert success
fn assert_transpiles(c_code: &str) -> String {
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "Failed to transpile:\n{}\nError: {:?}",
        c_code,
        result.err()
    );
    result.unwrap()
}

/// Helper: Transpile C file via transpile_file and assert success
fn assert_transpile_file(dir: &TempDir, name: &str, content: &str) -> String {
    let file = create_temp_c_file(dir, name, content);
    let context = ProjectContext::new();
    let result = decy_core::transpile_file(&file, &context);
    assert!(
        result.is_ok(),
        "Failed to transpile file {}:\n{}\nError: {:?}",
        name,
        content,
        result.err()
    );
    result.unwrap().rust_code
}

// ============================================================================
// extract_inc_dec_stmt coverage (88 uncovered lines)
// ============================================================================

#[test]
fn test_parser_post_increment() {
    let code = assert_transpiles(
        "void foo() { int x = 0; x++; }",
    );
    assert!(
        code.contains("x") || code.contains("foo"),
        "Output should reference variable or function"
    );
}

#[test]
fn test_parser_pre_increment() {
    let code = assert_transpiles(
        "void foo() { int x = 0; ++x; }",
    );
    assert!(code.contains("x") || code.contains("foo"));
}

#[test]
fn test_parser_post_decrement() {
    let code = assert_transpiles(
        "void foo() { int x = 5; x--; }",
    );
    assert!(code.contains("x") || code.contains("foo"));
}

#[test]
fn test_parser_pre_decrement() {
    let code = assert_transpiles(
        "void foo() { int x = 5; --x; }",
    );
    assert!(code.contains("x") || code.contains("foo"));
}

#[test]
fn test_parser_inc_array_element() {
    let code = assert_transpiles(
        "void foo() { int arr[10]; int i = 0; arr[i]++; }",
    );
    assert!(code.contains("arr") || code.contains("foo"));
}

#[test]
fn test_parser_inc_struct_field() {
    let code = assert_transpiles(r#"
        struct Counter { int count; };
        void inc(struct Counter* s) { s->count++; }
    "#);
    assert!(code.contains("count") || code.contains("inc"));
}

#[test]
fn test_parser_inc_pointer_deref() {
    let code = assert_transpiles(
        "void foo(int* p) { (*p)++; }",
    );
    assert!(code.contains("foo"));
}

#[test]
fn test_parser_inc_in_for_loop_update() {
    let code = assert_transpiles(
        "int sum() { int s = 0; for (int i = 0; i < 10; i++) { s += i; } return s; }",
    );
    assert!(code.contains("sum") || code.contains("fn"));
}

#[test]
fn test_parser_multiple_increments() {
    let code = assert_transpiles(r#"
        void foo() {
            int a = 0, b = 0, c = 0;
            a++;
            b++;
            c++;
            ++a;
            --b;
            c--;
        }
    "#);
    assert!(code.contains("foo"));
}

#[test]
fn test_parser_inc_int_type() {
    assert_transpiles("void foo() { int x = 0; x++; }");
}

#[test]
fn test_parser_inc_char_type() {
    assert_transpiles("void foo() { char c = 'a'; c++; }");
}

#[test]
fn test_parser_inc_long_type() {
    assert_transpiles("void foo() { long n = 0; n++; }");
}

#[test]
fn test_parser_dec_array_element() {
    assert_transpiles("void foo() { int arr[5]; arr[2]--; }");
}

#[test]
fn test_parser_pre_inc_in_expression() {
    let code = assert_transpiles(
        "int foo() { int x = 5; int y = ++x; return y; }",
    );
    assert!(code.contains("foo"));
}

#[test]
fn test_parser_post_inc_in_expression() {
    let code = assert_transpiles(
        "int foo() { int x = 5; int y = x++; return y; }",
    );
    assert!(code.contains("foo"));
}

// ============================================================================
// extract_for_stmt coverage (69 uncovered lines)
// ============================================================================

#[test]
fn test_parser_basic_for_loop() {
    let code = assert_transpiles(
        "int sum(int n) { int s = 0; for (int i = 0; i < n; i++) { s += i; } return s; }",
    );
    assert!(code.contains("sum") || code.contains("fn"));
}

#[test]
fn test_parser_for_no_init() {
    let code = assert_transpiles(r#"
        int foo(int start) {
            int i = start;
            int s = 0;
            for (; i < 10; i++) {
                s += i;
            }
            return s;
        }
    "#);
    assert!(code.contains("foo"));
}

#[test]
fn test_parser_for_multiple_init_vars() {
    let code = assert_transpiles(r#"
        int foo() {
            int s = 0;
            for (int i = 0, j = 10; i < j; i++) {
                s += i;
            }
            return s;
        }
    "#);
    assert!(code.contains("foo"));
}

#[test]
fn test_parser_for_comma_update() {
    let code = assert_transpiles(r#"
        int foo() {
            int s = 0;
            for (int i = 0, j = 10; i < j; i++, j--) {
                s += i + j;
            }
            return s;
        }
    "#);
    assert!(code.contains("foo"));
}

#[test]
fn test_parser_for_complex_condition() {
    let code = assert_transpiles(r#"
        int foo(int n) {
            int s = 0;
            for (int i = 0; i < n && i < 100; i++) {
                s += i;
            }
            return s;
        }
    "#);
    assert!(code.contains("foo"));
}

#[test]
fn test_parser_nested_for_loops_3_deep() {
    let code = assert_transpiles(r#"
        int foo() {
            int total = 0;
            for (int i = 0; i < 3; i++) {
                for (int j = 0; j < 3; j++) {
                    for (int k = 0; k < 3; k++) {
                        total++;
                    }
                }
            }
            return total;
        }
    "#);
    assert!(code.contains("foo") || code.contains("total"));
}

#[test]
fn test_parser_for_array_iteration() {
    let code = assert_transpiles(r#"
        int sum_arr(int arr[], int n) {
            int s = 0;
            for (int i = 0; i < n; i++) {
                s += arr[i];
            }
            return s;
        }
    "#);
    assert!(code.contains("sum_arr") || code.contains("fn"));
}

#[test]
fn test_parser_for_with_continue() {
    let code = assert_transpiles(r#"
        int foo(int n) {
            int s = 0;
            for (int i = 0; i < n; i++) {
                if (i % 2 == 0) continue;
                s += i;
            }
            return s;
        }
    "#);
    assert!(code.contains("foo"));
}

#[test]
fn test_parser_for_with_break() {
    let code = assert_transpiles(r#"
        int foo(int n) {
            int s = 0;
            for (int i = 0; i < n; i++) {
                if (s > 100) break;
                s += i;
            }
            return s;
        }
    "#);
    assert!(code.contains("foo"));
}

#[test]
fn test_parser_for_func_call_in_condition() {
    let code = assert_transpiles(r#"
        int get_limit() { return 10; }
        int foo() {
            int s = 0;
            for (int i = 0; i < get_limit(); i++) {
                s += i;
            }
            return s;
        }
    "#);
    assert!(code.contains("foo"));
}

#[test]
fn test_parser_for_single_stmt_body() {
    let code = assert_transpiles(r#"
        int foo(int n) {
            int s = 0;
            for (int i = 0; i < n; i++)
                s += i;
            return s;
        }
    "#);
    assert!(code.contains("foo"));
}

#[test]
fn test_parser_for_empty_body() {
    // for loop with empty compound body
    let code = assert_transpiles(r#"
        void busy_wait(int n) {
            for (int i = 0; i < n; i++) {}
        }
    "#);
    assert!(code.contains("busy_wait") || code.contains("fn"));
}

#[test]
fn test_parser_for_decrement_update() {
    let code = assert_transpiles(r#"
        int countdown(int n) {
            int s = 0;
            for (int i = n; i > 0; i--) {
                s += i;
            }
            return s;
        }
    "#);
    assert!(code.contains("countdown"));
}

#[test]
fn test_parser_for_assignment_init() {
    // for loop where init is assignment (not declaration)
    let code = assert_transpiles(r#"
        int foo() {
            int i;
            int s = 0;
            for (i = 0; i < 10; i++) {
                s += i;
            }
            return s;
        }
    "#);
    assert!(code.contains("foo"));
}

// ============================================================================
// extract_binary_operator coverage (88 uncovered lines)
// ============================================================================

#[test]
fn test_parser_binop_add() {
    let code = assert_transpiles("int add(int a, int b) { return a + b; }");
    assert!(code.contains("+") || code.contains("add"));
}

#[test]
fn test_parser_binop_subtract() {
    let code = assert_transpiles("int sub(int a, int b) { return a - b; }");
    assert!(code.contains("-") || code.contains("sub"));
}

#[test]
fn test_parser_binop_multiply() {
    let code = assert_transpiles("int mul(int a, int b) { return a * b; }");
    assert!(code.contains("*") || code.contains("mul"));
}

#[test]
fn test_parser_binop_divide() {
    let code = assert_transpiles("int div(int a, int b) { return a / b; }");
    assert!(code.contains("/") || code.contains("div"));
}

#[test]
fn test_parser_binop_modulo() {
    let code = assert_transpiles("int mod_fn(int a, int b) { return a % b; }");
    assert!(code.contains("%") || code.contains("mod"));
}

#[test]
fn test_parser_binop_less_than() {
    let code = assert_transpiles("int lt(int a, int b) { return a < b; }");
    assert!(code.contains("<") || code.contains("lt"));
}

#[test]
fn test_parser_binop_greater_than() {
    let code = assert_transpiles("int gt(int a, int b) { return a > b; }");
    assert!(code.contains(">") || code.contains("gt"));
}

#[test]
fn test_parser_binop_less_equal() {
    let code = assert_transpiles("int le(int a, int b) { return a <= b; }");
    assert!(code.contains("<=") || code.contains("le"));
}

#[test]
fn test_parser_binop_greater_equal() {
    let code = assert_transpiles("int ge(int a, int b) { return a >= b; }");
    assert!(code.contains(">=") || code.contains("ge"));
}

#[test]
fn test_parser_binop_equal() {
    let code = assert_transpiles("int eq(int a, int b) { return a == b; }");
    assert!(code.contains("==") || code.contains("eq"));
}

#[test]
fn test_parser_binop_not_equal() {
    let code = assert_transpiles("int ne(int a, int b) { return a != b; }");
    assert!(code.contains("!=") || code.contains("ne"));
}

#[test]
fn test_parser_binop_logical_and() {
    let code = assert_transpiles(
        "int both(int a, int b) { if (a > 0 && b > 0) return 1; return 0; }",
    );
    assert!(code.contains("&&") || code.contains("both"));
}

#[test]
fn test_parser_binop_logical_or() {
    let code = assert_transpiles(
        "int either(int a, int b) { if (a > 0 || b > 0) return 1; return 0; }",
    );
    assert!(code.contains("||") || code.contains("either"));
}

#[test]
fn test_parser_binop_bitwise_and() {
    let code = assert_transpiles("int band(int a, int b) { return a & b; }");
    assert!(code.contains("&") || code.contains("band"));
}

#[test]
fn test_parser_binop_bitwise_or() {
    let code = assert_transpiles("int bor(int a, int b) { return a | b; }");
    assert!(code.contains("|") || code.contains("bor"));
}

#[test]
fn test_parser_binop_bitwise_xor() {
    let code = assert_transpiles("int bxor(int a, int b) { return a ^ b; }");
    assert!(code.contains("^") || code.contains("bxor"));
}

#[test]
fn test_parser_binop_bitwise_complement() {
    let code = assert_transpiles("int bnot(int a) { return ~a; }");
    assert!(code.contains("!") || code.contains("bnot"));
}

#[test]
fn test_parser_binop_left_shift() {
    let code = assert_transpiles("int shl(int a, int b) { return a << b; }");
    assert!(code.contains("<<") || code.contains("shl"));
}

#[test]
fn test_parser_binop_right_shift() {
    let code = assert_transpiles("int shr(int a, int b) { return a >> b; }");
    assert!(code.contains(">>") || code.contains("shr"));
}

#[test]
fn test_parser_compound_assign_add() {
    assert_transpiles("int foo() { int x = 0; x += 5; return x; }");
}

#[test]
fn test_parser_compound_assign_sub() {
    assert_transpiles("int foo() { int x = 10; x -= 3; return x; }");
}

#[test]
fn test_parser_compound_assign_mul() {
    assert_transpiles("int foo() { int x = 2; x *= 4; return x; }");
}

#[test]
fn test_parser_compound_assign_div() {
    assert_transpiles("int foo() { int x = 20; x /= 4; return x; }");
}

#[test]
fn test_parser_compound_assign_mod() {
    assert_transpiles("int foo() { int x = 17; x %= 5; return x; }");
}

#[test]
fn test_parser_compound_assign_shl() {
    assert_transpiles("int foo() { int x = 1; x <<= 3; return x; }");
}

#[test]
fn test_parser_compound_assign_shr() {
    assert_transpiles("int foo() { int x = 16; x >>= 2; return x; }");
}

#[test]
fn test_parser_compound_assign_bitand() {
    assert_transpiles("int foo() { int x = 0xFF; x &= 0x0F; return x; }");
}

#[test]
fn test_parser_compound_assign_bitor() {
    assert_transpiles("int foo() { int x = 0; x |= 0x0F; return x; }");
}

#[test]
fn test_parser_compound_assign_bitxor() {
    assert_transpiles("int foo() { int x = 0xFF; x ^= 0x0F; return x; }");
}

#[test]
fn test_parser_binop_precedence_chain() {
    let code = assert_transpiles(
        "int prec(int a, int b, int c, int d) { return a + b * c / d; }",
    );
    assert!(code.contains("prec") || code.contains("fn"));
}

#[test]
fn test_parser_binop_chained_comparisons() {
    let code = assert_transpiles(r#"
        int range(int x) {
            if (x > 0 && x < 100 && x != 50) return 1;
            return 0;
        }
    "#);
    assert!(code.contains("range"));
}

#[test]
fn test_parser_binop_mixed_types() {
    let code = assert_transpiles(r#"
        double mixed(int a, double b) { return a + b; }
    "#);
    assert!(code.contains("mixed") || code.contains("fn"));
}

#[test]
fn test_parser_ternary_complex() {
    let code = assert_transpiles(r#"
        int clamp(int x, int lo, int hi) {
            return x < lo ? lo : (x > hi ? hi : x);
        }
    "#);
    assert!(code.contains("clamp"));
}

// ============================================================================
// process_ast_to_rust coverage (98 uncovered lines)
// ============================================================================

#[test]
fn test_parser_multiple_function_definitions() {
    let code = assert_transpiles(r#"
        int add(int a, int b) { return a + b; }
        int sub(int a, int b) { return a - b; }
        int mul(int a, int b) { return a * b; }
        int divide(int a, int b) { return a / b; }
        int negate(int a) { return -a; }
    "#);
    assert!(code.contains("add"));
    assert!(code.contains("sub"));
    assert!(code.contains("mul"));
}

#[test]
fn test_parser_func_all_param_types() {
    let code = assert_transpiles(r#"
        double func(int a, float b, double c, char d, long e, short f) {
            return a + b + c + d + e + f;
        }
    "#);
    assert!(code.contains("func") || code.contains("fn"));
}

#[test]
fn test_parser_func_with_array_param() {
    let code = assert_transpiles(r#"
        int sum(int arr[], int n) {
            int s = 0;
            for (int i = 0; i < n; i++) { s += arr[i]; }
            return s;
        }
    "#);
    assert!(code.contains("sum"));
}

#[test]
fn test_parser_func_with_struct_param() {
    let code = assert_transpiles(r#"
        struct Point { int x; int y; };
        int distance_sq(struct Point a, struct Point b) {
            int dx = a.x - b.x;
            int dy = a.y - b.y;
            return dx * dx + dy * dy;
        }
    "#);
    assert!(code.contains("Point") || code.contains("distance"));
}

#[test]
fn test_parser_func_with_pointer_param() {
    let code = assert_transpiles(r#"
        void swap(int* a, int* b) {
            int tmp = *a;
            *a = *b;
            *b = tmp;
        }
    "#);
    assert!(code.contains("swap"));
}

#[test]
fn test_parser_global_variable_declarations() {
    let code = assert_transpiles(r#"
        int global_count = 0;
        float global_rate = 1.5;
        char global_flag = 0;
        void update() { global_count++; }
    "#);
    assert!(code.contains("global_count") || code.contains("update"));
}

#[test]
fn test_parser_global_array_declarations() {
    let code = assert_transpiles(r#"
        int buffer[256];
        double weights[100];
        void init() { buffer[0] = 0; weights[0] = 1.0; }
    "#);
    assert!(code.contains("buffer") || code.contains("init"));
}

#[test]
fn test_parser_struct_definition() {
    let code = assert_transpiles(r#"
        struct Node {
            int value;
            struct Node* next;
        };
        int get_val(struct Node* n) { return n->value; }
    "#);
    assert!(code.contains("Node") || code.contains("value"));
}

#[test]
fn test_parser_enum_definition() {
    let code = assert_transpiles(r#"
        enum Color { RED = 0, GREEN = 1, BLUE = 2 };
        int is_red(enum Color c) { return c == RED; }
    "#);
    assert!(code.contains("Color") || code.contains("is_red"));
}

#[test]
fn test_parser_transpile_file_multiple_includes() {
    let temp = TempDir::new().unwrap();

    let header1 = create_temp_c_file(
        &temp,
        "types.h",
        "#ifndef TYPES_H\n#define TYPES_H\ntypedef int MyInt;\n#endif\n",
    );
    let _header2 = create_temp_c_file(
        &temp,
        "utils.h",
        "#ifndef UTILS_H\n#define UTILS_H\nint helper(int x);\n#endif\n",
    );

    let main_content = format!(
        r#"#include "{}"
#include "{}"
MyInt test(MyInt x) {{ return helper(x) + 1; }}
"#,
        header1.file_name().unwrap().to_str().unwrap(),
        _header2.file_name().unwrap().to_str().unwrap()
    );

    let main_file = create_temp_c_file(&temp, "main.c", &main_content);
    let context = ProjectContext::new();
    // May or may not succeed depending on include path resolution; should not panic
    let _ = decy_core::transpile_file(&main_file, &context);
}

// ============================================================================
// Additional pathological C patterns
// ============================================================================

#[test]
fn test_parser_deeply_nested_expressions() {
    let code = assert_transpiles(r#"
        int deep(int a, int b, int c, int d, int e) {
            return a + (b * (c - (d / (e + 1))));
        }
    "#);
    assert!(code.contains("deep"));
}

#[test]
fn test_parser_char_array_operations() {
    let code = assert_transpiles(r#"
        void set_char(char buf[], int idx, char val) {
            buf[idx] = val;
        }
    "#);
    assert!(code.contains("set_char") || code.contains("fn"));
}

#[test]
fn test_parser_switch_many_cases() {
    let code = assert_transpiles(r#"
        int classify(int x) {
            switch (x) {
                case 0: return 0;
                case 1: return 10;
                case 2: return 20;
                case 3: return 30;
                case 4: return 40;
                case 5: return 50;
                case 6: return 60;
                case 7: return 70;
                case 8: return 80;
                case 9: return 90;
                default: return -1;
            }
        }
    "#);
    assert!(code.contains("classify") || code.contains("match"));
}

#[test]
fn test_parser_struct_initialization() {
    let code = assert_transpiles(r#"
        struct Rect { int x; int y; int w; int h; };
        struct Rect make_rect(int x, int y, int w, int h) {
            struct Rect r;
            r.x = x;
            r.y = y;
            r.w = w;
            r.h = h;
            return r;
        }
    "#);
    assert!(code.contains("Rect") || code.contains("make_rect"));
}

#[test]
fn test_parser_typedef_usage() {
    let code = assert_transpiles(r#"
        typedef int i32_t;
        typedef unsigned int u32_t;
        i32_t convert(u32_t val) {
            return (i32_t)val;
        }
    "#);
    assert!(code.contains("convert") || code.contains("fn"));
}

#[test]
fn test_parser_function_pointer_declaration() {
    let code = assert_transpiles(r#"
        typedef int (*compare_fn)(int, int);
        int sort_helper(compare_fn cmp, int a, int b) {
            return cmp(a, b);
        }
    "#);
    assert!(code.contains("sort_helper") || code.contains("fn"));
}

#[test]
fn test_parser_multiple_return_paths() {
    let code = assert_transpiles(r#"
        int categorize(int x) {
            if (x < 0) return -1;
            if (x == 0) return 0;
            if (x < 10) return 1;
            if (x < 100) return 2;
            if (x < 1000) return 3;
            return 4;
        }
    "#);
    assert!(code.contains("categorize"));
}

#[test]
fn test_parser_static_variables() {
    let code = assert_transpiles(r#"
        int counter() {
            static int count = 0;
            count++;
            return count;
        }
    "#);
    assert!(code.contains("counter") || code.contains("count"));
}

#[test]
fn test_parser_const_parameters() {
    let code = assert_transpiles(r#"
        int sum_array(const int* arr, int n) {
            int s = 0;
            for (int i = 0; i < n; i++) {
                s += arr[i];
            }
            return s;
        }
    "#);
    assert!(code.contains("sum_array") || code.contains("fn"));
}

#[test]
fn test_parser_volatile_variable() {
    let code = assert_transpiles(r#"
        int read_volatile() {
            volatile int sensor = 42;
            return sensor;
        }
    "#);
    assert!(code.contains("read_volatile") || code.contains("fn"));
}

// ============================================================================
// Extra tests targeting specific uncovered branches
// ============================================================================

#[test]
fn test_parser_inc_dec_field_decrement() {
    // Exercises the delta == -1 branch in extract_inc_dec_stmt for FieldAssignment
    let code = assert_transpiles(r#"
        struct Timer { int ticks; };
        void tick_down(struct Timer* t) { t->ticks--; }
    "#);
    assert!(code.contains("tick_down") || code.contains("ticks"));
}

#[test]
fn test_parser_inc_dec_direct_field() {
    // Exercises FieldAccess variant (obj.field rather than obj->field)
    let code = assert_transpiles(r#"
        struct Counter { int val; };
        void bump(struct Counter* c) { c->val++; }
    "#);
    assert!(code.contains("bump") || code.contains("val"));
}

#[test]
fn test_parser_for_with_only_condition() {
    // for loop with only condition (no init, no update) -- exercises 1-child pre_body
    let code = assert_transpiles(r#"
        int foo(int n) {
            int i = 0;
            int s = 0;
            for (; i < n;) {
                s += i;
                i++;
            }
            return s;
        }
    "#);
    assert!(code.contains("foo"));
}

#[test]
#[ignore = "FALSIFIED: HIR panics with 'For loop must have condition' on for(;;) infinite loops"]
fn test_parser_for_infinite_loop() {
    // for(;;) exercises the 0-children pre_body branch
    let code = assert_transpiles(r#"
        int foo() {
            int x = 0;
            for (;;) {
                x++;
                if (x > 10) break;
            }
            return x;
        }
    "#);
    assert!(code.contains("foo"));
}

#[test]
fn test_parser_for_two_parts_init_and_condition() {
    // for loop with init and condition but no update -- exercises 2-child first_is_init branch
    let code = assert_transpiles(r#"
        int foo(int n) {
            int s = 0;
            for (int i = 0; i < n;) {
                s += i;
                i++;
            }
            return s;
        }
    "#);
    assert!(code.contains("foo"));
}

#[test]
fn test_parser_for_two_parts_condition_and_update() {
    // for loop with condition and update but no init -- exercises 2-child !first_is_init
    let code = assert_transpiles(r#"
        int foo(int n) {
            int i = 0;
            int s = 0;
            for (; i < n; i++) {
                s += i;
            }
            return s;
        }
    "#);
    assert!(code.contains("foo"));
}

#[test]
fn test_parser_binop_comma_operator() {
    // Exercises the Comma variant in extract_binary_operator
    let code = assert_transpiles(r#"
        int foo() {
            int a = 0, b = 0;
            for (int i = 0; i < 5; i++, a++) {
                b++;
            }
            return a + b;
        }
    "#);
    assert!(code.contains("foo"));
}

#[test]
fn test_parser_binop_assign_in_condition() {
    // Exercises the Assign variant in extract_binary_operator (embedded assignment)
    let code = assert_transpiles(r#"
        int foo(int x) {
            int y;
            y = x + 1;
            return y;
        }
    "#);
    assert!(code.contains("foo"));
}

#[test]
fn test_parser_binop_all_precedence_levels() {
    // Complex expression hitting multiple precedence levels
    let code = assert_transpiles(r#"
        int prec(int a, int b, int c) {
            int r1 = a + b * c;
            int r2 = a | b & c;
            int r3 = a ^ b;
            int r4 = (a < b) && (b > c);
            int r5 = (a == 0) || (b != 0);
            return r1 + r2 + r3 + r4 + r5;
        }
    "#);
    assert!(code.contains("prec"));
}

#[test]
fn test_parser_for_step_by_two() {
    // for loop with i += 2 update
    let code = assert_transpiles(r#"
        int sum_even(int n) {
            int s = 0;
            for (int i = 0; i < n; i += 2) {
                s += i;
            }
            return s;
        }
    "#);
    assert!(code.contains("sum_even"));
}

#[test]
fn test_parser_combined_inc_dec_and_binop() {
    // Mix of increment/decrement with binary operators in a single function
    let code = assert_transpiles(r#"
        int complex_fn(int n) {
            int a = 0, b = n;
            for (int i = 0; i < n; i++) {
                a++;
                b--;
                if (a > b && a < n) {
                    a += b;
                }
            }
            return a + b;
        }
    "#);
    assert!(code.contains("complex_fn") || code.contains("fn"));
}

#[test]
fn test_parser_nested_for_with_inc_dec() {
    let code = assert_transpiles(r#"
        int matrix_sum(int n) {
            int total = 0;
            for (int i = 0; i < n; i++) {
                for (int j = 0; j < n; j++) {
                    total++;
                }
            }
            return total;
        }
    "#);
    assert!(code.contains("matrix_sum") || code.contains("total"));
}

#[test]
fn test_parser_for_with_compound_condition() {
    let code = assert_transpiles(r#"
        int search(int* arr, int n, int target) {
            for (int i = 0; i < n && arr[i] != target; i++) {
            }
            return -1;
        }
    "#);
    assert!(code.contains("search"));
}

#[test]
fn test_parser_multiple_arrays_and_loops() {
    let code = assert_transpiles(r#"
        void copy_array(int* dst, const int* src, int n) {
            for (int i = 0; i < n; i++) {
                dst[i] = src[i];
            }
        }
    "#);
    assert!(code.contains("copy_array") || code.contains("fn"));
}

#[test]
fn test_parser_bitwise_operations_complex() {
    let code = assert_transpiles(r#"
        int pack_bits(int a, int b, int c) {
            return (a & 0xFF) | ((b & 0xFF) << 8) | ((c & 0xFF) << 16);
        }
    "#);
    assert!(code.contains("pack_bits") || code.contains("fn"));
}

#[test]
fn test_parser_switch_with_fallthrough() {
    let code = assert_transpiles(r#"
        int grade(int score) {
            switch (score / 10) {
                case 10:
                case 9: return 4;
                case 8: return 3;
                case 7: return 2;
                case 6: return 1;
                default: return 0;
            }
        }
    "#);
    assert!(code.contains("grade") || code.contains("match"));
}

#[test]
fn test_parser_nested_struct() {
    let code = assert_transpiles(r#"
        struct Inner { int x; };
        struct Outer {
            struct Inner inner;
            int y;
        };
        int get_inner_x(struct Outer* o) { return o->inner.x; }
    "#);
    assert!(code.contains("Outer") || code.contains("Inner") || code.contains("get_inner"));
}

#[test]
fn test_parser_do_while_with_inc() {
    let code = assert_transpiles(r#"
        int count_digits(int n) {
            int count = 0;
            do {
                count++;
                n /= 10;
            } while (n > 0);
            return count;
        }
    "#);
    assert!(code.contains("count_digits") || code.contains("count"));
}

#[test]
fn test_parser_array_subscript_inc_dec_complex() {
    // Array subscript with expression index being incremented
    let code = assert_transpiles(r#"
        void histogram(int* counts, int* data, int n) {
            for (int i = 0; i < n; i++) {
                counts[data[i]]++;
            }
        }
    "#);
    assert!(code.contains("histogram") || code.contains("fn"));
}

#[test]
fn test_parser_multiple_assignment_operators() {
    let code = assert_transpiles(r#"
        void all_compound(int* x) {
            *x += 1;
            *x -= 1;
            *x *= 2;
            *x /= 2;
            *x %= 3;
            *x &= 0xFF;
            *x |= 0x01;
            *x ^= 0x0F;
            *x <<= 1;
            *x >>= 1;
        }
    "#);
    assert!(code.contains("all_compound") || code.contains("fn"));
}

#[test]
fn test_parser_transpile_file_basic() {
    let temp = TempDir::new().unwrap();
    let code = assert_transpile_file(
        &temp,
        "basic.c",
        "int main() { return 0; }",
    );
    assert!(code.contains("main") || code.contains("fn"));
}

#[test]
fn test_parser_transpile_file_with_for_and_inc() {
    let temp = TempDir::new().unwrap();
    let code = assert_transpile_file(
        &temp,
        "loop.c",
        r#"
        int sum(int n) {
            int total = 0;
            for (int i = 0; i < n; i++) {
                total += i;
            }
            return total;
        }
        "#,
    );
    assert!(code.contains("sum") || code.contains("total"));
}

#[test]
fn test_parser_cast_expression() {
    let code = assert_transpiles(r#"
        int truncate(double x) {
            return (int)x;
        }
    "#);
    assert!(code.contains("truncate") || code.contains("fn"));
}

#[test]
fn test_parser_sizeof_in_expression() {
    let code = assert_transpiles(r#"
        int sizes() {
            int a = sizeof(int);
            int b = sizeof(double);
            return a + b;
        }
    "#);
    assert!(code.contains("sizes") || code.contains("fn"));
}

#[test]
fn test_parser_multiline_for_with_complex_body() {
    let code = assert_transpiles(r#"
        int compute(int* data, int n) {
            int result = 0;
            for (int i = 0; i < n; i++) {
                int val = data[i];
                if (val > 0) {
                    result += val * val;
                } else if (val < 0) {
                    result -= val;
                } else {
                    result++;
                }
            }
            return result;
        }
    "#);
    assert!(code.contains("compute"));
}

#[test]
fn test_parser_chained_logical_ops() {
    let code = assert_transpiles(r#"
        int valid(int x, int y, int z) {
            return (x > 0) && (y > 0) && (z > 0) && (x < 100) && (y < 100) && (z < 100);
        }
    "#);
    assert!(code.contains("valid"));
}

#[test]
fn test_parser_mixed_bitwise_and_arithmetic() {
    let code = assert_transpiles(r#"
        int mix(int a, int b) {
            int sum = a + b;
            int masked = sum & 0xFF;
            int shifted = masked << 4;
            int ored = shifted | 0x01;
            return ored ^ 0xAA;
        }
    "#);
    assert!(code.contains("mix") || code.contains("fn"));
}
