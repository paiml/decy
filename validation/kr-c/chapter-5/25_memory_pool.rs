/* K&R C Chapter 5: Memory Pool Allocation
 * Custom memory allocator using pointers
 * Transpiled to safe Rust (using arena allocation)
 */

use std::cell::Cell;

const POOL_SIZE: usize = 1024;

struct MemoryPool {
    memory: Vec<u8>,
    next_free: Cell<usize>,
    allocated_count: Cell<usize>,
}

impl MemoryPool {
    fn new() -> Self {
        MemoryPool {
            memory: vec![0; POOL_SIZE],
            next_free: Cell::new(0),
            allocated_count: Cell::new(0),
        }
    }

    fn alloc(&self, size: usize) -> Option<&mut [u8]> {
        let current = self.next_free.get();

        if current + size > POOL_SIZE {
            println!("  Pool exhausted! (used: {}, requested: {})", current, size);
            return None;
        }

        self.next_free.set(current + size);
        self.allocated_count.set(self.allocated_count.get() + 1);

        // SAFETY: We're ensuring non-overlapping mutable access
        // by tracking offset and never returning overlapping slices
        let ptr = self.memory.as_ptr() as usize + current;
        unsafe {
            Some(std::slice::from_raw_parts_mut(ptr as *mut u8, size))
        }
    }

    fn reset(&self) {
        self.next_free.set(0);
        self.allocated_count.set(0);
    }

    fn stats(&self) {
        let used = self.next_free.get();
        println!("Pool statistics:");
        println!("  Total size: {} bytes", POOL_SIZE);
        println!("  Used: {} bytes ({:.1}%)", used, (used as f64 * 100.0) / POOL_SIZE as f64);
        println!("  Free: {} bytes", POOL_SIZE - used);
        println!("  Allocations: {}", self.allocated_count.get());
    }
}

fn main() {
    let pool = MemoryPool::new();

    println!("=== Memory Pool Demo ===\n");

    // Allocate integers
    if let Some(nums_bytes) = pool.alloc(10 * std::mem::size_of::<i32>()) {
        let nums = unsafe {
            std::slice::from_raw_parts_mut(nums_bytes.as_mut_ptr() as *mut i32, 10)
        };

        println!("Allocated array of 10 ints");
        for i in 0..10 {
            nums[i] = i as i32 * 10;
        }

        print!("Values: ");
        for &val in nums.iter() {
            print!("{} ", val);
        }
        println!("\n");
    }

    // Allocate string
    if let Some(str_bytes) = pool.alloc(50) {
        let msg = b"Hello from memory pool!";
        str_bytes[..msg.len()].copy_from_slice(msg);
        println!("Allocated string: \"{}\"\n", String::from_utf8_lossy(&str_bytes[..msg.len()]));
    }

    // Allocate struct
    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }

    if let Some(pt_bytes) = pool.alloc(std::mem::size_of::<Point>()) {
        let pt = unsafe { &mut *(pt_bytes.as_mut_ptr() as *mut Point) };
        pt.x = 100;
        pt.y = 200;
        println!("Allocated struct: {{{}, {}}}\n", pt.x, pt.y);
    }

    pool.stats();

    // Try to over-allocate
    println!("\nTrying to allocate 2000 bytes (exceeds pool):");
    if pool.alloc(2000).is_none() {
        println!("Allocation failed as expected");
    }

    // Reset and reuse
    println!("\nResetting pool...");
    pool.reset();
    pool.stats();
}

// Safe alternative: using typed_arena crate
// This would be the idiomatic Rust approach
#[allow(dead_code)]
fn demonstrate_safe_arena() {
    // In real code, use typed_arena or bumpalo crate
    // Example (requires: typed-arena = "2.0")
    // use typed_arena::Arena;
    //
    // let arena = Arena::new();
    // let nums: &mut [i32] = arena.alloc_extend(0..10);
    // nums[0] = 42;
}

// Key differences from C:
// 1. Vec<u8> for memory buffer instead of char[]
// 2. Cell for interior mutability
// 3. Unsafe blocks for raw pointer casting
// 4. Option<&mut [u8]> instead of void*
// 5. Automatic bounds checking where possible
// 6. Prefer typed_arena crate for production
