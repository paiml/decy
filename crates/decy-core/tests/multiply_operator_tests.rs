//! Integration test: Multiply operator preservation in recursive/function calls
//!
//! **Test Category**: Integration (Regression)
//! **Purpose**: Verify multiply operator (*) is not confused with dereference (*)
//! **Ticket**: DECY-116 - P0 Critical Bug
//!
//! **Bug Description**:
//! When the right-hand side of a multiply operator is a function call with
//! arithmetic expression argument, the * is incorrectly converted to subtraction (-).
//!
//! **Example**:
//! C: `return n * factorial(n - 1);`
//! WRONG: `return n - factorial(n - 1);`
//! CORRECT: `return n * factorial(n - 1);`
//!
//! **Quality Gates**:
//! - Multiply operator preserved in all expression contexts
//! - No confusion with pointer dereference

use decy_core::transpile;

/// Test multiply with recursive function call
/// This is the primary failing case
#[test]
fn test_multiply_with_recursive_call() {
    let c_code = r#"
int rec(int n) {
    return n * rec(n - 1);
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // CRITICAL: The multiply operator MUST be preserved
    assert!(
        rust_code.contains("n * rec(n - 1)")
            || rust_code.contains("n * rec(n-1)")
            || rust_code.contains("* rec("),
        "DECY-116 BUG: Multiply operator converted to subtract!\nExpected: n * rec(n - 1)\nGot: {}",
        rust_code
    );

    // Verify it does NOT contain the buggy subtraction
    assert!(
        !rust_code.contains("n - rec(n - 1)"),
        "DECY-116 BUG: Found incorrect subtraction instead of multiplication!\nGot: {}",
        rust_code
    );
}

/// Test factorial pattern - the canonical failing case
#[test]
fn test_factorial_multiply_preserved() {
    let c_code = r#"
int factorial(int n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // The multiply MUST be preserved in the recursive call
    assert!(
        rust_code.contains("* factorial("),
        "DECY-116 BUG: Factorial multiply not preserved!\nGot: {}",
        rust_code
    );

    // Must NOT have subtraction instead of multiplication
    assert!(
        !rust_code.contains("n - factorial("),
        "DECY-116 BUG: Multiply incorrectly became subtract!\nGot: {}",
        rust_code
    );
}

/// Test multiply with function call having subtraction in argument
/// This isolates the specific pattern that triggers the bug
#[test]
fn test_multiply_with_function_call_subtraction_arg() {
    let c_code = r#"
int func(int x) {
    return x;
}

int test(int a, int b) {
    return a * func(b - 1);
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Multiply must be preserved
    assert!(
        rust_code.contains("a * func(") || rust_code.contains("* func("),
        "DECY-116 BUG: Multiply with function call not preserved!\nGot: {}",
        rust_code
    );
}

/// Test chained multiply with function calls
#[test]
fn test_chained_multiply_with_calls() {
    let c_code = r#"
int double_it(int x) {
    return x * 2;
}

int compute(int a, int b, int c) {
    return a * double_it(b) * double_it(c);
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Both multiplies must be preserved
    // Count occurrences of *
    let multiply_count = rust_code.matches(" * ").count() + rust_code.matches("a *").count();
    assert!(
        multiply_count >= 2,
        "Expected at least 2 multiply operators in chained expression!\nGot {} in: {}",
        multiply_count,
        rust_code
    );
}

/// Test that simple multiply still works (sanity check)
#[test]
fn test_simple_multiply_preserved() {
    let c_code = r#"
int mul(int a, int b) {
    return a * b;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    assert!(
        rust_code.contains("a * b"),
        "Simple multiply should work!\nGot: {}",
        rust_code
    );
}

/// Test multiply with variable that has subtraction in assignment
/// This should NOT trigger the bug (different pattern)
#[test]
fn test_multiply_with_variable_from_subtraction() {
    let c_code = r#"
int factorial(int n) {
    if (n <= 1) {
        return 1;
    }
    int sub = factorial(n - 1);
    return n * sub;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // This pattern with intermediate variable should work
    assert!(
        rust_code.contains("n * sub") || rust_code.contains("* sub"),
        "Multiply with intermediate variable should work!\nGot: {}",
        rust_code
    );
}

/// Test multiply followed by function call with addition (not just subtraction)
#[test]
fn test_multiply_with_addition_in_func_arg() {
    let c_code = r#"
int func(int x) {
    return x;
}

int test(int a, int b) {
    return a * func(b + 1);
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Multiply must be preserved regardless of operation in function arg
    assert!(
        rust_code.contains("* func("),
        "Multiply with addition in func arg should work!\nGot: {}",
        rust_code
    );
}
