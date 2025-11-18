/* K&R C Chapter 3: Signed vs Unsigned Types
 * K&R §3.10: Integer type modifiers
 * Transpiled to safe Rust
 */

fn demo_range_differences() {
    println!("=== Range Differences ===");

    println!("i8:   {} to {}", i8::MIN, i8::MAX);
    println!("u8:   0 to {}", u8::MAX);
    println!();

    println!("i32:  {} to {}", i32::MIN, i32::MAX);
    println!("u32:  0 to {}", u32::MAX);
    println!();
}

fn demo_wraparound() {
    println!("=== Wraparound Behavior ===");

    let uc: u8 = 255;
    println!("u8 = 255");
    println!("After wrapping_add(1): {} (wraps to 0)", uc.wrapping_add(1));
    println!("After wrapping_add(2): {}", uc.wrapping_add(2));
    println!();

    let sc: i8 = 127;
    println!("i8 = 127");
    println!("After wrapping_add(1): {} (wraps in release mode)", sc.wrapping_add(1));
    println!();
}

fn demo_comparison_safety() {
    println!("=== Comparison (Rust prevents pitfalls) ===");

    let si: i32 = -1;
    let ui: u32 = 1;

    // Rust prevents direct comparison
    // This would be a compile error:
    // if si < ui { }  // ERROR: cannot compare i32 and u32

    println!("Cannot compare i32 and u32 directly (compile error)");

    // Must explicitly convert
    if si < 0 || (si as u32) < ui {
        println!("-1 < 1: TRUE (explicit handling required)");
    }

    println!();
}

fn demo_division_differences() {
    println!("=== Division Differences ===");

    let si: i32 = -7;
    let div_s = si / 3;
    println!("i32:   -7 / 3 = {}", div_s);

    let ui = si as u32;
    let div_u = ui / 3;
    println!("u32:   (u32)-7 / 3 = {}", div_u);
    println!();
}

fn demo_right_shift() {
    println!("=== Right Shift Differences ===");

    let si: i32 = -8;
    println!("i32 -8 >> 1 = {} (arithmetic shift)", si >> 1);

    let ui = si as u32;
    println!("u32 >> 1 = {} (logical shift)", ui >> 1);
    println!();
}

fn demo_overflow_detection() {
    println!("=== Overflow Detection ===");

    let a = u32::MAX;
    let b = 1u32;
    let sum = a.wrapping_add(b);
    println!("u32::MAX.wrapping_add(1) = {} (wraps to 0)", sum);

    if sum < a {
        println!("Overflow detected (sum < a)");
    }
    println!();

    // Rust provides checked arithmetic
    match a.checked_add(b) {
        Some(result) => println!("Result: {}", result),
        None => println!("Overflow detected with checked_add!"),
    }
    println!();
}

fn demo_loop_safety() {
    println!("=== Loop Safety ===");

    println!("WRONG in C: for (unsigned i = 5; i >= 0; i--)");
    println!("Problem: i >= 0 always true for unsigned!");
    println!();

    println!("Rust solution - iterator:");
    for i in (0..=5).rev() {
        print!("{} ", i);
    }
    println!("\n");
}

fn main() {
    println!("=== Signed vs Unsigned Types ===\n");

    demo_range_differences();
    demo_wraparound();
    demo_comparison_safety();
    demo_division_differences();
    demo_right_shift();
    demo_overflow_detection();
    demo_loop_safety();

    println!("Key differences from C:");
    println!("  - Cannot compare signed/unsigned directly");
    println!("  - Overflow panics in debug, wraps in release");
    println!("  - Use wrapping_/checked_/saturating_ methods");
    println!("  - Right shift behavior is well-defined");
    println!("  - Iterators prevent unsigned loop pitfalls");
}
