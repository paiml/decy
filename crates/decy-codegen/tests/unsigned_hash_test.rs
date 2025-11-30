//! Test for DECY-158: unsigned int should become u32, not i32
//!
//! C's `unsigned int` must map to Rust's `u32` to prevent:
//! 1. Negative array indices (panic at runtime)
//! 2. Incorrect modulo results for hash functions
//!
//! Reference: ISO C99 §6.2.5 - Types

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

/// Test: unsigned int should become u32
#[test]
fn test_unsigned_int_becomes_u32() {
    let c_code = r#"
unsigned int hash(const char* key) {
    unsigned int h = 0;
    return h;
}
"#;

    let rust_code = transpile_c(c_code);

    // Should use u32 for unsigned int
    assert!(
        rust_code.contains("u32") || rust_code.contains("-> u32"),
        "DECY-158: unsigned int should become u32, not i32\nGenerated:\n{}",
        rust_code
    );

    // Should NOT use i32 for hash variable
    assert!(
        !rust_code.contains("let mut h: i32"),
        "DECY-158: unsigned int variable should be u32\nGenerated:\n{}",
        rust_code
    );
}

/// Test: unsigned int modulo should not produce negative values
#[test]
fn test_unsigned_modulo_safe_indexing() {
    let c_code = r#"
#define TABLE_SIZE 100

unsigned int hash(const char* key) {
    unsigned int h = 0;
    while (*key) {
        h = (h << 5) + *key++;
    }
    return h % TABLE_SIZE;
}
"#;

    let rust_code = transpile_c(c_code);

    // The return type should be unsigned
    assert!(
        rust_code.contains("-> u32") || rust_code.contains("-> usize"),
        "DECY-158: hash function should return unsigned type\nGenerated:\n{}",
        rust_code
    );
}

/// Test: unsigned int in array index context
#[test]
fn test_unsigned_array_index() {
    let c_code = r#"
void set_bucket(int* buckets, unsigned int index, int value) {
    buckets[index] = value;
}
"#;

    let rust_code = transpile_c(c_code);

    // unsigned int index should not need cast to usize (already unsigned)
    // or should be u32 which safely converts to usize
    assert!(
        rust_code.contains("u32") || rust_code.contains("usize"),
        "DECY-158: unsigned index should be u32 or usize\nGenerated:\n{}",
        rust_code
    );
}
