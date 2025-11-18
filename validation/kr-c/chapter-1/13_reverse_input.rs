/* K&R C Chapter 1 Exercise: Reverse input
 * Based on Chapter 1 concepts
 * Read input and print lines in reverse
 * Transpiled to safe Rust
 */

use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        if let Ok(mut line) = line {
            reverse(&mut line);
            println!("{}", line);
        }
    }
}

fn reverse(s: &mut String) {
    // Remove trailing newline if present (Rust lines() already strips it)
    // Reverse the string using safe iterator methods
    let reversed: String = s.chars().rev().collect();
    s.clear();
    s.push_str(&reversed);
}

// Alternative implementation using character array (closer to C)
fn _reverse_array(chars: &mut [char]) {
    let len = chars.len();

    // Swap elements from both ends
    for i in 0..len / 2 {
        chars.swap(i, len - 1 - i);
    }
}

// Demonstration of safe iteration vs pointer arithmetic
fn _manual_reverse_demo() {
    let mut text = String::from("Hello, world!");

    // Safe Rust approach: use built-in methods
    let reversed: String = text.chars().rev().collect();

    // Or use slice reversal
    let mut chars: Vec<char> = text.chars().collect();
    chars.reverse();

    text = chars.iter().collect();

    println!("Reversed: {}", reversed);
}
