/* K&R C Chapter 1.5.3: Line Counting
 * Page 17
 * Count lines in input
 * Transpiled to safe Rust
 */

use std::io::{self, BufRead};

fn main() {
    let mut nl: i32 = 0;
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        if line.is_ok() {
            nl += 1;
        }
    }

    println!("{}", nl);
}
