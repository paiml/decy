/* K&R C Chapter 3.2: If-Else
 * Page 52-53
 * Basic if-else statement
 * Transpiled to safe Rust
 */

fn main() {
    let n: i32 = 10;

    if n > 0 {
        println!("n is positive");
    } else if n < 0 {
        println!("n is negative");
    } else {
        println!("n is zero");
    }
}
