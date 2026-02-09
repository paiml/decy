//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C251-C275: Systems programming C patterns - OS kernel-style, embedded,
//! networking, and performance-critical low-level code.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world systems programming patterns commonly
//! found in OS kernels, network stacks, embedded firmware, and
//! high-performance infrastructure code -- all expressed as valid C99.
//!
//! Organization:
//! - C251-C255: Network and hardware register patterns
//! - C256-C260: Kernel and system data structures
//! - C261-C265: Advanced data structure patterns
//! - C266-C270: Signal handling, endianness, CRC, and I/O patterns
//! - C271-C275: Infrastructure and protocol patterns
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C251-C255: Network and Hardware Register Patterns
// ============================================================================

#[test]
fn c251_packet_header_parsing_with_bitfields() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

struct ip_header {
    unsigned int version : 4;
    unsigned int ihl : 4;
    unsigned int dscp : 6;
    unsigned int ecn : 2;
    uint16_t total_length;
    uint16_t identification;
    unsigned int flags : 3;
    unsigned int fragment_offset : 13;
    uint8_t ttl;
    uint8_t protocol;
    uint16_t header_checksum;
    uint32_t src_addr;
    uint32_t dst_addr;
};

uint16_t compute_checksum(const struct ip_header *hdr) {
    const uint16_t *data = (const uint16_t *)hdr;
    uint32_t sum = 0;
    int words = hdr->ihl * 2;
    for (int i = 0; i < words; i++) {
        sum += data[i];
    }
    while (sum >> 16) {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    return (uint16_t)(~sum);
}

int is_fragment(const struct ip_header *hdr) {
    return (hdr->flags & 0x1) || (hdr->fragment_offset != 0);
}

int header_length_bytes(const struct ip_header *hdr) {
    return hdr->ihl * 4;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C251: Packet header parsing with bitfields should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C251: Output should not be empty");
    assert!(
        code.contains("fn compute_checksum"),
        "C251: Should contain compute_checksum function"
    );
    assert!(
        code.contains("fn is_fragment"),
        "C251: Should contain is_fragment function"
    );
}

#[test]
fn c252_socket_circular_buffer() {
    let c_code = r#"
#define RING_SIZE 4096

typedef struct {
    unsigned char data[4096];
    int head;
    int tail;
    int count;
} ring_buffer_t;

void ring_init(ring_buffer_t *rb) {
    rb->head = 0;
    rb->tail = 0;
    rb->count = 0;
}

int ring_is_full(const ring_buffer_t *rb) {
    return rb->count == 4096;
}

int ring_is_empty(const ring_buffer_t *rb) {
    return rb->count == 0;
}

int ring_write(ring_buffer_t *rb, const unsigned char *src, int len) {
    int written = 0;
    for (int i = 0; i < len && rb->count < 4096; i++) {
        rb->data[rb->head] = src[i];
        rb->head = (rb->head + 1) % 4096;
        rb->count++;
        written++;
    }
    return written;
}

int ring_read(ring_buffer_t *rb, unsigned char *dst, int len) {
    int read_count = 0;
    for (int i = 0; i < len && rb->count > 0; i++) {
        dst[i] = rb->data[rb->tail];
        rb->tail = (rb->tail + 1) % 4096;
        rb->count--;
        read_count++;
    }
    return read_count;
}

int ring_peek(const ring_buffer_t *rb, unsigned char *dst, int len) {
    int peek_count = 0;
    int idx = rb->tail;
    for (int i = 0; i < len && i < rb->count; i++) {
        dst[i] = rb->data[idx];
        idx = (idx + 1) % 4096;
        peek_count++;
    }
    return peek_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C252: Socket circular buffer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C252: Output should not be empty");
    assert!(
        code.contains("fn ring_init"),
        "C252: Should contain ring_init function"
    );
    assert!(
        code.contains("fn ring_write"),
        "C252: Should contain ring_write function"
    );
    assert!(
        code.contains("fn ring_read"),
        "C252: Should contain ring_read function"
    );
}

#[test]
fn c253_page_table_entry_bit_masking() {
    let c_code = r#"
typedef unsigned long uint64_t;

#define PTE_PRESENT  (1UL << 0)
#define PTE_WRITABLE (1UL << 1)
#define PTE_USER     (1UL << 2)
#define PTE_PWT      (1UL << 3)
#define PTE_PCD      (1UL << 4)
#define PTE_ACCESSED (1UL << 5)
#define PTE_DIRTY    (1UL << 6)
#define PTE_HUGE     (1UL << 7)
#define PTE_GLOBAL   (1UL << 8)
#define PTE_NX       (1UL << 63)
#define PTE_ADDR_MASK 0x000FFFFFFFFFF000UL

uint64_t pte_make(uint64_t phys_addr, uint64_t flags) {
    return (phys_addr & PTE_ADDR_MASK) | flags;
}

uint64_t pte_get_addr(uint64_t pte) {
    return pte & PTE_ADDR_MASK;
}

int pte_is_present(uint64_t pte) {
    return (pte & PTE_PRESENT) != 0;
}

int pte_is_writable(uint64_t pte) {
    return (pte & PTE_WRITABLE) != 0;
}

uint64_t pte_set_flags(uint64_t pte, uint64_t flags) {
    return pte | flags;
}

uint64_t pte_clear_flags(uint64_t pte, uint64_t flags) {
    return pte & ~flags;
}

int pte_page_index(uint64_t vaddr, int level) {
    int shift = 12 + level * 9;
    return (int)((vaddr >> shift) & 0x1FF);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C253: Page table entry bit masking should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C253: Output should not be empty");
    assert!(
        code.contains("fn pte_make"),
        "C253: Should contain pte_make function"
    );
    assert!(
        code.contains("fn pte_get_addr"),
        "C253: Should contain pte_get_addr function"
    );
    assert!(
        code.contains("fn pte_is_present"),
        "C253: Should contain pte_is_present function"
    );
}

#[test]
fn c254_interrupt_handler_table() {
    let c_code = r#"
#define MAX_INTERRUPTS 256

typedef void (*isr_handler_t)(int irq_num, void *context);

typedef struct {
    isr_handler_t handler;
    void *context;
    int enabled;
    unsigned int count;
} irq_entry_t;

static irq_entry_t irq_table[256];

void irq_init(void) {
    for (int i = 0; i < 256; i++) {
        irq_table[i].handler = 0;
        irq_table[i].context = 0;
        irq_table[i].enabled = 0;
        irq_table[i].count = 0;
    }
}

int irq_register(int irq, isr_handler_t handler, void *ctx) {
    if (irq < 0 || irq >= 256) return -1;
    if (irq_table[irq].handler != 0) return -2;
    irq_table[irq].handler = handler;
    irq_table[irq].context = ctx;
    irq_table[irq].enabled = 1;
    return 0;
}

int irq_unregister(int irq) {
    if (irq < 0 || irq >= 256) return -1;
    irq_table[irq].handler = 0;
    irq_table[irq].context = 0;
    irq_table[irq].enabled = 0;
    return 0;
}

void irq_dispatch(int irq) {
    if (irq >= 0 && irq < 256 && irq_table[irq].enabled && irq_table[irq].handler) {
        irq_table[irq].count++;
        irq_table[irq].handler(irq, irq_table[irq].context);
    }
}

unsigned int irq_get_count(int irq) {
    if (irq < 0 || irq >= 256) return 0;
    return irq_table[irq].count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C254: Interrupt handler table should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C254: Output should not be empty");
    assert!(
        code.contains("fn irq_init"),
        "C254: Should contain irq_init function"
    );
    assert!(
        code.contains("fn irq_register"),
        "C254: Should contain irq_register function"
    );
    assert!(
        code.contains("fn irq_dispatch"),
        "C254: Should contain irq_dispatch function"
    );
}

#[test]
fn c255_memory_mapped_register_access() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    volatile uint32_t control;
    volatile uint32_t status;
    volatile uint32_t data;
    volatile uint32_t interrupt_mask;
    volatile uint32_t interrupt_status;
} uart_regs_t;

#define UART_CTRL_ENABLE    (1U << 0)
#define UART_CTRL_TX_EN     (1U << 1)
#define UART_CTRL_RX_EN     (1U << 2)
#define UART_STATUS_TX_FULL (1U << 0)
#define UART_STATUS_RX_READY (1U << 1)

void uart_init(uart_regs_t *regs) {
    regs->control = 0;
    regs->interrupt_mask = 0;
    regs->control = UART_CTRL_ENABLE | UART_CTRL_TX_EN | UART_CTRL_RX_EN;
}

void uart_write_byte(uart_regs_t *regs, unsigned char byte) {
    while (regs->status & UART_STATUS_TX_FULL) {
        /* spin wait */
    }
    regs->data = byte;
}

int uart_read_byte(uart_regs_t *regs) {
    if (!(regs->status & UART_STATUS_RX_READY)) {
        return -1;
    }
    return (int)(regs->data & 0xFF);
}

void uart_write_string(uart_regs_t *regs, const char *str) {
    while (*str) {
        uart_write_byte(regs, (unsigned char)*str);
        str++;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C255: Memory-mapped register access should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C255: Output should not be empty");
    assert!(
        code.contains("fn uart_init"),
        "C255: Should contain uart_init function"
    );
    assert!(
        code.contains("fn uart_write_byte"),
        "C255: Should contain uart_write_byte function"
    );
    assert!(
        code.contains("fn uart_read_byte"),
        "C255: Should contain uart_read_byte function"
    );
}

// ============================================================================
// C256-C260: Kernel and System Data Structures
// ============================================================================

#[test]
fn c256_dma_descriptor_ring() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define DMA_DESC_COUNT 128
#define DMA_OWN_BIT   (1U << 31)
#define DMA_FIRST_BIT (1U << 29)
#define DMA_LAST_BIT  (1U << 28)

typedef struct {
    uint32_t status;
    uint32_t control;
    uint64_t buf_addr;
    uint64_t next_desc;
} dma_desc_t;

typedef struct {
    dma_desc_t descs[128];
    int producer;
    int consumer;
    int free_count;
} dma_ring_t;

void dma_ring_init(dma_ring_t *ring) {
    ring->producer = 0;
    ring->consumer = 0;
    ring->free_count = 128;
    for (int i = 0; i < 128; i++) {
        ring->descs[i].status = 0;
        ring->descs[i].control = 0;
        ring->descs[i].buf_addr = 0;
        ring->descs[i].next_desc = 0;
    }
}

int dma_submit(dma_ring_t *ring, uint64_t buf, uint32_t len) {
    if (ring->free_count == 0) return -1;
    int idx = ring->producer;
    ring->descs[idx].buf_addr = buf;
    ring->descs[idx].control = len & 0xFFFF;
    ring->descs[idx].status = DMA_OWN_BIT | DMA_FIRST_BIT | DMA_LAST_BIT;
    ring->producer = (ring->producer + 1) % 128;
    ring->free_count--;
    return 0;
}

int dma_complete(dma_ring_t *ring) {
    if (ring->free_count == 128) return -1;
    int idx = ring->consumer;
    if (ring->descs[idx].status & DMA_OWN_BIT) {
        return -2;
    }
    ring->descs[idx].status = 0;
    ring->consumer = (ring->consumer + 1) % 128;
    ring->free_count++;
    return 0;
}

int dma_pending(const dma_ring_t *ring) {
    return 128 - ring->free_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C256: DMA descriptor ring should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C256: Output should not be empty");
    assert!(
        code.contains("fn dma_ring_init"),
        "C256: Should contain dma_ring_init function"
    );
    assert!(
        code.contains("fn dma_submit"),
        "C256: Should contain dma_submit function"
    );
    assert!(
        code.contains("fn dma_complete"),
        "C256: Should contain dma_complete function"
    );
}

#[test]
fn c257_filesystem_inode_structure() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define DIRECT_BLOCKS 12
#define BLOCK_SIZE 4096

typedef struct {
    uint32_t mode;
    uint32_t uid;
    uint32_t gid;
    uint32_t nlinks;
    uint64_t size;
    uint64_t atime;
    uint64_t mtime;
    uint64_t ctime;
    uint32_t direct[12];
    uint32_t indirect;
    uint32_t double_indirect;
    uint32_t triple_indirect;
} inode_t;

uint64_t inode_max_size(void) {
    uint64_t ptrs_per_block = 4096 / 4;
    uint64_t direct = 12 * 4096;
    uint64_t single = ptrs_per_block * 4096;
    uint64_t dbl = ptrs_per_block * ptrs_per_block * 4096;
    return direct + single + dbl;
}

int inode_block_index(const inode_t *inode, uint64_t offset) {
    uint32_t block_num = (uint32_t)(offset / 4096);
    if (block_num < 12) {
        return (int)inode->direct[block_num];
    }
    return -1;
}

int inode_is_directory(const inode_t *inode) {
    return (inode->mode & 0xF000) == 0x4000;
}

int inode_is_regular(const inode_t *inode) {
    return (inode->mode & 0xF000) == 0x8000;
}

uint32_t inode_permissions(const inode_t *inode) {
    return inode->mode & 0x1FF;
}

int inode_can_read(const inode_t *inode, uint32_t uid) {
    if (uid == 0) return 1;
    if (uid == inode->uid) return (inode->mode >> 6) & 0x4;
    return inode->mode & 0x4;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C257: Filesystem inode structure should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C257: Output should not be empty");
    assert!(
        code.contains("fn inode_max_size"),
        "C257: Should contain inode_max_size function"
    );
    assert!(
        code.contains("fn inode_block_index"),
        "C257: Should contain inode_block_index function"
    );
    assert!(
        code.contains("fn inode_is_directory"),
        "C257: Should contain inode_is_directory function"
    );
}

#[test]
fn c258_network_protocol_state_machine() {
    let c_code = r#"
enum tcp_state {
    TCP_CLOSED,
    TCP_LISTEN,
    TCP_SYN_SENT,
    TCP_SYN_RECEIVED,
    TCP_ESTABLISHED,
    TCP_FIN_WAIT_1,
    TCP_FIN_WAIT_2,
    TCP_CLOSE_WAIT,
    TCP_CLOSING,
    TCP_LAST_ACK,
    TCP_TIME_WAIT
};

enum tcp_event {
    EVT_OPEN_PASSIVE,
    EVT_OPEN_ACTIVE,
    EVT_SYN_RECV,
    EVT_SYN_ACK_RECV,
    EVT_ACK_RECV,
    EVT_FIN_RECV,
    EVT_CLOSE,
    EVT_TIMEOUT
};

typedef struct {
    enum tcp_state state;
    unsigned int seq_num;
    unsigned int ack_num;
    unsigned int window_size;
    int retransmit_count;
} tcp_conn_t;

void tcp_init(tcp_conn_t *conn) {
    conn->state = TCP_CLOSED;
    conn->seq_num = 0;
    conn->ack_num = 0;
    conn->window_size = 65535;
    conn->retransmit_count = 0;
}

int tcp_transition(tcp_conn_t *conn, enum tcp_event event) {
    switch (conn->state) {
    case TCP_CLOSED:
        if (event == EVT_OPEN_PASSIVE) { conn->state = TCP_LISTEN; return 0; }
        if (event == EVT_OPEN_ACTIVE) { conn->state = TCP_SYN_SENT; return 0; }
        break;
    case TCP_LISTEN:
        if (event == EVT_SYN_RECV) { conn->state = TCP_SYN_RECEIVED; return 0; }
        if (event == EVT_CLOSE) { conn->state = TCP_CLOSED; return 0; }
        break;
    case TCP_SYN_SENT:
        if (event == EVT_SYN_ACK_RECV) { conn->state = TCP_ESTABLISHED; return 0; }
        if (event == EVT_CLOSE) { conn->state = TCP_CLOSED; return 0; }
        break;
    case TCP_ESTABLISHED:
        if (event == EVT_FIN_RECV) { conn->state = TCP_CLOSE_WAIT; return 0; }
        if (event == EVT_CLOSE) { conn->state = TCP_FIN_WAIT_1; return 0; }
        break;
    case TCP_FIN_WAIT_1:
        if (event == EVT_ACK_RECV) { conn->state = TCP_FIN_WAIT_2; return 0; }
        if (event == EVT_FIN_RECV) { conn->state = TCP_CLOSING; return 0; }
        break;
    case TCP_FIN_WAIT_2:
        if (event == EVT_FIN_RECV) { conn->state = TCP_TIME_WAIT; return 0; }
        break;
    case TCP_CLOSE_WAIT:
        if (event == EVT_CLOSE) { conn->state = TCP_LAST_ACK; return 0; }
        break;
    case TCP_CLOSING:
        if (event == EVT_ACK_RECV) { conn->state = TCP_TIME_WAIT; return 0; }
        break;
    case TCP_LAST_ACK:
        if (event == EVT_ACK_RECV) { conn->state = TCP_CLOSED; return 0; }
        break;
    case TCP_TIME_WAIT:
        if (event == EVT_TIMEOUT) { conn->state = TCP_CLOSED; return 0; }
        break;
    default:
        break;
    }
    return -1;
}

int tcp_is_connected(const tcp_conn_t *conn) {
    return conn->state == TCP_ESTABLISHED;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C258: Network protocol state machine should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C258: Output should not be empty");
    assert!(
        code.contains("fn tcp_init"),
        "C258: Should contain tcp_init function"
    );
    assert!(
        code.contains("fn tcp_transition"),
        "C258: Should contain tcp_transition function"
    );
    assert!(
        code.contains("fn tcp_is_connected"),
        "C258: Should contain tcp_is_connected function"
    );
}

#[test]
fn c259_lock_free_spsc_queue() {
    let c_code = r#"
#define QUEUE_CAPACITY 1024

typedef struct {
    int buffer[1024];
    int head;
    int tail;
} spsc_queue_t;

void spsc_init(spsc_queue_t *q) {
    q->head = 0;
    q->tail = 0;
}

int spsc_push(spsc_queue_t *q, int value) {
    int next_head = (q->head + 1) % 1024;
    if (next_head == q->tail) {
        return -1;
    }
    q->buffer[q->head] = value;
    q->head = next_head;
    return 0;
}

int spsc_pop(spsc_queue_t *q, int *value) {
    if (q->tail == q->head) {
        return -1;
    }
    *value = q->buffer[q->tail];
    q->tail = (q->tail + 1) % 1024;
    return 0;
}

int spsc_size(const spsc_queue_t *q) {
    int diff = q->head - q->tail;
    if (diff < 0) diff += 1024;
    return diff;
}

int spsc_is_empty(const spsc_queue_t *q) {
    return q->head == q->tail;
}

int spsc_is_full(const spsc_queue_t *q) {
    return ((q->head + 1) % 1024) == q->tail;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C259: Lock-free SPSC queue should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C259: Output should not be empty");
    assert!(
        code.contains("fn spsc_init"),
        "C259: Should contain spsc_init function"
    );
    assert!(
        code.contains("fn spsc_push"),
        "C259: Should contain spsc_push function"
    );
    assert!(
        code.contains("fn spsc_pop"),
        "C259: Should contain spsc_pop function"
    );
}

#[test]
fn c260_timer_wheel_implementation() {
    let c_code = r#"
#define WHEEL_SIZE 256
#define MAX_TIMERS 1024

typedef void (*timer_cb_t)(void *arg);

typedef struct {
    timer_cb_t callback;
    void *arg;
    unsigned int expiry;
    int active;
} timer_entry_t;

typedef struct {
    timer_entry_t timers[1024];
    int wheel[256];
    unsigned int current_tick;
    int timer_count;
} timer_wheel_t;

void tw_init(timer_wheel_t *tw) {
    tw->current_tick = 0;
    tw->timer_count = 0;
    for (int i = 0; i < 256; i++) {
        tw->wheel[i] = -1;
    }
    for (int i = 0; i < 1024; i++) {
        tw->timers[i].active = 0;
    }
}

int tw_add_timer(timer_wheel_t *tw, unsigned int ticks, timer_cb_t cb, void *arg) {
    if (tw->timer_count >= 1024) return -1;
    int slot = -1;
    for (int i = 0; i < 1024; i++) {
        if (!tw->timers[i].active) { slot = i; break; }
    }
    if (slot < 0) return -1;
    tw->timers[slot].callback = cb;
    tw->timers[slot].arg = arg;
    tw->timers[slot].expiry = tw->current_tick + ticks;
    tw->timers[slot].active = 1;
    tw->timer_count++;
    return slot;
}

int tw_cancel_timer(timer_wheel_t *tw, int timer_id) {
    if (timer_id < 0 || timer_id >= 1024) return -1;
    if (!tw->timers[timer_id].active) return -1;
    tw->timers[timer_id].active = 0;
    tw->timer_count--;
    return 0;
}

int tw_tick(timer_wheel_t *tw) {
    tw->current_tick++;
    int fired = 0;
    for (int i = 0; i < 1024; i++) {
        if (tw->timers[i].active && tw->timers[i].expiry == tw->current_tick) {
            tw->timers[i].active = 0;
            tw->timer_count--;
            fired++;
        }
    }
    return fired;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C260: Timer wheel implementation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C260: Output should not be empty");
    assert!(
        code.contains("fn tw_init"),
        "C260: Should contain tw_init function"
    );
    assert!(
        code.contains("fn tw_add_timer"),
        "C260: Should contain tw_add_timer function"
    );
    assert!(
        code.contains("fn tw_tick"),
        "C260: Should contain tw_tick function"
    );
}

// ============================================================================
// C261-C265: Advanced Data Structure Patterns
// ============================================================================

#[test]
fn c261_slab_allocator_fixed_size_pool() {
    let c_code = r#"
#define SLAB_OBJECTS 64
#define OBJECT_SIZE 128

typedef struct {
    unsigned char pool[64 * 128];
    unsigned char bitmap[64];
    int free_count;
} slab_t;

void slab_init(slab_t *s) {
    s->free_count = 64;
    for (int i = 0; i < 64; i++) {
        s->bitmap[i] = 0;
    }
}

int slab_alloc(slab_t *s) {
    if (s->free_count == 0) return -1;
    for (int i = 0; i < 64; i++) {
        if (s->bitmap[i] == 0) {
            s->bitmap[i] = 1;
            s->free_count--;
            return i;
        }
    }
    return -1;
}

void slab_free(slab_t *s, int index) {
    if (index >= 0 && index < 64 && s->bitmap[index]) {
        s->bitmap[index] = 0;
        s->free_count++;
    }
}

unsigned char *slab_get_ptr(slab_t *s, int index) {
    if (index < 0 || index >= 64) return 0;
    return &s->pool[index * 128];
}

int slab_is_full(const slab_t *s) {
    return s->free_count == 0;
}

int slab_utilization_percent(const slab_t *s) {
    return ((64 - s->free_count) * 100) / 64;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C261: Slab allocator should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C261: Output should not be empty");
    assert!(
        code.contains("fn slab_init"),
        "C261: Should contain slab_init function"
    );
    assert!(
        code.contains("fn slab_alloc"),
        "C261: Should contain slab_alloc function"
    );
    assert!(
        code.contains("fn slab_free"),
        "C261: Should contain slab_free function"
    );
}

#[test]
fn c262_bloom_filter() {
    let c_code = r#"
#define BLOOM_BITS 8192
#define BLOOM_BYTES (8192 / 8)
#define BLOOM_HASH_COUNT 3

typedef struct {
    unsigned char bits[1024];
    unsigned int count;
} bloom_filter_t;

void bloom_init(bloom_filter_t *bf) {
    bf->count = 0;
    for (int i = 0; i < 1024; i++) {
        bf->bits[i] = 0;
    }
}

static unsigned int bloom_hash1(unsigned int key) {
    key = ((key >> 16) ^ key) * 0x45d9f3b;
    key = ((key >> 16) ^ key) * 0x45d9f3b;
    key = (key >> 16) ^ key;
    return key % 8192;
}

static unsigned int bloom_hash2(unsigned int key) {
    key = (key + 0x7ed55d16) + (key << 12);
    key = (key ^ 0xc761c23c) ^ (key >> 19);
    key = (key + 0x165667b1) + (key << 5);
    key = (key + 0xd3a2646c) ^ (key << 9);
    key = (key + 0xfd7046c5) + (key << 3);
    key = (key ^ 0xb55a4f09) ^ (key >> 16);
    return key % 8192;
}

static unsigned int bloom_hash3(unsigned int key) {
    key = ~key + (key << 15);
    key = key ^ (key >> 12);
    key = key + (key << 2);
    key = key ^ (key >> 4);
    key = key * 2057;
    key = key ^ (key >> 16);
    return key % 8192;
}

void bloom_add(bloom_filter_t *bf, unsigned int key) {
    unsigned int h1 = bloom_hash1(key);
    unsigned int h2 = bloom_hash2(key);
    unsigned int h3 = bloom_hash3(key);
    bf->bits[h1 / 8] |= (1 << (h1 % 8));
    bf->bits[h2 / 8] |= (1 << (h2 % 8));
    bf->bits[h3 / 8] |= (1 << (h3 % 8));
    bf->count++;
}

int bloom_check(const bloom_filter_t *bf, unsigned int key) {
    unsigned int h1 = bloom_hash1(key);
    unsigned int h2 = bloom_hash2(key);
    unsigned int h3 = bloom_hash3(key);
    if (!(bf->bits[h1 / 8] & (1 << (h1 % 8)))) return 0;
    if (!(bf->bits[h2 / 8] & (1 << (h2 % 8)))) return 0;
    if (!(bf->bits[h3 / 8] & (1 << (h3 % 8)))) return 0;
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C262: Bloom filter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C262: Output should not be empty");
    assert!(
        code.contains("fn bloom_init"),
        "C262: Should contain bloom_init function"
    );
    assert!(
        code.contains("fn bloom_add"),
        "C262: Should contain bloom_add function"
    );
    assert!(
        code.contains("fn bloom_check"),
        "C262: Should contain bloom_check function"
    );
}

#[test]
fn c263_trie_routing_table() {
    let c_code = r#"
#define TRIE_CHILDREN 2

typedef struct trie_node {
    int has_value;
    int value;
    int children[2];
} trie_node_t;

#define MAX_TRIE_NODES 4096

typedef struct {
    trie_node_t nodes[4096];
    int node_count;
} trie_t;

void trie_init(trie_t *t) {
    t->node_count = 1;
    t->nodes[0].has_value = 0;
    t->nodes[0].value = 0;
    t->nodes[0].children[0] = -1;
    t->nodes[0].children[1] = -1;
}

static int trie_new_node(trie_t *t) {
    if (t->node_count >= 4096) return -1;
    int idx = t->node_count++;
    t->nodes[idx].has_value = 0;
    t->nodes[idx].value = 0;
    t->nodes[idx].children[0] = -1;
    t->nodes[idx].children[1] = -1;
    return idx;
}

int trie_insert(trie_t *t, unsigned int prefix, int prefix_len, int value) {
    int node = 0;
    for (int i = prefix_len - 1; i >= 0; i--) {
        int bit = (prefix >> i) & 1;
        if (t->nodes[node].children[bit] == -1) {
            int child = trie_new_node(t);
            if (child < 0) return -1;
            t->nodes[node].children[bit] = child;
        }
        node = t->nodes[node].children[bit];
    }
    t->nodes[node].has_value = 1;
    t->nodes[node].value = value;
    return 0;
}

int trie_lookup(const trie_t *t, unsigned int addr, int addr_bits) {
    int node = 0;
    int best_value = -1;
    for (int i = addr_bits - 1; i >= 0; i--) {
        if (t->nodes[node].has_value) {
            best_value = t->nodes[node].value;
        }
        int bit = (addr >> i) & 1;
        if (t->nodes[node].children[bit] == -1) break;
        node = t->nodes[node].children[bit];
    }
    if (t->nodes[node].has_value) {
        best_value = t->nodes[node].value;
    }
    return best_value;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C263: Trie routing table should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C263: Output should not be empty");
    assert!(
        code.contains("fn trie_init"),
        "C263: Should contain trie_init function"
    );
    assert!(
        code.contains("fn trie_insert"),
        "C263: Should contain trie_insert function"
    );
    assert!(
        code.contains("fn trie_lookup"),
        "C263: Should contain trie_lookup function"
    );
}

#[test]
fn c264_skip_list_node_structure() {
    let c_code = r#"
#define SKIPLIST_MAX_LEVEL 16
#define SKIPLIST_CAPACITY 256

typedef struct {
    int key;
    int value;
    int forward[16];
} skip_node_t;

typedef struct {
    skip_node_t nodes[256];
    int node_count;
    int level;
    int head;
} skiplist_t;

static unsigned int sl_rand_state = 12345;

static int sl_random_level(void) {
    int lvl = 1;
    sl_rand_state = sl_rand_state * 1103515245 + 12345;
    while ((sl_rand_state & 0xFFFF) < 32768 && lvl < 16) {
        lvl++;
        sl_rand_state = sl_rand_state * 1103515245 + 12345;
    }
    return lvl;
}

void sl_init(skiplist_t *sl) {
    sl->node_count = 1;
    sl->level = 1;
    sl->head = 0;
    for (int i = 0; i < 16; i++) {
        sl->nodes[0].forward[i] = -1;
    }
    sl->nodes[0].key = -2147483647;
    sl->nodes[0].value = 0;
}

int sl_search(const skiplist_t *sl, int key) {
    int current = sl->head;
    for (int i = sl->level - 1; i >= 0; i--) {
        while (sl->nodes[current].forward[i] != -1 &&
               sl->nodes[sl->nodes[current].forward[i]].key < key) {
            current = sl->nodes[current].forward[i];
        }
    }
    int next = sl->nodes[current].forward[0];
    if (next != -1 && sl->nodes[next].key == key) {
        return sl->nodes[next].value;
    }
    return -1;
}

int sl_count(const skiplist_t *sl) {
    return sl->node_count - 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C264: Skip list node structure should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C264: Output should not be empty");
    assert!(
        code.contains("fn sl_init"),
        "C264: Should contain sl_init function"
    );
    assert!(
        code.contains("fn sl_search"),
        "C264: Should contain sl_search function"
    );
    assert!(
        code.contains("fn sl_count"),
        "C264: Should contain sl_count function"
    );
}

#[test]
fn c265_copy_on_write_refcounting() {
    let c_code = r#"
#define COW_BUF_SIZE 256

typedef struct {
    unsigned char data[256];
    int refcount;
    int len;
} cow_buffer_t;

typedef struct {
    cow_buffer_t *buf;
    int owned;
} cow_handle_t;

void cow_buf_init(cow_buffer_t *buf, int len) {
    buf->refcount = 1;
    buf->len = len;
    for (int i = 0; i < len && i < 256; i++) {
        buf->data[i] = 0;
    }
}

cow_handle_t cow_share(cow_buffer_t *buf) {
    cow_handle_t h;
    buf->refcount++;
    h.buf = buf;
    h.owned = 0;
    return h;
}

int cow_is_shared(const cow_buffer_t *buf) {
    return buf->refcount > 1;
}

void cow_release(cow_handle_t *h) {
    if (h->buf) {
        h->buf->refcount--;
        h->buf = 0;
    }
}

int cow_write_byte(cow_buffer_t *buf, int offset, unsigned char value) {
    if (offset < 0 || offset >= buf->len) return -1;
    buf->data[offset] = value;
    return 0;
}

unsigned char cow_read_byte(const cow_buffer_t *buf, int offset) {
    if (offset < 0 || offset >= buf->len) return 0;
    return buf->data[offset];
}

int cow_get_refcount(const cow_buffer_t *buf) {
    return buf->refcount;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C265: Copy-on-write refcounting should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C265: Output should not be empty");
    assert!(
        code.contains("fn cow_buf_init"),
        "C265: Should contain cow_buf_init function"
    );
    assert!(
        code.contains("fn cow_share"),
        "C265: Should contain cow_share function"
    );
    assert!(
        code.contains("fn cow_release"),
        "C265: Should contain cow_release function"
    );
}

// ============================================================================
// C266-C270: Signal Handling, Endianness, CRC, and I/O Patterns
// ============================================================================

#[test]
fn c266_signal_handler_registration() {
    let c_code = r#"
#define MAX_SIGNALS 32

typedef void (*sig_handler_t)(int);

typedef struct {
    sig_handler_t handler;
    int masked;
    int pending;
    unsigned int delivery_count;
} signal_entry_t;

typedef struct {
    signal_entry_t signals[32];
    unsigned int mask;
} signal_table_t;

void sig_table_init(signal_table_t *st) {
    st->mask = 0;
    for (int i = 0; i < 32; i++) {
        st->signals[i].handler = 0;
        st->signals[i].masked = 0;
        st->signals[i].pending = 0;
        st->signals[i].delivery_count = 0;
    }
}

int sig_register(signal_table_t *st, int signum, sig_handler_t handler) {
    if (signum < 0 || signum >= 32) return -1;
    st->signals[signum].handler = handler;
    return 0;
}

int sig_mask(signal_table_t *st, int signum) {
    if (signum < 0 || signum >= 32) return -1;
    st->signals[signum].masked = 1;
    st->mask |= (1U << signum);
    return 0;
}

int sig_unmask(signal_table_t *st, int signum) {
    if (signum < 0 || signum >= 32) return -1;
    st->signals[signum].masked = 0;
    st->mask &= ~(1U << signum);
    return 0;
}

int sig_raise(signal_table_t *st, int signum) {
    if (signum < 0 || signum >= 32) return -1;
    if (st->signals[signum].masked) {
        st->signals[signum].pending = 1;
        return 1;
    }
    if (st->signals[signum].handler) {
        st->signals[signum].delivery_count++;
        return 0;
    }
    return -2;
}

int sig_pending_count(const signal_table_t *st) {
    int count = 0;
    for (int i = 0; i < 32; i++) {
        if (st->signals[i].pending) count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C266: Signal handler registration should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C266: Output should not be empty");
    assert!(
        code.contains("fn sig_table_init"),
        "C266: Should contain sig_table_init function"
    );
    assert!(
        code.contains("fn sig_register"),
        "C266: Should contain sig_register function"
    );
    assert!(
        code.contains("fn sig_raise"),
        "C266: Should contain sig_raise function"
    );
}

#[test]
fn c267_endianness_conversion_functions() {
    let c_code = r#"
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

uint16_t swap16(uint16_t val) {
    return (val << 8) | (val >> 8);
}

uint32_t swap32(uint32_t val) {
    return ((val & 0xFF000000) >> 24) |
           ((val & 0x00FF0000) >> 8)  |
           ((val & 0x0000FF00) << 8)  |
           ((val & 0x000000FF) << 24);
}

uint64_t swap64(uint64_t val) {
    val = ((val << 8) & 0xFF00FF00FF00FF00ULL) |
          ((val >> 8) & 0x00FF00FF00FF00FFULL);
    val = ((val << 16) & 0xFFFF0000FFFF0000ULL) |
          ((val >> 16) & 0x0000FFFF0000FFFFULL);
    return (val << 32) | (val >> 32);
}

int is_big_endian(void) {
    unsigned int test = 1;
    unsigned char *byte = (unsigned char *)&test;
    return byte[0] == 0;
}

uint32_t read_be32(const unsigned char *buf) {
    return ((uint32_t)buf[0] << 24) |
           ((uint32_t)buf[1] << 16) |
           ((uint32_t)buf[2] << 8)  |
           ((uint32_t)buf[3]);
}

void write_be32(unsigned char *buf, uint32_t val) {
    buf[0] = (unsigned char)(val >> 24);
    buf[1] = (unsigned char)(val >> 16);
    buf[2] = (unsigned char)(val >> 8);
    buf[3] = (unsigned char)(val);
}

uint16_t read_le16(const unsigned char *buf) {
    return ((uint16_t)buf[1] << 8) | (uint16_t)buf[0];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C267: Endianness conversion functions should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C267: Output should not be empty");
    assert!(
        code.contains("fn swap16"),
        "C267: Should contain swap16 function"
    );
    assert!(
        code.contains("fn swap32"),
        "C267: Should contain swap32 function"
    );
    assert!(
        code.contains("fn swap64"),
        "C267: Should contain swap64 function"
    );
    assert!(
        code.contains("fn read_be32"),
        "C267: Should contain read_be32 function"
    );
}

#[test]
fn c268_crc32_lookup_table() {
    let c_code = r#"
typedef unsigned int uint32_t;

static uint32_t crc32_table[256];
static int crc32_table_init = 0;

void crc32_generate_table(void) {
    for (unsigned int i = 0; i < 256; i++) {
        uint32_t crc = i;
        for (int j = 0; j < 8; j++) {
            if (crc & 1) {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc = crc >> 1;
            }
        }
        crc32_table[i] = crc;
    }
    crc32_table_init = 1;
}

uint32_t crc32_compute(const unsigned char *data, int len) {
    if (!crc32_table_init) {
        crc32_generate_table();
    }
    uint32_t crc = 0xFFFFFFFF;
    for (int i = 0; i < len; i++) {
        unsigned char index = (unsigned char)((crc ^ data[i]) & 0xFF);
        crc = (crc >> 8) ^ crc32_table[index];
    }
    return crc ^ 0xFFFFFFFF;
}

uint32_t crc32_update(uint32_t prev_crc, const unsigned char *data, int len) {
    if (!crc32_table_init) {
        crc32_generate_table();
    }
    uint32_t crc = prev_crc ^ 0xFFFFFFFF;
    for (int i = 0; i < len; i++) {
        unsigned char index = (unsigned char)((crc ^ data[i]) & 0xFF);
        crc = (crc >> 8) ^ crc32_table[index];
    }
    return crc ^ 0xFFFFFFFF;
}

int crc32_verify(const unsigned char *data, int len, uint32_t expected) {
    return crc32_compute(data, len) == expected;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C268: CRC32 lookup table should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C268: Output should not be empty");
    assert!(
        code.contains("fn crc32_generate_table"),
        "C268: Should contain crc32_generate_table function"
    );
    assert!(
        code.contains("fn crc32_compute"),
        "C268: Should contain crc32_compute function"
    );
    assert!(
        code.contains("fn crc32_verify"),
        "C268: Should contain crc32_verify function"
    );
}

#[test]
fn c269_memory_barrier_fence_simulation() {
    let c_code = r#"
typedef struct {
    int sequence;
    int data;
    int ready;
} shared_state_t;

void ss_init(shared_state_t *ss) {
    ss->sequence = 0;
    ss->data = 0;
    ss->ready = 0;
}

void ss_publish(shared_state_t *ss, int value) {
    ss->data = value;
    ss->sequence++;
    ss->ready = 1;
}

int ss_read(const shared_state_t *ss, int *out_seq) {
    if (!ss->ready) return -1;
    *out_seq = ss->sequence;
    return ss->data;
}

int ss_try_read_consistent(const shared_state_t *ss, int *value) {
    int seq1 = ss->sequence;
    if (!ss->ready) return -1;
    *value = ss->data;
    int seq2 = ss->sequence;
    if (seq1 != seq2) return -2;
    return 0;
}

typedef struct {
    int counter;
    int version;
} seqlock_t;

void seqlock_init(seqlock_t *sl) {
    sl->counter = 0;
    sl->version = 0;
}

void seqlock_write_begin(seqlock_t *sl) {
    sl->version++;
}

void seqlock_write_end(seqlock_t *sl) {
    sl->version++;
}

int seqlock_read_begin(const seqlock_t *sl) {
    return sl->version;
}

int seqlock_read_valid(const seqlock_t *sl, int start_version) {
    return (sl->version == start_version) && ((start_version & 1) == 0);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C269: Memory barrier fence simulation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C269: Output should not be empty");
    assert!(
        code.contains("fn ss_init"),
        "C269: Should contain ss_init function"
    );
    assert!(
        code.contains("fn ss_publish"),
        "C269: Should contain ss_publish function"
    );
    assert!(
        code.contains("fn seqlock_init"),
        "C269: Should contain seqlock_init function"
    );
}

#[test]
fn c270_scatter_gather_io_vector() {
    let c_code = r#"
#define MAX_IOV 16

typedef struct {
    unsigned char *base;
    int len;
} iovec_t;

typedef struct {
    iovec_t iov[16];
    int iov_count;
} sg_list_t;

void sg_init(sg_list_t *sg) {
    sg->iov_count = 0;
}

int sg_add(sg_list_t *sg, unsigned char *buf, int len) {
    if (sg->iov_count >= 16) return -1;
    sg->iov[sg->iov_count].base = buf;
    sg->iov[sg->iov_count].len = len;
    sg->iov_count++;
    return 0;
}

int sg_total_len(const sg_list_t *sg) {
    int total = 0;
    for (int i = 0; i < sg->iov_count; i++) {
        total += sg->iov[i].len;
    }
    return total;
}

int sg_copy_to_flat(const sg_list_t *sg, unsigned char *dst, int dst_len) {
    int offset = 0;
    for (int i = 0; i < sg->iov_count; i++) {
        int copy_len = sg->iov[i].len;
        if (offset + copy_len > dst_len) {
            copy_len = dst_len - offset;
        }
        if (copy_len <= 0) break;
        for (int j = 0; j < copy_len; j++) {
            dst[offset + j] = sg->iov[i].base[j];
        }
        offset += copy_len;
    }
    return offset;
}

int sg_find_offset(const sg_list_t *sg, int global_offset, int *iov_idx, int *local_offset) {
    int cumulative = 0;
    for (int i = 0; i < sg->iov_count; i++) {
        if (global_offset < cumulative + sg->iov[i].len) {
            *iov_idx = i;
            *local_offset = global_offset - cumulative;
            return 0;
        }
        cumulative += sg->iov[i].len;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C270: Scatter-gather I/O vector should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C270: Output should not be empty");
    assert!(
        code.contains("fn sg_init"),
        "C270: Should contain sg_init function"
    );
    assert!(
        code.contains("fn sg_total_len"),
        "C270: Should contain sg_total_len function"
    );
    assert!(
        code.contains("fn sg_copy_to_flat"),
        "C270: Should contain sg_copy_to_flat function"
    );
}

// ============================================================================
// C271-C275: Infrastructure and Protocol Patterns
// ============================================================================

#[test]
fn c271_connection_pool_with_timeout() {
    let c_code = r#"
#define POOL_SIZE 64

typedef struct {
    int socket_fd;
    unsigned long last_used;
    int in_use;
    int healthy;
} pool_conn_t;

typedef struct {
    pool_conn_t conns[64];
    int active_count;
    unsigned long timeout_ms;
} conn_pool_t;

void pool_init(conn_pool_t *pool, unsigned long timeout_ms) {
    pool->active_count = 0;
    pool->timeout_ms = timeout_ms;
    for (int i = 0; i < 64; i++) {
        pool->conns[i].socket_fd = -1;
        pool->conns[i].last_used = 0;
        pool->conns[i].in_use = 0;
        pool->conns[i].healthy = 0;
    }
}

int pool_acquire(conn_pool_t *pool, unsigned long now) {
    for (int i = 0; i < 64; i++) {
        if (!pool->conns[i].in_use && pool->conns[i].healthy) {
            if (now - pool->conns[i].last_used < pool->timeout_ms) {
                pool->conns[i].in_use = 1;
                pool->conns[i].last_used = now;
                return i;
            }
        }
    }
    return -1;
}

void pool_release(conn_pool_t *pool, int idx, unsigned long now) {
    if (idx >= 0 && idx < 64) {
        pool->conns[idx].in_use = 0;
        pool->conns[idx].last_used = now;
    }
}

int pool_add(conn_pool_t *pool, int socket_fd, unsigned long now) {
    for (int i = 0; i < 64; i++) {
        if (pool->conns[i].socket_fd == -1) {
            pool->conns[i].socket_fd = socket_fd;
            pool->conns[i].last_used = now;
            pool->conns[i].in_use = 0;
            pool->conns[i].healthy = 1;
            pool->active_count++;
            return i;
        }
    }
    return -1;
}

int pool_evict_expired(conn_pool_t *pool, unsigned long now) {
    int evicted = 0;
    for (int i = 0; i < 64; i++) {
        if (pool->conns[i].socket_fd != -1 && !pool->conns[i].in_use) {
            if (now - pool->conns[i].last_used >= pool->timeout_ms) {
                pool->conns[i].socket_fd = -1;
                pool->conns[i].healthy = 0;
                pool->active_count--;
                evicted++;
            }
        }
    }
    return evicted;
}

int pool_healthy_count(const conn_pool_t *pool) {
    int count = 0;
    for (int i = 0; i < 64; i++) {
        if (pool->conns[i].healthy) count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C271: Connection pool with timeout should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C271: Output should not be empty");
    assert!(
        code.contains("fn pool_init"),
        "C271: Should contain pool_init function"
    );
    assert!(
        code.contains("fn pool_acquire"),
        "C271: Should contain pool_acquire function"
    );
    assert!(
        code.contains("fn pool_evict_expired"),
        "C271: Should contain pool_evict_expired function"
    );
}

#[test]
fn c272_rate_limiter_token_bucket() {
    let c_code = r#"
typedef struct {
    double tokens;
    double max_tokens;
    double refill_rate;
    unsigned long last_refill;
} token_bucket_t;

void tb_init(token_bucket_t *tb, double max_tokens, double refill_rate) {
    tb->tokens = max_tokens;
    tb->max_tokens = max_tokens;
    tb->refill_rate = refill_rate;
    tb->last_refill = 0;
}

void tb_refill(token_bucket_t *tb, unsigned long now_ms) {
    unsigned long elapsed = now_ms - tb->last_refill;
    double new_tokens = (elapsed * tb->refill_rate) / 1000.0;
    tb->tokens += new_tokens;
    if (tb->tokens > tb->max_tokens) {
        tb->tokens = tb->max_tokens;
    }
    tb->last_refill = now_ms;
}

int tb_try_consume(token_bucket_t *tb, double cost, unsigned long now_ms) {
    tb_refill(tb, now_ms);
    if (tb->tokens >= cost) {
        tb->tokens -= cost;
        return 1;
    }
    return 0;
}

double tb_available(const token_bucket_t *tb) {
    return tb->tokens;
}

int tb_is_full(const token_bucket_t *tb) {
    return tb->tokens >= tb->max_tokens;
}

double tb_wait_time_ms(const token_bucket_t *tb, double cost) {
    if (tb->tokens >= cost) return 0.0;
    double deficit = cost - tb->tokens;
    return (deficit / tb->refill_rate) * 1000.0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C272: Rate limiter token bucket should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C272: Output should not be empty");
    assert!(
        code.contains("fn tb_init"),
        "C272: Should contain tb_init function"
    );
    assert!(
        code.contains("fn tb_try_consume"),
        "C272: Should contain tb_try_consume function"
    );
    assert!(
        code.contains("fn tb_wait_time_ms"),
        "C272: Should contain tb_wait_time_ms function"
    );
}

#[test]
fn c273_consistent_hashing_ring() {
    let c_code = r#"
#define RING_POINTS 256
#define MAX_NODES 32

typedef struct {
    unsigned int hash;
    int node_id;
} ring_point_t;

typedef struct {
    ring_point_t points[256];
    int point_count;
    int node_count;
} hash_ring_t;

static unsigned int fnv1a_hash(unsigned int key) {
    unsigned int hash = 2166136261U;
    for (int i = 0; i < 4; i++) {
        hash ^= (key >> (i * 8)) & 0xFF;
        hash *= 16777619U;
    }
    return hash;
}

void ring_init(hash_ring_t *ring) {
    ring->point_count = 0;
    ring->node_count = 0;
}

static void ring_sort(hash_ring_t *ring) {
    for (int i = 1; i < ring->point_count; i++) {
        ring_point_t key_pt = ring->points[i];
        int j = i - 1;
        while (j >= 0 && ring->points[j].hash > key_pt.hash) {
            ring->points[j + 1] = ring->points[j];
            j--;
        }
        ring->points[j + 1] = key_pt;
    }
}

int ring_add_node(hash_ring_t *ring, int node_id, int replicas) {
    if (ring->point_count + replicas > 256) return -1;
    for (int r = 0; r < replicas; r++) {
        unsigned int h = fnv1a_hash((unsigned int)(node_id * 1000 + r));
        ring->points[ring->point_count].hash = h;
        ring->points[ring->point_count].node_id = node_id;
        ring->point_count++;
    }
    ring->node_count++;
    ring_sort(ring);
    return 0;
}

int ring_lookup(const hash_ring_t *ring, unsigned int key) {
    if (ring->point_count == 0) return -1;
    unsigned int h = fnv1a_hash(key);
    for (int i = 0; i < ring->point_count; i++) {
        if (ring->points[i].hash >= h) {
            return ring->points[i].node_id;
        }
    }
    return ring->points[0].node_id;
}

int ring_node_count(const hash_ring_t *ring) {
    return ring->node_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C273: Consistent hashing ring should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C273: Output should not be empty");
    assert!(
        code.contains("fn ring_init"),
        "C273: Should contain ring_init function"
    );
    assert!(
        code.contains("fn ring_add_node"),
        "C273: Should contain ring_add_node function"
    );
    assert!(
        code.contains("fn ring_lookup"),
        "C273: Should contain ring_lookup function"
    );
}

#[test]
fn c274_write_ahead_log_entry() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define WAL_MAGIC 0x57414C31
#define WAL_MAX_PAYLOAD 512
#define WAL_MAX_ENTRIES 128

typedef struct {
    uint32_t magic;
    uint32_t entry_len;
    uint64_t lsn;
    uint32_t crc;
    uint32_t type;
    unsigned char payload[512];
} wal_entry_t;

typedef struct {
    wal_entry_t entries[128];
    int count;
    uint64_t next_lsn;
} wal_log_t;

void wal_init(wal_log_t *log) {
    log->count = 0;
    log->next_lsn = 1;
}

static uint32_t wal_checksum(const unsigned char *data, int len) {
    uint32_t sum = 0;
    for (int i = 0; i < len; i++) {
        sum = sum * 31 + data[i];
    }
    return sum;
}

int wal_append(wal_log_t *log, uint32_t type, const unsigned char *data, int data_len) {
    if (log->count >= 128) return -1;
    if (data_len > 512) return -2;
    wal_entry_t *entry = &log->entries[log->count];
    entry->magic = WAL_MAGIC;
    entry->entry_len = (uint32_t)data_len;
    entry->lsn = log->next_lsn++;
    entry->type = type;
    for (int i = 0; i < data_len; i++) {
        entry->payload[i] = data[i];
    }
    entry->crc = wal_checksum(data, data_len);
    log->count++;
    return 0;
}

int wal_verify(const wal_entry_t *entry) {
    if (entry->magic != WAL_MAGIC) return -1;
    uint32_t computed = wal_checksum(entry->payload, (int)entry->entry_len);
    if (computed != entry->crc) return -2;
    return 0;
}

uint64_t wal_last_lsn(const wal_log_t *log) {
    if (log->count == 0) return 0;
    return log->entries[log->count - 1].lsn;
}

int wal_find_by_lsn(const wal_log_t *log, uint64_t lsn) {
    for (int i = 0; i < log->count; i++) {
        if (log->entries[i].lsn == lsn) return i;
    }
    return -1;
}

int wal_truncate_after(wal_log_t *log, uint64_t lsn) {
    for (int i = 0; i < log->count; i++) {
        if (log->entries[i].lsn > lsn) {
            log->count = i;
            log->next_lsn = lsn + 1;
            return 0;
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C274: Write-ahead log entry should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C274: Output should not be empty");
    assert!(
        code.contains("fn wal_init"),
        "C274: Should contain wal_init function"
    );
    assert!(
        code.contains("fn wal_append"),
        "C274: Should contain wal_append function"
    );
    assert!(
        code.contains("fn wal_verify"),
        "C274: Should contain wal_verify function"
    );
}

#[test]
fn c275_zero_copy_buffer_chain() {
    let c_code = r#"
#define MBUF_SIZE 2048
#define MAX_MBUFS 64

typedef struct {
    unsigned char data[2048];
    int data_offset;
    int data_len;
    int next;
    int refcount;
} mbuf_t;

typedef struct {
    mbuf_t bufs[64];
    unsigned char free_bitmap[64];
    int free_count;
} mbuf_pool_t;

void mbuf_pool_init(mbuf_pool_t *pool) {
    pool->free_count = 64;
    for (int i = 0; i < 64; i++) {
        pool->free_bitmap[i] = 1;
        pool->bufs[i].data_offset = 0;
        pool->bufs[i].data_len = 0;
        pool->bufs[i].next = -1;
        pool->bufs[i].refcount = 0;
    }
}

int mbuf_alloc(mbuf_pool_t *pool) {
    for (int i = 0; i < 64; i++) {
        if (pool->free_bitmap[i]) {
            pool->free_bitmap[i] = 0;
            pool->bufs[i].data_offset = 0;
            pool->bufs[i].data_len = 0;
            pool->bufs[i].next = -1;
            pool->bufs[i].refcount = 1;
            pool->free_count--;
            return i;
        }
    }
    return -1;
}

void mbuf_free(mbuf_pool_t *pool, int idx) {
    if (idx < 0 || idx >= 64) return;
    pool->bufs[idx].refcount--;
    if (pool->bufs[idx].refcount <= 0) {
        pool->free_bitmap[idx] = 1;
        pool->bufs[idx].next = -1;
        pool->free_count++;
    }
}

void mbuf_chain_free(mbuf_pool_t *pool, int head) {
    while (head != -1) {
        int next = pool->bufs[head].next;
        mbuf_free(pool, head);
        head = next;
    }
}

int mbuf_chain_append(mbuf_pool_t *pool, int chain_head, int new_mbuf) {
    if (chain_head == -1) return new_mbuf;
    int current = chain_head;
    while (pool->bufs[current].next != -1) {
        current = pool->bufs[current].next;
    }
    pool->bufs[current].next = new_mbuf;
    return chain_head;
}

int mbuf_chain_total_len(const mbuf_pool_t *pool, int head) {
    int total = 0;
    while (head != -1) {
        total += pool->bufs[head].data_len;
        head = pool->bufs[head].next;
    }
    return total;
}

int mbuf_chain_count(const mbuf_pool_t *pool, int head) {
    int count = 0;
    while (head != -1) {
        count++;
        head = pool->bufs[head].next;
    }
    return count;
}

int mbuf_write(mbuf_pool_t *pool, int idx, const unsigned char *data, int len) {
    if (idx < 0 || idx >= 64) return -1;
    if (len > 2048) return -2;
    for (int i = 0; i < len; i++) {
        pool->bufs[idx].data[i] = data[i];
    }
    pool->bufs[idx].data_len = len;
    pool->bufs[idx].data_offset = 0;
    return len;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C275: Zero-copy buffer chain should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C275: Output should not be empty");
    assert!(
        code.contains("fn mbuf_pool_init"),
        "C275: Should contain mbuf_pool_init function"
    );
    assert!(
        code.contains("fn mbuf_alloc"),
        "C275: Should contain mbuf_alloc function"
    );
    assert!(
        code.contains("fn mbuf_chain_free"),
        "C275: Should contain mbuf_chain_free function"
    );
    assert!(
        code.contains("fn mbuf_chain_total_len"),
        "C275: Should contain mbuf_chain_total_len function"
    );
}
