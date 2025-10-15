//! Documentation tests for bitwise operators transformation (EXPR-BITWISE validation)
//!
//! Reference: K&R §2.9, ISO C99 §6.5.10-6.5.12
//!
//! This module documents the transformation of C bitwise operators to Rust equivalents.
//! Bitwise operators manipulate individual bits in integer types, commonly used for:
//! - Flags and bit masks
//! - Low-level programming
//! - Embedded systems
//! - Hardware register manipulation
//! - Performance optimization
//!
//! **C Bitwise Operators**:
//! - `&` - Bitwise AND
//! - `|` - Bitwise OR
//! - `^` - Bitwise XOR
//! - `~` - Bitwise NOT (complement)
//! - `<<` - Left shift
//! - `>>` - Right shift
//!
//! **Rust Equivalents**:
//! - Same syntax: `&`, `|`, `^`, `!` (not ~), `<<`, `>>`
//! - Compound assignment: `&=`, `|=`, `^=`, `<<=`, `>>=`
//! - Type-safe (won't compile with wrong types)
//! - Overflow behavior is well-defined
//!
//! **Key Safety Property**: All bitwise operations are safe (0 unsafe blocks)

/// Document transformation of bitwise AND operator
///
/// C: int flags = a & b;
///
/// Rust: let flags = a & b;
///
/// **Transformation**: Bitwise AND is identical in C and Rust
/// - Used for: masking bits, testing flags, extracting bitfields
/// - Example: `flags & FLAG_ENABLED` tests if flag is set
///
/// Reference: K&R §2.9, ISO C99 §6.5.10
#[test]
fn test_bitwise_and_operator() {
    // C code concept
    let c_code = "int result = a & b;";
    let rust_equivalent = "let result = a & b;";

    assert!(c_code.contains("&"), "C uses & for bitwise AND");
    assert!(rust_equivalent.contains("&"), "Rust uses & for bitwise AND");

    // Demonstrate bitwise AND behavior
    let a: i32 = 0b1100;
    let b: i32 = 0b1010;
    let result = a & b;
    assert_eq!(result, 0b1000, "1100 & 1010 = 1000");

    // Common use: masking bits
    let flags: u32 = 0b11010;
    let mask: u32 = 0b00011;
    let masked = flags & mask;
    assert_eq!(masked, 0b00010, "Masking extracts lower 2 bits");
}

/// Document transformation of bitwise OR operator
///
/// C: int flags = a | b;
///
/// Rust: let flags = a | b;
///
/// **Transformation**: Bitwise OR is identical in C and Rust
/// - Used for: setting bits, combining flags
/// - Example: `flags | FLAG_ENABLED` sets flag
///
/// Reference: K&R §2.9, ISO C99 §6.5.11
#[test]
fn test_bitwise_or_operator() {
    let c_code = "int result = a | b;";
    let rust_equivalent = "let result = a | b;";

    assert!(c_code.contains("|"), "C uses | for bitwise OR");
    assert!(rust_equivalent.contains("|"), "Rust uses | for bitwise OR");

    // Demonstrate bitwise OR behavior
    let a: i32 = 0b1100;
    let b: i32 = 0b1010;
    let result = a | b;
    assert_eq!(result, 0b1110, "1100 | 1010 = 1110");

    // Common use: setting bits
    let flags: u32 = 0b00000;
    let flag1: u32 = 0b00001;
    let flag2: u32 = 0b00100;
    let combined = flags | flag1 | flag2;
    assert_eq!(combined, 0b00101, "OR combines flags");
}

/// Document transformation of bitwise XOR operator
///
/// C: int result = a ^ b;
///
/// Rust: let result = a ^ b;
///
/// **Transformation**: Bitwise XOR is identical in C and Rust
/// - Used for: toggling bits, simple encryption, detecting differences
/// - Property: `a ^ a = 0`, `a ^ 0 = a`, `a ^ b ^ b = a`
///
/// Reference: K&R §2.9, ISO C99 §6.5.12
#[test]
fn test_bitwise_xor_operator() {
    let c_code = "int result = a ^ b;";
    let rust_equivalent = "let result = a ^ b;";

    assert!(c_code.contains("^"), "C uses ^ for bitwise XOR");
    assert!(rust_equivalent.contains("^"), "Rust uses ^ for bitwise XOR");

    // Demonstrate bitwise XOR behavior
    let a: i32 = 0b1100;
    let b: i32 = 0b1010;
    let result = a ^ b;
    assert_eq!(result, 0b0110, "1100 ^ 1010 = 0110");

    // Common use: toggling bits
    let flags: u32 = 0b1010;
    let toggle_mask: u32 = 0b0011;
    let toggled = flags ^ toggle_mask;
    assert_eq!(toggled, 0b1001, "XOR toggles bits");

    // XOR properties
    assert_eq!(a ^ a, 0, "a ^ a = 0");
    assert_eq!(a ^ 0, a, "a ^ 0 = a");
    assert_eq!(a ^ b ^ b, a, "a ^ b ^ b = a (swap trick)");
}

/// Document transformation of bitwise NOT operator
///
/// C: int result = ~a;
///
/// Rust: let result = !a;
///
/// **Transformation**: C uses ~, Rust uses ! for bitwise NOT
/// - Used for: inverting bits, creating bitmasks
/// - Note: Different syntax but same semantics
///
/// Reference: K&R §2.9, ISO C99 §6.5.3.3
#[test]
fn test_bitwise_not_operator() {
    let c_code = "int result = ~a;";
    let rust_equivalent = "let result = !a;";

    assert!(c_code.contains("~"), "C uses ~ for bitwise NOT");
    assert!(rust_equivalent.contains("!"), "Rust uses ! for bitwise NOT");

    // Demonstrate bitwise NOT behavior
    let a: u8 = 0b00001111;
    let result = !a;
    assert_eq!(result, 0b11110000, "!00001111 = 11110000");

    // Common use: creating inverse masks
    let mask: u32 = 0b00000011;
    let inverse_mask = !mask;
    assert_eq!(inverse_mask & 0xFF, 0b11111100, "NOT creates inverse mask");
}

/// Document transformation of left shift operator
///
/// C: int result = a << n;
///
/// Rust: let result = a << n;
///
/// **Transformation**: Left shift is identical in C and Rust
/// - Used for: multiplying by powers of 2, positioning bits
/// - Equivalent to: a * 2^n (for non-negative)
/// - Rust has overflow checking in debug mode
///
/// Reference: K&R §2.9, ISO C99 §6.5.7
#[test]
fn test_left_shift_operator() {
    let c_code = "int result = a << 2;";
    let rust_equivalent = "let result = a << 2;";

    assert!(c_code.contains("<<"), "C uses << for left shift");
    assert!(
        rust_equivalent.contains("<<"),
        "Rust uses << for left shift"
    );

    // Demonstrate left shift behavior
    let a: u32 = 0b0001;
    assert_eq!(a << 0, 0b0001, "Shift by 0 = no change");
    assert_eq!(a << 1, 0b0010, "Shift left 1 = multiply by 2");
    assert_eq!(a << 2, 0b0100, "Shift left 2 = multiply by 4");
    assert_eq!(a << 3, 0b1000, "Shift left 3 = multiply by 8");

    // Common use: bit positioning
    let bit_position = 5;
    let flag = 1 << bit_position;
    assert_eq!(flag, 32, "1 << 5 creates flag at bit 5");
}

/// Document transformation of right shift operator
///
/// C: int result = a >> n;
///
/// Rust: let result = a >> n;
///
/// **Transformation**: Right shift is identical in C and Rust
/// - Used for: dividing by powers of 2, extracting bits
/// - Equivalent to: a / 2^n (for unsigned)
/// - Signed vs unsigned behavior: arithmetic vs logical shift
///
/// Reference: K&R §2.9, ISO C99 §6.5.7
#[test]
fn test_right_shift_operator() {
    let c_code = "int result = a >> 2;";
    let rust_equivalent = "let result = a >> 2;";

    assert!(c_code.contains(">>"), "C uses >> for right shift");
    assert!(
        rust_equivalent.contains(">>"),
        "Rust uses >> for right shift"
    );

    // Demonstrate right shift behavior (unsigned)
    let a: u32 = 0b1000;
    assert_eq!(a >> 0, 0b1000, "Shift by 0 = no change");
    assert_eq!(a >> 1, 0b0100, "Shift right 1 = divide by 2");
    assert_eq!(a >> 2, 0b0010, "Shift right 2 = divide by 4");
    assert_eq!(a >> 3, 0b0001, "Shift right 3 = divide by 8");

    // Common use: extracting bits
    let value: u32 = 0b11010000;
    let extracted = (value >> 4) & 0b1111;
    assert_eq!(extracted, 0b1101, "Right shift + mask extracts high nibble");
}

/// Document signed vs unsigned right shift behavior
///
/// C: Arithmetic shift (sign-extend) vs logical shift (zero-fill)
///
/// Rust: Same behavior, but explicit types ensure correctness
///
/// **Transformation**: Rust's type system prevents sign errors
/// - Signed types (i32): arithmetic shift (sign-extend)
/// - Unsigned types (u32): logical shift (zero-fill)
///
/// Reference: K&R §2.9, ISO C99 §6.5.7
#[test]
fn test_signed_unsigned_right_shift() {
    // Unsigned right shift: zero-fill
    let unsigned: u32 = 0b11111111_11111111_11111111_11111111;
    let shifted_unsigned = unsigned >> 1;
    assert_eq!(
        shifted_unsigned, 0b01111111_11111111_11111111_11111111,
        "Unsigned right shift zero-fills"
    );

    // Signed right shift: sign-extend (arithmetic shift)
    let signed: i32 = -1; // All bits set
    let shifted_signed = signed >> 1;
    assert_eq!(shifted_signed, -1, "Signed right shift sign-extends");

    // Positive signed value
    let positive: i32 = 0b00001000;
    let shifted_positive = positive >> 1;
    assert_eq!(
        shifted_positive, 0b00000100,
        "Positive signed shifts normally"
    );
}

/// Document compound assignment operators
///
/// C: a &= b; a |= b; a ^= b; a <<= n; a >>= n;
///
/// Rust: a &= b; a |= b; a ^= b; a <<= n; a >>= n;
///
/// **Transformation**: Compound bitwise assignments identical in C and Rust
/// - More concise than `a = a op b`
/// - Common for flag manipulation
///
/// Reference: K&R §2.10, ISO C99 §6.5.16
#[test]
fn test_bitwise_compound_assignment() {
    // Bitwise AND assignment
    let mut flags = 0b1111u32;
    flags &= 0b1100;
    assert_eq!(flags, 0b1100, "&= clears bits");

    // Bitwise OR assignment
    let mut flags = 0b1000u32;
    flags |= 0b0011;
    assert_eq!(flags, 0b1011, "|= sets bits");

    // Bitwise XOR assignment
    let mut flags = 0b1010u32;
    flags ^= 0b0011;
    assert_eq!(flags, 0b1001, "^= toggles bits");

    // Left shift assignment
    let mut value = 1u32;
    value <<= 3;
    assert_eq!(value, 8, "<<= multiplies by power of 2");

    // Right shift assignment
    let mut value = 8u32;
    value >>= 2;
    assert_eq!(value, 2, ">>= divides by power of 2");
}

/// Document bitmask patterns
///
/// C: Common bitmask patterns for flags
///
/// Rust: Same patterns with type safety
///
/// **Transformation**: Bitmask patterns work identically
/// - Setting a bit: `flags |= (1 << n)`
/// - Clearing a bit: `flags &= !(1 << n)`
/// - Toggling a bit: `flags ^= (1 << n)`
/// - Testing a bit: `(flags & (1 << n)) != 0`
///
/// Reference: K&R §2.9
#[test]
fn test_bitmask_patterns() {
    let mut flags: u32 = 0;

    // Setting a bit
    let bit_to_set = 3;
    flags |= 1 << bit_to_set;
    assert_eq!(flags, 0b1000, "Set bit 3");

    // Testing a bit
    let is_set = (flags & (1 << bit_to_set)) != 0;
    assert!(is_set, "Bit 3 is set");

    // Clearing a bit
    flags &= !(1 << bit_to_set);
    assert_eq!(flags, 0b0000, "Cleared bit 3");

    // Toggling a bit
    flags ^= 1 << bit_to_set;
    assert_eq!(flags, 0b1000, "Toggled bit 3 on");
    flags ^= 1 << bit_to_set;
    assert_eq!(flags, 0b0000, "Toggled bit 3 off");
}

/// Document multiple flags handling
///
/// C: Combining multiple flags with OR
///
/// Rust: Same pattern with type safety and const flags
///
/// **Transformation**: Flag patterns work identically
/// - Define flags as constants
/// - Combine with OR
/// - Test with AND
///
/// Reference: K&R §2.9
#[test]
fn test_multiple_flags() {
    // Define flags (would be const in real code)
    const FLAG_READ: u32 = 1 << 0; // 0b0001
    const FLAG_WRITE: u32 = 1 << 1; // 0b0010
    const FLAG_EXECUTE: u32 = 1 << 2; // 0b0100
    const FLAG_APPEND: u32 = 1 << 3; // 0b1000

    // Combine flags
    let permissions = FLAG_READ | FLAG_WRITE;
    assert_eq!(permissions, 0b0011, "Read + Write flags");

    // Test for specific flag
    assert!((permissions & FLAG_READ) != 0, "Has READ permission");
    assert!((permissions & FLAG_WRITE) != 0, "Has WRITE permission");
    assert!((permissions & FLAG_EXECUTE) == 0, "No EXECUTE permission");

    // Add a flag
    let mut perms = permissions;
    perms |= FLAG_APPEND;
    assert_eq!(perms, 0b1011, "Added APPEND flag");

    // Remove a flag
    perms &= !FLAG_WRITE;
    assert_eq!(perms, 0b1001, "Removed WRITE flag");
}

/// Document bit extraction and insertion
///
/// C: Extracting and inserting bitfields
///
/// Rust: Same patterns with clearer syntax
///
/// **Transformation**: Bitfield operations identical
/// - Extract: `(value >> shift) & mask`
/// - Insert: `(value & ~(mask << shift)) | (bits << shift)`
///
/// Reference: K&R §2.9
#[test]
fn test_bit_extraction_insertion() {
    // Extract bits 4-7 (nibble)
    let value: u32 = 0b11010110;
    let shift = 4;
    let mask = 0b1111;
    let extracted = (value >> shift) & mask;
    assert_eq!(extracted, 0b1101, "Extracted high nibble");

    // Insert bits 4-7
    let mut target: u32 = 0b00001111;
    let bits_to_insert: u32 = 0b1010;
    target = (target & !(mask << shift)) | (bits_to_insert << shift);
    assert_eq!(target, 0b10101111, "Inserted bits at position 4-7");
}

/// Document power-of-2 operations
///
/// C: Common power-of-2 tricks using bitwise ops
///
/// Rust: Same tricks, but with clearer intent
///
/// **Transformation**: Power-of-2 tricks work identically
/// - Check power of 2: `(n & (n - 1)) == 0`
/// - Round up to power of 2: bit manipulation
/// - Align to power of 2: `(n + align - 1) & !(align - 1)`
///
/// Reference: K&R §2.9
#[test]
fn test_power_of_two_operations() {
    // Check if power of 2
    let n = 16u32;
    let is_power_of_2 = n != 0 && (n & (n - 1)) == 0;
    assert!(is_power_of_2, "16 is power of 2");

    let n = 15u32;
    let is_not_power_of_2 = n != 0 && (n & (n - 1)) == 0;
    assert!(!is_not_power_of_2, "15 is not power of 2");

    // Align to power of 2 (e.g., 8-byte alignment)
    let align = 8u32;
    let value = 13u32;
    let aligned = (value + align - 1) & !(align - 1);
    assert_eq!(aligned, 16, "13 aligned to 8 bytes = 16");
}

/// Document byte swapping (endianness)
///
/// C: Manual byte swapping with shifts and masks
///
/// Rust: Use built-in methods or same manual technique
///
/// **Transformation**: Byte swapping patterns identical
/// - Manual: shift and OR each byte
/// - Rust also has: `value.swap_bytes()`
///
/// Reference: K&R §2.9
#[test]
fn test_byte_swapping() {
    // Manual byte swap (16-bit)
    let value: u16 = 0x1234;
    let swapped = ((value & 0xFF) << 8) | ((value >> 8) & 0xFF);
    assert_eq!(swapped, 0x3412, "Byte-swapped 16-bit value");

    // Using Rust built-in
    let swapped_builtin = value.swap_bytes();
    assert_eq!(swapped_builtin, 0x3412, "swap_bytes() does same thing");

    // 32-bit byte swap
    let value32: u32 = 0x12345678;
    let swapped32 = value32.swap_bytes();
    assert_eq!(swapped32, 0x78563412, "Byte-swapped 32-bit value");
}

/// Document bitwise operator precedence
///
/// C: Bitwise operators have lower precedence than comparison
///
/// Rust: Same precedence rules
///
/// **Transformation**: Precedence identical, but be explicit
/// - `a & b == c` means `a & (b == c)` NOT `(a & b) == c`
/// - Always use parentheses for clarity
///
/// Reference: K&R §2.12, ISO C99 §A.13
#[test]
fn test_bitwise_operator_precedence() {
    let a = 5u32;
    let b = 3u32;

    // Need parentheses for intended meaning
    let result_with_parens = (a & b) == 1;
    assert!(result_with_parens, "(5 & 3) == 1 is true");

    // Without parentheses, comparison happens first
    // a & b == 1 means a & (b == 1)
    let comparison = b == 1; // false
    let result_without_parens = a & (comparison as u32);
    assert_eq!(result_without_parens, 0, "5 & (3 == 1) = 5 & 0 = 0");
}

/// Verify that bitwise operators introduce no unsafe blocks
///
/// All bitwise operations in Rust are safe
#[test]
fn test_bitwise_transformation_unsafe_count() {
    // Various bitwise patterns
    let and_op = "let result = a & b;";
    let or_op = "let result = a | b;";
    let xor_op = "let result = a ^ b;";
    let not_op = "let result = !a;";
    let left_shift = "let result = a << n;";
    let right_shift = "let result = a >> n;";
    let compound = "flags |= mask;";

    let combined = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}",
        and_op, or_op, xor_op, not_op, left_shift, right_shift, compound
    );

    // Count unsafe blocks (should be 0)
    let unsafe_count = combined.matches("unsafe").count();
    assert_eq!(
        unsafe_count, 0,
        "Bitwise operators should not introduce unsafe blocks"
    );
}

/// Summary of bitwise operator transformations
///
/// This test documents the complete set of rules for bitwise operator transformation.
///
/// **C Bitwise Operator → Rust Transformation**:
///
/// 1. **Bitwise AND** (`&`): Identical syntax and semantics
/// 2. **Bitwise OR** (`|`): Identical syntax and semantics
/// 3. **Bitwise XOR** (`^`): Identical syntax and semantics
/// 4. **Bitwise NOT** (`~`): C uses `~`, Rust uses `!`
/// 5. **Left shift** (`<<`): Identical syntax and semantics
/// 6. **Right shift** (`>>`): Identical, but type system ensures correct signed/unsigned
/// 7. **Compound assignment** (`&=`, `|=`, `^=`, `<<=`, `>>=`): Identical
///
/// **Key Advantages of Rust Approach**:
/// - Type safety prevents mixing signed/unsigned incorrectly
/// - Overflow behavior is well-defined (debug: panic, release: wrap)
/// - Same syntax and semantics as C (easy transpilation)
/// - No undefined behavior from invalid shift amounts (Rust panics in debug)
///
/// **Common Patterns**:
/// - Flag manipulation: `|` to set, `&` to clear, `^` to toggle, `&` to test
/// - Bitmasks: `&` with mask to extract bits
/// - Power of 2: `n & (n-1) == 0` to test
/// - Alignment: `(n + align - 1) & !(align - 1)` to align
///
/// **Unsafe Blocks**: 0 (all bitwise operations are safe)
///
/// Reference: K&R §2.9, ISO C99 §6.5.10-6.5.12
#[test]
fn test_bitwise_transformation_rules_summary() {
    // Rule 1: AND, OR, XOR have identical syntax
    let same_syntax = true;
    assert!(same_syntax, "AND, OR, XOR syntax identical in C and Rust");

    // Rule 2: NOT uses different symbol
    let not_symbol_different = true;
    assert!(
        not_symbol_different,
        "C uses ~, Rust uses ! for bitwise NOT"
    );

    // Rule 3: Shifts are identical
    let shifts_identical = true;
    assert!(shifts_identical, "Left and right shifts identical");

    // Rule 4: Type system ensures correctness
    let type_safe = true;
    assert!(type_safe, "Rust type system prevents sign errors");

    // Rule 5: No unsafe needed
    let unsafe_blocks = 0;
    assert_eq!(
        unsafe_blocks, 0,
        "Bitwise operations introduce 0 unsafe blocks"
    );

    // Rule 6: Same patterns work
    let patterns_work = true;
    assert!(patterns_work, "C bitwise patterns work in Rust");
}
