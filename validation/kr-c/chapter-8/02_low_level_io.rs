/* K&R C Chapter 8.2: Low Level I/O - Read and Write
 * Page 170-171
 * Using read() and write() system calls
 * Transpiled to safe Rust using std::fs and std::io
 */

use std::env;
use std::fs::File;
use std::io::{self, Read, Write};

const BUFSIZE: usize = 4096;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    // Open file for reading
    let mut fd = File::open(&args[1])?;

    // Copy file to stdout
    let mut buf = [0u8; BUFSIZE];
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    loop {
        let n = fd.read(&mut buf)?;
        if n == 0 {
            break;
        }
        handle.write_all(&buf[..n])?;
    }

    Ok(())
}
