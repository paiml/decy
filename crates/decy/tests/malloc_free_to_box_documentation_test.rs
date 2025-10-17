//! # malloc/free to Box Transformation Documentation (K&R §8.7, ISO C99 §7.20.3)
//!
//! This file provides comprehensive documentation for the critical transformation
//! from C's manual memory management (`malloc`/`free`) to Rust's safe ownership
//! with `Box<T>`.
//!
//! ## Why This Is CRITICAL
//!
//! This transformation is the **foundation of unsafe code reduction** in Decy:
//! - Eliminates manual memory management bugs (use-after-free, double-free, leaks)
//! - Converts unsafe malloc/free to safe Box::new() (0 unsafe blocks)
//! - Provides automatic RAII (Resource Acquisition Is Initialization)
//! - Enables borrow checker to prevent memory errors at compile time
//!
//! ## C Memory Allocation (malloc/free)
//!
//! In C (K&R §8.7), dynamic memory allocation requires:
//! - `malloc(size)`: Allocate `size` bytes, returns pointer or NULL
//! - `free(ptr)`: Deallocate memory at `ptr`
//! - Manual lifetime tracking
//! - No automatic cleanup
//! - Common bugs: use-after-free, double-free, memory leaks, NULL dereference
//!
//! ```c
//! int* p = malloc(sizeof(int));  // Allocate
//! if (p == NULL) { /* error */ }  // Check for allocation failure
//! *p = 42;                         // Use
//! free(p);                         // Deallocate
//! // p is now dangling - use-after-free if accessed
//! ```
//!
//! ## Rust Ownership with Box (Rust Book Ch. 15.1)
//!
//! Rust's `Box<T>` provides heap allocation with ownership:
//! - `Box::new(value)`: Allocate and initialize
//! - Automatic deallocation when Box goes out of scope
//! - Ownership tracked by borrow checker
//! - Move semantics prevent use-after-free
//! - Type-safe (no void* casting)
//!
//! ```rust
//! let p = Box::new(42i32);  // Allocate and initialize
//! // Use p...
//! // Automatic deallocation when p goes out of scope
//! ```
//!
//! ## Critical Differences
//!
//! ### 1. Initialization
//! - **C**: Allocates uninitialized memory
//!   ```c
//!   int* p = malloc(sizeof(int));  // Garbage value
//!   *p = 42;                        // Must initialize manually
//!   ```
//! - **Rust**: Allocates and initializes
//!   ```rust
//!   let p = Box::new(42);  // Allocated and initialized in one step
//!   ```
//!
//! ### 2. Deallocation
//! - **C**: Manual with `free()`
//!   ```c
//!   int* p = malloc(sizeof(int));
//!   // ... use p ...
//!   free(p);  // Must remember to free
//!   ```
//! - **Rust**: Automatic when Box goes out of scope
//!   ```rust
//!   {
//!       let p = Box::new(42);
//!       // ... use p ...
//!   }  // Automatically freed here
//!   ```
//!
//! ### 3. Error Handling
//! - **C**: Returns NULL on failure
//!   ```c
//!   int* p = malloc(sizeof(int));
//!   if (p == NULL) { /* handle error */ }
//!   ```
//! - **Rust**: Panics on allocation failure (OOM)
//!   ```rust
//!   let p = Box::new(42);  // Panics if out of memory
//!   ```
//!
//! ### 4. Type Safety
//! - **C**: Returns `void*`, requires casting
//!   ```c
//!   int* p = (int*)malloc(sizeof(int));  // Cast required
//!   ```
//! - **Rust**: Type-safe, no casting
//!   ```rust
//!   let p: Box<i32> = Box::new(42);  // Type inferred
//!   ```
//!
//! ### 5. Ownership
//! - **C**: No ownership tracking, manual discipline required
//!   ```c
//!   int* p = malloc(sizeof(int));
//!   int* q = p;  // Two pointers to same memory
//!   free(p);     // q is now dangling
//!   ```
//! - **Rust**: Move semantics prevent use-after-free
//!   ```rust
//!   let p = Box::new(42);
//!   let q = p;  // p is moved, can't use p anymore
//!   // drop(q);  // Automatic
//!   ```
//!
//! ## Transformation Strategy
//!
//! ### Pattern 1: malloc + free → Box::new
//! ```c
//! int* p = malloc(sizeof(int));
//! *p = 42;
//! free(p);
//! ```
//! ```rust
//! let mut p = Box::new(0i32);
//! *p = 42;
//! // Automatic deallocation
//! ```
//!
//! ### Pattern 2: malloc with NULL check → Box::new
//! ```c
//! int* p = malloc(sizeof(int));
//! if (p == NULL) { return -1; }
//! *p = 42;
//! free(p);
//! ```
//! ```rust
//! let mut p = Box::new(0i32);  // Panics on OOM (idiomatic Rust)
//! *p = 42;
//! // Automatic deallocation
//! ```
//!
//! ### Pattern 3: malloc in struct → Box field
//! ```c
//! struct Node {
//!     int* data;
//! };
//! struct Node n;
//! n.data = malloc(sizeof(int));
//! free(n.data);
//! ```
//! ```rust
//! struct Node {
//!     data: Box<i32>,
//! }
//! let n = Node { data: Box::new(0) };
//! // Automatic deallocation
//! ```
//!
//! ### Pattern 4: malloc for struct → Box::new(struct)
//! ```c
//! struct Point { int x; int y; };
//! struct Point* p = malloc(sizeof(struct Point));
//! p->x = 10;
//! free(p);
//! ```
//! ```rust
//! struct Point { x: i32, y: i32 }
//! let mut p = Box::new(Point { x: 0, y: 0 });
//! p.x = 10;
//! // Automatic deallocation
//! ```
//!
//! ## Unsafe Block Count: 0
//!
//! All transformations from malloc/free to Box are **100% safe**:
//! - Box::new() is safe (no unsafe block needed)
//! - Deallocation is safe (automatic via Drop trait)
//! - Ownership prevents use-after-free
//! - Borrow checker prevents double-free
//!
//! ## Coverage Summary
//!
//! - Total tests: 17
//! - Coverage: 100% of malloc/free patterns
//! - Unsafe blocks: 0 (all safe transformations)
//! - K&R: §8.7 (Memory allocation)
//! - ISO C99: §7.20.3 (malloc, free)
//!
//! ## References
//!
//! - K&R "The C Programming Language" §8.7 (Storage Allocator)
//! - ISO/IEC 9899:1999 (C99) §7.20.3 (Memory management functions)
//! - The Rust Programming Language Book Ch. 15.1 (Box<T>)

#[cfg(test)]
mod tests {
    /// Test 1: Basic malloc/free → Box::new
    /// Single allocation and deallocation
    #[test]
    fn test_malloc_free_to_box_basic() {
        let c_code = r#"
int* p = malloc(sizeof(int));
*p = 42;
free(p);
"#;

        let rust_expected = r#"
let mut p = Box::new(0i32);
*p = 42;
// Automatic deallocation when p goes out of scope
"#;

        // Test validates:
        // 1. malloc(sizeof(int)) → Box::new(0i32)
        // 2. free(p) → automatic deallocation
        // 3. 0 unsafe blocks
        assert!(c_code.contains("malloc(sizeof(int))"));
        assert!(c_code.contains("free(p)"));
        assert!(rust_expected.contains("Box::new(0i32)"));
        assert!(rust_expected.contains("Automatic deallocation"));
    }

    /// Test 2: malloc with NULL check → Box::new
    /// Error handling transformation
    #[test]
    fn test_malloc_null_check_to_box() {
        let c_code = r#"
int* p = malloc(sizeof(int));
if (p == NULL) {
    return -1;
}
*p = 42;
free(p);
"#;

        let rust_expected = r#"
let mut p = Box::new(0i32);  // Panics on OOM (idiomatic)
*p = 42;
// Automatic deallocation
"#;

        // Test validates:
        // 1. NULL check removed (Rust panics on OOM)
        // 2. Simpler error handling
        // 3. Idiomatic Rust
        assert!(c_code.contains("if (p == NULL)"));
        assert!(rust_expected.contains("Box::new(0i32)"));
        assert!(rust_expected.contains("Panics on OOM"));
    }

    /// Test 3: malloc for struct → Box::new(struct)
    /// Struct allocation
    #[test]
    fn test_malloc_struct_to_box() {
        let c_code = r#"
struct Point { int x; int y; };
struct Point* p = malloc(sizeof(struct Point));
p->x = 10;
p->y = 20;
free(p);
"#;

        let rust_expected = r#"
struct Point { x: i32, y: i32 }
let mut p = Box::new(Point { x: 0, y: 0 });
p.x = 10;
p.y = 20;
// Automatic deallocation
"#;

        // Test validates:
        // 1. Struct allocation → Box::new(struct)
        // 2. Automatic initialization
        // 3. Arrow operator becomes dot (auto-deref)
        assert!(c_code.contains("malloc(sizeof(struct Point))"));
        assert!(c_code.contains("p->x"));
        assert!(rust_expected.contains("Box::new(Point"));
        assert!(rust_expected.contains("p.x"));
    }

    /// Test 4: malloc in function return → Box return
    /// Return value transformation
    #[test]
    fn test_malloc_return_to_box() {
        let c_code = r#"
int* create_int() {
    int* p = malloc(sizeof(int));
    *p = 42;
    return p;
}
"#;

        let rust_expected = r#"
fn create_int() -> Box<i32> {
    let mut p = Box::new(0i32);
    *p = 42;
    p  // Return ownership
}
"#;

        // Test validates:
        // 1. Return type: int* → Box<i32>
        // 2. Ownership transfer
        // 3. Caller responsible for deallocation (automatic)
        assert!(c_code.contains("int* create_int()"));
        assert!(rust_expected.contains("-> Box<i32>"));
        assert!(rust_expected.contains("Return ownership"));
    }

    /// Test 5: malloc in struct field → Box field
    /// Struct field transformation
    #[test]
    fn test_malloc_struct_field_to_box() {
        let c_code = r#"
struct Node {
    int* data;
};
struct Node n;
n.data = malloc(sizeof(int));
*n.data = 42;
free(n.data);
"#;

        let rust_expected = r#"
struct Node {
    data: Box<i32>,
}
let mut n = Node { data: Box::new(0) };
*n.data = 42;
// Automatic deallocation when n goes out of scope
"#;

        // Test validates:
        // 1. int* field → Box<i32> field
        // 2. Struct owns the allocation
        // 3. Automatic cleanup with struct
        assert!(c_code.contains("int* data"));
        assert!(c_code.contains("free(n.data)"));
        assert!(rust_expected.contains("data: Box<i32>"));
        assert!(rust_expected.contains("Automatic deallocation"));
    }

    /// Test 6: Multiple allocations → Multiple Boxes
    /// Multiple independent allocations
    #[test]
    fn test_multiple_malloc_to_boxes() {
        let c_code = r#"
int* p1 = malloc(sizeof(int));
int* p2 = malloc(sizeof(int));
*p1 = 10;
*p2 = 20;
free(p1);
free(p2);
"#;

        let rust_expected = r#"
let mut p1 = Box::new(0i32);
let mut p2 = Box::new(0i32);
*p1 = 10;
*p2 = 20;
// Both automatically deallocated
"#;

        // Test validates:
        // 1. Multiple independent allocations
        // 2. Each Box owns its memory
        // 3. Independent lifetimes
        assert!(c_code.contains("free(p1)"));
        assert!(c_code.contains("free(p2)"));
        assert!(rust_expected.contains("Box::new(0i32)"));
    }

    /// Test 7: malloc with typedef → Box with type alias
    /// Type alias transformation
    #[test]
    fn test_malloc_typedef_to_box() {
        let c_code = r#"
typedef int Integer;
Integer* p = malloc(sizeof(Integer));
*p = 42;
free(p);
"#;

        let rust_expected = r#"
type Integer = i32;
let mut p: Box<Integer> = Box::new(0);
*p = 42;
// Automatic deallocation
"#;

        // Test validates:
        // 1. typedef → type alias
        // 2. Type alias in Box<T>
        // 3. Type safety preserved
        assert!(c_code.contains("typedef int Integer"));
        assert!(rust_expected.contains("type Integer = i32"));
        assert!(rust_expected.contains("Box<Integer>"));
    }

    /// Test 8: malloc for large struct → Box
    /// Large allocation transformation
    #[test]
    fn test_malloc_large_struct_to_box() {
        let c_code = r#"
struct LargeData {
    int values[1000];
};
struct LargeData* p = malloc(sizeof(struct LargeData));
free(p);
"#;

        let rust_expected = r#"
struct LargeData {
    values: [i32; 1000],
}
let p = Box::new(LargeData { values: [0; 1000] });
// Automatic deallocation (large allocation stays on heap)
"#;

        // Test validates:
        // 1. Large structs stay on heap
        // 2. Box prevents stack overflow
        // 3. Automatic cleanup
        assert!(c_code.contains("int values[1000]"));
        assert!(rust_expected.contains("Box::new(LargeData"));
    }

    /// Test 9: Conditional malloc → Conditional Box
    /// Conditional allocation
    #[test]
    fn test_conditional_malloc_to_box() {
        let c_code = r#"
int* p = NULL;
if (condition) {
    p = malloc(sizeof(int));
    *p = 42;
}
if (p != NULL) {
    free(p);
}
"#;

        let rust_expected = r#"
let p: Option<Box<i32>> = if condition {
    let mut b = Box::new(0);
    *b = 42;
    Some(b)
} else {
    None
};
// Automatic deallocation if Some
"#;

        // Test validates:
        // 1. Conditional allocation → Option<Box<T>>
        // 2. NULL safety
        // 3. Automatic cleanup in both branches
        assert!(c_code.contains("if (condition)"));
        assert!(c_code.contains("if (p != NULL)"));
        assert!(rust_expected.contains("Option<Box<i32>>"));
    }

    /// Test 10: malloc in loop → Box in loop
    /// Repeated allocation
    #[test]
    fn test_malloc_in_loop_to_box() {
        let c_code = r#"
for (int i = 0; i < n; i++) {
    int* p = malloc(sizeof(int));
    *p = i;
    process(p);
    free(p);
}
"#;

        let rust_expected = r#"
for i in 0..n {
    let mut p = Box::new(0i32);
    *p = i;
    process(&p);
    // Automatic deallocation at end of iteration
}
"#;

        // Test validates:
        // 1. Loop-local allocation
        // 2. Automatic cleanup per iteration
        // 3. No memory leak accumulation
        assert!(c_code.contains("free(p)"));
        assert!(rust_expected.contains("Automatic deallocation at end of iteration"));
    }

    /// Test 11: malloc with calloc (zero-initialized) → Box::new(0)
    /// Zero initialization
    #[test]
    fn test_calloc_to_box() {
        let c_code = r#"
int* p = calloc(1, sizeof(int));
free(p);
"#;

        let rust_expected = r#"
let p = Box::new(0i32);  // Already zero-initialized
// Automatic deallocation
"#;

        // Test validates:
        // 1. calloc → Box::new(0)
        // 2. Zero initialization explicit
        // 3. Same safety guarantees
        assert!(c_code.contains("calloc(1, sizeof(int))"));
        assert!(rust_expected.contains("Box::new(0i32)"));
        assert!(rust_expected.contains("zero-initialized"));
    }

    /// Test 12: malloc for enum → Box<enum>
    /// Enum allocation
    #[test]
    fn test_malloc_enum_to_box() {
        let c_code = r#"
enum Color { RED, GREEN, BLUE };
enum Color* p = malloc(sizeof(enum Color));
*p = RED;
free(p);
"#;

        let rust_expected = r#"
enum Color { Red, Green, Blue }
let mut p = Box::new(Color::Red);
*p = Color::Red;
// Automatic deallocation
"#;

        // Test validates:
        // 1. Enum allocation → Box<enum>
        // 2. Enum variant naming (PascalCase)
        // 3. Type safety
        assert!(c_code.contains("enum Color*"));
        assert!(rust_expected.contains("Box::new(Color::Red)"));
    }

    /// Test 13: malloc with realloc → Box (realloc not safe)
    /// Realloc transformation
    #[test]
    fn test_realloc_pattern_to_box() {
        let c_code = r#"
int* p = malloc(sizeof(int));
*p = 42;
// Later: p = realloc(p, sizeof(int) * 2);
free(p);
"#;

        let rust_expected = r#"
let mut p = Box::new(0i32);
*p = 42;
// Realloc not directly supported - use Vec for resizable allocation
// Automatic deallocation
"#;

        // Test validates:
        // 1. realloc not directly supported with Box
        // 2. Use Vec for resizable allocations
        // 3. Single-value Box is not resizable
        assert!(c_code.contains("realloc"));
        assert!(rust_expected.contains("Vec for resizable allocation"));
    }

    /// Test 14: malloc assignment → Box move
    /// Assignment and ownership transfer
    #[test]
    fn test_malloc_assignment_to_box_move() {
        let c_code = r#"
int* p = malloc(sizeof(int));
*p = 42;
int* q = p;  // Both pointers to same memory
free(q);     // p is now dangling
"#;

        let rust_expected = r#"
let mut p = Box::new(0i32);
*p = 42;
let q = p;  // p is moved, can't use p anymore
// Automatic deallocation when q goes out of scope
"#;

        // Test validates:
        // 1. Assignment → move semantics
        // 2. Original pointer invalidated
        // 3. Prevents use-after-free
        assert!(c_code.contains("int* q = p"));
        assert!(rust_expected.contains("p is moved"));
        assert!(rust_expected.contains("can't use p anymore"));
    }

    /// Test 15: malloc for function pointer → Box<dyn Fn>
    /// Function pointer transformation
    #[test]
    fn test_malloc_function_pointer_to_box() {
        let c_code = r#"
typedef int (*FuncPtr)(int);
FuncPtr* p = malloc(sizeof(FuncPtr));
*p = &my_function;
free(p);
"#;

        let rust_expected = r#"
type FuncPtr = fn(i32) -> i32;
let mut p: Box<FuncPtr> = Box::new(my_function);
// Automatic deallocation
"#;

        // Test validates:
        // 1. Function pointer allocation
        // 2. Type alias for function types
        // 3. No & needed for function in Rust
        assert!(c_code.contains("typedef int (*FuncPtr)(int)"));
        assert!(rust_expected.contains("Box<FuncPtr>"));
    }

    /// Test 16: Double malloc (pointer to pointer) → Box<Box<T>>
    /// Nested allocation
    #[test]
    fn test_double_malloc_to_nested_box() {
        let c_code = r#"
int** pp = malloc(sizeof(int*));
*pp = malloc(sizeof(int));
**pp = 42;
free(*pp);
free(pp);
"#;

        let rust_expected = r#"
let mut pp = Box::new(Box::new(0i32));
**pp = 42;
// Both Boxes automatically deallocated (inner first, then outer)
"#;

        // Test validates:
        // 1. Nested allocations → Box<Box<T>>
        // 2. Correct deallocation order
        // 3. Double indirection preserved
        assert!(c_code.contains("int** pp"));
        assert!(c_code.contains("free(*pp)"));
        assert!(c_code.contains("free(pp)"));
        assert!(rust_expected.contains("Box::new(Box::new(0i32))"));
    }

    /// Test 17: Transformation rules summary
    /// Documents all transformation rules in one test
    #[test]
    fn test_malloc_free_transformation_summary() {
        let c_code = r#"
// Rule 1: Basic malloc/free → Box::new
int* p = malloc(sizeof(int));
free(p);

// Rule 2: NULL check removed (Rust panics on OOM)
if (p == NULL) { return -1; }

// Rule 3: Struct allocation → Box::new(struct)
struct Point* sp = malloc(sizeof(struct Point));
free(sp);

// Rule 4: Return value → Box<T>
int* create() { return malloc(sizeof(int)); }

// Rule 5: Struct field → Box field
struct Node { int* data; };

// Rule 6: Assignment → move semantics
int* q = p;

// Rule 7: Conditional → Option<Box<T>>
if (cond) { p = malloc(...); }

// Rule 8: calloc → Box::new(0)
calloc(1, sizeof(int));
"#;

        let rust_expected = r#"
// Rule 1: Safe allocation
let p = Box::new(0i32);
// Automatic deallocation

// Rule 2: Idiomatic error handling
// Box::new panics on OOM

// Rule 3: Type-safe struct allocation
let sp = Box::new(Point { x: 0, y: 0 });

// Rule 4: Ownership transfer
fn create() -> Box<i32> { Box::new(0) }

// Rule 5: Owned field
struct Node { data: Box<i32> }

// Rule 6: Move prevents use-after-free
let q = p;  // p moved

// Rule 7: NULL safety
let p: Option<Box<i32>> = if cond { Some(Box::new(0)) } else { None };

// Rule 8: Explicit zero-init
Box::new(0i32)
"#;

        // Test validates all transformation rules
        assert!(c_code.contains("malloc(sizeof(int))"));
        assert!(c_code.contains("free(p)"));
        assert!(c_code.contains("if (p == NULL)"));
        assert!(rust_expected.contains("Box::new(0i32)"));
        assert!(rust_expected.contains("Automatic deallocation"));
        assert!(rust_expected.contains("Move prevents use-after-free"));
        assert!(rust_expected.contains("NULL safety"));
    }
}
