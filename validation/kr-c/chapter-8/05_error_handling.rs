/* K&R C Chapter 8.5: Error Handling with System Calls
 * Transpiled to safe Rust (using io::Error)
 */

use std::fs::File;
use std::io::{self, Write};

fn main() {
    println!("=== Error Handling ===\n");
    
    // Try to open non-existent file
    match File::open("nonexistent.txt") {
        Ok(_) => println!("File opened"),
        Err(e) => {
            println!("Error opening file:");
            println!("  Kind: {:?}", e.kind());
            println!("  Message: {}", e);
            eprintln!("open: {}", e);
        }
    }
    
    println!();
    
    // Try to open and write to read-only file
    match File::open("/etc/passwd") {
        Ok(mut file) => {
            println!("Opened /etc/passwd successfully");
            
            match file.write_all(b"test") {
                Ok(_) => println!("Wrote successfully"),
                Err(e) => {
                    println!("Cannot write to read-only file");
                    eprintln!("write: {}", e);
                }
            }
        }
        Err(e) => eprintln!("open: {}", e),
    }
    
    println!();
    
    // Try to create file in non-existent directory
    match File::create("/nonexistent/dir/file.txt") {
        Ok(_) => println!("File created"),
        Err(e) => {
            println!("Cannot create file in non-existent directory");
            eprintln!("create: {}", e);
        }
    }
}

// Key differences from C:
// 1. io::Error instead of errno
// 2. Result<T,E> for all I/O operations
// 3. match for error handling
// 4. ErrorKind enum for error types
// 5. No global errno variable
// 6. Type-safe error propagation
