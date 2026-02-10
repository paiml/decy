//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1201-C1225: Blockchain & Distributed Ledger -- data structures and
//! cryptographic algorithms commonly found in blockchain implementations,
//! consensus engines, and distributed ledger systems.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! Organization:
//! - C1201-C1205: Hash functions (SHA-256 round, MD5-like, CRC32, FNV hash, HMAC)
//! - C1206-C1210: Block structures (block header, merkle tree, transaction, UTXO set, block validation)
//! - C1211-C1215: Consensus (proof of work, difficulty adjustment, chain selection, block reward, nonce search)
//! - C1216-C1220: Crypto primitives (modular exponentiation, EC point add, signature verify sim, key derivation, random seed)
//! - C1221-C1225: Distributed (gossip protocol, peer table, message routing, bloom filter index, consensus voting)

// ============================================================================
// C1201-C1205: Hash Functions
// ============================================================================

#[test]
fn c1201_sha256_compression_round() {
    let c_code = r#"
typedef unsigned int uint32_t;

uint32_t bc_rotr(uint32_t x, int n) {
    return (x >> n) | (x << (32 - n));
}

uint32_t bc_ch(uint32_t x, uint32_t y, uint32_t z) {
    return (x & y) ^ (~x & z);
}

uint32_t bc_maj(uint32_t x, uint32_t y, uint32_t z) {
    return (x & y) ^ (x & z) ^ (y & z);
}

uint32_t bc_sigma0(uint32_t x) {
    return bc_rotr(x, 2) ^ bc_rotr(x, 13) ^ bc_rotr(x, 22);
}

uint32_t bc_sigma1(uint32_t x) {
    return bc_rotr(x, 6) ^ bc_rotr(x, 11) ^ bc_rotr(x, 25);
}

void bc_sha256_round(uint32_t *state, uint32_t k, uint32_t w) {
    uint32_t t1 = state[7] + bc_sigma1(state[4])
                  + bc_ch(state[4], state[5], state[6]) + k + w;
    uint32_t t2 = bc_sigma0(state[0])
                  + bc_maj(state[0], state[1], state[2]);
    state[7] = state[6];
    state[6] = state[5];
    state[5] = state[4];
    state[4] = state[3] + t1;
    state[3] = state[2];
    state[2] = state[1];
    state[1] = state[0];
    state[0] = t1 + t2;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1201: SHA-256 compression round should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1201: Output should not be empty");
    assert!(
        code.contains("fn bc_sha256_round"),
        "C1201: Should contain bc_sha256_round function"
    );
}

#[test]
fn c1202_md5_like_transform() {
    let c_code = r#"
typedef unsigned int uint32_t;

uint32_t bc_md5_f(uint32_t x, uint32_t y, uint32_t z) {
    return (x & y) | (~x & z);
}

uint32_t bc_md5_g(uint32_t x, uint32_t y, uint32_t z) {
    return (x & z) | (y & ~z);
}

uint32_t bc_md5_rotl(uint32_t x, int n) {
    return (x << n) | (x >> (32 - n));
}

void bc_md5_step(uint32_t *a, uint32_t b, uint32_t c, uint32_t d,
                 uint32_t data, uint32_t t, int s) {
    uint32_t temp = *a + bc_md5_f(b, c, d) + data + t;
    *a = b + bc_md5_rotl(temp, s);
}

int bc_md5_selftest(void) {
    uint32_t a = 0x67452301;
    uint32_t b = 0xEFCDAB89;
    uint32_t c = 0x98BADCFE;
    uint32_t d = 0x10325476;
    bc_md5_step(&a, b, c, d, 0, 0xD76AA478, 7);
    return (a != 0) ? 0 : -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1202: MD5-like transform should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1202: Output should not be empty");
    assert!(
        code.contains("fn bc_md5_step"),
        "C1202: Should contain bc_md5_step function"
    );
}

#[test]
fn c1203_crc32_table_lookup() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;
typedef unsigned long size_t;

uint32_t bc_crc32_update(uint32_t crc, uint8_t byte) {
    uint32_t mask;
    int i;
    crc = crc ^ (uint32_t)byte;
    for (i = 0; i < 8; i++) {
        mask = -(crc & 1);
        crc = (crc >> 1) ^ (0xEDB88320 & mask);
    }
    return crc;
}

uint32_t bc_crc32_compute(const uint8_t *data, size_t len) {
    uint32_t crc = 0xFFFFFFFF;
    size_t i;
    for (i = 0; i < len; i++) {
        crc = bc_crc32_update(crc, data[i]);
    }
    return ~crc;
}

int bc_crc32_verify(const uint8_t *data, size_t len, uint32_t expected) {
    return bc_crc32_compute(data, len) == expected;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1203: CRC32 table lookup should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1203: Output should not be empty");
    assert!(
        code.contains("fn bc_crc32_compute"),
        "C1203: Should contain bc_crc32_compute function"
    );
}

#[test]
fn c1204_fnv_hash() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;
typedef unsigned char uint8_t;
typedef unsigned long size_t;

uint32_t bc_fnv1a_32(const uint8_t *data, size_t len) {
    uint32_t hash = 0x811C9DC5;
    size_t i;
    for (i = 0; i < len; i++) {
        hash ^= (uint32_t)data[i];
        hash *= 0x01000193;
    }
    return hash;
}

uint64_t bc_fnv1a_64(const uint8_t *data, size_t len) {
    uint64_t hash = 0xCBF29CE484222325ULL;
    size_t i;
    for (i = 0; i < len; i++) {
        hash ^= (uint64_t)data[i];
        hash *= 0x00000100000001B3ULL;
    }
    return hash;
}

int bc_fnv_selftest(void) {
    uint8_t test_data[3];
    test_data[0] = 'a';
    test_data[1] = 'b';
    test_data[2] = 'c';
    uint32_t h32 = bc_fnv1a_32(test_data, 3);
    return (h32 != 0) ? 0 : -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1204: FNV hash should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1204: Output should not be empty");
    assert!(
        code.contains("fn bc_fnv1a_32"),
        "C1204: Should contain bc_fnv1a_32 function"
    );
}

#[test]
fn c1205_hmac_construction() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned long size_t;

void bc_xor_block(uint8_t *out, const uint8_t *a, uint8_t val, int len) {
    int i;
    for (i = 0; i < len; i++) {
        out[i] = a[i] ^ val;
    }
}

void bc_hmac_prepare_pads(const uint8_t *key, int key_len,
                          uint8_t *ipad, uint8_t *opad) {
    int i;
    uint8_t padded_key[64];
    for (i = 0; i < 64; i++) padded_key[i] = 0;
    for (i = 0; i < key_len && i < 64; i++) {
        padded_key[i] = key[i];
    }
    bc_xor_block(ipad, padded_key, 0x36, 64);
    bc_xor_block(opad, padded_key, 0x5C, 64);
}

int bc_hmac_key_valid(const uint8_t *key, int key_len) {
    return key != 0 && key_len > 0 && key_len <= 64;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1205: HMAC construction should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1205: Output should not be empty");
    assert!(
        code.contains("fn bc_hmac_prepare_pads"),
        "C1205: Should contain bc_hmac_prepare_pads function"
    );
}

// ============================================================================
// C1206-C1210: Block Structures
// ============================================================================

#[test]
fn c1206_block_header() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint32_t version;
    uint8_t prev_hash[32];
    uint8_t merkle_root[32];
    uint32_t timestamp;
    uint32_t difficulty;
    uint32_t nonce;
} bc_block_header_t;

void bc_block_header_init(bc_block_header_t *hdr, uint32_t version,
                          uint32_t timestamp, uint32_t difficulty) {
    int i;
    hdr->version = version;
    hdr->timestamp = timestamp;
    hdr->difficulty = difficulty;
    hdr->nonce = 0;
    for (i = 0; i < 32; i++) {
        hdr->prev_hash[i] = 0;
        hdr->merkle_root[i] = 0;
    }
}

void bc_block_set_prev(bc_block_header_t *hdr, const uint8_t *hash) {
    int i;
    for (i = 0; i < 32; i++) {
        hdr->prev_hash[i] = hash[i];
    }
}

int bc_block_header_valid(const bc_block_header_t *hdr) {
    return hdr->version > 0 && hdr->timestamp > 0 && hdr->difficulty > 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1206: Block header should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1206: Output should not be empty");
    assert!(
        code.contains("fn bc_block_header_init"),
        "C1206: Should contain bc_block_header_init function"
    );
}

#[test]
fn c1207_merkle_tree_hash() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

void bc_hash_pair(const uint8_t *left, const uint8_t *right,
                  uint8_t *out) {
    int i;
    for (i = 0; i < 32; i++) {
        out[i] = left[i] ^ right[i];
    }
    uint32_t mix = 0;
    for (i = 0; i < 32; i++) {
        mix = mix * 31 + out[i];
        out[i] = (uint8_t)(mix & 0xFF);
    }
}

int bc_merkle_level(uint8_t hashes[][32], int count, uint8_t out[][32]) {
    int i;
    int out_count = 0;
    for (i = 0; i + 1 < count; i += 2) {
        bc_hash_pair(hashes[i], hashes[i + 1], out[out_count]);
        out_count++;
    }
    if (i < count) {
        bc_hash_pair(hashes[i], hashes[i], out[out_count]);
        out_count++;
    }
    return out_count;
}

int bc_merkle_is_power_of_two(int n) {
    return n > 0 && (n & (n - 1)) == 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1207: Merkle tree hash should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1207: Output should not be empty");
    assert!(
        code.contains("fn bc_merkle_level"),
        "C1207: Should contain bc_merkle_level function"
    );
}

#[test]
fn c1208_transaction_structure() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;
typedef unsigned char uint8_t;

typedef struct {
    uint8_t prev_tx_hash[32];
    uint32_t output_index;
} bc_tx_input_t;

typedef struct {
    uint64_t amount;
    uint8_t script_hash[20];
} bc_tx_output_t;

typedef struct {
    uint32_t version;
    int input_count;
    int output_count;
    uint32_t locktime;
} bc_transaction_t;

void bc_tx_init(bc_transaction_t *tx, uint32_t version) {
    tx->version = version;
    tx->input_count = 0;
    tx->output_count = 0;
    tx->locktime = 0;
}

uint64_t bc_tx_compute_fee(const bc_tx_output_t *inputs, int in_count,
                           const bc_tx_output_t *outputs, int out_count) {
    uint64_t total_in = 0;
    uint64_t total_out = 0;
    int i;
    for (i = 0; i < in_count; i++) total_in += inputs[i].amount;
    for (i = 0; i < out_count; i++) total_out += outputs[i].amount;
    return (total_in > total_out) ? (total_in - total_out) : 0;
}

int bc_tx_is_coinbase(const bc_tx_input_t *input) {
    int i;
    for (i = 0; i < 32; i++) {
        if (input->prev_tx_hash[i] != 0) return 0;
    }
    return input->output_index == 0xFFFFFFFF;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1208: Transaction structure should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1208: Output should not be empty");
    assert!(
        code.contains("fn bc_tx_compute_fee"),
        "C1208: Should contain bc_tx_compute_fee function"
    );
}

#[test]
fn c1209_utxo_set() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;
typedef unsigned char uint8_t;

typedef struct {
    uint8_t tx_hash[32];
    uint32_t index;
    uint64_t amount;
    int spent;
} bc_utxo_entry_t;

typedef struct {
    bc_utxo_entry_t entries[256];
    int count;
} bc_utxo_set_t;

void bc_utxo_init(bc_utxo_set_t *set) {
    set->count = 0;
}

int bc_utxo_add(bc_utxo_set_t *set, const uint8_t *tx_hash,
                uint32_t index, uint64_t amount) {
    int i;
    if (set->count >= 256) return -1;
    for (i = 0; i < 32; i++) {
        set->entries[set->count].tx_hash[i] = tx_hash[i];
    }
    set->entries[set->count].index = index;
    set->entries[set->count].amount = amount;
    set->entries[set->count].spent = 0;
    set->count++;
    return 0;
}

int bc_utxo_spend(bc_utxo_set_t *set, const uint8_t *tx_hash, uint32_t index) {
    int i, j;
    for (i = 0; i < set->count; i++) {
        int match = 1;
        for (j = 0; j < 32; j++) {
            if (set->entries[i].tx_hash[j] != tx_hash[j]) {
                match = 0;
                break;
            }
        }
        if (match && set->entries[i].index == index && !set->entries[i].spent) {
            set->entries[i].spent = 1;
            return 0;
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1209: UTXO set should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1209: Output should not be empty");
    assert!(
        code.contains("fn bc_utxo_add"),
        "C1209: Should contain bc_utxo_add function"
    );
}

#[test]
fn c1210_block_validation() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

int bc_hash_meets_difficulty(const uint8_t *hash, uint32_t difficulty) {
    uint32_t leading_zeros = difficulty / 8;
    uint32_t remaining_bits = difficulty % 8;
    uint32_t i;
    for (i = 0; i < leading_zeros && i < 32; i++) {
        if (hash[i] != 0) return 0;
    }
    if (remaining_bits > 0 && i < 32) {
        uint8_t mask = (uint8_t)(0xFF << (8 - remaining_bits));
        if ((hash[i] & mask) != 0) return 0;
    }
    return 1;
}

int bc_validate_timestamps(uint32_t block_time, uint32_t prev_time,
                           uint32_t network_time) {
    if (block_time <= prev_time) return 0;
    if (block_time > network_time + 7200) return 0;
    return 1;
}

int bc_validate_block(const uint8_t *hash, uint32_t difficulty,
                      uint32_t block_time, uint32_t prev_time,
                      uint32_t network_time) {
    if (!bc_hash_meets_difficulty(hash, difficulty)) return 0;
    if (!bc_validate_timestamps(block_time, prev_time, network_time)) return 0;
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1210: Block validation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1210: Output should not be empty");
    assert!(
        code.contains("fn bc_validate_block"),
        "C1210: Should contain bc_validate_block function"
    );
}

// ============================================================================
// C1211-C1215: Consensus
// ============================================================================

#[test]
fn c1211_proof_of_work() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

uint32_t bc_pow_simple_hash(const uint8_t *data, int len, uint32_t nonce) {
    uint32_t hash = 0x811C9DC5;
    int i;
    for (i = 0; i < len; i++) {
        hash ^= (uint32_t)data[i];
        hash *= 0x01000193;
    }
    hash ^= nonce;
    hash *= 0x01000193;
    hash ^= (nonce >> 16);
    hash *= 0x01000193;
    return hash;
}

int bc_pow_check(uint32_t hash, uint32_t target) {
    return hash < target;
}

uint32_t bc_pow_mine(const uint8_t *data, int len, uint32_t target,
                     uint32_t max_nonce) {
    uint32_t nonce;
    for (nonce = 0; nonce < max_nonce; nonce++) {
        uint32_t hash = bc_pow_simple_hash(data, len, nonce);
        if (bc_pow_check(hash, target)) return nonce;
    }
    return 0xFFFFFFFF;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1211: Proof of work should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1211: Output should not be empty");
    assert!(
        code.contains("fn bc_pow_mine"),
        "C1211: Should contain bc_pow_mine function"
    );
}

#[test]
fn c1212_difficulty_adjustment() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

uint32_t bc_adjust_difficulty(uint32_t current_diff, uint32_t actual_time,
                              uint32_t target_time) {
    uint64_t new_diff;
    if (actual_time == 0) actual_time = 1;
    new_diff = (uint64_t)current_diff * (uint64_t)target_time / (uint64_t)actual_time;
    if (new_diff > (uint64_t)current_diff * 4) {
        new_diff = (uint64_t)current_diff * 4;
    }
    if (new_diff < (uint64_t)current_diff / 4) {
        new_diff = (uint64_t)current_diff / 4;
    }
    if (new_diff == 0) new_diff = 1;
    return (uint32_t)new_diff;
}

uint32_t bc_epoch_time_span(const uint32_t *timestamps, int count) {
    if (count < 2) return 0;
    return timestamps[count - 1] - timestamps[0];
}

int bc_difficulty_valid(uint32_t difficulty, uint32_t min_diff) {
    return difficulty >= min_diff;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1212: Difficulty adjustment should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1212: Output should not be empty");
    assert!(
        code.contains("fn bc_adjust_difficulty"),
        "C1212: Should contain bc_adjust_difficulty function"
    );
}

#[test]
fn c1213_chain_selection() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

typedef struct {
    uint64_t cumulative_work;
    uint32_t height;
    uint32_t last_timestamp;
} bc_chain_tip_t;

void bc_chain_tip_init(bc_chain_tip_t *tip) {
    tip->cumulative_work = 0;
    tip->height = 0;
    tip->last_timestamp = 0;
}

void bc_chain_tip_add_block(bc_chain_tip_t *tip, uint32_t difficulty,
                            uint32_t timestamp) {
    tip->cumulative_work += (uint64_t)difficulty;
    tip->height++;
    tip->last_timestamp = timestamp;
}

int bc_chain_select_best(const bc_chain_tip_t *a, const bc_chain_tip_t *b) {
    if (a->cumulative_work > b->cumulative_work) return 0;
    if (b->cumulative_work > a->cumulative_work) return 1;
    if (a->height > b->height) return 0;
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1213: Chain selection should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1213: Output should not be empty");
    assert!(
        code.contains("fn bc_chain_select_best"),
        "C1213: Should contain bc_chain_select_best function"
    );
}

#[test]
fn c1214_block_reward_halving() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

uint64_t bc_block_reward(uint32_t height, uint64_t initial_reward,
                         uint32_t halving_interval) {
    uint32_t halvings;
    uint64_t reward;
    if (halving_interval == 0) return 0;
    halvings = height / halving_interval;
    if (halvings >= 64) return 0;
    reward = initial_reward >> halvings;
    return reward;
}

uint64_t bc_total_supply_at_height(uint32_t height, uint64_t initial_reward,
                                   uint32_t halving_interval) {
    uint64_t total = 0;
    uint32_t h;
    for (h = 0; h < height; h++) {
        total += bc_block_reward(h, initial_reward, halving_interval);
    }
    return total;
}

int bc_is_halving_block(uint32_t height, uint32_t halving_interval) {
    if (halving_interval == 0) return 0;
    return (height % halving_interval) == 0 && height > 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1214: Block reward halving should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1214: Output should not be empty");
    assert!(
        code.contains("fn bc_block_reward"),
        "C1214: Should contain bc_block_reward function"
    );
}

#[test]
fn c1215_nonce_search() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint32_t start;
    uint32_t end;
    uint32_t found_nonce;
    int found;
} bc_nonce_range_t;

uint32_t bc_nonce_hash(uint32_t header_hash, uint32_t nonce) {
    uint32_t h = header_hash;
    h ^= nonce;
    h *= 0x5BD1E995;
    h ^= h >> 13;
    h *= 0x5BD1E995;
    h ^= h >> 15;
    return h;
}

void bc_nonce_search(bc_nonce_range_t *range, uint32_t header_hash,
                     uint32_t target) {
    uint32_t n;
    range->found = 0;
    for (n = range->start; n < range->end; n++) {
        uint32_t h = bc_nonce_hash(header_hash, n);
        if (h < target) {
            range->found_nonce = n;
            range->found = 1;
            return;
        }
    }
}

int bc_nonce_range_valid(const bc_nonce_range_t *range) {
    return range->start < range->end;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1215: Nonce search should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1215: Output should not be empty");
    assert!(
        code.contains("fn bc_nonce_search"),
        "C1215: Should contain bc_nonce_search function"
    );
}

// ============================================================================
// C1216-C1220: Crypto Primitives
// ============================================================================

#[test]
fn c1216_modular_exponentiation() {
    let c_code = r#"
typedef unsigned long long uint64_t;

uint64_t bc_mulmod(uint64_t a, uint64_t b, uint64_t m) {
    uint64_t result = 0;
    a = a % m;
    while (b > 0) {
        if (b & 1) {
            result = (result + a) % m;
        }
        a = (a * 2) % m;
        b >>= 1;
    }
    return result;
}

uint64_t bc_powmod(uint64_t base, uint64_t exp, uint64_t mod) {
    uint64_t result = 1;
    base = base % mod;
    if (base == 0) return 0;
    while (exp > 0) {
        if (exp & 1) {
            result = bc_mulmod(result, base, mod);
        }
        exp >>= 1;
        base = bc_mulmod(base, base, mod);
    }
    return result;
}

int bc_is_probable_prime(uint64_t n) {
    uint64_t witnesses[3];
    int i;
    witnesses[0] = 2; witnesses[1] = 3; witnesses[2] = 5;
    if (n < 2) return 0;
    if (n < 6) return (n == 2 || n == 3 || n == 5);
    for (i = 0; i < 3; i++) {
        if (bc_powmod(witnesses[i], n - 1, n) != 1) return 0;
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1216: Modular exponentiation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1216: Output should not be empty");
    assert!(
        code.contains("fn bc_powmod"),
        "C1216: Should contain bc_powmod function"
    );
}

#[test]
fn c1217_ec_point_add() {
    let c_code = r#"
typedef long long int64_t;

typedef struct {
    int64_t x;
    int64_t y;
    int is_infinity;
} bc_ec_point_t;

int64_t bc_ec_mod(int64_t a, int64_t p) {
    int64_t r = a % p;
    return r < 0 ? r + p : r;
}

int64_t bc_ec_inv(int64_t a, int64_t p) {
    int64_t t = 0, new_t = 1;
    int64_t r = p, new_r = bc_ec_mod(a, p);
    int64_t q, tmp;
    while (new_r != 0) {
        q = r / new_r;
        tmp = t - q * new_t; t = new_t; new_t = tmp;
        tmp = r - q * new_r; r = new_r; new_r = tmp;
    }
    return bc_ec_mod(t, p);
}

bc_ec_point_t bc_ec_add(bc_ec_point_t a, bc_ec_point_t b, int64_t p) {
    bc_ec_point_t result;
    int64_t slope, dx, dy;
    if (a.is_infinity) return b;
    if (b.is_infinity) return a;
    dx = bc_ec_mod(b.x - a.x, p);
    dy = bc_ec_mod(b.y - a.y, p);
    if (dx == 0) {
        result.is_infinity = 1;
        result.x = 0;
        result.y = 0;
        return result;
    }
    slope = bc_ec_mod(dy * bc_ec_inv(dx, p), p);
    result.x = bc_ec_mod(slope * slope - a.x - b.x, p);
    result.y = bc_ec_mod(slope * (a.x - result.x) - a.y, p);
    result.is_infinity = 0;
    return result;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1217: EC point add should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1217: Output should not be empty");
    assert!(
        code.contains("fn bc_ec_add"),
        "C1217: Should contain bc_ec_add function"
    );
}

#[test]
fn c1218_signature_verify_sim() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint32_t r;
    uint32_t s;
} bc_sig_t;

uint32_t bc_sig_hash_message(const uint8_t *msg, int len) {
    uint32_t h = 0;
    int i;
    for (i = 0; i < len; i++) {
        h = h * 31 + msg[i];
    }
    return h;
}

int bc_sig_verify(const bc_sig_t *sig, uint32_t pub_key,
                  const uint8_t *msg, int msg_len) {
    uint32_t h = bc_sig_hash_message(msg, msg_len);
    uint32_t check = (sig->r ^ pub_key) + sig->s;
    return (check % 997) == (h % 997);
}

bc_sig_t bc_sig_sign(uint32_t priv_key, const uint8_t *msg, int msg_len,
                     uint32_t k) {
    bc_sig_t sig;
    uint32_t h = bc_sig_hash_message(msg, msg_len);
    sig.r = (k * 31) ^ priv_key;
    sig.s = ((h % 997) - sig.r % 997 + 997) * k;
    return sig;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1218: Signature verify sim should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1218: Output should not be empty");
    assert!(
        code.contains("fn bc_sig_verify"),
        "C1218: Should contain bc_sig_verify function"
    );
}

#[test]
fn c1219_key_derivation() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef unsigned long size_t;

void bc_kdf_mix(uint8_t *out, const uint8_t *seed, int seed_len,
                uint32_t iteration) {
    int i;
    uint32_t state = iteration ^ 0xDEADBEEF;
    for (i = 0; i < 32; i++) {
        state ^= (uint32_t)seed[i % seed_len];
        state *= 0x5BD1E995;
        state ^= state >> 15;
        out[i] = (uint8_t)(state & 0xFF);
    }
}

void bc_derive_key(const uint8_t *master, int master_len,
                   uint32_t index, uint8_t *child_key) {
    int i;
    bc_kdf_mix(child_key, master, master_len, index);
    for (i = 0; i < 32; i++) {
        child_key[i] ^= master[i % master_len];
    }
}

int bc_key_nonzero(const uint8_t *key, int len) {
    int i;
    for (i = 0; i < len; i++) {
        if (key[i] != 0) return 1;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1219: Key derivation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1219: Output should not be empty");
    assert!(
        code.contains("fn bc_derive_key"),
        "C1219: Should contain bc_derive_key function"
    );
}

#[test]
fn c1220_random_seed_generator() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;

typedef struct {
    uint32_t state[4];
} bc_rng_t;

void bc_rng_init(bc_rng_t *rng, uint32_t seed) {
    rng->state[0] = seed;
    rng->state[1] = seed ^ 0x6C078965;
    rng->state[2] = seed ^ 0x9D2C5680;
    rng->state[3] = seed ^ 0xEFC60000;
}

uint32_t bc_rng_next(bc_rng_t *rng) {
    uint32_t t = rng->state[3];
    uint32_t s = rng->state[0];
    rng->state[3] = rng->state[2];
    rng->state[2] = rng->state[1];
    rng->state[1] = s;
    t ^= t << 11;
    t ^= t >> 8;
    rng->state[0] = t ^ s ^ (s >> 19);
    return rng->state[0];
}

void bc_rng_fill(bc_rng_t *rng, uint8_t *buf, int len) {
    int i;
    for (i = 0; i < len; i++) {
        if ((i % 4) == 0) {
            uint32_t r = bc_rng_next(rng);
            buf[i] = (uint8_t)(r & 0xFF);
        } else {
            buf[i] = (uint8_t)((bc_rng_next(rng) >> ((i % 4) * 8)) & 0xFF);
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1220: Random seed generator should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1220: Output should not be empty");
    assert!(
        code.contains("fn bc_rng_next"),
        "C1220: Should contain bc_rng_next function"
    );
}

// ============================================================================
// C1221-C1225: Distributed
// ============================================================================

#[test]
fn c1221_gossip_protocol() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t peer_id;
    uint32_t message_hash;
    uint32_t timestamp;
    int hop_count;
} bc_gossip_msg_t;

typedef struct {
    uint32_t seen_hashes[128];
    int seen_count;
    int max_hops;
} bc_gossip_state_t;

void bc_gossip_init(bc_gossip_state_t *state, int max_hops) {
    state->seen_count = 0;
    state->max_hops = max_hops;
}

int bc_gossip_already_seen(const bc_gossip_state_t *state, uint32_t hash) {
    int i;
    for (i = 0; i < state->seen_count; i++) {
        if (state->seen_hashes[i] == hash) return 1;
    }
    return 0;
}

int bc_gossip_receive(bc_gossip_state_t *state, bc_gossip_msg_t *msg) {
    if (msg->hop_count >= state->max_hops) return 0;
    if (bc_gossip_already_seen(state, msg->message_hash)) return 0;
    if (state->seen_count < 128) {
        state->seen_hashes[state->seen_count] = msg->message_hash;
        state->seen_count++;
    }
    msg->hop_count++;
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1221: Gossip protocol should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1221: Output should not be empty");
    assert!(
        code.contains("fn bc_gossip_receive"),
        "C1221: Should contain bc_gossip_receive function"
    );
}

#[test]
fn c1222_peer_table() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t ip_addr;
    uint32_t port;
    uint32_t last_seen;
    int score;
    int connected;
} bc_peer_t;

typedef struct {
    bc_peer_t peers[64];
    int count;
} bc_peer_table_t;

void bc_peer_table_init(bc_peer_table_t *table) {
    table->count = 0;
}

int bc_peer_add(bc_peer_table_t *table, uint32_t ip, uint32_t port) {
    int i;
    if (table->count >= 64) return -1;
    for (i = 0; i < table->count; i++) {
        if (table->peers[i].ip_addr == ip && table->peers[i].port == port) {
            return 0;
        }
    }
    table->peers[table->count].ip_addr = ip;
    table->peers[table->count].port = port;
    table->peers[table->count].last_seen = 0;
    table->peers[table->count].score = 0;
    table->peers[table->count].connected = 0;
    table->count++;
    return 1;
}

void bc_peer_update_score(bc_peer_table_t *table, int index, int delta) {
    if (index < 0 || index >= table->count) return;
    table->peers[index].score += delta;
    if (table->peers[index].score < -10) {
        table->peers[index].connected = 0;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1222: Peer table should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1222: Output should not be empty");
    assert!(
        code.contains("fn bc_peer_add"),
        "C1222: Should contain bc_peer_add function"
    );
}

#[test]
fn c1223_message_routing() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t dest_id;
    uint32_t next_hop;
    int metric;
} bc_route_entry_t;

typedef struct {
    bc_route_entry_t routes[32];
    int count;
} bc_routing_table_t;

void bc_routing_init(bc_routing_table_t *table) {
    table->count = 0;
}

int bc_routing_add(bc_routing_table_t *table, uint32_t dest,
                   uint32_t next_hop, int metric) {
    int i;
    if (table->count >= 32) return -1;
    for (i = 0; i < table->count; i++) {
        if (table->routes[i].dest_id == dest) {
            if (metric < table->routes[i].metric) {
                table->routes[i].next_hop = next_hop;
                table->routes[i].metric = metric;
            }
            return 0;
        }
    }
    table->routes[table->count].dest_id = dest;
    table->routes[table->count].next_hop = next_hop;
    table->routes[table->count].metric = metric;
    table->count++;
    return 1;
}

uint32_t bc_routing_lookup(const bc_routing_table_t *table, uint32_t dest) {
    int i;
    for (i = 0; i < table->count; i++) {
        if (table->routes[i].dest_id == dest) {
            return table->routes[i].next_hop;
        }
    }
    return 0xFFFFFFFF;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1223: Message routing should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1223: Output should not be empty");
    assert!(
        code.contains("fn bc_routing_lookup"),
        "C1223: Should contain bc_routing_lookup function"
    );
}

#[test]
fn c1224_bloom_filter_index() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned char uint8_t;
typedef unsigned long size_t;

typedef struct {
    uint8_t bits[256];
    int num_hashes;
} bc_bloom_t;

void bc_bloom_init(bc_bloom_t *bloom, int num_hashes) {
    int i;
    for (i = 0; i < 256; i++) bloom->bits[i] = 0;
    bloom->num_hashes = num_hashes;
}

uint32_t bc_bloom_hash(uint32_t seed, const uint8_t *data, size_t len) {
    uint32_t h = seed;
    size_t i;
    for (i = 0; i < len; i++) {
        h = h * 33 + data[i];
    }
    return h;
}

void bc_bloom_add(bc_bloom_t *bloom, const uint8_t *data, size_t len) {
    int i;
    for (i = 0; i < bloom->num_hashes; i++) {
        uint32_t h = bc_bloom_hash((uint32_t)i * 0x9E3779B9, data, len);
        uint32_t bit = h % 2048;
        bloom->bits[bit / 8] |= (uint8_t)(1 << (bit % 8));
    }
}

int bc_bloom_check(const bc_bloom_t *bloom, const uint8_t *data, size_t len) {
    int i;
    for (i = 0; i < bloom->num_hashes; i++) {
        uint32_t h = bc_bloom_hash((uint32_t)i * 0x9E3779B9, data, len);
        uint32_t bit = h % 2048;
        if (!(bloom->bits[bit / 8] & (1 << (bit % 8)))) return 0;
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1224: Bloom filter index should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1224: Output should not be empty");
    assert!(
        code.contains("fn bc_bloom_add"),
        "C1224: Should contain bc_bloom_add function"
    );
}

#[test]
fn c1225_consensus_voting() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t validator_id;
    uint32_t block_hash;
    int vote;
} bc_vote_t;

typedef struct {
    bc_vote_t votes[64];
    int vote_count;
    int total_validators;
} bc_voting_round_t;

void bc_voting_init(bc_voting_round_t *round, int total_validators) {
    round->vote_count = 0;
    round->total_validators = total_validators;
}

int bc_voting_add(bc_voting_round_t *round, uint32_t validator,
                  uint32_t block_hash, int vote) {
    int i;
    if (round->vote_count >= 64) return -1;
    for (i = 0; i < round->vote_count; i++) {
        if (round->votes[i].validator_id == validator) return 0;
    }
    round->votes[round->vote_count].validator_id = validator;
    round->votes[round->vote_count].block_hash = block_hash;
    round->votes[round->vote_count].vote = vote;
    round->vote_count++;
    return 1;
}

int bc_voting_has_quorum(const bc_voting_round_t *round, uint32_t block_hash) {
    int yes_votes = 0;
    int i;
    int threshold;
    for (i = 0; i < round->vote_count; i++) {
        if (round->votes[i].block_hash == block_hash && round->votes[i].vote) {
            yes_votes++;
        }
    }
    threshold = (round->total_validators * 2 + 2) / 3;
    return yes_votes >= threshold;
}

int bc_voting_is_finalized(const bc_voting_round_t *round) {
    return round->vote_count >= round->total_validators;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1225: Consensus voting should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1225: Output should not be empty");
    assert!(
        code.contains("fn bc_voting_has_quorum"),
        "C1225: Should contain bc_voting_has_quorum function"
    );
}
