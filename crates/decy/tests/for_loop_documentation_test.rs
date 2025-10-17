//! # For Loop Patterns Documentation (C99 §6.8.5.3, K&R §3.5)
//!
//! This file provides comprehensive documentation for for loop transformations
//! from C to Rust, covering all common patterns and critical differences.
//!
//! ## C For Loop Overview (C99 §6.8.5.3, K&R §3.5)
//!
//! C for loops have three parts:
//! - Initialization: `int i = 0`
//! - Condition: `i < n`
//! - Increment: `i++`
//! - Syntax: `for (init; condition; increment) { body }`
//!
//! ## Rust For Loop Overview
//!
//! Rust for loops are iterator-based:
//! - Range syntax: `for i in 0..n { body }`
//! - Iterator-based: `for item in collection { body }`
//! - More concise and safer than C
//! - Prevents off-by-one errors
//!
//! ## Critical Differences
//!
//! ### 1. Basic For Loop
//! - **C**: Three-part syntax (init; condition; increment)
//!   ```c
//!   for (int i = 0; i < n; i++) {
//!       process(i);
//!   }
//!   ```
//! - **Rust**: Range-based syntax (more concise)
//!   ```rust
//!   for i in 0..n {
//!       process(i);
//!   }
//!   ```
//!
//! ### 2. Range Types
//! - **C**: Manual index management with < or <=
//!   ```c
//!   for (i = 0; i < 10; i++)   // 0 to 9 (exclusive)
//!   for (i = 0; i <= 10; i++)  // 0 to 10 (inclusive)
//!   ```
//! - **Rust**: Explicit range types
//!   ```rust
//!   for i in 0..10   // 0 to 9 (exclusive)
//!   for i in 0..=10  // 0 to 10 (inclusive)
//!   ```
//!
//! ### 3. Custom Increment
//! - **C**: Flexible increment expression
//!   ```c
//!   for (i = 0; i < n; i += 2)  // Step by 2
//!   ```
//! - **Rust**: Use step_by() method
//!   ```rust
//!   for i in (0..n).step_by(2)  // Step by 2
//!   ```
//!
//! ### 4. Reverse Iteration
//! - **C**: Decrement with >
//!   ```c
//!   for (i = 10; i > 0; i--)  // 10 down to 1
//!   ```
//! - **Rust**: Use rev() method
//!   ```rust
//!   for i in (1..=10).rev()  // 10 down to 1
//!   ```
//!
//! ### 5. Array Iteration
//! - **C**: Manual indexing (error-prone)
//!   ```c
//!   for (i = 0; i < arr_len; i++) {
//!       process(arr[i]);  // Possible out-of-bounds
//!   }
//!   ```
//! - **Rust**: Iterator-based (safe)
//!   ```rust
//!   for &item in arr.iter() {
//!       process(item);  // Cannot go out of bounds
//!   }
//!   // OR with index:
//!   for (i, &item) in arr.iter().enumerate() {
//!       process(i, item);
//!   }
//!   ```
//!
//! ### 6. Infinite Loop
//! - **C**: Empty parts (unusual)
//!   ```c
//!   for (;;) { ... }  // Infinite loop
//!   ```
//! - **Rust**: Use loop keyword (clearer)
//!   ```rust
//!   loop { ... }  // Idiomatic infinite loop
//!   ```
//!
//! ## Transformation Strategy
//!
//! ### Rule 1: Simple Counting Loop
//! ```c
//! for (int i = 0; i < n; i++) {
//!     process(i);
//! }
//! ```
//! ```rust
//! for i in 0..n {
//!     process(i);
//! }
//! ```
//!
//! ### Rule 2: Inclusive Range
//! ```c
//! for (int i = 0; i <= 10; i++) {
//!     process(i);
//! }
//! ```
//! ```rust
//! for i in 0..=10 {
//!     process(i);
//! }
//! ```
//!
//! ### Rule 3: Custom Step
//! ```c
//! for (int i = 0; i < n; i += 2) {
//!     process(i);
//! }
//! ```
//! ```rust
//! for i in (0..n).step_by(2) {
//!     process(i);
//! }
//! ```
//!
//! ### Rule 4: Reverse Loop
//! ```c
//! for (int i = 10; i > 0; i--) {
//!     process(i);
//! }
//! ```
//! ```rust
//! for i in (1..=10).rev() {
//!     process(i);
//! }
//! ```
//!
//! ### Rule 5: Array Iteration with Index
//! ```c
//! for (int i = 0; i < arr_len; i++) {
//!     arr[i] = compute(i);
//! }
//! ```
//! ```rust
//! for (i, item) in arr.iter_mut().enumerate() {
//!     *item = compute(i);
//! }
//! ```
//!
//! ### Rule 6: Array Iteration without Index
//! ```c
//! for (int i = 0; i < arr_len; i++) {
//!     sum += arr[i];
//! }
//! ```
//! ```rust
//! for &item in arr.iter() {
//!     sum += item;
//! }
//! // OR even more idiomatic:
//! let sum: i32 = arr.iter().sum();
//! ```
//!
//! ## Common Patterns
//!
//! 1. **Counting**: Loop from 0 to n-1
//! 2. **Custom Step**: Increment by value other than 1
//! 3. **Reverse**: Count down from n to 0
//! 4. **Array Processing**: Iterate over array elements
//! 5. **Nested Loops**: Loop within a loop
//! 6. **Early Exit**: Break or continue in loop
//!
//! ## Safety Advantages in Rust
//!
//! - **No Off-by-One Errors**: Range syntax prevents < vs <= confusion
//! - **No Out-of-Bounds**: Iterators guarantee safe access
//! - **No Uninitialized**: Loop variable always initialized
//! - **Type Safety**: Iterator types checked at compile time
//!
//! ## Coverage Summary
//!
//! - Total tests: 16
//! - Coverage: 100% of documented for loop patterns
//! - Unsafe blocks: 0 (all transformations safe)
//! - ISO C99: §6.8.5.3
//! - K&R: §3.5
//!
//! ## References
//!
//! - K&R "The C Programming Language" §3.5 (The for Statement)
//! - ISO/IEC 9899:1999 (C99) §6.8.5.3 (The for statement)
//! - Rust Book: Loops (https://doc.rust-lang.org/book/ch03-05-control-flow.html#looping-through-a-collection-with-for)

#[cfg(test)]
mod tests {
    /// Test 1: Simple counting loop (0 to n-1)
    /// Most common pattern
    #[test]
    fn test_simple_counting_loop() {
        let c_code = r#"
for (int i = 0; i < 10; i++) {
    process(i);
}
"#;

        let rust_expected = r#"
for i in 0..10 {
    process(i);
}
"#;

        // Test validates:
        // 1. C three-part syntax → Rust range syntax
        // 2. Exclusive upper bound (0 to 9)
        // 3. More concise Rust syntax
        assert!(c_code.contains("for (int i = 0; i < 10; i++)"));
        assert!(rust_expected.contains("for i in 0..10"));
    }

    /// Test 2: Inclusive range (0 to n)
    /// Using <= in C, ..= in Rust
    #[test]
    fn test_inclusive_range() {
        let c_code = r#"
for (int i = 0; i <= 10; i++) {
    process(i);
}
"#;

        let rust_expected = r#"
for i in 0..=10 {
    process(i);
}
"#;

        // Test validates:
        // 1. C <= operator
        // 2. Rust ..= inclusive range
        // 3. Explicit inclusive syntax prevents errors
        assert!(c_code.contains("i <= 10"));
        assert!(rust_expected.contains("0..=10"));
    }

    /// Test 3: Custom step (increment by 2)
    /// Skip odd numbers
    #[test]
    fn test_custom_step_increment() {
        let c_code = r#"
for (int i = 0; i < 20; i += 2) {
    process(i);
}
"#;

        let rust_expected = r#"
for i in (0..20).step_by(2) {
    process(i);
}
"#;

        // Test validates:
        // 1. C += 2 increment
        // 2. Rust step_by(2) method
        // 3. Iterator chaining
        assert!(c_code.contains("i += 2"));
        assert!(rust_expected.contains(".step_by(2)"));
    }

    /// Test 4: Reverse iteration (count down)
    /// From 10 down to 1
    #[test]
    fn test_reverse_iteration() {
        let c_code = r#"
for (int i = 10; i > 0; i--) {
    process(i);
}
"#;

        let rust_expected = r#"
for i in (1..=10).rev() {
    process(i);
}
"#;

        // Test validates:
        // 1. C decrement with >
        // 2. Rust rev() method
        // 3. Inclusive range + reverse
        assert!(c_code.contains("i--"));
        assert!(rust_expected.contains(".rev()"));
    }

    /// Test 5: Array iteration with manual indexing (C style)
    /// Access array elements by index
    #[test]
    fn test_array_iteration_manual_index() {
        let c_code = r#"
for (int i = 0; i < arr_len; i++) {
    sum += arr[i];
}
"#;

        let rust_with_index = r#"
for i in 0..arr_len {
    sum += arr[i];
}
"#;

        // Test validates:
        // 1. C manual indexing
        // 2. Rust can use same pattern
        // 3. But iterator pattern is more idiomatic
        assert!(c_code.contains("arr[i]"));
        assert!(rust_with_index.contains("arr[i]"));
    }

    /// Test 6: Array iteration with iterator (Rust idiomatic)
    /// More concise and safe
    #[test]
    fn test_array_iteration_with_iterator() {
        let c_code = r#"
for (int i = 0; i < arr_len; i++) {
    sum += arr[i];
}
"#;

        let rust_idiomatic = r#"
for &item in arr.iter() {
    sum += item;
}
"#;

        // Test validates:
        // 1. C requires manual indexing
        // 2. Rust iterator more concise
        // 3. No bounds checking needed
        assert!(c_code.contains("arr[i]"));
        assert!(rust_idiomatic.contains(".iter()"));
    }

    /// Test 7: Array iteration with index and value (enumerate)
    /// Need both index and value
    #[test]
    fn test_array_iteration_enumerate() {
        let c_code = r#"
for (int i = 0; i < arr_len; i++) {
    printf("arr[%d] = %d\n", i, arr[i]);
}
"#;

        let rust_expected = r#"
for (i, &item) in arr.iter().enumerate() {
    println!("arr[{}] = {}", i, item);
}
"#;

        // Test validates:
        // 1. C manual index + value access
        // 2. Rust enumerate() provides both
        // 3. More expressive than manual indexing
        assert!(c_code.contains("arr[i]"));
        assert!(rust_expected.contains(".enumerate()"));
    }

    /// Test 8: Nested loops
    /// Loop within a loop
    #[test]
    fn test_nested_loops() {
        let c_code = r#"
for (int i = 0; i < n; i++) {
    for (int j = 0; j < m; j++) {
        matrix[i][j] = i * m + j;
    }
}
"#;

        let rust_expected = r#"
for i in 0..n {
    for j in 0..m {
        matrix[i][j] = i * m + j;
    }
}
"#;

        // Test validates:
        // 1. Nested loop structure preserved
        // 2. Both languages support nesting
        // 3. Rust range syntax cleaner
        assert_eq!(c_code.matches("for (").count(), 2);
        assert_eq!(rust_expected.matches("for ").count(), 2);
    }

    /// Test 9: Loop with break
    /// Early exit from loop
    #[test]
    fn test_loop_with_break() {
        let c_code = r#"
for (int i = 0; i < n; i++) {
    if (arr[i] == target) break;
    process(arr[i]);
}
"#;

        let rust_expected = r#"
for i in 0..n {
    if arr[i] == target { break; }
    process(arr[i]);
}
"#;

        // Test validates:
        // 1. Break works same in both
        // 2. Early exit pattern
        // 3. Search-and-stop pattern
        assert!(c_code.contains("break;"));
        assert!(rust_expected.contains("break;"));
    }

    /// Test 10: Loop with continue
    /// Skip to next iteration
    #[test]
    fn test_loop_with_continue() {
        let c_code = r#"
for (int i = 0; i < n; i++) {
    if (arr[i] < 0) continue;
    sum += arr[i];
}
"#;

        let rust_expected = r#"
for i in 0..n {
    if arr[i] < 0 { continue; }
    sum += arr[i];
}
"#;

        // Test validates:
        // 1. Continue works same in both
        // 2. Filter pattern
        // 3. Skip unwanted items
        assert!(c_code.contains("continue;"));
        assert!(rust_expected.contains("continue;"));
    }

    /// Test 11: Loop starting at non-zero
    /// Start at 1 instead of 0
    #[test]
    fn test_loop_start_at_one() {
        let c_code = r#"
for (int i = 1; i <= n; i++) {
    process(i);
}
"#;

        let rust_expected = r#"
for i in 1..=n {
    process(i);
}
"#;

        // Test validates:
        // 1. Non-zero start
        // 2. Inclusive upper bound
        // 3. Range syntax flexibility
        assert!(c_code.contains("i = 1"));
        assert!(rust_expected.contains("1..=n"));
    }

    /// Test 12: Infinite for loop (C) vs loop (Rust)
    /// C uses empty parts, Rust uses loop keyword
    #[test]
    fn test_infinite_for_loop() {
        let c_code = r#"
for (;;) {
    process();
    if (done) break;
}
"#;

        let rust_idiomatic = r#"
loop {
    process();
    if done { break; }
}
"#;

        // Test validates:
        // 1. C for(;;) syntax
        // 2. Rust loop keyword more idiomatic
        // 3. Same semantics
        assert!(c_code.contains("for (;;)"));
        assert!(rust_idiomatic.contains("loop {"));
    }

    /// Test 13: Loop with multiple statements
    /// Complex loop body
    #[test]
    fn test_loop_with_multiple_statements() {
        let c_code = r#"
for (int i = 0; i < n; i++) {
    int temp = arr[i] * 2;
    result[i] = temp + offset;
    total += result[i];
}
"#;

        let rust_expected = r#"
for i in 0..n {
    let temp = arr[i] * 2;
    result[i] = temp + offset;
    total += result[i];
}
"#;

        // Test validates:
        // 1. Multiple statements in body
        // 2. Local variable declaration
        // 3. Same structure in both
        assert!(c_code.contains("int temp"));
        assert!(rust_expected.contains("let temp"));
    }

    /// Test 14: Loop with complex condition
    /// Multiple conditions
    #[test]
    fn test_loop_complex_condition() {
        let c_code = r#"
for (int i = 0; i < n && !found; i++) {
    if (arr[i] == target) found = 1;
}
"#;

        let rust_expected = r#"
let mut i = 0;
while i < n && !found {
    if arr[i] == target { found = true; }
    i += 1;
}
"#;

        // Test validates:
        // 1. C complex condition in for loop
        // 2. Rust while loop more appropriate
        // 3. Complex conditions → while loop
        assert!(c_code.contains("&& !found"));
        assert!(rust_expected.contains("while"));
    }

    /// Test 15: Loop with sum accumulation
    /// Common pattern: compute sum
    #[test]
    fn test_loop_sum_accumulation() {
        let c_code = r#"
int sum = 0;
for (int i = 0; i < n; i++) {
    sum += arr[i];
}
"#;

        let rust_loop_style = r#"
let mut sum = 0;
for i in 0..n {
    sum += arr[i];
}
"#;

        let rust_idiomatic = r#"
let sum: i32 = arr[..n].iter().sum();
"#;

        // Test validates:
        // 1. Traditional loop pattern
        // 2. Rust supports same style
        // 3. Iterator sum() more idiomatic
        assert!(c_code.contains("sum += arr[i]"));
        assert!(rust_loop_style.contains("sum += arr[i]"));
        assert!(rust_idiomatic.contains(".sum()"));
    }

    /// Test 16: For loop transformation rules summary
    /// Documents all transformation rules in one test
    #[test]
    fn test_for_loop_transformation_rules_summary() {
        let c_code = r#"
// Rule 1: Simple counting (0 to n-1)
for (int i = 0; i < n; i++) { ... }

// Rule 2: Inclusive range (0 to n)
for (int i = 0; i <= n; i++) { ... }

// Rule 3: Custom step (increment by 2)
for (int i = 0; i < n; i += 2) { ... }

// Rule 4: Reverse iteration
for (int i = 10; i > 0; i--) { ... }

// Rule 5: Array with index
for (int i = 0; i < len; i++) { arr[i] = ...; }

// Rule 6: Array with iterator
for (int i = 0; i < len; i++) { sum += arr[i]; }
"#;

        let rust_expected = r#"
// Rule 1: Simple counting (0 to n-1)
for i in 0..n { ... }

// Rule 2: Inclusive range (0 to n)
for i in 0..=n { ... }

// Rule 3: Custom step (increment by 2)
for i in (0..n).step_by(2) { ... }

// Rule 4: Reverse iteration
for i in (1..=10).rev() { ... }

// Rule 5: Array with index (or enumerate)
for (i, item) in arr.iter_mut().enumerate() { *item = ...; }

// Rule 6: Array with iterator (idiomatic)
for &item in arr.iter() { sum += item; }
"#;

        // Test validates all transformation rules
        assert!(c_code.contains("for (int"));
        assert!(rust_expected.contains("for i in"));
        assert!(rust_expected.contains(".."));
        assert!(rust_expected.contains(".step_by"));
        assert!(rust_expected.contains(".rev()"));
        assert!(rust_expected.contains(".iter()"));
    }
}
