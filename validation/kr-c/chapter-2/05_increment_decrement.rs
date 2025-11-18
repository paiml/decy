/* K&R C Chapter 2.8: Increment and Decrement Operators
 * Page 43
 * ++ and -- operators
 * Transpiled to safe Rust
 */

fn main() {
    let mut n: i32 = 5;
    let mut x: i32;
    let mut y: i32;

    // Post-increment (simulated)
    x = n;
    n += 1;
    println!("After n++: x = {}, n = {}", x, n);

    // Pre-increment
    n = 5;
    n += 1;
    x = n;
    println!("After ++n: x = {}, n = {}", x, n);

    // Post-decrement (simulated)
    n = 5;
    y = n;
    n -= 1;
    println!("After n--: y = {}, n = {}", y, n);

    // Pre-decrement
    n = 5;
    n -= 1;
    y = n;
    println!("After --n: y = {}, n = {}", y, n);
}
