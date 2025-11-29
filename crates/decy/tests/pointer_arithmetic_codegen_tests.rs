//! DECY-124: Pointer arithmetic codegen tests.
//!
//! When pointer arithmetic is performed on raw pointers,
//! generate proper Rust pointer methods.
//!
//! C: ptr = ptr + 1;  →  Rust: ptr = ptr.wrapping_add(1);
//! C: ptr = ptr - 1;  →  Rust: ptr = ptr.wrapping_sub(1);

use decy_core::transpile;

/// Test that pointer addition generates wrapping_add.
///
/// C: ptr = ptr + 1;
/// Expected Rust: ptr = ptr.wrapping_add(1);
#[test]
fn test_pointer_add_generates_wrapping_add() {
    let c_code = r#"
        void test(char *p) {
            p = p + 1;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should use wrapping_add for pointer arithmetic
    assert!(
        result.contains("wrapping_add"),
        "Should generate wrapping_add for ptr + n\nGenerated:\n{}",
        result
    );
}

/// Test that pointer subtraction generates wrapping_sub.
///
/// C: ptr = ptr - 1;
/// Expected Rust: ptr = ptr.wrapping_sub(1);
#[test]
fn test_pointer_sub_generates_wrapping_sub() {
    let c_code = r#"
        void test(char *p) {
            p = p - 1;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should use wrapping_sub for pointer arithmetic
    assert!(
        result.contains("wrapping_sub"),
        "Should generate wrapping_sub for ptr - n\nGenerated:\n{}",
        result
    );
}

/// Test that pointer dereference on raw pointer generates unsafe block.
///
/// C: *ptr = value;
/// Expected Rust: unsafe { *ptr = value; }
#[test]
fn test_raw_pointer_deref_generates_unsafe() {
    let c_code = r#"
        void test(char *p) {
            *p = 'x';
            p = p + 1;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should have unsafe block for raw pointer dereference
    assert!(
        result.contains("unsafe"),
        "Should generate unsafe for raw pointer dereference\nGenerated:\n{}",
        result
    );
}

/// Test string_copy pattern - full pointer arithmetic loop.
///
/// C: while (*src) { *dest++ = *src++; }
/// Expected: Uses wrapping_add and unsafe properly
#[test]
fn test_string_copy_pattern_compiles() {
    let c_code = r#"
        void str_copy(char *dest, char *src) {
            while (*src != '\0') {
                *dest = *src;
                dest = dest + 1;
                src = src + 1;
            }
            *dest = '\0';
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should have wrapping_add for both dest and src
    let add_count = result.matches("wrapping_add").count();
    assert!(
        add_count >= 2,
        "Should have at least 2 wrapping_add calls (for dest and src)\nFound: {}\nGenerated:\n{}",
        add_count,
        result
    );

    // Should have unsafe for dereferences
    assert!(
        result.contains("unsafe"),
        "Should have unsafe blocks for raw pointer operations\nGenerated:\n{}",
        result
    );
}
