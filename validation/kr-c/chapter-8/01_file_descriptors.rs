/* K&R C Chapter 8.1: File Descriptors
 * Page 169-170
 * Basic file descriptor operations
 * Transpiled to safe Rust using std::fs
 */

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

fn main() -> std::io::Result<()> {
    let mut buffer = [0u8; 100];

    // Standard file descriptors (conceptually)
    println!("STDIN: 0");
    println!("STDOUT: 1");
    println!("STDERR: 2");

    // Create and open a file
    let mut fd = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("test.txt")?;

    println!("Opened file");

    // Write to file
    fd.write_all(b"Hello, world!\n")?;

    // Drop file to close it
    drop(fd);

    // Reopen to read
    let mut fd = File::open("test.txt")?;
    let n = fd.read(&mut buffer)?;

    let content = std::str::from_utf8(&buffer[..n]).unwrap();
    print!("Read from file: {}", content);

    Ok(())
}
