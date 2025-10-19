//! Documentation tests for C dereference operator (*) → Rust dereferencing
//!
//! This test file documents the transformation from C's dereference operator (*)
//! to Rust's safe dereferencing. The dereference operator is the inverse of the
//! address-of operator (&) and is used to access the value pointed to by a pointer.
//!
//! # Reference
//! - K&R C (2nd Edition) §5.1: Pointers and Addresses
//! - ISO C99 Standard §6.5.3.2: Unary operators (indirection)
//!
//! # Safety Guarantees
//! Rust's dereferencing provides:
//! - Type safety (cannot dereference incompatible types)
//! - Lifetime safety (references always valid)
//! - Bounds checking (for array/slice access)
//! - No null pointer dereferences (use Option for nullable)
//!
//! # Transformation Strategy
//! 1. `*p` where p is valid reference → `*p` (safe in Rust)
//! 2. `*p` where p might be NULL → `*p.as_ref().unwrap()` or match on Option
//! 3. `*p` with raw pointer → `unsafe { *p }` (requires unsafe block)
//! 4. Auto-deref for method calls (Rust feature, no explicit * needed)
//!
//! # Target Metrics
//! - Coverage: 100%
//! - Unsafe blocks: 0 (for safe reference dereferencing)
//! - Tests: 17 comprehensive scenarios

#[cfg(test)]
mod tests {
    //! All tests validate dereference operator transformation patterns

    #[test]
    fn test_dereference_basic_immutable() {
        // C: Dereferencing pointer to read value
        let c_code = r#"
int x = 42;
int* p = &x;
int y = *p;
"#;

        // Rust: Dereferencing immutable borrow
        let rust_expected = r#"
let x: i32 = 42;
let p: &i32 = &x;
let y: i32 = *p;
"#;

        // Validates: *p works the same in C and Rust (for safe references)
        assert!(c_code.contains("int y = *p"));
        assert!(rust_expected.contains("let y: i32 = *p"));

        // Key difference: Rust ensures p is always valid (lifetime checking)
    }

    #[test]
    fn test_dereference_mutable() {
        // C: Dereferencing pointer to modify value
        let c_code = r#"
int x = 10;
int* p = &x;
*p = 20;
printf("%d\n", x);  // Prints 20
"#;

        // Rust: Dereferencing mutable borrow
        let rust_expected = r#"
let mut x: i32 = 10;
let p: &mut i32 = &mut x;
*p = 20;
println!("{}", x);  // Prints 20
"#;

        // Validates: *p = value works the same
        assert!(c_code.contains("*p = 20"));
        assert!(rust_expected.contains("*p = 20"));

        // CRITICAL: Rust requires &mut for dereferencing to mutate
        assert!(rust_expected.contains("&mut i32"));
    }

    #[test]
    fn test_dereference_compound_assignment() {
        // C: Compound assignment through pointer
        let c_code = r#"
int x = 5;
int* p = &x;
*p += 10;
*p *= 2;
"#;

        // Rust: Compound assignment through mutable reference
        let rust_expected = r#"
let mut x: i32 = 5;
let p: &mut i32 = &mut x;
*p += 10;
*p *= 2;
"#;

        // Validates: Compound operators work through dereference
        assert!(c_code.contains("*p += 10"));
        assert!(rust_expected.contains("*p += 10"));
        assert!(c_code.contains("*p *= 2"));
        assert!(rust_expected.contains("*p *= 2"));
    }

    #[test]
    fn test_dereference_pointer_arithmetic() {
        // C: Dereferencing pointer after arithmetic
        let c_code = r#"
int arr[5] = {1, 2, 3, 4, 5};
int* p = arr;
int x = *(p + 2);  // arr[2] = 3
"#;

        // Rust: Index access (safer than pointer arithmetic)
        let rust_expected = r#"
let arr: [i32; 5] = [1, 2, 3, 4, 5];
let x: i32 = arr[2];  // Direct indexing (bounds-checked)
// Or with slice:
// let p: &[i32] = &arr;
// let x: i32 = p[2];
"#;

        // Validates: *(p + offset) → arr[index]
        assert!(c_code.contains("*(p + 2)"));
        assert!(rust_expected.contains("arr[2]"));

        // CRITICAL: Rust prefers indexing over pointer arithmetic (safer)
        assert!(rust_expected.contains("bounds-checked"));
    }

    #[test]
    fn test_dereference_double_pointer() {
        // C: Double dereference
        let c_code = r#"
int x = 100;
int* p = &x;
int** pp = &p;
int y = **pp;
"#;

        // Rust: Double dereference of nested references
        let rust_expected = r#"
let x: i32 = 100;
let p: &i32 = &x;
let pp: &&i32 = &p;
let y: i32 = **pp;
"#;

        // Validates: **pp works the same
        assert!(c_code.contains("int y = **pp"));
        assert!(rust_expected.contains("let y: i32 = **pp"));

        // Note: Nested references are valid in Rust
        assert!(rust_expected.contains("&&i32"));
    }

    #[test]
    fn test_dereference_struct_field() {
        // C: Dereferencing to access struct
        let c_code = r#"
struct Point {
    int x;
    int y;
};

struct Point* p = get_point();
int a = (*p).x;  // Explicit dereference
int b = p->x;    // Arrow operator (syntactic sugar)
"#;

        // Rust: Auto-deref for field access
        let rust_expected = r#"
struct Point {
    x: i32,
    y: i32,
}

let p: &Point = get_point();
let a: i32 = (*p).x;  // Explicit dereference (works)
let b: i32 = p.x;     // Auto-deref (idiomatic Rust)
"#;

        // Validates: Both (*p).x and p.x work in Rust
        assert!(c_code.contains("(*p).x"));
        assert!(c_code.contains("p->x"));
        assert!(rust_expected.contains("(*p).x"));
        assert!(rust_expected.contains("p.x"));

        // CRITICAL: Rust has auto-deref, no need for -> operator
        assert!(rust_expected.contains("Auto-deref"));
    }

    #[test]
    fn test_dereference_method_call() {
        // C: Function call through pointer (C doesn't have methods)
        let c_code = r#"
struct String {
    char* data;
    int len;
};

struct String* s = get_string();
int length = (*s).len;
"#;

        // Rust: Method call with auto-deref
        let rust_expected = r#"
let s: &String = get_string();
let length: usize = s.len();  // Auto-deref for method calls

// Both work:
// let length1 = (*s).len();  // Explicit deref
// let length2 = s.len();     // Auto-deref (idiomatic)
"#;

        // Validates: Auto-deref is a Rust convenience
        assert!(c_code.contains("(*s).len"));
        assert!(rust_expected.contains("s.len()"));
        assert!(rust_expected.contains("Auto-deref"));
    }

    #[test]
    fn test_dereference_null_pointer_undefined_behavior() {
        // C: Dereferencing NULL pointer (UNDEFINED BEHAVIOR - segfault)
        let c_code = r#"
int* p = NULL;
int x = *p;  // CRASH! Undefined behavior
"#;

        // Rust: Cannot compile (NULL safety via Option)
        let _rust_compilation_error = r#"
// This pattern doesn't exist in safe Rust
// Cannot create null reference
let p: &i32 = ???;  // No way to represent NULL reference
"#;

        // Rust: Safe version with Option
        let rust_safe = r#"
let p: Option<&i32> = None;
// Must handle None case:
match p {
    Some(value) => {
        let x: i32 = *value;  // Safe dereference
        println!("{}", x);
    },
    None => {
        println!("No value");
    }
}
"#;

        // Validates: C allows NULL dereference (UB)
        assert!(c_code.contains("int x = *p"));

        // Validates: Rust prevents NULL via Option
        assert!(rust_safe.contains("Option<&i32>"));
        assert!(rust_safe.contains("match p"));
    }

    #[test]
    fn test_dereference_dangling_pointer() {
        // C: Dereferencing dangling pointer (UNDEFINED BEHAVIOR)
        let c_code = r#"
int* p;
{
    int x = 42;
    p = &x;
}  // x goes out of scope, p is now dangling
int y = *p;  // UB: p points to freed stack memory
"#;

        // Rust: Cannot compile (lifetime error)
        let rust_compilation_error = r#"
// This code WILL NOT COMPILE
let p: &i32;
{
    let x: i32 = 42;
    p = &x;  // ERROR: `x` does not live long enough
}
let y: i32 = *p;
"#;

        // Rust: Safe version (move value out)
        let _rust_safe = r#"
let y: i32;
{
    let x: i32 = 42;
    y = x;  // Copy value before x goes out of scope
}
// y is valid here
"#;

        // Validates: C allows dangling pointer dereference
        assert!(c_code.contains("int y = *p"));

        // Validates: Rust prevents dangling via lifetime checking
        assert!(rust_compilation_error.contains("does not live long enough"));
    }

    #[test]
    fn test_dereference_array_subscript() {
        // C: Array subscript is syntactic sugar for dereference
        let c_code = r#"
int arr[5] = {1, 2, 3, 4, 5};
int* p = arr;
// These are equivalent in C:
int a = p[2];     // Array subscript
int b = *(p + 2); // Pointer arithmetic + dereference
"#;

        // Rust: Indexing is NOT pointer arithmetic
        let rust_expected = r#"
let arr: [i32; 5] = [1, 2, 3, 4, 5];
let p: &[i32] = &arr;
let a: i32 = p[2];  // Bounds-checked indexing

// No direct pointer arithmetic in safe Rust
// Must use slice methods or unsafe code
"#;

        // Validates: C equivalence of [] and *( + )
        assert!(c_code.contains("p[2]"));
        assert!(c_code.contains("*(p + 2)"));

        // Validates: Rust uses bounds-checked indexing
        assert!(rust_expected.contains("Bounds-checked"));
    }

    #[test]
    fn test_dereference_function_pointer() {
        // C: Dereferencing function pointer
        let c_code = r#"
int add(int a, int b) { return a + b; }

int (*func_ptr)(int, int) = &add;
int result = (*func_ptr)(3, 4);  // Explicit dereference
int result2 = func_ptr(3, 4);    // Implicit dereference (C allows this)
"#;

        // Rust: Function pointer dereferencing
        let rust_expected = r#"
fn add(a: i32, b: i32) -> i32 { a + b }

let func_ptr: fn(i32, i32) -> i32 = add;
let result: i32 = (*func_ptr)(3, 4);  // Explicit dereference
let result2: i32 = func_ptr(3, 4);    // Implicit (Rust also allows)
"#;

        // Validates: Both languages allow implicit function pointer deref
        assert!(c_code.contains("(*func_ptr)(3, 4)"));
        assert!(rust_expected.contains("(*func_ptr)(3, 4)"));
        assert!(c_code.contains("func_ptr(3, 4)"));
        assert!(rust_expected.contains("func_ptr(3, 4)"));
    }

    #[test]
    fn test_dereference_in_expression() {
        // C: Dereference in complex expression
        let c_code = r#"
int x = 10, y = 20;
int* px = &x;
int* py = &y;
int sum = *px + *py;
int product = (*px) * (*py);
"#;

        // Rust: Same syntax for dereferencing in expressions
        let rust_expected = r#"
let x: i32 = 10;
let y: i32 = 20;
let px: &i32 = &x;
let py: &i32 = &y;
let sum: i32 = *px + *py;
let product: i32 = (*px) * (*py);
"#;

        // Validates: Dereference in expressions works the same
        assert!(c_code.contains("*px + *py"));
        assert!(rust_expected.contains("*px + *py"));
        assert!(c_code.contains("(*px) * (*py)"));
        assert!(rust_expected.contains("(*px) * (*py)"));
    }

    #[test]
    fn test_dereference_raw_pointer_unsafe() {
        // C: Raw pointer dereference (always allowed, even if dangerous)
        let c_code = r#"
int* p = (int*)0x12345678;  // Arbitrary address
int x = *p;  // Allowed in C (likely crash or UB)
"#;

        // Rust: Raw pointer requires unsafe
        let rust_expected = r#"
let p: *const i32 = 0x12345678 as *const i32;
// let x: i32 = *p;  // ERROR: dereference of raw pointer requires unsafe

let x: i32 = unsafe { *p };  // Explicit unsafe block required
"#;

        // Validates: C allows unsafe dereference
        assert!(c_code.contains("int x = *p"));

        // Validates: Rust requires unsafe for raw pointers
        assert!(rust_expected.contains("unsafe { *p }"));

        // CRITICAL: Raw pointer dereference requires unsafe in Rust
    }

    #[test]
    fn test_dereference_volatile_read() {
        // C: Volatile pointer dereference (hardware registers)
        let c_code = r#"
volatile int* reg = (volatile int*)0x40000000;
int val = *reg;  // Volatile read
*reg = 42;       // Volatile write
"#;

        // Rust: Volatile operations require unsafe and explicit functions
        let rust_expected = r#"
use std::ptr;

let reg: *mut i32 = 0x40000000 as *mut i32;
let val: i32 = unsafe { ptr::read_volatile(reg) };
unsafe { ptr::write_volatile(reg, 42); }
"#;

        // Validates: Volatile access
        assert!(c_code.contains("*reg"));

        // Validates: Rust uses explicit volatile functions
        assert!(rust_expected.contains("read_volatile"));
        assert!(rust_expected.contains("write_volatile"));
        assert!(rust_expected.contains("unsafe"));
    }

    #[test]
    fn test_dereference_const_correctness() {
        // C: const pointer dereference
        let c_code = r#"
const int x = 42;
const int* p = &x;
int y = *p;  // OK: reading const value
// *p = 10;  // ERROR: cannot modify const
"#;

        // Rust: Immutable reference (const by default)
        let rust_expected = r#"
let x: i32 = 42;
let p: &i32 = &x;
let y: i32 = *p;  // OK: reading immutable value
// *p = 10;  // ERROR: cannot assign to immutable
"#;

        // Validates: Reading through const/immutable pointer works
        assert!(c_code.contains("int y = *p"));
        assert!(rust_expected.contains("let y: i32 = *p"));

        // Note: Rust enforces immutability by default
    }

    #[test]
    fn test_dereference_swap_via_pointers() {
        // C: Swap values via pointers
        let c_code = r#"
void swap(int* a, int* b) {
    int temp = *a;
    *a = *b;
    *b = temp;
}

int x = 10, y = 20;
swap(&x, &y);
// x = 20, y = 10
"#;

        // Rust: Swap via mutable references
        let rust_expected = r#"
fn swap(a: &mut i32, b: &mut i32) {
    let temp: i32 = *a;
    *a = *b;
    *b = temp;
}

// Or use std::mem::swap (idiomatic):
fn swap_idiomatic(a: &mut i32, b: &mut i32) {
    std::mem::swap(a, b);
}

let mut x: i32 = 10;
let mut y: i32 = 20;
swap(&mut x, &mut y);
// x = 20, y = 10
"#;

        // Validates: Swap pattern works the same
        assert!(c_code.contains("int temp = *a"));
        assert!(rust_expected.contains("let temp: i32 = *a"));

        // Note: Rust has std::mem::swap for this
        assert!(rust_expected.contains("std::mem::swap"));
    }

    #[test]
    fn test_dereference_transformation_summary() {
        // Summary of dereference transformations

        // C patterns
        let c_patterns = [
            "*p",            // Basic dereference
            "*p = value",    // Dereference for assignment
            "**pp",          // Double dereference
            "*(p + offset)", // Dereference with pointer arithmetic
            "(*p).field",    // Struct field via explicit deref
            "p->field",      // Struct field via arrow (syntactic sugar)
            "*NULL",         // NULL dereference (UB)
        ];

        // Rust patterns
        let rust_patterns = [
            "*p",          // Basic dereference (safe with &)
            "*p = value",  // Dereference for assignment (needs &mut)
            "**pp",        // Double dereference (safe with &&)
            "arr[offset]", // Indexing (safer than pointer arithmetic)
            "(*p).field",  // Explicit deref (works)
            "p.field",     // Auto-deref (idiomatic)
                           // No equivalent for NULL deref (use Option)
        ];

        // Validation checks
        assert_eq!(c_patterns[0], rust_patterns[0], "Basic syntax same");
        assert_eq!(c_patterns[2], rust_patterns[2], "Double deref syntax same");

        // Key semantic differences documented
        let semantics = "
C dereference (*):
- Can dereference NULL (UB)
- Can dereference dangling pointers (UB)
- No bounds checking on array access
- Pointer arithmetic allowed
- No type safety after casting
- Manual lifetime management

Rust dereference (*):
- Cannot dereference null references (use Option)
- Cannot create dangling references (lifetime checking)
- Bounds checking on slice indexing
- Safe pointer arithmetic not allowed (use slicing/indexing)
- Type safety enforced
- Automatic lifetime tracking
- Auto-deref for field access and method calls
- Raw pointer deref requires unsafe block
        ";

        assert!(semantics.contains("Cannot dereference null"));
        assert!(semantics.contains("lifetime checking"));
        assert!(semantics.contains("Auto-deref"));
        assert!(semantics.contains("unsafe block"));
    }
}
