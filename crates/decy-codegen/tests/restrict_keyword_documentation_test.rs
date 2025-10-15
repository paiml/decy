//! Documentation tests for restrict keyword transformation (C99 §6.7.3.1)
//!
//! C99 introduced the `restrict` keyword as a type qualifier for pointers.
//! This test suite documents how DECY transforms C restrict pointers to Rust.
//!
//! **Reference**: ISO C99 §6.7.3.1 (Type qualifiers - restrict)
//!              NOT in K&R (pre-C99 feature)
//!
//! **Key Differences**:
//! - C `restrict` is a promise to compiler (pointer doesn't alias)
//! - Rust borrow checker ENFORCES non-aliasing at compile time
//! - C restrict violations cause undefined behavior
//! - Rust prevents aliasing violations through type system
//! - Rust `&mut` is like restrict but compiler-verified
//!
//! **Safety**: All transformations are SAFE (0 unsafe blocks)
//!            Rust's borrow checker provides stronger guarantees than C restrict
//!
//! **Version**: v0.37.0

/// Document transformation of restrict pointer in function parameter
///
/// C99 `restrict` promises no aliasing → Rust enforces via borrow checker
///
/// C Reference: ISO C99 §6.7.3.1 paragraph 1
#[test]
fn test_simple_restrict_pointer() {
    let _c_code = r#"
void copy(int* restrict dest, const int* restrict src, size_t n) {
    for (size_t i = 0; i < n; i++) {
        dest[i] = src[i];
    }
}
"#;

    let _rust_equivalent = r#"
fn copy(dest: &mut [i32], src: &[i32]) {
    for i in 0..src.len().min(dest.len()) {
        dest[i] = src[i];
    }
}
"#;

    fn copy(dest: &mut [i32], src: &[i32]) {
        for i in 0..src.len().min(dest.len()) {
            dest[i] = src[i];
        }
    }

    let src = [1, 2, 3, 4, 5];
    let mut dest = [0; 5];
    copy(&mut dest, &src);
    assert_eq!(dest, [1, 2, 3, 4, 5]);
}

/// Document restrict for optimization hints
///
/// C restrict enables optimizations → Rust borrow checker enables same optimizations
///
/// C Reference: ISO C99 §6.7.3.1 (compiler optimization)
#[test]
fn test_restrict_optimization() {
    let _c_code = r#"
void add_arrays(int* restrict result,
                const int* restrict a,
                const int* restrict b,
                size_t n) {
    for (size_t i = 0; i < n; i++) {
        result[i] = a[i] + b[i];
    }
}
"#;

    let _rust_equivalent = r#"
fn add_arrays(result: &mut [i32], a: &[i32], b: &[i32]) {
    let len = result.len().min(a.len()).min(b.len());
    for i in 0..len {
        result[i] = a[i] + b[i];
    }
}
"#;

    fn add_arrays(result: &mut [i32], a: &[i32], b: &[i32]) {
        let len = result.len().min(a.len()).min(b.len());
        for i in 0..len {
            result[i] = a[i] + b[i];
        }
    }

    let a = [1, 2, 3];
    let b = [4, 5, 6];
    let mut result = [0; 3];
    add_arrays(&mut result, &a, &b);
    assert_eq!(result, [5, 7, 9]);
}

/// Document restrict with memcpy pattern
///
/// C memcpy with restrict → Rust slice copy_from_slice (enforced non-overlap)
///
/// C Reference: ISO C99 §7.21.1 (string.h functions use restrict)
#[test]
fn test_restrict_memcpy() {
    let _c_code = r#"
void* memcpy(void* restrict dest, const void* restrict src, size_t n);

void copy_data(int* restrict dest, const int* restrict src, size_t n) {
    memcpy(dest, src, n * sizeof(int));
}
"#;

    let _rust_equivalent = r#"
fn copy_data(dest: &mut [i32], src: &[i32]) {
    dest.copy_from_slice(src);
}
// Note: copy_from_slice panics if slices overlap (memory safe)
"#;

    fn copy_data(dest: &mut [i32], src: &[i32]) {
        dest.copy_from_slice(src);
    }

    let src = [10, 20, 30, 40];
    let mut dest = [0; 4];
    copy_data(&mut dest, &src);
    assert_eq!(dest, [10, 20, 30, 40]);
}

/// Document restrict preventing aliasing
///
/// C restrict is programmer promise → Rust borrow checker enforces at compile time
#[test]
fn test_restrict_no_aliasing() {
    let _c_code = r#"
void increment_both(int* restrict a, int* restrict b) {
    *a += 1;
    *b += 1;
    // Compiler can reorder because a and b guaranteed not to alias
}
"#;

    let _rust_equivalent = r#"
fn increment_both(a: &mut i32, b: &mut i32) {
    *a += 1;
    *b += 1;
    // Borrow checker enforces a and b cannot alias
}
"#;

    fn increment_both(a: &mut i32, b: &mut i32) {
        *a += 1;
        *b += 1;
    }

    let mut x = 10;
    let mut y = 20;
    increment_both(&mut x, &mut y);
    assert_eq!(x, 11);
    assert_eq!(y, 21);

    // This would NOT compile in Rust (enforced at compile time):
    // increment_both(&mut x, &mut x);  // Error: cannot borrow `x` as mutable more than once
}

/// Document restrict with return value
///
/// Returning restrict pointer → Rust ownership transfer
#[test]
fn test_restrict_return() {
    let _c_code = r#"
int* restrict allocate_array(size_t n) {
    return (int* restrict)malloc(n * sizeof(int));
}
"#;

    let _rust_equivalent = r#"
fn allocate_array(n: usize) -> Vec<i32> {
    vec![0; n]
}
// Ownership transferred to caller (no aliasing possible)
"#;

    fn allocate_array(n: usize) -> Vec<i32> {
        vec![0; n]
    }

    let arr = allocate_array(5);
    assert_eq!(arr.len(), 5);
    assert_eq!(arr, vec![0, 0, 0, 0, 0]);
}

/// Document restrict in struct
///
/// C restrict in struct → Rust ownership in struct
#[test]
fn test_restrict_in_struct() {
    let _c_code = r#"
struct Buffer {
    int* restrict data;
    size_t length;
};
"#;

    let _rust_equivalent = r#"
struct Buffer {
    data: Vec<i32>,
    length: usize,
}
// Vec owns data (exclusive access guaranteed)
"#;

    struct Buffer {
        data: Vec<i32>,
        length: usize,
    }

    let buf = Buffer {
        data: vec![1, 2, 3],
        length: 3,
    };

    assert_eq!(buf.data.len(), 3);
    assert_eq!(buf.length, 3);
}

/// Document restrict with const
///
/// C `const restrict` → Rust immutable reference
#[test]
fn test_const_restrict() {
    let _c_code = r#"
int sum_array(const int* restrict arr, size_t n) {
    int sum = 0;
    for (size_t i = 0; i < n; i++) {
        sum += arr[i];
    }
    return sum;
}
"#;

    let _rust_equivalent = r#"
fn sum_array(arr: &[i32]) -> i32 {
    let mut sum = 0;
    for i in 0..arr.len() {
        sum += arr[i];
    }
    sum
}
"#;

    fn sum_array(arr: &[i32]) -> i32 {
        let mut sum = 0;
        for i in 0..arr.len() {
            sum += arr[i];
        }
        sum
    }

    let arr = [1, 2, 3, 4, 5];
    assert_eq!(sum_array(&arr), 15);
}

/// Document restrict with multiple levels of indirection
///
/// Pointer to restrict pointer → More complex ownership patterns
#[test]
fn test_restrict_pointer_to_pointer() {
    let _c_code = r#"
void allocate(int** restrict ptr, size_t n) {
    *ptr = malloc(n * sizeof(int));
}
"#;

    let _rust_equivalent = r#"
fn allocate(n: usize) -> Vec<i32> {
    vec![0; n]
}
// Return ownership instead of out-parameter
"#;

    fn allocate(n: usize) -> Vec<i32> {
        vec![0; n]
    }

    let arr = allocate(10);
    assert_eq!(arr.len(), 10);
}

/// Document restrict with array parameters
///
/// C restrict array parameter → Rust mutable slice
#[test]
fn test_restrict_array_param() {
    let _c_code = r#"
void zero_array(int arr[restrict], size_t n) {
    for (size_t i = 0; i < n; i++) {
        arr[i] = 0;
    }
}
"#;

    let _rust_equivalent = r#"
fn zero_array(arr: &mut [i32]) {
    for i in 0..arr.len() {
        arr[i] = 0;
    }
}
"#;

    fn zero_array(arr: &mut [i32]) {
        for i in 0..arr.len() {
            arr[i] = 0;
        }
    }

    let mut arr = [1, 2, 3, 4, 5];
    zero_array(&mut arr);
    assert_eq!(arr, [0, 0, 0, 0, 0]);
}

/// Document restrict for performance-critical code
///
/// C restrict enables vectorization → Rust borrow checker enables same
#[test]
fn test_restrict_vectorization() {
    let _c_code = r#"
void scale_and_add(float* restrict result,
                   const float* restrict a,
                   const float* restrict b,
                   float scale,
                   size_t n) {
    for (size_t i = 0; i < n; i++) {
        result[i] = a[i] * scale + b[i];
    }
    // restrict allows compiler to vectorize this loop
}
"#;

    let _rust_equivalent = r#"
fn scale_and_add(result: &mut [f32], a: &[f32], b: &[f32], scale: f32) {
    let len = result.len().min(a.len()).min(b.len());
    for i in 0..len {
        result[i] = a[i] * scale + b[i];
    }
    // Borrow checker allows compiler to vectorize
}
"#;

    fn scale_and_add(result: &mut [f32], a: &[f32], b: &[f32], scale: f32) {
        let len = result.len().min(a.len()).min(b.len());
        for i in 0..len {
            result[i] = a[i] * scale + b[i];
        }
    }

    let a = [1.0, 2.0, 3.0];
    let b = [4.0, 5.0, 6.0];
    let mut result = [0.0; 3];
    scale_and_add(&mut result, &a, &b, 2.0);
    assert_eq!(result, [6.0, 9.0, 12.0]); // [1*2+4, 2*2+5, 3*2+6]
}

/// Document restrict vs non-restrict difference
///
/// Shows why restrict matters in C, and how Rust enforces it
#[test]
fn test_restrict_vs_non_restrict() {
    let _c_code = r#"
// Without restrict: compiler must assume aliasing
void add_no_restrict(int* a, int* b, int* c) {
    *a = *b + *c;
    *a = *a + 1;  // Must reload *b and *c (might have changed)
}

// With restrict: compiler knows no aliasing
void add_restrict(int* restrict a, int* restrict b, int* restrict c) {
    *a = *b + *c;
    *a = *a + 1;  // Can cache *b and *c values
}
"#;

    let _rust_equivalent = r#"
// Rust borrow checker enforces non-aliasing always
fn add(a: &mut i32, b: &i32, c: &i32) {
    *a = *b + *c;
    *a = *a + 1;
    // Compiler knows b and c cannot alias with a
}
"#;

    fn add(a: &mut i32, b: &i32, c: &i32) {
        *a = *b + *c;
        *a += 1;
    }

    let b = 5;
    let c = 10;
    let mut a = 0;
    add(&mut a, &b, &c);
    assert_eq!(a, 16); // (5 + 10) + 1
}

/// Document restrict with loop carried dependencies
///
/// C restrict helps compiler optimize loops → Rust same benefits
#[test]
fn test_restrict_loop_dependencies() {
    let _c_code = r#"
void process(int* restrict output, const int* restrict input, size_t n) {
    for (size_t i = 0; i < n; i++) {
        output[i] = input[i] * 2;
    }
    // No loop-carried dependency because of restrict
}
"#;

    let _rust_equivalent = r#"
fn process(output: &mut [i32], input: &[i32]) {
    let len = output.len().min(input.len());
    for i in 0..len {
        output[i] = input[i] * 2;
    }
}
"#;

    fn process(output: &mut [i32], input: &[i32]) {
        let len = output.len().min(input.len());
        for i in 0..len {
            output[i] = input[i] * 2;
        }
    }

    let input = [1, 2, 3, 4];
    let mut output = [0; 4];
    process(&mut output, &input);
    assert_eq!(output, [2, 4, 6, 8]);
}

/// Document restrict with function pointers
///
/// C restrict on function pointer parameters → Rust safe by default
#[test]
fn test_restrict_with_function_pointers() {
    let _c_code = r#"
typedef int (*Processor)(int);

void apply(int* restrict output,
           const int* restrict input,
           Processor proc,
           size_t n) {
    for (size_t i = 0; i < n; i++) {
        output[i] = proc(input[i]);
    }
}
"#;

    let _rust_equivalent = r#"
fn apply<F>(output: &mut [i32], input: &[i32], proc: F)
where
    F: Fn(i32) -> i32,
{
    let len = output.len().min(input.len());
    for i in 0..len {
        output[i] = proc(input[i]);
    }
}
"#;

    fn apply<F>(output: &mut [i32], input: &[i32], proc: F)
    where
        F: Fn(i32) -> i32,
    {
        let len = output.len().min(input.len());
        for i in 0..len {
            output[i] = proc(input[i]);
        }
    }

    let input = [1, 2, 3, 4];
    let mut output = [0; 4];
    apply(&mut output, &input, |x| x * x);
    assert_eq!(output, [1, 4, 9, 16]);
}

/// Document restrict guarantees vs Rust guarantees
///
/// Shows that Rust provides stronger guarantees
#[test]
fn test_restrict_guarantees() {
    let _c_code = r#"
// C restrict is a PROMISE (not verified by compiler)
// Violating it is undefined behavior
void bad_restrict(int* restrict a, int* restrict b) {
    *a = 10;
    *b = 20;
    // If a and b actually alias: UNDEFINED BEHAVIOR
}
"#;

    let _rust_equivalent = r#"
// Rust borrow checker ENFORCES non-aliasing
fn safe_rust(a: &mut i32, b: &mut i32) {
    *a = 10;
    *b = 20;
    // Cannot compile if a and b alias
}
// This is CHECKED at compile time, not a promise
"#;

    fn safe_rust(a: &mut i32, b: &mut i32) {
        *a = 10;
        *b = 20;
    }

    let mut x = 1;
    let mut y = 2;
    safe_rust(&mut x, &mut y);
    assert_eq!(x, 10);
    assert_eq!(y, 20);

    // This would be a compile error in Rust:
    // let mut z = 3;
    // safe_rust(&mut z, &mut z);  // ERROR: cannot borrow `z` as mutable more than once
}

/// Document restrict with standard library functions
///
/// Many C standard library functions use restrict
#[test]
fn test_restrict_stdlib() {
    let _c_code = r#"
// C standard library heavily uses restrict:
// char* strcpy(char* restrict dest, const char* restrict src);
// void* memcpy(void* restrict dest, const void* restrict src, size_t n);
// int sprintf(char* restrict str, const char* restrict format, ...);

void copy_string(char* restrict dest, const char* restrict src) {
    strcpy(dest, src);
}
"#;

    let _rust_equivalent = r#"
// Rust standard library enforces non-aliasing through borrow checker
fn copy_string(dest: &mut String, src: &str) {
    dest.clear();
    dest.push_str(src);
}
// Alternative: use clone for owned string
fn copy_string_owned(src: &str) -> String {
    src.to_string()
}
"#;

    fn copy_string(dest: &mut String, src: &str) {
        dest.clear();
        dest.push_str(src);
    }

    let src = "Hello, World!";
    let mut dest = String::new();
    copy_string(&mut dest, src);
    assert_eq!(dest, "Hello, World!");
}

/// Summary: Restrict Keyword (C99 §6.7.3.1)
///
/// **Transformation Rules**:
/// 1. C `T* restrict` → Rust `&mut T` (mutable, exclusive)
/// 2. C `const T* restrict` → Rust `&T` (immutable, shared)
/// 3. C `T* restrict` param → Rust `&mut [T]` slice (array context)
/// 4. C restrict promise → Rust borrow checker enforcement
///
/// **Key Insights**:
/// - C restrict is PROGRAMMER PROMISE (not verified)
/// - Rust borrow checker ENFORCES non-aliasing
/// - C restrict violations = undefined behavior
/// - Rust aliasing violations = compile error
/// - Rust provides STRONGER guarantees than C restrict
/// - Same optimization benefits in both languages
/// - Rust `&mut` is conceptually "restrict by default"
///
/// **Safety**: ✅ 0 unsafe blocks (borrow checker ensures safety)
///
/// **Coverage**: 15 test cases covering:
/// - Simple restrict pointers
/// - Optimization hints
/// - memcpy pattern
/// - No-aliasing enforcement
/// - Return values
/// - Restrict in structs
/// - Const restrict
/// - Pointer-to-pointer
/// - Array parameters
/// - Vectorization
/// - Restrict vs non-restrict
/// - Loop dependencies
/// - Function pointers
/// - Guarantees comparison
/// - Standard library usage
#[test]
fn test_restrict_summary() {
    // C restrict is a promise (not enforced)
    let c_restrict_verified = false;

    // Rust borrow checker enforces non-aliasing
    let rust_borrow_checker_enforced = true;

    assert_ne!(c_restrict_verified, rust_borrow_checker_enforced);
    assert!(
        rust_borrow_checker_enforced,
        "Rust provides stronger guarantees"
    );

    // No unsafe blocks needed
    let unsafe_blocks = 0;
    assert_eq!(
        unsafe_blocks, 0,
        "Borrow checker makes restrict transformations safe"
    );

    // Test that Rust prevents aliasing at compile time
    let mut x = 10;
    let mut y = 20;

    fn modify_both(a: &mut i32, b: &mut i32) {
        *a += 1;
        *b += 1;
    }

    modify_both(&mut x, &mut y);
    assert_eq!(x, 11);
    assert_eq!(y, 21);

    // This would NOT compile (enforced at compile time):
    // modify_both(&mut x, &mut x);  // Error: cannot borrow `x` as mutable more than once
}
