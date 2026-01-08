// =============================================================================
// DECY-151: Fix char-to-int return type coercion
// =============================================================================
// In C, char arithmetic like `*s1 - *s2` is auto-promoted to int.
// When returning char subtraction from an int function, Rust needs explicit cast.
//
// C code:
//   int string_compare(char* s1, char* s2) {
//       return *s1 - *s2;  // char - char → int (C auto-promotes)
//   }
//
// Expected Rust:
//   fn string_compare(s1: &[u8], s2: &[u8]) -> i32 {
//       return (s1[idx] as i32) - (s2[idx] as i32);  // Explicit cast to i32
//   }
//
// NOT:
//   return s1[idx] - s2[idx];  // u8 - u8 → u8, but return type is i32!

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
            "--crate-name=test_char_coercion",
            "-A",
            "warnings",
            "-o",
            "/tmp/decy_char_coercion_test",
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

/// Test: Char subtraction returned as int should compile
#[test]
fn test_char_subtraction_return_compiles() {
    let c_code = r#"
int compare_chars(char* a, char* b) {
    return *a - *b;
}
"#;

    let rust_code = transpile_c(c_code);

    match compiles(&rust_code) {
        Ok(()) => {}
        Err(e) => panic!(
            "DECY-151: Char subtraction return should compile.\nCode:\n{}\nErrors:\n{}",
            rust_code, e
        ),
    }
}

/// Test: String comparison function should compile
#[test]
fn test_string_compare_compiles() {
    let c_code = r#"
int string_compare(char* s1, char* s2) {
    while (*s1 != 0 && *s2 != 0) {
        if (*s1 != *s2) {
            return *s1 - *s2;
        }
        s1 = s1 + 1;
        s2 = s2 + 1;
    }
    return *s1 - *s2;
}
"#;

    let rust_code = transpile_c(c_code);

    match compiles(&rust_code) {
        Ok(()) => {}
        Err(e) => panic!(
            "DECY-151: String compare should compile.\nCode:\n{}\nErrors:\n{}",
            rust_code, e
        ),
    }
}

/// Test: string_utils.c should compile
#[test]
fn test_string_utils_compiles() {
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "decy",
            "--quiet",
            "--",
            "transpile",
            "examples/real-world/string_utils.c",
        ])
        .current_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap())
        .output()
        .expect("Failed to run decy transpile");

    let rust_code = String::from_utf8_lossy(&output.stdout).to_string();

    match compiles(&rust_code) {
        Ok(()) => {}
        Err(e) => panic!(
            "DECY-151: string_utils.c should compile.\nCode:\n{}\nErrors:\n{}",
            rust_code, e
        ),
    }
}
