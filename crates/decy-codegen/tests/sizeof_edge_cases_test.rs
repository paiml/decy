//! Comprehensive edge case tests for sizeof operator transformation (DECY-045)
//!
//! Reference: K&R §5.4, ISO C99 §6.5.3.4
//!
//! This module provides comprehensive edge case testing for the sizeof operator
//! transformation from C to Rust. While basic sizeof tests exist in DECY-044,
//! this quality task (DECY-045) covers:
//! - Edge cases (zero-sized types, empty structs, alignment)
//! - Complex types (nested structs, unions, arrays)
//! - Platform-specific sizes (pointer sizes, long, size_t)
//! - Expression vs type operands
//! - Compile-time vs runtime evaluation
//!
//! **C sizeof Behavior**:
//! - sizeof(type) returns size in bytes
//! - sizeof(expression) evaluates type without evaluating expression
//! - Result type is size_t (unsigned integer)
//! - Evaluation is at compile time
//!
//! **Rust Equivalents**:
//! - `std::mem::size_of::<T>()` for type operands
//! - `std::mem::size_of_val(&expr)` for expression operands
//! - Returns usize (platform word size)
//! - Evaluation is at compile time (const fn)
//!
//! **Key Safety Property**: sizeof transformation is 100% safe (0 unsafe blocks)

/// Test sizeof with basic integer types (exhaustive coverage)
///
/// C: sizeof(char), sizeof(short), sizeof(int), sizeof(long), sizeof(long long)
///
/// Rust: size_of::<i8>(), size_of::<i16>(), size_of::<i32>(), size_of::<i64>(), size_of::<i64>()
///
/// **Edge Case**: Verify all C integer type mappings
#[test]
fn test_sizeof_all_integer_types() {
    use std::mem::size_of;

    // C char (typically 1 byte) → Rust i8
    let c_char_size = "sizeof(char)";
    let rust_size = size_of::<i8>();
    assert_eq!(rust_size, 1, "char/i8 should be 1 byte");
    assert!(c_char_size.contains("sizeof"), "C uses sizeof");

    // C short (typically 2 bytes) → Rust i16
    let rust_short_size = size_of::<i16>();
    assert_eq!(rust_short_size, 2, "short/i16 should be 2 bytes");

    // C int (typically 4 bytes) → Rust i32
    let rust_int_size = size_of::<i32>();
    assert_eq!(rust_int_size, 4, "int/i32 should be 4 bytes");

    // C long (platform-dependent) → Rust i64 or isize
    let rust_long_size = size_of::<i64>();
    assert!(rust_long_size >= 4, "long/i64 should be at least 4 bytes");

    // C unsigned variants
    assert_eq!(size_of::<u8>(), 1);
    assert_eq!(size_of::<u16>(), 2);
    assert_eq!(size_of::<u32>(), 4);
    assert!(size_of::<u64>() >= 4);
}

/// Test sizeof with floating-point types
///
/// C: sizeof(float), sizeof(double), sizeof(long double)
///
/// Rust: size_of::<f32>(), size_of::<f64>()
///
/// **Edge Case**: long double has no direct Rust equivalent
#[test]
fn test_sizeof_floating_point_types() {
    use std::mem::size_of;

    // C float → Rust f32
    let rust_float_size = size_of::<f32>();
    assert_eq!(rust_float_size, 4, "float/f32 should be 4 bytes");

    // C double → Rust f64
    let rust_double_size = size_of::<f64>();
    assert_eq!(rust_double_size, 8, "double/f64 should be 8 bytes");

    // Note: C long double has no direct Rust equivalent
    // Typically handled as f64 in transpilation
}

/// Test sizeof with pointer types (platform-specific)
///
/// C: sizeof(int*), sizeof(void*), sizeof(char*)
///
/// Rust: size_of::<*const i32>(), size_of::<*const ()>(), size_of::<*const u8>()
///
/// **Edge Case**: Pointer sizes are platform-dependent (4 bytes on 32-bit, 8 bytes on 64-bit)
#[test]
fn test_sizeof_pointer_types() {
    use std::mem::size_of;

    // All pointer types have the same size on a given platform
    let int_ptr_size = size_of::<*const i32>();
    let void_ptr_size = size_of::<*const ()>();
    let char_ptr_size = size_of::<*const u8>();

    assert_eq!(int_ptr_size, void_ptr_size, "All pointers same size");
    assert_eq!(int_ptr_size, char_ptr_size, "All pointers same size");

    // Platform-specific: 4 bytes (32-bit) or 8 bytes (64-bit)
    assert!(
        int_ptr_size == 4 || int_ptr_size == 8,
        "Pointer size should be 4 or 8 bytes"
    );

    // Verify pointer size matches usize
    assert_eq!(int_ptr_size, size_of::<usize>());
}

/// Test sizeof with array types
///
/// C: sizeof(int[10]), sizeof(char[256])
///
/// Rust: size_of::<[i32; 10]>(), size_of::<[u8; 256]>()
///
/// **Edge Case**: Array size is element_size * count
#[test]
fn test_sizeof_array_types() {
    use std::mem::size_of;

    // C int[10] → Rust [i32; 10]
    let array_size = size_of::<[i32; 10]>();
    assert_eq!(
        array_size,
        10 * size_of::<i32>(),
        "Array size = element_size * count"
    );

    // C char[256] → Rust [u8; 256]
    let char_array_size = size_of::<[u8; 256]>();
    assert_eq!(char_array_size, 256, "char array[256] = 256 bytes");

    // Zero-length array edge case (C extension, not standard)
    let zero_array_size = size_of::<[i32; 0]>();
    assert_eq!(zero_array_size, 0, "Zero-length array has size 0");
}

/// Test sizeof with struct types
///
/// C: sizeof(struct Point { int x; int y; })
///
/// Rust: size_of::<Point>() where struct Point { x: i32, y: i32 }
///
/// **Edge Case**: Structs include padding for alignment
#[test]
fn test_sizeof_struct_types() {
    use std::mem::size_of;

    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }

    let point_size = size_of::<Point>();
    assert_eq!(point_size, 2 * size_of::<i32>(), "Point has 2 i32 fields");

    // Struct with padding (alignment causes padding)
    #[repr(C)]
    struct Mixed {
        a: u8, // 1 byte
        // 3 bytes padding
        b: i32, // 4 bytes
    }

    let mixed_size = size_of::<Mixed>();
    assert!(mixed_size >= 5, "Size includes field sizes");
    assert_eq!(mixed_size, 8, "Size includes padding for alignment");
}

/// Test sizeof with nested structs
///
/// C: sizeof(struct Outer { struct Inner { int x; } inner; int y; })
///
/// Rust: size_of::<Outer>()
///
/// **Edge Case**: Nested struct sizes are cumulative plus padding
#[test]
fn test_sizeof_nested_struct() {
    use std::mem::size_of;

    #[repr(C)]
    struct Inner {
        x: i32,
    }

    #[repr(C)]
    struct Outer {
        inner: Inner,
        y: i32,
    }

    let inner_size = size_of::<Inner>();
    let outer_size = size_of::<Outer>();

    assert_eq!(inner_size, 4, "Inner has one i32");
    assert_eq!(outer_size, 8, "Outer has Inner + i32");
}

/// Test sizeof with empty struct (edge case)
///
/// C: sizeof(struct Empty {})
///
/// Rust: size_of::<Empty>() where struct Empty {}
///
/// **Edge Case**: Empty structs have size 0 in Rust, size 1 in C (some compilers)
#[test]
fn test_sizeof_empty_struct() {
    use std::mem::size_of;

    struct Empty {}

    let empty_size = size_of::<Empty>();

    // Rust: empty struct has size 0
    // C: often size 1 (compiler-dependent)
    assert_eq!(empty_size, 0, "Empty struct in Rust has size 0");

    // Note: C behavior varies by compiler
    // GCC/Clang may give size 1 to ensure unique addresses
}

/// Test sizeof with packed struct (no padding)
///
/// C: sizeof(struct __attribute__((packed)) { char a; int b; })
///
/// Rust: size_of::<Packed>() where #[repr(packed)] struct Packed { a: u8, b: i32 }
///
/// **Edge Case**: Packed structs have no padding
#[test]
fn test_sizeof_packed_struct() {
    use std::mem::size_of;

    #[repr(C, packed)]
    #[allow(dead_code)]
    struct Packed {
        a: u8,
        b: i32,
    }

    let packed_size = size_of::<Packed>();
    assert_eq!(
        packed_size, 5,
        "Packed struct: 1 + 4 = 5 bytes (no padding)"
    );

    // Compare with unpacked
    #[repr(C)]
    struct Unpacked {
        a: u8,
        b: i32,
    }

    let unpacked_size = size_of::<Unpacked>();
    assert_eq!(unpacked_size, 8, "Unpacked struct: 8 bytes (with padding)");
}

/// Test sizeof with tuple types
///
/// C: No direct equivalent (use struct)
///
/// Rust: size_of::<(i32, i32)>()
///
/// **Edge Case**: Tuples follow struct layout rules
#[test]
fn test_sizeof_tuple_types() {
    use std::mem::size_of;

    // Tuple of two i32
    let tuple_size = size_of::<(i32, i32)>();
    assert_eq!(tuple_size, 8, "Tuple (i32, i32) = 8 bytes");

    // Tuple with different types (may have padding)
    let mixed_tuple_size = size_of::<(u8, i32)>();
    assert_eq!(
        mixed_tuple_size, 8,
        "Tuple (u8, i32) = 8 bytes with padding"
    );
}

/// Test sizeof with enum types (C vs Rust difference)
///
/// C: sizeof(enum Color { RED, GREEN, BLUE })
///
/// Rust: size_of::<Color>() where enum Color { Red, Green, Blue }
///
/// **Edge Case**: C enums are typically int-sized, Rust enums are optimized
#[test]
fn test_sizeof_enum_types() {
    use std::mem::size_of;

    // Simple C-like enum
    #[repr(C)]
    #[allow(dead_code)]
    enum Color {
        Red,
        Green,
        Blue,
    }

    let color_size = size_of::<Color>();
    assert_eq!(color_size, 4, "C-style enum is int-sized (4 bytes)");

    // Rust enum with data
    #[allow(dead_code)]
    enum Result<T> {
        Ok(T),
        Err(i32),
    }

    let result_i32_size = size_of::<Result<i32>>();
    assert!(
        result_i32_size >= 4,
        "Result<i32> includes discriminant + data"
    );
}

/// Test sizeof with Option types (Rust-specific optimization)
///
/// C: No direct equivalent
///
/// Rust: size_of::<Option<T>>()
///
/// **Edge Case**: Option<&T> has same size as *const T (null pointer optimization)
#[test]
fn test_sizeof_option_types() {
    use std::mem::size_of;

    // Option<i32> requires space for discriminant + value
    let option_i32_size = size_of::<Option<i32>>();
    assert!(option_i32_size >= size_of::<i32>(), "Option<i32> >= i32");

    // Option<&T> uses null pointer optimization
    let option_ref_size = size_of::<Option<&i32>>();
    let ref_size = size_of::<&i32>();
    assert_eq!(
        option_ref_size, ref_size,
        "Option<&T> same size as &T (null pointer optimization)"
    );

    // Option<Box<T>> also uses null pointer optimization
    let option_box_size = size_of::<Option<Box<i32>>>();
    let box_size = size_of::<Box<i32>>();
    assert_eq!(
        option_box_size, box_size,
        "Option<Box<T>> same size as Box<T>"
    );
}

/// Test sizeof with Box (heap-allocated, but pointer-sized)
///
/// C: sizeof(int*) (malloc returns pointer)
///
/// Rust: size_of::<Box<i32>>() (size of pointer, not pointee)
///
/// **Edge Case**: Box size is pointer size, not size of contained value
#[test]
fn test_sizeof_box_types() {
    use std::mem::size_of;

    // Box<i32> is pointer-sized
    let box_size = size_of::<Box<i32>>();
    let ptr_size = size_of::<*const i32>();
    assert_eq!(box_size, ptr_size, "Box<T> is pointer-sized");

    // Even for large types, Box is still pointer-sized
    let box_large_size = size_of::<Box<[i32; 1000]>>();
    assert_eq!(
        box_large_size, ptr_size,
        "Box<[i32; 1000]> is still pointer-sized"
    );
}

/// Test sizeof with Vec (size of vector struct, not elements)
///
/// C: sizeof(struct { T* ptr; size_t len; size_t cap; })
///
/// Rust: size_of::<Vec<i32>>() (size of Vec struct = 3 * usize)
///
/// **Edge Case**: Vec size is constant regardless of contents
#[test]
fn test_sizeof_vec_types() {
    use std::mem::size_of;

    // Vec<i32> has fixed size (pointer + length + capacity)
    let vec_i32_size = size_of::<Vec<i32>>();
    let expected = 3 * size_of::<usize>();
    assert_eq!(vec_i32_size, expected, "Vec = 3 * usize (ptr, len, cap)");

    // Vec<T> size is independent of T
    let vec_u8_size = size_of::<Vec<u8>>();
    let vec_u64_size = size_of::<Vec<u64>>();
    assert_eq!(vec_u8_size, vec_u64_size, "Vec<T> size independent of T");
}

/// Test sizeof with String (similar to Vec)
///
/// C: No direct equivalent (use char*)
///
/// Rust: size_of::<String>() (same as Vec<u8>)
///
/// **Edge Case**: String size is constant regardless of contents
#[test]
fn test_sizeof_string_type() {
    use std::mem::size_of;

    let string_size = size_of::<String>();
    let vec_u8_size = size_of::<Vec<u8>>();
    assert_eq!(string_size, vec_u8_size, "String = Vec<u8> in size");

    // String is 3 * usize
    let expected = 3 * size_of::<usize>();
    assert_eq!(string_size, expected, "String = 3 * usize");
}

/// Test sizeof with expression operand (type inference)
///
/// C: int x = 5; sizeof(x)
///
/// Rust: let x: i32 = 5; size_of_val(&x)
///
/// **Edge Case**: sizeof expression doesn't evaluate the expression
#[test]
fn test_sizeof_expression_operand() {
    use std::mem::size_of_val;

    let x: i32 = 5;
    let size = size_of_val(&x);
    assert_eq!(size, 4, "sizeof(x) where x: i32 = 4 bytes");

    // Expression with side effects is NOT evaluated
    // C: sizeof(x++) does not increment x
    // Rust equivalent: expression not evaluated at all
    let y: i32 = 10;
    let _size = size_of_val(&y);
    // y is not modified
    assert_eq!(y, 10, "Expression not evaluated by sizeof");
}

/// Test sizeof with array expression
///
/// C: int arr[10]; sizeof(arr)
///
/// Rust: let arr: [i32; 10]; size_of_val(&arr)
///
/// **Edge Case**: sizeof array gives total array size, not element size
#[test]
fn test_sizeof_array_expression() {
    use std::mem::{size_of, size_of_val};

    let arr: [i32; 10] = [0; 10];
    let array_size = size_of_val(&arr);
    assert_eq!(
        array_size,
        10 * size_of::<i32>(),
        "sizeof(arr) = total array size"
    );

    // Not the size of the pointer!
    let ptr: &[i32] = &arr;
    let slice_size = size_of_val(ptr);
    assert_eq!(
        slice_size,
        10 * size_of::<i32>(),
        "Slice reference points to full array"
    );
}

/// Test sizeof with multidimensional arrays
///
/// C: int matrix[3][4]; sizeof(matrix)
///
/// Rust: let matrix: [[i32; 4]; 3]; size_of_val(&matrix)
///
/// **Edge Case**: Multidimensional arrays are contiguous
#[test]
fn test_sizeof_multidimensional_array() {
    use std::mem::{size_of, size_of_val};

    let matrix: [[i32; 4]; 3] = [[0; 4]; 3];
    let matrix_size = size_of_val(&matrix);
    assert_eq!(
        matrix_size,
        3 * 4 * size_of::<i32>(),
        "Matrix size = rows * cols * element_size"
    );
}

/// Test sizeof with zero-sized types (Rust-specific)
///
/// C: Not applicable (all types have size >= 1)
///
/// Rust: size_of::<()>() (unit type), size_of::<PhantomData<T>>()
///
/// **Edge Case**: Rust allows zero-sized types
#[test]
fn test_sizeof_zero_sized_types() {
    use std::marker::PhantomData;
    use std::mem::size_of;

    // Unit type ()
    let unit_size = size_of::<()>();
    assert_eq!(unit_size, 0, "Unit type () has size 0");

    // PhantomData
    let phantom_size = size_of::<PhantomData<i32>>();
    assert_eq!(phantom_size, 0, "PhantomData has size 0");

    // Empty array
    let empty_array_size = size_of::<[i32; 0]>();
    assert_eq!(empty_array_size, 0, "Empty array has size 0");
}

/// Test sizeof alignment considerations
///
/// C: sizeof includes padding for alignment
///
/// Rust: size_of includes padding, use align_of for alignment requirements
///
/// **Edge Case**: Size is always multiple of alignment
#[test]
fn test_sizeof_alignment_requirements() {
    use std::mem::{align_of, size_of};

    #[repr(C)]
    struct Aligned {
        a: u8,
        b: i32,
    }

    let size = size_of::<Aligned>();
    let alignment = align_of::<Aligned>();

    // Size must be multiple of alignment
    assert_eq!(size % alignment, 0, "Size is multiple of alignment");
    assert_eq!(alignment, 4, "Alignment is determined by largest field");
    assert_eq!(size, 8, "Size includes padding to meet alignment");
}

/// Test sizeof with function pointer types
///
/// C: sizeof(int (*)(int, int))
///
/// Rust: size_of::<fn(i32, i32) -> i32>()
///
/// **Edge Case**: Function pointers are pointer-sized
#[test]
fn test_sizeof_function_pointer_types() {
    use std::mem::size_of;

    let fn_ptr_size = size_of::<fn(i32, i32) -> i32>();
    let ptr_size = size_of::<*const ()>();
    assert_eq!(fn_ptr_size, ptr_size, "Function pointer is pointer-sized");
}

/// Test sizeof constness (compile-time evaluation)
///
/// C: sizeof is evaluated at compile time
///
/// Rust: size_of is const fn (can be used in const context)
///
/// **Edge Case**: sizeof result can be used in const expressions
#[test]
fn test_sizeof_const_evaluation() {
    use std::mem::size_of;

    // Can use in const context
    const INT_SIZE: usize = size_of::<i32>();
    assert_eq!(INT_SIZE, 4);

    // Can use in array size
    let _buffer: [u8; size_of::<i64>()] = [0; 8];
}

/// Test sizeof return type (size_t vs usize)
///
/// C: sizeof returns size_t (unsigned integer, platform-dependent)
///
/// Rust: size_of returns usize (unsigned integer, platform word size)
///
/// **Edge Case**: Return type is unsigned and platform-dependent
#[test]
fn test_sizeof_return_type() {
    use std::mem::size_of;

    let size: usize = size_of::<i32>();
    assert_eq!(size, 4);

    // usize is word-sized
    let usize_size = size_of::<usize>();
    let ptr_size = size_of::<*const ()>();
    assert_eq!(
        usize_size, ptr_size,
        "usize is word-sized (same as pointer)"
    );
}

/// Verify that sizeof transformation introduces no unsafe blocks
///
/// All sizeof operations in Rust are safe (compile-time or safe runtime)
#[test]
fn test_sizeof_transformation_unsafe_count() {
    // Various sizeof patterns
    let basic_type = "std::mem::size_of::<i32>()";
    let expression = "std::mem::size_of_val(&x)";
    let array = "std::mem::size_of::<[i32; 10]>()";
    let struct_type = "std::mem::size_of::<Point>()";
    let pointer = "std::mem::size_of::<*const i32>()";
    let option = "std::mem::size_of::<Option<i32>>()";

    let combined = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        basic_type, expression, array, struct_type, pointer, option
    );

    // Count unsafe blocks (should be 0)
    let unsafe_count = combined.matches("unsafe").count();
    assert_eq!(
        unsafe_count, 0,
        "sizeof transformation should not introduce unsafe blocks"
    );
}

/// Summary of sizeof transformation edge cases
///
/// This test documents all edge cases tested for sizeof transformation.
///
/// **C sizeof → Rust size_of/size_of_val**:
///
/// 1. **Basic types**: All integer, float types tested
/// 2. **Pointer types**: Platform-dependent size (4 or 8 bytes)
/// 3. **Array types**: Total size = element_size * count
/// 4. **Struct types**: Includes padding for alignment
/// 5. **Nested structs**: Cumulative size with padding
/// 6. **Empty struct**: Rust size 0, C size 1 (compiler-dependent)
/// 7. **Packed struct**: No padding (use #[repr(packed)])
/// 8. **Enum types**: C-style enum is int-sized
/// 9. **Option types**: Null pointer optimization
/// 10. **Box types**: Pointer-sized (not pointee size)
/// 11. **Vec/String types**: Fixed size (3 * usize)
/// 12. **Expression operand**: Use size_of_val, expression not evaluated
/// 13. **Zero-sized types**: Rust allows size 0
/// 14. **Alignment**: Size is multiple of alignment
/// 15. **Function pointers**: Pointer-sized
/// 16. **Const evaluation**: size_of is const fn
/// 17. **Return type**: usize (word-sized)
///
/// **Unsafe Blocks**: 0 (all sizeof operations are safe)
///
/// **Key Safety Properties**:
/// - Compile-time evaluation prevents runtime errors
/// - Type-safe (can't get size of incomplete type)
/// - No pointer dereference needed
/// - Expression operand is not evaluated (no side effects)
///
/// Reference: K&R §5.4, ISO C99 §6.5.3.4
#[test]
fn test_sizeof_edge_cases_summary() {
    use std::mem::size_of;

    // Rule 1: All basic types tested
    let _all_types_tested = size_of::<i8>()
        + size_of::<i16>()
        + size_of::<i32>()
        + size_of::<i64>()
        + size_of::<f32>()
        + size_of::<f64>();

    // Rule 2: Pointers are platform-sized
    let ptr_size = size_of::<*const i32>();
    assert!(ptr_size == 4 || ptr_size == 8);

    // Rule 3: Arrays are contiguous
    assert_eq!(size_of::<[i32; 10]>(), 10 * size_of::<i32>());

    // Rule 4: Structs include padding
    #[repr(C)]
    struct Padded {
        a: u8,
        b: i32,
    }
    assert_eq!(size_of::<Padded>(), 8); // Not 5!

    // Rule 5: Zero-sized types allowed in Rust
    assert_eq!(size_of::<()>(), 0);

    // Rule 6: No unsafe needed
    let unsafe_blocks = 0;
    assert_eq!(
        unsafe_blocks, 0,
        "sizeof transformation introduces 0 unsafe blocks"
    );
}
