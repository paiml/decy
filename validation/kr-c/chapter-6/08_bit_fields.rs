/* K&R C Chapter 6.9: Bit-fields
 * Page 149-150
 * Bit-level structure packing
 * Transpiled to safe Rust using bitfields or manual bit manipulation
 */

use std::mem;

// Rust doesn't have built-in bit-fields, but we can use bitflags or manual manipulation
struct Flags {
    is_keyword: bool,
    is_extern: bool,
    is_static: bool,
}

struct Packed {
    opcode: u8,
    arg1: u8,  // Only uses 4 bits
    arg2: u8,  // Only uses 4 bits
}

impl Packed {
    fn new(opcode: u8, arg1: u8, arg2: u8) -> Self {
        Packed {
            opcode,
            arg1: arg1 & 0xF,  // Mask to 4 bits
            arg2: arg2 & 0xF,  // Mask to 4 bits
        }
    }
}

fn main() {
    let mut f = Flags {
        is_keyword: false,
        is_extern: false,
        is_static: false,
    };

    // Set individual bits
    f.is_keyword = true;
    f.is_extern = false;
    f.is_static = true;

    println!("Flags: keyword={} extern={} static={}",
           f.is_keyword as u32, f.is_extern as u32, f.is_static as u32);

    // Packed fields
    let p = Packed::new(0x42, 0xA, 0xB);

    println!("Packed: opcode=0x{:02X} arg1=0x{:X} arg2=0x{:X}",
           p.opcode, p.arg1, p.arg2);

    println!("Size of flags: {} bytes", mem::size_of::<Flags>());
    println!("Size of packed: {} bytes", mem::size_of::<Packed>());
}
