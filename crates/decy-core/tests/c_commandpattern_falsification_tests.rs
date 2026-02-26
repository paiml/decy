//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1951-C1975: Command/Dispatch Patterns -- dispatch tables, command buffers,
//! pipelines, interpreter patterns, and RPC-style dispatch commonly found in
//! production command-driven architectures.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! Organization:
//! - C1951-C1955: Command dispatch (dispatch table, handler registration, lookup, execute, result)
//! - C1956-C1960: Command buffer (buffered commands, batch execute, replay, clear, history)
//! - C1961-C1965: Command pipeline (chain of commands, pipe result, error propagation, rollback)
//! - C1966-C1970: Interpreter pattern (tokenize, parse command, argument extraction, help text)
//! - C1971-C1975: RPC-style dispatch (method table, serialize args, deserialize result, error codes)

// ============================================================================
// C1951-C1955: Command Dispatch
// ============================================================================

/// C1951: Dispatch table with function pointers and command IDs
#[test]
fn c1951_dispatch_table() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;

#define CMD_MAX_HANDLERS 32

typedef int (*cmd_handler_fn)(int arg);

typedef struct {
    uint32_t cmd_id;
    cmd_handler_fn handler;
    int active;
} cmd_dispatch_entry_t;

typedef struct {
    cmd_dispatch_entry_t table[CMD_MAX_HANDLERS];
    int count;
    int last_result;
} cmd_dispatch_t;

void cmd_dispatch_init(cmd_dispatch_t *d) {
    int i;
    for (i = 0; i < CMD_MAX_HANDLERS; i++) {
        d->table[i].cmd_id = 0;
        d->table[i].handler = 0;
        d->table[i].active = 0;
    }
    d->count = 0;
    d->last_result = 0;
}

int cmd_dispatch_register(cmd_dispatch_t *d, uint32_t cmd_id, cmd_handler_fn fn) {
    if (d->count >= CMD_MAX_HANDLERS) return -1;
    int idx = d->count;
    d->table[idx].cmd_id = cmd_id;
    d->table[idx].handler = fn;
    d->table[idx].active = 1;
    d->count++;
    return idx;
}

int cmd_dispatch_execute(cmd_dispatch_t *d, uint32_t cmd_id, int arg) {
    int i;
    for (i = 0; i < d->count; i++) {
        if (d->table[i].active && d->table[i].cmd_id == cmd_id) {
            if (d->table[i].handler) {
                d->last_result = d->table[i].handler(arg);
                return d->last_result;
            }
        }
    }
    return -1;
}

int cmd_dispatch_count(cmd_dispatch_t *d) {
    int active = 0;
    int i;
    for (i = 0; i < d->count; i++) {
        if (d->table[i].active) active++;
    }
    return active;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1951: Dispatch table should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1951: Output should not be empty");
    assert!(code.contains("fn cmd_dispatch_init"), "C1951: Should contain cmd_dispatch_init");
    assert!(code.contains("fn cmd_dispatch_register"), "C1951: Should contain cmd_dispatch_register");
    assert!(code.contains("fn cmd_dispatch_execute"), "C1951: Should contain cmd_dispatch_execute");
}

/// C1952: Handler registration with priority ordering
#[test]
fn c1952_handler_registration() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define CMD_REG_MAX 16

typedef struct {
    uint32_t id;
    int priority;
    int enabled;
} cmd_reg_entry_t;

typedef struct {
    cmd_reg_entry_t entries[CMD_REG_MAX];
    int count;
} cmd_registry_t;

void cmd_reg_init(cmd_registry_t *r) {
    r->count = 0;
}

int cmd_reg_add(cmd_registry_t *r, uint32_t id, int priority) {
    if (r->count >= CMD_REG_MAX) return -1;
    int pos = r->count;
    int i;
    for (i = r->count - 1; i >= 0; i--) {
        if (r->entries[i].priority < priority) {
            r->entries[i + 1] = r->entries[i];
            pos = i;
        } else {
            break;
        }
    }
    r->entries[pos].id = id;
    r->entries[pos].priority = priority;
    r->entries[pos].enabled = 1;
    r->count++;
    return pos;
}

int cmd_reg_remove(cmd_registry_t *r, uint32_t id) {
    int i;
    for (i = 0; i < r->count; i++) {
        if (r->entries[i].id == id) {
            int j;
            for (j = i; j < r->count - 1; j++) {
                r->entries[j] = r->entries[j + 1];
            }
            r->count--;
            return 0;
        }
    }
    return -1;
}

uint32_t cmd_reg_highest_priority(cmd_registry_t *r) {
    if (r->count == 0) return 0;
    return r->entries[0].id;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1952: Handler registration should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1952: Output should not be empty");
    assert!(code.contains("fn cmd_reg_init"), "C1952: Should contain cmd_reg_init");
    assert!(code.contains("fn cmd_reg_add"), "C1952: Should contain cmd_reg_add");
    assert!(code.contains("fn cmd_reg_remove"), "C1952: Should contain cmd_reg_remove");
}

/// C1953: Command lookup with linear and hash-based search
#[test]
fn c1953_command_lookup() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define CMD_LOOKUP_SLOTS 64
#define CMD_NAME_LEN 16

typedef struct {
    char name[CMD_NAME_LEN];
    uint32_t code;
    int occupied;
} cmd_lookup_slot_t;

typedef struct {
    cmd_lookup_slot_t slots[CMD_LOOKUP_SLOTS];
    int count;
} cmd_lookup_table_t;

static uint32_t cmd_lookup_hash(const char *name) {
    uint32_t h = 5381;
    while (*name) {
        h = ((h << 5) + h) + (uint32_t)*name;
        name++;
    }
    return h % CMD_LOOKUP_SLOTS;
}

void cmd_lookup_init(cmd_lookup_table_t *t) {
    int i;
    for (i = 0; i < CMD_LOOKUP_SLOTS; i++) {
        t->slots[i].occupied = 0;
        t->slots[i].code = 0;
    }
    t->count = 0;
}

int cmd_lookup_insert(cmd_lookup_table_t *t, const char *name, uint32_t code) {
    if (t->count >= CMD_LOOKUP_SLOTS) return -1;
    uint32_t idx = cmd_lookup_hash(name);
    int probes = 0;
    while (t->slots[idx].occupied && probes < CMD_LOOKUP_SLOTS) {
        idx = (idx + 1) % CMD_LOOKUP_SLOTS;
        probes++;
    }
    if (probes >= CMD_LOOKUP_SLOTS) return -1;
    int i;
    for (i = 0; name[i] && i < CMD_NAME_LEN - 1; i++) {
        t->slots[idx].name[i] = name[i];
    }
    t->slots[idx].name[i] = '\0';
    t->slots[idx].code = code;
    t->slots[idx].occupied = 1;
    t->count++;
    return 0;
}

int cmd_lookup_find(cmd_lookup_table_t *t, const char *name, uint32_t *out_code) {
    uint32_t idx = cmd_lookup_hash(name);
    int probes = 0;
    while (t->slots[idx].occupied && probes < CMD_LOOKUP_SLOTS) {
        int match = 1;
        int k;
        for (k = 0; k < CMD_NAME_LEN; k++) {
            if (t->slots[idx].name[k] != name[k]) { match = 0; break; }
            if (name[k] == '\0') break;
        }
        if (match) {
            *out_code = t->slots[idx].code;
            return 0;
        }
        idx = (idx + 1) % CMD_LOOKUP_SLOTS;
        probes++;
    }
    return -1;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1953: Command lookup should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1953: Output should not be empty");
    assert!(code.contains("fn cmd_lookup_init"), "C1953: Should contain cmd_lookup_init");
    assert!(code.contains("fn cmd_lookup_insert"), "C1953: Should contain cmd_lookup_insert");
    assert!(code.contains("fn cmd_lookup_find"), "C1953: Should contain cmd_lookup_find");
}

/// C1954: Command execution with context and error return
#[test]
fn c1954_command_execute() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;

#define CMD_EXEC_OK 0
#define CMD_EXEC_ERR -1
#define CMD_EXEC_NOHANDLER -2
#define CMD_EXEC_MAX_ARGS 8

typedef struct {
    int32_t args[CMD_EXEC_MAX_ARGS];
    int arg_count;
    int32_t return_value;
    int error_code;
} cmd_exec_context_t;

void cmd_exec_ctx_init(cmd_exec_context_t *ctx) {
    int i;
    for (i = 0; i < CMD_EXEC_MAX_ARGS; i++) {
        ctx->args[i] = 0;
    }
    ctx->arg_count = 0;
    ctx->return_value = 0;
    ctx->error_code = CMD_EXEC_OK;
}

int cmd_exec_set_arg(cmd_exec_context_t *ctx, int index, int32_t value) {
    if (index < 0 || index >= CMD_EXEC_MAX_ARGS) return CMD_EXEC_ERR;
    ctx->args[index] = value;
    if (index >= ctx->arg_count) {
        ctx->arg_count = index + 1;
    }
    return CMD_EXEC_OK;
}

int32_t cmd_exec_sum_args(cmd_exec_context_t *ctx) {
    int32_t sum = 0;
    int i;
    for (i = 0; i < ctx->arg_count; i++) {
        sum += ctx->args[i];
    }
    ctx->return_value = sum;
    return sum;
}

int cmd_exec_validate(cmd_exec_context_t *ctx, int min_args) {
    if (ctx->arg_count < min_args) {
        ctx->error_code = CMD_EXEC_ERR;
        return CMD_EXEC_ERR;
    }
    ctx->error_code = CMD_EXEC_OK;
    return CMD_EXEC_OK;
}

int cmd_exec_test(void) {
    cmd_exec_context_t ctx;
    cmd_exec_ctx_init(&ctx);
    cmd_exec_set_arg(&ctx, 0, 10);
    cmd_exec_set_arg(&ctx, 1, 20);
    cmd_exec_set_arg(&ctx, 2, 30);
    if (cmd_exec_validate(&ctx, 3) != CMD_EXEC_OK) return -1;
    int32_t sum = cmd_exec_sum_args(&ctx);
    if (sum != 60) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1954: Command execute should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1954: Output should not be empty");
    assert!(code.contains("fn cmd_exec_ctx_init"), "C1954: Should contain cmd_exec_ctx_init");
    assert!(code.contains("fn cmd_exec_set_arg"), "C1954: Should contain cmd_exec_set_arg");
    assert!(code.contains("fn cmd_exec_sum_args"), "C1954: Should contain cmd_exec_sum_args");
}

/// C1955: Command result aggregation and status reporting
#[test]
fn c1955_command_result() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;

#define CMD_RES_MAX 32
#define CMD_RES_SUCCESS 0
#define CMD_RES_FAILURE 1
#define CMD_RES_TIMEOUT 2

typedef struct {
    uint32_t cmd_id;
    int status;
    int32_t value;
} cmd_result_entry_t;

typedef struct {
    cmd_result_entry_t results[CMD_RES_MAX];
    int count;
    int success_count;
    int failure_count;
} cmd_result_log_t;

void cmd_result_init(cmd_result_log_t *log) {
    log->count = 0;
    log->success_count = 0;
    log->failure_count = 0;
}

int cmd_result_record(cmd_result_log_t *log, uint32_t cmd_id, int status, int32_t value) {
    if (log->count >= CMD_RES_MAX) return -1;
    int idx = log->count;
    log->results[idx].cmd_id = cmd_id;
    log->results[idx].status = status;
    log->results[idx].value = value;
    log->count++;
    if (status == CMD_RES_SUCCESS) {
        log->success_count++;
    } else {
        log->failure_count++;
    }
    return 0;
}

int cmd_result_success_rate(cmd_result_log_t *log) {
    if (log->count == 0) return 0;
    return (log->success_count * 100) / log->count;
}

int32_t cmd_result_last_value(cmd_result_log_t *log) {
    if (log->count == 0) return 0;
    return log->results[log->count - 1].value;
}

int cmd_result_test(void) {
    cmd_result_log_t log;
    cmd_result_init(&log);
    cmd_result_record(&log, 1, CMD_RES_SUCCESS, 42);
    cmd_result_record(&log, 2, CMD_RES_SUCCESS, 99);
    cmd_result_record(&log, 3, CMD_RES_FAILURE, -1);
    int rate = cmd_result_success_rate(&log);
    if (rate != 66) return -1;
    if (cmd_result_last_value(&log) != -1) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1955: Command result should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1955: Output should not be empty");
    assert!(code.contains("fn cmd_result_init"), "C1955: Should contain cmd_result_init");
    assert!(code.contains("fn cmd_result_record"), "C1955: Should contain cmd_result_record");
    assert!(code.contains("fn cmd_result_success_rate"), "C1955: Should contain cmd_result_success_rate");
}

// ============================================================================
// C1956-C1960: Command Buffer
// ============================================================================

/// C1956: Buffered command queue with fixed-size ring buffer
#[test]
fn c1956_buffered_commands() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;

#define CMD_BUF_SIZE 16

typedef struct {
    uint32_t cmd_id;
    int32_t param;
} cmd_buf_entry_t;

typedef struct {
    cmd_buf_entry_t buffer[CMD_BUF_SIZE];
    int head;
    int tail;
    int count;
} cmd_buf_t;

void cmd_buf_init(cmd_buf_t *b) {
    b->head = 0;
    b->tail = 0;
    b->count = 0;
}

int cmd_buf_push(cmd_buf_t *b, uint32_t cmd_id, int32_t param) {
    if (b->count >= CMD_BUF_SIZE) return -1;
    b->buffer[b->tail].cmd_id = cmd_id;
    b->buffer[b->tail].param = param;
    b->tail = (b->tail + 1) % CMD_BUF_SIZE;
    b->count++;
    return 0;
}

int cmd_buf_pop(cmd_buf_t *b, uint32_t *out_id, int32_t *out_param) {
    if (b->count == 0) return -1;
    *out_id = b->buffer[b->head].cmd_id;
    *out_param = b->buffer[b->head].param;
    b->head = (b->head + 1) % CMD_BUF_SIZE;
    b->count--;
    return 0;
}

int cmd_buf_is_empty(cmd_buf_t *b) {
    return b->count == 0;
}

int cmd_buf_is_full(cmd_buf_t *b) {
    return b->count >= CMD_BUF_SIZE;
}

int cmd_buf_test(void) {
    cmd_buf_t buf;
    cmd_buf_init(&buf);
    cmd_buf_push(&buf, 1, 100);
    cmd_buf_push(&buf, 2, 200);
    uint32_t id;
    int32_t param;
    cmd_buf_pop(&buf, &id, &param);
    if (id != 1 || param != 100) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1956: Buffered commands should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1956: Output should not be empty");
    assert!(code.contains("fn cmd_buf_init"), "C1956: Should contain cmd_buf_init");
    assert!(code.contains("fn cmd_buf_push"), "C1956: Should contain cmd_buf_push");
    assert!(code.contains("fn cmd_buf_pop"), "C1956: Should contain cmd_buf_pop");
}

/// C1957: Batch command execution from queue
#[test]
fn c1957_batch_execute() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;

#define CMD_BATCH_MAX 32

typedef struct {
    uint32_t cmd_id;
    int32_t param;
    int executed;
    int32_t result;
} cmd_batch_item_t;

typedef struct {
    cmd_batch_item_t items[CMD_BATCH_MAX];
    int count;
    int executed_count;
    int failed_count;
} cmd_batch_t;

void cmd_batch_init(cmd_batch_t *b) {
    b->count = 0;
    b->executed_count = 0;
    b->failed_count = 0;
}

int cmd_batch_add(cmd_batch_t *b, uint32_t cmd_id, int32_t param) {
    if (b->count >= CMD_BATCH_MAX) return -1;
    b->items[b->count].cmd_id = cmd_id;
    b->items[b->count].param = param;
    b->items[b->count].executed = 0;
    b->items[b->count].result = 0;
    b->count++;
    return 0;
}

int cmd_batch_execute_all(cmd_batch_t *b) {
    int i;
    for (i = 0; i < b->count; i++) {
        if (!b->items[i].executed) {
            b->items[i].result = b->items[i].param * 2;
            b->items[i].executed = 1;
            b->executed_count++;
            if (b->items[i].param < 0) {
                b->failed_count++;
            }
        }
    }
    return b->executed_count;
}

int cmd_batch_pending(cmd_batch_t *b) {
    return b->count - b->executed_count;
}

int cmd_batch_test(void) {
    cmd_batch_t batch;
    cmd_batch_init(&batch);
    cmd_batch_add(&batch, 1, 10);
    cmd_batch_add(&batch, 2, -5);
    cmd_batch_add(&batch, 3, 20);
    int done = cmd_batch_execute_all(&batch);
    if (done != 3) return -1;
    if (cmd_batch_pending(&batch) != 0) return -2;
    if (batch.failed_count != 1) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1957: Batch execute should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1957: Output should not be empty");
    assert!(code.contains("fn cmd_batch_init"), "C1957: Should contain cmd_batch_init");
    assert!(code.contains("fn cmd_batch_add"), "C1957: Should contain cmd_batch_add");
    assert!(code.contains("fn cmd_batch_execute_all"), "C1957: Should contain cmd_batch_execute_all");
}

/// C1958: Command replay from recorded history
#[test]
fn c1958_command_replay() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;

#define CMD_REPLAY_MAX 64

typedef struct {
    uint32_t cmd_id;
    int32_t param;
    uint32_t timestamp;
} cmd_replay_entry_t;

typedef struct {
    cmd_replay_entry_t log[CMD_REPLAY_MAX];
    int count;
    int replay_pos;
    int32_t accumulated;
} cmd_replay_t;

void cmd_replay_init(cmd_replay_t *r) {
    r->count = 0;
    r->replay_pos = 0;
    r->accumulated = 0;
}

int cmd_replay_record(cmd_replay_t *r, uint32_t cmd_id, int32_t param, uint32_t ts) {
    if (r->count >= CMD_REPLAY_MAX) return -1;
    r->log[r->count].cmd_id = cmd_id;
    r->log[r->count].param = param;
    r->log[r->count].timestamp = ts;
    r->count++;
    return 0;
}

int cmd_replay_step(cmd_replay_t *r) {
    if (r->replay_pos >= r->count) return -1;
    r->accumulated += r->log[r->replay_pos].param;
    r->replay_pos++;
    return 0;
}

int cmd_replay_all(cmd_replay_t *r) {
    r->replay_pos = 0;
    r->accumulated = 0;
    while (r->replay_pos < r->count) {
        cmd_replay_step(r);
    }
    return r->replay_pos;
}

void cmd_replay_reset(cmd_replay_t *r) {
    r->replay_pos = 0;
    r->accumulated = 0;
}

int cmd_replay_test(void) {
    cmd_replay_t rep;
    cmd_replay_init(&rep);
    cmd_replay_record(&rep, 1, 10, 1000);
    cmd_replay_record(&rep, 2, 20, 1001);
    cmd_replay_record(&rep, 3, 30, 1002);
    int replayed = cmd_replay_all(&rep);
    if (replayed != 3) return -1;
    if (rep.accumulated != 60) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1958: Command replay should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1958: Output should not be empty");
    assert!(code.contains("fn cmd_replay_init"), "C1958: Should contain cmd_replay_init");
    assert!(code.contains("fn cmd_replay_record"), "C1958: Should contain cmd_replay_record");
    assert!(code.contains("fn cmd_replay_all"), "C1958: Should contain cmd_replay_all");
}

/// C1959: Command queue clear and drain operations
#[test]
fn c1959_command_clear() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;

#define CMD_CLR_MAX 24

typedef struct {
    uint32_t cmd_id;
    int32_t data;
    int valid;
} cmd_clr_entry_t;

typedef struct {
    cmd_clr_entry_t entries[CMD_CLR_MAX];
    int count;
    int drain_count;
} cmd_clr_queue_t;

void cmd_clr_init(cmd_clr_queue_t *q) {
    int i;
    for (i = 0; i < CMD_CLR_MAX; i++) {
        q->entries[i].valid = 0;
    }
    q->count = 0;
    q->drain_count = 0;
}

int cmd_clr_enqueue(cmd_clr_queue_t *q, uint32_t cmd_id, int32_t data) {
    if (q->count >= CMD_CLR_MAX) return -1;
    q->entries[q->count].cmd_id = cmd_id;
    q->entries[q->count].data = data;
    q->entries[q->count].valid = 1;
    q->count++;
    return 0;
}

int cmd_clr_drain(cmd_clr_queue_t *q, int32_t *sum_out) {
    int32_t sum = 0;
    int drained = 0;
    int i;
    for (i = 0; i < q->count; i++) {
        if (q->entries[i].valid) {
            sum += q->entries[i].data;
            q->entries[i].valid = 0;
            drained++;
        }
    }
    *sum_out = sum;
    q->drain_count += drained;
    q->count = 0;
    return drained;
}

void cmd_clr_clear(cmd_clr_queue_t *q) {
    int i;
    for (i = 0; i < CMD_CLR_MAX; i++) {
        q->entries[i].valid = 0;
    }
    q->count = 0;
}

int cmd_clr_test(void) {
    cmd_clr_queue_t q;
    cmd_clr_init(&q);
    cmd_clr_enqueue(&q, 1, 5);
    cmd_clr_enqueue(&q, 2, 15);
    int32_t sum;
    int n = cmd_clr_drain(&q, &sum);
    if (n != 2 || sum != 20) return -1;
    cmd_clr_clear(&q);
    if (q.count != 0) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1959: Command clear should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1959: Output should not be empty");
    assert!(code.contains("fn cmd_clr_init"), "C1959: Should contain cmd_clr_init");
    assert!(code.contains("fn cmd_clr_enqueue"), "C1959: Should contain cmd_clr_enqueue");
    assert!(code.contains("fn cmd_clr_drain"), "C1959: Should contain cmd_clr_drain");
}

/// C1960: Command history with undo tracking
#[test]
fn c1960_command_history() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;

#define CMD_HIST_MAX 48

typedef struct {
    uint32_t cmd_id;
    int32_t before_value;
    int32_t after_value;
} cmd_hist_entry_t;

typedef struct {
    cmd_hist_entry_t history[CMD_HIST_MAX];
    int count;
    int undo_pos;
    int32_t current_value;
} cmd_hist_t;

void cmd_hist_init(cmd_hist_t *h, int32_t initial) {
    h->count = 0;
    h->undo_pos = 0;
    h->current_value = initial;
}

int cmd_hist_apply(cmd_hist_t *h, uint32_t cmd_id, int32_t delta) {
    if (h->count >= CMD_HIST_MAX) return -1;
    h->history[h->count].cmd_id = cmd_id;
    h->history[h->count].before_value = h->current_value;
    h->current_value += delta;
    h->history[h->count].after_value = h->current_value;
    h->count++;
    h->undo_pos = h->count;
    return 0;
}

int cmd_hist_undo(cmd_hist_t *h) {
    if (h->undo_pos <= 0) return -1;
    h->undo_pos--;
    h->current_value = h->history[h->undo_pos].before_value;
    return 0;
}

int cmd_hist_redo(cmd_hist_t *h) {
    if (h->undo_pos >= h->count) return -1;
    h->current_value = h->history[h->undo_pos].after_value;
    h->undo_pos++;
    return 0;
}

int cmd_hist_test(void) {
    cmd_hist_t h;
    cmd_hist_init(&h, 100);
    cmd_hist_apply(&h, 1, 10);
    cmd_hist_apply(&h, 2, 20);
    if (h.current_value != 130) return -1;
    cmd_hist_undo(&h);
    if (h.current_value != 110) return -2;
    cmd_hist_redo(&h);
    if (h.current_value != 130) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1960: Command history should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1960: Output should not be empty");
    assert!(code.contains("fn cmd_hist_init"), "C1960: Should contain cmd_hist_init");
    assert!(code.contains("fn cmd_hist_apply"), "C1960: Should contain cmd_hist_apply");
    assert!(code.contains("fn cmd_hist_undo"), "C1960: Should contain cmd_hist_undo");
}

// ============================================================================
// C1961-C1965: Command Pipeline
// ============================================================================

/// C1961: Chain of commands with sequential execution
#[test]
fn c1961_command_chain() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;

#define CMD_CHAIN_MAX 16

typedef struct {
    uint32_t stage_id;
    int32_t multiplier;
    int enabled;
} cmd_chain_stage_t;

typedef struct {
    cmd_chain_stage_t stages[CMD_CHAIN_MAX];
    int count;
    int32_t accumulator;
} cmd_chain_t;

void cmd_chain_init(cmd_chain_t *c) {
    c->count = 0;
    c->accumulator = 0;
}

int cmd_chain_add_stage(cmd_chain_t *c, uint32_t stage_id, int32_t mult) {
    if (c->count >= CMD_CHAIN_MAX) return -1;
    c->stages[c->count].stage_id = stage_id;
    c->stages[c->count].multiplier = mult;
    c->stages[c->count].enabled = 1;
    c->count++;
    return 0;
}

int32_t cmd_chain_execute(cmd_chain_t *c, int32_t input) {
    int32_t value = input;
    int i;
    for (i = 0; i < c->count; i++) {
        if (c->stages[i].enabled) {
            value = value * c->stages[i].multiplier;
        }
    }
    c->accumulator = value;
    return value;
}

void cmd_chain_disable_stage(cmd_chain_t *c, int index) {
    if (index >= 0 && index < c->count) {
        c->stages[index].enabled = 0;
    }
}

int cmd_chain_test(void) {
    cmd_chain_t chain;
    cmd_chain_init(&chain);
    cmd_chain_add_stage(&chain, 1, 2);
    cmd_chain_add_stage(&chain, 2, 3);
    int32_t result = cmd_chain_execute(&chain, 5);
    if (result != 30) return -1;
    cmd_chain_disable_stage(&chain, 0);
    result = cmd_chain_execute(&chain, 5);
    if (result != 15) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1961: Command chain should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1961: Output should not be empty");
    assert!(code.contains("fn cmd_chain_init"), "C1961: Should contain cmd_chain_init");
    assert!(code.contains("fn cmd_chain_add_stage"), "C1961: Should contain cmd_chain_add_stage");
    assert!(code.contains("fn cmd_chain_execute"), "C1961: Should contain cmd_chain_execute");
}

/// C1962: Pipe results between command stages
#[test]
fn c1962_pipe_result() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;

#define CMD_PIPE_MAX 8

typedef struct {
    int32_t value;
    int valid;
    int error;
} cmd_pipe_data_t;

typedef struct {
    cmd_pipe_data_t slots[CMD_PIPE_MAX];
    int write_pos;
    int read_pos;
    int count;
} cmd_pipe_t;

void cmd_pipe_init(cmd_pipe_t *p) {
    p->write_pos = 0;
    p->read_pos = 0;
    p->count = 0;
    int i;
    for (i = 0; i < CMD_PIPE_MAX; i++) {
        p->slots[i].valid = 0;
        p->slots[i].error = 0;
    }
}

int cmd_pipe_write(cmd_pipe_t *p, int32_t value) {
    if (p->count >= CMD_PIPE_MAX) return -1;
    p->slots[p->write_pos].value = value;
    p->slots[p->write_pos].valid = 1;
    p->slots[p->write_pos].error = 0;
    p->write_pos = (p->write_pos + 1) % CMD_PIPE_MAX;
    p->count++;
    return 0;
}

int cmd_pipe_read(cmd_pipe_t *p, int32_t *out) {
    if (p->count == 0) return -1;
    if (!p->slots[p->read_pos].valid) return -2;
    *out = p->slots[p->read_pos].value;
    p->slots[p->read_pos].valid = 0;
    p->read_pos = (p->read_pos + 1) % CMD_PIPE_MAX;
    p->count--;
    return 0;
}

int cmd_pipe_transform(cmd_pipe_t *src, cmd_pipe_t *dst, int32_t addend) {
    int32_t val;
    int transformed = 0;
    while (cmd_pipe_read(src, &val) == 0) {
        cmd_pipe_write(dst, val + addend);
        transformed++;
    }
    return transformed;
}

int cmd_pipe_test(void) {
    cmd_pipe_t p1, p2;
    cmd_pipe_init(&p1);
    cmd_pipe_init(&p2);
    cmd_pipe_write(&p1, 10);
    cmd_pipe_write(&p1, 20);
    int n = cmd_pipe_transform(&p1, &p2, 100);
    if (n != 2) return -1;
    int32_t val;
    cmd_pipe_read(&p2, &val);
    if (val != 110) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1962: Pipe result should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1962: Output should not be empty");
    assert!(code.contains("fn cmd_pipe_init"), "C1962: Should contain cmd_pipe_init");
    assert!(code.contains("fn cmd_pipe_write"), "C1962: Should contain cmd_pipe_write");
    assert!(code.contains("fn cmd_pipe_read"), "C1962: Should contain cmd_pipe_read");
}

/// C1963: Error propagation through command pipeline
#[test]
fn c1963_error_propagation() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;

#define CMD_ERR_NONE 0
#define CMD_ERR_OVERFLOW 1
#define CMD_ERR_UNDERFLOW 2
#define CMD_ERR_INVALID 3
#define CMD_ERR_CHAIN_MAX 8

typedef struct {
    int error_codes[CMD_ERR_CHAIN_MAX];
    int count;
    int first_error;
    int propagated;
} cmd_err_chain_t;

void cmd_err_init(cmd_err_chain_t *e) {
    int i;
    for (i = 0; i < CMD_ERR_CHAIN_MAX; i++) {
        e->error_codes[i] = CMD_ERR_NONE;
    }
    e->count = 0;
    e->first_error = CMD_ERR_NONE;
    e->propagated = 0;
}

int cmd_err_push(cmd_err_chain_t *e, int code) {
    if (e->count >= CMD_ERR_CHAIN_MAX) return -1;
    e->error_codes[e->count] = code;
    if (e->first_error == CMD_ERR_NONE && code != CMD_ERR_NONE) {
        e->first_error = code;
    }
    e->count++;
    return 0;
}

int cmd_err_has_error(cmd_err_chain_t *e) {
    return e->first_error != CMD_ERR_NONE;
}

int cmd_err_propagate(cmd_err_chain_t *src, cmd_err_chain_t *dst) {
    if (!cmd_err_has_error(src)) return 0;
    cmd_err_push(dst, src->first_error);
    dst->propagated = 1;
    return 1;
}

int cmd_err_count_errors(cmd_err_chain_t *e) {
    int n = 0;
    int i;
    for (i = 0; i < e->count; i++) {
        if (e->error_codes[i] != CMD_ERR_NONE) n++;
    }
    return n;
}

int cmd_err_test(void) {
    cmd_err_chain_t e1, e2;
    cmd_err_init(&e1);
    cmd_err_init(&e2);
    cmd_err_push(&e1, CMD_ERR_NONE);
    cmd_err_push(&e1, CMD_ERR_OVERFLOW);
    cmd_err_push(&e1, CMD_ERR_INVALID);
    if (cmd_err_count_errors(&e1) != 2) return -1;
    cmd_err_propagate(&e1, &e2);
    if (e2.first_error != CMD_ERR_OVERFLOW) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1963: Error propagation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1963: Output should not be empty");
    assert!(code.contains("fn cmd_err_init"), "C1963: Should contain cmd_err_init");
    assert!(code.contains("fn cmd_err_push"), "C1963: Should contain cmd_err_push");
    assert!(code.contains("fn cmd_err_propagate"), "C1963: Should contain cmd_err_propagate");
}

/// C1964: Transaction rollback for failed command sequences
#[test]
fn c1964_command_rollback() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;

#define CMD_TXN_MAX 16

typedef struct {
    int32_t old_value;
    int32_t new_value;
    int committed;
} cmd_txn_op_t;

typedef struct {
    cmd_txn_op_t ops[CMD_TXN_MAX];
    int count;
    int32_t state;
    int rolled_back;
} cmd_txn_t;

void cmd_txn_init(cmd_txn_t *t, int32_t initial) {
    t->count = 0;
    t->state = initial;
    t->rolled_back = 0;
}

int cmd_txn_apply(cmd_txn_t *t, int32_t delta) {
    if (t->count >= CMD_TXN_MAX) return -1;
    t->ops[t->count].old_value = t->state;
    t->state += delta;
    t->ops[t->count].new_value = t->state;
    t->ops[t->count].committed = 0;
    t->count++;
    return 0;
}

int cmd_txn_commit(cmd_txn_t *t) {
    int i;
    for (i = 0; i < t->count; i++) {
        t->ops[i].committed = 1;
    }
    return t->count;
}

int cmd_txn_rollback(cmd_txn_t *t) {
    if (t->count == 0) return -1;
    t->state = t->ops[0].old_value;
    t->count = 0;
    t->rolled_back = 1;
    return 0;
}

int cmd_txn_rollback_last(cmd_txn_t *t) {
    if (t->count == 0) return -1;
    t->count--;
    t->state = t->ops[t->count].old_value;
    return 0;
}

int cmd_txn_test(void) {
    cmd_txn_t txn;
    cmd_txn_init(&txn, 50);
    cmd_txn_apply(&txn, 10);
    cmd_txn_apply(&txn, 20);
    if (txn.state != 80) return -1;
    cmd_txn_rollback(&txn);
    if (txn.state != 50) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1964: Command rollback should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1964: Output should not be empty");
    assert!(code.contains("fn cmd_txn_init"), "C1964: Should contain cmd_txn_init");
    assert!(code.contains("fn cmd_txn_apply"), "C1964: Should contain cmd_txn_apply");
    assert!(code.contains("fn cmd_txn_rollback"), "C1964: Should contain cmd_txn_rollback");
}

/// C1965: Pipeline stage with conditional bypass
#[test]
fn c1965_pipeline_bypass() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;

#define CMD_PL_MAX_STAGES 12

typedef struct {
    uint32_t id;
    int32_t threshold;
    int bypass;
    int invocations;
} cmd_pl_stage_t;

typedef struct {
    cmd_pl_stage_t stages[CMD_PL_MAX_STAGES];
    int count;
    int32_t last_output;
} cmd_pipeline_t;

void cmd_pipeline_init(cmd_pipeline_t *p) {
    p->count = 0;
    p->last_output = 0;
}

int cmd_pipeline_add(cmd_pipeline_t *p, uint32_t id, int32_t threshold) {
    if (p->count >= CMD_PL_MAX_STAGES) return -1;
    p->stages[p->count].id = id;
    p->stages[p->count].threshold = threshold;
    p->stages[p->count].bypass = 0;
    p->stages[p->count].invocations = 0;
    p->count++;
    return 0;
}

int32_t cmd_pipeline_run(cmd_pipeline_t *p, int32_t input) {
    int32_t val = input;
    int i;
    for (i = 0; i < p->count; i++) {
        if (p->stages[i].bypass) continue;
        p->stages[i].invocations++;
        if (val > p->stages[i].threshold) {
            val = val - p->stages[i].threshold;
        } else {
            val = val + p->stages[i].threshold;
        }
    }
    p->last_output = val;
    return val;
}

void cmd_pipeline_set_bypass(cmd_pipeline_t *p, int idx, int bypass) {
    if (idx >= 0 && idx < p->count) {
        p->stages[idx].bypass = bypass;
    }
}

int cmd_pipeline_test(void) {
    cmd_pipeline_t pl;
    cmd_pipeline_init(&pl);
    cmd_pipeline_add(&pl, 1, 10);
    cmd_pipeline_add(&pl, 2, 5);
    int32_t r = cmd_pipeline_run(&pl, 15);
    if (r != 10) return -1;
    cmd_pipeline_set_bypass(&pl, 0, 1);
    r = cmd_pipeline_run(&pl, 3);
    if (r != 8) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1965: Pipeline bypass should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1965: Output should not be empty");
    assert!(code.contains("fn cmd_pipeline_init"), "C1965: Should contain cmd_pipeline_init");
    assert!(code.contains("fn cmd_pipeline_add"), "C1965: Should contain cmd_pipeline_add");
    assert!(code.contains("fn cmd_pipeline_run"), "C1965: Should contain cmd_pipeline_run");
}

// ============================================================================
// C1966-C1970: Interpreter Pattern
// ============================================================================

/// C1966: Command tokenizer splitting input into tokens
#[test]
fn c1966_tokenize() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define CMD_TOK_MAX 16
#define CMD_TOK_LEN 32

typedef struct {
    char tokens[CMD_TOK_MAX][CMD_TOK_LEN];
    int count;
} cmd_tokenizer_t;

void cmd_tok_init(cmd_tokenizer_t *t) {
    t->count = 0;
}

int cmd_tok_split(cmd_tokenizer_t *t, const char *input, char delim) {
    t->count = 0;
    int pos = 0;
    int tok_pos = 0;
    while (input[pos] != '\0' && t->count < CMD_TOK_MAX) {
        if (input[pos] == delim) {
            if (tok_pos > 0) {
                t->tokens[t->count][tok_pos] = '\0';
                t->count++;
                tok_pos = 0;
            }
        } else {
            if (tok_pos < CMD_TOK_LEN - 1) {
                t->tokens[t->count][tok_pos] = input[pos];
                tok_pos++;
            }
        }
        pos++;
    }
    if (tok_pos > 0) {
        t->tokens[t->count][tok_pos] = '\0';
        t->count++;
    }
    return t->count;
}

int cmd_tok_get_count(cmd_tokenizer_t *t) {
    return t->count;
}

int cmd_tok_compare(cmd_tokenizer_t *t, int index, const char *expected) {
    if (index < 0 || index >= t->count) return 0;
    int i = 0;
    while (t->tokens[index][i] != '\0' && expected[i] != '\0') {
        if (t->tokens[index][i] != expected[i]) return 0;
        i++;
    }
    return (t->tokens[index][i] == expected[i]);
}

int cmd_tok_test(void) {
    cmd_tokenizer_t tok;
    cmd_tok_init(&tok);
    int n = cmd_tok_split(&tok, "hello world foo", ' ');
    if (n != 3) return -1;
    if (!cmd_tok_compare(&tok, 0, "hello")) return -2;
    if (!cmd_tok_compare(&tok, 2, "foo")) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1966: Tokenize should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1966: Output should not be empty");
    assert!(code.contains("fn cmd_tok_init"), "C1966: Should contain cmd_tok_init");
    assert!(code.contains("fn cmd_tok_split"), "C1966: Should contain cmd_tok_split");
    assert!(code.contains("fn cmd_tok_compare"), "C1966: Should contain cmd_tok_compare");
}

/// C1967: Parse command name and arguments from token stream
#[test]
fn c1967_parse_command() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;

#define CMD_PARSE_MAX_ARGS 8
#define CMD_PARSE_NAME_LEN 16

typedef struct {
    char name[CMD_PARSE_NAME_LEN];
    int32_t args[CMD_PARSE_MAX_ARGS];
    int arg_count;
    int valid;
} cmd_parsed_t;

void cmd_parse_init(cmd_parsed_t *p) {
    p->arg_count = 0;
    p->valid = 0;
    p->name[0] = '\0';
}

static int cmd_parse_is_digit(char c) {
    return c >= '0' && c <= '9';
}

static int32_t cmd_parse_atoi(const char *s) {
    int32_t val = 0;
    int neg = 0;
    int i = 0;
    if (s[0] == '-') { neg = 1; i = 1; }
    while (cmd_parse_is_digit(s[i])) {
        val = val * 10 + (s[i] - '0');
        i++;
    }
    return neg ? -val : val;
}

int cmd_parse_line(cmd_parsed_t *p, const char *line) {
    cmd_parse_init(p);
    int pos = 0;
    int name_pos = 0;
    while (line[pos] != '\0' && line[pos] != ' ' && name_pos < CMD_PARSE_NAME_LEN - 1) {
        p->name[name_pos++] = line[pos++];
    }
    p->name[name_pos] = '\0';
    if (name_pos == 0) return -1;
    while (line[pos] == ' ') pos++;
    while (line[pos] != '\0' && p->arg_count < CMD_PARSE_MAX_ARGS) {
        char buf[16];
        int bi = 0;
        while (line[pos] != '\0' && line[pos] != ' ' && bi < 15) {
            buf[bi++] = line[pos++];
        }
        buf[bi] = '\0';
        if (bi > 0) {
            p->args[p->arg_count] = cmd_parse_atoi(buf);
            p->arg_count++;
        }
        while (line[pos] == ' ') pos++;
    }
    p->valid = 1;
    return 0;
}

int cmd_parse_test(void) {
    cmd_parsed_t p;
    cmd_parse_line(&p, "add 10 20 30");
    if (!p.valid) return -1;
    if (p.arg_count != 3) return -2;
    if (p.args[0] != 10 || p.args[2] != 30) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1967: Parse command should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1967: Output should not be empty");
    assert!(code.contains("fn cmd_parse_init"), "C1967: Should contain cmd_parse_init");
    assert!(code.contains("fn cmd_parse_line"), "C1967: Should contain cmd_parse_line");
}

/// C1968: Argument extraction with type checking
#[test]
fn c1968_argument_extraction() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;

#define CMD_ARG_MAX 10
#define CMD_ARG_INT 1
#define CMD_ARG_BOOL 2
#define CMD_ARG_FLAG 3

typedef struct {
    int type;
    int32_t int_val;
    int bool_val;
    int present;
} cmd_arg_t;

typedef struct {
    cmd_arg_t args[CMD_ARG_MAX];
    int count;
    int errors;
} cmd_arg_set_t;

void cmd_arg_set_init(cmd_arg_set_t *s) {
    s->count = 0;
    s->errors = 0;
    int i;
    for (i = 0; i < CMD_ARG_MAX; i++) {
        s->args[i].present = 0;
        s->args[i].type = 0;
    }
}

int cmd_arg_add_int(cmd_arg_set_t *s, int32_t value) {
    if (s->count >= CMD_ARG_MAX) return -1;
    s->args[s->count].type = CMD_ARG_INT;
    s->args[s->count].int_val = value;
    s->args[s->count].present = 1;
    s->count++;
    return 0;
}

int cmd_arg_add_bool(cmd_arg_set_t *s, int value) {
    if (s->count >= CMD_ARG_MAX) return -1;
    s->args[s->count].type = CMD_ARG_BOOL;
    s->args[s->count].bool_val = value != 0;
    s->args[s->count].present = 1;
    s->count++;
    return 0;
}

int cmd_arg_get_int(cmd_arg_set_t *s, int index, int32_t *out) {
    if (index < 0 || index >= s->count) return -1;
    if (s->args[index].type != CMD_ARG_INT) {
        s->errors++;
        return -2;
    }
    *out = s->args[index].int_val;
    return 0;
}

int cmd_arg_validate(cmd_arg_set_t *s, int min_count) {
    if (s->count < min_count) return -1;
    int i;
    for (i = 0; i < s->count; i++) {
        if (!s->args[i].present) return -2;
    }
    return 0;
}

int cmd_arg_test(void) {
    cmd_arg_set_t args;
    cmd_arg_set_init(&args);
    cmd_arg_add_int(&args, 42);
    cmd_arg_add_bool(&args, 1);
    cmd_arg_add_int(&args, 99);
    if (cmd_arg_validate(&args, 3) != 0) return -1;
    int32_t val;
    if (cmd_arg_get_int(&args, 0, &val) != 0) return -2;
    if (val != 42) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1968: Argument extraction should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1968: Output should not be empty");
    assert!(code.contains("fn cmd_arg_set_init"), "C1968: Should contain cmd_arg_set_init");
    assert!(code.contains("fn cmd_arg_add_int"), "C1968: Should contain cmd_arg_add_int");
    assert!(code.contains("fn cmd_arg_validate"), "C1968: Should contain cmd_arg_validate");
}

/// C1969: Help text generation from command metadata
#[test]
fn c1969_help_text() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define CMD_HELP_MAX 16
#define CMD_HELP_NAME_LEN 20
#define CMD_HELP_DESC_LEN 64

typedef struct {
    char name[CMD_HELP_NAME_LEN];
    char desc[CMD_HELP_DESC_LEN];
    int arg_count;
    int hidden;
} cmd_help_entry_t;

typedef struct {
    cmd_help_entry_t entries[CMD_HELP_MAX];
    int count;
} cmd_help_registry_t;

void cmd_help_init(cmd_help_registry_t *r) {
    r->count = 0;
}

static void cmd_help_strcpy(char *dst, const char *src, int max_len) {
    int i;
    for (i = 0; src[i] && i < max_len - 1; i++) {
        dst[i] = src[i];
    }
    dst[i] = '\0';
}

int cmd_help_register(cmd_help_registry_t *r, const char *name, const char *desc, int args) {
    if (r->count >= CMD_HELP_MAX) return -1;
    cmd_help_strcpy(r->entries[r->count].name, name, CMD_HELP_NAME_LEN);
    cmd_help_strcpy(r->entries[r->count].desc, desc, CMD_HELP_DESC_LEN);
    r->entries[r->count].arg_count = args;
    r->entries[r->count].hidden = 0;
    r->count++;
    return 0;
}

void cmd_help_set_hidden(cmd_help_registry_t *r, int index) {
    if (index >= 0 && index < r->count) {
        r->entries[index].hidden = 1;
    }
}

int cmd_help_visible_count(cmd_help_registry_t *r) {
    int n = 0;
    int i;
    for (i = 0; i < r->count; i++) {
        if (!r->entries[i].hidden) n++;
    }
    return n;
}

int cmd_help_find(cmd_help_registry_t *r, const char *name) {
    int i;
    for (i = 0; i < r->count; i++) {
        int j = 0;
        int match = 1;
        while (r->entries[i].name[j] != '\0' || name[j] != '\0') {
            if (r->entries[i].name[j] != name[j]) { match = 0; break; }
            j++;
        }
        if (match) return i;
    }
    return -1;
}

int cmd_help_test(void) {
    cmd_help_registry_t reg;
    cmd_help_init(&reg);
    cmd_help_register(&reg, "add", "Add values", 2);
    cmd_help_register(&reg, "sub", "Subtract values", 2);
    cmd_help_register(&reg, "debug", "Debug mode", 0);
    cmd_help_set_hidden(&reg, 2);
    if (cmd_help_visible_count(&reg) != 2) return -1;
    if (cmd_help_find(&reg, "add") != 0) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1969: Help text should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1969: Output should not be empty");
    assert!(code.contains("fn cmd_help_init"), "C1969: Should contain cmd_help_init");
    assert!(code.contains("fn cmd_help_register"), "C1969: Should contain cmd_help_register");
    assert!(code.contains("fn cmd_help_visible_count"), "C1969: Should contain cmd_help_visible_count");
}

/// C1970: Interpreter loop with command evaluation
#[test]
fn c1970_interpreter_loop() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;

#define CMD_INTERP_STACK 16
#define CMD_INTERP_OP_ADD 1
#define CMD_INTERP_OP_SUB 2
#define CMD_INTERP_OP_MUL 3
#define CMD_INTERP_OP_DUP 4
#define CMD_INTERP_OP_HALT 0

typedef struct {
    int32_t stack[CMD_INTERP_STACK];
    int sp;
    int halted;
    int steps;
} cmd_interp_t;

void cmd_interp_init(cmd_interp_t *interp) {
    interp->sp = 0;
    interp->halted = 0;
    interp->steps = 0;
}

int cmd_interp_push(cmd_interp_t *interp, int32_t val) {
    if (interp->sp >= CMD_INTERP_STACK) return -1;
    interp->stack[interp->sp++] = val;
    return 0;
}

int32_t cmd_interp_pop(cmd_interp_t *interp) {
    if (interp->sp <= 0) return 0;
    return interp->stack[--interp->sp];
}

int cmd_interp_exec_op(cmd_interp_t *interp, int op) {
    int32_t a, b;
    interp->steps++;
    switch (op) {
        case CMD_INTERP_OP_ADD:
            a = cmd_interp_pop(interp);
            b = cmd_interp_pop(interp);
            cmd_interp_push(interp, a + b);
            break;
        case CMD_INTERP_OP_SUB:
            a = cmd_interp_pop(interp);
            b = cmd_interp_pop(interp);
            cmd_interp_push(interp, b - a);
            break;
        case CMD_INTERP_OP_MUL:
            a = cmd_interp_pop(interp);
            b = cmd_interp_pop(interp);
            cmd_interp_push(interp, a * b);
            break;
        case CMD_INTERP_OP_DUP:
            if (interp->sp > 0) {
                cmd_interp_push(interp, interp->stack[interp->sp - 1]);
            }
            break;
        case CMD_INTERP_OP_HALT:
            interp->halted = 1;
            break;
        default:
            return -1;
    }
    return 0;
}

int cmd_interp_run(cmd_interp_t *interp, const int *program, int len) {
    int pc;
    for (pc = 0; pc < len && !interp->halted; pc++) {
        cmd_interp_exec_op(interp, program[pc]);
    }
    return interp->steps;
}

int cmd_interp_test(void) {
    cmd_interp_t interp;
    cmd_interp_init(&interp);
    cmd_interp_push(&interp, 5);
    cmd_interp_push(&interp, 3);
    cmd_interp_exec_op(&interp, CMD_INTERP_OP_ADD);
    int32_t result = cmd_interp_pop(&interp);
    if (result != 8) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1970: Interpreter loop should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1970: Output should not be empty");
    assert!(code.contains("fn cmd_interp_init"), "C1970: Should contain cmd_interp_init");
    assert!(code.contains("fn cmd_interp_push"), "C1970: Should contain cmd_interp_push");
    assert!(code.contains("fn cmd_interp_exec_op"), "C1970: Should contain cmd_interp_exec_op");
}

// ============================================================================
// C1971-C1975: RPC-Style Dispatch
// ============================================================================

/// C1971: Method table with ID-based dispatch
#[test]
fn c1971_method_table() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;

#define CMD_METHOD_MAX 20

typedef struct {
    uint32_t method_id;
    int32_t param_count;
    int active;
    int call_count;
} cmd_method_entry_t;

typedef struct {
    cmd_method_entry_t methods[CMD_METHOD_MAX];
    int count;
    int total_calls;
} cmd_method_table_t;

void cmd_method_init(cmd_method_table_t *t) {
    t->count = 0;
    t->total_calls = 0;
}

int cmd_method_register(cmd_method_table_t *t, uint32_t id, int32_t params) {
    if (t->count >= CMD_METHOD_MAX) return -1;
    t->methods[t->count].method_id = id;
    t->methods[t->count].param_count = params;
    t->methods[t->count].active = 1;
    t->methods[t->count].call_count = 0;
    t->count++;
    return 0;
}

int cmd_method_find(cmd_method_table_t *t, uint32_t id) {
    int i;
    for (i = 0; i < t->count; i++) {
        if (t->methods[i].method_id == id && t->methods[i].active) {
            return i;
        }
    }
    return -1;
}

int cmd_method_invoke(cmd_method_table_t *t, uint32_t id, int32_t nargs) {
    int idx = cmd_method_find(t, id);
    if (idx < 0) return -1;
    if (nargs != t->methods[idx].param_count) return -2;
    t->methods[idx].call_count++;
    t->total_calls++;
    return 0;
}

void cmd_method_deactivate(cmd_method_table_t *t, uint32_t id) {
    int idx = cmd_method_find(t, id);
    if (idx >= 0) {
        t->methods[idx].active = 0;
    }
}

int cmd_method_test(void) {
    cmd_method_table_t tbl;
    cmd_method_init(&tbl);
    cmd_method_register(&tbl, 100, 2);
    cmd_method_register(&tbl, 200, 1);
    if (cmd_method_invoke(&tbl, 100, 2) != 0) return -1;
    if (cmd_method_invoke(&tbl, 100, 3) != -2) return -2;
    cmd_method_deactivate(&tbl, 200);
    if (cmd_method_find(&tbl, 200) != -1) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1971: Method table should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1971: Output should not be empty");
    assert!(code.contains("fn cmd_method_init"), "C1971: Should contain cmd_method_init");
    assert!(code.contains("fn cmd_method_register"), "C1971: Should contain cmd_method_register");
    assert!(code.contains("fn cmd_method_invoke"), "C1971: Should contain cmd_method_invoke");
}

/// C1972: Serialize arguments into byte buffer
#[test]
fn c1972_serialize_args() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;
typedef unsigned char uint8_t;

#define CMD_SER_BUF_SIZE 128

typedef struct {
    uint8_t data[CMD_SER_BUF_SIZE];
    int pos;
    int len;
} cmd_ser_buf_t;

void cmd_ser_init(cmd_ser_buf_t *b) {
    b->pos = 0;
    b->len = 0;
}

int cmd_ser_write_u32(cmd_ser_buf_t *b, uint32_t val) {
    if (b->pos + 4 > CMD_SER_BUF_SIZE) return -1;
    b->data[b->pos++] = (uint8_t)(val & 0xFF);
    b->data[b->pos++] = (uint8_t)((val >> 8) & 0xFF);
    b->data[b->pos++] = (uint8_t)((val >> 16) & 0xFF);
    b->data[b->pos++] = (uint8_t)((val >> 24) & 0xFF);
    b->len = b->pos;
    return 0;
}

int cmd_ser_write_i32(cmd_ser_buf_t *b, int32_t val) {
    return cmd_ser_write_u32(b, (uint32_t)val);
}

uint32_t cmd_ser_read_u32(cmd_ser_buf_t *b) {
    if (b->pos + 4 > b->len) return 0;
    uint32_t val = (uint32_t)b->data[b->pos]
        | ((uint32_t)b->data[b->pos + 1] << 8)
        | ((uint32_t)b->data[b->pos + 2] << 16)
        | ((uint32_t)b->data[b->pos + 3] << 24);
    b->pos += 4;
    return val;
}

void cmd_ser_reset_read(cmd_ser_buf_t *b) {
    b->pos = 0;
}

int cmd_ser_remaining(cmd_ser_buf_t *b) {
    return b->len - b->pos;
}

int cmd_ser_test(void) {
    cmd_ser_buf_t buf;
    cmd_ser_init(&buf);
    cmd_ser_write_u32(&buf, 0xDEADBEEF);
    cmd_ser_write_i32(&buf, -1);
    cmd_ser_reset_read(&buf);
    uint32_t v1 = cmd_ser_read_u32(&buf);
    uint32_t v2 = cmd_ser_read_u32(&buf);
    if (v1 != 0xDEADBEEF) return -1;
    if (v2 != 0xFFFFFFFF) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1972: Serialize args should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1972: Output should not be empty");
    assert!(code.contains("fn cmd_ser_init"), "C1972: Should contain cmd_ser_init");
    assert!(code.contains("fn cmd_ser_write_u32"), "C1972: Should contain cmd_ser_write_u32");
    assert!(code.contains("fn cmd_ser_read_u32"), "C1972: Should contain cmd_ser_read_u32");
}

/// C1973: Deserialize result from byte buffer
#[test]
fn c1973_deserialize_result() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;
typedef unsigned char uint8_t;

#define CMD_DESER_MAX 64

typedef struct {
    uint8_t buf[CMD_DESER_MAX];
    int len;
    int cursor;
    int error;
} cmd_deser_t;

void cmd_deser_init(cmd_deser_t *d, const uint8_t *data, int len) {
    int i;
    int copy_len = len < CMD_DESER_MAX ? len : CMD_DESER_MAX;
    for (i = 0; i < copy_len; i++) {
        d->buf[i] = data[i];
    }
    d->len = copy_len;
    d->cursor = 0;
    d->error = 0;
}

int32_t cmd_deser_read_i32(cmd_deser_t *d) {
    if (d->cursor + 4 > d->len) {
        d->error = 1;
        return 0;
    }
    int32_t val = (int32_t)d->buf[d->cursor]
        | ((int32_t)d->buf[d->cursor + 1] << 8)
        | ((int32_t)d->buf[d->cursor + 2] << 16)
        | ((int32_t)d->buf[d->cursor + 3] << 24);
    d->cursor += 4;
    return val;
}

uint8_t cmd_deser_read_u8(cmd_deser_t *d) {
    if (d->cursor >= d->len) {
        d->error = 1;
        return 0;
    }
    return d->buf[d->cursor++];
}

int cmd_deser_has_data(cmd_deser_t *d) {
    return d->cursor < d->len && !d->error;
}

int cmd_deser_bytes_read(cmd_deser_t *d) {
    return d->cursor;
}

int cmd_deser_test(void) {
    uint8_t data[8];
    data[0] = 0x0A; data[1] = 0x00; data[2] = 0x00; data[3] = 0x00;
    data[4] = 0xFF; data[5] = 0x01; data[6] = 0x00; data[7] = 0x00;
    cmd_deser_t d;
    cmd_deser_init(&d, data, 8);
    int32_t v1 = cmd_deser_read_i32(&d);
    int32_t v2 = cmd_deser_read_i32(&d);
    if (v1 != 10) return -1;
    if (v2 != 511) return -2;
    if (cmd_deser_has_data(&d)) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1973: Deserialize result should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1973: Output should not be empty");
    assert!(code.contains("fn cmd_deser_init"), "C1973: Should contain cmd_deser_init");
    assert!(code.contains("fn cmd_deser_read_i32"), "C1973: Should contain cmd_deser_read_i32");
    assert!(code.contains("fn cmd_deser_has_data"), "C1973: Should contain cmd_deser_has_data");
}

/// C1974: Error code registry with message mapping
#[test]
fn c1974_error_codes() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;

#define CMD_ERRCODE_MAX 32
#define CMD_ERRCODE_MSG_LEN 48

typedef struct {
    int32_t code;
    char message[CMD_ERRCODE_MSG_LEN];
    int severity;
    int registered;
} cmd_errcode_entry_t;

typedef struct {
    cmd_errcode_entry_t codes[CMD_ERRCODE_MAX];
    int count;
    int32_t last_error;
} cmd_errcode_registry_t;

void cmd_errcode_init(cmd_errcode_registry_t *r) {
    r->count = 0;
    r->last_error = 0;
    int i;
    for (i = 0; i < CMD_ERRCODE_MAX; i++) {
        r->codes[i].registered = 0;
    }
}

static void cmd_errcode_copy_str(char *dst, const char *src, int max) {
    int i;
    for (i = 0; src[i] && i < max - 1; i++) {
        dst[i] = src[i];
    }
    dst[i] = '\0';
}

int cmd_errcode_register(cmd_errcode_registry_t *r, int32_t code, const char *msg, int severity) {
    if (r->count >= CMD_ERRCODE_MAX) return -1;
    r->codes[r->count].code = code;
    cmd_errcode_copy_str(r->codes[r->count].message, msg, CMD_ERRCODE_MSG_LEN);
    r->codes[r->count].severity = severity;
    r->codes[r->count].registered = 1;
    r->count++;
    return 0;
}

int cmd_errcode_lookup(cmd_errcode_registry_t *r, int32_t code) {
    int i;
    for (i = 0; i < r->count; i++) {
        if (r->codes[i].registered && r->codes[i].code == code) {
            return i;
        }
    }
    return -1;
}

int cmd_errcode_severity(cmd_errcode_registry_t *r, int32_t code) {
    int idx = cmd_errcode_lookup(r, code);
    if (idx < 0) return -1;
    return r->codes[idx].severity;
}

void cmd_errcode_set_last(cmd_errcode_registry_t *r, int32_t code) {
    r->last_error = code;
}

int cmd_errcode_test(void) {
    cmd_errcode_registry_t reg;
    cmd_errcode_init(&reg);
    cmd_errcode_register(&reg, 100, "not found", 1);
    cmd_errcode_register(&reg, 200, "timeout", 2);
    cmd_errcode_register(&reg, 300, "fatal", 3);
    if (cmd_errcode_lookup(&reg, 200) < 0) return -1;
    if (cmd_errcode_severity(&reg, 300) != 3) return -2;
    cmd_errcode_set_last(&reg, 100);
    if (reg.last_error != 100) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1974: Error codes should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1974: Output should not be empty");
    assert!(code.contains("fn cmd_errcode_init"), "C1974: Should contain cmd_errcode_init");
    assert!(code.contains("fn cmd_errcode_register"), "C1974: Should contain cmd_errcode_register");
    assert!(code.contains("fn cmd_errcode_lookup"), "C1974: Should contain cmd_errcode_lookup");
}

/// C1975: RPC request/response envelope with sequence numbers
#[test]
fn c1975_rpc_envelope() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef int int32_t;

#define CMD_RPC_MAX_PENDING 16
#define CMD_RPC_STATUS_PENDING 0
#define CMD_RPC_STATUS_OK 1
#define CMD_RPC_STATUS_ERR 2

typedef struct {
    uint32_t seq_num;
    uint32_t method_id;
    int32_t payload;
    int status;
    int32_t result;
} cmd_rpc_request_t;

typedef struct {
    cmd_rpc_request_t pending[CMD_RPC_MAX_PENDING];
    int count;
    uint32_t next_seq;
    int completed;
    int errors;
} cmd_rpc_context_t;

void cmd_rpc_init(cmd_rpc_context_t *ctx) {
    ctx->count = 0;
    ctx->next_seq = 1;
    ctx->completed = 0;
    ctx->errors = 0;
}

int cmd_rpc_send(cmd_rpc_context_t *ctx, uint32_t method_id, int32_t payload) {
    if (ctx->count >= CMD_RPC_MAX_PENDING) return -1;
    int idx = ctx->count;
    ctx->pending[idx].seq_num = ctx->next_seq++;
    ctx->pending[idx].method_id = method_id;
    ctx->pending[idx].payload = payload;
    ctx->pending[idx].status = CMD_RPC_STATUS_PENDING;
    ctx->pending[idx].result = 0;
    ctx->count++;
    return (int)ctx->pending[idx].seq_num;
}

int cmd_rpc_complete(cmd_rpc_context_t *ctx, uint32_t seq, int32_t result, int success) {
    int i;
    for (i = 0; i < ctx->count; i++) {
        if (ctx->pending[i].seq_num == seq && ctx->pending[i].status == CMD_RPC_STATUS_PENDING) {
            ctx->pending[i].result = result;
            ctx->pending[i].status = success ? CMD_RPC_STATUS_OK : CMD_RPC_STATUS_ERR;
            ctx->completed++;
            if (!success) ctx->errors++;
            return 0;
        }
    }
    return -1;
}

int cmd_rpc_pending_count(cmd_rpc_context_t *ctx) {
    int n = 0;
    int i;
    for (i = 0; i < ctx->count; i++) {
        if (ctx->pending[i].status == CMD_RPC_STATUS_PENDING) n++;
    }
    return n;
}

int32_t cmd_rpc_get_result(cmd_rpc_context_t *ctx, uint32_t seq) {
    int i;
    for (i = 0; i < ctx->count; i++) {
        if (ctx->pending[i].seq_num == seq) {
            return ctx->pending[i].result;
        }
    }
    return -1;
}

int cmd_rpc_test(void) {
    cmd_rpc_context_t ctx;
    cmd_rpc_init(&ctx);
    int s1 = cmd_rpc_send(&ctx, 10, 42);
    int s2 = cmd_rpc_send(&ctx, 20, 99);
    if (cmd_rpc_pending_count(&ctx) != 2) return -1;
    cmd_rpc_complete(&ctx, (uint32_t)s1, 100, 1);
    if (cmd_rpc_pending_count(&ctx) != 1) return -2;
    if (cmd_rpc_get_result(&ctx, (uint32_t)s1) != 100) return -3;
    cmd_rpc_complete(&ctx, (uint32_t)s2, -1, 0);
    if (ctx.errors != 1) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1975: RPC envelope should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1975: Output should not be empty");
    assert!(code.contains("fn cmd_rpc_init"), "C1975: Should contain cmd_rpc_init");
    assert!(code.contains("fn cmd_rpc_send"), "C1975: Should contain cmd_rpc_send");
    assert!(code.contains("fn cmd_rpc_complete"), "C1975: Should contain cmd_rpc_complete");
}
