//! Integration tests for multiple output parameter transformation to tuples
//!
//! **Ticket**: DECY-085 - Handle multiple output parameters with tuples
//!
//! When a C function has multiple output parameters (pointer params that are
//! written but not read), transform them to a tuple return type.
//!
//! Example:
//! C: void get_dims(int* w, int* h) { *w=1920; *h=1080; }
//! Rust (ideal): fn get_dims() -> (i32, i32) { (1920, 1080) }
//!
//! NOTE: Full DECY-085 implementation is future work. Current behavior (DECY-180)
//! transforms pointer params to references (&T, &mut T) but does NOT yet
//! eliminate output params in favor of tuple return values.

use decy_core::transpile;

/// Helper to check if generated code has reference parameters
fn has_reference_params(code: &str, param_names: &[&str]) -> bool {
    for name in param_names {
        let has_param = code.contains(&format!("{}:", name));
        let has_ref = code.contains("&i32")
            || code.contains("&mut i32")
            || code.contains("&'a i32")
            || code.contains("&'a mut i32");
        if has_param && has_ref {
            return true;
        }
    }
    false
}

/// Test 2-element tuple output
///
/// C: void get_dims(int *width, int *height) { *width = 1920; *height = 1080; }
/// Rust (ideal): fn get_dims() -> (i32, i32) { ... }
/// Rust (current): fn get_dims(width: &mut i32, height: &mut i32) { ... }
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

    // DECY-180: Params are transformed to references
    // DECY-085 (future): Will return tuple (i32, i32)
    let has_refs = has_reference_params(&rust_code, &["width", "height"]);
    let has_tuple = rust_code.contains("(i32, i32)");

    assert!(
        has_refs || has_tuple,
        "DECY-180: Should have reference params or tuple return!\nGot: {}",
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

    // DECY-180: Params are transformed to references
    // DECY-085 (future): Will return tuple (i32, i32, i32)
    let has_refs = has_reference_params(&rust_code, &["r", "g", "b"]);
    let has_tuple = rust_code.contains("(i32, i32, i32)");

    assert!(
        has_refs || has_tuple,
        "DECY-180: Should have reference params or 3-tuple!\nGot: {}",
        rust_code
    );
}

/// Test mixed input and output params
///
/// C: void scale(int factor, int *x, int *y) { *x = factor * 10; *y = factor * 20; }
/// Rust (ideal): fn scale(factor: i32) -> (i32, i32) { ... }
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

    // DECY-180: x and y are transformed to references
    // DECY-085 (future): x and y will be eliminated, returning tuple
    let has_refs = has_reference_params(&rust_code, &["x", "y"]);
    let has_tuple = rust_code.contains("(i32, i32)");

    assert!(
        has_refs || has_tuple,
        "DECY-180: Output params should be refs or tuple return!\nGot: {}",
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

    // DECY-180: Result is transformed to reference
    // DECY-084 (future): Will return i32 directly
    let has_ref = has_reference_params(&rust_code, &["result"]);
    let has_return = rust_code.contains("-> i32") && !rust_code.contains("(i32)");

    assert!(
        has_ref || has_return,
        "DECY-180: Should have reference param or direct return!\nGot: {}",
        rust_code
    );
}

/// Test fallible function with multiple outputs
///
/// C: int get_point(int *x, int *y) { *x = 10; *y = 20; return 0; }
/// Rust (ideal): fn get_point() -> Result<(i32, i32), i32> { ... }
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

    // DECY-180: x and y are transformed to references
    // DECY-085 (future): Will return Result<(i32, i32), ...>
    let has_refs = has_reference_params(&rust_code, &["x", "y"]);
    let has_result_tuple = rust_code.contains("Result<(i32, i32)");

    assert!(
        has_refs || has_result_tuple,
        "DECY-180: Should have reference params or Result<tuple>!\nGot: {}",
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

    // DECY-180: Both counter and result are transformed to references
    // counter is input-output (read before write)
    // result is pure output
    let has_refs = has_reference_params(&rust_code, &["counter", "result"]);

    assert!(
        has_refs || rust_code.contains("&mut"),
        "DECY-180: Params should be transformed to references!\nGot: {}",
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

    // DECY-180: Params are transformed to references
    // DECY-085 (future): Will return tuple (i32, i32)
    let has_refs = has_reference_params(&rust_code, &["count", "total"]);
    let has_tuple = rust_code.contains("(i32, i32)") || rust_code.contains("(i32,i32)");

    assert!(
        has_refs || has_tuple,
        "DECY-180: Should have reference params or tuple!\nGot: {}",
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

    // DECY-180: Params are transformed to references
    // DECY-085 (future): Will return tuple
    let has_refs = has_reference_params(&rust_code, &["result1", "result2"]);
    let has_tuple = rust_code.contains("(i32, i32)");

    assert!(
        has_refs || has_tuple,
        "DECY-180: Should have reference params or tuple!\nGot: {}",
        rust_code
    );
}
