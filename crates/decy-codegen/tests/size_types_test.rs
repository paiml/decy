//! DECY-167: Tests for platform size type typedef generation
//!
//! C's `size_t` should map to Rust's `usize`, not `u32`.
//! C's `ssize_t` should map to Rust's `isize`, not `i32`.
//! C's `ptrdiff_t` should map to Rust's `isize`, not `i32`.
//!
//! This ensures proper interoperability with Rust methods like `.len()`
//! which return `usize`.

use decy_codegen::CodeGenerator;
use decy_hir::{HirType, HirTypedef};

/// Create a code generator
fn create_generator() -> CodeGenerator {
    CodeGenerator::new()
}

#[test]
fn test_size_t_maps_to_usize() {
    // C code: typedef unsigned long size_t;
    // Expected Rust: pub type size_t = usize;
    // NOT: pub type size_t = u32;
    //
    // This ensures .len() (which returns usize) can be assigned to size_t variables.

    let gen = create_generator();

    // size_t is typically defined as unsigned long/unsigned int
    let typedef = HirTypedef::new("size_t".to_string(), HirType::UnsignedInt);
    let code = gen.generate_typedef(&typedef).unwrap();

    println!("Generated: {}", code);

    // Should use usize, not u32
    assert!(
        code.contains("usize"),
        "size_t should map to usize, got: {}",
        code
    );
    assert!(
        !code.contains("u32"),
        "size_t should NOT map to u32, got: {}",
        code
    );
}

#[test]
fn test_ssize_t_maps_to_isize() {
    // C code: typedef long ssize_t;
    // Expected Rust: pub type ssize_t = isize;
    // NOT: pub type ssize_t = i32;

    let gen = create_generator();

    // ssize_t is typically defined as long/int (signed)
    let typedef = HirTypedef::new("ssize_t".to_string(), HirType::Int);
    let code = gen.generate_typedef(&typedef).unwrap();

    println!("Generated: {}", code);

    // Should use isize, not i32
    assert!(
        code.contains("isize"),
        "ssize_t should map to isize, got: {}",
        code
    );
    assert!(
        !code.contains("i32"),
        "ssize_t should NOT map to i32, got: {}",
        code
    );
}

#[test]
fn test_ptrdiff_t_maps_to_isize() {
    // C code: typedef long ptrdiff_t;
    // Expected Rust: pub type ptrdiff_t = isize;
    // NOT: pub type ptrdiff_t = i32;

    let gen = create_generator();

    // ptrdiff_t is typically defined as long/int (signed)
    let typedef = HirTypedef::new("ptrdiff_t".to_string(), HirType::Int);
    let code = gen.generate_typedef(&typedef).unwrap();

    println!("Generated: {}", code);

    // Should use isize, not i32
    assert!(
        code.contains("isize"),
        "ptrdiff_t should map to isize, got: {}",
        code
    );
    assert!(
        !code.contains("i32"),
        "ptrdiff_t should NOT map to i32, got: {}",
        code
    );
}

#[test]
fn test_regular_typedef_unchanged() {
    // C code: typedef int MyInt;
    // Expected Rust: pub type MyInt = i32;
    // Regular typedefs should still work as before

    let gen = create_generator();

    let typedef = HirTypedef::new("MyInt".to_string(), HirType::Int);
    let code = gen.generate_typedef(&typedef).unwrap();

    println!("Generated: {}", code);

    assert!(
        code.contains("pub type MyInt = i32"),
        "Regular typedef should work, got: {}",
        code
    );
}
