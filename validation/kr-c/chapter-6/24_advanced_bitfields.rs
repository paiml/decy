/* K&R C Chapter 6: Advanced Bit-Fields
 * K&R §6.9: Bit-field usage patterns
 * Transpiled to safe Rust (using bitflags and bit manipulation)
 */

// Rust doesn't have bit-fields, but we can use bit manipulation

// File permissions (similar to Unix chmod)
struct FilePermissions {
    bits: u16,
}

impl FilePermissions {
    const USER_READ: u16 = 1 << 8;
    const USER_WRITE: u16 = 1 << 7;
    const USER_EXECUTE: u16 = 1 << 6;
    const GROUP_READ: u16 = 1 << 5;
    const GROUP_WRITE: u16 = 1 << 4;
    const GROUP_EXECUTE: u16 = 1 << 3;
    const OTHER_READ: u16 = 1 << 2;
    const OTHER_WRITE: u16 = 1 << 1;
    const OTHER_EXECUTE: u16 = 1 << 0;

    fn new() -> Self {
        FilePermissions { bits: 0 }
    }

    fn set(&mut self, perm: u16) {
        self.bits |= perm;
    }

    fn clear(&mut self, perm: u16) {
        self.bits &= !perm;
    }

    fn has(&self, perm: u16) -> bool {
        self.bits & perm != 0
    }

    fn to_octal(&self) -> u16 {
        let user = ((self.bits >> 6) & 0b111) as u16;
        let group = ((self.bits >> 3) & 0b111) as u16;
        let other = (self.bits & 0b111) as u16;
        user * 100 + group * 10 + other
    }

    fn print(&self) {
        let user_r = if self.has(Self::USER_READ) { 'r' } else { '-' };
        let user_w = if self.has(Self::USER_WRITE) { 'w' } else { '-' };
        let user_x = if self.has(Self::USER_EXECUTE) { 'x' } else { '-' };
        let group_r = if self.has(Self::GROUP_READ) { 'r' } else { '-' };
        let group_w = if self.has(Self::GROUP_WRITE) { 'w' } else { '-' };
        let group_x = if self.has(Self::GROUP_EXECUTE) { 'x' } else { '-' };
        let other_r = if self.has(Self::OTHER_READ) { 'r' } else { '-' };
        let other_w = if self.has(Self::OTHER_WRITE) { 'w' } else { '-' };
        let other_x = if self.has(Self::OTHER_EXECUTE) { 'x' } else { '-' };

        println!("  User:   {}{}{}", user_r, user_w, user_x);
        println!("  Group:  {}{}{}", group_r, group_w, group_x);
        println!("  Other:  {}{}{}", other_r, other_w, other_x);
        println!("  Octal:  0{}", self.to_octal());
    }
}

// RGB565 color (16-bit color)
#[derive(Clone, Copy)]
struct RGB565 {
    value: u16,
}

impl RGB565 {
    fn new(red: u8, green: u8, blue: u8) -> Self {
        let r = ((red as u16) & 0x1F) << 11;
        let g = ((green as u16) & 0x3F) << 5;
        let b = (blue as u16) & 0x1F;
        RGB565 { value: r | g | b }
    }

    fn red(&self) -> u8 {
        ((self.value >> 11) & 0x1F) as u8
    }

    fn green(&self) -> u8 {
        ((self.value >> 5) & 0x3F) as u8
    }

    fn blue(&self) -> u8 {
        (self.value & 0x1F) as u8
    }
}

// CPU flags register
struct CPUFlags {
    bits: u32,
}

impl CPUFlags {
    const CARRY: u32 = 1 << 0;
    const ZERO: u32 = 1 << 1;
    const INTERRUPT: u32 = 1 << 2;
    const DIRECTION: u32 = 1 << 3;
    const OVERFLOW: u32 = 1 << 4;
    const SIGN: u32 = 1 << 5;
    const TRAP: u32 = 1 << 6;

    fn new() -> Self {
        CPUFlags { bits: 0 }
    }

    fn set(&mut self, flag: u32) {
        self.bits |= flag;
    }

    fn clear(&mut self, flag: u32) {
        self.bits &= !flag;
    }

    fn is_set(&self, flag: u32) -> bool {
        self.bits & flag != 0
    }

    fn print(&self) {
        println!("C={} Z={} I={} D={} O={} S={} T={}",
                 if self.is_set(Self::CARRY) { 1 } else { 0 },
                 if self.is_set(Self::ZERO) { 1 } else { 0 },
                 if self.is_set(Self::INTERRUPT) { 1 } else { 0 },
                 if self.is_set(Self::DIRECTION) { 1 } else { 0 },
                 if self.is_set(Self::OVERFLOW) { 1 } else { 0 },
                 if self.is_set(Self::SIGN) { 1 } else { 0 },
                 if self.is_set(Self::TRAP) { 1 } else { 0 });
    }
}

fn file_permissions_demo() {
    println!("=== File Permissions Demo ===");

    let mut perms = FilePermissions::new();

    // Set rwxr-xr-- (754)
    perms.set(FilePermissions::USER_READ);
    perms.set(FilePermissions::USER_WRITE);
    perms.set(FilePermissions::USER_EXECUTE);
    perms.set(FilePermissions::GROUP_READ);
    perms.set(FilePermissions::GROUP_EXECUTE);
    perms.set(FilePermissions::OTHER_READ);

    println!("Permissions:");
    perms.print();
    println!("  Size:   {} bytes\n", std::mem::size_of::<FilePermissions>());
}

fn rgb565_demo() {
    println!("=== RGB565 Color Demo ===");

    let red = RGB565::new(31, 0, 0);
    let green = RGB565::new(0, 63, 0);
    let blue = RGB565::new(0, 0, 31);
    let white = RGB565::new(31, 63, 31);

    println!("RGB565 colors:");
    println!("  Red:   R={} G={} B={}", red.red(), red.green(), red.blue());
    println!("  Green: R={} G={} B={}", green.red(), green.green(), green.blue());
    println!("  Blue:  R={} G={} B={}", blue.red(), blue.green(), blue.blue());
    println!("  White: R={} G={} B={}", white.red(), white.green(), white.blue());
    println!("  Size:  {} bytes (16-bit color)\n", std::mem::size_of::<RGB565>());
}

fn cpu_flags_demo() {
    println!("=== CPU Flags Demo ===");

    let mut flags = CPUFlags::new();

    print!("Initial flags: ");
    flags.print();

    // Simulate arithmetic operation
    flags.set(CPUFlags::ZERO);
    flags.clear(CPUFlags::CARRY);

    print!("After operation: ");
    flags.print();

    println!("Size: {} bytes\n", std::mem::size_of::<CPUFlags>());
}

fn bitfield_vs_masking() {
    println!("=== Bit-Field vs Bit Masking ===");

    // Bit masking approach (idiomatic Rust)
    let value: u8 = 0x42;
    let flags: u8 = 0xAB;
    let reg: u16 = ((flags as u16) << 8) | (value as u16);

    println!("Bit masking approach:");
    println!("  Value: 0x{:02X}", reg & 0xFF);
    println!("  Flags: 0x{:02X}", (reg >> 8) & 0xFF);

    println!("\nBit masking: Portable, explicit control\n");
}

fn main() {
    println!("=== Advanced Bit-Fields (Rust Alternative) ===\n");

    file_permissions_demo();
    rgb565_demo();
    cpu_flags_demo();
    bitfield_vs_masking();

    println!("Bit-field use cases:");
    println!("  - Hardware register access");
    println!("  - Network protocol headers");
    println!("  - Packed data structures");
    println!("  - Memory-efficient flags");
    println!("  - Graphics/image formats");
    println!("\nBest practices:");
    println!("  - Use bit manipulation (shifts and masks)");
    println!("  - Use bitflags crate for flags");
    println!("  - Document bit ordering");
    println!("  - Consider endianness");
}

// Idiomatic: use bitflags crate
#[allow(dead_code)]
fn demonstrate_bitflags() {
    // In Cargo.toml: bitflags = "2.0"
    // use bitflags::bitflags;
    //
    // bitflags! {
    //     struct Permissions: u16 {
    //         const READ = 0b00000001;
    //         const WRITE = 0b00000010;
    //         const EXECUTE = 0b00000100;
    //     }
    // }
    //
    // let mut perms = Permissions::READ | Permissions::WRITE;
    // perms.insert(Permissions::EXECUTE);
    // if perms.contains(Permissions::WRITE) {
    //     println!("Has write permission");
    // }
}

// Key differences from C:
// 1. No bit-field syntax in Rust
// 2. Use bit manipulation (shifts, masks, OR, AND)
// 3. bitflags crate for flag sets
// 4. Explicit bit positions (safer)
// 5. Methods for get/set operations
// 6. Type safety enforced
// 7. No implementation-defined behavior
// 8. Portable across platforms
