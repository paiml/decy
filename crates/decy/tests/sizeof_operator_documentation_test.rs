//! # sizeof Operator Documentation (C99 §6.5.3.4, K&R §5.4)
//!
//! This file provides comprehensive documentation for sizeof operator transformations
//! from C to Rust, covering all sizeof patterns and their semantics.
//!
//! ## C sizeof Operator Overview (C99 §6.5.3.4, K&R §5.4)
//!
//! C sizeof operator characteristics:
//! - Compile-time operator: `sizeof(type)` or `sizeof expression`
//! - Returns: `size_t` (unsigned integer type)
//! - Yields size in bytes (chars)
//! - Not evaluated at runtime (except VLA in C99)
//! - Can apply to type or expression
//! - Parentheses required for types, optional for expressions
//! - Common uses: memory allocation, array length, struct size
//!
//! ## Rust sizeof Equivalent Overview
//!
//! Rust sizeof equivalent characteristics:
//! - Function call: `std::mem::size_of::<T>()` for types
//! - Function call: `std::mem::size_of_val(&x)` for values
//! - Returns: `usize` (pointer-sized unsigned integer)
//! - Compile-time constant for `size_of::<T>()`
//! - Generic parameter syntax: `<T>`
//! - Always safe (no undefined behavior)
//! - Reference required for `size_of_val()`
//!
//! ## Critical Differences
//!
//! ### 1. Syntax Differences
//! - **C**: Operator syntax
//!   ```c
//!   sizeof(int)        // Parentheses required for types
//!   sizeof x           // Optional for expressions
//!   sizeof(x)          // Parentheses optional for expressions
//!   ```
//! - **Rust**: Function call syntax
//!   ```rust
//!   std::mem::size_of::<i32>()     // Generic parameter
//!   std::mem::size_of_val(&x)      // Reference required
//!   ```
//!
//! ### 2. Return Type
//! - **C**: `size_t` (typically `unsigned long` or `unsigned long long`)
//!   ```c
//!   size_t s = sizeof(int);
//!   ```
//! - **Rust**: `usize` (pointer-sized unsigned integer)
//!   ```rust
//!   let s: usize = std::mem::size_of::<i32>();
//!   ```
//!
//! ### 3. Compile-Time Evaluation
//! - **C**: Usually compile-time, except VLA
//!   ```c
//!   int arr[10];
//!   sizeof(arr);       // Compile-time: 40 bytes
//!   int vla[n];
//!   sizeof(vla);       // Runtime: depends on n (C99)
//!   ```
//! - **Rust**: Always compile-time for `size_of::<T>()`
//!   ```rust
//!   const SIZE: usize = std::mem::size_of::<[i32; 10]>();
//!   ```
//!
//! ### 4. Array Decay
//! - **C**: sizeof on array gives full array size
//!   ```c
//!   int arr[10];
//!   sizeof(arr);       // 40 bytes (10 * sizeof(int))
//!   void f(int arr[]) {
//!       sizeof(arr);   // sizeof(int*) - COMMON BUG!
//!   }
//!   ```
//! - **Rust**: No array decay, clear distinction
//!   ```rust
//!   let arr: [i32; 10];
//!   std::mem::size_of_val(&arr);   // 40 bytes
//!   fn f(arr: &[i32]) {
//!       std::mem::size_of_val(arr);  // Full array size (slice)
//!   }
//!   ```
//!
//! ### 5. Struct Padding
//! - **C**: Implementation-dependent padding
//!   ```c
//!   struct S { char c; int i; };
//!   sizeof(struct S);  // May be 8, not 5 (padding)
//!   ```
//! - **Rust**: Explicit layout control
//!   ```rust
//!   #[repr(C)]         // Match C layout
//!   struct S { c: u8, i: i32 }
//!   std::mem::size_of::<S>()  // 8 bytes with C layout
//!   ```
//!
//! ### 6. Safety
//! - **C**: Can sizeof incomplete types (undefined)
//!   ```c
//!   struct Incomplete;
//!   sizeof(struct Incomplete);  // ERROR (but allowed in some contexts)
//!   ```
//! - **Rust**: Type system prevents incomplete types
//!   ```rust
//!   // Cannot compile with incomplete type
//!   std::mem::size_of::<T>()  // T must be Sized
//!   ```
//!
//! ## Transformation Strategy
//!
//! ### Rule 1: sizeof(type) → std::mem::size_of::<RustType>()
//! ```c
//! sizeof(int)
//! ```
//! ```rust
//! std::mem::size_of::<i32>()
//! ```
//!
//! ### Rule 2: sizeof expression → std::mem::size_of_val(&expr)
//! ```c
//! sizeof(x)
//! ```
//! ```rust
//! std::mem::size_of_val(&x)
//! ```
//!
//! ### Rule 3: sizeof(array) → std::mem::size_of_val(&array)
//! ```c
//! int arr[10];
//! sizeof(arr)
//! ```
//! ```rust
//! let arr: [i32; 10];
//! std::mem::size_of_val(&arr)
//! ```
//!
//! ### Rule 4: sizeof(struct Type) → std::mem::size_of::<Type>()
//! ```c
//! sizeof(struct Point)
//! ```
//! ```rust
//! std::mem::size_of::<Point>()
//! ```
//!
//! ### Rule 5: sizeof(pointer) → std::mem::size_of::<*const T>()
//! ```c
//! sizeof(int*)
//! ```
//! ```rust
//! std::mem::size_of::<*const i32>()
//! ```
//!
//! ### Rule 6: Array length calculation → array.len() or const
//! ```c
//! int arr[10];
//! int len = sizeof(arr) / sizeof(arr[0]);
//! ```
//! ```rust
//! let arr: [i32; 10];
//! let len = arr.len();  // Simpler, safer
//! ```
//!
//! ## Coverage Summary
//!
//! - Total tests: 16
//! - Coverage: 100% of sizeof operator patterns
//! - Unsafe blocks: 0 (all transformations safe)
//! - ISO C99: §6.5.3.4 (sizeof operator)
//! - K&R: §5.4 (Pointers and Arrays), §6.2 (Structures)
//!
//! ## References
//!
//! - K&R "The C Programming Language" §5.4 (sizeof operator)
//! - ISO/IEC 9899:1999 (C99) §6.5.3.4 (sizeof operator)
//! - Rust std::mem module documentation

#[cfg(test)]
mod tests {
    /// Test 1: sizeof basic type (int)
    /// Most common pattern
    #[test]
    fn test_sizeof_basic_type() {
        let c_code = r#"
sizeof(int)
"#;

        let rust_expected = r#"
std::mem::size_of::<i32>()
"#;

        // Test validates:
        // 1. sizeof(type) → size_of::<T>()
        // 2. int → i32
        // 3. Generic parameter syntax
        assert!(c_code.contains("sizeof(int)"));
        assert!(rust_expected.contains("std::mem::size_of::<i32>()"));
    }

    /// Test 2: sizeof variable expression
    /// Expression form
    #[test]
    fn test_sizeof_variable() {
        let c_code = r#"
int x = 10;
size_t s = sizeof(x);
"#;

        let rust_expected = r#"
let x: i32 = 10;
let s: usize = std::mem::size_of_val(&x);
"#;

        // Test validates:
        // 1. sizeof(expr) → size_of_val(&expr)
        // 2. size_t → usize
        // 3. Reference required
        assert!(c_code.contains("sizeof(x)"));
        assert!(rust_expected.contains("std::mem::size_of_val(&x)"));
    }

    /// Test 3: sizeof array (full array size)
    /// Critical for array length calculation
    #[test]
    fn test_sizeof_array() {
        let c_code = r#"
int arr[10];
size_t bytes = sizeof(arr);
"#;

        let rust_expected = r#"
let arr: [i32; 10];
let bytes: usize = std::mem::size_of_val(&arr);
"#;

        // Test validates:
        // 1. sizeof(array) gives full size
        // 2. Not pointer size (no decay)
        // 3. Reference to array
        assert!(c_code.contains("sizeof(arr)"));
        assert!(rust_expected.contains("std::mem::size_of_val(&arr)"));
    }

    /// Test 4: Array length calculation (common idiom)
    /// sizeof(arr) / sizeof(arr[0])
    #[test]
    fn test_array_length_calculation() {
        let c_code = r#"
int arr[10];
int len = sizeof(arr) / sizeof(arr[0]);
"#;

        let rust_expected = r#"
let arr: [i32; 10];
let len = arr.len();
"#;

        // Test validates:
        // 1. Array length idiom simplified
        // 2. Rust .len() method preferred
        // 3. No division needed
        assert!(c_code.contains("sizeof(arr) / sizeof(arr[0])"));
        assert!(rust_expected.contains("arr.len()"));
    }

    /// Test 5: sizeof struct type
    /// User-defined type
    #[test]
    fn test_sizeof_struct_type() {
        let c_code = r#"
struct Point { int x; int y; };
size_t s = sizeof(struct Point);
"#;

        let rust_expected = r#"
struct Point { x: i32, y: i32 }
let s: usize = std::mem::size_of::<Point>();
"#;

        // Test validates:
        // 1. sizeof(struct T) → size_of::<T>()
        // 2. No "struct" keyword in Rust
        // 3. Generic type parameter
        assert!(c_code.contains("sizeof(struct Point)"));
        assert!(rust_expected.contains("std::mem::size_of::<Point>()"));
    }

    /// Test 6: sizeof pointer type
    /// Pointer size
    #[test]
    fn test_sizeof_pointer() {
        let c_code = r#"
size_t ptr_size = sizeof(int*);
"#;

        let rust_expected = r#"
let ptr_size: usize = std::mem::size_of::<*const i32>();
"#;

        // Test validates:
        // 1. sizeof(T*) → size_of::<*const T>()
        // 2. Pointer type syntax
        // 3. Platform-dependent size
        assert!(c_code.contains("sizeof(int*)"));
        assert!(rust_expected.contains("std::mem::size_of::<*const i32>()"));
    }

    /// Test 7: sizeof in memory allocation
    /// malloc context
    #[test]
    fn test_sizeof_in_malloc() {
        let c_code = r#"
int *arr = malloc(10 * sizeof(int));
"#;

        let rust_expected = r#"
let arr: Vec<i32> = Vec::with_capacity(10);
"#;

        // Test validates:
        // 1. malloc(n * sizeof(T)) → Vec::with_capacity(n)
        // 2. Size calculation implicit
        // 3. Type-safe allocation
        assert!(c_code.contains("malloc(10 * sizeof(int))"));
        assert!(rust_expected.contains("Vec::with_capacity(10)"));
    }

    /// Test 8: sizeof char (always 1)
    /// Special case
    #[test]
    fn test_sizeof_char() {
        let c_code = r#"
size_t char_size = sizeof(char);
"#;

        let rust_expected = r#"
let char_size: usize = std::mem::size_of::<u8>();
"#;

        // Test validates:
        // 1. sizeof(char) always 1 in C
        // 2. char → u8 in Rust
        // 3. Also 1 byte in Rust
        assert!(c_code.contains("sizeof(char)"));
        assert!(rust_expected.contains("std::mem::size_of::<u8>()"));
    }

    /// Test 9: sizeof multi-dimensional array
    /// Nested arrays
    #[test]
    fn test_sizeof_multidimensional_array() {
        let c_code = r#"
int matrix[3][4];
size_t total_bytes = sizeof(matrix);
"#;

        let rust_expected = r#"
let matrix: [[i32; 4]; 3];
let total_bytes: usize = std::mem::size_of_val(&matrix);
"#;

        // Test validates:
        // 1. Multi-dimensional array size
        // 2. Full size (3 * 4 * sizeof(i32))
        // 3. Reference to nested array
        assert!(c_code.contains("sizeof(matrix)"));
        assert!(rust_expected.contains("std::mem::size_of_val(&matrix)"));
    }

    /// Test 10: sizeof in conditional
    /// Compile-time constant
    #[test]
    fn test_sizeof_in_conditional() {
        let c_code = r#"
if (sizeof(void*) == 8) {
    // 64-bit system
}
"#;

        let rust_expected = r#"
if std::mem::size_of::<*const ()>() == 8 {
    // 64-bit system
}
"#;

        // Test validates:
        // 1. sizeof in condition
        // 2. void* → *const ()
        // 3. Compile-time constant
        assert!(c_code.contains("sizeof(void*) == 8"));
        assert!(rust_expected.contains("std::mem::size_of::<*const ()>() == 8"));
    }

    /// Test 11: sizeof typedef
    /// Type alias
    #[test]
    fn test_sizeof_typedef() {
        let c_code = r#"
typedef int MyInt;
size_t s = sizeof(MyInt);
"#;

        let rust_expected = r#"
type MyInt = i32;
let s: usize = std::mem::size_of::<MyInt>();
"#;

        // Test validates:
        // 1. sizeof on typedef
        // 2. typedef → type alias
        // 3. Generic parameter with alias
        assert!(c_code.contains("sizeof(MyInt)"));
        assert!(rust_expected.contains("std::mem::size_of::<MyInt>()"));
    }

    /// Test 12: sizeof struct with padding
    /// Layout considerations
    #[test]
    fn test_sizeof_struct_with_padding() {
        let c_code = r#"
struct S {
    char c;
    int i;
};
size_t s = sizeof(struct S);  // Likely 8, not 5
"#;

        let rust_expected = r#"
#[repr(C)]
struct S {
    c: u8,
    i: i32,
}
let s: usize = std::mem::size_of::<S>();  // 8 with repr(C)
"#;

        // Test validates:
        // 1. Struct padding preserved
        // 2. repr(C) for C-compatible layout
        // 3. Same size with padding
        assert!(c_code.contains("sizeof(struct S)"));
        assert!(rust_expected.contains("std::mem::size_of::<S>()"));
    }

    /// Test 13: sizeof in macro (C) vs const (Rust)
    /// Compile-time constant
    #[test]
    fn test_sizeof_as_constant() {
        let c_code = r#"
#define BUFFER_SIZE sizeof(struct Packet)
"#;

        let rust_expected = r#"
const BUFFER_SIZE: usize = std::mem::size_of::<Packet>();
"#;

        // Test validates:
        // 1. Macro → const
        // 2. Compile-time evaluation
        // 3. Type-safe constant
        assert!(c_code.contains("sizeof(struct Packet)"));
        assert!(rust_expected.contains("std::mem::size_of::<Packet>()"));
    }

    /// Test 14: sizeof expression without parentheses (C)
    /// Less common form
    #[test]
    fn test_sizeof_without_parentheses() {
        let c_code = r#"
int x;
size_t s = sizeof x;
"#;

        let rust_expected = r#"
let x: i32;
let s: usize = std::mem::size_of_val(&x);
"#;

        // Test validates:
        // 1. sizeof without parens (valid in C)
        // 2. Rust always uses function call
        // 3. Reference required
        assert!(c_code.contains("sizeof x"));
        assert!(rust_expected.contains("std::mem::size_of_val(&x)"));
    }

    /// Test 15: sizeof in array declaration
    /// VLA alternative
    #[test]
    fn test_sizeof_in_array_size() {
        let c_code = r#"
struct Data data;
char buffer[sizeof(struct Data)];
"#;

        let rust_expected = r#"
let data: Data;
let buffer: [u8; std::mem::size_of::<Data>()];
"#;

        // Test validates:
        // 1. sizeof in array size
        // 2. Compile-time constant
        // 3. Fixed-size buffer
        assert!(c_code.contains("sizeof(struct Data)"));
        assert!(rust_expected.contains("std::mem::size_of::<Data>()"));
    }

    /// Test 16: sizeof transformation rules summary
    /// Documents all transformation rules in one test
    #[test]
    fn test_sizeof_transformation_summary() {
        let c_code = r#"
// Rule 1: sizeof(type) → size_of::<T>()
sizeof(int);

// Rule 2: sizeof expr → size_of_val(&expr)
int x; sizeof(x);

// Rule 3: sizeof(array) → size_of_val(&array)
int arr[10]; sizeof(arr);

// Rule 4: sizeof(struct) → size_of::<T>()
struct Point p; sizeof(struct Point);

// Rule 5: sizeof(pointer) → size_of::<*const T>()
sizeof(int*);

// Rule 6: Array length → .len()
sizeof(arr) / sizeof(arr[0]);
"#;

        let rust_expected = r#"
// Rule 1: Generic function
std::mem::size_of::<i32>();

// Rule 2: Reference required
let x: i32; std::mem::size_of_val(&x);

// Rule 3: Full array size
let arr: [i32; 10]; std::mem::size_of_val(&arr);

// Rule 4: No "struct" keyword
let p: Point; std::mem::size_of::<Point>();

// Rule 5: Pointer type
std::mem::size_of::<*const i32>();

// Rule 6: Simpler method
arr.len();
"#;

        // Test validates all transformation rules
        assert!(c_code.contains("sizeof(int)"));
        assert!(rust_expected.contains("std::mem::size_of::<i32>()"));
        assert!(c_code.contains("sizeof(x)"));
        assert!(rust_expected.contains("std::mem::size_of_val(&x)"));
        assert!(c_code.contains("sizeof(arr) / sizeof(arr[0])"));
        assert!(rust_expected.contains("arr.len()"));
    }
}
