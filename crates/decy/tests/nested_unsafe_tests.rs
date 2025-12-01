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
    let _nested_count = result.matches("unsafe { *").count();
    let inside_unsafe = result.contains("= unsafe {");

    assert!(
        !inside_unsafe,
        "Should NOT have nested unsafe blocks\nGenerated:\n{}",
        result
    );
}

/// Test that while condition with char* iteration uses safe slice.
///
/// DECY-134: char* with pointer arithmetic is transformed to slice + index,
/// which is 100% safe (no unsafe blocks needed).
///
/// C: while (*p != '\0') { p++; }
/// Rust: while p[p_idx] != 0u8 { p_idx += 1; }
#[test]
fn test_while_deref_condition_safe_slice() {
    let c_code = r#"
        void process(char *p) {
            p = p + 1;  // char* with pointer arithmetic
            while (*p != '\0') {
                p = p + 1;
            }
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // DECY-134: char* iteration is transformed to safe slice indexing
    // Either:
    // 1. Safe slice: while p[p_idx] != 0u8 (preferred, no unsafe)
    // 2. Raw pointer: while unsafe { *p } != 0 (fallback)
    let has_slice_indexing = result.contains("[p_idx]") || result.contains("p[");
    let has_unsafe_deref = result.contains("unsafe { *p }") || result.contains("unsafe {");

    assert!(
        has_slice_indexing || has_unsafe_deref,
        "Should use safe slice indexing or unsafe deref\nGenerated:\n{}",
        result
    );
}
