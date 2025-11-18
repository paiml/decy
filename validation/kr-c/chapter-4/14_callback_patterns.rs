/* K&R C Chapter 4: Callback Patterns
 * Using function pointers for callbacks
 * Transpiled to safe Rust (using closures and function pointers)
 */

// Rust uses closures and function pointers for callbacks

fn array_foreach<F>(arr: &[i32], mut callback: F)
where
    F: FnMut(usize, i32),
{
    for (index, &value) in arr.iter().enumerate() {
        callback(index, value);
    }
}

fn print_element(index: usize, value: i32) {
    println!("  arr[{}] = {}", index, value);
}

// Filter with callback
fn filter_array<F>(arr: &[i32], filter: F) -> Vec<i32>
where
    F: Fn(i32) -> bool,
{
    arr.iter().copied().filter(|&x| filter(x)).collect()
}

fn is_even(x: i32) -> bool {
    x % 2 == 0
}

fn is_positive(x: i32) -> bool {
    x > 0
}

fn is_greater_than_10(x: i32) -> bool {
    x > 10
}

// Transform callback (map)
fn array_map<F>(arr: &mut [i32], transform: F)
where
    F: Fn(i32) -> i32,
{
    for val in arr.iter_mut() {
        *val = transform(*val);
    }
}

fn double_value(x: i32) -> i32 {
    x * 2
}

fn square_value(x: i32) -> i32 {
    x * x
}

// Sort with comparison callback
fn bubble_sort<F>(arr: &mut [i32], compare: F)
where
    F: Fn(i32, i32) -> std::cmp::Ordering,
{
    let n = arr.len();
    for i in 0..n.saturating_sub(1) {
        for j in 0..n - i - 1 {
            if compare(arr[j], arr[j + 1]) == std::cmp::Ordering::Greater {
                arr.swap(j, j + 1);
            }
        }
    }
}

fn compare_asc(a: i32, b: i32) -> std::cmp::Ordering {
    a.cmp(&b)
}

fn compare_desc(a: i32, b: i32) -> std::cmp::Ordering {
    b.cmp(&a)
}

fn main() {
    println!("=== Callback Patterns ===\n");

    let numbers = vec![5, -2, 8, -3, 12, 1, 15, -7];

    // Foreach with callback
    println!("Array elements:");
    array_foreach(&numbers, print_element);

    // Using closure to accumulate
    let mut sum = 0;
    array_foreach(&numbers, |_index, value| {
        sum += value;
    });
    println!("Sum: {}\n", sum);

    // Filter with callback
    let evens = filter_array(&numbers, is_even);
    println!("Even numbers ({}): {:?}", evens.len(), evens);

    let positives = filter_array(&numbers, is_positive);
    println!("Positive numbers ({}): {:?}", positives.len(), positives);

    let greater = filter_array(&numbers, is_greater_than_10);
    println!("Greater than 10 ({}): {:?}\n", greater.len(), greater);

    // Map/transform
    let mut values = vec![1, 2, 3, 4, 5];

    println!("Original: {:?}", values);

    array_map(&mut values, double_value);
    println!("Doubled: {:?}", values);

    array_map(&mut values, square_value);
    println!("Squared: {:?}\n", values);

    // Sort with callback
    let mut nums = vec![5, 2, 8, 1, 9];

    println!("Original: {:?}", nums);

    bubble_sort(&mut nums, compare_asc);
    println!("Sorted ascending: {:?}", nums);

    bubble_sort(&mut nums, compare_desc);
    println!("Sorted descending: {:?}", nums);
}

// Demonstrate Rust's more idiomatic iterator approach
#[allow(dead_code)]
fn idiomatic_rust_approach() {
    let numbers = vec![5, -2, 8, -3, 12, 1, 15, -7];

    // Filter using iterator
    let evens: Vec<i32> = numbers.iter().filter(|&&x| x % 2 == 0).copied().collect();

    // Map using iterator
    let doubled: Vec<i32> = numbers.iter().map(|&x| x * 2).collect();

    // Sum using iterator
    let sum: i32 = numbers.iter().sum();

    println!("Evens: {:?}", evens);
    println!("Doubled: {:?}", doubled);
    println!("Sum: {}", sum);
}
