//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1651-C1675: Memory Pool and Arena Allocator domain -- fixed-size pools,
//! arena allocators, pool management, custom allocators, and memory tracking.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise memory pool and arena allocation patterns commonly
//! found in game engines, embedded runtimes, real-time systems, and
//! high-performance computing -- all expressed as valid C99 without #include.
//!
//! Organization:
//! - C1651-C1655: Fixed-size pools (slab allocator, object pool, free list pool, bitmap allocator, block pool)
//! - C1656-C1660: Arena allocators (linear arena, arena with reset, scoped arena, arena chain, typed arena)
//! - C1661-C1665: Pool management (pool statistics, watermark monitoring, pool resizing, defragmentation, pool isolation)
//! - C1666-C1670: Custom allocators (stack allocator, buddy allocator, segregated fits, TLSF allocator, zone allocator)
//! - C1671-C1675: Memory tracking (allocation tracking, leak detection, double-free detection, use-after-free guard, memory profiler)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

use decy_core::transpile;

// ============================================================================
// C1651-C1655: Fixed-Size Pools
// ============================================================================

#[test]
fn c1651_slab_allocator() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define MP_SLAB_OBJ_SIZE 64
#define MP_SLAB_CAPACITY 32

typedef struct {
    char storage[MP_SLAB_CAPACITY * MP_SLAB_OBJ_SIZE];
    int freelist[MP_SLAB_CAPACITY];
    int free_top;
    int active_count;
    uint32_t alloc_total;
} mp_slab_t;

void mp_slab_init(mp_slab_t *s) {
    int i;
    for (i = 0; i < MP_SLAB_CAPACITY; i++) {
        s->freelist[i] = i;
    }
    s->free_top = MP_SLAB_CAPACITY;
    s->active_count = 0;
    s->alloc_total = 0;
}

void *mp_slab_alloc(mp_slab_t *s) {
    if (s->free_top <= 0) return (void *)0;
    s->free_top--;
    int idx = s->freelist[s->free_top];
    s->active_count++;
    s->alloc_total++;
    return &s->storage[idx * MP_SLAB_OBJ_SIZE];
}

void mp_slab_free(mp_slab_t *s, void *ptr) {
    char *base = s->storage;
    int idx = (int)((char *)ptr - base) / MP_SLAB_OBJ_SIZE;
    if (idx >= 0 && idx < MP_SLAB_CAPACITY) {
        s->freelist[s->free_top++] = idx;
        s->active_count--;
    }
}

int mp_slab_available(mp_slab_t *s) {
    return s->free_top;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1651: Slab allocator should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1651: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1651: Should contain mp_ functions");
}

#[test]
fn c1652_object_pool() {
    let c_code = r##"
typedef unsigned long size_t;

#define MP_OBJPOOL_MAX 48
#define MP_OBJ_SIZE 96

typedef struct {
    char objects[MP_OBJPOOL_MAX * MP_OBJ_SIZE];
    int in_use[MP_OBJPOOL_MAX];
    int count_active;
    int count_recycled;
} mp_objpool_t;

void mp_objpool_init(mp_objpool_t *p) {
    int i;
    for (i = 0; i < MP_OBJPOOL_MAX; i++) {
        p->in_use[i] = 0;
    }
    p->count_active = 0;
    p->count_recycled = 0;
}

void *mp_objpool_acquire(mp_objpool_t *p) {
    int i;
    for (i = 0; i < MP_OBJPOOL_MAX; i++) {
        if (!p->in_use[i]) {
            p->in_use[i] = 1;
            p->count_active++;
            return &p->objects[i * MP_OBJ_SIZE];
        }
    }
    return (void *)0;
}

void mp_objpool_release(mp_objpool_t *p, void *ptr) {
    char *base = p->objects;
    int idx = (int)((char *)ptr - base) / MP_OBJ_SIZE;
    if (idx >= 0 && idx < MP_OBJPOOL_MAX && p->in_use[idx]) {
        p->in_use[idx] = 0;
        p->count_active--;
        p->count_recycled++;
    }
}

int mp_objpool_utilization(mp_objpool_t *p) {
    return (p->count_active * 100) / MP_OBJPOOL_MAX;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1652: Object pool should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1652: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1652: Should contain mp_ functions");
}

#[test]
fn c1653_free_list_pool() {
    let c_code = r##"
typedef unsigned long size_t;

#define MP_FREEPOOL_SLOTS 64
#define MP_FREEPOOL_SLOT_SIZE 128

typedef struct {
    char data[MP_FREEPOOL_SLOTS * MP_FREEPOOL_SLOT_SIZE];
    int next_free[MP_FREEPOOL_SLOTS];
    int head;
    int allocated;
} mp_freepool_t;

void mp_freepool_init(mp_freepool_t *fp) {
    int i;
    for (i = 0; i < MP_FREEPOOL_SLOTS - 1; i++) {
        fp->next_free[i] = i + 1;
    }
    fp->next_free[MP_FREEPOOL_SLOTS - 1] = -1;
    fp->head = 0;
    fp->allocated = 0;
}

void *mp_freepool_alloc(mp_freepool_t *fp) {
    if (fp->head < 0) return (void *)0;
    int idx = fp->head;
    fp->head = fp->next_free[idx];
    fp->next_free[idx] = -2;
    fp->allocated++;
    return &fp->data[idx * MP_FREEPOOL_SLOT_SIZE];
}

void mp_freepool_free(mp_freepool_t *fp, void *ptr) {
    char *base = fp->data;
    int idx = (int)((char *)ptr - base) / MP_FREEPOOL_SLOT_SIZE;
    if (idx >= 0 && idx < MP_FREEPOOL_SLOTS && fp->next_free[idx] == -2) {
        fp->next_free[idx] = fp->head;
        fp->head = idx;
        fp->allocated--;
    }
}

int mp_freepool_count_free(mp_freepool_t *fp) {
    return MP_FREEPOOL_SLOTS - fp->allocated;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1653: Free list pool should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1653: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1653: Should contain mp_ functions");
}

#[test]
fn c1654_bitmap_pool_allocator() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define MP_BMPOOL_BLOCKS 128
#define MP_BMPOOL_WORDS 4
#define MP_BMPOOL_BLOCK_SIZE 32

typedef struct {
    char memory[MP_BMPOOL_BLOCKS * MP_BMPOOL_BLOCK_SIZE];
    uint32_t bitmap[MP_BMPOOL_WORDS];
    int total_allocs;
    int total_frees;
} mp_bmpool_t;

void mp_bmpool_init(mp_bmpool_t *b) {
    int i;
    for (i = 0; i < MP_BMPOOL_WORDS; i++) {
        b->bitmap[i] = 0;
    }
    b->total_allocs = 0;
    b->total_frees = 0;
}

int mp_bmpool_alloc(mp_bmpool_t *b) {
    int w, bit;
    for (w = 0; w < MP_BMPOOL_WORDS; w++) {
        if (b->bitmap[w] != (uint32_t)0xFFFFFFFF) {
            for (bit = 0; bit < 32; bit++) {
                if (!(b->bitmap[w] & (1u << bit))) {
                    b->bitmap[w] |= (1u << bit);
                    b->total_allocs++;
                    return w * 32 + bit;
                }
            }
        }
    }
    return -1;
}

void mp_bmpool_free(mp_bmpool_t *b, int idx) {
    int w = idx / 32;
    int bit = idx % 32;
    if (w >= 0 && w < MP_BMPOOL_WORDS) {
        b->bitmap[w] &= ~(1u << bit);
        b->total_frees++;
    }
}

int mp_bmpool_count_used(mp_bmpool_t *b) {
    return b->total_allocs - b->total_frees;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1654: Bitmap pool allocator should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1654: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1654: Should contain mp_ functions");
}

#[test]
fn c1655_block_pool() {
    let c_code = r##"
typedef unsigned long size_t;

#define MP_BLOCKPOOL_SIZE 256
#define MP_BLOCK_SIZE 64

typedef struct {
    char blocks[MP_BLOCKPOOL_SIZE * MP_BLOCK_SIZE];
    int generation[MP_BLOCKPOOL_SIZE];
    int available[MP_BLOCKPOOL_SIZE];
    int free_count;
    int current_gen;
} mp_blockpool_t;

void mp_blockpool_init(mp_blockpool_t *bp) {
    int i;
    for (i = 0; i < MP_BLOCKPOOL_SIZE; i++) {
        bp->generation[i] = 0;
        bp->available[i] = 1;
    }
    bp->free_count = MP_BLOCKPOOL_SIZE;
    bp->current_gen = 1;
}

int mp_blockpool_alloc(mp_blockpool_t *bp) {
    int i;
    for (i = 0; i < MP_BLOCKPOOL_SIZE; i++) {
        if (bp->available[i]) {
            bp->available[i] = 0;
            bp->generation[i] = bp->current_gen;
            bp->free_count--;
            return i;
        }
    }
    return -1;
}

void mp_blockpool_free(mp_blockpool_t *bp, int idx) {
    if (idx >= 0 && idx < MP_BLOCKPOOL_SIZE && !bp->available[idx]) {
        bp->available[idx] = 1;
        bp->free_count++;
    }
}

int mp_blockpool_validate(mp_blockpool_t *bp, int idx) {
    if (idx < 0 || idx >= MP_BLOCKPOOL_SIZE) return 0;
    if (bp->available[idx]) return 0;
    return bp->generation[idx] == bp->current_gen;
}

void mp_blockpool_reset_gen(mp_blockpool_t *bp) {
    bp->current_gen++;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1655: Block pool should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1655: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1655: Should contain mp_ functions");
}

// ============================================================================
// C1656-C1660: Arena Allocators
// ============================================================================

#[test]
fn c1656_linear_arena() {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    char buffer[8192];
    size_t offset;
    size_t peak;
    int alloc_count;
} mp_arena_t;

void mp_arena_init(mp_arena_t *a) {
    a->offset = 0;
    a->peak = 0;
    a->alloc_count = 0;
}

void *mp_arena_alloc(mp_arena_t *a, size_t size) {
    size_t aligned = (size + 7) & ~((size_t)7);
    if (a->offset + aligned > 8192) return (void *)0;
    void *p = &a->buffer[a->offset];
    a->offset += aligned;
    if (a->offset > a->peak) a->peak = a->offset;
    a->alloc_count++;
    return p;
}

size_t mp_arena_remaining(mp_arena_t *a) {
    return 8192 - a->offset;
}

size_t mp_arena_used(mp_arena_t *a) {
    return a->offset;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1656: Linear arena should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1656: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1656: Should contain mp_ functions");
}

#[test]
fn c1657_arena_with_reset() {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    char buffer[4096];
    size_t offset;
    int generation;
    int alloc_count;
    int reset_count;
} mp_resettable_arena_t;

void mp_resettable_arena_init(mp_resettable_arena_t *a) {
    a->offset = 0;
    a->generation = 0;
    a->alloc_count = 0;
    a->reset_count = 0;
}

void *mp_resettable_arena_alloc(mp_resettable_arena_t *a, size_t size) {
    size_t aligned = (size + 7) & ~((size_t)7);
    if (a->offset + aligned > 4096) return (void *)0;
    void *p = &a->buffer[a->offset];
    a->offset += aligned;
    a->alloc_count++;
    return p;
}

void mp_resettable_arena_reset(mp_resettable_arena_t *a) {
    a->offset = 0;
    a->generation++;
    a->alloc_count = 0;
    a->reset_count++;
}

int mp_resettable_arena_generation(mp_resettable_arena_t *a) {
    return a->generation;
}

int mp_resettable_arena_is_full(mp_resettable_arena_t *a) {
    return a->offset >= 4096;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1657: Arena with reset should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1657: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1657: Should contain mp_ functions");
}

#[test]
fn c1658_scoped_arena() {
    let c_code = r##"
typedef unsigned long size_t;

#define MP_SCOPE_MAX_DEPTH 16

typedef struct {
    char buffer[4096];
    size_t offset;
    size_t scope_stack[MP_SCOPE_MAX_DEPTH];
    int scope_depth;
} mp_scoped_arena_t;

void mp_scoped_arena_init(mp_scoped_arena_t *a) {
    a->offset = 0;
    a->scope_depth = 0;
}

int mp_scoped_arena_push(mp_scoped_arena_t *a) {
    if (a->scope_depth >= MP_SCOPE_MAX_DEPTH) return -1;
    a->scope_stack[a->scope_depth] = a->offset;
    a->scope_depth++;
    return a->scope_depth;
}

void mp_scoped_arena_pop(mp_scoped_arena_t *a) {
    if (a->scope_depth > 0) {
        a->scope_depth--;
        a->offset = a->scope_stack[a->scope_depth];
    }
}

void *mp_scoped_arena_alloc(mp_scoped_arena_t *a, size_t size) {
    size_t aligned = (size + 7) & ~((size_t)7);
    if (a->offset + aligned > 4096) return (void *)0;
    void *p = &a->buffer[a->offset];
    a->offset += aligned;
    return p;
}

int mp_scoped_arena_depth(mp_scoped_arena_t *a) {
    return a->scope_depth;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1658: Scoped arena should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1658: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1658: Should contain mp_ functions");
}

#[test]
fn c1659_arena_chain() {
    let c_code = r##"
typedef unsigned long size_t;

#define MP_CHAIN_BLOCK_SIZE 1024
#define MP_CHAIN_MAX_BLOCKS 8

typedef struct {
    char blocks[MP_CHAIN_MAX_BLOCKS][MP_CHAIN_BLOCK_SIZE];
    size_t offsets[MP_CHAIN_MAX_BLOCKS];
    int current_block;
    int total_blocks;
    int total_allocs;
} mp_chain_arena_t;

void mp_chain_arena_init(mp_chain_arena_t *c) {
    c->current_block = 0;
    c->total_blocks = 1;
    c->offsets[0] = 0;
    c->total_allocs = 0;
}

void *mp_chain_arena_alloc(mp_chain_arena_t *c, size_t size) {
    size_t aligned = (size + 7) & ~((size_t)7);
    if (aligned > MP_CHAIN_BLOCK_SIZE) return (void *)0;
    if (c->offsets[c->current_block] + aligned > MP_CHAIN_BLOCK_SIZE) {
        if (c->total_blocks >= MP_CHAIN_MAX_BLOCKS) return (void *)0;
        c->current_block = c->total_blocks;
        c->offsets[c->current_block] = 0;
        c->total_blocks++;
    }
    void *p = &c->blocks[c->current_block][c->offsets[c->current_block]];
    c->offsets[c->current_block] += aligned;
    c->total_allocs++;
    return p;
}

void mp_chain_arena_reset(mp_chain_arena_t *c) {
    int i;
    for (i = 0; i < c->total_blocks; i++) {
        c->offsets[i] = 0;
    }
    c->current_block = 0;
    c->total_blocks = 1;
    c->total_allocs = 0;
}

size_t mp_chain_arena_total_used(mp_chain_arena_t *c) {
    size_t total = 0;
    int i;
    for (i = 0; i < c->total_blocks; i++) {
        total += c->offsets[i];
    }
    return total;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1659: Arena chain should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1659: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1659: Should contain mp_ functions");
}

#[test]
fn c1660_typed_arena() {
    let c_code = r##"
typedef unsigned long size_t;

#define MP_TYPED_MAX_ENTRIES 64
#define MP_TYPED_ENTRY_SIZE 48

typedef struct {
    char pool[MP_TYPED_MAX_ENTRIES * MP_TYPED_ENTRY_SIZE];
    int used[MP_TYPED_MAX_ENTRIES];
    size_t entry_size;
    int count;
    int capacity;
} mp_typed_arena_t;

void mp_typed_arena_init(mp_typed_arena_t *ta, size_t entry_size) {
    int i;
    ta->entry_size = entry_size;
    if (ta->entry_size > MP_TYPED_ENTRY_SIZE) ta->entry_size = MP_TYPED_ENTRY_SIZE;
    ta->capacity = MP_TYPED_MAX_ENTRIES;
    ta->count = 0;
    for (i = 0; i < MP_TYPED_MAX_ENTRIES; i++) {
        ta->used[i] = 0;
    }
}

void *mp_typed_arena_new(mp_typed_arena_t *ta) {
    int i;
    for (i = 0; i < ta->capacity; i++) {
        if (!ta->used[i]) {
            ta->used[i] = 1;
            ta->count++;
            return &ta->pool[i * MP_TYPED_ENTRY_SIZE];
        }
    }
    return (void *)0;
}

void mp_typed_arena_delete(mp_typed_arena_t *ta, void *ptr) {
    char *base = ta->pool;
    int idx = (int)((char *)ptr - base) / MP_TYPED_ENTRY_SIZE;
    if (idx >= 0 && idx < ta->capacity && ta->used[idx]) {
        ta->used[idx] = 0;
        ta->count--;
    }
}

int mp_typed_arena_count(mp_typed_arena_t *ta) {
    return ta->count;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1660: Typed arena should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1660: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1660: Should contain mp_ functions");
}

// ============================================================================
// C1661-C1665: Pool Management
// ============================================================================

#[test]
fn c1661_pool_statistics() {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int total_allocs;
    int total_frees;
    size_t total_bytes_allocated;
    size_t total_bytes_freed;
    size_t peak_usage;
    size_t current_usage;
    int fragmentation_count;
} mp_pool_stats_t;

void mp_pool_stats_init(mp_pool_stats_t *s) {
    s->total_allocs = 0;
    s->total_frees = 0;
    s->total_bytes_allocated = 0;
    s->total_bytes_freed = 0;
    s->peak_usage = 0;
    s->current_usage = 0;
    s->fragmentation_count = 0;
}

void mp_pool_stats_record_alloc(mp_pool_stats_t *s, size_t bytes) {
    s->total_allocs++;
    s->total_bytes_allocated += bytes;
    s->current_usage += bytes;
    if (s->current_usage > s->peak_usage) {
        s->peak_usage = s->current_usage;
    }
}

void mp_pool_stats_record_free(mp_pool_stats_t *s, size_t bytes) {
    s->total_frees++;
    s->total_bytes_freed += bytes;
    if (s->current_usage >= bytes) {
        s->current_usage -= bytes;
    }
}

int mp_pool_stats_efficiency(mp_pool_stats_t *s) {
    if (s->total_bytes_allocated == 0) return 100;
    return (int)((s->total_bytes_freed * 100) / s->total_bytes_allocated);
}

int mp_pool_stats_has_leaks(mp_pool_stats_t *s) {
    return s->total_allocs != s->total_frees;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1661: Pool statistics should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1661: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1661: Should contain mp_ functions");
}

#[test]
fn c1662_watermark_monitoring() {
    let c_code = r##"
typedef unsigned long size_t;

#define MP_WM_POOL_SIZE 512

typedef struct {
    int slots[MP_WM_POOL_SIZE];
    int used;
    int high_watermark;
    int low_watermark;
    int warning_threshold;
    int critical_threshold;
    int warnings_issued;
} mp_watermark_pool_t;

void mp_watermark_init(mp_watermark_pool_t *w) {
    w->used = 0;
    w->high_watermark = 0;
    w->low_watermark = MP_WM_POOL_SIZE;
    w->warning_threshold = MP_WM_POOL_SIZE * 75 / 100;
    w->critical_threshold = MP_WM_POOL_SIZE * 90 / 100;
    w->warnings_issued = 0;
}

int mp_watermark_alloc(mp_watermark_pool_t *w, int value) {
    if (w->used >= MP_WM_POOL_SIZE) return -1;
    w->slots[w->used] = value;
    w->used++;
    if (w->used > w->high_watermark) {
        w->high_watermark = w->used;
    }
    if (w->used >= w->warning_threshold) {
        w->warnings_issued++;
    }
    return w->used - 1;
}

void mp_watermark_free(mp_watermark_pool_t *w) {
    if (w->used > 0) {
        w->used--;
        if (w->used < w->low_watermark) {
            w->low_watermark = w->used;
        }
    }
}

int mp_watermark_status(mp_watermark_pool_t *w) {
    if (w->used >= w->critical_threshold) return 2;
    if (w->used >= w->warning_threshold) return 1;
    return 0;
}

int mp_watermark_high(mp_watermark_pool_t *w) {
    return w->high_watermark;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1662: Watermark monitoring should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1662: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1662: Should contain mp_ functions");
}

#[test]
fn c1663_pool_resizing() {
    let c_code = r##"
typedef unsigned long size_t;

#define MP_RESIZE_INITIAL 16
#define MP_RESIZE_MAX 256

typedef struct {
    int entries[MP_RESIZE_MAX];
    int capacity;
    int used;
    int grow_count;
    int shrink_count;
} mp_resizable_pool_t;

void mp_resizable_init(mp_resizable_pool_t *p) {
    p->capacity = MP_RESIZE_INITIAL;
    p->used = 0;
    p->grow_count = 0;
    p->shrink_count = 0;
}

int mp_resizable_grow(mp_resizable_pool_t *p) {
    int new_cap = p->capacity * 2;
    if (new_cap > MP_RESIZE_MAX) new_cap = MP_RESIZE_MAX;
    if (new_cap <= p->capacity) return -1;
    p->capacity = new_cap;
    p->grow_count++;
    return new_cap;
}

int mp_resizable_shrink(mp_resizable_pool_t *p) {
    if (p->used > 0 && p->used <= p->capacity / 4 && p->capacity > MP_RESIZE_INITIAL) {
        p->capacity = p->capacity / 2;
        if (p->capacity < MP_RESIZE_INITIAL) p->capacity = MP_RESIZE_INITIAL;
        p->shrink_count++;
        return p->capacity;
    }
    return 0;
}

int mp_resizable_add(mp_resizable_pool_t *p, int value) {
    if (p->used >= p->capacity) {
        if (mp_resizable_grow(p) < 0) return -1;
    }
    p->entries[p->used] = value;
    return p->used++;
}

void mp_resizable_remove(mp_resizable_pool_t *p, int idx) {
    if (idx >= 0 && idx < p->used) {
        int i;
        for (i = idx; i < p->used - 1; i++) {
            p->entries[i] = p->entries[i + 1];
        }
        p->used--;
        mp_resizable_shrink(p);
    }
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1663: Pool resizing should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1663: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1663: Should contain mp_ functions");
}

#[test]
fn c1664_defragmentation() {
    let c_code = r##"
typedef unsigned long size_t;

#define MP_DEFRAG_MAX 64

typedef struct {
    size_t offset;
    size_t size;
    int active;
    int id;
} mp_defrag_block_t;

typedef struct {
    mp_defrag_block_t blocks[MP_DEFRAG_MAX];
    int count;
    size_t total_size;
    int defrag_count;
} mp_defrag_pool_t;

void mp_defrag_init(mp_defrag_pool_t *d, size_t total) {
    d->count = 0;
    d->total_size = total;
    d->defrag_count = 0;
}

int mp_defrag_alloc(mp_defrag_pool_t *d, size_t size, int id) {
    if (d->count >= MP_DEFRAG_MAX) return -1;
    size_t offset = 0;
    int i;
    for (i = 0; i < d->count; i++) {
        if (d->blocks[i].active) {
            size_t end = d->blocks[i].offset + d->blocks[i].size;
            if (end > offset) offset = end;
        }
    }
    if (offset + size > d->total_size) return -1;
    int idx = d->count++;
    d->blocks[idx].offset = offset;
    d->blocks[idx].size = size;
    d->blocks[idx].active = 1;
    d->blocks[idx].id = id;
    return idx;
}

void mp_defrag_free(mp_defrag_pool_t *d, int idx) {
    if (idx >= 0 && idx < d->count) {
        d->blocks[idx].active = 0;
    }
}

int mp_defrag_compact(mp_defrag_pool_t *d) {
    size_t next_offset = 0;
    int moved = 0;
    int i;
    for (i = 0; i < d->count; i++) {
        if (d->blocks[i].active) {
            if (d->blocks[i].offset != next_offset) {
                d->blocks[i].offset = next_offset;
                moved++;
            }
            next_offset += d->blocks[i].size;
        }
    }
    d->defrag_count++;
    return moved;
}

size_t mp_defrag_largest_free(mp_defrag_pool_t *d) {
    size_t used_end = 0;
    int i;
    for (i = 0; i < d->count; i++) {
        if (d->blocks[i].active) {
            size_t end = d->blocks[i].offset + d->blocks[i].size;
            if (end > used_end) used_end = end;
        }
    }
    return d->total_size - used_end;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1664: Defragmentation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1664: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1664: Should contain mp_ functions");
}

#[test]
fn c1665_pool_isolation() {
    let c_code = r##"
typedef unsigned long size_t;

#define MP_ISO_ZONES 4
#define MP_ISO_ZONE_SIZE 1024

typedef struct {
    char memory[MP_ISO_ZONES * MP_ISO_ZONE_SIZE];
    size_t zone_offsets[MP_ISO_ZONES];
    int zone_alloc_counts[MP_ISO_ZONES];
    int zone_active[MP_ISO_ZONES];
} mp_isolated_pool_t;

void mp_isolated_init(mp_isolated_pool_t *p) {
    int i;
    for (i = 0; i < MP_ISO_ZONES; i++) {
        p->zone_offsets[i] = 0;
        p->zone_alloc_counts[i] = 0;
        p->zone_active[i] = 1;
    }
}

void *mp_isolated_alloc(mp_isolated_pool_t *p, int zone, size_t size) {
    if (zone < 0 || zone >= MP_ISO_ZONES) return (void *)0;
    if (!p->zone_active[zone]) return (void *)0;
    size_t aligned = (size + 7) & ~((size_t)7);
    if (p->zone_offsets[zone] + aligned > MP_ISO_ZONE_SIZE) return (void *)0;
    size_t base_offset = (size_t)zone * MP_ISO_ZONE_SIZE;
    void *ptr = &p->memory[base_offset + p->zone_offsets[zone]];
    p->zone_offsets[zone] += aligned;
    p->zone_alloc_counts[zone]++;
    return ptr;
}

void mp_isolated_reset_zone(mp_isolated_pool_t *p, int zone) {
    if (zone >= 0 && zone < MP_ISO_ZONES) {
        p->zone_offsets[zone] = 0;
        p->zone_alloc_counts[zone] = 0;
    }
}

void mp_isolated_disable_zone(mp_isolated_pool_t *p, int zone) {
    if (zone >= 0 && zone < MP_ISO_ZONES) {
        p->zone_active[zone] = 0;
    }
}

size_t mp_isolated_zone_used(mp_isolated_pool_t *p, int zone) {
    if (zone < 0 || zone >= MP_ISO_ZONES) return 0;
    return p->zone_offsets[zone];
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1665: Pool isolation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1665: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1665: Should contain mp_ functions");
}

// ============================================================================
// C1666-C1670: Custom Allocators
// ============================================================================

#[test]
fn c1666_stack_allocator() {
    let c_code = r##"
typedef unsigned long size_t;

#define MP_STACK_SIZE 4096
#define MP_STACK_MAX_MARKS 32

typedef struct {
    char buffer[MP_STACK_SIZE];
    size_t top;
    size_t marks[MP_STACK_MAX_MARKS];
    int mark_count;
    int alloc_count;
} mp_stack_alloc_t;

void mp_stack_alloc_init(mp_stack_alloc_t *s) {
    s->top = 0;
    s->mark_count = 0;
    s->alloc_count = 0;
}

void *mp_stack_alloc_push(mp_stack_alloc_t *s, size_t size) {
    size_t aligned = (size + 7) & ~((size_t)7);
    if (s->top + aligned > MP_STACK_SIZE) return (void *)0;
    void *p = &s->buffer[s->top];
    s->top += aligned;
    s->alloc_count++;
    return p;
}

int mp_stack_alloc_save(mp_stack_alloc_t *s) {
    if (s->mark_count >= MP_STACK_MAX_MARKS) return -1;
    s->marks[s->mark_count] = s->top;
    return s->mark_count++;
}

void mp_stack_alloc_restore(mp_stack_alloc_t *s, int mark) {
    if (mark >= 0 && mark < s->mark_count) {
        s->top = s->marks[mark];
        s->mark_count = mark;
    }
}

size_t mp_stack_alloc_remaining(mp_stack_alloc_t *s) {
    return MP_STACK_SIZE - s->top;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1666: Stack allocator should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1666: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1666: Should contain mp_ functions");
}

#[test]
fn c1667_buddy_allocator() {
    let c_code = r##"
typedef unsigned long size_t;

#define MP_BUDDY_LEVELS 6
#define MP_BUDDY_NODES 127

typedef struct {
    int split[MP_BUDDY_NODES];
    int used[MP_BUDDY_NODES];
    size_t total_size;
    int alloc_count;
} mp_buddy_t;

void mp_buddy_init(mp_buddy_t *b, size_t size) {
    int i;
    for (i = 0; i < MP_BUDDY_NODES; i++) {
        b->split[i] = 0;
        b->used[i] = 0;
    }
    b->total_size = size;
    b->alloc_count = 0;
}

static int mp_buddy_find(mp_buddy_t *b, int node, int level, int target) {
    if (node >= MP_BUDDY_NODES) return -1;
    if (level == target) {
        if (!b->used[node] && !b->split[node]) {
            b->used[node] = 1;
            return node;
        }
        return -1;
    }
    int left = 2 * node + 1;
    int right = 2 * node + 2;
    if (left >= MP_BUDDY_NODES) return -1;
    if (!b->split[node]) {
        b->split[node] = 1;
    }
    int r = mp_buddy_find(b, left, level + 1, target);
    if (r >= 0) return r;
    return mp_buddy_find(b, right, level + 1, target);
}

int mp_buddy_alloc(mp_buddy_t *b, int level) {
    if (level < 0 || level >= MP_BUDDY_LEVELS) return -1;
    int node = mp_buddy_find(b, 0, 0, level);
    if (node >= 0) b->alloc_count++;
    return node;
}

void mp_buddy_free(mp_buddy_t *b, int node) {
    if (node >= 0 && node < MP_BUDDY_NODES && b->used[node]) {
        b->used[node] = 0;
        b->alloc_count--;
    }
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1667: Buddy allocator should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1667: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1667: Should contain mp_ functions");
}

#[test]
fn c1668_segregated_fits() {
    let c_code = r##"
typedef unsigned long size_t;

#define MP_SEG_CLASSES 5
#define MP_SEG_PER_CLASS 16

typedef struct {
    int free_indices[MP_SEG_PER_CLASS];
    int count;
    size_t min_size;
    size_t max_size;
} mp_seg_class_t;

typedef struct {
    mp_seg_class_t classes[MP_SEG_CLASSES];
    int total_allocs;
    int total_frees;
} mp_segregated_t;

void mp_segregated_init(mp_segregated_t *s) {
    int c;
    size_t sizes[5] = {16, 32, 64, 128, 256};
    for (c = 0; c < MP_SEG_CLASSES; c++) {
        s->classes[c].count = 0;
        s->classes[c].min_size = (c == 0) ? 1 : sizes[c - 1] + 1;
        s->classes[c].max_size = sizes[c];
    }
    s->total_allocs = 0;
    s->total_frees = 0;
}

int mp_segregated_find_class(mp_segregated_t *s, size_t size) {
    int c;
    for (c = 0; c < MP_SEG_CLASSES; c++) {
        if (size <= s->classes[c].max_size) return c;
    }
    return -1;
}

int mp_segregated_alloc(mp_segregated_t *s, size_t size) {
    int c = mp_segregated_find_class(s, size);
    if (c < 0) return -1;
    if (s->classes[c].count > 0) {
        s->classes[c].count--;
        s->total_allocs++;
        return s->classes[c].free_indices[s->classes[c].count];
    }
    s->total_allocs++;
    return c * 1000 + s->total_allocs;
}

void mp_segregated_free(mp_segregated_t *s, int handle, size_t size) {
    int c = mp_segregated_find_class(s, size);
    if (c >= 0 && s->classes[c].count < MP_SEG_PER_CLASS) {
        s->classes[c].free_indices[s->classes[c].count++] = handle;
        s->total_frees++;
    }
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1668: Segregated fits should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1668: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1668: Should contain mp_ functions");
}

#[test]
fn c1669_tlsf_allocator() {
    let c_code = r##"
typedef unsigned long size_t;

#define MP_TLSF_FL_COUNT 8
#define MP_TLSF_SL_COUNT 4
#define MP_TLSF_MAX_BLOCKS 32

typedef struct {
    size_t size;
    int free;
    int fl;
    int sl;
} mp_tlsf_block_t;

typedef struct {
    int fl_bitmap;
    int sl_bitmap[MP_TLSF_FL_COUNT];
    int heads[MP_TLSF_FL_COUNT][MP_TLSF_SL_COUNT];
    mp_tlsf_block_t blocks[MP_TLSF_MAX_BLOCKS];
    int block_count;
} mp_tlsf_t;

void mp_tlsf_init(mp_tlsf_t *t) {
    int i, j;
    t->fl_bitmap = 0;
    for (i = 0; i < MP_TLSF_FL_COUNT; i++) {
        t->sl_bitmap[i] = 0;
        for (j = 0; j < MP_TLSF_SL_COUNT; j++) {
            t->heads[i][j] = -1;
        }
    }
    t->block_count = 0;
}

void mp_tlsf_mapping(size_t size, int *fl, int *sl) {
    int f = 0;
    size_t tmp = size;
    while (tmp > 1) { tmp >>= 1; f++; }
    *fl = (f < MP_TLSF_FL_COUNT) ? f : MP_TLSF_FL_COUNT - 1;
    *sl = (int)((size >> (*fl > 1 ? *fl - 1 : 0)) & (MP_TLSF_SL_COUNT - 1));
}

int mp_tlsf_insert(mp_tlsf_t *t, size_t size) {
    if (t->block_count >= MP_TLSF_MAX_BLOCKS) return -1;
    int idx = t->block_count++;
    int fl, sl;
    mp_tlsf_mapping(size, &fl, &sl);
    t->blocks[idx].size = size;
    t->blocks[idx].free = 1;
    t->blocks[idx].fl = fl;
    t->blocks[idx].sl = sl;
    t->fl_bitmap |= (1 << fl);
    t->sl_bitmap[fl] |= (1 << sl);
    t->heads[fl][sl] = idx;
    return idx;
}

void mp_tlsf_remove(mp_tlsf_t *t, int idx) {
    if (idx >= 0 && idx < t->block_count) {
        t->blocks[idx].free = 0;
    }
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1669: TLSF allocator should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1669: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1669: Should contain mp_ functions");
}

#[test]
fn c1670_zone_allocator() {
    let c_code = r##"
typedef unsigned long size_t;

#define MP_ZONE_SIZE 2048
#define MP_ZONE_MAX 8

typedef struct {
    char data[MP_ZONE_SIZE];
    size_t used;
    int tag;
} mp_zone_block_t;

typedef struct {
    mp_zone_block_t zones[MP_ZONE_MAX];
    int zone_count;
    int active_zone;
} mp_zone_alloc_t;

void mp_zone_alloc_init(mp_zone_alloc_t *z) {
    z->zone_count = 1;
    z->active_zone = 0;
    z->zones[0].used = 0;
    z->zones[0].tag = 0;
}

void *mp_zone_alloc_get(mp_zone_alloc_t *z, size_t size, int tag) {
    size_t aligned = (size + 7) & ~((size_t)7);
    if (aligned > MP_ZONE_SIZE) return (void *)0;
    if (z->zones[z->active_zone].used + aligned > MP_ZONE_SIZE) {
        if (z->zone_count >= MP_ZONE_MAX) return (void *)0;
        z->active_zone = z->zone_count;
        z->zones[z->active_zone].used = 0;
        z->zones[z->active_zone].tag = tag;
        z->zone_count++;
    }
    void *p = &z->zones[z->active_zone].data[z->zones[z->active_zone].used];
    z->zones[z->active_zone].used += aligned;
    return p;
}

void mp_zone_alloc_free_tag(mp_zone_alloc_t *z, int tag) {
    int i;
    for (i = 0; i < z->zone_count; i++) {
        if (z->zones[i].tag == tag) {
            z->zones[i].used = 0;
        }
    }
}

size_t mp_zone_alloc_total_used(mp_zone_alloc_t *z) {
    size_t total = 0;
    int i;
    for (i = 0; i < z->zone_count; i++) {
        total += z->zones[i].used;
    }
    return total;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1670: Zone allocator should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1670: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1670: Should contain mp_ functions");
}

// ============================================================================
// C1671-C1675: Memory Tracking
// ============================================================================

#[test]
fn c1671_allocation_tracking() {
    let c_code = r##"
typedef unsigned long size_t;

#define MP_TRACK_MAX 128

typedef struct {
    size_t size;
    int active;
    int alloc_order;
    int source_line;
} mp_track_entry_t;

typedef struct {
    mp_track_entry_t entries[MP_TRACK_MAX];
    int count;
    int next_order;
    size_t total_allocated;
    size_t current_allocated;
    size_t peak_allocated;
} mp_tracker_t;

void mp_tracker_init(mp_tracker_t *t) {
    t->count = 0;
    t->next_order = 0;
    t->total_allocated = 0;
    t->current_allocated = 0;
    t->peak_allocated = 0;
}

int mp_tracker_record_alloc(mp_tracker_t *t, size_t size, int line) {
    if (t->count >= MP_TRACK_MAX) return -1;
    int idx = t->count++;
    t->entries[idx].size = size;
    t->entries[idx].active = 1;
    t->entries[idx].alloc_order = t->next_order++;
    t->entries[idx].source_line = line;
    t->total_allocated += size;
    t->current_allocated += size;
    if (t->current_allocated > t->peak_allocated) {
        t->peak_allocated = t->current_allocated;
    }
    return idx;
}

int mp_tracker_record_free(mp_tracker_t *t, int idx) {
    if (idx < 0 || idx >= t->count) return -1;
    if (!t->entries[idx].active) return -2;
    t->entries[idx].active = 0;
    t->current_allocated -= t->entries[idx].size;
    return 0;
}

int mp_tracker_active_count(mp_tracker_t *t) {
    int count = 0;
    int i;
    for (i = 0; i < t->count; i++) {
        if (t->entries[i].active) count++;
    }
    return count;
}

size_t mp_tracker_peak(mp_tracker_t *t) {
    return t->peak_allocated;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1671: Allocation tracking should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1671: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1671: Should contain mp_ functions");
}

#[test]
fn c1672_leak_detection() {
    let c_code = r##"
typedef unsigned long size_t;

#define MP_LEAK_MAX 64

typedef struct {
    size_t size;
    int active;
    int tag;
} mp_leak_entry_t;

typedef struct {
    mp_leak_entry_t entries[MP_LEAK_MAX];
    int count;
    int leak_check_enabled;
    int leaks_found;
} mp_leak_detector_t;

void mp_leak_detector_init(mp_leak_detector_t *ld) {
    ld->count = 0;
    ld->leak_check_enabled = 1;
    ld->leaks_found = 0;
}

int mp_leak_record_alloc(mp_leak_detector_t *ld, size_t size, int tag) {
    if (ld->count >= MP_LEAK_MAX) return -1;
    int idx = ld->count++;
    ld->entries[idx].size = size;
    ld->entries[idx].active = 1;
    ld->entries[idx].tag = tag;
    return idx;
}

int mp_leak_record_free(mp_leak_detector_t *ld, int idx) {
    if (idx < 0 || idx >= ld->count) return -1;
    if (!ld->entries[idx].active) return -2;
    ld->entries[idx].active = 0;
    return 0;
}

int mp_leak_check(mp_leak_detector_t *ld) {
    if (!ld->leak_check_enabled) return 0;
    int leaks = 0;
    int i;
    for (i = 0; i < ld->count; i++) {
        if (ld->entries[i].active) {
            leaks++;
        }
    }
    ld->leaks_found = leaks;
    return leaks;
}

size_t mp_leak_total_leaked_bytes(mp_leak_detector_t *ld) {
    size_t total = 0;
    int i;
    for (i = 0; i < ld->count; i++) {
        if (ld->entries[i].active) {
            total += ld->entries[i].size;
        }
    }
    return total;
}

void mp_leak_disable(mp_leak_detector_t *ld) {
    ld->leak_check_enabled = 0;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1672: Leak detection should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1672: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1672: Should contain mp_ functions");
}

#[test]
fn c1673_double_free_detection() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define MP_DBLF_MAX 64
#define MP_DBLF_MAGIC_ACTIVE 0xABCD1234
#define MP_DBLF_MAGIC_FREED 0xDEAD5678

typedef struct {
    uint32_t magic;
    size_t size;
    int state;
} mp_dblf_entry_t;

typedef struct {
    mp_dblf_entry_t entries[MP_DBLF_MAX];
    int count;
    int double_free_count;
    int corruption_count;
} mp_dblf_detector_t;

void mp_dblf_init(mp_dblf_detector_t *d) {
    d->count = 0;
    d->double_free_count = 0;
    d->corruption_count = 0;
}

int mp_dblf_alloc(mp_dblf_detector_t *d, size_t size) {
    if (d->count >= MP_DBLF_MAX) return -1;
    int idx = d->count++;
    d->entries[idx].magic = MP_DBLF_MAGIC_ACTIVE;
    d->entries[idx].size = size;
    d->entries[idx].state = 1;
    return idx;
}

int mp_dblf_free(mp_dblf_detector_t *d, int idx) {
    if (idx < 0 || idx >= d->count) return -1;
    if (d->entries[idx].magic != MP_DBLF_MAGIC_ACTIVE &&
        d->entries[idx].magic != MP_DBLF_MAGIC_FREED) {
        d->corruption_count++;
        return -3;
    }
    if (d->entries[idx].state == 0) {
        d->double_free_count++;
        return -2;
    }
    d->entries[idx].magic = MP_DBLF_MAGIC_FREED;
    d->entries[idx].state = 0;
    return 0;
}

int mp_dblf_get_double_frees(mp_dblf_detector_t *d) {
    return d->double_free_count;
}

int mp_dblf_get_corruptions(mp_dblf_detector_t *d) {
    return d->corruption_count;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1673: Double-free detection should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1673: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1673: Should contain mp_ functions");
}

#[test]
fn c1674_use_after_free_guard() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define MP_UAF_MAX 48
#define MP_UAF_CANARY 0xFEEDFACE

typedef struct {
    uint32_t canary;
    int alive;
    size_t size;
    int generation;
} mp_uaf_slot_t;

typedef struct {
    mp_uaf_slot_t slots[MP_UAF_MAX];
    int count;
    int current_gen;
    int uaf_violations;
} mp_uaf_guard_t;

void mp_uaf_guard_init(mp_uaf_guard_t *g) {
    g->count = 0;
    g->current_gen = 1;
    g->uaf_violations = 0;
}

int mp_uaf_alloc(mp_uaf_guard_t *g, size_t size) {
    if (g->count >= MP_UAF_MAX) return -1;
    int idx = g->count++;
    g->slots[idx].canary = MP_UAF_CANARY;
    g->slots[idx].alive = 1;
    g->slots[idx].size = size;
    g->slots[idx].generation = g->current_gen;
    return idx;
}

int mp_uaf_free(mp_uaf_guard_t *g, int idx) {
    if (idx < 0 || idx >= g->count) return -1;
    if (!g->slots[idx].alive) {
        g->uaf_violations++;
        return -2;
    }
    g->slots[idx].alive = 0;
    g->slots[idx].canary = 0;
    g->current_gen++;
    return 0;
}

int mp_uaf_check_access(mp_uaf_guard_t *g, int idx) {
    if (idx < 0 || idx >= g->count) return -1;
    if (g->slots[idx].canary != MP_UAF_CANARY) {
        g->uaf_violations++;
        return -3;
    }
    if (!g->slots[idx].alive) {
        g->uaf_violations++;
        return -2;
    }
    return 0;
}

int mp_uaf_violations(mp_uaf_guard_t *g) {
    return g->uaf_violations;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1674: Use-after-free guard should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1674: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1674: Should contain mp_ functions");
}

#[test]
fn c1675_memory_profiler() {
    let c_code = r##"
typedef unsigned long size_t;

#define MP_PROF_BUCKETS 8
#define MP_PROF_HISTORY 64

typedef struct {
    size_t min_size;
    size_t max_size;
    int alloc_count;
    size_t total_bytes;
} mp_prof_bucket_t;

typedef struct {
    size_t size;
    int is_alloc;
    int bucket;
} mp_prof_event_t;

typedef struct {
    mp_prof_bucket_t buckets[MP_PROF_BUCKETS];
    mp_prof_event_t history[MP_PROF_HISTORY];
    int history_count;
    size_t current_usage;
    size_t peak_usage;
    int total_ops;
} mp_profiler_t;

void mp_profiler_init(mp_profiler_t *p) {
    int i;
    size_t sz = 8;
    for (i = 0; i < MP_PROF_BUCKETS; i++) {
        p->buckets[i].min_size = (i == 0) ? 0 : sz / 2 + 1;
        p->buckets[i].max_size = sz;
        p->buckets[i].alloc_count = 0;
        p->buckets[i].total_bytes = 0;
        sz = sz * 4;
    }
    p->history_count = 0;
    p->current_usage = 0;
    p->peak_usage = 0;
    p->total_ops = 0;
}

int mp_profiler_find_bucket(mp_profiler_t *p, size_t size) {
    int i;
    for (i = 0; i < MP_PROF_BUCKETS; i++) {
        if (size <= p->buckets[i].max_size) return i;
    }
    return MP_PROF_BUCKETS - 1;
}

void mp_profiler_record_alloc(mp_profiler_t *p, size_t size) {
    int b = mp_profiler_find_bucket(p, size);
    p->buckets[b].alloc_count++;
    p->buckets[b].total_bytes += size;
    p->current_usage += size;
    if (p->current_usage > p->peak_usage) {
        p->peak_usage = p->current_usage;
    }
    if (p->history_count < MP_PROF_HISTORY) {
        p->history[p->history_count].size = size;
        p->history[p->history_count].is_alloc = 1;
        p->history[p->history_count].bucket = b;
        p->history_count++;
    }
    p->total_ops++;
}

void mp_profiler_record_free(mp_profiler_t *p, size_t size) {
    if (p->current_usage >= size) {
        p->current_usage -= size;
    }
    if (p->history_count < MP_PROF_HISTORY) {
        int b = mp_profiler_find_bucket(p, size);
        p->history[p->history_count].size = size;
        p->history[p->history_count].is_alloc = 0;
        p->history[p->history_count].bucket = b;
        p->history_count++;
    }
    p->total_ops++;
}

int mp_profiler_hottest_bucket(mp_profiler_t *p) {
    int hot = 0;
    int i;
    for (i = 1; i < MP_PROF_BUCKETS; i++) {
        if (p->buckets[i].alloc_count > p->buckets[hot].alloc_count) {
            hot = i;
        }
    }
    return hot;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1675: Memory profiler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1675: Output should not be empty");
    assert!(code.contains("fn mp_"), "C1675: Should contain mp_ functions");
}
