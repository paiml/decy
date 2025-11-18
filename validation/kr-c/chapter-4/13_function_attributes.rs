/* K&R C Chapter 4: Function Attributes
 * GCC function attributes for optimization and control
 * Transpiled to safe Rust (using Rust attributes)
 */

use std::process;

// Rust equivalent: functions that don't return use -> !
fn fatal_error(msg: &str) -> ! {
    eprintln!("FATAL: {}", msg);
    process::exit(1);
}

// Pure function in Rust: no side effects, result depends only on args
// Rust doesn't have explicit "pure" attribute, but const fn is similar
const fn pure_add(a: i32, b: i32) -> i32 {
    a + b
}

// Const function - can be evaluated at compile time
const fn const_multiply(a: i32, b: i32) -> i32 {
    a * b
}

// Rust has #[must_use] for warn_unused_result
#[must_use]
fn important_function() -> i32 {
    42
}

// Always inline
#[inline(always)]
fn force_inline(x: i32) -> i32 {
    x * 2
}

// Never inline
#[inline(never)]
fn never_inline(x: i32) -> i32 {
    x * 3
}

// Deprecated function
#[deprecated(since = "1.0.0", note = "use new_function instead")]
fn old_function() {
    println!("This function is deprecated");
}

#[allow(dead_code)]
fn new_function() {
    println!("This is the new function");
}

// Rust's println! macro has compile-time format string checking
// No need for format attribute - it's built into the macro system

fn main() {
    println!("=== Function Attributes ===\n");

    // Pure and const functions
    let result1 = pure_add(10, 20);
    let result2 = const_multiply(5, 6);

    println!("pure_add(10, 20) = {}", result1);
    println!("const_multiply(5, 6) = {}", result2);

    // Inline attributes
    let val = force_inline(5);
    println!("force_inline(5) = {}", val);

    let val = never_inline(7);
    println!("never_inline(7) = {}", val);

    // Important function - should use result
    let important = important_function();
    println!("important_function() = {}", important);

    // This would trigger a warning: important_function();

    // Using deprecated function (will warn)
    // old_function();  // Uncomment to see deprecation warning

    // Format checking is automatic with println! macro
    println!("Testing format: {}, {}", 123, "hello");

    // Noreturn example (commented to avoid exit)
    // fatal_error("Program terminated");
}

// Compile-time evaluation demonstration
const COMPILE_TIME_RESULT: i32 = const_multiply(5, 6);

#[allow(dead_code)]
fn show_compile_time() {
    println!("Computed at compile time: {}", COMPILE_TIME_RESULT);
}

// Key Rust attributes vs GCC attributes:
// - __attribute__((noreturn))     -> -> !
// - __attribute__((pure))          -> const fn (stronger guarantee)
// - __attribute__((const))         -> const fn
// - __attribute__((warn_unused))   -> #[must_use]
// - __attribute__((always_inline)) -> #[inline(always)]
// - __attribute__((noinline))      -> #[inline(never)]
// - __attribute__((deprecated))    -> #[deprecated]
// - __attribute__((format))        -> Built into macros
