/* K&R C Chapter 7: Vectored I/O (Scatter-Gather)
 * Transpiled to safe Rust (mentioning std::io::IoSlice)
 */

use std::fs::File;
use std::io::{self, IoSlice, Write};

fn main() -> io::Result<()> {
    println!("=== Vectored I/O (Scatter-Gather) ===\n");
    
    writev_demo()?;
    
    println!("Vectored I/O in Rust:");
    println!("  - write_vectored() method");
    println!("  - IoSlice for output buffers");
    println!("  - IoSliceMut for input buffers");
    println!("  - Fewer syscalls than multiple writes");
    
    Ok(())
}

fn writev_demo() -> io::Result<()> {
    println!("=== write_vectored Demo ===");
    
    let mut file = File::create("writev_test.txt")?;
    
    let buf1 = b"First part, ";
    let buf2 = b"second part, ";
    let buf3 = b"third part.\n";
    
    let bufs = [
        IoSlice::new(buf1),
        IoSlice::new(buf2),
        IoSlice::new(buf3),
    ];
    
    let nwritten = file.write_vectored(&bufs)?;
    println!("Wrote {} bytes in one write_vectored call", nwritten);
    
    drop(file);
    
    let contents = std::fs::read_to_string("writev_test.txt")?;
    println!("File contents: {}", contents);
    
    std::fs::remove_file("writev_test.txt")?;
    
    Ok(())
}

// Key differences from C:
// 1. write_vectored() method instead of writev()
// 2. IoSlice instead of struct iovec
// 3. Type-safe buffer slices
// 4. RAII: automatic cleanup
// 5. Cross-platform API
// 6. No manual iov_len tracking
