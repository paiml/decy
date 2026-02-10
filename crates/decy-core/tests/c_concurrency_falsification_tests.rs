//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//! C626-C650: Concurrency and Parallel Programming Patterns
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world concurrency and parallel programming
//! patterns commonly found in OS kernels, lock-free libraries, thread pools,
//! and high-performance concurrent data structures -- all expressed as valid
//! C99 with array-based representations (no malloc/free, no pthreads includes).
//!
//! Organization:
//! - C626-C630: Low-level locking primitives (spinlock, ticket lock, rwlock, lock-free stack/queue)
//! - C631-C635: Synchronization patterns (semaphore, barrier, ring buffer, double-checked locking, thread pool)
//! - C636-C640: Concurrency models (future/promise, actor, map-reduce, pipeline, fork-join)
//! - C641-C645: Memory reclamation (work stealing, hazard pointers, epoch reclamation, seqlock, RCU)
//! - C646-C650: Concurrent containers (memory pool, object pool, concurrent hashmap, skiplist, timer wheel)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C626-C630: Low-Level Locking Primitives
// ============================================================================

/// C626: Spinlock using atomic test-and-set simulation
#[test]
fn c626_spinlock_test_and_set() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int locked;
} spinlock_t;

void spinlock_init(spinlock_t *lock) {
    lock->locked = 0;
}

int spinlock_try_acquire(spinlock_t *lock) {
    int old = lock->locked;
    if (old == 0) {
        lock->locked = 1;
        return 1;
    }
    return 0;
}

void spinlock_acquire(spinlock_t *lock) {
    int spins = 0;
    while (lock->locked != 0) {
        spins++;
        if (spins > 1000000) return;
    }
    lock->locked = 1;
}

void spinlock_release(spinlock_t *lock) {
    lock->locked = 0;
}

int spinlock_is_locked(const spinlock_t *lock) {
    return lock->locked != 0;
}

int spinlock_test(void) {
    spinlock_t lock;
    spinlock_init(&lock);
    if (spinlock_is_locked(&lock)) return -1;
    spinlock_acquire(&lock);
    if (!spinlock_is_locked(&lock)) return -2;
    spinlock_release(&lock);
    if (spinlock_is_locked(&lock)) return -3;
    if (!spinlock_try_acquire(&lock)) return -4;
    if (spinlock_try_acquire(&lock)) return -5;
    spinlock_release(&lock);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C626: Spinlock should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C626: empty output");
    assert!(code.contains("fn spinlock_init"), "C626: Should contain spinlock_init");
    assert!(code.contains("fn spinlock_acquire"), "C626: Should contain spinlock_acquire");
    assert!(code.contains("fn spinlock_release"), "C626: Should contain spinlock_release");
    Ok(())
}

/// C627: Ticket lock with fair ordering
#[test]
fn c627_ticket_lock() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int next_ticket;
    int now_serving;
} ticket_lock_t;

void ticket_init(ticket_lock_t *lock) {
    lock->next_ticket = 0;
    lock->now_serving = 0;
}

int ticket_acquire(ticket_lock_t *lock) {
    int my_ticket = lock->next_ticket;
    lock->next_ticket = my_ticket + 1;
    int spins = 0;
    while (lock->now_serving != my_ticket) {
        spins++;
        if (spins > 1000000) return -1;
    }
    return my_ticket;
}

void ticket_release(ticket_lock_t *lock) {
    lock->now_serving = lock->now_serving + 1;
}

int ticket_is_locked(const ticket_lock_t *lock) {
    return lock->next_ticket != lock->now_serving;
}

int ticket_waiters(const ticket_lock_t *lock) {
    return lock->next_ticket - lock->now_serving;
}

int ticket_test(void) {
    ticket_lock_t lock;
    ticket_init(&lock);
    if (ticket_is_locked(&lock)) return -1;
    int t = ticket_acquire(&lock);
    if (t != 0) return -2;
    if (!ticket_is_locked(&lock)) return -3;
    if (ticket_waiters(&lock) != 1) return -4;
    ticket_release(&lock);
    if (ticket_is_locked(&lock)) return -5;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C627: Ticket lock should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C627: empty output");
    assert!(code.contains("fn ticket_init"), "C627: Should contain ticket_init");
    assert!(code.contains("fn ticket_acquire"), "C627: Should contain ticket_acquire");
    assert!(code.contains("fn ticket_release"), "C627: Should contain ticket_release");
    Ok(())
}

/// C628: Reader-writer lock using counters
#[test]
fn c628_reader_writer_lock() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int readers;
    int writer;
    int write_waiters;
} rwlock_t;

void rwlock_init(rwlock_t *rw) {
    rw->readers = 0;
    rw->writer = 0;
    rw->write_waiters = 0;
}

int rwlock_read_acquire(rwlock_t *rw) {
    if (rw->writer != 0 || rw->write_waiters > 0) {
        return -1;
    }
    rw->readers = rw->readers + 1;
    return 0;
}

void rwlock_read_release(rwlock_t *rw) {
    if (rw->readers > 0) {
        rw->readers = rw->readers - 1;
    }
}

int rwlock_write_acquire(rwlock_t *rw) {
    if (rw->writer != 0 || rw->readers > 0) {
        rw->write_waiters = rw->write_waiters + 1;
        return -1;
    }
    rw->writer = 1;
    return 0;
}

void rwlock_write_release(rwlock_t *rw) {
    rw->writer = 0;
}

int rwlock_is_idle(const rwlock_t *rw) {
    return rw->readers == 0 && rw->writer == 0;
}

int rwlock_test(void) {
    rwlock_t rw;
    rwlock_init(&rw);
    if (!rwlock_is_idle(&rw)) return -1;
    if (rwlock_read_acquire(&rw) != 0) return -2;
    if (rwlock_read_acquire(&rw) != 0) return -3;
    if (rw.readers != 2) return -4;
    rwlock_read_release(&rw);
    rwlock_read_release(&rw);
    if (rwlock_write_acquire(&rw) != 0) return -5;
    rwlock_write_release(&rw);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C628: RW lock should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C628: empty output");
    assert!(code.contains("fn rwlock_init"), "C628: Should contain rwlock_init");
    assert!(code.contains("fn rwlock_read_acquire"), "C628: Should contain rwlock_read_acquire");
    assert!(code.contains("fn rwlock_write_acquire"), "C628: Should contain rwlock_write_acquire");
    Ok(())
}

/// C629: Lock-free stack using CAS simulation
#[test]
fn c629_lock_free_stack_cas() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int data[256];
    int next[256];
    int top;
    int free_head;
    int size;
} lf_stack_t;

void lfs_init(lf_stack_t *s) {
    int i;
    s->top = -1;
    s->free_head = 0;
    s->size = 0;
    for (i = 0; i < 255; i = i + 1) {
        s->next[i] = i + 1;
    }
    s->next[255] = -1;
}

int lfs_cas(int *target, int expected, int desired) {
    if (*target == expected) {
        *target = desired;
        return 1;
    }
    return 0;
}

int lfs_push(lf_stack_t *s, int value) {
    if (s->free_head == -1) return -1;
    int node = s->free_head;
    s->free_head = s->next[node];
    s->data[node] = value;
    s->next[node] = s->top;
    s->top = node;
    s->size = s->size + 1;
    return 0;
}

int lfs_pop(lf_stack_t *s, int *out) {
    if (s->top == -1) return -1;
    int node = s->top;
    *out = s->data[node];
    s->top = s->next[node];
    s->next[node] = s->free_head;
    s->free_head = node;
    s->size = s->size - 1;
    return 0;
}

int lfs_peek(const lf_stack_t *s) {
    if (s->top == -1) return -1;
    return s->data[s->top];
}

int lfs_is_empty(const lf_stack_t *s) {
    return s->top == -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C629: Lock-free stack should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C629: empty output");
    assert!(code.contains("fn lfs_init"), "C629: Should contain lfs_init");
    assert!(code.contains("fn lfs_push"), "C629: Should contain lfs_push");
    assert!(code.contains("fn lfs_pop"), "C629: Should contain lfs_pop");
    Ok(())
}

/// C630: Lock-free queue (Michael-Scott style, simulated with arrays)
#[test]
fn c630_lock_free_queue_michael_scott() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int data[256];
    int next[256];
    int head;
    int tail;
    int free_head;
    int count;
} lf_queue_t;

void lfq_init(lf_queue_t *q) {
    int i;
    for (i = 0; i < 255; i = i + 1) {
        q->next[i] = i + 1;
    }
    q->next[255] = -1;
    q->free_head = 1;
    q->data[0] = 0;
    q->next[0] = -1;
    q->head = 0;
    q->tail = 0;
    q->count = 0;
}

int lfq_alloc_node(lf_queue_t *q) {
    if (q->free_head == -1) return -1;
    int n = q->free_head;
    q->free_head = q->next[n];
    return n;
}

void lfq_free_node(lf_queue_t *q, int n) {
    q->next[n] = q->free_head;
    q->free_head = n;
}

int lfq_enqueue(lf_queue_t *q, int value) {
    int node = lfq_alloc_node(q);
    if (node == -1) return -1;
    q->data[node] = value;
    q->next[node] = -1;
    q->next[q->tail] = node;
    q->tail = node;
    q->count = q->count + 1;
    return 0;
}

int lfq_dequeue(lf_queue_t *q, int *out) {
    if (q->next[q->head] == -1) return -1;
    int first = q->next[q->head];
    *out = q->data[first];
    int old_head = q->head;
    q->head = first;
    lfq_free_node(q, old_head);
    q->count = q->count - 1;
    return 0;
}

int lfq_is_empty(const lf_queue_t *q) {
    return q->next[q->head] == -1;
}

int lfq_size(const lf_queue_t *q) {
    return q->count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C630: Lock-free queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C630: empty output");
    assert!(code.contains("fn lfq_init"), "C630: Should contain lfq_init");
    assert!(code.contains("fn lfq_enqueue"), "C630: Should contain lfq_enqueue");
    assert!(code.contains("fn lfq_dequeue"), "C630: Should contain lfq_dequeue");
    Ok(())
}

// ============================================================================
// C631-C635: Synchronization Patterns
// ============================================================================

/// C631: Counting semaphore via struct
#[test]
fn c631_counting_semaphore() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int value;
    int max_value;
    int waiters;
} semaphore_t;

void sem_init_val(semaphore_t *s, int initial, int max_val) {
    s->value = initial;
    s->max_value = max_val;
    s->waiters = 0;
}

int sem_try_wait(semaphore_t *s) {
    if (s->value > 0) {
        s->value = s->value - 1;
        return 0;
    }
    s->waiters = s->waiters + 1;
    return -1;
}

int sem_post(semaphore_t *s) {
    if (s->value >= s->max_value) {
        return -1;
    }
    s->value = s->value + 1;
    if (s->waiters > 0) {
        s->waiters = s->waiters - 1;
    }
    return 0;
}

int sem_get_value(const semaphore_t *s) {
    return s->value;
}

int sem_test(void) {
    semaphore_t s;
    sem_init_val(&s, 3, 5);
    if (sem_get_value(&s) != 3) return -1;
    if (sem_try_wait(&s) != 0) return -2;
    if (sem_try_wait(&s) != 0) return -3;
    if (sem_try_wait(&s) != 0) return -4;
    if (sem_try_wait(&s) != -1) return -5;
    if (sem_post(&s) != 0) return -6;
    if (sem_get_value(&s) != 1) return -7;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C631: Semaphore should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C631: empty output");
    assert!(code.contains("fn sem_init_val"), "C631: Should contain sem_init_val");
    assert!(code.contains("fn sem_try_wait"), "C631: Should contain sem_try_wait");
    assert!(code.contains("fn sem_post"), "C631: Should contain sem_post");
    Ok(())
}

/// C632: Barrier synchronization point
#[test]
fn c632_barrier_sync_point() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int count;
    int total;
    int generation;
    int waiting;
} barrier_t;

void barrier_init(barrier_t *b, int num_threads) {
    b->count = 0;
    b->total = num_threads;
    b->generation = 0;
    b->waiting = 0;
}

int barrier_arrive(barrier_t *b) {
    int gen = b->generation;
    b->count = b->count + 1;
    b->waiting = b->waiting + 1;
    if (b->count >= b->total) {
        b->count = 0;
        b->generation = gen + 1;
        b->waiting = 0;
        return 1;
    }
    return 0;
}

int barrier_is_complete(const barrier_t *b) {
    return b->count == 0 && b->generation > 0;
}

int barrier_generation(const barrier_t *b) {
    return b->generation;
}

int barrier_waiting_count(const barrier_t *b) {
    return b->waiting;
}

int barrier_test(void) {
    barrier_t b;
    barrier_init(&b, 3);
    if (barrier_arrive(&b) != 0) return -1;
    if (barrier_arrive(&b) != 0) return -2;
    if (barrier_arrive(&b) != 1) return -3;
    if (barrier_generation(&b) != 1) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C632: Barrier should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C632: empty output");
    assert!(code.contains("fn barrier_init"), "C632: Should contain barrier_init");
    assert!(code.contains("fn barrier_arrive"), "C632: Should contain barrier_arrive");
    Ok(())
}

/// C633: Single-producer single-consumer lock-free ring buffer
#[test]
fn c633_spsc_ring_buffer() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int buffer[64];
    int read_pos;
    int write_pos;
    int capacity;
} spsc_ring_t;

void spsc_init(spsc_ring_t *r) {
    int i;
    for (i = 0; i < 64; i = i + 1) {
        r->buffer[i] = 0;
    }
    r->read_pos = 0;
    r->write_pos = 0;
    r->capacity = 64;
}

int spsc_is_full(const spsc_ring_t *r) {
    int next_write = (r->write_pos + 1) % r->capacity;
    return next_write == r->read_pos;
}

int spsc_is_empty(const spsc_ring_t *r) {
    return r->read_pos == r->write_pos;
}

int spsc_size(const spsc_ring_t *r) {
    int diff = r->write_pos - r->read_pos;
    if (diff < 0) diff = diff + r->capacity;
    return diff;
}

int spsc_write(spsc_ring_t *r, int value) {
    if (spsc_is_full(r)) return -1;
    r->buffer[r->write_pos] = value;
    r->write_pos = (r->write_pos + 1) % r->capacity;
    return 0;
}

int spsc_read(spsc_ring_t *r, int *out) {
    if (spsc_is_empty(r)) return -1;
    *out = r->buffer[r->read_pos];
    r->read_pos = (r->read_pos + 1) % r->capacity;
    return 0;
}

int spsc_test(void) {
    spsc_ring_t ring;
    spsc_init(&ring);
    if (!spsc_is_empty(&ring)) return -1;
    spsc_write(&ring, 42);
    spsc_write(&ring, 99);
    if (spsc_size(&ring) != 2) return -2;
    int val = 0;
    spsc_read(&ring, &val);
    if (val != 42) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C633: SPSC ring buffer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C633: empty output");
    assert!(code.contains("fn spsc_init"), "C633: Should contain spsc_init");
    assert!(code.contains("fn spsc_write"), "C633: Should contain spsc_write");
    assert!(code.contains("fn spsc_read"), "C633: Should contain spsc_read");
    Ok(())
}

/// C634: Double-checked locking pattern
#[test]
fn c634_double_checked_locking() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int initialized;
    int value;
    int lock;
} lazy_singleton_t;

void singleton_reset(lazy_singleton_t *s) {
    s->initialized = 0;
    s->value = 0;
    s->lock = 0;
}

int singleton_get(lazy_singleton_t *s) {
    if (s->initialized) {
        return s->value;
    }
    s->lock = 1;
    if (!s->initialized) {
        s->value = 42;
        s->initialized = 1;
    }
    s->lock = 0;
    return s->value;
}

int singleton_is_initialized(const lazy_singleton_t *s) {
    return s->initialized;
}

int singleton_test(void) {
    lazy_singleton_t s;
    singleton_reset(&s);
    if (singleton_is_initialized(&s)) return -1;
    int v1 = singleton_get(&s);
    if (v1 != 42) return -2;
    if (!singleton_is_initialized(&s)) return -3;
    int v2 = singleton_get(&s);
    if (v2 != 42) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C634: Double-checked locking should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C634: empty output");
    assert!(code.contains("fn singleton_reset"), "C634: Should contain singleton_reset");
    assert!(code.contains("fn singleton_get"), "C634: Should contain singleton_get");
    Ok(())
}

/// C635: Thread pool with task queue
#[test]
fn c635_thread_pool_task_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int task_id;
    int priority;
    int status;
} task_t;

typedef struct {
    task_t tasks[128];
    int head;
    int tail;
    int count;
    int active_workers;
    int max_workers;
} thread_pool_t;

void pool_init(thread_pool_t *p, int workers) {
    p->head = 0;
    p->tail = 0;
    p->count = 0;
    p->active_workers = 0;
    p->max_workers = workers;
}

int pool_submit(thread_pool_t *p, int task_id, int priority) {
    if (p->count >= 128) return -1;
    p->tasks[p->tail].task_id = task_id;
    p->tasks[p->tail].priority = priority;
    p->tasks[p->tail].status = 0;
    p->tail = (p->tail + 1) % 128;
    p->count = p->count + 1;
    return 0;
}

int pool_fetch_task(thread_pool_t *p, task_t *out) {
    if (p->count <= 0) return -1;
    out->task_id = p->tasks[p->head].task_id;
    out->priority = p->tasks[p->head].priority;
    out->status = 1;
    p->head = (p->head + 1) % 128;
    p->count = p->count - 1;
    p->active_workers = p->active_workers + 1;
    return 0;
}

void pool_complete_task(thread_pool_t *p) {
    if (p->active_workers > 0) {
        p->active_workers = p->active_workers - 1;
    }
}

int pool_pending(const thread_pool_t *p) {
    return p->count;
}

int pool_active(const thread_pool_t *p) {
    return p->active_workers;
}

int pool_test(void) {
    thread_pool_t pool;
    pool_init(&pool, 4);
    pool_submit(&pool, 1, 10);
    pool_submit(&pool, 2, 5);
    if (pool_pending(&pool) != 2) return -1;
    task_t t;
    pool_fetch_task(&pool, &t);
    if (t.task_id != 1) return -2;
    if (pool_active(&pool) != 1) return -3;
    pool_complete_task(&pool);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C635: Thread pool should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C635: empty output");
    assert!(code.contains("fn pool_init"), "C635: Should contain pool_init");
    assert!(code.contains("fn pool_submit"), "C635: Should contain pool_submit");
    assert!(code.contains("fn pool_fetch_task"), "C635: Should contain pool_fetch_task");
    Ok(())
}

// ============================================================================
// C636-C640: Concurrency Models
// ============================================================================

/// C636: Future/Promise state machine
#[test]
fn c636_future_promise_state_machine() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int state;
    int value;
    int error_code;
    int has_callbacks;
} future_t;

void future_init(future_t *f) {
    f->state = 0;
    f->value = 0;
    f->error_code = 0;
    f->has_callbacks = 0;
}

int future_is_pending(const future_t *f) {
    return f->state == 0;
}

int future_is_resolved(const future_t *f) {
    return f->state == 1;
}

int future_is_rejected(const future_t *f) {
    return f->state == 2;
}

void future_resolve(future_t *f, int value) {
    if (f->state != 0) return;
    f->state = 1;
    f->value = value;
}

void future_reject(future_t *f, int error) {
    if (f->state != 0) return;
    f->state = 2;
    f->error_code = error;
}

int future_get(const future_t *f, int *out) {
    if (f->state != 1) return -1;
    *out = f->value;
    return 0;
}

int future_get_error(const future_t *f) {
    if (f->state != 2) return 0;
    return f->error_code;
}

int future_chain(future_t *src, future_t *dst, int offset) {
    if (src->state != 1) return -1;
    future_resolve(dst, src->value + offset);
    return 0;
}

int future_test(void) {
    future_t f;
    future_init(&f);
    if (!future_is_pending(&f)) return -1;
    future_resolve(&f, 100);
    if (!future_is_resolved(&f)) return -2;
    int val = 0;
    future_get(&f, &val);
    if (val != 100) return -3;
    future_t f2;
    future_init(&f2);
    future_reject(&f2, 404);
    if (!future_is_rejected(&f2)) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C636: Future/Promise should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C636: empty output");
    assert!(code.contains("fn future_init"), "C636: Should contain future_init");
    assert!(code.contains("fn future_resolve"), "C636: Should contain future_resolve");
    assert!(code.contains("fn future_reject"), "C636: Should contain future_reject");
    Ok(())
}

/// C637: Actor model with message queue
#[test]
fn c637_actor_message_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int msg_type;
    int sender_id;
    int payload;
} message_t;

typedef struct {
    int id;
    int state;
    message_t mailbox[64];
    int mail_head;
    int mail_tail;
    int mail_count;
    int processed;
} actor_t;

void actor_init(actor_t *a, int id) {
    a->id = id;
    a->state = 0;
    a->mail_head = 0;
    a->mail_tail = 0;
    a->mail_count = 0;
    a->processed = 0;
}

int actor_send(actor_t *a, int msg_type, int sender_id, int payload) {
    if (a->mail_count >= 64) return -1;
    a->mailbox[a->mail_tail].msg_type = msg_type;
    a->mailbox[a->mail_tail].sender_id = sender_id;
    a->mailbox[a->mail_tail].payload = payload;
    a->mail_tail = (a->mail_tail + 1) % 64;
    a->mail_count = a->mail_count + 1;
    return 0;
}

int actor_receive(actor_t *a, message_t *out) {
    if (a->mail_count <= 0) return -1;
    out->msg_type = a->mailbox[a->mail_head].msg_type;
    out->sender_id = a->mailbox[a->mail_head].sender_id;
    out->payload = a->mailbox[a->mail_head].payload;
    a->mail_head = (a->mail_head + 1) % 64;
    a->mail_count = a->mail_count - 1;
    a->processed = a->processed + 1;
    return 0;
}

int actor_has_messages(const actor_t *a) {
    return a->mail_count > 0;
}

int actor_pending_count(const actor_t *a) {
    return a->mail_count;
}

int actor_test(void) {
    actor_t a;
    actor_init(&a, 1);
    actor_send(&a, 10, 2, 999);
    actor_send(&a, 20, 3, 888);
    if (actor_pending_count(&a) != 2) return -1;
    message_t msg;
    actor_receive(&a, &msg);
    if (msg.msg_type != 10) return -2;
    if (msg.payload != 999) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C637: Actor model should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C637: empty output");
    assert!(code.contains("fn actor_init"), "C637: Should contain actor_init");
    assert!(code.contains("fn actor_send"), "C637: Should contain actor_send");
    assert!(code.contains("fn actor_receive"), "C637: Should contain actor_receive");
    Ok(())
}

/// C638: Map-reduce pattern with partition and combine
#[test]
fn c638_map_reduce_partition_combine() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int data[256];
    int len;
} partition_t;

typedef struct {
    partition_t partitions[4];
    int num_partitions;
    int results[4];
} map_reduce_t;

void mr_init(map_reduce_t *mr, const int *data, int len) {
    int chunk_size = len / 4;
    int i;
    int j;
    mr->num_partitions = 4;
    for (i = 0; i < 4; i = i + 1) {
        int start = i * chunk_size;
        int end = start + chunk_size;
        if (i == 3 && end < len) end = len;
        mr->partitions[i].len = 0;
        for (j = start; j < end && j < len; j = j + 1) {
            mr->partitions[i].data[mr->partitions[i].len] = data[j];
            mr->partitions[i].len = mr->partitions[i].len + 1;
        }
        mr->results[i] = 0;
    }
}

int mr_map_sum(const partition_t *p) {
    int sum = 0;
    int i;
    for (i = 0; i < p->len; i = i + 1) {
        sum = sum + p->data[i];
    }
    return sum;
}

int mr_map_max(const partition_t *p) {
    if (p->len == 0) return 0;
    int mx = p->data[0];
    int i;
    for (i = 1; i < p->len; i = i + 1) {
        if (p->data[i] > mx) mx = p->data[i];
    }
    return mx;
}

void mr_execute_sum(map_reduce_t *mr) {
    int i;
    for (i = 0; i < mr->num_partitions; i = i + 1) {
        mr->results[i] = mr_map_sum(&mr->partitions[i]);
    }
}

int mr_reduce_sum(const map_reduce_t *mr) {
    int total = 0;
    int i;
    for (i = 0; i < mr->num_partitions; i = i + 1) {
        total = total + mr->results[i];
    }
    return total;
}

int mr_reduce_max(const int *results, int n) {
    int mx = results[0];
    int i;
    for (i = 1; i < n; i = i + 1) {
        if (results[i] > mx) mx = results[i];
    }
    return mx;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C638: Map-reduce should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C638: empty output");
    assert!(code.contains("fn mr_init"), "C638: Should contain mr_init");
    assert!(code.contains("fn mr_map_sum"), "C638: Should contain mr_map_sum");
    assert!(code.contains("fn mr_reduce_sum"), "C638: Should contain mr_reduce_sum");
    Ok(())
}

/// C639: Pipeline pattern with staged processing
#[test]
fn c639_pipeline_staged_processing() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int buffer[64];
    int head;
    int tail;
    int count;
} stage_queue_t;

typedef struct {
    stage_queue_t stages[4];
    int num_stages;
    int items_processed;
} pipeline_t;

void stage_init(stage_queue_t *sq) {
    sq->head = 0;
    sq->tail = 0;
    sq->count = 0;
}

int stage_put(stage_queue_t *sq, int val) {
    if (sq->count >= 64) return -1;
    sq->buffer[sq->tail] = val;
    sq->tail = (sq->tail + 1) % 64;
    sq->count = sq->count + 1;
    return 0;
}

int stage_get(stage_queue_t *sq, int *out) {
    if (sq->count <= 0) return -1;
    *out = sq->buffer[sq->head];
    sq->head = (sq->head + 1) % 64;
    sq->count = sq->count - 1;
    return 0;
}

void pipeline_init(pipeline_t *p, int num_stages) {
    int i;
    p->num_stages = num_stages;
    p->items_processed = 0;
    for (i = 0; i < num_stages && i < 4; i = i + 1) {
        stage_init(&p->stages[i]);
    }
}

int pipeline_feed(pipeline_t *p, int value) {
    return stage_put(&p->stages[0], value);
}

int pipeline_advance(pipeline_t *p, int stage_idx) {
    if (stage_idx < 0 || stage_idx >= p->num_stages - 1) return -1;
    int val = 0;
    int rc = stage_get(&p->stages[stage_idx], &val);
    if (rc != 0) return -1;
    val = val + stage_idx + 1;
    stage_put(&p->stages[stage_idx + 1], val);
    return 0;
}

int pipeline_drain(pipeline_t *p, int *out) {
    int last = p->num_stages - 1;
    int rc = stage_get(&p->stages[last], out);
    if (rc == 0) {
        p->items_processed = p->items_processed + 1;
    }
    return rc;
}

int pipeline_total_processed(const pipeline_t *p) {
    return p->items_processed;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C639: Pipeline should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C639: empty output");
    assert!(code.contains("fn pipeline_init"), "C639: Should contain pipeline_init");
    assert!(code.contains("fn pipeline_feed"), "C639: Should contain pipeline_feed");
    assert!(code.contains("fn pipeline_advance"), "C639: Should contain pipeline_advance");
    Ok(())
}

/// C640: Fork-join recursive task splitting
#[test]
fn c640_fork_join_task_splitting() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int start;
    int end;
    int result;
    int completed;
} fj_task_t;

typedef struct {
    fj_task_t tasks[64];
    int count;
    int threshold;
} fj_pool_t;

void fj_init(fj_pool_t *pool, int threshold) {
    pool->count = 0;
    pool->threshold = threshold;
}

int fj_create_task(fj_pool_t *pool, int start, int end) {
    if (pool->count >= 64) return -1;
    int idx = pool->count;
    pool->tasks[idx].start = start;
    pool->tasks[idx].end = end;
    pool->tasks[idx].result = 0;
    pool->tasks[idx].completed = 0;
    pool->count = pool->count + 1;
    return idx;
}

int fj_compute_leaf(const int *data, int start, int end) {
    int sum = 0;
    int i;
    for (i = start; i < end; i = i + 1) {
        sum = sum + data[i];
    }
    return sum;
}

void fj_execute(fj_pool_t *pool, const int *data) {
    int i;
    for (i = 0; i < pool->count; i = i + 1) {
        fj_task_t *t = &pool->tasks[i];
        int size = t->end - t->start;
        if (size <= pool->threshold) {
            t->result = fj_compute_leaf(data, t->start, t->end);
            t->completed = 1;
        }
    }
}

int fj_join(const fj_pool_t *pool) {
    int total = 0;
    int i;
    for (i = 0; i < pool->count; i = i + 1) {
        if (pool->tasks[i].completed) {
            total = total + pool->tasks[i].result;
        }
    }
    return total;
}

int fj_all_completed(const fj_pool_t *pool) {
    int i;
    for (i = 0; i < pool->count; i = i + 1) {
        if (!pool->tasks[i].completed) return 0;
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C640: Fork-join should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C640: empty output");
    assert!(code.contains("fn fj_init"), "C640: Should contain fj_init");
    assert!(code.contains("fn fj_create_task"), "C640: Should contain fj_create_task");
    assert!(code.contains("fn fj_execute"), "C640: Should contain fj_execute");
    assert!(code.contains("fn fj_join"), "C640: Should contain fj_join");
    Ok(())
}

// ============================================================================
// C641-C645: Memory Reclamation and Advanced Locking
// ============================================================================

/// C641: Work stealing deque
#[test]
fn c641_work_stealing_deque() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int tasks[128];
    int top;
    int bottom;
} ws_deque_t;

void ws_init(ws_deque_t *d) {
    d->top = 0;
    d->bottom = 0;
}

int ws_push_bottom(ws_deque_t *d, int task) {
    if (d->bottom >= 128) return -1;
    d->tasks[d->bottom] = task;
    d->bottom = d->bottom + 1;
    return 0;
}

int ws_pop_bottom(ws_deque_t *d, int *out) {
    if (d->bottom <= d->top) return -1;
    d->bottom = d->bottom - 1;
    *out = d->tasks[d->bottom];
    if (d->top > d->bottom) {
        d->bottom = d->bottom + 1;
        return -1;
    }
    return 0;
}

int ws_steal_top(ws_deque_t *d, int *out) {
    if (d->top >= d->bottom) return -1;
    *out = d->tasks[d->top];
    d->top = d->top + 1;
    return 0;
}

int ws_size(const ws_deque_t *d) {
    int s = d->bottom - d->top;
    if (s < 0) return 0;
    return s;
}

int ws_is_empty(const ws_deque_t *d) {
    return d->bottom <= d->top;
}

int ws_test(void) {
    ws_deque_t d;
    ws_init(&d);
    ws_push_bottom(&d, 10);
    ws_push_bottom(&d, 20);
    ws_push_bottom(&d, 30);
    if (ws_size(&d) != 3) return -1;
    int val = 0;
    ws_steal_top(&d, &val);
    if (val != 10) return -2;
    ws_pop_bottom(&d, &val);
    if (val != 30) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C641: Work stealing deque should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C641: empty output");
    assert!(code.contains("fn ws_init"), "C641: Should contain ws_init");
    assert!(code.contains("fn ws_push_bottom"), "C641: Should contain ws_push_bottom");
    assert!(code.contains("fn ws_steal_top"), "C641: Should contain ws_steal_top");
    Ok(())
}

/// C642: Hazard pointer tracking
#[test]
fn c642_hazard_pointer_tracking() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int hazard_ptrs[16];
    int active[16];
    int num_threads;
    int retired[64];
    int retired_count;
} hp_tracker_t;

void hp_init(hp_tracker_t *hp, int num_threads) {
    int i;
    hp->num_threads = num_threads;
    hp->retired_count = 0;
    for (i = 0; i < 16; i = i + 1) {
        hp->hazard_ptrs[i] = -1;
        hp->active[i] = 0;
    }
    for (i = 0; i < 64; i = i + 1) {
        hp->retired[i] = -1;
    }
}

void hp_protect(hp_tracker_t *hp, int thread_id, int ptr_val) {
    if (thread_id >= 0 && thread_id < 16) {
        hp->hazard_ptrs[thread_id] = ptr_val;
        hp->active[thread_id] = 1;
    }
}

void hp_clear(hp_tracker_t *hp, int thread_id) {
    if (thread_id >= 0 && thread_id < 16) {
        hp->hazard_ptrs[thread_id] = -1;
        hp->active[thread_id] = 0;
    }
}

int hp_is_protected(const hp_tracker_t *hp, int ptr_val) {
    int i;
    for (i = 0; i < hp->num_threads; i = i + 1) {
        if (hp->active[i] && hp->hazard_ptrs[i] == ptr_val) {
            return 1;
        }
    }
    return 0;
}

int hp_retire(hp_tracker_t *hp, int ptr_val) {
    if (hp->retired_count >= 64) return -1;
    hp->retired[hp->retired_count] = ptr_val;
    hp->retired_count = hp->retired_count + 1;
    return 0;
}

int hp_scan_reclaimable(const hp_tracker_t *hp) {
    int reclaimable = 0;
    int i;
    for (i = 0; i < hp->retired_count; i = i + 1) {
        if (hp->retired[i] != -1 && !hp_is_protected(hp, hp->retired[i])) {
            reclaimable = reclaimable + 1;
        }
    }
    return reclaimable;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C642: Hazard pointer tracking should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C642: empty output");
    assert!(code.contains("fn hp_init"), "C642: Should contain hp_init");
    assert!(code.contains("fn hp_protect"), "C642: Should contain hp_protect");
    assert!(code.contains("fn hp_is_protected"), "C642: Should contain hp_is_protected");
    Ok(())
}

/// C643: Epoch-based reclamation
#[test]
fn c643_epoch_based_reclamation() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int global_epoch;
    int thread_epochs[8];
    int thread_active[8];
    int num_threads;
    int garbage[3][32];
    int garbage_count[3];
} epoch_tracker_t;

void epoch_init(epoch_tracker_t *et, int num_threads) {
    int i;
    int j;
    et->global_epoch = 0;
    et->num_threads = num_threads;
    for (i = 0; i < 8; i = i + 1) {
        et->thread_epochs[i] = 0;
        et->thread_active[i] = 0;
    }
    for (i = 0; i < 3; i = i + 1) {
        et->garbage_count[i] = 0;
        for (j = 0; j < 32; j = j + 1) {
            et->garbage[i][j] = -1;
        }
    }
}

void epoch_enter(epoch_tracker_t *et, int thread_id) {
    if (thread_id >= 0 && thread_id < 8) {
        et->thread_epochs[thread_id] = et->global_epoch;
        et->thread_active[thread_id] = 1;
    }
}

void epoch_exit(epoch_tracker_t *et, int thread_id) {
    if (thread_id >= 0 && thread_id < 8) {
        et->thread_active[thread_id] = 0;
    }
}

int epoch_can_advance(const epoch_tracker_t *et) {
    int i;
    for (i = 0; i < et->num_threads; i = i + 1) {
        if (et->thread_active[i] && et->thread_epochs[i] != et->global_epoch) {
            return 0;
        }
    }
    return 1;
}

void epoch_try_advance(epoch_tracker_t *et) {
    if (epoch_can_advance(et)) {
        et->global_epoch = (et->global_epoch + 1) % 3;
        et->garbage_count[et->global_epoch] = 0;
    }
}

int epoch_retire(epoch_tracker_t *et, int ptr_val) {
    int e = et->global_epoch;
    if (et->garbage_count[e] >= 32) return -1;
    et->garbage[e][et->garbage_count[e]] = ptr_val;
    et->garbage_count[e] = et->garbage_count[e] + 1;
    return 0;
}

int epoch_current(const epoch_tracker_t *et) {
    return et->global_epoch;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C643: Epoch-based reclamation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C643: empty output");
    assert!(code.contains("fn epoch_init"), "C643: Should contain epoch_init");
    assert!(code.contains("fn epoch_enter"), "C643: Should contain epoch_enter");
    assert!(code.contains("fn epoch_try_advance"), "C643: Should contain epoch_try_advance");
    Ok(())
}

/// C644: Sequence lock (seqlock)
#[test]
fn c644_sequence_lock() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int sequence;
    int data_a;
    int data_b;
} seqlock_t;

void seqlock_init(seqlock_t *sl) {
    sl->sequence = 0;
    sl->data_a = 0;
    sl->data_b = 0;
}

void seqlock_write_begin(seqlock_t *sl) {
    sl->sequence = sl->sequence + 1;
}

void seqlock_write_end(seqlock_t *sl) {
    sl->sequence = sl->sequence + 1;
}

void seqlock_write(seqlock_t *sl, int a, int b) {
    seqlock_write_begin(sl);
    sl->data_a = a;
    sl->data_b = b;
    seqlock_write_end(sl);
}

int seqlock_read_begin(const seqlock_t *sl) {
    return sl->sequence;
}

int seqlock_read_retry(const seqlock_t *sl, int start_seq) {
    if ((start_seq & 1) != 0) return 1;
    if (sl->sequence != start_seq) return 1;
    return 0;
}

int seqlock_read(const seqlock_t *sl, int *a, int *b) {
    int attempts = 0;
    while (attempts < 100) {
        int seq = seqlock_read_begin(sl);
        if ((seq & 1) != 0) {
            attempts = attempts + 1;
            continue;
        }
        *a = sl->data_a;
        *b = sl->data_b;
        if (!seqlock_read_retry(sl, seq)) {
            return 0;
        }
        attempts = attempts + 1;
    }
    return -1;
}

int seqlock_test(void) {
    seqlock_t sl;
    seqlock_init(&sl);
    seqlock_write(&sl, 10, 20);
    int a = 0;
    int b = 0;
    if (seqlock_read(&sl, &a, &b) != 0) return -1;
    if (a != 10 || b != 20) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C644: Sequence lock should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C644: empty output");
    assert!(code.contains("fn seqlock_init"), "C644: Should contain seqlock_init");
    assert!(code.contains("fn seqlock_write"), "C644: Should contain seqlock_write");
    assert!(code.contains("fn seqlock_read"), "C644: Should contain seqlock_read");
    Ok(())
}

/// C645: Read-copy-update (RCU) simulation
#[test]
fn c645_read_copy_update_rcu() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int key;
    int value;
    int version;
    int deleted;
} rcu_entry_t;

typedef struct {
    rcu_entry_t entries[32];
    int count;
    int current_version;
    int reader_count;
    int grace_period;
} rcu_table_t;

void rcu_init(rcu_table_t *t) {
    int i;
    t->count = 0;
    t->current_version = 0;
    t->reader_count = 0;
    t->grace_period = 0;
    for (i = 0; i < 32; i = i + 1) {
        t->entries[i].key = -1;
        t->entries[i].value = 0;
        t->entries[i].version = 0;
        t->entries[i].deleted = 0;
    }
}

void rcu_read_lock(rcu_table_t *t) {
    t->reader_count = t->reader_count + 1;
}

void rcu_read_unlock(rcu_table_t *t) {
    if (t->reader_count > 0) {
        t->reader_count = t->reader_count - 1;
    }
}

int rcu_lookup(const rcu_table_t *t, int key) {
    int i;
    for (i = 0; i < t->count; i = i + 1) {
        if (t->entries[i].key == key && !t->entries[i].deleted) {
            return t->entries[i].value;
        }
    }
    return -1;
}

int rcu_update(rcu_table_t *t, int key, int new_value) {
    int i;
    for (i = 0; i < t->count; i = i + 1) {
        if (t->entries[i].key == key && !t->entries[i].deleted) {
            t->entries[i].deleted = 1;
            if (t->count >= 32) return -1;
            int idx = t->count;
            t->entries[idx].key = key;
            t->entries[idx].value = new_value;
            t->entries[idx].version = t->current_version + 1;
            t->entries[idx].deleted = 0;
            t->count = t->count + 1;
            t->current_version = t->current_version + 1;
            return 0;
        }
    }
    return -1;
}

int rcu_insert(rcu_table_t *t, int key, int value) {
    if (t->count >= 32) return -1;
    int idx = t->count;
    t->entries[idx].key = key;
    t->entries[idx].value = value;
    t->entries[idx].version = t->current_version;
    t->entries[idx].deleted = 0;
    t->count = t->count + 1;
    return 0;
}

int rcu_synchronize(rcu_table_t *t) {
    if (t->reader_count == 0) {
        t->grace_period = t->grace_period + 1;
        return 1;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C645: RCU should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C645: empty output");
    assert!(code.contains("fn rcu_init"), "C645: Should contain rcu_init");
    assert!(code.contains("fn rcu_lookup"), "C645: Should contain rcu_lookup");
    assert!(code.contains("fn rcu_update"), "C645: Should contain rcu_update");
    Ok(())
}

// ============================================================================
// C646-C650: Concurrent Containers
// ============================================================================

/// C646: Memory pool with fixed-size block allocator
#[test]
fn c646_memory_pool_block_allocator() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int blocks[256];
    int free_list[256];
    int free_count;
    int block_size;
    int total_blocks;
    int allocated;
} mem_pool_t;

void mempool_init(mem_pool_t *p, int block_size) {
    int i;
    p->block_size = block_size;
    p->total_blocks = 256;
    p->allocated = 0;
    p->free_count = 256;
    for (i = 0; i < 256; i = i + 1) {
        p->free_list[i] = i;
        p->blocks[i] = 0;
    }
}

int mempool_alloc(mem_pool_t *p) {
    if (p->free_count <= 0) return -1;
    p->free_count = p->free_count - 1;
    int idx = p->free_list[p->free_count];
    p->allocated = p->allocated + 1;
    return idx;
}

int mempool_free(mem_pool_t *p, int idx) {
    if (idx < 0 || idx >= p->total_blocks) return -1;
    if (p->free_count >= p->total_blocks) return -2;
    p->free_list[p->free_count] = idx;
    p->free_count = p->free_count + 1;
    p->blocks[idx] = 0;
    p->allocated = p->allocated - 1;
    return 0;
}

int mempool_available(const mem_pool_t *p) {
    return p->free_count;
}

int mempool_used(const mem_pool_t *p) {
    return p->allocated;
}

int mempool_is_full(const mem_pool_t *p) {
    return p->free_count == 0;
}

int mempool_test(void) {
    mem_pool_t pool;
    mempool_init(&pool, 64);
    if (mempool_available(&pool) != 256) return -1;
    int b1 = mempool_alloc(&pool);
    int b2 = mempool_alloc(&pool);
    if (b1 < 0 || b2 < 0) return -2;
    if (mempool_used(&pool) != 2) return -3;
    mempool_free(&pool, b1);
    if (mempool_used(&pool) != 1) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C646: Memory pool should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C646: empty output");
    assert!(code.contains("fn mempool_init"), "C646: Should contain mempool_init");
    assert!(code.contains("fn mempool_alloc"), "C646: Should contain mempool_alloc");
    assert!(code.contains("fn mempool_free"), "C646: Should contain mempool_free");
    Ok(())
}

/// C647: Object pool with reuse pattern
#[test]
fn c647_object_pool_reuse() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int id;
    int type_tag;
    int ref_count;
    int data[4];
} pooled_obj_t;

typedef struct {
    pooled_obj_t objects[64];
    int in_use[64];
    int count;
    int next_id;
    int reuse_count;
} obj_pool_t;

void objpool_init(obj_pool_t *p) {
    int i;
    p->count = 64;
    p->next_id = 1;
    p->reuse_count = 0;
    for (i = 0; i < 64; i = i + 1) {
        p->in_use[i] = 0;
        p->objects[i].id = 0;
        p->objects[i].ref_count = 0;
    }
}

int objpool_acquire(obj_pool_t *p, int type_tag) {
    int i;
    for (i = 0; i < p->count; i = i + 1) {
        if (!p->in_use[i]) {
            p->in_use[i] = 1;
            p->objects[i].id = p->next_id;
            p->next_id = p->next_id + 1;
            p->objects[i].type_tag = type_tag;
            p->objects[i].ref_count = 1;
            if (p->objects[i].id > 1) {
                p->reuse_count = p->reuse_count + 1;
            }
            return i;
        }
    }
    return -1;
}

void objpool_release(obj_pool_t *p, int idx) {
    if (idx >= 0 && idx < p->count) {
        p->objects[idx].ref_count = p->objects[idx].ref_count - 1;
        if (p->objects[idx].ref_count <= 0) {
            p->in_use[idx] = 0;
        }
    }
}

int objpool_add_ref(obj_pool_t *p, int idx) {
    if (idx < 0 || idx >= p->count || !p->in_use[idx]) return -1;
    p->objects[idx].ref_count = p->objects[idx].ref_count + 1;
    return p->objects[idx].ref_count;
}

int objpool_active_count(const obj_pool_t *p) {
    int count = 0;
    int i;
    for (i = 0; i < p->count; i = i + 1) {
        if (p->in_use[i]) count = count + 1;
    }
    return count;
}

int objpool_reuse_rate(const obj_pool_t *p) {
    if (p->next_id <= 1) return 0;
    return (p->reuse_count * 100) / (p->next_id - 1);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C647: Object pool should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C647: empty output");
    assert!(code.contains("fn objpool_init"), "C647: Should contain objpool_init");
    assert!(code.contains("fn objpool_acquire"), "C647: Should contain objpool_acquire");
    assert!(code.contains("fn objpool_release"), "C647: Should contain objpool_release");
    Ok(())
}

/// C648: Concurrent hash map with bucket-level locking
#[test]
fn c648_concurrent_hash_map() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int key;
    int value;
    int occupied;
} chm_entry_t;

typedef struct {
    chm_entry_t entries[8];
    int count;
    int locked;
} chm_bucket_t;

typedef struct {
    chm_bucket_t buckets[16];
    int num_buckets;
    int total_count;
} conc_hashmap_t;

int chm_hash(int key) {
    int h = key;
    h = h ^ (h >> 16);
    h = h * 0x45d9f3b;
    h = h ^ (h >> 16);
    if (h < 0) h = -h;
    return h % 16;
}

void chm_init(conc_hashmap_t *m) {
    int i;
    int j;
    m->num_buckets = 16;
    m->total_count = 0;
    for (i = 0; i < 16; i = i + 1) {
        m->buckets[i].count = 0;
        m->buckets[i].locked = 0;
        for (j = 0; j < 8; j = j + 1) {
            m->buckets[i].entries[j].occupied = 0;
        }
    }
}

int chm_put(conc_hashmap_t *m, int key, int value) {
    int b = chm_hash(key);
    chm_bucket_t *bucket = &m->buckets[b];
    int i;
    for (i = 0; i < 8; i = i + 1) {
        if (bucket->entries[i].occupied && bucket->entries[i].key == key) {
            bucket->entries[i].value = value;
            return 0;
        }
    }
    for (i = 0; i < 8; i = i + 1) {
        if (!bucket->entries[i].occupied) {
            bucket->entries[i].key = key;
            bucket->entries[i].value = value;
            bucket->entries[i].occupied = 1;
            bucket->count = bucket->count + 1;
            m->total_count = m->total_count + 1;
            return 0;
        }
    }
    return -1;
}

int chm_get(const conc_hashmap_t *m, int key, int *out) {
    int b = chm_hash(key);
    const chm_bucket_t *bucket = &m->buckets[b];
    int i;
    for (i = 0; i < 8; i = i + 1) {
        if (bucket->entries[i].occupied && bucket->entries[i].key == key) {
            *out = bucket->entries[i].value;
            return 0;
        }
    }
    return -1;
}

int chm_remove(conc_hashmap_t *m, int key) {
    int b = chm_hash(key);
    chm_bucket_t *bucket = &m->buckets[b];
    int i;
    for (i = 0; i < 8; i = i + 1) {
        if (bucket->entries[i].occupied && bucket->entries[i].key == key) {
            bucket->entries[i].occupied = 0;
            bucket->count = bucket->count - 1;
            m->total_count = m->total_count - 1;
            return 0;
        }
    }
    return -1;
}

int chm_size(const conc_hashmap_t *m) {
    return m->total_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C648: Concurrent hash map should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C648: empty output");
    assert!(code.contains("fn chm_init"), "C648: Should contain chm_init");
    assert!(code.contains("fn chm_put"), "C648: Should contain chm_put");
    assert!(code.contains("fn chm_get"), "C648: Should contain chm_get");
    Ok(())
}

/// C649: Concurrent skip list with level generation
#[test]
fn c649_concurrent_skip_list() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int key;
    int value;
    int next[8];
    int level;
    int active;
} skip_node_t;

typedef struct {
    skip_node_t nodes[128];
    int head;
    int max_level;
    int count;
    int free_head;
    int rand_state;
} skip_list_t;

int sl_random_level(skip_list_t *sl) {
    int level = 1;
    sl->rand_state = sl->rand_state * 1103515245 + 12345;
    int r = (sl->rand_state >> 16) & 0x7FFF;
    while ((r & 1) == 1 && level < sl->max_level) {
        level = level + 1;
        r = r >> 1;
    }
    return level;
}

void sl_init(skip_list_t *sl) {
    int i;
    int j;
    sl->max_level = 8;
    sl->count = 0;
    sl->rand_state = 42;
    sl->head = 0;
    sl->free_head = 1;
    sl->nodes[0].key = -2147483647;
    sl->nodes[0].level = 8;
    sl->nodes[0].active = 1;
    for (j = 0; j < 8; j = j + 1) {
        sl->nodes[0].next[j] = -1;
    }
    for (i = 1; i < 127; i = i + 1) {
        sl->nodes[i].next[0] = i + 1;
        sl->nodes[i].active = 0;
    }
    sl->nodes[127].next[0] = -1;
    sl->nodes[127].active = 0;
}

int sl_alloc_node(skip_list_t *sl) {
    if (sl->free_head == -1) return -1;
    int n = sl->free_head;
    sl->free_head = sl->nodes[n].next[0];
    return n;
}

int sl_search(const skip_list_t *sl, int key) {
    int cur = sl->head;
    int lvl = sl->max_level - 1;
    while (lvl >= 0) {
        int nxt = sl->nodes[cur].next[lvl];
        while (nxt != -1 && sl->nodes[nxt].key < key) {
            cur = nxt;
            nxt = sl->nodes[cur].next[lvl];
        }
        lvl = lvl - 1;
    }
    int candidate = sl->nodes[cur].next[0];
    if (candidate != -1 && sl->nodes[candidate].key == key && sl->nodes[candidate].active) {
        return sl->nodes[candidate].value;
    }
    return -1;
}

int sl_insert(skip_list_t *sl, int key, int value) {
    int update[8];
    int cur = sl->head;
    int lvl;
    for (lvl = sl->max_level - 1; lvl >= 0; lvl = lvl - 1) {
        int nxt = sl->nodes[cur].next[lvl];
        while (nxt != -1 && sl->nodes[nxt].key < key) {
            cur = nxt;
            nxt = sl->nodes[cur].next[lvl];
        }
        update[lvl] = cur;
    }
    int n = sl_alloc_node(sl);
    if (n == -1) return -1;
    int new_level = sl_random_level(sl);
    sl->nodes[n].key = key;
    sl->nodes[n].value = value;
    sl->nodes[n].level = new_level;
    sl->nodes[n].active = 1;
    for (lvl = 0; lvl < new_level; lvl = lvl + 1) {
        sl->nodes[n].next[lvl] = sl->nodes[update[lvl]].next[lvl];
        sl->nodes[update[lvl]].next[lvl] = n;
    }
    sl->count = sl->count + 1;
    return 0;
}

int sl_size(const skip_list_t *sl) {
    return sl->count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C649: Skip list should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C649: empty output");
    assert!(code.contains("fn sl_init"), "C649: Should contain sl_init");
    assert!(code.contains("fn sl_search"), "C649: Should contain sl_search");
    assert!(code.contains("fn sl_insert"), "C649: Should contain sl_insert");
    Ok(())
}

/// C650: Hierarchical timer wheel
#[test]
fn c650_hierarchical_timer_wheel() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int callback_id;
    int expiry;
    int interval;
    int active;
} timer_entry_t;

typedef struct {
    timer_entry_t timers[256];
    int slots[64];
    int slot_counts[64];
    int current_tick;
    int num_slots;
    int timer_count;
    int fired_count;
} timer_wheel_t;

void tw_init(timer_wheel_t *tw) {
    int i;
    tw->current_tick = 0;
    tw->num_slots = 64;
    tw->timer_count = 0;
    tw->fired_count = 0;
    for (i = 0; i < 64; i = i + 1) {
        tw->slots[i] = 0;
        tw->slot_counts[i] = 0;
    }
    for (i = 0; i < 256; i = i + 1) {
        tw->timers[i].active = 0;
    }
}

int tw_add_timer(timer_wheel_t *tw, int callback_id, int delay, int interval) {
    if (tw->timer_count >= 256) return -1;
    int idx = 0;
    int i;
    for (i = 0; i < 256; i = i + 1) {
        if (!tw->timers[i].active) {
            idx = i;
            break;
        }
    }
    tw->timers[idx].callback_id = callback_id;
    tw->timers[idx].expiry = tw->current_tick + delay;
    tw->timers[idx].interval = interval;
    tw->timers[idx].active = 1;
    int slot = (tw->current_tick + delay) % tw->num_slots;
    tw->slot_counts[slot] = tw->slot_counts[slot] + 1;
    tw->timer_count = tw->timer_count + 1;
    return idx;
}

int tw_cancel_timer(timer_wheel_t *tw, int idx) {
    if (idx < 0 || idx >= 256 || !tw->timers[idx].active) return -1;
    int slot = tw->timers[idx].expiry % tw->num_slots;
    tw->timers[idx].active = 0;
    if (tw->slot_counts[slot] > 0) {
        tw->slot_counts[slot] = tw->slot_counts[slot] - 1;
    }
    tw->timer_count = tw->timer_count - 1;
    return 0;
}

int tw_tick(timer_wheel_t *tw, int *fired_ids, int max_fired) {
    tw->current_tick = tw->current_tick + 1;
    int slot = tw->current_tick % tw->num_slots;
    int fired = 0;
    int i;
    for (i = 0; i < 256 && fired < max_fired; i = i + 1) {
        if (tw->timers[i].active && tw->timers[i].expiry == tw->current_tick) {
            fired_ids[fired] = tw->timers[i].callback_id;
            fired = fired + 1;
            tw->fired_count = tw->fired_count + 1;
            if (tw->timers[i].interval > 0) {
                tw->timers[i].expiry = tw->current_tick + tw->timers[i].interval;
                int new_slot = tw->timers[i].expiry % tw->num_slots;
                tw->slot_counts[new_slot] = tw->slot_counts[new_slot] + 1;
            } else {
                tw->timers[i].active = 0;
                tw->timer_count = tw->timer_count - 1;
            }
            if (tw->slot_counts[slot] > 0) {
                tw->slot_counts[slot] = tw->slot_counts[slot] - 1;
            }
        }
    }
    return fired;
}

int tw_pending_count(const timer_wheel_t *tw) {
    return tw->timer_count;
}

int tw_total_fired(const timer_wheel_t *tw) {
    return tw->fired_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C650: Timer wheel should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C650: empty output");
    assert!(code.contains("fn tw_init"), "C650: Should contain tw_init");
    assert!(code.contains("fn tw_add_timer"), "C650: Should contain tw_add_timer");
    assert!(code.contains("fn tw_tick"), "C650: Should contain tw_tick");
    Ok(())
}
