//! DECY-122: Character literal tests.
//!
//! Tests that character literals including escape sequences are properly parsed
//! and converted to Rust byte literals.
//!
//! C: char c = 'a';  →  Rust: let mut c: u8 = b'a';
//! C: char c = '\0'; →  Rust: let mut c: u8 = 0u8;
//! C: char c = '\n'; →  Rust: let mut c: u8 = 10u8;

use decy_core::transpile;

/// Test that plain character literals are converted properly.
///
/// C: char c = 'x';
/// Expected: let mut c: u8 = b'x';
#[test]
fn test_plain_char_literal() {
    let c_code = r#"
        int main() {
            char c = 'x';
            return c;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should contain b'x' (byte literal)
    assert!(
        result.contains("b'x'"),
        "Should convert 'x' to b'x'\nGenerated:\n{}",
        result
    );
}

/// Test that null character escape sequence works.
///
/// C: char c = '\0';
/// Expected: let mut c: u8 = 0u8;
#[test]
fn test_null_char_escape() {
    let c_code = r#"
        int main() {
            char c = '\0';
            return c;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should contain 0u8 for null character
    assert!(
        result.contains("0u8"),
        "Should convert '\\0' to 0u8\nGenerated:\n{}",
        result
    );
}

/// Test that newline escape sequence works.
///
/// C: char c = '\n';
/// Expected: Rust code with appropriate byte value (10)
#[test]
fn test_newline_char_escape() {
    let c_code = r#"
        int main() {
            char c = '\n';
            return c;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should contain 10u8 for newline (or b'\n')
    assert!(
        result.contains("10u8") || result.contains("b'\\n'"),
        "Should convert '\\n' to 10u8 or b'\\n'\nGenerated:\n{}",
        result
    );
}

/// Test that tab escape sequence works.
///
/// C: char c = '\t';
/// Expected: Rust code with appropriate byte value (9)
#[test]
fn test_tab_char_escape() {
    let c_code = r#"
        int main() {
            char c = '\t';
            return c;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should contain 9u8 for tab (or b'\t')
    assert!(
        result.contains("9u8") || result.contains("b'\\t'"),
        "Should convert '\\t' to 9u8 or b'\\t'\nGenerated:\n{}",
        result
    );
}

/// Test that character comparison with escape sequence works.
///
/// C: if (*str == '\0') { ... }
/// Expected: if *str == 0u8 { ... }
#[test]
fn test_char_comparison_with_null() {
    let c_code = r#"
        int check_null(char *s) {
            if (*s == '\0') {
                return 0;
            }
            return 1;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should have the if statement (not empty body)
    assert!(
        result.contains("if") && result.contains("return") && result.contains("0u8"),
        "Should have if statement comparing to 0u8\nGenerated:\n{}",
        result
    );
}
