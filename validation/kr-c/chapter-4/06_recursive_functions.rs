/* K&R C Chapter 4.10: Recursion
 * Page 86-87
 * Recursive functions - factorial and Fibonacci
 * Transpiled to safe Rust
 */

// factorial: compute n! recursively
fn factorial(n: i32) -> i32 {
    if n <= 1 {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}

// fibonacci: compute nth Fibonacci number recursively
fn fibonacci(n: i32) -> i32 {
    if n <= 1 {
        return n;
    } else {
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
}

// power: compute x^n recursively
fn power(x: f64, n: i32) -> f64 {
    if n == 0 {
        return 1.0;
    } else if n > 0 {
        return x * power(x, n - 1);
    } else {
        return 1.0 / power(x, -n);
    }
}

fn main() {
    println!("Factorials:");
    for i in 0..=10 {
        println!("{}! = {}", i, factorial(i));
    }

    println!("\nFibonacci sequence:");
    for i in 0..15 {
        println!("fib({}) = {}", i, fibonacci(i));
    }

    println!("\nPowers:");
    println!("2^10 = {:.0}", power(2.0, 10));
    println!("3^5 = {:.0}", power(3.0, 5));
    println!("2^-3 = {:.3}", power(2.0, -3));
}
