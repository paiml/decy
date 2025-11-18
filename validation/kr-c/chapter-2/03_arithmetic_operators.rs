/* K&R C Chapter 2.5: Arithmetic Operators
 * Page 39-40
 * Binary arithmetic operators
 * Transpiled to safe Rust
 */

fn main() {
    let a: i32 = 10;
    let b: i32 = 3;
    let sum: i32;
    let diff: i32;
    let prod: i32;
    let quot: i32;
    let rem: i32;

    sum = a + b;
    diff = a - b;
    prod = a * b;
    quot = a / b;
    rem = a % b;

    println!("a = {}, b = {}", a, b);
    println!("a + b = {}", sum);
    println!("a - b = {}", diff);
    println!("a * b = {}", prod);
    println!("a / b = {}", quot);
    println!("a % b = {}", rem);
}
