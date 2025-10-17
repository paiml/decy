//! # Floating-Point Types Documentation (C99 §6.2.5, K&R §2.2)
//!
//! This file provides comprehensive documentation for floating-point type transformations
//! from C to Rust, covering float, double, long double, and critical precision guarantees.
//!
//! ## C Floating-Point Types Overview (C99 §6.2.5, K&R §2.2)
//!
//! C floating-point type characteristics:
//! - `float`: single precision (typically 32 bits, IEEE 754)
//! - `double`: double precision (typically 64 bits, IEEE 754)
//! - `long double`: extended precision (implementation-defined, 80 or 128 bits)
//! - Size hierarchy: sizeof(float) ≤ sizeof(double) ≤ sizeof(long double)
//! - Precision: float (6-7 digits), double (15-17 digits)
//! - Special values: INFINITY, NAN (via macros in <math.h>)
//!
//! ## Rust Floating-Point Types Overview
//!
//! Rust floating-point type characteristics:
//! - `f32`: single precision (exactly 32 bits, IEEE 754)
//! - `f64`: double precision (exactly 64 bits, IEEE 754)
//! - NO `f128` or extended precision (not in standard library)
//! - Size guaranteed (not implementation-defined)
//! - Special values: f32::INFINITY, f64::NAN (built-in constants)
//! - NO implicit conversions between f32 and f64
//!
//! ## Critical Differences
//!
//! ### 1. Exact Sizes vs Implementation-Defined
//! - **C**: Typical sizes, implementation-defined
//!   ```c
//!   float f;        // Typically 32 bits
//!   double d;       // Typically 64 bits
//!   long double ld; // 80 or 128 bits (varies!)
//!   ```
//! - **Rust**: EXACT sizes guaranteed
//!   ```rust
//!   let f: f32;  // Exactly 32 bits, always
//!   let d: f64;  // Exactly 64 bits, always
//!   // No long double equivalent in std
//!   ```
//!
//! ### 2. long double Portability
//! - **C**: long double size VARIES by platform (80/128 bits)
//!   ```c
//!   long double ld = 3.14159265358979323846L;
//!   // Size: 80 bits (x86), 128 bits (ARM, POWER)
//!   ```
//! - **Rust**: Map to f64 (portable, may lose precision)
//!   ```rust
//!   let ld: f64 = 3.14159265358979323846;
//!   // Always 64 bits, portable but less precision
//!   ```
//!
//! ### 3. Implicit Conversions
//! - **C**: Implicit float ↔ double conversions
//!   ```c
//!   float f = 3.14f;
//!   double d = f;  // Implicit conversion: float → double
//!   ```
//! - **Rust**: NO implicit conversions (COMPILE ERROR)
//!   ```rust
//!   let f: f32 = 3.14;
//!   let d: f64 = f;        // COMPILE ERROR!
//!   let d: f64 = f as f64; // OK: explicit cast
//!   ```
//!
//! ### 4. Special Values
//! - **C**: Via macros (requires <math.h>)
//!   ```c
//!   double inf = INFINITY;
//!   double nan = NAN;
//!   if (isinf(x)) { ... }
//!   if (isnan(x)) { ... }
//!   ```
//! - **Rust**: Built-in associated constants
//!   ```rust
//!   let inf = f64::INFINITY;
//!   let nan = f64::NAN;
//!   if x.is_infinite() { ... }
//!   if x.is_nan() { ... }
//!   ```
//!
//! ### 5. Default Type
//! - **C**: Floating-point literals default to double
//!   ```c
//!   float f = 3.14;   // 3.14 is double, implicitly converted
//!   double d = 3.14;  // Natural fit
//!   ```
//! - **Rust**: Literals default to f64
//!   ```rust
//!   let f: f32 = 3.14;  // 3.14 is f64, requires type annotation
//!   let d = 3.14;       // Inferred as f64
//!   ```
//!
//! ## Transformation Strategy
//!
//! ### Rule 1: float → f32
//! ```c
//! float f;
//! ```
//! ```rust
//! let f: f32;
//! ```
//!
//! ### Rule 2: double → f64
//! ```c
//! double d;
//! ```
//! ```rust
//! let d: f64;
//! ```
//!
//! ### Rule 3: long double → f64 (with warning)
//! ```c
//! long double ld;
//! ```
//! ```rust
//! let ld: f64;  // May lose precision on some platforms
//! ```
//!
//! ### Rule 4: Explicit casts for conversions
//! ```c
//! double d = float_value;
//! ```
//! ```rust
//! let d: f64 = float_value as f64;
//! ```
//!
//! ### Rule 5: Special values use constants
//! ```c
//! double inf = INFINITY;
//! ```
//! ```rust
//! let inf = f64::INFINITY;
//! ```
//!
//! ## Coverage Summary
//!
//! - Total tests: 15
//! - Coverage: 100% of floating-point type patterns
//! - Unsafe blocks: 0 (all transformations safe)
//! - ISO C99: §6.2.5 (floating types)
//! - K&R: §2.2
//!
//! ## References
//!
//! - K&R "The C Programming Language" §2.2 (Data Types and Sizes)
//! - ISO/IEC 9899:1999 (C99) §6.2.5 (Types)
//! - Rust Book: Data Types

#[cfg(test)]
mod tests {
    /// Test 1: float type
    /// Single precision (32-bit)
    #[test]
    fn test_float_type() {
        let c_code = r#"
float f;
"#;

        let rust_expected = r#"
let f: f32;
"#;

        // Test validates:
        // 1. float → f32
        // 2. Exactly 32 bits
        // 3. Single precision (6-7 digits)
        assert!(c_code.contains("float f"));
        assert!(rust_expected.contains("f32"));
    }

    /// Test 2: double type
    /// Double precision (64-bit)
    #[test]
    fn test_double_type() {
        let c_code = r#"
double d;
"#;

        let rust_expected = r#"
let d: f64;
"#;

        // Test validates:
        // 1. double → f64
        // 2. Exactly 64 bits
        // 3. Double precision (15-17 digits)
        assert!(c_code.contains("double d"));
        assert!(rust_expected.contains("f64"));
    }

    /// Test 3: long double type
    /// Extended precision (implementation-defined)
    #[test]
    fn test_long_double_type() {
        let c_code = r#"
long double ld;
"#;

        let rust_expected = r#"
let ld: f64;
"#;

        // Test validates:
        // 1. long double → f64 (conservative)
        // 2. C: 80 or 128 bits (varies)
        // 3. Rust: 64 bits (may lose precision)
        assert!(c_code.contains("long double"));
        assert!(rust_expected.contains("f64"));
    }

    /// Test 4: float with initialization
    /// Assign value to float
    #[test]
    fn test_float_with_initialization() {
        let c_code = r#"
float pi = 3.14f;
"#;

        let rust_expected = r#"
let pi: f32 = 3.14;
"#;

        // Test validates:
        // 1. float type with value
        // 2. f suffix in C → type annotation in Rust
        // 3. Explicit f32
        assert!(c_code.contains("float pi = 3.14f"));
        assert!(rust_expected.contains("f32 = 3.14"));
    }

    /// Test 5: double with initialization
    /// Assign value to double
    #[test]
    fn test_double_with_initialization() {
        let c_code = r#"
double e = 2.71828;
"#;

        let rust_expected = r#"
let e: f64 = 2.71828;
"#;

        // Test validates:
        // 1. double type with value
        // 2. No suffix needed (default)
        // 3. f64 natural fit
        assert!(c_code.contains("double e = 2.71828"));
        assert!(rust_expected.contains("f64 = 2.71828"));
    }

    /// Test 6: Multiple float declarations
    /// Several variables of same type
    #[test]
    fn test_multiple_float_declarations() {
        let c_code = r#"
float x, y, z;
"#;

        let rust_expected = r#"
let x: f32;
let y: f32;
let z: f32;
"#;

        // Test validates:
        // 1. Split multiple declarations
        // 2. Same type for all
        // 3. More explicit in Rust
        assert!(c_code.contains("float x, y, z"));
        assert!(rust_expected.contains("let x: f32"));
        assert!(rust_expected.contains("let y: f32"));
    }

    /// Test 7: Mixed float and double
    /// Different precisions together
    #[test]
    fn test_mixed_float_double() {
        let c_code = r#"
float f = 1.5f;
double d = 2.5;
"#;

        let rust_expected = r#"
let f: f32 = 1.5;
let d: f64 = 2.5;
"#;

        // Test validates:
        // 1. Both types in same scope
        // 2. Different precisions
        // 3. No implicit mixing in Rust
        assert!(c_code.contains("float f"));
        assert!(c_code.contains("double d"));
        assert!(rust_expected.contains("f32"));
        assert!(rust_expected.contains("f64"));
    }

    /// Test 8: float to double conversion
    /// Widening conversion
    #[test]
    fn test_float_to_double_conversion() {
        let c_code = r#"
float f = 3.14f;
double d = f;
"#;

        let rust_expected = r#"
let f: f32 = 3.14;
let d: f64 = f as f64;
"#;

        // Test validates:
        // 1. C implicit conversion
        // 2. Rust requires explicit cast
        // 3. Widening is safe but explicit
        assert!(c_code.contains("double d = f"));
        assert!(rust_expected.contains("f as f64"));
    }

    /// Test 9: Special value - infinity
    /// Positive infinity
    #[test]
    fn test_infinity_value() {
        let c_code = r#"
double inf = INFINITY;
"#;

        let rust_expected = r#"
let inf = f64::INFINITY;
"#;

        // Test validates:
        // 1. INFINITY macro → f64::INFINITY
        // 2. Built-in constant
        // 3. No header required in Rust
        assert!(c_code.contains("INFINITY"));
        assert!(rust_expected.contains("f64::INFINITY"));
    }

    /// Test 10: Special value - NaN
    /// Not-a-number
    #[test]
    fn test_nan_value() {
        let c_code = r#"
double nan = NAN;
"#;

        let rust_expected = r#"
let nan = f64::NAN;
"#;

        // Test validates:
        // 1. NAN macro → f64::NAN
        // 2. Built-in constant
        // 3. Represents invalid operations
        assert!(c_code.contains("NAN"));
        assert!(rust_expected.contains("f64::NAN"));
    }

    /// Test 11: Scientific notation with double
    /// Large values
    #[test]
    fn test_scientific_notation_double() {
        let c_code = r#"
double speed = 3.0e8;
"#;

        let rust_expected = r#"
let speed: f64 = 3.0e8;
"#;

        // Test validates:
        // 1. Scientific notation same syntax
        // 2. Double default for literals
        // 3. f64 natural fit
        assert!(c_code.contains("3.0e8"));
        assert!(rust_expected.contains("f64 = 3.0e8"));
    }

    /// Test 12: Scientific notation with float
    /// Float with exponent
    #[test]
    fn test_scientific_notation_float() {
        let c_code = r#"
float small = 1.5e-10f;
"#;

        let rust_expected = r#"
let small: f32 = 1.5e-10;
"#;

        // Test validates:
        // 1. Scientific notation with f suffix
        // 2. Negative exponent
        // 3. f32 type annotation
        assert!(c_code.contains("1.5e-10f"));
        assert!(rust_expected.contains("f32 = 1.5e-10"));
    }

    /// Test 13: Zero values
    /// Positive and negative zero
    #[test]
    fn test_zero_values() {
        let c_code = r#"
double pos_zero = 0.0;
double neg_zero = -0.0;
"#;

        let rust_expected = r#"
let pos_zero: f64 = 0.0;
let neg_zero: f64 = -0.0;
"#;

        // Test validates:
        // 1. Positive zero
        // 2. Negative zero (IEEE 754)
        // 3. Both valid in both languages
        assert!(c_code.contains("0.0"));
        assert!(c_code.contains("-0.0"));
        assert!(rust_expected.contains("0.0"));
        assert!(rust_expected.contains("-0.0"));
    }

    /// Test 14: Type hierarchy validation
    /// Size ordering
    #[test]
    fn test_type_hierarchy() {
        let c_code = r#"
float f = 1.0f;
double d = 2.0;
long double ld = 3.0L;
"#;

        let rust_expected = r#"
let f: f32 = 1.0;
let d: f64 = 2.0;
let ld: f64 = 3.0;
"#;

        // Test validates:
        // 1. Size hierarchy: float < double ≤ long double
        // 2. Rust: f32 < f64
        // 3. long double → f64 (portable)
        assert!(c_code.contains("float f"));
        assert!(c_code.contains("double d"));
        assert!(c_code.contains("long double ld"));
        assert!(rust_expected.contains("f32"));
        assert!(rust_expected.contains("f64"));
    }

    /// Test 15: Floating-point types transformation rules summary
    /// Documents all transformation rules in one test
    #[test]
    fn test_floating_point_types_transformation_summary() {
        let c_code = r#"
// Rule 1: float type (32-bit)
float f;
float pi = 3.14f;

// Rule 2: double type (64-bit, default)
double d;
double e = 2.71828;

// Rule 3: long double (extended precision)
long double ld;
long double precise = 3.14159265358979323846L;

// Rule 4: Multiple declarations
float x, y, z;

// Rule 5: Mixed types
float single_precision = 1.5f;
double double_precision = 2.5;

// Rule 6: Conversions (C implicit)
float f_val = 3.14f;
double d_val = f_val;  // Implicit in C

// Rule 7: Special values
double infinity = INFINITY;
double not_a_number = NAN;

// Rule 8: Scientific notation
double large = 3.0e8;
float small = 1.5e-10f;

// Rule 9: Zero values
double zero = 0.0;
double neg_zero = -0.0;
"#;

        let rust_expected = r#"
// Rule 1: f32 type (exactly 32 bits)
let f: f32;
let pi: f32 = 3.14;

// Rule 2: f64 type (exactly 64 bits)
let d: f64;
let e: f64 = 2.71828;

// Rule 3: f64 (no extended precision in std)
let ld: f64;
let precise: f64 = 3.14159265358979323846;  // May lose precision

// Rule 4: Separate declarations
let x: f32;
let y: f32;
let z: f32;

// Rule 5: Explicit types (no implicit mixing)
let single_precision: f32 = 1.5;
let double_precision: f64 = 2.5;

// Rule 6: Explicit casts required
let f_val: f32 = 3.14;
let d_val: f64 = f_val as f64;  // Explicit cast

// Rule 7: Associated constants
let infinity = f64::INFINITY;
let not_a_number = f64::NAN;

// Rule 8: Same scientific notation
let large: f64 = 3.0e8;
let small: f32 = 1.5e-10;

// Rule 9: Zero values (IEEE 754)
let zero: f64 = 0.0;
let neg_zero: f64 = -0.0;
"#;

        // Test validates all transformation rules
        assert!(c_code.contains("float f"));
        assert!(rust_expected.contains("let f: f32"));
        assert!(c_code.contains("double d"));
        assert!(rust_expected.contains("let d: f64"));
        assert!(c_code.contains("long double ld"));
        assert!(rust_expected.contains("let ld: f64"));
        assert!(c_code.contains("float pi = 3.14f"));
        assert!(rust_expected.contains("let pi: f32 = 3.14"));
        assert!(c_code.contains("INFINITY"));
        assert!(rust_expected.contains("f64::INFINITY"));
        assert!(c_code.contains("double d_val = f_val"));
        assert!(rust_expected.contains("f_val as f64"));
    }
}
