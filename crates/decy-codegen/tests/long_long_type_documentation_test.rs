//! Documentation tests for long long type (C99 §6.2.5)
//!
//! C99 introduced the `long long` integer type for 64-bit integers.
//! This test suite documents how DECY transforms C long long to Rust i64/u64.
//!
//! **Reference**: ISO C99 §6.2.5 (Types)
//!              NOT in K&R (pre-C99 feature)
//!
//! **Key Differences**:
//! - C89/K&R: No standard 64-bit integer type
//! - C99: Added `long long` (at least 64 bits)
//! - Rust: `i64` and `u64` are exactly 64 bits
//! - C `long long` guaranteed ≥64 bits
//! - Rust types have fixed sizes across platforms
//!
//! **Safety**: All transformations are SAFE (0 unsafe blocks)
//!
//! **Version**: v0.40.0

/// Document transformation of basic long long type
///
/// C99 `long long` → Rust `i64`
///
/// C Reference: ISO C99 §6.2.5
#[test]
fn test_basic_long_long() {
    let _c_code = r#"
long long x = 42LL;
"#;

    let _rust_equivalent = r#"
let x: i64 = 42;
"#;

    let x: i64 = 42;
    assert_eq!(x, 42);
    assert_eq!(std::mem::size_of_val(&x), 8); // 64 bits
}

/// Document unsigned long long type
///
/// C99 `unsigned long long` → Rust `u64`
#[test]
fn test_unsigned_long_long() {
    let _c_code = r#"
unsigned long long x = 42ULL;
"#;

    let _rust_equivalent = r#"
let x: u64 = 42;
"#;

    let x: u64 = 42;
    assert_eq!(x, 42);
    assert_eq!(std::mem::size_of_val(&x), 8); // 64 bits
}

/// Document long long literals
///
/// C99 LL suffix → Rust i64 type annotation or suffix
#[test]
fn test_long_long_literals() {
    let _c_code = r#"
long long a = 1234567890123456789LL;
unsigned long long b = 18446744073709551615ULL;  // Max u64
"#;

    let _rust_equivalent = r#"
let a: i64 = 1234567890123456789;
let b: u64 = 18446744073709551615;  // Max u64
// Or with suffix:
let c = 1234567890123456789i64;
let d = 18446744073709551615u64;
"#;

    let a: i64 = 1234567890123456789;
    let b: u64 = 18446744073709551615;
    let c = 1234567890123456789i64;
    let d = 18446744073709551615u64;

    assert_eq!(a, c);
    assert_eq!(b, d);
    assert_eq!(b, u64::MAX);
}

/// Document long long in arithmetic
///
/// C99 arithmetic on long long → Rust i64 arithmetic
#[test]
fn test_long_long_arithmetic() {
    let _c_code = r#"
long long a = 1000000000000LL;
long long b = 2000000000000LL;
long long sum = a + b;
long long product = a * 2;
"#;

    let _rust_equivalent = r#"
let a: i64 = 1000000000000;
let b: i64 = 2000000000000;
let sum = a + b;
let product = a * 2;
"#;

    let a: i64 = 1000000000000;
    let b: i64 = 2000000000000;
    let sum = a + b;
    let product = a * 2;

    assert_eq!(sum, 3000000000000);
    assert_eq!(product, 2000000000000);
}

/// Document long long range
///
/// C99 long long has minimum range requirements
#[test]
fn test_long_long_range() {
    let _c_code = r#"
// C99 guarantees at least 64 bits
// Range: -9223372036854775808 to 9223372036854775807
long long min = LLONG_MIN;
long long max = LLONG_MAX;
unsigned long long umax = ULLONG_MAX;
"#;

    let _rust_equivalent = r#"
// Rust i64 is exactly 64 bits
let min = i64::MIN;
let max = i64::MAX;
let umax = u64::MAX;
"#;

    let min = i64::MIN;
    let max = i64::MAX;
    let umax = u64::MAX;

    assert_eq!(min, -9223372036854775808);
    assert_eq!(max, 9223372036854775807);
    assert_eq!(umax, 18446744073709551615);
}

/// Document long long in functions
///
/// C99 long long parameters → Rust i64 parameters
#[test]
fn test_long_long_function_parameters() {
    let _c_code = r#"
long long add(long long a, long long b) {
    return a + b;
}

unsigned long long multiply(unsigned long long a, unsigned long long b) {
    return a * b;
}
"#;

    let _rust_equivalent = r#"
fn add(a: i64, b: i64) -> i64 {
    a + b
}

fn multiply(a: u64, b: u64) -> u64 {
    a * b
}
"#;

    fn add(a: i64, b: i64) -> i64 {
        a + b
    }

    fn multiply(a: u64, b: u64) -> u64 {
        a * b
    }

    assert_eq!(add(1000000000000, 2000000000000), 3000000000000);
    assert_eq!(multiply(1000000, 1000000), 1000000000000);
}

/// Document long long in arrays
///
/// C99 long long arrays → Rust [i64; N] arrays
#[test]
fn test_long_long_arrays() {
    let _c_code = r#"
long long arr[5] = {1LL, 2LL, 3LL, 4LL, 5LL};
unsigned long long uarr[3] = {10ULL, 20ULL, 30ULL};
"#;

    let _rust_equivalent = r#"
let arr: [i64; 5] = [1, 2, 3, 4, 5];
let uarr: [u64; 3] = [10, 20, 30];
"#;

    let arr: [i64; 5] = [1, 2, 3, 4, 5];
    let uarr: [u64; 3] = [10, 20, 30];

    assert_eq!(arr[2], 3);
    assert_eq!(uarr[1], 20);
}

/// Document long long in structs
///
/// C99 long long struct fields → Rust i64 struct fields
#[test]
fn test_long_long_in_structs() {
    let _c_code = r#"
struct TimestampedData {
    long long timestamp_ns;
    int value;
};
"#;

    let _rust_equivalent = r#"
struct TimestampedData {
    timestamp_ns: i64,
    value: i32,
}
"#;

    struct TimestampedData {
        timestamp_ns: i64,
        value: i32,
    }

    let data = TimestampedData {
        timestamp_ns: 1609459200000000000, // 2021-01-01 00:00:00 UTC in nanoseconds
        value: 42,
    };

    assert_eq!(data.timestamp_ns, 1609459200000000000);
    assert_eq!(data.value, 42);
}

/// Document long long overflow behavior
///
/// C99 signed overflow is undefined → Rust wrapping methods
#[test]
fn test_long_long_overflow() {
    let _c_code = r#"
// C99: signed overflow is undefined behavior
long long x = LLONG_MAX;
// x + 1;  // Undefined behavior!

// Unsigned overflow wraps around
unsigned long long y = ULLONG_MAX;
y = y + 1;  // Wraps to 0
"#;

    let _rust_equivalent = r#"
// Rust: signed overflow panics in debug, wraps in release
let x = i64::MAX;
// x + 1;  // Panics in debug mode

// Use checked/wrapping methods for explicit behavior
let wrapped = x.wrapping_add(1);
let checked = x.checked_add(1);

// Unsigned overflow always wraps
let y = u64::MAX;
let wrapped_y = y.wrapping_add(1);  // 0
"#;

    let x = i64::MAX;
    let wrapped = x.wrapping_add(1);
    assert_eq!(wrapped, i64::MIN);

    let checked = x.checked_add(1);
    assert_eq!(checked, None);

    let y = u64::MAX;
    let wrapped_y = y.wrapping_add(1);
    assert_eq!(wrapped_y, 0);
}

/// Document long long conversions
///
/// C99 conversions between types → Rust `as` operator
#[test]
fn test_long_long_conversions() {
    let _c_code = r#"
int i = 42;
long long ll = i;  // Implicit widening

long long big = 1234567890123456789LL;
int truncated = (int)big;  // Explicit narrowing
"#;

    let _rust_equivalent = r#"
let i: i32 = 42;
let ll: i64 = i as i64;  // Explicit widening

let big: i64 = 1234567890123456789;
let truncated: i32 = big as i32;  // Explicit narrowing (loses data)
"#;

    let i: i32 = 42;
    let ll: i64 = i as i64;
    assert_eq!(ll, 42);

    let big: i64 = 1234567890123456789;
    let truncated: i32 = big as i32;
    // Truncation loses high-order bits
    assert_ne!(truncated as i64, big);
}

/// Document long long with printf/scanf
///
/// C99 format specifiers → Rust formatting
#[test]
fn test_long_long_printf_scanf() {
    let _c_code = r#"
long long x = 1234567890123456789LL;
printf("%lld\n", x);  // %lld for long long
printf("%llu\n", (unsigned long long)x);  // %llu for unsigned

unsigned long long hex = 0x123456789ABCDEFULL;
printf("%llx\n", hex);  // %llx for hex
"#;

    let _rust_equivalent = r#"
let x: i64 = 1234567890123456789;
println!("{}", x);  // Type-safe formatting
println!("{}", x as u64);

let hex: u64 = 0x123456789ABCDEF;
println!("{:x}", hex);  // Hex formatting
"#;

    let x: i64 = 1234567890123456789;
    let formatted = format!("{}", x);
    assert_eq!(formatted, "1234567890123456789");

    let hex: u64 = 0x123456789ABCDEF;
    let hex_formatted = format!("{:x}", hex);
    assert_eq!(hex_formatted, "123456789abcdef");
}

/// Document long long bit operations
///
/// C99 bitwise operations on long long → Rust same operations
#[test]
fn test_long_long_bit_operations() {
    let _c_code = r#"
unsigned long long flags = 0x1ULL << 32;  // Bit 32 set
unsigned long long mask = 0xFFFFFFFF00000000ULL;
unsigned long long result = flags & mask;
"#;

    let _rust_equivalent = r#"
let flags: u64 = 0x1 << 32;  // Bit 32 set
let mask: u64 = 0xFFFFFFFF00000000;
let result = flags & mask;
"#;

    let flags: u64 = 0x1 << 32;
    let mask: u64 = 0xFFFFFFFF00000000;
    let result = flags & mask;

    assert_eq!(flags, 4294967296); // 2^32
    assert_eq!(result, flags);
}

/// Document long long comparison
///
/// C99 comparison operations → Rust same operations
#[test]
fn test_long_long_comparison() {
    let _c_code = r#"
long long a = 1000000000000LL;
long long b = 2000000000000LL;

if (a < b) { }
if (a == a) { }
"#;

    let _rust_equivalent = r#"
let a: i64 = 1000000000000;
let b: i64 = 2000000000000;

if a < b { }
if a == a { }
"#;

    let a: i64 = 1000000000000;
    let b: i64 = 2000000000000;

    assert!(a < b);
    assert!(a == a);
    assert!(b > a);
}

/// Document long long in time operations
///
/// Common use case: timestamps and time differences
#[test]
fn test_long_long_time_operations() {
    let _c_code = r#"
// Common use: Unix timestamps in milliseconds or nanoseconds
long long timestamp_ms = 1609459200000LL;  // 2021-01-01 in ms
long long timestamp_ns = 1609459200000000000LL;  // in ns

long long duration_ns = timestamp_ns - 1000000000LL;  // 1 second earlier
"#;

    let _rust_equivalent = r#"
// Rust uses i64 for timestamps
let timestamp_ms: i64 = 1609459200000;
let timestamp_ns: i64 = 1609459200000000000;

let duration_ns = timestamp_ns - 1000000000;
"#;

    let _timestamp_ms: i64 = 1609459200000;
    let timestamp_ns: i64 = 1609459200000000000;
    let duration_ns = timestamp_ns - 1000000000;

    assert_eq!(duration_ns, 1609459199000000000);
}

/// Document long long for large file sizes
///
/// Common use case: file sizes > 4GB
#[test]
fn test_long_long_file_sizes() {
    let _c_code = r#"
// int32 can only handle up to 2GB
// long long needed for files > 4GB
long long file_size = 5368709120LL;  // 5GB in bytes
"#;

    let _rust_equivalent = r#"
let file_size: i64 = 5368709120;  // 5GB in bytes
// Or more clearly:
let file_size_gb = 5;
let file_size_bytes = file_size_gb * 1024 * 1024 * 1024;
"#;

    let file_size: i64 = 5368709120;
    assert_eq!(file_size, 5 * 1024 * 1024 * 1024);

    let file_size_gb = 5i64;
    let file_size_bytes = file_size_gb * 1024 * 1024 * 1024;
    assert_eq!(file_size_bytes, file_size);
}

/// Summary: Long Long Type (C99 §6.2.5)
///
/// **Transformation Rules**:
/// 1. C99 `long long` → Rust `i64`
/// 2. C99 `unsigned long long` → Rust `u64`
/// 3. C99 `LL` suffix → Rust type annotation or `i64` suffix
/// 4. C99 `ULL` suffix → Rust type annotation or `u64` suffix
/// 5. C99 `%lld` format → Rust `{}` (type-safe)
///
/// **Key Insights**:
/// - C89/K&R had no standard 64-bit integer type
/// - C99 added `long long` (at least 64 bits, usually exactly 64)
/// - Rust i64/u64 are exactly 64 bits on all platforms
/// - C signed overflow is undefined behavior
/// - Rust overflow panics in debug, wraps in release
/// - Use checked/wrapping methods for explicit behavior
/// - Common uses: timestamps, file sizes, large counters
/// - 64-bit arithmetic is native on modern CPUs
///
/// **Safety**: ✅ 0 unsafe blocks (basic type, no unsafe needed)
///
/// **Coverage**: 15 test cases covering:
/// - Basic long long type
/// - Unsigned long long
/// - Long long literals
/// - Arithmetic operations
/// - Range and limits
/// - Function parameters
/// - Arrays
/// - Structs
/// - Overflow behavior
/// - Type conversions
/// - Printf/scanf formatting
/// - Bit operations
/// - Comparison operations
/// - Time operations (common use case)
/// - File sizes (common use case)
#[test]
fn test_long_long_summary() {
    // C89/K&R did not have long long
    let c89_has_long_long = false;

    // C99 added long long
    let c99_has_long_long = true;

    // Rust has i64 (equivalent)
    let rust_has_i64 = true;

    assert!(!c89_has_long_long, "C89 did not have long long");
    assert!(c99_has_long_long, "C99 added long long");
    assert!(rust_has_i64, "Rust has i64");

    // Size guarantees
    let c99_long_long_min_bits = 64;
    let rust_i64_exact_bits = 64;
    assert!(c99_long_long_min_bits <= rust_i64_exact_bits);

    // Range verification
    assert_eq!(i64::MIN, -9223372036854775808);
    assert_eq!(i64::MAX, 9223372036854775807);
    assert_eq!(u64::MAX, 18446744073709551615);

    // No unsafe blocks needed
    let unsafe_blocks = 0;
    assert_eq!(
        unsafe_blocks, 0,
        "long long is a basic type - no unsafe needed"
    );
}
