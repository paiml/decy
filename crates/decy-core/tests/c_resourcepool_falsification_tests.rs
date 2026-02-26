//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1926-C1950: Resource Pooling and Management domain -- object pools,
//! connection pools, buffer pools, handle tables, and resource lifecycle.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise resource pooling and management patterns commonly
//! found in servers, databases, game engines, operating systems, and
//! high-performance networking -- all expressed as valid C99 without #include.
//!
//! Organization:
//! - C1926-C1930: Object pool (fixed-size pool, acquire, release, pool stats, pool exhaustion)
//! - C1931-C1935: Connection pool (connection slots, borrow, return, health check, timeout)
//! - C1936-C1940: Buffer pool (buffer allocation, buffer chain, buffer recycle, pool growth)
//! - C1941-C1945: Handle table (handle allocation, lookup, revoke, generation counter, compact)
//! - C1946-C1950: Resource lifecycle (init/deinit tracking, ref counting, leak detection, cleanup)

use decy_core::transpile;

// ============================================================================
// C1926-C1930: Object Pool
// ============================================================================

#[test]
fn c1926_fixed_size_object_pool() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define RP_OBJ_POOL_CAP 64
#define RP_OBJ_SIZE 128

typedef struct {
    char storage[RP_OBJ_POOL_CAP * RP_OBJ_SIZE];
    int free_stack[RP_OBJ_POOL_CAP];
    int free_top;
    int active_count;
    uint32_t total_acquires;
    uint32_t total_releases;
} rp_obj_pool_t;

void rp_obj_pool_init(rp_obj_pool_t *p) {
    int i;
    for (i = 0; i < RP_OBJ_POOL_CAP; i++) {
        p->free_stack[i] = i;
    }
    p->free_top = RP_OBJ_POOL_CAP;
    p->active_count = 0;
    p->total_acquires = 0;
    p->total_releases = 0;
}

void *rp_obj_pool_acquire(rp_obj_pool_t *p) {
    if (p->free_top <= 0) return (void *)0;
    p->free_top--;
    int idx = p->free_stack[p->free_top];
    p->active_count++;
    p->total_acquires++;
    return &p->storage[idx * RP_OBJ_SIZE];
}

void rp_obj_pool_release(rp_obj_pool_t *p, void *ptr) {
    char *base = p->storage;
    int idx = (int)((char *)ptr - base) / RP_OBJ_SIZE;
    if (idx >= 0 && idx < RP_OBJ_POOL_CAP) {
        p->free_stack[p->free_top++] = idx;
        p->active_count--;
        p->total_releases++;
    }
}

int rp_obj_pool_available(rp_obj_pool_t *p) {
    return p->free_top;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1926: Fixed-size object pool should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1926: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1926: Should contain rp_ functions");
}

#[test]
fn c1927_object_pool_acquire_pattern() {
    let c_code = r##"
typedef unsigned long size_t;

#define RP_ACQ_MAX 32
#define RP_ACQ_OBJ_SIZE 64

typedef struct {
    char objects[RP_ACQ_MAX * RP_ACQ_OBJ_SIZE];
    int in_use[RP_ACQ_MAX];
    int search_hint;
    int active;
    int peak_active;
} rp_acquire_pool_t;

void rp_acquire_pool_init(rp_acquire_pool_t *p) {
    int i;
    for (i = 0; i < RP_ACQ_MAX; i++) {
        p->in_use[i] = 0;
    }
    p->search_hint = 0;
    p->active = 0;
    p->peak_active = 0;
}

int rp_acquire_get(rp_acquire_pool_t *p) {
    int i;
    int start = p->search_hint;
    for (i = 0; i < RP_ACQ_MAX; i++) {
        int idx = (start + i) % RP_ACQ_MAX;
        if (!p->in_use[idx]) {
            p->in_use[idx] = 1;
            p->active++;
            if (p->active > p->peak_active) {
                p->peak_active = p->active;
            }
            p->search_hint = (idx + 1) % RP_ACQ_MAX;
            return idx;
        }
    }
    return -1;
}

void rp_acquire_put(rp_acquire_pool_t *p, int idx) {
    if (idx >= 0 && idx < RP_ACQ_MAX && p->in_use[idx]) {
        p->in_use[idx] = 0;
        p->active--;
        p->search_hint = idx;
    }
}

int rp_acquire_peak(rp_acquire_pool_t *p) {
    return p->peak_active;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1927: Object pool acquire pattern should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1927: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1927: Should contain rp_ functions");
}

#[test]
fn c1928_object_pool_release_batch() {
    let c_code = r##"
typedef unsigned long size_t;

#define RP_BATCH_POOL_SIZE 48

typedef struct {
    int slots[RP_BATCH_POOL_SIZE];
    int occupied[RP_BATCH_POOL_SIZE];
    int count;
    int batch_release_count;
} rp_batch_pool_t;

void rp_batch_pool_init(rp_batch_pool_t *p) {
    int i;
    for (i = 0; i < RP_BATCH_POOL_SIZE; i++) {
        p->occupied[i] = 0;
        p->slots[i] = 0;
    }
    p->count = 0;
    p->batch_release_count = 0;
}

int rp_batch_pool_alloc(rp_batch_pool_t *p, int value) {
    int i;
    for (i = 0; i < RP_BATCH_POOL_SIZE; i++) {
        if (!p->occupied[i]) {
            p->occupied[i] = 1;
            p->slots[i] = value;
            p->count++;
            return i;
        }
    }
    return -1;
}

void rp_batch_pool_free(rp_batch_pool_t *p, int idx) {
    if (idx >= 0 && idx < RP_BATCH_POOL_SIZE && p->occupied[idx]) {
        p->occupied[idx] = 0;
        p->slots[idx] = 0;
        p->count--;
    }
}

int rp_batch_pool_release_all_matching(rp_batch_pool_t *p, int value) {
    int released = 0;
    int i;
    for (i = 0; i < RP_BATCH_POOL_SIZE; i++) {
        if (p->occupied[i] && p->slots[i] == value) {
            p->occupied[i] = 0;
            p->count--;
            released++;
        }
    }
    p->batch_release_count++;
    return released;
}

int rp_batch_pool_count(rp_batch_pool_t *p) {
    return p->count;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1928: Object pool release batch should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1928: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1928: Should contain rp_ functions");
}

#[test]
fn c1929_object_pool_stats() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define RP_STATS_POOL_CAP 64

typedef struct {
    uint32_t total_acquires;
    uint32_t total_releases;
    uint32_t total_exhaustions;
    uint32_t current_active;
    uint32_t peak_active;
    uint32_t reuse_count;
} rp_pool_stats_t;

typedef struct {
    int in_use[RP_STATS_POOL_CAP];
    int reused[RP_STATS_POOL_CAP];
    rp_pool_stats_t stats;
} rp_stats_pool_t;

void rp_stats_pool_init(rp_stats_pool_t *p) {
    int i;
    for (i = 0; i < RP_STATS_POOL_CAP; i++) {
        p->in_use[i] = 0;
        p->reused[i] = 0;
    }
    p->stats.total_acquires = 0;
    p->stats.total_releases = 0;
    p->stats.total_exhaustions = 0;
    p->stats.current_active = 0;
    p->stats.peak_active = 0;
    p->stats.reuse_count = 0;
}

int rp_stats_pool_acquire(rp_stats_pool_t *p) {
    int i;
    for (i = 0; i < RP_STATS_POOL_CAP; i++) {
        if (!p->in_use[i]) {
            p->in_use[i] = 1;
            p->stats.total_acquires++;
            p->stats.current_active++;
            if (p->reused[i]) p->stats.reuse_count++;
            if (p->stats.current_active > p->stats.peak_active) {
                p->stats.peak_active = p->stats.current_active;
            }
            return i;
        }
    }
    p->stats.total_exhaustions++;
    return -1;
}

void rp_stats_pool_release(rp_stats_pool_t *p, int idx) {
    if (idx >= 0 && idx < RP_STATS_POOL_CAP && p->in_use[idx]) {
        p->in_use[idx] = 0;
        p->reused[idx] = 1;
        p->stats.total_releases++;
        p->stats.current_active--;
    }
}

int rp_stats_pool_utilization_pct(rp_stats_pool_t *p) {
    return (int)(p->stats.current_active * 100) / RP_STATS_POOL_CAP;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1929: Object pool stats should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1929: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1929: Should contain rp_ functions");
}

#[test]
fn c1930_object_pool_exhaustion() {
    let c_code = r##"
typedef unsigned long size_t;

#define RP_EXHAUST_CAP 16
#define RP_EXHAUST_WAIT_MAX 32

typedef struct {
    int occupied[RP_EXHAUST_CAP];
    int waiters[RP_EXHAUST_WAIT_MAX];
    int waiter_count;
    int active;
    int reject_count;
} rp_exhaust_pool_t;

void rp_exhaust_pool_init(rp_exhaust_pool_t *p) {
    int i;
    for (i = 0; i < RP_EXHAUST_CAP; i++) {
        p->occupied[i] = 0;
    }
    p->waiter_count = 0;
    p->active = 0;
    p->reject_count = 0;
}

int rp_exhaust_pool_try_acquire(rp_exhaust_pool_t *p) {
    int i;
    for (i = 0; i < RP_EXHAUST_CAP; i++) {
        if (!p->occupied[i]) {
            p->occupied[i] = 1;
            p->active++;
            return i;
        }
    }
    return -1;
}

int rp_exhaust_pool_acquire_or_wait(rp_exhaust_pool_t *p, int waiter_id) {
    int slot = rp_exhaust_pool_try_acquire(p);
    if (slot >= 0) return slot;
    if (p->waiter_count < RP_EXHAUST_WAIT_MAX) {
        p->waiters[p->waiter_count++] = waiter_id;
        return -2;
    }
    p->reject_count++;
    return -3;
}

int rp_exhaust_pool_release_and_notify(rp_exhaust_pool_t *p, int idx) {
    if (idx < 0 || idx >= RP_EXHAUST_CAP || !p->occupied[idx]) return -1;
    p->occupied[idx] = 0;
    p->active--;
    if (p->waiter_count > 0) {
        p->waiter_count--;
        return p->waiters[p->waiter_count];
    }
    return 0;
}

int rp_exhaust_pool_is_full(rp_exhaust_pool_t *p) {
    return p->active >= RP_EXHAUST_CAP;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1930: Object pool exhaustion should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1930: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1930: Should contain rp_ functions");
}

// ============================================================================
// C1931-C1935: Connection Pool
// ============================================================================

#[test]
fn c1931_connection_pool_slots() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define RP_CONN_MAX 32

typedef struct {
    int state;
    uint32_t id;
    int healthy;
    int use_count;
    int last_check_time;
} rp_conn_slot_t;

typedef struct {
    rp_conn_slot_t slots[RP_CONN_MAX];
    int count;
    uint32_t next_id;
    int max_per_host;
} rp_conn_pool_t;

void rp_conn_pool_init(rp_conn_pool_t *p, int max_per_host) {
    int i;
    for (i = 0; i < RP_CONN_MAX; i++) {
        p->slots[i].state = 0;
        p->slots[i].id = 0;
        p->slots[i].healthy = 0;
        p->slots[i].use_count = 0;
        p->slots[i].last_check_time = 0;
    }
    p->count = 0;
    p->next_id = 1;
    p->max_per_host = max_per_host;
}

int rp_conn_pool_create(rp_conn_pool_t *p) {
    if (p->count >= RP_CONN_MAX) return -1;
    int i;
    for (i = 0; i < RP_CONN_MAX; i++) {
        if (p->slots[i].state == 0) {
            p->slots[i].state = 1;
            p->slots[i].id = p->next_id++;
            p->slots[i].healthy = 1;
            p->slots[i].use_count = 0;
            p->count++;
            return i;
        }
    }
    return -1;
}

void rp_conn_pool_destroy(rp_conn_pool_t *p, int idx) {
    if (idx >= 0 && idx < RP_CONN_MAX && p->slots[idx].state != 0) {
        p->slots[idx].state = 0;
        p->slots[idx].healthy = 0;
        p->count--;
    }
}

int rp_conn_pool_active_count(rp_conn_pool_t *p) {
    return p->count;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1931: Connection pool slots should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1931: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1931: Should contain rp_ functions");
}

#[test]
fn c1932_connection_pool_borrow() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define RP_BORROW_MAX 24

typedef struct {
    int available;
    int borrowed;
    uint32_t borrow_count;
    int owner_id;
} rp_borrow_slot_t;

typedef struct {
    rp_borrow_slot_t slots[RP_BORROW_MAX];
    int total_slots;
    int borrowed_count;
    uint32_t contention_count;
} rp_borrow_pool_t;

void rp_borrow_pool_init(rp_borrow_pool_t *p, int initial_count) {
    int i;
    if (initial_count > RP_BORROW_MAX) initial_count = RP_BORROW_MAX;
    for (i = 0; i < RP_BORROW_MAX; i++) {
        p->slots[i].available = (i < initial_count) ? 1 : 0;
        p->slots[i].borrowed = 0;
        p->slots[i].borrow_count = 0;
        p->slots[i].owner_id = -1;
    }
    p->total_slots = initial_count;
    p->borrowed_count = 0;
    p->contention_count = 0;
}

int rp_borrow_pool_checkout(rp_borrow_pool_t *p, int owner_id) {
    int i;
    for (i = 0; i < p->total_slots; i++) {
        if (p->slots[i].available && !p->slots[i].borrowed) {
            p->slots[i].borrowed = 1;
            p->slots[i].owner_id = owner_id;
            p->slots[i].borrow_count++;
            p->borrowed_count++;
            return i;
        }
    }
    p->contention_count++;
    return -1;
}

int rp_borrow_pool_checkin(rp_borrow_pool_t *p, int idx, int owner_id) {
    if (idx < 0 || idx >= p->total_slots) return -1;
    if (!p->slots[idx].borrowed) return -2;
    if (p->slots[idx].owner_id != owner_id) return -3;
    p->slots[idx].borrowed = 0;
    p->slots[idx].owner_id = -1;
    p->borrowed_count--;
    return 0;
}

int rp_borrow_pool_free_count(rp_borrow_pool_t *p) {
    return p->total_slots - p->borrowed_count;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1932: Connection pool borrow should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1932: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1932: Should contain rp_ functions");
}

#[test]
fn c1933_connection_pool_return() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define RP_RET_POOL_MAX 20

typedef struct {
    int active;
    int dirty;
    uint32_t use_count;
    int needs_reset;
} rp_ret_conn_t;

typedef struct {
    rp_ret_conn_t conns[RP_RET_POOL_MAX];
    int count;
    int dirty_count;
    int reset_count;
} rp_return_pool_t;

void rp_return_pool_init(rp_return_pool_t *p) {
    int i;
    for (i = 0; i < RP_RET_POOL_MAX; i++) {
        p->conns[i].active = 0;
        p->conns[i].dirty = 0;
        p->conns[i].use_count = 0;
        p->conns[i].needs_reset = 0;
    }
    p->count = 0;
    p->dirty_count = 0;
    p->reset_count = 0;
}

int rp_return_pool_open(rp_return_pool_t *p) {
    int i;
    for (i = 0; i < RP_RET_POOL_MAX; i++) {
        if (!p->conns[i].active) {
            p->conns[i].active = 1;
            p->conns[i].dirty = 0;
            p->conns[i].needs_reset = 0;
            p->count++;
            return i;
        }
    }
    return -1;
}

int rp_return_pool_give_back(rp_return_pool_t *p, int idx, int was_dirty) {
    if (idx < 0 || idx >= RP_RET_POOL_MAX) return -1;
    if (!p->conns[idx].active) return -2;
    p->conns[idx].use_count++;
    if (was_dirty) {
        p->conns[idx].dirty = 1;
        p->conns[idx].needs_reset = 1;
        p->dirty_count++;
    }
    return 0;
}

int rp_return_pool_reset_dirty(rp_return_pool_t *p) {
    int reset = 0;
    int i;
    for (i = 0; i < RP_RET_POOL_MAX; i++) {
        if (p->conns[i].active && p->conns[i].needs_reset) {
            p->conns[i].dirty = 0;
            p->conns[i].needs_reset = 0;
            reset++;
        }
    }
    p->reset_count += reset;
    return reset;
}

int rp_return_pool_dirty_count(rp_return_pool_t *p) {
    return p->dirty_count;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1933: Connection pool return should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1933: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1933: Should contain rp_ functions");
}

#[test]
fn c1934_connection_pool_health_check() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define RP_HEALTH_MAX 16
#define RP_HEALTH_INTERVAL 30
#define RP_HEALTH_MAX_FAILURES 3

typedef struct {
    int alive;
    int healthy;
    int failure_count;
    int last_check;
    int total_checks;
} rp_health_conn_t;

typedef struct {
    rp_health_conn_t conns[RP_HEALTH_MAX];
    int count;
    int evicted;
    int current_time;
} rp_health_pool_t;

void rp_health_pool_init(rp_health_pool_t *p) {
    int i;
    for (i = 0; i < RP_HEALTH_MAX; i++) {
        p->conns[i].alive = 0;
        p->conns[i].healthy = 0;
        p->conns[i].failure_count = 0;
        p->conns[i].last_check = 0;
        p->conns[i].total_checks = 0;
    }
    p->count = 0;
    p->evicted = 0;
    p->current_time = 0;
}

int rp_health_pool_add(rp_health_pool_t *p) {
    int i;
    for (i = 0; i < RP_HEALTH_MAX; i++) {
        if (!p->conns[i].alive) {
            p->conns[i].alive = 1;
            p->conns[i].healthy = 1;
            p->conns[i].failure_count = 0;
            p->conns[i].last_check = p->current_time;
            p->count++;
            return i;
        }
    }
    return -1;
}

int rp_health_pool_check(rp_health_pool_t *p, int idx, int passed) {
    if (idx < 0 || idx >= RP_HEALTH_MAX || !p->conns[idx].alive) return -1;
    p->conns[idx].last_check = p->current_time;
    p->conns[idx].total_checks++;
    if (passed) {
        p->conns[idx].healthy = 1;
        p->conns[idx].failure_count = 0;
    } else {
        p->conns[idx].failure_count++;
        if (p->conns[idx].failure_count >= RP_HEALTH_MAX_FAILURES) {
            p->conns[idx].healthy = 0;
        }
    }
    return p->conns[idx].healthy;
}

int rp_health_pool_evict_unhealthy(rp_health_pool_t *p) {
    int evicted = 0;
    int i;
    for (i = 0; i < RP_HEALTH_MAX; i++) {
        if (p->conns[i].alive && !p->conns[i].healthy) {
            p->conns[i].alive = 0;
            p->count--;
            evicted++;
        }
    }
    p->evicted += evicted;
    return evicted;
}

int rp_health_pool_healthy_count(rp_health_pool_t *p) {
    int count = 0;
    int i;
    for (i = 0; i < RP_HEALTH_MAX; i++) {
        if (p->conns[i].alive && p->conns[i].healthy) count++;
    }
    return count;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1934: Connection pool health check should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1934: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1934: Should contain rp_ functions");
}

#[test]
fn c1935_connection_pool_timeout() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define RP_TIMEOUT_MAX 24
#define RP_IDLE_TIMEOUT 120
#define RP_MAX_LIFETIME 3600

typedef struct {
    int active;
    int idle;
    uint32_t created_at;
    uint32_t last_used_at;
    uint32_t use_count;
} rp_timeout_conn_t;

typedef struct {
    rp_timeout_conn_t conns[RP_TIMEOUT_MAX];
    int count;
    uint32_t current_time;
    int timeout_evictions;
    int lifetime_evictions;
} rp_timeout_pool_t;

void rp_timeout_pool_init(rp_timeout_pool_t *p) {
    int i;
    for (i = 0; i < RP_TIMEOUT_MAX; i++) {
        p->conns[i].active = 0;
        p->conns[i].idle = 0;
    }
    p->count = 0;
    p->current_time = 0;
    p->timeout_evictions = 0;
    p->lifetime_evictions = 0;
}

int rp_timeout_pool_open(rp_timeout_pool_t *p) {
    int i;
    for (i = 0; i < RP_TIMEOUT_MAX; i++) {
        if (!p->conns[i].active) {
            p->conns[i].active = 1;
            p->conns[i].idle = 1;
            p->conns[i].created_at = p->current_time;
            p->conns[i].last_used_at = p->current_time;
            p->conns[i].use_count = 0;
            p->count++;
            return i;
        }
    }
    return -1;
}

void rp_timeout_pool_touch(rp_timeout_pool_t *p, int idx) {
    if (idx >= 0 && idx < RP_TIMEOUT_MAX && p->conns[idx].active) {
        p->conns[idx].last_used_at = p->current_time;
        p->conns[idx].idle = 0;
        p->conns[idx].use_count++;
    }
}

int rp_timeout_pool_reap(rp_timeout_pool_t *p) {
    int reaped = 0;
    int i;
    for (i = 0; i < RP_TIMEOUT_MAX; i++) {
        if (!p->conns[i].active) continue;
        uint32_t idle_time = p->current_time - p->conns[i].last_used_at;
        uint32_t lifetime = p->current_time - p->conns[i].created_at;
        if (idle_time > RP_IDLE_TIMEOUT) {
            p->conns[i].active = 0;
            p->count--;
            p->timeout_evictions++;
            reaped++;
        } else if (lifetime > RP_MAX_LIFETIME) {
            p->conns[i].active = 0;
            p->count--;
            p->lifetime_evictions++;
            reaped++;
        }
    }
    return reaped;
}

void rp_timeout_pool_advance_time(rp_timeout_pool_t *p, uint32_t delta) {
    p->current_time += delta;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1935: Connection pool timeout should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1935: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1935: Should contain rp_ functions");
}

// ============================================================================
// C1936-C1940: Buffer Pool
// ============================================================================

#[test]
fn c1936_buffer_pool_allocation() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define RP_BUF_POOL_SIZE 32
#define RP_BUF_SIZE 256

typedef struct {
    char data[RP_BUF_SIZE];
    int in_use;
    size_t used_bytes;
    uint32_t alloc_id;
} rp_buffer_t;

typedef struct {
    rp_buffer_t buffers[RP_BUF_POOL_SIZE];
    int free_count;
    uint32_t next_alloc_id;
    uint32_t total_allocs;
} rp_buf_pool_t;

void rp_buf_pool_init(rp_buf_pool_t *p) {
    int i;
    for (i = 0; i < RP_BUF_POOL_SIZE; i++) {
        p->buffers[i].in_use = 0;
        p->buffers[i].used_bytes = 0;
        p->buffers[i].alloc_id = 0;
    }
    p->free_count = RP_BUF_POOL_SIZE;
    p->next_alloc_id = 1;
    p->total_allocs = 0;
}

int rp_buf_pool_alloc(rp_buf_pool_t *p) {
    int i;
    for (i = 0; i < RP_BUF_POOL_SIZE; i++) {
        if (!p->buffers[i].in_use) {
            p->buffers[i].in_use = 1;
            p->buffers[i].used_bytes = 0;
            p->buffers[i].alloc_id = p->next_alloc_id++;
            p->free_count--;
            p->total_allocs++;
            return i;
        }
    }
    return -1;
}

int rp_buf_pool_write(rp_buf_pool_t *p, int idx, size_t bytes) {
    if (idx < 0 || idx >= RP_BUF_POOL_SIZE) return -1;
    if (!p->buffers[idx].in_use) return -2;
    if (p->buffers[idx].used_bytes + bytes > RP_BUF_SIZE) return -3;
    p->buffers[idx].used_bytes += bytes;
    return 0;
}

void rp_buf_pool_free(rp_buf_pool_t *p, int idx) {
    if (idx >= 0 && idx < RP_BUF_POOL_SIZE && p->buffers[idx].in_use) {
        p->buffers[idx].in_use = 0;
        p->buffers[idx].used_bytes = 0;
        p->free_count++;
    }
}

int rp_buf_pool_free_count(rp_buf_pool_t *p) {
    return p->free_count;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1936: Buffer pool allocation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1936: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1936: Should contain rp_ functions");
}

#[test]
fn c1937_buffer_chain() {
    let c_code = r##"
typedef unsigned long size_t;

#define RP_CHAIN_MAX_BUFS 16
#define RP_CHAIN_BUF_SIZE 512

typedef struct {
    char data[RP_CHAIN_BUF_SIZE];
    size_t used;
    int next;
} rp_chain_buf_t;

typedef struct {
    rp_chain_buf_t bufs[RP_CHAIN_MAX_BUFS];
    int allocated[RP_CHAIN_MAX_BUFS];
    int head;
    int tail;
    int chain_length;
    size_t total_bytes;
} rp_buf_chain_t;

void rp_buf_chain_init(rp_buf_chain_t *c) {
    int i;
    for (i = 0; i < RP_CHAIN_MAX_BUFS; i++) {
        c->bufs[i].used = 0;
        c->bufs[i].next = -1;
        c->allocated[i] = 0;
    }
    c->head = -1;
    c->tail = -1;
    c->chain_length = 0;
    c->total_bytes = 0;
}

int rp_buf_chain_alloc_buf(rp_buf_chain_t *c) {
    int i;
    for (i = 0; i < RP_CHAIN_MAX_BUFS; i++) {
        if (!c->allocated[i]) {
            c->allocated[i] = 1;
            c->bufs[i].used = 0;
            c->bufs[i].next = -1;
            return i;
        }
    }
    return -1;
}

int rp_buf_chain_append(rp_buf_chain_t *c, size_t bytes) {
    int idx = rp_buf_chain_alloc_buf(c);
    if (idx < 0) return -1;
    c->bufs[idx].used = (bytes > RP_CHAIN_BUF_SIZE) ? RP_CHAIN_BUF_SIZE : bytes;
    if (c->tail >= 0) {
        c->bufs[c->tail].next = idx;
    }
    c->tail = idx;
    if (c->head < 0) c->head = idx;
    c->chain_length++;
    c->total_bytes += c->bufs[idx].used;
    return idx;
}

void rp_buf_chain_pop_front(rp_buf_chain_t *c) {
    if (c->head < 0) return;
    int old_head = c->head;
    c->total_bytes -= c->bufs[old_head].used;
    c->head = c->bufs[old_head].next;
    c->allocated[old_head] = 0;
    c->chain_length--;
    if (c->head < 0) c->tail = -1;
}

size_t rp_buf_chain_total(rp_buf_chain_t *c) {
    return c->total_bytes;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1937: Buffer chain should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1937: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1937: Should contain rp_ functions");
}

#[test]
fn c1938_buffer_recycle() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define RP_RECYCLE_MAX 24

typedef struct {
    int in_use;
    size_t capacity;
    size_t used;
    uint32_t recycle_count;
} rp_recycle_buf_t;

typedef struct {
    rp_recycle_buf_t bufs[RP_RECYCLE_MAX];
    int free_stack[RP_RECYCLE_MAX];
    int free_top;
    uint32_t total_recycled;
    uint32_t total_fresh;
} rp_recycle_pool_t;

void rp_recycle_pool_init(rp_recycle_pool_t *p, size_t buf_cap) {
    int i;
    for (i = 0; i < RP_RECYCLE_MAX; i++) {
        p->bufs[i].in_use = 0;
        p->bufs[i].capacity = buf_cap;
        p->bufs[i].used = 0;
        p->bufs[i].recycle_count = 0;
        p->free_stack[i] = i;
    }
    p->free_top = RP_RECYCLE_MAX;
    p->total_recycled = 0;
    p->total_fresh = 0;
}

int rp_recycle_pool_get(rp_recycle_pool_t *p) {
    if (p->free_top <= 0) return -1;
    p->free_top--;
    int idx = p->free_stack[p->free_top];
    p->bufs[idx].in_use = 1;
    p->bufs[idx].used = 0;
    if (p->bufs[idx].recycle_count > 0) {
        p->total_recycled++;
    } else {
        p->total_fresh++;
    }
    return idx;
}

void rp_recycle_pool_put(rp_recycle_pool_t *p, int idx) {
    if (idx < 0 || idx >= RP_RECYCLE_MAX || !p->bufs[idx].in_use) return;
    p->bufs[idx].in_use = 0;
    p->bufs[idx].used = 0;
    p->bufs[idx].recycle_count++;
    p->free_stack[p->free_top++] = idx;
}

int rp_recycle_pool_recycle_ratio(rp_recycle_pool_t *p) {
    uint32_t total = p->total_recycled + p->total_fresh;
    if (total == 0) return 0;
    return (int)((p->total_recycled * 100) / total);
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1938: Buffer recycle should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1938: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1938: Should contain rp_ functions");
}

#[test]
fn c1939_buffer_pool_growth() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define RP_GROW_INITIAL 8
#define RP_GROW_MAX 64
#define RP_GROW_THRESHOLD_PCT 75

typedef struct {
    int active[RP_GROW_MAX];
    int capacity;
    int used;
    int grow_events;
    int shrink_events;
    uint32_t high_watermark;
} rp_growable_pool_t;

void rp_growable_pool_init(rp_growable_pool_t *p) {
    int i;
    for (i = 0; i < RP_GROW_MAX; i++) {
        p->active[i] = 0;
    }
    p->capacity = RP_GROW_INITIAL;
    p->used = 0;
    p->grow_events = 0;
    p->shrink_events = 0;
    p->high_watermark = 0;
}

int rp_growable_pool_should_grow(rp_growable_pool_t *p) {
    return (p->used * 100 / p->capacity) >= RP_GROW_THRESHOLD_PCT;
}

int rp_growable_pool_grow(rp_growable_pool_t *p) {
    int new_cap = p->capacity * 2;
    if (new_cap > RP_GROW_MAX) new_cap = RP_GROW_MAX;
    if (new_cap <= p->capacity) return 0;
    p->capacity = new_cap;
    p->grow_events++;
    return new_cap;
}

int rp_growable_pool_alloc(rp_growable_pool_t *p) {
    if (p->used >= p->capacity) {
        if (rp_growable_pool_grow(p) == 0) return -1;
    }
    int i;
    for (i = 0; i < p->capacity; i++) {
        if (!p->active[i]) {
            p->active[i] = 1;
            p->used++;
            if (p->used > (int)p->high_watermark) {
                p->high_watermark = (uint32_t)p->used;
            }
            return i;
        }
    }
    return -1;
}

void rp_growable_pool_free(rp_growable_pool_t *p, int idx) {
    if (idx >= 0 && idx < p->capacity && p->active[idx]) {
        p->active[idx] = 0;
        p->used--;
    }
}

int rp_growable_pool_capacity(rp_growable_pool_t *p) {
    return p->capacity;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1939: Buffer pool growth should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1939: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1939: Should contain rp_ functions");
}

#[test]
fn c1940_buffer_pool_size_classes() {
    let c_code = r##"
typedef unsigned long size_t;

#define RP_SIZE_CLASSES 4
#define RP_PER_CLASS 8

typedef struct {
    int free_count;
    size_t buf_size;
    int alloc_count;
    int miss_count;
} rp_size_class_t;

typedef struct {
    rp_size_class_t classes[RP_SIZE_CLASSES];
    int total_allocs;
    int total_misses;
} rp_sized_buf_pool_t;

void rp_sized_buf_pool_init(rp_sized_buf_pool_t *p) {
    size_t sizes[4] = {64, 256, 1024, 4096};
    int i;
    for (i = 0; i < RP_SIZE_CLASSES; i++) {
        p->classes[i].buf_size = sizes[i];
        p->classes[i].free_count = RP_PER_CLASS;
        p->classes[i].alloc_count = 0;
        p->classes[i].miss_count = 0;
    }
    p->total_allocs = 0;
    p->total_misses = 0;
}

int rp_sized_buf_pool_find_class(rp_sized_buf_pool_t *p, size_t needed) {
    int i;
    for (i = 0; i < RP_SIZE_CLASSES; i++) {
        if (needed <= p->classes[i].buf_size) return i;
    }
    return -1;
}

int rp_sized_buf_pool_alloc(rp_sized_buf_pool_t *p, size_t needed) {
    int cls = rp_sized_buf_pool_find_class(p, needed);
    if (cls < 0) {
        p->total_misses++;
        return -1;
    }
    if (p->classes[cls].free_count <= 0) {
        p->classes[cls].miss_count++;
        p->total_misses++;
        return -2;
    }
    p->classes[cls].free_count--;
    p->classes[cls].alloc_count++;
    p->total_allocs++;
    return cls;
}

void rp_sized_buf_pool_free(rp_sized_buf_pool_t *p, int cls) {
    if (cls >= 0 && cls < RP_SIZE_CLASSES) {
        if (p->classes[cls].free_count < RP_PER_CLASS) {
            p->classes[cls].free_count++;
        }
    }
}

int rp_sized_buf_pool_total_free(rp_sized_buf_pool_t *p) {
    int total = 0;
    int i;
    for (i = 0; i < RP_SIZE_CLASSES; i++) {
        total += p->classes[i].free_count;
    }
    return total;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1940: Buffer pool size classes should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1940: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1940: Should contain rp_ functions");
}

// ============================================================================
// C1941-C1945: Handle Table
// ============================================================================

#[test]
fn c1941_handle_allocation() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define RP_HANDLE_MAX 128

typedef struct {
    int active;
    uint32_t generation;
    int data_index;
} rp_handle_entry_t;

typedef struct {
    rp_handle_entry_t entries[RP_HANDLE_MAX];
    int free_list[RP_HANDLE_MAX];
    int free_top;
    int count;
} rp_handle_table_t;

void rp_handle_table_init(rp_handle_table_t *t) {
    int i;
    for (i = 0; i < RP_HANDLE_MAX; i++) {
        t->entries[i].active = 0;
        t->entries[i].generation = 0;
        t->entries[i].data_index = -1;
        t->free_list[i] = i;
    }
    t->free_top = RP_HANDLE_MAX;
    t->count = 0;
}

uint32_t rp_handle_table_alloc(rp_handle_table_t *t, int data_index) {
    if (t->free_top <= 0) return 0;
    t->free_top--;
    int idx = t->free_list[t->free_top];
    t->entries[idx].active = 1;
    t->entries[idx].generation++;
    t->entries[idx].data_index = data_index;
    t->count++;
    return (t->entries[idx].generation << 16) | (uint32_t)idx;
}

void rp_handle_table_free(rp_handle_table_t *t, int idx) {
    if (idx >= 0 && idx < RP_HANDLE_MAX && t->entries[idx].active) {
        t->entries[idx].active = 0;
        t->entries[idx].data_index = -1;
        t->free_list[t->free_top++] = idx;
        t->count--;
    }
}

int rp_handle_table_count(rp_handle_table_t *t) {
    return t->count;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1941: Handle allocation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1941: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1941: Should contain rp_ functions");
}

#[test]
fn c1942_handle_lookup() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define RP_LOOKUP_MAX 64

typedef struct {
    int active;
    uint32_t generation;
    int value;
    int access_count;
} rp_lookup_entry_t;

typedef struct {
    rp_lookup_entry_t entries[RP_LOOKUP_MAX];
    int count;
    int miss_count;
    int stale_count;
} rp_lookup_table_t;

void rp_lookup_table_init(rp_lookup_table_t *t) {
    int i;
    for (i = 0; i < RP_LOOKUP_MAX; i++) {
        t->entries[i].active = 0;
        t->entries[i].generation = 0;
        t->entries[i].value = 0;
        t->entries[i].access_count = 0;
    }
    t->count = 0;
    t->miss_count = 0;
    t->stale_count = 0;
}

int rp_lookup_table_insert(rp_lookup_table_t *t, int value) {
    int i;
    for (i = 0; i < RP_LOOKUP_MAX; i++) {
        if (!t->entries[i].active) {
            t->entries[i].active = 1;
            t->entries[i].generation++;
            t->entries[i].value = value;
            t->entries[i].access_count = 0;
            t->count++;
            return i;
        }
    }
    return -1;
}

int rp_lookup_table_get(rp_lookup_table_t *t, int idx, uint32_t expected_gen) {
    if (idx < 0 || idx >= RP_LOOKUP_MAX) {
        t->miss_count++;
        return -1;
    }
    if (!t->entries[idx].active) {
        t->miss_count++;
        return -2;
    }
    if (t->entries[idx].generation != expected_gen) {
        t->stale_count++;
        return -3;
    }
    t->entries[idx].access_count++;
    return t->entries[idx].value;
}

void rp_lookup_table_remove(rp_lookup_table_t *t, int idx) {
    if (idx >= 0 && idx < RP_LOOKUP_MAX && t->entries[idx].active) {
        t->entries[idx].active = 0;
        t->count--;
    }
}

int rp_lookup_table_miss_rate(rp_lookup_table_t *t) {
    int total = t->miss_count + t->stale_count + t->count;
    if (total == 0) return 0;
    return (t->miss_count * 100) / total;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1942: Handle lookup should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1942: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1942: Should contain rp_ functions");
}

#[test]
fn c1943_handle_revoke() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define RP_REVOKE_MAX 48

typedef struct {
    int active;
    uint32_t generation;
    int owner;
    int revoked;
} rp_revoke_entry_t;

typedef struct {
    rp_revoke_entry_t entries[RP_REVOKE_MAX];
    int count;
    int revoke_count;
    int total_revoked;
} rp_revoke_table_t;

void rp_revoke_table_init(rp_revoke_table_t *t) {
    int i;
    for (i = 0; i < RP_REVOKE_MAX; i++) {
        t->entries[i].active = 0;
        t->entries[i].generation = 0;
        t->entries[i].owner = -1;
        t->entries[i].revoked = 0;
    }
    t->count = 0;
    t->revoke_count = 0;
    t->total_revoked = 0;
}

int rp_revoke_table_grant(rp_revoke_table_t *t, int owner) {
    int i;
    for (i = 0; i < RP_REVOKE_MAX; i++) {
        if (!t->entries[i].active) {
            t->entries[i].active = 1;
            t->entries[i].generation++;
            t->entries[i].owner = owner;
            t->entries[i].revoked = 0;
            t->count++;
            return i;
        }
    }
    return -1;
}

int rp_revoke_table_revoke(rp_revoke_table_t *t, int idx) {
    if (idx < 0 || idx >= RP_REVOKE_MAX) return -1;
    if (!t->entries[idx].active) return -2;
    if (t->entries[idx].revoked) return -3;
    t->entries[idx].revoked = 1;
    t->entries[idx].generation++;
    t->revoke_count++;
    t->total_revoked++;
    return 0;
}

int rp_revoke_table_is_valid(rp_revoke_table_t *t, int idx, uint32_t gen) {
    if (idx < 0 || idx >= RP_REVOKE_MAX) return 0;
    if (!t->entries[idx].active) return 0;
    if (t->entries[idx].revoked) return 0;
    return t->entries[idx].generation == gen;
}

int rp_revoke_table_revoke_by_owner(rp_revoke_table_t *t, int owner) {
    int revoked = 0;
    int i;
    for (i = 0; i < RP_REVOKE_MAX; i++) {
        if (t->entries[i].active && !t->entries[i].revoked &&
            t->entries[i].owner == owner) {
            t->entries[i].revoked = 1;
            t->entries[i].generation++;
            revoked++;
        }
    }
    t->total_revoked += revoked;
    return revoked;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1943: Handle revoke should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1943: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1943: Should contain rp_ functions");
}

#[test]
fn c1944_handle_generation_counter() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define RP_GEN_MAX 32

typedef struct {
    uint32_t generation;
    int active;
    int data;
} rp_gen_slot_t;

typedef struct {
    rp_gen_slot_t slots[RP_GEN_MAX];
    int free_list[RP_GEN_MAX];
    int free_top;
    uint32_t global_gen;
    int stale_access_count;
} rp_gen_table_t;

void rp_gen_table_init(rp_gen_table_t *t) {
    int i;
    for (i = 0; i < RP_GEN_MAX; i++) {
        t->slots[i].generation = 0;
        t->slots[i].active = 0;
        t->slots[i].data = 0;
        t->free_list[i] = i;
    }
    t->free_top = RP_GEN_MAX;
    t->global_gen = 1;
    t->stale_access_count = 0;
}

uint32_t rp_gen_table_create(rp_gen_table_t *t, int data) {
    if (t->free_top <= 0) return 0;
    t->free_top--;
    int idx = t->free_list[t->free_top];
    t->slots[idx].generation = t->global_gen++;
    t->slots[idx].active = 1;
    t->slots[idx].data = data;
    return (t->slots[idx].generation << 8) | (uint32_t)idx;
}

int rp_gen_table_resolve(rp_gen_table_t *t, uint32_t handle) {
    int idx = (int)(handle & 0xFF);
    uint32_t gen = handle >> 8;
    if (idx < 0 || idx >= RP_GEN_MAX) return -1;
    if (!t->slots[idx].active) return -2;
    if (t->slots[idx].generation != gen) {
        t->stale_access_count++;
        return -3;
    }
    return t->slots[idx].data;
}

void rp_gen_table_destroy(rp_gen_table_t *t, int idx) {
    if (idx >= 0 && idx < RP_GEN_MAX && t->slots[idx].active) {
        t->slots[idx].active = 0;
        t->free_list[t->free_top++] = idx;
    }
}

int rp_gen_table_stale_count(rp_gen_table_t *t) {
    return t->stale_access_count;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1944: Handle generation counter should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1944: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1944: Should contain rp_ functions");
}

#[test]
fn c1945_handle_table_compact() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define RP_COMPACT_MAX 64

typedef struct {
    int active;
    int data;
    int original_index;
} rp_compact_entry_t;

typedef struct {
    rp_compact_entry_t entries[RP_COMPACT_MAX];
    int count;
    int active_count;
    int compact_ops;
    int holes;
} rp_compact_table_t;

void rp_compact_table_init(rp_compact_table_t *t) {
    int i;
    for (i = 0; i < RP_COMPACT_MAX; i++) {
        t->entries[i].active = 0;
        t->entries[i].data = 0;
        t->entries[i].original_index = i;
    }
    t->count = 0;
    t->active_count = 0;
    t->compact_ops = 0;
    t->holes = 0;
}

int rp_compact_table_add(rp_compact_table_t *t, int data) {
    if (t->count >= RP_COMPACT_MAX) return -1;
    int idx = t->count++;
    t->entries[idx].active = 1;
    t->entries[idx].data = data;
    t->entries[idx].original_index = idx;
    t->active_count++;
    return idx;
}

void rp_compact_table_remove(rp_compact_table_t *t, int idx) {
    if (idx >= 0 && idx < t->count && t->entries[idx].active) {
        t->entries[idx].active = 0;
        t->active_count--;
        t->holes++;
    }
}

int rp_compact_table_compact(rp_compact_table_t *t) {
    int write_pos = 0;
    int moved = 0;
    int i;
    for (i = 0; i < t->count; i++) {
        if (t->entries[i].active) {
            if (i != write_pos) {
                t->entries[write_pos] = t->entries[i];
                t->entries[i].active = 0;
                moved++;
            }
            write_pos++;
        }
    }
    t->count = write_pos;
    t->holes = 0;
    t->compact_ops++;
    return moved;
}

int rp_compact_table_fragmentation_pct(rp_compact_table_t *t) {
    if (t->count == 0) return 0;
    return (t->holes * 100) / t->count;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1945: Handle table compact should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1945: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1945: Should contain rp_ functions");
}

// ============================================================================
// C1946-C1950: Resource Lifecycle
// ============================================================================

#[test]
fn c1946_init_deinit_tracking() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define RP_LIFECYCLE_MAX 32

typedef struct {
    int initialized;
    int finalized;
    uint32_t init_order;
    uint32_t fini_order;
    int resource_type;
} rp_lifecycle_entry_t;

typedef struct {
    rp_lifecycle_entry_t entries[RP_LIFECYCLE_MAX];
    int count;
    uint32_t next_init_order;
    uint32_t next_fini_order;
    int init_errors;
    int fini_errors;
} rp_lifecycle_tracker_t;

void rp_lifecycle_tracker_init(rp_lifecycle_tracker_t *t) {
    int i;
    for (i = 0; i < RP_LIFECYCLE_MAX; i++) {
        t->entries[i].initialized = 0;
        t->entries[i].finalized = 0;
        t->entries[i].init_order = 0;
        t->entries[i].fini_order = 0;
        t->entries[i].resource_type = 0;
    }
    t->count = 0;
    t->next_init_order = 1;
    t->next_fini_order = 1;
    t->init_errors = 0;
    t->fini_errors = 0;
}

int rp_lifecycle_tracker_register_init(rp_lifecycle_tracker_t *t, int rtype) {
    if (t->count >= RP_LIFECYCLE_MAX) return -1;
    int idx = t->count++;
    t->entries[idx].initialized = 1;
    t->entries[idx].finalized = 0;
    t->entries[idx].init_order = t->next_init_order++;
    t->entries[idx].resource_type = rtype;
    return idx;
}

int rp_lifecycle_tracker_register_fini(rp_lifecycle_tracker_t *t, int idx) {
    if (idx < 0 || idx >= t->count) return -1;
    if (!t->entries[idx].initialized) {
        t->fini_errors++;
        return -2;
    }
    if (t->entries[idx].finalized) {
        t->fini_errors++;
        return -3;
    }
    t->entries[idx].finalized = 1;
    t->entries[idx].fini_order = t->next_fini_order++;
    return 0;
}

int rp_lifecycle_tracker_check_leaks(rp_lifecycle_tracker_t *t) {
    int leaks = 0;
    int i;
    for (i = 0; i < t->count; i++) {
        if (t->entries[i].initialized && !t->entries[i].finalized) {
            leaks++;
        }
    }
    return leaks;
}

int rp_lifecycle_tracker_total(rp_lifecycle_tracker_t *t) {
    return t->count;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1946: Init/deinit tracking should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1946: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1946: Should contain rp_ functions");
}

#[test]
fn c1947_reference_counting() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define RP_REFCOUNT_MAX 48

typedef struct {
    int active;
    uint32_t ref_count;
    int data;
    int destroy_count;
} rp_refcounted_t;

typedef struct {
    rp_refcounted_t objects[RP_REFCOUNT_MAX];
    int count;
    int total_created;
    int total_destroyed;
} rp_refcount_pool_t;

void rp_refcount_pool_init(rp_refcount_pool_t *p) {
    int i;
    for (i = 0; i < RP_REFCOUNT_MAX; i++) {
        p->objects[i].active = 0;
        p->objects[i].ref_count = 0;
        p->objects[i].data = 0;
        p->objects[i].destroy_count = 0;
    }
    p->count = 0;
    p->total_created = 0;
    p->total_destroyed = 0;
}

int rp_refcount_pool_create(rp_refcount_pool_t *p, int data) {
    int i;
    for (i = 0; i < RP_REFCOUNT_MAX; i++) {
        if (!p->objects[i].active) {
            p->objects[i].active = 1;
            p->objects[i].ref_count = 1;
            p->objects[i].data = data;
            p->count++;
            p->total_created++;
            return i;
        }
    }
    return -1;
}

int rp_refcount_pool_retain(rp_refcount_pool_t *p, int idx) {
    if (idx < 0 || idx >= RP_REFCOUNT_MAX) return -1;
    if (!p->objects[idx].active) return -2;
    p->objects[idx].ref_count++;
    return (int)p->objects[idx].ref_count;
}

int rp_refcount_pool_release(rp_refcount_pool_t *p, int idx) {
    if (idx < 0 || idx >= RP_REFCOUNT_MAX) return -1;
    if (!p->objects[idx].active) return -2;
    if (p->objects[idx].ref_count == 0) return -3;
    p->objects[idx].ref_count--;
    if (p->objects[idx].ref_count == 0) {
        p->objects[idx].active = 0;
        p->objects[idx].destroy_count++;
        p->count--;
        p->total_destroyed++;
        return 0;
    }
    return (int)p->objects[idx].ref_count;
}

uint32_t rp_refcount_pool_get_count(rp_refcount_pool_t *p, int idx) {
    if (idx < 0 || idx >= RP_REFCOUNT_MAX || !p->objects[idx].active) return 0;
    return p->objects[idx].ref_count;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1947: Reference counting should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1947: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1947: Should contain rp_ functions");
}

#[test]
fn c1948_resource_leak_detection() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define RP_LEAK_DET_MAX 64

typedef struct {
    int allocated;
    int freed;
    size_t size;
    int tag;
    uint32_t alloc_time;
} rp_leak_record_t;

typedef struct {
    rp_leak_record_t records[RP_LEAK_DET_MAX];
    int count;
    uint32_t current_time;
    int detected_leaks;
    size_t leaked_bytes;
} rp_leak_detector_t;

void rp_leak_detector_init(rp_leak_detector_t *d) {
    int i;
    for (i = 0; i < RP_LEAK_DET_MAX; i++) {
        d->records[i].allocated = 0;
        d->records[i].freed = 0;
    }
    d->count = 0;
    d->current_time = 0;
    d->detected_leaks = 0;
    d->leaked_bytes = 0;
}

int rp_leak_detector_track_alloc(rp_leak_detector_t *d, size_t size, int tag) {
    if (d->count >= RP_LEAK_DET_MAX) return -1;
    int idx = d->count++;
    d->records[idx].allocated = 1;
    d->records[idx].freed = 0;
    d->records[idx].size = size;
    d->records[idx].tag = tag;
    d->records[idx].alloc_time = d->current_time;
    return idx;
}

int rp_leak_detector_track_free(rp_leak_detector_t *d, int idx) {
    if (idx < 0 || idx >= d->count) return -1;
    if (!d->records[idx].allocated) return -2;
    if (d->records[idx].freed) return -3;
    d->records[idx].freed = 1;
    return 0;
}

int rp_leak_detector_scan(rp_leak_detector_t *d) {
    int leaks = 0;
    size_t bytes = 0;
    int i;
    for (i = 0; i < d->count; i++) {
        if (d->records[i].allocated && !d->records[i].freed) {
            leaks++;
            bytes += d->records[i].size;
        }
    }
    d->detected_leaks = leaks;
    d->leaked_bytes = bytes;
    return leaks;
}

size_t rp_leak_detector_leaked_bytes(rp_leak_detector_t *d) {
    return d->leaked_bytes;
}

int rp_leak_detector_leaks_by_tag(rp_leak_detector_t *d, int tag) {
    int leaks = 0;
    int i;
    for (i = 0; i < d->count; i++) {
        if (d->records[i].allocated && !d->records[i].freed &&
            d->records[i].tag == tag) {
            leaks++;
        }
    }
    return leaks;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1948: Resource leak detection should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1948: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1948: Should contain rp_ functions");
}

#[test]
fn c1949_resource_cleanup_stack() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define RP_CLEANUP_MAX 32

typedef struct {
    int resource_id;
    int resource_type;
    int cleaned;
} rp_cleanup_entry_t;

typedef struct {
    rp_cleanup_entry_t stack[RP_CLEANUP_MAX];
    int top;
    int total_cleaned;
    int total_registered;
    int cleanup_errors;
} rp_cleanup_stack_t;

void rp_cleanup_stack_init(rp_cleanup_stack_t *s) {
    s->top = 0;
    s->total_cleaned = 0;
    s->total_registered = 0;
    s->cleanup_errors = 0;
}

int rp_cleanup_stack_push(rp_cleanup_stack_t *s, int res_id, int res_type) {
    if (s->top >= RP_CLEANUP_MAX) return -1;
    s->stack[s->top].resource_id = res_id;
    s->stack[s->top].resource_type = res_type;
    s->stack[s->top].cleaned = 0;
    s->top++;
    s->total_registered++;
    return s->top - 1;
}

int rp_cleanup_stack_pop_and_clean(rp_cleanup_stack_t *s) {
    if (s->top <= 0) return -1;
    s->top--;
    if (s->stack[s->top].cleaned) {
        s->cleanup_errors++;
        return -2;
    }
    s->stack[s->top].cleaned = 1;
    s->total_cleaned++;
    return s->stack[s->top].resource_id;
}

int rp_cleanup_stack_unwind_all(rp_cleanup_stack_t *s) {
    int cleaned = 0;
    while (s->top > 0) {
        s->top--;
        if (!s->stack[s->top].cleaned) {
            s->stack[s->top].cleaned = 1;
            cleaned++;
        }
    }
    s->total_cleaned += cleaned;
    return cleaned;
}

int rp_cleanup_stack_pending(rp_cleanup_stack_t *s) {
    int pending = 0;
    int i;
    for (i = 0; i < s->top; i++) {
        if (!s->stack[i].cleaned) pending++;
    }
    return pending;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1949: Resource cleanup stack should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1949: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1949: Should contain rp_ functions");
}

#[test]
fn c1950_resource_scope_guard() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define RP_SCOPE_MAX_DEPTH 8
#define RP_SCOPE_MAX_RESOURCES 16

typedef struct {
    int resource_ids[RP_SCOPE_MAX_RESOURCES];
    int resource_count;
    int scope_id;
} rp_scope_t;

typedef struct {
    rp_scope_t scopes[RP_SCOPE_MAX_DEPTH];
    int depth;
    int next_scope_id;
    int total_resources_guarded;
    int total_scopes_exited;
} rp_scope_guard_t;

void rp_scope_guard_init(rp_scope_guard_t *g) {
    g->depth = 0;
    g->next_scope_id = 1;
    g->total_resources_guarded = 0;
    g->total_scopes_exited = 0;
}

int rp_scope_guard_enter(rp_scope_guard_t *g) {
    if (g->depth >= RP_SCOPE_MAX_DEPTH) return -1;
    int idx = g->depth;
    g->scopes[idx].resource_count = 0;
    g->scopes[idx].scope_id = g->next_scope_id++;
    g->depth++;
    return g->scopes[idx].scope_id;
}

int rp_scope_guard_register(rp_scope_guard_t *g, int resource_id) {
    if (g->depth <= 0) return -1;
    int scope_idx = g->depth - 1;
    if (g->scopes[scope_idx].resource_count >= RP_SCOPE_MAX_RESOURCES) return -2;
    int res_idx = g->scopes[scope_idx].resource_count;
    g->scopes[scope_idx].resource_ids[res_idx] = resource_id;
    g->scopes[scope_idx].resource_count++;
    g->total_resources_guarded++;
    return 0;
}

int rp_scope_guard_exit(rp_scope_guard_t *g) {
    if (g->depth <= 0) return -1;
    g->depth--;
    int released = g->scopes[g->depth].resource_count;
    g->scopes[g->depth].resource_count = 0;
    g->total_scopes_exited++;
    return released;
}

int rp_scope_guard_depth(rp_scope_guard_t *g) {
    return g->depth;
}

int rp_scope_guard_current_resource_count(rp_scope_guard_t *g) {
    if (g->depth <= 0) return 0;
    return g->scopes[g->depth - 1].resource_count;
}
"##;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1950: Resource scope guard should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1950: Output should not be empty");
    assert!(code.contains("fn rp_"), "C1950: Should contain rp_ functions");
}
