/* K&R C Chapter 6: Structure Padding and Alignment
 * Understanding memory layout of structures
 * Transpiled to safe Rust (repr attributes)
 */

use std::mem::{size_of, align_of, offset_of};

// Unoptimized structure - poor alignment (C-compatible)
#[repr(C)]
struct Unoptimized {
    c1: u8,      // 1 byte
    i: i32,      // 4 bytes (4-byte aligned)
    c2: u8,      // 1 byte
    d: f64,      // 8 bytes (8-byte aligned)
    c3: u8,      // 1 byte
}

// Optimized structure - better alignment
#[repr(C)]
struct Optimized {
    d: f64,      // 8 bytes
    i: i32,      // 4 bytes
    c1: u8,      // 1 byte
    c2: u8,      // 1 byte
    c3: u8,      // 1 byte
    // Compiler adds padding for alignment
}

// Packed structure (may be less efficient)
#[repr(C, packed)]
struct Packed {
    c1: u8,
    i: i32,
    c2: u8,
    d: f64,
    c3: u8,
}

// Example for offset demonstration
#[repr(C)]
struct Example {
    a: u8,
    b: i32,
    c: f64,
    d: u8,
}

fn main() {
    println!("=== Structure Sizes ===");
    println!("sizeof(u8) = {}", size_of::<u8>());
    println!("sizeof(i32) = {}", size_of::<i32>());
    println!("sizeof(f64) = {}", size_of::<f64>());
    println!();

    // Unoptimized structure
    println!("Unoptimized structure:");
    let expected_min = size_of::<u8>() + size_of::<i32>() + size_of::<u8>() +
                       size_of::<f64>() + size_of::<u8>();
    println!("  Expected minimum: {} bytes (1+4+1+8+1)", expected_min);
    println!("  Actual size: {} bytes", size_of::<Unoptimized>());
    println!("  Padding: {} bytes",
             size_of::<Unoptimized>() - expected_min);

    // Optimized structure
    println!("\nOptimized structure:");
    let expected_opt = size_of::<f64>() + size_of::<i32>() + 3 * size_of::<u8>();
    println!("  Expected minimum: {} bytes (8+4+1+1+1)", expected_opt);
    println!("  Actual size: {} bytes", size_of::<Optimized>());
    println!("  Padding: {} bytes",
             size_of::<Optimized>() - expected_opt);

    // Packed structure
    println!("\nPacked structure:");
    println!("  Size: {} bytes (no padding)", size_of::<Packed>());

    // Member offsets
    println!("\n=== Member Offsets (offset_of!) ===");
    println!("struct Example:");
    println!("  offset_of!(a) = {}", offset_of!(Example, a));
    println!("  offset_of!(b) = {}", offset_of!(Example, b));
    println!("  offset_of!(c) = {}", offset_of!(Example, c));
    println!("  offset_of!(d) = {}", offset_of!(Example, d));
    println!("  sizeof(Example) = {}", size_of::<Example>());

    // Visualize memory layout
    let u = Unoptimized {
        c1: 0,
        i: 0,
        c2: 0,
        d: 0.0,
        c3: 0,
    };

    println!("\n=== Memory Layout (unoptimized) ===");
    println!("Base address: {:p}", &u);
    println!("  c1 at: {:p} (offset {})", &u.c1, offset_of!(Unoptimized, c1));
    println!("  i  at: {:p} (offset {})", &u.i, offset_of!(Unoptimized, i));
    println!("  c2 at: {:p} (offset {})", &u.c2, offset_of!(Unoptimized, c2));
    println!("  d  at: {:p} (offset {})", &u.d, offset_of!(Unoptimized, d));
    println!("  c3 at: {:p} (offset {})", &u.c3, offset_of!(Unoptimized, c3));

    // Alignment requirements
    println!("\n=== Alignment Requirements ===");
    println!("align_of(u8) = {}", align_of::<u8>());
    println!("align_of(i32) = {}", align_of::<i32>());
    println!("align_of(f64) = {}", align_of::<f64>());
    println!("align_of(Unoptimized) = {}", align_of::<Unoptimized>());
}

// Custom alignment
#[allow(dead_code)]
fn demonstrate_custom_alignment() {
    // Force specific alignment
    #[repr(C, align(16))]
    struct Aligned16 {
        value: i32,
    }

    #[repr(C, align(64))]
    struct Aligned64 {
        value: i32,
    }

    println!("Aligned16 size: {}, align: {}",
             size_of::<Aligned16>(), align_of::<Aligned16>());
    println!("Aligned64 size: {}, align: {}",
             size_of::<Aligned64>(), align_of::<Aligned64>());
}

// Key differences from C:
// 1. #[repr(C)] for C-compatible layout
// 2. #[repr(packed)] for no padding
// 3. #[repr(align(N))] for custom alignment
// 4. offset_of! macro (stable in Rust 1.77+)
// 5. size_of and align_of functions
// 6. No _Alignof keyword (use align_of)
// 7. Rust's default repr is not C-compatible
// 8. Packed structs require unsafe for unaligned access
