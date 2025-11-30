//! DECY-134: String copy patterns → safe slice iteration
//!
//! C string copy with pointer arithmetic should transform to safe Rust:
//! - `char *dest, char *src` → `&mut [u8], &[u8]`
//! - `while (*src)` loop → iterator or index-based loop
//! - No unsafe blocks needed for string operations

use decy_core::transpile;

/// Test string copy function transforms to safe slice operations.
///
/// C:   void str_copy(char *dest, char *src) { while(*src) { *dest++ = *src++; } }
/// Rust: Should use slice indexing or iterators, NOT raw pointers
#[test]
fn test_string_copy_no_unsafe() {
    let c_code = r#"
        void str_copy(char *dest, char *src) {
            while (*src != '\0') {
                *dest = *src;
                dest = dest + 1;
                src = src + 1;
            }
            *dest = '\0';
        }

        int main() {
            char buffer[20];
            str_copy(buffer, "hello");
            return buffer[0];
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Count unsafe blocks - target is 0 for string operations
    let unsafe_count = result.matches("unsafe").count();

    // String copy should be safe - no unsafe blocks
    assert!(
        unsafe_count == 0,
        "String copy should not require unsafe blocks. Found {} unsafe blocks.\nGenerated:\n{}",
        unsafe_count,
        result
    );
}

/// Test that simple string iteration uses slices.
///
/// C:   int strlen(char *s) { int n=0; while(*s++) n++; return n; }
/// Rust: Should use .iter() or slice indexing
#[test]
fn test_strlen_pattern_no_unsafe() {
    let c_code = r#"
        int my_strlen(char *s) {
            int n = 0;
            while (*s != '\0') {
                n = n + 1;
                s = s + 1;
            }
            return n;
        }

        int main() {
            return my_strlen("hello");
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    let unsafe_count = result.matches("unsafe").count();

    assert!(
        unsafe_count == 0,
        "strlen pattern should not require unsafe. Found {} unsafe blocks.\nGenerated:\n{}",
        unsafe_count,
        result
    );
}
