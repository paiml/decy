/* K&R C Chapter 2.6: Relational and Logical Operators
 * Page 42-43
 * Short-circuit evaluation
 * Transpiled to safe Rust
 */

use std::sync::atomic::{AtomicI32, Ordering};

static COUNT: AtomicI32 = AtomicI32::new(0);

// Function with side effect
fn increment_and_return(x: i32) -> bool {
    let count = COUNT.fetch_add(1, Ordering::Relaxed) + 1;
    println!("  increment_and_return({}) called, count={}", x, count);
    x != 0
}

fn main() {
    println!("=== Logical AND short-circuit ===");
    COUNT.store(0, Ordering::Relaxed);

    // First operand is false, second not evaluated
    if increment_and_return(0) && increment_and_return(1) {
        println!("Both true");
    } else {
        println!("Result: false (second function NOT called)\n");
    }

    COUNT.store(0, Ordering::Relaxed);

    // First operand is true, second is evaluated
    if increment_and_return(1) && increment_and_return(1) {
        println!("Result: both true (second function WAS called)\n");
    }

    println!("=== Logical OR short-circuit ===");
    COUNT.store(0, Ordering::Relaxed);

    // First operand is true, second not evaluated
    if increment_and_return(1) || increment_and_return(1) {
        println!("Result: true (second function NOT called)\n");
    }

    COUNT.store(0, Ordering::Relaxed);

    // First operand is false, second is evaluated
    if increment_and_return(0) || increment_and_return(1) {
        println!("Result: true (second function WAS called)\n");
    }

    println!("=== Practical example: safe array access ===");
    let arr = [10, 20, 30, 40, 50];
    let size = arr.len();
    let mut index = 3;

    // Safe: checks bounds before accessing
    if index < size && arr[index] > 25 {
        println!("arr[{}] = {} is > 25", index, arr[index]);
    }

    // Short-circuit prevents out-of-bounds access
    index = 10;
    if index < size && arr[index] > 25 {
        println!("arr[{}] > 25", index);
    } else {
        println!("Index {} out of bounds (safe due to short-circuit)", index);
    }

    // Rust also has bounds checking, so even without short-circuit:
    // arr[index] would panic with a helpful message

    println!("\n=== Ternary operator (if-else expression) ===");
    COUNT.store(0, Ordering::Relaxed);

    let result = if false {
        increment_and_return(1) as i32
    } else {
        increment_and_return(2) as i32
    };
    println!("Result: {} (only second branch evaluated)", result);

    COUNT.store(0, Ordering::Relaxed);

    let result = if true {
        increment_and_return(1) as i32
    } else {
        increment_and_return(2) as i32
    };
    println!("Result: {} (only first branch evaluated)", result);
}

// Demonstrate Rust's Option-based safe access
fn safe_array_access_demo() {
    let arr = vec![10, 20, 30, 40, 50];

    // Using .get() returns Option<&T>
    if let Some(&value) = arr.get(3) {
        if value > 25 {
            println!("Value {} is > 25", value);
        }
    }

    // Out of bounds returns None, not panic
    match arr.get(10) {
        Some(value) => println!("Value: {}", value),
        None => println!("Index out of bounds (safe!)"),
    }
}

// Key differences from C:
// 1. && and || work identically (short-circuit)
// 2. Conditions must be bool (no implicit int->bool)
// 3. if-else is an expression (returns value)
// 4. Array access panics on out-of-bounds (runtime check)
// 5. Use .get() for Option-based safe access
// 6. static mut requires unsafe, use atomics instead
