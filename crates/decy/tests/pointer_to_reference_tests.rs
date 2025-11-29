//! DECY-111: Pointer Parameter to Reference Transformation Tests
//!
//! **Goal**: Transform C pointer parameters to Rust references instead of raw pointers.
//!
//! **Current Behavior**:
//! - `void swap(int *a, int *b)` → `fn swap(mut a: *mut i32, mut b: *mut i32)`
//! - Call sites: `swap(&x, &y)` fails with E0308 (type mismatch)
//!
//! **Expected Behavior**:
//! - `void swap(int *a, int *b)` → `fn swap(a: &mut i32, b: &mut i32)`
//! - Call sites: `swap(&mut x, &mut y)` compiles cleanly
//!
//! **Safety Impact**: Eliminates unsafe blocks for pointer dereference.

use decy_core::transpile;

/// Test: Swap function with mutable pointer parameters.
///
/// C: `void swap(int *a, int *b)` with mutation of both params
/// Expected: `fn swap(a: &mut i32, b: &mut i32)` with NO unsafe blocks
#[test]
fn test_swap_ptr_to_mut_ref() {
    let c_code = r#"
void swap(int *a, int *b) {
    int temp = *a;
    *a = *b;
    *b = temp;
}

int main() {
    int x = 10;
    int y = 20;
    swap(&x, &y);
    return x;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Should NOT have raw pointer types
    assert!(
        !rust_code.contains("*mut i32"),
        "Should NOT use raw pointers for non-FFI params!\nGot:\n{}",
        rust_code
    );

    // Should have mutable reference types
    assert!(
        rust_code.contains("&mut i32") || rust_code.contains("& mut i32"),
        "Should use mutable references for mutating params!\nGot:\n{}",
        rust_code
    );

    // Should NOT have unsafe dereference
    assert!(
        !rust_code.contains("unsafe {") || rust_code.matches("unsafe").count() == 0,
        "Should NOT need unsafe blocks for reference operations!\nGot:\n{}",
        rust_code
    );
}

/// Test: Read-only pointer parameter becomes immutable reference.
///
/// C: `int sum(const int *arr, int len)` - read-only access
/// Expected: `fn sum(arr: &i32, len: i32)` or slice `fn sum(arr: &[i32])`
#[test]
fn test_readonly_param_to_immut_ref() {
    let c_code = r#"
int read_value(int *ptr) {
    return *ptr;
}

int main() {
    int x = 42;
    return read_value(&x);
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // For read-only access, could be &i32 (immutable ref) or &mut i32
    // Either is acceptable for this test, but should NOT be *mut
    assert!(
        !rust_code.contains("*mut i32") && !rust_code.contains("*const i32"),
        "Should NOT use raw pointers for simple params!\nGot:\n{}",
        rust_code
    );
}

/// Test: Call site generates correct reference syntax.
///
/// When function takes `&mut i32`, call site should use `&mut x` not `&x`.
#[test]
fn test_call_site_mut_ref_syntax() {
    let c_code = r#"
void increment(int *x) {
    *x = *x + 1;
}

int main() {
    int val = 0;
    increment(&val);
    return val;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Call site should use mutable reference
    // The pattern should be increment(&mut val) or similar
    assert!(
        rust_code.contains("&mut"),
        "Call site should use mutable reference syntax!\nGot:\n{}",
        rust_code
    );
}

/// Test: Swap pattern from training corpus (swap_ptr.c).
///
/// This is the exact pattern from reprorusted-c-cli that was failing.
#[test]
fn test_corpus_swap_pattern() {
    let c_code = r#"
void swap(int *a, int *b) {
    int temp = *a;
    *a = *b;
    *b = temp;
}

int main() {
    int x = 10;
    int y = 20;
    swap(&x, &y);
    return x;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // The generated code should compile without E0308 type mismatch
    // This means params should be references, not raw pointers
    assert!(
        !rust_code.contains("fn swap(mut a: *mut i32"),
        "Swap should NOT have raw pointer params!\nGot:\n{}",
        rust_code
    );
}

/// Test: Increment both pattern from training corpus (mut_borrow.c).
///
/// Note: This C code has undefined behavior (aliased mutable access),
/// but we should still transpile it without type errors.
#[test]
fn test_corpus_increment_both_pattern() {
    let c_code = r#"
void increment_both(int *a, int *b) {
    *a = *a + 1;
    *b = *b + 1;
}

int main() {
    int x = 0;
    increment_both(&x, &x);
    return x;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Should not have raw pointers
    assert!(
        !rust_code.contains("fn increment_both(mut a: *mut i32"),
        "Should NOT have raw pointer params!\nGot:\n{}",
        rust_code
    );
}

/// Test: Return pointer pattern should be handled differently.
///
/// C: `int* get_ptr(int *arr)` - returning a pointer is different
/// This may need to stay as raw pointer for certain patterns.
#[test]
fn test_return_pointer_pattern() {
    let c_code = r#"
int* get_first(int *arr) {
    return arr;
}

int main() {
    int nums[3];
    nums[0] = 42;
    int *p = get_first(nums);
    return *p;
}
"#;

    let result = transpile(c_code);
    // This is a more complex pattern - returning pointer
    // May or may not transform cleanly
    if let Ok(rust_code) = result {
        // At minimum, should not panic
        assert!(!rust_code.is_empty());
    }
}

/// Test: Output parameter pattern.
///
/// C: `void get_dimensions(int *width, int *height)`
/// Common pattern for "returning" multiple values.
#[test]
fn test_output_parameter_pattern() {
    let c_code = r#"
void get_dimensions(int *width, int *height) {
    *width = 800;
    *height = 600;
}

int main() {
    int w, h;
    get_dimensions(&w, &h);
    return w + h;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Should use mutable references for output params
    assert!(
        !rust_code.contains("*mut i32") || rust_code.contains("&mut"),
        "Output params should use mutable references!\nGot:\n{}",
        rust_code
    );
}

/// Test: Mixed read/write params.
///
/// C: `void update(int *src, int *dst)` - src is read, dst is written
#[test]
fn test_mixed_read_write_params() {
    let c_code = r#"
void copy_value(int *src, int *dst) {
    *dst = *src;
}

int main() {
    int a = 42;
    int b = 0;
    copy_value(&a, &b);
    return b;
}
"#;

    let result = transpile(c_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let rust_code = result.unwrap();

    // Both params used with pointers, at least dst needs &mut
    // Should not use raw pointers
    assert!(
        !rust_code.contains("fn copy_value(mut src: *mut"),
        "Should NOT have raw pointer params!\nGot:\n{}",
        rust_code
    );
}
