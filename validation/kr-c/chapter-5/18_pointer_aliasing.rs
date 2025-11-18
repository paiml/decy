/* K&R C Chapter 5: Pointer Aliasing and Restrict
 * Multiple pointers to same data
 * Transpiled to safe Rust (borrow checker prevents issues)
 */

// Rust's borrow checker prevents problematic aliasing
fn modify_value(val: &mut i32) {
    *val = 100;
    println!("After modification: val = {}", val);
}

// Cannot have two mutable references simultaneously
fn safe_array_modification(arr: &mut [i32]) {
    println!("Array before modification:");
    println!("  {:?}", arr);

    arr[0] = 999;

    println!("After arr[0] = 999:");
    println!("  {:?}", arr);
}

fn main() {
    // Rust prevents dangerous aliasing at compile time
    println!("=== Safe Borrowing (No Aliasing Issues) ===");
    let mut value = 42;

    // Can have multiple immutable borrows
    let ptr1 = &value;
    let ptr2 = &value;
    println!("Initial: value = {}, *ptr1 = {}, *ptr2 = {}", value, ptr1, ptr2);

    // But cannot have mutable and immutable borrows simultaneously
    let ptr_mut = &mut value;
    *ptr_mut = 100;
    println!("After mutation: value = {}", value);

    // Array borrowing
    println!("\n=== Array Borrowing ===");
    let mut numbers = vec![1, 2, 3, 4, 5];
    safe_array_modification(&mut numbers);

    // Demonstrate borrow checker preventing issues
    println!("\n=== Borrow Checker Prevents Aliasing ===");
    let mut x = 10;
    modify_value(&mut x);
    println!("Final value: {}", x);

    // Overlapping memory operations (using safe methods)
    println!("\n=== Safe Overlapping Operations ===");
    let mut buffer = vec!['H', 'e', 'l', 'l', 'o'];
    println!("Original: {:?}", buffer);

    // Safe copy within slice
    let (src, dst) = buffer.split_at_mut(2);
    dst[0] = src[0];
    println!("After safe copy: {:?}", buffer);
}

// Demonstrate split_at_mut for non-overlapping mutable access
fn demonstrate_split_at_mut() {
    let mut arr = [1, 2, 3, 4, 5, 6];

    // Split into two non-overlapping mutable slices
    let (left, right) = arr.split_at_mut(3);

    left[0] = 10;
    right[0] = 40;

    println!("After split_at_mut: {:?}", arr);
}

// Cell and RefCell for interior mutability (controlled aliasing)
use std::cell::RefCell;

fn demonstrate_refcell() {
    let data = RefCell::new(vec![1, 2, 3]);

    // Can borrow mutably at runtime
    {
        let mut borrow = data.borrow_mut();
        borrow[0] = 10;
    }  // Mutable borrow dropped here

    // Now can borrow immutably
    let borrow = data.borrow();
    println!("Data: {:?}", *borrow);

    // RefCell provides runtime-checked borrowing
}

// Key differences from C:
// 1. Borrow checker prevents aliasing issues at compile time
// 2. Cannot have &mut and & simultaneously
// 3. split_at_mut for safe non-overlapping access
// 4. RefCell for controlled runtime-checked aliasing
// 5. No undefined behavior from aliasing
// 6. Compiler enforces memory safety
