//! End-to-end tests for extern "C" block support (DECY-055 RED phase)
//!
//! Tests verify that C code with extern "C" linkage specifications
//! transpiles correctly. These are used for C++ compatibility.
//!
//! References:
//! - K&R §A2.6: C/C++ interoperability
//! - DECY-051 validation: 80% of real-world headers use extern "C"

use decy_core::transpile;

#[test]
fn test_transpile_extern_c_bare_block() {
    // Test extern "C" without #ifdef guards
    let c_code = r#"
extern "C" {
    int add(int a, int b) {
        return a + b;
    }
}
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Should transpile the function, ignore the linkage spec
    assert!(rust_code.contains("fn add"), "Should contain function");
    assert!(rust_code.contains("i32"), "Should have Rust types");
    assert!(!rust_code.contains("extern"), "Should not contain extern");
}

#[test]
fn test_transpile_extern_c_with_declaration() {
    let c_code = r#"
extern "C" {
    int multiply(int a, int b);
}
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Function declarations should transpile
    assert!(
        rust_code.contains("multiply"),
        "Should contain function name"
    );
}

#[test]
fn test_transpile_extern_c_multiple_declarations() {
    let c_code = r#"
extern "C" {
    int add(int a, int b);
    int subtract(int a, int b);
    void print_result(int x);
}
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    assert!(rust_code.contains("add"));
    assert!(rust_code.contains("subtract"));
    assert!(rust_code.contains("print_result"));
}

#[test]
fn test_transpile_extern_c_with_ifdef_guards() {
    // This pattern already works but test for regression
    let c_code = r#"
#ifdef __cplusplus
extern "C" {
#endif

int add(int a, int b) {
    return a + b;
}

#ifdef __cplusplus
}
#endif
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    assert!(rust_code.contains("fn add"));
    assert!(!rust_code.contains("__cplusplus"));
    assert!(!rust_code.contains("extern"));
}

#[test]
fn test_transpile_extern_c_mixed_content() {
    let c_code = r#"
extern "C" {
    int global_var;

    int calculate(int x) {
        return x * 2;
    }

    typedef int (*callback_t)(int);
}
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    assert!(rust_code.contains("global_var"));
    assert!(rust_code.contains("calculate"));
    assert!(rust_code.contains("callback_t") || rust_code.contains("Callback"));
}

#[test]
fn test_transpile_without_extern_c() {
    // Control test - regular C without extern "C"
    let c_code = r#"
int add(int a, int b) {
    return a + b;
}
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    assert!(rust_code.contains("fn add"));
}

#[test]
fn test_transpile_extern_c_multiline_format() {
    // Test extern "C" on separate line from brace (miniz.c pattern)
    let c_code = r#"
extern "C"
{
    int my_function(int x);
}
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    assert!(rust_code.contains("my_function"));
}

#[test]
fn test_transpile_nested_functions_in_extern_c() {
    let c_code = r#"
extern "C" {
    int outer_function() {
        int inner_result = 42;
        return inner_result;
    }
}
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    assert!(rust_code.contains("outer_function"));
    assert!(rust_code.contains("inner_result"));
}
