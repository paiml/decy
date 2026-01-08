// =============================================================================
// DECY-148: Fix raw pointer return incorrectly calling .as_mut_ptr()
// =============================================================================
// When a function returns a raw pointer parameter, codegen should return
// the pointer directly, NOT call .as_mut_ptr() on it.
//
// C code:
//   TreeNode* insert(TreeNode* root, int value) {
//       return root;
//   }
//
// Expected Rust:
//   fn insert(mut root: *mut TreeNode, mut value: i32) -> *mut TreeNode {
//       return root;  // Direct return, no .as_mut_ptr()
//   }
//
// NOT: return root.as_mut_ptr();  // Error: *mut T has no as_mut_ptr method

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

/// Helper to check if generated Rust compiles
fn compiles(rust_code: &str) -> Result<(), String> {
    let mut temp_file = NamedTempFile::with_suffix(".rs").expect("Failed to create temp file");
    temp_file
        .write_all(rust_code.as_bytes())
        .expect("Failed to write Rust code");

    let output = Command::new("rustc")
        .args([
            "--crate-type=lib",
            "--edition=2021",
            "--crate-name=test_raw_ptr",
            "-A",
            "warnings",
            "-o",
            "/tmp/decy_raw_ptr_test",
        ])
        .arg(temp_file.path())
        .output()
        .expect("Failed to run rustc");

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Test: Raw pointer parameter returned directly (no .as_mut_ptr())
#[test]
fn test_raw_ptr_return_no_as_mut_ptr() {
    let c_code = r#"
struct Node {
    int value;
    struct Node* next;
};

struct Node* identity(struct Node* ptr) {
    return ptr;
}
"#;

    let rust_code = transpile_c(c_code);

    // Should NOT have .as_mut_ptr() on the return
    assert!(
        !rust_code.contains("ptr.as_mut_ptr()"),
        "DECY-148: Should not call .as_mut_ptr() on raw pointer return.\nGot:\n{}",
        rust_code
    );

    // Should have direct return
    assert!(
        rust_code.contains("return ptr") || rust_code.contains("ptr\n}"),
        "DECY-148: Should return raw pointer directly.\nGot:\n{}",
        rust_code
    );
}

/// Test: Generated code should compile
#[test]
fn test_raw_ptr_return_compiles() {
    let c_code = r#"
struct Node {
    int value;
    struct Node* next;
};

struct Node* identity(struct Node* ptr) {
    return ptr;
}

struct Node* modify_and_return(struct Node* node, int new_val) {
    node->value = new_val;
    return node;
}
"#;

    let rust_code = transpile_c(c_code);

    match compiles(&rust_code) {
        Ok(()) => {}
        Err(e) => panic!(
            "DECY-148: Raw pointer return should compile.\nCode:\n{}\nErrors:\n{}",
            rust_code, e
        ),
    }
}

/// Test: binary_tree.c insert function compiles
#[test]
fn test_binary_tree_insert_compiles() {
    let c_code = r#"
struct TreeNode {
    int value;
    struct TreeNode* left;
    struct TreeNode* right;
};

struct TreeNode* insert(struct TreeNode* root, int value) {
    if (root == 0) {
        return 0;
    }
    if (value < root->value) {
        root->left = insert(root->left, value);
    }
    return root;
}
"#;

    let rust_code = transpile_c(c_code);

    match compiles(&rust_code) {
        Ok(()) => {}
        Err(e) => panic!(
            "DECY-148: Binary tree insert should compile.\nCode:\n{}\nErrors:\n{}",
            rust_code, e
        ),
    }
}

/// Test: Full binary_tree.c example compiles
#[test]
fn test_binary_tree_full_compiles() {
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
        .current_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap())
        .output()
        .expect("Failed to run decy transpile");

    let rust_code = String::from_utf8_lossy(&output.stdout).to_string();

    // Should NOT have .as_mut_ptr() on pointer returns
    assert!(
        !rust_code.contains("root.as_mut_ptr()"),
        "DECY-148: Should not call .as_mut_ptr() on root return.\nGot:\n{}",
        rust_code
    );

    match compiles(&rust_code) {
        Ok(()) => {}
        Err(e) => panic!(
            "DECY-148: binary_tree.c should compile.\nCode:\n{}\nErrors:\n{}",
            rust_code, e
        ),
    }
}
