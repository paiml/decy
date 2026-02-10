//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1526-C1550: Garbage Collection Algorithms domain -- mark-sweep, reference counting,
//! generational GC, copying collectors, and advanced GC techniques.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise garbage collection algorithm patterns commonly found in
//! language runtimes, virtual machines, managed heaps, and memory-managed
//! systems -- all expressed as valid C99 without #include.
//!
//! Organization:
//! - C1526-C1530: Mark-sweep (tri-color marking, worklist sweep, bitmap allocator, free list, fragmentation compaction)
//! - C1531-C1535: Reference counting (basic refcount, weak refs, cycle detection, deferred decrement, epoch reclamation)
//! - C1536-C1540: Generational GC (nursery alloc, minor collection, write barrier, promoted tracking, remembered set)
//! - C1541-C1545: Copying collector (semi-space copy, Cheney scan, forwarding pointers, root set scan, stack map)
//! - C1546-C1550: Advanced GC (incremental marking, concurrent sweep, card table, large object space, finalization queue)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

use decy_core::transpile;

// ============================================================================
// C1526-C1530: Mark-Sweep GC
// ============================================================================

#[test]
fn c1526_tricolor_mark_sweep() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_HEAP_SIZE 256
#define GC_WHITE 0
#define GC_GRAY  1
#define GC_BLACK 2

typedef struct gc_obj {
    int color;
    int marked;
    size_t size;
    int ref_a;
    int ref_b;
    char data[32];
} gc_obj_t;

static gc_obj_t gc_heap[GC_HEAP_SIZE];
static int gc_alloc_count = 0;

int gc_tricolor_alloc(size_t size) {
    if (gc_alloc_count >= GC_HEAP_SIZE) return -1;
    int idx = gc_alloc_count++;
    gc_heap[idx].color = GC_WHITE;
    gc_heap[idx].marked = 0;
    gc_heap[idx].size = size;
    gc_heap[idx].ref_a = -1;
    gc_heap[idx].ref_b = -1;
    return idx;
}

void gc_tricolor_mark_gray(int idx) {
    if (idx < 0 || idx >= gc_alloc_count) return;
    if (gc_heap[idx].color == GC_WHITE) {
        gc_heap[idx].color = GC_GRAY;
    }
}

int gc_tricolor_process_grays(void) {
    int found = 0;
    int i;
    for (i = 0; i < gc_alloc_count; i++) {
        if (gc_heap[i].color == GC_GRAY) {
            gc_heap[i].color = GC_BLACK;
            gc_heap[i].marked = 1;
            if (gc_heap[i].ref_a >= 0)
                gc_tricolor_mark_gray(gc_heap[i].ref_a);
            if (gc_heap[i].ref_b >= 0)
                gc_tricolor_mark_gray(gc_heap[i].ref_b);
            found = 1;
        }
    }
    return found;
}

int gc_tricolor_sweep(void) {
    int freed = 0;
    int i;
    for (i = 0; i < gc_alloc_count; i++) {
        if (gc_heap[i].color == GC_WHITE) {
            gc_heap[i].size = 0;
            freed++;
        }
        gc_heap[i].color = GC_WHITE;
        gc_heap[i].marked = 0;
    }
    return freed;
}

int gc_tricolor_collect(int root) {
    gc_tricolor_mark_gray(root);
    while (gc_tricolor_process_grays()) { }
    return gc_tricolor_sweep();
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1526: Tri-color mark-sweep GC should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1527_worklist_sweep() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_WL_MAX 128
#define GC_WL_STACK 64

typedef struct {
    int alive;
    size_t size;
    int refs[4];
    int ref_count;
} gc_wl_obj_t;

static gc_wl_obj_t gc_wl_heap[GC_WL_MAX];
static int gc_wl_count = 0;

int gc_wl_alloc(size_t size) {
    if (gc_wl_count >= GC_WL_MAX) return -1;
    int idx = gc_wl_count++;
    gc_wl_heap[idx].alive = 1;
    gc_wl_heap[idx].size = size;
    gc_wl_heap[idx].ref_count = 0;
    return idx;
}

void gc_wl_add_ref(int from, int to) {
    if (from < 0 || from >= gc_wl_count) return;
    if (gc_wl_heap[from].ref_count < 4) {
        gc_wl_heap[from].refs[gc_wl_heap[from].ref_count++] = to;
    }
}

int gc_wl_sweep(int root) {
    int visited[GC_WL_MAX];
    int worklist[GC_WL_STACK];
    int wl_top = 0;
    int i;
    int freed = 0;

    for (i = 0; i < gc_wl_count; i++) visited[i] = 0;

    if (root >= 0 && root < gc_wl_count) {
        worklist[wl_top++] = root;
        visited[root] = 1;
    }

    while (wl_top > 0) {
        int cur = worklist[--wl_top];
        int j;
        for (j = 0; j < gc_wl_heap[cur].ref_count; j++) {
            int ref = gc_wl_heap[cur].refs[j];
            if (ref >= 0 && ref < gc_wl_count && !visited[ref]) {
                visited[ref] = 1;
                if (wl_top < GC_WL_STACK) {
                    worklist[wl_top++] = ref;
                }
            }
        }
    }

    for (i = 0; i < gc_wl_count; i++) {
        if (!visited[i] && gc_wl_heap[i].alive) {
            gc_wl_heap[i].alive = 0;
            freed++;
        }
    }
    return freed;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1527: Worklist-based sweep should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1528_bitmap_gc_allocator() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_BM_BLOCKS 256
#define GC_BM_BLOCK_SIZE 64
#define GC_BM_WORDS 8

typedef struct {
    char pool[GC_BM_BLOCKS * GC_BM_BLOCK_SIZE];
    unsigned int alloc_bits[GC_BM_WORDS];
    unsigned int mark_bits[GC_BM_WORDS];
} gc_bitmap_t;

void gc_bm_init(gc_bitmap_t *bm) {
    int i;
    for (i = 0; i < GC_BM_WORDS; i++) {
        bm->alloc_bits[i] = 0;
        bm->mark_bits[i] = 0;
    }
}

int gc_bm_alloc(gc_bitmap_t *bm) {
    int i;
    for (i = 0; i < GC_BM_BLOCKS; i++) {
        int word = i / 32;
        int bit = i % 32;
        if (!(bm->alloc_bits[word] & (1u << bit))) {
            bm->alloc_bits[word] |= (1u << bit);
            return i;
        }
    }
    return -1;
}

void gc_bm_mark(gc_bitmap_t *bm, int idx) {
    if (idx >= 0 && idx < GC_BM_BLOCKS) {
        int word = idx / 32;
        int bit = idx % 32;
        bm->mark_bits[word] |= (1u << bit);
    }
}

int gc_bm_sweep(gc_bitmap_t *bm) {
    int freed = 0;
    int i;
    for (i = 0; i < GC_BM_WORDS; i++) {
        unsigned int garbage = bm->alloc_bits[i] & ~bm->mark_bits[i];
        while (garbage) {
            garbage &= garbage - 1;
            freed++;
        }
        bm->alloc_bits[i] &= bm->mark_bits[i];
        bm->mark_bits[i] = 0;
    }
    return freed;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1528: Bitmap-based GC allocator should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1529_free_list_collector() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_FL_MAX 128

typedef struct gc_fl_node {
    size_t size;
    int in_use;
    int marked;
    int next_free;
} gc_fl_node_t;

static gc_fl_node_t gc_fl_nodes[GC_FL_MAX];
static int gc_fl_free_head = 0;
static int gc_fl_count = 0;

void gc_fl_init(void) {
    int i;
    for (i = 0; i < GC_FL_MAX; i++) {
        gc_fl_nodes[i].size = 0;
        gc_fl_nodes[i].in_use = 0;
        gc_fl_nodes[i].marked = 0;
        gc_fl_nodes[i].next_free = i + 1;
    }
    gc_fl_nodes[GC_FL_MAX - 1].next_free = -1;
    gc_fl_free_head = 0;
    gc_fl_count = 0;
}

int gc_fl_alloc(size_t size) {
    if (gc_fl_free_head < 0) return -1;
    int idx = gc_fl_free_head;
    gc_fl_free_head = gc_fl_nodes[idx].next_free;
    gc_fl_nodes[idx].size = size;
    gc_fl_nodes[idx].in_use = 1;
    gc_fl_nodes[idx].marked = 0;
    gc_fl_count++;
    return idx;
}

void gc_fl_mark(int idx) {
    if (idx >= 0 && idx < GC_FL_MAX) {
        gc_fl_nodes[idx].marked = 1;
    }
}

int gc_fl_sweep(void) {
    int freed = 0;
    int i;
    for (i = 0; i < GC_FL_MAX; i++) {
        if (gc_fl_nodes[i].in_use && !gc_fl_nodes[i].marked) {
            gc_fl_nodes[i].in_use = 0;
            gc_fl_nodes[i].next_free = gc_fl_free_head;
            gc_fl_free_head = i;
            gc_fl_count--;
            freed++;
        }
        gc_fl_nodes[i].marked = 0;
    }
    return freed;
}

int gc_fl_live_count(void) {
    return gc_fl_count;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1529: Free-list GC collector should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1530_fragmentation_compactor() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_CMP_MAX 64

typedef struct {
    int active;
    size_t offset;
    size_t size;
    int forwarded;
    size_t new_offset;
} gc_cmp_obj_t;

static gc_cmp_obj_t gc_cmp_objs[GC_CMP_MAX];
static int gc_cmp_count = 0;
static size_t gc_cmp_heap_top = 0;

int gc_cmp_alloc(size_t size) {
    if (gc_cmp_count >= GC_CMP_MAX) return -1;
    int idx = gc_cmp_count++;
    gc_cmp_objs[idx].active = 1;
    gc_cmp_objs[idx].offset = gc_cmp_heap_top;
    gc_cmp_objs[idx].size = size;
    gc_cmp_objs[idx].forwarded = 0;
    gc_cmp_objs[idx].new_offset = 0;
    gc_cmp_heap_top += size;
    return idx;
}

void gc_cmp_free(int idx) {
    if (idx >= 0 && idx < gc_cmp_count) {
        gc_cmp_objs[idx].active = 0;
    }
}

size_t gc_cmp_compact(void) {
    size_t write_ptr = 0;
    int i;
    for (i = 0; i < gc_cmp_count; i++) {
        if (gc_cmp_objs[i].active) {
            gc_cmp_objs[i].new_offset = write_ptr;
            gc_cmp_objs[i].forwarded = 1;
            write_ptr += gc_cmp_objs[i].size;
        }
    }
    for (i = 0; i < gc_cmp_count; i++) {
        if (gc_cmp_objs[i].forwarded) {
            gc_cmp_objs[i].offset = gc_cmp_objs[i].new_offset;
            gc_cmp_objs[i].forwarded = 0;
        }
    }
    size_t reclaimed = gc_cmp_heap_top - write_ptr;
    gc_cmp_heap_top = write_ptr;
    return reclaimed;
}

size_t gc_cmp_fragmentation(void) {
    size_t gaps = 0;
    size_t last_end = 0;
    int i;
    for (i = 0; i < gc_cmp_count; i++) {
        if (gc_cmp_objs[i].active) {
            if (gc_cmp_objs[i].offset > last_end) {
                gaps += gc_cmp_objs[i].offset - last_end;
            }
            last_end = gc_cmp_objs[i].offset + gc_cmp_objs[i].size;
        }
    }
    return gaps;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1530: Fragmentation compactor should transpile: {:?}",
        result.err()
    );
}

// ============================================================================
// C1531-C1535: Reference Counting
// ============================================================================

#[test]
fn c1531_basic_refcount() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_RC_MAX 128

typedef struct {
    int refcount;
    int alive;
    size_t size;
    char data[16];
} gc_rc_obj_t;

static gc_rc_obj_t gc_rc_heap[GC_RC_MAX];
static int gc_rc_count = 0;

int gc_rc_alloc(size_t size) {
    if (gc_rc_count >= GC_RC_MAX) return -1;
    int idx = gc_rc_count++;
    gc_rc_heap[idx].refcount = 1;
    gc_rc_heap[idx].alive = 1;
    gc_rc_heap[idx].size = size;
    return idx;
}

void gc_rc_incref(int idx) {
    if (idx >= 0 && idx < gc_rc_count && gc_rc_heap[idx].alive) {
        gc_rc_heap[idx].refcount++;
    }
}

void gc_rc_decref(int idx) {
    if (idx < 0 || idx >= gc_rc_count) return;
    if (!gc_rc_heap[idx].alive) return;
    gc_rc_heap[idx].refcount--;
    if (gc_rc_heap[idx].refcount <= 0) {
        gc_rc_heap[idx].alive = 0;
    }
}

int gc_rc_is_alive(int idx) {
    if (idx < 0 || idx >= gc_rc_count) return 0;
    return gc_rc_heap[idx].alive;
}

int gc_rc_get_refcount(int idx) {
    if (idx < 0 || idx >= gc_rc_count) return 0;
    return gc_rc_heap[idx].refcount;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1531: Basic reference counting should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1532_weak_references() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_WR_MAX 64
#define GC_WR_WEAK_MAX 64

typedef struct {
    int strong_count;
    int alive;
    size_t size;
} gc_wr_obj_t;

typedef struct {
    int target;
    int valid;
} gc_wr_weak_t;

static gc_wr_obj_t gc_wr_objs[GC_WR_MAX];
static gc_wr_weak_t gc_wr_weaks[GC_WR_WEAK_MAX];
static int gc_wr_obj_count = 0;
static int gc_wr_weak_count = 0;

int gc_wr_alloc(size_t size) {
    if (gc_wr_obj_count >= GC_WR_MAX) return -1;
    int idx = gc_wr_obj_count++;
    gc_wr_objs[idx].strong_count = 1;
    gc_wr_objs[idx].alive = 1;
    gc_wr_objs[idx].size = size;
    return idx;
}

int gc_wr_create_weak(int target) {
    if (gc_wr_weak_count >= GC_WR_WEAK_MAX) return -1;
    if (target < 0 || target >= gc_wr_obj_count) return -1;
    int idx = gc_wr_weak_count++;
    gc_wr_weaks[idx].target = target;
    gc_wr_weaks[idx].valid = gc_wr_objs[target].alive;
    return idx;
}

int gc_wr_upgrade_weak(int weak_idx) {
    if (weak_idx < 0 || weak_idx >= gc_wr_weak_count) return -1;
    if (!gc_wr_weaks[weak_idx].valid) return -1;
    int target = gc_wr_weaks[weak_idx].target;
    if (!gc_wr_objs[target].alive) {
        gc_wr_weaks[weak_idx].valid = 0;
        return -1;
    }
    gc_wr_objs[target].strong_count++;
    return target;
}

void gc_wr_release(int idx) {
    if (idx < 0 || idx >= gc_wr_obj_count) return;
    gc_wr_objs[idx].strong_count--;
    if (gc_wr_objs[idx].strong_count <= 0) {
        gc_wr_objs[idx].alive = 0;
        int i;
        for (i = 0; i < gc_wr_weak_count; i++) {
            if (gc_wr_weaks[i].target == idx) {
                gc_wr_weaks[i].valid = 0;
            }
        }
    }
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1532: Weak reference tracking should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1533_cycle_detection() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_CD_MAX 64

typedef struct {
    int refcount;
    int alive;
    int next;
    int visited;
    int in_stack;
} gc_cd_obj_t;

static gc_cd_obj_t gc_cd_objs[GC_CD_MAX];
static int gc_cd_count = 0;

int gc_cd_alloc(void) {
    if (gc_cd_count >= GC_CD_MAX) return -1;
    int idx = gc_cd_count++;
    gc_cd_objs[idx].refcount = 1;
    gc_cd_objs[idx].alive = 1;
    gc_cd_objs[idx].next = -1;
    gc_cd_objs[idx].visited = 0;
    gc_cd_objs[idx].in_stack = 0;
    return idx;
}

void gc_cd_set_next(int from, int to) {
    if (from < 0 || from >= gc_cd_count) return;
    if (gc_cd_objs[from].next >= 0 && gc_cd_objs[from].next < gc_cd_count)
        gc_cd_objs[gc_cd_objs[from].next].refcount--;
    gc_cd_objs[from].next = to;
    if (to >= 0 && to < gc_cd_count)
        gc_cd_objs[to].refcount++;
}

int gc_cd_detect_cycle(int start) {
    int slow = start;
    int fast = start;
    while (fast >= 0 && gc_cd_objs[fast].next >= 0) {
        slow = gc_cd_objs[slow].next;
        fast = gc_cd_objs[gc_cd_objs[fast].next].next;
        if (slow == fast) return 1;
    }
    return 0;
}

int gc_cd_collect_cycles(void) {
    int freed = 0;
    int i;
    for (i = 0; i < gc_cd_count; i++) {
        gc_cd_objs[i].visited = 0;
    }
    for (i = 0; i < gc_cd_count; i++) {
        if (gc_cd_objs[i].alive && gc_cd_objs[i].refcount > 0) {
            if (gc_cd_detect_cycle(i)) {
                int cur = i;
                do {
                    if (gc_cd_objs[cur].alive && !gc_cd_objs[cur].visited) {
                        gc_cd_objs[cur].visited = 1;
                    }
                    cur = gc_cd_objs[cur].next;
                } while (cur != i && cur >= 0);
            }
        }
    }
    return freed;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1533: Cycle detection for reference counting should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1534_deferred_decrement() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_DD_MAX 64
#define GC_DD_QUEUE 32

typedef struct {
    int refcount;
    int alive;
    size_t size;
} gc_dd_obj_t;

static gc_dd_obj_t gc_dd_objs[GC_DD_MAX];
static int gc_dd_count = 0;
static int gc_dd_queue[GC_DD_QUEUE];
static int gc_dd_queue_head = 0;
static int gc_dd_queue_tail = 0;

int gc_dd_alloc(size_t size) {
    if (gc_dd_count >= GC_DD_MAX) return -1;
    int idx = gc_dd_count++;
    gc_dd_objs[idx].refcount = 1;
    gc_dd_objs[idx].alive = 1;
    gc_dd_objs[idx].size = size;
    return idx;
}

void gc_dd_incref(int idx) {
    if (idx >= 0 && idx < gc_dd_count) {
        gc_dd_objs[idx].refcount++;
    }
}

void gc_dd_defer_decref(int idx) {
    if (idx < 0 || idx >= gc_dd_count) return;
    int next_tail = (gc_dd_queue_tail + 1) % GC_DD_QUEUE;
    if (next_tail != gc_dd_queue_head) {
        gc_dd_queue[gc_dd_queue_tail] = idx;
        gc_dd_queue_tail = next_tail;
    }
}

int gc_dd_process_deferred(void) {
    int freed = 0;
    while (gc_dd_queue_head != gc_dd_queue_tail) {
        int idx = gc_dd_queue[gc_dd_queue_head];
        gc_dd_queue_head = (gc_dd_queue_head + 1) % GC_DD_QUEUE;
        if (idx >= 0 && idx < gc_dd_count && gc_dd_objs[idx].alive) {
            gc_dd_objs[idx].refcount--;
            if (gc_dd_objs[idx].refcount <= 0) {
                gc_dd_objs[idx].alive = 0;
                freed++;
            }
        }
    }
    return freed;
}

int gc_dd_pending_count(void) {
    if (gc_dd_queue_tail >= gc_dd_queue_head)
        return gc_dd_queue_tail - gc_dd_queue_head;
    return GC_DD_QUEUE - gc_dd_queue_head + gc_dd_queue_tail;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1534: Deferred reference decrement should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1535_epoch_reclamation() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_EP_MAX 64
#define GC_EP_EPOCHS 3

typedef struct {
    int alive;
    size_t size;
    int retire_epoch;
} gc_ep_obj_t;

static gc_ep_obj_t gc_ep_objs[GC_EP_MAX];
static int gc_ep_count = 0;
static int gc_ep_current = 0;
static int gc_ep_thread_epochs[4];
static int gc_ep_num_threads = 0;

int gc_ep_alloc(size_t size) {
    if (gc_ep_count >= GC_EP_MAX) return -1;
    int idx = gc_ep_count++;
    gc_ep_objs[idx].alive = 1;
    gc_ep_objs[idx].size = size;
    gc_ep_objs[idx].retire_epoch = -1;
    return idx;
}

void gc_ep_register_thread(int tid) {
    if (tid >= 0 && tid < 4) {
        gc_ep_thread_epochs[tid] = gc_ep_current;
        if (tid >= gc_ep_num_threads) gc_ep_num_threads = tid + 1;
    }
}

void gc_ep_enter(int tid) {
    if (tid >= 0 && tid < 4) {
        gc_ep_thread_epochs[tid] = gc_ep_current;
    }
}

void gc_ep_retire(int idx) {
    if (idx >= 0 && idx < gc_ep_count) {
        gc_ep_objs[idx].retire_epoch = gc_ep_current;
    }
}

void gc_ep_advance(void) {
    gc_ep_current++;
}

int gc_ep_safe_epoch(void) {
    int min_epoch = gc_ep_current;
    int i;
    for (i = 0; i < gc_ep_num_threads; i++) {
        if (gc_ep_thread_epochs[i] < min_epoch) {
            min_epoch = gc_ep_thread_epochs[i];
        }
    }
    return min_epoch;
}

int gc_ep_reclaim(void) {
    int safe = gc_ep_safe_epoch();
    int freed = 0;
    int i;
    for (i = 0; i < gc_ep_count; i++) {
        if (gc_ep_objs[i].alive && gc_ep_objs[i].retire_epoch >= 0) {
            if (gc_ep_objs[i].retire_epoch < safe) {
                gc_ep_objs[i].alive = 0;
                freed++;
            }
        }
    }
    return freed;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1535: Epoch-based reclamation should transpile: {:?}",
        result.err()
    );
}

// ============================================================================
// C1536-C1540: Generational GC
// ============================================================================

#[test]
fn c1536_nursery_alloc() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_NURSERY_SIZE 1024
#define GC_NURSERY_MAX_OBJS 64

typedef struct {
    size_t offset;
    size_t size;
    int alive;
    int age;
} gc_nursery_obj_t;

static char gc_nursery_space[GC_NURSERY_SIZE];
static gc_nursery_obj_t gc_nursery_objs[GC_NURSERY_MAX_OBJS];
static int gc_nursery_count = 0;
static size_t gc_nursery_top = 0;

int gc_nursery_alloc(size_t size) {
    if (gc_nursery_count >= GC_NURSERY_MAX_OBJS) return -1;
    size_t aligned = (size + 7) & ~((size_t)7);
    if (gc_nursery_top + aligned > GC_NURSERY_SIZE) return -1;
    int idx = gc_nursery_count++;
    gc_nursery_objs[idx].offset = gc_nursery_top;
    gc_nursery_objs[idx].size = aligned;
    gc_nursery_objs[idx].alive = 1;
    gc_nursery_objs[idx].age = 0;
    gc_nursery_top += aligned;
    return idx;
}

void gc_nursery_mark(int idx) {
    if (idx >= 0 && idx < gc_nursery_count) {
        gc_nursery_objs[idx].alive = 1;
    }
}

int gc_nursery_is_full(void) {
    return gc_nursery_top >= GC_NURSERY_SIZE / 2;
}

size_t gc_nursery_used(void) {
    return gc_nursery_top;
}

void gc_nursery_reset(void) {
    gc_nursery_count = 0;
    gc_nursery_top = 0;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1536: Nursery allocation for generational GC should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1537_minor_collection() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_MC_YOUNG_MAX 64
#define GC_MC_OLD_MAX 64
#define GC_MC_TENURE_AGE 3

typedef struct {
    int alive;
    int marked;
    int age;
    size_t size;
    int generation;
} gc_mc_obj_t;

static gc_mc_obj_t gc_mc_young[GC_MC_YOUNG_MAX];
static gc_mc_obj_t gc_mc_old[GC_MC_OLD_MAX];
static int gc_mc_young_count = 0;
static int gc_mc_old_count = 0;

int gc_mc_young_alloc(size_t size) {
    if (gc_mc_young_count >= GC_MC_YOUNG_MAX) return -1;
    int idx = gc_mc_young_count++;
    gc_mc_young[idx].alive = 1;
    gc_mc_young[idx].marked = 0;
    gc_mc_young[idx].age = 0;
    gc_mc_young[idx].size = size;
    gc_mc_young[idx].generation = 0;
    return idx;
}

void gc_mc_mark_young(int idx) {
    if (idx >= 0 && idx < gc_mc_young_count) {
        gc_mc_young[idx].marked = 1;
    }
}

int gc_mc_promote(int idx) {
    if (gc_mc_old_count >= GC_MC_OLD_MAX) return -1;
    if (idx < 0 || idx >= gc_mc_young_count) return -1;
    int old_idx = gc_mc_old_count++;
    gc_mc_old[old_idx].alive = 1;
    gc_mc_old[old_idx].marked = 0;
    gc_mc_old[old_idx].age = gc_mc_young[idx].age;
    gc_mc_old[old_idx].size = gc_mc_young[idx].size;
    gc_mc_old[old_idx].generation = 1;
    gc_mc_young[idx].alive = 0;
    return old_idx;
}

int gc_mc_minor_collect(void) {
    int freed = 0;
    int i;
    for (i = 0; i < gc_mc_young_count; i++) {
        if (!gc_mc_young[i].alive) continue;
        if (gc_mc_young[i].marked) {
            gc_mc_young[i].age++;
            if (gc_mc_young[i].age >= GC_MC_TENURE_AGE) {
                gc_mc_promote(i);
            }
            gc_mc_young[i].marked = 0;
        } else {
            gc_mc_young[i].alive = 0;
            freed++;
        }
    }
    return freed;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1537: Minor collection with promotion should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1538_write_barrier() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_WB_MAX 64
#define GC_WB_DIRTY_MAX 32

typedef struct {
    int generation;
    int alive;
    int ref_target;
} gc_wb_obj_t;

static gc_wb_obj_t gc_wb_objs[GC_WB_MAX];
static int gc_wb_count = 0;
static int gc_wb_dirty_cards[GC_WB_DIRTY_MAX];
static int gc_wb_dirty_count = 0;

int gc_wb_alloc(int gen) {
    if (gc_wb_count >= GC_WB_MAX) return -1;
    int idx = gc_wb_count++;
    gc_wb_objs[idx].generation = gen;
    gc_wb_objs[idx].alive = 1;
    gc_wb_objs[idx].ref_target = -1;
    return idx;
}

void gc_wb_store(int src, int dst) {
    if (src < 0 || src >= gc_wb_count) return;
    if (dst < 0 || dst >= gc_wb_count) return;
    gc_wb_objs[src].ref_target = dst;
    if (gc_wb_objs[src].generation > gc_wb_objs[dst].generation) {
        if (gc_wb_dirty_count < GC_WB_DIRTY_MAX) {
            gc_wb_dirty_cards[gc_wb_dirty_count++] = src;
        }
    }
}

int gc_wb_get_dirty_count(void) {
    return gc_wb_dirty_count;
}

void gc_wb_clear_dirty(void) {
    gc_wb_dirty_count = 0;
}

int gc_wb_is_cross_gen(int src, int dst) {
    if (src < 0 || src >= gc_wb_count) return 0;
    if (dst < 0 || dst >= gc_wb_count) return 0;
    return gc_wb_objs[src].generation != gc_wb_objs[dst].generation;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1538: Write barrier for generational GC should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1539_promoted_tracking() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_PT_MAX 64
#define GC_PT_LOG_MAX 32

typedef struct {
    int alive;
    int generation;
    size_t size;
    int promoted_from;
    int promote_count;
} gc_pt_obj_t;

typedef struct {
    int obj_idx;
    int from_gen;
    int to_gen;
} gc_pt_log_t;

static gc_pt_obj_t gc_pt_objs[GC_PT_MAX];
static gc_pt_log_t gc_pt_log[GC_PT_LOG_MAX];
static int gc_pt_count = 0;
static int gc_pt_log_count = 0;

int gc_pt_alloc(size_t size) {
    if (gc_pt_count >= GC_PT_MAX) return -1;
    int idx = gc_pt_count++;
    gc_pt_objs[idx].alive = 1;
    gc_pt_objs[idx].generation = 0;
    gc_pt_objs[idx].size = size;
    gc_pt_objs[idx].promoted_from = -1;
    gc_pt_objs[idx].promote_count = 0;
    return idx;
}

int gc_pt_promote(int idx) {
    if (idx < 0 || idx >= gc_pt_count) return -1;
    if (!gc_pt_objs[idx].alive) return -1;
    int old_gen = gc_pt_objs[idx].generation;
    gc_pt_objs[idx].generation++;
    gc_pt_objs[idx].promoted_from = old_gen;
    gc_pt_objs[idx].promote_count++;
    if (gc_pt_log_count < GC_PT_LOG_MAX) {
        gc_pt_log[gc_pt_log_count].obj_idx = idx;
        gc_pt_log[gc_pt_log_count].from_gen = old_gen;
        gc_pt_log[gc_pt_log_count].to_gen = gc_pt_objs[idx].generation;
        gc_pt_log_count++;
    }
    return gc_pt_objs[idx].generation;
}

int gc_pt_total_promotions(void) {
    return gc_pt_log_count;
}

int gc_pt_gen_count(int gen) {
    int count = 0;
    int i;
    for (i = 0; i < gc_pt_count; i++) {
        if (gc_pt_objs[i].alive && gc_pt_objs[i].generation == gen)
            count++;
    }
    return count;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1539: Promoted object tracking should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1540_remembered_set() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_RS_MAX 64
#define GC_RS_SET_MAX 128

typedef struct {
    int src;
    int dst;
} gc_rs_entry_t;

static gc_rs_entry_t gc_rs_set[GC_RS_SET_MAX];
static int gc_rs_set_count = 0;
static int gc_rs_generations[GC_RS_MAX];
static int gc_rs_obj_count = 0;

int gc_rs_alloc_obj(int gen) {
    if (gc_rs_obj_count >= GC_RS_MAX) return -1;
    int idx = gc_rs_obj_count++;
    gc_rs_generations[idx] = gen;
    return idx;
}

void gc_rs_record(int src, int dst) {
    if (src < 0 || dst < 0) return;
    if (src >= gc_rs_obj_count || dst >= gc_rs_obj_count) return;
    if (gc_rs_generations[src] > gc_rs_generations[dst]) {
        if (gc_rs_set_count < GC_RS_SET_MAX) {
            gc_rs_set[gc_rs_set_count].src = src;
            gc_rs_set[gc_rs_set_count].dst = dst;
            gc_rs_set_count++;
        }
    }
}

int gc_rs_entries_for_gen(int gen) {
    int count = 0;
    int i;
    for (i = 0; i < gc_rs_set_count; i++) {
        if (gc_rs_generations[gc_rs_set[i].dst] == gen) {
            count++;
        }
    }
    return count;
}

void gc_rs_clear(void) {
    gc_rs_set_count = 0;
}

int gc_rs_size(void) {
    return gc_rs_set_count;
}

int gc_rs_contains(int src, int dst) {
    int i;
    for (i = 0; i < gc_rs_set_count; i++) {
        if (gc_rs_set[i].src == src && gc_rs_set[i].dst == dst)
            return 1;
    }
    return 0;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1540: Remembered set for generational GC should transpile: {:?}",
        result.err()
    );
}

// ============================================================================
// C1541-C1545: Copying Collector
// ============================================================================

#[test]
fn c1541_semispace_copy() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_SS_SIZE 2048
#define GC_SS_MAX_OBJS 64

typedef struct {
    size_t offset;
    size_t size;
    int alive;
    int space;
} gc_ss_obj_t;

static char gc_ss_space0[GC_SS_SIZE];
static char gc_ss_space1[GC_SS_SIZE];
static gc_ss_obj_t gc_ss_objs[GC_SS_MAX_OBJS];
static int gc_ss_count = 0;
static int gc_ss_active = 0;
static size_t gc_ss_top = 0;

int gc_ss_alloc(size_t size) {
    if (gc_ss_count >= GC_SS_MAX_OBJS) return -1;
    size_t aligned = (size + 7) & ~((size_t)7);
    if (gc_ss_top + aligned > GC_SS_SIZE) return -1;
    int idx = gc_ss_count++;
    gc_ss_objs[idx].offset = gc_ss_top;
    gc_ss_objs[idx].size = aligned;
    gc_ss_objs[idx].alive = 1;
    gc_ss_objs[idx].space = gc_ss_active;
    gc_ss_top += aligned;
    return idx;
}

int gc_ss_flip(void) {
    int to_space = 1 - gc_ss_active;
    size_t new_top = 0;
    int copied = 0;
    int i;
    for (i = 0; i < gc_ss_count; i++) {
        if (gc_ss_objs[i].alive) {
            gc_ss_objs[i].offset = new_top;
            gc_ss_objs[i].space = to_space;
            new_top += gc_ss_objs[i].size;
            copied++;
        }
    }
    gc_ss_active = to_space;
    gc_ss_top = new_top;
    return copied;
}

size_t gc_ss_free_space(void) {
    return GC_SS_SIZE - gc_ss_top;
}

int gc_ss_active_space(void) {
    return gc_ss_active;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1541: Semi-space copying collector should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1542_cheney_scan() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_CH_MAX 64

typedef struct {
    int alive;
    int copied;
    int refs[2];
    int ref_count;
    size_t size;
} gc_ch_obj_t;

static gc_ch_obj_t gc_ch_from[GC_CH_MAX];
static gc_ch_obj_t gc_ch_to[GC_CH_MAX];
static int gc_ch_from_count = 0;
static int gc_ch_to_count = 0;

int gc_ch_alloc(size_t size) {
    if (gc_ch_from_count >= GC_CH_MAX) return -1;
    int idx = gc_ch_from_count++;
    gc_ch_from[idx].alive = 1;
    gc_ch_from[idx].copied = 0;
    gc_ch_from[idx].ref_count = 0;
    gc_ch_from[idx].size = size;
    return idx;
}

void gc_ch_add_ref(int from, int to) {
    if (from < 0 || from >= gc_ch_from_count) return;
    if (gc_ch_from[from].ref_count < 2) {
        gc_ch_from[from].refs[gc_ch_from[from].ref_count++] = to;
    }
}

int gc_ch_copy_obj(int idx) {
    if (idx < 0 || idx >= gc_ch_from_count) return -1;
    if (gc_ch_from[idx].copied) return -1;
    if (gc_ch_to_count >= GC_CH_MAX) return -1;
    int new_idx = gc_ch_to_count++;
    gc_ch_to[new_idx].alive = 1;
    gc_ch_to[new_idx].copied = 0;
    gc_ch_to[new_idx].ref_count = gc_ch_from[idx].ref_count;
    gc_ch_to[new_idx].size = gc_ch_from[idx].size;
    int i;
    for (i = 0; i < gc_ch_from[idx].ref_count; i++) {
        gc_ch_to[new_idx].refs[i] = gc_ch_from[idx].refs[i];
    }
    gc_ch_from[idx].copied = 1;
    return new_idx;
}

int gc_ch_cheney_collect(int root) {
    gc_ch_to_count = 0;
    int scan = 0;
    gc_ch_copy_obj(root);
    while (scan < gc_ch_to_count) {
        int j;
        for (j = 0; j < gc_ch_to[scan].ref_count; j++) {
            int ref = gc_ch_to[scan].refs[j];
            if (ref >= 0 && ref < gc_ch_from_count && !gc_ch_from[ref].copied) {
                gc_ch_copy_obj(ref);
            }
        }
        scan++;
    }
    return gc_ch_to_count;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1542: Cheney scanning algorithm should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1543_forwarding_pointers() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_FP_MAX 64

typedef struct {
    int alive;
    size_t size;
    int forward_idx;
    int ref_target;
} gc_fp_obj_t;

static gc_fp_obj_t gc_fp_from[GC_FP_MAX];
static gc_fp_obj_t gc_fp_to[GC_FP_MAX];
static int gc_fp_from_count = 0;
static int gc_fp_to_count = 0;

int gc_fp_alloc(size_t size) {
    if (gc_fp_from_count >= GC_FP_MAX) return -1;
    int idx = gc_fp_from_count++;
    gc_fp_from[idx].alive = 1;
    gc_fp_from[idx].size = size;
    gc_fp_from[idx].forward_idx = -1;
    gc_fp_from[idx].ref_target = -1;
    return idx;
}

int gc_fp_forward(int idx) {
    if (idx < 0 || idx >= gc_fp_from_count) return -1;
    if (gc_fp_from[idx].forward_idx >= 0) {
        return gc_fp_from[idx].forward_idx;
    }
    if (gc_fp_to_count >= GC_FP_MAX) return -1;
    int new_idx = gc_fp_to_count++;
    gc_fp_to[new_idx].alive = 1;
    gc_fp_to[new_idx].size = gc_fp_from[idx].size;
    gc_fp_to[new_idx].forward_idx = -1;
    gc_fp_to[new_idx].ref_target = gc_fp_from[idx].ref_target;
    gc_fp_from[idx].forward_idx = new_idx;
    return new_idx;
}

int gc_fp_is_forwarded(int idx) {
    if (idx < 0 || idx >= gc_fp_from_count) return 0;
    return gc_fp_from[idx].forward_idx >= 0;
}

int gc_fp_resolve(int idx) {
    if (idx < 0 || idx >= gc_fp_from_count) return -1;
    if (gc_fp_from[idx].forward_idx >= 0)
        return gc_fp_from[idx].forward_idx;
    return idx;
}

void gc_fp_update_refs(void) {
    int i;
    for (i = 0; i < gc_fp_to_count; i++) {
        int ref = gc_fp_to[i].ref_target;
        if (ref >= 0 && ref < gc_fp_from_count) {
            if (gc_fp_from[ref].forward_idx >= 0) {
                gc_fp_to[i].ref_target = gc_fp_from[ref].forward_idx;
            }
        }
    }
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1543: Forwarding pointers in copying GC should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1544_root_set_scan() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_ROOT_MAX 32
#define GC_OBJ_MAX 64

typedef struct {
    int alive;
    int marked;
    int refs[3];
    int ref_count;
} gc_root_obj_t;

static gc_root_obj_t gc_root_objs[GC_OBJ_MAX];
static int gc_root_set[GC_ROOT_MAX];
static int gc_root_obj_count = 0;
static int gc_root_count = 0;

int gc_root_alloc(void) {
    if (gc_root_obj_count >= GC_OBJ_MAX) return -1;
    int idx = gc_root_obj_count++;
    gc_root_objs[idx].alive = 1;
    gc_root_objs[idx].marked = 0;
    gc_root_objs[idx].ref_count = 0;
    return idx;
}

void gc_root_add_root(int idx) {
    if (gc_root_count < GC_ROOT_MAX && idx >= 0) {
        gc_root_set[gc_root_count++] = idx;
    }
}

void gc_root_add_ref(int from, int to) {
    if (from < 0 || from >= gc_root_obj_count) return;
    if (gc_root_objs[from].ref_count < 3) {
        gc_root_objs[from].refs[gc_root_objs[from].ref_count++] = to;
    }
}

void gc_root_mark_recursive(int idx) {
    if (idx < 0 || idx >= gc_root_obj_count) return;
    if (gc_root_objs[idx].marked) return;
    gc_root_objs[idx].marked = 1;
    int i;
    for (i = 0; i < gc_root_objs[idx].ref_count; i++) {
        gc_root_mark_recursive(gc_root_objs[idx].refs[i]);
    }
}

int gc_root_scan_and_mark(void) {
    int i;
    for (i = 0; i < gc_root_obj_count; i++) {
        gc_root_objs[i].marked = 0;
    }
    for (i = 0; i < gc_root_count; i++) {
        gc_root_mark_recursive(gc_root_set[i]);
    }
    int marked = 0;
    for (i = 0; i < gc_root_obj_count; i++) {
        if (gc_root_objs[i].marked) marked++;
    }
    return marked;
}

int gc_root_count_roots(void) {
    return gc_root_count;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1544: Root set scanning should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1545_stack_map() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_SM_FRAMES 16
#define GC_SM_SLOTS 8

typedef struct {
    int slot_ids[GC_SM_SLOTS];
    int slot_count;
    int active;
} gc_sm_frame_t;

static gc_sm_frame_t gc_sm_frames[GC_SM_FRAMES];
static int gc_sm_frame_top = 0;

void gc_sm_init(void) {
    gc_sm_frame_top = 0;
}

int gc_sm_push_frame(void) {
    if (gc_sm_frame_top >= GC_SM_FRAMES) return -1;
    int idx = gc_sm_frame_top++;
    gc_sm_frames[idx].slot_count = 0;
    gc_sm_frames[idx].active = 1;
    return idx;
}

void gc_sm_pop_frame(void) {
    if (gc_sm_frame_top > 0) {
        gc_sm_frame_top--;
        gc_sm_frames[gc_sm_frame_top].active = 0;
    }
}

int gc_sm_register_slot(int obj_id) {
    if (gc_sm_frame_top <= 0) return -1;
    gc_sm_frame_t *frame = &gc_sm_frames[gc_sm_frame_top - 1];
    if (frame->slot_count >= GC_SM_SLOTS) return -1;
    frame->slot_ids[frame->slot_count++] = obj_id;
    return frame->slot_count - 1;
}

int gc_sm_scan_roots(int *out_roots, int max_roots) {
    int count = 0;
    int i;
    for (i = 0; i < gc_sm_frame_top; i++) {
        int j;
        for (j = 0; j < gc_sm_frames[i].slot_count; j++) {
            if (count < max_roots) {
                out_roots[count++] = gc_sm_frames[i].slot_ids[j];
            }
        }
    }
    return count;
}

int gc_sm_depth(void) {
    return gc_sm_frame_top;
}

int gc_sm_frame_slots(int frame_idx) {
    if (frame_idx < 0 || frame_idx >= gc_sm_frame_top) return 0;
    return gc_sm_frames[frame_idx].slot_count;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1545: Stack map for GC root scanning should transpile: {:?}",
        result.err()
    );
}

// ============================================================================
// C1546-C1550: Advanced GC Techniques
// ============================================================================

#[test]
fn c1546_incremental_marking() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_IM_MAX 128
#define GC_IM_STEP 8

typedef struct {
    int alive;
    int color;
    int refs[2];
    int ref_count;
} gc_im_obj_t;

static gc_im_obj_t gc_im_objs[GC_IM_MAX];
static int gc_im_count = 0;
static int gc_im_scan_pos = 0;
static int gc_im_phase = 0;

int gc_im_alloc(void) {
    if (gc_im_count >= GC_IM_MAX) return -1;
    int idx = gc_im_count++;
    gc_im_objs[idx].alive = 1;
    gc_im_objs[idx].color = 0;
    gc_im_objs[idx].ref_count = 0;
    return idx;
}

void gc_im_add_ref(int from, int to) {
    if (from < 0 || from >= gc_im_count) return;
    if (gc_im_objs[from].ref_count < 2) {
        gc_im_objs[from].refs[gc_im_objs[from].ref_count++] = to;
    }
}

void gc_im_start_mark(int root) {
    gc_im_phase = 1;
    gc_im_scan_pos = 0;
    if (root >= 0 && root < gc_im_count) {
        gc_im_objs[root].color = 1;
    }
}

int gc_im_mark_step(void) {
    int work = 0;
    int i;
    for (i = gc_im_scan_pos; i < gc_im_count && work < GC_IM_STEP; i++) {
        if (gc_im_objs[i].color == 1) {
            gc_im_objs[i].color = 2;
            int j;
            for (j = 0; j < gc_im_objs[i].ref_count; j++) {
                int ref = gc_im_objs[i].refs[j];
                if (ref >= 0 && ref < gc_im_count && gc_im_objs[ref].color == 0) {
                    gc_im_objs[ref].color = 1;
                }
            }
            work++;
        }
    }
    gc_im_scan_pos = i;
    return work;
}

int gc_im_is_complete(void) {
    int i;
    for (i = 0; i < gc_im_count; i++) {
        if (gc_im_objs[i].color == 1) return 0;
    }
    return 1;
}

int gc_im_sweep(void) {
    int freed = 0;
    int i;
    for (i = 0; i < gc_im_count; i++) {
        if (gc_im_objs[i].alive && gc_im_objs[i].color == 0) {
            gc_im_objs[i].alive = 0;
            freed++;
        }
        gc_im_objs[i].color = 0;
    }
    gc_im_phase = 0;
    return freed;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1546: Incremental marking GC should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1547_concurrent_sweep() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_CS_MAX 64
#define GC_CS_BATCH 4

typedef struct {
    int alive;
    int marked;
    size_t size;
    int swept;
} gc_cs_obj_t;

static gc_cs_obj_t gc_cs_objs[GC_CS_MAX];
static int gc_cs_count = 0;
static int gc_cs_sweep_cursor = 0;
static int gc_cs_sweep_active = 0;

int gc_cs_alloc(size_t size) {
    if (gc_cs_count >= GC_CS_MAX) return -1;
    int idx = gc_cs_count++;
    gc_cs_objs[idx].alive = 1;
    gc_cs_objs[idx].marked = 0;
    gc_cs_objs[idx].size = size;
    gc_cs_objs[idx].swept = 0;
    return idx;
}

void gc_cs_mark(int idx) {
    if (idx >= 0 && idx < gc_cs_count) {
        gc_cs_objs[idx].marked = 1;
    }
}

void gc_cs_begin_sweep(void) {
    gc_cs_sweep_cursor = 0;
    gc_cs_sweep_active = 1;
}

int gc_cs_sweep_batch(void) {
    if (!gc_cs_sweep_active) return 0;
    int freed = 0;
    int processed = 0;
    while (gc_cs_sweep_cursor < gc_cs_count && processed < GC_CS_BATCH) {
        int i = gc_cs_sweep_cursor++;
        gc_cs_objs[i].swept = 1;
        if (gc_cs_objs[i].alive && !gc_cs_objs[i].marked) {
            gc_cs_objs[i].alive = 0;
            freed++;
        }
        gc_cs_objs[i].marked = 0;
        processed++;
    }
    if (gc_cs_sweep_cursor >= gc_cs_count) {
        gc_cs_sweep_active = 0;
    }
    return freed;
}

int gc_cs_sweep_done(void) {
    return !gc_cs_sweep_active;
}

float gc_cs_sweep_progress(void) {
    if (gc_cs_count == 0) return 1.0f;
    return (float)gc_cs_sweep_cursor / (float)gc_cs_count;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1547: Concurrent sweep in batches should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1548_card_table() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_CT_HEAP_SIZE 4096
#define GC_CT_CARD_SIZE 128
#define GC_CT_NUM_CARDS 32

typedef struct {
    char heap[GC_CT_HEAP_SIZE];
    unsigned char cards[GC_CT_NUM_CARDS];
    size_t alloc_top;
} gc_ct_heap_t;

void gc_ct_init(gc_ct_heap_t *h) {
    int i;
    for (i = 0; i < GC_CT_NUM_CARDS; i++) {
        h->cards[i] = 0;
    }
    h->alloc_top = 0;
}

int gc_ct_card_for_offset(size_t offset) {
    return (int)(offset / GC_CT_CARD_SIZE);
}

void gc_ct_dirty(gc_ct_heap_t *h, size_t offset) {
    int card = gc_ct_card_for_offset(offset);
    if (card >= 0 && card < GC_CT_NUM_CARDS) {
        h->cards[card] = 1;
    }
}

int gc_ct_is_dirty(gc_ct_heap_t *h, int card) {
    if (card < 0 || card >= GC_CT_NUM_CARDS) return 0;
    return h->cards[card];
}

int gc_ct_dirty_count(gc_ct_heap_t *h) {
    int count = 0;
    int i;
    for (i = 0; i < GC_CT_NUM_CARDS; i++) {
        if (h->cards[i]) count++;
    }
    return count;
}

void gc_ct_clear_all(gc_ct_heap_t *h) {
    int i;
    for (i = 0; i < GC_CT_NUM_CARDS; i++) {
        h->cards[i] = 0;
    }
}

void gc_ct_clear_card(gc_ct_heap_t *h, int card) {
    if (card >= 0 && card < GC_CT_NUM_CARDS) {
        h->cards[card] = 0;
    }
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1548: Card table for write barrier tracking should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1549_large_object_space() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_LOS_MAX 32
#define GC_LOS_THRESHOLD 512

typedef struct {
    int alive;
    int marked;
    size_t size;
    int pinned;
} gc_los_obj_t;

static gc_los_obj_t gc_los_objs[GC_LOS_MAX];
static int gc_los_count = 0;
static size_t gc_los_total_bytes = 0;

int gc_los_alloc(size_t size) {
    if (size < GC_LOS_THRESHOLD) return -1;
    if (gc_los_count >= GC_LOS_MAX) return -1;
    int idx = gc_los_count++;
    gc_los_objs[idx].alive = 1;
    gc_los_objs[idx].marked = 0;
    gc_los_objs[idx].size = size;
    gc_los_objs[idx].pinned = 0;
    gc_los_total_bytes += size;
    return idx;
}

void gc_los_pin(int idx) {
    if (idx >= 0 && idx < gc_los_count) {
        gc_los_objs[idx].pinned = 1;
    }
}

void gc_los_mark(int idx) {
    if (idx >= 0 && idx < gc_los_count) {
        gc_los_objs[idx].marked = 1;
    }
}

int gc_los_sweep(void) {
    int freed = 0;
    int i;
    for (i = 0; i < gc_los_count; i++) {
        if (gc_los_objs[i].alive && !gc_los_objs[i].marked && !gc_los_objs[i].pinned) {
            gc_los_total_bytes -= gc_los_objs[i].size;
            gc_los_objs[i].alive = 0;
            freed++;
        }
        gc_los_objs[i].marked = 0;
    }
    return freed;
}

size_t gc_los_total_size(void) {
    return gc_los_total_bytes;
}

int gc_los_live_count(void) {
    int count = 0;
    int i;
    for (i = 0; i < gc_los_count; i++) {
        if (gc_los_objs[i].alive) count++;
    }
    return count;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1549: Large object space management should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1550_finalization_queue() {
    let c_code = r##"
typedef unsigned long size_t;

#define GC_FQ_MAX 64
#define GC_FQ_QUEUE 32

typedef struct {
    int alive;
    int marked;
    int has_finalizer;
    int finalized;
    size_t size;
} gc_fq_obj_t;

static gc_fq_obj_t gc_fq_objs[GC_FQ_MAX];
static int gc_fq_count = 0;
static int gc_fq_queue[GC_FQ_QUEUE];
static int gc_fq_queue_head = 0;
static int gc_fq_queue_tail = 0;

int gc_fq_alloc(size_t size, int has_finalizer) {
    if (gc_fq_count >= GC_FQ_MAX) return -1;
    int idx = gc_fq_count++;
    gc_fq_objs[idx].alive = 1;
    gc_fq_objs[idx].marked = 0;
    gc_fq_objs[idx].has_finalizer = has_finalizer;
    gc_fq_objs[idx].finalized = 0;
    gc_fq_objs[idx].size = size;
    return idx;
}

void gc_fq_mark(int idx) {
    if (idx >= 0 && idx < gc_fq_count) {
        gc_fq_objs[idx].marked = 1;
    }
}

int gc_fq_enqueue_finalizable(void) {
    int enqueued = 0;
    int i;
    for (i = 0; i < gc_fq_count; i++) {
        if (gc_fq_objs[i].alive && !gc_fq_objs[i].marked && gc_fq_objs[i].has_finalizer && !gc_fq_objs[i].finalized) {
            int next_tail = (gc_fq_queue_tail + 1) % GC_FQ_QUEUE;
            if (next_tail != gc_fq_queue_head) {
                gc_fq_queue[gc_fq_queue_tail] = i;
                gc_fq_queue_tail = next_tail;
                gc_fq_objs[i].marked = 1;
                enqueued++;
            }
        }
    }
    return enqueued;
}

int gc_fq_run_finalizers(void) {
    int ran = 0;
    while (gc_fq_queue_head != gc_fq_queue_tail) {
        int idx = gc_fq_queue[gc_fq_queue_head];
        gc_fq_queue_head = (gc_fq_queue_head + 1) % GC_FQ_QUEUE;
        if (idx >= 0 && idx < gc_fq_count) {
            gc_fq_objs[idx].finalized = 1;
            ran++;
        }
    }
    return ran;
}

int gc_fq_sweep(void) {
    int freed = 0;
    int i;
    for (i = 0; i < gc_fq_count; i++) {
        if (gc_fq_objs[i].alive && !gc_fq_objs[i].marked) {
            if (!gc_fq_objs[i].has_finalizer || gc_fq_objs[i].finalized) {
                gc_fq_objs[i].alive = 0;
                freed++;
            }
        }
        gc_fq_objs[i].marked = 0;
    }
    return freed;
}

int gc_fq_pending_finalizers(void) {
    if (gc_fq_queue_tail >= gc_fq_queue_head)
        return gc_fq_queue_tail - gc_fq_queue_head;
    return GC_FQ_QUEUE - gc_fq_queue_head + gc_fq_queue_tail;
}
"##;
    let result = transpile(c_code);
    assert!(
        result.is_ok(),
        "C1550: Finalization queue for GC should transpile: {:?}",
        result.err()
    );
}
