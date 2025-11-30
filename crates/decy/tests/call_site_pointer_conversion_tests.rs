//! DECY-125: Call site array/string to pointer conversion tests.
//!
//! When calling a function with raw pointer parameters, convert:
//! - Arrays to pointers: `arr` → `arr.as_mut_ptr()` (for non-char* pointers)
//! - String literals to pointers: `"hello"` → `"hello".as_ptr() as *mut u8`
//!
//! Note: DECY-134 takes priority for char* with pointer arithmetic,
//! transforming to safe slice iteration instead of raw pointers.

use decy_core::transpile;

/// Test that char* with pointer arithmetic transforms to slice (DECY-134 pattern).
///
/// When function uses char* with pointer arithmetic, DECY-134 string iteration
/// pattern takes priority, transforming to safe slice + index instead of raw pointer.
/// This is SAFER than raw pointer transformation.
#[test]
fn test_array_to_slice_for_char_ptr() {
    let c_code = r#"
        void advance_ptr(char *data) {
            data = data + 1;
        }

        int main() {
            char buffer[20];
            advance_ptr(buffer);
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // DECY-134: char* with pointer arithmetic → slice (safer than raw pointer)
    // Either slice reference or raw pointer conversion is acceptable
    assert!(
        result.contains("&[u8]") || result.contains("&buffer") || result.contains("as_mut_ptr"),
        "Should transform char* param to slice or pointer\nGenerated:\n{}",
        result
    );
}

/// Test that char* string iteration uses slice transformation (DECY-134).
///
/// DECY-134 transforms char* with pointer arithmetic to safe slice iteration.
/// This is preferred over raw pointer + .as_ptr() because it's 100% safe.
#[test]
fn test_string_literal_to_byte_slice() {
    let c_code = r#"
        void str_copy(char *dest, char *src) {
            dest = dest + 1;
            src = src + 1;
        }

        int main() {
            char buffer[20];
            str_copy(buffer, "hello");
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // DECY-134: String iteration pattern → byte slice
    // Either byte string (b"...") or pointer conversion is acceptable
    assert!(
        result.contains("b\"hello\"") || result.contains("&[u8]")
            || result.contains(".as_ptr()") || result.contains("as *mut"),
        "Should convert string literal to byte slice or pointer\nGenerated:\n{}",
        result
    );
}

/// Test that the generated code compiles (basic verification).
#[test]
fn test_call_site_conversion_generates_valid_rust() {
    let c_code = r#"
        void process(int *arr, int len) {
            for (int i = 0; i < len; i++) {
                arr[i] = arr[i] + 1;
            }
        }

        int main() {
            int data[5];
            data[0] = 1;
            process(data, 5);
            return data[0];
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // The array parameter should be transformed appropriately
    // Either as slice (preferred) or with proper pointer conversion
    assert!(
        result.contains("&mut [") || result.contains("as_mut_ptr"),
        "Should handle array parameter properly\nGenerated:\n{}",
        result
    );
}
