//! DECY-123: Pointer arithmetic detection tests.
//!
//! When a pointer parameter uses pointer arithmetic (p = p + 1),
//! it should NOT be transformed to a reference.
//!
//! C: void f(int *p) { p = p + 1; }
//! Expected: fn f(p: *mut i32) - keep as raw pointer
//!
//! Note: char* with pointer arithmetic triggers DECY-134 (string iteration)
//! which transforms to slice + index (safer than raw pointers).

use decy_core::transpile;

/// Test that int* pointer arithmetic is detected and parameter stays as raw pointer.
///
/// Note: Using int* to test raw pointer preservation.
/// char* triggers DECY-134 string iteration (safer slice transform).
///
/// C: void f(int *p) { p = p + 1; }
/// Expected: fn f(p: *mut i32) - NOT &mut i32
#[test]
fn test_pointer_arithmetic_detection_int() {
    let c_code = r#"
        void test(int *p) {
            p = p + 1;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should keep as raw pointer due to pointer arithmetic
    assert!(
        result.contains("*mut i32") || result.contains("*const i32"),
        "Should keep int* pointer arithmetic param as raw pointer\nGenerated:\n{}",
        result
    );
}

/// Test that int* pointer subtraction is also detected.
///
/// C: void f(int *p) { p = p - 1; }
/// Expected: fn f(p: *mut i32)
#[test]
fn test_pointer_subtraction_detection_int() {
    let c_code = r#"
        void test(int *p) {
            p = p - 1;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should keep as raw pointer due to pointer arithmetic
    assert!(
        result.contains("*mut i32") || result.contains("*const i32"),
        "Should keep int* pointer subtraction param as raw pointer\nGenerated:\n{}",
        result
    );
}

/// Test that char* pointer arithmetic triggers DECY-134 slice transform.
///
/// DECY-134: char* with pointer arithmetic is transformed to slice + index
/// for safe string iteration (preferred over raw pointers).
///
/// C: void f(char *p) { p = p + 1; }
/// Expected: fn f(p: &[u8]) with p_idx indexing
#[test]
fn test_char_ptr_arithmetic_becomes_slice() {
    let c_code = r#"
        void test(char *p) {
            p = p + 1;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // DECY-134: char* with arithmetic → slice (safer than raw pointer)
    assert!(
        result.contains("&[u8]") || result.contains("_idx"),
        "char* with pointer arithmetic should use DECY-134 slice pattern\nGenerated:\n{}",
        result
    );
}

/// Test that simple dereference without arithmetic becomes reference.
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
