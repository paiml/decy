/* K&R C Chapter 2.4: Declarations - Type Qualifiers
 * Page 40-41
 * const and volatile qualifiers
 * Transpiled to safe Rust
 */

// Const globals
const PI: f64 = 3.14159265359;
const MAX_SIZE: i32 = 1000;

fn main() {
    // Const variables (immutable by default in Rust)
    let max = 100;
    let message = "Hello, World!";

    println!("PI = {:.10}", PI);
    println!("MAX_SIZE = {}", MAX_SIZE);
    println!("max = {}", max);
    println!("message = {}", message);

    // Cannot modify immutable bindings
    // PI = 3.14;  // ERROR: cannot assign to immutable constant
    // max = 200;  // ERROR: cannot assign to immutable variable

    // Can rebind (shadow) the variable
    let message = "Goodbye!";  // OK: new binding
    println!("message = {}", message);

    // In Rust, strings are immutable by default
    // message.chars_mut() doesn't exist

    // Mutable variable with immutable reference
    let mut value = 42;
    let ptr1 = &value;  // Immutable reference
    // *ptr1 = 100;  // ERROR: cannot mutate through immutable reference

    // Mutable reference
    let ptr2 = &mut value;
    *ptr2 = 100;  // OK: can modify through mutable reference
    println!("value = {}", value);

    // Const array (immutable by default)
    let primes = [2, 3, 5, 7, 11, 13];
    let size = primes.len();

    println!("\nPrime numbers: ");
    for i in 0..size {
        print!("{} ", primes[i]);
    }
    println!();

    // Cannot modify immutable array
    // primes[0] = 1;  // ERROR: cannot assign to immutable indexed content

    // Volatile equivalent in Rust: std::ptr::read_volatile/write_volatile
    use std::sync::atomic::{AtomicI32, Ordering};

    // For hardware registers, use volatile reads/writes or atomics
    let hardware_status = AtomicI32::new(0);
    hardware_status.store(1, Ordering::Relaxed);

    println!("\nhardware_status = {}", hardware_status.load(Ordering::Relaxed));

    // Const slice parameter (immutable by default)
    let nums = [10, 20, 30, 40, 50];
    let sum: i32 = nums.iter().sum();

    println!("Sum of const array: {}", sum);
}

// Demonstrate const fn (compile-time evaluation)
const fn compile_time_multiply(a: i32, b: i32) -> i32 {
    a * b
}

const COMPUTED_AT_COMPILE_TIME: i32 = compile_time_multiply(5, 6);

// Key differences from C:
// 1. Immutable by default (use 'mut' for mutable)
// 2. const is compile-time constant (like C++ constexpr)
// 3. No need for 'const' keyword on local variables (immutable by default)
// 4. References are & (immutable) or &mut (mutable)
// 5. Volatile: use atomics or std::ptr::read_volatile/write_volatile
// 6. Cannot cast away constness (unlike C)
