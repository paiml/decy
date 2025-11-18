/* K&R C Chapter 1.5.2: Character Counting
 * Page 16
 * Count characters in input
 * Transpiled to safe Rust
 */

use std::io::{self, Read};

fn main() {
    let mut nc: i64 = 0;
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut buffer = [0u8; 1];

    loop {
        match handle.read(&mut buffer) {
            Ok(0) => break,  // EOF
            Ok(_) => {
                nc += 1;
            }
            Err(_) => break,
        }
    }

    println!("{}", nc);
}
