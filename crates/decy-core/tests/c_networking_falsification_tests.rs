//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//! C701-C725: Networking/Sockets Domain
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise networking and socket programming patterns commonly
//! found in production C codebases: socket abstractions, protocol parsers,
//! connection managers, rate limiters, and network utility algorithms.
//! All tests use self-contained C code with no #include directives.
//!
//! Organization:
//! - C701-C705: Socket primitives (TCP, UDP, select, sockaddr, DNS)
//! - C706-C710: Application protocols (HTTP parser, URL parser, ring buffer, conn pool, rate limiter)
//! - C711-C715: Network utilities (IP conversion, checksum, fragmentation, ARP cache, routing)
//! - C716-C720: Low-level network (byte order, CIDR, port scanner, DNS builder, SOCKS proxy)
//! - C721-C725: Modern protocols (WebSocket, MQTT, interface stats, bandwidth throttle, timeout)

// ============================================================================
// C701-C705: Socket Primitives
// ============================================================================

/// C701: TCP socket creation and binding (socket, bind, listen simulation)
#[test]
fn c701_tcp_socket_create_bind_listen() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;
typedef int int32_t;

typedef struct {
    int fd;
    int domain;
    int type;
    int protocol;
    int bound;
    int listening;
    int backlog;
    uint32_t bind_addr;
    uint16_t bind_port;
} netsock_tcp_socket_t;

int netsock_tcp_create(netsock_tcp_socket_t *sock, int domain, int type) {
    static int next_fd = 3;
    sock->fd = next_fd++;
    sock->domain = domain;
    sock->type = type;
    sock->protocol = 6;
    sock->bound = 0;
    sock->listening = 0;
    sock->backlog = 0;
    sock->bind_addr = 0;
    sock->bind_port = 0;
    return sock->fd;
}

int netsock_tcp_bind(netsock_tcp_socket_t *sock, uint32_t addr, uint16_t port) {
    if (sock->fd < 0) return -1;
    if (sock->bound) return -2;
    if (port == 0) return -3;
    sock->bind_addr = addr;
    sock->bind_port = port;
    sock->bound = 1;
    return 0;
}

int netsock_tcp_listen(netsock_tcp_socket_t *sock, int backlog) {
    if (!sock->bound) return -1;
    if (sock->listening) return -2;
    if (backlog <= 0) backlog = 128;
    sock->listening = 1;
    sock->backlog = backlog;
    return 0;
}

int netsock_tcp_is_ready(const netsock_tcp_socket_t *sock) {
    return sock->bound && sock->listening;
}

int netsock_tcp_close(netsock_tcp_socket_t *sock) {
    sock->fd = -1;
    sock->bound = 0;
    sock->listening = 0;
    return 0;
}

int main(void) {
    netsock_tcp_socket_t sock;
    int fd = netsock_tcp_create(&sock, 2, 1);
    if (fd < 3) return 1;
    if (netsock_tcp_bind(&sock, 0, 8080) != 0) return 2;
    if (netsock_tcp_listen(&sock, 64) != 0) return 3;
    if (!netsock_tcp_is_ready(&sock)) return 4;
    netsock_tcp_close(&sock);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C701: TCP socket create/bind/listen should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C701: should produce non-empty output");
    assert!(code.contains("fn netsock_tcp_create"), "C701: should contain netsock_tcp_create");
    assert!(code.contains("fn netsock_tcp_bind"), "C701: should contain netsock_tcp_bind");
    assert!(code.contains("fn netsock_tcp_listen"), "C701: should contain netsock_tcp_listen");
}

/// C702: UDP socket send/receive simulation
#[test]
fn c702_udp_socket_send_receive() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

#define NETSOCK_UDP_MAX_DGRAM 1500

typedef struct {
    uint32_t addr;
    uint16_t port;
} netsock_udp_endpoint_t;

typedef struct {
    uint8_t data[1500];
    int length;
    netsock_udp_endpoint_t src;
    netsock_udp_endpoint_t dst;
} netsock_udp_datagram_t;

typedef struct {
    int fd;
    uint32_t local_addr;
    uint16_t local_port;
    int bound;
    netsock_udp_datagram_t rx_buffer[16];
    int rx_head;
    int rx_tail;
    int rx_count;
} netsock_udp_socket_t;

void netsock_udp_init(netsock_udp_socket_t *sock) {
    sock->fd = -1;
    sock->local_addr = 0;
    sock->local_port = 0;
    sock->bound = 0;
    sock->rx_head = 0;
    sock->rx_tail = 0;
    sock->rx_count = 0;
}

int netsock_udp_bind(netsock_udp_socket_t *sock, uint32_t addr, uint16_t port) {
    if (sock->bound) return -1;
    sock->local_addr = addr;
    sock->local_port = port;
    sock->bound = 1;
    sock->fd = 4;
    return 0;
}

int netsock_udp_enqueue_rx(netsock_udp_socket_t *sock, const uint8_t *data,
                           int len, uint32_t src_addr, uint16_t src_port) {
    if (sock->rx_count >= 16) return -1;
    if (len > NETSOCK_UDP_MAX_DGRAM) return -2;
    netsock_udp_datagram_t *dgram = &sock->rx_buffer[sock->rx_tail];
    int i;
    for (i = 0; i < len; i++) {
        dgram->data[i] = data[i];
    }
    dgram->length = len;
    dgram->src.addr = src_addr;
    dgram->src.port = src_port;
    sock->rx_tail = (sock->rx_tail + 1) % 16;
    sock->rx_count++;
    return 0;
}

int netsock_udp_recv(netsock_udp_socket_t *sock, uint8_t *buf, int buflen,
                     netsock_udp_endpoint_t *from) {
    if (sock->rx_count == 0) return -1;
    netsock_udp_datagram_t *dgram = &sock->rx_buffer[sock->rx_head];
    int copy_len = dgram->length;
    if (copy_len > buflen) copy_len = buflen;
    int i;
    for (i = 0; i < copy_len; i++) {
        buf[i] = dgram->data[i];
    }
    if (from) {
        from->addr = dgram->src.addr;
        from->port = dgram->src.port;
    }
    sock->rx_head = (sock->rx_head + 1) % 16;
    sock->rx_count--;
    return copy_len;
}

int netsock_udp_pending(const netsock_udp_socket_t *sock) {
    return sock->rx_count;
}

int main(void) {
    netsock_udp_socket_t sock;
    netsock_udp_init(&sock);
    netsock_udp_bind(&sock, 0, 5000);
    uint8_t test_data[4];
    test_data[0] = 1; test_data[1] = 2; test_data[2] = 3; test_data[3] = 4;
    netsock_udp_enqueue_rx(&sock, test_data, 4, 0x0A000001, 9999);
    uint8_t buf[64];
    netsock_udp_endpoint_t from;
    int n = netsock_udp_recv(&sock, buf, 64, &from);
    if (n != 4) return 1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C702: UDP socket send/receive should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C702: should produce non-empty output");
    assert!(code.contains("fn netsock_udp_init"), "C702: should contain netsock_udp_init");
    assert!(code.contains("fn netsock_udp_recv"), "C702: should contain netsock_udp_recv");
}

/// C703: Non-blocking socket with select() simulation
#[test]
fn c703_nonblocking_socket_select() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;

#define NETSOCK_SEL_MAXFD 64
#define NETSOCK_SEL_READ  1
#define NETSOCK_SEL_WRITE 2
#define NETSOCK_SEL_ERROR 4

typedef struct {
    int fds[64];
    int events[64];
    int ready[64];
    int count;
} netsock_fdset_t;

void netsock_fdset_init(netsock_fdset_t *set) {
    set->count = 0;
    int i;
    for (i = 0; i < NETSOCK_SEL_MAXFD; i++) {
        set->fds[i] = -1;
        set->events[i] = 0;
        set->ready[i] = 0;
    }
}

int netsock_fdset_add(netsock_fdset_t *set, int fd, int events) {
    if (set->count >= NETSOCK_SEL_MAXFD) return -1;
    if (fd < 0) return -2;
    set->fds[set->count] = fd;
    set->events[set->count] = events;
    set->ready[set->count] = 0;
    set->count++;
    return 0;
}

void netsock_fdset_mark_ready(netsock_fdset_t *set, int fd, int ready_events) {
    int i;
    for (i = 0; i < set->count; i++) {
        if (set->fds[i] == fd) {
            set->ready[i] = ready_events & set->events[i];
        }
    }
}

int netsock_fdset_is_ready(const netsock_fdset_t *set, int fd) {
    int i;
    for (i = 0; i < set->count; i++) {
        if (set->fds[i] == fd) {
            return set->ready[i];
        }
    }
    return 0;
}

int netsock_fdset_count_ready(const netsock_fdset_t *set) {
    int n = 0;
    int i;
    for (i = 0; i < set->count; i++) {
        if (set->ready[i] != 0) n++;
    }
    return n;
}

int netsock_select_sim(netsock_fdset_t *set, int timeout_ms) {
    int ready = netsock_fdset_count_ready(set);
    if (ready > 0 || timeout_ms == 0) return ready;
    return 0;
}

int main(void) {
    netsock_fdset_t fds;
    netsock_fdset_init(&fds);
    netsock_fdset_add(&fds, 3, NETSOCK_SEL_READ);
    netsock_fdset_add(&fds, 4, NETSOCK_SEL_WRITE);
    netsock_fdset_mark_ready(&fds, 3, NETSOCK_SEL_READ);
    int n = netsock_select_sim(&fds, 1000);
    if (n != 1) return 1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C703: Non-blocking socket select should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C703: should produce non-empty output");
    assert!(code.contains("fn netsock_fdset_init"), "C703: should contain netsock_fdset_init");
    assert!(code.contains("fn netsock_select_sim"), "C703: should contain netsock_select_sim");
}

/// C704: Socket address structures (sockaddr_in simulation)
#[test]
fn c704_socket_address_structures() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

typedef struct {
    uint16_t family;
    uint16_t port;
    uint32_t addr;
    uint8_t zero[8];
} netsock_sockaddr_in_t;

typedef struct {
    uint16_t family;
    uint8_t data[14];
} netsock_sockaddr_t;

void netsock_addr_init(netsock_sockaddr_in_t *sa, uint32_t addr, uint16_t port) {
    sa->family = 2;
    sa->port = ((port >> 8) & 0xFF) | ((port & 0xFF) << 8);
    sa->addr = addr;
    int i;
    for (i = 0; i < 8; i++) sa->zero[i] = 0;
}

uint32_t netsock_addr_make_ipv4(uint8_t a, uint8_t b, uint8_t c, uint8_t d) {
    return ((uint32_t)a << 24) | ((uint32_t)b << 16) |
           ((uint32_t)c << 8) | (uint32_t)d;
}

uint16_t netsock_addr_get_port(const netsock_sockaddr_in_t *sa) {
    return ((sa->port >> 8) & 0xFF) | ((sa->port & 0xFF) << 8);
}

uint32_t netsock_addr_get_ip(const netsock_sockaddr_in_t *sa) {
    return sa->addr;
}

int netsock_addr_is_loopback(const netsock_sockaddr_in_t *sa) {
    return (sa->addr >> 24) == 127;
}

int netsock_addr_is_any(const netsock_sockaddr_in_t *sa) {
    return sa->addr == 0;
}

int netsock_addr_equal(const netsock_sockaddr_in_t *a, const netsock_sockaddr_in_t *b) {
    return a->addr == b->addr && a->port == b->port;
}

int main(void) {
    netsock_sockaddr_in_t sa;
    uint32_t ip = netsock_addr_make_ipv4(127, 0, 0, 1);
    netsock_addr_init(&sa, ip, 8080);
    if (!netsock_addr_is_loopback(&sa)) return 1;
    if (netsock_addr_get_port(&sa) != 8080) return 2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C704: Socket address structures should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C704: should produce non-empty output");
    assert!(code.contains("fn netsock_addr_init"), "C704: should contain netsock_addr_init");
    assert!(code.contains("fn netsock_addr_make_ipv4"), "C704: should contain netsock_addr_make_ipv4");
}

/// C705: DNS lookup simulation (linked list of addresses)
#[test]
fn c705_dns_lookup_simulation() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;

#define NETSOCK_DNS_MAX_RESULTS 8
#define NETSOCK_DNS_MAX_NAME 64

typedef struct {
    uint32_t addresses[8];
    int count;
    char name[64];
    int ttl;
    int resolved;
} netsock_dns_result_t;

typedef struct {
    char name[64];
    uint32_t addresses[8];
    int addr_count;
    int ttl;
    int valid;
} netsock_dns_cache_entry_t;

typedef struct {
    netsock_dns_cache_entry_t entries[32];
    int count;
} netsock_dns_cache_t;

void netsock_dns_cache_init(netsock_dns_cache_t *cache) {
    cache->count = 0;
    int i;
    for (i = 0; i < 32; i++) {
        cache->entries[i].valid = 0;
    }
}

static int netsock_dns_str_eq(const char *a, const char *b) {
    int i;
    for (i = 0; i < 64; i++) {
        if (a[i] != b[i]) return 0;
        if (a[i] == 0) return 1;
    }
    return 1;
}

static void netsock_dns_str_copy(char *dst, const char *src) {
    int i;
    for (i = 0; i < 63; i++) {
        dst[i] = src[i];
        if (src[i] == 0) return;
    }
    dst[63] = 0;
}

int netsock_dns_cache_lookup(const netsock_dns_cache_t *cache, const char *name,
                             netsock_dns_result_t *result) {
    int i;
    for (i = 0; i < cache->count; i++) {
        if (cache->entries[i].valid && netsock_dns_str_eq(cache->entries[i].name, name)) {
            int j;
            for (j = 0; j < cache->entries[i].addr_count; j++) {
                result->addresses[j] = cache->entries[i].addresses[j];
            }
            result->count = cache->entries[i].addr_count;
            netsock_dns_str_copy(result->name, name);
            result->ttl = cache->entries[i].ttl;
            result->resolved = 1;
            return 0;
        }
    }
    result->resolved = 0;
    return -1;
}

int netsock_dns_cache_insert(netsock_dns_cache_t *cache, const char *name,
                             uint32_t addr, int ttl) {
    int i;
    for (i = 0; i < cache->count; i++) {
        if (cache->entries[i].valid && netsock_dns_str_eq(cache->entries[i].name, name)) {
            int c = cache->entries[i].addr_count;
            if (c >= 8) return -1;
            cache->entries[i].addresses[c] = addr;
            cache->entries[i].addr_count = c + 1;
            return 0;
        }
    }
    if (cache->count >= 32) return -2;
    netsock_dns_cache_entry_t *e = &cache->entries[cache->count];
    netsock_dns_str_copy(e->name, name);
    e->addresses[0] = addr;
    e->addr_count = 1;
    e->ttl = ttl;
    e->valid = 1;
    cache->count++;
    return 0;
}

int main(void) {
    netsock_dns_cache_t cache;
    netsock_dns_cache_init(&cache);
    netsock_dns_cache_insert(&cache, "example.com", 0x0A000001, 300);
    netsock_dns_result_t result;
    int rc = netsock_dns_cache_lookup(&cache, "example.com", &result);
    if (rc != 0) return 1;
    if (result.count != 1) return 2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C705: DNS lookup simulation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C705: should produce non-empty output");
    assert!(code.contains("fn netsock_dns_cache_init"), "C705: should contain netsock_dns_cache_init");
    assert!(code.contains("fn netsock_dns_cache_lookup"), "C705: should contain netsock_dns_cache_lookup");
}

// ============================================================================
// C706-C710: Application Protocols
// ============================================================================

/// C706: HTTP request parser (state machine)
#[test]
fn c706_http_request_parser() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

#define NETHTTP_METHOD_GET    1
#define NETHTTP_METHOD_POST   2
#define NETHTTP_METHOD_PUT    3
#define NETHTTP_METHOD_DELETE 4

#define NETHTTP_STATE_METHOD 0
#define NETHTTP_STATE_URI    1
#define NETHTTP_STATE_VERSION 2
#define NETHTTP_STATE_HEADER_NAME 3
#define NETHTTP_STATE_HEADER_VALUE 4
#define NETHTTP_STATE_BODY   5
#define NETHTTP_STATE_DONE   6
#define NETHTTP_STATE_ERROR  7

typedef struct {
    int method;
    char uri[256];
    int uri_len;
    int state;
    int content_length;
    int header_count;
    int major_version;
    int minor_version;
} nethttp_request_t;

void nethttp_request_init(nethttp_request_t *req) {
    req->method = 0;
    req->uri_len = 0;
    req->state = NETHTTP_STATE_METHOD;
    req->content_length = 0;
    req->header_count = 0;
    req->major_version = 1;
    req->minor_version = 1;
}

static int nethttp_is_space(char c) {
    return c == ' ' || c == '\t';
}

int nethttp_parse_method(nethttp_request_t *req, const char *data, int len) {
    if (len >= 3 && data[0] == 'G' && data[1] == 'E' && data[2] == 'T') {
        req->method = NETHTTP_METHOD_GET;
        return 3;
    }
    if (len >= 4 && data[0] == 'P' && data[1] == 'O' && data[2] == 'S' && data[3] == 'T') {
        req->method = NETHTTP_METHOD_POST;
        return 4;
    }
    if (len >= 3 && data[0] == 'P' && data[1] == 'U' && data[2] == 'T') {
        req->method = NETHTTP_METHOD_PUT;
        return 3;
    }
    return -1;
}

int nethttp_parse_uri(nethttp_request_t *req, const char *data, int len) {
    int i;
    req->uri_len = 0;
    for (i = 0; i < len && i < 255; i++) {
        if (nethttp_is_space(data[i])) break;
        req->uri[req->uri_len++] = data[i];
    }
    req->uri[req->uri_len] = 0;
    return i;
}

int nethttp_is_complete(const nethttp_request_t *req) {
    return req->state == NETHTTP_STATE_DONE;
}

const char *nethttp_method_name(int method) {
    if (method == NETHTTP_METHOD_GET) return "GET";
    if (method == NETHTTP_METHOD_POST) return "POST";
    if (method == NETHTTP_METHOD_PUT) return "PUT";
    if (method == NETHTTP_METHOD_DELETE) return "DELETE";
    return "UNKNOWN";
}

int main(void) {
    nethttp_request_t req;
    nethttp_request_init(&req);
    const char *line = "GET /index.html HTTP/1.1";
    int consumed = nethttp_parse_method(&req, line, 24);
    if (consumed != 3) return 1;
    if (req.method != NETHTTP_METHOD_GET) return 2;
    int skip = consumed;
    while (nethttp_is_space(line[skip])) skip++;
    nethttp_parse_uri(&req, line + skip, 24 - skip);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C706: HTTP request parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C706: should produce non-empty output");
    assert!(code.contains("fn nethttp_request_init"), "C706: should contain nethttp_request_init");
    assert!(code.contains("fn nethttp_parse_method"), "C706: should contain nethttp_parse_method");
}

/// C707: URL parser (scheme, host, port, path)
#[test]
fn c707_url_parser() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned short uint16_t;

typedef struct {
    char scheme[16];
    char host[128];
    uint16_t port;
    char path[256];
    int valid;
    int scheme_len;
    int host_len;
    int path_len;
} neturl_parsed_t;

static int neturl_is_alpha(char c) {
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z');
}

static int neturl_is_digit(char c) {
    return c >= '0' && c <= '9';
}

int neturl_parse(neturl_parsed_t *url, const char *input) {
    int i = 0;
    int j;
    url->valid = 0;
    url->scheme_len = 0;
    url->host_len = 0;
    url->path_len = 0;
    url->port = 0;

    while (neturl_is_alpha(input[i])) {
        if (url->scheme_len < 15) {
            url->scheme[url->scheme_len++] = input[i];
        }
        i++;
    }
    url->scheme[url->scheme_len] = 0;

    if (input[i] != ':' || input[i+1] != '/' || input[i+2] != '/') return -1;
    i += 3;

    while (input[i] != 0 && input[i] != ':' && input[i] != '/' && url->host_len < 127) {
        url->host[url->host_len++] = input[i];
        i++;
    }
    url->host[url->host_len] = 0;

    if (input[i] == ':') {
        i++;
        while (neturl_is_digit(input[i])) {
            url->port = url->port * 10 + (input[i] - '0');
            i++;
        }
    }

    if (input[i] == '/') {
        while (input[i] != 0 && url->path_len < 255) {
            url->path[url->path_len++] = input[i];
            i++;
        }
    } else {
        url->path[0] = '/';
        url->path_len = 1;
    }
    url->path[url->path_len] = 0;

    url->valid = 1;
    return 0;
}

int neturl_has_port(const neturl_parsed_t *url) {
    return url->port != 0;
}

uint16_t neturl_default_port(const neturl_parsed_t *url) {
    if (url->scheme[0] == 'h' && url->scheme[1] == 't' &&
        url->scheme[2] == 't' && url->scheme[3] == 'p') {
        if (url->scheme[4] == 's') return 443;
        return 80;
    }
    return 0;
}

int main(void) {
    neturl_parsed_t url;
    int rc = neturl_parse(&url, "http://example.com:8080/path");
    if (rc != 0) return 1;
    if (!url.valid) return 2;
    if (url.port != 8080) return 3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C707: URL parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C707: should produce non-empty output");
    assert!(code.contains("fn neturl_parse"), "C707: should contain neturl_parse");
}

/// C708: Ring buffer for network packets
#[test]
fn c708_ring_buffer_network_packets() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

#define NETRING_CAPACITY 32
#define NETRING_PKT_SIZE 256

typedef struct {
    uint8_t data[256];
    int length;
    uint32_t timestamp;
} netring_packet_t;

typedef struct {
    netring_packet_t packets[32];
    int head;
    int tail;
    int count;
    uint32_t total_enqueued;
    uint32_t total_dropped;
} netring_buffer_t;

void netring_init(netring_buffer_t *rb) {
    rb->head = 0;
    rb->tail = 0;
    rb->count = 0;
    rb->total_enqueued = 0;
    rb->total_dropped = 0;
}

int netring_is_full(const netring_buffer_t *rb) {
    return rb->count >= NETRING_CAPACITY;
}

int netring_is_empty(const netring_buffer_t *rb) {
    return rb->count == 0;
}

int netring_enqueue(netring_buffer_t *rb, const uint8_t *data, int len,
                    uint32_t timestamp) {
    if (netring_is_full(rb)) {
        rb->total_dropped++;
        return -1;
    }
    if (len > NETRING_PKT_SIZE) {
        rb->total_dropped++;
        return -2;
    }
    netring_packet_t *pkt = &rb->packets[rb->tail];
    int i;
    for (i = 0; i < len; i++) {
        pkt->data[i] = data[i];
    }
    pkt->length = len;
    pkt->timestamp = timestamp;
    rb->tail = (rb->tail + 1) % NETRING_CAPACITY;
    rb->count++;
    rb->total_enqueued++;
    return 0;
}

int netring_dequeue(netring_buffer_t *rb, uint8_t *buf, int buflen,
                    uint32_t *timestamp) {
    if (netring_is_empty(rb)) return -1;
    netring_packet_t *pkt = &rb->packets[rb->head];
    int copy_len = pkt->length;
    if (copy_len > buflen) copy_len = buflen;
    int i;
    for (i = 0; i < copy_len; i++) {
        buf[i] = pkt->data[i];
    }
    if (timestamp) *timestamp = pkt->timestamp;
    rb->head = (rb->head + 1) % NETRING_CAPACITY;
    rb->count--;
    return copy_len;
}

int netring_count(const netring_buffer_t *rb) {
    return rb->count;
}

float netring_drop_rate(const netring_buffer_t *rb) {
    uint32_t total = rb->total_enqueued + rb->total_dropped;
    if (total == 0) return 0.0f;
    return (float)rb->total_dropped / (float)total;
}

int main(void) {
    netring_buffer_t rb;
    netring_init(&rb);
    uint8_t data[4];
    data[0] = 0xDE; data[1] = 0xAD; data[2] = 0xBE; data[3] = 0xEF;
    netring_enqueue(&rb, data, 4, 1000);
    if (netring_count(&rb) != 1) return 1;
    uint8_t out[64];
    uint32_t ts;
    int n = netring_dequeue(&rb, out, 64, &ts);
    if (n != 4) return 2;
    if (ts != 1000) return 3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C708: Ring buffer for network packets should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C708: should produce non-empty output");
    assert!(code.contains("fn netring_init"), "C708: should contain netring_init");
    assert!(code.contains("fn netring_enqueue"), "C708: should contain netring_enqueue");
    assert!(code.contains("fn netring_dequeue"), "C708: should contain netring_dequeue");
}

/// C709: Connection pool manager
#[test]
fn c709_connection_pool_manager() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;

#define NETPOOL_MAX_CONNS 16
#define NETPOOL_STATE_FREE   0
#define NETPOOL_STATE_ACTIVE 1
#define NETPOOL_STATE_IDLE   2
#define NETPOOL_STATE_CLOSED 3

typedef struct {
    int fd;
    int state;
    uint32_t remote_addr;
    uint16_t remote_port;
    uint32_t created_at;
    uint32_t last_used;
    int uses;
} netpool_conn_t;

typedef struct {
    netpool_conn_t conns[16];
    int capacity;
    int active_count;
    int idle_count;
    uint32_t total_acquired;
    uint32_t total_released;
} netpool_t;

void netpool_init(netpool_t *pool) {
    pool->capacity = NETPOOL_MAX_CONNS;
    pool->active_count = 0;
    pool->idle_count = 0;
    pool->total_acquired = 0;
    pool->total_released = 0;
    int i;
    for (i = 0; i < NETPOOL_MAX_CONNS; i++) {
        pool->conns[i].fd = -1;
        pool->conns[i].state = NETPOOL_STATE_FREE;
        pool->conns[i].uses = 0;
    }
}

int netpool_acquire(netpool_t *pool, uint32_t addr, uint16_t port, uint32_t now) {
    int i;
    for (i = 0; i < pool->capacity; i++) {
        if (pool->conns[i].state == NETPOOL_STATE_IDLE &&
            pool->conns[i].remote_addr == addr &&
            pool->conns[i].remote_port == port) {
            pool->conns[i].state = NETPOOL_STATE_ACTIVE;
            pool->conns[i].last_used = now;
            pool->conns[i].uses++;
            pool->active_count++;
            pool->idle_count--;
            pool->total_acquired++;
            return i;
        }
    }
    for (i = 0; i < pool->capacity; i++) {
        if (pool->conns[i].state == NETPOOL_STATE_FREE) {
            pool->conns[i].fd = 100 + i;
            pool->conns[i].state = NETPOOL_STATE_ACTIVE;
            pool->conns[i].remote_addr = addr;
            pool->conns[i].remote_port = port;
            pool->conns[i].created_at = now;
            pool->conns[i].last_used = now;
            pool->conns[i].uses = 1;
            pool->active_count++;
            pool->total_acquired++;
            return i;
        }
    }
    return -1;
}

int netpool_release(netpool_t *pool, int idx) {
    if (idx < 0 || idx >= pool->capacity) return -1;
    if (pool->conns[idx].state != NETPOOL_STATE_ACTIVE) return -2;
    pool->conns[idx].state = NETPOOL_STATE_IDLE;
    pool->active_count--;
    pool->idle_count++;
    pool->total_released++;
    return 0;
}

int netpool_evict_idle(netpool_t *pool, uint32_t max_idle_time, uint32_t now) {
    int evicted = 0;
    int i;
    for (i = 0; i < pool->capacity; i++) {
        if (pool->conns[i].state == NETPOOL_STATE_IDLE &&
            (now - pool->conns[i].last_used) > max_idle_time) {
            pool->conns[i].state = NETPOOL_STATE_FREE;
            pool->conns[i].fd = -1;
            pool->idle_count--;
            evicted++;
        }
    }
    return evicted;
}

int netpool_available(const netpool_t *pool) {
    return pool->idle_count + (pool->capacity - pool->active_count - pool->idle_count);
}

int main(void) {
    netpool_t pool;
    netpool_init(&pool);
    int c1 = netpool_acquire(&pool, 0x0A000001, 80, 1000);
    if (c1 < 0) return 1;
    if (pool.active_count != 1) return 2;
    netpool_release(&pool, c1);
    if (pool.idle_count != 1) return 3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C709: Connection pool manager should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C709: should produce non-empty output");
    assert!(code.contains("fn netpool_init"), "C709: should contain netpool_init");
    assert!(code.contains("fn netpool_acquire"), "C709: should contain netpool_acquire");
    assert!(code.contains("fn netpool_release"), "C709: should contain netpool_release");
}

/// C710: Rate limiter (token bucket algorithm)
#[test]
fn c710_rate_limiter_token_bucket() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

typedef struct {
    uint32_t tokens;
    uint32_t max_tokens;
    uint32_t refill_rate;
    uint32_t last_refill_time;
    uint32_t total_allowed;
    uint32_t total_denied;
} netrl_bucket_t;

void netrl_init(netrl_bucket_t *rl, uint32_t max_tokens, uint32_t refill_rate) {
    rl->tokens = max_tokens;
    rl->max_tokens = max_tokens;
    rl->refill_rate = refill_rate;
    rl->last_refill_time = 0;
    rl->total_allowed = 0;
    rl->total_denied = 0;
}

void netrl_refill(netrl_bucket_t *rl, uint32_t now) {
    uint32_t elapsed = now - rl->last_refill_time;
    uint32_t new_tokens = elapsed * rl->refill_rate;
    rl->tokens += new_tokens;
    if (rl->tokens > rl->max_tokens) {
        rl->tokens = rl->max_tokens;
    }
    rl->last_refill_time = now;
}

int netrl_try_consume(netrl_bucket_t *rl, uint32_t cost, uint32_t now) {
    netrl_refill(rl, now);
    if (rl->tokens >= cost) {
        rl->tokens -= cost;
        rl->total_allowed++;
        return 1;
    }
    rl->total_denied++;
    return 0;
}

uint32_t netrl_available(const netrl_bucket_t *rl) {
    return rl->tokens;
}

float netrl_denial_rate(const netrl_bucket_t *rl) {
    uint32_t total = rl->total_allowed + rl->total_denied;
    if (total == 0) return 0.0f;
    return (float)rl->total_denied / (float)total;
}

int main(void) {
    netrl_bucket_t rl;
    netrl_init(&rl, 100, 10);
    if (!netrl_try_consume(&rl, 50, 0)) return 1;
    if (netrl_available(&rl) != 50) return 2;
    if (!netrl_try_consume(&rl, 50, 0)) return 3;
    if (netrl_try_consume(&rl, 1, 0)) return 4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C710: Rate limiter token bucket should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C710: should produce non-empty output");
    assert!(code.contains("fn netrl_init"), "C710: should contain netrl_init");
    assert!(code.contains("fn netrl_try_consume"), "C710: should contain netrl_try_consume");
}

// ============================================================================
// C711-C715: Network Utilities
// ============================================================================

/// C711: IP address conversion (string to int and back)
#[test]
fn c711_ip_address_conversion() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

uint32_t netip_from_octets(uint8_t a, uint8_t b, uint8_t c, uint8_t d) {
    return ((uint32_t)a << 24) | ((uint32_t)b << 16) |
           ((uint32_t)c << 8) | (uint32_t)d;
}

void netip_to_octets(uint32_t ip, uint8_t *a, uint8_t *b, uint8_t *c, uint8_t *d) {
    *a = (uint8_t)((ip >> 24) & 0xFF);
    *b = (uint8_t)((ip >> 16) & 0xFF);
    *c = (uint8_t)((ip >> 8) & 0xFF);
    *d = (uint8_t)(ip & 0xFF);
}

int netip_to_string(uint32_t ip, char *buf, int buflen) {
    uint8_t a, b, c, d;
    netip_to_octets(ip, &a, &b, &c, &d);
    int pos = 0;
    uint8_t octets[4];
    octets[0] = a; octets[1] = b; octets[2] = c; octets[3] = d;
    int i;
    for (i = 0; i < 4; i++) {
        if (i > 0 && pos < buflen) buf[pos++] = '.';
        uint8_t val = octets[i];
        if (val >= 100 && pos < buflen) {
            buf[pos++] = '0' + val / 100;
            val = val % 100;
            buf[pos++] = '0' + val / 10;
            buf[pos++] = '0' + val % 10;
        } else if (val >= 10 && pos < buflen) {
            buf[pos++] = '0' + val / 10;
            buf[pos++] = '0' + val % 10;
        } else if (pos < buflen) {
            buf[pos++] = '0' + val;
        }
    }
    if (pos < buflen) buf[pos] = 0;
    return pos;
}

int netip_is_private(uint32_t ip) {
    uint8_t first = (uint8_t)((ip >> 24) & 0xFF);
    uint8_t second = (uint8_t)((ip >> 16) & 0xFF);
    if (first == 10) return 1;
    if (first == 172 && second >= 16 && second <= 31) return 1;
    if (first == 192 && second == 168) return 1;
    return 0;
}

int netip_is_multicast(uint32_t ip) {
    uint8_t first = (uint8_t)((ip >> 24) & 0xFF);
    return first >= 224 && first <= 239;
}

int main(void) {
    uint32_t ip = netip_from_octets(192, 168, 1, 100);
    uint8_t a, b, c, d;
    netip_to_octets(ip, &a, &b, &c, &d);
    if (a != 192 || b != 168 || c != 1 || d != 100) return 1;
    if (!netip_is_private(ip)) return 2;
    char buf[16];
    netip_to_string(ip, buf, 16);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C711: IP address conversion should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C711: should produce non-empty output");
    assert!(code.contains("fn netip_from_octets"), "C711: should contain netip_from_octets");
    assert!(code.contains("fn netip_to_string"), "C711: should contain netip_to_string");
}

/// C712: Checksum calculation (TCP/IP style)
#[test]
fn c712_checksum_calculation() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

uint16_t netchk_ones_complement(const uint8_t *data, int len) {
    uint32_t sum = 0;
    int i;
    for (i = 0; i + 1 < len; i += 2) {
        uint16_t word = ((uint16_t)data[i] << 8) | (uint16_t)data[i + 1];
        sum += word;
    }
    if (i < len) {
        sum += (uint16_t)data[i] << 8;
    }
    while (sum >> 16) {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    return (uint16_t)(~sum & 0xFFFF);
}

uint16_t netchk_tcp_pseudo(uint32_t src, uint32_t dst, uint8_t protocol,
                           uint16_t tcp_len, const uint8_t *tcp_data, int data_len) {
    uint32_t sum = 0;
    sum += (src >> 16) & 0xFFFF;
    sum += src & 0xFFFF;
    sum += (dst >> 16) & 0xFFFF;
    sum += dst & 0xFFFF;
    sum += (uint16_t)protocol;
    sum += tcp_len;
    int i;
    for (i = 0; i + 1 < data_len; i += 2) {
        sum += ((uint16_t)tcp_data[i] << 8) | (uint16_t)tcp_data[i + 1];
    }
    if (i < data_len) {
        sum += (uint16_t)tcp_data[i] << 8;
    }
    while (sum >> 16) {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    return (uint16_t)(~sum & 0xFFFF);
}

int netchk_verify(const uint8_t *data, int len) {
    uint32_t sum = 0;
    int i;
    for (i = 0; i + 1 < len; i += 2) {
        sum += ((uint16_t)data[i] << 8) | (uint16_t)data[i + 1];
    }
    if (i < len) {
        sum += (uint16_t)data[i] << 8;
    }
    while (sum >> 16) {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    return (sum & 0xFFFF) == 0xFFFF;
}

int main(void) {
    uint8_t data[8];
    data[0] = 0x00; data[1] = 0x01;
    data[2] = 0x00; data[3] = 0x02;
    data[4] = 0x00; data[5] = 0x03;
    data[6] = 0x00; data[7] = 0x04;
    uint16_t cksum = netchk_ones_complement(data, 8);
    if (cksum == 0) return 1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C712: Checksum calculation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C712: should produce non-empty output");
    assert!(code.contains("fn netchk_ones_complement"), "C712: should contain netchk_ones_complement");
    assert!(code.contains("fn netchk_tcp_pseudo"), "C712: should contain netchk_tcp_pseudo");
}

/// C713: Packet fragmentation and reassembly
#[test]
fn c713_packet_fragmentation_reassembly() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

#define NETFRAG_MTU 128
#define NETFRAG_MAX_FRAGS 16

typedef struct {
    uint16_t id;
    uint16_t offset;
    uint16_t length;
    int more_fragments;
    uint8_t data[128];
} netfrag_fragment_t;

typedef struct {
    uint16_t id;
    netfrag_fragment_t frags[16];
    int frag_count;
    int total_length;
    int complete;
} netfrag_reassembly_t;

void netfrag_reassembly_init(netfrag_reassembly_t *ra, uint16_t id) {
    ra->id = id;
    ra->frag_count = 0;
    ra->total_length = 0;
    ra->complete = 0;
}

int netfrag_fragment_data(const uint8_t *data, int data_len, uint16_t id,
                          netfrag_fragment_t *frags, int max_frags) {
    int count = 0;
    int offset = 0;
    while (offset < data_len && count < max_frags) {
        int chunk = data_len - offset;
        if (chunk > NETFRAG_MTU) chunk = NETFRAG_MTU;
        frags[count].id = id;
        frags[count].offset = (uint16_t)offset;
        frags[count].length = (uint16_t)chunk;
        frags[count].more_fragments = (offset + chunk < data_len) ? 1 : 0;
        int i;
        for (i = 0; i < chunk; i++) {
            frags[count].data[i] = data[offset + i];
        }
        count++;
        offset += chunk;
    }
    return count;
}

int netfrag_add_fragment(netfrag_reassembly_t *ra, const netfrag_fragment_t *frag) {
    if (frag->id != ra->id) return -1;
    if (ra->frag_count >= NETFRAG_MAX_FRAGS) return -2;
    int i;
    for (i = 0; i < ra->frag_count; i++) {
        if (ra->frags[i].offset == frag->offset) return -3;
    }
    ra->frags[ra->frag_count] = *frag;
    ra->frag_count++;
    if (!frag->more_fragments) {
        ra->total_length = frag->offset + frag->length;
    }
    return 0;
}

int netfrag_is_complete(const netfrag_reassembly_t *ra) {
    if (ra->total_length == 0) return 0;
    int covered = 0;
    int i;
    for (i = 0; i < ra->frag_count; i++) {
        covered += ra->frags[i].length;
    }
    return covered >= ra->total_length;
}

int main(void) {
    uint8_t data[300];
    int i;
    for (i = 0; i < 300; i++) data[i] = (uint8_t)(i & 0xFF);
    netfrag_fragment_t frags[16];
    int n = netfrag_fragment_data(data, 300, 42, frags, 16);
    if (n < 2) return 1;
    netfrag_reassembly_t ra;
    netfrag_reassembly_init(&ra, 42);
    for (i = 0; i < n; i++) {
        netfrag_add_fragment(&ra, &frags[i]);
    }
    if (!netfrag_is_complete(&ra)) return 2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C713: Packet fragmentation/reassembly should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C713: should produce non-empty output");
    assert!(code.contains("fn netfrag_reassembly_init"), "C713: should contain netfrag_reassembly_init");
    assert!(code.contains("fn netfrag_fragment_data"), "C713: should contain netfrag_fragment_data");
}

/// C714: ARP cache table
#[test]
fn c714_arp_cache_table() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define NETARP_MAX_ENTRIES 64
#define NETARP_STATE_FREE    0
#define NETARP_STATE_PENDING 1
#define NETARP_STATE_VALID   2
#define NETARP_STATE_STALE   3

typedef struct {
    uint32_t ip_addr;
    uint8_t mac_addr[6];
    int state;
    uint32_t timestamp;
    uint32_t ttl;
} netarp_entry_t;

typedef struct {
    netarp_entry_t entries[64];
    int count;
    uint32_t hits;
    uint32_t misses;
} netarp_cache_t;

void netarp_init(netarp_cache_t *cache) {
    cache->count = 0;
    cache->hits = 0;
    cache->misses = 0;
    int i;
    for (i = 0; i < NETARP_MAX_ENTRIES; i++) {
        cache->entries[i].state = NETARP_STATE_FREE;
    }
}

int netarp_lookup(netarp_cache_t *cache, uint32_t ip, uint8_t *mac_out) {
    int i;
    for (i = 0; i < cache->count; i++) {
        if (cache->entries[i].state == NETARP_STATE_VALID &&
            cache->entries[i].ip_addr == ip) {
            int j;
            for (j = 0; j < 6; j++) {
                mac_out[j] = cache->entries[i].mac_addr[j];
            }
            cache->hits++;
            return 0;
        }
    }
    cache->misses++;
    return -1;
}

int netarp_insert(netarp_cache_t *cache, uint32_t ip, const uint8_t *mac,
                  uint32_t now, uint32_t ttl) {
    int i;
    for (i = 0; i < cache->count; i++) {
        if (cache->entries[i].ip_addr == ip) {
            int j;
            for (j = 0; j < 6; j++) {
                cache->entries[i].mac_addr[j] = mac[j];
            }
            cache->entries[i].state = NETARP_STATE_VALID;
            cache->entries[i].timestamp = now;
            cache->entries[i].ttl = ttl;
            return 0;
        }
    }
    if (cache->count >= NETARP_MAX_ENTRIES) return -1;
    netarp_entry_t *e = &cache->entries[cache->count];
    e->ip_addr = ip;
    int j;
    for (j = 0; j < 6; j++) {
        e->mac_addr[j] = mac[j];
    }
    e->state = NETARP_STATE_VALID;
    e->timestamp = now;
    e->ttl = ttl;
    cache->count++;
    return 0;
}

int netarp_expire(netarp_cache_t *cache, uint32_t now) {
    int expired = 0;
    int i;
    for (i = 0; i < cache->count; i++) {
        if (cache->entries[i].state == NETARP_STATE_VALID &&
            (now - cache->entries[i].timestamp) > cache->entries[i].ttl) {
            cache->entries[i].state = NETARP_STATE_STALE;
            expired++;
        }
    }
    return expired;
}

float netarp_hit_rate(const netarp_cache_t *cache) {
    uint32_t total = cache->hits + cache->misses;
    if (total == 0) return 0.0f;
    return (float)cache->hits / (float)total;
}

int main(void) {
    netarp_cache_t cache;
    netarp_init(&cache);
    uint8_t mac[6];
    mac[0] = 0xAA; mac[1] = 0xBB; mac[2] = 0xCC;
    mac[3] = 0xDD; mac[4] = 0xEE; mac[5] = 0xFF;
    netarp_insert(&cache, 0xC0A80101, mac, 1000, 300);
    uint8_t out[6];
    if (netarp_lookup(&cache, 0xC0A80101, out) != 0) return 1;
    if (out[0] != 0xAA) return 2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C714: ARP cache table should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C714: should produce non-empty output");
    assert!(code.contains("fn netarp_init"), "C714: should contain netarp_init");
    assert!(code.contains("fn netarp_lookup"), "C714: should contain netarp_lookup");
    assert!(code.contains("fn netarp_insert"), "C714: should contain netarp_insert");
}

/// C715: Routing table longest prefix match
#[test]
fn c715_routing_table_longest_prefix_match() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define NETRT_MAX_ROUTES 32

typedef struct {
    uint32_t network;
    uint32_t mask;
    uint32_t gateway;
    int interface_id;
    int metric;
    int prefix_len;
} netrt_route_t;

typedef struct {
    netrt_route_t routes[32];
    int count;
} netrt_table_t;

void netrt_init(netrt_table_t *table) {
    table->count = 0;
}

uint32_t netrt_prefix_to_mask(int prefix_len) {
    if (prefix_len <= 0) return 0;
    if (prefix_len >= 32) return 0xFFFFFFFF;
    return 0xFFFFFFFF << (32 - prefix_len);
}

int netrt_add_route(netrt_table_t *table, uint32_t network, int prefix_len,
                    uint32_t gateway, int iface, int metric) {
    if (table->count >= NETRT_MAX_ROUTES) return -1;
    netrt_route_t *r = &table->routes[table->count];
    r->mask = netrt_prefix_to_mask(prefix_len);
    r->network = network & r->mask;
    r->gateway = gateway;
    r->interface_id = iface;
    r->metric = metric;
    r->prefix_len = prefix_len;
    table->count++;
    return 0;
}

int netrt_lookup(const netrt_table_t *table, uint32_t dest, netrt_route_t *result) {
    int best = -1;
    int best_prefix = -1;
    int best_metric = 0x7FFFFFFF;
    int i;
    for (i = 0; i < table->count; i++) {
        if ((dest & table->routes[i].mask) == table->routes[i].network) {
            if (table->routes[i].prefix_len > best_prefix ||
                (table->routes[i].prefix_len == best_prefix &&
                 table->routes[i].metric < best_metric)) {
                best = i;
                best_prefix = table->routes[i].prefix_len;
                best_metric = table->routes[i].metric;
            }
        }
    }
    if (best >= 0) {
        *result = table->routes[best];
        return 0;
    }
    return -1;
}

int netrt_delete_route(netrt_table_t *table, uint32_t network, int prefix_len) {
    uint32_t mask = netrt_prefix_to_mask(prefix_len);
    uint32_t net = network & mask;
    int i;
    for (i = 0; i < table->count; i++) {
        if (table->routes[i].network == net && table->routes[i].prefix_len == prefix_len) {
            int j;
            for (j = i; j < table->count - 1; j++) {
                table->routes[j] = table->routes[j + 1];
            }
            table->count--;
            return 0;
        }
    }
    return -1;
}

int main(void) {
    netrt_table_t table;
    netrt_init(&table);
    netrt_add_route(&table, 0x0A000000, 8, 0x0A000001, 0, 10);
    netrt_add_route(&table, 0x0A010000, 16, 0x0A010001, 1, 5);
    netrt_add_route(&table, 0x00000000, 0, 0x0A000001, 0, 100);
    netrt_route_t result;
    if (netrt_lookup(&table, 0x0A010064, &result) != 0) return 1;
    if (result.prefix_len != 16) return 2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C715: Routing table longest prefix match should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C715: should produce non-empty output");
    assert!(code.contains("fn netrt_init"), "C715: should contain netrt_init");
    assert!(code.contains("fn netrt_lookup"), "C715: should contain netrt_lookup");
}

// ============================================================================
// C716-C720: Low-Level Network
// ============================================================================

/// C716: Network byte order conversion (htonl/ntohl simulation)
#[test]
fn c716_network_byte_order_conversion() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

uint32_t netbo_htonl(uint32_t host) {
    return ((host & 0xFF000000) >> 24) |
           ((host & 0x00FF0000) >> 8) |
           ((host & 0x0000FF00) << 8) |
           ((host & 0x000000FF) << 24);
}

uint32_t netbo_ntohl(uint32_t net) {
    return netbo_htonl(net);
}

uint16_t netbo_htons(uint16_t host) {
    return ((host & 0xFF00) >> 8) | ((host & 0x00FF) << 8);
}

uint16_t netbo_ntohs(uint16_t net) {
    return netbo_htons(net);
}

void netbo_put_u32(uint8_t *buf, uint32_t val) {
    buf[0] = (uint8_t)((val >> 24) & 0xFF);
    buf[1] = (uint8_t)((val >> 16) & 0xFF);
    buf[2] = (uint8_t)((val >> 8) & 0xFF);
    buf[3] = (uint8_t)(val & 0xFF);
}

uint32_t netbo_get_u32(const uint8_t *buf) {
    return ((uint32_t)buf[0] << 24) | ((uint32_t)buf[1] << 16) |
           ((uint32_t)buf[2] << 8) | (uint32_t)buf[3];
}

void netbo_put_u16(uint8_t *buf, uint16_t val) {
    buf[0] = (uint8_t)((val >> 8) & 0xFF);
    buf[1] = (uint8_t)(val & 0xFF);
}

uint16_t netbo_get_u16(const uint8_t *buf) {
    return ((uint16_t)buf[0] << 8) | (uint16_t)buf[1];
}

int netbo_is_big_endian(void) {
    uint32_t test = 1;
    uint8_t *byte = (uint8_t *)&test;
    return byte[0] == 0;
}

int main(void) {
    uint32_t val = 0x01020304;
    uint32_t net = netbo_htonl(val);
    uint32_t back = netbo_ntohl(net);
    if (back != val) return 1;
    uint16_t port = 8080;
    uint16_t net_port = netbo_htons(port);
    if (netbo_ntohs(net_port) != port) return 2;
    uint8_t buf[4];
    netbo_put_u32(buf, 0xDEADBEEF);
    if (netbo_get_u32(buf) != 0xDEADBEEF) return 3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C716: Network byte order conversion should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C716: should produce non-empty output");
    assert!(code.contains("fn netbo_htonl"), "C716: should contain netbo_htonl");
    assert!(code.contains("fn netbo_htons"), "C716: should contain netbo_htons");
}

/// C717: CIDR subnet calculator
#[test]
fn c717_cidr_subnet_calculator() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint32_t network;
    uint32_t mask;
    uint32_t broadcast;
    uint32_t first_host;
    uint32_t last_host;
    uint32_t host_count;
    int prefix_len;
} netcidr_subnet_t;

uint32_t netcidr_mask_from_prefix(int prefix) {
    if (prefix <= 0) return 0;
    if (prefix >= 32) return 0xFFFFFFFF;
    return 0xFFFFFFFF << (32 - prefix);
}

int netcidr_prefix_from_mask(uint32_t mask) {
    int count = 0;
    uint32_t m = mask;
    while (m & 0x80000000) {
        count++;
        m <<= 1;
    }
    return count;
}

void netcidr_calculate(netcidr_subnet_t *subnet, uint32_t ip, int prefix) {
    subnet->prefix_len = prefix;
    subnet->mask = netcidr_mask_from_prefix(prefix);
    subnet->network = ip & subnet->mask;
    subnet->broadcast = subnet->network | ~subnet->mask;
    if (prefix >= 31) {
        subnet->first_host = subnet->network;
        subnet->last_host = subnet->broadcast;
        subnet->host_count = (prefix == 32) ? 1 : 2;
    } else {
        subnet->first_host = subnet->network + 1;
        subnet->last_host = subnet->broadcast - 1;
        subnet->host_count = (1u << (32 - prefix)) - 2;
    }
}

int netcidr_contains(const netcidr_subnet_t *subnet, uint32_t ip) {
    return (ip & subnet->mask) == subnet->network;
}

int netcidr_overlaps(const netcidr_subnet_t *a, const netcidr_subnet_t *b) {
    return netcidr_contains(a, b->network) || netcidr_contains(a, b->broadcast) ||
           netcidr_contains(b, a->network) || netcidr_contains(b, a->broadcast);
}

int netcidr_is_host_addr(const netcidr_subnet_t *subnet, uint32_t ip) {
    if (!netcidr_contains(subnet, ip)) return 0;
    if (ip == subnet->network) return 0;
    if (ip == subnet->broadcast) return 0;
    return 1;
}

int main(void) {
    netcidr_subnet_t subnet;
    netcidr_calculate(&subnet, 0xC0A80164, 24);
    if (subnet.network != 0xC0A80100) return 1;
    if (subnet.broadcast != 0xC0A801FF) return 2;
    if (subnet.host_count != 254) return 3;
    if (!netcidr_contains(&subnet, 0xC0A80132)) return 4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C717: CIDR subnet calculator should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C717: should produce non-empty output");
    assert!(code.contains("fn netcidr_calculate"), "C717: should contain netcidr_calculate");
    assert!(code.contains("fn netcidr_contains"), "C717: should contain netcidr_contains");
}

/// C718: Port scanner state tracker
#[test]
fn c718_port_scanner_state_tracker() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;

#define NETSCAN_STATE_UNKNOWN  0
#define NETSCAN_STATE_OPEN     1
#define NETSCAN_STATE_CLOSED   2
#define NETSCAN_STATE_FILTERED 3
#define NETSCAN_MAX_PORTS 256

typedef struct {
    uint16_t port;
    int state;
    int response_time_ms;
    int retries;
} netscan_port_result_t;

typedef struct {
    uint32_t target_ip;
    netscan_port_result_t ports[256];
    int port_count;
    int open_count;
    int closed_count;
    int filtered_count;
    int scan_complete;
} netscan_tracker_t;

void netscan_init(netscan_tracker_t *tracker, uint32_t target) {
    tracker->target_ip = target;
    tracker->port_count = 0;
    tracker->open_count = 0;
    tracker->closed_count = 0;
    tracker->filtered_count = 0;
    tracker->scan_complete = 0;
}

int netscan_add_port(netscan_tracker_t *tracker, uint16_t port) {
    if (tracker->port_count >= NETSCAN_MAX_PORTS) return -1;
    netscan_port_result_t *pr = &tracker->ports[tracker->port_count];
    pr->port = port;
    pr->state = NETSCAN_STATE_UNKNOWN;
    pr->response_time_ms = -1;
    pr->retries = 0;
    tracker->port_count++;
    return 0;
}

int netscan_update_port(netscan_tracker_t *tracker, uint16_t port, int state,
                        int response_time) {
    int i;
    for (i = 0; i < tracker->port_count; i++) {
        if (tracker->ports[i].port == port) {
            int old_state = tracker->ports[i].state;
            tracker->ports[i].state = state;
            tracker->ports[i].response_time_ms = response_time;
            if (old_state == NETSCAN_STATE_UNKNOWN) {
                if (state == NETSCAN_STATE_OPEN) tracker->open_count++;
                else if (state == NETSCAN_STATE_CLOSED) tracker->closed_count++;
                else if (state == NETSCAN_STATE_FILTERED) tracker->filtered_count++;
            }
            return 0;
        }
    }
    return -1;
}

int netscan_progress_pct(const netscan_tracker_t *tracker) {
    if (tracker->port_count == 0) return 100;
    int scanned = tracker->open_count + tracker->closed_count + tracker->filtered_count;
    return (scanned * 100) / tracker->port_count;
}

int netscan_get_open_ports(const netscan_tracker_t *tracker, uint16_t *out, int max_out) {
    int count = 0;
    int i;
    for (i = 0; i < tracker->port_count && count < max_out; i++) {
        if (tracker->ports[i].state == NETSCAN_STATE_OPEN) {
            out[count++] = tracker->ports[i].port;
        }
    }
    return count;
}

int main(void) {
    netscan_tracker_t tracker;
    netscan_init(&tracker, 0x0A000001);
    netscan_add_port(&tracker, 22);
    netscan_add_port(&tracker, 80);
    netscan_add_port(&tracker, 443);
    netscan_update_port(&tracker, 22, NETSCAN_STATE_OPEN, 5);
    netscan_update_port(&tracker, 80, NETSCAN_STATE_OPEN, 3);
    netscan_update_port(&tracker, 443, NETSCAN_STATE_CLOSED, 10);
    if (tracker.open_count != 2) return 1;
    if (netscan_progress_pct(&tracker) != 100) return 2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C718: Port scanner state tracker should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C718: should produce non-empty output");
    assert!(code.contains("fn netscan_init"), "C718: should contain netscan_init");
    assert!(code.contains("fn netscan_update_port"), "C718: should contain netscan_update_port");
}

/// C719: Simple DNS message builder
#[test]
fn c719_dns_message_builder() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

#define NETDNS_TYPE_A     1
#define NETDNS_TYPE_AAAA  28
#define NETDNS_TYPE_CNAME 5
#define NETDNS_CLASS_IN   1
#define NETDNS_MAX_MSG    512

typedef struct {
    uint8_t buf[512];
    int pos;
    uint16_t id;
    uint16_t flags;
    uint16_t qdcount;
    uint16_t ancount;
} netdns_builder_t;

static void netdns_put16(uint8_t *buf, int pos, uint16_t val) {
    buf[pos] = (uint8_t)((val >> 8) & 0xFF);
    buf[pos + 1] = (uint8_t)(val & 0xFF);
}

void netdns_builder_init(netdns_builder_t *b, uint16_t id, uint16_t flags) {
    b->id = id;
    b->flags = flags;
    b->qdcount = 0;
    b->ancount = 0;
    b->pos = 12;
    netdns_put16(b->buf, 0, id);
    netdns_put16(b->buf, 2, flags);
    netdns_put16(b->buf, 4, 0);
    netdns_put16(b->buf, 6, 0);
    netdns_put16(b->buf, 8, 0);
    netdns_put16(b->buf, 10, 0);
}

int netdns_add_question(netdns_builder_t *b, const char *name,
                        uint16_t qtype, uint16_t qclass) {
    int i = 0;
    while (name[i] != 0) {
        int label_start = b->pos;
        b->pos++;
        int label_len = 0;
        while (name[i] != 0 && name[i] != '.') {
            if (b->pos >= NETDNS_MAX_MSG - 4) return -1;
            b->buf[b->pos++] = (uint8_t)name[i];
            label_len++;
            i++;
        }
        b->buf[label_start] = (uint8_t)label_len;
        if (name[i] == '.') i++;
    }
    if (b->pos >= NETDNS_MAX_MSG - 5) return -1;
    b->buf[b->pos++] = 0;
    netdns_put16(b->buf, b->pos, qtype);
    b->pos += 2;
    netdns_put16(b->buf, b->pos, qclass);
    b->pos += 2;
    b->qdcount++;
    netdns_put16(b->buf, 4, b->qdcount);
    return 0;
}

int netdns_get_length(const netdns_builder_t *b) {
    return b->pos;
}

uint16_t netdns_get_id(const netdns_builder_t *b) {
    return b->id;
}

int main(void) {
    netdns_builder_t builder;
    netdns_builder_init(&builder, 0x1234, 0x0100);
    int rc = netdns_add_question(&builder, "example.com", NETDNS_TYPE_A, NETDNS_CLASS_IN);
    if (rc != 0) return 1;
    if (netdns_get_length(&builder) < 12) return 2;
    if (netdns_get_id(&builder) != 0x1234) return 3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C719: DNS message builder should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C719: should produce non-empty output");
    assert!(code.contains("fn netdns_builder_init"), "C719: should contain netdns_builder_init");
    assert!(code.contains("fn netdns_add_question"), "C719: should contain netdns_add_question");
}

/// C720: SOCKS proxy protocol handler
#[test]
fn c720_socks_proxy_protocol_handler() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

#define NETSOCKS_VER5 5
#define NETSOCKS_CMD_CONNECT  1
#define NETSOCKS_CMD_BIND     2
#define NETSOCKS_ATYPE_IPV4   1
#define NETSOCKS_ATYPE_DOMAIN 3
#define NETSOCKS_ATYPE_IPV6   4
#define NETSOCKS_AUTH_NONE    0
#define NETSOCKS_AUTH_USERPASS 2

#define NETSOCKS_REP_SUCCESS  0
#define NETSOCKS_REP_FAILURE  1
#define NETSOCKS_REP_NOTALLOWED 2
#define NETSOCKS_REP_NETUNREACH 3

typedef struct {
    uint8_t version;
    uint8_t command;
    uint8_t address_type;
    uint32_t dest_addr;
    uint16_t dest_port;
    char dest_domain[256];
    int dest_domain_len;
    int authenticated;
} netsocks_request_t;

typedef struct {
    uint8_t buf[512];
    int pos;
    int len;
} netsocks_response_t;

void netsocks_request_init(netsocks_request_t *req) {
    req->version = 0;
    req->command = 0;
    req->address_type = 0;
    req->dest_addr = 0;
    req->dest_port = 0;
    req->dest_domain_len = 0;
    req->authenticated = 0;
}

int netsocks_parse_greeting(const uint8_t *data, int len, int *has_noauth) {
    if (len < 2) return -1;
    if (data[0] != NETSOCKS_VER5) return -2;
    int nmethods = data[1];
    if (len < 2 + nmethods) return -3;
    *has_noauth = 0;
    int i;
    for (i = 0; i < nmethods; i++) {
        if (data[2 + i] == NETSOCKS_AUTH_NONE) {
            *has_noauth = 1;
        }
    }
    return 2 + nmethods;
}

int netsocks_build_greeting_response(uint8_t *buf, uint8_t method) {
    buf[0] = NETSOCKS_VER5;
    buf[1] = method;
    return 2;
}

int netsocks_parse_connect(const uint8_t *data, int len, netsocks_request_t *req) {
    if (len < 4) return -1;
    if (data[0] != NETSOCKS_VER5) return -2;
    req->version = data[0];
    req->command = data[1];
    req->address_type = data[3];
    int pos = 4;
    if (req->address_type == NETSOCKS_ATYPE_IPV4) {
        if (len < pos + 6) return -3;
        req->dest_addr = ((uint32_t)data[pos] << 24) | ((uint32_t)data[pos+1] << 16) |
                         ((uint32_t)data[pos+2] << 8) | (uint32_t)data[pos+3];
        pos += 4;
    } else if (req->address_type == NETSOCKS_ATYPE_DOMAIN) {
        if (len < pos + 1) return -3;
        int dlen = data[pos++];
        if (len < pos + dlen + 2) return -3;
        int i;
        for (i = 0; i < dlen && i < 255; i++) {
            req->dest_domain[i] = (char)data[pos + i];
        }
        req->dest_domain[i] = 0;
        req->dest_domain_len = dlen;
        pos += dlen;
    } else {
        return -4;
    }
    req->dest_port = ((uint16_t)data[pos] << 8) | (uint16_t)data[pos + 1];
    pos += 2;
    return pos;
}

int netsocks_build_reply(uint8_t *buf, uint8_t reply, uint32_t bind_addr,
                         uint16_t bind_port) {
    buf[0] = NETSOCKS_VER5;
    buf[1] = reply;
    buf[2] = 0;
    buf[3] = NETSOCKS_ATYPE_IPV4;
    buf[4] = (uint8_t)((bind_addr >> 24) & 0xFF);
    buf[5] = (uint8_t)((bind_addr >> 16) & 0xFF);
    buf[6] = (uint8_t)((bind_addr >> 8) & 0xFF);
    buf[7] = (uint8_t)(bind_addr & 0xFF);
    buf[8] = (uint8_t)((bind_port >> 8) & 0xFF);
    buf[9] = (uint8_t)(bind_port & 0xFF);
    return 10;
}

int main(void) {
    uint8_t greeting[4];
    greeting[0] = 5; greeting[1] = 2; greeting[2] = 0; greeting[3] = 2;
    int has_noauth = 0;
    int n = netsocks_parse_greeting(greeting, 4, &has_noauth);
    if (n != 4) return 1;
    if (!has_noauth) return 2;
    uint8_t resp[2];
    netsocks_build_greeting_response(resp, NETSOCKS_AUTH_NONE);
    if (resp[0] != 5 || resp[1] != 0) return 3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C720: SOCKS proxy protocol handler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C720: should produce non-empty output");
    assert!(code.contains("fn netsocks_parse_greeting"), "C720: should contain netsocks_parse_greeting");
    assert!(code.contains("fn netsocks_parse_connect"), "C720: should contain netsocks_parse_connect");
}

// ============================================================================
// C721-C725: Modern Protocols
// ============================================================================

/// C721: WebSocket frame encoder/decoder
#[test]
fn c721_websocket_frame_codec() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

#define NETWS_OP_CONTINUATION 0x0
#define NETWS_OP_TEXT         0x1
#define NETWS_OP_BINARY       0x2
#define NETWS_OP_CLOSE        0x8
#define NETWS_OP_PING         0x9
#define NETWS_OP_PONG         0xA

typedef struct {
    int fin;
    uint8_t opcode;
    int masked;
    uint8_t mask_key[4];
    uint32_t payload_length;
    int header_length;
} netws_frame_header_t;

int netws_decode_header(const uint8_t *data, int len, netws_frame_header_t *hdr) {
    if (len < 2) return -1;
    hdr->fin = (data[0] >> 7) & 1;
    hdr->opcode = data[0] & 0x0F;
    hdr->masked = (data[1] >> 7) & 1;
    uint8_t len7 = data[1] & 0x7F;
    int pos = 2;

    if (len7 < 126) {
        hdr->payload_length = len7;
    } else if (len7 == 126) {
        if (len < 4) return -1;
        hdr->payload_length = ((uint32_t)data[2] << 8) | (uint32_t)data[3];
        pos = 4;
    } else {
        if (len < 10) return -1;
        hdr->payload_length = ((uint32_t)data[6] << 24) | ((uint32_t)data[7] << 16) |
                              ((uint32_t)data[8] << 8) | (uint32_t)data[9];
        pos = 10;
    }

    if (hdr->masked) {
        if (len < pos + 4) return -1;
        hdr->mask_key[0] = data[pos];
        hdr->mask_key[1] = data[pos + 1];
        hdr->mask_key[2] = data[pos + 2];
        hdr->mask_key[3] = data[pos + 3];
        pos += 4;
    }

    hdr->header_length = pos;
    return pos;
}

int netws_encode_header(uint8_t *buf, int buflen, int fin, uint8_t opcode,
                        int masked, const uint8_t *mask_key, uint32_t payload_len) {
    int pos = 0;
    if (buflen < 2) return -1;
    buf[pos++] = (uint8_t)((fin ? 0x80 : 0) | (opcode & 0x0F));
    uint8_t mask_bit = masked ? 0x80 : 0;
    if (payload_len < 126) {
        buf[pos++] = mask_bit | (uint8_t)payload_len;
    } else if (payload_len <= 0xFFFF) {
        if (buflen < 4) return -1;
        buf[pos++] = mask_bit | 126;
        buf[pos++] = (uint8_t)((payload_len >> 8) & 0xFF);
        buf[pos++] = (uint8_t)(payload_len & 0xFF);
    } else {
        if (buflen < 10) return -1;
        buf[pos++] = mask_bit | 127;
        buf[pos++] = 0; buf[pos++] = 0;
        buf[pos++] = 0; buf[pos++] = 0;
        buf[pos++] = (uint8_t)((payload_len >> 24) & 0xFF);
        buf[pos++] = (uint8_t)((payload_len >> 16) & 0xFF);
        buf[pos++] = (uint8_t)((payload_len >> 8) & 0xFF);
        buf[pos++] = (uint8_t)(payload_len & 0xFF);
    }
    if (masked && mask_key) {
        if (pos + 4 > buflen) return -1;
        buf[pos++] = mask_key[0];
        buf[pos++] = mask_key[1];
        buf[pos++] = mask_key[2];
        buf[pos++] = mask_key[3];
    }
    return pos;
}

void netws_apply_mask(uint8_t *data, int len, const uint8_t *mask_key) {
    int i;
    for (i = 0; i < len; i++) {
        data[i] ^= mask_key[i % 4];
    }
}

int netws_is_control(uint8_t opcode) {
    return opcode >= 0x8;
}

int main(void) {
    uint8_t frame[16];
    int hlen = netws_encode_header(frame, 16, 1, NETWS_OP_TEXT, 0, 0, 5);
    if (hlen != 2) return 1;
    netws_frame_header_t hdr;
    int decoded = netws_decode_header(frame, hlen, &hdr);
    if (decoded < 0) return 2;
    if (hdr.opcode != NETWS_OP_TEXT) return 3;
    if (hdr.payload_length != 5) return 4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C721: WebSocket frame codec should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C721: should produce non-empty output");
    assert!(code.contains("fn netws_decode_header"), "C721: should contain netws_decode_header");
    assert!(code.contains("fn netws_encode_header"), "C721: should contain netws_encode_header");
}

/// C722: MQTT packet serializer
#[test]
fn c722_mqtt_packet_serializer() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

#define NETMQTT_CONNECT     1
#define NETMQTT_CONNACK     2
#define NETMQTT_PUBLISH     3
#define NETMQTT_SUBSCRIBE   8
#define NETMQTT_SUBACK      9
#define NETMQTT_PINGREQ     12
#define NETMQTT_PINGRESP    13
#define NETMQTT_DISCONNECT  14

typedef struct {
    uint8_t buf[512];
    int pos;
    int capacity;
} netmqtt_writer_t;

void netmqtt_writer_init(netmqtt_writer_t *w) {
    w->pos = 0;
    w->capacity = 512;
}

static int netmqtt_write_byte(netmqtt_writer_t *w, uint8_t val) {
    if (w->pos >= w->capacity) return -1;
    w->buf[w->pos++] = val;
    return 0;
}

static int netmqtt_write_u16(netmqtt_writer_t *w, uint16_t val) {
    if (w->pos + 2 > w->capacity) return -1;
    w->buf[w->pos++] = (uint8_t)((val >> 8) & 0xFF);
    w->buf[w->pos++] = (uint8_t)(val & 0xFF);
    return 0;
}

static int netmqtt_write_string(netmqtt_writer_t *w, const char *str) {
    int len = 0;
    while (str[len] != 0) len++;
    if (netmqtt_write_u16(w, (uint16_t)len) < 0) return -1;
    int i;
    for (i = 0; i < len; i++) {
        if (netmqtt_write_byte(w, (uint8_t)str[i]) < 0) return -1;
    }
    return 0;
}

static int netmqtt_encode_remaining_length(netmqtt_writer_t *w, uint32_t length) {
    do {
        uint8_t encoded = (uint8_t)(length % 128);
        length = length / 128;
        if (length > 0) encoded = encoded | 0x80;
        if (netmqtt_write_byte(w, encoded) < 0) return -1;
    } while (length > 0);
    return 0;
}

int netmqtt_build_connect(netmqtt_writer_t *w, const char *client_id,
                          uint16_t keepalive) {
    netmqtt_writer_t payload;
    netmqtt_writer_init(&payload);

    netmqtt_write_string(&payload, "MQTT");
    netmqtt_write_byte(&payload, 4);
    netmqtt_write_byte(&payload, 0x02);
    netmqtt_write_u16(&payload, keepalive);
    netmqtt_write_string(&payload, client_id);

    netmqtt_write_byte(w, (uint8_t)(NETMQTT_CONNECT << 4));
    netmqtt_encode_remaining_length(w, (uint32_t)payload.pos);
    int i;
    for (i = 0; i < payload.pos; i++) {
        netmqtt_write_byte(w, payload.buf[i]);
    }
    return w->pos;
}

int netmqtt_build_publish(netmqtt_writer_t *w, const char *topic,
                          const uint8_t *payload_data, int payload_len,
                          int qos, int retain) {
    uint8_t flags = (uint8_t)((NETMQTT_PUBLISH << 4) | ((qos & 3) << 1) | (retain ? 1 : 0));
    int topic_len = 0;
    while (topic[topic_len] != 0) topic_len++;
    uint32_t remaining = 2 + (uint32_t)topic_len + (uint32_t)payload_len;
    if (qos > 0) remaining += 2;

    netmqtt_write_byte(w, flags);
    netmqtt_encode_remaining_length(w, remaining);
    netmqtt_write_string(w, topic);
    if (qos > 0) {
        netmqtt_write_u16(w, 1);
    }
    int i;
    for (i = 0; i < payload_len; i++) {
        netmqtt_write_byte(w, payload_data[i]);
    }
    return w->pos;
}

int netmqtt_build_pingreq(netmqtt_writer_t *w) {
    netmqtt_write_byte(w, (uint8_t)(NETMQTT_PINGREQ << 4));
    netmqtt_write_byte(w, 0);
    return w->pos;
}

int main(void) {
    netmqtt_writer_t w;
    netmqtt_writer_init(&w);
    int len = netmqtt_build_connect(&w, "decy-test", 60);
    if (len <= 0) return 1;
    if (w.buf[0] != (NETMQTT_CONNECT << 4)) return 2;
    netmqtt_writer_init(&w);
    netmqtt_build_pingreq(&w);
    if (w.pos != 2) return 3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C722: MQTT packet serializer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C722: should produce non-empty output");
    assert!(code.contains("fn netmqtt_writer_init"), "C722: should contain netmqtt_writer_init");
    assert!(code.contains("fn netmqtt_build_connect"), "C722: should contain netmqtt_build_connect");
}

/// C723: Network interface statistics
#[test]
fn c723_network_interface_statistics() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

#define NETIF_MAX_INTERFACES 8
#define NETIF_NAME_LEN 16
#define NETIF_STATE_DOWN 0
#define NETIF_STATE_UP   1

typedef struct {
    uint32_t rx_bytes;
    uint32_t tx_bytes;
    uint32_t rx_packets;
    uint32_t tx_packets;
    uint32_t rx_errors;
    uint32_t tx_errors;
    uint32_t rx_dropped;
    uint32_t tx_dropped;
    uint32_t collisions;
} netif_stats_t;

typedef struct {
    char name[16];
    int name_len;
    int state;
    uint32_t ip_addr;
    uint32_t netmask;
    uint32_t mtu;
    uint8_t mac[6];
    netif_stats_t stats;
} netif_interface_t;

typedef struct {
    netif_interface_t interfaces[8];
    int count;
} netif_table_t;

void netif_table_init(netif_table_t *table) {
    table->count = 0;
}

static void netif_str_copy(char *dst, const char *src, int max) {
    int i;
    for (i = 0; i < max - 1 && src[i] != 0; i++) {
        dst[i] = src[i];
    }
    dst[i] = 0;
}

int netif_add(netif_table_t *table, const char *name, uint32_t ip,
              uint32_t mask, uint32_t mtu) {
    if (table->count >= NETIF_MAX_INTERFACES) return -1;
    netif_interface_t *iface = &table->interfaces[table->count];
    netif_str_copy(iface->name, name, NETIF_NAME_LEN);
    iface->state = NETIF_STATE_DOWN;
    iface->ip_addr = ip;
    iface->netmask = mask;
    iface->mtu = mtu;
    iface->stats.rx_bytes = 0;
    iface->stats.tx_bytes = 0;
    iface->stats.rx_packets = 0;
    iface->stats.tx_packets = 0;
    iface->stats.rx_errors = 0;
    iface->stats.tx_errors = 0;
    iface->stats.rx_dropped = 0;
    iface->stats.tx_dropped = 0;
    iface->stats.collisions = 0;
    table->count++;
    return table->count - 1;
}

int netif_set_state(netif_table_t *table, int idx, int state) {
    if (idx < 0 || idx >= table->count) return -1;
    table->interfaces[idx].state = state;
    return 0;
}

void netif_record_rx(netif_table_t *table, int idx, uint32_t bytes, int error) {
    if (idx < 0 || idx >= table->count) return;
    if (error) {
        table->interfaces[idx].stats.rx_errors++;
    } else {
        table->interfaces[idx].stats.rx_bytes += bytes;
        table->interfaces[idx].stats.rx_packets++;
    }
}

void netif_record_tx(netif_table_t *table, int idx, uint32_t bytes, int error) {
    if (idx < 0 || idx >= table->count) return;
    if (error) {
        table->interfaces[idx].stats.tx_errors++;
    } else {
        table->interfaces[idx].stats.tx_bytes += bytes;
        table->interfaces[idx].stats.tx_packets++;
    }
}

uint32_t netif_total_rx_bytes(const netif_table_t *table) {
    uint32_t total = 0;
    int i;
    for (i = 0; i < table->count; i++) {
        total += table->interfaces[i].stats.rx_bytes;
    }
    return total;
}

uint32_t netif_total_tx_bytes(const netif_table_t *table) {
    uint32_t total = 0;
    int i;
    for (i = 0; i < table->count; i++) {
        total += table->interfaces[i].stats.tx_bytes;
    }
    return total;
}

int main(void) {
    netif_table_t table;
    netif_table_init(&table);
    int eth0 = netif_add(&table, "eth0", 0xC0A80164, 0xFFFFFF00, 1500);
    netif_set_state(&table, eth0, NETIF_STATE_UP);
    netif_record_rx(&table, eth0, 1024, 0);
    netif_record_tx(&table, eth0, 512, 0);
    if (netif_total_rx_bytes(&table) != 1024) return 1;
    if (netif_total_tx_bytes(&table) != 512) return 2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C723: Network interface statistics should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C723: should produce non-empty output");
    assert!(code.contains("fn netif_table_init"), "C723: should contain netif_table_init");
    assert!(code.contains("fn netif_add"), "C723: should contain netif_add");
    assert!(code.contains("fn netif_record_rx"), "C723: should contain netif_record_rx");
}

/// C724: Bandwidth throttler
#[test]
fn c724_bandwidth_throttler() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

typedef struct {
    uint32_t bytes_per_second;
    uint32_t bucket_size;
    uint32_t available_bytes;
    uint32_t last_update_ms;
    uint32_t total_bytes_passed;
    uint32_t total_bytes_delayed;
    uint32_t total_bytes_dropped;
} netbw_throttle_t;

void netbw_init(netbw_throttle_t *t, uint32_t bytes_per_second, uint32_t burst_size) {
    t->bytes_per_second = bytes_per_second;
    t->bucket_size = burst_size;
    t->available_bytes = burst_size;
    t->last_update_ms = 0;
    t->total_bytes_passed = 0;
    t->total_bytes_delayed = 0;
    t->total_bytes_dropped = 0;
}

void netbw_update(netbw_throttle_t *t, uint32_t now_ms) {
    uint32_t elapsed = now_ms - t->last_update_ms;
    uint32_t new_bytes = (t->bytes_per_second * elapsed) / 1000;
    t->available_bytes += new_bytes;
    if (t->available_bytes > t->bucket_size) {
        t->available_bytes = t->bucket_size;
    }
    t->last_update_ms = now_ms;
}

int netbw_try_send(netbw_throttle_t *t, uint32_t bytes, uint32_t now_ms) {
    netbw_update(t, now_ms);
    if (t->available_bytes >= bytes) {
        t->available_bytes -= bytes;
        t->total_bytes_passed += bytes;
        return 1;
    }
    t->total_bytes_delayed += bytes;
    return 0;
}

uint32_t netbw_delay_ms(const netbw_throttle_t *t, uint32_t bytes) {
    if (t->available_bytes >= bytes) return 0;
    uint32_t deficit = bytes - t->available_bytes;
    if (t->bytes_per_second == 0) return 0xFFFFFFFF;
    return (deficit * 1000) / t->bytes_per_second;
}

float netbw_utilization(const netbw_throttle_t *t) {
    uint32_t total = t->total_bytes_passed + t->total_bytes_delayed + t->total_bytes_dropped;
    if (total == 0) return 0.0f;
    return (float)t->total_bytes_passed / (float)total;
}

void netbw_set_rate(netbw_throttle_t *t, uint32_t new_rate) {
    t->bytes_per_second = new_rate;
}

int main(void) {
    netbw_throttle_t throttle;
    netbw_init(&throttle, 1000, 5000);
    if (!netbw_try_send(&throttle, 3000, 0)) return 1;
    if (!netbw_try_send(&throttle, 2000, 0)) return 2;
    if (netbw_try_send(&throttle, 1000, 0)) return 3;
    if (!netbw_try_send(&throttle, 1000, 1000)) return 4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C724: Bandwidth throttler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C724: should produce non-empty output");
    assert!(code.contains("fn netbw_init"), "C724: should contain netbw_init");
    assert!(code.contains("fn netbw_try_send"), "C724: should contain netbw_try_send");
}

/// C725: Connection timeout manager
#[test]
fn c725_connection_timeout_manager() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;

#define NETTO_MAX_TIMERS 32
#define NETTO_STATE_INACTIVE 0
#define NETTO_STATE_ACTIVE   1
#define NETTO_STATE_EXPIRED  2

typedef struct {
    int conn_id;
    uint32_t deadline_ms;
    uint32_t timeout_ms;
    int state;
    int retries;
    int max_retries;
} netto_timer_t;

typedef struct {
    netto_timer_t timers[32];
    int count;
    uint32_t total_timeouts;
    uint32_t total_retries;
} netto_manager_t;

void netto_init(netto_manager_t *mgr) {
    mgr->count = 0;
    mgr->total_timeouts = 0;
    mgr->total_retries = 0;
    int i;
    for (i = 0; i < NETTO_MAX_TIMERS; i++) {
        mgr->timers[i].state = NETTO_STATE_INACTIVE;
    }
}

int netto_add_timer(netto_manager_t *mgr, int conn_id, uint32_t timeout_ms,
                    int max_retries, uint32_t now_ms) {
    if (mgr->count >= NETTO_MAX_TIMERS) return -1;
    int i;
    for (i = 0; i < NETTO_MAX_TIMERS; i++) {
        if (mgr->timers[i].state == NETTO_STATE_INACTIVE) {
            mgr->timers[i].conn_id = conn_id;
            mgr->timers[i].timeout_ms = timeout_ms;
            mgr->timers[i].deadline_ms = now_ms + timeout_ms;
            mgr->timers[i].state = NETTO_STATE_ACTIVE;
            mgr->timers[i].retries = 0;
            mgr->timers[i].max_retries = max_retries;
            mgr->count++;
            return i;
        }
    }
    return -1;
}

int netto_reset_timer(netto_manager_t *mgr, int timer_id, uint32_t now_ms) {
    if (timer_id < 0 || timer_id >= NETTO_MAX_TIMERS) return -1;
    if (mgr->timers[timer_id].state != NETTO_STATE_ACTIVE) return -2;
    mgr->timers[timer_id].deadline_ms = now_ms + mgr->timers[timer_id].timeout_ms;
    return 0;
}

int netto_cancel_timer(netto_manager_t *mgr, int timer_id) {
    if (timer_id < 0 || timer_id >= NETTO_MAX_TIMERS) return -1;
    if (mgr->timers[timer_id].state == NETTO_STATE_INACTIVE) return -2;
    mgr->timers[timer_id].state = NETTO_STATE_INACTIVE;
    mgr->count--;
    return 0;
}

int netto_check_expired(netto_manager_t *mgr, uint32_t now_ms,
                        int *expired_conn_ids, int max_expired) {
    int count = 0;
    int i;
    for (i = 0; i < NETTO_MAX_TIMERS && count < max_expired; i++) {
        if (mgr->timers[i].state == NETTO_STATE_ACTIVE &&
            now_ms >= mgr->timers[i].deadline_ms) {
            if (mgr->timers[i].retries < mgr->timers[i].max_retries) {
                mgr->timers[i].retries++;
                mgr->timers[i].deadline_ms = now_ms + mgr->timers[i].timeout_ms;
                mgr->total_retries++;
            } else {
                mgr->timers[i].state = NETTO_STATE_EXPIRED;
                expired_conn_ids[count++] = mgr->timers[i].conn_id;
                mgr->total_timeouts++;
                mgr->count--;
            }
        }
    }
    return count;
}

int netto_active_count(const netto_manager_t *mgr) {
    int n = 0;
    int i;
    for (i = 0; i < NETTO_MAX_TIMERS; i++) {
        if (mgr->timers[i].state == NETTO_STATE_ACTIVE) n++;
    }
    return n;
}

uint32_t netto_next_deadline(const netto_manager_t *mgr) {
    uint32_t earliest = 0xFFFFFFFF;
    int i;
    for (i = 0; i < NETTO_MAX_TIMERS; i++) {
        if (mgr->timers[i].state == NETTO_STATE_ACTIVE &&
            mgr->timers[i].deadline_ms < earliest) {
            earliest = mgr->timers[i].deadline_ms;
        }
    }
    return earliest;
}

int main(void) {
    netto_manager_t mgr;
    netto_init(&mgr);
    int t1 = netto_add_timer(&mgr, 100, 5000, 3, 0);
    int t2 = netto_add_timer(&mgr, 200, 3000, 0, 0);
    if (t1 < 0 || t2 < 0) return 1;
    if (netto_active_count(&mgr) != 2) return 2;
    int expired[4];
    int n = netto_check_expired(&mgr, 4000, expired, 4);
    if (n != 1) return 3;
    if (expired[0] != 200) return 4;
    netto_reset_timer(&mgr, t1, 4000);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C725: Connection timeout manager should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C725: should produce non-empty output");
    assert!(code.contains("fn netto_init"), "C725: should contain netto_init");
    assert!(code.contains("fn netto_add_timer"), "C725: should contain netto_add_timer");
    assert!(code.contains("fn netto_check_expired"), "C725: should contain netto_check_expired");
}
