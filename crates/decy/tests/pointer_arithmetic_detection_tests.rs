//! DECY-123: Pointer arithmetic detection tests.
//!
//! When a pointer parameter uses pointer arithmetic (p = p + 1),
//! it should NOT be transformed to a reference.
//!
//! C: void f(char *p) { p = p + 1; }
//! Expected: fn f(p: *mut u8) - keep as raw pointer

use decy_core::transpile;

/// Test that pointer arithmetic is detected and parameter stays as raw pointer.
///
/// C: void f(char *p) { p = p + 1; }
/// Expected: fn f(p: *mut u8) - NOT &mut u8
#[test]
fn test_pointer_arithmetic_detection() {
    let c_code = r#"
        void test(char *p) {
            p = p + 1;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should keep as raw pointer due to pointer arithmetic
    assert!(
        result.contains("*mut u8") || result.contains("*const u8"),
        "Should keep pointer arithmetic param as raw pointer\nGenerated:\n{}",
        result
    );

    // Should NOT convert to reference
    assert!(
        !result.contains("&mut u8") && !result.contains("& u8"),
        "Should NOT convert to reference when pointer arithmetic is used\nGenerated:\n{}",
        result
    );
}

/// Test that pointer subtraction is also detected.
///
/// C: void f(char *p) { p = p - 1; }
/// Expected: fn f(p: *mut u8)
#[test]
fn test_pointer_subtraction_detection() {
    let c_code = r#"
        void test(char *p) {
            p = p - 1;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should keep as raw pointer due to pointer arithmetic
    assert!(
        result.contains("*mut u8") || result.contains("*const u8"),
        "Should keep pointer subtraction param as raw pointer\nGenerated:\n{}",
        result
    );
}

/// Test that simple dereference without arithmetic still becomes reference.
///
/// C: void f(char *p) { *p = 'x'; }
/// Expected: fn f(p: &mut u8)
#[test]
fn test_no_arithmetic_becomes_reference() {
    let c_code = r#"
        void test(char *p) {
            *p = 'x';
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should convert to reference (no pointer arithmetic)
    assert!(
        result.contains("&mut u8"),
        "Should convert to reference when no pointer arithmetic\nGenerated:\n{}",
        result
    );
}
