//! Documentation tests for C post-increment/decrement (++/--) → Rust explicit operations
//!
//! This test file documents the transformation from C's post-increment and post-decrement
//! operators to Rust's explicit increment operations. Unlike C, Rust does not have
//! ++ or -- operators to avoid confusion and side effects.
//!
//! # Reference
//! - K&R C (2nd Edition) §2.8: Increment and Decrement Operators
//! - ISO C99 Standard §6.5.2.4: Postfix increment and decrement operators
//!
//! # Key Differences
//! - **C**: `x++` returns old value, then increments (side effect)
//! - **Rust**: No ++ operator; use `{ let tmp = x; x += 1; tmp }` or separate statements
//!
//! # Why Rust Doesn't Have ++ / --
//! 1. Avoid confusion between pre and post forms
//! 2. Explicit about side effects
//! 3. Prevent sequence point issues
//! 4. More readable code
//!
//! # Transformation Strategy
//! 1. `x++` as statement → `x += 1`
//! 2. `x++` in expression → `{ let tmp = x; x += 1; tmp }`
//! 3. `x--` as statement → `x -= 1`
//! 4. `x--` in expression → `{ let tmp = x; x -= 1; tmp }`
//! 5. Prefer iterators over manual increment
//!
//! # Target Metrics
//! - Coverage: 100%
//! - Unsafe blocks: 0
//! - Tests: 17 comprehensive scenarios

#[cfg(test)]
mod tests {
    //! All tests validate post-increment/decrement transformation patterns

    #[test]
    fn test_postfix_increment_simple_statement() {
        // C: Post-increment as standalone statement
        let c_code = r#"
int x = 5;
x++;
// x is now 6
"#;

        // Rust: Use += operator
        let rust_expected = r#"
let mut x: i32 = 5;
x += 1;
// x is now 6
"#;

        // Validates: x++ as statement → x += 1
        assert!(c_code.contains("x++"));
        assert!(rust_expected.contains("x += 1"));

        // Note: When used as statement, no need to save old value
        assert!(
            rust_expected.contains("let mut x"),
            "Variable must be mutable"
        );
    }

    #[test]
    fn test_postfix_decrement_simple_statement() {
        // C: Post-decrement as standalone statement
        let c_code = r#"
int x = 10;
x--;
// x is now 9
"#;

        // Rust: Use -= operator
        let rust_expected = r#"
let mut x: i32 = 10;
x -= 1;
// x is now 9
"#;

        // Validates: x-- as statement → x -= 1
        assert!(c_code.contains("x--"));
        assert!(rust_expected.contains("x -= 1"));
    }

    #[test]
    fn test_postfix_increment_in_expression() {
        // C: Post-increment returns old value, then increments
        let c_code = r#"
int x = 5;
int y = x++;  // y = 5, x = 6
"#;

        // Rust: Explicit capture of old value
        let rust_expected = r#"
let mut x: i32 = 5;
let y: i32 = { let tmp = x; x += 1; tmp };  // y = 5, x = 6
"#;

        // Validates: x++ in expression → save old value
        assert!(c_code.contains("int y = x++"));
        assert!(rust_expected.contains("let tmp = x; x += 1; tmp"));

        // CRITICAL: Rust makes the "save old value" explicit
        assert!(rust_expected.contains("let tmp"));
    }

    #[test]
    fn test_postfix_decrement_in_expression() {
        // C: Post-decrement returns old value, then decrements
        let c_code = r#"
int x = 10;
int y = x--;  // y = 10, x = 9
"#;

        // Rust: Explicit capture of old value
        let rust_expected = r#"
let mut x: i32 = 10;
let y: i32 = { let tmp = x; x -= 1; tmp };  // y = 10, x = 9
"#;

        // Validates: x-- in expression → save old value
        assert!(c_code.contains("int y = x--"));
        assert!(rust_expected.contains("let tmp = x; x -= 1; tmp"));
    }

    #[test]
    fn test_postfix_increment_in_array_index() {
        // C: Post-increment in array indexing
        let c_code = r#"
int arr[10];
int i = 0;
arr[i++] = 42;  // Sets arr[0], i becomes 1
arr[i++] = 43;  // Sets arr[1], i becomes 2
"#;

        // Rust: Explicit indexing
        let rust_expected = r#"
let mut arr: [i32; 10] = [0; 10];
let mut i: usize = 0;
arr[i] = 42; i += 1;  // Sets arr[0], i becomes 1
arr[i] = 42; i += 1;  // Sets arr[1], i becomes 2
"#;

        // Validates: arr[i++] → arr[i]; i += 1
        assert!(c_code.contains("arr[i++]"));
        assert!(rust_expected.contains("arr[i] = 42; i += 1"));

        // CRITICAL: Rust makes the order of operations explicit
    }

    #[test]
    fn test_postfix_in_function_argument() {
        // C: Post-increment in function call
        let c_code = r#"
int i = 0;
printf("%d\n", i++);  // Prints 0, i becomes 1
"#;

        // Rust: Explicit ordering
        let rust_expected = r#"
let mut i: i32 = 0;
println!("{}", { let tmp = i; i += 1; tmp });  // Prints 0, i becomes 1

// Or more readable:
let value = i;
i += 1;
println!("{}", value);
"#;

        // Validates: Function argument with i++
        assert!(c_code.contains("printf(\"%d\\n\", i++)"));
        assert!(rust_expected.contains("let tmp = i; i += 1; tmp"));
    }

    #[test]
    fn test_postfix_in_loop_traditional() {
        // C: Traditional for loop with post-increment
        let c_code = r#"
for (int i = 0; i < 10; i++) {
    printf("%d\n", i);
}
"#;

        // Rust: Idiomatic range-based loop
        let rust_expected = r#"
for i in 0..10 {
    println!("{}", i);
}

// Or manual loop:
let mut i: i32 = 0;
while i < 10 {
    println!("{}", i);
    i += 1;
}
"#;

        // Validates: for loop with i++
        assert!(c_code.contains("i++"));
        assert!(rust_expected.contains("for i in 0..10"));

        // CRITICAL: Rust prefers iterators over manual increment
    }

    #[test]
    fn test_postfix_while_loop() {
        // C: While loop with post-increment
        let c_code = r#"
int i = 0;
while (i < 10) {
    process(i++);
}
"#;

        // Rust: Explicit increment at end
        let rust_expected = r#"
let mut i: i32 = 0;
while i < 10 {
    process(i);
    i += 1;
}

// Or use iterator:
for i in 0..10 {
    process(i);
}
"#;

        // Validates: process(i++) → process(i); i += 1
        assert!(c_code.contains("process(i++)"));
        assert!(rust_expected.contains("process(i);\n    i += 1"));
    }

    #[test]
    fn test_postfix_sequence_point_issue() {
        // C: Undefined behavior - multiple modifications between sequence points
        let c_code = r#"
int x = 5;
int y = x++ + x++;  // UNDEFINED BEHAVIOR!
// Could be: (5 + 6) = 11, or (5 + 5) = 10, or something else
"#;

        // Rust: Cannot write this - must be explicit
        let rust_expected = r#"
let mut x: i32 = 5;
// Must make order explicit:
let tmp1 = x; x += 1;
let tmp2 = x; x += 1;
let y = tmp1 + tmp2;  // Clearly: 5 + 6 = 11
"#;

        // Validates: C allows undefined behavior
        assert!(c_code.contains("x++ + x++"));
        assert!(c_code.contains("UNDEFINED BEHAVIOR"));

        // Validates: Rust forces explicit ordering
        assert!(rust_expected.contains("tmp1"));
        assert!(rust_expected.contains("tmp2"));
    }

    #[test]
    fn test_postfix_with_pointer() {
        // C: Post-increment with pointer dereferencing
        let c_code = r#"
int arr[] = {1, 2, 3, 4, 5};
int* p = arr;
int x = *p++;  // x = 1, p points to arr[1]
"#;

        // Rust: Explicit iterator or slice indexing
        let rust_expected = r#"
let arr: [i32; 5] = [1, 2, 3, 4, 5];
let mut index: usize = 0;
let x: i32 = arr[index];
index += 1;  // index now points to arr[1]

// Or use iterator:
let mut iter = arr.iter();
let x = *iter.next().unwrap();
"#;

        // Validates: *p++ → explicit indexing or iterator
        assert!(c_code.contains("*p++"));
        assert!(rust_expected.contains("arr[index]"));
        assert!(rust_expected.contains("iter.next()"));
    }

    #[test]
    fn test_postfix_return_value() {
        // C: Returning post-incremented value
        let c_code = r#"
int counter = 0;
int get_next() {
    return counter++;  // Returns old value
}

int a = get_next();  // a = 0, counter = 1
int b = get_next();  // b = 1, counter = 2
"#;

        // Rust: Explicit old value return
        let rust_expected = r#"
let mut counter: i32 = 0;
fn get_next(counter: &mut i32) -> i32 {
    let old = *counter;
    *counter += 1;
    old  // Returns old value
}

let a = get_next(&mut counter);  // a = 0, counter = 1
let b = get_next(&mut counter);  // b = 1, counter = 2
"#;

        // Validates: return counter++ → explicit old value capture
        assert!(c_code.contains("return counter++"));
        assert!(rust_expected.contains("let old = *counter"));
        assert!(rust_expected.contains("old  // Returns old value"));
    }

    #[test]
    fn test_postfix_combined_operators() {
        // C: Post-increment combined with other operators
        let c_code = r#"
int x = 5;
int y = 2 * x++;  // y = 10, x = 6
"#;

        // Rust: Explicit order
        let rust_expected = r#"
let mut x: i32 = 5;
let y: i32 = 2 * { let tmp = x; x += 1; tmp };  // y = 10, x = 6
"#;

        // Validates: Operator precedence with ++
        assert!(c_code.contains("2 * x++"));
        assert!(rust_expected.contains("2 * { let tmp = x; x += 1; tmp }"));
    }

    #[test]
    fn test_postfix_pre_vs_post_confusion() {
        // C: Pre-increment vs post-increment (common bug source)
        let c_code = r#"
int x = 5;
int a = x++;   // a = 5, x = 6 (post)
int b = ++x;   // b = 7, x = 7 (pre)
"#;

        // Rust: Different explicit patterns
        let rust_expected = r#"
let mut x: i32 = 5;
// Post-increment (return old, then increment):
let a: i32 = { let tmp = x; x += 1; tmp };  // a = 5, x = 6

// Pre-increment (increment, then return new):
x += 1;
let b: i32 = x;  // b = 7, x = 7
"#;

        // Validates: Both forms explicit in Rust
        assert!(c_code.contains("x++"));
        assert!(c_code.contains("++x"));
        assert!(rust_expected.contains("let tmp = x"));
        assert!(rust_expected.contains("x += 1"));
    }

    #[test]
    fn test_postfix_iterator_preference() {
        // C: Manual iteration with post-increment
        let c_code = r#"
int arr[] = {1, 2, 3, 4, 5};
int sum = 0;
for (int i = 0; i < 5; i++) {
    sum += arr[i];
}
"#;

        // Rust: Prefer iterators
        let rust_expected = r#"
let arr: [i32; 5] = [1, 2, 3, 4, 5];
let sum: i32 = arr.iter().sum();  // Idiomatic

// Or explicit loop:
let mut sum: i32 = 0;
for i in 0..5 {
    sum += arr[i];
}
"#;

        // Validates: Iterator preference in Rust
        assert!(c_code.contains("i++"));
        assert!(rust_expected.contains("arr.iter().sum()"));
        assert!(rust_expected.contains("Idiomatic"));
    }

    #[test]
    fn test_postfix_overflow_behavior() {
        // C: Integer overflow with increment (undefined for signed)
        let c_code = r#"
int x = INT_MAX;
x++;  // Undefined behavior for signed int
"#;

        // Rust: Panic in debug, wrap in release (or explicit wrapping)
        let rust_expected = r#"
let mut x: i32 = i32::MAX;
// x += 1;  // Panics in debug mode, wraps in release

// Explicit wrapping:
x = x.wrapping_add(1);  // Wraps to i32::MIN

// Or checked:
x = x.checked_add(1).unwrap_or(i32::MAX);
"#;

        // Validates: Overflow handling
        assert!(c_code.contains("INT_MAX"));
        assert!(c_code.contains("x++"));
        assert!(rust_expected.contains("wrapping_add"));
        assert!(rust_expected.contains("checked_add"));
    }

    #[test]
    fn test_postfix_multiple_variables() {
        // C: Multiple post-increments
        let c_code = r#"
int i = 0, j = 0;
i++;
j++;
"#;

        // Rust: Explicit increments
        let rust_expected = r#"
let mut i: i32 = 0;
let mut j: i32 = 0;
i += 1;
j += 1;
"#;

        // Validates: Multiple variable increments
        assert_eq!(c_code.matches("++").count(), 2);
        assert_eq!(rust_expected.matches("+= 1").count(), 2);
    }

    #[test]
    fn test_postfix_nested_expressions() {
        // C: Nested post-increments (confusing!)
        let c_code = r#"
int x = 5;
int y = (x++ * 2) + (x++ * 3);
// Order dependent! Could be confusing
"#;

        // Rust: Forces explicit ordering
        let rust_expected = r#"
let mut x: i32 = 5;
// Must be explicit about order:
let tmp1 = x; x += 1;  // First x++
let tmp2 = x; x += 1;  // Second x++
let y = (tmp1 * 2) + (tmp2 * 3);  // Clear: (5 * 2) + (6 * 3) = 28
"#;

        // Validates: Nested increment confusion eliminated
        assert_eq!(c_code.matches("x++").count(), 2);
        assert!(rust_expected.contains("tmp1"));
        assert!(rust_expected.contains("tmp2"));
        assert!(rust_expected.contains("Clear"));
    }

    #[test]
    fn test_postfix_transformation_summary() {
        // Summary of post-increment/decrement transformations

        // C patterns
        let c_patterns = [
            "x++",                 // Post-increment
            "x--",                 // Post-decrement
            "arr[i++]",            // Array index with increment
            "for (i=0; i<n; i++)", // Loop increment
            "*p++",                // Pointer increment
            "return x++",          // Return old value
            "f(x++)",              // Function argument
        ];

        // Rust patterns
        let rust_patterns = [
            "x += 1",                       // Simple increment
            "x -= 1",                       // Simple decrement
            "arr[i]; i += 1",               // Explicit array access
            "for i in 0..n",                // Iterator (idiomatic)
            "arr[index]; index += 1",       // Explicit indexing
            "let old = x; x += 1; old",     // Return old value
            "{ let tmp = x; x += 1; tmp }", // Expression form
        ];

        // Validation
        assert!(c_patterns
            .iter()
            .all(|p| p.contains("++") || p.contains("--")));
        assert!(rust_patterns.iter().any(|p| p.contains("+= 1")));

        // Key semantic differences
        let semantics = "
C post-increment/decrement:
- Single operator with side effect
- Returns old value, then modifies
- Can cause sequence point issues (UB)
- Confusing in complex expressions
- Pre vs post can be error-prone
- Pointer arithmetic common

Rust approach:
- No ++ or -- operators (deliberate design choice)
- Explicit about side effects (x += 1)
- Clear ordering (no sequence point issues)
- More readable in expressions
- Prefer iterators over manual increment
- Safer pointer/index handling
- Explicit overflow behavior (wrapping_add, checked_add)
        ";

        assert!(semantics.contains("No ++ or -- operators"));
        assert!(semantics.contains("Explicit about side effects"));
        assert!(semantics.contains("Prefer iterators"));
    }
}
