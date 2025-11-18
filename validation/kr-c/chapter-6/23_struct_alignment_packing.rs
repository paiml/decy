/* K&R C Chapter 6: Structure Alignment and Packing
 * K&R §6.9: Memory layout and alignment
 * Transpiled to safe Rust (using repr attributes)
 */

use std::mem::{size_of, align_of};

// Natural alignment - compiler adds padding
#[repr(C)]
struct Natural {
    c: u8,      // 1 byte
    // 3 bytes padding
    i: i32,     // 4 bytes
    s: i16,     // 2 bytes
    // 2 bytes padding
}

// Packed structure - no padding
#[repr(C, packed)]
struct Packed {
    c: u8,      // 1 byte
    i: i32,     // 4 bytes
    s: i16,     // 2 bytes
}

// Reordered for better packing
#[repr(C)]
struct Reordered {
    i: i32,     // 4 bytes
    s: i16,     // 2 bytes
    c: u8,      // 1 byte
    // 1 byte padding
}

// Example with multiple alignment requirements
#[repr(C)]
struct AlignmentDemo {
    c1: u8,     // 1 byte
    // 7 bytes padding
    d: f64,     // 8 bytes - requires 8-byte alignment
    c2: u8,     // 1 byte
    // 3 bytes padding
    i: i32,     // 4 bytes
}

fn show_offsets() {
    println!("=== Structure Member Offsets ===");

    println!("Natural structure:");
    println!("  offset(c): {}", offset_of!(Natural, c));
    println!("  offset(i): {}", offset_of!(Natural, i));
    println!("  offset(s): {}", offset_of!(Natural, s));
    println!("  Total size:  {} bytes\n", size_of::<Natural>());

    println!("Packed structure:");
    println!("  offset(c): {}", offset_of!(Packed, c));
    println!("  offset(i): {}", offset_of!(Packed, i));
    println!("  offset(s): {}", offset_of!(Packed, s));
    println!("  Total size:  {} bytes\n", size_of::<Packed>());

    println!("Reordered structure:");
    println!("  offset(i): {}", offset_of!(Reordered, i));
    println!("  offset(s): {}", offset_of!(Reordered, s));
    println!("  offset(c): {}", offset_of!(Reordered, c));
    println!("  Total size:  {} bytes\n", size_of::<Reordered>());
}

fn show_padding() {
    println!("=== Structure Padding ===");

    let natural_members = size_of::<u8>() + size_of::<i32>() + size_of::<i16>();
    let natural_padding = size_of::<Natural>() - natural_members;

    println!("Natural structure:");
    println!("  Members:     {} bytes", natural_members);
    println!("  Padding:     {} bytes", natural_padding);
    println!("  Total:       {} bytes", size_of::<Natural>());
    println!("  Efficiency:  {:.1}%\n",
             (natural_members as f64 * 100.0) / size_of::<Natural>() as f64);

    let packed_members = size_of::<u8>() + size_of::<i32>() + size_of::<i16>();
    println!("Packed structure:");
    println!("  Members:     {} bytes", packed_members);
    println!("  Padding:     0 bytes");
    println!("  Total:       {} bytes", size_of::<Packed>());
    println!("  Efficiency:  100%\n");

    let reordered_members = size_of::<i32>() + size_of::<i16>() + size_of::<u8>();
    let reordered_padding = size_of::<Reordered>() - reordered_members;

    println!("Reordered structure:");
    println!("  Members:     {} bytes", reordered_members);
    println!("  Padding:     {} bytes", reordered_padding);
    println!("  Total:       {} bytes", size_of::<Reordered>());
    println!("  Efficiency:  {:.1}%\n",
             (reordered_members as f64 * 100.0) / size_of::<Reordered>() as f64);
}

fn show_alignment() {
    println!("=== Alignment Requirements ===");

    println!("Type alignments:");
    println!("  u8:     {} bytes", align_of::<u8>());
    println!("  i16:    {} bytes", align_of::<i16>());
    println!("  i32:    {} bytes", align_of::<i32>());
    println!("  i64:    {} bytes", align_of::<i64>());
    println!("  f32:    {} bytes", align_of::<f32>());
    println!("  f64:    {} bytes", align_of::<f64>());
    println!("  *const: {} bytes\n", align_of::<*const ()>());

    println!("Structure alignments:");
    println!("  Natural:       {} bytes", align_of::<Natural>());
    println!("  Packed:        {} bytes", align_of::<Packed>());
    println!("  Reordered:     {} bytes", align_of::<Reordered>());
    println!("  AlignmentDemo: {} bytes\n", align_of::<AlignmentDemo>());
}

fn array_memory_layout() {
    println!("=== Array Memory Layout ===");

    let arr = [Natural { c: 0, i: 0, s: 0 }; 3];

    println!("Natural array[3]:");
    println!("  Element 0: {:p}", &arr[0]);
    println!("  Element 1: {:p} (offset: {} bytes)",
             &arr[1],
             (&arr[1] as *const _ as usize) - (&arr[0] as *const _ as usize));
    println!("  Element 2: {:p} (offset: {} bytes)",
             &arr[2],
             (&arr[2] as *const _ as usize) - (&arr[0] as *const _ as usize));
    println!("  Total array size: {} bytes\n", size_of::<[Natural; 3]>());
}

// Cache line alignment
#[repr(C, align(64))]
struct CacheAligned {
    data: i32,
}

fn cache_alignment() {
    println!("=== Cache Line Alignment ===");

    println!("CacheAligned structure:");
    println!("  Size:      {} bytes", size_of::<CacheAligned>());
    println!("  Alignment: {} bytes\n", align_of::<CacheAligned>());

    println!("Aligned to 64 bytes (typical cache line size)");
    println!("Reduces false sharing in multithreaded code\n");
}

fn main() {
    println!("=== Structure Alignment and Packing ===\n");

    show_offsets();
    show_padding();
    show_alignment();
    array_memory_layout();
    cache_alignment();

    println!("Key takeaways:");
    println!("  - Compilers add padding for alignment");
    println!("  - Reorder members to reduce padding");
    println!("  - Use #[repr(C, packed)] sparingly");
    println!("  - Alignment affects performance");
    println!("  - Cache line alignment prevents false sharing");
    println!("  - Use offset_of! to inspect layout");
}

// Key differences from C:
// 1. #[repr(C)] for C-compatible layout
// 2. #[repr(packed)] for no padding
// 3. #[repr(align(N))] for custom alignment
// 4. offset_of! macro (stable in Rust 1.77+)
// 5. size_of/align_of functions
// 6. No _Alignof keyword
// 7. Rust's default repr optimizes layout
// 8. Unsafe required for packed struct field access
