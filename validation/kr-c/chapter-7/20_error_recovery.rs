/* K&R C Chapter 7: Error Recovery Patterns
 * Transpiled to safe Rust (using Result)
 */

use std::fs::{self, File};
use std::io::{self, Write};
use std::thread::sleep;
use std::time::Duration;

fn main() -> io::Result<()> {
    println!("=== Error Recovery Patterns ===\n");
    
    fs::write("test_input.txt", "Test data\n")?;
    
    println!("1. Retry Pattern:");
    open_with_retry("test_input.txt", 3)?;
    let _ = open_with_retry("nonexistent.txt", 3);
    println!();
    
    println!("2. Fallback Pattern:");
    open_with_fallback("nonexistent.txt", "test_input.txt")?;
    println!();
    
    println!("3. Transaction Pattern:");
    write_transaction("transaction_test.txt", "Critical data\n")?;
    println!();
    
    fs::remove_file("test_input.txt")?;
    fs::remove_file("transaction_test.txt")?;
    
    Ok(())
}

fn open_with_retry(filename: &str, max_attempts: u32) -> io::Result<File> {
    println!("Attempting to open: {}", filename);
    
    for attempt in 1..=max_attempts {
        match File::open(filename) {
            Ok(file) => {
                println!("  Success on attempt {}", attempt);
                return Ok(file);
            }
            Err(e) if attempt < max_attempts => {
                println!("  Attempt {} failed: {}", attempt, e);
                println!("  Retrying...");
                sleep(Duration::from_millis(100));
            }
            Err(e) => {
                println!("  Failed after {} attempts: {}", max_attempts, e);
                return Err(e);
            }
        }
    }
    
    unreachable!()
}

fn open_with_fallback(primary: &str, fallback: &str) -> io::Result<File> {
    match File::open(primary) {
        Ok(file) => {
            println!("Opened primary file: {}", primary);
            Ok(file)
        }
        Err(e) => {
            println!("Primary file failed: {}", e);
            println!("Trying fallback: {}", fallback);
            File::open(fallback).map(|file| {
                println!("Opened fallback file: {}", fallback);
                file
            })
        }
    }
}

fn write_transaction(filename: &str, data: &str) -> io::Result<()> {
    println!("Transaction: Writing to temp file");
    
    let temp_filename = format!("{}.tmp", filename);
    
    let mut file = File::create(&temp_filename)?;
    file.write_all(data.as_bytes())?;
    file.flush()?;
    drop(file);
    
    println!("  Renaming temp file to target");
    fs::rename(&temp_filename, filename)?;
    
    println!("  Transaction committed successfully");
    Ok(())
}

// Key differences from C:
// 1. Result<T, E> instead of errno
// 2. ? operator for error propagation
// 3. match for pattern matching errors
// 4. RAII: automatic cleanup
// 5. No goto needed for cleanup
// 6. Type-safe error handling
