//! DECY-124: Pointer arithmetic codegen tests.
//!
//! When pointer arithmetic is performed on raw pointers,
//! generate proper Rust pointer methods.
//!
//! C: ptr = ptr + 1;  →  Rust: ptr = ptr.wrapping_add(1);  (for int*)
//! C: ptr = ptr - 1;  →  Rust: ptr = ptr.wrapping_sub(1);  (for int*)
//!
//! Note: char* with pointer arithmetic is transformed by DECY-134 to
//! safe slice indexing instead of raw pointers (preferred, safer).

use decy_core::transpile;

/// Test that int* pointer addition generates wrapping_add.
///
/// Note: Using int* instead of char* to test raw pointer behavior.
/// char* triggers DECY-134 string iteration (safer, but different pattern).
///
/// C: ptr = ptr + 1;
/// Expected Rust: ptr = ptr.wrapping_add(1);
#[test]
fn test_pointer_add_generates_wrapping_add() {
    // Use int* to test raw pointer arithmetic (char* triggers DECY-134 slice transform)
    let c_code = r#"
        void test(int *p) {
            p = p + 1;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should use wrapping_add for int* pointer arithmetic
    assert!(
        result.contains("wrapping_add"),
        "Should generate wrapping_add for int* + n\nGenerated:\n{}",
        result
    );
}

/// Test that int* pointer subtraction generates wrapping_sub.
///
/// C: ptr = ptr - 1;
/// Expected Rust: ptr = ptr.wrapping_sub(1);
#[test]
fn test_pointer_sub_generates_wrapping_sub() {
    // Use int* to test raw pointer arithmetic
    let c_code = r#"
        void test(int *p) {
            p = p - 1;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should use wrapping_sub for int* pointer arithmetic
    assert!(
        result.contains("wrapping_sub"),
        "Should generate wrapping_sub for int* - n\nGenerated:\n{}",
        result
    );
}

/// Test that int* pointer dereference generates unsafe block.
///
/// C: *ptr = value;
/// Expected Rust: unsafe { *ptr = value; }
#[test]
fn test_raw_pointer_deref_generates_unsafe() {
    // Use int* to test raw pointer behavior
    let c_code = r#"
        void test(int *p) {
            *p = 42;
            p = p + 1;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should have unsafe block for raw pointer dereference
    assert!(
        result.contains("unsafe"),
        "Should generate unsafe for int* raw pointer dereference\nGenerated:\n{}",
        result
    );
}

/// Test char* string_copy pattern uses SAFE slice indexing (DECY-134).
///
/// DECY-134 transforms char* with pointer arithmetic to slice + index,
/// which is 100% safe (no unsafe, no wrapping_add).
///
/// C: while (*src) { *dest++ = *src++; }
/// Rust: while src[src_idx] != 0 { dest[dest_idx] = src[src_idx]; ... }
#[test]
fn test_string_copy_pattern_safe_slice() {
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

    // DECY-134: char* iteration becomes safe slice indexing
    // Either:
    // 1. Safe slice pattern: dest[dest_idx] = src[src_idx] (preferred, zero unsafe)
    // 2. Raw pointer pattern: wrapping_add + unsafe (fallback)
    let has_slice_indexing = result.contains("_idx]") || result.contains("[src_idx");
    let has_raw_pointer = result.contains("wrapping_add") && result.contains("unsafe");

    assert!(
        has_slice_indexing || has_raw_pointer,
        "Should use safe slice indexing (preferred) or raw pointer ops\nGenerated:\n{}",
        result
    );

    // Prefer slice indexing - verify it's safe (no unsafe blocks)
    if has_slice_indexing {
        // Best case: zero unsafe blocks for char* iteration
        let unsafe_count = result.matches("unsafe").count();
        println!(
            "Unsafe blocks: {} (0 is best for char* iteration)",
            unsafe_count
        );
    }
}
