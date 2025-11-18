/* K&R C Chapter 5: Function Pointers - Callbacks
 * Using function pointers for callbacks and sorting
 * Transpiled to safe Rust (using closures and sort_by)
 */

use std::cmp::Ordering;

// Comparison functions for sorting
fn compare_ints_asc(a: &i32, b: &i32) -> Ordering {
    a.cmp(b)
}

fn compare_ints_desc(a: &i32, b: &i32) -> Ordering {
    b.cmp(a)
}

// Generic apply function using closure
fn apply_to_array<F>(arr: &mut [i32], func: F)
where
    F: Fn(&mut i32),
{
    for item in arr {
        func(item);
    }
}

fn double_value(x: &mut i32) {
    *x *= 2;
}

fn square_value(x: &mut i32) {
    *x *= *x;
}

// Callback-based iteration
fn foreach<F>(arr: &[i32], mut callback: F)
where
    F: FnMut(usize, i32),
{
    for (i, &val) in arr.iter().enumerate() {
        callback(i, val);
    }
}

fn main() {
    // Sorting with different comparison functions
    let mut numbers = vec![5, 2, 8, 1, 9, 3, 7, 4, 6];

    println!("Original: {:?}", numbers);

    numbers.sort_by(compare_ints_asc);
    println!("Ascending: {:?}", numbers);

    numbers.sort_by(compare_ints_desc);
    println!("Descending: {:?}", numbers);

    // String sorting (using built-in)
    let mut words = vec!["zebra", "apple", "mango", "banana", "cherry"];

    println!("\nOriginal words: {:?}", words);
    words.sort();
    println!("Sorted words: {:?}", words);

    // Apply functions to array
    let mut values = vec![1, 2, 3, 4, 5];

    println!("\nOriginal values: {:?}", values);

    apply_to_array(&mut values, double_value);
    println!("After doubling: {:?}", values);

    apply_to_array(&mut values, square_value);
    println!("After squaring: {:?}", values);

    // Callback iteration
    println!("\nUsing callback:");
    foreach(&values, |i, val| {
        println!("  arr[{}] = {}", i, val);
    });

    // Using closures (more idiomatic)
    println!("\nUsing closures:");
    let mut nums = vec![1, 2, 3, 4, 5];

    // Map with closure
    nums.iter_mut().for_each(|x| *x *= 2);
    println!("Doubled: {:?}", nums);

    // Sort with closure
    nums.sort_by(|a, b| b.cmp(a));  // Descending
    println!("Sorted desc: {:?}", nums);

    // Filter and collect
    let evens: Vec<i32> = nums.iter().filter(|&&x| x % 2 == 0).copied().collect();
    println!("Even numbers: {:?}", evens);
}

// Demonstrate function pointer types
type IntComparator = fn(&i32, &i32) -> Ordering;
type IntTransform = fn(&mut i32);

fn demonstrate_function_types() {
    let comparators: [IntComparator; 2] = [compare_ints_asc, compare_ints_desc];
    let transforms: [IntTransform; 2] = [double_value, square_value];

    let mut arr = vec![5, 2, 8];

    // Use different sorters
    for (i, cmp) in comparators.iter().enumerate() {
        let mut copy = arr.clone();
        copy.sort_by(*cmp);
        println!("Sorted {} {:?}", i, copy);
    }

    // Use different transforms
    for (i, transform) in transforms.iter().enumerate() {
        let mut copy = arr.clone();
        apply_to_array(&mut copy, *transform);
        println!("Transformed {}: {:?}", i, copy);
    }
}

// Key differences from C:
// 1. Closures instead of void* callbacks
// 2. Generic functions with trait bounds
// 3. FnOnce, FnMut, Fn traits for closure types
// 4. No void* needed - type-safe generics
// 5. Built-in .sort_by() instead of qsort
// 6. Iterator methods (.filter, .map, etc.)
// 7. Type inference for closures
