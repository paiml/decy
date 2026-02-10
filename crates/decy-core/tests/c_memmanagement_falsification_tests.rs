//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C726-C750: Memory Management domain -- custom allocators, garbage collectors,
//! reference counting, memory pools, and smart pointer patterns.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise memory management patterns commonly found in
//! high-performance runtimes, embedded systems, game engines, and
//! systems programming -- all expressed as valid C99 without #include.
//!
//! Organization:
//! - C726-C730: Custom allocators (malloc/free, slab, buddy, pool, arena)
//! - C731-C735: GC and lifecycle patterns (refcount, mark-sweep, copying GC, mmap sim, stack alloc)
//! - C736-C740: Cache and safety patterns (object cache, defrag, TLSF, leak detect, canary)
//! - C741-C745: Alignment and sharing (alignment, shared mem, COW, barrier, generational)
//! - C746-C750: Advanced patterns (object pool, bitfield pack, realloc, weak ref, unique_ptr)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C726-C730: Custom Allocators
// ============================================================================

#[test]
fn c726_custom_malloc_free_with_free_list() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_SIZE 4096
#define BLOCK_MAGIC 0xDEADBEEF

typedef struct mm_block {
    unsigned int magic;
    size_t size;
    int is_free;
    int next_offset;
} mm_block_t;

static char mm_heap[HEAP_SIZE];
static int mm_initialized = 0;

void mm_init(void) {
    mm_block_t *first = (mm_block_t *)mm_heap;
    first->magic = BLOCK_MAGIC;
    first->size = HEAP_SIZE - sizeof(mm_block_t);
    first->is_free = 1;
    first->next_offset = -1;
    mm_initialized = 1;
}

void *mm_alloc(size_t size) {
    if (!mm_initialized) mm_init();
    int offset = 0;
    while (offset >= 0 && (size_t)offset < HEAP_SIZE) {
        mm_block_t *block = (mm_block_t *)(mm_heap + offset);
        if (block->magic != BLOCK_MAGIC) return (void *)0;
        if (block->is_free && block->size >= size) {
            if (block->size > size + sizeof(mm_block_t) + 16) {
                int new_offset = offset + (int)sizeof(mm_block_t) + (int)size;
                mm_block_t *new_block = (mm_block_t *)(mm_heap + new_offset);
                new_block->magic = BLOCK_MAGIC;
                new_block->size = block->size - size - sizeof(mm_block_t);
                new_block->is_free = 1;
                new_block->next_offset = block->next_offset;
                block->size = size;
                block->next_offset = new_offset;
            }
            block->is_free = 0;
            return (void *)(mm_heap + offset + sizeof(mm_block_t));
        }
        offset = block->next_offset;
    }
    return (void *)0;
}

void mm_free(void *ptr) {
    if (!ptr) return;
    char *p = (char *)ptr;
    mm_block_t *block = (mm_block_t *)(p - sizeof(mm_block_t));
    if (block->magic != BLOCK_MAGIC) return;
    block->is_free = 1;
}

size_t mm_free_space(void) {
    size_t total = 0;
    int offset = 0;
    while (offset >= 0 && (size_t)offset < HEAP_SIZE) {
        mm_block_t *block = (mm_block_t *)(mm_heap + offset);
        if (block->magic != BLOCK_MAGIC) break;
        if (block->is_free) total += block->size;
        offset = block->next_offset;
    }
    return total;
}

int main(void) {
    mm_init();
    void *a = mm_alloc(64);
    void *b = mm_alloc(128);
    mm_free(a);
    size_t free_bytes = mm_free_space();
    return (a != (void *)0 && b != (void *)0) ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C726: Custom malloc/free with free list should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C726: Should produce non-empty output");
}

#[test]
fn c727_slab_allocator() {
    let c_code = r#"
typedef unsigned long size_t;

#define SLAB_OBJ_SIZE 64
#define SLAB_CAPACITY 32

typedef struct {
    char data[SLAB_OBJ_SIZE];
} slab_obj_t;

typedef struct {
    slab_obj_t objects[SLAB_CAPACITY];
    int free_bitmap[SLAB_CAPACITY];
    int num_free;
    int total;
} slab_t;

void slab_init(slab_t *s) {
    int i;
    s->num_free = SLAB_CAPACITY;
    s->total = SLAB_CAPACITY;
    for (i = 0; i < SLAB_CAPACITY; i++) {
        s->free_bitmap[i] = 1;
    }
}

int slab_alloc(slab_t *s) {
    int i;
    if (s->num_free == 0) return -1;
    for (i = 0; i < SLAB_CAPACITY; i++) {
        if (s->free_bitmap[i]) {
            s->free_bitmap[i] = 0;
            s->num_free--;
            return i;
        }
    }
    return -1;
}

void slab_free(slab_t *s, int idx) {
    if (idx < 0 || idx >= SLAB_CAPACITY) return;
    if (s->free_bitmap[idx] == 0) {
        s->free_bitmap[idx] = 1;
        s->num_free++;
    }
}

void *slab_get_ptr(slab_t *s, int idx) {
    if (idx < 0 || idx >= SLAB_CAPACITY) return (void *)0;
    return (void *)&s->objects[idx];
}

int slab_usage_percent(const slab_t *s) {
    int used = s->total - s->num_free;
    return (used * 100) / s->total;
}

int main(void) {
    slab_t slab;
    slab_init(&slab);
    int a = slab_alloc(&slab);
    int b = slab_alloc(&slab);
    slab_free(&slab, a);
    int pct = slab_usage_percent(&slab);
    return (b >= 0) ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C727: Slab allocator should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C727: Should produce non-empty output");
}

#[test]
fn c728_buddy_allocator() {
    let c_code = r#"
typedef unsigned long size_t;

#define BUDDY_LEVELS 8
#define BUDDY_MAX_ORDER 7
#define BUDDY_POOL_SIZE 256

typedef struct {
    char pool[BUDDY_POOL_SIZE];
    int free_list[BUDDY_LEVELS][BUDDY_POOL_SIZE];
    int free_count[BUDDY_LEVELS];
} buddy_alloc_t;

static int buddy_block_size(int order) {
    int s = 1;
    int i;
    for (i = 0; i < order; i++) s *= 2;
    return s;
}

void buddy_init(buddy_alloc_t *ba) {
    int i, j;
    for (i = 0; i < BUDDY_LEVELS; i++) {
        ba->free_count[i] = 0;
        for (j = 0; j < BUDDY_POOL_SIZE; j++) {
            ba->free_list[i][j] = -1;
        }
    }
    ba->free_list[BUDDY_MAX_ORDER][0] = 0;
    ba->free_count[BUDDY_MAX_ORDER] = 1;
}

int buddy_find_order(int size) {
    int order = 0;
    int block = 1;
    while (block < size && order < BUDDY_MAX_ORDER) {
        block *= 2;
        order++;
    }
    return order;
}

int buddy_alloc(buddy_alloc_t *ba, int size) {
    int order = buddy_find_order(size);
    int level = order;
    while (level <= BUDDY_MAX_ORDER && ba->free_count[level] == 0) {
        level++;
    }
    if (level > BUDDY_MAX_ORDER) return -1;
    int idx = ba->free_list[level][ba->free_count[level] - 1];
    ba->free_count[level]--;
    while (level > order) {
        level--;
        int bsize = buddy_block_size(level);
        int buddy_idx = idx + bsize;
        ba->free_list[level][ba->free_count[level]] = buddy_idx;
        ba->free_count[level]++;
    }
    return idx;
}

void buddy_free(buddy_alloc_t *ba, int offset, int order) {
    ba->free_list[order][ba->free_count[order]] = offset;
    ba->free_count[order]++;
}

int main(void) {
    buddy_alloc_t ba;
    buddy_init(&ba);
    int a = buddy_alloc(&ba, 4);
    int b = buddy_alloc(&ba, 8);
    buddy_free(&ba, a, buddy_find_order(4));
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C728: Buddy allocator should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C728: Should produce non-empty output");
}

#[test]
fn c729_memory_pool_fixed_size_blocks() {
    let c_code = r#"
typedef unsigned long size_t;

#define MPOOL_BLOCK_SIZE 32
#define MPOOL_NUM_BLOCKS 64

typedef struct {
    char storage[MPOOL_BLOCK_SIZE * MPOOL_NUM_BLOCKS];
    int free_stack[MPOOL_NUM_BLOCKS];
    int stack_top;
    int total_allocs;
    int total_frees;
} mpool_t;

void mpool_init(mpool_t *p) {
    int i;
    p->stack_top = MPOOL_NUM_BLOCKS - 1;
    p->total_allocs = 0;
    p->total_frees = 0;
    for (i = 0; i < MPOOL_NUM_BLOCKS; i++) {
        p->free_stack[i] = i;
    }
}

int mpool_alloc(mpool_t *p) {
    if (p->stack_top < 0) return -1;
    int block_idx = p->free_stack[p->stack_top];
    p->stack_top--;
    p->total_allocs++;
    return block_idx;
}

void mpool_free(mpool_t *p, int block_idx) {
    if (block_idx < 0 || block_idx >= MPOOL_NUM_BLOCKS) return;
    p->stack_top++;
    p->free_stack[p->stack_top] = block_idx;
    p->total_frees++;
}

void *mpool_get_ptr(mpool_t *p, int block_idx) {
    if (block_idx < 0 || block_idx >= MPOOL_NUM_BLOCKS) return (void *)0;
    return (void *)&p->storage[block_idx * MPOOL_BLOCK_SIZE];
}

int mpool_available(const mpool_t *p) {
    return p->stack_top + 1;
}

int main(void) {
    mpool_t pool;
    mpool_init(&pool);
    int a = mpool_alloc(&pool);
    int b = mpool_alloc(&pool);
    mpool_free(&pool, a);
    int avail = mpool_available(&pool);
    return (avail == MPOOL_NUM_BLOCKS - 1) ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C729: Memory pool with fixed-size blocks should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C729: Should produce non-empty output");
}

#[test]
fn c730_arena_allocator_bump() {
    let c_code = r#"
typedef unsigned long size_t;

#define ARENA_SIZE 2048

typedef struct {
    char buffer[ARENA_SIZE];
    size_t offset;
    size_t peak;
    int alloc_count;
} arena_t;

void arena_init(arena_t *a) {
    a->offset = 0;
    a->peak = 0;
    a->alloc_count = 0;
}

void *arena_alloc(arena_t *a, size_t size) {
    size_t aligned = (size + 7) & ~((size_t)7);
    if (a->offset + aligned > ARENA_SIZE) return (void *)0;
    void *ptr = (void *)&a->buffer[a->offset];
    a->offset += aligned;
    if (a->offset > a->peak) a->peak = a->offset;
    a->alloc_count++;
    return ptr;
}

void arena_reset(arena_t *a) {
    a->offset = 0;
    a->alloc_count = 0;
}

size_t arena_used(const arena_t *a) {
    return a->offset;
}

size_t arena_remaining(const arena_t *a) {
    return ARENA_SIZE - a->offset;
}

int main(void) {
    arena_t arena;
    arena_init(&arena);
    void *p1 = arena_alloc(&arena, 100);
    void *p2 = arena_alloc(&arena, 200);
    size_t used = arena_used(&arena);
    arena_reset(&arena);
    return (p1 != (void *)0 && p2 != (void *)0) ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C730: Arena allocator (bump allocator) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C730: Should produce non-empty output");
}

// ============================================================================
// C731-C735: GC and Lifecycle Patterns
// ============================================================================

#[test]
fn c731_reference_counting_manual() {
    let c_code = r#"
typedef unsigned long size_t;

#define RC_MAX_OBJECTS 64

typedef struct {
    int refcount;
    int active;
    int data;
    int id;
} rc_object_t;

typedef struct {
    rc_object_t objects[RC_MAX_OBJECTS];
    int next_id;
    int live_count;
} rc_heap_t;

void rc_heap_init(rc_heap_t *h) {
    int i;
    h->next_id = 0;
    h->live_count = 0;
    for (i = 0; i < RC_MAX_OBJECTS; i++) {
        h->objects[i].active = 0;
        h->objects[i].refcount = 0;
        h->objects[i].data = 0;
        h->objects[i].id = -1;
    }
}

int rc_create(rc_heap_t *h, int data) {
    int i;
    for (i = 0; i < RC_MAX_OBJECTS; i++) {
        if (!h->objects[i].active) {
            h->objects[i].active = 1;
            h->objects[i].refcount = 1;
            h->objects[i].data = data;
            h->objects[i].id = h->next_id++;
            h->live_count++;
            return i;
        }
    }
    return -1;
}

void rc_retain(rc_heap_t *h, int idx) {
    if (idx >= 0 && idx < RC_MAX_OBJECTS && h->objects[idx].active) {
        h->objects[idx].refcount++;
    }
}

void rc_release(rc_heap_t *h, int idx) {
    if (idx < 0 || idx >= RC_MAX_OBJECTS) return;
    if (!h->objects[idx].active) return;
    h->objects[idx].refcount--;
    if (h->objects[idx].refcount <= 0) {
        h->objects[idx].active = 0;
        h->objects[idx].refcount = 0;
        h->live_count--;
    }
}

int rc_get_refcount(const rc_heap_t *h, int idx) {
    if (idx < 0 || idx >= RC_MAX_OBJECTS) return -1;
    return h->objects[idx].refcount;
}

int main(void) {
    rc_heap_t heap;
    rc_heap_init(&heap);
    int obj = rc_create(&heap, 42);
    rc_retain(&heap, obj);
    rc_release(&heap, obj);
    int rc = rc_get_refcount(&heap, obj);
    rc_release(&heap, obj);
    return (heap.live_count == 0) ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C731: Reference counting (manual refcount) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C731: Should produce non-empty output");
}

#[test]
fn c732_mark_and_sweep_gc() {
    let c_code = r#"
typedef unsigned long size_t;

#define GC_HEAP_SIZE 128
#define GC_MAX_ROOTS 16

typedef struct {
    int marked;
    int active;
    int value;
    int ref1;
    int ref2;
} gc_obj_t;

typedef struct {
    gc_obj_t heap[GC_HEAP_SIZE];
    int roots[GC_MAX_ROOTS];
    int num_roots;
    int live_count;
} gc_state_t;

void gc_init(gc_state_t *gc) {
    int i;
    gc->num_roots = 0;
    gc->live_count = 0;
    for (i = 0; i < GC_HEAP_SIZE; i++) {
        gc->heap[i].marked = 0;
        gc->heap[i].active = 0;
        gc->heap[i].ref1 = -1;
        gc->heap[i].ref2 = -1;
    }
}

int gc_alloc(gc_state_t *gc, int value) {
    int i;
    for (i = 0; i < GC_HEAP_SIZE; i++) {
        if (!gc->heap[i].active) {
            gc->heap[i].active = 1;
            gc->heap[i].marked = 0;
            gc->heap[i].value = value;
            gc->heap[i].ref1 = -1;
            gc->heap[i].ref2 = -1;
            gc->live_count++;
            return i;
        }
    }
    return -1;
}

void gc_add_root(gc_state_t *gc, int idx) {
    if (gc->num_roots < GC_MAX_ROOTS) {
        gc->roots[gc->num_roots++] = idx;
    }
}

void gc_mark(gc_state_t *gc, int idx) {
    if (idx < 0 || idx >= GC_HEAP_SIZE) return;
    if (!gc->heap[idx].active || gc->heap[idx].marked) return;
    gc->heap[idx].marked = 1;
    gc_mark(gc, gc->heap[idx].ref1);
    gc_mark(gc, gc->heap[idx].ref2);
}

int gc_sweep(gc_state_t *gc) {
    int freed = 0;
    int i;
    for (i = 0; i < GC_HEAP_SIZE; i++) {
        if (gc->heap[i].active && !gc->heap[i].marked) {
            gc->heap[i].active = 0;
            gc->live_count--;
            freed++;
        }
        gc->heap[i].marked = 0;
    }
    return freed;
}

int gc_collect(gc_state_t *gc) {
    int i;
    for (i = 0; i < gc->num_roots; i++) {
        gc_mark(gc, gc->roots[i]);
    }
    return gc_sweep(gc);
}

int main(void) {
    gc_state_t gc;
    gc_init(&gc);
    int a = gc_alloc(&gc, 10);
    int b = gc_alloc(&gc, 20);
    int c = gc_alloc(&gc, 30);
    gc.heap[a].ref1 = b;
    gc_add_root(&gc, a);
    int freed = gc_collect(&gc);
    return (freed == 1) ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C732: Mark-and-sweep garbage collector should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C732: Should produce non-empty output");
}

#[test]
fn c733_copying_gc_semispace() {
    let c_code = r#"
typedef unsigned long size_t;

#define SEMI_SIZE 128

typedef struct {
    int value;
    int size;
    int forwarded;
    int forward_idx;
} semi_obj_t;

typedef struct {
    semi_obj_t from_space[SEMI_SIZE];
    semi_obj_t to_space[SEMI_SIZE];
    int from_top;
    int to_top;
    int num_collections;
} semi_gc_t;

void semi_init(semi_gc_t *gc) {
    gc->from_top = 0;
    gc->to_top = 0;
    gc->num_collections = 0;
    int i;
    for (i = 0; i < SEMI_SIZE; i++) {
        gc->from_space[i].forwarded = 0;
        gc->from_space[i].forward_idx = -1;
        gc->to_space[i].forwarded = 0;
        gc->to_space[i].forward_idx = -1;
    }
}

int semi_alloc(semi_gc_t *gc, int value) {
    if (gc->from_top >= SEMI_SIZE) return -1;
    int idx = gc->from_top;
    gc->from_space[idx].value = value;
    gc->from_space[idx].size = 1;
    gc->from_space[idx].forwarded = 0;
    gc->from_space[idx].forward_idx = -1;
    gc->from_top++;
    return idx;
}

int semi_copy_obj(semi_gc_t *gc, int from_idx) {
    if (from_idx < 0 || from_idx >= gc->from_top) return -1;
    if (gc->from_space[from_idx].forwarded) {
        return gc->from_space[from_idx].forward_idx;
    }
    int to_idx = gc->to_top;
    gc->to_space[to_idx].value = gc->from_space[from_idx].value;
    gc->to_space[to_idx].size = gc->from_space[from_idx].size;
    gc->to_space[to_idx].forwarded = 0;
    gc->to_space[to_idx].forward_idx = -1;
    gc->to_top++;
    gc->from_space[from_idx].forwarded = 1;
    gc->from_space[from_idx].forward_idx = to_idx;
    return to_idx;
}

void semi_flip(semi_gc_t *gc) {
    int i;
    for (i = 0; i < gc->to_top; i++) {
        gc->from_space[i] = gc->to_space[i];
    }
    gc->from_top = gc->to_top;
    gc->to_top = 0;
    gc->num_collections++;
    for (i = 0; i < SEMI_SIZE; i++) {
        gc->from_space[i].forwarded = 0;
        gc->from_space[i].forward_idx = -1;
    }
}

int main(void) {
    semi_gc_t gc;
    semi_init(&gc);
    int a = semi_alloc(&gc, 100);
    int b = semi_alloc(&gc, 200);
    int c = semi_alloc(&gc, 300);
    semi_copy_obj(&gc, a);
    semi_copy_obj(&gc, c);
    semi_flip(&gc);
    return (gc.from_top == 2) ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C733: Copying garbage collector (semi-space) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C733: Should produce non-empty output");
}

#[test]
fn c734_memory_mapped_file_simulation() {
    let c_code = r#"
typedef unsigned long size_t;

#define MMAP_PAGE_SIZE 256
#define MMAP_NUM_PAGES 16

typedef struct {
    char data[MMAP_PAGE_SIZE];
    int dirty;
    int valid;
    int page_id;
} mmap_page_t;

typedef struct {
    mmap_page_t pages[MMAP_NUM_PAGES];
    char backing_store[MMAP_PAGE_SIZE * MMAP_NUM_PAGES];
    int num_faults;
    int num_flushes;
} mmap_file_t;

void mmap_init(mmap_file_t *mf) {
    int i, j;
    mf->num_faults = 0;
    mf->num_flushes = 0;
    for (i = 0; i < MMAP_NUM_PAGES; i++) {
        mf->pages[i].dirty = 0;
        mf->pages[i].valid = 0;
        mf->pages[i].page_id = i;
        for (j = 0; j < MMAP_PAGE_SIZE; j++) {
            mf->pages[i].data[j] = 0;
            mf->backing_store[i * MMAP_PAGE_SIZE + j] = 0;
        }
    }
}

void mmap_load_page(mmap_file_t *mf, int page_idx) {
    if (page_idx < 0 || page_idx >= MMAP_NUM_PAGES) return;
    int base = page_idx * MMAP_PAGE_SIZE;
    int j;
    for (j = 0; j < MMAP_PAGE_SIZE; j++) {
        mf->pages[page_idx].data[j] = mf->backing_store[base + j];
    }
    mf->pages[page_idx].valid = 1;
    mf->pages[page_idx].dirty = 0;
    mf->num_faults++;
}

void mmap_write_byte(mmap_file_t *mf, int page_idx, int offset, char val) {
    if (page_idx < 0 || page_idx >= MMAP_NUM_PAGES) return;
    if (offset < 0 || offset >= MMAP_PAGE_SIZE) return;
    if (!mf->pages[page_idx].valid) mmap_load_page(mf, page_idx);
    mf->pages[page_idx].data[offset] = val;
    mf->pages[page_idx].dirty = 1;
}

char mmap_read_byte(mmap_file_t *mf, int page_idx, int offset) {
    if (page_idx < 0 || page_idx >= MMAP_NUM_PAGES) return 0;
    if (offset < 0 || offset >= MMAP_PAGE_SIZE) return 0;
    if (!mf->pages[page_idx].valid) mmap_load_page(mf, page_idx);
    return mf->pages[page_idx].data[offset];
}

void mmap_flush_page(mmap_file_t *mf, int page_idx) {
    if (page_idx < 0 || page_idx >= MMAP_NUM_PAGES) return;
    if (!mf->pages[page_idx].dirty) return;
    int base = page_idx * MMAP_PAGE_SIZE;
    int j;
    for (j = 0; j < MMAP_PAGE_SIZE; j++) {
        mf->backing_store[base + j] = mf->pages[page_idx].data[j];
    }
    mf->pages[page_idx].dirty = 0;
    mf->num_flushes++;
}

int main(void) {
    mmap_file_t mf;
    mmap_init(&mf);
    mmap_write_byte(&mf, 0, 10, 'A');
    char val = mmap_read_byte(&mf, 0, 10);
    mmap_flush_page(&mf, 0);
    return (val == 'A') ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C734: Memory-mapped file simulation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C734: Should produce non-empty output");
}

#[test]
fn c735_stack_allocator_lifo() {
    let c_code = r#"
typedef unsigned long size_t;

#define STALLOC_SIZE 1024

typedef struct {
    char buffer[STALLOC_SIZE];
    size_t top;
    size_t markers[32];
    int marker_count;
    int alloc_count;
} stack_alloc_t;

void stalloc_init(stack_alloc_t *sa) {
    sa->top = 0;
    sa->marker_count = 0;
    sa->alloc_count = 0;
}

void *stalloc_alloc(stack_alloc_t *sa, size_t size) {
    size_t aligned = (size + 7) & ~((size_t)7);
    if (sa->top + aligned > STALLOC_SIZE) return (void *)0;
    void *ptr = (void *)&sa->buffer[sa->top];
    sa->top += aligned;
    sa->alloc_count++;
    return ptr;
}

size_t stalloc_push_marker(stack_alloc_t *sa) {
    if (sa->marker_count >= 32) return sa->top;
    sa->markers[sa->marker_count] = sa->top;
    sa->marker_count++;
    return sa->top;
}

void stalloc_pop_to_marker(stack_alloc_t *sa) {
    if (sa->marker_count <= 0) return;
    sa->marker_count--;
    sa->top = sa->markers[sa->marker_count];
}

size_t stalloc_used(const stack_alloc_t *sa) {
    return sa->top;
}

size_t stalloc_remaining(const stack_alloc_t *sa) {
    return STALLOC_SIZE - sa->top;
}

int main(void) {
    stack_alloc_t sa;
    stalloc_init(&sa);
    stalloc_push_marker(&sa);
    void *p1 = stalloc_alloc(&sa, 64);
    void *p2 = stalloc_alloc(&sa, 128);
    stalloc_pop_to_marker(&sa);
    size_t used = stalloc_used(&sa);
    return (used == 0) ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C735: Stack allocator (LIFO) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C735: Should produce non-empty output");
}

// ============================================================================
// C736-C740: Cache and Safety Patterns
// ============================================================================

#[test]
fn c736_object_cache_with_free_list() {
    let c_code = r#"
typedef unsigned long size_t;

#define OCACHE_MAX 32

typedef struct {
    int data[4];
    int in_use;
} cached_obj_t;

typedef struct {
    cached_obj_t objects[OCACHE_MAX];
    int free_indices[OCACHE_MAX];
    int free_top;
    int cache_hits;
    int cache_misses;
} obj_cache_t;

void ocache_init(obj_cache_t *c) {
    int i;
    c->free_top = OCACHE_MAX - 1;
    c->cache_hits = 0;
    c->cache_misses = 0;
    for (i = 0; i < OCACHE_MAX; i++) {
        c->objects[i].in_use = 0;
        c->free_indices[i] = i;
    }
}

int ocache_acquire(obj_cache_t *c) {
    if (c->free_top < 0) {
        c->cache_misses++;
        return -1;
    }
    int idx = c->free_indices[c->free_top];
    c->free_top--;
    c->objects[idx].in_use = 1;
    c->cache_hits++;
    return idx;
}

void ocache_release(obj_cache_t *c, int idx) {
    if (idx < 0 || idx >= OCACHE_MAX) return;
    if (!c->objects[idx].in_use) return;
    c->objects[idx].in_use = 0;
    c->free_top++;
    c->free_indices[c->free_top] = idx;
}

int ocache_hit_rate_pct(const obj_cache_t *c) {
    int total = c->cache_hits + c->cache_misses;
    if (total == 0) return 0;
    return (c->cache_hits * 100) / total;
}

int main(void) {
    obj_cache_t cache;
    ocache_init(&cache);
    int a = ocache_acquire(&cache);
    int b = ocache_acquire(&cache);
    ocache_release(&cache, a);
    int c_obj = ocache_acquire(&cache);
    int rate = ocache_hit_rate_pct(&cache);
    return (rate == 100) ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C736: Object cache with free list should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C736: Should produce non-empty output");
}

#[test]
fn c737_memory_defragmentation_compaction() {
    let c_code = r#"
typedef unsigned long size_t;

#define DEFRAG_SLOTS 32

typedef struct {
    int value;
    int active;
    int original_slot;
} defrag_entry_t;

typedef struct {
    defrag_entry_t slots[DEFRAG_SLOTS];
    int active_count;
    int compaction_count;
} defrag_heap_t;

void defrag_init(defrag_heap_t *h) {
    int i;
    h->active_count = 0;
    h->compaction_count = 0;
    for (i = 0; i < DEFRAG_SLOTS; i++) {
        h->slots[i].active = 0;
        h->slots[i].value = 0;
        h->slots[i].original_slot = i;
    }
}

int defrag_alloc(defrag_heap_t *h, int value) {
    int i;
    for (i = 0; i < DEFRAG_SLOTS; i++) {
        if (!h->slots[i].active) {
            h->slots[i].active = 1;
            h->slots[i].value = value;
            h->slots[i].original_slot = i;
            h->active_count++;
            return i;
        }
    }
    return -1;
}

void defrag_free(defrag_heap_t *h, int idx) {
    if (idx < 0 || idx >= DEFRAG_SLOTS) return;
    if (h->slots[idx].active) {
        h->slots[idx].active = 0;
        h->active_count--;
    }
}

int defrag_compact(defrag_heap_t *h) {
    int write_idx = 0;
    int read_idx;
    int moved = 0;
    for (read_idx = 0; read_idx < DEFRAG_SLOTS; read_idx++) {
        if (h->slots[read_idx].active) {
            if (write_idx != read_idx) {
                h->slots[write_idx] = h->slots[read_idx];
                h->slots[read_idx].active = 0;
                moved++;
            }
            write_idx++;
        }
    }
    h->compaction_count++;
    return moved;
}

int defrag_fragmentation_pct(const defrag_heap_t *h) {
    if (h->active_count == 0) return 0;
    int last_active = -1;
    int i;
    for (i = DEFRAG_SLOTS - 1; i >= 0; i--) {
        if (h->slots[i].active) { last_active = i; break; }
    }
    if (last_active < 0) return 0;
    int span = last_active + 1;
    int holes = span - h->active_count;
    return (holes * 100) / span;
}

int main(void) {
    defrag_heap_t heap;
    defrag_init(&heap);
    defrag_alloc(&heap, 10);
    int b = defrag_alloc(&heap, 20);
    defrag_alloc(&heap, 30);
    defrag_free(&heap, b);
    int moved = defrag_compact(&heap);
    return (moved > 0) ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C737: Memory defragmentation/compaction should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C737: Should produce non-empty output");
}

#[test]
fn c738_tlsf_two_level_segregated_fit() {
    let c_code = r#"
typedef unsigned long size_t;

#define TLSF_FL_COUNT 8
#define TLSF_SL_COUNT 4
#define TLSF_POOL_SIZE 512

typedef struct {
    int offset;
    int size;
    int free;
    int next;
    int prev;
} tlsf_block_t;

typedef struct {
    tlsf_block_t blocks[64];
    int block_count;
    int fl_bitmap;
    int sl_bitmap[TLSF_FL_COUNT];
    int free_heads[TLSF_FL_COUNT][TLSF_SL_COUNT];
    char pool[TLSF_POOL_SIZE];
} tlsf_t;

static int tlsf_fls(int x) {
    int r = 0;
    while (x > 1) { x >>= 1; r++; }
    return r;
}

static void tlsf_mapping(int size, int *fl, int *sl) {
    *fl = tlsf_fls(size);
    if (*fl >= TLSF_FL_COUNT) *fl = TLSF_FL_COUNT - 1;
    int shift = (*fl > 2) ? (*fl - 2) : 0;
    *sl = (size >> shift) & (TLSF_SL_COUNT - 1);
}

void tlsf_init(tlsf_t *t) {
    int i, j;
    t->block_count = 0;
    t->fl_bitmap = 0;
    for (i = 0; i < TLSF_FL_COUNT; i++) {
        t->sl_bitmap[i] = 0;
        for (j = 0; j < TLSF_SL_COUNT; j++) {
            t->free_heads[i][j] = -1;
        }
    }
    t->blocks[0].offset = 0;
    t->blocks[0].size = TLSF_POOL_SIZE;
    t->blocks[0].free = 1;
    t->blocks[0].next = -1;
    t->blocks[0].prev = -1;
    t->block_count = 1;
    int fl, sl;
    tlsf_mapping(TLSF_POOL_SIZE, &fl, &sl);
    t->free_heads[fl][sl] = 0;
    t->fl_bitmap |= (1 << fl);
    t->sl_bitmap[fl] |= (1 << sl);
}

int tlsf_alloc(tlsf_t *t, int size) {
    int fl, sl;
    tlsf_mapping(size, &fl, &sl);
    int idx = t->free_heads[fl][sl];
    if (idx < 0) return -1;
    t->free_heads[fl][sl] = t->blocks[idx].next;
    if (t->free_heads[fl][sl] < 0) {
        t->sl_bitmap[fl] &= ~(1 << sl);
        if (t->sl_bitmap[fl] == 0) t->fl_bitmap &= ~(1 << fl);
    }
    t->blocks[idx].free = 0;
    return t->blocks[idx].offset;
}

int main(void) {
    tlsf_t t;
    tlsf_init(&t);
    int a = tlsf_alloc(&t, 32);
    return (a >= 0) ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C738: TLSF allocator should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C738: Should produce non-empty output");
}

#[test]
fn c739_memory_leak_detector() {
    let c_code = r#"
typedef unsigned long size_t;

#define LEAK_MAX_ALLOCS 64

typedef struct {
    int id;
    int size;
    int line;
    int freed;
} leak_record_t;

typedef struct {
    leak_record_t records[LEAK_MAX_ALLOCS];
    int count;
    int next_id;
    int total_allocated;
    int total_freed;
} leak_detector_t;

void leak_init(leak_detector_t *ld) {
    ld->count = 0;
    ld->next_id = 1;
    ld->total_allocated = 0;
    ld->total_freed = 0;
}

int leak_record_alloc(leak_detector_t *ld, int size, int line) {
    if (ld->count >= LEAK_MAX_ALLOCS) return -1;
    int idx = ld->count;
    ld->records[idx].id = ld->next_id++;
    ld->records[idx].size = size;
    ld->records[idx].line = line;
    ld->records[idx].freed = 0;
    ld->count++;
    ld->total_allocated += size;
    return ld->records[idx].id;
}

int leak_record_free(leak_detector_t *ld, int id) {
    int i;
    for (i = 0; i < ld->count; i++) {
        if (ld->records[i].id == id && !ld->records[i].freed) {
            ld->records[i].freed = 1;
            ld->total_freed += ld->records[i].size;
            return 0;
        }
    }
    return -1;
}

int leak_count_leaks(const leak_detector_t *ld) {
    int leaks = 0;
    int i;
    for (i = 0; i < ld->count; i++) {
        if (!ld->records[i].freed) leaks++;
    }
    return leaks;
}

int leak_total_leaked_bytes(const leak_detector_t *ld) {
    int total = 0;
    int i;
    for (i = 0; i < ld->count; i++) {
        if (!ld->records[i].freed) total += ld->records[i].size;
    }
    return total;
}

int main(void) {
    leak_detector_t ld;
    leak_init(&ld);
    int id1 = leak_record_alloc(&ld, 64, 10);
    int id2 = leak_record_alloc(&ld, 128, 20);
    int id3 = leak_record_alloc(&ld, 256, 30);
    leak_record_free(&ld, id1);
    leak_record_free(&ld, id3);
    int leaks = leak_count_leaks(&ld);
    int leaked_bytes = leak_total_leaked_bytes(&ld);
    return (leaks == 1 && leaked_bytes == 128) ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C739: Memory leak detector should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C739: Should produce non-empty output");
}

#[test]
fn c740_buffer_overflow_guard_canary() {
    let c_code = r#"
typedef unsigned long size_t;

#define CANARY_VALUE 0xDEADCAFE
#define GUARD_BUF_SIZE 64

typedef struct {
    unsigned int front_canary;
    char data[GUARD_BUF_SIZE];
    unsigned int back_canary;
    int write_pos;
} guarded_buf_t;

void guard_init(guarded_buf_t *gb) {
    gb->front_canary = CANARY_VALUE;
    gb->back_canary = CANARY_VALUE;
    gb->write_pos = 0;
    int i;
    for (i = 0; i < GUARD_BUF_SIZE; i++) {
        gb->data[i] = 0;
    }
}

int guard_check(const guarded_buf_t *gb) {
    if (gb->front_canary != CANARY_VALUE) return -1;
    if (gb->back_canary != CANARY_VALUE) return -2;
    return 0;
}

int guard_write(guarded_buf_t *gb, char val) {
    if (guard_check(gb) != 0) return -1;
    if (gb->write_pos >= GUARD_BUF_SIZE) return -2;
    gb->data[gb->write_pos] = val;
    gb->write_pos++;
    return 0;
}

int guard_write_n(guarded_buf_t *gb, const char *src, int n) {
    int i;
    for (i = 0; i < n; i++) {
        int result = guard_write(gb, src[i]);
        if (result != 0) return result;
    }
    return 0;
}

char guard_read(const guarded_buf_t *gb, int pos) {
    if (pos < 0 || pos >= GUARD_BUF_SIZE) return 0;
    if (guard_check(gb) != 0) return 0;
    return gb->data[pos];
}

int main(void) {
    guarded_buf_t buf;
    guard_init(&buf);
    guard_write(&buf, 'H');
    guard_write(&buf, 'i');
    int ok = guard_check(&buf);
    char ch = guard_read(&buf, 0);
    return (ok == 0 && ch == 'H') ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C740: Buffer overflow guard (canary values) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C740: Should produce non-empty output");
}

// ============================================================================
// C741-C745: Alignment and Sharing
// ============================================================================

#[test]
fn c741_memory_alignment_utilities() {
    let c_code = r#"
typedef unsigned long size_t;

size_t align_up(size_t value, size_t alignment) {
    size_t mask = alignment - 1;
    return (value + mask) & ~mask;
}

size_t align_down(size_t value, size_t alignment) {
    return value & ~(alignment - 1);
}

int is_aligned(size_t value, size_t alignment) {
    return (value & (alignment - 1)) == 0;
}

int is_power_of_two(size_t x) {
    return x > 0 && (x & (x - 1)) == 0;
}

size_t next_power_of_two(size_t x) {
    if (x == 0) return 1;
    x--;
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    return x + 1;
}

typedef struct {
    char data[256];
    size_t size;
    size_t alignment;
} aligned_buf_t;

void abuf_init(aligned_buf_t *ab, size_t alignment) {
    ab->size = 0;
    ab->alignment = alignment;
}

size_t abuf_alloc(aligned_buf_t *ab, size_t size) {
    size_t offset = align_up(ab->size, ab->alignment);
    if (offset + size > 256) return (size_t)-1;
    ab->size = offset + size;
    return offset;
}

int main(void) {
    size_t a = align_up(13, 8);
    size_t b = align_down(13, 8);
    int c = is_aligned(16, 8);
    int d = is_power_of_two(32);
    size_t e = next_power_of_two(13);
    aligned_buf_t buf;
    abuf_init(&buf, 16);
    size_t off = abuf_alloc(&buf, 30);
    return (a == 16 && b == 8 && c && d && e == 16) ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C741: Memory alignment utilities should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C741: Should produce non-empty output");
}

#[test]
fn c742_shared_memory_region_manager() {
    let c_code = r#"
typedef unsigned long size_t;

#define SHMEM_REGION_SIZE 512
#define SHMEM_MAX_CLIENTS 8

typedef struct {
    char region[SHMEM_REGION_SIZE];
    int client_offsets[SHMEM_MAX_CLIENTS];
    int client_sizes[SHMEM_MAX_CLIENTS];
    int client_active[SHMEM_MAX_CLIENTS];
    int num_clients;
    size_t next_offset;
} shmem_mgr_t;

void shmem_init(shmem_mgr_t *sm) {
    int i;
    sm->num_clients = 0;
    sm->next_offset = 0;
    for (i = 0; i < SHMEM_MAX_CLIENTS; i++) {
        sm->client_active[i] = 0;
        sm->client_offsets[i] = 0;
        sm->client_sizes[i] = 0;
    }
}

int shmem_attach(shmem_mgr_t *sm, int requested_size) {
    if (sm->num_clients >= SHMEM_MAX_CLIENTS) return -1;
    if (sm->next_offset + (size_t)requested_size > SHMEM_REGION_SIZE) return -1;
    int client_id = sm->num_clients;
    sm->client_offsets[client_id] = (int)sm->next_offset;
    sm->client_sizes[client_id] = requested_size;
    sm->client_active[client_id] = 1;
    sm->next_offset += (size_t)requested_size;
    sm->num_clients++;
    return client_id;
}

void shmem_detach(shmem_mgr_t *sm, int client_id) {
    if (client_id < 0 || client_id >= SHMEM_MAX_CLIENTS) return;
    sm->client_active[client_id] = 0;
}

int shmem_write(shmem_mgr_t *sm, int client_id, int offset, char val) {
    if (client_id < 0 || client_id >= SHMEM_MAX_CLIENTS) return -1;
    if (!sm->client_active[client_id]) return -1;
    if (offset < 0 || offset >= sm->client_sizes[client_id]) return -1;
    sm->region[sm->client_offsets[client_id] + offset] = val;
    return 0;
}

char shmem_read(const shmem_mgr_t *sm, int client_id, int offset) {
    if (client_id < 0 || client_id >= SHMEM_MAX_CLIENTS) return 0;
    if (!sm->client_active[client_id]) return 0;
    if (offset < 0 || offset >= sm->client_sizes[client_id]) return 0;
    return sm->region[sm->client_offsets[client_id] + offset];
}

int main(void) {
    shmem_mgr_t sm;
    shmem_init(&sm);
    int c1 = shmem_attach(&sm, 64);
    int c2 = shmem_attach(&sm, 128);
    shmem_write(&sm, c1, 0, 'X');
    char val = shmem_read(&sm, c1, 0);
    shmem_detach(&sm, c2);
    return (val == 'X') ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C742: Shared memory region manager should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C742: Should produce non-empty output");
}

#[test]
fn c743_copy_on_write_page_simulation() {
    let c_code = r#"
typedef unsigned long size_t;

#define COW_PAGE_SIZE 64
#define COW_MAX_PAGES 16

typedef struct {
    char data[COW_PAGE_SIZE];
    int ref_count;
    int page_id;
} cow_page_t;

typedef struct {
    cow_page_t pages[COW_MAX_PAGES];
    int page_refs[COW_MAX_PAGES];
    int num_pages;
    int num_copies;
} cow_mgr_t;

void cow_init(cow_mgr_t *cm) {
    cm->num_pages = 0;
    cm->num_copies = 0;
    int i;
    for (i = 0; i < COW_MAX_PAGES; i++) {
        cm->pages[i].ref_count = 0;
        cm->pages[i].page_id = i;
        cm->page_refs[i] = 0;
    }
}

int cow_create_page(cow_mgr_t *cm) {
    if (cm->num_pages >= COW_MAX_PAGES) return -1;
    int idx = cm->num_pages;
    cm->pages[idx].ref_count = 1;
    cm->pages[idx].page_id = idx;
    cm->page_refs[idx] = 1;
    cm->num_pages++;
    return idx;
}

int cow_share_page(cow_mgr_t *cm, int page_idx) {
    if (page_idx < 0 || page_idx >= cm->num_pages) return -1;
    cm->pages[page_idx].ref_count++;
    cm->page_refs[page_idx]++;
    return page_idx;
}

int cow_write_page(cow_mgr_t *cm, int page_idx, int offset, char val) {
    if (page_idx < 0 || page_idx >= cm->num_pages) return -1;
    if (offset < 0 || offset >= COW_PAGE_SIZE) return -1;
    if (cm->pages[page_idx].ref_count > 1) {
        int new_idx = cow_create_page(cm);
        if (new_idx < 0) return -1;
        int j;
        for (j = 0; j < COW_PAGE_SIZE; j++) {
            cm->pages[new_idx].data[j] = cm->pages[page_idx].data[j];
        }
        cm->pages[page_idx].ref_count--;
        cm->num_copies++;
        page_idx = new_idx;
    }
    cm->pages[page_idx].data[offset] = val;
    return page_idx;
}

char cow_read_page(const cow_mgr_t *cm, int page_idx, int offset) {
    if (page_idx < 0 || page_idx >= cm->num_pages) return 0;
    if (offset < 0 || offset >= COW_PAGE_SIZE) return 0;
    return cm->pages[page_idx].data[offset];
}

int main(void) {
    cow_mgr_t cm;
    cow_init(&cm);
    int p1 = cow_create_page(&cm);
    cow_write_page(&cm, p1, 0, 'A');
    cow_share_page(&cm, p1);
    int p2 = cow_write_page(&cm, p1, 0, 'B');
    char orig = cow_read_page(&cm, p1, 0);
    return (cm.num_copies == 1) ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C743: Copy-on-write page simulation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C743: Should produce non-empty output");
}

#[test]
fn c744_memory_barrier_fence_simulation() {
    let c_code = r#"
typedef unsigned long size_t;

#define FENCE_QUEUE_SIZE 32

typedef struct {
    int value;
    int sequence;
    int committed;
} fence_entry_t;

typedef struct {
    fence_entry_t queue[FENCE_QUEUE_SIZE];
    int write_seq;
    int commit_seq;
    int read_seq;
    int fence_count;
} fence_queue_t;

void fence_init(fence_queue_t *fq) {
    fq->write_seq = 0;
    fq->commit_seq = 0;
    fq->read_seq = 0;
    fq->fence_count = 0;
    int i;
    for (i = 0; i < FENCE_QUEUE_SIZE; i++) {
        fq->queue[i].value = 0;
        fq->queue[i].sequence = -1;
        fq->queue[i].committed = 0;
    }
}

int fence_write(fence_queue_t *fq, int value) {
    int idx = fq->write_seq % FENCE_QUEUE_SIZE;
    fq->queue[idx].value = value;
    fq->queue[idx].sequence = fq->write_seq;
    fq->queue[idx].committed = 0;
    fq->write_seq++;
    return idx;
}

void fence_barrier(fence_queue_t *fq) {
    int i;
    for (i = fq->commit_seq; i < fq->write_seq; i++) {
        int idx = i % FENCE_QUEUE_SIZE;
        fq->queue[idx].committed = 1;
    }
    fq->commit_seq = fq->write_seq;
    fq->fence_count++;
}

int fence_read(fence_queue_t *fq, int *value) {
    if (fq->read_seq >= fq->commit_seq) return -1;
    int idx = fq->read_seq % FENCE_QUEUE_SIZE;
    if (!fq->queue[idx].committed) return -1;
    *value = fq->queue[idx].value;
    fq->read_seq++;
    return 0;
}

int fence_pending_count(const fence_queue_t *fq) {
    return fq->write_seq - fq->commit_seq;
}

int main(void) {
    fence_queue_t fq;
    fence_init(&fq);
    fence_write(&fq, 10);
    fence_write(&fq, 20);
    fence_barrier(&fq);
    fence_write(&fq, 30);
    int val = 0;
    fence_read(&fq, &val);
    int pending = fence_pending_count(&fq);
    return (val == 10 && pending == 1) ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C744: Memory barrier/fence simulation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C744: Should produce non-empty output");
}

#[test]
fn c745_generational_memory_manager() {
    let c_code = r#"
typedef unsigned long size_t;

#define GEN_YOUNG_SIZE 64
#define GEN_OLD_SIZE 64
#define GEN_MAX_OBJECTS 32

typedef struct {
    int value;
    int generation;
    int active;
    int age;
} gen_obj_t;

typedef struct {
    gen_obj_t young[GEN_MAX_OBJECTS];
    gen_obj_t old[GEN_MAX_OBJECTS];
    int young_count;
    int old_count;
    int promote_threshold;
    int minor_gc_count;
    int major_gc_count;
} gen_gc_t;

void gen_init(gen_gc_t *gc) {
    gc->young_count = 0;
    gc->old_count = 0;
    gc->promote_threshold = 3;
    gc->minor_gc_count = 0;
    gc->major_gc_count = 0;
    int i;
    for (i = 0; i < GEN_MAX_OBJECTS; i++) {
        gc->young[i].active = 0;
        gc->old[i].active = 0;
    }
}

int gen_alloc_young(gen_gc_t *gc, int value) {
    int i;
    for (i = 0; i < GEN_MAX_OBJECTS; i++) {
        if (!gc->young[i].active) {
            gc->young[i].value = value;
            gc->young[i].generation = 0;
            gc->young[i].active = 1;
            gc->young[i].age = 0;
            gc->young_count++;
            return i;
        }
    }
    return -1;
}

int gen_promote(gen_gc_t *gc, int young_idx) {
    if (young_idx < 0 || young_idx >= GEN_MAX_OBJECTS) return -1;
    if (!gc->young[young_idx].active) return -1;
    int i;
    for (i = 0; i < GEN_MAX_OBJECTS; i++) {
        if (!gc->old[i].active) {
            gc->old[i] = gc->young[young_idx];
            gc->old[i].generation = 1;
            gc->old[i].active = 1;
            gc->young[young_idx].active = 0;
            gc->young_count--;
            gc->old_count++;
            return i;
        }
    }
    return -1;
}

int gen_minor_gc(gen_gc_t *gc) {
    int promoted = 0;
    int i;
    for (i = 0; i < GEN_MAX_OBJECTS; i++) {
        if (gc->young[i].active) {
            gc->young[i].age++;
            if (gc->young[i].age >= gc->promote_threshold) {
                gen_promote(gc, i);
                promoted++;
            }
        }
    }
    gc->minor_gc_count++;
    return promoted;
}

int gen_major_gc(gen_gc_t *gc) {
    int freed = 0;
    int i;
    for (i = 0; i < GEN_MAX_OBJECTS; i++) {
        if (gc->old[i].active && gc->old[i].value == 0) {
            gc->old[i].active = 0;
            gc->old_count--;
            freed++;
        }
    }
    gc->major_gc_count++;
    return freed;
}

int main(void) {
    gen_gc_t gc;
    gen_init(&gc);
    gen_alloc_young(&gc, 10);
    gen_alloc_young(&gc, 20);
    gen_alloc_young(&gc, 0);
    gen_minor_gc(&gc);
    gen_minor_gc(&gc);
    gen_minor_gc(&gc);
    int freed = gen_major_gc(&gc);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C745: Generational memory manager should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C745: Should produce non-empty output");
}

// ============================================================================
// C746-C750: Advanced Patterns
// ============================================================================

#[test]
fn c746_object_pooling_with_type_tags() {
    let c_code = r#"
typedef unsigned long size_t;

#define TPOOL_MAX 32
#define TPOOL_TAG_INT 1
#define TPOOL_TAG_FLOAT 2
#define TPOOL_TAG_STRING 3

typedef struct {
    int tag;
    int active;
    union {
        int i_val;
        float f_val;
        char s_val[16];
    } data;
} tagged_obj_t;

typedef struct {
    tagged_obj_t objects[TPOOL_MAX];
    int free_list[TPOOL_MAX];
    int free_top;
    int type_counts[4];
} typed_pool_t;

void tpool_init(typed_pool_t *tp) {
    int i;
    tp->free_top = TPOOL_MAX - 1;
    for (i = 0; i < 4; i++) tp->type_counts[i] = 0;
    for (i = 0; i < TPOOL_MAX; i++) {
        tp->objects[i].active = 0;
        tp->objects[i].tag = 0;
        tp->free_list[i] = i;
    }
}

int tpool_alloc_int(typed_pool_t *tp, int value) {
    if (tp->free_top < 0) return -1;
    int idx = tp->free_list[tp->free_top];
    tp->free_top--;
    tp->objects[idx].active = 1;
    tp->objects[idx].tag = TPOOL_TAG_INT;
    tp->objects[idx].data.i_val = value;
    tp->type_counts[TPOOL_TAG_INT]++;
    return idx;
}

int tpool_alloc_float(typed_pool_t *tp, float value) {
    if (tp->free_top < 0) return -1;
    int idx = tp->free_list[tp->free_top];
    tp->free_top--;
    tp->objects[idx].active = 1;
    tp->objects[idx].tag = TPOOL_TAG_FLOAT;
    tp->objects[idx].data.f_val = value;
    tp->type_counts[TPOOL_TAG_FLOAT]++;
    return idx;
}

void tpool_free(typed_pool_t *tp, int idx) {
    if (idx < 0 || idx >= TPOOL_MAX) return;
    if (!tp->objects[idx].active) return;
    tp->type_counts[tp->objects[idx].tag]--;
    tp->objects[idx].active = 0;
    tp->free_top++;
    tp->free_list[tp->free_top] = idx;
}

int tpool_get_tag(const typed_pool_t *tp, int idx) {
    if (idx < 0 || idx >= TPOOL_MAX) return 0;
    return tp->objects[idx].tag;
}

int main(void) {
    typed_pool_t pool;
    tpool_init(&pool);
    int a = tpool_alloc_int(&pool, 42);
    int b = tpool_alloc_float(&pool, 3.14f);
    int tag_a = tpool_get_tag(&pool, a);
    tpool_free(&pool, b);
    return (tag_a == TPOOL_TAG_INT) ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C746: Object pooling with type tags should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C746: Should produce non-empty output");
}

#[test]
fn c747_memory_efficient_bitfield_packing() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint32_t words[8];
    int num_bits;
} bitpack_t;

void bp_init(bitpack_t *bp) {
    int i;
    bp->num_bits = 0;
    for (i = 0; i < 8; i++) bp->words[i] = 0;
}

void bp_set_bit(bitpack_t *bp, int pos) {
    if (pos < 0 || pos >= 256) return;
    int word = pos / 32;
    int bit = pos % 32;
    bp->words[word] |= ((uint32_t)1 << bit);
    if (pos >= bp->num_bits) bp->num_bits = pos + 1;
}

void bp_clear_bit(bitpack_t *bp, int pos) {
    if (pos < 0 || pos >= 256) return;
    int word = pos / 32;
    int bit = pos % 32;
    bp->words[word] &= ~((uint32_t)1 << bit);
}

int bp_get_bit(const bitpack_t *bp, int pos) {
    if (pos < 0 || pos >= 256) return 0;
    int word = pos / 32;
    int bit = pos % 32;
    return (bp->words[word] >> bit) & 1;
}

int bp_popcount_word(uint32_t x) {
    x = x - ((x >> 1) & 0x55555555);
    x = (x & 0x33333333) + ((x >> 2) & 0x33333333);
    x = (x + (x >> 4)) & 0x0F0F0F0F;
    return (int)((x * 0x01010101) >> 24);
}

int bp_popcount(const bitpack_t *bp) {
    int total = 0;
    int i;
    for (i = 0; i < 8; i++) {
        total += bp_popcount_word(bp->words[i]);
    }
    return total;
}

void bp_and(bitpack_t *dst, const bitpack_t *a, const bitpack_t *b) {
    int i;
    for (i = 0; i < 8; i++) {
        dst->words[i] = a->words[i] & b->words[i];
    }
}

void bp_or(bitpack_t *dst, const bitpack_t *a, const bitpack_t *b) {
    int i;
    for (i = 0; i < 8; i++) {
        dst->words[i] = a->words[i] | b->words[i];
    }
}

int main(void) {
    bitpack_t bp;
    bp_init(&bp);
    bp_set_bit(&bp, 0);
    bp_set_bit(&bp, 5);
    bp_set_bit(&bp, 31);
    bp_set_bit(&bp, 32);
    int count = bp_popcount(&bp);
    bp_clear_bit(&bp, 5);
    int b5 = bp_get_bit(&bp, 5);
    return (count == 4 && b5 == 0) ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C747: Memory-efficient bitfield packing should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C747: Should produce non-empty output");
}

#[test]
fn c748_custom_realloc_with_copy() {
    let c_code = r#"
typedef unsigned long size_t;

#define REALLOC_POOL_SIZE 1024
#define REALLOC_MAX_ALLOCS 32

typedef struct {
    int offset;
    int size;
    int active;
} ralloc_entry_t;

typedef struct {
    char pool[REALLOC_POOL_SIZE];
    ralloc_entry_t entries[REALLOC_MAX_ALLOCS];
    int num_entries;
    int next_offset;
} ralloc_t;

void ralloc_init(ralloc_t *ra) {
    ra->num_entries = 0;
    ra->next_offset = 0;
}

int ralloc_alloc(ralloc_t *ra, int size) {
    if (ra->num_entries >= REALLOC_MAX_ALLOCS) return -1;
    if (ra->next_offset + size > REALLOC_POOL_SIZE) return -1;
    int idx = ra->num_entries;
    ra->entries[idx].offset = ra->next_offset;
    ra->entries[idx].size = size;
    ra->entries[idx].active = 1;
    ra->next_offset += size;
    ra->num_entries++;
    return idx;
}

void ralloc_free(ralloc_t *ra, int idx) {
    if (idx < 0 || idx >= ra->num_entries) return;
    ra->entries[idx].active = 0;
}

int ralloc_realloc(ralloc_t *ra, int idx, int new_size) {
    if (idx < 0 || idx >= ra->num_entries) return -1;
    if (!ra->entries[idx].active) return -1;
    if (new_size <= ra->entries[idx].size) {
        ra->entries[idx].size = new_size;
        return idx;
    }
    if (ra->next_offset + new_size > REALLOC_POOL_SIZE) return -1;
    int new_idx = ra->num_entries;
    if (new_idx >= REALLOC_MAX_ALLOCS) return -1;
    int old_offset = ra->entries[idx].offset;
    int old_size = ra->entries[idx].size;
    int new_offset = ra->next_offset;
    int i;
    for (i = 0; i < old_size; i++) {
        ra->pool[new_offset + i] = ra->pool[old_offset + i];
    }
    ra->entries[idx].active = 0;
    ra->entries[new_idx].offset = new_offset;
    ra->entries[new_idx].size = new_size;
    ra->entries[new_idx].active = 1;
    ra->next_offset += new_size;
    ra->num_entries++;
    return new_idx;
}

void ralloc_write(ralloc_t *ra, int idx, int offset, char val) {
    if (idx < 0 || idx >= ra->num_entries) return;
    if (!ra->entries[idx].active) return;
    if (offset < 0 || offset >= ra->entries[idx].size) return;
    ra->pool[ra->entries[idx].offset + offset] = val;
}

char ralloc_read(const ralloc_t *ra, int idx, int offset) {
    if (idx < 0 || idx >= ra->num_entries) return 0;
    if (!ra->entries[idx].active) return 0;
    if (offset < 0 || offset >= ra->entries[idx].size) return 0;
    return ra->pool[ra->entries[idx].offset + offset];
}

int main(void) {
    ralloc_t ra;
    ralloc_init(&ra);
    int a = ralloc_alloc(&ra, 16);
    ralloc_write(&ra, a, 0, 'Z');
    int b = ralloc_realloc(&ra, a, 64);
    char val = ralloc_read(&ra, b, 0);
    return (val == 'Z') ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C748: Custom realloc with copy should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C748: Should produce non-empty output");
}

#[test]
fn c749_weak_reference_simulation() {
    let c_code = r#"
typedef unsigned long size_t;

#define WREF_MAX_OBJECTS 32
#define WREF_MAX_WEAK 64

typedef struct {
    int value;
    int strong_count;
    int active;
    int object_id;
} wref_object_t;

typedef struct {
    int target_id;
    int valid;
} weak_ref_t;

typedef struct {
    wref_object_t objects[WREF_MAX_OBJECTS];
    weak_ref_t weak_refs[WREF_MAX_WEAK];
    int num_objects;
    int num_weak;
    int next_id;
} wref_system_t;

void wref_init(wref_system_t *ws) {
    ws->num_objects = 0;
    ws->num_weak = 0;
    ws->next_id = 1;
    int i;
    for (i = 0; i < WREF_MAX_OBJECTS; i++) {
        ws->objects[i].active = 0;
    }
    for (i = 0; i < WREF_MAX_WEAK; i++) {
        ws->weak_refs[i].valid = 0;
    }
}

int wref_create(wref_system_t *ws, int value) {
    int i;
    for (i = 0; i < WREF_MAX_OBJECTS; i++) {
        if (!ws->objects[i].active) {
            ws->objects[i].value = value;
            ws->objects[i].strong_count = 1;
            ws->objects[i].active = 1;
            ws->objects[i].object_id = ws->next_id++;
            ws->num_objects++;
            return i;
        }
    }
    return -1;
}

int wref_create_weak(wref_system_t *ws, int obj_idx) {
    if (obj_idx < 0 || obj_idx >= WREF_MAX_OBJECTS) return -1;
    if (!ws->objects[obj_idx].active) return -1;
    int i;
    for (i = 0; i < WREF_MAX_WEAK; i++) {
        if (!ws->weak_refs[i].valid) {
            ws->weak_refs[i].target_id = ws->objects[obj_idx].object_id;
            ws->weak_refs[i].valid = 1;
            ws->num_weak++;
            return i;
        }
    }
    return -1;
}

int wref_upgrade_weak(const wref_system_t *ws, int weak_idx) {
    if (weak_idx < 0 || weak_idx >= WREF_MAX_WEAK) return -1;
    if (!ws->weak_refs[weak_idx].valid) return -1;
    int target_id = ws->weak_refs[weak_idx].target_id;
    int i;
    for (i = 0; i < WREF_MAX_OBJECTS; i++) {
        if (ws->objects[i].active && ws->objects[i].object_id == target_id) {
            return i;
        }
    }
    return -1;
}

void wref_destroy(wref_system_t *ws, int obj_idx) {
    if (obj_idx < 0 || obj_idx >= WREF_MAX_OBJECTS) return;
    ws->objects[obj_idx].strong_count--;
    if (ws->objects[obj_idx].strong_count <= 0) {
        ws->objects[obj_idx].active = 0;
        ws->num_objects--;
    }
}

int wref_is_alive(const wref_system_t *ws, int weak_idx) {
    int obj_idx = wref_upgrade_weak(ws, weak_idx);
    return obj_idx >= 0;
}

int main(void) {
    wref_system_t ws;
    wref_init(&ws);
    int obj = wref_create(&ws, 42);
    int weak = wref_create_weak(&ws, obj);
    int alive1 = wref_is_alive(&ws, weak);
    wref_destroy(&ws, obj);
    int alive2 = wref_is_alive(&ws, weak);
    return (alive1 == 1 && alive2 == 0) ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C749: Weak reference simulation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C749: Should produce non-empty output");
}

#[test]
fn c750_smart_pointer_unique_ptr_style() {
    let c_code = r#"
typedef unsigned long size_t;

#define UPTR_POOL_SIZE 32

typedef struct {
    int value;
    int active;
    int owner_id;
} uptr_resource_t;

typedef struct {
    int resource_idx;
    int valid;
    int id;
} unique_ptr_t;

typedef struct {
    uptr_resource_t resources[UPTR_POOL_SIZE];
    int num_resources;
    int next_ptr_id;
} uptr_system_t;

void uptr_sys_init(uptr_system_t *sys) {
    sys->num_resources = 0;
    sys->next_ptr_id = 1;
    int i;
    for (i = 0; i < UPTR_POOL_SIZE; i++) {
        sys->resources[i].active = 0;
        sys->resources[i].owner_id = -1;
    }
}

unique_ptr_t uptr_make(uptr_system_t *sys, int value) {
    unique_ptr_t ptr;
    ptr.valid = 0;
    ptr.resource_idx = -1;
    ptr.id = 0;
    int i;
    for (i = 0; i < UPTR_POOL_SIZE; i++) {
        if (!sys->resources[i].active) {
            sys->resources[i].value = value;
            sys->resources[i].active = 1;
            sys->resources[i].owner_id = sys->next_ptr_id;
            ptr.resource_idx = i;
            ptr.valid = 1;
            ptr.id = sys->next_ptr_id;
            sys->next_ptr_id++;
            sys->num_resources++;
            return ptr;
        }
    }
    return ptr;
}

unique_ptr_t uptr_move(uptr_system_t *sys, unique_ptr_t *src) {
    unique_ptr_t dst;
    dst.valid = 0;
    dst.resource_idx = -1;
    dst.id = 0;
    if (!src->valid) return dst;
    dst.resource_idx = src->resource_idx;
    dst.id = sys->next_ptr_id++;
    dst.valid = 1;
    sys->resources[dst.resource_idx].owner_id = dst.id;
    src->valid = 0;
    src->resource_idx = -1;
    return dst;
}

int uptr_get(const uptr_system_t *sys, const unique_ptr_t *ptr) {
    if (!ptr->valid) return -1;
    if (ptr->resource_idx < 0 || ptr->resource_idx >= UPTR_POOL_SIZE) return -1;
    if (!sys->resources[ptr->resource_idx].active) return -1;
    if (sys->resources[ptr->resource_idx].owner_id != ptr->id) return -1;
    return sys->resources[ptr->resource_idx].value;
}

void uptr_reset(uptr_system_t *sys, unique_ptr_t *ptr) {
    if (!ptr->valid) return;
    if (ptr->resource_idx >= 0 && ptr->resource_idx < UPTR_POOL_SIZE) {
        if (sys->resources[ptr->resource_idx].owner_id == ptr->id) {
            sys->resources[ptr->resource_idx].active = 0;
            sys->resources[ptr->resource_idx].owner_id = -1;
            sys->num_resources--;
        }
    }
    ptr->valid = 0;
    ptr->resource_idx = -1;
}

int main(void) {
    uptr_system_t sys;
    uptr_sys_init(&sys);
    unique_ptr_t p1 = uptr_make(&sys, 99);
    int val1 = uptr_get(&sys, &p1);
    unique_ptr_t p2 = uptr_move(&sys, &p1);
    int val_old = uptr_get(&sys, &p1);
    int val_new = uptr_get(&sys, &p2);
    uptr_reset(&sys, &p2);
    return (val1 == 99 && val_old == -1 && val_new == 99 && sys.num_resources == 0) ? 0 : 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C750: Smart pointer (unique_ptr style) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C750: Should produce non-empty output");
}
