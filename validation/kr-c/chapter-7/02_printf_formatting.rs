/* K&R C Chapter 7.2: Formatted Output - printf
 * Page 153-154
 * Various printf format specifiers
 * Transpiled to safe Rust using println! macro
 */

fn main() {
    let i: i32 = 42;
    let f: f32 = 3.14159;
    let d: f64 = 2.71828;
    let c: char = 'A';
    let s: &str = "Hello, World!";

    // Integer formats
    println!("Integer: {}", i);
    println!("Hex: {:x}, {:X}", i, i);
    println!("Octal: {:o}", i);

    // Float formats
    println!("Float: {}", f);
    println!("Float (precision): {:.2}", f);
    println!("Scientific: {:e}", f);

    // Character and string
    println!("Character: {}", c);
    println!("String: {}", s);

    // Width and alignment
    println!("Right-aligned: {:>10}", i);
    println!("Left-aligned: {:<10}|", i);
    println!("Zero-padded: {:05}", i);

    // Mixed formats
    println!("{}: {} = 0x{:X} = {:o} (octal)", "Number", i, i, i);
}
