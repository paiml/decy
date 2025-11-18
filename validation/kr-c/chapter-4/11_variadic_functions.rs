/* K&R C Chapter 4: Variadic Functions
 * Functions with variable number of arguments
 * Transpiled to safe Rust (using slices and generics)
 */

// Rust doesn't have variadic functions in the same way as C
// Instead, use slices, vectors, or macros

fn sum(numbers: &[i32]) -> i32 {
    numbers.iter().sum()
}

fn max(numbers: &[i32]) -> i32 {
    *numbers.iter().max().unwrap_or(&0)
}

// For printf-like functionality, use macros (print!, println!, format!)
// Rust macros are type-safe and checked at compile time

fn average(numbers: &[f64]) -> f64 {
    if numbers.is_empty() {
        return 0.0;
    }

    let total: f64 = numbers.iter().sum();
    total / numbers.len() as f64
}

fn main() {
    println!("=== Variadic Functions ===\n");

    // Sum function - use slices instead of va_args
    println!("sum(&[10, 20, 30]) = {}", sum(&[10, 20, 30]));
    println!("sum(&[1, 2, 3, 4, 5]) = {}", sum(&[1, 2, 3, 4, 5]));
    println!("sum(&[100]) = {}", sum(&[100]));

    // Max function
    println!("\nmax(&[25, 10, 50, 30]) = {}", max(&[25, 10, 50, 30]));
    println!("max(&[3, 7, 2, 9, 1, 5]) = {}", max(&[3, 7, 2, 9, 1, 5]));

    // Rust's print! and println! are macros that handle variable args
    println!();
    println!("Hello, {}! Number: {}", "World", 42);
    println!("Float: {:.2}, String: {}", 3.14159, "test");

    // Average
    println!("\naverage(&[10.0, 20.0, 30.0]) = {:.2}",
             average(&[10.0, 20.0, 30.0]));
    println!("average(&[1.5, 2.5, 3.5, 4.5, 5.5]) = {:.2}",
             average(&[1.5, 2.5, 3.5, 4.5, 5.5]));
}

// Demonstrate variadic-style using vec! macro
fn demonstrate_vec_macro() {
    // vec! macro acts like variadic constructor
    let v = vec![1, 2, 3, 4, 5];
    println!("Vec sum: {}", v.iter().sum::<i32>());
}

// Custom "variadic" using generic args and Into<Vec>
fn sum_generic<T: IntoIterator<Item = i32>>(items: T) -> i32 {
    items.into_iter().sum()
}

// Key differences from C:
// 1. Type-safe slices instead of va_list
// 2. Compile-time checked format strings (println! macro)
// 3. No runtime parsing of format specifiers
// 4. Cannot pass wrong type - compiler error
// 5. Bounds checking on slice access
