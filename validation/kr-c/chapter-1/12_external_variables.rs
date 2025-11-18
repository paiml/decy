/* K&R C Chapter 1.10: External Variables and Scope
 * Page 31-33
 * Longest line using external variables
 * Transpiled to safe Rust using static mut (matching C semantics)
 */

use std::io::{self, BufRead};

const MAXLINE: usize = 1000;

// External variables (global state)
// In Rust, we use static mut, which requires unsafe access
static mut MAX: usize = 0;
static mut LINE: [u8; MAXLINE] = [0; MAXLINE];
static mut LONGEST: [u8; MAXLINE] = [0; MAXLINE];

fn main() {
    let stdin = io::stdin();

    unsafe {
        MAX = 0;

        for line_result in stdin.lock().lines() {
            if let Ok(line_str) = line_result {
                let len = getline_ext(&line_str);
                if len > MAX {
                    MAX = len;
                    copy_ext();
                }
            }
        }

        if MAX > 0 {
            let longest_str = String::from_utf8_lossy(&LONGEST[..MAX]);
            println!("Longest: {}", longest_str);
        }
    }
}

fn getline_ext(input: &str) -> usize {
    unsafe {
        let bytes = input.as_bytes();
        let len = bytes.len().min(MAXLINE - 1);

        LINE[..len].copy_from_slice(&bytes[..len]);
        LINE[len] = 0;  // null terminator

        len
    }
}

fn copy_ext() {
    unsafe {
        LONGEST.copy_from_slice(&LINE);
    }
}

// Note: This uses unsafe to match C's global variable semantics
// A safer Rust approach would use thread-local storage or pass state as parameters
// But for educational comparison with K&R C, we preserve the external variable pattern
