//! Documentation tests for C address-of operator (&) → Rust borrowing
//!
//! This test file documents the transformation from C's address-of operator (&)
//! to Rust's safe borrowing system. While the syntax appears similar, the semantics
//! are fundamentally different:
//!
//! - **C**: `&x` creates a pointer to x's memory location
//! - **Rust**: `&x` creates a safe, lifetime-bound borrow of x
//!
//! # Reference
//! - K&R C (2nd Edition) §5.1: Pointers and Addresses
//! - ISO C99 Standard §6.5.3.2: Address-of operator
//!
//! # Safety Guarantees
//! Rust's borrowing system provides:
//! - No dangling pointers (lifetime checking)
//! - No data races (exclusive mutable or shared immutable)
//! - Type safety (cannot cast between incompatible types)
//!
//! # Transformation Strategy
//! 1. `&x` (C) → `&x` (Rust) for immutable borrows
//! 2. `&x` (C, later mutated) → `&mut x` (Rust) for mutable borrows
//! 3. Lifetime inference ensures safety
//! 4. No `unsafe` blocks needed for address-of in safe contexts
//!
//! # Target Metrics
//! - Coverage: 100%
//! - Unsafe blocks: 0
//! - Tests: 17 comprehensive scenarios

#[cfg(test)]
mod tests {
    //! All tests validate address-of operator transformation patterns

    #[test]
    fn test_address_of_basic_immutable() {
        // C: Taking address of a variable for read-only access
        let c_code = r#"
int x = 42;
int* p = &x;
printf("%d\n", *p);
"#;

        // Rust: Immutable borrow with explicit lifetime
        let rust_expected = r#"
let x: i32 = 42;
let p: &i32 = &x;
println!("{}", *p);
"#;

        // Validates: &x in C → &x in Rust (immutable borrow)
        assert!(c_code.contains("int* p = &x"));
        assert!(rust_expected.contains("let p: &i32 = &x"));

        // Semantic difference: Rust's & is a borrow, not a raw pointer
        assert!(
            rust_expected.contains("&i32"),
            "Rust uses reference type, not pointer"
        );
    }

    #[test]
    fn test_address_of_mutable() {
        // C: Taking address for mutation
        let c_code = r#"
int x = 10;
int* p = &x;
*p = 20;
"#;

        // Rust: Mutable borrow required for mutation
        let rust_expected = r#"
let mut x: i32 = 10;
let p: &mut i32 = &mut x;
*p = 20;
"#;

        // Validates: &x with mutation → &mut x in Rust
        assert!(c_code.contains("int* p = &x"));
        assert!(rust_expected.contains("let p: &mut i32 = &mut x"));

        // CRITICAL: Rust requires `mut` on both variable and borrow
        assert!(
            rust_expected.contains("let mut x"),
            "Variable must be mutable"
        );
        assert!(rust_expected.contains("&mut i32"), "Borrow must be mutable");
    }

    #[test]
    fn test_address_of_function_parameter() {
        // C: Passing address to function (call by reference)
        let c_code = r#"
void increment(int* p) {
    (*p)++;
}

int main() {
    int x = 5;
    increment(&x);
    return 0;
}
"#;

        // Rust: Mutable reference parameter
        let rust_expected = r#"
fn increment(p: &mut i32) {
    *p += 1;
}

fn main() {
    let mut x: i32 = 5;
    increment(&mut x);
}
"#;

        // Validates: Function parameter int* → &mut i32
        assert!(c_code.contains("void increment(int* p)"));
        assert!(rust_expected.contains("fn increment(p: &mut i32)"));

        // Validates: Call site &x → &mut x
        assert!(c_code.contains("increment(&x)"));
        assert!(rust_expected.contains("increment(&mut x)"));
    }

    #[test]
    fn test_address_of_array_element() {
        // C: Taking address of array element
        let c_code = r#"
int arr[5] = {1, 2, 3, 4, 5};
int* p = &arr[2];
*p = 99;
"#;

        // Rust: Mutable borrow of array element
        let rust_expected = r#"
let mut arr: [i32; 5] = [1, 2, 3, 4, 5];
let p: &mut i32 = &mut arr[2];
*p = 99;
"#;

        // Validates: &arr[i] → &mut arr[i]
        assert!(c_code.contains("&arr[2]"));
        assert!(rust_expected.contains("&mut arr[2]"));

        // Safety: Rust enforces bounds checking
        assert!(
            rust_expected.contains("[i32; 5]"),
            "Array type includes length"
        );
    }

    #[test]
    fn test_address_of_struct_member() {
        // C: Taking address of struct member
        let c_code = r#"
struct Point {
    int x;
    int y;
};

struct Point p = {10, 20};
int* px = &p.x;
*px = 30;
"#;

        // Rust: Mutable borrow of struct field
        let rust_expected = r#"
struct Point {
    x: i32,
    y: i32,
}

let mut p = Point { x: 10, y: 20 };
let px: &mut i32 = &mut p.x;
*px = 30;
"#;

        // Validates: &p.x → &mut p.x
        assert!(c_code.contains("&p.x"));
        assert!(rust_expected.contains("&mut p.x"));

        // Note: Rust borrows the field, not the whole struct
    }

    #[test]
    fn test_address_of_multiple_immutable_borrows() {
        // C: Multiple pointers to same variable (dangerous in C)
        let c_code = r#"
int x = 42;
int* p1 = &x;
int* p2 = &x;
printf("%d %d\n", *p1, *p2);
"#;

        // Rust: Multiple immutable borrows allowed (safe!)
        let rust_expected = r#"
let x: i32 = 42;
let p1: &i32 = &x;
let p2: &i32 = &x;
println!("{} {}", *p1, *p2);
"#;

        // Validates: Multiple immutable borrows are safe in Rust
        assert!(c_code.contains("int* p1 = &x"));
        assert!(c_code.contains("int* p2 = &x"));
        assert!(rust_expected.contains("let p1: &i32 = &x"));
        assert!(rust_expected.contains("let p2: &i32 = &x"));

        // CRITICAL: Rust allows multiple immutable borrows (aliasing is safe for reads)
    }

    #[test]
    fn test_address_of_no_mutable_aliasing() {
        // C: Multiple pointers with mutation (UNDEFINED BEHAVIOR in C!)
        let c_code = r#"
int x = 10;
int* p1 = &x;
int* p2 = &x;
*p1 = 20;  // UB: x is modified while p2 also points to it
*p2 = 30;  // UB
"#;

        // Rust: Cannot compile! Only one mutable borrow allowed
        let rust_compilation_error = r#"
// This code WILL NOT COMPILE in Rust
let mut x: i32 = 10;
let p1: &mut i32 = &mut x;
let p2: &mut i32 = &mut x;  // ERROR: cannot borrow `x` as mutable more than once
*p1 = 20;
*p2 = 30;
"#;

        // Rust: Safe version (sequential borrows)
        let rust_safe = r#"
let mut x: i32 = 10;
{
    let p1: &mut i32 = &mut x;
    *p1 = 20;
}  // p1 goes out of scope here
{
    let p2: &mut i32 = &mut x;
    *p2 = 30;
}
"#;

        // Validates: C allows dangerous aliasing
        assert!(c_code.contains("int* p1 = &x"));
        assert!(c_code.contains("int* p2 = &x"));

        // Validates: Rust prevents aliasing at compile time
        assert!(rust_compilation_error.contains("cannot borrow"));

        // Validates: Rust safe version uses scopes
        assert!(rust_safe.contains("{"));
    }

    #[test]
    fn test_address_of_return_local_dangling_pointer() {
        // C: Returning address of local variable (UNDEFINED BEHAVIOR!)
        let c_code = r#"
int* dangerous() {
    int x = 42;
    return &x;  // BUG: x is destroyed after function returns
}
"#;

        // Rust: Cannot compile! Lifetime error
        let rust_compilation_error = r#"
// This code WILL NOT COMPILE in Rust
fn dangerous() -> &i32 {
    let x: i32 = 42;
    &x  // ERROR: `x` does not live long enough
}
"#;

        // Rust: Safe version (return owned value)
        let rust_safe = r#"
fn safe() -> i32 {
    let x: i32 = 42;
    x  // Move ownership out
}

// Or use Box for heap allocation
fn safe_boxed() -> Box<i32> {
    Box::new(42)
}
"#;

        // Validates: C code compiles but has UB
        assert!(c_code.contains("return &x"));

        // Validates: Rust prevents dangling pointers
        assert!(rust_compilation_error.contains("does not live long enough"));

        // Validates: Rust safe version moves or boxes
        assert!(rust_safe.contains("x  // Move ownership"));
        assert!(rust_safe.contains("Box::new(42)"));
    }

    #[test]
    fn test_address_of_with_lifetime_annotations() {
        // C: Function returning pointer to parameter
        let c_code = r#"
int* get_larger(int* a, int* b) {
    return (*a > *b) ? a : b;
}
"#;

        // Rust: Explicit lifetime annotation
        let rust_expected = r#"
fn get_larger<'a>(a: &'a i32, b: &'a i32) -> &'a i32 {
    if *a > *b { a } else { b }
}
"#;

        // Validates: Return type matches parameter lifetimes
        assert!(c_code.contains("int* get_larger(int* a, int* b)"));
        assert!(rust_expected.contains("fn get_larger<'a>(a: &'a i32, b: &'a i32) -> &'a i32"));

        // CRITICAL: Lifetime 'a ensures returned reference is valid
        assert!(
            rust_expected.contains("<'a>"),
            "Lifetime parameter required"
        );
    }

    #[test]
    fn test_address_of_const_pointer() {
        // C: Pointer to const
        let c_code = r#"
const int x = 42;
const int* p = &x;
// *p = 10;  // ERROR in C
printf("%d\n", *p);
"#;

        // Rust: Immutable borrow (const is default)
        let rust_expected = r#"
let x: i32 = 42;
let p: &i32 = &x;
// *p = 10;  // ERROR in Rust
println!("{}", *p);
"#;

        // Validates: const in C → default immutability in Rust
        assert!(c_code.contains("const int* p = &x"));
        assert!(rust_expected.contains("let p: &i32 = &x"));

        // Note: Rust doesn't need `const` keyword for immutable borrows
    }

    #[test]
    fn test_address_of_double_pointer() {
        // C: Pointer to pointer
        let c_code = r#"
int x = 10;
int* p = &x;
int** pp = &p;
**pp = 20;
"#;

        // Rust: Mutable reference to mutable reference
        let rust_expected = r#"
let mut x: i32 = 10;
let mut p: &mut i32 = &mut x;
let pp: &mut &mut i32 = &mut p;
**pp = 20;
"#;

        // Validates: int** → &mut &mut i32
        assert!(c_code.contains("int** pp = &p"));
        assert!(rust_expected.contains("let pp: &mut &mut i32 = &mut p"));

        // Note: Double dereferencing works the same
        assert!(c_code.contains("**pp"));
        assert!(rust_expected.contains("**pp"));
    }

    #[test]
    fn test_address_of_in_conditional() {
        // C: Conditional address-of
        let c_code = r#"
int x = 10, y = 20;
int* p = (x > y) ? &x : &y;
*p = 99;
"#;

        // Rust: Conditional mutable borrow
        let rust_expected = r#"
let mut x: i32 = 10;
let mut y: i32 = 20;
let p: &mut i32 = if x > y { &mut x } else { &mut y };
*p = 99;
"#;

        // Validates: Ternary with & → if expression with &mut
        assert!(c_code.contains("? &x : &y"));
        assert!(rust_expected.contains("if x > y { &mut x } else { &mut y }"));
    }

    #[test]
    fn test_address_of_array_decay() {
        // C: Array decays to pointer
        let c_code = r#"
int arr[5] = {1, 2, 3, 4, 5};
int* p = arr;  // Implicit &arr[0]
int* p2 = &arr[0];  // Explicit
"#;

        // Rust: Slice reference or pointer
        let rust_expected = r#"
let arr: [i32; 5] = [1, 2, 3, 4, 5];
let p: &[i32] = &arr;  // Slice reference
let p2: &i32 = &arr[0];  // Element reference
"#;

        // Validates: Array decay → slice reference
        assert!(c_code.contains("int* p = arr"));
        assert!(rust_expected.contains("let p: &[i32] = &arr"));

        // Validates: Explicit &arr[0] works the same
        assert!(c_code.contains("&arr[0]"));
        assert!(rust_expected.contains("&arr[0]"));
    }

    #[test]
    fn test_address_of_string_literal() {
        // C: Address of string literal
        let c_code = r#"
const char* s = "hello";
const char* p = &s[0];
"#;

        // Rust: String slice and byte reference
        let rust_expected = r#"
let s: &str = "hello";
let p: &u8 = &s.as_bytes()[0];
"#;

        // Validates: String literal pointer → &str
        assert!(c_code.contains("const char* s = \"hello\""));
        assert!(rust_expected.contains("let s: &str = \"hello\""));

        // Note: Character access requires as_bytes() in Rust
        assert!(rust_expected.contains("as_bytes()"));
    }

    #[test]
    fn test_address_of_volatile_access() {
        // C: Address of volatile (hardware register access)
        let c_code = r#"
volatile int* reg = (volatile int*)0x40000000;
int val = *reg;
*reg = 42;
"#;

        // Rust: Volatile reads/writes via std::ptr
        let rust_expected = r#"
use std::ptr;

let reg: *mut i32 = 0x40000000 as *mut i32;
unsafe {
    let val = ptr::read_volatile(reg);
    ptr::write_volatile(reg, 42);
}
"#;

        // Validates: volatile pointer → unsafe volatile operations
        assert!(c_code.contains("volatile int*"));
        assert!(rust_expected.contains("read_volatile"));
        assert!(rust_expected.contains("write_volatile"));

        // Note: This is one case where unsafe is needed
        assert!(rust_expected.contains("unsafe {"));
    }

    #[test]
    fn test_address_of_null_pointer_check() {
        // C: Check if pointer is NULL before dereferencing
        let c_code = r#"
int* p = get_value();
if (p != NULL) {
    printf("%d\n", *p);
}
"#;

        // Rust: Option type for nullable pointers
        let rust_expected = r#"
let p: Option<&i32> = get_value();
if let Some(val) = p {
    println!("{}", *val);
}
"#;

        // Validates: NULL check → Option pattern match
        assert!(c_code.contains("if (p != NULL)"));
        assert!(rust_expected.contains("if let Some(val) = p"));

        // CRITICAL: Rust enforces NULL safety at compile time via Option
        assert!(rust_expected.contains("Option<&i32>"));
    }

    #[test]
    fn test_address_of_transformation_summary() {
        // Summary of address-of transformations

        // C patterns
        let c_patterns = [
            "&x",            // Basic address-of
            "int* p = &x",   // Pointer initialization
            "&arr[i]",       // Array element address
            "&s.field",      // Struct field address
            "return &local", // Dangling pointer (UB)
            "int** pp",      // Pointer to pointer
        ];

        // Rust patterns
        let rust_patterns = [
            "&x",       // Immutable borrow
            "&mut x",   // Mutable borrow
            "&arr[i]",  // Element reference
            "&s.field", // Field reference
            // No equivalent for dangling (compile error)
            "&mut &mut T", // Reference to reference
        ];

        // Validation checks
        assert_eq!(
            c_patterns[0], rust_patterns[0],
            "Syntax similar but semantics differ"
        );
        assert!(
            rust_patterns[1].contains("mut"),
            "Rust requires explicit mutability"
        );

        // Key semantic differences documented
        let semantics = "
C address-of (&):
- Creates raw pointer to memory location
- No lifetime tracking
- Allows aliasing with mutation (UB risk)
- Can create dangling pointers
- Manual memory management

Rust borrowing (&, &mut):
- Creates lifetime-bound reference
- Compile-time lifetime checking
- Enforces exclusive mutable XOR shared immutable
- Prevents dangling references at compile time
- Automatic memory management via RAII
        ";

        assert!(semantics.contains("lifetime-bound reference"));
        assert!(semantics.contains("compile time"));
    }
}
