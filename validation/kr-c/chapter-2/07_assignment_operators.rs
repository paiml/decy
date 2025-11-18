/* K&R C Chapter 2.10: Assignment Operators and Expressions
 * Page 50
 * Compound assignment operators (+=, -=, *=, /=, etc.)
 * Transpiled to safe Rust
 */

fn main() {
    let mut n = 10;

    println!("Initial value: n = {}\n", n);

    // Arithmetic assignment
    n += 5;
    println!("n += 5  -> n = {}", n);

    n -= 3;
    println!("n -= 3  -> n = {}", n);

    n *= 2;
    println!("n *= 2  -> n = {}", n);

    n /= 4;
    println!("n /= 4  -> n = {}", n);

    n %= 5;
    println!("n %= 5  -> n = {}", n);

    // Bitwise assignment
    let mut i = 0x0F;
    println!("\ni = 0x{:02X}", i);

    i &= 0x55;
    println!("i &= 0x55  -> i = 0x{:02X}", i);

    i |= 0xAA;
    println!("i |= 0xAA  -> i = 0x{:02X}", i);

    i ^= 0xFF;
    println!("i ^= 0xFF  -> i = 0x{:02X}", i);

    i <<= 2;
    println!("i <<= 2    -> i = 0x{:02X}", i & 0xFF);

    i >>= 1;
    println!("i >>= 1    -> i = 0x{:02X}", i);
}

// Rust compound assignment operators work identically to C
// Key differences:
// 1. Must explicitly use 'mut' to allow mutation
// 2. Variables cannot be used before initialization
// 3. Overflow checked in debug mode, wrapping in release
// 4. Can use .wrapping_add(), .saturating_add(), .checked_add() for explicit behavior
