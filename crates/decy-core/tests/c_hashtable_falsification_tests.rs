//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1026-C1050: Hash Tables -- open addressing, chaining, cuckoo hashing,
//! Robin Hood hashing, probabilistic structures, and database-style hash joins.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world hash table patterns commonly found in
//! database engines, compilers, operating systems, and game engines
//! -- all expressed as valid C99 without #include directives.
//!
//! Organization:
//! - C1026-C1030: Core hash tables (open addressing, chaining, cuckoo, Robin Hood, double hashing)
//! - C1031-C1035: Advanced probing (quadratic, hopscotch, Swiss table, extendible, linear hashing)
//! - C1036-C1040: Hash functions (perfect hashing, FNV-1a, MurmurHash3, CityHash, SipHash)
//! - C1041-C1045: Probabilistic & specialized (Zobrist, consistent ring, Bloom, counting Bloom, skip list)
//! - C1046-C1050: Compound structures (hash set, multi-map, ordered map, coalesced, hash join)
//!
//! Results: 24 passing, 1 falsified (96.0% pass rate)

// ============================================================================
// C1026-C1030: Core Hash Tables
// ============================================================================

/// C1026: Open addressing with linear probing
#[test]
fn c1026_open_addressing_linear_probing() {
    let c_code = r#"
typedef unsigned long size_t;

#define HT_OA_CAP 256
#define HT_OA_EMPTY -1
#define HT_OA_DELETED -2

typedef struct {
    int keys[HT_OA_CAP];
    int values[HT_OA_CAP];
    int status[HT_OA_CAP];
    int size;
} ht_oa_t;

void ht_oa_init(ht_oa_t *ht) {
    int i;
    ht->size = 0;
    for (i = 0; i < HT_OA_CAP; i++) {
        ht->keys[i] = 0;
        ht->values[i] = 0;
        ht->status[i] = HT_OA_EMPTY;
    }
}

static unsigned int ht_oa_hash(int key) {
    unsigned int h = (unsigned int)key;
    h = ((h >> 16) ^ h) * 0x45d9f3b;
    h = ((h >> 16) ^ h) * 0x45d9f3b;
    h = (h >> 16) ^ h;
    return h % HT_OA_CAP;
}

int ht_oa_put(ht_oa_t *ht, int key, int value) {
    if (ht->size >= HT_OA_CAP) return -1;
    unsigned int idx = ht_oa_hash(key);
    int i;
    for (i = 0; i < HT_OA_CAP; i++) {
        unsigned int pos = (idx + i) % HT_OA_CAP;
        if (ht->status[pos] == HT_OA_EMPTY || ht->status[pos] == HT_OA_DELETED) {
            ht->keys[pos] = key;
            ht->values[pos] = value;
            ht->status[pos] = 1;
            ht->size++;
            return 0;
        }
        if (ht->status[pos] == 1 && ht->keys[pos] == key) {
            ht->values[pos] = value;
            return 0;
        }
    }
    return -1;
}

int ht_oa_get(const ht_oa_t *ht, int key, int *out_value) {
    unsigned int idx = ht_oa_hash(key);
    int i;
    for (i = 0; i < HT_OA_CAP; i++) {
        unsigned int pos = (idx + i) % HT_OA_CAP;
        if (ht->status[pos] == HT_OA_EMPTY) return -1;
        if (ht->status[pos] == 1 && ht->keys[pos] == key) {
            *out_value = ht->values[pos];
            return 0;
        }
    }
    return -1;
}

int ht_oa_delete(ht_oa_t *ht, int key) {
    unsigned int idx = ht_oa_hash(key);
    int i;
    for (i = 0; i < HT_OA_CAP; i++) {
        unsigned int pos = (idx + i) % HT_OA_CAP;
        if (ht->status[pos] == HT_OA_EMPTY) return -1;
        if (ht->status[pos] == 1 && ht->keys[pos] == key) {
            ht->status[pos] = HT_OA_DELETED;
            ht->size--;
            return 0;
        }
    }
    return -1;
}

int ht_oa_test(void) {
    ht_oa_t ht;
    ht_oa_init(&ht);
    ht_oa_put(&ht, 42, 100);
    ht_oa_put(&ht, 99, 200);
    int v = 0;
    if (ht_oa_get(&ht, 42, &v) != 0) return -1;
    if (v != 100) return -2;
    ht_oa_delete(&ht, 42);
    if (ht_oa_get(&ht, 42, &v) == 0) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1026: Open addressing linear probing should transpile: {:?}", result.err());
}

/// C1027: Separate chaining with array-based buckets
#[test]
fn c1027_separate_chaining() {
    let c_code = r#"
typedef unsigned long size_t;

#define HT_SC_BUCKETS 64
#define HT_SC_CHAIN 8
#define HT_SC_EMPTY -1

typedef struct {
    int keys[HT_SC_CHAIN];
    int values[HT_SC_CHAIN];
    int count;
} ht_sc_bucket_t;

typedef struct {
    ht_sc_bucket_t buckets[HT_SC_BUCKETS];
    int size;
} ht_sc_t;

void ht_sc_init(ht_sc_t *ht) {
    int i, j;
    ht->size = 0;
    for (i = 0; i < HT_SC_BUCKETS; i++) {
        ht->buckets[i].count = 0;
        for (j = 0; j < HT_SC_CHAIN; j++) {
            ht->buckets[i].keys[j] = HT_SC_EMPTY;
            ht->buckets[i].values[j] = 0;
        }
    }
}

static unsigned int ht_sc_hash(int key) {
    unsigned int h = (unsigned int)key;
    h ^= h >> 16;
    h *= 0x85ebca6b;
    h ^= h >> 13;
    return h % HT_SC_BUCKETS;
}

int ht_sc_put(ht_sc_t *ht, int key, int value) {
    unsigned int b = ht_sc_hash(key);
    ht_sc_bucket_t *bkt = &ht->buckets[b];
    int i;
    for (i = 0; i < bkt->count; i++) {
        if (bkt->keys[i] == key) {
            bkt->values[i] = value;
            return 0;
        }
    }
    if (bkt->count >= HT_SC_CHAIN) return -1;
    bkt->keys[bkt->count] = key;
    bkt->values[bkt->count] = value;
    bkt->count++;
    ht->size++;
    return 0;
}

int ht_sc_get(const ht_sc_t *ht, int key, int *out) {
    unsigned int b = ht_sc_hash(key);
    const ht_sc_bucket_t *bkt = &ht->buckets[b];
    int i;
    for (i = 0; i < bkt->count; i++) {
        if (bkt->keys[i] == key) {
            *out = bkt->values[i];
            return 0;
        }
    }
    return -1;
}

int ht_sc_remove(ht_sc_t *ht, int key) {
    unsigned int b = ht_sc_hash(key);
    ht_sc_bucket_t *bkt = &ht->buckets[b];
    int i;
    for (i = 0; i < bkt->count; i++) {
        if (bkt->keys[i] == key) {
            bkt->keys[i] = bkt->keys[bkt->count - 1];
            bkt->values[i] = bkt->values[bkt->count - 1];
            bkt->count--;
            ht->size--;
            return 0;
        }
    }
    return -1;
}

int ht_sc_test(void) {
    ht_sc_t ht;
    ht_sc_init(&ht);
    ht_sc_put(&ht, 10, 100);
    ht_sc_put(&ht, 74, 200);
    int v = 0;
    if (ht_sc_get(&ht, 10, &v) != 0) return -1;
    if (v != 100) return -2;
    ht_sc_remove(&ht, 10);
    if (ht_sc_get(&ht, 10, &v) == 0) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1027: Separate chaining should transpile: {:?}", result.err());
}

/// C1028: Cuckoo hashing with two tables
#[test]
fn c1028_cuckoo_hashing() {
    let c_code = r#"
typedef unsigned long size_t;

#define HT_CK_CAP 128
#define HT_CK_EMPTY -1
#define HT_CK_MAX_KICKS 32

typedef struct {
    int keys1[HT_CK_CAP];
    int vals1[HT_CK_CAP];
    int occ1[HT_CK_CAP];
    int keys2[HT_CK_CAP];
    int vals2[HT_CK_CAP];
    int occ2[HT_CK_CAP];
    int size;
} ht_cuckoo_t;

void ht_cuckoo_init(ht_cuckoo_t *ht) {
    int i;
    ht->size = 0;
    for (i = 0; i < HT_CK_CAP; i++) {
        ht->occ1[i] = 0;
        ht->occ2[i] = 0;
        ht->keys1[i] = 0;
        ht->vals1[i] = 0;
        ht->keys2[i] = 0;
        ht->vals2[i] = 0;
    }
}

static unsigned int ht_cuckoo_h1(int key) {
    unsigned int h = (unsigned int)key;
    h = ((h >> 16) ^ h) * 0x45d9f3b;
    return h % HT_CK_CAP;
}

static unsigned int ht_cuckoo_h2(int key) {
    unsigned int h = (unsigned int)key;
    h = ((h >> 16) ^ h) * 0x119de1f3;
    h = ((h >> 16) ^ h);
    return h % HT_CK_CAP;
}

int ht_cuckoo_get(const ht_cuckoo_t *ht, int key, int *out) {
    unsigned int p1 = ht_cuckoo_h1(key);
    if (ht->occ1[p1] && ht->keys1[p1] == key) {
        *out = ht->vals1[p1];
        return 0;
    }
    unsigned int p2 = ht_cuckoo_h2(key);
    if (ht->occ2[p2] && ht->keys2[p2] == key) {
        *out = ht->vals2[p2];
        return 0;
    }
    return -1;
}

int ht_cuckoo_put(ht_cuckoo_t *ht, int key, int value) {
    int cur_key = key;
    int cur_val = value;
    int i;
    for (i = 0; i < HT_CK_MAX_KICKS; i++) {
        unsigned int p1 = ht_cuckoo_h1(cur_key);
        if (!ht->occ1[p1]) {
            ht->keys1[p1] = cur_key;
            ht->vals1[p1] = cur_val;
            ht->occ1[p1] = 1;
            ht->size++;
            return 0;
        }
        int tmp_k = ht->keys1[p1];
        int tmp_v = ht->vals1[p1];
        ht->keys1[p1] = cur_key;
        ht->vals1[p1] = cur_val;
        cur_key = tmp_k;
        cur_val = tmp_v;

        unsigned int p2 = ht_cuckoo_h2(cur_key);
        if (!ht->occ2[p2]) {
            ht->keys2[p2] = cur_key;
            ht->vals2[p2] = cur_val;
            ht->occ2[p2] = 1;
            ht->size++;
            return 0;
        }
        tmp_k = ht->keys2[p2];
        tmp_v = ht->vals2[p2];
        ht->keys2[p2] = cur_key;
        ht->vals2[p2] = cur_val;
        cur_key = tmp_k;
        cur_val = tmp_v;
    }
    return -1;
}

int ht_cuckoo_test(void) {
    ht_cuckoo_t ht;
    ht_cuckoo_init(&ht);
    ht_cuckoo_put(&ht, 5, 50);
    ht_cuckoo_put(&ht, 133, 1330);
    int v = 0;
    if (ht_cuckoo_get(&ht, 5, &v) != 0) return -1;
    if (v != 50) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1028: Cuckoo hashing should transpile: {:?}", result.err());
}

/// C1029: Robin Hood hashing with displacement tracking
#[test]
fn c1029_robin_hood_hashing() {
    let c_code = r#"
typedef unsigned long size_t;

#define HT_RH_CAP 256
#define HT_RH_EMPTY 0
#define HT_RH_OCCUPIED 1

typedef struct {
    int keys[HT_RH_CAP];
    int values[HT_RH_CAP];
    int dist[HT_RH_CAP];
    int flags[HT_RH_CAP];
    int size;
} ht_robinhood_t;

void ht_rh_init(ht_robinhood_t *ht) {
    int i;
    ht->size = 0;
    for (i = 0; i < HT_RH_CAP; i++) {
        ht->flags[i] = HT_RH_EMPTY;
        ht->dist[i] = 0;
        ht->keys[i] = 0;
        ht->values[i] = 0;
    }
}

static unsigned int ht_rh_hash(int key) {
    unsigned int h = (unsigned int)key;
    h ^= h >> 16;
    h *= 0x85ebca6b;
    h ^= h >> 13;
    h *= 0xc2b2ae35;
    h ^= h >> 16;
    return h % HT_RH_CAP;
}

int ht_rh_put(ht_robinhood_t *ht, int key, int value) {
    if (ht->size >= HT_RH_CAP) return -1;
    unsigned int idx = ht_rh_hash(key);
    int cur_key = key;
    int cur_val = value;
    int cur_dist = 0;
    int i;
    for (i = 0; i < HT_RH_CAP; i++) {
        unsigned int pos = (idx + i) % HT_RH_CAP;
        if (ht->flags[pos] == HT_RH_EMPTY) {
            ht->keys[pos] = cur_key;
            ht->values[pos] = cur_val;
            ht->dist[pos] = cur_dist;
            ht->flags[pos] = HT_RH_OCCUPIED;
            ht->size++;
            return 0;
        }
        if (ht->keys[pos] == cur_key) {
            ht->values[pos] = cur_val;
            return 0;
        }
        if (cur_dist > ht->dist[pos]) {
            int tmp_k = ht->keys[pos];
            int tmp_v = ht->values[pos];
            int tmp_d = ht->dist[pos];
            ht->keys[pos] = cur_key;
            ht->values[pos] = cur_val;
            ht->dist[pos] = cur_dist;
            cur_key = tmp_k;
            cur_val = tmp_v;
            cur_dist = tmp_d;
        }
        cur_dist++;
    }
    return -1;
}

int ht_rh_get(const ht_robinhood_t *ht, int key, int *out) {
    unsigned int idx = ht_rh_hash(key);
    int d = 0;
    int i;
    for (i = 0; i < HT_RH_CAP; i++) {
        unsigned int pos = (idx + i) % HT_RH_CAP;
        if (ht->flags[pos] == HT_RH_EMPTY) return -1;
        if (d > ht->dist[pos]) return -1;
        if (ht->keys[pos] == key) {
            *out = ht->values[pos];
            return 0;
        }
        d++;
    }
    return -1;
}

int ht_rh_test(void) {
    ht_robinhood_t ht;
    ht_rh_init(&ht);
    ht_rh_put(&ht, 7, 70);
    ht_rh_put(&ht, 263, 2630);
    int v = 0;
    if (ht_rh_get(&ht, 7, &v) != 0) return -1;
    if (v != 70) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1029: Robin Hood hashing should transpile: {:?}", result.err());
}

/// C1030: Double hashing
#[test]
fn c1030_double_hashing() {
    let c_code = r#"
typedef unsigned long size_t;

#define HT_DH_CAP 251
#define HT_DH_EMPTY 0
#define HT_DH_OCCUPIED 1
#define HT_DH_DELETED 2

typedef struct {
    int keys[HT_DH_CAP];
    int values[HT_DH_CAP];
    int state[HT_DH_CAP];
    int size;
} ht_double_t;

void ht_dh_init(ht_double_t *ht) {
    int i;
    ht->size = 0;
    for (i = 0; i < HT_DH_CAP; i++) {
        ht->state[i] = HT_DH_EMPTY;
        ht->keys[i] = 0;
        ht->values[i] = 0;
    }
}

static unsigned int ht_dh_h1(int key) {
    unsigned int h = (unsigned int)(key < 0 ? -key : key);
    return h % HT_DH_CAP;
}

static unsigned int ht_dh_h2(int key) {
    unsigned int h = (unsigned int)(key < 0 ? -key : key);
    return 1 + (h % (HT_DH_CAP - 1));
}

int ht_dh_put(ht_double_t *ht, int key, int value) {
    if (ht->size >= HT_DH_CAP) return -1;
    unsigned int h1 = ht_dh_h1(key);
    unsigned int h2 = ht_dh_h2(key);
    int i;
    for (i = 0; i < HT_DH_CAP; i++) {
        unsigned int pos = (h1 + (unsigned int)i * h2) % HT_DH_CAP;
        if (ht->state[pos] != HT_DH_OCCUPIED) {
            ht->keys[pos] = key;
            ht->values[pos] = value;
            ht->state[pos] = HT_DH_OCCUPIED;
            ht->size++;
            return 0;
        }
        if (ht->keys[pos] == key) {
            ht->values[pos] = value;
            return 0;
        }
    }
    return -1;
}

int ht_dh_get(const ht_double_t *ht, int key, int *out) {
    unsigned int h1 = ht_dh_h1(key);
    unsigned int h2 = ht_dh_h2(key);
    int i;
    for (i = 0; i < HT_DH_CAP; i++) {
        unsigned int pos = (h1 + (unsigned int)i * h2) % HT_DH_CAP;
        if (ht->state[pos] == HT_DH_EMPTY) return -1;
        if (ht->state[pos] == HT_DH_OCCUPIED && ht->keys[pos] == key) {
            *out = ht->values[pos];
            return 0;
        }
    }
    return -1;
}

int ht_dh_remove(ht_double_t *ht, int key) {
    unsigned int h1 = ht_dh_h1(key);
    unsigned int h2 = ht_dh_h2(key);
    int i;
    for (i = 0; i < HT_DH_CAP; i++) {
        unsigned int pos = (h1 + (unsigned int)i * h2) % HT_DH_CAP;
        if (ht->state[pos] == HT_DH_EMPTY) return -1;
        if (ht->state[pos] == HT_DH_OCCUPIED && ht->keys[pos] == key) {
            ht->state[pos] = HT_DH_DELETED;
            ht->size--;
            return 0;
        }
    }
    return -1;
}

int ht_dh_test(void) {
    ht_double_t ht;
    ht_dh_init(&ht);
    ht_dh_put(&ht, 15, 150);
    ht_dh_put(&ht, 266, 2660);
    int v = 0;
    if (ht_dh_get(&ht, 15, &v) != 0) return -1;
    if (v != 150) return -2;
    ht_dh_remove(&ht, 15);
    if (ht_dh_get(&ht, 15, &v) == 0) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1030: Double hashing should transpile: {:?}", result.err());
}

// ============================================================================
// C1031-C1035: Advanced Probing Strategies
// ============================================================================

/// C1031: Quadratic probing
#[test]
fn c1031_quadratic_probing() {
    let c_code = r#"
typedef unsigned long size_t;

#define HT_QP_CAP 256

typedef struct {
    int keys[HT_QP_CAP];
    int values[HT_QP_CAP];
    int occupied[HT_QP_CAP];
    int size;
} ht_quad_t;

void ht_qp_init(ht_quad_t *ht) {
    int i;
    ht->size = 0;
    for (i = 0; i < HT_QP_CAP; i++) {
        ht->occupied[i] = 0;
        ht->keys[i] = 0;
        ht->values[i] = 0;
    }
}

static unsigned int ht_qp_hash(int key) {
    unsigned int h = (unsigned int)key;
    h = ((h >> 16) ^ h) * 0x45d9f3b;
    return h % HT_QP_CAP;
}

int ht_qp_put(ht_quad_t *ht, int key, int value) {
    if (ht->size >= HT_QP_CAP / 2) return -1;
    unsigned int h = ht_qp_hash(key);
    int i;
    for (i = 0; i < HT_QP_CAP; i++) {
        unsigned int pos = (h + (unsigned int)(i * i)) % HT_QP_CAP;
        if (!ht->occupied[pos]) {
            ht->keys[pos] = key;
            ht->values[pos] = value;
            ht->occupied[pos] = 1;
            ht->size++;
            return 0;
        }
        if (ht->keys[pos] == key) {
            ht->values[pos] = value;
            return 0;
        }
    }
    return -1;
}

int ht_qp_get(const ht_quad_t *ht, int key, int *out) {
    unsigned int h = ht_qp_hash(key);
    int i;
    for (i = 0; i < HT_QP_CAP; i++) {
        unsigned int pos = (h + (unsigned int)(i * i)) % HT_QP_CAP;
        if (!ht->occupied[pos]) return -1;
        if (ht->keys[pos] == key) {
            *out = ht->values[pos];
            return 0;
        }
    }
    return -1;
}

int ht_qp_test(void) {
    ht_quad_t ht;
    ht_qp_init(&ht);
    ht_qp_put(&ht, 3, 30);
    ht_qp_put(&ht, 259, 2590);
    int v = 0;
    if (ht_qp_get(&ht, 3, &v) != 0) return -1;
    if (v != 30) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1031: Quadratic probing should transpile: {:?}", result.err());
}

/// C1032: Hopscotch hashing with bitmap neighborhoods
#[test]
fn c1032_hopscotch_hashing() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define HT_HS_CAP 256
#define HT_HS_NBHD 8

typedef struct {
    int keys[HT_HS_CAP];
    int values[HT_HS_CAP];
    uint32_t hop_info[HT_HS_CAP];
    int occupied[HT_HS_CAP];
    int size;
} ht_hopscotch_t;

void ht_hs_init(ht_hopscotch_t *ht) {
    int i;
    ht->size = 0;
    for (i = 0; i < HT_HS_CAP; i++) {
        ht->occupied[i] = 0;
        ht->hop_info[i] = 0;
        ht->keys[i] = 0;
        ht->values[i] = 0;
    }
}

static unsigned int ht_hs_hash(int key) {
    unsigned int h = (unsigned int)key;
    h ^= h >> 16;
    h *= 0x85ebca6b;
    return h % HT_HS_CAP;
}

int ht_hs_get(const ht_hopscotch_t *ht, int key, int *out) {
    unsigned int bucket = ht_hs_hash(key);
    uint32_t bitmap = ht->hop_info[bucket];
    int i;
    for (i = 0; i < HT_HS_NBHD; i++) {
        if (bitmap & (1u << i)) {
            unsigned int pos = (bucket + i) % HT_HS_CAP;
            if (ht->keys[pos] == key) {
                *out = ht->values[pos];
                return 0;
            }
        }
    }
    return -1;
}

int ht_hs_put(ht_hopscotch_t *ht, int key, int value) {
    unsigned int bucket = ht_hs_hash(key);
    unsigned int pos = bucket;
    int found = -1;
    int i;
    for (i = 0; i < HT_HS_CAP; i++) {
        unsigned int p = (bucket + i) % HT_HS_CAP;
        if (!ht->occupied[p]) {
            found = (int)p;
            break;
        }
    }
    if (found < 0) return -1;
    pos = (unsigned int)found;
    while (1) {
        int dist = (int)((pos - bucket + HT_HS_CAP) % HT_HS_CAP);
        if (dist < HT_HS_NBHD) {
            ht->keys[pos] = key;
            ht->values[pos] = value;
            ht->occupied[pos] = 1;
            ht->hop_info[bucket] |= (1u << dist);
            ht->size++;
            return 0;
        }
        int moved = 0;
        int j;
        for (j = HT_HS_NBHD - 1; j >= 1; j--) {
            unsigned int swap_bucket = (pos - j + HT_HS_CAP) % HT_HS_CAP;
            uint32_t swap_info = ht->hop_info[swap_bucket];
            int k;
            for (k = 0; k < j; k++) {
                if (swap_info & (1u << k)) {
                    unsigned int swap_pos = (swap_bucket + k) % HT_HS_CAP;
                    ht->keys[pos] = ht->keys[swap_pos];
                    ht->values[pos] = ht->values[swap_pos];
                    ht->occupied[pos] = 1;
                    ht->hop_info[swap_bucket] &= ~(1u << k);
                    ht->hop_info[swap_bucket] |= (1u << j);
                    ht->occupied[swap_pos] = 0;
                    pos = swap_pos;
                    moved = 1;
                    break;
                }
            }
            if (moved) break;
        }
        if (!moved) return -1;
    }
}

int ht_hs_test(void) {
    ht_hopscotch_t ht;
    ht_hs_init(&ht);
    ht_hs_put(&ht, 11, 110);
    ht_hs_put(&ht, 267, 2670);
    int v = 0;
    if (ht_hs_get(&ht, 11, &v) != 0) return -1;
    if (v != 110) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1032: Hopscotch hashing should transpile: {:?}", result.err());
}

/// C1033: Swiss table (simplified SIMD-like group probing)
#[test]
fn c1033_swiss_table() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

#define HT_SW_CAP 256
#define HT_SW_GROUP 16
#define HT_SW_EMPTY 0xFF
#define HT_SW_DELETED 0xFE

typedef struct {
    uint8_t ctrl[HT_SW_CAP];
    int keys[HT_SW_CAP];
    int values[HT_SW_CAP];
    int size;
} ht_swiss_t;

void ht_sw_init(ht_swiss_t *ht) {
    int i;
    ht->size = 0;
    for (i = 0; i < HT_SW_CAP; i++) {
        ht->ctrl[i] = HT_SW_EMPTY;
        ht->keys[i] = 0;
        ht->values[i] = 0;
    }
}

static uint8_t ht_sw_h2(unsigned int hash) {
    return (uint8_t)(hash >> 24) & 0x7F;
}

static unsigned int ht_sw_h1(int key) {
    unsigned int h = (unsigned int)key;
    h ^= h >> 16;
    h *= 0x85ebca6b;
    h ^= h >> 13;
    return h;
}

static int ht_sw_group_match(const ht_swiss_t *ht, unsigned int group_start, uint8_t h2_val, int key) {
    int i;
    for (i = 0; i < HT_SW_GROUP; i++) {
        unsigned int pos = (group_start + i) % HT_SW_CAP;
        if (ht->ctrl[pos] == h2_val && ht->keys[pos] == key) return (int)pos;
    }
    return -1;
}

static int ht_sw_group_find_empty(const ht_swiss_t *ht, unsigned int group_start) {
    int i;
    for (i = 0; i < HT_SW_GROUP; i++) {
        unsigned int pos = (group_start + i) % HT_SW_CAP;
        if (ht->ctrl[pos] == HT_SW_EMPTY || ht->ctrl[pos] == HT_SW_DELETED) return (int)pos;
    }
    return -1;
}

int ht_sw_get(const ht_swiss_t *ht, int key, int *out) {
    unsigned int hash = ht_sw_h1(key);
    uint8_t h2 = ht_sw_h2(hash);
    unsigned int g = hash % HT_SW_CAP;
    int num_groups = HT_SW_CAP / HT_SW_GROUP;
    int gi;
    for (gi = 0; gi < num_groups; gi++) {
        unsigned int gs = (g + gi * HT_SW_GROUP) % HT_SW_CAP;
        int pos = ht_sw_group_match(ht, gs, h2, key);
        if (pos >= 0) {
            *out = ht->values[pos];
            return 0;
        }
        int has_empty = 0;
        int i;
        for (i = 0; i < HT_SW_GROUP; i++) {
            if (ht->ctrl[(gs + i) % HT_SW_CAP] == HT_SW_EMPTY) { has_empty = 1; break; }
        }
        if (has_empty) return -1;
    }
    return -1;
}

int ht_sw_put(ht_swiss_t *ht, int key, int value) {
    if (ht->size >= HT_SW_CAP * 7 / 8) return -1;
    unsigned int hash = ht_sw_h1(key);
    uint8_t h2 = ht_sw_h2(hash);
    unsigned int g = hash % HT_SW_CAP;
    int num_groups = HT_SW_CAP / HT_SW_GROUP;
    int gi;
    for (gi = 0; gi < num_groups; gi++) {
        unsigned int gs = (g + gi * HT_SW_GROUP) % HT_SW_CAP;
        int pos = ht_sw_group_match(ht, gs, h2, key);
        if (pos >= 0) {
            ht->values[pos] = value;
            return 0;
        }
        int empty_pos = ht_sw_group_find_empty(ht, gs);
        if (empty_pos >= 0) {
            ht->ctrl[empty_pos] = h2;
            ht->keys[empty_pos] = key;
            ht->values[empty_pos] = value;
            ht->size++;
            return 0;
        }
    }
    return -1;
}

int ht_sw_test(void) {
    ht_swiss_t ht;
    ht_sw_init(&ht);
    ht_sw_put(&ht, 42, 420);
    ht_sw_put(&ht, 298, 2980);
    int v = 0;
    if (ht_sw_get(&ht, 42, &v) != 0) return -1;
    if (v != 420) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1033: Swiss table should transpile: {:?}", result.err());
}

/// C1034: Extendible hashing with directory doubling
#[test]
fn c1034_extendible_hashing() {
    let c_code = r#"
typedef unsigned long size_t;

#define HT_EX_BUCKET_SZ 4
#define HT_EX_MAX_DIR 64
#define HT_EX_MAX_BUCKETS 32

typedef struct {
    int keys[HT_EX_BUCKET_SZ];
    int values[HT_EX_BUCKET_SZ];
    int count;
    int local_depth;
} ht_ex_bucket_t;

typedef struct {
    ht_ex_bucket_t buckets[HT_EX_MAX_BUCKETS];
    int directory[HT_EX_MAX_DIR];
    int global_depth;
    int num_buckets;
    int dir_size;
} ht_extendible_t;

void ht_ex_init(ht_extendible_t *ht) {
    ht->global_depth = 1;
    ht->num_buckets = 2;
    ht->dir_size = 2;
    int i;
    for (i = 0; i < HT_EX_MAX_BUCKETS; i++) {
        ht->buckets[i].count = 0;
        ht->buckets[i].local_depth = 1;
    }
    ht->directory[0] = 0;
    ht->directory[1] = 1;
    for (i = 2; i < HT_EX_MAX_DIR; i++) {
        ht->directory[i] = -1;
    }
}

static unsigned int ht_ex_hash(int key) {
    unsigned int h = (unsigned int)(key < 0 ? -key : key);
    h ^= h >> 16;
    h *= 0x45d9f3b;
    return h;
}

static int ht_ex_get_dir_idx(const ht_extendible_t *ht, int key) {
    unsigned int h = ht_ex_hash(key);
    return (int)(h & ((unsigned int)ht->dir_size - 1));
}

int ht_ex_find(const ht_extendible_t *ht, int key, int *out) {
    int dir_idx = ht_ex_get_dir_idx(ht, key);
    int b = ht->directory[dir_idx];
    if (b < 0) return -1;
    const ht_ex_bucket_t *bkt = &ht->buckets[b];
    int i;
    for (i = 0; i < bkt->count; i++) {
        if (bkt->keys[i] == key) {
            *out = bkt->values[i];
            return 0;
        }
    }
    return -1;
}

int ht_ex_insert(ht_extendible_t *ht, int key, int value) {
    int dir_idx = ht_ex_get_dir_idx(ht, key);
    int b = ht->directory[dir_idx];
    if (b < 0) return -1;
    ht_ex_bucket_t *bkt = &ht->buckets[b];
    if (bkt->count < HT_EX_BUCKET_SZ) {
        bkt->keys[bkt->count] = key;
        bkt->values[bkt->count] = value;
        bkt->count++;
        return 0;
    }
    return -1;
}

int ht_ex_test(void) {
    ht_extendible_t ht;
    ht_ex_init(&ht);
    ht_ex_insert(&ht, 10, 100);
    ht_ex_insert(&ht, 3, 30);
    int v = 0;
    if (ht_ex_find(&ht, 10, &v) != 0) return -1;
    if (v != 100) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1034: Extendible hashing should transpile: {:?}", result.err());
}

/// C1035: Linear hashing with dynamic bucket splitting
#[test]
fn c1035_linear_hashing() {
    let c_code = r#"
typedef unsigned long size_t;

#define HT_LH_MAX_BUCKETS 128
#define HT_LH_BUCKET_SZ 4

typedef struct {
    int keys[HT_LH_BUCKET_SZ];
    int values[HT_LH_BUCKET_SZ];
    int count;
} ht_lh_bucket_t;

typedef struct {
    ht_lh_bucket_t buckets[HT_LH_MAX_BUCKETS];
    int num_buckets;
    int split_ptr;
    int level;
    int total_items;
} ht_linear_t;

void ht_lh_init(ht_linear_t *ht, int initial_buckets) {
    int i;
    ht->num_buckets = initial_buckets;
    ht->split_ptr = 0;
    ht->level = 0;
    ht->total_items = 0;
    for (i = 0; i < HT_LH_MAX_BUCKETS; i++) {
        ht->buckets[i].count = 0;
    }
}

static int ht_lh_hash(const ht_linear_t *ht, int key) {
    unsigned int h = (unsigned int)(key < 0 ? -key : key);
    h ^= h >> 16;
    h *= 0x45d9f3b;
    int initial = ht->num_buckets - ht->split_ptr;
    if (initial <= 0) initial = ht->num_buckets;
    int bucket = (int)(h % (unsigned int)initial);
    if (bucket < ht->split_ptr) {
        bucket = (int)(h % (unsigned int)(initial * 2));
    }
    return bucket;
}

int ht_lh_insert(ht_linear_t *ht, int key, int value) {
    int b = ht_lh_hash(ht, key);
    if (b < 0 || b >= ht->num_buckets) return -1;
    ht_lh_bucket_t *bkt = &ht->buckets[b];
    if (bkt->count >= HT_LH_BUCKET_SZ) {
        if (ht->num_buckets < HT_LH_MAX_BUCKETS) {
            ht->num_buckets++;
            ht->split_ptr++;
        }
        return -1;
    }
    bkt->keys[bkt->count] = key;
    bkt->values[bkt->count] = value;
    bkt->count++;
    ht->total_items++;
    return 0;
}

int ht_lh_find(const ht_linear_t *ht, int key, int *out) {
    int b = ht_lh_hash(ht, key);
    if (b < 0 || b >= ht->num_buckets) return -1;
    const ht_lh_bucket_t *bkt = &ht->buckets[b];
    int i;
    for (i = 0; i < bkt->count; i++) {
        if (bkt->keys[i] == key) {
            *out = bkt->values[i];
            return 0;
        }
    }
    return -1;
}

int ht_lh_test(void) {
    ht_linear_t ht;
    ht_lh_init(&ht, 4);
    ht_lh_insert(&ht, 7, 70);
    ht_lh_insert(&ht, 11, 110);
    int v = 0;
    if (ht_lh_find(&ht, 7, &v) != 0) return -1;
    if (v != 70) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1035: Linear hashing should transpile: {:?}", result.err());
}

// ============================================================================
// C1036-C1040: Hash Functions
// ============================================================================

/// C1036: Perfect hashing (two-level scheme)
#[test]
fn c1036_perfect_hashing() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define HT_PH_N 16
#define HT_PH_SLOTS 64

typedef struct {
    int keys[HT_PH_SLOTS];
    int values[HT_PH_SLOTS];
    int occupied[HT_PH_SLOTS];
    uint32_t a;
    uint32_t b;
    int m;
    int size;
} ht_perfect_t;

void ht_ph_init(ht_perfect_t *ht) {
    int i;
    ht->a = 31;
    ht->b = 17;
    ht->m = HT_PH_SLOTS;
    ht->size = 0;
    for (i = 0; i < HT_PH_SLOTS; i++) {
        ht->occupied[i] = 0;
        ht->keys[i] = 0;
        ht->values[i] = 0;
    }
}

static uint32_t ht_ph_hash(const ht_perfect_t *ht, int key) {
    uint32_t k = (uint32_t)(key < 0 ? -key : key);
    return ((ht->a * k + ht->b) % 101) % (uint32_t)ht->m;
}

int ht_ph_insert(ht_perfect_t *ht, int key, int value) {
    uint32_t pos = ht_ph_hash(ht, key);
    if (ht->occupied[pos] && ht->keys[pos] != key) {
        int i;
        for (i = 1; i < ht->m; i++) {
            uint32_t alt = (pos + (uint32_t)i) % (uint32_t)ht->m;
            if (!ht->occupied[alt]) {
                pos = alt;
                break;
            }
            if (ht->keys[alt] == key) {
                ht->values[alt] = value;
                return 0;
            }
        }
    }
    ht->keys[pos] = key;
    ht->values[pos] = value;
    if (!ht->occupied[pos]) {
        ht->occupied[pos] = 1;
        ht->size++;
    }
    return 0;
}

int ht_ph_lookup(const ht_perfect_t *ht, int key, int *out) {
    uint32_t pos = ht_ph_hash(ht, key);
    int i;
    for (i = 0; i < ht->m; i++) {
        uint32_t p = (pos + (uint32_t)i) % (uint32_t)ht->m;
        if (!ht->occupied[p]) return -1;
        if (ht->keys[p] == key) {
            *out = ht->values[p];
            return 0;
        }
    }
    return -1;
}

int ht_ph_test(void) {
    ht_perfect_t ht;
    ht_ph_init(&ht);
    ht_ph_insert(&ht, 5, 50);
    ht_ph_insert(&ht, 23, 230);
    int v = 0;
    if (ht_ph_lookup(&ht, 5, &v) != 0) return -1;
    if (v != 50) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1036: Perfect hashing should transpile: {:?}", result.err());
}

/// C1037: FNV-1a hash function with hash table
#[test]
fn c1037_fnv1a_hash() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define HT_FNV_CAP 128
#define HT_FNV_BASIS 0x811c9dc5u
#define HT_FNV_PRIME 0x01000193u

typedef struct {
    int keys[HT_FNV_CAP];
    int values[HT_FNV_CAP];
    int occupied[HT_FNV_CAP];
    int size;
} ht_fnv_t;

void ht_fnv_init(ht_fnv_t *ht) {
    int i;
    ht->size = 0;
    for (i = 0; i < HT_FNV_CAP; i++) {
        ht->occupied[i] = 0;
    }
}

static uint32_t ht_fnv1a_int(int key) {
    uint32_t hash = HT_FNV_BASIS;
    unsigned char *bytes = (unsigned char *)&key;
    int i;
    for (i = 0; i < 4; i++) {
        hash ^= bytes[i];
        hash *= HT_FNV_PRIME;
    }
    return hash;
}

static uint32_t ht_fnv1a_str(const char *str, int len) {
    uint32_t hash = HT_FNV_BASIS;
    int i;
    for (i = 0; i < len; i++) {
        hash ^= (unsigned char)str[i];
        hash *= HT_FNV_PRIME;
    }
    return hash;
}

int ht_fnv_put(ht_fnv_t *ht, int key, int value) {
    if (ht->size >= HT_FNV_CAP) return -1;
    uint32_t h = ht_fnv1a_int(key) % HT_FNV_CAP;
    int i;
    for (i = 0; i < HT_FNV_CAP; i++) {
        uint32_t pos = (h + (uint32_t)i) % HT_FNV_CAP;
        if (!ht->occupied[pos]) {
            ht->keys[pos] = key;
            ht->values[pos] = value;
            ht->occupied[pos] = 1;
            ht->size++;
            return 0;
        }
        if (ht->keys[pos] == key) {
            ht->values[pos] = value;
            return 0;
        }
    }
    return -1;
}

int ht_fnv_get(const ht_fnv_t *ht, int key, int *out) {
    uint32_t h = ht_fnv1a_int(key) % HT_FNV_CAP;
    int i;
    for (i = 0; i < HT_FNV_CAP; i++) {
        uint32_t pos = (h + (uint32_t)i) % HT_FNV_CAP;
        if (!ht->occupied[pos]) return -1;
        if (ht->keys[pos] == key) {
            *out = ht->values[pos];
            return 0;
        }
    }
    return -1;
}

int ht_fnv_test(void) {
    ht_fnv_t ht;
    ht_fnv_init(&ht);
    ht_fnv_put(&ht, 99, 990);
    ht_fnv_put(&ht, 227, 2270);
    int v = 0;
    if (ht_fnv_get(&ht, 99, &v) != 0) return -1;
    if (v != 990) return -2;
    uint32_t h = ht_fnv1a_str("hello", 5);
    if (h == 0) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1037: FNV-1a hash should transpile: {:?}", result.err());
}

/// C1038: MurmurHash3 (32-bit) with hash table
#[test]
fn c1038_murmurhash3() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define HT_MM_CAP 128

static uint32_t ht_mm3_rotl32(uint32_t x, int r) {
    return (x << r) | (x >> (32 - r));
}

static uint32_t ht_mm3_fmix32(uint32_t h) {
    h ^= h >> 16;
    h *= 0x85ebca6b;
    h ^= h >> 13;
    h *= 0xc2b2ae35;
    h ^= h >> 16;
    return h;
}

uint32_t ht_murmurhash3_32(const void *key_data, int len, uint32_t seed) {
    const unsigned char *data = (const unsigned char *)key_data;
    int nblocks = len / 4;
    uint32_t h1 = seed;
    uint32_t c1 = 0xcc9e2d51;
    uint32_t c2 = 0x1b873593;
    int i;
    for (i = 0; i < nblocks; i++) {
        uint32_t k1 = (uint32_t)data[i * 4]
                     | ((uint32_t)data[i * 4 + 1] << 8)
                     | ((uint32_t)data[i * 4 + 2] << 16)
                     | ((uint32_t)data[i * 4 + 3] << 24);
        k1 *= c1;
        k1 = ht_mm3_rotl32(k1, 15);
        k1 *= c2;
        h1 ^= k1;
        h1 = ht_mm3_rotl32(h1, 13);
        h1 = h1 * 5 + 0xe6546b64;
    }
    const unsigned char *tail = data + nblocks * 4;
    uint32_t k1 = 0;
    int rem = len & 3;
    if (rem >= 3) k1 ^= (uint32_t)tail[2] << 16;
    if (rem >= 2) k1 ^= (uint32_t)tail[1] << 8;
    if (rem >= 1) {
        k1 ^= (uint32_t)tail[0];
        k1 *= c1;
        k1 = ht_mm3_rotl32(k1, 15);
        k1 *= c2;
        h1 ^= k1;
    }
    h1 ^= (uint32_t)len;
    h1 = ht_mm3_fmix32(h1);
    return h1;
}

typedef struct {
    int keys[HT_MM_CAP];
    int values[HT_MM_CAP];
    int occupied[HT_MM_CAP];
    int size;
} ht_murmur_t;

void ht_murmur_init(ht_murmur_t *ht) {
    int i;
    ht->size = 0;
    for (i = 0; i < HT_MM_CAP; i++) ht->occupied[i] = 0;
}

int ht_murmur_put(ht_murmur_t *ht, int key, int value) {
    uint32_t h = ht_murmurhash3_32(&key, 4, 42) % HT_MM_CAP;
    int i;
    for (i = 0; i < HT_MM_CAP; i++) {
        uint32_t pos = (h + (uint32_t)i) % HT_MM_CAP;
        if (!ht->occupied[pos]) {
            ht->keys[pos] = key;
            ht->values[pos] = value;
            ht->occupied[pos] = 1;
            ht->size++;
            return 0;
        }
        if (ht->keys[pos] == key) {
            ht->values[pos] = value;
            return 0;
        }
    }
    return -1;
}

int ht_murmur_get(const ht_murmur_t *ht, int key, int *out) {
    uint32_t h = ht_murmurhash3_32(&key, 4, 42) % HT_MM_CAP;
    int i;
    for (i = 0; i < HT_MM_CAP; i++) {
        uint32_t pos = (h + (uint32_t)i) % HT_MM_CAP;
        if (!ht->occupied[pos]) return -1;
        if (ht->keys[pos] == key) {
            *out = ht->values[pos];
            return 0;
        }
    }
    return -1;
}

int ht_murmur_test(void) {
    ht_murmur_t ht;
    ht_murmur_init(&ht);
    ht_murmur_put(&ht, 1001, 10010);
    int v = 0;
    if (ht_murmur_get(&ht, 1001, &v) != 0) return -1;
    if (v != 10010) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1038: MurmurHash3 should transpile: {:?}", result.err());
}

/// C1039: CityHash (simplified 32-bit)
#[test]
fn c1039_cityhash() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

static uint32_t ht_city_rotate32(uint32_t val, int shift) {
    return shift == 0 ? val : ((val >> shift) | (val << (32 - shift)));
}

static uint32_t ht_city_mur(uint32_t a, uint32_t h) {
    a *= 0xcc9e2d51u;
    a = ht_city_rotate32(a, 17);
    a *= 0x1b873593u;
    h ^= a;
    h = ht_city_rotate32(h, 19);
    return h * 5 + 0xe6546b64u;
}

static uint32_t ht_city_fetch32(const unsigned char *p) {
    return (uint32_t)p[0]
         | ((uint32_t)p[1] << 8)
         | ((uint32_t)p[2] << 16)
         | ((uint32_t)p[3] << 24);
}

uint32_t ht_cityhash32(const unsigned char *s, int len) {
    if (len <= 4) {
        uint32_t b = 0;
        uint32_t c = 9;
        int i;
        for (i = 0; i < len; i++) {
            b = b * 0xcc9e2d51u + (uint32_t)s[i];
            c ^= b;
        }
        return ht_city_mur(b, ht_city_mur(len, c));
    }
    uint32_t h = (uint32_t)len;
    uint32_t g = (uint32_t)len * 0xcc9e2d51u;
    uint32_t f = g;
    uint32_t a0 = ht_city_rotate32(ht_city_fetch32(s + len - 4) * 0xcc9e2d51u, 17) * 0x1b873593u;
    uint32_t a1 = ht_city_rotate32(ht_city_fetch32(s + len - 8) * 0xcc9e2d51u, 17) * 0x1b873593u;
    f += a0;
    g += a1;
    h = ht_city_mur(a0, h);
    h = ht_city_mur(a1, h);
    h = h * 5 + 0xe6546b64u;
    return h ^ (g >> 3);
}

#define HT_CH_CAP 64

typedef struct {
    int keys[HT_CH_CAP];
    int values[HT_CH_CAP];
    int occupied[HT_CH_CAP];
    int size;
} ht_city_t;

void ht_city_init(ht_city_t *ht) {
    int i;
    ht->size = 0;
    for (i = 0; i < HT_CH_CAP; i++) ht->occupied[i] = 0;
}

int ht_city_put(ht_city_t *ht, int key, int value) {
    uint32_t h = ht_cityhash32((const unsigned char *)&key, 4) % HT_CH_CAP;
    int i;
    for (i = 0; i < HT_CH_CAP; i++) {
        uint32_t pos = (h + (uint32_t)i) % HT_CH_CAP;
        if (!ht->occupied[pos]) {
            ht->keys[pos] = key;
            ht->values[pos] = value;
            ht->occupied[pos] = 1;
            ht->size++;
            return 0;
        }
        if (ht->keys[pos] == key) {
            ht->values[pos] = value;
            return 0;
        }
    }
    return -1;
}

int ht_city_get(const ht_city_t *ht, int key, int *out) {
    uint32_t h = ht_cityhash32((const unsigned char *)&key, 4) % HT_CH_CAP;
    int i;
    for (i = 0; i < HT_CH_CAP; i++) {
        uint32_t pos = (h + (uint32_t)i) % HT_CH_CAP;
        if (!ht->occupied[pos]) return -1;
        if (ht->keys[pos] == key) {
            *out = ht->values[pos];
            return 0;
        }
    }
    return -1;
}

int ht_city_test(void) {
    ht_city_t ht;
    ht_city_init(&ht);
    ht_city_put(&ht, 77, 770);
    int v = 0;
    if (ht_city_get(&ht, 77, &v) != 0) return -1;
    if (v != 770) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1039: CityHash should transpile: {:?}", result.err());
}

/// C1040: SipHash (simplified 2-4)
#[test]
fn c1040_siphash() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned long long uint64_t;

static uint64_t ht_sip_rotl64(uint64_t x, int b) {
    return (x << b) | (x >> (64 - b));
}

static void ht_sip_round(uint64_t *v0, uint64_t *v1, uint64_t *v2, uint64_t *v3) {
    *v0 += *v1;
    *v1 = ht_sip_rotl64(*v1, 13);
    *v1 ^= *v0;
    *v0 = ht_sip_rotl64(*v0, 32);
    *v2 += *v3;
    *v3 = ht_sip_rotl64(*v3, 16);
    *v3 ^= *v2;
    *v0 += *v3;
    *v3 = ht_sip_rotl64(*v3, 21);
    *v3 ^= *v0;
    *v2 += *v1;
    *v1 = ht_sip_rotl64(*v1, 17);
    *v1 ^= *v2;
    *v2 = ht_sip_rotl64(*v2, 32);
}

uint64_t ht_siphash24(const unsigned char *data, int len, uint64_t k0, uint64_t k1) {
    uint64_t v0 = k0 ^ 0x736f6d6570736575ULL;
    uint64_t v1 = k1 ^ 0x646f72616e646f6dULL;
    uint64_t v2 = k0 ^ 0x6c7967656e657261ULL;
    uint64_t v3 = k1 ^ 0x7465646279746573ULL;
    int blocks = len / 8;
    int i;
    for (i = 0; i < blocks; i++) {
        uint64_t m = 0;
        int j;
        for (j = 0; j < 8; j++) {
            m |= ((uint64_t)data[i * 8 + j]) << (j * 8);
        }
        v3 ^= m;
        ht_sip_round(&v0, &v1, &v2, &v3);
        ht_sip_round(&v0, &v1, &v2, &v3);
        v0 ^= m;
    }
    uint64_t last = ((uint64_t)len) << 56;
    int remaining = len & 7;
    int offset = blocks * 8;
    if (remaining >= 7) last |= ((uint64_t)data[offset + 6]) << 48;
    if (remaining >= 6) last |= ((uint64_t)data[offset + 5]) << 40;
    if (remaining >= 5) last |= ((uint64_t)data[offset + 4]) << 32;
    if (remaining >= 4) last |= ((uint64_t)data[offset + 3]) << 24;
    if (remaining >= 3) last |= ((uint64_t)data[offset + 2]) << 16;
    if (remaining >= 2) last |= ((uint64_t)data[offset + 1]) << 8;
    if (remaining >= 1) last |= ((uint64_t)data[offset]);
    v3 ^= last;
    ht_sip_round(&v0, &v1, &v2, &v3);
    ht_sip_round(&v0, &v1, &v2, &v3);
    v0 ^= last;
    v2 ^= 0xff;
    ht_sip_round(&v0, &v1, &v2, &v3);
    ht_sip_round(&v0, &v1, &v2, &v3);
    ht_sip_round(&v0, &v1, &v2, &v3);
    ht_sip_round(&v0, &v1, &v2, &v3);
    return v0 ^ v1 ^ v2 ^ v3;
}

int ht_siphash_test(void) {
    unsigned char msg[4];
    msg[0] = 0x01; msg[1] = 0x02; msg[2] = 0x03; msg[3] = 0x04;
    uint64_t h = ht_siphash24(msg, 4, 0x0706050403020100ULL, 0x0f0e0d0c0b0a0908ULL);
    if (h == 0) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1040: SipHash should transpile: {:?}", result.err());
}

// ============================================================================
// C1041-C1045: Probabilistic & Specialized Structures
// ============================================================================

/// C1041: Zobrist hashing for board game states
#[test]
fn c1041_zobrist_hashing() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define HT_ZOB_BOARD 64
#define HT_ZOB_PIECES 12
#define HT_ZOB_TT_SIZE 1024

typedef struct {
    uint32_t table[HT_ZOB_BOARD][HT_ZOB_PIECES];
    uint32_t side_to_move;
} ht_zobrist_keys_t;

typedef struct {
    uint32_t hash_key;
    int score;
    int depth;
    int valid;
} ht_zobrist_entry_t;

typedef struct {
    ht_zobrist_entry_t entries[HT_ZOB_TT_SIZE];
} ht_zobrist_tt_t;

static uint32_t ht_zob_xorshift(uint32_t *state) {
    uint32_t x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    return x;
}

void ht_zob_init_keys(ht_zobrist_keys_t *zk) {
    uint32_t rng = 12345;
    int sq, pc;
    for (sq = 0; sq < HT_ZOB_BOARD; sq++) {
        for (pc = 0; pc < HT_ZOB_PIECES; pc++) {
            zk->table[sq][pc] = ht_zob_xorshift(&rng);
        }
    }
    zk->side_to_move = ht_zob_xorshift(&rng);
}

void ht_zob_tt_init(ht_zobrist_tt_t *tt) {
    int i;
    for (i = 0; i < HT_ZOB_TT_SIZE; i++) {
        tt->entries[i].valid = 0;
    }
}

uint32_t ht_zob_compute_hash(const ht_zobrist_keys_t *zk, const int *board, int side) {
    uint32_t hash = 0;
    int i;
    for (i = 0; i < HT_ZOB_BOARD; i++) {
        if (board[i] >= 0 && board[i] < HT_ZOB_PIECES) {
            hash ^= zk->table[i][board[i]];
        }
    }
    if (side) hash ^= zk->side_to_move;
    return hash;
}

void ht_zob_tt_store(ht_zobrist_tt_t *tt, uint32_t hash, int score, int depth) {
    int idx = (int)(hash % HT_ZOB_TT_SIZE);
    ht_zobrist_entry_t *e = &tt->entries[idx];
    if (!e->valid || e->depth <= depth) {
        e->hash_key = hash;
        e->score = score;
        e->depth = depth;
        e->valid = 1;
    }
}

int ht_zob_tt_probe(const ht_zobrist_tt_t *tt, uint32_t hash, int depth, int *score) {
    int idx = (int)(hash % HT_ZOB_TT_SIZE);
    const ht_zobrist_entry_t *e = &tt->entries[idx];
    if (e->valid && e->hash_key == hash && e->depth >= depth) {
        *score = e->score;
        return 1;
    }
    return 0;
}

int ht_zobrist_test(void) {
    ht_zobrist_keys_t zk;
    ht_zobrist_tt_t tt;
    ht_zob_init_keys(&zk);
    ht_zob_tt_init(&tt);
    int board[64];
    int i;
    for (i = 0; i < 64; i++) board[i] = -1;
    board[0] = 0;
    board[4] = 6;
    uint32_t h = ht_zob_compute_hash(&zk, board, 0);
    ht_zob_tt_store(&tt, h, 100, 5);
    int score = 0;
    if (!ht_zob_tt_probe(&tt, h, 5, &score)) return -1;
    if (score != 100) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1041: Zobrist hashing should transpile: {:?}", result.err());
}

/// C1042: Consistent hash ring
#[test]
fn c1042_consistent_hash_ring() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define HT_CR_MAX_NODES 32
#define HT_CR_VNODES 4
#define HT_CR_MAX_POINTS (HT_CR_MAX_NODES * HT_CR_VNODES)

typedef struct {
    uint32_t hash;
    int node_id;
} ht_cr_point_t;

typedef struct {
    ht_cr_point_t ring[HT_CR_MAX_POINTS];
    int num_points;
    int num_nodes;
} ht_consistent_ring_t;

static uint32_t ht_cr_hash(int key, int vnode) {
    uint32_t h = (uint32_t)key * 2654435761u + (uint32_t)vnode * 0x9e3779b9u;
    h ^= h >> 16;
    h *= 0x85ebca6b;
    h ^= h >> 13;
    return h;
}

static void ht_cr_sort(ht_cr_point_t *arr, int n) {
    int i, j;
    for (i = 1; i < n; i++) {
        ht_cr_point_t tmp = arr[i];
        j = i - 1;
        while (j >= 0 && arr[j].hash > tmp.hash) {
            arr[j + 1] = arr[j];
            j--;
        }
        arr[j + 1] = tmp;
    }
}

void ht_cr_init(ht_consistent_ring_t *ring) {
    ring->num_points = 0;
    ring->num_nodes = 0;
}

int ht_cr_add_node(ht_consistent_ring_t *ring, int node_id) {
    if (ring->num_points + HT_CR_VNODES > HT_CR_MAX_POINTS) return -1;
    int i;
    for (i = 0; i < HT_CR_VNODES; i++) {
        ring->ring[ring->num_points].hash = ht_cr_hash(node_id, i);
        ring->ring[ring->num_points].node_id = node_id;
        ring->num_points++;
    }
    ring->num_nodes++;
    ht_cr_sort(ring->ring, ring->num_points);
    return 0;
}

int ht_cr_lookup(const ht_consistent_ring_t *ring, int key) {
    if (ring->num_points == 0) return -1;
    uint32_t h = ht_cr_hash(key, 0);
    int lo = 0, hi = ring->num_points - 1;
    int result = 0;
    while (lo <= hi) {
        int mid = lo + (hi - lo) / 2;
        if (ring->ring[mid].hash >= h) {
            result = mid;
            hi = mid - 1;
        } else {
            lo = mid + 1;
        }
    }
    if (lo >= ring->num_points) result = 0;
    return ring->ring[result].node_id;
}

int ht_cr_test(void) {
    ht_consistent_ring_t ring;
    ht_cr_init(&ring);
    ht_cr_add_node(&ring, 1);
    ht_cr_add_node(&ring, 2);
    ht_cr_add_node(&ring, 3);
    int node = ht_cr_lookup(&ring, 42);
    if (node < 1 || node > 3) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1042: Consistent hash ring should transpile: {:?}", result.err());
}

/// C1043: Bloom filter with multiple hash functions
#[test]
fn c1043_bloom_filter() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define HT_BF_BITS 2048
#define HT_BF_BYTES (HT_BF_BITS / 8)
#define HT_BF_K 3

typedef struct {
    uint8_t bits[HT_BF_BYTES];
    int count;
} ht_bloom_t;

void ht_bloom_init(ht_bloom_t *bf) {
    int i;
    bf->count = 0;
    for (i = 0; i < HT_BF_BYTES; i++) {
        bf->bits[i] = 0;
    }
}

static uint32_t ht_bloom_hash(int key, int seed) {
    uint32_t h = (uint32_t)key;
    h += (uint32_t)seed * 0x9e3779b9u;
    h ^= h >> 16;
    h *= 0x85ebca6b;
    h ^= h >> 13;
    h *= 0xc2b2ae35;
    h ^= h >> 16;
    return h % HT_BF_BITS;
}

static void ht_bloom_set_bit(ht_bloom_t *bf, uint32_t pos) {
    bf->bits[pos / 8] |= (uint8_t)(1u << (pos % 8));
}

static int ht_bloom_get_bit(const ht_bloom_t *bf, uint32_t pos) {
    return (bf->bits[pos / 8] >> (pos % 8)) & 1;
}

void ht_bloom_add(ht_bloom_t *bf, int key) {
    int i;
    for (i = 0; i < HT_BF_K; i++) {
        uint32_t pos = ht_bloom_hash(key, i);
        ht_bloom_set_bit(bf, pos);
    }
    bf->count++;
}

int ht_bloom_check(const ht_bloom_t *bf, int key) {
    int i;
    for (i = 0; i < HT_BF_K; i++) {
        uint32_t pos = ht_bloom_hash(key, i);
        if (!ht_bloom_get_bit(bf, pos)) return 0;
    }
    return 1;
}

double ht_bloom_fpr(const ht_bloom_t *bf) {
    int set_bits = 0;
    int i;
    for (i = 0; i < HT_BF_BITS; i++) {
        if (ht_bloom_get_bit(bf, (uint32_t)i)) set_bits++;
    }
    double fill = (double)set_bits / (double)HT_BF_BITS;
    double fpr = 1.0;
    int k;
    for (k = 0; k < HT_BF_K; k++) {
        fpr *= fill;
    }
    return fpr;
}

int ht_bloom_test(void) {
    ht_bloom_t bf;
    ht_bloom_init(&bf);
    ht_bloom_add(&bf, 42);
    ht_bloom_add(&bf, 99);
    ht_bloom_add(&bf, 256);
    if (!ht_bloom_check(&bf, 42)) return -1;
    if (!ht_bloom_check(&bf, 99)) return -2;
    double fpr = ht_bloom_fpr(&bf);
    if (fpr < 0.0 || fpr > 1.0) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1043: Bloom filter should transpile: {:?}", result.err());
}

/// C1044: Counting Bloom filter with decrement support
#[test]
fn c1044_counting_bloom_filter() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define HT_CBF_SLOTS 512
#define HT_CBF_K 3

typedef struct {
    uint8_t counters[HT_CBF_SLOTS];
    int count;
} ht_counting_bloom_t;

void ht_cbf_init(ht_counting_bloom_t *cbf) {
    int i;
    cbf->count = 0;
    for (i = 0; i < HT_CBF_SLOTS; i++) {
        cbf->counters[i] = 0;
    }
}

static uint32_t ht_cbf_hash(int key, int seed) {
    uint32_t h = (uint32_t)key + (uint32_t)seed * 2654435761u;
    h ^= h >> 16;
    h *= 0x45d9f3b;
    h ^= h >> 16;
    return h % HT_CBF_SLOTS;
}

void ht_cbf_add(ht_counting_bloom_t *cbf, int key) {
    int i;
    for (i = 0; i < HT_CBF_K; i++) {
        uint32_t pos = ht_cbf_hash(key, i);
        if (cbf->counters[pos] < 255) {
            cbf->counters[pos]++;
        }
    }
    cbf->count++;
}

int ht_cbf_remove(ht_counting_bloom_t *cbf, int key) {
    int i;
    for (i = 0; i < HT_CBF_K; i++) {
        uint32_t pos = ht_cbf_hash(key, i);
        if (cbf->counters[pos] == 0) return -1;
    }
    for (i = 0; i < HT_CBF_K; i++) {
        uint32_t pos = ht_cbf_hash(key, i);
        cbf->counters[pos]--;
    }
    cbf->count--;
    return 0;
}

int ht_cbf_check(const ht_counting_bloom_t *cbf, int key) {
    int i;
    for (i = 0; i < HT_CBF_K; i++) {
        uint32_t pos = ht_cbf_hash(key, i);
        if (cbf->counters[pos] == 0) return 0;
    }
    return 1;
}

int ht_cbf_test(void) {
    ht_counting_bloom_t cbf;
    ht_cbf_init(&cbf);
    ht_cbf_add(&cbf, 10);
    ht_cbf_add(&cbf, 20);
    if (!ht_cbf_check(&cbf, 10)) return -1;
    if (!ht_cbf_check(&cbf, 20)) return -2;
    ht_cbf_remove(&cbf, 10);
    if (ht_cbf_check(&cbf, 10)) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1044: Counting Bloom filter should transpile: {:?}", result.err());
}

/// C1045: Skip list (probabilistic sorted structure)
#[test]
fn c1045_skip_list() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define HT_SL_MAX_LEVEL 8
#define HT_SL_MAX_NODES 256

typedef struct {
    int key;
    int value;
    int forward[HT_SL_MAX_LEVEL];
} ht_sl_node_t;

typedef struct {
    ht_sl_node_t nodes[HT_SL_MAX_NODES];
    int level;
    int head;
    int free_idx;
    uint32_t rng_state;
} ht_skiplist_t;

void ht_sl_init(ht_skiplist_t *sl) {
    sl->level = 1;
    sl->head = 0;
    sl->free_idx = 1;
    sl->rng_state = 42;
    int j;
    for (j = 0; j < HT_SL_MAX_LEVEL; j++) {
        sl->nodes[0].forward[j] = -1;
    }
    sl->nodes[0].key = -2147483647;
}

static uint32_t ht_sl_rand(ht_skiplist_t *sl) {
    sl->rng_state ^= sl->rng_state << 13;
    sl->rng_state ^= sl->rng_state >> 17;
    sl->rng_state ^= sl->rng_state << 5;
    return sl->rng_state;
}

static int ht_sl_random_level(ht_skiplist_t *sl) {
    int lvl = 1;
    while (lvl < HT_SL_MAX_LEVEL && (ht_sl_rand(sl) & 1)) {
        lvl++;
    }
    return lvl;
}

int ht_sl_insert(ht_skiplist_t *sl, int key, int value) {
    if (sl->free_idx >= HT_SL_MAX_NODES) return -1;
    int update[HT_SL_MAX_LEVEL];
    int cur = sl->head;
    int i;
    for (i = sl->level - 1; i >= 0; i--) {
        while (sl->nodes[cur].forward[i] >= 0 &&
               sl->nodes[sl->nodes[cur].forward[i]].key < key) {
            cur = sl->nodes[cur].forward[i];
        }
        update[i] = cur;
    }
    int next = sl->nodes[cur].forward[0];
    if (next >= 0 && sl->nodes[next].key == key) {
        sl->nodes[next].value = value;
        return 0;
    }
    int new_level = ht_sl_random_level(sl);
    if (new_level > sl->level) {
        for (i = sl->level; i < new_level; i++) {
            update[i] = sl->head;
        }
        sl->level = new_level;
    }
    int new_node = sl->free_idx++;
    sl->nodes[new_node].key = key;
    sl->nodes[new_node].value = value;
    for (i = 0; i < new_level; i++) {
        sl->nodes[new_node].forward[i] = sl->nodes[update[i]].forward[i];
        sl->nodes[update[i]].forward[i] = new_node;
    }
    for (i = new_level; i < HT_SL_MAX_LEVEL; i++) {
        sl->nodes[new_node].forward[i] = -1;
    }
    return 0;
}

int ht_sl_search(const ht_skiplist_t *sl, int key, int *out) {
    int cur = sl->head;
    int i;
    for (i = sl->level - 1; i >= 0; i--) {
        while (sl->nodes[cur].forward[i] >= 0 &&
               sl->nodes[sl->nodes[cur].forward[i]].key < key) {
            cur = sl->nodes[cur].forward[i];
        }
    }
    int next = sl->nodes[cur].forward[0];
    if (next >= 0 && sl->nodes[next].key == key) {
        *out = sl->nodes[next].value;
        return 0;
    }
    return -1;
}

int ht_skiplist_test(void) {
    ht_skiplist_t sl;
    ht_sl_init(&sl);
    ht_sl_insert(&sl, 10, 100);
    ht_sl_insert(&sl, 5, 50);
    ht_sl_insert(&sl, 20, 200);
    int v = 0;
    if (ht_sl_search(&sl, 10, &v) != 0) return -1;
    if (v != 100) return -2;
    if (ht_sl_search(&sl, 5, &v) != 0) return -3;
    if (v != 50) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1045: Skip list should transpile: {:?}", result.err());
}

// ============================================================================
// C1046-C1050: Compound Hash Structures
// ============================================================================

/// C1046: Hash set with membership test
#[test]
fn c1046_hash_set() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define HT_SET_CAP 256

typedef struct {
    int keys[HT_SET_CAP];
    int occupied[HT_SET_CAP];
    int size;
} ht_hashset_t;

void ht_set_init(ht_hashset_t *hs) {
    int i;
    hs->size = 0;
    for (i = 0; i < HT_SET_CAP; i++) {
        hs->occupied[i] = 0;
    }
}

static uint32_t ht_set_hash(int key) {
    uint32_t h = (uint32_t)key;
    h ^= h >> 16;
    h *= 0x45d9f3b;
    h ^= h >> 16;
    return h % HT_SET_CAP;
}

int ht_set_add(ht_hashset_t *hs, int key) {
    if (hs->size >= HT_SET_CAP * 3 / 4) return -1;
    uint32_t idx = ht_set_hash(key);
    int i;
    for (i = 0; i < HT_SET_CAP; i++) {
        uint32_t pos = (idx + (uint32_t)i) % HT_SET_CAP;
        if (!hs->occupied[pos]) {
            hs->keys[pos] = key;
            hs->occupied[pos] = 1;
            hs->size++;
            return 1;
        }
        if (hs->keys[pos] == key) return 0;
    }
    return -1;
}

int ht_set_contains(const ht_hashset_t *hs, int key) {
    uint32_t idx = ht_set_hash(key);
    int i;
    for (i = 0; i < HT_SET_CAP; i++) {
        uint32_t pos = (idx + (uint32_t)i) % HT_SET_CAP;
        if (!hs->occupied[pos]) return 0;
        if (hs->keys[pos] == key) return 1;
    }
    return 0;
}

int ht_set_remove(ht_hashset_t *hs, int key) {
    uint32_t idx = ht_set_hash(key);
    int i;
    for (i = 0; i < HT_SET_CAP; i++) {
        uint32_t pos = (idx + (uint32_t)i) % HT_SET_CAP;
        if (!hs->occupied[pos]) return 0;
        if (hs->keys[pos] == key) {
            hs->occupied[pos] = 0;
            hs->size--;
            return 1;
        }
    }
    return 0;
}

int ht_set_union(const ht_hashset_t *a, const ht_hashset_t *b, ht_hashset_t *out) {
    ht_set_init(out);
    int i;
    for (i = 0; i < HT_SET_CAP; i++) {
        if (a->occupied[i]) ht_set_add(out, a->keys[i]);
    }
    for (i = 0; i < HT_SET_CAP; i++) {
        if (b->occupied[i]) ht_set_add(out, b->keys[i]);
    }
    return out->size;
}

int ht_set_intersection(const ht_hashset_t *a, const ht_hashset_t *b, ht_hashset_t *out) {
    ht_set_init(out);
    int i;
    for (i = 0; i < HT_SET_CAP; i++) {
        if (a->occupied[i] && ht_set_contains(b, a->keys[i])) {
            ht_set_add(out, a->keys[i]);
        }
    }
    return out->size;
}

int ht_hashset_test(void) {
    ht_hashset_t a, b, result;
    ht_set_init(&a);
    ht_set_init(&b);
    ht_set_add(&a, 1);
    ht_set_add(&a, 2);
    ht_set_add(&a, 3);
    ht_set_add(&b, 2);
    ht_set_add(&b, 3);
    ht_set_add(&b, 4);
    if (!ht_set_contains(&a, 1)) return -1;
    if (ht_set_contains(&a, 4)) return -2;
    ht_set_intersection(&a, &b, &result);
    if (result.size != 2) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1046: Hash set should transpile: {:?}", result.err());
}

/// C1047: Multi-map (key -> multiple values)
#[test]
fn c1047_multimap() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define HT_MM_BUCKETS 64
#define HT_MM_MAX_VALS 8

typedef struct {
    int key;
    int values[HT_MM_MAX_VALS];
    int val_count;
    int occupied;
} ht_mm_entry_t;

typedef struct {
    ht_mm_entry_t entries[HT_MM_BUCKETS];
    int size;
} ht_multimap_t;

void ht_mm_init(ht_multimap_t *mm) {
    int i;
    mm->size = 0;
    for (i = 0; i < HT_MM_BUCKETS; i++) {
        mm->entries[i].occupied = 0;
        mm->entries[i].val_count = 0;
    }
}

static uint32_t ht_mm_hash(int key) {
    uint32_t h = (uint32_t)key;
    h ^= h >> 16;
    h *= 0x85ebca6b;
    return h % HT_MM_BUCKETS;
}

static int ht_mm_find_slot(const ht_multimap_t *mm, int key) {
    uint32_t h = ht_mm_hash(key);
    int i;
    for (i = 0; i < HT_MM_BUCKETS; i++) {
        int pos = (int)((h + (uint32_t)i) % HT_MM_BUCKETS);
        if (!mm->entries[pos].occupied) return -(pos + 1);
        if (mm->entries[pos].key == key) return pos;
    }
    return -1;
}

int ht_mm_add(ht_multimap_t *mm, int key, int value) {
    int slot = ht_mm_find_slot(mm, key);
    if (slot >= 0) {
        ht_mm_entry_t *e = &mm->entries[slot];
        if (e->val_count >= HT_MM_MAX_VALS) return -1;
        e->values[e->val_count++] = value;
        return 0;
    }
    int new_slot = -(slot + 1);
    if (new_slot < 0 || new_slot >= HT_MM_BUCKETS) return -1;
    ht_mm_entry_t *e = &mm->entries[new_slot];
    e->key = key;
    e->values[0] = value;
    e->val_count = 1;
    e->occupied = 1;
    mm->size++;
    return 0;
}

int ht_mm_get_values(const ht_multimap_t *mm, int key, int *out, int max_out) {
    int slot = ht_mm_find_slot(mm, key);
    if (slot < 0) return 0;
    const ht_mm_entry_t *e = &mm->entries[slot];
    int count = e->val_count < max_out ? e->val_count : max_out;
    int i;
    for (i = 0; i < count; i++) {
        out[i] = e->values[i];
    }
    return count;
}

int ht_mm_count(const ht_multimap_t *mm, int key) {
    int slot = ht_mm_find_slot(mm, key);
    if (slot < 0) return 0;
    return mm->entries[slot].val_count;
}

int ht_multimap_test(void) {
    ht_multimap_t mm;
    ht_mm_init(&mm);
    ht_mm_add(&mm, 1, 10);
    ht_mm_add(&mm, 1, 20);
    ht_mm_add(&mm, 1, 30);
    ht_mm_add(&mm, 2, 100);
    if (ht_mm_count(&mm, 1) != 3) return -1;
    if (ht_mm_count(&mm, 2) != 1) return -2;
    int vals[8];
    int n = ht_mm_get_values(&mm, 1, vals, 8);
    if (n != 3) return -3;
    if (vals[0] != 10 || vals[1] != 20) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1047: Multi-map should transpile: {:?}", result.err());
}

/// C1048: Ordered map (hash table + linked list for insertion order)
#[test]
fn c1048_ordered_map() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define HT_OM_CAP 128

typedef struct {
    int key;
    int value;
    int next_order;
    int prev_order;
    int occupied;
} ht_om_entry_t;

typedef struct {
    ht_om_entry_t entries[HT_OM_CAP];
    int head;
    int tail;
    int size;
} ht_ordered_map_t;

void ht_om_init(ht_ordered_map_t *om) {
    int i;
    om->head = -1;
    om->tail = -1;
    om->size = 0;
    for (i = 0; i < HT_OM_CAP; i++) {
        om->entries[i].occupied = 0;
        om->entries[i].next_order = -1;
        om->entries[i].prev_order = -1;
    }
}

static uint32_t ht_om_hash(int key) {
    uint32_t h = (uint32_t)key;
    h ^= h >> 16;
    h *= 0x45d9f3b;
    return h % HT_OM_CAP;
}

int ht_om_put(ht_ordered_map_t *om, int key, int value) {
    uint32_t h = ht_om_hash(key);
    int i;
    for (i = 0; i < HT_OM_CAP; i++) {
        int pos = (int)((h + (uint32_t)i) % HT_OM_CAP);
        if (!om->entries[pos].occupied) {
            om->entries[pos].key = key;
            om->entries[pos].value = value;
            om->entries[pos].occupied = 1;
            om->entries[pos].next_order = -1;
            om->entries[pos].prev_order = om->tail;
            if (om->tail >= 0) {
                om->entries[om->tail].next_order = pos;
            }
            om->tail = pos;
            if (om->head < 0) om->head = pos;
            om->size++;
            return 0;
        }
        if (om->entries[pos].key == key) {
            om->entries[pos].value = value;
            return 0;
        }
    }
    return -1;
}

int ht_om_get(const ht_ordered_map_t *om, int key, int *out) {
    uint32_t h = ht_om_hash(key);
    int i;
    for (i = 0; i < HT_OM_CAP; i++) {
        int pos = (int)((h + (uint32_t)i) % HT_OM_CAP);
        if (!om->entries[pos].occupied) return -1;
        if (om->entries[pos].key == key) {
            *out = om->entries[pos].value;
            return 0;
        }
    }
    return -1;
}

int ht_om_iterate_order(const ht_ordered_map_t *om, int *keys_out, int *vals_out, int max) {
    int cur = om->head;
    int count = 0;
    while (cur >= 0 && count < max) {
        keys_out[count] = om->entries[cur].key;
        vals_out[count] = om->entries[cur].value;
        count++;
        cur = om->entries[cur].next_order;
    }
    return count;
}

int ht_ordered_map_test(void) {
    ht_ordered_map_t om;
    ht_om_init(&om);
    ht_om_put(&om, 30, 300);
    ht_om_put(&om, 10, 100);
    ht_om_put(&om, 20, 200);
    int keys[3], vals[3];
    int n = ht_om_iterate_order(&om, keys, vals, 3);
    if (n != 3) return -1;
    if (keys[0] != 30) return -2;
    if (keys[1] != 10) return -3;
    if (keys[2] != 20) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1048: Ordered map should transpile: {:?}", result.err());
}

/// C1049: Coalesced hashing (chaining within the table)
#[test]
fn c1049_coalesced_hashing() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define HT_CO_CAP 128
#define HT_CO_CELLAR 32
#define HT_CO_ADDR (HT_CO_CAP - HT_CO_CELLAR)

typedef struct {
    int key;
    int value;
    int next;
    int occupied;
} ht_co_slot_t;

typedef struct {
    ht_co_slot_t slots[HT_CO_CAP];
    int cellar_free;
    int size;
} ht_coalesced_t;

void ht_co_init(ht_coalesced_t *ht) {
    int i;
    ht->size = 0;
    ht->cellar_free = HT_CO_CAP - 1;
    for (i = 0; i < HT_CO_CAP; i++) {
        ht->slots[i].occupied = 0;
        ht->slots[i].next = -1;
    }
}

static uint32_t ht_co_hash(int key) {
    uint32_t h = (uint32_t)(key < 0 ? -key : key);
    h ^= h >> 16;
    h *= 0x45d9f3b;
    return h % HT_CO_ADDR;
}

static int ht_co_alloc_cellar(ht_coalesced_t *ht) {
    while (ht->cellar_free >= HT_CO_ADDR) {
        if (!ht->slots[ht->cellar_free].occupied) {
            return ht->cellar_free;
        }
        ht->cellar_free--;
    }
    return -1;
}

int ht_co_insert(ht_coalesced_t *ht, int key, int value) {
    uint32_t h = ht_co_hash(key);
    int pos = (int)h;
    if (!ht->slots[pos].occupied) {
        ht->slots[pos].key = key;
        ht->slots[pos].value = value;
        ht->slots[pos].occupied = 1;
        ht->slots[pos].next = -1;
        ht->size++;
        return 0;
    }
    int cur = pos;
    while (cur >= 0) {
        if (ht->slots[cur].key == key) {
            ht->slots[cur].value = value;
            return 0;
        }
        if (ht->slots[cur].next < 0) break;
        cur = ht->slots[cur].next;
    }
    int new_slot = ht_co_alloc_cellar(ht);
    if (new_slot < 0) return -1;
    ht->slots[new_slot].key = key;
    ht->slots[new_slot].value = value;
    ht->slots[new_slot].occupied = 1;
    ht->slots[new_slot].next = -1;
    ht->slots[cur].next = new_slot;
    ht->size++;
    return 0;
}

int ht_co_find(const ht_coalesced_t *ht, int key, int *out) {
    uint32_t h = ht_co_hash(key);
    int cur = (int)h;
    while (cur >= 0) {
        if (ht->slots[cur].occupied && ht->slots[cur].key == key) {
            *out = ht->slots[cur].value;
            return 0;
        }
        cur = ht->slots[cur].next;
    }
    return -1;
}

int ht_coalesced_test(void) {
    ht_coalesced_t ht;
    ht_co_init(&ht);
    ht_co_insert(&ht, 5, 50);
    ht_co_insert(&ht, 101, 1010);
    ht_co_insert(&ht, 197, 1970);
    int v = 0;
    if (ht_co_find(&ht, 5, &v) != 0) return -1;
    if (v != 50) return -2;
    if (ht_co_find(&ht, 197, &v) != 0) return -3;
    if (v != 1970) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1049: Coalesced hashing should transpile: {:?}", result.err());
}

/// C1050: Hash join (database-style equi-join)
#[test]
fn c1050_hash_join() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define HT_HJ_BUILD_CAP 128
#define HT_HJ_MAX_RESULTS 256

typedef struct {
    int key;
    int payload_a;
} ht_hj_row_a_t;

typedef struct {
    int key;
    int payload_b;
} ht_hj_row_b_t;

typedef struct {
    int key;
    int payload_a;
    int payload_b;
} ht_hj_result_t;

typedef struct {
    int keys[HT_HJ_BUILD_CAP];
    int payloads[HT_HJ_BUILD_CAP];
    int occupied[HT_HJ_BUILD_CAP];
    int next[HT_HJ_BUILD_CAP];
    int size;
} ht_hj_build_t;

static uint32_t ht_hj_hash(int key) {
    uint32_t h = (uint32_t)key;
    h ^= h >> 16;
    h *= 0x85ebca6b;
    h ^= h >> 13;
    return h % HT_HJ_BUILD_CAP;
}

void ht_hj_build_init(ht_hj_build_t *bt) {
    int i;
    bt->size = 0;
    for (i = 0; i < HT_HJ_BUILD_CAP; i++) {
        bt->occupied[i] = 0;
        bt->next[i] = -1;
    }
}

int ht_hj_build_insert(ht_hj_build_t *bt, int key, int payload) {
    if (bt->size >= HT_HJ_BUILD_CAP) return -1;
    uint32_t h = ht_hj_hash(key);
    int i;
    for (i = 0; i < HT_HJ_BUILD_CAP; i++) {
        int pos = (int)((h + (uint32_t)i) % HT_HJ_BUILD_CAP);
        if (!bt->occupied[pos]) {
            bt->keys[pos] = key;
            bt->payloads[pos] = payload;
            bt->occupied[pos] = 1;
            bt->size++;
            return 0;
        }
    }
    return -1;
}

int ht_hj_probe(const ht_hj_build_t *bt, int key, int *out_payload) {
    uint32_t h = ht_hj_hash(key);
    int i;
    for (i = 0; i < HT_HJ_BUILD_CAP; i++) {
        int pos = (int)((h + (uint32_t)i) % HT_HJ_BUILD_CAP);
        if (!bt->occupied[pos]) return -1;
        if (bt->keys[pos] == key) {
            *out_payload = bt->payloads[pos];
            return 0;
        }
    }
    return -1;
}

int ht_hash_join(const ht_hj_row_a_t *table_a, int na,
                 const ht_hj_row_b_t *table_b, int nb,
                 ht_hj_result_t *results, int max_results) {
    ht_hj_build_t build;
    ht_hj_build_init(&build);
    int i;
    for (i = 0; i < na; i++) {
        ht_hj_build_insert(&build, table_a[i].key, table_a[i].payload_a);
    }
    int count = 0;
    for (i = 0; i < nb; i++) {
        int payload_a;
        if (ht_hj_probe(&build, table_b[i].key, &payload_a) == 0) {
            if (count < max_results) {
                results[count].key = table_b[i].key;
                results[count].payload_a = payload_a;
                results[count].payload_b = table_b[i].payload_b;
                count++;
            }
        }
    }
    return count;
}

int ht_hash_join_test(void) {
    ht_hj_row_a_t a[4];
    a[0].key = 1; a[0].payload_a = 10;
    a[1].key = 2; a[1].payload_a = 20;
    a[2].key = 3; a[2].payload_a = 30;
    a[3].key = 4; a[3].payload_a = 40;

    ht_hj_row_b_t b[3];
    b[0].key = 2; b[0].payload_b = 200;
    b[1].key = 4; b[1].payload_b = 400;
    b[2].key = 5; b[2].payload_b = 500;

    ht_hj_result_t results[8];
    int n = ht_hash_join(a, 4, b, 3, results, 8);
    if (n != 2) return -1;
    if (results[0].key != 2) return -2;
    if (results[0].payload_a != 20 || results[0].payload_b != 200) return -3;
    if (results[1].key != 4) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1050: Hash join should transpile: {:?}", result.err());
}
