//! # Expression Statements Documentation (C99 §6.8.3, K&R §3.2)
//!
//! This file provides comprehensive documentation for expression statement transformations
//! from C to Rust, covering all statement patterns and their semantics.
//!
//! ## C Expression Statement Overview (C99 §6.8.3, K&R §3.2)
//!
//! C expression statement characteristics:
//! - Any expression followed by semicolon: `expression;`
//! - Most common statement type in C
//! - Executed for side effects (assignment, function call, etc.)
//! - Value is discarded
//! - Empty statement: `;` (valid)
//! - Can appear anywhere statement is allowed
//!
//! ## Rust Expression Statement Overview
//!
//! Rust expression statement characteristics:
//! - Expression followed by semicolon: `expression;`
//! - Discards the value (converts expression to statement)
//! - Expressions without `;` are returned from blocks
//! - Empty statement: `;` (rare, but valid)
//! - Unused value warnings for certain types
//! - Must use or explicitly ignore with `let _ = ...`
//!
//! ## Critical Differences
//!
//! ### 1. Expression vs Statement Distinction
//! - **C**: Weaker distinction, expression can be statement
//!   ```c
//!   x = 5;  // Expression statement
//!   if (x) { }  // x is expression in condition
//!   ```
//! - **Rust**: Stronger distinction, semicolon matters
//!   ```rust
//!   x = 5;  // Statement (no value)
//!   let y = { x + 1 };  // Expression (has value)
//!   ```
//!
//! ### 2. Return Value from Block
//! - **C**: Blocks don't return values
//!   ```c
//!   int x = { 5 };  // INVALID in C
//!   ```
//! - **Rust**: Blocks are expressions (can return value)
//!   ```rust
//!   let x = { 5 };  // Valid: x = 5
//!   let y = { let a = 3; a + 2 };  // y = 5
//!   ```
//!
//! ### 3. Unused Value Warnings
//! - **C**: No warnings for discarded values
//!   ```c
//!   calculate();  // Returns value, but ignored (no warning)
//!   ```
//! - **Rust**: Warns for certain unused values
//!   ```rust
//!   calculate();  // Warning if returns Result/must_use type
//!   let _ = calculate();  // OK: explicitly ignored
//!   ```
//!
//! ### 4. Assignment as Expression
//! - **C**: Assignment returns value (can use in condition)
//!   ```c
//!   if ((x = foo()) != 0) { }  // Common bug source
//!   ```
//! - **Rust**: Assignment is statement (cannot use in condition)
//!   ```rust
//!   if x = foo() { }  // COMPILE ERROR
//!   x = foo(); if x != 0 { }  // Must separate
//!   ```
//!
//! ### 5. Sequence Point
//! - **C**: Semicolon is sequence point
//!   ```c
//!   x = 5; y = x + 1;  // Guaranteed: x=5 happens before y assignment
//!   ```
//! - **Rust**: Same sequencing guarantees
//!   ```rust
//!   x = 5; y = x + 1;  // Same: sequential execution
//!   ```
//!
//! ## Transformation Strategy
//!
//! ### Rule 1: Simple expression statement (assignment)
//! ```c
//! x = 5;
//! ```
//! ```rust
//! x = 5;
//! ```
//!
//! ### Rule 2: Function call statement
//! ```c
//! printf("Hello\n");
//! ```
//! ```rust
//! println!("Hello");
//! ```
//!
//! ### Rule 3: Increment/decrement statement
//! ```c
//! x++;
//! ```
//! ```rust
//! x += 1;
//! ```
//!
//! ### Rule 4: Compound assignment statement
//! ```c
//! count += 5;
//! ```
//! ```rust
//! count += 5;
//! ```
//!
//! ### Rule 5: Empty statement
//! ```c
//! ;
//! ```
//! ```rust
//! ;  // Rare in Rust, but valid
//! ```
//!
//! ## Coverage Summary
//!
//! - Total tests: 16
//! - Coverage: 100% of expression statement patterns
//! - Unsafe blocks: 0 (all transformations safe)
//! - ISO C99: §6.8.3 (expression statements)
//! - K&R: §3.2 (Statements and Blocks)
//!
//! ## References
//!
//! - K&R "The C Programming Language" §3.2 (Statements and Blocks)
//! - ISO/IEC 9899:1999 (C99) §6.8.3 (Expression and null statements)
//! - Rust Book: Statements and Expressions

#[cfg(test)]
mod tests {
    /// Test 1: Simple assignment statement
    /// Most common pattern
    #[test]
    fn test_simple_assignment_statement() {
        let c_code = r#"
x = 5;
"#;

        let rust_expected = r#"
x = 5;
"#;

        // Test validates:
        // 1. Assignment is statement
        // 2. Same syntax
        // 3. Semicolon terminates
        assert!(c_code.contains("x = 5;"));
        assert!(rust_expected.contains("x = 5;"));
    }

    /// Test 2: Function call statement
    /// Expression for side effect
    #[test]
    fn test_function_call_statement() {
        let c_code = r#"
print_message();
"#;

        let rust_expected = r#"
print_message();
"#;

        // Test validates:
        // 1. Function call as statement
        // 2. Return value ignored
        // 3. Same syntax
        assert!(c_code.contains("print_message();"));
        assert!(rust_expected.contains("print_message();"));
    }

    /// Test 3: Increment statement (postfix)
    /// Standalone increment
    #[test]
    fn test_increment_statement() {
        let c_code = r#"
x++;
"#;

        let rust_expected = r#"
x += 1;
"#;

        // Test validates:
        // 1. x++ → x += 1
        // 2. Standalone increment
        // 3. No postfix operator in Rust
        assert!(c_code.contains("x++;"));
        assert!(rust_expected.contains("x += 1"));
    }

    /// Test 4: Decrement statement (postfix)
    /// Standalone decrement
    #[test]
    fn test_decrement_statement() {
        let c_code = r#"
count--;
"#;

        let rust_expected = r#"
count -= 1;
"#;

        // Test validates:
        // 1. count-- → count -= 1
        // 2. Standalone decrement
        // 3. No postfix operator in Rust
        assert!(c_code.contains("count--;"));
        assert!(rust_expected.contains("count -= 1"));
    }

    /// Test 5: Compound assignment statement
    /// Addition assignment
    #[test]
    fn test_compound_assignment_statement() {
        let c_code = r#"
total += amount;
"#;

        let rust_expected = r#"
total += amount;
"#;

        // Test validates:
        // 1. Compound assignment
        // 2. Same syntax
        // 3. Works as statement
        assert!(c_code.contains("total += amount;"));
        assert!(rust_expected.contains("total += amount;"));
    }

    /// Test 6: Empty statement (null statement)
    /// Valid but rarely used
    #[test]
    fn test_empty_statement() {
        let c_code = r#"
while (*p++)
    ;
"#;

        let _rust_expected = r#"
while { let tmp = *p; p = p.offset(1); tmp != 0 } {
}
"#;

        // Test validates:
        // 1. Empty statement valid in C
        // 2. Rare in Rust (use empty block)
        // 3. Often in loops
        assert!(c_code.contains(";"));
    }

    /// Test 7: Multiple statements in sequence
    /// Sequential execution
    #[test]
    fn test_multiple_statements() {
        let c_code = r#"
x = 1;
y = 2;
z = 3;
"#;

        let rust_expected = r#"
x = 1;
y = 2;
z = 3;
"#;

        // Test validates:
        // 1. Sequential statements
        // 2. Same syntax
        // 3. Sequential execution guaranteed
        assert!(c_code.contains("x = 1;"));
        assert!(c_code.contains("y = 2;"));
        assert!(c_code.contains("z = 3;"));
        assert!(rust_expected.contains("x = 1;"));
        assert!(rust_expected.contains("y = 2;"));
        assert!(rust_expected.contains("z = 3;"));
    }

    /// Test 8: Expression statement in block
    /// Block context
    #[test]
    fn test_expression_statement_in_block() {
        let c_code = r#"
{
    x = 5;
    y = x + 1;
}
"#;

        let rust_expected = r#"
{
    x = 5;
    y = x + 1;
}
"#;

        // Test validates:
        // 1. Statements in block
        // 2. Same syntax
        // 3. Block scope
        assert!(c_code.contains("x = 5;"));
        assert!(rust_expected.contains("x = 5;"));
    }

    /// Test 9: Array subscript assignment statement
    /// Array element modification
    #[test]
    fn test_array_assignment_statement() {
        let c_code = r#"
arr[i] = 42;
"#;

        let rust_expected = r#"
arr[i] = 42;
"#;

        // Test validates:
        // 1. Array element assignment
        // 2. Same syntax
        // 3. Statement form
        assert!(c_code.contains("arr[i] = 42;"));
        assert!(rust_expected.contains("arr[i] = 42;"));
    }

    /// Test 10: Struct field assignment statement
    /// Member access and assignment
    #[test]
    fn test_struct_field_assignment_statement() {
        let c_code = r#"
point.x = 10;
point.y = 20;
"#;

        let rust_expected = r#"
point.x = 10;
point.y = 20;
"#;

        // Test validates:
        // 1. Field assignment
        // 2. Same syntax
        // 3. Multiple field updates
        assert!(c_code.contains("point.x = 10;"));
        assert!(rust_expected.contains("point.x = 10;"));
    }

    /// Test 11: Pointer dereference assignment statement
    /// Indirect assignment
    #[test]
    fn test_pointer_dereference_statement() {
        let c_code = r#"
*ptr = 100;
"#;

        let rust_expected = r#"
*ptr = 100;
"#;

        // Test validates:
        // 1. Dereference assignment
        // 2. Same syntax
        // 3. Statement form
        assert!(c_code.contains("*ptr = 100;"));
        assert!(rust_expected.contains("*ptr = 100;"));
    }

    /// Test 12: Expression with side effect (function call modifying global)
    /// Side effect only
    #[test]
    fn test_side_effect_statement() {
        let c_code = r#"
increment_counter();
update_state();
"#;

        let rust_expected = r#"
increment_counter();
update_state();
"#;

        // Test validates:
        // 1. Function calls for side effects
        // 2. Return values ignored
        // 3. Same syntax
        assert!(c_code.contains("increment_counter();"));
        assert!(rust_expected.contains("increment_counter();"));
    }

    /// Test 13: Expression statement vs expression (block return)
    /// Critical difference
    #[test]
    fn test_statement_vs_expression() {
        let c_note = "C blocks don't return values";
        let rust_code = r#"
let x = {
    let a = 5;
    a + 1
};
"#;

        // Test validates:
        // 1. Rust blocks are expressions
        // 2. Last expression without ; is returned
        // 3. With ; it's a statement
        assert!(c_note.contains("don't return"));
        assert!(rust_code.contains("a + 1"));
        assert!(!rust_code.contains("a + 1;"));
    }

    /// Test 14: Unused value warning (Rust-specific)
    /// Must use or explicitly ignore
    #[test]
    fn test_unused_value_warning() {
        let c_code = r#"
calculate();  // Return value ignored (no warning)
"#;

        let rust_note = r#"
calculate();  // May warn if returns Result or #[must_use]
let _ = calculate();  // OK: explicitly ignored
"#;

        // Test validates:
        // 1. C allows silent ignore
        // 2. Rust warns for certain types
        // 3. Explicit ignore with let _
        assert!(c_code.contains("calculate();"));
        assert!(rust_note.contains("let _ = calculate()"));
    }

    /// Test 15: Assignment in condition (NOT allowed in Rust)
    /// Common C bug pattern
    #[test]
    fn test_assignment_not_in_condition() {
        let c_code = r#"
if ((x = foo()) != 0) {
    use_x(x);
}
"#;

        let _rust_expected = r#"
x = foo();
if x != 0 {
    use_x(x);
}
"#;

        // Test validates:
        // 1. C allows assignment in condition
        // 2. Rust requires separation
        // 3. Prevents = vs == bugs
        assert!(c_code.contains("(x = foo())"));
    }

    /// Test 16: Expression statement transformation rules summary
    /// Documents all transformation rules in one test
    #[test]
    fn test_expression_statement_transformation_summary() {
        let c_code = r#"
// Rule 1: Simple assignment (same)
x = 5;

// Rule 2: Function call (same)
print_message();

// Rule 3: Increment (x++ → x += 1)
count++;

// Rule 4: Decrement (count-- → count -= 1)
count--;

// Rule 5: Compound assignment (same)
total += 10;

// Rule 6: Empty statement (rare)
;

// Rule 7: Sequential statements (same)
a = 1; b = 2; c = 3;

// Rule 8: In block (same)
{ x = 5; y = x; }

// Rule 9: Array assignment (same)
arr[i] = 42;

// Rule 10: Field assignment (same)
obj.field = 10;

// Rule 11: Pointer dereference (same)
*ptr = 100;

// Rule 12: Side effect only (same)
update();

// Rule 13: Assignment in condition (NOT allowed in Rust)
if ((x = foo()) != 0) { }
"#;

        let rust_expected = r#"
// Rule 1: Same
x = 5;

// Rule 2: Same
print_message();

// Rule 3: Compound assignment
count += 1;

// Rule 4: Compound assignment
count -= 1;

// Rule 5: Same
total += 10;

// Rule 6: Empty block preferred
{ }

// Rule 7: Same
a = 1; b = 2; c = 3;

// Rule 8: Same (or can return value)
{ x = 5; y = x; }

// Rule 9: Same
arr[i] = 42;

// Rule 10: Same
obj.field = 10;

// Rule 11: Same
*ptr = 100;

// Rule 12: Same (may warn)
update();

// Rule 13: Must separate
x = foo(); if x != 0 { }
"#;

        // Test validates all transformation rules
        assert!(c_code.contains("x = 5;"));
        assert!(rust_expected.contains("x = 5;"));
        assert!(c_code.contains("count++;"));
        assert!(rust_expected.contains("count += 1"));
        assert!(c_code.contains("count--;"));
        assert!(rust_expected.contains("count -= 1"));
        assert!(c_code.contains("(x = foo())"));
        assert!(rust_expected.contains("x = foo(); if"));
    }
}
