//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//! C1701-C1725: Thread Pool and Task Scheduling Systems
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise thread pool implementations, task scheduling algorithms,
//! synchronization primitives, and load balancing strategies -- all expressed as
//! valid C99 with array-based representations (no #include, no pthreads).
//!
//! Organization:
//! - C1701-C1705: Work queue (FIFO, priority, work stealing, batch, cancellation)
//! - C1706-C1710: Thread management (creation, affinity, TLS, shutdown, scaling)
//! - C1711-C1715: Task scheduling (round-robin, weighted fair, deadline, dependency, futures)
//! - C1716-C1720: Synchronization (spinlock, ticket lock, RW lock, barrier, condvar)
//! - C1721-C1725: Load balancing (least-loaded, round-robin assign, hash, adaptive, backpressure)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C1701-C1705: Work Queue
// ============================================================================

/// C1701: FIFO work queue with circular buffer
#[test]
fn c1701_fifo_work_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

typedef struct {
    int task_id;
    int priority;
    int status;
} tp_task_t;

typedef struct {
    tp_task_t tasks[256];
    int head;
    int tail;
    int count;
    int capacity;
} tp_work_queue_t;

void tp_queue_init(tp_work_queue_t *q) {
    q->head = 0;
    q->tail = 0;
    q->count = 0;
    q->capacity = 256;
    int i;
    for (i = 0; i < 256; i++) {
        q->tasks[i].task_id = -1;
        q->tasks[i].priority = 0;
        q->tasks[i].status = 0;
    }
}

int tp_queue_push(tp_work_queue_t *q, int task_id, int priority) {
    if (q->count >= q->capacity) {
        return -1;
    }
    q->tasks[q->tail].task_id = task_id;
    q->tasks[q->tail].priority = priority;
    q->tasks[q->tail].status = 1;
    q->tail = (q->tail + 1) % q->capacity;
    q->count = q->count + 1;
    return 0;
}

int tp_queue_pop(tp_work_queue_t *q) {
    if (q->count <= 0) {
        return -1;
    }
    int task_id = q->tasks[q->head].task_id;
    q->tasks[q->head].status = 0;
    q->head = (q->head + 1) % q->capacity;
    q->count = q->count - 1;
    return task_id;
}

int tp_queue_peek(const tp_work_queue_t *q) {
    if (q->count <= 0) return -1;
    return q->tasks[q->head].task_id;
}

int tp_queue_is_empty(const tp_work_queue_t *q) {
    return q->count == 0;
}

int tp_queue_size(const tp_work_queue_t *q) {
    return q->count;
}

int tp_queue_test(void) {
    tp_work_queue_t q;
    tp_queue_init(&q);
    if (!tp_queue_is_empty(&q)) return -1;
    if (tp_queue_push(&q, 10, 1) != 0) return -2;
    if (tp_queue_push(&q, 20, 2) != 0) return -3;
    if (tp_queue_size(&q) != 2) return -4;
    if (tp_queue_peek(&q) != 10) return -5;
    if (tp_queue_pop(&q) != 10) return -6;
    if (tp_queue_pop(&q) != 20) return -7;
    if (!tp_queue_is_empty(&q)) return -8;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1701: FIFO work queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1701: Output should not be empty");
    assert!(code.contains("fn tp_queue_init"), "C1701: Should contain tp_queue_init");
    assert!(code.contains("fn tp_queue_push"), "C1701: Should contain tp_queue_push");
    assert!(code.contains("fn tp_queue_pop"), "C1701: Should contain tp_queue_pop");
    Ok(())
}

/// C1702: Priority work queue with heap-like selection
#[test]
fn c1702_priority_work_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int task_id;
    int priority;
} tp_prio_entry_t;

typedef struct {
    tp_prio_entry_t entries[128];
    int count;
} tp_prio_queue_t;

void tp_prio_init(tp_prio_queue_t *pq) {
    pq->count = 0;
    int i;
    for (i = 0; i < 128; i++) {
        pq->entries[i].task_id = -1;
        pq->entries[i].priority = 0;
    }
}

int tp_prio_insert(tp_prio_queue_t *pq, int task_id, int priority) {
    if (pq->count >= 128) return -1;
    int pos = pq->count;
    pq->entries[pos].task_id = task_id;
    pq->entries[pos].priority = priority;
    while (pos > 0) {
        int parent = (pos - 1) / 2;
        if (pq->entries[parent].priority >= pq->entries[pos].priority) break;
        tp_prio_entry_t tmp = pq->entries[parent];
        pq->entries[parent] = pq->entries[pos];
        pq->entries[pos] = tmp;
        pos = parent;
    }
    pq->count = pq->count + 1;
    return 0;
}

int tp_prio_extract_max(tp_prio_queue_t *pq) {
    if (pq->count <= 0) return -1;
    int result = pq->entries[0].task_id;
    pq->count = pq->count - 1;
    pq->entries[0] = pq->entries[pq->count];
    int pos = 0;
    while (1) {
        int left = 2 * pos + 1;
        int right = 2 * pos + 2;
        int largest = pos;
        if (left < pq->count && pq->entries[left].priority > pq->entries[largest].priority) {
            largest = left;
        }
        if (right < pq->count && pq->entries[right].priority > pq->entries[largest].priority) {
            largest = right;
        }
        if (largest == pos) break;
        tp_prio_entry_t tmp = pq->entries[pos];
        pq->entries[pos] = pq->entries[largest];
        pq->entries[largest] = tmp;
        pos = largest;
    }
    return result;
}

int tp_prio_peek(const tp_prio_queue_t *pq) {
    if (pq->count <= 0) return -1;
    return pq->entries[0].task_id;
}

int tp_prio_test(void) {
    tp_prio_queue_t pq;
    tp_prio_init(&pq);
    tp_prio_insert(&pq, 1, 10);
    tp_prio_insert(&pq, 2, 30);
    tp_prio_insert(&pq, 3, 20);
    if (tp_prio_peek(&pq) != 2) return -1;
    if (tp_prio_extract_max(&pq) != 2) return -2;
    if (tp_prio_extract_max(&pq) != 3) return -3;
    if (tp_prio_extract_max(&pq) != 1) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1702: Priority work queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1702: Output should not be empty");
    assert!(code.contains("fn tp_prio_init"), "C1702: Should contain tp_prio_init");
    assert!(code.contains("fn tp_prio_insert"), "C1702: Should contain tp_prio_insert");
    assert!(code.contains("fn tp_prio_extract_max"), "C1702: Should contain tp_prio_extract_max");
    Ok(())
}

/// C1703: Work stealing deque (double-ended queue)
#[test]
fn c1703_work_stealing_deque() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int items[256];
    int top;
    int bottom;
} tp_wsdeque_t;

void tp_wsdeque_init(tp_wsdeque_t *d) {
    d->top = 0;
    d->bottom = 0;
    int i;
    for (i = 0; i < 256; i++) {
        d->items[i] = 0;
    }
}

int tp_wsdeque_push_bottom(tp_wsdeque_t *d, int item) {
    if (d->bottom - d->top >= 256) return -1;
    d->items[d->bottom % 256] = item;
    d->bottom = d->bottom + 1;
    return 0;
}

int tp_wsdeque_pop_bottom(tp_wsdeque_t *d) {
    if (d->bottom <= d->top) return -1;
    d->bottom = d->bottom - 1;
    return d->items[d->bottom % 256];
}

int tp_wsdeque_steal(tp_wsdeque_t *d) {
    if (d->top >= d->bottom) return -1;
    int item = d->items[d->top % 256];
    d->top = d->top + 1;
    return item;
}

int tp_wsdeque_size(const tp_wsdeque_t *d) {
    int s = d->bottom - d->top;
    if (s < 0) return 0;
    return s;
}

int tp_wsdeque_is_empty(const tp_wsdeque_t *d) {
    return d->bottom <= d->top;
}

int tp_wsdeque_test(void) {
    tp_wsdeque_t d;
    tp_wsdeque_init(&d);
    if (!tp_wsdeque_is_empty(&d)) return -1;
    tp_wsdeque_push_bottom(&d, 10);
    tp_wsdeque_push_bottom(&d, 20);
    tp_wsdeque_push_bottom(&d, 30);
    if (tp_wsdeque_size(&d) != 3) return -2;
    if (tp_wsdeque_steal(&d) != 10) return -3;
    if (tp_wsdeque_pop_bottom(&d) != 30) return -4;
    if (tp_wsdeque_size(&d) != 1) return -5;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1703: Work stealing deque should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1703: Output should not be empty");
    assert!(code.contains("fn tp_wsdeque_init"), "C1703: Should contain tp_wsdeque_init");
    assert!(code.contains("fn tp_wsdeque_push_bottom"), "C1703: Should contain tp_wsdeque_push_bottom");
    assert!(code.contains("fn tp_wsdeque_steal"), "C1703: Should contain tp_wsdeque_steal");
    Ok(())
}

/// C1704: Batch submission of tasks
#[test]
fn c1704_batch_submission() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int task_ids[64];
    int count;
} tp_batch_t;

typedef struct {
    int queue[512];
    int head;
    int tail;
    int count;
} tp_batch_queue_t;

void tp_batch_init(tp_batch_t *b) {
    b->count = 0;
    int i;
    for (i = 0; i < 64; i++) {
        b->task_ids[i] = 0;
    }
}

int tp_batch_add(tp_batch_t *b, int task_id) {
    if (b->count >= 64) return -1;
    b->task_ids[b->count] = task_id;
    b->count = b->count + 1;
    return 0;
}

void tp_bqueue_init(tp_batch_queue_t *q) {
    q->head = 0;
    q->tail = 0;
    q->count = 0;
}

int tp_batch_submit(tp_batch_queue_t *q, const tp_batch_t *b) {
    int i;
    for (i = 0; i < b->count; i++) {
        if (q->count >= 512) return i;
        q->queue[q->tail] = b->task_ids[i];
        q->tail = (q->tail + 1) % 512;
        q->count = q->count + 1;
    }
    return b->count;
}

int tp_bqueue_pop(tp_batch_queue_t *q) {
    if (q->count <= 0) return -1;
    int val = q->queue[q->head];
    q->head = (q->head + 1) % 512;
    q->count = q->count - 1;
    return val;
}

int tp_batch_test(void) {
    tp_batch_t batch;
    tp_batch_init(&batch);
    tp_batch_add(&batch, 100);
    tp_batch_add(&batch, 200);
    tp_batch_add(&batch, 300);
    tp_batch_queue_t q;
    tp_bqueue_init(&q);
    int submitted = tp_batch_submit(&q, &batch);
    if (submitted != 3) return -1;
    if (q.count != 3) return -2;
    if (tp_bqueue_pop(&q) != 100) return -3;
    if (tp_bqueue_pop(&q) != 200) return -4;
    if (tp_bqueue_pop(&q) != 300) return -5;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1704: Batch submission should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1704: Output should not be empty");
    assert!(code.contains("fn tp_batch_init"), "C1704: Should contain tp_batch_init");
    assert!(code.contains("fn tp_batch_submit"), "C1704: Should contain tp_batch_submit");
    assert!(code.contains("fn tp_bqueue_pop"), "C1704: Should contain tp_bqueue_pop");
    Ok(())
}

/// C1705: Work cancellation with status tracking
#[test]
fn c1705_work_cancellation() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int task_id;
    int status;
    int cancelled;
} tp_cancel_task_t;

typedef struct {
    tp_cancel_task_t tasks[128];
    int count;
} tp_cancel_pool_t;

void tp_cancel_pool_init(tp_cancel_pool_t *pool) {
    pool->count = 0;
    int i;
    for (i = 0; i < 128; i++) {
        pool->tasks[i].task_id = -1;
        pool->tasks[i].status = 0;
        pool->tasks[i].cancelled = 0;
    }
}

int tp_cancel_submit(tp_cancel_pool_t *pool, int task_id) {
    if (pool->count >= 128) return -1;
    pool->tasks[pool->count].task_id = task_id;
    pool->tasks[pool->count].status = 1;
    pool->tasks[pool->count].cancelled = 0;
    pool->count = pool->count + 1;
    return pool->count - 1;
}

int tp_cancel_task(tp_cancel_pool_t *pool, int handle) {
    if (handle < 0 || handle >= pool->count) return -1;
    if (pool->tasks[handle].cancelled) return -2;
    if (pool->tasks[handle].status == 2) return -3;
    pool->tasks[handle].cancelled = 1;
    pool->tasks[handle].status = 3;
    return 0;
}

int tp_cancel_is_cancelled(const tp_cancel_pool_t *pool, int handle) {
    if (handle < 0 || handle >= pool->count) return -1;
    return pool->tasks[handle].cancelled;
}

int tp_cancel_active_count(const tp_cancel_pool_t *pool) {
    int active = 0;
    int i;
    for (i = 0; i < pool->count; i++) {
        if (pool->tasks[i].status == 1 && !pool->tasks[i].cancelled) {
            active = active + 1;
        }
    }
    return active;
}

int tp_cancel_test(void) {
    tp_cancel_pool_t pool;
    tp_cancel_pool_init(&pool);
    int h0 = tp_cancel_submit(&pool, 10);
    int h1 = tp_cancel_submit(&pool, 20);
    int h2 = tp_cancel_submit(&pool, 30);
    if (tp_cancel_active_count(&pool) != 3) return -1;
    if (tp_cancel_task(&pool, h1) != 0) return -2;
    if (!tp_cancel_is_cancelled(&pool, h1)) return -3;
    if (tp_cancel_active_count(&pool) != 2) return -4;
    if (tp_cancel_task(&pool, h1) != -2) return -5;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1705: Work cancellation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1705: Output should not be empty");
    assert!(code.contains("fn tp_cancel_pool_init"), "C1705: Should contain tp_cancel_pool_init");
    assert!(code.contains("fn tp_cancel_submit"), "C1705: Should contain tp_cancel_submit");
    assert!(code.contains("fn tp_cancel_task"), "C1705: Should contain tp_cancel_task");
    Ok(())
}

// ============================================================================
// C1706-C1710: Thread Management
// ============================================================================

/// C1706: Thread creation tracking with thread descriptors
#[test]
fn c1706_thread_creation_tracking() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int thread_id;
    int active;
    int tasks_completed;
    int cpu_affinity;
} tp_thread_desc_t;

typedef struct {
    tp_thread_desc_t threads[32];
    int count;
    int max_threads;
} tp_thread_pool_t;

void tp_tpool_init(tp_thread_pool_t *pool, int max_threads) {
    pool->count = 0;
    pool->max_threads = max_threads;
    if (pool->max_threads > 32) pool->max_threads = 32;
    int i;
    for (i = 0; i < 32; i++) {
        pool->threads[i].thread_id = -1;
        pool->threads[i].active = 0;
        pool->threads[i].tasks_completed = 0;
        pool->threads[i].cpu_affinity = -1;
    }
}

int tp_tpool_create_thread(tp_thread_pool_t *pool) {
    if (pool->count >= pool->max_threads) return -1;
    int idx = pool->count;
    pool->threads[idx].thread_id = idx;
    pool->threads[idx].active = 1;
    pool->threads[idx].tasks_completed = 0;
    pool->count = pool->count + 1;
    return idx;
}

int tp_tpool_destroy_thread(tp_thread_pool_t *pool, int idx) {
    if (idx < 0 || idx >= pool->count) return -1;
    if (!pool->threads[idx].active) return -2;
    pool->threads[idx].active = 0;
    return 0;
}

int tp_tpool_active_count(const tp_thread_pool_t *pool) {
    int active = 0;
    int i;
    for (i = 0; i < pool->count; i++) {
        if (pool->threads[i].active) active = active + 1;
    }
    return active;
}

int tp_tpool_total_completed(const tp_thread_pool_t *pool) {
    int total = 0;
    int i;
    for (i = 0; i < pool->count; i++) {
        total = total + pool->threads[i].tasks_completed;
    }
    return total;
}

int tp_tpool_test(void) {
    tp_thread_pool_t pool;
    tp_tpool_init(&pool, 4);
    int t0 = tp_tpool_create_thread(&pool);
    int t1 = tp_tpool_create_thread(&pool);
    if (tp_tpool_active_count(&pool) != 2) return -1;
    pool.threads[t0].tasks_completed = 5;
    pool.threads[t1].tasks_completed = 3;
    if (tp_tpool_total_completed(&pool) != 8) return -2;
    tp_tpool_destroy_thread(&pool, t0);
    if (tp_tpool_active_count(&pool) != 1) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1706: Thread creation tracking should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1706: Output should not be empty");
    assert!(code.contains("fn tp_tpool_init"), "C1706: Should contain tp_tpool_init");
    assert!(code.contains("fn tp_tpool_create_thread"), "C1706: Should contain tp_tpool_create_thread");
    assert!(code.contains("fn tp_tpool_active_count"), "C1706: Should contain tp_tpool_active_count");
    Ok(())
}

/// C1707: Thread affinity assignment
#[test]
fn c1707_thread_affinity() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int thread_id;
    int cpu_id;
    int pinned;
} tp_affinity_entry_t;

typedef struct {
    tp_affinity_entry_t entries[32];
    int count;
    int num_cpus;
} tp_affinity_map_t;

void tp_affinity_init(tp_affinity_map_t *m, int num_cpus) {
    m->count = 0;
    m->num_cpus = num_cpus;
    int i;
    for (i = 0; i < 32; i++) {
        m->entries[i].thread_id = -1;
        m->entries[i].cpu_id = -1;
        m->entries[i].pinned = 0;
    }
}

int tp_affinity_assign(tp_affinity_map_t *m, int thread_id, int cpu_id) {
    if (m->count >= 32) return -1;
    if (cpu_id < 0 || cpu_id >= m->num_cpus) return -2;
    m->entries[m->count].thread_id = thread_id;
    m->entries[m->count].cpu_id = cpu_id;
    m->entries[m->count].pinned = 1;
    m->count = m->count + 1;
    return 0;
}

int tp_affinity_get_cpu(const tp_affinity_map_t *m, int thread_id) {
    int i;
    for (i = 0; i < m->count; i++) {
        if (m->entries[i].thread_id == thread_id) {
            return m->entries[i].cpu_id;
        }
    }
    return -1;
}

int tp_affinity_cpu_load(const tp_affinity_map_t *m, int cpu_id) {
    int load = 0;
    int i;
    for (i = 0; i < m->count; i++) {
        if (m->entries[i].cpu_id == cpu_id) load = load + 1;
    }
    return load;
}

int tp_affinity_test(void) {
    tp_affinity_map_t m;
    tp_affinity_init(&m, 4);
    tp_affinity_assign(&m, 0, 0);
    tp_affinity_assign(&m, 1, 1);
    tp_affinity_assign(&m, 2, 0);
    if (tp_affinity_get_cpu(&m, 1) != 1) return -1;
    if (tp_affinity_cpu_load(&m, 0) != 2) return -2;
    if (tp_affinity_cpu_load(&m, 1) != 1) return -3;
    if (tp_affinity_assign(&m, 3, 5) != -2) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1707: Thread affinity should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1707: Output should not be empty");
    assert!(code.contains("fn tp_affinity_init"), "C1707: Should contain tp_affinity_init");
    assert!(code.contains("fn tp_affinity_assign"), "C1707: Should contain tp_affinity_assign");
    assert!(code.contains("fn tp_affinity_get_cpu"), "C1707: Should contain tp_affinity_get_cpu");
    Ok(())
}

/// C1708: Thread-local storage simulation
#[test]
fn c1708_thread_local_storage() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int key;
    int value;
    int valid;
} tp_tls_slot_t;

typedef struct {
    tp_tls_slot_t slots[16];
    int thread_id;
    int slot_count;
} tp_tls_t;

void tp_tls_init(tp_tls_t *tls, int thread_id) {
    tls->thread_id = thread_id;
    tls->slot_count = 0;
    int i;
    for (i = 0; i < 16; i++) {
        tls->slots[i].key = -1;
        tls->slots[i].value = 0;
        tls->slots[i].valid = 0;
    }
}

int tp_tls_set(tp_tls_t *tls, int key, int value) {
    int i;
    for (i = 0; i < tls->slot_count; i++) {
        if (tls->slots[i].key == key) {
            tls->slots[i].value = value;
            return 0;
        }
    }
    if (tls->slot_count >= 16) return -1;
    tls->slots[tls->slot_count].key = key;
    tls->slots[tls->slot_count].value = value;
    tls->slots[tls->slot_count].valid = 1;
    tls->slot_count = tls->slot_count + 1;
    return 0;
}

int tp_tls_get(const tp_tls_t *tls, int key) {
    int i;
    for (i = 0; i < tls->slot_count; i++) {
        if (tls->slots[i].key == key && tls->slots[i].valid) {
            return tls->slots[i].value;
        }
    }
    return -1;
}

int tp_tls_delete(tp_tls_t *tls, int key) {
    int i;
    for (i = 0; i < tls->slot_count; i++) {
        if (tls->slots[i].key == key) {
            tls->slots[i].valid = 0;
            return 0;
        }
    }
    return -1;
}

int tp_tls_test(void) {
    tp_tls_t tls;
    tp_tls_init(&tls, 0);
    tp_tls_set(&tls, 1, 100);
    tp_tls_set(&tls, 2, 200);
    if (tp_tls_get(&tls, 1) != 100) return -1;
    if (tp_tls_get(&tls, 2) != 200) return -2;
    tp_tls_set(&tls, 1, 150);
    if (tp_tls_get(&tls, 1) != 150) return -3;
    tp_tls_delete(&tls, 2);
    if (tp_tls_get(&tls, 2) != -1) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1708: Thread-local storage should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1708: Output should not be empty");
    assert!(code.contains("fn tp_tls_init"), "C1708: Should contain tp_tls_init");
    assert!(code.contains("fn tp_tls_set"), "C1708: Should contain tp_tls_set");
    assert!(code.contains("fn tp_tls_get"), "C1708: Should contain tp_tls_get");
    Ok(())
}

/// C1709: Graceful shutdown with drain
#[test]
fn c1709_graceful_shutdown() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int pending_tasks[64];
    int pending_count;
    int shutdown_requested;
    int drained;
    int active_workers;
} tp_shutdown_t;

void tp_shutdown_init(tp_shutdown_t *s, int workers) {
    s->pending_count = 0;
    s->shutdown_requested = 0;
    s->drained = 0;
    s->active_workers = workers;
    int i;
    for (i = 0; i < 64; i++) {
        s->pending_tasks[i] = 0;
    }
}

int tp_shutdown_add_task(tp_shutdown_t *s, int task_id) {
    if (s->shutdown_requested) return -1;
    if (s->pending_count >= 64) return -2;
    s->pending_tasks[s->pending_count] = task_id;
    s->pending_count = s->pending_count + 1;
    return 0;
}

void tp_shutdown_request(tp_shutdown_t *s) {
    s->shutdown_requested = 1;
}

int tp_shutdown_drain(tp_shutdown_t *s) {
    if (!s->shutdown_requested) return -1;
    int drained = s->pending_count;
    s->pending_count = 0;
    s->drained = 1;
    return drained;
}

int tp_shutdown_is_complete(const tp_shutdown_t *s) {
    return s->shutdown_requested && s->drained && s->pending_count == 0;
}

int tp_shutdown_test(void) {
    tp_shutdown_t s;
    tp_shutdown_init(&s, 4);
    tp_shutdown_add_task(&s, 1);
    tp_shutdown_add_task(&s, 2);
    tp_shutdown_add_task(&s, 3);
    if (tp_shutdown_is_complete(&s)) return -1;
    tp_shutdown_request(&s);
    if (tp_shutdown_add_task(&s, 4) != -1) return -2;
    int drained = tp_shutdown_drain(&s);
    if (drained != 3) return -3;
    if (!tp_shutdown_is_complete(&s)) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1709: Graceful shutdown should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1709: Output should not be empty");
    assert!(code.contains("fn tp_shutdown_init"), "C1709: Should contain tp_shutdown_init");
    assert!(code.contains("fn tp_shutdown_request"), "C1709: Should contain tp_shutdown_request");
    assert!(code.contains("fn tp_shutdown_drain"), "C1709: Should contain tp_shutdown_drain");
    Ok(())
}

/// C1710: Thread pool scaling (grow/shrink)
#[test]
fn c1710_thread_pool_scaling() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int workers[64];
    int worker_count;
    int min_workers;
    int max_workers;
    int queue_depth;
    int scale_threshold;
} tp_scaler_t;

void tp_scaler_init(tp_scaler_t *s, int min_w, int max_w, int threshold) {
    s->worker_count = min_w;
    s->min_workers = min_w;
    s->max_workers = max_w;
    s->queue_depth = 0;
    s->scale_threshold = threshold;
    int i;
    for (i = 0; i < 64; i++) {
        s->workers[i] = (i < min_w) ? 1 : 0;
    }
}

int tp_scaler_scale_up(tp_scaler_t *s) {
    if (s->worker_count >= s->max_workers) return -1;
    s->workers[s->worker_count] = 1;
    s->worker_count = s->worker_count + 1;
    return s->worker_count;
}

int tp_scaler_scale_down(tp_scaler_t *s) {
    if (s->worker_count <= s->min_workers) return -1;
    s->worker_count = s->worker_count - 1;
    s->workers[s->worker_count] = 0;
    return s->worker_count;
}

int tp_scaler_evaluate(tp_scaler_t *s) {
    if (s->queue_depth > s->scale_threshold * s->worker_count) {
        return tp_scaler_scale_up(s);
    }
    if (s->queue_depth < s->worker_count && s->worker_count > s->min_workers) {
        return tp_scaler_scale_down(s);
    }
    return 0;
}

int tp_scaler_test(void) {
    tp_scaler_t s;
    tp_scaler_init(&s, 2, 8, 5);
    if (s.worker_count != 2) return -1;
    s.queue_depth = 20;
    tp_scaler_evaluate(&s);
    if (s.worker_count != 3) return -2;
    s.queue_depth = 1;
    tp_scaler_evaluate(&s);
    if (s.worker_count != 2) return -3;
    if (tp_scaler_scale_down(&s) != -1) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1710: Thread pool scaling should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1710: Output should not be empty");
    assert!(code.contains("fn tp_scaler_init"), "C1710: Should contain tp_scaler_init");
    assert!(code.contains("fn tp_scaler_scale_up"), "C1710: Should contain tp_scaler_scale_up");
    assert!(code.contains("fn tp_scaler_evaluate"), "C1710: Should contain tp_scaler_evaluate");
    Ok(())
}

// ============================================================================
// C1711-C1715: Task Scheduling
// ============================================================================

/// C1711: Round-robin scheduler
#[test]
fn c1711_round_robin_scheduler() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int task_ids[128];
    int count;
    int current;
    int quantum;
} tp_rr_sched_t;

void tp_rr_init(tp_rr_sched_t *s, int quantum) {
    s->count = 0;
    s->current = 0;
    s->quantum = quantum;
    int i;
    for (i = 0; i < 128; i++) {
        s->task_ids[i] = -1;
    }
}

int tp_rr_add_task(tp_rr_sched_t *s, int task_id) {
    if (s->count >= 128) return -1;
    s->task_ids[s->count] = task_id;
    s->count = s->count + 1;
    return 0;
}

int tp_rr_next(tp_rr_sched_t *s) {
    if (s->count <= 0) return -1;
    int task = s->task_ids[s->current];
    s->current = (s->current + 1) % s->count;
    return task;
}

int tp_rr_remove_task(tp_rr_sched_t *s, int task_id) {
    int i;
    for (i = 0; i < s->count; i++) {
        if (s->task_ids[i] == task_id) {
            int j;
            for (j = i; j < s->count - 1; j++) {
                s->task_ids[j] = s->task_ids[j + 1];
            }
            s->count = s->count - 1;
            if (s->current >= s->count && s->count > 0) {
                s->current = 0;
            }
            return 0;
        }
    }
    return -1;
}

int tp_rr_test(void) {
    tp_rr_sched_t s;
    tp_rr_init(&s, 10);
    tp_rr_add_task(&s, 1);
    tp_rr_add_task(&s, 2);
    tp_rr_add_task(&s, 3);
    if (tp_rr_next(&s) != 1) return -1;
    if (tp_rr_next(&s) != 2) return -2;
    if (tp_rr_next(&s) != 3) return -3;
    if (tp_rr_next(&s) != 1) return -4;
    tp_rr_remove_task(&s, 2);
    if (s.count != 2) return -5;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1711: Round-robin scheduler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1711: Output should not be empty");
    assert!(code.contains("fn tp_rr_init"), "C1711: Should contain tp_rr_init");
    assert!(code.contains("fn tp_rr_next"), "C1711: Should contain tp_rr_next");
    assert!(code.contains("fn tp_rr_add_task"), "C1711: Should contain tp_rr_add_task");
    Ok(())
}

/// C1712: Weighted fair queue scheduler
#[test]
fn c1712_weighted_fair_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int task_id;
    int weight;
    int remaining;
} tp_wfq_entry_t;

typedef struct {
    tp_wfq_entry_t entries[64];
    int count;
} tp_wfq_sched_t;

void tp_wfq_init(tp_wfq_sched_t *s) {
    s->count = 0;
    int i;
    for (i = 0; i < 64; i++) {
        s->entries[i].task_id = -1;
        s->entries[i].weight = 0;
        s->entries[i].remaining = 0;
    }
}

int tp_wfq_add(tp_wfq_sched_t *s, int task_id, int weight) {
    if (s->count >= 64) return -1;
    if (weight <= 0) return -2;
    s->entries[s->count].task_id = task_id;
    s->entries[s->count].weight = weight;
    s->entries[s->count].remaining = weight;
    s->count = s->count + 1;
    return 0;
}

int tp_wfq_next(tp_wfq_sched_t *s) {
    if (s->count <= 0) return -1;
    int best = -1;
    int best_remaining = 0;
    int i;
    for (i = 0; i < s->count; i++) {
        if (s->entries[i].remaining > best_remaining) {
            best = i;
            best_remaining = s->entries[i].remaining;
        }
    }
    if (best < 0) {
        for (i = 0; i < s->count; i++) {
            s->entries[i].remaining = s->entries[i].weight;
        }
        return tp_wfq_next(s);
    }
    s->entries[best].remaining = s->entries[best].remaining - 1;
    return s->entries[best].task_id;
}

int tp_wfq_total_weight(const tp_wfq_sched_t *s) {
    int total = 0;
    int i;
    for (i = 0; i < s->count; i++) {
        total = total + s->entries[i].weight;
    }
    return total;
}

int tp_wfq_test(void) {
    tp_wfq_sched_t s;
    tp_wfq_init(&s);
    tp_wfq_add(&s, 10, 3);
    tp_wfq_add(&s, 20, 1);
    if (tp_wfq_total_weight(&s) != 4) return -1;
    if (tp_wfq_next(&s) != 10) return -2;
    if (tp_wfq_next(&s) != 10) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1712: Weighted fair queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1712: Output should not be empty");
    assert!(code.contains("fn tp_wfq_init"), "C1712: Should contain tp_wfq_init");
    assert!(code.contains("fn tp_wfq_add"), "C1712: Should contain tp_wfq_add");
    assert!(code.contains("fn tp_wfq_next"), "C1712: Should contain tp_wfq_next");
    Ok(())
}

/// C1713: Deadline scheduler (earliest deadline first)
#[test]
fn c1713_deadline_scheduler() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int task_id;
    int deadline;
    int completed;
} tp_deadline_task_t;

typedef struct {
    tp_deadline_task_t tasks[64];
    int count;
} tp_deadline_sched_t;

void tp_deadline_init(tp_deadline_sched_t *s) {
    s->count = 0;
    int i;
    for (i = 0; i < 64; i++) {
        s->tasks[i].task_id = -1;
        s->tasks[i].deadline = 0;
        s->tasks[i].completed = 0;
    }
}

int tp_deadline_add(tp_deadline_sched_t *s, int task_id, int deadline) {
    if (s->count >= 64) return -1;
    s->tasks[s->count].task_id = task_id;
    s->tasks[s->count].deadline = deadline;
    s->tasks[s->count].completed = 0;
    s->count = s->count + 1;
    return 0;
}

int tp_deadline_next(tp_deadline_sched_t *s) {
    int best = -1;
    int best_deadline = 2147483647;
    int i;
    for (i = 0; i < s->count; i++) {
        if (!s->tasks[i].completed && s->tasks[i].deadline < best_deadline) {
            best = i;
            best_deadline = s->tasks[i].deadline;
        }
    }
    if (best < 0) return -1;
    s->tasks[best].completed = 1;
    return s->tasks[best].task_id;
}

int tp_deadline_pending(const tp_deadline_sched_t *s) {
    int pending = 0;
    int i;
    for (i = 0; i < s->count; i++) {
        if (!s->tasks[i].completed) pending = pending + 1;
    }
    return pending;
}

int tp_deadline_test(void) {
    tp_deadline_sched_t s;
    tp_deadline_init(&s);
    tp_deadline_add(&s, 1, 100);
    tp_deadline_add(&s, 2, 50);
    tp_deadline_add(&s, 3, 75);
    if (tp_deadline_next(&s) != 2) return -1;
    if (tp_deadline_next(&s) != 3) return -2;
    if (tp_deadline_next(&s) != 1) return -3;
    if (tp_deadline_pending(&s) != 0) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1713: Deadline scheduler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1713: Output should not be empty");
    assert!(code.contains("fn tp_deadline_init"), "C1713: Should contain tp_deadline_init");
    assert!(code.contains("fn tp_deadline_add"), "C1713: Should contain tp_deadline_add");
    assert!(code.contains("fn tp_deadline_next"), "C1713: Should contain tp_deadline_next");
    Ok(())
}

/// C1714: Dependency graph executor (topological ordering)
#[test]
fn c1714_dependency_graph_executor() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int from;
    int to;
} tp_dep_edge_t;

typedef struct {
    int task_ids[32];
    int task_count;
    tp_dep_edge_t edges[64];
    int edge_count;
    int completed[32];
    int in_degree[32];
} tp_dep_graph_t;

void tp_dep_init(tp_dep_graph_t *g) {
    g->task_count = 0;
    g->edge_count = 0;
    int i;
    for (i = 0; i < 32; i++) {
        g->task_ids[i] = -1;
        g->completed[i] = 0;
        g->in_degree[i] = 0;
    }
}

int tp_dep_add_task(tp_dep_graph_t *g, int task_id) {
    if (g->task_count >= 32) return -1;
    g->task_ids[g->task_count] = task_id;
    g->task_count = g->task_count + 1;
    return 0;
}

int tp_dep_add_edge(tp_dep_graph_t *g, int from, int to) {
    if (g->edge_count >= 64) return -1;
    g->edges[g->edge_count].from = from;
    g->edges[g->edge_count].to = to;
    g->edge_count = g->edge_count + 1;
    int i;
    for (i = 0; i < g->task_count; i++) {
        if (g->task_ids[i] == to) {
            g->in_degree[i] = g->in_degree[i] + 1;
            break;
        }
    }
    return 0;
}

int tp_dep_find_ready(const tp_dep_graph_t *g) {
    int i;
    for (i = 0; i < g->task_count; i++) {
        if (!g->completed[i] && g->in_degree[i] == 0) {
            return g->task_ids[i];
        }
    }
    return -1;
}

int tp_dep_complete(tp_dep_graph_t *g, int task_id) {
    int idx = -1;
    int i;
    for (i = 0; i < g->task_count; i++) {
        if (g->task_ids[i] == task_id) {
            idx = i;
            break;
        }
    }
    if (idx < 0) return -1;
    g->completed[idx] = 1;
    int e;
    for (e = 0; e < g->edge_count; e++) {
        if (g->edges[e].from == task_id) {
            int j;
            for (j = 0; j < g->task_count; j++) {
                if (g->task_ids[j] == g->edges[e].to) {
                    g->in_degree[j] = g->in_degree[j] - 1;
                    break;
                }
            }
        }
    }
    return 0;
}

int tp_dep_test(void) {
    tp_dep_graph_t g;
    tp_dep_init(&g);
    tp_dep_add_task(&g, 1);
    tp_dep_add_task(&g, 2);
    tp_dep_add_task(&g, 3);
    tp_dep_add_edge(&g, 1, 2);
    tp_dep_add_edge(&g, 1, 3);
    if (tp_dep_find_ready(&g) != 1) return -1;
    tp_dep_complete(&g, 1);
    int r = tp_dep_find_ready(&g);
    if (r != 2 && r != 3) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1714: Dependency graph executor should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1714: Output should not be empty");
    assert!(code.contains("fn tp_dep_init"), "C1714: Should contain tp_dep_init");
    assert!(code.contains("fn tp_dep_add_edge"), "C1714: Should contain tp_dep_add_edge");
    assert!(code.contains("fn tp_dep_find_ready"), "C1714: Should contain tp_dep_find_ready");
    Ok(())
}

/// C1715: Futures and promises simulation
#[test]
fn c1715_futures_promises() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int id;
    int resolved;
    int value;
    int error;
} tp_future_t;

typedef struct {
    tp_future_t futures[64];
    int count;
} tp_future_pool_t;

void tp_future_pool_init(tp_future_pool_t *pool) {
    pool->count = 0;
    int i;
    for (i = 0; i < 64; i++) {
        pool->futures[i].id = -1;
        pool->futures[i].resolved = 0;
        pool->futures[i].value = 0;
        pool->futures[i].error = 0;
    }
}

int tp_future_create(tp_future_pool_t *pool) {
    if (pool->count >= 64) return -1;
    int idx = pool->count;
    pool->futures[idx].id = idx;
    pool->futures[idx].resolved = 0;
    pool->futures[idx].value = 0;
    pool->futures[idx].error = 0;
    pool->count = pool->count + 1;
    return idx;
}

int tp_future_resolve(tp_future_pool_t *pool, int id, int value) {
    if (id < 0 || id >= pool->count) return -1;
    if (pool->futures[id].resolved) return -2;
    pool->futures[id].resolved = 1;
    pool->futures[id].value = value;
    return 0;
}

int tp_future_reject(tp_future_pool_t *pool, int id, int error_code) {
    if (id < 0 || id >= pool->count) return -1;
    if (pool->futures[id].resolved) return -2;
    pool->futures[id].resolved = 1;
    pool->futures[id].error = error_code;
    return 0;
}

int tp_future_is_ready(const tp_future_pool_t *pool, int id) {
    if (id < 0 || id >= pool->count) return -1;
    return pool->futures[id].resolved;
}

int tp_future_get(const tp_future_pool_t *pool, int id) {
    if (id < 0 || id >= pool->count) return -1;
    if (!pool->futures[id].resolved) return -2;
    if (pool->futures[id].error != 0) return -3;
    return pool->futures[id].value;
}

int tp_future_test(void) {
    tp_future_pool_t pool;
    tp_future_pool_init(&pool);
    int f1 = tp_future_create(&pool);
    int f2 = tp_future_create(&pool);
    if (tp_future_is_ready(&pool, f1)) return -1;
    tp_future_resolve(&pool, f1, 42);
    if (!tp_future_is_ready(&pool, f1)) return -2;
    if (tp_future_get(&pool, f1) != 42) return -3;
    tp_future_reject(&pool, f2, 99);
    if (tp_future_get(&pool, f2) != -3) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1715: Futures/promises should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1715: Output should not be empty");
    assert!(code.contains("fn tp_future_pool_init"), "C1715: Should contain tp_future_pool_init");
    assert!(code.contains("fn tp_future_create"), "C1715: Should contain tp_future_create");
    assert!(code.contains("fn tp_future_resolve"), "C1715: Should contain tp_future_resolve");
    Ok(())
}

// ============================================================================
// C1716-C1720: Synchronization
// ============================================================================

/// C1716: Spinlock with backoff
#[test]
fn c1716_spinlock() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int locked;
    int owner;
    int spin_count;
} tp_spinlock_t;

void tp_spinlock_init(tp_spinlock_t *lock) {
    lock->locked = 0;
    lock->owner = -1;
    lock->spin_count = 0;
}

int tp_spinlock_try_lock(tp_spinlock_t *lock, int thread_id) {
    if (lock->locked) return -1;
    lock->locked = 1;
    lock->owner = thread_id;
    return 0;
}

int tp_spinlock_lock(tp_spinlock_t *lock, int thread_id) {
    int spins = 0;
    while (lock->locked) {
        spins = spins + 1;
        if (spins > 100000) return -1;
    }
    lock->locked = 1;
    lock->owner = thread_id;
    lock->spin_count = lock->spin_count + spins;
    return 0;
}

int tp_spinlock_unlock(tp_spinlock_t *lock, int thread_id) {
    if (!lock->locked) return -1;
    if (lock->owner != thread_id) return -2;
    lock->locked = 0;
    lock->owner = -1;
    return 0;
}

int tp_spinlock_is_locked(const tp_spinlock_t *lock) {
    return lock->locked;
}

int tp_spinlock_test(void) {
    tp_spinlock_t lock;
    tp_spinlock_init(&lock);
    if (tp_spinlock_is_locked(&lock)) return -1;
    if (tp_spinlock_lock(&lock, 0) != 0) return -2;
    if (!tp_spinlock_is_locked(&lock)) return -3;
    if (tp_spinlock_try_lock(&lock, 1) != -1) return -4;
    if (tp_spinlock_unlock(&lock, 1) != -2) return -5;
    if (tp_spinlock_unlock(&lock, 0) != 0) return -6;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1716: Spinlock should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1716: Output should not be empty");
    assert!(code.contains("fn tp_spinlock_init"), "C1716: Should contain tp_spinlock_init");
    assert!(code.contains("fn tp_spinlock_lock"), "C1716: Should contain tp_spinlock_lock");
    assert!(code.contains("fn tp_spinlock_unlock"), "C1716: Should contain tp_spinlock_unlock");
    Ok(())
}

/// C1717: Ticket lock with fair ordering
#[test]
fn c1717_ticket_lock() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int next_ticket;
    int now_serving;
    int holder;
} tp_ticket_lock_t;

void tp_ticket_init(tp_ticket_lock_t *lock) {
    lock->next_ticket = 0;
    lock->now_serving = 0;
    lock->holder = -1;
}

int tp_ticket_acquire(tp_ticket_lock_t *lock, int thread_id) {
    int my_ticket = lock->next_ticket;
    lock->next_ticket = my_ticket + 1;
    int spins = 0;
    while (lock->now_serving != my_ticket) {
        spins = spins + 1;
        if (spins > 100000) return -1;
    }
    lock->holder = thread_id;
    return my_ticket;
}

void tp_ticket_release(tp_ticket_lock_t *lock) {
    lock->holder = -1;
    lock->now_serving = lock->now_serving + 1;
}

int tp_ticket_is_locked(const tp_ticket_lock_t *lock) {
    return lock->next_ticket != lock->now_serving;
}

int tp_ticket_waiters(const tp_ticket_lock_t *lock) {
    return lock->next_ticket - lock->now_serving;
}

int tp_ticket_test(void) {
    tp_ticket_lock_t lock;
    tp_ticket_init(&lock);
    if (tp_ticket_is_locked(&lock)) return -1;
    int t = tp_ticket_acquire(&lock, 0);
    if (t != 0) return -2;
    if (lock.holder != 0) return -3;
    if (tp_ticket_waiters(&lock) != 1) return -4;
    tp_ticket_release(&lock);
    if (tp_ticket_is_locked(&lock)) return -5;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1717: Ticket lock should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1717: Output should not be empty");
    assert!(code.contains("fn tp_ticket_init"), "C1717: Should contain tp_ticket_init");
    assert!(code.contains("fn tp_ticket_acquire"), "C1717: Should contain tp_ticket_acquire");
    assert!(code.contains("fn tp_ticket_release"), "C1717: Should contain tp_ticket_release");
    Ok(())
}

/// C1718: Read-write lock
#[test]
fn c1718_read_write_lock() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int readers;
    int writer;
    int write_waiters;
    int read_count;
    int write_count;
} tp_rwlock_t;

void tp_rwlock_init(tp_rwlock_t *rw) {
    rw->readers = 0;
    rw->writer = 0;
    rw->write_waiters = 0;
    rw->read_count = 0;
    rw->write_count = 0;
}

int tp_rwlock_read_lock(tp_rwlock_t *rw) {
    if (rw->writer || rw->write_waiters > 0) return -1;
    rw->readers = rw->readers + 1;
    rw->read_count = rw->read_count + 1;
    return 0;
}

void tp_rwlock_read_unlock(tp_rwlock_t *rw) {
    if (rw->readers > 0) {
        rw->readers = rw->readers - 1;
    }
}

int tp_rwlock_write_lock(tp_rwlock_t *rw) {
    if (rw->writer || rw->readers > 0) {
        rw->write_waiters = rw->write_waiters + 1;
        return -1;
    }
    rw->writer = 1;
    rw->write_count = rw->write_count + 1;
    return 0;
}

void tp_rwlock_write_unlock(tp_rwlock_t *rw) {
    rw->writer = 0;
}

int tp_rwlock_is_idle(const tp_rwlock_t *rw) {
    return rw->readers == 0 && rw->writer == 0;
}

int tp_rwlock_test(void) {
    tp_rwlock_t rw;
    tp_rwlock_init(&rw);
    if (!tp_rwlock_is_idle(&rw)) return -1;
    if (tp_rwlock_read_lock(&rw) != 0) return -2;
    if (tp_rwlock_read_lock(&rw) != 0) return -3;
    if (rw.readers != 2) return -4;
    tp_rwlock_read_unlock(&rw);
    tp_rwlock_read_unlock(&rw);
    if (tp_rwlock_write_lock(&rw) != 0) return -5;
    tp_rwlock_write_unlock(&rw);
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1718: Read-write lock should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1718: Output should not be empty");
    assert!(code.contains("fn tp_rwlock_init"), "C1718: Should contain tp_rwlock_init");
    assert!(code.contains("fn tp_rwlock_read_lock"), "C1718: Should contain tp_rwlock_read_lock");
    assert!(code.contains("fn tp_rwlock_write_lock"), "C1718: Should contain tp_rwlock_write_lock");
    Ok(())
}

/// C1719: Barrier synchronization
#[test]
fn c1719_barrier() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int count;
    int threshold;
    int generation;
    int waiting;
} tp_barrier_t;

void tp_barrier_init(tp_barrier_t *b, int threshold) {
    b->count = 0;
    b->threshold = threshold;
    b->generation = 0;
    b->waiting = 0;
}

int tp_barrier_wait(tp_barrier_t *b) {
    b->count = b->count + 1;
    b->waiting = b->waiting + 1;
    if (b->count >= b->threshold) {
        b->generation = b->generation + 1;
        b->count = 0;
        b->waiting = 0;
        return 1;
    }
    return 0;
}

int tp_barrier_is_complete(const tp_barrier_t *b) {
    return b->count == 0 && b->generation > 0;
}

int tp_barrier_generation(const tp_barrier_t *b) {
    return b->generation;
}

int tp_barrier_waiting_count(const tp_barrier_t *b) {
    return b->waiting;
}

int tp_barrier_test(void) {
    tp_barrier_t b;
    tp_barrier_init(&b, 3);
    if (tp_barrier_wait(&b) != 0) return -1;
    if (tp_barrier_wait(&b) != 0) return -2;
    if (tp_barrier_wait(&b) != 1) return -3;
    if (tp_barrier_generation(&b) != 1) return -4;
    if (tp_barrier_wait(&b) != 0) return -5;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1719: Barrier should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1719: Output should not be empty");
    assert!(code.contains("fn tp_barrier_init"), "C1719: Should contain tp_barrier_init");
    assert!(code.contains("fn tp_barrier_wait"), "C1719: Should contain tp_barrier_wait");
    assert!(code.contains("fn tp_barrier_generation"), "C1719: Should contain tp_barrier_generation");
    Ok(())
}

/// C1720: Condition variable emulation with wait/notify
#[test]
fn c1720_condition_variable() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int waiters[32];
    int waiter_count;
    int signaled;
    int broadcast;
    int signal_count;
} tp_condvar_t;

void tp_condvar_init(tp_condvar_t *cv) {
    cv->waiter_count = 0;
    cv->signaled = 0;
    cv->broadcast = 0;
    cv->signal_count = 0;
    int i;
    for (i = 0; i < 32; i++) {
        cv->waiters[i] = -1;
    }
}

int tp_condvar_wait(tp_condvar_t *cv, int thread_id) {
    if (cv->waiter_count >= 32) return -1;
    cv->waiters[cv->waiter_count] = thread_id;
    cv->waiter_count = cv->waiter_count + 1;
    return 0;
}

int tp_condvar_signal(tp_condvar_t *cv) {
    if (cv->waiter_count <= 0) return -1;
    cv->signaled = 1;
    cv->signal_count = cv->signal_count + 1;
    cv->waiter_count = cv->waiter_count - 1;
    int i;
    for (i = 0; i < cv->waiter_count; i++) {
        cv->waiters[i] = cv->waiters[i + 1];
    }
    return 0;
}

int tp_condvar_broadcast(tp_condvar_t *cv) {
    int woken = cv->waiter_count;
    cv->waiter_count = 0;
    cv->broadcast = 1;
    cv->signal_count = cv->signal_count + woken;
    return woken;
}

int tp_condvar_has_waiters(const tp_condvar_t *cv) {
    return cv->waiter_count > 0;
}

int tp_condvar_test(void) {
    tp_condvar_t cv;
    tp_condvar_init(&cv);
    tp_condvar_wait(&cv, 0);
    tp_condvar_wait(&cv, 1);
    tp_condvar_wait(&cv, 2);
    if (!tp_condvar_has_waiters(&cv)) return -1;
    tp_condvar_signal(&cv);
    if (cv.waiter_count != 2) return -2;
    int woken = tp_condvar_broadcast(&cv);
    if (woken != 2) return -3;
    if (tp_condvar_has_waiters(&cv)) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1720: Condition variable should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1720: Output should not be empty");
    assert!(code.contains("fn tp_condvar_init"), "C1720: Should contain tp_condvar_init");
    assert!(code.contains("fn tp_condvar_signal"), "C1720: Should contain tp_condvar_signal");
    assert!(code.contains("fn tp_condvar_broadcast"), "C1720: Should contain tp_condvar_broadcast");
    Ok(())
}

// ============================================================================
// C1721-C1725: Load Balancing
// ============================================================================

/// C1721: Least-loaded thread selection
#[test]
fn c1721_least_loaded_selection() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int thread_id;
    int load;
    int capacity;
} tp_worker_stats_t;

typedef struct {
    tp_worker_stats_t workers[16];
    int count;
} tp_lb_least_loaded_t;

void tp_lb_ll_init(tp_lb_least_loaded_t *lb) {
    lb->count = 0;
    int i;
    for (i = 0; i < 16; i++) {
        lb->workers[i].thread_id = -1;
        lb->workers[i].load = 0;
        lb->workers[i].capacity = 100;
    }
}

int tp_lb_ll_add_worker(tp_lb_least_loaded_t *lb, int thread_id, int capacity) {
    if (lb->count >= 16) return -1;
    lb->workers[lb->count].thread_id = thread_id;
    lb->workers[lb->count].load = 0;
    lb->workers[lb->count].capacity = capacity;
    lb->count = lb->count + 1;
    return 0;
}

int tp_lb_ll_select(const tp_lb_least_loaded_t *lb) {
    if (lb->count <= 0) return -1;
    int best = 0;
    int i;
    for (i = 1; i < lb->count; i++) {
        if (lb->workers[i].load < lb->workers[best].load) {
            best = i;
        }
    }
    return lb->workers[best].thread_id;
}

int tp_lb_ll_assign(tp_lb_least_loaded_t *lb, int thread_id) {
    int i;
    for (i = 0; i < lb->count; i++) {
        if (lb->workers[i].thread_id == thread_id) {
            if (lb->workers[i].load >= lb->workers[i].capacity) return -1;
            lb->workers[i].load = lb->workers[i].load + 1;
            return 0;
        }
    }
    return -2;
}

int tp_lb_ll_release(tp_lb_least_loaded_t *lb, int thread_id) {
    int i;
    for (i = 0; i < lb->count; i++) {
        if (lb->workers[i].thread_id == thread_id) {
            if (lb->workers[i].load > 0) {
                lb->workers[i].load = lb->workers[i].load - 1;
            }
            return 0;
        }
    }
    return -1;
}

int tp_lb_ll_test(void) {
    tp_lb_least_loaded_t lb;
    tp_lb_ll_init(&lb);
    tp_lb_ll_add_worker(&lb, 0, 10);
    tp_lb_ll_add_worker(&lb, 1, 10);
    tp_lb_ll_assign(&lb, 0);
    tp_lb_ll_assign(&lb, 0);
    if (tp_lb_ll_select(&lb) != 1) return -1;
    tp_lb_ll_assign(&lb, 1);
    tp_lb_ll_assign(&lb, 1);
    tp_lb_ll_assign(&lb, 1);
    if (tp_lb_ll_select(&lb) != 0) return -2;
    tp_lb_ll_release(&lb, 1);
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1721: Least-loaded selection should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1721: Output should not be empty");
    assert!(code.contains("fn tp_lb_ll_init"), "C1721: Should contain tp_lb_ll_init");
    assert!(code.contains("fn tp_lb_ll_select"), "C1721: Should contain tp_lb_ll_select");
    assert!(code.contains("fn tp_lb_ll_assign"), "C1721: Should contain tp_lb_ll_assign");
    Ok(())
}

/// C1722: Round-robin assignment
#[test]
fn c1722_round_robin_assignment() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int worker_ids[32];
    int worker_count;
    int next_worker;
    int total_assigned;
} tp_lb_rr_t;

void tp_lb_rr_init(tp_lb_rr_t *lb) {
    lb->worker_count = 0;
    lb->next_worker = 0;
    lb->total_assigned = 0;
    int i;
    for (i = 0; i < 32; i++) {
        lb->worker_ids[i] = -1;
    }
}

int tp_lb_rr_add_worker(tp_lb_rr_t *lb, int worker_id) {
    if (lb->worker_count >= 32) return -1;
    lb->worker_ids[lb->worker_count] = worker_id;
    lb->worker_count = lb->worker_count + 1;
    return 0;
}

int tp_lb_rr_assign(tp_lb_rr_t *lb) {
    if (lb->worker_count <= 0) return -1;
    int selected = lb->worker_ids[lb->next_worker];
    lb->next_worker = (lb->next_worker + 1) % lb->worker_count;
    lb->total_assigned = lb->total_assigned + 1;
    return selected;
}

int tp_lb_rr_remove_worker(tp_lb_rr_t *lb, int worker_id) {
    int i;
    for (i = 0; i < lb->worker_count; i++) {
        if (lb->worker_ids[i] == worker_id) {
            int j;
            for (j = i; j < lb->worker_count - 1; j++) {
                lb->worker_ids[j] = lb->worker_ids[j + 1];
            }
            lb->worker_count = lb->worker_count - 1;
            if (lb->next_worker >= lb->worker_count && lb->worker_count > 0) {
                lb->next_worker = 0;
            }
            return 0;
        }
    }
    return -1;
}

int tp_lb_rr_test(void) {
    tp_lb_rr_t lb;
    tp_lb_rr_init(&lb);
    tp_lb_rr_add_worker(&lb, 10);
    tp_lb_rr_add_worker(&lb, 20);
    tp_lb_rr_add_worker(&lb, 30);
    if (tp_lb_rr_assign(&lb) != 10) return -1;
    if (tp_lb_rr_assign(&lb) != 20) return -2;
    if (tp_lb_rr_assign(&lb) != 30) return -3;
    if (tp_lb_rr_assign(&lb) != 10) return -4;
    tp_lb_rr_remove_worker(&lb, 20);
    if (lb.worker_count != 2) return -5;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1722: Round-robin assignment should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1722: Output should not be empty");
    assert!(code.contains("fn tp_lb_rr_init"), "C1722: Should contain tp_lb_rr_init");
    assert!(code.contains("fn tp_lb_rr_assign"), "C1722: Should contain tp_lb_rr_assign");
    assert!(code.contains("fn tp_lb_rr_add_worker"), "C1722: Should contain tp_lb_rr_add_worker");
    Ok(())
}

/// C1723: Hash-based partitioning
#[test]
fn c1723_hash_based_partitioning() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

typedef struct {
    int partition_loads[16];
    int num_partitions;
    int total_items;
} tp_hash_part_t;

void tp_hash_part_init(tp_hash_part_t *hp, int num_partitions) {
    hp->num_partitions = num_partitions;
    if (hp->num_partitions > 16) hp->num_partitions = 16;
    hp->total_items = 0;
    int i;
    for (i = 0; i < 16; i++) {
        hp->partition_loads[i] = 0;
    }
}

uint32_t tp_hash_func(int key) {
    uint32_t h = (uint32_t)key;
    h = h ^ (h >> 16);
    h = h * 2654435761u;
    h = h ^ (h >> 13);
    return h;
}

int tp_hash_part_assign(tp_hash_part_t *hp, int key) {
    uint32_t h = tp_hash_func(key);
    int partition = (int)(h % (uint32_t)hp->num_partitions);
    hp->partition_loads[partition] = hp->partition_loads[partition] + 1;
    hp->total_items = hp->total_items + 1;
    return partition;
}

int tp_hash_part_get_load(const tp_hash_part_t *hp, int partition) {
    if (partition < 0 || partition >= hp->num_partitions) return -1;
    return hp->partition_loads[partition];
}

int tp_hash_part_max_load(const tp_hash_part_t *hp) {
    int max_load = 0;
    int i;
    for (i = 0; i < hp->num_partitions; i++) {
        if (hp->partition_loads[i] > max_load) {
            max_load = hp->partition_loads[i];
        }
    }
    return max_load;
}

int tp_hash_part_test(void) {
    tp_hash_part_t hp;
    tp_hash_part_init(&hp, 4);
    int p1 = tp_hash_part_assign(&hp, 100);
    int p2 = tp_hash_part_assign(&hp, 100);
    if (p1 != p2) return -1;
    int i;
    for (i = 0; i < 100; i++) {
        tp_hash_part_assign(&hp, i);
    }
    if (hp.total_items != 102) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1723: Hash-based partitioning should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1723: Output should not be empty");
    assert!(code.contains("fn tp_hash_part_init"), "C1723: Should contain tp_hash_part_init");
    assert!(code.contains("fn tp_hash_func"), "C1723: Should contain tp_hash_func");
    assert!(code.contains("fn tp_hash_part_assign"), "C1723: Should contain tp_hash_part_assign");
    Ok(())
}

/// C1724: Adaptive load balancing with moving averages
#[test]
fn c1724_adaptive_load_balancing() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int current_load;
    int avg_load;
    int peak_load;
    int samples;
} tp_load_stats_t;

typedef struct {
    tp_load_stats_t workers[16];
    int count;
    int rebalance_threshold;
} tp_adaptive_lb_t;

void tp_adaptive_init(tp_adaptive_lb_t *lb, int threshold) {
    lb->count = 0;
    lb->rebalance_threshold = threshold;
    int i;
    for (i = 0; i < 16; i++) {
        lb->workers[i].current_load = 0;
        lb->workers[i].avg_load = 0;
        lb->workers[i].peak_load = 0;
        lb->workers[i].samples = 0;
    }
}

int tp_adaptive_add_worker(tp_adaptive_lb_t *lb) {
    if (lb->count >= 16) return -1;
    int idx = lb->count;
    lb->count = lb->count + 1;
    return idx;
}

void tp_adaptive_update_load(tp_adaptive_lb_t *lb, int idx, int load) {
    if (idx < 0 || idx >= lb->count) return;
    lb->workers[idx].current_load = load;
    lb->workers[idx].samples = lb->workers[idx].samples + 1;
    lb->workers[idx].avg_load =
        (lb->workers[idx].avg_load * (lb->workers[idx].samples - 1) + load)
        / lb->workers[idx].samples;
    if (load > lb->workers[idx].peak_load) {
        lb->workers[idx].peak_load = load;
    }
}

int tp_adaptive_select(const tp_adaptive_lb_t *lb) {
    if (lb->count <= 0) return -1;
    int best = 0;
    int i;
    for (i = 1; i < lb->count; i++) {
        if (lb->workers[i].avg_load < lb->workers[best].avg_load) {
            best = i;
        }
    }
    return best;
}

int tp_adaptive_needs_rebalance(const tp_adaptive_lb_t *lb) {
    if (lb->count < 2) return 0;
    int min_load = lb->workers[0].current_load;
    int max_load = lb->workers[0].current_load;
    int i;
    for (i = 1; i < lb->count; i++) {
        if (lb->workers[i].current_load < min_load) min_load = lb->workers[i].current_load;
        if (lb->workers[i].current_load > max_load) max_load = lb->workers[i].current_load;
    }
    return (max_load - min_load) > lb->rebalance_threshold;
}

int tp_adaptive_test(void) {
    tp_adaptive_lb_t lb;
    tp_adaptive_init(&lb, 10);
    int w0 = tp_adaptive_add_worker(&lb);
    int w1 = tp_adaptive_add_worker(&lb);
    tp_adaptive_update_load(&lb, w0, 50);
    tp_adaptive_update_load(&lb, w1, 10);
    if (tp_adaptive_select(&lb) != w1) return -1;
    if (!tp_adaptive_needs_rebalance(&lb)) return -2;
    tp_adaptive_update_load(&lb, w0, 12);
    tp_adaptive_update_load(&lb, w1, 11);
    if (tp_adaptive_needs_rebalance(&lb)) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1724: Adaptive load balancing should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1724: Output should not be empty");
    assert!(code.contains("fn tp_adaptive_init"), "C1724: Should contain tp_adaptive_init");
    assert!(code.contains("fn tp_adaptive_select"), "C1724: Should contain tp_adaptive_select");
    assert!(code.contains("fn tp_adaptive_needs_rebalance"), "C1724: Should contain tp_adaptive_needs_rebalance");
    Ok(())
}

/// C1725: Backpressure controller
#[test]
fn c1725_backpressure_controller() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    int queue_size;
    int max_queue_size;
    int high_watermark;
    int low_watermark;
    int throttled;
    int accepted;
    int rejected;
} tp_backpressure_t;

void tp_bp_init(tp_backpressure_t *bp, int max_size, int high_wm, int low_wm) {
    bp->queue_size = 0;
    bp->max_queue_size = max_size;
    bp->high_watermark = high_wm;
    bp->low_watermark = low_wm;
    bp->throttled = 0;
    bp->accepted = 0;
    bp->rejected = 0;
}

int tp_bp_try_enqueue(tp_backpressure_t *bp) {
    if (bp->queue_size >= bp->max_queue_size) {
        bp->rejected = bp->rejected + 1;
        return -1;
    }
    if (bp->throttled && bp->queue_size >= bp->high_watermark) {
        bp->rejected = bp->rejected + 1;
        return -2;
    }
    bp->queue_size = bp->queue_size + 1;
    bp->accepted = bp->accepted + 1;
    if (bp->queue_size >= bp->high_watermark) {
        bp->throttled = 1;
    }
    return 0;
}

int tp_bp_dequeue(tp_backpressure_t *bp) {
    if (bp->queue_size <= 0) return -1;
    bp->queue_size = bp->queue_size - 1;
    if (bp->queue_size <= bp->low_watermark) {
        bp->throttled = 0;
    }
    return 0;
}

int tp_bp_is_throttled(const tp_backpressure_t *bp) {
    return bp->throttled;
}

int tp_bp_utilization(const tp_backpressure_t *bp) {
    if (bp->max_queue_size == 0) return 0;
    return (bp->queue_size * 100) / bp->max_queue_size;
}

int tp_bp_acceptance_rate(const tp_backpressure_t *bp) {
    int total = bp->accepted + bp->rejected;
    if (total == 0) return 100;
    return (bp->accepted * 100) / total;
}

int tp_bp_test(void) {
    tp_backpressure_t bp;
    tp_bp_init(&bp, 100, 80, 20);
    if (tp_bp_is_throttled(&bp)) return -1;
    int i;
    for (i = 0; i < 80; i++) {
        tp_bp_try_enqueue(&bp);
    }
    if (!tp_bp_is_throttled(&bp)) return -2;
    if (tp_bp_utilization(&bp) != 80) return -3;
    int j;
    for (j = 0; j < 70; j++) {
        tp_bp_dequeue(&bp);
    }
    if (tp_bp_is_throttled(&bp)) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1725: Backpressure controller should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1725: Output should not be empty");
    assert!(code.contains("fn tp_bp_init"), "C1725: Should contain tp_bp_init");
    assert!(code.contains("fn tp_bp_try_enqueue"), "C1725: Should contain tp_bp_try_enqueue");
    assert!(code.contains("fn tp_bp_is_throttled"), "C1725: Should contain tp_bp_is_throttled");
    Ok(())
}
