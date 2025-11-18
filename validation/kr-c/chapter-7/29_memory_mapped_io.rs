/* K&R C Chapter 7: Memory-Mapped I/O
 * Transpiled to safe Rust (using memmap2 crate pattern)
 */

use std::fs::File;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    println!("=== Memory-Mapped I/O (Rust Pattern) ===\n");
    
    println!("Rust memory-mapped I/O:");
    println!("  - Use memmap2 crate for production");
    println!("  - unsafe but zero-copy performance");
    println!("  - Automatic cleanup on drop");
    println!();
    
    println!("Example with memmap2 crate:");
    println!("```rust");
    println!("use memmap2::MmapOptions;");
    println!("use std::fs::File;");
    println!();
    println!("let file = File::open(\"data.txt\")?;");
    println!("let mmap = unsafe {{ MmapOptions::new().map(&file)? }};");
    println!();
    println!("// Read from mapped memory");
    println!("let first_byte = mmap[0];");
    println!();
    println!("// Mutable mapping");
    println!("let file = OpenOptions::new().read(true).write(true).open(\"data.txt\")?;");
    println!("let mut mmap = unsafe {{ MmapOptions::new().map_mut(&file)? }};");
    println!("mmap[0] = b'X';  // Modify in-place");
    println!("mmap.flush()?;    // Sync to disk");
    println!("```");
    println!();
    
    println!("Benefits:");
    println!("  - Fast access to file contents");
    println!("  - OS handles paging automatically");
    println!("  - Efficient for large files");
    println!("  - In-place modification");
    
    Ok(())
}

// Production code should use memmap2:
// [dependencies]
// memmap2 = "0.9"

// Key differences from C:
// 1. memmap2 crate instead of mmap syscall
// 2. Mmap/MmapMut types
// 3. RAII: automatic munmap on drop
// 4. flush() instead of msync
// 5. Type-safe (with unsafe block)
