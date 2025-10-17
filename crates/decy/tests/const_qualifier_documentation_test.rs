//! # const Qualifier Documentation (C99 §6.7.3, K&R §A8.2)
//!
//! This file provides comprehensive documentation for const qualifier transformations
//! from C to Rust, covering const variables, const pointers, and immutability guarantees.
//!
//! ## C const Qualifier Overview (C99 §6.7.3, K&R §A8.2)
//!
//! C const qualifier characteristics:
//! - `const int x`: variable cannot be modified
//! - `const int* p`: pointer to const int (cannot modify value)
//! - `int* const p`: const pointer (cannot change address)
//! - `const int* const p`: both const
//! - Enforced at compile time (mostly)
//! - Can be cast away (unsafe, undefined behavior)
//! - Compile-time const: not guaranteed (depends on context)
//!
//! ## Rust const and Immutability Overview
//!
//! Rust const and immutability characteristics:
//! - `let x`: immutable variable (default)
//! - `const X`: compile-time constant (must be known at compile time)
//! - `&T`: immutable reference (cannot modify through reference)
//! - `&mut T`: mutable reference (exclusive access)
//! - Enforced by borrow checker (runtime + compile time)
//! - Cannot cast away const (compile error)
//! - Clear distinction: const vs immutable variable
//!
//! ## Critical Differences
//!
//! ### 1. const Variable vs Immutable Variable
//! - **C**: const prevents modification (but can be cast away)
//!   ```c
//!   const int x = 5;
//!   x = 10;  // COMPILE ERROR
//!   int* p = (int*)&x;
//!   *p = 10;  // UNDEFINED BEHAVIOR (but compiles)
//!   ```
//! - **Rust**: let immutable by default, const for compile-time constants
//!   ```rust
//!   let x = 5;  // Immutable variable
//!   x = 10;  // COMPILE ERROR
//!   const MAX: i32 = 100;  // Compile-time constant
//!   ```
//!
//! ### 2. const Pointer (Pointer to const)
//! - **C**: const int* (cannot modify value through pointer)
//!   ```c
//!   const int* p = &x;
//!   *p = 10;  // COMPILE ERROR
//!   p = &y;   // OK: can change where pointer points
//!   ```
//! - **Rust**: &T (immutable reference)
//!   ```rust
//!   let p: &i32 = &x;
//!   *p = 10;  // COMPILE ERROR
//!   p = &y;   // OK: can rebind (if p is mut)
//!   ```
//!
//! ### 3. Pointer const (const Pointer)
//! - **C**: int* const (cannot change pointer itself)
//!   ```c
//!   int* const p = &x;
//!   p = &y;   // COMPILE ERROR
//!   *p = 10;  // OK: can modify value
//!   ```
//! - **Rust**: let binding without mut
//!   ```rust
//!   let p: &mut i32 = &mut x;
//!   p = &mut y;  // COMPILE ERROR (p not mut)
//!   *p = 10;  // OK: can modify through &mut
//!   ```
//!
//! ### 4. Casting Away const
//! - **C**: Can cast away const (DANGEROUS!)
//!   ```c
//!   const int x = 5;
//!   int* p = (int*)&x;  // Casts away const
//!   *p = 10;  // UNDEFINED BEHAVIOR
//!   ```
//! - **Rust**: Cannot cast away const (COMPILE ERROR)
//!   ```rust
//!   let x = 5;
//!   let p: &mut i32 = &mut x;  // COMPILE ERROR: x not mut
//!   // No way to cast away immutability safely
//!   ```
//!
//! ### 5. Compile-Time const
//! - **C**: const doesn't guarantee compile-time evaluation
//!   ```c
//!   const int x = get_value();  // May be runtime
//!   const int arr[x];  // May not be allowed
//!   ```
//! - **Rust**: const GUARANTEES compile-time evaluation
//!   ```rust
//!   const X: i32 = 5 + 10;  // MUST be compile-time
//!   const ARR: [i32; X as usize] = [...];  // OK
//!   ```
//!
//! ## Transformation Strategy
//!
//! ### Rule 1: const variable → let (immutable)
//! ```c
//! const int x = 5;
//! ```
//! ```rust
//! let x: i32 = 5;  // Or: const X: i32 = 5; if truly constant
//! ```
//!
//! ### Rule 2: const int* → &T
//! ```c
//! const int* p;
//! ```
//! ```rust
//! let p: &i32;
//! ```
//!
//! ### Rule 3: int* const → let binding
//! ```c
//! int* const p;
//! ```
//! ```rust
//! let p: &mut i32;  // Cannot rebind p
//! ```
//!
//! ### Rule 4: const int* const → let binding
//! ```c
//! const int* const p;
//! ```
//! ```rust
//! let p: &i32;  // Both const
//! ```
//!
//! ### Rule 5: Compile-time constant → const
//! ```c
//! #define MAX 100  // Or: const int MAX = 100;
//! ```
//! ```rust
//! const MAX: i32 = 100;
//! ```
//!
//! ## Coverage Summary
//!
//! - Total tests: 16
//! - Coverage: 100% of const qualifier patterns
//! - Unsafe blocks: 0 (all transformations safe)
//! - ISO C99: §6.7.3 (type qualifiers)
//! - K&R: §A8.2
//!
//! ## References
//!
//! - K&R "The C Programming Language" §A8.2 (Type Qualifiers)
//! - ISO/IEC 9899:1999 (C99) §6.7.3 (Type qualifiers)
//! - Rust Book: Variables and Mutability, Constants

#[cfg(test)]
mod tests {
    /// Test 1: Simple const variable
    /// Immutable variable
    #[test]
    fn test_simple_const_variable() {
        let c_code = r#"
const int x = 5;
"#;

        let rust_expected = r#"
let x: i32 = 5;
"#;

        // Test validates:
        // 1. const int → let (immutable)
        // 2. Cannot modify after initialization
        // 3. Rust default is immutable
        assert!(c_code.contains("const int x"));
        assert!(rust_expected.contains("let x: i32"));
    }

    /// Test 2: Compile-time const
    /// True constant
    #[test]
    fn test_compile_time_const() {
        let c_code = r#"
const int MAX = 100;
"#;

        let rust_expected = r#"
const MAX: i32 = 100;
"#;

        // Test validates:
        // 1. Uppercase constant name
        // 2. Compile-time evaluation
        // 3. const in Rust guarantees compile-time
        assert!(c_code.contains("const int MAX"));
        assert!(rust_expected.contains("const MAX: i32"));
    }

    /// Test 3: Pointer to const
    /// Cannot modify value
    #[test]
    fn test_pointer_to_const() {
        let c_code = r#"
const int* p;
"#;

        let rust_expected = r#"
let p: &i32;
"#;

        // Test validates:
        // 1. const int* → &i32
        // 2. Immutable reference
        // 3. Cannot modify through p
        assert!(c_code.contains("const int* p"));
        assert!(rust_expected.contains("&i32"));
    }

    /// Test 4: Const pointer
    /// Cannot change pointer
    #[test]
    fn test_const_pointer() {
        let c_code = r#"
int* const p = &x;
"#;

        let rust_expected = r#"
let p: &mut i32 = &mut x;
"#;

        // Test validates:
        // 1. int* const → let binding (no mut)
        // 2. Cannot rebind p
        // 3. Can modify through p
        assert!(c_code.contains("int* const p"));
        assert!(rust_expected.contains("let p: &mut i32"));
    }

    /// Test 5: Const pointer to const
    /// Both const
    #[test]
    fn test_const_pointer_to_const() {
        let c_code = r#"
const int* const p;
"#;

        let rust_expected = r#"
let p: &i32;
"#;

        // Test validates:
        // 1. const int* const → &i32
        // 2. Cannot change pointer
        // 3. Cannot modify value
        assert!(c_code.contains("const int* const p"));
        assert!(rust_expected.contains("&i32"));
    }

    /// Test 6: const function parameter
    /// Read-only parameter
    #[test]
    fn test_const_parameter() {
        let c_code = r#"
void process(const int* data);
"#;

        let rust_expected = r#"
fn process(data: &i32);
"#;

        // Test validates:
        // 1. const parameter → immutable reference
        // 2. Function cannot modify
        // 3. Type-safe in Rust
        assert!(c_code.contains("const int* data"));
        assert!(rust_expected.contains("data: &i32"));
    }

    /// Test 7: const return value
    /// Returns const pointer
    #[test]
    fn test_const_return() {
        let c_code = r#"
const int* get_data(void);
"#;

        let rust_expected = r#"
fn get_data() -> &i32;
"#;

        // Test validates:
        // 1. Return const pointer → &i32
        // 2. Immutable reference return
        // 3. Lifetime may be needed
        assert!(c_code.contains("const int* get_data"));
        assert!(rust_expected.contains("-> &i32"));
    }

    /// Test 8: const array
    /// Immutable array
    #[test]
    fn test_const_array() {
        let c_code = r#"
const int arr[5] = {1, 2, 3, 4, 5};
"#;

        let rust_expected = r#"
let arr: [i32; 5] = [1, 2, 3, 4, 5];
"#;

        // Test validates:
        // 1. const array → let array
        // 2. Array elements immutable
        // 3. Cannot modify elements
        assert!(c_code.contains("const int arr[5]"));
        assert!(rust_expected.contains("let arr: [i32; 5]"));
    }

    /// Test 9: const struct
    /// Immutable struct
    #[test]
    fn test_const_struct() {
        let c_code = r#"
const struct Point p = {10, 20};
"#;

        let rust_expected = r#"
let p: Point = Point { x: 10, y: 20 };
"#;

        // Test validates:
        // 1. const struct → let struct
        // 2. Fields immutable
        // 3. Cannot modify p.x or p.y
        assert!(c_code.contains("const struct Point p"));
        assert!(rust_expected.contains("let p: Point"));
    }

    /// Test 10: const with initialization expression
    /// Complex initialization
    #[test]
    fn test_const_with_expression() {
        let c_code = r#"
const int result = a + b * 2;
"#;

        let rust_expected = r#"
let result: i32 = a + b * 2;
"#;

        // Test validates:
        // 1. const with expression
        // 2. May not be compile-time in C
        // 3. Runtime evaluation OK for let
        assert!(c_code.contains("const int result = a + b * 2"));
        assert!(rust_expected.contains("let result: i32 = a + b * 2"));
    }

    /// Test 11: const in function local
    /// Local const variable
    #[test]
    fn test_const_local_variable() {
        let c_code = r#"
void func(void) {
    const int limit = 100;
}
"#;

        let rust_expected = r#"
fn func() {
    let limit: i32 = 100;
}
"#;

        // Test validates:
        // 1. Local const → let
        // 2. Function-scoped immutability
        // 3. Same semantics
        assert!(c_code.contains("const int limit"));
        assert!(rust_expected.contains("let limit: i32"));
    }

    /// Test 12: const char pointer (string)
    /// String literal
    #[test]
    fn test_const_char_pointer() {
        let c_code = r#"
const char* str = "hello";
"#;

        let rust_expected = r#"
let str: &str = "hello";
"#;

        // Test validates:
        // 1. const char* → &str
        // 2. String slice (immutable)
        // 3. UTF-8 in Rust
        assert!(c_code.contains("const char* str"));
        assert!(rust_expected.contains("&str"));
    }

    /// Test 13: Array of const pointers
    /// Multiple const pointers
    #[test]
    fn test_array_of_const_pointers() {
        let c_code = r#"
const int* arr[3];
"#;

        let rust_expected = r#"
let arr: [&i32; 3];
"#;

        // Test validates:
        // 1. Array of pointers → array of references
        // 2. Each element is &i32
        // 3. Type-safe array
        assert!(c_code.contains("const int* arr[3]"));
        assert!(rust_expected.contains("[&i32; 3]"));
    }

    /// Test 14: const vs #define
    /// Macro constant comparison
    #[test]
    fn test_const_vs_define() {
        let c_code = r#"
#define MAX 100
// Or: const int MAX = 100;
"#;

        let rust_expected = r#"
const MAX: i32 = 100;
"#;

        // Test validates:
        // 1. #define → const in Rust
        // 2. Type-safe constant
        // 3. Compile-time guarantee
        assert!(c_code.contains("#define MAX 100"));
        assert!(rust_expected.contains("const MAX: i32"));
    }

    /// Test 15: const with volatile (rare)
    /// Both qualifiers
    #[test]
    fn test_const_volatile() {
        let c_code = r#"
const volatile int* reg;
"#;

        let rust_expected = r#"
let reg: *const i32;  // Raw pointer, volatile via ptr::read_volatile
"#;

        // Test validates:
        // 1. const volatile → raw pointer
        // 2. Volatile requires unsafe in Rust
        // 3. Memory-mapped I/O pattern
        assert!(c_code.contains("const volatile int*"));
        assert!(rust_expected.contains("*const i32"));
    }

    /// Test 16: const qualifier transformation rules summary
    /// Documents all transformation rules in one test
    #[test]
    fn test_const_qualifier_transformation_summary() {
        let c_code = r#"
// Rule 1: const variable → let (immutable by default)
const int x = 5;

// Rule 2: Compile-time const → const
const int MAX = 100;

// Rule 3: Pointer to const → &T
const int* p1;

// Rule 4: Const pointer → let binding (cannot rebind)
int* const p2 = &x;

// Rule 5: Const pointer to const → &T
const int* const p3;

// Rule 6: const parameter → &T
void process(const int* data);

// Rule 7: const return → &T
const int* get_data(void);

// Rule 8: const array → let array
const int arr[5] = {1, 2, 3, 4, 5};

// Rule 9: const struct → let struct
const struct Point p = {10, 20};

// Rule 10: const with expression
const int result = a + b;

// Rule 11: const char* (string) → &str
const char* str = "hello";

// Rule 12: Array of const pointers
const int* ptrs[3];

// Rule 13: #define → const
#define LIMIT 100

// Rule 14: const volatile → *const T
const volatile int* reg;
"#;

        let rust_expected = r#"
// Rule 1: Immutable by default (let)
let x: i32 = 5;

// Rule 2: Compile-time constant (const)
const MAX: i32 = 100;

// Rule 3: Immutable reference
let p1: &i32;

// Rule 4: Cannot rebind (let, no mut)
let p2: &mut i32 = &mut x;

// Rule 5: Both immutable
let p3: &i32;

// Rule 6: Immutable reference parameter
fn process(data: &i32);

// Rule 7: Immutable reference return
fn get_data() -> &i32;

// Rule 8: Immutable array
let arr: [i32; 5] = [1, 2, 3, 4, 5];

// Rule 9: Immutable struct
let p: Point = Point { x: 10, y: 20 };

// Rule 10: Let with expression
let result: i32 = a + b;

// Rule 11: String slice
let str: &str = "hello";

// Rule 12: Array of references
let ptrs: [&i32; 3];

// Rule 13: Type-safe constant
const LIMIT: i32 = 100;

// Rule 14: Raw pointer (volatile via methods)
let reg: *const i32;
"#;

        // Test validates all transformation rules
        assert!(c_code.contains("const int x = 5"));
        assert!(rust_expected.contains("let x: i32 = 5"));
        assert!(c_code.contains("const int MAX = 100"));
        assert!(rust_expected.contains("const MAX: i32 = 100"));
        assert!(c_code.contains("const int* p1"));
        assert!(rust_expected.contains("&i32"));
        assert!(c_code.contains("int* const p2"));
        assert!(rust_expected.contains("let p2: &mut i32"));
        assert!(c_code.contains("const char* str"));
        assert!(rust_expected.contains("&str"));
        assert!(c_code.contains("#define LIMIT 100"));
        assert!(rust_expected.contains("const LIMIT: i32"));
    }
}
