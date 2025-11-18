/* K&R C Chapter 2.6: Relational and Logical Operators
 * Page 40-41
 * Relational and logical operators
 * Transpiled to safe Rust
 */

fn main() {
    let a: i32 = 5;
    let b: i32 = 10;
    let mut result: bool;

    // Relational operators
    result = a > b;
    println!("a > b: {}", result as i32);

    result = a < b;
    println!("a < b: {}", result as i32);

    result = a >= b;
    println!("a >= b: {}", result as i32);

    result = a <= b;
    println!("a <= b: {}", result as i32);

    result = a == b;
    println!("a == b: {}", result as i32);

    result = a != b;
    println!("a != b: {}", result as i32);

    // Logical operators
    result = a < b && b < 20;
    println!("a < b && b < 20: {}", result as i32);

    result = a > b || b > 0;
    println!("a > b || b > 0: {}", result as i32);

    result = !(a == b);
    println!("!(a == b): {}", result as i32);
}
