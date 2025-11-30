//! DECY-115: Linked list pointer to Option<Box<T>> transformation tests
//!
//! Target: Transform raw pointers in recursive structs to safe Option<Box<T>>.
//!
//! C: struct Node { int value; struct Node *next; };
//! Expected Rust: pub next: Option<Box<Node>>
//!
//! This eliminates unsafe blocks in linked list traversal.

use decy_core::transpile;

/// Test that self-referential struct uses Option<Box<T>> instead of raw pointer.
///
/// C: struct Node { int value; struct Node *next; };
/// Expected: pub next: Option<Box<Node>>
/// Current: pub next: *mut Node  // FAILS - should use Option<Box>
#[test]
#[ignore = "RED phase: DECY-115 - Option<Box<T>> transformation not implemented"]
fn test_recursive_struct_to_option_box() {
    let c_code = r#"
        struct Node {
            int value;
            struct Node *next;
        };
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should use Option<Box<Node>> for recursive pointer
    assert!(
        result.contains("Option<Box<Node>>"),
        "Should use Option<Box<Node>> for recursive pointer\nGenerated:\n{}",
        result
    );

    // Should NOT use raw pointer
    assert!(
        !result.contains("*mut Node") && !result.contains("*const Node"),
        "Should NOT use raw pointer\nGenerated:\n{}",
        result
    );
}

/// Test that NULL assignment becomes None.
///
/// C: node->next = 0;
/// Expected: node.next = None;
/// Current: node.next = std::ptr::null_mut();  // FAILS
#[test]
#[ignore = "RED phase: DECY-115 - None transformation not implemented"]
fn test_null_assignment_becomes_none() {
    let c_code = r#"
        struct Node {
            int value;
            struct Node *next;
        };

        void set_end(struct Node *node) {
            node->next = 0;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should use None instead of null_mut
    assert!(
        result.contains("= None") || result.contains("None;"),
        "Should use None for NULL assignment\nGenerated:\n{}",
        result
    );

    // Should NOT use null_mut
    assert!(
        !result.contains("null_mut()"),
        "Should NOT use null_mut()\nGenerated:\n{}",
        result
    );
}

/// Test that NULL check becomes Option pattern matching.
///
/// C: while (current != 0) { ... current = current->next; }
/// Expected: while let Some(ref node) = current { ... }
/// Current: while current != std::ptr::null_mut() { ... }  // FAILS
#[test]
#[ignore = "RED phase: DECY-115 - while let transformation not implemented"]
fn test_null_check_to_while_let() {
    let c_code = r#"
        struct Node {
            int value;
            struct Node *next;
        };

        int sum_list(struct Node *head) {
            int sum = 0;
            struct Node *current = head;
            while (current != 0) {
                sum = sum + current->value;
                current = current->next;
            }
            return sum;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should use while let Some pattern
    assert!(
        result.contains("while let Some") || result.contains("if let Some"),
        "Should use Option pattern matching\nGenerated:\n{}",
        result
    );

    // Should NOT use null_mut comparison
    assert!(
        !result.contains("null_mut()"),
        "Should NOT compare to null_mut()\nGenerated:\n{}",
        result
    );
}

/// Test that linked list traversal has NO unsafe blocks.
///
/// This is the ultimate goal: safe linked list traversal.
#[test]
#[ignore = "RED phase: DECY-115 - Safe linked list traversal not implemented"]
fn test_linked_list_no_unsafe() {
    let c_code = r#"
        struct Node {
            int value;
            struct Node *next;
        };

        int sum_list(struct Node *head) {
            int sum = 0;
            struct Node *current = head;
            while (current != 0) {
                sum = sum + current->value;
                current = current->next;
            }
            return sum;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should have ZERO unsafe blocks
    let unsafe_count = result.matches("unsafe").count();
    assert_eq!(
        unsafe_count, 0,
        "Should have NO unsafe blocks in linked list traversal\nFound {} unsafe blocks\nGenerated:\n{}",
        unsafe_count, result
    );
}

/// Test that the transformed code compiles.
#[test]
#[ignore = "RED phase: DECY-115 - Full transformation not implemented"]
fn test_linked_list_option_compiles() {
    let c_code = r#"
        struct Node {
            int value;
            struct Node *next;
        };

        int sum_list(struct Node *head) {
            int sum = 0;
            struct Node *current = head;
            while (current != 0) {
                sum = sum + current->value;
                current = current->next;
            }
            return sum;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    // Verify it compiles
    use std::process::Command;
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let temp_file = temp_dir.path().join("linked_list_test.rs");
    std::fs::write(&temp_file, format!("#![allow(unused)]\n{}", result))
        .expect("Failed to write temp file");

    let output = Command::new("rustc")
        .arg("--emit=metadata")
        .arg("--crate-type=lib")
        .arg("--crate-name=linked_list_test")
        .arg("--out-dir")
        .arg(temp_dir.path())
        .arg(&temp_file)
        .output()
        .expect("Failed to run rustc");

    assert!(
        output.status.success(),
        "Generated code should compile:\n{}\n\nStderr:\n{}",
        result,
        String::from_utf8_lossy(&output.stderr)
    );
}
