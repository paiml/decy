/* K&R C Chapter 5: Pointer to Const vs Const Pointer
 * All combinations of const with pointers
 * Transpiled to safe Rust (using & and &mut)
 */

fn demo_immutable_reference() {
    let mut x = 10;
    let mut y = 20;

    // Immutable reference (like const T*)
    let ptr = &x;

    println!("Immutable reference:");
    println!("  *ptr = {}", ptr);

    // Cannot modify through immutable reference
    // *ptr = 15;  // ERROR: cannot mutate

    // Can rebind to different variable
    let ptr = &y;
    println!("  After rebinding: *ptr = {}", ptr);
}

fn demo_mutable_reference() {
    let mut x = 10;
    let mut y = 20;

    // Mutable reference (like T*)
    let ptr = &mut x;

    println!("\nMutable reference:");
    println!("  *ptr = {}", ptr);

    *ptr = 15;  // OK: can modify value
    println!("  After *ptr = 15: *ptr = {}", ptr);

    // Can rebind (Rust allows rebinding)
    let ptr = &mut y;
    println!("  After rebinding: *ptr = {}", ptr);
}

// Function parameters with immutable slice
fn print_array(arr: &[i32]) {
    print!("Array: ");
    for &val in arr {
        print!("{} ", val);
    }
    println!();

    // arr[0] = 100;  // ERROR: cannot mutate through immutable reference
}

// Function parameters with mutable slice
fn modify_array(arr: &mut [i32]) {
    for val in arr {
        *val *= 2;
    }

    // arr = &mut [1, 2, 3];  // Can rebind in Rust
}

fn main() {
    demo_immutable_reference();
    demo_mutable_reference();

    // Function parameter examples
    let mut numbers = vec![1, 2, 3, 4, 5];

    println!("\nFunction with immutable slice:");
    print_array(&numbers);

    println!("\nFunction with mutable slice:");
    println!("Before: ");
    print_array(&numbers);
    modify_array(&mut numbers);
    println!("After:  ");
    print_array(&numbers);

    // Immutable array (like const int arr[])
    let const_arr = [10, 20, 30];
    println!("\nImmutable array:");
    print_array(&const_arr);
    // const_arr[0] = 100;  // ERROR: cannot mutate immutable binding
}

// Key differences from C:
// 1. &T = immutable reference (const T*)
// 2. &mut T = mutable reference (T*)
// 3. No "const pointer" concept - can always rebind
// 4. Borrow checker enforces at compile time
// 5. Cannot have &T and &mut T simultaneously
// 6. Immutable by default - more const-correct than C
