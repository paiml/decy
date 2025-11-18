/* K&R C Chapter 3: Type Conversions
 * K&R §3.2: Implicit and explicit type conversions
 * Tests type casting and promotion rules
 * Transpiled to safe Rust
 */

fn demo_explicit_casts() {
    println!("=== Explicit Type Casts ===");

    // Integer promotion (automatic in C, explicit in Rust)
    let c: i8 = 100;
    let s: i16 = 200;
    let result = c as i32 + s as i32;
    println!("i8(100) + i16(200) = i32({})", result);

    // Float/double (Rust uses f32 and f64)
    let f: f32 = 3.14;
    let d = f as f64 * 2.0;
    println!("f32(3.14) * f64(2.0) = f64({:.2})", d);

    // Mixed arithmetic
    let i = 5;
    let f2: f32 = 2.5;
    let result2 = i as f32 * f2;
    println!("i32(5) * f32(2.5) = f32({:.2})", result2);

    println!();
}

fn demo_type_truncation() {
    println!("=== Explicit Type Casts ===");

    let d = 3.7;
    let i = d as i32;  // Truncation
    println!("3.7 as i32 = {}", i);

    let num = 7;
    let den = 2;
    let ratio = num as f64 / den as f64;
    println!("7 as f64 / 2 as f64 = {:.2}", ratio);

    // Unsigned to signed
    let u = u32::MAX;
    let signed_u = u as i32;
    println!("u32::MAX as i32 = {}", signed_u);

    println!();
}

fn demo_overflow() {
    println!("=== Integer Overflow ===");

    let mut c: i8 = 127;
    println!("i8: {}", c);
    c = c.wrapping_add(1);  // Explicit wrapping in Rust
    println!("After wrapping_add(1): {} (overflow)", c);

    let mut uc: u8 = 255;
    println!("\nu8: {}", uc);
    uc = uc.wrapping_add(1);
    println!("After wrapping_add(1): {} (wraps to 0)", uc);

    println!();
}

fn demo_byte_representation() {
    println!("=== Byte Representation ===");

    let arr = [1i32, 2, 3, 4];

    // View as bytes using safe slice conversion
    let bytes = unsafe {
        std::slice::from_raw_parts(
            arr.as_ptr() as *const u8,
            std::mem::size_of_val(&arr)
        )
    };

    print!("i32 array as bytes: ");
    for &byte in bytes {
        print!("{:02X} ", byte);
    }
    println!("\n");
}

fn main() {
    println!("=== Type Conversions ===\n");

    demo_explicit_casts();
    demo_type_truncation();
    demo_overflow();
    demo_byte_representation();

    println!("Conversion rules:");
    println!("  - Rust requires explicit casts with 'as'");
    println!("  - No implicit integer widening");
    println!("  - Use wrapping_* for defined overflow");
    println!("  - Truncation from float to int");
    println!("  - Checked/saturating variants available");
}

// Key differences from C:
// 1. All casts explicit with 'as' keyword
// 2. No implicit promotions
// 3. Overflow: checked/wrapping/saturating
// 4. as f32/f64 instead of automatic promotion
// 5. Compiler warns about lossy conversions
