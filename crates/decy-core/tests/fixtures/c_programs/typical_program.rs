/* Typical C program for testing transpilation
 * Transpiled to safe Rust
 *
 * This program demonstrates common C constructs that Decy handles:
 * - Function definitions
 * - Variable declarations
 * - Arithmetic operations
 * - Control flow (if/else)
 * - Loops (for)
 * - I/O (println!)
 *
 * Expected output:
 *   Sum: 55
 *   Factorial: 120
 */

fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn factorial(n: i32) -> i32 {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}

fn main() {
    // Calculate sum of 0..10
    let mut sum: i32 = 0;
    let mut i: i32 = 0;
    while i <= 10 {
        sum = add(sum, i);
        i += 1;
    }

    println!("Sum: {}", sum);

    // Calculate factorial of 5
    let fact = factorial(5);
    println!("Factorial: {}", fact);
}
