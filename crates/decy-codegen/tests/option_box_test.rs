// =============================================================================
// DECY-144: Transform self-referential struct pointers to Option<Box<T>>
// =============================================================================
// Tests for generating Option<Box<T>> instead of *mut T for self-referential
// struct fields (trees, linked lists, etc).
//
// Current (unsafe-heavy):
//   struct TreeNode {
//       value: i32,
//       left: *mut TreeNode,
//       right: *mut TreeNode,
//   }
//
// Expected (safe):
//   struct TreeNode {
//       value: i32,
//       left: Option<Box<TreeNode>>,
//       right: Option<Box<TreeNode>>,
//   }

use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

/// Helper to transpile C code and return the generated Rust
fn transpile_c(c_code: &str) -> String {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(c_code.as_bytes())
        .expect("Failed to write C code");

    let output = Command::new("cargo")
        .args(["run", "-p", "decy", "--quiet", "--", "transpile"])
        .arg(temp_file.path())
        .output()
        .expect("Failed to run decy transpile");

    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Count unsafe blocks in code
fn count_unsafe_blocks(code: &str) -> usize {
    code.matches("unsafe {").count()
}

/// Test that tree node struct uses Option<Box<T>> for children.
#[test]
fn test_tree_node_uses_option_box() {
    let c_code = r#"
typedef struct TreeNode {
    int value;
    struct TreeNode* left;
    struct TreeNode* right;
} TreeNode;
"#;

    let rust_code = transpile_c(c_code);

    // Should use Option<Box<TreeNode>> for self-referential fields
    assert!(
        rust_code.contains("Option<Box<TreeNode>>"),
        "DECY-144: Self-referential struct fields should use Option<Box<T>>.\nGot:\n{}",
        rust_code
    );

    // Should NOT use raw pointer for these fields
    assert!(
        !rust_code.contains("left: *mut TreeNode"),
        "DECY-144: Should NOT use *mut T for self-referential fields.\nGot:\n{}",
        rust_code
    );
}

/// Test that linked list node uses Option<Box<T>> for next pointer.
#[test]
fn test_linked_list_uses_option_box() {
    let c_code = r#"
typedef struct Node {
    int data;
    struct Node* next;
} Node;
"#;

    let rust_code = transpile_c(c_code);

    // Should use Option<Box<Node>> for self-referential next field
    assert!(
        rust_code.contains("Option<Box<Node>>"),
        "DECY-144: Linked list next pointer should use Option<Box<T>>.\nGot:\n{}",
        rust_code
    );
}

/// Test that NULL assignment to self-referential field becomes None.
#[test]
fn test_null_becomes_none() {
    let c_code = r#"
typedef struct TreeNode {
    int value;
    struct TreeNode* left;
    struct TreeNode* right;
} TreeNode;

TreeNode* create_leaf(int value) {
    TreeNode* node = (TreeNode*)malloc(sizeof(TreeNode));
    node->value = value;
    node->left = NULL;
    node->right = NULL;
    return node;
}
"#;

    let rust_code = transpile_c(c_code);

    // NULL assignment should become None
    assert!(
        rust_code.contains("= None") || rust_code.contains("= None;"),
        "DECY-144: NULL to self-referential field should become None.\nGot:\n{}",
        rust_code
    );
}

/// Test that binary_tree.c has reduced unsafe count with Option<Box<T>>.
#[test]
fn test_binary_tree_reduced_unsafe() {
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "decy",
            "--quiet",
            "--",
            "transpile",
            "examples/data_structures/binary_tree.c",
        ])
        .current_dir("/home/noah/src/decy")
        .output()
        .expect("Failed to run decy transpile");

    let rust_code = String::from_utf8_lossy(&output.stdout).to_string();
    let unsafe_count = count_unsafe_blocks(&rust_code);
    let loc = rust_code.lines().count();

    // Calculate unsafe per 1000 LOC
    let unsafe_per_1000 = (unsafe_count as f64 * 1000.0) / loc as f64;

    // Target: <20 per 1000 LOC (current is ~90, so this is a 75% reduction goal)
    // More aggressive target would be <5, but let's be realistic
    assert!(
        unsafe_per_1000 < 20.0,
        "DECY-144: binary_tree.c should have <20 unsafe per 1000 LOC.\nCurrent: {:.1} ({} unsafe / {} LOC)",
        unsafe_per_1000, unsafe_count, loc
    );
}

/// Test that hash_table Entry.next uses Option<Box<Entry>>.
#[test]
fn test_hash_table_entry_uses_option_box() {
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "decy",
            "--quiet",
            "--",
            "transpile",
            "examples/data_structures/hash_table.c",
        ])
        .current_dir("/home/noah/src/decy")
        .output()
        .expect("Failed to run decy transpile");

    let rust_code = String::from_utf8_lossy(&output.stdout).to_string();

    // Entry.next should use Option<Box<Entry>>
    assert!(
        rust_code.contains("Option<Box<Entry>>"),
        "DECY-144: hash_table Entry.next should use Option<Box<Entry>>.\nGot:\n{}",
        rust_code
    );
}

/// Test that generated code compiles after Option<Box<T>> transformation.
#[test]
fn test_option_box_compiles() {
    let c_code = r#"
typedef struct Node {
    int data;
    struct Node* next;
} Node;

Node* create_node(int data) {
    Node* n = (Node*)malloc(sizeof(Node));
    n->data = data;
    n->next = NULL;
    return n;
}
"#;

    let rust_code = transpile_c(c_code);

    // Write to temp file and compile
    let mut temp_file = NamedTempFile::with_suffix(".rs").expect("Failed to create temp file");
    temp_file
        .write_all(rust_code.as_bytes())
        .expect("Failed to write Rust code");

    let output = Command::new("rustc")
        .args([
            "--crate-type=lib",
            "--edition=2021",
            "--crate-name=test_option_box",
            "-A",
            "warnings",
        ])
        .arg(temp_file.path())
        .arg("-o")
        .arg("/tmp/test_option_box_lib")
        .output()
        .expect("Failed to run rustc");

    let success = output.status.success();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    assert!(
        success,
        "Generated Option<Box<T>> code should compile.\nCode:\n{}\\nErrors:\n{}",
        rust_code, stderr
    );
}
