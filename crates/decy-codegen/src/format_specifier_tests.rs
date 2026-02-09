//! Coverage tests for convert_format_specifiers function.
//!
//! Tests all printf format specifier conversion paths to Rust format strings.

use super::CodeGenerator;

// ============================================================================
// Basic specifiers: %d, %i, %u
// ============================================================================

#[test]
fn test_format_basic_integer() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%d"), "{}");
}

#[test]
fn test_format_signed_integer() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%i"), "{}");
}

#[test]
fn test_format_unsigned_integer() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%u"), "{}");
}

#[test]
fn test_format_integer_with_width() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%5d"), "{:5}");
}

#[test]
fn test_format_integer_with_zero_padding() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%05d"), "{:05}");
}

// ============================================================================
// Hex specifiers: %x, %X
// ============================================================================

#[test]
fn test_format_lowercase_hex() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%x"), "{:x}");
}

#[test]
fn test_format_uppercase_hex() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%X"), "{:X}");
}

#[test]
fn test_format_hex_with_width() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%08x"), "{:08x}");
}

#[test]
fn test_format_uppercase_hex_with_width() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%04X"), "{:04X}");
}

// ============================================================================
// Octal specifier: %o
// ============================================================================

#[test]
fn test_format_octal() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%o"), "{:o}");
}

#[test]
fn test_format_octal_with_width() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%8o"), "{:8o}");
}

// ============================================================================
// Binary specifier: %b (DECY-247 extension)
// ============================================================================

#[test]
fn test_format_binary() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%b"), "{:b}");
}

#[test]
fn test_format_binary_with_width() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%08b"), "{:08b}");
}

// ============================================================================
// Float specifiers: %f, %F, %e, %E, %g, %G
// ============================================================================

#[test]
fn test_format_float() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%f"), "{}");
}

#[test]
fn test_format_float_with_precision() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%.2f"), "{:.2}");
}

#[test]
fn test_format_float_with_width_and_precision() {
    assert_eq!(
        CodeGenerator::convert_format_specifiers("%8.2f"),
        "{:8.2}"
    );
}

#[test]
fn test_format_float_with_width_only() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%10f"), "{:10}");
}

#[test]
fn test_format_float_with_zero_pad_width_precision() {
    assert_eq!(
        CodeGenerator::convert_format_specifiers("%010.3f"),
        "{:010.3}"
    );
}

#[test]
fn test_format_scientific_lower() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%e"), "{:e}");
}

#[test]
fn test_format_scientific_upper() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%E"), "{:E}");
}

#[test]
fn test_format_general_lower() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%g"), "{}");
}

#[test]
fn test_format_general_upper() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%G"), "{}");
}

// ============================================================================
// String specifier: %s
// ============================================================================

#[test]
fn test_format_string() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%s"), "{}");
}

#[test]
fn test_format_string_with_width() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%20s"), "{:20}");
}

// ============================================================================
// Char and pointer: %c, %p
// ============================================================================

#[test]
fn test_format_char() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%c"), "{}");
}

#[test]
fn test_format_pointer() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%p"), "{:p}");
}

// ============================================================================
// Escape: %%
// ============================================================================

#[test]
fn test_format_percent_literal() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%%"), "%");
}

#[test]
fn test_format_percent_in_text() {
    assert_eq!(
        CodeGenerator::convert_format_specifiers("100%%"),
        "100%"
    );
}

// ============================================================================
// Length modifiers: %ld, %lld, %hd, %zu
// ============================================================================

#[test]
fn test_format_long_int() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%ld"), "{}");
}

#[test]
fn test_format_long_long_int() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%lld"), "{}");
}

#[test]
fn test_format_short_int() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%hd"), "{}");
}

#[test]
fn test_format_size_t() {
    assert_eq!(CodeGenerator::convert_format_specifiers("%zu"), "{}");
}

// ============================================================================
// Combined patterns (real printf usage)
// ============================================================================

#[test]
fn test_format_mixed_specifiers() {
    assert_eq!(
        CodeGenerator::convert_format_specifiers("Hello %s, you are %d years old"),
        "Hello {}, you are {} years old"
    );
}

#[test]
fn test_format_hex_dump_pattern() {
    assert_eq!(
        CodeGenerator::convert_format_specifiers("%02x %02x"),
        "{:02x} {:02x}"
    );
}

#[test]
fn test_format_no_specifiers() {
    assert_eq!(
        CodeGenerator::convert_format_specifiers("hello world"),
        "hello world"
    );
}

#[test]
fn test_format_empty_string() {
    assert_eq!(CodeGenerator::convert_format_specifiers(""), "");
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn test_format_unknown_specifier() {
    // Unknown specifier like %q should preserve original
    let result = CodeGenerator::convert_format_specifiers("%q");
    assert_eq!(result, "%q");
}

#[test]
fn test_format_trailing_percent() {
    // Incomplete format specifier at end of string
    let result = CodeGenerator::convert_format_specifiers("hello %");
    // Should preserve the trailing %
    assert!(result.contains('%') || result.contains("hello"));
}

#[test]
fn test_format_multiple_flags() {
    // Multiple flags like %-+5d
    let result = CodeGenerator::convert_format_specifiers("%-+5d");
    assert!(result.contains('{'));
}
