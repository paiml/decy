/* K&R C Chapter 2.7: Type Conversions
 * Page 44-46
 * Implicit and explicit type conversions
 * Transpiled to safe Rust
 */

fn main() {
    let i: i32 = 42;
    let f: f32 = 3.14;
    let d: f64 = 2.71828;
    let c: char = 'A';

    println!("Implicit conversions:");

    // int to float
    let f1: f32 = i as f32;
    println!("int {} to float: {}", i, f1);

    // float to int (truncation)
    let i1: i32 = f as i32;
    println!("float {} to int: {}", f, i1);

    // char to int
    let i2: i32 = c as i32;
    println!("char '{}' to int: {}", c, i2);

    // Mixed arithmetic
    let result: f32 = (i as f32) + f;
    println!("int + float: {} + {} = {}", i, f, result);

    println!("\nExplicit conversions (casts):");

    // Explicit cast
    let i3: i32 = d as i32;
    println!("(int){} = {}", d, i3);

    // Cast in expression
    let ratio: f32 = (i as f32) / 3.0;
    println!("(float){} / 3 = {}", i, ratio);

    // Without cast (integer division)
    let int_div: i32 = i / 3;
    println!("{} / 3 (int division) = {}", i, int_div);

    // Pointer representation (safe alternative)
    let addr: usize = &i as *const i32 as usize;
    println!("Address of i: {} (0x{:X})", addr, addr);
}
