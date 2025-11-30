//! DECY-133: C99 Designated Initializers
//!
//! C99 §6.7.8 Initialization - Designated initializers allow specifying
//! which struct members or array indices to initialize:
//!
//! struct: {.x = 10, .y = 20}
//! array: {[2] = 100, [4] = 200}
//!
//! Expected Rust transformations:
//! - Struct designated init → Struct { x: 10, y: 20, ..Default::default() }
//! - Array designated init → array with specified indices set

use decy_core::transpile;

/// Test struct designated initializer.
///
/// C: struct Point p = {.x = 10, .y = 20};
/// Expected Rust: let p = Point { x: 10, y: 20, z: 0 };
#[test]
fn test_struct_designated_initializer() {
    let c_code = r#"
        struct Point {
            int x;
            int y;
            int z;
        };

        int main() {
            struct Point p = {.x = 10, .y = 20};
            return p.x + p.y;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should contain struct initialization with named fields
    assert!(
        result.contains("Point {") || result.contains("Point{"),
        "Should generate struct initialization\nGenerated:\n{}",
        result
    );

    // Should contain x: 10
    assert!(
        result.contains("x: 10") || result.contains("x:10"),
        "Should set x field to 10\nGenerated:\n{}",
        result
    );

    // Should contain y: 20
    assert!(
        result.contains("y: 20") || result.contains("y:20"),
        "Should set y field to 20\nGenerated:\n{}",
        result
    );
}

/// Test array designated initializer.
///
/// C: int arr[5] = {[2] = 100, [4] = 200};
/// Expected Rust: let arr = [0, 0, 100, 0, 200];
#[test]
fn test_array_designated_initializer() {
    let c_code = r#"
        int main() {
            int arr[5] = {[2] = 100, [4] = 200};
            return arr[2] + arr[4];
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should contain array with correct values
    // Either explicit array or vec! with values
    assert!(
        result.contains("100") && result.contains("200"),
        "Should contain initialized values 100 and 200\nGenerated:\n{}",
        result
    );
}

/// Test mixed struct and array designated init.
///
/// C: struct Data { int values[3]; };
///    struct Data d = {.values = {[1] = 42}};
#[test]
fn test_nested_designated_initializer() {
    let c_code = r#"
        struct Data {
            int values[3];
        };

        int main() {
            struct Data d = {.values = {[1] = 42}};
            return d.values[1];
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should contain the value 42
    assert!(
        result.contains("42"),
        "Should contain initialized value 42\nGenerated:\n{}",
        result
    );
}
