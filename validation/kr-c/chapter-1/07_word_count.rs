/* K&R C Chapter 1.5.4: Word Counting
 * Page 18-19
 * Count lines, words, and characters in input
 * Transpiled to safe Rust
 */

use std::io::{self, Read};

const IN: i32 = 1;   // inside a word
const OUT: i32 = 0;  // outside a word

fn main() {
    let mut c: u8;
    let mut nl: i32 = 0;
    let mut nw: i32 = 0;
    let mut nc: i32 = 0;
    let mut state: i32 = OUT;

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut buffer = [0u8; 1];

    loop {
        match handle.read(&mut buffer) {
            Ok(0) => break,  // EOF
            Ok(_) => {
                c = buffer[0];
                nc += 1;
                if c == b'\n' {
                    nl += 1;
                }
                if c == b' ' || c == b'\n' || c == b'\t' {
                    state = OUT;
                } else if state == OUT {
                    state = IN;
                    nw += 1;
                }
            }
            Err(_) => break,
        }
    }

    println!("{} {} {}", nl, nw, nc);
}
