//! Integration tests for output parameter transformation
//!
//! **Ticket**: DECY-084 - Generate Result/Option returns for output params
//!
//! These tests verify that C output parameters are transformed to
//! idiomatic Rust return values:
//! - Non-fallible output → direct return T
//! - Fallible output (with error codes) → Result<T, Error>
//! - Optional output → Option<T>

use decy_core::transpile;

/// Test non-fallible output parameter transforms to direct return
///
/// C: void get_sum(int a, int b, int *result) { *result = a + b; }
/// Rust: fn get_sum(a: i32, b: i32) -> i32 { a + b }
#[test]
fn test_nonfallible_output_param_to_return() {
    let c_code = r#"
void get_sum(int a, int b, int *result) {
    *result = a + b;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Should NOT have output parameter in signature
    assert!(
        !rust_code.contains("result: &mut"),
        "DECY-084: Output param should be eliminated from signature!\nGot: {}",
        rust_code
    );

    // Should return i32 directly
    assert!(
        rust_code.contains("-> i32") || rust_code.contains("->i32"),
        "DECY-084: Should return i32 directly!\nGot: {}",
        rust_code
    );
}

/// Test fallible output parameter transforms to Result
///
/// C: int parse(const char *s, int *result) { if (!s) return -1; *result = 42; return 0; }
/// Rust: fn parse(s: &str) -> Result<i32, Error> { ... }
#[test]
fn test_fallible_output_param_to_result() {
    let c_code = r#"
int parse_value(const char *str, int *result) {
    if (str == 0) {
        return -1;
    }
    *result = 42;
    return 0;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Should NOT have output parameter in signature
    assert!(
        !rust_code.contains("result: &mut"),
        "DECY-084: Output param should be eliminated from signature!\nGot: {}",
        rust_code
    );

    // Should return Result type
    assert!(
        rust_code.contains("Result<") || rust_code.contains("-> Result"),
        "DECY-084: Fallible function should return Result!\nGot: {}",
        rust_code
    );
}

/// Test that input-output params are NOT transformed (they're still needed)
///
/// C: void increment(int *value) { *value = *value + 1; }
/// Should remain: fn increment(value: &mut i32) { ... }
#[test]
fn test_input_output_param_preserved() {
    let c_code = r#"
void increment(int *value) {
    *value = *value + 1;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Input-output params should be preserved (reads before writes)
    assert!(
        rust_code.contains("value: &mut i32") || rust_code.contains("&mut i32"),
        "Input-output param should be preserved!\nGot: {}",
        rust_code
    );
}

/// Test output param with void return becomes direct return
#[test]
fn test_void_function_with_output_param() {
    let c_code = r#"
void compute_square(int x, int *result) {
    *result = x * x;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Should transform void + output param to returning value
    assert!(
        rust_code.contains("-> i32") && !rust_code.contains("result: &mut"),
        "DECY-084: void + output param should become -> T!\nGot: {}",
        rust_code
    );
}

/// Test multiple output params (related to DECY-085, but basic case)
#[test]
fn test_single_output_param_basic() {
    let c_code = r#"
int get_length(const char *str, int *len) {
    if (str == 0) return -1;
    *len = 5;
    return 0;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Output param 'len' should not appear in signature
    assert!(
        !rust_code.contains("len: &mut"),
        "DECY-084: Output param 'len' should be eliminated!\nGot: {}",
        rust_code
    );
}

/// Test that const pointer params are not treated as output params
#[test]
fn test_const_pointer_not_output() {
    let c_code = r#"
int read_value(const int *ptr) {
    return *ptr;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // const pointer should become &i32, not &mut i32
    assert!(
        rust_code.contains("ptr: &i32") || rust_code.contains(": &i32"),
        "const pointer should be immutable reference!\nGot: {}",
        rust_code
    );
}
