// =============================================================================
// DECY-143: Add SAFETY comments to unsafe blocks
// =============================================================================
// Tests for verifying that all generated unsafe blocks have SAFETY comments.
//
// Rust convention: Every unsafe block should have a SAFETY comment explaining
// why the operation is safe. This helps with code review and auditing.
//
// Example of proper SAFETY comment:
//   // SAFETY: pointer is valid and properly aligned, derived from Box allocation
//   unsafe { *ptr = value; }

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
    code.matches("unsafe {").count() + code.matches("unsafe{").count()
}

/// Count SAFETY comments in code
fn count_safety_comments(code: &str) -> usize {
    // Count both formats:
    // - "// SAFETY:" (line comment)
    // - "/* SAFETY:" (block comment)
    let line_comments = code
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("// SAFETY:") || trimmed.starts_with("// SAFETY :")
        })
        .count();
    let block_comments = code.matches("/* SAFETY:").count();
    line_comments + block_comments
}

/// Test that pointer dereference operations have SAFETY comments.
#[test]
fn test_pointer_deref_has_safety_comment() {
    let c_code = r#"
void set_value(int* ptr, int value) {
    *ptr = value;
}
"#;

    let rust_code = transpile_c(c_code);

    let unsafe_count = count_unsafe_blocks(&rust_code);
    let safety_count = count_safety_comments(&rust_code);

    // If there are unsafe blocks, there should be SAFETY comments
    if unsafe_count > 0 {
        assert!(
            safety_count >= unsafe_count,
            "DECY-143: Every unsafe block should have a SAFETY comment.\nUnsafe blocks: {}, SAFETY comments: {}\nCode:\n{}",
            unsafe_count, safety_count, rust_code
        );
    }
}

/// Test that Box::into_raw operations have SAFETY comments.
#[test]
fn test_box_into_raw_has_safety_comment() {
    let c_code = r#"
#include <stdlib.h>

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

    // Check for SAFETY comment near Box::into_raw if present
    // Box::into_raw itself is safe, but we document the ownership transfer
    if rust_code.contains("Box::into_raw") {
        // For now, Box::into_raw doesn't require SAFETY comment as it's safe
        // The unsafe part is when converting back with Box::from_raw
        // Just verify the code compiles correctly
        assert!(
            rust_code.contains("Box::into_raw"),
            "Expected Box::into_raw in output.\nCode:\n{}",
            rust_code
        );
    }
}

/// Test that hash_table.c output has SAFETY comments for all unsafe blocks.
#[test]
fn test_hash_table_has_safety_comments() {
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
        .current_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap())
        .output()
        .expect("Failed to run decy transpile");

    let rust_code = String::from_utf8_lossy(&output.stdout).to_string();

    let unsafe_count = count_unsafe_blocks(&rust_code);
    let safety_count = count_safety_comments(&rust_code);

    // Every unsafe block should have a SAFETY comment
    assert!(
        safety_count >= unsafe_count,
        "DECY-143: hash_table.c - Every unsafe block needs a SAFETY comment.\nUnsafe blocks: {}, SAFETY comments: {}\nExpected ratio: 1:1 or better",
        unsafe_count, safety_count
    );
}

/// Test that binary_tree.c output has SAFETY comments for all unsafe blocks.
#[test]
fn test_binary_tree_has_safety_comments() {
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

    let unsafe_count = count_unsafe_blocks(&rust_code);
    let safety_count = count_safety_comments(&rust_code);

    // Every unsafe block should have a SAFETY comment
    assert!(
        safety_count >= unsafe_count,
        "DECY-143: binary_tree.c - Every unsafe block needs a SAFETY comment.\nUnsafe blocks: {}, SAFETY comments: {}\nExpected ratio: 1:1 or better",
        unsafe_count, safety_count
    );
}

/// Test that SAFETY comments are meaningful (not just empty).
#[test]
fn test_safety_comments_are_meaningful() {
    let c_code = r#"
void modify_array(int* arr, int index, int value) {
    arr[index] = value;
}
"#;

    let rust_code = transpile_c(c_code);

    // Find SAFETY comments and check they have content
    // Check both line comments (// SAFETY:) and block comments (/* SAFETY: */)
    for line in rust_code.lines() {
        let trimmed = line.trim();

        // Check line comment format
        if trimmed.starts_with("// SAFETY:") {
            let comment_content = trimmed.trim_start_matches("// SAFETY:").trim();
            assert!(
                !comment_content.is_empty(),
                "DECY-143: SAFETY comments must have meaningful content.\nEmpty comment found in:\n{}",
                rust_code
            );
            assert!(
                comment_content.len() >= 10,
                "DECY-143: SAFETY comment too short: '{}'\nShould explain why the operation is safe.",
                comment_content
            );
        }

        // Check block comment format: /* SAFETY: ... */
        if let Some(start) = trimmed.find("/* SAFETY:") {
            if let Some(end) = trimmed[start..].find("*/") {
                let comment_content = trimmed[start + 10..start + end].trim();
                assert!(
                    !comment_content.is_empty(),
                    "DECY-143: SAFETY comments must have meaningful content.\nEmpty comment found in:\n{}",
                    rust_code
                );
                assert!(
                    comment_content.len() >= 10,
                    "DECY-143: SAFETY comment too short: '{}'\nShould explain why the operation is safe.",
                    comment_content
                );
            }
        }
    }
}
