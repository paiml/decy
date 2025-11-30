//! DECY-136: C99 Flexible Array Members → Vec
//!
//! C99 §6.7.2.1 Flexible array members allow structs to have a trailing
//! array of unspecified size:
//!
//! struct Buffer {
//!     int size;
//!     char data[];  // Flexible array member
//! };
//!
//! Expected Rust transformation:
//! - `char data[]` → `data: Vec<u8>`
//! - `int values[]` → `values: Vec<i32>`

use decy_core::transpile;

/// Test flexible array member transforms to Vec.
///
/// C: struct Buffer { int size; char data[]; };
/// Expected Rust: struct Buffer { size: i32, data: Vec<u8> }
#[test]
fn test_flexible_array_member_to_vec() {
    let c_code = r#"
        struct Buffer {
            int size;
            char data[];
        };

        int main() {
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should contain Buffer struct
    assert!(
        result.contains("pub struct Buffer"),
        "Should have Buffer struct\nGenerated:\n{}",
        result
    );

    // Should have data field as Vec<u8>
    assert!(
        result.contains("data: Vec<u8>") || result.contains("data: Vec<i8>"),
        "Flexible array member should become Vec\nGenerated:\n{}",
        result
    );
}

/// Test flexible int array member transforms to Vec<i32>.
///
/// C: struct IntArray { int count; int values[]; };
/// Expected Rust: struct IntArray { count: i32, values: Vec<i32> }
#[test]
fn test_flexible_int_array_to_vec() {
    let c_code = r#"
        struct IntArray {
            int count;
            int values[];
        };

        int main() {
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should have values field as Vec<i32>
    assert!(
        result.contains("values: Vec<i32>"),
        "Flexible int array should become Vec<i32>\nGenerated:\n{}",
        result
    );
}
