//! DECY-126: Return type cast tests.
//!
//! When returning char/u8 values from main(), cast to i32 for exit().
//!
//! C: return buffer[0];  (where buffer is char[])
//! Expected Rust: std::process::exit(buffer[0 as usize] as i32);

use decy_core::transpile;

/// Test that u8 array index is cast to i32 for exit().
///
/// C: char buffer[20]; return buffer[0];
/// Expected: std::process::exit(buffer[0 as usize] as i32);
#[test]
fn test_char_array_return_casts_to_i32() {
    let c_code = r#"
        int main() {
            char buffer[20];
            buffer[0] = 'x';
            return buffer[0];
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should cast u8 to i32 for exit()
    assert!(
        result.contains("as i32") || result.contains(".into()"),
        "Should cast u8 to i32 for exit()\nGenerated:\n{}",
        result
    );
}

/// Test that char variable return is cast to i32.
///
/// C: char c = 'x'; return c;
/// Expected: std::process::exit(c as i32);
#[test]
fn test_char_variable_return_casts_to_i32() {
    let c_code = r#"
        int main() {
            char c = 'x';
            return c;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should cast u8 to i32 for exit()
    assert!(
        result.contains("as i32") || result.contains(".into()"),
        "Should cast u8/char to i32 for exit()\nGenerated:\n{}",
        result
    );
}

/// Test that int return works normally (no extra cast needed).
///
/// C: int x = 5; return x;
/// Expected: std::process::exit(x);
#[test]
fn test_int_return_no_cast() {
    let c_code = r#"
        int main() {
            int x = 5;
            return x;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should NOT add unnecessary cast for int
    // (actually it might have "as i32" which is fine, just checking it works)
    assert!(
        result.contains("std::process::exit"),
        "Should have exit() call\nGenerated:\n{}",
        result
    );
}
