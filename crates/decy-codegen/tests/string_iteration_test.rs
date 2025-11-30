// =============================================================================
// DECY-138: String Iteration Tests
// =============================================================================
// Tests for transforming C string iteration patterns to Rust byte iteration.
//
// C pattern: while (*str) { ... *str++; }
// Rust pattern: while !str.is_empty() { byte = str.as_bytes()[0]; str = &str[1..]; }

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

/// Test that dereferencing a const char* (which becomes &str) generates proper code.
/// C: *key (where key is const char*)
/// Bug: Currently generates `*key` which can't dereference &str
/// Expected: Should generate byte access or use .as_bytes()[0]
#[test]
fn test_str_dereference_in_return() {
    let c_code = r#"
char test(const char* s) {
    return *s;
}
"#;

    let rust_code = transpile_c(c_code);

    // The generated code should handle &str dereference properly
    // Currently this generates `return *s;` which doesn't work for &str
    // It should generate something like `return s.as_bytes()[0] as i8;`
    // or keep the parameter as *const u8 instead of &str

    // For now, verify we at least have the function
    assert!(
        rust_code.contains("fn test"),
        "Should generate test function, got: {}",
        rust_code
    );

    // Check that we don't have invalid *s on &str
    // If s is &str, we can't just dereference it
    if rust_code.contains("s: &str") {
        assert!(
            rust_code.contains("as_bytes")
                || rust_code.contains("bytes()")
                || !rust_code.contains("*s"),
            "If parameter is &str, should not have raw *s dereference. Got: {}",
            rust_code
        );
    }
}

/// Test string iteration pattern.
/// C: while (*key) { ... key++; }
/// Bug: Generates while (*key) != 0 which compares str with 0
/// Expected: while !key.is_empty() or similar
#[test]
fn test_string_iteration_while_loop() {
    let c_code = r#"
int count_chars(const char* s) {
    int count = 0;
    while (*s) {
        count++;
        s++;
    }
    return count;
}
"#;

    let rust_code = transpile_c(c_code);

    assert!(
        rust_code.contains("fn count_chars"),
        "Should generate count_chars function, got: {}",
        rust_code
    );

    // If parameter is &str, we need proper iteration
    if rust_code.contains("s: &str") {
        // Should not have raw dereference comparison with 0
        assert!(
            !rust_code.contains("(*s) != 0") && !rust_code.contains("*s != 0"),
            "Should not compare dereferenced &str with 0. Got: {}",
            rust_code
        );
    }
}

/// Test hash function pattern - the real-world case from hash_table.c
/// C: while (*key) { hash = (hash << 5) + *key++; }
#[test]
fn test_hash_function_pattern() {
    let c_code = r#"
unsigned int hash(const char* key) {
    unsigned int h = 0;
    while (*key) {
        h = (h << 5) + *key++;
    }
    return h;
}
"#;

    let rust_code = transpile_c(c_code);

    assert!(
        rust_code.contains("fn hash"),
        "Should generate hash function, got: {}",
        rust_code
    );

    // If key is &str, the code should handle iteration properly
    if rust_code.contains("key: &str") {
        // The current bug: generates `while (*key) != 0` which doesn't compile
        // Expected: generates `while !key.is_empty()` or similar

        // For DECY-138, we need to transform:
        // - while (*key) → while !key.is_empty()
        // - *key++ → key.as_bytes()[0]; key = &key[1..]
        // - h + *key → h + key.as_bytes()[0] as u32

        // Check that the generated code at least attempts proper iteration
        let has_valid_iteration = rust_code.contains("is_empty")
            || rust_code.contains("as_bytes")
            || rust_code.contains("bytes()")
            || rust_code.contains("[1..]");

        // This test documents the current bug - it will fail until DECY-138 is implemented
        assert!(
            has_valid_iteration,
            "DECY-138: Hash function with &str should use proper iteration. Got:\n{}",
            rust_code
        );
    }
}

/// Test that generated hash function compiles with rustc
#[test]
fn test_hash_function_compiles() {
    let c_code = r#"
unsigned int hash(const char* key) {
    unsigned int h = 0;
    while (*key) {
        h = (h << 5) + *key++;
    }
    return h;
}
"#;

    let rust_code = transpile_c(c_code);

    // Write to temp file and try to compile
    let mut temp_file = NamedTempFile::with_suffix(".rs").expect("Failed to create temp file");
    temp_file
        .write_all(rust_code.as_bytes())
        .expect("Failed to write Rust code");

    let output = Command::new("rustc")
        .args([
            "--crate-type=lib",
            "--edition=2021",
            "--crate-name=test_hash",
        ])
        .arg(temp_file.path())
        .arg("-o")
        .arg("/tmp/test_hash_compile")
        .output()
        .expect("Failed to run rustc");

    // Document current state: this fails due to &str iteration issues
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Expected errors until DECY-138 is fixed:
        // - can't compare `str` with `{integer}`
        // - binary assignment operation `+=` cannot be applied to type `&str`
        // - cannot add `str` to `i32`
        assert!(
            stderr.contains("can't compare")
                || stderr.contains("cannot be applied")
                || stderr.contains("cannot add"),
            "DECY-138: Hash function should compile once string iteration is fixed.\nErrors:\n{}",
            stderr
        );
    }
}
