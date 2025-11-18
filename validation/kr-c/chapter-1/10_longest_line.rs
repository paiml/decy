/* K&R C Chapter 1.9: Character Arrays
 * Page 27-29
 * Print longest line from input
 * Transpiled to safe Rust
 */

use std::io::{self, BufRead};

const MAXLINE: usize = 1000;

fn main() {
    let mut max: usize = 0;
    let mut longest: String = String::new();

    let stdin = io::stdin();
    for line_result in stdin.lock().lines() {
        if let Ok(line) = line_result {
            let len = line.len();
            if len > max {
                max = len;
                longest = line.clone();
            }
        }
    }

    if max > 0 {
        println!("{}", longest);
    }
}
