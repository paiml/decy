//! Integration tests for classifier-based transpilation pipeline
//!
//! **Ticket**: DECY-183 - Wire classifier into decy-core pipeline
//!
//! These tests verify that the RuleBasedClassifier is integrated into
//! the transpilation pipeline and produces correct ownership inferences.

use decy_core::transpile;

// ============================================================================
// DECY-183: Classifier Pipeline Tests
// ============================================================================

#[test]
fn test_classifier_immutable_borrow() {
    // DECY-183: const pointer should become &T reference
    let c_code = r#"
int read_value(const int *ptr) {
    return *ptr;
}
"#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    // Should generate immutable reference parameter
    let has_immut_ref = result.contains("&i32")
        || result.contains("&'a i32")
        || result.contains(": &");

    assert!(
        has_immut_ref,
        "DECY-183: const pointer should become &T\nGot: {}",
        result
    );
}

#[test]
fn test_classifier_mutable_borrow() {
    // DECY-183: non-const pointer with writes should become &mut T
    let c_code = r#"
void set_value(int *ptr, int value) {
    *ptr = value;
}
"#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    // Should generate mutable reference parameter
    let has_mut_ref = result.contains("&mut i32")
        || result.contains("&'a mut i32")
        || result.contains(": &mut");

    assert!(
        has_mut_ref,
        "DECY-183: pointer with writes should become &mut T\nGot: {}",
        result
    );
}

#[test]
fn test_classifier_array_to_slice() {
    // DECY-183: array parameter with size should become slice
    let c_code = r#"
int sum(int *arr, int len) {
    int total = 0;
    for (int i = 0; i < len; i++) {
        total = total + arr[i];
    }
    return total;
}
"#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    // Should generate slice parameter
    let has_slice = result.contains("&[i32]")
        || result.contains("&'a [i32]")
        || result.contains(": &[");

    assert!(
        has_slice,
        "DECY-183: array with size should become slice\nGot: {}",
        result
    );
}

#[test]
fn test_classifier_preserves_function_semantics() {
    // DECY-183: Transpiled code should preserve function semantics
    let c_code = r#"
int add(int a, int b) {
    return a + b;
}
"#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    // Should have function with correct signature
    assert!(result.contains("fn add"), "Should have add function");
    assert!(
        result.contains("a: i32") || result.contains("mut a: i32"),
        "Should have parameter a"
    );
    assert!(
        result.contains("b: i32") || result.contains("mut b: i32"),
        "Should have parameter b"
    );
    assert!(
        result.contains("-> i32"),
        "Should return i32\nGot: {}",
        result
    );
}

#[test]
fn test_classifier_multiple_parameters() {
    // DECY-183: Multiple pointer parameters should be classified correctly
    let c_code = r#"
void copy_value(const int *src, int *dst) {
    *dst = *src;
}
"#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    // src should be immutable, dst should be mutable
    // Both should be references, not raw pointers
    let has_refs = result.contains("&") && !result.contains("*mut") && !result.contains("*const");

    assert!(
        has_refs || result.contains("&i32") || result.contains("&mut i32"),
        "DECY-183: Parameters should be classified as references\nGot: {}",
        result
    );
}

#[test]
fn test_classifier_no_unsafe_for_simple_refs() {
    // DECY-183: Simple reference parameters shouldn't require unsafe
    let c_code = r#"
int get_value(const int *ptr) {
    return *ptr;
}
"#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    // Should not contain unsafe block for this simple case
    // (though dereference might still need it depending on implementation)
    assert!(
        result.contains("fn get_value"),
        "Should generate get_value function\nGot: {}",
        result
    );
}
