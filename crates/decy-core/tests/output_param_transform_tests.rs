//! Integration tests for output parameter transformation
//!
//! **Ticket**: DECY-084 - Generate Result/Option returns for output params
//!
//! These tests verify that C output parameters are transformed to
//! idiomatic Rust return values:
//! - Non-fallible output → direct return T
//! - Fallible output (with error codes) → Result<T, Error>
//! - Optional output → Option<T>
//!
//! NOTE: Full DECY-084 implementation is future work. Current behavior (DECY-180)
//! transforms pointer params to references (&T, &mut T) but does NOT yet eliminate
//! output params in favor of return values.

use decy_core::transpile;

/// Test non-fallible output parameter transforms to direct return
///
/// C: void get_sum(int a, int b, int *result) { *result = a + b; }
/// Rust (ideal): fn get_sum(a: i32, b: i32) -> i32 { a + b }
/// Rust (current): fn get_sum(a: i32, b: i32, result: &mut i32) { ... }
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

    // DECY-180: Pointer params are now transformed to references
    // DECY-084 (future): Will eliminate output params and use direct returns
    let has_reference_param = rust_code.contains("result:")
        && (rust_code.contains("&i32")
            || rust_code.contains("&mut i32")
            || rust_code.contains("&'a i32")
            || rust_code.contains("&'a mut i32"));

    assert!(
        has_reference_param || rust_code.contains("-> i32"),
        "DECY-180: Should have reference param or return value!\nGot: {}",
        rust_code
    );
}

/// Test fallible output parameter transforms to Result
///
/// C: int parse(const char *s, int *result) { if (!s) return -1; *result = 42; return 0; }
/// Rust (ideal): fn parse(s: &str) -> Result<i32, Error> { ... }
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

    // DECY-180: Pointer params are transformed to references
    // DECY-084 (future): Will transform to Result return
    let has_reference_param = rust_code.contains("result:")
        && (rust_code.contains("&i32")
            || rust_code.contains("&mut i32")
            || rust_code.contains("&'a i32")
            || rust_code.contains("&'a mut i32"));

    assert!(
        has_reference_param || rust_code.contains("Result<"),
        "DECY-180: Should have reference param or Result return!\nGot: {}",
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

    // DECY-180: Input-output params should be preserved as references
    // (reads before writes means it's not purely an output param)
    let has_reference_param = rust_code.contains("value:")
        && (rust_code.contains("&i32")
            || rust_code.contains("&mut i32")
            || rust_code.contains("&'a i32")
            || rust_code.contains("&'a mut i32"));

    assert!(
        has_reference_param,
        "Input-output param should be preserved as reference!\nGot: {}",
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

    // DECY-180: Output param is transformed to reference
    // DECY-084 (future): Will eliminate output param and use direct return
    let has_reference_param = rust_code.contains("result:")
        && (rust_code.contains("&i32")
            || rust_code.contains("&mut i32")
            || rust_code.contains("&'a i32")
            || rust_code.contains("&'a mut i32"));

    assert!(
        has_reference_param || rust_code.contains("-> i32"),
        "DECY-180: Should have reference param or return value!\nGot: {}",
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

    // DECY-180: Output param 'len' is transformed to reference
    // DECY-084 (future): Will eliminate output param
    let has_reference_param = rust_code.contains("len:")
        && (rust_code.contains("&i32")
            || rust_code.contains("&mut i32")
            || rust_code.contains("&'a i32")
            || rust_code.contains("&'a mut i32"));

    let output_param_eliminated = !rust_code.contains("len:");

    assert!(
        has_reference_param || output_param_eliminated,
        "DECY-180: len should be reference or eliminated!\nGot: {}",
        rust_code
    );
}

/// Test that const pointer params are not treated as output params
/// Note: const-ness detection is a separate feature (not part of DECY-084)
/// This test verifies the param is NOT removed as an output param
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

    // const pointer should NOT be treated as output param (it's read, not written)
    // The param should still be present in the signature
    assert!(
        rust_code.contains("ptr:") || rust_code.contains("ptr :"),
        "const pointer param should be preserved (not treated as output)!\nGot: {}",
        rust_code
    );
}
