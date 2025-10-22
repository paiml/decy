//! End-to-end tests for typedef compile-time assertion support (DECY-057 RED phase)
//!
//! Tests verify that C typedef array assertions transpile correctly to Rust.
//! These are compile-time assertions used in portable C code.
//!
//! References:
//! - ISO C99 §6.7.2.1: Array declarators
//! - DECY-051 validation: Common pattern in miniz.c and production C

use decy_core::transpile;

#[test]
fn test_transpile_typedef_simple_array_assertion() {
    // Simple typedef array assertion with sizeof
    let c_code = r#"
typedef unsigned char validate_int_size[sizeof(int) == 4 ? 1 : -1];

int main() {
    return 0;
}
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Should generate Rust const assertion
    assert!(
        rust_code.contains("const _") || rust_code.contains("assert!"),
        "Should generate const assertion"
    );
    assert!(
        rust_code.contains("size_of") || rust_code.contains("i32"),
        "Should check type size"
    );
}

#[test]
fn test_transpile_typedef_uint16_assertion() {
    // Real-world pattern from miniz.c
    let c_code = r#"
typedef unsigned short mz_uint16;
typedef unsigned char mz_validate_uint16[sizeof(mz_uint16) == 2 ? 1 : -1];

int main() {
    return 0;
}
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Should have both typedef and assertion
    assert!(
        rust_code.contains("u16") || rust_code.contains("mz_uint16"),
        "Should contain uint16 typedef"
    );
    assert!(
        rust_code.contains("assert!") || rust_code.contains("const _"),
        "Should contain compile-time assertion"
    );
    assert!(rust_code.contains("2"), "Should check size == 2");
}

#[test]
fn test_transpile_typedef_pointer_size_assertion() {
    // Check pointer size (common in 32/64-bit compatibility)
    let c_code = r#"
typedef char check_ptr_size[sizeof(void*) == 8 ? 1 : -1];

int main() {
    return 0;
}
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Should generate pointer size check
    assert!(
        rust_code.contains("*const") || rust_code.contains("pointer"),
        "Should check pointer type"
    );
    assert!(rust_code.contains("8"), "Should check size == 8");
}

#[test]
fn test_transpile_multiple_typedef_assertions() {
    // Multiple assertions in one file
    let c_code = r#"
typedef char check_int[sizeof(int) == 4 ? 1 : -1];
typedef char check_long[sizeof(long) == 8 ? 1 : -1];
typedef char check_ptr[sizeof(void*) == 8 ? 1 : -1];

int main() {
    return 0;
}
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Should generate multiple assertions
    let assertion_count = rust_code.matches("const _").count() + rust_code.matches("assert!").count();
    assert!(assertion_count >= 3, "Should have 3 compile-time assertions");
}

#[test]
fn test_transpile_typedef_assertion_negative_case() {
    // Assertion that should fail (for testing)
    let c_code = r#"
typedef char always_fail[1 == 2 ? 1 : -1];

int main() {
    return 0;
}
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Should generate assertion that will fail at compile time
    // The transpiled Rust code should contain the failing condition
    assert!(
        rust_code.contains("1 == 2") || rust_code.contains("false"),
        "Should preserve failing condition"
    );
}

#[test]
fn test_transpile_typedef_assertion_with_struct() {
    // Assertion about struct size
    let c_code = r#"
struct Point {
    int x;
    int y;
};

typedef char check_point_size[sizeof(struct Point) == 8 ? 1 : -1];

int main() {
    return 0;
}
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Should have both struct and assertion
    assert!(
        rust_code.contains("struct Point") || rust_code.contains("Point"),
        "Should contain Point struct"
    );
    assert!(
        rust_code.contains("size_of") || rust_code.contains("assert!"),
        "Should check struct size"
    );
}

#[test]
fn test_parse_typedef_without_assertion() {
    // Regular typedef (not an assertion) should still work
    let c_code = r#"
typedef unsigned char byte;
typedef byte buffer[256];

int main() {
    return 0;
}
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Should have regular typedef
    assert!(
        rust_code.contains("byte") || rust_code.contains("u8"),
        "Should contain byte typedef"
    );
}

#[test]
fn test_transpile_typedef_assertion_complex_expression() {
    // More complex expression in assertion
    let c_code = r#"
typedef char check_alignment[sizeof(long) >= sizeof(int) ? 1 : -1];

int main() {
    return 0;
}
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Should handle >= operator
    assert!(
        rust_code.contains(">=") || rust_code.contains("assert!"),
        "Should handle >= comparison"
    );
}

#[test]
fn test_transpile_typedef_assertion_generates_valid_rust() {
    // Ensure generated Rust code is syntactically valid
    let c_code = r#"
typedef char validate[sizeof(int) == 4 ? 1 : -1];

int add(int a, int b) {
    return a + b;
}
    "#;

    let rust_code = transpile(c_code).expect("Transpilation should succeed");

    // Basic sanity checks for valid Rust
    assert!(!rust_code.is_empty(), "Should generate non-empty code");
    assert!(rust_code.contains("fn add"), "Should contain function");

    // Check for balanced braces
    let open_braces = rust_code.matches('{').count();
    let close_braces = rust_code.matches('}').count();
    assert_eq!(
        open_braces, close_braces,
        "Should have balanced braces"
    );
}
