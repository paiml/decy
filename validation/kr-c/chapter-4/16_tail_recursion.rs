/* K&R C Chapter 4: Tail Recursion Optimization
 * Comparing normal and tail-recursive functions
 * Transpiled to safe Rust
 */

// Normal recursion - not tail-recursive
fn factorial_normal(n: i32) -> i32 {
    if n <= 1 {
        1
    } else {
        n * factorial_normal(n - 1)  // Multiplication after recursive call
    }
}

// Tail recursion - can be optimized by LLVM
fn factorial_tail_helper(n: i32, accumulator: i32) -> i32 {
    if n <= 1 {
        accumulator
    } else {
        factorial_tail_helper(n - 1, n * accumulator)  // Nothing after recursive call
    }
}

fn factorial_tail(n: i32) -> i32 {
    factorial_tail_helper(n, 1)
}

// Normal recursion - Fibonacci
fn fib_normal(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        fib_normal(n - 1) + fib_normal(n - 2)  // Two recursive calls
    }
}

// Tail recursion - Fibonacci with accumulator
fn fib_tail_helper(n: i32, a: i32, b: i32) -> i32 {
    match n {
        0 => a,
        1 => b,
        _ => fib_tail_helper(n - 1, b, a + b),
    }
}

fn fib_tail(n: i32) -> i32 {
    fib_tail_helper(n, 0, 1)
}

// Sum of slice - normal recursion
fn sum_normal(arr: &[i32]) -> i32 {
    if arr.is_empty() {
        0
    } else {
        arr[0] + sum_normal(&arr[1..])
    }
}

// Sum of slice - tail recursion
fn sum_tail_helper(arr: &[i32], accumulator: i32) -> i32 {
    if arr.is_empty() {
        accumulator
    } else {
        sum_tail_helper(&arr[1..], accumulator + arr[0])
    }
}

fn sum_tail(arr: &[i32]) -> i32 {
    sum_tail_helper(arr, 0)
}

fn main() {
    println!("=== Tail Recursion ===\n");

    // Factorial comparison
    println!("Factorial:");
    for i in 0..=10 {
        println!("  {}! = {} (normal) = {} (tail)",
                 i, factorial_normal(i), factorial_tail(i));
    }

    // Fibonacci comparison
    println!("\nFibonacci (first 15):");
    print!("Normal: ");
    for i in 0..15 {
        print!("{} ", fib_normal(i));
    }
    println!();

    print!("Tail:   ");
    for i in 0..15 {
        print!("{} ", fib_tail(i));
    }
    println!("\n");

    // Array sum
    let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    println!("Array sum:");
    println!("  Normal recursion: {}", sum_normal(&arr));
    println!("  Tail recursion: {}", sum_tail(&arr));

    println!("\nTail recursion benefits:");
    println!("  - Can be optimized to iteration by LLVM");
    println!("  - Uses constant stack space");
    println!("  - No risk of stack overflow for deep recursion");
}

// Idiomatic Rust alternative: use iterators
fn demonstrate_iterators() {
    let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // Sum using iterator (most efficient)
    let sum: i32 = arr.iter().sum();
    println!("Iterator sum: {}", sum);

    // Factorial using iterator
    let factorial = (1..=10).product::<i32>();
    println!("Factorial 10!: {}", factorial);
}

// Key differences from C:
// 1. Rust LLVM optimizes tail recursion well
// 2. Slices (&[T]) instead of pointers
// 3. Iterators are more idiomatic than recursion
// 4. Pattern matching instead of if-else chains
// 5. No stack overflow risk with safe recursion limits
