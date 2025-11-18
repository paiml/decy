/* K&R C Chapter 7.5: File Copying
 * Page 162
 * Transpiled to safe Rust (using std::fs::copy and Read/Write traits)
 */

use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::process;

fn main() {
    println!("=== File Copying ===\n");

    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <source> <destination>", args[0]);
        process::exit(1);
    }

    let source = &args[1];
    let dest = &args[2];

    // Method 1: Using std::fs::copy (simplest, idiomatic)
    println!("=== Method 1: fs::copy ===");
    match fs::copy(source, dest) {
        Ok(bytes) => {
            println!("Copied {} bytes from '{}' to '{}'", bytes, source, dest);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }

    // Method 2: Manual copying with Read/Write (like C version)
    println!("\n=== Method 2: Manual byte-by-byte copy ===");
    if let Err(e) = copy_manual(source, "output_manual.txt") {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    // Method 3: Buffered copying (efficient)
    println!("\n=== Method 3: Buffered copy ===");
    if let Err(e) = copy_buffered(source, "output_buffered.txt") {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    println!("\nFile copied successfully");
}

// Manual byte-by-byte copy (similar to C version)
fn copy_manual(source: &str, dest: &str) -> io::Result<()> {
    let mut input = File::open(source)?;
    let mut output = File::create(dest)?;

    let mut byte = [0u8; 1];
    let mut count = 0;

    loop {
        match input.read(&mut byte)? {
            0 => break,  // EOF
            _ => {
                output.write_all(&byte)?;
                count += 1;
            }
        }
    }

    println!("Copied {} bytes manually", count);
    Ok(())
}

// Buffered copy (more efficient)
fn copy_buffered(source: &str, dest: &str) -> io::Result<()> {
    let mut input = File::open(source)?;
    let mut output = File::create(dest)?;

    let bytes_copied = io::copy(&mut input, &mut output)?;

    println!("Copied {} bytes with buffering", bytes_copied);
    Ok(())
}

// Copy with progress reporting
#[allow(dead_code)]
fn copy_with_progress(source: &str, dest: &str) -> io::Result<()> {
    let mut input = File::open(source)?;
    let mut output = File::create(dest)?;

    let mut buffer = [0u8; 8192];  // 8KB buffer
    let mut total = 0;

    loop {
        let bytes_read = input.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        output.write_all(&buffer[..bytes_read])?;
        total += bytes_read;

        if total % (1024 * 1024) == 0 {  // Every MB
            println!("Copied {} MB", total / (1024 * 1024));
        }
    }

    println!("Total: {} bytes", total);
    Ok(())
}

// Key differences from C:
// 1. fs::copy() for simple copying
// 2. Read/Write traits instead of getc/putc
// 3. Result<T, E> for error handling
// 4. ? operator for error propagation
// 5. RAII: files auto-close
// 6. No need for explicit fclose
// 7. io::copy for efficient buffered copying
// 8. Type-safe file operations
