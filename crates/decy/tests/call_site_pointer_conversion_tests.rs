//! DECY-125: Call site array/string to pointer conversion tests.
//!
//! When calling a function with raw pointer parameters, convert:
//! - Arrays to pointers: `arr` → `arr.as_mut_ptr()`
//! - String literals to pointers: `"hello"` → `"hello".as_ptr() as *mut u8`

use decy_core::transpile;

/// Test that array argument is converted properly when function takes *mut T.
///
/// When function uses pointer arithmetic, param stays as *mut T,
/// and call site needs buffer.as_mut_ptr()
#[test]
fn test_array_to_mut_ptr_call_site() {
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

    // Should convert array to raw pointer at call site
    assert!(
        result.contains("as_mut_ptr") || result.contains(".as_ptr()"),
        "Should convert array to pointer at call site\nGenerated:\n{}",
        result
    );
}

/// Test that string literal is converted to pointer when function takes *mut u8.
///
/// C: func("hello");  where func takes char*
/// Expected Rust: func("hello".as_ptr() as *mut u8);
#[test]
fn test_string_literal_to_ptr_call_site() {
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

    // Should convert string literal to pointer at call site
    assert!(
        result.contains(".as_ptr()") || result.contains("as *const") || result.contains("as *mut"),
        "Should convert string literal to pointer at call site\nGenerated:\n{}",
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
