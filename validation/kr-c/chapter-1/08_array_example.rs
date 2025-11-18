/* K&R C Chapter 1.6: Arrays
 * Page 20-21
 * Count digits, white space, and other characters
 * Transpiled to safe Rust
 */

use std::io::{self, Read};

fn main() {
    let mut c: u8;
    let mut i: usize;
    let mut nwhite: i32 = 0;
    let mut nother: i32 = 0;
    let mut ndigit: [i32; 10] = [0; 10];

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut buffer = [0u8; 1];

    loop {
        match handle.read(&mut buffer) {
            Ok(0) => break,  // EOF
            Ok(_) => {
                c = buffer[0];
                if c >= b'0' && c <= b'9' {
                    ndigit[(c - b'0') as usize] += 1;
                } else if c == b' ' || c == b'\n' || c == b'\t' {
                    nwhite += 1;
                } else {
                    nother += 1;
                }
            }
            Err(_) => break,
        }
    }

    print!("digits =");
    for i in 0..10 {
        print!(" {}", ndigit[i]);
    }
    println!(", white space = {}, other = {}", nwhite, nother);
}
