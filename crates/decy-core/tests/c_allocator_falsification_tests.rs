//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1326-C1350: Memory Allocator domain -- bump/arena, pool, free-list, slab,
//! buddy system, TLSF, bitmap, and memory management utility patterns.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise memory allocator patterns commonly found in
//! operating systems, game engines, embedded runtimes, and high-performance
//! systems -- all expressed as valid C99 without #include.
//!
//! Organization:
//! - C1326-C1330: Simple allocators (bump, stack, pool, linear, scratch)
//! - C1331-C1335: Free-list allocators (first-fit, best-fit, worst-fit, next-fit, segregated)
//! - C1336-C1340: Slab allocators (fixed-size, slab cache, magazine, per-CPU, coloring)
//! - C1341-C1345: Advanced allocators (buddy, TLSF, bitmap, rbtree, coalescing)
//! - C1346-C1350: Memory utilities (alignment, guard/canary, zeroing, tracking, pool resize)
//!
//! Results: 24 passing, 1 falsified (96.0% pass rate)
//! Falsified: C1343 (bitmap allocator - HIR panics on nested for inside if with hex constant)

use decy_core::transpile;

// ============================================================================
// C1326-C1330: Simple Allocators
// ============================================================================

#[test]
fn c1326_bump_allocator() {
    let c_code = r#"
typedef unsigned long size_t;
typedef struct {
    char buffer[4096];
    size_t offset;
} alloc_bump_t;

void alloc_bump_init(alloc_bump_t *a) {
    a->offset = 0;
}

void *alloc_bump_alloc(alloc_bump_t *a, size_t size) {
    if (a->offset + size > 4096) return (void *)0;
    void *p = &a->buffer[a->offset];
    a->offset += size;
    return p;
}

void alloc_bump_reset(alloc_bump_t *a) {
    a->offset = 0;
}

size_t alloc_bump_remaining(alloc_bump_t *a) {
    return 4096 - a->offset;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1326 failed: {:?}", result.err());
}

#[test]
fn c1327_stack_allocator() {
    let c_code = r#"
typedef unsigned long size_t;
typedef struct {
    char buffer[2048];
    size_t top;
    size_t markers[32];
    int marker_count;
} alloc_stack_t;

void alloc_stack_init(alloc_stack_t *s) {
    s->top = 0;
    s->marker_count = 0;
}

void *alloc_stack_push(alloc_stack_t *s, size_t size) {
    if (s->top + size > 2048) return (void *)0;
    void *p = &s->buffer[s->top];
    s->top += size;
    return p;
}

int alloc_stack_save(alloc_stack_t *s) {
    if (s->marker_count >= 32) return -1;
    s->markers[s->marker_count] = s->top;
    return s->marker_count++;
}

void alloc_stack_restore(alloc_stack_t *s, int marker) {
    if (marker >= 0 && marker < s->marker_count) {
        s->top = s->markers[marker];
        s->marker_count = marker;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1327 failed: {:?}", result.err());
}

#[test]
fn c1328_pool_allocator() {
    let c_code = r#"
typedef unsigned long size_t;
#define ALLOC_POOL_SLOTS 64
#define ALLOC_SLOT_SIZE 128

typedef struct {
    char storage[ALLOC_POOL_SLOTS * ALLOC_SLOT_SIZE];
    int free_bitmap[ALLOC_POOL_SLOTS];
    int allocated_count;
} alloc_pool_t;

void alloc_pool_init(alloc_pool_t *p) {
    int i;
    for (i = 0; i < ALLOC_POOL_SLOTS; i++) p->free_bitmap[i] = 1;
    p->allocated_count = 0;
}

void *alloc_pool_get(alloc_pool_t *p) {
    int i;
    for (i = 0; i < ALLOC_POOL_SLOTS; i++) {
        if (p->free_bitmap[i]) {
            p->free_bitmap[i] = 0;
            p->allocated_count++;
            return &p->storage[i * ALLOC_SLOT_SIZE];
        }
    }
    return (void *)0;
}

void alloc_pool_put(alloc_pool_t *p, void *ptr) {
    char *base = p->storage;
    char *cp = (char *)ptr;
    int idx = (int)(cp - base) / ALLOC_SLOT_SIZE;
    if (idx >= 0 && idx < ALLOC_POOL_SLOTS) {
        p->free_bitmap[idx] = 1;
        p->allocated_count--;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1328 failed: {:?}", result.err());
}

#[test]
fn c1329_linear_allocator() {
    let c_code = r#"
typedef unsigned long size_t;
typedef struct {
    char memory[8192];
    size_t used;
    size_t peak;
    int alloc_count;
} alloc_linear_t;

void alloc_linear_init(alloc_linear_t *a) {
    a->used = 0;
    a->peak = 0;
    a->alloc_count = 0;
}

void *alloc_linear_alloc(alloc_linear_t *a, size_t size) {
    size_t aligned = (size + 7) & ~((size_t)7);
    if (a->used + aligned > 8192) return (void *)0;
    void *p = &a->memory[a->used];
    a->used += aligned;
    if (a->used > a->peak) a->peak = a->used;
    a->alloc_count++;
    return p;
}

void alloc_linear_reset(alloc_linear_t *a) {
    a->used = 0;
    a->alloc_count = 0;
}

size_t alloc_linear_peak_usage(alloc_linear_t *a) {
    return a->peak;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1329 failed: {:?}", result.err());
}

#[test]
fn c1330_scratch_allocator() {
    let c_code = r#"
typedef unsigned long size_t;
typedef struct {
    char buf[1024];
    size_t head;
    size_t tail;
} alloc_scratch_t;

void alloc_scratch_init(alloc_scratch_t *s) {
    s->head = 0;
    s->tail = 1024;
}

void *alloc_scratch_front(alloc_scratch_t *s, size_t size) {
    if (s->head + size > s->tail) return (void *)0;
    void *p = &s->buf[s->head];
    s->head += size;
    return p;
}

void *alloc_scratch_back(alloc_scratch_t *s, size_t size) {
    if (s->tail < s->head + size) return (void *)0;
    s->tail -= size;
    return &s->buf[s->tail];
}

void alloc_scratch_reset(alloc_scratch_t *s) {
    s->head = 0;
    s->tail = 1024;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1330 failed: {:?}", result.err());
}

// ============================================================================
// C1331-C1335: Free-List Allocators
// ============================================================================

#[test]
fn c1331_first_fit_free_list() {
    let c_code = r#"
typedef unsigned long size_t;
typedef struct alloc_ff_node {
    size_t size;
    int used;
    int next_idx;
} alloc_ff_node_t;

#define ALLOC_FF_MAX 128

static alloc_ff_node_t alloc_ff_nodes[ALLOC_FF_MAX];
static int alloc_ff_count = 0;

void alloc_ff_init(void) {
    alloc_ff_nodes[0].size = 65536;
    alloc_ff_nodes[0].used = 0;
    alloc_ff_nodes[0].next_idx = -1;
    alloc_ff_count = 1;
}

int alloc_ff_alloc(size_t size) {
    int i;
    for (i = 0; i < alloc_ff_count; i++) {
        if (!alloc_ff_nodes[i].used && alloc_ff_nodes[i].size >= size) {
            if (alloc_ff_nodes[i].size > size + 16 && alloc_ff_count < ALLOC_FF_MAX) {
                alloc_ff_nodes[alloc_ff_count].size = alloc_ff_nodes[i].size - size;
                alloc_ff_nodes[alloc_ff_count].used = 0;
                alloc_ff_nodes[alloc_ff_count].next_idx = alloc_ff_nodes[i].next_idx;
                alloc_ff_nodes[i].next_idx = alloc_ff_count;
                alloc_ff_count++;
                alloc_ff_nodes[i].size = size;
            }
            alloc_ff_nodes[i].used = 1;
            return i;
        }
    }
    return -1;
}

void alloc_ff_free(int idx) {
    if (idx >= 0 && idx < alloc_ff_count)
        alloc_ff_nodes[idx].used = 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1331 failed: {:?}", result.err());
}

#[test]
fn c1332_best_fit_free_list() {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    size_t size;
    int free;
} alloc_bf_block_t;

#define ALLOC_BF_MAX 64
static alloc_bf_block_t alloc_bf_blocks[ALLOC_BF_MAX];
static int alloc_bf_count = 0;

void alloc_bf_init(void) {
    alloc_bf_blocks[0].size = 32768;
    alloc_bf_blocks[0].free = 1;
    alloc_bf_count = 1;
}

int alloc_bf_alloc(size_t size) {
    int best = -1;
    size_t best_size = (size_t)-1;
    int i;
    for (i = 0; i < alloc_bf_count; i++) {
        if (alloc_bf_blocks[i].free && alloc_bf_blocks[i].size >= size) {
            if (alloc_bf_blocks[i].size < best_size) {
                best_size = alloc_bf_blocks[i].size;
                best = i;
            }
        }
    }
    if (best >= 0) alloc_bf_blocks[best].free = 0;
    return best;
}

void alloc_bf_free(int idx) {
    if (idx >= 0 && idx < alloc_bf_count)
        alloc_bf_blocks[idx].free = 1;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1332 failed: {:?}", result.err());
}

#[test]
fn c1333_worst_fit_free_list() {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    size_t size;
    int free;
} alloc_wf_block_t;

#define ALLOC_WF_MAX 64
static alloc_wf_block_t alloc_wf_blocks[ALLOC_WF_MAX];
static int alloc_wf_count = 0;

void alloc_wf_init(void) {
    alloc_wf_blocks[0].size = 32768;
    alloc_wf_blocks[0].free = 1;
    alloc_wf_count = 1;
}

int alloc_wf_alloc(size_t size) {
    int worst = -1;
    size_t worst_size = 0;
    int i;
    for (i = 0; i < alloc_wf_count; i++) {
        if (alloc_wf_blocks[i].free && alloc_wf_blocks[i].size >= size) {
            if (alloc_wf_blocks[i].size > worst_size) {
                worst_size = alloc_wf_blocks[i].size;
                worst = i;
            }
        }
    }
    if (worst >= 0) alloc_wf_blocks[worst].free = 0;
    return worst;
}

void alloc_wf_free(int idx) {
    if (idx >= 0 && idx < alloc_wf_count)
        alloc_wf_blocks[idx].free = 1;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1333 failed: {:?}", result.err());
}

#[test]
fn c1334_next_fit_free_list() {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    size_t size;
    int free;
} alloc_nf_block_t;

#define ALLOC_NF_MAX 64
static alloc_nf_block_t alloc_nf_blocks[ALLOC_NF_MAX];
static int alloc_nf_count = 0;
static int alloc_nf_last = 0;

void alloc_nf_init(void) {
    alloc_nf_blocks[0].size = 32768;
    alloc_nf_blocks[0].free = 1;
    alloc_nf_count = 1;
    alloc_nf_last = 0;
}

int alloc_nf_alloc(size_t size) {
    int i, idx;
    for (i = 0; i < alloc_nf_count; i++) {
        idx = (alloc_nf_last + i) % alloc_nf_count;
        if (alloc_nf_blocks[idx].free && alloc_nf_blocks[idx].size >= size) {
            alloc_nf_blocks[idx].free = 0;
            alloc_nf_last = (idx + 1) % alloc_nf_count;
            return idx;
        }
    }
    return -1;
}

void alloc_nf_free(int idx) {
    if (idx >= 0 && idx < alloc_nf_count) {
        alloc_nf_blocks[idx].free = 1;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1334 failed: {:?}", result.err());
}

#[test]
fn c1335_segregated_free_list() {
    let c_code = r#"
typedef unsigned long size_t;
#define ALLOC_SEG_CLASSES 4
#define ALLOC_SEG_PER_CLASS 16

typedef struct {
    int free_indices[ALLOC_SEG_PER_CLASS];
    int count;
    size_t block_size;
} alloc_seg_class_t;

static alloc_seg_class_t alloc_seg_classes[ALLOC_SEG_CLASSES];

void alloc_seg_init(void) {
    size_t sizes[4] = {32, 64, 128, 256};
    int c;
    for (c = 0; c < ALLOC_SEG_CLASSES; c++) {
        alloc_seg_classes[c].block_size = sizes[c];
        alloc_seg_classes[c].count = 0;
    }
}

int alloc_seg_find_class(size_t size) {
    int c;
    for (c = 0; c < ALLOC_SEG_CLASSES; c++) {
        if (alloc_seg_classes[c].block_size >= size) return c;
    }
    return -1;
}

int alloc_seg_alloc(size_t size) {
    int c = alloc_seg_find_class(size);
    if (c < 0) return -1;
    if (alloc_seg_classes[c].count > 0) {
        alloc_seg_classes[c].count--;
        return alloc_seg_classes[c].free_indices[alloc_seg_classes[c].count];
    }
    return c * 1000;
}

void alloc_seg_free(int idx, size_t size) {
    int c = alloc_seg_find_class(size);
    if (c >= 0 && alloc_seg_classes[c].count < ALLOC_SEG_PER_CLASS) {
        alloc_seg_classes[c].free_indices[alloc_seg_classes[c].count++] = idx;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1335 failed: {:?}", result.err());
}

// ============================================================================
// C1336-C1340: Slab Allocators
// ============================================================================

#[test]
fn c1336_fixed_size_slab() {
    let c_code = r#"
typedef unsigned long size_t;
#define ALLOC_SLAB_OBJ_SIZE 64
#define ALLOC_SLAB_COUNT 32

typedef struct {
    char objects[ALLOC_SLAB_COUNT * ALLOC_SLAB_OBJ_SIZE];
    int freelist[ALLOC_SLAB_COUNT];
    int free_top;
    int active;
} alloc_slab_t;

void alloc_slab_init(alloc_slab_t *s) {
    int i;
    for (i = 0; i < ALLOC_SLAB_COUNT; i++) s->freelist[i] = i;
    s->free_top = ALLOC_SLAB_COUNT;
    s->active = 0;
}

void *alloc_slab_get(alloc_slab_t *s) {
    if (s->free_top <= 0) return (void *)0;
    s->free_top--;
    int idx = s->freelist[s->free_top];
    s->active++;
    return &s->objects[idx * ALLOC_SLAB_OBJ_SIZE];
}

void alloc_slab_put(alloc_slab_t *s, void *ptr) {
    char *base = s->objects;
    int idx = (int)((char *)ptr - base) / ALLOC_SLAB_OBJ_SIZE;
    if (idx >= 0 && idx < ALLOC_SLAB_COUNT) {
        s->freelist[s->free_top++] = idx;
        s->active--;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1336 failed: {:?}", result.err());
}

#[test]
fn c1337_slab_cache() {
    let c_code = r#"
typedef unsigned long size_t;
#define ALLOC_CACHE_SLABS 4
#define ALLOC_CACHE_OBJ_PER_SLAB 16
#define ALLOC_CACHE_OBJ_SIZE 48

typedef struct {
    char data[ALLOC_CACHE_OBJ_PER_SLAB * ALLOC_CACHE_OBJ_SIZE];
    int used[ALLOC_CACHE_OBJ_PER_SLAB];
    int used_count;
} alloc_cache_slab_t;

typedef struct {
    alloc_cache_slab_t slabs[ALLOC_CACHE_SLABS];
    int slab_count;
    size_t obj_size;
} alloc_cache_t;

void alloc_cache_init(alloc_cache_t *c) {
    int i, j;
    c->obj_size = ALLOC_CACHE_OBJ_SIZE;
    c->slab_count = 1;
    for (i = 0; i < ALLOC_CACHE_SLABS; i++) {
        for (j = 0; j < ALLOC_CACHE_OBJ_PER_SLAB; j++) c->slabs[i].used[j] = 0;
        c->slabs[i].used_count = 0;
    }
}

void *alloc_cache_alloc(alloc_cache_t *c) {
    int s, i;
    for (s = 0; s < c->slab_count; s++) {
        if (c->slabs[s].used_count < ALLOC_CACHE_OBJ_PER_SLAB) {
            for (i = 0; i < ALLOC_CACHE_OBJ_PER_SLAB; i++) {
                if (!c->slabs[s].used[i]) {
                    c->slabs[s].used[i] = 1;
                    c->slabs[s].used_count++;
                    return &c->slabs[s].data[i * ALLOC_CACHE_OBJ_SIZE];
                }
            }
        }
    }
    if (c->slab_count < ALLOC_CACHE_SLABS) {
        c->slab_count++;
        return alloc_cache_alloc(c);
    }
    return (void *)0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1337 failed: {:?}", result.err());
}

#[test]
fn c1338_magazine_layer() {
    let c_code = r#"
typedef unsigned long size_t;
#define ALLOC_MAG_SIZE 8

typedef struct {
    int slots[ALLOC_MAG_SIZE];
    int count;
} alloc_magazine_t;

typedef struct {
    alloc_magazine_t loaded;
    alloc_magazine_t previous;
    int next_id;
} alloc_mag_cache_t;

void alloc_mag_init(alloc_mag_cache_t *mc) {
    mc->loaded.count = 0;
    mc->previous.count = 0;
    mc->next_id = 1;
}

int alloc_mag_get(alloc_mag_cache_t *mc) {
    if (mc->loaded.count > 0) {
        mc->loaded.count--;
        return mc->loaded.slots[mc->loaded.count];
    }
    if (mc->previous.count > 0) {
        alloc_magazine_t tmp = mc->loaded;
        mc->loaded = mc->previous;
        mc->previous = tmp;
        mc->loaded.count--;
        return mc->loaded.slots[mc->loaded.count];
    }
    return mc->next_id++;
}

void alloc_mag_put(alloc_mag_cache_t *mc, int id) {
    if (mc->loaded.count < ALLOC_MAG_SIZE) {
        mc->loaded.slots[mc->loaded.count++] = id;
    } else if (mc->previous.count < ALLOC_MAG_SIZE) {
        mc->previous.slots[mc->previous.count++] = id;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1338 failed: {:?}", result.err());
}

#[test]
fn c1339_per_cpu_cache() {
    let c_code = r#"
typedef unsigned long size_t;
#define ALLOC_MAX_CPUS 4
#define ALLOC_CPU_CACHE_SIZE 16

typedef struct {
    int items[ALLOC_CPU_CACHE_SIZE];
    int count;
    int hits;
    int misses;
} alloc_cpu_cache_t;

static alloc_cpu_cache_t alloc_per_cpu[ALLOC_MAX_CPUS];
static int alloc_global_next = 1;

void alloc_cpu_init(void) {
    int c;
    for (c = 0; c < ALLOC_MAX_CPUS; c++) {
        alloc_per_cpu[c].count = 0;
        alloc_per_cpu[c].hits = 0;
        alloc_per_cpu[c].misses = 0;
    }
}

int alloc_cpu_get(int cpu_id) {
    if (cpu_id < 0 || cpu_id >= ALLOC_MAX_CPUS) return -1;
    alloc_cpu_cache_t *cache = &alloc_per_cpu[cpu_id];
    if (cache->count > 0) {
        cache->hits++;
        return cache->items[--cache->count];
    }
    cache->misses++;
    return alloc_global_next++;
}

void alloc_cpu_put(int cpu_id, int item) {
    if (cpu_id < 0 || cpu_id >= ALLOC_MAX_CPUS) return;
    alloc_cpu_cache_t *cache = &alloc_per_cpu[cpu_id];
    if (cache->count < ALLOC_CPU_CACHE_SIZE) {
        cache->items[cache->count++] = item;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1339 failed: {:?}", result.err());
}

#[test]
fn c1340_slab_coloring() {
    let c_code = r#"
typedef unsigned long size_t;
#define ALLOC_COLOR_SLOTS 16
#define ALLOC_COLOR_OBJ_SIZE 64
#define ALLOC_COLOR_LINE 64

typedef struct {
    char data[ALLOC_COLOR_SLOTS * (ALLOC_COLOR_OBJ_SIZE + ALLOC_COLOR_LINE)];
    int color_offset;
    int used[ALLOC_COLOR_SLOTS];
    int alloc_count;
} alloc_colored_slab_t;

static int alloc_color_next = 0;

void alloc_colored_init(alloc_colored_slab_t *s) {
    int i;
    s->color_offset = (alloc_color_next * ALLOC_COLOR_LINE) % (ALLOC_COLOR_LINE * 4);
    alloc_color_next++;
    for (i = 0; i < ALLOC_COLOR_SLOTS; i++) s->used[i] = 0;
    s->alloc_count = 0;
}

void *alloc_colored_get(alloc_colored_slab_t *s) {
    int i;
    for (i = 0; i < ALLOC_COLOR_SLOTS; i++) {
        if (!s->used[i]) {
            s->used[i] = 1;
            s->alloc_count++;
            int offset = s->color_offset + i * ALLOC_COLOR_OBJ_SIZE;
            return &s->data[offset];
        }
    }
    return (void *)0;
}

void alloc_colored_put(alloc_colored_slab_t *s, int idx) {
    if (idx >= 0 && idx < ALLOC_COLOR_SLOTS) {
        s->used[idx] = 0;
        s->alloc_count--;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1340 failed: {:?}", result.err());
}

// ============================================================================
// C1341-C1345: Advanced Allocators
// ============================================================================

#[test]
fn c1341_buddy_system_allocator() {
    let c_code = r#"
typedef unsigned long size_t;
#define ALLOC_BUDDY_LEVELS 5
#define ALLOC_BUDDY_NODES 63

typedef struct {
    int split[ALLOC_BUDDY_NODES];
    int used[ALLOC_BUDDY_NODES];
    size_t total_size;
} alloc_buddy_t;

void alloc_buddy_init(alloc_buddy_t *b, size_t size) {
    int i;
    for (i = 0; i < ALLOC_BUDDY_NODES; i++) {
        b->split[i] = 0;
        b->used[i] = 0;
    }
    b->total_size = size;
}

static int alloc_buddy_find(alloc_buddy_t *b, int node, int level, int target) {
    if (level == target) {
        if (!b->used[node] && !b->split[node]) {
            b->used[node] = 1;
            return node;
        }
        return -1;
    }
    int left = 2 * node + 1;
    int right = 2 * node + 2;
    if (!b->split[node]) {
        b->split[node] = 1;
        b->used[left] = 0;
        b->used[right] = 0;
    }
    int r = alloc_buddy_find(b, left, level + 1, target);
    if (r >= 0) return r;
    return alloc_buddy_find(b, right, level + 1, target);
}

int alloc_buddy_alloc(alloc_buddy_t *b, int level) {
    if (level < 0 || level >= ALLOC_BUDDY_LEVELS) return -1;
    return alloc_buddy_find(b, 0, 0, level);
}

void alloc_buddy_free(alloc_buddy_t *b, int node) {
    if (node >= 0 && node < ALLOC_BUDDY_NODES) {
        b->used[node] = 0;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1341 failed: {:?}", result.err());
}

#[test]
fn c1342_tlsf_two_level_allocator() {
    let c_code = r#"
typedef unsigned long size_t;
#define ALLOC_TLSF_FL_COUNT 8
#define ALLOC_TLSF_SL_COUNT 4
#define ALLOC_TLSF_MAX_BLOCKS 64

typedef struct {
    size_t size;
    int free;
    int fl;
    int sl;
} alloc_tlsf_block_t;

typedef struct {
    int fl_bitmap;
    int sl_bitmap[ALLOC_TLSF_FL_COUNT];
    int heads[ALLOC_TLSF_FL_COUNT][ALLOC_TLSF_SL_COUNT];
    alloc_tlsf_block_t blocks[ALLOC_TLSF_MAX_BLOCKS];
    int block_count;
} alloc_tlsf_t;

void alloc_tlsf_init(alloc_tlsf_t *t) {
    int i, j;
    t->fl_bitmap = 0;
    for (i = 0; i < ALLOC_TLSF_FL_COUNT; i++) {
        t->sl_bitmap[i] = 0;
        for (j = 0; j < ALLOC_TLSF_SL_COUNT; j++) t->heads[i][j] = -1;
    }
    t->block_count = 0;
}

void alloc_tlsf_mapping(size_t size, int *fl, int *sl) {
    int f = 0;
    size_t tmp = size;
    while (tmp > 1) { tmp >>= 1; f++; }
    *fl = (f < ALLOC_TLSF_FL_COUNT) ? f : ALLOC_TLSF_FL_COUNT - 1;
    *sl = (int)((size >> (*fl > 1 ? *fl - 1 : 0)) & (ALLOC_TLSF_SL_COUNT - 1));
}

int alloc_tlsf_insert(alloc_tlsf_t *t, size_t size) {
    if (t->block_count >= ALLOC_TLSF_MAX_BLOCKS) return -1;
    int idx = t->block_count++;
    int fl, sl;
    alloc_tlsf_mapping(size, &fl, &sl);
    t->blocks[idx].size = size;
    t->blocks[idx].free = 1;
    t->blocks[idx].fl = fl;
    t->blocks[idx].sl = sl;
    t->fl_bitmap |= (1 << fl);
    t->sl_bitmap[fl] |= (1 << sl);
    t->heads[fl][sl] = idx;
    return idx;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1342 failed: {:?}", result.err());
}

#[test]
#[ignore = "FALSIFIED: HIR panics 'For loop must have condition' on nested for inside if with hex constant comparison"]
fn c1343_bitmap_allocator() {
    let c_code = r#"
typedef unsigned long size_t;
#define ALLOC_BM_BITS 256
#define ALLOC_BM_WORDS (ALLOC_BM_BITS / 32)

typedef struct {
    unsigned int bitmap[ALLOC_BM_WORDS];
    size_t block_size;
    int total_free;
} alloc_bitmap_t;

void alloc_bitmap_init(alloc_bitmap_t *b, size_t block_size) {
    int i;
    for (i = 0; i < ALLOC_BM_WORDS; i++) b->bitmap[i] = 0;
    b->block_size = block_size;
    b->total_free = ALLOC_BM_BITS;
}

int alloc_bitmap_alloc(alloc_bitmap_t *b) {
    int w, bit;
    for (w = 0; w < ALLOC_BM_WORDS; w++) {
        if (b->bitmap[w] != 0xFFFFFFFF) {
            for (bit = 0; bit < 32; bit++) {
                if (!(b->bitmap[w] & (1u << bit))) {
                    b->bitmap[w] |= (1u << bit);
                    b->total_free--;
                    return w * 32 + bit;
                }
            }
        }
    }
    return -1;
}

void alloc_bitmap_free(alloc_bitmap_t *b, int idx) {
    int w = idx / 32;
    int bit = idx % 32;
    if (w >= 0 && w < ALLOC_BM_WORDS) {
        b->bitmap[w] &= ~(1u << bit);
        b->total_free++;
    }
}

int alloc_bitmap_is_used(alloc_bitmap_t *b, int idx) {
    int w = idx / 32;
    int bit = idx % 32;
    return (b->bitmap[w] >> bit) & 1;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1343 failed: {:?}", result.err());
}

#[test]
fn c1344_rbtree_allocator() {
    let c_code = r#"
typedef unsigned long size_t;
#define ALLOC_RB_MAX 32
#define ALLOC_RB_RED 0
#define ALLOC_RB_BLACK 1

typedef struct {
    size_t size;
    int color;
    int left;
    int right;
    int parent;
    int free;
} alloc_rb_node_t;

typedef struct {
    alloc_rb_node_t nodes[ALLOC_RB_MAX];
    int count;
    int root;
} alloc_rb_tree_t;

void alloc_rb_init(alloc_rb_tree_t *t) {
    t->count = 0;
    t->root = -1;
}

int alloc_rb_insert(alloc_rb_tree_t *t, size_t size) {
    if (t->count >= ALLOC_RB_MAX) return -1;
    int idx = t->count++;
    t->nodes[idx].size = size;
    t->nodes[idx].color = ALLOC_RB_RED;
    t->nodes[idx].left = -1;
    t->nodes[idx].right = -1;
    t->nodes[idx].parent = -1;
    t->nodes[idx].free = 1;
    if (t->root < 0) {
        t->root = idx;
        t->nodes[idx].color = ALLOC_RB_BLACK;
    }
    return idx;
}

int alloc_rb_find_best(alloc_rb_tree_t *t, size_t size) {
    int best = -1;
    int i;
    for (i = 0; i < t->count; i++) {
        if (t->nodes[i].free && t->nodes[i].size >= size) {
            if (best < 0 || t->nodes[i].size < t->nodes[best].size)
                best = i;
        }
    }
    return best;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1344 failed: {:?}", result.err());
}

#[test]
fn c1345_memory_coalescing() {
    let c_code = r#"
typedef unsigned long size_t;
#define ALLOC_COAL_MAX 64

typedef struct {
    size_t offset;
    size_t size;
    int free;
} alloc_coal_block_t;

typedef struct {
    alloc_coal_block_t blocks[ALLOC_COAL_MAX];
    int count;
} alloc_coal_t;

void alloc_coal_init(alloc_coal_t *c, size_t total) {
    c->blocks[0].offset = 0;
    c->blocks[0].size = total;
    c->blocks[0].free = 1;
    c->count = 1;
}

void alloc_coal_coalesce(alloc_coal_t *c) {
    int i = 0;
    while (i < c->count - 1) {
        if (c->blocks[i].free && c->blocks[i + 1].free) {
            c->blocks[i].size += c->blocks[i + 1].size;
            int j;
            for (j = i + 1; j < c->count - 1; j++)
                c->blocks[j] = c->blocks[j + 1];
            c->count--;
        } else {
            i++;
        }
    }
}

int alloc_coal_alloc(alloc_coal_t *c, size_t size) {
    int i;
    for (i = 0; i < c->count; i++) {
        if (c->blocks[i].free && c->blocks[i].size >= size) {
            c->blocks[i].free = 0;
            if (c->blocks[i].size > size && c->count < ALLOC_COAL_MAX) {
                int j;
                for (j = c->count; j > i + 1; j--)
                    c->blocks[j] = c->blocks[j - 1];
                c->blocks[i + 1].offset = c->blocks[i].offset + size;
                c->blocks[i + 1].size = c->blocks[i].size - size;
                c->blocks[i + 1].free = 1;
                c->blocks[i].size = size;
                c->count++;
            }
            return i;
        }
    }
    return -1;
}

void alloc_coal_free(alloc_coal_t *c, int idx) {
    if (idx >= 0 && idx < c->count) {
        c->blocks[idx].free = 1;
        alloc_coal_coalesce(c);
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1345 failed: {:?}", result.err());
}

// ============================================================================
// C1346-C1350: Memory Management Utilities
// ============================================================================

#[test]
fn c1346_memory_alignment() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned long uintptr_t;

typedef struct {
    char buffer[4096];
    size_t offset;
} alloc_aligned_t;

void alloc_aligned_init(alloc_aligned_t *a) {
    a->offset = 0;
}

static size_t alloc_align_up(size_t val, size_t align) {
    return (val + align - 1) & ~(align - 1);
}

void *alloc_aligned_alloc(alloc_aligned_t *a, size_t size, size_t align) {
    uintptr_t base = (uintptr_t)&a->buffer[a->offset];
    uintptr_t aligned = (base + align - 1) & ~(align - 1);
    size_t padding = (size_t)(aligned - base);
    size_t total = padding + size;
    if (a->offset + total > 4096) return (void *)0;
    a->offset += total;
    return (void *)aligned;
}

int alloc_is_aligned(void *ptr, size_t align) {
    return ((uintptr_t)ptr & (align - 1)) == 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1346 failed: {:?}", result.err());
}

#[test]
fn c1347_guard_canary() {
    let c_code = r#"
typedef unsigned long size_t;
#define ALLOC_CANARY_VALUE 0xDEADC0DE
#define ALLOC_GUARD_SLOTS 32

typedef struct {
    unsigned int canary_head;
    char data[256];
    unsigned int canary_tail;
} alloc_guarded_block_t;

static alloc_guarded_block_t alloc_guard_pool[ALLOC_GUARD_SLOTS];
static int alloc_guard_used[ALLOC_GUARD_SLOTS];

void alloc_guard_init(void) {
    int i;
    for (i = 0; i < ALLOC_GUARD_SLOTS; i++) {
        alloc_guard_pool[i].canary_head = ALLOC_CANARY_VALUE;
        alloc_guard_pool[i].canary_tail = ALLOC_CANARY_VALUE;
        alloc_guard_used[i] = 0;
    }
}

int alloc_guard_get(void) {
    int i;
    for (i = 0; i < ALLOC_GUARD_SLOTS; i++) {
        if (!alloc_guard_used[i]) {
            alloc_guard_used[i] = 1;
            return i;
        }
    }
    return -1;
}

int alloc_guard_check(int idx) {
    if (idx < 0 || idx >= ALLOC_GUARD_SLOTS) return -1;
    if (alloc_guard_pool[idx].canary_head != ALLOC_CANARY_VALUE) return 1;
    if (alloc_guard_pool[idx].canary_tail != ALLOC_CANARY_VALUE) return 2;
    return 0;
}

void alloc_guard_put(int idx) {
    if (idx >= 0 && idx < ALLOC_GUARD_SLOTS) {
        alloc_guard_pool[idx].canary_head = ALLOC_CANARY_VALUE;
        alloc_guard_pool[idx].canary_tail = ALLOC_CANARY_VALUE;
        alloc_guard_used[idx] = 0;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1347 failed: {:?}", result.err());
}

#[test]
fn c1348_memory_zeroing() {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    char buffer[2048];
    size_t offset;
    int zero_on_free;
} alloc_zeroing_t;

void alloc_zeroing_init(alloc_zeroing_t *z, int secure) {
    z->offset = 0;
    z->zero_on_free = secure;
}

void alloc_memzero(void *ptr, size_t len) {
    volatile char *p = (volatile char *)ptr;
    size_t i;
    for (i = 0; i < len; i++) p[i] = 0;
}

void *alloc_zeroing_alloc(alloc_zeroing_t *z, size_t size) {
    if (z->offset + size > 2048) return (void *)0;
    void *p = &z->buffer[z->offset];
    alloc_memzero(p, size);
    z->offset += size;
    return p;
}

void alloc_zeroing_free_block(alloc_zeroing_t *z, void *ptr, size_t size) {
    if (z->zero_on_free) {
        alloc_memzero(ptr, size);
    }
}

void alloc_zeroing_reset(alloc_zeroing_t *z) {
    if (z->zero_on_free) {
        alloc_memzero(z->buffer, z->offset);
    }
    z->offset = 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1348 failed: {:?}", result.err());
}

#[test]
fn c1349_allocation_tracking_stats() {
    let c_code = r#"
typedef unsigned long size_t;
#define ALLOC_TRACK_MAX 64

typedef struct {
    size_t size;
    int active;
    int line;
} alloc_track_entry_t;

typedef struct {
    alloc_track_entry_t entries[ALLOC_TRACK_MAX];
    int count;
    size_t total_allocated;
    size_t total_freed;
    size_t peak_usage;
    size_t current_usage;
    int alloc_calls;
    int free_calls;
} alloc_tracker_t;

void alloc_tracker_init(alloc_tracker_t *t) {
    t->count = 0;
    t->total_allocated = 0;
    t->total_freed = 0;
    t->peak_usage = 0;
    t->current_usage = 0;
    t->alloc_calls = 0;
    t->free_calls = 0;
}

int alloc_tracker_record(alloc_tracker_t *t, size_t size, int line) {
    if (t->count >= ALLOC_TRACK_MAX) return -1;
    int idx = t->count++;
    t->entries[idx].size = size;
    t->entries[idx].active = 1;
    t->entries[idx].line = line;
    t->total_allocated += size;
    t->current_usage += size;
    t->alloc_calls++;
    if (t->current_usage > t->peak_usage) t->peak_usage = t->current_usage;
    return idx;
}

void alloc_tracker_free(alloc_tracker_t *t, int idx) {
    if (idx < 0 || idx >= t->count || !t->entries[idx].active) return;
    t->entries[idx].active = 0;
    t->total_freed += t->entries[idx].size;
    t->current_usage -= t->entries[idx].size;
    t->free_calls++;
}

int alloc_tracker_leaks(alloc_tracker_t *t) {
    int leaks = 0;
    int i;
    for (i = 0; i < t->count; i++) {
        if (t->entries[i].active) leaks++;
    }
    return leaks;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1349 failed: {:?}", result.err());
}

#[test]
fn c1350_memory_pool_resize() {
    let c_code = r#"
typedef unsigned long size_t;
#define ALLOC_RESIZE_INITIAL 16
#define ALLOC_RESIZE_MAX 128

typedef struct {
    int slots[ALLOC_RESIZE_MAX];
    int capacity;
    int used;
    int resize_count;
} alloc_resizable_pool_t;

void alloc_resize_init(alloc_resizable_pool_t *p) {
    p->capacity = ALLOC_RESIZE_INITIAL;
    p->used = 0;
    p->resize_count = 0;
}

int alloc_resize_grow(alloc_resizable_pool_t *p) {
    int new_cap = p->capacity * 2;
    if (new_cap > ALLOC_RESIZE_MAX) new_cap = ALLOC_RESIZE_MAX;
    if (new_cap <= p->capacity) return -1;
    p->capacity = new_cap;
    p->resize_count++;
    return new_cap;
}

int alloc_resize_get(alloc_resizable_pool_t *p, int value) {
    if (p->used >= p->capacity) {
        if (alloc_resize_grow(p) < 0) return -1;
    }
    p->slots[p->used] = value;
    return p->used++;
}

int alloc_resize_shrink(alloc_resizable_pool_t *p) {
    if (p->used > 0 && p->used <= p->capacity / 4 && p->capacity > ALLOC_RESIZE_INITIAL) {
        p->capacity = p->capacity / 2;
        if (p->capacity < ALLOC_RESIZE_INITIAL) p->capacity = ALLOC_RESIZE_INITIAL;
        p->resize_count++;
        return p->capacity;
    }
    return 0;
}

void alloc_resize_remove(alloc_resizable_pool_t *p, int idx) {
    if (idx >= 0 && idx < p->used) {
        int i;
        for (i = idx; i < p->used - 1; i++) p->slots[i] = p->slots[i + 1];
        p->used--;
        alloc_resize_shrink(p);
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1350 failed: {:?}", result.err());
}
