/* K&R C Chapter 4.6: Static Variables
 * Page 83-84
 * Register variables for performance
 * Transpiled to safe Rust
 */

use std::time::Instant;

// Rust doesn't have register keyword - the compiler optimizes automatically
// Modern LLVM backend makes better decisions than manual hints

fn sum_array_normal(arr: &[i32]) -> i64 {
    let mut sum: i64 = 0;

    for &val in arr {
        sum += val as i64;
    }

    sum
}

// In Rust, we trust the optimizer
// No "register" needed - LLVM will optimize loop variables
fn sum_array_optimized(arr: &[i32]) -> i64 {
    // Using iterator (often faster than manual indexing)
    arr.iter().map(|&x| x as i64).sum()
}

fn main() {
    let size = 10_000_000;
    let arr: Vec<i32> = (0..size).map(|i| i % 100).collect();

    println!("Array size: {} elements\n", size);

    // Normal version
    let start = Instant::now();
    let result1 = sum_array_normal(&arr);
    let elapsed1 = start.elapsed();
    println!("Normal:   sum = {}, time = {:.6} seconds",
             result1, elapsed1.as_secs_f64());

    // Optimized (iterator) version
    let start = Instant::now();
    let result2 = sum_array_optimized(&arr);
    let elapsed2 = start.elapsed();
    println!("Iterator: sum = {}, time = {:.6} seconds",
             result2, elapsed2.as_secs_f64());

    assert_eq!(result1, result2);
}

// Key differences from C:
// 1. No manual "register" hint needed
// 2. Safe Vec instead of malloc/free
// 3. RAII - automatic cleanup
// 4. Bounds checking (can be optimized away)
// 5. Modern LLVM optimization is superior to manual hints
