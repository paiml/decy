//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1501-C1525: Distributed Systems Algorithms -- consensus protocols,
//! distributed hash tables, replication strategies, failure detection,
//! and scheduling algorithms commonly found in production distributed
//! systems infrastructure.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! Organization:
//! - C1501-C1505: Consensus (Raft leader election, Paxos prepare/accept, vector clocks, Lamport timestamps, two-phase commit)
//! - C1506-C1510: DHT (consistent hashing ring, virtual nodes, chord finger table, Kademlia XOR distance, gossip protocol)
//! - C1511-C1515: Replication (primary-backup, quorum read/write, anti-entropy, Merkle tree sync, CRDT G-counter)
//! - C1516-C1520: Failure detection (phi accrual detector, heartbeat monitor, SWIM protocol, WAL failure recovery, circuit breaker)
//! - C1521-C1525: Scheduling (work stealing deque, task DAG scheduler, fair share scheduler, load balancer, rate limiter token bucket)

// ============================================================================
// C1501-C1505: Consensus Protocols
// ============================================================================

/// C1501: Raft leader election with term management and vote tracking
#[test]
fn c1501_raft_leader_election() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

typedef enum {
    RAFT_FOLLOWER = 0,
    RAFT_CANDIDATE = 1,
    RAFT_LEADER = 2
} dist_raft_role_t;

typedef struct {
    uint32_t node_id;
    uint64_t current_term;
    int voted_for;
    dist_raft_role_t role;
    int votes_received;
    int cluster_size;
    uint64_t last_log_index;
    uint64_t last_log_term;
    int election_timeout_ms;
    int elapsed_ms;
} dist_raft_node_t;

void dist_raft_init(dist_raft_node_t *node, uint32_t id, int cluster_size) {
    node->node_id = id;
    node->current_term = 0;
    node->voted_for = -1;
    node->role = RAFT_FOLLOWER;
    node->votes_received = 0;
    node->cluster_size = cluster_size;
    node->last_log_index = 0;
    node->last_log_term = 0;
    node->election_timeout_ms = 150 + (id * 37) % 150;
    node->elapsed_ms = 0;
}

void dist_raft_start_election(dist_raft_node_t *node) {
    node->current_term = node->current_term + 1;
    node->role = RAFT_CANDIDATE;
    node->voted_for = (int)node->node_id;
    node->votes_received = 1;
    node->elapsed_ms = 0;
}

int dist_raft_has_quorum(const dist_raft_node_t *node) {
    return node->votes_received > node->cluster_size / 2;
}

void dist_raft_become_leader(dist_raft_node_t *node) {
    if (dist_raft_has_quorum(node)) {
        node->role = RAFT_LEADER;
    }
}

int dist_raft_handle_vote_request(dist_raft_node_t *node, uint64_t term,
                                   uint32_t candidate_id, uint64_t last_log_index,
                                   uint64_t last_log_term) {
    if (term < node->current_term) {
        return 0;
    }
    if (term > node->current_term) {
        node->current_term = term;
        node->role = RAFT_FOLLOWER;
        node->voted_for = -1;
    }
    if (node->voted_for == -1 || node->voted_for == (int)candidate_id) {
        if (last_log_term > node->last_log_term ||
            (last_log_term == node->last_log_term && last_log_index >= node->last_log_index)) {
            node->voted_for = (int)candidate_id;
            return 1;
        }
    }
    return 0;
}

void dist_raft_receive_vote(dist_raft_node_t *node, int granted) {
    if (granted && node->role == RAFT_CANDIDATE) {
        node->votes_received = node->votes_received + 1;
        dist_raft_become_leader(node);
    }
}

int dist_raft_tick(dist_raft_node_t *node, int delta_ms) {
    node->elapsed_ms = node->elapsed_ms + delta_ms;
    if (node->role != RAFT_LEADER && node->elapsed_ms >= node->election_timeout_ms) {
        dist_raft_start_election(node);
        return 1;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1501: Raft leader election should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1501: Output should not be empty");
    assert!(code.contains("fn dist_raft_init"), "C1501: Should contain dist_raft_init");
    assert!(code.contains("fn dist_raft_start_election"), "C1501: Should contain dist_raft_start_election");
    assert!(code.contains("fn dist_raft_handle_vote_request"), "C1501: Should contain dist_raft_handle_vote_request");
}

/// C1502: Paxos prepare/accept phases with proposal numbers and ballot tracking
#[test]
fn c1502_paxos_prepare_accept() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

typedef enum {
    PAXOS_NONE = 0,
    PAXOS_PREPARED = 1,
    PAXOS_ACCEPTED = 2,
    PAXOS_CHOSEN = 3
} dist_paxos_state_t;

typedef struct {
    uint64_t proposal_number;
    uint32_t proposer_id;
    int value;
} dist_paxos_proposal_t;

typedef struct {
    uint32_t node_id;
    uint64_t highest_promised;
    uint64_t highest_accepted_num;
    int accepted_value;
    int has_accepted;
    dist_paxos_state_t state;
} dist_paxos_acceptor_t;

typedef struct {
    uint32_t proposer_id;
    uint64_t round;
    int proposed_value;
    int prepare_acks;
    int accept_acks;
    int quorum_size;
    uint64_t highest_seen_accepted;
    int highest_seen_value;
    int has_seen_accepted;
} dist_paxos_proposer_t;

uint64_t dist_paxos_make_proposal_num(uint64_t round, uint32_t id) {
    return (round << 16) | (uint64_t)id;
}

void dist_paxos_acceptor_init(dist_paxos_acceptor_t *acc, uint32_t id) {
    acc->node_id = id;
    acc->highest_promised = 0;
    acc->highest_accepted_num = 0;
    acc->accepted_value = 0;
    acc->has_accepted = 0;
    acc->state = PAXOS_NONE;
}

int dist_paxos_handle_prepare(dist_paxos_acceptor_t *acc, uint64_t proposal_num,
                               uint64_t *out_accepted_num, int *out_accepted_val) {
    if (proposal_num > acc->highest_promised) {
        acc->highest_promised = proposal_num;
        acc->state = PAXOS_PREPARED;
        *out_accepted_num = acc->highest_accepted_num;
        *out_accepted_val = acc->accepted_value;
        return 1;
    }
    return 0;
}

int dist_paxos_handle_accept(dist_paxos_acceptor_t *acc, uint64_t proposal_num, int value) {
    if (proposal_num >= acc->highest_promised) {
        acc->highest_promised = proposal_num;
        acc->highest_accepted_num = proposal_num;
        acc->accepted_value = value;
        acc->has_accepted = 1;
        acc->state = PAXOS_ACCEPTED;
        return 1;
    }
    return 0;
}

void dist_paxos_proposer_init(dist_paxos_proposer_t *prop, uint32_t id, int quorum) {
    prop->proposer_id = id;
    prop->round = 0;
    prop->proposed_value = 0;
    prop->prepare_acks = 0;
    prop->accept_acks = 0;
    prop->quorum_size = quorum;
    prop->highest_seen_accepted = 0;
    prop->highest_seen_value = 0;
    prop->has_seen_accepted = 0;
}

void dist_paxos_proposer_on_promise(dist_paxos_proposer_t *prop,
                                     uint64_t accepted_num, int accepted_val) {
    prop->prepare_acks = prop->prepare_acks + 1;
    if (accepted_num > prop->highest_seen_accepted) {
        prop->highest_seen_accepted = accepted_num;
        prop->highest_seen_value = accepted_val;
        prop->has_seen_accepted = 1;
    }
}

int dist_paxos_proposer_ready(const dist_paxos_proposer_t *prop) {
    return prop->prepare_acks >= prop->quorum_size;
}

int dist_paxos_proposer_value(const dist_paxos_proposer_t *prop) {
    if (prop->has_seen_accepted) {
        return prop->highest_seen_value;
    }
    return prop->proposed_value;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1502: Paxos prepare/accept should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1502: Output should not be empty");
    assert!(code.contains("fn dist_paxos_handle_prepare"), "C1502: Should contain dist_paxos_handle_prepare");
    assert!(code.contains("fn dist_paxos_handle_accept"), "C1502: Should contain dist_paxos_handle_accept");
}

/// C1503: Vector clocks for causal ordering in distributed systems
#[test]
fn c1503_vector_clocks() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define DIST_VC_MAX_NODES 16

typedef struct {
    uint64_t clock[16];
    int num_nodes;
    int my_id;
} dist_vector_clock_t;

void dist_vc_init(dist_vector_clock_t *vc, int my_id, int num_nodes) {
    int i;
    vc->my_id = my_id;
    vc->num_nodes = num_nodes;
    for (i = 0; i < 16; i++) {
        vc->clock[i] = 0;
    }
}

void dist_vc_increment(dist_vector_clock_t *vc) {
    vc->clock[vc->my_id] = vc->clock[vc->my_id] + 1;
}

void dist_vc_merge(dist_vector_clock_t *vc, const uint64_t *other) {
    int i;
    for (i = 0; i < vc->num_nodes; i++) {
        if (other[i] > vc->clock[i]) {
            vc->clock[i] = other[i];
        }
    }
    dist_vc_increment(vc);
}

int dist_vc_happened_before(const dist_vector_clock_t *a, const dist_vector_clock_t *b) {
    int at_least_one_less = 0;
    int i;
    for (i = 0; i < a->num_nodes; i++) {
        if (a->clock[i] > b->clock[i]) {
            return 0;
        }
        if (a->clock[i] < b->clock[i]) {
            at_least_one_less = 1;
        }
    }
    return at_least_one_less;
}

int dist_vc_concurrent(const dist_vector_clock_t *a, const dist_vector_clock_t *b) {
    return !dist_vc_happened_before(a, b) && !dist_vc_happened_before(b, a);
}

int dist_vc_equal(const dist_vector_clock_t *a, const dist_vector_clock_t *b) {
    int i;
    if (a->num_nodes != b->num_nodes) return 0;
    for (i = 0; i < a->num_nodes; i++) {
        if (a->clock[i] != b->clock[i]) return 0;
    }
    return 1;
}

void dist_vc_copy(dist_vector_clock_t *dst, const dist_vector_clock_t *src) {
    int i;
    dst->my_id = src->my_id;
    dst->num_nodes = src->num_nodes;
    for (i = 0; i < 16; i++) {
        dst->clock[i] = src->clock[i];
    }
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1503: Vector clocks should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1503: Output should not be empty");
    assert!(code.contains("fn dist_vc_init"), "C1503: Should contain dist_vc_init");
    assert!(code.contains("fn dist_vc_merge"), "C1503: Should contain dist_vc_merge");
    assert!(code.contains("fn dist_vc_happened_before"), "C1503: Should contain dist_vc_happened_before");
}

/// C1504: Lamport logical timestamps with message passing simulation
#[test]
fn c1504_lamport_timestamps() {
    let c_code = r#"
typedef unsigned long uint64_t;
typedef unsigned int uint32_t;

typedef struct {
    uint64_t timestamp;
    uint32_t node_id;
} dist_lamport_t;

typedef struct {
    uint64_t send_ts;
    uint32_t sender_id;
    uint32_t receiver_id;
    int payload;
} dist_lamport_msg_t;

typedef struct {
    dist_lamport_msg_t messages[64];
    int count;
    int capacity;
} dist_lamport_log_t;

void dist_lamport_init(dist_lamport_t *lc, uint32_t id) {
    lc->timestamp = 0;
    lc->node_id = id;
}

uint64_t dist_lamport_tick(dist_lamport_t *lc) {
    lc->timestamp = lc->timestamp + 1;
    return lc->timestamp;
}

uint64_t dist_lamport_send(dist_lamport_t *lc) {
    return dist_lamport_tick(lc);
}

void dist_lamport_receive(dist_lamport_t *lc, uint64_t msg_ts) {
    if (msg_ts > lc->timestamp) {
        lc->timestamp = msg_ts;
    }
    lc->timestamp = lc->timestamp + 1;
}

int dist_lamport_compare(uint64_t ts_a, uint32_t id_a, uint64_t ts_b, uint32_t id_b) {
    if (ts_a < ts_b) return -1;
    if (ts_a > ts_b) return 1;
    if (id_a < id_b) return -1;
    if (id_a > id_b) return 1;
    return 0;
}

void dist_lamport_log_init(dist_lamport_log_t *log) {
    log->count = 0;
    log->capacity = 64;
}

int dist_lamport_log_append(dist_lamport_log_t *log, uint64_t ts,
                             uint32_t sender, uint32_t receiver, int payload) {
    if (log->count >= log->capacity) return 0;
    log->messages[log->count].send_ts = ts;
    log->messages[log->count].sender_id = sender;
    log->messages[log->count].receiver_id = receiver;
    log->messages[log->count].payload = payload;
    log->count = log->count + 1;
    return 1;
}

int dist_lamport_log_is_ordered(const dist_lamport_log_t *log) {
    int i;
    for (i = 1; i < log->count; i++) {
        if (log->messages[i].send_ts < log->messages[i - 1].send_ts) {
            return 0;
        }
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1504: Lamport timestamps should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1504: Output should not be empty");
    assert!(code.contains("fn dist_lamport_init"), "C1504: Should contain dist_lamport_init");
    assert!(code.contains("fn dist_lamport_receive"), "C1504: Should contain dist_lamport_receive");
}

/// C1505: Two-phase commit protocol with coordinator and participant state machines
#[test]
fn c1505_two_phase_commit() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef enum {
    TPC_INIT = 0,
    TPC_WAITING = 1,
    TPC_PREPARED = 2,
    TPC_COMMITTED = 3,
    TPC_ABORTED = 4
} dist_tpc_state_t;

typedef struct {
    uint32_t participant_id;
    dist_tpc_state_t state;
    int vote;
} dist_tpc_participant_t;

typedef struct {
    uint32_t txn_id;
    dist_tpc_state_t state;
    dist_tpc_participant_t participants[8];
    int num_participants;
    int votes_received;
    int all_yes;
} dist_tpc_coordinator_t;

void dist_tpc_coord_init(dist_tpc_coordinator_t *coord, uint32_t txn_id, int num_parts) {
    int i;
    coord->txn_id = txn_id;
    coord->state = TPC_INIT;
    coord->num_participants = num_parts;
    coord->votes_received = 0;
    coord->all_yes = 1;
    for (i = 0; i < num_parts && i < 8; i++) {
        coord->participants[i].participant_id = (uint32_t)i;
        coord->participants[i].state = TPC_INIT;
        coord->participants[i].vote = 0;
    }
}

void dist_tpc_begin_prepare(dist_tpc_coordinator_t *coord) {
    coord->state = TPC_WAITING;
}

void dist_tpc_receive_vote(dist_tpc_coordinator_t *coord, uint32_t part_id, int vote_yes) {
    if ((int)part_id < coord->num_participants) {
        coord->participants[part_id].vote = vote_yes;
        coord->participants[part_id].state = TPC_PREPARED;
        coord->votes_received = coord->votes_received + 1;
        if (!vote_yes) {
            coord->all_yes = 0;
        }
    }
}

int dist_tpc_all_voted(const dist_tpc_coordinator_t *coord) {
    return coord->votes_received >= coord->num_participants;
}

void dist_tpc_decide(dist_tpc_coordinator_t *coord) {
    int i;
    if (!dist_tpc_all_voted(coord)) return;
    if (coord->all_yes) {
        coord->state = TPC_COMMITTED;
        for (i = 0; i < coord->num_participants; i++) {
            coord->participants[i].state = TPC_COMMITTED;
        }
    } else {
        coord->state = TPC_ABORTED;
        for (i = 0; i < coord->num_participants; i++) {
            coord->participants[i].state = TPC_ABORTED;
        }
    }
}

void dist_tpc_part_init(dist_tpc_participant_t *part, uint32_t id) {
    part->participant_id = id;
    part->state = TPC_INIT;
    part->vote = 0;
}

int dist_tpc_part_prepare(dist_tpc_participant_t *part, int can_commit) {
    part->state = TPC_PREPARED;
    part->vote = can_commit;
    return can_commit;
}

void dist_tpc_part_commit(dist_tpc_participant_t *part) {
    part->state = TPC_COMMITTED;
}

void dist_tpc_part_abort(dist_tpc_participant_t *part) {
    part->state = TPC_ABORTED;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1505: Two-phase commit should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1505: Output should not be empty");
    assert!(code.contains("fn dist_tpc_coord_init"), "C1505: Should contain dist_tpc_coord_init");
    assert!(code.contains("fn dist_tpc_decide"), "C1505: Should contain dist_tpc_decide");
}

// ============================================================================
// C1506-C1510: Distributed Hash Tables (DHT)
// ============================================================================

/// C1506: Consistent hashing ring with node placement and key routing
#[test]
fn c1506_consistent_hashing_ring() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

typedef struct {
    uint32_t node_id;
    uint32_t hash_pos;
    int active;
} dist_ring_node_t;

typedef struct {
    dist_ring_node_t nodes[64];
    int num_nodes;
} dist_hash_ring_t;

uint32_t dist_ring_hash(uint32_t key) {
    key = ((key >> 16) ^ key) * 0x45d9f3b;
    key = ((key >> 16) ^ key) * 0x45d9f3b;
    key = (key >> 16) ^ key;
    return key;
}

void dist_ring_init(dist_hash_ring_t *ring) {
    ring->num_nodes = 0;
}

int dist_ring_add_node(dist_hash_ring_t *ring, uint32_t node_id) {
    int i;
    uint32_t pos;
    if (ring->num_nodes >= 64) return 0;
    pos = dist_ring_hash(node_id);
    i = ring->num_nodes;
    while (i > 0 && ring->nodes[i - 1].hash_pos > pos) {
        ring->nodes[i] = ring->nodes[i - 1];
        i = i - 1;
    }
    ring->nodes[i].node_id = node_id;
    ring->nodes[i].hash_pos = pos;
    ring->nodes[i].active = 1;
    ring->num_nodes = ring->num_nodes + 1;
    return 1;
}

uint32_t dist_ring_find_node(const dist_hash_ring_t *ring, uint32_t key) {
    uint32_t hash = dist_ring_hash(key);
    int lo = 0;
    int hi = ring->num_nodes - 1;
    int mid;
    if (ring->num_nodes == 0) return 0;
    if (hash > ring->nodes[hi].hash_pos) {
        return ring->nodes[0].node_id;
    }
    while (lo < hi) {
        mid = lo + (hi - lo) / 2;
        if (ring->nodes[mid].hash_pos < hash) {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    return ring->nodes[lo].node_id;
}

int dist_ring_remove_node(dist_hash_ring_t *ring, uint32_t node_id) {
    int i, found = -1;
    for (i = 0; i < ring->num_nodes; i++) {
        if (ring->nodes[i].node_id == node_id) {
            found = i;
            break;
        }
    }
    if (found < 0) return 0;
    for (i = found; i < ring->num_nodes - 1; i++) {
        ring->nodes[i] = ring->nodes[i + 1];
    }
    ring->num_nodes = ring->num_nodes - 1;
    return 1;
}

int dist_ring_node_count(const dist_hash_ring_t *ring) {
    return ring->num_nodes;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1506: Consistent hashing ring should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1506: Output should not be empty");
    assert!(code.contains("fn dist_ring_find_node"), "C1506: Should contain dist_ring_find_node");
    assert!(code.contains("fn dist_ring_add_node"), "C1506: Should contain dist_ring_add_node");
}

/// C1507: Virtual nodes for balanced distribution on a consistent hash ring
#[test]
fn c1507_virtual_nodes() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t physical_id;
    uint32_t virtual_id;
    uint32_t hash_pos;
} dist_vnode_t;

typedef struct {
    dist_vnode_t vnodes[256];
    int count;
    int vnodes_per_physical;
} dist_vnode_ring_t;

uint32_t dist_vnode_hash(uint32_t key, uint32_t seed) {
    uint32_t h = key ^ seed;
    h = h * 0x9e3779b9;
    h = h ^ (h >> 16);
    h = h * 0x85ebca6b;
    h = h ^ (h >> 13);
    return h;
}

void dist_vnode_ring_init(dist_vnode_ring_t *ring, int vnodes_per_phys) {
    ring->count = 0;
    ring->vnodes_per_physical = vnodes_per_phys;
}

int dist_vnode_add_physical(dist_vnode_ring_t *ring, uint32_t phys_id) {
    int i, pos;
    uint32_t h;
    int added = 0;
    for (i = 0; i < ring->vnodes_per_physical; i++) {
        if (ring->count >= 256) break;
        h = dist_vnode_hash(phys_id, (uint32_t)i);
        pos = ring->count;
        while (pos > 0 && ring->vnodes[pos - 1].hash_pos > h) {
            ring->vnodes[pos] = ring->vnodes[pos - 1];
            pos = pos - 1;
        }
        ring->vnodes[pos].physical_id = phys_id;
        ring->vnodes[pos].virtual_id = (uint32_t)i;
        ring->vnodes[pos].hash_pos = h;
        ring->count = ring->count + 1;
        added = added + 1;
    }
    return added;
}

uint32_t dist_vnode_lookup(const dist_vnode_ring_t *ring, uint32_t key) {
    uint32_t h = dist_vnode_hash(key, 0);
    int lo = 0, hi = ring->count - 1, mid;
    if (ring->count == 0) return 0;
    if (h > ring->vnodes[hi].hash_pos) {
        return ring->vnodes[0].physical_id;
    }
    while (lo < hi) {
        mid = lo + (hi - lo) / 2;
        if (ring->vnodes[mid].hash_pos < h) lo = mid + 1;
        else hi = mid;
    }
    return ring->vnodes[lo].physical_id;
}

int dist_vnode_count_for_physical(const dist_vnode_ring_t *ring, uint32_t phys_id) {
    int count = 0;
    int i;
    for (i = 0; i < ring->count; i++) {
        if (ring->vnodes[i].physical_id == phys_id) count = count + 1;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1507: Virtual nodes should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1507: Output should not be empty");
    assert!(code.contains("fn dist_vnode_add_physical"), "C1507: Should contain dist_vnode_add_physical");
    assert!(code.contains("fn dist_vnode_lookup"), "C1507: Should contain dist_vnode_lookup");
}

/// C1508: Chord finger table for O(log N) DHT lookups
#[test]
fn c1508_chord_finger_table() {
    let c_code = r##"
typedef unsigned int uint32_t;

#define DIST_CHORD_BITS 8
#define DIST_CHORD_RING_SIZE 256

typedef struct {
    uint32_t start;
    uint32_t node;
} dist_chord_finger_t;

typedef struct {
    uint32_t node_id;
    uint32_t successor;
    uint32_t predecessor;
    dist_chord_finger_t fingers[8];
} dist_chord_node_t;

int dist_chord_in_range(uint32_t id, uint32_t start, uint32_t end) {
    if (start < end) {
        return id > start && id <= end;
    }
    return id > start || id <= end;
}

void dist_chord_init(dist_chord_node_t *node, uint32_t id) {
    int i;
    node->node_id = id;
    node->successor = id;
    node->predecessor = id;
    for (i = 0; i < 8; i++) {
        node->fingers[i].start = (id + (1 << i)) % 256;
        node->fingers[i].node = id;
    }
}

uint32_t dist_chord_closest_preceding(const dist_chord_node_t *node, uint32_t key) {
    int i;
    for (i = 7; i >= 0; i--) {
        if (dist_chord_in_range(node->fingers[i].node, node->node_id, key)) {
            return node->fingers[i].node;
        }
    }
    return node->node_id;
}

uint32_t dist_chord_find_successor(const dist_chord_node_t *node, uint32_t key) {
    if (dist_chord_in_range(key, node->node_id, node->successor)) {
        return node->successor;
    }
    return dist_chord_closest_preceding(node, key);
}

void dist_chord_update_finger(dist_chord_node_t *node, int idx, uint32_t new_node) {
    if (idx >= 0 && idx < 8) {
        node->fingers[idx].node = new_node;
    }
}

void dist_chord_set_successor(dist_chord_node_t *node, uint32_t succ) {
    node->successor = succ;
    node->fingers[0].node = succ;
}

void dist_chord_set_predecessor(dist_chord_node_t *node, uint32_t pred) {
    node->predecessor = pred;
}

int dist_chord_stabilize_check(const dist_chord_node_t *node, uint32_t x) {
    return dist_chord_in_range(x, node->node_id, node->successor);
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1508: Chord finger table should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1508: Output should not be empty");
    assert!(code.contains("fn dist_chord_init"), "C1508: Should contain dist_chord_init");
    assert!(code.contains("fn dist_chord_find_successor"), "C1508: Should contain dist_chord_find_successor");
}

/// C1509: Kademlia XOR distance metric and k-bucket routing
#[test]
fn c1509_kademlia_xor_distance() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t id;
    int active;
    int last_seen;
} dist_kad_contact_t;

typedef struct {
    dist_kad_contact_t entries[8];
    int count;
    int k;
} dist_kad_bucket_t;

typedef struct {
    uint32_t self_id;
    dist_kad_bucket_t buckets[32];
    int num_buckets;
} dist_kad_table_t;

uint32_t dist_kad_xor_distance(uint32_t a, uint32_t b) {
    return a ^ b;
}

int dist_kad_log2_distance(uint32_t distance) {
    int bit = 0;
    if (distance == 0) return -1;
    while (distance > 1) {
        distance = distance >> 1;
        bit = bit + 1;
    }
    return bit;
}

int dist_kad_bucket_index(uint32_t self_id, uint32_t other_id) {
    uint32_t dist = dist_kad_xor_distance(self_id, other_id);
    return dist_kad_log2_distance(dist);
}

void dist_kad_bucket_init(dist_kad_bucket_t *bucket, int k) {
    bucket->count = 0;
    bucket->k = k;
}

int dist_kad_bucket_add(dist_kad_bucket_t *bucket, uint32_t id, int timestamp) {
    int i;
    for (i = 0; i < bucket->count; i++) {
        if (bucket->entries[i].id == id) {
            bucket->entries[i].last_seen = timestamp;
            return 1;
        }
    }
    if (bucket->count < bucket->k) {
        bucket->entries[bucket->count].id = id;
        bucket->entries[bucket->count].active = 1;
        bucket->entries[bucket->count].last_seen = timestamp;
        bucket->count = bucket->count + 1;
        return 1;
    }
    return 0;
}

void dist_kad_table_init(dist_kad_table_t *table, uint32_t self_id) {
    int i;
    table->self_id = self_id;
    table->num_buckets = 32;
    for (i = 0; i < 32; i++) {
        dist_kad_bucket_init(&table->buckets[i], 8);
    }
}

int dist_kad_table_insert(dist_kad_table_t *table, uint32_t peer_id, int timestamp) {
    int idx = dist_kad_bucket_index(table->self_id, peer_id);
    if (idx < 0 || idx >= table->num_buckets) return 0;
    return dist_kad_bucket_add(&table->buckets[idx], peer_id, timestamp);
}

int dist_kad_find_closest(const dist_kad_table_t *table, uint32_t target,
                           uint32_t *results, int max_results) {
    int count = 0;
    int idx = dist_kad_bucket_index(table->self_id, target);
    int i, offset;
    if (idx < 0) idx = 0;
    for (i = 0; i < table->buckets[idx].count && count < max_results; i++) {
        results[count] = table->buckets[idx].entries[i].id;
        count = count + 1;
    }
    for (offset = 1; offset < table->num_buckets && count < max_results; offset++) {
        int lo = idx - offset;
        int hi = idx + offset;
        if (lo >= 0) {
            for (i = 0; i < table->buckets[lo].count && count < max_results; i++) {
                results[count] = table->buckets[lo].entries[i].id;
                count = count + 1;
            }
        }
        if (hi < table->num_buckets) {
            for (i = 0; i < table->buckets[hi].count && count < max_results; i++) {
                results[count] = table->buckets[hi].entries[i].id;
                count = count + 1;
            }
        }
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1509: Kademlia XOR distance should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1509: Output should not be empty");
    assert!(code.contains("fn dist_kad_xor_distance"), "C1509: Should contain dist_kad_xor_distance");
    assert!(code.contains("fn dist_kad_table_insert"), "C1509: Should contain dist_kad_table_insert");
}

/// C1510: Gossip protocol with infection-style dissemination and rumor mongering
#[test]
fn c1510_gossip_protocol() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

typedef struct {
    uint32_t key;
    uint32_t value;
    uint64_t version;
    uint32_t origin_node;
} dist_gossip_entry_t;

typedef struct {
    dist_gossip_entry_t entries[128];
    int count;
    uint32_t node_id;
    int fanout;
    uint64_t local_version;
} dist_gossip_state_t;

void dist_gossip_init(dist_gossip_state_t *state, uint32_t id, int fanout) {
    state->count = 0;
    state->node_id = id;
    state->fanout = fanout;
    state->local_version = 0;
}

int dist_gossip_find(const dist_gossip_state_t *state, uint32_t key) {
    int i;
    for (i = 0; i < state->count; i++) {
        if (state->entries[i].key == key) return i;
    }
    return -1;
}

int dist_gossip_update(dist_gossip_state_t *state, uint32_t key, uint32_t value) {
    int idx = dist_gossip_find(state, key);
    state->local_version = state->local_version + 1;
    if (idx >= 0) {
        state->entries[idx].value = value;
        state->entries[idx].version = state->local_version;
        state->entries[idx].origin_node = state->node_id;
        return 1;
    }
    if (state->count >= 128) return 0;
    state->entries[state->count].key = key;
    state->entries[state->count].value = value;
    state->entries[state->count].version = state->local_version;
    state->entries[state->count].origin_node = state->node_id;
    state->count = state->count + 1;
    return 1;
}

int dist_gossip_merge_entry(dist_gossip_state_t *state, const dist_gossip_entry_t *entry) {
    int idx = dist_gossip_find(state, entry->key);
    if (idx >= 0) {
        if (entry->version > state->entries[idx].version) {
            state->entries[idx].value = entry->value;
            state->entries[idx].version = entry->version;
            state->entries[idx].origin_node = entry->origin_node;
            return 1;
        }
        return 0;
    }
    if (state->count >= 128) return 0;
    state->entries[state->count] = *entry;
    state->count = state->count + 1;
    return 1;
}

int dist_gossip_merge_digest(dist_gossip_state_t *state,
                              const dist_gossip_entry_t *remote, int remote_count) {
    int merged = 0;
    int i;
    for (i = 0; i < remote_count; i++) {
        merged = merged + dist_gossip_merge_entry(state, &remote[i]);
    }
    return merged;
}

int dist_gossip_select_peers(const dist_gossip_state_t *state,
                              uint32_t *peers, int max_peers, uint32_t seed) {
    int selected = 0;
    uint32_t rng = seed;
    int i;
    for (i = 0; i < state->fanout && selected < max_peers; i++) {
        rng = rng * 1103515245 + 12345;
        peers[selected] = rng;
        selected = selected + 1;
    }
    return selected;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1510: Gossip protocol should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1510: Output should not be empty");
    assert!(code.contains("fn dist_gossip_init"), "C1510: Should contain dist_gossip_init");
    assert!(code.contains("fn dist_gossip_merge_entry"), "C1510: Should contain dist_gossip_merge_entry");
}

// ============================================================================
// C1511-C1515: Replication Strategies
// ============================================================================

/// C1511: Primary-backup replication with log-based state transfer
#[test]
fn c1511_primary_backup_replication() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

typedef enum {
    REPLICA_PRIMARY = 0,
    REPLICA_BACKUP = 1,
    REPLICA_SYNCING = 2,
    REPLICA_FAILED = 3
} dist_replica_role_t;

typedef struct {
    uint32_t seq;
    int op_type;
    uint32_t key;
    int value;
} dist_repl_log_entry_t;

typedef struct {
    uint32_t node_id;
    dist_replica_role_t role;
    dist_repl_log_entry_t log[256];
    int log_len;
    uint32_t committed_seq;
    uint32_t applied_seq;
    int data[64];
} dist_replica_t;

void dist_replica_init(dist_replica_t *r, uint32_t id, dist_replica_role_t role) {
    int i;
    r->node_id = id;
    r->role = role;
    r->log_len = 0;
    r->committed_seq = 0;
    r->applied_seq = 0;
    for (i = 0; i < 64; i++) r->data[i] = 0;
}

int dist_replica_append(dist_replica_t *r, int op_type, uint32_t key, int value) {
    if (r->role != REPLICA_PRIMARY) return 0;
    if (r->log_len >= 256) return 0;
    r->log[r->log_len].seq = (uint32_t)(r->log_len + 1);
    r->log[r->log_len].op_type = op_type;
    r->log[r->log_len].key = key;
    r->log[r->log_len].value = value;
    r->log_len = r->log_len + 1;
    return 1;
}

int dist_replica_replicate(dist_replica_t *backup, const dist_repl_log_entry_t *entry) {
    if (backup->role == REPLICA_FAILED) return 0;
    if (backup->log_len >= 256) return 0;
    backup->log[backup->log_len] = *entry;
    backup->log_len = backup->log_len + 1;
    return 1;
}

void dist_replica_commit(dist_replica_t *r, uint32_t seq) {
    if (seq > r->committed_seq) {
        r->committed_seq = seq;
    }
}

int dist_replica_apply(dist_replica_t *r) {
    int applied = 0;
    while (r->applied_seq < r->committed_seq && (int)r->applied_seq < r->log_len) {
        uint32_t idx = r->applied_seq;
        uint32_t key = r->log[idx].key;
        if (key < 64) {
            if (r->log[idx].op_type == 1) {
                r->data[key] = r->log[idx].value;
            } else if (r->log[idx].op_type == 2) {
                r->data[key] = 0;
            }
        }
        r->applied_seq = r->applied_seq + 1;
        applied = applied + 1;
    }
    return applied;
}

void dist_replica_promote(dist_replica_t *r) {
    r->role = REPLICA_PRIMARY;
}

int dist_replica_lag(const dist_replica_t *r) {
    return r->log_len - (int)r->applied_seq;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1511: Primary-backup replication should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1511: Output should not be empty");
    assert!(code.contains("fn dist_replica_init"), "C1511: Should contain dist_replica_init");
    assert!(code.contains("fn dist_replica_apply"), "C1511: Should contain dist_replica_apply");
}

/// C1512: Quorum-based read/write with configurable R, W, N
#[test]
fn c1512_quorum_read_write() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

typedef struct {
    uint32_t key;
    int value;
    uint64_t version;
} dist_quorum_entry_t;

typedef struct {
    dist_quorum_entry_t store[128];
    int count;
    uint32_t node_id;
} dist_quorum_node_t;

typedef struct {
    int n;
    int r;
    int w;
} dist_quorum_config_t;

void dist_quorum_config_init(dist_quorum_config_t *cfg, int n, int r, int w) {
    cfg->n = n;
    cfg->r = r;
    cfg->w = w;
}

int dist_quorum_valid(const dist_quorum_config_t *cfg) {
    if (cfg->r + cfg->w <= cfg->n) return 0;
    if (cfg->r > cfg->n || cfg->w > cfg->n) return 0;
    return 1;
}

void dist_quorum_node_init(dist_quorum_node_t *node, uint32_t id) {
    node->count = 0;
    node->node_id = id;
}

int dist_quorum_find(const dist_quorum_node_t *node, uint32_t key) {
    int i;
    for (i = 0; i < node->count; i++) {
        if (node->store[i].key == key) return i;
    }
    return -1;
}

int dist_quorum_write(dist_quorum_node_t *node, uint32_t key, int value, uint64_t version) {
    int idx = dist_quorum_find(node, key);
    if (idx >= 0) {
        if (version > node->store[idx].version) {
            node->store[idx].value = value;
            node->store[idx].version = version;
            return 1;
        }
        return 0;
    }
    if (node->count >= 128) return 0;
    node->store[node->count].key = key;
    node->store[node->count].value = value;
    node->store[node->count].version = version;
    node->count = node->count + 1;
    return 1;
}

int dist_quorum_read(const dist_quorum_node_t *node, uint32_t key,
                      int *out_value, uint64_t *out_version) {
    int idx = dist_quorum_find(node, key);
    if (idx < 0) return 0;
    *out_value = node->store[idx].value;
    *out_version = node->store[idx].version;
    return 1;
}

int dist_quorum_resolve(const int *values, const uint64_t *versions, int count,
                         int *out_value) {
    int best = 0;
    uint64_t best_ver = 0;
    int i;
    for (i = 0; i < count; i++) {
        if (versions[i] > best_ver) {
            best_ver = versions[i];
            best = i;
        }
    }
    *out_value = values[best];
    return (int)best_ver > 0;
}

int dist_quorum_has_strong_consistency(const dist_quorum_config_t *cfg) {
    return cfg->r + cfg->w > cfg->n;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1512: Quorum read/write should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1512: Output should not be empty");
    assert!(code.contains("fn dist_quorum_write"), "C1512: Should contain dist_quorum_write");
    assert!(code.contains("fn dist_quorum_resolve"), "C1512: Should contain dist_quorum_resolve");
}

/// C1513: Anti-entropy protocol for replica synchronization
#[test]
fn c1513_anti_entropy() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

typedef struct {
    uint32_t key;
    uint64_t version;
    int value;
    int tombstone;
} dist_ae_entry_t;

typedef struct {
    dist_ae_entry_t entries[256];
    int count;
    uint32_t node_id;
    uint64_t sync_generation;
} dist_ae_store_t;

void dist_ae_init(dist_ae_store_t *store, uint32_t id) {
    store->count = 0;
    store->node_id = id;
    store->sync_generation = 0;
}

int dist_ae_find(const dist_ae_store_t *store, uint32_t key) {
    int i;
    for (i = 0; i < store->count; i++) {
        if (store->entries[i].key == key) return i;
    }
    return -1;
}

int dist_ae_put(dist_ae_store_t *store, uint32_t key, int value, uint64_t version) {
    int idx = dist_ae_find(store, key);
    if (idx >= 0) {
        if (version > store->entries[idx].version) {
            store->entries[idx].value = value;
            store->entries[idx].version = version;
            store->entries[idx].tombstone = 0;
            return 1;
        }
        return 0;
    }
    if (store->count >= 256) return 0;
    store->entries[store->count].key = key;
    store->entries[store->count].value = value;
    store->entries[store->count].version = version;
    store->entries[store->count].tombstone = 0;
    store->count = store->count + 1;
    return 1;
}

int dist_ae_delete(dist_ae_store_t *store, uint32_t key, uint64_t version) {
    int idx = dist_ae_find(store, key);
    if (idx >= 0 && version > store->entries[idx].version) {
        store->entries[idx].tombstone = 1;
        store->entries[idx].version = version;
        return 1;
    }
    return 0;
}

int dist_ae_compute_diff(const dist_ae_store_t *local, const dist_ae_store_t *remote,
                          int *diff_indices, int max_diff) {
    int diff_count = 0;
    int i, j;
    for (i = 0; i < local->count && diff_count < max_diff; i++) {
        int found = 0;
        for (j = 0; j < remote->count; j++) {
            if (local->entries[i].key == remote->entries[j].key) {
                found = 1;
                if (local->entries[i].version != remote->entries[j].version) {
                    diff_indices[diff_count] = i;
                    diff_count = diff_count + 1;
                }
                break;
            }
        }
        if (!found) {
            diff_indices[diff_count] = i;
            diff_count = diff_count + 1;
        }
    }
    return diff_count;
}

int dist_ae_sync(dist_ae_store_t *dst, const dist_ae_store_t *src) {
    int synced = 0;
    int i;
    for (i = 0; i < src->count; i++) {
        int idx = dist_ae_find(dst, src->entries[i].key);
        if (idx < 0) {
            if (dst->count < 256) {
                dst->entries[dst->count] = src->entries[i];
                dst->count = dst->count + 1;
                synced = synced + 1;
            }
        } else if (src->entries[i].version > dst->entries[idx].version) {
            dst->entries[idx] = src->entries[i];
            synced = synced + 1;
        }
    }
    dst->sync_generation = dst->sync_generation + 1;
    return synced;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1513: Anti-entropy should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1513: Output should not be empty");
    assert!(code.contains("fn dist_ae_sync"), "C1513: Should contain dist_ae_sync");
    assert!(code.contains("fn dist_ae_compute_diff"), "C1513: Should contain dist_ae_compute_diff");
}

/// C1514: Merkle tree for efficient replica divergence detection
#[test]
fn c1514_merkle_tree_sync() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t hash;
    int is_leaf;
    uint32_t key;
    int value;
} dist_merkle_node_t;

typedef struct {
    dist_merkle_node_t nodes[255];
    int depth;
    int num_leaves;
} dist_merkle_tree_t;

uint32_t dist_merkle_hash(uint32_t a, uint32_t b) {
    uint32_t h = a * 0x9e3779b9 + b;
    h = h ^ (h >> 16);
    h = h * 0x85ebca6b;
    h = h ^ (h >> 13);
    return h;
}

uint32_t dist_merkle_leaf_hash(uint32_t key, int value) {
    return dist_merkle_hash(key, (uint32_t)value);
}

void dist_merkle_init(dist_merkle_tree_t *tree, int depth) {
    int i;
    int total = 1;
    int d;
    tree->depth = depth;
    for (d = 0; d < depth; d++) total = total * 2;
    tree->num_leaves = total;
    for (i = 0; i < 255; i++) {
        tree->nodes[i].hash = 0;
        tree->nodes[i].is_leaf = 0;
        tree->nodes[i].key = 0;
        tree->nodes[i].value = 0;
    }
    for (i = total - 1; i < 2 * total - 1 && i < 255; i++) {
        tree->nodes[i].is_leaf = 1;
    }
}

void dist_merkle_set_leaf(dist_merkle_tree_t *tree, int idx, uint32_t key, int value) {
    int leaf_offset = tree->num_leaves - 1;
    int pos = leaf_offset + idx;
    if (pos >= 255) return;
    tree->nodes[pos].key = key;
    tree->nodes[pos].value = value;
    tree->nodes[pos].hash = dist_merkle_leaf_hash(key, value);
}

void dist_merkle_rebuild(dist_merkle_tree_t *tree) {
    int i;
    int leaf_offset = tree->num_leaves - 1;
    for (i = leaf_offset - 1; i >= 0; i--) {
        int left = 2 * i + 1;
        int right = 2 * i + 2;
        if (left < 255 && right < 255) {
            tree->nodes[i].hash = dist_merkle_hash(
                tree->nodes[left].hash, tree->nodes[right].hash);
        }
    }
}

int dist_merkle_compare_roots(const dist_merkle_tree_t *a, const dist_merkle_tree_t *b) {
    return a->nodes[0].hash == b->nodes[0].hash;
}

int dist_merkle_find_diffs(const dist_merkle_tree_t *a, const dist_merkle_tree_t *b,
                            int *diff_leaves, int max_diffs) {
    int stack[32];
    int top = 0;
    int count = 0;
    int leaf_offset = a->num_leaves - 1;
    stack[top] = 0;
    top = top + 1;
    while (top > 0 && count < max_diffs) {
        int node;
        top = top - 1;
        node = stack[top];
        if (node >= 255) continue;
        if (a->nodes[node].hash == b->nodes[node].hash) continue;
        if (node >= leaf_offset) {
            diff_leaves[count] = node - leaf_offset;
            count = count + 1;
        } else {
            int left = 2 * node + 1;
            int right = 2 * node + 2;
            if (top < 30) { stack[top] = left; top = top + 1; }
            if (top < 30) { stack[top] = right; top = top + 1; }
        }
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1514: Merkle tree sync should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1514: Output should not be empty");
    assert!(code.contains("fn dist_merkle_rebuild"), "C1514: Should contain dist_merkle_rebuild");
    assert!(code.contains("fn dist_merkle_find_diffs"), "C1514: Should contain dist_merkle_find_diffs");
}

/// C1515: CRDT G-Counter (grow-only counter) with merge semantics
#[test]
fn c1515_crdt_gcounter() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define DIST_CRDT_MAX_NODES 16

typedef struct {
    uint64_t counts[16];
    int num_nodes;
    int my_id;
} dist_gcounter_t;

void dist_gcounter_init(dist_gcounter_t *gc, int my_id, int num_nodes) {
    int i;
    gc->my_id = my_id;
    gc->num_nodes = num_nodes;
    for (i = 0; i < 16; i++) {
        gc->counts[i] = 0;
    }
}

void dist_gcounter_increment(dist_gcounter_t *gc) {
    gc->counts[gc->my_id] = gc->counts[gc->my_id] + 1;
}

void dist_gcounter_increment_by(dist_gcounter_t *gc, uint64_t amount) {
    gc->counts[gc->my_id] = gc->counts[gc->my_id] + amount;
}

uint64_t dist_gcounter_value(const dist_gcounter_t *gc) {
    uint64_t total = 0;
    int i;
    for (i = 0; i < gc->num_nodes; i++) {
        total = total + gc->counts[i];
    }
    return total;
}

void dist_gcounter_merge(dist_gcounter_t *gc, const dist_gcounter_t *other) {
    int i;
    for (i = 0; i < gc->num_nodes; i++) {
        if (other->counts[i] > gc->counts[i]) {
            gc->counts[i] = other->counts[i];
        }
    }
}

int dist_gcounter_compare(const dist_gcounter_t *a, const dist_gcounter_t *b) {
    int a_leq_b = 1;
    int b_leq_a = 1;
    int i;
    for (i = 0; i < a->num_nodes; i++) {
        if (a->counts[i] > b->counts[i]) a_leq_b = 0;
        if (b->counts[i] > a->counts[i]) b_leq_a = 0;
    }
    if (a_leq_b && b_leq_a) return 0;
    if (a_leq_b) return -1;
    if (b_leq_a) return 1;
    return 2;
}

int dist_gcounter_equal(const dist_gcounter_t *a, const dist_gcounter_t *b) {
    return dist_gcounter_compare(a, b) == 0;
}

uint64_t dist_gcounter_local_count(const dist_gcounter_t *gc) {
    return gc->counts[gc->my_id];
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1515: CRDT G-Counter should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1515: Output should not be empty");
    assert!(code.contains("fn dist_gcounter_merge"), "C1515: Should contain dist_gcounter_merge");
    assert!(code.contains("fn dist_gcounter_value"), "C1515: Should contain dist_gcounter_value");
}

// ============================================================================
// C1516-C1520: Failure Detection
// ============================================================================

/// C1516: Phi accrual failure detector with adaptive suspicion thresholds
#[test]
fn c1516_phi_accrual_detector() {
    let c_code = r#"
typedef unsigned long uint64_t;
typedef unsigned int uint32_t;

typedef struct {
    uint64_t intervals[64];
    int count;
    int head;
    int capacity;
    uint64_t last_heartbeat;
    uint64_t sum;
    uint64_t sum_sq;
} dist_phi_detector_t;

void dist_phi_init(dist_phi_detector_t *det) {
    int i;
    det->count = 0;
    det->head = 0;
    det->capacity = 64;
    det->last_heartbeat = 0;
    det->sum = 0;
    det->sum_sq = 0;
    for (i = 0; i < 64; i++) det->intervals[i] = 0;
}

void dist_phi_heartbeat(dist_phi_detector_t *det, uint64_t now) {
    if (det->last_heartbeat > 0) {
        uint64_t interval = now - det->last_heartbeat;
        if (det->count >= det->capacity) {
            int oldest = (det->head + 1) % det->capacity;
            det->sum = det->sum - det->intervals[oldest];
            det->sum_sq = det->sum_sq - det->intervals[oldest] * det->intervals[oldest];
        } else {
            det->count = det->count + 1;
        }
        det->head = (det->head + 1) % det->capacity;
        det->intervals[det->head] = interval;
        det->sum = det->sum + interval;
        det->sum_sq = det->sum_sq + interval * interval;
    }
    det->last_heartbeat = now;
}

uint64_t dist_phi_mean(const dist_phi_detector_t *det) {
    if (det->count == 0) return 1000;
    return det->sum / (uint64_t)det->count;
}

uint64_t dist_phi_variance(const dist_phi_detector_t *det) {
    uint64_t mean;
    if (det->count < 2) return 100;
    mean = dist_phi_mean(det);
    return det->sum_sq / (uint64_t)det->count - mean * mean;
}

int dist_phi_value(const dist_phi_detector_t *det, uint64_t now) {
    uint64_t elapsed;
    uint64_t mean;
    uint64_t diff;
    if (det->last_heartbeat == 0 || det->count == 0) return 0;
    elapsed = now - det->last_heartbeat;
    mean = dist_phi_mean(det);
    if (elapsed <= mean) return 0;
    diff = elapsed - mean;
    if (mean == 0) return 100;
    return (int)((diff * 10) / mean);
}

int dist_phi_is_alive(const dist_phi_detector_t *det, uint64_t now, int threshold) {
    return dist_phi_value(det, now) < threshold;
}

int dist_phi_sample_count(const dist_phi_detector_t *det) {
    return det->count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1516: Phi accrual detector should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1516: Output should not be empty");
    assert!(code.contains("fn dist_phi_heartbeat"), "C1516: Should contain dist_phi_heartbeat");
    assert!(code.contains("fn dist_phi_is_alive"), "C1516: Should contain dist_phi_is_alive");
}

/// C1517: Heartbeat monitor with timeout-based failure detection
#[test]
fn c1517_heartbeat_monitor() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

typedef enum {
    HB_ALIVE = 0,
    HB_SUSPECTED = 1,
    HB_FAILED = 2,
    HB_UNKNOWN = 3
} dist_hb_status_t;

typedef struct {
    uint32_t node_id;
    uint64_t last_heartbeat;
    dist_hb_status_t status;
    int miss_count;
    int heartbeat_count;
} dist_hb_entry_t;

typedef struct {
    dist_hb_entry_t nodes[32];
    int num_nodes;
    uint64_t timeout_ms;
    int suspect_threshold;
    int fail_threshold;
} dist_hb_monitor_t;

void dist_hb_monitor_init(dist_hb_monitor_t *mon, uint64_t timeout, int suspect_thresh, int fail_thresh) {
    mon->num_nodes = 0;
    mon->timeout_ms = timeout;
    mon->suspect_threshold = suspect_thresh;
    mon->fail_threshold = fail_thresh;
}

int dist_hb_register(dist_hb_monitor_t *mon, uint32_t node_id, uint64_t now) {
    if (mon->num_nodes >= 32) return 0;
    mon->nodes[mon->num_nodes].node_id = node_id;
    mon->nodes[mon->num_nodes].last_heartbeat = now;
    mon->nodes[mon->num_nodes].status = HB_ALIVE;
    mon->nodes[mon->num_nodes].miss_count = 0;
    mon->nodes[mon->num_nodes].heartbeat_count = 0;
    mon->num_nodes = mon->num_nodes + 1;
    return 1;
}

int dist_hb_find(const dist_hb_monitor_t *mon, uint32_t node_id) {
    int i;
    for (i = 0; i < mon->num_nodes; i++) {
        if (mon->nodes[i].node_id == node_id) return i;
    }
    return -1;
}

void dist_hb_receive(dist_hb_monitor_t *mon, uint32_t node_id, uint64_t now) {
    int idx = dist_hb_find(mon, node_id);
    if (idx >= 0) {
        mon->nodes[idx].last_heartbeat = now;
        mon->nodes[idx].miss_count = 0;
        mon->nodes[idx].heartbeat_count = mon->nodes[idx].heartbeat_count + 1;
        mon->nodes[idx].status = HB_ALIVE;
    }
}

void dist_hb_check(dist_hb_monitor_t *mon, uint64_t now) {
    int i;
    for (i = 0; i < mon->num_nodes; i++) {
        if (mon->nodes[i].status == HB_FAILED) continue;
        if (now - mon->nodes[i].last_heartbeat > mon->timeout_ms) {
            mon->nodes[i].miss_count = mon->nodes[i].miss_count + 1;
            if (mon->nodes[i].miss_count >= mon->fail_threshold) {
                mon->nodes[i].status = HB_FAILED;
            } else if (mon->nodes[i].miss_count >= mon->suspect_threshold) {
                mon->nodes[i].status = HB_SUSPECTED;
            }
        }
    }
}

int dist_hb_alive_count(const dist_hb_monitor_t *mon) {
    int count = 0;
    int i;
    for (i = 0; i < mon->num_nodes; i++) {
        if (mon->nodes[i].status == HB_ALIVE) count = count + 1;
    }
    return count;
}

int dist_hb_failed_count(const dist_hb_monitor_t *mon) {
    int count = 0;
    int i;
    for (i = 0; i < mon->num_nodes; i++) {
        if (mon->nodes[i].status == HB_FAILED) count = count + 1;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1517: Heartbeat monitor should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1517: Output should not be empty");
    assert!(code.contains("fn dist_hb_check"), "C1517: Should contain dist_hb_check");
    assert!(code.contains("fn dist_hb_receive"), "C1517: Should contain dist_hb_receive");
}

/// C1518: SWIM protocol with suspicion-based membership and protocol period
#[test]
fn c1518_swim_protocol() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

typedef enum {
    SWIM_ALIVE = 0,
    SWIM_SUSPECT = 1,
    SWIM_DEAD = 2
} dist_swim_state_t;

typedef struct {
    uint32_t node_id;
    dist_swim_state_t state;
    uint64_t incarnation;
    uint64_t last_update;
} dist_swim_member_t;

typedef struct {
    dist_swim_member_t members[64];
    int num_members;
    uint32_t self_id;
    uint64_t self_incarnation;
    int protocol_period_ms;
    int suspect_timeout_ms;
    int probe_index;
} dist_swim_t;

void dist_swim_init(dist_swim_t *swim, uint32_t self_id, int period_ms, int suspect_ms) {
    swim->num_members = 0;
    swim->self_id = self_id;
    swim->self_incarnation = 1;
    swim->protocol_period_ms = period_ms;
    swim->suspect_timeout_ms = suspect_ms;
    swim->probe_index = 0;
}

int dist_swim_add_member(dist_swim_t *swim, uint32_t id, uint64_t now) {
    if (swim->num_members >= 64) return 0;
    swim->members[swim->num_members].node_id = id;
    swim->members[swim->num_members].state = SWIM_ALIVE;
    swim->members[swim->num_members].incarnation = 0;
    swim->members[swim->num_members].last_update = now;
    swim->num_members = swim->num_members + 1;
    return 1;
}

int dist_swim_find(const dist_swim_t *swim, uint32_t id) {
    int i;
    for (i = 0; i < swim->num_members; i++) {
        if (swim->members[i].node_id == id) return i;
    }
    return -1;
}

void dist_swim_mark_suspect(dist_swim_t *swim, uint32_t id, uint64_t now) {
    int idx = dist_swim_find(swim, id);
    if (idx >= 0 && swim->members[idx].state == SWIM_ALIVE) {
        swim->members[idx].state = SWIM_SUSPECT;
        swim->members[idx].last_update = now;
    }
}

void dist_swim_mark_alive(dist_swim_t *swim, uint32_t id, uint64_t incarnation, uint64_t now) {
    int idx = dist_swim_find(swim, id);
    if (idx >= 0) {
        if (incarnation > swim->members[idx].incarnation) {
            swim->members[idx].state = SWIM_ALIVE;
            swim->members[idx].incarnation = incarnation;
            swim->members[idx].last_update = now;
        }
    }
}

void dist_swim_mark_dead(dist_swim_t *swim, uint32_t id, uint64_t now) {
    int idx = dist_swim_find(swim, id);
    if (idx >= 0) {
        swim->members[idx].state = SWIM_DEAD;
        swim->members[idx].last_update = now;
    }
}

void dist_swim_check_suspects(dist_swim_t *swim, uint64_t now) {
    int i;
    for (i = 0; i < swim->num_members; i++) {
        if (swim->members[i].state == SWIM_SUSPECT) {
            if (now - swim->members[i].last_update > (uint64_t)swim->suspect_timeout_ms) {
                swim->members[i].state = SWIM_DEAD;
                swim->members[i].last_update = now;
            }
        }
    }
}

uint32_t dist_swim_next_probe_target(dist_swim_t *swim) {
    int start = swim->probe_index;
    int i;
    for (i = 0; i < swim->num_members; i++) {
        int idx = (start + i) % swim->num_members;
        if (swim->members[idx].state != SWIM_DEAD &&
            swim->members[idx].node_id != swim->self_id) {
            swim->probe_index = (idx + 1) % swim->num_members;
            return swim->members[idx].node_id;
        }
    }
    return swim->self_id;
}

int dist_swim_alive_count(const dist_swim_t *swim) {
    int count = 0;
    int i;
    for (i = 0; i < swim->num_members; i++) {
        if (swim->members[i].state == SWIM_ALIVE) count = count + 1;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1518: SWIM protocol should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1518: Output should not be empty");
    assert!(code.contains("fn dist_swim_init"), "C1518: Should contain dist_swim_init");
    assert!(code.contains("fn dist_swim_check_suspects"), "C1518: Should contain dist_swim_check_suspects");
}

/// C1519: Write-ahead log (WAL) for crash recovery with checkpointing
#[test]
fn c1519_wal_failure_recovery() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

typedef struct {
    uint64_t lsn;
    uint32_t txn_id;
    int op;
    uint32_t key;
    int old_value;
    int new_value;
} dist_wal_record_t;

typedef struct {
    dist_wal_record_t records[512];
    int count;
    uint64_t next_lsn;
    uint64_t checkpoint_lsn;
    int data[64];
} dist_wal_t;

void dist_wal_init(dist_wal_t *wal) {
    int i;
    wal->count = 0;
    wal->next_lsn = 1;
    wal->checkpoint_lsn = 0;
    for (i = 0; i < 64; i++) wal->data[i] = 0;
}

int dist_wal_append(dist_wal_t *wal, uint32_t txn_id, int op,
                     uint32_t key, int old_val, int new_val) {
    if (wal->count >= 512) return 0;
    wal->records[wal->count].lsn = wal->next_lsn;
    wal->records[wal->count].txn_id = txn_id;
    wal->records[wal->count].op = op;
    wal->records[wal->count].key = key;
    wal->records[wal->count].old_value = old_val;
    wal->records[wal->count].new_value = new_val;
    wal->count = wal->count + 1;
    wal->next_lsn = wal->next_lsn + 1;
    return 1;
}

int dist_wal_set(dist_wal_t *wal, uint32_t txn_id, uint32_t key, int value) {
    int old_val = 0;
    if (key < 64) old_val = wal->data[key];
    if (!dist_wal_append(wal, txn_id, 1, key, old_val, value)) return 0;
    if (key < 64) wal->data[key] = value;
    return 1;
}

int dist_wal_undo(dist_wal_t *wal, uint32_t txn_id) {
    int i;
    int undone = 0;
    for (i = wal->count - 1; i >= 0; i--) {
        if (wal->records[i].txn_id == txn_id && wal->records[i].op == 1) {
            uint32_t key = wal->records[i].key;
            if (key < 64) {
                wal->data[key] = wal->records[i].old_value;
            }
            undone = undone + 1;
        }
    }
    return undone;
}

int dist_wal_redo_from(dist_wal_t *wal, uint64_t from_lsn) {
    int i;
    int redone = 0;
    for (i = 0; i < wal->count; i++) {
        if (wal->records[i].lsn >= from_lsn && wal->records[i].op == 1) {
            uint32_t key = wal->records[i].key;
            if (key < 64) {
                wal->data[key] = wal->records[i].new_value;
            }
            redone = redone + 1;
        }
    }
    return redone;
}

void dist_wal_checkpoint(dist_wal_t *wal) {
    wal->checkpoint_lsn = wal->next_lsn - 1;
    dist_wal_append(wal, 0, 3, 0, 0, 0);
}

int dist_wal_recover(dist_wal_t *wal) {
    return dist_wal_redo_from(wal, wal->checkpoint_lsn);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1519: WAL failure recovery should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1519: Output should not be empty");
    assert!(code.contains("fn dist_wal_init"), "C1519: Should contain dist_wal_init");
    assert!(code.contains("fn dist_wal_undo"), "C1519: Should contain dist_wal_undo");
    assert!(code.contains("fn dist_wal_recover"), "C1519: Should contain dist_wal_recover");
}

/// C1520: Circuit breaker pattern for cascading failure prevention
#[test]
fn c1520_circuit_breaker() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

typedef enum {
    CB_CLOSED = 0,
    CB_OPEN = 1,
    CB_HALF_OPEN = 2
} dist_cb_state_t;

typedef struct {
    dist_cb_state_t state;
    int failure_count;
    int success_count;
    int failure_threshold;
    int success_threshold;
    uint64_t last_failure_time;
    uint64_t cooldown_ms;
    uint64_t total_requests;
    uint64_t total_failures;
    uint64_t total_rejections;
} dist_circuit_breaker_t;

void dist_cb_init(dist_circuit_breaker_t *cb, int fail_thresh, int success_thresh,
                   uint64_t cooldown) {
    cb->state = CB_CLOSED;
    cb->failure_count = 0;
    cb->success_count = 0;
    cb->failure_threshold = fail_thresh;
    cb->success_threshold = success_thresh;
    cb->last_failure_time = 0;
    cb->cooldown_ms = cooldown;
    cb->total_requests = 0;
    cb->total_failures = 0;
    cb->total_rejections = 0;
}

int dist_cb_allow_request(dist_circuit_breaker_t *cb, uint64_t now) {
    if (cb->state == CB_CLOSED) return 1;
    if (cb->state == CB_OPEN) {
        if (now - cb->last_failure_time >= cb->cooldown_ms) {
            cb->state = CB_HALF_OPEN;
            cb->success_count = 0;
            return 1;
        }
        cb->total_rejections = cb->total_rejections + 1;
        return 0;
    }
    return 1;
}

void dist_cb_on_success(dist_circuit_breaker_t *cb) {
    cb->total_requests = cb->total_requests + 1;
    if (cb->state == CB_HALF_OPEN) {
        cb->success_count = cb->success_count + 1;
        if (cb->success_count >= cb->success_threshold) {
            cb->state = CB_CLOSED;
            cb->failure_count = 0;
        }
    } else {
        cb->failure_count = 0;
    }
}

void dist_cb_on_failure(dist_circuit_breaker_t *cb, uint64_t now) {
    cb->total_requests = cb->total_requests + 1;
    cb->total_failures = cb->total_failures + 1;
    cb->failure_count = cb->failure_count + 1;
    cb->last_failure_time = now;
    if (cb->state == CB_HALF_OPEN) {
        cb->state = CB_OPEN;
    } else if (cb->failure_count >= cb->failure_threshold) {
        cb->state = CB_OPEN;
    }
}

int dist_cb_get_state(const dist_circuit_breaker_t *cb) {
    return (int)cb->state;
}

uint64_t dist_cb_failure_rate(const dist_circuit_breaker_t *cb) {
    if (cb->total_requests == 0) return 0;
    return (cb->total_failures * 100) / cb->total_requests;
}

void dist_cb_reset(dist_circuit_breaker_t *cb) {
    cb->state = CB_CLOSED;
    cb->failure_count = 0;
    cb->success_count = 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1520: Circuit breaker should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1520: Output should not be empty");
    assert!(code.contains("fn dist_cb_init"), "C1520: Should contain dist_cb_init");
    assert!(code.contains("fn dist_cb_allow_request"), "C1520: Should contain dist_cb_allow_request");
    assert!(code.contains("fn dist_cb_on_failure"), "C1520: Should contain dist_cb_on_failure");
}

// ============================================================================
// C1521-C1525: Scheduling Algorithms
// ============================================================================

/// C1521: Work-stealing deque (Chase-Lev) for parallel task scheduling
#[test]
fn c1521_work_stealing_deque() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

typedef struct {
    int tasks[256];
    int top;
    int bottom;
    int capacity;
} dist_ws_deque_t;

void dist_ws_init(dist_ws_deque_t *dq) {
    int i;
    dq->top = 0;
    dq->bottom = 0;
    dq->capacity = 256;
    for (i = 0; i < 256; i++) dq->tasks[i] = 0;
}

int dist_ws_size(const dist_ws_deque_t *dq) {
    int s = dq->bottom - dq->top;
    if (s < 0) s = 0;
    return s;
}

int dist_ws_is_empty(const dist_ws_deque_t *dq) {
    return dq->bottom <= dq->top;
}

int dist_ws_push_bottom(dist_ws_deque_t *dq, int task) {
    if (dq->bottom >= dq->capacity) return 0;
    dq->tasks[dq->bottom % dq->capacity] = task;
    dq->bottom = dq->bottom + 1;
    return 1;
}

int dist_ws_pop_bottom(dist_ws_deque_t *dq, int *out_task) {
    int b, t;
    if (dist_ws_is_empty(dq)) return 0;
    dq->bottom = dq->bottom - 1;
    b = dq->bottom;
    t = dq->top;
    if (b > t) {
        *out_task = dq->tasks[b % dq->capacity];
        return 1;
    }
    if (b == t) {
        *out_task = dq->tasks[b % dq->capacity];
        dq->top = t + 1;
        dq->bottom = t + 1;
        return 1;
    }
    dq->bottom = t;
    return 0;
}

int dist_ws_steal(dist_ws_deque_t *dq, int *out_task) {
    int t = dq->top;
    int b = dq->bottom;
    if (t >= b) return 0;
    *out_task = dq->tasks[t % dq->capacity];
    dq->top = t + 1;
    return 1;
}

typedef struct {
    dist_ws_deque_t workers[8];
    int num_workers;
} dist_ws_pool_t;

void dist_ws_pool_init(dist_ws_pool_t *pool, int num_workers) {
    int i;
    pool->num_workers = num_workers;
    for (i = 0; i < num_workers && i < 8; i++) {
        dist_ws_init(&pool->workers[i]);
    }
}

int dist_ws_pool_submit(dist_ws_pool_t *pool, int worker_id, int task) {
    if (worker_id < 0 || worker_id >= pool->num_workers) return 0;
    return dist_ws_push_bottom(&pool->workers[worker_id], task);
}

int dist_ws_pool_steal_from(dist_ws_pool_t *pool, int thief, int victim, int *out_task) {
    if (victim < 0 || victim >= pool->num_workers) return 0;
    if (thief == victim) return 0;
    return dist_ws_steal(&pool->workers[victim], out_task);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1521: Work stealing deque should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1521: Output should not be empty");
    assert!(code.contains("fn dist_ws_push_bottom"), "C1521: Should contain dist_ws_push_bottom");
    assert!(code.contains("fn dist_ws_steal"), "C1521: Should contain dist_ws_steal");
}

/// C1522: Task DAG scheduler with topological ordering and dependency resolution
#[test]
fn c1522_task_dag_scheduler() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t task_id;
    int deps[8];
    int num_deps;
    int remaining_deps;
    int completed;
    int priority;
    int execution_time;
} dist_dag_task_t;

typedef struct {
    dist_dag_task_t tasks[64];
    int num_tasks;
    int ready_queue[64];
    int ready_count;
    int completed_count;
} dist_dag_scheduler_t;

void dist_dag_init(dist_dag_scheduler_t *sched) {
    sched->num_tasks = 0;
    sched->ready_count = 0;
    sched->completed_count = 0;
}

int dist_dag_add_task(dist_dag_scheduler_t *sched, uint32_t id, int priority, int exec_time) {
    int idx;
    if (sched->num_tasks >= 64) return -1;
    idx = sched->num_tasks;
    sched->tasks[idx].task_id = id;
    sched->tasks[idx].num_deps = 0;
    sched->tasks[idx].remaining_deps = 0;
    sched->tasks[idx].completed = 0;
    sched->tasks[idx].priority = priority;
    sched->tasks[idx].execution_time = exec_time;
    sched->num_tasks = sched->num_tasks + 1;
    return idx;
}

int dist_dag_find_task(const dist_dag_scheduler_t *sched, uint32_t id) {
    int i;
    for (i = 0; i < sched->num_tasks; i++) {
        if (sched->tasks[i].task_id == id) return i;
    }
    return -1;
}

int dist_dag_add_dep(dist_dag_scheduler_t *sched, uint32_t task_id, uint32_t dep_id) {
    int idx = dist_dag_find_task(sched, task_id);
    if (idx < 0) return 0;
    if (sched->tasks[idx].num_deps >= 8) return 0;
    sched->tasks[idx].deps[sched->tasks[idx].num_deps] = (int)dep_id;
    sched->tasks[idx].num_deps = sched->tasks[idx].num_deps + 1;
    sched->tasks[idx].remaining_deps = sched->tasks[idx].remaining_deps + 1;
    return 1;
}

void dist_dag_compute_ready(dist_dag_scheduler_t *sched) {
    int i;
    sched->ready_count = 0;
    for (i = 0; i < sched->num_tasks; i++) {
        if (!sched->tasks[i].completed && sched->tasks[i].remaining_deps == 0) {
            sched->ready_queue[sched->ready_count] = i;
            sched->ready_count = sched->ready_count + 1;
        }
    }
}

int dist_dag_pick_highest_priority(dist_dag_scheduler_t *sched) {
    int best = -1;
    int best_pri = -1;
    int i;
    for (i = 0; i < sched->ready_count; i++) {
        int idx = sched->ready_queue[i];
        if (sched->tasks[idx].priority > best_pri) {
            best_pri = sched->tasks[idx].priority;
            best = i;
        }
    }
    if (best >= 0) {
        return sched->ready_queue[best];
    }
    return -1;
}

void dist_dag_complete_task(dist_dag_scheduler_t *sched, uint32_t id) {
    int idx = dist_dag_find_task(sched, id);
    int i, j;
    if (idx < 0) return;
    sched->tasks[idx].completed = 1;
    sched->completed_count = sched->completed_count + 1;
    for (i = 0; i < sched->num_tasks; i++) {
        for (j = 0; j < sched->tasks[i].num_deps; j++) {
            if ((uint32_t)sched->tasks[i].deps[j] == id) {
                sched->tasks[i].remaining_deps = sched->tasks[i].remaining_deps - 1;
            }
        }
    }
}

int dist_dag_all_complete(const dist_dag_scheduler_t *sched) {
    return sched->completed_count >= sched->num_tasks;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1522: Task DAG scheduler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1522: Output should not be empty");
    assert!(code.contains("fn dist_dag_add_task"), "C1522: Should contain dist_dag_add_task");
    assert!(code.contains("fn dist_dag_complete_task"), "C1522: Should contain dist_dag_complete_task");
}

/// C1523: Fair share scheduler with per-user resource tracking and borrowing
#[test]
fn c1523_fair_share_scheduler() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

typedef struct {
    uint32_t user_id;
    int share_weight;
    uint64_t used_resources;
    uint64_t allocated;
    int active_tasks;
    int max_tasks;
} dist_fs_user_t;

typedef struct {
    uint32_t task_id;
    uint32_t user_id;
    int resource_need;
    int running;
    uint64_t start_time;
} dist_fs_task_t;

typedef struct {
    dist_fs_user_t users[16];
    int num_users;
    dist_fs_task_t tasks[128];
    int num_tasks;
    uint64_t total_resources;
    uint64_t allocated_resources;
} dist_fs_scheduler_t;

void dist_fs_init(dist_fs_scheduler_t *sched, uint64_t total_resources) {
    sched->num_users = 0;
    sched->num_tasks = 0;
    sched->total_resources = total_resources;
    sched->allocated_resources = 0;
}

int dist_fs_add_user(dist_fs_scheduler_t *sched, uint32_t user_id, int weight, int max_tasks) {
    if (sched->num_users >= 16) return 0;
    sched->users[sched->num_users].user_id = user_id;
    sched->users[sched->num_users].share_weight = weight;
    sched->users[sched->num_users].used_resources = 0;
    sched->users[sched->num_users].allocated = 0;
    sched->users[sched->num_users].active_tasks = 0;
    sched->users[sched->num_users].max_tasks = max_tasks;
    sched->num_users = sched->num_users + 1;
    return 1;
}

int dist_fs_find_user(const dist_fs_scheduler_t *sched, uint32_t user_id) {
    int i;
    for (i = 0; i < sched->num_users; i++) {
        if (sched->users[i].user_id == user_id) return i;
    }
    return -1;
}

uint64_t dist_fs_fair_share(const dist_fs_scheduler_t *sched, int user_idx) {
    int total_weight = 0;
    int i;
    for (i = 0; i < sched->num_users; i++) {
        total_weight = total_weight + sched->users[i].share_weight;
    }
    if (total_weight == 0) return 0;
    return (sched->total_resources * (uint64_t)sched->users[user_idx].share_weight)
           / (uint64_t)total_weight;
}

int dist_fs_submit_task(dist_fs_scheduler_t *sched, uint32_t task_id,
                         uint32_t user_id, int resource_need) {
    int uidx = dist_fs_find_user(sched, user_id);
    if (uidx < 0) return 0;
    if (sched->num_tasks >= 128) return 0;
    sched->tasks[sched->num_tasks].task_id = task_id;
    sched->tasks[sched->num_tasks].user_id = user_id;
    sched->tasks[sched->num_tasks].resource_need = resource_need;
    sched->tasks[sched->num_tasks].running = 0;
    sched->tasks[sched->num_tasks].start_time = 0;
    sched->num_tasks = sched->num_tasks + 1;
    return 1;
}

int dist_fs_schedule_next(dist_fs_scheduler_t *sched, uint64_t now) {
    int best_task = -1;
    uint64_t best_deficit = 0;
    int i;
    for (i = 0; i < sched->num_tasks; i++) {
        if (!sched->tasks[i].running) {
            int uidx = dist_fs_find_user(sched, sched->tasks[i].user_id);
            if (uidx >= 0 && sched->users[uidx].active_tasks < sched->users[uidx].max_tasks) {
                uint64_t share = dist_fs_fair_share(sched, uidx);
                uint64_t used = sched->users[uidx].used_resources;
                uint64_t deficit = share > used ? share - used : 0;
                if (best_task < 0 || deficit > best_deficit) {
                    best_deficit = deficit;
                    best_task = i;
                }
            }
        }
    }
    if (best_task >= 0) {
        int uidx = dist_fs_find_user(sched, sched->tasks[best_task].user_id);
        sched->tasks[best_task].running = 1;
        sched->tasks[best_task].start_time = now;
        if (uidx >= 0) {
            sched->users[uidx].active_tasks = sched->users[uidx].active_tasks + 1;
            sched->users[uidx].used_resources = sched->users[uidx].used_resources
                + (uint64_t)sched->tasks[best_task].resource_need;
        }
        sched->allocated_resources = sched->allocated_resources
            + (uint64_t)sched->tasks[best_task].resource_need;
    }
    return best_task;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1523: Fair share scheduler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1523: Output should not be empty");
    assert!(code.contains("fn dist_fs_init"), "C1523: Should contain dist_fs_init");
    assert!(code.contains("fn dist_fs_schedule_next"), "C1523: Should contain dist_fs_schedule_next");
}

/// C1524: Weighted round-robin load balancer with health checking
#[test]
fn c1524_load_balancer() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

typedef struct {
    uint32_t server_id;
    int weight;
    int current_weight;
    int active_connections;
    int max_connections;
    int healthy;
    uint64_t total_requests;
    uint64_t total_failures;
} dist_lb_server_t;

typedef struct {
    dist_lb_server_t servers[16];
    int num_servers;
    int current_index;
    uint64_t total_routed;
} dist_lb_t;

void dist_lb_init(dist_lb_t *lb) {
    lb->num_servers = 0;
    lb->current_index = 0;
    lb->total_routed = 0;
}

int dist_lb_add_server(dist_lb_t *lb, uint32_t id, int weight, int max_conn) {
    if (lb->num_servers >= 16) return 0;
    lb->servers[lb->num_servers].server_id = id;
    lb->servers[lb->num_servers].weight = weight;
    lb->servers[lb->num_servers].current_weight = 0;
    lb->servers[lb->num_servers].active_connections = 0;
    lb->servers[lb->num_servers].max_connections = max_conn;
    lb->servers[lb->num_servers].healthy = 1;
    lb->servers[lb->num_servers].total_requests = 0;
    lb->servers[lb->num_servers].total_failures = 0;
    lb->num_servers = lb->num_servers + 1;
    return 1;
}

int dist_lb_next_wrr(dist_lb_t *lb) {
    int total_weight = 0;
    int best = -1;
    int best_weight = -1;
    int i;
    for (i = 0; i < lb->num_servers; i++) {
        if (!lb->servers[i].healthy) continue;
        if (lb->servers[i].active_connections >= lb->servers[i].max_connections) continue;
        total_weight = total_weight + lb->servers[i].weight;
    }
    if (total_weight == 0) return -1;
    for (i = 0; i < lb->num_servers; i++) {
        if (!lb->servers[i].healthy) continue;
        if (lb->servers[i].active_connections >= lb->servers[i].max_connections) continue;
        lb->servers[i].current_weight = lb->servers[i].current_weight + lb->servers[i].weight;
        if (lb->servers[i].current_weight > best_weight) {
            best_weight = lb->servers[i].current_weight;
            best = i;
        }
    }
    if (best >= 0) {
        lb->servers[best].current_weight = lb->servers[best].current_weight - total_weight;
        lb->servers[best].active_connections = lb->servers[best].active_connections + 1;
        lb->servers[best].total_requests = lb->servers[best].total_requests + 1;
        lb->total_routed = lb->total_routed + 1;
    }
    if (best >= 0) {
        return (int)lb->servers[best].server_id;
    }
    return -1;
}

void dist_lb_release(dist_lb_t *lb, uint32_t server_id) {
    int i;
    for (i = 0; i < lb->num_servers; i++) {
        if (lb->servers[i].server_id == server_id) {
            if (lb->servers[i].active_connections > 0) {
                lb->servers[i].active_connections = lb->servers[i].active_connections - 1;
            }
            return;
        }
    }
}

void dist_lb_mark_unhealthy(dist_lb_t *lb, uint32_t server_id) {
    int i;
    for (i = 0; i < lb->num_servers; i++) {
        if (lb->servers[i].server_id == server_id) {
            lb->servers[i].healthy = 0;
            return;
        }
    }
}

void dist_lb_mark_healthy(dist_lb_t *lb, uint32_t server_id) {
    int i;
    for (i = 0; i < lb->num_servers; i++) {
        if (lb->servers[i].server_id == server_id) {
            lb->servers[i].healthy = 1;
            return;
        }
    }
}

int dist_lb_healthy_count(const dist_lb_t *lb) {
    int count = 0;
    int i;
    for (i = 0; i < lb->num_servers; i++) {
        if (lb->servers[i].healthy) count = count + 1;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1524: Load balancer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1524: Output should not be empty");
    assert!(code.contains("fn dist_lb_next_wrr"), "C1524: Should contain dist_lb_next_wrr");
    assert!(code.contains("fn dist_lb_add_server"), "C1524: Should contain dist_lb_add_server");
}

/// C1525: Token bucket rate limiter with burst capacity and refill
#[test]
fn c1525_rate_limiter_token_bucket() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

typedef struct {
    uint64_t tokens;
    uint64_t max_tokens;
    uint64_t refill_rate;
    uint64_t last_refill_time;
    uint64_t total_allowed;
    uint64_t total_denied;
} dist_token_bucket_t;

typedef struct {
    uint32_t client_id;
    dist_token_bucket_t bucket;
} dist_rl_client_t;

typedef struct {
    dist_rl_client_t clients[32];
    int num_clients;
    uint64_t default_rate;
    uint64_t default_burst;
} dist_rate_limiter_t;

void dist_tb_init(dist_token_bucket_t *tb, uint64_t max_tokens, uint64_t refill_rate) {
    tb->tokens = max_tokens;
    tb->max_tokens = max_tokens;
    tb->refill_rate = refill_rate;
    tb->last_refill_time = 0;
    tb->total_allowed = 0;
    tb->total_denied = 0;
}

void dist_tb_refill(dist_token_bucket_t *tb, uint64_t now) {
    uint64_t elapsed;
    uint64_t new_tokens;
    if (tb->last_refill_time == 0) {
        tb->last_refill_time = now;
        return;
    }
    elapsed = now - tb->last_refill_time;
    new_tokens = (elapsed * tb->refill_rate) / 1000;
    if (new_tokens > 0) {
        tb->tokens = tb->tokens + new_tokens;
        if (tb->tokens > tb->max_tokens) {
            tb->tokens = tb->max_tokens;
        }
        tb->last_refill_time = now;
    }
}

int dist_tb_consume(dist_token_bucket_t *tb, uint64_t tokens, uint64_t now) {
    dist_tb_refill(tb, now);
    if (tb->tokens >= tokens) {
        tb->tokens = tb->tokens - tokens;
        tb->total_allowed = tb->total_allowed + 1;
        return 1;
    }
    tb->total_denied = tb->total_denied + 1;
    return 0;
}

int dist_tb_try_consume(dist_token_bucket_t *tb, uint64_t now) {
    return dist_tb_consume(tb, 1, now);
}

void dist_rl_init(dist_rate_limiter_t *rl, uint64_t default_rate, uint64_t default_burst) {
    rl->num_clients = 0;
    rl->default_rate = default_rate;
    rl->default_burst = default_burst;
}

int dist_rl_find_client(const dist_rate_limiter_t *rl, uint32_t client_id) {
    int i;
    for (i = 0; i < rl->num_clients; i++) {
        if (rl->clients[i].client_id == client_id) return i;
    }
    return -1;
}

int dist_rl_register_client(dist_rate_limiter_t *rl, uint32_t client_id) {
    if (rl->num_clients >= 32) return 0;
    rl->clients[rl->num_clients].client_id = client_id;
    dist_tb_init(&rl->clients[rl->num_clients].bucket, rl->default_burst, rl->default_rate);
    rl->num_clients = rl->num_clients + 1;
    return 1;
}

int dist_rl_allow(dist_rate_limiter_t *rl, uint32_t client_id, uint64_t now) {
    int idx = dist_rl_find_client(rl, client_id);
    if (idx < 0) {
        if (!dist_rl_register_client(rl, client_id)) return 0;
        idx = rl->num_clients - 1;
    }
    return dist_tb_try_consume(&rl->clients[idx].bucket, now);
}

uint64_t dist_rl_tokens_remaining(const dist_rate_limiter_t *rl, uint32_t client_id) {
    int idx = dist_rl_find_client(rl, client_id);
    if (idx < 0) return 0;
    return rl->clients[idx].bucket.tokens;
}

uint64_t dist_rl_total_denied(const dist_rate_limiter_t *rl) {
    uint64_t total = 0;
    int i;
    for (i = 0; i < rl->num_clients; i++) {
        total = total + rl->clients[i].bucket.total_denied;
    }
    return total;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1525: Rate limiter token bucket should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1525: Output should not be empty");
    assert!(code.contains("fn dist_tb_consume"), "C1525: Should contain dist_tb_consume");
    assert!(code.contains("fn dist_rl_allow"), "C1525: Should contain dist_rl_allow");
    assert!(code.contains("fn dist_rl_init"), "C1525: Should contain dist_rl_init");
}
