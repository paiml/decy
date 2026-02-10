//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1851-C1875: Queue & Message Queue Implementations -- circular buffers,
//! priority queues, message queues, batch operations, and queue applications.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world queue patterns commonly found in operating
//! systems, message brokers, event systems, and task schedulers -- all expressed
//! as valid C99 without #include directives.
//!
//! Organization:
//! - C1851-C1855: Basic queues (circular buffer, double-ended, bounded, multi-queue, ring buffer)
//! - C1856-C1860: Priority queues (min-priority, max-priority, stable priority, indexed priority, multi-level)
//! - C1861-C1865: Message queues (producer-consumer, broadcast, filtered, delayed, reliable)
//! - C1866-C1870: Queue operations (batch enqueue, drain, peek many, queue merging, queue splitting)
//! - C1871-C1875: Queue applications (task queue, event queue, command queue, undo queue, log queue)

// ============================================================================
// C1851-C1855: Basic Queues
// ============================================================================

/// C1851: Circular buffer queue with wrap-around
#[test]
fn c1851_circular_buffer_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define QUE_CIRC_CAP 128

typedef struct {
    int data[QUE_CIRC_CAP];
    int head;
    int tail;
    int count;
} que_circ_t;

void que_circ_init(que_circ_t *q) {
    q->head = 0;
    q->tail = 0;
    q->count = 0;
    int i;
    for (i = 0; i < QUE_CIRC_CAP; i++) {
        q->data[i] = 0;
    }
}

int que_circ_enqueue(que_circ_t *q, int val) {
    if (q->count >= QUE_CIRC_CAP) return -1;
    q->data[q->tail] = val;
    q->tail = (q->tail + 1) % QUE_CIRC_CAP;
    q->count = q->count + 1;
    return 0;
}

int que_circ_dequeue(que_circ_t *q, int *out) {
    if (q->count <= 0) return -1;
    *out = q->data[q->head];
    q->head = (q->head + 1) % QUE_CIRC_CAP;
    q->count = q->count - 1;
    return 0;
}

int que_circ_peek(const que_circ_t *q) {
    if (q->count <= 0) return -1;
    return q->data[q->head];
}

int que_circ_is_full(const que_circ_t *q) {
    return q->count >= QUE_CIRC_CAP;
}

int que_circ_test(void) {
    que_circ_t q;
    que_circ_init(&q);
    que_circ_enqueue(&q, 10);
    que_circ_enqueue(&q, 20);
    que_circ_enqueue(&q, 30);
    if (que_circ_peek(&q) != 10) return -1;
    int val = 0;
    que_circ_dequeue(&q, &val);
    if (val != 10) return -2;
    if (q.count != 2) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1851: Circular buffer queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1851: Output should not be empty");
    assert!(code.contains("fn que_circ_init"), "C1851: Should contain que_circ_init");
    assert!(code.contains("fn que_circ_enqueue"), "C1851: Should contain que_circ_enqueue");
    assert!(code.contains("fn que_circ_dequeue"), "C1851: Should contain que_circ_dequeue");
    Ok(())
}

/// C1852: Double-ended queue (deque) with push/pop front and back
#[test]
fn c1852_double_ended_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

#define QUE_DEQUE_CAP 64

typedef struct {
    int data[QUE_DEQUE_CAP];
    int front;
    int back;
    int count;
} que_deque_t;

void que_deque_init(que_deque_t *dq) {
    dq->front = QUE_DEQUE_CAP / 2;
    dq->back = QUE_DEQUE_CAP / 2;
    dq->count = 0;
}

int que_deque_push_back(que_deque_t *dq, int val) {
    if (dq->count >= QUE_DEQUE_CAP) return -1;
    dq->data[dq->back] = val;
    dq->back = (dq->back + 1) % QUE_DEQUE_CAP;
    dq->count = dq->count + 1;
    return 0;
}

int que_deque_push_front(que_deque_t *dq, int val) {
    if (dq->count >= QUE_DEQUE_CAP) return -1;
    dq->front = (dq->front - 1 + QUE_DEQUE_CAP) % QUE_DEQUE_CAP;
    dq->data[dq->front] = val;
    dq->count = dq->count + 1;
    return 0;
}

int que_deque_pop_front(que_deque_t *dq, int *out) {
    if (dq->count <= 0) return -1;
    *out = dq->data[dq->front];
    dq->front = (dq->front + 1) % QUE_DEQUE_CAP;
    dq->count = dq->count - 1;
    return 0;
}

int que_deque_pop_back(que_deque_t *dq, int *out) {
    if (dq->count <= 0) return -1;
    dq->back = (dq->back - 1 + QUE_DEQUE_CAP) % QUE_DEQUE_CAP;
    *out = dq->data[dq->back];
    dq->count = dq->count - 1;
    return 0;
}

int que_deque_test(void) {
    que_deque_t dq;
    que_deque_init(&dq);
    que_deque_push_back(&dq, 10);
    que_deque_push_front(&dq, 5);
    que_deque_push_back(&dq, 20);
    int val = 0;
    que_deque_pop_front(&dq, &val);
    if (val != 5) return -1;
    que_deque_pop_back(&dq, &val);
    if (val != 20) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1852: Double-ended queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1852: Output should not be empty");
    assert!(code.contains("fn que_deque_init"), "C1852: Should contain que_deque_init");
    assert!(code.contains("fn que_deque_push_back"), "C1852: Should contain que_deque_push_back");
    assert!(code.contains("fn que_deque_pop_front"), "C1852: Should contain que_deque_pop_front");
    Ok(())
}

/// C1853: Bounded queue with high/low watermark flow control
#[test]
fn c1853_bounded_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

#define QUE_BOUND_CAP 256

typedef struct {
    int data[QUE_BOUND_CAP];
    int head;
    int tail;
    int count;
    int high_watermark;
    int low_watermark;
    int flow_blocked;
} que_bounded_t;

void que_bounded_init(que_bounded_t *q, int high, int low) {
    q->head = 0;
    q->tail = 0;
    q->count = 0;
    q->high_watermark = high;
    q->low_watermark = low;
    q->flow_blocked = 0;
}

int que_bounded_enqueue(que_bounded_t *q, int val) {
    if (q->count >= QUE_BOUND_CAP) return -1;
    if (q->flow_blocked) return -2;
    q->data[q->tail] = val;
    q->tail = (q->tail + 1) % QUE_BOUND_CAP;
    q->count = q->count + 1;
    if (q->count >= q->high_watermark) {
        q->flow_blocked = 1;
    }
    return 0;
}

int que_bounded_dequeue(que_bounded_t *q, int *out) {
    if (q->count <= 0) return -1;
    *out = q->data[q->head];
    q->head = (q->head + 1) % QUE_BOUND_CAP;
    q->count = q->count - 1;
    if (q->flow_blocked && q->count <= q->low_watermark) {
        q->flow_blocked = 0;
    }
    return 0;
}

int que_bounded_is_blocked(const que_bounded_t *q) {
    return q->flow_blocked;
}

int que_bounded_test(void) {
    que_bounded_t q;
    que_bounded_init(&q, 5, 2);
    int i;
    for (i = 0; i < 5; i++) {
        que_bounded_enqueue(&q, i * 10);
    }
    if (!que_bounded_is_blocked(&q)) return -1;
    int val = 0;
    for (i = 0; i < 3; i++) {
        que_bounded_dequeue(&q, &val);
    }
    if (que_bounded_is_blocked(&q)) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1853: Bounded queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1853: Output should not be empty");
    assert!(code.contains("fn que_bounded_init"), "C1853: Should contain que_bounded_init");
    assert!(code.contains("fn que_bounded_enqueue"), "C1853: Should contain que_bounded_enqueue");
    assert!(code.contains("fn que_bounded_dequeue"), "C1853: Should contain que_bounded_dequeue");
    Ok(())
}

/// C1854: Multi-queue with round-robin dispatch across lanes
#[test]
fn c1854_multi_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

#define QUE_MULTI_LANES 4
#define QUE_MULTI_LANE_CAP 64

typedef struct {
    int data[QUE_MULTI_LANE_CAP];
    int head;
    int tail;
    int count;
} que_lane_t;

typedef struct {
    que_lane_t lanes[QUE_MULTI_LANES];
    int num_lanes;
    int dispatch_idx;
    int total_items;
} que_multi_t;

void que_multi_init(que_multi_t *mq) {
    int i;
    mq->num_lanes = QUE_MULTI_LANES;
    mq->dispatch_idx = 0;
    mq->total_items = 0;
    for (i = 0; i < QUE_MULTI_LANES; i++) {
        mq->lanes[i].head = 0;
        mq->lanes[i].tail = 0;
        mq->lanes[i].count = 0;
    }
}

int que_multi_enqueue(que_multi_t *mq, int val) {
    int lane = mq->dispatch_idx;
    que_lane_t *l = &mq->lanes[lane];
    if (l->count >= QUE_MULTI_LANE_CAP) return -1;
    l->data[l->tail] = val;
    l->tail = (l->tail + 1) % QUE_MULTI_LANE_CAP;
    l->count = l->count + 1;
    mq->total_items = mq->total_items + 1;
    mq->dispatch_idx = (mq->dispatch_idx + 1) % mq->num_lanes;
    return lane;
}

int que_multi_dequeue(que_multi_t *mq, int lane, int *out) {
    if (lane < 0 || lane >= mq->num_lanes) return -1;
    que_lane_t *l = &mq->lanes[lane];
    if (l->count <= 0) return -2;
    *out = l->data[l->head];
    l->head = (l->head + 1) % QUE_MULTI_LANE_CAP;
    l->count = l->count - 1;
    mq->total_items = mq->total_items - 1;
    return 0;
}

int que_multi_lane_size(const que_multi_t *mq, int lane) {
    if (lane < 0 || lane >= mq->num_lanes) return -1;
    return mq->lanes[lane].count;
}

int que_multi_test(void) {
    que_multi_t mq;
    que_multi_init(&mq);
    que_multi_enqueue(&mq, 100);
    que_multi_enqueue(&mq, 200);
    que_multi_enqueue(&mq, 300);
    que_multi_enqueue(&mq, 400);
    if (que_multi_lane_size(&mq, 0) != 1) return -1;
    if (que_multi_lane_size(&mq, 1) != 1) return -2;
    if (mq.total_items != 4) return -3;
    int val = 0;
    que_multi_dequeue(&mq, 0, &val);
    if (val != 100) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1854: Multi-queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1854: Output should not be empty");
    assert!(code.contains("fn que_multi_init"), "C1854: Should contain que_multi_init");
    assert!(code.contains("fn que_multi_enqueue"), "C1854: Should contain que_multi_enqueue");
    assert!(code.contains("fn que_multi_dequeue"), "C1854: Should contain que_multi_dequeue");
    Ok(())
}

/// C1855: Ring buffer queue with overwrite-on-full semantics
#[test]
fn c1855_ring_buffer_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

#define QUE_RING_CAP 32

typedef struct {
    int data[QUE_RING_CAP];
    int write_pos;
    int read_pos;
    int count;
    int overwrite_count;
} que_ring_t;

void que_ring_init(que_ring_t *r) {
    r->write_pos = 0;
    r->read_pos = 0;
    r->count = 0;
    r->overwrite_count = 0;
    int i;
    for (i = 0; i < QUE_RING_CAP; i++) {
        r->data[i] = 0;
    }
}

void que_ring_write(que_ring_t *r, int val) {
    if (r->count >= QUE_RING_CAP) {
        r->read_pos = (r->read_pos + 1) % QUE_RING_CAP;
        r->overwrite_count = r->overwrite_count + 1;
    } else {
        r->count = r->count + 1;
    }
    r->data[r->write_pos] = val;
    r->write_pos = (r->write_pos + 1) % QUE_RING_CAP;
}

int que_ring_read(que_ring_t *r, int *out) {
    if (r->count <= 0) return -1;
    *out = r->data[r->read_pos];
    r->read_pos = (r->read_pos + 1) % QUE_RING_CAP;
    r->count = r->count - 1;
    return 0;
}

int que_ring_available(const que_ring_t *r) {
    return r->count;
}

int que_ring_test(void) {
    que_ring_t r;
    que_ring_init(&r);
    int i;
    for (i = 0; i < 40; i++) {
        que_ring_write(&r, i);
    }
    if (que_ring_available(&r) != QUE_RING_CAP) return -1;
    if (r.overwrite_count != 8) return -2;
    int val = 0;
    que_ring_read(&r, &val);
    if (val != 8) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1855: Ring buffer queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1855: Output should not be empty");
    assert!(code.contains("fn que_ring_init"), "C1855: Should contain que_ring_init");
    assert!(code.contains("fn que_ring_write"), "C1855: Should contain que_ring_write");
    assert!(code.contains("fn que_ring_read"), "C1855: Should contain que_ring_read");
    Ok(())
}

// ============================================================================
// C1856-C1860: Priority Queues
// ============================================================================

/// C1856: Min-priority queue using sorted insertion
#[test]
fn c1856_min_priority_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

#define QUE_MINPQ_CAP 128

typedef struct {
    int key;
    int value;
} que_minpq_entry_t;

typedef struct {
    que_minpq_entry_t entries[QUE_MINPQ_CAP];
    int count;
} que_minpq_t;

void que_minpq_init(que_minpq_t *pq) {
    pq->count = 0;
}

static void que_minpq_swap(que_minpq_entry_t *a, que_minpq_entry_t *b) {
    que_minpq_entry_t tmp;
    tmp.key = a->key;
    tmp.value = a->value;
    a->key = b->key;
    a->value = b->value;
    b->key = tmp.key;
    b->value = tmp.value;
}

static void que_minpq_sift_up(que_minpq_t *pq, int idx) {
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (pq->entries[parent].key > pq->entries[idx].key) {
            que_minpq_swap(&pq->entries[parent], &pq->entries[idx]);
            idx = parent;
        } else {
            break;
        }
    }
}

static void que_minpq_sift_down(que_minpq_t *pq, int idx) {
    while (1) {
        int smallest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < pq->count && pq->entries[left].key < pq->entries[smallest].key)
            smallest = left;
        if (right < pq->count && pq->entries[right].key < pq->entries[smallest].key)
            smallest = right;
        if (smallest != idx) {
            que_minpq_swap(&pq->entries[idx], &pq->entries[smallest]);
            idx = smallest;
        } else {
            break;
        }
    }
}

int que_minpq_insert(que_minpq_t *pq, int key, int value) {
    if (pq->count >= QUE_MINPQ_CAP) return -1;
    pq->entries[pq->count].key = key;
    pq->entries[pq->count].value = value;
    que_minpq_sift_up(pq, pq->count);
    pq->count = pq->count + 1;
    return 0;
}

int que_minpq_extract(que_minpq_t *pq, int *key_out, int *val_out) {
    if (pq->count <= 0) return -1;
    *key_out = pq->entries[0].key;
    *val_out = pq->entries[0].value;
    pq->count = pq->count - 1;
    pq->entries[0] = pq->entries[pq->count];
    que_minpq_sift_down(pq, 0);
    return 0;
}

int que_minpq_test(void) {
    que_minpq_t pq;
    que_minpq_init(&pq);
    que_minpq_insert(&pq, 30, 300);
    que_minpq_insert(&pq, 10, 100);
    que_minpq_insert(&pq, 20, 200);
    int k = 0, v = 0;
    que_minpq_extract(&pq, &k, &v);
    if (k != 10 || v != 100) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1856: Min-priority queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1856: Output should not be empty");
    assert!(code.contains("fn que_minpq_init"), "C1856: Should contain que_minpq_init");
    assert!(code.contains("fn que_minpq_insert"), "C1856: Should contain que_minpq_insert");
    assert!(code.contains("fn que_minpq_extract"), "C1856: Should contain que_minpq_extract");
    Ok(())
}

/// C1857: Max-priority queue with peek and size
#[test]
fn c1857_max_priority_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

#define QUE_MAXPQ_CAP 128

typedef struct {
    int priority;
    int data;
} que_maxpq_entry_t;

typedef struct {
    que_maxpq_entry_t heap[QUE_MAXPQ_CAP];
    int size;
} que_maxpq_t;

void que_maxpq_init(que_maxpq_t *pq) {
    pq->size = 0;
}

static void que_maxpq_swap_entries(que_maxpq_entry_t *a, que_maxpq_entry_t *b) {
    que_maxpq_entry_t tmp;
    tmp.priority = a->priority;
    tmp.data = a->data;
    a->priority = b->priority;
    a->data = b->data;
    b->priority = tmp.priority;
    b->data = tmp.data;
}

int que_maxpq_insert(que_maxpq_t *pq, int priority, int data) {
    if (pq->size >= QUE_MAXPQ_CAP) return -1;
    int idx = pq->size;
    pq->heap[idx].priority = priority;
    pq->heap[idx].data = data;
    pq->size = pq->size + 1;
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (pq->heap[parent].priority < pq->heap[idx].priority) {
            que_maxpq_swap_entries(&pq->heap[parent], &pq->heap[idx]);
            idx = parent;
        } else {
            break;
        }
    }
    return 0;
}

int que_maxpq_extract(que_maxpq_t *pq, int *out_data) {
    if (pq->size <= 0) return -1;
    *out_data = pq->heap[0].data;
    pq->size = pq->size - 1;
    pq->heap[0] = pq->heap[pq->size];
    int idx = 0;
    while (1) {
        int largest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < pq->size && pq->heap[left].priority > pq->heap[largest].priority)
            largest = left;
        if (right < pq->size && pq->heap[right].priority > pq->heap[largest].priority)
            largest = right;
        if (largest != idx) {
            que_maxpq_swap_entries(&pq->heap[idx], &pq->heap[largest]);
            idx = largest;
        } else {
            break;
        }
    }
    return 0;
}

int que_maxpq_peek(const que_maxpq_t *pq) {
    if (pq->size <= 0) return -1;
    return pq->heap[0].data;
}

int que_maxpq_test(void) {
    que_maxpq_t pq;
    que_maxpq_init(&pq);
    que_maxpq_insert(&pq, 5, 50);
    que_maxpq_insert(&pq, 9, 90);
    que_maxpq_insert(&pq, 3, 30);
    if (que_maxpq_peek(&pq) != 90) return -1;
    int val = 0;
    que_maxpq_extract(&pq, &val);
    if (val != 90) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1857: Max-priority queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1857: Output should not be empty");
    assert!(code.contains("fn que_maxpq_init"), "C1857: Should contain que_maxpq_init");
    assert!(code.contains("fn que_maxpq_insert"), "C1857: Should contain que_maxpq_insert");
    assert!(code.contains("fn que_maxpq_extract"), "C1857: Should contain que_maxpq_extract");
    Ok(())
}

/// C1858: Stable priority queue preserving insertion order at same priority
#[test]
fn c1858_stable_priority_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define QUE_STABLE_CAP 128

typedef struct {
    int priority;
    int data;
    uint32_t sequence;
} que_stable_entry_t;

typedef struct {
    que_stable_entry_t entries[QUE_STABLE_CAP];
    int count;
    uint32_t next_seq;
} que_stable_pq_t;

void que_stable_init(que_stable_pq_t *pq) {
    pq->count = 0;
    pq->next_seq = 0;
}

int que_stable_insert(que_stable_pq_t *pq, int priority, int data) {
    if (pq->count >= QUE_STABLE_CAP) return -1;
    int pos = pq->count;
    pq->entries[pos].priority = priority;
    pq->entries[pos].data = data;
    pq->entries[pos].sequence = pq->next_seq;
    pq->next_seq = pq->next_seq + 1;
    pq->count = pq->count + 1;
    while (pos > 0) {
        int parent = (pos - 1) / 2;
        int swap = 0;
        if (pq->entries[parent].priority > pq->entries[pos].priority) {
            swap = 1;
        } else if (pq->entries[parent].priority == pq->entries[pos].priority &&
                   pq->entries[parent].sequence > pq->entries[pos].sequence) {
            swap = 1;
        }
        if (swap) {
            que_stable_entry_t tmp = pq->entries[parent];
            pq->entries[parent] = pq->entries[pos];
            pq->entries[pos] = tmp;
            pos = parent;
        } else {
            break;
        }
    }
    return 0;
}

int que_stable_extract(que_stable_pq_t *pq, int *out) {
    if (pq->count <= 0) return -1;
    *out = pq->entries[0].data;
    pq->count = pq->count - 1;
    pq->entries[0] = pq->entries[pq->count];
    int idx = 0;
    while (1) {
        int best = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < pq->count) {
            if (pq->entries[left].priority < pq->entries[best].priority ||
                (pq->entries[left].priority == pq->entries[best].priority &&
                 pq->entries[left].sequence < pq->entries[best].sequence))
                best = left;
        }
        if (right < pq->count) {
            if (pq->entries[right].priority < pq->entries[best].priority ||
                (pq->entries[right].priority == pq->entries[best].priority &&
                 pq->entries[right].sequence < pq->entries[best].sequence))
                best = right;
        }
        if (best != idx) {
            que_stable_entry_t tmp = pq->entries[idx];
            pq->entries[idx] = pq->entries[best];
            pq->entries[best] = tmp;
            idx = best;
        } else {
            break;
        }
    }
    return 0;
}

int que_stable_test(void) {
    que_stable_pq_t pq;
    que_stable_init(&pq);
    que_stable_insert(&pq, 1, 10);
    que_stable_insert(&pq, 1, 20);
    que_stable_insert(&pq, 1, 30);
    int val = 0;
    que_stable_extract(&pq, &val);
    if (val != 10) return -1;
    que_stable_extract(&pq, &val);
    if (val != 20) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1858: Stable priority queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1858: Output should not be empty");
    assert!(code.contains("fn que_stable_init"), "C1858: Should contain que_stable_init");
    assert!(code.contains("fn que_stable_insert"), "C1858: Should contain que_stable_insert");
    assert!(code.contains("fn que_stable_extract"), "C1858: Should contain que_stable_extract");
    Ok(())
}

/// C1859: Indexed priority queue with decrease-key operation
#[test]
fn c1859_indexed_priority_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

#define QUE_IDXPQ_CAP 64

typedef struct {
    int keys[QUE_IDXPQ_CAP];
    int heap[QUE_IDXPQ_CAP];
    int pos[QUE_IDXPQ_CAP];
    int size;
    int max_id;
} que_idxpq_t;

void que_idxpq_init(que_idxpq_t *pq) {
    pq->size = 0;
    pq->max_id = QUE_IDXPQ_CAP;
    int i;
    for (i = 0; i < QUE_IDXPQ_CAP; i++) {
        pq->keys[i] = 0;
        pq->heap[i] = -1;
        pq->pos[i] = -1;
    }
}

static void que_idxpq_swap(que_idxpq_t *pq, int i, int j) {
    int hi = pq->heap[i];
    int hj = pq->heap[j];
    pq->heap[i] = hj;
    pq->heap[j] = hi;
    pq->pos[hj] = i;
    pq->pos[hi] = j;
}

static void que_idxpq_bubble_up(que_idxpq_t *pq, int idx) {
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (pq->keys[pq->heap[idx]] < pq->keys[pq->heap[parent]]) {
            que_idxpq_swap(pq, idx, parent);
            idx = parent;
        } else {
            break;
        }
    }
}

int que_idxpq_insert(que_idxpq_t *pq, int id, int key) {
    if (id < 0 || id >= pq->max_id) return -1;
    if (pq->size >= QUE_IDXPQ_CAP) return -2;
    pq->keys[id] = key;
    pq->heap[pq->size] = id;
    pq->pos[id] = pq->size;
    pq->size = pq->size + 1;
    que_idxpq_bubble_up(pq, pq->pos[id]);
    return 0;
}

int que_idxpq_decrease_key(que_idxpq_t *pq, int id, int new_key) {
    if (id < 0 || id >= pq->max_id) return -1;
    if (pq->pos[id] < 0) return -2;
    if (new_key >= pq->keys[id]) return -3;
    pq->keys[id] = new_key;
    que_idxpq_bubble_up(pq, pq->pos[id]);
    return 0;
}

int que_idxpq_peek_id(const que_idxpq_t *pq) {
    if (pq->size <= 0) return -1;
    return pq->heap[0];
}

int que_idxpq_test(void) {
    que_idxpq_t pq;
    que_idxpq_init(&pq);
    que_idxpq_insert(&pq, 0, 50);
    que_idxpq_insert(&pq, 1, 30);
    que_idxpq_insert(&pq, 2, 70);
    if (que_idxpq_peek_id(&pq) != 1) return -1;
    que_idxpq_decrease_key(&pq, 2, 10);
    if (que_idxpq_peek_id(&pq) != 2) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1859: Indexed priority queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1859: Output should not be empty");
    assert!(code.contains("fn que_idxpq_init"), "C1859: Should contain que_idxpq_init");
    assert!(code.contains("fn que_idxpq_insert"), "C1859: Should contain que_idxpq_insert");
    assert!(code.contains("fn que_idxpq_decrease_key"), "C1859: Should contain que_idxpq_decrease_key");
    Ok(())
}

/// C1860: Multi-level feedback queue with aging promotion
#[test]
fn c1860_multilevel_feedback_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

#define QUE_MLFQ_LEVELS 4
#define QUE_MLFQ_CAP 32

typedef struct {
    int task_id;
    int age;
    int cpu_used;
} que_mlfq_task_t;

typedef struct {
    que_mlfq_task_t tasks[QUE_MLFQ_CAP];
    int head;
    int tail;
    int count;
} que_mlfq_level_t;

typedef struct {
    que_mlfq_level_t levels[QUE_MLFQ_LEVELS];
    int age_threshold;
    int total_tasks;
} que_mlfq_t;

void que_mlfq_init(que_mlfq_t *mq, int age_thresh) {
    mq->age_threshold = age_thresh;
    mq->total_tasks = 0;
    int i;
    for (i = 0; i < QUE_MLFQ_LEVELS; i++) {
        mq->levels[i].head = 0;
        mq->levels[i].tail = 0;
        mq->levels[i].count = 0;
    }
}

int que_mlfq_add_task(que_mlfq_t *mq, int level, int task_id) {
    if (level < 0 || level >= QUE_MLFQ_LEVELS) return -1;
    que_mlfq_level_t *lv = &mq->levels[level];
    if (lv->count >= QUE_MLFQ_CAP) return -2;
    lv->tasks[lv->tail].task_id = task_id;
    lv->tasks[lv->tail].age = 0;
    lv->tasks[lv->tail].cpu_used = 0;
    lv->tail = (lv->tail + 1) % QUE_MLFQ_CAP;
    lv->count = lv->count + 1;
    mq->total_tasks = mq->total_tasks + 1;
    return 0;
}

int que_mlfq_promote_aged(que_mlfq_t *mq) {
    int promoted = 0;
    int level;
    for (level = 1; level < QUE_MLFQ_LEVELS; level++) {
        que_mlfq_level_t *lv = &mq->levels[level];
        int checked = 0;
        while (checked < lv->count) {
            int idx = (lv->head + checked) % QUE_MLFQ_CAP;
            lv->tasks[idx].age = lv->tasks[idx].age + 1;
            if (lv->tasks[idx].age >= mq->age_threshold) {
                promoted = promoted + 1;
            }
            checked = checked + 1;
        }
    }
    return promoted;
}

int que_mlfq_select_task(const que_mlfq_t *mq) {
    int level;
    for (level = 0; level < QUE_MLFQ_LEVELS; level++) {
        if (mq->levels[level].count > 0) {
            int head = mq->levels[level].head;
            return mq->levels[level].tasks[head].task_id;
        }
    }
    return -1;
}

int que_mlfq_test(void) {
    que_mlfq_t mq;
    que_mlfq_init(&mq, 5);
    que_mlfq_add_task(&mq, 0, 100);
    que_mlfq_add_task(&mq, 1, 200);
    que_mlfq_add_task(&mq, 2, 300);
    if (que_mlfq_select_task(&mq) != 100) return -1;
    if (mq.total_tasks != 3) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1860: Multi-level feedback queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1860: Output should not be empty");
    assert!(code.contains("fn que_mlfq_init"), "C1860: Should contain que_mlfq_init");
    assert!(code.contains("fn que_mlfq_add_task"), "C1860: Should contain que_mlfq_add_task");
    assert!(code.contains("fn que_mlfq_select_task"), "C1860: Should contain que_mlfq_select_task");
    Ok(())
}

// ============================================================================
// C1861-C1865: Message Queues
// ============================================================================

/// C1861: Producer-consumer message queue with typed messages
#[test]
fn c1861_producer_consumer_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define QUE_MSG_CAP 64
#define QUE_MSG_DATA_LEN 32

typedef struct {
    uint32_t msg_type;
    int sender_id;
    int payload[QUE_MSG_DATA_LEN];
    int payload_len;
} que_msg_t;

typedef struct {
    que_msg_t buffer[QUE_MSG_CAP];
    int head;
    int tail;
    int count;
    int producers_active;
    int consumers_active;
} que_prodcons_t;

void que_prodcons_init(que_prodcons_t *q) {
    q->head = 0;
    q->tail = 0;
    q->count = 0;
    q->producers_active = 0;
    q->consumers_active = 0;
}

int que_prodcons_send(que_prodcons_t *q, uint32_t msg_type, int sender, const int *data, int len) {
    if (q->count >= QUE_MSG_CAP) return -1;
    if (len > QUE_MSG_DATA_LEN) len = QUE_MSG_DATA_LEN;
    que_msg_t *m = &q->buffer[q->tail];
    m->msg_type = msg_type;
    m->sender_id = sender;
    m->payload_len = len;
    int i;
    for (i = 0; i < len; i++) {
        m->payload[i] = data[i];
    }
    q->tail = (q->tail + 1) % QUE_MSG_CAP;
    q->count = q->count + 1;
    return 0;
}

int que_prodcons_recv(que_prodcons_t *q, que_msg_t *out) {
    if (q->count <= 0) return -1;
    *out = q->buffer[q->head];
    q->head = (q->head + 1) % QUE_MSG_CAP;
    q->count = q->count - 1;
    return 0;
}

int que_prodcons_pending(const que_prodcons_t *q) {
    return q->count;
}

int que_prodcons_test(void) {
    que_prodcons_t q;
    que_prodcons_init(&q);
    int data[4];
    data[0] = 1; data[1] = 2; data[2] = 3; data[3] = 4;
    que_prodcons_send(&q, 1, 0, data, 4);
    que_prodcons_send(&q, 2, 1, data, 2);
    if (que_prodcons_pending(&q) != 2) return -1;
    que_msg_t msg;
    que_prodcons_recv(&q, &msg);
    if (msg.msg_type != 1) return -2;
    if (msg.sender_id != 0) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1861: Producer-consumer queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1861: Output should not be empty");
    assert!(code.contains("fn que_prodcons_init"), "C1861: Should contain que_prodcons_init");
    assert!(code.contains("fn que_prodcons_send"), "C1861: Should contain que_prodcons_send");
    assert!(code.contains("fn que_prodcons_recv"), "C1861: Should contain que_prodcons_recv");
    Ok(())
}

/// C1862: Broadcast queue with multiple subscriber cursors
#[test]
fn c1862_broadcast_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

#define QUE_BCAST_CAP 64
#define QUE_BCAST_MAX_SUBS 8

typedef struct {
    int data[QUE_BCAST_CAP];
    int write_pos;
    int sub_read_pos[QUE_BCAST_MAX_SUBS];
    int sub_active[QUE_BCAST_MAX_SUBS];
    int num_subs;
    int total_written;
} que_bcast_t;

void que_bcast_init(que_bcast_t *bq) {
    bq->write_pos = 0;
    bq->num_subs = 0;
    bq->total_written = 0;
    int i;
    for (i = 0; i < QUE_BCAST_MAX_SUBS; i++) {
        bq->sub_read_pos[i] = 0;
        bq->sub_active[i] = 0;
    }
}

int que_bcast_subscribe(que_bcast_t *bq) {
    if (bq->num_subs >= QUE_BCAST_MAX_SUBS) return -1;
    int id = bq->num_subs;
    bq->sub_active[id] = 1;
    bq->sub_read_pos[id] = bq->write_pos;
    bq->num_subs = bq->num_subs + 1;
    return id;
}

int que_bcast_publish(que_bcast_t *bq, int val) {
    bq->data[bq->write_pos % QUE_BCAST_CAP] = val;
    bq->write_pos = bq->write_pos + 1;
    bq->total_written = bq->total_written + 1;
    return 0;
}

int que_bcast_consume(que_bcast_t *bq, int sub_id, int *out) {
    if (sub_id < 0 || sub_id >= bq->num_subs) return -1;
    if (!bq->sub_active[sub_id]) return -2;
    if (bq->sub_read_pos[sub_id] >= bq->write_pos) return -3;
    *out = bq->data[bq->sub_read_pos[sub_id] % QUE_BCAST_CAP];
    bq->sub_read_pos[sub_id] = bq->sub_read_pos[sub_id] + 1;
    return 0;
}

int que_bcast_test(void) {
    que_bcast_t bq;
    que_bcast_init(&bq);
    int s0 = que_bcast_subscribe(&bq);
    int s1 = que_bcast_subscribe(&bq);
    que_bcast_publish(&bq, 42);
    que_bcast_publish(&bq, 99);
    int val = 0;
    que_bcast_consume(&bq, s0, &val);
    if (val != 42) return -1;
    que_bcast_consume(&bq, s1, &val);
    if (val != 42) return -2;
    if (s0 != 0 || s1 != 1) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1862: Broadcast queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1862: Output should not be empty");
    assert!(code.contains("fn que_bcast_init"), "C1862: Should contain que_bcast_init");
    assert!(code.contains("fn que_bcast_publish"), "C1862: Should contain que_bcast_publish");
    assert!(code.contains("fn que_bcast_consume"), "C1862: Should contain que_bcast_consume");
    Ok(())
}

/// C1863: Filtered queue that only dequeues messages matching a predicate type
#[test]
fn c1863_filtered_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define QUE_FILT_CAP 64

typedef struct {
    uint32_t msg_type;
    int value;
    int processed;
} que_filt_msg_t;

typedef struct {
    que_filt_msg_t msgs[QUE_FILT_CAP];
    int count;
} que_filt_t;

void que_filt_init(que_filt_t *q) {
    q->count = 0;
    int i;
    for (i = 0; i < QUE_FILT_CAP; i++) {
        q->msgs[i].msg_type = 0;
        q->msgs[i].value = 0;
        q->msgs[i].processed = 0;
    }
}

int que_filt_push(que_filt_t *q, uint32_t msg_type, int value) {
    if (q->count >= QUE_FILT_CAP) return -1;
    q->msgs[q->count].msg_type = msg_type;
    q->msgs[q->count].value = value;
    q->msgs[q->count].processed = 0;
    q->count = q->count + 1;
    return 0;
}

int que_filt_pop_by_type(que_filt_t *q, uint32_t target_type, int *out_value) {
    int i;
    for (i = 0; i < q->count; i++) {
        if (q->msgs[i].msg_type == target_type && !q->msgs[i].processed) {
            *out_value = q->msgs[i].value;
            q->msgs[i].processed = 1;
            return 0;
        }
    }
    return -1;
}

int que_filt_count_by_type(const que_filt_t *q, uint32_t target_type) {
    int cnt = 0;
    int i;
    for (i = 0; i < q->count; i++) {
        if (q->msgs[i].msg_type == target_type && !q->msgs[i].processed) {
            cnt = cnt + 1;
        }
    }
    return cnt;
}

int que_filt_test(void) {
    que_filt_t q;
    que_filt_init(&q);
    que_filt_push(&q, 1, 100);
    que_filt_push(&q, 2, 200);
    que_filt_push(&q, 1, 300);
    que_filt_push(&q, 3, 400);
    if (que_filt_count_by_type(&q, 1) != 2) return -1;
    int val = 0;
    que_filt_pop_by_type(&q, 2, &val);
    if (val != 200) return -2;
    if (que_filt_count_by_type(&q, 2) != 0) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1863: Filtered queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1863: Output should not be empty");
    assert!(code.contains("fn que_filt_init"), "C1863: Should contain que_filt_init");
    assert!(code.contains("fn que_filt_push"), "C1863: Should contain que_filt_push");
    assert!(code.contains("fn que_filt_pop_by_type"), "C1863: Should contain que_filt_pop_by_type");
    Ok(())
}

/// C1864: Delayed queue where messages become available after a delay tick
#[test]
fn c1864_delayed_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define QUE_DELAY_CAP 64

typedef struct {
    int value;
    uint32_t ready_at;
    int active;
} que_delay_entry_t;

typedef struct {
    que_delay_entry_t entries[QUE_DELAY_CAP];
    int count;
    uint32_t current_tick;
} que_delay_t;

void que_delay_init(que_delay_t *q) {
    q->count = 0;
    q->current_tick = 0;
    int i;
    for (i = 0; i < QUE_DELAY_CAP; i++) {
        q->entries[i].active = 0;
    }
}

int que_delay_enqueue(que_delay_t *q, int value, uint32_t delay_ticks) {
    if (q->count >= QUE_DELAY_CAP) return -1;
    int i;
    for (i = 0; i < QUE_DELAY_CAP; i++) {
        if (!q->entries[i].active) {
            q->entries[i].value = value;
            q->entries[i].ready_at = q->current_tick + delay_ticks;
            q->entries[i].active = 1;
            q->count = q->count + 1;
            return 0;
        }
    }
    return -2;
}

int que_delay_try_dequeue(que_delay_t *q, int *out) {
    int i;
    int best_idx = -1;
    uint32_t best_ready = 0;
    for (i = 0; i < QUE_DELAY_CAP; i++) {
        if (q->entries[i].active && q->entries[i].ready_at <= q->current_tick) {
            if (best_idx < 0 || q->entries[i].ready_at < best_ready) {
                best_idx = i;
                best_ready = q->entries[i].ready_at;
            }
        }
    }
    if (best_idx < 0) return -1;
    *out = q->entries[best_idx].value;
    q->entries[best_idx].active = 0;
    q->count = q->count - 1;
    return 0;
}

void que_delay_tick(que_delay_t *q) {
    q->current_tick = q->current_tick + 1;
}

int que_delay_ready_count(const que_delay_t *q) {
    int cnt = 0;
    int i;
    for (i = 0; i < QUE_DELAY_CAP; i++) {
        if (q->entries[i].active && q->entries[i].ready_at <= q->current_tick) {
            cnt = cnt + 1;
        }
    }
    return cnt;
}

int que_delay_test(void) {
    que_delay_t q;
    que_delay_init(&q);
    que_delay_enqueue(&q, 100, 3);
    que_delay_enqueue(&q, 200, 1);
    if (que_delay_ready_count(&q) != 0) return -1;
    que_delay_tick(&q);
    if (que_delay_ready_count(&q) != 1) return -2;
    int val = 0;
    que_delay_try_dequeue(&q, &val);
    if (val != 200) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1864: Delayed queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1864: Output should not be empty");
    assert!(code.contains("fn que_delay_init"), "C1864: Should contain que_delay_init");
    assert!(code.contains("fn que_delay_enqueue"), "C1864: Should contain que_delay_enqueue");
    assert!(code.contains("fn que_delay_try_dequeue"), "C1864: Should contain que_delay_try_dequeue");
    Ok(())
}

/// C1865: Reliable message queue with acknowledgment tracking
#[test]
fn c1865_reliable_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define QUE_REL_CAP 64
#define QUE_REL_PENDING 0
#define QUE_REL_DELIVERED 1
#define QUE_REL_ACKED 2

typedef struct {
    int value;
    uint32_t msg_id;
    int status;
    int retry_count;
} que_rel_msg_t;

typedef struct {
    que_rel_msg_t msgs[QUE_REL_CAP];
    int count;
    uint32_t next_id;
    int max_retries;
} que_rel_t;

void que_rel_init(que_rel_t *q, int max_retries) {
    q->count = 0;
    q->next_id = 1;
    q->max_retries = max_retries;
    int i;
    for (i = 0; i < QUE_REL_CAP; i++) {
        q->msgs[i].status = -1;
    }
}

uint32_t que_rel_send(que_rel_t *q, int value) {
    if (q->count >= QUE_REL_CAP) return 0;
    int i;
    for (i = 0; i < QUE_REL_CAP; i++) {
        if (q->msgs[i].status < 0) {
            q->msgs[i].value = value;
            q->msgs[i].msg_id = q->next_id;
            q->msgs[i].status = QUE_REL_PENDING;
            q->msgs[i].retry_count = 0;
            q->count = q->count + 1;
            q->next_id = q->next_id + 1;
            return q->msgs[i].msg_id;
        }
    }
    return 0;
}

int que_rel_deliver(que_rel_t *q, int *out_value, uint32_t *out_id) {
    int i;
    for (i = 0; i < QUE_REL_CAP; i++) {
        if (q->msgs[i].status == QUE_REL_PENDING) {
            *out_value = q->msgs[i].value;
            *out_id = q->msgs[i].msg_id;
            q->msgs[i].status = QUE_REL_DELIVERED;
            return 0;
        }
    }
    return -1;
}

int que_rel_ack(que_rel_t *q, uint32_t msg_id) {
    int i;
    for (i = 0; i < QUE_REL_CAP; i++) {
        if (q->msgs[i].msg_id == msg_id && q->msgs[i].status == QUE_REL_DELIVERED) {
            q->msgs[i].status = QUE_REL_ACKED;
            q->count = q->count - 1;
            return 0;
        }
    }
    return -1;
}

int que_rel_test(void) {
    que_rel_t q;
    que_rel_init(&q, 3);
    uint32_t id1 = que_rel_send(&q, 42);
    uint32_t id2 = que_rel_send(&q, 99);
    if (id1 != 1 || id2 != 2) return -1;
    int val = 0;
    uint32_t mid = 0;
    que_rel_deliver(&q, &val, &mid);
    if (val != 42 || mid != 1) return -2;
    que_rel_ack(&q, mid);
    if (q.count != 1) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1865: Reliable queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1865: Output should not be empty");
    assert!(code.contains("fn que_rel_init"), "C1865: Should contain que_rel_init");
    assert!(code.contains("fn que_rel_send"), "C1865: Should contain que_rel_send");
    assert!(code.contains("fn que_rel_ack"), "C1865: Should contain que_rel_ack");
    Ok(())
}

// ============================================================================
// C1866-C1870: Queue Operations
// ============================================================================

/// C1866: Batch enqueue operation inserting multiple items atomically
#[test]
fn c1866_batch_enqueue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

#define QUE_BATCH_CAP 128

typedef struct {
    int data[QUE_BATCH_CAP];
    int head;
    int tail;
    int count;
} que_batch_t;

void que_batch_init(que_batch_t *q) {
    q->head = 0;
    q->tail = 0;
    q->count = 0;
}

int que_batch_enqueue_many(que_batch_t *q, const int *items, int n) {
    if (q->count + n > QUE_BATCH_CAP) return -1;
    int i;
    for (i = 0; i < n; i++) {
        q->data[q->tail] = items[i];
        q->tail = (q->tail + 1) % QUE_BATCH_CAP;
    }
    q->count = q->count + n;
    return n;
}

int que_batch_dequeue(que_batch_t *q, int *out) {
    if (q->count <= 0) return -1;
    *out = q->data[q->head];
    q->head = (q->head + 1) % QUE_BATCH_CAP;
    q->count = q->count - 1;
    return 0;
}

int que_batch_size(const que_batch_t *q) {
    return q->count;
}

int que_batch_test(void) {
    que_batch_t q;
    que_batch_init(&q);
    int items[5];
    items[0] = 10; items[1] = 20; items[2] = 30; items[3] = 40; items[4] = 50;
    int added = que_batch_enqueue_many(&q, items, 5);
    if (added != 5) return -1;
    if (que_batch_size(&q) != 5) return -2;
    int val = 0;
    que_batch_dequeue(&q, &val);
    if (val != 10) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1866: Batch enqueue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1866: Output should not be empty");
    assert!(code.contains("fn que_batch_init"), "C1866: Should contain que_batch_init");
    assert!(code.contains("fn que_batch_enqueue_many"), "C1866: Should contain que_batch_enqueue_many");
    assert!(code.contains("fn que_batch_dequeue"), "C1866: Should contain que_batch_dequeue");
    Ok(())
}

/// C1867: Queue drain operation removing all elements into a buffer
#[test]
fn c1867_queue_drain() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

#define QUE_DRAIN_CAP 64

typedef struct {
    int data[QUE_DRAIN_CAP];
    int head;
    int tail;
    int count;
} que_drain_t;

void que_drain_init(que_drain_t *q) {
    q->head = 0;
    q->tail = 0;
    q->count = 0;
}

int que_drain_push(que_drain_t *q, int val) {
    if (q->count >= QUE_DRAIN_CAP) return -1;
    q->data[q->tail] = val;
    q->tail = (q->tail + 1) % QUE_DRAIN_CAP;
    q->count = q->count + 1;
    return 0;
}

int que_drain_all(que_drain_t *q, int *output, int max_out) {
    int drained = 0;
    while (q->count > 0 && drained < max_out) {
        output[drained] = q->data[q->head];
        q->head = (q->head + 1) % QUE_DRAIN_CAP;
        q->count = q->count - 1;
        drained = drained + 1;
    }
    return drained;
}

int que_drain_is_empty(const que_drain_t *q) {
    return q->count == 0;
}

int que_drain_test(void) {
    que_drain_t q;
    que_drain_init(&q);
    que_drain_push(&q, 1);
    que_drain_push(&q, 2);
    que_drain_push(&q, 3);
    que_drain_push(&q, 4);
    int buf[10];
    int n = que_drain_all(&q, buf, 10);
    if (n != 4) return -1;
    if (buf[0] != 1 || buf[3] != 4) return -2;
    if (!que_drain_is_empty(&q)) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1867: Queue drain should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1867: Output should not be empty");
    assert!(code.contains("fn que_drain_init"), "C1867: Should contain que_drain_init");
    assert!(code.contains("fn que_drain_push"), "C1867: Should contain que_drain_push");
    assert!(code.contains("fn que_drain_all"), "C1867: Should contain que_drain_all");
    Ok(())
}

/// C1868: Peek-many operation inspecting multiple front elements without removal
#[test]
fn c1868_peek_many() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

#define QUE_PEEK_CAP 64

typedef struct {
    int data[QUE_PEEK_CAP];
    int head;
    int tail;
    int count;
} que_peek_t;

void que_peek_init(que_peek_t *q) {
    q->head = 0;
    q->tail = 0;
    q->count = 0;
}

int que_peek_enqueue(que_peek_t *q, int val) {
    if (q->count >= QUE_PEEK_CAP) return -1;
    q->data[q->tail] = val;
    q->tail = (q->tail + 1) % QUE_PEEK_CAP;
    q->count = q->count + 1;
    return 0;
}

int que_peek_front(const que_peek_t *q) {
    if (q->count <= 0) return -1;
    return q->data[q->head];
}

int que_peek_many(const que_peek_t *q, int *output, int n) {
    int available = q->count;
    if (n > available) n = available;
    int i;
    for (i = 0; i < n; i++) {
        int idx = (q->head + i) % QUE_PEEK_CAP;
        output[i] = q->data[idx];
    }
    return n;
}

int que_peek_dequeue(que_peek_t *q, int *out) {
    if (q->count <= 0) return -1;
    *out = q->data[q->head];
    q->head = (q->head + 1) % QUE_PEEK_CAP;
    q->count = q->count - 1;
    return 0;
}

int que_peek_test(void) {
    que_peek_t q;
    que_peek_init(&q);
    que_peek_enqueue(&q, 10);
    que_peek_enqueue(&q, 20);
    que_peek_enqueue(&q, 30);
    que_peek_enqueue(&q, 40);
    int buf[3];
    int got = que_peek_many(&q, buf, 3);
    if (got != 3) return -1;
    if (buf[0] != 10 || buf[1] != 20 || buf[2] != 30) return -2;
    if (q.count != 4) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1868: Peek-many should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1868: Output should not be empty");
    assert!(code.contains("fn que_peek_init"), "C1868: Should contain que_peek_init");
    assert!(code.contains("fn que_peek_many"), "C1868: Should contain que_peek_many");
    assert!(code.contains("fn que_peek_enqueue"), "C1868: Should contain que_peek_enqueue");
    Ok(())
}

/// C1869: Queue merging -- combine two sorted queues into one sorted result
#[test]
fn c1869_queue_merging() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

#define QUE_MERGE_CAP 128

typedef struct {
    int data[QUE_MERGE_CAP];
    int head;
    int tail;
    int count;
} que_merge_t;

void que_merge_init(que_merge_t *q) {
    q->head = 0;
    q->tail = 0;
    q->count = 0;
}

int que_merge_push(que_merge_t *q, int val) {
    if (q->count >= QUE_MERGE_CAP) return -1;
    q->data[q->tail] = val;
    q->tail = (q->tail + 1) % QUE_MERGE_CAP;
    q->count = q->count + 1;
    return 0;
}

int que_merge_pop(que_merge_t *q, int *out) {
    if (q->count <= 0) return -1;
    *out = q->data[q->head];
    q->head = (q->head + 1) % QUE_MERGE_CAP;
    q->count = q->count - 1;
    return 0;
}

int que_merge_front(const que_merge_t *q) {
    if (q->count <= 0) return -1;
    return q->data[q->head];
}

int que_merge_two(que_merge_t *a, que_merge_t *b, que_merge_t *out) {
    que_merge_init(out);
    int merged = 0;
    while (a->count > 0 && b->count > 0) {
        int va = que_merge_front(a);
        int vb = que_merge_front(b);
        int val = 0;
        if (va <= vb) {
            que_merge_pop(a, &val);
        } else {
            que_merge_pop(b, &val);
        }
        que_merge_push(out, val);
        merged = merged + 1;
    }
    while (a->count > 0) {
        int val = 0;
        que_merge_pop(a, &val);
        que_merge_push(out, val);
        merged = merged + 1;
    }
    while (b->count > 0) {
        int val = 0;
        que_merge_pop(b, &val);
        que_merge_push(out, val);
        merged = merged + 1;
    }
    return merged;
}

int que_merge_test(void) {
    que_merge_t a, b, out;
    que_merge_init(&a);
    que_merge_init(&b);
    que_merge_push(&a, 1); que_merge_push(&a, 3); que_merge_push(&a, 5);
    que_merge_push(&b, 2); que_merge_push(&b, 4); que_merge_push(&b, 6);
    int n = que_merge_two(&a, &b, &out);
    if (n != 6) return -1;
    int val = 0;
    que_merge_pop(&out, &val);
    if (val != 1) return -2;
    que_merge_pop(&out, &val);
    if (val != 2) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1869: Queue merging should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1869: Output should not be empty");
    assert!(code.contains("fn que_merge_init"), "C1869: Should contain que_merge_init");
    assert!(code.contains("fn que_merge_two"), "C1869: Should contain que_merge_two");
    assert!(code.contains("fn que_merge_pop"), "C1869: Should contain que_merge_pop");
    Ok(())
}

/// C1870: Queue splitting -- partition a queue into two by predicate (even/odd)
#[test]
fn c1870_queue_splitting() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

#define QUE_SPLIT_CAP 64

typedef struct {
    int data[QUE_SPLIT_CAP];
    int head;
    int tail;
    int count;
} que_split_t;

void que_split_init(que_split_t *q) {
    q->head = 0;
    q->tail = 0;
    q->count = 0;
}

int que_split_push(que_split_t *q, int val) {
    if (q->count >= QUE_SPLIT_CAP) return -1;
    q->data[q->tail] = val;
    q->tail = (q->tail + 1) % QUE_SPLIT_CAP;
    q->count = q->count + 1;
    return 0;
}

int que_split_pop(que_split_t *q, int *out) {
    if (q->count <= 0) return -1;
    *out = q->data[q->head];
    q->head = (q->head + 1) % QUE_SPLIT_CAP;
    q->count = q->count - 1;
    return 0;
}

int que_split_partition(que_split_t *src, que_split_t *evens, que_split_t *odds) {
    que_split_init(evens);
    que_split_init(odds);
    int total = 0;
    while (src->count > 0) {
        int val = 0;
        que_split_pop(src, &val);
        if (val % 2 == 0) {
            que_split_push(evens, val);
        } else {
            que_split_push(odds, val);
        }
        total = total + 1;
    }
    return total;
}

int que_split_size(const que_split_t *q) {
    return q->count;
}

int que_split_test(void) {
    que_split_t src, evens, odds;
    que_split_init(&src);
    que_split_push(&src, 1);
    que_split_push(&src, 2);
    que_split_push(&src, 3);
    que_split_push(&src, 4);
    que_split_push(&src, 5);
    que_split_push(&src, 6);
    int n = que_split_partition(&src, &evens, &odds);
    if (n != 6) return -1;
    if (que_split_size(&evens) != 3) return -2;
    if (que_split_size(&odds) != 3) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1870: Queue splitting should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1870: Output should not be empty");
    assert!(code.contains("fn que_split_init"), "C1870: Should contain que_split_init");
    assert!(code.contains("fn que_split_partition"), "C1870: Should contain que_split_partition");
    assert!(code.contains("fn que_split_pop"), "C1870: Should contain que_split_pop");
    Ok(())
}

// ============================================================================
// C1871-C1875: Queue Applications
// ============================================================================

/// C1871: Task queue with priority scheduling and completion tracking
#[test]
fn c1871_task_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define QUE_TASK_CAP 64
#define QUE_TASK_PENDING 0
#define QUE_TASK_RUNNING 1
#define QUE_TASK_DONE 2

typedef struct {
    uint32_t task_id;
    int priority;
    int status;
    int result;
} que_task_entry_t;

typedef struct {
    que_task_entry_t tasks[QUE_TASK_CAP];
    int count;
    uint32_t next_id;
    int completed;
} que_task_queue_t;

void que_task_init(que_task_queue_t *tq) {
    tq->count = 0;
    tq->next_id = 1;
    tq->completed = 0;
    int i;
    for (i = 0; i < QUE_TASK_CAP; i++) {
        tq->tasks[i].status = -1;
    }
}

uint32_t que_task_submit(que_task_queue_t *tq, int priority) {
    if (tq->count >= QUE_TASK_CAP) return 0;
    int i;
    for (i = 0; i < QUE_TASK_CAP; i++) {
        if (tq->tasks[i].status < 0) {
            tq->tasks[i].task_id = tq->next_id;
            tq->tasks[i].priority = priority;
            tq->tasks[i].status = QUE_TASK_PENDING;
            tq->tasks[i].result = 0;
            tq->next_id = tq->next_id + 1;
            tq->count = tq->count + 1;
            return tq->tasks[i].task_id;
        }
    }
    return 0;
}

int que_task_schedule(que_task_queue_t *tq) {
    int best = -1;
    int best_pri = -1;
    int i;
    for (i = 0; i < QUE_TASK_CAP; i++) {
        if (tq->tasks[i].status == QUE_TASK_PENDING) {
            if (best < 0 || tq->tasks[i].priority > best_pri) {
                best = i;
                best_pri = tq->tasks[i].priority;
            }
        }
    }
    if (best < 0) return -1;
    tq->tasks[best].status = QUE_TASK_RUNNING;
    return tq->tasks[best].task_id;
}

int que_task_complete(que_task_queue_t *tq, uint32_t task_id, int result) {
    int i;
    for (i = 0; i < QUE_TASK_CAP; i++) {
        if (tq->tasks[i].task_id == task_id && tq->tasks[i].status == QUE_TASK_RUNNING) {
            tq->tasks[i].status = QUE_TASK_DONE;
            tq->tasks[i].result = result;
            tq->completed = tq->completed + 1;
            return 0;
        }
    }
    return -1;
}

int que_task_test(void) {
    que_task_queue_t tq;
    que_task_init(&tq);
    que_task_submit(&tq, 5);
    que_task_submit(&tq, 10);
    que_task_submit(&tq, 3);
    int tid = que_task_schedule(&tq);
    if (tid <= 0) return -1;
    que_task_complete(&tq, tid, 42);
    if (tq.completed != 1) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1871: Task queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1871: Output should not be empty");
    assert!(code.contains("fn que_task_init"), "C1871: Should contain que_task_init");
    assert!(code.contains("fn que_task_submit"), "C1871: Should contain que_task_submit");
    assert!(code.contains("fn que_task_schedule"), "C1871: Should contain que_task_schedule");
    Ok(())
}

/// C1872: Event queue dispatching typed events to handlers
#[test]
fn c1872_event_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define QUE_EVT_CAP 64
#define QUE_EVT_DATA_LEN 8

typedef struct {
    uint32_t event_type;
    uint32_t timestamp;
    int data[QUE_EVT_DATA_LEN];
    int data_len;
} que_event_t;

typedef struct {
    que_event_t events[QUE_EVT_CAP];
    int head;
    int tail;
    int count;
    uint32_t events_processed;
    uint32_t events_dropped;
} que_evtqueue_t;

void que_evtqueue_init(que_evtqueue_t *eq) {
    eq->head = 0;
    eq->tail = 0;
    eq->count = 0;
    eq->events_processed = 0;
    eq->events_dropped = 0;
}

int que_evtqueue_post(que_evtqueue_t *eq, uint32_t etype, uint32_t ts, const int *data, int dlen) {
    if (eq->count >= QUE_EVT_CAP) {
        eq->events_dropped = eq->events_dropped + 1;
        return -1;
    }
    que_event_t *e = &eq->events[eq->tail];
    e->event_type = etype;
    e->timestamp = ts;
    e->data_len = dlen;
    if (dlen > QUE_EVT_DATA_LEN) dlen = QUE_EVT_DATA_LEN;
    int i;
    for (i = 0; i < dlen; i++) {
        e->data[i] = data[i];
    }
    eq->tail = (eq->tail + 1) % QUE_EVT_CAP;
    eq->count = eq->count + 1;
    return 0;
}

int que_evtqueue_dispatch(que_evtqueue_t *eq, que_event_t *out) {
    if (eq->count <= 0) return -1;
    *out = eq->events[eq->head];
    eq->head = (eq->head + 1) % QUE_EVT_CAP;
    eq->count = eq->count - 1;
    eq->events_processed = eq->events_processed + 1;
    return 0;
}

int que_evtqueue_pending(const que_evtqueue_t *eq) {
    return eq->count;
}

int que_evtqueue_test(void) {
    que_evtqueue_t eq;
    que_evtqueue_init(&eq);
    int data[2];
    data[0] = 42; data[1] = 99;
    que_evtqueue_post(&eq, 1, 1000, data, 2);
    que_evtqueue_post(&eq, 2, 1001, data, 1);
    if (que_evtqueue_pending(&eq) != 2) return -1;
    que_event_t evt;
    que_evtqueue_dispatch(&eq, &evt);
    if (evt.event_type != 1) return -2;
    if (evt.timestamp != 1000) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1872: Event queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1872: Output should not be empty");
    assert!(code.contains("fn que_evtqueue_init"), "C1872: Should contain que_evtqueue_init");
    assert!(code.contains("fn que_evtqueue_post"), "C1872: Should contain que_evtqueue_post");
    assert!(code.contains("fn que_evtqueue_dispatch"), "C1872: Should contain que_evtqueue_dispatch");
    Ok(())
}

/// C1873: Command queue with execute and rollback support
#[test]
fn c1873_command_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define QUE_CMD_CAP 32
#define QUE_CMD_SET 1
#define QUE_CMD_ADD 2
#define QUE_CMD_MUL 3

typedef struct {
    int cmd_type;
    int operand;
    int prev_value;
    int executed;
} que_cmd_t;

typedef struct {
    que_cmd_t commands[QUE_CMD_CAP];
    int head;
    int tail;
    int count;
    int state_value;
} que_cmdqueue_t;

void que_cmdqueue_init(que_cmdqueue_t *cq, int initial_value) {
    cq->head = 0;
    cq->tail = 0;
    cq->count = 0;
    cq->state_value = initial_value;
}

int que_cmdqueue_enqueue(que_cmdqueue_t *cq, int cmd_type, int operand) {
    if (cq->count >= QUE_CMD_CAP) return -1;
    cq->commands[cq->tail].cmd_type = cmd_type;
    cq->commands[cq->tail].operand = operand;
    cq->commands[cq->tail].prev_value = 0;
    cq->commands[cq->tail].executed = 0;
    cq->tail = (cq->tail + 1) % QUE_CMD_CAP;
    cq->count = cq->count + 1;
    return 0;
}

int que_cmdqueue_execute_next(que_cmdqueue_t *cq) {
    if (cq->count <= 0) return -1;
    que_cmd_t *cmd = &cq->commands[cq->head];
    cmd->prev_value = cq->state_value;
    if (cmd->cmd_type == QUE_CMD_SET) {
        cq->state_value = cmd->operand;
    } else if (cmd->cmd_type == QUE_CMD_ADD) {
        cq->state_value = cq->state_value + cmd->operand;
    } else if (cmd->cmd_type == QUE_CMD_MUL) {
        cq->state_value = cq->state_value * cmd->operand;
    }
    cmd->executed = 1;
    cq->head = (cq->head + 1) % QUE_CMD_CAP;
    cq->count = cq->count - 1;
    return cq->state_value;
}

int que_cmdqueue_get_state(const que_cmdqueue_t *cq) {
    return cq->state_value;
}

int que_cmdqueue_test(void) {
    que_cmdqueue_t cq;
    que_cmdqueue_init(&cq, 10);
    que_cmdqueue_enqueue(&cq, QUE_CMD_ADD, 5);
    que_cmdqueue_enqueue(&cq, QUE_CMD_MUL, 3);
    que_cmdqueue_execute_next(&cq);
    if (que_cmdqueue_get_state(&cq) != 15) return -1;
    que_cmdqueue_execute_next(&cq);
    if (que_cmdqueue_get_state(&cq) != 45) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1873: Command queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1873: Output should not be empty");
    assert!(code.contains("fn que_cmdqueue_init"), "C1873: Should contain que_cmdqueue_init");
    assert!(code.contains("fn que_cmdqueue_enqueue"), "C1873: Should contain que_cmdqueue_enqueue");
    assert!(code.contains("fn que_cmdqueue_execute_next"), "C1873: Should contain que_cmdqueue_execute_next");
    Ok(())
}

/// C1874: Undo queue recording operations for revert capability
#[test]
fn c1874_undo_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

#define QUE_UNDO_CAP 32

typedef struct {
    int old_value;
    int new_value;
    int field_id;
} que_undo_entry_t;

typedef struct {
    que_undo_entry_t history[QUE_UNDO_CAP];
    int top;
    int redo_top;
    int values[16];
} que_undo_t;

void que_undo_init(que_undo_t *u) {
    u->top = 0;
    u->redo_top = 0;
    int i;
    for (i = 0; i < 16; i++) {
        u->values[i] = 0;
    }
}

int que_undo_set(que_undo_t *u, int field, int new_val) {
    if (field < 0 || field >= 16) return -1;
    if (u->top >= QUE_UNDO_CAP) return -2;
    u->history[u->top].field_id = field;
    u->history[u->top].old_value = u->values[field];
    u->history[u->top].new_value = new_val;
    u->top = u->top + 1;
    u->redo_top = u->top;
    u->values[field] = new_val;
    return 0;
}

int que_undo_revert(que_undo_t *u) {
    if (u->top <= 0) return -1;
    u->top = u->top - 1;
    int field = u->history[u->top].field_id;
    u->values[field] = u->history[u->top].old_value;
    return 0;
}

int que_undo_redo(que_undo_t *u) {
    if (u->top >= u->redo_top) return -1;
    int field = u->history[u->top].field_id;
    u->values[field] = u->history[u->top].new_value;
    u->top = u->top + 1;
    return 0;
}

int que_undo_get(const que_undo_t *u, int field) {
    if (field < 0 || field >= 16) return -1;
    return u->values[field];
}

int que_undo_test(void) {
    que_undo_t u;
    que_undo_init(&u);
    que_undo_set(&u, 0, 100);
    que_undo_set(&u, 0, 200);
    if (que_undo_get(&u, 0) != 200) return -1;
    que_undo_revert(&u);
    if (que_undo_get(&u, 0) != 100) return -2;
    que_undo_redo(&u);
    if (que_undo_get(&u, 0) != 200) return -3;
    que_undo_revert(&u);
    que_undo_revert(&u);
    if (que_undo_get(&u, 0) != 0) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1874: Undo queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1874: Output should not be empty");
    assert!(code.contains("fn que_undo_init"), "C1874: Should contain que_undo_init");
    assert!(code.contains("fn que_undo_set"), "C1874: Should contain que_undo_set");
    assert!(code.contains("fn que_undo_revert"), "C1874: Should contain que_undo_revert");
    Ok(())
}

/// C1875: Log queue with severity levels and circular overwrite
#[test]
fn c1875_log_queue() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define QUE_LOG_CAP 128
#define QUE_LOG_DEBUG 0
#define QUE_LOG_INFO 1
#define QUE_LOG_WARN 2
#define QUE_LOG_ERROR 3

typedef struct {
    int severity;
    uint32_t timestamp;
    int code;
    int value;
} que_log_entry_t;

typedef struct {
    que_log_entry_t entries[QUE_LOG_CAP];
    int write_pos;
    int count;
    int min_severity;
    uint32_t total_logged;
    int error_count;
} que_logqueue_t;

void que_logqueue_init(que_logqueue_t *lq, int min_sev) {
    lq->write_pos = 0;
    lq->count = 0;
    lq->min_severity = min_sev;
    lq->total_logged = 0;
    lq->error_count = 0;
}

int que_logqueue_write(que_logqueue_t *lq, int severity, uint32_t ts, int code, int value) {
    if (severity < lq->min_severity) return -1;
    que_log_entry_t *e = &lq->entries[lq->write_pos % QUE_LOG_CAP];
    e->severity = severity;
    e->timestamp = ts;
    e->code = code;
    e->value = value;
    lq->write_pos = lq->write_pos + 1;
    if (lq->count < QUE_LOG_CAP) {
        lq->count = lq->count + 1;
    }
    lq->total_logged = lq->total_logged + 1;
    if (severity == QUE_LOG_ERROR) {
        lq->error_count = lq->error_count + 1;
    }
    return 0;
}

int que_logqueue_read_recent(const que_logqueue_t *lq, int n, que_log_entry_t *out) {
    if (n > lq->count) n = lq->count;
    int i;
    for (i = 0; i < n; i++) {
        int idx = (lq->write_pos - n + i + QUE_LOG_CAP) % QUE_LOG_CAP;
        out[i] = lq->entries[idx];
    }
    return n;
}

int que_logqueue_count_by_severity(const que_logqueue_t *lq, int severity) {
    int cnt = 0;
    int start = 0;
    if (lq->count >= QUE_LOG_CAP) {
        start = lq->write_pos;
    }
    int i;
    for (i = 0; i < lq->count; i++) {
        int idx = (start + i) % QUE_LOG_CAP;
        if (lq->entries[idx].severity == severity) {
            cnt = cnt + 1;
        }
    }
    return cnt;
}

int que_logqueue_test(void) {
    que_logqueue_t lq;
    que_logqueue_init(&lq, QUE_LOG_INFO);
    que_logqueue_write(&lq, QUE_LOG_DEBUG, 1, 100, 0);
    que_logqueue_write(&lq, QUE_LOG_INFO, 2, 101, 10);
    que_logqueue_write(&lq, QUE_LOG_ERROR, 3, 102, 20);
    que_logqueue_write(&lq, QUE_LOG_WARN, 4, 103, 30);
    if (lq.count != 3) return -1;
    if (lq.error_count != 1) return -2;
    que_log_entry_t recent[2];
    int got = que_logqueue_read_recent(&lq, 2, recent);
    if (got != 2) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1875: Log queue should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1875: Output should not be empty");
    assert!(code.contains("fn que_logqueue_init"), "C1875: Should contain que_logqueue_init");
    assert!(code.contains("fn que_logqueue_write"), "C1875: Should contain que_logqueue_write");
    assert!(code.contains("fn que_logqueue_read_recent"), "C1875: Should contain que_logqueue_read_recent");
    Ok(())
}
