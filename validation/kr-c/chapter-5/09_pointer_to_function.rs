/* K&R C Chapter 5.11: Pointers to Functions
 * Page 118-119
 * Function pointers for callback
 * Transpiled to safe Rust
 */

fn add(a: i32, b: i32) -> i32 { a + b }
fn subtract(a: i32, b: i32) -> i32 { a - b }
fn multiply(a: i32, b: i32) -> i32 { a * b }

fn apply(operation: fn(i32, i32) -> i32, x: i32, y: i32) -> i32 {
    operation(x, y)
}

fn main() {
    let a: i32 = 10;
    let b: i32 = 5;

    println!("add({}, {}) = {}", a, b, apply(add, a, b));
    println!("subtract({}, {}) = {}", a, b, apply(subtract, a, b));
    println!("multiply({}, {}) = {}", a, b, apply(multiply, a, b));
}
