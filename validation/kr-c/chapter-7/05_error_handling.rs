/* K&R C Chapter 7.6: Error Handling - stderr and exit
 * Page 163-164
 * Transpiled to safe Rust (using Result, eprintln!, std::process::exit)
 */

use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::process;

fn main() {
    println!("=== Error Handling (stderr and exit) ===\n");

    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];

    // Attempt to open file
    let mut file = match File::open(filename) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: Cannot open file '{}': {}", filename, e);
            process::exit(2);
        }
    };

    println!("Successfully opened: {}", filename);

    // Read file contents
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(bytes) => println!("Read {} bytes", bytes),
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            process::exit(3);
        }
    }

    // File is automatically closed (RAII)
    println!("File closed automatically");

    // Demonstrate Result-based error handling (idiomatic Rust)
    println!("\n=== Idiomatic Rust Error Handling ===\n");

    if let Err(e) = process_file_idiomatic(filename) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

// Idiomatic Rust: return Result instead of exit
fn process_file_idiomatic(filename: &str) -> io::Result<()> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    println!("Read {} bytes from {}", contents.len(), filename);
    println!("First 50 chars: {}", &contents.chars().take(50).collect::<String>());

    Ok(())
}

// Key differences from C:
// 1. eprintln! writes to stderr
// 2. process::exit(code) instead of exit(code)
// 3. Result<T, E> for recoverable errors
// 4. ? operator for error propagation
// 5. RAII: files auto-close on drop
// 6. No NULL pointers
// 7. Type-safe error handling
// 8. env::args() for command line arguments
