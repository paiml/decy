/* K&R C Chapter 8.7: A Storage Allocator
 * Transpiled to safe Rust (demonstrating arena pattern)
 */

use std::cell::RefCell;

const ALLOCSIZE: usize = 10000;

struct SimpleAllocator {
    buffer: [u8; ALLOCSIZE],
    offset: usize,
}

impl SimpleAllocator {
    fn new() -> Self {
        SimpleAllocator {
            buffer: [0; ALLOCSIZE],
            offset: 0,
        }
    }
    
    fn alloc(&mut self, n: usize) -> Option<&mut [u8]> {
        if self.offset + n <= ALLOCSIZE {
            let start = self.offset;
            self.offset += n;
            Some(&mut self.buffer[start..self.offset])
        } else {
            None
        }
    }
    
    fn free_all(&mut self) {
        self.offset = 0;
    }
    
    fn available(&self) -> usize {
        ALLOCSIZE - self.offset
    }
}

fn main() {
    println!("=== Simple Allocator (Arena Pattern) ===\n");
    
    let allocator = RefCell::new(SimpleAllocator::new());
    
    println!("Allocator buffer size: {} bytes", ALLOCSIZE);
    
    // Allocate some memory
    {
        let mut alloc = allocator.borrow_mut();
        
        if let Some(p1) = alloc.alloc(100) {
            println!("Allocated 100 bytes");
            p1.copy_from_slice(b"Hello from p1\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
        }
        
        if let Some(_p2) = alloc.alloc(200) {
            println!("Allocated 200 bytes");
        }
        
        if let Some(_p3) = alloc.alloc(5000) {
            println!("Allocated 5000 bytes");
        }
        
        println!("Remaining: {} bytes", alloc.available());
        
        // Try to allocate too much
        if alloc.alloc(20000).is_none() {
            println!("Failed to allocate 20000 bytes (buffer full)");
        }
    }
    
    // Free all
    allocator.borrow_mut().free_all();
    println!("Freed all memory");
    
    println!("\nRust memory management:");
    println!("  - Box<T> for single heap allocation");
    println!("  - Vec<T> for dynamic arrays");
    println!("  - Arena/bump allocators for batch allocation");
    println!("  - bumpalo crate for production arenas");
}

// Key differences from C:
// 1. Arena pattern (bump allocator)
// 2. No raw pointers - safe slices
// 3. Lifetimes prevent use-after-free
// 4. Box<T>, Vec<T> for normal allocation
// 5. RefCell for interior mutability
// 6. Automatic deallocation (RAII)
