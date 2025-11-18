/* K&R C Chapter 2.2: Data Types and Sizes
 * Page 36
 * sizeof operator for type sizes
 * Transpiled to safe Rust
 */

use std::mem;

fn main() {
    println!("Size of fundamental types:");
    println!("sizeof(u8)        = {} bytes", mem::size_of::<u8>());
    println!("sizeof(i16)       = {} bytes", mem::size_of::<i16>());
    println!("sizeof(i32)       = {} bytes", mem::size_of::<i32>());
    println!("sizeof(i64)       = {} bytes", mem::size_of::<i64>());
    println!("sizeof(f32)       = {} bytes", mem::size_of::<f32>());
    println!("sizeof(f64)       = {} bytes", mem::size_of::<f64>());
    println!();

    println!("Size of pointer types:");
    println!("sizeof(&u8)       = {} bytes", mem::size_of::<&u8>());
    println!("sizeof(&i32)      = {} bytes", mem::size_of::<&i32>());
    println!("sizeof(*const ()) = {} bytes", mem::size_of::<*const ()>());
    println!();

    // sizeof on variables
    let arr: [i32; 10] = [0; 10];
    let str_buf: [u8; 100] = [0; 100];

    println!("Size of arrays:");
    println!("sizeof(arr[10])   = {} bytes", mem::size_of_val(&arr));
    println!("sizeof(str[100])  = {} bytes", mem::size_of_val(&str_buf));
    println!();

    // sizeof in expressions
    let n = mem::size_of::<i32>() * 8;
    println!("Bits in i32: {}", n);

    // Array length - Rust provides .len() method
    let len = arr.len();
    println!("Length of arr: {}", len);

    // Demonstrate additional Rust memory functions
    println!("\nAdditional Rust memory info:");
    println!("align_of::<i32>() = {}", mem::align_of::<i32>());
    println!("align_of::<i64>() = {}", mem::align_of::<i64>());
}

// Key differences from C:
// 1. std::mem::size_of::<T>() instead of sizeof(T)
// 2. std::mem::size_of_val(&var) instead of sizeof(var)
// 3. array.len() for array length (always available)
// 4. Additional functions: align_of, size_of_val, discriminant_size_hint
// 5. All sizes known at compile time for sized types
