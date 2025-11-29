//! DECY-114: Default derive for structs tests.
//!
//! C structs that are zero-initialized should derive Default in Rust.
//!
//! C: struct Node n = {0};  or uninitialized struct
//! Expected Rust: #[derive(Default)] pub struct Node { ... }
//!                let n = Node::default();

use decy_core::transpile;

/// Test that structs get #[derive(Default)] attribute.
///
/// C: struct Point { int x; int y; };
/// Expected: #[derive(Default)] pub struct Point { ... }
#[test]
fn test_struct_derives_default() {
    let c_code = r#"
        struct Point {
            int x;
            int y;
        };

        int main() {
            struct Point p;
            p.x = 10;
            return p.x;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should have Default in the derive list
    assert!(
        result.contains("Default"),
        "Struct should derive Default\nGenerated:\n{}",
        result
    );
}

/// Test that struct initialization uses Default::default().
///
/// C: struct Node n;  (uninitialized)
/// Expected Rust: let n = Node::default();  or  let n: Node = Default::default();
#[test]
fn test_struct_init_uses_default() {
    let c_code = r#"
        struct Node {
            int value;
            struct Node* next;
        };

        int main() {
            struct Node n;
            n.value = 42;
            return n.value;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should use ::default() for struct initialization
    assert!(
        result.contains("::default()") || result.contains("Default::default()"),
        "Struct init should use default()\nGenerated:\n{}",
        result
    );
}

/// Test that the generated code compiles.
#[test]
fn test_struct_default_compiles() {
    let c_code = r#"
        struct Point {
            int x;
            int y;
        };

        int main() {
            struct Point p;
            p.x = 10;
            p.y = 20;
            return p.x + p.y;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Try to compile the generated code
    use std::process::Command;

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let temp_file_path = temp_dir.path().join("test_code.rs");
    std::fs::write(&temp_file_path, format!("#![allow(unused)]\n{}", result))
        .expect("Failed to write temp file");

    let output = Command::new("rustc")
        .arg("--emit=metadata")
        .arg("--crate-type=lib")
        .arg("--crate-name=decy_test")
        .arg("--out-dir")
        .arg(temp_dir.path())
        .arg(&temp_file_path)
        .output()
        .expect("Failed to run rustc");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Generated code should compile:\n{}\n\nStderr:\n{}",
        result,
        stderr
    );
}

/// Test that linked list struct compiles with Default.
#[test]
fn test_linked_list_struct_default() {
    let c_code = r#"
        struct Node {
            int value;
            struct Node* next;
        };

        int sum_list(struct Node* head) {
            int sum = 0;
            struct Node* current = head;
            while (current != 0) {
                sum = sum + current->value;
                current = current->next;
            }
            return sum;
        }

        int main() {
            struct Node n3;
            n3.value = 3;
            n3.next = 0;

            struct Node n2;
            n2.value = 2;
            n2.next = &n3;

            struct Node n1;
            n1.value = 1;
            n1.next = &n2;

            return sum_list(&n1);
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should derive Default
    assert!(
        result.contains("Default"),
        "Node struct should derive Default\nGenerated:\n{}",
        result
    );

    // Should use ::default() for initialization
    assert!(
        result.contains("::default()"),
        "Should use default() for struct init\nGenerated:\n{}",
        result
    );
}
