/* K&R C Chapter 6: Flexible Array Members
 * Structure with variable-length array at end (C99)
 * Transpiled to safe Rust (using Vec instead)
 */

// Idiomatic Rust: use Vec for variable-length data
struct Packet {
    id: i32,
    data: Vec<u8>,
}

impl Packet {
    fn new(id: i32, data: &[u8]) -> Self {
        Packet {
            id,
            data: data.to_vec(),
        }
    }

    fn from_str(id: i32, s: &str) -> Self {
        Packet {
            id,
            data: s.as_bytes().to_vec(),
        }
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.data)
    }

    fn print(&self) {
        if let Ok(s) = self.as_str() {
            println!("Packet {} [{} bytes]: \"{}\"", self.id, self.len(), s);
        } else {
            println!("Packet {} [{} bytes]: (binary data)", self.id, self.len());
        }
    }
}

fn main() {
    println!("=== Flexible Array Member Demo ===\n");

    let p1 = Packet::from_str(1, "Hello");
    let p2 = Packet::from_str(2, "This is a longer message");
    let p3 = Packet::from_str(3, "X");

    p1.print();
    p2.print();
    p3.print();

    println!("\nSizes:");
    println!("  sizeof(Packet header) = {} bytes (stack)",
             std::mem::size_of::<Packet>());
    println!("  Packet 1 data: {} bytes (heap)", p1.len());
    println!("  Packet 2 data: {} bytes (heap)", p2.len());
}

// Alternative: Using Box<[u8]> for fixed-size after creation
#[allow(dead_code)]
struct PacketBoxed {
    id: i32,
    length: usize,
    data: Box<[u8]>,
}

impl PacketBoxed {
    fn new(id: i32, data: &[u8]) -> Self {
        PacketBoxed {
            id,
            length: data.len(),
            data: data.to_vec().into_boxed_slice(),
        }
    }

    fn print(&self) {
        if let Ok(s) = std::str::from_utf8(&self.data) {
            println!("Packet {} [{} bytes]: \"{}\"", self.id, self.length, s);
        }
    }
}

// Unsafe alternative: truly flexible array (not recommended)
#[repr(C)]
struct PacketUnsafe {
    id: i32,
    length: usize,
    // data: [u8; 0], // Zero-sized array (requires unsafe)
}

impl PacketUnsafe {
    // Allocate with variable-length data
    fn alloc(id: i32, data: &[u8]) -> Box<Self> {
        unsafe {
            let layout = std::alloc::Layout::from_size_align(
                std::mem::size_of::<PacketUnsafe>() + data.len(),
                std::mem::align_of::<PacketUnsafe>(),
            )
            .unwrap();

            let ptr = std::alloc::alloc(layout) as *mut PacketUnsafe;
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }

            (*ptr).id = id;
            (*ptr).length = data.len();

            // Copy data after struct
            let data_ptr = ptr.add(1) as *mut u8;
            std::ptr::copy_nonoverlapping(data.as_ptr(), data_ptr, data.len());

            Box::from_raw(ptr)
        }
    }

    fn data(&self) -> &[u8] {
        unsafe {
            let data_ptr = (self as *const Self).add(1) as *const u8;
            std::slice::from_raw_parts(data_ptr, self.length)
        }
    }
}

// Key differences from C:
// 1. Vec<u8> instead of flexible array member
// 2. No manual malloc with size calculation
// 3. RAII: automatic cleanup
// 4. Length tracked automatically by Vec
// 5. Box<[u8]> for fixed-size heap array
// 6. Unsafe required for true flexible arrays
// 7. Layout API for custom allocation
// 8. Prefer Vec for variable-length data (idiomatic)
