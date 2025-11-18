/* K&R C Chapter 3: Bitwise Operations
 * K&R §3.4: Bit manipulation operators
 * Transpiled to safe Rust
 */

fn print_binary(n: u32) {
    for i in (0..32).rev() {
        print!("{}", (n >> i) & 1);
        if i % 4 == 0 {
            print!(" ");
        }
    }
}

fn demo_bitwise_and() {
    println!("=== Bitwise AND (&) ===");
    let a: u32 = 0b11110000;
    let b: u32 = 0b10101010;
    let result = a & b;

    print!("a:      ");
    print_binary(a);
    print!("\nb:      ");
    print_binary(b);
    print!("\na & b:  ");
    print_binary(result);
    println!("\n");
}

fn demo_bitwise_or() {
    println!("=== Bitwise OR (|) ===");
    let a: u32 = 0b11110000;
    let b: u32 = 0b10101010;
    let result = a | b;

    print!("a:      ");
    print_binary(a);
    print!("\nb:      ");
    print_binary(b);
    print!("\na | b:  ");
    print_binary(result);
    println!("\n");
}

fn demo_bitwise_xor() {
    println!("=== Bitwise XOR (^) ===");
    let a: u32 = 0b11110000;
    let b: u32 = 0b10101010;
    let result = a ^ b;

    print!("a:      ");
    print_binary(a);
    print!("\nb:      ");
    print_binary(b);
    print!("\na ^ b:  ");
    print_binary(result);
    println!("\n");
}

fn demo_bitwise_not() {
    println!("=== Bitwise NOT (~) ===");
    let a: u8 = 0b11110000;
    let result = !a;  // ! in Rust, not ~

    println!("a:    {:02X} (binary: {:08b})", a, a);
    println!("!a:   {:02X} (binary: {:08b})", result, result);
    println!();
}

fn demo_left_shift() {
    println!("=== Left Shift (<<) ===");
    let a: u32 = 0b00000101;

    println!("a:       {}", a);
    println!("a << 1:  {} (multiply by 2)", a << 1);
    println!("a << 2:  {} (multiply by 4)", a << 2);
    println!("a << 3:  {} (multiply by 8)", a << 3);
    println!();
}

fn demo_right_shift() {
    println!("=== Right Shift (>>) ===");
    let a: u32 = 40;

    println!("a:       {}", a);
    println!("a >> 1:  {} (divide by 2)", a >> 1);
    println!("a >> 2:  {} (divide by 4)", a >> 2);
    println!("a >> 3:  {} (divide by 8)", a >> 3);
    println!();
}

fn demo_bit_masking() {
    println!("=== Bit Masking Examples ===");

    let mut flags: u32 = 0;

    // Set bit
    flags |= 1 << 2;  // Set bit 2
    println!("Set bit 2:     0x{:02X}", flags);

    // Clear bit
    flags &= !(1 << 1);  // Clear bit 1
    println!("Clear bit 1:   0x{:02X}", flags);

    // Toggle bit
    flags ^= 1 << 3;  // Toggle bit 3
    println!("Toggle bit 3:  0x{:02X}", flags);

    // Check bit
    let is_set = (flags & (1 << 2)) != 0;
    println!("Bit 2 is set:  {}", if is_set { "Yes" } else { "No" });

    println!();
}

fn main() {
    println!("=== Bitwise Operations ===\n");

    demo_bitwise_and();
    demo_bitwise_or();
    demo_bitwise_xor();
    demo_bitwise_not();
    demo_left_shift();
    demo_right_shift();
    demo_bit_masking();

    println!("Bitwise operators:");
    println!("  &   AND");
    println!("  |   OR");
    println!("  ^   XOR");
    println!("  !   NOT (Rust uses ! not ~)");
    println!("  <<  Left shift");
    println!("  >>  Right shift");
}
