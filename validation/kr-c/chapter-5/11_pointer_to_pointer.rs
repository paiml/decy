/* K&R C Chapter 5: Pointers to Pointers
 * Advanced pointer concepts
 * Transpiled to safe Rust (using slices)
 */

// Sort strings using safe slice operations
fn sort_strings(strings: &mut [&str]) {
    strings.sort();  // Rust's built-in sort is safe and efficient
}

fn main() {
    let mut fruits = vec![
        "banana",
        "apple",
        "cherry",
        "date",
        "elderberry",
    ];

    println!("Before sorting:");
    for fruit in &fruits {
        println!("  {}", fruit);
    }

    sort_strings(&mut fruits);

    println!("\nAfter sorting:");
    for fruit in &fruits {
        println!("  {}", fruit);
    }

    // Slice of string slices (equivalent to char**)
    let ptr: &[&str] = &fruits;
    println!("\nUsing slice:");
    for (i, &s) in ptr.iter().enumerate() {
        println!("  ptr[{}] = {}", i, s);
    }
}

// Manual bubble sort (for educational comparison)
fn bubble_sort_strings(strings: &mut [&str]) {
    let n = strings.len();
    for i in 0..n.saturating_sub(1) {
        for j in (i + 1)..n {
            if strings[i] > strings[j] {
                strings.swap(i, j);
            }
        }
    }
}

// Key differences from C:
// 1. &mut [&str] instead of char**
// 2. No manual pointer arithmetic
// 3. .swap() instead of manual auxiliary variable
// 4. Built-in .sort() uses optimized algorithm
// 5. No null pointers possible
// 6. Bounds checking (can be optimized away)
