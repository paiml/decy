/* K&R C Chapter 8: Shared Memory with mmap
 * Transpiled to safe Rust (using Arc + Atomic pattern)
 */

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    println!("=== Shared Memory (Rust Pattern) ===\n");
    
    demo_shared_atomic();
    
    println!("Shared memory in Rust:");
    println!("  - Arc<T> for shared ownership");
    println!("  - Atomic types for lock-free sync");
    println!("  - Mutex<T> for complex data");
    println!("  - memmap2 crate for file-backed shared memory");
    println!("  - No MAP_SHARED - use thread-safe abstractions");
    
    println!("\nFor file-backed shared memory:");
    println!("```rust");
    println!("use memmap2::MmapMut;");
    println!("use std::fs::OpenOptions;");
    println!();
    println!("let file = OpenOptions::new()");
    println!("    .read(true).write(true).create(true)");
    println!("    .open(\"shared.dat\")?;");
    println!("file.set_len(4096)?;");
    println!();
    println!("let mut mmap = unsafe {{ MmapMut::map_mut(&file)? }};");
    println!("mmap[0] = 42;  // Shared with other processes");
    println!("```");
}

fn demo_shared_atomic() {
    println!("=== Shared Atomic Counter ===");
    
    let counter = Arc::new(AtomicI32::new(0));
    
    let counter1 = Arc::clone(&counter);
    let handle1 = thread::spawn(move || {
        for _ in 0..1000 {
            counter1.fetch_add(1, Ordering::SeqCst);
        }
    });
    
    let counter2 = Arc::clone(&counter);
    let handle2 = thread::spawn(move || {
        for _ in 0..1000 {
            counter2.fetch_add(1, Ordering::SeqCst);
        }
    });
    
    handle1.join().unwrap();
    handle2.join().unwrap();
    
    println!("Final counter: {}", counter.load(Ordering::SeqCst));
    println!("(Expected: 2000)\n");
}

// Key differences from C:
// 1. Arc<Atomic*> instead of mmap
// 2. Thread-safe by design
// 3. No raw shared memory pointers
// 4. Ordering semantics explicit
// 5. memmap2 crate for file-backed IPC
// 6. Type-safe synchronization
