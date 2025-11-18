/* K&R C Chapter 1.7: Functions
 * Page 24
 * Simple function example - power function
 * Transpiled to safe Rust
 */

// power: raise base to n-th power; n >= 0
fn power(base: i32, n: i32) -> i32 {
    let mut i: i32;
    let mut p: i32;

    p = 1;
    i = 1;
    while i <= n {
        p = p * base;
        i += 1;
    }
    return p;
}

fn main() {
    let mut i: i32;

    i = 0;
    while i < 10 {
        println!("{} {} {}", i, power(2, i), power(-3, i));
        i += 1;
    }
}
