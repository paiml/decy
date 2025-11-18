/* K&R C Chapter 4: Inline Functions (C99)
 * Inline function optimization hint
 * Transpiled to safe Rust
 */

// Rust has #[inline] attribute for optimization hints
// The compiler usually makes good decisions automatically

#[inline]
fn max_inline(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

#[inline]
fn min_inline(a: i32, b: i32) -> i32 {
    if a < b { a } else { b }
}

#[inline]
fn square_inline(x: i32) -> i32 {
    x * x
}

// Non-inline for comparison
fn square_normal(x: i32) -> i32 {
    x * x
}

// #[inline] with more complex logic
#[inline]
fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[inline]
fn abs_inline(x: i32) -> i32 {
    if x < 0 { -x } else { x }
}

// Force always inline (like GCC's always_inline)
#[inline(always)]
fn force_inline(x: i32) -> i32 {
    x * 2
}

// Hint to never inline
#[inline(never)]
fn never_inline(x: i32) -> i32 {
    x * 3
}

fn main() {
    println!("=== Inline Functions ===\n");

    let a = 10;
    let b = 20;

    println!("max({}, {}) = {}", a, b, max_inline(a, b));
    println!("min({}, {}) = {}", a, b, min_inline(a, b));

    println!("\nSquares:");
    for i in 1..=5 {
        println!("  square({}) = {}", i, square_inline(i));
    }

    println!("\nClamp examples:");
    println!("  clamp(5.0, 0.0, 10.0) = {:.1}", clamp(5.0, 0.0, 10.0));
    println!("  clamp(-5.0, 0.0, 10.0) = {:.1}", clamp(-5.0, 0.0, 10.0));
    println!("  clamp(15.0, 0.0, 10.0) = {:.1}", clamp(15.0, 0.0, 10.0));

    println!("\nAbsolute values:");
    println!("  abs({}) = {}", -42, abs_inline(-42));
    println!("  abs({}) = {}", 42, abs_inline(42));

    // Inline vs normal
    println!("\nInline vs normal (both should give same result):");
    println!("  inline: square(7) = {}", square_inline(7));
    println!("  normal: square(7) = {}", square_normal(7));

    // Force inline/never inline
    println!("\nForce inline/never inline:");
    println!("  force_inline(5) = {}", force_inline(5));
    println!("  never_inline(7) = {}", never_inline(7));
}

// Key differences from C:
// 1. #[inline] instead of "inline" keyword
// 2. #[inline(always)] for forced inlining
// 3. #[inline(never)] to prevent inlining
// 4. Compiler has excellent heuristics - manual hints rarely needed
// 5. Cross-crate inlining works better with LTO
