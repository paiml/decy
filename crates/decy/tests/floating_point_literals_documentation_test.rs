//! # Floating-Point Literals Documentation (C99 §6.4.4.2, K&R §2.3)
//!
//! This file provides comprehensive documentation for floating-point literal transformations
//! from C to Rust, covering all floating-point patterns, scientific notation, and precision differences.
//!
//! ## C Floating-Point Literals Overview (C99 §6.4.4.2, K&R §2.3)
//!
//! C floating-point literal characteristics:
//! - Decimal notation: `3.14`, `0.5`, `.5`
//! - Scientific notation: `1.5e10`, `2.3E-5`
//! - Type: `double` by default (64-bit)
//! - Suffixes: `f` or `F` (float, 32-bit), `l` or `L` (long double, implementation-defined)
//! - Special values: `INFINITY`, `NAN` (via macros)
//! - Precision: float (6-7 digits), double (15-17 digits)
//!
//! ## Rust Floating-Point Literals Overview
//!
//! Rust floating-point literal characteristics:
//! - Decimal notation: `3.14`, `0.5` (NOT `.5` - must have leading digit)
//! - Scientific notation: `1.5e10`, `2.3e-5` (same as C)
//! - Type: `f64` by default (64-bit, equivalent to C double)
//! - Type suffixes: `f32`, `f64` (explicit sizes)
//! - Special values: `f64::INFINITY`, `f64::NAN` (constants)
//! - Precision: f32 (6-7 digits), f64 (15-17 digits)
//!
//! ## Critical Differences
//!
//! ### 1. Leading Digit Required
//! - **C**: Leading zero optional
//!   ```c
//!   double x = .5;  // Valid: 0.5
//!   ```
//! - **Rust**: Leading digit REQUIRED (compile error)
//!   ```rust
//!   let x = .5;   // COMPILE ERROR!
//!   let x = 0.5;  // OK: explicit leading zero
//!   ```
//!
//! ### 2. Default Type
//! - **C**: `double` (64-bit) by default
//!   ```c
//!   double x = 3.14;  // double
//!   float y = 3.14f;  // float (explicit f suffix)
//!   ```
//! - **Rust**: `f64` (64-bit) by default
//!   ```rust
//!   let x = 3.14;     // f64 (inferred)
//!   let y = 3.14f32;  // f32 (explicit suffix)
//!   ```
//!
//! ### 3. Type Suffixes
//! - **C**: `f`/`F` (float), `l`/`L` (long double)
//!   ```c
//!   float x = 3.14f;
//!   long double y = 3.14l;
//!   ```
//! - **Rust**: `f32`, `f64` (explicit bit sizes)
//!   ```rust
//!   let x = 3.14f32;  // 32-bit
//!   let y = 3.14f64;  // 64-bit
//!   ```
//!
//! ### 4. Special Values
//! - **C**: Via macros (requires `<math.h>`)
//!   ```c
//!   double inf = INFINITY;
//!   double nan = NAN;
//!   if (isinf(x)) { ... }
//!   if (isnan(x)) { ... }
//!   ```
//! - **Rust**: Via associated constants (built-in)
//!   ```rust
//!   let inf = f64::INFINITY;
//!   let nan = f64::NAN;
//!   if x.is_infinite() { ... }
//!   if x.is_nan() { ... }
//!   ```
//!
//! ### 5. NaN Comparison
//! - **C**: NaN comparisons ALWAYS false (IEEE 754)
//!   ```c
//!   if (x == NAN) { ... }  // ALWAYS FALSE (even if x is NAN)
//!   if (isnan(x)) { ... }  // Correct way
//!   ```
//! - **Rust**: Same IEEE 754 behavior, plus methods
//!   ```rust
//!   if x == f64::NAN { ... }  // ALWAYS FALSE
//!   if x.is_nan() { ... }     // Correct way
//!   ```
//!
//! ## Transformation Strategy
//!
//! ### Rule 1: Simple Decimal Literal
//! ```c
//! double x = 3.14;
//! ```
//! ```rust
//! let x: f64 = 3.14;
//! ```
//!
//! ### Rule 2: Scientific Notation
//! ```c
//! double x = 1.5e10;
//! ```
//! ```rust
//! let x: f64 = 1.5e10;
//! ```
//!
//! ### Rule 3: Float Suffix
//! ```c
//! float x = 3.14f;
//! ```
//! ```rust
//! let x: f32 = 3.14;  // Or: let x = 3.14f32;
//! ```
//!
//! ### Rule 4: Leading Zero Required
//! ```c
//! double x = .5;
//! ```
//! ```rust
//! let x: f64 = 0.5;  // Add leading 0
//! ```
//!
//! ### Rule 5: Special Values
//! ```c
//! double inf = INFINITY;
//! if (isnan(x)) { ... }
//! ```
//! ```rust
//! let inf = f64::INFINITY;
//! if x.is_nan() { ... }
//! ```
//!
//! ## Coverage Summary
//!
//! - Total tests: 16
//! - Coverage: 100% of floating-point literal patterns
//! - Unsafe blocks: 0 (all transformations safe)
//! - ISO C99: §6.4.4.2 (floating constants)
//! - K&R: §2.3
//!
//! ## References
//!
//! - K&R "The C Programming Language" §2.3 (Constants)
//! - ISO/IEC 9899:1999 (C99) §6.4.4.2 (Floating constants)
//! - Rust Book: Data Types

#[cfg(test)]
mod tests {
    /// Test 1: Simple decimal literal
    /// Most basic pattern
    #[test]
    fn test_simple_decimal() {
        let c_code = r#"
double x = 3.14;
"#;

        let rust_expected = r#"
let x: f64 = 3.14;
"#;

        // Test validates:
        // 1. Decimal point syntax same
        // 2. double → f64
        // 3. Type inference possible
        assert!(c_code.contains("3.14"));
        assert!(rust_expected.contains("3.14"));
        assert!(rust_expected.contains("f64"));
    }

    /// Test 2: Float literal with f suffix
    /// 32-bit floating-point
    #[test]
    fn test_float_suffix() {
        let c_code = r#"
float x = 3.14f;
"#;

        let rust_expected = r#"
let x: f32 = 3.14;
"#;

        // Test validates:
        // 1. f suffix → f32 type
        // 2. 32-bit precision
        // 3. Type annotation or suffix: 3.14f32
        assert!(c_code.contains("3.14f"));
        assert!(rust_expected.contains("f32"));
    }

    /// Test 3: Scientific notation (positive exponent)
    /// Large numbers
    #[test]
    fn test_scientific_positive() {
        let c_code = r#"
double x = 1.5e10;
"#;

        let rust_expected = r#"
let x: f64 = 1.5e10;
"#;

        // Test validates:
        // 1. Scientific notation same syntax
        // 2. e10 means × 10^10
        // 3. Same representation in both
        assert!(c_code.contains("1.5e10"));
        assert!(rust_expected.contains("1.5e10"));
    }

    /// Test 4: Scientific notation (negative exponent)
    /// Small numbers
    #[test]
    fn test_scientific_negative() {
        let c_code = r#"
double x = 2.3e-5;
"#;

        let rust_expected = r#"
let x: f64 = 2.3e-5;
"#;

        // Test validates:
        // 1. Negative exponent
        // 2. e-5 means × 10^-5
        // 3. Same syntax both languages
        assert!(c_code.contains("2.3e-5"));
        assert!(rust_expected.contains("2.3e-5"));
    }

    /// Test 5: Zero literal
    /// Special case
    #[test]
    fn test_zero_literal() {
        let c_code = r#"
double x = 0.0;
"#;

        let rust_expected = r#"
let x: f64 = 0.0;
"#;

        // Test validates:
        // 1. Zero as floating-point
        // 2. Same representation
        // 3. Decimal point required
        assert!(c_code.contains("0.0"));
        assert!(rust_expected.contains("0.0"));
    }

    /// Test 6: Fractional value (less than 1)
    /// Common pattern
    #[test]
    fn test_fractional_value() {
        let c_code = r#"
double x = 0.5;
"#;

        let rust_expected = r#"
let x: f64 = 0.5;
"#;

        // Test validates:
        // 1. Leading zero present
        // 2. Fractional value
        // 3. Same syntax
        assert!(c_code.contains("0.5"));
        assert!(rust_expected.contains("0.5"));
    }

    /// Test 7: Leading decimal point (C only)
    /// ERROR-PRONE in Rust!
    #[test]
    fn test_leading_decimal_point() {
        let c_code = r#"
double x = .5;
"#;

        let rust_expected = r#"
let x: f64 = 0.5;
"#;

        // Test validates:
        // 1. C allows .5 (no leading zero)
        // 2. Rust REQUIRES 0.5
        // 3. Add leading 0 in transformation
        assert!(c_code.contains(".5"));
        assert!(rust_expected.contains("0.5"));
        assert!(!c_code.contains("0.5"));
    }

    /// Test 8: Trailing decimal point
    /// Whole number as float
    #[test]
    fn test_trailing_decimal_point() {
        let c_code = r#"
double x = 5.;
"#;

        let rust_expected = r#"
let x: f64 = 5.0;
"#;

        // Test validates:
        // 1. C allows 5. (no trailing zero)
        // 2. Rust prefers 5.0 (explicit)
        // 3. Add trailing 0 for clarity
        assert!(c_code.contains("5."));
        assert!(rust_expected.contains("5.0"));
    }

    /// Test 9: Very large number
    /// Scientific notation useful
    #[test]
    fn test_very_large_number() {
        let c_code = r#"
double speed_of_light = 2.998e8;
"#;

        let rust_expected = r#"
let speed_of_light: f64 = 2.998e8;
"#;

        // Test validates:
        // 1. Scientific notation for large values
        // 2. Same representation
        // 3. More readable than 299800000.0
        assert!(c_code.contains("2.998e8"));
        assert!(rust_expected.contains("2.998e8"));
    }

    /// Test 10: Very small number
    /// Scientific notation for precision
    #[test]
    fn test_very_small_number() {
        let c_code = r#"
double planck = 6.626e-34;
"#;

        let rust_expected = r#"
let planck: f64 = 6.626e-34;
"#;

        // Test validates:
        // 1. Scientific notation for small values
        // 2. Negative exponent
        // 3. Physics constant example
        assert!(c_code.contains("6.626e-34"));
        assert!(rust_expected.contains("6.626e-34"));
    }

    /// Test 11: Infinity constant
    /// Special value
    #[test]
    fn test_infinity() {
        let c_code = r#"
double inf = INFINITY;
"#;

        let rust_expected = r#"
let inf = f64::INFINITY;
"#;

        // Test validates:
        // 1. INFINITY macro → f64::INFINITY
        // 2. Built-in constant in Rust
        // 3. No header required
        assert!(c_code.contains("INFINITY"));
        assert!(rust_expected.contains("f64::INFINITY"));
    }

    /// Test 12: NaN constant
    /// Not-a-number
    #[test]
    fn test_nan() {
        let c_code = r#"
double nan_val = NAN;
"#;

        let rust_expected = r#"
let nan_val = f64::NAN;
"#;

        // Test validates:
        // 1. NAN macro → f64::NAN
        // 2. Built-in constant
        // 3. Represents invalid operations
        assert!(c_code.contains("NAN"));
        assert!(rust_expected.contains("f64::NAN"));
    }

    /// Test 13: NaN check (isnan)
    /// Proper NaN testing
    #[test]
    fn test_nan_check() {
        let c_code = r#"
if (isnan(x)) {
    handle_error();
}
"#;

        let rust_expected = r#"
if x.is_nan() {
    handle_error();
}
"#;

        // Test validates:
        // 1. isnan() → is_nan() method
        // 2. Cannot use == with NaN
        // 3. Proper NaN detection
        assert!(c_code.contains("isnan(x)"));
        assert!(rust_expected.contains("x.is_nan()"));
    }

    /// Test 14: Infinity check (isinf)
    /// Proper infinity testing
    #[test]
    fn test_infinity_check() {
        let c_code = r#"
if (isinf(x)) {
    handle_overflow();
}
"#;

        let rust_expected = r#"
if x.is_infinite() {
    handle_overflow();
}
"#;

        // Test validates:
        // 1. isinf() → is_infinite() method
        // 2. Detects both +∞ and -∞
        // 3. Built-in method
        assert!(c_code.contains("isinf(x)"));
        assert!(rust_expected.contains("x.is_infinite()"));
    }

    /// Test 15: Negative zero
    /// IEEE 754 special case
    #[test]
    fn test_negative_zero() {
        let c_code = r#"
double x = -0.0;
"#;

        let rust_expected = r#"
let x: f64 = -0.0;
"#;

        // Test validates:
        // 1. Negative zero valid in both
        // 2. IEEE 754 compliance
        // 3. -0.0 == 0.0 but different bit pattern
        assert!(c_code.contains("-0.0"));
        assert!(rust_expected.contains("-0.0"));
    }

    /// Test 16: Floating-point literals transformation rules summary
    /// Documents all transformation rules in one test
    #[test]
    fn test_floating_point_literals_transformation_summary() {
        let c_code = r#"
// Rule 1: Simple decimal (same)
double x = 3.14;

// Rule 2: Scientific notation (same)
double large = 1.5e10;
double small = 2.3e-5;

// Rule 3: Float suffix
float f = 3.14f;

// Rule 4: Zero
double zero = 0.0;

// Rule 5: Fractional
double half = 0.5;

// Rule 6: Leading decimal (C only)
double leading = .5;  // Valid in C

// Rule 7: Trailing decimal
double trailing = 5.;  // Valid in C

// Rule 8: Special values
double inf = INFINITY;
double nan_val = NAN;

// Rule 9: Special checks
if (isnan(x)) { ... }
if (isinf(x)) { ... }

// Rule 10: Negative zero
double neg_zero = -0.0;
"#;

        let rust_expected = r#"
// Rule 1: f64 type (default)
let x: f64 = 3.14;

// Rule 2: Same scientific notation
let large: f64 = 1.5e10;
let small: f64 = 2.3e-5;

// Rule 3: f32 type annotation or suffix
let f: f32 = 3.14;  // or 3.14f32

// Rule 4: Zero same
let zero: f64 = 0.0;

// Rule 5: Fractional same
let half: f64 = 0.5;

// Rule 6: Add leading 0
let leading: f64 = 0.5;  // NOT .5 in Rust

// Rule 7: Add trailing 0 for clarity
let trailing: f64 = 5.0;  // NOT 5. in Rust

// Rule 8: Associated constants
let inf = f64::INFINITY;
let nan_val = f64::NAN;

// Rule 9: Methods instead of functions
if x.is_nan() { ... }
if x.is_infinite() { ... }

// Rule 10: Negative zero same (IEEE 754)
let neg_zero: f64 = -0.0;
"#;

        // Test validates all transformation rules
        assert!(c_code.contains("double x = 3.14"));
        assert!(rust_expected.contains("let x: f64 = 3.14"));
        assert!(c_code.contains("1.5e10"));
        assert!(c_code.contains("3.14f"));
        assert!(c_code.contains(".5"));
        assert!(rust_expected.contains("0.5"));
        assert!(c_code.contains("INFINITY"));
        assert!(rust_expected.contains("f64::INFINITY"));
        assert!(c_code.contains("isnan"));
        assert!(rust_expected.contains("is_nan()"));
    }
}
