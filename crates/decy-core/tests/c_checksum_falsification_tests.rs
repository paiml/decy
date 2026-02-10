//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1751-C1775: Checksum and Hash Function implementations -- the kind of C code
//! found in data integrity libraries, hash table implementations, network protocols,
//! and probabilistic data structures.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world checksum and hashing patterns commonly
//! found in zlib, xxHash, MurmurHash, SipHash, and similar libraries --
//! all expressed as valid C99 without #include directives.
//!
//! Organization:
//! - C1751-C1755: Simple checksums (XOR, additive, Fletcher-16, Adler-32, Luhn)
//! - C1756-C1760: CRC variants (CRC-8, CRC-16/CCITT, CRC-32 table, CRC-32C, custom poly)
//! - C1761-C1765: Hash functions (DJB2, FNV-1a, Jenkins, MurmurHash3, SipHash)
//! - C1766-C1770: Rolling hashes (Rabin, polynomial, cyclic, content-defined, Buzhash)
//! - C1771-C1775: Hash applications (Bloom filter, count-min sketch, HyperLogLog, consistent hash, hash table)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C1751-C1755: Simple Checksums
// ============================================================================

#[test]
fn c1751_xor_checksum() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

uint8_t chk_xor_checksum(const uint8_t *data, size_t len) {
    uint8_t checksum = 0;
    size_t i;
    for (i = 0; i < len; i++) {
        checksum ^= data[i];
    }
    return checksum;
}

uint32_t chk_xor_checksum32(const uint8_t *data, size_t len) {
    uint32_t checksum = 0;
    size_t i;
    for (i = 0; i < len; i++) {
        checksum ^= ((uint32_t)data[i]) << ((i % 4) * 8);
    }
    return checksum;
}

int chk_xor_verify(const uint8_t *data, size_t len, uint8_t expected) {
    return chk_xor_checksum(data, len) == expected ? 1 : 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1751: XOR checksum should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1751: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1751: Should contain chk_ functions");
}

#[test]
fn c1752_additive_checksum() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

uint16_t chk_additive_checksum(const uint8_t *data, size_t len) {
    uint32_t sum = 0;
    size_t i;
    for (i = 0; i < len; i++) {
        sum += data[i];
    }
    return (uint16_t)(sum & 0xFFFF);
}

uint16_t chk_ones_complement_checksum(const uint8_t *data, size_t len) {
    uint32_t sum = 0;
    size_t i;
    for (i = 0; i + 1 < len; i += 2) {
        sum += ((uint32_t)data[i] << 8) | data[i + 1];
    }
    if (len % 2 != 0) {
        sum += (uint32_t)data[len - 1] << 8;
    }
    while (sum >> 16) {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    return (uint16_t)(~sum & 0xFFFF);
}

int chk_additive_verify(const uint8_t *data, size_t len, uint16_t expected) {
    return chk_additive_checksum(data, len) == expected ? 1 : 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1752: Additive checksum should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1752: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1752: Should contain chk_ functions");
}

#[test]
fn c1753_fletcher16_checksum() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

uint16_t chk_fletcher16(const uint8_t *data, size_t len) {
    uint16_t sum1 = 0;
    uint16_t sum2 = 0;
    size_t i;
    for (i = 0; i < len; i++) {
        sum1 = (sum1 + data[i]) % 255;
        sum2 = (sum2 + sum1) % 255;
    }
    return (sum2 << 8) | sum1;
}

int chk_fletcher16_verify(const uint8_t *data, size_t len, uint16_t expected) {
    return chk_fletcher16(data, len) == expected ? 1 : 0;
}

uint16_t chk_fletcher16_update(uint16_t prev, uint8_t byte) {
    uint16_t sum1 = prev & 0xFF;
    uint16_t sum2 = (prev >> 8) & 0xFF;
    sum1 = (sum1 + byte) % 255;
    sum2 = (sum2 + sum1) % 255;
    return (sum2 << 8) | sum1;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1753: Fletcher-16 checksum should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1753: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1753: Should contain chk_ functions");
}

#[test]
fn c1754_adler32_checksum() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

uint32_t chk_adler32(const uint8_t *data, size_t len) {
    uint32_t a = 1;
    uint32_t b = 0;
    size_t i;
    for (i = 0; i < len; i++) {
        a = (a + data[i]) % 65521;
        b = (b + a) % 65521;
    }
    return (b << 16) | a;
}

uint32_t chk_adler32_combine(uint32_t adler1, uint32_t adler2, size_t len2) {
    uint32_t a1 = adler1 & 0xFFFF;
    uint32_t b1 = (adler1 >> 16) & 0xFFFF;
    uint32_t a2 = adler2 & 0xFFFF;
    uint32_t b2 = (adler2 >> 16) & 0xFFFF;
    uint32_t a = (a1 + a2 - 1) % 65521;
    uint32_t b = (b1 + b2 + a1 * len2 - len2) % 65521;
    return (b << 16) | a;
}

int chk_adler32_verify(const uint8_t *data, size_t len, uint32_t expected) {
    return chk_adler32(data, len) == expected ? 1 : 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1754: Adler-32 checksum should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1754: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1754: Should contain chk_ functions");
}

#[test]
fn c1755_luhn_algorithm() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

int chk_luhn_digit_value(uint8_t ch) {
    if (ch >= 48 && ch <= 57) {
        return ch - 48;
    }
    return -1;
}

int chk_luhn_validate(const uint8_t *digits, size_t len) {
    int sum = 0;
    int alt = 0;
    int i;
    int n;
    if (len == 0) return 0;
    for (i = (int)len - 1; i >= 0; i--) {
        n = chk_luhn_digit_value(digits[i]);
        if (n < 0) return 0;
        if (alt) {
            n *= 2;
            if (n > 9) n -= 9;
        }
        sum += n;
        alt = !alt;
    }
    return (sum % 10) == 0 ? 1 : 0;
}

uint8_t chk_luhn_compute_check(const uint8_t *digits, size_t len) {
    int sum = 0;
    int alt = 1;
    int i;
    int n;
    for (i = (int)len - 1; i >= 0; i--) {
        n = chk_luhn_digit_value(digits[i]);
        if (n < 0) return 0;
        if (alt) {
            n *= 2;
            if (n > 9) n -= 9;
        }
        sum += n;
        alt = !alt;
    }
    return (uint8_t)((10 - (sum % 10)) % 10 + 48);
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1755: Luhn algorithm should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1755: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1755: Should contain chk_ functions");
}

// ============================================================================
// C1756-C1760: CRC Variants
// ============================================================================

#[test]
fn c1756_crc8() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

uint8_t chk_crc8_byte(uint8_t crc, uint8_t data) {
    int i;
    crc ^= data;
    for (i = 0; i < 8; i++) {
        if (crc & 0x80) {
            crc = (crc << 1) ^ 0x07;
        } else {
            crc = crc << 1;
        }
    }
    return crc;
}

uint8_t chk_crc8(const uint8_t *data, size_t len) {
    uint8_t crc = 0x00;
    size_t i;
    for (i = 0; i < len; i++) {
        crc = chk_crc8_byte(crc, data[i]);
    }
    return crc;
}

int chk_crc8_verify(const uint8_t *data, size_t len, uint8_t expected) {
    return chk_crc8(data, len) == expected ? 1 : 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1756: CRC-8 should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1756: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1756: Should contain chk_ functions");
}

#[test]
fn c1757_crc16_ccitt() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

uint16_t chk_crc16_ccitt_byte(uint16_t crc, uint8_t data) {
    int i;
    crc ^= (uint16_t)data << 8;
    for (i = 0; i < 8; i++) {
        if (crc & 0x8000) {
            crc = (crc << 1) ^ 0x1021;
        } else {
            crc = crc << 1;
        }
    }
    return crc;
}

uint16_t chk_crc16_ccitt(const uint8_t *data, size_t len) {
    uint16_t crc = 0xFFFF;
    size_t i;
    for (i = 0; i < len; i++) {
        crc = chk_crc16_ccitt_byte(crc, data[i]);
    }
    return crc;
}

int chk_crc16_ccitt_verify(const uint8_t *data, size_t len, uint16_t expected) {
    return chk_crc16_ccitt(data, len) == expected ? 1 : 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1757: CRC-16/CCITT should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1757: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1757: Should contain chk_ functions");
}

#[test]
fn c1758_crc32_table_driven() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

void chk_crc32_build_table(uint32_t table[256]) {
    uint32_t crc;
    int i;
    int j;
    for (i = 0; i < 256; i++) {
        crc = (uint32_t)i;
        for (j = 0; j < 8; j++) {
            if (crc & 1) {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc = crc >> 1;
            }
        }
        table[i] = crc;
    }
}

uint32_t chk_crc32_compute(const uint8_t *data, size_t len, const uint32_t table[256]) {
    uint32_t crc = 0xFFFFFFFF;
    size_t i;
    for (i = 0; i < len; i++) {
        crc = table[(crc ^ data[i]) & 0xFF] ^ (crc >> 8);
    }
    return crc ^ 0xFFFFFFFF;
}

int chk_crc32_selftest(void) {
    uint32_t table[256];
    uint8_t test_data[9];
    uint32_t result;
    test_data[0] = 49; test_data[1] = 50; test_data[2] = 51;
    test_data[3] = 52; test_data[4] = 53; test_data[5] = 54;
    test_data[6] = 55; test_data[7] = 56; test_data[8] = 57;
    chk_crc32_build_table(table);
    result = chk_crc32_compute(test_data, 9, table);
    return result != 0 ? 1 : 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1758: CRC-32 table-driven should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1758: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1758: Should contain chk_ functions");
}

#[test]
fn c1759_crc32c_castagnoli() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

uint32_t chk_crc32c_byte(uint32_t crc, uint8_t data) {
    int i;
    crc ^= data;
    for (i = 0; i < 8; i++) {
        if (crc & 1) {
            crc = (crc >> 1) ^ 0x82F63B78;
        } else {
            crc = crc >> 1;
        }
    }
    return crc;
}

uint32_t chk_crc32c(const uint8_t *data, size_t len) {
    uint32_t crc = 0xFFFFFFFF;
    size_t i;
    for (i = 0; i < len; i++) {
        crc = chk_crc32c_byte(crc, data[i]);
    }
    return crc ^ 0xFFFFFFFF;
}

int chk_crc32c_verify(const uint8_t *data, size_t len, uint32_t expected) {
    return chk_crc32c(data, len) == expected ? 1 : 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1759: CRC-32C Castagnoli should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1759: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1759: Should contain chk_ functions");
}

#[test]
fn c1760_custom_polynomial_crc() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint32_t polynomial;
    uint32_t init_value;
    uint32_t final_xor;
    int reflect_input;
    int reflect_output;
} chk_crc_config_t;

uint32_t chk_reflect32(uint32_t value, int bits) {
    uint32_t result = 0;
    int i;
    for (i = 0; i < bits; i++) {
        if (value & (1u << i)) {
            result |= 1u << (bits - 1 - i);
        }
    }
    return result;
}

uint32_t chk_custom_crc(const uint8_t *data, size_t len, const chk_crc_config_t *cfg) {
    uint32_t crc = cfg->init_value;
    size_t i;
    int j;
    uint8_t byte;
    for (i = 0; i < len; i++) {
        byte = data[i];
        if (cfg->reflect_input) {
            byte = (uint8_t)chk_reflect32(byte, 8);
        }
        crc ^= (uint32_t)byte << 24;
        for (j = 0; j < 8; j++) {
            if (crc & 0x80000000) {
                crc = (crc << 1) ^ cfg->polynomial;
            } else {
                crc = crc << 1;
            }
        }
    }
    if (cfg->reflect_output) {
        crc = chk_reflect32(crc, 32);
    }
    return crc ^ cfg->final_xor;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1760: Custom polynomial CRC should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1760: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1760: Should contain chk_ functions");
}

// ============================================================================
// C1761-C1765: Hash Functions
// ============================================================================

#[test]
fn c1761_djb2_hash() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

uint32_t chk_djb2_hash(const uint8_t *data, size_t len) {
    uint32_t hash = 5381;
    size_t i;
    for (i = 0; i < len; i++) {
        hash = ((hash << 5) + hash) + data[i];
    }
    return hash;
}

uint32_t chk_djb2a_hash(const uint8_t *data, size_t len) {
    uint32_t hash = 5381;
    size_t i;
    for (i = 0; i < len; i++) {
        hash = ((hash << 5) + hash) ^ data[i];
    }
    return hash;
}

uint32_t chk_sdbm_hash(const uint8_t *data, size_t len) {
    uint32_t hash = 0;
    size_t i;
    for (i = 0; i < len; i++) {
        hash = data[i] + (hash << 6) + (hash << 16) - hash;
    }
    return hash;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1761: DJB2 hash should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1761: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1761: Should contain chk_ functions");
}

#[test]
fn c1762_fnv1a_hash() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

uint32_t chk_fnv1a_32(const uint8_t *data, size_t len) {
    uint32_t hash = 0x811C9DC5;
    size_t i;
    for (i = 0; i < len; i++) {
        hash ^= data[i];
        hash *= 0x01000193;
    }
    return hash;
}

uint32_t chk_fnv1_32(const uint8_t *data, size_t len) {
    uint32_t hash = 0x811C9DC5;
    size_t i;
    for (i = 0; i < len; i++) {
        hash *= 0x01000193;
        hash ^= data[i];
    }
    return hash;
}

uint32_t chk_fnv1a_fold16(const uint8_t *data, size_t len) {
    uint32_t hash = chk_fnv1a_32(data, len);
    return ((hash >> 16) ^ hash) & 0xFFFF;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1762: FNV-1a hash should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1762: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1762: Should contain chk_ functions");
}

#[test]
fn c1763_jenkins_one_at_a_time() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

uint32_t chk_jenkins_oaat(const uint8_t *data, size_t len) {
    uint32_t hash = 0;
    size_t i;
    for (i = 0; i < len; i++) {
        hash += data[i];
        hash += hash << 10;
        hash ^= hash >> 6;
    }
    hash += hash << 3;
    hash ^= hash >> 11;
    hash += hash << 15;
    return hash;
}

uint32_t chk_jenkins_lookup3_mix(uint32_t a, uint32_t b, uint32_t c) {
    a -= c; a ^= (c << 4) | (c >> 28); c += b;
    b -= a; b ^= (a << 6) | (a >> 26); a += c;
    c -= b; c ^= (b << 8) | (b >> 24); b += a;
    a -= c; a ^= (c << 16) | (c >> 16); c += b;
    b -= a; b ^= (a << 19) | (a >> 13); a += c;
    c -= b; c ^= (b << 4) | (b >> 28); b += a;
    return c;
}

int chk_jenkins_selftest(void) {
    uint8_t test[3];
    test[0] = 65; test[1] = 66; test[2] = 67;
    return chk_jenkins_oaat(test, 3) != 0 ? 1 : 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1763: Jenkins one-at-a-time should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1763: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1763: Should contain chk_ functions");
}

#[test]
fn c1764_murmurhash3_32() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

uint32_t chk_murmur3_rotl32(uint32_t x, int r) {
    return (x << r) | (x >> (32 - r));
}

uint32_t chk_murmur3_fmix32(uint32_t h) {
    h ^= h >> 16;
    h *= 0x85EBCA6B;
    h ^= h >> 13;
    h *= 0xC2B2AE35;
    h ^= h >> 16;
    return h;
}

uint32_t chk_murmur3_32(const uint8_t *data, size_t len, uint32_t seed) {
    uint32_t h1 = seed;
    uint32_t c1 = 0xCC9E2D51;
    uint32_t c2 = 0x1B873593;
    size_t nblocks = len / 4;
    size_t i;
    uint32_t k1;
    uint32_t tail_val;
    const uint8_t *tail;

    for (i = 0; i < nblocks; i++) {
        k1 = ((uint32_t)data[i * 4 + 0])
           | ((uint32_t)data[i * 4 + 1] << 8)
           | ((uint32_t)data[i * 4 + 2] << 16)
           | ((uint32_t)data[i * 4 + 3] << 24);
        k1 *= c1;
        k1 = chk_murmur3_rotl32(k1, 15);
        k1 *= c2;
        h1 ^= k1;
        h1 = chk_murmur3_rotl32(h1, 13);
        h1 = h1 * 5 + 0xE6546B64;
    }

    tail = data + nblocks * 4;
    tail_val = 0;
    switch (len & 3) {
        case 3: tail_val ^= (uint32_t)tail[2] << 16;
        case 2: tail_val ^= (uint32_t)tail[1] << 8;
        case 1: tail_val ^= (uint32_t)tail[0];
                tail_val *= c1;
                tail_val = chk_murmur3_rotl32(tail_val, 15);
                tail_val *= c2;
                h1 ^= tail_val;
    }

    h1 ^= (uint32_t)len;
    h1 = chk_murmur3_fmix32(h1);
    return h1;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1764: MurmurHash3 32-bit should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1764: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1764: Should contain chk_ functions");
}

#[test]
fn c1765_siphash_simplified() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint32_t v0;
    uint32_t v1;
    uint32_t v2;
    uint32_t v3;
} chk_siphash_state_t;

uint32_t chk_sip_rotl(uint32_t x, int n) {
    return (x << n) | (x >> (32 - n));
}

void chk_sip_round(chk_siphash_state_t *s) {
    s->v0 += s->v1;
    s->v1 = chk_sip_rotl(s->v1, 5) ^ s->v0;
    s->v0 = chk_sip_rotl(s->v0, 16);
    s->v2 += s->v3;
    s->v3 = chk_sip_rotl(s->v3, 8) ^ s->v2;
    s->v0 += s->v3;
    s->v3 = chk_sip_rotl(s->v3, 7) ^ s->v0;
    s->v2 += s->v1;
    s->v1 = chk_sip_rotl(s->v1, 13) ^ s->v2;
    s->v2 = chk_sip_rotl(s->v2, 16);
}

void chk_sip_init(chk_siphash_state_t *s, uint32_t k0, uint32_t k1) {
    s->v0 = k0 ^ 0x736F6D65;
    s->v1 = k1 ^ 0x646F7261;
    s->v2 = k0 ^ 0x6C796765;
    s->v3 = k1 ^ 0x74656462;
}

uint32_t chk_siphash_simple(const uint8_t *data, size_t len, uint32_t k0, uint32_t k1) {
    chk_siphash_state_t s;
    size_t i;
    uint32_t m;
    chk_sip_init(&s, k0, k1);
    for (i = 0; i + 3 < len; i += 4) {
        m = ((uint32_t)data[i])
          | ((uint32_t)data[i+1] << 8)
          | ((uint32_t)data[i+2] << 16)
          | ((uint32_t)data[i+3] << 24);
        s.v3 ^= m;
        chk_sip_round(&s);
        chk_sip_round(&s);
        s.v0 ^= m;
    }
    s.v2 ^= 0xFF;
    chk_sip_round(&s);
    chk_sip_round(&s);
    chk_sip_round(&s);
    chk_sip_round(&s);
    return s.v0 ^ s.v1 ^ s.v2 ^ s.v3;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1765: SipHash simplified should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1765: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1765: Should contain chk_ functions");
}

// ============================================================================
// C1766-C1770: Rolling Hashes
// ============================================================================

#[test]
fn c1766_rabin_fingerprint() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint32_t hash;
    uint32_t pow;
    uint32_t base;
    uint32_t modulus;
    int window_size;
} chk_rabin_state_t;

void chk_rabin_init(chk_rabin_state_t *r, int window_size) {
    int i;
    r->hash = 0;
    r->base = 256;
    r->modulus = 1000000007;
    r->window_size = window_size;
    r->pow = 1;
    for (i = 0; i < window_size - 1; i++) {
        r->pow = (r->pow * r->base) % r->modulus;
    }
}

void chk_rabin_slide(chk_rabin_state_t *r, uint8_t old_byte, uint8_t new_byte) {
    uint32_t old_contribution = ((uint32_t)old_byte * r->pow) % r->modulus;
    r->hash = (r->hash + r->modulus - old_contribution) % r->modulus;
    r->hash = (r->hash * r->base + new_byte) % r->modulus;
}

uint32_t chk_rabin_compute(const uint8_t *data, size_t len, uint32_t base, uint32_t modulus) {
    uint32_t hash = 0;
    size_t i;
    for (i = 0; i < len; i++) {
        hash = (hash * base + data[i]) % modulus;
    }
    return hash;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1766: Rabin fingerprint should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1766: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1766: Should contain chk_ functions");
}

#[test]
fn c1767_polynomial_rolling_hash() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint32_t hash;
    uint32_t base;
    uint32_t modulus;
    uint32_t base_pow;
    int count;
} chk_polyhash_t;

void chk_polyhash_init(chk_polyhash_t *ph, uint32_t base, uint32_t modulus) {
    ph->hash = 0;
    ph->base = base;
    ph->modulus = modulus;
    ph->base_pow = 1;
    ph->count = 0;
}

void chk_polyhash_append(chk_polyhash_t *ph, uint8_t byte) {
    ph->hash = (ph->hash * ph->base + byte) % ph->modulus;
    ph->base_pow = (ph->base_pow * ph->base) % ph->modulus;
    ph->count++;
}

uint32_t chk_polyhash_full(const uint8_t *data, size_t len) {
    chk_polyhash_t ph;
    size_t i;
    chk_polyhash_init(&ph, 31, 1000000007);
    for (i = 0; i < len; i++) {
        chk_polyhash_append(&ph, data[i]);
    }
    return ph.hash;
}

int chk_polyhash_compare(const uint8_t *a, size_t alen, const uint8_t *b, size_t blen) {
    uint32_t ha = chk_polyhash_full(a, alen);
    uint32_t hb = chk_polyhash_full(b, blen);
    return ha == hb ? 1 : 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1767: Polynomial rolling hash should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1767: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1767: Should contain chk_ functions");
}

#[test]
fn c1768_cyclic_polynomial_hash() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

uint32_t chk_cyclic_rotl(uint32_t x, int n) {
    return (x << n) | (x >> (32 - n));
}

typedef struct {
    uint32_t hash;
    int window_size;
    int pos;
} chk_cyclic_hash_t;

void chk_cyclic_init(chk_cyclic_hash_t *ch, int window_size) {
    ch->hash = 0;
    ch->window_size = window_size;
    ch->pos = 0;
}

void chk_cyclic_update(chk_cyclic_hash_t *ch, uint8_t new_byte) {
    ch->hash = chk_cyclic_rotl(ch->hash, 1) ^ (uint32_t)new_byte;
    ch->pos++;
}

void chk_cyclic_slide(chk_cyclic_hash_t *ch, uint8_t old_byte, uint8_t new_byte) {
    uint32_t old_contrib = chk_cyclic_rotl((uint32_t)old_byte, ch->window_size);
    ch->hash = chk_cyclic_rotl(ch->hash, 1) ^ old_contrib ^ (uint32_t)new_byte;
}

uint32_t chk_cyclic_compute(const uint8_t *data, size_t len) {
    chk_cyclic_hash_t ch;
    size_t i;
    chk_cyclic_init(&ch, (int)len);
    for (i = 0; i < len; i++) {
        chk_cyclic_update(&ch, data[i]);
    }
    return ch.hash;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1768: Cyclic polynomial hash should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1768: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1768: Should contain chk_ functions");
}

#[test]
fn c1769_content_defined_chunking() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint32_t hash;
    uint32_t mask;
    int min_chunk;
    int max_chunk;
} chk_cdc_state_t;

void chk_cdc_init(chk_cdc_state_t *cdc, int avg_chunk_bits, int min_chunk, int max_chunk) {
    cdc->hash = 0;
    cdc->mask = (1u << avg_chunk_bits) - 1;
    cdc->min_chunk = min_chunk;
    cdc->max_chunk = max_chunk;
}

int chk_cdc_is_boundary(chk_cdc_state_t *cdc) {
    return (cdc->hash & cdc->mask) == 0 ? 1 : 0;
}

int chk_cdc_find_boundary(chk_cdc_state_t *cdc, const uint8_t *data, size_t len) {
    size_t i;
    cdc->hash = 0;
    for (i = 0; i < len; i++) {
        cdc->hash = (cdc->hash << 1) ^ data[i];
        if ((int)i >= cdc->min_chunk && chk_cdc_is_boundary(cdc)) {
            return (int)i + 1;
        }
        if ((int)i >= cdc->max_chunk) {
            return cdc->max_chunk;
        }
    }
    return (int)len;
}

int chk_cdc_count_chunks(const uint8_t *data, size_t len, int avg_bits) {
    chk_cdc_state_t cdc;
    int chunks = 0;
    size_t offset = 0;
    int boundary;
    chk_cdc_init(&cdc, avg_bits, 64, 65536);
    while (offset < len) {
        boundary = chk_cdc_find_boundary(&cdc, data + offset, len - offset);
        chunks++;
        offset += boundary;
    }
    return chunks;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1769: Content-defined chunking should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1769: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1769: Should contain chk_ functions");
}

#[test]
fn c1770_buzhash() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

uint32_t chk_buz_table[256];

void chk_buz_init_table(uint32_t seed) {
    int i;
    uint32_t state = seed;
    for (i = 0; i < 256; i++) {
        state = state * 1664525 + 1013904223;
        chk_buz_table[i] = state;
    }
}

uint32_t chk_buz_rotl(uint32_t x, int n) {
    return (x << n) | (x >> (32 - n));
}

uint32_t chk_buz_hash(const uint8_t *data, size_t len) {
    uint32_t hash = 0;
    size_t i;
    for (i = 0; i < len; i++) {
        hash = chk_buz_rotl(hash, 1) ^ chk_buz_table[data[i]];
    }
    return hash;
}

uint32_t chk_buz_slide(uint32_t hash, uint8_t old_byte, uint8_t new_byte, int window_size) {
    hash = chk_buz_rotl(hash, 1);
    hash ^= chk_buz_rotl(chk_buz_table[old_byte], window_size);
    hash ^= chk_buz_table[new_byte];
    return hash;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1770: Buzhash should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1770: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1770: Should contain chk_ functions");
}

// ============================================================================
// C1771-C1775: Hash Applications
// ============================================================================

#[test]
fn c1771_bloom_filter() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint8_t bits[1024];
    int num_hashes;
    int size_bits;
} chk_bloom_t;

void chk_bloom_init(chk_bloom_t *bf, int num_hashes) {
    int i;
    for (i = 0; i < 1024; i++) {
        bf->bits[i] = 0;
    }
    bf->num_hashes = num_hashes;
    bf->size_bits = 1024 * 8;
}

uint32_t chk_bloom_hash(const uint8_t *data, size_t len, uint32_t seed) {
    uint32_t h = seed;
    size_t i;
    for (i = 0; i < len; i++) {
        h ^= data[i];
        h *= 0x01000193;
    }
    return h;
}

void chk_bloom_set_bit(chk_bloom_t *bf, int bit) {
    bf->bits[bit / 8] |= (1 << (bit % 8));
}

int chk_bloom_get_bit(const chk_bloom_t *bf, int bit) {
    return (bf->bits[bit / 8] >> (bit % 8)) & 1;
}

void chk_bloom_add(chk_bloom_t *bf, const uint8_t *data, size_t len) {
    int i;
    uint32_t h;
    for (i = 0; i < bf->num_hashes; i++) {
        h = chk_bloom_hash(data, len, (uint32_t)i * 0x9E3779B9);
        chk_bloom_set_bit(bf, (int)(h % bf->size_bits));
    }
}

int chk_bloom_maybe_contains(const chk_bloom_t *bf, const uint8_t *data, size_t len) {
    int i;
    uint32_t h;
    for (i = 0; i < bf->num_hashes; i++) {
        h = chk_bloom_hash(data, len, (uint32_t)i * 0x9E3779B9);
        if (!chk_bloom_get_bit(bf, (int)(h % bf->size_bits))) {
            return 0;
        }
    }
    return 1;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1771: Bloom filter should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1771: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1771: Should contain chk_ functions");
}

#[test]
fn c1772_count_min_sketch() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint32_t counters[4][256];
    int depth;
    int width;
} chk_cms_t;

void chk_cms_init(chk_cms_t *cms) {
    int i;
    int j;
    cms->depth = 4;
    cms->width = 256;
    for (i = 0; i < 4; i++) {
        for (j = 0; j < 256; j++) {
            cms->counters[i][j] = 0;
        }
    }
}

uint32_t chk_cms_hash(const uint8_t *data, size_t len, uint32_t seed) {
    uint32_t h = seed ^ 0xDEADBEEF;
    size_t i;
    for (i = 0; i < len; i++) {
        h = (h * 31) + data[i];
    }
    return h;
}

void chk_cms_add(chk_cms_t *cms, const uint8_t *data, size_t len, uint32_t count) {
    int i;
    uint32_t h;
    for (i = 0; i < cms->depth; i++) {
        h = chk_cms_hash(data, len, (uint32_t)i * 0x12345678);
        cms->counters[i][h % cms->width] += count;
    }
}

uint32_t chk_cms_estimate(const chk_cms_t *cms, const uint8_t *data, size_t len) {
    uint32_t min_val = 0xFFFFFFFF;
    int i;
    uint32_t h;
    uint32_t val;
    for (i = 0; i < cms->depth; i++) {
        h = chk_cms_hash(data, len, (uint32_t)i * 0x12345678);
        val = cms->counters[i][h % cms->width];
        if (val < min_val) {
            min_val = val;
        }
    }
    return min_val;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1772: Count-min sketch should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1772: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1772: Should contain chk_ functions");
}

#[test]
fn c1773_hyperloglog_cardinality() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint8_t registers[64];
    int num_registers;
    int precision;
} chk_hll_t;

void chk_hll_init(chk_hll_t *hll) {
    int i;
    hll->num_registers = 64;
    hll->precision = 6;
    for (i = 0; i < 64; i++) {
        hll->registers[i] = 0;
    }
}

uint32_t chk_hll_hash(const uint8_t *data, size_t len) {
    uint32_t h = 0x811C9DC5;
    size_t i;
    for (i = 0; i < len; i++) {
        h ^= data[i];
        h *= 0x01000193;
    }
    return h;
}

int chk_hll_leading_zeros(uint32_t value) {
    int count = 0;
    int i;
    for (i = 31; i >= 0; i--) {
        if (value & (1u << i)) {
            break;
        }
        count++;
    }
    return count;
}

void chk_hll_add(chk_hll_t *hll, const uint8_t *data, size_t len) {
    uint32_t h = chk_hll_hash(data, len);
    int idx = h & (hll->num_registers - 1);
    int rank = chk_hll_leading_zeros(h >> hll->precision) + 1;
    if ((int)hll->registers[idx] < rank) {
        hll->registers[idx] = (uint8_t)rank;
    }
}

uint32_t chk_hll_count(const chk_hll_t *hll) {
    int i;
    int zeros = 0;
    uint32_t sum_inv = 0;
    uint32_t estimate;
    for (i = 0; i < hll->num_registers; i++) {
        if (hll->registers[i] == 0) {
            zeros++;
            sum_inv += 1;
        }
    }
    estimate = (uint32_t)(hll->num_registers * hll->num_registers);
    if (zeros > 0) {
        estimate = (uint32_t)(hll->num_registers * 2);
    }
    return estimate;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1773: HyperLogLog cardinality should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1773: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1773: Should contain chk_ functions");
}

#[test]
fn c1774_consistent_hashing_ring() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint32_t hash_points[256];
    int node_ids[256];
    int count;
} chk_hashring_t;

void chk_hashring_init(chk_hashring_t *ring) {
    ring->count = 0;
}

uint32_t chk_hashring_hash(const uint8_t *data, size_t len) {
    uint32_t h = 5381;
    size_t i;
    for (i = 0; i < len; i++) {
        h = ((h << 5) + h) + data[i];
    }
    return h;
}

void chk_hashring_add_node(chk_hashring_t *ring, int node_id, uint32_t point) {
    int i;
    int pos = ring->count;
    if (ring->count >= 256) return;
    for (i = ring->count - 1; i >= 0; i--) {
        if (ring->hash_points[i] > point) {
            ring->hash_points[i + 1] = ring->hash_points[i];
            ring->node_ids[i + 1] = ring->node_ids[i];
            pos = i;
        } else {
            break;
        }
    }
    ring->hash_points[pos] = point;
    ring->node_ids[pos] = node_id;
    ring->count++;
}

int chk_hashring_lookup(const chk_hashring_t *ring, const uint8_t *key, size_t key_len) {
    uint32_t h;
    int i;
    if (ring->count == 0) return -1;
    h = chk_hashring_hash(key, key_len);
    for (i = 0; i < ring->count; i++) {
        if (ring->hash_points[i] >= h) {
            return ring->node_ids[i];
        }
    }
    return ring->node_ids[0];
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1774: Consistent hashing ring should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1774: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1774: Should contain chk_ functions");
}

#[test]
fn c1775_hash_table_linear_probing() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint32_t keys[128];
    uint32_t values[128];
    uint8_t occupied[128];
    int capacity;
    int count;
} chk_hashtable_t;

void chk_ht_init(chk_hashtable_t *ht) {
    int i;
    ht->capacity = 128;
    ht->count = 0;
    for (i = 0; i < 128; i++) {
        ht->occupied[i] = 0;
    }
}

int chk_ht_hash(uint32_t key, int capacity) {
    key = ((key >> 16) ^ key) * 0x45D9F3B;
    key = ((key >> 16) ^ key) * 0x45D9F3B;
    key = (key >> 16) ^ key;
    return (int)(key % (uint32_t)capacity);
}

int chk_ht_insert(chk_hashtable_t *ht, uint32_t key, uint32_t value) {
    int idx;
    int i;
    if (ht->count >= ht->capacity / 2) return -1;
    idx = chk_ht_hash(key, ht->capacity);
    for (i = 0; i < ht->capacity; i++) {
        int probe = (idx + i) % ht->capacity;
        if (!ht->occupied[probe]) {
            ht->keys[probe] = key;
            ht->values[probe] = value;
            ht->occupied[probe] = 1;
            ht->count++;
            return 0;
        }
        if (ht->occupied[probe] && ht->keys[probe] == key) {
            ht->values[probe] = value;
            return 0;
        }
    }
    return -1;
}

int chk_ht_lookup(const chk_hashtable_t *ht, uint32_t key, uint32_t *value) {
    int idx;
    int i;
    idx = chk_ht_hash(key, ht->capacity);
    for (i = 0; i < ht->capacity; i++) {
        int probe = (idx + i) % ht->capacity;
        if (!ht->occupied[probe]) return 0;
        if (ht->keys[probe] == key) {
            *value = ht->values[probe];
            return 1;
        }
    }
    return 0;
}

int chk_ht_remove(chk_hashtable_t *ht, uint32_t key) {
    int idx;
    int i;
    idx = chk_ht_hash(key, ht->capacity);
    for (i = 0; i < ht->capacity; i++) {
        int probe = (idx + i) % ht->capacity;
        if (!ht->occupied[probe]) return 0;
        if (ht->keys[probe] == key) {
            ht->occupied[probe] = 0;
            ht->count--;
            return 1;
        }
    }
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1775: Hash table linear probing should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1775: Output should not be empty");
    assert!(code.contains("fn chk_"), "C1775: Should contain chk_ functions");
}
