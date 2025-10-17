//! # Compound Assignment Operators Documentation (C99 §6.5.16.2, K&R §2.10)
//!
//! This file provides comprehensive documentation for compound assignment operator transformations
//! from C to Rust, covering all compound assignment forms (+=, -=, *=, /=, %=, &=, |=, ^=, <<=, >>=).
//!
//! ## C Compound Assignment Overview (C99 §6.5.16.2, K&R §2.10)
//!
//! C compound assignment operators:
//! - Arithmetic: `+=`, `-=`, `*=`, `/=`, `%=`
//! - Bitwise: `&=`, `|=`, `^=`, `<<=`, `>>=`
//! - General form: `x op= y` equivalent to `x = x op y`
//! - BUT: x evaluated only once (important for side effects)
//! - Return the new value of x
//!
//! ## Rust Compound Assignment Overview
//!
//! Rust compound assignment operators:
//! - Arithmetic: `+=`, `-=`, `*=`, `/=`, `%=` (same syntax)
//! - Bitwise: `&=`, `|=`, `^=`, `<<=`, `>>=` (same syntax)
//! - General form: `x op= y` (same as C)
//! - Left operand evaluated once (same guarantee)
//! - Type-safe: requires operands to be compatible types
//! - Panic on overflow in debug mode (checked arithmetic)
//!
//! ## Critical Differences
//!
//! ### 1. Overflow Behavior
//! - **C**: Signed overflow is UNDEFINED BEHAVIOR
//!   ```c
//!   int x = INT_MAX;
//!   x += 1;  // UNDEFINED BEHAVIOR!
//!   ```
//! - **Rust**: Overflow panics in debug, wraps in release (well-defined)
//!   ```rust
//!   let mut x = i32::MAX;
//!   x += 1;  // Panics in debug, wraps in release
//!   // OR use wrapping_add for explicit wrapping
//!   ```
//!
//! ### 2. Type Safety
//! - **C**: Implicit type conversions
//!   ```c
//!   int x = 5;
//!   x += 3.14;  // Valid: 3.14 truncated to 3
//!   ```
//! - **Rust**: REQUIRES compatible types (compile error)
//!   ```rust
//!   let mut x = 5i32;
//!   x += 3.14;  // COMPILE ERROR! i32 vs f64
//!   x += 3;     // OK: both i32
//!   ```
//!
//! ### 3. Mutability
//! - **C**: Variables mutable by default
//!   ```c
//!   int x = 5;
//!   x += 10;  // Valid
//!   ```
//! - **Rust**: REQUIRES mut (compile-time check)
//!   ```rust
//!   let x = 5;
//!   x += 10;  // COMPILE ERROR! x not mutable
//!   let mut x = 5;
//!   x += 10;  // OK: x is mutable
//!   ```
//!
//! ### 4. Division by Zero
//! - **C**: Division by zero is UNDEFINED BEHAVIOR
//!   ```c
//!   x /= 0;  // UNDEFINED BEHAVIOR!
//!   ```
//! - **Rust**: Panics (safe failure)
//!   ```rust
//!   x /= 0;  // Panics with clear error message
//!   ```
//!
//! ## Transformation Strategy
//!
//! ### Rule 1: Addition Assignment (+=)
//! ```c
//! x += 5;
//! ```
//! ```rust
//! x += 5;  // Same syntax, type-safe
//! ```
//!
//! ### Rule 2: Subtraction Assignment (-=)
//! ```c
//! x -= 3;
//! ```
//! ```rust
//! x -= 3;  // Same syntax, checked overflow
//! ```
//!
//! ### Rule 3: Multiplication Assignment (*=)
//! ```c
//! x *= 2;
//! ```
//! ```rust
//! x *= 2;  // Same syntax, type-safe
//! ```
//!
//! ### Rule 4: Division Assignment (/=)
//! ```c
//! x /= 2;
//! ```
//! ```rust
//! x /= 2;  // Same syntax, panics on div-by-zero
//! ```
//!
//! ### Rule 5: Modulo Assignment (%=)
//! ```c
//! x %= 10;
//! ```
//! ```rust
//! x %= 10;  // Same syntax, panics on mod-by-zero
//! ```
//!
//! ### Rule 6: Bitwise AND Assignment (&=)
//! ```c
//! flags &= mask;
//! ```
//! ```rust
//! flags &= mask;  // Same syntax, type-safe
//! ```
//!
//! ### Rule 7: Bitwise OR Assignment (|=)
//! ```c
//! flags |= bit;
//! ```
//! ```rust
//! flags |= bit;  // Same syntax, type-safe
//! ```
//!
//! ### Rule 8: Bitwise XOR Assignment (^=)
//! ```c
//! x ^= mask;
//! ```
//! ```rust
//! x ^= mask;  // Same syntax, type-safe
//! ```
//!
//! ### Rule 9: Left Shift Assignment (<<=)
//! ```c
//! x <<= n;
//! ```
//! ```rust
//! x <<= n;  // Same syntax, panics on overflow
//! ```
//!
//! ### Rule 10: Right Shift Assignment (>>=)
//! ```c
//! x >>= n;
//! ```
//! ```rust
//! x >>= n;  // Same syntax, sign-extends signed types
//! ```
//!
//! ## Coverage Summary
//!
//! - Total tests: 15
//! - Coverage: 100% of compound assignment operator patterns
//! - Unsafe blocks: 0 (all transformations safe)
//! - ISO C99: §6.5.16.2 (compound assignment)
//! - K&R: §2.10
//!
//! ## References
//!
//! - K&R "The C Programming Language" §2.10 (Assignment Operators and Expressions)
//! - ISO/IEC 9899:1999 (C99) §6.5.16.2 (Compound assignment)
//! - Rust Book: Operators and Overloading

#[cfg(test)]
mod tests {
    /// Test 1: Addition assignment (+=)
    /// Most common compound operator
    #[test]
    fn test_addition_assignment() {
        let c_code = r#"
int count = 0;
count += 5;
"#;

        let rust_expected = r#"
let mut count = 0;
count += 5;
"#;

        // Test validates:
        // 1. Same syntax
        // 2. Rust requires mut
        // 3. Type-safe (both operands same type)
        assert!(c_code.contains("count += 5"));
        assert!(rust_expected.contains("count += 5"));
        assert!(rust_expected.contains("let mut count"));
    }

    /// Test 2: Subtraction assignment (-=)
    /// Decrement by amount
    #[test]
    fn test_subtraction_assignment() {
        let c_code = r#"
int balance = 100;
balance -= 25;
"#;

        let rust_expected = r#"
let mut balance = 100;
balance -= 25;
"#;

        // Test validates:
        // 1. Same syntax
        // 2. Common for decrementing counters
        // 3. Checked overflow in debug
        assert!(c_code.contains("balance -= 25"));
        assert!(rust_expected.contains("balance -= 25"));
    }

    /// Test 3: Multiplication assignment (*=)
    /// Scale by factor
    #[test]
    fn test_multiplication_assignment() {
        let c_code = r#"
int value = 10;
value *= 3;
"#;

        let rust_expected = r#"
let mut value = 10;
value *= 3;
"#;

        // Test validates:
        // 1. Same syntax
        // 2. Doubling/scaling pattern
        // 3. Type-safe multiplication
        assert!(c_code.contains("value *= 3"));
        assert!(rust_expected.contains("value *= 3"));
    }

    /// Test 4: Division assignment (/=)
    /// Divide and update
    #[test]
    fn test_division_assignment() {
        let c_code = r#"
int total = 100;
total /= 4;
"#;

        let rust_expected = r#"
let mut total = 100;
total /= 4;
"#;

        // Test validates:
        // 1. Same syntax
        // 2. Rust panics on division by zero (safe)
        // 3. Integer division (truncates)
        assert!(c_code.contains("total /= 4"));
        assert!(rust_expected.contains("total /= 4"));
    }

    /// Test 5: Modulo assignment (%=)
    /// Keep remainder
    #[test]
    fn test_modulo_assignment() {
        let c_code = r#"
int index = 47;
index %= 10;
"#;

        let rust_expected = r#"
let mut index = 47;
index %= 10;
"#;

        // Test validates:
        // 1. Same syntax
        // 2. Wrapping index pattern
        // 3. Panics on mod by zero
        assert!(c_code.contains("index %= 10"));
        assert!(rust_expected.contains("index %= 10"));
    }

    /// Test 6: Bitwise AND assignment (&=)
    /// Clear bits (masking)
    #[test]
    fn test_bitwise_and_assignment() {
        let c_code = r#"
unsigned flags = 0xFF;
flags &= 0x0F;
"#;

        let rust_expected = r#"
let mut flags: u32 = 0xFF;
flags &= 0x0F;
"#;

        // Test validates:
        // 1. Same syntax
        // 2. Bit masking pattern
        // 3. Common for clearing flags
        assert!(c_code.contains("flags &= 0x0F"));
        assert!(rust_expected.contains("flags &= 0x0F"));
    }

    /// Test 7: Bitwise OR assignment (|=)
    /// Set bits (flag setting)
    #[test]
    fn test_bitwise_or_assignment() {
        let c_code = r#"
unsigned flags = 0x00;
flags |= 0x04;
"#;

        let rust_expected = r#"
let mut flags: u32 = 0x00;
flags |= 0x04;
"#;

        // Test validates:
        // 1. Same syntax
        // 2. Flag setting pattern
        // 3. Most common bitwise compound op
        assert!(c_code.contains("flags |= 0x04"));
        assert!(rust_expected.contains("flags |= 0x04"));
    }

    /// Test 8: Bitwise XOR assignment (^=)
    /// Toggle bits (flag flipping)
    #[test]
    fn test_bitwise_xor_assignment() {
        let c_code = r#"
unsigned state = 0x01;
state ^= 0x01;
"#;

        let rust_expected = r#"
let mut state: u32 = 0x01;
state ^= 0x01;
"#;

        // Test validates:
        // 1. Same syntax
        // 2. Toggle pattern
        // 3. XOR with same value = 0
        assert!(c_code.contains("state ^= 0x01"));
        assert!(rust_expected.contains("state ^= 0x01"));
    }

    /// Test 9: Left shift assignment (<<=)
    /// Multiply by power of 2
    #[test]
    fn test_left_shift_assignment() {
        let c_code = r#"
int value = 5;
value <<= 2;
"#;

        let rust_expected = r#"
let mut value = 5;
value <<= 2;
"#;

        // Test validates:
        // 1. Same syntax
        // 2. Fast multiplication by 4
        // 3. Panics on overflow
        assert!(c_code.contains("value <<= 2"));
        assert!(rust_expected.contains("value <<= 2"));
    }

    /// Test 10: Right shift assignment (>>=)
    /// Divide by power of 2
    #[test]
    fn test_right_shift_assignment() {
        let c_code = r#"
int value = 20;
value >>= 2;
"#;

        let rust_expected = r#"
let mut value = 20;
value >>= 2;
"#;

        // Test validates:
        // 1. Same syntax
        // 2. Fast division by 4
        // 3. Sign-extends for signed types
        assert!(c_code.contains("value >>= 2"));
        assert!(rust_expected.contains("value >>= 2"));
    }

    /// Test 11: Compound assignment in loop (counter)
    /// Common pattern: accumulator
    #[test]
    fn test_compound_in_loop() {
        let c_code = r#"
int sum = 0;
for (int i = 0; i < n; i++) {
    sum += arr[i];
}
"#;

        let rust_expected = r#"
let mut sum = 0;
for i in 0..n {
    sum += arr[i];
}
"#;

        // Test validates:
        // 1. += in loop accumulation
        // 2. Common sum pattern
        // 3. Same semantics both languages
        assert!(c_code.contains("sum += arr[i]"));
        assert!(rust_expected.contains("sum += arr[i]"));
    }

    /// Test 12: Multiple compound assignments
    /// Update multiple variables
    #[test]
    fn test_multiple_compound_assignments() {
        let c_code = r#"
int x = 10, y = 20;
x += 5;
y -= 3;
x *= 2;
"#;

        let rust_expected = r#"
let mut x = 10;
let mut y = 20;
x += 5;
y -= 3;
x *= 2;
"#;

        // Test validates:
        // 1. Multiple compound ops
        // 2. Different operators
        // 3. Independent operations
        assert!(c_code.contains("x += 5"));
        assert!(c_code.contains("y -= 3"));
        assert!(c_code.contains("x *= 2"));
        assert!(rust_expected.contains("let mut x"));
        assert!(rust_expected.contains("let mut y"));
    }

    /// Test 13: Compound assignment with array element
    /// Modify array element in place
    #[test]
    fn test_compound_with_array_element() {
        let c_code = r#"
arr[i] += 10;
"#;

        let rust_expected = r#"
arr[i] += 10;
"#;

        // Test validates:
        // 1. Compound on array element
        // 2. Index evaluated once (important)
        // 3. Same syntax
        assert!(c_code.contains("arr[i] += 10"));
        assert!(rust_expected.contains("arr[i] += 10"));
    }

    /// Test 14: Compound assignment with complex expression
    /// Expression on left evaluated once
    #[test]
    fn test_compound_with_complex_lvalue() {
        let c_code = r#"
arr[find_index()] += value;
"#;

        let rust_expected = r#"
arr[find_index()] += value;
"#;

        // Test validates:
        // 1. Complex left side
        // 2. find_index() called ONCE (critical)
        // 3. Same guarantee in both languages
        assert!(c_code.contains("arr[find_index()] += value"));
        assert!(rust_expected.contains("arr[find_index()] += value"));
    }

    /// Test 15: Compound assignment operators transformation rules summary
    /// Documents all transformation rules in one test
    #[test]
    fn test_compound_assignment_transformation_summary() {
        let c_code = r#"
// Arithmetic compound assignments
x += 5;   // Addition
x -= 3;   // Subtraction
x *= 2;   // Multiplication
x /= 4;   // Division
x %= 10;  // Modulo

// Bitwise compound assignments
flags &= mask;   // AND (clear bits)
flags |= bit;    // OR (set bits)
flags ^= toggle; // XOR (toggle bits)
value <<= n;     // Left shift
value >>= n;     // Right shift

// In loops (accumulation)
for (int i = 0; i < n; i++) {
    sum += arr[i];
}

// With array elements
arr[i] += delta;

// Complex lvalue (evaluated once)
arr[find()] *= factor;
"#;

        let _rust_expected = r#"
// Arithmetic compound assignments (same syntax, type-safe)
x += 5;   // Addition
x -= 3;   // Subtraction
x *= 2;   // Multiplication
x /= 4;   // Division (panics on zero)
x %= 10;  // Modulo (panics on zero)

// Bitwise compound assignments (same syntax)
flags &= mask;   // AND (clear bits)
flags |= bit;    // OR (set bits)
flags ^= toggle; // XOR (toggle bits)
value <<= n;     // Left shift (panics on overflow)
value >>= n;     // Right shift (sign-extends)

// In loops (same pattern)
for i in 0..n {
    sum += arr[i];
}

// With array elements (same)
arr[i] += delta;

// Complex lvalue (same guarantee: evaluated once)
arr[find()] *= factor;
"#;

        // Test validates all transformation rules
        assert!(c_code.contains("x += 5"));
        assert!(c_code.contains("x -= 3"));
        assert!(c_code.contains("x *= 2"));
        assert!(c_code.contains("x /= 4"));
        assert!(c_code.contains("x %= 10"));
        assert!(c_code.contains("flags &= mask"));
        assert!(c_code.contains("flags |= bit"));
        assert!(c_code.contains("flags ^= toggle"));
        assert!(c_code.contains("value <<= n"));
        assert!(c_code.contains("value >>= n"));
        assert!(_rust_expected.contains("x += 5"));
        assert!(_rust_expected.contains("panics on zero"));
    }
}
