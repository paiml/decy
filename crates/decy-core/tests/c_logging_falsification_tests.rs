//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1601-C1625: Logging & Tracing Systems -- ring buffer loggers, trace event
//! systems, log aggregation, performance counters, and diagnostic frameworks.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world logging and tracing patterns commonly
//! found in operating systems, observability frameworks, application servers,
//! and performance monitoring tools -- all expressed as valid C99.
//!
//! Organization:
//! - C1601-C1605: Ring buffer logging (circular buffer, level filtering, structured entry, rotation, flush)
//! - C1606-C1610: Trace event systems (trace point, span tracking, event correlation, sampling, context propagation)
//! - C1611-C1615: Log aggregation (merging, timestamp normalization, deduplication, pattern filter, rate limiting)
//! - C1616-C1620: Performance counters (hardware abstraction, overflow, hierarchy, snapshot, percentile)
//! - C1621-C1625: Diagnostic systems (health check, metric collection, alert threshold, diagnostic dump, crash reporter)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C1601-C1605: Ring Buffer Logging
// ============================================================================

/// C1601: Circular buffer logger with write/read pointers
#[test]
fn c1601_ring_buffer_logger() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define LOG_BUF_SIZE 1024
#define LOG_MSG_MAX 128

typedef struct {
    char buffer[LOG_BUF_SIZE];
    int write_pos;
    int read_pos;
    int count;
    int capacity;
    int overflows;
} log_ring_t;

void log_ring_init(log_ring_t *ring) {
    ring->write_pos = 0;
    ring->read_pos = 0;
    ring->count = 0;
    ring->capacity = LOG_BUF_SIZE;
    ring->overflows = 0;
    int i;
    for (i = 0; i < LOG_BUF_SIZE; i++) {
        ring->buffer[i] = 0;
    }
}

int log_ring_write(log_ring_t *ring, const char *msg, int len) {
    if (len <= 0 || len > LOG_MSG_MAX) return -1;
    if (ring->count + len + 1 > ring->capacity) {
        ring->overflows++;
        return -2;
    }
    int i;
    ring->buffer[ring->write_pos] = (char)len;
    ring->write_pos = (ring->write_pos + 1) % ring->capacity;
    for (i = 0; i < len; i++) {
        ring->buffer[ring->write_pos] = msg[i];
        ring->write_pos = (ring->write_pos + 1) % ring->capacity;
    }
    ring->count += len + 1;
    return 0;
}

int log_ring_read(log_ring_t *ring, char *out, int out_max) {
    if (ring->count <= 0) return -1;
    int len = (int)(unsigned char)ring->buffer[ring->read_pos];
    if (len > out_max) return -2;
    ring->read_pos = (ring->read_pos + 1) % ring->capacity;
    int i;
    for (i = 0; i < len; i++) {
        out[i] = ring->buffer[ring->read_pos];
        ring->read_pos = (ring->read_pos + 1) % ring->capacity;
    }
    ring->count -= len + 1;
    return len;
}

int log_ring_test(void) {
    log_ring_t ring;
    log_ring_init(&ring);
    const char *msg = "hello";
    int rc = log_ring_write(&ring, msg, 5);
    if (rc != 0) return -1;
    char buf[32];
    int n = log_ring_read(&ring, buf, 32);
    if (n != 5) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1601: Ring buffer logger should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1601: Output should not be empty");
    assert!(code.contains("fn log_ring_"), "C1601: Should contain log_ring_ functions");
}

/// C1602: Log level filtering with severity thresholds
#[test]
fn c1602_log_level_filtering() {
    let c_code = r##"
typedef unsigned int uint32_t;

#define LOG_LEVEL_TRACE  0
#define LOG_LEVEL_DEBUG  1
#define LOG_LEVEL_INFO   2
#define LOG_LEVEL_WARN   3
#define LOG_LEVEL_ERROR  4
#define LOG_LEVEL_FATAL  5
#define LOG_MAX_ENTRIES 256

typedef struct {
    int level;
    uint32_t timestamp;
    int source_id;
    char message[64];
} log_entry_t;

typedef struct {
    log_entry_t entries[LOG_MAX_ENTRIES];
    int count;
    int min_level;
    int dropped_count;
} log_filter_t;

void log_filter_init(log_filter_t *f, int min_level) {
    f->count = 0;
    f->min_level = min_level;
    f->dropped_count = 0;
}

int log_filter_accept(log_filter_t *f, int level) {
    if (level < f->min_level) {
        f->dropped_count++;
        return 0;
    }
    return 1;
}

int log_filter_add(log_filter_t *f, int level, uint32_t ts, int src) {
    if (!log_filter_accept(f, level)) return -1;
    if (f->count >= LOG_MAX_ENTRIES) return -2;
    int idx = f->count;
    f->entries[idx].level = level;
    f->entries[idx].timestamp = ts;
    f->entries[idx].source_id = src;
    f->count++;
    return idx;
}

int log_filter_count_by_level(log_filter_t *f, int level) {
    int c = 0;
    int i;
    for (i = 0; i < f->count; i++) {
        if (f->entries[i].level == level) c++;
    }
    return c;
}

int log_filter_test(void) {
    log_filter_t filt;
    log_filter_init(&filt, LOG_LEVEL_WARN);
    log_filter_add(&filt, LOG_LEVEL_DEBUG, 100, 1);
    log_filter_add(&filt, LOG_LEVEL_WARN, 200, 2);
    log_filter_add(&filt, LOG_LEVEL_ERROR, 300, 3);
    if (filt.count != 2) return -1;
    if (filt.dropped_count != 1) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1602: Log level filtering should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1602: Output should not be empty");
    assert!(code.contains("fn log_filter_"), "C1602: Should contain log_filter_ functions");
}

/// C1603: Structured log entry with field serialization
#[test]
fn c1603_structured_log_entry() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned long size_t;

#define LOG_FIELD_MAX 16
#define LOG_FIELD_INT    0
#define LOG_FIELD_STR    1
#define LOG_FIELD_FLOAT  2

typedef struct {
    int field_type;
    int key_len;
    char key[32];
    int int_val;
    char str_val[64];
} log_field_t;

typedef struct {
    uint32_t timestamp;
    int level;
    log_field_t fields[LOG_FIELD_MAX];
    int field_count;
} log_structured_t;

void log_structured_init(log_structured_t *entry, int level, uint32_t ts) {
    entry->level = level;
    entry->timestamp = ts;
    entry->field_count = 0;
}

int log_structured_add_int(log_structured_t *entry, const char *key, int val) {
    if (entry->field_count >= LOG_FIELD_MAX) return -1;
    int idx = entry->field_count;
    entry->fields[idx].field_type = LOG_FIELD_INT;
    entry->fields[idx].int_val = val;
    int i;
    for (i = 0; key[i] != 0 && i < 31; i++) {
        entry->fields[idx].key[i] = key[i];
    }
    entry->fields[idx].key[i] = 0;
    entry->fields[idx].key_len = i;
    entry->field_count++;
    return 0;
}

int log_structured_add_str(log_structured_t *entry, const char *key, const char *val) {
    if (entry->field_count >= LOG_FIELD_MAX) return -1;
    int idx = entry->field_count;
    entry->fields[idx].field_type = LOG_FIELD_STR;
    int i;
    for (i = 0; key[i] != 0 && i < 31; i++) {
        entry->fields[idx].key[i] = key[i];
    }
    entry->fields[idx].key[i] = 0;
    entry->fields[idx].key_len = i;
    for (i = 0; val[i] != 0 && i < 63; i++) {
        entry->fields[idx].str_val[i] = val[i];
    }
    entry->fields[idx].str_val[i] = 0;
    entry->field_count++;
    return 0;
}

int log_structured_test(void) {
    log_structured_t entry;
    log_structured_init(&entry, 2, 1000);
    log_structured_add_int(&entry, "pid", 42);
    log_structured_add_str(&entry, "host", "server1");
    if (entry.field_count != 2) return -1;
    if (entry.fields[0].int_val != 42) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1603: Structured log entry should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1603: Output should not be empty");
    assert!(code.contains("fn log_structured_"), "C1603: Should contain log_structured_ functions");
}

/// C1604: Log rotation trigger with size and count limits
#[test]
fn c1604_log_rotation_trigger() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned long size_t;

#define LOG_ROT_MAX_FILES 8

typedef struct {
    uint32_t current_size;
    uint32_t max_size;
    int current_index;
    int max_files;
    int rotation_count;
    int bytes_written_total;
} log_rotation_t;

void log_rotation_init(log_rotation_t *rot, uint32_t max_sz, int max_files) {
    rot->current_size = 0;
    rot->max_size = max_sz;
    rot->current_index = 0;
    rot->max_files = max_files > LOG_ROT_MAX_FILES ? LOG_ROT_MAX_FILES : max_files;
    rot->rotation_count = 0;
    rot->bytes_written_total = 0;
}

int log_rotation_needs_rotate(log_rotation_t *rot) {
    return rot->current_size >= rot->max_size;
}

int log_rotation_rotate(log_rotation_t *rot) {
    if (!log_rotation_needs_rotate(rot)) return 0;
    rot->current_index = (rot->current_index + 1) % rot->max_files;
    rot->current_size = 0;
    rot->rotation_count++;
    return 1;
}

int log_rotation_write(log_rotation_t *rot, int bytes) {
    if (bytes <= 0) return -1;
    if (log_rotation_needs_rotate(rot)) {
        log_rotation_rotate(rot);
    }
    rot->current_size += bytes;
    rot->bytes_written_total += bytes;
    return rot->current_index;
}

int log_rotation_test(void) {
    log_rotation_t rot;
    log_rotation_init(&rot, 100, 4);
    log_rotation_write(&rot, 60);
    log_rotation_write(&rot, 60);
    if (rot.rotation_count != 1) return -1;
    if (rot.current_index != 1) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1604: Log rotation trigger should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1604: Output should not be empty");
    assert!(code.contains("fn log_rotation_"), "C1604: Should contain log_rotation_ functions");
}

/// C1605: Async log flush with pending queue
#[test]
fn c1605_async_log_flush() {
    let c_code = r##"
typedef unsigned int uint32_t;

#define LOG_FLUSH_BUF 512
#define LOG_FLUSH_PENDING 64

typedef struct {
    int msg_len;
    int priority;
    uint32_t enqueue_time;
} log_pending_t;

typedef struct {
    log_pending_t pending[LOG_FLUSH_PENDING];
    int pending_count;
    int flushed_count;
    int flush_in_progress;
    uint32_t last_flush_time;
    int total_bytes_flushed;
} log_flusher_t;

void log_flusher_init(log_flusher_t *fl) {
    fl->pending_count = 0;
    fl->flushed_count = 0;
    fl->flush_in_progress = 0;
    fl->last_flush_time = 0;
    fl->total_bytes_flushed = 0;
}

int log_flusher_enqueue(log_flusher_t *fl, int msg_len, int priority, uint32_t now) {
    if (fl->pending_count >= LOG_FLUSH_PENDING) return -1;
    int idx = fl->pending_count;
    fl->pending[idx].msg_len = msg_len;
    fl->pending[idx].priority = priority;
    fl->pending[idx].enqueue_time = now;
    fl->pending_count++;
    return idx;
}

int log_flusher_flush(log_flusher_t *fl, uint32_t now) {
    if (fl->flush_in_progress) return -1;
    if (fl->pending_count == 0) return 0;
    fl->flush_in_progress = 1;
    int flushed = 0;
    int i;
    for (i = 0; i < fl->pending_count; i++) {
        fl->total_bytes_flushed += fl->pending[i].msg_len;
        flushed++;
    }
    fl->flushed_count += flushed;
    fl->pending_count = 0;
    fl->flush_in_progress = 0;
    fl->last_flush_time = now;
    return flushed;
}

int log_flusher_test(void) {
    log_flusher_t fl;
    log_flusher_init(&fl);
    log_flusher_enqueue(&fl, 100, 1, 10);
    log_flusher_enqueue(&fl, 200, 2, 20);
    int n = log_flusher_flush(&fl, 30);
    if (n != 2) return -1;
    if (fl.total_bytes_flushed != 300) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1605: Async log flush should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1605: Output should not be empty");
    assert!(code.contains("fn log_flusher_"), "C1605: Should contain log_flusher_ functions");
}

// ============================================================================
// C1606-C1610: Trace Event Systems
// ============================================================================

/// C1606: Trace point registration with unique IDs
#[test]
fn c1606_trace_point_registration() {
    let c_code = r##"
typedef unsigned int uint32_t;

#define LOG_TP_MAX 128

typedef struct {
    uint32_t id;
    int enabled;
    int hit_count;
    int category;
} log_tracepoint_t;

typedef struct {
    log_tracepoint_t points[LOG_TP_MAX];
    int count;
    uint32_t next_id;
    int enabled_count;
} log_tp_registry_t;

void log_tp_init(log_tp_registry_t *reg) {
    reg->count = 0;
    reg->next_id = 1;
    reg->enabled_count = 0;
}

int log_tp_register(log_tp_registry_t *reg, int category) {
    if (reg->count >= LOG_TP_MAX) return -1;
    int idx = reg->count;
    reg->points[idx].id = reg->next_id++;
    reg->points[idx].enabled = 1;
    reg->points[idx].hit_count = 0;
    reg->points[idx].category = category;
    reg->count++;
    reg->enabled_count++;
    return idx;
}

void log_tp_hit(log_tp_registry_t *reg, int idx) {
    if (idx >= 0 && idx < reg->count && reg->points[idx].enabled) {
        reg->points[idx].hit_count++;
    }
}

int log_tp_disable(log_tp_registry_t *reg, int idx) {
    if (idx < 0 || idx >= reg->count) return -1;
    if (reg->points[idx].enabled) {
        reg->points[idx].enabled = 0;
        reg->enabled_count--;
    }
    return 0;
}

int log_tp_test(void) {
    log_tp_registry_t reg;
    log_tp_init(&reg);
    int tp1 = log_tp_register(&reg, 1);
    int tp2 = log_tp_register(&reg, 2);
    log_tp_hit(&reg, tp1);
    log_tp_hit(&reg, tp1);
    log_tp_hit(&reg, tp2);
    if (reg.points[tp1].hit_count != 2) return -1;
    log_tp_disable(&reg, tp2);
    if (reg.enabled_count != 1) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1606: Trace point registration should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1606: Output should not be empty");
    assert!(code.contains("fn log_tp_"), "C1606: Should contain log_tp_ functions");
}

/// C1607: Span tracking with enter/exit timestamps
#[test]
fn c1607_span_tracking() {
    let c_code = r##"
typedef unsigned int uint32_t;

#define LOG_SPAN_MAX 64
#define LOG_SPAN_STACK 16

typedef struct {
    uint32_t span_id;
    uint32_t parent_id;
    uint32_t start_time;
    uint32_t end_time;
    int active;
} log_span_t;

typedef struct {
    log_span_t spans[LOG_SPAN_MAX];
    int span_count;
    int stack[LOG_SPAN_STACK];
    int stack_depth;
    uint32_t next_id;
} log_span_tracker_t;

void log_span_init(log_span_tracker_t *t) {
    t->span_count = 0;
    t->stack_depth = 0;
    t->next_id = 1;
}

int log_span_enter(log_span_tracker_t *t, uint32_t now) {
    if (t->span_count >= LOG_SPAN_MAX) return -1;
    if (t->stack_depth >= LOG_SPAN_STACK) return -2;
    int idx = t->span_count;
    t->spans[idx].span_id = t->next_id++;
    t->spans[idx].parent_id = t->stack_depth > 0 ?
        t->spans[t->stack[t->stack_depth - 1]].span_id : 0;
    t->spans[idx].start_time = now;
    t->spans[idx].end_time = 0;
    t->spans[idx].active = 1;
    t->stack[t->stack_depth] = idx;
    t->stack_depth++;
    t->span_count++;
    return idx;
}

int log_span_exit(log_span_tracker_t *t, uint32_t now) {
    if (t->stack_depth <= 0) return -1;
    t->stack_depth--;
    int idx = t->stack[t->stack_depth];
    t->spans[idx].end_time = now;
    t->spans[idx].active = 0;
    return idx;
}

uint32_t log_span_duration(log_span_tracker_t *t, int idx) {
    if (idx < 0 || idx >= t->span_count) return 0;
    if (t->spans[idx].active) return 0;
    return t->spans[idx].end_time - t->spans[idx].start_time;
}

int log_span_test(void) {
    log_span_tracker_t tracker;
    log_span_init(&tracker);
    int s1 = log_span_enter(&tracker, 100);
    int s2 = log_span_enter(&tracker, 150);
    log_span_exit(&tracker, 200);
    log_span_exit(&tracker, 300);
    if (log_span_duration(&tracker, s2) != 50) return -1;
    if (log_span_duration(&tracker, s1) != 200) return -2;
    if (tracker.spans[s2].parent_id != tracker.spans[s1].span_id) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1607: Span tracking should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1607: Output should not be empty");
    assert!(code.contains("fn log_span_"), "C1607: Should contain log_span_ functions");
}

/// C1608: Event correlation with correlation IDs
#[test]
fn c1608_event_correlation() {
    let c_code = r##"
typedef unsigned int uint32_t;

#define LOG_CORR_MAX 128

typedef struct {
    uint32_t event_id;
    uint32_t correlation_id;
    uint32_t timestamp;
    int event_type;
} log_corr_event_t;

typedef struct {
    log_corr_event_t events[LOG_CORR_MAX];
    int count;
    uint32_t next_event_id;
} log_correlator_t;

void log_corr_init(log_correlator_t *c) {
    c->count = 0;
    c->next_event_id = 1;
}

int log_corr_add(log_correlator_t *c, uint32_t corr_id, int etype, uint32_t ts) {
    if (c->count >= LOG_CORR_MAX) return -1;
    int idx = c->count;
    c->events[idx].event_id = c->next_event_id++;
    c->events[idx].correlation_id = corr_id;
    c->events[idx].timestamp = ts;
    c->events[idx].event_type = etype;
    c->count++;
    return idx;
}

int log_corr_count_by_id(log_correlator_t *c, uint32_t corr_id) {
    int n = 0;
    int i;
    for (i = 0; i < c->count; i++) {
        if (c->events[i].correlation_id == corr_id) n++;
    }
    return n;
}

int log_corr_find_first(log_correlator_t *c, uint32_t corr_id) {
    int i;
    for (i = 0; i < c->count; i++) {
        if (c->events[i].correlation_id == corr_id) return i;
    }
    return -1;
}

int log_corr_test(void) {
    log_correlator_t corr;
    log_corr_init(&corr);
    log_corr_add(&corr, 100, 1, 10);
    log_corr_add(&corr, 100, 2, 20);
    log_corr_add(&corr, 200, 1, 30);
    log_corr_add(&corr, 100, 3, 40);
    if (log_corr_count_by_id(&corr, 100) != 3) return -1;
    if (log_corr_find_first(&corr, 200) != 2) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1608: Event correlation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1608: Output should not be empty");
    assert!(code.contains("fn log_corr_"), "C1608: Should contain log_corr_ functions");
}

/// C1609: Trace sampling with configurable rate
#[test]
fn c1609_trace_sampling() {
    let c_code = r##"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t sample_rate;
    uint32_t counter;
    int sampled_count;
    int dropped_count;
    uint32_t seed;
} log_sampler_t;

void log_sampler_init(log_sampler_t *s, uint32_t rate) {
    s->sample_rate = rate;
    s->counter = 0;
    s->sampled_count = 0;
    s->dropped_count = 0;
    s->seed = 12345;
}

static uint32_t log_sampler_rng(log_sampler_t *s) {
    s->seed = s->seed * 1103515245 + 12345;
    return (s->seed >> 16) & 0x7FFF;
}

int log_sampler_should_sample(log_sampler_t *s) {
    s->counter++;
    if (s->sample_rate == 0) {
        s->dropped_count++;
        return 0;
    }
    if (s->sample_rate >= 100) {
        s->sampled_count++;
        return 1;
    }
    uint32_t r = log_sampler_rng(s) % 100;
    if (r < s->sample_rate) {
        s->sampled_count++;
        return 1;
    }
    s->dropped_count++;
    return 0;
}

int log_sampler_total(log_sampler_t *s) {
    return s->sampled_count + s->dropped_count;
}

int log_sampler_test(void) {
    log_sampler_t sampler;
    log_sampler_init(&sampler, 100);
    int i;
    for (i = 0; i < 10; i++) {
        log_sampler_should_sample(&sampler);
    }
    if (sampler.sampled_count != 10) return -1;
    log_sampler_init(&sampler, 0);
    for (i = 0; i < 10; i++) {
        log_sampler_should_sample(&sampler);
    }
    if (sampler.dropped_count != 10) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1609: Trace sampling should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1609: Output should not be empty");
    assert!(code.contains("fn log_sampler_"), "C1609: Should contain log_sampler_ functions");
}

/// C1610: Context propagation across trace boundaries
#[test]
fn c1610_context_propagation() {
    let c_code = r##"
typedef unsigned int uint32_t;

#define LOG_CTX_MAX 32
#define LOG_CTX_KEY_LEN 32

typedef struct {
    char key[LOG_CTX_KEY_LEN];
    uint32_t value;
    int key_len;
} log_ctx_entry_t;

typedef struct {
    log_ctx_entry_t entries[LOG_CTX_MAX];
    int count;
    uint32_t trace_id;
    uint32_t span_id;
} log_context_t;

void log_ctx_init(log_context_t *ctx, uint32_t trace_id, uint32_t span_id) {
    ctx->count = 0;
    ctx->trace_id = trace_id;
    ctx->span_id = span_id;
}

int log_ctx_set(log_context_t *ctx, const char *key, uint32_t value) {
    int i;
    for (i = 0; i < ctx->count; i++) {
        int match = 1;
        int j;
        for (j = 0; j < ctx->entries[i].key_len; j++) {
            if (ctx->entries[i].key[j] != key[j]) { match = 0; break; }
        }
        if (match && key[ctx->entries[i].key_len] == 0) {
            ctx->entries[i].value = value;
            return i;
        }
    }
    if (ctx->count >= LOG_CTX_MAX) return -1;
    int idx = ctx->count;
    for (i = 0; key[i] != 0 && i < LOG_CTX_KEY_LEN - 1; i++) {
        ctx->entries[idx].key[i] = key[i];
    }
    ctx->entries[idx].key[i] = 0;
    ctx->entries[idx].key_len = i;
    ctx->entries[idx].value = value;
    ctx->count++;
    return idx;
}

int log_ctx_get(log_context_t *ctx, const char *key, uint32_t *out) {
    int i;
    for (i = 0; i < ctx->count; i++) {
        int match = 1;
        int j;
        for (j = 0; j < ctx->entries[i].key_len; j++) {
            if (ctx->entries[i].key[j] != key[j]) { match = 0; break; }
        }
        if (match && key[ctx->entries[i].key_len] == 0) {
            *out = ctx->entries[i].value;
            return 0;
        }
    }
    return -1;
}

int log_ctx_propagate(log_context_t *src, log_context_t *dst) {
    int i;
    int propagated = 0;
    for (i = 0; i < src->count; i++) {
        log_ctx_set(dst, src->entries[i].key, src->entries[i].value);
        propagated++;
    }
    return propagated;
}

int log_ctx_test(void) {
    log_context_t ctx1;
    log_context_t ctx2;
    log_ctx_init(&ctx1, 1000, 1);
    log_ctx_init(&ctx2, 1000, 2);
    log_ctx_set(&ctx1, "user", 42);
    log_ctx_set(&ctx1, "req", 99);
    int n = log_ctx_propagate(&ctx1, &ctx2);
    if (n != 2) return -1;
    uint32_t val = 0;
    log_ctx_get(&ctx2, "user", &val);
    if (val != 42) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1610: Context propagation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1610: Output should not be empty");
    assert!(code.contains("fn log_ctx_"), "C1610: Should contain log_ctx_ functions");
}

// ============================================================================
// C1611-C1615: Log Aggregation
// ============================================================================

/// C1611: Log merging from multiple sources
#[test]
fn c1611_log_merging() {
    let c_code = r##"
typedef unsigned int uint32_t;

#define LOG_MERGE_SOURCES 8
#define LOG_MERGE_BUF 64

typedef struct {
    uint32_t timestamp;
    int source_id;
    int severity;
} log_merge_entry_t;

typedef struct {
    log_merge_entry_t buffer[LOG_MERGE_BUF];
    int count;
    int source_counts[LOG_MERGE_SOURCES];
    int num_sources;
} log_merger_t;

void log_merger_init(log_merger_t *m, int num_src) {
    m->count = 0;
    m->num_sources = num_src > LOG_MERGE_SOURCES ? LOG_MERGE_SOURCES : num_src;
    int i;
    for (i = 0; i < LOG_MERGE_SOURCES; i++) {
        m->source_counts[i] = 0;
    }
}

int log_merger_add(log_merger_t *m, uint32_t ts, int src, int sev) {
    if (m->count >= LOG_MERGE_BUF) return -1;
    if (src < 0 || src >= m->num_sources) return -2;
    int idx = m->count;
    m->buffer[idx].timestamp = ts;
    m->buffer[idx].source_id = src;
    m->buffer[idx].severity = sev;
    m->source_counts[src]++;
    m->count++;
    return idx;
}

void log_merger_sort_by_time(log_merger_t *m) {
    int i, j;
    for (i = 0; i < m->count - 1; i++) {
        for (j = 0; j < m->count - 1 - i; j++) {
            if (m->buffer[j].timestamp > m->buffer[j + 1].timestamp) {
                log_merge_entry_t tmp = m->buffer[j];
                m->buffer[j] = m->buffer[j + 1];
                m->buffer[j + 1] = tmp;
            }
        }
    }
}

int log_merger_test(void) {
    log_merger_t merger;
    log_merger_init(&merger, 3);
    log_merger_add(&merger, 300, 0, 2);
    log_merger_add(&merger, 100, 1, 1);
    log_merger_add(&merger, 200, 2, 3);
    log_merger_sort_by_time(&merger);
    if (merger.buffer[0].timestamp != 100) return -1;
    if (merger.buffer[2].timestamp != 300) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1611: Log merging should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1611: Output should not be empty");
    assert!(code.contains("fn log_merger_"), "C1611: Should contain log_merger_ functions");
}

/// C1612: Timestamp normalization across clock sources
#[test]
fn c1612_timestamp_normalization() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef long long int64_t;

#define LOG_CLOCK_MAX 8

typedef struct {
    int64_t offset;
    int64_t drift_ppb;
    uint32_t last_sync;
    int active;
} log_clock_source_t;

typedef struct {
    log_clock_source_t clocks[LOG_CLOCK_MAX];
    int clock_count;
    int reference_clock;
} log_time_normalizer_t;

void log_time_init(log_time_normalizer_t *n) {
    n->clock_count = 0;
    n->reference_clock = 0;
}

int log_time_add_clock(log_time_normalizer_t *n, int64_t offset, int64_t drift) {
    if (n->clock_count >= LOG_CLOCK_MAX) return -1;
    int idx = n->clock_count;
    n->clocks[idx].offset = offset;
    n->clocks[idx].drift_ppb = drift;
    n->clocks[idx].last_sync = 0;
    n->clocks[idx].active = 1;
    n->clock_count++;
    return idx;
}

int64_t log_time_normalize(log_time_normalizer_t *n, int clock_id, uint32_t raw_ts) {
    if (clock_id < 0 || clock_id >= n->clock_count) return -1;
    if (!n->clocks[clock_id].active) return -1;
    int64_t adjusted = (int64_t)raw_ts + n->clocks[clock_id].offset;
    int64_t drift_correction = ((int64_t)raw_ts * n->clocks[clock_id].drift_ppb) / 1000000000;
    return adjusted + drift_correction;
}

int log_time_sync(log_time_normalizer_t *n, int clock_id, int64_t new_offset, uint32_t now) {
    if (clock_id < 0 || clock_id >= n->clock_count) return -1;
    n->clocks[clock_id].offset = new_offset;
    n->clocks[clock_id].last_sync = now;
    return 0;
}

int log_time_test(void) {
    log_time_normalizer_t norm;
    log_time_init(&norm);
    log_time_add_clock(&norm, 100, 0);
    log_time_add_clock(&norm, -50, 0);
    int64_t t1 = log_time_normalize(&norm, 0, 1000);
    int64_t t2 = log_time_normalize(&norm, 1, 1000);
    if (t1 != 1100) return -1;
    if (t2 != 950) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1612: Timestamp normalization should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1612: Output should not be empty");
    assert!(code.contains("fn log_time_"), "C1612: Should contain log_time_ functions");
}

/// C1613: Log deduplication with content hashing
#[test]
fn c1613_log_deduplication() {
    let c_code = r##"
typedef unsigned int uint32_t;

#define LOG_DEDUP_SLOTS 128
#define LOG_DEDUP_MASK  127

typedef struct {
    uint32_t hash;
    int count;
    uint32_t first_seen;
    uint32_t last_seen;
    int occupied;
} log_dedup_slot_t;

typedef struct {
    log_dedup_slot_t slots[LOG_DEDUP_SLOTS];
    int unique_count;
    int duplicate_count;
    int total_count;
} log_dedup_t;

void log_dedup_init(log_dedup_t *d) {
    d->unique_count = 0;
    d->duplicate_count = 0;
    d->total_count = 0;
    int i;
    for (i = 0; i < LOG_DEDUP_SLOTS; i++) {
        d->slots[i].occupied = 0;
        d->slots[i].count = 0;
        d->slots[i].hash = 0;
    }
}

static uint32_t log_dedup_hash(const char *msg, int len) {
    uint32_t h = 5381;
    int i;
    for (i = 0; i < len; i++) {
        h = ((h << 5) + h) + (uint32_t)msg[i];
    }
    return h;
}

int log_dedup_check(log_dedup_t *d, const char *msg, int len, uint32_t ts) {
    uint32_t h = log_dedup_hash(msg, len);
    int slot = (int)(h & LOG_DEDUP_MASK);
    d->total_count++;
    if (d->slots[slot].occupied && d->slots[slot].hash == h) {
        d->slots[slot].count++;
        d->slots[slot].last_seen = ts;
        d->duplicate_count++;
        return 0;
    }
    d->slots[slot].hash = h;
    d->slots[slot].count = 1;
    d->slots[slot].first_seen = ts;
    d->slots[slot].last_seen = ts;
    d->slots[slot].occupied = 1;
    d->unique_count++;
    return 1;
}

int log_dedup_test(void) {
    log_dedup_t dedup;
    log_dedup_init(&dedup);
    int r1 = log_dedup_check(&dedup, "error A", 7, 100);
    int r2 = log_dedup_check(&dedup, "error A", 7, 200);
    int r3 = log_dedup_check(&dedup, "error B", 7, 300);
    if (r1 != 1) return -1;
    if (r2 != 0) return -2;
    if (r3 != 1) return -3;
    if (dedup.duplicate_count != 1) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1613: Log deduplication should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1613: Output should not be empty");
    assert!(code.contains("fn log_dedup_"), "C1613: Should contain log_dedup_ functions");
}

/// C1614: Pattern matching log filter
#[test]
fn c1614_pattern_matching_filter() {
    let c_code = r##"
typedef unsigned int uint32_t;

#define LOG_PAT_MAX 16
#define LOG_PAT_LEN 32

typedef struct {
    char pattern[LOG_PAT_LEN];
    int pat_len;
    int include;
    int match_count;
} log_pattern_rule_t;

typedef struct {
    log_pattern_rule_t rules[LOG_PAT_MAX];
    int rule_count;
    int accepted;
    int rejected;
} log_pat_filter_t;

void log_pat_init(log_pat_filter_t *f) {
    f->rule_count = 0;
    f->accepted = 0;
    f->rejected = 0;
}

int log_pat_add_rule(log_pat_filter_t *f, const char *pat, int len, int include) {
    if (f->rule_count >= LOG_PAT_MAX) return -1;
    int idx = f->rule_count;
    int i;
    for (i = 0; i < len && i < LOG_PAT_LEN - 1; i++) {
        f->rules[idx].pattern[i] = pat[i];
    }
    f->rules[idx].pattern[i] = 0;
    f->rules[idx].pat_len = i;
    f->rules[idx].include = include;
    f->rules[idx].match_count = 0;
    f->rule_count++;
    return idx;
}

static int log_pat_substr(const char *haystack, int hlen, const char *needle, int nlen) {
    int i, j;
    if (nlen > hlen) return 0;
    for (i = 0; i <= hlen - nlen; i++) {
        int match = 1;
        for (j = 0; j < nlen; j++) {
            if (haystack[i + j] != needle[j]) { match = 0; break; }
        }
        if (match) return 1;
    }
    return 0;
}

int log_pat_evaluate(log_pat_filter_t *f, const char *msg, int msg_len) {
    int i;
    for (i = 0; i < f->rule_count; i++) {
        if (log_pat_substr(msg, msg_len, f->rules[i].pattern, f->rules[i].pat_len)) {
            f->rules[i].match_count++;
            if (!f->rules[i].include) {
                f->rejected++;
                return 0;
            }
        }
    }
    f->accepted++;
    return 1;
}

int log_pat_test(void) {
    log_pat_filter_t filt;
    log_pat_init(&filt);
    log_pat_add_rule(&filt, "ERR", 3, 1);
    log_pat_add_rule(&filt, "DEBUG", 5, 0);
    int r1 = log_pat_evaluate(&filt, "ERR: disk fail", 14);
    int r2 = log_pat_evaluate(&filt, "DEBUG: verbose", 14);
    if (r1 != 1) return -1;
    if (r2 != 0) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1614: Pattern matching filter should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1614: Output should not be empty");
    assert!(code.contains("fn log_pat_"), "C1614: Should contain log_pat_ functions");
}

/// C1615: Rate limiting for log output
#[test]
fn c1615_rate_limiting() {
    let c_code = r##"
typedef unsigned int uint32_t;

#define LOG_RL_BUCKETS 16

typedef struct {
    uint32_t tokens;
    uint32_t max_tokens;
    uint32_t refill_rate;
    uint32_t last_refill;
} log_rl_bucket_t;

typedef struct {
    log_rl_bucket_t buckets[LOG_RL_BUCKETS];
    int bucket_count;
    int allowed;
    int throttled;
} log_rate_limiter_t;

void log_rl_init(log_rate_limiter_t *rl, int num_buckets, uint32_t max_tok, uint32_t rate) {
    rl->bucket_count = num_buckets > LOG_RL_BUCKETS ? LOG_RL_BUCKETS : num_buckets;
    rl->allowed = 0;
    rl->throttled = 0;
    int i;
    for (i = 0; i < rl->bucket_count; i++) {
        rl->buckets[i].tokens = max_tok;
        rl->buckets[i].max_tokens = max_tok;
        rl->buckets[i].refill_rate = rate;
        rl->buckets[i].last_refill = 0;
    }
}

void log_rl_refill(log_rate_limiter_t *rl, int bucket, uint32_t now) {
    if (bucket < 0 || bucket >= rl->bucket_count) return;
    uint32_t elapsed = now - rl->buckets[bucket].last_refill;
    uint32_t add = elapsed * rl->buckets[bucket].refill_rate;
    rl->buckets[bucket].tokens += add;
    if (rl->buckets[bucket].tokens > rl->buckets[bucket].max_tokens) {
        rl->buckets[bucket].tokens = rl->buckets[bucket].max_tokens;
    }
    rl->buckets[bucket].last_refill = now;
}

int log_rl_allow(log_rate_limiter_t *rl, int bucket, uint32_t now) {
    if (bucket < 0 || bucket >= rl->bucket_count) return 0;
    log_rl_refill(rl, bucket, now);
    if (rl->buckets[bucket].tokens > 0) {
        rl->buckets[bucket].tokens--;
        rl->allowed++;
        return 1;
    }
    rl->throttled++;
    return 0;
}

int log_rl_test(void) {
    log_rate_limiter_t limiter;
    log_rl_init(&limiter, 2, 3, 1);
    int r1 = log_rl_allow(&limiter, 0, 0);
    int r2 = log_rl_allow(&limiter, 0, 0);
    int r3 = log_rl_allow(&limiter, 0, 0);
    int r4 = log_rl_allow(&limiter, 0, 0);
    if (r1 != 1) return -1;
    if (r2 != 1) return -2;
    if (r3 != 1) return -3;
    if (r4 != 0) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1615: Rate limiting should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1615: Output should not be empty");
    assert!(code.contains("fn log_rl_"), "C1615: Should contain log_rl_ functions");
}

// ============================================================================
// C1616-C1620: Performance Counters
// ============================================================================

/// C1616: Hardware counter abstraction layer
#[test]
fn c1616_hardware_counter_abstraction() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

#define LOG_HW_CTR_MAX 32

typedef struct {
    uint64_t value;
    uint64_t prev_value;
    uint32_t id;
    int enabled;
    int event_type;
} log_hw_counter_t;

typedef struct {
    log_hw_counter_t counters[LOG_HW_CTR_MAX];
    int count;
    int active_count;
} log_hw_ctr_mgr_t;

void log_hw_ctr_init(log_hw_ctr_mgr_t *mgr) {
    mgr->count = 0;
    mgr->active_count = 0;
}

int log_hw_ctr_add(log_hw_ctr_mgr_t *mgr, uint32_t id, int etype) {
    if (mgr->count >= LOG_HW_CTR_MAX) return -1;
    int idx = mgr->count;
    mgr->counters[idx].value = 0;
    mgr->counters[idx].prev_value = 0;
    mgr->counters[idx].id = id;
    mgr->counters[idx].enabled = 1;
    mgr->counters[idx].event_type = etype;
    mgr->count++;
    mgr->active_count++;
    return idx;
}

void log_hw_ctr_update(log_hw_ctr_mgr_t *mgr, int idx, uint64_t new_val) {
    if (idx < 0 || idx >= mgr->count) return;
    mgr->counters[idx].prev_value = mgr->counters[idx].value;
    mgr->counters[idx].value = new_val;
}

uint64_t log_hw_ctr_delta(log_hw_ctr_mgr_t *mgr, int idx) {
    if (idx < 0 || idx >= mgr->count) return 0;
    return mgr->counters[idx].value - mgr->counters[idx].prev_value;
}

int log_hw_ctr_test(void) {
    log_hw_ctr_mgr_t mgr;
    log_hw_ctr_init(&mgr);
    int c1 = log_hw_ctr_add(&mgr, 1, 0);
    log_hw_ctr_update(&mgr, c1, 100);
    log_hw_ctr_update(&mgr, c1, 250);
    if (log_hw_ctr_delta(&mgr, c1) != 150) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1616: Hardware counter abstraction should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1616: Output should not be empty");
    assert!(code.contains("fn log_hw_ctr_"), "C1616: Should contain log_hw_ctr_ functions");
}

/// C1617: Counter overflow handling with wraparound detection
#[test]
fn c1617_counter_overflow_handling() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

#define LOG_OVF_MAX 16

typedef struct {
    uint32_t value;
    uint32_t prev_value;
    int overflow_count;
    uint64_t total_accumulated;
} log_ovf_counter_t;

typedef struct {
    log_ovf_counter_t counters[LOG_OVF_MAX];
    int count;
} log_ovf_tracker_t;

void log_ovf_init(log_ovf_tracker_t *t) {
    t->count = 0;
}

int log_ovf_add(log_ovf_tracker_t *t) {
    if (t->count >= LOG_OVF_MAX) return -1;
    int idx = t->count;
    t->counters[idx].value = 0;
    t->counters[idx].prev_value = 0;
    t->counters[idx].overflow_count = 0;
    t->counters[idx].total_accumulated = 0;
    t->count++;
    return idx;
}

void log_ovf_update(log_ovf_tracker_t *t, int idx, uint32_t new_val) {
    if (idx < 0 || idx >= t->count) return;
    t->counters[idx].prev_value = t->counters[idx].value;
    if (new_val < t->counters[idx].prev_value) {
        t->counters[idx].overflow_count++;
        uint64_t wrapped = (uint64_t)0xFFFFFFFF - (uint64_t)t->counters[idx].prev_value + (uint64_t)new_val + 1;
        t->counters[idx].total_accumulated += wrapped;
    } else {
        t->counters[idx].total_accumulated += (uint64_t)(new_val - t->counters[idx].prev_value);
    }
    t->counters[idx].value = new_val;
}

int log_ovf_test(void) {
    log_ovf_tracker_t tracker;
    log_ovf_init(&tracker);
    int c1 = log_ovf_add(&tracker);
    log_ovf_update(&tracker, c1, 100);
    log_ovf_update(&tracker, c1, 200);
    if (tracker.counters[c1].total_accumulated != 200) return -1;
    if (tracker.counters[c1].overflow_count != 0) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1617: Counter overflow handling should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1617: Output should not be empty");
    assert!(code.contains("fn log_ovf_"), "C1617: Should contain log_ovf_ functions");
}

/// C1618: Counter hierarchy with parent-child aggregation
#[test]
fn c1618_counter_hierarchy() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

#define LOG_HIER_MAX 32

typedef struct {
    uint64_t value;
    int parent;
    int children[4];
    int child_count;
    int id;
} log_hier_counter_t;

typedef struct {
    log_hier_counter_t nodes[LOG_HIER_MAX];
    int count;
} log_hier_tree_t;

void log_hier_init(log_hier_tree_t *t) {
    t->count = 0;
}

int log_hier_add(log_hier_tree_t *t, int parent_idx) {
    if (t->count >= LOG_HIER_MAX) return -1;
    int idx = t->count;
    t->nodes[idx].value = 0;
    t->nodes[idx].parent = parent_idx;
    t->nodes[idx].child_count = 0;
    t->nodes[idx].id = idx;
    if (parent_idx >= 0 && parent_idx < t->count) {
        if (t->nodes[parent_idx].child_count < 4) {
            t->nodes[parent_idx].children[t->nodes[parent_idx].child_count] = idx;
            t->nodes[parent_idx].child_count++;
        }
    }
    t->count++;
    return idx;
}

void log_hier_increment(log_hier_tree_t *t, int idx, uint64_t amount) {
    if (idx < 0 || idx >= t->count) return;
    t->nodes[idx].value += amount;
}

uint64_t log_hier_aggregate(log_hier_tree_t *t, int idx) {
    if (idx < 0 || idx >= t->count) return 0;
    uint64_t total = t->nodes[idx].value;
    int i;
    for (i = 0; i < t->nodes[idx].child_count; i++) {
        total += log_hier_aggregate(t, t->nodes[idx].children[i]);
    }
    return total;
}

int log_hier_test(void) {
    log_hier_tree_t tree;
    log_hier_init(&tree);
    int root = log_hier_add(&tree, -1);
    int c1 = log_hier_add(&tree, root);
    int c2 = log_hier_add(&tree, root);
    log_hier_increment(&tree, c1, 10);
    log_hier_increment(&tree, c2, 20);
    log_hier_increment(&tree, root, 5);
    uint64_t total = log_hier_aggregate(&tree, root);
    if (total != 35) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1618: Counter hierarchy should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1618: Output should not be empty");
    assert!(code.contains("fn log_hier_"), "C1618: Should contain log_hier_ functions");
}

/// C1619: Snapshot aggregation for periodic metric collection
#[test]
fn c1619_snapshot_aggregation() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

#define LOG_SNAP_MAX 32
#define LOG_SNAP_HISTORY 8

typedef struct {
    uint64_t values[LOG_SNAP_MAX];
    uint32_t timestamp;
    int value_count;
} log_snapshot_t;

typedef struct {
    log_snapshot_t history[LOG_SNAP_HISTORY];
    int history_count;
    int current_slot;
    int metric_count;
} log_snap_collector_t;

void log_snap_init(log_snap_collector_t *c, int metrics) {
    c->history_count = 0;
    c->current_slot = 0;
    c->metric_count = metrics > LOG_SNAP_MAX ? LOG_SNAP_MAX : metrics;
}

int log_snap_capture(log_snap_collector_t *c, uint64_t *values, int count, uint32_t ts) {
    if (count > c->metric_count) return -1;
    int slot = c->current_slot;
    c->history[slot].timestamp = ts;
    c->history[slot].value_count = count;
    int i;
    for (i = 0; i < count; i++) {
        c->history[slot].values[i] = values[i];
    }
    c->current_slot = (c->current_slot + 1) % LOG_SNAP_HISTORY;
    if (c->history_count < LOG_SNAP_HISTORY) c->history_count++;
    return slot;
}

uint64_t log_snap_average(log_snap_collector_t *c, int metric_idx) {
    if (metric_idx < 0 || metric_idx >= c->metric_count) return 0;
    if (c->history_count == 0) return 0;
    uint64_t sum = 0;
    int i;
    for (i = 0; i < c->history_count; i++) {
        if (metric_idx < c->history[i].value_count) {
            sum += c->history[i].values[metric_idx];
        }
    }
    return sum / (uint64_t)c->history_count;
}

int log_snap_test(void) {
    log_snap_collector_t coll;
    log_snap_init(&coll, 4);
    uint64_t v1[4] = {10, 20, 30, 40};
    uint64_t v2[4] = {20, 40, 60, 80};
    log_snap_capture(&coll, v1, 4, 100);
    log_snap_capture(&coll, v2, 4, 200);
    uint64_t avg = log_snap_average(&coll, 0);
    if (avg != 15) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1619: Snapshot aggregation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1619: Output should not be empty");
    assert!(code.contains("fn log_snap_"), "C1619: Should contain log_snap_ functions");
}

/// C1620: Percentile computation for latency metrics
#[test]
fn c1620_percentile_computation() {
    let c_code = r##"
typedef unsigned int uint32_t;

#define LOG_PCT_MAX 256

typedef struct {
    uint32_t samples[LOG_PCT_MAX];
    int count;
    int sorted;
} log_percentile_t;

void log_pct_init(log_percentile_t *p) {
    p->count = 0;
    p->sorted = 0;
}

int log_pct_add(log_percentile_t *p, uint32_t value) {
    if (p->count >= LOG_PCT_MAX) return -1;
    p->samples[p->count] = value;
    p->count++;
    p->sorted = 0;
    return 0;
}

static void log_pct_sort(log_percentile_t *p) {
    if (p->sorted) return;
    int i, j;
    for (i = 0; i < p->count - 1; i++) {
        for (j = 0; j < p->count - 1 - i; j++) {
            if (p->samples[j] > p->samples[j + 1]) {
                uint32_t tmp = p->samples[j];
                p->samples[j] = p->samples[j + 1];
                p->samples[j + 1] = tmp;
            }
        }
    }
    p->sorted = 1;
}

uint32_t log_pct_get(log_percentile_t *p, int percentile) {
    if (p->count == 0) return 0;
    if (percentile < 0) percentile = 0;
    if (percentile > 100) percentile = 100;
    log_pct_sort(p);
    int idx = (percentile * (p->count - 1)) / 100;
    return p->samples[idx];
}

uint32_t log_pct_median(log_percentile_t *p) {
    return log_pct_get(p, 50);
}

int log_pct_test(void) {
    log_percentile_t pct;
    log_pct_init(&pct);
    int i;
    for (i = 1; i <= 100; i++) {
        log_pct_add(&pct, (uint32_t)i);
    }
    uint32_t p50 = log_pct_get(&pct, 50);
    uint32_t p99 = log_pct_get(&pct, 99);
    if (p50 != 50) return -1;
    if (p99 != 99) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1620: Percentile computation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1620: Output should not be empty");
    assert!(code.contains("fn log_pct_"), "C1620: Should contain log_pct_ functions");
}

// ============================================================================
// C1621-C1625: Diagnostic Systems
// ============================================================================

/// C1621: Health check framework with status tracking
#[test]
fn c1621_health_check_framework() {
    let c_code = r##"
typedef unsigned int uint32_t;

#define LOG_HC_MAX 16
#define LOG_HC_OK      0
#define LOG_HC_WARN    1
#define LOG_HC_CRIT    2
#define LOG_HC_UNKNOWN 3

typedef struct {
    int status;
    uint32_t last_check;
    int consecutive_failures;
    int total_checks;
    int total_failures;
} log_health_check_t;

typedef struct {
    log_health_check_t checks[LOG_HC_MAX];
    int check_count;
    int overall_status;
} log_health_mgr_t;

void log_health_init(log_health_mgr_t *m) {
    m->check_count = 0;
    m->overall_status = LOG_HC_UNKNOWN;
}

int log_health_add(log_health_mgr_t *m) {
    if (m->check_count >= LOG_HC_MAX) return -1;
    int idx = m->check_count;
    m->checks[idx].status = LOG_HC_UNKNOWN;
    m->checks[idx].last_check = 0;
    m->checks[idx].consecutive_failures = 0;
    m->checks[idx].total_checks = 0;
    m->checks[idx].total_failures = 0;
    m->check_count++;
    return idx;
}

void log_health_report(log_health_mgr_t *m, int idx, int status, uint32_t now) {
    if (idx < 0 || idx >= m->check_count) return;
    m->checks[idx].status = status;
    m->checks[idx].last_check = now;
    m->checks[idx].total_checks++;
    if (status != LOG_HC_OK) {
        m->checks[idx].consecutive_failures++;
        m->checks[idx].total_failures++;
    } else {
        m->checks[idx].consecutive_failures = 0;
    }
}

int log_health_overall(log_health_mgr_t *m) {
    int worst = LOG_HC_OK;
    int i;
    for (i = 0; i < m->check_count; i++) {
        if (m->checks[i].status > worst) {
            worst = m->checks[i].status;
        }
    }
    m->overall_status = worst;
    return worst;
}

int log_health_test(void) {
    log_health_mgr_t mgr;
    log_health_init(&mgr);
    int h1 = log_health_add(&mgr);
    int h2 = log_health_add(&mgr);
    log_health_report(&mgr, h1, LOG_HC_OK, 100);
    log_health_report(&mgr, h2, LOG_HC_WARN, 100);
    if (log_health_overall(&mgr) != LOG_HC_WARN) return -1;
    log_health_report(&mgr, h2, LOG_HC_OK, 200);
    if (log_health_overall(&mgr) != LOG_HC_OK) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1621: Health check framework should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1621: Output should not be empty");
    assert!(code.contains("fn log_health_"), "C1621: Should contain log_health_ functions");
}

/// C1622: Metric collection with named gauges and counters
#[test]
fn c1622_metric_collection() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

#define LOG_METRIC_MAX 32
#define LOG_METRIC_COUNTER 0
#define LOG_METRIC_GAUGE   1

typedef struct {
    int metric_type;
    uint64_t value;
    uint64_t min_val;
    uint64_t max_val;
    int update_count;
} log_metric_t;

typedef struct {
    log_metric_t metrics[LOG_METRIC_MAX];
    int count;
} log_metric_store_t;

void log_metric_init(log_metric_store_t *s) {
    s->count = 0;
}

int log_metric_add_counter(log_metric_store_t *s) {
    if (s->count >= LOG_METRIC_MAX) return -1;
    int idx = s->count;
    s->metrics[idx].metric_type = LOG_METRIC_COUNTER;
    s->metrics[idx].value = 0;
    s->metrics[idx].min_val = 0;
    s->metrics[idx].max_val = 0;
    s->metrics[idx].update_count = 0;
    s->count++;
    return idx;
}

int log_metric_add_gauge(log_metric_store_t *s) {
    if (s->count >= LOG_METRIC_MAX) return -1;
    int idx = s->count;
    s->metrics[idx].metric_type = LOG_METRIC_GAUGE;
    s->metrics[idx].value = 0;
    s->metrics[idx].min_val = 0xFFFFFFFFFFFFFFFF;
    s->metrics[idx].max_val = 0;
    s->metrics[idx].update_count = 0;
    s->count++;
    return idx;
}

void log_metric_inc(log_metric_store_t *s, int idx, uint64_t amount) {
    if (idx < 0 || idx >= s->count) return;
    s->metrics[idx].value += amount;
    s->metrics[idx].update_count++;
    if (s->metrics[idx].value > s->metrics[idx].max_val) {
        s->metrics[idx].max_val = s->metrics[idx].value;
    }
}

void log_metric_set(log_metric_store_t *s, int idx, uint64_t value) {
    if (idx < 0 || idx >= s->count) return;
    s->metrics[idx].value = value;
    s->metrics[idx].update_count++;
    if (value < s->metrics[idx].min_val) s->metrics[idx].min_val = value;
    if (value > s->metrics[idx].max_val) s->metrics[idx].max_val = value;
}

int log_metric_test(void) {
    log_metric_store_t store;
    log_metric_init(&store);
    int ctr = log_metric_add_counter(&store);
    int gau = log_metric_add_gauge(&store);
    log_metric_inc(&store, ctr, 5);
    log_metric_inc(&store, ctr, 3);
    if (store.metrics[ctr].value != 8) return -1;
    log_metric_set(&store, gau, 42);
    log_metric_set(&store, gau, 10);
    if (store.metrics[gau].min_val != 10) return -2;
    if (store.metrics[gau].max_val != 42) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1622: Metric collection should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1622: Output should not be empty");
    assert!(code.contains("fn log_metric_"), "C1622: Should contain log_metric_ functions");
}

/// C1623: Alert threshold with hysteresis
#[test]
fn c1623_alert_threshold() {
    let c_code = r##"
typedef unsigned int uint32_t;

#define LOG_ALERT_MAX 16
#define LOG_ALERT_CLEAR  0
#define LOG_ALERT_ACTIVE 1

typedef struct {
    uint32_t high_threshold;
    uint32_t low_threshold;
    int state;
    int trigger_count;
    int clear_count;
    uint32_t last_value;
} log_alert_t;

typedef struct {
    log_alert_t alerts[LOG_ALERT_MAX];
    int count;
    int active_alerts;
} log_alert_mgr_t;

void log_alert_init(log_alert_mgr_t *m) {
    m->count = 0;
    m->active_alerts = 0;
}

int log_alert_add(log_alert_mgr_t *m, uint32_t high, uint32_t low) {
    if (m->count >= LOG_ALERT_MAX) return -1;
    int idx = m->count;
    m->alerts[idx].high_threshold = high;
    m->alerts[idx].low_threshold = low;
    m->alerts[idx].state = LOG_ALERT_CLEAR;
    m->alerts[idx].trigger_count = 0;
    m->alerts[idx].clear_count = 0;
    m->alerts[idx].last_value = 0;
    m->count++;
    return idx;
}

int log_alert_evaluate(log_alert_mgr_t *m, int idx, uint32_t value) {
    if (idx < 0 || idx >= m->count) return -1;
    m->alerts[idx].last_value = value;
    if (m->alerts[idx].state == LOG_ALERT_CLEAR) {
        if (value >= m->alerts[idx].high_threshold) {
            m->alerts[idx].state = LOG_ALERT_ACTIVE;
            m->alerts[idx].trigger_count++;
            m->active_alerts++;
            return 1;
        }
    } else {
        if (value <= m->alerts[idx].low_threshold) {
            m->alerts[idx].state = LOG_ALERT_CLEAR;
            m->alerts[idx].clear_count++;
            m->active_alerts--;
            return 2;
        }
    }
    return 0;
}

int log_alert_test(void) {
    log_alert_mgr_t mgr;
    log_alert_init(&mgr);
    int a = log_alert_add(&mgr, 80, 20);
    log_alert_evaluate(&mgr, a, 50);
    if (mgr.alerts[a].state != LOG_ALERT_CLEAR) return -1;
    log_alert_evaluate(&mgr, a, 85);
    if (mgr.alerts[a].state != LOG_ALERT_ACTIVE) return -2;
    log_alert_evaluate(&mgr, a, 50);
    if (mgr.alerts[a].state != LOG_ALERT_ACTIVE) return -3;
    log_alert_evaluate(&mgr, a, 15);
    if (mgr.alerts[a].state != LOG_ALERT_CLEAR) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1623: Alert threshold should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1623: Output should not be empty");
    assert!(code.contains("fn log_alert_"), "C1623: Should contain log_alert_ functions");
}

/// C1624: Diagnostic dump with key-value pairs
#[test]
fn c1624_diagnostic_dump() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

#define LOG_DIAG_MAX 64
#define LOG_DIAG_KEY_LEN 24

typedef struct {
    char key[LOG_DIAG_KEY_LEN];
    int key_len;
    uint64_t value;
    int category;
} log_diag_entry_t;

typedef struct {
    log_diag_entry_t entries[LOG_DIAG_MAX];
    int count;
    uint32_t dump_time;
} log_diag_dump_t;

void log_diag_init(log_diag_dump_t *d) {
    d->count = 0;
    d->dump_time = 0;
}

int log_diag_add(log_diag_dump_t *d, const char *key, uint64_t value, int cat) {
    if (d->count >= LOG_DIAG_MAX) return -1;
    int idx = d->count;
    int i;
    for (i = 0; key[i] != 0 && i < LOG_DIAG_KEY_LEN - 1; i++) {
        d->entries[idx].key[i] = key[i];
    }
    d->entries[idx].key[i] = 0;
    d->entries[idx].key_len = i;
    d->entries[idx].value = value;
    d->entries[idx].category = cat;
    d->count++;
    return idx;
}

int log_diag_count_by_category(log_diag_dump_t *d, int cat) {
    int c = 0;
    int i;
    for (i = 0; i < d->count; i++) {
        if (d->entries[i].category == cat) c++;
    }
    return c;
}

void log_diag_stamp(log_diag_dump_t *d, uint32_t ts) {
    d->dump_time = ts;
}

int log_diag_test(void) {
    log_diag_dump_t dump;
    log_diag_init(&dump);
    log_diag_add(&dump, "cpu_usage", 75, 1);
    log_diag_add(&dump, "mem_free", 1024, 1);
    log_diag_add(&dump, "disk_io", 500, 2);
    log_diag_stamp(&dump, 999);
    if (dump.count != 3) return -1;
    if (log_diag_count_by_category(&dump, 1) != 2) return -2;
    if (dump.dump_time != 999) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1624: Diagnostic dump should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1624: Output should not be empty");
    assert!(code.contains("fn log_diag_"), "C1624: Should contain log_diag_ functions");
}

/// C1625: Crash reporter with stack-like backtrace recording
#[test]
fn c1625_crash_reporter() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

#define LOG_CRASH_FRAMES 32
#define LOG_CRASH_REPORTS 4

typedef struct {
    uint64_t address;
    uint32_t offset;
    int module_id;
} log_crash_frame_t;

typedef struct {
    log_crash_frame_t frames[LOG_CRASH_FRAMES];
    int frame_count;
    uint32_t signal_num;
    uint32_t timestamp;
    uint64_t fault_addr;
} log_crash_report_t;

typedef struct {
    log_crash_report_t reports[LOG_CRASH_REPORTS];
    int report_count;
    int current_slot;
} log_crash_mgr_t;

void log_crash_init(log_crash_mgr_t *m) {
    m->report_count = 0;
    m->current_slot = 0;
}

int log_crash_begin(log_crash_mgr_t *m, uint32_t sig, uint64_t fault, uint32_t ts) {
    int slot = m->current_slot;
    m->reports[slot].signal_num = sig;
    m->reports[slot].fault_addr = fault;
    m->reports[slot].timestamp = ts;
    m->reports[slot].frame_count = 0;
    return slot;
}

int log_crash_add_frame(log_crash_mgr_t *m, int slot, uint64_t addr, uint32_t off, int mod_id) {
    if (slot < 0 || slot >= LOG_CRASH_REPORTS) return -1;
    if (m->reports[slot].frame_count >= LOG_CRASH_FRAMES) return -2;
    int idx = m->reports[slot].frame_count;
    m->reports[slot].frames[idx].address = addr;
    m->reports[slot].frames[idx].offset = off;
    m->reports[slot].frames[idx].module_id = mod_id;
    m->reports[slot].frame_count++;
    return idx;
}

void log_crash_commit(log_crash_mgr_t *m) {
    m->current_slot = (m->current_slot + 1) % LOG_CRASH_REPORTS;
    if (m->report_count < LOG_CRASH_REPORTS) {
        m->report_count++;
    }
}

int log_crash_test(void) {
    log_crash_mgr_t mgr;
    log_crash_init(&mgr);
    int slot = log_crash_begin(&mgr, 11, 0xDEAD, 5000);
    log_crash_add_frame(&mgr, slot, 0x400100, 0x10, 0);
    log_crash_add_frame(&mgr, slot, 0x400200, 0x20, 0);
    log_crash_add_frame(&mgr, slot, 0x400300, 0x30, 1);
    log_crash_commit(&mgr);
    if (mgr.report_count != 1) return -1;
    if (mgr.reports[0].frame_count != 3) return -2;
    if (mgr.reports[0].signal_num != 11) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1625: Crash reporter should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1625: Output should not be empty");
    assert!(code.contains("fn log_crash_"), "C1625: Should contain log_crash_ functions");
}
