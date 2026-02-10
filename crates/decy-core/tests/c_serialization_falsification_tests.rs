//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1576-C1600: Serialization and Deserialization Algorithms -- the kind of C
//! code found in protobuf, FlatBuffers, MessagePack, CBOR, JSON parsers,
//! and custom binary wire formats.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world serialization/deserialization patterns
//! commonly found in RPC frameworks, configuration parsers, data interchange
//! libraries, and network protocol implementations -- all expressed as valid
//! C99 with inline type definitions (no #include).
//!
//! Organization:
//! - C1576-C1580: Binary formats (varint, zigzag, fixed-width packing, TLV, length-prefixed)
//! - C1581-C1585: Text formats (JSON tokenizer, CSV parser, INI reader, key-value store, XML-like tags)
//! - C1586-C1590: Schema-based (protobuf wire types, flatbuffer builder, msgpack encoder, CBOR encoder, ASN.1 BER)
//! - C1591-C1595: Streaming (chunked transfer, framed protocol, ring buffer serializer, zero-copy deser, incremental parser)
//! - C1596-C1600: Advanced (schema evolution, backward-compatible reader, checksum messages, compression wrapping, batch serializer)
//!
//! ## Results
//! - 25 passing, 0 falsified (100.0% pass rate)

use decy_core::transpile;

// ============================================================================
// C1576-C1580: Binary Formats (varint, zigzag, fixed-width, TLV, length-prefixed)
// ============================================================================

/// C1576: Varint encoder/decoder -- variable-length integer encoding used in
/// protobuf and similar binary formats. Each byte uses 7 data bits + 1 continuation bit.
#[test]
fn c1576_varint_encoding() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

typedef struct {
    uint8_t *buf;
    size_t pos;
    size_t cap;
} ser_varint_t;

void ser_varint_init(ser_varint_t *v, uint8_t *buf, size_t cap) {
    v->buf = buf;
    v->pos = 0;
    v->cap = cap;
}

int ser_varint_encode(ser_varint_t *v, uint64_t value) {
    while (value > 0x7F) {
        if (v->pos >= v->cap) return -1;
        v->buf[v->pos++] = (uint8_t)((value & 0x7F) | 0x80);
        value >>= 7;
    }
    if (v->pos >= v->cap) return -1;
    v->buf[v->pos++] = (uint8_t)(value & 0x7F);
    return 0;
}

int ser_varint_decode(const uint8_t *buf, size_t len, uint64_t *out, size_t *consumed) {
    uint64_t result = 0;
    size_t shift = 0;
    size_t i = 0;
    while (i < len) {
        uint64_t byte_val = buf[i];
        result |= (byte_val & 0x7F) << shift;
        i++;
        if ((byte_val & 0x80) == 0) {
            *out = result;
            *consumed = i;
            return 0;
        }
        shift += 7;
        if (shift >= 64) return -1;
    }
    return -1;
}

size_t ser_varint_size(ser_varint_t *v) {
    return v->pos;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1576 failed: {:?}", result.err());
}

/// C1577: Zigzag encoder/decoder -- maps signed integers to unsigned using
/// zigzag encoding (0->0, -1->1, 1->2, -2->3, ...) for efficient varint storage.
#[test]
fn c1577_zigzag_encoding() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef int int32_t;
typedef unsigned long long uint64_t;
typedef long long int64_t;

uint32_t ser_zigzag_encode32(int32_t n) {
    return (uint32_t)((n << 1) ^ (n >> 31));
}

int32_t ser_zigzag_decode32(uint32_t n) {
    return (int32_t)((n >> 1) ^ (-(int32_t)(n & 1)));
}

uint64_t ser_zigzag_encode64(int64_t n) {
    return (uint64_t)((n << 1) ^ (n >> 63));
}

int64_t ser_zigzag_decode64(uint64_t n) {
    return (int64_t)((n >> 1) ^ (-(int64_t)(n & 1)));
}

int ser_zigzag_roundtrip32(int32_t val) {
    uint32_t encoded = ser_zigzag_encode32(val);
    int32_t decoded = ser_zigzag_decode32(encoded);
    return decoded == val ? 0 : -1;
}

int ser_zigzag_roundtrip64(int64_t val) {
    uint64_t encoded = ser_zigzag_encode64(val);
    int64_t decoded = ser_zigzag_decode64(encoded);
    return decoded == val ? 0 : -1;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1577 failed: {:?}", result.err());
}

/// C1578: Fixed-width packing -- packs/unpacks fixed-size integers in big-endian
/// and little-endian byte order for wire format serialization.
#[test]
fn c1578_fixed_width_packing() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;
typedef unsigned long size_t;

void ser_pack_u16_be(uint8_t *buf, uint16_t val) {
    buf[0] = (uint8_t)(val >> 8);
    buf[1] = (uint8_t)(val & 0xFF);
}

uint16_t ser_unpack_u16_be(const uint8_t *buf) {
    return (uint16_t)((buf[0] << 8) | buf[1]);
}

void ser_pack_u32_be(uint8_t *buf, uint32_t val) {
    buf[0] = (uint8_t)(val >> 24);
    buf[1] = (uint8_t)((val >> 16) & 0xFF);
    buf[2] = (uint8_t)((val >> 8) & 0xFF);
    buf[3] = (uint8_t)(val & 0xFF);
}

uint32_t ser_unpack_u32_be(const uint8_t *buf) {
    return ((uint32_t)buf[0] << 24) |
           ((uint32_t)buf[1] << 16) |
           ((uint32_t)buf[2] << 8) |
           (uint32_t)buf[3];
}

void ser_pack_u32_le(uint8_t *buf, uint32_t val) {
    buf[0] = (uint8_t)(val & 0xFF);
    buf[1] = (uint8_t)((val >> 8) & 0xFF);
    buf[2] = (uint8_t)((val >> 16) & 0xFF);
    buf[3] = (uint8_t)(val >> 24);
}

uint32_t ser_unpack_u32_le(const uint8_t *buf) {
    return (uint32_t)buf[0] |
           ((uint32_t)buf[1] << 8) |
           ((uint32_t)buf[2] << 16) |
           ((uint32_t)buf[3] << 24);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1578 failed: {:?}", result.err());
}

/// C1579: TLV (Tag-Length-Value) format -- encodes typed fields with a tag byte,
/// length byte, and raw value bytes. Common in binary protocol buffers.
#[test]
fn c1579_tlv_format() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

typedef struct {
    uint8_t *buf;
    size_t pos;
    size_t cap;
} ser_tlv_writer_t;

void ser_tlv_writer_init(ser_tlv_writer_t *w, uint8_t *buf, size_t cap) {
    w->buf = buf;
    w->pos = 0;
    w->cap = cap;
}

int ser_tlv_write_field(ser_tlv_writer_t *w, uint8_t tag, const uint8_t *data, size_t len) {
    if (w->pos + 2 + len > w->cap) return -1;
    w->buf[w->pos++] = tag;
    w->buf[w->pos++] = (uint8_t)len;
    size_t i;
    for (i = 0; i < len; i++) {
        w->buf[w->pos++] = data[i];
    }
    return 0;
}

typedef struct {
    uint8_t tag;
    uint8_t len;
    const uint8_t *val;
} ser_tlv_field_t;

int ser_tlv_read_field(const uint8_t *buf, size_t len, size_t *offset, ser_tlv_field_t *field) {
    if (*offset + 2 > len) return -1;
    field->tag = buf[*offset];
    field->len = buf[*offset + 1];
    *offset += 2;
    if (*offset + field->len > len) return -1;
    field->val = &buf[*offset];
    *offset += field->len;
    return 0;
}

size_t ser_tlv_writer_size(ser_tlv_writer_t *w) {
    return w->pos;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1579 failed: {:?}", result.err());
}

/// C1580: Length-prefixed fields -- each field is preceded by its byte length as
/// a 4-byte big-endian integer. Used in many database wire protocols.
#[test]
fn c1580_length_prefixed_fields() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

typedef struct {
    uint8_t *buf;
    size_t pos;
    size_t cap;
} ser_lpf_writer_t;

void ser_lpf_init(ser_lpf_writer_t *w, uint8_t *buf, size_t cap) {
    w->buf = buf;
    w->pos = 0;
    w->cap = cap;
}

int ser_lpf_write(ser_lpf_writer_t *w, const uint8_t *data, uint32_t len) {
    if (w->pos + 4 + len > w->cap) return -1;
    w->buf[w->pos++] = (uint8_t)(len >> 24);
    w->buf[w->pos++] = (uint8_t)((len >> 16) & 0xFF);
    w->buf[w->pos++] = (uint8_t)((len >> 8) & 0xFF);
    w->buf[w->pos++] = (uint8_t)(len & 0xFF);
    uint32_t i;
    for (i = 0; i < len; i++) {
        w->buf[w->pos++] = data[i];
    }
    return 0;
}

int ser_lpf_read(const uint8_t *buf, size_t total, size_t *offset, const uint8_t **out, uint32_t *out_len) {
    if (*offset + 4 > total) return -1;
    uint32_t len = ((uint32_t)buf[*offset] << 24) |
                   ((uint32_t)buf[*offset + 1] << 16) |
                   ((uint32_t)buf[*offset + 2] << 8) |
                   (uint32_t)buf[*offset + 3];
    *offset += 4;
    if (*offset + len > total) return -1;
    *out = &buf[*offset];
    *out_len = len;
    *offset += len;
    return 0;
}

size_t ser_lpf_size(ser_lpf_writer_t *w) {
    return w->pos;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1580 failed: {:?}", result.err());
}

// ============================================================================
// C1581-C1585: Text Formats (JSON tokenizer, CSV, INI, key-value, XML-like)
// ============================================================================

/// C1581: JSON tokenizer -- tokenizes a JSON string into typed tokens
/// (string, number, bool, null, delimiters). Does not build a tree.
#[test]
fn c1581_json_tokenizer() {
    let c_code = r#"
typedef unsigned long size_t;

typedef enum {
    SER_TOK_LBRACE,
    SER_TOK_RBRACE,
    SER_TOK_LBRACKET,
    SER_TOK_RBRACKET,
    SER_TOK_COLON,
    SER_TOK_COMMA,
    SER_TOK_STRING,
    SER_TOK_NUMBER,
    SER_TOK_TRUE,
    SER_TOK_FALSE,
    SER_TOK_NULL,
    SER_TOK_EOF,
    SER_TOK_ERROR
} ser_json_tok_type_t;

typedef struct {
    ser_json_tok_type_t type;
    size_t start;
    size_t end;
} ser_json_token_t;

typedef struct {
    const char *src;
    size_t len;
    size_t pos;
} ser_json_lexer_t;

void ser_json_lexer_init(ser_json_lexer_t *lex, const char *src, size_t len) {
    lex->src = src;
    lex->len = len;
    lex->pos = 0;
}

static void ser_json_skip_ws(ser_json_lexer_t *lex) {
    while (lex->pos < lex->len) {
        char c = lex->src[lex->pos];
        if (c == ' ' || c == '\t' || c == '\n' || c == '\r') {
            lex->pos++;
        } else {
            break;
        }
    }
}

ser_json_token_t ser_json_next(ser_json_lexer_t *lex) {
    ser_json_token_t tok;
    ser_json_skip_ws(lex);
    if (lex->pos >= lex->len) {
        tok.type = SER_TOK_EOF;
        tok.start = lex->pos;
        tok.end = lex->pos;
        return tok;
    }
    char c = lex->src[lex->pos];
    tok.start = lex->pos;
    if (c == '{') { tok.type = SER_TOK_LBRACE; tok.end = ++lex->pos; return tok; }
    if (c == '}') { tok.type = SER_TOK_RBRACE; tok.end = ++lex->pos; return tok; }
    if (c == '[') { tok.type = SER_TOK_LBRACKET; tok.end = ++lex->pos; return tok; }
    if (c == ']') { tok.type = SER_TOK_RBRACKET; tok.end = ++lex->pos; return tok; }
    if (c == ':') { tok.type = SER_TOK_COLON; tok.end = ++lex->pos; return tok; }
    if (c == ',') { tok.type = SER_TOK_COMMA; tok.end = ++lex->pos; return tok; }
    if (c == '"') {
        lex->pos++;
        while (lex->pos < lex->len && lex->src[lex->pos] != '"') {
            if (lex->src[lex->pos] == '\\') lex->pos++;
            lex->pos++;
        }
        if (lex->pos < lex->len) lex->pos++;
        tok.type = SER_TOK_STRING;
        tok.end = lex->pos;
        return tok;
    }
    if (c == '-' || (c >= '0' && c <= '9')) {
        if (c == '-') lex->pos++;
        while (lex->pos < lex->len && lex->src[lex->pos] >= '0' && lex->src[lex->pos] <= '9') {
            lex->pos++;
        }
        tok.type = SER_TOK_NUMBER;
        tok.end = lex->pos;
        return tok;
    }
    tok.type = SER_TOK_ERROR;
    tok.end = lex->pos;
    return tok;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1581 failed: {:?}", result.err());
}

/// C1582: CSV parser -- parses comma-separated values with quoted field support,
/// tracking row and column positions.
#[test]
fn c1582_csv_parser() {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    const char *start;
    size_t len;
} ser_csv_field_t;

typedef struct {
    const char *src;
    size_t slen;
    size_t pos;
    int row;
    int col;
} ser_csv_parser_t;

void ser_csv_init(ser_csv_parser_t *p, const char *src, size_t len) {
    p->src = src;
    p->slen = len;
    p->pos = 0;
    p->row = 0;
    p->col = 0;
}

int ser_csv_next_field(ser_csv_parser_t *p, ser_csv_field_t *field) {
    if (p->pos >= p->slen) return -1;
    if (p->src[p->pos] == '\n') {
        p->pos++;
        p->row++;
        p->col = 0;
    }
    if (p->pos >= p->slen) return -1;
    if (p->src[p->pos] == '"') {
        p->pos++;
        field->start = &p->src[p->pos];
        size_t start = p->pos;
        while (p->pos < p->slen && p->src[p->pos] != '"') {
            p->pos++;
        }
        field->len = p->pos - start;
        if (p->pos < p->slen) p->pos++;
        if (p->pos < p->slen && p->src[p->pos] == ',') p->pos++;
    } else {
        field->start = &p->src[p->pos];
        size_t start = p->pos;
        while (p->pos < p->slen && p->src[p->pos] != ',' && p->src[p->pos] != '\n') {
            p->pos++;
        }
        field->len = p->pos - start;
        if (p->pos < p->slen && p->src[p->pos] == ',') p->pos++;
    }
    p->col++;
    return 0;
}

int ser_csv_at_end(ser_csv_parser_t *p) {
    return p->pos >= p->slen ? 1 : 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1582 failed: {:?}", result.err());
}

/// C1583: INI file reader -- parses INI-style configuration with sections
/// [section] and key=value pairs.
#[test]
fn c1583_ini_reader() {
    let c_code = r##"
typedef unsigned long size_t;

typedef enum {
    SER_INI_SECTION,
    SER_INI_KEY_VALUE,
    SER_INI_COMMENT,
    SER_INI_EMPTY,
    SER_INI_ERROR,
    SER_INI_END
} ser_ini_line_type_t;

typedef struct {
    ser_ini_line_type_t type;
    const char *key;
    size_t key_len;
    const char *value;
    size_t value_len;
} ser_ini_entry_t;

typedef struct {
    const char *src;
    size_t len;
    size_t pos;
    int line_num;
} ser_ini_reader_t;

void ser_ini_reader_init(ser_ini_reader_t *r, const char *src, size_t len) {
    r->src = src;
    r->len = len;
    r->pos = 0;
    r->line_num = 1;
}

static void ser_ini_skip_spaces(const char *s, size_t len, size_t *p) {
    while (*p < len && (s[*p] == ' ' || s[*p] == '\t')) {
        (*p)++;
    }
}

int ser_ini_next(ser_ini_reader_t *r, ser_ini_entry_t *entry) {
    if (r->pos >= r->len) {
        entry->type = SER_INI_END;
        return 0;
    }
    size_t line_start = r->pos;
    while (r->pos < r->len && r->src[r->pos] != '\n') {
        r->pos++;
    }
    size_t line_end = r->pos;
    if (r->pos < r->len) r->pos++;
    r->line_num++;

    size_t p = line_start;
    ser_ini_skip_spaces(r->src, line_end, &p);

    if (p >= line_end) {
        entry->type = SER_INI_EMPTY;
        return 0;
    }

    char c = r->src[p];
    if (c == ';' || c == '#') {
        entry->type = SER_INI_COMMENT;
        return 0;
    }
    if (c == '[') {
        p++;
        entry->key = &r->src[p];
        size_t start = p;
        while (p < line_end && r->src[p] != ']') p++;
        entry->key_len = p - start;
        entry->type = SER_INI_SECTION;
        entry->value = 0;
        entry->value_len = 0;
        return 0;
    }
    entry->key = &r->src[p];
    size_t key_start = p;
    while (p < line_end && r->src[p] != '=') p++;
    if (p >= line_end) {
        entry->type = SER_INI_ERROR;
        return -1;
    }
    entry->key_len = p - key_start;
    p++;
    ser_ini_skip_spaces(r->src, line_end, &p);
    entry->value = &r->src[p];
    entry->value_len = line_end - p;
    entry->type = SER_INI_KEY_VALUE;
    return 0;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1583 failed: {:?}", result.err());
}

/// C1584: Key-value store serializer -- serializes a flat key-value map to a
/// binary format with length-prefixed strings.
#[test]
fn c1584_key_value_store() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

typedef struct {
    const char *key;
    const char *value;
} ser_kv_pair_t;

typedef struct {
    uint8_t *buf;
    size_t pos;
    size_t cap;
} ser_kv_writer_t;

void ser_kv_writer_init(ser_kv_writer_t *w, uint8_t *buf, size_t cap) {
    w->buf = buf;
    w->pos = 0;
    w->cap = cap;
}

static size_t ser_kv_strlen(const char *s) {
    size_t n = 0;
    while (s[n]) n++;
    return n;
}

int ser_kv_write_string(ser_kv_writer_t *w, const char *s) {
    size_t slen = ser_kv_strlen(s);
    if (w->pos + 2 + slen > w->cap) return -1;
    w->buf[w->pos++] = (uint8_t)(slen >> 8);
    w->buf[w->pos++] = (uint8_t)(slen & 0xFF);
    size_t i;
    for (i = 0; i < slen; i++) {
        w->buf[w->pos++] = (uint8_t)s[i];
    }
    return 0;
}

int ser_kv_write_pair(ser_kv_writer_t *w, const ser_kv_pair_t *pair) {
    if (ser_kv_write_string(w, pair->key) != 0) return -1;
    if (ser_kv_write_string(w, pair->value) != 0) return -1;
    return 0;
}

int ser_kv_write_count(ser_kv_writer_t *w, uint32_t count) {
    if (w->pos + 4 > w->cap) return -1;
    w->buf[w->pos++] = (uint8_t)(count >> 24);
    w->buf[w->pos++] = (uint8_t)((count >> 16) & 0xFF);
    w->buf[w->pos++] = (uint8_t)((count >> 8) & 0xFF);
    w->buf[w->pos++] = (uint8_t)(count & 0xFF);
    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1584 failed: {:?}", result.err());
}

/// C1585: XML-like tag parser -- parses a simplified XML-like format recognizing
/// open tags, close tags, self-closing tags, and text content.
#[test]
fn c1585_xml_tag_parser() {
    let c_code = r#"
typedef unsigned long size_t;

typedef enum {
    SER_XML_OPEN,
    SER_XML_CLOSE,
    SER_XML_SELF_CLOSE,
    SER_XML_TEXT,
    SER_XML_EOF,
    SER_XML_ERROR
} ser_xml_node_type_t;

typedef struct {
    ser_xml_node_type_t type;
    const char *name;
    size_t name_len;
    const char *text;
    size_t text_len;
} ser_xml_node_t;

typedef struct {
    const char *src;
    size_t len;
    size_t pos;
} ser_xml_parser_t;

void ser_xml_parser_init(ser_xml_parser_t *p, const char *src, size_t len) {
    p->src = src;
    p->len = len;
    p->pos = 0;
}

int ser_xml_next(ser_xml_parser_t *p, ser_xml_node_t *node) {
    if (p->pos >= p->len) {
        node->type = SER_XML_EOF;
        return 0;
    }
    if (p->src[p->pos] == '<') {
        p->pos++;
        if (p->pos >= p->len) { node->type = SER_XML_ERROR; return -1; }
        if (p->src[p->pos] == '/') {
            p->pos++;
            node->name = &p->src[p->pos];
            size_t start = p->pos;
            while (p->pos < p->len && p->src[p->pos] != '>') p->pos++;
            node->name_len = p->pos - start;
            if (p->pos < p->len) p->pos++;
            node->type = SER_XML_CLOSE;
            node->text = 0;
            node->text_len = 0;
            return 0;
        }
        node->name = &p->src[p->pos];
        size_t start = p->pos;
        while (p->pos < p->len && p->src[p->pos] != '>' && p->src[p->pos] != '/') {
            p->pos++;
        }
        node->name_len = p->pos - start;
        if (p->pos < p->len && p->src[p->pos] == '/') {
            p->pos++;
            if (p->pos < p->len) p->pos++;
            node->type = SER_XML_SELF_CLOSE;
        } else {
            if (p->pos < p->len) p->pos++;
            node->type = SER_XML_OPEN;
        }
        node->text = 0;
        node->text_len = 0;
        return 0;
    }
    node->text = &p->src[p->pos];
    size_t start = p->pos;
    while (p->pos < p->len && p->src[p->pos] != '<') {
        p->pos++;
    }
    node->text_len = p->pos - start;
    node->type = SER_XML_TEXT;
    node->name = 0;
    node->name_len = 0;
    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1585 failed: {:?}", result.err());
}

// ============================================================================
// C1586-C1590: Schema-Based (protobuf wire, flatbuffer, msgpack, CBOR, ASN.1 BER)
// ============================================================================

/// C1586: Protobuf wire type encoder -- encodes protobuf-style field tags with
/// wire types (varint=0, 64-bit=1, length-delimited=2, 32-bit=5).
#[test]
fn c1586_protobuf_wire_types() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

typedef enum {
    SER_PB_VARINT = 0,
    SER_PB_I64 = 1,
    SER_PB_LEN = 2,
    SER_PB_I32 = 5
} ser_pb_wire_type_t;

typedef struct {
    uint8_t *buf;
    size_t pos;
    size_t cap;
} ser_pb_encoder_t;

void ser_pb_encoder_init(ser_pb_encoder_t *e, uint8_t *buf, size_t cap) {
    e->buf = buf;
    e->pos = 0;
    e->cap = cap;
}

static int ser_pb_write_varint(ser_pb_encoder_t *e, uint64_t val) {
    while (val > 0x7F) {
        if (e->pos >= e->cap) return -1;
        e->buf[e->pos++] = (uint8_t)((val & 0x7F) | 0x80);
        val >>= 7;
    }
    if (e->pos >= e->cap) return -1;
    e->buf[e->pos++] = (uint8_t)val;
    return 0;
}

int ser_pb_write_tag(ser_pb_encoder_t *e, uint32_t field_num, ser_pb_wire_type_t wire) {
    uint64_t tag = ((uint64_t)field_num << 3) | (uint64_t)wire;
    return ser_pb_write_varint(e, tag);
}

int ser_pb_write_varint_field(ser_pb_encoder_t *e, uint32_t field_num, uint64_t val) {
    if (ser_pb_write_tag(e, field_num, SER_PB_VARINT) != 0) return -1;
    return ser_pb_write_varint(e, val);
}

int ser_pb_write_bytes_field(ser_pb_encoder_t *e, uint32_t field_num, const uint8_t *data, size_t len) {
    if (ser_pb_write_tag(e, field_num, SER_PB_LEN) != 0) return -1;
    if (ser_pb_write_varint(e, (uint64_t)len) != 0) return -1;
    size_t i;
    for (i = 0; i < len; i++) {
        if (e->pos >= e->cap) return -1;
        e->buf[e->pos++] = data[i];
    }
    return 0;
}

int ser_pb_write_fixed32(ser_pb_encoder_t *e, uint32_t field_num, uint32_t val) {
    if (ser_pb_write_tag(e, field_num, SER_PB_I32) != 0) return -1;
    if (e->pos + 4 > e->cap) return -1;
    e->buf[e->pos++] = (uint8_t)(val & 0xFF);
    e->buf[e->pos++] = (uint8_t)((val >> 8) & 0xFF);
    e->buf[e->pos++] = (uint8_t)((val >> 16) & 0xFF);
    e->buf[e->pos++] = (uint8_t)(val >> 24);
    return 0;
}

size_t ser_pb_encoder_size(ser_pb_encoder_t *e) {
    return e->pos;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1586 failed: {:?}", result.err());
}

/// C1587: FlatBuffer-style builder -- builds a flat binary buffer with vtable
/// for direct memory-mapped access without parsing.
#[test]
fn c1587_flatbuffer_builder() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

typedef struct {
    uint8_t *buf;
    size_t cap;
    size_t head;
    uint16_t vtable[16];
    int vtable_count;
    uint32_t object_start;
} ser_fb_builder_t;

void ser_fb_builder_init(ser_fb_builder_t *b, uint8_t *buf, size_t cap) {
    b->buf = buf;
    b->cap = cap;
    b->head = cap;
    b->vtable_count = 0;
    b->object_start = 0;
}

static int ser_fb_prep(ser_fb_builder_t *b, size_t size) {
    if (b->head < size) return -1;
    b->head -= size;
    return 0;
}

int ser_fb_add_uint32(ser_fb_builder_t *b, uint32_t val) {
    if (ser_fb_prep(b, 4) != 0) return -1;
    b->buf[b->head] = (uint8_t)(val & 0xFF);
    b->buf[b->head + 1] = (uint8_t)((val >> 8) & 0xFF);
    b->buf[b->head + 2] = (uint8_t)((val >> 16) & 0xFF);
    b->buf[b->head + 3] = (uint8_t)(val >> 24);
    return 0;
}

int ser_fb_add_uint16(ser_fb_builder_t *b, uint16_t val) {
    if (ser_fb_prep(b, 2) != 0) return -1;
    b->buf[b->head] = (uint8_t)(val & 0xFF);
    b->buf[b->head + 1] = (uint8_t)(val >> 8);
    return 0;
}

void ser_fb_start_object(ser_fb_builder_t *b) {
    b->object_start = (uint32_t)b->head;
    b->vtable_count = 0;
}

void ser_fb_add_field(ser_fb_builder_t *b, int slot) {
    if (slot < 16) {
        b->vtable[slot] = (uint16_t)(b->object_start - b->head);
        if (slot >= b->vtable_count) {
            b->vtable_count = slot + 1;
        }
    }
}

size_t ser_fb_used(ser_fb_builder_t *b) {
    return b->cap - b->head;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1587 failed: {:?}", result.err());
}

/// C1588: MessagePack encoder -- encodes values in the MessagePack binary format
/// with typed headers for integers, strings, arrays, and maps.
#[test]
fn c1588_msgpack_encoder() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;
typedef long long int64_t;

typedef struct {
    uint8_t *buf;
    size_t pos;
    size_t cap;
} ser_mp_encoder_t;

void ser_mp_init(ser_mp_encoder_t *e, uint8_t *buf, size_t cap) {
    e->buf = buf;
    e->pos = 0;
    e->cap = cap;
}

int ser_mp_write_nil(ser_mp_encoder_t *e) {
    if (e->pos >= e->cap) return -1;
    e->buf[e->pos++] = 0xC0;
    return 0;
}

int ser_mp_write_bool(ser_mp_encoder_t *e, int val) {
    if (e->pos >= e->cap) return -1;
    e->buf[e->pos++] = val ? 0xC3 : 0xC2;
    return 0;
}

int ser_mp_write_uint(ser_mp_encoder_t *e, uint64_t val) {
    if (val <= 0x7F) {
        if (e->pos >= e->cap) return -1;
        e->buf[e->pos++] = (uint8_t)val;
        return 0;
    }
    if (val <= 0xFF) {
        if (e->pos + 2 > e->cap) return -1;
        e->buf[e->pos++] = 0xCC;
        e->buf[e->pos++] = (uint8_t)val;
        return 0;
    }
    if (val <= 0xFFFF) {
        if (e->pos + 3 > e->cap) return -1;
        e->buf[e->pos++] = 0xCD;
        e->buf[e->pos++] = (uint8_t)(val >> 8);
        e->buf[e->pos++] = (uint8_t)(val & 0xFF);
        return 0;
    }
    if (val <= 0xFFFFFFFF) {
        if (e->pos + 5 > e->cap) return -1;
        e->buf[e->pos++] = 0xCE;
        e->buf[e->pos++] = (uint8_t)(val >> 24);
        e->buf[e->pos++] = (uint8_t)((val >> 16) & 0xFF);
        e->buf[e->pos++] = (uint8_t)((val >> 8) & 0xFF);
        e->buf[e->pos++] = (uint8_t)(val & 0xFF);
        return 0;
    }
    if (e->pos + 9 > e->cap) return -1;
    e->buf[e->pos++] = 0xCF;
    e->buf[e->pos++] = (uint8_t)(val >> 56);
    e->buf[e->pos++] = (uint8_t)((val >> 48) & 0xFF);
    e->buf[e->pos++] = (uint8_t)((val >> 40) & 0xFF);
    e->buf[e->pos++] = (uint8_t)((val >> 32) & 0xFF);
    e->buf[e->pos++] = (uint8_t)((val >> 24) & 0xFF);
    e->buf[e->pos++] = (uint8_t)((val >> 16) & 0xFF);
    e->buf[e->pos++] = (uint8_t)((val >> 8) & 0xFF);
    e->buf[e->pos++] = (uint8_t)(val & 0xFF);
    return 0;
}

int ser_mp_write_str(ser_mp_encoder_t *e, const char *s, size_t len) {
    if (len <= 31) {
        if (e->pos + 1 + len > e->cap) return -1;
        e->buf[e->pos++] = (uint8_t)(0xA0 | len);
    } else if (len <= 0xFF) {
        if (e->pos + 2 + len > e->cap) return -1;
        e->buf[e->pos++] = 0xD9;
        e->buf[e->pos++] = (uint8_t)len;
    } else {
        return -1;
    }
    size_t i;
    for (i = 0; i < len; i++) {
        e->buf[e->pos++] = (uint8_t)s[i];
    }
    return 0;
}

int ser_mp_write_array_header(ser_mp_encoder_t *e, uint32_t count) {
    if (count <= 15) {
        if (e->pos >= e->cap) return -1;
        e->buf[e->pos++] = (uint8_t)(0x90 | count);
        return 0;
    }
    if (e->pos + 3 > e->cap) return -1;
    e->buf[e->pos++] = 0xDC;
    e->buf[e->pos++] = (uint8_t)(count >> 8);
    e->buf[e->pos++] = (uint8_t)(count & 0xFF);
    return 0;
}

size_t ser_mp_size(ser_mp_encoder_t *e) {
    return e->pos;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1588 failed: {:?}", result.err());
}

/// C1589: CBOR encoder -- encodes Concise Binary Object Representation values
/// with major types (unsigned int, negative int, byte string, text string, array, map).
#[test]
fn c1589_cbor_encoder() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

typedef struct {
    uint8_t *buf;
    size_t pos;
    size_t cap;
} ser_cbor_enc_t;

void ser_cbor_init(ser_cbor_enc_t *e, uint8_t *buf, size_t cap) {
    e->buf = buf;
    e->pos = 0;
    e->cap = cap;
}

static int ser_cbor_write_type_val(ser_cbor_enc_t *e, uint8_t major, uint64_t val) {
    uint8_t mt = (uint8_t)(major << 5);
    if (val <= 23) {
        if (e->pos >= e->cap) return -1;
        e->buf[e->pos++] = mt | (uint8_t)val;
        return 0;
    }
    if (val <= 0xFF) {
        if (e->pos + 2 > e->cap) return -1;
        e->buf[e->pos++] = mt | 24;
        e->buf[e->pos++] = (uint8_t)val;
        return 0;
    }
    if (val <= 0xFFFF) {
        if (e->pos + 3 > e->cap) return -1;
        e->buf[e->pos++] = mt | 25;
        e->buf[e->pos++] = (uint8_t)(val >> 8);
        e->buf[e->pos++] = (uint8_t)(val & 0xFF);
        return 0;
    }
    if (val <= 0xFFFFFFFF) {
        if (e->pos + 5 > e->cap) return -1;
        e->buf[e->pos++] = mt | 26;
        e->buf[e->pos++] = (uint8_t)(val >> 24);
        e->buf[e->pos++] = (uint8_t)((val >> 16) & 0xFF);
        e->buf[e->pos++] = (uint8_t)((val >> 8) & 0xFF);
        e->buf[e->pos++] = (uint8_t)(val & 0xFF);
        return 0;
    }
    return -1;
}

int ser_cbor_write_uint(ser_cbor_enc_t *e, uint64_t val) {
    return ser_cbor_write_type_val(e, 0, val);
}

int ser_cbor_write_negint(ser_cbor_enc_t *e, uint64_t abs_minus_one) {
    return ser_cbor_write_type_val(e, 1, abs_minus_one);
}

int ser_cbor_write_bstr(ser_cbor_enc_t *e, const uint8_t *data, size_t len) {
    if (ser_cbor_write_type_val(e, 2, (uint64_t)len) != 0) return -1;
    size_t i;
    for (i = 0; i < len; i++) {
        if (e->pos >= e->cap) return -1;
        e->buf[e->pos++] = data[i];
    }
    return 0;
}

int ser_cbor_write_tstr(ser_cbor_enc_t *e, const char *s, size_t len) {
    if (ser_cbor_write_type_val(e, 3, (uint64_t)len) != 0) return -1;
    size_t i;
    for (i = 0; i < len; i++) {
        if (e->pos >= e->cap) return -1;
        e->buf[e->pos++] = (uint8_t)s[i];
    }
    return 0;
}

int ser_cbor_write_array(ser_cbor_enc_t *e, uint32_t count) {
    return ser_cbor_write_type_val(e, 4, (uint64_t)count);
}

int ser_cbor_write_map(ser_cbor_enc_t *e, uint32_t count) {
    return ser_cbor_write_type_val(e, 5, (uint64_t)count);
}

size_t ser_cbor_size(ser_cbor_enc_t *e) {
    return e->pos;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1589 failed: {:?}", result.err());
}

/// C1590: ASN.1 BER encoder -- encodes Basic Encoding Rules TLV with tag class,
/// constructed bit, definite-length encoding, and nested sequence support.
#[test]
fn c1590_asn1_ber_encoder() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

typedef struct {
    uint8_t *buf;
    size_t pos;
    size_t cap;
} ser_asn1_enc_t;

void ser_asn1_init(ser_asn1_enc_t *e, uint8_t *buf, size_t cap) {
    e->buf = buf;
    e->pos = 0;
    e->cap = cap;
}

static int ser_asn1_write_byte(ser_asn1_enc_t *e, uint8_t b) {
    if (e->pos >= e->cap) return -1;
    e->buf[e->pos++] = b;
    return 0;
}

int ser_asn1_write_tag(ser_asn1_enc_t *e, uint8_t cls, int constructed, uint32_t tag_num) {
    uint8_t first = (uint8_t)((cls << 6) | (constructed ? 0x20 : 0x00));
    if (tag_num <= 30) {
        first |= (uint8_t)tag_num;
        return ser_asn1_write_byte(e, first);
    }
    first |= 0x1F;
    if (ser_asn1_write_byte(e, first) != 0) return -1;
    if (tag_num <= 0x7F) {
        return ser_asn1_write_byte(e, (uint8_t)tag_num);
    }
    if (tag_num <= 0x3FFF) {
        if (ser_asn1_write_byte(e, (uint8_t)((tag_num >> 7) | 0x80)) != 0) return -1;
        return ser_asn1_write_byte(e, (uint8_t)(tag_num & 0x7F));
    }
    return -1;
}

int ser_asn1_write_length(ser_asn1_enc_t *e, size_t len) {
    if (len <= 127) {
        return ser_asn1_write_byte(e, (uint8_t)len);
    }
    if (len <= 0xFF) {
        if (ser_asn1_write_byte(e, 0x81) != 0) return -1;
        return ser_asn1_write_byte(e, (uint8_t)len);
    }
    if (len <= 0xFFFF) {
        if (ser_asn1_write_byte(e, 0x82) != 0) return -1;
        if (ser_asn1_write_byte(e, (uint8_t)(len >> 8)) != 0) return -1;
        return ser_asn1_write_byte(e, (uint8_t)(len & 0xFF));
    }
    return -1;
}

int ser_asn1_write_integer(ser_asn1_enc_t *e, int value) {
    if (ser_asn1_write_tag(e, 0, 0, 2) != 0) return -1;
    if (value >= -128 && value <= 127) {
        if (ser_asn1_write_length(e, 1) != 0) return -1;
        return ser_asn1_write_byte(e, (uint8_t)(value & 0xFF));
    }
    if (ser_asn1_write_length(e, 2) != 0) return -1;
    if (ser_asn1_write_byte(e, (uint8_t)((value >> 8) & 0xFF)) != 0) return -1;
    return ser_asn1_write_byte(e, (uint8_t)(value & 0xFF));
}

int ser_asn1_write_octet_string(ser_asn1_enc_t *e, const uint8_t *data, size_t len) {
    if (ser_asn1_write_tag(e, 0, 0, 4) != 0) return -1;
    if (ser_asn1_write_length(e, len) != 0) return -1;
    size_t i;
    for (i = 0; i < len; i++) {
        if (ser_asn1_write_byte(e, data[i]) != 0) return -1;
    }
    return 0;
}

size_t ser_asn1_size(ser_asn1_enc_t *e) {
    return e->pos;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1590 failed: {:?}", result.err());
}

// ============================================================================
// C1591-C1595: Streaming (chunked, framed, ring buffer, zero-copy, incremental)
// ============================================================================

/// C1591: Chunked transfer encoder/decoder -- splits data into sized chunks with
/// length headers, similar to HTTP chunked transfer encoding.
#[test]
fn c1591_chunked_transfer() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

typedef struct {
    uint8_t *buf;
    size_t pos;
    size_t cap;
    uint32_t chunk_size;
} ser_chunk_writer_t;

void ser_chunk_writer_init(ser_chunk_writer_t *w, uint8_t *buf, size_t cap, uint32_t chunk_size) {
    w->buf = buf;
    w->pos = 0;
    w->cap = cap;
    w->chunk_size = chunk_size;
}

static void ser_chunk_write_u32(ser_chunk_writer_t *w, uint32_t val) {
    w->buf[w->pos++] = (uint8_t)(val >> 24);
    w->buf[w->pos++] = (uint8_t)((val >> 16) & 0xFF);
    w->buf[w->pos++] = (uint8_t)((val >> 8) & 0xFF);
    w->buf[w->pos++] = (uint8_t)(val & 0xFF);
}

int ser_chunk_write(ser_chunk_writer_t *w, const uint8_t *data, size_t len) {
    size_t offset = 0;
    while (offset < len) {
        uint32_t remain = (uint32_t)(len - offset);
        uint32_t this_chunk = remain < w->chunk_size ? remain : w->chunk_size;
        if (w->pos + 4 + this_chunk > w->cap) return -1;
        ser_chunk_write_u32(w, this_chunk);
        size_t i;
        for (i = 0; i < this_chunk; i++) {
            w->buf[w->pos++] = data[offset + i];
        }
        offset += this_chunk;
    }
    return 0;
}

int ser_chunk_write_end(ser_chunk_writer_t *w) {
    if (w->pos + 4 > w->cap) return -1;
    ser_chunk_write_u32(w, 0);
    return 0;
}

typedef struct {
    const uint8_t *buf;
    size_t len;
    size_t pos;
} ser_chunk_reader_t;

void ser_chunk_reader_init(ser_chunk_reader_t *r, const uint8_t *buf, size_t len) {
    r->buf = buf;
    r->len = len;
    r->pos = 0;
}

int ser_chunk_read(ser_chunk_reader_t *r, const uint8_t **chunk_out, uint32_t *chunk_len) {
    if (r->pos + 4 > r->len) return -1;
    uint32_t clen = ((uint32_t)r->buf[r->pos] << 24) |
                    ((uint32_t)r->buf[r->pos + 1] << 16) |
                    ((uint32_t)r->buf[r->pos + 2] << 8) |
                    (uint32_t)r->buf[r->pos + 3];
    r->pos += 4;
    if (clen == 0) { *chunk_out = 0; *chunk_len = 0; return 1; }
    if (r->pos + clen > r->len) return -1;
    *chunk_out = &r->buf[r->pos];
    *chunk_len = clen;
    r->pos += clen;
    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1591 failed: {:?}", result.err());
}

/// C1592: Framed protocol -- wraps messages in frames with magic bytes, length,
/// and a simple XOR checksum for basic integrity.
#[test]
fn c1592_framed_protocol() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

typedef struct {
    uint8_t *buf;
    size_t pos;
    size_t cap;
} ser_frame_writer_t;

void ser_frame_writer_init(ser_frame_writer_t *w, uint8_t *buf, size_t cap) {
    w->buf = buf;
    w->pos = 0;
    w->cap = cap;
}

static uint8_t ser_frame_xor_checksum(const uint8_t *data, size_t len) {
    uint8_t cs = 0;
    size_t i;
    for (i = 0; i < len; i++) {
        cs ^= data[i];
    }
    return cs;
}

int ser_frame_write(ser_frame_writer_t *w, const uint8_t *msg, uint16_t msg_len) {
    size_t needed = 2 + 2 + msg_len + 1;
    if (w->pos + needed > w->cap) return -1;
    w->buf[w->pos++] = 0xAA;
    w->buf[w->pos++] = 0x55;
    w->buf[w->pos++] = (uint8_t)(msg_len >> 8);
    w->buf[w->pos++] = (uint8_t)(msg_len & 0xFF);
    size_t i;
    for (i = 0; i < msg_len; i++) {
        w->buf[w->pos++] = msg[i];
    }
    w->buf[w->pos++] = ser_frame_xor_checksum(msg, msg_len);
    return 0;
}

typedef struct {
    const uint8_t *data;
    uint16_t len;
    int valid;
} ser_frame_msg_t;

int ser_frame_read(const uint8_t *buf, size_t buf_len, size_t *offset, ser_frame_msg_t *msg) {
    if (*offset + 4 > buf_len) return -1;
    if (buf[*offset] != 0xAA || buf[*offset + 1] != 0x55) return -1;
    *offset += 2;
    uint16_t msg_len = (uint16_t)((buf[*offset] << 8) | buf[*offset + 1]);
    *offset += 2;
    if (*offset + msg_len + 1 > buf_len) return -1;
    msg->data = &buf[*offset];
    msg->len = msg_len;
    uint8_t expected_cs = ser_frame_xor_checksum(&buf[*offset], msg_len);
    *offset += msg_len;
    uint8_t actual_cs = buf[*offset];
    *offset += 1;
    msg->valid = (expected_cs == actual_cs) ? 1 : 0;
    return 0;
}

size_t ser_frame_writer_size(ser_frame_writer_t *w) {
    return w->pos;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1592 failed: {:?}", result.err());
}

/// C1593: Ring buffer serializer -- writes serialized data to a fixed-size ring
/// buffer with wrap-around, supporting continuous streaming.
#[test]
fn c1593_ring_buffer_serializer() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

typedef struct {
    uint8_t *buf;
    size_t cap;
    size_t head;
    size_t tail;
    size_t count;
} ser_ring_t;

void ser_ring_init(ser_ring_t *r, uint8_t *buf, size_t cap) {
    r->buf = buf;
    r->cap = cap;
    r->head = 0;
    r->tail = 0;
    r->count = 0;
}

int ser_ring_write(ser_ring_t *r, const uint8_t *data, size_t len) {
    if (r->count + len > r->cap) return -1;
    size_t i;
    for (i = 0; i < len; i++) {
        r->buf[r->head] = data[i];
        r->head = (r->head + 1) % r->cap;
    }
    r->count += len;
    return 0;
}

int ser_ring_read(ser_ring_t *r, uint8_t *out, size_t len) {
    if (len > r->count) return -1;
    size_t i;
    for (i = 0; i < len; i++) {
        out[i] = r->buf[r->tail];
        r->tail = (r->tail + 1) % r->cap;
    }
    r->count -= len;
    return 0;
}

size_t ser_ring_available(ser_ring_t *r) {
    return r->count;
}

size_t ser_ring_free_space(ser_ring_t *r) {
    return r->cap - r->count;
}

int ser_ring_write_framed(ser_ring_t *r, const uint8_t *msg, uint8_t msg_len) {
    if (r->count + 1 + msg_len > r->cap) return -1;
    if (ser_ring_write(r, &msg_len, 1) != 0) return -1;
    return ser_ring_write(r, msg, msg_len);
}

int ser_ring_read_framed(ser_ring_t *r, uint8_t *out, size_t out_cap, uint8_t *msg_len) {
    if (r->count < 1) return -1;
    uint8_t len;
    if (ser_ring_read(r, &len, 1) != 0) return -1;
    if (len > out_cap) return -1;
    if (ser_ring_read(r, out, len) != 0) return -1;
    *msg_len = len;
    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1593 failed: {:?}", result.err());
}

/// C1594: Zero-copy deserializer -- reads structured data directly from a buffer
/// without copying, returning pointers into the original buffer.
#[test]
fn c1594_zero_copy_deserializer() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;

typedef struct {
    const uint8_t *buf;
    size_t len;
    size_t pos;
} ser_zc_reader_t;

void ser_zc_reader_init(ser_zc_reader_t *r, const uint8_t *buf, size_t len) {
    r->buf = buf;
    r->len = len;
    r->pos = 0;
}

int ser_zc_read_u8(ser_zc_reader_t *r, uint8_t *val) {
    if (r->pos >= r->len) return -1;
    *val = r->buf[r->pos++];
    return 0;
}

int ser_zc_read_u16_be(ser_zc_reader_t *r, uint16_t *val) {
    if (r->pos + 2 > r->len) return -1;
    *val = (uint16_t)((r->buf[r->pos] << 8) | r->buf[r->pos + 1]);
    r->pos += 2;
    return 0;
}

int ser_zc_read_u32_be(ser_zc_reader_t *r, uint32_t *val) {
    if (r->pos + 4 > r->len) return -1;
    *val = ((uint32_t)r->buf[r->pos] << 24) |
           ((uint32_t)r->buf[r->pos + 1] << 16) |
           ((uint32_t)r->buf[r->pos + 2] << 8) |
           (uint32_t)r->buf[r->pos + 3];
    r->pos += 4;
    return 0;
}

int ser_zc_read_bytes(ser_zc_reader_t *r, const uint8_t **out, size_t count) {
    if (r->pos + count > r->len) return -1;
    *out = &r->buf[r->pos];
    r->pos += count;
    return 0;
}

int ser_zc_read_string(ser_zc_reader_t *r, const char **out, uint16_t *out_len) {
    uint16_t slen;
    if (ser_zc_read_u16_be(r, &slen) != 0) return -1;
    if (r->pos + slen > r->len) return -1;
    *out = (const char *)&r->buf[r->pos];
    *out_len = slen;
    r->pos += slen;
    return 0;
}

size_t ser_zc_remaining(ser_zc_reader_t *r) {
    return r->len - r->pos;
}

int ser_zc_skip(ser_zc_reader_t *r, size_t count) {
    if (r->pos + count > r->len) return -1;
    r->pos += count;
    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1594 failed: {:?}", result.err());
}

/// C1595: Incremental parser -- parses a stream of bytes incrementally,
/// maintaining parse state across calls for partial input handling.
#[test]
fn c1595_incremental_parser() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

typedef enum {
    SER_INC_WANT_HEADER,
    SER_INC_WANT_LENGTH,
    SER_INC_WANT_BODY,
    SER_INC_COMPLETE,
    SER_INC_ERROR
} ser_inc_state_t;

typedef struct {
    ser_inc_state_t state;
    uint8_t msg_type;
    uint32_t body_len;
    uint32_t body_received;
    uint8_t header_buf[4];
    int header_pos;
    uint8_t *body_buf;
    size_t body_cap;
} ser_inc_parser_t;

void ser_inc_init(ser_inc_parser_t *p, uint8_t *body_buf, size_t body_cap) {
    p->state = SER_INC_WANT_HEADER;
    p->msg_type = 0;
    p->body_len = 0;
    p->body_received = 0;
    p->header_pos = 0;
    p->body_buf = body_buf;
    p->body_cap = body_cap;
}

int ser_inc_feed(ser_inc_parser_t *p, const uint8_t *data, size_t len, size_t *consumed) {
    *consumed = 0;
    while (*consumed < len) {
        uint8_t b = data[*consumed];
        if (p->state == SER_INC_WANT_HEADER) {
            p->header_buf[p->header_pos++] = b;
            (*consumed)++;
            if (p->header_pos >= 2) {
                p->msg_type = p->header_buf[0];
                p->state = SER_INC_WANT_LENGTH;
            }
        } else if (p->state == SER_INC_WANT_LENGTH) {
            p->header_buf[p->header_pos++] = b;
            (*consumed)++;
            if (p->header_pos >= 4) {
                p->body_len = ((uint32_t)p->header_buf[2] << 8) | (uint32_t)p->header_buf[3];
                if (p->body_len > p->body_cap) {
                    p->state = SER_INC_ERROR;
                    return -1;
                }
                if (p->body_len == 0) {
                    p->state = SER_INC_COMPLETE;
                    return 1;
                }
                p->body_received = 0;
                p->state = SER_INC_WANT_BODY;
            }
        } else if (p->state == SER_INC_WANT_BODY) {
            p->body_buf[p->body_received++] = b;
            (*consumed)++;
            if (p->body_received >= p->body_len) {
                p->state = SER_INC_COMPLETE;
                return 1;
            }
        } else {
            return -1;
        }
    }
    return 0;
}

void ser_inc_reset(ser_inc_parser_t *p) {
    p->state = SER_INC_WANT_HEADER;
    p->header_pos = 0;
    p->body_received = 0;
}

int ser_inc_is_complete(ser_inc_parser_t *p) {
    return p->state == SER_INC_COMPLETE ? 1 : 0;
}

uint8_t ser_inc_msg_type(ser_inc_parser_t *p) {
    return p->msg_type;
}

uint32_t ser_inc_body_len(ser_inc_parser_t *p) {
    return p->body_len;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1595 failed: {:?}", result.err());
}

// ============================================================================
// C1596-C1600: Advanced (schema evolution, backward compat, checksum, compression, batch)
// ============================================================================

/// C1596: Schema evolution / versioning -- encodes messages with a version field
/// and supports reading older versions by applying default values for missing fields.
#[test]
fn c1596_schema_evolution() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

typedef struct {
    uint8_t *buf;
    size_t pos;
    size_t cap;
} ser_evo_writer_t;

void ser_evo_writer_init(ser_evo_writer_t *w, uint8_t *buf, size_t cap) {
    w->buf = buf;
    w->pos = 0;
    w->cap = cap;
}

int ser_evo_write_header(ser_evo_writer_t *w, uint16_t version, uint16_t field_count) {
    if (w->pos + 4 > w->cap) return -1;
    w->buf[w->pos++] = (uint8_t)(version >> 8);
    w->buf[w->pos++] = (uint8_t)(version & 0xFF);
    w->buf[w->pos++] = (uint8_t)(field_count >> 8);
    w->buf[w->pos++] = (uint8_t)(field_count & 0xFF);
    return 0;
}

int ser_evo_write_field(ser_evo_writer_t *w, uint16_t field_id, const uint8_t *data, uint16_t len) {
    if (w->pos + 4 + len > w->cap) return -1;
    w->buf[w->pos++] = (uint8_t)(field_id >> 8);
    w->buf[w->pos++] = (uint8_t)(field_id & 0xFF);
    w->buf[w->pos++] = (uint8_t)(len >> 8);
    w->buf[w->pos++] = (uint8_t)(len & 0xFF);
    uint16_t i;
    for (i = 0; i < len; i++) {
        w->buf[w->pos++] = data[i];
    }
    return 0;
}

typedef struct {
    const uint8_t *buf;
    size_t len;
    size_t pos;
    uint16_t version;
    uint16_t field_count;
    uint16_t fields_read;
} ser_evo_reader_t;

int ser_evo_reader_init(ser_evo_reader_t *r, const uint8_t *buf, size_t len) {
    r->buf = buf;
    r->len = len;
    r->pos = 0;
    r->fields_read = 0;
    if (len < 4) return -1;
    r->version = (uint16_t)((buf[0] << 8) | buf[1]);
    r->field_count = (uint16_t)((buf[2] << 8) | buf[3]);
    r->pos = 4;
    return 0;
}

int ser_evo_read_field(ser_evo_reader_t *r, uint16_t *field_id, const uint8_t **data, uint16_t *data_len) {
    if (r->fields_read >= r->field_count) return -1;
    if (r->pos + 4 > r->len) return -1;
    *field_id = (uint16_t)((r->buf[r->pos] << 8) | r->buf[r->pos + 1]);
    *data_len = (uint16_t)((r->buf[r->pos + 2] << 8) | r->buf[r->pos + 3]);
    r->pos += 4;
    if (r->pos + *data_len > r->len) return -1;
    *data = &r->buf[r->pos];
    r->pos += *data_len;
    r->fields_read++;
    return 0;
}

int ser_evo_skip_unknown(ser_evo_reader_t *r) {
    while (r->fields_read < r->field_count) {
        uint16_t fid;
        const uint8_t *fdata;
        uint16_t flen;
        if (ser_evo_read_field(r, &fid, &fdata, &flen) != 0) return -1;
    }
    return 0;
}

size_t ser_evo_writer_size(ser_evo_writer_t *w) {
    return w->pos;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1596 failed: {:?}", result.err());
}

/// C1597: Backward-compatible reader -- reads messages encoded with newer schemas
/// by skipping unrecognized fields and providing defaults for known fields.
#[test]
fn c1597_backward_compatible_reader() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

typedef struct {
    uint16_t id;
    uint16_t len;
    const uint8_t *data;
} ser_bc_field_t;

typedef struct {
    const uint8_t *buf;
    size_t len;
    size_t pos;
    uint16_t version;
    uint16_t num_fields;
} ser_bc_reader_t;

int ser_bc_reader_init(ser_bc_reader_t *r, const uint8_t *buf, size_t len) {
    if (len < 4) return -1;
    r->buf = buf;
    r->len = len;
    r->version = (uint16_t)((buf[0] << 8) | buf[1]);
    r->num_fields = (uint16_t)((buf[2] << 8) | buf[3]);
    r->pos = 4;
    return 0;
}

int ser_bc_find_field(ser_bc_reader_t *r, uint16_t target_id, ser_bc_field_t *out) {
    size_t saved_pos = r->pos;
    r->pos = 4;
    uint16_t i;
    for (i = 0; i < r->num_fields; i++) {
        if (r->pos + 4 > r->len) { r->pos = saved_pos; return -1; }
        uint16_t fid = (uint16_t)((r->buf[r->pos] << 8) | r->buf[r->pos + 1]);
        uint16_t flen = (uint16_t)((r->buf[r->pos + 2] << 8) | r->buf[r->pos + 3]);
        r->pos += 4;
        if (r->pos + flen > r->len) { r->pos = saved_pos; return -1; }
        if (fid == target_id) {
            out->id = fid;
            out->len = flen;
            out->data = &r->buf[r->pos];
            r->pos = saved_pos;
            return 0;
        }
        r->pos += flen;
    }
    r->pos = saved_pos;
    return -1;
}

uint32_t ser_bc_read_u32_or_default(ser_bc_reader_t *r, uint16_t field_id, uint32_t default_val) {
    ser_bc_field_t f;
    if (ser_bc_find_field(r, field_id, &f) != 0 || f.len < 4) {
        return default_val;
    }
    return ((uint32_t)f.data[0] << 24) |
           ((uint32_t)f.data[1] << 16) |
           ((uint32_t)f.data[2] << 8) |
           (uint32_t)f.data[3];
}

uint16_t ser_bc_version(ser_bc_reader_t *r) {
    return r->version;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1597 failed: {:?}", result.err());
}

/// C1598: Checksum-verified messages -- wraps serialized messages with a CRC32
/// checksum for integrity verification on deserialization.
#[test]
fn c1598_checksum_verified_messages() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

static uint32_t ser_crc32_table[256];
static int ser_crc32_table_init = 0;

void ser_crc32_init_table(void) {
    if (ser_crc32_table_init) return;
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
        ser_crc32_table[i] = crc;
    }
    ser_crc32_table_init = 1;
}

uint32_t ser_crc32_compute(const uint8_t *data, size_t len) {
    ser_crc32_init_table();
    uint32_t crc = 0xFFFFFFFF;
    size_t i;
    for (i = 0; i < len; i++) {
        uint8_t idx = (uint8_t)((crc ^ data[i]) & 0xFF);
        crc = (crc >> 8) ^ ser_crc32_table[idx];
    }
    return crc ^ 0xFFFFFFFF;
}

typedef struct {
    uint8_t *buf;
    size_t pos;
    size_t cap;
} ser_crc_writer_t;

void ser_crc_writer_init(ser_crc_writer_t *w, uint8_t *buf, size_t cap) {
    w->buf = buf;
    w->pos = 0;
    w->cap = cap;
}

int ser_crc_write_message(ser_crc_writer_t *w, const uint8_t *msg, uint32_t msg_len) {
    if (w->pos + 4 + msg_len + 4 > w->cap) return -1;
    w->buf[w->pos++] = (uint8_t)(msg_len >> 24);
    w->buf[w->pos++] = (uint8_t)((msg_len >> 16) & 0xFF);
    w->buf[w->pos++] = (uint8_t)((msg_len >> 8) & 0xFF);
    w->buf[w->pos++] = (uint8_t)(msg_len & 0xFF);
    uint32_t i;
    for (i = 0; i < msg_len; i++) {
        w->buf[w->pos++] = msg[i];
    }
    uint32_t crc = ser_crc32_compute(msg, msg_len);
    w->buf[w->pos++] = (uint8_t)(crc >> 24);
    w->buf[w->pos++] = (uint8_t)((crc >> 16) & 0xFF);
    w->buf[w->pos++] = (uint8_t)((crc >> 8) & 0xFF);
    w->buf[w->pos++] = (uint8_t)(crc & 0xFF);
    return 0;
}

int ser_crc_verify_message(const uint8_t *buf, size_t buf_len, const uint8_t **msg_out, uint32_t *msg_len_out) {
    if (buf_len < 8) return -1;
    uint32_t msg_len = ((uint32_t)buf[0] << 24) |
                       ((uint32_t)buf[1] << 16) |
                       ((uint32_t)buf[2] << 8) |
                       (uint32_t)buf[3];
    if (4 + msg_len + 4 > buf_len) return -1;
    const uint8_t *msg = &buf[4];
    uint32_t stored_crc = ((uint32_t)buf[4 + msg_len] << 24) |
                          ((uint32_t)buf[4 + msg_len + 1] << 16) |
                          ((uint32_t)buf[4 + msg_len + 2] << 8) |
                          (uint32_t)buf[4 + msg_len + 3];
    uint32_t computed_crc = ser_crc32_compute(msg, msg_len);
    if (stored_crc != computed_crc) return -2;
    *msg_out = msg;
    *msg_len_out = msg_len;
    return 0;
}

size_t ser_crc_writer_size(ser_crc_writer_t *w) {
    return w->pos;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1598 failed: {:?}", result.err());
}

/// C1599: Compression-wrapped encoding -- applies simple RLE compression around
/// serialized data with a header indicating compressed vs uncompressed.
#[test]
fn c1599_compression_wrapped_encoding() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

typedef struct {
    uint8_t *buf;
    size_t pos;
    size_t cap;
} ser_comp_writer_t;

void ser_comp_writer_init(ser_comp_writer_t *w, uint8_t *buf, size_t cap) {
    w->buf = buf;
    w->pos = 0;
    w->cap = cap;
}

static size_t ser_comp_rle_compress(const uint8_t *src, size_t src_len, uint8_t *dst, size_t dst_cap) {
    size_t sp = 0;
    size_t dp = 0;
    while (sp < src_len) {
        uint8_t val = src[sp];
        size_t run = 1;
        while (sp + run < src_len && src[sp + run] == val && run < 255) {
            run++;
        }
        if (dp + 2 > dst_cap) return 0;
        dst[dp++] = (uint8_t)run;
        dst[dp++] = val;
        sp += run;
    }
    return dp;
}

int ser_comp_write(ser_comp_writer_t *w, const uint8_t *data, uint32_t len) {
    uint8_t compressed[1024];
    size_t comp_len = ser_comp_rle_compress(data, len, compressed, 1024);
    int use_compressed = (comp_len > 0 && comp_len < len) ? 1 : 0;

    if (w->pos + 1 + 4 > w->cap) return -1;
    w->buf[w->pos++] = use_compressed ? 1 : 0;
    uint32_t write_len = use_compressed ? (uint32_t)comp_len : len;
    w->buf[w->pos++] = (uint8_t)(len >> 24);
    w->buf[w->pos++] = (uint8_t)((len >> 16) & 0xFF);
    w->buf[w->pos++] = (uint8_t)((len >> 8) & 0xFF);
    w->buf[w->pos++] = (uint8_t)(len & 0xFF);

    if (w->pos + 4 + write_len > w->cap) return -1;
    w->buf[w->pos++] = (uint8_t)(write_len >> 24);
    w->buf[w->pos++] = (uint8_t)((write_len >> 16) & 0xFF);
    w->buf[w->pos++] = (uint8_t)((write_len >> 8) & 0xFF);
    w->buf[w->pos++] = (uint8_t)(write_len & 0xFF);

    const uint8_t *src = use_compressed ? compressed : data;
    uint32_t i;
    for (i = 0; i < write_len; i++) {
        w->buf[w->pos++] = src[i];
    }
    return 0;
}

size_t ser_comp_writer_size(ser_comp_writer_t *w) {
    return w->pos;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1599 failed: {:?}", result.err());
}

/// C1600: Batch serializer -- collects multiple records into a batch with a
/// batch header (record count, total size) and per-record offsets for random access.
#[test]
fn c1600_batch_serializer() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

typedef struct {
    uint8_t *buf;
    size_t pos;
    size_t cap;
    uint32_t offsets[256];
    uint32_t record_count;
    size_t header_pos;
} ser_batch_t;

void ser_batch_init(ser_batch_t *b, uint8_t *buf, size_t cap) {
    b->buf = buf;
    b->cap = cap;
    b->record_count = 0;
    b->pos = 8;
    b->header_pos = 0;
}

int ser_batch_add_record(ser_batch_t *b, const uint8_t *data, uint32_t len) {
    if (b->record_count >= 256) return -1;
    if (b->pos + 4 + len > b->cap) return -1;
    b->offsets[b->record_count] = (uint32_t)b->pos;
    b->record_count++;
    b->buf[b->pos++] = (uint8_t)(len >> 24);
    b->buf[b->pos++] = (uint8_t)((len >> 16) & 0xFF);
    b->buf[b->pos++] = (uint8_t)((len >> 8) & 0xFF);
    b->buf[b->pos++] = (uint8_t)(len & 0xFF);
    uint32_t i;
    for (i = 0; i < len; i++) {
        b->buf[b->pos++] = data[i];
    }
    return 0;
}

void ser_batch_finalize(ser_batch_t *b) {
    b->buf[0] = (uint8_t)(b->record_count >> 24);
    b->buf[1] = (uint8_t)((b->record_count >> 16) & 0xFF);
    b->buf[2] = (uint8_t)((b->record_count >> 8) & 0xFF);
    b->buf[3] = (uint8_t)(b->record_count & 0xFF);
    uint32_t total = (uint32_t)b->pos;
    b->buf[4] = (uint8_t)(total >> 24);
    b->buf[5] = (uint8_t)((total >> 16) & 0xFF);
    b->buf[6] = (uint8_t)((total >> 8) & 0xFF);
    b->buf[7] = (uint8_t)(total & 0xFF);
}

int ser_batch_read_record(const uint8_t *buf, size_t buf_len, uint32_t index,
                          const uint8_t **data_out, uint32_t *len_out) {
    if (buf_len < 8) return -1;
    uint32_t count = ((uint32_t)buf[0] << 24) | ((uint32_t)buf[1] << 16) |
                     ((uint32_t)buf[2] << 8) | (uint32_t)buf[3];
    if (index >= count) return -1;
    size_t pos = 8;
    uint32_t i;
    for (i = 0; i < index; i++) {
        if (pos + 4 > buf_len) return -1;
        uint32_t rlen = ((uint32_t)buf[pos] << 24) | ((uint32_t)buf[pos + 1] << 16) |
                        ((uint32_t)buf[pos + 2] << 8) | (uint32_t)buf[pos + 3];
        pos += 4 + rlen;
    }
    if (pos + 4 > buf_len) return -1;
    uint32_t rlen = ((uint32_t)buf[pos] << 24) | ((uint32_t)buf[pos + 1] << 16) |
                    ((uint32_t)buf[pos + 2] << 8) | (uint32_t)buf[pos + 3];
    pos += 4;
    if (pos + rlen > buf_len) return -1;
    *data_out = &buf[pos];
    *len_out = rlen;
    return 0;
}

uint32_t ser_batch_count(ser_batch_t *b) {
    return b->record_count;
}

size_t ser_batch_size(ser_batch_t *b) {
    return b->pos;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1600 failed: {:?}", result.err());
}
