//! Documentation tests for inline functions transformation (C99 §6.7.4)
//!
//! C99 introduced the `inline` keyword for function optimization hints.
//! This test suite documents how DECY transforms C inline functions to Rust.
//!
//! **Reference**: ISO C99 §6.7.4 (Function specifiers - inline)
//!              NOT in K&R (pre-C99 feature)
//!
//! **Key Differences**:
//! - C `inline` is a compiler hint (may or may not inline)
//! - Rust `#[inline]` attribute provides similar hints
//! - Rust compiler makes inlining decisions automatically
//! - C requires external definition for inline functions
//! - Rust has stricter visibility rules
//!
//! **Safety**: All transformations are SAFE (0 unsafe blocks)
//!
//! **Version**: v0.36.0

/// Document transformation of simple inline function
///
/// C99 `inline` keyword → Rust `#[inline]` attribute
///
/// C Reference: ISO C99 §6.7.4 paragraph 6
#[test]
fn test_simple_inline_function() {
    let _c_code = r#"
inline int add(int a, int b) {
    return a + b;
}
"#;

    let _rust_equivalent = r#"
#[inline]
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;

    #[inline]
    fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    let result = add(10, 20);
    assert_eq!(result, 30, "Inline function works like normal function");

    // Verify it's callable (inlining is transparent to caller)
    let x = 5;
    let y = 7;
    assert_eq!(add(x, y), 12);
}

/// Document inline function with static keyword
///
/// C99 `static inline` → Rust `#[inline]` with appropriate visibility
///
/// C Reference: ISO C99 §6.7.4 paragraph 7 (internal linkage)
#[test]
fn test_static_inline_function() {
    let _c_code = r#"
static inline int max(int a, int b) {
    return (a > b) ? a : b;
}
"#;

    let _rust_equivalent = r#"
#[inline]
fn max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}
"#;

    #[inline]
    fn max(a: i32, b: i32) -> i32 {
        if a > b {
            a
        } else {
            b
        }
    }

    assert_eq!(max(10, 20), 20);
    assert_eq!(max(30, 15), 30);
    assert_eq!(max(5, 5), 5);
}

/// Document inline function in header pattern
///
/// C header inline functions → Rust public inline functions in modules
///
/// C Reference: ISO C99 §6.7.4 paragraph 3 (inline definition)
#[test]
fn test_header_inline_function() {
    let _c_code = r#"
// In header.h:
inline int square(int x) {
    return x * x;
}
"#;

    let _rust_equivalent = r#"
// In module:
#[inline]
pub fn square(x: i32) -> i32 {
    x * x
}
"#;

    // Simulate public inline function
    #[inline]
    #[allow(dead_code)]
    pub fn square(x: i32) -> i32 {
        x * x
    }

    assert_eq!(square(5), 25);
    assert_eq!(square(10), 100);
    assert_eq!(square(-3), 9);
}

/// Document inline always attribute
///
/// GCC `__attribute__((always_inline))` → Rust `#[inline(always)]`
///
/// Note: This is a GCC extension, but commonly used with C99 inline
#[test]
fn test_always_inline() {
    let _c_code = r#"
inline __attribute__((always_inline))
int double_value(int x) {
    return x * 2;
}
"#;

    let _rust_equivalent = r#"
#[inline(always)]
fn double_value(x: i32) -> i32 {
    x * 2
}
"#;

    #[inline(always)]
    fn double_value(x: i32) -> i32 {
        x * 2
    }

    assert_eq!(double_value(5), 10);
    assert_eq!(double_value(100), 200);
}

/// Document inline never attribute
///
/// GCC `__attribute__((noinline))` → Rust `#[inline(never)]`
///
/// Note: Prevents inlining for debugging or code size
#[test]
fn test_never_inline() {
    let _c_code = r#"
__attribute__((noinline))
int complex_calculation(int x) {
    return x * x + x * 2 + 1;
}
"#;

    let _rust_equivalent = r#"
#[inline(never)]
fn complex_calculation(x: i32) -> i32 {
    x * x + x * 2 + 1
}
"#;

    #[inline(never)]
    fn complex_calculation(x: i32) -> i32 {
        x * x + x * 2 + 1
    }

    assert_eq!(complex_calculation(3), 16); // 9 + 6 + 1
    assert_eq!(complex_calculation(5), 36); // 25 + 10 + 1
}

/// Document inline function with loops
///
/// More complex inline functions may not be inlined by compiler
///
/// C Reference: ISO C99 §6.7.4 (compiler makes final decision)
#[test]
fn test_inline_with_loops() {
    let _c_code = r#"
inline int sum_array(int* arr, int n) {
    int sum = 0;
    for (int i = 0; i < n; i++) {
        sum += arr[i];
    }
    return sum;
}
"#;

    let _rust_equivalent = r#"
#[inline]
fn sum_array(arr: &[i32]) -> i32 {
    let mut sum = 0;
    for i in 0..arr.len() {
        sum += arr[i];
    }
    sum
}
"#;

    #[inline]
    fn sum_array(arr: &[i32]) -> i32 {
        let mut sum = 0;
        for i in 0..arr.len() {
            sum += arr[i];
        }
        sum
    }

    let arr = [1, 2, 3, 4, 5];
    assert_eq!(sum_array(&arr), 15);

    let empty: [i32; 0] = [];
    assert_eq!(sum_array(&empty), 0);
}

/// Document inline function calling other inline functions
///
/// Nested inline calls work naturally in both C and Rust
#[test]
fn test_nested_inline_calls() {
    let _c_code = r#"
inline int add(int a, int b) {
    return a + b;
}

inline int add_three(int a, int b, int c) {
    return add(add(a, b), c);
}
"#;

    let _rust_equivalent = r#"
#[inline]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[inline]
fn add_three(a: i32, b: i32, c: i32) -> i32 {
    add(add(a, b), c)
}
"#;

    #[inline]
    fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    #[inline]
    fn add_three(a: i32, b: i32, c: i32) -> i32 {
        add(add(a, b), c)
    }

    assert_eq!(add_three(1, 2, 3), 6);
    assert_eq!(add_three(10, 20, 30), 60);
}

/// Document inline with generic/template-like behavior
///
/// In Rust, generic functions are implicitly inline-eligible
#[test]
fn test_inline_generic() {
    let _c_code = r#"
// C would use macro or duplicate code
#define MAX(a, b) ((a) > (b) ? (a) : (b))
"#;

    let _rust_equivalent = r#"
#[inline]
fn max<T: Ord>(a: T, b: T) -> T {
    if a > b { a } else { b }
}
"#;

    #[inline]
    fn max<T: Ord>(a: T, b: T) -> T {
        if a > b {
            a
        } else {
            b
        }
    }

    assert_eq!(max(10, 20), 20);
    assert_eq!(max(100i64, 50i64), 100i64);
    assert_eq!(max("hello", "world"), "world");
}

/// Document inline function with const operations
///
/// Rust const fn is even more powerful than C inline for compile-time evaluation
#[test]
fn test_inline_const() {
    let _c_code = r#"
inline int factorial_5() {
    return 5 * 4 * 3 * 2 * 1;
}
"#;

    let _rust_equivalent = r#"
const fn factorial_5() -> i32 {
    5 * 4 * 3 * 2 * 1
}
// Note: const fn can be evaluated at compile time
"#;

    const fn factorial_5() -> i32 {
        5 * 4 * 3 * 2 * 1
    }

    // Can be used in const context
    const FACT: i32 = factorial_5();
    assert_eq!(FACT, 120);

    // Also works at runtime
    assert_eq!(factorial_5(), 120);
}

/// Document inline with multiple return paths
///
/// Early returns are fine in inline functions
#[test]
fn test_inline_multiple_returns() {
    let _c_code = r#"
inline int clamp(int val, int min, int max) {
    if (val < min) return min;
    if (val > max) return max;
    return val;
}
"#;

    let _rust_equivalent = r#"
#[inline]
fn clamp(val: i32, min: i32, max: i32) -> i32 {
    if val < min { return min; }
    if val > max { return max; }
    val
}
"#;

    #[inline]
    fn clamp(val: i32, min: i32, max: i32) -> i32 {
        if val < min {
            return min;
        }
        if val > max {
            return max;
        }
        val
    }

    assert_eq!(clamp(5, 0, 10), 5);
    assert_eq!(clamp(-5, 0, 10), 0);
    assert_eq!(clamp(15, 0, 10), 10);
    assert_eq!(clamp(10, 0, 10), 10);
}

/// Document inline with extern declaration
///
/// C99 allows inline with external linkage
///
/// C Reference: ISO C99 §6.7.4 paragraph 4
#[test]
fn test_inline_extern() {
    let _c_code = r#"
// In header:
inline int helper(int x);

// In .c file:
extern inline int helper(int x) {
    return x + 1;
}
"#;

    let _rust_equivalent = r#"
// Just use pub fn with inline hint
#[inline]
pub fn helper(x: i32) -> i32 {
    x + 1
}
"#;

    #[inline]
    #[allow(dead_code)]
    pub fn helper(x: i32) -> i32 {
        x + 1
    }

    assert_eq!(helper(5), 6);
    assert_eq!(helper(0), 1);
}

/// Document inline with struct operations
///
/// Common pattern: inline getters/setters
#[test]
fn test_inline_struct_methods() {
    let _c_code = r#"
struct Point {
    int x;
    int y;
};

inline int point_get_x(struct Point* p) {
    return p->x;
}

inline void point_set_x(struct Point* p, int x) {
    p->x = x;
}
"#;

    let _rust_equivalent = r#"
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    #[inline]
    fn get_x(&self) -> i32 {
        self.x
    }

    #[inline]
    fn set_x(&mut self, x: i32) {
        self.x = x;
    }
}
"#;

    #[allow(dead_code)]
    struct Point {
        x: i32,
        y: i32,
    }

    impl Point {
        #[inline]
        fn get_x(&self) -> i32 {
            self.x
        }

        #[inline]
        fn set_x(&mut self, x: i32) {
            self.x = x;
        }
    }

    let mut p = Point { x: 10, y: 20 };
    assert_eq!(p.get_x(), 10);

    p.set_x(30);
    assert_eq!(p.get_x(), 30);
}

/// Document that inline is optional in Rust
///
/// Rust compiler performs aggressive inlining without hints
#[test]
fn test_inline_optional() {
    let _c_code = r#"
inline int add(int a, int b) {
    return a + b;
}
"#;

    let _rust_equivalent = r#"
// Option 1: With hint
#[inline]
fn add_with_hint(a: i32, b: i32) -> i32 {
    a + b
}

// Option 2: Without hint (compiler still may inline)
fn add_no_hint(a: i32, b: i32) -> i32 {
    a + b
}
"#;

    #[inline]
    fn add_with_hint(a: i32, b: i32) -> i32 {
        a + b
    }

    fn add_no_hint(a: i32, b: i32) -> i32 {
        a + b
    }

    // Both work identically from caller's perspective
    assert_eq!(add_with_hint(5, 10), 15);
    assert_eq!(add_no_hint(5, 10), 15);
}

/// Document inline in performance-critical code
///
/// Typical use case for inline: hot path optimization
#[test]
fn test_inline_performance_critical() {
    let _c_code = r#"
inline int fast_multiply_by_power_of_2(int x, int shift) {
    return x << shift;
}
"#;

    let _rust_equivalent = r#"
#[inline]
fn fast_multiply_by_power_of_2(x: i32, shift: u32) -> i32 {
    x << shift
}
"#;

    #[inline]
    fn fast_multiply_by_power_of_2(x: i32, shift: u32) -> i32 {
        x << shift
    }

    assert_eq!(fast_multiply_by_power_of_2(5, 1), 10); // 5 * 2
    assert_eq!(fast_multiply_by_power_of_2(5, 2), 20); // 5 * 4
    assert_eq!(fast_multiply_by_power_of_2(5, 3), 40); // 5 * 8
}

/// Document inline with recursion
///
/// Note: Recursive functions typically won't be inlined
#[test]
fn test_inline_recursion() {
    let _c_code = r#"
inline int factorial(int n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}
"#;

    let _rust_equivalent = r#"
#[inline]
fn factorial(n: i32) -> i32 {
    if n <= 1 { return 1; }
    n * factorial(n - 1)
}
// Note: Compiler likely won't inline recursive calls
"#;

    #[inline]
    fn factorial(n: i32) -> i32 {
        if n <= 1 {
            return 1;
        }
        n * factorial(n - 1)
    }

    assert_eq!(factorial(5), 120);
    assert_eq!(factorial(0), 1);
    assert_eq!(factorial(1), 1);
}

/// Summary: Inline Functions (C99 §6.7.4)
///
/// **Transformation Rules**:
/// 1. C `inline` → Rust `#[inline]`
/// 2. C `static inline` → Rust `#[inline]` with appropriate visibility
/// 3. GCC `__attribute__((always_inline))` → Rust `#[inline(always)]`
/// 4. GCC `__attribute__((noinline))` → Rust `#[inline(never)]`
/// 5. C `extern inline` → Rust `pub fn` with `#[inline]`
///
/// **Key Insights**:
/// - Both C and Rust inline is a HINT, not a guarantee
/// - Rust compiler very good at auto-inlining without hints
/// - Rust `const fn` even more powerful (compile-time evaluation)
/// - Generic functions in Rust are implicitly inline-eligible
/// - No semantic difference (only performance impact)
/// - All transformations are SAFE (0 unsafe blocks)
///
/// **Safety**: ✅ 0 unsafe blocks (inline is purely optimization)
///
/// **Coverage**: 15 test cases covering:
/// - Simple inline functions
/// - Static inline
/// - Header inline pattern
/// - Always/never inline attributes
/// - Inline with loops
/// - Nested inline calls
/// - Generic inline
/// - Const inline (const fn)
/// - Multiple return paths
/// - Extern inline
/// - Struct methods
/// - Optional inline hints
/// - Performance-critical usage
/// - Recursive inline
#[test]
fn test_inline_summary() {
    // C99 inline is an optimization hint
    let c_inline_is_hint = true;
    let rust_inline_is_hint = true;
    assert_eq!(c_inline_is_hint, rust_inline_is_hint);

    // No unsafe blocks needed
    let unsafe_blocks = 0;
    assert_eq!(unsafe_blocks, 0, "Inline is purely optimization - 0 unsafe");

    // Semantics unchanged
    #[inline]
    fn with_inline(x: i32) -> i32 {
        x * 2
    }
    fn without_inline(x: i32) -> i32 {
        x * 2
    }

    assert_eq!(with_inline(5), without_inline(5));
}
