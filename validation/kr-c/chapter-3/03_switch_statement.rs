/* K&R C Chapter 3.4: Switch
 * Page 55-57
 * Switch statement example
 * Transpiled to safe Rust using match
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
                match c {
                    b'0' | b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' | b'8' | b'9' => {
                        ndigit[(c - b'0') as usize] += 1;
                    }
                    b' ' | b'\n' | b'\t' => {
                        nwhite += 1;
                    }
                    _ => {
                        nother += 1;
                    }
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
