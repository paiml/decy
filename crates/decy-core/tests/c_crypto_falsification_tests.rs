//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C301-C325: Cryptography and Security patterns -- the kind of C code found
//! in crypto libraries, TLS implementations, and security tools.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world cryptographic programming patterns commonly
//! found in OpenSSL, libsodium, WolfSSL, mbedTLS, and similar crypto
//! libraries -- all expressed as valid C99.
//!
//! Organization:
//! - C301-C305: Symmetric ciphers and hash primitives
//! - C306-C310: Timing-safe operations and stream ciphers
//! - C311-C315: PRNG, encoding, ECC, block modes, key schedules
//! - C316-C320: MAC verification, nonce handling, secure erasure, ASN.1, password hashing
//! - C321-C325: Key exchange, TOTP, X.509, entropy, side-channel resistance
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C301-C305: Symmetric Ciphers and Hash Primitives
// ============================================================================

#[test]
fn c301_xor_cipher_with_key_rotation() {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct {
    uint8_t key[32];
    int key_len;
    int key_pos;
} xor_cipher_t;

void xor_cipher_init(xor_cipher_t *ctx, const uint8_t *key, int key_len) {
    int i;
    for (i = 0; i < key_len && i < 32; i++) {
        ctx->key[i] = key[i];
    }
    ctx->key_len = key_len < 32 ? key_len : 32;
    ctx->key_pos = 0;
}

void xor_cipher_process(xor_cipher_t *ctx, uint8_t *data, int len) {
    int i;
    for (i = 0; i < len; i++) {
        data[i] ^= ctx->key[ctx->key_pos];
        ctx->key_pos = (ctx->key_pos + 1) % ctx->key_len;
    }
}

void xor_cipher_reset(xor_cipher_t *ctx) {
    ctx->key_pos = 0;
}

int xor_cipher_selftest(void) {
    uint8_t key[4];
    uint8_t data[8];
    xor_cipher_t ctx;
    int i;
    key[0] = 0xAA; key[1] = 0xBB; key[2] = 0xCC; key[3] = 0xDD;
    for (i = 0; i < 8; i++) data[i] = (uint8_t)i;
    xor_cipher_init(&ctx, key, 4);
    xor_cipher_process(&ctx, data, 8);
    xor_cipher_reset(&ctx);
    xor_cipher_process(&ctx, data, 8);
    for (i = 0; i < 8; i++) {
        if (data[i] != (uint8_t)i) return -1;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C301: XOR cipher with key rotation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C301: Output should not be empty");
    assert!(
        code.contains("fn xor_cipher_init"),
        "C301: Should contain xor_cipher_init function"
    );
    assert!(
        code.contains("fn xor_cipher_process"),
        "C301: Should contain xor_cipher_process function"
    );
}

#[test]
fn c302_sha256_message_schedule() {
    let c_code = r#"
typedef unsigned int uint32_t;

uint32_t rotr32(uint32_t x, int n) {
    return (x >> n) | (x << (32 - n));
}

uint32_t sha256_sigma0(uint32_t x) {
    return rotr32(x, 7) ^ rotr32(x, 18) ^ (x >> 3);
}

uint32_t sha256_sigma1(uint32_t x) {
    return rotr32(x, 17) ^ rotr32(x, 19) ^ (x >> 10);
}

void sha256_expand_schedule(uint32_t W[64], const uint32_t M[16]) {
    int t;
    for (t = 0; t < 16; t++) {
        W[t] = M[t];
    }
    for (t = 16; t < 64; t++) {
        W[t] = sha256_sigma1(W[t - 2]) + W[t - 7]
             + sha256_sigma0(W[t - 15]) + W[t - 16];
    }
}

uint32_t sha256_ch(uint32_t x, uint32_t y, uint32_t z) {
    return (x & y) ^ (~x & z);
}

uint32_t sha256_maj(uint32_t x, uint32_t y, uint32_t z) {
    return (x & y) ^ (x & z) ^ (y & z);
}

uint32_t sha256_big_sigma0(uint32_t x) {
    return rotr32(x, 2) ^ rotr32(x, 13) ^ rotr32(x, 22);
}

uint32_t sha256_big_sigma1(uint32_t x) {
    return rotr32(x, 6) ^ rotr32(x, 11) ^ rotr32(x, 25);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C302: SHA-256 message schedule should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C302: Output should not be empty");
    assert!(
        code.contains("fn sha256_expand_schedule"),
        "C302: Should contain sha256_expand_schedule function"
    );
    assert!(
        code.contains("fn sha256_ch"),
        "C302: Should contain sha256_ch function"
    );
}

#[test]
fn c303_aes_subbytes_sbox_lookup() {
    let c_code = r#"
typedef unsigned char uint8_t;

static const uint8_t sbox[256] = {
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5,
    0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0,
    0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc,
    0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a,
    0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0,
    0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b,
    0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85,
    0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5,
    0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17,
    0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88,
    0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c,
    0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9,
    0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6,
    0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e,
    0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94,
    0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68,
    0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16
};

void aes_sub_bytes(uint8_t state[16]) {
    int i;
    for (i = 0; i < 16; i++) {
        state[i] = sbox[state[i]];
    }
}

void aes_shift_rows(uint8_t state[16]) {
    uint8_t tmp;
    tmp = state[1]; state[1] = state[5]; state[5] = state[9];
    state[9] = state[13]; state[13] = tmp;
    tmp = state[2]; state[2] = state[10]; state[10] = tmp;
    tmp = state[6]; state[6] = state[14]; state[14] = tmp;
    tmp = state[15]; state[15] = state[11]; state[11] = state[7];
    state[7] = state[3]; state[3] = tmp;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C303: AES SubBytes with S-box lookup should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C303: Output should not be empty");
    assert!(
        code.contains("fn aes_sub_bytes"),
        "C303: Should contain aes_sub_bytes function"
    );
}

#[test]
fn c304_rsa_modular_exponentiation() {
    let c_code = r#"
typedef unsigned long uint64_t;
typedef unsigned int uint32_t;

uint64_t mod_mul(uint64_t a, uint64_t b, uint64_t mod) {
    uint64_t result = 0;
    a = a % mod;
    while (b > 0) {
        if (b & 1) {
            result = (result + a) % mod;
        }
        a = (a * 2) % mod;
        b >>= 1;
    }
    return result;
}

uint64_t mod_exp(uint64_t base, uint64_t exp, uint64_t mod) {
    uint64_t result = 1;
    base = base % mod;
    if (base == 0) return 0;
    while (exp > 0) {
        if (exp & 1) {
            result = mod_mul(result, base, mod);
        }
        exp >>= 1;
        base = mod_mul(base, base, mod);
    }
    return result;
}

int is_probable_prime(uint64_t n, int rounds) {
    uint64_t d = n - 1;
    int r = 0;
    uint64_t bases[4];
    int i;
    if (n < 2) return 0;
    if (n == 2 || n == 3) return 1;
    if ((n & 1) == 0) return 0;
    while ((d & 1) == 0) {
        d >>= 1;
        r++;
    }
    bases[0] = 2; bases[1] = 3; bases[2] = 5; bases[3] = 7;
    for (i = 0; i < 4 && i < rounds; i++) {
        uint64_t a = bases[i];
        uint64_t x = mod_exp(a, d, n);
        int j;
        if (x == 1 || x == n - 1) continue;
        for (j = 0; j < r - 1; j++) {
            x = mod_mul(x, x, n);
            if (x == n - 1) break;
        }
        if (x != n - 1) return 0;
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C304: RSA modular exponentiation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C304: Output should not be empty");
    assert!(
        code.contains("fn mod_exp"),
        "C304: Should contain mod_exp function"
    );
    assert!(
        code.contains("fn is_probable_prime"),
        "C304: Should contain is_probable_prime function"
    );
}

#[test]
fn c305_hmac_construction() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

#define BLOCK_SIZE 64
#define HASH_SIZE 32

void simple_hash(const uint8_t *data, int len, uint8_t out[32]) {
    uint32_t h = 0x6a09e667;
    int i;
    for (i = 0; i < len; i++) {
        h ^= ((uint32_t)data[i]) << ((i % 4) * 8);
        h = (h << 5) | (h >> 27);
        h += 0x5be0cd19;
    }
    for (i = 0; i < 32; i++) {
        out[i] = (uint8_t)((h >> ((i % 4) * 8)) & 0xFF);
        if (i % 4 == 3) {
            h ^= h >> 16;
            h *= 0x85ebca6b;
        }
    }
}

void hmac_compute(const uint8_t *key, int key_len,
                  const uint8_t *msg, int msg_len,
                  uint8_t out[32]) {
    uint8_t k_pad[64];
    uint8_t inner_input[128];
    uint8_t inner_hash[32];
    uint8_t outer_input[96];
    int i;

    for (i = 0; i < 64; i++) {
        k_pad[i] = (i < key_len) ? key[i] : 0x00;
    }

    for (i = 0; i < 64; i++) {
        inner_input[i] = k_pad[i] ^ 0x36;
    }
    for (i = 0; i < msg_len && i < 64; i++) {
        inner_input[64 + i] = msg[i];
    }
    simple_hash(inner_input, 64 + (msg_len < 64 ? msg_len : 64), inner_hash);

    for (i = 0; i < 64; i++) {
        outer_input[i] = k_pad[i] ^ 0x5c;
    }
    for (i = 0; i < 32; i++) {
        outer_input[64 + i] = inner_hash[i];
    }
    simple_hash(outer_input, 96, out);
}

int hmac_verify(const uint8_t *key, int key_len,
                const uint8_t *msg, int msg_len,
                const uint8_t expected[32]) {
    uint8_t computed[32];
    int diff = 0;
    int i;
    hmac_compute(key, key_len, msg, msg_len, computed);
    for (i = 0; i < 32; i++) {
        diff |= computed[i] ^ expected[i];
    }
    return diff == 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C305: HMAC construction should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C305: Output should not be empty");
    assert!(
        code.contains("fn hmac_compute"),
        "C305: Should contain hmac_compute function"
    );
    assert!(
        code.contains("fn hmac_verify"),
        "C305: Should contain hmac_verify function"
    );
}

// ============================================================================
// C306-C310: Timing-Safe Operations and Stream Ciphers
// ============================================================================

#[test]
fn c306_constant_time_byte_comparison() {
    let c_code = r#"
typedef unsigned char uint8_t;

int ct_compare(const uint8_t *a, const uint8_t *b, int len) {
    uint8_t diff = 0;
    int i;
    for (i = 0; i < len; i++) {
        diff |= a[i] ^ b[i];
    }
    return (int)(1 & ((diff - 1) >> 8));
}

uint8_t ct_select(uint8_t mask, uint8_t a, uint8_t b) {
    return b ^ (mask & (a ^ b));
}

void ct_copy_if(uint8_t *dst, const uint8_t *src, int len, int condition) {
    uint8_t mask = (uint8_t)(-(condition != 0));
    int i;
    for (i = 0; i < len; i++) {
        dst[i] = ct_select(mask, src[i], dst[i]);
    }
}

int ct_is_zero(uint8_t x) {
    uint8_t v = x;
    v |= v >> 4;
    v |= v >> 2;
    v |= v >> 1;
    return 1 - (v & 1);
}

uint8_t ct_max(uint8_t a, uint8_t b) {
    uint8_t diff = a ^ b;
    uint8_t mask = (uint8_t)((int)(b - a) >> 8);
    return b ^ (diff & mask);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C306: Constant-time byte comparison should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C306: Output should not be empty");
    assert!(
        code.contains("fn ct_compare"),
        "C306: Should contain ct_compare function"
    );
    assert!(
        code.contains("fn ct_select"),
        "C306: Should contain ct_select function"
    );
}

#[test]
fn c307_pbkdf2_key_derivation() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

void simple_prf(const uint8_t *key, int key_len,
                const uint8_t *data, int data_len,
                uint8_t out[32]) {
    uint32_t h = 0x811c9dc5;
    int i;
    for (i = 0; i < key_len; i++) {
        h ^= key[i];
        h *= 0x01000193;
    }
    for (i = 0; i < data_len; i++) {
        h ^= data[i];
        h *= 0x01000193;
    }
    for (i = 0; i < 32; i++) {
        out[i] = (uint8_t)((h >> ((i % 4) * 8)) & 0xFF);
        if (i % 4 == 3) h = h * 0x6c078965 + 1;
    }
}

void xor_buffers(uint8_t *dst, const uint8_t *src, int len) {
    int i;
    for (i = 0; i < len; i++) {
        dst[i] ^= src[i];
    }
}

void pbkdf2_derive(const uint8_t *password, int pass_len,
                   const uint8_t *salt, int salt_len,
                   int iterations,
                   uint8_t *output, int out_len) {
    uint8_t U[32];
    uint8_t T[32];
    uint8_t salt_block[80];
    int block = 1;
    int offset = 0;
    int i, j, copy_len;

    while (offset < out_len) {
        for (i = 0; i < salt_len && i < 76; i++) {
            salt_block[i] = salt[i];
        }
        salt_block[salt_len] = (uint8_t)((block >> 24) & 0xFF);
        salt_block[salt_len + 1] = (uint8_t)((block >> 16) & 0xFF);
        salt_block[salt_len + 2] = (uint8_t)((block >> 8) & 0xFF);
        salt_block[salt_len + 3] = (uint8_t)(block & 0xFF);

        simple_prf(password, pass_len, salt_block, salt_len + 4, U);
        for (i = 0; i < 32; i++) T[i] = U[i];

        for (j = 1; j < iterations; j++) {
            simple_prf(password, pass_len, U, 32, U);
            xor_buffers(T, U, 32);
        }

        copy_len = out_len - offset;
        if (copy_len > 32) copy_len = 32;
        for (i = 0; i < copy_len; i++) {
            output[offset + i] = T[i];
        }
        offset += copy_len;
        block++;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C307: PBKDF2 key derivation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C307: Output should not be empty");
    assert!(
        code.contains("fn pbkdf2_derive"),
        "C307: Should contain pbkdf2_derive function"
    );
}

#[test]
fn c308_chacha20_quarter_round() {
    let c_code = r#"
typedef unsigned int uint32_t;

uint32_t rotl32(uint32_t v, int n) {
    return (v << n) | (v >> (32 - n));
}

void quarter_round(uint32_t *a, uint32_t *b, uint32_t *c, uint32_t *d) {
    *a += *b; *d ^= *a; *d = rotl32(*d, 16);
    *c += *d; *b ^= *c; *b = rotl32(*b, 12);
    *a += *b; *d ^= *a; *d = rotl32(*d, 8);
    *c += *d; *b ^= *c; *b = rotl32(*b, 7);
}

void chacha20_block(uint32_t state[16], uint32_t out[16]) {
    uint32_t x[16];
    int i;
    for (i = 0; i < 16; i++) x[i] = state[i];
    for (i = 0; i < 10; i++) {
        quarter_round(&x[0], &x[4], &x[8],  &x[12]);
        quarter_round(&x[1], &x[5], &x[9],  &x[13]);
        quarter_round(&x[2], &x[6], &x[10], &x[14]);
        quarter_round(&x[3], &x[7], &x[11], &x[15]);
        quarter_round(&x[0], &x[5], &x[10], &x[15]);
        quarter_round(&x[1], &x[6], &x[11], &x[12]);
        quarter_round(&x[2], &x[7], &x[8],  &x[13]);
        quarter_round(&x[3], &x[4], &x[9],  &x[14]);
    }
    for (i = 0; i < 16; i++) out[i] = x[i] + state[i];
}

void chacha20_init(uint32_t state[16],
                   const uint32_t key[8],
                   const uint32_t nonce[3],
                   uint32_t counter) {
    state[0]  = 0x61707865;
    state[1]  = 0x3320646e;
    state[2]  = 0x79622d32;
    state[3]  = 0x6b206574;
    state[4]  = key[0]; state[5]  = key[1];
    state[6]  = key[2]; state[7]  = key[3];
    state[8]  = key[4]; state[9]  = key[5];
    state[10] = key[6]; state[11] = key[7];
    state[12] = counter;
    state[13] = nonce[0];
    state[14] = nonce[1];
    state[15] = nonce[2];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C308: ChaCha20 quarter round should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C308: Output should not be empty");
    assert!(
        code.contains("fn quarter_round"),
        "C308: Should contain quarter_round function"
    );
    assert!(
        code.contains("fn chacha20_block"),
        "C308: Should contain chacha20_block function"
    );
}

#[test]
fn c309_galois_field_multiplication() {
    let c_code = r#"
typedef unsigned char uint8_t;

uint8_t gf256_mul(uint8_t a, uint8_t b) {
    uint8_t p = 0;
    int i;
    for (i = 0; i < 8; i++) {
        if (b & 1) {
            p ^= a;
        }
        int carry = a & 0x80;
        a <<= 1;
        if (carry) {
            a ^= 0x1B;
        }
        b >>= 1;
    }
    return p;
}

uint8_t gf256_inv(uint8_t a) {
    uint8_t r = a;
    int i;
    for (i = 0; i < 6; i++) {
        r = gf256_mul(r, r);
        r = gf256_mul(r, a);
    }
    r = gf256_mul(r, r);
    return r;
}

void gf256_mix_columns(uint8_t state[16]) {
    int col;
    for (col = 0; col < 4; col++) {
        uint8_t a0 = state[col * 4 + 0];
        uint8_t a1 = state[col * 4 + 1];
        uint8_t a2 = state[col * 4 + 2];
        uint8_t a3 = state[col * 4 + 3];
        state[col * 4 + 0] = gf256_mul(2, a0) ^ gf256_mul(3, a1) ^ a2 ^ a3;
        state[col * 4 + 1] = a0 ^ gf256_mul(2, a1) ^ gf256_mul(3, a2) ^ a3;
        state[col * 4 + 2] = a0 ^ a1 ^ gf256_mul(2, a2) ^ gf256_mul(3, a3);
        state[col * 4 + 3] = gf256_mul(3, a0) ^ a1 ^ a2 ^ gf256_mul(2, a3);
    }
}

void gf256_power_table(uint8_t table[256], uint8_t generator) {
    uint8_t val = 1;
    int i;
    for (i = 0; i < 256; i++) {
        table[i] = val;
        val = gf256_mul(val, generator);
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C309: Galois field multiplication should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C309: Output should not be empty");
    assert!(
        code.contains("fn gf256_mul"),
        "C309: Should contain gf256_mul function"
    );
    assert!(
        code.contains("fn gf256_mix_columns"),
        "C309: Should contain gf256_mix_columns function"
    );
}

#[test]
fn c310_certificate_chain_validation_state_machine() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

#define MAX_CHAIN_DEPTH 10

#define CERT_STATE_INIT         0
#define CERT_STATE_ISSUER_CHECK 1
#define CERT_STATE_TIME_CHECK   2
#define CERT_STATE_SIG_CHECK    3
#define CERT_STATE_VALID        4
#define CERT_STATE_INVALID      5

typedef struct {
    uint8_t subject_hash[32];
    uint8_t issuer_hash[32];
    uint32_t not_before;
    uint32_t not_after;
    uint8_t sig_hash[32];
    int is_ca;
    int path_len;
} cert_info_t;

typedef struct {
    int state;
    int depth;
    int max_depth;
    uint32_t current_time;
    int error_code;
} chain_validator_t;

void validator_init(chain_validator_t *v, uint32_t current_time, int max_depth) {
    v->state = CERT_STATE_INIT;
    v->depth = 0;
    v->max_depth = max_depth < MAX_CHAIN_DEPTH ? max_depth : MAX_CHAIN_DEPTH;
    v->current_time = current_time;
    v->error_code = 0;
}

int hashes_match(const uint8_t a[32], const uint8_t b[32]) {
    int diff = 0;
    int i;
    for (i = 0; i < 32; i++) {
        diff |= a[i] ^ b[i];
    }
    return diff == 0;
}

int validate_cert(chain_validator_t *v, const cert_info_t *cert,
                  const cert_info_t *issuer) {
    v->state = CERT_STATE_ISSUER_CHECK;
    if (!hashes_match(cert->issuer_hash, issuer->subject_hash)) {
        v->state = CERT_STATE_INVALID;
        v->error_code = 1;
        return 0;
    }

    v->state = CERT_STATE_TIME_CHECK;
    if (v->current_time < cert->not_before || v->current_time > cert->not_after) {
        v->state = CERT_STATE_INVALID;
        v->error_code = 2;
        return 0;
    }

    v->state = CERT_STATE_SIG_CHECK;
    if (!hashes_match(cert->sig_hash, issuer->subject_hash)) {
        v->state = CERT_STATE_INVALID;
        v->error_code = 3;
        return 0;
    }

    if (v->depth > 0 && !issuer->is_ca) {
        v->state = CERT_STATE_INVALID;
        v->error_code = 4;
        return 0;
    }

    if (issuer->path_len >= 0 && v->depth > issuer->path_len) {
        v->state = CERT_STATE_INVALID;
        v->error_code = 5;
        return 0;
    }

    v->depth++;
    if (v->depth >= v->max_depth) {
        v->state = CERT_STATE_INVALID;
        v->error_code = 6;
        return 0;
    }

    v->state = CERT_STATE_VALID;
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C310: Certificate chain validation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C310: Output should not be empty");
    assert!(
        code.contains("fn validator_init"),
        "C310: Should contain validator_init function"
    );
    assert!(
        code.contains("fn validate_cert"),
        "C310: Should contain validate_cert function"
    );
}

// ============================================================================
// C311-C315: PRNG, Encoding, ECC, Block Modes, Key Schedules
// ============================================================================

#[test]
fn c311_prng_xorshift128plus() {
    let c_code = r#"
typedef unsigned long long uint64_t;

typedef struct {
    uint64_t s0;
    uint64_t s1;
} xorshift128p_t;

void xorshift128p_seed(xorshift128p_t *rng, uint64_t seed0, uint64_t seed1) {
    rng->s0 = seed0;
    rng->s1 = seed1;
    if (rng->s0 == 0 && rng->s1 == 0) {
        rng->s0 = 1;
    }
}

uint64_t xorshift128p_next(xorshift128p_t *rng) {
    uint64_t s1 = rng->s0;
    uint64_t s0 = rng->s1;
    uint64_t result = s0 + s1;
    rng->s0 = s0;
    s1 ^= s1 << 23;
    rng->s1 = s1 ^ s0 ^ (s1 >> 17) ^ (s0 >> 26);
    return result;
}

uint64_t xorshift128p_bounded(xorshift128p_t *rng, uint64_t bound) {
    uint64_t threshold;
    uint64_t r;
    if (bound == 0) return 0;
    threshold = (0ULL - bound) % bound;
    do {
        r = xorshift128p_next(rng);
    } while (r < threshold);
    return r % bound;
}

void xorshift128p_fill(xorshift128p_t *rng, unsigned char *buf, int len) {
    int i;
    for (i = 0; i + 8 <= len; i += 8) {
        uint64_t val = xorshift128p_next(rng);
        buf[i]   = (unsigned char)(val & 0xFF);
        buf[i+1] = (unsigned char)((val >> 8) & 0xFF);
        buf[i+2] = (unsigned char)((val >> 16) & 0xFF);
        buf[i+3] = (unsigned char)((val >> 24) & 0xFF);
        buf[i+4] = (unsigned char)((val >> 32) & 0xFF);
        buf[i+5] = (unsigned char)((val >> 40) & 0xFF);
        buf[i+6] = (unsigned char)((val >> 48) & 0xFF);
        buf[i+7] = (unsigned char)((val >> 56) & 0xFF);
    }
    if (i < len) {
        uint64_t val = xorshift128p_next(rng);
        while (i < len) {
            buf[i] = (unsigned char)(val & 0xFF);
            val >>= 8;
            i++;
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C311: PRNG Xorshift128+ should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C311: Output should not be empty");
    assert!(
        code.contains("fn xorshift128p_next"),
        "C311: Should contain xorshift128p_next function"
    );
    assert!(
        code.contains("fn xorshift128p_fill"),
        "C311: Should contain xorshift128p_fill function"
    );
}

#[test]
fn c312_base64_encode_decode_tables() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

static const char b64_encode_table[64] = {
    'A','B','C','D','E','F','G','H','I','J','K','L','M',
    'N','O','P','Q','R','S','T','U','V','W','X','Y','Z',
    'a','b','c','d','e','f','g','h','i','j','k','l','m',
    'n','o','p','q','r','s','t','u','v','w','x','y','z',
    '0','1','2','3','4','5','6','7','8','9','+','/'
};

int b64_char_value(char c) {
    if (c >= 'A' && c <= 'Z') return c - 'A';
    if (c >= 'a' && c <= 'z') return c - 'a' + 26;
    if (c >= '0' && c <= '9') return c - '0' + 52;
    if (c == '+') return 62;
    if (c == '/') return 63;
    return -1;
}

int b64_encode(const uint8_t *input, int in_len, char *output, int out_max) {
    int i, j = 0;
    for (i = 0; i + 2 < in_len; i += 3) {
        uint32_t triple;
        if (j + 4 > out_max) return -1;
        triple = ((uint32_t)input[i] << 16) |
                 ((uint32_t)input[i+1] << 8) |
                 (uint32_t)input[i+2];
        output[j++] = b64_encode_table[(triple >> 18) & 0x3F];
        output[j++] = b64_encode_table[(triple >> 12) & 0x3F];
        output[j++] = b64_encode_table[(triple >> 6) & 0x3F];
        output[j++] = b64_encode_table[triple & 0x3F];
    }
    if (i < in_len) {
        uint32_t triple = (uint32_t)input[i] << 16;
        if (j + 4 > out_max) return -1;
        if (i + 1 < in_len) triple |= (uint32_t)input[i+1] << 8;
        output[j++] = b64_encode_table[(triple >> 18) & 0x3F];
        output[j++] = b64_encode_table[(triple >> 12) & 0x3F];
        output[j++] = (i + 1 < in_len) ? b64_encode_table[(triple >> 6) & 0x3F] : '=';
        output[j++] = '=';
    }
    if (j < out_max) output[j] = '\0';
    return j;
}

int b64_decode(const char *input, int in_len, uint8_t *output, int out_max) {
    int i, j = 0;
    for (i = 0; i + 3 < in_len; i += 4) {
        int a = b64_char_value(input[i]);
        int b = b64_char_value(input[i+1]);
        int c = (input[i+2] == '=') ? 0 : b64_char_value(input[i+2]);
        int d = (input[i+3] == '=') ? 0 : b64_char_value(input[i+3]);
        uint32_t triple;
        if (a < 0 || b < 0 || c < 0 || d < 0) return -1;
        triple = ((uint32_t)a << 18) | ((uint32_t)b << 12) |
                 ((uint32_t)c << 6) | (uint32_t)d;
        if (j < out_max) output[j++] = (uint8_t)((triple >> 16) & 0xFF);
        if (input[i+2] != '=' && j < out_max) output[j++] = (uint8_t)((triple >> 8) & 0xFF);
        if (input[i+3] != '=' && j < out_max) output[j++] = (uint8_t)(triple & 0xFF);
    }
    return j;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C312: Base64 encode/decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C312: Output should not be empty");
    assert!(
        code.contains("fn b64_encode"),
        "C312: Should contain b64_encode function"
    );
    assert!(
        code.contains("fn b64_decode"),
        "C312: Should contain b64_decode function"
    );
}

#[test]
fn c313_elliptic_curve_point_addition() {
    let c_code = r#"
typedef long long int64_t;

typedef struct {
    int64_t x;
    int64_t y;
    int is_infinity;
} ec_point_t;

static const int64_t EC_PRIME = 2147483647;

int64_t ec_mod(int64_t a, int64_t p) {
    int64_t r = a % p;
    if (r < 0) r += p;
    return r;
}

int64_t ec_mod_inv(int64_t a, int64_t p) {
    int64_t t = 0, newt = 1;
    int64_t r = p, newr = ec_mod(a, p);
    while (newr != 0) {
        int64_t q = r / newr;
        int64_t tmp;
        tmp = t - q * newt; t = newt; newt = tmp;
        tmp = r - q * newr; r = newr; newr = tmp;
    }
    return ec_mod(t, p);
}

ec_point_t ec_point_add(ec_point_t P, ec_point_t Q, int64_t a, int64_t p) {
    ec_point_t R;
    int64_t lambda, dx, dy;

    if (P.is_infinity) return Q;
    if (Q.is_infinity) return P;

    if (P.x == Q.x && P.y == Q.y) {
        if (P.y == 0) {
            R.x = 0; R.y = 0; R.is_infinity = 1;
            return R;
        }
        int64_t num = ec_mod(3 * ec_mod(P.x * P.x, p) + a, p);
        int64_t den = ec_mod(2 * P.y, p);
        lambda = ec_mod(num * ec_mod_inv(den, p), p);
    } else {
        dx = ec_mod(Q.x - P.x, p);
        dy = ec_mod(Q.y - P.y, p);
        if (dx == 0) {
            R.x = 0; R.y = 0; R.is_infinity = 1;
            return R;
        }
        lambda = ec_mod(dy * ec_mod_inv(dx, p), p);
    }

    R.x = ec_mod(lambda * lambda - P.x - Q.x, p);
    R.y = ec_mod(lambda * (P.x - R.x) - P.y, p);
    R.is_infinity = 0;
    return R;
}

ec_point_t ec_scalar_mul(ec_point_t P, int64_t k, int64_t a, int64_t p) {
    ec_point_t R;
    R.x = 0; R.y = 0; R.is_infinity = 1;
    while (k > 0) {
        if (k & 1) {
            R = ec_point_add(R, P, a, p);
        }
        P = ec_point_add(P, P, a, p);
        k >>= 1;
    }
    return R;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C313: Elliptic curve point addition should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C313: Output should not be empty");
    assert!(
        code.contains("fn ec_point_add"),
        "C313: Should contain ec_point_add function"
    );
    assert!(
        code.contains("fn ec_scalar_mul"),
        "C313: Should contain ec_scalar_mul function"
    );
}

#[test]
fn c314_cbc_encryption_with_iv() {
    let c_code = r#"
typedef unsigned char uint8_t;

void simple_block_encrypt(const uint8_t key[16], const uint8_t in[16], uint8_t out[16]) {
    int i;
    for (i = 0; i < 16; i++) {
        out[i] = in[i] ^ key[i];
        out[i] = (out[i] << 3) | (out[i] >> 5);
        out[i] ^= key[(i + 1) % 16];
    }
}

void simple_block_decrypt(const uint8_t key[16], const uint8_t in[16], uint8_t out[16]) {
    int i;
    for (i = 0; i < 16; i++) {
        uint8_t tmp = in[i] ^ key[(i + 1) % 16];
        tmp = (tmp >> 3) | (tmp << 5);
        out[i] = tmp ^ key[i];
    }
}

int cbc_encrypt(const uint8_t key[16], const uint8_t iv[16],
                const uint8_t *plaintext, int len,
                uint8_t *ciphertext) {
    uint8_t prev[16];
    uint8_t block[16];
    int blocks, b, i;

    if (len % 16 != 0) return -1;
    blocks = len / 16;

    for (i = 0; i < 16; i++) prev[i] = iv[i];

    for (b = 0; b < blocks; b++) {
        for (i = 0; i < 16; i++) {
            block[i] = plaintext[b * 16 + i] ^ prev[i];
        }
        simple_block_encrypt(key, block, &ciphertext[b * 16]);
        for (i = 0; i < 16; i++) {
            prev[i] = ciphertext[b * 16 + i];
        }
    }
    return blocks * 16;
}

int cbc_decrypt(const uint8_t key[16], const uint8_t iv[16],
                const uint8_t *ciphertext, int len,
                uint8_t *plaintext) {
    uint8_t prev[16];
    uint8_t decrypted[16];
    int blocks, b, i;

    if (len % 16 != 0) return -1;
    blocks = len / 16;

    for (i = 0; i < 16; i++) prev[i] = iv[i];

    for (b = 0; b < blocks; b++) {
        simple_block_decrypt(key, &ciphertext[b * 16], decrypted);
        for (i = 0; i < 16; i++) {
            plaintext[b * 16 + i] = decrypted[i] ^ prev[i];
            prev[i] = ciphertext[b * 16 + i];
        }
    }
    return blocks * 16;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C314: CBC encryption with IV should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C314: Output should not be empty");
    assert!(
        code.contains("fn cbc_encrypt"),
        "C314: Should contain cbc_encrypt function"
    );
    assert!(
        code.contains("fn cbc_decrypt"),
        "C314: Should contain cbc_decrypt function"
    );
}

#[test]
fn c315_aes_key_schedule_expansion() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

static const uint8_t aes_rcon[10] = {
    0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1B, 0x36
};

static const uint8_t ks_sbox[256] = {
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5,
    0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0,
    0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc,
    0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a,
    0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0,
    0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b,
    0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85,
    0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5,
    0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17,
    0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88,
    0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c,
    0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9,
    0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6,
    0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e,
    0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94,
    0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68,
    0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16
};

uint32_t ks_sub_word(uint32_t w) {
    uint8_t b0 = ks_sbox[(w >> 24) & 0xFF];
    uint8_t b1 = ks_sbox[(w >> 16) & 0xFF];
    uint8_t b2 = ks_sbox[(w >> 8) & 0xFF];
    uint8_t b3 = ks_sbox[w & 0xFF];
    return ((uint32_t)b0 << 24) | ((uint32_t)b1 << 16) |
           ((uint32_t)b2 << 8) | (uint32_t)b3;
}

uint32_t ks_rot_word(uint32_t w) {
    return (w << 8) | (w >> 24);
}

void aes128_key_expand(const uint8_t key[16], uint32_t round_keys[44]) {
    int i;
    for (i = 0; i < 4; i++) {
        round_keys[i] = ((uint32_t)key[4*i] << 24) |
                        ((uint32_t)key[4*i+1] << 16) |
                        ((uint32_t)key[4*i+2] << 8) |
                        (uint32_t)key[4*i+3];
    }
    for (i = 4; i < 44; i++) {
        uint32_t temp = round_keys[i - 1];
        if (i % 4 == 0) {
            temp = ks_sub_word(ks_rot_word(temp));
            temp ^= ((uint32_t)aes_rcon[i/4 - 1]) << 24;
        }
        round_keys[i] = round_keys[i - 4] ^ temp;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C315: AES key schedule expansion should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C315: Output should not be empty");
    assert!(
        code.contains("fn aes128_key_expand"),
        "C315: Should contain aes128_key_expand function"
    );
    assert!(
        code.contains("fn ks_sub_word"),
        "C315: Should contain ks_sub_word function"
    );
}

// ============================================================================
// C316-C320: MAC Verification, Nonce Handling, Secure Erasure, ASN.1, Password Hashing
// ============================================================================

#[test]
fn c316_mac_tag_verification_constant_time() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

#define TAG_SIZE 16

typedef struct {
    uint8_t key[16];
    uint32_t state;
} mac_ctx_t;

void mac_init(mac_ctx_t *ctx, const uint8_t key[16]) {
    int i;
    for (i = 0; i < 16; i++) {
        ctx->key[i] = key[i];
    }
    ctx->state = 0x12345678;
}

void mac_update(mac_ctx_t *ctx, const uint8_t *data, int len) {
    int i;
    for (i = 0; i < len; i++) {
        ctx->state ^= (uint32_t)data[i] << ((i % 4) * 8);
        ctx->state = (ctx->state << 7) | (ctx->state >> 25);
        ctx->state += 0x9e3779b9;
    }
}

void mac_finalize(mac_ctx_t *ctx, uint8_t tag[16]) {
    uint32_t h = ctx->state;
    int i;
    for (i = 0; i < 16; i++) {
        h ^= (uint32_t)ctx->key[i];
        h = (h << 5) | (h >> 27);
        h *= 0x85ebca6b;
        tag[i] = (uint8_t)((h >> ((i % 4) * 8)) & 0xFF);
    }
}

int mac_verify_ct(const uint8_t computed[16], const uint8_t expected[16]) {
    uint8_t diff = 0;
    int i;
    for (i = 0; i < 16; i++) {
        diff |= computed[i] ^ expected[i];
    }
    return (1 & ((diff - 1) >> 8));
}

int authenticate_and_verify(const uint8_t key[16],
                            const uint8_t *message, int msg_len,
                            const uint8_t expected_tag[16]) {
    mac_ctx_t ctx;
    uint8_t computed_tag[16];
    mac_init(&ctx, key);
    mac_update(&ctx, message, msg_len);
    mac_finalize(&ctx, computed_tag);
    return mac_verify_ct(computed_tag, expected_tag);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C316: MAC tag verification should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C316: Output should not be empty");
    assert!(
        code.contains("fn mac_verify_ct"),
        "C316: Should contain mac_verify_ct function"
    );
    assert!(
        code.contains("fn authenticate_and_verify"),
        "C316: Should contain authenticate_and_verify function"
    );
}

#[test]
fn c317_nonce_generation_with_counter() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

typedef struct {
    uint8_t prefix[8];
    uint64_t counter;
    uint32_t random_part;
} nonce_gen_t;

void nonce_gen_init(nonce_gen_t *ng, const uint8_t prefix[8], uint32_t random_seed) {
    int i;
    for (i = 0; i < 8; i++) {
        ng->prefix[i] = prefix[i];
    }
    ng->counter = 0;
    ng->random_part = random_seed;
}

void nonce_gen_next(nonce_gen_t *ng, uint8_t nonce[16]) {
    int i;
    for (i = 0; i < 8; i++) {
        nonce[i] = ng->prefix[i];
    }
    nonce[8]  = (uint8_t)((ng->counter >> 24) & 0xFF);
    nonce[9]  = (uint8_t)((ng->counter >> 16) & 0xFF);
    nonce[10] = (uint8_t)((ng->counter >> 8) & 0xFF);
    nonce[11] = (uint8_t)(ng->counter & 0xFF);

    ng->random_part ^= ng->random_part << 13;
    ng->random_part ^= ng->random_part >> 17;
    ng->random_part ^= ng->random_part << 5;

    nonce[12] = (uint8_t)((ng->random_part >> 24) & 0xFF);
    nonce[13] = (uint8_t)((ng->random_part >> 16) & 0xFF);
    nonce[14] = (uint8_t)((ng->random_part >> 8) & 0xFF);
    nonce[15] = (uint8_t)(ng->random_part & 0xFF);

    ng->counter++;
}

int nonce_gen_is_exhausted(const nonce_gen_t *ng) {
    return ng->counter >= 0xFFFFFFFF;
}

void nonce_gen_reset(nonce_gen_t *ng) {
    ng->counter = 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C317: Nonce generation with counter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C317: Output should not be empty");
    assert!(
        code.contains("fn nonce_gen_next"),
        "C317: Should contain nonce_gen_next function"
    );
    assert!(
        code.contains("fn nonce_gen_init"),
        "C317: Should contain nonce_gen_init function"
    );
}

#[test]
fn c318_secure_memory_zeroing() {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef unsigned char volatile vol_uint8_t;

void secure_zero(void *ptr, int len) {
    vol_uint8_t *p = (vol_uint8_t *)ptr;
    int i;
    for (i = 0; i < len; i++) {
        p[i] = 0;
    }
}

void secure_zero_pattern(void *ptr, int len) {
    vol_uint8_t *p = (vol_uint8_t *)ptr;
    int i;
    for (i = 0; i < len; i++) p[i] = 0xFF;
    for (i = 0; i < len; i++) p[i] = 0xAA;
    for (i = 0; i < len; i++) p[i] = 0x55;
    for (i = 0; i < len; i++) p[i] = 0x00;
}

int is_zeroed(const uint8_t *buf, int len) {
    uint8_t acc = 0;
    int i;
    for (i = 0; i < len; i++) {
        acc |= buf[i];
    }
    return acc == 0;
}

typedef struct {
    uint8_t data[256];
    int len;
    int sensitive;
} secure_buffer_t;

void secure_buffer_init(secure_buffer_t *sb) {
    int i;
    for (i = 0; i < 256; i++) sb->data[i] = 0;
    sb->len = 0;
    sb->sensitive = 1;
}

void secure_buffer_destroy(secure_buffer_t *sb) {
    if (sb->sensitive) {
        secure_zero(sb->data, 256);
    }
    sb->len = 0;
    sb->sensitive = 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C318: Secure memory zeroing should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C318: Output should not be empty");
    assert!(
        code.contains("fn secure_zero"),
        "C318: Should contain secure_zero function"
    );
    assert!(
        code.contains("fn secure_buffer_destroy"),
        "C318: Should contain secure_buffer_destroy function"
    );
}

#[test]
fn c319_asn1_der_tlv_parser() {
    let c_code = r#"
typedef unsigned char uint8_t;

#define ASN1_TAG_INTEGER     0x02
#define ASN1_TAG_BITSTRING   0x03
#define ASN1_TAG_OCTETSTRING 0x04
#define ASN1_TAG_NULL        0x05
#define ASN1_TAG_OID         0x06
#define ASN1_TAG_SEQUENCE    0x30
#define ASN1_TAG_SET         0x31

typedef struct {
    uint8_t tag;
    int length;
    int header_len;
    int total_len;
} asn1_tlv_t;

int asn1_parse_tlv(const uint8_t *data, int data_len, asn1_tlv_t *tlv) {
    int pos = 0;
    if (data_len < 2) return -1;

    tlv->tag = data[pos++];

    if (data[pos] < 0x80) {
        tlv->length = data[pos++];
    } else {
        int num_bytes = data[pos] & 0x7F;
        pos++;
        if (num_bytes > 4 || pos + num_bytes > data_len) return -1;
        tlv->length = 0;
        while (num_bytes > 0) {
            tlv->length = (tlv->length << 8) | data[pos++];
            num_bytes--;
        }
    }

    tlv->header_len = pos;
    tlv->total_len = pos + tlv->length;
    if (tlv->total_len > data_len) return -1;
    return 0;
}

int asn1_parse_integer(const uint8_t *data, int data_len, long *value) {
    asn1_tlv_t tlv;
    int i;
    long v;
    if (asn1_parse_tlv(data, data_len, &tlv) != 0) return -1;
    if (tlv.tag != ASN1_TAG_INTEGER) return -1;
    if (tlv.length > 8) return -1;

    v = 0;
    if (data[tlv.header_len] & 0x80) {
        v = -1;
    }
    for (i = 0; i < tlv.length; i++) {
        v = (v << 8) | data[tlv.header_len + i];
    }
    *value = v;
    return tlv.total_len;
}

int asn1_count_sequence_items(const uint8_t *data, int data_len) {
    asn1_tlv_t outer;
    int count = 0;
    int pos;
    if (asn1_parse_tlv(data, data_len, &outer) != 0) return -1;
    if (outer.tag != ASN1_TAG_SEQUENCE) return -1;

    pos = outer.header_len;
    while (pos < outer.total_len) {
        asn1_tlv_t inner;
        if (asn1_parse_tlv(data + pos, outer.total_len - pos, &inner) != 0) return -1;
        count++;
        pos += inner.total_len;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C319: ASN.1 DER TLV parser should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C319: Output should not be empty");
    assert!(
        code.contains("fn asn1_parse_tlv"),
        "C319: Should contain asn1_parse_tlv function"
    );
    assert!(
        code.contains("fn asn1_count_sequence_items"),
        "C319: Should contain asn1_count_sequence_items function"
    );
}

#[test]
fn c320_password_hashing_with_salt() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

uint32_t fnv1a_mix(const uint8_t *data, int len) {
    uint32_t h = 0x811c9dc5;
    int i;
    for (i = 0; i < len; i++) {
        h ^= data[i];
        h *= 0x01000193;
    }
    return h;
}

void password_hash(const uint8_t *password, int pass_len,
                   const uint8_t *salt, int salt_len,
                   int cost,
                   uint8_t output[32]) {
    uint8_t state[64];
    uint32_t h;
    int round, i, j;

    for (i = 0; i < 32 && i < pass_len; i++) {
        state[i] = password[i];
    }
    for (; i < 32; i++) {
        state[i] = 0;
    }
    for (i = 0; i < 32 && i < salt_len; i++) {
        state[32 + i] = salt[i];
    }
    for (; i < 32; i++) {
        state[32 + i] = 0;
    }

    for (round = 0; round < cost; round++) {
        h = fnv1a_mix(state, 64);
        for (i = 0; i < 64; i++) {
            state[i] ^= (uint8_t)((h >> ((i % 4) * 8)) & 0xFF);
            h = (h << 9) | (h >> 23);
            h += state[(i + 17) % 64];
        }
        for (j = 0; j < 4; j++) {
            for (i = 0; i < 63; i++) {
                state[i] ^= state[i + 1];
            }
            state[63] ^= state[0];
        }
    }

    for (i = 0; i < 32; i++) {
        output[i] = state[i] ^ state[32 + i];
    }
}

int password_verify(const uint8_t *password, int pass_len,
                    const uint8_t *salt, int salt_len,
                    int cost,
                    const uint8_t expected[32]) {
    uint8_t computed[32];
    uint8_t diff = 0;
    int i;
    password_hash(password, pass_len, salt, salt_len, cost, computed);
    for (i = 0; i < 32; i++) {
        diff |= computed[i] ^ expected[i];
    }
    return diff == 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C320: Password hashing with salt should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C320: Output should not be empty");
    assert!(
        code.contains("fn password_hash"),
        "C320: Should contain password_hash function"
    );
    assert!(
        code.contains("fn password_verify"),
        "C320: Should contain password_verify function"
    );
}

// ============================================================================
// C321-C325: Key Exchange, TOTP, X.509, Entropy, Side-Channel Resistance
// ============================================================================

#[test]
fn c321_diffie_hellman_key_exchange() {
    let c_code = r#"
typedef unsigned long long uint64_t;

uint64_t dh_mod_exp(uint64_t base, uint64_t exp, uint64_t mod) {
    uint64_t result = 1;
    base = base % mod;
    if (base == 0) return 0;
    while (exp > 0) {
        if (exp & 1) {
            result = (result * base) % mod;
        }
        exp >>= 1;
        base = (base * base) % mod;
    }
    return result;
}

typedef struct {
    uint64_t prime;
    uint64_t generator;
    uint64_t private_key;
    uint64_t public_key;
} dh_params_t;

void dh_init(dh_params_t *params, uint64_t prime, uint64_t generator, uint64_t private_key) {
    params->prime = prime;
    params->generator = generator;
    params->private_key = private_key;
    params->public_key = dh_mod_exp(generator, private_key, prime);
}

uint64_t dh_compute_shared_secret(const dh_params_t *params, uint64_t other_public) {
    return dh_mod_exp(other_public, params->private_key, params->prime);
}

int dh_validate_public_key(const dh_params_t *params, uint64_t pub_key) {
    if (pub_key < 2) return 0;
    if (pub_key >= params->prime - 1) return 0;
    if (dh_mod_exp(pub_key, (params->prime - 1) / 2, params->prime) != 1) {
        return 0;
    }
    return 1;
}

int dh_key_exchange_selftest(void) {
    dh_params_t alice, bob;
    uint64_t shared_a, shared_b;
    uint64_t p = 0x7FFFFFFF;
    uint64_t g = 5;
    dh_init(&alice, p, g, 12345);
    dh_init(&bob, p, g, 67890);
    shared_a = dh_compute_shared_secret(&alice, bob.public_key);
    shared_b = dh_compute_shared_secret(&bob, alice.public_key);
    return (shared_a == shared_b) ? 0 : -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C321: Diffie-Hellman key exchange should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C321: Output should not be empty");
    assert!(
        code.contains("fn dh_init"),
        "C321: Should contain dh_init function"
    );
    assert!(
        code.contains("fn dh_compute_shared_secret"),
        "C321: Should contain dh_compute_shared_secret function"
    );
}

#[test]
fn c322_totp_time_based_otp() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

void totp_hmac(const uint8_t *key, int key_len,
               const uint8_t *msg, int msg_len,
               uint8_t out[20]) {
    uint8_t k_ipad[64];
    uint8_t k_opad[64];
    uint32_t h = 0x67452301;
    int i;

    for (i = 0; i < 64; i++) {
        uint8_t kb = (i < key_len) ? key[i] : 0;
        k_ipad[i] = kb ^ 0x36;
        k_opad[i] = kb ^ 0x5c;
    }

    for (i = 0; i < 64; i++) {
        h ^= k_ipad[i];
        h = (h << 5) | (h >> 27);
        h += 0x5a827999;
    }
    for (i = 0; i < msg_len; i++) {
        h ^= msg[i];
        h = (h << 5) | (h >> 27);
        h += 0x6ed9eba1;
    }

    for (i = 0; i < 20; i++) {
        out[i] = (uint8_t)((h >> ((i % 4) * 8)) & 0xFF);
        if (i % 4 == 3) h = h * 0x8f1bbcdc + 1;
    }
}

uint32_t totp_truncate(const uint8_t hmac_result[20]) {
    int offset = hmac_result[19] & 0x0f;
    uint32_t code = ((uint32_t)(hmac_result[offset] & 0x7f) << 24)
                  | ((uint32_t)hmac_result[offset + 1] << 16)
                  | ((uint32_t)hmac_result[offset + 2] << 8)
                  | (uint32_t)hmac_result[offset + 3];
    return code % 1000000;
}

uint32_t totp_generate(const uint8_t *key, int key_len,
                       uint64_t timestamp, int time_step) {
    uint8_t msg[8];
    uint8_t hmac_out[20];
    uint64_t counter = timestamp / (uint64_t)time_step;
    int i;

    for (i = 7; i >= 0; i--) {
        msg[i] = (uint8_t)(counter & 0xFF);
        counter >>= 8;
    }

    totp_hmac(key, key_len, msg, 8, hmac_out);
    return totp_truncate(hmac_out);
}

int totp_verify(const uint8_t *key, int key_len,
                uint64_t timestamp, int time_step,
                uint32_t provided_code, int window) {
    int i;
    for (i = -window; i <= window; i++) {
        uint64_t ts = timestamp + (uint64_t)(i * time_step);
        if (totp_generate(key, key_len, ts, time_step) == provided_code) {
            return 1;
        }
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C322: TOTP generation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C322: Output should not be empty");
    assert!(
        code.contains("fn totp_generate"),
        "C322: Should contain totp_generate function"
    );
    assert!(
        code.contains("fn totp_verify"),
        "C322: Should contain totp_verify function"
    );
}

#[test]
fn c323_x509_certificate_field_extraction() {
    let c_code = r#"
typedef unsigned char uint8_t;

#define X509_TAG_SEQUENCE  0x30
#define X509_TAG_SET       0x31
#define X509_TAG_INTEGER   0x02
#define X509_TAG_BITSTRING 0x03
#define X509_TAG_OID       0x06
#define X509_TAG_UTF8STR   0x0C
#define X509_TAG_PRINTSTR  0x13
#define X509_TAG_UTCTIME   0x17

typedef struct {
    const uint8_t *data;
    int length;
} x509_field_t;

typedef struct {
    x509_field_t serial;
    x509_field_t issuer;
    x509_field_t subject;
    x509_field_t validity_start;
    x509_field_t validity_end;
    x509_field_t pubkey;
    int version;
} x509_cert_t;

int x509_read_length(const uint8_t *data, int max_len, int *length, int *header_bytes) {
    if (max_len < 1) return -1;
    if (data[0] < 0x80) {
        *length = data[0];
        *header_bytes = 1;
        return 0;
    }
    int num_bytes = data[0] & 0x7F;
    if (num_bytes > 4 || num_bytes + 1 > max_len) return -1;
    *length = 0;
    int i;
    for (i = 0; i < num_bytes; i++) {
        *length = (*length << 8) | data[1 + i];
    }
    *header_bytes = 1 + num_bytes;
    return 0;
}

int x509_skip_tag(const uint8_t *data, int max_len, uint8_t expected_tag,
                  const uint8_t **content, int *content_len) {
    int length, hdr;
    if (max_len < 2) return -1;
    if (data[0] != expected_tag) return -1;
    if (x509_read_length(data + 1, max_len - 1, &length, &hdr) != 0) return -1;
    *content = data + 1 + hdr;
    *content_len = length;
    return 1 + hdr + length;
}

int x509_extract_serial(const uint8_t *tbs, int tbs_len, x509_field_t *serial) {
    const uint8_t *content;
    int content_len;
    int consumed = x509_skip_tag(tbs, tbs_len, X509_TAG_INTEGER, &content, &content_len);
    if (consumed < 0) return -1;
    serial->data = content;
    serial->length = content_len;
    return consumed;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C323: X.509 certificate field extraction should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C323: Output should not be empty");
    assert!(
        code.contains("fn x509_read_length"),
        "C323: Should contain x509_read_length function"
    );
    assert!(
        code.contains("fn x509_extract_serial"),
        "C323: Should contain x509_extract_serial function"
    );
}

#[test]
fn c324_entropy_pool_mixing() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

#define POOL_SIZE 256

typedef struct {
    uint8_t pool[256];
    int write_pos;
    int entropy_bits;
    uint32_t mix_counter;
} entropy_pool_t;

void entropy_pool_init(entropy_pool_t *ep) {
    int i;
    for (i = 0; i < 256; i++) {
        ep->pool[i] = 0;
    }
    ep->write_pos = 0;
    ep->entropy_bits = 0;
    ep->mix_counter = 0;
}

void entropy_pool_add(entropy_pool_t *ep, const uint8_t *data, int len, int entropy_estimate) {
    int i;
    for (i = 0; i < len; i++) {
        ep->pool[ep->write_pos] ^= data[i];

        uint8_t left = ep->pool[(ep->write_pos + 1) % 256];
        uint8_t right = ep->pool[(ep->write_pos + 255) % 256];
        ep->pool[ep->write_pos] ^= (left << 3) ^ (right >> 5);

        ep->write_pos = (ep->write_pos + 1) % 256;
    }
    ep->entropy_bits += entropy_estimate;
    if (ep->entropy_bits > 256 * 8) ep->entropy_bits = 256 * 8;
}

void entropy_pool_mix(entropy_pool_t *ep) {
    uint32_t t = ep->mix_counter;
    int i, round;
    for (round = 0; round < 4; round++) {
        for (i = 0; i < 256; i++) {
            t ^= t << 13;
            t ^= t >> 17;
            t ^= t << 5;
            t += ep->pool[i];
            ep->pool[i] ^= (uint8_t)(t >> ((i % 4) * 8));
        }
    }
    ep->mix_counter++;
}

int entropy_pool_extract(entropy_pool_t *ep, uint8_t *output, int len) {
    int i;
    if (ep->entropy_bits < len * 8) return -1;
    entropy_pool_mix(ep);
    for (i = 0; i < len && i < 256; i++) {
        output[i] = ep->pool[i] ^ ep->pool[255 - i];
    }
    ep->entropy_bits -= len * 8;
    entropy_pool_mix(ep);
    return len;
}

int entropy_pool_has_enough(const entropy_pool_t *ep, int needed_bits) {
    return ep->entropy_bits >= needed_bits;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C324: Entropy pool mixing should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C324: Output should not be empty");
    assert!(
        code.contains("fn entropy_pool_add"),
        "C324: Should contain entropy_pool_add function"
    );
    assert!(
        code.contains("fn entropy_pool_extract"),
        "C324: Should contain entropy_pool_extract function"
    );
}

#[test]
fn c325_side_channel_resistant_table_lookup() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

uint8_t ct_table_lookup(const uint8_t table[256], uint8_t index) {
    uint8_t result = 0;
    int i;
    for (i = 0; i < 256; i++) {
        uint8_t mask = (uint8_t)(-(int)(i == (int)index));
        result |= table[i] & mask;
    }
    return result;
}

void ct_table_lookup_block(const uint8_t table[256], const uint8_t *indices,
                           uint8_t *output, int count) {
    int n;
    for (n = 0; n < count; n++) {
        output[n] = ct_table_lookup(table, indices[n]);
    }
}

uint32_t ct_table_lookup32(const uint32_t *table, int table_len, int index) {
    uint32_t result = 0;
    int i;
    for (i = 0; i < table_len; i++) {
        uint32_t mask = (uint32_t)(-(int)(i == index));
        result |= table[i] & mask;
    }
    return result;
}

void ct_scatter(uint8_t *table, int table_len, int index, uint8_t value) {
    int i;
    for (i = 0; i < table_len; i++) {
        uint8_t mask = (uint8_t)(-(int)(i == index));
        table[i] = (table[i] & ~mask) | (value & mask);
    }
}

void ct_conditional_swap(uint8_t *a, uint8_t *b, int len, int condition) {
    uint8_t mask = (uint8_t)(-(condition != 0));
    int i;
    for (i = 0; i < len; i++) {
        uint8_t diff = (a[i] ^ b[i]) & mask;
        a[i] ^= diff;
        b[i] ^= diff;
    }
}

int ct_array_contains(const uint8_t *arr, int len, uint8_t target) {
    uint8_t found = 0;
    int i;
    for (i = 0; i < len; i++) {
        found |= (uint8_t)(-(int)(arr[i] == target));
    }
    return found & 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C325: Side-channel resistant table lookup should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C325: Output should not be empty");
    assert!(
        code.contains("fn ct_table_lookup"),
        "C325: Should contain ct_table_lookup function"
    );
    assert!(
        code.contains("fn ct_conditional_swap"),
        "C325: Should contain ct_conditional_swap function"
    );
    assert!(
        code.contains("fn ct_array_contains"),
        "C325: Should contain ct_array_contains function"
    );
}
