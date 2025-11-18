/* K&R C Chapter 4: Const Variables
 * Demonstrates const qualifier on global variables
 * Transpiled to safe Rust
 */

const MAX_BUFFER: i32 = 1024;
const PI: f64 = 3.14159;

fn main() {
    println!("MAX_BUFFER = {}", MAX_BUFFER);
    println!("PI = {}", PI);
}

// Rust consts are compile-time constants and always immutable
// This is safer than C const because:
// 1. Cannot be cast away (unlike C's const)
// 2. Evaluated at compile time
// 3. No memory address (inlined)

// For runtime constants that need an address, use static:
static RUNTIME_MAX: i32 = 2048;
static EULER: f64 = 2.71828;

#[allow(dead_code)]
fn demonstrate_static() {
    // static has a fixed memory address
    let _ptr_to_static: *const i32 = &RUNTIME_MAX;

    println!("RUNTIME_MAX = {}", RUNTIME_MAX);
    println!("EULER = {}", EULER);
}
