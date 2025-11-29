//! DECY-119: Struct pointer field assignment tests.
//!
//! When assigning &x to a struct field of type *mut T, cast properly.
//!
//! C: node.next = &other;
//! Expected Rust: node.next = &mut other as *mut T;

use decy_core::transpile;

/// Test that struct field assignment with address-of casts to raw pointer.
///
/// C: n2.next = &n3;  where next is *mut Node
/// Expected: n2.next = &mut n3 as *mut Node;
#[test]
fn test_struct_field_address_of_casts_to_ptr() {
    let c_code = r#"
        struct Node {
            int value;
            struct Node *next;
        };

        int main() {
            struct Node n1;
            struct Node n2;
            n1.value = 10;
            n2.value = 20;
            n1.next = &n2;
            return n1.value;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should cast &n2 to raw pointer
    assert!(
        result.contains("as *mut Node") || result.contains("as *mut"),
        "Should cast &x to *mut T\nGenerated:\n{}",
        result
    );

    // Should NOT have bare &n2 assigned to pointer field
    assert!(
        !result.contains(".next = &n2;"),
        "Should NOT assign bare &x to pointer field\nGenerated:\n{}",
        result
    );
}

/// Test that null pointer assignment still works.
#[test]
fn test_struct_field_null_still_works() {
    let c_code = r#"
        struct Node {
            int value;
            struct Node *next;
        };

        int main() {
            struct Node n;
            n.value = 10;
            n.next = 0;
            return n.value;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should use null_mut() for 0 assignment to pointer
    assert!(
        result.contains("std::ptr::null_mut()"),
        "Should use null_mut() for 0 assignment\nGenerated:\n{}",
        result
    );
}
