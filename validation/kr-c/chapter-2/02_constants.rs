/* K&R C Chapter 2.3: Constants
 * Page 36-37
 * Various constant examples
 * Transpiled to safe Rust
 */

fn main() {
    let decimal: i32 = 100;
    let octal: i32 = 0o144;      // 100 in octal
    let hex: i32 = 0x64;         // 100 in hexadecimal
    let long_const: i64 = 123456789_i64;
    let float_const: f32 = 123.45_f32;
    let double_const: f64 = 1e-2;  // 0.01
    let char_const: char = 'x';
    let newline: char = '\n';
    let tab: char = '\t';

    println!("decimal: {}", decimal);
    println!("octal: {}", octal);
    println!("hex: {}", hex);
    println!("long: {}", long_const);
    println!("float: {}", float_const);
    println!("double: {}", double_const);
    println!("char: {}", char_const);
}
