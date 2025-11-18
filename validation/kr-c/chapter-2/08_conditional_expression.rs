/* K&R C Chapter 2.11: Conditional Expressions
 * Page 51
 * Ternary operator (? :)
 * Transpiled to safe Rust
 */

fn main() {
    let a: i32 = 10;
    let b: i32 = 20;
    let max: i32;
    let min: i32;

    // Find maximum using if expression
    max = if a > b { a } else { b };
    println!("max({}, {}) = {}", a, b, max);

    // Find minimum
    min = if a < b { a } else { b };
    println!("min({}, {}) = {}", a, b, min);

    // Nested conditional
    let x: i32 = 5;
    let y: i32 = 10;
    let z: i32 = 15;
    let largest: i32 = if x > y {
        if x > z { x } else { z }
    } else {
        if y > z { y } else { z }
    };
    println!("max({}, {}, {}) = {}", x, y, z, largest);

    // Conditional in expression
    println!("{} is {}", a, if a % 2 == 0 { "even" } else { "odd" });
    println!("{} is {}", b + 1, if (b + 1) % 2 == 0 { "even" } else { "odd" });

    // Return absolute value
    let n: i32 = -42;
    let abs_n: i32 = if n < 0 { -n } else { n };
    println!("abs({}) = {}", n, abs_n);
}
