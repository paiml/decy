//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C476-C500: OS Kernel and System Programming C patterns -- process
//! management, virtual memory, file systems, device drivers, schedulers,
//! and kernel infrastructure code.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world OS kernel patterns commonly found in
//! Linux, xv6, MINIX, and similar operating system kernels -- all
//! expressed as valid C99.
//!
//! Organization:
//! - C476-C480: Process management (PCB, page fault, FD table, pipe, scheduler)
//! - C481-C485: Kernel subsystems (syscall dispatch, driver queue, FS, buffer cache, TTY)
//! - C486-C490: Kernel infrastructure (skbuff, buddy alloc, signals, mount, printk)
//! - C491-C495: Hardware and memory (interrupt controller, TLS, COW, modules, elevator)
//! - C496-C500: Networking and boot (packet filter, timekeeping, RNG, kref, ELF loader)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C476-C480: Process Management
// ============================================================================

#[test]
fn c476_process_control_block_with_context_switch() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

#define MAX_PROCS 256
#define PROC_RUNNING 1
#define PROC_READY   2
#define PROC_BLOCKED 3
#define PROC_ZOMBIE  4

struct cpu_context {
    uint32_t eax;
    uint32_t ebx;
    uint32_t ecx;
    uint32_t edx;
    uint32_t esi;
    uint32_t edi;
    uint32_t ebp;
    uint32_t esp;
    uint32_t eip;
    uint32_t eflags;
};

struct pcb {
    int pid;
    int ppid;
    int state;
    int priority;
    int time_slice;
    int exit_code;
    struct cpu_context ctx;
    uint32_t page_dir;
    int open_fds[16];
    int fd_count;
};

static struct pcb proc_table[256];
static int next_pid = 1;
static int current_pid = 0;

int pcb_alloc(int ppid, int priority) {
    if (next_pid >= MAX_PROCS) return -1;
    int pid = next_pid++;
    proc_table[pid].pid = pid;
    proc_table[pid].ppid = ppid;
    proc_table[pid].state = PROC_READY;
    proc_table[pid].priority = priority;
    proc_table[pid].time_slice = 10;
    proc_table[pid].exit_code = 0;
    proc_table[pid].fd_count = 0;
    proc_table[pid].page_dir = 0;
    return pid;
}

void pcb_save_context(int pid, const struct cpu_context *ctx) {
    proc_table[pid].ctx = *ctx;
}

void pcb_set_state(int pid, int state) {
    proc_table[pid].state = state;
}

int pcb_find_ready(void) {
    int best = -1;
    int best_prio = -1;
    for (int i = 1; i < next_pid; i++) {
        if (proc_table[i].state == PROC_READY && proc_table[i].priority > best_prio) {
            best = i;
            best_prio = proc_table[i].priority;
        }
    }
    return best;
}

void pcb_exit(int pid, int code) {
    proc_table[pid].state = PROC_ZOMBIE;
    proc_table[pid].exit_code = code;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C476: Process control block - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C476: empty output");
    assert!(code.contains("fn pcb_alloc"), "C476: Should contain pcb_alloc");
    assert!(code.contains("fn pcb_find_ready"), "C476: Should contain pcb_find_ready");
}

#[test]
fn c477_virtual_memory_page_fault_handler() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define PAGE_SIZE 4096
#define PAGE_PRESENT  0x01
#define PAGE_WRITE    0x02
#define PAGE_USER     0x04
#define PT_ENTRIES    1024

struct page_table_entry {
    uint32_t frame;
    uint32_t flags;
};

struct page_directory {
    struct page_table_entry entries[1024];
    int mapped_count;
};

static uint32_t next_frame = 0x100000;

uint32_t alloc_frame(void) {
    uint32_t f = next_frame;
    next_frame += PAGE_SIZE;
    return f;
}

int pte_is_present(const struct page_table_entry *pte) {
    return (pte->flags & PAGE_PRESENT) != 0;
}

void pte_set(struct page_table_entry *pte, uint32_t frame, uint32_t flags) {
    pte->frame = frame;
    pte->flags = flags | PAGE_PRESENT;
}

int page_directory_lookup(struct page_directory *pd, uint32_t vaddr) {
    uint32_t idx = (vaddr / PAGE_SIZE) % PT_ENTRIES;
    return pte_is_present(&pd->entries[idx]);
}

int handle_page_fault(struct page_directory *pd, uint32_t fault_addr) {
    uint32_t idx = (fault_addr / PAGE_SIZE) % PT_ENTRIES;
    if (pte_is_present(&pd->entries[idx])) {
        return -1;
    }
    uint32_t frame = alloc_frame();
    pte_set(&pd->entries[idx], frame, PAGE_WRITE | PAGE_USER);
    pd->mapped_count++;
    return 0;
}

int count_mapped_pages(const struct page_directory *pd) {
    int count = 0;
    for (int i = 0; i < PT_ENTRIES; i++) {
        if (pte_is_present(&pd->entries[i])) {
            count++;
        }
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C477: Page fault handler - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C477: empty output");
    assert!(code.contains("fn handle_page_fault"), "C477: Should contain handle_page_fault");
    assert!(code.contains("fn alloc_frame"), "C477: Should contain alloc_frame");
}

#[test]
fn c478_file_descriptor_table() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_FDS 64
#define FD_FREE   0
#define FD_FILE   1
#define FD_PIPE   2
#define FD_SOCKET 3

struct fd_entry {
    int type;
    int inode;
    int offset;
    int flags;
    int ref_count;
};

struct fd_table {
    struct fd_entry fds[64];
    int count;
};

void fd_table_init(struct fd_table *ft) {
    for (int i = 0; i < MAX_FDS; i++) {
        ft->fds[i].type = FD_FREE;
        ft->fds[i].ref_count = 0;
    }
    ft->count = 0;
}

int fd_alloc(struct fd_table *ft, int type, int inode) {
    for (int i = 0; i < MAX_FDS; i++) {
        if (ft->fds[i].type == FD_FREE) {
            ft->fds[i].type = type;
            ft->fds[i].inode = inode;
            ft->fds[i].offset = 0;
            ft->fds[i].flags = 0;
            ft->fds[i].ref_count = 1;
            ft->count++;
            return i;
        }
    }
    return -1;
}

int fd_close(struct fd_table *ft, int fd) {
    if (fd < 0 || fd >= MAX_FDS) return -1;
    if (ft->fds[fd].type == FD_FREE) return -1;
    ft->fds[fd].ref_count--;
    if (ft->fds[fd].ref_count <= 0) {
        ft->fds[fd].type = FD_FREE;
        ft->count--;
    }
    return 0;
}

int fd_dup(struct fd_table *ft, int oldfd) {
    if (oldfd < 0 || oldfd >= MAX_FDS) return -1;
    if (ft->fds[oldfd].type == FD_FREE) return -1;
    int newfd = fd_alloc(ft, ft->fds[oldfd].type, ft->fds[oldfd].inode);
    if (newfd >= 0) {
        ft->fds[newfd].offset = ft->fds[oldfd].offset;
        ft->fds[newfd].flags = ft->fds[oldfd].flags;
    }
    return newfd;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C478: File descriptor table - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C478: empty output");
    assert!(code.contains("fn fd_alloc"), "C478: Should contain fd_alloc");
    assert!(code.contains("fn fd_close"), "C478: Should contain fd_close");
    assert!(code.contains("fn fd_dup"), "C478: Should contain fd_dup");
}

#[test]
fn c479_pipe_buffer_reader_writer() {
    let c_code = r#"
typedef unsigned char uint8_t;

#define PIPE_BUF_SIZE 512

struct pipe_buf {
    uint8_t data[512];
    int read_pos;
    int write_pos;
    int count;
    int readers;
    int writers;
    int closed;
};

void pipe_init(struct pipe_buf *p) {
    p->read_pos = 0;
    p->write_pos = 0;
    p->count = 0;
    p->readers = 1;
    p->writers = 1;
    p->closed = 0;
}

int pipe_is_full(const struct pipe_buf *p) {
    return p->count >= PIPE_BUF_SIZE;
}

int pipe_is_empty(const struct pipe_buf *p) {
    return p->count == 0;
}

int pipe_write(struct pipe_buf *p, uint8_t byte) {
    if (p->closed || p->readers <= 0) return -1;
    if (pipe_is_full(p)) return 0;
    p->data[p->write_pos] = byte;
    p->write_pos = (p->write_pos + 1) % PIPE_BUF_SIZE;
    p->count++;
    return 1;
}

int pipe_read(struct pipe_buf *p, uint8_t *out) {
    if (pipe_is_empty(p)) {
        if (p->writers <= 0) return -1;
        return 0;
    }
    *out = p->data[p->read_pos];
    p->read_pos = (p->read_pos + 1) % PIPE_BUF_SIZE;
    p->count--;
    return 1;
}

void pipe_close_writer(struct pipe_buf *p) {
    p->writers--;
    if (p->writers <= 0 && p->readers <= 0) {
        p->closed = 1;
    }
}

void pipe_close_reader(struct pipe_buf *p) {
    p->readers--;
    if (p->writers <= 0 && p->readers <= 0) {
        p->closed = 1;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C479: Pipe buffer - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C479: empty output");
    assert!(code.contains("fn pipe_write"), "C479: Should contain pipe_write");
    assert!(code.contains("fn pipe_read"), "C479: Should contain pipe_read");
}

#[test]
fn c480_round_robin_process_scheduler() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_TASKS 32
#define QUANTUM 10
#define TASK_IDLE    0
#define TASK_READY   1
#define TASK_RUNNING 2
#define TASK_BLOCKED 3

struct task {
    int tid;
    int state;
    int remaining;
    int total_time;
    int priority;
};

struct scheduler {
    struct task tasks[32];
    int task_count;
    int current;
    int ticks;
};

void sched_init(struct scheduler *s) {
    s->task_count = 0;
    s->current = -1;
    s->ticks = 0;
}

int sched_add_task(struct scheduler *s, int priority) {
    if (s->task_count >= MAX_TASKS) return -1;
    int tid = s->task_count;
    s->tasks[tid].tid = tid;
    s->tasks[tid].state = TASK_READY;
    s->tasks[tid].remaining = QUANTUM;
    s->tasks[tid].total_time = 0;
    s->tasks[tid].priority = priority;
    s->task_count++;
    return tid;
}

int sched_pick_next(struct scheduler *s) {
    int start = (s->current + 1) % s->task_count;
    int i = start;
    do {
        if (s->tasks[i].state == TASK_READY) {
            return i;
        }
        i = (i + 1) % s->task_count;
    } while (i != start);
    return -1;
}

void sched_tick(struct scheduler *s) {
    s->ticks++;
    if (s->current < 0) return;
    s->tasks[s->current].remaining--;
    s->tasks[s->current].total_time++;
    if (s->tasks[s->current].remaining <= 0) {
        s->tasks[s->current].state = TASK_READY;
        s->tasks[s->current].remaining = QUANTUM;
        int next = sched_pick_next(s);
        if (next >= 0) {
            s->current = next;
            s->tasks[next].state = TASK_RUNNING;
        }
    }
}

void sched_block(struct scheduler *s, int tid) {
    s->tasks[tid].state = TASK_BLOCKED;
    if (tid == s->current) {
        int next = sched_pick_next(s);
        if (next >= 0) {
            s->current = next;
            s->tasks[next].state = TASK_RUNNING;
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C480: Round-robin scheduler - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C480: empty output");
    assert!(code.contains("fn sched_tick"), "C480: Should contain sched_tick");
    assert!(code.contains("fn sched_pick_next"), "C480: Should contain sched_pick_next");
}

// ============================================================================
// C481-C485: Kernel Subsystems
// ============================================================================

#[test]
fn c481_system_call_dispatch_table() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define SYS_READ   0
#define SYS_WRITE  1
#define SYS_OPEN   2
#define SYS_CLOSE  3
#define SYS_FORK   4
#define SYS_EXIT   5
#define SYS_WAIT   6
#define SYS_EXEC   7
#define NR_SYSCALLS 8

struct syscall_args {
    uint32_t arg0;
    uint32_t arg1;
    uint32_t arg2;
    uint32_t arg3;
};

static int syscall_counts[8];

int sys_read_handler(struct syscall_args *args) {
    syscall_counts[SYS_READ]++;
    return (int)args->arg2;
}

int sys_write_handler(struct syscall_args *args) {
    syscall_counts[SYS_WRITE]++;
    return (int)args->arg2;
}

int sys_open_handler(struct syscall_args *args) {
    syscall_counts[SYS_OPEN]++;
    return 3;
}

int sys_close_handler(struct syscall_args *args) {
    syscall_counts[SYS_CLOSE]++;
    return 0;
}

int syscall_dispatch(int nr, struct syscall_args *args) {
    if (nr < 0 || nr >= NR_SYSCALLS) return -1;
    switch (nr) {
        case SYS_READ:  return sys_read_handler(args);
        case SYS_WRITE: return sys_write_handler(args);
        case SYS_OPEN:  return sys_open_handler(args);
        case SYS_CLOSE: return sys_close_handler(args);
        default: return -1;
    }
}

int get_syscall_count(int nr) {
    if (nr < 0 || nr >= NR_SYSCALLS) return -1;
    return syscall_counts[nr];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C481: Syscall dispatch - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C481: empty output");
    assert!(code.contains("fn syscall_dispatch"), "C481: Should contain syscall_dispatch");
}

#[test]
fn c482_device_driver_request_queue() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define MAX_REQUESTS 128
#define REQ_READ  1
#define REQ_WRITE 2
#define REQ_IOCTL 3
#define REQ_PENDING  0
#define REQ_ACTIVE   1
#define REQ_DONE     2
#define REQ_ERROR    3

struct dev_request {
    int type;
    int status;
    uint64_t sector;
    uint32_t count;
    int priority;
    int result;
};

struct request_queue {
    struct dev_request reqs[128];
    int head;
    int tail;
    int count;
    int active;
};

void rq_init(struct request_queue *rq) {
    rq->head = 0;
    rq->tail = 0;
    rq->count = 0;
    rq->active = -1;
}

int rq_submit(struct request_queue *rq, int type, uint64_t sector, uint32_t cnt) {
    if (rq->count >= MAX_REQUESTS) return -1;
    int idx = rq->tail;
    rq->reqs[idx].type = type;
    rq->reqs[idx].status = REQ_PENDING;
    rq->reqs[idx].sector = sector;
    rq->reqs[idx].count = cnt;
    rq->reqs[idx].priority = 0;
    rq->reqs[idx].result = 0;
    rq->tail = (rq->tail + 1) % MAX_REQUESTS;
    rq->count++;
    return idx;
}

int rq_dequeue(struct request_queue *rq) {
    if (rq->count == 0) return -1;
    int idx = rq->head;
    rq->reqs[idx].status = REQ_ACTIVE;
    rq->active = idx;
    rq->head = (rq->head + 1) % MAX_REQUESTS;
    rq->count--;
    return idx;
}

void rq_complete(struct request_queue *rq, int idx, int result) {
    rq->reqs[idx].status = REQ_DONE;
    rq->reqs[idx].result = result;
    if (rq->active == idx) {
        rq->active = -1;
    }
}

int rq_pending_count(const struct request_queue *rq) {
    return rq->count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C482: Device driver request queue - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C482: empty output");
    assert!(code.contains("fn rq_submit"), "C482: Should contain rq_submit");
    assert!(code.contains("fn rq_dequeue"), "C482: Should contain rq_dequeue");
}

#[test]
fn c483_filesystem_superblock_and_inode() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;

#define MAX_INODES 1024
#define BLOCKS_PER_INODE 12
#define INODE_FREE  0
#define INODE_FILE  1
#define INODE_DIR   2

struct inode {
    uint32_t ino;
    uint16_t type;
    uint16_t permissions;
    uint32_t size;
    uint32_t blocks[12];
    uint32_t block_count;
    uint32_t link_count;
    uint32_t uid;
    uint32_t gid;
};

struct superblock {
    uint32_t magic;
    uint32_t total_blocks;
    uint32_t free_blocks;
    uint32_t total_inodes;
    uint32_t free_inodes;
    uint32_t block_size;
    struct inode inodes[1024];
};

void sb_init(struct superblock *sb, uint32_t total_blocks) {
    sb->magic = 0xDECF5000;
    sb->total_blocks = total_blocks;
    sb->free_blocks = total_blocks;
    sb->total_inodes = MAX_INODES;
    sb->free_inodes = MAX_INODES;
    sb->block_size = 4096;
    for (int i = 0; i < MAX_INODES; i++) {
        sb->inodes[i].ino = (uint32_t)i;
        sb->inodes[i].type = INODE_FREE;
        sb->inodes[i].size = 0;
        sb->inodes[i].block_count = 0;
        sb->inodes[i].link_count = 0;
    }
}

int sb_alloc_inode(struct superblock *sb, uint16_t type) {
    for (int i = 1; i < MAX_INODES; i++) {
        if (sb->inodes[i].type == INODE_FREE) {
            sb->inodes[i].type = type;
            sb->inodes[i].link_count = 1;
            sb->free_inodes--;
            return i;
        }
    }
    return -1;
}

int sb_free_inode(struct superblock *sb, uint32_t ino) {
    if (ino == 0 || ino >= MAX_INODES) return -1;
    if (sb->inodes[ino].type == INODE_FREE) return -1;
    sb->inodes[ino].type = INODE_FREE;
    sb->inodes[ino].size = 0;
    sb->inodes[ino].block_count = 0;
    sb->free_inodes++;
    return 0;
}

uint32_t sb_inode_size(const struct superblock *sb, uint32_t ino) {
    if (ino >= MAX_INODES) return 0;
    return sb->inodes[ino].size;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C483: Filesystem superblock - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C483: empty output");
    assert!(code.contains("fn sb_init"), "C483: Should contain sb_init");
    assert!(code.contains("fn sb_alloc_inode"), "C483: Should contain sb_alloc_inode");
}

#[test]
fn c484_block_device_buffer_cache() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define CACHE_SIZE 64
#define BLOCK_SIZE 512
#define BUF_CLEAN 0
#define BUF_DIRTY 1
#define BUF_EMPTY 2

struct buf_entry {
    uint32_t block_no;
    int state;
    int ref_count;
    int lru_time;
    uint8_t data[512];
};

struct buffer_cache {
    struct buf_entry bufs[64];
    int clock;
};

void bcache_init(struct buffer_cache *bc) {
    bc->clock = 0;
    for (int i = 0; i < CACHE_SIZE; i++) {
        bc->bufs[i].block_no = 0;
        bc->bufs[i].state = BUF_EMPTY;
        bc->bufs[i].ref_count = 0;
        bc->bufs[i].lru_time = 0;
    }
}

int bcache_find(struct buffer_cache *bc, uint32_t block_no) {
    for (int i = 0; i < CACHE_SIZE; i++) {
        if (bc->bufs[i].state != BUF_EMPTY && bc->bufs[i].block_no == block_no) {
            bc->bufs[i].ref_count++;
            bc->bufs[i].lru_time = bc->clock++;
            return i;
        }
    }
    return -1;
}

int bcache_evict(struct buffer_cache *bc) {
    int oldest = -1;
    int oldest_time = 0x7FFFFFFF;
    for (int i = 0; i < CACHE_SIZE; i++) {
        if (bc->bufs[i].ref_count == 0 && bc->bufs[i].lru_time < oldest_time) {
            oldest = i;
            oldest_time = bc->bufs[i].lru_time;
        }
    }
    if (oldest >= 0 && bc->bufs[oldest].state == BUF_DIRTY) {
        bc->bufs[oldest].state = BUF_CLEAN;
    }
    return oldest;
}

int bcache_get(struct buffer_cache *bc, uint32_t block_no) {
    int idx = bcache_find(bc, block_no);
    if (idx >= 0) return idx;
    idx = bcache_evict(bc);
    if (idx < 0) return -1;
    bc->bufs[idx].block_no = block_no;
    bc->bufs[idx].state = BUF_CLEAN;
    bc->bufs[idx].ref_count = 1;
    bc->bufs[idx].lru_time = bc->clock++;
    return idx;
}

void bcache_mark_dirty(struct buffer_cache *bc, int idx) {
    if (idx >= 0 && idx < CACHE_SIZE) {
        bc->bufs[idx].state = BUF_DIRTY;
    }
}

void bcache_release(struct buffer_cache *bc, int idx) {
    if (idx >= 0 && idx < CACHE_SIZE && bc->bufs[idx].ref_count > 0) {
        bc->bufs[idx].ref_count--;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C484: Buffer cache - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C484: empty output");
    assert!(code.contains("fn bcache_get"), "C484: Should contain bcache_get");
    assert!(code.contains("fn bcache_evict"), "C484: Should contain bcache_evict");
}

#[test]
fn c485_terminal_line_discipline() {
    let c_code = r#"
typedef unsigned char uint8_t;

#define LINE_BUF_SIZE 256
#define CHAR_BS    0x08
#define CHAR_DEL   0x7F
#define CHAR_CR    0x0D
#define CHAR_LF    0x0A
#define CHAR_EOT   0x04
#define CHAR_KILL  0x15

struct line_disc {
    uint8_t buf[256];
    int pos;
    int len;
    int canonical;
    int echo;
    int complete;
};

void ld_init(struct line_disc *ld, int canonical, int echo) {
    ld->pos = 0;
    ld->len = 0;
    ld->canonical = canonical;
    ld->echo = echo;
    ld->complete = 0;
}

void ld_reset(struct line_disc *ld) {
    ld->pos = 0;
    ld->len = 0;
    ld->complete = 0;
}

int ld_input_char(struct line_disc *ld, uint8_t ch) {
    if (!ld->canonical) {
        if (ld->len < LINE_BUF_SIZE) {
            ld->buf[ld->len++] = ch;
            return 1;
        }
        return 0;
    }
    if (ch == CHAR_BS || ch == CHAR_DEL) {
        if (ld->len > 0) {
            ld->len--;
            return 1;
        }
        return 0;
    }
    if (ch == CHAR_KILL) {
        ld->len = 0;
        return 1;
    }
    if (ch == CHAR_CR || ch == CHAR_LF) {
        ld->complete = 1;
        return 1;
    }
    if (ch == CHAR_EOT) {
        ld->complete = 1;
        return -1;
    }
    if (ld->len < LINE_BUF_SIZE - 1) {
        ld->buf[ld->len++] = ch;
        return 1;
    }
    return 0;
}

int ld_is_complete(const struct line_disc *ld) {
    return ld->complete;
}

int ld_get_line_length(const struct line_disc *ld) {
    return ld->len;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C485: Terminal line discipline - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C485: empty output");
    assert!(code.contains("fn ld_input_char"), "C485: Should contain ld_input_char");
    assert!(code.contains("fn ld_is_complete"), "C485: Should contain ld_is_complete");
}

// ============================================================================
// C486-C490: Kernel Infrastructure
// ============================================================================

#[test]
fn c486_network_socket_buffer_chain() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

#define SKB_POOL_SIZE 64
#define SKB_DATA_SIZE 1500
#define SKB_FREE  0
#define SKB_ALLOC 1

struct sk_buff {
    uint8_t data[1500];
    uint16_t len;
    uint16_t protocol;
    int next;
    int state;
};

static struct sk_buff skb_pool[64];
static int skb_free_head = 0;

void skb_pool_init(void) {
    for (int i = 0; i < SKB_POOL_SIZE - 1; i++) {
        skb_pool[i].state = SKB_FREE;
        skb_pool[i].next = i + 1;
        skb_pool[i].len = 0;
    }
    skb_pool[SKB_POOL_SIZE - 1].state = SKB_FREE;
    skb_pool[SKB_POOL_SIZE - 1].next = -1;
    skb_free_head = 0;
}

int skb_alloc(void) {
    if (skb_free_head < 0) return -1;
    int idx = skb_free_head;
    skb_free_head = skb_pool[idx].next;
    skb_pool[idx].state = SKB_ALLOC;
    skb_pool[idx].next = -1;
    skb_pool[idx].len = 0;
    return idx;
}

void skb_free(int idx) {
    if (idx < 0 || idx >= SKB_POOL_SIZE) return;
    skb_pool[idx].state = SKB_FREE;
    skb_pool[idx].next = skb_free_head;
    skb_free_head = idx;
}

int skb_set_data(int idx, const uint8_t *src, uint16_t len) {
    if (idx < 0 || idx >= SKB_POOL_SIZE) return -1;
    if (len > SKB_DATA_SIZE) return -1;
    for (int i = 0; i < len; i++) {
        skb_pool[idx].data[i] = src[i];
    }
    skb_pool[idx].len = len;
    return 0;
}

uint16_t skb_get_len(int idx) {
    if (idx < 0 || idx >= SKB_POOL_SIZE) return 0;
    return skb_pool[idx].len;
}

int skb_chain(int head, int tail) {
    if (head < 0 || tail < 0) return -1;
    skb_pool[head].next = tail;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C486: Socket buffer chain - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C486: empty output");
    assert!(code.contains("fn skb_alloc"), "C486: Should contain skb_alloc");
    assert!(code.contains("fn skb_free"), "C486: Should contain skb_free");
}

#[test]
fn c487_kernel_buddy_allocator() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_ORDER 10
#define BUDDY_FREE 0
#define BUDDY_USED 1
#define POOL_SIZE 1024

struct buddy_block {
    int order;
    int state;
    int next_free;
};

struct buddy_alloc {
    struct buddy_block blocks[1024];
    int free_lists[11];
    int total_free;
};

void buddy_init(struct buddy_alloc *ba) {
    ba->total_free = POOL_SIZE;
    for (int i = 0; i <= MAX_ORDER; i++) {
        ba->free_lists[i] = -1;
    }
    for (int i = 0; i < POOL_SIZE; i++) {
        ba->blocks[i].order = 0;
        ba->blocks[i].state = BUDDY_FREE;
        ba->blocks[i].next_free = -1;
    }
    ba->blocks[0].order = MAX_ORDER;
    ba->blocks[0].next_free = -1;
    ba->free_lists[MAX_ORDER] = 0;
}

int buddy_find_block(struct buddy_alloc *ba, int order) {
    int o = order;
    while (o <= MAX_ORDER) {
        if (ba->free_lists[o] >= 0) {
            return o;
        }
        o++;
    }
    return -1;
}

int buddy_split(struct buddy_alloc *ba, int idx, int from_order, int to_order) {
    int current_order = from_order;
    while (current_order > to_order) {
        current_order--;
        int buddy_size = 1 << current_order;
        int buddy_idx = idx + buddy_size;
        if (buddy_idx < POOL_SIZE) {
            ba->blocks[buddy_idx].order = current_order;
            ba->blocks[buddy_idx].state = BUDDY_FREE;
            ba->blocks[buddy_idx].next_free = ba->free_lists[current_order];
            ba->free_lists[current_order] = buddy_idx;
        }
    }
    ba->blocks[idx].order = to_order;
    ba->blocks[idx].state = BUDDY_USED;
    return idx;
}

int buddy_alloc_order(struct buddy_alloc *ba, int order) {
    int found = buddy_find_block(ba, order);
    if (found < 0) return -1;
    int idx = ba->free_lists[found];
    ba->free_lists[found] = ba->blocks[idx].next_free;
    int result = buddy_split(ba, idx, found, order);
    ba->total_free -= (1 << order);
    return result;
}

void buddy_free_block(struct buddy_alloc *ba, int idx) {
    if (idx < 0 || idx >= POOL_SIZE) return;
    ba->blocks[idx].state = BUDDY_FREE;
    int order = ba->blocks[idx].order;
    ba->blocks[idx].next_free = ba->free_lists[order];
    ba->free_lists[order] = idx;
    ba->total_free += (1 << order);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C487: Buddy allocator - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C487: empty output");
    assert!(code.contains("fn buddy_alloc_order"), "C487: Should contain buddy_alloc_order");
    assert!(code.contains("fn buddy_free_block"), "C487: Should contain buddy_free_block");
}

#[test]
fn c488_process_signal_delivery_queue() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_SIGNALS 32
#define SIG_PENDING 0
#define SIG_HANDLED 1
#define SIG_IGNORED 2

#define SIGTERM  15
#define SIGKILL  9
#define SIGUSR1  10
#define SIGUSR2  12
#define SIGCHLD  17

struct signal_entry {
    int signo;
    int state;
    int sender_pid;
    uint32_t timestamp;
};

struct signal_queue {
    struct signal_entry queue[32];
    int count;
    uint32_t mask;
    uint32_t pending_set;
};

void sigq_init(struct signal_queue *sq) {
    sq->count = 0;
    sq->mask = 0;
    sq->pending_set = 0;
}

int sigq_send(struct signal_queue *sq, int signo, int sender) {
    if (sq->count >= MAX_SIGNALS) return -1;
    if (signo < 1 || signo > 31) return -1;
    if ((sq->mask & (1U << signo)) && signo != SIGKILL) {
        return 0;
    }
    sq->queue[sq->count].signo = signo;
    sq->queue[sq->count].state = SIG_PENDING;
    sq->queue[sq->count].sender_pid = sender;
    sq->queue[sq->count].timestamp = 0;
    sq->count++;
    sq->pending_set |= (1U << signo);
    return 1;
}

int sigq_dequeue(struct signal_queue *sq) {
    for (int i = 0; i < sq->count; i++) {
        if (sq->queue[i].state == SIG_PENDING) {
            sq->queue[i].state = SIG_HANDLED;
            return sq->queue[i].signo;
        }
    }
    return 0;
}

void sigq_mask(struct signal_queue *sq, int signo) {
    if (signo != SIGKILL) {
        sq->mask |= (1U << signo);
    }
}

void sigq_unmask(struct signal_queue *sq, int signo) {
    sq->mask &= ~(1U << signo);
}

int sigq_is_pending(const struct signal_queue *sq, int signo) {
    return (sq->pending_set & (1U << signo)) != 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C488: Signal delivery queue - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C488: empty output");
    assert!(code.contains("fn sigq_send"), "C488: Should contain sigq_send");
    assert!(code.contains("fn sigq_dequeue"), "C488: Should contain sigq_dequeue");
}

#[test]
fn c489_filesystem_mount_table() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_MOUNTS 16
#define FS_EXT4  1
#define FS_TMPFS 2
#define FS_PROC  3
#define FS_DEVFS 4
#define MNT_RDONLY   0x01
#define MNT_NOSUID   0x02
#define MNT_NOEXEC   0x04

struct mount_entry {
    int fs_type;
    uint32_t flags;
    int device_id;
    int root_inode;
    int active;
    int ref_count;
};

struct mount_table {
    struct mount_entry mounts[16];
    int count;
};

void mt_init(struct mount_table *mt) {
    mt->count = 0;
    for (int i = 0; i < MAX_MOUNTS; i++) {
        mt->mounts[i].active = 0;
        mt->mounts[i].ref_count = 0;
    }
}

int mt_mount(struct mount_table *mt, int fs_type, int dev_id, uint32_t flags) {
    if (mt->count >= MAX_MOUNTS) return -1;
    for (int i = 0; i < MAX_MOUNTS; i++) {
        if (!mt->mounts[i].active) {
            mt->mounts[i].fs_type = fs_type;
            mt->mounts[i].device_id = dev_id;
            mt->mounts[i].flags = flags;
            mt->mounts[i].root_inode = 2;
            mt->mounts[i].active = 1;
            mt->mounts[i].ref_count = 1;
            mt->count++;
            return i;
        }
    }
    return -1;
}

int mt_unmount(struct mount_table *mt, int idx) {
    if (idx < 0 || idx >= MAX_MOUNTS) return -1;
    if (!mt->mounts[idx].active) return -1;
    if (mt->mounts[idx].ref_count > 1) return -1;
    mt->mounts[idx].active = 0;
    mt->count--;
    return 0;
}

int mt_is_readonly(const struct mount_table *mt, int idx) {
    if (idx < 0 || idx >= MAX_MOUNTS) return 1;
    return (mt->mounts[idx].flags & MNT_RDONLY) != 0;
}

int mt_find_by_device(const struct mount_table *mt, int dev_id) {
    for (int i = 0; i < MAX_MOUNTS; i++) {
        if (mt->mounts[i].active && mt->mounts[i].device_id == dev_id) {
            return i;
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C489: Mount table - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C489: empty output");
    assert!(code.contains("fn mt_mount"), "C489: Should contain mt_mount");
    assert!(code.contains("fn mt_unmount"), "C489: Should contain mt_unmount");
}

#[test]
fn c490_kernel_log_ring_buffer() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define LOG_BUF_SIZE 4096
#define LOG_EMERG  0
#define LOG_ERR    3
#define LOG_WARN   4
#define LOG_INFO   6
#define LOG_DEBUG  7
#define MSG_MAX_LEN 128

struct log_entry {
    uint32_t seq;
    uint8_t level;
    uint8_t len;
    char msg[128];
};

struct log_ring {
    struct log_entry entries[32];
    int head;
    int tail;
    int count;
    uint32_t next_seq;
    int max_entries;
};

void logr_init(struct log_ring *lr) {
    lr->head = 0;
    lr->tail = 0;
    lr->count = 0;
    lr->next_seq = 1;
    lr->max_entries = 32;
}

int logr_write(struct log_ring *lr, uint8_t level, const char *msg, int len) {
    if (len > MSG_MAX_LEN) len = MSG_MAX_LEN;
    int idx = lr->tail;
    lr->entries[idx].seq = lr->next_seq++;
    lr->entries[idx].level = level;
    lr->entries[idx].len = (uint8_t)len;
    for (int i = 0; i < len; i++) {
        lr->entries[idx].msg[i] = msg[i];
    }
    lr->tail = (lr->tail + 1) % lr->max_entries;
    if (lr->count < lr->max_entries) {
        lr->count++;
    } else {
        lr->head = (lr->head + 1) % lr->max_entries;
    }
    return idx;
}

int logr_read(struct log_ring *lr, uint32_t from_seq, struct log_entry *out) {
    for (int i = 0; i < lr->count; i++) {
        int idx = (lr->head + i) % lr->max_entries;
        if (lr->entries[idx].seq >= from_seq) {
            *out = lr->entries[idx];
            return 1;
        }
    }
    return 0;
}

int logr_count_by_level(const struct log_ring *lr, uint8_t level) {
    int c = 0;
    for (int i = 0; i < lr->count; i++) {
        int idx = (lr->head + i) % lr->max_entries;
        if (lr->entries[idx].level == level) {
            c++;
        }
    }
    return c;
}

void logr_clear(struct log_ring *lr) {
    lr->head = 0;
    lr->tail = 0;
    lr->count = 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C490: Kernel log ring buffer - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C490: empty output");
    assert!(code.contains("fn logr_write"), "C490: Should contain logr_write");
    assert!(code.contains("fn logr_read"), "C490: Should contain logr_read");
}

// ============================================================================
// C491-C495: Hardware and Memory Management
// ============================================================================

#[test]
fn c491_interrupt_controller_simulation() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

#define MAX_IRQS 16
#define IRQ_DISABLED 0
#define IRQ_ENABLED  1
#define IRQ_PENDING  2

struct irq_desc {
    int state;
    int priority;
    uint32_t count;
    int masked;
};

struct irq_controller {
    struct irq_desc irqs[16];
    uint16_t mask_reg;
    uint16_t pending_reg;
    uint16_t in_service;
    int nested_depth;
};

void irqc_init(struct irq_controller *ic) {
    ic->mask_reg = 0xFFFF;
    ic->pending_reg = 0;
    ic->in_service = 0;
    ic->nested_depth = 0;
    for (int i = 0; i < MAX_IRQS; i++) {
        ic->irqs[i].state = IRQ_DISABLED;
        ic->irqs[i].priority = i;
        ic->irqs[i].count = 0;
        ic->irqs[i].masked = 1;
    }
}

void irqc_enable(struct irq_controller *ic, int irq) {
    if (irq < 0 || irq >= MAX_IRQS) return;
    ic->irqs[irq].state = IRQ_ENABLED;
    ic->irqs[irq].masked = 0;
    ic->mask_reg &= ~(1U << irq);
}

void irqc_disable(struct irq_controller *ic, int irq) {
    if (irq < 0 || irq >= MAX_IRQS) return;
    ic->irqs[irq].state = IRQ_DISABLED;
    ic->irqs[irq].masked = 1;
    ic->mask_reg |= (1U << irq);
}

void irqc_raise(struct irq_controller *ic, int irq) {
    if (irq < 0 || irq >= MAX_IRQS) return;
    if (!ic->irqs[irq].masked) {
        ic->pending_reg |= (1U << irq);
        ic->irqs[irq].state = IRQ_PENDING;
    }
}

int irqc_get_highest(struct irq_controller *ic) {
    int best = -1;
    int best_prio = 0x7FFFFFFF;
    for (int i = 0; i < MAX_IRQS; i++) {
        if ((ic->pending_reg & (1U << i)) && ic->irqs[i].priority < best_prio) {
            best = i;
            best_prio = ic->irqs[i].priority;
        }
    }
    return best;
}

void irqc_ack(struct irq_controller *ic, int irq) {
    if (irq < 0 || irq >= MAX_IRQS) return;
    ic->pending_reg &= ~(1U << irq);
    ic->in_service |= (1U << irq);
    ic->irqs[irq].count++;
    ic->irqs[irq].state = IRQ_ENABLED;
    ic->nested_depth++;
}

void irqc_eoi(struct irq_controller *ic, int irq) {
    if (irq < 0 || irq >= MAX_IRQS) return;
    ic->in_service &= ~(1U << irq);
    if (ic->nested_depth > 0) ic->nested_depth--;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C491: Interrupt controller - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C491: empty output");
    assert!(code.contains("fn irqc_raise"), "C491: Should contain irqc_raise");
    assert!(code.contains("fn irqc_get_highest"), "C491: Should contain irqc_get_highest");
}

#[test]
fn c492_thread_local_storage_allocation() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_TLS_KEYS 64
#define MAX_THREADS 32
#define TLS_FREE  0
#define TLS_USED  1

struct tls_key {
    int state;
    uint32_t default_val;
};

struct tls_store {
    struct tls_key keys[64];
    uint32_t values[32][64];
    int key_count;
};

void tls_init(struct tls_store *ts) {
    ts->key_count = 0;
    for (int i = 0; i < MAX_TLS_KEYS; i++) {
        ts->keys[i].state = TLS_FREE;
        ts->keys[i].default_val = 0;
    }
    for (int t = 0; t < MAX_THREADS; t++) {
        for (int k = 0; k < MAX_TLS_KEYS; k++) {
            ts->values[t][k] = 0;
        }
    }
}

int tls_create_key(struct tls_store *ts, uint32_t default_val) {
    for (int i = 0; i < MAX_TLS_KEYS; i++) {
        if (ts->keys[i].state == TLS_FREE) {
            ts->keys[i].state = TLS_USED;
            ts->keys[i].default_val = default_val;
            ts->key_count++;
            for (int t = 0; t < MAX_THREADS; t++) {
                ts->values[t][i] = default_val;
            }
            return i;
        }
    }
    return -1;
}

int tls_delete_key(struct tls_store *ts, int key) {
    if (key < 0 || key >= MAX_TLS_KEYS) return -1;
    if (ts->keys[key].state == TLS_FREE) return -1;
    ts->keys[key].state = TLS_FREE;
    ts->key_count--;
    return 0;
}

uint32_t tls_get(const struct tls_store *ts, int thread, int key) {
    if (thread < 0 || thread >= MAX_THREADS) return 0;
    if (key < 0 || key >= MAX_TLS_KEYS) return 0;
    return ts->values[thread][key];
}

int tls_set(struct tls_store *ts, int thread, int key, uint32_t val) {
    if (thread < 0 || thread >= MAX_THREADS) return -1;
    if (key < 0 || key >= MAX_TLS_KEYS) return -1;
    if (ts->keys[key].state == TLS_FREE) return -1;
    ts->values[thread][key] = val;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C492: Thread-local storage - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C492: empty output");
    assert!(code.contains("fn tls_create_key"), "C492: Should contain tls_create_key");
    assert!(code.contains("fn tls_get"), "C492: Should contain tls_get");
    assert!(code.contains("fn tls_set"), "C492: Should contain tls_set");
}

#[test]
fn c493_copy_on_write_page_sharing() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_PAGES 256
#define PAGE_COW     0x01
#define PAGE_SHARED  0x02
#define PAGE_DIRTY   0x04
#define PAGE_FREE    0

struct page_info {
    uint32_t flags;
    int ref_count;
    int owner;
    uint32_t phys_addr;
};

struct cow_manager {
    struct page_info pages[256];
    int total_pages;
    int shared_count;
    int cow_faults;
};

void cow_init(struct cow_manager *cm) {
    cm->total_pages = MAX_PAGES;
    cm->shared_count = 0;
    cm->cow_faults = 0;
    for (int i = 0; i < MAX_PAGES; i++) {
        cm->pages[i].flags = PAGE_FREE;
        cm->pages[i].ref_count = 0;
        cm->pages[i].owner = -1;
        cm->pages[i].phys_addr = (uint32_t)(i * 4096);
    }
}

int cow_alloc_page(struct cow_manager *cm, int owner) {
    for (int i = 0; i < MAX_PAGES; i++) {
        if (cm->pages[i].flags == PAGE_FREE) {
            cm->pages[i].flags = 0;
            cm->pages[i].ref_count = 1;
            cm->pages[i].owner = owner;
            return i;
        }
    }
    return -1;
}

int cow_share_page(struct cow_manager *cm, int page_idx, int new_owner) {
    if (page_idx < 0 || page_idx >= MAX_PAGES) return -1;
    if (cm->pages[page_idx].flags == PAGE_FREE) return -1;
    cm->pages[page_idx].ref_count++;
    cm->pages[page_idx].flags |= PAGE_COW | PAGE_SHARED;
    cm->shared_count++;
    return 0;
}

int cow_handle_fault(struct cow_manager *cm, int page_idx, int pid) {
    if (page_idx < 0 || page_idx >= MAX_PAGES) return -1;
    if (!(cm->pages[page_idx].flags & PAGE_COW)) return -1;
    cm->cow_faults++;
    if (cm->pages[page_idx].ref_count == 1) {
        cm->pages[page_idx].flags &= ~PAGE_COW;
        cm->pages[page_idx].flags &= ~PAGE_SHARED;
        return page_idx;
    }
    int new_page = cow_alloc_page(cm, pid);
    if (new_page < 0) return -1;
    cm->pages[page_idx].ref_count--;
    if (cm->pages[page_idx].ref_count == 1) {
        cm->pages[page_idx].flags &= ~(PAGE_COW | PAGE_SHARED);
    }
    return new_page;
}

void cow_free_page(struct cow_manager *cm, int page_idx) {
    if (page_idx < 0 || page_idx >= MAX_PAGES) return;
    cm->pages[page_idx].ref_count--;
    if (cm->pages[page_idx].ref_count <= 0) {
        cm->pages[page_idx].flags = PAGE_FREE;
        cm->pages[page_idx].owner = -1;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C493: Copy-on-write pages - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C493: empty output");
    assert!(code.contains("fn cow_handle_fault"), "C493: Should contain cow_handle_fault");
    assert!(code.contains("fn cow_share_page"), "C493: Should contain cow_share_page");
}

#[test]
fn c494_kernel_module_dependency_graph() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_MODULES 32
#define MAX_DEPS 8
#define MOD_UNLOADED 0
#define MOD_LOADED   1
#define MOD_LOADING  2
#define MOD_ERROR    3

struct kmodule {
    int id;
    int state;
    int deps[8];
    int dep_count;
    int ref_count;
    uint32_t size;
};

struct mod_registry {
    struct kmodule modules[32];
    int count;
};

void modr_init(struct mod_registry *mr) {
    mr->count = 0;
    for (int i = 0; i < MAX_MODULES; i++) {
        mr->modules[i].id = i;
        mr->modules[i].state = MOD_UNLOADED;
        mr->modules[i].dep_count = 0;
        mr->modules[i].ref_count = 0;
        mr->modules[i].size = 0;
    }
}

int modr_register(struct mod_registry *mr, uint32_t size) {
    if (mr->count >= MAX_MODULES) return -1;
    int id = mr->count;
    mr->modules[id].state = MOD_UNLOADED;
    mr->modules[id].size = size;
    mr->modules[id].dep_count = 0;
    mr->modules[id].ref_count = 0;
    mr->count++;
    return id;
}

int modr_add_dep(struct mod_registry *mr, int mod_id, int dep_id) {
    if (mod_id < 0 || mod_id >= mr->count) return -1;
    if (dep_id < 0 || dep_id >= mr->count) return -1;
    if (mr->modules[mod_id].dep_count >= MAX_DEPS) return -1;
    mr->modules[mod_id].deps[mr->modules[mod_id].dep_count] = dep_id;
    mr->modules[mod_id].dep_count++;
    return 0;
}

int modr_can_load(const struct mod_registry *mr, int mod_id) {
    if (mod_id < 0 || mod_id >= mr->count) return 0;
    const struct kmodule *m = &mr->modules[mod_id];
    for (int i = 0; i < m->dep_count; i++) {
        if (mr->modules[m->deps[i]].state != MOD_LOADED) {
            return 0;
        }
    }
    return 1;
}

int modr_load(struct mod_registry *mr, int mod_id) {
    if (!modr_can_load(mr, mod_id)) return -1;
    mr->modules[mod_id].state = MOD_LOADED;
    for (int i = 0; i < mr->modules[mod_id].dep_count; i++) {
        mr->modules[mr->modules[mod_id].deps[i]].ref_count++;
    }
    return 0;
}

int modr_can_unload(const struct mod_registry *mr, int mod_id) {
    if (mod_id < 0 || mod_id >= mr->count) return 0;
    return mr->modules[mod_id].ref_count == 0 &&
           mr->modules[mod_id].state == MOD_LOADED;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C494: Kernel module deps - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C494: empty output");
    assert!(code.contains("fn modr_load"), "C494: Should contain modr_load");
    assert!(code.contains("fn modr_can_load"), "C494: Should contain modr_can_load");
}

#[test]
fn c495_block_io_elevator_scheduler() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define ELEV_QUEUE_SIZE 128
#define DIR_UP   1
#define DIR_DOWN 0
#define IO_READ  0
#define IO_WRITE 1

struct io_request {
    uint64_t sector;
    uint32_t count;
    int type;
    int served;
};

struct elevator {
    struct io_request queue[128];
    int req_count;
    uint64_t head_pos;
    int direction;
    int served_count;
};

void elev_init(struct elevator *e) {
    e->req_count = 0;
    e->head_pos = 0;
    e->direction = DIR_UP;
    e->served_count = 0;
}

int elev_add_request(struct elevator *e, uint64_t sector, uint32_t count, int type) {
    if (e->req_count >= ELEV_QUEUE_SIZE) return -1;
    int idx = e->req_count;
    e->queue[idx].sector = sector;
    e->queue[idx].count = count;
    e->queue[idx].type = type;
    e->queue[idx].served = 0;
    e->req_count++;
    return idx;
}

int elev_scan_next(struct elevator *e) {
    int best = -1;
    uint64_t best_dist = 0xFFFFFFFFFFFFFFFFULL;
    if (e->direction == DIR_UP) {
        for (int i = 0; i < e->req_count; i++) {
            if (!e->queue[i].served && e->queue[i].sector >= e->head_pos) {
                uint64_t dist = e->queue[i].sector - e->head_pos;
                if (dist < best_dist) {
                    best_dist = dist;
                    best = i;
                }
            }
        }
        if (best < 0) {
            e->direction = DIR_DOWN;
            return elev_scan_next(e);
        }
    } else {
        for (int i = 0; i < e->req_count; i++) {
            if (!e->queue[i].served && e->queue[i].sector <= e->head_pos) {
                uint64_t dist = e->head_pos - e->queue[i].sector;
                if (dist < best_dist) {
                    best_dist = dist;
                    best = i;
                }
            }
        }
        if (best < 0) {
            e->direction = DIR_UP;
            return -1;
        }
    }
    return best;
}

void elev_serve(struct elevator *e, int idx) {
    if (idx < 0 || idx >= e->req_count) return;
    e->queue[idx].served = 1;
    e->head_pos = e->queue[idx].sector;
    e->served_count++;
}

int elev_pending(const struct elevator *e) {
    int count = 0;
    for (int i = 0; i < e->req_count; i++) {
        if (!e->queue[i].served) count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C495: Elevator scheduler - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C495: empty output");
    assert!(code.contains("fn elev_scan_next"), "C495: Should contain elev_scan_next");
    assert!(code.contains("fn elev_serve"), "C495: Should contain elev_serve");
}

// ============================================================================
// C496-C500: Networking, Timekeeping, and Boot
// ============================================================================

#[test]
fn c496_network_packet_filter_rules() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;

#define MAX_RULES 64
#define ACTION_ACCEPT 1
#define ACTION_DROP   2
#define ACTION_LOG    3
#define PROTO_TCP 6
#define PROTO_UDP 17
#define PROTO_ICMP 1

struct filter_rule {
    uint32_t src_ip;
    uint32_t src_mask;
    uint32_t dst_ip;
    uint32_t dst_mask;
    uint16_t src_port;
    uint16_t dst_port;
    uint8_t protocol;
    uint8_t action;
    int enabled;
    uint32_t match_count;
};

struct packet_filter {
    struct filter_rule rules[64];
    int rule_count;
    uint32_t default_action;
    uint32_t total_packets;
    uint32_t dropped;
    uint32_t accepted;
};

void pf_init(struct packet_filter *pf) {
    pf->rule_count = 0;
    pf->default_action = ACTION_ACCEPT;
    pf->total_packets = 0;
    pf->dropped = 0;
    pf->accepted = 0;
}

int pf_add_rule(struct packet_filter *pf, uint32_t src, uint32_t smask,
                uint32_t dst, uint32_t dmask, uint8_t proto, uint8_t action) {
    if (pf->rule_count >= MAX_RULES) return -1;
    int idx = pf->rule_count;
    pf->rules[idx].src_ip = src;
    pf->rules[idx].src_mask = smask;
    pf->rules[idx].dst_ip = dst;
    pf->rules[idx].dst_mask = dmask;
    pf->rules[idx].protocol = proto;
    pf->rules[idx].action = action;
    pf->rules[idx].src_port = 0;
    pf->rules[idx].dst_port = 0;
    pf->rules[idx].enabled = 1;
    pf->rules[idx].match_count = 0;
    pf->rule_count++;
    return idx;
}

int pf_match_packet(struct packet_filter *pf, uint32_t src, uint32_t dst,
                    uint8_t proto) {
    pf->total_packets++;
    for (int i = 0; i < pf->rule_count; i++) {
        if (!pf->rules[i].enabled) continue;
        if ((src & pf->rules[i].src_mask) != (pf->rules[i].src_ip & pf->rules[i].src_mask))
            continue;
        if ((dst & pf->rules[i].dst_mask) != (pf->rules[i].dst_ip & pf->rules[i].dst_mask))
            continue;
        if (pf->rules[i].protocol != 0 && pf->rules[i].protocol != proto)
            continue;
        pf->rules[i].match_count++;
        if (pf->rules[i].action == ACTION_DROP) {
            pf->dropped++;
        } else {
            pf->accepted++;
        }
        return pf->rules[i].action;
    }
    if (pf->default_action == ACTION_DROP) {
        pf->dropped++;
    } else {
        pf->accepted++;
    }
    return pf->default_action;
}

void pf_disable_rule(struct packet_filter *pf, int idx) {
    if (idx >= 0 && idx < pf->rule_count) {
        pf->rules[idx].enabled = 0;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C496: Packet filter - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C496: empty output");
    assert!(code.contains("fn pf_match_packet"), "C496: Should contain pf_match_packet");
    assert!(code.contains("fn pf_add_rule"), "C496: Should contain pf_add_rule");
}

#[test]
fn c497_kernel_timekeeping() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define HZ 1000
#define NSEC_PER_SEC  1000000000UL
#define NSEC_PER_MSEC 1000000UL
#define NSEC_PER_USEC 1000UL

struct timespec {
    uint64_t tv_sec;
    uint32_t tv_nsec;
};

struct kernel_clock {
    uint64_t jiffies;
    struct timespec wall_time;
    struct timespec boot_time;
    uint64_t uptime_ms;
    uint32_t tick_nsec;
};

void kclock_init(struct kernel_clock *kc) {
    kc->jiffies = 0;
    kc->wall_time.tv_sec = 0;
    kc->wall_time.tv_nsec = 0;
    kc->boot_time.tv_sec = 0;
    kc->boot_time.tv_nsec = 0;
    kc->uptime_ms = 0;
    kc->tick_nsec = NSEC_PER_SEC / HZ;
}

void kclock_tick(struct kernel_clock *kc) {
    kc->jiffies++;
    kc->wall_time.tv_nsec += kc->tick_nsec;
    if (kc->wall_time.tv_nsec >= NSEC_PER_SEC) {
        kc->wall_time.tv_sec++;
        kc->wall_time.tv_nsec -= (uint32_t)NSEC_PER_SEC;
    }
    kc->uptime_ms = kc->jiffies * 1000 / HZ;
}

uint64_t kclock_get_jiffies(const struct kernel_clock *kc) {
    return kc->jiffies;
}

uint64_t kclock_uptime_seconds(const struct kernel_clock *kc) {
    return kc->uptime_ms / 1000;
}

uint64_t jiffies_to_msec(uint64_t j) {
    return j * 1000 / HZ;
}

uint64_t msec_to_jiffies(uint64_t ms) {
    return ms * HZ / 1000;
}

int timespec_compare(const struct timespec *a, const struct timespec *b) {
    if (a->tv_sec < b->tv_sec) return -1;
    if (a->tv_sec > b->tv_sec) return 1;
    if (a->tv_nsec < b->tv_nsec) return -1;
    if (a->tv_nsec > b->tv_nsec) return 1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C497: Kernel timekeeping - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C497: empty output");
    assert!(code.contains("fn kclock_tick"), "C497: Should contain kclock_tick");
    assert!(code.contains("fn jiffies_to_msec"), "C497: Should contain jiffies_to_msec");
}

#[test]
fn c498_kernel_pseudo_random_generator() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

struct kernel_rng {
    uint64_t state[4];
    uint32_t entropy_count;
    int initialized;
};

static uint64_t rotl(uint64_t x, int k) {
    return (x << k) | (x >> (64 - k));
}

void krng_init(struct kernel_rng *rng, uint64_t seed) {
    rng->state[0] = seed;
    rng->state[1] = seed ^ 0x6A09E667F3BCC908ULL;
    rng->state[2] = seed ^ 0xBB67AE8584CAA73BULL;
    rng->state[3] = seed ^ 0x3C6EF372FE94F82BULL;
    rng->entropy_count = 64;
    rng->initialized = 1;
}

uint64_t krng_next(struct kernel_rng *rng) {
    uint64_t result = rotl(rng->state[0] + rng->state[3], 23) + rng->state[0];
    uint64_t t = rng->state[1] << 17;
    rng->state[2] ^= rng->state[0];
    rng->state[3] ^= rng->state[1];
    rng->state[1] ^= rng->state[2];
    rng->state[0] ^= rng->state[3];
    rng->state[2] ^= t;
    rng->state[3] = rotl(rng->state[3], 45);
    return result;
}

uint32_t krng_next_u32(struct kernel_rng *rng) {
    return (uint32_t)(krng_next(rng) >> 32);
}

uint32_t krng_bounded(struct kernel_rng *rng, uint32_t bound) {
    if (bound == 0) return 0;
    return krng_next_u32(rng) % bound;
}

void krng_add_entropy(struct kernel_rng *rng, uint64_t noise) {
    rng->state[0] ^= noise;
    rng->entropy_count += 8;
    if (rng->entropy_count > 256) {
        rng->entropy_count = 256;
    }
    krng_next(rng);
}

int krng_has_entropy(const struct kernel_rng *rng) {
    return rng->entropy_count >= 64;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C498: Kernel RNG - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C498: empty output");
    assert!(code.contains("fn krng_next"), "C498: Should contain krng_next");
    assert!(code.contains("fn krng_add_entropy"), "C498: Should contain krng_add_entropy");
}

#[test]
fn c499_kernel_object_reference_counting() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_KOBJS 128
#define KOBJ_FREE    0
#define KOBJ_ACTIVE  1
#define KOBJ_DYING   2

struct kref {
    int count;
};

struct kobject {
    int id;
    int state;
    struct kref ref;
    int parent;
    int type;
    uint32_t flags;
};

struct kobj_pool {
    struct kobject objects[128];
    int count;
};

void kref_init(struct kref *kr) {
    kr->count = 1;
}

void kref_get(struct kref *kr) {
    kr->count++;
}

int kref_put(struct kref *kr) {
    kr->count--;
    return kr->count == 0;
}

void kpool_init(struct kobj_pool *kp) {
    kp->count = 0;
    for (int i = 0; i < MAX_KOBJS; i++) {
        kp->objects[i].id = i;
        kp->objects[i].state = KOBJ_FREE;
        kp->objects[i].ref.count = 0;
        kp->objects[i].parent = -1;
        kp->objects[i].type = 0;
        kp->objects[i].flags = 0;
    }
}

int kpool_alloc(struct kobj_pool *kp, int parent, int type) {
    for (int i = 0; i < MAX_KOBJS; i++) {
        if (kp->objects[i].state == KOBJ_FREE) {
            kp->objects[i].state = KOBJ_ACTIVE;
            kp->objects[i].parent = parent;
            kp->objects[i].type = type;
            kref_init(&kp->objects[i].ref);
            kp->count++;
            if (parent >= 0 && parent < MAX_KOBJS) {
                kref_get(&kp->objects[parent].ref);
            }
            return i;
        }
    }
    return -1;
}

int kpool_release(struct kobj_pool *kp, int id) {
    if (id < 0 || id >= MAX_KOBJS) return -1;
    if (kp->objects[id].state != KOBJ_ACTIVE) return -1;
    if (kref_put(&kp->objects[id].ref)) {
        kp->objects[id].state = KOBJ_FREE;
        kp->count--;
        int parent = kp->objects[id].parent;
        if (parent >= 0) {
            kpool_release(kp, parent);
        }
        return 1;
    }
    return 0;
}

int kpool_ref_count(const struct kobj_pool *kp, int id) {
    if (id < 0 || id >= MAX_KOBJS) return 0;
    return kp->objects[id].ref.count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C499: Kernel object refcounting - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C499: empty output");
    assert!(code.contains("fn kref_get"), "C499: Should contain kref_get");
    assert!(code.contains("fn kpool_release"), "C499: Should contain kpool_release");
}

#[test]
fn c500_elf_binary_header_parser() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;
typedef unsigned char uint8_t;
typedef unsigned long uint64_t;

#define ELF_MAGIC 0x464C457F
#define ET_EXEC 2
#define ET_DYN  3
#define EM_386    3
#define EM_X86_64 62
#define PT_LOAD 1
#define PT_NOTE 4
#define MAX_PHDRS 16

struct elf_header {
    uint32_t magic;
    uint8_t elf_class;
    uint8_t data_encoding;
    uint8_t version;
    uint8_t os_abi;
    uint16_t type;
    uint16_t machine;
    uint32_t entry;
    uint32_t phoff;
    uint32_t shoff;
    uint16_t phnum;
    uint16_t shnum;
};

struct program_header {
    uint32_t type;
    uint32_t offset;
    uint32_t vaddr;
    uint32_t paddr;
    uint32_t filesz;
    uint32_t memsz;
    uint32_t flags;
    uint32_t align;
};

struct elf_info {
    struct elf_header hdr;
    struct program_header phdrs[16];
    int valid;
    int loadable_count;
    uint32_t total_memsz;
};

int elf_validate(struct elf_info *ei) {
    if (ei->hdr.magic != ELF_MAGIC) {
        ei->valid = 0;
        return -1;
    }
    if (ei->hdr.type != ET_EXEC && ei->hdr.type != ET_DYN) {
        ei->valid = 0;
        return -2;
    }
    ei->valid = 1;
    return 0;
}

int elf_count_loadable(struct elf_info *ei) {
    int count = 0;
    int n = ei->hdr.phnum;
    if (n > MAX_PHDRS) n = MAX_PHDRS;
    for (int i = 0; i < n; i++) {
        if (ei->phdrs[i].type == PT_LOAD) {
            count++;
        }
    }
    ei->loadable_count = count;
    return count;
}

uint32_t elf_total_memory(struct elf_info *ei) {
    uint32_t total = 0;
    int n = ei->hdr.phnum;
    if (n > MAX_PHDRS) n = MAX_PHDRS;
    for (int i = 0; i < n; i++) {
        if (ei->phdrs[i].type == PT_LOAD) {
            total += ei->phdrs[i].memsz;
        }
    }
    ei->total_memsz = total;
    return total;
}

int elf_is_64bit(const struct elf_info *ei) {
    return ei->hdr.elf_class == 2;
}

int elf_is_executable(const struct elf_info *ei) {
    return ei->hdr.type == ET_EXEC;
}

uint32_t elf_entry_point(const struct elf_info *ei) {
    return ei->hdr.entry;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C500: ELF header parser - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C500: empty output");
    assert!(code.contains("fn elf_validate"), "C500: Should contain elf_validate");
    assert!(code.contains("fn elf_count_loadable"), "C500: Should contain elf_count_loadable");
    assert!(code.contains("fn elf_entry_point"), "C500: Should contain elf_entry_point");
}
