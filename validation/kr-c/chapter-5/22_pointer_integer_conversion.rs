/* K&R C Chapter 5: Pointer and Integer Conversions
 * Converting between pointers and integers
 * Transpiled to safe Rust (using raw pointers when needed)
 */

fn main() {
    let x = 42;
    let ptr = &x as *const i32;

    // Pointer to integer conversion
    println!("Pointer value: {:p}", ptr);

    // Using usize (guaranteed to hold pointer)
    let addr = ptr as usize;
    println!("Address as integer: {} (0x{:X})", addr, addr);

    // Integer to pointer conversion (unsafe)
    let ptr2 = addr as *const i32;
    println!("Converted back: {:p}", ptr2);
    unsafe {
        println!("Value through converted pointer: {}", *ptr2);
    }

    // Slice operations instead of pointer arithmetic
    let arr = [10, 20, 30, 40, 50];
    let p1 = &arr[0] as *const i32;
    let p2 = &arr[4] as *const i32;

    println!("\nPointer arithmetic:");
    println!("p1 = {:p}, p2 = {:p}", p1, p2);

    unsafe {
        let diff = p2.offset_from(p1);
        println!("p2.offset_from(p1) = {} elements", diff);

        let byte_diff = (p2 as usize) - (p1 as usize);
        println!("Byte difference: {} bytes", byte_diff);
    }

    // Pointer alignment
    println!("\nPointer alignment:");
    println!("Address of arr[0]: 0x{:X}", &arr[0] as *const i32 as usize);
    println!("Address of arr[1]: 0x{:X}", &arr[1] as *const i32 as usize);
    println!("Difference: {} bytes", std::mem::size_of::<i32>());

    // NULL equivalent: Option<&T>
    let null_ref: Option<&i32> = None;
    println!("\nOption::None (Rust's NULL):");
    match null_ref {
        Some(r) => println!("Has value: {}", r),
        None => println!("Is None (like NULL)"),
    }

    // Offset calculations with iterators (safe)
    println!("\nSafe offset calculations:");
    for (i, &val) in arr.iter().enumerate() {
        let current_ptr = &arr[i] as *const i32;
        println!("arr[{}]: address = {:p}, value = {}", i, current_ptr, val);
    }
}

// Demonstrate safe alternatives
fn demonstrate_safe_alternatives() {
    let arr = vec![10, 20, 30, 40, 50];

    // Instead of pointer arithmetic, use slices
    let slice = &arr[1..4];
    println!("Slice [1..4]: {:?}", slice);

    // Instead of pointer comparison, use indices
    for (i, &val) in arr.iter().enumerate() {
        println!("[{}] = {}", i, val);
    }
}

// Key differences from C:
// 1. Option<&T> instead of NULL pointers
// 2. Raw pointers require unsafe
// 3. References cannot be null
// 4. Pointer arithmetic requires unsafe
// 5. Prefer slices and iterators
// 6. usize for pointer-sized integers
