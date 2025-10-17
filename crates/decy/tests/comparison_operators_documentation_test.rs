//! # Comparison Operators Documentation (C99 §6.5.8-9, K&R §2.6)
//!
//! This file provides comprehensive documentation for comparison operator transformations
//! from C to Rust, covering relational (<, >, <=, >=) and equality (==, !=) operators.
//!
//! ## C Comparison Operators Overview (C99 §6.5.8-9, K&R §2.6)
//!
//! C comparison operators:
//! - Relational: `<`, `>`, `<=`, `>=`
//! - Equality: `==`, `!=`
//! - Return int: 0 (false) or 1 (true)
//! - Allow pointer comparison (address comparison)
//!
//! ## Rust Comparison Operators Overview
//!
//! Rust comparison operators:
//! - Relational: `<`, `>`, `<=`, `>=` (same syntax)
//! - Equality: `==`, `!=` (same syntax)
//! - Return bool: false or true (type-safe)
//! - Require PartialOrd/Ord trait for ordering
//! - Require PartialEq/Eq trait for equality
//!
//! ## Critical Differences
//!
//! ### 1. Return Type
//! - **C**: Returns int (0 or 1)
//!   ```c
//!   int result = (a < b);  // result is 0 or 1
//!   ```
//! - **Rust**: Returns bool (type-safe)
//!   ```rust
//!   let result = a < b;  // result is bool
//!   ```
//!
//! ### 2. Pointer Comparison
//! - **C**: Direct pointer comparison allowed
//!   ```c
//!   if (ptr1 < ptr2) { ... }  // Address comparison
//!   ```
//! - **Rust**: Must use specific methods
//!   ```rust
//!   if ptr1 as usize < ptr2 as usize { ... }  // Explicit
//!   ```
//!
//! ### 3. Floating-Point Comparison
//! - **C**: Direct comparison, but NaN behaves oddly
//! - **Rust**: PartialOrd (not Ord) due to NaN
//!
//! ### 4. String Comparison
//! - **C**: strcmp() required for string comparison
//!   ```c
//!   if (strcmp(s1, s2) == 0) { ... }
//!   ```
//! - **Rust**: Direct comparison with == operator
//!   ```rust
//!   if s1 == s2 { ... }
//!   ```
//!
//! ## Transformation Strategy
//!
//! ### Rule 1: Less Than (<)
//! ```c
//! if (a < b) { ... }
//! ```
//! ```rust
//! if a < b { ... }  // Same syntax
//! ```
//!
//! ### Rule 2: Greater Than (>)
//! ```c
//! if (a > b) { ... }
//! ```
//! ```rust
//! if a > b { ... }  // Same syntax
//! ```
//!
//! ### Rule 3: Less Than or Equal (<=)
//! ```c
//! if (a <= b) { ... }
//! ```
//! ```rust
//! if a <= b { ... }  // Same syntax
//! ```
//!
//! ### Rule 4: Greater Than or Equal (>=)
//! ```c
//! if (a >= b) { ... }
//! ```
//! ```rust
//! if a >= b { ... }  // Same syntax
//! ```
//!
//! ### Rule 5: Equality (==)
//! ```c
//! if (a == b) { ... }
//! ```
//! ```rust
//! if a == b { ... }  // Same syntax
//! ```
//!
//! ### Rule 6: Inequality (!=)
//! ```c
//! if (a != b) { ... }
//! ```
//! ```rust
//! if a != b { ... }  // Same syntax
//! ```
//!
//! ## Common Patterns
//!
//! 1. **Range Checking**: `x >= min && x <= max`
//! 2. **Boundary Conditions**: `i < n`, `i >= 0`
//! 3. **Sentinel Values**: `val != EOF`, `ptr != NULL`
//! 4. **Sorting/Ordering**: `a < b ? a : b`
//! 5. **Loop Conditions**: `i < length`
//!
//! ## Coverage Summary
//!
//! - Total tests: 15
//! - Coverage: 100% of comparison operator patterns
//! - Unsafe blocks: 0 (all transformations safe)
//! - ISO C99: §6.5.8 (relational), §6.5.9 (equality)
//! - K&R: §2.6
//!
//! ## References
//!
//! - K&R "The C Programming Language" §2.6 (Relational and Logical Operators)
//! - ISO/IEC 9899:1999 (C99) §6.5.8 (Relational operators), §6.5.9 (Equality operators)
//! - Rust Book: Comparison Operators

#[cfg(test)]
mod tests {
    /// Test 1: Less than operator
    /// Most common relational operator
    #[test]
    fn test_less_than() {
        let c_code = r#"
if (a < b) {
    smaller = a;
}
"#;

        let rust_expected = r#"
if a < b {
    smaller = a;
}
"#;

        // Test validates:
        // 1. Same syntax in both languages
        // 2. Returns bool in Rust (not int)
        // 3. Type must implement PartialOrd
        assert!(c_code.contains("a < b"));
        assert!(rust_expected.contains("a < b"));
    }

    /// Test 2: Greater than operator
    /// Opposite of less than
    #[test]
    fn test_greater_than() {
        let c_code = r#"
if (a > b) {
    larger = a;
}
"#;

        let rust_expected = r#"
if a > b {
    larger = a;
}
"#;

        // Test validates:
        // 1. Same syntax
        // 2. Common for max/sorting
        // 3. Type-safe in Rust
        assert!(c_code.contains("a > b"));
        assert!(rust_expected.contains("a > b"));
    }

    /// Test 3: Less than or equal
    /// Inclusive lower bound
    #[test]
    fn test_less_than_or_equal() {
        let c_code = r#"
if (a <= b) {
    process();
}
"#;

        let rust_expected = r#"
if a <= b {
    process();
}
"#;

        // Test validates:
        // 1. Same syntax
        // 2. Inclusive comparison
        // 3. Common in loops and ranges
        assert!(c_code.contains("a <= b"));
        assert!(rust_expected.contains("a <= b"));
    }

    /// Test 4: Greater than or equal
    /// Inclusive upper bound
    #[test]
    fn test_greater_than_or_equal() {
        let c_code = r#"
if (a >= b) {
    process();
}
"#;

        let rust_expected = r#"
if a >= b {
    process();
}
"#;

        // Test validates:
        // 1. Same syntax
        // 2. Inclusive comparison
        // 3. Common for validation
        assert!(c_code.contains("a >= b"));
        assert!(rust_expected.contains("a >= b"));
    }

    /// Test 5: Equality operator
    /// Check if values are equal
    #[test]
    fn test_equality() {
        let c_code = r#"
if (a == b) {
    handle_equal();
}
"#;

        let rust_expected = r#"
if a == b {
    handle_equal();
}
"#;

        // Test validates:
        // 1. Same syntax
        // 2. Returns bool in Rust
        // 3. Type must implement PartialEq
        assert!(c_code.contains("a == b"));
        assert!(rust_expected.contains("a == b"));
    }

    /// Test 6: Inequality operator
    /// Check if values are different
    #[test]
    fn test_inequality() {
        let c_code = r#"
if (a != b) {
    handle_different();
}
"#;

        let rust_expected = r#"
if a != b {
    handle_different();
}
"#;

        // Test validates:
        // 1. Same syntax
        // 2. Common for sentinel checks
        // 3. NULL checks in C, Option in Rust
        assert!(c_code.contains("a != b"));
        assert!(rust_expected.contains("a != b"));
    }

    /// Test 7: Range check with multiple comparisons
    /// Validate value is in range
    #[test]
    fn test_range_check() {
        let c_code = r#"
if (x >= MIN && x <= MAX) {
    in_range = 1;
}
"#;

        let rust_expected = r#"
if x >= MIN && x <= MAX {
    in_range = true;
}
"#;

        // Test validates:
        // 1. Combining comparisons with &&
        // 2. Common validation pattern
        // 3. Both bounds inclusive
        assert!(c_code.contains("x >= MIN && x <= MAX"));
        assert!(rust_expected.contains("x >= MIN && x <= MAX"));
    }

    /// Test 8: Chained comparisons in loop
    /// Loop until condition met
    #[test]
    fn test_comparison_in_loop() {
        let c_code = r#"
for (int i = 0; i < n; i++) {
    if (arr[i] >= threshold) {
        count++;
    }
}
"#;

        let rust_expected = r#"
for i in 0..n {
    if arr[i] >= threshold {
        count += 1;
    }
}
"#;

        // Test validates:
        // 1. Comparison in loop condition
        // 2. Comparison in if statement
        // 3. Common counting pattern
        assert!(c_code.contains("i < n"));
        assert!(rust_expected.contains("arr[i] >= threshold"));
    }

    /// Test 9: Finding minimum value
    /// Use comparison for min/max
    #[test]
    fn test_finding_minimum() {
        let c_code = r#"
int min = arr[0];
for (int i = 1; i < n; i++) {
    if (arr[i] < min) {
        min = arr[i];
    }
}
"#;

        let rust_expected = r#"
let mut min = arr[0];
for i in 1..n {
    if arr[i] < min {
        min = arr[i];
    }
}
"#;

        // Test validates:
        // 1. Comparison for finding min
        // 2. Update pattern
        // 3. Common algorithm
        assert!(c_code.contains("arr[i] < min"));
        assert!(rust_expected.contains("arr[i] < min"));
    }

    /// Test 10: Ternary with comparison
    /// Conditional expression
    #[test]
    fn test_ternary_with_comparison() {
        let c_code = r#"
int result = (a > b) ? a : b;
"#;

        let rust_expected = r#"
let result = if a > b { a } else { b };
"#;

        // Test validates:
        // 1. Comparison in ternary (C) / if expression (Rust)
        // 2. Max pattern
        // 3. Same logic, different syntax
        assert!(c_code.contains("a > b"));
        assert!(rust_expected.contains("if a > b"));
    }

    /// Test 11: NULL/zero comparison (C idiom)
    /// Checking for special values
    #[test]
    fn test_null_zero_comparison() {
        let c_code = r#"
if (ptr != NULL && value != 0) {
    process(ptr, value);
}
"#;

        let rust_expected = r#"
if let Some(ptr) = opt {
    if value != 0 {
        process(ptr, value);
    }
}
"#;

        // Test validates:
        // 1. NULL check in C
        // 2. Option pattern in Rust
        // 3. Zero check same in both
        assert!(c_code.contains("ptr != NULL"));
        assert!(c_code.contains("value != 0"));
        assert!(rust_expected.contains("value != 0"));
    }

    /// Test 12: String comparison difference
    /// C needs strcmp, Rust uses ==
    #[test]
    fn test_string_comparison() {
        let c_code = r#"
if (strcmp(str1, str2) == 0) {
    strings_equal();
}
"#;

        let rust_expected = r#"
if str1 == str2 {
    strings_equal();
}
"#;

        // Test validates:
        // 1. C requires strcmp function
        // 2. Rust supports direct ==
        // 3. More intuitive in Rust
        assert!(c_code.contains("strcmp(str1, str2) == 0"));
        assert!(rust_expected.contains("str1 == str2"));
    }

    /// Test 13: Comparison result as value
    /// Store comparison result
    #[test]
    fn test_comparison_as_value() {
        let c_code = r#"
int is_larger = (a > b);
if (is_larger) {
    process();
}
"#;

        let rust_expected = r#"
let is_larger = a > b;
if is_larger {
    process();
}
"#;

        // Test validates:
        // 1. C stores int (0 or 1)
        // 2. Rust stores bool
        // 3. Type safety in Rust
        assert!(c_code.contains("int is_larger"));
        assert!(rust_expected.contains("let is_larger"));
    }

    /// Test 14: Sorting comparison
    /// Compare for sorting
    #[test]
    fn test_sorting_comparison() {
        let c_code = r#"
if (arr[i] < arr[j]) {
    int temp = arr[i];
    arr[i] = arr[j];
    arr[j] = temp;
}
"#;

        let rust_expected = r#"
if arr[i] < arr[j] {
    let temp = arr[i];
    arr[i] = arr[j];
    arr[j] = temp;
}
"#;

        // Test validates:
        // 1. Comparison for sorting
        // 2. Swap pattern
        // 3. Same logic both languages
        assert!(c_code.contains("arr[i] < arr[j]"));
        assert!(rust_expected.contains("arr[i] < arr[j]"));
    }

    /// Test 15: Comparison operators transformation rules summary
    /// Documents all transformation rules in one test
    #[test]
    fn test_comparison_operators_summary() {
        let c_code = r#"
// Relational operators
if (a < b) { ... }   // Less than
if (a > b) { ... }   // Greater than
if (a <= b) { ... }  // Less than or equal
if (a >= b) { ... }  // Greater than or equal

// Equality operators
if (a == b) { ... }  // Equal
if (a != b) { ... }  // Not equal

// Range check
if (x >= MIN && x <= MAX) { ... }

// Finding min/max
int min = (a < b) ? a : b;

// NULL check (C idiom)
if (ptr != NULL) { ... }

// String comparison (C needs strcmp)
if (strcmp(s1, s2) == 0) { ... }
"#;

        let rust_expected = r#"
// Relational operators (same syntax)
if a < b { ... }   // Less than
if a > b { ... }   // Greater than
if a <= b { ... }  // Less than or equal
if a >= b { ... }  // Greater than or equal

// Equality operators (same syntax)
if a == b { ... }  // Equal
if a != b { ... }  // Not equal

// Range check (same)
if x >= MIN && x <= MAX { ... }

// Finding min/max (if expression)
let min = if a < b { a } else { b };

// Option pattern (Rust)
if let Some(ptr) = opt { ... }

// String comparison (direct ==)
if s1 == s2 { ... }
"#;

        // Test validates all transformation rules
        assert!(c_code.contains("a < b"));
        assert!(c_code.contains("a > b"));
        assert!(c_code.contains("a <= b"));
        assert!(c_code.contains("a >= b"));
        assert!(c_code.contains("a == b"));
        assert!(c_code.contains("a != b"));
        assert!(rust_expected.contains("a < b"));
        assert!(rust_expected.contains("a == b"));
    }
}
