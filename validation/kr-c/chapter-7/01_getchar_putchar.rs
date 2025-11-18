/* K&R C Chapter 7.1: Standard Input and Output
 * Page 151-152
 * Basic character I/O with getchar and putchar
 * Transpiled to safe Rust
 */

use std::io::{self, Read};

fn main() {
    println!("Type some characters (Ctrl+D to end):");

    // Copy input to output
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
