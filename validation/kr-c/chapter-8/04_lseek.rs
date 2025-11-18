/* K&R C Chapter 8.4: Random Access - lseek
 * Transpiled to safe Rust (using Seek trait)
 */

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};

fn main() -> io::Result<()> {
    println!("=== Random Access with Seek ===\n");
    
    // Create file with content
    let mut file = File::create("seektest.txt")?;
    file.write_all(b"0123456789ABCDEFGHIJ")?;
    drop(file);
    
    // Open for reading
    let mut file = File::open("seektest.txt")?;
    
    // Read from beginning
    let mut buffer = [0u8; 5];
    file.read_exact(&mut buffer)?;
    println!("First 5 bytes: {}", String::from_utf8_lossy(&buffer));
    
    // Seek to position 10
    let pos = file.seek(SeekFrom::Start(10))?;
    println!("Seeked to position: {}", pos);
    
    file.read_exact(&mut buffer)?;
    println!("Next 5 bytes: {}", String::from_utf8_lossy(&buffer));
    
    // Seek to end
    let pos = file.seek(SeekFrom::End(0))?;
    println!("File size: {} bytes", pos);
    
    // Seek backward from end
    file.seek(SeekFrom::End(-5))?;
    file.read_exact(&mut buffer)?;
    println!("Last 5 bytes: {}", String::from_utf8_lossy(&buffer));
    
    drop(file);
    std::fs::remove_file("seektest.txt")?;
    
    Ok(())
}

// Key differences from C:
// 1. Seek trait instead of lseek()
// 2. SeekFrom enum for whence parameter
// 3. Type-safe seek operations
// 4. RAII: automatic close on drop
// 5. Result<T,E> for error handling
