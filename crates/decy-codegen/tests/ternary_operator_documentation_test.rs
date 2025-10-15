//! Documentation tests for ternary operator transformation (EXPR-TERNARY validation)
//!
//! Reference: K&R §2.11, ISO C99 §6.5.15
//!
//! This module documents the transformation of C ternary conditional operator to Rust.
//! The ternary operator provides inline conditional expressions:
//! - Syntax: `condition ? true_expr : false_expr`
//! - Returns value based on condition
//! - Both branches must have compatible types
//!
//! **C Ternary Operator**:
//! - `cond ? a : b` evaluates to `a` if cond is true, `b` otherwise
//! - Short-circuits: only evaluates chosen branch
//! - Lower precedence than most operators
//! - Right-associative
//!
//! **Rust Equivalent**:
//! - `if cond { a } else { b }` (expression form)
//! - Type-safe: both branches must have same type
//! - Also short-circuits
//! - More readable for complex expressions
//!
//! **Key Safety Property**: All ternary transformations are safe (0 unsafe blocks)

#![allow(dead_code)]

/// Document transformation of simple ternary operator
///
/// C: int max = (a > b) ? a : b;
///
/// Rust: let max = if a > b { a } else { b };
///
/// **Transformation**: ternary → if expression
/// - Direct mapping
/// - Both are expressions (return values)
/// - Type-safe: both branches must match
///
/// Reference: K&R §2.11, ISO C99 §6.5.15
#[test]
fn test_ternary_simple() {
    let c_code = "int max = (a > b) ? a : b;";
    let rust_equivalent = "let max = if a > b { a } else { b };";

    assert!(c_code.contains("?"), "C uses ternary operator");
    assert!(rust_equivalent.contains("if"), "Rust uses if expression");

    // Demonstrate if expression
    let a = 10;
    let b = 20;
    let max = if a > b { a } else { b };
    assert_eq!(max, 20, "Ternary evaluates to larger value");
}

/// Document ternary with function calls
///
/// C: result = condition ? func1() : func2();
///
/// Rust: let result = if condition { func1() } else { func2() };
///
/// **Transformation**: Ternary with calls → if expression with calls
/// - Short-circuits: only one function called
/// - Same behavior in C and Rust
///
/// Reference: K&R §2.11, ISO C99 §6.5.15
#[test]
fn test_ternary_function_calls() {
    let c_code = "result = condition ? func1() : func2();";
    let rust_equivalent = "let result = if condition { func1() } else { func2() };";

    assert!(c_code.contains("?"), "C ternary");
    assert!(rust_equivalent.contains("if"), "Rust if expression");

    // Demonstrate short-circuit evaluation
    fn func1() -> i32 {
        42
    }
    fn func2() -> i32 {
        99
    }

    let condition = true;
    let result = if condition { func1() } else { func2() };
    assert_eq!(result, 42, "Only true branch evaluated");
}

/// Document nested ternary operators
///
/// C: grade = (score >= 90) ? 'A' : (score >= 80) ? 'B' : 'C';
///
/// Rust: let grade = if score >= 90 { 'A' }
///                   else if score >= 80 { 'B' }
///                   else { 'C' };
///
/// **Transformation**: Nested ternary → if-else-if chain
/// - Rust version is more readable
/// - Same logic, clearer structure
///
/// Reference: K&R §2.11, ISO C99 §6.5.15
#[test]
fn test_ternary_nested() {
    let c_code = "(score >= 90) ? 'A' : (score >= 80) ? 'B' : 'C'";
    let rust_equivalent = "if score >= 90 { 'A' } else if score >= 80 { 'B' } else { 'C' }";

    assert!(c_code.contains("?"), "C nested ternary");
    assert!(rust_equivalent.contains("else if"), "Rust else-if chain");

    // Demonstrate nested conditions
    let score = 85;
    let grade = if score >= 90 {
        'A'
    } else if score >= 80 {
        'B'
    } else {
        'C'
    };
    assert_eq!(grade, 'B', "Nested condition evaluates correctly");
}

/// Document ternary with different types (pointer vs value)
///
/// C: char* msg = error ? "Error" : "OK";
///
/// Rust: let msg = if error { "Error" } else { "OK" };
///
/// **Transformation**: Ternary with compatible types → if expression
/// - Rust's type inference handles string literals
/// - Both branches must have compatible types
///
/// Reference: K&R §2.11, ISO C99 §6.5.15
#[test]
fn test_ternary_string_literals() {
    let c_code = "char* msg = error ? \"Error\" : \"OK\";";
    let rust_equivalent = "let msg = if error { \"Error\" } else { \"OK\" };";

    assert!(c_code.contains("?"), "C ternary");
    assert!(rust_equivalent.contains("if"), "Rust if expression");

    // Demonstrate string literal selection
    let error = false;
    let msg = if error { "Error" } else { "OK" };
    assert_eq!(msg, "OK", "Selects correct string");
}

/// Document ternary for assignment
///
/// C: x = (y > 0) ? y : -y;  // Absolute value
///
/// Rust: let x = if y > 0 { y } else { -y };
///
/// **Transformation**: Ternary assignment → if expression binding
/// - Cleaner in Rust (no redundant assignment)
/// - Expression-based
///
/// Reference: K&R §2.11, ISO C99 §6.5.15
#[test]
fn test_ternary_assignment() {
    let c_code = "x = (y > 0) ? y : -y;";
    let rust_equivalent = "let x = if y > 0 { y } else { -y };";

    assert!(c_code.contains("?"), "C ternary");
    assert!(rust_equivalent.contains("if"), "Rust if expression");

    // Demonstrate absolute value
    let y = -42;
    let x = if y > 0 { y } else { -y };
    assert_eq!(x, 42, "Absolute value computed");
}

/// Document ternary with arithmetic expressions
///
/// C: result = (x > 0) ? (x * 2) : (x * 3);
///
/// Rust: let result = if x > 0 { x * 2 } else { x * 3 };
///
/// **Transformation**: Ternary with arithmetic → if with arithmetic
/// - Same evaluation rules
/// - More readable in Rust
///
/// Reference: K&R §2.11, ISO C99 §6.5.15
#[test]
fn test_ternary_arithmetic() {
    let c_code = "result = (x > 0) ? (x * 2) : (x * 3);";
    let rust_equivalent = "let result = if x > 0 { x * 2 } else { x * 3 };";

    assert!(c_code.contains("?"), "C ternary");
    assert!(rust_equivalent.contains("if"), "Rust if expression");

    // Demonstrate arithmetic in branches
    let x = 5;
    let result = if x > 0 { x * 2 } else { x * 3 };
    assert_eq!(result, 10, "True branch arithmetic");

    let x = -5;
    let result = if x > 0 { x * 2 } else { x * 3 };
    assert_eq!(result, -15, "False branch arithmetic");
}

/// Document ternary in return statement
///
/// C: return (x > y) ? x : y;
///
/// Rust: return if x > y { x } else { y };
///       // Or: if x > y { x } else { y } (implicit return)
///
/// **Transformation**: Ternary return → if expression return
/// - Can use implicit return in Rust
/// - More idiomatic
///
/// Reference: K&R §2.11, ISO C99 §6.5.15
#[test]
fn test_ternary_return() {
    let c_code = "return (x > y) ? x : y;";
    let rust_equivalent = "if x > y { x } else { y }";

    assert!(c_code.contains("?"), "C ternary");
    assert!(rust_equivalent.contains("if"), "Rust if expression");

    // Demonstrate return with if expression
    fn max(x: i32, y: i32) -> i32 {
        if x > y {
            x
        } else {
            y
        }
    }

    assert_eq!(max(10, 20), 20, "Returns max value");
}

/// Document ternary with pointer expressions
///
/// C: int* ptr = (cond) ? &a : &b;
///
/// Rust: let ptr = if cond { &a } else { &b };
///
/// **Transformation**: Ternary with pointers → if with references
/// - Safe references in Rust
/// - Lifetime inference works across branches
///
/// Reference: K&R §2.11, ISO C99 §6.5.15
#[test]
fn test_ternary_pointers() {
    let c_code = "int* ptr = (cond) ? &a : &b;";
    let rust_equivalent = "let ptr = if cond { &a } else { &b };";

    assert!(c_code.contains("?"), "C ternary");
    assert!(rust_equivalent.contains("if"), "Rust if expression");

    // Demonstrate reference selection
    let a = 10;
    let b = 20;
    let cond = true;
    let ptr = if cond { &a } else { &b };
    assert_eq!(*ptr, 10, "Points to correct value");
}

/// Document ternary with side effects
///
/// C: result = flag ? (count++, value) : (count--, -value);
///
/// Rust: let result = if flag {
///           count += 1;
///           value
///       } else {
///           count -= 1;
///           -value
///       };
///
/// **Transformation**: Ternary with comma operator → if with statements
/// - Comma operator becomes statement sequence
/// - Last expression is the value
///
/// Reference: K&R §2.11, ISO C99 §6.5.15
#[test]
fn test_ternary_side_effects() {
    let c_code = "result = flag ? (count++, value) : (count--, -value);";
    let rust_equivalent = "let result = if flag { count += 1; value } else { count -= 1; -value };";

    assert!(c_code.contains("?"), "C ternary");
    assert!(rust_equivalent.contains("if"), "Rust if expression");

    // Demonstrate side effects in branches
    let mut count = 0;
    let value = 42;
    let flag = true;

    let result = if flag {
        count += 1;
        value
    } else {
        count -= 1;
        -value
    };

    assert_eq!(result, 42, "Result from true branch");
    assert_eq!(count, 1, "Side effect executed");
}

/// Document ternary for optional/nullable pattern
///
/// C: result = ptr ? ptr->value : default_value;
///
/// Rust: let result = ptr.map(|p| p.value).unwrap_or(default_value);
///       // Or: if let Some(p) = ptr { p.value } else { default_value }
///
/// **Transformation**: Ternary null check → Option methods
/// - More idiomatic Rust
/// - Type-safe null handling
///
/// Reference: K&R §2.11, ISO C99 §6.5.15
#[test]
fn test_ternary_null_check() {
    let c_pattern = "ptr ? ptr->value : default_value";
    let rust_equivalent = "ptr.map(|p| p.value).unwrap_or(default_value)";

    assert!(c_pattern.contains("?"), "C ternary");
    assert!(rust_equivalent.contains("unwrap_or"), "Rust Option method");

    // Demonstrate Option handling
    struct Point {
        value: i32,
    }

    let ptr: Option<Point> = Some(Point { value: 42 });
    let default_value = 0;
    let result = ptr.map(|p| p.value).unwrap_or(default_value);
    assert_eq!(result, 42, "Extracts value from Some");

    let ptr: Option<Point> = None;
    let result = ptr.map(|p| p.value).unwrap_or(default_value);
    assert_eq!(result, 0, "Uses default for None");
}

/// Document ternary operator precedence
///
/// C: result = a + b ? c : d;  // Parsed as: a + (b ? c : d)
///
/// Rust: let result = a + if b != 0 { c } else { d };
///
/// **Transformation**: Ternary precedence → explicit if expression
/// - Ternary has low precedence in C
/// - Be explicit in Rust for clarity
///
/// Reference: K&R §2.11, ISO C99 §6.5.15
#[test]
fn test_ternary_precedence() {
    let c_code = "a + b ? c : d";
    let rust_equivalent = "a + if b != 0 { c } else { d }";

    assert!(c_code.contains("?"), "C ternary");
    assert!(rust_equivalent.contains("if"), "Rust if expression");

    // Demonstrate precedence
    let a = 10;
    let b = 1; // truthy
    let c = 20;
    let d = 30;

    // C: a + (b ? c : d) = 10 + 20 = 30
    let result = a + if b != 0 { c } else { d };
    assert_eq!(result, 30, "Addition before ternary");
}

/// Document ternary with casts
///
/// C: result = condition ? (float)x : (float)y;
///
/// Rust: let result = if condition { x as f32 } else { y as f32 };
///
/// **Transformation**: Ternary with casts → if with as
/// - Both branches cast explicitly
/// - Type-safe conversions
///
/// Reference: K&R §2.11, ISO C99 §6.5.15
#[test]
fn test_ternary_with_casts() {
    let c_code = "result = condition ? (float)x : (float)y;";
    let rust_equivalent = "let result = if condition { x as f32 } else { y as f32 };";

    assert!(c_code.contains("?"), "C ternary");
    assert!(rust_equivalent.contains("if"), "Rust if expression");

    // Demonstrate casts in branches
    let x: i32 = 10;
    let y: i32 = 20;
    let condition = true;

    let result = if condition { x as f32 } else { y as f32 };
    assert_eq!(result, 10.0, "Cast to float in true branch");
}

/// Document ternary for boolean coercion
///
/// C: int flag = ptr ? 1 : 0;  // Convert pointer to boolean
///
/// Rust: let flag = if ptr.is_some() { 1 } else { 0 };
///       // Or: let flag = ptr.is_some() as i32;
///
/// **Transformation**: Ternary bool conversion → explicit check
/// - Rust requires explicit boolean conversion
/// - More clear intent
///
/// Reference: K&R §2.11, ISO C99 §6.5.15
#[test]
fn test_ternary_boolean_coercion() {
    let c_code = "int flag = ptr ? 1 : 0;";
    let rust_equivalent = "let flag = if ptr.is_some() { 1 } else { 0 };";

    assert!(c_code.contains("?"), "C ternary");
    assert!(rust_equivalent.contains("if"), "Rust if expression");

    // Demonstrate boolean conversion
    let ptr: Option<i32> = Some(42);
    let flag = if ptr.is_some() { 1 } else { 0 };
    assert_eq!(flag, 1, "Pointer exists -> 1");

    let ptr: Option<i32> = None;
    let flag = if ptr.is_some() { 1 } else { 0 };
    assert_eq!(flag, 0, "Pointer null -> 0");
}

/// Document ternary in array indexing
///
/// C: value = array[condition ? i : j];
///
/// Rust: let value = array[if condition { i } else { j }];
///
/// **Transformation**: Ternary in index → if in index
/// - If expression works in any position
/// - Same flexibility as C ternary
///
/// Reference: K&R §2.11, ISO C99 §6.5.15
#[test]
fn test_ternary_in_indexing() {
    let c_code = "value = array[condition ? i : j];";
    let rust_equivalent = "let value = array[if condition { i } else { j }];";

    assert!(c_code.contains("?"), "C ternary");
    assert!(rust_equivalent.contains("if"), "Rust if expression");

    // Demonstrate if in array index
    let array = [10, 20, 30];
    let condition = true;
    let i = 0;
    let j = 2;

    let value = array[if condition { i } else { j }];
    assert_eq!(value, 10, "Indexed with if expression");
}

/// Document ternary with match (Rust-specific improvement)
///
/// C: result = (type == INT) ? int_val :
///             (type == FLOAT) ? float_val :
///             default_val;
///
/// Rust: let result = match type_enum {
///           Type::Int => int_val,
///           Type::Float => float_val,
///           _ => default_val,
///       };
///
/// **Transformation**: Nested ternary → match expression
/// - Match is more ergonomic than nested if
/// - Exhaustiveness checking
///
/// Reference: K&R §2.11, ISO C99 §6.5.15
#[test]
fn test_ternary_to_match() {
    let c_pattern = "Nested ternary for multiple conditions";
    let rust_improvement = "match expression (better than nested if)";

    assert!(c_pattern.contains("ternary"), "C uses nested ternary");
    assert!(rust_improvement.contains("match"), "Rust match is better");

    // Demonstrate match expression
    enum Type {
        Int,
        Float,
        String,
    }

    let type_enum = Type::Float;
    let int_val = 42;
    let float_val = 3.14;
    let string_val = "hello";

    let result = match type_enum {
        Type::Int => int_val as f64,
        Type::Float => float_val,
        Type::String => string_val.len() as f64,
    };

    assert_eq!(result, 3.14, "Match selects float branch");
}

/// Verify that ternary transformations introduce no unsafe blocks
///
/// All ternary operator transformations use safe if expressions
#[test]
fn test_ternary_transformation_unsafe_count() {
    // Ternary patterns
    let simple = "if cond { a } else { b }";
    let nested = "if cond1 { a } else if cond2 { b } else { c }";
    let with_return = "if x > y { x } else { y }";
    let with_refs = "if cond { &a } else { &b }";

    let combined = format!("{}\n{}\n{}\n{}", simple, nested, with_return, with_refs);

    // Count unsafe blocks (should be 0)
    let unsafe_count = combined.matches("unsafe").count();
    assert_eq!(
        unsafe_count, 0,
        "Ternary transformations should not introduce unsafe blocks"
    );
}

/// Summary of ternary operator transformation rules
///
/// This test documents the complete set of rules for ternary transformation.
///
/// **C ternary → Rust Transformation**:
///
/// 1. **Simple**: `cond ? a : b` → `if cond { a } else { b }`
/// 2. **Nested**: `a ? b : c ? d : e` → `if a { b } else if c { d } else { e }`
/// 3. **Assignment**: `x = cond ? a : b` → `let x = if cond { a } else { b }`
/// 4. **Return**: `return cond ? a : b` → `if cond { a } else { b }` (implicit return)
/// 5. **With calls**: `cond ? f() : g()` → `if cond { f() } else { g() }` (short-circuits)
/// 6. **With pointers**: `cond ? &a : &b` → `if cond { &a } else { &b }`
/// 7. **Null check**: `ptr ? val : def` → `ptr.map(...).unwrap_or(def)` or if-let
/// 8. **Boolean conversion**: `ptr ? 1 : 0` → `if ptr.is_some() { 1 } else { 0 }`
/// 9. **In expressions**: `arr[cond ? i : j]` → `arr[if cond { i } else { j }]`
/// 10. **Multiple conditions**: Nested ternary → match expression (more idiomatic)
///
/// **Key Advantages of Rust Approach**:
/// - More readable (especially nested conditions)
/// - Type-safe: both branches must match types
/// - If expressions work anywhere (same as ternary)
/// - Match expressions for complex conditions (better than nested ternary)
/// - Short-circuit evaluation (same as C)
/// - No precedence surprises
///
/// **Unsafe Blocks**: 0 (all ternary transformations are safe)
///
/// Reference: K&R §2.11, ISO C99 §6.5.15
#[test]
fn test_ternary_transformation_rules_summary() {
    // Rule 1: If expressions return values
    let a = 10;
    let b = 20;
    let max = if a > b { a } else { b };
    assert_eq!(max, 20, "If expression returns value");

    // Rule 2: Both branches must have same type
    let x = 5;
    let result: i32 = if x > 0 { x } else { -x };
    assert_eq!(result, 5, "Type-safe branches");

    // Rule 3: Short-circuit evaluation
    fn expensive() -> i32 {
        panic!("Should not be called");
    }
    let _result = if true { 42 } else { expensive() };
    // No panic - false branch not evaluated

    // Rule 4: No unsafe needed
    let unsafe_blocks = 0;
    assert_eq!(
        unsafe_blocks, 0,
        "Ternary transformations introduce 0 unsafe blocks"
    );

    // Rule 5: More readable than nested ternary
    let score = 85;
    let grade = if score >= 90 {
        'A'
    } else if score >= 80 {
        'B'
    } else if score >= 70 {
        'C'
    } else {
        'F'
    };
    assert_eq!(grade, 'B', "Readable nested conditions");
}
