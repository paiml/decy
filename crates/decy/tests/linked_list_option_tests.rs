//! DECY-135: Linked List Patterns
//!
//! Tests for linked list struct transformation.
//! Note: Full Option<Box<T>> transformation requires detecting heap allocation
//! patterns. Stack-allocated linked lists (like the corpus example) cannot
//! safely use Option<Box<T>>.

use decy_core::transpile;

/// Test that linked list struct compiles (uses raw pointers for now).
///
/// C: struct Node { int value; struct Node *next; };
/// Current: pub next: *mut Node (raw pointer preserved for safety)
///
/// Note: Option<Box<T>> transformation deferred - requires detecting
/// malloc patterns vs stack allocation.
#[test]
fn test_linked_list_struct_compiles() {
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

    // Should have Node struct with next field
    assert!(
        result.contains("pub struct Node"),
        "Should generate Node struct\nGenerated:\n{}",
        result
    );

    assert!(
        result.contains("pub next:"),
        "Should have next field\nGenerated:\n{}",
        result
    );
}

/// Test that NULL assignment to pointer field generates null_mut.
///
/// C: n.next = 0;
/// Rust: n.next = std::ptr::null_mut();
#[test]
fn test_null_assignment_to_pointer_field() {
    let c_code = r#"
        struct Node {
            int value;
            struct Node *next;
        };

        int main() {
            struct Node n;
            n.next = 0;
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // NULL assignment generates null_mut (raw pointer)
    assert!(
        result.contains("null_mut()"),
        "NULL assignment should generate null_mut()\nGenerated:\n{}",
        result
    );
}

/// Test that linked list traversal compiles (may use unsafe).
///
/// Note: Eliminating unsafe from linked list traversal requires
/// detecting that nodes are heap-allocated with malloc, then
/// transforming to Option<Box<T>> pattern.
#[test]
fn test_linked_list_traversal_compiles() {
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

        int main() {
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should have sum_list function
    assert!(
        result.contains("fn sum_list"),
        "Should generate sum_list function\nGenerated:\n{}",
        result
    );
}
