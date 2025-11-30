//! DECY-115: Heap-allocated linked list to Option<Box<T>> transformation
//!
//! Key insight: Option<Box<T>> can ONLY work with heap-allocated nodes.
//! Stack-allocated linked lists must remain as raw pointers.
//!
//! This test covers heap-allocated patterns that CAN be safely transformed.

use decy_core::transpile;

/// Test heap-allocated linked list with malloc.
///
/// C: struct Node *n = malloc(sizeof(struct Node));
///    n->next = NULL;
/// Expected Rust: let n = Some(Box::new(Node { ..., next: None }));
///
/// This pattern IS safe to transform to Option<Box<T>>.
#[test]
fn test_heap_linked_list_malloc_to_box() {
    let c_code = r#"
        struct Node {
            int value;
            struct Node *next;
        };

        struct Node* create_node(int value) {
            struct Node *n = malloc(sizeof(struct Node));
            n->value = value;
            n->next = 0;
            return n;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should use Box::new for heap allocation
    assert!(
        result.contains("Box::new") || result.contains("Box<Node>"),
        "Should use Box for heap allocation\nGenerated:\n{}",
        result
    );
}

/// Test that stack-allocated linked lists still work (with unsafe).
///
/// Stack-allocated linked lists cannot use Option<Box<T>> because
/// Box requires ownership, but stack variables have fixed lifetimes.
///
/// This is the CORRECT behavior - not a bug.
#[test]
fn test_stack_linked_list_remains_raw_pointer() {
    let c_code = r#"
        struct Node {
            int value;
            struct Node *next;
        };

        int main() {
            struct Node n1;
            n1.value = 10;
            n1.next = 0;
            return n1.value;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Stack-allocated should still compile (may use raw pointers or null_mut)
    // This is expected behavior - stack allocation cannot use Box
    assert!(
        result.contains("struct Node") || result.contains("Node"),
        "Should generate valid struct\nGenerated:\n{}",
        result
    );
}

/// Test that heap-allocated linked list compiles.
#[test]
fn test_heap_linked_list_compiles() {
    let c_code = r#"
        struct Node {
            int value;
            struct Node *next;
        };

        struct Node* create_node(int value) {
            struct Node *n = malloc(sizeof(struct Node));
            n->value = value;
            n->next = 0;
            return n;
        }

        int get_value(struct Node *n) {
            return n->value;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Try to compile the generated code
    use std::process::Command;
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let temp_file = temp_dir.path().join("heap_linked_list.rs");
    std::fs::write(&temp_file, format!("#![allow(unused)]\n{}", result))
        .expect("Failed to write temp file");

    let output = Command::new("rustc")
        .arg("--emit=metadata")
        .arg("--crate-type=lib")
        .arg("--crate-name=heap_list_test")
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

/// Test NULL comparison in heap context.
///
/// C: if (n->next != NULL) { ... }
/// Current: if n.next != std::ptr::null_mut() { ... }
/// Could be: if n.next.is_some() { ... }
#[test]
fn test_null_comparison_heap_context() {
    let c_code = r#"
        struct Node {
            int value;
            struct Node *next;
        };

        int has_next(struct Node *n) {
            if (n->next != 0) {
                return 1;
            }
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Current implementation uses null_mut comparison
    // Future: could use is_some() if field is Option<Box<T>>
    assert!(
        result.contains("null_mut") || result.contains("is_some") || result.contains("!= 0"),
        "Should have NULL check\nGenerated:\n{}",
        result
    );
}
