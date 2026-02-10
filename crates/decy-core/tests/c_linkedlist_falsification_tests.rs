//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1251-C1275: Linked List Implementations -- array-based linked lists
//! exercising index-based node management, pointer-free traversal, and
//! classic linked list algorithms expressed in pure C99.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! All C code is self-contained with NO #include directives.
//! Uses array-based nodes (index-based next/prev) instead of malloc/free.
//!
//! Organization:
//! - C1251-C1255: Singly linked (array-based SLL, insert/delete, reverse, find middle, detect cycle)
//! - C1256-C1260: Doubly linked (array-based DLL, insert/delete both ends, LRU operations, splice, flatten)
//! - C1261-C1265: Circular lists (circular buffer, Josephus problem, round-robin, circular DLL, clock hand)
//! - C1266-C1270: List algorithms (merge two sorted, partition around pivot, remove duplicates, intersection, zip/unzip)
//! - C1271-C1275: Advanced (skip list, XOR linked list sim, unrolled linked list, self-organizing, memory pool freelist)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C1251-C1255: Singly Linked Lists
// ============================================================================

#[test]
fn c1251_sll_array_based_insert_delete() {
    let c_code = r#"
typedef struct {
    int value;
    int next;
} ll_sll_node_t;

typedef struct {
    ll_sll_node_t nodes[256];
    int head;
    int free_head;
    int size;
} ll_sll_t;

void ll_sll_init(ll_sll_t *list) {
    int i;
    list->head = -1;
    list->free_head = 0;
    list->size = 0;
    for (i = 0; i < 255; i++) {
        list->nodes[i].next = i + 1;
    }
    list->nodes[255].next = -1;
}

int ll_sll_alloc(ll_sll_t *list) {
    int idx;
    if (list->free_head == -1) return -1;
    idx = list->free_head;
    list->free_head = list->nodes[idx].next;
    return idx;
}

void ll_sll_free_node(ll_sll_t *list, int idx) {
    list->nodes[idx].next = list->free_head;
    list->free_head = idx;
}

void ll_sll_insert_front(ll_sll_t *list, int value) {
    int idx = ll_sll_alloc(list);
    if (idx == -1) return;
    list->nodes[idx].value = value;
    list->nodes[idx].next = list->head;
    list->head = idx;
    list->size++;
}

int ll_sll_delete_value(ll_sll_t *list, int value) {
    int prev = -1;
    int cur = list->head;
    while (cur != -1) {
        if (list->nodes[cur].value == value) {
            if (prev == -1) {
                list->head = list->nodes[cur].next;
            } else {
                list->nodes[prev].next = list->nodes[cur].next;
            }
            ll_sll_free_node(list, cur);
            list->size--;
            return 1;
        }
        prev = cur;
        cur = list->nodes[cur].next;
    }
    return 0;
}

int ll_sll_search(const ll_sll_t *list, int value) {
    int cur = list->head;
    while (cur != -1) {
        if (list->nodes[cur].value == value) return cur;
        cur = list->nodes[cur].next;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1251: SLL insert/delete - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1251: empty output");
    assert!(code.contains("fn ll_sll_init"), "C1251: Should contain ll_sll_init");
    assert!(code.contains("fn ll_sll_insert_front"), "C1251: Should contain ll_sll_insert_front");
    assert!(code.contains("fn ll_sll_delete_value"), "C1251: Should contain ll_sll_delete_value");
}

#[test]
fn c1252_sll_insert_sorted_and_delete_nth() {
    let c_code = r#"
typedef struct {
    int value;
    int next;
} ll_sorted_node_t;

typedef struct {
    ll_sorted_node_t nodes[128];
    int head;
    int free_head;
    int size;
} ll_sorted_list_t;

void ll_sorted_init(ll_sorted_list_t *list) {
    int i;
    list->head = -1;
    list->free_head = 0;
    list->size = 0;
    for (i = 0; i < 127; i++) {
        list->nodes[i].next = i + 1;
    }
    list->nodes[127].next = -1;
}

int ll_sorted_alloc(ll_sorted_list_t *list) {
    int idx;
    if (list->free_head == -1) return -1;
    idx = list->free_head;
    list->free_head = list->nodes[idx].next;
    return idx;
}

void ll_sorted_insert(ll_sorted_list_t *list, int value) {
    int idx = ll_sorted_alloc(list);
    int prev = -1;
    int cur;
    if (idx == -1) return;
    list->nodes[idx].value = value;
    cur = list->head;
    while (cur != -1 && list->nodes[cur].value < value) {
        prev = cur;
        cur = list->nodes[cur].next;
    }
    list->nodes[idx].next = cur;
    if (prev == -1) {
        list->head = idx;
    } else {
        list->nodes[prev].next = idx;
    }
    list->size++;
}

int ll_sorted_delete_nth(ll_sorted_list_t *list, int n) {
    int prev = -1;
    int cur = list->head;
    int count = 0;
    while (cur != -1 && count < n) {
        prev = cur;
        cur = list->nodes[cur].next;
        count++;
    }
    if (cur == -1) return -1;
    if (prev == -1) {
        list->head = list->nodes[cur].next;
    } else {
        list->nodes[prev].next = list->nodes[cur].next;
    }
    list->nodes[cur].next = list->free_head;
    list->free_head = cur;
    list->size--;
    return list->nodes[cur].value;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1252: SLL sorted insert/delete nth - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1252: empty output");
    assert!(code.contains("fn ll_sorted_insert"), "C1252: Should contain ll_sorted_insert");
    assert!(code.contains("fn ll_sorted_delete_nth"), "C1252: Should contain ll_sorted_delete_nth");
}

#[test]
fn c1253_sll_reverse_iterative() {
    let c_code = r#"
typedef struct {
    int value;
    int next;
} ll_rev_node_t;

typedef struct {
    ll_rev_node_t nodes[128];
    int head;
    int size;
} ll_rev_list_t;

void ll_rev_init(ll_rev_list_t *list) {
    list->head = -1;
    list->size = 0;
}

void ll_rev_push(ll_rev_list_t *list, int idx, int value) {
    list->nodes[idx].value = value;
    list->nodes[idx].next = list->head;
    list->head = idx;
    list->size++;
}

void ll_rev_reverse(ll_rev_list_t *list) {
    int prev = -1;
    int cur = list->head;
    int next_idx;
    while (cur != -1) {
        next_idx = list->nodes[cur].next;
        list->nodes[cur].next = prev;
        prev = cur;
        cur = next_idx;
    }
    list->head = prev;
}

int ll_rev_to_array(const ll_rev_list_t *list, int *out, int max_len) {
    int cur = list->head;
    int count = 0;
    while (cur != -1 && count < max_len) {
        out[count] = list->nodes[cur].value;
        cur = list->nodes[cur].next;
        count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1253: SLL reverse - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1253: empty output");
    assert!(code.contains("fn ll_rev_reverse"), "C1253: Should contain ll_rev_reverse");
    assert!(code.contains("fn ll_rev_to_array"), "C1253: Should contain ll_rev_to_array");
}

#[test]
fn c1254_sll_find_middle_node() {
    let c_code = r#"
typedef struct {
    int value;
    int next;
} ll_mid_node_t;

typedef struct {
    ll_mid_node_t nodes[128];
    int head;
    int size;
} ll_mid_list_t;

void ll_mid_init(ll_mid_list_t *list) {
    list->head = -1;
    list->size = 0;
}

void ll_mid_push(ll_mid_list_t *list, int idx, int value) {
    list->nodes[idx].value = value;
    list->nodes[idx].next = list->head;
    list->head = idx;
    list->size++;
}

int ll_mid_find_middle(const ll_mid_list_t *list) {
    int slow = list->head;
    int fast = list->head;
    if (slow == -1) return -1;
    while (fast != -1 && list->nodes[fast].next != -1) {
        slow = list->nodes[slow].next;
        fast = list->nodes[list->nodes[fast].next].next;
    }
    return slow;
}

int ll_mid_get_value(const ll_mid_list_t *list, int idx) {
    if (idx < 0 || idx >= 128) return -1;
    return list->nodes[idx].value;
}

int ll_mid_length(const ll_mid_list_t *list) {
    int cur = list->head;
    int count = 0;
    while (cur != -1) {
        count++;
        cur = list->nodes[cur].next;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1254: SLL find middle - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1254: empty output");
    assert!(code.contains("fn ll_mid_find_middle"), "C1254: Should contain ll_mid_find_middle");
    assert!(code.contains("fn ll_mid_length"), "C1254: Should contain ll_mid_length");
}

#[test]
fn c1255_sll_detect_cycle_floyd() {
    let c_code = r#"
typedef struct {
    int value;
    int next;
} ll_cyc_node_t;

typedef struct {
    ll_cyc_node_t nodes[64];
    int head;
    int count;
} ll_cyc_list_t;

void ll_cyc_init(ll_cyc_list_t *list) {
    list->head = -1;
    list->count = 0;
}

void ll_cyc_add(ll_cyc_list_t *list, int idx, int value, int next_idx) {
    list->nodes[idx].value = value;
    list->nodes[idx].next = next_idx;
    if (list->count == 0) list->head = idx;
    list->count++;
}

int ll_cyc_has_cycle(const ll_cyc_list_t *list) {
    int slow = list->head;
    int fast = list->head;
    if (slow == -1) return 0;
    while (fast != -1 && list->nodes[fast].next != -1) {
        slow = list->nodes[slow].next;
        fast = list->nodes[list->nodes[fast].next].next;
        if (slow == fast) return 1;
    }
    return 0;
}

int ll_cyc_find_cycle_start(const ll_cyc_list_t *list) {
    int slow = list->head;
    int fast = list->head;
    int found = 0;
    while (fast != -1 && list->nodes[fast].next != -1) {
        slow = list->nodes[slow].next;
        fast = list->nodes[list->nodes[fast].next].next;
        if (slow == fast) { found = 1; break; }
    }
    if (!found) return -1;
    slow = list->head;
    while (slow != fast) {
        slow = list->nodes[slow].next;
        fast = list->nodes[fast].next;
    }
    return slow;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1255: SLL cycle detection (Floyd) - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1255: empty output");
    assert!(code.contains("fn ll_cyc_has_cycle"), "C1255: Should contain ll_cyc_has_cycle");
    assert!(code.contains("fn ll_cyc_find_cycle_start"), "C1255: Should contain ll_cyc_find_cycle_start");
}

// ============================================================================
// C1256-C1260: Doubly Linked Lists
// ============================================================================

#[test]
fn c1256_dll_array_based_insert_delete_both_ends() {
    let c_code = r#"
typedef struct {
    int value;
    int prev;
    int next;
} ll_dll_node_t;

typedef struct {
    ll_dll_node_t nodes[128];
    int head;
    int tail;
    int free_head;
    int size;
} ll_dll_t;

void ll_dll_init(ll_dll_t *list) {
    int i;
    list->head = -1;
    list->tail = -1;
    list->free_head = 0;
    list->size = 0;
    for (i = 0; i < 127; i++) {
        list->nodes[i].next = i + 1;
    }
    list->nodes[127].next = -1;
}

int ll_dll_alloc(ll_dll_t *list) {
    int idx;
    if (list->free_head == -1) return -1;
    idx = list->free_head;
    list->free_head = list->nodes[idx].next;
    return idx;
}

void ll_dll_push_front(ll_dll_t *list, int value) {
    int idx = ll_dll_alloc(list);
    if (idx == -1) return;
    list->nodes[idx].value = value;
    list->nodes[idx].prev = -1;
    list->nodes[idx].next = list->head;
    if (list->head != -1) {
        list->nodes[list->head].prev = idx;
    } else {
        list->tail = idx;
    }
    list->head = idx;
    list->size++;
}

void ll_dll_push_back(ll_dll_t *list, int value) {
    int idx = ll_dll_alloc(list);
    if (idx == -1) return;
    list->nodes[idx].value = value;
    list->nodes[idx].next = -1;
    list->nodes[idx].prev = list->tail;
    if (list->tail != -1) {
        list->nodes[list->tail].next = idx;
    } else {
        list->head = idx;
    }
    list->tail = idx;
    list->size++;
}

int ll_dll_pop_front(ll_dll_t *list) {
    int idx = list->head;
    int val;
    if (idx == -1) return -1;
    val = list->nodes[idx].value;
    list->head = list->nodes[idx].next;
    if (list->head != -1) {
        list->nodes[list->head].prev = -1;
    } else {
        list->tail = -1;
    }
    list->nodes[idx].next = list->free_head;
    list->free_head = idx;
    list->size--;
    return val;
}

int ll_dll_pop_back(ll_dll_t *list) {
    int idx = list->tail;
    int val;
    if (idx == -1) return -1;
    val = list->nodes[idx].value;
    list->tail = list->nodes[idx].prev;
    if (list->tail != -1) {
        list->nodes[list->tail].next = -1;
    } else {
        list->head = -1;
    }
    list->nodes[idx].next = list->free_head;
    list->free_head = idx;
    list->size--;
    return val;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1256: DLL insert/delete both ends - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1256: empty output");
    assert!(code.contains("fn ll_dll_push_front"), "C1256: Should contain ll_dll_push_front");
    assert!(code.contains("fn ll_dll_push_back"), "C1256: Should contain ll_dll_push_back");
    assert!(code.contains("fn ll_dll_pop_front"), "C1256: Should contain ll_dll_pop_front");
    assert!(code.contains("fn ll_dll_pop_back"), "C1256: Should contain ll_dll_pop_back");
}

#[test]
fn c1257_dll_insert_after_and_delete_node() {
    let c_code = r#"
typedef struct {
    int value;
    int prev;
    int next;
} ll_dlm_node_t;

typedef struct {
    ll_dlm_node_t nodes[64];
    int head;
    int tail;
    int free_head;
    int size;
} ll_dlm_t;

void ll_dlm_init(ll_dlm_t *list) {
    int i;
    list->head = -1;
    list->tail = -1;
    list->free_head = 0;
    list->size = 0;
    for (i = 0; i < 63; i++) {
        list->nodes[i].next = i + 1;
    }
    list->nodes[63].next = -1;
}

int ll_dlm_alloc(ll_dlm_t *list) {
    int idx;
    if (list->free_head == -1) return -1;
    idx = list->free_head;
    list->free_head = list->nodes[idx].next;
    return idx;
}

void ll_dlm_free_node(ll_dlm_t *list, int idx) {
    list->nodes[idx].next = list->free_head;
    list->free_head = idx;
}

void ll_dlm_insert_after(ll_dlm_t *list, int after_idx, int value) {
    int idx = ll_dlm_alloc(list);
    int next_idx;
    if (idx == -1) return;
    list->nodes[idx].value = value;
    next_idx = list->nodes[after_idx].next;
    list->nodes[idx].prev = after_idx;
    list->nodes[idx].next = next_idx;
    list->nodes[after_idx].next = idx;
    if (next_idx != -1) {
        list->nodes[next_idx].prev = idx;
    } else {
        list->tail = idx;
    }
    list->size++;
}

void ll_dlm_delete_node(ll_dlm_t *list, int idx) {
    int p = list->nodes[idx].prev;
    int n = list->nodes[idx].next;
    if (p != -1) {
        list->nodes[p].next = n;
    } else {
        list->head = n;
    }
    if (n != -1) {
        list->nodes[n].prev = p;
    } else {
        list->tail = p;
    }
    ll_dlm_free_node(list, idx);
    list->size--;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1257: DLL insert after/delete node - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1257: empty output");
    assert!(code.contains("fn ll_dlm_insert_after"), "C1257: Should contain ll_dlm_insert_after");
    assert!(code.contains("fn ll_dlm_delete_node"), "C1257: Should contain ll_dlm_delete_node");
}

#[test]
fn c1258_dll_lru_move_to_front() {
    let c_code = r#"
typedef struct {
    int key;
    int value;
    int prev;
    int next;
} ll_lru_node_t;

typedef struct {
    ll_lru_node_t nodes[32];
    int head;
    int tail;
    int free_head;
    int size;
    int capacity;
} ll_lru_t;

void ll_lru_init(ll_lru_t *cache, int capacity) {
    int i;
    cache->head = -1;
    cache->tail = -1;
    cache->free_head = 0;
    cache->size = 0;
    cache->capacity = capacity;
    for (i = 0; i < 31; i++) {
        cache->nodes[i].next = i + 1;
    }
    cache->nodes[31].next = -1;
}

void ll_lru_detach(ll_lru_t *cache, int idx) {
    int p = cache->nodes[idx].prev;
    int n = cache->nodes[idx].next;
    if (p != -1) cache->nodes[p].next = n;
    else cache->head = n;
    if (n != -1) cache->nodes[n].prev = p;
    else cache->tail = p;
}

void ll_lru_attach_front(ll_lru_t *cache, int idx) {
    cache->nodes[idx].prev = -1;
    cache->nodes[idx].next = cache->head;
    if (cache->head != -1) {
        cache->nodes[cache->head].prev = idx;
    } else {
        cache->tail = idx;
    }
    cache->head = idx;
}

void ll_lru_move_to_front(ll_lru_t *cache, int idx) {
    ll_lru_detach(cache, idx);
    ll_lru_attach_front(cache, idx);
}

int ll_lru_get(ll_lru_t *cache, int key) {
    int cur = cache->head;
    while (cur != -1) {
        if (cache->nodes[cur].key == key) {
            ll_lru_move_to_front(cache, cur);
            return cache->nodes[cur].value;
        }
        cur = cache->nodes[cur].next;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1258: DLL LRU move-to-front - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1258: empty output");
    assert!(code.contains("fn ll_lru_move_to_front"), "C1258: Should contain ll_lru_move_to_front");
    assert!(code.contains("fn ll_lru_get"), "C1258: Should contain ll_lru_get");
}

#[test]
fn c1259_dll_splice_sublist() {
    let c_code = r#"
typedef struct {
    int value;
    int prev;
    int next;
} ll_spl_node_t;

typedef struct {
    ll_spl_node_t nodes[64];
    int head;
    int tail;
    int size;
} ll_spl_t;

void ll_spl_init(ll_spl_t *list) {
    list->head = -1;
    list->tail = -1;
    list->size = 0;
}

void ll_spl_add(ll_spl_t *list, int idx, int value) {
    list->nodes[idx].value = value;
    list->nodes[idx].next = -1;
    list->nodes[idx].prev = list->tail;
    if (list->tail != -1) {
        list->nodes[list->tail].next = idx;
    } else {
        list->head = idx;
    }
    list->tail = idx;
    list->size++;
}

void ll_spl_splice_after(ll_spl_t *dst, int after_idx,
                         ll_spl_t *src) {
    int dst_next;
    if (src->head == -1) return;
    dst_next = dst->nodes[after_idx].next;
    dst->nodes[after_idx].next = src->head;
    src->nodes[src->head].prev = after_idx;
    if (dst_next != -1) {
        dst->nodes[dst_next].prev = src->tail;
        src->nodes[src->tail].next = dst_next;
    } else {
        dst->tail = src->tail;
    }
    dst->size = dst->size + src->size;
    src->head = -1;
    src->tail = -1;
    src->size = 0;
}

int ll_spl_count(const ll_spl_t *list) {
    int cur = list->head;
    int count = 0;
    while (cur != -1) {
        count++;
        cur = list->nodes[cur].next;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1259: DLL splice sublist - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1259: empty output");
    assert!(code.contains("fn ll_spl_splice_after"), "C1259: Should contain ll_spl_splice_after");
    assert!(code.contains("fn ll_spl_count"), "C1259: Should contain ll_spl_count");
}

#[test]
fn c1260_dll_flatten_multilevel() {
    let c_code = r#"
typedef struct {
    int value;
    int prev;
    int next;
    int child;
} ll_flat_node_t;

typedef struct {
    ll_flat_node_t nodes[64];
    int head;
    int size;
} ll_flat_list_t;

void ll_flat_init(ll_flat_list_t *list) {
    list->head = -1;
    list->size = 0;
}

void ll_flat_set_node(ll_flat_list_t *list, int idx, int value,
                      int prev, int next, int child) {
    list->nodes[idx].value = value;
    list->nodes[idx].prev = prev;
    list->nodes[idx].next = next;
    list->nodes[idx].child = child;
    list->size++;
}

void ll_flat_flatten(ll_flat_list_t *list) {
    int cur = list->head;
    int child;
    int tail_of_child;
    int next_saved;
    while (cur != -1) {
        if (list->nodes[cur].child != -1) {
            child = list->nodes[cur].child;
            next_saved = list->nodes[cur].next;
            list->nodes[cur].next = child;
            list->nodes[child].prev = cur;
            list->nodes[cur].child = -1;
            tail_of_child = child;
            while (list->nodes[tail_of_child].next != -1) {
                tail_of_child = list->nodes[tail_of_child].next;
            }
            list->nodes[tail_of_child].next = next_saved;
            if (next_saved != -1) {
                list->nodes[next_saved].prev = tail_of_child;
            }
        }
        cur = list->nodes[cur].next;
    }
}

int ll_flat_to_array(const ll_flat_list_t *list, int *out, int max_len) {
    int cur = list->head;
    int count = 0;
    while (cur != -1 && count < max_len) {
        out[count] = list->nodes[cur].value;
        cur = list->nodes[cur].next;
        count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1260: DLL flatten multilevel - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1260: empty output");
    assert!(code.contains("fn ll_flat_flatten"), "C1260: Should contain ll_flat_flatten");
    assert!(code.contains("fn ll_flat_to_array"), "C1260: Should contain ll_flat_to_array");
}

// ============================================================================
// C1261-C1265: Circular Lists
// ============================================================================

#[test]
fn c1261_circular_buffer_linked() {
    let c_code = r#"
typedef struct {
    int value;
    int next;
} ll_cbuf_node_t;

typedef struct {
    ll_cbuf_node_t nodes[32];
    int tail;
    int size;
    int capacity;
} ll_cbuf_t;

void ll_cbuf_init(ll_cbuf_t *buf, int capacity) {
    int i;
    buf->tail = -1;
    buf->size = 0;
    buf->capacity = capacity;
    for (i = 0; i < capacity; i++) {
        buf->nodes[i].value = 0;
        buf->nodes[i].next = -1;
    }
}

void ll_cbuf_insert(ll_cbuf_t *buf, int value) {
    int new_idx = buf->size;
    if (buf->size >= buf->capacity) return;
    buf->nodes[new_idx].value = value;
    if (buf->tail == -1) {
        buf->nodes[new_idx].next = new_idx;
    } else {
        buf->nodes[new_idx].next = buf->nodes[buf->tail].next;
        buf->nodes[buf->tail].next = new_idx;
    }
    buf->tail = new_idx;
    buf->size++;
}

int ll_cbuf_traverse(const ll_cbuf_t *buf, int *out, int max_len) {
    int start;
    int cur;
    int count = 0;
    if (buf->tail == -1) return 0;
    start = buf->nodes[buf->tail].next;
    cur = start;
    do {
        if (count >= max_len) break;
        out[count] = buf->nodes[cur].value;
        count++;
        cur = buf->nodes[cur].next;
    } while (cur != start);
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1261: Circular buffer linked - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1261: empty output");
    assert!(code.contains("fn ll_cbuf_insert"), "C1261: Should contain ll_cbuf_insert");
    assert!(code.contains("fn ll_cbuf_traverse"), "C1261: Should contain ll_cbuf_traverse");
}

#[test]
fn c1262_josephus_problem_circular() {
    let c_code = r#"
typedef struct {
    int id;
    int next;
    int alive;
} ll_jos_node_t;

typedef struct {
    ll_jos_node_t nodes[64];
    int head;
    int count;
} ll_jos_t;

void ll_jos_init(ll_jos_t *ring, int n) {
    int i;
    ring->count = n;
    ring->head = 0;
    for (i = 0; i < n; i++) {
        ring->nodes[i].id = i + 1;
        ring->nodes[i].next = (i + 1) % n;
        ring->nodes[i].alive = 1;
    }
}

int ll_jos_next_alive(const ll_jos_t *ring, int from) {
    int cur = ring->nodes[from].next;
    while (!ring->nodes[cur].alive && cur != from) {
        cur = ring->nodes[cur].next;
    }
    return cur;
}

int ll_jos_eliminate(ll_jos_t *ring, int step) {
    int cur = ring->head;
    int prev;
    int i;
    int last_eliminated = -1;
    while (ring->count > 1) {
        for (i = 1; i < step; i++) {
            cur = ll_jos_next_alive(ring, cur);
        }
        last_eliminated = ring->nodes[cur].id;
        ring->nodes[cur].alive = 0;
        ring->count--;
        prev = cur;
        cur = ll_jos_next_alive(ring, cur);
        ring->head = cur;
    }
    return ring->nodes[ring->head].id;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1262: Josephus problem - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1262: empty output");
    assert!(code.contains("fn ll_jos_eliminate"), "C1262: Should contain ll_jos_eliminate");
    assert!(code.contains("fn ll_jos_next_alive"), "C1262: Should contain ll_jos_next_alive");
}

#[test]
fn c1263_round_robin_scheduler() {
    let c_code = r#"
typedef struct {
    int task_id;
    int remaining;
    int next;
    int active;
} ll_rr_node_t;

typedef struct {
    ll_rr_node_t nodes[16];
    int head;
    int count;
    int quantum;
} ll_rr_sched_t;

void ll_rr_init(ll_rr_sched_t *sched, int quantum) {
    sched->head = -1;
    sched->count = 0;
    sched->quantum = quantum;
}

void ll_rr_add_task(ll_rr_sched_t *sched, int task_id, int burst) {
    int idx = sched->count;
    if (idx >= 16) return;
    sched->nodes[idx].task_id = task_id;
    sched->nodes[idx].remaining = burst;
    sched->nodes[idx].active = 1;
    if (sched->count == 0) {
        sched->nodes[idx].next = idx;
        sched->head = idx;
    } else {
        int last = sched->head;
        while (sched->nodes[last].next != sched->head) {
            last = sched->nodes[last].next;
        }
        sched->nodes[last].next = idx;
        sched->nodes[idx].next = sched->head;
    }
    sched->count++;
}

int ll_rr_run_step(ll_rr_sched_t *sched) {
    int cur;
    int executed;
    if (sched->head == -1) return -1;
    cur = sched->head;
    while (!sched->nodes[cur].active) {
        cur = sched->nodes[cur].next;
        if (cur == sched->head) return -1;
    }
    executed = sched->nodes[cur].task_id;
    if (sched->nodes[cur].remaining <= sched->quantum) {
        sched->nodes[cur].remaining = 0;
        sched->nodes[cur].active = 0;
    } else {
        sched->nodes[cur].remaining -= sched->quantum;
    }
    sched->head = sched->nodes[cur].next;
    return executed;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1263: Round-robin scheduler - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1263: empty output");
    assert!(code.contains("fn ll_rr_add_task"), "C1263: Should contain ll_rr_add_task");
    assert!(code.contains("fn ll_rr_run_step"), "C1263: Should contain ll_rr_run_step");
}

#[test]
fn c1264_circular_dll_operations() {
    let c_code = r#"
typedef struct {
    int value;
    int prev;
    int next;
} ll_cdll_node_t;

typedef struct {
    ll_cdll_node_t nodes[32];
    int head;
    int free_head;
    int size;
} ll_cdll_t;

void ll_cdll_init(ll_cdll_t *list) {
    int i;
    list->head = -1;
    list->free_head = 0;
    list->size = 0;
    for (i = 0; i < 31; i++) {
        list->nodes[i].next = i + 1;
    }
    list->nodes[31].next = -1;
}

int ll_cdll_alloc(ll_cdll_t *list) {
    int idx;
    if (list->free_head == -1) return -1;
    idx = list->free_head;
    list->free_head = list->nodes[idx].next;
    return idx;
}

void ll_cdll_insert(ll_cdll_t *list, int value) {
    int idx = ll_cdll_alloc(list);
    if (idx == -1) return;
    list->nodes[idx].value = value;
    if (list->head == -1) {
        list->nodes[idx].next = idx;
        list->nodes[idx].prev = idx;
        list->head = idx;
    } else {
        int tail = list->nodes[list->head].prev;
        list->nodes[idx].next = list->head;
        list->nodes[idx].prev = tail;
        list->nodes[tail].next = idx;
        list->nodes[list->head].prev = idx;
    }
    list->size++;
}

void ll_cdll_remove(ll_cdll_t *list, int idx) {
    int p = list->nodes[idx].prev;
    int n = list->nodes[idx].next;
    if (p == idx) {
        list->head = -1;
    } else {
        list->nodes[p].next = n;
        list->nodes[n].prev = p;
        if (list->head == idx) list->head = n;
    }
    list->nodes[idx].next = list->free_head;
    list->free_head = idx;
    list->size--;
}

int ll_cdll_traverse(const ll_cdll_t *list, int *out, int max_len) {
    int cur;
    int count = 0;
    if (list->head == -1) return 0;
    cur = list->head;
    do {
        if (count >= max_len) break;
        out[count] = list->nodes[cur].value;
        count++;
        cur = list->nodes[cur].next;
    } while (cur != list->head);
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1264: Circular DLL - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1264: empty output");
    assert!(code.contains("fn ll_cdll_insert"), "C1264: Should contain ll_cdll_insert");
    assert!(code.contains("fn ll_cdll_remove"), "C1264: Should contain ll_cdll_remove");
    assert!(code.contains("fn ll_cdll_traverse"), "C1264: Should contain ll_cdll_traverse");
}

#[test]
fn c1265_clock_hand_algorithm() {
    let c_code = r#"
typedef struct {
    int page_id;
    int referenced;
    int next;
} ll_clk_frame_t;

typedef struct {
    ll_clk_frame_t frames[16];
    int hand;
    int count;
    int capacity;
} ll_clk_t;

void ll_clk_init(ll_clk_t *clk, int capacity) {
    clk->hand = 0;
    clk->count = 0;
    clk->capacity = capacity;
}

void ll_clk_load_initial(ll_clk_t *clk, int page_id) {
    int idx = clk->count;
    if (idx >= clk->capacity) return;
    clk->frames[idx].page_id = page_id;
    clk->frames[idx].referenced = 1;
    clk->frames[idx].next = (idx + 1) % clk->capacity;
    clk->count++;
    if (clk->count > 1) {
        clk->frames[idx - 1].next = idx;
    }
    if (clk->count == clk->capacity) {
        clk->frames[idx].next = 0;
    }
}

int ll_clk_find_page(const ll_clk_t *clk, int page_id) {
    int i;
    for (i = 0; i < clk->count; i++) {
        if (clk->frames[i].page_id == page_id) return i;
    }
    return -1;
}

int ll_clk_replace(ll_clk_t *clk, int new_page_id) {
    int evicted;
    int loops = 0;
    while (loops < clk->capacity * 2) {
        if (!clk->frames[clk->hand].referenced) {
            evicted = clk->frames[clk->hand].page_id;
            clk->frames[clk->hand].page_id = new_page_id;
            clk->frames[clk->hand].referenced = 1;
            clk->hand = clk->frames[clk->hand].next;
            return evicted;
        }
        clk->frames[clk->hand].referenced = 0;
        clk->hand = clk->frames[clk->hand].next;
        loops++;
    }
    return -1;
}

int ll_clk_access(ll_clk_t *clk, int page_id) {
    int idx = ll_clk_find_page(clk, page_id);
    if (idx != -1) {
        clk->frames[idx].referenced = 1;
        return 1;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1265: Clock hand algorithm - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1265: empty output");
    assert!(code.contains("fn ll_clk_replace"), "C1265: Should contain ll_clk_replace");
    assert!(code.contains("fn ll_clk_access"), "C1265: Should contain ll_clk_access");
}

// ============================================================================
// C1266-C1270: List Algorithms
// ============================================================================

#[test]
fn c1266_merge_two_sorted_lists() {
    let c_code = r#"
typedef struct {
    int value;
    int next;
} ll_mrg_node_t;

typedef struct {
    ll_mrg_node_t nodes[256];
    int head;
    int size;
} ll_mrg_list_t;

void ll_mrg_init(ll_mrg_list_t *list) {
    list->head = -1;
    list->size = 0;
}

void ll_mrg_append(ll_mrg_list_t *list, int idx, int value) {
    list->nodes[idx].value = value;
    list->nodes[idx].next = -1;
    if (list->head == -1) {
        list->head = idx;
    } else {
        int cur = list->head;
        while (list->nodes[cur].next != -1) {
            cur = list->nodes[cur].next;
        }
        list->nodes[cur].next = idx;
    }
    list->size++;
}

int ll_mrg_merge_sorted(ll_mrg_list_t *out, const ll_mrg_list_t *a,
                        const ll_mrg_list_t *b, int start_idx) {
    int ca = a->head;
    int cb = b->head;
    int idx = start_idx;
    int prev = -1;
    out->head = -1;
    out->size = 0;
    while (ca != -1 && cb != -1) {
        if (a->nodes[ca].value <= b->nodes[cb].value) {
            out->nodes[idx].value = a->nodes[ca].value;
            ca = a->nodes[ca].next;
        } else {
            out->nodes[idx].value = b->nodes[cb].value;
            cb = b->nodes[cb].next;
        }
        out->nodes[idx].next = -1;
        if (prev != -1) out->nodes[prev].next = idx;
        else out->head = idx;
        prev = idx;
        idx++;
        out->size++;
    }
    while (ca != -1) {
        out->nodes[idx].value = a->nodes[ca].value;
        out->nodes[idx].next = -1;
        if (prev != -1) out->nodes[prev].next = idx;
        else out->head = idx;
        prev = idx;
        ca = a->nodes[ca].next;
        idx++;
        out->size++;
    }
    while (cb != -1) {
        out->nodes[idx].value = b->nodes[cb].value;
        out->nodes[idx].next = -1;
        if (prev != -1) out->nodes[prev].next = idx;
        else out->head = idx;
        prev = idx;
        cb = b->nodes[cb].next;
        idx++;
        out->size++;
    }
    return idx;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1266: Merge two sorted lists - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1266: empty output");
    assert!(code.contains("fn ll_mrg_merge_sorted"), "C1266: Should contain ll_mrg_merge_sorted");
}

#[test]
fn c1267_partition_around_pivot() {
    let c_code = r#"
typedef struct {
    int value;
    int next;
} ll_par_node_t;

typedef struct {
    ll_par_node_t nodes[64];
    int head;
    int size;
} ll_par_list_t;

void ll_par_init(ll_par_list_t *list) {
    list->head = -1;
    list->size = 0;
}

void ll_par_push(ll_par_list_t *list, int idx, int value) {
    list->nodes[idx].value = value;
    list->nodes[idx].next = list->head;
    list->head = idx;
    list->size++;
}

void ll_par_partition(ll_par_list_t *list, int pivot) {
    int less_head = -1;
    int less_tail = -1;
    int ge_head = -1;
    int ge_tail = -1;
    int cur = list->head;
    int next_saved;
    while (cur != -1) {
        next_saved = list->nodes[cur].next;
        list->nodes[cur].next = -1;
        if (list->nodes[cur].value < pivot) {
            if (less_tail != -1) {
                list->nodes[less_tail].next = cur;
            } else {
                less_head = cur;
            }
            less_tail = cur;
        } else {
            if (ge_tail != -1) {
                list->nodes[ge_tail].next = cur;
            } else {
                ge_head = cur;
            }
            ge_tail = cur;
        }
        cur = next_saved;
    }
    if (less_tail != -1) {
        list->head = less_head;
        list->nodes[less_tail].next = ge_head;
    } else {
        list->head = ge_head;
    }
}

int ll_par_to_array(const ll_par_list_t *list, int *out, int max_len) {
    int cur = list->head;
    int count = 0;
    while (cur != -1 && count < max_len) {
        out[count] = list->nodes[cur].value;
        cur = list->nodes[cur].next;
        count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1267: Partition around pivot - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1267: empty output");
    assert!(code.contains("fn ll_par_partition"), "C1267: Should contain ll_par_partition");
    assert!(code.contains("fn ll_par_to_array"), "C1267: Should contain ll_par_to_array");
}

#[test]
fn c1268_remove_duplicates_sorted() {
    let c_code = r#"
typedef struct {
    int value;
    int next;
} ll_dup_node_t;

typedef struct {
    ll_dup_node_t nodes[64];
    int head;
    int free_head;
    int size;
} ll_dup_list_t;

void ll_dup_init(ll_dup_list_t *list) {
    int i;
    list->head = -1;
    list->free_head = 0;
    list->size = 0;
    for (i = 0; i < 63; i++) {
        list->nodes[i].next = i + 1;
    }
    list->nodes[63].next = -1;
}

int ll_dup_alloc(ll_dup_list_t *list) {
    int idx;
    if (list->free_head == -1) return -1;
    idx = list->free_head;
    list->free_head = list->nodes[idx].next;
    return idx;
}

void ll_dup_insert_sorted(ll_dup_list_t *list, int value) {
    int idx = ll_dup_alloc(list);
    int prev = -1;
    int cur;
    if (idx == -1) return;
    list->nodes[idx].value = value;
    cur = list->head;
    while (cur != -1 && list->nodes[cur].value < value) {
        prev = cur;
        cur = list->nodes[cur].next;
    }
    list->nodes[idx].next = cur;
    if (prev == -1) {
        list->head = idx;
    } else {
        list->nodes[prev].next = idx;
    }
    list->size++;
}

int ll_dup_remove_duplicates(ll_dup_list_t *list) {
    int cur = list->head;
    int removed = 0;
    int next_idx;
    while (cur != -1 && list->nodes[cur].next != -1) {
        next_idx = list->nodes[cur].next;
        if (list->nodes[cur].value == list->nodes[next_idx].value) {
            list->nodes[cur].next = list->nodes[next_idx].next;
            list->nodes[next_idx].next = list->free_head;
            list->free_head = next_idx;
            list->size--;
            removed++;
        } else {
            cur = list->nodes[cur].next;
        }
    }
    return removed;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1268: Remove duplicates sorted - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1268: empty output");
    assert!(code.contains("fn ll_dup_remove_duplicates"), "C1268: Should contain ll_dup_remove_duplicates");
    assert!(code.contains("fn ll_dup_insert_sorted"), "C1268: Should contain ll_dup_insert_sorted");
}

#[test]
fn c1269_list_intersection() {
    let c_code = r#"
typedef struct {
    int value;
    int next;
} ll_isect_node_t;

typedef struct {
    ll_isect_node_t nodes[128];
    int head;
    int size;
} ll_isect_list_t;

void ll_isect_init(ll_isect_list_t *list) {
    list->head = -1;
    list->size = 0;
}

void ll_isect_append_sorted(ll_isect_list_t *list, int idx, int value) {
    list->nodes[idx].value = value;
    list->nodes[idx].next = -1;
    if (list->head == -1) {
        list->head = idx;
    } else {
        int cur = list->head;
        while (list->nodes[cur].next != -1) {
            cur = list->nodes[cur].next;
        }
        list->nodes[cur].next = idx;
    }
    list->size++;
}

int ll_isect_intersect(ll_isect_list_t *out, const ll_isect_list_t *a,
                       const ll_isect_list_t *b, int start_idx) {
    int ca = a->head;
    int cb = b->head;
    int idx = start_idx;
    int prev = -1;
    out->head = -1;
    out->size = 0;
    while (ca != -1 && cb != -1) {
        if (a->nodes[ca].value == b->nodes[cb].value) {
            out->nodes[idx].value = a->nodes[ca].value;
            out->nodes[idx].next = -1;
            if (prev != -1) out->nodes[prev].next = idx;
            else out->head = idx;
            prev = idx;
            idx++;
            out->size++;
            ca = a->nodes[ca].next;
            cb = b->nodes[cb].next;
        } else if (a->nodes[ca].value < b->nodes[cb].value) {
            ca = a->nodes[ca].next;
        } else {
            cb = b->nodes[cb].next;
        }
    }
    return out->size;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1269: List intersection - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1269: empty output");
    assert!(code.contains("fn ll_isect_intersect"), "C1269: Should contain ll_isect_intersect");
}

#[test]
fn c1270_zip_unzip_lists() {
    let c_code = r#"
typedef struct {
    int value;
    int next;
} ll_zip_node_t;

typedef struct {
    ll_zip_node_t nodes[128];
    int head;
    int size;
} ll_zip_list_t;

void ll_zip_init(ll_zip_list_t *list) {
    list->head = -1;
    list->size = 0;
}

void ll_zip_push_back(ll_zip_list_t *list, int idx, int value) {
    list->nodes[idx].value = value;
    list->nodes[idx].next = -1;
    if (list->head == -1) {
        list->head = idx;
    } else {
        int cur = list->head;
        while (list->nodes[cur].next != -1) {
            cur = list->nodes[cur].next;
        }
        list->nodes[cur].next = idx;
    }
    list->size++;
}

void ll_zip_interleave(ll_zip_list_t *a, ll_zip_list_t *b) {
    int ca = a->head;
    int cb = b->head;
    int an;
    int bn;
    while (ca != -1 && cb != -1) {
        an = a->nodes[ca].next;
        bn = b->nodes[cb].next;
        a->nodes[ca].next = cb;
        if (an != -1) {
            b->nodes[cb].next = an;
        }
        ca = an;
        cb = bn;
    }
    a->size = a->size + b->size;
    b->head = -1;
    b->size = 0;
}

void ll_zip_split_alternate(ll_zip_list_t *src, ll_zip_list_t *even,
                            ll_zip_list_t *odd) {
    int cur = src->head;
    int index = 0;
    int next_saved;
    even->head = -1;
    even->size = 0;
    odd->head = -1;
    odd->size = 0;
    while (cur != -1) {
        next_saved = src->nodes[cur].next;
        src->nodes[cur].next = -1;
        if (index % 2 == 0) {
            if (even->head == -1) {
                even->head = cur;
            } else {
                int t = even->head;
                while (src->nodes[t].next != -1) t = src->nodes[t].next;
                src->nodes[t].next = cur;
            }
            even->size++;
        } else {
            if (odd->head == -1) {
                odd->head = cur;
            } else {
                int t = odd->head;
                while (src->nodes[t].next != -1) t = src->nodes[t].next;
                src->nodes[t].next = cur;
            }
            odd->size++;
        }
        cur = next_saved;
        index++;
    }
    src->head = -1;
    src->size = 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1270: Zip/unzip lists - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1270: empty output");
    assert!(code.contains("fn ll_zip_interleave"), "C1270: Should contain ll_zip_interleave");
    assert!(code.contains("fn ll_zip_split_alternate"), "C1270: Should contain ll_zip_split_alternate");
}

// ============================================================================
// C1271-C1275: Advanced Linked List Structures
// ============================================================================

#[test]
fn c1271_skip_list_array_based() {
    let c_code = r#"
typedef struct {
    int value;
    int next[4];
    int level;
} ll_skip_node_t;

typedef struct {
    ll_skip_node_t nodes[64];
    int head;
    int free_head;
    int max_level;
    int size;
} ll_skip_list_t;

void ll_skip_init(ll_skip_list_t *list) {
    int i;
    int j;
    list->head = 0;
    list->free_head = 1;
    list->max_level = 0;
    list->size = 0;
    for (j = 0; j < 4; j++) {
        list->nodes[0].next[j] = -1;
    }
    list->nodes[0].level = 3;
    list->nodes[0].value = -1;
    for (i = 1; i < 63; i++) {
        list->nodes[i].next[0] = i + 1;
    }
    list->nodes[63].next[0] = -1;
}

int ll_skip_alloc(ll_skip_list_t *list) {
    int idx;
    if (list->free_head == -1) return -1;
    idx = list->free_head;
    list->free_head = list->nodes[idx].next[0];
    return idx;
}

int ll_skip_random_level(int seed) {
    int level = 0;
    int r = seed;
    while ((r & 1) && level < 3) {
        level++;
        r = r >> 1;
    }
    return level;
}

void ll_skip_insert(ll_skip_list_t *list, int value, int seed) {
    int update[4];
    int cur = list->head;
    int idx;
    int new_level;
    int i;
    for (i = list->max_level; i >= 0; i--) {
        while (list->nodes[cur].next[i] != -1 &&
               list->nodes[list->nodes[cur].next[i]].value < value) {
            cur = list->nodes[cur].next[i];
        }
        update[i] = cur;
    }
    idx = ll_skip_alloc(list);
    if (idx == -1) return;
    new_level = ll_skip_random_level(seed);
    if (new_level > list->max_level) {
        for (i = list->max_level + 1; i <= new_level; i++) {
            update[i] = list->head;
        }
        list->max_level = new_level;
    }
    list->nodes[idx].value = value;
    list->nodes[idx].level = new_level;
    for (i = 0; i <= new_level; i++) {
        list->nodes[idx].next[i] = list->nodes[update[i]].next[i];
        list->nodes[update[i]].next[i] = idx;
    }
    list->size++;
}

int ll_skip_search(const ll_skip_list_t *list, int value) {
    int cur = list->head;
    int i;
    for (i = list->max_level; i >= 0; i--) {
        while (list->nodes[cur].next[i] != -1 &&
               list->nodes[list->nodes[cur].next[i]].value < value) {
            cur = list->nodes[cur].next[i];
        }
    }
    cur = list->nodes[cur].next[0];
    if (cur != -1 && list->nodes[cur].value == value) return cur;
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1271: Skip list - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1271: empty output");
    assert!(code.contains("fn ll_skip_insert"), "C1271: Should contain ll_skip_insert");
    assert!(code.contains("fn ll_skip_search"), "C1271: Should contain ll_skip_search");
}

#[test]
fn c1272_xor_linked_list_simulation() {
    let c_code = r#"
typedef struct {
    int value;
    int xor_link;
} ll_xor_node_t;

typedef struct {
    ll_xor_node_t nodes[64];
    int head;
    int tail;
    int free_head;
    int size;
} ll_xor_list_t;

void ll_xor_init(ll_xor_list_t *list) {
    int i;
    list->head = -1;
    list->tail = -1;
    list->free_head = 0;
    list->size = 0;
    for (i = 0; i < 63; i++) {
        list->nodes[i].xor_link = i + 1;
    }
    list->nodes[63].xor_link = -1;
}

int ll_xor_alloc(ll_xor_list_t *list) {
    int idx;
    if (list->free_head == -1) return -1;
    idx = list->free_head;
    list->free_head = list->nodes[idx].xor_link;
    return idx;
}

int ll_xor_encode(int prev, int next) {
    if (prev == -1 && next == -1) return 0;
    if (prev == -1) return next;
    if (next == -1) return prev;
    return prev ^ next;
}

void ll_xor_push_front(ll_xor_list_t *list, int value) {
    int idx = ll_xor_alloc(list);
    if (idx == -1) return;
    list->nodes[idx].value = value;
    if (list->head == -1) {
        list->nodes[idx].xor_link = ll_xor_encode(-1, -1);
        list->head = idx;
        list->tail = idx;
    } else {
        list->nodes[idx].xor_link = ll_xor_encode(-1, list->head);
        int old_link = list->nodes[list->head].xor_link;
        int old_next = old_link ^ 0;
        list->nodes[list->head].xor_link = ll_xor_encode(idx, old_next);
        list->head = idx;
    }
    list->size++;
}

int ll_xor_traverse_forward(const ll_xor_list_t *list, int *out, int max_len) {
    int prev = -1;
    int cur = list->head;
    int count = 0;
    int next;
    while (cur != -1 && count < max_len) {
        out[count] = list->nodes[cur].value;
        count++;
        if (prev == -1) {
            next = list->nodes[cur].xor_link;
        } else {
            next = prev ^ list->nodes[cur].xor_link;
        }
        prev = cur;
        if (next == 0 && cur == list->tail) break;
        cur = next;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1272: XOR linked list simulation - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1272: empty output");
    assert!(code.contains("fn ll_xor_push_front"), "C1272: Should contain ll_xor_push_front");
    assert!(code.contains("fn ll_xor_traverse_forward"), "C1272: Should contain ll_xor_traverse_forward");
}

#[test]
fn c1273_unrolled_linked_list() {
    let c_code = r#"
typedef struct {
    int values[8];
    int count;
    int next;
} ll_unroll_block_t;

typedef struct {
    ll_unroll_block_t blocks[32];
    int head;
    int free_head;
    int total_size;
} ll_unroll_t;

void ll_unroll_init(ll_unroll_t *list) {
    int i;
    list->head = -1;
    list->free_head = 0;
    list->total_size = 0;
    for (i = 0; i < 31; i++) {
        list->blocks[i].next = i + 1;
        list->blocks[i].count = 0;
    }
    list->blocks[31].next = -1;
    list->blocks[31].count = 0;
}

int ll_unroll_alloc_block(ll_unroll_t *list) {
    int idx;
    if (list->free_head == -1) return -1;
    idx = list->free_head;
    list->free_head = list->blocks[idx].next;
    list->blocks[idx].next = -1;
    list->blocks[idx].count = 0;
    return idx;
}

void ll_unroll_insert(ll_unroll_t *list, int value) {
    int blk;
    int new_blk;
    int i;
    if (list->head == -1) {
        blk = ll_unroll_alloc_block(list);
        if (blk == -1) return;
        list->head = blk;
    }
    blk = list->head;
    while (list->blocks[blk].next != -1 && list->blocks[blk].count == 8) {
        blk = list->blocks[blk].next;
    }
    if (list->blocks[blk].count == 8) {
        new_blk = ll_unroll_alloc_block(list);
        if (new_blk == -1) return;
        list->blocks[blk].next = new_blk;
        for (i = 4; i < 8; i++) {
            list->blocks[new_blk].values[i - 4] = list->blocks[blk].values[i];
        }
        list->blocks[new_blk].count = 4;
        list->blocks[blk].count = 4;
        blk = new_blk;
    }
    list->blocks[blk].values[list->blocks[blk].count] = value;
    list->blocks[blk].count++;
    list->total_size++;
}

int ll_unroll_get(const ll_unroll_t *list, int index) {
    int blk = list->head;
    int remaining = index;
    while (blk != -1) {
        if (remaining < list->blocks[blk].count) {
            return list->blocks[blk].values[remaining];
        }
        remaining -= list->blocks[blk].count;
        blk = list->blocks[blk].next;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1273: Unrolled linked list - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1273: empty output");
    assert!(code.contains("fn ll_unroll_insert"), "C1273: Should contain ll_unroll_insert");
    assert!(code.contains("fn ll_unroll_get"), "C1273: Should contain ll_unroll_get");
}

#[test]
fn c1274_self_organizing_move_to_front() {
    let c_code = r#"
typedef struct {
    int key;
    int access_count;
    int next;
} ll_sorg_node_t;

typedef struct {
    ll_sorg_node_t nodes[64];
    int head;
    int free_head;
    int size;
} ll_sorg_list_t;

void ll_sorg_init(ll_sorg_list_t *list) {
    int i;
    list->head = -1;
    list->free_head = 0;
    list->size = 0;
    for (i = 0; i < 63; i++) {
        list->nodes[i].next = i + 1;
    }
    list->nodes[63].next = -1;
}

int ll_sorg_alloc(ll_sorg_list_t *list) {
    int idx;
    if (list->free_head == -1) return -1;
    idx = list->free_head;
    list->free_head = list->nodes[idx].next;
    return idx;
}

void ll_sorg_insert(ll_sorg_list_t *list, int key) {
    int idx = ll_sorg_alloc(list);
    if (idx == -1) return;
    list->nodes[idx].key = key;
    list->nodes[idx].access_count = 0;
    list->nodes[idx].next = list->head;
    list->head = idx;
    list->size++;
}

int ll_sorg_search_mtf(ll_sorg_list_t *list, int key) {
    int prev = -1;
    int cur = list->head;
    while (cur != -1) {
        if (list->nodes[cur].key == key) {
            list->nodes[cur].access_count++;
            if (prev != -1) {
                list->nodes[prev].next = list->nodes[cur].next;
                list->nodes[cur].next = list->head;
                list->head = cur;
            }
            return cur;
        }
        prev = cur;
        cur = list->nodes[cur].next;
    }
    return -1;
}

int ll_sorg_search_transpose(ll_sorg_list_t *list, int key) {
    int pprev = -1;
    int prev = -1;
    int cur = list->head;
    while (cur != -1) {
        if (list->nodes[cur].key == key) {
            list->nodes[cur].access_count++;
            if (prev != -1) {
                list->nodes[prev].next = list->nodes[cur].next;
                list->nodes[cur].next = prev;
                if (pprev != -1) {
                    list->nodes[pprev].next = cur;
                } else {
                    list->head = cur;
                }
            }
            return cur;
        }
        pprev = prev;
        prev = cur;
        cur = list->nodes[cur].next;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1274: Self-organizing list - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1274: empty output");
    assert!(code.contains("fn ll_sorg_search_mtf"), "C1274: Should contain ll_sorg_search_mtf");
    assert!(code.contains("fn ll_sorg_search_transpose"), "C1274: Should contain ll_sorg_search_transpose");
}

#[test]
fn c1275_memory_pool_freelist() {
    let c_code = r#"
typedef struct {
    int next_free;
    int data[4];
    int in_use;
} ll_pool_block_t;

typedef struct {
    ll_pool_block_t blocks[32];
    int free_head;
    int allocated;
    int capacity;
} ll_pool_t;

void ll_pool_init(ll_pool_t *pool, int capacity) {
    int i;
    pool->free_head = 0;
    pool->allocated = 0;
    pool->capacity = capacity;
    for (i = 0; i < capacity - 1; i++) {
        pool->blocks[i].next_free = i + 1;
        pool->blocks[i].in_use = 0;
    }
    pool->blocks[capacity - 1].next_free = -1;
    pool->blocks[capacity - 1].in_use = 0;
}

int ll_pool_alloc(ll_pool_t *pool) {
    int idx;
    if (pool->free_head == -1) return -1;
    idx = pool->free_head;
    pool->free_head = pool->blocks[idx].next_free;
    pool->blocks[idx].in_use = 1;
    pool->blocks[idx].next_free = -1;
    pool->allocated++;
    return idx;
}

void ll_pool_free(ll_pool_t *pool, int idx) {
    if (idx < 0 || idx >= pool->capacity) return;
    if (!pool->blocks[idx].in_use) return;
    pool->blocks[idx].in_use = 0;
    pool->blocks[idx].next_free = pool->free_head;
    pool->free_head = idx;
    pool->allocated--;
}

int ll_pool_available(const ll_pool_t *pool) {
    return pool->capacity - pool->allocated;
}

int ll_pool_is_valid(const ll_pool_t *pool, int idx) {
    if (idx < 0 || idx >= pool->capacity) return 0;
    return pool->blocks[idx].in_use;
}

void ll_pool_set_data(ll_pool_t *pool, int idx, int offset, int value) {
    if (idx < 0 || idx >= pool->capacity) return;
    if (!pool->blocks[idx].in_use) return;
    if (offset < 0 || offset >= 4) return;
    pool->blocks[idx].data[offset] = value;
}

int ll_pool_get_data(const ll_pool_t *pool, int idx, int offset) {
    if (idx < 0 || idx >= pool->capacity) return -1;
    if (!pool->blocks[idx].in_use) return -1;
    if (offset < 0 || offset >= 4) return -1;
    return pool->blocks[idx].data[offset];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1275: Memory pool freelist - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1275: empty output");
    assert!(code.contains("fn ll_pool_alloc"), "C1275: Should contain ll_pool_alloc");
    assert!(code.contains("fn ll_pool_free"), "C1275: Should contain ll_pool_free");
    assert!(code.contains("fn ll_pool_get_data"), "C1275: Should contain ll_pool_get_data");
}
