/* K&R C Chapter 2.9: Bitwise Operators
 * Page 48-49
 * Bitwise AND, OR, XOR, complement, shift operations
 * Transpiled to safe Rust
 */

fn main() {
    let a: u32 = 0x0F;  // 00001111
    let b: u32 = 0x55;  // 01010101

    println!("a = 0x{:02X} ({})", a, a);
    println!("b = 0x{:02X} ({})", b, b);
    println!();

    // Bitwise AND
    println!("a & b  = 0x{:02X} ({})", a & b, a & b);

    // Bitwise OR
    println!("a | b  = 0x{:02X} ({})", a | b, a | b);

    // Bitwise XOR
    println!("a ^ b  = 0x{:02X} ({})", a ^ b, a ^ b);

    // Complement
    println!("~a     = 0x{:02X}", !a & 0xFF);

    // Left shift
    println!("a << 2 = 0x{:02X} ({})", a << 2, a << 2);

    // Right shift
    println!("b >> 2 = 0x{:02X} ({})", b >> 2, b >> 2);
}
