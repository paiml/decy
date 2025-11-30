//! DECY-131: Integer in boolean context tests.
//!
//! In C, any non-zero value is truthy in boolean context (&&, ||, if).
//! In Rust, we need explicit conversion: `x != 0` for i32 → bool.
//!
//! C: if (x && y)  where x, y are int
//! Expected Rust: if x != 0 && y != 0

use decy_core::transpile;

/// Test that struct fields in && context get boolean conversion.
///
/// C: if (f.active && f.ready)  where active, ready are int
/// Expected: if f.active != 0 && f.ready != 0
#[test]
fn test_int_fields_in_logical_and_context() {
    let c_code = r#"
        struct Flags {
            int active;
            int ready;
        };

        int check(struct Flags f) {
            if (f.active && f.ready) {
                return 1;
            }
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should convert int to bool for && operator
    // Either explicit != 0 or generate bool type
    assert!(
        result.contains("!= 0") || result.contains(": bool"),
        "Should convert int to bool in && context\nGenerated:\n{}",
        result
    );
}

/// Test that int variables in if condition get boolean conversion.
///
/// C: if (x)  where x is int
/// Expected: if x != 0
#[test]
fn test_int_variable_in_if_condition() {
    let c_code = r#"
        int is_nonzero(int x) {
            if (x) {
                return 1;
            }
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Should convert int to bool for if condition
    assert!(
        result.contains("!= 0") || result.contains("x != 0") || result.contains("x as bool"),
        "Should convert int to bool in if condition\nGenerated:\n{}",
        result
    );
}

/// Test logical NOT on integer.
///
/// C: if (!error)  where error is int
/// Expected: if error == 0
#[test]
fn test_logical_not_on_int() {
    let c_code = r#"
        int check_no_error(int error) {
            if (!error) {
                return 1;
            }
            return 0;
        }
    "#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    println!("Generated Rust code:\n{}", result);

    // Logical NOT on int should become == 0
    assert!(
        result.contains("== 0") || result.contains("error == 0"),
        "Should convert !int to int == 0\nGenerated:\n{}",
        result
    );
}
