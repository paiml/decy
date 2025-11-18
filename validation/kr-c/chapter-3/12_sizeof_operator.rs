/* K&R C Chapter 3: sizeof Operator
 * K&R §3.3: Determining type and object sizes
 * Transpiled to safe Rust (using std::mem)
 */

use std::mem;

struct Example {
    c: u8,
    i: i32,
    d: f64,
}

fn basic_types() {
    println!("=== Basic Type Sizes ===");
    println!("u8:        {} bytes", mem::size_of::<u8>());
    println!("i16:       {} bytes", mem::size_of::<i16>());
    println!("i32:       {} bytes", mem::size_of::<i32>());
    println!("i64:       {} bytes", mem::size_of::<i64>());
    println!("i128:      {} bytes", mem::size_of::<i128>());
    println!("f32:       {} bytes", mem::size_of::<f32>());
    println!("f64:       {} bytes", mem::size_of::<f64>());
    println!("pointer:   {} bytes", mem::size_of::<*const ()>());
    println!();
}

fn array_sizes() {
    println!("=== Array Sizes ===");
    let arr: [i32; 10] = [0; 10];
    println!("i32 arr[10]: {} bytes", mem::size_of_val(&arr));
    println!("Element count: {}", arr.len());

    let str_bytes = b"Hello";
    println!("&[u8] \"Hello\": {} bytes (includes \\0)", str_bytes.len() + 1);
    println!();
}

fn struct_sizes() {
    println!("=== Structure Sizes ===");
    println!("struct Example: {} bytes", mem::size_of::<Example>());
    println!("  c (u8):   {} bytes", mem::size_of::<u8>());
    println!("  i (i32):  {} bytes", mem::size_of::<i32>());
    println!("  d (f64):  {} bytes", mem::size_of::<f64>());
    println!();
}

fn main() {
    println!("=== sizeof Operator ===\n");

    basic_types();
    array_sizes();
    struct_sizes();

    println!("mem::size_of is:");
    println!("  - Compile-time function");
    println!("  - Returns usize");
    println!("  - Works on types: size_of::<T>()");
    println!("  - Works on values: size_of_val(&var)");
    println!("  - Accounts for padding in structs");
}
