//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1501-C1525: Networking Stack Patterns
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise low-level networking stack patterns commonly found
//! in production C codebases: packet processing, protocol headers, connection
//! state machines, routing tables, and application layer protocols.
//! All tests use self-contained C99 code with no #include directives.
//!
//! Organization:
//! - C1501-C1505: Packet processing (buffer, checksum, fragmentation, reassembly, byte order)
//! - C1506-C1510: Protocol headers (ethernet, IP header, TCP segment, UDP datagram, ICMP echo)
//! - C1511-C1515: Connection state (TCP state machine, conn table, seq tracking, window mgmt, retransmit)
//! - C1516-C1520: Routing (routing table, longest prefix, next hop, ARP cache, metrics)
//! - C1521-C1525: Application layer (DNS query, HTTP header, TLS record, DHCP options, NTP timestamp)

// ============================================================================
// C1501-C1505: Packet Processing
// ============================================================================

/// C1501: Packet buffer with head/tail room for header prepend and payload append
#[test]
fn c1501_packet_buffer_management() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

#define NS_PKT_MAX 1536
#define NS_PKT_HEADROOM 64

typedef struct {
    uint8_t data[NS_PKT_MAX];
    uint16_t head;
    uint16_t tail;
    uint16_t len;
} ns_pktbuf_t;

void ns_pktbuf_init(ns_pktbuf_t *pb) {
    pb->head = NS_PKT_HEADROOM;
    pb->tail = NS_PKT_HEADROOM;
    pb->len = 0;
}

int ns_pktbuf_append(ns_pktbuf_t *pb, const uint8_t *data, uint16_t dlen) {
    if (pb->tail + dlen > NS_PKT_MAX) return -1;
    uint16_t i;
    for (i = 0; i < dlen; i++) {
        pb->data[pb->tail + i] = data[i];
    }
    pb->tail += dlen;
    pb->len += dlen;
    return 0;
}

int ns_pktbuf_prepend(ns_pktbuf_t *pb, const uint8_t *hdr, uint16_t hlen) {
    if (hlen > pb->head) return -1;
    pb->head -= hlen;
    uint16_t i;
    for (i = 0; i < hlen; i++) {
        pb->data[pb->head + i] = hdr[i];
    }
    pb->len += hlen;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1501: Packet buffer management should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1501: Output should not be empty");
    assert!(
        code.contains("fn ns_pktbuf_init"),
        "C1501: Should contain ns_pktbuf_init"
    );
    assert!(
        code.contains("fn ns_pktbuf_append"),
        "C1501: Should contain ns_pktbuf_append"
    );
}

/// C1502: Internet checksum (RFC 1071) computation over a buffer
#[test]
fn c1502_internet_checksum_computation() {
    let c_code = r#"
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;
typedef unsigned long size_t;
typedef unsigned char uint8_t;

uint16_t ns_checksum_compute(const uint8_t *buf, int len) {
    uint32_t sum = 0;
    int i;
    for (i = 0; i + 1 < len; i += 2) {
        uint16_t word = ((uint16_t)buf[i] << 8) | buf[i + 1];
        sum += word;
    }
    if (len & 1) {
        sum += (uint16_t)buf[len - 1] << 8;
    }
    while (sum >> 16) {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    return (uint16_t)(~sum & 0xFFFF);
}

int ns_checksum_verify(const uint8_t *buf, int len) {
    uint16_t cksum = ns_checksum_compute(buf, len);
    return (cksum == 0) ? 1 : 0;
}

uint16_t ns_checksum_update(uint16_t old_cksum, uint16_t old_val, uint16_t new_val) {
    uint32_t sum = (~old_cksum & 0xFFFF) + (~old_val & 0xFFFF) + new_val;
    while (sum >> 16) {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    return (uint16_t)(~sum & 0xFFFF);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1502: Internet checksum should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1502: Output should not be empty");
    assert!(
        code.contains("fn ns_checksum_compute"),
        "C1502: Should contain ns_checksum_compute"
    );
}

/// C1503: IP packet fragmentation with offset and more-fragments flag
#[test]
fn c1503_ip_fragmentation() {
    let c_code = r#"
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;
typedef unsigned long size_t;

#define NS_FRAG_MTU 576
#define NS_FRAG_HDR 20
#define NS_FRAG_MAX_PAYLOAD (NS_FRAG_MTU - NS_FRAG_HDR)

typedef struct {
    uint16_t id;
    uint16_t offset;
    int more_fragments;
    uint16_t payload_len;
    uint8_t payload[NS_FRAG_MAX_PAYLOAD];
} ns_fragment_t;

int ns_fragment_count(uint16_t total_len) {
    uint16_t payload = total_len - NS_FRAG_HDR;
    int max_data = NS_FRAG_MAX_PAYLOAD & ~7;
    int count = (payload + max_data - 1) / max_data;
    return count > 0 ? count : 1;
}

int ns_fragment_build(ns_fragment_t *frag, uint16_t id, uint16_t offset,
                      const uint8_t *data, uint16_t dlen, int is_last) {
    frag->id = id;
    frag->offset = offset;
    frag->more_fragments = is_last ? 0 : 1;
    frag->payload_len = dlen;
    uint16_t i;
    for (i = 0; i < dlen; i++) {
        frag->payload[i] = data[i];
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1503: IP fragmentation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1503: Output should not be empty");
    assert!(
        code.contains("fn ns_fragment_count"),
        "C1503: Should contain ns_fragment_count"
    );
}

/// C1504: Fragment reassembly tracking with bitmap for received fragments
#[test]
fn c1504_fragment_reassembly() {
    let c_code = r#"
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

#define NS_REASM_MAX_FRAGS 64

typedef struct {
    uint16_t id;
    uint8_t received[NS_REASM_MAX_FRAGS];
    int total_frags;
    int received_count;
    int complete;
} ns_reassembly_t;

void ns_reasm_init(ns_reassembly_t *r, uint16_t id) {
    r->id = id;
    r->total_frags = -1;
    r->received_count = 0;
    r->complete = 0;
    int i;
    for (i = 0; i < NS_REASM_MAX_FRAGS; i++) {
        r->received[i] = 0;
    }
}

int ns_reasm_add(ns_reassembly_t *r, int frag_idx, int is_last) {
    if (frag_idx < 0 || frag_idx >= NS_REASM_MAX_FRAGS) return -1;
    if (r->received[frag_idx]) return 0;
    r->received[frag_idx] = 1;
    r->received_count++;
    if (is_last) {
        r->total_frags = frag_idx + 1;
    }
    if (r->total_frags > 0 && r->received_count == r->total_frags) {
        r->complete = 1;
    }
    return 1;
}

int ns_reasm_is_complete(const ns_reassembly_t *r) {
    return r->complete;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1504: Fragment reassembly should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1504: Output should not be empty");
    assert!(
        code.contains("fn ns_reasm_init"),
        "C1504: Should contain ns_reasm_init"
    );
    assert!(
        code.contains("fn ns_reasm_add"),
        "C1504: Should contain ns_reasm_add"
    );
}

/// C1505: Byte order conversion (host to network, network to host) for 16/32-bit
#[test]
fn c1505_byte_order_conversion() {
    let c_code = r#"
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

uint16_t ns_htons(uint16_t hostshort) {
    return (uint16_t)((hostshort >> 8) | (hostshort << 8));
}

uint16_t ns_ntohs(uint16_t netshort) {
    return (uint16_t)((netshort >> 8) | (netshort << 8));
}

uint32_t ns_htonl(uint32_t hostlong) {
    return ((hostlong & 0xFF000000) >> 24) |
           ((hostlong & 0x00FF0000) >> 8)  |
           ((hostlong & 0x0000FF00) << 8)  |
           ((hostlong & 0x000000FF) << 24);
}

uint32_t ns_ntohl(uint32_t netlong) {
    return ns_htonl(netlong);
}

int ns_byteorder_selftest(void) {
    if (ns_htons(ns_ntohs(0x1234)) != 0x1234) return -1;
    if (ns_htonl(ns_ntohl(0xDEADBEEF)) != 0xDEADBEEF) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1505: Byte order conversion should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1505: Output should not be empty");
    assert!(
        code.contains("fn ns_htons"),
        "C1505: Should contain ns_htons"
    );
    assert!(
        code.contains("fn ns_htonl"),
        "C1505: Should contain ns_htonl"
    );
}

// ============================================================================
// C1506-C1510: Protocol Headers
// ============================================================================

/// C1506: Ethernet frame builder with MAC addresses and EtherType
#[test]
fn c1506_ethernet_frame_builder() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

#define NS_ETH_ALEN 6
#define NS_ETH_HDR_LEN 14

typedef struct {
    uint8_t dst[NS_ETH_ALEN];
    uint8_t src[NS_ETH_ALEN];
    uint16_t ethertype;
} ns_eth_header_t;

void ns_eth_set_addrs(ns_eth_header_t *hdr,
                      const uint8_t *dst, const uint8_t *src) {
    int i;
    for (i = 0; i < NS_ETH_ALEN; i++) {
        hdr->dst[i] = dst[i];
        hdr->src[i] = src[i];
    }
}

void ns_eth_set_type(ns_eth_header_t *hdr, uint16_t etype) {
    hdr->ethertype = ((etype >> 8) & 0xFF) | ((etype & 0xFF) << 8);
}

int ns_eth_is_broadcast(const ns_eth_header_t *hdr) {
    int i;
    for (i = 0; i < NS_ETH_ALEN; i++) {
        if (hdr->dst[i] != 0xFF) return 0;
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1506: Ethernet frame builder should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1506: Output should not be empty");
    assert!(
        code.contains("fn ns_eth_set_addrs"),
        "C1506: Should contain ns_eth_set_addrs"
    );
}

/// C1507: IPv4 header builder with TTL, protocol, and address fields
#[test]
fn c1507_ipv4_header_builder() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

typedef struct {
    uint8_t version_ihl;
    uint8_t tos;
    uint16_t total_length;
    uint16_t identification;
    uint16_t flags_fragoff;
    uint8_t ttl;
    uint8_t protocol;
    uint16_t checksum;
    uint32_t src_addr;
    uint32_t dst_addr;
} ns_ipv4_hdr_t;

void ns_ipv4_init(ns_ipv4_hdr_t *hdr) {
    hdr->version_ihl = 0x45;
    hdr->tos = 0;
    hdr->total_length = 0;
    hdr->identification = 0;
    hdr->flags_fragoff = 0;
    hdr->ttl = 64;
    hdr->protocol = 0;
    hdr->checksum = 0;
    hdr->src_addr = 0;
    hdr->dst_addr = 0;
}

void ns_ipv4_set_addrs(ns_ipv4_hdr_t *hdr, uint32_t src, uint32_t dst) {
    hdr->src_addr = src;
    hdr->dst_addr = dst;
}

void ns_ipv4_set_proto(ns_ipv4_hdr_t *hdr, uint8_t proto, uint16_t payload_len) {
    hdr->protocol = proto;
    hdr->total_length = 20 + payload_len;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1507: IPv4 header builder should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1507: Output should not be empty");
    assert!(
        code.contains("fn ns_ipv4_init"),
        "C1507: Should contain ns_ipv4_init"
    );
    assert!(
        code.contains("fn ns_ipv4_set_addrs"),
        "C1507: Should contain ns_ipv4_set_addrs"
    );
}

/// C1508: TCP segment builder with flags, sequence, and acknowledgment numbers
#[test]
fn c1508_tcp_segment_builder() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

#define NS_TCP_FIN 0x01
#define NS_TCP_SYN 0x02
#define NS_TCP_RST 0x04
#define NS_TCP_ACK 0x10

typedef struct {
    uint16_t src_port;
    uint16_t dst_port;
    uint32_t seq_num;
    uint32_t ack_num;
    uint8_t data_offset;
    uint8_t flags;
    uint16_t window;
} ns_tcp_hdr_t;

void ns_tcp_init(ns_tcp_hdr_t *hdr, uint16_t src, uint16_t dst) {
    hdr->src_port = src;
    hdr->dst_port = dst;
    hdr->seq_num = 0;
    hdr->ack_num = 0;
    hdr->data_offset = 5;
    hdr->flags = 0;
    hdr->window = 65535;
}

void ns_tcp_set_syn(ns_tcp_hdr_t *hdr, uint32_t isn) {
    hdr->flags = NS_TCP_SYN;
    hdr->seq_num = isn;
}

void ns_tcp_set_ack(ns_tcp_hdr_t *hdr, uint32_t seq, uint32_t ack) {
    hdr->flags = NS_TCP_ACK;
    hdr->seq_num = seq;
    hdr->ack_num = ack;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1508: TCP segment builder should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1508: Output should not be empty");
    assert!(
        code.contains("fn ns_tcp_init"),
        "C1508: Should contain ns_tcp_init"
    );
}

/// C1509: UDP datagram builder with port and length fields
#[test]
fn c1509_udp_datagram_builder() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

typedef struct {
    uint16_t src_port;
    uint16_t dst_port;
    uint16_t length;
    uint16_t checksum;
} ns_udp_hdr_t;

void ns_udp_init(ns_udp_hdr_t *hdr, uint16_t src, uint16_t dst) {
    hdr->src_port = src;
    hdr->dst_port = dst;
    hdr->length = 8;
    hdr->checksum = 0;
}

void ns_udp_set_payload_len(ns_udp_hdr_t *hdr, uint16_t plen) {
    hdr->length = 8 + plen;
}

int ns_udp_validate(const ns_udp_hdr_t *hdr) {
    if (hdr->length < 8) return -1;
    if (hdr->src_port == 0) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1509: UDP datagram builder should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1509: Output should not be empty");
    assert!(
        code.contains("fn ns_udp_init"),
        "C1509: Should contain ns_udp_init"
    );
}

/// C1510: ICMP echo request/reply builder with identifier and sequence
#[test]
fn c1510_icmp_echo_builder() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

#define NS_ICMP_ECHO_REQUEST 8
#define NS_ICMP_ECHO_REPLY   0

typedef struct {
    uint8_t type;
    uint8_t code;
    uint16_t checksum;
    uint16_t identifier;
    uint16_t sequence;
} ns_icmp_echo_t;

void ns_icmp_echo_request(ns_icmp_echo_t *pkt, uint16_t id, uint16_t seq) {
    pkt->type = NS_ICMP_ECHO_REQUEST;
    pkt->code = 0;
    pkt->checksum = 0;
    pkt->identifier = id;
    pkt->sequence = seq;
}

void ns_icmp_echo_reply(ns_icmp_echo_t *pkt, const ns_icmp_echo_t *req) {
    pkt->type = NS_ICMP_ECHO_REPLY;
    pkt->code = 0;
    pkt->checksum = 0;
    pkt->identifier = req->identifier;
    pkt->sequence = req->sequence;
}

int ns_icmp_is_echo_reply(const ns_icmp_echo_t *pkt, uint16_t expected_id) {
    return (pkt->type == NS_ICMP_ECHO_REPLY && pkt->identifier == expected_id) ? 1 : 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1510: ICMP echo builder should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1510: Output should not be empty");
    assert!(
        code.contains("fn ns_icmp_echo_request"),
        "C1510: Should contain ns_icmp_echo_request"
    );
}

// ============================================================================
// C1511-C1515: Connection State
// ============================================================================

/// C1511: TCP state machine with SYN/ACK/FIN transitions
#[test]
fn c1511_tcp_state_machine() {
    let c_code = r#"
typedef unsigned char uint8_t;

#define NS_TCP_CLOSED      0
#define NS_TCP_LISTEN      1
#define NS_TCP_SYN_SENT    2
#define NS_TCP_SYN_RCVD    3
#define NS_TCP_ESTABLISHED 4
#define NS_TCP_FIN_WAIT_1  5
#define NS_TCP_FIN_WAIT_2  6
#define NS_TCP_CLOSE_WAIT  7
#define NS_TCP_TIME_WAIT   8

typedef struct {
    uint8_t state;
    int is_server;
} ns_tcp_fsm_t;

void ns_tcp_fsm_init(ns_tcp_fsm_t *fsm, int is_server) {
    fsm->state = is_server ? NS_TCP_LISTEN : NS_TCP_CLOSED;
    fsm->is_server = is_server;
}

int ns_tcp_fsm_event(ns_tcp_fsm_t *fsm, int event) {
    /* events: 0=SYN, 1=SYN_ACK, 2=ACK, 3=FIN, 4=CLOSE */
    switch (fsm->state) {
        case NS_TCP_CLOSED:
            if (event == 0) { fsm->state = NS_TCP_SYN_SENT; return 0; }
            break;
        case NS_TCP_LISTEN:
            if (event == 0) { fsm->state = NS_TCP_SYN_RCVD; return 0; }
            break;
        case NS_TCP_SYN_SENT:
            if (event == 1) { fsm->state = NS_TCP_ESTABLISHED; return 0; }
            break;
        case NS_TCP_SYN_RCVD:
            if (event == 2) { fsm->state = NS_TCP_ESTABLISHED; return 0; }
            break;
        case NS_TCP_ESTABLISHED:
            if (event == 3) { fsm->state = NS_TCP_CLOSE_WAIT; return 0; }
            if (event == 4) { fsm->state = NS_TCP_FIN_WAIT_1; return 0; }
            break;
        case NS_TCP_FIN_WAIT_1:
            if (event == 2) { fsm->state = NS_TCP_FIN_WAIT_2; return 0; }
            break;
        case NS_TCP_FIN_WAIT_2:
            if (event == 3) { fsm->state = NS_TCP_TIME_WAIT; return 0; }
            break;
    }
    return -1;
}

int ns_tcp_fsm_is_connected(const ns_tcp_fsm_t *fsm) {
    return fsm->state == NS_TCP_ESTABLISHED;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1511: TCP state machine should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1511: Output should not be empty");
    assert!(
        code.contains("fn ns_tcp_fsm_init"),
        "C1511: Should contain ns_tcp_fsm_init"
    );
    assert!(
        code.contains("fn ns_tcp_fsm_event"),
        "C1511: Should contain ns_tcp_fsm_event"
    );
}

/// C1512: Connection table mapping 4-tuple keys to connection state
#[test]
fn c1512_connection_table() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;

#define NS_CONN_TABLE_SIZE 64

typedef struct {
    uint32_t src_ip;
    uint32_t dst_ip;
    uint16_t src_port;
    uint16_t dst_port;
    int state;
    int active;
} ns_conn_entry_t;

typedef struct {
    ns_conn_entry_t entries[NS_CONN_TABLE_SIZE];
    int count;
} ns_conn_table_t;

void ns_conn_table_init(ns_conn_table_t *tbl) {
    tbl->count = 0;
    int i;
    for (i = 0; i < NS_CONN_TABLE_SIZE; i++) {
        tbl->entries[i].active = 0;
    }
}

int ns_conn_table_insert(ns_conn_table_t *tbl, uint32_t sip, uint32_t dip,
                         uint16_t sp, uint16_t dp) {
    if (tbl->count >= NS_CONN_TABLE_SIZE) return -1;
    int i;
    for (i = 0; i < NS_CONN_TABLE_SIZE; i++) {
        if (!tbl->entries[i].active) {
            tbl->entries[i].src_ip = sip;
            tbl->entries[i].dst_ip = dip;
            tbl->entries[i].src_port = sp;
            tbl->entries[i].dst_port = dp;
            tbl->entries[i].state = 0;
            tbl->entries[i].active = 1;
            tbl->count++;
            return i;
        }
    }
    return -1;
}

int ns_conn_table_find(const ns_conn_table_t *tbl, uint32_t sip, uint32_t dip,
                       uint16_t sp, uint16_t dp) {
    int i;
    for (i = 0; i < NS_CONN_TABLE_SIZE; i++) {
        if (tbl->entries[i].active &&
            tbl->entries[i].src_ip == sip && tbl->entries[i].dst_ip == dip &&
            tbl->entries[i].src_port == sp && tbl->entries[i].dst_port == dp) {
            return i;
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1512: Connection table should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1512: Output should not be empty");
    assert!(
        code.contains("fn ns_conn_table_init"),
        "C1512: Should contain ns_conn_table_init"
    );
}

/// C1513: TCP sequence number tracking with wraparound comparison
#[test]
fn c1513_sequence_number_tracking() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t snd_nxt;
    uint32_t snd_una;
    uint32_t rcv_nxt;
    uint32_t iss;
} ns_seq_tracker_t;

int ns_seq_before(uint32_t a, uint32_t b) {
    return (int)(a - b) < 0;
}

int ns_seq_after(uint32_t a, uint32_t b) {
    return (int)(b - a) < 0;
}

void ns_seq_init(ns_seq_tracker_t *t, uint32_t isn) {
    t->iss = isn;
    t->snd_nxt = isn + 1;
    t->snd_una = isn;
    t->rcv_nxt = 0;
}

int ns_seq_advance_send(ns_seq_tracker_t *t, uint32_t len) {
    t->snd_nxt += len;
    return 0;
}

int ns_seq_ack_received(ns_seq_tracker_t *t, uint32_t ack) {
    if (ns_seq_before(t->snd_una, ack) && !ns_seq_after(ack, t->snd_nxt)) {
        t->snd_una = ack;
        return 1;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1513: Sequence number tracking should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1513: Output should not be empty");
    assert!(
        code.contains("fn ns_seq_before"),
        "C1513: Should contain ns_seq_before"
    );
    assert!(
        code.contains("fn ns_seq_init"),
        "C1513: Should contain ns_seq_init"
    );
}

/// C1514: TCP sliding window management for flow control
#[test]
fn c1514_window_management() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;

typedef struct {
    uint32_t window_start;
    uint16_t window_size;
    uint32_t bytes_in_flight;
    uint16_t max_window;
} ns_window_t;

void ns_window_init(ns_window_t *w, uint16_t size) {
    w->window_start = 0;
    w->window_size = size;
    w->bytes_in_flight = 0;
    w->max_window = size;
}

int ns_window_can_send(const ns_window_t *w, uint32_t len) {
    return (w->bytes_in_flight + len <= w->window_size) ? 1 : 0;
}

void ns_window_on_send(ns_window_t *w, uint32_t len) {
    w->bytes_in_flight += len;
}

void ns_window_on_ack(ns_window_t *w, uint32_t acked) {
    if (acked > w->bytes_in_flight) {
        w->bytes_in_flight = 0;
    } else {
        w->bytes_in_flight -= acked;
    }
}

void ns_window_update(ns_window_t *w, uint16_t new_size) {
    w->window_size = new_size;
    if (new_size > w->max_window) {
        w->max_window = new_size;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1514: Window management should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1514: Output should not be empty");
    assert!(
        code.contains("fn ns_window_init"),
        "C1514: Should contain ns_window_init"
    );
    assert!(
        code.contains("fn ns_window_can_send"),
        "C1514: Should contain ns_window_can_send"
    );
}

/// C1515: Retransmission timer with exponential backoff
#[test]
fn c1515_retransmission_timer() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define NS_RTO_MIN   200
#define NS_RTO_MAX   60000
#define NS_RTO_INIT  1000

typedef struct {
    uint32_t rto;
    uint32_t srtt;
    uint32_t rttvar;
    int backoff_count;
    int has_measurement;
} ns_rto_timer_t;

void ns_rto_init(ns_rto_timer_t *t) {
    t->rto = NS_RTO_INIT;
    t->srtt = 0;
    t->rttvar = 0;
    t->backoff_count = 0;
    t->has_measurement = 0;
}

void ns_rto_update(ns_rto_timer_t *t, uint32_t rtt_ms) {
    if (!t->has_measurement) {
        t->srtt = rtt_ms;
        t->rttvar = rtt_ms / 2;
        t->has_measurement = 1;
    } else {
        int delta = (int)rtt_ms - (int)t->srtt;
        if (delta < 0) delta = -delta;
        t->rttvar = (3 * t->rttvar + (uint32_t)delta) / 4;
        t->srtt = (7 * t->srtt + rtt_ms) / 8;
    }
    t->rto = t->srtt + 4 * t->rttvar;
    if (t->rto < NS_RTO_MIN) t->rto = NS_RTO_MIN;
    if (t->rto > NS_RTO_MAX) t->rto = NS_RTO_MAX;
    t->backoff_count = 0;
}

void ns_rto_backoff(ns_rto_timer_t *t) {
    t->rto *= 2;
    if (t->rto > NS_RTO_MAX) t->rto = NS_RTO_MAX;
    t->backoff_count++;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1515: Retransmission timer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1515: Output should not be empty");
    assert!(
        code.contains("fn ns_rto_init"),
        "C1515: Should contain ns_rto_init"
    );
    assert!(
        code.contains("fn ns_rto_update"),
        "C1515: Should contain ns_rto_update"
    );
}

// ============================================================================
// C1516-C1520: Routing
// ============================================================================

/// C1516: Routing table with prefix/mask entries and next-hop lookup
#[test]
fn c1516_routing_table() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define NS_RT_MAX_ENTRIES 32

typedef struct {
    uint32_t network;
    uint32_t mask;
    uint32_t gateway;
    uint8_t prefix_len;
    int active;
} ns_route_entry_t;

typedef struct {
    ns_route_entry_t routes[NS_RT_MAX_ENTRIES];
    int count;
} ns_routing_table_t;

void ns_rt_init(ns_routing_table_t *rt) {
    rt->count = 0;
    int i;
    for (i = 0; i < NS_RT_MAX_ENTRIES; i++) {
        rt->routes[i].active = 0;
    }
}

int ns_rt_add(ns_routing_table_t *rt, uint32_t net, uint8_t prefix_len, uint32_t gw) {
    if (rt->count >= NS_RT_MAX_ENTRIES) return -1;
    int i;
    for (i = 0; i < NS_RT_MAX_ENTRIES; i++) {
        if (!rt->routes[i].active) {
            uint32_t mask = (prefix_len == 0) ? 0 : ~((1u << (32 - prefix_len)) - 1);
            rt->routes[i].network = net & mask;
            rt->routes[i].mask = mask;
            rt->routes[i].gateway = gw;
            rt->routes[i].prefix_len = prefix_len;
            rt->routes[i].active = 1;
            rt->count++;
            return i;
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1516: Routing table should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1516: Output should not be empty");
    assert!(
        code.contains("fn ns_rt_init"),
        "C1516: Should contain ns_rt_init"
    );
    assert!(
        code.contains("fn ns_rt_add"),
        "C1516: Should contain ns_rt_add"
    );
}

/// C1517: Longest prefix match for IP routing decisions
#[test]
fn c1517_longest_prefix_match() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define NS_LPM_SIZE 16

typedef struct {
    uint32_t prefix;
    uint32_t mask;
    uint8_t prefix_len;
    uint32_t next_hop;
    int valid;
} ns_lpm_entry_t;

typedef struct {
    ns_lpm_entry_t entries[NS_LPM_SIZE];
    int count;
} ns_lpm_table_t;

void ns_lpm_init(ns_lpm_table_t *t) {
    t->count = 0;
    int i;
    for (i = 0; i < NS_LPM_SIZE; i++) t->entries[i].valid = 0;
}

int ns_lpm_lookup(const ns_lpm_table_t *t, uint32_t addr, uint32_t *next_hop) {
    int best = -1;
    uint8_t best_len = 0;
    int i;
    for (i = 0; i < NS_LPM_SIZE; i++) {
        if (t->entries[i].valid &&
            (addr & t->entries[i].mask) == t->entries[i].prefix) {
            if (best == -1 || t->entries[i].prefix_len > best_len) {
                best = i;
                best_len = t->entries[i].prefix_len;
            }
        }
    }
    if (best >= 0) {
        *next_hop = t->entries[best].next_hop;
        return 0;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1517: Longest prefix match should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1517: Output should not be empty");
    assert!(
        code.contains("fn ns_lpm_init"),
        "C1517: Should contain ns_lpm_init"
    );
    assert!(
        code.contains("fn ns_lpm_lookup"),
        "C1517: Should contain ns_lpm_lookup"
    );
}

/// C1518: Next hop lookup with interface selection and metric comparison
#[test]
fn c1518_next_hop_lookup() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define NS_NH_MAX 8

typedef struct {
    uint32_t gateway;
    int interface_id;
    int metric;
    int active;
} ns_nexthop_t;

typedef struct {
    ns_nexthop_t hops[NS_NH_MAX];
    int count;
} ns_nexthop_table_t;

void ns_nh_init(ns_nexthop_table_t *t) {
    t->count = 0;
    int i;
    for (i = 0; i < NS_NH_MAX; i++) t->hops[i].active = 0;
}

int ns_nh_add(ns_nexthop_table_t *t, uint32_t gw, int iface, int metric) {
    if (t->count >= NS_NH_MAX) return -1;
    int i;
    for (i = 0; i < NS_NH_MAX; i++) {
        if (!t->hops[i].active) {
            t->hops[i].gateway = gw;
            t->hops[i].interface_id = iface;
            t->hops[i].metric = metric;
            t->hops[i].active = 1;
            t->count++;
            return 0;
        }
    }
    return -1;
}

int ns_nh_best(const ns_nexthop_table_t *t, uint32_t *gw) {
    int best = -1;
    int best_metric = 0x7FFFFFFF;
    int i;
    for (i = 0; i < NS_NH_MAX; i++) {
        if (t->hops[i].active && t->hops[i].metric < best_metric) {
            best = i;
            best_metric = t->hops[i].metric;
        }
    }
    if (best >= 0) {
        *gw = t->hops[best].gateway;
        return 0;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1518: Next hop lookup should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1518: Output should not be empty");
    assert!(
        code.contains("fn ns_nh_init"),
        "C1518: Should contain ns_nh_init"
    );
    assert!(
        code.contains("fn ns_nh_best"),
        "C1518: Should contain ns_nh_best"
    );
}

/// C1519: ARP cache with MAC-to-IP mapping and TTL expiration
#[test]
fn c1519_arp_cache() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define NS_ARP_SIZE 32
#define NS_ARP_MAC_LEN 6

typedef struct {
    uint32_t ip_addr;
    uint8_t mac_addr[NS_ARP_MAC_LEN];
    uint32_t ttl;
    int valid;
} ns_arp_entry_t;

typedef struct {
    ns_arp_entry_t entries[NS_ARP_SIZE];
    int count;
} ns_arp_cache_t;

void ns_arp_init(ns_arp_cache_t *c) {
    c->count = 0;
    int i;
    for (i = 0; i < NS_ARP_SIZE; i++) c->entries[i].valid = 0;
}

int ns_arp_lookup(const ns_arp_cache_t *c, uint32_t ip, uint8_t *mac_out) {
    int i;
    for (i = 0; i < NS_ARP_SIZE; i++) {
        if (c->entries[i].valid && c->entries[i].ip_addr == ip && c->entries[i].ttl > 0) {
            int j;
            for (j = 0; j < NS_ARP_MAC_LEN; j++) {
                mac_out[j] = c->entries[i].mac_addr[j];
            }
            return 0;
        }
    }
    return -1;
}

int ns_arp_insert(ns_arp_cache_t *c, uint32_t ip, const uint8_t *mac, uint32_t ttl) {
    int i;
    for (i = 0; i < NS_ARP_SIZE; i++) {
        if (c->entries[i].valid && c->entries[i].ip_addr == ip) {
            int j;
            for (j = 0; j < NS_ARP_MAC_LEN; j++) c->entries[i].mac_addr[j] = mac[j];
            c->entries[i].ttl = ttl;
            return 0;
        }
    }
    for (i = 0; i < NS_ARP_SIZE; i++) {
        if (!c->entries[i].valid) {
            c->entries[i].ip_addr = ip;
            int j;
            for (j = 0; j < NS_ARP_MAC_LEN; j++) c->entries[i].mac_addr[j] = mac[j];
            c->entries[i].ttl = ttl;
            c->entries[i].valid = 1;
            c->count++;
            return 0;
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1519: ARP cache should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1519: Output should not be empty");
    assert!(
        code.contains("fn ns_arp_init"),
        "C1519: Should contain ns_arp_init"
    );
    assert!(
        code.contains("fn ns_arp_lookup"),
        "C1519: Should contain ns_arp_lookup"
    );
}

/// C1520: Routing metrics computation with hop count, latency, and bandwidth
#[test]
fn c1520_routing_metrics() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t hop_count;
    uint32_t latency_us;
    uint32_t bandwidth_kbps;
    uint32_t reliability;
} ns_route_metric_t;

void ns_metric_init(ns_route_metric_t *m) {
    m->hop_count = 0;
    m->latency_us = 0;
    m->bandwidth_kbps = 0;
    m->reliability = 100;
}

uint32_t ns_metric_composite(const ns_route_metric_t *m) {
    uint32_t score = m->hop_count * 1000;
    score += m->latency_us / 100;
    if (m->bandwidth_kbps > 0) {
        score += 10000000 / m->bandwidth_kbps;
    }
    score += (100 - m->reliability) * 500;
    return score;
}

int ns_metric_compare(const ns_route_metric_t *a, const ns_route_metric_t *b) {
    uint32_t sa = ns_metric_composite(a);
    uint32_t sb = ns_metric_composite(b);
    if (sa < sb) return -1;
    if (sa > sb) return 1;
    return 0;
}

void ns_metric_add_hop(ns_route_metric_t *m, uint32_t lat, uint32_t bw) {
    m->hop_count++;
    m->latency_us += lat;
    if (m->bandwidth_kbps == 0 || bw < m->bandwidth_kbps) {
        m->bandwidth_kbps = bw;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1520: Routing metrics should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1520: Output should not be empty");
    assert!(
        code.contains("fn ns_metric_init"),
        "C1520: Should contain ns_metric_init"
    );
    assert!(
        code.contains("fn ns_metric_composite"),
        "C1520: Should contain ns_metric_composite"
    );
}

// ============================================================================
// C1521-C1525: Application Layer
// ============================================================================

/// C1521: DNS query builder with domain name encoding and question record
#[test]
fn c1521_dns_query_builder() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

typedef struct {
    uint8_t buf[512];
    int pos;
    uint16_t id;
    uint16_t qdcount;
} ns_dns_query_t;

void ns_dns_query_init(ns_dns_query_t *q, uint16_t id) {
    q->pos = 0;
    q->id = id;
    q->qdcount = 0;
    /* Write header: ID, flags=0x0100 (RD), qdcount placeholder */
    q->buf[q->pos++] = (uint8_t)(id >> 8);
    q->buf[q->pos++] = (uint8_t)(id & 0xFF);
    q->buf[q->pos++] = 0x01; /* RD flag */
    q->buf[q->pos++] = 0x00;
    int i;
    for (i = 0; i < 8; i++) q->buf[q->pos++] = 0;
}

int ns_dns_encode_name(ns_dns_query_t *q, const char *name) {
    int start = q->pos;
    int label_start = q->pos++;
    int len = 0;
    while (*name) {
        if (*name == '.') {
            q->buf[label_start] = (uint8_t)len;
            label_start = q->pos++;
            len = 0;
        } else {
            q->buf[q->pos++] = (uint8_t)*name;
            len++;
        }
        name++;
    }
    q->buf[label_start] = (uint8_t)len;
    q->buf[q->pos++] = 0;
    return q->pos - start;
}

void ns_dns_add_question(ns_dns_query_t *q, uint16_t qtype) {
    q->buf[q->pos++] = (uint8_t)(qtype >> 8);
    q->buf[q->pos++] = (uint8_t)(qtype & 0xFF);
    q->buf[q->pos++] = 0x00;
    q->buf[q->pos++] = 0x01; /* IN class */
    q->qdcount++;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1521: DNS query builder should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1521: Output should not be empty");
    assert!(
        code.contains("fn ns_dns_query_init"),
        "C1521: Should contain ns_dns_query_init"
    );
    assert!(
        code.contains("fn ns_dns_encode_name"),
        "C1521: Should contain ns_dns_encode_name"
    );
}

/// C1522: HTTP header parser extracting key-value pairs from header lines
#[test]
fn c1522_http_header_parser() {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    char name[64];
    char value[128];
} ns_http_hdr_t;

typedef struct {
    ns_http_hdr_t headers[16];
    int count;
} ns_http_headers_t;

void ns_http_headers_init(ns_http_headers_t *h) {
    h->count = 0;
}

int ns_http_parse_header(ns_http_headers_t *h, const char *line, int len) {
    if (h->count >= 16) return -1;
    int colon = -1;
    int i;
    for (i = 0; i < len; i++) {
        if (line[i] == ':') { colon = i; break; }
    }
    if (colon < 0) return -2;
    int ni = 0;
    for (i = 0; i < colon && ni < 63; i++) {
        h->headers[h->count].name[ni++] = line[i];
    }
    h->headers[h->count].name[ni] = '\0';
    int vi = 0;
    i = colon + 1;
    while (i < len && line[i] == ' ') i++;
    for (; i < len && vi < 127; i++) {
        h->headers[h->count].value[vi++] = line[i];
    }
    h->headers[h->count].value[vi] = '\0';
    h->count++;
    return 0;
}

int ns_http_find_header(const ns_http_headers_t *h, const char *name) {
    int i;
    for (i = 0; i < h->count; i++) {
        const char *a = h->headers[i].name;
        const char *b = name;
        int match = 1;
        while (*a && *b) {
            if (*a != *b) { match = 0; break; }
            a++; b++;
        }
        if (match && *a == '\0' && *b == '\0') return i;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1522: HTTP header parser should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1522: Output should not be empty");
    assert!(
        code.contains("fn ns_http_headers_init"),
        "C1522: Should contain ns_http_headers_init"
    );
    assert!(
        code.contains("fn ns_http_parse_header"),
        "C1522: Should contain ns_http_parse_header"
    );
}

/// C1523: TLS record layer framing with content type and length
#[test]
fn c1523_tls_record_layer() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

#define NS_TLS_CHANGE_CIPHER 20
#define NS_TLS_ALERT         21
#define NS_TLS_HANDSHAKE     22
#define NS_TLS_APP_DATA      23
#define NS_TLS_MAX_RECORD    16384

typedef struct {
    uint8_t content_type;
    uint8_t version_major;
    uint8_t version_minor;
    uint16_t length;
} ns_tls_record_hdr_t;

int ns_tls_build_header(ns_tls_record_hdr_t *hdr, uint8_t ctype, uint16_t payload_len) {
    if (payload_len > NS_TLS_MAX_RECORD) return -1;
    hdr->content_type = ctype;
    hdr->version_major = 3;
    hdr->version_minor = 3;
    hdr->length = payload_len;
    return 0;
}

int ns_tls_serialize(const ns_tls_record_hdr_t *hdr, uint8_t *out) {
    out[0] = hdr->content_type;
    out[1] = hdr->version_major;
    out[2] = hdr->version_minor;
    out[3] = (uint8_t)(hdr->length >> 8);
    out[4] = (uint8_t)(hdr->length & 0xFF);
    return 5;
}

int ns_tls_parse(const uint8_t *buf, int len, ns_tls_record_hdr_t *hdr) {
    if (len < 5) return -1;
    hdr->content_type = buf[0];
    hdr->version_major = buf[1];
    hdr->version_minor = buf[2];
    hdr->length = ((uint16_t)buf[3] << 8) | buf[4];
    if (hdr->length > NS_TLS_MAX_RECORD) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1523: TLS record layer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1523: Output should not be empty");
    assert!(
        code.contains("fn ns_tls_build_header"),
        "C1523: Should contain ns_tls_build_header"
    );
    assert!(
        code.contains("fn ns_tls_parse"),
        "C1523: Should contain ns_tls_parse"
    );
}

/// C1524: DHCP option parser/builder for TLV-encoded options
#[test]
fn c1524_dhcp_options() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

#define NS_DHCP_OPT_PAD       0
#define NS_DHCP_OPT_SUBNET    1
#define NS_DHCP_OPT_ROUTER    3
#define NS_DHCP_OPT_DNS       6
#define NS_DHCP_OPT_LEASE     51
#define NS_DHCP_OPT_MSG_TYPE  53
#define NS_DHCP_OPT_END       255

typedef struct {
    uint8_t buf[256];
    int pos;
} ns_dhcp_opts_t;

void ns_dhcp_opts_init(ns_dhcp_opts_t *opts) {
    opts->pos = 0;
}

int ns_dhcp_opts_add(ns_dhcp_opts_t *opts, uint8_t code, uint8_t len, const uint8_t *data) {
    if (opts->pos + 2 + len > 255) return -1;
    opts->buf[opts->pos++] = code;
    opts->buf[opts->pos++] = len;
    int i;
    for (i = 0; i < len; i++) {
        opts->buf[opts->pos++] = data[i];
    }
    return 0;
}

int ns_dhcp_opts_add_ip(ns_dhcp_opts_t *opts, uint8_t code, uint32_t ip) {
    uint8_t data[4];
    data[0] = (uint8_t)(ip >> 24);
    data[1] = (uint8_t)(ip >> 16);
    data[2] = (uint8_t)(ip >> 8);
    data[3] = (uint8_t)(ip & 0xFF);
    return ns_dhcp_opts_add(opts, code, 4, data);
}

void ns_dhcp_opts_end(ns_dhcp_opts_t *opts) {
    if (opts->pos < 256) {
        opts->buf[opts->pos++] = NS_DHCP_OPT_END;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1524: DHCP options should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1524: Output should not be empty");
    assert!(
        code.contains("fn ns_dhcp_opts_init"),
        "C1524: Should contain ns_dhcp_opts_init"
    );
    assert!(
        code.contains("fn ns_dhcp_opts_add"),
        "C1524: Should contain ns_dhcp_opts_add"
    );
}

/// C1525: NTP timestamp conversion between epoch seconds and NTP 64-bit format
#[test]
fn c1525_ntp_timestamp() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define NS_NTP_EPOCH_OFFSET 2208988800u

typedef struct {
    uint32_t seconds;
    uint32_t fraction;
} ns_ntp_timestamp_t;

void ns_ntp_from_epoch(ns_ntp_timestamp_t *ntp, uint32_t epoch_secs, uint32_t frac_us) {
    ntp->seconds = epoch_secs + NS_NTP_EPOCH_OFFSET;
    /* Convert microseconds to NTP fraction: frac * 2^32 / 1000000 */
    uint64_t f = (uint64_t)frac_us * 4294967296u;
    ntp->fraction = (uint32_t)(f / 1000000u);
}

uint32_t ns_ntp_to_epoch(const ns_ntp_timestamp_t *ntp) {
    if (ntp->seconds < NS_NTP_EPOCH_OFFSET) return 0;
    return ntp->seconds - NS_NTP_EPOCH_OFFSET;
}

int ns_ntp_compare(const ns_ntp_timestamp_t *a, const ns_ntp_timestamp_t *b) {
    if (a->seconds < b->seconds) return -1;
    if (a->seconds > b->seconds) return 1;
    if (a->fraction < b->fraction) return -1;
    if (a->fraction > b->fraction) return 1;
    return 0;
}

uint32_t ns_ntp_diff_ms(const ns_ntp_timestamp_t *a, const ns_ntp_timestamp_t *b) {
    uint32_t sec_diff = a->seconds - b->seconds;
    uint32_t ms = sec_diff * 1000;
    if (a->fraction > b->fraction) {
        uint64_t fdiff = (uint64_t)(a->fraction - b->fraction) * 1000;
        ms += (uint32_t)(fdiff >> 32);
    }
    return ms;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1525: NTP timestamp should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1525: Output should not be empty");
    assert!(
        code.contains("fn ns_ntp_from_epoch"),
        "C1525: Should contain ns_ntp_from_epoch"
    );
    assert!(
        code.contains("fn ns_ntp_to_epoch"),
        "C1525: Should contain ns_ntp_to_epoch"
    );
}
