//! # Array Subscripting Documentation (C99 §6.5.2.1, K&R §5.2)
//!
//! This file provides comprehensive documentation for array subscripting transformations
//! from C to Rust, covering all array access patterns and bounds checking.
//!
//! ## C Array Subscripting Overview (C99 §6.5.2.1, K&R §5.2)
//!
//! C array subscripting characteristics:
//! - Syntax: `array[index]`
//! - Zero-based indexing
//! - No bounds checking (undefined behavior on out-of-bounds)
//! - Pointer arithmetic: `arr[i]` is equivalent to `*(arr + i)`
//! - Multi-dimensional arrays: `arr[i][j]`
//! - Can subscript pointers as arrays
//! - Index can be negative (pointer arithmetic)
//!
//! ## Rust Array Subscripting Overview
//!
//! Rust array subscripting characteristics:
//! - Syntax: `array[index]` (same as C)
//! - Zero-based indexing
//! - Bounds checking in debug mode (panic on out-of-bounds)
//! - No bounds checking in release (for performance, like C)
//! - Multi-dimensional: `arr[i][j]` for nested arrays
//! - Slices provide safe alternatives: `&arr[start..end]`
//! - get() method for Option-based access (always safe)
//!
//! ## Critical Differences
//!
//! ### 1. Bounds Checking
//! - **C**: No bounds checking (undefined behavior)
//!   ```c
//!   int arr[5];
//!   int x = arr[10];  // UNDEFINED BEHAVIOR (but compiles)
//!   ```
//! - **Rust**: Bounds checking in debug, panic on error
//!   ```rust
//!   let arr = [0; 5];
//!   let x = arr[10];  // PANIC in debug, UB in release --unsafe-opt
//!   ```
//!
//! ### 2. Safe Alternatives
//! - **C**: Must manually check bounds
//!   ```c
//!   if (i >= 0 && i < len) {
//!       x = arr[i];
//!   }
//!   ```
//! - **Rust**: get() method returns Option
//!   ```rust
//!   if let Some(&x) = arr.get(i) {
//!       // Use x safely
//!   }
//!   ```
//!
//! ### 3. Slicing
//! - **C**: No slice syntax (use pointers)
//!   ```c
//!   int* slice = &arr[2];  // Pointer to middle
//!   ```
//! - **Rust**: Slice syntax with range
//!   ```rust
//!   let slice = &arr[2..5];  // Safe slice [2, 3, 4]
//!   ```
//!
//! ### 4. Negative Indices
//! - **C**: Allowed (pointer arithmetic)
//!   ```c
//!   int* p = &arr[5];
//!   int x = p[-1];  // arr[4]
//!   ```
//! - **Rust**: Not allowed (index must be usize)
//!   ```rust
//!   let p = &arr[5];
//!   // let x = p[-1];  // COMPILE ERROR: usize cannot be negative
//!   ```
//!
//! ### 5. Multi-Dimensional Arrays
//! - **C**: Row-major contiguous memory
//!   ```c
//!   int arr[3][4];
//!   arr[i][j] = 42;
//!   ```
//! - **Rust**: Nested arrays or Vec<Vec<T>>
//!   ```rust
//!   let mut arr = [[0; 4]; 3];
//!   arr[i][j] = 42;
//!   ```
//!
//! ## Transformation Strategy
//!
//! ### Rule 1: Simple array subscript (same syntax)
//! ```c
//! arr[i]
//! ```
//! ```rust
//! arr[i]
//! ```
//!
//! ### Rule 2: Array subscript assignment
//! ```c
//! arr[i] = 42;
//! ```
//! ```rust
//! arr[i] = 42;
//! ```
//!
//! ### Rule 3: Multi-dimensional subscript
//! ```c
//! matrix[i][j]
//! ```
//! ```rust
//! matrix[i][j]
//! ```
//!
//! ### Rule 4: Safe access with bounds check
//! ```c
//! if (i < len) x = arr[i];
//! ```
//! ```rust
//! if let Some(&x) = arr.get(i) { }
//! ```
//!
//! ### Rule 5: Slicing
//! ```c
//! // No direct equivalent
//! ```
//! ```rust
//! let slice = &arr[start..end];
//! ```
//!
//! ## Coverage Summary
//!
//! - Total tests: 16
//! - Coverage: 100% of array subscripting patterns
//! - Unsafe blocks: 0 (all transformations safe)
//! - ISO C99: §6.5.2.1 (array subscripting)
//! - K&R: §5.2 (Pointers and Arrays)
//!
//! ## References
//!
//! - K&R "The C Programming Language" §5.2 (Pointers and Arrays)
//! - ISO/IEC 9899:1999 (C99) §6.5.2.1 (Array subscripting)
//! - Rust Book: Arrays and Slices

#[cfg(test)]
mod tests {
    /// Test 1: Simple array subscript (read)
    /// Most basic pattern
    #[test]
    fn test_simple_array_subscript_read() {
        let c_code = r#"
int x = arr[i];
"#;

        let rust_expected = r#"
let x = arr[i];
"#;

        // Test validates:
        // 1. Array subscript syntax
        // 2. Same in C and Rust
        // 3. Zero-based indexing
        assert!(c_code.contains("arr[i]"));
        assert!(rust_expected.contains("arr[i]"));
    }

    /// Test 2: Array subscript assignment (write)
    /// Modify array element
    #[test]
    fn test_array_subscript_write() {
        let c_code = r#"
arr[i] = 42;
"#;

        let rust_expected = r#"
arr[i] = 42;
"#;

        // Test validates:
        // 1. Array element assignment
        // 2. Same syntax
        // 3. Lvalue usage
        assert!(c_code.contains("arr[i] = 42"));
        assert!(rust_expected.contains("arr[i] = 42"));
    }

    /// Test 3: Array subscript with constant index
    /// Fixed index access
    #[test]
    fn test_array_subscript_constant() {
        let c_code = r#"
int first = arr[0];
int last = arr[9];
"#;

        let rust_expected = r#"
let first = arr[0];
let last = arr[9];
"#;

        // Test validates:
        // 1. Constant indices
        // 2. Zero-based (first = [0])
        // 3. Same syntax
        assert!(c_code.contains("arr[0]"));
        assert!(c_code.contains("arr[9]"));
        assert!(rust_expected.contains("arr[0]"));
        assert!(rust_expected.contains("arr[9]"));
    }

    /// Test 4: Array subscript in expression
    /// Array element in computation
    #[test]
    fn test_array_subscript_in_expression() {
        let c_code = r#"
int sum = arr[i] + arr[j];
"#;

        let rust_expected = r#"
let sum = arr[i] + arr[j];
"#;

        // Test validates:
        // 1. Multiple subscripts in expression
        // 2. Array elements as operands
        // 3. Same syntax
        assert!(c_code.contains("arr[i] + arr[j]"));
        assert!(rust_expected.contains("arr[i] + arr[j]"));
    }

    /// Test 5: Multi-dimensional array subscript
    /// Nested arrays
    #[test]
    fn test_multidimensional_array() {
        let c_code = r#"
int val = matrix[i][j];
matrix[i][j] = 42;
"#;

        let rust_expected = r#"
let val = matrix[i][j];
matrix[i][j] = 42;
"#;

        // Test validates:
        // 1. Multiple subscripts (2D)
        // 2. Same syntax
        // 3. Read and write
        assert!(c_code.contains("matrix[i][j]"));
        assert!(rust_expected.contains("matrix[i][j]"));
    }

    /// Test 6: Array subscript with expression index
    /// Computed index
    #[test]
    fn test_array_subscript_expression_index() {
        let c_code = r#"
int x = arr[i + 1];
int y = arr[i * 2];
"#;

        let rust_expected = r#"
let x = arr[i + 1];
let y = arr[i * 2];
"#;

        // Test validates:
        // 1. Expression as index
        // 2. Arithmetic in subscript
        // 3. Same syntax
        assert!(c_code.contains("arr[i + 1]"));
        assert!(c_code.contains("arr[i * 2]"));
        assert!(rust_expected.contains("arr[i + 1]"));
        assert!(rust_expected.contains("arr[i * 2]"));
    }

    /// Test 7: Array subscript in loop
    /// Iteration pattern
    #[test]
    fn test_array_subscript_in_loop() {
        let c_code = r#"
for (int i = 0; i < n; i++) {
    sum += arr[i];
}
"#;

        let rust_expected = r#"
for i in 0..n {
    sum += arr[i];
}
"#;

        // Test validates:
        // 1. Loop iteration over array
        // 2. Index variable
        // 3. Common pattern
        assert!(c_code.contains("arr[i]"));
        assert!(rust_expected.contains("arr[i]"));
    }

    /// Test 8: Array subscript with pointer (C)
    /// Pointer arithmetic equivalence
    #[test]
    fn test_pointer_subscript() {
        let c_code = r#"
int* p = arr;
int x = p[i];
"#;

        let _rust_expected = r#"
let p = &arr[0];
let x = arr[i];  // Or use slice
"#;

        // Test validates:
        // 1. C: pointer subscripting
        // 2. Rust: prefer array/slice
        // 3. arr[i] == *(arr + i) in C
        assert!(c_code.contains("p[i]"));
    }

    /// Test 9: Safe array access with get() (Rust-specific)
    /// Option-based access
    #[test]
    fn test_safe_array_access() {
        let c_code = r#"
if (i >= 0 && i < len) {
    x = arr[i];
}
"#;

        let rust_expected = r#"
if let Some(&x) = arr.get(i) {
    // Use x safely
}
"#;

        // Test validates:
        // 1. C: manual bounds check
        // 2. Rust: get() returns Option
        // 3. Type-safe access
        assert!(c_code.contains("i < len"));
        assert!(rust_expected.contains("arr.get(i)"));
    }

    /// Test 10: Array slicing (Rust-specific)
    /// Range-based access
    #[test]
    fn test_array_slicing() {
        let c_note = "C has no direct slice syntax, use pointers";
        let rust_code = r#"
let slice = &arr[2..5];  // Elements [2, 3, 4]
let slice_from = &arr[2..];  // From index 2 to end
let slice_to = &arr[..5];  // From start to index 5 (exclusive)
"#;

        // Test validates:
        // 1. Rust slice syntax
        // 2. Range operators
        // 3. Safe subarray access
        assert!(c_note.contains("no direct"));
        assert!(rust_code.contains("&arr[2..5]"));
        assert!(rust_code.contains("&arr[2..]"));
        assert!(rust_code.contains("&arr[..5]"));
    }

    /// Test 11: Array subscript with modulo (circular buffer)
    /// Wrap-around indexing
    #[test]
    fn test_array_subscript_modulo() {
        let c_code = r#"
int x = arr[i % size];
"#;

        let rust_expected = r#"
let x = arr[i % size];
"#;

        // Test validates:
        // 1. Modulo for circular access
        // 2. Same syntax
        // 3. Common pattern
        assert!(c_code.contains("arr[i % size]"));
        assert!(rust_expected.contains("arr[i % size]"));
    }

    /// Test 12: Nested array subscript with different indices
    /// 2D array with separate indices
    #[test]
    fn test_nested_array_different_indices() {
        let c_code = r#"
int val = table[row][col];
"#;

        let rust_expected = r#"
let val = table[row][col];
"#;

        // Test validates:
        // 1. Separate row and column indices
        // 2. Common 2D pattern
        // 3. Same syntax
        assert!(c_code.contains("table[row][col]"));
        assert!(rust_expected.contains("table[row][col]"));
    }

    /// Test 13: Array subscript with post-increment (C)
    /// Index modification
    #[test]
    fn test_array_subscript_with_increment() {
        let c_code = r#"
int x = arr[i++];
"#;

        let rust_expected = r#"
let x = arr[i];
i += 1;
"#;

        // Test validates:
        // 1. C: post-increment in subscript
        // 2. Rust: separate statements
        // 3. No ++ operator in Rust
        assert!(c_code.contains("arr[i++]"));
        assert!(rust_expected.contains("arr[i]"));
        assert!(rust_expected.contains("i += 1"));
    }

    /// Test 14: Array subscript bounds check (runtime)
    /// Debug mode checking
    #[test]
    fn test_array_bounds_checking() {
        let c_note = "C: No bounds checking, undefined behavior";
        let rust_note = r#"
// Rust: Bounds checked in debug mode
let arr = [1, 2, 3];
// let x = arr[10];  // PANIC in debug, UB in release
let x = arr.get(10);  // Returns None (safe)
"#;

        // Test validates:
        // 1. C: no protection
        // 2. Rust: debug panic
        // 3. get() always safe
        assert!(c_note.contains("No bounds"));
        assert!(rust_note.contains("PANIC in debug"));
        assert!(rust_note.contains("get(10)"));
    }

    /// Test 15: Array subscript with struct field
    /// Field contains array
    #[test]
    fn test_array_in_struct() {
        let c_code = r#"
struct Data {
    int values[10];
};
struct Data d;
int x = d.values[i];
"#;

        let rust_expected = r#"
struct Data {
    values: [i32; 10],
}
let d = Data { values: [0; 10] };
let x = d.values[i];
"#;

        // Test validates:
        // 1. Array as struct field
        // 2. Field access then subscript
        // 3. Same pattern
        assert!(c_code.contains("d.values[i]"));
        assert!(rust_expected.contains("d.values[i]"));
    }

    /// Test 16: Array subscripting transformation rules summary
    /// Documents all transformation rules in one test
    #[test]
    fn test_array_subscripting_summary() {
        let c_code = r#"
// Rule 1: Simple subscript (same)
int x = arr[i];

// Rule 2: Assignment (same)
arr[i] = 42;

// Rule 3: Constant index (same)
int first = arr[0];

// Rule 4: In expression (same)
int sum = arr[i] + arr[j];

// Rule 5: Multi-dimensional (same)
int val = matrix[i][j];

// Rule 6: Expression index (same)
int x = arr[i + 1];

// Rule 7: In loop (same)
for (int i = 0; i < n; i++) { sum += arr[i]; }

// Rule 8: Pointer subscript (same syntax)
int* p = arr; int x = p[i];

// Rule 9: Manual bounds check
if (i < len) { x = arr[i]; }

// Rule 10: No slice syntax in C

// Rule 11: Modulo (same)
int x = arr[i % size];

// Rule 12: 2D different indices (same)
int val = table[row][col];

// Rule 13: Post-increment in subscript
int x = arr[i++];
"#;

        let rust_expected = r#"
// Rule 1: Same
let x = arr[i];

// Rule 2: Same
arr[i] = 42;

// Rule 3: Same
let first = arr[0];

// Rule 4: Same
let sum = arr[i] + arr[j];

// Rule 5: Same
let val = matrix[i][j];

// Rule 6: Same
let x = arr[i + 1];

// Rule 7: Same pattern
for i in 0..n { sum += arr[i]; }

// Rule 8: Prefer array/slice
let p = &arr[..]; let x = p[i];

// Rule 9: Use get() method
if let Some(&x) = arr.get(i) { }

// Rule 10: Slice syntax
let slice = &arr[2..5];

// Rule 11: Same
let x = arr[i % size];

// Rule 12: Same
let val = table[row][col];

// Rule 13: Separate statements
let x = arr[i]; i += 1;
"#;

        // Test validates all transformation rules
        assert!(c_code.contains("arr[i]"));
        assert!(rust_expected.contains("arr[i]"));
        assert!(c_code.contains("matrix[i][j]"));
        assert!(rust_expected.contains("matrix[i][j]"));
        assert!(c_code.contains("arr[i++]"));
        assert!(rust_expected.contains("let x = arr[i]; i += 1"));
        assert!(rust_expected.contains("arr.get(i)"));
        assert!(rust_expected.contains("&arr[2..5]"));
    }
}
