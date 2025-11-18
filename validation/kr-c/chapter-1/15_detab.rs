/* K&R C Chapter 1 Exercise: Detab
 * Based on Chapter 1 concepts
 * Replace tabs with spaces
 * Transpiled to safe Rust
 */

use std::io::{self, Read};

const TABSTOP: usize = 8;

fn main() {
    let mut c: u8;
    let mut pos: usize = 0;

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut buffer = [0u8; 1];

    loop {
        match handle.read(&mut buffer) {
            Ok(0) => break,  // EOF
            Ok(_) => {
                c = buffer[0];
                if c == b'\t' {
                    // Replace tab with spaces to next tab stop
                    loop {
                        print!(" ");
                        pos += 1;
                        if pos % TABSTOP == 0 {
                            break;
                        }
                    }
                } else if c == b'\n' {
                    print!("{}", c as char);
                    pos = 0;
                } else {
                    print!("{}", c as char);
                    pos += 1;
                }
            }
            Err(_) => break,
        }
    }
}
