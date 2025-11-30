//! Integration tests for multiple output parameter transformation to tuples
//!
//! **Ticket**: DECY-085 - Handle multiple output parameters with tuples
//!
//! When a C function has multiple output parameters (pointer params that are
//! written but not read), transform them to a tuple return type.
//!
//! Example:
//! C: void get_dims(int* w, int* h) { *w=1920; *h=1080; }
//! Rust: fn get_dims() -> (i32, i32) { (1920, 1080) }

use decy_core::transpile;

/// Test 2-element tuple output
///
/// C: void get_dims(int *width, int *height) { *width = 1920; *height = 1080; }
/// Rust: fn get_dims() -> (i32, i32) { ... }
#[test]
fn test_two_output_params_become_tuple() {
    let c_code = r#"
void get_dims(int *width, int *height) {
    *width = 1920;
    *height = 1080;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Both output params should be removed from signature
    assert!(
        !rust_code.contains("width:") && !rust_code.contains("height:"),
        "DECY-085: Output params should be removed from signature!\nGot: {}",
        rust_code
    );

    // Should return tuple
    assert!(
        rust_code.contains("(i32, i32)"),
        "DECY-085: Should return tuple (i32, i32)!\nGot: {}",
        rust_code
    );
}

/// Test 3-element tuple output
#[test]
fn test_three_output_params_become_tuple() {
    let c_code = r#"
void get_color(int *r, int *g, int *b) {
    *r = 255;
    *g = 128;
    *b = 64;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // All output params should be removed
    assert!(
        !rust_code.contains("r:") && !rust_code.contains("g:") && !rust_code.contains("b:"),
        "DECY-085: All output params should be removed!\nGot: {}",
        rust_code
    );

    // Should return 3-tuple
    assert!(
        rust_code.contains("(i32, i32, i32)"),
        "DECY-085: Should return tuple (i32, i32, i32)!\nGot: {}",
        rust_code
    );
}

/// Test mixed input and output params
///
/// C: void scale(int factor, int *x, int *y) { *x = factor * 10; *y = factor * 20; }
/// Rust: fn scale(factor: i32) -> (i32, i32) { ... }
#[test]
fn test_mixed_input_output_params() {
    let c_code = r#"
void scale(int factor, int *x, int *y) {
    *x = factor * 10;
    *y = factor * 20;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // factor should remain (input param)
    assert!(
        rust_code.contains("factor"),
        "Input param 'factor' should remain!\nGot: {}",
        rust_code
    );

    // x and y should be removed (output params)
    assert!(
        !rust_code.contains("x:") && !rust_code.contains("y:"),
        "DECY-085: Output params x, y should be removed!\nGot: {}",
        rust_code
    );

    // Should return tuple
    assert!(
        rust_code.contains("(i32, i32)"),
        "DECY-085: Should return tuple (i32, i32)!\nGot: {}",
        rust_code
    );
}

/// Test single output param with input (from DECY-084 - should still work)
#[test]
fn test_single_output_with_inputs_still_works() {
    let c_code = r#"
void get_sum(int a, int b, int *result) {
    *result = a + b;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Should return i32 (single output = not a tuple)
    assert!(
        rust_code.contains("-> i32") && !rust_code.contains("(i32)"),
        "Single output should return i32, not tuple!\nGot: {}",
        rust_code
    );
}

/// Test fallible function with multiple outputs
///
/// C: int get_point(int *x, int *y) { *x = 10; *y = 20; return 0; }
/// Rust: fn get_point() -> Result<(i32, i32), i32> { ... }
#[test]
fn test_fallible_with_multiple_outputs() {
    let c_code = r#"
int get_point(int *x, int *y) {
    if (x == 0 || y == 0) return -1;
    *x = 10;
    *y = 20;
    return 0;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Should return Result with tuple
    assert!(
        rust_code.contains("Result<(i32, i32)"),
        "DECY-085: Fallible with 2 outputs should return Result<(i32, i32), ...>!\nGot: {}",
        rust_code
    );
}

/// Test that input-output params are NOT included in tuple
#[test]
fn test_input_output_not_in_tuple() {
    let c_code = r#"
void process(int *counter, int *result) {
    *counter = *counter + 1;
    *result = *counter * 10;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // counter is input-output (read before write), should stay as param
    // result is pure output, should become return
    // So we expect: fn process(counter: &mut i32) -> i32
    assert!(
        rust_code.contains("counter") && rust_code.contains("&mut"),
        "Input-output param 'counter' should remain as &mut!\nGot: {}",
        rust_code
    );
}

/// Test multiple output params of same type in tuple
#[test]
fn test_same_types_in_tuple() {
    let c_code = r#"
void get_info(int *count, int *total) {
    *count = 5;
    *total = 100;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Should generate tuple with same types
    assert!(
        rust_code.contains("(i32, i32)") || rust_code.contains("(i32,i32)"),
        "DECY-085: Should return tuple (i32, i32)!\nGot: {}",
        rust_code
    );
}

/// Test output param with output-like name triggers transformation
#[test]
fn test_output_name_triggers_tuple() {
    let c_code = r#"
void get_results(int *result1, int *result2) {
    *result1 = 100;
    *result2 = 200;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Both result params should be removed and tuple returned
    assert!(
        rust_code.contains("(i32, i32)"),
        "DECY-085: Two output params should become tuple!\nGot: {}",
        rust_code
    );
}
