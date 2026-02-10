//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1626-C1650: Event-Driven Systems -- event loops, publish-subscribe,
//! event sourcing, reactive streams, and signal handling patterns commonly
//! found in production event-driven architectures.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! Organization:
//! - C1626-C1630: Event loop (poll-based, epoll abstraction, timer wheel, priorities, coalescing)
//! - C1631-C1635: Publish-subscribe (topic subscription, wildcard matching, event filtering, backpressure, fan-out)
//! - C1636-C1640: Event sourcing (event store append, replay, snapshot, projection, command handler)
//! - C1641-C1645: Reactive streams (observable creation, map/filter, merge, buffer strategy, error propagation)
//! - C1646-C1650: Signal handling (registration, masking, queue, async-safe handler, forwarding)

// ============================================================================
// C1626-C1630: Event Loop
// ============================================================================

/// C1626: Poll-based event loop with file descriptor monitoring
#[test]
fn c1626_poll_event_loop() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;

#define EVT_MAX_FDS 64
#define EVT_POLL_READ  0x01
#define EVT_POLL_WRITE 0x02
#define EVT_POLL_ERROR 0x04

typedef struct {
    int fd;
    uint32_t events;
    uint32_t revents;
} evt_poll_entry_t;

typedef struct {
    evt_poll_entry_t entries[EVT_MAX_FDS];
    int count;
    int running;
    int timeout_ms;
    int iterations;
} evt_poll_loop_t;

void evt_poll_init(evt_poll_loop_t *loop) {
    loop->count = 0;
    loop->running = 0;
    loop->timeout_ms = 100;
    loop->iterations = 0;
}

int evt_poll_add_fd(evt_poll_loop_t *loop, int fd, uint32_t events) {
    if (loop->count >= EVT_MAX_FDS) return -1;
    int idx = loop->count;
    loop->entries[idx].fd = fd;
    loop->entries[idx].events = events;
    loop->entries[idx].revents = 0;
    loop->count++;
    return idx;
}

int evt_poll_remove_fd(evt_poll_loop_t *loop, int fd) {
    int i;
    for (i = 0; i < loop->count; i++) {
        if (loop->entries[i].fd == fd) {
            int j;
            for (j = i; j < loop->count - 1; j++) {
                loop->entries[j] = loop->entries[j + 1];
            }
            loop->count--;
            return 0;
        }
    }
    return -1;
}

int evt_poll_check(evt_poll_loop_t *loop) {
    int ready = 0;
    int i;
    for (i = 0; i < loop->count; i++) {
        loop->entries[i].revents = 0;
        if (loop->entries[i].events & EVT_POLL_READ) {
            loop->entries[i].revents |= EVT_POLL_READ;
            ready++;
        }
    }
    loop->iterations++;
    return ready;
}

int evt_poll_run(evt_poll_loop_t *loop, int max_iterations) {
    loop->running = 1;
    int total = 0;
    while (loop->running && loop->iterations < max_iterations) {
        int n = evt_poll_check(loop);
        total += n;
    }
    loop->running = 0;
    return total;
}

int evt_poll_test(void) {
    evt_poll_loop_t loop;
    evt_poll_init(&loop);
    evt_poll_add_fd(&loop, 3, EVT_POLL_READ);
    evt_poll_add_fd(&loop, 4, EVT_POLL_READ | EVT_POLL_WRITE);
    int ready = evt_poll_check(&loop);
    if (ready != 2) return -1;
    evt_poll_remove_fd(&loop, 3);
    if (loop.count != 1) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1626: Poll event loop should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1626: Output should not be empty");
    assert!(code.contains("fn evt_poll_init"), "C1626: Should contain evt_poll_init");
    assert!(code.contains("fn evt_poll_add_fd"), "C1626: Should contain evt_poll_add_fd");
    assert!(code.contains("fn evt_poll_check"), "C1626: Should contain evt_poll_check");
}

/// C1627: Epoll-style abstraction with interest sets and readiness
#[test]
fn c1627_epoll_abstraction() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define EVT_EPOLL_MAX 128
#define EVT_EPOLLIN  0x001
#define EVT_EPOLLOUT 0x004
#define EVT_EPOLLERR 0x008

typedef struct {
    int fd;
    uint32_t interest;
    uint32_t ready;
    int active;
} evt_epoll_entry_t;

typedef struct {
    evt_epoll_entry_t entries[EVT_EPOLL_MAX];
    int count;
    int ready_count;
} evt_epoll_ctx_t;

void evt_epoll_create(evt_epoll_ctx_t *ctx) {
    ctx->count = 0;
    ctx->ready_count = 0;
}

int evt_epoll_ctl_add(evt_epoll_ctx_t *ctx, int fd, uint32_t interest) {
    if (ctx->count >= EVT_EPOLL_MAX) return -1;
    int idx = ctx->count;
    ctx->entries[idx].fd = fd;
    ctx->entries[idx].interest = interest;
    ctx->entries[idx].ready = 0;
    ctx->entries[idx].active = 1;
    ctx->count++;
    return 0;
}

int evt_epoll_ctl_mod(evt_epoll_ctx_t *ctx, int fd, uint32_t interest) {
    int i;
    for (i = 0; i < ctx->count; i++) {
        if (ctx->entries[i].fd == fd && ctx->entries[i].active) {
            ctx->entries[i].interest = interest;
            return 0;
        }
    }
    return -1;
}

int evt_epoll_ctl_del(evt_epoll_ctx_t *ctx, int fd) {
    int i;
    for (i = 0; i < ctx->count; i++) {
        if (ctx->entries[i].fd == fd) {
            ctx->entries[i].active = 0;
            return 0;
        }
    }
    return -1;
}

int evt_epoll_wait(evt_epoll_ctx_t *ctx, int *ready_fds, int max_events) {
    int found = 0;
    int i;
    for (i = 0; i < ctx->count && found < max_events; i++) {
        if (ctx->entries[i].active && ctx->entries[i].ready) {
            ready_fds[found] = ctx->entries[i].fd;
            found++;
        }
    }
    ctx->ready_count = found;
    return found;
}

int evt_epoll_test(void) {
    evt_epoll_ctx_t ctx;
    evt_epoll_create(&ctx);
    evt_epoll_ctl_add(&ctx, 5, EVT_EPOLLIN);
    evt_epoll_ctl_add(&ctx, 6, EVT_EPOLLIN | EVT_EPOLLOUT);
    ctx.entries[0].ready = EVT_EPOLLIN;
    int fds[8];
    int n = evt_epoll_wait(&ctx, fds, 8);
    if (n != 1) return -1;
    if (fds[0] != 5) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1627: Epoll abstraction should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1627: Output should not be empty");
    assert!(code.contains("fn evt_epoll_create"), "C1627: Should contain evt_epoll_create");
    assert!(code.contains("fn evt_epoll_ctl_add"), "C1627: Should contain evt_epoll_ctl_add");
    assert!(code.contains("fn evt_epoll_wait"), "C1627: Should contain evt_epoll_wait");
}

/// C1628: Timer wheel with hierarchical bucket management
#[test]
fn c1628_timer_wheel() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define EVT_WHEEL_SLOTS 64
#define EVT_MAX_TIMERS 256

typedef struct {
    int id;
    uint64_t expiry;
    uint64_t interval;
    int repeating;
    int active;
    int slot;
} evt_timer_entry_t;

typedef struct {
    evt_timer_entry_t timers[EVT_MAX_TIMERS];
    int timer_count;
    int slots[EVT_WHEEL_SLOTS];
    uint64_t current_tick;
    int current_slot;
    int fired_count;
} evt_timer_wheel_t;

void evt_wheel_init(evt_timer_wheel_t *wheel) {
    wheel->timer_count = 0;
    wheel->current_tick = 0;
    wheel->current_slot = 0;
    wheel->fired_count = 0;
    int i;
    for (i = 0; i < EVT_WHEEL_SLOTS; i++) {
        wheel->slots[i] = 0;
    }
}

int evt_wheel_add_timer(evt_timer_wheel_t *wheel, uint64_t delay, uint64_t interval, int repeating) {
    if (wheel->timer_count >= EVT_MAX_TIMERS) return -1;
    int id = wheel->timer_count;
    wheel->timers[id].id = id;
    wheel->timers[id].expiry = wheel->current_tick + delay;
    wheel->timers[id].interval = interval;
    wheel->timers[id].repeating = repeating;
    wheel->timers[id].active = 1;
    wheel->timers[id].slot = (int)((wheel->current_tick + delay) % EVT_WHEEL_SLOTS);
    wheel->slots[wheel->timers[id].slot]++;
    wheel->timer_count++;
    return id;
}

int evt_wheel_cancel_timer(evt_timer_wheel_t *wheel, int id) {
    if (id < 0 || id >= wheel->timer_count) return -1;
    if (!wheel->timers[id].active) return -1;
    wheel->timers[id].active = 0;
    wheel->slots[wheel->timers[id].slot]--;
    return 0;
}

int evt_wheel_advance(evt_timer_wheel_t *wheel) {
    wheel->current_tick++;
    wheel->current_slot = (int)(wheel->current_tick % EVT_WHEEL_SLOTS);
    int fired = 0;
    int i;
    for (i = 0; i < wheel->timer_count; i++) {
        if (wheel->timers[i].active && wheel->timers[i].expiry == wheel->current_tick) {
            fired++;
            wheel->fired_count++;
            if (wheel->timers[i].repeating) {
                wheel->timers[i].expiry = wheel->current_tick + wheel->timers[i].interval;
                int old_slot = wheel->timers[i].slot;
                wheel->timers[i].slot = (int)(wheel->timers[i].expiry % EVT_WHEEL_SLOTS);
                wheel->slots[old_slot]--;
                wheel->slots[wheel->timers[i].slot]++;
            } else {
                wheel->timers[i].active = 0;
                wheel->slots[wheel->timers[i].slot]--;
            }
        }
    }
    return fired;
}

int evt_wheel_test(void) {
    evt_timer_wheel_t wheel;
    evt_wheel_init(&wheel);
    evt_wheel_add_timer(&wheel, 5, 0, 0);
    evt_wheel_add_timer(&wheel, 10, 10, 1);
    int i;
    for (i = 0; i < 5; i++) evt_wheel_advance(&wheel);
    if (wheel.fired_count != 1) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1628: Timer wheel should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1628: Output should not be empty");
    assert!(code.contains("fn evt_wheel_init"), "C1628: Should contain evt_wheel_init");
    assert!(code.contains("fn evt_wheel_add_timer"), "C1628: Should contain evt_wheel_add_timer");
    assert!(code.contains("fn evt_wheel_advance"), "C1628: Should contain evt_wheel_advance");
}

/// C1629: Event priorities with multi-level dispatch queue
#[test]
fn c1629_event_priorities() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define EVT_PRIO_LEVELS 4
#define EVT_QUEUE_SIZE 32
#define EVT_PRIO_CRITICAL 0
#define EVT_PRIO_HIGH     1
#define EVT_PRIO_NORMAL   2
#define EVT_PRIO_LOW      3

typedef struct {
    int event_id;
    int type;
    uint32_t data;
    int priority;
} evt_prio_event_t;

typedef struct {
    evt_prio_event_t queue[EVT_QUEUE_SIZE];
    int head;
    int tail;
    int count;
} evt_prio_queue_t;

typedef struct {
    evt_prio_queue_t levels[EVT_PRIO_LEVELS];
    int total_dispatched;
    int total_enqueued;
} evt_prio_dispatcher_t;

void evt_prio_queue_init(evt_prio_queue_t *q) {
    q->head = 0;
    q->tail = 0;
    q->count = 0;
}

void evt_prio_init(evt_prio_dispatcher_t *disp) {
    int i;
    for (i = 0; i < EVT_PRIO_LEVELS; i++) {
        evt_prio_queue_init(&disp->levels[i]);
    }
    disp->total_dispatched = 0;
    disp->total_enqueued = 0;
}

int evt_prio_enqueue(evt_prio_dispatcher_t *disp, int event_id, int type, uint32_t data, int priority) {
    if (priority < 0 || priority >= EVT_PRIO_LEVELS) return -1;
    evt_prio_queue_t *q = &disp->levels[priority];
    if (q->count >= EVT_QUEUE_SIZE) return -2;
    q->queue[q->tail].event_id = event_id;
    q->queue[q->tail].type = type;
    q->queue[q->tail].data = data;
    q->queue[q->tail].priority = priority;
    q->tail = (q->tail + 1) % EVT_QUEUE_SIZE;
    q->count++;
    disp->total_enqueued++;
    return 0;
}

int evt_prio_dequeue(evt_prio_dispatcher_t *disp, evt_prio_event_t *out) {
    int i;
    for (i = 0; i < EVT_PRIO_LEVELS; i++) {
        evt_prio_queue_t *q = &disp->levels[i];
        if (q->count > 0) {
            *out = q->queue[q->head];
            q->head = (q->head + 1) % EVT_QUEUE_SIZE;
            q->count--;
            disp->total_dispatched++;
            return 0;
        }
    }
    return -1;
}

int evt_prio_pending(evt_prio_dispatcher_t *disp) {
    int total = 0;
    int i;
    for (i = 0; i < EVT_PRIO_LEVELS; i++) {
        total += disp->levels[i].count;
    }
    return total;
}

int evt_prio_test(void) {
    evt_prio_dispatcher_t disp;
    evt_prio_init(&disp);
    evt_prio_enqueue(&disp, 1, 10, 100, EVT_PRIO_LOW);
    evt_prio_enqueue(&disp, 2, 20, 200, EVT_PRIO_CRITICAL);
    evt_prio_event_t out;
    evt_prio_dequeue(&disp, &out);
    if (out.event_id != 2) return -1;
    if (evt_prio_pending(&disp) != 1) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1629: Event priorities should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1629: Output should not be empty");
    assert!(code.contains("fn evt_prio_init"), "C1629: Should contain evt_prio_init");
    assert!(code.contains("fn evt_prio_enqueue"), "C1629: Should contain evt_prio_enqueue");
    assert!(code.contains("fn evt_prio_dequeue"), "C1629: Should contain evt_prio_dequeue");
}

/// C1630: Event coalescing with dedup and batching
#[test]
fn c1630_event_coalescing() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define EVT_COALESCE_MAX 128

typedef struct {
    int event_type;
    uint32_t data;
    int count;
    uint32_t first_timestamp;
    uint32_t last_timestamp;
} evt_coalesced_t;

typedef struct {
    evt_coalesced_t events[EVT_COALESCE_MAX];
    int count;
    int total_received;
    int total_coalesced;
    uint32_t window_ms;
} evt_coalescer_t;

void evt_coalesce_init(evt_coalescer_t *c, uint32_t window_ms) {
    c->count = 0;
    c->total_received = 0;
    c->total_coalesced = 0;
    c->window_ms = window_ms;
}

int evt_coalesce_find(evt_coalescer_t *c, int event_type) {
    int i;
    for (i = 0; i < c->count; i++) {
        if (c->events[i].event_type == event_type) {
            return i;
        }
    }
    return -1;
}

int evt_coalesce_submit(evt_coalescer_t *c, int event_type, uint32_t data, uint32_t timestamp) {
    c->total_received++;
    int idx = evt_coalesce_find(c, event_type);
    if (idx >= 0) {
        if (timestamp - c->events[idx].last_timestamp <= c->window_ms) {
            c->events[idx].data = data;
            c->events[idx].count++;
            c->events[idx].last_timestamp = timestamp;
            c->total_coalesced++;
            return 1;
        }
    }
    if (c->count >= EVT_COALESCE_MAX) return -1;
    idx = c->count;
    c->events[idx].event_type = event_type;
    c->events[idx].data = data;
    c->events[idx].count = 1;
    c->events[idx].first_timestamp = timestamp;
    c->events[idx].last_timestamp = timestamp;
    c->count++;
    return 0;
}

int evt_coalesce_flush(evt_coalescer_t *c) {
    int flushed = c->count;
    c->count = 0;
    return flushed;
}

int evt_coalesce_test(void) {
    evt_coalescer_t c;
    evt_coalesce_init(&c, 50);
    evt_coalesce_submit(&c, 1, 100, 10);
    evt_coalesce_submit(&c, 1, 200, 30);
    evt_coalesce_submit(&c, 2, 300, 40);
    if (c.count != 2) return -1;
    if (c.total_coalesced != 1) return -2;
    if (evt_coalesce_flush(&c) != 2) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1630: Event coalescing should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1630: Output should not be empty");
    assert!(code.contains("fn evt_coalesce_init"), "C1630: Should contain evt_coalesce_init");
    assert!(code.contains("fn evt_coalesce_submit"), "C1630: Should contain evt_coalesce_submit");
    assert!(code.contains("fn evt_coalesce_flush"), "C1630: Should contain evt_coalesce_flush");
}

// ============================================================================
// C1631-C1635: Publish-Subscribe
// ============================================================================

/// C1631: Topic subscription with subscriber registry
#[test]
fn c1631_topic_subscription() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define EVT_MAX_TOPICS 32
#define EVT_MAX_SUBSCRIBERS 16
#define EVT_TOPIC_NAME_LEN 32

typedef struct {
    int subscriber_id;
    int active;
} evt_subscriber_t;

typedef struct {
    char name[EVT_TOPIC_NAME_LEN];
    evt_subscriber_t subscribers[EVT_MAX_SUBSCRIBERS];
    int sub_count;
    int msg_count;
} evt_topic_t;

typedef struct {
    evt_topic_t topics[EVT_MAX_TOPICS];
    int topic_count;
    int next_sub_id;
} evt_pubsub_t;

void evt_pubsub_init(evt_pubsub_t *ps) {
    ps->topic_count = 0;
    ps->next_sub_id = 1;
}

int evt_pubsub_find_topic(evt_pubsub_t *ps, const char *name) {
    int i;
    for (i = 0; i < ps->topic_count; i++) {
        int j = 0;
        int match = 1;
        while (ps->topics[i].name[j] != 0 || name[j] != 0) {
            if (ps->topics[i].name[j] != name[j]) { match = 0; break; }
            j++;
        }
        if (match) return i;
    }
    return -1;
}

int evt_pubsub_create_topic(evt_pubsub_t *ps, const char *name) {
    if (ps->topic_count >= EVT_MAX_TOPICS) return -1;
    int idx = ps->topic_count;
    int i = 0;
    while (name[i] != 0 && i < EVT_TOPIC_NAME_LEN - 1) {
        ps->topics[idx].name[i] = name[i];
        i++;
    }
    ps->topics[idx].name[i] = 0;
    ps->topics[idx].sub_count = 0;
    ps->topics[idx].msg_count = 0;
    ps->topic_count++;
    return idx;
}

int evt_pubsub_subscribe(evt_pubsub_t *ps, const char *topic_name) {
    int tidx = evt_pubsub_find_topic(ps, topic_name);
    if (tidx < 0) return -1;
    evt_topic_t *t = &ps->topics[tidx];
    if (t->sub_count >= EVT_MAX_SUBSCRIBERS) return -2;
    int sid = ps->next_sub_id++;
    t->subscribers[t->sub_count].subscriber_id = sid;
    t->subscribers[t->sub_count].active = 1;
    t->sub_count++;
    return sid;
}

int evt_pubsub_publish(evt_pubsub_t *ps, const char *topic_name) {
    int tidx = evt_pubsub_find_topic(ps, topic_name);
    if (tidx < 0) return -1;
    ps->topics[tidx].msg_count++;
    int delivered = 0;
    int i;
    for (i = 0; i < ps->topics[tidx].sub_count; i++) {
        if (ps->topics[tidx].subscribers[i].active) {
            delivered++;
        }
    }
    return delivered;
}

int evt_pubsub_test(void) {
    evt_pubsub_t ps;
    evt_pubsub_init(&ps);
    evt_pubsub_create_topic(&ps, "events");
    evt_pubsub_subscribe(&ps, "events");
    evt_pubsub_subscribe(&ps, "events");
    int delivered = evt_pubsub_publish(&ps, "events");
    if (delivered != 2) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1631: Topic subscription should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1631: Output should not be empty");
    assert!(code.contains("fn evt_pubsub_init"), "C1631: Should contain evt_pubsub_init");
    assert!(code.contains("fn evt_pubsub_subscribe"), "C1631: Should contain evt_pubsub_subscribe");
    assert!(code.contains("fn evt_pubsub_publish"), "C1631: Should contain evt_pubsub_publish");
}

/// C1632: Wildcard topic matching with glob-style patterns
#[test]
fn c1632_wildcard_matching() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define EVT_WILD_MAX_SUBS 64
#define EVT_WILD_PATTERN_LEN 64

typedef struct {
    char pattern[EVT_WILD_PATTERN_LEN];
    int subscriber_id;
    int active;
    int match_count;
} evt_wild_sub_t;

typedef struct {
    evt_wild_sub_t subs[EVT_WILD_MAX_SUBS];
    int sub_count;
    int next_id;
} evt_wild_matcher_t;

void evt_wild_init(evt_wild_matcher_t *m) {
    m->sub_count = 0;
    m->next_id = 1;
}

int evt_wild_match_segment(const char *pattern, const char *topic) {
    int pi = 0;
    int ti = 0;
    while (pattern[pi] != 0 && topic[ti] != 0) {
        if (pattern[pi] == '*') {
            return 1;
        }
        if (pattern[pi] == '?') {
            pi++;
            ti++;
            continue;
        }
        if (pattern[pi] != topic[ti]) return 0;
        pi++;
        ti++;
    }
    if (pattern[pi] == '*') return 1;
    return (pattern[pi] == 0 && topic[ti] == 0) ? 1 : 0;
}

int evt_wild_subscribe(evt_wild_matcher_t *m, const char *pattern) {
    if (m->sub_count >= EVT_WILD_MAX_SUBS) return -1;
    int idx = m->sub_count;
    int i = 0;
    while (pattern[i] != 0 && i < EVT_WILD_PATTERN_LEN - 1) {
        m->subs[idx].pattern[i] = pattern[i];
        i++;
    }
    m->subs[idx].pattern[i] = 0;
    m->subs[idx].subscriber_id = m->next_id++;
    m->subs[idx].active = 1;
    m->subs[idx].match_count = 0;
    m->sub_count++;
    return m->subs[idx].subscriber_id;
}

int evt_wild_dispatch(evt_wild_matcher_t *m, const char *topic) {
    int matched = 0;
    int i;
    for (i = 0; i < m->sub_count; i++) {
        if (m->subs[i].active && evt_wild_match_segment(m->subs[i].pattern, topic)) {
            m->subs[i].match_count++;
            matched++;
        }
    }
    return matched;
}

int evt_wild_test(void) {
    evt_wild_matcher_t m;
    evt_wild_init(&m);
    evt_wild_subscribe(&m, "sensor.*");
    evt_wild_subscribe(&m, "sensor.temp");
    int n = evt_wild_dispatch(&m, "sensor.temp");
    if (n != 2) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1632: Wildcard matching should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1632: Output should not be empty");
    assert!(code.contains("fn evt_wild_init"), "C1632: Should contain evt_wild_init");
    assert!(code.contains("fn evt_wild_match_segment"), "C1632: Should contain evt_wild_match_segment");
    assert!(code.contains("fn evt_wild_dispatch"), "C1632: Should contain evt_wild_dispatch");
}

/// C1633: Event filtering with predicate chains
#[test]
fn c1633_event_filtering() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define EVT_FILTER_MAX 32
#define EVT_FILTER_EQ   0
#define EVT_FILTER_GT   1
#define EVT_FILTER_LT   2
#define EVT_FILTER_RANGE 3

typedef struct {
    int field_id;
    int op;
    int value;
    int value_max;
} evt_filter_predicate_t;

typedef struct {
    evt_filter_predicate_t predicates[EVT_FILTER_MAX];
    int count;
    int matched;
    int rejected;
} evt_filter_chain_t;

void evt_filter_init(evt_filter_chain_t *chain) {
    chain->count = 0;
    chain->matched = 0;
    chain->rejected = 0;
}

int evt_filter_add(evt_filter_chain_t *chain, int field_id, int op, int value, int value_max) {
    if (chain->count >= EVT_FILTER_MAX) return -1;
    int idx = chain->count;
    chain->predicates[idx].field_id = field_id;
    chain->predicates[idx].op = op;
    chain->predicates[idx].value = value;
    chain->predicates[idx].value_max = value_max;
    chain->count++;
    return idx;
}

int evt_filter_eval_predicate(evt_filter_predicate_t *pred, int field_value) {
    if (pred->op == EVT_FILTER_EQ) return field_value == pred->value;
    if (pred->op == EVT_FILTER_GT) return field_value > pred->value;
    if (pred->op == EVT_FILTER_LT) return field_value < pred->value;
    if (pred->op == EVT_FILTER_RANGE) return field_value >= pred->value && field_value <= pred->value_max;
    return 0;
}

int evt_filter_check(evt_filter_chain_t *chain, int *field_values, int num_fields) {
    int i;
    for (i = 0; i < chain->count; i++) {
        int fid = chain->predicates[i].field_id;
        if (fid < 0 || fid >= num_fields) {
            chain->rejected++;
            return 0;
        }
        if (!evt_filter_eval_predicate(&chain->predicates[i], field_values[fid])) {
            chain->rejected++;
            return 0;
        }
    }
    chain->matched++;
    return 1;
}

int evt_filter_test(void) {
    evt_filter_chain_t chain;
    evt_filter_init(&chain);
    evt_filter_add(&chain, 0, EVT_FILTER_GT, 10, 0);
    evt_filter_add(&chain, 1, EVT_FILTER_RANGE, 5, 15);
    int fields1[2];
    fields1[0] = 20;
    fields1[1] = 10;
    if (!evt_filter_check(&chain, fields1, 2)) return -1;
    int fields2[2];
    fields2[0] = 5;
    fields2[1] = 10;
    if (evt_filter_check(&chain, fields2, 2)) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1633: Event filtering should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1633: Output should not be empty");
    assert!(code.contains("fn evt_filter_init"), "C1633: Should contain evt_filter_init");
    assert!(code.contains("fn evt_filter_add"), "C1633: Should contain evt_filter_add");
    assert!(code.contains("fn evt_filter_check"), "C1633: Should contain evt_filter_check");
}

/// C1634: Backpressure handler with flow control
#[test]
fn c1634_backpressure_handler() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define EVT_BP_BUFFER_SIZE 256
#define EVT_BP_STATE_OPEN     0
#define EVT_BP_STATE_THROTTLE 1
#define EVT_BP_STATE_CLOSED   2

typedef struct {
    uint32_t buffer[EVT_BP_BUFFER_SIZE];
    int head;
    int tail;
    int count;
    int state;
    int high_watermark;
    int low_watermark;
    int dropped_count;
    int total_in;
    int total_out;
} evt_backpressure_t;

void evt_bp_init(evt_backpressure_t *bp, int high_wm, int low_wm) {
    bp->head = 0;
    bp->tail = 0;
    bp->count = 0;
    bp->state = EVT_BP_STATE_OPEN;
    bp->high_watermark = high_wm;
    bp->low_watermark = low_wm;
    bp->dropped_count = 0;
    bp->total_in = 0;
    bp->total_out = 0;
}

void evt_bp_update_state(evt_backpressure_t *bp) {
    if (bp->count >= bp->high_watermark) {
        bp->state = EVT_BP_STATE_CLOSED;
    } else if (bp->count <= bp->low_watermark) {
        bp->state = EVT_BP_STATE_OPEN;
    } else if (bp->state == EVT_BP_STATE_CLOSED) {
        bp->state = EVT_BP_STATE_THROTTLE;
    }
}

int evt_bp_push(evt_backpressure_t *bp, uint32_t value) {
    if (bp->state == EVT_BP_STATE_CLOSED) {
        bp->dropped_count++;
        return -1;
    }
    if (bp->count >= EVT_BP_BUFFER_SIZE) {
        bp->dropped_count++;
        return -2;
    }
    bp->buffer[bp->tail] = value;
    bp->tail = (bp->tail + 1) % EVT_BP_BUFFER_SIZE;
    bp->count++;
    bp->total_in++;
    evt_bp_update_state(bp);
    return 0;
}

int evt_bp_pop(evt_backpressure_t *bp, uint32_t *out) {
    if (bp->count == 0) return -1;
    *out = bp->buffer[bp->head];
    bp->head = (bp->head + 1) % EVT_BP_BUFFER_SIZE;
    bp->count--;
    bp->total_out++;
    evt_bp_update_state(bp);
    return 0;
}

int evt_bp_test(void) {
    evt_backpressure_t bp;
    evt_bp_init(&bp, 200, 50);
    int i;
    for (i = 0; i < 210; i++) {
        evt_bp_push(&bp, (uint32_t)i);
    }
    if (bp.dropped_count == 0) return -1;
    uint32_t val;
    for (i = 0; i < 160; i++) {
        evt_bp_pop(&bp, &val);
    }
    if (bp.state != EVT_BP_STATE_OPEN) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1634: Backpressure handler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1634: Output should not be empty");
    assert!(code.contains("fn evt_bp_init"), "C1634: Should contain evt_bp_init");
    assert!(code.contains("fn evt_bp_push"), "C1634: Should contain evt_bp_push");
    assert!(code.contains("fn evt_bp_pop"), "C1634: Should contain evt_bp_pop");
}

/// C1635: Fan-out dispatcher with load distribution
#[test]
fn c1635_fanout_dispatcher() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define EVT_FANOUT_MAX 16
#define EVT_FANOUT_QUEUE_SIZE 64

typedef struct {
    int handler_id;
    int active;
    int processed;
    int capacity;
    int current_load;
} evt_fanout_handler_t;

typedef struct {
    evt_fanout_handler_t handlers[EVT_FANOUT_MAX];
    int handler_count;
    int next_handler;
    int total_dispatched;
    int strategy;
} evt_fanout_t;

void evt_fanout_init(evt_fanout_t *fo, int strategy) {
    fo->handler_count = 0;
    fo->next_handler = 0;
    fo->total_dispatched = 0;
    fo->strategy = strategy;
}

int evt_fanout_add_handler(evt_fanout_t *fo, int capacity) {
    if (fo->handler_count >= EVT_FANOUT_MAX) return -1;
    int idx = fo->handler_count;
    fo->handlers[idx].handler_id = idx;
    fo->handlers[idx].active = 1;
    fo->handlers[idx].processed = 0;
    fo->handlers[idx].capacity = capacity;
    fo->handlers[idx].current_load = 0;
    fo->handler_count++;
    return idx;
}

int evt_fanout_round_robin(evt_fanout_t *fo) {
    int attempts = 0;
    while (attempts < fo->handler_count) {
        int idx = fo->next_handler;
        fo->next_handler = (fo->next_handler + 1) % fo->handler_count;
        if (fo->handlers[idx].active && fo->handlers[idx].current_load < fo->handlers[idx].capacity) {
            return idx;
        }
        attempts++;
    }
    return -1;
}

int evt_fanout_least_loaded(evt_fanout_t *fo) {
    int best = -1;
    int min_load = 0x7FFFFFFF;
    int i;
    for (i = 0; i < fo->handler_count; i++) {
        if (fo->handlers[i].active && fo->handlers[i].current_load < min_load) {
            min_load = fo->handlers[i].current_load;
            best = i;
        }
    }
    return best;
}

int evt_fanout_dispatch(evt_fanout_t *fo) {
    int target;
    if (fo->strategy == 0) {
        target = evt_fanout_round_robin(fo);
    } else {
        target = evt_fanout_least_loaded(fo);
    }
    if (target < 0) return -1;
    fo->handlers[target].processed++;
    fo->handlers[target].current_load++;
    fo->total_dispatched++;
    return target;
}

int evt_fanout_complete(evt_fanout_t *fo, int handler_id) {
    if (handler_id < 0 || handler_id >= fo->handler_count) return -1;
    if (fo->handlers[handler_id].current_load > 0) {
        fo->handlers[handler_id].current_load--;
    }
    return 0;
}

int evt_fanout_test(void) {
    evt_fanout_t fo;
    evt_fanout_init(&fo, 0);
    evt_fanout_add_handler(&fo, 10);
    evt_fanout_add_handler(&fo, 10);
    evt_fanout_dispatch(&fo);
    evt_fanout_dispatch(&fo);
    if (fo.handlers[0].processed != 1) return -1;
    if (fo.handlers[1].processed != 1) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1635: Fan-out dispatcher should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1635: Output should not be empty");
    assert!(code.contains("fn evt_fanout_init"), "C1635: Should contain evt_fanout_init");
    assert!(code.contains("fn evt_fanout_dispatch"), "C1635: Should contain evt_fanout_dispatch");
    assert!(code.contains("fn evt_fanout_least_loaded"), "C1635: Should contain evt_fanout_least_loaded");
}

// ============================================================================
// C1636-C1640: Event Sourcing
// ============================================================================

/// C1636: Event store with append-only log
#[test]
fn c1636_event_store_append() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define EVT_STORE_MAX 512
#define EVT_STORE_DATA_LEN 64

typedef struct {
    uint64_t sequence;
    uint32_t timestamp;
    int event_type;
    int aggregate_id;
    char data[EVT_STORE_DATA_LEN];
    int data_len;
} evt_store_event_t;

typedef struct {
    evt_store_event_t events[EVT_STORE_MAX];
    uint64_t next_sequence;
    int count;
    uint32_t current_time;
} evt_store_t;

void evt_store_init(evt_store_t *store) {
    store->next_sequence = 1;
    store->count = 0;
    store->current_time = 0;
}

int evt_store_append(evt_store_t *store, int event_type, int aggregate_id,
                     const char *data, int data_len) {
    if (store->count >= EVT_STORE_MAX) return -1;
    if (data_len > EVT_STORE_DATA_LEN) return -2;
    int idx = store->count;
    store->events[idx].sequence = store->next_sequence++;
    store->events[idx].timestamp = store->current_time;
    store->events[idx].event_type = event_type;
    store->events[idx].aggregate_id = aggregate_id;
    store->events[idx].data_len = data_len;
    int i;
    for (i = 0; i < data_len; i++) {
        store->events[idx].data[i] = data[i];
    }
    store->count++;
    return idx;
}

int evt_store_count_by_aggregate(evt_store_t *store, int aggregate_id) {
    int count = 0;
    int i;
    for (i = 0; i < store->count; i++) {
        if (store->events[i].aggregate_id == aggregate_id) {
            count++;
        }
    }
    return count;
}

uint64_t evt_store_last_sequence(evt_store_t *store) {
    if (store->count == 0) return 0;
    return store->events[store->count - 1].sequence;
}

int evt_store_test(void) {
    evt_store_t store;
    evt_store_init(&store);
    evt_store_append(&store, 1, 100, "create", 6);
    evt_store_append(&store, 2, 100, "update", 6);
    evt_store_append(&store, 1, 200, "create", 6);
    if (evt_store_count_by_aggregate(&store, 100) != 2) return -1;
    if (evt_store_last_sequence(&store) != 3) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1636: Event store append should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1636: Output should not be empty");
    assert!(code.contains("fn evt_store_init"), "C1636: Should contain evt_store_init");
    assert!(code.contains("fn evt_store_append"), "C1636: Should contain evt_store_append");
    assert!(code.contains("fn evt_store_last_sequence"), "C1636: Should contain evt_store_last_sequence");
}

/// C1637: Event replay from stored log
#[test]
fn c1637_event_replay() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define EVT_REPLAY_MAX 256

typedef struct {
    uint64_t sequence;
    int event_type;
    int value;
} evt_replay_entry_t;

typedef struct {
    evt_replay_entry_t log[EVT_REPLAY_MAX];
    int log_count;
    uint64_t next_seq;
} evt_replay_log_t;

typedef struct {
    int balance;
    int credit_count;
    int debit_count;
    uint64_t last_applied;
} evt_replay_state_t;

void evt_replay_log_init(evt_replay_log_t *log) {
    log->log_count = 0;
    log->next_seq = 1;
}

void evt_replay_state_init(evt_replay_state_t *state) {
    state->balance = 0;
    state->credit_count = 0;
    state->debit_count = 0;
    state->last_applied = 0;
}

int evt_replay_record(evt_replay_log_t *log, int event_type, int value) {
    if (log->log_count >= EVT_REPLAY_MAX) return -1;
    int idx = log->log_count;
    log->log[idx].sequence = log->next_seq++;
    log->log[idx].event_type = event_type;
    log->log[idx].value = value;
    log->log_count++;
    return 0;
}

void evt_replay_apply_event(evt_replay_state_t *state, evt_replay_entry_t *entry) {
    if (entry->event_type == 1) {
        state->balance += entry->value;
        state->credit_count++;
    } else if (entry->event_type == 2) {
        state->balance -= entry->value;
        state->debit_count++;
    }
    state->last_applied = entry->sequence;
}

int evt_replay_all(evt_replay_log_t *log, evt_replay_state_t *state) {
    int applied = 0;
    int i;
    for (i = 0; i < log->log_count; i++) {
        if (log->log[i].sequence > state->last_applied) {
            evt_replay_apply_event(state, &log->log[i]);
            applied++;
        }
    }
    return applied;
}

int evt_replay_test(void) {
    evt_replay_log_t log;
    evt_replay_state_t state;
    evt_replay_log_init(&log);
    evt_replay_state_init(&state);
    evt_replay_record(&log, 1, 100);
    evt_replay_record(&log, 1, 50);
    evt_replay_record(&log, 2, 30);
    int applied = evt_replay_all(&log, &state);
    if (applied != 3) return -1;
    if (state.balance != 120) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1637: Event replay should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1637: Output should not be empty");
    assert!(code.contains("fn evt_replay_log_init"), "C1637: Should contain evt_replay_log_init");
    assert!(code.contains("fn evt_replay_apply_event"), "C1637: Should contain evt_replay_apply_event");
    assert!(code.contains("fn evt_replay_all"), "C1637: Should contain evt_replay_all");
}

/// C1638: Snapshot creation from aggregate state
#[test]
fn c1638_snapshot_creation() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define EVT_SNAP_MAX 32
#define EVT_SNAP_DATA_LEN 128

typedef struct {
    uint64_t sequence;
    int aggregate_id;
    int state_value;
    int event_count;
    uint32_t timestamp;
} evt_snapshot_t;

typedef struct {
    evt_snapshot_t snapshots[EVT_SNAP_MAX];
    int count;
    int snapshot_interval;
} evt_snapshot_store_t;

void evt_snap_init(evt_snapshot_store_t *store, int interval) {
    store->count = 0;
    store->snapshot_interval = interval;
}

int evt_snap_should_snapshot(evt_snapshot_store_t *store, int events_since_last) {
    return events_since_last >= store->snapshot_interval;
}

int evt_snap_create(evt_snapshot_store_t *store, uint64_t sequence,
                    int aggregate_id, int state_value, int event_count, uint32_t ts) {
    if (store->count >= EVT_SNAP_MAX) return -1;
    int idx = store->count;
    store->snapshots[idx].sequence = sequence;
    store->snapshots[idx].aggregate_id = aggregate_id;
    store->snapshots[idx].state_value = state_value;
    store->snapshots[idx].event_count = event_count;
    store->snapshots[idx].timestamp = ts;
    store->count++;
    return idx;
}

int evt_snap_find_latest(evt_snapshot_store_t *store, int aggregate_id) {
    int best = -1;
    uint64_t best_seq = 0;
    int i;
    for (i = 0; i < store->count; i++) {
        if (store->snapshots[i].aggregate_id == aggregate_id &&
            store->snapshots[i].sequence > best_seq) {
            best = i;
            best_seq = store->snapshots[i].sequence;
        }
    }
    return best;
}

int evt_snap_test(void) {
    evt_snapshot_store_t store;
    evt_snap_init(&store, 10);
    evt_snap_create(&store, 10, 1, 500, 10, 1000);
    evt_snap_create(&store, 20, 1, 800, 20, 2000);
    evt_snap_create(&store, 15, 2, 300, 15, 1500);
    int latest = evt_snap_find_latest(&store, 1);
    if (latest != 1) return -1;
    if (store.snapshots[latest].state_value != 800) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1638: Snapshot creation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1638: Output should not be empty");
    assert!(code.contains("fn evt_snap_init"), "C1638: Should contain evt_snap_init");
    assert!(code.contains("fn evt_snap_create"), "C1638: Should contain evt_snap_create");
    assert!(code.contains("fn evt_snap_find_latest"), "C1638: Should contain evt_snap_find_latest");
}

/// C1639: Event projection that builds read models from events
#[test]
fn c1639_event_projection() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define EVT_PROJ_MAX_ENTITIES 64
#define EVT_PROJ_MAX_EVENTS 256

typedef struct {
    int entity_id;
    int total_credits;
    int total_debits;
    int net_balance;
    int event_count;
} evt_proj_entity_t;

typedef struct {
    evt_proj_entity_t entities[EVT_PROJ_MAX_ENTITIES];
    int entity_count;
    uint64_t last_processed;
} evt_proj_t;

void evt_proj_init(evt_proj_t *proj) {
    proj->entity_count = 0;
    proj->last_processed = 0;
}

int evt_proj_find_entity(evt_proj_t *proj, int entity_id) {
    int i;
    for (i = 0; i < proj->entity_count; i++) {
        if (proj->entities[i].entity_id == entity_id) return i;
    }
    return -1;
}

int evt_proj_ensure_entity(evt_proj_t *proj, int entity_id) {
    int idx = evt_proj_find_entity(proj, entity_id);
    if (idx >= 0) return idx;
    if (proj->entity_count >= EVT_PROJ_MAX_ENTITIES) return -1;
    idx = proj->entity_count;
    proj->entities[idx].entity_id = entity_id;
    proj->entities[idx].total_credits = 0;
    proj->entities[idx].total_debits = 0;
    proj->entities[idx].net_balance = 0;
    proj->entities[idx].event_count = 0;
    proj->entity_count++;
    return idx;
}

int evt_proj_apply(evt_proj_t *proj, uint64_t seq, int entity_id, int event_type, int amount) {
    if (seq <= proj->last_processed) return 0;
    int idx = evt_proj_ensure_entity(proj, entity_id);
    if (idx < 0) return -1;
    if (event_type == 1) {
        proj->entities[idx].total_credits += amount;
        proj->entities[idx].net_balance += amount;
    } else if (event_type == 2) {
        proj->entities[idx].total_debits += amount;
        proj->entities[idx].net_balance -= amount;
    }
    proj->entities[idx].event_count++;
    proj->last_processed = seq;
    return 1;
}

int evt_proj_test(void) {
    evt_proj_t proj;
    evt_proj_init(&proj);
    evt_proj_apply(&proj, 1, 10, 1, 100);
    evt_proj_apply(&proj, 2, 10, 2, 30);
    evt_proj_apply(&proj, 3, 20, 1, 50);
    int idx = evt_proj_find_entity(&proj, 10);
    if (idx < 0) return -1;
    if (proj.entities[idx].net_balance != 70) return -2;
    if (proj.entity_count != 2) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1639: Event projection should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1639: Output should not be empty");
    assert!(code.contains("fn evt_proj_init"), "C1639: Should contain evt_proj_init");
    assert!(code.contains("fn evt_proj_apply"), "C1639: Should contain evt_proj_apply");
    assert!(code.contains("fn evt_proj_find_entity"), "C1639: Should contain evt_proj_find_entity");
}

/// C1640: Command handler that validates and produces events
#[test]
fn c1640_command_handler() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define EVT_CMD_MAX_EVENTS 64
#define EVT_CMD_OK      0
#define EVT_CMD_INVALID -1
#define EVT_CMD_REJECTED -2

typedef struct {
    int command_type;
    int entity_id;
    int amount;
    uint32_t timestamp;
} evt_command_t;

typedef struct {
    uint64_t sequence;
    int event_type;
    int entity_id;
    int amount;
} evt_cmd_event_t;

typedef struct {
    evt_cmd_event_t events[EVT_CMD_MAX_EVENTS];
    int event_count;
    uint64_t next_seq;
    int accepted;
    int rejected;
} evt_cmd_handler_t;

void evt_cmd_init(evt_cmd_handler_t *handler) {
    handler->event_count = 0;
    handler->next_seq = 1;
    handler->accepted = 0;
    handler->rejected = 0;
}

int evt_cmd_validate(evt_command_t *cmd) {
    if (cmd->entity_id <= 0) return EVT_CMD_INVALID;
    if (cmd->amount < 0) return EVT_CMD_INVALID;
    if (cmd->command_type < 1 || cmd->command_type > 3) return EVT_CMD_INVALID;
    return EVT_CMD_OK;
}

int evt_cmd_emit(evt_cmd_handler_t *handler, int event_type, int entity_id, int amount) {
    if (handler->event_count >= EVT_CMD_MAX_EVENTS) return -1;
    int idx = handler->event_count;
    handler->events[idx].sequence = handler->next_seq++;
    handler->events[idx].event_type = event_type;
    handler->events[idx].entity_id = entity_id;
    handler->events[idx].amount = amount;
    handler->event_count++;
    return 0;
}

int evt_cmd_handle(evt_cmd_handler_t *handler, evt_command_t *cmd) {
    int valid = evt_cmd_validate(cmd);
    if (valid != EVT_CMD_OK) {
        handler->rejected++;
        return valid;
    }
    if (cmd->command_type == 1) {
        evt_cmd_emit(handler, 10, cmd->entity_id, cmd->amount);
    } else if (cmd->command_type == 2) {
        evt_cmd_emit(handler, 20, cmd->entity_id, cmd->amount);
    } else if (cmd->command_type == 3) {
        evt_cmd_emit(handler, 10, cmd->entity_id, cmd->amount);
        evt_cmd_emit(handler, 20, cmd->entity_id, cmd->amount);
    }
    handler->accepted++;
    return EVT_CMD_OK;
}

int evt_cmd_test(void) {
    evt_cmd_handler_t handler;
    evt_cmd_init(&handler);
    evt_command_t cmd1;
    cmd1.command_type = 1;
    cmd1.entity_id = 5;
    cmd1.amount = 100;
    cmd1.timestamp = 1000;
    evt_cmd_handle(&handler, &cmd1);
    evt_command_t cmd2;
    cmd2.command_type = 1;
    cmd2.entity_id = -1;
    cmd2.amount = 50;
    cmd2.timestamp = 2000;
    evt_cmd_handle(&handler, &cmd2);
    if (handler.accepted != 1) return -1;
    if (handler.rejected != 1) return -2;
    if (handler.event_count != 1) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1640: Command handler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1640: Output should not be empty");
    assert!(code.contains("fn evt_cmd_init"), "C1640: Should contain evt_cmd_init");
    assert!(code.contains("fn evt_cmd_handle"), "C1640: Should contain evt_cmd_handle");
    assert!(code.contains("fn evt_cmd_validate"), "C1640: Should contain evt_cmd_validate");
}

// ============================================================================
// C1641-C1645: Reactive Streams
// ============================================================================

/// C1641: Observable creation with value emission
#[test]
fn c1641_observable_creation() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define EVT_OBS_BUFFER_MAX 128
#define EVT_OBS_STATE_ACTIVE   0
#define EVT_OBS_STATE_COMPLETE 1
#define EVT_OBS_STATE_ERROR    2

typedef struct {
    int values[EVT_OBS_BUFFER_MAX];
    int count;
    int state;
    int error_code;
    int subscribers;
} evt_observable_t;

void evt_obs_create(evt_observable_t *obs) {
    obs->count = 0;
    obs->state = EVT_OBS_STATE_ACTIVE;
    obs->error_code = 0;
    obs->subscribers = 0;
}

int evt_obs_emit(evt_observable_t *obs, int value) {
    if (obs->state != EVT_OBS_STATE_ACTIVE) return -1;
    if (obs->count >= EVT_OBS_BUFFER_MAX) return -2;
    obs->values[obs->count] = value;
    obs->count++;
    return 0;
}

void evt_obs_complete(evt_observable_t *obs) {
    obs->state = EVT_OBS_STATE_COMPLETE;
}

void evt_obs_error(evt_observable_t *obs, int error_code) {
    obs->state = EVT_OBS_STATE_ERROR;
    obs->error_code = error_code;
}

int evt_obs_subscribe(evt_observable_t *obs) {
    obs->subscribers++;
    return obs->subscribers;
}

int evt_obs_get(evt_observable_t *obs, int index) {
    if (index < 0 || index >= obs->count) return -1;
    return obs->values[index];
}

int evt_obs_test(void) {
    evt_observable_t obs;
    evt_obs_create(&obs);
    evt_obs_subscribe(&obs);
    evt_obs_emit(&obs, 10);
    evt_obs_emit(&obs, 20);
    evt_obs_emit(&obs, 30);
    evt_obs_complete(&obs);
    if (obs.count != 3) return -1;
    if (evt_obs_get(&obs, 1) != 20) return -2;
    if (obs.state != EVT_OBS_STATE_COMPLETE) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1641: Observable creation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1641: Output should not be empty");
    assert!(code.contains("fn evt_obs_create"), "C1641: Should contain evt_obs_create");
    assert!(code.contains("fn evt_obs_emit"), "C1641: Should contain evt_obs_emit");
    assert!(code.contains("fn evt_obs_complete"), "C1641: Should contain evt_obs_complete");
}

/// C1642: Map and filter operators for stream transformation
#[test]
fn c1642_map_filter_operators() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define EVT_STREAM_MAX 128

typedef struct {
    int values[EVT_STREAM_MAX];
    int count;
} evt_stream_t;

void evt_stream_init(evt_stream_t *s) {
    s->count = 0;
}

int evt_stream_push(evt_stream_t *s, int value) {
    if (s->count >= EVT_STREAM_MAX) return -1;
    s->values[s->count] = value;
    s->count++;
    return 0;
}

int evt_stream_map_double(evt_stream_t *src, evt_stream_t *dst) {
    evt_stream_init(dst);
    int i;
    for (i = 0; i < src->count; i++) {
        if (dst->count >= EVT_STREAM_MAX) return -1;
        dst->values[dst->count] = src->values[i] * 2;
        dst->count++;
    }
    return dst->count;
}

int evt_stream_filter_positive(evt_stream_t *src, evt_stream_t *dst) {
    evt_stream_init(dst);
    int i;
    for (i = 0; i < src->count; i++) {
        if (src->values[i] > 0) {
            if (dst->count >= EVT_STREAM_MAX) return -1;
            dst->values[dst->count] = src->values[i];
            dst->count++;
        }
    }
    return dst->count;
}

int evt_stream_map_filter(evt_stream_t *src, evt_stream_t *dst, int min_val) {
    evt_stream_init(dst);
    int i;
    for (i = 0; i < src->count; i++) {
        int doubled = src->values[i] * 2;
        if (doubled >= min_val) {
            if (dst->count >= EVT_STREAM_MAX) return -1;
            dst->values[dst->count] = doubled;
            dst->count++;
        }
    }
    return dst->count;
}

int evt_stream_test(void) {
    evt_stream_t src;
    evt_stream_t dst;
    evt_stream_init(&src);
    evt_stream_push(&src, -5);
    evt_stream_push(&src, 3);
    evt_stream_push(&src, 10);
    evt_stream_push(&src, -2);
    evt_stream_push(&src, 7);
    evt_stream_filter_positive(&src, &dst);
    if (dst.count != 3) return -1;
    evt_stream_map_double(&dst, &src);
    if (src.values[0] != 6) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1642: Map/filter operators should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1642: Output should not be empty");
    assert!(code.contains("fn evt_stream_init"), "C1642: Should contain evt_stream_init");
    assert!(code.contains("fn evt_stream_map_double"), "C1642: Should contain evt_stream_map_double");
    assert!(code.contains("fn evt_stream_filter_positive"), "C1642: Should contain evt_stream_filter_positive");
}

/// C1643: Merge multiple streams into one
#[test]
fn c1643_merge_streams() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define EVT_MERGE_MAX 128
#define EVT_MERGE_SOURCES 4

typedef struct {
    int values[EVT_MERGE_MAX];
    int count;
    int source_id;
} evt_merge_source_t;

typedef struct {
    evt_merge_source_t sources[EVT_MERGE_SOURCES];
    int source_count;
    int output[EVT_MERGE_MAX];
    int output_count;
} evt_merge_t;

void evt_merge_init(evt_merge_t *m) {
    m->source_count = 0;
    m->output_count = 0;
}

int evt_merge_add_source(evt_merge_t *m) {
    if (m->source_count >= EVT_MERGE_SOURCES) return -1;
    int idx = m->source_count;
    m->sources[idx].count = 0;
    m->sources[idx].source_id = idx;
    m->source_count++;
    return idx;
}

int evt_merge_push_to_source(evt_merge_t *m, int source_id, int value) {
    if (source_id < 0 || source_id >= m->source_count) return -1;
    evt_merge_source_t *s = &m->sources[source_id];
    if (s->count >= EVT_MERGE_MAX) return -2;
    s->values[s->count] = value;
    s->count++;
    return 0;
}

int evt_merge_execute(evt_merge_t *m) {
    m->output_count = 0;
    int i;
    int j;
    for (i = 0; i < m->source_count; i++) {
        for (j = 0; j < m->sources[i].count; j++) {
            if (m->output_count >= EVT_MERGE_MAX) return m->output_count;
            m->output[m->output_count] = m->sources[i].values[j];
            m->output_count++;
        }
    }
    return m->output_count;
}

int evt_merge_test(void) {
    evt_merge_t m;
    evt_merge_init(&m);
    int s0 = evt_merge_add_source(&m);
    int s1 = evt_merge_add_source(&m);
    evt_merge_push_to_source(&m, s0, 10);
    evt_merge_push_to_source(&m, s0, 20);
    evt_merge_push_to_source(&m, s1, 30);
    int total = evt_merge_execute(&m);
    if (total != 3) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1643: Merge streams should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1643: Output should not be empty");
    assert!(code.contains("fn evt_merge_init"), "C1643: Should contain evt_merge_init");
    assert!(code.contains("fn evt_merge_add_source"), "C1643: Should contain evt_merge_add_source");
    assert!(code.contains("fn evt_merge_execute"), "C1643: Should contain evt_merge_execute");
}

/// C1644: Buffer strategy with time and count-based flushing
#[test]
fn c1644_buffer_strategy() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define EVT_BUF_MAX 256
#define EVT_BUF_STRATEGY_COUNT 0
#define EVT_BUF_STRATEGY_TIME  1

typedef struct {
    int items[EVT_BUF_MAX];
    int count;
    int max_count;
    uint32_t window_start;
    uint32_t window_duration;
    int strategy;
    int flush_count;
    int total_items;
} evt_buffer_t;

void evt_buf_init(evt_buffer_t *buf, int strategy, int max_count, uint32_t window_ms) {
    buf->count = 0;
    buf->max_count = max_count;
    buf->window_start = 0;
    buf->window_duration = window_ms;
    buf->strategy = strategy;
    buf->flush_count = 0;
    buf->total_items = 0;
}

int evt_buf_should_flush(evt_buffer_t *buf, uint32_t current_time) {
    if (buf->strategy == EVT_BUF_STRATEGY_COUNT) {
        return buf->count >= buf->max_count;
    } else if (buf->strategy == EVT_BUF_STRATEGY_TIME) {
        return (current_time - buf->window_start) >= buf->window_duration;
    }
    return 0;
}

int evt_buf_flush(evt_buffer_t *buf) {
    int flushed = buf->count;
    buf->count = 0;
    buf->flush_count++;
    return flushed;
}

int evt_buf_add(evt_buffer_t *buf, int item, uint32_t timestamp) {
    if (buf->count == 0) {
        buf->window_start = timestamp;
    }
    if (buf->count >= EVT_BUF_MAX) return -1;
    buf->items[buf->count] = item;
    buf->count++;
    buf->total_items++;
    if (evt_buf_should_flush(buf, timestamp)) {
        return evt_buf_flush(buf);
    }
    return 0;
}

int evt_buf_test(void) {
    evt_buffer_t buf;
    evt_buf_init(&buf, EVT_BUF_STRATEGY_COUNT, 3, 0);
    evt_buf_add(&buf, 10, 0);
    evt_buf_add(&buf, 20, 1);
    int result = evt_buf_add(&buf, 30, 2);
    if (result != 3) return -1;
    if (buf.count != 0) return -2;
    if (buf.flush_count != 1) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1644: Buffer strategy should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1644: Output should not be empty");
    assert!(code.contains("fn evt_buf_init"), "C1644: Should contain evt_buf_init");
    assert!(code.contains("fn evt_buf_add"), "C1644: Should contain evt_buf_add");
    assert!(code.contains("fn evt_buf_should_flush"), "C1644: Should contain evt_buf_should_flush");
}

/// C1645: Error propagation in reactive pipelines
#[test]
fn c1645_error_propagation() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define EVT_ERR_MAX 64
#define EVT_ERR_NONE     0
#define EVT_ERR_OVERFLOW 1
#define EVT_ERR_INVALID  2
#define EVT_ERR_TIMEOUT  3

typedef struct {
    int error_type;
    int source_stage;
    int value;
} evt_error_t;

typedef struct {
    int values[EVT_ERR_MAX];
    int count;
    evt_error_t errors[EVT_ERR_MAX];
    int error_count;
    int completed;
} evt_err_pipeline_t;

void evt_err_init(evt_err_pipeline_t *pipe) {
    pipe->count = 0;
    pipe->error_count = 0;
    pipe->completed = 0;
}

int evt_err_record(evt_err_pipeline_t *pipe, int error_type, int stage, int value) {
    if (pipe->error_count >= EVT_ERR_MAX) return -1;
    int idx = pipe->error_count;
    pipe->errors[idx].error_type = error_type;
    pipe->errors[idx].source_stage = stage;
    pipe->errors[idx].value = value;
    pipe->error_count++;
    return idx;
}

int evt_err_process(evt_err_pipeline_t *pipe, int value, int stage) {
    if (value < 0) {
        evt_err_record(pipe, EVT_ERR_INVALID, stage, value);
        return -1;
    }
    if (pipe->count >= EVT_ERR_MAX) {
        evt_err_record(pipe, EVT_ERR_OVERFLOW, stage, value);
        return -2;
    }
    pipe->values[pipe->count] = value;
    pipe->count++;
    return 0;
}

int evt_err_retry(evt_err_pipeline_t *pipe, int error_idx) {
    if (error_idx < 0 || error_idx >= pipe->error_count) return -1;
    int value = pipe->errors[error_idx].value;
    if (value < 0) value = -value;
    return evt_err_process(pipe, value, pipe->errors[error_idx].source_stage);
}

int evt_err_has_errors(evt_err_pipeline_t *pipe) {
    return pipe->error_count > 0;
}

int evt_err_test(void) {
    evt_err_pipeline_t pipe;
    evt_err_init(&pipe);
    evt_err_process(&pipe, 10, 0);
    evt_err_process(&pipe, -5, 1);
    evt_err_process(&pipe, 20, 2);
    if (pipe.count != 2) return -1;
    if (pipe.error_count != 1) return -2;
    evt_err_retry(&pipe, 0);
    if (pipe.count != 3) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1645: Error propagation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1645: Output should not be empty");
    assert!(code.contains("fn evt_err_init"), "C1645: Should contain evt_err_init");
    assert!(code.contains("fn evt_err_process"), "C1645: Should contain evt_err_process");
    assert!(code.contains("fn evt_err_retry"), "C1645: Should contain evt_err_retry");
}

// ============================================================================
// C1646-C1650: Signal Handling
// ============================================================================

/// C1646: Signal registration with handler table
#[test]
fn c1646_signal_registration() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define EVT_SIG_MAX 32
#define EVT_SIG_DEFAULT 0
#define EVT_SIG_IGNORE  1
#define EVT_SIG_CUSTOM  2

typedef struct {
    int signal_num;
    int action;
    int handler_id;
    int triggered_count;
    int blocked;
} evt_sig_entry_t;

typedef struct {
    evt_sig_entry_t signals[EVT_SIG_MAX];
    int count;
    int next_handler_id;
    int pending_count;
} evt_sig_table_t;

void evt_sig_init(evt_sig_table_t *table) {
    table->count = 0;
    table->next_handler_id = 1;
    table->pending_count = 0;
}

int evt_sig_register(evt_sig_table_t *table, int signal_num, int action) {
    int i;
    for (i = 0; i < table->count; i++) {
        if (table->signals[i].signal_num == signal_num) {
            table->signals[i].action = action;
            if (action == EVT_SIG_CUSTOM) {
                table->signals[i].handler_id = table->next_handler_id++;
            }
            return table->signals[i].handler_id;
        }
    }
    if (table->count >= EVT_SIG_MAX) return -1;
    int idx = table->count;
    table->signals[idx].signal_num = signal_num;
    table->signals[idx].action = action;
    table->signals[idx].triggered_count = 0;
    table->signals[idx].blocked = 0;
    if (action == EVT_SIG_CUSTOM) {
        table->signals[idx].handler_id = table->next_handler_id++;
    } else {
        table->signals[idx].handler_id = 0;
    }
    table->count++;
    return table->signals[idx].handler_id;
}

int evt_sig_raise(evt_sig_table_t *table, int signal_num) {
    int i;
    for (i = 0; i < table->count; i++) {
        if (table->signals[i].signal_num == signal_num) {
            if (table->signals[i].blocked) {
                table->pending_count++;
                return 0;
            }
            if (table->signals[i].action == EVT_SIG_IGNORE) return 0;
            table->signals[i].triggered_count++;
            return table->signals[i].handler_id;
        }
    }
    return -1;
}

int evt_sig_test(void) {
    evt_sig_table_t table;
    evt_sig_init(&table);
    evt_sig_register(&table, 2, EVT_SIG_CUSTOM);
    evt_sig_register(&table, 15, EVT_SIG_IGNORE);
    int hid = evt_sig_raise(&table, 2);
    if (hid <= 0) return -1;
    int ign = evt_sig_raise(&table, 15);
    if (ign != 0) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1646: Signal registration should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1646: Output should not be empty");
    assert!(code.contains("fn evt_sig_init"), "C1646: Should contain evt_sig_init");
    assert!(code.contains("fn evt_sig_register"), "C1646: Should contain evt_sig_register");
    assert!(code.contains("fn evt_sig_raise"), "C1646: Should contain evt_sig_raise");
}

/// C1647: Signal masking with block/unblock sets
#[test]
fn c1647_signal_masking() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define EVT_MASK_BITS 32

typedef struct {
    uint32_t mask;
    uint32_t pending;
    int blocked_count;
    int delivered_count;
} evt_sigmask_t;

void evt_mask_init(evt_sigmask_t *sm) {
    sm->mask = 0;
    sm->pending = 0;
    sm->blocked_count = 0;
    sm->delivered_count = 0;
}

void evt_mask_block(evt_sigmask_t *sm, int signum) {
    if (signum >= 0 && signum < EVT_MASK_BITS) {
        sm->mask |= (1u << signum);
    }
}

void evt_mask_unblock(evt_sigmask_t *sm, int signum) {
    if (signum >= 0 && signum < EVT_MASK_BITS) {
        sm->mask &= ~(1u << signum);
    }
}

int evt_mask_is_blocked(evt_sigmask_t *sm, int signum) {
    if (signum < 0 || signum >= EVT_MASK_BITS) return 0;
    return (sm->mask >> signum) & 1;
}

int evt_mask_deliver(evt_sigmask_t *sm, int signum) {
    if (signum < 0 || signum >= EVT_MASK_BITS) return -1;
    if (evt_mask_is_blocked(sm, signum)) {
        sm->pending |= (1u << signum);
        sm->blocked_count++;
        return 0;
    }
    sm->delivered_count++;
    return 1;
}

int evt_mask_flush_pending(evt_sigmask_t *sm) {
    int flushed = 0;
    int i;
    for (i = 0; i < EVT_MASK_BITS; i++) {
        if ((sm->pending >> i) & 1) {
            if (!evt_mask_is_blocked(sm, i)) {
                sm->pending &= ~(1u << i);
                sm->delivered_count++;
                flushed++;
            }
        }
    }
    return flushed;
}

int evt_mask_test(void) {
    evt_sigmask_t sm;
    evt_mask_init(&sm);
    evt_mask_block(&sm, 2);
    evt_mask_deliver(&sm, 2);
    if (sm.blocked_count != 1) return -1;
    evt_mask_deliver(&sm, 5);
    if (sm.delivered_count != 1) return -2;
    evt_mask_unblock(&sm, 2);
    int flushed = evt_mask_flush_pending(&sm);
    if (flushed != 1) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1647: Signal masking should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1647: Output should not be empty");
    assert!(code.contains("fn evt_mask_init"), "C1647: Should contain evt_mask_init");
    assert!(code.contains("fn evt_mask_block"), "C1647: Should contain evt_mask_block");
    assert!(code.contains("fn evt_mask_flush_pending"), "C1647: Should contain evt_mask_flush_pending");
}

/// C1648: Signal queue with FIFO delivery ordering
#[test]
fn c1648_signal_queue() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define EVT_SIGQ_MAX 64

typedef struct {
    int signal_num;
    int value;
    uint32_t timestamp;
} evt_sigq_entry_t;

typedef struct {
    evt_sigq_entry_t queue[EVT_SIGQ_MAX];
    int head;
    int tail;
    int count;
    int overflow_count;
} evt_sigqueue_t;

void evt_sigq_init(evt_sigqueue_t *sq) {
    sq->head = 0;
    sq->tail = 0;
    sq->count = 0;
    sq->overflow_count = 0;
}

int evt_sigq_enqueue(evt_sigqueue_t *sq, int signal_num, int value, uint32_t timestamp) {
    if (sq->count >= EVT_SIGQ_MAX) {
        sq->overflow_count++;
        return -1;
    }
    sq->queue[sq->tail].signal_num = signal_num;
    sq->queue[sq->tail].value = value;
    sq->queue[sq->tail].timestamp = timestamp;
    sq->tail = (sq->tail + 1) % EVT_SIGQ_MAX;
    sq->count++;
    return 0;
}

int evt_sigq_dequeue(evt_sigqueue_t *sq, evt_sigq_entry_t *out) {
    if (sq->count == 0) return -1;
    *out = sq->queue[sq->head];
    sq->head = (sq->head + 1) % EVT_SIGQ_MAX;
    sq->count--;
    return 0;
}

int evt_sigq_peek(evt_sigqueue_t *sq, evt_sigq_entry_t *out) {
    if (sq->count == 0) return -1;
    *out = sq->queue[sq->head];
    return 0;
}

int evt_sigq_drain_signal(evt_sigqueue_t *sq, int signal_num, int *count) {
    *count = 0;
    evt_sigqueue_t temp;
    evt_sigq_init(&temp);
    while (sq->count > 0) {
        evt_sigq_entry_t entry;
        evt_sigq_dequeue(sq, &entry);
        if (entry.signal_num == signal_num) {
            (*count)++;
        } else {
            evt_sigq_enqueue(&temp, entry.signal_num, entry.value, entry.timestamp);
        }
    }
    while (temp.count > 0) {
        evt_sigq_entry_t entry;
        evt_sigq_dequeue(&temp, &entry);
        evt_sigq_enqueue(sq, entry.signal_num, entry.value, entry.timestamp);
    }
    return *count;
}

int evt_sigq_test(void) {
    evt_sigqueue_t sq;
    evt_sigq_init(&sq);
    evt_sigq_enqueue(&sq, 2, 100, 1000);
    evt_sigq_enqueue(&sq, 15, 200, 2000);
    evt_sigq_enqueue(&sq, 2, 300, 3000);
    evt_sigq_entry_t out;
    evt_sigq_dequeue(&sq, &out);
    if (out.signal_num != 2) return -1;
    if (out.value != 100) return -2;
    int drained;
    evt_sigq_drain_signal(&sq, 2, &drained);
    if (drained != 1) return -3;
    if (sq.count != 1) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1648: Signal queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1648: Output should not be empty");
    assert!(code.contains("fn evt_sigq_init"), "C1648: Should contain evt_sigq_init");
    assert!(code.contains("fn evt_sigq_enqueue"), "C1648: Should contain evt_sigq_enqueue");
    assert!(code.contains("fn evt_sigq_dequeue"), "C1648: Should contain evt_sigq_dequeue");
}

/// C1649: Async-safe signal handler with volatile flag communication
#[test]
fn c1649_async_safe_handler() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define EVT_ASYNC_MAX_SIGNALS 16
#define EVT_ASYNC_FLAG_NONE    0
#define EVT_ASYNC_FLAG_PENDING 1
#define EVT_ASYNC_FLAG_HANDLED 2

typedef struct {
    int signal_num;
    int flag;
    int invocation_count;
} evt_async_sig_t;

typedef struct {
    evt_async_sig_t signals[EVT_ASYNC_MAX_SIGNALS];
    int count;
    int pipe_write_fd;
    int pipe_read_fd;
    int total_handled;
} evt_async_handler_t;

void evt_async_init(evt_async_handler_t *h) {
    h->count = 0;
    h->pipe_write_fd = -1;
    h->pipe_read_fd = -1;
    h->total_handled = 0;
}

int evt_async_register(evt_async_handler_t *h, int signal_num) {
    if (h->count >= EVT_ASYNC_MAX_SIGNALS) return -1;
    int idx = h->count;
    h->signals[idx].signal_num = signal_num;
    h->signals[idx].flag = EVT_ASYNC_FLAG_NONE;
    h->signals[idx].invocation_count = 0;
    h->count++;
    return idx;
}

int evt_async_set_pending(evt_async_handler_t *h, int signal_num) {
    int i;
    for (i = 0; i < h->count; i++) {
        if (h->signals[i].signal_num == signal_num) {
            h->signals[i].flag = EVT_ASYNC_FLAG_PENDING;
            h->signals[i].invocation_count++;
            return 0;
        }
    }
    return -1;
}

int evt_async_process_pending(evt_async_handler_t *h) {
    int processed = 0;
    int i;
    for (i = 0; i < h->count; i++) {
        if (h->signals[i].flag == EVT_ASYNC_FLAG_PENDING) {
            h->signals[i].flag = EVT_ASYNC_FLAG_HANDLED;
            h->total_handled++;
            processed++;
        }
    }
    return processed;
}

int evt_async_reset(evt_async_handler_t *h, int signal_num) {
    int i;
    for (i = 0; i < h->count; i++) {
        if (h->signals[i].signal_num == signal_num) {
            h->signals[i].flag = EVT_ASYNC_FLAG_NONE;
            return 0;
        }
    }
    return -1;
}

int evt_async_test(void) {
    evt_async_handler_t h;
    evt_async_init(&h);
    evt_async_register(&h, 2);
    evt_async_register(&h, 15);
    evt_async_set_pending(&h, 2);
    evt_async_set_pending(&h, 15);
    int processed = evt_async_process_pending(&h);
    if (processed != 2) return -1;
    if (h.total_handled != 2) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1649: Async-safe handler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1649: Output should not be empty");
    assert!(code.contains("fn evt_async_init"), "C1649: Should contain evt_async_init");
    assert!(code.contains("fn evt_async_set_pending"), "C1649: Should contain evt_async_set_pending");
    assert!(code.contains("fn evt_async_process_pending"), "C1649: Should contain evt_async_process_pending");
}

/// C1650: Signal forwarding with chain of handlers
#[test]
fn c1650_signal_forwarding() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define EVT_FWD_MAX_HANDLERS 16
#define EVT_FWD_MAX_CHAINS 8
#define EVT_FWD_ACTION_HANDLE  0
#define EVT_FWD_ACTION_FORWARD 1
#define EVT_FWD_ACTION_DROP    2

typedef struct {
    int handler_id;
    int action;
    int forward_to;
    int handled_count;
} evt_fwd_handler_t;

typedef struct {
    int signal_num;
    int first_handler;
    int handler_count;
} evt_fwd_chain_t;

typedef struct {
    evt_fwd_handler_t handlers[EVT_FWD_MAX_HANDLERS];
    int handler_count;
    evt_fwd_chain_t chains[EVT_FWD_MAX_CHAINS];
    int chain_count;
    int total_forwarded;
    int total_dropped;
} evt_fwd_table_t;

void evt_fwd_init(evt_fwd_table_t *table) {
    table->handler_count = 0;
    table->chain_count = 0;
    table->total_forwarded = 0;
    table->total_dropped = 0;
}

int evt_fwd_add_handler(evt_fwd_table_t *table, int action, int forward_to) {
    if (table->handler_count >= EVT_FWD_MAX_HANDLERS) return -1;
    int idx = table->handler_count;
    table->handlers[idx].handler_id = idx;
    table->handlers[idx].action = action;
    table->handlers[idx].forward_to = forward_to;
    table->handlers[idx].handled_count = 0;
    table->handler_count++;
    return idx;
}

int evt_fwd_add_chain(evt_fwd_table_t *table, int signal_num, int first_handler) {
    if (table->chain_count >= EVT_FWD_MAX_CHAINS) return -1;
    int idx = table->chain_count;
    table->chains[idx].signal_num = signal_num;
    table->chains[idx].first_handler = first_handler;
    table->chains[idx].handler_count = 0;
    table->chain_count++;
    return idx;
}

int evt_fwd_dispatch(evt_fwd_table_t *table, int signal_num) {
    int i;
    for (i = 0; i < table->chain_count; i++) {
        if (table->chains[i].signal_num == signal_num) {
            int hid = table->chains[i].first_handler;
            int depth = 0;
            while (hid >= 0 && hid < table->handler_count && depth < EVT_FWD_MAX_HANDLERS) {
                evt_fwd_handler_t *h = &table->handlers[hid];
                h->handled_count++;
                if (h->action == EVT_FWD_ACTION_HANDLE) {
                    return hid;
                } else if (h->action == EVT_FWD_ACTION_FORWARD) {
                    table->total_forwarded++;
                    hid = h->forward_to;
                } else {
                    table->total_dropped++;
                    return -2;
                }
                depth++;
            }
            return -3;
        }
    }
    return -1;
}

int evt_fwd_test(void) {
    evt_fwd_table_t table;
    evt_fwd_init(&table);
    int h0 = evt_fwd_add_handler(&table, EVT_FWD_ACTION_FORWARD, 1);
    int h1 = evt_fwd_add_handler(&table, EVT_FWD_ACTION_FORWARD, 2);
    int h2 = evt_fwd_add_handler(&table, EVT_FWD_ACTION_HANDLE, -1);
    evt_fwd_add_chain(&table, 9, h0);
    int result = evt_fwd_dispatch(&table, 9);
    if (result != 2) return -1;
    if (table.total_forwarded != 2) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1650: Signal forwarding should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1650: Output should not be empty");
    assert!(code.contains("fn evt_fwd_init"), "C1650: Should contain evt_fwd_init");
    assert!(code.contains("fn evt_fwd_add_handler"), "C1650: Should contain evt_fwd_add_handler");
    assert!(code.contains("fn evt_fwd_dispatch"), "C1650: Should contain evt_fwd_dispatch");
}
