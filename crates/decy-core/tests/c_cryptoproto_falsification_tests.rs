//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1376-C1400: Cryptographic Protocol patterns -- the kind of C code found
//! in TLS handshake implementations, key exchange protocols, authenticated
//! encryption schemes, and secure communication libraries.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! Organization:
//! - C1376-C1380: Symmetric ciphers (XOR, Caesar/ROT13, substitution, Vigenere, Feistel)
//! - C1381-C1385: Block cipher modes (ECB, CBC, CTR, PKCS7 padding, IV generation)
//! - C1386-C1390: Hash constructions (Merkle-Damgard, sponge, HMAC, hash chain, commitment)
//! - C1391-C1395: Key exchange (DH modular, key derivation, nonce gen, secret sharing, key schedule)
//! - C1396-C1400: Protocol primitives (MAC, challenge-response, token gen, session key, constant-time cmp)

// ============================================================================
// C1376-C1380: Symmetric Ciphers
// ============================================================================

#[test]
fn c1376_xor_cipher() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

void cp_xor_encrypt(uint8_t *data, size_t len, uint8_t key) {
    size_t i;
    for (i = 0; i < len; i++) {
        data[i] ^= key;
    }
}

void cp_xor_decrypt(uint8_t *data, size_t len, uint8_t key) {
    cp_xor_encrypt(data, len, key);
}

void cp_xor_roundtrip(uint8_t *buf, size_t len, uint8_t key) {
    cp_xor_encrypt(buf, len, key);
    cp_xor_decrypt(buf, len, key);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1376 failed: {:?}", result.err());
}

#[test]
fn c1377_caesar_rot13() {
    let c_code = r#"
typedef unsigned long size_t;

char cp_caesar_shift(char c, int shift) {
    if (c >= 'a' && c <= 'z')
        return 'a' + (c - 'a' + shift) % 26;
    if (c >= 'A' && c <= 'Z')
        return 'A' + (c - 'A' + shift) % 26;
    return c;
}

void cp_caesar_encrypt(char *text, size_t len, int shift) {
    size_t i;
    for (i = 0; i < len; i++) {
        text[i] = cp_caesar_shift(text[i], shift);
    }
}

void cp_rot13(char *text, size_t len) {
    cp_caesar_encrypt(text, len, 13);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1377 failed: {:?}", result.err());
}

#[test]
fn c1378_substitution_cipher() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

void cp_sub_build_table(const uint8_t *key, uint8_t *table) {
    int i;
    for (i = 0; i < 256; i++) {
        table[i] = key[i % 16] ^ (uint8_t)i;
    }
}

void cp_sub_encrypt(uint8_t *data, size_t len, const uint8_t *table) {
    size_t i;
    for (i = 0; i < len; i++) {
        data[i] = table[data[i]];
    }
}

void cp_sub_build_inverse(const uint8_t *table, uint8_t *inv) {
    int i;
    for (i = 0; i < 256; i++) {
        inv[table[i]] = (uint8_t)i;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1378 failed: {:?}", result.err());
}

#[test]
fn c1379_vigenere_cipher() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

void cp_vigenere_encrypt(uint8_t *data, size_t dlen, const uint8_t *key, size_t klen) {
    size_t i;
    if (klen == 0) return;
    for (i = 0; i < dlen; i++) {
        data[i] = (data[i] + key[i % klen]) & 0xFF;
    }
}

void cp_vigenere_decrypt(uint8_t *data, size_t dlen, const uint8_t *key, size_t klen) {
    size_t i;
    if (klen == 0) return;
    for (i = 0; i < dlen; i++) {
        data[i] = (data[i] - key[i % klen]) & 0xFF;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1379 failed: {:?}", result.err());
}

#[test]
fn c1380_simple_feistel() {
    let c_code = r#"
typedef unsigned int uint32_t;

uint32_t cp_feistel_f(uint32_t half, uint32_t subkey) {
    uint32_t x = half ^ subkey;
    x = ((x << 5) | (x >> 27)) ^ ((x << 13) | (x >> 19));
    return x + 0x9E3779B9;
}

void cp_feistel_round(uint32_t *left, uint32_t *right, uint32_t subkey) {
    uint32_t tmp = *right;
    *right = *left ^ cp_feistel_f(*right, subkey);
    *left = tmp;
}

void cp_feistel_encrypt(uint32_t *left, uint32_t *right, const uint32_t *keys, int rounds) {
    int i;
    for (i = 0; i < rounds; i++) {
        cp_feistel_round(left, right, keys[i]);
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1380 failed: {:?}", result.err());
}

// ============================================================================
// C1381-C1385: Block Cipher Modes
// ============================================================================

#[test]
fn c1381_ecb_mode() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

void cp_ecb_block(uint8_t *block, const uint8_t *key) {
    int i;
    for (i = 0; i < 16; i++) {
        block[i] ^= key[i];
        block[i] = (block[i] << 3) | (block[i] >> 5);
    }
}

void cp_ecb_encrypt(uint8_t *data, size_t len, const uint8_t *key) {
    size_t off;
    for (off = 0; off + 16 <= len; off += 16) {
        cp_ecb_block(data + off, key);
    }
}

int cp_ecb_nblocks(size_t len) {
    return (int)(len / 16);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1381 failed: {:?}", result.err());
}

#[test]
fn c1382_cbc_mode() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

void cp_xor_block(uint8_t *dst, const uint8_t *src, int blen) {
    int i;
    for (i = 0; i < blen; i++) {
        dst[i] ^= src[i];
    }
}

void cp_cbc_encrypt(uint8_t *data, size_t len, const uint8_t *key, uint8_t *iv) {
    size_t off;
    for (off = 0; off + 16 <= len; off += 16) {
        cp_xor_block(data + off, iv, 16);
        cp_xor_block(data + off, key, 16);
        iv = data + off;
    }
}

void cp_copy_block(uint8_t *dst, const uint8_t *src) {
    int i;
    for (i = 0; i < 16; i++) dst[i] = src[i];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1382 failed: {:?}", result.err());
}

#[test]
fn c1383_ctr_mode() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

void cp_inc_counter(uint8_t *ctr) {
    int i;
    for (i = 15; i >= 0; i--) {
        ctr[i]++;
        if (ctr[i] != 0) break;
    }
}

void cp_ctr_keystream(uint8_t *ks, const uint8_t *ctr, const uint8_t *key) {
    int i;
    for (i = 0; i < 16; i++) {
        ks[i] = ctr[i] ^ key[i];
    }
}

void cp_ctr_encrypt(uint8_t *data, size_t len, const uint8_t *key, uint8_t *ctr) {
    uint8_t ks[16];
    size_t i;
    size_t off;
    for (off = 0; off < len; off += 16) {
        cp_ctr_keystream(ks, ctr, key);
        for (i = 0; i < 16 && off + i < len; i++) {
            data[off + i] ^= ks[i];
        }
        cp_inc_counter(ctr);
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1383 failed: {:?}", result.err());
}

#[test]
fn c1384_pkcs7_padding() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

size_t cp_pkcs7_pad(uint8_t *data, size_t len, int block_size) {
    int pad_val = block_size - (int)(len % block_size);
    int i;
    for (i = 0; i < pad_val; i++) {
        data[len + i] = (uint8_t)pad_val;
    }
    return len + pad_val;
}

int cp_pkcs7_unpad(const uint8_t *data, size_t len) {
    uint8_t pad_val;
    int i;
    if (len == 0) return -1;
    pad_val = data[len - 1];
    if (pad_val == 0 || pad_val > 16) return -1;
    for (i = 0; i < pad_val; i++) {
        if (data[len - 1 - i] != pad_val) return -1;
    }
    return (int)(len - pad_val);
}

int cp_pkcs7_valid(const uint8_t *data, size_t len) {
    return cp_pkcs7_unpad(data, len) >= 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1384 failed: {:?}", result.err());
}

#[test]
fn c1385_iv_generation() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

static uint32_t cp_iv_state = 12345;

uint32_t cp_iv_next(void) {
    cp_iv_state ^= cp_iv_state << 13;
    cp_iv_state ^= cp_iv_state >> 17;
    cp_iv_state ^= cp_iv_state << 5;
    return cp_iv_state;
}

void cp_generate_iv(uint8_t *iv, int len) {
    int i;
    for (i = 0; i < len; i++) {
        iv[i] = (uint8_t)(cp_iv_next() & 0xFF);
    }
}

void cp_iv_seed(uint32_t seed) {
    cp_iv_state = seed;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1385 failed: {:?}", result.err());
}

// ============================================================================
// C1386-C1390: Hash Constructions
// ============================================================================

#[test]
fn c1386_merkle_damgard() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

uint32_t cp_compress(uint32_t state, uint32_t block) {
    uint32_t x = state ^ block;
    x = x * 0x45D9F3B + 0x1B873593;
    x = (x << 13) | (x >> 19);
    return x * 5 + 0xE6546B64;
}

uint32_t cp_md_hash(const uint32_t *blocks, int nblocks) {
    uint32_t state = 0x811C9DC5;
    int i;
    for (i = 0; i < nblocks; i++) {
        state = cp_compress(state, blocks[i]);
    }
    return state;
}

uint32_t cp_md_finalize(uint32_t state, uint32_t length) {
    return cp_compress(state, length);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1386 failed: {:?}", result.err());
}

#[test]
fn c1387_sponge_construction() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

typedef struct {
    uint8_t state[25];
    int rate;
    int pos;
} cp_sponge_t;

void cp_sponge_init(cp_sponge_t *s, int rate) {
    int i;
    for (i = 0; i < 25; i++) s->state[i] = 0;
    s->rate = rate;
    s->pos = 0;
}

void cp_sponge_permute(uint8_t *state) {
    int i;
    for (i = 0; i < 25; i++) {
        state[i] ^= (state[(i + 1) % 25] << 1) | (state[(i + 1) % 25] >> 7);
    }
}

void cp_sponge_absorb(cp_sponge_t *s, const uint8_t *data, size_t len) {
    size_t i;
    for (i = 0; i < len; i++) {
        s->state[s->pos] ^= data[i];
        s->pos++;
        if (s->pos >= s->rate) {
            cp_sponge_permute(s->state);
            s->pos = 0;
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1387 failed: {:?}", result.err());
}

#[test]
fn c1388_hmac_construction() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

uint32_t cp_simple_hash(const uint8_t *data, size_t len) {
    uint32_t h = 0x811C9DC5;
    size_t i;
    for (i = 0; i < len; i++) {
        h ^= data[i];
        h *= 0x01000193;
    }
    return h;
}

void cp_hmac_pad(uint8_t *pad, const uint8_t *key, int klen, uint8_t val) {
    int i;
    for (i = 0; i < 64; i++) {
        pad[i] = (i < klen ? key[i] : 0) ^ val;
    }
}

uint32_t cp_hmac(const uint8_t *key, int klen, const uint8_t *msg, size_t mlen) {
    uint8_t ipad[64];
    uint8_t opad[64];
    uint32_t inner;
    cp_hmac_pad(ipad, key, klen, 0x36);
    cp_hmac_pad(opad, key, klen, 0x5C);
    inner = cp_simple_hash(ipad, 64) ^ cp_simple_hash(msg, mlen);
    return cp_simple_hash(opad, 64) ^ inner;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1388 failed: {:?}", result.err());
}

#[test]
fn c1389_hash_chain() {
    let c_code = r#"
typedef unsigned int uint32_t;

uint32_t cp_hash_step(uint32_t val) {
    val ^= val << 13;
    val ^= val >> 17;
    val ^= val << 5;
    return val;
}

uint32_t cp_hash_chain(uint32_t seed, int depth) {
    int i;
    uint32_t h = seed;
    for (i = 0; i < depth; i++) {
        h = cp_hash_step(h);
    }
    return h;
}

int cp_hash_chain_verify(uint32_t start, uint32_t end, int depth) {
    return cp_hash_chain(start, depth) == end;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1389 failed: {:?}", result.err());
}

#[test]
fn c1390_commitment_scheme() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

uint32_t cp_mix(uint32_t a, uint32_t b) {
    a ^= b;
    a *= 0x45D9F3B;
    a = (a << 16) | (a >> 16);
    return a;
}

uint32_t cp_commit(uint32_t value, uint32_t nonce) {
    return cp_mix(value, nonce) ^ cp_mix(nonce, value);
}

int cp_verify_commit(uint32_t commitment, uint32_t value, uint32_t nonce) {
    return cp_commit(value, nonce) == commitment;
}

uint32_t cp_commit_hash(const uint8_t *data, int len, uint32_t nonce) {
    uint32_t h = nonce;
    int i;
    for (i = 0; i < len; i++) {
        h = cp_mix(h, data[i]);
    }
    return h;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1390 failed: {:?}", result.err());
}

// ============================================================================
// C1391-C1395: Key Exchange
// ============================================================================

#[test]
fn c1391_dh_modular() {
    let c_code = r#"
typedef unsigned long uint64_t;

uint64_t cp_modpow(uint64_t base, uint64_t exp, uint64_t mod) {
    uint64_t result = 1;
    base = base % mod;
    while (exp > 0) {
        if (exp & 1) {
            result = (result * base) % mod;
        }
        exp >>= 1;
        base = (base * base) % mod;
    }
    return result;
}

uint64_t cp_dh_public(uint64_t g, uint64_t priv_key, uint64_t p) {
    return cp_modpow(g, priv_key, p);
}

uint64_t cp_dh_shared(uint64_t other_pub, uint64_t priv_key, uint64_t p) {
    return cp_modpow(other_pub, priv_key, p);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1391 failed: {:?}", result.err());
}

#[test]
fn c1392_key_derivation() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

uint32_t cp_prf(uint32_t key, uint32_t counter) {
    uint32_t x = key ^ counter;
    x *= 0x45D9F3B;
    x ^= x >> 16;
    x *= 0x45D9F3B;
    return x;
}

void cp_kdf_expand(const uint32_t master, uint8_t *out, size_t outlen) {
    uint32_t counter = 1;
    size_t pos = 0;
    while (pos < outlen) {
        uint32_t block = cp_prf(master, counter);
        size_t i;
        for (i = 0; i < 4 && pos < outlen; i++) {
            out[pos++] = (uint8_t)(block >> (i * 8));
        }
        counter++;
    }
}

uint32_t cp_kdf_extract(uint32_t salt, uint32_t ikm) {
    return cp_prf(salt, ikm) ^ cp_prf(ikm, salt);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1392 failed: {:?}", result.err());
}

#[test]
fn c1393_nonce_generation() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint32_t counter;
    uint32_t epoch;
} cp_nonce_ctx_t;

void cp_nonce_init(cp_nonce_ctx_t *ctx, uint32_t epoch) {
    ctx->counter = 0;
    ctx->epoch = epoch;
}

uint32_t cp_nonce_next(cp_nonce_ctx_t *ctx) {
    uint32_t n = ctx->epoch ^ ctx->counter;
    ctx->counter++;
    n *= 0x9E3779B9;
    n ^= n >> 16;
    return n;
}

void cp_nonce_fill(cp_nonce_ctx_t *ctx, uint8_t *buf, int len) {
    int i;
    for (i = 0; i < len; i++) {
        if (i % 4 == 0) {
            uint32_t n = cp_nonce_next(ctx);
            buf[i] = (uint8_t)(n & 0xFF);
        } else {
            buf[i] = buf[i - 1] ^ (uint8_t)(i * 37);
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1393 failed: {:?}", result.err());
}

#[test]
fn c1394_secret_sharing() {
    let c_code = r#"
typedef unsigned int uint32_t;

uint32_t cp_gf_mul(uint32_t a, uint32_t b, uint32_t prime) {
    uint32_t result = 0;
    a = a % prime;
    while (b > 0) {
        if (b & 1) result = (result + a) % prime;
        a = (a * 2) % prime;
        b >>= 1;
    }
    return result;
}

uint32_t cp_poly_eval(const uint32_t *coeffs, int degree, uint32_t x, uint32_t prime) {
    uint32_t result = coeffs[degree];
    int i;
    for (i = degree - 1; i >= 0; i--) {
        result = (cp_gf_mul(result, x, prime) + coeffs[i]) % prime;
    }
    return result;
}

void cp_share_generate(const uint32_t *coeffs, int degree, uint32_t prime,
                       uint32_t *xs, uint32_t *ys, int n) {
    int i;
    for (i = 0; i < n; i++) {
        xs[i] = (uint32_t)(i + 1);
        ys[i] = cp_poly_eval(coeffs, degree, xs[i], prime);
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1394 failed: {:?}", result.err());
}

#[test]
fn c1395_key_schedule() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

uint32_t cp_ks_rotl(uint32_t x, int n) {
    return (x << n) | (x >> (32 - n));
}

void cp_key_schedule(const uint8_t *master, uint32_t *round_keys, int rounds) {
    int i;
    uint32_t prev;
    round_keys[0] = ((uint32_t)master[0] << 24) | ((uint32_t)master[1] << 16)
                   | ((uint32_t)master[2] << 8) | master[3];
    for (i = 1; i < rounds; i++) {
        prev = round_keys[i - 1];
        round_keys[i] = cp_ks_rotl(prev, 3) ^ (prev >> 5) ^ (uint32_t)i;
    }
}

uint32_t cp_subkey(const uint32_t *round_keys, int round) {
    return round_keys[round] ^ 0xDEADBEEF;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1395 failed: {:?}", result.err());
}

// ============================================================================
// C1396-C1400: Protocol Primitives
// ============================================================================

#[test]
fn c1396_message_auth_code() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

uint32_t cp_mac_round(uint32_t state, uint8_t byte) {
    state ^= byte;
    state *= 0x01000193;
    state ^= state >> 16;
    return state;
}

uint32_t cp_mac_compute(const uint8_t *key, int klen, const uint8_t *msg, size_t mlen) {
    uint32_t state = 0xCAFEBABE;
    int i;
    size_t j;
    for (i = 0; i < klen; i++) {
        state = cp_mac_round(state, key[i]);
    }
    for (j = 0; j < mlen; j++) {
        state = cp_mac_round(state, msg[j]);
    }
    return state;
}

int cp_mac_verify(const uint8_t *key, int klen, const uint8_t *msg, size_t mlen, uint32_t tag) {
    return cp_mac_compute(key, klen, msg, mlen) == tag;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1396 failed: {:?}", result.err());
}

#[test]
fn c1397_challenge_response() {
    let c_code = r#"
typedef unsigned int uint32_t;

uint32_t cp_cr_hash(uint32_t input, uint32_t secret) {
    uint32_t x = input ^ secret;
    x *= 0x45D9F3B;
    x ^= x >> 16;
    x *= 0x45D9F3B;
    x ^= x >> 16;
    return x;
}

uint32_t cp_cr_respond(uint32_t challenge, uint32_t secret) {
    return cp_cr_hash(challenge, secret);
}

int cp_cr_verify(uint32_t challenge, uint32_t response, uint32_t secret) {
    return cp_cr_respond(challenge, secret) == response;
}

uint32_t cp_cr_gen_challenge(uint32_t seed) {
    seed ^= seed << 13;
    seed ^= seed >> 17;
    seed ^= seed << 5;
    return seed;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1397 failed: {:?}", result.err());
}

#[test]
fn c1398_token_generation() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint32_t user_id;
    uint32_t timestamp;
    uint32_t nonce;
} cp_token_t;

uint32_t cp_token_sign(const cp_token_t *tok, uint32_t secret) {
    uint32_t h = secret;
    h ^= tok->user_id * 0x45D9F3B;
    h ^= tok->timestamp * 0x01000193;
    h ^= tok->nonce * 0x9E3779B9;
    return h;
}

int cp_token_valid(const cp_token_t *tok, uint32_t sig, uint32_t secret) {
    return cp_token_sign(tok, secret) == sig;
}

int cp_token_expired(const cp_token_t *tok, uint32_t now, uint32_t ttl) {
    return (now - tok->timestamp) > ttl;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1398 failed: {:?}", result.err());
}

#[test]
fn c1399_session_key_mgmt() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t keys[4];
    int active;
    int count;
} cp_session_t;

void cp_session_init(cp_session_t *s) {
    int i;
    for (i = 0; i < 4; i++) s->keys[i] = 0;
    s->active = 0;
    s->count = 0;
}

void cp_session_rotate(cp_session_t *s, uint32_t new_key) {
    s->active = (s->active + 1) % 4;
    s->keys[s->active] = new_key;
    if (s->count < 4) s->count++;
}

uint32_t cp_session_current(const cp_session_t *s) {
    return s->keys[s->active];
}

int cp_session_find(const cp_session_t *s, uint32_t key) {
    int i;
    for (i = 0; i < s->count; i++) {
        if (s->keys[i] == key) return 1;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1399 failed: {:?}", result.err());
}

#[test]
fn c1400_constant_time_compare() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

int cp_ct_compare(const uint8_t *a, const uint8_t *b, size_t len) {
    uint8_t diff = 0;
    size_t i;
    for (i = 0; i < len; i++) {
        diff |= a[i] ^ b[i];
    }
    return diff == 0;
}

int cp_ct_is_zero(const uint8_t *buf, size_t len) {
    uint8_t acc = 0;
    size_t i;
    for (i = 0; i < len; i++) {
        acc |= buf[i];
    }
    return acc == 0;
}

void cp_ct_select(uint8_t *out, const uint8_t *a, const uint8_t *b, size_t len, int sel) {
    uint8_t mask = (uint8_t)(-(sel & 1));
    size_t i;
    for (i = 0; i < len; i++) {
        out[i] = (a[i] & ~mask) | (b[i] & mask);
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1400 failed: {:?}", result.err());
}
