//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1201-C1225: Bitwise Operations & Bit Manipulation -- the kind of C code found
//! in embedded firmware, compression engines, hash functions, and performance-critical
//! systems code where bit-level control is essential.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world bit manipulation patterns commonly found in
//! Linux kernel, GCC builtins, Hacker's Delight algorithms, chess engines,
//! and high-performance computing -- all expressed as valid C99.
//!
//! Organization:
//! - C1201-C1205: Basic bit ops (popcount, parity, reverse bits, trailing zeros, leading zeros)
//! - C1206-C1210: Bit tricks (power of 2, next power of 2, Morton/interleave, Gray code, de Bruijn)
//! - C1211-C1215: Bitfield manipulation (set/clear/toggle ranges, extract, deposit, rotate, permute)
//! - C1216-C1220: SIMD-style (parallel prefix, horizontal add, butterfly, bit matrix transpose, saturating add)
//! - C1221-C1225: Applied (CRC32 bitwise, Hamming code, bit-parallel string match, compact bitboard, Fenwick bit tree)

// ============================================================================
// C1201-C1205: Basic Bit Operations
// ============================================================================

#[test]
fn c1201_population_count_multiple_methods() {
    let c_code = r#"
typedef unsigned int uint32_t;

int bit_popcount_naive(uint32_t x) {
    int count = 0;
    while (x) {
        count += x & 1;
        x >>= 1;
    }
    return count;
}

int bit_popcount_kernighan(uint32_t x) {
    int count = 0;
    while (x) {
        x &= (x - 1);
        count++;
    }
    return count;
}

int bit_popcount_parallel(uint32_t x) {
    x = x - ((x >> 1) & 0x55555555);
    x = (x & 0x33333333) + ((x >> 2) & 0x33333333);
    x = (x + (x >> 4)) & 0x0F0F0F0F;
    return (int)((x * 0x01010101) >> 24);
}

int bit_popcount_selftest(void) {
    uint32_t val = 0xDEADBEEF;
    int n = bit_popcount_naive(val);
    int k = bit_popcount_kernighan(val);
    int p = bit_popcount_parallel(val);
    if (n != k || k != p) return -1;
    if (bit_popcount_naive(0) != 0) return -2;
    if (bit_popcount_naive(0xFFFFFFFF) != 32) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1201: Population count multiple methods should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1201: Output should not be empty");
    assert!(
        code.contains("fn bit_popcount_naive"),
        "C1201: Should contain bit_popcount_naive function"
    );
    assert!(
        code.contains("fn bit_popcount_kernighan"),
        "C1201: Should contain bit_popcount_kernighan function"
    );
    assert!(
        code.contains("fn bit_popcount_parallel"),
        "C1201: Should contain bit_popcount_parallel function"
    );
}

#[test]
fn c1202_parity_computation() {
    let c_code = r#"
typedef unsigned int uint32_t;

int bit_parity_naive(uint32_t x) {
    int parity = 0;
    while (x) {
        parity ^= (x & 1);
        x >>= 1;
    }
    return parity;
}

int bit_parity_fold(uint32_t x) {
    x ^= x >> 16;
    x ^= x >> 8;
    x ^= x >> 4;
    x ^= x >> 2;
    x ^= x >> 1;
    return (int)(x & 1);
}

int bit_parity_kernighan(uint32_t x) {
    int parity = 0;
    while (x) {
        parity ^= 1;
        x &= (x - 1);
    }
    return parity;
}

int bit_parity_selftest(void) {
    uint32_t test_vals[4];
    int expected[4];
    int i;
    test_vals[0] = 0x00000000; expected[0] = 0;
    test_vals[1] = 0x00000001; expected[1] = 1;
    test_vals[2] = 0x00000003; expected[2] = 0;
    test_vals[3] = 0xFFFFFFFF; expected[3] = 0;
    for (i = 0; i < 4; i++) {
        if (bit_parity_naive(test_vals[i]) != expected[i]) return -(i + 1);
        if (bit_parity_fold(test_vals[i]) != expected[i]) return -(i + 10);
        if (bit_parity_kernighan(test_vals[i]) != expected[i]) return -(i + 20);
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1202: Parity computation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1202: Output should not be empty");
    assert!(
        code.contains("fn bit_parity_naive"),
        "C1202: Should contain bit_parity_naive function"
    );
    assert!(
        code.contains("fn bit_parity_fold"),
        "C1202: Should contain bit_parity_fold function"
    );
}

#[test]
fn c1203_reverse_bits() {
    let c_code = r#"
typedef unsigned int uint32_t;

uint32_t bit_reverse_naive(uint32_t x) {
    uint32_t result = 0;
    int i;
    for (i = 0; i < 32; i++) {
        result = (result << 1) | (x & 1);
        x >>= 1;
    }
    return result;
}

uint32_t bit_reverse_parallel(uint32_t x) {
    x = ((x & 0x55555555) << 1)  | ((x >> 1)  & 0x55555555);
    x = ((x & 0x33333333) << 2)  | ((x >> 2)  & 0x33333333);
    x = ((x & 0x0F0F0F0F) << 4)  | ((x >> 4)  & 0x0F0F0F0F);
    x = ((x & 0x00FF00FF) << 8)  | ((x >> 8)  & 0x00FF00FF);
    x = (x << 16) | (x >> 16);
    return x;
}

int bit_reverse_selftest(void) {
    uint32_t val = 0x80000000;
    if (bit_reverse_naive(val) != 0x00000001) return -1;
    if (bit_reverse_parallel(val) != 0x00000001) return -2;
    if (bit_reverse_naive(0) != 0) return -3;
    if (bit_reverse_naive(bit_reverse_naive(0xDEADBEEF)) != 0xDEADBEEF) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1203: Reverse bits should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1203: Output should not be empty");
    assert!(
        code.contains("fn bit_reverse_naive"),
        "C1203: Should contain bit_reverse_naive function"
    );
    assert!(
        code.contains("fn bit_reverse_parallel"),
        "C1203: Should contain bit_reverse_parallel function"
    );
}

#[test]
fn c1204_count_trailing_zeros() {
    let c_code = r#"
typedef unsigned int uint32_t;

int bit_ctz_naive(uint32_t x) {
    int count = 0;
    if (x == 0) return 32;
    while ((x & 1) == 0) {
        count++;
        x >>= 1;
    }
    return count;
}

int bit_ctz_debruijn(uint32_t x) {
    static const int debruijn_table[32] = {
        0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8,
        31, 27, 13, 23, 21, 19, 16, 7, 26, 12, 18, 6, 11, 5, 10, 9
    };
    if (x == 0) return 32;
    return debruijn_table[((uint32_t)((x & (0 - x)) * 0x077CB531)) >> 27];
}

int bit_ctz_binary_search(uint32_t x) {
    int n = 0;
    if (x == 0) return 32;
    if ((x & 0x0000FFFF) == 0) { n += 16; x >>= 16; }
    if ((x & 0x000000FF) == 0) { n +=  8; x >>=  8; }
    if ((x & 0x0000000F) == 0) { n +=  4; x >>=  4; }
    if ((x & 0x00000003) == 0) { n +=  2; x >>=  2; }
    if ((x & 0x00000001) == 0) { n +=  1; }
    return n;
}

int bit_ctz_selftest(void) {
    if (bit_ctz_naive(0) != 32) return -1;
    if (bit_ctz_naive(1) != 0) return -2;
    if (bit_ctz_naive(0x80000000) != 31) return -3;
    if (bit_ctz_debruijn(16) != 4) return -4;
    if (bit_ctz_binary_search(256) != 8) return -5;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1204: Count trailing zeros should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1204: Output should not be empty");
    assert!(
        code.contains("fn bit_ctz_naive"),
        "C1204: Should contain bit_ctz_naive function"
    );
    assert!(
        code.contains("fn bit_ctz_debruijn"),
        "C1204: Should contain bit_ctz_debruijn function"
    );
}

#[test]
fn c1205_count_leading_zeros() {
    let c_code = r#"
typedef unsigned int uint32_t;

int bit_clz_naive(uint32_t x) {
    int count = 0;
    if (x == 0) return 32;
    while ((x & 0x80000000) == 0) {
        count++;
        x <<= 1;
    }
    return count;
}

int bit_clz_binary_search(uint32_t x) {
    int n = 0;
    if (x == 0) return 32;
    if (x <= 0x0000FFFF) { n += 16; x <<= 16; }
    if (x <= 0x00FFFFFF) { n +=  8; x <<=  8; }
    if (x <= 0x0FFFFFFF) { n +=  4; x <<=  4; }
    if (x <= 0x3FFFFFFF) { n +=  2; x <<=  2; }
    if (x <= 0x7FFFFFFF) { n +=  1; }
    return n;
}

int bit_floor_log2(uint32_t x) {
    if (x == 0) return -1;
    return 31 - bit_clz_binary_search(x);
}

int bit_clz_selftest(void) {
    if (bit_clz_naive(0) != 32) return -1;
    if (bit_clz_naive(1) != 31) return -2;
    if (bit_clz_naive(0x80000000) != 0) return -3;
    if (bit_floor_log2(1) != 0) return -4;
    if (bit_floor_log2(256) != 8) return -5;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1205: Count leading zeros should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1205: Output should not be empty");
    assert!(
        code.contains("fn bit_clz_naive"),
        "C1205: Should contain bit_clz_naive function"
    );
    assert!(
        code.contains("fn bit_clz_binary_search"),
        "C1205: Should contain bit_clz_binary_search function"
    );
}

// ============================================================================
// C1206-C1210: Bit Tricks
// ============================================================================

#[test]
fn c1206_power_of_two_check() {
    let c_code = r#"
typedef unsigned int uint32_t;

int bit_is_power_of_2(uint32_t x) {
    return (x != 0) && ((x & (x - 1)) == 0);
}

uint32_t bit_round_down_power_of_2(uint32_t x) {
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    return x - (x >> 1);
}

int bit_is_power_of_2_signed(int x) {
    if (x <= 0) return 0;
    return (x & (x - 1)) == 0;
}

uint32_t bit_isolate_lowest_set(uint32_t x) {
    return x & (0 - x);
}

uint32_t bit_turn_off_lowest_set(uint32_t x) {
    return x & (x - 1);
}

int bit_power2_selftest(void) {
    if (!bit_is_power_of_2(1)) return -1;
    if (!bit_is_power_of_2(1024)) return -2;
    if (bit_is_power_of_2(0)) return -3;
    if (bit_is_power_of_2(6)) return -4;
    if (bit_round_down_power_of_2(13) != 8) return -5;
    if (bit_isolate_lowest_set(0x30) != 0x10) return -6;
    if (bit_turn_off_lowest_set(0x30) != 0x20) return -7;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1206: Power of two check should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1206: Output should not be empty");
    assert!(
        code.contains("fn bit_is_power_of_2"),
        "C1206: Should contain bit_is_power_of_2 function"
    );
    assert!(
        code.contains("fn bit_round_down_power_of_2"),
        "C1206: Should contain bit_round_down_power_of_2 function"
    );
}

#[test]
fn c1207_next_power_of_two() {
    let c_code = r#"
typedef unsigned int uint32_t;

uint32_t bit_next_power_of_2(uint32_t x) {
    if (x == 0) return 1;
    x--;
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    x++;
    return x;
}

uint32_t bit_align_up(uint32_t value, uint32_t alignment) {
    uint32_t mask = alignment - 1;
    return (value + mask) & ~mask;
}

uint32_t bit_align_down(uint32_t value, uint32_t alignment) {
    return value & ~(alignment - 1);
}

int bit_is_aligned(uint32_t value, uint32_t alignment) {
    return (value & (alignment - 1)) == 0;
}

int bit_next_pow2_selftest(void) {
    if (bit_next_power_of_2(0) != 1) return -1;
    if (bit_next_power_of_2(1) != 1) return -2;
    if (bit_next_power_of_2(5) != 8) return -3;
    if (bit_next_power_of_2(16) != 16) return -4;
    if (bit_align_up(100, 64) != 128) return -5;
    if (bit_align_down(100, 64) != 64) return -6;
    if (!bit_is_aligned(256, 16)) return -7;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1207: Next power of two should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1207: Output should not be empty");
    assert!(
        code.contains("fn bit_next_power_of_2"),
        "C1207: Should contain bit_next_power_of_2 function"
    );
    assert!(
        code.contains("fn bit_align_up"),
        "C1207: Should contain bit_align_up function"
    );
}

#[test]
fn c1208_morton_code_interleave() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

uint32_t bit_spread_bits(uint32_t x) {
    x = (x | (x << 8)) & 0x00FF00FF;
    x = (x | (x << 4)) & 0x0F0F0F0F;
    x = (x | (x << 2)) & 0x33333333;
    x = (x | (x << 1)) & 0x55555555;
    return x;
}

uint32_t bit_compact_bits(uint32_t x) {
    x &= 0x55555555;
    x = (x | (x >> 1)) & 0x33333333;
    x = (x | (x >> 2)) & 0x0F0F0F0F;
    x = (x | (x >> 4)) & 0x00FF00FF;
    x = (x | (x >> 8)) & 0x0000FFFF;
    return x;
}

uint32_t bit_morton_encode_2d(uint32_t x, uint32_t y) {
    return (bit_spread_bits(x) | (bit_spread_bits(y) << 1));
}

void bit_morton_decode_2d(uint32_t code, uint32_t *x, uint32_t *y) {
    *x = bit_compact_bits(code);
    *y = bit_compact_bits(code >> 1);
}

int bit_morton_selftest(void) {
    uint32_t code, rx, ry;
    code = bit_morton_encode_2d(5, 9);
    bit_morton_decode_2d(code, &rx, &ry);
    if (rx != 5 || ry != 9) return -1;
    code = bit_morton_encode_2d(0, 0);
    if (code != 0) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1208: Morton code interleave should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1208: Output should not be empty");
    assert!(
        code.contains("fn bit_morton_encode_2d"),
        "C1208: Should contain bit_morton_encode_2d function"
    );
    assert!(
        code.contains("fn bit_morton_decode_2d"),
        "C1208: Should contain bit_morton_decode_2d function"
    );
}

#[test]
fn c1209_gray_code_conversion() {
    let c_code = r#"
typedef unsigned int uint32_t;

uint32_t bit_to_gray(uint32_t n) {
    return n ^ (n >> 1);
}

uint32_t bit_from_gray(uint32_t gray) {
    uint32_t n = gray;
    n ^= n >> 16;
    n ^= n >> 8;
    n ^= n >> 4;
    n ^= n >> 2;
    n ^= n >> 1;
    return n;
}

int bit_gray_next_differs_by_one(uint32_t a, uint32_t b) {
    uint32_t diff = a ^ b;
    return diff != 0 && (diff & (diff - 1)) == 0;
}

int bit_gray_selftest(void) {
    uint32_t i;
    for (i = 0; i < 256; i++) {
        uint32_t g = bit_to_gray(i);
        uint32_t back = bit_from_gray(g);
        if (back != i) return -1;
    }
    for (i = 0; i < 255; i++) {
        uint32_t g1 = bit_to_gray(i);
        uint32_t g2 = bit_to_gray(i + 1);
        if (!bit_gray_next_differs_by_one(g1, g2)) return -2;
    }
    if (bit_to_gray(0) != 0) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1209: Gray code conversion should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1209: Output should not be empty");
    assert!(
        code.contains("fn bit_to_gray"),
        "C1209: Should contain bit_to_gray function"
    );
    assert!(
        code.contains("fn bit_from_gray"),
        "C1209: Should contain bit_from_gray function"
    );
}

#[test]
fn c1210_debruijn_sequence_lookup() {
    let c_code = r#"
typedef unsigned int uint32_t;

static const int bit_debruijn_ctz_table[32] = {
    0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8,
    31, 27, 13, 23, 21, 19, 16, 7, 26, 12, 18, 6, 11, 5, 10, 9
};

static const int bit_debruijn_clz_table[32] = {
    31, 22, 30, 21, 18, 10, 29, 2, 20, 17, 15, 13, 9, 6, 28, 1,
    23, 19, 11, 3, 16, 14, 7, 24, 12, 4, 8, 25, 5, 26, 27, 0
};

int bit_debruijn_ctz(uint32_t x) {
    if (x == 0) return 32;
    return bit_debruijn_ctz_table[((uint32_t)((x & (0 - x)) * 0x077CB531)) >> 27];
}

int bit_debruijn_clz(uint32_t x) {
    if (x == 0) return 32;
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    return bit_debruijn_clz_table[(uint32_t)(x * 0x07C4ACDD) >> 27];
}

int bit_debruijn_log2(uint32_t x) {
    if (x == 0) return -1;
    return 31 - bit_debruijn_clz(x);
}

int bit_debruijn_selftest(void) {
    if (bit_debruijn_ctz(1) != 0) return -1;
    if (bit_debruijn_ctz(8) != 3) return -2;
    if (bit_debruijn_clz(0x80000000) != 0) return -3;
    if (bit_debruijn_clz(1) != 31) return -4;
    if (bit_debruijn_log2(1) != 0) return -5;
    if (bit_debruijn_log2(1024) != 10) return -6;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1210: de Bruijn sequence lookup should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1210: Output should not be empty");
    assert!(
        code.contains("fn bit_debruijn_ctz"),
        "C1210: Should contain bit_debruijn_ctz function"
    );
    assert!(
        code.contains("fn bit_debruijn_clz"),
        "C1210: Should contain bit_debruijn_clz function"
    );
}

// ============================================================================
// C1211-C1215: Bitfield Manipulation
// ============================================================================

#[test]
fn c1211_set_clear_toggle_bit_ranges() {
    let c_code = r#"
typedef unsigned int uint32_t;

uint32_t bit_set_range(uint32_t x, int low, int high) {
    uint32_t mask = ((1u << (high - low + 1)) - 1) << low;
    return x | mask;
}

uint32_t bit_clear_range(uint32_t x, int low, int high) {
    uint32_t mask = ((1u << (high - low + 1)) - 1) << low;
    return x & ~mask;
}

uint32_t bit_toggle_range(uint32_t x, int low, int high) {
    uint32_t mask = ((1u << (high - low + 1)) - 1) << low;
    return x ^ mask;
}

int bit_test_bit(uint32_t x, int pos) {
    return (int)((x >> pos) & 1);
}

uint32_t bit_modify_bit(uint32_t x, int pos, int val) {
    uint32_t mask = 1u << pos;
    return (x & ~mask) | ((uint32_t)val << pos);
}

int bit_range_selftest(void) {
    uint32_t val = 0;
    val = bit_set_range(val, 4, 7);
    if (val != 0xF0) return -1;
    val = bit_clear_range(val, 5, 6);
    if (val != 0x90) return -2;
    val = bit_toggle_range(0xFF, 2, 5);
    if (val != 0xC3) return -3;
    if (bit_test_bit(0x80, 7) != 1) return -4;
    if (bit_test_bit(0x80, 6) != 0) return -5;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1211: Set/clear/toggle bit ranges should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1211: Output should not be empty");
    assert!(
        code.contains("fn bit_set_range"),
        "C1211: Should contain bit_set_range function"
    );
    assert!(
        code.contains("fn bit_clear_range"),
        "C1211: Should contain bit_clear_range function"
    );
}

#[test]
fn c1212_extract_and_deposit_bitfield() {
    let c_code = r#"
typedef unsigned int uint32_t;

uint32_t bit_extract_field(uint32_t x, int start, int width) {
    return (x >> start) & ((1u << width) - 1);
}

uint32_t bit_deposit_field(uint32_t x, uint32_t field, int start, int width) {
    uint32_t mask = ((1u << width) - 1) << start;
    return (x & ~mask) | ((field << start) & mask);
}

uint32_t bit_sign_extend(uint32_t x, int bits) {
    uint32_t m = 1u << (bits - 1);
    x &= (1u << bits) - 1;
    return (x ^ m) - m;
}

uint32_t bit_zero_extend(uint32_t x, int bits) {
    return x & ((1u << bits) - 1);
}

int bit_field_selftest(void) {
    uint32_t val = 0xDEADBEEF;
    uint32_t field = bit_extract_field(val, 8, 8);
    if (field != 0xBE) return -1;
    uint32_t modified = bit_deposit_field(0, 0xFF, 16, 8);
    if (modified != 0x00FF0000) return -2;
    uint32_t se = bit_sign_extend(0xF0, 8);
    if ((int)se != -16) return -3;
    if (bit_zero_extend(0xFFFF, 8) != 0xFF) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1212: Extract and deposit bitfield should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1212: Output should not be empty");
    assert!(
        code.contains("fn bit_extract_field"),
        "C1212: Should contain bit_extract_field function"
    );
    assert!(
        code.contains("fn bit_deposit_field"),
        "C1212: Should contain bit_deposit_field function"
    );
}

#[test]
fn c1213_bit_rotate_operations() {
    let c_code = r#"
typedef unsigned int uint32_t;

uint32_t bit_rotl32(uint32_t x, int n) {
    n &= 31;
    return (x << n) | (x >> (32 - n));
}

uint32_t bit_rotr32(uint32_t x, int n) {
    n &= 31;
    return (x >> n) | (x << (32 - n));
}

uint32_t bit_double_rotate(uint32_t hi, uint32_t lo, int n) {
    uint32_t result;
    n &= 63;
    if (n < 32) {
        result = (hi << n) | (lo >> (32 - n));
    } else {
        result = (lo << (n - 32)) | (hi >> (64 - n));
    }
    return result;
}

uint32_t bit_funnel_shift(uint32_t hi, uint32_t lo, int n) {
    n &= 31;
    if (n == 0) return hi;
    return (hi << n) | (lo >> (32 - n));
}

int bit_rotate_selftest(void) {
    uint32_t x = 0x80000001;
    if (bit_rotl32(x, 1) != 0x00000003) return -1;
    if (bit_rotr32(x, 1) != 0xC0000000) return -2;
    if (bit_rotl32(x, 0) != x) return -3;
    if (bit_rotr32(bit_rotl32(x, 13), 13) != x) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1213: Bit rotate operations should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1213: Output should not be empty");
    assert!(
        code.contains("fn bit_rotl32"),
        "C1213: Should contain bit_rotl32 function"
    );
    assert!(
        code.contains("fn bit_rotr32"),
        "C1213: Should contain bit_rotr32 function"
    );
}

#[test]
fn c1214_bit_permutation() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

uint32_t bit_permute_by_table(uint32_t x, const uint8_t *table, int nbits) {
    uint32_t result = 0;
    int i;
    for (i = 0; i < nbits; i++) {
        if (x & (1u << table[i])) {
            result |= (1u << i);
        }
    }
    return result;
}

uint32_t bit_swap_bits(uint32_t x, int i, int j) {
    uint32_t bit_i = (x >> i) & 1;
    uint32_t bit_j = (x >> j) & 1;
    if (bit_i != bit_j) {
        x ^= (1u << i) | (1u << j);
    }
    return x;
}

uint32_t bit_reverse_nibbles(uint32_t x) {
    x = ((x & 0x0F0F0F0F) << 4) | ((x >> 4) & 0x0F0F0F0F);
    return x;
}

uint32_t bit_swap_bytes(uint32_t x) {
    return ((x & 0x000000FF) << 24) |
           ((x & 0x0000FF00) << 8)  |
           ((x & 0x00FF0000) >> 8)  |
           ((x & 0xFF000000) >> 24);
}

int bit_permute_selftest(void) {
    uint32_t x = 0x12345678;
    uint32_t swapped = bit_swap_bytes(x);
    if (swapped != 0x78563412) return -1;
    uint32_t s = bit_swap_bits(0x05, 0, 2);
    if (s != 0x06) return -2;
    uint32_t rev = bit_reverse_nibbles(0xAB);
    if (rev != 0xBA) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1214: Bit permutation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1214: Output should not be empty");
    assert!(
        code.contains("fn bit_permute_by_table"),
        "C1214: Should contain bit_permute_by_table function"
    );
    assert!(
        code.contains("fn bit_swap_bytes"),
        "C1214: Should contain bit_swap_bytes function"
    );
}

#[test]
fn c1215_bitfield_struct_packing() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint32_t packed;
} bit_packed_color_t;

bit_packed_color_t bit_pack_rgba(uint8_t r, uint8_t g, uint8_t b, uint8_t a) {
    bit_packed_color_t c;
    c.packed = ((uint32_t)a << 24) | ((uint32_t)r << 16) |
               ((uint32_t)g << 8) | (uint32_t)b;
    return c;
}

uint8_t bit_unpack_red(bit_packed_color_t c) {
    return (uint8_t)((c.packed >> 16) & 0xFF);
}

uint8_t bit_unpack_green(bit_packed_color_t c) {
    return (uint8_t)((c.packed >> 8) & 0xFF);
}

uint8_t bit_unpack_blue(bit_packed_color_t c) {
    return (uint8_t)(c.packed & 0xFF);
}

uint8_t bit_unpack_alpha(bit_packed_color_t c) {
    return (uint8_t)((c.packed >> 24) & 0xFF);
}

int bit_pack_selftest(void) {
    bit_packed_color_t c = bit_pack_rgba(255, 128, 64, 200);
    if (bit_unpack_red(c) != 255) return -1;
    if (bit_unpack_green(c) != 128) return -2;
    if (bit_unpack_blue(c) != 64) return -3;
    if (bit_unpack_alpha(c) != 200) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1215: Bitfield struct packing should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1215: Output should not be empty");
    assert!(
        code.contains("fn bit_pack_rgba"),
        "C1215: Should contain bit_pack_rgba function"
    );
    assert!(
        code.contains("fn bit_unpack_red"),
        "C1215: Should contain bit_unpack_red function"
    );
}

// ============================================================================
// C1216-C1220: SIMD-Style Bit Parallel Operations
// ============================================================================

#[test]
fn c1216_parallel_prefix_sum() {
    let c_code = r#"
typedef unsigned int uint32_t;

uint32_t bit_prefix_or(uint32_t x) {
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    return x;
}

uint32_t bit_prefix_popcount(uint32_t x) {
    x = x - ((x >> 1) & 0x55555555);
    x = (x & 0x33333333) + ((x >> 2) & 0x33333333);
    x = (x + (x >> 4)) & 0x0F0F0F0F;
    x = x + (x >> 8);
    x = x + (x >> 16);
    return x & 0x3F;
}

uint32_t bit_parallel_suffix_or(uint32_t x) {
    x |= x << 1;
    x |= x << 2;
    x |= x << 4;
    x |= x << 8;
    x |= x << 16;
    return x;
}

uint32_t bit_highest_power_of_2_leq(uint32_t x) {
    x = bit_prefix_or(x);
    return x - (x >> 1);
}

int bit_prefix_selftest(void) {
    if (bit_prefix_or(0x100) != 0x1FF) return -1;
    if (bit_prefix_popcount(0xFF) != 8) return -2;
    if (bit_prefix_popcount(0) != 0) return -3;
    if (bit_highest_power_of_2_leq(100) != 64) return -4;
    if (bit_parallel_suffix_or(0x00800000) != 0xFF800000) return -5;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1216: Parallel prefix sum should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1216: Output should not be empty");
    assert!(
        code.contains("fn bit_prefix_or"),
        "C1216: Should contain bit_prefix_or function"
    );
    assert!(
        code.contains("fn bit_prefix_popcount"),
        "C1216: Should contain bit_prefix_popcount function"
    );
}

#[test]
fn c1217_horizontal_byte_add() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

uint32_t bit_horizontal_byte_add(uint32_t x) {
    uint32_t sum = 0;
    sum += (x & 0xFF);
    sum += ((x >> 8) & 0xFF);
    sum += ((x >> 16) & 0xFF);
    sum += ((x >> 24) & 0xFF);
    return sum;
}

uint32_t bit_horizontal_byte_max(uint32_t x) {
    uint8_t b0 = (uint8_t)(x & 0xFF);
    uint8_t b1 = (uint8_t)((x >> 8) & 0xFF);
    uint8_t b2 = (uint8_t)((x >> 16) & 0xFF);
    uint8_t b3 = (uint8_t)((x >> 24) & 0xFF);
    uint8_t m = b0;
    if (b1 > m) m = b1;
    if (b2 > m) m = b2;
    if (b3 > m) m = b3;
    return (uint32_t)m;
}

uint32_t bit_byte_contains_zero(uint32_t x) {
    return ((x - 0x01010101) & ~x & 0x80808080) != 0;
}

uint32_t bit_byte_has_value(uint32_t x, uint8_t val) {
    uint32_t spread = 0x01010101 * (uint32_t)val;
    return bit_byte_contains_zero(x ^ spread);
}

int bit_horiz_selftest(void) {
    if (bit_horizontal_byte_add(0x01020304) != 10) return -1;
    if (bit_horizontal_byte_max(0x01FF0203) != 255) return -2;
    if (!bit_byte_contains_zero(0x00FF0102)) return -3;
    if (bit_byte_contains_zero(0x01020304)) return -4;
    if (!bit_byte_has_value(0x01020304, 3)) return -5;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1217: Horizontal byte add should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1217: Output should not be empty");
    assert!(
        code.contains("fn bit_horizontal_byte_add"),
        "C1217: Should contain bit_horizontal_byte_add function"
    );
    assert!(
        code.contains("fn bit_byte_contains_zero"),
        "C1217: Should contain bit_byte_contains_zero function"
    );
}

#[test]
fn c1218_butterfly_network_shuffle() {
    let c_code = r#"
typedef unsigned int uint32_t;

uint32_t bit_butterfly_step(uint32_t x, uint32_t mask, int shift) {
    uint32_t t = ((x >> shift) ^ x) & mask;
    return x ^ t ^ (t << shift);
}

uint32_t bit_outer_perfect_shuffle(uint32_t x) {
    x = bit_butterfly_step(x, 0x00FF00FF, 8);
    x = bit_butterfly_step(x, 0x0F0F0F0F, 4);
    x = bit_butterfly_step(x, 0x33333333, 2);
    x = bit_butterfly_step(x, 0x55555555, 1);
    return x;
}

uint32_t bit_outer_perfect_unshuffle(uint32_t x) {
    x = bit_butterfly_step(x, 0x55555555, 1);
    x = bit_butterfly_step(x, 0x33333333, 2);
    x = bit_butterfly_step(x, 0x0F0F0F0F, 4);
    x = bit_butterfly_step(x, 0x00FF00FF, 8);
    return x;
}

uint32_t bit_swap_even_odd_bits(uint32_t x) {
    return ((x & 0x55555555) << 1) | ((x >> 1) & 0x55555555);
}

int bit_butterfly_selftest(void) {
    uint32_t x = 0xAABBCCDD;
    uint32_t shuffled = bit_outer_perfect_shuffle(x);
    uint32_t unshuffled = bit_outer_perfect_unshuffle(shuffled);
    if (unshuffled != x) return -1;
    if (bit_swap_even_odd_bits(0xAAAAAAAA) != 0x55555555) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1218: Butterfly network shuffle should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1218: Output should not be empty");
    assert!(
        code.contains("fn bit_butterfly_step"),
        "C1218: Should contain bit_butterfly_step function"
    );
    assert!(
        code.contains("fn bit_outer_perfect_shuffle"),
        "C1218: Should contain bit_outer_perfect_shuffle function"
    );
}

#[test]
fn c1219_bit_matrix_transpose_8x8() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;
typedef unsigned char uint8_t;

void bit_transpose_8x8(uint8_t *matrix) {
    uint32_t x, y, t;
    int i;

    x = ((uint32_t)matrix[0] << 24) | ((uint32_t)matrix[1] << 16) |
        ((uint32_t)matrix[2] << 8)  | (uint32_t)matrix[3];
    y = ((uint32_t)matrix[4] << 24) | ((uint32_t)matrix[5] << 16) |
        ((uint32_t)matrix[6] << 8)  | (uint32_t)matrix[7];

    t = (x ^ (x >> 7)) & 0x00AA00AA; x = x ^ t ^ (t << 7);
    t = (y ^ (y >> 7)) & 0x00AA00AA; y = y ^ t ^ (t << 7);

    t = (x ^ (x >> 14)) & 0x0000CCCC; x = x ^ t ^ (t << 14);
    t = (y ^ (y >> 14)) & 0x0000CCCC; y = y ^ t ^ (t << 14);

    t = (x & 0xF0F0F0F0) | ((y >> 4) & 0x0F0F0F0F);
    y = ((x << 4) & 0xF0F0F0F0) | (y & 0x0F0F0F0F);
    x = t;

    matrix[0] = (uint8_t)(x >> 24); matrix[1] = (uint8_t)(x >> 16);
    matrix[2] = (uint8_t)(x >> 8);  matrix[3] = (uint8_t)x;
    matrix[4] = (uint8_t)(y >> 24); matrix[5] = (uint8_t)(y >> 16);
    matrix[6] = (uint8_t)(y >> 8);  matrix[7] = (uint8_t)y;
}

int bit_transpose_selftest(void) {
    uint8_t m[8];
    uint8_t backup[8];
    int i;
    m[0] = 0xFF; m[1] = 0x00; m[2] = 0xFF; m[3] = 0x00;
    m[4] = 0xFF; m[5] = 0x00; m[6] = 0xFF; m[7] = 0x00;
    for (i = 0; i < 8; i++) backup[i] = m[i];
    bit_transpose_8x8(m);
    bit_transpose_8x8(m);
    for (i = 0; i < 8; i++) {
        if (m[i] != backup[i]) return -1;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1219: Bit matrix transpose 8x8 should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1219: Output should not be empty");
    assert!(
        code.contains("fn bit_transpose_8x8"),
        "C1219: Should contain bit_transpose_8x8 function"
    );
}

#[test]
fn c1220_saturating_arithmetic() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

uint8_t bit_sat_add_u8(uint8_t a, uint8_t b) {
    uint8_t result = a + b;
    if (result < a) result = 255;
    return result;
}

uint8_t bit_sat_sub_u8(uint8_t a, uint8_t b) {
    uint8_t result = a - b;
    if (result > a) result = 0;
    return result;
}

uint32_t bit_sat_add_u32(uint32_t a, uint32_t b) {
    uint32_t result = a + b;
    if (result < a) return 0xFFFFFFFF;
    return result;
}

uint32_t bit_sat_sub_u32(uint32_t a, uint32_t b) {
    uint32_t result = a - b;
    if (result > a) return 0;
    return result;
}

uint32_t bit_average_no_overflow(uint32_t a, uint32_t b) {
    return (a & b) + ((a ^ b) >> 1);
}

int bit_sat_selftest(void) {
    if (bit_sat_add_u8(200, 100) != 255) return -1;
    if (bit_sat_add_u8(100, 50) != 150) return -2;
    if (bit_sat_sub_u8(10, 20) != 0) return -3;
    if (bit_sat_sub_u8(20, 10) != 10) return -4;
    if (bit_sat_add_u32(0xFFFFFFFF, 1) != 0xFFFFFFFF) return -5;
    if (bit_average_no_overflow(6, 10) != 8) return -6;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1220: Saturating arithmetic should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1220: Output should not be empty");
    assert!(
        code.contains("fn bit_sat_add_u8"),
        "C1220: Should contain bit_sat_add_u8 function"
    );
    assert!(
        code.contains("fn bit_sat_sub_u8"),
        "C1220: Should contain bit_sat_sub_u8 function"
    );
}

// ============================================================================
// C1221-C1225: Applied Bit Manipulation
// ============================================================================

#[test]
fn c1221_crc32_bitwise_computation() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;
typedef unsigned long size_t;

uint32_t bit_crc32_update_byte(uint32_t crc, uint8_t byte) {
    int i;
    crc ^= (uint32_t)byte;
    for (i = 0; i < 8; i++) {
        if (crc & 1) {
            crc = (crc >> 1) ^ 0xEDB88320;
        } else {
            crc >>= 1;
        }
    }
    return crc;
}

uint32_t bit_crc32_compute(const uint8_t *data, size_t len) {
    uint32_t crc = 0xFFFFFFFF;
    size_t i;
    for (i = 0; i < len; i++) {
        crc = bit_crc32_update_byte(crc, data[i]);
    }
    return crc ^ 0xFFFFFFFF;
}

void bit_crc32_build_table(uint32_t table[256]) {
    uint32_t i;
    for (i = 0; i < 256; i++) {
        uint32_t crc = i;
        int j;
        for (j = 0; j < 8; j++) {
            if (crc & 1) {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
        table[i] = crc;
    }
}

int bit_crc32_selftest(void) {
    uint8_t data[4];
    uint32_t table[256];
    data[0] = 'T'; data[1] = 'e'; data[2] = 's'; data[3] = 't';
    uint32_t crc = bit_crc32_compute(data, 4);
    if (crc == 0) return -1;
    bit_crc32_build_table(table);
    if (table[0] != 0) return -2;
    if (table[1] == 0) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1221: CRC32 bitwise computation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1221: Output should not be empty");
    assert!(
        code.contains("fn bit_crc32_update_byte"),
        "C1221: Should contain bit_crc32_update_byte function"
    );
    assert!(
        code.contains("fn bit_crc32_compute"),
        "C1221: Should contain bit_crc32_compute function"
    );
}

#[test]
fn c1222_hamming_code_encode_decode() {
    let c_code = r#"
typedef unsigned int uint32_t;

int bit_hamming_parity_bit(uint32_t data, int pbit) {
    int parity = 0;
    int i;
    for (i = pbit; i <= 15; i++) {
        if ((i + 1) & pbit) {
            parity ^= (data >> i) & 1;
        }
    }
    return parity;
}

uint32_t bit_hamming_encode_11_to_15(uint32_t data) {
    uint32_t encoded = 0;
    int di = 0;
    int i;

    for (i = 0; i < 15; i++) {
        int pos = i + 1;
        if ((pos & (pos - 1)) == 0) {
            continue;
        }
        if (data & (1u << di)) {
            encoded |= (1u << i);
        }
        di++;
    }

    for (i = 0; i < 4; i++) {
        int pbit = 1 << i;
        int parity = bit_hamming_parity_bit(encoded, pbit);
        if (parity) {
            encoded |= (1u << (pbit - 1));
        }
    }
    return encoded;
}

int bit_hamming_syndrome(uint32_t encoded) {
    int syndrome = 0;
    int i;
    for (i = 0; i < 4; i++) {
        int pbit = 1 << i;
        int parity = bit_hamming_parity_bit(encoded, pbit);
        if (parity) {
            syndrome |= pbit;
        }
    }
    return syndrome;
}

uint32_t bit_hamming_correct(uint32_t encoded) {
    int syndrome = bit_hamming_syndrome(encoded);
    if (syndrome != 0 && syndrome <= 15) {
        encoded ^= (1u << (syndrome - 1));
    }
    return encoded;
}

int bit_hamming_selftest(void) {
    uint32_t data = 0x5A3;
    uint32_t encoded = bit_hamming_encode_11_to_15(data);
    if (bit_hamming_syndrome(encoded) != 0) return -1;
    uint32_t corrupted = encoded ^ (1u << 5);
    if (bit_hamming_syndrome(corrupted) == 0) return -2;
    uint32_t corrected = bit_hamming_correct(corrupted);
    if (corrected != encoded) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1222: Hamming code encode/decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1222: Output should not be empty");
    assert!(
        code.contains("fn bit_hamming_encode_11_to_15"),
        "C1222: Should contain bit_hamming_encode_11_to_15 function"
    );
    assert!(
        code.contains("fn bit_hamming_correct"),
        "C1222: Should contain bit_hamming_correct function"
    );
}

#[test]
fn c1223_bit_parallel_string_match() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long size_t;

int bit_bitap_search(const char *text, size_t text_len,
                     const char *pattern, size_t pat_len) {
    uint32_t pattern_mask[128];
    uint32_t state;
    uint32_t match_bit;
    size_t i;
    int c;

    if (pat_len == 0) return 0;
    if (pat_len > 31) return -1;

    match_bit = 1u << (pat_len - 1);

    for (c = 0; c < 128; c++) {
        pattern_mask[c] = 0xFFFFFFFF;
    }
    for (i = 0; i < pat_len; i++) {
        c = (int)pattern[i];
        if (c >= 0 && c < 128) {
            pattern_mask[c] &= ~(1u << i);
        }
    }

    state = 0xFFFFFFFF;
    for (i = 0; i < text_len; i++) {
        c = (int)text[i];
        if (c >= 0 && c < 128) {
            state = (state << 1) | pattern_mask[c];
        }
        if ((state & match_bit) == 0) {
            return (int)(i - pat_len + 1);
        }
    }
    return -1;
}

int bit_has_char(const char *str, size_t len, char target) {
    uint32_t bloom = 0;
    size_t i;
    for (i = 0; i < len; i++) {
        bloom |= 1u << ((unsigned char)str[i] & 31);
    }
    return (bloom >> ((unsigned char)target & 31)) & 1;
}

int bit_bitap_selftest(void) {
    const char *text = "hello world bitap test";
    const char *pat = "bitap";
    int pos = bit_bitap_search(text, 22, pat, 5);
    if (pos < 0) return -1;
    if (pos != 12) return -2;
    pos = bit_bitap_search(text, 22, "xyz", 3);
    if (pos != -1) return -3;
    if (!bit_has_char("abcdef", 6, 'c')) return -4;
    if (bit_has_char("abcdef", 6, 'z')) return -5;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1223: Bit-parallel string match should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1223: Output should not be empty");
    assert!(
        code.contains("fn bit_bitap_search"),
        "C1223: Should contain bit_bitap_search function"
    );
    assert!(
        code.contains("fn bit_has_char"),
        "C1223: Should contain bit_has_char function"
    );
}

#[test]
fn c1224_compact_bitboard_operations() {
    let c_code = r#"
typedef unsigned long long uint64_t;
typedef unsigned int uint32_t;

uint64_t bit_board_set(uint64_t board, int row, int col) {
    return board | (1ULL << (row * 8 + col));
}

uint64_t bit_board_clear(uint64_t board, int row, int col) {
    return board & ~(1ULL << (row * 8 + col));
}

int bit_board_test(uint64_t board, int row, int col) {
    return (int)((board >> (row * 8 + col)) & 1);
}

int bit_board_popcount64(uint64_t x) {
    x = x - ((x >> 1) & 0x5555555555555555ULL);
    x = (x & 0x3333333333333333ULL) + ((x >> 2) & 0x3333333333333333ULL);
    x = (x + (x >> 4)) & 0x0F0F0F0F0F0F0F0FULL;
    return (int)((x * 0x0101010101010101ULL) >> 56);
}

uint64_t bit_board_shift_north(uint64_t board) {
    return board << 8;
}

uint64_t bit_board_shift_south(uint64_t board) {
    return board >> 8;
}

uint64_t bit_board_shift_east(uint64_t board) {
    return (board << 1) & 0xFEFEFEFEFEFEFEFEULL;
}

uint64_t bit_board_shift_west(uint64_t board) {
    return (board >> 1) & 0x7F7F7F7F7F7F7F7FULL;
}

uint64_t bit_board_king_attacks(uint64_t king) {
    uint64_t attacks = bit_board_shift_east(king) | bit_board_shift_west(king);
    uint64_t king_set = king | attacks;
    attacks |= bit_board_shift_north(king_set) | bit_board_shift_south(king_set);
    return attacks;
}

int bit_board_selftest(void) {
    uint64_t board = 0;
    board = bit_board_set(board, 3, 4);
    if (!bit_board_test(board, 3, 4)) return -1;
    if (bit_board_test(board, 3, 5)) return -2;
    if (bit_board_popcount64(board) != 1) return -3;
    uint64_t king = bit_board_set(0, 4, 4);
    uint64_t attacks = bit_board_king_attacks(king);
    if (bit_board_popcount64(attacks) != 8) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1224: Compact bitboard operations should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1224: Output should not be empty");
    assert!(
        code.contains("fn bit_board_set"),
        "C1224: Should contain bit_board_set function"
    );
    assert!(
        code.contains("fn bit_board_king_attacks"),
        "C1224: Should contain bit_board_king_attacks function"
    );
}

#[test]
fn c1225_fenwick_bit_tree() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    int tree[1024];
    int n;
} bit_fenwick_t;

void bit_fenwick_init(bit_fenwick_t *f, int n) {
    int i;
    f->n = n;
    for (i = 0; i <= n; i++) {
        f->tree[i] = 0;
    }
}

void bit_fenwick_update(bit_fenwick_t *f, int i, int delta) {
    for (i++; i <= f->n; i += i & (-i)) {
        f->tree[i] += delta;
    }
}

int bit_fenwick_prefix_sum(bit_fenwick_t *f, int i) {
    int sum = 0;
    for (i++; i > 0; i -= i & (-i)) {
        sum += f->tree[i];
    }
    return sum;
}

int bit_fenwick_range_sum(bit_fenwick_t *f, int l, int r) {
    int right_sum = bit_fenwick_prefix_sum(f, r);
    if (l == 0) return right_sum;
    return right_sum - bit_fenwick_prefix_sum(f, l - 1);
}

int bit_fenwick_find_kth(bit_fenwick_t *f, int k) {
    int pos = 0;
    int log2n;
    int bit_mask;
    for (log2n = 0; (1 << log2n) <= f->n; log2n++) {}
    for (bit_mask = 1 << (log2n - 1); bit_mask > 0; bit_mask >>= 1) {
        int next = pos + bit_mask;
        if (next <= f->n && f->tree[next] < k) {
            k -= f->tree[next];
            pos = next;
        }
    }
    return pos;
}

int bit_fenwick_selftest(void) {
    bit_fenwick_t f;
    bit_fenwick_init(&f, 10);
    bit_fenwick_update(&f, 0, 3);
    bit_fenwick_update(&f, 1, 5);
    bit_fenwick_update(&f, 2, 7);
    bit_fenwick_update(&f, 3, 2);
    if (bit_fenwick_prefix_sum(&f, 3) != 17) return -1;
    if (bit_fenwick_range_sum(&f, 1, 2) != 12) return -2;
    bit_fenwick_update(&f, 1, -2);
    if (bit_fenwick_prefix_sum(&f, 3) != 15) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1225: Fenwick bit tree should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1225: Output should not be empty");
    assert!(
        code.contains("fn bit_fenwick_init"),
        "C1225: Should contain bit_fenwick_init function"
    );
    assert!(
        code.contains("fn bit_fenwick_update"),
        "C1225: Should contain bit_fenwick_update function"
    );
    assert!(
        code.contains("fn bit_fenwick_prefix_sum"),
        "C1225: Should contain bit_fenwick_prefix_sum function"
    );
}
