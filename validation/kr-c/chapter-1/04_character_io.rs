/* K&R C Chapter 1.5.1: File Copying
 * Page 15
 * Character input/output example
 * Transpiled to safe Rust
 */

use std::io::{self, Read};

fn main() {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut buffer = [0u8; 1];

    loop {
        match handle.read(&mut buffer) {
            Ok(0) => break,  // EOF
            Ok(_) => {
                print!("{}", buffer[0] as char);
            }
            Err(_) => break,
        }
    }
}
