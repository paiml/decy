//! DECY-127: Nested unsafe block tests.
//!
//! When generating unsafe blocks for raw pointer operations,
//! avoid unnecessary nesting that causes compiler warnings.
//!
//! Bad:  unsafe { *dest = unsafe { *src }; }
//! Good: unsafe { *dest = *src; }

use decy_core::transpile;

/// Test that deref assignment with deref RHS doesn't nest unsafe.
///
/// C: *dest = *src;  (where dest/src are raw pointers)
/// Expected: unsafe { *dest = *src; }  (single unsafe block)
/// Not: unsafe { *dest = unsafe { *src }; }
#[test]
fn test_deref_assign_deref_no_nested_unsafe() {
    let c_code = r#"
        void copy_char(char *dest, char *src) {
            dest = dest + 1;  // force raw pointer type
            src = src + 1;
            *dest = *src;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Count unsafe blocks - should not have nested unsafe
    let nested_count = result.matches("unsafe { *").count();
    let inside_unsafe = result.contains("= unsafe {");

    assert!(
        !inside_unsafe,
        "Should NOT have nested unsafe blocks\nGenerated:\n{}",
        result
    );
}

/// Test that while condition with deref is properly wrapped.
#[test]
fn test_while_deref_condition_single_unsafe() {
    let c_code = r#"
        void process(char *p) {
            p = p + 1;  // force raw pointer
            while (*p != '\0') {
                p = p + 1;
            }
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should have unsafe for the condition dereference
    assert!(
        result.contains("unsafe { *p }") || result.contains("unsafe { *p }"),
        "Should wrap condition deref in unsafe\nGenerated:\n{}",
        result
    );
}
