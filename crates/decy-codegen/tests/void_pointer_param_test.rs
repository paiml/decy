//! DECY-168: Tests for void pointer parameter handling
//!
//! C's `void*` parameters should generate `*mut ()` in Rust (raw pointers),
//! NOT `&mut ()` (mutable references to unit type).
//!
//! This is critical for stdlib stubs like realloc, memcpy, etc. which take void*.

use decy_codegen::CodeGenerator;
use decy_hir::{HirFunction, HirParameter, HirType};

/// Create a code generator
fn create_generator() -> CodeGenerator {
    CodeGenerator::new()
}

#[test]
fn test_void_pointer_param_stays_raw_pointer() {
    // C code: void* realloc(void* ptr, size_t size);
    // Expected Rust: fn realloc(ptr: *mut (), size: usize) -> *mut ()
    // NOT: fn realloc(ptr: &mut (), size: usize) -> *mut ()
    //
    // void* has no pattern analysis, so it must stay as raw pointer

    let gen = create_generator();

    let func = HirFunction::new_with_body(
        "realloc".to_string(),
        HirType::Pointer(Box::new(HirType::Void)), // Returns void*
        vec![
            HirParameter::new(
                "ptr".to_string(),
                HirType::Pointer(Box::new(HirType::Void)), // void* ptr
            ),
            HirParameter::new(
                "size".to_string(),
                HirType::UnsignedInt, // size_t
            ),
        ],
        vec![], // Empty body (stub)
    );

    let code = gen.generate_function(&func);

    println!("Generated: {}", code);

    // The void* parameter should be *mut (), not &mut ()
    assert!(
        code.contains("ptr: *mut ()"),
        "void* param should become *mut (), got: {}",
        code
    );
    assert!(
        !code.contains("ptr: &mut ()"),
        "void* param should NOT become &mut (), got: {}",
        code
    );
}

#[test]
fn test_memcpy_void_pointers() {
    // C code: void* memcpy(void* dest, const void* src, size_t n);
    // Expected Rust: fn memcpy(dest: *mut (), src: *mut (), n: usize) -> *mut ()
    // (const void* also becomes *mut () for simplicity in stubs)

    let gen = create_generator();

    let func = HirFunction::new_with_body(
        "memcpy".to_string(),
        HirType::Pointer(Box::new(HirType::Void)),
        vec![
            HirParameter::new(
                "dest".to_string(),
                HirType::Pointer(Box::new(HirType::Void)),
            ),
            HirParameter::new("src".to_string(), HirType::Pointer(Box::new(HirType::Void))),
            HirParameter::new("n".to_string(), HirType::UnsignedInt),
        ],
        vec![],
    );

    let code = gen.generate_function(&func);

    println!("Generated: {}", code);

    // Both void* params should be *mut ()
    assert!(
        code.contains("dest: *mut ()"),
        "void* dest should become *mut (), got: {}",
        code
    );
    // Note: src may be different due to naming, but should NOT be a reference
    assert!(
        !code.contains(": &mut ()") && !code.contains(": &()"),
        "void* params should NOT become references, got: {}",
        code
    );
}

#[test]
fn test_free_void_pointer() {
    // C code: void free(void* ptr);
    // Expected Rust: fn free(ptr: *mut ())

    let gen = create_generator();

    let func = HirFunction::new_with_body(
        "free".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Void)),
        )],
        vec![],
    );

    let code = gen.generate_function(&func);

    println!("Generated: {}", code);

    assert!(
        code.contains("ptr: *mut ()"),
        "void* param should become *mut (), got: {}",
        code
    );
    assert!(
        !code.contains("ptr: &mut ()"),
        "void* param should NOT become &mut (), got: {}",
        code
    );
}
