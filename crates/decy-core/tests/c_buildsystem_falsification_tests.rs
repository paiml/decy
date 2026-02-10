//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1576-C1600: Build System / Dependency Management -- dependency graphs,
//! file tracking, task scheduling, package management, and build caches.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world build system patterns commonly
//! found in Make, Bazel, Cargo, CMake, and similar build tools -- all
//! expressed as valid C99.
//!
//! Organization:
//! - C1576-C1580: Dependency graph (node/edge, topological sort, cycle detection, transitive closure, incremental update)
//! - C1581-C1585: File tracking (timestamp comparison, content hash, dirty checking, glob expansion, file watcher state)
//! - C1586-C1590: Task scheduling (task queue, parallel execution, work stealing, task priority, resource semaphore)
//! - C1591-C1595: Package management (version comparison, dependency resolution, conflict detection, lock file entry, package registry)
//! - C1596-C1600: Build cache (cache key computation, LRU eviction, content addressable store, artifact manifest, incremental compilation unit)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C1576-C1580: Dependency Graph
// ============================================================================

/// C1576: Dependency graph with node/edge management
#[test]
fn c1576_depgraph_node_edge() {
    let c_code = r#"
#define BLD_MAX_NODES 64
#define BLD_MAX_EDGES 256

typedef struct {
    int from;
    int to;
} bld_edge_t;

typedef struct {
    int node_ids[BLD_MAX_NODES];
    int node_count;
    bld_edge_t edges[BLD_MAX_EDGES];
    int edge_count;
} bld_depgraph_t;

void bld_depgraph_init(bld_depgraph_t *g) {
    g->node_count = 0;
    g->edge_count = 0;
}

int bld_depgraph_add_node(bld_depgraph_t *g, int id) {
    if (g->node_count >= BLD_MAX_NODES) return -1;
    g->node_ids[g->node_count] = id;
    g->node_count++;
    return 0;
}

int bld_depgraph_add_edge(bld_depgraph_t *g, int from, int to) {
    if (g->edge_count >= BLD_MAX_EDGES) return -1;
    g->edges[g->edge_count].from = from;
    g->edges[g->edge_count].to = to;
    g->edge_count++;
    return 0;
}

int bld_depgraph_has_edge(bld_depgraph_t *g, int from, int to) {
    int i;
    for (i = 0; i < g->edge_count; i++) {
        if (g->edges[i].from == from && g->edges[i].to == to) return 1;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1576: depgraph node/edge should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_depgraph_add_node"));
    assert!(code.contains("fn bld_depgraph_has_edge"));
}

/// C1577: Topological sort for build order
#[test]
fn c1577_topological_sort() {
    let c_code = r#"
#define BLD_TOPO_MAX 32

typedef struct {
    int adj[BLD_TOPO_MAX][BLD_TOPO_MAX];
    int in_degree[BLD_TOPO_MAX];
    int n;
} bld_topo_t;

void bld_topo_init(bld_topo_t *t, int n) {
    int i, j;
    t->n = n;
    for (i = 0; i < BLD_TOPO_MAX; i++) {
        t->in_degree[i] = 0;
        for (j = 0; j < BLD_TOPO_MAX; j++)
            t->adj[i][j] = 0;
    }
}

void bld_topo_add_dep(bld_topo_t *t, int from, int to) {
    if (!t->adj[from][to]) {
        t->adj[from][to] = 1;
        t->in_degree[to]++;
    }
}

int bld_topo_sort(bld_topo_t *t, int *order) {
    int queue[BLD_TOPO_MAX];
    int deg[BLD_TOPO_MAX];
    int front = 0, back = 0, count = 0;
    int i, j;
    for (i = 0; i < t->n; i++) deg[i] = t->in_degree[i];
    for (i = 0; i < t->n; i++) {
        if (deg[i] == 0) queue[back++] = i;
    }
    while (front < back) {
        int u = queue[front++];
        order[count++] = u;
        for (j = 0; j < t->n; j++) {
            if (t->adj[u][j]) {
                deg[j]--;
                if (deg[j] == 0) queue[back++] = j;
            }
        }
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1577: topological sort should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_topo_sort"));
}

/// C1578: Cycle detection in dependency graph via DFS coloring
#[test]
fn c1578_cycle_detection() {
    let c_code = r#"
#define BLD_CYC_MAX 32

typedef struct {
    int adj[BLD_CYC_MAX][BLD_CYC_MAX];
    int n;
} bld_cyc_graph_t;

void bld_cyc_init(bld_cyc_graph_t *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < BLD_CYC_MAX; i++)
        for (j = 0; j < BLD_CYC_MAX; j++)
            g->adj[i][j] = 0;
}

void bld_cyc_add_edge(bld_cyc_graph_t *g, int u, int v) {
    g->adj[u][v] = 1;
}

static int bld_cyc_dfs(bld_cyc_graph_t *g, int u, int *color) {
    int v;
    color[u] = 1;
    for (v = 0; v < g->n; v++) {
        if (g->adj[u][v]) {
            if (color[v] == 1) return 1;
            if (color[v] == 0 && bld_cyc_dfs(g, v, color)) return 1;
        }
    }
    color[u] = 2;
    return 0;
}

int bld_cyc_has_cycle(bld_cyc_graph_t *g) {
    int color[BLD_CYC_MAX];
    int i;
    for (i = 0; i < g->n; i++) color[i] = 0;
    for (i = 0; i < g->n; i++) {
        if (color[i] == 0 && bld_cyc_dfs(g, i, color)) return 1;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1578: cycle detection should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_cyc_has_cycle"));
}

/// C1579: Transitive closure of dependency graph (Warshall)
#[test]
fn c1579_transitive_closure() {
    let c_code = r#"
#define BLD_TC_MAX 16

typedef struct {
    int reach[BLD_TC_MAX][BLD_TC_MAX];
    int n;
} bld_tc_t;

void bld_tc_init(bld_tc_t *tc, int n) {
    int i, j;
    tc->n = n;
    for (i = 0; i < BLD_TC_MAX; i++)
        for (j = 0; j < BLD_TC_MAX; j++)
            tc->reach[i][j] = 0;
}

void bld_tc_add_edge(bld_tc_t *tc, int u, int v) {
    tc->reach[u][v] = 1;
}

void bld_tc_compute(bld_tc_t *tc) {
    int i, j, k;
    for (k = 0; k < tc->n; k++)
        for (i = 0; i < tc->n; i++)
            for (j = 0; j < tc->n; j++)
                if (tc->reach[i][k] && tc->reach[k][j])
                    tc->reach[i][j] = 1;
}

int bld_tc_can_reach(bld_tc_t *tc, int u, int v) {
    return tc->reach[u][v];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1579: transitive closure should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_tc_compute"));
    assert!(code.contains("fn bld_tc_can_reach"));
}

/// C1580: Incremental dependency update (invalidation propagation)
#[test]
fn c1580_incremental_update() {
    let c_code = r#"
#define BLD_INC_MAX 32

typedef struct {
    int deps[BLD_INC_MAX][BLD_INC_MAX];
    int dirty[BLD_INC_MAX];
    int n;
} bld_inc_t;

void bld_inc_init(bld_inc_t *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < BLD_INC_MAX; i++) {
        g->dirty[i] = 0;
        for (j = 0; j < BLD_INC_MAX; j++)
            g->deps[i][j] = 0;
    }
}

void bld_inc_add_dep(bld_inc_t *g, int target, int dep) {
    g->deps[target][dep] = 1;
}

void bld_inc_mark_dirty(bld_inc_t *g, int node) {
    int i;
    if (g->dirty[node]) return;
    g->dirty[node] = 1;
    for (i = 0; i < g->n; i++) {
        if (g->deps[i][node])
            bld_inc_mark_dirty(g, i);
    }
}

int bld_inc_count_dirty(bld_inc_t *g) {
    int i, count = 0;
    for (i = 0; i < g->n; i++)
        if (g->dirty[i]) count++;
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1580: incremental update should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_inc_mark_dirty"));
    assert!(code.contains("fn bld_inc_count_dirty"));
}

// ============================================================================
// C1581-C1585: File Tracking
// ============================================================================

/// C1581: Timestamp comparison for file freshness
#[test]
fn c1581_timestamp_compare() {
    let c_code = r#"
typedef long bld_time_t;

typedef struct {
    int id;
    bld_time_t mtime;
} bld_fentry_t;

int bld_ts_is_newer(bld_fentry_t *a, bld_fentry_t *b) {
    return a->mtime > b->mtime;
}

int bld_ts_needs_rebuild(bld_fentry_t *target, bld_fentry_t *deps, int ndeps) {
    int i;
    for (i = 0; i < ndeps; i++) {
        if (deps[i].mtime > target->mtime) return 1;
    }
    return 0;
}

bld_time_t bld_ts_newest(bld_fentry_t *files, int n) {
    bld_time_t best = 0;
    int i;
    for (i = 0; i < n; i++) {
        if (files[i].mtime > best) best = files[i].mtime;
    }
    return best;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1581: timestamp compare should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_ts_is_newer"));
    assert!(code.contains("fn bld_ts_needs_rebuild"));
}

/// C1582: Content hash for change detection
#[test]
fn c1582_content_hash() {
    let c_code = r#"
typedef unsigned long bld_hash_t;

bld_hash_t bld_hash_bytes(const char *data, int len) {
    bld_hash_t h = 5381;
    int i;
    for (i = 0; i < len; i++) {
        h = ((h << 5) + h) + (unsigned char)data[i];
    }
    return h;
}

int bld_hash_changed(bld_hash_t old_hash, const char *data, int len) {
    return bld_hash_bytes(data, len) != old_hash;
}

bld_hash_t bld_hash_combine(bld_hash_t a, bld_hash_t b) {
    return a ^ (b + 0x9e3779b9 + (a << 6) + (a >> 2));
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1582: content hash should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_hash_bytes"));
    assert!(code.contains("fn bld_hash_combine"));
}

/// C1583: Dirty checking for build targets
#[test]
fn c1583_dirty_checking() {
    let c_code = r#"
typedef unsigned long bld_hash_t;

#define BLD_DIRTY_MAX 64

typedef struct {
    int id;
    bld_hash_t content_hash;
    bld_hash_t prev_hash;
    int dirty;
} bld_dirty_entry_t;

typedef struct {
    bld_dirty_entry_t entries[BLD_DIRTY_MAX];
    int count;
} bld_dirty_tracker_t;

void bld_dirty_init(bld_dirty_tracker_t *t) {
    t->count = 0;
}

int bld_dirty_add(bld_dirty_tracker_t *t, int id, bld_hash_t hash) {
    if (t->count >= BLD_DIRTY_MAX) return -1;
    t->entries[t->count].id = id;
    t->entries[t->count].content_hash = hash;
    t->entries[t->count].prev_hash = hash;
    t->entries[t->count].dirty = 0;
    t->count++;
    return 0;
}

void bld_dirty_update(bld_dirty_tracker_t *t, int id, bld_hash_t new_hash) {
    int i;
    for (i = 0; i < t->count; i++) {
        if (t->entries[i].id == id) {
            t->entries[i].prev_hash = t->entries[i].content_hash;
            t->entries[i].content_hash = new_hash;
            t->entries[i].dirty = (new_hash != t->entries[i].prev_hash);
            return;
        }
    }
}

int bld_dirty_count(bld_dirty_tracker_t *t) {
    int i, c = 0;
    for (i = 0; i < t->count; i++)
        if (t->entries[i].dirty) c++;
    return c;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1583: dirty checking should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_dirty_update"));
    assert!(code.contains("fn bld_dirty_count"));
}

/// C1584: Glob expansion for file pattern matching
#[test]
fn c1584_glob_expansion() {
    let c_code = r#"
#define BLD_GLOB_MAX 128

typedef struct {
    char pattern[64];
    int matches[BLD_GLOB_MAX];
    int match_count;
} bld_glob_t;

void bld_glob_init(bld_glob_t *g, const char *pat) {
    int i;
    for (i = 0; pat[i] && i < 63; i++)
        g->pattern[i] = pat[i];
    g->pattern[i] = '\0';
    g->match_count = 0;
}

static int bld_glob_char_match(char p, char c) {
    if (p == '?') return 1;
    return p == c;
}

int bld_glob_simple_match(const char *pattern, const char *name) {
    while (*pattern && *name) {
        if (*pattern == '*') {
            pattern++;
            while (*name) {
                if (bld_glob_simple_match(pattern, name)) return 1;
                name++;
            }
            return *pattern == '\0';
        }
        if (!bld_glob_char_match(*pattern, *name)) return 0;
        pattern++;
        name++;
    }
    while (*pattern == '*') pattern++;
    return *pattern == '\0' && *name == '\0';
}

void bld_glob_add_match(bld_glob_t *g, int file_id) {
    if (g->match_count < BLD_GLOB_MAX)
        g->matches[g->match_count++] = file_id;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1584: glob expansion should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_glob_simple_match"));
}

/// C1585: File watcher state machine
#[test]
fn c1585_file_watcher_state() {
    let c_code = r#"
typedef unsigned long bld_hash_t;

#define BLD_WATCH_MAX 32
#define BLD_WATCH_IDLE 0
#define BLD_WATCH_MODIFIED 1
#define BLD_WATCH_DELETED 2
#define BLD_WATCH_CREATED 3

typedef struct {
    int file_id;
    int state;
    bld_hash_t last_hash;
} bld_watch_entry_t;

typedef struct {
    bld_watch_entry_t files[BLD_WATCH_MAX];
    int count;
} bld_watcher_t;

void bld_watcher_init(bld_watcher_t *w) {
    w->count = 0;
}

int bld_watcher_add(bld_watcher_t *w, int file_id, bld_hash_t hash) {
    if (w->count >= BLD_WATCH_MAX) return -1;
    w->files[w->count].file_id = file_id;
    w->files[w->count].state = BLD_WATCH_IDLE;
    w->files[w->count].last_hash = hash;
    w->count++;
    return 0;
}

void bld_watcher_check(bld_watcher_t *w, int file_id, bld_hash_t cur_hash, int exists) {
    int i;
    for (i = 0; i < w->count; i++) {
        if (w->files[i].file_id == file_id) {
            if (!exists) { w->files[i].state = BLD_WATCH_DELETED; return; }
            if (cur_hash != w->files[i].last_hash) {
                w->files[i].state = BLD_WATCH_MODIFIED;
                w->files[i].last_hash = cur_hash;
            } else {
                w->files[i].state = BLD_WATCH_IDLE;
            }
            return;
        }
    }
}

int bld_watcher_any_changed(bld_watcher_t *w) {
    int i;
    for (i = 0; i < w->count; i++)
        if (w->files[i].state != BLD_WATCH_IDLE) return 1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1585: file watcher state should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_watcher_check"));
    assert!(code.contains("fn bld_watcher_any_changed"));
}

// ============================================================================
// C1586-C1590: Task Scheduling
// ============================================================================

/// C1586: Task queue with FIFO scheduling
#[test]
fn c1586_task_queue() {
    let c_code = r#"
#define BLD_TQ_MAX 64
#define BLD_TASK_PENDING 0
#define BLD_TASK_RUNNING 1
#define BLD_TASK_DONE 2

typedef struct {
    int task_id;
    int status;
    int deps_remaining;
} bld_task_t;

typedef struct {
    bld_task_t tasks[BLD_TQ_MAX];
    int head;
    int tail;
    int count;
} bld_taskqueue_t;

void bld_tq_init(bld_taskqueue_t *q) {
    q->head = 0;
    q->tail = 0;
    q->count = 0;
}

int bld_tq_push(bld_taskqueue_t *q, int id, int deps) {
    if (q->count >= BLD_TQ_MAX) return -1;
    q->tasks[q->tail].task_id = id;
    q->tasks[q->tail].status = BLD_TASK_PENDING;
    q->tasks[q->tail].deps_remaining = deps;
    q->tail = (q->tail + 1) % BLD_TQ_MAX;
    q->count++;
    return 0;
}

int bld_tq_pop_ready(bld_taskqueue_t *q) {
    int i, idx;
    for (i = 0; i < q->count; i++) {
        idx = (q->head + i) % BLD_TQ_MAX;
        if (q->tasks[idx].status == BLD_TASK_PENDING && q->tasks[idx].deps_remaining == 0) {
            q->tasks[idx].status = BLD_TASK_RUNNING;
            return q->tasks[idx].task_id;
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1586: task queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_tq_push"));
    assert!(code.contains("fn bld_tq_pop_ready"));
}

/// C1587: Parallel execution tracker with worker slots
#[test]
fn c1587_parallel_execution() {
    let c_code = r#"
#define BLD_PAR_WORKERS 8

typedef struct {
    int worker_task[BLD_PAR_WORKERS];
    int worker_busy[BLD_PAR_WORKERS];
    int num_workers;
    int tasks_completed;
} bld_par_exec_t;

void bld_par_init(bld_par_exec_t *e, int nworkers) {
    int i;
    e->num_workers = nworkers > BLD_PAR_WORKERS ? BLD_PAR_WORKERS : nworkers;
    e->tasks_completed = 0;
    for (i = 0; i < BLD_PAR_WORKERS; i++) {
        e->worker_task[i] = -1;
        e->worker_busy[i] = 0;
    }
}

int bld_par_assign(bld_par_exec_t *e, int task_id) {
    int i;
    for (i = 0; i < e->num_workers; i++) {
        if (!e->worker_busy[i]) {
            e->worker_task[i] = task_id;
            e->worker_busy[i] = 1;
            return i;
        }
    }
    return -1;
}

void bld_par_complete(bld_par_exec_t *e, int worker_id) {
    if (worker_id >= 0 && worker_id < e->num_workers) {
        e->worker_busy[worker_id] = 0;
        e->worker_task[worker_id] = -1;
        e->tasks_completed++;
    }
}

int bld_par_active_count(bld_par_exec_t *e) {
    int i, c = 0;
    for (i = 0; i < e->num_workers; i++)
        if (e->worker_busy[i]) c++;
    return c;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1587: parallel execution should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_par_assign"));
    assert!(code.contains("fn bld_par_active_count"));
}

/// C1588: Work stealing deque for build tasks
#[test]
fn c1588_work_stealing() {
    let c_code = r#"
#define BLD_WS_CAP 64

typedef struct {
    int items[BLD_WS_CAP];
    int top;
    int bottom;
} bld_ws_deque_t;

void bld_ws_init(bld_ws_deque_t *d) {
    d->top = 0;
    d->bottom = 0;
}

int bld_ws_push(bld_ws_deque_t *d, int task) {
    int b = d->bottom;
    if (b - d->top >= BLD_WS_CAP) return -1;
    d->items[b % BLD_WS_CAP] = task;
    d->bottom = b + 1;
    return 0;
}

int bld_ws_pop(bld_ws_deque_t *d) {
    int b = d->bottom - 1;
    d->bottom = b;
    if (d->top > b) {
        d->bottom = d->top;
        return -1;
    }
    return d->items[b % BLD_WS_CAP];
}

int bld_ws_steal(bld_ws_deque_t *d) {
    int t = d->top;
    if (t >= d->bottom) return -1;
    int val = d->items[t % BLD_WS_CAP];
    d->top = t + 1;
    return val;
}

int bld_ws_size(bld_ws_deque_t *d) {
    int s = d->bottom - d->top;
    return s > 0 ? s : 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1588: work stealing should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_ws_steal"));
    assert!(code.contains("fn bld_ws_size"));
}

/// C1589: Task priority queue (min-heap by priority)
#[test]
fn c1589_task_priority() {
    let c_code = r#"
#define BLD_PQ_MAX 64

typedef struct {
    int task_id;
    int priority;
} bld_pq_entry_t;

typedef struct {
    bld_pq_entry_t heap[BLD_PQ_MAX];
    int size;
} bld_pq_t;

void bld_pq_init(bld_pq_t *pq) {
    pq->size = 0;
}

static void bld_pq_swap(bld_pq_entry_t *a, bld_pq_entry_t *b) {
    bld_pq_entry_t tmp = *a;
    *a = *b;
    *b = tmp;
}

int bld_pq_insert(bld_pq_t *pq, int task_id, int pri) {
    if (pq->size >= BLD_PQ_MAX) return -1;
    int i = pq->size;
    pq->heap[i].task_id = task_id;
    pq->heap[i].priority = pri;
    pq->size++;
    while (i > 0) {
        int p = (i - 1) / 2;
        if (pq->heap[p].priority > pq->heap[i].priority) {
            bld_pq_swap(&pq->heap[p], &pq->heap[i]);
            i = p;
        } else break;
    }
    return 0;
}

int bld_pq_pop(bld_pq_t *pq) {
    if (pq->size == 0) return -1;
    int id = pq->heap[0].task_id;
    pq->size--;
    pq->heap[0] = pq->heap[pq->size];
    int i = 0;
    while (1) {
        int l = 2 * i + 1, r = 2 * i + 2, m = i;
        if (l < pq->size && pq->heap[l].priority < pq->heap[m].priority) m = l;
        if (r < pq->size && pq->heap[r].priority < pq->heap[m].priority) m = r;
        if (m == i) break;
        bld_pq_swap(&pq->heap[i], &pq->heap[m]);
        i = m;
    }
    return id;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1589: task priority should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_pq_insert"));
    assert!(code.contains("fn bld_pq_pop"));
}

/// C1590: Resource semaphore for limiting concurrent builds
#[test]
fn c1590_resource_semaphore() {
    let c_code = r#"
#define BLD_SEM_MAX_WAITERS 32

typedef struct {
    int value;
    int max_value;
    int waiters[BLD_SEM_MAX_WAITERS];
    int waiter_count;
} bld_sem_t;

void bld_sem_init(bld_sem_t *s, int max_val) {
    s->value = max_val;
    s->max_value = max_val;
    s->waiter_count = 0;
}

int bld_sem_try_acquire(bld_sem_t *s, int task_id) {
    if (s->value > 0) {
        s->value--;
        return 1;
    }
    if (s->waiter_count < BLD_SEM_MAX_WAITERS) {
        s->waiters[s->waiter_count++] = task_id;
    }
    return 0;
}

int bld_sem_release(bld_sem_t *s) {
    if (s->value < s->max_value) {
        s->value++;
    }
    if (s->waiter_count > 0) {
        int next = s->waiters[0];
        int i;
        for (i = 1; i < s->waiter_count; i++)
            s->waiters[i - 1] = s->waiters[i];
        s->waiter_count--;
        return next;
    }
    return -1;
}

int bld_sem_available(bld_sem_t *s) {
    return s->value;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1590: resource semaphore should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_sem_try_acquire"));
    assert!(code.contains("fn bld_sem_release"));
}

// ============================================================================
// C1591-C1595: Package Management
// ============================================================================

/// C1591: Semantic version comparison
#[test]
fn c1591_semver_compare() {
    let c_code = r#"
typedef struct {
    int major;
    int minor;
    int patch;
} bld_semver_t;

void bld_semver_set(bld_semver_t *v, int maj, int min, int pat) {
    v->major = maj;
    v->minor = min;
    v->patch = pat;
}

int bld_semver_cmp(bld_semver_t *a, bld_semver_t *b) {
    if (a->major != b->major) return a->major - b->major;
    if (a->minor != b->minor) return a->minor - b->minor;
    return a->patch - b->patch;
}

int bld_semver_compatible(bld_semver_t *req, bld_semver_t *have) {
    if (have->major != req->major) return 0;
    if (have->minor < req->minor) return 0;
    if (have->minor == req->minor && have->patch < req->patch) return 0;
    return 1;
}

int bld_semver_is_prerelease(bld_semver_t *v) {
    return v->major == 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1591: semver compare should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_semver_cmp"));
    assert!(code.contains("fn bld_semver_compatible"));
}

/// C1592: Dependency resolution with version constraints
#[test]
fn c1592_dependency_resolution() {
    let c_code = r#"
#define BLD_DEP_MAX 32

typedef struct {
    int pkg_id;
    int min_major;
    int min_minor;
    int resolved_version;
} bld_dep_constraint_t;

typedef struct {
    bld_dep_constraint_t deps[BLD_DEP_MAX];
    int count;
} bld_dep_resolver_t;

void bld_dep_init(bld_dep_resolver_t *r) {
    r->count = 0;
}

int bld_dep_add(bld_dep_resolver_t *r, int pkg, int min_maj, int min_min) {
    if (r->count >= BLD_DEP_MAX) return -1;
    r->deps[r->count].pkg_id = pkg;
    r->deps[r->count].min_major = min_maj;
    r->deps[r->count].min_minor = min_min;
    r->deps[r->count].resolved_version = -1;
    r->count++;
    return 0;
}

int bld_dep_resolve(bld_dep_resolver_t *r, int pkg, int avail_maj, int avail_min) {
    int i;
    for (i = 0; i < r->count; i++) {
        if (r->deps[i].pkg_id == pkg) {
            if (avail_maj > r->deps[i].min_major ||
                (avail_maj == r->deps[i].min_major && avail_min >= r->deps[i].min_minor)) {
                r->deps[i].resolved_version = avail_maj * 1000 + avail_min;
                return 1;
            }
            return 0;
        }
    }
    return -1;
}

int bld_dep_all_resolved(bld_dep_resolver_t *r) {
    int i;
    for (i = 0; i < r->count; i++)
        if (r->deps[i].resolved_version < 0) return 0;
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1592: dependency resolution should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_dep_resolve"));
    assert!(code.contains("fn bld_dep_all_resolved"));
}

/// C1593: Dependency conflict detection
#[test]
fn c1593_conflict_detection() {
    let c_code = r#"
#define BLD_CONF_MAX 32

typedef struct {
    int pkg_id;
    int version_a;
    int version_b;
} bld_conflict_t;

typedef struct {
    int pkg_versions[BLD_CONF_MAX][2];
    int pkg_count;
    bld_conflict_t conflicts[BLD_CONF_MAX];
    int conflict_count;
} bld_conflict_checker_t;

void bld_conf_init(bld_conflict_checker_t *c) {
    c->pkg_count = 0;
    c->conflict_count = 0;
}

void bld_conf_require(bld_conflict_checker_t *c, int pkg, int version) {
    int i;
    for (i = 0; i < c->pkg_count; i++) {
        if (c->pkg_versions[i][0] == pkg) {
            if (c->pkg_versions[i][1] != version && c->conflict_count < BLD_CONF_MAX) {
                c->conflicts[c->conflict_count].pkg_id = pkg;
                c->conflicts[c->conflict_count].version_a = c->pkg_versions[i][1];
                c->conflicts[c->conflict_count].version_b = version;
                c->conflict_count++;
            }
            return;
        }
    }
    if (c->pkg_count < BLD_CONF_MAX) {
        c->pkg_versions[c->pkg_count][0] = pkg;
        c->pkg_versions[c->pkg_count][1] = version;
        c->pkg_count++;
    }
}

int bld_conf_has_conflicts(bld_conflict_checker_t *c) {
    return c->conflict_count > 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1593: conflict detection should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_conf_require"));
    assert!(code.contains("fn bld_conf_has_conflicts"));
}

/// C1594: Lock file entry management
#[test]
fn c1594_lock_file_entry() {
    let c_code = r#"
typedef unsigned long bld_hash_t;

#define BLD_LOCK_MAX 64

typedef struct {
    int pkg_id;
    int version;
    bld_hash_t integrity;
    int pinned;
} bld_lock_entry_t;

typedef struct {
    bld_lock_entry_t entries[BLD_LOCK_MAX];
    int count;
} bld_lockfile_t;

void bld_lock_init(bld_lockfile_t *lf) {
    lf->count = 0;
}

int bld_lock_add(bld_lockfile_t *lf, int pkg, int ver, bld_hash_t hash) {
    if (lf->count >= BLD_LOCK_MAX) return -1;
    lf->entries[lf->count].pkg_id = pkg;
    lf->entries[lf->count].version = ver;
    lf->entries[lf->count].integrity = hash;
    lf->entries[lf->count].pinned = 0;
    lf->count++;
    return 0;
}

int bld_lock_verify(bld_lockfile_t *lf, int pkg, bld_hash_t hash) {
    int i;
    for (i = 0; i < lf->count; i++) {
        if (lf->entries[i].pkg_id == pkg) {
            return lf->entries[i].integrity == hash;
        }
    }
    return -1;
}

void bld_lock_pin(bld_lockfile_t *lf, int pkg) {
    int i;
    for (i = 0; i < lf->count; i++) {
        if (lf->entries[i].pkg_id == pkg) {
            lf->entries[i].pinned = 1;
            return;
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1594: lock file entry should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_lock_add"));
    assert!(code.contains("fn bld_lock_verify"));
}

/// C1595: Package registry with search
#[test]
fn c1595_package_registry() {
    let c_code = r#"
#define BLD_REG_MAX 64
#define BLD_REG_NAME_LEN 32

typedef struct {
    char name[BLD_REG_NAME_LEN];
    int latest_version;
    int download_count;
    int deprecated;
} bld_pkg_info_t;

typedef struct {
    bld_pkg_info_t packages[BLD_REG_MAX];
    int count;
} bld_registry_t;

void bld_reg_init(bld_registry_t *r) {
    r->count = 0;
}

int bld_reg_publish(bld_registry_t *r, const char *name, int version) {
    int i;
    if (r->count >= BLD_REG_MAX) return -1;
    for (i = 0; name[i] && i < BLD_REG_NAME_LEN - 1; i++)
        r->packages[r->count].name[i] = name[i];
    r->packages[r->count].name[i] = '\0';
    r->packages[r->count].latest_version = version;
    r->packages[r->count].download_count = 0;
    r->packages[r->count].deprecated = 0;
    r->count++;
    return 0;
}

static int bld_reg_streq(const char *a, const char *b) {
    while (*a && *b) {
        if (*a != *b) return 0;
        a++; b++;
    }
    return *a == *b;
}

int bld_reg_find(bld_registry_t *r, const char *name) {
    int i;
    for (i = 0; i < r->count; i++) {
        if (bld_reg_streq(r->packages[i].name, name))
            return i;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1595: package registry should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_reg_publish"));
    assert!(code.contains("fn bld_reg_find"));
}

// ============================================================================
// C1596-C1600: Build Cache
// ============================================================================

/// C1596: Cache key computation from input hashes
#[test]
fn c1596_cache_key_computation() {
    let c_code = r#"
typedef unsigned long bld_hash_t;

#define BLD_CK_MAX_INPUTS 16

typedef struct {
    bld_hash_t input_hashes[BLD_CK_MAX_INPUTS];
    int input_count;
    bld_hash_t key;
} bld_cache_key_t;

void bld_ck_init(bld_cache_key_t *ck) {
    ck->input_count = 0;
    ck->key = 0;
}

void bld_ck_add_input(bld_cache_key_t *ck, bld_hash_t h) {
    if (ck->input_count < BLD_CK_MAX_INPUTS)
        ck->input_hashes[ck->input_count++] = h;
}

static bld_hash_t bld_ck_mix(bld_hash_t a, bld_hash_t b) {
    return a ^ (b * 0x517cc1b727220a95UL + 0x6c62272e07bb0142UL);
}

bld_hash_t bld_ck_compute(bld_cache_key_t *ck) {
    bld_hash_t h = 0xcbf29ce484222325UL;
    int i;
    for (i = 0; i < ck->input_count; i++)
        h = bld_ck_mix(h, ck->input_hashes[i]);
    ck->key = h;
    return h;
}

int bld_ck_matches(bld_cache_key_t *a, bld_cache_key_t *b) {
    return a->key == b->key;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1596: cache key computation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_ck_compute"));
    assert!(code.contains("fn bld_ck_matches"));
}

/// C1597: LRU eviction for build cache
#[test]
fn c1597_lru_eviction() {
    let c_code = r#"
typedef unsigned long bld_hash_t;

#define BLD_LRU_CAP 32

typedef struct {
    bld_hash_t keys[BLD_LRU_CAP];
    int access_time[BLD_LRU_CAP];
    int valid[BLD_LRU_CAP];
    int count;
    int clock;
} bld_lru_cache_t;

void bld_lru_init(bld_lru_cache_t *c) {
    int i;
    c->count = 0;
    c->clock = 0;
    for (i = 0; i < BLD_LRU_CAP; i++) c->valid[i] = 0;
}

int bld_lru_find(bld_lru_cache_t *c, bld_hash_t key) {
    int i;
    for (i = 0; i < BLD_LRU_CAP; i++) {
        if (c->valid[i] && c->keys[i] == key) {
            c->access_time[i] = ++c->clock;
            return i;
        }
    }
    return -1;
}

static int bld_lru_find_victim(bld_lru_cache_t *c) {
    int i, oldest = 0;
    int oldest_time = c->access_time[0];
    for (i = 1; i < BLD_LRU_CAP; i++) {
        if (c->valid[i] && c->access_time[i] < oldest_time) {
            oldest = i;
            oldest_time = c->access_time[i];
        }
    }
    return oldest;
}

int bld_lru_insert(bld_lru_cache_t *c, bld_hash_t key) {
    int slot;
    if (c->count < BLD_LRU_CAP) {
        slot = c->count++;
    } else {
        slot = bld_lru_find_victim(c);
    }
    c->keys[slot] = key;
    c->access_time[slot] = ++c->clock;
    c->valid[slot] = 1;
    return slot;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1597: LRU eviction should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_lru_find"));
    assert!(code.contains("fn bld_lru_insert"));
}

/// C1598: Content-addressable store for build artifacts
#[test]
fn c1598_content_addressable_store() {
    let c_code = r#"
typedef unsigned long bld_hash_t;

#define BLD_CAS_BUCKETS 64

typedef struct {
    bld_hash_t hash;
    int size;
    int ref_count;
    int occupied;
} bld_cas_entry_t;

typedef struct {
    bld_cas_entry_t store[BLD_CAS_BUCKETS];
    int total_size;
} bld_cas_t;

void bld_cas_init(bld_cas_t *cas) {
    int i;
    cas->total_size = 0;
    for (i = 0; i < BLD_CAS_BUCKETS; i++)
        cas->store[i].occupied = 0;
}

int bld_cas_put(bld_cas_t *cas, bld_hash_t hash, int size) {
    int idx = (int)(hash % BLD_CAS_BUCKETS);
    int i;
    for (i = 0; i < BLD_CAS_BUCKETS; i++) {
        int slot = (idx + i) % BLD_CAS_BUCKETS;
        if (cas->store[slot].occupied && cas->store[slot].hash == hash) {
            cas->store[slot].ref_count++;
            return slot;
        }
        if (!cas->store[slot].occupied) {
            cas->store[slot].hash = hash;
            cas->store[slot].size = size;
            cas->store[slot].ref_count = 1;
            cas->store[slot].occupied = 1;
            cas->total_size += size;
            return slot;
        }
    }
    return -1;
}

int bld_cas_has(bld_cas_t *cas, bld_hash_t hash) {
    int idx = (int)(hash % BLD_CAS_BUCKETS);
    int i;
    for (i = 0; i < BLD_CAS_BUCKETS; i++) {
        int slot = (idx + i) % BLD_CAS_BUCKETS;
        if (!cas->store[slot].occupied) return 0;
        if (cas->store[slot].hash == hash) return 1;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1598: content addressable store should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_cas_put"));
    assert!(code.contains("fn bld_cas_has"));
}

/// C1599: Artifact manifest tracking build outputs
#[test]
fn c1599_artifact_manifest() {
    let c_code = r#"
typedef unsigned long bld_hash_t;

#define BLD_MAN_MAX 32

typedef struct {
    int artifact_id;
    bld_hash_t content_hash;
    int size_bytes;
    long build_time;
} bld_artifact_t;

typedef struct {
    bld_artifact_t items[BLD_MAN_MAX];
    int count;
    long created_at;
} bld_manifest_t;

void bld_man_init(bld_manifest_t *m, long timestamp) {
    m->count = 0;
    m->created_at = timestamp;
}

int bld_man_add(bld_manifest_t *m, int id, bld_hash_t hash, int size, long btime) {
    if (m->count >= BLD_MAN_MAX) return -1;
    m->items[m->count].artifact_id = id;
    m->items[m->count].content_hash = hash;
    m->items[m->count].size_bytes = size;
    m->items[m->count].build_time = btime;
    m->count++;
    return 0;
}

int bld_man_total_size(bld_manifest_t *m) {
    int i, total = 0;
    for (i = 0; i < m->count; i++)
        total += m->items[i].size_bytes;
    return total;
}

int bld_man_find(bld_manifest_t *m, int artifact_id) {
    int i;
    for (i = 0; i < m->count; i++)
        if (m->items[i].artifact_id == artifact_id) return i;
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1599: artifact manifest should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_man_add"));
    assert!(code.contains("fn bld_man_find"));
}

/// C1600: Incremental compilation unit tracking
#[test]
fn c1600_incremental_compilation_unit() {
    let c_code = r#"
typedef unsigned long bld_hash_t;

#define BLD_ICU_MAX 64

typedef struct {
    int unit_id;
    bld_hash_t source_hash;
    bld_hash_t object_hash;
    int needs_recompile;
    int dep_units[8];
    int dep_count;
} bld_comp_unit_t;

typedef struct {
    bld_comp_unit_t units[BLD_ICU_MAX];
    int count;
} bld_icu_tracker_t;

void bld_icu_init(bld_icu_tracker_t *t) {
    t->count = 0;
}

int bld_icu_add(bld_icu_tracker_t *t, int id, bld_hash_t src_hash) {
    if (t->count >= BLD_ICU_MAX) return -1;
    t->units[t->count].unit_id = id;
    t->units[t->count].source_hash = src_hash;
    t->units[t->count].object_hash = 0;
    t->units[t->count].needs_recompile = 1;
    t->units[t->count].dep_count = 0;
    t->count++;
    return 0;
}

void bld_icu_mark_compiled(bld_icu_tracker_t *t, int id, bld_hash_t obj_hash) {
    int i;
    for (i = 0; i < t->count; i++) {
        if (t->units[i].unit_id == id) {
            t->units[i].object_hash = obj_hash;
            t->units[i].needs_recompile = 0;
            return;
        }
    }
}

void bld_icu_invalidate(bld_icu_tracker_t *t, int id, bld_hash_t new_hash) {
    int i, j, k;
    for (i = 0; i < t->count; i++) {
        if (t->units[i].unit_id == id) {
            t->units[i].source_hash = new_hash;
            t->units[i].needs_recompile = 1;
            for (j = 0; j < t->count; j++) {
                for (k = 0; k < t->units[j].dep_count; k++) {
                    if (t->units[j].dep_units[k] == id)
                        t->units[j].needs_recompile = 1;
                }
            }
            return;
        }
    }
}

int bld_icu_recompile_count(bld_icu_tracker_t *t) {
    int i, c = 0;
    for (i = 0; i < t->count; i++)
        if (t->units[i].needs_recompile) c++;
    return c;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1600: incremental compilation unit should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.contains("fn bld_icu_invalidate"));
    assert!(code.contains("fn bld_icu_recompile_count"));
}
