//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1801-C1825: Timer & Clock Management Patterns -- countdown timers,
//! timer wheels, clock synchronization, timeout handling, and timer
//! applications commonly found in embedded systems, networking stacks,
//! and real-time schedulers.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! Organization:
//! - C1801-C1805: Basic timers (countdown, interval, one-shot, elapsed time, timestamp)
//! - C1806-C1810: Timer wheels (hierarchical timers, tick-based scheduling, timer buckets, cascading, overflow)
//! - C1811-C1815: Clock management (monotonic clock, wall clock, clock skew, clock sync, clock domain)
//! - C1816-C1820: Timeout handling (deadline tracking, expiry check, retry timers, backoff, timeout queue)
//! - C1821-C1825: Timer applications (debounce, rate limiting, heartbeat, watchdog, profiling timer)

// ============================================================================
// C1801-C1805: Basic Timers
// ============================================================================

/// C1801: Countdown timer with start, tick, and expiry detection
#[test]
fn c1801_countdown_timer() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define TMR_MAX_TIMERS 32
#define TMR_STATE_IDLE    0
#define TMR_STATE_RUNNING 1
#define TMR_STATE_EXPIRED 2

typedef struct {
    uint32_t duration;
    uint32_t remaining;
    int state;
    int auto_reload;
} tmr_countdown_t;

void tmr_countdown_init(tmr_countdown_t *t, uint32_t duration, int auto_reload) {
    t->duration = duration;
    t->remaining = 0;
    t->state = TMR_STATE_IDLE;
    t->auto_reload = auto_reload;
}

void tmr_countdown_start(tmr_countdown_t *t) {
    t->remaining = t->duration;
    t->state = TMR_STATE_RUNNING;
}

int tmr_countdown_tick(tmr_countdown_t *t) {
    if (t->state != TMR_STATE_RUNNING) return 0;
    if (t->remaining > 0) {
        t->remaining--;
    }
    if (t->remaining == 0) {
        if (t->auto_reload) {
            t->remaining = t->duration;
        } else {
            t->state = TMR_STATE_EXPIRED;
        }
        return 1;
    }
    return 0;
}

int tmr_countdown_is_expired(tmr_countdown_t *t) {
    return t->state == TMR_STATE_EXPIRED;
}

int tmr_countdown_test(void) {
    tmr_countdown_t t;
    tmr_countdown_init(&t, 3, 0);
    tmr_countdown_start(&t);
    tmr_countdown_tick(&t);
    tmr_countdown_tick(&t);
    int fired = tmr_countdown_tick(&t);
    if (!fired) return -1;
    if (!tmr_countdown_is_expired(&t)) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1801: Countdown timer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1801: Output should not be empty");
    assert!(code.contains("fn tmr_countdown_init"), "C1801: Should contain tmr_countdown_init");
    assert!(code.contains("fn tmr_countdown_tick"), "C1801: Should contain tmr_countdown_tick");
}

/// C1802: Interval timer with periodic callback tracking
#[test]
fn c1802_interval_timer() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define TMR_MAX_INTERVALS 16

typedef struct {
    uint32_t period;
    uint32_t elapsed;
    uint32_t fire_count;
    int active;
    int id;
} tmr_interval_entry_t;

typedef struct {
    tmr_interval_entry_t timers[TMR_MAX_INTERVALS];
    int count;
    uint32_t global_tick;
} tmr_interval_mgr_t;

void tmr_interval_mgr_init(tmr_interval_mgr_t *mgr) {
    mgr->count = 0;
    mgr->global_tick = 0;
}

int tmr_interval_add(tmr_interval_mgr_t *mgr, int id, uint32_t period) {
    if (mgr->count >= TMR_MAX_INTERVALS) return -1;
    int idx = mgr->count;
    mgr->timers[idx].id = id;
    mgr->timers[idx].period = period;
    mgr->timers[idx].elapsed = 0;
    mgr->timers[idx].fire_count = 0;
    mgr->timers[idx].active = 1;
    mgr->count++;
    return idx;
}

int tmr_interval_tick_all(tmr_interval_mgr_t *mgr) {
    int fired = 0;
    int i;
    mgr->global_tick++;
    for (i = 0; i < mgr->count; i++) {
        if (!mgr->timers[i].active) continue;
        mgr->timers[i].elapsed++;
        if (mgr->timers[i].elapsed >= mgr->timers[i].period) {
            mgr->timers[i].elapsed = 0;
            mgr->timers[i].fire_count++;
            fired++;
        }
    }
    return fired;
}

int tmr_interval_test(void) {
    tmr_interval_mgr_t mgr;
    tmr_interval_mgr_init(&mgr);
    tmr_interval_add(&mgr, 1, 5);
    tmr_interval_add(&mgr, 2, 3);
    int total_fired = 0;
    int t;
    for (t = 0; t < 15; t++) {
        total_fired += tmr_interval_tick_all(&mgr);
    }
    if (total_fired < 5) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1802: Interval timer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1802: Output should not be empty");
    assert!(code.contains("fn tmr_interval_mgr_init"), "C1802: Should contain tmr_interval_mgr_init");
    assert!(code.contains("fn tmr_interval_tick_all"), "C1802: Should contain tmr_interval_tick_all");
}

/// C1803: One-shot timer with arm and disarm
#[test]
fn c1803_oneshot_timer() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define TMR_OS_DISARMED 0
#define TMR_OS_ARMED    1
#define TMR_OS_FIRED    2

typedef struct {
    uint32_t deadline;
    int state;
    int tag;
} tmr_oneshot_t;

typedef struct {
    tmr_oneshot_t slots[8];
    int count;
    uint32_t now;
} tmr_oneshot_pool_t;

void tmr_oneshot_pool_init(tmr_oneshot_pool_t *pool) {
    pool->count = 0;
    pool->now = 0;
}

int tmr_oneshot_arm(tmr_oneshot_pool_t *pool, int tag, uint32_t delay) {
    if (pool->count >= 8) return -1;
    int idx = pool->count;
    pool->slots[idx].deadline = pool->now + delay;
    pool->slots[idx].state = TMR_OS_ARMED;
    pool->slots[idx].tag = tag;
    pool->count++;
    return idx;
}

int tmr_oneshot_disarm(tmr_oneshot_pool_t *pool, int idx) {
    if (idx < 0 || idx >= pool->count) return -1;
    if (pool->slots[idx].state != TMR_OS_ARMED) return -2;
    pool->slots[idx].state = TMR_OS_DISARMED;
    return 0;
}

int tmr_oneshot_advance(tmr_oneshot_pool_t *pool, uint32_t ticks) {
    pool->now += ticks;
    int fired = 0;
    int i;
    for (i = 0; i < pool->count; i++) {
        if (pool->slots[i].state == TMR_OS_ARMED && pool->now >= pool->slots[i].deadline) {
            pool->slots[i].state = TMR_OS_FIRED;
            fired++;
        }
    }
    return fired;
}

int tmr_oneshot_test(void) {
    tmr_oneshot_pool_t pool;
    tmr_oneshot_pool_init(&pool);
    tmr_oneshot_arm(&pool, 10, 5);
    tmr_oneshot_arm(&pool, 20, 10);
    int f1 = tmr_oneshot_advance(&pool, 6);
    if (f1 != 1) return -1;
    int f2 = tmr_oneshot_advance(&pool, 5);
    if (f2 != 1) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1803: One-shot timer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1803: Output should not be empty");
    assert!(code.contains("fn tmr_oneshot_arm"), "C1803: Should contain tmr_oneshot_arm");
    assert!(code.contains("fn tmr_oneshot_advance"), "C1803: Should contain tmr_oneshot_advance");
}

/// C1804: Elapsed time tracker with lap and split support
#[test]
fn c1804_elapsed_time_tracker() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define TMR_MAX_LAPS 32

typedef struct {
    uint32_t start_tick;
    uint32_t current_tick;
    uint32_t laps[TMR_MAX_LAPS];
    int lap_count;
    int running;
} tmr_elapsed_t;

void tmr_elapsed_init(tmr_elapsed_t *e) {
    e->start_tick = 0;
    e->current_tick = 0;
    e->lap_count = 0;
    e->running = 0;
}

void tmr_elapsed_start(tmr_elapsed_t *e, uint32_t tick) {
    e->start_tick = tick;
    e->current_tick = tick;
    e->running = 1;
    e->lap_count = 0;
}

uint32_t tmr_elapsed_update(tmr_elapsed_t *e, uint32_t tick) {
    if (!e->running) return 0;
    e->current_tick = tick;
    return tick - e->start_tick;
}

int tmr_elapsed_lap(tmr_elapsed_t *e) {
    if (!e->running || e->lap_count >= TMR_MAX_LAPS) return -1;
    uint32_t elapsed = e->current_tick - e->start_tick;
    e->laps[e->lap_count] = elapsed;
    e->lap_count++;
    return e->lap_count;
}

uint32_t tmr_elapsed_get_lap(tmr_elapsed_t *e, int idx) {
    if (idx < 0 || idx >= e->lap_count) return 0;
    if (idx == 0) return e->laps[0];
    return e->laps[idx] - e->laps[idx - 1];
}

int tmr_elapsed_test(void) {
    tmr_elapsed_t e;
    tmr_elapsed_init(&e);
    tmr_elapsed_start(&e, 100);
    tmr_elapsed_update(&e, 150);
    tmr_elapsed_lap(&e);
    tmr_elapsed_update(&e, 200);
    tmr_elapsed_lap(&e);
    uint32_t split = tmr_elapsed_get_lap(&e, 1);
    if (split != 50) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1804: Elapsed time tracker should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1804: Output should not be empty");
    assert!(code.contains("fn tmr_elapsed_init"), "C1804: Should contain tmr_elapsed_init");
    assert!(code.contains("fn tmr_elapsed_lap"), "C1804: Should contain tmr_elapsed_lap");
}

/// C1805: Timestamp generator with monotonic counter and formatting
#[test]
fn c1805_timestamp_generator() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

typedef struct {
    uint64_t ticks;
    uint32_t ticks_per_sec;
    uint32_t epoch_offset;
} tmr_timestamp_t;

void tmr_timestamp_init(tmr_timestamp_t *ts, uint32_t tps, uint32_t epoch) {
    ts->ticks = 0;
    ts->ticks_per_sec = tps;
    ts->epoch_offset = epoch;
}

void tmr_timestamp_advance(tmr_timestamp_t *ts, uint32_t delta) {
    ts->ticks += delta;
}

uint32_t tmr_timestamp_seconds(tmr_timestamp_t *ts) {
    if (ts->ticks_per_sec == 0) return 0;
    return (uint32_t)(ts->ticks / ts->ticks_per_sec) + ts->epoch_offset;
}

uint32_t tmr_timestamp_millis(tmr_timestamp_t *ts) {
    if (ts->ticks_per_sec == 0) return 0;
    uint64_t ms = (ts->ticks * 1000) / ts->ticks_per_sec;
    return (uint32_t)(ms % 1000);
}

int tmr_timestamp_compare(tmr_timestamp_t *a, tmr_timestamp_t *b) {
    if (a->ticks < b->ticks) return -1;
    if (a->ticks > b->ticks) return 1;
    return 0;
}

int tmr_timestamp_test(void) {
    tmr_timestamp_t ts;
    tmr_timestamp_init(&ts, 1000, 0);
    tmr_timestamp_advance(&ts, 2500);
    uint32_t sec = tmr_timestamp_seconds(&ts);
    uint32_t ms = tmr_timestamp_millis(&ts);
    if (sec != 2) return -1;
    if (ms != 500) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1805: Timestamp generator should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1805: Output should not be empty");
    assert!(code.contains("fn tmr_timestamp_init"), "C1805: Should contain tmr_timestamp_init");
    assert!(code.contains("fn tmr_timestamp_seconds"), "C1805: Should contain tmr_timestamp_seconds");
}

// ============================================================================
// C1806-C1810: Timer Wheels
// ============================================================================

/// C1806: Hierarchical timer wheel with multi-level buckets
#[test]
fn c1806_hierarchical_timer_wheel() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define TMR_WHEEL_SLOTS 64
#define TMR_WHEEL_LEVELS 3
#define TMR_MAX_ENTRIES 128

typedef struct {
    uint32_t expiry;
    int tag;
    int active;
} tmr_wheel_entry_t;

typedef struct {
    int entries[TMR_WHEEL_SLOTS];
    int entry_count[TMR_WHEEL_SLOTS];
    uint32_t current_slot;
} tmr_wheel_level_t;

typedef struct {
    tmr_wheel_entry_t pool[TMR_MAX_ENTRIES];
    tmr_wheel_level_t levels[TMR_WHEEL_LEVELS];
    int pool_used;
    uint32_t current_tick;
} tmr_hwheel_t;

void tmr_hwheel_init(tmr_hwheel_t *hw) {
    hw->pool_used = 0;
    hw->current_tick = 0;
    int l;
    for (l = 0; l < TMR_WHEEL_LEVELS; l++) {
        hw->levels[l].current_slot = 0;
        int s;
        for (s = 0; s < TMR_WHEEL_SLOTS; s++) {
            hw->levels[l].entries[s] = -1;
            hw->levels[l].entry_count[s] = 0;
        }
    }
}

int tmr_hwheel_schedule(tmr_hwheel_t *hw, int tag, uint32_t delay) {
    if (hw->pool_used >= TMR_MAX_ENTRIES) return -1;
    int idx = hw->pool_used++;
    hw->pool[idx].expiry = hw->current_tick + delay;
    hw->pool[idx].tag = tag;
    hw->pool[idx].active = 1;
    uint32_t slot = (hw->current_tick + delay) % TMR_WHEEL_SLOTS;
    int level = 0;
    if (delay >= TMR_WHEEL_SLOTS * TMR_WHEEL_SLOTS) level = 2;
    else if (delay >= TMR_WHEEL_SLOTS) level = 1;
    hw->levels[level].entries[slot] = idx;
    hw->levels[level].entry_count[slot]++;
    return idx;
}

int tmr_hwheel_advance(tmr_hwheel_t *hw) {
    hw->current_tick++;
    int fired = 0;
    int i;
    for (i = 0; i < hw->pool_used; i++) {
        if (hw->pool[i].active && hw->current_tick >= hw->pool[i].expiry) {
            hw->pool[i].active = 0;
            fired++;
        }
    }
    return fired;
}

int tmr_hwheel_test(void) {
    tmr_hwheel_t hw;
    tmr_hwheel_init(&hw);
    tmr_hwheel_schedule(&hw, 1, 3);
    tmr_hwheel_schedule(&hw, 2, 100);
    int f = 0;
    int t;
    for (t = 0; t < 5; t++) f += tmr_hwheel_advance(&hw);
    if (f != 1) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1806: Hierarchical timer wheel should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1806: Output should not be empty");
    assert!(code.contains("fn tmr_hwheel_init"), "C1806: Should contain tmr_hwheel_init");
    assert!(code.contains("fn tmr_hwheel_advance"), "C1806: Should contain tmr_hwheel_advance");
}

/// C1807: Tick-based scheduler with priority ordering
#[test]
fn c1807_tick_scheduler() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define TMR_SCHED_MAX 32

typedef struct {
    uint32_t next_fire;
    uint32_t interval;
    int priority;
    int active;
    int fire_count;
} tmr_tick_entry_t;

typedef struct {
    tmr_tick_entry_t entries[TMR_SCHED_MAX];
    int count;
    uint32_t tick;
} tmr_tick_sched_t;

void tmr_tick_sched_init(tmr_tick_sched_t *s) {
    s->count = 0;
    s->tick = 0;
}

int tmr_tick_sched_add(tmr_tick_sched_t *s, uint32_t interval, int priority) {
    if (s->count >= TMR_SCHED_MAX) return -1;
    int idx = s->count;
    s->entries[idx].next_fire = s->tick + interval;
    s->entries[idx].interval = interval;
    s->entries[idx].priority = priority;
    s->entries[idx].active = 1;
    s->entries[idx].fire_count = 0;
    s->count++;
    return idx;
}

int tmr_tick_sched_step(tmr_tick_sched_t *s) {
    s->tick++;
    int best = -1;
    int best_prio = -1;
    int i;
    for (i = 0; i < s->count; i++) {
        if (!s->entries[i].active) continue;
        if (s->tick >= s->entries[i].next_fire) {
            if (s->entries[i].priority > best_prio) {
                best_prio = s->entries[i].priority;
                best = i;
            }
        }
    }
    if (best >= 0) {
        s->entries[best].fire_count++;
        s->entries[best].next_fire = s->tick + s->entries[best].interval;
        return best;
    }
    return -1;
}

int tmr_tick_sched_test(void) {
    tmr_tick_sched_t s;
    tmr_tick_sched_init(&s);
    tmr_tick_sched_add(&s, 2, 10);
    tmr_tick_sched_add(&s, 3, 20);
    int fired_id = -1;
    int t;
    for (t = 0; t < 6; t++) {
        int r = tmr_tick_sched_step(&s);
        if (r >= 0) fired_id = r;
    }
    if (fired_id < 0) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1807: Tick scheduler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1807: Output should not be empty");
    assert!(code.contains("fn tmr_tick_sched_init"), "C1807: Should contain tmr_tick_sched_init");
    assert!(code.contains("fn tmr_tick_sched_step"), "C1807: Should contain tmr_tick_sched_step");
}

/// C1808: Timer bucket system with hash-based assignment
#[test]
fn c1808_timer_buckets() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define TMR_NUM_BUCKETS 16
#define TMR_BUCKET_CAP 8

typedef struct {
    int tags[TMR_BUCKET_CAP];
    uint32_t deadlines[TMR_BUCKET_CAP];
    int count;
} tmr_bucket_t;

typedef struct {
    tmr_bucket_t buckets[TMR_NUM_BUCKETS];
    uint32_t now;
    int total_timers;
} tmr_bucket_set_t;

void tmr_bucket_set_init(tmr_bucket_set_t *bs) {
    bs->now = 0;
    bs->total_timers = 0;
    int i;
    for (i = 0; i < TMR_NUM_BUCKETS; i++) {
        bs->buckets[i].count = 0;
    }
}

int tmr_bucket_insert(tmr_bucket_set_t *bs, int tag, uint32_t deadline) {
    uint32_t idx = deadline % TMR_NUM_BUCKETS;
    tmr_bucket_t *b = &bs->buckets[idx];
    if (b->count >= TMR_BUCKET_CAP) return -1;
    b->tags[b->count] = tag;
    b->deadlines[b->count] = deadline;
    b->count++;
    bs->total_timers++;
    return 0;
}

int tmr_bucket_collect_expired(tmr_bucket_set_t *bs, int *out_tags, int max_out) {
    int collected = 0;
    uint32_t idx = bs->now % TMR_NUM_BUCKETS;
    tmr_bucket_t *b = &bs->buckets[idx];
    int i;
    for (i = 0; i < b->count && collected < max_out; i++) {
        if (b->deadlines[i] <= bs->now) {
            out_tags[collected++] = b->tags[i];
        }
    }
    return collected;
}

int tmr_bucket_test(void) {
    tmr_bucket_set_t bs;
    tmr_bucket_set_init(&bs);
    tmr_bucket_insert(&bs, 100, 5);
    tmr_bucket_insert(&bs, 200, 21);
    bs.now = 5;
    int tags[4];
    int n = tmr_bucket_collect_expired(&bs, tags, 4);
    if (n < 1) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1808: Timer buckets should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1808: Output should not be empty");
    assert!(code.contains("fn tmr_bucket_set_init"), "C1808: Should contain tmr_bucket_set_init");
    assert!(code.contains("fn tmr_bucket_insert"), "C1808: Should contain tmr_bucket_insert");
}

/// C1809: Cascading timer with overflow from fine to coarse granularity
#[test]
fn c1809_cascading_timer() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define TMR_CASCADE_FINE   256
#define TMR_CASCADE_COARSE 64

typedef struct {
    uint32_t fine_counts[TMR_CASCADE_FINE];
    uint32_t coarse_counts[TMR_CASCADE_COARSE];
    uint32_t fine_pos;
    uint32_t coarse_pos;
    uint32_t overflow_count;
    uint32_t total_ticks;
} tmr_cascade_t;

void tmr_cascade_init(tmr_cascade_t *c) {
    c->fine_pos = 0;
    c->coarse_pos = 0;
    c->overflow_count = 0;
    c->total_ticks = 0;
    int i;
    for (i = 0; i < TMR_CASCADE_FINE; i++) c->fine_counts[i] = 0;
    for (i = 0; i < TMR_CASCADE_COARSE; i++) c->coarse_counts[i] = 0;
}

int tmr_cascade_tick(tmr_cascade_t *c) {
    c->total_ticks++;
    c->fine_pos++;
    int cascaded = 0;
    if (c->fine_pos >= TMR_CASCADE_FINE) {
        c->fine_pos = 0;
        c->coarse_pos++;
        cascaded = 1;
        if (c->coarse_pos >= TMR_CASCADE_COARSE) {
            c->coarse_pos = 0;
            c->overflow_count++;
        }
    }
    c->fine_counts[c->fine_pos]++;
    if (cascaded) {
        c->coarse_counts[c->coarse_pos]++;
    }
    return cascaded;
}

uint32_t tmr_cascade_read(tmr_cascade_t *c) {
    return c->overflow_count * TMR_CASCADE_FINE * TMR_CASCADE_COARSE
         + c->coarse_pos * TMR_CASCADE_FINE
         + c->fine_pos;
}

int tmr_cascade_test(void) {
    tmr_cascade_t c;
    tmr_cascade_init(&c);
    int i;
    for (i = 0; i < 300; i++) tmr_cascade_tick(&c);
    uint32_t val = tmr_cascade_read(&c);
    if (val != 300) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1809: Cascading timer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1809: Output should not be empty");
    assert!(code.contains("fn tmr_cascade_init"), "C1809: Should contain tmr_cascade_init");
    assert!(code.contains("fn tmr_cascade_tick"), "C1809: Should contain tmr_cascade_tick");
}

/// C1810: Timer overflow detection and wraparound handling
#[test]
fn c1810_timer_overflow() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define TMR_WRAP_MAX 0xFFFFFFFFu

typedef struct {
    uint32_t counter;
    uint32_t wrap_count;
    uint32_t last_value;
    int wrapped;
} tmr_overflow_t;

void tmr_overflow_init(tmr_overflow_t *o) {
    o->counter = 0;
    o->wrap_count = 0;
    o->last_value = 0;
    o->wrapped = 0;
}

void tmr_overflow_set(tmr_overflow_t *o, uint32_t val) {
    o->last_value = o->counter;
    o->counter = val;
    if (val < o->last_value) {
        o->wrap_count++;
        o->wrapped = 1;
    } else {
        o->wrapped = 0;
    }
}

uint32_t tmr_overflow_delta(tmr_overflow_t *o) {
    if (o->counter >= o->last_value) {
        return o->counter - o->last_value;
    }
    return (TMR_WRAP_MAX - o->last_value) + o->counter + 1;
}

int tmr_overflow_has_wrapped(tmr_overflow_t *o) {
    return o->wrapped;
}

int tmr_overflow_safe_compare(uint32_t a, uint32_t b) {
    int diff = (int)(a - b);
    if (diff > 0) return 1;
    if (diff < 0) return -1;
    return 0;
}

int tmr_overflow_test(void) {
    tmr_overflow_t o;
    tmr_overflow_init(&o);
    tmr_overflow_set(&o, 100);
    tmr_overflow_set(&o, 50);
    if (!tmr_overflow_has_wrapped(&o)) return -1;
    uint32_t d = tmr_overflow_delta(&o);
    if (d == 0) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1810: Timer overflow should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1810: Output should not be empty");
    assert!(code.contains("fn tmr_overflow_init"), "C1810: Should contain tmr_overflow_init");
    assert!(code.contains("fn tmr_overflow_delta"), "C1810: Should contain tmr_overflow_delta");
}

// ============================================================================
// C1811-C1815: Clock Management
// ============================================================================

/// C1811: Monotonic clock with guaranteed forward progress
#[test]
fn c1811_monotonic_clock() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

typedef struct {
    uint64_t ticks;
    uint32_t frequency;
    uint64_t last_read;
    int initialized;
} tmr_mono_clock_t;

void tmr_mono_init(tmr_mono_clock_t *clk, uint32_t freq) {
    clk->ticks = 0;
    clk->frequency = freq;
    clk->last_read = 0;
    clk->initialized = 1;
}

void tmr_mono_feed(tmr_mono_clock_t *clk, uint32_t hw_ticks) {
    uint64_t new_val = clk->ticks + hw_ticks;
    if (new_val > clk->ticks) {
        clk->ticks = new_val;
    }
}

uint64_t tmr_mono_read(tmr_mono_clock_t *clk) {
    if (clk->ticks < clk->last_read) {
        return clk->last_read;
    }
    clk->last_read = clk->ticks;
    return clk->ticks;
}

uint32_t tmr_mono_elapsed_ms(tmr_mono_clock_t *clk, uint64_t start) {
    uint64_t now = tmr_mono_read(clk);
    if (now <= start) return 0;
    uint64_t delta = now - start;
    if (clk->frequency == 0) return 0;
    return (uint32_t)((delta * 1000) / clk->frequency);
}

int tmr_mono_test(void) {
    tmr_mono_clock_t clk;
    tmr_mono_init(&clk, 1000000);
    uint64_t t0 = tmr_mono_read(&clk);
    tmr_mono_feed(&clk, 500000);
    uint32_t ms = tmr_mono_elapsed_ms(&clk, t0);
    if (ms != 500) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1811: Monotonic clock should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1811: Output should not be empty");
    assert!(code.contains("fn tmr_mono_init"), "C1811: Should contain tmr_mono_init");
    assert!(code.contains("fn tmr_mono_read"), "C1811: Should contain tmr_mono_read");
}

/// C1812: Wall clock with hours/minutes/seconds decomposition
#[test]
fn c1812_wall_clock() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

typedef struct {
    uint32_t total_seconds;
    int hours;
    int minutes;
    int seconds;
    int day_of_week;
} tmr_wall_clock_t;

void tmr_wall_init(tmr_wall_clock_t *w, uint32_t initial_secs) {
    w->total_seconds = initial_secs;
    w->seconds = initial_secs % 60;
    w->minutes = (initial_secs / 60) % 60;
    w->hours = (initial_secs / 3600) % 24;
    w->day_of_week = (initial_secs / 86400) % 7;
}

void tmr_wall_advance(tmr_wall_clock_t *w, uint32_t secs) {
    w->total_seconds += secs;
    w->seconds = w->total_seconds % 60;
    w->minutes = (w->total_seconds / 60) % 60;
    w->hours = (w->total_seconds / 3600) % 24;
    w->day_of_week = (w->total_seconds / 86400) % 7;
}

int tmr_wall_is_midnight(tmr_wall_clock_t *w) {
    return (w->hours == 0 && w->minutes == 0 && w->seconds == 0);
}

uint32_t tmr_wall_until_midnight(tmr_wall_clock_t *w) {
    uint32_t day_secs = w->hours * 3600 + w->minutes * 60 + w->seconds;
    return 86400 - day_secs;
}

int tmr_wall_test(void) {
    tmr_wall_clock_t w;
    tmr_wall_init(&w, 3661);
    if (w.hours != 1) return -1;
    if (w.minutes != 1) return -2;
    if (w.seconds != 1) return -3;
    tmr_wall_advance(&w, 59);
    if (w.minutes != 2) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1812: Wall clock should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1812: Output should not be empty");
    assert!(code.contains("fn tmr_wall_init"), "C1812: Should contain tmr_wall_init");
    assert!(code.contains("fn tmr_wall_advance"), "C1812: Should contain tmr_wall_advance");
}

/// C1813: Clock skew detection between two clock sources
#[test]
fn c1813_clock_skew() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef long int32_t;

#define TMR_SKEW_HISTORY 16

typedef struct {
    uint32_t ref_readings[TMR_SKEW_HISTORY];
    uint32_t local_readings[TMR_SKEW_HISTORY];
    int32_t skew_samples[TMR_SKEW_HISTORY];
    int sample_count;
    int head;
    int32_t avg_skew;
} tmr_skew_tracker_t;

void tmr_skew_init(tmr_skew_tracker_t *sk) {
    sk->sample_count = 0;
    sk->head = 0;
    sk->avg_skew = 0;
}

void tmr_skew_record(tmr_skew_tracker_t *sk, uint32_t ref_time, uint32_t local_time) {
    int idx = sk->head % TMR_SKEW_HISTORY;
    sk->ref_readings[idx] = ref_time;
    sk->local_readings[idx] = local_time;
    sk->skew_samples[idx] = (int32_t)(local_time - ref_time);
    sk->head++;
    if (sk->sample_count < TMR_SKEW_HISTORY) {
        sk->sample_count++;
    }
}

int32_t tmr_skew_compute_avg(tmr_skew_tracker_t *sk) {
    if (sk->sample_count == 0) return 0;
    int32_t sum = 0;
    int i;
    for (i = 0; i < sk->sample_count; i++) {
        int idx = (sk->head - sk->sample_count + i) % TMR_SKEW_HISTORY;
        if (idx < 0) idx += TMR_SKEW_HISTORY;
        sum += sk->skew_samples[idx];
    }
    sk->avg_skew = sum / sk->sample_count;
    return sk->avg_skew;
}

int tmr_skew_exceeds(tmr_skew_tracker_t *sk, int32_t threshold) {
    int32_t avg = tmr_skew_compute_avg(sk);
    if (avg > threshold || avg < -threshold) return 1;
    return 0;
}

int tmr_skew_test(void) {
    tmr_skew_tracker_t sk;
    tmr_skew_init(&sk);
    tmr_skew_record(&sk, 100, 103);
    tmr_skew_record(&sk, 200, 205);
    tmr_skew_record(&sk, 300, 307);
    int32_t avg = tmr_skew_compute_avg(&sk);
    if (avg < 3 || avg > 7) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1813: Clock skew should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1813: Output should not be empty");
    assert!(code.contains("fn tmr_skew_init"), "C1813: Should contain tmr_skew_init");
    assert!(code.contains("fn tmr_skew_compute_avg"), "C1813: Should contain tmr_skew_compute_avg");
}

/// C1814: Clock synchronization with NTP-style offset correction
#[test]
fn c1814_clock_sync() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef long int32_t;

#define TMR_SYNC_MAX_PEERS 8

typedef struct {
    uint32_t local_time;
    int32_t offset;
    uint32_t roundtrip;
    int valid;
} tmr_sync_peer_t;

typedef struct {
    tmr_sync_peer_t peers[TMR_SYNC_MAX_PEERS];
    int peer_count;
    uint32_t local_clock;
    int32_t correction;
    int synced;
} tmr_sync_state_t;

void tmr_sync_init(tmr_sync_state_t *ss) {
    ss->peer_count = 0;
    ss->local_clock = 0;
    ss->correction = 0;
    ss->synced = 0;
}

int tmr_sync_add_peer(tmr_sync_state_t *ss, uint32_t local, uint32_t remote, uint32_t rtt) {
    if (ss->peer_count >= TMR_SYNC_MAX_PEERS) return -1;
    int idx = ss->peer_count;
    ss->peers[idx].local_time = local;
    ss->peers[idx].offset = (int32_t)(remote - local) + (int32_t)(rtt / 2);
    ss->peers[idx].roundtrip = rtt;
    ss->peers[idx].valid = 1;
    ss->peer_count++;
    return idx;
}

int32_t tmr_sync_compute_offset(tmr_sync_state_t *ss) {
    if (ss->peer_count == 0) return 0;
    int32_t total_offset = 0;
    uint32_t min_rtt = 0xFFFFFFFF;
    int best_peer = 0;
    int i;
    for (i = 0; i < ss->peer_count; i++) {
        if (!ss->peers[i].valid) continue;
        total_offset += ss->peers[i].offset;
        if (ss->peers[i].roundtrip < min_rtt) {
            min_rtt = ss->peers[i].roundtrip;
            best_peer = i;
        }
    }
    ss->correction = ss->peers[best_peer].offset;
    ss->synced = 1;
    return ss->correction;
}

uint32_t tmr_sync_corrected_time(tmr_sync_state_t *ss) {
    return (uint32_t)((int32_t)ss->local_clock + ss->correction);
}

int tmr_sync_test(void) {
    tmr_sync_state_t ss;
    tmr_sync_init(&ss);
    tmr_sync_add_peer(&ss, 1000, 1010, 4);
    tmr_sync_add_peer(&ss, 1000, 1012, 8);
    int32_t off = tmr_sync_compute_offset(&ss);
    if (off == 0) return -1;
    if (!ss.synced) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1814: Clock sync should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1814: Output should not be empty");
    assert!(code.contains("fn tmr_sync_init"), "C1814: Should contain tmr_sync_init");
    assert!(code.contains("fn tmr_sync_compute_offset"), "C1814: Should contain tmr_sync_compute_offset");
}

/// C1815: Clock domain crossing with rate conversion
#[test]
fn c1815_clock_domain() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

typedef struct {
    uint32_t src_freq;
    uint32_t dst_freq;
    uint64_t src_ticks;
    uint64_t dst_ticks;
    uint64_t remainder;
} tmr_domain_xing_t;

void tmr_domain_init(tmr_domain_xing_t *dx, uint32_t src_freq, uint32_t dst_freq) {
    dx->src_freq = src_freq;
    dx->dst_freq = dst_freq;
    dx->src_ticks = 0;
    dx->dst_ticks = 0;
    dx->remainder = 0;
}

uint64_t tmr_domain_convert(tmr_domain_xing_t *dx, uint32_t src_delta) {
    if (dx->src_freq == 0) return 0;
    dx->src_ticks += src_delta;
    uint64_t total = (uint64_t)src_delta * dx->dst_freq + dx->remainder;
    uint64_t converted = total / dx->src_freq;
    dx->remainder = total % dx->src_freq;
    dx->dst_ticks += converted;
    return converted;
}

uint32_t tmr_domain_src_to_ms(tmr_domain_xing_t *dx, uint64_t src_val) {
    if (dx->src_freq == 0) return 0;
    return (uint32_t)((src_val * 1000) / dx->src_freq);
}

uint32_t tmr_domain_dst_to_ms(tmr_domain_xing_t *dx, uint64_t dst_val) {
    if (dx->dst_freq == 0) return 0;
    return (uint32_t)((dst_val * 1000) / dx->dst_freq);
}

int tmr_domain_test(void) {
    tmr_domain_xing_t dx;
    tmr_domain_init(&dx, 48000, 44100);
    uint64_t out = tmr_domain_convert(&dx, 48000);
    if (out != 44100) return -1;
    uint32_t ms_src = tmr_domain_src_to_ms(&dx, 48000);
    if (ms_src != 1000) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1815: Clock domain should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1815: Output should not be empty");
    assert!(code.contains("fn tmr_domain_init"), "C1815: Should contain tmr_domain_init");
    assert!(code.contains("fn tmr_domain_convert"), "C1815: Should contain tmr_domain_convert");
}

// ============================================================================
// C1816-C1820: Timeout Handling
// ============================================================================

/// C1816: Deadline tracker with absolute time comparison
#[test]
fn c1816_deadline_tracker() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define TMR_MAX_DEADLINES 16
#define TMR_DL_PENDING  0
#define TMR_DL_MET      1
#define TMR_DL_MISSED   2

typedef struct {
    uint32_t deadline;
    uint32_t created_at;
    int state;
    int task_id;
} tmr_deadline_entry_t;

typedef struct {
    tmr_deadline_entry_t entries[TMR_MAX_DEADLINES];
    int count;
    uint32_t now;
    int missed_count;
    int met_count;
} tmr_deadline_mgr_t;

void tmr_deadline_init(tmr_deadline_mgr_t *dm) {
    dm->count = 0;
    dm->now = 0;
    dm->missed_count = 0;
    dm->met_count = 0;
}

int tmr_deadline_add(tmr_deadline_mgr_t *dm, int task_id, uint32_t abs_deadline) {
    if (dm->count >= TMR_MAX_DEADLINES) return -1;
    int idx = dm->count;
    dm->entries[idx].deadline = abs_deadline;
    dm->entries[idx].created_at = dm->now;
    dm->entries[idx].state = TMR_DL_PENDING;
    dm->entries[idx].task_id = task_id;
    dm->count++;
    return idx;
}

void tmr_deadline_complete(tmr_deadline_mgr_t *dm, int idx) {
    if (idx < 0 || idx >= dm->count) return;
    if (dm->entries[idx].state != TMR_DL_PENDING) return;
    if (dm->now <= dm->entries[idx].deadline) {
        dm->entries[idx].state = TMR_DL_MET;
        dm->met_count++;
    } else {
        dm->entries[idx].state = TMR_DL_MISSED;
        dm->missed_count++;
    }
}

int tmr_deadline_check_overdue(tmr_deadline_mgr_t *dm) {
    int overdue = 0;
    int i;
    for (i = 0; i < dm->count; i++) {
        if (dm->entries[i].state == TMR_DL_PENDING && dm->now > dm->entries[i].deadline) {
            dm->entries[i].state = TMR_DL_MISSED;
            dm->missed_count++;
            overdue++;
        }
    }
    return overdue;
}

int tmr_deadline_test(void) {
    tmr_deadline_mgr_t dm;
    tmr_deadline_init(&dm);
    int idx = tmr_deadline_add(&dm, 1, 100);
    dm.now = 80;
    tmr_deadline_complete(&dm, idx);
    if (dm.met_count != 1) return -1;
    tmr_deadline_add(&dm, 2, 90);
    dm.now = 110;
    int overdue = tmr_deadline_check_overdue(&dm);
    if (overdue != 1) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1816: Deadline tracker should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1816: Output should not be empty");
    assert!(code.contains("fn tmr_deadline_init"), "C1816: Should contain tmr_deadline_init");
    assert!(code.contains("fn tmr_deadline_check_overdue"), "C1816: Should contain tmr_deadline_check_overdue");
}

/// C1817: Expiry checker with soft and hard timeout thresholds
#[test]
fn c1817_expiry_checker() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define TMR_EXPIRY_ACTIVE  0
#define TMR_EXPIRY_SOFT    1
#define TMR_EXPIRY_HARD    2
#define TMR_EXPIRY_SLOTS   8

typedef struct {
    uint32_t start_time;
    uint32_t soft_timeout;
    uint32_t hard_timeout;
    int state;
    int id;
} tmr_expiry_slot_t;

typedef struct {
    tmr_expiry_slot_t slots[TMR_EXPIRY_SLOTS];
    int count;
    uint32_t now;
} tmr_expiry_checker_t;

void tmr_expiry_init(tmr_expiry_checker_t *ec) {
    ec->count = 0;
    ec->now = 0;
}

int tmr_expiry_register(tmr_expiry_checker_t *ec, int id, uint32_t soft_ms, uint32_t hard_ms) {
    if (ec->count >= TMR_EXPIRY_SLOTS) return -1;
    int idx = ec->count;
    ec->slots[idx].id = id;
    ec->slots[idx].start_time = ec->now;
    ec->slots[idx].soft_timeout = soft_ms;
    ec->slots[idx].hard_timeout = hard_ms;
    ec->slots[idx].state = TMR_EXPIRY_ACTIVE;
    ec->count++;
    return idx;
}

int tmr_expiry_check(tmr_expiry_checker_t *ec, int idx) {
    if (idx < 0 || idx >= ec->count) return -1;
    uint32_t elapsed = ec->now - ec->slots[idx].start_time;
    if (elapsed >= ec->slots[idx].hard_timeout) {
        ec->slots[idx].state = TMR_EXPIRY_HARD;
        return TMR_EXPIRY_HARD;
    }
    if (elapsed >= ec->slots[idx].soft_timeout) {
        ec->slots[idx].state = TMR_EXPIRY_SOFT;
        return TMR_EXPIRY_SOFT;
    }
    return TMR_EXPIRY_ACTIVE;
}

int tmr_expiry_count_expired(tmr_expiry_checker_t *ec) {
    int expired = 0;
    int i;
    for (i = 0; i < ec->count; i++) {
        tmr_expiry_check(ec, i);
        if (ec->slots[i].state != TMR_EXPIRY_ACTIVE) expired++;
    }
    return expired;
}

int tmr_expiry_test(void) {
    tmr_expiry_checker_t ec;
    tmr_expiry_init(&ec);
    tmr_expiry_register(&ec, 1, 50, 100);
    ec.now = 60;
    int state = tmr_expiry_check(&ec, 0);
    if (state != TMR_EXPIRY_SOFT) return -1;
    ec.now = 110;
    state = tmr_expiry_check(&ec, 0);
    if (state != TMR_EXPIRY_HARD) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1817: Expiry checker should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1817: Output should not be empty");
    assert!(code.contains("fn tmr_expiry_init"), "C1817: Should contain tmr_expiry_init");
    assert!(code.contains("fn tmr_expiry_check"), "C1817: Should contain tmr_expiry_check");
}

/// C1818: Retry timer with configurable attempt count and delay
#[test]
fn c1818_retry_timer() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define TMR_RETRY_MAX 8

typedef struct {
    uint32_t base_delay;
    uint32_t current_delay;
    uint32_t next_retry_at;
    int attempt;
    int max_attempts;
    int exhausted;
} tmr_retry_t;

void tmr_retry_init(tmr_retry_t *r, uint32_t base_delay, int max_attempts) {
    r->base_delay = base_delay;
    r->current_delay = base_delay;
    r->next_retry_at = 0;
    r->attempt = 0;
    r->max_attempts = max_attempts;
    r->exhausted = 0;
}

int tmr_retry_should_fire(tmr_retry_t *r, uint32_t now) {
    if (r->exhausted) return 0;
    if (now >= r->next_retry_at) return 1;
    return 0;
}

int tmr_retry_fire(tmr_retry_t *r, uint32_t now) {
    if (r->exhausted) return -1;
    r->attempt++;
    if (r->attempt >= r->max_attempts) {
        r->exhausted = 1;
        return -2;
    }
    r->current_delay = r->base_delay;
    r->next_retry_at = now + r->current_delay;
    return r->attempt;
}

void tmr_retry_reset(tmr_retry_t *r) {
    r->attempt = 0;
    r->current_delay = r->base_delay;
    r->next_retry_at = 0;
    r->exhausted = 0;
}

int tmr_retry_test(void) {
    tmr_retry_t r;
    tmr_retry_init(&r, 100, 3);
    int a1 = tmr_retry_fire(&r, 0);
    if (a1 != 1) return -1;
    int a2 = tmr_retry_fire(&r, 100);
    if (a2 != 2) return -2;
    int a3 = tmr_retry_fire(&r, 200);
    if (a3 != -2) return -3;
    if (!r.exhausted) return -4;
    tmr_retry_reset(&r);
    if (r.exhausted) return -5;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1818: Retry timer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1818: Output should not be empty");
    assert!(code.contains("fn tmr_retry_init"), "C1818: Should contain tmr_retry_init");
    assert!(code.contains("fn tmr_retry_fire"), "C1818: Should contain tmr_retry_fire");
}

/// C1819: Exponential backoff timer for network retry scenarios
#[test]
fn c1819_backoff_timer() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

typedef struct {
    uint32_t initial_delay;
    uint32_t max_delay;
    uint32_t current_delay;
    uint32_t multiplier;
    uint32_t jitter_mask;
    int attempt;
} tmr_backoff_t;

void tmr_backoff_init(tmr_backoff_t *b, uint32_t initial, uint32_t max_delay, uint32_t mult) {
    b->initial_delay = initial;
    b->max_delay = max_delay;
    b->current_delay = initial;
    b->multiplier = mult;
    b->jitter_mask = 0x0F;
    b->attempt = 0;
}

uint32_t tmr_backoff_next(tmr_backoff_t *b) {
    uint32_t delay = b->current_delay;
    b->attempt++;
    b->current_delay = b->current_delay * b->multiplier;
    if (b->current_delay > b->max_delay) {
        b->current_delay = b->max_delay;
    }
    uint32_t jitter = (b->attempt * 7) & b->jitter_mask;
    return delay + jitter;
}

void tmr_backoff_reset(tmr_backoff_t *b) {
    b->current_delay = b->initial_delay;
    b->attempt = 0;
}

int tmr_backoff_is_capped(tmr_backoff_t *b) {
    return b->current_delay >= b->max_delay;
}

int tmr_backoff_test(void) {
    tmr_backoff_t b;
    tmr_backoff_init(&b, 100, 3200, 2);
    uint32_t d1 = tmr_backoff_next(&b);
    if (d1 < 100) return -1;
    uint32_t d2 = tmr_backoff_next(&b);
    if (d2 < 200) return -2;
    uint32_t d3 = tmr_backoff_next(&b);
    if (d3 < 400) return -3;
    int i;
    for (i = 0; i < 10; i++) tmr_backoff_next(&b);
    if (!tmr_backoff_is_capped(&b)) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1819: Backoff timer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1819: Output should not be empty");
    assert!(code.contains("fn tmr_backoff_init"), "C1819: Should contain tmr_backoff_init");
    assert!(code.contains("fn tmr_backoff_next"), "C1819: Should contain tmr_backoff_next");
}

/// C1820: Timeout queue with priority-ordered expiration
#[test]
fn c1820_timeout_queue() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define TMR_TQ_CAPACITY 32

typedef struct {
    uint32_t expiry;
    int tag;
    int active;
} tmr_tq_entry_t;

typedef struct {
    tmr_tq_entry_t heap[TMR_TQ_CAPACITY];
    int size;
    uint32_t now;
} tmr_timeout_queue_t;

void tmr_tq_init(tmr_timeout_queue_t *tq) {
    tq->size = 0;
    tq->now = 0;
}

void tmr_tq_swap(tmr_tq_entry_t *a, tmr_tq_entry_t *b) {
    tmr_tq_entry_t tmp = *a;
    *a = *b;
    *b = tmp;
}

void tmr_tq_sift_up(tmr_timeout_queue_t *tq, int idx) {
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (tq->heap[idx].expiry < tq->heap[parent].expiry) {
            tmr_tq_swap(&tq->heap[idx], &tq->heap[parent]);
            idx = parent;
        } else {
            break;
        }
    }
}

int tmr_tq_insert(tmr_timeout_queue_t *tq, int tag, uint32_t timeout) {
    if (tq->size >= TMR_TQ_CAPACITY) return -1;
    int idx = tq->size;
    tq->heap[idx].expiry = tq->now + timeout;
    tq->heap[idx].tag = tag;
    tq->heap[idx].active = 1;
    tq->size++;
    tmr_tq_sift_up(tq, idx);
    return idx;
}

int tmr_tq_peek_expired(tmr_timeout_queue_t *tq) {
    if (tq->size == 0) return -1;
    if (tq->heap[0].expiry <= tq->now) {
        return tq->heap[0].tag;
    }
    return -1;
}

int tmr_tq_test(void) {
    tmr_timeout_queue_t tq;
    tmr_tq_init(&tq);
    tmr_tq_insert(&tq, 10, 50);
    tmr_tq_insert(&tq, 20, 30);
    tmr_tq_insert(&tq, 30, 10);
    tq.now = 15;
    int tag = tmr_tq_peek_expired(&tq);
    if (tag != 30) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1820: Timeout queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1820: Output should not be empty");
    assert!(code.contains("fn tmr_tq_init"), "C1820: Should contain tmr_tq_init");
    assert!(code.contains("fn tmr_tq_insert"), "C1820: Should contain tmr_tq_insert");
}

// ============================================================================
// C1821-C1825: Timer Applications
// ============================================================================

/// C1821: Debounce timer for input signal filtering
#[test]
fn c1821_debounce_timer() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define TMR_DEBOUNCE_CHANNELS 8

typedef struct {
    uint32_t last_change;
    uint32_t debounce_ms;
    int raw_state;
    int stable_state;
    int pending;
} tmr_debounce_ch_t;

typedef struct {
    tmr_debounce_ch_t channels[TMR_DEBOUNCE_CHANNELS];
    int count;
    uint32_t now;
} tmr_debounce_t;

void tmr_debounce_init(tmr_debounce_t *db) {
    db->count = 0;
    db->now = 0;
}

int tmr_debounce_add(tmr_debounce_t *db, uint32_t debounce_ms) {
    if (db->count >= TMR_DEBOUNCE_CHANNELS) return -1;
    int idx = db->count;
    db->channels[idx].debounce_ms = debounce_ms;
    db->channels[idx].raw_state = 0;
    db->channels[idx].stable_state = 0;
    db->channels[idx].pending = 0;
    db->channels[idx].last_change = 0;
    db->count++;
    return idx;
}

int tmr_debounce_input(tmr_debounce_t *db, int ch, int value) {
    if (ch < 0 || ch >= db->count) return -1;
    if (value != db->channels[ch].raw_state) {
        db->channels[ch].raw_state = value;
        db->channels[ch].last_change = db->now;
        db->channels[ch].pending = 1;
    }
    return 0;
}

int tmr_debounce_process(tmr_debounce_t *db, int ch) {
    if (ch < 0 || ch >= db->count) return -1;
    if (!db->channels[ch].pending) return 0;
    uint32_t elapsed = db->now - db->channels[ch].last_change;
    if (elapsed >= db->channels[ch].debounce_ms) {
        db->channels[ch].stable_state = db->channels[ch].raw_state;
        db->channels[ch].pending = 0;
        return 1;
    }
    return 0;
}

int tmr_debounce_test(void) {
    tmr_debounce_t db;
    tmr_debounce_init(&db);
    int ch = tmr_debounce_add(&db, 20);
    tmr_debounce_input(&db, ch, 1);
    db.now = 10;
    int changed = tmr_debounce_process(&db, ch);
    if (changed != 0) return -1;
    db.now = 25;
    changed = tmr_debounce_process(&db, ch);
    if (changed != 1) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1821: Debounce timer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1821: Output should not be empty");
    assert!(code.contains("fn tmr_debounce_init"), "C1821: Should contain tmr_debounce_init");
    assert!(code.contains("fn tmr_debounce_process"), "C1821: Should contain tmr_debounce_process");
}

/// C1822: Rate limiter with token bucket algorithm
#[test]
fn c1822_rate_limiter() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

typedef struct {
    uint32_t tokens;
    uint32_t max_tokens;
    uint32_t refill_rate;
    uint32_t last_refill;
    uint32_t total_allowed;
    uint32_t total_denied;
} tmr_rate_limiter_t;

void tmr_rate_init(tmr_rate_limiter_t *rl, uint32_t max_tokens, uint32_t refill_rate) {
    rl->tokens = max_tokens;
    rl->max_tokens = max_tokens;
    rl->refill_rate = refill_rate;
    rl->last_refill = 0;
    rl->total_allowed = 0;
    rl->total_denied = 0;
}

void tmr_rate_refill(tmr_rate_limiter_t *rl, uint32_t now) {
    uint32_t elapsed = now - rl->last_refill;
    uint32_t new_tokens = elapsed * rl->refill_rate / 1000;
    if (new_tokens > 0) {
        rl->tokens += new_tokens;
        if (rl->tokens > rl->max_tokens) {
            rl->tokens = rl->max_tokens;
        }
        rl->last_refill = now;
    }
}

int tmr_rate_allow(tmr_rate_limiter_t *rl, uint32_t now, uint32_t cost) {
    tmr_rate_refill(rl, now);
    if (rl->tokens >= cost) {
        rl->tokens -= cost;
        rl->total_allowed++;
        return 1;
    }
    rl->total_denied++;
    return 0;
}

uint32_t tmr_rate_until_available(tmr_rate_limiter_t *rl, uint32_t cost) {
    if (rl->tokens >= cost) return 0;
    uint32_t deficit = cost - rl->tokens;
    if (rl->refill_rate == 0) return 0xFFFFFFFF;
    return (deficit * 1000) / rl->refill_rate;
}

int tmr_rate_test(void) {
    tmr_rate_limiter_t rl;
    tmr_rate_init(&rl, 10, 1);
    int i;
    for (i = 0; i < 10; i++) {
        if (!tmr_rate_allow(&rl, 0, 1)) return -1;
    }
    if (tmr_rate_allow(&rl, 0, 1)) return -2;
    tmr_rate_refill(&rl, 5000);
    if (!tmr_rate_allow(&rl, 5000, 1)) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1822: Rate limiter should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1822: Output should not be empty");
    assert!(code.contains("fn tmr_rate_init"), "C1822: Should contain tmr_rate_init");
    assert!(code.contains("fn tmr_rate_allow"), "C1822: Should contain tmr_rate_allow");
}

/// C1823: Heartbeat monitor with missed-beat detection
#[test]
fn c1823_heartbeat_monitor() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define TMR_HB_MAX_PEERS 16
#define TMR_HB_ALIVE   0
#define TMR_HB_SUSPECT 1
#define TMR_HB_DEAD    2

typedef struct {
    int peer_id;
    uint32_t last_seen;
    uint32_t interval;
    int missed;
    int max_missed;
    int state;
} tmr_hb_peer_t;

typedef struct {
    tmr_hb_peer_t peers[TMR_HB_MAX_PEERS];
    int count;
    uint32_t now;
    int dead_count;
} tmr_heartbeat_t;

void tmr_heartbeat_init(tmr_heartbeat_t *hb) {
    hb->count = 0;
    hb->now = 0;
    hb->dead_count = 0;
}

int tmr_heartbeat_register(tmr_heartbeat_t *hb, int peer_id, uint32_t interval, int max_missed) {
    if (hb->count >= TMR_HB_MAX_PEERS) return -1;
    int idx = hb->count;
    hb->peers[idx].peer_id = peer_id;
    hb->peers[idx].last_seen = hb->now;
    hb->peers[idx].interval = interval;
    hb->peers[idx].missed = 0;
    hb->peers[idx].max_missed = max_missed;
    hb->peers[idx].state = TMR_HB_ALIVE;
    hb->count++;
    return idx;
}

void tmr_heartbeat_receive(tmr_heartbeat_t *hb, int peer_id) {
    int i;
    for (i = 0; i < hb->count; i++) {
        if (hb->peers[i].peer_id == peer_id) {
            hb->peers[i].last_seen = hb->now;
            hb->peers[i].missed = 0;
            hb->peers[i].state = TMR_HB_ALIVE;
            return;
        }
    }
}

int tmr_heartbeat_check(tmr_heartbeat_t *hb) {
    int newly_dead = 0;
    int i;
    for (i = 0; i < hb->count; i++) {
        if (hb->peers[i].state == TMR_HB_DEAD) continue;
        uint32_t elapsed = hb->now - hb->peers[i].last_seen;
        int expected_beats = 0;
        if (hb->peers[i].interval > 0) {
            expected_beats = elapsed / hb->peers[i].interval;
        }
        hb->peers[i].missed = expected_beats;
        if (expected_beats >= hb->peers[i].max_missed) {
            hb->peers[i].state = TMR_HB_DEAD;
            hb->dead_count++;
            newly_dead++;
        } else if (expected_beats > 0) {
            hb->peers[i].state = TMR_HB_SUSPECT;
        }
    }
    return newly_dead;
}

int tmr_heartbeat_test(void) {
    tmr_heartbeat_t hb;
    tmr_heartbeat_init(&hb);
    tmr_heartbeat_register(&hb, 1, 100, 3);
    tmr_heartbeat_register(&hb, 2, 100, 3);
    hb.now = 350;
    tmr_heartbeat_receive(&hb, 1);
    int dead = tmr_heartbeat_check(&hb);
    if (dead != 1) return -1;
    if (hb.peers[0].state != TMR_HB_ALIVE) return -2;
    if (hb.peers[1].state != TMR_HB_DEAD) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1823: Heartbeat monitor should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1823: Output should not be empty");
    assert!(code.contains("fn tmr_heartbeat_init"), "C1823: Should contain tmr_heartbeat_init");
    assert!(code.contains("fn tmr_heartbeat_check"), "C1823: Should contain tmr_heartbeat_check");
}

/// C1824: Watchdog timer with kick and bite mechanisms
#[test]
fn c1824_watchdog_timer() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define TMR_WDT_STOPPED 0
#define TMR_WDT_RUNNING 1
#define TMR_WDT_BITTEN  2

typedef struct {
    uint32_t timeout;
    uint32_t last_kick;
    uint32_t bite_count;
    uint32_t kick_count;
    int state;
    int window_mode;
    uint32_t window_open;
} tmr_watchdog_t;

void tmr_watchdog_init(tmr_watchdog_t *wd, uint32_t timeout, int window_mode) {
    wd->timeout = timeout;
    wd->last_kick = 0;
    wd->bite_count = 0;
    wd->kick_count = 0;
    wd->state = TMR_WDT_STOPPED;
    wd->window_mode = window_mode;
    wd->window_open = timeout / 2;
}

void tmr_watchdog_start(tmr_watchdog_t *wd, uint32_t now) {
    wd->last_kick = now;
    wd->state = TMR_WDT_RUNNING;
}

int tmr_watchdog_kick(tmr_watchdog_t *wd, uint32_t now) {
    if (wd->state != TMR_WDT_RUNNING) return -1;
    if (wd->window_mode) {
        uint32_t elapsed = now - wd->last_kick;
        if (elapsed < wd->window_open) {
            wd->state = TMR_WDT_BITTEN;
            wd->bite_count++;
            return -2;
        }
    }
    wd->last_kick = now;
    wd->kick_count++;
    return 0;
}

int tmr_watchdog_check(tmr_watchdog_t *wd, uint32_t now) {
    if (wd->state != TMR_WDT_RUNNING) return 0;
    uint32_t elapsed = now - wd->last_kick;
    if (elapsed >= wd->timeout) {
        wd->state = TMR_WDT_BITTEN;
        wd->bite_count++;
        return 1;
    }
    return 0;
}

int tmr_watchdog_test(void) {
    tmr_watchdog_t wd;
    tmr_watchdog_init(&wd, 1000, 0);
    tmr_watchdog_start(&wd, 0);
    tmr_watchdog_kick(&wd, 500);
    int bit = tmr_watchdog_check(&wd, 800);
    if (bit != 0) return -1;
    bit = tmr_watchdog_check(&wd, 1600);
    if (bit != 1) return -2;
    if (wd.bite_count != 1) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1824: Watchdog timer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1824: Output should not be empty");
    assert!(code.contains("fn tmr_watchdog_init"), "C1824: Should contain tmr_watchdog_init");
    assert!(code.contains("fn tmr_watchdog_check"), "C1824: Should contain tmr_watchdog_check");
}

/// C1825: Profiling timer with hierarchical scope tracking
#[test]
fn c1825_profiling_timer() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

#define TMR_PROF_MAX_SCOPES 16
#define TMR_PROF_MAX_DEPTH  8

typedef struct {
    uint64_t total_ticks;
    uint64_t start_tick;
    uint32_t call_count;
    uint32_t max_ticks;
    uint32_t min_ticks;
    int active;
    int id;
} tmr_prof_scope_t;

typedef struct {
    tmr_prof_scope_t scopes[TMR_PROF_MAX_SCOPES];
    int scope_count;
    int stack[TMR_PROF_MAX_DEPTH];
    int stack_depth;
    uint64_t clock;
} tmr_profiler_t;

void tmr_profiler_init(tmr_profiler_t *p) {
    p->scope_count = 0;
    p->stack_depth = 0;
    p->clock = 0;
}

int tmr_profiler_register(tmr_profiler_t *p, int id) {
    if (p->scope_count >= TMR_PROF_MAX_SCOPES) return -1;
    int idx = p->scope_count;
    p->scopes[idx].id = id;
    p->scopes[idx].total_ticks = 0;
    p->scopes[idx].start_tick = 0;
    p->scopes[idx].call_count = 0;
    p->scopes[idx].max_ticks = 0;
    p->scopes[idx].min_ticks = 0xFFFFFFFF;
    p->scopes[idx].active = 0;
    p->scope_count++;
    return idx;
}

int tmr_profiler_enter(tmr_profiler_t *p, int idx) {
    if (idx < 0 || idx >= p->scope_count) return -1;
    if (p->stack_depth >= TMR_PROF_MAX_DEPTH) return -2;
    p->scopes[idx].start_tick = p->clock;
    p->scopes[idx].active = 1;
    p->stack[p->stack_depth] = idx;
    p->stack_depth++;
    return 0;
}

int tmr_profiler_leave(tmr_profiler_t *p) {
    if (p->stack_depth <= 0) return -1;
    p->stack_depth--;
    int idx = p->stack[p->stack_depth];
    uint64_t elapsed = p->clock - p->scopes[idx].start_tick;
    p->scopes[idx].total_ticks += elapsed;
    p->scopes[idx].call_count++;
    if ((uint32_t)elapsed > p->scopes[idx].max_ticks) {
        p->scopes[idx].max_ticks = (uint32_t)elapsed;
    }
    if ((uint32_t)elapsed < p->scopes[idx].min_ticks) {
        p->scopes[idx].min_ticks = (uint32_t)elapsed;
    }
    p->scopes[idx].active = 0;
    return idx;
}

uint32_t tmr_profiler_avg(tmr_profiler_t *p, int idx) {
    if (idx < 0 || idx >= p->scope_count) return 0;
    if (p->scopes[idx].call_count == 0) return 0;
    return (uint32_t)(p->scopes[idx].total_ticks / p->scopes[idx].call_count);
}

int tmr_profiler_test(void) {
    tmr_profiler_t p;
    tmr_profiler_init(&p);
    int s0 = tmr_profiler_register(&p, 100);
    int s1 = tmr_profiler_register(&p, 200);
    p.clock = 10;
    tmr_profiler_enter(&p, s0);
    p.clock = 20;
    tmr_profiler_enter(&p, s1);
    p.clock = 35;
    tmr_profiler_leave(&p);
    p.clock = 50;
    tmr_profiler_leave(&p);
    uint32_t avg0 = tmr_profiler_avg(&p, s0);
    uint32_t avg1 = tmr_profiler_avg(&p, s1);
    if (avg0 != 40) return -1;
    if (avg1 != 15) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1825: Profiling timer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1825: Output should not be empty");
    assert!(code.contains("fn tmr_profiler_init"), "C1825: Should contain tmr_profiler_init");
    assert!(code.contains("fn tmr_profiler_enter"), "C1825: Should contain tmr_profiler_enter");
}
