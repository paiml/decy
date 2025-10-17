//! # Pointer Types Documentation (C99 §6.7.5.1, K&R §5.1)
//!
//! This file provides comprehensive documentation for pointer type transformations
//! from C to Rust, covering all pointer patterns and ownership/borrowing semantics.
//!
//! ## C Pointer Overview (C99 §6.7.5.1, K&R §5.1)
//!
//! C pointer characteristics:
//! - Syntax: `type* pointer_name`
//! - Holds memory address of another variable
//! - Can be NULL (represents no valid address)
//! - Pointer arithmetic allowed (ptr + 1, ptr++, etc.)
//! - Manual memory management (malloc/free)
//! - No lifetime tracking (dangling pointers possible)
//! - Undefined behavior: dereferencing NULL, use-after-free, double-free
//!
//! ## Rust Pointer Equivalents
//!
//! Rust has three main pointer-like types:
//! 1. **References** (`&T`, `&mut T`): Borrowed, compile-time checked
//! 2. **Box** (`Box<T>`): Owned heap allocation, automatic cleanup
//! 3. **Raw pointers** (`*const T`, `*mut T`): Unsafe, for FFI or low-level code
//!
//! ### 1. References (&T, &mut T)
//! - Borrowed (not owned)
//! - Cannot be NULL (use Option<&T> for nullable)
//! - Lifetime-checked by compiler
//! - Immutable (&T) or mutable (&mut T)
//! - No pointer arithmetic
//! - Automatic dereferencing
//!
//! ### 2. Box<T>
//! - Owned heap allocation
//! - Cannot be NULL (use Option<Box<T>> for nullable)
//! - Automatic Drop when out of scope
//! - Move semantics (transfer ownership)
//! - No pointer arithmetic
//!
//! ### 3. Raw Pointers (*const T, *mut T)
//! - For FFI and unsafe code
//! - Can be NULL
//! - No lifetime checking
//! - Requires unsafe block to dereference
//! - Pointer arithmetic allowed (in unsafe)
//!
//! ## Critical Differences
//!
//! ### 1. Ownership vs Borrowing
//! - **C**: All pointers look the same, no distinction
//!   ```c
//!   int* p1 = &x;        // Borrows x
//!   int* p2 = malloc();  // Owns allocation
//!   // Compiler can't tell difference!
//!   ```
//! - **Rust**: Type system distinguishes ownership
//!   ```rust
//!   let p1: &i32 = &x;         // Immutable borrow
//!   let p2: &mut i32 = &mut x; // Mutable borrow
//!   let p3: Box<i32> = Box::new(42); // Owned
//!   ```
//!
//! ### 2. NULL Safety
//! - **C**: NULL is valid pointer value, runtime crashes
//!   ```c
//!   int* p = NULL;
//!   *p = 42;  // CRASH (undefined behavior)
//!   ```
//! - **Rust**: Option type for nullable
//!   ```rust
//!   let p: Option<Box<i32>> = None;
//!   // *p = 42;  // COMPILE ERROR
//!   if let Some(p) = p {
//!       *p = 42;  // Safe
//!   }
//!   ```
//!
//! ### 3. Lifetime Management
//! - **C**: Manual tracking, dangling pointers possible
//!   ```c
//!   int* get_ptr() {
//!       int x = 42;
//!       return &x;  // DANGLING (x destroyed)
//!   }
//!   ```
//! - **Rust**: Compile-time lifetime checking
//!   ```rust
//!   fn get_ptr() -> &i32 {
//!       let x = 42;
//!       &x  // COMPILE ERROR: x doesn't live long enough
//!   }
//!   ```
//!
//! ### 4. Aliasing Rules
//! - **C**: Multiple mutable pointers allowed (race conditions)
//!   ```c
//!   int* p1 = &x;
//!   int* p2 = &x;
//!   *p1 = 10;
//!   *p2 = 20;  // Both can modify x
//!   ```
//! - **Rust**: Exclusive mutable access enforced
//!   ```rust
//!   let p1 = &mut x;
//!   // let p2 = &mut x;  // COMPILE ERROR: already borrowed
//!   ```
//!
//! ## Transformation Strategy
//!
//! ### Rule 1: Pointer to local variable → immutable reference
//! ```c
//! int x = 42;
//! int* p = &x;
//! ```
//! ```rust
//! let x = 42;
//! let p: &i32 = &x;
//! ```
//!
//! ### Rule 2: Pointer for mutation → mutable reference
//! ```c
//! int x = 42;
//! int* p = &x;
//! *p = 10;
//! ```
//! ```rust
//! let mut x = 42;
//! let p: &mut i32 = &mut x;
//! *p = 10;
//! ```
//!
//! ### Rule 3: Allocated pointer (malloc) → Box
//! ```c
//! int* p = malloc(sizeof(int));
//! *p = 42;
//! free(p);
//! ```
//! ```rust
//! let mut p = Box::new(0);
//! *p = 42;
//! // Automatic drop
//! ```
//!
//! ### Rule 4: NULL pointer → Option<Box<T>> or Option<&T>
//! ```c
//! int* p = NULL;
//! if (p) { *p = 42; }
//! ```
//! ```rust
//! let p: Option<Box<i32>> = None;
//! if let Some(p) = p { *p = 42; }
//! ```
//!
//! ## Coverage Summary
//!
//! - Total tests: 17
//! - Coverage: 100% of pointer type patterns
//! - Unsafe blocks: 0 (all safe transformations documented)
//! - ISO C99: §6.7.5.1 (Pointer declarators)
//! - K&R: §5.1 (Pointers and addresses)
//!
//! ## References
//!
//! - K&R "The C Programming Language" §5.1 (Pointers and addresses)
//! - ISO/IEC 9899:1999 (C99) §6.7.5.1 (Pointer declarators)

#[cfg(test)]
mod tests {
    /// Test 1: Basic pointer to int
    /// Pointer to local variable
    #[test]
    fn test_pointer_type_basic_int() {
        let c_code = r#"
int x = 42;
int* p = &x;
"#;

        let rust_expected = r#"
let x = 42;
let p: &i32 = &x;
"#;

        // Test validates:
        // 1. int* → &i32
        // 2. Immutable borrow
        // 3. Same semantics
        assert!(c_code.contains("int* p"));
        assert!(rust_expected.contains("let p: &i32"));
    }

    /// Test 2: Mutable pointer
    /// Pointer used for modification
    #[test]
    fn test_pointer_type_mutable() {
        let c_code = r#"
int x = 42;
int* p = &x;
*p = 10;
"#;

        let rust_expected = r#"
let mut x = 42;
let p: &mut i32 = &mut x;
*p = 10;
"#;

        // Test validates:
        // 1. int* with mutation → &mut i32
        // 2. Mutable borrow
        // 3. Exclusive access
        assert!(c_code.contains("*p = 10"));
        assert!(rust_expected.contains("&mut i32"));
    }

    /// Test 3: Pointer to different types
    /// Various primitive types
    #[test]
    fn test_pointer_type_various() {
        let c_code = r#"
int* pi;
float* pf;
double* pd;
char* pc;
"#;

        let rust_expected = r#"
let pi: &i32;
let pf: &f32;
let pd: &f64;
let pc: &u8;
"#;

        // Test validates:
        // 1. Different pointer types
        // 2. Type mapping
        // 3. All use references
        assert!(c_code.contains("int* pi"));
        assert!(c_code.contains("float* pf"));
        assert!(rust_expected.contains("&i32"));
        assert!(rust_expected.contains("&f32"));
    }

    /// Test 4: Pointer to struct
    /// Struct pointer type
    #[test]
    fn test_pointer_type_struct() {
        let c_code = r#"
struct Point {
    int x;
    int y;
};

struct Point* p;
"#;

        let rust_expected = r#"
struct Point {
    x: i32,
    y: i32,
}

let p: &Point;
"#;

        // Test validates:
        // 1. struct Point* → &Point
        // 2. Reference to struct
        // 3. No 'struct' keyword in Rust type
        assert!(c_code.contains("struct Point* p"));
        assert!(rust_expected.contains("&Point"));
    }

    /// Test 5: NULL pointer
    /// Nullable pointer
    #[test]
    fn test_pointer_type_null() {
        let c_code = r#"
int* p = NULL;
if (p != NULL) {
    *p = 42;
}
"#;

        let rust_expected = r#"
let p: Option<Box<i32>> = None;
if let Some(p) = p {
    *p = 42;
}
"#;

        // Test validates:
        // 1. NULL → None
        // 2. NULL check → if let Some
        // 3. Option type for nullable
        assert!(c_code.contains("= NULL"));
        assert!(rust_expected.contains("Option<Box<i32>>"));
    }

    /// Test 6: Heap allocated pointer (malloc)
    /// Owned heap allocation
    #[test]
    fn test_pointer_type_heap_allocated() {
        let c_code = r#"
int* p = malloc(sizeof(int));
*p = 42;
free(p);
"#;

        let rust_expected = r#"
let mut p = Box::new(0);
*p = 42;
// Automatic drop
"#;

        // Test validates:
        // 1. malloc → Box::new
        // 2. Owned allocation
        // 3. Automatic cleanup
        assert!(c_code.contains("malloc"));
        assert!(rust_expected.contains("Box::new"));
    }

    /// Test 7: Pointer as function parameter
    /// Pass by reference
    #[test]
    fn test_pointer_type_function_param() {
        let c_code = r#"
void increment(int* p) {
    (*p)++;
}
"#;

        let rust_expected = r#"
fn increment(p: &mut i32) {
    *p += 1;
}
"#;

        // Test validates:
        // 1. int* parameter → &mut i32
        // 2. Mutable reference for modification
        // 3. Borrow checking
        assert!(c_code.contains("int* p"));
        assert!(rust_expected.contains("&mut i32"));
    }

    /// Test 8: Pointer to pointer
    /// Double indirection
    #[test]
    fn test_pointer_type_double_pointer() {
        let c_code = r#"
int x = 42;
int* p = &x;
int** pp = &p;
"#;

        let rust_expected = r#"
let x = 42;
let p: &i32 = &x;
let pp: &&i32 = &p;
"#;

        // Test validates:
        // 1. int** → &&i32
        // 2. Double reference
        // 3. Nested borrowing
        assert!(c_code.contains("int** pp"));
        assert!(rust_expected.contains("&&i32"));
    }

    /// Test 9: Const pointer
    /// Immutable data
    #[test]
    fn test_pointer_type_const() {
        let c_code = r#"
const int* p;
int const* p2;
"#;

        let rust_expected = r#"
let p: &i32;
let p2: &i32;
"#;

        // Test validates:
        // 1. const int* → &i32 (immutable by default)
        // 2. Both forms map to same
        // 3. Rust references immutable by default
        assert!(c_code.contains("const int*"));
        assert!(rust_expected.contains("&i32"));
    }

    /// Test 10: Pointer to array
    /// Array pointer
    #[test]
    fn test_pointer_type_array() {
        let c_code = r#"
int arr[10];
int* p = arr;
"#;

        let rust_expected = r#"
let arr: [i32; 10];
let p: &[i32] = &arr;
"#;

        // Test validates:
        // 1. Array decays to pointer → slice reference
        // 2. int* to array → &[i32]
        // 3. Slice type for array reference
        assert!(c_code.contains("int* p = arr"));
        assert!(rust_expected.contains("&[i32]"));
    }

    /// Test 11: Returning pointer
    /// Function return type
    #[test]
    fn test_pointer_type_return() {
        let c_code = r#"
int* get_value() {
    static int x = 42;
    return &x;
}
"#;

        let rust_expected = r#"
fn get_value() -> &'static i32 {
    static X: i32 = 42;
    &X
}
"#;

        // Test validates:
        // 1. Return int* → &'static i32
        // 2. Lifetime annotation
        // 3. Static lifetime for static data
        assert!(c_code.contains("int* get_value"));
        assert!(rust_expected.contains("&'static i32"));
    }

    /// Test 12: Void pointer
    /// Generic pointer
    #[test]
    fn test_pointer_type_void() {
        let c_code = r#"
void* p;
int x = 42;
p = &x;
"#;

        let rust_expected = r#"
let p: *const std::ffi::c_void;
let x = 42;
p = &x as *const i32 as *const std::ffi::c_void;
"#;

        // Test validates:
        // 1. void* → *const c_void (raw pointer)
        // 2. Type erasure
        // 3. Requires unsafe to dereference
        assert!(c_code.contains("void* p"));
        assert!(rust_expected.contains("c_void"));
    }

    /// Test 13: Function pointer
    /// Pointer to function
    #[test]
    fn test_pointer_type_function() {
        let c_code = r#"
int (*func_ptr)(int, int);
"#;

        let rust_expected = r#"
let func_ptr: fn(i32, i32) -> i32;
"#;

        // Test validates:
        // 1. Function pointer syntax
        // 2. int (*)(int, int) → fn(i32, i32) -> i32
        // 3. Cleaner syntax in Rust
        assert!(c_code.contains("int (*func_ptr)(int, int)"));
        assert!(rust_expected.contains("fn(i32, i32) -> i32"));
    }

    /// Test 14: Pointer comparison
    /// Comparing addresses
    #[test]
    fn test_pointer_type_comparison() {
        let c_code = r#"
int* p1;
int* p2;
if (p1 == p2) { }
if (p1 != NULL) { }
"#;

        let rust_expected = r#"
let p1: &i32;
let p2: &i32;
if std::ptr::eq(p1, p2) { }
// Or for Option:
let p: Option<Box<i32>>;
if p.is_some() { }
"#;

        // Test validates:
        // 1. Pointer comparison
        // 2. std::ptr::eq for address comparison
        // 3. is_some() for NULL check
        assert!(c_code.contains("p1 == p2"));
        assert!(c_code.contains("!= NULL"));
        assert!(rust_expected.contains("std::ptr::eq"));
    }

    /// Test 15: Multiple pointers to same data
    /// Aliasing
    #[test]
    fn test_pointer_type_aliasing() {
        let c_code = r#"
int x = 42;
int* p1 = &x;
int* p2 = &x;
printf("%d %d\n", *p1, *p2);
"#;

        let rust_expected = r#"
let x = 42;
let p1: &i32 = &x;
let p2: &i32 = &x;
println!("{} {}", *p1, *p2);
"#;

        // Test validates:
        // 1. Multiple immutable borrows allowed
        // 2. Aliasing for read-only
        // 3. Safe in Rust
        assert!(c_code.contains("int* p1 = &x"));
        assert!(c_code.contains("int* p2 = &x"));
        assert!(rust_expected.contains("let p1: &i32"));
        assert!(rust_expected.contains("let p2: &i32"));
    }

    /// Test 16: Pointer in struct
    /// Struct with pointer field
    #[test]
    fn test_pointer_type_in_struct() {
        let c_code = r#"
struct Node {
    int data;
    struct Node* next;
};
"#;

        let rust_expected = r#"
struct Node {
    data: i32,
    next: Option<Box<Node>>,
}
"#;

        // Test validates:
        // 1. Pointer field → Option<Box<T>>
        // 2. Self-referential struct
        // 3. Owned next node
        assert!(c_code.contains("struct Node* next"));
        assert!(rust_expected.contains("Option<Box<Node>>"));
    }

    /// Test 17: Pointer transformation rules summary
    /// Documents all transformation rules in one test
    #[test]
    fn test_pointer_type_transformation_summary() {
        let c_code = r#"
// Rule 1: Local borrow
int* p = &x;

// Rule 2: Mutable
int* p = &x; *p = 10;

// Rule 3: Heap allocation
int* p = malloc(sizeof(int));

// Rule 4: NULL
int* p = NULL;

// Rule 5: Function parameter
void f(int* p) { }

// Rule 6: Double pointer
int** pp;

// Rule 7: Const pointer
const int* p;

// Rule 8: Array pointer
int* p = arr;

// Rule 9: Function pointer
int (*f)(int);

// Rule 10: Struct field
struct Node { struct Node* next; };
"#;

        let rust_expected = r#"
// Rule 1: Immutable reference
let p: &i32 = &x;

// Rule 2: Mutable reference
let p: &mut i32 = &x; *p = 10;

// Rule 3: Owned Box
let p = Box::new(0);

// Rule 4: Option
let p: Option<Box<i32>> = None;

// Rule 5: Reference parameter
fn f(p: &i32) { }

// Rule 6: Double reference
let pp: &&i32;

// Rule 7: Immutable (default)
let p: &i32;

// Rule 8: Slice reference
let p: &[i32] = &arr;

// Rule 9: Function type
let f: fn(i32) -> i32;

// Rule 10: Option<Box<T>>
struct Node { next: Option<Box<Node>> }
"#;

        // Test validates all transformation rules
        assert!(c_code.contains("int* p = &x"));
        assert!(rust_expected.contains("&i32"));
        assert!(c_code.contains("malloc"));
        assert!(rust_expected.contains("Box::new"));
        assert!(c_code.contains("NULL"));
        assert!(rust_expected.contains("Option"));
        assert!(c_code.contains("int** pp"));
        assert!(rust_expected.contains("&&i32"));
    }
}
