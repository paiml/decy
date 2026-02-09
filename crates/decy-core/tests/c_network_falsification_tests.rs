//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C326-C350: Network Protocols and Parsers -- protocol implementations,
//! packet parsers, and network stack patterns commonly found in production
//! network code.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! Organization:
//! - C326-C330: Core protocols (IPv4, TCP, HTTP, DNS, ARP)
//! - C331-C335: Application/transport layers (DHCP, URL, WebSocket, ICMP, routing)
//! - C336-C340: Security/transport (TLS, MQTT, gRPC, Ethernet, NAT)
//! - C341-C345: Modern protocols (QUIC, BGP, SNMP, conntrack, chunked encoding)
//! - C346-C350: Signaling/management (RADIUS, SIP, STUN, mDNS, Netflow)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C326-C330: Core Protocols (IPv4, TCP, HTTP, DNS, ARP)
// ============================================================================

#[test]
fn c326_ipv4_header_checksum_computation() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

typedef struct {
    uint8_t version_ihl;
    uint8_t tos;
    uint16_t total_length;
    uint16_t identification;
    uint16_t flags_fragment;
    uint8_t ttl;
    uint8_t protocol;
    uint16_t checksum;
    uint32_t src_addr;
    uint32_t dst_addr;
} ipv4_header_t;

uint16_t ipv4_checksum(const uint8_t *data, int len) {
    uint32_t sum = 0;
    int i;
    for (i = 0; i + 1 < len; i += 2) {
        sum += ((uint32_t)data[i] << 8) | (uint32_t)data[i + 1];
    }
    if (i < len) {
        sum += (uint32_t)data[i] << 8;
    }
    while (sum >> 16) {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    return (uint16_t)(~sum & 0xFFFF);
}

void ipv4_init_header(ipv4_header_t *hdr, uint32_t src, uint32_t dst,
                      uint8_t protocol, uint16_t payload_len) {
    hdr->version_ihl = 0x45;
    hdr->tos = 0;
    hdr->total_length = 20 + payload_len;
    hdr->identification = 0;
    hdr->flags_fragment = 0x4000;
    hdr->ttl = 64;
    hdr->protocol = protocol;
    hdr->checksum = 0;
    hdr->src_addr = src;
    hdr->dst_addr = dst;
}

int ipv4_get_version(const ipv4_header_t *hdr) {
    return (hdr->version_ihl >> 4) & 0x0F;
}

int ipv4_get_ihl(const ipv4_header_t *hdr) {
    return (hdr->version_ihl & 0x0F) * 4;
}

int ipv4_is_fragment(const ipv4_header_t *hdr) {
    return (hdr->flags_fragment & 0x1FFF) != 0 ||
           (hdr->flags_fragment & 0x2000) != 0;
}

int ipv4_validate_header(const ipv4_header_t *hdr) {
    if (ipv4_get_version(hdr) != 4) return -1;
    if (ipv4_get_ihl(hdr) < 20) return -2;
    if (hdr->total_length < 20) return -3;
    if (hdr->ttl == 0) return -4;
    return 0;
}

uint16_t ipv4_payload_length(const ipv4_header_t *hdr) {
    return hdr->total_length - (uint16_t)ipv4_get_ihl(hdr);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C326: IPv4 header checksum computation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C326: Output should not be empty");
    assert!(
        code.contains("fn ipv4_checksum"),
        "C326: Should contain ipv4_checksum function"
    );
    assert!(
        code.contains("fn ipv4_init_header"),
        "C326: Should contain ipv4_init_header function"
    );
    assert!(
        code.contains("fn ipv4_validate_header"),
        "C326: Should contain ipv4_validate_header function"
    );
}

#[test]
fn c327_tcp_three_way_handshake_state_machine() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

#define TCP_STATE_CLOSED     0
#define TCP_STATE_LISTEN     1
#define TCP_STATE_SYN_SENT   2
#define TCP_STATE_SYN_RCVD   3
#define TCP_STATE_ESTABLISHED 4
#define TCP_STATE_FIN_WAIT1  5
#define TCP_STATE_FIN_WAIT2  6
#define TCP_STATE_CLOSE_WAIT 7
#define TCP_STATE_LAST_ACK   8
#define TCP_STATE_TIME_WAIT  9

#define TCP_FLAG_SYN 0x02
#define TCP_FLAG_ACK 0x10
#define TCP_FLAG_FIN 0x01
#define TCP_FLAG_RST 0x04

typedef struct {
    int state;
    uint32_t seq_num;
    uint32_t ack_num;
    uint32_t peer_seq;
    uint16_t local_port;
    uint16_t remote_port;
    uint16_t window_size;
    uint32_t retransmit_count;
    int is_server;
} tcp_connection_t;

void tcp_init(tcp_connection_t *conn, uint16_t local_port, int is_server) {
    conn->state = TCP_STATE_CLOSED;
    conn->seq_num = 1000;
    conn->ack_num = 0;
    conn->peer_seq = 0;
    conn->local_port = local_port;
    conn->remote_port = 0;
    conn->window_size = 65535;
    conn->retransmit_count = 0;
    conn->is_server = is_server;
}

int tcp_listen(tcp_connection_t *conn) {
    if (conn->state != TCP_STATE_CLOSED) return -1;
    if (!conn->is_server) return -2;
    conn->state = TCP_STATE_LISTEN;
    return 0;
}

int tcp_connect(tcp_connection_t *conn, uint16_t remote_port) {
    if (conn->state != TCP_STATE_CLOSED) return -1;
    conn->remote_port = remote_port;
    conn->state = TCP_STATE_SYN_SENT;
    return 0;
}

int tcp_handle_segment(tcp_connection_t *conn, uint8_t flags,
                       uint32_t seg_seq, uint32_t seg_ack) {
    if (conn->state == TCP_STATE_LISTEN && (flags & TCP_FLAG_SYN)) {
        conn->peer_seq = seg_seq;
        conn->ack_num = seg_seq + 1;
        conn->state = TCP_STATE_SYN_RCVD;
        return 0;
    }
    if (conn->state == TCP_STATE_SYN_SENT &&
        (flags & TCP_FLAG_SYN) && (flags & TCP_FLAG_ACK)) {
        conn->peer_seq = seg_seq;
        conn->ack_num = seg_seq + 1;
        conn->seq_num = seg_ack;
        conn->state = TCP_STATE_ESTABLISHED;
        return 0;
    }
    if (conn->state == TCP_STATE_SYN_RCVD && (flags & TCP_FLAG_ACK)) {
        conn->seq_num = seg_ack;
        conn->state = TCP_STATE_ESTABLISHED;
        return 0;
    }
    if (conn->state == TCP_STATE_ESTABLISHED && (flags & TCP_FLAG_FIN)) {
        conn->ack_num = seg_seq + 1;
        conn->state = TCP_STATE_CLOSE_WAIT;
        return 0;
    }
    if (flags & TCP_FLAG_RST) {
        conn->state = TCP_STATE_CLOSED;
        return 0;
    }
    return -1;
}

int tcp_close(tcp_connection_t *conn) {
    if (conn->state == TCP_STATE_ESTABLISHED) {
        conn->state = TCP_STATE_FIN_WAIT1;
        return 0;
    }
    if (conn->state == TCP_STATE_CLOSE_WAIT) {
        conn->state = TCP_STATE_LAST_ACK;
        return 0;
    }
    return -1;
}

int tcp_get_state(const tcp_connection_t *conn) {
    return conn->state;
}

int tcp_is_established(const tcp_connection_t *conn) {
    return conn->state == TCP_STATE_ESTABLISHED;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C327: TCP three-way handshake state machine should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C327: Output should not be empty");
    assert!(
        code.contains("fn tcp_init"),
        "C327: Should contain tcp_init function"
    );
    assert!(
        code.contains("fn tcp_handle_segment"),
        "C327: Should contain tcp_handle_segment function"
    );
    assert!(
        code.contains("fn tcp_close"),
        "C327: Should contain tcp_close function"
    );
}

#[test]
fn c328_http_request_line_parser() {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct {
    char method[16];
    char path[256];
    char version[16];
    int method_len;
    int path_len;
    int version_len;
    int valid;
} http_request_line_t;

int is_space(char c) {
    return c == ' ';
}

int is_token_char(char c) {
    return (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') ||
           (c >= '0' && c <= '9') || c == '-' || c == '_' || c == '.';
}

int http_parse_request_line(const char *line, int line_len,
                            http_request_line_t *req) {
    int pos = 0;
    int i;
    req->valid = 0;
    req->method_len = 0;
    req->path_len = 0;
    req->version_len = 0;

    for (i = pos; i < line_len && !is_space(line[i]); i++) {
        if (req->method_len < 15) {
            req->method[req->method_len] = line[i];
            req->method_len++;
        }
    }
    req->method[req->method_len] = 0;
    if (i >= line_len) return -1;
    pos = i + 1;

    for (i = pos; i < line_len && !is_space(line[i]); i++) {
        if (req->path_len < 255) {
            req->path[req->path_len] = line[i];
            req->path_len++;
        }
    }
    req->path[req->path_len] = 0;
    if (i >= line_len) return -2;
    pos = i + 1;

    for (i = pos; i < line_len && line[i] != '\r' && line[i] != '\n'; i++) {
        if (req->version_len < 15) {
            req->version[req->version_len] = line[i];
            req->version_len++;
        }
    }
    req->version[req->version_len] = 0;

    if (req->method_len == 0 || req->path_len == 0 || req->version_len == 0) {
        return -3;
    }
    req->valid = 1;
    return 0;
}

int http_is_get(const http_request_line_t *req) {
    return req->method[0] == 'G' && req->method[1] == 'E' &&
           req->method[2] == 'T' && req->method[3] == 0;
}

int http_is_post(const http_request_line_t *req) {
    return req->method[0] == 'P' && req->method[1] == 'O' &&
           req->method[2] == 'S' && req->method[3] == 'T' && req->method[4] == 0;
}

int http_is_http11(const http_request_line_t *req) {
    return req->version[0] == 'H' && req->version[5] == '1' &&
           req->version[7] == '1';
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C328: HTTP request line parser should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C328: Output should not be empty");
    assert!(
        code.contains("fn http_parse_request_line"),
        "C328: Should contain http_parse_request_line function"
    );
    assert!(
        code.contains("fn http_is_get"),
        "C328: Should contain http_is_get function"
    );
}

#[test]
fn c329_dns_query_builder_label_encoding() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

#define DNS_MAX_NAME 256
#define DNS_MAX_LABELS 16
#define DNS_TYPE_A     1
#define DNS_TYPE_AAAA 28
#define DNS_TYPE_CNAME 5
#define DNS_CLASS_IN   1

typedef struct {
    uint8_t buffer[512];
    int length;
    uint16_t query_id;
    uint16_t flags;
    uint16_t qdcount;
} dns_query_t;

void dns_query_init(dns_query_t *q, uint16_t id) {
    q->query_id = id;
    q->flags = 0x0100;
    q->qdcount = 0;
    q->length = 0;
    for (int i = 0; i < 12; i++) {
        q->buffer[i] = 0;
    }
    q->buffer[0] = (uint8_t)(id >> 8);
    q->buffer[1] = (uint8_t)(id & 0xFF);
    q->buffer[2] = 0x01;
    q->buffer[3] = 0x00;
    q->length = 12;
}

int dns_encode_name(uint8_t *buf, int max_len, const char *name, int name_len) {
    int pos = 0;
    int label_start = 0;
    int i;
    for (i = 0; i <= name_len; i++) {
        if (i == name_len || name[i] == '.') {
            int label_len = i - label_start;
            if (label_len > 63 || label_len == 0) return -1;
            if (pos + 1 + label_len >= max_len) return -2;
            buf[pos] = (uint8_t)label_len;
            pos++;
            for (int j = 0; j < label_len; j++) {
                buf[pos] = (uint8_t)name[label_start + j];
                pos++;
            }
            label_start = i + 1;
        }
    }
    if (pos >= max_len) return -2;
    buf[pos] = 0;
    pos++;
    return pos;
}

int dns_add_question(dns_query_t *q, const char *name, int name_len,
                     uint16_t qtype, uint16_t qclass) {
    int encoded = dns_encode_name(q->buffer + q->length,
                                  512 - q->length, name, name_len);
    if (encoded < 0) return encoded;
    q->length += encoded;
    if (q->length + 4 > 512) return -3;
    q->buffer[q->length] = (uint8_t)(qtype >> 8);
    q->buffer[q->length + 1] = (uint8_t)(qtype & 0xFF);
    q->buffer[q->length + 2] = (uint8_t)(qclass >> 8);
    q->buffer[q->length + 3] = (uint8_t)(qclass & 0xFF);
    q->length += 4;
    q->qdcount++;
    q->buffer[4] = (uint8_t)(q->qdcount >> 8);
    q->buffer[5] = (uint8_t)(q->qdcount & 0xFF);
    return 0;
}

int dns_get_length(const dns_query_t *q) {
    return q->length;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C329: DNS query builder with label encoding should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C329: Output should not be empty");
    assert!(
        code.contains("fn dns_query_init"),
        "C329: Should contain dns_query_init function"
    );
    assert!(
        code.contains("fn dns_encode_name"),
        "C329: Should contain dns_encode_name function"
    );
    assert!(
        code.contains("fn dns_add_question"),
        "C329: Should contain dns_add_question function"
    );
}

#[test]
fn c330_arp_table_timeout_expiry() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

#define ARP_TABLE_SIZE 64
#define ARP_TIMEOUT_SEC 300

typedef struct {
    uint32_t ip_addr;
    uint8_t mac_addr[6];
    uint32_t timestamp;
    int valid;
    int is_static;
} arp_entry_t;

typedef struct {
    arp_entry_t entries[64];
    int count;
    uint32_t lookups;
    uint32_t hits;
    uint32_t misses;
    uint32_t evictions;
} arp_table_t;

void arp_init(arp_table_t *t) {
    t->count = 0;
    t->lookups = 0;
    t->hits = 0;
    t->misses = 0;
    t->evictions = 0;
    for (int i = 0; i < 64; i++) {
        t->entries[i].valid = 0;
        t->entries[i].is_static = 0;
    }
}

int arp_lookup(arp_table_t *t, uint32_t ip, uint8_t *mac_out, uint32_t now) {
    t->lookups++;
    for (int i = 0; i < 64; i++) {
        if (!t->entries[i].valid) continue;
        if (t->entries[i].ip_addr != ip) continue;
        if (!t->entries[i].is_static &&
            (now - t->entries[i].timestamp) > ARP_TIMEOUT_SEC) {
            t->entries[i].valid = 0;
            t->count--;
            t->evictions++;
            continue;
        }
        for (int j = 0; j < 6; j++) {
            mac_out[j] = t->entries[i].mac_addr[j];
        }
        t->hits++;
        return i;
    }
    t->misses++;
    return -1;
}

int arp_insert(arp_table_t *t, uint32_t ip, const uint8_t *mac,
               uint32_t now, int is_static) {
    for (int i = 0; i < 64; i++) {
        if (t->entries[i].valid && t->entries[i].ip_addr == ip) {
            for (int j = 0; j < 6; j++) {
                t->entries[i].mac_addr[j] = mac[j];
            }
            t->entries[i].timestamp = now;
            t->entries[i].is_static = is_static;
            return i;
        }
    }
    for (int i = 0; i < 64; i++) {
        if (!t->entries[i].valid) {
            t->entries[i].ip_addr = ip;
            for (int j = 0; j < 6; j++) {
                t->entries[i].mac_addr[j] = mac[j];
            }
            t->entries[i].timestamp = now;
            t->entries[i].valid = 1;
            t->entries[i].is_static = is_static;
            t->count++;
            return i;
        }
    }
    return -1;
}

void arp_expire(arp_table_t *t, uint32_t now) {
    for (int i = 0; i < 64; i++) {
        if (!t->entries[i].valid) continue;
        if (t->entries[i].is_static) continue;
        if ((now - t->entries[i].timestamp) > ARP_TIMEOUT_SEC) {
            t->entries[i].valid = 0;
            t->count--;
            t->evictions++;
        }
    }
}

int arp_count(const arp_table_t *t) {
    return t->count;
}

uint32_t arp_hit_rate(const arp_table_t *t) {
    if (t->lookups == 0) return 0;
    return (t->hits * 100) / t->lookups;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C330: ARP table with timeout-based expiry should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C330: Output should not be empty");
    assert!(
        code.contains("fn arp_init"),
        "C330: Should contain arp_init function"
    );
    assert!(
        code.contains("fn arp_lookup"),
        "C330: Should contain arp_lookup function"
    );
    assert!(
        code.contains("fn arp_insert"),
        "C330: Should contain arp_insert function"
    );
    assert!(
        code.contains("fn arp_expire"),
        "C330: Should contain arp_expire function"
    );
}

// ============================================================================
// C331-C335: Application/Transport (DHCP, URL, WebSocket, ICMP, Routing)
// ============================================================================

#[test]
fn c331_dhcp_option_parser_tlv() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

#define DHCP_OPT_SUBNET    1
#define DHCP_OPT_ROUTER    3
#define DHCP_OPT_DNS       6
#define DHCP_OPT_LEASE    51
#define DHCP_OPT_MSG_TYPE 53
#define DHCP_OPT_SERVER   54
#define DHCP_OPT_END     255
#define DHCP_OPT_PAD       0

#define DHCP_MAX_OPTIONS 32

typedef struct {
    uint8_t type;
    uint8_t length;
    uint8_t value[256];
} dhcp_option_t;

typedef struct {
    dhcp_option_t options[32];
    int option_count;
    int parse_error;
} dhcp_options_t;

void dhcp_opts_init(dhcp_options_t *opts) {
    opts->option_count = 0;
    opts->parse_error = 0;
}

int dhcp_parse_options(dhcp_options_t *opts, const uint8_t *data, int data_len) {
    int pos = 0;
    opts->option_count = 0;
    opts->parse_error = 0;
    while (pos < data_len) {
        uint8_t type = data[pos];
        if (type == DHCP_OPT_END) break;
        if (type == DHCP_OPT_PAD) { pos++; continue; }
        pos++;
        if (pos >= data_len) { opts->parse_error = -1; return -1; }
        uint8_t length = data[pos];
        pos++;
        if (pos + length > data_len) { opts->parse_error = -2; return -2; }
        if (opts->option_count < 32) {
            opts->options[opts->option_count].type = type;
            opts->options[opts->option_count].length = length;
            for (int i = 0; i < length && i < 256; i++) {
                opts->options[opts->option_count].value[i] = data[pos + i];
            }
            opts->option_count++;
        }
        pos += length;
    }
    return opts->option_count;
}

int dhcp_find_option(const dhcp_options_t *opts, uint8_t type) {
    for (int i = 0; i < opts->option_count; i++) {
        if (opts->options[i].type == type) return i;
    }
    return -1;
}

uint32_t dhcp_get_ip_option(const dhcp_options_t *opts, int idx) {
    if (idx < 0 || idx >= opts->option_count) return 0;
    if (opts->options[idx].length < 4) return 0;
    return ((uint32_t)opts->options[idx].value[0] << 24) |
           ((uint32_t)opts->options[idx].value[1] << 16) |
           ((uint32_t)opts->options[idx].value[2] << 8) |
           (uint32_t)opts->options[idx].value[3];
}

int dhcp_get_message_type(const dhcp_options_t *opts) {
    int idx = dhcp_find_option(opts, DHCP_OPT_MSG_TYPE);
    if (idx < 0) return -1;
    if (opts->options[idx].length < 1) return -2;
    return opts->options[idx].value[0];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C331: DHCP option parser TLV format should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C331: Output should not be empty");
    assert!(
        code.contains("fn dhcp_parse_options"),
        "C331: Should contain dhcp_parse_options function"
    );
    assert!(
        code.contains("fn dhcp_find_option"),
        "C331: Should contain dhcp_find_option function"
    );
    assert!(
        code.contains("fn dhcp_get_message_type"),
        "C331: Should contain dhcp_get_message_type function"
    );
}

#[test]
fn c332_url_percent_encoding_decoding() {
    let c_code = r#"
typedef unsigned char uint8_t;

int hex_digit_value(char c) {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'A' && c <= 'F') return c - 'A' + 10;
    if (c >= 'a' && c <= 'f') return c - 'a' + 10;
    return -1;
}

char to_hex_upper(int nibble) {
    if (nibble < 10) return (char)('0' + nibble);
    return (char)('A' + nibble - 10);
}

int is_unreserved(char c) {
    return (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') ||
           (c >= '0' && c <= '9') || c == '-' || c == '_' ||
           c == '.' || c == '~';
}

int url_encode(const char *input, int input_len, char *output, int max_out) {
    int out_pos = 0;
    for (int i = 0; i < input_len; i++) {
        if (is_unreserved(input[i])) {
            if (out_pos >= max_out) return -1;
            output[out_pos++] = input[i];
        } else {
            if (out_pos + 3 > max_out) return -1;
            output[out_pos++] = '%';
            output[out_pos++] = to_hex_upper((input[i] >> 4) & 0x0F);
            output[out_pos++] = to_hex_upper(input[i] & 0x0F);
        }
    }
    if (out_pos >= max_out) return -1;
    output[out_pos] = 0;
    return out_pos;
}

int url_decode(const char *input, int input_len, char *output, int max_out) {
    int out_pos = 0;
    for (int i = 0; i < input_len; i++) {
        if (input[i] == '%' && i + 2 < input_len) {
            int high = hex_digit_value(input[i + 1]);
            int low = hex_digit_value(input[i + 2]);
            if (high < 0 || low < 0) return -2;
            if (out_pos >= max_out) return -1;
            output[out_pos++] = (char)((high << 4) | low);
            i += 2;
        } else if (input[i] == '+') {
            if (out_pos >= max_out) return -1;
            output[out_pos++] = ' ';
        } else {
            if (out_pos >= max_out) return -1;
            output[out_pos++] = input[i];
        }
    }
    if (out_pos >= max_out) return -1;
    output[out_pos] = 0;
    return out_pos;
}

int url_needs_encoding(const char *input, int len) {
    for (int i = 0; i < len; i++) {
        if (!is_unreserved(input[i])) return 1;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C332: URL percent-encoding/decoding should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C332: Output should not be empty");
    assert!(
        code.contains("fn url_encode"),
        "C332: Should contain url_encode function"
    );
    assert!(
        code.contains("fn url_decode"),
        "C332: Should contain url_decode function"
    );
    assert!(
        code.contains("fn hex_digit_value"),
        "C332: Should contain hex_digit_value function"
    );
}

#[test]
fn c333_websocket_frame_parser() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

#define WS_OP_CONTINUATION 0x0
#define WS_OP_TEXT         0x1
#define WS_OP_BINARY       0x2
#define WS_OP_CLOSE        0x8
#define WS_OP_PING         0x9
#define WS_OP_PONG         0xA

typedef struct {
    int fin;
    int opcode;
    int masked;
    uint32_t payload_len;
    uint8_t mask_key[4];
    int header_len;
    int valid;
} ws_frame_header_t;

int ws_parse_frame(const uint8_t *data, int data_len, ws_frame_header_t *frame) {
    if (data_len < 2) return -1;
    frame->fin = (data[0] >> 7) & 1;
    frame->opcode = data[0] & 0x0F;
    frame->masked = (data[1] >> 7) & 1;
    uint32_t payload = data[1] & 0x7F;
    int pos = 2;

    if (payload == 126) {
        if (data_len < 4) return -2;
        payload = ((uint32_t)data[2] << 8) | (uint32_t)data[3];
        pos = 4;
    } else if (payload == 127) {
        if (data_len < 10) return -3;
        payload = ((uint32_t)data[6] << 24) | ((uint32_t)data[7] << 16) |
                  ((uint32_t)data[8] << 8) | (uint32_t)data[9];
        pos = 10;
    }

    frame->payload_len = payload;

    if (frame->masked) {
        if (pos + 4 > data_len) return -4;
        frame->mask_key[0] = data[pos];
        frame->mask_key[1] = data[pos + 1];
        frame->mask_key[2] = data[pos + 2];
        frame->mask_key[3] = data[pos + 3];
        pos += 4;
    }

    frame->header_len = pos;
    frame->valid = 1;
    return pos;
}

void ws_unmask_payload(uint8_t *payload, uint32_t len, const uint8_t *mask) {
    for (uint32_t i = 0; i < len; i++) {
        payload[i] ^= mask[i % 4];
    }
}

int ws_is_control_frame(int opcode) {
    return opcode >= 0x8;
}

int ws_frame_total_len(const ws_frame_header_t *frame) {
    return frame->header_len + (int)frame->payload_len;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C333: WebSocket frame parser should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C333: Output should not be empty");
    assert!(
        code.contains("fn ws_parse_frame"),
        "C333: Should contain ws_parse_frame function"
    );
    assert!(
        code.contains("fn ws_unmask_payload"),
        "C333: Should contain ws_unmask_payload function"
    );
}

#[test]
fn c334_icmp_echo_request_reply_handler() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

#define ICMP_ECHO_REQUEST 8
#define ICMP_ECHO_REPLY   0
#define ICMP_DEST_UNREACH 3
#define ICMP_TIME_EXCEEDED 11

typedef struct {
    uint8_t type;
    uint8_t code;
    uint16_t checksum;
    uint16_t identifier;
    uint16_t sequence;
} icmp_header_t;

typedef struct {
    uint16_t identifier;
    uint16_t next_sequence;
    uint32_t sent_count;
    uint32_t received_count;
    uint32_t error_count;
    uint32_t min_rtt_ms;
    uint32_t max_rtt_ms;
    uint32_t total_rtt_ms;
} icmp_ping_ctx_t;

void icmp_ping_init(icmp_ping_ctx_t *ctx, uint16_t id) {
    ctx->identifier = id;
    ctx->next_sequence = 1;
    ctx->sent_count = 0;
    ctx->received_count = 0;
    ctx->error_count = 0;
    ctx->min_rtt_ms = 0xFFFFFFFF;
    ctx->max_rtt_ms = 0;
    ctx->total_rtt_ms = 0;
}

int icmp_build_echo_request(icmp_ping_ctx_t *ctx, uint8_t *buf, int max_len) {
    if (max_len < 8) return -1;
    buf[0] = ICMP_ECHO_REQUEST;
    buf[1] = 0;
    buf[2] = 0;
    buf[3] = 0;
    buf[4] = (uint8_t)(ctx->identifier >> 8);
    buf[5] = (uint8_t)(ctx->identifier & 0xFF);
    buf[6] = (uint8_t)(ctx->next_sequence >> 8);
    buf[7] = (uint8_t)(ctx->next_sequence & 0xFF);
    ctx->next_sequence++;
    ctx->sent_count++;
    return 8;
}

int icmp_handle_reply(icmp_ping_ctx_t *ctx, const uint8_t *data,
                      int data_len, uint32_t rtt_ms) {
    if (data_len < 8) return -1;
    if (data[0] != ICMP_ECHO_REPLY) {
        ctx->error_count++;
        return -2;
    }
    uint16_t id = ((uint16_t)data[4] << 8) | (uint16_t)data[5];
    if (id != ctx->identifier) return -3;
    ctx->received_count++;
    ctx->total_rtt_ms += rtt_ms;
    if (rtt_ms < ctx->min_rtt_ms) ctx->min_rtt_ms = rtt_ms;
    if (rtt_ms > ctx->max_rtt_ms) ctx->max_rtt_ms = rtt_ms;
    return 0;
}

uint32_t icmp_avg_rtt(const icmp_ping_ctx_t *ctx) {
    if (ctx->received_count == 0) return 0;
    return ctx->total_rtt_ms / ctx->received_count;
}

uint32_t icmp_loss_percent(const icmp_ping_ctx_t *ctx) {
    if (ctx->sent_count == 0) return 0;
    return ((ctx->sent_count - ctx->received_count) * 100) / ctx->sent_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C334: ICMP echo request/reply handler should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C334: Output should not be empty");
    assert!(
        code.contains("fn icmp_ping_init"),
        "C334: Should contain icmp_ping_init function"
    );
    assert!(
        code.contains("fn icmp_build_echo_request"),
        "C334: Should contain icmp_build_echo_request function"
    );
    assert!(
        code.contains("fn icmp_handle_reply"),
        "C334: Should contain icmp_handle_reply function"
    );
    assert!(
        code.contains("fn icmp_loss_percent"),
        "C334: Should contain icmp_loss_percent function"
    );
}

#[test]
fn c335_routing_table_longest_prefix_match() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define RT_MAX_ROUTES 128
#define RT_METRIC_DIRECT 0
#define RT_METRIC_STATIC 1

typedef struct {
    uint32_t network;
    uint32_t netmask;
    uint32_t gateway;
    int prefix_len;
    int metric;
    int interface_id;
    int active;
    uint32_t hit_count;
} route_entry_t;

typedef struct {
    route_entry_t routes[128];
    int route_count;
    uint32_t total_lookups;
    uint32_t default_gateway;
} routing_table_t;

void rt_init(routing_table_t *rt) {
    rt->route_count = 0;
    rt->total_lookups = 0;
    rt->default_gateway = 0;
}

uint32_t rt_prefix_to_mask(int prefix_len) {
    if (prefix_len <= 0) return 0;
    if (prefix_len >= 32) return 0xFFFFFFFF;
    return ~((1U << (32 - prefix_len)) - 1);
}

int rt_add_route(routing_table_t *rt, uint32_t network, int prefix_len,
                 uint32_t gateway, int metric, int iface) {
    if (rt->route_count >= 128) return -1;
    if (prefix_len < 0 || prefix_len > 32) return -2;
    int idx = rt->route_count;
    rt->routes[idx].network = network;
    rt->routes[idx].netmask = rt_prefix_to_mask(prefix_len);
    rt->routes[idx].gateway = gateway;
    rt->routes[idx].prefix_len = prefix_len;
    rt->routes[idx].metric = metric;
    rt->routes[idx].interface_id = iface;
    rt->routes[idx].active = 1;
    rt->routes[idx].hit_count = 0;
    rt->route_count++;
    return idx;
}

int rt_lookup(routing_table_t *rt, uint32_t dest_ip) {
    int best = -1;
    int best_prefix = -1;
    int best_metric = 0x7FFFFFFF;
    rt->total_lookups++;
    for (int i = 0; i < rt->route_count; i++) {
        if (!rt->routes[i].active) continue;
        if ((dest_ip & rt->routes[i].netmask) ==
            (rt->routes[i].network & rt->routes[i].netmask)) {
            if (rt->routes[i].prefix_len > best_prefix ||
                (rt->routes[i].prefix_len == best_prefix &&
                 rt->routes[i].metric < best_metric)) {
                best = i;
                best_prefix = rt->routes[i].prefix_len;
                best_metric = rt->routes[i].metric;
            }
        }
    }
    if (best >= 0) {
        rt->routes[best].hit_count++;
    }
    return best;
}

int rt_delete_route(routing_table_t *rt, int route_id) {
    if (route_id < 0 || route_id >= rt->route_count) return -1;
    rt->routes[route_id].active = 0;
    return 0;
}

int rt_active_count(const routing_table_t *rt) {
    int count = 0;
    for (int i = 0; i < rt->route_count; i++) {
        if (rt->routes[i].active) count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C335: Routing table with longest-prefix matching should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C335: Output should not be empty");
    assert!(
        code.contains("fn rt_init"),
        "C335: Should contain rt_init function"
    );
    assert!(
        code.contains("fn rt_lookup"),
        "C335: Should contain rt_lookup function"
    );
    assert!(
        code.contains("fn rt_add_route"),
        "C335: Should contain rt_add_route function"
    );
    assert!(
        code.contains("fn rt_prefix_to_mask"),
        "C335: Should contain rt_prefix_to_mask function"
    );
}

// ============================================================================
// C336-C340: Security/Transport (TLS, MQTT, gRPC, Ethernet, NAT)
// ============================================================================

#[test]
fn c336_tls_record_layer_parser() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

#define TLS_CT_CHANGE_CIPHER  20
#define TLS_CT_ALERT          21
#define TLS_CT_HANDSHAKE      22
#define TLS_CT_APPLICATION    23

#define TLS_VER_10 0x0301
#define TLS_VER_11 0x0302
#define TLS_VER_12 0x0303
#define TLS_VER_13 0x0304

#define TLS_MAX_RECORD 16384

typedef struct {
    uint8_t content_type;
    uint16_t version;
    uint16_t length;
    int valid;
} tls_record_t;

typedef struct {
    int records_parsed;
    int handshake_count;
    int app_data_count;
    int alert_count;
    int error_count;
    uint16_t negotiated_version;
} tls_parser_ctx_t;

void tls_parser_init(tls_parser_ctx_t *ctx) {
    ctx->records_parsed = 0;
    ctx->handshake_count = 0;
    ctx->app_data_count = 0;
    ctx->alert_count = 0;
    ctx->error_count = 0;
    ctx->negotiated_version = 0;
}

int tls_parse_record(const uint8_t *data, int data_len, tls_record_t *rec) {
    if (data_len < 5) return -1;
    rec->content_type = data[0];
    rec->version = ((uint16_t)data[1] << 8) | (uint16_t)data[2];
    rec->length = ((uint16_t)data[3] << 8) | (uint16_t)data[4];
    rec->valid = 0;
    if (rec->content_type < TLS_CT_CHANGE_CIPHER ||
        rec->content_type > TLS_CT_APPLICATION) return -2;
    if (rec->length > TLS_MAX_RECORD) return -3;
    if (rec->version < TLS_VER_10 || rec->version > TLS_VER_13) return -4;
    if (5 + rec->length > data_len) return -5;
    rec->valid = 1;
    return 5 + rec->length;
}

int tls_process_record(tls_parser_ctx_t *ctx, const uint8_t *data, int data_len) {
    tls_record_t rec;
    int consumed = tls_parse_record(data, data_len, &rec);
    if (consumed < 0) {
        ctx->error_count++;
        return consumed;
    }
    ctx->records_parsed++;
    if (rec.content_type == TLS_CT_HANDSHAKE) ctx->handshake_count++;
    if (rec.content_type == TLS_CT_APPLICATION) ctx->app_data_count++;
    if (rec.content_type == TLS_CT_ALERT) ctx->alert_count++;
    if (ctx->negotiated_version == 0 || rec.version > ctx->negotiated_version) {
        ctx->negotiated_version = rec.version;
    }
    return consumed;
}

int tls_is_valid_version(uint16_t version) {
    return version >= TLS_VER_10 && version <= TLS_VER_13;
}

int tls_get_records_parsed(const tls_parser_ctx_t *ctx) {
    return ctx->records_parsed;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C336: TLS record layer parser should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C336: Output should not be empty");
    assert!(
        code.contains("fn tls_parser_init"),
        "C336: Should contain tls_parser_init function"
    );
    assert!(
        code.contains("fn tls_parse_record"),
        "C336: Should contain tls_parse_record function"
    );
    assert!(
        code.contains("fn tls_process_record"),
        "C336: Should contain tls_process_record function"
    );
}

#[test]
fn c337_mqtt_topic_filter_matching() {
    let c_code = r#"
typedef unsigned char uint8_t;

int mqtt_topic_match(const char *filter, int flen,
                     const char *topic, int tlen) {
    int fi = 0;
    int ti = 0;
    while (fi < flen && ti < tlen) {
        if (filter[fi] == '#') {
            return 1;
        }
        if (filter[fi] == '+') {
            while (ti < tlen && topic[ti] != '/') {
                ti++;
            }
            fi++;
            continue;
        }
        if (filter[fi] != topic[ti]) {
            return 0;
        }
        fi++;
        ti++;
    }
    if (fi == flen && ti == tlen) return 1;
    if (fi < flen && filter[fi] == '#') return 1;
    if (fi < flen && filter[fi] == '/' && fi + 1 < flen && filter[fi + 1] == '#') return 1;
    return 0;
}

int mqtt_validate_topic(const char *topic, int len) {
    if (len == 0) return -1;
    for (int i = 0; i < len; i++) {
        if (topic[i] == '#' || topic[i] == '+') return -2;
        if (topic[i] == 0) return -3;
    }
    return 0;
}

int mqtt_validate_filter(const char *filter, int len) {
    if (len == 0) return -1;
    for (int i = 0; i < len; i++) {
        if (filter[i] == '#') {
            if (i != len - 1) return -2;
            if (i > 0 && filter[i - 1] != '/') return -3;
        }
        if (filter[i] == '+') {
            if (i > 0 && filter[i - 1] != '/') return -4;
            if (i + 1 < len && filter[i + 1] != '/') return -5;
        }
    }
    return 0;
}

int mqtt_count_levels(const char *topic, int len) {
    int count = 1;
    for (int i = 0; i < len; i++) {
        if (topic[i] == '/') count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C337: MQTT topic filter matching should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C337: Output should not be empty");
    assert!(
        code.contains("fn mqtt_topic_match"),
        "C337: Should contain mqtt_topic_match function"
    );
    assert!(
        code.contains("fn mqtt_validate_filter"),
        "C337: Should contain mqtt_validate_filter function"
    );
    assert!(
        code.contains("fn mqtt_count_levels"),
        "C337: Should contain mqtt_count_levels function"
    );
}

#[test]
fn c338_grpc_varint_encoding_decoding() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

int varint_encode(uint64_t value, uint8_t *buf, int max_len) {
    int pos = 0;
    while (value > 0x7F) {
        if (pos >= max_len) return -1;
        buf[pos] = (uint8_t)((value & 0x7F) | 0x80);
        value >>= 7;
        pos++;
    }
    if (pos >= max_len) return -1;
    buf[pos] = (uint8_t)(value & 0x7F);
    pos++;
    return pos;
}

int varint_decode(const uint8_t *buf, int buf_len, uint64_t *value) {
    *value = 0;
    int shift = 0;
    int pos = 0;
    while (pos < buf_len) {
        uint64_t byte_val = buf[pos];
        *value |= (byte_val & 0x7F) << shift;
        if (!(byte_val & 0x80)) {
            return pos + 1;
        }
        shift += 7;
        pos++;
        if (shift >= 64) return -2;
    }
    return -1;
}

int varint_encoded_size(uint64_t value) {
    int size = 1;
    while (value > 0x7F) {
        size++;
        value >>= 7;
    }
    return size;
}

uint32_t zigzag_encode(int value) {
    return (uint32_t)((value << 1) ^ (value >> 31));
}

int zigzag_decode(uint32_t value) {
    return (int)((value >> 1) ^ -(int)(value & 1));
}

int grpc_frame_header_size(void) {
    return 5;
}

int grpc_encode_length_prefix(uint8_t *buf, int max_len,
                              int compressed, uint32_t msg_len) {
    if (max_len < 5) return -1;
    buf[0] = (uint8_t)(compressed ? 1 : 0);
    buf[1] = (uint8_t)((msg_len >> 24) & 0xFF);
    buf[2] = (uint8_t)((msg_len >> 16) & 0xFF);
    buf[3] = (uint8_t)((msg_len >> 8) & 0xFF);
    buf[4] = (uint8_t)(msg_len & 0xFF);
    return 5;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C338: gRPC varint encoding/decoding should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C338: Output should not be empty");
    assert!(
        code.contains("fn varint_encode"),
        "C338: Should contain varint_encode function"
    );
    assert!(
        code.contains("fn varint_decode"),
        "C338: Should contain varint_decode function"
    );
    assert!(
        code.contains("fn zigzag_encode"),
        "C338: Should contain zigzag_encode function"
    );
}

#[test]
fn c339_ethernet_frame_builder_vlan_tag() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

#define ETH_ALEN 6
#define ETH_TYPE_IP   0x0800
#define ETH_TYPE_ARP  0x0806
#define ETH_TYPE_VLAN 0x8100
#define ETH_MTU 1500

typedef struct {
    uint8_t dst_mac[6];
    uint8_t src_mac[6];
    uint16_t ethertype;
    int has_vlan;
    uint16_t vlan_id;
    uint8_t vlan_prio;
    int frame_len;
} eth_frame_t;

void eth_init(eth_frame_t *f) {
    for (int i = 0; i < 6; i++) {
        f->dst_mac[i] = 0;
        f->src_mac[i] = 0;
    }
    f->ethertype = 0;
    f->has_vlan = 0;
    f->vlan_id = 0;
    f->vlan_prio = 0;
    f->frame_len = 0;
}

void eth_set_dst(eth_frame_t *f, const uint8_t *mac) {
    for (int i = 0; i < 6; i++) f->dst_mac[i] = mac[i];
}

void eth_set_src(eth_frame_t *f, const uint8_t *mac) {
    for (int i = 0; i < 6; i++) f->src_mac[i] = mac[i];
}

void eth_set_vlan(eth_frame_t *f, uint16_t vlan_id, uint8_t prio) {
    f->has_vlan = 1;
    f->vlan_id = vlan_id & 0x0FFF;
    f->vlan_prio = prio & 0x07;
}

int eth_build_header(const eth_frame_t *f, uint8_t *buf, int max_len) {
    int pos = 0;
    if (max_len < 14) return -1;
    for (int i = 0; i < 6; i++) buf[pos++] = f->dst_mac[i];
    for (int i = 0; i < 6; i++) buf[pos++] = f->src_mac[i];
    if (f->has_vlan) {
        if (max_len < 18) return -2;
        buf[pos++] = (uint8_t)(ETH_TYPE_VLAN >> 8);
        buf[pos++] = (uint8_t)(ETH_TYPE_VLAN & 0xFF);
        uint16_t tci = ((uint16_t)f->vlan_prio << 13) | f->vlan_id;
        buf[pos++] = (uint8_t)(tci >> 8);
        buf[pos++] = (uint8_t)(tci & 0xFF);
    }
    buf[pos++] = (uint8_t)(f->ethertype >> 8);
    buf[pos++] = (uint8_t)(f->ethertype & 0xFF);
    return pos;
}

int eth_is_broadcast(const uint8_t *mac) {
    for (int i = 0; i < 6; i++) {
        if (mac[i] != 0xFF) return 0;
    }
    return 1;
}

int eth_is_multicast(const uint8_t *mac) {
    return (mac[0] & 0x01) != 0;
}

int eth_header_len(const eth_frame_t *f) {
    return f->has_vlan ? 18 : 14;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C339: Ethernet frame builder with VLAN tag should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C339: Output should not be empty");
    assert!(
        code.contains("fn eth_init"),
        "C339: Should contain eth_init function"
    );
    assert!(
        code.contains("fn eth_build_header"),
        "C339: Should contain eth_build_header function"
    );
    assert!(
        code.contains("fn eth_set_vlan"),
        "C339: Should contain eth_set_vlan function"
    );
}

#[test]
fn c340_nat_translation_table() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;

#define NAT_TABLE_SIZE 256
#define NAT_TIMEOUT 120

typedef struct {
    uint32_t internal_ip;
    uint16_t internal_port;
    uint32_t external_ip;
    uint16_t external_port;
    uint32_t remote_ip;
    uint16_t remote_port;
    uint32_t timestamp;
    int active;
    uint32_t packet_count;
    uint32_t byte_count;
} nat_entry_t;

typedef struct {
    nat_entry_t entries[256];
    int count;
    uint16_t next_port;
    uint32_t external_ip;
    uint32_t total_translations;
    uint32_t total_expired;
} nat_table_t;

void nat_init(nat_table_t *nat, uint32_t ext_ip) {
    nat->count = 0;
    nat->next_port = 10000;
    nat->external_ip = ext_ip;
    nat->total_translations = 0;
    nat->total_expired = 0;
    for (int i = 0; i < 256; i++) {
        nat->entries[i].active = 0;
    }
}

int nat_find_outbound(const nat_table_t *nat, uint32_t int_ip,
                      uint16_t int_port, uint32_t rem_ip, uint16_t rem_port) {
    for (int i = 0; i < 256; i++) {
        if (!nat->entries[i].active) continue;
        if (nat->entries[i].internal_ip == int_ip &&
            nat->entries[i].internal_port == int_port &&
            nat->entries[i].remote_ip == rem_ip &&
            nat->entries[i].remote_port == rem_port) {
            return i;
        }
    }
    return -1;
}

int nat_find_inbound(const nat_table_t *nat, uint16_t ext_port,
                     uint32_t rem_ip, uint16_t rem_port) {
    for (int i = 0; i < 256; i++) {
        if (!nat->entries[i].active) continue;
        if (nat->entries[i].external_port == ext_port &&
            nat->entries[i].remote_ip == rem_ip &&
            nat->entries[i].remote_port == rem_port) {
            return i;
        }
    }
    return -1;
}

int nat_create_mapping(nat_table_t *nat, uint32_t int_ip, uint16_t int_port,
                       uint32_t rem_ip, uint16_t rem_port, uint32_t now) {
    for (int i = 0; i < 256; i++) {
        if (!nat->entries[i].active) {
            nat->entries[i].internal_ip = int_ip;
            nat->entries[i].internal_port = int_port;
            nat->entries[i].external_ip = nat->external_ip;
            nat->entries[i].external_port = nat->next_port;
            nat->entries[i].remote_ip = rem_ip;
            nat->entries[i].remote_port = rem_port;
            nat->entries[i].timestamp = now;
            nat->entries[i].active = 1;
            nat->entries[i].packet_count = 0;
            nat->entries[i].byte_count = 0;
            nat->next_port++;
            nat->count++;
            nat->total_translations++;
            return i;
        }
    }
    return -1;
}

void nat_expire(nat_table_t *nat, uint32_t now) {
    for (int i = 0; i < 256; i++) {
        if (!nat->entries[i].active) continue;
        if ((now - nat->entries[i].timestamp) > NAT_TIMEOUT) {
            nat->entries[i].active = 0;
            nat->count--;
            nat->total_expired++;
        }
    }
}

int nat_active_count(const nat_table_t *nat) {
    return nat->count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C340: NAT translation table should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C340: Output should not be empty");
    assert!(
        code.contains("fn nat_init"),
        "C340: Should contain nat_init function"
    );
    assert!(
        code.contains("fn nat_find_outbound"),
        "C340: Should contain nat_find_outbound function"
    );
    assert!(
        code.contains("fn nat_create_mapping"),
        "C340: Should contain nat_create_mapping function"
    );
    assert!(
        code.contains("fn nat_expire"),
        "C340: Should contain nat_expire function"
    );
}

// ============================================================================
// C341-C345: Modern Protocols (QUIC, BGP, SNMP, Conntrack, Chunked)
// ============================================================================

#[test]
fn c341_quic_variable_length_integer_encoding() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

int quic_varint_encode(uint64_t value, uint8_t *buf, int max_len) {
    if (value <= 63) {
        if (max_len < 1) return -1;
        buf[0] = (uint8_t)value;
        return 1;
    }
    if (value <= 16383) {
        if (max_len < 2) return -1;
        buf[0] = (uint8_t)(0x40 | (value >> 8));
        buf[1] = (uint8_t)(value & 0xFF);
        return 2;
    }
    if (value <= 1073741823ULL) {
        if (max_len < 4) return -1;
        buf[0] = (uint8_t)(0x80 | (value >> 24));
        buf[1] = (uint8_t)((value >> 16) & 0xFF);
        buf[2] = (uint8_t)((value >> 8) & 0xFF);
        buf[3] = (uint8_t)(value & 0xFF);
        return 4;
    }
    if (max_len < 8) return -1;
    buf[0] = (uint8_t)(0xC0 | (value >> 56));
    buf[1] = (uint8_t)((value >> 48) & 0xFF);
    buf[2] = (uint8_t)((value >> 40) & 0xFF);
    buf[3] = (uint8_t)((value >> 32) & 0xFF);
    buf[4] = (uint8_t)((value >> 24) & 0xFF);
    buf[5] = (uint8_t)((value >> 16) & 0xFF);
    buf[6] = (uint8_t)((value >> 8) & 0xFF);
    buf[7] = (uint8_t)(value & 0xFF);
    return 8;
}

int quic_varint_decode(const uint8_t *buf, int buf_len, uint64_t *value) {
    if (buf_len < 1) return -1;
    int prefix = buf[0] >> 6;
    int len = 1 << prefix;
    if (buf_len < len) return -2;
    *value = buf[0] & 0x3F;
    for (int i = 1; i < len; i++) {
        *value = (*value << 8) | buf[i];
    }
    return len;
}

int quic_varint_size(uint64_t value) {
    if (value <= 63) return 1;
    if (value <= 16383) return 2;
    if (value <= 1073741823ULL) return 4;
    return 8;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C341: QUIC variable-length integer encoding should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C341: Output should not be empty");
    assert!(
        code.contains("fn quic_varint_encode"),
        "C341: Should contain quic_varint_encode function"
    );
    assert!(
        code.contains("fn quic_varint_decode"),
        "C341: Should contain quic_varint_decode function"
    );
    assert!(
        code.contains("fn quic_varint_size"),
        "C341: Should contain quic_varint_size function"
    );
}

#[test]
fn c342_bgp_path_attribute_parser() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

#define BGP_ATTR_ORIGIN     1
#define BGP_ATTR_AS_PATH    2
#define BGP_ATTR_NEXT_HOP   3
#define BGP_ATTR_MED        4
#define BGP_ATTR_LOCAL_PREF 5
#define BGP_ATTR_COMMUNITY  8

#define BGP_MAX_ATTRS 32
#define BGP_MAX_ATTR_LEN 256

typedef struct {
    uint8_t flags;
    uint8_t type_code;
    uint16_t length;
    uint8_t value[256];
} bgp_path_attr_t;

typedef struct {
    bgp_path_attr_t attrs[32];
    int attr_count;
    int parse_error;
} bgp_update_t;

void bgp_update_init(bgp_update_t *u) {
    u->attr_count = 0;
    u->parse_error = 0;
}

int bgp_parse_attrs(bgp_update_t *u, const uint8_t *data, int data_len) {
    int pos = 0;
    u->attr_count = 0;
    u->parse_error = 0;
    while (pos < data_len && u->attr_count < 32) {
        if (pos + 2 > data_len) { u->parse_error = -1; return -1; }
        uint8_t flags = data[pos];
        uint8_t type_code = data[pos + 1];
        pos += 2;
        uint16_t length;
        if (flags & 0x10) {
            if (pos + 2 > data_len) { u->parse_error = -2; return -2; }
            length = ((uint16_t)data[pos] << 8) | (uint16_t)data[pos + 1];
            pos += 2;
        } else {
            if (pos + 1 > data_len) { u->parse_error = -3; return -3; }
            length = data[pos];
            pos += 1;
        }
        if (pos + length > data_len) { u->parse_error = -4; return -4; }
        u->attrs[u->attr_count].flags = flags;
        u->attrs[u->attr_count].type_code = type_code;
        u->attrs[u->attr_count].length = length;
        for (int i = 0; i < length && i < 256; i++) {
            u->attrs[u->attr_count].value[i] = data[pos + i];
        }
        u->attr_count++;
        pos += length;
    }
    return u->attr_count;
}

int bgp_find_attr(const bgp_update_t *u, uint8_t type_code) {
    for (int i = 0; i < u->attr_count; i++) {
        if (u->attrs[i].type_code == type_code) return i;
    }
    return -1;
}

uint32_t bgp_get_next_hop(const bgp_update_t *u) {
    int idx = bgp_find_attr(u, BGP_ATTR_NEXT_HOP);
    if (idx < 0 || u->attrs[idx].length < 4) return 0;
    return ((uint32_t)u->attrs[idx].value[0] << 24) |
           ((uint32_t)u->attrs[idx].value[1] << 16) |
           ((uint32_t)u->attrs[idx].value[2] << 8) |
           (uint32_t)u->attrs[idx].value[3];
}

int bgp_get_origin(const bgp_update_t *u) {
    int idx = bgp_find_attr(u, BGP_ATTR_ORIGIN);
    if (idx < 0 || u->attrs[idx].length < 1) return -1;
    return u->attrs[idx].value[0];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C342: BGP path attribute parser should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C342: Output should not be empty");
    assert!(
        code.contains("fn bgp_parse_attrs"),
        "C342: Should contain bgp_parse_attrs function"
    );
    assert!(
        code.contains("fn bgp_find_attr"),
        "C342: Should contain bgp_find_attr function"
    );
    assert!(
        code.contains("fn bgp_get_next_hop"),
        "C342: Should contain bgp_get_next_hop function"
    );
}

#[test]
fn c343_snmp_oid_encoding_ber() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

#define OID_MAX_COMPONENTS 32

typedef struct {
    uint32_t components[32];
    int length;
} snmp_oid_t;

void oid_init(snmp_oid_t *oid) {
    oid->length = 0;
}

int oid_append(snmp_oid_t *oid, uint32_t component) {
    if (oid->length >= 32) return -1;
    oid->components[oid->length] = component;
    oid->length++;
    return 0;
}

int oid_encode_ber(const snmp_oid_t *oid, uint8_t *buf, int max_len) {
    if (oid->length < 2) return -1;
    int pos = 0;
    if (pos >= max_len) return -2;
    buf[pos++] = (uint8_t)(oid->components[0] * 40 + oid->components[1]);
    for (int i = 2; i < oid->length; i++) {
        uint32_t val = oid->components[i];
        uint8_t temp[5];
        int temp_len = 0;
        temp[temp_len++] = (uint8_t)(val & 0x7F);
        val >>= 7;
        while (val > 0) {
            temp[temp_len++] = (uint8_t)((val & 0x7F) | 0x80);
            val >>= 7;
        }
        for (int j = temp_len - 1; j >= 0; j--) {
            if (pos >= max_len) return -2;
            buf[pos++] = temp[j];
        }
    }
    return pos;
}

int oid_decode_ber(snmp_oid_t *oid, const uint8_t *buf, int buf_len) {
    if (buf_len < 1) return -1;
    oid->length = 0;
    oid->components[0] = buf[0] / 40;
    oid->components[1] = buf[0] % 40;
    oid->length = 2;
    int pos = 1;
    while (pos < buf_len) {
        uint32_t val = 0;
        int byte_count = 0;
        while (pos < buf_len) {
            val = (val << 7) | (buf[pos] & 0x7F);
            byte_count++;
            if (!(buf[pos] & 0x80)) { pos++; break; }
            pos++;
            if (byte_count > 4) return -2;
        }
        if (oid->length >= 32) return -3;
        oid->components[oid->length] = val;
        oid->length++;
    }
    return oid->length;
}

int oid_compare(const snmp_oid_t *a, const snmp_oid_t *b) {
    int min_len = a->length < b->length ? a->length : b->length;
    for (int i = 0; i < min_len; i++) {
        if (a->components[i] < b->components[i]) return -1;
        if (a->components[i] > b->components[i]) return 1;
    }
    if (a->length < b->length) return -1;
    if (a->length > b->length) return 1;
    return 0;
}

int oid_is_prefix(const snmp_oid_t *prefix, const snmp_oid_t *oid) {
    if (prefix->length > oid->length) return 0;
    for (int i = 0; i < prefix->length; i++) {
        if (prefix->components[i] != oid->components[i]) return 0;
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C343: SNMP OID encoding BER should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C343: Output should not be empty");
    assert!(
        code.contains("fn oid_encode_ber"),
        "C343: Should contain oid_encode_ber function"
    );
    assert!(
        code.contains("fn oid_decode_ber"),
        "C343: Should contain oid_decode_ber function"
    );
    assert!(
        code.contains("fn oid_compare"),
        "C343: Should contain oid_compare function"
    );
}

#[test]
fn c344_connection_tracker_five_tuple_hash() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

#define CT_TABLE_SIZE 1024
#define CT_HASH_MASK 1023
#define CT_TIMEOUT 300

typedef struct {
    uint32_t src_ip;
    uint32_t dst_ip;
    uint16_t src_port;
    uint16_t dst_port;
    uint8_t protocol;
    int state;
    uint32_t timestamp;
    uint32_t packet_count;
    uint32_t byte_count;
    int active;
} ct_entry_t;

typedef struct {
    ct_entry_t entries[1024];
    int count;
    uint32_t total_tracked;
    uint32_t total_expired;
    uint32_t hash_collisions;
} ct_table_t;

uint32_t ct_hash(uint32_t src_ip, uint32_t dst_ip,
                 uint16_t src_port, uint16_t dst_port, uint8_t proto) {
    uint32_t h = src_ip;
    h ^= dst_ip;
    h ^= ((uint32_t)src_port << 16) | (uint32_t)dst_port;
    h ^= (uint32_t)proto;
    h = (h ^ (h >> 16)) * 0x45d9f3b;
    h = (h ^ (h >> 16)) * 0x45d9f3b;
    h = h ^ (h >> 16);
    return h & CT_HASH_MASK;
}

void ct_init(ct_table_t *ct) {
    ct->count = 0;
    ct->total_tracked = 0;
    ct->total_expired = 0;
    ct->hash_collisions = 0;
    for (int i = 0; i < 1024; i++) {
        ct->entries[i].active = 0;
    }
}

int ct_lookup(const ct_table_t *ct, uint32_t src_ip, uint32_t dst_ip,
              uint16_t src_port, uint16_t dst_port, uint8_t proto) {
    uint32_t idx = ct_hash(src_ip, dst_ip, src_port, dst_port, proto);
    for (int probe = 0; probe < 16; probe++) {
        int pos = (int)((idx + (uint32_t)probe) & CT_HASH_MASK);
        if (!ct->entries[pos].active) return -1;
        if (ct->entries[pos].src_ip == src_ip &&
            ct->entries[pos].dst_ip == dst_ip &&
            ct->entries[pos].src_port == src_port &&
            ct->entries[pos].dst_port == dst_port &&
            ct->entries[pos].protocol == proto) {
            return pos;
        }
    }
    return -1;
}

int ct_insert(ct_table_t *ct, uint32_t src_ip, uint32_t dst_ip,
              uint16_t src_port, uint16_t dst_port, uint8_t proto,
              uint32_t now) {
    uint32_t idx = ct_hash(src_ip, dst_ip, src_port, dst_port, proto);
    for (int probe = 0; probe < 16; probe++) {
        int pos = (int)((idx + (uint32_t)probe) & CT_HASH_MASK);
        if (!ct->entries[pos].active) {
            ct->entries[pos].src_ip = src_ip;
            ct->entries[pos].dst_ip = dst_ip;
            ct->entries[pos].src_port = src_port;
            ct->entries[pos].dst_port = dst_port;
            ct->entries[pos].protocol = proto;
            ct->entries[pos].state = 0;
            ct->entries[pos].timestamp = now;
            ct->entries[pos].packet_count = 1;
            ct->entries[pos].byte_count = 0;
            ct->entries[pos].active = 1;
            ct->count++;
            ct->total_tracked++;
            return pos;
        }
        if (probe > 0) ct->hash_collisions++;
    }
    return -1;
}

void ct_expire(ct_table_t *ct, uint32_t now) {
    for (int i = 0; i < 1024; i++) {
        if (!ct->entries[i].active) continue;
        if ((now - ct->entries[i].timestamp) > CT_TIMEOUT) {
            ct->entries[i].active = 0;
            ct->count--;
            ct->total_expired++;
        }
    }
}

int ct_active_count(const ct_table_t *ct) {
    return ct->count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C344: Connection tracker with 5-tuple hash should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C344: Output should not be empty");
    assert!(
        code.contains("fn ct_hash"),
        "C344: Should contain ct_hash function"
    );
    assert!(
        code.contains("fn ct_init"),
        "C344: Should contain ct_init function"
    );
    assert!(
        code.contains("fn ct_lookup"),
        "C344: Should contain ct_lookup function"
    );
    assert!(
        code.contains("fn ct_insert"),
        "C344: Should contain ct_insert function"
    );
}

#[test]
fn c345_http_chunked_transfer_encoding_parser() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

#define CHUNK_STATE_SIZE   0
#define CHUNK_STATE_DATA   1
#define CHUNK_STATE_TRAIL  2
#define CHUNK_STATE_DONE   3
#define CHUNK_STATE_ERROR  4

typedef struct {
    int state;
    uint32_t chunk_size;
    uint32_t chunk_read;
    uint32_t total_bytes;
    int chunk_count;
    int parse_error;
} chunked_parser_t;

void chunked_init(chunked_parser_t *p) {
    p->state = CHUNK_STATE_SIZE;
    p->chunk_size = 0;
    p->chunk_read = 0;
    p->total_bytes = 0;
    p->chunk_count = 0;
    p->parse_error = 0;
}

int hex_val(char c) {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'f') return c - 'a' + 10;
    if (c >= 'A' && c <= 'F') return c - 'A' + 10;
    return -1;
}

int chunked_parse(chunked_parser_t *p, const uint8_t *data, int data_len,
                  uint8_t *output, int max_out) {
    int in_pos = 0;
    int out_pos = 0;

    while (in_pos < data_len && p->state != CHUNK_STATE_DONE &&
           p->state != CHUNK_STATE_ERROR) {
        if (p->state == CHUNK_STATE_SIZE) {
            char c = (char)data[in_pos];
            if (c == '\r') {
                in_pos++;
                continue;
            }
            if (c == '\n') {
                if (p->chunk_size == 0) {
                    p->state = CHUNK_STATE_DONE;
                } else {
                    p->state = CHUNK_STATE_DATA;
                    p->chunk_read = 0;
                    p->chunk_count++;
                }
                in_pos++;
                continue;
            }
            int h = hex_val(c);
            if (h < 0) {
                p->state = CHUNK_STATE_ERROR;
                p->parse_error = -1;
                return -1;
            }
            p->chunk_size = p->chunk_size * 16 + (uint32_t)h;
            in_pos++;
        } else if (p->state == CHUNK_STATE_DATA) {
            uint32_t remaining = p->chunk_size - p->chunk_read;
            int avail = data_len - in_pos;
            uint32_t to_copy = remaining < (uint32_t)avail ? remaining : (uint32_t)avail;
            if (out_pos + (int)to_copy > max_out) {
                to_copy = (uint32_t)(max_out - out_pos);
            }
            for (uint32_t i = 0; i < to_copy; i++) {
                output[out_pos++] = data[in_pos++];
            }
            p->chunk_read += to_copy;
            p->total_bytes += to_copy;
            if (p->chunk_read >= p->chunk_size) {
                p->state = CHUNK_STATE_TRAIL;
            }
        } else if (p->state == CHUNK_STATE_TRAIL) {
            if (data[in_pos] == '\r' || data[in_pos] == '\n') {
                in_pos++;
                if (in_pos < data_len && data[in_pos] == '\n') {
                    in_pos++;
                }
                p->state = CHUNK_STATE_SIZE;
                p->chunk_size = 0;
            } else {
                in_pos++;
            }
        }
    }
    return out_pos;
}

int chunked_is_done(const chunked_parser_t *p) {
    return p->state == CHUNK_STATE_DONE;
}

uint32_t chunked_total_bytes(const chunked_parser_t *p) {
    return p->total_bytes;
}

int chunked_get_chunk_count(const chunked_parser_t *p) {
    return p->chunk_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C345: HTTP chunked transfer encoding parser should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C345: Output should not be empty");
    assert!(
        code.contains("fn chunked_init"),
        "C345: Should contain chunked_init function"
    );
    assert!(
        code.contains("fn chunked_parse"),
        "C345: Should contain chunked_parse function"
    );
    assert!(
        code.contains("fn chunked_is_done"),
        "C345: Should contain chunked_is_done function"
    );
}

// ============================================================================
// C346-C350: Signaling/Management (RADIUS, SIP, STUN, mDNS, Netflow)
// ============================================================================

#[test]
fn c346_radius_attribute_value_pair_encoder() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

#define RADIUS_ATTR_USER_NAME     1
#define RADIUS_ATTR_USER_PASS     2
#define RADIUS_ATTR_NAS_IP        4
#define RADIUS_ATTR_NAS_PORT      5
#define RADIUS_ATTR_FRAMED_IP    8
#define RADIUS_ATTR_SESSION_TIMEOUT 27

#define RADIUS_MAX_ATTRS 32
#define RADIUS_MAX_ATTR_LEN 253

typedef struct {
    uint8_t type;
    uint8_t length;
    uint8_t value[253];
} radius_avp_t;

typedef struct {
    uint8_t code;
    uint8_t identifier;
    uint16_t pkt_length;
    uint8_t authenticator[16];
    radius_avp_t attrs[32];
    int attr_count;
} radius_packet_t;

void radius_init(radius_packet_t *pkt, uint8_t code, uint8_t id) {
    pkt->code = code;
    pkt->identifier = id;
    pkt->pkt_length = 20;
    pkt->attr_count = 0;
    for (int i = 0; i < 16; i++) {
        pkt->authenticator[i] = 0;
    }
}

int radius_add_string_attr(radius_packet_t *pkt, uint8_t type,
                           const uint8_t *value, int val_len) {
    if (pkt->attr_count >= 32) return -1;
    if (val_len > 253) return -2;
    int idx = pkt->attr_count;
    pkt->attrs[idx].type = type;
    pkt->attrs[idx].length = (uint8_t)(val_len + 2);
    for (int i = 0; i < val_len; i++) {
        pkt->attrs[idx].value[i] = value[i];
    }
    pkt->attr_count++;
    pkt->pkt_length += (uint16_t)(val_len + 2);
    return 0;
}

int radius_add_ip_attr(radius_packet_t *pkt, uint8_t type, uint32_t ip) {
    if (pkt->attr_count >= 32) return -1;
    int idx = pkt->attr_count;
    pkt->attrs[idx].type = type;
    pkt->attrs[idx].length = 6;
    pkt->attrs[idx].value[0] = (uint8_t)((ip >> 24) & 0xFF);
    pkt->attrs[idx].value[1] = (uint8_t)((ip >> 16) & 0xFF);
    pkt->attrs[idx].value[2] = (uint8_t)((ip >> 8) & 0xFF);
    pkt->attrs[idx].value[3] = (uint8_t)(ip & 0xFF);
    pkt->attr_count++;
    pkt->pkt_length += 6;
    return 0;
}

int radius_add_int_attr(radius_packet_t *pkt, uint8_t type, uint32_t value) {
    if (pkt->attr_count >= 32) return -1;
    int idx = pkt->attr_count;
    pkt->attrs[idx].type = type;
    pkt->attrs[idx].length = 6;
    pkt->attrs[idx].value[0] = (uint8_t)((value >> 24) & 0xFF);
    pkt->attrs[idx].value[1] = (uint8_t)((value >> 16) & 0xFF);
    pkt->attrs[idx].value[2] = (uint8_t)((value >> 8) & 0xFF);
    pkt->attrs[idx].value[3] = (uint8_t)(value & 0xFF);
    pkt->attr_count++;
    pkt->pkt_length += 6;
    return 0;
}

int radius_encode(const radius_packet_t *pkt, uint8_t *buf, int max_len) {
    if (max_len < (int)pkt->pkt_length) return -1;
    int pos = 0;
    buf[pos++] = pkt->code;
    buf[pos++] = pkt->identifier;
    buf[pos++] = (uint8_t)(pkt->pkt_length >> 8);
    buf[pos++] = (uint8_t)(pkt->pkt_length & 0xFF);
    for (int i = 0; i < 16; i++) {
        buf[pos++] = pkt->authenticator[i];
    }
    for (int i = 0; i < pkt->attr_count; i++) {
        buf[pos++] = pkt->attrs[i].type;
        buf[pos++] = pkt->attrs[i].length;
        for (int j = 0; j < pkt->attrs[i].length - 2; j++) {
            buf[pos++] = pkt->attrs[i].value[j];
        }
    }
    return pos;
}

int radius_attr_count(const radius_packet_t *pkt) {
    return pkt->attr_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C346: RADIUS attribute value pair encoder should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C346: Output should not be empty");
    assert!(
        code.contains("fn radius_init"),
        "C346: Should contain radius_init function"
    );
    assert!(
        code.contains("fn radius_add_string_attr"),
        "C346: Should contain radius_add_string_attr function"
    );
    assert!(
        code.contains("fn radius_encode"),
        "C346: Should contain radius_encode function"
    );
}

#[test]
fn c347_sip_message_parser() {
    let c_code = r#"
typedef unsigned char uint8_t;

#define SIP_MAX_HEADERS 32
#define SIP_MAX_HEADER_LEN 256

typedef struct {
    char name[64];
    char value[256];
    int name_len;
    int value_len;
} sip_header_t;

typedef struct {
    char method[16];
    char request_uri[256];
    char sip_version[16];
    sip_header_t headers[32];
    int header_count;
    int is_request;
    int status_code;
    int valid;
} sip_message_t;

void sip_init(sip_message_t *msg) {
    msg->header_count = 0;
    msg->is_request = 0;
    msg->status_code = 0;
    msg->valid = 0;
    msg->method[0] = 0;
    msg->request_uri[0] = 0;
}

int str_copy_until(const char *src, int src_len, int start,
                   char *dst, int max_dst, char delim) {
    int len = 0;
    int i = start;
    while (i < src_len && src[i] != delim && len < max_dst - 1) {
        dst[len++] = src[i++];
    }
    dst[len] = 0;
    return i;
}

int sip_parse_request_line(sip_message_t *msg, const char *line, int line_len) {
    int pos = str_copy_until(line, line_len, 0, msg->method, 16, ' ');
    if (pos >= line_len) return -1;
    pos++;
    pos = str_copy_until(line, line_len, pos, msg->request_uri, 256, ' ');
    if (pos >= line_len) return -2;
    pos++;
    str_copy_until(line, line_len, pos, msg->sip_version, 16, '\r');
    msg->is_request = 1;
    return 0;
}

int sip_add_header(sip_message_t *msg, const char *name, int nlen,
                   const char *value, int vlen) {
    if (msg->header_count >= 32) return -1;
    int idx = msg->header_count;
    for (int i = 0; i < nlen && i < 63; i++) {
        msg->headers[idx].name[i] = name[i];
    }
    msg->headers[idx].name[nlen < 63 ? nlen : 63] = 0;
    msg->headers[idx].name_len = nlen;
    for (int i = 0; i < vlen && i < 255; i++) {
        msg->headers[idx].value[i] = value[i];
    }
    msg->headers[idx].value[vlen < 255 ? vlen : 255] = 0;
    msg->headers[idx].value_len = vlen;
    msg->header_count++;
    return 0;
}

int sip_find_header(const sip_message_t *msg, const char *name, int nlen) {
    for (int i = 0; i < msg->header_count; i++) {
        if (msg->headers[i].name_len != nlen) continue;
        int match = 1;
        for (int j = 0; j < nlen; j++) {
            char a = msg->headers[i].name[j];
            char b = name[j];
            if (a >= 'A' && a <= 'Z') a += 32;
            if (b >= 'A' && b <= 'Z') b += 32;
            if (a != b) { match = 0; break; }
        }
        if (match) return i;
    }
    return -1;
}

int sip_header_count(const sip_message_t *msg) {
    return msg->header_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C347: SIP message parser should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C347: Output should not be empty");
    assert!(
        code.contains("fn sip_init"),
        "C347: Should contain sip_init function"
    );
    assert!(
        code.contains("fn sip_parse_request_line"),
        "C347: Should contain sip_parse_request_line function"
    );
    assert!(
        code.contains("fn sip_find_header"),
        "C347: Should contain sip_find_header function"
    );
}

#[test]
fn c348_stun_message_builder() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

#define STUN_BINDING_REQUEST  0x0001
#define STUN_BINDING_RESPONSE 0x0101
#define STUN_MAGIC_COOKIE 0x2112A442
#define STUN_HEADER_SIZE 20

#define STUN_ATTR_MAPPED_ADDR   0x0001
#define STUN_ATTR_XOR_MAPPED    0x0020
#define STUN_ATTR_SOFTWARE      0x8022
#define STUN_ATTR_FINGERPRINT   0x8028

#define STUN_MAX_ATTRS 16

typedef struct {
    uint16_t type;
    uint16_t length;
    uint8_t value[128];
} stun_attr_t;

typedef struct {
    uint16_t msg_type;
    uint16_t msg_length;
    uint8_t transaction_id[12];
    stun_attr_t attrs[16];
    int attr_count;
} stun_message_t;

void stun_init(stun_message_t *msg, uint16_t type) {
    msg->msg_type = type;
    msg->msg_length = 0;
    msg->attr_count = 0;
    for (int i = 0; i < 12; i++) {
        msg->transaction_id[i] = (uint8_t)(i + 1);
    }
}

int stun_add_attr(stun_message_t *msg, uint16_t type,
                  const uint8_t *value, uint16_t length) {
    if (msg->attr_count >= 16) return -1;
    if (length > 128) return -2;
    int idx = msg->attr_count;
    msg->attrs[idx].type = type;
    msg->attrs[idx].length = length;
    for (int i = 0; i < length; i++) {
        msg->attrs[idx].value[i] = value[i];
    }
    msg->attr_count++;
    uint16_t padded = (length + 3) & ~3;
    msg->msg_length += 4 + padded;
    return 0;
}

int stun_encode(const stun_message_t *msg, uint8_t *buf, int max_len) {
    int total = 20 + (int)msg->msg_length;
    if (max_len < total) return -1;
    int pos = 0;
    buf[pos++] = (uint8_t)(msg->msg_type >> 8);
    buf[pos++] = (uint8_t)(msg->msg_type & 0xFF);
    buf[pos++] = (uint8_t)(msg->msg_length >> 8);
    buf[pos++] = (uint8_t)(msg->msg_length & 0xFF);
    buf[pos++] = (uint8_t)(STUN_MAGIC_COOKIE >> 24);
    buf[pos++] = (uint8_t)((STUN_MAGIC_COOKIE >> 16) & 0xFF);
    buf[pos++] = (uint8_t)((STUN_MAGIC_COOKIE >> 8) & 0xFF);
    buf[pos++] = (uint8_t)(STUN_MAGIC_COOKIE & 0xFF);
    for (int i = 0; i < 12; i++) {
        buf[pos++] = msg->transaction_id[i];
    }
    for (int a = 0; a < msg->attr_count; a++) {
        buf[pos++] = (uint8_t)(msg->attrs[a].type >> 8);
        buf[pos++] = (uint8_t)(msg->attrs[a].type & 0xFF);
        buf[pos++] = (uint8_t)(msg->attrs[a].length >> 8);
        buf[pos++] = (uint8_t)(msg->attrs[a].length & 0xFF);
        for (int i = 0; i < msg->attrs[a].length; i++) {
            buf[pos++] = msg->attrs[a].value[i];
        }
        int pad = (4 - (msg->attrs[a].length % 4)) % 4;
        for (int i = 0; i < pad; i++) {
            buf[pos++] = 0;
        }
    }
    return pos;
}

int stun_find_attr(const stun_message_t *msg, uint16_t type) {
    for (int i = 0; i < msg->attr_count; i++) {
        if (msg->attrs[i].type == type) return i;
    }
    return -1;
}

int stun_is_request(const stun_message_t *msg) {
    return (msg->msg_type & 0x0110) == 0x0000;
}

int stun_is_response(const stun_message_t *msg) {
    return (msg->msg_type & 0x0110) == 0x0100;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C348: STUN message builder should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C348: Output should not be empty");
    assert!(
        code.contains("fn stun_init"),
        "C348: Should contain stun_init function"
    );
    assert!(
        code.contains("fn stun_encode"),
        "C348: Should contain stun_encode function"
    );
    assert!(
        code.contains("fn stun_find_attr"),
        "C348: Should contain stun_find_attr function"
    );
}

#[test]
fn c349_mdns_response_record_formatter() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

#define MDNS_TYPE_A      1
#define MDNS_TYPE_PTR   12
#define MDNS_TYPE_SRV   33
#define MDNS_TYPE_TXT   16
#define MDNS_CLASS_IN    1
#define MDNS_CACHE_FLUSH 0x8000

#define MDNS_MAX_RECORDS 16

typedef struct {
    uint8_t name[256];
    int name_len;
    uint16_t rtype;
    uint16_t rclass;
    uint32_t ttl;
    uint8_t rdata[256];
    int rdata_len;
} mdns_record_t;

typedef struct {
    mdns_record_t records[16];
    int record_count;
    uint16_t transaction_id;
    uint16_t flags;
} mdns_response_t;

void mdns_init(mdns_response_t *resp) {
    resp->record_count = 0;
    resp->transaction_id = 0;
    resp->flags = 0x8400;
}

int mdns_encode_name(uint8_t *buf, int max_len, const char *name, int name_len) {
    int pos = 0;
    int label_start = 0;
    for (int i = 0; i <= name_len; i++) {
        if (i == name_len || name[i] == '.') {
            int llen = i - label_start;
            if (llen > 63 || llen == 0) return -1;
            if (pos + 1 + llen >= max_len) return -2;
            buf[pos++] = (uint8_t)llen;
            for (int j = 0; j < llen; j++) {
                buf[pos++] = (uint8_t)name[label_start + j];
            }
            label_start = i + 1;
        }
    }
    if (pos >= max_len) return -2;
    buf[pos++] = 0;
    return pos;
}

int mdns_add_a_record(mdns_response_t *resp, const char *name, int name_len,
                      uint32_t ttl, uint32_t ipv4_addr) {
    if (resp->record_count >= 16) return -1;
    int idx = resp->record_count;
    int nlen = mdns_encode_name(resp->records[idx].name, 256, name, name_len);
    if (nlen < 0) return nlen;
    resp->records[idx].name_len = nlen;
    resp->records[idx].rtype = MDNS_TYPE_A;
    resp->records[idx].rclass = MDNS_CLASS_IN | MDNS_CACHE_FLUSH;
    resp->records[idx].ttl = ttl;
    resp->records[idx].rdata[0] = (uint8_t)((ipv4_addr >> 24) & 0xFF);
    resp->records[idx].rdata[1] = (uint8_t)((ipv4_addr >> 16) & 0xFF);
    resp->records[idx].rdata[2] = (uint8_t)((ipv4_addr >> 8) & 0xFF);
    resp->records[idx].rdata[3] = (uint8_t)(ipv4_addr & 0xFF);
    resp->records[idx].rdata_len = 4;
    resp->record_count++;
    return 0;
}

int mdns_add_srv_record(mdns_response_t *resp, const char *name, int name_len,
                        uint32_t ttl, uint16_t priority, uint16_t weight,
                        uint16_t port, const char *target, int target_len) {
    if (resp->record_count >= 16) return -1;
    int idx = resp->record_count;
    int nlen = mdns_encode_name(resp->records[idx].name, 256, name, name_len);
    if (nlen < 0) return nlen;
    resp->records[idx].name_len = nlen;
    resp->records[idx].rtype = MDNS_TYPE_SRV;
    resp->records[idx].rclass = MDNS_CLASS_IN | MDNS_CACHE_FLUSH;
    resp->records[idx].ttl = ttl;
    int rpos = 0;
    resp->records[idx].rdata[rpos++] = (uint8_t)(priority >> 8);
    resp->records[idx].rdata[rpos++] = (uint8_t)(priority & 0xFF);
    resp->records[idx].rdata[rpos++] = (uint8_t)(weight >> 8);
    resp->records[idx].rdata[rpos++] = (uint8_t)(weight & 0xFF);
    resp->records[idx].rdata[rpos++] = (uint8_t)(port >> 8);
    resp->records[idx].rdata[rpos++] = (uint8_t)(port & 0xFF);
    int tlen = mdns_encode_name(resp->records[idx].rdata + rpos,
                                256 - rpos, target, target_len);
    if (tlen < 0) return tlen;
    rpos += tlen;
    resp->records[idx].rdata_len = rpos;
    resp->record_count++;
    return 0;
}

int mdns_record_count(const mdns_response_t *resp) {
    return resp->record_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C349: mDNS response record formatter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C349: Output should not be empty");
    assert!(
        code.contains("fn mdns_init"),
        "C349: Should contain mdns_init function"
    );
    assert!(
        code.contains("fn mdns_add_a_record"),
        "C349: Should contain mdns_add_a_record function"
    );
    assert!(
        code.contains("fn mdns_add_srv_record"),
        "C349: Should contain mdns_add_srv_record function"
    );
}

#[test]
fn c350_netflow_v5_v9_record_collector() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

#define NF_MAX_RECORDS 64
#define NF_V5_HEADER_SIZE 24
#define NF_V5_RECORD_SIZE 48

typedef struct {
    uint32_t src_addr;
    uint32_t dst_addr;
    uint32_t next_hop;
    uint16_t input_iface;
    uint16_t output_iface;
    uint32_t packets;
    uint32_t bytes;
    uint32_t first_ts;
    uint32_t last_ts;
    uint16_t src_port;
    uint16_t dst_port;
    uint8_t tcp_flags;
    uint8_t protocol;
    uint8_t tos;
    uint16_t src_as;
    uint16_t dst_as;
} nf_v5_record_t;

typedef struct {
    uint16_t version;
    uint16_t count;
    uint32_t sys_uptime;
    uint32_t unix_secs;
    uint32_t sequence;
    nf_v5_record_t records[64];
    int record_count;
    uint32_t total_packets_seen;
    uint32_t total_bytes_seen;
} nf_collector_t;

void nf_init(nf_collector_t *c) {
    c->version = 5;
    c->count = 0;
    c->sys_uptime = 0;
    c->unix_secs = 0;
    c->sequence = 0;
    c->record_count = 0;
    c->total_packets_seen = 0;
    c->total_bytes_seen = 0;
}

int nf_add_record(nf_collector_t *c, uint32_t src, uint32_t dst,
                  uint16_t sport, uint16_t dport, uint8_t proto,
                  uint32_t packets, uint32_t bytes) {
    if (c->record_count >= 64) return -1;
    int idx = c->record_count;
    c->records[idx].src_addr = src;
    c->records[idx].dst_addr = dst;
    c->records[idx].next_hop = 0;
    c->records[idx].input_iface = 0;
    c->records[idx].output_iface = 0;
    c->records[idx].packets = packets;
    c->records[idx].bytes = bytes;
    c->records[idx].first_ts = c->sys_uptime;
    c->records[idx].last_ts = c->sys_uptime;
    c->records[idx].src_port = sport;
    c->records[idx].dst_port = dport;
    c->records[idx].tcp_flags = 0;
    c->records[idx].protocol = proto;
    c->records[idx].tos = 0;
    c->records[idx].src_as = 0;
    c->records[idx].dst_as = 0;
    c->record_count++;
    c->count++;
    c->total_packets_seen += packets;
    c->total_bytes_seen += bytes;
    return idx;
}

int nf_find_flow(const nf_collector_t *c, uint32_t src, uint32_t dst,
                 uint16_t sport, uint16_t dport, uint8_t proto) {
    for (int i = 0; i < c->record_count; i++) {
        if (c->records[i].src_addr == src &&
            c->records[i].dst_addr == dst &&
            c->records[i].src_port == sport &&
            c->records[i].dst_port == dport &&
            c->records[i].protocol == proto) {
            return i;
        }
    }
    return -1;
}

int nf_update_flow(nf_collector_t *c, int idx, uint32_t packets,
                   uint32_t bytes, uint32_t timestamp) {
    if (idx < 0 || idx >= c->record_count) return -1;
    c->records[idx].packets += packets;
    c->records[idx].bytes += bytes;
    c->records[idx].last_ts = timestamp;
    c->total_packets_seen += packets;
    c->total_bytes_seen += bytes;
    return 0;
}

uint32_t nf_total_flows(const nf_collector_t *c) {
    return (uint32_t)c->record_count;
}

uint32_t nf_total_bytes(const nf_collector_t *c) {
    return c->total_bytes_seen;
}

int nf_top_talker(const nf_collector_t *c) {
    int best = -1;
    uint32_t max_bytes = 0;
    for (int i = 0; i < c->record_count; i++) {
        if (c->records[i].bytes > max_bytes) {
            max_bytes = c->records[i].bytes;
            best = i;
        }
    }
    return best;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C350: Netflow v5/v9 record collector should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C350: Output should not be empty");
    assert!(
        code.contains("fn nf_init"),
        "C350: Should contain nf_init function"
    );
    assert!(
        code.contains("fn nf_add_record"),
        "C350: Should contain nf_add_record function"
    );
    assert!(
        code.contains("fn nf_find_flow"),
        "C350: Should contain nf_find_flow function"
    );
    assert!(
        code.contains("fn nf_top_talker"),
        "C350: Should contain nf_top_talker function"
    );
}
