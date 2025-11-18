/* K&R C Chapter 2.2: Data Types - Signed and Unsigned
 * Page 37-38
 * Signed and unsigned integer types
 * Transpiled to safe Rust
 */

fn main() {
    // Signed integers
    let sc: i8 = -128;
    let ss: i16 = -32768;
    let si: i32 = -2147483648;

    println!("Signed types:");
    println!("  i8:  {}", sc);
    println!("  i16: {}", ss);
    println!("  i32: {}", si);

    // Unsigned integers
    let uc: u8 = 255;
    let us: u16 = 65535;
    let ui: u32 = 4294967295;

    println!("\nUnsigned types:");
    println!("  u8:  {}", uc);
    println!("  u16: {}", us);
    println!("  u32: {}", ui);

    // Limits (using constants)
    println!("\nType limits:");
    println!("  i8::MIN = {}, i8::MAX = {}", i8::MIN, i8::MAX);
    println!("  i16::MIN = {}, i16::MAX = {}", i16::MIN, i16::MAX);
    println!("  i32::MIN = {}, i32::MAX = {}", i32::MIN, i32::MAX);
    println!("  u32::MAX = {}", u32::MAX);

    // Overflow behavior
    println!("\nOverflow (wrapping):");
    let uc_max: u8 = 255;
    println!("  uc_max = {}", uc_max);
    println!("  uc_max.wrapping_add(1) = {} (wraps to 0)", uc_max.wrapping_add(1));

    let uc_min: u8 = 0;
    println!("  uc_min = {}", uc_min);
    println!("  uc_min.wrapping_sub(1) = {} (wraps to 255)", uc_min.wrapping_sub(1));

    // Mixed signed/unsigned (Rust requires explicit conversion)
    println!("\nMixed signed/unsigned:");
    let s: i32 = -1;
    let u: u32 = 1;

    println!("  signed -1 = {}", s);
    println!("  unsigned 1 = {}", u);

    // Rust doesn't allow direct comparison of signed and unsigned
    // Must explicitly convert
    if s < 0 || (s as u32) < u {
        println!("  -1 < 1 (explicit handling)");
    } else {
        println!("  -1 >= 1 (unexpected!)");
    }

    // Explicit comparison with casting
    println!("  Comparing as i32: {} < {} = {}", s, u as i32, s < (u as i32));

    // Hexadecimal notation
    println!("\nHexadecimal:");
    let hex1: u32 = 0xFF;
    let hex2: u32 = 0xDEADBEEF;
    println!("  0xFF = {} ({})", hex1, hex1 as i32);
    println!("  0xDEADBEEF = {}", hex2);
}

// Demonstrate Rust's explicit overflow handling
fn demonstrate_overflow_handling() {
    let a: u8 = 250;

    // Different overflow strategies:
    println!("Overflow handling strategies:");

    // Wrapping (like C's unsigned)
    println!("  wrapping_add: {}", a.wrapping_add(10));  // 4

    // Saturating (clamps to max)
    println!("  saturating_add: {}", a.saturating_add(10));  // 255

    // Checked (returns None on overflow)
    match a.checked_add(10) {
        Some(result) => println!("  checked_add: {}", result),
        None => println!("  checked_add: overflow!"),
    }

    // Overflowing (returns result + bool)
    let (result, overflowed) = a.overflowing_add(10);
    println!("  overflowing_add: {} (overflow: {})", result, overflowed);
}

// Key differences from C:
// 1. Explicit type names: i8, i16, i32, i64, u8, u16, u32, u64
// 2. No implicit signed/unsigned conversion (must use 'as')
// 3. Overflow panics in debug mode, wraps in release (configurable)
// 4. Explicit overflow methods: wrapping_, saturating_, checked_, overflowing_
// 5. Type suffixes: 10u8, -5i32, etc.
// 6. Cannot compare signed and unsigned directly
