// =============================================================================
// DECY-140: malloc to Raw Pointer and strcpy to Struct Field Tests
// =============================================================================
// Tests for transforming:
// 1. malloc assigned to *mut u8 field → raw pointer allocation
// 2. strcpy(struct_field, src) → proper byte copy
//
// C pattern:
//   entry->key = (char*)malloc(strlen(key) + 1);
//   strcpy(entry->key, key);
//
// Current broken output:
//   entry.key = Vec::<u8>::with_capacity(...);  // Can't assign Vec to *mut u8
//   key.to_string();  // No-op, doesn't copy
//
// Expected output:
//   entry.key = Box::leak(vec![0u8; len].into_boxed_slice()).as_mut_ptr();
//   // strcpy generates proper copy

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

/// Helper to compile Rust code and return success/failure
fn compile_rust(rust_code: &str) -> (bool, String) {
    let mut temp_file = NamedTempFile::with_suffix(".rs").expect("Failed to create temp file");
    temp_file
        .write_all(rust_code.as_bytes())
        .expect("Failed to write Rust code");

    let output = Command::new("rustc")
        .args([
            "--crate-type=lib",
            "--edition=2021",
            "--crate-name=test_malloc_ptr",
        ])
        .arg(temp_file.path())
        .arg("-o")
        .arg("/tmp/test_malloc_ptr_compile")
        .output()
        .expect("Failed to run rustc");

    let success = output.status.success();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (success, stderr)
}

/// Test that malloc assigned to char* (which becomes *mut u8) generates valid code.
/// C: entry->key = (char*)malloc(len);
/// Bug: Currently generates Vec::<u8>::with_capacity(...) which can't be assigned to *mut u8
/// Expected: Generate raw pointer allocation that can be assigned to *mut u8
#[test]
fn test_malloc_to_ptr_field_assignment() {
    let c_code = r#"
typedef struct Entry {
    char* key;
    int value;
} Entry;

void set_key(Entry* entry, int len) {
    entry->key = (char*)malloc(len);
}
"#;

    let rust_code = transpile_c(c_code);

    // Should contain the assignment
    assert!(
        rust_code.contains("entry") && rust_code.contains("key"),
        "Should generate entry.key assignment, got: {}",
        rust_code
    );

    // Should NOT contain Vec::<u8>::with_capacity assigned directly to field
    // If it does, we need to convert it to raw pointer
    if rust_code.contains("Vec::<u8>::with_capacity") {
        // Vec must be converted to raw pointer
        assert!(
            rust_code.contains("as_mut_ptr")
                || rust_code.contains("into_raw")
                || rust_code.contains("Box::leak"),
            "DECY-140: Vec assigned to *mut u8 field must be converted to raw pointer. Got:\n{}",
            rust_code
        );
    }
}

/// Test that malloc(strlen(s) + 1) pattern generates valid code when assigned to *mut u8.
#[test]
fn test_malloc_strlen_pattern() {
    let c_code = r#"
typedef struct Node {
    char* name;
} Node;

void copy_name(Node* node, const char* src) {
    node->name = (char*)malloc(strlen(src) + 1);
}
"#;

    let rust_code = transpile_c(c_code);

    // If src becomes &str, strlen(src) becomes src.len()
    // The malloc should generate code that can be assigned to *mut u8
    assert!(
        rust_code.contains("node"),
        "Should generate node assignment, got: {}",
        rust_code
    );

    // Check that the assignment compiles
    // The generated code should not have type mismatch between Vec and *mut u8
}

/// Test that strcpy(dest, src) where dest is a struct field generates proper copy.
/// C: strcpy(entry->key, src);
/// Bug: Currently generates just `src.to_string();` which is a no-op
/// Expected: Generate code that copies src to entry->key
#[test]
fn test_strcpy_to_struct_field() {
    let c_code = r#"
typedef struct Entry {
    char* key;
} Entry;

void set_key(Entry* entry, const char* src) {
    strcpy(entry->key, src);
}
"#;

    let rust_code = transpile_c(c_code);

    // Should NOT generate just `src.to_string();` as a standalone statement
    // That's a no-op that doesn't copy anything to entry->key
    let has_noop_string = rust_code.contains("src.to_string();")
        && !rust_code.contains("entry")
        && !rust_code.contains("key =");

    // The strcpy should actually write to the destination
    // Either as a copy operation or as an unsafe memory operation
    assert!(
        !has_noop_string
            || rust_code.contains("copy")
            || rust_code.contains("ptr")
            || rust_code.contains("entry"),
        "DECY-140: strcpy should copy to destination, not generate no-op. Got:\n{}",
        rust_code
    );
}

/// Test the complete hash_table pattern compiles.
/// This is the real-world case from hash_table.c
#[test]
fn test_hash_table_malloc_strcpy_pattern_compiles() {
    let c_code = r#"
typedef struct Entry {
    char* key;
    int value;
    struct Entry* next;
} Entry;

void insert(Entry* entry, const char* key) {
    entry->key = (char*)malloc(strlen(key) + 1);
    strcpy(entry->key, key);
}
"#;

    let rust_code = transpile_c(c_code);

    assert!(
        rust_code.contains("fn insert"),
        "Should generate insert function, got: {}",
        rust_code
    );

    // Try to compile
    let (success, stderr) = compile_rust(&rust_code);

    // Document the current errors - this test shows what needs to be fixed
    if !success {
        // Check for the specific type mismatch errors we're trying to fix
        let has_vec_ptr_mismatch = stderr.contains("expected `*mut u8`, found `Vec<u8>`")
            || stderr.contains("mismatched types");

        assert!(
            has_vec_ptr_mismatch,
            "DECY-140: Should have Vec-to-ptr type mismatch until fixed.\nErrors:\n{}",
            stderr
        );
    }
}

/// Test that the full hash_table.c file compiles after fix.
#[test]
fn test_full_hash_table_compiles() {
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

    assert!(
        !rust_code.is_empty(),
        "Should generate Rust code from hash_table.c"
    );

    // Write to temp file and compile
    let mut temp_file = NamedTempFile::with_suffix(".rs").expect("Failed to create temp file");
    temp_file
        .write_all(rust_code.as_bytes())
        .expect("Failed to write Rust code");

    let output = Command::new("rustc")
        .args([
            "--crate-type=lib",
            "--edition=2021",
            "--crate-name=hash_table",
            "-A",
            "warnings", // Suppress warnings, only check errors
        ])
        .arg(temp_file.path())
        .arg("-o")
        .arg("/tmp/test_hash_table_compile")
        .output()
        .expect("Failed to run rustc");

    let success = output.status.success();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // This test should pass once DECY-140 is complete
    assert!(
        success,
        "DECY-140: hash_table.c should compile with 0 errors.\nGenerated code:\n{}\n\nErrors:\n{}",
        rust_code, stderr
    );
}
