//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1901-C1925: Encoding and Decoding patterns -- the kind of C code found
//! in data interchange libraries, protocol buffers, compression utilities,
//! and binary format handlers.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world encoding/decoding patterns commonly
//! found in base64 libraries, URL encoders, RLE compressors, varint codecs,
//! and bit-packing utilities -- all expressed as valid C99 with inline type
//! definitions (no #include).
//!
//! Organization:
//! - C1901-C1905: Base64 encoding (encode table, decode table, encode, decode, padding)
//! - C1906-C1910: URL encoding (percent encode, decode, reserved chars, query string, path)
//! - C1911-C1915: Run-length encoding (compress, decompress, count runs, mixed data, worst case)
//! - C1916-C1920: Variable-length integer (varint encode, decode, zigzag encode, zigzag decode, multi-byte)
//! - C1921-C1925: Bit packing (pack bits, unpack bits, field extraction, field insertion, alignment)
//!
//! ## Results
//! - 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C1901-C1905: Base64 Encoding
// ============================================================================

/// C1901: Base64 encode table -- builds the 64-character encoding alphabet
#[test]
fn c1901_base64_encode_table() {
    let c_code = r##"
typedef unsigned char uint8_t;

static const char enc_b64_table[64] = {
    'A','B','C','D','E','F','G','H','I','J','K','L','M',
    'N','O','P','Q','R','S','T','U','V','W','X','Y','Z',
    'a','b','c','d','e','f','g','h','i','j','k','l','m',
    'n','o','p','q','r','s','t','u','v','w','x','y','z',
    '0','1','2','3','4','5','6','7','8','9','+','/'
};

char enc_b64_char(uint8_t index) {
    if (index >= 64) return '=';
    return enc_b64_table[index];
}

void enc_b64_triplet(const uint8_t *in, char *out) {
    out[0] = enc_b64_char((in[0] >> 2) & 0x3F);
    out[1] = enc_b64_char(((in[0] & 0x03) << 4) | ((in[1] >> 4) & 0x0F));
    out[2] = enc_b64_char(((in[1] & 0x0F) << 2) | ((in[2] >> 6) & 0x03));
    out[3] = enc_b64_char(in[2] & 0x3F);
}

int enc_b64_encoded_len(int input_len) {
    return ((input_len + 2) / 3) * 4;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1901: Base64 encode table should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1901: Output should not be empty");
    assert!(code.contains("fn enc_b64_char"), "C1901: Should contain enc_b64_char function");
    assert!(code.contains("fn enc_b64_triplet"), "C1901: Should contain enc_b64_triplet function");
}

/// C1902: Base64 decode table -- reverse lookup from ASCII to 6-bit values
#[test]
fn c1902_base64_decode_table() {
    let c_code = r##"
typedef unsigned char uint8_t;

int enc_b64_decode_char(char c) {
    if (c >= 'A' && c <= 'Z') return c - 'A';
    if (c >= 'a' && c <= 'z') return c - 'a' + 26;
    if (c >= '0' && c <= '9') return c - '0' + 52;
    if (c == '+') return 62;
    if (c == '/') return 63;
    return -1;
}

int enc_b64_is_valid_char(char c) {
    return enc_b64_decode_char(c) >= 0 || c == '=';
}

int enc_b64_validate(const char *input, int len) {
    int i;
    if (len % 4 != 0) return 0;
    for (i = 0; i < len; i++) {
        if (!enc_b64_is_valid_char(input[i])) return 0;
    }
    return 1;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1902: Base64 decode table should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1902: Output should not be empty");
    assert!(code.contains("fn enc_b64_decode_char"), "C1902: Should contain enc_b64_decode_char function");
    assert!(code.contains("fn enc_b64_validate"), "C1902: Should contain enc_b64_validate function");
}

/// C1903: Base64 encode -- encodes arbitrary bytes into base64 string
#[test]
fn c1903_base64_encode() {
    let c_code = r##"
typedef unsigned char uint8_t;
typedef unsigned long size_t;

static const char enc_b64_alpha[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

size_t enc_b64_encode(const uint8_t *src, size_t src_len, char *dst, size_t dst_cap) {
    size_t i = 0;
    size_t j = 0;
    uint8_t a, b, c;
    while (i < src_len && j + 4 <= dst_cap) {
        a = src[i++];
        b = (i < src_len) ? src[i++] : 0;
        c = (i < src_len) ? src[i++] : 0;
        dst[j++] = enc_b64_alpha[(a >> 2) & 0x3F];
        dst[j++] = enc_b64_alpha[((a & 0x03) << 4) | ((b >> 4) & 0x0F)];
        dst[j++] = enc_b64_alpha[((b & 0x0F) << 2) | ((c >> 6) & 0x03)];
        dst[j++] = enc_b64_alpha[c & 0x3F];
    }
    if (j < dst_cap) dst[j] = '\0';
    return j;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1903: Base64 encode should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1903: Output should not be empty");
    assert!(code.contains("fn enc_b64_encode"), "C1903: Should contain enc_b64_encode function");
}

/// C1904: Base64 decode -- decodes base64 string back to bytes
#[test]
fn c1904_base64_decode() {
    let c_code = r##"
typedef unsigned char uint8_t;
typedef unsigned long size_t;

static int enc_b64_val(char c) {
    if (c >= 'A' && c <= 'Z') return c - 'A';
    if (c >= 'a' && c <= 'z') return c - 'a' + 26;
    if (c >= '0' && c <= '9') return c - '0' + 52;
    if (c == '+') return 62;
    if (c == '/') return 63;
    return -1;
}

size_t enc_b64_decode(const char *src, size_t src_len, uint8_t *dst, size_t dst_cap) {
    size_t i = 0;
    size_t j = 0;
    int v0, v1, v2, v3;
    while (i + 4 <= src_len && j < dst_cap) {
        v0 = enc_b64_val(src[i++]);
        v1 = enc_b64_val(src[i++]);
        v2 = enc_b64_val(src[i++]);
        v3 = enc_b64_val(src[i++]);
        if (v0 < 0 || v1 < 0) break;
        dst[j++] = (uint8_t)((v0 << 2) | (v1 >> 4));
        if (v2 >= 0 && j < dst_cap)
            dst[j++] = (uint8_t)(((v1 & 0x0F) << 4) | (v2 >> 2));
        if (v3 >= 0 && j < dst_cap)
            dst[j++] = (uint8_t)(((v2 & 0x03) << 6) | v3);
    }
    return j;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1904: Base64 decode should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1904: Output should not be empty");
    assert!(code.contains("fn enc_b64_decode"), "C1904: Should contain enc_b64_decode function");
}

/// C1905: Base64 padding handling -- correctly handles 1-byte and 2-byte padding cases
#[test]
fn c1905_base64_padding() {
    let c_code = r##"
typedef unsigned char uint8_t;
typedef unsigned long size_t;

int enc_b64_padding_needed(size_t input_len) {
    int rem = (int)(input_len % 3);
    if (rem == 0) return 0;
    return 3 - rem;
}

void enc_b64_apply_padding(char *encoded, size_t encoded_len, int padding) {
    size_t pos = encoded_len;
    int i;
    for (i = 0; i < padding; i++) {
        if (pos > 0) {
            pos--;
            encoded[pos] = '=';
        }
    }
}

size_t enc_b64_decoded_len(const char *encoded, size_t encoded_len) {
    size_t len;
    if (encoded_len == 0) return 0;
    len = (encoded_len / 4) * 3;
    if (encoded_len >= 1 && encoded[encoded_len - 1] == '=') len--;
    if (encoded_len >= 2 && encoded[encoded_len - 2] == '=') len--;
    return len;
}

int enc_b64_is_padded(const char *encoded, size_t len) {
    if (len == 0) return 0;
    return encoded[len - 1] == '=';
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1905: Base64 padding should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1905: Output should not be empty");
    assert!(code.contains("fn enc_b64_padding_needed"), "C1905: Should contain enc_b64_padding_needed function");
    assert!(code.contains("fn enc_b64_apply_padding"), "C1905: Should contain enc_b64_apply_padding function");
}

// ============================================================================
// C1906-C1910: URL Encoding
// ============================================================================

/// C1906: Percent encode -- encodes non-safe bytes as %XX hex pairs
#[test]
fn c1906_url_percent_encode() {
    let c_code = r##"
typedef unsigned long size_t;

static const char enc_url_hex[] = "0123456789ABCDEF";

int enc_url_is_unreserved(char c) {
    if (c >= 'A' && c <= 'Z') return 1;
    if (c >= 'a' && c <= 'z') return 1;
    if (c >= '0' && c <= '9') return 1;
    if (c == '-' || c == '_' || c == '.' || c == '~') return 1;
    return 0;
}

size_t enc_url_percent_encode(const char *src, size_t src_len, char *dst, size_t dst_cap) {
    size_t i, j = 0;
    unsigned char uc;
    for (i = 0; i < src_len && j < dst_cap; i++) {
        if (enc_url_is_unreserved(src[i])) {
            dst[j++] = src[i];
        } else {
            if (j + 3 > dst_cap) break;
            uc = (unsigned char)src[i];
            dst[j++] = '%';
            dst[j++] = enc_url_hex[(uc >> 4) & 0x0F];
            dst[j++] = enc_url_hex[uc & 0x0F];
        }
    }
    if (j < dst_cap) dst[j] = '\0';
    return j;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1906: URL percent encode should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1906: Output should not be empty");
    assert!(code.contains("fn enc_url_percent_encode"), "C1906: Should contain enc_url_percent_encode function");
}

/// C1907: URL percent decode -- converts %XX hex pairs back to bytes
#[test]
fn c1907_url_percent_decode() {
    let c_code = r##"
typedef unsigned long size_t;

static int enc_url_hex_val(char c) {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'A' && c <= 'F') return c - 'A' + 10;
    if (c >= 'a' && c <= 'f') return c - 'a' + 10;
    return -1;
}

size_t enc_url_percent_decode(const char *src, size_t src_len, char *dst, size_t dst_cap) {
    size_t i = 0, j = 0;
    int hi, lo;
    while (i < src_len && j < dst_cap) {
        if (src[i] == '%' && i + 2 < src_len) {
            hi = enc_url_hex_val(src[i + 1]);
            lo = enc_url_hex_val(src[i + 2]);
            if (hi >= 0 && lo >= 0) {
                dst[j++] = (char)((hi << 4) | lo);
                i += 3;
                continue;
            }
        }
        if (src[i] == '+') {
            dst[j++] = ' ';
        } else {
            dst[j++] = src[i];
        }
        i++;
    }
    if (j < dst_cap) dst[j] = '\0';
    return j;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1907: URL percent decode should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1907: Output should not be empty");
    assert!(code.contains("fn enc_url_percent_decode"), "C1907: Should contain enc_url_percent_decode function");
}

/// C1908: URL reserved character detection -- identifies RFC 3986 reserved chars
#[test]
fn c1908_url_reserved_chars() {
    let c_code = r##"
int enc_url_is_gen_delim(char c) {
    return c == ':' || c == '/' || c == '?' || c == '#' ||
           c == '[' || c == ']' || c == '@';
}

int enc_url_is_sub_delim(char c) {
    return c == '!' || c == '$' || c == '&' || c == '\'' ||
           c == '(' || c == ')' || c == '*' || c == '+' ||
           c == ',' || c == ';' || c == '=';
}

int enc_url_is_reserved(char c) {
    return enc_url_is_gen_delim(c) || enc_url_is_sub_delim(c);
}

int enc_url_needs_encoding(char c) {
    if (c >= 'A' && c <= 'Z') return 0;
    if (c >= 'a' && c <= 'z') return 0;
    if (c >= '0' && c <= '9') return 0;
    if (c == '-' || c == '_' || c == '.' || c == '~') return 0;
    return 1;
}

int enc_url_count_encoded_chars(const char *str, int len) {
    int count = 0;
    int i;
    for (i = 0; i < len; i++) {
        if (enc_url_needs_encoding(str[i])) count++;
    }
    return count;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1908: URL reserved chars should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1908: Output should not be empty");
    assert!(code.contains("fn enc_url_is_reserved"), "C1908: Should contain enc_url_is_reserved function");
    assert!(code.contains("fn enc_url_needs_encoding"), "C1908: Should contain enc_url_needs_encoding function");
}

/// C1909: URL query string encoder -- encodes key=value pairs with & separators
#[test]
fn c1909_url_query_string() {
    let c_code = r##"
typedef unsigned long size_t;

static const char enc_qs_hex[] = "0123456789ABCDEF";

static int enc_qs_safe(char c) {
    if (c >= 'A' && c <= 'Z') return 1;
    if (c >= 'a' && c <= 'z') return 1;
    if (c >= '0' && c <= '9') return 1;
    if (c == '-' || c == '_' || c == '.') return 1;
    return 0;
}

static size_t enc_qs_append(const char *src, size_t src_len, char *dst, size_t pos, size_t cap) {
    size_t i;
    unsigned char uc;
    for (i = 0; i < src_len && pos < cap; i++) {
        if (enc_qs_safe(src[i])) {
            dst[pos++] = src[i];
        } else if (src[i] == ' ') {
            dst[pos++] = '+';
        } else {
            if (pos + 3 > cap) break;
            uc = (unsigned char)src[i];
            dst[pos++] = '%';
            dst[pos++] = enc_qs_hex[(uc >> 4) & 0x0F];
            dst[pos++] = enc_qs_hex[uc & 0x0F];
        }
    }
    return pos;
}

size_t enc_qs_build(const char *key, size_t key_len,
                    const char *val, size_t val_len,
                    char *dst, size_t cap) {
    size_t pos = 0;
    pos = enc_qs_append(key, key_len, dst, pos, cap);
    if (pos < cap) dst[pos++] = '=';
    pos = enc_qs_append(val, val_len, dst, pos, cap);
    if (pos < cap) dst[pos] = '\0';
    return pos;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1909: URL query string should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1909: Output should not be empty");
    assert!(code.contains("fn enc_qs_build"), "C1909: Should contain enc_qs_build function");
}

/// C1910: URL path encoder -- encodes path segments preserving slash separators
#[test]
fn c1910_url_path_encode() {
    let c_code = r##"
typedef unsigned long size_t;

static const char enc_path_hex[] = "0123456789ABCDEF";

static int enc_path_is_safe(char c) {
    if (c >= 'A' && c <= 'Z') return 1;
    if (c >= 'a' && c <= 'z') return 1;
    if (c >= '0' && c <= '9') return 1;
    if (c == '-' || c == '_' || c == '.' || c == '~') return 1;
    if (c == '/') return 1;
    return 0;
}

size_t enc_path_encode(const char *src, size_t src_len, char *dst, size_t dst_cap) {
    size_t i, j = 0;
    unsigned char uc;
    for (i = 0; i < src_len && j < dst_cap; i++) {
        if (enc_path_is_safe(src[i])) {
            dst[j++] = src[i];
        } else {
            if (j + 3 > dst_cap) break;
            uc = (unsigned char)src[i];
            dst[j++] = '%';
            dst[j++] = enc_path_hex[(uc >> 4) & 0x0F];
            dst[j++] = enc_path_hex[uc & 0x0F];
        }
    }
    if (j < dst_cap) dst[j] = '\0';
    return j;
}

int enc_path_count_segments(const char *path, size_t len) {
    int count = 0;
    size_t i;
    for (i = 0; i < len; i++) {
        if (path[i] == '/') count++;
    }
    return count + 1;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1910: URL path encode should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1910: Output should not be empty");
    assert!(code.contains("fn enc_path_encode"), "C1910: Should contain enc_path_encode function");
}

// ============================================================================
// C1911-C1915: Run-Length Encoding
// ============================================================================

/// C1911: RLE compress -- encodes repeated byte runs as (count, byte) pairs
#[test]
fn c1911_rle_compress() {
    let c_code = r##"
typedef unsigned char uint8_t;
typedef unsigned long size_t;

size_t enc_rle_compress(const uint8_t *src, size_t src_len, uint8_t *dst, size_t dst_cap) {
    size_t i = 0, j = 0;
    uint8_t current;
    uint8_t count;
    while (i < src_len && j + 2 <= dst_cap) {
        current = src[i];
        count = 1;
        while (i + count < src_len && src[i + count] == current && count < 255) {
            count++;
        }
        dst[j++] = count;
        dst[j++] = current;
        i += count;
    }
    return j;
}

size_t enc_rle_compressed_size(const uint8_t *src, size_t src_len) {
    size_t i = 0;
    size_t pairs = 0;
    uint8_t count;
    while (i < src_len) {
        count = 1;
        while (i + count < src_len && src[i + count] == src[i] && count < 255) {
            count++;
        }
        pairs++;
        i += count;
    }
    return pairs * 2;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1911: RLE compress should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1911: Output should not be empty");
    assert!(code.contains("fn enc_rle_compress"), "C1911: Should contain enc_rle_compress function");
}

/// C1912: RLE decompress -- expands (count, byte) pairs back to original data
#[test]
fn c1912_rle_decompress() {
    let c_code = r##"
typedef unsigned char uint8_t;
typedef unsigned long size_t;

size_t enc_rle_decompress(const uint8_t *src, size_t src_len, uint8_t *dst, size_t dst_cap) {
    size_t i = 0, j = 0;
    uint8_t count, val;
    int k;
    while (i + 1 < src_len && j < dst_cap) {
        count = src[i++];
        val = src[i++];
        for (k = 0; k < count && j < dst_cap; k++) {
            dst[j++] = val;
        }
    }
    return j;
}

size_t enc_rle_decompressed_size(const uint8_t *src, size_t src_len) {
    size_t i = 0;
    size_t total = 0;
    while (i + 1 < src_len) {
        total += src[i];
        i += 2;
    }
    return total;
}

int enc_rle_validate(const uint8_t *compressed, size_t len) {
    if (len % 2 != 0) return 0;
    size_t i;
    for (i = 0; i < len; i += 2) {
        if (compressed[i] == 0) return 0;
    }
    return 1;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1912: RLE decompress should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1912: Output should not be empty");
    assert!(code.contains("fn enc_rle_decompress"), "C1912: Should contain enc_rle_decompress function");
}

/// C1913: RLE count runs -- analyzes byte stream for run statistics
#[test]
fn c1913_rle_count_runs() {
    let c_code = r##"
typedef unsigned char uint8_t;
typedef unsigned long size_t;

typedef struct {
    int total_runs;
    int max_run_length;
    int single_runs;
    size_t total_bytes;
} enc_rle_stats_t;

void enc_rle_count_runs(const uint8_t *data, size_t len, enc_rle_stats_t *stats) {
    size_t i = 0;
    int run_len;
    stats->total_runs = 0;
    stats->max_run_length = 0;
    stats->single_runs = 0;
    stats->total_bytes = len;
    while (i < len) {
        run_len = 1;
        while (i + run_len < len && data[i + run_len] == data[i]) {
            run_len++;
        }
        stats->total_runs++;
        if (run_len > stats->max_run_length) {
            stats->max_run_length = run_len;
        }
        if (run_len == 1) {
            stats->single_runs++;
        }
        i += run_len;
    }
}

float enc_rle_compression_ratio(const enc_rle_stats_t *stats) {
    if (stats->total_bytes == 0) return 0.0f;
    return (float)(stats->total_runs * 2) / (float)stats->total_bytes;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1913: RLE count runs should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1913: Output should not be empty");
    assert!(code.contains("fn enc_rle_count_runs"), "C1913: Should contain enc_rle_count_runs function");
}

/// C1914: RLE mixed data -- handles alternating literal and run segments
#[test]
fn c1914_rle_mixed_data() {
    let c_code = r##"
typedef unsigned char uint8_t;
typedef unsigned long size_t;

#define ENC_RLE_RUN_FLAG 0x80
#define ENC_RLE_MAX_LIT  127
#define ENC_RLE_MAX_RUN  127

size_t enc_rle_mixed_encode(const uint8_t *src, size_t src_len, uint8_t *dst, size_t dst_cap) {
    size_t i = 0, j = 0;
    int run_len;
    int lit_start;
    int lit_len;
    while (i < src_len && j < dst_cap) {
        run_len = 1;
        while (i + run_len < src_len && src[i + run_len] == src[i] && run_len < ENC_RLE_MAX_RUN) {
            run_len++;
        }
        if (run_len >= 3) {
            if (j + 2 > dst_cap) break;
            dst[j++] = (uint8_t)(ENC_RLE_RUN_FLAG | run_len);
            dst[j++] = src[i];
            i += run_len;
        } else {
            lit_start = (int)i;
            lit_len = 0;
            while (i < src_len && lit_len < ENC_RLE_MAX_LIT) {
                if (i + 2 < src_len && src[i] == src[i + 1] && src[i] == src[i + 2]) break;
                lit_len++;
                i++;
            }
            if (j + 1 + lit_len > dst_cap) break;
            dst[j++] = (uint8_t)lit_len;
            int k;
            for (k = 0; k < lit_len; k++) {
                dst[j++] = src[lit_start + k];
            }
        }
    }
    return j;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1914: RLE mixed data should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1914: Output should not be empty");
    assert!(code.contains("fn enc_rle_mixed_encode"), "C1914: Should contain enc_rle_mixed_encode function");
}

/// C1915: RLE worst case -- handles the pathological case of no repeated bytes
#[test]
fn c1915_rle_worst_case() {
    let c_code = r##"
typedef unsigned char uint8_t;
typedef unsigned long size_t;

int enc_rle_is_worst_case(const uint8_t *data, size_t len) {
    size_t i;
    for (i = 1; i < len; i++) {
        if (data[i] == data[i - 1]) return 0;
    }
    return 1;
}

size_t enc_rle_worst_size(size_t input_len) {
    return input_len * 2;
}

size_t enc_rle_escape_encode(const uint8_t *src, size_t src_len, uint8_t *dst, size_t dst_cap) {
    size_t i, j = 0;
    for (i = 0; i < src_len && j + 2 <= dst_cap; i++) {
        dst[j++] = 1;
        dst[j++] = src[i];
    }
    return j;
}

float enc_rle_efficiency(size_t original_len, size_t compressed_len) {
    if (original_len == 0) return 0.0f;
    return 1.0f - ((float)compressed_len / (float)original_len);
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1915: RLE worst case should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1915: Output should not be empty");
    assert!(code.contains("fn enc_rle_is_worst_case"), "C1915: Should contain enc_rle_is_worst_case function");
    assert!(code.contains("fn enc_rle_escape_encode"), "C1915: Should contain enc_rle_escape_encode function");
}

// ============================================================================
// C1916-C1920: Variable-Length Integer Encoding
// ============================================================================

/// C1916: Varint encode -- encodes unsigned integer with 7-bit continuation groups
#[test]
fn c1916_varint_encode() {
    let c_code = r##"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef unsigned long size_t;

size_t enc_varint_encode(uint32_t value, uint8_t *dst, size_t dst_cap) {
    size_t i = 0;
    while (value >= 0x80 && i < dst_cap) {
        dst[i++] = (uint8_t)(value | 0x80);
        value >>= 7;
    }
    if (i < dst_cap) {
        dst[i++] = (uint8_t)value;
    }
    return i;
}

int enc_varint_encoded_size(uint32_t value) {
    int size = 1;
    while (value >= 0x80) {
        value >>= 7;
        size++;
    }
    return size;
}

int enc_varint_max_bytes(void) {
    return 5;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1916: Varint encode should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1916: Output should not be empty");
    assert!(code.contains("fn enc_varint_encode"), "C1916: Should contain enc_varint_encode function");
}

/// C1917: Varint decode -- decodes variable-length integer back to uint32
#[test]
fn c1917_varint_decode() {
    let c_code = r##"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef unsigned long size_t;

int enc_varint_decode(const uint8_t *src, size_t src_len, uint32_t *out_value, size_t *bytes_read) {
    uint32_t result = 0;
    size_t i = 0;
    int shift = 0;
    while (i < src_len) {
        uint32_t byte_val = src[i];
        result |= (byte_val & 0x7F) << shift;
        i++;
        if ((byte_val & 0x80) == 0) {
            *out_value = result;
            *bytes_read = i;
            return 0;
        }
        shift += 7;
        if (shift >= 35) {
            return -1;
        }
    }
    return -1;
}

int enc_varint_peek_size(uint8_t first_byte) {
    if ((first_byte & 0x80) == 0) return 1;
    return -1;
}

int enc_varint_is_single_byte(uint32_t value) {
    return value < 128;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1917: Varint decode should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1917: Output should not be empty");
    assert!(code.contains("fn enc_varint_decode"), "C1917: Should contain enc_varint_decode function");
}

/// C1918: Zigzag encode -- maps signed integers to unsigned using zigzag encoding
#[test]
fn c1918_zigzag_encode() {
    let c_code = r##"
typedef unsigned int uint32_t;

uint32_t enc_zigzag_encode(int value) {
    return (uint32_t)((value << 1) ^ (value >> 31));
}

uint32_t enc_zigzag_encode_pair(int a, int b) {
    uint32_t za = enc_zigzag_encode(a);
    uint32_t zb = enc_zigzag_encode(b);
    return (za << 16) | (zb & 0xFFFF);
}

int enc_zigzag_is_negative(uint32_t encoded) {
    return (encoded & 1) != 0;
}

uint32_t enc_zigzag_magnitude(uint32_t encoded) {
    if (encoded & 1) {
        return (encoded >> 1) + 1;
    }
    return encoded >> 1;
}

int enc_zigzag_compare(uint32_t a, uint32_t b) {
    uint32_t mag_a = enc_zigzag_magnitude(a);
    uint32_t mag_b = enc_zigzag_magnitude(b);
    if (mag_a < mag_b) return -1;
    if (mag_a > mag_b) return 1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1918: Zigzag encode should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1918: Output should not be empty");
    assert!(code.contains("fn enc_zigzag_encode"), "C1918: Should contain enc_zigzag_encode function");
}

/// C1919: Zigzag decode -- recovers signed integers from zigzag encoding
#[test]
fn c1919_zigzag_decode() {
    let c_code = r##"
typedef unsigned int uint32_t;

int enc_zigzag_decode(uint32_t encoded) {
    return (int)((encoded >> 1) ^ (-(int)(encoded & 1)));
}

int enc_zigzag_roundtrip(int value) {
    uint32_t encoded = (uint32_t)((value << 1) ^ (value >> 31));
    return enc_zigzag_decode(encoded);
}

int enc_zigzag_decode_delta(const uint32_t *encoded, int count, int *decoded) {
    int prev = 0;
    int i;
    for (i = 0; i < count; i++) {
        int delta = enc_zigzag_decode(encoded[i]);
        prev += delta;
        decoded[i] = prev;
    }
    return count;
}

int enc_zigzag_min_bytes(int value) {
    uint32_t encoded = (uint32_t)((value << 1) ^ (value >> 31));
    if (encoded < 128) return 1;
    if (encoded < 16384) return 2;
    if (encoded < 2097152) return 3;
    if (encoded < 268435456) return 4;
    return 5;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1919: Zigzag decode should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1919: Output should not be empty");
    assert!(code.contains("fn enc_zigzag_decode"), "C1919: Should contain enc_zigzag_decode function");
}

/// C1920: Multi-byte varint -- handles 64-bit values with variable-length encoding
#[test]
fn c1920_varint_multibyte() {
    let c_code = r##"
typedef unsigned char uint8_t;
typedef unsigned long uint64_t;
typedef unsigned long size_t;

size_t enc_varint64_encode(uint64_t value, uint8_t *dst, size_t dst_cap) {
    size_t i = 0;
    while (value >= 0x80 && i < dst_cap) {
        dst[i++] = (uint8_t)(value | 0x80);
        value >>= 7;
    }
    if (i < dst_cap) {
        dst[i++] = (uint8_t)value;
    }
    return i;
}

int enc_varint64_decode(const uint8_t *src, size_t src_len, uint64_t *out_value, size_t *bytes_read) {
    uint64_t result = 0;
    size_t i = 0;
    int shift = 0;
    while (i < src_len && shift < 63) {
        uint64_t byte_val = src[i];
        result |= (byte_val & 0x7F) << shift;
        i++;
        if ((byte_val & 0x80) == 0) {
            *out_value = result;
            *bytes_read = i;
            return 0;
        }
        shift += 7;
    }
    return -1;
}

int enc_varint64_encoded_size(uint64_t value) {
    int size = 1;
    while (value >= 0x80) {
        value >>= 7;
        size++;
    }
    return size;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1920: Varint multi-byte should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1920: Output should not be empty");
    assert!(code.contains("fn enc_varint64_encode"), "C1920: Should contain enc_varint64_encode function");
}

// ============================================================================
// C1921-C1925: Bit Packing
// ============================================================================

/// C1921: Pack bits -- packs multiple small values into a single word
#[test]
fn c1921_pack_bits() {
    let c_code = r##"
typedef unsigned int uint32_t;

uint32_t enc_pack_bits(const int *values, const int *widths, int count) {
    uint32_t packed = 0;
    int offset = 0;
    int i;
    for (i = 0; i < count; i++) {
        uint32_t mask = (1u << widths[i]) - 1;
        packed |= ((uint32_t)values[i] & mask) << offset;
        offset += widths[i];
    }
    return packed;
}

uint32_t enc_pack_two(uint32_t a, int a_bits, uint32_t b, int b_bits) {
    uint32_t mask_a = (1u << a_bits) - 1;
    uint32_t mask_b = (1u << b_bits) - 1;
    return (a & mask_a) | ((b & mask_b) << a_bits);
}

int enc_pack_total_bits(const int *widths, int count) {
    int total = 0;
    int i;
    for (i = 0; i < count; i++) {
        total += widths[i];
    }
    return total;
}

int enc_pack_fits_32(const int *widths, int count) {
    return enc_pack_total_bits(widths, count) <= 32;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1921: Pack bits should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1921: Output should not be empty");
    assert!(code.contains("fn enc_pack_bits"), "C1921: Should contain enc_pack_bits function");
}

/// C1922: Unpack bits -- extracts multiple small values from a packed word
#[test]
fn c1922_unpack_bits() {
    let c_code = r##"
typedef unsigned int uint32_t;

void enc_unpack_bits(uint32_t packed, const int *widths, int count, int *values) {
    int offset = 0;
    int i;
    for (i = 0; i < count; i++) {
        uint32_t mask = (1u << widths[i]) - 1;
        values[i] = (int)((packed >> offset) & mask);
        offset += widths[i];
    }
}

uint32_t enc_unpack_one(uint32_t packed, int offset, int width) {
    uint32_t mask = (1u << width) - 1;
    return (packed >> offset) & mask;
}

int enc_unpack_signed(uint32_t packed, int offset, int width) {
    uint32_t raw = enc_unpack_one(packed, offset, width);
    uint32_t sign_bit = 1u << (width - 1);
    if (raw & sign_bit) {
        return (int)(raw | ~((1u << width) - 1));
    }
    return (int)raw;
}

int enc_unpack_count_nonzero(uint32_t packed, const int *widths, int count) {
    int nonzero = 0;
    int offset = 0;
    int i;
    for (i = 0; i < count; i++) {
        uint32_t mask = (1u << widths[i]) - 1;
        if ((packed >> offset) & mask) nonzero++;
        offset += widths[i];
    }
    return nonzero;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1922: Unpack bits should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1922: Output should not be empty");
    assert!(code.contains("fn enc_unpack_bits"), "C1922: Should contain enc_unpack_bits function");
}

/// C1923: Bit field extraction -- extracts named fields from a bitfield struct
#[test]
fn c1923_bitfield_extraction() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint32_t word;
} enc_bitfield_t;

void enc_bitfield_init(enc_bitfield_t *bf) {
    bf->word = 0;
}

uint32_t enc_bitfield_get(const enc_bitfield_t *bf, int offset, int width) {
    uint32_t mask = (1u << width) - 1;
    return (bf->word >> offset) & mask;
}

void enc_bitfield_set(enc_bitfield_t *bf, int offset, int width, uint32_t value) {
    uint32_t mask = (1u << width) - 1;
    bf->word &= ~(mask << offset);
    bf->word |= (value & mask) << offset;
}

int enc_bitfield_test(const enc_bitfield_t *bf, int bit_pos) {
    return (bf->word >> bit_pos) & 1;
}

void enc_bitfield_toggle(enc_bitfield_t *bf, int bit_pos) {
    bf->word ^= (1u << bit_pos);
}

int enc_bitfield_popcount(const enc_bitfield_t *bf) {
    uint32_t v = bf->word;
    int count = 0;
    while (v) {
        count += v & 1;
        v >>= 1;
    }
    return count;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1923: Bitfield extraction should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1923: Output should not be empty");
    assert!(code.contains("fn enc_bitfield_get"), "C1923: Should contain enc_bitfield_get function");
    assert!(code.contains("fn enc_bitfield_set"), "C1923: Should contain enc_bitfield_set function");
}

/// C1924: Bit field insertion -- inserts values into specific bit positions
#[test]
fn c1924_bitfield_insertion() {
    let c_code = r##"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t fields[4];
    int used_bits;
} enc_bitstream_t;

void enc_bitstream_init(enc_bitstream_t *bs) {
    bs->fields[0] = 0;
    bs->fields[1] = 0;
    bs->fields[2] = 0;
    bs->fields[3] = 0;
    bs->used_bits = 0;
}

int enc_bitstream_insert(enc_bitstream_t *bs, uint32_t value, int width) {
    int word_idx = bs->used_bits / 32;
    int bit_idx = bs->used_bits % 32;
    uint32_t mask;
    if (word_idx >= 4) return -1;
    mask = (1u << width) - 1;
    bs->fields[word_idx] |= (value & mask) << bit_idx;
    if (bit_idx + width > 32 && word_idx + 1 < 4) {
        int overflow = bit_idx + width - 32;
        bs->fields[word_idx + 1] |= (value & mask) >> (width - overflow);
    }
    bs->used_bits += width;
    return 0;
}

int enc_bitstream_remaining(const enc_bitstream_t *bs) {
    return 128 - bs->used_bits;
}

uint32_t enc_bitstream_read(const enc_bitstream_t *bs, int offset, int width) {
    int word_idx = offset / 32;
    int bit_idx = offset % 32;
    uint32_t mask = (1u << width) - 1;
    return (bs->fields[word_idx] >> bit_idx) & mask;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1924: Bitfield insertion should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1924: Output should not be empty");
    assert!(code.contains("fn enc_bitstream_insert"), "C1924: Should contain enc_bitstream_insert function");
    assert!(code.contains("fn enc_bitstream_read"), "C1924: Should contain enc_bitstream_read function");
}

/// C1925: Bit alignment -- aligns bit positions to byte/word boundaries
#[test]
fn c1925_bit_alignment() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned long size_t;

int enc_align_up(int value, int alignment) {
    return (value + alignment - 1) & ~(alignment - 1);
}

int enc_align_to_byte(int bit_position) {
    return enc_align_up(bit_position, 8);
}

int enc_align_to_word(int bit_position) {
    return enc_align_up(bit_position, 32);
}

int enc_padding_bits(int bit_position, int alignment) {
    int aligned = enc_align_up(bit_position, alignment);
    return aligned - bit_position;
}

size_t enc_packed_byte_size(const int *widths, int count) {
    int total_bits = 0;
    int i;
    for (i = 0; i < count; i++) {
        total_bits += widths[i];
    }
    return (size_t)enc_align_up(total_bits, 8) / 8;
}

int enc_is_aligned(int bit_position, int alignment) {
    return (bit_position % alignment) == 0;
}

uint32_t enc_align_mask(int alignment) {
    return ~((uint32_t)alignment - 1);
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1925: Bit alignment should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1925: Output should not be empty");
    assert!(code.contains("fn enc_align_up"), "C1925: Should contain enc_align_up function");
    assert!(code.contains("fn enc_align_to_byte"), "C1925: Should contain enc_align_to_byte function");
    assert!(code.contains("fn enc_packed_byte_size"), "C1925: Should contain enc_packed_byte_size function");
}
