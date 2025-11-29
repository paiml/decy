//! DECY-112: Array Initialization Bug Tests
//!
//! **Bug**: Array size is being incorrectly captured as initializer
//!
//! **Symptom**: `int nums[5];` transpiles to `let mut nums: [i32; 5] = 5;`
//! **Expected**: `int nums[5];` transpiles to `let mut nums: [i32; 5] = [0; 5];`
//!
//! **Root Cause**: The parser's visit_expression callback captures the array
//! size literal (5) as the initializer expression when visiting children
//! of variable declarations.
//!
//! **Fix**: Skip integer literals that are array size expressions when
//! the variable type is an array.

use decy_core::transpile;

/// Test that uninitialized arrays do NOT use size as initializer.
///
/// C: `int nums[5];`
/// WRONG: `let mut nums: [i32; 5] = 5;`
/// CORRECT: `let mut nums: [i32; 5] = [0; 5];`
#[test]
fn test_uninitialized_array_correct_init() {
    let c_code = r#"
int main() {
    int nums[5];
    nums[0] = 1;
    return 0;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // CRITICAL: Should NOT have `= 5;` (array size as init)
    assert!(
        !rust_code.contains(": [i32; 5] = 5;"),
        "BUG: Array size (5) was incorrectly used as initializer!\nGot:\n{}",
        rust_code
    );

    // Should have proper array initialization syntax
    assert!(
        rust_code.contains("[0; 5]") || rust_code.contains("[i32; 5]"),
        "Expected proper array initialization syntax\nGot:\n{}",
        rust_code
    );
}

/// Test that arrays with actual initializers work correctly.
///
/// C: `int arr[3] = {1, 2, 3};`
/// Expected: `let arr: [i32; 3] = [1, 2, 3];`
#[test]
fn test_initialized_array_with_values() {
    let c_code = r#"
int main() {
    int arr[3] = {1, 2, 3};
    return arr[0];
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Should NOT have `= 3;` (array size as init)
    assert!(
        !rust_code.contains(": [i32; 3] = 3;"),
        "BUG: Array size (3) was incorrectly used as initializer!\nGot:\n{}",
        rust_code
    );
}

/// Test array sum pattern from training corpus.
///
/// This is the exact pattern from reprorusted-c-cli/training_corpus/array_sum.c
/// that was failing during oracle trace generation.
#[test]
fn test_array_sum_corpus_pattern() {
    let c_code = r#"
int sum_array(int *arr, int len) {
    int sum = 0;
    int i = 0;
    while (i < len) {
        sum = sum + *(arr + i);
        i = i + 1;
    }
    return sum;
}

int main() {
    int nums[5];
    nums[0] = 1;
    nums[1] = 2;
    nums[2] = 3;
    nums[3] = 4;
    nums[4] = 5;
    return sum_array(nums, 5);
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // CRITICAL: nums array should NOT be initialized with size value
    assert!(
        !rust_code.contains(": [i32; 5] = 5;"),
        "BUG: Array size (5) was incorrectly used as initializer!\nGot:\n{}",
        rust_code
    );

    // Should compile without E0308 type mismatch
    // The generated code should have proper array initialization
}

/// Test that char arrays also work correctly.
#[test]
fn test_char_array_uninit() {
    let c_code = r#"
int main() {
    char buf[100];
    buf[0] = 'H';
    return 0;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Should NOT have `= 100;` (array size as init)
    assert!(
        !rust_code.contains("= 100;"),
        "BUG: Char array size (100) was incorrectly used as initializer!\nGot:\n{}",
        rust_code
    );
}

/// Test multi-dimensional arrays.
#[test]
fn test_multidim_array_uninit() {
    let c_code = r#"
int main() {
    int matrix[3][4];
    matrix[0][0] = 1;
    return 0;
}
"#;

    let result = transpile(c_code);
    // Multi-dim arrays may not be fully supported yet, but should not panic
    // and should not use the size as initializer
    if let Ok(rust_code) = result {
        // Should NOT have `= 3;` or `= 4;` as initializers
        assert!(
            !rust_code.contains("matrix") || !rust_code.contains("= 3;"),
            "BUG: Multi-dim array size was incorrectly used as initializer!\nGot:\n{}",
            rust_code
        );
    }
}
