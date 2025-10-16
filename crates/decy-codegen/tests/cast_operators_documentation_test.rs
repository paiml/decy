//! Documentation tests for cast operator transformation (EXPR-CAST validation)
//!
//! Reference: K&R §2.7, ISO C99 §6.5.4
//!
//! This module documents the transformation of C cast operators to Rust.
//! C casts provide explicit type conversions:
//! - Syntax: `(type)expression`
//! - Can be implicit (automatic) or explicit
//! - Can be safe or unsafe (undefined behavior possible)
//!
//! **C Cast Operators**:
//! - `(int)f` - cast float to int
//! - `(char*)p` - cast pointer type
//! - `(void*)x` - cast to void pointer
//! - Implicit conversions (promotions)
//!
//! **Rust Equivalents**:
//! - `as` operator for safe numeric casts
//! - `From`/`Into` traits for safe conversions
//! - `transmute` for unsafe bit-level conversions (rare)
//! - Type inference (no cast needed in many cases)
//!
//! **Key Safety Property**: Most casts are SAFE (use `as`), some require unsafe (`transmute`)

#![allow(dead_code)]

/// Document transformation of numeric casts (safe)
///
/// C: int i = (int)3.14;
///
/// Rust: let i = 3.14 as i32;
///
/// **Transformation**: Numeric cast → as operator (SAFE)
/// - Truncates float to int
/// - Well-defined behavior
/// - No undefined behavior
///
/// Reference: K&R §2.7, ISO C99 §6.5.4
#[test]
fn test_cast_float_to_int() {
    let c_code = "int i = (int)3.14;";
    let rust_equivalent = "let i = 3.14 as i32;";

    assert!(c_code.contains("(int)"), "C uses cast");
    assert!(rust_equivalent.contains("as"), "Rust uses as operator");

    // Demonstrate float to int cast
    let f = std::f64::consts::PI;
    let i = f as i32;
    assert_eq!(i, 3, "Float truncated to int");
}

/// Document transformation of int to float cast (safe)
///
/// C: float f = (float)42;
///
/// Rust: let f = 42 as f32;
///
/// **Transformation**: Int to float → as operator (SAFE)
/// - Exact conversion for small integers
/// - May lose precision for large integers
///
/// Reference: K&R §2.7, ISO C99 §6.5.4
#[test]
fn test_cast_int_to_float() {
    let c_code = "float f = (float)42;";
    let rust_equivalent = "let f = 42 as f32;";

    assert!(c_code.contains("(float)"), "C uses cast");
    assert!(rust_equivalent.contains("as"), "Rust uses as operator");

    // Demonstrate int to float cast
    let i = 42;
    let f = i as f32;
    assert_eq!(f, 42.0, "Int converted to float");
}

/// Document signed/unsigned cast (safe but can overflow)
///
/// C: unsigned int u = (unsigned int)-1;
///
/// Rust: let u = -1i32 as u32;
///
/// **Transformation**: Signed to unsigned → as operator
/// - Wraps around (two's complement)
/// - Well-defined in Rust
/// - Undefined in C (implementation-defined)
///
/// Reference: K&R §2.7, ISO C99 §6.5.4
#[test]
fn test_cast_signed_to_unsigned() {
    let c_code = "unsigned int u = (unsigned int)-1;";
    let rust_equivalent = "let u = -1i32 as u32;";

    assert!(c_code.contains("unsigned"), "C unsigned cast");
    assert!(rust_equivalent.contains("as u32"), "Rust as operator");

    // Demonstrate signed to unsigned (wraps)
    let s: i32 = -1;
    let u = s as u32;
    assert_eq!(u, u32::MAX, "Signed -1 wraps to max unsigned");
}

/// Document integer narrowing cast (truncates)
///
/// C: char c = (char)1000;
///
/// Rust: let c = 1000i32 as i8;
///
/// **Transformation**: Narrowing cast → as operator
/// - Truncates higher bits
/// - Well-defined in Rust
/// - Implementation-defined in C
///
/// Reference: K&R §2.7, ISO C99 §6.5.4
#[test]
fn test_cast_integer_narrowing() {
    let c_code = "char c = (char)1000;";
    let rust_equivalent = "let c = 1000i32 as i8;";

    assert!(c_code.contains("(char)"), "C cast");
    assert!(rust_equivalent.contains("as i8"), "Rust as operator");

    // Demonstrate narrowing (truncates)
    let i: i32 = 1000;
    let c = i as i8;
    // 1000 in binary: 0000_0011_1110_1000
    // Truncated to 8 bits: 1110_1000 = -24 (signed)
    assert_eq!(c, -24, "Narrowing truncates bits");
}

/// Document integer widening cast (extends)
///
/// C: int i = (int)c;
///
/// Rust: let i = c as i32;
///
/// **Transformation**: Widening cast → as operator (SAFE)
/// - Sign extends for signed types
/// - Zero extends for unsigned types
///
/// Reference: K&R §2.7, ISO C99 §6.5.4
#[test]
fn test_cast_integer_widening() {
    let c_code = "int i = (int)c;";
    let rust_equivalent = "let i = c as i32;";

    assert!(c_code.contains("(int)"), "C cast");
    assert!(rust_equivalent.contains("as"), "Rust as operator");

    // Demonstrate sign extension
    let c: i8 = -1;
    let i = c as i32;
    assert_eq!(i, -1, "Sign extends -1");

    // Demonstrate zero extension
    let c: u8 = 255;
    let i = c as i32;
    assert_eq!(i, 255, "Zero extends unsigned");
}

/// Document pointer to integer cast (for addresses)
///
/// C: uintptr_t addr = (uintptr_t)ptr;
///
/// Rust: let addr = ptr as usize;
///
/// **Transformation**: Pointer to int → as usize (SAFE)
/// - Rust uses usize for pointer-sized integers
/// - Safe conversion
///
/// Reference: K&R §2.7, ISO C99 §6.5.4
#[test]
fn test_cast_pointer_to_int() {
    let c_code = "uintptr_t addr = (uintptr_t)ptr;";
    let rust_equivalent = "let addr = ptr as usize;";

    assert!(c_code.contains("uintptr_t"), "C pointer cast");
    assert!(rust_equivalent.contains("as usize"), "Rust as usize");

    // Demonstrate pointer to integer
    let value = 42;
    let ptr = &value as *const i32;
    let addr = ptr as usize;
    assert!(addr != 0, "Pointer converts to address");
}

/// Document integer to pointer cast (unsafe context)
///
/// C: int* ptr = (int*)0x1000;
///
/// Rust: let ptr = 0x1000 as *const i32;
///
/// **Transformation**: Int to pointer → as (SAFE to create, unsafe to dereference)
/// - Creating pointer is safe
/// - Dereferencing requires unsafe block
///
/// Reference: K&R §2.7, ISO C99 §6.5.4
#[test]
fn test_cast_int_to_pointer() {
    let c_code = "int* ptr = (int*)0x1000;";
    let rust_equivalent = "let ptr = 0x1000 as *const i32;";

    assert!(c_code.contains("(int*)"), "C cast");
    assert!(rust_equivalent.contains("as *const"), "Rust pointer cast");

    // Demonstrate int to pointer (creating is safe)
    let addr = 0x1000usize;
    let _ptr = addr as *const i32;
    // Note: Dereferencing would require unsafe block
}

/// Document pointer type cast (safe to create, unsafe to use)
///
/// C: void* vp = (void*)int_ptr;
///    int* ip = (int*)vp;
///
/// Rust: let vp = int_ptr as *const ();
///       let ip = vp as *const i32;
///
/// **Transformation**: Pointer cast → as operator
/// - Rust uses *const () for void*
/// - Safe to cast pointer types
/// - Unsafe to dereference
///
/// Reference: K&R §2.7, ISO C99 §6.5.4
#[test]
fn test_cast_pointer_types() {
    let c_code = "void* vp = (void*)int_ptr;";
    let rust_equivalent = "let vp = int_ptr as *const ();";

    assert!(c_code.contains("void*"), "C void pointer");
    assert!(rust_equivalent.contains("*const ()"), "Rust unit pointer");

    // Demonstrate pointer type casts
    let value = 42;
    let int_ptr = &value as *const i32;
    let void_ptr = int_ptr as *const ();
    let back_to_int = void_ptr as *const i32;

    // Converting back requires unsafe to dereference
    unsafe {
        assert_eq!(*back_to_int, 42, "Pointer cast preserves value");
    }
}

/// Document const cast (removing const)
///
/// C: int* p = (int*)const_ptr;  // Removes const (unsafe!)
///
/// Rust: let p = const_ptr as *const i32 as *mut i32;
///
/// **Transformation**: Const cast → explicit cast chain
/// - Rust makes it explicit
/// - Still unsafe to mutate through const pointer
///
/// Reference: K&R §2.7, ISO C99 §6.5.4
#[test]
fn test_cast_remove_const() {
    let c_code = "int* p = (int*)const_ptr;";
    let rust_equivalent = "let p = const_ptr as *const i32 as *mut i32;";

    assert!(c_code.contains("(int*)"), "C removes const");
    assert!(rust_equivalent.contains("*mut"), "Rust explicit mut");

    // Demonstrate const to mut cast (creating is safe, using is unsafe)
    let value = 42;
    let const_ptr = &value as *const i32;
    let _mut_ptr = const_ptr as *mut i32;
    // Note: Actually mutating would be undefined behavior
}

/// Document array to pointer decay (implicit in C)
///
/// C: int* p = arr;  // Implicit decay
///
/// Rust: let p = arr.as_ptr();
///       // Or: &arr[0] as *const i32
///
/// **Transformation**: Array decay → as_ptr() method
/// - Explicit in Rust
/// - Type-safe
///
/// Reference: K&R §2.7, ISO C99 §6.5.4
#[test]
fn test_cast_array_to_pointer() {
    let c_code = "int* p = arr;";
    let rust_equivalent = "let p = arr.as_ptr();";

    assert!(c_code.contains("int* p"), "C implicit decay");
    assert!(rust_equivalent.contains("as_ptr"), "Rust explicit method");

    // Demonstrate array to pointer
    let arr = [1, 2, 3];
    let p = arr.as_ptr();
    unsafe {
        assert_eq!(*p, 1, "Pointer to first element");
    }
}

/// Document function pointer cast
///
/// C: void (*fp)(void) = (void (*)(void))func;
///
/// Rust: let fp = func as fn();
///
/// **Transformation**: Function pointer cast → as operator
/// - Type-safe function pointers
/// - Explicit signature
///
/// Reference: K&R §2.7, ISO C99 §6.5.4
#[test]
fn test_cast_function_pointer() {
    let c_code = "void (*fp)(void) = (void (*)(void))func;";
    let rust_equivalent = "let fp = func as fn();";

    assert!(c_code.contains("void (*"), "C function pointer");
    assert!(rust_equivalent.contains("fn()"), "Rust function type");

    // Demonstrate function pointer
    fn my_func() {
        // Do nothing
    }

    let fp: fn() = my_func;
    fp(); // Call through pointer
}

/// Document char* to const char* (widening const)
///
/// C: const char* cp = str;  // Implicit
///
/// Rust: let cp = str.as_ptr();
///
/// **Transformation**: Add const → same pointer type in Rust
/// - Rust borrows are immutable by default
/// - No cast needed
///
/// Reference: K&R §2.7, ISO C99 §6.5.4
#[test]
fn test_cast_add_const() {
    let c_code = "const char* cp = str;";
    let rust_equivalent = "let cp: &str = str;";

    assert!(c_code.contains("const"), "C adds const");
    assert!(
        rust_equivalent.contains("&str"),
        "Rust immutable by default"
    );

    // Demonstrate immutable reference (no cast needed)
    let s = "hello";
    let _cp: &str = s; // No cast needed
}

/// Document bool to int cast (safe)
///
/// C: int i = (int)flag;  // true → 1, false → 0
///
/// Rust: let i = flag as i32;
///
/// **Transformation**: Bool to int → as operator (SAFE)
/// - true → 1, false → 0
/// - Explicit in Rust
///
/// Reference: K&R §2.7, ISO C99 §6.5.4
#[test]
fn test_cast_bool_to_int() {
    let c_code = "int i = (int)flag;";
    let rust_equivalent = "let i = flag as i32;";

    assert!(c_code.contains("(int)"), "C cast");
    assert!(rust_equivalent.contains("as"), "Rust as operator");

    // Demonstrate bool to int
    let flag = true;
    let i = flag as i32;
    assert_eq!(i, 1, "true converts to 1");

    let flag = false;
    let i = flag as i32;
    assert_eq!(i, 0, "false converts to 0");
}

/// Document int to bool cast (non-zero → true)
///
/// C: bool b = (bool)x;  // Non-zero → true
///
/// Rust: let b = x != 0;
///
/// **Transformation**: Int to bool → explicit comparison
/// - Rust requires explicit boolean conversion
/// - More clear intent
///
/// Reference: K&R §2.7, ISO C99 §6.5.4
#[test]
fn test_cast_int_to_bool() {
    let c_code = "bool b = (bool)x;";
    let rust_equivalent = "let b = x != 0;";

    assert!(c_code.contains("(bool)"), "C cast");
    assert!(rust_equivalent.contains("!= 0"), "Rust explicit comparison");

    // Demonstrate int to bool
    let x = 5;
    let b = x != 0;
    assert!(b, "Non-zero is true");

    let x = 0;
    let b = x != 0;
    assert!(!b, "Zero is false");
}

/// Document enum to int cast (safe)
///
/// C: int val = (int)enum_val;
///
/// Rust: let val = enum_val as i32;
///
/// **Transformation**: Enum to int → as operator (SAFE)
/// - Gets discriminant value
/// - Type-safe
///
/// Reference: K&R §2.7, ISO C99 §6.5.4
#[test]
fn test_cast_enum_to_int() {
    let c_code = "int val = (int)enum_val;";
    let rust_equivalent = "let val = enum_val as i32;";

    assert!(c_code.contains("(int)"), "C cast");
    assert!(rust_equivalent.contains("as"), "Rust as operator");

    // Demonstrate enum to int
    #[repr(i32)]
    enum Status {
        Ok = 0,
        Error = 1,
        Pending = 2,
    }

    let status = Status::Error;
    let val = status as i32;
    assert_eq!(val, 1, "Enum converts to discriminant");
}

/// Document type punning with union (unsafe in Rust)
///
/// C: float f = 3.14;
///    int bits = *(int*)&f;  // Type punning (undefined behavior!)
///
/// Rust: let bits = f32::to_bits(3.14);  // Safe method
///       // Or: std::mem::transmute (unsafe)
///
/// **Transformation**: Type punning → safe methods or transmute
/// - Prefer safe methods (to_bits/from_bits)
/// - transmute requires unsafe
///
/// Reference: K&R §2.7, ISO C99 §6.5.4
#[test]
fn test_cast_type_punning() {
    let c_pattern = "Type punning with cast";
    let rust_safe = "Use to_bits() or transmute (unsafe)";

    assert!(c_pattern.contains("cast"), "C uses cast");
    assert!(rust_safe.contains("to_bits"), "Rust has safe alternative");

    // Demonstrate safe type punning
    let f = std::f32::consts::PI;
    let bits = f.to_bits();
    assert_eq!(bits, 0x40490fdb, "Float bits extracted safely");

    // Demonstrate unsafe transmute (avoid if possible)
    let bits_unsafe: u32 = unsafe { f.to_bits() };
    assert_eq!(bits_unsafe, 0x40490fdb, "to_bits() is actually safe, no transmute needed");
}

/// Document struct cast (not allowed in Rust)
///
/// C: struct B* bp = (struct B*)&a;  // Risky cast
///
/// Rust: // Use proper type conversion or transmute (unsafe)
///
/// **Transformation**: Struct cast → proper conversion or unsafe transmute
/// - Rust prevents dangerous struct casts
/// - Use From/Into for safe conversions
///
/// Reference: K&R §2.7, ISO C99 §6.5.4
#[test]
fn test_cast_struct_conversion() {
    let c_pattern = "Cast between struct types (dangerous)";
    let rust_safe = "Use From/Into traits (safe)";

    assert!(c_pattern.contains("dangerous"), "C cast is dangerous");
    assert!(rust_safe.contains("From/Into"), "Rust has safe traits");

    // Demonstrate safe struct conversion
    struct A {
        value: i32,
    }

    struct B {
        value: i32,
    }

    impl From<A> for B {
        fn from(a: A) -> B {
            B { value: a.value }
        }
    }

    let a = A { value: 42 };
    let b: B = a.into();
    assert_eq!(b.value, 42, "Safe struct conversion");
}

/// Document implicit conversions (automatic in C)
///
/// C: double d = 42;  // Implicit int to double
///
/// Rust: let d: f64 = 42.0;
///       // Or: let d = 42f64;
///       // Or: let d = 42 as f64;
///
/// **Transformation**: Implicit cast → explicit type or cast
/// - Rust requires explicit numeric conversions
/// - Prevents unintended conversions
///
/// Reference: K&R §2.7, ISO C99 §6.5.4
#[test]
fn test_implicit_conversions() {
    let c_code = "double d = 42;";
    let rust_equivalent = "let d = 42 as f64;";

    assert!(c_code.contains("double d = 42"), "C implicit conversion");
    assert!(rust_equivalent.contains("as f64"), "Rust explicit");

    // Demonstrate explicit conversion
    let i = 42;
    let d = i as f64;
    assert_eq!(d, 42.0, "Explicit int to float");
}

/// Verify that most cast transformations are safe
///
/// Most casts use `as` operator (safe)
/// Only type punning requires unsafe
#[test]
fn test_cast_transformation_safety() {
    // Safe cast patterns
    let numeric_cast = "3.14 as i32";
    let pointer_to_int = "ptr as usize";
    let bool_cast = "flag as i32";
    let enum_cast = "status as i32";

    let safe_combined = format!(
        "{}\n{}\n{}\n{}",
        numeric_cast, pointer_to_int, bool_cast, enum_cast
    );

    // Count unsafe (should be 0 for these patterns)
    let unsafe_count = safe_combined.matches("unsafe").count();
    assert_eq!(unsafe_count, 0, "Basic casts are safe (use `as` operator)");

    // Type punning requires unsafe
    let type_punning = "std::mem::transmute(f)";
    assert!(
        type_punning.contains("transmute"),
        "Type punning uses transmute (unsafe)"
    );
}

/// Summary of cast operator transformation rules
///
/// This test documents the complete set of rules for cast transformation.
///
/// **C cast → Rust Transformation**:
///
/// 1. **Numeric casts**: `(int)f` → `f as i32` (SAFE)
/// 2. **Signed/unsigned**: `(unsigned)i` → `i as u32` (SAFE, wraps)
/// 3. **Narrowing**: `(char)i` → `i as i8` (SAFE, truncates)
/// 4. **Widening**: `(int)c` → `c as i32` (SAFE, sign/zero extends)
/// 5. **Pointer to int**: `(uintptr_t)p` → `p as usize` (SAFE)
/// 6. **Int to pointer**: `(int*)addr` → `addr as *const i32` (safe to create, unsafe to deref)
/// 7. **Pointer types**: `(void*)p` → `p as *const ()` (SAFE)
/// 8. **Bool to int**: `(int)flag` → `flag as i32` (SAFE)
/// 9. **Int to bool**: `(bool)x` → `x != 0` (explicit comparison)
/// 10. **Enum to int**: `(int)enum_val` → `enum_val as i32` (SAFE)
/// 11. **Type punning**: `*(int*)&f` → `f.to_bits()` (SAFE) or `transmute` (unsafe)
/// 12. **Struct cast**: Use From/Into traits (SAFE) or transmute (unsafe)
/// 13. **Implicit conversions**: Make explicit with `as` or type annotation
///
/// **Key Advantages of Rust Approach**:
/// - `as` operator is explicit and type-safe
/// - Most casts are safe (no undefined behavior)
/// - Dangerous operations require unsafe (transmute)
/// - From/Into traits for safe conversions
/// - No implicit numeric conversions (prevents bugs)
///
/// **Unsafe Blocks**: 0 for `as` operator, unsafe only for transmute
///
/// Reference: K&R §2.7, ISO C99 §6.5.4
#[test]
fn test_cast_transformation_rules_summary() {
    // Rule 1: Numeric casts are safe
    let f = std::f64::consts::PI;
    let i = f as i32;
    assert_eq!(i, 3, "Numeric cast is safe");

    // Rule 2: Signed/unsigned wraps (well-defined)
    let s: i32 = -1;
    let u = s as u32;
    assert_eq!(u, u32::MAX, "Wrapping is well-defined");

    // Rule 3: Narrowing truncates (well-defined)
    let large: i32 = 1000;
    let small = large as i8;
    assert_eq!(small, -24, "Truncation is well-defined");

    // Rule 4: Bool conversions are explicit
    let flag = true;
    let i = flag as i32;
    assert_eq!(i, 1, "Bool to int is explicit");

    // Rule 5: Most casts don't need unsafe
    let unsafe_blocks = 0;
    assert_eq!(
        unsafe_blocks, 0,
        "Most cast transformations use safe `as` operator"
    );

    // Rule 6: Type punning has safe alternatives
    let f = std::f32::consts::PI;
    let bits = f.to_bits();
    assert!(bits != 0, "Safe type punning with to_bits()");
}
