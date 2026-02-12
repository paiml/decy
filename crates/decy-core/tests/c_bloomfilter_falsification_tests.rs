//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1876-C1900: Bloom Filter and Probabilistic Data Structure implementations --
//! the kind of C code found in database engines, network monitoring tools,
//! caching systems, and analytics pipelines.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world probabilistic data structure patterns commonly
//! found in LevelDB, Redis, Cassandra, and similar systems --
//! all expressed as valid C99 without #include directives.
//!
//! Organization:
//! - C1876-C1880: Basic bloom filter (bit array, hash functions, insert, query, false positive rate)
//! - C1881-C1885: Counting bloom filter (counter array, insert/delete, counting, overflow check)
//! - C1886-C1890: Scalable bloom filter (multi-level, dynamic resize, capacity estimation, merge)
//! - C1891-C1895: Cuckoo filter (bucket array, fingerprint, insert/lookup, relocation, delete)
//! - C1896-C1900: HyperLogLog (register array, hash, add element, cardinality estimate, merge)
//!
//! Results: 25 passing, 0 falsified (100% pass rate — 13 un-falsified by S2 for(;;) fix)

// ============================================================================
// C1876-C1880: Basic Bloom Filter
// ============================================================================

/// C1876: Bloom filter bit array with multiple hash functions
#[test]
// UN-FALSIFIED: for(;;) fix (S2) resolved HIR panic on computed #define expressions
fn c1876_bloom_filter_bit_array() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_BITS 1024
#define BF_BYTES (BF_BITS / 8)
#define BF_NUM_HASHES 3

typedef struct {
    uint8_t bits[BF_BYTES];
    uint32_t count;
} bf_bloom_t;

void bf_init(bf_bloom_t *bf) {
    uint32_t i;
    for (i = 0; i < BF_BYTES; i++) {
        bf->bits[i] = 0;
    }
    bf->count = 0;
}

static uint32_t bf_hash1(uint32_t key) {
    key = ((key >> 16) ^ key) * 0x45d9f3b;
    key = ((key >> 16) ^ key) * 0x45d9f3b;
    key = (key >> 16) ^ key;
    return key % BF_BITS;
}

static uint32_t bf_hash2(uint32_t key) {
    key = (key ^ 0xdeadbeef) * 0x01000193;
    key = key ^ (key >> 15);
    key = key * 0x735a2d97;
    return key % BF_BITS;
}

static uint32_t bf_hash3(uint32_t key) {
    key = key * 2654435761u;
    key = key ^ (key >> 17);
    key = key * 0xed5ad4bb;
    return key % BF_BITS;
}

void bf_set_bit(bf_bloom_t *bf, uint32_t pos) {
    bf->bits[pos / 8] |= (1 << (pos % 8));
}

int bf_get_bit(const bf_bloom_t *bf, uint32_t pos) {
    return (bf->bits[pos / 8] >> (pos % 8)) & 1;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1876: Bloom filter bit array should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1876: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1876: Should contain bf_ functions");
}

/// C1877: Bloom filter insert operation
#[test]
// UN-FALSIFIED: for(;;) fix (S2) resolved HIR panic on computed #define expressions
fn c1877_bloom_filter_insert() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_INS_BITS 2048
#define BF_INS_BYTES (BF_INS_BITS / 8)

typedef struct {
    uint8_t bits[BF_INS_BYTES];
    uint32_t count;
    uint32_t num_hashes;
} bf_insert_t;

void bf_insert_init(bf_insert_t *bf, uint32_t num_hashes) {
    uint32_t i;
    for (i = 0; i < BF_INS_BYTES; i++) {
        bf->bits[i] = 0;
    }
    bf->count = 0;
    bf->num_hashes = num_hashes;
}

static uint32_t bf_insert_hash(uint32_t key, uint32_t seed) {
    uint32_t h = key;
    h ^= seed;
    h = (h ^ (h >> 16)) * 0x85ebca6b;
    h = (h ^ (h >> 13)) * 0xc2b2ae35;
    h = h ^ (h >> 16);
    return h % BF_INS_BITS;
}

void bf_insert_add(bf_insert_t *bf, uint32_t key) {
    uint32_t i;
    for (i = 0; i < bf->num_hashes; i++) {
        uint32_t pos = bf_insert_hash(key, i * 0x9e3779b9);
        bf->bits[pos / 8] |= (uint8_t)(1 << (pos % 8));
    }
    bf->count++;
}

int bf_insert_contains(const bf_insert_t *bf, uint32_t key) {
    uint32_t i;
    for (i = 0; i < bf->num_hashes; i++) {
        uint32_t pos = bf_insert_hash(key, i * 0x9e3779b9);
        if (((bf->bits[pos / 8] >> (pos % 8)) & 1) == 0) {
            return 0;
        }
    }
    return 1;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1877: Bloom filter insert should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1877: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1877: Should contain bf_ functions");
}

/// C1878: Bloom filter query with false positive awareness
#[test]
// UN-FALSIFIED: for(;;) fix (S2) resolved HIR panic on computed #define expressions
fn c1878_bloom_filter_query() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_Q_SIZE 4096
#define BF_Q_BYTES (BF_Q_SIZE / 8)
#define BF_Q_HASHES 5

typedef struct {
    uint8_t bits[BF_Q_BYTES];
    uint32_t inserted;
} bf_query_t;

void bf_query_init(bf_query_t *bf) {
    uint32_t i;
    for (i = 0; i < BF_Q_BYTES; i++) {
        bf->bits[i] = 0;
    }
    bf->inserted = 0;
}

static uint32_t bf_query_hash(uint32_t key, uint32_t idx) {
    uint32_t h = key + idx * 2654435761u;
    h ^= h >> 16;
    h *= 0x85ebca6b;
    h ^= h >> 13;
    return h % BF_Q_SIZE;
}

void bf_query_add(bf_query_t *bf, uint32_t key) {
    uint32_t i;
    for (i = 0; i < BF_Q_HASHES; i++) {
        uint32_t pos = bf_query_hash(key, i);
        bf->bits[pos / 8] |= (uint8_t)(1 << (pos % 8));
    }
    bf->inserted++;
}

int bf_query_check(const bf_query_t *bf, uint32_t key) {
    uint32_t i;
    for (i = 0; i < BF_Q_HASHES; i++) {
        uint32_t pos = bf_query_hash(key, i);
        if (((bf->bits[pos / 8] >> (pos % 8)) & 1) == 0) {
            return 0;
        }
    }
    return 1;
}

int bf_query_test(void) {
    bf_query_t bf;
    bf_query_init(&bf);
    bf_query_add(&bf, 42);
    bf_query_add(&bf, 100);
    if (!bf_query_check(&bf, 42)) return -1;
    if (!bf_query_check(&bf, 100)) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1878: Bloom filter query should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1878: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1878: Should contain bf_ functions");
}

/// C1879: Bloom filter false positive rate estimation
#[test]
// UN-FALSIFIED: for(;;) fix (S2) resolved HIR panic on computed #define expressions
fn c1879_bloom_filter_false_positive_rate() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_FPR_BITS 8192
#define BF_FPR_BYTES (BF_FPR_BITS / 8)
#define BF_FPR_HASHES 7

typedef struct {
    uint8_t bits[BF_FPR_BYTES];
    uint32_t inserted;
    uint32_t bit_count;
} bf_fpr_t;

void bf_fpr_init(bf_fpr_t *bf) {
    uint32_t i;
    for (i = 0; i < BF_FPR_BYTES; i++) {
        bf->bits[i] = 0;
    }
    bf->inserted = 0;
    bf->bit_count = 0;
}

static uint32_t bf_fpr_hash(uint32_t key, uint32_t seed) {
    uint32_t h = key ^ seed;
    h = (h ^ (h >> 16)) * 0x45d9f3b;
    h = (h ^ (h >> 16)) * 0x45d9f3b;
    h = h ^ (h >> 16);
    return h % BF_FPR_BITS;
}

void bf_fpr_add(bf_fpr_t *bf, uint32_t key) {
    uint32_t i;
    for (i = 0; i < BF_FPR_HASHES; i++) {
        uint32_t pos = bf_fpr_hash(key, i * 0x517cc1b7);
        if (((bf->bits[pos / 8] >> (pos % 8)) & 1) == 0) {
            bf->bits[pos / 8] |= (uint8_t)(1 << (pos % 8));
            bf->bit_count++;
        }
    }
    bf->inserted++;
}

uint32_t bf_fpr_popcount(const bf_fpr_t *bf) {
    uint32_t count = 0;
    uint32_t i;
    for (i = 0; i < BF_FPR_BYTES; i++) {
        uint8_t b = bf->bits[i];
        while (b) {
            count += b & 1;
            b >>= 1;
        }
    }
    return count;
}

int bf_fpr_estimate_full(const bf_fpr_t *bf) {
    uint32_t set_bits = bf_fpr_popcount(bf);
    uint32_t threshold = BF_FPR_BITS / 2;
    return set_bits > threshold ? 1 : 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1879: Bloom filter FPR estimation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1879: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1879: Should contain bf_ functions");
}

/// C1880: Bloom filter optimal parameters calculation
#[test]
fn c1880_bloom_filter_optimal_params() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_OPT_MAX_BITS 65536
#define BF_OPT_MAX_HASHES 20

typedef struct {
    uint32_t num_bits;
    uint32_t num_hashes;
    uint32_t expected_items;
} bf_opt_params_t;

uint32_t bf_opt_calc_bits(uint32_t n, uint32_t target_fpr_inv) {
    uint32_t m = n * 10;
    if (target_fpr_inv > 100) {
        m = n * 15;
    }
    if (target_fpr_inv > 1000) {
        m = n * 20;
    }
    if (m > BF_OPT_MAX_BITS) {
        m = BF_OPT_MAX_BITS;
    }
    return m;
}

uint32_t bf_opt_calc_hashes(uint32_t m, uint32_t n) {
    uint32_t k;
    if (n == 0) return 1;
    k = (m * 7) / (n * 10);
    if (k < 1) k = 1;
    if (k > BF_OPT_MAX_HASHES) k = BF_OPT_MAX_HASHES;
    return k;
}

void bf_opt_compute(bf_opt_params_t *params, uint32_t n, uint32_t fpr_inv) {
    params->expected_items = n;
    params->num_bits = bf_opt_calc_bits(n, fpr_inv);
    params->num_hashes = bf_opt_calc_hashes(params->num_bits, n);
}

int bf_opt_test(void) {
    bf_opt_params_t params;
    bf_opt_compute(&params, 1000, 100);
    if (params.num_bits < 1000) return -1;
    if (params.num_hashes < 1) return -2;
    if (params.num_hashes > BF_OPT_MAX_HASHES) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1880: Bloom filter optimal params should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1880: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1880: Should contain bf_ functions");
}

// ============================================================================
// C1881-C1885: Counting Bloom Filter
// ============================================================================

/// C1881: Counting bloom filter with counter array
#[test]
fn c1881_counting_bloom_counter_array() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_CBF_SIZE 512
#define BF_CBF_HASHES 4
#define BF_CBF_MAX_COUNT 255

typedef struct {
    uint8_t counters[BF_CBF_SIZE];
    uint32_t num_items;
} bf_counting_t;

void bf_counting_init(bf_counting_t *cbf) {
    uint32_t i;
    for (i = 0; i < BF_CBF_SIZE; i++) {
        cbf->counters[i] = 0;
    }
    cbf->num_items = 0;
}

static uint32_t bf_counting_hash(uint32_t key, uint32_t seed) {
    uint32_t h = key ^ seed;
    h = (h ^ (h >> 16)) * 0x85ebca6b;
    h = (h ^ (h >> 13)) * 0xc2b2ae35;
    h = h ^ (h >> 16);
    return h % BF_CBF_SIZE;
}

int bf_counting_add(bf_counting_t *cbf, uint32_t key) {
    uint32_t i;
    for (i = 0; i < BF_CBF_HASHES; i++) {
        uint32_t pos = bf_counting_hash(key, i * 0x9e3779b9);
        if (cbf->counters[pos] < BF_CBF_MAX_COUNT) {
            cbf->counters[pos]++;
        } else {
            return -1;
        }
    }
    cbf->num_items++;
    return 0;
}

int bf_counting_query(const bf_counting_t *cbf, uint32_t key) {
    uint32_t i;
    for (i = 0; i < BF_CBF_HASHES; i++) {
        uint32_t pos = bf_counting_hash(key, i * 0x9e3779b9);
        if (cbf->counters[pos] == 0) {
            return 0;
        }
    }
    return 1;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1881: Counting bloom counter array should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1881: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1881: Should contain bf_ functions");
}

/// C1882: Counting bloom filter delete operation
#[test]
fn c1882_counting_bloom_delete() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_CBD_SIZE 512
#define BF_CBD_HASHES 4

typedef struct {
    uint8_t counters[BF_CBD_SIZE];
    uint32_t num_items;
} bf_cbf_del_t;

void bf_cbf_del_init(bf_cbf_del_t *cbf) {
    uint32_t i;
    for (i = 0; i < BF_CBD_SIZE; i++) {
        cbf->counters[i] = 0;
    }
    cbf->num_items = 0;
}

static uint32_t bf_cbf_del_hash(uint32_t key, uint32_t seed) {
    uint32_t h = key ^ seed;
    h *= 0x01000193;
    h ^= h >> 15;
    h *= 0x735a2d97;
    return h % BF_CBD_SIZE;
}

void bf_cbf_del_add(bf_cbf_del_t *cbf, uint32_t key) {
    uint32_t i;
    for (i = 0; i < BF_CBD_HASHES; i++) {
        uint32_t pos = bf_cbf_del_hash(key, i * 0xcc9e2d51);
        if (cbf->counters[pos] < 255) {
            cbf->counters[pos]++;
        }
    }
    cbf->num_items++;
}

int bf_cbf_del_remove(bf_cbf_del_t *cbf, uint32_t key) {
    uint32_t i;
    for (i = 0; i < BF_CBD_HASHES; i++) {
        uint32_t pos = bf_cbf_del_hash(key, i * 0xcc9e2d51);
        if (cbf->counters[pos] == 0) {
            return -1;
        }
    }
    for (i = 0; i < BF_CBD_HASHES; i++) {
        uint32_t pos = bf_cbf_del_hash(key, i * 0xcc9e2d51);
        cbf->counters[pos]--;
    }
    cbf->num_items--;
    return 0;
}

int bf_cbf_del_test(void) {
    bf_cbf_del_t cbf;
    bf_cbf_del_init(&cbf);
    bf_cbf_del_add(&cbf, 42);
    bf_cbf_del_add(&cbf, 99);
    if (bf_cbf_del_remove(&cbf, 42) != 0) return -1;
    if (cbf.num_items != 1) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1882: Counting bloom delete should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1882: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1882: Should contain bf_ functions");
}

/// C1883: Counting bloom filter frequency estimation
#[test]
fn c1883_counting_bloom_frequency() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_FREQ_SIZE 1024
#define BF_FREQ_HASHES 4

typedef struct {
    uint8_t counters[BF_FREQ_SIZE];
    uint32_t total_inserts;
} bf_freq_t;

void bf_freq_init(bf_freq_t *f) {
    uint32_t i;
    for (i = 0; i < BF_FREQ_SIZE; i++) {
        f->counters[i] = 0;
    }
    f->total_inserts = 0;
}

static uint32_t bf_freq_hash(uint32_t key, uint32_t seed) {
    uint32_t h = key + seed;
    h ^= h >> 16;
    h *= 0x85ebca6b;
    h ^= h >> 13;
    h *= 0xc2b2ae35;
    h ^= h >> 16;
    return h % BF_FREQ_SIZE;
}

void bf_freq_add(bf_freq_t *f, uint32_t key) {
    uint32_t i;
    for (i = 0; i < BF_FREQ_HASHES; i++) {
        uint32_t pos = bf_freq_hash(key, i * 0x9e3779b9);
        if (f->counters[pos] < 255) {
            f->counters[pos]++;
        }
    }
    f->total_inserts++;
}

uint32_t bf_freq_estimate(const bf_freq_t *f, uint32_t key) {
    uint32_t min_count = 255;
    uint32_t i;
    for (i = 0; i < BF_FREQ_HASHES; i++) {
        uint32_t pos = bf_freq_hash(key, i * 0x9e3779b9);
        if (f->counters[pos] < min_count) {
            min_count = f->counters[pos];
        }
    }
    return min_count;
}

int bf_freq_test(void) {
    bf_freq_t f;
    bf_freq_init(&f);
    bf_freq_add(&f, 10);
    bf_freq_add(&f, 10);
    bf_freq_add(&f, 10);
    uint32_t est = bf_freq_estimate(&f, 10);
    if (est < 3) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1883: Counting bloom frequency should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1883: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1883: Should contain bf_ functions");
}

/// C1884: Counting bloom filter overflow detection
#[test]
fn c1884_counting_bloom_overflow() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

#define BF_OVF_SIZE 256
#define BF_OVF_HASHES 3
#define BF_OVF_THRESHOLD 250

typedef struct {
    uint8_t counters[BF_OVF_SIZE];
    uint32_t overflow_count;
    uint32_t insert_count;
} bf_overflow_t;

void bf_overflow_init(bf_overflow_t *bf) {
    uint32_t i;
    for (i = 0; i < BF_OVF_SIZE; i++) {
        bf->counters[i] = 0;
    }
    bf->overflow_count = 0;
    bf->insert_count = 0;
}

static uint32_t bf_overflow_hash(uint32_t key, uint32_t seed) {
    uint32_t h = key ^ seed;
    h = (h * 0x01000193) ^ (h >> 11);
    h = (h * 0x735a2d97) ^ (h >> 17);
    return h % BF_OVF_SIZE;
}

int bf_overflow_add(bf_overflow_t *bf, uint32_t key) {
    int overflowed = 0;
    uint32_t i;
    for (i = 0; i < BF_OVF_HASHES; i++) {
        uint32_t pos = bf_overflow_hash(key, i * 0xdeadbeef);
        if (bf->counters[pos] >= BF_OVF_THRESHOLD) {
            overflowed = 1;
        }
        if (bf->counters[pos] < 255) {
            bf->counters[pos]++;
        }
    }
    bf->insert_count++;
    if (overflowed) {
        bf->overflow_count++;
    }
    return overflowed;
}

int bf_overflow_is_saturated(const bf_overflow_t *bf) {
    uint32_t saturated = 0;
    uint32_t i;
    for (i = 0; i < BF_OVF_SIZE; i++) {
        if (bf->counters[i] >= BF_OVF_THRESHOLD) {
            saturated++;
        }
    }
    return saturated > (BF_OVF_SIZE / 4) ? 1 : 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1884: Counting bloom overflow should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1884: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1884: Should contain bf_ functions");
}

/// C1885: Counting bloom filter with wide counters
#[test]
fn c1885_counting_bloom_wide_counters() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;

#define BF_WIDE_SIZE 256
#define BF_WIDE_HASHES 4
#define BF_WIDE_MAX 65535

typedef struct {
    uint16_t counters[BF_WIDE_SIZE];
    uint32_t num_items;
    uint32_t total_ops;
} bf_wide_t;

void bf_wide_init(bf_wide_t *w) {
    uint32_t i;
    for (i = 0; i < BF_WIDE_SIZE; i++) {
        w->counters[i] = 0;
    }
    w->num_items = 0;
    w->total_ops = 0;
}

static uint32_t bf_wide_hash(uint32_t key, uint32_t seed) {
    uint32_t h = key ^ seed;
    h ^= h >> 16;
    h *= 0x85ebca6b;
    h ^= h >> 13;
    return h % BF_WIDE_SIZE;
}

void bf_wide_add(bf_wide_t *w, uint32_t key) {
    uint32_t i;
    for (i = 0; i < BF_WIDE_HASHES; i++) {
        uint32_t pos = bf_wide_hash(key, i * 0x9e3779b9);
        if (w->counters[pos] < BF_WIDE_MAX) {
            w->counters[pos]++;
        }
    }
    w->num_items++;
    w->total_ops++;
}

int bf_wide_remove(bf_wide_t *w, uint32_t key) {
    uint32_t i;
    for (i = 0; i < BF_WIDE_HASHES; i++) {
        uint32_t pos = bf_wide_hash(key, i * 0x9e3779b9);
        if (w->counters[pos] == 0) return -1;
    }
    for (i = 0; i < BF_WIDE_HASHES; i++) {
        uint32_t pos = bf_wide_hash(key, i * 0x9e3779b9);
        w->counters[pos]--;
    }
    w->num_items--;
    w->total_ops++;
    return 0;
}

uint32_t bf_wide_min_count(const bf_wide_t *w, uint32_t key) {
    uint32_t min_val = BF_WIDE_MAX;
    uint32_t i;
    for (i = 0; i < BF_WIDE_HASHES; i++) {
        uint32_t pos = bf_wide_hash(key, i * 0x9e3779b9);
        if (w->counters[pos] < min_val) {
            min_val = w->counters[pos];
        }
    }
    return min_val;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1885: Counting bloom wide counters should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1885: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1885: Should contain bf_ functions");
}

// ============================================================================
// C1886-C1890: Scalable Bloom Filter
// ============================================================================

/// C1886: Scalable bloom filter with multi-level structure
#[test]
// UN-FALSIFIED: for(;;) fix (S2) resolved HIR panic on computed #define expressions
fn c1886_scalable_bloom_multilevel() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_SBF_LEVELS 4
#define BF_SBF_LEVEL_BITS 1024
#define BF_SBF_LEVEL_BYTES (BF_SBF_LEVEL_BITS / 8)

typedef struct {
    uint8_t bits[BF_SBF_LEVELS][BF_SBF_LEVEL_BYTES];
    uint32_t level_counts[BF_SBF_LEVELS];
    uint32_t active_levels;
    uint32_t level_capacity;
} bf_scalable_t;

void bf_scalable_init(bf_scalable_t *sbf, uint32_t cap_per_level) {
    uint32_t i;
    uint32_t j;
    for (i = 0; i < BF_SBF_LEVELS; i++) {
        for (j = 0; j < BF_SBF_LEVEL_BYTES; j++) {
            sbf->bits[i][j] = 0;
        }
        sbf->level_counts[i] = 0;
    }
    sbf->active_levels = 1;
    sbf->level_capacity = cap_per_level;
}

static uint32_t bf_scalable_hash(uint32_t key, uint32_t level, uint32_t idx) {
    uint32_t h = key ^ (level * 0xdeadbeef) ^ (idx * 0x9e3779b9);
    h = (h ^ (h >> 16)) * 0x45d9f3b;
    h = (h ^ (h >> 16)) * 0x45d9f3b;
    h = h ^ (h >> 16);
    return h % BF_SBF_LEVEL_BITS;
}

void bf_scalable_add(bf_scalable_t *sbf, uint32_t key) {
    uint32_t level = sbf->active_levels - 1;
    uint32_t i;
    for (i = 0; i < 3; i++) {
        uint32_t pos = bf_scalable_hash(key, level, i);
        sbf->bits[level][pos / 8] |= (uint8_t)(1 << (pos % 8));
    }
    sbf->level_counts[level]++;
    if (sbf->level_counts[level] >= sbf->level_capacity && sbf->active_levels < BF_SBF_LEVELS) {
        sbf->active_levels++;
    }
}

int bf_scalable_query(const bf_scalable_t *sbf, uint32_t key) {
    uint32_t level;
    for (level = 0; level < sbf->active_levels; level++) {
        int found = 1;
        uint32_t i;
        for (i = 0; i < 3; i++) {
            uint32_t pos = bf_scalable_hash(key, level, i);
            if (((sbf->bits[level][pos / 8] >> (pos % 8)) & 1) == 0) {
                found = 0;
                break;
            }
        }
        if (found) return 1;
    }
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1886: Scalable bloom multilevel should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1886: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1886: Should contain bf_ functions");
}

/// C1887: Scalable bloom filter dynamic resize trigger
#[test]
// UN-FALSIFIED: for(;;) fix (S2) resolved HIR panic on computed #define expressions
fn c1887_scalable_bloom_resize() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_RSZ_BITS 512
#define BF_RSZ_BYTES (BF_RSZ_BITS / 8)
#define BF_RSZ_MAX_FILTERS 8

typedef struct {
    uint8_t bits[BF_RSZ_BYTES];
    uint32_t count;
    uint32_t capacity;
} bf_rsz_layer_t;

typedef struct {
    bf_rsz_layer_t layers[BF_RSZ_MAX_FILTERS];
    uint32_t num_layers;
} bf_resizable_t;

void bf_resizable_init(bf_resizable_t *rbf) {
    uint32_t i;
    uint32_t j;
    for (i = 0; i < BF_RSZ_MAX_FILTERS; i++) {
        for (j = 0; j < BF_RSZ_BYTES; j++) {
            rbf->layers[i].bits[j] = 0;
        }
        rbf->layers[i].count = 0;
        rbf->layers[i].capacity = BF_RSZ_BITS / 2;
    }
    rbf->num_layers = 1;
}

static uint32_t bf_resizable_hash(uint32_t key, uint32_t seed) {
    uint32_t h = key ^ seed;
    h *= 0x01000193;
    h ^= h >> 15;
    return h % BF_RSZ_BITS;
}

int bf_resizable_needs_grow(const bf_resizable_t *rbf) {
    uint32_t top = rbf->num_layers - 1;
    return rbf->layers[top].count >= rbf->layers[top].capacity ? 1 : 0;
}

void bf_resizable_add(bf_resizable_t *rbf, uint32_t key) {
    if (bf_resizable_needs_grow(rbf) && rbf->num_layers < BF_RSZ_MAX_FILTERS) {
        rbf->num_layers++;
    }
    uint32_t layer = rbf->num_layers - 1;
    uint32_t pos1 = bf_resizable_hash(key, 0x12345678);
    uint32_t pos2 = bf_resizable_hash(key, 0x87654321);
    rbf->layers[layer].bits[pos1 / 8] |= (uint8_t)(1 << (pos1 % 8));
    rbf->layers[layer].bits[pos2 / 8] |= (uint8_t)(1 << (pos2 % 8));
    rbf->layers[layer].count++;
}

uint32_t bf_resizable_total_items(const bf_resizable_t *rbf) {
    uint32_t total = 0;
    uint32_t i;
    for (i = 0; i < rbf->num_layers; i++) {
        total += rbf->layers[i].count;
    }
    return total;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1887: Scalable bloom resize should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1887: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1887: Should contain bf_ functions");
}

/// C1888: Scalable bloom filter capacity estimation
#[test]
// UN-FALSIFIED: for(;;) fix (S2) resolved HIR panic on computed #define expressions
fn c1888_scalable_bloom_capacity() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_CAP_BITS 2048
#define BF_CAP_BYTES (BF_CAP_BITS / 8)

typedef struct {
    uint8_t bits[BF_CAP_BYTES];
    uint32_t num_inserted;
    uint32_t num_hashes;
} bf_capacity_t;

void bf_capacity_init(bf_capacity_t *bf, uint32_t k) {
    uint32_t i;
    for (i = 0; i < BF_CAP_BYTES; i++) {
        bf->bits[i] = 0;
    }
    bf->num_inserted = 0;
    bf->num_hashes = k;
}

uint32_t bf_capacity_popcount(const bf_capacity_t *bf) {
    uint32_t count = 0;
    uint32_t i;
    for (i = 0; i < BF_CAP_BYTES; i++) {
        uint8_t b = bf->bits[i];
        b = b - ((b >> 1) & 0x55);
        b = (b & 0x33) + ((b >> 2) & 0x33);
        count += (b + (b >> 4)) & 0x0F;
    }
    return count;
}

uint32_t bf_capacity_estimate_items(const bf_capacity_t *bf) {
    uint32_t set_bits = bf_capacity_popcount(bf);
    if (set_bits == 0 || set_bits >= BF_CAP_BITS) return bf->num_inserted;
    uint32_t m = BF_CAP_BITS;
    uint32_t k = bf->num_hashes;
    if (k == 0) k = 1;
    uint32_t empty_bits = m - set_bits;
    uint32_t estimate = (m * 10) / k;
    if (empty_bits > 0) {
        estimate = estimate - (empty_bits * 10) / k;
    }
    return estimate / 10;
}

uint32_t bf_capacity_fill_ratio(const bf_capacity_t *bf) {
    uint32_t set_bits = bf_capacity_popcount(bf);
    return (set_bits * 100) / BF_CAP_BITS;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1888: Scalable bloom capacity should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1888: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1888: Should contain bf_ functions");
}

/// C1889: Scalable bloom filter merge operation
#[test]
// UN-FALSIFIED: for(;;) fix (S2) resolved HIR panic on computed #define expressions
fn c1889_scalable_bloom_merge() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_MRG_SIZE 256
#define BF_MRG_BYTES (BF_MRG_SIZE / 8)

typedef struct {
    uint8_t bits[BF_MRG_BYTES];
    uint32_t count;
} bf_mergeable_t;

void bf_mergeable_init(bf_mergeable_t *bf) {
    uint32_t i;
    for (i = 0; i < BF_MRG_BYTES; i++) {
        bf->bits[i] = 0;
    }
    bf->count = 0;
}

static uint32_t bf_mergeable_hash(uint32_t key, uint32_t seed) {
    uint32_t h = key ^ seed;
    h = (h ^ (h >> 16)) * 0x45d9f3b;
    h = h ^ (h >> 16);
    return h % BF_MRG_SIZE;
}

void bf_mergeable_add(bf_mergeable_t *bf, uint32_t key) {
    uint32_t pos1 = bf_mergeable_hash(key, 0xAAAAAAAA);
    uint32_t pos2 = bf_mergeable_hash(key, 0x55555555);
    uint32_t pos3 = bf_mergeable_hash(key, 0xDEADBEEF);
    bf->bits[pos1 / 8] |= (uint8_t)(1 << (pos1 % 8));
    bf->bits[pos2 / 8] |= (uint8_t)(1 << (pos2 % 8));
    bf->bits[pos3 / 8] |= (uint8_t)(1 << (pos3 % 8));
    bf->count++;
}

void bf_mergeable_union(bf_mergeable_t *dst, const bf_mergeable_t *src) {
    uint32_t i;
    for (i = 0; i < BF_MRG_BYTES; i++) {
        dst->bits[i] |= src->bits[i];
    }
    dst->count += src->count;
}

void bf_mergeable_intersect(bf_mergeable_t *dst, const bf_mergeable_t *src) {
    uint32_t i;
    for (i = 0; i < BF_MRG_BYTES; i++) {
        dst->bits[i] &= src->bits[i];
    }
}

uint32_t bf_mergeable_popcount(const bf_mergeable_t *bf) {
    uint32_t count = 0;
    uint32_t i;
    for (i = 0; i < BF_MRG_BYTES; i++) {
        uint8_t b = bf->bits[i];
        while (b) {
            count += b & 1;
            b >>= 1;
        }
    }
    return count;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1889: Scalable bloom merge should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1889: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1889: Should contain bf_ functions");
}

/// C1890: Scalable bloom filter with tightening ratio
#[test]
// UN-FALSIFIED: for(;;) fix (S2) resolved HIR panic on computed #define expressions
fn c1890_scalable_bloom_tightening() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_TGT_BITS 1024
#define BF_TGT_BYTES (BF_TGT_BITS / 8)
#define BF_TGT_MAX_SLICES 6

typedef struct {
    uint8_t bits[BF_TGT_MAX_SLICES][BF_TGT_BYTES];
    uint32_t slice_hashes[BF_TGT_MAX_SLICES];
    uint32_t slice_counts[BF_TGT_MAX_SLICES];
    uint32_t num_slices;
    uint32_t base_hashes;
} bf_tightening_t;

void bf_tightening_init(bf_tightening_t *tbf, uint32_t base_k) {
    uint32_t i;
    uint32_t j;
    for (i = 0; i < BF_TGT_MAX_SLICES; i++) {
        for (j = 0; j < BF_TGT_BYTES; j++) {
            tbf->bits[i][j] = 0;
        }
        tbf->slice_counts[i] = 0;
        tbf->slice_hashes[i] = base_k + i;
    }
    tbf->num_slices = 1;
    tbf->base_hashes = base_k;
}

static uint32_t bf_tightening_hash(uint32_t key, uint32_t seed) {
    uint32_t h = key ^ seed;
    h = (h ^ (h >> 16)) * 0x85ebca6b;
    h = (h ^ (h >> 13)) * 0xc2b2ae35;
    return h % BF_TGT_BITS;
}

void bf_tightening_add(bf_tightening_t *tbf, uint32_t key) {
    uint32_t slice = tbf->num_slices - 1;
    uint32_t k = tbf->slice_hashes[slice];
    uint32_t i;
    for (i = 0; i < k; i++) {
        uint32_t pos = bf_tightening_hash(key, i * 0x9e3779b9 + slice);
        tbf->bits[slice][pos / 8] |= (uint8_t)(1 << (pos % 8));
    }
    tbf->slice_counts[slice]++;
    if (tbf->slice_counts[slice] > 100 && tbf->num_slices < BF_TGT_MAX_SLICES) {
        tbf->num_slices++;
    }
}

uint32_t bf_tightening_total(const bf_tightening_t *tbf) {
    uint32_t total = 0;
    uint32_t i;
    for (i = 0; i < tbf->num_slices; i++) {
        total += tbf->slice_counts[i];
    }
    return total;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1890: Scalable bloom tightening should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1890: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1890: Should contain bf_ functions");
}

// ============================================================================
// C1891-C1895: Cuckoo Filter
// ============================================================================

/// C1891: Cuckoo filter bucket array with fingerprints
#[test]
fn c1891_cuckoo_filter_bucket_array() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_CUCKOO_BUCKETS 128
#define BF_CUCKOO_SLOTS 4
#define BF_CUCKOO_EMPTY 0

typedef struct {
    uint8_t fingerprints[BF_CUCKOO_BUCKETS][BF_CUCKOO_SLOTS];
    uint32_t count;
} bf_cuckoo_t;

void bf_cuckoo_init(bf_cuckoo_t *cf) {
    uint32_t i;
    uint32_t j;
    for (i = 0; i < BF_CUCKOO_BUCKETS; i++) {
        for (j = 0; j < BF_CUCKOO_SLOTS; j++) {
            cf->fingerprints[i][j] = BF_CUCKOO_EMPTY;
        }
    }
    cf->count = 0;
}

static uint8_t bf_cuckoo_fingerprint(uint32_t key) {
    uint32_t h = key * 0x5bd1e995;
    h ^= h >> 15;
    uint8_t fp = (uint8_t)(h & 0xFF);
    if (fp == BF_CUCKOO_EMPTY) fp = 1;
    return fp;
}

static uint32_t bf_cuckoo_index1(uint32_t key) {
    uint32_t h = key;
    h ^= h >> 16;
    h *= 0x45d9f3b;
    return h % BF_CUCKOO_BUCKETS;
}

static uint32_t bf_cuckoo_index2(uint32_t i1, uint8_t fp) {
    uint32_t h = (uint32_t)fp * 0x5bd1e995;
    return (i1 ^ h) % BF_CUCKOO_BUCKETS;
}

int bf_cuckoo_bucket_has_space(const bf_cuckoo_t *cf, uint32_t bucket) {
    uint32_t j;
    for (j = 0; j < BF_CUCKOO_SLOTS; j++) {
        if (cf->fingerprints[bucket][j] == BF_CUCKOO_EMPTY) {
            return 1;
        }
    }
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1891: Cuckoo filter bucket array should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1891: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1891: Should contain bf_ functions");
}

/// C1892: Cuckoo filter fingerprint computation
#[test]
fn c1892_cuckoo_filter_fingerprint() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_FP_BUCKETS 256
#define BF_FP_SLOTS 4

typedef struct {
    uint8_t fps[BF_FP_BUCKETS][BF_FP_SLOTS];
    uint32_t count;
} bf_fp_filter_t;

void bf_fp_init(bf_fp_filter_t *f) {
    uint32_t i;
    uint32_t j;
    for (i = 0; i < BF_FP_BUCKETS; i++) {
        for (j = 0; j < BF_FP_SLOTS; j++) {
            f->fps[i][j] = 0;
        }
    }
    f->count = 0;
}

static uint8_t bf_fp_compute(uint32_t key, uint32_t bits) {
    uint32_t h = key;
    h = (h ^ (h >> 16)) * 0x85ebca6b;
    h = (h ^ (h >> 13)) * 0xc2b2ae35;
    h = h ^ (h >> 16);
    uint8_t fp = (uint8_t)(h & ((1 << bits) - 1));
    if (fp == 0) fp = 1;
    return fp;
}

static uint32_t bf_fp_primary_index(uint32_t key) {
    return (key * 2654435761u) % BF_FP_BUCKETS;
}

static uint32_t bf_fp_alt_index(uint32_t idx, uint8_t fp) {
    return (idx ^ ((uint32_t)fp * 0x5bd1e995)) % BF_FP_BUCKETS;
}

int bf_fp_bucket_contains(const bf_fp_filter_t *f, uint32_t bucket, uint8_t fp) {
    uint32_t j;
    for (j = 0; j < BF_FP_SLOTS; j++) {
        if (f->fps[bucket][j] == fp) {
            return 1;
        }
    }
    return 0;
}

int bf_fp_bucket_insert(bf_fp_filter_t *f, uint32_t bucket, uint8_t fp) {
    uint32_t j;
    for (j = 0; j < BF_FP_SLOTS; j++) {
        if (f->fps[bucket][j] == 0) {
            f->fps[bucket][j] = fp;
            f->count++;
            return 0;
        }
    }
    return -1;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1892: Cuckoo filter fingerprint should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1892: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1892: Should contain bf_ functions");
}

/// C1893: Cuckoo filter insert and lookup
#[test]
fn c1893_cuckoo_filter_insert_lookup() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_CIL_BUCKETS 128
#define BF_CIL_SLOTS 4
#define BF_CIL_MAX_KICKS 500

typedef struct {
    uint8_t fps[BF_CIL_BUCKETS][BF_CIL_SLOTS];
    uint32_t count;
} bf_cil_t;

void bf_cil_init(bf_cil_t *cf) {
    uint32_t i;
    uint32_t j;
    for (i = 0; i < BF_CIL_BUCKETS; i++) {
        for (j = 0; j < BF_CIL_SLOTS; j++) {
            cf->fps[i][j] = 0;
        }
    }
    cf->count = 0;
}

static uint8_t bf_cil_fingerprint(uint32_t key) {
    uint32_t h = key * 0x5bd1e995;
    h ^= h >> 15;
    uint8_t fp = (uint8_t)(h & 0xFF);
    return fp == 0 ? 1 : fp;
}

static uint32_t bf_cil_hash(uint32_t key) {
    return (key ^ (key >> 16)) % BF_CIL_BUCKETS;
}

static uint32_t bf_cil_alt(uint32_t idx, uint8_t fp) {
    return (idx ^ ((uint32_t)fp * 0x5bd1e995)) % BF_CIL_BUCKETS;
}

int bf_cil_insert(bf_cil_t *cf, uint32_t key) {
    uint8_t fp = bf_cil_fingerprint(key);
    uint32_t i1 = bf_cil_hash(key);
    uint32_t i2 = bf_cil_alt(i1, fp);
    uint32_t j;
    for (j = 0; j < BF_CIL_SLOTS; j++) {
        if (cf->fps[i1][j] == 0) { cf->fps[i1][j] = fp; cf->count++; return 0; }
    }
    for (j = 0; j < BF_CIL_SLOTS; j++) {
        if (cf->fps[i2][j] == 0) { cf->fps[i2][j] = fp; cf->count++; return 0; }
    }
    return -1;
}

int bf_cil_lookup(const bf_cil_t *cf, uint32_t key) {
    uint8_t fp = bf_cil_fingerprint(key);
    uint32_t i1 = bf_cil_hash(key);
    uint32_t i2 = bf_cil_alt(i1, fp);
    uint32_t j;
    for (j = 0; j < BF_CIL_SLOTS; j++) {
        if (cf->fps[i1][j] == fp) return 1;
        if (cf->fps[i2][j] == fp) return 1;
    }
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1893: Cuckoo filter insert/lookup should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1893: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1893: Should contain bf_ functions");
}

/// C1894: Cuckoo filter relocation (kick) operation
#[test]
fn c1894_cuckoo_filter_relocation() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_KICK_BUCKETS 64
#define BF_KICK_SLOTS 4
#define BF_KICK_MAX 500

typedef struct {
    uint8_t fps[BF_KICK_BUCKETS][BF_KICK_SLOTS];
    uint32_t count;
    uint32_t kick_count;
} bf_kick_t;

void bf_kick_init(bf_kick_t *cf) {
    uint32_t i;
    uint32_t j;
    for (i = 0; i < BF_KICK_BUCKETS; i++) {
        for (j = 0; j < BF_KICK_SLOTS; j++) {
            cf->fps[i][j] = 0;
        }
    }
    cf->count = 0;
    cf->kick_count = 0;
}

static uint32_t bf_kick_alt_bucket(uint32_t idx, uint8_t fp) {
    return (idx ^ ((uint32_t)fp * 0x5bd1e995)) % BF_KICK_BUCKETS;
}

int bf_kick_relocate(bf_kick_t *cf, uint32_t bucket, uint8_t fp) {
    uint32_t cur_bucket = bucket;
    uint8_t cur_fp = fp;
    uint32_t n;
    for (n = 0; n < BF_KICK_MAX; n++) {
        uint32_t slot = n % BF_KICK_SLOTS;
        uint8_t old_fp = cf->fps[cur_bucket][slot];
        cf->fps[cur_bucket][slot] = cur_fp;
        cf->kick_count++;
        if (old_fp == 0) {
            cf->count++;
            return 0;
        }
        cur_fp = old_fp;
        cur_bucket = bf_kick_alt_bucket(cur_bucket, cur_fp);
        uint32_t j;
        for (j = 0; j < BF_KICK_SLOTS; j++) {
            if (cf->fps[cur_bucket][j] == 0) {
                cf->fps[cur_bucket][j] = cur_fp;
                cf->count++;
                return 0;
            }
        }
    }
    return -1;
}

uint32_t bf_kick_stats(const bf_kick_t *cf) {
    return cf->kick_count;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1894: Cuckoo filter relocation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1894: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1894: Should contain bf_ functions");
}

/// C1895: Cuckoo filter delete operation
#[test]
fn c1895_cuckoo_filter_delete() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_DEL_BUCKETS 128
#define BF_DEL_SLOTS 4

typedef struct {
    uint8_t fps[BF_DEL_BUCKETS][BF_DEL_SLOTS];
    uint32_t count;
    uint32_t delete_count;
} bf_del_cuckoo_t;

void bf_del_cuckoo_init(bf_del_cuckoo_t *cf) {
    uint32_t i;
    uint32_t j;
    for (i = 0; i < BF_DEL_BUCKETS; i++) {
        for (j = 0; j < BF_DEL_SLOTS; j++) {
            cf->fps[i][j] = 0;
        }
    }
    cf->count = 0;
    cf->delete_count = 0;
}

static uint8_t bf_del_fingerprint(uint32_t key) {
    uint32_t h = key * 0x5bd1e995;
    h ^= h >> 15;
    uint8_t fp = (uint8_t)(h & 0xFF);
    return fp == 0 ? 1 : fp;
}

static uint32_t bf_del_hash(uint32_t key) {
    return (key ^ (key >> 16)) % BF_DEL_BUCKETS;
}

static uint32_t bf_del_alt(uint32_t idx, uint8_t fp) {
    return (idx ^ ((uint32_t)fp * 0x5bd1e995)) % BF_DEL_BUCKETS;
}

int bf_del_cuckoo_remove(bf_del_cuckoo_t *cf, uint32_t key) {
    uint8_t fp = bf_del_fingerprint(key);
    uint32_t i1 = bf_del_hash(key);
    uint32_t i2 = bf_del_alt(i1, fp);
    uint32_t j;
    for (j = 0; j < BF_DEL_SLOTS; j++) {
        if (cf->fps[i1][j] == fp) {
            cf->fps[i1][j] = 0;
            cf->count--;
            cf->delete_count++;
            return 0;
        }
    }
    for (j = 0; j < BF_DEL_SLOTS; j++) {
        if (cf->fps[i2][j] == fp) {
            cf->fps[i2][j] = 0;
            cf->count--;
            cf->delete_count++;
            return 0;
        }
    }
    return -1;
}

int bf_del_cuckoo_test(void) {
    bf_del_cuckoo_t cf;
    bf_del_cuckoo_init(&cf);
    return (int)cf.count;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1895: Cuckoo filter delete should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1895: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1895: Should contain bf_ functions");
}

// ============================================================================
// C1896-C1900: HyperLogLog
// ============================================================================

/// C1896: HyperLogLog register array initialization
#[test]
// UN-FALSIFIED: for(;;) fix (S2) resolved HIR panic on computed #define expressions
fn c1896_hyperloglog_register_array() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_HLL_P 14
#define BF_HLL_M (1 << BF_HLL_P)

typedef struct {
    uint8_t registers[BF_HLL_M];
    uint32_t count;
} bf_hll_t;

void bf_hll_init(bf_hll_t *hll) {
    uint32_t i;
    for (i = 0; i < BF_HLL_M; i++) {
        hll->registers[i] = 0;
    }
    hll->count = 0;
}

void bf_hll_reset(bf_hll_t *hll) {
    uint32_t i;
    for (i = 0; i < BF_HLL_M; i++) {
        hll->registers[i] = 0;
    }
    hll->count = 0;
}

uint32_t bf_hll_num_zeros(const bf_hll_t *hll) {
    uint32_t zeros = 0;
    uint32_t i;
    for (i = 0; i < BF_HLL_M; i++) {
        if (hll->registers[i] == 0) {
            zeros++;
        }
    }
    return zeros;
}

uint8_t bf_hll_max_register(const bf_hll_t *hll) {
    uint8_t max_val = 0;
    uint32_t i;
    for (i = 0; i < BF_HLL_M; i++) {
        if (hll->registers[i] > max_val) {
            max_val = hll->registers[i];
        }
    }
    return max_val;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1896: HyperLogLog register array should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1896: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1896: Should contain bf_ functions");
}

/// C1897: HyperLogLog hash function with leading zeros
#[test]
fn c1897_hyperloglog_hash() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_HLLH_P 10
#define BF_HLLH_M (1 << BF_HLLH_P)

static uint32_t bf_hll_hash(uint32_t key) {
    uint32_t h = key;
    h ^= h >> 16;
    h *= 0x85ebca6b;
    h ^= h >> 13;
    h *= 0xc2b2ae35;
    h ^= h >> 16;
    return h;
}

static uint8_t bf_hll_leading_zeros(uint32_t value) {
    uint8_t count = 0;
    if (value == 0) return 32;
    while ((value & 0x80000000) == 0) {
        count++;
        value <<= 1;
    }
    return count;
}

static uint8_t bf_hll_rho(uint32_t hash_val, uint32_t p) {
    uint32_t w = hash_val >> p;
    return bf_hll_leading_zeros(w) + 1;
}

uint32_t bf_hll_hash_to_index(uint32_t hash_val) {
    return hash_val & (BF_HLLH_M - 1);
}

int bf_hll_hash_test(void) {
    uint32_t h = bf_hll_hash(42);
    uint32_t idx = bf_hll_hash_to_index(h);
    if (idx >= BF_HLLH_M) return -1;
    uint8_t rho = bf_hll_rho(h, BF_HLLH_P);
    if (rho == 0 || rho > 32) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1897: HyperLogLog hash should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1897: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1897: Should contain bf_ functions");
}

/// C1898: HyperLogLog add element
#[test]
// UN-FALSIFIED: for(;;) fix (S2) resolved HIR panic on computed #define expressions
fn c1898_hyperloglog_add_element() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_HLLA_P 12
#define BF_HLLA_M (1 << BF_HLLA_P)

typedef struct {
    uint8_t registers[BF_HLLA_M];
    uint32_t total_added;
} bf_hll_add_t;

void bf_hll_add_init(bf_hll_add_t *hll) {
    uint32_t i;
    for (i = 0; i < BF_HLLA_M; i++) {
        hll->registers[i] = 0;
    }
    hll->total_added = 0;
}

static uint32_t bf_hll_add_hash(uint32_t key) {
    uint32_t h = key;
    h ^= h >> 16;
    h *= 0x85ebca6b;
    h ^= h >> 13;
    h *= 0xc2b2ae35;
    h ^= h >> 16;
    return h;
}

static uint8_t bf_hll_add_clz(uint32_t val) {
    uint8_t n = 0;
    if (val == 0) return 32;
    if ((val & 0xFFFF0000) == 0) { n += 16; val <<= 16; }
    if ((val & 0xFF000000) == 0) { n += 8; val <<= 8; }
    if ((val & 0xF0000000) == 0) { n += 4; val <<= 4; }
    if ((val & 0xC0000000) == 0) { n += 2; val <<= 2; }
    if ((val & 0x80000000) == 0) { n += 1; }
    return n;
}

void bf_hll_add_element(bf_hll_add_t *hll, uint32_t key) {
    uint32_t h = bf_hll_add_hash(key);
    uint32_t idx = h & (BF_HLLA_M - 1);
    uint32_t w = h >> BF_HLLA_P;
    uint8_t rho = bf_hll_add_clz(w) + 1;
    if (rho > hll->registers[idx]) {
        hll->registers[idx] = rho;
    }
    hll->total_added++;
}

int bf_hll_add_test(void) {
    bf_hll_add_t hll;
    bf_hll_add_init(&hll);
    bf_hll_add_element(&hll, 1);
    bf_hll_add_element(&hll, 2);
    bf_hll_add_element(&hll, 3);
    if (hll.total_added != 3) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1898: HyperLogLog add element should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1898: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1898: Should contain bf_ functions");
}

/// C1899: HyperLogLog cardinality estimation
#[test]
// UN-FALSIFIED: for(;;) fix (S2) resolved HIR panic on computed #define expressions
fn c1899_hyperloglog_cardinality() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_HLLC_P 10
#define BF_HLLC_M (1 << BF_HLLC_P)

typedef struct {
    uint8_t registers[BF_HLLC_M];
} bf_hll_card_t;

void bf_hll_card_init(bf_hll_card_t *hll) {
    uint32_t i;
    for (i = 0; i < BF_HLLC_M; i++) {
        hll->registers[i] = 0;
    }
}

uint32_t bf_hll_card_count_zeros(const bf_hll_card_t *hll) {
    uint32_t zeros = 0;
    uint32_t i;
    for (i = 0; i < BF_HLLC_M; i++) {
        if (hll->registers[i] == 0) zeros++;
    }
    return zeros;
}

uint32_t bf_hll_card_raw_estimate(const bf_hll_card_t *hll) {
    uint32_t sum = 0;
    uint32_t i;
    for (i = 0; i < BF_HLLC_M; i++) {
        sum += (1 << hll->registers[i]);
    }
    if (sum == 0) return 0;
    uint32_t alpha_m2 = (BF_HLLC_M * BF_HLLC_M * 72) / 100;
    uint32_t estimate = alpha_m2 / (sum / BF_HLLC_M);
    return estimate;
}

uint32_t bf_hll_card_estimate(const bf_hll_card_t *hll) {
    uint32_t raw = bf_hll_card_raw_estimate(hll);
    uint32_t zeros = bf_hll_card_count_zeros(hll);
    if (raw < (5 * BF_HLLC_M / 2) && zeros > 0) {
        uint32_t linear = BF_HLLC_M;
        if (zeros < BF_HLLC_M) {
            linear = (BF_HLLC_M * 10) / ((zeros * 10) / BF_HLLC_M + 1);
        }
        return linear;
    }
    return raw;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1899: HyperLogLog cardinality should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1899: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1899: Should contain bf_ functions");
}

/// C1900: HyperLogLog merge two sketches
#[test]
// UN-FALSIFIED: for(;;) fix (S2) resolved HIR panic on computed #define expressions
fn c1900_hyperloglog_merge() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

#define BF_HLLM_P 10
#define BF_HLLM_M (1 << BF_HLLM_P)

typedef struct {
    uint8_t registers[BF_HLLM_M];
    uint32_t merge_count;
} bf_hll_merge_t;

void bf_hll_merge_init(bf_hll_merge_t *hll) {
    uint32_t i;
    for (i = 0; i < BF_HLLM_M; i++) {
        hll->registers[i] = 0;
    }
    hll->merge_count = 0;
}

void bf_hll_merge_combine(bf_hll_merge_t *dst, const bf_hll_merge_t *src) {
    uint32_t i;
    for (i = 0; i < BF_HLLM_M; i++) {
        if (src->registers[i] > dst->registers[i]) {
            dst->registers[i] = src->registers[i];
        }
    }
    dst->merge_count++;
}

uint32_t bf_hll_merge_nonzero(const bf_hll_merge_t *hll) {
    uint32_t count = 0;
    uint32_t i;
    for (i = 0; i < BF_HLLM_M; i++) {
        if (hll->registers[i] > 0) count++;
    }
    return count;
}

int bf_hll_merge_equal(const bf_hll_merge_t *a, const bf_hll_merge_t *b) {
    uint32_t i;
    for (i = 0; i < BF_HLLM_M; i++) {
        if (a->registers[i] != b->registers[i]) return 0;
    }
    return 1;
}

int bf_hll_merge_test(void) {
    bf_hll_merge_t a;
    bf_hll_merge_t b;
    bf_hll_merge_init(&a);
    bf_hll_merge_init(&b);
    a.registers[0] = 5;
    a.registers[1] = 3;
    b.registers[0] = 3;
    b.registers[1] = 7;
    bf_hll_merge_combine(&a, &b);
    if (a.registers[0] != 5) return -1;
    if (a.registers[1] != 7) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1900: HyperLogLog merge should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1900: Output should not be empty");
    assert!(code.contains("fn bf_"), "C1900: Should contain bf_ functions");
}
