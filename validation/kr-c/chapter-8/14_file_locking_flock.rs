/* K&R C Chapter 8: File Locking with flock
 * Transpiled to safe Rust (using fs2 crate pattern)
 */

use std::fs::File;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn main() -> io::Result<()> {
    println!("=== File Locking (Rust Pattern) ===\n");
    
    println!("Note: Rust std doesn't expose flock directly");
    println!("Use fs2 crate for file locking:\n");
    
    demo_file_locking()?;
    
    println!("File locking in Rust:");
    println!("  - fs2 crate: lock_exclusive(), lock_shared()");
    println!("  - Cross-platform (flock on Unix, LockFile on Windows)");
    println!("  - Advisory locking (cooperative)");
    println!("  - RAII: unlock on drop");
    
    println!("\nExample with fs2 crate:");
    println!("```rust");
    println!("use fs2::FileExt;");
    println!("use std::fs::File;");
    println!();
    println!("let file = File::create(\"lockfile.txt\")?;");
    println!();
    println!("// Exclusive lock");
    println!("file.lock_exclusive()?;");
    println!("// ... critical section ...");
    println!("file.unlock()?;");
    println!();
    println!("// Shared lock");
    println!("file.lock_shared()?;");
    println!("// ... read-only section ...");
    println!("file.unlock()?;");
    println!();
    println!("// Non-blocking");
    println!("file.try_lock_exclusive()?;");
    println!("```");
    
    Ok(())
}

fn demo_file_locking() -> io::Result<()> {
    println!("=== Simulated Lock Demo ===");
    
    let mut file = File::create("lockfile.txt")?;
    
    println!("Acquiring lock...");
    file.write_all(b"Locked data\n")?;
    
    println!("  Holding lock for 2 seconds...");
    thread::sleep(Duration::from_secs(2));
    
    println!("  Lock released");
    
    drop(file);
    std::fs::remove_file("lockfile.txt")?;
    println!();
    
    Ok(())
}

// Key differences from C:
// 1. fs2 crate instead of flock()
// 2. FileExt trait
// 3. RAII: automatic unlock on drop
// 4. Cross-platform API
// 5. Type-safe lock operations
