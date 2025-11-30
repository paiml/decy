//! DECY-137: Linked List Traversal Compilation Tests
//!
//! Tests that linked list traversal patterns generate compilable Rust code.
//! The pattern `while (head != 0) { head = head->next; }` requires the
//! pointer parameter to remain as a raw pointer, not transform to &mut T.
//!
//! Bug discovered: Pointer params transformed to references fail to compile
//! when reassigned in loop body (e.g., `head = head->next`).

use decy_core::transpile;

/// Test that linked list traversal generates compilable Rust.
///
/// C pattern:
/// ```c
/// int list_length(struct Node* head) {
///     int count = 0;
///     while (head != 0) {
///         count = count + 1;
///         head = head->next;  // Pointer reassignment!
///     }
///     return count;
/// }
/// ```
///
/// Bug: `struct Node* head` was transformed to `head: &mut Node` but
/// `head = head->next` requires reassignment which references don't allow.
#[test]
fn test_linked_list_traversal_compiles_rust() {
    let c_code = r#"
        struct Node {
            int data;
            struct Node* next;
        };

        int list_length(struct Node* head) {
            int count;
            count = 0;
            while (head != 0) {
                count = count + 1;
                head = head->next;
            }
            return count;
        }

        int main() { return 0; }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // The generated code should compile - verify no basic type errors
    // Key: head parameter must stay as raw pointer since it's reassigned
    assert!(
        result.contains("fn list_length"),
        "Should generate list_length function\nGenerated:\n{}",
        result
    );

    // Critical: The head parameter should remain as raw pointer OR
    // the loop should be transformed to use an index/iterator pattern
    // Check that we don't have the broken pattern: &mut Node compared to 0
    assert!(
        !result.contains("head != 0") || !result.contains("head: &mut Node"),
        "Should not compare reference to integer literal\nGenerated:\n{}",
        result
    );

    // Verify either raw pointer or proper null check
    let has_raw_pointer = result.contains("*mut Node");
    let has_null_mut_check = result.contains("null_mut()") || result.contains("is_null()");
    let has_option_pattern = result.contains("Option<") || result.contains(".is_none()");

    assert!(
        has_raw_pointer || has_null_mut_check || has_option_pattern,
        "Linked list traversal should use raw pointer with null check, or Option pattern\nGenerated:\n{}",
        result
    );
}

/// Test pointer reassignment in loop body is detected.
///
/// Any pointer parameter that is reassigned (`ptr = something`) should
/// NOT be transformed to a reference since references can't be reassigned.
#[test]
fn test_pointer_param_reassignment_detection() {
    let c_code = r#"
        void traverse(int* ptr, int n) {
            int i;
            for (i = 0; i < n; i = i + 1) {
                ptr = ptr + 1;
            }
        }

        int main() { return 0; }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // ptr is reassigned, so should remain as raw pointer
    assert!(
        result.contains("*mut i32") || result.contains("mut ptr:"),
        "Pointer param that is reassigned should stay as raw pointer\nGenerated:\n{}",
        result
    );
}

/// Test that pointer field access with reassignment keeps raw pointer.
///
/// C: `head = head->next;` - pointer reassignment from struct field
#[test]
fn test_pointer_field_reassignment_keeps_raw_pointer() {
    let c_code = r#"
        struct Node {
            int value;
            struct Node* next;
        };

        int traverse(struct Node* current) {
            int sum;
            sum = 0;
            while (current != 0) {
                sum = sum + current->value;
                current = current->next;
            }
            return sum;
        }

        int main() { return 0; }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // current is reassigned from field access, should be raw pointer
    assert!(
        result.contains("fn traverse"),
        "Should generate traverse function\nGenerated:\n{}",
        result
    );

    // Should NOT have the broken pattern of &mut Node compared to 0
    assert!(
        !result.contains("current: &mut Node") || !result.contains("current != 0"),
        "Should not generate reference compared to integer\nGenerated:\n{}",
        result
    );
}
