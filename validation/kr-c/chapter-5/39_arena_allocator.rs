/* K&R C Chapter 5: Memory Arena Allocator
 * K&R §5.10, §8.7: Fast linear allocator for temporary data
 * Transpiled to safe Rust (using Vec and lifetime management)
 */

use std::cell::Cell;

const ARENA_SIZE: usize = 1024 * 1024; // 1 MB

struct Arena {
    memory: Vec<u8>,
    used: Cell<usize>,
}

impl Arena {
    fn new(size: usize) -> Self {
        println!("Arena created: {} bytes", size);
        Arena {
            memory: vec![0; size],
            used: Cell::new(0),
        }
    }

    fn alloc(&self, size: usize) -> Option<&mut [u8]> {
        // Align to 8 bytes
        let aligned_size = (size + 7) & !7;
        let current = self.used.get();

        if current + aligned_size > self.memory.len() {
            eprintln!("Arena out of memory");
            return None;
        }

        self.used.set(current + aligned_size);

        // SAFETY: We ensure non-overlapping slices through offset tracking
        unsafe {
            let ptr = self.memory.as_ptr().add(current) as *mut u8;
            Some(std::slice::from_raw_parts_mut(ptr, size))
        }
    }

    fn reset(&self) {
        let size = self.memory.len();
        self.used.set(0);
        println!("Arena reset: {} bytes reclaimed", size);
    }

    fn stats(&self) {
        let total = self.memory.len();
        let used = self.used.get();
        println!("Arena stats:");
        println!("  Total:     {} bytes", total);
        println!("  Used:      {} bytes", used);
        println!("  Available: {} bytes", total - used);
        println!("  Usage:     {:.1}%", (used as f64 * 100.0) / total as f64);
    }
}

// String pool example
struct StringPool<'a> {
    strings: Vec<&'a str>,
}

impl<'a> StringPool<'a> {
    fn new() -> Self {
        StringPool {
            strings: Vec::new(),
        }
    }

    fn add(&mut self, arena: &'a Arena, s: &str) -> &'a str {
        if let Some(buf) = arena.alloc(s.len()) {
            buf.copy_from_slice(s.as_bytes());
            // SAFETY: We just copied valid UTF-8 bytes
            let string = unsafe { std::str::from_utf8_unchecked(buf) };
            self.strings.push(string);
            string
        } else {
            panic!("Arena out of memory");
        }
    }

    fn print(&self) {
        println!("String pool ({} strings):", self.strings.len());
        for (i, s) in self.strings.iter().enumerate() {
            println!("  [{}] {}", i, s);
        }
    }
}

fn main() {
    println!("=== Memory Arena Allocator ===\n");

    let arena = Arena::new(ARENA_SIZE);
    println!();

    // String pool example
    println!("=== String Pool Example ===");
    let mut pool = StringPool::new();
    pool.add(&arena, "Hello");
    pool.add(&arena, "World");
    pool.add(&arena, "Arena");
    pool.add(&arena, "Allocator");
    pool.print();

    println!();
    arena.stats();

    // Temporary allocations
    println!("\n=== Temporary Data ===");
    let start_used = arena.used.get();

    if let Some(buffer) = arena.alloc(100) {
        buffer[0] = 42;
        println!("Allocated 100 bytes, wrote value: {}", buffer[0]);
    }

    if let Some(buffer) = arena.alloc(200) {
        println!("Allocated 200 bytes");
    }

    let used = arena.used.get() - start_used;
    println!("Temporary data used: {} bytes", used);
    println!("(No individual free() calls needed!)");

    println!();
    arena.stats();

    // Reset arena
    println!();
    arena.reset();
    arena.stats();

    println!("\nArena allocator benefits:");
    println!("  - O(1) allocation (pointer bump)");
    println!("  - No fragmentation");
    println!("  - Bulk deallocation (reset/destroy)");
    println!("  - Cache-friendly (linear memory)");
    println!("  - Perfect for temporary/scoped data");
}

// Idiomatic Rust: use typed_arena or bumpalo crate
#[allow(dead_code)]
fn demonstrate_typed_arena() {
    // In Cargo.toml: typed-arena = "2.0"
    // use typed_arena::Arena;
    //
    // let arena = Arena::new();
    // let num1 = arena.alloc(42);
    // let num2 = arena.alloc(100);
    //
    // println!("{} {}", num1, num2);
    //
    // // All allocations freed when arena is dropped
}

#[allow(dead_code)]
fn demonstrate_bumpalo() {
    // In Cargo.toml: bumpalo = "3.0"
    // use bumpalo::Bump;
    //
    // let bump = Bump::new();
    //
    // let nums = bump.alloc_slice_fill_default::<i32>(10);
    // nums[0] = 42;
    //
    // let string = bump.alloc_str("Hello");
    // println!("{}", string);
    //
    // // Reset and reuse
    // bump.reset();
}

// Key differences from C:
// 1. Vec<u8> instead of char*
// 2. Cell for interior mutability
// 3. Unsafe for raw pointer manipulation
// 4. Lifetimes ensure arena outlives allocations
// 5. RAII: automatic cleanup
// 6. Alignment with bitwise operations
// 7. For production: typed_arena or bumpalo crate
// 8. No manual free() needed
