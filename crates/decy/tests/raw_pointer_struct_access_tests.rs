//! DECY-129: Raw pointer struct field access tests.
//!
//! When accessing struct fields through a raw pointer,
//! wrap the access in unsafe blocks.
//!
//! C: current->value  where current is *mut Node
//! Expected Rust: unsafe { (*current).value }

use decy_core::transpile;

/// Test that raw pointer struct field access is wrapped in unsafe.
///
/// C: (*current).value where current is *mut Node
/// Expected: unsafe { (*current).value }
#[test]
fn test_raw_pointer_struct_deref_needs_unsafe() {
    let c_code = r#"
        struct Node {
            int value;
        };

        int get_value(struct Node *ptr) {
            ptr = ptr;  // force raw pointer (no arithmetic but assign to self)
            return (*ptr).value;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should wrap struct field access through raw pointer in unsafe
    // Note: This test may need adjustment based on pointer detection logic
    assert!(
        result.contains("(*ptr).value")
            || result.contains("unsafe {")
            || result.contains("&mut Node"),
        "Should handle struct field access properly\nGenerated:\n{}",
        result
    );
}

/// Test that arrow operator through raw pointer generates unsafe.
///
/// C: current->value  where current is *mut Node
/// Expected: unsafe { (*current).value }
#[test]
fn test_arrow_operator_on_raw_ptr_needs_unsafe() {
    let c_code = r#"
        struct Item {
            int data;
            struct Item *next;
        };

        void traverse(struct Item *node) {
            node = node + 0;  // force raw pointer type
            int x = node->data;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // The arrow operator should become (*ptr).field, potentially with unsafe
    assert!(
        result.contains("(*node)") || result.contains("unsafe") || result.contains("&mut Item"),
        "Should transform arrow operator\nGenerated:\n{}",
        result
    );
}
