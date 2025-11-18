/* K&R C Chapter 5: Dynamic Arrays with Pointers
 * Memory allocation and pointer manipulation
 * Transpiled to safe Rust (using Vec)
 */

// Create and initialize dynamic array
fn create_array(size: usize) -> Vec<i32> {
    (0..size).map(|i| i as i32 * 10).collect()
}

// Resize array (Vec handles this automatically)
fn resize_array(mut arr: Vec<i32>, new_size: usize) -> Vec<i32> {
    arr.resize(new_size, 0);  // Fill new elements with 0
    arr
}

fn main() {
    // Create dynamic array
    let mut arr = create_array(5);

    println!("Initial array (size {}):", arr.len());
    for (i, &val) in arr.iter().enumerate() {
        println!("  arr[{}] = {}", i, val);
    }

    // Resize array
    arr = resize_array(arr, 10);

    println!("\nAfter resize (size {}):", arr.len());
    for (i, &val) in arr.iter().enumerate() {
        println!("  arr[{}] = {}", i, val);
    }

    // Iterator-based access (safe, no pointer arithmetic)
    println!("\nUsing iterator:");
    for (i, &val) in arr.iter().enumerate() {
        println!("  arr[{}] = {}", i, val);
    }

    // Vec automatically freed when it goes out of scope (RAII)
    println!("\nMemory automatically freed at end of scope");
}

// Alternative: using with_capacity for efficiency
fn demonstrate_capacity() {
    let mut v = Vec::with_capacity(10);
    println!("Capacity: {}, Length: {}", v.capacity(), v.len());

    for i in 0..5 {
        v.push(i * 10);
    }

    println!("After push: Capacity: {}, Length: {}", v.capacity(), v.len());
}

// Key differences from C:
// 1. Vec<T> instead of T* with malloc/free
// 2. Automatic memory management (Drop trait)
// 3. .resize() instead of manual realloc
// 4. No possibility of memory leaks
// 5. No possibility of use-after-free
// 6. Bounds checking on access
// 7. Iterators instead of pointer arithmetic
