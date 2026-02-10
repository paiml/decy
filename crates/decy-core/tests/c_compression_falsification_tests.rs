//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C526-C550: Compression and Encoding algorithms -- the kind of C code found
//! in zlib, brotli, lz4, base64 libraries, and data compression toolkits.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world compression and encoding patterns commonly
//! found in zlib, liblzma, libbrotli, and similar libraries -- all expressed
//! as valid C99 with array-based representations (no malloc/free).
//!
//! Organization:
//! - C526-C530: Basic compression (RLE, Huffman, LZ77, LZW)
//! - C531-C535: Encoding and checksums (Base64, CRC32, Adler32, DEFLATE)
//! - C536-C540: Advanced coding (arithmetic, delta, BWT, MTF, bitstream)
//! - C541-C545: Variable-length codes (bitstream reader, varint, Golomb-Rice, Elias gamma, Fibonacci)
//! - C546-C550: Specialized (BPE, PPM, range encoder, Shannon entropy, JPEG quantization)
//!
//! ## Results
//! - 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C526-C530: Basic Compression (RLE, Huffman, LZ77, LZW)
// ============================================================================

/// C526: Run-length encoding -- compresses repeated byte sequences
#[test]
fn c526_run_length_encoding() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct {
    uint8_t data[512];
    int len;
} rle_buf_t;

int rle_encode(const uint8_t *input, int in_len, uint8_t *output, int max_out) {
    int i = 0;
    int out_pos = 0;
    while (i < in_len && out_pos + 2 <= max_out) {
        uint8_t cur = input[i];
        int count = 1;
        while (i + count < in_len && input[i + count] == cur && count < 255) {
            count++;
        }
        output[out_pos] = (uint8_t)count;
        output[out_pos + 1] = cur;
        out_pos += 2;
        i += count;
    }
    return out_pos;
}

int rle_decode(const uint8_t *input, int in_len, uint8_t *output, int max_out) {
    int i = 0;
    int out_pos = 0;
    while (i + 1 < in_len) {
        int count = input[i];
        uint8_t val = input[i + 1];
        int j;
        for (j = 0; j < count && out_pos < max_out; j++) {
            output[out_pos] = val;
            out_pos++;
        }
        i += 2;
    }
    return out_pos;
}

int rle_roundtrip_test(void) {
    uint8_t input[16];
    uint8_t encoded[64];
    uint8_t decoded[64];
    int i;
    for (i = 0; i < 8; i++) input[i] = 0xAA;
    for (i = 8; i < 16; i++) input[i] = 0xBB;
    int enc_len = rle_encode(input, 16, encoded, 64);
    int dec_len = rle_decode(encoded, enc_len, decoded, 64);
    if (dec_len != 16) return -1;
    for (i = 0; i < 16; i++) {
        if (decoded[i] != input[i]) return -2;
    }
    return 0;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C526: Should produce output");
    assert!(
        rust_code.contains("fn rle_encode"),
        "C526: Should contain rle_encode function"
    );
    assert!(
        rust_code.contains("fn rle_decode"),
        "C526: Should contain rle_decode function"
    );
    Ok(())
}

/// C527: Huffman tree building -- construct frequency-based binary tree
#[test]
fn c527_huffman_tree_building() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int freq;
    int left;
    int right;
    int parent;
    int symbol;
} huff_node_t;

typedef struct {
    huff_node_t nodes[512];
    int count;
} huff_tree_t;

void huff_init(huff_tree_t *tree) {
    int i;
    tree->count = 0;
    for (i = 0; i < 512; i++) {
        tree->nodes[i].freq = 0;
        tree->nodes[i].left = -1;
        tree->nodes[i].right = -1;
        tree->nodes[i].parent = -1;
        tree->nodes[i].symbol = -1;
    }
}

int huff_add_leaf(huff_tree_t *tree, int symbol, int freq) {
    if (tree->count >= 512) return -1;
    int idx = tree->count;
    tree->nodes[idx].symbol = symbol;
    tree->nodes[idx].freq = freq;
    tree->nodes[idx].left = -1;
    tree->nodes[idx].right = -1;
    tree->nodes[idx].parent = -1;
    tree->count++;
    return idx;
}

int huff_find_min(const huff_tree_t *tree, int exclude) {
    int min_idx = -1;
    int min_freq = 2147483647;
    int i;
    for (i = 0; i < tree->count; i++) {
        if (i == exclude) continue;
        if (tree->nodes[i].parent != -1) continue;
        if (tree->nodes[i].freq < min_freq) {
            min_freq = tree->nodes[i].freq;
            min_idx = i;
        }
    }
    return min_idx;
}

int huff_build_step(huff_tree_t *tree) {
    int left = huff_find_min(tree, -1);
    if (left == -1) return -1;
    int right = huff_find_min(tree, left);
    if (right == -1) return -1;
    int parent = tree->count;
    if (parent >= 512) return -1;
    tree->nodes[parent].freq = tree->nodes[left].freq + tree->nodes[right].freq;
    tree->nodes[parent].left = left;
    tree->nodes[parent].right = right;
    tree->nodes[parent].symbol = -1;
    tree->nodes[parent].parent = -1;
    tree->nodes[left].parent = parent;
    tree->nodes[right].parent = parent;
    tree->count++;
    return parent;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C527: Should produce output");
    assert!(
        rust_code.contains("fn huff_build_step"),
        "C527: Should contain huff_build_step function"
    );
    Ok(())
}

/// C528: Huffman encoding -- encode data using Huffman code table
#[test]
fn c528_huffman_encoding() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

typedef struct {
    uint32_t code;
    int bits;
} huff_code_t;

typedef struct {
    huff_code_t table[256];
    uint8_t output[1024];
    int byte_pos;
    int bit_pos;
} huff_encoder_t;

void huff_enc_init(huff_encoder_t *enc) {
    int i;
    for (i = 0; i < 256; i++) {
        enc->table[i].code = 0;
        enc->table[i].bits = 0;
    }
    enc->byte_pos = 0;
    enc->bit_pos = 0;
    for (i = 0; i < 1024; i++) {
        enc->output[i] = 0;
    }
}

void huff_enc_set_code(huff_encoder_t *enc, int symbol, uint32_t code, int bits) {
    if (symbol >= 0 && symbol < 256) {
        enc->table[symbol].code = code;
        enc->table[symbol].bits = bits;
    }
}

void huff_enc_write_bit(huff_encoder_t *enc, int bit) {
    if (enc->byte_pos >= 1024) return;
    if (bit) {
        enc->output[enc->byte_pos] |= (uint8_t)(1 << (7 - enc->bit_pos));
    }
    enc->bit_pos++;
    if (enc->bit_pos >= 8) {
        enc->bit_pos = 0;
        enc->byte_pos++;
    }
}

void huff_enc_encode_symbol(huff_encoder_t *enc, int symbol) {
    if (symbol < 0 || symbol >= 256) return;
    huff_code_t c = enc->table[symbol];
    int i;
    for (i = 0; i < c.bits; i++) {
        int bit = (c.code >> (c.bits - 1 - i)) & 1;
        huff_enc_write_bit(enc, bit);
    }
}

int huff_enc_get_total_bits(const huff_encoder_t *enc) {
    return enc->byte_pos * 8 + enc->bit_pos;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C528: Should produce output");
    assert!(
        rust_code.contains("fn huff_enc_encode_symbol"),
        "C528: Should contain huff_enc_encode_symbol function"
    );
    Ok(())
}

/// C529: LZ77 sliding window -- find matches in a lookback buffer
#[test]
fn c529_lz77_sliding_window() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct {
    int offset;
    int length;
    uint8_t next_char;
} lz77_match_t;

typedef struct {
    uint8_t window[4096];
    int win_size;
    int lookahead;
} lz77_ctx_t;

void lz77_init(lz77_ctx_t *ctx, int win_size, int lookahead) {
    int i;
    ctx->win_size = win_size < 4096 ? win_size : 4096;
    ctx->lookahead = lookahead;
    for (i = 0; i < 4096; i++) {
        ctx->window[i] = 0;
    }
}

lz77_match_t lz77_find_match(const uint8_t *data, int data_len, int pos, int win_start) {
    lz77_match_t best;
    int i;
    best.offset = 0;
    best.length = 0;
    best.next_char = (pos < data_len) ? data[pos] : 0;
    for (i = win_start; i < pos; i++) {
        int match_len = 0;
        while (pos + match_len < data_len && data[i + match_len] == data[pos + match_len]) {
            match_len++;
            if (match_len >= 255) break;
        }
        if (match_len > best.length) {
            best.offset = pos - i;
            best.length = match_len;
            if (pos + match_len < data_len) {
                best.next_char = data[pos + match_len];
            } else {
                best.next_char = 0;
            }
        }
    }
    return best;
}

int lz77_compress_step(const uint8_t *data, int data_len, int pos, lz77_match_t *out) {
    int win_start = pos > 4096 ? pos - 4096 : 0;
    *out = lz77_find_match(data, data_len, pos, win_start);
    if (out->length > 0) {
        return out->length + 1;
    }
    return 1;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C529: Should produce output");
    assert!(
        rust_code.contains("fn lz77_find_match"),
        "C529: Should contain lz77_find_match function"
    );
    Ok(())
}

/// C530: LZW dictionary compression -- build and use a string table
#[test]
fn c530_lzw_dictionary_compression() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

typedef struct {
    uint16_t prefix;
    uint8_t character;
    int valid;
} lzw_entry_t;

typedef struct {
    lzw_entry_t dict[4096];
    int next_code;
    int max_code;
} lzw_dict_t;

void lzw_init(lzw_dict_t *d) {
    int i;
    d->next_code = 256;
    d->max_code = 4096;
    for (i = 0; i < 256; i++) {
        d->dict[i].prefix = 0xFFFF;
        d->dict[i].character = (uint8_t)i;
        d->dict[i].valid = 1;
    }
    for (i = 256; i < 4096; i++) {
        d->dict[i].valid = 0;
    }
}

int lzw_add_entry(lzw_dict_t *d, uint16_t prefix, uint8_t character) {
    if (d->next_code >= d->max_code) return -1;
    int code = d->next_code;
    d->dict[code].prefix = prefix;
    d->dict[code].character = character;
    d->dict[code].valid = 1;
    d->next_code++;
    return code;
}

int lzw_find_entry(const lzw_dict_t *d, uint16_t prefix, uint8_t character) {
    int i;
    for (i = 0; i < d->next_code; i++) {
        if (d->dict[i].valid &&
            d->dict[i].prefix == prefix &&
            d->dict[i].character == character) {
            return i;
        }
    }
    return -1;
}

int lzw_decode_string(const lzw_dict_t *d, int code, uint8_t *out, int max_len) {
    int pos = 0;
    uint8_t stack[256];
    int sp = 0;
    while (code != 0xFFFF && sp < 256) {
        if (code < 0 || code >= d->next_code) break;
        stack[sp] = d->dict[code].character;
        sp++;
        code = d->dict[code].prefix;
    }
    while (sp > 0 && pos < max_len) {
        sp--;
        out[pos] = stack[sp];
        pos++;
    }
    return pos;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C530: Should produce output");
    assert!(
        rust_code.contains("fn lzw_init"),
        "C530: Should contain lzw_init function"
    );
    Ok(())
}

// ============================================================================
// C531-C535: Encoding and Checksums (Base64, CRC32, Adler32, DEFLATE)
// ============================================================================

/// C531: Base64 encoding -- encode binary data to ASCII
#[test]
fn c531_base64_encoding() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned char uint8_t;

static const char b64_table[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

int base64_encode(const uint8_t *input, int in_len, char *output, int max_out) {
    int i = 0;
    int o = 0;
    while (i < in_len && o + 4 <= max_out) {
        uint8_t b0 = input[i];
        uint8_t b1 = (i + 1 < in_len) ? input[i + 1] : 0;
        uint8_t b2 = (i + 2 < in_len) ? input[i + 2] : 0;
        output[o]     = b64_table[(b0 >> 2) & 0x3F];
        output[o + 1] = b64_table[((b0 << 4) | (b1 >> 4)) & 0x3F];
        if (i + 1 < in_len) {
            output[o + 2] = b64_table[((b1 << 2) | (b2 >> 6)) & 0x3F];
        } else {
            output[o + 2] = '=';
        }
        if (i + 2 < in_len) {
            output[o + 3] = b64_table[b2 & 0x3F];
        } else {
            output[o + 3] = '=';
        }
        i += 3;
        o += 4;
    }
    if (o < max_out) output[o] = 0;
    return o;
}

int base64_encoded_len(int in_len) {
    return ((in_len + 2) / 3) * 4;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C531: Should produce output");
    assert!(
        rust_code.contains("fn base64_encode"),
        "C531: Should contain base64_encode function"
    );
    Ok(())
}

/// C532: Base64 decoding -- decode ASCII back to binary
#[test]
fn c532_base64_decoding() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned char uint8_t;

int b64_char_to_val(char c) {
    if (c >= 'A' && c <= 'Z') return c - 'A';
    if (c >= 'a' && c <= 'z') return c - 'a' + 26;
    if (c >= '0' && c <= '9') return c - '0' + 52;
    if (c == '+') return 62;
    if (c == '/') return 63;
    return -1;
}

int base64_decode(const char *input, int in_len, uint8_t *output, int max_out) {
    int i = 0;
    int o = 0;
    while (i + 3 < in_len && o < max_out) {
        int v0 = b64_char_to_val(input[i]);
        int v1 = b64_char_to_val(input[i + 1]);
        int v2 = b64_char_to_val(input[i + 2]);
        int v3 = b64_char_to_val(input[i + 3]);
        if (v0 < 0 || v1 < 0) break;
        output[o] = (uint8_t)((v0 << 2) | (v1 >> 4));
        o++;
        if (v2 >= 0 && o < max_out) {
            output[o] = (uint8_t)((v1 << 4) | (v2 >> 2));
            o++;
        }
        if (v3 >= 0 && o < max_out) {
            output[o] = (uint8_t)((v2 << 6) | v3);
            o++;
        }
        i += 4;
    }
    return o;
}

int base64_roundtrip_check(void) {
    uint8_t data[4];
    data[0] = 0xDE; data[1] = 0xAD; data[2] = 0xBE; data[3] = 0xEF;
    return 0;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C532: Should produce output");
    assert!(
        rust_code.contains("fn base64_decode"),
        "C532: Should contain base64_decode function"
    );
    Ok(())
}

/// C533: CRC32 computation -- compute cyclic redundancy check
#[test]
fn c533_crc32_computation() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

static uint32_t crc32_table[256];

void crc32_init_table(void) {
    uint32_t poly = 0xEDB88320;
    int i;
    for (i = 0; i < 256; i++) {
        uint32_t crc = (uint32_t)i;
        int j;
        for (j = 0; j < 8; j++) {
            if (crc & 1) {
                crc = (crc >> 1) ^ poly;
            } else {
                crc = crc >> 1;
            }
        }
        crc32_table[i] = crc;
    }
}

uint32_t crc32_update(uint32_t crc, const uint8_t *data, int len) {
    int i;
    crc = crc ^ 0xFFFFFFFF;
    for (i = 0; i < len; i++) {
        uint32_t idx = (crc ^ data[i]) & 0xFF;
        crc = (crc >> 8) ^ crc32_table[idx];
    }
    return crc ^ 0xFFFFFFFF;
}

uint32_t crc32_compute(const uint8_t *data, int len) {
    return crc32_update(0, data, len);
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C533: Should produce output");
    assert!(
        rust_code.contains("fn crc32_update"),
        "C533: Should contain crc32_update function"
    );
    Ok(())
}

/// C534: Adler32 checksum -- fast rolling checksum used in zlib
#[test]
fn c534_adler32_checksum() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

uint32_t adler32_compute(const uint8_t *data, int len) {
    uint32_t a = 1;
    uint32_t b = 0;
    int i;
    for (i = 0; i < len; i++) {
        a = (a + data[i]) % 65521;
        b = (b + a) % 65521;
    }
    return (b << 16) | a;
}

uint32_t adler32_combine(uint32_t adler1, uint32_t adler2, int len2) {
    uint32_t a1 = adler1 & 0xFFFF;
    uint32_t b1 = (adler1 >> 16) & 0xFFFF;
    uint32_t a2 = adler2 & 0xFFFF;
    uint32_t b2 = (adler2 >> 16) & 0xFFFF;
    uint32_t a = (a1 + a2 - 1) % 65521;
    uint32_t b = (b1 + b2 + a1 * (uint32_t)len2 - (uint32_t)len2) % 65521;
    return (b << 16) | a;
}

int adler32_verify(const uint8_t *data, int len, uint32_t expected) {
    uint32_t actual = adler32_compute(data, len);
    return actual == expected ? 0 : -1;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C534: Should produce output");
    assert!(
        rust_code.contains("fn adler32_compute"),
        "C534: Should contain adler32_compute function"
    );
    Ok(())
}

/// C535: DEFLATE static block -- emit fixed Huffman-coded literals
#[test]
fn c535_deflate_static_block() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

typedef struct {
    uint8_t buf[1024];
    int byte_pos;
    int bit_pos;
} bitwriter_t;

void bw_init(bitwriter_t *bw) {
    int i;
    bw->byte_pos = 0;
    bw->bit_pos = 0;
    for (i = 0; i < 1024; i++) {
        bw->buf[i] = 0;
    }
}

void bw_write_bits(bitwriter_t *bw, uint16_t value, int nbits) {
    int i;
    for (i = 0; i < nbits; i++) {
        if (bw->byte_pos >= 1024) return;
        if (value & (1 << i)) {
            bw->buf[bw->byte_pos] |= (uint8_t)(1 << bw->bit_pos);
        }
        bw->bit_pos++;
        if (bw->bit_pos >= 8) {
            bw->bit_pos = 0;
            bw->byte_pos++;
        }
    }
}

uint16_t deflate_fixed_code(int literal) {
    if (literal <= 143) {
        return (uint16_t)(0x30 + literal);
    } else if (literal <= 255) {
        return (uint16_t)(0x190 + (literal - 144));
    } else if (literal == 256) {
        return 0;
    }
    return 0xFFFF;
}

int deflate_fixed_bits(int literal) {
    if (literal <= 143) return 8;
    if (literal <= 255) return 9;
    if (literal == 256) return 7;
    return 0;
}

void deflate_emit_literal(bitwriter_t *bw, int literal) {
    uint16_t code = deflate_fixed_code(literal);
    int bits = deflate_fixed_bits(literal);
    bw_write_bits(bw, code, bits);
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C535: Should produce output");
    assert!(
        rust_code.contains("fn deflate_emit_literal"),
        "C535: Should contain deflate_emit_literal function"
    );
    Ok(())
}

// ============================================================================
// C536-C540: Advanced Coding (Arithmetic, Delta, BWT, MTF, Bitstream)
// ============================================================================

/// C536: Arithmetic coding -- encode symbols using interval subdivision
#[test]
fn c536_arithmetic_coding() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t low;
    uint32_t high;
    uint32_t range;
    int pending_bits;
    int total_symbols;
} arith_enc_t;

void arith_init(arith_enc_t *enc) {
    enc->low = 0;
    enc->high = 0xFFFFFFFF;
    enc->range = 0xFFFFFFFF;
    enc->pending_bits = 0;
    enc->total_symbols = 0;
}

void arith_encode_symbol(arith_enc_t *enc, int cum_freq_low, int cum_freq_high, int total_freq) {
    uint32_t range = enc->high - enc->low + 1;
    enc->high = enc->low + (range * (uint32_t)cum_freq_high) / (uint32_t)total_freq - 1;
    enc->low = enc->low + (range * (uint32_t)cum_freq_low) / (uint32_t)total_freq;
    enc->total_symbols++;
}

void arith_normalize(arith_enc_t *enc) {
    while (1) {
        if ((enc->high & 0x80000000) == (enc->low & 0x80000000)) {
            enc->pending_bits++;
            enc->low = (enc->low << 1) & 0xFFFFFFFF;
            enc->high = ((enc->high << 1) | 1) & 0xFFFFFFFF;
        } else if ((enc->low & 0x40000000) && !(enc->high & 0x40000000)) {
            enc->low = (enc->low << 1) & 0x7FFFFFFF;
            enc->high = ((enc->high << 1) | 0x80000001);
            enc->pending_bits++;
        } else {
            break;
        }
    }
}

uint32_t arith_get_low(const arith_enc_t *enc) {
    return enc->low;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C536: Should produce output");
    assert!(
        rust_code.contains("fn arith_encode_symbol"),
        "C536: Should contain arith_encode_symbol function"
    );
    Ok(())
}

/// C537: Delta encoding -- store differences between consecutive values
#[test]
fn c537_delta_encoding() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int data[256];
    int len;
} delta_buf_t;

void delta_encode(const int *input, int len, int *output) {
    int i;
    if (len <= 0) return;
    output[0] = input[0];
    for (i = 1; i < len; i++) {
        output[i] = input[i] - input[i - 1];
    }
}

void delta_decode(const int *input, int len, int *output) {
    int i;
    if (len <= 0) return;
    output[0] = input[0];
    for (i = 1; i < len; i++) {
        output[i] = output[i - 1] + input[i];
    }
}

int delta_roundtrip(const int *data, int len) {
    int encoded[256];
    int decoded[256];
    int i;
    if (len > 256) return -1;
    delta_encode(data, len, encoded);
    delta_decode(encoded, len, decoded);
    for (i = 0; i < len; i++) {
        if (decoded[i] != data[i]) return -2;
    }
    return 0;
}

int delta_compression_ratio(const int *deltas, int len) {
    int zero_count = 0;
    int i;
    for (i = 0; i < len; i++) {
        if (deltas[i] == 0) zero_count++;
    }
    if (len == 0) return 0;
    return (zero_count * 100) / len;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C537: Should produce output");
    assert!(
        rust_code.contains("fn delta_encode"),
        "C537: Should contain delta_encode function"
    );
    assert!(
        rust_code.contains("fn delta_decode"),
        "C537: Should contain delta_decode function"
    );
    Ok(())
}

/// C538: Burrows-Wheeler transform -- permute input for better compression
#[test]
fn c538_burrows_wheeler_transform() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct {
    int indices[256];
    uint8_t output[256];
    int original_idx;
    int len;
} bwt_ctx_t;

int bwt_compare(const uint8_t *data, int len, int a, int b) {
    int i;
    for (i = 0; i < len; i++) {
        int ca = data[(a + i) % len];
        int cb = data[(b + i) % len];
        if (ca < cb) return -1;
        if (ca > cb) return 1;
    }
    return 0;
}

void bwt_sort_indices(const uint8_t *data, int len, int *indices) {
    int i;
    int j;
    for (i = 0; i < len; i++) {
        indices[i] = i;
    }
    for (i = 1; i < len; i++) {
        int key = indices[i];
        j = i - 1;
        while (j >= 0 && bwt_compare(data, len, indices[j], key) > 0) {
            indices[j + 1] = indices[j];
            j--;
        }
        indices[j + 1] = key;
    }
}

void bwt_transform(bwt_ctx_t *ctx, const uint8_t *data, int len) {
    int i;
    if (len > 256) len = 256;
    ctx->len = len;
    bwt_sort_indices(data, len, ctx->indices);
    ctx->original_idx = -1;
    for (i = 0; i < len; i++) {
        int rot = ctx->indices[i];
        ctx->output[i] = data[(rot + len - 1) % len];
        if (rot == 0) {
            ctx->original_idx = i;
        }
    }
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C538: Should produce output");
    assert!(
        rust_code.contains("fn bwt_transform"),
        "C538: Should contain bwt_transform function"
    );
    Ok(())
}

/// C539: Move-to-front encoding -- adaptive byte reordering
#[test]
fn c539_move_to_front_encoding() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct {
    uint8_t list[256];
} mtf_ctx_t;

void mtf_init(mtf_ctx_t *ctx) {
    int i;
    for (i = 0; i < 256; i++) {
        ctx->list[i] = (uint8_t)i;
    }
}

int mtf_find(const mtf_ctx_t *ctx, uint8_t value) {
    int i;
    for (i = 0; i < 256; i++) {
        if (ctx->list[i] == value) return i;
    }
    return -1;
}

void mtf_move_to_front(mtf_ctx_t *ctx, int pos) {
    uint8_t val = ctx->list[pos];
    int i;
    for (i = pos; i > 0; i--) {
        ctx->list[i] = ctx->list[i - 1];
    }
    ctx->list[0] = val;
}

void mtf_encode(mtf_ctx_t *ctx, const uint8_t *input, int len, uint8_t *output) {
    int i;
    for (i = 0; i < len; i++) {
        int pos = mtf_find(ctx, input[i]);
        output[i] = (uint8_t)pos;
        mtf_move_to_front(ctx, pos);
    }
}

void mtf_decode(mtf_ctx_t *ctx, const uint8_t *input, int len, uint8_t *output) {
    int i;
    for (i = 0; i < len; i++) {
        int pos = input[i];
        output[i] = ctx->list[pos];
        mtf_move_to_front(ctx, pos);
    }
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C539: Should produce output");
    assert!(
        rust_code.contains("fn mtf_encode"),
        "C539: Should contain mtf_encode function"
    );
    assert!(
        rust_code.contains("fn mtf_decode"),
        "C539: Should contain mtf_decode function"
    );
    Ok(())
}

/// C540: Bit-level I/O (bitstream writer) -- write individual bits to a byte buffer
#[test]
fn c540_bitstream_writer() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

typedef struct {
    uint8_t buffer[512];
    int byte_pos;
    int bit_pos;
    int capacity;
} bit_writer_t;

void bw_create(bit_writer_t *w, int capacity) {
    int i;
    w->byte_pos = 0;
    w->bit_pos = 0;
    w->capacity = capacity < 512 ? capacity : 512;
    for (i = 0; i < 512; i++) {
        w->buffer[i] = 0;
    }
}

int bw_put_bit(bit_writer_t *w, int bit) {
    if (w->byte_pos >= w->capacity) return -1;
    if (bit) {
        w->buffer[w->byte_pos] |= (uint8_t)(1 << (7 - w->bit_pos));
    }
    w->bit_pos++;
    if (w->bit_pos >= 8) {
        w->bit_pos = 0;
        w->byte_pos++;
    }
    return 0;
}

int bw_put_bits(bit_writer_t *w, uint32_t value, int nbits) {
    int i;
    for (i = nbits - 1; i >= 0; i--) {
        int bit = (value >> i) & 1;
        if (bw_put_bit(w, bit) < 0) return -1;
    }
    return 0;
}

int bw_total_bits(const bit_writer_t *w) {
    return w->byte_pos * 8 + w->bit_pos;
}

int bw_flush(bit_writer_t *w) {
    if (w->bit_pos > 0) {
        w->byte_pos++;
        w->bit_pos = 0;
    }
    return w->byte_pos;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C540: Should produce output");
    assert!(
        rust_code.contains("fn bw_put_bit"),
        "C540: Should contain bw_put_bit function"
    );
    Ok(())
}

// ============================================================================
// C541-C545: Variable-Length Codes (Bitstream Reader, Varint, Golomb, Elias, Fibonacci)
// ============================================================================

/// C541: Bit-level I/O (bitstream reader) -- read individual bits from a byte buffer
#[test]
fn c541_bitstream_reader() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

typedef struct {
    const uint8_t *data;
    int len;
    int byte_pos;
    int bit_pos;
} bit_reader_t;

void br_init(bit_reader_t *r, const uint8_t *data, int len) {
    r->data = data;
    r->len = len;
    r->byte_pos = 0;
    r->bit_pos = 0;
}

int br_get_bit(bit_reader_t *r) {
    if (r->byte_pos >= r->len) return -1;
    int bit = (r->data[r->byte_pos] >> (7 - r->bit_pos)) & 1;
    r->bit_pos++;
    if (r->bit_pos >= 8) {
        r->bit_pos = 0;
        r->byte_pos++;
    }
    return bit;
}

int br_get_bits(bit_reader_t *r, int nbits, uint32_t *out) {
    uint32_t value = 0;
    int i;
    for (i = 0; i < nbits; i++) {
        int bit = br_get_bit(r);
        if (bit < 0) return -1;
        value = (value << 1) | (uint32_t)bit;
    }
    *out = value;
    return 0;
}

int br_remaining_bits(const bit_reader_t *r) {
    return (r->len - r->byte_pos) * 8 - r->bit_pos;
}

int br_is_aligned(const bit_reader_t *r) {
    return r->bit_pos == 0;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C541: Should produce output");
    assert!(
        rust_code.contains("fn br_get_bit"),
        "C541: Should contain br_get_bit function"
    );
    Ok(())
}

/// C542: Variable-length integer encoding (varint) -- protobuf-style encoding
#[test]
fn c542_varint_encoding() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

int varint_encode(uint32_t value, uint8_t *output, int max_len) {
    int pos = 0;
    while (value > 0x7F && pos < max_len) {
        output[pos] = (uint8_t)((value & 0x7F) | 0x80);
        value >>= 7;
        pos++;
    }
    if (pos < max_len) {
        output[pos] = (uint8_t)(value & 0x7F);
        pos++;
    }
    return pos;
}

int varint_decode(const uint8_t *input, int max_len, uint32_t *value) {
    uint32_t result = 0;
    int shift = 0;
    int pos = 0;
    while (pos < max_len) {
        uint32_t byte_val = input[pos];
        result |= (byte_val & 0x7F) << shift;
        pos++;
        if (!(byte_val & 0x80)) break;
        shift += 7;
        if (shift >= 35) return -1;
    }
    *value = result;
    return pos;
}

int varint_encoded_size(uint32_t value) {
    int size = 1;
    while (value > 0x7F) {
        value >>= 7;
        size++;
    }
    return size;
}

int varint_roundtrip(uint32_t input) {
    uint8_t buf[8];
    uint32_t output = 0;
    int enc_len = varint_encode(input, buf, 8);
    if (enc_len <= 0) return -1;
    int dec_len = varint_decode(buf, enc_len, &output);
    if (dec_len != enc_len) return -2;
    if (output != input) return -3;
    return 0;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C542: Should produce output");
    assert!(
        rust_code.contains("fn varint_encode"),
        "C542: Should contain varint_encode function"
    );
    assert!(
        rust_code.contains("fn varint_decode"),
        "C542: Should contain varint_decode function"
    );
    Ok(())
}

/// C543: Golomb-Rice coding -- encode integers with tunable parameter
#[test]
fn c543_golomb_rice_coding() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    int quotient;
    uint32_t remainder;
    int m_bits;
} golomb_code_t;

golomb_code_t golomb_rice_encode(uint32_t value, int k) {
    golomb_code_t code;
    uint32_t m = (uint32_t)1 << k;
    code.quotient = (int)(value / m);
    code.remainder = value % m;
    code.m_bits = k;
    return code;
}

uint32_t golomb_rice_decode(int quotient, uint32_t remainder, int k) {
    uint32_t m = (uint32_t)1 << k;
    return (uint32_t)quotient * m + remainder;
}

int golomb_rice_total_bits(golomb_code_t code) {
    return code.quotient + 1 + code.m_bits;
}

int golomb_rice_roundtrip(uint32_t value, int k) {
    golomb_code_t code = golomb_rice_encode(value, k);
    uint32_t decoded = golomb_rice_decode(code.quotient, code.remainder, k);
    return (decoded == value) ? 0 : -1;
}

int golomb_rice_optimal_k(const uint32_t *data, int len) {
    uint32_t sum = 0;
    int i;
    for (i = 0; i < len; i++) {
        sum += data[i];
    }
    if (len == 0) return 0;
    uint32_t mean = sum / (uint32_t)len;
    int k = 0;
    while ((uint32_t)(1 << k) < mean && k < 16) {
        k++;
    }
    return k > 0 ? k - 1 : 0;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C543: Should produce output");
    assert!(
        rust_code.contains("fn golomb_rice_encode"),
        "C543: Should contain golomb_rice_encode function"
    );
    Ok(())
}

/// C544: Elias gamma coding -- encode positive integers using prefix-free code
#[test]
fn c544_elias_gamma_coding() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;

int elias_floor_log2(uint32_t n) {
    int log = 0;
    while (n > 1) {
        n >>= 1;
        log++;
    }
    return log;
}

int elias_gamma_bits(uint32_t value) {
    if (value == 0) return 1;
    int n = elias_floor_log2(value);
    return 2 * n + 1;
}

int elias_gamma_encode_to_array(uint32_t value, int *bits, int max_bits) {
    if (value == 0) {
        if (max_bits < 1) return -1;
        bits[0] = 0;
        return 1;
    }
    int n = elias_floor_log2(value);
    int total = 2 * n + 1;
    if (total > max_bits) return -1;
    int i;
    for (i = 0; i < n; i++) {
        bits[i] = 0;
    }
    for (i = 0; i <= n; i++) {
        bits[n + i] = (value >> (n - i)) & 1;
    }
    return total;
}

uint32_t elias_gamma_decode_from_array(const int *bits, int len, int *consumed) {
    int zeros = 0;
    int i;
    for (i = 0; i < len; i++) {
        if (bits[i] == 0) {
            zeros++;
        } else {
            break;
        }
    }
    uint32_t value = 0;
    for (i = 0; i <= zeros; i++) {
        if (zeros + i >= len) break;
        value = (value << 1) | (uint32_t)bits[zeros + i];
    }
    *consumed = 2 * zeros + 1;
    return value;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C544: Should produce output");
    assert!(
        rust_code.contains("fn elias_gamma_bits"),
        "C544: Should contain elias_gamma_bits function"
    );
    Ok(())
}

/// C545: Fibonacci coding -- represent integers using Fibonacci number sums
#[test]
fn c545_fibonacci_coding() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;

void fib_generate(uint32_t *fibs, int max_count) {
    if (max_count < 2) return;
    fibs[0] = 1;
    fibs[1] = 2;
    int i;
    for (i = 2; i < max_count; i++) {
        fibs[i] = fibs[i - 1] + fibs[i - 2];
        if (fibs[i] > 0x7FFFFFFF) {
            fibs[i] = 0;
            break;
        }
    }
}

int fib_encode(uint32_t value, int *bits, int max_bits) {
    uint32_t fibs[47];
    int bit_count = 0;
    int i;
    fib_generate(fibs, 47);
    int highest = 0;
    for (i = 0; i < 47; i++) {
        if (fibs[i] == 0) break;
        if (fibs[i] <= value) highest = i;
    }
    if (highest + 2 > max_bits) return -1;
    for (i = 0; i <= highest; i++) {
        bits[i] = 0;
    }
    for (i = highest; i >= 0; i--) {
        if (fibs[i] <= value) {
            bits[i] = 1;
            value -= fibs[i];
        }
    }
    bit_count = highest + 1;
    bits[bit_count] = 1;
    bit_count++;
    return bit_count;
}

int fib_code_length(uint32_t value) {
    int bits[64];
    return fib_encode(value, bits, 64);
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C545: Should produce output");
    assert!(
        rust_code.contains("fn fib_encode"),
        "C545: Should contain fib_encode function"
    );
    Ok(())
}

// ============================================================================
// C546-C550: Specialized (BPE, PPM, Range Encoder, Shannon Entropy, JPEG Quant)
// ============================================================================

/// C546: Byte-pair encoding -- iteratively merge most frequent byte pairs
#[test]
fn c546_byte_pair_encoding() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct {
    uint8_t first;
    uint8_t second;
    int count;
} bpe_pair_t;

typedef struct {
    bpe_pair_t pairs[256];
    int num_pairs;
    uint8_t merge_table[256];
    int next_symbol;
} bpe_ctx_t;

void bpe_init(bpe_ctx_t *ctx) {
    int i;
    ctx->num_pairs = 0;
    ctx->next_symbol = 128;
    for (i = 0; i < 256; i++) {
        ctx->merge_table[i] = (uint8_t)i;
    }
}

void bpe_count_pairs(const uint8_t *data, int len, bpe_pair_t *pairs, int *num_pairs) {
    int i;
    *num_pairs = 0;
    for (i = 0; i + 1 < len; i++) {
        int j;
        int found = 0;
        for (j = 0; j < *num_pairs; j++) {
            if (pairs[j].first == data[i] && pairs[j].second == data[i + 1]) {
                pairs[j].count++;
                found = 1;
                break;
            }
        }
        if (!found && *num_pairs < 256) {
            pairs[*num_pairs].first = data[i];
            pairs[*num_pairs].second = data[i + 1];
            pairs[*num_pairs].count = 1;
            (*num_pairs)++;
        }
    }
}

int bpe_find_best_pair(const bpe_pair_t *pairs, int num_pairs) {
    int best = -1;
    int best_count = 0;
    int i;
    for (i = 0; i < num_pairs; i++) {
        if (pairs[i].count > best_count) {
            best_count = pairs[i].count;
            best = i;
        }
    }
    return best;
}

int bpe_merge_pair(uint8_t *data, int len, uint8_t first, uint8_t second, uint8_t replacement) {
    int new_len = 0;
    int i = 0;
    while (i < len) {
        if (i + 1 < len && data[i] == first && data[i + 1] == second) {
            data[new_len] = replacement;
            new_len++;
            i += 2;
        } else {
            data[new_len] = data[i];
            new_len++;
            i++;
        }
    }
    return new_len;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C546: Should produce output");
    assert!(
        rust_code.contains("fn bpe_merge_pair"),
        "C546: Should contain bpe_merge_pair function"
    );
    Ok(())
}

/// C547: Prediction by partial matching (PPM context) -- context modeling for compression
#[test]
fn c547_ppm_context() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct {
    uint8_t context[4];
    int ctx_len;
    int counts[256];
    int total;
} ppm_ctx_t;

void ppm_init(ppm_ctx_t *ctx) {
    int i;
    ctx->ctx_len = 0;
    ctx->total = 0;
    for (i = 0; i < 4; i++) {
        ctx->context[i] = 0;
    }
    for (i = 0; i < 256; i++) {
        ctx->counts[i] = 0;
    }
}

void ppm_update(ppm_ctx_t *ctx, uint8_t symbol) {
    ctx->counts[symbol]++;
    ctx->total++;
    int i;
    for (i = 0; i < 3; i++) {
        ctx->context[i] = ctx->context[i + 1];
    }
    ctx->context[3] = symbol;
    if (ctx->ctx_len < 4) ctx->ctx_len++;
}

int ppm_predict(const ppm_ctx_t *ctx) {
    int best = -1;
    int best_count = 0;
    int i;
    for (i = 0; i < 256; i++) {
        if (ctx->counts[i] > best_count) {
            best_count = ctx->counts[i];
            best = i;
        }
    }
    return best;
}

int ppm_probability(const ppm_ctx_t *ctx, uint8_t symbol) {
    if (ctx->total == 0) return 0;
    return (ctx->counts[symbol] * 1000) / ctx->total;
}

int ppm_escape_probability(const ppm_ctx_t *ctx) {
    int unique = 0;
    int i;
    for (i = 0; i < 256; i++) {
        if (ctx->counts[i] > 0) unique++;
    }
    if (ctx->total == 0) return 1000;
    return (unique * 1000) / (ctx->total + unique);
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C547: Should produce output");
    assert!(
        rust_code.contains("fn ppm_predict"),
        "C547: Should contain ppm_predict function"
    );
    Ok(())
}

/// C548: Range encoder -- encode symbols using range subdivision
#[test]
fn c548_range_encoder() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint32_t low;
    uint32_t range;
    uint8_t output[1024];
    int out_pos;
    int symbols_encoded;
} range_enc_t;

void range_enc_init(range_enc_t *enc) {
    int i;
    enc->low = 0;
    enc->range = 0xFFFFFFFF;
    enc->out_pos = 0;
    enc->symbols_encoded = 0;
    for (i = 0; i < 1024; i++) {
        enc->output[i] = 0;
    }
}

void range_enc_emit_byte(range_enc_t *enc) {
    if (enc->out_pos < 1024) {
        enc->output[enc->out_pos] = (uint8_t)(enc->low >> 24);
        enc->out_pos++;
    }
    enc->low <<= 8;
    enc->range <<= 8;
}

void range_enc_encode(range_enc_t *enc, uint32_t cum_freq, uint32_t freq, uint32_t total) {
    enc->range = enc->range / total;
    enc->low += cum_freq * enc->range;
    enc->range = freq * enc->range;
    while (enc->range < 0x01000000) {
        range_enc_emit_byte(enc);
    }
    enc->symbols_encoded++;
}

void range_enc_flush(range_enc_t *enc) {
    int i;
    for (i = 0; i < 4; i++) {
        range_enc_emit_byte(enc);
    }
}

int range_enc_output_size(const range_enc_t *enc) {
    return enc->out_pos;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C548: Should produce output");
    assert!(
        rust_code.contains("fn range_enc_encode"),
        "C548: Should contain range_enc_encode function"
    );
    Ok(())
}

/// C549: Entropy calculation (Shannon entropy) -- measure information content
#[test]
fn c549_shannon_entropy() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct {
    int counts[256];
    int total;
} entropy_ctx_t;

void entropy_init(entropy_ctx_t *ctx) {
    int i;
    ctx->total = 0;
    for (i = 0; i < 256; i++) {
        ctx->counts[i] = 0;
    }
}

void entropy_add_data(entropy_ctx_t *ctx, const uint8_t *data, int len) {
    int i;
    for (i = 0; i < len; i++) {
        ctx->counts[data[i]]++;
        ctx->total++;
    }
}

int entropy_unique_symbols(const entropy_ctx_t *ctx) {
    int count = 0;
    int i;
    for (i = 0; i < 256; i++) {
        if (ctx->counts[i] > 0) count++;
    }
    return count;
}

int entropy_most_frequent(const entropy_ctx_t *ctx) {
    int best = 0;
    int best_count = 0;
    int i;
    for (i = 0; i < 256; i++) {
        if (ctx->counts[i] > best_count) {
            best_count = ctx->counts[i];
            best = i;
        }
    }
    return best;
}

int entropy_compute_scaled(const entropy_ctx_t *ctx) {
    int result = 0;
    int i;
    if (ctx->total == 0) return 0;
    for (i = 0; i < 256; i++) {
        if (ctx->counts[i] == 0) continue;
        int freq_pct = (ctx->counts[i] * 1000) / ctx->total;
        if (freq_pct <= 0) continue;
        int log_approx = 0;
        int tmp = freq_pct;
        while (tmp > 1) {
            tmp >>= 1;
            log_approx++;
        }
        result += freq_pct * log_approx;
    }
    return result;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C549: Should produce output");
    assert!(
        rust_code.contains("fn entropy_compute_scaled"),
        "C549: Should contain entropy_compute_scaled function"
    );
    Ok(())
}

/// C550: JPEG quantization matrix -- apply DCT coefficient quantization
#[test]
fn c550_jpeg_quantization_matrix() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int table[64];
    int quality;
} jpeg_qtable_t;

static const int jpeg_std_luma_qt[64] = {
    16, 11, 10, 16, 24, 40, 51, 61,
    12, 12, 14, 19, 26, 58, 60, 55,
    14, 13, 16, 24, 40, 57, 69, 56,
    14, 17, 22, 29, 51, 87, 80, 62,
    18, 22, 37, 56, 68,109,103, 77,
    24, 35, 55, 64, 81,104,113, 92,
    49, 64, 78, 87,103,121,120,101,
    72, 92, 95, 98,112,100,103, 99
};

void jpeg_qt_init(jpeg_qtable_t *qt, int quality) {
    int scale;
    int i;
    if (quality < 1) quality = 1;
    if (quality > 100) quality = 100;
    if (quality < 50) {
        scale = 5000 / quality;
    } else {
        scale = 200 - quality * 2;
    }
    qt->quality = quality;
    for (i = 0; i < 64; i++) {
        int val = (jpeg_std_luma_qt[i] * scale + 50) / 100;
        if (val < 1) val = 1;
        if (val > 255) val = 255;
        qt->table[i] = val;
    }
}

int jpeg_qt_quantize(const jpeg_qtable_t *qt, int coeff, int index) {
    if (index < 0 || index >= 64) return 0;
    int q = qt->table[index];
    if (q == 0) return 0;
    return (coeff + (q / 2)) / q;
}

int jpeg_qt_dequantize(const jpeg_qtable_t *qt, int qcoeff, int index) {
    if (index < 0 || index >= 64) return 0;
    return qcoeff * qt->table[index];
}

int jpeg_qt_count_zeros(const int *quantized, int len) {
    int count = 0;
    int i;
    for (i = 0; i < len && i < 64; i++) {
        if (quantized[i] == 0) count++;
    }
    return count;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C550: Should produce output");
    assert!(
        rust_code.contains("fn jpeg_qt_init"),
        "C550: Should contain jpeg_qt_init function"
    );
    Ok(())
}
