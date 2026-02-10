//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1101-C1125: Protocol Parser Implementations
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise protocol parsing and decoding patterns commonly found
//! in networking C code: HTTP parsing, DNS protocol handling, binary protocol
//! decoders, network header parsers, and application protocol processors.
//! All tests use self-contained C99 code with no #include directives.
//!
//! Organization:
//! - C1101-C1105: HTTP parsing (request line, headers, chunked transfer, URL, cookies)
//! - C1106-C1110: DNS protocol (query parser, response builder, name compression, record type, zone file)
//! - C1111-C1115: Binary protocols (TLV parser, protobuf varint, MQTT packet, Redis RESP, MessagePack)
//! - C1116-C1120: Network headers (IPv4, TCP, UDP checksum, ARP, ICMP)
//! - C1121-C1125: Application protocols (SMTP command, FTP response, SIP header, RTSP request, WebSocket frame)

use decy_core::transpile;

// ============================================================================
// C1101-C1105: HTTP Parsing
// ============================================================================

/// C1101: HTTP request line parser - extracts method, URI, and version from request line
#[test]
fn c1101_http_request_line_parser() {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    char method[16];
    char uri[256];
    int version_major;
    int version_minor;
    int valid;
} proto_http_request_line_t;

int proto_http_parse_request_line(const char *buf, int len, proto_http_request_line_t *req) {
    int i = 0;
    int j = 0;

    req->valid = 0;
    req->version_major = 0;
    req->version_minor = 0;

    /* Parse method (GET, POST, PUT, DELETE, etc.) */
    while (i < len && buf[i] != ' ' && j < 15) {
        req->method[j++] = buf[i++];
    }
    req->method[j] = '\0';

    if (i >= len || buf[i] != ' ') return -1;
    i++; /* skip space */

    /* Parse URI */
    j = 0;
    while (i < len && buf[i] != ' ' && j < 255) {
        req->uri[j++] = buf[i++];
    }
    req->uri[j] = '\0';

    if (i >= len || buf[i] != ' ') return -2;
    i++; /* skip space */

    /* Parse HTTP version: HTTP/x.y */
    if (i + 4 >= len) return -3;
    if (buf[i] != 'H' || buf[i+1] != 'T' || buf[i+2] != 'T' || buf[i+3] != 'P' || buf[i+4] != '/') return -3;
    i += 5;

    if (i < len && buf[i] >= '0' && buf[i] <= '9') {
        req->version_major = buf[i] - '0';
        i++;
    }
    if (i < len && buf[i] == '.') i++;
    if (i < len && buf[i] >= '0' && buf[i] <= '9') {
        req->version_minor = buf[i] - '0';
    }

    req->valid = 1;
    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1101 HTTP request line parser failed: {:?}",
        result.err()
    );
}

/// C1102: HTTP header parser - parses key-value header lines
#[test]
fn c1102_http_header_parser() {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    char name[64];
    char value[256];
} proto_http_header_t;

typedef struct {
    proto_http_header_t headers[32];
    int count;
    int content_length;
    int chunked;
} proto_http_headers_t;

static int proto_http_streq_nocase(const char *a, const char *b) {
    while (*a && *b) {
        char ca = *a;
        char cb = *b;
        if (ca >= 'A' && ca <= 'Z') ca += 32;
        if (cb >= 'A' && cb <= 'Z') cb += 32;
        if (ca != cb) return 0;
        a++;
        b++;
    }
    return (*a == '\0' && *b == '\0');
}

int proto_http_parse_header_line(const char *line, int len, proto_http_headers_t *hdrs) {
    int i = 0;
    int j = 0;

    if (hdrs->count >= 32) return -1;

    proto_http_header_t *h = &hdrs->headers[hdrs->count];

    /* Parse header name up to colon */
    while (i < len && line[i] != ':' && j < 63) {
        h->name[j++] = line[i++];
    }
    h->name[j] = '\0';

    if (i >= len || line[i] != ':') return -2;
    i++; /* skip colon */

    /* Skip optional whitespace */
    while (i < len && (line[i] == ' ' || line[i] == '\t')) i++;

    /* Parse value */
    j = 0;
    while (i < len && line[i] != '\r' && line[i] != '\n' && j < 255) {
        h->value[j++] = line[i++];
    }
    h->value[j] = '\0';

    /* Check special headers */
    if (proto_http_streq_nocase(h->name, "content-length")) {
        int val = 0;
        const char *p = h->value;
        while (*p >= '0' && *p <= '9') {
            val = val * 10 + (*p - '0');
            p++;
        }
        hdrs->content_length = val;
    }

    hdrs->count++;
    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1102 HTTP header parser failed: {:?}",
        result.err()
    );
}

/// C1103: HTTP chunked transfer decoder - decodes chunked encoding body
#[test]
fn c1103_http_chunked_transfer_decoder() {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    int state;
    int chunk_remaining;
    int total_decoded;
    int done;
} proto_http_chunked_t;

void proto_http_chunked_init(proto_http_chunked_t *ctx) {
    ctx->state = 0;
    ctx->chunk_remaining = 0;
    ctx->total_decoded = 0;
    ctx->done = 0;
}

static int proto_hex_digit(char c) {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'f') return c - 'a' + 10;
    if (c >= 'A' && c <= 'F') return c - 'A' + 10;
    return -1;
}

int proto_http_chunked_decode(proto_http_chunked_t *ctx, const char *input,
                               int input_len, char *output, int output_cap) {
    int in_pos = 0;
    int out_pos = 0;

    while (in_pos < input_len && !ctx->done) {
        if (ctx->state == 0) {
            /* Reading chunk size (hex) */
            int hex_val = proto_hex_digit(input[in_pos]);
            if (hex_val >= 0) {
                ctx->chunk_remaining = ctx->chunk_remaining * 16 + hex_val;
                in_pos++;
            } else if (input[in_pos] == '\r') {
                in_pos++;
            } else if (input[in_pos] == '\n') {
                in_pos++;
                if (ctx->chunk_remaining == 0) {
                    ctx->done = 1;
                } else {
                    ctx->state = 1;
                }
            } else {
                return -1;
            }
        } else if (ctx->state == 1) {
            /* Reading chunk data */
            int to_copy = ctx->chunk_remaining;
            if (to_copy > input_len - in_pos) to_copy = input_len - in_pos;
            if (to_copy > output_cap - out_pos) to_copy = output_cap - out_pos;

            int k;
            for (k = 0; k < to_copy; k++) {
                output[out_pos++] = input[in_pos++];
            }
            ctx->chunk_remaining -= to_copy;
            ctx->total_decoded += to_copy;

            if (ctx->chunk_remaining == 0) {
                ctx->state = 2;
            }
        } else if (ctx->state == 2) {
            /* Expecting CRLF after chunk data */
            if (input[in_pos] == '\r' || input[in_pos] == '\n') {
                in_pos++;
                if (input[in_pos - 1] == '\n') {
                    ctx->state = 0;
                }
            } else {
                return -2;
            }
        }
    }

    return out_pos;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1103 HTTP chunked transfer decoder failed: {:?}",
        result.err()
    );
}

/// C1104: URL parser - splits URL into scheme, host, port, path components
#[test]
fn c1104_url_parser() {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    char scheme[16];
    char host[128];
    int port;
    char path[256];
    char query[256];
    int valid;
} proto_url_parts_t;

int proto_url_parse(const char *url, int url_len, proto_url_parts_t *parts) {
    int i = 0;
    int j = 0;

    parts->valid = 0;
    parts->port = 0;
    parts->scheme[0] = '\0';
    parts->host[0] = '\0';
    parts->path[0] = '\0';
    parts->query[0] = '\0';

    /* Parse scheme */
    while (i < url_len && url[i] != ':' && j < 15) {
        parts->scheme[j++] = url[i++];
    }
    parts->scheme[j] = '\0';

    if (i + 2 >= url_len || url[i] != ':' || url[i+1] != '/' || url[i+2] != '/') {
        return -1;
    }
    i += 3; /* skip :// */

    /* Parse host */
    j = 0;
    while (i < url_len && url[i] != ':' && url[i] != '/' && url[i] != '?' && j < 127) {
        parts->host[j++] = url[i++];
    }
    parts->host[j] = '\0';

    /* Parse optional port */
    if (i < url_len && url[i] == ':') {
        i++;
        int port = 0;
        while (i < url_len && url[i] >= '0' && url[i] <= '9') {
            port = port * 10 + (url[i] - '0');
            i++;
        }
        parts->port = port;
    }

    /* Parse path */
    j = 0;
    if (i < url_len && url[i] == '/') {
        while (i < url_len && url[i] != '?' && url[i] != '#' && j < 255) {
            parts->path[j++] = url[i++];
        }
    }
    parts->path[j] = '\0';

    /* Parse query string */
    j = 0;
    if (i < url_len && url[i] == '?') {
        i++;
        while (i < url_len && url[i] != '#' && j < 255) {
            parts->query[j++] = url[i++];
        }
    }
    parts->query[j] = '\0';

    parts->valid = 1;
    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1104 URL parser failed: {:?}",
        result.err()
    );
}

/// C1105: HTTP cookie parser - parses Set-Cookie header value into structured fields
#[test]
fn c1105_http_cookie_parser() {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    char name[64];
    char value[128];
    char domain[64];
    char path[64];
    int max_age;
    int secure;
    int http_only;
} proto_cookie_t;

static void proto_cookie_trim_leading(const char *src, char *dst, int dst_cap) {
    while (*src == ' ' || *src == '\t') src++;
    int j = 0;
    while (*src && *src != ';' && j < dst_cap - 1) {
        dst[j++] = *src++;
    }
    dst[j] = '\0';
}

int proto_cookie_parse(const char *header, int len, proto_cookie_t *cookie) {
    int i = 0;
    int j = 0;

    cookie->max_age = -1;
    cookie->secure = 0;
    cookie->http_only = 0;
    cookie->domain[0] = '\0';
    cookie->path[0] = '\0';

    /* Parse cookie name */
    while (i < len && header[i] != '=' && j < 63) {
        cookie->name[j++] = header[i++];
    }
    cookie->name[j] = '\0';

    if (i >= len || header[i] != '=') return -1;
    i++;

    /* Parse cookie value */
    j = 0;
    while (i < len && header[i] != ';' && j < 127) {
        cookie->value[j++] = header[i++];
    }
    cookie->value[j] = '\0';

    /* Parse attributes */
    while (i < len) {
        if (header[i] == ';') i++;
        while (i < len && header[i] == ' ') i++;

        char attr[32];
        j = 0;
        while (i < len && header[i] != '=' && header[i] != ';' && j < 31) {
            attr[j++] = header[i++];
        }
        attr[j] = '\0';

        if (attr[0] == 'S' && attr[1] == 'e' && attr[2] == 'c') {
            cookie->secure = 1;
        } else if (attr[0] == 'H' && attr[1] == 't' && attr[2] == 't') {
            cookie->http_only = 1;
        } else if (i < len && header[i] == '=') {
            i++;
            if (attr[0] == 'M' && attr[1] == 'a' && attr[2] == 'x') {
                int val = 0;
                while (i < len && header[i] >= '0' && header[i] <= '9') {
                    val = val * 10 + (header[i] - '0');
                    i++;
                }
                cookie->max_age = val;
            } else if (attr[0] == 'D' && attr[1] == 'o' && attr[2] == 'm') {
                j = 0;
                while (i < len && header[i] != ';' && j < 63) {
                    cookie->domain[j++] = header[i++];
                }
                cookie->domain[j] = '\0';
            } else if (attr[0] == 'P' && attr[1] == 'a' && attr[2] == 't') {
                j = 0;
                while (i < len && header[i] != ';' && j < 63) {
                    cookie->path[j++] = header[i++];
                }
                cookie->path[j] = '\0';
            } else {
                while (i < len && header[i] != ';') i++;
            }
        }
    }

    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1105 HTTP cookie parser failed: {:?}",
        result.err()
    );
}

// ============================================================================
// C1106-C1110: DNS Protocol
// ============================================================================

/// C1106: DNS query parser - parses DNS question section from wire format
#[test]
fn c1106_dns_query_parser() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

typedef struct {
    char name[256];
    uint16_t qtype;
    uint16_t qclass;
    int name_len;
} proto_dns_question_t;

int proto_dns_parse_name(const uint8_t *buf, int buf_len, int offset,
                          char *name, int name_cap) {
    int pos = offset;
    int out = 0;
    int jumps = 0;
    int first_label = 1;

    while (pos < buf_len && jumps < 10) {
        uint8_t label_len = buf[pos];

        if (label_len == 0) {
            pos++;
            break;
        }

        /* Check for compression pointer */
        if ((label_len & 0xC0) == 0xC0) {
            if (pos + 1 >= buf_len) return -1;
            int ptr = ((label_len & 0x3F) << 8) | buf[pos + 1];
            if (jumps == 0) pos += 2;
            pos = ptr;
            jumps++;
            continue;
        }

        if (!first_label && out < name_cap - 1) {
            name[out++] = '.';
        }
        first_label = 0;

        pos++;
        int k;
        for (k = 0; k < label_len && pos < buf_len && out < name_cap - 1; k++) {
            name[out++] = buf[pos++];
        }
    }

    name[out] = '\0';
    return pos;
}

int proto_dns_parse_question(const uint8_t *buf, int buf_len, int offset,
                              proto_dns_question_t *q) {
    int pos = proto_dns_parse_name(buf, buf_len, offset, q->name, 256);
    if (pos < 0) return -1;

    if (pos + 4 > buf_len) return -2;

    q->qtype = (buf[pos] << 8) | buf[pos + 1];
    q->qclass = (buf[pos + 2] << 8) | buf[pos + 3];
    q->name_len = pos - offset;

    return pos + 4;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1106 DNS query parser failed: {:?}",
        result.err()
    );
}

/// C1107: DNS response builder - constructs DNS response packets
#[test]
fn c1107_dns_response_builder() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

typedef struct {
    uint8_t buf[512];
    int pos;
    int question_count;
    int answer_count;
} proto_dns_response_t;

void proto_dns_resp_init(proto_dns_response_t *resp, uint16_t id, uint16_t flags) {
    int i;
    for (i = 0; i < 512; i++) resp->buf[i] = 0;

    resp->buf[0] = (id >> 8) & 0xFF;
    resp->buf[1] = id & 0xFF;
    resp->buf[2] = (flags >> 8) & 0xFF;
    resp->buf[3] = flags & 0xFF;
    resp->pos = 12;
    resp->question_count = 0;
    resp->answer_count = 0;
}

static void proto_dns_write_u16(uint8_t *buf, int pos, uint16_t val) {
    buf[pos] = (val >> 8) & 0xFF;
    buf[pos + 1] = val & 0xFF;
}

static void proto_dns_write_u32(uint8_t *buf, int pos, uint32_t val) {
    buf[pos] = (val >> 24) & 0xFF;
    buf[pos + 1] = (val >> 16) & 0xFF;
    buf[pos + 2] = (val >> 8) & 0xFF;
    buf[pos + 3] = val & 0xFF;
}

int proto_dns_resp_add_name(proto_dns_response_t *resp, const char *name) {
    const char *p = name;
    while (*p) {
        const char *dot = p;
        while (*dot && *dot != '.') dot++;
        int label_len = dot - p;
        if (label_len > 63 || resp->pos + label_len + 1 > 510) return -1;
        resp->buf[resp->pos++] = (uint8_t)label_len;
        int k;
        for (k = 0; k < label_len; k++) {
            resp->buf[resp->pos++] = p[k];
        }
        p = dot;
        if (*p == '.') p++;
    }
    resp->buf[resp->pos++] = 0;
    return 0;
}

int proto_dns_resp_add_a_record(proto_dns_response_t *resp, const char *name,
                                 uint32_t ttl, uint32_t ipv4_addr) {
    if (proto_dns_resp_add_name(resp, name) < 0) return -1;
    if (resp->pos + 14 > 512) return -2;

    proto_dns_write_u16(resp->buf, resp->pos, 1);     /* type A */
    resp->pos += 2;
    proto_dns_write_u16(resp->buf, resp->pos, 1);     /* class IN */
    resp->pos += 2;
    proto_dns_write_u32(resp->buf, resp->pos, ttl);
    resp->pos += 4;
    proto_dns_write_u16(resp->buf, resp->pos, 4);     /* rdlength */
    resp->pos += 2;
    proto_dns_write_u32(resp->buf, resp->pos, ipv4_addr);
    resp->pos += 4;

    resp->answer_count++;
    proto_dns_write_u16(resp->buf, 6, resp->answer_count);
    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1107 DNS response builder failed: {:?}",
        result.err()
    );
}

/// C1108: DNS name compression - implements RFC 1035 message compression
#[test]
fn c1108_dns_name_compression() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

typedef struct {
    char name[64];
    int offset;
} proto_dns_comp_entry_t;

typedef struct {
    proto_dns_comp_entry_t entries[32];
    int count;
} proto_dns_comp_table_t;

void proto_dns_comp_init(proto_dns_comp_table_t *table) {
    table->count = 0;
}

static int proto_dns_comp_streq(const char *a, const char *b) {
    while (*a && *b) {
        char ca = *a;
        char cb = *b;
        if (ca >= 'A' && ca <= 'Z') ca += 32;
        if (cb >= 'A' && cb <= 'Z') cb += 32;
        if (ca != cb) return 0;
        a++;
        b++;
    }
    return (*a == '\0' && *b == '\0');
}

int proto_dns_comp_find(proto_dns_comp_table_t *table, const char *name) {
    int i;
    for (i = 0; i < table->count; i++) {
        if (proto_dns_comp_streq(table->entries[i].name, name)) {
            return table->entries[i].offset;
        }
    }
    return -1;
}

int proto_dns_comp_write_name(proto_dns_comp_table_t *table, uint8_t *buf,
                               int pos, const char *name) {
    int ptr = proto_dns_comp_find(table, name);
    if (ptr >= 0 && ptr < 16384) {
        buf[pos] = 0xC0 | ((ptr >> 8) & 0x3F);
        buf[pos + 1] = ptr & 0xFF;
        return pos + 2;
    }

    /* Record this name for future compression */
    if (table->count < 32) {
        proto_dns_comp_entry_t *e = &table->entries[table->count];
        int j;
        for (j = 0; name[j] && j < 63; j++) {
            e->name[j] = name[j];
        }
        e->name[j] = '\0';
        e->offset = pos;
        table->count++;
    }

    /* Write labels */
    const char *p = name;
    while (*p) {
        const char *dot = p;
        while (*dot && *dot != '.') dot++;
        int label_len = dot - p;
        buf[pos++] = (uint8_t)label_len;
        int k;
        for (k = 0; k < label_len; k++) {
            buf[pos++] = p[k];
        }
        p = dot;
        if (*p == '.') p++;
    }
    buf[pos++] = 0;
    return pos;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1108 DNS name compression failed: {:?}",
        result.err()
    );
}

/// C1109: DNS record type decoder - decodes various DNS record types from wire format
#[test]
fn c1109_dns_record_type_decoder() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

typedef struct {
    uint16_t rtype;
    uint16_t rclass;
    uint32_t ttl;
    uint16_t rdlength;
    uint32_t a_addr;
    uint16_t mx_priority;
    char rdata_text[256];
} proto_dns_record_t;

int proto_dns_decode_rdata(const uint8_t *buf, int buf_len, int offset,
                            proto_dns_record_t *rec) {
    int pos = offset;

    if (pos + 10 > buf_len) return -1;

    rec->rtype = (buf[pos] << 8) | buf[pos + 1];
    rec->rclass = (buf[pos + 2] << 8) | buf[pos + 3];
    rec->ttl = ((uint32_t)buf[pos + 4] << 24) | ((uint32_t)buf[pos + 5] << 16) |
               ((uint32_t)buf[pos + 6] << 8) | buf[pos + 7];
    rec->rdlength = (buf[pos + 8] << 8) | buf[pos + 9];
    pos += 10;

    if (pos + rec->rdlength > buf_len) return -2;

    rec->a_addr = 0;
    rec->mx_priority = 0;
    rec->rdata_text[0] = '\0';

    if (rec->rtype == 1 && rec->rdlength == 4) {
        /* A record - IPv4 address */
        rec->a_addr = ((uint32_t)buf[pos] << 24) | ((uint32_t)buf[pos + 1] << 16) |
                      ((uint32_t)buf[pos + 2] << 8) | buf[pos + 3];
    } else if (rec->rtype == 16) {
        /* TXT record */
        int text_pos = 0;
        int rd_end = pos + rec->rdlength;
        while (pos < rd_end) {
            uint8_t txt_len = buf[pos++];
            int k;
            for (k = 0; k < txt_len && pos < rd_end && text_pos < 255; k++) {
                rec->rdata_text[text_pos++] = buf[pos++];
            }
        }
        rec->rdata_text[text_pos] = '\0';
    } else if (rec->rtype == 15) {
        /* MX record */
        if (rec->rdlength >= 2) {
            rec->mx_priority = (buf[pos] << 8) | buf[pos + 1];
        }
    }

    return offset + 10 + rec->rdlength;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1109 DNS record type decoder failed: {:?}",
        result.err()
    );
}

/// C1110: DNS zone file parser - parses simplified zone file entries
#[test]
fn c1110_dns_zone_file_parser() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

typedef struct {
    char name[64];
    uint32_t ttl;
    char rclass[4];
    char rtype[8];
    char rdata[128];
} proto_dns_zone_entry_t;

static int proto_zone_skip_ws(const char *line, int pos, int len) {
    while (pos < len && (line[pos] == ' ' || line[pos] == '\t')) pos++;
    return pos;
}

static int proto_zone_read_token(const char *line, int pos, int len,
                                  char *out, int out_cap) {
    int j = 0;
    while (pos < len && line[pos] != ' ' && line[pos] != '\t'
           && line[pos] != '\n' && line[pos] != ';' && j < out_cap - 1) {
        out[j++] = line[pos++];
    }
    out[j] = '\0';
    return pos;
}

int proto_dns_zone_parse_line(const char *line, int len, proto_dns_zone_entry_t *entry) {
    int pos = 0;

    entry->ttl = 0;
    entry->name[0] = '\0';
    entry->rclass[0] = '\0';
    entry->rtype[0] = '\0';
    entry->rdata[0] = '\0';

    /* Skip blank lines and comments */
    if (len == 0 || line[0] == ';' || line[0] == '\n') return 0;

    /* Parse name */
    pos = proto_zone_read_token(line, pos, len, entry->name, 64);
    pos = proto_zone_skip_ws(line, pos, len);

    /* Parse TTL (numeric) */
    if (pos < len && line[pos] >= '0' && line[pos] <= '9') {
        uint32_t ttl = 0;
        while (pos < len && line[pos] >= '0' && line[pos] <= '9') {
            ttl = ttl * 10 + (line[pos] - '0');
            pos++;
        }
        entry->ttl = ttl;
        pos = proto_zone_skip_ws(line, pos, len);
    }

    /* Parse class (IN, CH, HS) */
    pos = proto_zone_read_token(line, pos, len, entry->rclass, 4);
    pos = proto_zone_skip_ws(line, pos, len);

    /* Parse record type */
    pos = proto_zone_read_token(line, pos, len, entry->rtype, 8);
    pos = proto_zone_skip_ws(line, pos, len);

    /* Parse rdata (rest of line) */
    int j = 0;
    while (pos < len && line[pos] != '\n' && line[pos] != ';' && j < 127) {
        entry->rdata[j++] = line[pos++];
    }
    entry->rdata[j] = '\0';

    return 1;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1110 DNS zone file parser failed: {:?}",
        result.err()
    );
}

// ============================================================================
// C1111-C1115: Binary Protocols
// ============================================================================

/// C1111: TLV (Type-Length-Value) parser - generic binary protocol decoder
#[test]
fn c1111_tlv_parser() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

typedef struct {
    uint16_t tag;
    uint16_t length;
    int value_offset;
} proto_tlv_element_t;

typedef struct {
    proto_tlv_element_t elements[64];
    int count;
    int total_bytes;
} proto_tlv_parsed_t;

int proto_tlv_parse(const uint8_t *data, int data_len, proto_tlv_parsed_t *result) {
    int pos = 0;
    result->count = 0;
    result->total_bytes = 0;

    while (pos + 4 <= data_len && result->count < 64) {
        proto_tlv_element_t *elem = &result->elements[result->count];

        /* Read tag (2 bytes, big-endian) */
        elem->tag = (data[pos] << 8) | data[pos + 1];
        pos += 2;

        /* Read length (2 bytes, big-endian) */
        elem->length = (data[pos] << 8) | data[pos + 1];
        pos += 2;

        /* Validate length */
        if (pos + elem->length > data_len) return -1;

        elem->value_offset = pos;
        pos += elem->length;
        result->count++;
    }

    result->total_bytes = pos;
    return result->count;
}

int proto_tlv_find_tag(const proto_tlv_parsed_t *parsed, uint16_t tag) {
    int i;
    for (i = 0; i < parsed->count; i++) {
        if (parsed->elements[i].tag == tag) {
            return i;
        }
    }
    return -1;
}

uint16_t proto_tlv_get_u16(const uint8_t *data, const proto_tlv_element_t *elem) {
    if (elem->length < 2) return 0;
    int off = elem->value_offset;
    return (data[off] << 8) | data[off + 1];
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1111 TLV parser failed: {:?}",
        result.err()
    );
}

/// C1112: Protobuf varint decoder - decodes variable-length integers (LEB128)
#[test]
fn c1112_protobuf_varint_decoder() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned long long uint64_t;
typedef long long int64_t;
typedef unsigned int uint32_t;

typedef struct {
    uint64_t value;
    int bytes_consumed;
} proto_varint_result_t;

int proto_pb_decode_varint(const uint8_t *buf, int buf_len, proto_varint_result_t *out) {
    uint64_t result = 0;
    int shift = 0;
    int i = 0;

    out->value = 0;
    out->bytes_consumed = 0;

    while (i < buf_len && i < 10) {
        uint8_t byte = buf[i];
        result |= ((uint64_t)(byte & 0x7F)) << shift;
        i++;

        if ((byte & 0x80) == 0) {
            out->value = result;
            out->bytes_consumed = i;
            return 0;
        }

        shift += 7;
        if (shift >= 64) return -1;
    }

    return -2;
}

int64_t proto_pb_zigzag_decode(uint64_t n) {
    return (int64_t)((n >> 1) ^ (-(int64_t)(n & 1)));
}

int proto_pb_decode_field(const uint8_t *buf, int buf_len,
                           uint32_t *field_number, uint32_t *wire_type,
                           int *header_size) {
    proto_varint_result_t vr;
    int rc = proto_pb_decode_varint(buf, buf_len, &vr);
    if (rc != 0) return rc;

    *field_number = (uint32_t)(vr.value >> 3);
    *wire_type = (uint32_t)(vr.value & 0x07);
    *header_size = vr.bytes_consumed;
    return 0;
}

int proto_pb_skip_field(const uint8_t *buf, int buf_len, uint32_t wire_type) {
    proto_varint_result_t vr;
    int rc;

    if (wire_type == 0) {
        rc = proto_pb_decode_varint(buf, buf_len, &vr);
        return (rc == 0) ? vr.bytes_consumed : -1;
    } else if (wire_type == 1) {
        return (buf_len >= 8) ? 8 : -1;
    } else if (wire_type == 2) {
        rc = proto_pb_decode_varint(buf, buf_len, &vr);
        if (rc != 0) return -1;
        int total = vr.bytes_consumed + (int)vr.value;
        return (total <= buf_len) ? total : -1;
    } else if (wire_type == 5) {
        return (buf_len >= 4) ? 4 : -1;
    }
    return -1;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1112 protobuf varint decoder failed: {:?}",
        result.err()
    );
}

/// C1113: MQTT packet parser - parses MQTT fixed header and variable header
#[test]
fn c1113_mqtt_packet_parser() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

typedef struct {
    uint8_t packet_type;
    uint8_t flags;
    int remaining_length;
    int header_size;
} proto_mqtt_fixed_header_t;

typedef struct {
    char topic[128];
    int topic_len;
    uint16_t packet_id;
    uint8_t qos;
    uint8_t retain;
    uint8_t dup;
} proto_mqtt_publish_t;

int proto_mqtt_decode_remaining_len(const uint8_t *buf, int buf_len,
                                     int *remaining, int *bytes_used) {
    int multiplier = 1;
    int value = 0;
    int idx = 0;

    do {
        if (idx >= buf_len || idx >= 4) return -1;
        uint8_t byte = buf[idx];
        value += (byte & 0x7F) * multiplier;
        multiplier *= 128;
        idx++;
        if ((byte & 0x80) == 0) break;
    } while (1);

    *remaining = value;
    *bytes_used = idx;
    return 0;
}

int proto_mqtt_parse_fixed_header(const uint8_t *buf, int buf_len,
                                   proto_mqtt_fixed_header_t *hdr) {
    if (buf_len < 2) return -1;

    hdr->packet_type = (buf[0] >> 4) & 0x0F;
    hdr->flags = buf[0] & 0x0F;

    int remaining = 0;
    int bytes_used = 0;
    int rc = proto_mqtt_decode_remaining_len(buf + 1, buf_len - 1, &remaining, &bytes_used);
    if (rc != 0) return rc;

    hdr->remaining_length = remaining;
    hdr->header_size = 1 + bytes_used;

    return 0;
}

int proto_mqtt_parse_publish(const uint8_t *buf, int buf_len,
                              const proto_mqtt_fixed_header_t *hdr,
                              proto_mqtt_publish_t *pub) {
    int pos = hdr->header_size;

    pub->qos = (hdr->flags >> 1) & 0x03;
    pub->retain = hdr->flags & 0x01;
    pub->dup = (hdr->flags >> 3) & 0x01;
    pub->packet_id = 0;

    /* Read topic length */
    if (pos + 2 > buf_len) return -1;
    int topic_len = (buf[pos] << 8) | buf[pos + 1];
    pos += 2;

    if (pos + topic_len > buf_len || topic_len > 127) return -2;

    int k;
    for (k = 0; k < topic_len; k++) {
        pub->topic[k] = buf[pos + k];
    }
    pub->topic[topic_len] = '\0';
    pub->topic_len = topic_len;
    pos += topic_len;

    /* Read packet identifier (QoS > 0) */
    if (pub->qos > 0) {
        if (pos + 2 > buf_len) return -3;
        pub->packet_id = (buf[pos] << 8) | buf[pos + 1];
        pos += 2;
    }

    return pos;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1113 MQTT packet parser failed: {:?}",
        result.err()
    );
}

/// C1114: Redis RESP protocol parser - parses Redis Serialization Protocol
#[test]
fn c1114_redis_resp_parser() {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    int type;
    int integer_val;
    char string_val[256];
    int string_len;
    int array_count;
    int bytes_consumed;
} proto_resp_value_t;

static int proto_resp_read_line(const char *buf, int buf_len, int start,
                                 char *line, int line_cap) {
    int i = start;
    int j = 0;
    while (i < buf_len - 1 && j < line_cap - 1) {
        if (buf[i] == '\r' && buf[i+1] == '\n') {
            line[j] = '\0';
            return i + 2;
        }
        line[j++] = buf[i++];
    }
    line[j] = '\0';
    return -1;
}

static int proto_resp_parse_int(const char *s) {
    int neg = 0;
    int val = 0;
    int i = 0;
    if (s[0] == '-') { neg = 1; i = 1; }
    while (s[i] >= '0' && s[i] <= '9') {
        val = val * 10 + (s[i] - '0');
        i++;
    }
    return neg ? -val : val;
}

int proto_resp_parse(const char *buf, int buf_len, int offset, proto_resp_value_t *val) {
    if (offset >= buf_len) return -1;

    char line[256];
    char type_char = buf[offset];
    int next = proto_resp_read_line(buf, buf_len, offset + 1, line, 256);
    if (next < 0) return -1;

    val->type = type_char;
    val->integer_val = 0;
    val->string_len = 0;
    val->array_count = 0;
    val->string_val[0] = '\0';

    if (type_char == '+') {
        /* Simple string */
        int k;
        for (k = 0; line[k] && k < 255; k++) {
            val->string_val[k] = line[k];
        }
        val->string_val[k] = '\0';
        val->string_len = k;
        val->bytes_consumed = next - offset;
        return 0;
    } else if (type_char == '-') {
        /* Error */
        int k;
        for (k = 0; line[k] && k < 255; k++) {
            val->string_val[k] = line[k];
        }
        val->string_val[k] = '\0';
        val->string_len = k;
        val->bytes_consumed = next - offset;
        return 0;
    } else if (type_char == ':') {
        /* Integer */
        val->integer_val = proto_resp_parse_int(line);
        val->bytes_consumed = next - offset;
        return 0;
    } else if (type_char == '$') {
        /* Bulk string */
        int str_len = proto_resp_parse_int(line);
        if (str_len < 0) {
            val->string_len = -1;
            val->bytes_consumed = next - offset;
            return 0;
        }
        if (next + str_len + 2 > buf_len) return -2;
        int k;
        for (k = 0; k < str_len && k < 255; k++) {
            val->string_val[k] = buf[next + k];
        }
        val->string_val[k] = '\0';
        val->string_len = str_len;
        val->bytes_consumed = (next + str_len + 2) - offset;
        return 0;
    } else if (type_char == '*') {
        /* Array */
        val->array_count = proto_resp_parse_int(line);
        val->bytes_consumed = next - offset;
        return 0;
    }

    return -3;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1114 Redis RESP parser failed: {:?}",
        result.err()
    );
}

/// C1115: MessagePack decoder - decodes MessagePack binary format
#[test]
fn c1115_messagepack_decoder() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;
typedef long long int64_t;

typedef struct {
    int type;
    int64_t int_val;
    char str_val[256];
    int str_len;
    int array_len;
    int map_len;
    int bytes_consumed;
} proto_msgpack_value_t;

int proto_msgpack_decode(const uint8_t *buf, int buf_len, int offset,
                          proto_msgpack_value_t *val) {
    if (offset >= buf_len) return -1;

    uint8_t first = buf[offset];
    int pos = offset + 1;

    val->int_val = 0;
    val->str_len = 0;
    val->array_len = 0;
    val->map_len = 0;
    val->str_val[0] = '\0';

    /* Positive fixint (0x00 - 0x7f) */
    if (first <= 0x7F) {
        val->type = 1; /* integer */
        val->int_val = first;
        val->bytes_consumed = 1;
        return 0;
    }

    /* Negative fixint (0xe0 - 0xff) */
    if (first >= 0xE0) {
        val->type = 1;
        val->int_val = (int64_t)((signed char)first);
        val->bytes_consumed = 1;
        return 0;
    }

    /* Fixstr (0xa0 - 0xbf) */
    if ((first & 0xE0) == 0xA0) {
        int len = first & 0x1F;
        val->type = 3; /* string */
        if (pos + len > buf_len) return -2;
        int k;
        for (k = 0; k < len && k < 255; k++) {
            val->str_val[k] = buf[pos + k];
        }
        val->str_val[k] = '\0';
        val->str_len = len;
        val->bytes_consumed = 1 + len;
        return 0;
    }

    /* Fixarray (0x90 - 0x9f) */
    if ((first & 0xF0) == 0x90) {
        val->type = 4; /* array */
        val->array_len = first & 0x0F;
        val->bytes_consumed = 1;
        return 0;
    }

    /* Fixmap (0x80 - 0x8f) */
    if ((first & 0xF0) == 0x80) {
        val->type = 5; /* map */
        val->map_len = first & 0x0F;
        val->bytes_consumed = 1;
        return 0;
    }

    /* uint8 */
    if (first == 0xCC) {
        if (pos >= buf_len) return -3;
        val->type = 1;
        val->int_val = buf[pos];
        val->bytes_consumed = 2;
        return 0;
    }

    /* uint16 */
    if (first == 0xCD) {
        if (pos + 1 >= buf_len) return -3;
        val->type = 1;
        val->int_val = (buf[pos] << 8) | buf[pos + 1];
        val->bytes_consumed = 3;
        return 0;
    }

    /* uint32 */
    if (first == 0xCE) {
        if (pos + 3 >= buf_len) return -3;
        val->type = 1;
        val->int_val = ((uint32_t)buf[pos] << 24) | ((uint32_t)buf[pos+1] << 16) |
                       ((uint32_t)buf[pos+2] << 8) | buf[pos+3];
        val->bytes_consumed = 5;
        return 0;
    }

    /* nil */
    if (first == 0xC0) {
        val->type = 0;
        val->bytes_consumed = 1;
        return 0;
    }

    /* true/false */
    if (first == 0xC2 || first == 0xC3) {
        val->type = 2; /* bool */
        val->int_val = (first == 0xC3) ? 1 : 0;
        val->bytes_consumed = 1;
        return 0;
    }

    return -4;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1115 MessagePack decoder failed: {:?}",
        result.err()
    );
}

// ============================================================================
// C1116-C1120: Network Headers
// ============================================================================

/// C1116: IPv4 header parser - parses IPv4 packet header fields
#[test]
fn c1116_ipv4_header_parser() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

typedef struct {
    uint8_t version;
    uint8_t ihl;
    uint8_t dscp;
    uint8_t ecn;
    uint16_t total_length;
    uint16_t identification;
    uint8_t flags;
    uint16_t fragment_offset;
    uint8_t ttl;
    uint8_t protocol;
    uint16_t header_checksum;
    uint32_t src_addr;
    uint32_t dst_addr;
    int header_valid;
} proto_ipv4_header_t;

static uint16_t proto_ipv4_read_u16(const uint8_t *p) {
    return (uint16_t)((p[0] << 8) | p[1]);
}

static uint32_t proto_ipv4_read_u32(const uint8_t *p) {
    return ((uint32_t)p[0] << 24) | ((uint32_t)p[1] << 16) |
           ((uint32_t)p[2] << 8) | p[3];
}

int proto_ipv4_parse(const uint8_t *pkt, int pkt_len, proto_ipv4_header_t *hdr) {
    if (pkt_len < 20) return -1;

    hdr->version = (pkt[0] >> 4) & 0x0F;
    hdr->ihl = pkt[0] & 0x0F;

    if (hdr->version != 4) return -2;
    if (hdr->ihl < 5) return -3;

    int header_bytes = hdr->ihl * 4;
    if (header_bytes > pkt_len) return -4;

    hdr->dscp = (pkt[1] >> 2) & 0x3F;
    hdr->ecn = pkt[1] & 0x03;
    hdr->total_length = proto_ipv4_read_u16(pkt + 2);
    hdr->identification = proto_ipv4_read_u16(pkt + 4);

    uint16_t flags_frag = proto_ipv4_read_u16(pkt + 6);
    hdr->flags = (flags_frag >> 13) & 0x07;
    hdr->fragment_offset = flags_frag & 0x1FFF;

    hdr->ttl = pkt[8];
    hdr->protocol = pkt[9];
    hdr->header_checksum = proto_ipv4_read_u16(pkt + 10);
    hdr->src_addr = proto_ipv4_read_u32(pkt + 12);
    hdr->dst_addr = proto_ipv4_read_u32(pkt + 16);

    /* Verify header checksum */
    uint32_t sum = 0;
    int i;
    for (i = 0; i < header_bytes; i += 2) {
        sum += proto_ipv4_read_u16(pkt + i);
    }
    while (sum >> 16) {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    hdr->header_valid = ((uint16_t)~sum == 0) ? 1 : 0;

    return header_bytes;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1116 IPv4 header parser failed: {:?}",
        result.err()
    );
}

/// C1117: TCP header parser - parses TCP segment header and flags
#[test]
fn c1117_tcp_header_parser() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

typedef struct {
    uint16_t src_port;
    uint16_t dst_port;
    uint32_t seq_num;
    uint32_t ack_num;
    uint8_t data_offset;
    uint8_t flags_fin;
    uint8_t flags_syn;
    uint8_t flags_rst;
    uint8_t flags_psh;
    uint8_t flags_ack;
    uint8_t flags_urg;
    uint16_t window_size;
    uint16_t checksum;
    uint16_t urgent_ptr;
    int header_len;
} proto_tcp_header_t;

int proto_tcp_parse(const uint8_t *seg, int seg_len, proto_tcp_header_t *hdr) {
    if (seg_len < 20) return -1;

    hdr->src_port = (seg[0] << 8) | seg[1];
    hdr->dst_port = (seg[2] << 8) | seg[3];

    hdr->seq_num = ((uint32_t)seg[4] << 24) | ((uint32_t)seg[5] << 16) |
                   ((uint32_t)seg[6] << 8) | seg[7];
    hdr->ack_num = ((uint32_t)seg[8] << 24) | ((uint32_t)seg[9] << 16) |
                   ((uint32_t)seg[10] << 8) | seg[11];

    hdr->data_offset = (seg[12] >> 4) & 0x0F;
    hdr->header_len = hdr->data_offset * 4;

    if (hdr->header_len < 20 || hdr->header_len > seg_len) return -2;

    uint8_t flags = seg[13];
    hdr->flags_fin = (flags & 0x01) ? 1 : 0;
    hdr->flags_syn = (flags & 0x02) ? 1 : 0;
    hdr->flags_rst = (flags & 0x04) ? 1 : 0;
    hdr->flags_psh = (flags & 0x08) ? 1 : 0;
    hdr->flags_ack = (flags & 0x10) ? 1 : 0;
    hdr->flags_urg = (flags & 0x20) ? 1 : 0;

    hdr->window_size = (seg[14] << 8) | seg[15];
    hdr->checksum = (seg[16] << 8) | seg[17];
    hdr->urgent_ptr = (seg[18] << 8) | seg[19];

    return hdr->header_len;
}

int proto_tcp_is_syn(const proto_tcp_header_t *hdr) {
    return hdr->flags_syn && !hdr->flags_ack;
}

int proto_tcp_is_synack(const proto_tcp_header_t *hdr) {
    return hdr->flags_syn && hdr->flags_ack;
}

int proto_tcp_is_fin(const proto_tcp_header_t *hdr) {
    return hdr->flags_fin != 0;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1117 TCP header parser failed: {:?}",
        result.err()
    );
}

/// C1118: UDP checksum calculator - computes UDP pseudo-header checksum
#[test]
fn c1118_udp_checksum_calculator() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

typedef struct {
    uint16_t src_port;
    uint16_t dst_port;
    uint16_t length;
    uint16_t checksum;
} proto_udp_header_t;

int proto_udp_parse(const uint8_t *data, int data_len, proto_udp_header_t *hdr) {
    if (data_len < 8) return -1;

    hdr->src_port = (data[0] << 8) | data[1];
    hdr->dst_port = (data[2] << 8) | data[3];
    hdr->length = (data[4] << 8) | data[5];
    hdr->checksum = (data[6] << 8) | data[7];

    if (hdr->length < 8 || hdr->length > data_len) return -2;
    return 0;
}

static uint32_t proto_udp_sum_words(const uint8_t *data, int len) {
    uint32_t sum = 0;
    int i;
    for (i = 0; i + 1 < len; i += 2) {
        sum += (data[i] << 8) | data[i + 1];
    }
    if (len & 1) {
        sum += data[len - 1] << 8;
    }
    return sum;
}

uint16_t proto_udp_compute_checksum(uint32_t src_ip, uint32_t dst_ip,
                                     const uint8_t *udp_data, int udp_len) {
    uint32_t sum = 0;

    /* Pseudo-header */
    sum += (src_ip >> 16) & 0xFFFF;
    sum += src_ip & 0xFFFF;
    sum += (dst_ip >> 16) & 0xFFFF;
    sum += dst_ip & 0xFFFF;
    sum += 17; /* UDP protocol number */
    sum += udp_len;

    /* UDP header + data (skip checksum field) */
    sum += proto_udp_sum_words(udp_data, udp_len);

    /* Subtract existing checksum field to zero it */
    sum -= (udp_data[6] << 8) | udp_data[7];

    /* Fold carry bits */
    while (sum >> 16) {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    return (uint16_t)(~sum & 0xFFFF);
}

int proto_udp_verify_checksum(uint32_t src_ip, uint32_t dst_ip,
                               const uint8_t *udp_data, int udp_len) {
    if (udp_len < 8) return -1;
    uint16_t stored = (udp_data[6] << 8) | udp_data[7];
    if (stored == 0) return 1; /* checksum disabled */
    uint16_t computed = proto_udp_compute_checksum(src_ip, dst_ip, udp_data, udp_len);
    return (computed == 0) ? 1 : 0;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1118 UDP checksum calculator failed: {:?}",
        result.err()
    );
}

/// C1119: ARP packet parser - parses Address Resolution Protocol packets
#[test]
fn c1119_arp_packet_parser() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

typedef struct {
    uint16_t hw_type;
    uint16_t proto_type;
    uint8_t hw_addr_len;
    uint8_t proto_addr_len;
    uint16_t operation;
    uint8_t sender_hw_addr[6];
    uint32_t sender_proto_addr;
    uint8_t target_hw_addr[6];
    uint32_t target_proto_addr;
} proto_arp_packet_t;

int proto_arp_parse(const uint8_t *pkt, int pkt_len, proto_arp_packet_t *arp) {
    if (pkt_len < 28) return -1;

    arp->hw_type = (pkt[0] << 8) | pkt[1];
    arp->proto_type = (pkt[2] << 8) | pkt[3];
    arp->hw_addr_len = pkt[4];
    arp->proto_addr_len = pkt[5];
    arp->operation = (pkt[6] << 8) | pkt[7];

    /* Validate lengths for Ethernet + IPv4 */
    if (arp->hw_addr_len != 6 || arp->proto_addr_len != 4) return -2;

    int pos = 8;
    int k;
    for (k = 0; k < 6; k++) arp->sender_hw_addr[k] = pkt[pos + k];
    pos += 6;

    arp->sender_proto_addr = ((uint32_t)pkt[pos] << 24) | ((uint32_t)pkt[pos+1] << 16) |
                              ((uint32_t)pkt[pos+2] << 8) | pkt[pos+3];
    pos += 4;

    for (k = 0; k < 6; k++) arp->target_hw_addr[k] = pkt[pos + k];
    pos += 6;

    arp->target_proto_addr = ((uint32_t)pkt[pos] << 24) | ((uint32_t)pkt[pos+1] << 16) |
                              ((uint32_t)pkt[pos+2] << 8) | pkt[pos+3];
    return 0;
}

int proto_arp_is_request(const proto_arp_packet_t *arp) {
    return arp->operation == 1;
}

int proto_arp_is_reply(const proto_arp_packet_t *arp) {
    return arp->operation == 2;
}

int proto_arp_build_reply(const proto_arp_packet_t *request,
                           const uint8_t *our_hw_addr,
                           uint8_t *out_pkt, int out_cap) {
    if (out_cap < 28) return -1;
    if (request->operation != 1) return -2;

    /* Hardware type */
    out_pkt[0] = 0x00; out_pkt[1] = 0x01;
    /* Protocol type (IPv4) */
    out_pkt[2] = 0x08; out_pkt[3] = 0x00;
    out_pkt[4] = 6; out_pkt[5] = 4;
    /* Operation: reply */
    out_pkt[6] = 0x00; out_pkt[7] = 0x02;

    int pos = 8;
    int k;
    /* Sender = us */
    for (k = 0; k < 6; k++) out_pkt[pos + k] = our_hw_addr[k];
    pos += 6;
    out_pkt[pos] = (request->target_proto_addr >> 24) & 0xFF;
    out_pkt[pos+1] = (request->target_proto_addr >> 16) & 0xFF;
    out_pkt[pos+2] = (request->target_proto_addr >> 8) & 0xFF;
    out_pkt[pos+3] = request->target_proto_addr & 0xFF;
    pos += 4;

    /* Target = original sender */
    for (k = 0; k < 6; k++) out_pkt[pos + k] = request->sender_hw_addr[k];
    pos += 6;
    out_pkt[pos] = (request->sender_proto_addr >> 24) & 0xFF;
    out_pkt[pos+1] = (request->sender_proto_addr >> 16) & 0xFF;
    out_pkt[pos+2] = (request->sender_proto_addr >> 8) & 0xFF;
    out_pkt[pos+3] = request->sender_proto_addr & 0xFF;

    return 28;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1119 ARP packet parser failed: {:?}",
        result.err()
    );
}

/// C1120: ICMP packet parser - parses ICMP echo request/reply
#[test]
fn c1120_icmp_packet_parser() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

typedef struct {
    uint8_t type;
    uint8_t code;
    uint16_t checksum;
    uint16_t identifier;
    uint16_t sequence;
    int payload_offset;
    int payload_len;
    int checksum_valid;
} proto_icmp_packet_t;

static uint16_t proto_icmp_checksum(const uint8_t *data, int len) {
    uint32_t sum = 0;
    int i;
    for (i = 0; i + 1 < len; i += 2) {
        sum += (data[i] << 8) | data[i + 1];
    }
    if (len & 1) {
        sum += data[len - 1] << 8;
    }
    while (sum >> 16) {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    return (uint16_t)(~sum & 0xFFFF);
}

int proto_icmp_parse(const uint8_t *pkt, int pkt_len, proto_icmp_packet_t *icmp) {
    if (pkt_len < 8) return -1;

    icmp->type = pkt[0];
    icmp->code = pkt[1];
    icmp->checksum = (pkt[2] << 8) | pkt[3];
    icmp->identifier = (pkt[4] << 8) | pkt[5];
    icmp->sequence = (pkt[6] << 8) | pkt[7];
    icmp->payload_offset = 8;
    icmp->payload_len = pkt_len - 8;

    /* Verify checksum */
    uint16_t computed = proto_icmp_checksum(pkt, pkt_len);
    icmp->checksum_valid = (computed == 0) ? 1 : 0;

    return 0;
}

int proto_icmp_is_echo_request(const proto_icmp_packet_t *icmp) {
    return (icmp->type == 8 && icmp->code == 0);
}

int proto_icmp_is_echo_reply(const proto_icmp_packet_t *icmp) {
    return (icmp->type == 0 && icmp->code == 0);
}

int proto_icmp_build_echo_reply(const uint8_t *request, int req_len,
                                 uint8_t *reply, int reply_cap) {
    if (req_len < 8 || reply_cap < req_len) return -1;

    int i;
    for (i = 0; i < req_len; i++) {
        reply[i] = request[i];
    }

    reply[0] = 0; /* Echo Reply type */
    reply[1] = 0; /* Code 0 */
    reply[2] = 0; /* Zero checksum for calculation */
    reply[3] = 0;

    uint16_t cksum = proto_icmp_checksum(reply, req_len);
    reply[2] = (cksum >> 8) & 0xFF;
    reply[3] = cksum & 0xFF;

    return req_len;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1120 ICMP packet parser failed: {:?}",
        result.err()
    );
}

// ============================================================================
// C1121-C1125: Application Protocols
// ============================================================================

/// C1121: SMTP command parser - parses SMTP commands and arguments
#[test]
fn c1121_smtp_command_parser() {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    char command[8];
    char argument[256];
    int valid;
} proto_smtp_cmd_t;

static int proto_smtp_toupper(char c) {
    if (c >= 'a' && c <= 'z') return c - 32;
    return c;
}

int proto_smtp_parse_command(const char *line, int len, proto_smtp_cmd_t *cmd) {
    int i = 0;
    int j = 0;

    cmd->valid = 0;
    cmd->command[0] = '\0';
    cmd->argument[0] = '\0';

    /* Skip leading whitespace */
    while (i < len && (line[i] == ' ' || line[i] == '\t')) i++;

    /* Parse command (uppercase, max 4 chars) */
    while (i < len && line[i] != ' ' && line[i] != '\r' && line[i] != '\n' && j < 7) {
        cmd->command[j++] = proto_smtp_toupper(line[i++]);
    }
    cmd->command[j] = '\0';

    if (j == 0) return -1;

    /* Skip whitespace before argument */
    while (i < len && line[i] == ' ') i++;

    /* Parse argument (rest of line minus CRLF) */
    j = 0;
    while (i < len && line[i] != '\r' && line[i] != '\n' && j < 255) {
        cmd->argument[j++] = line[i++];
    }
    cmd->argument[j] = '\0';

    /* Trim trailing spaces from argument */
    while (j > 0 && cmd->argument[j-1] == ' ') {
        j--;
        cmd->argument[j] = '\0';
    }

    cmd->valid = 1;
    return 0;
}

int proto_smtp_is_ehlo(const proto_smtp_cmd_t *cmd) {
    return (cmd->command[0] == 'E' && cmd->command[1] == 'H' &&
            cmd->command[2] == 'L' && cmd->command[3] == 'O' &&
            cmd->command[4] == '\0');
}

int proto_smtp_is_mail_from(const proto_smtp_cmd_t *cmd) {
    return (cmd->command[0] == 'M' && cmd->command[1] == 'A' &&
            cmd->command[2] == 'I' && cmd->command[3] == 'L' &&
            cmd->command[4] == '\0');
}

int proto_smtp_extract_email(const char *arg, char *email, int email_cap) {
    const char *start = arg;
    int j = 0;

    /* Find opening < */
    while (*start && *start != '<') start++;
    if (*start == '<') start++;

    /* Copy until > or end */
    while (*start && *start != '>' && j < email_cap - 1) {
        email[j++] = *start++;
    }
    email[j] = '\0';

    return (j > 0) ? 0 : -1;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1121 SMTP command parser failed: {:?}",
        result.err()
    );
}

/// C1122: FTP response parser - parses FTP response codes and messages
#[test]
fn c1122_ftp_response_parser() {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    int code;
    char message[256];
    int is_multiline;
    int is_final;
} proto_ftp_response_t;

int proto_ftp_parse_response(const char *line, int len, proto_ftp_response_t *resp) {
    resp->code = 0;
    resp->message[0] = '\0';
    resp->is_multiline = 0;
    resp->is_final = 0;

    if (len < 3) return -1;

    /* Parse 3-digit code */
    int i;
    for (i = 0; i < 3; i++) {
        if (line[i] < '0' || line[i] > '9') return -2;
        resp->code = resp->code * 10 + (line[i] - '0');
    }

    /* Check separator: space = final, hyphen = multiline continuation */
    if (i < len) {
        if (line[i] == ' ') {
            resp->is_final = 1;
            i++;
        } else if (line[i] == '-') {
            resp->is_multiline = 1;
            i++;
        } else {
            return -3;
        }
    }

    /* Copy message text */
    int j = 0;
    while (i < len && line[i] != '\r' && line[i] != '\n' && j < 255) {
        resp->message[j++] = line[i++];
    }
    resp->message[j] = '\0';

    return 0;
}

int proto_ftp_response_class(int code) {
    /* 1xx=preliminary, 2xx=completion, 3xx=intermediate, 4xx=transient, 5xx=permanent */
    if (code >= 100 && code < 200) return 1;
    if (code >= 200 && code < 300) return 2;
    if (code >= 300 && code < 400) return 3;
    if (code >= 400 && code < 500) return 4;
    if (code >= 500 && code < 600) return 5;
    return -1;
}

int proto_ftp_is_success(int code) {
    return (code >= 200 && code < 300);
}

int proto_ftp_parse_pasv(const char *msg, int msg_len,
                          unsigned char *ip, unsigned short *port) {
    int i = 0;
    int nums[6];
    int num_idx = 0;

    /* Find opening parenthesis */
    while (i < msg_len && msg[i] != '(') i++;
    if (i >= msg_len) return -1;
    i++;

    /* Parse 6 comma-separated numbers: h1,h2,h3,h4,p1,p2 */
    while (num_idx < 6 && i < msg_len) {
        int val = 0;
        while (i < msg_len && msg[i] >= '0' && msg[i] <= '9') {
            val = val * 10 + (msg[i] - '0');
            i++;
        }
        nums[num_idx++] = val;
        if (i < msg_len && msg[i] == ',') i++;
    }

    if (num_idx != 6) return -2;

    ip[0] = (unsigned char)nums[0];
    ip[1] = (unsigned char)nums[1];
    ip[2] = (unsigned char)nums[2];
    ip[3] = (unsigned char)nums[3];
    *port = (unsigned short)(nums[4] * 256 + nums[5]);

    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1122 FTP response parser failed: {:?}",
        result.err()
    );
}

/// C1123: SIP header parser - parses Session Initiation Protocol headers
#[test]
fn c1123_sip_header_parser() {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    char method[16];
    char request_uri[128];
    char sip_version[16];
    int is_request;
    int status_code;
} proto_sip_start_line_t;

typedef struct {
    char name[32];
    char value[256];
} proto_sip_header_t;

int proto_sip_parse_start_line(const char *line, int len, proto_sip_start_line_t *sl) {
    int i = 0;
    int j = 0;

    sl->is_request = 0;
    sl->status_code = 0;
    sl->method[0] = '\0';
    sl->request_uri[0] = '\0';
    sl->sip_version[0] = '\0';

    /* Check if this is a response (starts with SIP/) or request */
    if (len > 3 && line[0] == 'S' && line[1] == 'I' && line[2] == 'P' && line[3] == '/') {
        /* Response: SIP/2.0 200 OK */
        while (i < len && line[i] != ' ' && j < 15) {
            sl->sip_version[j++] = line[i++];
        }
        sl->sip_version[j] = '\0';

        if (i < len) i++; /* skip space */

        /* Parse status code */
        while (i < len && line[i] >= '0' && line[i] <= '9') {
            sl->status_code = sl->status_code * 10 + (line[i] - '0');
            i++;
        }

        sl->is_request = 0;
        return 0;
    }

    /* Request: INVITE sip:user@example.com SIP/2.0 */
    sl->is_request = 1;

    while (i < len && line[i] != ' ' && j < 15) {
        sl->method[j++] = line[i++];
    }
    sl->method[j] = '\0';

    if (i < len) i++; /* skip space */

    j = 0;
    while (i < len && line[i] != ' ' && j < 127) {
        sl->request_uri[j++] = line[i++];
    }
    sl->request_uri[j] = '\0';

    if (i < len) i++; /* skip space */

    j = 0;
    while (i < len && line[i] != '\r' && line[i] != '\n' && j < 15) {
        sl->sip_version[j++] = line[i++];
    }
    sl->sip_version[j] = '\0';

    return 0;
}

int proto_sip_parse_header(const char *line, int len, proto_sip_header_t *hdr) {
    int i = 0;
    int j = 0;

    hdr->name[0] = '\0';
    hdr->value[0] = '\0';

    /* Parse header name */
    while (i < len && line[i] != ':' && j < 31) {
        hdr->name[j++] = line[i++];
    }
    hdr->name[j] = '\0';

    if (i >= len || line[i] != ':') return -1;
    i++;

    /* Skip whitespace */
    while (i < len && (line[i] == ' ' || line[i] == '\t')) i++;

    /* Parse value */
    j = 0;
    while (i < len && line[i] != '\r' && line[i] != '\n' && j < 255) {
        hdr->value[j++] = line[i++];
    }
    hdr->value[j] = '\0';

    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1123 SIP header parser failed: {:?}",
        result.err()
    );
}

/// C1124: RTSP request parser - parses Real Time Streaming Protocol requests
#[test]
fn c1124_rtsp_request_parser() {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    char method[16];
    char uri[256];
    char version[16];
    int cseq;
    char session[64];
    int content_length;
} proto_rtsp_request_t;

static int proto_rtsp_streq(const char *a, const char *b) {
    while (*a && *b) {
        if (*a != *b) return 0;
        a++;
        b++;
    }
    return (*a == '\0' && *b == '\0');
}

int proto_rtsp_parse_request_line(const char *line, int len,
                                   proto_rtsp_request_t *req) {
    int i = 0;
    int j = 0;

    req->method[0] = '\0';
    req->uri[0] = '\0';
    req->version[0] = '\0';
    req->cseq = 0;
    req->session[0] = '\0';
    req->content_length = 0;

    /* Parse method */
    while (i < len && line[i] != ' ' && j < 15) {
        req->method[j++] = line[i++];
    }
    req->method[j] = '\0';

    if (i < len) i++;

    /* Parse URI */
    j = 0;
    while (i < len && line[i] != ' ' && j < 255) {
        req->uri[j++] = line[i++];
    }
    req->uri[j] = '\0';

    if (i < len) i++;

    /* Parse version */
    j = 0;
    while (i < len && line[i] != '\r' && line[i] != '\n' && j < 15) {
        req->version[j++] = line[i++];
    }
    req->version[j] = '\0';

    return 0;
}

int proto_rtsp_parse_header(proto_rtsp_request_t *req,
                             const char *name, const char *value) {
    if (proto_rtsp_streq(name, "CSeq")) {
        int val = 0;
        const char *p = value;
        while (*p >= '0' && *p <= '9') {
            val = val * 10 + (*p - '0');
            p++;
        }
        req->cseq = val;
    } else if (proto_rtsp_streq(name, "Session")) {
        int j = 0;
        while (value[j] && value[j] != ';' && j < 63) {
            req->session[j] = value[j];
            j++;
        }
        req->session[j] = '\0';
    } else if (proto_rtsp_streq(name, "Content-Length")) {
        int val = 0;
        const char *p = value;
        while (*p >= '0' && *p <= '9') {
            val = val * 10 + (*p - '0');
            p++;
        }
        req->content_length = val;
    }

    return 0;
}

int proto_rtsp_method_id(const char *method) {
    if (proto_rtsp_streq(method, "DESCRIBE")) return 1;
    if (proto_rtsp_streq(method, "ANNOUNCE")) return 2;
    if (proto_rtsp_streq(method, "SETUP")) return 3;
    if (proto_rtsp_streq(method, "PLAY")) return 4;
    if (proto_rtsp_streq(method, "PAUSE")) return 5;
    if (proto_rtsp_streq(method, "TEARDOWN")) return 6;
    if (proto_rtsp_streq(method, "OPTIONS")) return 7;
    if (proto_rtsp_streq(method, "RECORD")) return 8;
    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1124 RTSP request parser failed: {:?}",
        result.err()
    );
}

/// C1125: WebSocket frame parser - parses WebSocket frame header per RFC 6455
#[test]
fn c1125_websocket_frame_parser() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned long long uint64_t;

typedef struct {
    uint8_t fin;
    uint8_t opcode;
    uint8_t masked;
    uint64_t payload_length;
    uint8_t masking_key[4];
    int header_size;
} proto_ws_frame_t;

int proto_ws_parse_frame(const uint8_t *buf, int buf_len, proto_ws_frame_t *frame) {
    if (buf_len < 2) return -1;

    int pos = 0;

    frame->fin = (buf[pos] >> 7) & 0x01;
    frame->opcode = buf[pos] & 0x0F;
    pos++;

    frame->masked = (buf[pos] >> 7) & 0x01;
    uint8_t len_byte = buf[pos] & 0x7F;
    pos++;

    if (len_byte <= 125) {
        frame->payload_length = len_byte;
    } else if (len_byte == 126) {
        if (pos + 2 > buf_len) return -2;
        frame->payload_length = ((uint64_t)buf[pos] << 8) | buf[pos + 1];
        pos += 2;
    } else {
        /* len_byte == 127 */
        if (pos + 8 > buf_len) return -3;
        frame->payload_length = 0;
        int k;
        for (k = 0; k < 8; k++) {
            frame->payload_length = (frame->payload_length << 8) | buf[pos + k];
        }
        pos += 8;
    }

    if (frame->masked) {
        if (pos + 4 > buf_len) return -4;
        frame->masking_key[0] = buf[pos];
        frame->masking_key[1] = buf[pos + 1];
        frame->masking_key[2] = buf[pos + 2];
        frame->masking_key[3] = buf[pos + 3];
        pos += 4;
    } else {
        frame->masking_key[0] = 0;
        frame->masking_key[1] = 0;
        frame->masking_key[2] = 0;
        frame->masking_key[3] = 0;
    }

    frame->header_size = pos;
    return 0;
}

void proto_ws_unmask_payload(uint8_t *payload, int payload_len,
                              const uint8_t *masking_key) {
    int i;
    for (i = 0; i < payload_len; i++) {
        payload[i] ^= masking_key[i % 4];
    }
}

int proto_ws_is_control_frame(uint8_t opcode) {
    return (opcode >= 0x08);
}

int proto_ws_is_text(uint8_t opcode) {
    return (opcode == 0x01);
}

int proto_ws_is_binary(uint8_t opcode) {
    return (opcode == 0x02);
}

int proto_ws_is_close(uint8_t opcode) {
    return (opcode == 0x08);
}

int proto_ws_is_ping(uint8_t opcode) {
    return (opcode == 0x09);
}

int proto_ws_is_pong(uint8_t opcode) {
    return (opcode == 0x0A);
}
"#;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1125 WebSocket frame parser failed: {:?}",
        result.err()
    );
}
