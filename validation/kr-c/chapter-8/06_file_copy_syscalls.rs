/* K&R C Chapter 8.2: File Copy Using System Calls
 * Transpiled to safe Rust (using std::fs::copy)
 */

use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process;

const BUFSIZE: usize = 8192;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <source> <destination>", args[0]);
        process::exit(1);
    }
    
    if let Err(e) = copy_file(&args[1], &args[2]) {
        eprintln!("Error: {}", e);
        process::exit(2);
    }
    
    println!("File copied successfully");
}

fn copy_file(source: &str, dest: &str) -> io::Result<()> {
    // Method 1: Using std::fs::copy (most idiomatic)
    // std::fs::copy(source, dest)?;
    
    // Method 2: Manual buffered copy (like C version)
    let mut file_in = File::open(source)?;
    let mut file_out = File::create(dest)?;
    
    let mut buffer = [0u8; BUFSIZE];
    
    loop {
        let n = file_in.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        
        file_out.write_all(&buffer[..n])?;
    }
    
    Ok(())
}

// Key differences from C:
// 1. std::fs::copy for simple cases
// 2. Read/Write traits for manual copy
// 3. Result<T,E> propagation with ?
// 4. RAII: automatic close on drop
// 5. write_all() ensures full write
// 6. No manual error code tracking
