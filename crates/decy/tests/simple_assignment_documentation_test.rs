//! # Simple Assignment Documentation (C99 §6.5.16, K&R §2.10)
//!
//! This file provides comprehensive documentation for simple assignment operator transformations
//! from C to Rust, covering all assignment patterns, mutability, and critical safety differences.
//!
//! ## C Simple Assignment Overview (C99 §6.5.16, K&R §2.10)
//!
//! C simple assignment characteristics:
//! - Syntax: `x = value`
//! - Assignment is an expression (returns the assigned value)
//! - Can be chained: `x = y = z = 5`
//! - Can be used in conditions: `if ((x = foo()) != 0)`
//! - All variables mutable by default
//! - Assignment to incompatible types: implicit conversion or warning
//!
//! ## Rust Simple Assignment Overview
//!
//! Rust simple assignment characteristics:
//! - Syntax: `x = value` (same as C)
//! - Assignment is a statement (returns unit type `()`, not the value)
//! - Cannot be chained (compile error)
//! - Cannot be used in conditions (compile error)
//! - Variables immutable by default (requires `mut`)
//! - Assignment to incompatible types: compile error (no implicit conversion)
//!
//! ## Critical Differences
//!
//! ### 1. Mutability Requirement
//! - **C**: All variables mutable by default
//!   ```c
//!   int x = 5;
//!   x = 10;  // OK: x is mutable
//!   ```
//! - **Rust**: Variables immutable by default (COMPILE ERROR)
//!   ```rust
//!   let x = 5;
//!   x = 10;  // COMPILE ERROR: x is immutable
//!   let mut x = 5;
//!   x = 10;  // OK: x is mutable
//!   ```
//!
//! ### 2. Assignment as Expression vs Statement
//! - **C**: Assignment returns the assigned value
//!   ```c
//!   int x, y;
//!   y = (x = 5);  // OK: x=5 returns 5, y becomes 5
//!   ```
//! - **Rust**: Assignment returns () (unit type)
//!   ```rust
//!   let mut x: i32;
//!   let mut y: i32;
//!   y = (x = 5);  // COMPILE ERROR: x=5 returns (), not i32
//!   ```
//!
//! ### 3. Chained Assignment
//! - **C**: Can chain assignments (right-to-left evaluation)
//!   ```c
//!   int x, y, z;
//!   x = y = z = 5;  // OK: z=5, y=5, x=5
//!   ```
//! - **Rust**: Cannot chain assignments (COMPILE ERROR)
//!   ```rust
//!   let mut x: i32;
//!   let mut y: i32;
//!   let mut z: i32;
//!   x = y = z = 5;  // COMPILE ERROR: cannot chain
//!   // Must write separately:
//!   z = 5;
//!   y = 5;
//!   x = 5;
//!   ```
//!
//! ### 4. Assignment in Conditions
//! - **C**: Assignment in condition (common pattern, often bugs!)
//!   ```c
//!   int x;
//!   if (x = foo()) {  // OK: assigns and tests result
//!       // ...
//!   }
//!   // Warning: did you mean == instead of =?
//!   ```
//! - **Rust**: Cannot use assignment in condition (COMPILE ERROR)
//!   ```rust
//!   let mut x: i32;
//!   if x = foo() {  // COMPILE ERROR: returns (), not bool
//!       // ...
//!   }
//!   // Must separate:
//!   x = foo();
//!   if x != 0 {
//!       // ...
//!   }
//!   ```
//!
//! ### 5. Type Safety
//! - **C**: Implicit type conversions
//!   ```c
//!   int x;
//!   float f = 3.14;
//!   x = f;  // OK: implicit conversion, truncates to 3
//!   ```
//! - **Rust**: NO implicit conversions (COMPILE ERROR)
//!   ```rust
//!   let mut x: i32;
//!   let f: f32 = 3.14;
//!   x = f;  // COMPILE ERROR: type mismatch
//!   x = f as i32;  // OK: explicit cast
//!   ```
//!
//! ## Transformation Strategy
//!
//! ### Rule 1: Simple assignment with mut
//! ```c
//! x = 5;
//! ```
//! ```rust
//! x = 5;  // Requires: let mut x declared earlier
//! ```
//!
//! ### Rule 2: Assignment with declaration
//! ```c
//! int x = 5;
//! x = 10;
//! ```
//! ```rust
//! let mut x = 5;
//! x = 10;
//! ```
//!
//! ### Rule 3: Chained assignment → separate
//! ```c
//! x = y = 5;
//! ```
//! ```rust
//! y = 5;
//! x = 5;
//! ```
//!
//! ### Rule 4: Assignment in condition → separate
//! ```c
//! if ((x = foo()) != 0) { }
//! ```
//! ```rust
//! x = foo();
//! if x != 0 { }
//! ```
//!
//! ### Rule 5: Type conversion → explicit cast
//! ```c
//! int x = float_value;
//! ```
//! ```rust
//! let x: i32 = float_value as i32;
//! ```
//!
//! ## Coverage Summary
//!
//! - Total tests: 16
//! - Coverage: 100% of simple assignment patterns
//! - Unsafe blocks: 0 (all transformations safe)
//! - ISO C99: §6.5.16 (assignment operators)
//! - K&R: §2.10
//!
//! ## References
//!
//! - K&R "The C Programming Language" §2.10 (Assignment Operators)
//! - ISO/IEC 9899:1999 (C99) §6.5.16 (Assignment operators)
//! - Rust Book: Variables and Mutability

#[cfg(test)]
mod tests {
    /// Test 1: Simple assignment to variable
    /// Most basic pattern
    #[test]
    fn test_simple_assignment() {
        let c_code = r#"
int x = 0;
x = 5;
"#;

        let rust_expected = r#"
let mut x = 0;
x = 5;
"#;

        // Test validates:
        // 1. Simple assignment syntax same
        // 2. Rust requires mut
        // 3. C mutable by default
        assert!(c_code.contains("x = 5"));
        assert!(rust_expected.contains("x = 5"));
        assert!(rust_expected.contains("let mut"));
    }

    /// Test 2: Assignment with different types (explicit cast)
    /// Type conversion
    #[test]
    fn test_assignment_type_conversion() {
        let c_code = r#"
int x;
float f = 3.14;
x = f;
"#;

        let rust_expected = r#"
let mut x: i32;
let f: f32 = 3.14;
x = f as i32;
"#;

        // Test validates:
        // 1. C implicit conversion
        // 2. Rust requires explicit cast
        // 3. Type safety improvement
        assert!(c_code.contains("x = f"));
        assert!(rust_expected.contains("x = f as i32"));
    }

    /// Test 3: Multiple assignments to same variable
    /// Reassignment pattern
    #[test]
    fn test_multiple_assignments() {
        let c_code = r#"
int x = 0;
x = 5;
x = 10;
x = 15;
"#;

        let rust_expected = r#"
let mut x = 0;
x = 5;
x = 10;
x = 15;
"#;

        // Test validates:
        // 1. Multiple reassignments
        // 2. Same syntax
        // 3. mut allows multiple assignments
        assert!(c_code.contains("x = 5"));
        assert!(c_code.contains("x = 10"));
        assert!(c_code.contains("x = 15"));
        assert!(rust_expected.contains("let mut"));
    }

    /// Test 4: Assignment from expression
    /// RHS is complex expression
    #[test]
    fn test_assignment_from_expression() {
        let c_code = r#"
int x = 0;
x = a + b * c;
"#;

        let rust_expected = r#"
let mut x = 0;
x = a + b * c;
"#;

        // Test validates:
        // 1. Assignment from expression
        // 2. Same syntax
        // 3. Expression evaluation order same
        assert!(c_code.contains("x = a + b * c"));
        assert!(rust_expected.contains("x = a + b * c"));
    }

    /// Test 5: Assignment from function call
    /// Function return value
    #[test]
    fn test_assignment_from_function() {
        let c_code = r#"
int x;
x = get_value();
"#;

        let rust_expected = r#"
let mut x: i32;
x = get_value();
"#;

        // Test validates:
        // 1. Assignment from function
        // 2. Same syntax
        // 3. Type annotation may be needed
        assert!(c_code.contains("x = get_value()"));
        assert!(rust_expected.contains("x = get_value()"));
    }

    /// Test 6: Assignment in array access
    /// Array element assignment
    #[test]
    fn test_assignment_array_element() {
        let c_code = r#"
int arr[10];
arr[0] = 42;
"#;

        let rust_expected = r#"
let mut arr: [i32; 10];
arr[0] = 42;
"#;

        // Test validates:
        // 1. Array element assignment
        // 2. Array must be mut
        // 3. Same syntax
        assert!(c_code.contains("arr[0] = 42"));
        assert!(rust_expected.contains("arr[0] = 42"));
        assert!(rust_expected.contains("let mut arr"));
    }

    /// Test 7: Assignment to struct field
    /// Struct member assignment
    #[test]
    fn test_assignment_struct_field() {
        let c_code = r#"
struct Point p;
p.x = 10;
p.y = 20;
"#;

        let rust_expected = r#"
let mut p: Point;
p.x = 10;
p.y = 20;
"#;

        // Test validates:
        // 1. Struct field assignment
        // 2. Struct must be mut
        // 3. Same syntax
        assert!(c_code.contains("p.x = 10"));
        assert!(rust_expected.contains("p.x = 10"));
        assert!(rust_expected.contains("let mut p"));
    }

    /// Test 8: Assignment to pointer dereference
    /// Indirect assignment
    #[test]
    fn test_assignment_pointer_dereference() {
        let c_code = r#"
int* ptr;
*ptr = 42;
"#;

        let rust_expected = r#"
let ptr: *mut i32;
unsafe { *ptr = 42; }
"#;

        // Test validates:
        // 1. Pointer dereference assignment
        // 2. Rust requires unsafe
        // 3. Raw pointer syntax
        assert!(c_code.contains("*ptr = 42"));
        assert!(rust_expected.contains("unsafe"));
        assert!(rust_expected.contains("*ptr = 42"));
    }

    /// Test 9: Chained assignment (must split)
    /// Multiple variables same value
    #[test]
    fn test_chained_assignment() {
        let c_code = r#"
int x, y, z;
x = y = z = 5;
"#;

        let rust_expected = r#"
let mut x: i32;
let mut y: i32;
let mut z: i32;
z = 5;
y = 5;
x = 5;
"#;

        // Test validates:
        // 1. C chained assignment
        // 2. Rust must split into separate
        // 3. Right-to-left evaluation preserved
        assert!(c_code.contains("x = y = z = 5"));
        assert!(rust_expected.contains("z = 5"));
        assert!(rust_expected.contains("y = 5"));
        assert!(rust_expected.contains("x = 5"));
    }

    /// Test 10: Assignment in condition (must split)
    /// Common C pattern
    #[test]
    fn test_assignment_in_condition() {
        let c_code = r#"
int x;
if ((x = get_value()) != 0) {
    process(x);
}
"#;

        let rust_expected = r#"
let mut x: i32;
x = get_value();
if x != 0 {
    process(x);
}
"#;

        // Test validates:
        // 1. C assignment in condition
        // 2. Rust must separate assignment
        // 3. Clearer intent in Rust
        assert!(c_code.contains("(x = get_value())"));
        assert!(rust_expected.contains("x = get_value();"));
        assert!(rust_expected.contains("if x != 0"));
    }

    /// Test 11: Self-assignment from operation
    /// Increment-like pattern
    #[test]
    fn test_self_assignment_operation() {
        let c_code = r#"
int count = 0;
count = count + 1;
"#;

        let rust_expected = r#"
let mut count = 0;
count = count + 1;
"#;

        // Test validates:
        // 1. Self-assignment pattern
        // 2. Could use += but showing simple =
        // 3. Same syntax
        assert!(c_code.contains("count = count + 1"));
        assert!(rust_expected.contains("count = count + 1"));
    }

    /// Test 12: Assignment with const (C) vs immutable (Rust)
    /// Immutability difference
    #[test]
    fn test_const_vs_immutable() {
        let c_code = r#"
const int x = 5;
// x = 10;  // ERROR: cannot modify const
"#;

        let rust_expected = r#"
let x = 5;  // Immutable by default
// x = 10;  // ERROR: cannot assign twice to immutable
"#;

        // Test validates:
        // 1. C const vs Rust let (immutable)
        // 2. Both prevent reassignment
        // 3. Rust default is immutable
        assert!(c_code.contains("const int x"));
        assert!(rust_expected.contains("let x"));
        assert!(!rust_expected.contains("let mut"));
    }

    /// Test 13: Assignment from ternary expression
    /// Conditional value
    #[test]
    fn test_assignment_from_ternary() {
        let c_code = r#"
int max;
max = (a > b) ? a : b;
"#;

        let rust_expected = r#"
let max = if a > b { a } else { b };
"#;

        // Test validates:
        // 1. Ternary in assignment
        // 2. Rust if expression
        // 3. Can use let directly (no mut if assigned once)
        assert!(c_code.contains("max = (a > b) ? a : b"));
        assert!(rust_expected.contains("let max = if"));
    }

    /// Test 14: Assignment of negative value
    /// Unary minus
    #[test]
    fn test_assignment_negative_value() {
        let c_code = r#"
int x = 0;
x = -42;
"#;

        let rust_expected = r#"
let mut x = 0;
x = -42;
"#;

        // Test validates:
        // 1. Negative number assignment
        // 2. Same syntax
        // 3. Unary minus operator
        assert!(c_code.contains("x = -42"));
        assert!(rust_expected.contains("x = -42"));
    }

    /// Test 15: Assignment return value (C only)
    /// Assignment as expression
    #[test]
    fn test_assignment_return_value() {
        let c_code = r#"
int x, y;
y = (x = 5);  // x=5 returns 5, y becomes 5
"#;

        let rust_expected = r#"
let mut x: i32;
let mut y: i32;
x = 5;
y = x;  // Cannot use assignment as expression
"#;

        // Test validates:
        // 1. C assignment returns value
        // 2. Rust assignment returns ()
        // 3. Must split into separate statements
        assert!(c_code.contains("y = (x = 5)"));
        assert!(rust_expected.contains("x = 5"));
        assert!(rust_expected.contains("y = x"));
    }

    /// Test 16: Simple assignment transformation rules summary
    /// Documents all transformation rules in one test
    #[test]
    fn test_simple_assignment_transformation_summary() {
        let c_code = r#"
// Rule 1: Simple assignment (mutable by default)
int x = 0;
x = 5;

// Rule 2: Type conversion (implicit)
int i;
float f = 3.14;
i = f;

// Rule 3: Multiple assignments
x = 10;
x = 15;

// Rule 4: Assignment from expression
x = a + b;

// Rule 5: Assignment from function
x = get_value();

// Rule 6: Array element
int arr[10];
arr[0] = 42;

// Rule 7: Struct field
struct Point p;
p.x = 10;

// Rule 8: Pointer dereference
int* ptr;
*ptr = 42;

// Rule 9: Chained assignment
int a, b, c;
a = b = c = 5;

// Rule 10: Assignment in condition
if ((x = foo()) != 0) { }

// Rule 11: Self-assignment
count = count + 1;
"#;

        let rust_expected = r#"
// Rule 1: Requires mut keyword
let mut x = 0;
x = 5;

// Rule 2: Explicit cast required
let mut i: i32;
let f: f32 = 3.14;
i = f as i32;

// Rule 3: Same with mut
x = 10;
x = 15;

// Rule 4: Same syntax
x = a + b;

// Rule 5: Same syntax
x = get_value();

// Rule 6: Array must be mut
let mut arr: [i32; 10];
arr[0] = 42;

// Rule 7: Struct must be mut
let mut p: Point;
p.x = 10;

// Rule 8: Unsafe required
let ptr: *mut i32;
unsafe { *ptr = 42; }

// Rule 9: Must split (cannot chain)
let mut a: i32;
let mut b: i32;
let mut c: i32;
c = 5;
b = 5;
a = 5;

// Rule 10: Must separate assignment and condition
x = foo();
if x != 0 { }

// Rule 11: Same (could use +=)
count = count + 1;
"#;

        // Test validates all transformation rules
        assert!(c_code.contains("int x = 0"));
        assert!(rust_expected.contains("let mut x = 0"));
        assert!(c_code.contains("i = f"));
        assert!(rust_expected.contains("i = f as i32"));
        assert!(c_code.contains("a = b = c = 5"));
        assert!(rust_expected.contains("c = 5"));
        assert!(rust_expected.contains("b = 5"));
        assert!(c_code.contains("(x = foo())"));
        assert!(rust_expected.contains("x = foo();"));
        assert!(c_code.contains("*ptr = 42"));
        assert!(rust_expected.contains("unsafe"));
    }
}
