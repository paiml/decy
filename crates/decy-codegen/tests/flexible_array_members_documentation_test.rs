//! Documentation tests for flexible array member transformations from C to Rust
//!
//! This test suite documents how C99 flexible array members (FAM) transform to
//! Rust patterns using Vec, slices, or unsafe with custom allocators.
//!
//! ## C99 Standard References
//! - ISO C99 §6.7.2.1 paragraph 16: Flexible array members (new in C99)
//! - K&R C 2nd Ed: Does NOT cover flexible array members (pre-C99)
//!
//! ## Key Transformations
//!
//! ### Basic Flexible Array Member
//! ```c
//! struct Buffer {
//!     size_t length;
//!     char data[];  // C99 flexible array member
//! };
//! ```
//! ```rust
//! struct Buffer {
//!     length: usize,
//!     data: Vec<u8>,  // Rust uses Vec for variable-length data
//! }
//! ```
//!
//! ### Allocation Pattern
//! ```c
//! struct Buffer* buf = malloc(sizeof(struct Buffer) + n);  // C99
//! buf->length = n;
//! ```
//! ```rust
//! let mut buf = Buffer {                                    // Rust
//!     length: n,
//!     data: vec![0; n],
//! };
//! ```
//!
//! ## Safety Considerations
//!
//! - **C FAM requires manual allocation**: malloc(sizeof(struct) + array_size)
//! - **Rust uses Vec**: Automatic memory management, no manual calculation
//! - **No buffer overflows**: Vec bounds checking prevents overruns
//! - **Type-safe**: Rust ensures length and data stay synchronized
//! - **RAII**: Automatic cleanup when struct goes out of scope
//!
//! ## Common Patterns
//!
//! 1. **Variable-length packets**: Network protocols with variable payload
//! 2. **String buffers**: Text with dynamic length
//! 3. **Dynamic arrays**: Arrays whose size is determined at runtime
//! 4. **Cache-friendly structures**: Keep header and data together
//!
//! ## Differences from C
//!
//! - Rust uses Vec (heap-allocated, owned) instead of inline array
//! - No manual size calculation needed
//! - Automatic memory management (no explicit free)
//! - Bounds checking on array access
//! - Cannot have truly zero-size FAM (Rust Vec has pointer overhead)

#[cfg(test)]
mod flexible_array_members_documentation_tests {
    /// Document transformation of basic flexible array member
    ///
    /// C99: `struct Buffer { size_t len; char data[]; };`
    /// Rust: `struct Buffer { len: usize, data: Vec<u8> }`
    ///
    /// Reference: ISO C99 §6.7.2.1 paragraph 16
    #[test]
    fn test_basic_flexible_array_member() {
        let c_code = r#"
struct Buffer {
    size_t length;
    char data[];  // Flexible array member
};
"#;
        let rust_equivalent = r#"
struct Buffer {
    length: usize,
    data: Vec<u8>,
}
"#;

        // Rust: use Vec for variable-length data
        struct Buffer {
            length: usize,
            data: Vec<u8>,
        }

        let buf = Buffer {
            length: 5,
            data: vec![1, 2, 3, 4, 5],
        };

        assert_eq!(buf.length, 5);
        assert_eq!(buf.data.len(), 5);
        assert_eq!(buf.data[0], 1);

        assert!(c_code.contains("char data[]"));
        assert!(rust_equivalent.contains("data: Vec<u8>"));
    }

    /// Document allocation pattern for flexible array member
    ///
    /// C99: `malloc(sizeof(struct Buffer) + n * sizeof(char))`
    /// Rust: `Buffer { length: n, data: vec![0; n] }`
    ///
    /// Reference: ISO C99 §6.7.2.1 paragraph 16
    #[test]
    fn test_flexible_array_allocation() {
        let c_code = r#"
struct Buffer* buf = malloc(sizeof(struct Buffer) + n);
buf->length = n;
"#;
        let rust_equivalent = r#"
let buf = Buffer {
    length: n,
    data: vec![0; n],
};
"#;

        struct Buffer {
            length: usize,
            data: Vec<u8>,
        }

        let n = 10;
        // Rust: no manual size calculation, Vec handles allocation
        let buf = Buffer {
            length: n,
            data: vec![0; n],
        };

        assert_eq!(buf.length, 10);
        assert_eq!(buf.data.len(), 10);
        assert_eq!(buf.data.capacity(), 10);

        assert!(c_code.contains("malloc(sizeof(struct Buffer) + n)"));
        assert!(rust_equivalent.contains("vec![0; n]"));
    }

    /// Document network packet with flexible array (common use case)
    ///
    /// C99: Variable-length payload in packet
    /// Rust: Vec for payload
    ///
    /// Reference: Common C99 pattern
    #[test]
    fn test_network_packet_flexible_array() {
        let c_code = r#"
struct Packet {
    uint32_t id;
    uint16_t length;
    uint8_t payload[];  // Flexible array
};
"#;
        let rust_equivalent = r#"
struct Packet {
    id: u32,
    length: u16,
    payload: Vec<u8>,
}
"#;

        struct Packet {
            id: u32,
            length: u16,
            payload: Vec<u8>,
        }

        let packet = Packet {
            id: 12345,
            length: 4,
            payload: vec![0xDE, 0xAD, 0xBE, 0xEF],
        };

        assert_eq!(packet.id, 12345);
        assert_eq!(packet.length, 4);
        assert_eq!(packet.payload.len(), 4);
        assert_eq!(packet.payload[0], 0xDE);

        assert!(c_code.contains("uint8_t payload[]"));
        assert!(rust_equivalent.contains("payload: Vec<u8>"));
    }

    /// Document string buffer with flexible array
    ///
    /// C99: `struct String { size_t len; char str[]; }`
    /// Rust: `struct StringBuffer { len: usize, str: String }`
    ///
    /// Reference: ISO C99 §6.7.2.1 paragraph 16
    #[test]
    fn test_string_buffer_flexible_array() {
        let c_code = r#"
struct StringBuffer {
    size_t len;
    char str[];  // Flexible array for string
};
"#;
        let rust_equivalent = r#"
struct StringBuffer {
    len: usize,
    str: String,
}
"#;

        struct StringBuffer {
            len: usize,
            str: String,
        }

        let text = "Hello, World!";
        let buf = StringBuffer {
            len: text.len(),
            str: text.to_string(),
        };

        assert_eq!(buf.len, 13);
        assert_eq!(buf.str, "Hello, World!");

        assert!(c_code.contains("char str[]"));
        assert!(rust_equivalent.contains("str: String"));
    }

    /// Document modification of flexible array member
    ///
    /// C99: Direct modification of array elements
    /// Rust: Vec modification with bounds checking
    ///
    /// Reference: ISO C99 §6.7.2.1 paragraph 16
    #[test]
    fn test_flexible_array_modification() {
        let c_code = r#"
buf->data[i] = value;  // Direct array access
"#;
        let rust_equivalent = "buf.data[i] = value;  // Bounds-checked access";

        struct Buffer {
            length: usize,
            data: Vec<u8>,
        }

        let mut buf = Buffer {
            length: 5,
            data: vec![0, 0, 0, 0, 0],
        };

        // Modify elements (bounds-checked)
        buf.data[0] = 10;
        buf.data[4] = 50;

        assert_eq!(buf.data[0], 10);
        assert_eq!(buf.data[4], 50);

        // Rust prevents out-of-bounds access at runtime
        // buf.data[10] = 100;  // Would panic!

        assert!(c_code.contains("buf->data[i]"));
        assert!(rust_equivalent.contains("buf.data[i]"));
    }

    /// Document copying flexible array member structs
    ///
    /// C99: Manual memcpy with size calculation
    /// Rust: Clone trait or manual copy
    ///
    /// Reference: ISO C99 §6.7.2.1 paragraph 16
    #[test]
    fn test_flexible_array_copy() {
        let c_code = r#"
struct Buffer* copy = malloc(sizeof(struct Buffer) + src->length);
memcpy(copy, src, sizeof(struct Buffer) + src->length);
"#;
        let rust_equivalent = "let copy = buf.clone();";

        #[derive(Debug, Clone, PartialEq)]
        struct Buffer {
            length: usize,
            data: Vec<u8>,
        }

        let buf = Buffer {
            length: 3,
            data: vec![1, 2, 3],
        };

        // Rust: simple clone (Vec is cloned automatically)
        let copy = buf.clone();

        assert_eq!(copy.length, buf.length);
        assert_eq!(copy.data, buf.data);
        assert_eq!(copy, buf);

        assert!(c_code.contains("memcpy"));
        assert!(rust_equivalent.contains("clone()"));
    }

    /// Document flexible array with different element types
    ///
    /// C99: `struct Container { size_t count; int values[]; }`
    /// Rust: `struct Container { count: usize, values: Vec<i32> }`
    ///
    /// Reference: ISO C99 §6.7.2.1 paragraph 16
    #[test]
    fn test_flexible_array_different_types() {
        let c_code = r#"
struct IntArray {
    size_t count;
    int values[];  // Flexible array of integers
};
"#;
        let rust_equivalent = r#"
struct IntArray {
    count: usize,
    values: Vec<i32>,
}
"#;

        struct IntArray {
            count: usize,
            values: Vec<i32>,
        }

        let arr = IntArray {
            count: 4,
            values: vec![10, 20, 30, 40],
        };

        assert_eq!(arr.count, 4);
        assert_eq!(arr.values.len(), 4);
        assert_eq!(arr.values[2], 30);

        assert!(c_code.contains("int values[]"));
        assert!(rust_equivalent.contains("values: Vec<i32>"));
    }

    /// Document zero-length array (GCC extension before C99)
    ///
    /// GCC: `char data[0];` (pre-C99 extension)
    /// C99: `char data[];` (standard)
    /// Rust: `Vec<u8>`
    ///
    /// Reference: GCC extension, ISO C99 §6.7.2.1
    #[test]
    fn test_zero_length_array_gcc_extension() {
        let c_code_gcc = "char data[0];  // GCC extension (pre-C99)";
        let c_code_c99 = "char data[];   // C99 flexible array member";
        let rust_equivalent = "data: Vec<u8>";

        struct Buffer {
            length: usize,
            data: Vec<u8>,
        }

        let buf = Buffer {
            length: 0,
            data: Vec::new(), // Empty Vec
        };

        assert_eq!(buf.length, 0);
        assert_eq!(buf.data.len(), 0);
        assert!(buf.data.is_empty());

        assert!(c_code_gcc.contains("[0]"));
        assert!(c_code_c99.contains("[]"));
        assert!(rust_equivalent.contains("Vec<u8>"));
    }

    /// Document flexible array with struct elements
    ///
    /// C99: `struct Container { size_t count; struct Point items[]; }`
    /// Rust: `struct Container { count: usize, items: Vec<Point> }`
    ///
    /// Reference: ISO C99 §6.7.2.1 paragraph 16
    #[test]
    fn test_flexible_array_struct_elements() {
        let c_code = r#"
struct Container {
    size_t count;
    struct Point items[];  // Flexible array of structs
};
"#;
        let rust_equivalent = r#"
struct Container {
    count: usize,
    items: Vec<Point>,
}
"#;

        #[derive(Debug, Clone, Copy, PartialEq)]
        struct Point {
            x: i32,
            y: i32,
        }

        struct Container {
            count: usize,
            items: Vec<Point>,
        }

        let container = Container {
            count: 2,
            items: vec![Point { x: 1, y: 2 }, Point { x: 3, y: 4 }],
        };

        assert_eq!(container.count, 2);
        assert_eq!(container.items.len(), 2);
        assert_eq!(container.items[0], Point { x: 1, y: 2 });

        assert!(c_code.contains("struct Point items[]"));
        assert!(rust_equivalent.contains("items: Vec<Point>"));
    }

    /// Document restrictions on flexible array members
    ///
    /// C99: FAM must be last member, only one per struct
    /// Rust: Vec can be anywhere, multiple Vecs allowed
    ///
    /// Reference: ISO C99 §6.7.2.1 paragraph 16
    #[test]
    fn test_flexible_array_restrictions() {
        let c_note = r#"
C99 Restrictions:
1. FAM must be the last member
2. Only one FAM per struct
3. Struct must have at least one named member before FAM
4. Cannot use sizeof on struct with FAM
"#;
        let rust_note = r#"
Rust: No such restrictions
1. Vec can be anywhere in struct
2. Multiple Vecs allowed
3. sizeof works normally (includes Vec pointer/len/cap)
"#;

        // Rust: multiple Vecs, any position
        struct MultiBuffer {
            data1: Vec<u8>, // First Vec (allowed in Rust)
            length: usize,
            data2: Vec<u8>, // Second Vec (allowed in Rust)
        }

        let buf = MultiBuffer {
            data1: vec![1, 2, 3],
            length: 3,
            data2: vec![4, 5, 6],
        };

        assert_eq!(buf.data1.len(), 3);
        assert_eq!(buf.data2.len(), 3);

        assert!(c_note.contains("must be the last member"));
        assert!(rust_note.contains("Vec can be anywhere"));
    }

    /// Document realloc pattern for flexible array
    ///
    /// C99: `realloc(buf, sizeof(struct Buffer) + new_size)`
    /// Rust: `buf.data.resize(new_size, 0)`
    ///
    /// Reference: ISO C99 §6.7.2.1, §7.20.3.4
    #[test]
    fn test_flexible_array_realloc() {
        let c_code = r#"
buf = realloc(buf, sizeof(struct Buffer) + new_size);
buf->length = new_size;
"#;
        let rust_equivalent = r#"
buf.data.resize(new_size, 0);
buf.length = new_size;
"#;

        struct Buffer {
            length: usize,
            data: Vec<u8>,
        }

        let mut buf = Buffer {
            length: 5,
            data: vec![1, 2, 3, 4, 5],
        };

        // Resize the flexible array
        let new_size = 8;
        buf.data.resize(new_size, 0); // Resize, fill new elements with 0
        buf.length = new_size;

        assert_eq!(buf.length, 8);
        assert_eq!(buf.data.len(), 8);
        assert_eq!(buf.data[4], 5); // Old data preserved
        assert_eq!(buf.data[7], 0); // New elements initialized to 0

        assert!(c_code.contains("realloc"));
        assert!(rust_equivalent.contains("resize"));
    }

    /// Document using slice instead of Vec for borrowed data
    ///
    /// C99: Pointer to FAM struct
    /// Rust: Struct with slice reference (borrowed, not owned)
    ///
    /// Reference: Rust alternative pattern
    #[test]
    fn test_flexible_array_with_slice() {
        let c_note = "C99 FAM is always owned (part of allocated struct)";
        let rust_note = "Rust can use slice (&[T]) for borrowed data";

        // Rust: use slice for borrowed (non-owned) data
        struct BufferRef<'a> {
            length: usize,
            data: &'a [u8],
        }

        let backing_data = [1, 2, 3, 4, 5];
        let buf = BufferRef {
            length: 5,
            data: &backing_data,
        };

        assert_eq!(buf.length, 5);
        assert_eq!(buf.data.len(), 5);
        assert_eq!(buf.data[0], 1);

        // Slice doesn't own the data - backing_data must outlive buf

        assert!(c_note.contains("owned"));
        assert!(rust_note.contains("&[T]"));
    }

    /// Document Box with DST (dynamically sized type) alternative
    ///
    /// C99: FAM with malloc
    /// Rust: Box<[T]> for heap-allocated slice (advanced)
    ///
    /// Reference: Rust DST pattern
    #[test]
    fn test_flexible_array_box_slice() {
        let c_note = "C99 FAM: malloc with calculated size";
        let rust_note = "Rust advanced: Box<[T]> for heap slice";

        // Rust: Box<[T]> for owned heap-allocated slice
        struct BufferBox {
            length: usize,
            data: Box<[u8]>,
        }

        let buf = BufferBox {
            length: 5,
            data: vec![1, 2, 3, 4, 5].into_boxed_slice(),
        };

        assert_eq!(buf.length, 5);
        assert_eq!(buf.data.len(), 5);
        assert_eq!(buf.data[0], 1);

        // Box<[T]> is heap-allocated, fixed size (can't resize like Vec)

        assert!(c_note.contains("malloc"));
        assert!(rust_note.contains("Box<[T]>"));
    }

    /// Document transformation rules summary
    ///
    /// This test documents all transformation rules and differences
    /// between C99 flexible array members and Rust patterns.
    #[test]
    fn test_flexible_array_transformation_rules() {
        let c_summary = r#"
C99 Flexible Array Member Rules:
1. Syntax: type array[] at end of struct
2. Must be last member
3. Only one per struct
4. Requires manual allocation: malloc(sizeof(struct) + array_size)
5. Requires manual size calculation
6. No bounds checking
7. Manual memory management (malloc/free/realloc)
"#;

        let rust_summary = r#"
Rust Transformation Patterns:
1. Vec<T>: Owned, growable, heap-allocated (most common)
2. &[T]: Borrowed slice (non-owning reference)
3. Box<[T]>: Owned, fixed-size, heap-allocated
4. Automatic memory management (RAII)
5. Bounds checking (runtime safety)
6. No manual size calculation
7. Multiple Vecs allowed, any position
"#;

        // Most common pattern: Vec
        struct BufferVec {
            length: usize,
            data: Vec<u8>,
        }

        let buf = BufferVec {
            length: 3,
            data: vec![1, 2, 3],
        };
        assert_eq!(buf.data.len(), 3);

        // Alternative: slice reference
        struct BufferRef<'a> {
            length: usize,
            data: &'a [u8],
        }

        let backing = [1, 2, 3];
        let buf_ref = BufferRef {
            length: 3,
            data: &backing,
        };
        assert_eq!(buf_ref.data.len(), 3);

        assert!(c_summary.contains("Must be last member"));
        assert!(rust_summary.contains("Vec<T>: Owned, growable"));
    }
}
