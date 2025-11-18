/* K&R C Chapter 7: File Locking
 * Transpiled to safe Rust (using fs2 crate pattern)
 */

use std::fs::{File, OpenOptions};
use std::io::{self, Write};

fn main() -> io::Result<()> {
    println!("=== File Locking (Rust Pattern) ===\n");
    
    let test_file = "lock_test.txt";
    
    println!("Safe write:");
    safe_write(test_file, "Locked write operation\n")?;
    println!();
    
    println!("Safe append:");
    safe_append(test_file, "Appended line 1\n")?;
    safe_append(test_file, "Appended line 2\n")?;
    println!();
    
    std::fs::remove_file(test_file)?;
    
    println!("File locking in Rust:");
    println!("  - Use `fs2` crate for portable locking");
    println!("  - lock_exclusive() for write locks");
    println!("  - lock_shared() for read locks");
    println!("  - Automatic unlock on drop (RAII)");
    println!("\nNote: This example shows the pattern");
    println!("Production code should use the fs2 crate");
    
    Ok(())
}

fn safe_write(filename: &str, data: &str) -> io::Result<()> {
    println!("Safe write to: {}", filename);
    
    let mut file = File::create(filename)?;
    
    // In production, use: file.lock_exclusive()?;
    
    file.write_all(data.as_bytes())?;
    println!("  Wrote {} bytes", data.len());
    
    // Automatic unlock on drop
    Ok(())
}

fn safe_append(filename: &str, data: &str) -> io::Result<()> {
    println!("Safe append to: {}", filename);
    
    let mut file = OpenOptions::new()
        .append(true)
        .open(filename)?;
    
    // In production, use: file.lock_exclusive()?;
    
    file.write_all(data.as_bytes())?;
    println!("  Appended {} bytes", data.len());
    
    Ok(())
}

// Production example with fs2 crate:
// use fs2::FileExt;
//
// fn locked_write(filename: &str, data: &str) -> io::Result<()> {
//     let file = File::create(filename)?;
//     file.lock_exclusive()?;  // Blocks until lock acquired
//     file.write_all(data.as_bytes())?;
//     file.unlock()?;  // Or drop for automatic unlock
//     Ok(())
// }

// Key differences from C:
// 1. fs2 crate instead of fcntl
// 2. lock_exclusive/lock_shared methods
// 3. Automatic unlock on drop
// 4. Cross-platform API
// 5. RAII for lock management
