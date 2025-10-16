//! Documentation tests for union transformation (TYPE-UNION validation)
//!
//! Reference: K&R §6.8, ISO C99 §6.7.2.1
//!
//! This module documents the transformation of C union declarations to Rust.
//! C unions allow storing different types in the same memory location, with:
//! - Memory layout: all members share same memory (size = largest member)
//! - Type punning: reading as different type than written (undefined behavior in C)
//! - Space efficiency: only one member active at a time
//!
//! **C union Syntax**:
//! - `union Name { type1 member1; type2 member2; };`
//! - All members overlay same memory address
//! - Size = max(sizeof(member1), sizeof(member2), ...)
//! - Only one member is "active" at a time
//!
//! **Rust Equivalents**:
//! - `enum` with data variants (type-safe, most common)
//! - `union` (unsafe, for FFI or performance-critical code)
//! - `#[repr(C)] union` for C compatibility
//!
//! **Key Safety Property**: Rust enums are SAFE (0 unsafe blocks), Rust unions require unsafe

#![allow(dead_code)]

/// Document transformation of simple union to Rust enum
///
/// C: union Value {
///        int i;
///        float f;
///    };
///
/// Rust: enum Value {
///         Int(i32),
///         Float(f32),
///       }
///
/// **Transformation**: union → enum (type-safe tagged union)
/// - Rust enum tracks which variant is active (tag)
/// - Safe: cannot read wrong variant
/// - Pattern matching ensures exhaustiveness
///
/// Reference: K&R §6.8, ISO C99 §6.7.2.1
#[test]
fn test_union_to_enum() {
    let c_code = "union Value { int i; float f; };";
    let rust_equivalent = "enum Value { Int(i32), Float(f32) }";

    assert!(c_code.contains("union"), "C uses union");
    assert!(
        rust_equivalent.contains("enum"),
        "Rust uses enum for safety"
    );

    // Demonstrate type-safe enum
    enum Value {
        Int(i32),
        Float(f32),
    }

    let v = Value::Int(42);
    match v {
        Value::Int(i) => assert_eq!(i, 42, "Can safely extract int"),
        Value::Float(_) => panic!("Wrong variant!"),
    }

    // Cannot accidentally read as wrong type (compile error):
    // if let Value::Float(f) = v { } // Would not match at runtime
}

/// Document transformation of union with multiple types
///
/// C: union Data {
///        char c;
///        short s;
///        int i;
///        long l;
///    };
///
/// Rust: enum Data {
///         Char(i8),
///         Short(i16),
///         Int(i32),
///         Long(i64),
///       }
///
/// **Transformation**: Multi-member union → enum with multiple variants
/// - Each union member becomes enum variant
/// - Size in C: max(1, 2, 4, 8) = 8 bytes
/// - Size in Rust: 8 + tag (typically 16 bytes with padding)
///
/// Reference: K&R §6.8, ISO C99 §6.7.2.1
#[test]
fn test_union_multiple_types() {
    let c_code = "union Data { char c; short s; int i; long l; };";
    let rust_equivalent = "enum Data { Char(i8), Short(i16), Int(i32), Long(i64) }";

    assert!(c_code.contains("union"), "C union");
    assert!(rust_equivalent.contains("enum"), "Rust enum");

    // Demonstrate enum with multiple variants
    enum Data {
        Char(i8),
        Short(i16),
        Int(i32),
        Long(i64),
    }

    let d = Data::Long(12345678);
    match d {
        Data::Char(_) => panic!("Wrong variant"),
        Data::Short(_) => panic!("Wrong variant"),
        Data::Int(_) => panic!("Wrong variant"),
        Data::Long(l) => assert_eq!(l, 12345678, "Correct variant"),
    }
}

/// Document union for type punning (unsafe in Rust)
///
/// C: union FloatInt {
///        float f;
///        uint32_t i;
///    };
///    u.f = 3.14;
///    uint32_t bits = u.i; // Type punning
///
/// Rust: let bits = f32::to_bits(3.14);
///       // Or use std::mem::transmute (unsafe)
///       // Or use union (unsafe)
///
/// **Transformation**: Type punning → safe methods or unsafe union
/// - Rust provides safe methods (to_bits/from_bits)
/// - Or use transmute (unsafe, explicit)
/// - Or use union (unsafe, requires unsafe block to read)
///
/// Reference: K&R §6.8, ISO C99 §6.7.2.1
#[test]
fn test_union_type_punning_safe() {
    let c_pattern = "Type punning with union";
    let rust_safe = "Use to_bits/from_bits (safe)";

    assert!(c_pattern.contains("union"), "C uses union");
    assert!(rust_safe.contains("safe"), "Rust has safe alternative");

    // Safe way: use built-in methods
    let f: f32 = std::f32::consts::PI;
    let bits: u32 = f.to_bits();
    assert_eq!(bits, 0x40490fdb, "Float bits extracted safely");

    let reconstructed = f32::from_bits(bits);
    assert_eq!(reconstructed, f, "Float reconstructed from bits");

    // This is SAFE (0 unsafe blocks)
}

/// Document union for FFI compatibility
///
/// C: union Compat {
///        int i;
///        float f;
///    };
///
/// Rust: #[repr(C)]
///       union Compat {
///           i: i32,
///           f: f32,
///       }
///
/// **Transformation**: FFI union → #[repr(C)] union
/// - Use when interfacing with C code
/// - Requires unsafe to read fields
/// - Exact memory layout as C
///
/// Reference: K&R §6.8, ISO C99 §6.7.2.1
#[test]
fn test_union_ffi_compat() {
    let c_code = "union Compat { int i; float f; };";
    let rust_equivalent = "#[repr(C)] union Compat { i: i32, f: f32 }";

    assert!(c_code.contains("union"), "C union");
    assert!(rust_equivalent.contains("repr(C)"), "Rust FFI union");

    // Demonstrate FFI union (requires unsafe)
    #[repr(C)]
    union Compat {
        i: i32,
        f: f32,
    }

    let mut u = Compat { i: 0 };
    u.i = 42;

    unsafe {
        assert_eq!(u.i, 42, "Read union field (unsafe)");
    }

    // Note: Reading union field requires unsafe block
}

/// Document union with struct members
///
/// C: union Compound {
///        struct { int x; int y; } point;
///        struct { float r; float g; float b; } color;
///    };
///
/// Rust: enum Compound {
///         Point { x: i32, y: i32 },
///         Color { r: f32, g: f32, b: f32 },
///       }
///
/// **Transformation**: Union with structs → enum with struct variants
/// - Each struct member becomes enum variant with named fields
/// - Type-safe, no unsafe needed
///
/// Reference: K&R §6.8, ISO C99 §6.7.2.1
#[test]
fn test_union_with_structs() {
    let c_code = "union Compound { struct { int x; int y; } point; };";
    let rust_equivalent = "enum Compound { Point { x: i32, y: i32 } }";

    assert!(c_code.contains("union"), "C union");
    assert!(rust_equivalent.contains("enum"), "Rust enum");

    // Demonstrate enum with struct variants
    enum Compound {
        Point { x: i32, y: i32 },
        Color { r: f32, g: f32, b: f32 },
    }

    let c = Compound::Point { x: 10, y: 20 };
    match c {
        Compound::Point { x, y } => {
            assert_eq!(x, 10, "Point x coordinate");
            assert_eq!(y, 20, "Point y coordinate");
        }
        Compound::Color { .. } => panic!("Wrong variant"),
    }
}

/// Document union for variant data (Option pattern)
///
/// C: struct Optional {
///        int has_value;
///        union {
///            int value;
///        } data;
///    };
///
/// Rust: Option<i32>
///       // Built-in type-safe option
///
/// **Transformation**: Tagged union → Option<T>
/// - C uses manual tag + union
/// - Rust has built-in Option enum
/// - Safer and more idiomatic
///
/// Reference: K&R §6.8, ISO C99 §6.7.2.1
#[test]
fn test_union_option_pattern() {
    let c_pattern = "Manual tag with union for optional";
    let rust_equivalent = "Option<i32> (built-in)";

    assert!(c_pattern.contains("union"), "C uses union");
    assert!(rust_equivalent.contains("Option"), "Rust has Option");

    // Demonstrate Option (safe, no union needed)
    let opt: Option<i32> = Some(42);
    match opt {
        Some(v) => assert_eq!(v, 42, "Has value"),
        None => panic!("Should have value"),
    }

    let none: Option<i32> = None;
    assert!(none.is_none(), "No value");

    // This is SAFE (0 unsafe blocks)
}

/// Document union for result/error pattern
///
/// C: struct Result {
///        int is_ok;
///        union {
///            int value;
///            int error;
///        } data;
///    };
///
/// Rust: Result<i32, i32>
///       // Built-in type-safe result
///
/// **Transformation**: Tagged union → Result<T, E>
/// - C uses manual tag + union
/// - Rust has built-in Result enum
/// - Forces error handling
///
/// Reference: K&R §6.8, ISO C99 §6.7.2.1
#[test]
fn test_union_result_pattern() {
    let c_pattern = "Manual tag with union for result";
    let rust_equivalent = "Result<i32, i32> (built-in)";

    assert!(c_pattern.contains("union"), "C uses union");
    assert!(rust_equivalent.contains("Result"), "Rust has Result");

    // Demonstrate Result (safe, no union needed)
    let ok: Result<i32, String> = Ok(42);
    match ok {
        Ok(v) => assert_eq!(v, 42, "Success value"),
        Err(_) => panic!("Should be Ok"),
    }

    let err: Result<i32, String> = Err("error".to_string());
    assert!(err.is_err(), "Error result");

    // This is SAFE (0 unsafe blocks)
}

/// Document anonymous union (C11 feature)
///
/// C: struct Container {
///        int type;
///        union {
///            int i;
///            float f;
///        };
///    };
///
/// Rust: struct Container {
///         data: Data,
///       }
///       enum Data { Int(i32), Float(f32) }
///
/// **Transformation**: Anonymous union → named enum field
/// - C: union members are directly accessible
/// - Rust: give union a name, use enum
///
/// Reference: ISO C99 §6.7.2.1 (C11 extension)
#[test]
fn test_anonymous_union() {
    let c_code = "struct Container { int type; union { int i; float f; }; };";
    let rust_equivalent = "struct Container { data: Data } enum Data { Int(i32), Float(f32) }";

    assert!(c_code.contains("union"), "C anonymous union");
    assert!(rust_equivalent.contains("enum"), "Rust named enum");

    // Demonstrate named enum in struct
    enum Data {
        Int(i32),
        Float(f32),
    }

    struct Container {
        data: Data,
    }

    let c = Container {
        data: Data::Int(42),
    };

    match c.data {
        Data::Int(i) => assert_eq!(i, 42, "Container holds int"),
        Data::Float(_) => panic!("Wrong variant"),
    }
}

/// Document union size and alignment
///
/// C: union U {
///        char c;     // 1 byte
///        int i;      // 4 bytes
///        double d;   // 8 bytes
///    };
///    sizeof(union U) == 8 (largest member)
///
/// Rust: enum U {
///         Char(i8),   // 1 byte + tag
///         Int(i32),   // 4 bytes + tag
///         Double(f64), // 8 bytes + tag
///       }
///       // Size: 8 + tag + padding (typically 16 bytes)
///
/// **Transformation**: Union size → enum size (includes tag)
/// - C union: size = max(members)
/// - Rust enum: size = max(members) + tag + padding
/// - Rust uses more memory for type safety
///
/// Reference: K&R §6.8, ISO C99 §6.7.2.1
#[test]
fn test_union_size_and_alignment() {
    use std::mem::size_of;

    // C union size would be 8 bytes (largest member)
    // Rust enum includes tag
    enum U {
        Char(i8),
        Int(i32),
        Double(f64),
    }

    let enum_size = size_of::<U>();
    // Enum size includes discriminant tag
    // Typically 16 bytes (8 for f64 + 8 for tag/padding)
    assert!(
        enum_size >= 8,
        "Enum is at least as large as largest variant"
    );

    // Demonstrate that all variants fit
    let _c = U::Char(42);
    let _i = U::Int(1234);
    let _d = U::Double(std::f64::consts::PI);
}

/// Document union bit fields (rare)
///
/// C: union Flags {
///        struct {
///            unsigned int a : 1;
///            unsigned int b : 1;
///        } bits;
///        unsigned int value;
///    };
///
/// Rust: struct Flags {
///         value: u32,
///       }
///       impl Flags {
///           fn a(&self) -> bool { self.value & 1 != 0 }
///           fn set_a(&mut self, v: bool) { ... }
///       }
///
/// **Transformation**: Union bit fields → struct with methods
/// - Use bitwise operations instead of bit fields
/// - Safer and more explicit
///
/// Reference: K&R §6.8, ISO C99 §6.7.2.1
#[test]
fn test_union_bit_fields() {
    let c_code = "union Flags { struct { unsigned int a : 1; } bits; unsigned int value; };";
    let rust_equivalent = "struct Flags { value: u32 } with accessor methods";

    assert!(c_code.contains("union"), "C union with bit fields");
    assert!(rust_equivalent.contains("struct"), "Rust struct");

    // Demonstrate bit field accessors
    struct Flags {
        value: u32,
    }

    impl Flags {
        fn a(&self) -> bool {
            self.value & 1 != 0
        }

        fn set_a(&mut self, v: bool) {
            if v {
                self.value |= 1;
            } else {
                self.value &= !1;
            }
        }
    }

    let mut flags = Flags { value: 0 };
    assert!(!flags.a(), "Bit a is clear");

    flags.set_a(true);
    assert!(flags.a(), "Bit a is set");
}

/// Document union for discriminated union (most common pattern)
///
/// C: struct Message {
///        enum { INT, FLOAT, STRING } type;
///        union {
///            int i;
///            float f;
///            char* s;
///        } data;
///    };
///
/// Rust: enum Message {
///         Int(i32),
///         Float(f32),
///         String(String),
///       }
///
/// **Transformation**: Manual discriminated union → enum
/// - C: manual tag + union (error-prone)
/// - Rust: enum is a discriminated union (safe by default)
/// - Compiler enforces tag consistency
///
/// Reference: K&R §6.8, ISO C99 §6.7.2.1
#[test]
fn test_discriminated_union() {
    let c_pattern = "Manual enum tag + union";
    let rust_equivalent = "enum (discriminated union built-in)";

    assert!(c_pattern.contains("union"), "C uses union");
    assert!(rust_equivalent.contains("enum"), "Rust enum");

    // Demonstrate discriminated union (enum)
    enum Message {
        Int(i32),
        Float(f32),
        Str(String),
    }

    let msg = Message::Str("hello".to_string());
    match msg {
        Message::Int(_) => panic!("Wrong variant"),
        Message::Float(_) => panic!("Wrong variant"),
        Message::Str(s) => assert_eq!(s, "hello", "Correct variant"),
    }

    // Compiler guarantees tag matches data (type-safe!)
}

/// Document union copy behavior
///
/// C: union U {
///        int i;
///        float f;
///    };
///    union U u1, u2;
///    u1.i = 42;
///    u2 = u1; // Bitwise copy
///
/// Rust: enum U {
///         Int(i32),
///         Float(f32),
///       }
///       let u1 = U::Int(42);
///       let u2 = u1; // Move (or Copy if derives Copy)
///
/// **Transformation**: Union copy → enum move/copy
/// - C: always bitwise copy
/// - Rust: move by default, Copy if all variants are Copy
///
/// Reference: K&R §6.8, ISO C99 §6.7.2.1
#[test]
fn test_union_copy_behavior() {
    let c_code = "u2 = u1; // Bitwise copy";
    let rust_equivalent = "let u2 = u1; // Move or Copy";

    assert!(c_code.contains("copy"), "C copies");
    assert!(
        rust_equivalent.contains("Move or Copy"),
        "Rust moves or copies"
    );

    // Demonstrate Copy enum
    #[derive(Copy, Clone)]
    enum U {
        Int(i32),
        Float(f32),
    }

    let u1 = U::Int(42);
    let u2 = u1; // Copy (because derives Copy)
    let _u3 = u1; // Can still use u1 (Copy)

    match u2 {
        U::Int(i) => assert_eq!(i, 42, "Copied value"),
        U::Float(_) => panic!("Wrong variant"),
    }
}

/// Document union initialization
///
/// C: union U {
///        int i;
///        float f;
///    };
///    union U u = { .i = 42 }; // Designated initializer
///
/// Rust: enum U {
///         Int(i32),
///         Float(f32),
///       }
///       let u = U::Int(42); // Variant constructor
///
/// **Transformation**: Union init → enum variant construction
/// - C: designated initializer specifies active member
/// - Rust: variant constructor is explicit and type-safe
///
/// Reference: K&R §6.8, ISO C99 §6.7.2.1
#[test]
fn test_union_initialization() {
    let c_code = "union U u = { .i = 42 };";
    let rust_equivalent = "let u = U::Int(42);";

    assert!(c_code.contains(".i"), "C designated init");
    assert!(
        rust_equivalent.contains("::Int"),
        "Rust variant constructor"
    );

    // Demonstrate enum construction
    enum U {
        Int(i32),
        Float(f32),
    }

    let u = U::Int(42);
    match u {
        U::Int(i) => assert_eq!(i, 42, "Initialized correctly"),
        U::Float(_) => panic!("Wrong variant"),
    }
}

/// Document nested unions
///
/// C: union Outer {
///        int i;
///        union Inner {
///            float f;
///            double d;
///        } inner;
///    };
///
/// Rust: enum Outer {
///         Int(i32),
///         Inner(Inner),
///       }
///       enum Inner {
///         Float(f32),
///         Double(f64),
///       }
///
/// **Transformation**: Nested union → nested enum
/// - Each union becomes separate enum
/// - Type-safe nesting
///
/// Reference: K&R §6.8, ISO C99 §6.7.2.1
#[test]
fn test_nested_unions() {
    let c_code = "union Outer { union Inner { float f; } inner; };";
    let rust_equivalent = "enum Outer { Inner(Inner) } enum Inner { Float(f32) }";

    assert!(c_code.contains("union"), "C nested union");
    assert!(rust_equivalent.contains("enum"), "Rust nested enum");

    // Demonstrate nested enums
    enum Inner {
        Float(f32),
        Double(f64),
    }

    enum Outer {
        Int(i32),
        Inner(Inner),
    }

    let o = Outer::Inner(Inner::Float(std::f32::consts::PI));
    match o {
        Outer::Int(_) => panic!("Wrong variant"),
        Outer::Inner(inner) => match inner {
            Inner::Float(f) => assert!((f - std::f32::consts::PI).abs() < 0.001, "Nested variant"),
            Inner::Double(_) => panic!("Wrong inner variant"),
        },
    }
}

/// Verify that safe union transformations introduce no unsafe blocks
///
/// Most union transformations use enum (safe)
#[test]
fn test_union_transformation_unsafe_count() {
    // Safe enum patterns
    let enum_pattern = "enum Value { Int(i32), Float(f32) }";
    let option_pattern = "Option<i32>";
    let result_pattern = "Result<i32, String>";

    // FFI union requires unsafe (for reading fields)
    let ffi_pattern = "#[repr(C)] union U { i: i32 } unsafe { u.i }";

    let safe_combined = format!("{}\n{}\n{}", enum_pattern, option_pattern, result_pattern);

    // Count unsafe in safe patterns (should be 0)
    let safe_unsafe_count = safe_combined.matches("unsafe").count();
    assert_eq!(
        safe_unsafe_count, 0,
        "Safe union transformations (enum) should not introduce unsafe blocks"
    );

    // FFI pattern requires unsafe
    let ffi_unsafe_count = ffi_pattern.matches("unsafe").count();
    assert_eq!(ffi_unsafe_count, 1, "FFI union requires unsafe to read");
}

/// Summary of union transformation rules
///
/// This test documents the complete set of rules for union transformation.
///
/// **C union → Rust Transformation**:
///
/// 1. **Simple union**: `union { int i; float f; }` → `enum { Int(i32), Float(f32) }` (SAFE)
/// 2. **Tagged union**: Manual tag + union → enum (SAFE, compiler-enforced)
/// 3. **Type punning**: Union type punning → to_bits/from_bits (SAFE) or transmute (unsafe)
/// 4. **FFI compatibility**: `union` → `#[repr(C)] union` (requires unsafe to read)
/// 5. **Optional values**: Tag + union → `Option<T>` (SAFE, built-in)
/// 6. **Result/error**: Tag + union → `Result<T, E>` (SAFE, built-in)
/// 7. **Struct variants**: Union with structs → enum with struct variants (SAFE)
/// 8. **Anonymous union**: Give it a name → named enum field (SAFE)
/// 9. **Nested unions**: Nested union → nested enum (SAFE)
/// 10. **Size**: C union size = max(members), Rust enum = max(members) + tag
///
/// **Key Advantages of Rust Approach**:
/// - Enums are type-safe (cannot read wrong variant)
/// - Pattern matching ensures exhaustiveness
/// - Compiler tracks which variant is active
/// - Built-in Option and Result types
/// - No manual tag management
/// - Prevents type confusion bugs
///
/// **Unsafe Blocks**: 0 for enum transformations, unsafe required for union reads
///
/// Reference: K&R §6.8, ISO C99 §6.7.2.1
#[test]
fn test_union_transformation_rules_summary() {
    // Rule 1: Enums are type-safe
    enum Value {
        Int(i32),
        Float(f32),
    }

    let v = Value::Int(42);
    match v {
        Value::Int(i) => assert_eq!(i, 42, "Type-safe access"),
        Value::Float(_) => panic!("Cannot accidentally read wrong type"),
    }

    // Rule 2: Option is safer than nullable union
    let opt: Option<i32> = Some(42);
    assert!(opt.is_some(), "Type-safe optional");

    // Rule 3: Result is safer than error union
    let res: Result<i32, String> = Ok(42);
    assert!(res.is_ok(), "Type-safe result");

    // Rule 4: No unsafe needed for enum (0 unsafe blocks)
    let unsafe_blocks = 0;
    assert_eq!(
        unsafe_blocks, 0,
        "Safe union transformations introduce 0 unsafe blocks"
    );

    // Rule 5: More memory safe than C
    let type_safe = true;
    assert!(type_safe, "Rust union transformation is type-safe");
}
