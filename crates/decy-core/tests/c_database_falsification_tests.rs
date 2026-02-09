//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C351-C375: Database and Storage Engine patterns -- the kind of C code found
//! in databases, key-value stores, B-trees, WAL, and storage engines.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world database programming patterns commonly
//! found in SQLite, LevelDB, RocksDB, PostgreSQL, and similar storage
//! engines -- all expressed as valid C99.
//!
//! Organization:
//! - C351-C355: Core storage structures (B-tree, LSM, WAL, buffer pool, tuples)
//! - C356-C360: Indexing and concurrency (RLE, hash index, bloom filter, MVCC, locks)
//! - C361-C365: Query execution (cost estimator, merge join, index scan, checkpoint, redo)
//! - C366-C370: Storage management (bitmap, extent allocator, serialization, B+tree, compaction)
//! - C371-C375: Catalog and recovery (cursor, schema, histogram, buffer pin, undo log)
//!
//! Results: 24 passing, 1 falsified (96.0% pass rate)

// ============================================================================
// C351-C355: Core Storage Structures
// ============================================================================

#[test]
fn c351_btree_node_split() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define BTREE_ORDER 4
#define BTREE_MAX_KEYS (2 * BTREE_ORDER - 1)
#define BTREE_MIN_KEYS (BTREE_ORDER - 1)

typedef struct btree_node {
    uint32_t keys[BTREE_MAX_KEYS];
    uint32_t values[BTREE_MAX_KEYS];
    int children[BTREE_MAX_KEYS + 1];
    int num_keys;
    int is_leaf;
    int id;
} btree_node_t;

int btree_find_insert_pos(const btree_node_t *node, uint32_t key) {
    int lo = 0;
    int hi = node->num_keys - 1;
    while (lo <= hi) {
        int mid = lo + (hi - lo) / 2;
        if (node->keys[mid] < key) {
            lo = mid + 1;
        } else if (node->keys[mid] > key) {
            hi = mid - 1;
        } else {
            return mid;
        }
    }
    return lo;
}

void btree_insert_into_leaf(btree_node_t *node, uint32_t key, uint32_t value) {
    int pos = btree_find_insert_pos(node, key);
    int i;
    for (i = node->num_keys; i > pos; i--) {
        node->keys[i] = node->keys[i - 1];
        node->values[i] = node->values[i - 1];
    }
    node->keys[pos] = key;
    node->values[pos] = value;
    node->num_keys++;
}

uint32_t btree_split_node(btree_node_t *left, btree_node_t *right) {
    int mid = left->num_keys / 2;
    uint32_t median_key = left->keys[mid];
    int i;
    int j = 0;
    for (i = mid + 1; i < left->num_keys; i++) {
        right->keys[j] = left->keys[i];
        right->values[j] = left->values[i];
        j++;
    }
    right->num_keys = j;
    left->num_keys = mid;
    right->is_leaf = left->is_leaf;
    return median_key;
}

int btree_node_is_full(const btree_node_t *node) {
    return node->num_keys >= BTREE_MAX_KEYS;
}

int btree_search(const btree_node_t *node, uint32_t key, uint32_t *value) {
    int pos = btree_find_insert_pos(node, key);
    if (pos < node->num_keys && node->keys[pos] == key) {
        *value = node->values[pos];
        return 1;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C351: B-tree node split should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C351: Output should not be empty");
    assert!(
        code.contains("fn btree_split_node"),
        "C351: Should contain btree_split_node function"
    );
    assert!(
        code.contains("fn btree_find_insert_pos"),
        "C351: Should contain btree_find_insert_pos function"
    );
}

#[test]
fn c352_lsm_tree_memtable_sorted_insert() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define MEMTABLE_MAX_ENTRIES 1024

typedef struct {
    uint32_t key;
    uint32_t value;
    int deleted;
} memtable_entry_t;

typedef struct {
    memtable_entry_t entries[MEMTABLE_MAX_ENTRIES];
    int count;
    int size_bytes;
    int max_size_bytes;
} memtable_t;

void memtable_init(memtable_t *mt, int max_size) {
    mt->count = 0;
    mt->size_bytes = 0;
    mt->max_size_bytes = max_size;
}

int memtable_find(const memtable_t *mt, uint32_t key) {
    int lo = 0;
    int hi = mt->count - 1;
    while (lo <= hi) {
        int mid = lo + (hi - lo) / 2;
        if (mt->entries[mid].key < key) {
            lo = mid + 1;
        } else if (mt->entries[mid].key > key) {
            hi = mid - 1;
        } else {
            return mid;
        }
    }
    return -(lo + 1);
}

int memtable_put(memtable_t *mt, uint32_t key, uint32_t value) {
    int idx = memtable_find(mt, key);
    if (idx >= 0) {
        mt->entries[idx].value = value;
        mt->entries[idx].deleted = 0;
        return 0;
    }
    if (mt->count >= MEMTABLE_MAX_ENTRIES) {
        return -1;
    }
    int insert_pos = -(idx + 1);
    int i;
    for (i = mt->count; i > insert_pos; i--) {
        mt->entries[i] = mt->entries[i - 1];
    }
    mt->entries[insert_pos].key = key;
    mt->entries[insert_pos].value = value;
    mt->entries[insert_pos].deleted = 0;
    mt->count++;
    mt->size_bytes += 12;
    return 0;
}

int memtable_get(const memtable_t *mt, uint32_t key, uint32_t *value) {
    int idx = memtable_find(mt, key);
    if (idx >= 0 && !mt->entries[idx].deleted) {
        *value = mt->entries[idx].value;
        return 1;
    }
    return 0;
}

int memtable_delete(memtable_t *mt, uint32_t key) {
    int idx = memtable_find(mt, key);
    if (idx >= 0) {
        mt->entries[idx].deleted = 1;
        return 0;
    }
    return -1;
}

int memtable_is_full(const memtable_t *mt) {
    return mt->size_bytes >= mt->max_size_bytes;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C352: LSM-tree memtable should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C352: Output should not be empty");
    assert!(
        code.contains("fn memtable_put"),
        "C352: Should contain memtable_put function"
    );
    assert!(
        code.contains("fn memtable_get"),
        "C352: Should contain memtable_get function"
    );
}

#[test]
fn c353_write_ahead_log_record_format() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

#define WAL_RECORD_INSERT 1
#define WAL_RECORD_UPDATE 2
#define WAL_RECORD_DELETE 3
#define WAL_RECORD_COMMIT 4
#define WAL_RECORD_ABORT  5

#define WAL_MAX_PAYLOAD 256

typedef struct {
    uint64_t lsn;
    uint64_t txn_id;
    uint32_t table_id;
    uint16_t record_type;
    uint16_t payload_len;
    uint32_t checksum;
    uint8_t payload[WAL_MAX_PAYLOAD];
} wal_record_t;

uint32_t wal_crc32(const uint8_t *data, int len) {
    uint32_t crc = 0xFFFFFFFF;
    int i;
    int j;
    for (i = 0; i < len; i++) {
        crc ^= data[i];
        for (j = 0; j < 8; j++) {
            if (crc & 1) {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc = crc >> 1;
            }
        }
    }
    return ~crc;
}

void wal_record_init(wal_record_t *rec, uint64_t lsn, uint64_t txn_id,
                     uint32_t table_id, uint16_t type) {
    rec->lsn = lsn;
    rec->txn_id = txn_id;
    rec->table_id = table_id;
    rec->record_type = type;
    rec->payload_len = 0;
    rec->checksum = 0;
}

int wal_record_set_payload(wal_record_t *rec, const uint8_t *data, uint16_t len) {
    if (len > WAL_MAX_PAYLOAD) {
        return -1;
    }
    int i;
    for (i = 0; i < len; i++) {
        rec->payload[i] = data[i];
    }
    rec->payload_len = len;
    rec->checksum = wal_crc32(rec->payload, len);
    return 0;
}

int wal_record_verify(const wal_record_t *rec) {
    uint32_t computed = wal_crc32(rec->payload, rec->payload_len);
    return computed == rec->checksum;
}

int wal_record_is_commit(const wal_record_t *rec) {
    return rec->record_type == WAL_RECORD_COMMIT;
}

int wal_record_is_data(const wal_record_t *rec) {
    return rec->record_type == WAL_RECORD_INSERT ||
           rec->record_type == WAL_RECORD_UPDATE ||
           rec->record_type == WAL_RECORD_DELETE;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C353: WAL record format should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C353: Output should not be empty");
    assert!(
        code.contains("fn wal_record_init"),
        "C353: Should contain wal_record_init function"
    );
    assert!(
        code.contains("fn wal_crc32"),
        "C353: Should contain wal_crc32 function"
    );
}

#[test]
fn c354_page_buffer_pool_lru_eviction() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define POOL_SIZE 16
#define PAGE_SIZE 4096

typedef struct {
    uint32_t page_id;
    int pin_count;
    int dirty;
    int valid;
    uint32_t lru_counter;
    unsigned char data[PAGE_SIZE];
} buffer_frame_t;

typedef struct {
    buffer_frame_t frames[POOL_SIZE];
    uint32_t clock_counter;
    int num_pages;
} buffer_pool_t;

void pool_init(buffer_pool_t *pool) {
    int i;
    for (i = 0; i < POOL_SIZE; i++) {
        pool->frames[i].page_id = 0;
        pool->frames[i].pin_count = 0;
        pool->frames[i].dirty = 0;
        pool->frames[i].valid = 0;
        pool->frames[i].lru_counter = 0;
    }
    pool->clock_counter = 0;
    pool->num_pages = 0;
}

int pool_find_page(const buffer_pool_t *pool, uint32_t page_id) {
    int i;
    for (i = 0; i < POOL_SIZE; i++) {
        if (pool->frames[i].valid && pool->frames[i].page_id == page_id) {
            return i;
        }
    }
    return -1;
}

int pool_find_victim_lru(const buffer_pool_t *pool) {
    int victim = -1;
    uint32_t min_counter = 0xFFFFFFFF;
    int i;
    for (i = 0; i < POOL_SIZE; i++) {
        if (!pool->frames[i].valid) {
            return i;
        }
        if (pool->frames[i].pin_count == 0 &&
            pool->frames[i].lru_counter < min_counter) {
            min_counter = pool->frames[i].lru_counter;
            victim = i;
        }
    }
    return victim;
}

int pool_pin_page(buffer_pool_t *pool, uint32_t page_id) {
    int idx = pool_find_page(pool, page_id);
    if (idx >= 0) {
        pool->frames[idx].pin_count++;
        pool->frames[idx].lru_counter = ++pool->clock_counter;
        return idx;
    }
    idx = pool_find_victim_lru(pool);
    if (idx < 0) {
        return -1;
    }
    pool->frames[idx].page_id = page_id;
    pool->frames[idx].pin_count = 1;
    pool->frames[idx].dirty = 0;
    pool->frames[idx].valid = 1;
    pool->frames[idx].lru_counter = ++pool->clock_counter;
    if (!pool->frames[idx].valid) {
        pool->num_pages++;
    }
    return idx;
}

void pool_unpin_page(buffer_pool_t *pool, int frame_idx, int is_dirty) {
    if (frame_idx >= 0 && frame_idx < POOL_SIZE) {
        if (pool->frames[frame_idx].pin_count > 0) {
            pool->frames[frame_idx].pin_count--;
        }
        if (is_dirty) {
            pool->frames[frame_idx].dirty = 1;
        }
    }
}

int pool_is_dirty(const buffer_pool_t *pool, int frame_idx) {
    if (frame_idx >= 0 && frame_idx < POOL_SIZE) {
        return pool->frames[frame_idx].dirty;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C354: Buffer pool LRU eviction should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C354: Output should not be empty");
    assert!(
        code.contains("fn pool_find_victim_lru"),
        "C354: Should contain pool_find_victim_lru function"
    );
    assert!(
        code.contains("fn pool_pin_page"),
        "C354: Should contain pool_pin_page function"
    );
}

#[test]
fn c355_row_oriented_tuple_layout() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

#define MAX_COLUMNS 16
#define TUPLE_MAX_SIZE 512

typedef struct {
    uint16_t col_offsets[MAX_COLUMNS];
    uint16_t col_lengths[MAX_COLUMNS];
    uint8_t col_types[MAX_COLUMNS];
    int num_cols;
} tuple_desc_t;

typedef struct {
    uint8_t data[TUPLE_MAX_SIZE];
    uint16_t size;
    uint32_t null_bitmap;
} tuple_t;

void tuple_desc_init(tuple_desc_t *desc) {
    desc->num_cols = 0;
}

int tuple_desc_add_col(tuple_desc_t *desc, uint8_t type, uint16_t length) {
    if (desc->num_cols >= MAX_COLUMNS) {
        return -1;
    }
    uint16_t offset = 0;
    if (desc->num_cols > 0) {
        int prev = desc->num_cols - 1;
        offset = desc->col_offsets[prev] + desc->col_lengths[prev];
    }
    desc->col_offsets[desc->num_cols] = offset;
    desc->col_lengths[desc->num_cols] = length;
    desc->col_types[desc->num_cols] = type;
    desc->num_cols++;
    return 0;
}

int tuple_get_col(const tuple_t *tuple, const tuple_desc_t *desc,
                  int col_idx, uint8_t *out, uint16_t *out_len) {
    if (col_idx < 0 || col_idx >= desc->num_cols) {
        return -1;
    }
    if (tuple->null_bitmap & (1u << col_idx)) {
        *out_len = 0;
        return 1;
    }
    uint16_t offset = desc->col_offsets[col_idx];
    uint16_t length = desc->col_lengths[col_idx];
    int i;
    for (i = 0; i < length && (offset + i) < tuple->size; i++) {
        out[i] = tuple->data[offset + i];
    }
    *out_len = (uint16_t)i;
    return 0;
}

int tuple_set_col(tuple_t *tuple, const tuple_desc_t *desc,
                  int col_idx, const uint8_t *data, uint16_t len) {
    if (col_idx < 0 || col_idx >= desc->num_cols) {
        return -1;
    }
    uint16_t offset = desc->col_offsets[col_idx];
    uint16_t max_len = desc->col_lengths[col_idx];
    uint16_t copy_len = len < max_len ? len : max_len;
    int i;
    for (i = 0; i < copy_len; i++) {
        tuple->data[offset + i] = data[i];
    }
    tuple->null_bitmap &= ~(1u << col_idx);
    uint16_t end = offset + max_len;
    if (end > tuple->size) {
        tuple->size = end;
    }
    return 0;
}

void tuple_set_null(tuple_t *tuple, int col_idx) {
    tuple->null_bitmap |= (1u << col_idx);
}

int tuple_is_null(const tuple_t *tuple, int col_idx) {
    return (tuple->null_bitmap & (1u << col_idx)) != 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C355: Row-oriented tuple layout should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C355: Output should not be empty");
    assert!(
        code.contains("fn tuple_get_col"),
        "C355: Should contain tuple_get_col function"
    );
    assert!(
        code.contains("fn tuple_set_col"),
        "C355: Should contain tuple_set_col function"
    );
}

// ============================================================================
// C356-C360: Indexing and Concurrency
// ============================================================================

#[test]
fn c356_column_oriented_rle_compression() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define RLE_MAX_RUNS 512

typedef struct {
    uint32_t value;
    uint32_t run_length;
} rle_run_t;

typedef struct {
    rle_run_t runs[RLE_MAX_RUNS];
    int num_runs;
    uint32_t total_values;
} rle_column_t;

void rle_column_init(rle_column_t *col) {
    col->num_runs = 0;
    col->total_values = 0;
}

int rle_column_append(rle_column_t *col, uint32_t value) {
    if (col->num_runs > 0 &&
        col->runs[col->num_runs - 1].value == value) {
        col->runs[col->num_runs - 1].run_length++;
        col->total_values++;
        return 0;
    }
    if (col->num_runs >= RLE_MAX_RUNS) {
        return -1;
    }
    col->runs[col->num_runs].value = value;
    col->runs[col->num_runs].run_length = 1;
    col->num_runs++;
    col->total_values++;
    return 0;
}

int rle_column_get(const rle_column_t *col, uint32_t index, uint32_t *value) {
    uint32_t pos = 0;
    int i;
    for (i = 0; i < col->num_runs; i++) {
        if (index < pos + col->runs[i].run_length) {
            *value = col->runs[i].value;
            return 0;
        }
        pos += col->runs[i].run_length;
    }
    return -1;
}

double rle_compression_ratio(const rle_column_t *col) {
    if (col->total_values == 0) {
        return 0.0;
    }
    return (double)col->num_runs / (double)col->total_values;
}

uint32_t rle_column_count_value(const rle_column_t *col, uint32_t value) {
    uint32_t count = 0;
    int i;
    for (i = 0; i < col->num_runs; i++) {
        if (col->runs[i].value == value) {
            count += col->runs[i].run_length;
        }
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C356: Column RLE compression should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C356: Output should not be empty");
    assert!(
        code.contains("fn rle_column_append"),
        "C356: Should contain rle_column_append function"
    );
    assert!(
        code.contains("fn rle_column_get"),
        "C356: Should contain rle_column_get function"
    );
}

#[test]
fn c357_hash_index_linear_probing() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define HASH_TABLE_SIZE 64
#define HASH_EMPTY 0xFFFFFFFF
#define HASH_DELETED 0xFFFFFFFE

typedef struct {
    uint32_t keys[HASH_TABLE_SIZE];
    uint32_t values[HASH_TABLE_SIZE];
    int count;
} hash_index_t;

void hash_index_init(hash_index_t *ht) {
    int i;
    for (i = 0; i < HASH_TABLE_SIZE; i++) {
        ht->keys[i] = HASH_EMPTY;
        ht->values[i] = 0;
    }
    ht->count = 0;
}

uint32_t hash_func(uint32_t key) {
    key = ((key >> 16) ^ key) * 0x45d9f3b;
    key = ((key >> 16) ^ key) * 0x45d9f3b;
    key = (key >> 16) ^ key;
    return key % HASH_TABLE_SIZE;
}

int hash_index_put(hash_index_t *ht, uint32_t key, uint32_t value) {
    if (ht->count >= HASH_TABLE_SIZE / 2) {
        return -1;
    }
    uint32_t idx = hash_func(key);
    int i;
    for (i = 0; i < HASH_TABLE_SIZE; i++) {
        uint32_t probe = (idx + i) % HASH_TABLE_SIZE;
        if (ht->keys[probe] == HASH_EMPTY || ht->keys[probe] == HASH_DELETED) {
            ht->keys[probe] = key;
            ht->values[probe] = value;
            ht->count++;
            return 0;
        }
        if (ht->keys[probe] == key) {
            ht->values[probe] = value;
            return 0;
        }
    }
    return -1;
}

int hash_index_get(const hash_index_t *ht, uint32_t key, uint32_t *value) {
    uint32_t idx = hash_func(key);
    int i;
    for (i = 0; i < HASH_TABLE_SIZE; i++) {
        uint32_t probe = (idx + i) % HASH_TABLE_SIZE;
        if (ht->keys[probe] == HASH_EMPTY) {
            return 0;
        }
        if (ht->keys[probe] == key) {
            *value = ht->values[probe];
            return 1;
        }
    }
    return 0;
}

int hash_index_delete(hash_index_t *ht, uint32_t key) {
    uint32_t idx = hash_func(key);
    int i;
    for (i = 0; i < HASH_TABLE_SIZE; i++) {
        uint32_t probe = (idx + i) % HASH_TABLE_SIZE;
        if (ht->keys[probe] == HASH_EMPTY) {
            return -1;
        }
        if (ht->keys[probe] == key) {
            ht->keys[probe] = HASH_DELETED;
            ht->count--;
            return 0;
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C357: Hash index linear probing should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C357: Output should not be empty");
    assert!(
        code.contains("fn hash_index_put"),
        "C357: Should contain hash_index_put function"
    );
    assert!(
        code.contains("fn hash_func"),
        "C357: Should contain hash_func function"
    );
}

#[test]
#[ignore = "FALSIFIED: transpiler panics with 'For loop must have condition' in HIR lowering for nested for loops with popcount pattern"]
fn c358_bloom_filter_bit_array() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BLOOM_BITS 256
#define BLOOM_BYTES (BLOOM_BITS / 8)
#define BLOOM_K 3

typedef struct {
    uint8_t bits[BLOOM_BYTES];
    int num_items;
} bloom_filter_t;

void bloom_init(bloom_filter_t *bf) {
    int i;
    for (i = 0; i < BLOOM_BYTES; i++) {
        bf->bits[i] = 0;
    }
    bf->num_items = 0;
}

uint32_t bloom_hash1(uint32_t key) {
    key = ((key >> 16) ^ key) * 0x45d9f3b;
    key = ((key >> 16) ^ key) * 0x45d9f3b;
    key = (key >> 16) ^ key;
    return key % BLOOM_BITS;
}

uint32_t bloom_hash2(uint32_t key) {
    key = (key ^ 0xDEADBEEF) * 0x119de1f3;
    key = ((key >> 16) ^ key) * 0x119de1f3;
    key = (key >> 16) ^ key;
    return key % BLOOM_BITS;
}

void bloom_set_bit(bloom_filter_t *bf, uint32_t bit) {
    bf->bits[bit / 8] |= (1u << (bit % 8));
}

int bloom_get_bit(const bloom_filter_t *bf, uint32_t bit) {
    return (bf->bits[bit / 8] >> (bit % 8)) & 1;
}

void bloom_add(bloom_filter_t *bf, uint32_t key) {
    uint32_t h1 = bloom_hash1(key);
    uint32_t h2 = bloom_hash2(key);
    int i;
    for (i = 0; i < BLOOM_K; i++) {
        uint32_t bit = (h1 + i * h2) % BLOOM_BITS;
        bloom_set_bit(bf, bit);
    }
    bf->num_items++;
}

int bloom_may_contain(const bloom_filter_t *bf, uint32_t key) {
    uint32_t h1 = bloom_hash1(key);
    uint32_t h2 = bloom_hash2(key);
    int i;
    for (i = 0; i < BLOOM_K; i++) {
        uint32_t bit = (h1 + i * h2) % BLOOM_BITS;
        if (!bloom_get_bit(bf, bit)) {
            return 0;
        }
    }
    return 1;
}

int bloom_count_set_bits(const bloom_filter_t *bf) {
    int count = 0;
    int i;
    int j;
    for (i = 0; i < BLOOM_BYTES; i++) {
        uint8_t byte = bf->bits[i];
        for (j = 0; j < 8; j++) {
            count += (byte >> j) & 1;
        }
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C358: Bloom filter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C358: Output should not be empty");
    assert!(
        code.contains("fn bloom_add"),
        "C358: Should contain bloom_add function"
    );
    assert!(
        code.contains("fn bloom_may_contain"),
        "C358: Should contain bloom_may_contain function"
    );
}

#[test]
fn c359_mvcc_version_chain() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

#define MVCC_MAX_VERSIONS 8
#define MVCC_MAX_ROWS 64

typedef struct {
    uint64_t txn_id;
    uint64_t begin_ts;
    uint64_t end_ts;
    uint32_t value;
    int valid;
} version_t;

typedef struct {
    uint32_t row_id;
    version_t versions[MVCC_MAX_VERSIONS];
    int num_versions;
} mvcc_row_t;

typedef struct {
    mvcc_row_t rows[MVCC_MAX_ROWS];
    int num_rows;
    uint64_t next_txn_id;
    uint64_t next_ts;
} mvcc_store_t;

void mvcc_init(mvcc_store_t *store) {
    store->num_rows = 0;
    store->next_txn_id = 1;
    store->next_ts = 1;
}

uint64_t mvcc_begin_txn(mvcc_store_t *store) {
    return store->next_txn_id++;
}

uint64_t mvcc_get_timestamp(mvcc_store_t *store) {
    return store->next_ts++;
}

int mvcc_find_row(const mvcc_store_t *store, uint32_t row_id) {
    int i;
    for (i = 0; i < store->num_rows; i++) {
        if (store->rows[i].row_id == row_id) {
            return i;
        }
    }
    return -1;
}

int mvcc_read(const mvcc_store_t *store, uint32_t row_id,
              uint64_t read_ts, uint32_t *value) {
    int ri = mvcc_find_row(store, row_id);
    if (ri < 0) return -1;
    const mvcc_row_t *row = &store->rows[ri];
    int best = -1;
    uint64_t best_ts = 0;
    int i;
    for (i = 0; i < row->num_versions; i++) {
        if (row->versions[i].valid &&
            row->versions[i].begin_ts <= read_ts &&
            (row->versions[i].end_ts == 0 || row->versions[i].end_ts > read_ts)) {
            if (row->versions[i].begin_ts > best_ts) {
                best_ts = row->versions[i].begin_ts;
                best = i;
            }
        }
    }
    if (best >= 0) {
        *value = row->versions[best].value;
        return 0;
    }
    return -1;
}

int mvcc_write(mvcc_store_t *store, uint32_t row_id,
               uint64_t txn_id, uint64_t ts, uint32_t value) {
    int ri = mvcc_find_row(store, row_id);
    if (ri < 0) {
        if (store->num_rows >= MVCC_MAX_ROWS) return -1;
        ri = store->num_rows++;
        store->rows[ri].row_id = row_id;
        store->rows[ri].num_versions = 0;
    }
    mvcc_row_t *row = &store->rows[ri];
    if (row->num_versions >= MVCC_MAX_VERSIONS) return -1;
    int vi = row->num_versions++;
    row->versions[vi].txn_id = txn_id;
    row->versions[vi].begin_ts = ts;
    row->versions[vi].end_ts = 0;
    row->versions[vi].value = value;
    row->versions[vi].valid = 1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C359: MVCC version chain should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C359: Output should not be empty");
    assert!(
        code.contains("fn mvcc_read"),
        "C359: Should contain mvcc_read function"
    );
    assert!(
        code.contains("fn mvcc_write"),
        "C359: Should contain mvcc_write function"
    );
}

#[test]
fn c360_lock_manager_deadlock_detection() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

#define MAX_LOCKS 32
#define MAX_TXNS 16

#define LOCK_SHARED    1
#define LOCK_EXCLUSIVE 2

typedef struct {
    uint32_t resource_id;
    uint64_t txn_id;
    int lock_mode;
    int granted;
} lock_entry_t;

typedef struct {
    lock_entry_t locks[MAX_LOCKS];
    int num_locks;
    int wait_for[MAX_TXNS][MAX_TXNS];
    int num_txns;
} lock_manager_t;

void lock_mgr_init(lock_manager_t *lm) {
    lm->num_locks = 0;
    lm->num_txns = 0;
    int i;
    int j;
    for (i = 0; i < MAX_TXNS; i++) {
        for (j = 0; j < MAX_TXNS; j++) {
            lm->wait_for[i][j] = 0;
        }
    }
}

int lock_mgr_find_txn_idx(lock_manager_t *lm, uint64_t txn_id) {
    int i;
    for (i = 0; i < lm->num_txns; i++) {
        if (lm->locks[i].txn_id == txn_id) {
            return i;
        }
    }
    return -1;
}

int lock_mgr_is_compatible(int held_mode, int requested_mode) {
    if (held_mode == LOCK_SHARED && requested_mode == LOCK_SHARED) {
        return 1;
    }
    return 0;
}

int lock_mgr_has_conflict(const lock_manager_t *lm, uint32_t resource_id,
                          uint64_t txn_id, int mode) {
    int i;
    for (i = 0; i < lm->num_locks; i++) {
        if (lm->locks[i].resource_id == resource_id &&
            lm->locks[i].txn_id != txn_id &&
            lm->locks[i].granted &&
            !lock_mgr_is_compatible(lm->locks[i].lock_mode, mode)) {
            return 1;
        }
    }
    return 0;
}

static int deadlock_dfs(const lock_manager_t *lm, int node,
                        int *visited, int *in_stack) {
    visited[node] = 1;
    in_stack[node] = 1;
    int i;
    for (i = 0; i < lm->num_txns; i++) {
        if (lm->wait_for[node][i]) {
            if (!visited[i]) {
                if (deadlock_dfs(lm, i, visited, in_stack)) {
                    return 1;
                }
            } else if (in_stack[i]) {
                return 1;
            }
        }
    }
    in_stack[node] = 0;
    return 0;
}

int lock_mgr_detect_deadlock(const lock_manager_t *lm) {
    int visited[MAX_TXNS];
    int in_stack[MAX_TXNS];
    int i;
    for (i = 0; i < MAX_TXNS; i++) {
        visited[i] = 0;
        in_stack[i] = 0;
    }
    for (i = 0; i < lm->num_txns; i++) {
        if (!visited[i]) {
            if (deadlock_dfs(lm, i, visited, in_stack)) {
                return 1;
            }
        }
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C360: Lock manager deadlock detection should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C360: Output should not be empty");
    assert!(
        code.contains("fn lock_mgr_detect_deadlock"),
        "C360: Should contain lock_mgr_detect_deadlock function"
    );
    assert!(
        code.contains("fn lock_mgr_has_conflict"),
        "C360: Should contain lock_mgr_has_conflict function"
    );
}

// ============================================================================
// C361-C365: Query Execution
// ============================================================================

#[test]
fn c361_query_plan_cost_estimator() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define PLAN_SEQ_SCAN   1
#define PLAN_INDEX_SCAN 2
#define PLAN_HASH_JOIN  3
#define PLAN_MERGE_JOIN 4
#define PLAN_SORT       5

typedef struct {
    int plan_type;
    uint32_t estimated_rows;
    uint32_t num_pages;
    double selectivity;
    double io_cost;
    double cpu_cost;
    double total_cost;
} plan_node_t;

double cost_seq_scan(uint32_t num_pages, uint32_t num_rows, double selectivity) {
    double io = (double)num_pages * 1.0;
    double cpu = (double)num_rows * 0.01;
    return io + cpu;
}

double cost_index_scan(uint32_t num_rows, double selectivity, uint32_t index_height) {
    double matching_rows = (double)num_rows * selectivity;
    double io = (double)index_height + matching_rows * 1.0;
    double cpu = matching_rows * 0.005;
    return io + cpu;
}

double cost_hash_join(double left_cost, double right_cost,
                      uint32_t left_rows, uint32_t right_rows) {
    double build = (double)left_rows * 0.02;
    double probe = (double)right_rows * 0.01;
    return left_cost + right_cost + build + probe;
}

double cost_merge_join(double left_cost, double right_cost,
                       uint32_t left_rows, uint32_t right_rows,
                       int left_sorted, int right_sorted) {
    double sort_left = left_sorted ? 0.0 : (double)left_rows * 0.05;
    double sort_right = right_sorted ? 0.0 : (double)right_rows * 0.05;
    double merge = ((double)left_rows + (double)right_rows) * 0.005;
    return left_cost + right_cost + sort_left + sort_right + merge;
}

int choose_join_method(uint32_t left_rows, uint32_t right_rows,
                       int left_sorted, int right_sorted) {
    double hash = cost_hash_join(0, 0, left_rows, right_rows);
    double merge = cost_merge_join(0, 0, left_rows, right_rows,
                                   left_sorted, right_sorted);
    if (merge < hash) {
        return PLAN_MERGE_JOIN;
    }
    return PLAN_HASH_JOIN;
}

int choose_scan_method(uint32_t num_pages, uint32_t num_rows,
                       double selectivity, uint32_t index_height) {
    double seq = cost_seq_scan(num_pages, num_rows, selectivity);
    double idx = cost_index_scan(num_rows, selectivity, index_height);
    if (idx < seq) {
        return PLAN_INDEX_SCAN;
    }
    return PLAN_SEQ_SCAN;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C361: Query plan cost estimator should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C361: Output should not be empty");
    assert!(
        code.contains("fn cost_seq_scan"),
        "C361: Should contain cost_seq_scan function"
    );
    assert!(
        code.contains("fn choose_join_method"),
        "C361: Should contain choose_join_method function"
    );
}

#[test]
fn c362_sort_merge_join() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define JOIN_MAX_ROWS 128

typedef struct {
    uint32_t key;
    uint32_t value;
} join_row_t;

typedef struct {
    join_row_t rows[JOIN_MAX_ROWS];
    int count;
} join_result_t;

void join_result_init(join_result_t *res) {
    res->count = 0;
}

int join_result_add(join_result_t *res, uint32_t left_val, uint32_t right_val) {
    if (res->count >= JOIN_MAX_ROWS) {
        return -1;
    }
    res->rows[res->count].key = left_val;
    res->rows[res->count].value = right_val;
    res->count++;
    return 0;
}

void sort_rows(join_row_t *rows, int n) {
    int i;
    int j;
    for (i = 0; i < n - 1; i++) {
        for (j = 0; j < n - i - 1; j++) {
            if (rows[j].key > rows[j + 1].key) {
                join_row_t tmp = rows[j];
                rows[j] = rows[j + 1];
                rows[j + 1] = tmp;
            }
        }
    }
}

int sort_merge_join(const join_row_t *left, int left_n,
                    const join_row_t *right, int right_n,
                    join_result_t *result) {
    int li = 0;
    int ri = 0;
    join_result_init(result);
    while (li < left_n && ri < right_n) {
        if (left[li].key < right[ri].key) {
            li++;
        } else if (left[li].key > right[ri].key) {
            ri++;
        } else {
            int mark = ri;
            while (ri < right_n && right[ri].key == left[li].key) {
                if (join_result_add(result, left[li].value, right[ri].value) < 0) {
                    return -1;
                }
                ri++;
            }
            li++;
            if (li < left_n && left[li].key == left[li - 1].key) {
                ri = mark;
            }
        }
    }
    return result->count;
}

int count_matching_keys(const join_row_t *left, int left_n,
                        const join_row_t *right, int right_n) {
    int count = 0;
    int li = 0;
    int ri = 0;
    while (li < left_n && ri < right_n) {
        if (left[li].key < right[ri].key) {
            li++;
        } else if (left[li].key > right[ri].key) {
            ri++;
        } else {
            count++;
            li++;
            ri++;
        }
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C362: Sort-merge join should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C362: Output should not be empty");
    assert!(
        code.contains("fn sort_merge_join"),
        "C362: Should contain sort_merge_join function"
    );
    assert!(
        code.contains("fn sort_rows"),
        "C362: Should contain sort_rows function"
    );
}

#[test]
fn c363_index_scan_iterator() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define INDEX_MAX_ENTRIES 256

typedef struct {
    uint32_t key;
    uint32_t row_id;
} index_entry_t;

typedef struct {
    index_entry_t entries[INDEX_MAX_ENTRIES];
    int num_entries;
} sorted_index_t;

typedef struct {
    const sorted_index_t *index;
    int current_pos;
    uint32_t lower_bound;
    uint32_t upper_bound;
    int exhausted;
} index_scan_iter_t;

void index_build(sorted_index_t *idx) {
    int i;
    int j;
    for (i = 0; i < idx->num_entries - 1; i++) {
        for (j = 0; j < idx->num_entries - i - 1; j++) {
            if (idx->entries[j].key > idx->entries[j + 1].key) {
                index_entry_t tmp = idx->entries[j];
                idx->entries[j] = idx->entries[j + 1];
                idx->entries[j + 1] = tmp;
            }
        }
    }
}

int index_lower_bound(const sorted_index_t *idx, uint32_t key) {
    int lo = 0;
    int hi = idx->num_entries;
    while (lo < hi) {
        int mid = lo + (hi - lo) / 2;
        if (idx->entries[mid].key < key) {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    return lo;
}

void iter_init(index_scan_iter_t *iter, const sorted_index_t *idx,
               uint32_t lower, uint32_t upper) {
    iter->index = idx;
    iter->lower_bound = lower;
    iter->upper_bound = upper;
    iter->current_pos = index_lower_bound(idx, lower);
    iter->exhausted = 0;
}

int iter_next(index_scan_iter_t *iter, uint32_t *row_id) {
    if (iter->exhausted) {
        return 0;
    }
    if (iter->current_pos >= iter->index->num_entries) {
        iter->exhausted = 1;
        return 0;
    }
    uint32_t key = iter->index->entries[iter->current_pos].key;
    if (key > iter->upper_bound) {
        iter->exhausted = 1;
        return 0;
    }
    *row_id = iter->index->entries[iter->current_pos].row_id;
    iter->current_pos++;
    return 1;
}

int iter_count_remaining(index_scan_iter_t *iter) {
    int count = 0;
    int pos = iter->current_pos;
    while (pos < iter->index->num_entries &&
           iter->index->entries[pos].key <= iter->upper_bound) {
        count++;
        pos++;
    }
    return count;
}

void iter_reset(index_scan_iter_t *iter) {
    iter->current_pos = index_lower_bound(iter->index, iter->lower_bound);
    iter->exhausted = 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C363: Index scan iterator should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C363: Output should not be empty");
    assert!(
        code.contains("fn iter_init"),
        "C363: Should contain iter_init function"
    );
    assert!(
        code.contains("fn iter_next"),
        "C363: Should contain iter_next function"
    );
}

#[test]
fn c364_checkpoint_manager() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

#define CKPT_MAX_DIRTY 64

typedef struct {
    uint32_t page_id;
    uint64_t lsn;
} dirty_page_entry_t;

typedef struct {
    uint64_t checkpoint_lsn;
    uint64_t oldest_active_txn;
    dirty_page_entry_t dirty_pages[CKPT_MAX_DIRTY];
    int num_dirty;
    int checkpoint_active;
} checkpoint_mgr_t;

void ckpt_init(checkpoint_mgr_t *mgr) {
    mgr->checkpoint_lsn = 0;
    mgr->oldest_active_txn = 0;
    mgr->num_dirty = 0;
    mgr->checkpoint_active = 0;
}

int ckpt_add_dirty_page(checkpoint_mgr_t *mgr, uint32_t page_id, uint64_t lsn) {
    int i;
    for (i = 0; i < mgr->num_dirty; i++) {
        if (mgr->dirty_pages[i].page_id == page_id) {
            if (lsn > mgr->dirty_pages[i].lsn) {
                mgr->dirty_pages[i].lsn = lsn;
            }
            return 0;
        }
    }
    if (mgr->num_dirty >= CKPT_MAX_DIRTY) {
        return -1;
    }
    mgr->dirty_pages[mgr->num_dirty].page_id = page_id;
    mgr->dirty_pages[mgr->num_dirty].lsn = lsn;
    mgr->num_dirty++;
    return 0;
}

void ckpt_remove_dirty_page(checkpoint_mgr_t *mgr, uint32_t page_id) {
    int i;
    for (i = 0; i < mgr->num_dirty; i++) {
        if (mgr->dirty_pages[i].page_id == page_id) {
            mgr->dirty_pages[i] = mgr->dirty_pages[mgr->num_dirty - 1];
            mgr->num_dirty--;
            return;
        }
    }
}

uint64_t ckpt_get_min_recovery_lsn(const checkpoint_mgr_t *mgr) {
    if (mgr->num_dirty == 0) {
        return mgr->checkpoint_lsn;
    }
    uint64_t min_lsn = mgr->dirty_pages[0].lsn;
    int i;
    for (i = 1; i < mgr->num_dirty; i++) {
        if (mgr->dirty_pages[i].lsn < min_lsn) {
            min_lsn = mgr->dirty_pages[i].lsn;
        }
    }
    return min_lsn;
}

int ckpt_begin(checkpoint_mgr_t *mgr, uint64_t current_lsn) {
    if (mgr->checkpoint_active) {
        return -1;
    }
    mgr->checkpoint_active = 1;
    mgr->checkpoint_lsn = current_lsn;
    return 0;
}

int ckpt_end(checkpoint_mgr_t *mgr) {
    if (!mgr->checkpoint_active) {
        return -1;
    }
    mgr->checkpoint_active = 0;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C364: Checkpoint manager should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C364: Output should not be empty");
    assert!(
        code.contains("fn ckpt_begin"),
        "C364: Should contain ckpt_begin function"
    );
    assert!(
        code.contains("fn ckpt_get_min_recovery_lsn"),
        "C364: Should contain ckpt_get_min_recovery_lsn function"
    );
}

#[test]
fn c365_redo_log_replay() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

#define REDO_MAX_RECORDS 128
#define REDO_SET_VALUE   1
#define REDO_INCREMENT   2
#define REDO_CLEAR       3

typedef struct {
    uint64_t lsn;
    uint32_t page_id;
    uint16_t offset;
    uint8_t op_type;
    uint32_t old_value;
    uint32_t new_value;
} redo_record_t;

typedef struct {
    redo_record_t records[REDO_MAX_RECORDS];
    int count;
    uint64_t last_applied_lsn;
} redo_log_t;

void redo_log_init(redo_log_t *log) {
    log->count = 0;
    log->last_applied_lsn = 0;
}

int redo_log_append(redo_log_t *log, uint64_t lsn, uint32_t page_id,
                    uint16_t offset, uint8_t op_type,
                    uint32_t old_val, uint32_t new_val) {
    if (log->count >= REDO_MAX_RECORDS) {
        return -1;
    }
    redo_record_t *rec = &log->records[log->count];
    rec->lsn = lsn;
    rec->page_id = page_id;
    rec->offset = offset;
    rec->op_type = op_type;
    rec->old_value = old_val;
    rec->new_value = new_val;
    log->count++;
    return 0;
}

uint32_t redo_apply_op(uint32_t current, uint8_t op_type, uint32_t new_value) {
    if (op_type == REDO_SET_VALUE) {
        return new_value;
    } else if (op_type == REDO_INCREMENT) {
        return current + new_value;
    } else if (op_type == REDO_CLEAR) {
        return 0;
    }
    return current;
}

int redo_log_replay(const redo_log_t *log, uint64_t from_lsn,
                    uint32_t *page_data, uint32_t page_id, int page_slots) {
    int applied = 0;
    int i;
    for (i = 0; i < log->count; i++) {
        const redo_record_t *rec = &log->records[i];
        if (rec->lsn <= from_lsn) {
            continue;
        }
        if (rec->page_id != page_id) {
            continue;
        }
        if (rec->offset < page_slots) {
            page_data[rec->offset] = redo_apply_op(
                page_data[rec->offset], rec->op_type, rec->new_value);
            applied++;
        }
    }
    return applied;
}

int redo_log_count_for_page(const redo_log_t *log, uint32_t page_id) {
    int count = 0;
    int i;
    for (i = 0; i < log->count; i++) {
        if (log->records[i].page_id == page_id) {
            count++;
        }
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C365: Redo log replay should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C365: Output should not be empty");
    assert!(
        code.contains("fn redo_log_replay"),
        "C365: Should contain redo_log_replay function"
    );
    assert!(
        code.contains("fn redo_apply_op"),
        "C365: Should contain redo_apply_op function"
    );
}

// ============================================================================
// C366-C370: Storage Management
// ============================================================================

#[test]
fn c366_bitmap_index_operations() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BITMAP_MAX_WORDS 32
#define BITS_PER_WORD 32

typedef struct {
    uint32_t words[BITMAP_MAX_WORDS];
    int num_words;
    int num_rows;
} bitmap_index_t;

void bitmap_init(bitmap_index_t *bm, int num_rows) {
    bm->num_rows = num_rows;
    bm->num_words = (num_rows + BITS_PER_WORD - 1) / BITS_PER_WORD;
    int i;
    for (i = 0; i < BITMAP_MAX_WORDS; i++) {
        bm->words[i] = 0;
    }
}

void bitmap_set(bitmap_index_t *bm, int row) {
    if (row >= 0 && row < bm->num_rows) {
        bm->words[row / BITS_PER_WORD] |= (1u << (row % BITS_PER_WORD));
    }
}

int bitmap_test(const bitmap_index_t *bm, int row) {
    if (row >= 0 && row < bm->num_rows) {
        return (bm->words[row / BITS_PER_WORD] >> (row % BITS_PER_WORD)) & 1;
    }
    return 0;
}

void bitmap_and(bitmap_index_t *result, const bitmap_index_t *a,
                const bitmap_index_t *b) {
    int words = a->num_words < b->num_words ? a->num_words : b->num_words;
    result->num_words = words;
    result->num_rows = a->num_rows < b->num_rows ? a->num_rows : b->num_rows;
    int i;
    for (i = 0; i < words; i++) {
        result->words[i] = a->words[i] & b->words[i];
    }
}

void bitmap_or(bitmap_index_t *result, const bitmap_index_t *a,
               const bitmap_index_t *b) {
    int words = a->num_words > b->num_words ? a->num_words : b->num_words;
    result->num_words = words;
    result->num_rows = a->num_rows > b->num_rows ? a->num_rows : b->num_rows;
    int i;
    for (i = 0; i < words; i++) {
        uint32_t va = i < a->num_words ? a->words[i] : 0;
        uint32_t vb = i < b->num_words ? b->words[i] : 0;
        result->words[i] = va | vb;
    }
}

int bitmap_popcount(const bitmap_index_t *bm) {
    int count = 0;
    int i;
    for (i = 0; i < bm->num_words; i++) {
        uint32_t w = bm->words[i];
        while (w) {
            count += w & 1;
            w >>= 1;
        }
    }
    return count;
}

void bitmap_not(bitmap_index_t *result, const bitmap_index_t *bm) {
    result->num_words = bm->num_words;
    result->num_rows = bm->num_rows;
    int i;
    for (i = 0; i < bm->num_words; i++) {
        result->words[i] = ~bm->words[i];
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C366: Bitmap index operations should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C366: Output should not be empty");
    assert!(
        code.contains("fn bitmap_and"),
        "C366: Should contain bitmap_and function"
    );
    assert!(
        code.contains("fn bitmap_popcount"),
        "C366: Should contain bitmap_popcount function"
    );
}

#[test]
fn c367_extent_allocator_free_space_management() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define MAX_EXTENTS 64
#define EXTENT_FREE 0
#define EXTENT_USED 1

typedef struct {
    uint32_t start_page;
    uint32_t num_pages;
    int status;
} extent_t;

typedef struct {
    extent_t extents[MAX_EXTENTS];
    int num_extents;
    uint32_t total_pages;
    uint32_t free_pages;
} extent_allocator_t;

void extent_alloc_init(extent_allocator_t *ea, uint32_t total_pages) {
    ea->num_extents = 1;
    ea->extents[0].start_page = 0;
    ea->extents[0].num_pages = total_pages;
    ea->extents[0].status = EXTENT_FREE;
    ea->total_pages = total_pages;
    ea->free_pages = total_pages;
}

int extent_alloc_find_first_fit(const extent_allocator_t *ea, uint32_t num_pages) {
    int i;
    for (i = 0; i < ea->num_extents; i++) {
        if (ea->extents[i].status == EXTENT_FREE &&
            ea->extents[i].num_pages >= num_pages) {
            return i;
        }
    }
    return -1;
}

int extent_allocate(extent_allocator_t *ea, uint32_t num_pages, uint32_t *start) {
    int idx = extent_alloc_find_first_fit(ea, num_pages);
    if (idx < 0) {
        return -1;
    }
    *start = ea->extents[idx].start_page;
    if (ea->extents[idx].num_pages == num_pages) {
        ea->extents[idx].status = EXTENT_USED;
    } else {
        if (ea->num_extents >= MAX_EXTENTS) {
            return -1;
        }
        int i;
        for (i = ea->num_extents; i > idx + 1; i--) {
            ea->extents[i] = ea->extents[i - 1];
        }
        ea->extents[idx + 1].start_page = ea->extents[idx].start_page + num_pages;
        ea->extents[idx + 1].num_pages = ea->extents[idx].num_pages - num_pages;
        ea->extents[idx + 1].status = EXTENT_FREE;
        ea->extents[idx].num_pages = num_pages;
        ea->extents[idx].status = EXTENT_USED;
        ea->num_extents++;
    }
    ea->free_pages -= num_pages;
    return 0;
}

void extent_coalesce(extent_allocator_t *ea) {
    int i = 0;
    while (i < ea->num_extents - 1) {
        if (ea->extents[i].status == EXTENT_FREE &&
            ea->extents[i + 1].status == EXTENT_FREE) {
            ea->extents[i].num_pages += ea->extents[i + 1].num_pages;
            int j;
            for (j = i + 1; j < ea->num_extents - 1; j++) {
                ea->extents[j] = ea->extents[j + 1];
            }
            ea->num_extents--;
        } else {
            i++;
        }
    }
}

int extent_free(extent_allocator_t *ea, uint32_t start_page) {
    int i;
    for (i = 0; i < ea->num_extents; i++) {
        if (ea->extents[i].start_page == start_page &&
            ea->extents[i].status == EXTENT_USED) {
            ea->extents[i].status = EXTENT_FREE;
            ea->free_pages += ea->extents[i].num_pages;
            extent_coalesce(ea);
            return 0;
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C367: Extent allocator should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C367: Output should not be empty");
    assert!(
        code.contains("fn extent_allocate"),
        "C367: Should contain extent_allocate function"
    );
    assert!(
        code.contains("fn extent_coalesce"),
        "C367: Should contain extent_coalesce function"
    );
}

#[test]
fn c368_record_serialization_variable_length() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

#define RECORD_MAX_SIZE 512
#define FIELD_INT32   1
#define FIELD_VARCHAR 2
#define FIELD_BOOL    3

typedef struct {
    uint8_t field_type;
    uint16_t field_length;
} field_header_t;

int serialize_int32(uint8_t *buf, int buf_size, int *offset, uint32_t value) {
    if (*offset + 3 + 4 > buf_size) return -1;
    buf[*offset] = FIELD_INT32;
    buf[*offset + 1] = 0;
    buf[*offset + 2] = 4;
    buf[*offset + 3] = (uint8_t)(value >> 24);
    buf[*offset + 4] = (uint8_t)(value >> 16);
    buf[*offset + 5] = (uint8_t)(value >> 8);
    buf[*offset + 6] = (uint8_t)(value);
    *offset += 7;
    return 0;
}

int serialize_varchar(uint8_t *buf, int buf_size, int *offset,
                      const uint8_t *str, uint16_t len) {
    if (*offset + 3 + len > buf_size) return -1;
    buf[*offset] = FIELD_VARCHAR;
    buf[*offset + 1] = (uint8_t)(len >> 8);
    buf[*offset + 2] = (uint8_t)(len);
    int i;
    for (i = 0; i < len; i++) {
        buf[*offset + 3 + i] = str[i];
    }
    *offset += 3 + len;
    return 0;
}

int serialize_bool(uint8_t *buf, int buf_size, int *offset, int value) {
    if (*offset + 3 + 1 > buf_size) return -1;
    buf[*offset] = FIELD_BOOL;
    buf[*offset + 1] = 0;
    buf[*offset + 2] = 1;
    buf[*offset + 3] = value ? 1 : 0;
    *offset += 4;
    return 0;
}

int deserialize_field_type(const uint8_t *buf, int buf_size, int offset) {
    if (offset >= buf_size) return -1;
    return buf[offset];
}

uint16_t deserialize_field_length(const uint8_t *buf, int offset) {
    return ((uint16_t)buf[offset + 1] << 8) | (uint16_t)buf[offset + 2];
}

uint32_t deserialize_int32(const uint8_t *buf, int offset) {
    int data_off = offset + 3;
    return ((uint32_t)buf[data_off] << 24) |
           ((uint32_t)buf[data_off + 1] << 16) |
           ((uint32_t)buf[data_off + 2] << 8) |
           (uint32_t)buf[data_off + 3];
}

int count_fields(const uint8_t *buf, int buf_size) {
    int count = 0;
    int offset = 0;
    while (offset < buf_size) {
        uint16_t len = deserialize_field_length(buf, offset);
        offset += 3 + len;
        count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C368: Record serialization should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C368: Output should not be empty");
    assert!(
        code.contains("fn serialize_int32"),
        "C368: Should contain serialize_int32 function"
    );
    assert!(
        code.contains("fn deserialize_int32"),
        "C368: Should contain deserialize_int32 function"
    );
}

#[test]
fn c369_bplus_tree_leaf_page_layout() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;

#define LEAF_MAX_KEYS 32

typedef struct {
    uint32_t keys[LEAF_MAX_KEYS];
    uint32_t row_ids[LEAF_MAX_KEYS];
    int num_keys;
    uint32_t next_leaf;
    uint32_t prev_leaf;
    uint32_t page_id;
} leaf_page_t;

void leaf_init(leaf_page_t *page, uint32_t page_id) {
    page->num_keys = 0;
    page->next_leaf = 0;
    page->prev_leaf = 0;
    page->page_id = page_id;
}

int leaf_find_key(const leaf_page_t *page, uint32_t key) {
    int lo = 0;
    int hi = page->num_keys - 1;
    while (lo <= hi) {
        int mid = lo + (hi - lo) / 2;
        if (page->keys[mid] == key) return mid;
        if (page->keys[mid] < key) lo = mid + 1;
        else hi = mid - 1;
    }
    return -(lo + 1);
}

int leaf_insert(leaf_page_t *page, uint32_t key, uint32_t row_id) {
    if (page->num_keys >= LEAF_MAX_KEYS) return -1;
    int idx = leaf_find_key(page, key);
    int insert_pos;
    if (idx >= 0) {
        page->row_ids[idx] = row_id;
        return 0;
    }
    insert_pos = -(idx + 1);
    int i;
    for (i = page->num_keys; i > insert_pos; i--) {
        page->keys[i] = page->keys[i - 1];
        page->row_ids[i] = page->row_ids[i - 1];
    }
    page->keys[insert_pos] = key;
    page->row_ids[insert_pos] = row_id;
    page->num_keys++;
    return 0;
}

int leaf_delete(leaf_page_t *page, uint32_t key) {
    int idx = leaf_find_key(page, key);
    if (idx < 0) return -1;
    int i;
    for (i = idx; i < page->num_keys - 1; i++) {
        page->keys[i] = page->keys[i + 1];
        page->row_ids[i] = page->row_ids[i + 1];
    }
    page->num_keys--;
    return 0;
}

uint32_t leaf_split(leaf_page_t *left, leaf_page_t *right) {
    int mid = left->num_keys / 2;
    int j = 0;
    int i;
    for (i = mid; i < left->num_keys; i++) {
        right->keys[j] = left->keys[i];
        right->row_ids[j] = left->row_ids[i];
        j++;
    }
    right->num_keys = j;
    left->num_keys = mid;
    right->next_leaf = left->next_leaf;
    right->prev_leaf = left->page_id;
    left->next_leaf = right->page_id;
    return right->keys[0];
}

int leaf_is_underflow(const leaf_page_t *page) {
    return page->num_keys < LEAF_MAX_KEYS / 2;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C369: B+ tree leaf page should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C369: Output should not be empty");
    assert!(
        code.contains("fn leaf_insert"),
        "C369: Should contain leaf_insert function"
    );
    assert!(
        code.contains("fn leaf_split"),
        "C369: Should contain leaf_split function"
    );
}

#[test]
fn c370_lsm_compaction() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define SSTABLE_MAX_ENTRIES 64
#define MAX_LEVELS 4

typedef struct {
    uint32_t key;
    uint32_t value;
    uint32_t sequence;
    int deleted;
} kv_entry_t;

typedef struct {
    kv_entry_t entries[SSTABLE_MAX_ENTRIES];
    int count;
    uint32_t min_key;
    uint32_t max_key;
    int level;
} sstable_t;

void sstable_init(sstable_t *sst, int level) {
    sst->count = 0;
    sst->min_key = 0xFFFFFFFF;
    sst->max_key = 0;
    sst->level = level;
}

int sstable_add(sstable_t *sst, uint32_t key, uint32_t value,
                uint32_t seq, int deleted) {
    if (sst->count >= SSTABLE_MAX_ENTRIES) return -1;
    sst->entries[sst->count].key = key;
    sst->entries[sst->count].value = value;
    sst->entries[sst->count].sequence = seq;
    sst->entries[sst->count].deleted = deleted;
    sst->count++;
    if (key < sst->min_key) sst->min_key = key;
    if (key > sst->max_key) sst->max_key = key;
    return 0;
}

int sstable_overlaps(const sstable_t *a, const sstable_t *b) {
    return a->min_key <= b->max_key && a->max_key >= b->min_key;
}

int compact_merge(const sstable_t *upper, const sstable_t *lower,
                  sstable_t *output) {
    int ui = 0;
    int li = 0;
    sstable_init(output, lower->level);
    while (ui < upper->count && li < lower->count) {
        if (upper->entries[ui].key < lower->entries[li].key) {
            if (!upper->entries[ui].deleted) {
                if (sstable_add(output, upper->entries[ui].key,
                                upper->entries[ui].value,
                                upper->entries[ui].sequence, 0) < 0) return -1;
            }
            ui++;
        } else if (upper->entries[ui].key > lower->entries[li].key) {
            if (!lower->entries[li].deleted) {
                if (sstable_add(output, lower->entries[li].key,
                                lower->entries[li].value,
                                lower->entries[li].sequence, 0) < 0) return -1;
            }
            li++;
        } else {
            if (!upper->entries[ui].deleted) {
                if (sstable_add(output, upper->entries[ui].key,
                                upper->entries[ui].value,
                                upper->entries[ui].sequence, 0) < 0) return -1;
            }
            ui++;
            li++;
        }
    }
    while (ui < upper->count) {
        if (!upper->entries[ui].deleted) {
            if (sstable_add(output, upper->entries[ui].key,
                            upper->entries[ui].value,
                            upper->entries[ui].sequence, 0) < 0) return -1;
        }
        ui++;
    }
    while (li < lower->count) {
        if (!lower->entries[li].deleted) {
            if (sstable_add(output, lower->entries[li].key,
                            lower->entries[li].value,
                            lower->entries[li].sequence, 0) < 0) return -1;
        }
        li++;
    }
    return output->count;
}

int sstable_lookup(const sstable_t *sst, uint32_t key, uint32_t *value) {
    int lo = 0;
    int hi = sst->count - 1;
    while (lo <= hi) {
        int mid = lo + (hi - lo) / 2;
        if (sst->entries[mid].key == key) {
            if (sst->entries[mid].deleted) return -1;
            *value = sst->entries[mid].value;
            return 0;
        }
        if (sst->entries[mid].key < key) lo = mid + 1;
        else hi = mid - 1;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C370: LSM compaction should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C370: Output should not be empty");
    assert!(
        code.contains("fn compact_merge"),
        "C370: Should contain compact_merge function"
    );
    assert!(
        code.contains("fn sstable_lookup"),
        "C370: Should contain sstable_lookup function"
    );
}

// ============================================================================
// C371-C375: Catalog and Recovery
// ============================================================================

#[test]
fn c371_cursor_based_iteration() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define CURSOR_MAX_ROWS 128

typedef struct {
    uint32_t key;
    uint32_t value;
} row_t;

typedef struct {
    row_t rows[CURSOR_MAX_ROWS];
    int num_rows;
} table_t;

typedef struct {
    const table_t *table;
    int position;
    int direction;
    int valid;
} cursor_t;

void table_init(table_t *tbl) {
    tbl->num_rows = 0;
}

int table_insert(table_t *tbl, uint32_t key, uint32_t value) {
    if (tbl->num_rows >= CURSOR_MAX_ROWS) return -1;
    tbl->rows[tbl->num_rows].key = key;
    tbl->rows[tbl->num_rows].value = value;
    tbl->num_rows++;
    return 0;
}

void cursor_open(cursor_t *cur, const table_t *tbl, int forward) {
    cur->table = tbl;
    cur->direction = forward ? 1 : -1;
    cur->position = forward ? 0 : tbl->num_rows - 1;
    cur->valid = tbl->num_rows > 0;
}

int cursor_is_valid(const cursor_t *cur) {
    return cur->valid &&
           cur->position >= 0 &&
           cur->position < cur->table->num_rows;
}

int cursor_get(const cursor_t *cur, uint32_t *key, uint32_t *value) {
    if (!cursor_is_valid(cur)) return -1;
    *key = cur->table->rows[cur->position].key;
    *value = cur->table->rows[cur->position].value;
    return 0;
}

int cursor_advance(cursor_t *cur) {
    if (!cur->valid) return -1;
    cur->position += cur->direction;
    if (cur->position < 0 || cur->position >= cur->table->num_rows) {
        cur->valid = 0;
        return -1;
    }
    return 0;
}

void cursor_rewind(cursor_t *cur) {
    if (cur->direction > 0) {
        cur->position = 0;
    } else {
        cur->position = cur->table->num_rows - 1;
    }
    cur->valid = cur->table->num_rows > 0;
}

int cursor_seek(cursor_t *cur, uint32_t key) {
    int i;
    for (i = 0; i < cur->table->num_rows; i++) {
        if (cur->table->rows[i].key == key) {
            cur->position = i;
            cur->valid = 1;
            return 0;
        }
    }
    cur->valid = 0;
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C371: Cursor-based iteration should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C371: Output should not be empty");
    assert!(
        code.contains("fn cursor_advance"),
        "C371: Should contain cursor_advance function"
    );
    assert!(
        code.contains("fn cursor_seek"),
        "C371: Should contain cursor_seek function"
    );
}

#[test]
fn c372_schema_catalog_lookup() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define MAX_TABLES 16
#define MAX_COLUMNS 16
#define NAME_MAX_LEN 32

#define COL_TYPE_INT    1
#define COL_TYPE_FLOAT  2
#define COL_TYPE_TEXT   3
#define COL_TYPE_BOOL   4

typedef struct {
    uint8_t name[NAME_MAX_LEN];
    int name_len;
    uint8_t col_type;
    int nullable;
    int is_primary_key;
} column_def_t;

typedef struct {
    uint8_t name[NAME_MAX_LEN];
    int name_len;
    uint32_t table_id;
    column_def_t columns[MAX_COLUMNS];
    int num_columns;
    uint32_t row_count;
} table_def_t;

typedef struct {
    table_def_t tables[MAX_TABLES];
    int num_tables;
    uint32_t next_table_id;
} catalog_t;

void catalog_init(catalog_t *cat) {
    cat->num_tables = 0;
    cat->next_table_id = 1;
}

static int name_equals(const uint8_t *a, int a_len, const uint8_t *b, int b_len) {
    if (a_len != b_len) return 0;
    int i;
    for (i = 0; i < a_len; i++) {
        if (a[i] != b[i]) return 0;
    }
    return 1;
}

int catalog_find_table(const catalog_t *cat, const uint8_t *name, int name_len) {
    int i;
    for (i = 0; i < cat->num_tables; i++) {
        if (name_equals(cat->tables[i].name, cat->tables[i].name_len,
                        name, name_len)) {
            return i;
        }
    }
    return -1;
}

int catalog_create_table(catalog_t *cat, const uint8_t *name, int name_len) {
    if (cat->num_tables >= MAX_TABLES) return -1;
    if (catalog_find_table(cat, name, name_len) >= 0) return -2;
    int idx = cat->num_tables;
    int i;
    int copy_len = name_len < NAME_MAX_LEN ? name_len : NAME_MAX_LEN;
    for (i = 0; i < copy_len; i++) {
        cat->tables[idx].name[i] = name[i];
    }
    cat->tables[idx].name_len = copy_len;
    cat->tables[idx].table_id = cat->next_table_id++;
    cat->tables[idx].num_columns = 0;
    cat->tables[idx].row_count = 0;
    cat->num_tables++;
    return idx;
}

int catalog_add_column(catalog_t *cat, int table_idx,
                       const uint8_t *name, int name_len,
                       uint8_t col_type, int nullable) {
    if (table_idx < 0 || table_idx >= cat->num_tables) return -1;
    table_def_t *tbl = &cat->tables[table_idx];
    if (tbl->num_columns >= MAX_COLUMNS) return -1;
    int ci = tbl->num_columns;
    int i;
    int copy_len = name_len < NAME_MAX_LEN ? name_len : NAME_MAX_LEN;
    for (i = 0; i < copy_len; i++) {
        tbl->columns[ci].name[i] = name[i];
    }
    tbl->columns[ci].name_len = copy_len;
    tbl->columns[ci].col_type = col_type;
    tbl->columns[ci].nullable = nullable;
    tbl->columns[ci].is_primary_key = 0;
    tbl->num_columns++;
    return 0;
}

int catalog_find_column(const catalog_t *cat, int table_idx,
                        const uint8_t *name, int name_len) {
    if (table_idx < 0 || table_idx >= cat->num_tables) return -1;
    const table_def_t *tbl = &cat->tables[table_idx];
    int i;
    for (i = 0; i < tbl->num_columns; i++) {
        if (name_equals(tbl->columns[i].name, tbl->columns[i].name_len,
                        name, name_len)) {
            return i;
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C372: Schema catalog lookup should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C372: Output should not be empty");
    assert!(
        code.contains("fn catalog_create_table"),
        "C372: Should contain catalog_create_table function"
    );
    assert!(
        code.contains("fn catalog_find_column"),
        "C372: Should contain catalog_find_column function"
    );
}

#[test]
fn c373_statistics_histogram_equi_depth() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define HIST_MAX_BUCKETS 32

typedef struct {
    uint32_t lower_bound;
    uint32_t upper_bound;
    uint32_t frequency;
    uint32_t distinct_values;
} hist_bucket_t;

typedef struct {
    hist_bucket_t buckets[HIST_MAX_BUCKETS];
    int num_buckets;
    uint32_t total_rows;
    uint32_t min_value;
    uint32_t max_value;
    uint32_t num_distinct;
} histogram_t;

void hist_init(histogram_t *h) {
    h->num_buckets = 0;
    h->total_rows = 0;
    h->min_value = 0xFFFFFFFF;
    h->max_value = 0;
    h->num_distinct = 0;
}

int hist_add_bucket(histogram_t *h, uint32_t lower, uint32_t upper,
                    uint32_t freq, uint32_t distinct) {
    if (h->num_buckets >= HIST_MAX_BUCKETS) return -1;
    h->buckets[h->num_buckets].lower_bound = lower;
    h->buckets[h->num_buckets].upper_bound = upper;
    h->buckets[h->num_buckets].frequency = freq;
    h->buckets[h->num_buckets].distinct_values = distinct;
    h->num_buckets++;
    h->total_rows += freq;
    if (lower < h->min_value) h->min_value = lower;
    if (upper > h->max_value) h->max_value = upper;
    h->num_distinct += distinct;
    return 0;
}

double hist_estimate_selectivity(const histogram_t *h,
                                 uint32_t lower, uint32_t upper) {
    if (h->total_rows == 0) return 0.0;
    uint32_t matching = 0;
    int i;
    for (i = 0; i < h->num_buckets; i++) {
        uint32_t bl = h->buckets[i].lower_bound;
        uint32_t bu = h->buckets[i].upper_bound;
        if (bu < lower || bl > upper) continue;
        uint32_t overlap_lo = bl > lower ? bl : lower;
        uint32_t overlap_hi = bu < upper ? bu : upper;
        uint32_t range = bu - bl;
        if (range == 0) {
            matching += h->buckets[i].frequency;
        } else {
            double frac = (double)(overlap_hi - overlap_lo) / (double)range;
            matching += (uint32_t)(h->buckets[i].frequency * frac);
        }
    }
    return (double)matching / (double)h->total_rows;
}

double hist_estimate_distinct(const histogram_t *h,
                              uint32_t lower, uint32_t upper) {
    if (h->num_distinct == 0) return 0.0;
    uint32_t distinct = 0;
    int i;
    for (i = 0; i < h->num_buckets; i++) {
        if (h->buckets[i].upper_bound >= lower &&
            h->buckets[i].lower_bound <= upper) {
            distinct += h->buckets[i].distinct_values;
        }
    }
    return (double)distinct;
}

int hist_find_bucket(const histogram_t *h, uint32_t value) {
    int i;
    for (i = 0; i < h->num_buckets; i++) {
        if (value >= h->buckets[i].lower_bound &&
            value <= h->buckets[i].upper_bound) {
            return i;
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C373: Statistics histogram should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C373: Output should not be empty");
    assert!(
        code.contains("fn hist_estimate_selectivity"),
        "C373: Should contain hist_estimate_selectivity function"
    );
    assert!(
        code.contains("fn hist_find_bucket"),
        "C373: Should contain hist_find_bucket function"
    );
}

#[test]
fn c374_buffer_frame_pinning_protocol() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define FRAME_POOL_SIZE 32

#define FRAME_STATE_FREE     0
#define FRAME_STATE_CLEAN    1
#define FRAME_STATE_DIRTY    2

typedef struct {
    uint32_t page_id;
    int state;
    int pin_count;
    int reference_bit;
    uint32_t last_access;
} frame_t;

typedef struct {
    frame_t frames[FRAME_POOL_SIZE];
    int clock_hand;
    uint32_t access_counter;
} frame_pool_t;

void frame_pool_init(frame_pool_t *pool) {
    int i;
    for (i = 0; i < FRAME_POOL_SIZE; i++) {
        pool->frames[i].page_id = 0;
        pool->frames[i].state = FRAME_STATE_FREE;
        pool->frames[i].pin_count = 0;
        pool->frames[i].reference_bit = 0;
        pool->frames[i].last_access = 0;
    }
    pool->clock_hand = 0;
    pool->access_counter = 0;
}

int frame_pool_find(const frame_pool_t *pool, uint32_t page_id) {
    int i;
    for (i = 0; i < FRAME_POOL_SIZE; i++) {
        if (pool->frames[i].state != FRAME_STATE_FREE &&
            pool->frames[i].page_id == page_id) {
            return i;
        }
    }
    return -1;
}

int frame_pool_clock_evict(frame_pool_t *pool) {
    int start = pool->clock_hand;
    int loops = 0;
    while (loops < 2 * FRAME_POOL_SIZE) {
        int idx = pool->clock_hand;
        pool->clock_hand = (pool->clock_hand + 1) % FRAME_POOL_SIZE;
        if (pool->frames[idx].state == FRAME_STATE_FREE) {
            return idx;
        }
        if (pool->frames[idx].pin_count == 0) {
            if (pool->frames[idx].reference_bit) {
                pool->frames[idx].reference_bit = 0;
            } else {
                return idx;
            }
        }
        loops++;
    }
    return -1;
}

int frame_pin(frame_pool_t *pool, uint32_t page_id) {
    int idx = frame_pool_find(pool, page_id);
    if (idx >= 0) {
        pool->frames[idx].pin_count++;
        pool->frames[idx].reference_bit = 1;
        pool->frames[idx].last_access = ++pool->access_counter;
        return idx;
    }
    idx = frame_pool_clock_evict(pool);
    if (idx < 0) return -1;
    pool->frames[idx].page_id = page_id;
    pool->frames[idx].state = FRAME_STATE_CLEAN;
    pool->frames[idx].pin_count = 1;
    pool->frames[idx].reference_bit = 1;
    pool->frames[idx].last_access = ++pool->access_counter;
    return idx;
}

void frame_unpin(frame_pool_t *pool, int idx, int dirty) {
    if (idx >= 0 && idx < FRAME_POOL_SIZE) {
        if (pool->frames[idx].pin_count > 0) {
            pool->frames[idx].pin_count--;
        }
        if (dirty) {
            pool->frames[idx].state = FRAME_STATE_DIRTY;
        }
    }
}

int frame_flush(frame_pool_t *pool, int idx) {
    if (idx < 0 || idx >= FRAME_POOL_SIZE) return -1;
    if (pool->frames[idx].state == FRAME_STATE_DIRTY) {
        pool->frames[idx].state = FRAME_STATE_CLEAN;
        return 1;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C374: Buffer frame pinning should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C374: Output should not be empty");
    assert!(
        code.contains("fn frame_pin"),
        "C374: Should contain frame_pin function"
    );
    assert!(
        code.contains("fn frame_pool_clock_evict"),
        "C374: Should contain frame_pool_clock_evict function"
    );
}

#[test]
fn c375_undo_log_for_rollback() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

#define UNDO_MAX_ENTRIES 128
#define UNDO_OP_SET     1
#define UNDO_OP_INSERT  2
#define UNDO_OP_DELETE  3

typedef struct {
    uint64_t txn_id;
    uint32_t table_id;
    uint32_t row_id;
    int op_type;
    uint32_t old_value;
    int has_old_value;
} undo_entry_t;

typedef struct {
    undo_entry_t entries[UNDO_MAX_ENTRIES];
    int count;
    uint64_t active_txn;
} undo_log_t;

void undo_init(undo_log_t *log) {
    log->count = 0;
    log->active_txn = 0;
}

int undo_log_set(undo_log_t *log, uint64_t txn_id,
                 uint32_t table_id, uint32_t row_id, uint32_t old_value) {
    if (log->count >= UNDO_MAX_ENTRIES) return -1;
    undo_entry_t *e = &log->entries[log->count];
    e->txn_id = txn_id;
    e->table_id = table_id;
    e->row_id = row_id;
    e->op_type = UNDO_OP_SET;
    e->old_value = old_value;
    e->has_old_value = 1;
    log->count++;
    return 0;
}

int undo_log_insert(undo_log_t *log, uint64_t txn_id,
                    uint32_t table_id, uint32_t row_id) {
    if (log->count >= UNDO_MAX_ENTRIES) return -1;
    undo_entry_t *e = &log->entries[log->count];
    e->txn_id = txn_id;
    e->table_id = table_id;
    e->row_id = row_id;
    e->op_type = UNDO_OP_INSERT;
    e->has_old_value = 0;
    log->count++;
    return 0;
}

int undo_log_delete(undo_log_t *log, uint64_t txn_id,
                    uint32_t table_id, uint32_t row_id, uint32_t old_value) {
    if (log->count >= UNDO_MAX_ENTRIES) return -1;
    undo_entry_t *e = &log->entries[log->count];
    e->txn_id = txn_id;
    e->table_id = table_id;
    e->row_id = row_id;
    e->op_type = UNDO_OP_DELETE;
    e->old_value = old_value;
    e->has_old_value = 1;
    log->count++;
    return 0;
}

int undo_rollback(undo_log_t *log, uint64_t txn_id,
                  uint32_t *rollback_ops) {
    int ops = 0;
    int i;
    for (i = log->count - 1; i >= 0; i--) {
        if (log->entries[i].txn_id == txn_id) {
            rollback_ops[ops] = log->entries[i].row_id;
            ops++;
        }
    }
    return ops;
}

int undo_count_for_txn(const undo_log_t *log, uint64_t txn_id) {
    int count = 0;
    int i;
    for (i = 0; i < log->count; i++) {
        if (log->entries[i].txn_id == txn_id) {
            count++;
        }
    }
    return count;
}

void undo_truncate(undo_log_t *log, uint64_t txn_id) {
    int write = 0;
    int i;
    for (i = 0; i < log->count; i++) {
        if (log->entries[i].txn_id != txn_id) {
            if (write != i) {
                log->entries[write] = log->entries[i];
            }
            write++;
        }
    }
    log->count = write;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C375: Undo log for rollback should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C375: Output should not be empty");
    assert!(
        code.contains("fn undo_rollback"),
        "C375: Should contain undo_rollback function"
    );
    assert!(
        code.contains("fn undo_truncate"),
        "C375: Should contain undo_truncate function"
    );
}
