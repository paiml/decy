/* K&R C Chapter 1.9: Character Arrays
 * Page 29
 * Print longest input line
 * Transpiled to safe Rust
 */

use std::io::{self, BufRead};

const MAXLINE: usize = 1000;

fn main() {
    let stdin = io::stdin();
    let mut max_len = 0;
    let mut longest = String::new();

    for line in stdin.lock().lines() {
        if let Ok(line) = line {
            let len = line.len();
            if len > max_len {
                max_len = len;
                longest = line;
            }
        }
    }

    if max_len > 0 {
        println!("Longest line ({} chars):", max_len);
        println!("{}", longest);
    }
}

// Alternative implementation using getline-style function
fn _getline_kr(max: usize) -> Option<String> {
    let stdin = io::stdin();
    let mut line = String::new();

    match stdin.lock().read_line(&mut line) {
        Ok(0) => None,  // EOF
        Ok(n) if n <= max => Some(line),
        Ok(_) => Some(line[..max].to_string()),  // Truncate if too long
        Err(_) => None,
    }
}

// No need for explicit copy function - String::clone() handles it
// No need for manual memory management - RAII handles cleanup
