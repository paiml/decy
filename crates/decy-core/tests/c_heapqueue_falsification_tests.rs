//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1226-C1250: Heap & Priority Queue Implementations -- binary heaps,
//! advanced heap variants, priority queues, and heap-based applications.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world heap and priority queue patterns commonly
//! found in operating system schedulers, graph algorithms, compression codecs,
//! and real-time event systems -- all expressed as valid C99 without #include
//! directives.
//!
//! Organization:
//! - C1226-C1230: Binary heaps (min-heap, max-heap, heapify, heap sort, k-way merge)
//! - C1231-C1235: Advanced heaps (d-ary heap, binomial heap, pairing heap, leftist heap, skew heap)
//! - C1236-C1240: Priority queues (indexed PQ, double-ended PQ, bounded PQ, lazy deletion, median maintenance)
//! - C1241-C1245: Heap applications (Huffman tree, running median, k closest points, merge k sorted, task scheduler)
//! - C1246-C1250: Specialized (interval scheduling, bandwidth allocation, event queue, Dijkstra PQ, A* open set)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C1226-C1230: Binary Heaps
// ============================================================================

/// C1226: Min-heap with insert, extract-min, and peek
#[test]
fn c1226_min_heap() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_MIN_CAP 256

typedef struct {
    int data[HEAP_MIN_CAP];
    int size;
} heap_min_t;

void heap_min_init(heap_min_t *h) {
    h->size = 0;
}

static void heap_min_swap(int *a, int *b) {
    int tmp = *a;
    *a = *b;
    *b = tmp;
}

static void heap_min_sift_up(heap_min_t *h, int idx) {
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (h->data[parent] > h->data[idx]) {
            heap_min_swap(&h->data[parent], &h->data[idx]);
            idx = parent;
        } else {
            break;
        }
    }
}

static void heap_min_sift_down(heap_min_t *h, int idx) {
    while (1) {
        int smallest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < h->size && h->data[left] < h->data[smallest])
            smallest = left;
        if (right < h->size && h->data[right] < h->data[smallest])
            smallest = right;
        if (smallest != idx) {
            heap_min_swap(&h->data[idx], &h->data[smallest]);
            idx = smallest;
        } else {
            break;
        }
    }
}

int heap_min_insert(heap_min_t *h, int val) {
    if (h->size >= HEAP_MIN_CAP) return -1;
    h->data[h->size] = val;
    heap_min_sift_up(h, h->size);
    h->size++;
    return 0;
}

int heap_min_peek(const heap_min_t *h) {
    if (h->size == 0) return -1;
    return h->data[0];
}

int heap_min_extract(heap_min_t *h, int *out) {
    if (h->size == 0) return -1;
    *out = h->data[0];
    h->size--;
    h->data[0] = h->data[h->size];
    heap_min_sift_down(h, 0);
    return 0;
}

int heap_min_test(void) {
    heap_min_t h;
    heap_min_init(&h);
    heap_min_insert(&h, 50);
    heap_min_insert(&h, 20);
    heap_min_insert(&h, 80);
    heap_min_insert(&h, 10);
    heap_min_insert(&h, 40);
    if (heap_min_peek(&h) != 10) return -1;
    int val = 0;
    heap_min_extract(&h, &val);
    if (val != 10) return -2;
    heap_min_extract(&h, &val);
    if (val != 20) return -3;
    if (h.size != 3) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1226: Min-heap should transpile: {:?}", result.err());
}

/// C1227: Max-heap with insert, extract-max, and increase-key
#[test]
fn c1227_max_heap() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_MAX_CAP 256

typedef struct {
    int data[HEAP_MAX_CAP];
    int size;
} heap_max_t;

void heap_max_init(heap_max_t *h) {
    h->size = 0;
}

static void heap_max_swap(int *a, int *b) {
    int tmp = *a;
    *a = *b;
    *b = tmp;
}

static void heap_max_sift_up(heap_max_t *h, int idx) {
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (h->data[parent] < h->data[idx]) {
            heap_max_swap(&h->data[parent], &h->data[idx]);
            idx = parent;
        } else {
            break;
        }
    }
}

static void heap_max_sift_down(heap_max_t *h, int idx) {
    while (1) {
        int largest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < h->size && h->data[left] > h->data[largest])
            largest = left;
        if (right < h->size && h->data[right] > h->data[largest])
            largest = right;
        if (largest != idx) {
            heap_max_swap(&h->data[idx], &h->data[largest]);
            idx = largest;
        } else {
            break;
        }
    }
}

int heap_max_insert(heap_max_t *h, int val) {
    if (h->size >= HEAP_MAX_CAP) return -1;
    h->data[h->size] = val;
    heap_max_sift_up(h, h->size);
    h->size++;
    return 0;
}

int heap_max_extract(heap_max_t *h, int *out) {
    if (h->size == 0) return -1;
    *out = h->data[0];
    h->size--;
    h->data[0] = h->data[h->size];
    heap_max_sift_down(h, 0);
    return 0;
}

int heap_max_increase_key(heap_max_t *h, int idx, int new_val) {
    if (idx < 0 || idx >= h->size) return -1;
    if (new_val < h->data[idx]) return -2;
    h->data[idx] = new_val;
    heap_max_sift_up(h, idx);
    return 0;
}

int heap_max_test(void) {
    heap_max_t h;
    heap_max_init(&h);
    heap_max_insert(&h, 10);
    heap_max_insert(&h, 50);
    heap_max_insert(&h, 30);
    heap_max_insert(&h, 70);
    int val = 0;
    heap_max_extract(&h, &val);
    if (val != 70) return -1;
    heap_max_extract(&h, &val);
    if (val != 50) return -2;
    heap_max_increase_key(&h, 0, 100);
    heap_max_extract(&h, &val);
    if (val != 100) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1227: Max-heap should transpile: {:?}", result.err());
}

/// C1228: Build heap via heapify (Floyd's algorithm, O(n))
#[test]
fn c1228_heapify_build() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_BLD_CAP 128

static void heap_bld_swap(int *a, int *b) {
    int tmp = *a;
    *a = *b;
    *b = tmp;
}

static void heap_bld_sift_down(int *arr, int n, int idx) {
    while (1) {
        int smallest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < n && arr[left] < arr[smallest])
            smallest = left;
        if (right < n && arr[right] < arr[smallest])
            smallest = right;
        if (smallest != idx) {
            heap_bld_swap(&arr[idx], &arr[smallest]);
            idx = smallest;
        } else {
            break;
        }
    }
}

void heap_bld_heapify(int *arr, int n) {
    int i;
    for (i = n / 2 - 1; i >= 0; i--) {
        heap_bld_sift_down(arr, n, i);
    }
}

int heap_bld_is_min_heap(const int *arr, int n) {
    int i;
    for (i = 0; i < n; i++) {
        int left = 2 * i + 1;
        int right = 2 * i + 2;
        if (left < n && arr[i] > arr[left]) return 0;
        if (right < n && arr[i] > arr[right]) return 0;
    }
    return 1;
}

int heap_bld_test(void) {
    int arr[10] = {50, 30, 80, 10, 70, 20, 90, 40, 60, 5};
    heap_bld_heapify(arr, 10);
    if (!heap_bld_is_min_heap(arr, 10)) return -1;
    if (arr[0] != 5) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1228: Heapify build should transpile: {:?}", result.err());
}

/// C1229: Heap sort (in-place, ascending order via max-heap)
#[test]
fn c1229_heap_sort() {
    let c_code = r#"
typedef unsigned long size_t;

static void heap_sort_swap(int *a, int *b) {
    int tmp = *a;
    *a = *b;
    *b = tmp;
}

static void heap_sort_sift_down(int *arr, int n, int idx) {
    while (1) {
        int largest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < n && arr[left] > arr[largest])
            largest = left;
        if (right < n && arr[right] > arr[largest])
            largest = right;
        if (largest != idx) {
            heap_sort_swap(&arr[idx], &arr[largest]);
            idx = largest;
        } else {
            break;
        }
    }
}

void heap_sort_run(int *arr, int n) {
    int i;
    for (i = n / 2 - 1; i >= 0; i--) {
        heap_sort_sift_down(arr, n, i);
    }
    for (i = n - 1; i > 0; i--) {
        heap_sort_swap(&arr[0], &arr[i]);
        heap_sort_sift_down(arr, i, 0);
    }
}

int heap_sort_is_sorted(const int *arr, int n) {
    int i;
    for (i = 1; i < n; i++) {
        if (arr[i] < arr[i - 1]) return 0;
    }
    return 1;
}

int heap_sort_test(void) {
    int arr[8] = {64, 25, 12, 22, 11, 90, 45, 33};
    heap_sort_run(arr, 8);
    if (!heap_sort_is_sorted(arr, 8)) return -1;
    if (arr[0] != 11) return -2;
    if (arr[7] != 90) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1229: Heap sort should transpile: {:?}", result.err());
}

/// C1230: K-way merge using a min-heap of (value, list_index) pairs
#[test]
fn c1230_kway_merge() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_KM_MAX_K 16
#define HEAP_KM_MAX_LEN 64

typedef struct {
    int value;
    int list_idx;
} heap_km_entry_t;

typedef struct {
    heap_km_entry_t data[HEAP_KM_MAX_K];
    int size;
} heap_km_minheap_t;

static void heap_km_swap_entry(heap_km_entry_t *a, heap_km_entry_t *b) {
    heap_km_entry_t tmp = *a;
    *a = *b;
    *b = tmp;
}

static void heap_km_sift_up(heap_km_minheap_t *h, int idx) {
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (h->data[parent].value > h->data[idx].value) {
            heap_km_swap_entry(&h->data[parent], &h->data[idx]);
            idx = parent;
        } else {
            break;
        }
    }
}

static void heap_km_sift_down(heap_km_minheap_t *h, int idx) {
    while (1) {
        int smallest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < h->size && h->data[left].value < h->data[smallest].value)
            smallest = left;
        if (right < h->size && h->data[right].value < h->data[smallest].value)
            smallest = right;
        if (smallest != idx) {
            heap_km_swap_entry(&h->data[idx], &h->data[smallest]);
            idx = smallest;
        } else {
            break;
        }
    }
}

int heap_km_merge(int lists[HEAP_KM_MAX_K][HEAP_KM_MAX_LEN],
                  int lengths[HEAP_KM_MAX_K], int k,
                  int *output, int max_out) {
    heap_km_minheap_t h;
    int cursors[HEAP_KM_MAX_K];
    int i;
    h.size = 0;
    for (i = 0; i < k; i++) {
        cursors[i] = 0;
        if (lengths[i] > 0) {
            h.data[h.size].value = lists[i][0];
            h.data[h.size].list_idx = i;
            h.size++;
            heap_km_sift_up(&h, h.size - 1);
            cursors[i] = 1;
        }
    }
    int out_count = 0;
    while (h.size > 0 && out_count < max_out) {
        heap_km_entry_t top = h.data[0];
        output[out_count++] = top.value;
        int li = top.list_idx;
        if (cursors[li] < lengths[li]) {
            h.data[0].value = lists[li][cursors[li]];
            h.data[0].list_idx = li;
            cursors[li]++;
            heap_km_sift_down(&h, 0);
        } else {
            h.size--;
            h.data[0] = h.data[h.size];
            heap_km_sift_down(&h, 0);
        }
    }
    return out_count;
}

int heap_kway_test(void) {
    int lists[HEAP_KM_MAX_K][HEAP_KM_MAX_LEN];
    int lengths[HEAP_KM_MAX_K];
    lists[0][0] = 1; lists[0][1] = 5; lists[0][2] = 9;
    lengths[0] = 3;
    lists[1][0] = 2; lists[1][1] = 6; lists[1][2] = 10;
    lengths[1] = 3;
    lists[2][0] = 3; lists[2][1] = 7;
    lengths[2] = 2;
    int output[64];
    int n = heap_km_merge(lists, lengths, 3, output, 64);
    if (n != 8) return -1;
    if (output[0] != 1) return -2;
    if (output[7] != 10) return -3;
    int j;
    for (j = 1; j < n; j++) {
        if (output[j] < output[j - 1]) return -4;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1230: K-way merge should transpile: {:?}", result.err());
}

// ============================================================================
// C1231-C1235: Advanced Heaps
// ============================================================================

/// C1231: D-ary heap (generalized heap with D children per node)
#[test]
fn c1231_dary_heap() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_DA_CAP 256
#define HEAP_DA_D 4

typedef struct {
    int data[HEAP_DA_CAP];
    int size;
} heap_dary_t;

void heap_dary_init(heap_dary_t *h) {
    h->size = 0;
}

static void heap_dary_swap(int *a, int *b) {
    int tmp = *a;
    *a = *b;
    *b = tmp;
}

static int heap_dary_parent(int idx) {
    return (idx - 1) / HEAP_DA_D;
}

static int heap_dary_child(int idx, int k) {
    return HEAP_DA_D * idx + k + 1;
}

static void heap_dary_sift_up(heap_dary_t *h, int idx) {
    while (idx > 0) {
        int p = heap_dary_parent(idx);
        if (h->data[p] > h->data[idx]) {
            heap_dary_swap(&h->data[p], &h->data[idx]);
            idx = p;
        } else {
            break;
        }
    }
}

static void heap_dary_sift_down(heap_dary_t *h, int idx) {
    while (1) {
        int smallest = idx;
        int k;
        for (k = 0; k < HEAP_DA_D; k++) {
            int c = heap_dary_child(idx, k);
            if (c < h->size && h->data[c] < h->data[smallest])
                smallest = c;
        }
        if (smallest != idx) {
            heap_dary_swap(&h->data[idx], &h->data[smallest]);
            idx = smallest;
        } else {
            break;
        }
    }
}

int heap_dary_insert(heap_dary_t *h, int val) {
    if (h->size >= HEAP_DA_CAP) return -1;
    h->data[h->size] = val;
    heap_dary_sift_up(h, h->size);
    h->size++;
    return 0;
}

int heap_dary_extract_min(heap_dary_t *h, int *out) {
    if (h->size == 0) return -1;
    *out = h->data[0];
    h->size--;
    h->data[0] = h->data[h->size];
    heap_dary_sift_down(h, 0);
    return 0;
}

int heap_dary_test(void) {
    heap_dary_t h;
    heap_dary_init(&h);
    int vals[8] = {40, 10, 60, 20, 50, 30, 70, 5};
    int i;
    for (i = 0; i < 8; i++) {
        heap_dary_insert(&h, vals[i]);
    }
    int prev = -1;
    int val;
    for (i = 0; i < 8; i++) {
        heap_dary_extract_min(&h, &val);
        if (val < prev) return -1;
        prev = val;
    }
    if (h.size != 0) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1231: D-ary heap should transpile: {:?}", result.err());
}

/// C1232: Binomial heap (array-of-trees representation)
#[test]
fn c1232_binomial_heap() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_BN_MAX_NODES 256
#define HEAP_BN_MAX_ORDER 8

typedef struct {
    int key;
    int order;
    int parent;
    int child;
    int sibling;
    int active;
} heap_bn_node_t;

typedef struct {
    heap_bn_node_t nodes[HEAP_BN_MAX_NODES];
    int roots[HEAP_BN_MAX_ORDER];
    int free_idx;
    int size;
} heap_binomial_t;

void heap_bn_init(heap_binomial_t *h) {
    h->free_idx = 0;
    h->size = 0;
    int i;
    for (i = 0; i < HEAP_BN_MAX_ORDER; i++) {
        h->roots[i] = -1;
    }
}

static int heap_bn_alloc(heap_binomial_t *h, int key) {
    if (h->free_idx >= HEAP_BN_MAX_NODES) return -1;
    int idx = h->free_idx++;
    h->nodes[idx].key = key;
    h->nodes[idx].order = 0;
    h->nodes[idx].parent = -1;
    h->nodes[idx].child = -1;
    h->nodes[idx].sibling = -1;
    h->nodes[idx].active = 1;
    return idx;
}

static int heap_bn_link(heap_binomial_t *h, int a, int b) {
    if (h->nodes[a].key > h->nodes[b].key) {
        int tmp = a; a = b; b = tmp;
    }
    h->nodes[b].parent = a;
    h->nodes[b].sibling = h->nodes[a].child;
    h->nodes[a].child = b;
    h->nodes[a].order++;
    return a;
}

int heap_bn_insert(heap_binomial_t *h, int key) {
    int node = heap_bn_alloc(h, key);
    if (node < 0) return -1;
    int carry = node;
    int i;
    for (i = 0; i < HEAP_BN_MAX_ORDER && carry >= 0; i++) {
        if (h->roots[i] < 0) {
            h->roots[i] = carry;
            carry = -1;
        } else {
            carry = heap_bn_link(h, h->roots[i], carry);
            h->roots[i] = -1;
        }
    }
    h->size++;
    return 0;
}

int heap_bn_find_min(const heap_binomial_t *h) {
    int min_val = 2147483647;
    int i;
    for (i = 0; i < HEAP_BN_MAX_ORDER; i++) {
        if (h->roots[i] >= 0 && h->nodes[h->roots[i]].key < min_val) {
            min_val = h->nodes[h->roots[i]].key;
        }
    }
    return min_val;
}

int heap_binomial_test(void) {
    heap_binomial_t h;
    heap_bn_init(&h);
    heap_bn_insert(&h, 50);
    heap_bn_insert(&h, 20);
    heap_bn_insert(&h, 80);
    heap_bn_insert(&h, 10);
    heap_bn_insert(&h, 40);
    if (heap_bn_find_min(&h) != 10) return -1;
    if (h.size != 5) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1232: Binomial heap should transpile: {:?}", result.err());
}

/// C1233: Pairing heap (array-based, simplified)
#[test]
fn c1233_pairing_heap() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_PH_MAX 256

typedef struct {
    int key;
    int child;
    int sibling;
    int active;
} heap_ph_node_t;

typedef struct {
    heap_ph_node_t nodes[HEAP_PH_MAX];
    int root;
    int free_idx;
    int size;
} heap_pairing_t;

void heap_ph_init(heap_pairing_t *h) {
    h->root = -1;
    h->free_idx = 0;
    h->size = 0;
}

static int heap_ph_alloc(heap_pairing_t *h, int key) {
    if (h->free_idx >= HEAP_PH_MAX) return -1;
    int idx = h->free_idx++;
    h->nodes[idx].key = key;
    h->nodes[idx].child = -1;
    h->nodes[idx].sibling = -1;
    h->nodes[idx].active = 1;
    return idx;
}

static int heap_ph_merge(heap_pairing_t *h, int a, int b) {
    if (a < 0) return b;
    if (b < 0) return a;
    if (h->nodes[a].key <= h->nodes[b].key) {
        h->nodes[b].sibling = h->nodes[a].child;
        h->nodes[a].child = b;
        return a;
    } else {
        h->nodes[a].sibling = h->nodes[b].child;
        h->nodes[b].child = a;
        return b;
    }
}

int heap_ph_insert(heap_pairing_t *h, int key) {
    int node = heap_ph_alloc(h, key);
    if (node < 0) return -1;
    h->root = heap_ph_merge(h, h->root, node);
    h->size++;
    return 0;
}

int heap_ph_find_min(const heap_pairing_t *h) {
    if (h->root < 0) return -1;
    return h->nodes[h->root].key;
}

static int heap_ph_two_pass(heap_pairing_t *h, int first_child) {
    if (first_child < 0) return -1;
    int pairs[HEAP_PH_MAX];
    int count = 0;
    int cur = first_child;
    while (cur >= 0) {
        int next_sib = h->nodes[cur].sibling;
        h->nodes[cur].sibling = -1;
        if (next_sib >= 0) {
            int after = h->nodes[next_sib].sibling;
            h->nodes[next_sib].sibling = -1;
            pairs[count++] = heap_ph_merge(h, cur, next_sib);
            cur = after;
        } else {
            pairs[count++] = cur;
            cur = -1;
        }
    }
    int result = pairs[count - 1];
    int i;
    for (i = count - 2; i >= 0; i--) {
        result = heap_ph_merge(h, result, pairs[i]);
    }
    return result;
}

int heap_ph_extract_min(heap_pairing_t *h, int *out) {
    if (h->root < 0) return -1;
    *out = h->nodes[h->root].key;
    int children = h->nodes[h->root].child;
    h->nodes[h->root].active = 0;
    h->root = heap_ph_two_pass(h, children);
    h->size--;
    return 0;
}

int heap_pairing_test(void) {
    heap_pairing_t h;
    heap_ph_init(&h);
    heap_ph_insert(&h, 30);
    heap_ph_insert(&h, 10);
    heap_ph_insert(&h, 50);
    heap_ph_insert(&h, 5);
    if (heap_ph_find_min(&h) != 5) return -1;
    int val = 0;
    heap_ph_extract_min(&h, &val);
    if (val != 5) return -2;
    if (heap_ph_find_min(&h) != 10) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1233: Pairing heap should transpile: {:?}", result.err());
}

/// C1234: Leftist heap (merge-centric min-heap with rank tracking)
#[test]
fn c1234_leftist_heap() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_LH_MAX 256

typedef struct {
    int key;
    int rank;
    int left;
    int right;
    int active;
} heap_lh_node_t;

typedef struct {
    heap_lh_node_t nodes[HEAP_LH_MAX];
    int root;
    int free_idx;
    int size;
} heap_leftist_t;

void heap_lh_init(heap_leftist_t *h) {
    h->root = -1;
    h->free_idx = 0;
    h->size = 0;
}

static int heap_lh_alloc(heap_leftist_t *h, int key) {
    if (h->free_idx >= HEAP_LH_MAX) return -1;
    int idx = h->free_idx++;
    h->nodes[idx].key = key;
    h->nodes[idx].rank = 1;
    h->nodes[idx].left = -1;
    h->nodes[idx].right = -1;
    h->nodes[idx].active = 1;
    return idx;
}

static int heap_lh_get_rank(const heap_leftist_t *h, int idx) {
    if (idx < 0) return 0;
    return h->nodes[idx].rank;
}

static int heap_lh_merge(heap_leftist_t *h, int a, int b) {
    if (a < 0) return b;
    if (b < 0) return a;
    if (h->nodes[a].key > h->nodes[b].key) {
        int tmp = a; a = b; b = tmp;
    }
    h->nodes[a].right = heap_lh_merge(h, h->nodes[a].right, b);
    if (heap_lh_get_rank(h, h->nodes[a].left) < heap_lh_get_rank(h, h->nodes[a].right)) {
        int tmp = h->nodes[a].left;
        h->nodes[a].left = h->nodes[a].right;
        h->nodes[a].right = tmp;
    }
    h->nodes[a].rank = heap_lh_get_rank(h, h->nodes[a].right) + 1;
    return a;
}

int heap_lh_insert(heap_leftist_t *h, int key) {
    int node = heap_lh_alloc(h, key);
    if (node < 0) return -1;
    h->root = heap_lh_merge(h, h->root, node);
    h->size++;
    return 0;
}

int heap_lh_extract_min(heap_leftist_t *h, int *out) {
    if (h->root < 0) return -1;
    *out = h->nodes[h->root].key;
    int left = h->nodes[h->root].left;
    int right = h->nodes[h->root].right;
    h->nodes[h->root].active = 0;
    h->root = heap_lh_merge(h, left, right);
    h->size--;
    return 0;
}

int heap_leftist_test(void) {
    heap_leftist_t h;
    heap_lh_init(&h);
    heap_lh_insert(&h, 40);
    heap_lh_insert(&h, 15);
    heap_lh_insert(&h, 60);
    heap_lh_insert(&h, 8);
    heap_lh_insert(&h, 25);
    int val = 0;
    heap_lh_extract_min(&h, &val);
    if (val != 8) return -1;
    heap_lh_extract_min(&h, &val);
    if (val != 15) return -2;
    if (h.size != 3) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1234: Leftist heap should transpile: {:?}", result.err());
}

/// C1235: Skew heap (self-adjusting leftist heap, simpler merge)
#[test]
fn c1235_skew_heap() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_SK_MAX 256

typedef struct {
    int key;
    int left;
    int right;
    int active;
} heap_sk_node_t;

typedef struct {
    heap_sk_node_t nodes[HEAP_SK_MAX];
    int root;
    int free_idx;
    int size;
} heap_skew_t;

void heap_sk_init(heap_skew_t *h) {
    h->root = -1;
    h->free_idx = 0;
    h->size = 0;
}

static int heap_sk_alloc(heap_skew_t *h, int key) {
    if (h->free_idx >= HEAP_SK_MAX) return -1;
    int idx = h->free_idx++;
    h->nodes[idx].key = key;
    h->nodes[idx].left = -1;
    h->nodes[idx].right = -1;
    h->nodes[idx].active = 1;
    return idx;
}

static int heap_sk_merge(heap_skew_t *h, int a, int b) {
    if (a < 0) return b;
    if (b < 0) return a;
    if (h->nodes[a].key > h->nodes[b].key) {
        int tmp = a; a = b; b = tmp;
    }
    int tmp = h->nodes[a].left;
    h->nodes[a].left = heap_sk_merge(h, h->nodes[a].right, b);
    h->nodes[a].right = tmp;
    return a;
}

int heap_sk_insert(heap_skew_t *h, int key) {
    int node = heap_sk_alloc(h, key);
    if (node < 0) return -1;
    h->root = heap_sk_merge(h, h->root, node);
    h->size++;
    return 0;
}

int heap_sk_extract_min(heap_skew_t *h, int *out) {
    if (h->root < 0) return -1;
    *out = h->nodes[h->root].key;
    int left = h->nodes[h->root].left;
    int right = h->nodes[h->root].right;
    h->nodes[h->root].active = 0;
    h->root = heap_sk_merge(h, left, right);
    h->size--;
    return 0;
}

int heap_skew_test(void) {
    heap_skew_t h;
    heap_sk_init(&h);
    heap_sk_insert(&h, 35);
    heap_sk_insert(&h, 12);
    heap_sk_insert(&h, 55);
    heap_sk_insert(&h, 3);
    heap_sk_insert(&h, 22);
    int val = 0;
    heap_sk_extract_min(&h, &val);
    if (val != 3) return -1;
    heap_sk_extract_min(&h, &val);
    if (val != 12) return -2;
    heap_sk_extract_min(&h, &val);
    if (val != 22) return -3;
    if (h.size != 2) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1235: Skew heap should transpile: {:?}", result.err());
}

// ============================================================================
// C1236-C1240: Priority Queues
// ============================================================================

/// C1236: Indexed priority queue (supports decrease-key by handle)
#[test]
fn c1236_indexed_priority_queue() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_IPQ_CAP 128

typedef struct {
    int keys[HEAP_IPQ_CAP];
    int heap[HEAP_IPQ_CAP];
    int pos[HEAP_IPQ_CAP];
    int in_heap[HEAP_IPQ_CAP];
    int size;
} heap_ipq_t;

void heap_ipq_init(heap_ipq_t *pq, int n) {
    pq->size = 0;
    int i;
    for (i = 0; i < n && i < HEAP_IPQ_CAP; i++) {
        pq->in_heap[i] = 0;
        pq->pos[i] = -1;
    }
}

static void heap_ipq_swap(heap_ipq_t *pq, int i, int j) {
    int a = pq->heap[i];
    int b = pq->heap[j];
    pq->heap[i] = b;
    pq->heap[j] = a;
    pq->pos[b] = i;
    pq->pos[a] = j;
}

static void heap_ipq_sift_up(heap_ipq_t *pq, int idx) {
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (pq->keys[pq->heap[idx]] < pq->keys[pq->heap[parent]]) {
            heap_ipq_swap(pq, idx, parent);
            idx = parent;
        } else {
            break;
        }
    }
}

static void heap_ipq_sift_down(heap_ipq_t *pq, int idx) {
    while (1) {
        int smallest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < pq->size && pq->keys[pq->heap[left]] < pq->keys[pq->heap[smallest]])
            smallest = left;
        if (right < pq->size && pq->keys[pq->heap[right]] < pq->keys[pq->heap[smallest]])
            smallest = right;
        if (smallest != idx) {
            heap_ipq_swap(pq, idx, smallest);
            idx = smallest;
        } else {
            break;
        }
    }
}

int heap_ipq_insert(heap_ipq_t *pq, int id, int key) {
    if (pq->size >= HEAP_IPQ_CAP || pq->in_heap[id]) return -1;
    pq->keys[id] = key;
    pq->heap[pq->size] = id;
    pq->pos[id] = pq->size;
    pq->in_heap[id] = 1;
    heap_ipq_sift_up(pq, pq->size);
    pq->size++;
    return 0;
}

int heap_ipq_decrease_key(heap_ipq_t *pq, int id, int new_key) {
    if (!pq->in_heap[id]) return -1;
    if (new_key >= pq->keys[id]) return -2;
    pq->keys[id] = new_key;
    heap_ipq_sift_up(pq, pq->pos[id]);
    return 0;
}

int heap_ipq_extract_min(heap_ipq_t *pq, int *out_id) {
    if (pq->size == 0) return -1;
    *out_id = pq->heap[0];
    pq->in_heap[pq->heap[0]] = 0;
    pq->size--;
    if (pq->size > 0) {
        pq->heap[0] = pq->heap[pq->size];
        pq->pos[pq->heap[0]] = 0;
        heap_ipq_sift_down(pq, 0);
    }
    return 0;
}

int heap_ipq_test(void) {
    heap_ipq_t pq;
    heap_ipq_init(&pq, 10);
    heap_ipq_insert(&pq, 0, 50);
    heap_ipq_insert(&pq, 1, 30);
    heap_ipq_insert(&pq, 2, 70);
    heap_ipq_insert(&pq, 3, 10);
    heap_ipq_decrease_key(&pq, 2, 5);
    int id = -1;
    heap_ipq_extract_min(&pq, &id);
    if (id != 2) return -1;
    heap_ipq_extract_min(&pq, &id);
    if (id != 3) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1236: Indexed priority queue should transpile: {:?}", result.err());
}

/// C1237: Double-ended priority queue (min and max extraction)
#[test]
fn c1237_double_ended_pq() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_DEPQ_CAP 128

typedef struct {
    int data[HEAP_DEPQ_CAP];
    int size;
} heap_depq_t;

void heap_depq_init(heap_depq_t *pq) {
    pq->size = 0;
}

static void heap_depq_swap(int *a, int *b) {
    int tmp = *a;
    *a = *b;
    *b = tmp;
}

static int heap_depq_is_min_level(int idx) {
    int level = 0;
    int pos = idx + 1;
    while (pos > 1) {
        pos = pos / 2;
        level++;
    }
    return (level % 2) == 0;
}

static void heap_depq_push_down_min(heap_depq_t *pq, int idx) {
    while (1) {
        int smallest = idx;
        int c1 = 2 * idx + 1;
        int c2 = 2 * idx + 2;
        int gc[4];
        int i, ng = 0;
        if (c1 < pq->size) {
            if (pq->data[c1] < pq->data[smallest]) smallest = c1;
            gc[ng++] = 2 * c1 + 1;
            gc[ng++] = 2 * c1 + 2;
        }
        if (c2 < pq->size) {
            if (pq->data[c2] < pq->data[smallest]) smallest = c2;
            gc[ng++] = 2 * c2 + 1;
            gc[ng++] = 2 * c2 + 2;
        }
        for (i = 0; i < ng; i++) {
            if (gc[i] < pq->size && pq->data[gc[i]] < pq->data[smallest])
                smallest = gc[i];
        }
        if (smallest == idx) break;
        heap_depq_swap(&pq->data[idx], &pq->data[smallest]);
        if (smallest != c1 && smallest != c2) {
            int par = (smallest - 1) / 2;
            if (pq->data[smallest] > pq->data[par])
                heap_depq_swap(&pq->data[smallest], &pq->data[par]);
            idx = smallest;
        } else {
            break;
        }
    }
}

static void heap_depq_push_down_max(heap_depq_t *pq, int idx) {
    while (1) {
        int largest = idx;
        int c1 = 2 * idx + 1;
        int c2 = 2 * idx + 2;
        int gc[4];
        int i, ng = 0;
        if (c1 < pq->size) {
            if (pq->data[c1] > pq->data[largest]) largest = c1;
            gc[ng++] = 2 * c1 + 1;
            gc[ng++] = 2 * c1 + 2;
        }
        if (c2 < pq->size) {
            if (pq->data[c2] > pq->data[largest]) largest = c2;
            gc[ng++] = 2 * c2 + 1;
            gc[ng++] = 2 * c2 + 2;
        }
        for (i = 0; i < ng; i++) {
            if (gc[i] < pq->size && pq->data[gc[i]] > pq->data[largest])
                largest = gc[i];
        }
        if (largest == idx) break;
        heap_depq_swap(&pq->data[idx], &pq->data[largest]);
        if (largest != c1 && largest != c2) {
            int par = (largest - 1) / 2;
            if (pq->data[largest] < pq->data[par])
                heap_depq_swap(&pq->data[largest], &pq->data[par]);
            idx = largest;
        } else {
            break;
        }
    }
}

static void heap_depq_push_down(heap_depq_t *pq, int idx) {
    if (heap_depq_is_min_level(idx))
        heap_depq_push_down_min(pq, idx);
    else
        heap_depq_push_down_max(pq, idx);
}

int heap_depq_insert(heap_depq_t *pq, int val) {
    if (pq->size >= HEAP_DEPQ_CAP) return -1;
    pq->data[pq->size] = val;
    pq->size++;
    int idx = pq->size - 1;
    if (idx > 0) {
        int parent = (idx - 1) / 2;
        if (heap_depq_is_min_level(idx)) {
            if (pq->data[idx] > pq->data[parent]) {
                heap_depq_swap(&pq->data[idx], &pq->data[parent]);
                idx = parent;
                while (idx > 2) {
                    int gp = ((idx - 1) / 2 - 1) / 2;
                    if (pq->data[idx] > pq->data[gp]) {
                        heap_depq_swap(&pq->data[idx], &pq->data[gp]);
                        idx = gp;
                    } else break;
                }
            } else {
                while (idx > 2) {
                    int gp = ((idx - 1) / 2 - 1) / 2;
                    if (pq->data[idx] < pq->data[gp]) {
                        heap_depq_swap(&pq->data[idx], &pq->data[gp]);
                        idx = gp;
                    } else break;
                }
            }
        } else {
            if (pq->data[idx] < pq->data[parent]) {
                heap_depq_swap(&pq->data[idx], &pq->data[parent]);
                idx = parent;
                while (idx > 2) {
                    int gp = ((idx - 1) / 2 - 1) / 2;
                    if (pq->data[idx] < pq->data[gp]) {
                        heap_depq_swap(&pq->data[idx], &pq->data[gp]);
                        idx = gp;
                    } else break;
                }
            } else {
                while (idx > 2) {
                    int gp = ((idx - 1) / 2 - 1) / 2;
                    if (pq->data[idx] > pq->data[gp]) {
                        heap_depq_swap(&pq->data[idx], &pq->data[gp]);
                        idx = gp;
                    } else break;
                }
            }
        }
    }
    return 0;
}

int heap_depq_get_min(const heap_depq_t *pq) {
    if (pq->size == 0) return -1;
    return pq->data[0];
}

int heap_depq_get_max(const heap_depq_t *pq) {
    if (pq->size == 0) return -1;
    if (pq->size == 1) return pq->data[0];
    if (pq->size == 2) return pq->data[1];
    return pq->data[1] > pq->data[2] ? pq->data[1] : pq->data[2];
}

int heap_depq_test(void) {
    heap_depq_t pq;
    heap_depq_init(&pq);
    heap_depq_insert(&pq, 50);
    heap_depq_insert(&pq, 10);
    heap_depq_insert(&pq, 90);
    heap_depq_insert(&pq, 30);
    heap_depq_insert(&pq, 70);
    if (heap_depq_get_min(&pq) != 10) return -1;
    if (heap_depq_get_max(&pq) != 90) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1237: Double-ended PQ should transpile: {:?}", result.err());
}

/// C1238: Bounded priority queue (fixed capacity, evicts lowest priority)
#[test]
fn c1238_bounded_priority_queue() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_BPQ_CAP 8

typedef struct {
    int keys[HEAP_BPQ_CAP];
    int vals[HEAP_BPQ_CAP];
    int size;
    int capacity;
} heap_bounded_pq_t;

void heap_bpq_init(heap_bounded_pq_t *pq, int cap) {
    pq->size = 0;
    pq->capacity = cap < HEAP_BPQ_CAP ? cap : HEAP_BPQ_CAP;
}

static void heap_bpq_swap_entries(heap_bounded_pq_t *pq, int i, int j) {
    int tk = pq->keys[i]; pq->keys[i] = pq->keys[j]; pq->keys[j] = tk;
    int tv = pq->vals[i]; pq->vals[i] = pq->vals[j]; pq->vals[j] = tv;
}

static void heap_bpq_sift_up(heap_bounded_pq_t *pq, int idx) {
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (pq->keys[idx] < pq->keys[parent]) {
            heap_bpq_swap_entries(pq, idx, parent);
            idx = parent;
        } else {
            break;
        }
    }
}

static void heap_bpq_sift_down(heap_bounded_pq_t *pq, int idx) {
    while (1) {
        int smallest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < pq->size && pq->keys[left] < pq->keys[smallest])
            smallest = left;
        if (right < pq->size && pq->keys[right] < pq->keys[smallest])
            smallest = right;
        if (smallest != idx) {
            heap_bpq_swap_entries(pq, idx, smallest);
            idx = smallest;
        } else {
            break;
        }
    }
}

int heap_bpq_insert(heap_bounded_pq_t *pq, int key, int val) {
    if (pq->size < pq->capacity) {
        pq->keys[pq->size] = key;
        pq->vals[pq->size] = val;
        heap_bpq_sift_up(pq, pq->size);
        pq->size++;
        return 1;
    }
    if (key > pq->keys[0]) {
        pq->keys[0] = key;
        pq->vals[0] = val;
        heap_bpq_sift_down(pq, 0);
        return 1;
    }
    return 0;
}

int heap_bpq_peek_min(const heap_bounded_pq_t *pq, int *key, int *val) {
    if (pq->size == 0) return -1;
    *key = pq->keys[0];
    *val = pq->vals[0];
    return 0;
}

int heap_bpq_test(void) {
    heap_bounded_pq_t pq;
    heap_bpq_init(&pq, 3);
    heap_bpq_insert(&pq, 10, 100);
    heap_bpq_insert(&pq, 50, 500);
    heap_bpq_insert(&pq, 30, 300);
    heap_bpq_insert(&pq, 70, 700);
    heap_bpq_insert(&pq, 20, 200);
    if (pq.size != 3) return -1;
    int k = 0, v = 0;
    heap_bpq_peek_min(&pq, &k, &v);
    if (k < 30) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1238: Bounded priority queue should transpile: {:?}", result.err());
}

/// C1239: Lazy deletion priority queue (mark-and-skip on extract)
#[test]
fn c1239_lazy_deletion_pq() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_LZ_CAP 256

typedef struct {
    int keys[HEAP_LZ_CAP];
    int ids[HEAP_LZ_CAP];
    int size;
    int deleted[HEAP_LZ_CAP];
} heap_lazy_pq_t;

void heap_lz_init(heap_lazy_pq_t *pq) {
    pq->size = 0;
    int i;
    for (i = 0; i < HEAP_LZ_CAP; i++) {
        pq->deleted[i] = 0;
    }
}

static void heap_lz_swap(heap_lazy_pq_t *pq, int i, int j) {
    int tk = pq->keys[i]; pq->keys[i] = pq->keys[j]; pq->keys[j] = tk;
    int ti = pq->ids[i]; pq->ids[i] = pq->ids[j]; pq->ids[j] = ti;
}

static void heap_lz_sift_up(heap_lazy_pq_t *pq, int idx) {
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (pq->keys[idx] < pq->keys[parent]) {
            heap_lz_swap(pq, idx, parent);
            idx = parent;
        } else {
            break;
        }
    }
}

static void heap_lz_sift_down(heap_lazy_pq_t *pq, int idx) {
    while (1) {
        int smallest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < pq->size && pq->keys[left] < pq->keys[smallest])
            smallest = left;
        if (right < pq->size && pq->keys[right] < pq->keys[smallest])
            smallest = right;
        if (smallest != idx) {
            heap_lz_swap(pq, idx, smallest);
            idx = smallest;
        } else {
            break;
        }
    }
}

int heap_lz_insert(heap_lazy_pq_t *pq, int id, int key) {
    if (pq->size >= HEAP_LZ_CAP) return -1;
    pq->keys[pq->size] = key;
    pq->ids[pq->size] = id;
    heap_lz_sift_up(pq, pq->size);
    pq->size++;
    return 0;
}

void heap_lz_mark_deleted(heap_lazy_pq_t *pq, int id) {
    pq->deleted[id] = 1;
}

int heap_lz_extract_min(heap_lazy_pq_t *pq, int *out_id, int *out_key) {
    while (pq->size > 0) {
        int top_id = pq->ids[0];
        int top_key = pq->keys[0];
        pq->size--;
        if (pq->size > 0) {
            pq->keys[0] = pq->keys[pq->size];
            pq->ids[0] = pq->ids[pq->size];
            heap_lz_sift_down(pq, 0);
        }
        if (!pq->deleted[top_id]) {
            *out_id = top_id;
            *out_key = top_key;
            return 0;
        }
    }
    return -1;
}

int heap_lazy_pq_test(void) {
    heap_lazy_pq_t pq;
    heap_lz_init(&pq);
    heap_lz_insert(&pq, 0, 50);
    heap_lz_insert(&pq, 1, 10);
    heap_lz_insert(&pq, 2, 30);
    heap_lz_insert(&pq, 3, 20);
    heap_lz_mark_deleted(&pq, 1);
    int id = -1, key = -1;
    heap_lz_extract_min(&pq, &id, &key);
    if (id != 3 || key != 20) return -1;
    heap_lz_extract_min(&pq, &id, &key);
    if (id != 2 || key != 30) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1239: Lazy deletion PQ should transpile: {:?}", result.err());
}

/// C1240: Median maintenance using two heaps
#[test]
fn c1240_median_maintenance() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_MED_CAP 128

typedef struct {
    int data[HEAP_MED_CAP];
    int size;
} heap_med_half_t;

typedef struct {
    heap_med_half_t max_heap;
    heap_med_half_t min_heap;
} heap_median_t;

void heap_med_init(heap_median_t *m) {
    m->max_heap.size = 0;
    m->min_heap.size = 0;
}

static void heap_med_swap(int *a, int *b) {
    int tmp = *a; *a = *b; *b = tmp;
}

static void heap_med_max_sift_up(heap_med_half_t *h, int idx) {
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (h->data[idx] > h->data[parent]) {
            heap_med_swap(&h->data[idx], &h->data[parent]);
            idx = parent;
        } else break;
    }
}

static void heap_med_max_sift_down(heap_med_half_t *h, int idx) {
    while (1) {
        int largest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < h->size && h->data[left] > h->data[largest]) largest = left;
        if (right < h->size && h->data[right] > h->data[largest]) largest = right;
        if (largest != idx) {
            heap_med_swap(&h->data[idx], &h->data[largest]);
            idx = largest;
        } else break;
    }
}

static void heap_med_min_sift_up(heap_med_half_t *h, int idx) {
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (h->data[idx] < h->data[parent]) {
            heap_med_swap(&h->data[idx], &h->data[parent]);
            idx = parent;
        } else break;
    }
}

static void heap_med_min_sift_down(heap_med_half_t *h, int idx) {
    while (1) {
        int smallest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < h->size && h->data[left] < h->data[smallest]) smallest = left;
        if (right < h->size && h->data[right] < h->data[smallest]) smallest = right;
        if (smallest != idx) {
            heap_med_swap(&h->data[idx], &h->data[smallest]);
            idx = smallest;
        } else break;
    }
}

void heap_med_add(heap_median_t *m, int val) {
    if (m->max_heap.size == 0 || val <= m->max_heap.data[0]) {
        m->max_heap.data[m->max_heap.size] = val;
        heap_med_max_sift_up(&m->max_heap, m->max_heap.size);
        m->max_heap.size++;
    } else {
        m->min_heap.data[m->min_heap.size] = val;
        heap_med_min_sift_up(&m->min_heap, m->min_heap.size);
        m->min_heap.size++;
    }
    if (m->max_heap.size > m->min_heap.size + 1) {
        int top = m->max_heap.data[0];
        m->max_heap.size--;
        m->max_heap.data[0] = m->max_heap.data[m->max_heap.size];
        heap_med_max_sift_down(&m->max_heap, 0);
        m->min_heap.data[m->min_heap.size] = top;
        heap_med_min_sift_up(&m->min_heap, m->min_heap.size);
        m->min_heap.size++;
    } else if (m->min_heap.size > m->max_heap.size) {
        int top = m->min_heap.data[0];
        m->min_heap.size--;
        m->min_heap.data[0] = m->min_heap.data[m->min_heap.size];
        heap_med_min_sift_down(&m->min_heap, 0);
        m->max_heap.data[m->max_heap.size] = top;
        heap_med_max_sift_up(&m->max_heap, m->max_heap.size);
        m->max_heap.size++;
    }
}

int heap_med_get_median(const heap_median_t *m) {
    return m->max_heap.data[0];
}

int heap_median_test(void) {
    heap_median_t m;
    heap_med_init(&m);
    heap_med_add(&m, 5);
    heap_med_add(&m, 15);
    heap_med_add(&m, 1);
    heap_med_add(&m, 3);
    heap_med_add(&m, 8);
    int med = heap_med_get_median(&m);
    if (med != 5) return -1;
    heap_med_add(&m, 2);
    heap_med_add(&m, 7);
    med = heap_med_get_median(&m);
    if (med != 5) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1240: Median maintenance should transpile: {:?}", result.err());
}

// ============================================================================
// C1241-C1245: Heap Applications
// ============================================================================

/// C1241: Huffman tree building via min-heap of frequencies
#[test]
fn c1241_huffman_tree() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_HF_MAX_SYMS 64
#define HEAP_HF_MAX_NODES 128

typedef struct {
    int freq;
    int symbol;
    int left;
    int right;
} heap_hf_node_t;

typedef struct {
    int indices[HEAP_HF_MAX_NODES];
    int size;
} heap_hf_minheap_t;

typedef struct {
    heap_hf_node_t nodes[HEAP_HF_MAX_NODES];
    heap_hf_minheap_t heap;
    int node_count;
} heap_huffman_t;

void heap_hf_init(heap_huffman_t *hf) {
    hf->node_count = 0;
    hf->heap.size = 0;
}

static void heap_hf_swap(int *a, int *b) {
    int tmp = *a; *a = *b; *b = tmp;
}

static void heap_hf_sift_up(heap_huffman_t *hf, int idx) {
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        int pi = hf->heap.indices[parent];
        int ci = hf->heap.indices[idx];
        if (hf->nodes[ci].freq < hf->nodes[pi].freq) {
            heap_hf_swap(&hf->heap.indices[parent], &hf->heap.indices[idx]);
            idx = parent;
        } else {
            break;
        }
    }
}

static void heap_hf_sift_down(heap_huffman_t *hf, int idx) {
    while (1) {
        int smallest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < hf->heap.size &&
            hf->nodes[hf->heap.indices[left]].freq < hf->nodes[hf->heap.indices[smallest]].freq)
            smallest = left;
        if (right < hf->heap.size &&
            hf->nodes[hf->heap.indices[right]].freq < hf->nodes[hf->heap.indices[smallest]].freq)
            smallest = right;
        if (smallest != idx) {
            heap_hf_swap(&hf->heap.indices[idx], &hf->heap.indices[smallest]);
            idx = smallest;
        } else {
            break;
        }
    }
}

static int heap_hf_extract_min(heap_huffman_t *hf) {
    int result = hf->heap.indices[0];
    hf->heap.size--;
    hf->heap.indices[0] = hf->heap.indices[hf->heap.size];
    heap_hf_sift_down(hf, 0);
    return result;
}

static void heap_hf_heap_insert(heap_huffman_t *hf, int node_idx) {
    hf->heap.indices[hf->heap.size] = node_idx;
    heap_hf_sift_up(hf, hf->heap.size);
    hf->heap.size++;
}

int heap_hf_build(heap_huffman_t *hf, const int *freqs, int n) {
    int i;
    for (i = 0; i < n; i++) {
        int idx = hf->node_count++;
        hf->nodes[idx].freq = freqs[i];
        hf->nodes[idx].symbol = i;
        hf->nodes[idx].left = -1;
        hf->nodes[idx].right = -1;
        heap_hf_heap_insert(hf, idx);
    }
    while (hf->heap.size > 1) {
        int a = heap_hf_extract_min(hf);
        int b = heap_hf_extract_min(hf);
        int parent = hf->node_count++;
        hf->nodes[parent].freq = hf->nodes[a].freq + hf->nodes[b].freq;
        hf->nodes[parent].symbol = -1;
        hf->nodes[parent].left = a;
        hf->nodes[parent].right = b;
        heap_hf_heap_insert(hf, parent);
    }
    return hf->heap.indices[0];
}

int heap_hf_depth(const heap_huffman_t *hf, int node, int sym) {
    if (node < 0) return -1;
    if (hf->nodes[node].symbol == sym) return 0;
    int left = heap_hf_depth(hf, hf->nodes[node].left, sym);
    if (left >= 0) return left + 1;
    int right = heap_hf_depth(hf, hf->nodes[node].right, sym);
    if (right >= 0) return right + 1;
    return -1;
}

int heap_huffman_test(void) {
    heap_huffman_t hf;
    heap_hf_init(&hf);
    int freqs[4] = {5, 9, 12, 13};
    int root = heap_hf_build(&hf, freqs, 4);
    if (hf.nodes[root].freq != 39) return -1;
    int d0 = heap_hf_depth(&hf, root, 0);
    int d3 = heap_hf_depth(&hf, root, 3);
    if (d0 < d3) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1241: Huffman tree should transpile: {:?}", result.err());
}

/// C1242: Running median over a sliding window
#[test]
fn c1242_running_median() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_RM_CAP 64

typedef struct {
    int data[HEAP_RM_CAP];
    int size;
} heap_rm_half_t;

static void heap_rm_swap(int *a, int *b) {
    int tmp = *a; *a = *b; *b = tmp;
}

static void heap_rm_max_push(heap_rm_half_t *h, int val) {
    h->data[h->size] = val;
    int idx = h->size;
    h->size++;
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (h->data[idx] > h->data[parent]) {
            heap_rm_swap(&h->data[idx], &h->data[parent]);
            idx = parent;
        } else break;
    }
}

static int heap_rm_max_pop(heap_rm_half_t *h) {
    int val = h->data[0];
    h->size--;
    h->data[0] = h->data[h->size];
    int idx = 0;
    while (1) {
        int largest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < h->size && h->data[left] > h->data[largest]) largest = left;
        if (right < h->size && h->data[right] > h->data[largest]) largest = right;
        if (largest != idx) {
            heap_rm_swap(&h->data[idx], &h->data[largest]);
            idx = largest;
        } else break;
    }
    return val;
}

static void heap_rm_min_push(heap_rm_half_t *h, int val) {
    h->data[h->size] = val;
    int idx = h->size;
    h->size++;
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (h->data[idx] < h->data[parent]) {
            heap_rm_swap(&h->data[idx], &h->data[parent]);
            idx = parent;
        } else break;
    }
}

static void heap_rm_balance(heap_rm_half_t *lo, heap_rm_half_t *hi) {
    while (lo->size > hi->size + 1) {
        int val = heap_rm_max_pop(lo);
        heap_rm_min_push(hi, val);
    }
    while (hi->size > lo->size) {
        int val = hi->data[0];
        hi->size--;
        hi->data[0] = hi->data[hi->size];
        int idx = 0;
        while (1) {
            int smallest = idx;
            int left = 2 * idx + 1;
            int right = 2 * idx + 2;
            if (left < hi->size && hi->data[left] < hi->data[smallest]) smallest = left;
            if (right < hi->size && hi->data[right] < hi->data[smallest]) smallest = right;
            if (smallest != idx) {
                heap_rm_swap(&hi->data[idx], &hi->data[smallest]);
                idx = smallest;
            } else break;
        }
        heap_rm_max_push(lo, val);
    }
}

int heap_rm_compute_medians(const int *stream, int n, int *medians) {
    heap_rm_half_t lo, hi;
    lo.size = 0;
    hi.size = 0;
    int i;
    for (i = 0; i < n; i++) {
        if (lo.size == 0 || stream[i] <= lo.data[0]) {
            heap_rm_max_push(&lo, stream[i]);
        } else {
            heap_rm_min_push(&hi, stream[i]);
        }
        heap_rm_balance(&lo, &hi);
        medians[i] = lo.data[0];
    }
    return n;
}

int heap_running_median_test(void) {
    int stream[7] = {1, 5, 2, 8, 3, 9, 4};
    int medians[7];
    heap_rm_compute_medians(stream, 7, medians);
    if (medians[0] != 1) return -1;
    if (medians[1] != 1) return -2;
    if (medians[2] != 2) return -3;
    if (medians[3] != 2) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1242: Running median should transpile: {:?}", result.err());
}

/// C1243: K closest points to origin using max-heap
#[test]
fn c1243_k_closest_points() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_KP_MAX 64

typedef struct {
    int x;
    int y;
    int dist_sq;
} heap_kp_point_t;

typedef struct {
    heap_kp_point_t data[HEAP_KP_MAX];
    int size;
    int k;
} heap_kp_maxheap_t;

void heap_kp_init(heap_kp_maxheap_t *h, int k) {
    h->size = 0;
    h->k = k < HEAP_KP_MAX ? k : HEAP_KP_MAX;
}

static void heap_kp_swap(heap_kp_point_t *a, heap_kp_point_t *b) {
    heap_kp_point_t tmp = *a;
    *a = *b;
    *b = tmp;
}

static void heap_kp_sift_up(heap_kp_maxheap_t *h, int idx) {
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (h->data[idx].dist_sq > h->data[parent].dist_sq) {
            heap_kp_swap(&h->data[idx], &h->data[parent]);
            idx = parent;
        } else break;
    }
}

static void heap_kp_sift_down(heap_kp_maxheap_t *h, int idx) {
    while (1) {
        int largest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < h->size && h->data[left].dist_sq > h->data[largest].dist_sq)
            largest = left;
        if (right < h->size && h->data[right].dist_sq > h->data[largest].dist_sq)
            largest = right;
        if (largest != idx) {
            heap_kp_swap(&h->data[idx], &h->data[largest]);
            idx = largest;
        } else break;
    }
}

void heap_kp_add(heap_kp_maxheap_t *h, int x, int y) {
    int dsq = x * x + y * y;
    if (h->size < h->k) {
        h->data[h->size].x = x;
        h->data[h->size].y = y;
        h->data[h->size].dist_sq = dsq;
        heap_kp_sift_up(h, h->size);
        h->size++;
    } else if (dsq < h->data[0].dist_sq) {
        h->data[0].x = x;
        h->data[0].y = y;
        h->data[0].dist_sq = dsq;
        heap_kp_sift_down(h, 0);
    }
}

int heap_kp_test(void) {
    heap_kp_maxheap_t h;
    heap_kp_init(&h, 3);
    heap_kp_add(&h, 3, 3);
    heap_kp_add(&h, 1, 1);
    heap_kp_add(&h, 5, 5);
    heap_kp_add(&h, -1, 0);
    heap_kp_add(&h, 2, 2);
    if (h.size != 3) return -1;
    if (h.data[0].dist_sq > 8) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1243: K closest points should transpile: {:?}", result.err());
}

/// C1244: Merge K sorted arrays using min-heap
#[test]
fn c1244_merge_k_sorted_arrays() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_MK_MAX_K 8
#define HEAP_MK_MAX_LEN 32
#define HEAP_MK_OUT_MAX 256

typedef struct {
    int value;
    int arr_idx;
    int elem_idx;
} heap_mk_entry_t;

typedef struct {
    heap_mk_entry_t data[HEAP_MK_MAX_K];
    int size;
} heap_mk_minheap_t;

static void heap_mk_swap(heap_mk_entry_t *a, heap_mk_entry_t *b) {
    heap_mk_entry_t tmp = *a;
    *a = *b;
    *b = tmp;
}

static void heap_mk_sift_up(heap_mk_minheap_t *h, int idx) {
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (h->data[idx].value < h->data[parent].value) {
            heap_mk_swap(&h->data[idx], &h->data[parent]);
            idx = parent;
        } else break;
    }
}

static void heap_mk_sift_down(heap_mk_minheap_t *h, int idx) {
    while (1) {
        int smallest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < h->size && h->data[left].value < h->data[smallest].value)
            smallest = left;
        if (right < h->size && h->data[right].value < h->data[smallest].value)
            smallest = right;
        if (smallest != idx) {
            heap_mk_swap(&h->data[idx], &h->data[smallest]);
            idx = smallest;
        } else break;
    }
}

int heap_mk_merge(int arrs[HEAP_MK_MAX_K][HEAP_MK_MAX_LEN],
                  int lens[HEAP_MK_MAX_K], int k,
                  int *output) {
    heap_mk_minheap_t h;
    h.size = 0;
    int i;
    for (i = 0; i < k; i++) {
        if (lens[i] > 0) {
            h.data[h.size].value = arrs[i][0];
            h.data[h.size].arr_idx = i;
            h.data[h.size].elem_idx = 0;
            h.size++;
            heap_mk_sift_up(&h, h.size - 1);
        }
    }
    int out_count = 0;
    while (h.size > 0 && out_count < HEAP_MK_OUT_MAX) {
        heap_mk_entry_t top = h.data[0];
        output[out_count++] = top.value;
        int ai = top.arr_idx;
        int next_ei = top.elem_idx + 1;
        if (next_ei < lens[ai]) {
            h.data[0].value = arrs[ai][next_ei];
            h.data[0].elem_idx = next_ei;
            heap_mk_sift_down(&h, 0);
        } else {
            h.size--;
            if (h.size > 0) {
                h.data[0] = h.data[h.size];
                heap_mk_sift_down(&h, 0);
            }
        }
    }
    return out_count;
}

int heap_mk_sorted_test(void) {
    int arrs[HEAP_MK_MAX_K][HEAP_MK_MAX_LEN];
    int lens[HEAP_MK_MAX_K];
    arrs[0][0] = 1; arrs[0][1] = 4; arrs[0][2] = 7; lens[0] = 3;
    arrs[1][0] = 2; arrs[1][1] = 5; arrs[1][2] = 8; lens[1] = 3;
    arrs[2][0] = 3; arrs[2][1] = 6; arrs[2][2] = 9; lens[2] = 3;
    int output[256];
    int n = heap_mk_merge(arrs, lens, 3, output);
    if (n != 9) return -1;
    int j;
    for (j = 1; j < n; j++) {
        if (output[j] < output[j - 1]) return -2;
    }
    if (output[0] != 1 || output[8] != 9) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1244: Merge K sorted arrays should transpile: {:?}", result.err());
}

/// C1245: Task scheduler using priority queue (earliest deadline first)
#[test]
fn c1245_task_scheduler() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_TS_MAX_TASKS 128

typedef struct {
    int task_id;
    int deadline;
    int duration;
} heap_ts_task_t;

typedef struct {
    heap_ts_task_t tasks[HEAP_TS_MAX_TASKS];
    int size;
} heap_task_sched_t;

void heap_ts_init(heap_task_sched_t *s) {
    s->size = 0;
}

static void heap_ts_swap(heap_ts_task_t *a, heap_ts_task_t *b) {
    heap_ts_task_t tmp = *a;
    *a = *b;
    *b = tmp;
}

static void heap_ts_sift_up(heap_task_sched_t *s, int idx) {
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (s->tasks[idx].deadline < s->tasks[parent].deadline) {
            heap_ts_swap(&s->tasks[idx], &s->tasks[parent]);
            idx = parent;
        } else break;
    }
}

static void heap_ts_sift_down(heap_task_sched_t *s, int idx) {
    while (1) {
        int smallest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < s->size && s->tasks[left].deadline < s->tasks[smallest].deadline)
            smallest = left;
        if (right < s->size && s->tasks[right].deadline < s->tasks[smallest].deadline)
            smallest = right;
        if (smallest != idx) {
            heap_ts_swap(&s->tasks[idx], &s->tasks[smallest]);
            idx = smallest;
        } else break;
    }
}

int heap_ts_add_task(heap_task_sched_t *s, int id, int deadline, int duration) {
    if (s->size >= HEAP_TS_MAX_TASKS) return -1;
    s->tasks[s->size].task_id = id;
    s->tasks[s->size].deadline = deadline;
    s->tasks[s->size].duration = duration;
    heap_ts_sift_up(s, s->size);
    s->size++;
    return 0;
}

int heap_ts_next_task(heap_task_sched_t *s, heap_ts_task_t *out) {
    if (s->size == 0) return -1;
    *out = s->tasks[0];
    s->size--;
    if (s->size > 0) {
        s->tasks[0] = s->tasks[s->size];
        heap_ts_sift_down(s, 0);
    }
    return 0;
}

int heap_ts_schedule(heap_task_sched_t *s, int *order, int max_out) {
    int count = 0;
    int current_time = 0;
    heap_ts_task_t task;
    while (count < max_out && heap_ts_next_task(s, &task) == 0) {
        order[count] = task.task_id;
        current_time += task.duration;
        count++;
    }
    return count;
}

int heap_task_scheduler_test(void) {
    heap_task_sched_t s;
    heap_ts_init(&s);
    heap_ts_add_task(&s, 0, 100, 10);
    heap_ts_add_task(&s, 1, 50, 5);
    heap_ts_add_task(&s, 2, 200, 20);
    heap_ts_add_task(&s, 3, 30, 3);
    int order[10];
    int n = heap_ts_schedule(&s, order, 10);
    if (n != 4) return -1;
    if (order[0] != 3) return -2;
    if (order[1] != 1) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1245: Task scheduler should transpile: {:?}", result.err());
}

// ============================================================================
// C1246-C1250: Specialized Heap Applications
// ============================================================================

/// C1246: Interval scheduling using min-heap of end times
#[test]
fn c1246_interval_scheduling() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_IS_MAX 128

typedef struct {
    int start;
    int end;
    int id;
} heap_is_interval_t;

static void heap_is_swap_iv(heap_is_interval_t *a, heap_is_interval_t *b) {
    heap_is_interval_t tmp = *a;
    *a = *b;
    *b = tmp;
}

static void heap_is_sort_by_start(heap_is_interval_t *arr, int n) {
    int i, j;
    for (i = 1; i < n; i++) {
        heap_is_interval_t key = arr[i];
        j = i - 1;
        while (j >= 0 && arr[j].start > key.start) {
            arr[j + 1] = arr[j];
            j--;
        }
        arr[j + 1] = key;
    }
}

typedef struct {
    int end_times[HEAP_IS_MAX];
    int size;
} heap_is_endheap_t;

static void heap_is_heap_push(heap_is_endheap_t *h, int end) {
    h->end_times[h->size] = end;
    int idx = h->size;
    h->size++;
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (h->end_times[idx] < h->end_times[parent]) {
            int tmp = h->end_times[idx];
            h->end_times[idx] = h->end_times[parent];
            h->end_times[parent] = tmp;
            idx = parent;
        } else break;
    }
}

static int heap_is_heap_peek(const heap_is_endheap_t *h) {
    if (h->size == 0) return -1;
    return h->end_times[0];
}

static void heap_is_heap_pop(heap_is_endheap_t *h) {
    h->size--;
    h->end_times[0] = h->end_times[h->size];
    int idx = 0;
    while (1) {
        int smallest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < h->size && h->end_times[left] < h->end_times[smallest]) smallest = left;
        if (right < h->size && h->end_times[right] < h->end_times[smallest]) smallest = right;
        if (smallest != idx) {
            int tmp = h->end_times[idx];
            h->end_times[idx] = h->end_times[smallest];
            h->end_times[smallest] = tmp;
            idx = smallest;
        } else break;
    }
}

int heap_is_min_rooms(heap_is_interval_t *intervals, int n) {
    if (n == 0) return 0;
    heap_is_sort_by_start(intervals, n);
    heap_is_endheap_t h;
    h.size = 0;
    heap_is_heap_push(&h, intervals[0].end);
    int i;
    for (i = 1; i < n; i++) {
        if (intervals[i].start >= heap_is_heap_peek(&h)) {
            heap_is_heap_pop(&h);
        }
        heap_is_heap_push(&h, intervals[i].end);
    }
    return h.size;
}

int heap_interval_scheduling_test(void) {
    heap_is_interval_t ivs[5];
    ivs[0].start = 0; ivs[0].end = 30; ivs[0].id = 0;
    ivs[1].start = 5; ivs[1].end = 10; ivs[1].id = 1;
    ivs[2].start = 15; ivs[2].end = 20; ivs[2].id = 2;
    ivs[3].start = 7; ivs[3].end = 25; ivs[3].id = 3;
    ivs[4].start = 20; ivs[4].end = 35; ivs[4].id = 4;
    int rooms = heap_is_min_rooms(ivs, 5);
    if (rooms < 2) return -1;
    if (rooms > 5) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1246: Interval scheduling should transpile: {:?}", result.err());
}

/// C1247: Bandwidth allocation with max-heap of available bandwidth
#[test]
fn c1247_bandwidth_allocation() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_BW_MAX_CHANNELS 64

typedef struct {
    int channel_id;
    int bandwidth;
} heap_bw_channel_t;

typedef struct {
    heap_bw_channel_t channels[HEAP_BW_MAX_CHANNELS];
    int size;
} heap_bw_allocator_t;

void heap_bw_init(heap_bw_allocator_t *alloc) {
    alloc->size = 0;
}

static void heap_bw_swap(heap_bw_channel_t *a, heap_bw_channel_t *b) {
    heap_bw_channel_t tmp = *a;
    *a = *b;
    *b = tmp;
}

static void heap_bw_sift_up(heap_bw_allocator_t *alloc, int idx) {
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (alloc->channels[idx].bandwidth > alloc->channels[parent].bandwidth) {
            heap_bw_swap(&alloc->channels[idx], &alloc->channels[parent]);
            idx = parent;
        } else break;
    }
}

static void heap_bw_sift_down(heap_bw_allocator_t *alloc, int idx) {
    while (1) {
        int largest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < alloc->size && alloc->channels[left].bandwidth > alloc->channels[largest].bandwidth)
            largest = left;
        if (right < alloc->size && alloc->channels[right].bandwidth > alloc->channels[largest].bandwidth)
            largest = right;
        if (largest != idx) {
            heap_bw_swap(&alloc->channels[idx], &alloc->channels[largest]);
            idx = largest;
        } else break;
    }
}

int heap_bw_add_channel(heap_bw_allocator_t *alloc, int id, int bw) {
    if (alloc->size >= HEAP_BW_MAX_CHANNELS) return -1;
    alloc->channels[alloc->size].channel_id = id;
    alloc->channels[alloc->size].bandwidth = bw;
    heap_bw_sift_up(alloc, alloc->size);
    alloc->size++;
    return 0;
}

int heap_bw_allocate(heap_bw_allocator_t *alloc, int requested,
                     int *out_channel_id) {
    if (alloc->size == 0) return -1;
    if (alloc->channels[0].bandwidth < requested) return -2;
    *out_channel_id = alloc->channels[0].channel_id;
    alloc->channels[0].bandwidth -= requested;
    if (alloc->channels[0].bandwidth == 0) {
        alloc->size--;
        if (alloc->size > 0) {
            alloc->channels[0] = alloc->channels[alloc->size];
        }
    }
    heap_bw_sift_down(alloc, 0);
    return 0;
}

int heap_bw_total(const heap_bw_allocator_t *alloc) {
    int total = 0;
    int i;
    for (i = 0; i < alloc->size; i++) {
        total += alloc->channels[i].bandwidth;
    }
    return total;
}

int heap_bandwidth_test(void) {
    heap_bw_allocator_t alloc;
    heap_bw_init(&alloc);
    heap_bw_add_channel(&alloc, 0, 100);
    heap_bw_add_channel(&alloc, 1, 200);
    heap_bw_add_channel(&alloc, 2, 50);
    int ch = -1;
    if (heap_bw_allocate(&alloc, 150, &ch) != 0) return -1;
    if (ch != 1) return -2;
    int total = heap_bw_total(&alloc);
    if (total != 200) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1247: Bandwidth allocation should transpile: {:?}", result.err());
}

/// C1248: Event queue (time-ordered event dispatch)
#[test]
fn c1248_event_queue() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_EQ_MAX 256

typedef struct {
    int timestamp;
    int event_type;
    int payload;
} heap_eq_event_t;

typedef struct {
    heap_eq_event_t events[HEAP_EQ_MAX];
    int size;
    int total_dispatched;
} heap_event_queue_t;

void heap_eq_init(heap_event_queue_t *q) {
    q->size = 0;
    q->total_dispatched = 0;
}

static void heap_eq_swap(heap_eq_event_t *a, heap_eq_event_t *b) {
    heap_eq_event_t tmp = *a;
    *a = *b;
    *b = tmp;
}

static void heap_eq_sift_up(heap_event_queue_t *q, int idx) {
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (q->events[idx].timestamp < q->events[parent].timestamp) {
            heap_eq_swap(&q->events[idx], &q->events[parent]);
            idx = parent;
        } else break;
    }
}

static void heap_eq_sift_down(heap_event_queue_t *q, int idx) {
    while (1) {
        int smallest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < q->size && q->events[left].timestamp < q->events[smallest].timestamp)
            smallest = left;
        if (right < q->size && q->events[right].timestamp < q->events[smallest].timestamp)
            smallest = right;
        if (smallest != idx) {
            heap_eq_swap(&q->events[idx], &q->events[smallest]);
            idx = smallest;
        } else break;
    }
}

int heap_eq_schedule(heap_event_queue_t *q, int timestamp, int etype, int payload) {
    if (q->size >= HEAP_EQ_MAX) return -1;
    q->events[q->size].timestamp = timestamp;
    q->events[q->size].event_type = etype;
    q->events[q->size].payload = payload;
    heap_eq_sift_up(q, q->size);
    q->size++;
    return 0;
}

int heap_eq_dispatch(heap_event_queue_t *q, heap_eq_event_t *out) {
    if (q->size == 0) return -1;
    *out = q->events[0];
    q->size--;
    if (q->size > 0) {
        q->events[0] = q->events[q->size];
        heap_eq_sift_down(q, 0);
    }
    q->total_dispatched++;
    return 0;
}

int heap_eq_dispatch_until(heap_event_queue_t *q, int max_time,
                           int *dispatched_types, int max_out) {
    int count = 0;
    heap_eq_event_t ev;
    while (count < max_out && q->size > 0 &&
           q->events[0].timestamp <= max_time) {
        heap_eq_dispatch(q, &ev);
        dispatched_types[count] = ev.event_type;
        count++;
    }
    return count;
}

int heap_event_queue_test(void) {
    heap_event_queue_t q;
    heap_eq_init(&q);
    heap_eq_schedule(&q, 100, 1, 10);
    heap_eq_schedule(&q, 50, 2, 20);
    heap_eq_schedule(&q, 200, 3, 30);
    heap_eq_schedule(&q, 75, 4, 40);
    int types[10];
    int n = heap_eq_dispatch_until(&q, 150, types, 10);
    if (n != 3) return -1;
    if (types[0] != 2) return -2;
    if (types[1] != 4) return -3;
    if (types[2] != 1) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1248: Event queue should transpile: {:?}", result.err());
}

/// C1249: Dijkstra's shortest path using priority queue
#[test]
fn c1249_dijkstra_pq() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_DJ_MAX_V 64
#define HEAP_DJ_MAX_E 256
#define HEAP_DJ_INF 2147483647

typedef struct {
    int to;
    int weight;
    int next;
} heap_dj_edge_t;

typedef struct {
    heap_dj_edge_t edges[HEAP_DJ_MAX_E];
    int head[HEAP_DJ_MAX_V];
    int edge_count;
    int vertex_count;
} heap_dj_graph_t;

typedef struct {
    int vertex;
    int dist;
} heap_dj_entry_t;

typedef struct {
    heap_dj_entry_t data[HEAP_DJ_MAX_V];
    int size;
} heap_dj_pq_t;

void heap_dj_graph_init(heap_dj_graph_t *g, int n) {
    g->edge_count = 0;
    g->vertex_count = n;
    int i;
    for (i = 0; i < n; i++) {
        g->head[i] = -1;
    }
}

void heap_dj_add_edge(heap_dj_graph_t *g, int from, int to, int weight) {
    int idx = g->edge_count++;
    g->edges[idx].to = to;
    g->edges[idx].weight = weight;
    g->edges[idx].next = g->head[from];
    g->head[from] = idx;
}

static void heap_dj_swap(heap_dj_entry_t *a, heap_dj_entry_t *b) {
    heap_dj_entry_t tmp = *a;
    *a = *b;
    *b = tmp;
}

static void heap_dj_pq_push(heap_dj_pq_t *pq, int vertex, int dist) {
    pq->data[pq->size].vertex = vertex;
    pq->data[pq->size].dist = dist;
    int idx = pq->size;
    pq->size++;
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (pq->data[idx].dist < pq->data[parent].dist) {
            heap_dj_swap(&pq->data[idx], &pq->data[parent]);
            idx = parent;
        } else break;
    }
}

static heap_dj_entry_t heap_dj_pq_pop(heap_dj_pq_t *pq) {
    heap_dj_entry_t top = pq->data[0];
    pq->size--;
    if (pq->size > 0) {
        pq->data[0] = pq->data[pq->size];
        int idx = 0;
        while (1) {
            int smallest = idx;
            int left = 2 * idx + 1;
            int right = 2 * idx + 2;
            if (left < pq->size && pq->data[left].dist < pq->data[smallest].dist)
                smallest = left;
            if (right < pq->size && pq->data[right].dist < pq->data[smallest].dist)
                smallest = right;
            if (smallest != idx) {
                heap_dj_swap(&pq->data[idx], &pq->data[smallest]);
                idx = smallest;
            } else break;
        }
    }
    return top;
}

void heap_dj_shortest_path(const heap_dj_graph_t *g, int src, int *dist) {
    int visited[HEAP_DJ_MAX_V];
    heap_dj_pq_t pq;
    pq.size = 0;
    int i;
    for (i = 0; i < g->vertex_count; i++) {
        dist[i] = HEAP_DJ_INF;
        visited[i] = 0;
    }
    dist[src] = 0;
    heap_dj_pq_push(&pq, src, 0);
    while (pq.size > 0) {
        heap_dj_entry_t top = heap_dj_pq_pop(&pq);
        int u = top.vertex;
        if (visited[u]) continue;
        visited[u] = 1;
        int e = g->head[u];
        while (e >= 0) {
            int v = g->edges[e].to;
            int w = g->edges[e].weight;
            if (dist[u] + w < dist[v]) {
                dist[v] = dist[u] + w;
                heap_dj_pq_push(&pq, v, dist[v]);
            }
            e = g->edges[e].next;
        }
    }
}

int heap_dijkstra_test(void) {
    heap_dj_graph_t g;
    heap_dj_graph_init(&g, 5);
    heap_dj_add_edge(&g, 0, 1, 10);
    heap_dj_add_edge(&g, 0, 3, 5);
    heap_dj_add_edge(&g, 1, 2, 1);
    heap_dj_add_edge(&g, 3, 1, 3);
    heap_dj_add_edge(&g, 3, 2, 9);
    heap_dj_add_edge(&g, 3, 4, 2);
    heap_dj_add_edge(&g, 4, 2, 6);
    int dist[HEAP_DJ_MAX_V];
    heap_dj_shortest_path(&g, 0, dist);
    if (dist[0] != 0) return -1;
    if (dist[1] != 8) return -2;
    if (dist[2] != 9) return -3;
    if (dist[3] != 5) return -4;
    if (dist[4] != 7) return -5;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1249: Dijkstra PQ should transpile: {:?}", result.err());
}

/// C1250: A* open set using priority queue with f-score ordering
#[test]
fn c1250_astar_open_set() {
    let c_code = r#"
typedef unsigned long size_t;

#define HEAP_AS_GRID 8
#define HEAP_AS_MAX_NODES 64
#define HEAP_AS_INF 2147483647

typedef struct {
    int x;
    int y;
    int f_score;
} heap_as_entry_t;

typedef struct {
    heap_as_entry_t data[HEAP_AS_MAX_NODES];
    int size;
} heap_as_openset_t;

static void heap_as_swap(heap_as_entry_t *a, heap_as_entry_t *b) {
    heap_as_entry_t tmp = *a;
    *a = *b;
    *b = tmp;
}

static void heap_as_sift_up(heap_as_openset_t *os, int idx) {
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (os->data[idx].f_score < os->data[parent].f_score) {
            heap_as_swap(&os->data[idx], &os->data[parent]);
            idx = parent;
        } else break;
    }
}

static void heap_as_sift_down(heap_as_openset_t *os, int idx) {
    while (1) {
        int smallest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < os->size && os->data[left].f_score < os->data[smallest].f_score)
            smallest = left;
        if (right < os->size && os->data[right].f_score < os->data[smallest].f_score)
            smallest = right;
        if (smallest != idx) {
            heap_as_swap(&os->data[idx], &os->data[smallest]);
            idx = smallest;
        } else break;
    }
}

void heap_as_push(heap_as_openset_t *os, int x, int y, int f) {
    os->data[os->size].x = x;
    os->data[os->size].y = y;
    os->data[os->size].f_score = f;
    heap_as_sift_up(os, os->size);
    os->size++;
}

heap_as_entry_t heap_as_pop(heap_as_openset_t *os) {
    heap_as_entry_t top = os->data[0];
    os->size--;
    if (os->size > 0) {
        os->data[0] = os->data[os->size];
        heap_as_sift_down(os, 0);
    }
    return top;
}

static int heap_as_heuristic(int x1, int y1, int x2, int y2) {
    int dx = x1 - x2;
    int dy = y1 - y2;
    if (dx < 0) dx = -dx;
    if (dy < 0) dy = -dy;
    return dx + dy;
}

int heap_as_find_path(const int grid[HEAP_AS_GRID][HEAP_AS_GRID],
                      int sx, int sy, int gx, int gy,
                      int *path_length) {
    int g_score[HEAP_AS_GRID][HEAP_AS_GRID];
    int closed[HEAP_AS_GRID][HEAP_AS_GRID];
    int dx[4] = {0, 0, 1, -1};
    int dy[4] = {1, -1, 0, 0};
    int i, j;
    for (i = 0; i < HEAP_AS_GRID; i++) {
        for (j = 0; j < HEAP_AS_GRID; j++) {
            g_score[i][j] = HEAP_AS_INF;
            closed[i][j] = 0;
        }
    }
    heap_as_openset_t os;
    os.size = 0;
    g_score[sx][sy] = 0;
    int h = heap_as_heuristic(sx, sy, gx, gy);
    heap_as_push(&os, sx, sy, h);
    while (os.size > 0) {
        heap_as_entry_t cur = heap_as_pop(&os);
        int cx = cur.x;
        int cy = cur.y;
        if (cx == gx && cy == gy) {
            *path_length = g_score[gx][gy];
            return 0;
        }
        if (closed[cx][cy]) continue;
        closed[cx][cy] = 1;
        int d;
        for (d = 0; d < 4; d++) {
            int nx = cx + dx[d];
            int ny = cy + dy[d];
            if (nx < 0 || nx >= HEAP_AS_GRID || ny < 0 || ny >= HEAP_AS_GRID)
                continue;
            if (grid[nx][ny] == 1 || closed[nx][ny]) continue;
            int tentative_g = g_score[cx][cy] + 1;
            if (tentative_g < g_score[nx][ny]) {
                g_score[nx][ny] = tentative_g;
                int f = tentative_g + heap_as_heuristic(nx, ny, gx, gy);
                heap_as_push(&os, nx, ny, f);
            }
        }
    }
    return -1;
}

int heap_astar_test(void) {
    int grid[HEAP_AS_GRID][HEAP_AS_GRID];
    int i, j;
    for (i = 0; i < HEAP_AS_GRID; i++) {
        for (j = 0; j < HEAP_AS_GRID; j++) {
            grid[i][j] = 0;
        }
    }
    grid[1][1] = 1; grid[1][2] = 1; grid[1][3] = 1;
    grid[2][3] = 1; grid[3][1] = 1;
    int path_len = 0;
    int result = heap_as_find_path(grid, 0, 0, 4, 4, &path_len);
    if (result != 0) return -1;
    if (path_len < 8) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1250: A* open set should transpile: {:?}", result.err());
}
