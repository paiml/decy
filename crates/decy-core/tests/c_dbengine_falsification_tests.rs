//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1526-C1550: Database Engine Patterns -- page managers, buffer pools,
//! WAL records, B-tree operations, query engines, transactions, index
//! structures, and storage utilities.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world database engine patterns commonly
//! found in SQLite, PostgreSQL, MySQL/InnoDB, LevelDB, and similar
//! database systems -- all expressed as valid C99.
//!
//! Organization:
//! - C1526-C1530: Storage engine (page manager, buffer pool, WAL record, B-tree node, leaf page)
//! - C1531-C1535: Query engine (tuple format, predicate eval, hash join bucket, sort merge, aggregate accum)
//! - C1536-C1540: Transaction (MVCC version chain, lock table, deadlock detector, commit log, snapshot isolation)
//! - C1541-C1545: Index structures (B+ tree insert, hash index, bitmap index, bloom filter index, skip list index)
//! - C1546-C1550: Utilities (row ID generator, statistics collector, query plan node, cursor iterator, compaction scheduler)
//!
//! Results: 24 passing, 1 falsified (96.0% pass rate)

// ============================================================================
// C1526-C1530: Storage Engine
// ============================================================================

/// C1526: Page manager with fixed-size page allocation and free list
#[test]
fn c1526_page_manager() {
    let c_code = r#"
#define DB_PAGE_SIZE 4096
#define DB_MAX_PAGES 256

typedef struct {
    int free_list[DB_MAX_PAGES];
    int free_count;
    int total_pages;
    int allocated;
} db_page_mgr_t;

void db_page_mgr_init(db_page_mgr_t *pm) {
    int i;
    pm->free_count = DB_MAX_PAGES;
    pm->total_pages = DB_MAX_PAGES;
    pm->allocated = 0;
    for (i = 0; i < DB_MAX_PAGES; i++)
        pm->free_list[i] = DB_MAX_PAGES - 1 - i;
}

int db_page_alloc(db_page_mgr_t *pm) {
    if (pm->free_count <= 0) return -1;
    pm->free_count--;
    pm->allocated++;
    return pm->free_list[pm->free_count];
}

void db_page_free(db_page_mgr_t *pm, int page_id) {
    if (pm->free_count >= DB_MAX_PAGES) return;
    if (page_id < 0 || page_id >= DB_MAX_PAGES) return;
    pm->free_list[pm->free_count] = page_id;
    pm->free_count++;
    pm->allocated--;
}

int db_page_mgr_usage(db_page_mgr_t *pm) {
    return pm->allocated;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1526: Page manager should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1526: Output should not be empty");
    assert!(code.contains("fn db_page_alloc"), "C1526: Should contain db_page_alloc");
    assert!(code.contains("fn db_page_free"), "C1526: Should contain db_page_free");
}

/// C1527: Buffer pool with pin/unpin and dirty page tracking
#[test]
fn c1527_buffer_pool() {
    let c_code = r#"
#define DB_BUF_POOL_SIZE 64

typedef struct {
    int page_id[DB_BUF_POOL_SIZE];
    int pin_count[DB_BUF_POOL_SIZE];
    int dirty[DB_BUF_POOL_SIZE];
    int valid[DB_BUF_POOL_SIZE];
    int clock_hand;
    int size;
} db_buf_pool_t;

void db_buf_pool_init(db_buf_pool_t *bp) {
    int i;
    bp->clock_hand = 0;
    bp->size = DB_BUF_POOL_SIZE;
    for (i = 0; i < DB_BUF_POOL_SIZE; i++) {
        bp->page_id[i] = -1;
        bp->pin_count[i] = 0;
        bp->dirty[i] = 0;
        bp->valid[i] = 0;
    }
}

int db_buf_pool_find(db_buf_pool_t *bp, int pid) {
    int i;
    for (i = 0; i < bp->size; i++) {
        if (bp->valid[i] && bp->page_id[i] == pid) return i;
    }
    return -1;
}

int db_buf_pool_pin(db_buf_pool_t *bp, int pid) {
    int idx = db_buf_pool_find(bp, pid);
    if (idx >= 0) {
        bp->pin_count[idx]++;
        return idx;
    }
    return -1;
}

void db_buf_pool_unpin(db_buf_pool_t *bp, int idx, int is_dirty) {
    if (idx < 0 || idx >= bp->size) return;
    if (bp->pin_count[idx] > 0) bp->pin_count[idx]--;
    if (is_dirty) bp->dirty[idx] = 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1527: Buffer pool should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1527: Output should not be empty");
    assert!(code.contains("fn db_buf_pool_pin"), "C1527: Should contain db_buf_pool_pin");
    assert!(code.contains("fn db_buf_pool_unpin"), "C1527: Should contain db_buf_pool_unpin");
}

/// C1528: WAL (Write-Ahead Log) record append and checksum
#[test]
fn c1528_wal_record() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define DB_WAL_MAX_RECORDS 512

typedef struct {
    uint64_t lsn;
    uint32_t txn_id;
    uint32_t page_id;
    int op_type;
    int data_len;
} db_wal_record_t;

typedef struct {
    db_wal_record_t records[DB_WAL_MAX_RECORDS];
    int count;
    uint64_t next_lsn;
} db_wal_t;

void db_wal_init(db_wal_t *wal) {
    wal->count = 0;
    wal->next_lsn = 1;
}

int db_wal_append(db_wal_t *wal, uint32_t txn_id, uint32_t page_id, int op_type) {
    if (wal->count >= DB_WAL_MAX_RECORDS) return -1;
    db_wal_record_t *rec = &wal->records[wal->count];
    rec->lsn = wal->next_lsn++;
    rec->txn_id = txn_id;
    rec->page_id = page_id;
    rec->op_type = op_type;
    rec->data_len = 0;
    wal->count++;
    return 0;
}

uint32_t db_wal_checksum(db_wal_t *wal) {
    uint32_t sum = 0;
    int i;
    for (i = 0; i < wal->count; i++) {
        sum ^= wal->records[i].txn_id;
        sum ^= wal->records[i].page_id;
        sum += (uint32_t)wal->records[i].lsn;
    }
    return sum;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1528: WAL record should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1528: Output should not be empty");
    assert!(code.contains("fn db_wal_append"), "C1528: Should contain db_wal_append");
    assert!(code.contains("fn db_wal_checksum"), "C1528: Should contain db_wal_checksum");
}

/// C1529: B-tree node with key search and insertion
#[test]
fn c1529_btree_node() {
    let c_code = r#"
#define DB_BT_ORDER 8
#define DB_BT_MAX_KEYS (2 * DB_BT_ORDER - 1)

typedef struct {
    int keys[DB_BT_MAX_KEYS];
    int values[DB_BT_MAX_KEYS];
    int num_keys;
    int is_leaf;
} db_bt_node_t;

void db_bt_node_init(db_bt_node_t *n, int leaf) {
    n->num_keys = 0;
    n->is_leaf = leaf;
}

int db_bt_search_key(db_bt_node_t *n, int key) {
    int lo = 0;
    int hi = n->num_keys - 1;
    while (lo <= hi) {
        int mid = lo + (hi - lo) / 2;
        if (n->keys[mid] == key) return mid;
        if (n->keys[mid] < key) lo = mid + 1;
        else hi = mid - 1;
    }
    return -1;
}

int db_bt_insert_key(db_bt_node_t *n, int key, int value) {
    if (n->num_keys >= DB_BT_MAX_KEYS) return -1;
    int i = n->num_keys - 1;
    while (i >= 0 && n->keys[i] > key) {
        n->keys[i + 1] = n->keys[i];
        n->values[i + 1] = n->values[i];
        i--;
    }
    n->keys[i + 1] = key;
    n->values[i + 1] = value;
    n->num_keys++;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1529: B-tree node should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1529: Output should not be empty");
    assert!(code.contains("fn db_bt_search_key"), "C1529: Should contain db_bt_search_key");
    assert!(code.contains("fn db_bt_insert_key"), "C1529: Should contain db_bt_insert_key");
}

/// C1530: Leaf page with slot directory and record storage
#[test]
fn c1530_leaf_page() {
    let c_code = r#"
#define DB_LEAF_SLOTS 32

typedef struct {
    int keys[DB_LEAF_SLOTS];
    int offsets[DB_LEAF_SLOTS];
    int lengths[DB_LEAF_SLOTS];
    int num_slots;
    int free_offset;
} db_leaf_page_t;

void db_leaf_init(db_leaf_page_t *lp) {
    lp->num_slots = 0;
    lp->free_offset = 0;
}

int db_leaf_insert(db_leaf_page_t *lp, int key, int data_len) {
    if (lp->num_slots >= DB_LEAF_SLOTS) return -1;
    int idx = lp->num_slots;
    lp->keys[idx] = key;
    lp->offsets[idx] = lp->free_offset;
    lp->lengths[idx] = data_len;
    lp->free_offset += data_len;
    lp->num_slots++;
    return idx;
}

int db_leaf_find(db_leaf_page_t *lp, int key) {
    int i;
    for (i = 0; i < lp->num_slots; i++) {
        if (lp->keys[i] == key) return i;
    }
    return -1;
}

int db_leaf_used_space(db_leaf_page_t *lp) {
    return lp->free_offset;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1530: Leaf page should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1530: Output should not be empty");
    assert!(code.contains("fn db_leaf_insert"), "C1530: Should contain db_leaf_insert");
    assert!(code.contains("fn db_leaf_find"), "C1530: Should contain db_leaf_find");
}

// ============================================================================
// C1531-C1535: Query Engine
// ============================================================================

/// C1531: Tuple format with fixed-width column storage
#[test]
fn c1531_tuple_format() {
    let c_code = r#"
#define DB_TUPLE_MAX_COLS 16

typedef struct {
    int col_offsets[DB_TUPLE_MAX_COLS];
    int col_widths[DB_TUPLE_MAX_COLS];
    int col_types[DB_TUPLE_MAX_COLS];
    int num_cols;
    int total_width;
} db_tuple_desc_t;

void db_tuple_desc_init(db_tuple_desc_t *td) {
    td->num_cols = 0;
    td->total_width = 0;
}

int db_tuple_add_col(db_tuple_desc_t *td, int width, int col_type) {
    if (td->num_cols >= DB_TUPLE_MAX_COLS) return -1;
    int idx = td->num_cols;
    td->col_offsets[idx] = td->total_width;
    td->col_widths[idx] = width;
    td->col_types[idx] = col_type;
    td->total_width += width;
    td->num_cols++;
    return idx;
}

int db_tuple_get_offset(db_tuple_desc_t *td, int col_idx) {
    if (col_idx < 0 || col_idx >= td->num_cols) return -1;
    return td->col_offsets[col_idx];
}

int db_tuple_row_size(db_tuple_desc_t *td) {
    return td->total_width;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1531: Tuple format should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1531: Output should not be empty");
    assert!(code.contains("fn db_tuple_add_col"), "C1531: Should contain db_tuple_add_col");
    assert!(code.contains("fn db_tuple_row_size"), "C1531: Should contain db_tuple_row_size");
}

/// C1532: Predicate evaluation with comparison operators
#[test]
fn c1532_predicate_evaluation() {
    let c_code = r#"
#define DB_PRED_EQ 0
#define DB_PRED_LT 1
#define DB_PRED_GT 2
#define DB_PRED_LE 3
#define DB_PRED_GE 4
#define DB_PRED_NE 5

typedef struct {
    int col_idx;
    int op;
    int operand;
} db_predicate_t;

int db_pred_eval(db_predicate_t *pred, int value) {
    switch (pred->op) {
        case DB_PRED_EQ: return value == pred->operand;
        case DB_PRED_LT: return value < pred->operand;
        case DB_PRED_GT: return value > pred->operand;
        case DB_PRED_LE: return value <= pred->operand;
        case DB_PRED_GE: return value >= pred->operand;
        case DB_PRED_NE: return value != pred->operand;
        default: return 0;
    }
}

int db_pred_and(db_predicate_t *preds, int count, int *values) {
    int i;
    for (i = 0; i < count; i++) {
        if (!db_pred_eval(&preds[i], values[preds[i].col_idx]))
            return 0;
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1532: Predicate eval should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1532: Output should not be empty");
    assert!(code.contains("fn db_pred_eval"), "C1532: Should contain db_pred_eval");
    assert!(code.contains("fn db_pred_and"), "C1532: Should contain db_pred_and");
}

/// C1533: Hash join bucket with overflow chaining
#[test]
fn c1533_hash_join_bucket() {
    let c_code = r#"
#define DB_HJ_BUCKETS 64
#define DB_HJ_ENTRIES 256

typedef struct {
    int key;
    int row_id;
    int next;
} db_hj_entry_t;

typedef struct {
    int heads[DB_HJ_BUCKETS];
    db_hj_entry_t entries[DB_HJ_ENTRIES];
    int entry_count;
} db_hj_table_t;

void db_hj_init(db_hj_table_t *ht) {
    int i;
    ht->entry_count = 0;
    for (i = 0; i < DB_HJ_BUCKETS; i++)
        ht->heads[i] = -1;
}

int db_hj_hash(int key) {
    return (key * 2654435761u) % DB_HJ_BUCKETS;
}

int db_hj_insert(db_hj_table_t *ht, int key, int row_id) {
    if (ht->entry_count >= DB_HJ_ENTRIES) return -1;
    int bucket = db_hj_hash(key);
    int idx = ht->entry_count++;
    ht->entries[idx].key = key;
    ht->entries[idx].row_id = row_id;
    ht->entries[idx].next = ht->heads[bucket];
    ht->heads[bucket] = idx;
    return 0;
}

int db_hj_probe(db_hj_table_t *ht, int key) {
    int bucket = db_hj_hash(key);
    int idx = ht->heads[bucket];
    while (idx >= 0) {
        if (ht->entries[idx].key == key)
            return ht->entries[idx].row_id;
        idx = ht->entries[idx].next;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1533: Hash join should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1533: Output should not be empty");
    assert!(code.contains("fn db_hj_insert"), "C1533: Should contain db_hj_insert");
    assert!(code.contains("fn db_hj_probe"), "C1533: Should contain db_hj_probe");
}

/// C1534: Sort-merge join with two sorted runs
#[test]
fn c1534_sort_merge() {
    let c_code = r#"
#define DB_SM_MAX 128

typedef struct {
    int keys[DB_SM_MAX];
    int vals[DB_SM_MAX];
    int count;
} db_sm_run_t;

void db_sm_run_init(db_sm_run_t *r) {
    r->count = 0;
}

int db_sm_run_add(db_sm_run_t *r, int key, int val) {
    if (r->count >= DB_SM_MAX) return -1;
    r->keys[r->count] = key;
    r->vals[r->count] = val;
    r->count++;
    return 0;
}

int db_sm_merge_count(db_sm_run_t *left, db_sm_run_t *right) {
    int li = 0;
    int ri = 0;
    int matches = 0;
    while (li < left->count && ri < right->count) {
        if (left->keys[li] == right->keys[ri]) {
            matches++;
            li++;
            ri++;
        } else if (left->keys[li] < right->keys[ri]) {
            li++;
        } else {
            ri++;
        }
    }
    return matches;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1534: Sort merge should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1534: Output should not be empty");
    assert!(code.contains("fn db_sm_run_add"), "C1534: Should contain db_sm_run_add");
    assert!(code.contains("fn db_sm_merge_count"), "C1534: Should contain db_sm_merge_count");
}

/// C1535: Aggregate accumulator for SUM/COUNT/MIN/MAX/AVG
#[test]
fn c1535_aggregate_accumulator() {
    let c_code = r#"
#define DB_AGG_SUM 0
#define DB_AGG_COUNT 1
#define DB_AGG_MIN 2
#define DB_AGG_MAX 3

typedef struct {
    int agg_type;
    long sum;
    int count;
    int min_val;
    int max_val;
} db_agg_accum_t;

void db_agg_init(db_agg_accum_t *a, int agg_type) {
    a->agg_type = agg_type;
    a->sum = 0;
    a->count = 0;
    a->min_val = 0x7FFFFFFF;
    a->max_val = -2147483647 - 1;
}

void db_agg_feed(db_agg_accum_t *a, int value) {
    a->sum += value;
    a->count++;
    if (value < a->min_val) a->min_val = value;
    if (value > a->max_val) a->max_val = value;
}

long db_agg_result(db_agg_accum_t *a) {
    switch (a->agg_type) {
        case DB_AGG_SUM: return a->sum;
        case DB_AGG_COUNT: return a->count;
        case DB_AGG_MIN: return a->min_val;
        case DB_AGG_MAX: return a->max_val;
        default: return 0;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1535: Aggregate accum should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1535: Output should not be empty");
    assert!(code.contains("fn db_agg_feed"), "C1535: Should contain db_agg_feed");
    assert!(code.contains("fn db_agg_result"), "C1535: Should contain db_agg_result");
}

// ============================================================================
// C1536-C1540: Transaction
// ============================================================================

/// C1536: MVCC version chain with version visibility checks
#[test]
fn c1536_mvcc_version_chain() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define DB_MVCC_MAX_VERSIONS 64

typedef struct {
    uint32_t txn_id;
    uint64_t begin_ts;
    uint64_t end_ts;
    int value;
    int next_version;
} db_mvcc_version_t;

typedef struct {
    db_mvcc_version_t versions[DB_MVCC_MAX_VERSIONS];
    int count;
    int head;
} db_mvcc_chain_t;

void db_mvcc_init(db_mvcc_chain_t *ch) {
    ch->count = 0;
    ch->head = -1;
}

int db_mvcc_add_version(db_mvcc_chain_t *ch, uint32_t txn_id, uint64_t ts, int value) {
    if (ch->count >= DB_MVCC_MAX_VERSIONS) return -1;
    int idx = ch->count++;
    ch->versions[idx].txn_id = txn_id;
    ch->versions[idx].begin_ts = ts;
    ch->versions[idx].end_ts = 0;
    ch->versions[idx].value = value;
    ch->versions[idx].next_version = ch->head;
    if (ch->head >= 0)
        ch->versions[ch->head].end_ts = ts;
    ch->head = idx;
    return idx;
}

int db_mvcc_read_at(db_mvcc_chain_t *ch, uint64_t read_ts) {
    int idx = ch->head;
    while (idx >= 0) {
        uint64_t begin = ch->versions[idx].begin_ts;
        uint64_t end = ch->versions[idx].end_ts;
        if (begin <= read_ts && (end == 0 || end > read_ts))
            return ch->versions[idx].value;
        idx = ch->versions[idx].next_version;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1536: MVCC version chain should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1536: Output should not be empty");
    assert!(code.contains("fn db_mvcc_add_version"), "C1536: Should contain db_mvcc_add_version");
    assert!(code.contains("fn db_mvcc_read_at"), "C1536: Should contain db_mvcc_read_at");
}

/// C1537: Lock table with shared/exclusive lock modes
#[test]
fn c1537_lock_table() {
    let c_code = r#"
#define DB_LK_MAX 128
#define DB_LK_NONE 0
#define DB_LK_SHARED 1
#define DB_LK_EXCLUSIVE 2

typedef struct {
    int resource_id;
    int mode;
    int holder_txn;
    int shared_count;
} db_lock_entry_t;

typedef struct {
    db_lock_entry_t locks[DB_LK_MAX];
    int count;
} db_lock_table_t;

void db_lock_init(db_lock_table_t *lt) {
    lt->count = 0;
}

int db_lock_find(db_lock_table_t *lt, int resource_id) {
    int i;
    for (i = 0; i < lt->count; i++) {
        if (lt->locks[i].resource_id == resource_id)
            return i;
    }
    return -1;
}

int db_lock_acquire(db_lock_table_t *lt, int resource_id, int txn_id, int mode) {
    int idx = db_lock_find(lt, resource_id);
    if (idx >= 0) {
        if (lt->locks[idx].mode == DB_LK_EXCLUSIVE) return -1;
        if (mode == DB_LK_EXCLUSIVE) return -1;
        lt->locks[idx].shared_count++;
        return 0;
    }
    if (lt->count >= DB_LK_MAX) return -2;
    idx = lt->count++;
    lt->locks[idx].resource_id = resource_id;
    lt->locks[idx].mode = mode;
    lt->locks[idx].holder_txn = txn_id;
    lt->locks[idx].shared_count = (mode == DB_LK_SHARED) ? 1 : 0;
    return 0;
}

void db_lock_release(db_lock_table_t *lt, int resource_id) {
    int idx = db_lock_find(lt, resource_id);
    if (idx < 0) return;
    if (lt->locks[idx].mode == DB_LK_SHARED) {
        lt->locks[idx].shared_count--;
        if (lt->locks[idx].shared_count > 0) return;
    }
    lt->locks[idx] = lt->locks[lt->count - 1];
    lt->count--;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1537: Lock table should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1537: Output should not be empty");
    assert!(code.contains("fn db_lock_acquire"), "C1537: Should contain db_lock_acquire");
    assert!(code.contains("fn db_lock_release"), "C1537: Should contain db_lock_release");
}

/// C1538: Deadlock detector using wait-for graph cycle detection
#[test]
fn c1538_deadlock_detector() {
    let c_code = r#"
#define DB_DL_MAX_TXNS 32

typedef struct {
    int waits_for[DB_DL_MAX_TXNS][DB_DL_MAX_TXNS];
    int active[DB_DL_MAX_TXNS];
    int num_txns;
} db_deadlock_t;

void db_deadlock_init(db_deadlock_t *dd) {
    int i;
    int j;
    dd->num_txns = DB_DL_MAX_TXNS;
    for (i = 0; i < DB_DL_MAX_TXNS; i++) {
        dd->active[i] = 0;
        for (j = 0; j < DB_DL_MAX_TXNS; j++)
            dd->waits_for[i][j] = 0;
    }
}

void db_deadlock_add_edge(db_deadlock_t *dd, int from, int to) {
    if (from >= 0 && from < DB_DL_MAX_TXNS && to >= 0 && to < DB_DL_MAX_TXNS) {
        dd->waits_for[from][to] = 1;
        dd->active[from] = 1;
        dd->active[to] = 1;
    }
}

static int db_deadlock_dfs(db_deadlock_t *dd, int node, int *visited, int *stack) {
    visited[node] = 1;
    stack[node] = 1;
    int i;
    for (i = 0; i < DB_DL_MAX_TXNS; i++) {
        if (dd->waits_for[node][i]) {
            if (!visited[i]) {
                if (db_deadlock_dfs(dd, i, visited, stack)) return 1;
            } else if (stack[i]) {
                return 1;
            }
        }
    }
    stack[node] = 0;
    return 0;
}

int db_deadlock_detect(db_deadlock_t *dd) {
    int visited[DB_DL_MAX_TXNS];
    int stack[DB_DL_MAX_TXNS];
    int i;
    for (i = 0; i < DB_DL_MAX_TXNS; i++) {
        visited[i] = 0;
        stack[i] = 0;
    }
    for (i = 0; i < DB_DL_MAX_TXNS; i++) {
        if (dd->active[i] && !visited[i]) {
            if (db_deadlock_dfs(dd, i, visited, stack)) return 1;
        }
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1538: Deadlock detector should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1538: Output should not be empty");
    assert!(code.contains("fn db_deadlock_add_edge"), "C1538: Should contain db_deadlock_add_edge");
    assert!(code.contains("fn db_deadlock_detect"), "C1538: Should contain db_deadlock_detect");
}

/// C1539: Commit log with transaction state tracking
#[test]
fn c1539_commit_log() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define DB_CLOG_MAX 256
#define DB_TXN_ACTIVE 0
#define DB_TXN_COMMITTED 1
#define DB_TXN_ABORTED 2

typedef struct {
    uint32_t txn_id;
    int state;
    uint64_t commit_ts;
} db_clog_entry_t;

typedef struct {
    db_clog_entry_t entries[DB_CLOG_MAX];
    int count;
    uint32_t next_txn_id;
} db_clog_t;

void db_clog_init(db_clog_t *cl) {
    cl->count = 0;
    cl->next_txn_id = 1;
}

uint32_t db_clog_begin(db_clog_t *cl) {
    if (cl->count >= DB_CLOG_MAX) return 0;
    int idx = cl->count++;
    cl->entries[idx].txn_id = cl->next_txn_id++;
    cl->entries[idx].state = DB_TXN_ACTIVE;
    cl->entries[idx].commit_ts = 0;
    return cl->entries[idx].txn_id;
}

int db_clog_commit(db_clog_t *cl, uint32_t txn_id, uint64_t ts) {
    int i;
    for (i = 0; i < cl->count; i++) {
        if (cl->entries[i].txn_id == txn_id && cl->entries[i].state == DB_TXN_ACTIVE) {
            cl->entries[i].state = DB_TXN_COMMITTED;
            cl->entries[i].commit_ts = ts;
            return 0;
        }
    }
    return -1;
}

int db_clog_is_committed(db_clog_t *cl, uint32_t txn_id) {
    int i;
    for (i = 0; i < cl->count; i++) {
        if (cl->entries[i].txn_id == txn_id)
            return cl->entries[i].state == DB_TXN_COMMITTED;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1539: Commit log should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1539: Output should not be empty");
    assert!(code.contains("fn db_clog_begin"), "C1539: Should contain db_clog_begin");
    assert!(code.contains("fn db_clog_commit"), "C1539: Should contain db_clog_commit");
}

/// C1540: Snapshot isolation with active transaction tracking
#[test]
fn c1540_snapshot_isolation() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define DB_SNAP_MAX_ACTIVE 64

typedef struct {
    uint32_t active_txns[DB_SNAP_MAX_ACTIVE];
    int active_count;
    uint64_t snapshot_ts;
    uint32_t min_active;
    uint32_t max_active;
} db_snapshot_t;

void db_snap_init(db_snapshot_t *s, uint64_t ts) {
    s->active_count = 0;
    s->snapshot_ts = ts;
    s->min_active = 0xFFFFFFFF;
    s->max_active = 0;
}

void db_snap_add_active(db_snapshot_t *s, uint32_t txn_id) {
    if (s->active_count >= DB_SNAP_MAX_ACTIVE) return;
    s->active_txns[s->active_count++] = txn_id;
    if (txn_id < s->min_active) s->min_active = txn_id;
    if (txn_id > s->max_active) s->max_active = txn_id;
}

int db_snap_is_visible(db_snapshot_t *s, uint32_t txn_id, uint64_t commit_ts) {
    int i;
    if (commit_ts > s->snapshot_ts) return 0;
    for (i = 0; i < s->active_count; i++) {
        if (s->active_txns[i] == txn_id) return 0;
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1540: Snapshot isolation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1540: Output should not be empty");
    assert!(code.contains("fn db_snap_add_active"), "C1540: Should contain db_snap_add_active");
    assert!(code.contains("fn db_snap_is_visible"), "C1540: Should contain db_snap_is_visible");
}

// ============================================================================
// C1541-C1545: Index Structures
// ============================================================================

/// C1541: B+ tree insert with leaf splitting
#[test]
fn c1541_bplus_tree_insert() {
    let c_code = r#"
#define DB_BP_ORDER 4
#define DB_BP_MAX_KEYS (2 * DB_BP_ORDER)
#define DB_BP_MAX_NODES 64

typedef struct {
    int keys[DB_BP_MAX_KEYS];
    int values[DB_BP_MAX_KEYS];
    int num_keys;
    int next_leaf;
    int is_leaf;
} db_bp_node_t;

typedef struct {
    db_bp_node_t nodes[DB_BP_MAX_NODES];
    int node_count;
    int root;
} db_bp_tree_t;

void db_bp_init(db_bp_tree_t *t) {
    t->node_count = 1;
    t->root = 0;
    t->nodes[0].num_keys = 0;
    t->nodes[0].is_leaf = 1;
    t->nodes[0].next_leaf = -1;
}

int db_bp_leaf_insert(db_bp_node_t *node, int key, int value) {
    if (node->num_keys >= DB_BP_MAX_KEYS) return -1;
    int i = node->num_keys - 1;
    while (i >= 0 && node->keys[i] > key) {
        node->keys[i + 1] = node->keys[i];
        node->values[i + 1] = node->values[i];
        i--;
    }
    node->keys[i + 1] = key;
    node->values[i + 1] = value;
    node->num_keys++;
    return 0;
}

int db_bp_search(db_bp_tree_t *t, int key) {
    db_bp_node_t *node = &t->nodes[t->root];
    int i;
    for (i = 0; i < node->num_keys; i++) {
        if (node->keys[i] == key)
            return node->values[i];
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1541: B+ tree insert should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1541: Output should not be empty");
    assert!(code.contains("fn db_bp_leaf_insert"), "C1541: Should contain db_bp_leaf_insert");
    assert!(code.contains("fn db_bp_search"), "C1541: Should contain db_bp_search");
}

/// C1542: Hash index with linear probing
#[test]
fn c1542_hash_index() {
    let c_code = r#"
#define DB_HI_SIZE 128

typedef struct {
    int keys[DB_HI_SIZE];
    int values[DB_HI_SIZE];
    int occupied[DB_HI_SIZE];
    int count;
} db_hash_idx_t;

void db_hash_idx_init(db_hash_idx_t *h) {
    int i;
    h->count = 0;
    for (i = 0; i < DB_HI_SIZE; i++) {
        h->keys[i] = 0;
        h->values[i] = 0;
        h->occupied[i] = 0;
    }
}

static int db_hash_idx_hash(int key) {
    unsigned int k = (unsigned int)key;
    k = ((k >> 16) ^ k) * 0x45d9f3b;
    k = ((k >> 16) ^ k) * 0x45d9f3b;
    k = (k >> 16) ^ k;
    return (int)(k % DB_HI_SIZE);
}

int db_hash_idx_put(db_hash_idx_t *h, int key, int value) {
    if (h->count >= DB_HI_SIZE) return -1;
    int slot = db_hash_idx_hash(key);
    while (h->occupied[slot]) {
        if (h->keys[slot] == key) {
            h->values[slot] = value;
            return 0;
        }
        slot = (slot + 1) % DB_HI_SIZE;
    }
    h->keys[slot] = key;
    h->values[slot] = value;
    h->occupied[slot] = 1;
    h->count++;
    return 0;
}

int db_hash_idx_get(db_hash_idx_t *h, int key) {
    int slot = db_hash_idx_hash(key);
    int start = slot;
    while (h->occupied[slot]) {
        if (h->keys[slot] == key) return h->values[slot];
        slot = (slot + 1) % DB_HI_SIZE;
        if (slot == start) break;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1542: Hash index should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1542: Output should not be empty");
    assert!(code.contains("fn db_hash_idx_put"), "C1542: Should contain db_hash_idx_put");
    assert!(code.contains("fn db_hash_idx_get"), "C1542: Should contain db_hash_idx_get");
}

/// C1543: Bitmap index with bitwise operations
#[test]
fn c1543_bitmap_index() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define DB_BM_WORDS 8
#define DB_BM_BITS (DB_BM_WORDS * 32)

typedef struct {
    uint32_t bits[DB_BM_WORDS];
} db_bitmap_t;

void db_bitmap_clear(db_bitmap_t *bm) {
    int i;
    for (i = 0; i < DB_BM_WORDS; i++)
        bm->bits[i] = 0;
}

void db_bitmap_set(db_bitmap_t *bm, int pos) {
    if (pos < 0 || pos >= DB_BM_BITS) return;
    bm->bits[pos / 32] |= (1u << (pos % 32));
}

int db_bitmap_test(db_bitmap_t *bm, int pos) {
    if (pos < 0 || pos >= DB_BM_BITS) return 0;
    return (bm->bits[pos / 32] >> (pos % 32)) & 1;
}

void db_bitmap_and(db_bitmap_t *dst, db_bitmap_t *a, db_bitmap_t *b) {
    int i;
    for (i = 0; i < DB_BM_WORDS; i++)
        dst->bits[i] = a->bits[i] & b->bits[i];
}

int db_bitmap_popcount(db_bitmap_t *bm) {
    int count = 0;
    int i;
    for (i = 0; i < DB_BM_WORDS; i++) {
        uint32_t v = bm->bits[i];
        while (v) {
            count += v & 1;
            v >>= 1;
        }
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1543: Bitmap index should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1543: Output should not be empty");
    assert!(code.contains("fn db_bitmap_set"), "C1543: Should contain db_bitmap_set");
    assert!(code.contains("fn db_bitmap_popcount"), "C1543: Should contain db_bitmap_popcount");
}

/// C1544: Bloom filter index for approximate membership testing
#[test]
#[ignore = "FALSIFIED: panic in HIR lowering 'For loop must have condition' on db_bloom_hash loop"]
fn c1544_bloom_filter_index() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define DB_BF_SIZE 256
#define DB_BF_WORDS (DB_BF_SIZE / 32)
#define DB_BF_HASHES 3

typedef struct {
    uint32_t bits[DB_BF_WORDS];
    int count;
} db_bloom_t;

void db_bloom_init(db_bloom_t *bf) {
    int i;
    bf->count = 0;
    for (i = 0; i < DB_BF_WORDS; i++)
        bf->bits[i] = 0;
}

static uint32_t db_bloom_hash(int key, int seed) {
    uint32_t h = (uint32_t)key;
    h ^= (uint32_t)seed;
    h *= 2654435761u;
    h ^= h >> 16;
    return h % DB_BF_SIZE;
}

void db_bloom_add(db_bloom_t *bf, int key) {
    int i;
    for (i = 0; i < DB_BF_HASHES; i++) {
        uint32_t pos = db_bloom_hash(key, i);
        bf->bits[pos / 32] |= (1u << (pos % 32));
    }
    bf->count++;
}

int db_bloom_may_contain(db_bloom_t *bf, int key) {
    int i;
    for (i = 0; i < DB_BF_HASHES; i++) {
        uint32_t pos = db_bloom_hash(key, i);
        if (!(bf->bits[pos / 32] & (1u << (pos % 32))))
            return 0;
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1544: Bloom filter should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1544: Output should not be empty");
    assert!(code.contains("fn db_bloom_add"), "C1544: Should contain db_bloom_add");
    assert!(code.contains("fn db_bloom_may_contain"), "C1544: Should contain db_bloom_may_contain");
}

/// C1545: Skip list index with random level generation
#[test]
fn c1545_skip_list_index() {
    let c_code = r#"
#define DB_SL_MAX_LEVEL 8
#define DB_SL_MAX_NODES 128

typedef struct {
    int key;
    int value;
    int forward[DB_SL_MAX_LEVEL];
    int level;
} db_sl_node_t;

typedef struct {
    db_sl_node_t nodes[DB_SL_MAX_NODES];
    int node_count;
    int head;
    int max_level;
    int seed;
} db_skiplist_t;

void db_sl_init(db_skiplist_t *sl) {
    sl->node_count = 1;
    sl->head = 0;
    sl->max_level = 1;
    sl->seed = 42;
    int i;
    sl->nodes[0].key = -2147483647 - 1;
    sl->nodes[0].value = 0;
    sl->nodes[0].level = DB_SL_MAX_LEVEL;
    for (i = 0; i < DB_SL_MAX_LEVEL; i++)
        sl->nodes[0].forward[i] = -1;
}

static int db_sl_random_level(db_skiplist_t *sl) {
    int lvl = 1;
    sl->seed = sl->seed * 1103515245 + 12345;
    while ((sl->seed & 1) && lvl < DB_SL_MAX_LEVEL) {
        lvl++;
        sl->seed = sl->seed * 1103515245 + 12345;
    }
    return lvl;
}

int db_sl_search(db_skiplist_t *sl, int key) {
    int cur = sl->head;
    int i;
    for (i = sl->max_level - 1; i >= 0; i--) {
        while (sl->nodes[cur].forward[i] >= 0 &&
               sl->nodes[sl->nodes[cur].forward[i]].key < key) {
            cur = sl->nodes[cur].forward[i];
        }
    }
    cur = sl->nodes[cur].forward[0];
    if (cur >= 0 && sl->nodes[cur].key == key)
        return sl->nodes[cur].value;
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1545: Skip list index should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1545: Output should not be empty");
    assert!(code.contains("fn db_sl_init"), "C1545: Should contain db_sl_init");
    assert!(code.contains("fn db_sl_search"), "C1545: Should contain db_sl_search");
}

// ============================================================================
// C1546-C1550: Utilities
// ============================================================================

/// C1546: Row ID generator with epoch-based sequencing
#[test]
fn c1546_row_id_generator() {
    let c_code = r#"
typedef unsigned long uint64_t;
typedef unsigned int uint32_t;

typedef struct {
    uint32_t epoch;
    uint32_t sequence;
    uint32_t shard_id;
} db_rowid_gen_t;

void db_rowid_gen_init(db_rowid_gen_t *g, uint32_t shard_id) {
    g->epoch = 0;
    g->sequence = 0;
    g->shard_id = shard_id;
}

uint64_t db_rowid_next(db_rowid_gen_t *g, uint32_t current_epoch) {
    if (current_epoch > g->epoch) {
        g->epoch = current_epoch;
        g->sequence = 0;
    }
    uint64_t id = ((uint64_t)g->epoch << 32) |
                  ((uint64_t)g->shard_id << 24) |
                  (uint64_t)g->sequence;
    g->sequence++;
    return id;
}

uint32_t db_rowid_get_epoch(uint64_t row_id) {
    return (uint32_t)(row_id >> 32);
}

uint32_t db_rowid_get_shard(uint64_t row_id) {
    return (uint32_t)((row_id >> 24) & 0xFF);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1546: Row ID generator should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1546: Output should not be empty");
    assert!(code.contains("fn db_rowid_next"), "C1546: Should contain db_rowid_next");
    assert!(code.contains("fn db_rowid_get_epoch"), "C1546: Should contain db_rowid_get_epoch");
}

/// C1547: Statistics collector for column histograms
#[test]
fn c1547_statistics_collector() {
    let c_code = r#"
#define DB_STAT_BUCKETS 16
#define DB_STAT_MAX_VALS 256

typedef struct {
    int min_val;
    int max_val;
    int bucket_counts[DB_STAT_BUCKETS];
    int total_count;
    long sum;
    int distinct_approx;
} db_col_stats_t;

void db_stats_init(db_col_stats_t *s) {
    int i;
    s->min_val = 0x7FFFFFFF;
    s->max_val = -2147483647 - 1;
    s->total_count = 0;
    s->sum = 0;
    s->distinct_approx = 0;
    for (i = 0; i < DB_STAT_BUCKETS; i++)
        s->bucket_counts[i] = 0;
}

void db_stats_observe(db_col_stats_t *s, int value) {
    if (value < s->min_val) s->min_val = value;
    if (value > s->max_val) s->max_val = value;
    s->sum += value;
    s->total_count++;
}

void db_stats_build_histogram(db_col_stats_t *s) {
    if (s->total_count == 0) return;
    int range = s->max_val - s->min_val + 1;
    if (range <= 0) range = 1;
    int bucket_width = range / DB_STAT_BUCKETS;
    if (bucket_width <= 0) bucket_width = 1;
    s->distinct_approx = range < s->total_count ? range : s->total_count;
}

int db_stats_estimate_selectivity(db_col_stats_t *s, int low, int high) {
    if (s->total_count == 0) return 0;
    int range = s->max_val - s->min_val + 1;
    if (range <= 0) return s->total_count;
    int query_range = high - low + 1;
    if (query_range <= 0) return 0;
    return (s->total_count * query_range) / range;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1547: Stats collector should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1547: Output should not be empty");
    assert!(code.contains("fn db_stats_observe"), "C1547: Should contain db_stats_observe");
    assert!(code.contains("fn db_stats_estimate_selectivity"), "C1547: Should contain db_stats_estimate_selectivity");
}

/// C1548: Query plan node with cost estimation
#[test]
fn c1548_query_plan_node() {
    let c_code = r#"
#define DB_QP_SCAN 0
#define DB_QP_INDEX 1
#define DB_QP_NESTED_LOOP 2
#define DB_QP_HASH_JOIN 3
#define DB_QP_SORT 4

typedef struct {
    int node_type;
    int table_id;
    int estimated_rows;
    int cost;
    int left_child;
    int right_child;
} db_qp_node_t;

#define DB_QP_MAX_NODES 32

typedef struct {
    db_qp_node_t nodes[DB_QP_MAX_NODES];
    int count;
    int root;
} db_qp_tree_t;

void db_qp_init(db_qp_tree_t *qp) {
    qp->count = 0;
    qp->root = -1;
}

int db_qp_add_scan(db_qp_tree_t *qp, int table_id, int rows) {
    if (qp->count >= DB_QP_MAX_NODES) return -1;
    int idx = qp->count++;
    qp->nodes[idx].node_type = DB_QP_SCAN;
    qp->nodes[idx].table_id = table_id;
    qp->nodes[idx].estimated_rows = rows;
    qp->nodes[idx].cost = rows;
    qp->nodes[idx].left_child = -1;
    qp->nodes[idx].right_child = -1;
    return idx;
}

int db_qp_add_join(db_qp_tree_t *qp, int join_type, int left, int right) {
    if (qp->count >= DB_QP_MAX_NODES) return -1;
    int idx = qp->count++;
    qp->nodes[idx].node_type = join_type;
    qp->nodes[idx].left_child = left;
    qp->nodes[idx].right_child = right;
    int lrows = qp->nodes[left].estimated_rows;
    int rrows = qp->nodes[right].estimated_rows;
    if (join_type == DB_QP_NESTED_LOOP)
        qp->nodes[idx].cost = lrows * rrows;
    else
        qp->nodes[idx].cost = lrows + rrows;
    qp->nodes[idx].estimated_rows = (lrows * rrows) / 10;
    return idx;
}

int db_qp_total_cost(db_qp_tree_t *qp, int node_idx) {
    if (node_idx < 0 || node_idx >= qp->count) return 0;
    int cost = qp->nodes[node_idx].cost;
    cost += db_qp_total_cost(qp, qp->nodes[node_idx].left_child);
    cost += db_qp_total_cost(qp, qp->nodes[node_idx].right_child);
    return cost;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1548: Query plan node should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1548: Output should not be empty");
    assert!(code.contains("fn db_qp_add_scan"), "C1548: Should contain db_qp_add_scan");
    assert!(code.contains("fn db_qp_total_cost"), "C1548: Should contain db_qp_total_cost");
}

/// C1549: Cursor iterator for sequential page scanning
#[test]
fn c1549_cursor_iterator() {
    let c_code = r#"
#define DB_CUR_MAX_PAGES 128
#define DB_CUR_PAGE_SLOTS 16

typedef struct {
    int page_ids[DB_CUR_MAX_PAGES];
    int page_count;
    int current_page;
    int current_slot;
    int direction;
    int exhausted;
} db_cursor_t;

void db_cursor_init(db_cursor_t *cur, int direction) {
    cur->page_count = 0;
    cur->current_page = 0;
    cur->current_slot = 0;
    cur->direction = direction;
    cur->exhausted = 0;
}

int db_cursor_add_page(db_cursor_t *cur, int page_id) {
    if (cur->page_count >= DB_CUR_MAX_PAGES) return -1;
    cur->page_ids[cur->page_count++] = page_id;
    return 0;
}

int db_cursor_next(db_cursor_t *cur) {
    if (cur->exhausted) return -1;
    int result = cur->page_ids[cur->current_page] * DB_CUR_PAGE_SLOTS + cur->current_slot;
    cur->current_slot++;
    if (cur->current_slot >= DB_CUR_PAGE_SLOTS) {
        cur->current_slot = 0;
        cur->current_page += cur->direction;
        if (cur->current_page < 0 || cur->current_page >= cur->page_count)
            cur->exhausted = 1;
    }
    return result;
}

void db_cursor_reset(db_cursor_t *cur) {
    if (cur->direction >= 0)
        cur->current_page = 0;
    else
        cur->current_page = cur->page_count - 1;
    cur->current_slot = 0;
    cur->exhausted = 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1549: Cursor iterator should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1549: Output should not be empty");
    assert!(code.contains("fn db_cursor_next"), "C1549: Should contain db_cursor_next");
    assert!(code.contains("fn db_cursor_reset"), "C1549: Should contain db_cursor_reset");
}

/// C1550: Compaction scheduler with level-based priority
#[test]
fn c1550_compaction_scheduler() {
    let c_code = r#"
#define DB_COMPACT_MAX_LEVELS 8
#define DB_COMPACT_MAX_TASKS 32

typedef struct {
    int level;
    int priority;
    int num_files;
    int total_size;
    int status;
} db_compact_task_t;

typedef struct {
    db_compact_task_t tasks[DB_COMPACT_MAX_TASKS];
    int task_count;
    int level_sizes[DB_COMPACT_MAX_LEVELS];
    int level_limits[DB_COMPACT_MAX_LEVELS];
} db_compaction_t;

void db_compact_init(db_compaction_t *c) {
    int i;
    c->task_count = 0;
    for (i = 0; i < DB_COMPACT_MAX_LEVELS; i++) {
        c->level_sizes[i] = 0;
        c->level_limits[i] = 10 * (1 << i);
    }
}

int db_compact_needs_compaction(db_compaction_t *c, int level) {
    if (level < 0 || level >= DB_COMPACT_MAX_LEVELS) return 0;
    return c->level_sizes[level] > c->level_limits[level];
}

int db_compact_schedule(db_compaction_t *c, int level, int num_files, int total_size) {
    if (c->task_count >= DB_COMPACT_MAX_TASKS) return -1;
    if (level < 0 || level >= DB_COMPACT_MAX_LEVELS) return -1;
    int idx = c->task_count++;
    c->tasks[idx].level = level;
    c->tasks[idx].priority = DB_COMPACT_MAX_LEVELS - level;
    c->tasks[idx].num_files = num_files;
    c->tasks[idx].total_size = total_size;
    c->tasks[idx].status = 0;
    return idx;
}

int db_compact_pick_best(db_compaction_t *c) {
    int best = -1;
    int best_prio = -1;
    int i;
    for (i = 0; i < c->task_count; i++) {
        if (c->tasks[i].status == 0 && c->tasks[i].priority > best_prio) {
            best_prio = c->tasks[i].priority;
            best = i;
        }
    }
    if (best >= 0) c->tasks[best].status = 1;
    return best;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1550: Compaction scheduler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1550: Output should not be empty");
    assert!(code.contains("fn db_compact_schedule"), "C1550: Should contain db_compact_schedule");
    assert!(code.contains("fn db_compact_pick_best"), "C1550: Should contain db_compact_pick_best");
}
