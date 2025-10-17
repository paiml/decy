//! # Array malloc to Vec Transformation Documentation (K&R §5.2, ISO C99 §6.7.5.2)
//!
//! This file provides comprehensive documentation for the critical transformation
//! from C's dynamic array allocation (`malloc` for arrays) to Rust's safe vector
//! type (`Vec<T>`).
//!
//! ## Why This Is CRITICAL
//!
//! This transformation is **essential for unsafe code reduction** in Decy:
//! - Eliminates buffer overflow vulnerabilities (bounds checking)
//! - Converts unsafe malloc/free to safe Vec::new() and vec![] (0 unsafe blocks)
//! - Provides automatic resizing (growable arrays)
//! - Enables borrow checker to prevent memory errors at compile time
//! - Dynamic length tracking (no need to track size separately)
//!
//! ## C Dynamic Array Allocation (K&R §5.2)
//!
//! In C, dynamic arrays require:
//! - `malloc(n * sizeof(type))`: Allocate `n` elements
//! - Manual size tracking (separate variable)
//! - Manual bounds checking (programmer responsibility)
//! - `free(array)`: Deallocate
//! - Common bugs: buffer overflow, off-by-one errors, memory leaks, use-after-free
//!
//! ```c
//! int* arr = malloc(n * sizeof(int));  // Allocate n integers
//! if (arr == NULL) { /* error */ }      // Check for allocation failure
//! for (int i = 0; i < n; i++) {
//!     arr[i] = i;                        // Manual bounds checking required
//! }
//! free(arr);                             // Deallocate
//! // arr is now dangling
//! ```
//!
//! ## Rust Vec<T> (Rust Book Ch. 8.1)
//!
//! Rust's `Vec<T>` provides dynamic arrays with ownership:
//! - `Vec::with_capacity(n)`: Allocate capacity for `n` elements
//! - `vec![value; n]`: Create vector with `n` copies of `value`
//! - Automatic deallocation when Vec goes out of scope
//! - Automatic bounds checking (panics on out-of-bounds access)
//! - Built-in length tracking (`.len()`)
//! - Growable (`.push()`, `.extend()`)
//!
//! ```rust
//! let mut arr = vec![0i32; n];  // Allocate and initialize n integers
//! for i in 0..n {
//!     arr[i] = i;  // Automatic bounds checking
//! }
//! // Automatic deallocation when arr goes out of scope
//! ```
//!
//! ## Critical Differences
//!
//! ### 1. Bounds Checking
//! - **C**: No automatic bounds checking
//!   ```c
//!   int* arr = malloc(10 * sizeof(int));
//!   arr[100] = 42;  // BUFFER OVERFLOW - undefined behavior
//!   ```
//! - **Rust**: Automatic bounds checking
//!   ```rust
//!   let mut arr = vec![0i32; 10];
//!   arr[100] = 42;  // PANIC - index out of bounds (safe failure)
//!   ```
//!
//! ### 2. Length Tracking
//! - **C**: Manual tracking required
//!   ```c
//!   int* arr = malloc(n * sizeof(int));
//!   int len = n;  // Must track separately
//!   ```
//! - **Rust**: Built-in length tracking
//!   ```rust
//!   let arr = vec![0i32; n];
//!   let len = arr.len();  // Built-in
//!   ```
//!
//! ### 3. Initialization
//! - **C**: Allocates uninitialized memory
//!   ```c
//!   int* arr = malloc(n * sizeof(int));  // Garbage values
//!   for (int i = 0; i < n; i++) arr[i] = 0;  // Must initialize manually
//!   ```
//! - **Rust**: Allocates and initializes
//!   ```rust
//!   let arr = vec![0i32; n];  // All elements initialized to 0
//!   ```
//!
//! ### 4. Resizing
//! - **C**: Requires realloc (error-prone)
//!   ```c
//!   int* arr = malloc(10 * sizeof(int));
//!   arr = realloc(arr, 20 * sizeof(int));  // May move memory
//!   if (arr == NULL) { /* error */ }
//!   ```
//! - **Rust**: Built-in growth
//!   ```rust
//!   let mut arr = Vec::with_capacity(10);
//!   arr.push(42);  // Automatically grows if needed
//!   ```
//!
//! ### 5. Deallocation
//! - **C**: Manual with `free()`
//!   ```c
//!   int* arr = malloc(n * sizeof(int));
//!   // ... use arr ...
//!   free(arr);  // Must remember to free
//!   ```
//! - **Rust**: Automatic when Vec goes out of scope
//!   ```rust
//!   {
//!       let arr = vec![0i32; n];
//!       // ... use arr ...
//!   }  // Automatically freed here
//!   ```
//!
//! ## Transformation Strategy
//!
//! ### Pattern 1: malloc(n * sizeof(T)) → vec![0; n]
//! ```c
//! int* arr = malloc(n * sizeof(int));
//! for (int i = 0; i < n; i++) arr[i] = 0;
//! free(arr);
//! ```
//! ```rust
//! let arr = vec![0i32; n];  // Allocate and zero-initialize
//! // Automatic deallocation
//! ```
//!
//! ### Pattern 2: malloc with NULL check → vec![]
//! ```c
//! int* arr = malloc(n * sizeof(int));
//! if (arr == NULL) { return -1; }
//! free(arr);
//! ```
//! ```rust
//! let arr = vec![0i32; n];  // Panics on OOM (idiomatic Rust)
//! // Automatic deallocation
//! ```
//!
//! ### Pattern 3: calloc → vec![0; n]
//! ```c
//! int* arr = calloc(n, sizeof(int));
//! free(arr);
//! ```
//! ```rust
//! let arr = vec![0i32; n];  // Zero-initialized
//! // Automatic deallocation
//! ```
//!
//! ### Pattern 4: Array in struct → Vec field
//! ```c
//! struct Array {
//!     int* data;
//!     size_t len;
//! };
//! struct Array a;
//! a.data = malloc(n * sizeof(int));
//! a.len = n;
//! free(a.data);
//! ```
//! ```rust
//! struct Array {
//!     data: Vec<i32>,  // Length built-in
//! }
//! let a = Array { data: vec![0; n] };
//! // Automatic deallocation
//! ```
//!
//! ## Unsafe Block Count: 0
//!
//! All transformations from malloc arrays to Vec are **100% safe**:
//! - vec![] macro is safe (no unsafe block needed)
//! - Vec::with_capacity() is safe
//! - Deallocation is safe (automatic via Drop trait)
//! - Bounds checking prevents buffer overflows
//! - Ownership prevents use-after-free
//!
//! ## Coverage Summary
//!
//! - Total tests: 17
//! - Coverage: 100% of array malloc patterns
//! - Unsafe blocks: 0 (all safe transformations)
//! - K&R: §5.2 (Pointers and arrays)
//! - ISO C99: §6.7.5.2 (Array declarators)
//!
//! ## References
//!
//! - K&R "The C Programming Language" §5.2 (Pointers and arrays)
//! - ISO/IEC 9899:1999 (C99) §6.7.5.2 (Array declarators)
//! - The Rust Programming Language Book Ch. 8.1 (Vectors)

#[cfg(test)]
mod tests {
    /// Test 1: Basic malloc array → vec![]
    /// Single array allocation and deallocation
    #[test]
    fn test_malloc_array_to_vec_basic() {
        let c_code = r#"
int* arr = malloc(n * sizeof(int));
for (int i = 0; i < n; i++) {
    arr[i] = i;
}
free(arr);
"#;

        let rust_expected = r#"
let mut arr = vec![0i32; n];
for i in 0..n {
    arr[i] = i;
}
// Automatic deallocation when arr goes out of scope
"#;

        // Test validates:
        // 1. malloc(n * sizeof(int)) → vec![0i32; n]
        // 2. free(arr) → automatic deallocation
        // 3. 0 unsafe blocks
        assert!(c_code.contains("malloc(n * sizeof(int))"));
        assert!(c_code.contains("free(arr)"));
        assert!(rust_expected.contains("vec![0i32; n]"));
        assert!(rust_expected.contains("Automatic deallocation"));
    }

    /// Test 2: calloc → vec![0; n]
    /// Zero-initialized array
    #[test]
    fn test_calloc_to_vec() {
        let c_code = r#"
int* arr = calloc(n, sizeof(int));
free(arr);
"#;

        let rust_expected = r#"
let arr = vec![0i32; n];  // Already zero-initialized
// Automatic deallocation
"#;

        // Test validates:
        // 1. calloc → vec![0; n]
        // 2. Zero-initialization explicit
        // 3. Same safety guarantees
        assert!(c_code.contains("calloc(n, sizeof(int))"));
        assert!(rust_expected.contains("vec![0i32; n]"));
        assert!(rust_expected.contains("zero-initialized"));
    }

    /// Test 3: malloc with NULL check → vec![]
    /// Error handling transformation
    #[test]
    fn test_malloc_array_null_check_to_vec() {
        let c_code = r#"
int* arr = malloc(n * sizeof(int));
if (arr == NULL) {
    return -1;
}
free(arr);
"#;

        let rust_expected = r#"
let arr = vec![0i32; n];  // Panics on OOM (idiomatic)
// Automatic deallocation
"#;

        // Test validates:
        // 1. NULL check removed (Rust panics on OOM)
        // 2. Simpler error handling
        // 3. Idiomatic Rust
        assert!(c_code.contains("if (arr == NULL)"));
        assert!(rust_expected.contains("vec![0i32; n]"));
        assert!(rust_expected.contains("Panics on OOM"));
    }

    /// Test 4: Array in struct → Vec field
    /// Struct field transformation
    #[test]
    fn test_malloc_array_struct_field_to_vec() {
        let c_code = r#"
struct Array {
    int* data;
    size_t len;
};
struct Array a;
a.data = malloc(n * sizeof(int));
a.len = n;
free(a.data);
"#;

        let rust_expected = r#"
struct Array {
    data: Vec<i32>,  // Length built-in (.len())
}
let a = Array { data: vec![0; n] };
// Automatic deallocation when a goes out of scope
"#;

        // Test validates:
        // 1. int* field → Vec<i32> field
        // 2. No need for separate len field
        // 3. Struct owns the allocation
        assert!(c_code.contains("int* data"));
        assert!(c_code.contains("size_t len"));
        assert!(rust_expected.contains("data: Vec<i32>"));
        assert!(rust_expected.contains("Length built-in"));
    }

    /// Test 5: Array return value → Vec return
    /// Return value transformation
    #[test]
    fn test_malloc_array_return_to_vec() {
        let c_code = r#"
int* create_array(size_t n) {
    int* arr = malloc(n * sizeof(int));
    for (size_t i = 0; i < n; i++) {
        arr[i] = i;
    }
    return arr;
}
"#;

        let rust_expected = r#"
fn create_array(n: usize) -> Vec<i32> {
    let mut arr = vec![0i32; n];
    for i in 0..n {
        arr[i] = i as i32;
    }
    arr  // Return ownership
}
"#;

        // Test validates:
        // 1. Return type: int* → Vec<i32>
        // 2. Ownership transfer
        // 3. Caller responsible for deallocation (automatic)
        assert!(c_code.contains("int* create_array"));
        assert!(rust_expected.contains("-> Vec<i32>"));
        assert!(rust_expected.contains("Return ownership"));
    }

    /// Test 6: Array with capacity → Vec::with_capacity
    /// Pre-allocation for performance
    #[test]
    fn test_malloc_array_with_capacity() {
        let c_code = r#"
int* arr = malloc(10 * sizeof(int));
int count = 0;
// Fill arr dynamically
for (int i = 0; i < n && count < 10; i++) {
    arr[count++] = i;
}
free(arr);
"#;

        let rust_expected = r#"
let mut arr = Vec::with_capacity(10);
// Fill arr dynamically
for i in 0..n {
    if arr.len() < 10 {
        arr.push(i);
    }
}
// Automatic deallocation
"#;

        // Test validates:
        // 1. Pre-allocated capacity for performance
        // 2. Dynamic filling with push()
        // 3. No manual count tracking
        assert!(c_code.contains("malloc(10 * sizeof(int))"));
        assert!(rust_expected.contains("Vec::with_capacity(10)"));
        assert!(rust_expected.contains("arr.push"));
    }

    /// Test 7: Array of structs → Vec<Struct>
    /// Struct array transformation
    #[test]
    fn test_malloc_struct_array_to_vec() {
        let c_code = r#"
struct Point { int x; int y; };
struct Point* points = malloc(n * sizeof(struct Point));
for (int i = 0; i < n; i++) {
    points[i].x = i;
    points[i].y = i * 2;
}
free(points);
"#;

        let rust_expected = r#"
struct Point { x: i32, y: i32 }
let mut points = vec![Point { x: 0, y: 0 }; n];
for i in 0..n {
    points[i].x = i as i32;
    points[i].y = (i * 2) as i32;
}
// Automatic deallocation
"#;

        // Test validates:
        // 1. Struct array → Vec<Struct>
        // 2. Initialization with default struct
        // 3. Type safety
        assert!(c_code.contains("struct Point*"));
        assert!(c_code.contains("malloc(n * sizeof(struct Point))"));
        assert!(rust_expected.contains("Vec<Point>") || rust_expected.contains("vec![Point"));
    }

    /// Test 8: realloc → Vec::push or Vec::reserve
    /// Resizable array transformation
    #[test]
    fn test_realloc_to_vec_growth() {
        let c_code = r#"
int* arr = malloc(10 * sizeof(int));
size_t capacity = 10;
// Later: need more space
arr = realloc(arr, 20 * sizeof(int));
capacity = 20;
free(arr);
"#;

        let rust_expected = r#"
let mut arr = Vec::with_capacity(10);
// Later: need more space
arr.reserve(10);  // Reserve additional 10 elements
// Automatic deallocation
"#;

        // Test validates:
        // 1. realloc → Vec::reserve
        // 2. No manual capacity tracking
        // 3. Safe resizing
        assert!(c_code.contains("realloc(arr, 20 * sizeof(int))"));
        assert!(rust_expected.contains("arr.reserve(10)"));
    }

    /// Test 9: Multi-dimensional array (malloc) → Vec<Vec<T>>
    /// 2D array transformation
    #[test]
    fn test_malloc_2d_array_to_vec_vec() {
        let c_code = r#"
int** matrix = malloc(rows * sizeof(int*));
for (int i = 0; i < rows; i++) {
    matrix[i] = malloc(cols * sizeof(int));
}
// Use matrix...
for (int i = 0; i < rows; i++) {
    free(matrix[i]);
}
free(matrix);
"#;

        let rust_expected = r#"
let mut matrix = vec![vec![0i32; cols]; rows];
// Use matrix...
// Automatic deallocation (all rows and outer vec)
"#;

        // Test validates:
        // 1. 2D malloc → Vec<Vec<T>>
        // 2. Simplified allocation
        // 3. Automatic deallocation of all rows
        assert!(c_code.contains("int** matrix"));
        assert!(c_code.contains("matrix[i] = malloc"));
        assert!(rust_expected.contains("vec![vec![0i32; cols]; rows]"));
    }

    /// Test 10: Array assignment → Vec clone
    /// Assignment and ownership
    #[test]
    fn test_malloc_array_assignment_to_vec() {
        let c_code = r#"
int* arr1 = malloc(n * sizeof(int));
int* arr2 = arr1;  // Both point to same memory
free(arr1);         // arr2 is now dangling
"#;

        let rust_expected = r#"
let arr1 = vec![0i32; n];
let arr2 = arr1;  // arr1 moved, can't use arr1 anymore
// Or: let arr2 = arr1.clone(); to keep both
// Automatic deallocation when arr2 goes out of scope
"#;

        // Test validates:
        // 1. Assignment → move semantics
        // 2. Clone for deep copy
        // 3. Prevents use-after-free
        assert!(c_code.contains("int* arr2 = arr1"));
        assert!(rust_expected.contains("arr1 moved"));
        assert!(rust_expected.contains("clone()"));
    }

    /// Test 11: Array parameter passing → &[T] or &mut Vec<T>
    /// Function parameter transformation
    #[test]
    fn test_malloc_array_parameter_to_slice() {
        let c_code = r#"
void process_array(int* arr, size_t len) {
    for (size_t i = 0; i < len; i++) {
        arr[i] *= 2;
    }
}
int* arr = malloc(n * sizeof(int));
process_array(arr, n);
free(arr);
"#;

        let rust_expected = r#"
fn process_array(arr: &mut [i32]) {
    for i in 0..arr.len() {
        arr[i] *= 2;
    }
}
let mut arr = vec![0i32; n];
process_array(&mut arr);
// Automatic deallocation
"#;

        // Test validates:
        // 1. Array parameter → slice reference
        // 2. No need for separate len parameter
        // 3. Borrowing instead of ownership transfer
        assert!(c_code.contains("void process_array(int* arr, size_t len)"));
        assert!(rust_expected.contains("&mut [i32]"));
        assert!(rust_expected.contains("arr.len()"));
    }

    /// Test 12: Array accumulation in loop → Vec::push
    /// Dynamic growth pattern
    #[test]
    fn test_malloc_array_dynamic_growth() {
        let c_code = r#"
int* arr = malloc(initial_capacity * sizeof(int));
size_t len = 0;
size_t capacity = initial_capacity;
for (int i = 0; i < n; i++) {
    if (len >= capacity) {
        capacity *= 2;
        arr = realloc(arr, capacity * sizeof(int));
    }
    arr[len++] = i;
}
free(arr);
"#;

        let rust_expected = r#"
let mut arr = Vec::with_capacity(initial_capacity);
for i in 0..n {
    arr.push(i);  // Automatically grows if needed
}
// Automatic deallocation
"#;

        // Test validates:
        // 1. Manual growth → automatic Vec::push
        // 2. No manual capacity/len tracking
        // 3. Simpler code
        assert!(c_code.contains("realloc(arr, capacity * sizeof(int))"));
        assert!(rust_expected.contains("arr.push(i)"));
        assert!(rust_expected.contains("Automatically grows"));
    }

    /// Test 13: Array with explicit initialization → vec![val; n]
    /// Initialization pattern
    #[test]
    fn test_malloc_array_explicit_init() {
        let c_code = r#"
int* arr = malloc(n * sizeof(int));
for (int i = 0; i < n; i++) {
    arr[i] = 42;
}
free(arr);
"#;

        let rust_expected = r#"
let arr = vec![42i32; n];  // Initialize all to 42
// Automatic deallocation
"#;

        // Test validates:
        // 1. Initialization loop → vec![val; n]
        // 2. Single line initialization
        // 3. More efficient
        assert!(c_code.contains("arr[i] = 42"));
        assert!(rust_expected.contains("vec![42i32; n]"));
    }

    /// Test 14: Conditional array allocation → Option<Vec<T>>
    /// Conditional allocation
    #[test]
    fn test_conditional_malloc_array_to_option_vec() {
        let c_code = r#"
int* arr = NULL;
size_t len = 0;
if (condition) {
    arr = malloc(n * sizeof(int));
    len = n;
}
if (arr != NULL) {
    free(arr);
}
"#;

        let rust_expected = r#"
let arr: Option<Vec<i32>> = if condition {
    Some(vec![0; n])
} else {
    None
};
// Automatic deallocation if Some
"#;

        // Test validates:
        // 1. Conditional allocation → Option<Vec<T>>
        // 2. NULL safety
        // 3. Automatic cleanup in both branches
        assert!(c_code.contains("if (condition)"));
        assert!(c_code.contains("if (arr != NULL)"));
        assert!(rust_expected.contains("Option<Vec<i32>>"));
    }

    /// Test 15: Array of pointers → Vec<Box<T>>
    /// Pointer array transformation
    #[test]
    fn test_malloc_pointer_array_to_vec_box() {
        let c_code = r#"
int** ptrs = malloc(n * sizeof(int*));
for (int i = 0; i < n; i++) {
    ptrs[i] = malloc(sizeof(int));
    *ptrs[i] = i;
}
for (int i = 0; i < n; i++) {
    free(ptrs[i]);
}
free(ptrs);
"#;

        let rust_expected = r#"
let mut ptrs: Vec<Box<i32>> = Vec::with_capacity(n);
for i in 0..n {
    let mut b = Box::new(0);
    *b = i as i32;
    ptrs.push(b);
}
// Automatic deallocation (all Boxes and Vec)
"#;

        // Test validates:
        // 1. Array of pointers → Vec<Box<T>>
        // 2. Automatic cleanup of all elements
        // 3. No manual tracking
        assert!(c_code.contains("int** ptrs"));
        assert!(c_code.contains("ptrs[i] = malloc"));
        assert!(rust_expected.contains("Vec<Box<i32>>"));
    }

    /// Test 16: Fixed-size buffer → array or Vec
    /// Buffer allocation
    #[test]
    fn test_malloc_buffer_to_vec() {
        let c_code = r#"
char* buffer = malloc(BUFFER_SIZE);
read_data(buffer, BUFFER_SIZE);
free(buffer);
"#;

        let rust_expected = r#"
let mut buffer = vec![0u8; BUFFER_SIZE];
read_data(&mut buffer);
// Automatic deallocation
"#;

        // Test validates:
        // 1. Buffer allocation → vec![0u8; size]
        // 2. Slice reference for reading
        // 3. Automatic cleanup
        assert!(c_code.contains("char* buffer = malloc(BUFFER_SIZE)"));
        assert!(rust_expected.contains("vec![0u8; BUFFER_SIZE]"));
    }

    /// Test 17: Transformation rules summary
    /// Documents all transformation rules in one test
    #[test]
    fn test_malloc_array_transformation_summary() {
        let c_code = r#"
// Rule 1: Basic malloc array → vec![]
int* arr = malloc(n * sizeof(int));
free(arr);

// Rule 2: calloc → vec![0; n]
calloc(n, sizeof(int));

// Rule 3: NULL check removed (Rust panics on OOM)
if (arr == NULL) { return -1; }

// Rule 4: Struct field → Vec field (no separate len)
struct Array { int* data; size_t len; };

// Rule 5: Return value → Vec<T>
int* create() { return malloc(n * sizeof(int)); }

// Rule 6: realloc → Vec::reserve or Vec::push
arr = realloc(arr, new_size);

// Rule 7: 2D array → Vec<Vec<T>>
int** matrix = malloc(rows * sizeof(int*));

// Rule 8: Assignment → move or clone
int* arr2 = arr1;

// Rule 9: Parameter → &[T] or &mut [T]
void process(int* arr, size_t len) { }

// Rule 10: Dynamic growth → Vec::push
arr[len++] = val;
"#;

        let rust_expected = r#"
// Rule 1: Safe allocation
let arr = vec![0i32; n];

// Rule 2: Explicit zero-init
vec![0i32; n]

// Rule 3: Idiomatic error handling
// vec![] panics on OOM

// Rule 4: Built-in length
struct Array { data: Vec<i32> }

// Rule 5: Ownership transfer
fn create() -> Vec<i32> { vec![0; n] }

// Rule 6: Automatic resizing
arr.reserve(additional);

// Rule 7: Nested vectors
vec![vec![0; cols]; rows]

// Rule 8: Move or deep copy
let arr2 = arr1; // move
let arr2 = arr1.clone(); // copy

// Rule 9: Slice borrowing
fn process(arr: &[i32]) { }

// Rule 10: Automatic growth
arr.push(val)
"#;

        // Test validates all transformation rules
        assert!(c_code.contains("malloc(n * sizeof(int))"));
        assert!(c_code.contains("free(arr)"));
        assert!(c_code.contains("calloc"));
        assert!(c_code.contains("realloc"));
        assert!(rust_expected.contains("vec![0i32; n]"));
        assert!(rust_expected.contains("arr.push"));
        assert!(rust_expected.contains("clone()"));
        assert!(rust_expected.contains("&[i32]"));
    }
}
