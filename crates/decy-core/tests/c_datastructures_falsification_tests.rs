//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C501-C525: Classic Data Structure implementations -- the kind of C code found
//! in textbooks, competitive programming, and systems software.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise classic data structure patterns commonly
//! found in CLRS, Sedgewick, and real-world C codebases -- all expressed
//! as valid C99 with array-based representations (no malloc/free).
//!
//! Organization:
//! - C501-C505: Linear structures (singly linked list, doubly linked list, stack, queue, BST)
//! - C506-C510: Balanced trees and heaps (AVL, red-black, min-heap, max-heap, hash map)
//! - C511-C515: Tree variants and graphs (trie, segment tree, Fenwick tree, union-find, graph matrix)
//! - C516-C520: Advanced structures (graph list, deque, treap, B-tree, bloom filter)
//! - C521-C525: Specialized structures (LRU cache, skip list, circular buffer, suffix array, persistent stack)
//!
//! Results: 23 passing, 2 falsified (92.0% pass rate)

// ============================================================================
// C501-C505: Linear Structures
// ============================================================================

#[test]
fn c501_singly_linked_list_array_based() {
    let c_code = r#"
typedef struct {
    int value;
    int next;
} sll_node_t;

typedef struct {
    sll_node_t nodes[256];
    int head;
    int free_head;
    int size;
} sll_t;

void sll_init(sll_t *list) {
    int i;
    list->head = -1;
    list->free_head = 0;
    list->size = 0;
    for (i = 0; i < 255; i++) {
        list->nodes[i].next = i + 1;
    }
    list->nodes[255].next = -1;
}

int sll_alloc_node(sll_t *list) {
    if (list->free_head == -1) return -1;
    int idx = list->free_head;
    list->free_head = list->nodes[idx].next;
    return idx;
}

void sll_insert_front(sll_t *list, int value) {
    int idx = sll_alloc_node(list);
    if (idx == -1) return;
    list->nodes[idx].value = value;
    list->nodes[idx].next = list->head;
    list->head = idx;
    list->size++;
}

int sll_search(const sll_t *list, int value) {
    int cur = list->head;
    while (cur != -1) {
        if (list->nodes[cur].value == value) return cur;
        cur = list->nodes[cur].next;
    }
    return -1;
}

int sll_delete(sll_t *list, int value) {
    int prev = -1;
    int cur = list->head;
    while (cur != -1) {
        if (list->nodes[cur].value == value) {
            if (prev == -1) {
                list->head = list->nodes[cur].next;
            } else {
                list->nodes[prev].next = list->nodes[cur].next;
            }
            list->nodes[cur].next = list->free_head;
            list->free_head = cur;
            list->size--;
            return 1;
        }
        prev = cur;
        cur = list->nodes[cur].next;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C501: Singly linked list - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C501: empty output");
    assert!(code.contains("fn sll_init"), "C501: Should contain sll_init");
    assert!(code.contains("fn sll_insert_front"), "C501: Should contain sll_insert_front");
    assert!(code.contains("fn sll_search"), "C501: Should contain sll_search");
}

#[test]
fn c502_doubly_linked_list_array_based() {
    let c_code = r#"
typedef struct {
    int value;
    int prev;
    int next;
} dll_node_t;

typedef struct {
    dll_node_t nodes[128];
    int head;
    int tail;
    int free_head;
    int size;
} dll_t;

void dll_init(dll_t *list) {
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

void dll_insert_front(dll_t *list, int value) {
    if (list->free_head == -1) return;
    int idx = list->free_head;
    list->free_head = list->nodes[idx].next;
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

void dll_remove(dll_t *list, int idx) {
    int p = list->nodes[idx].prev;
    int n = list->nodes[idx].next;
    if (p != -1) list->nodes[p].next = n;
    else list->head = n;
    if (n != -1) list->nodes[n].prev = p;
    else list->tail = p;
    list->nodes[idx].next = list->free_head;
    list->free_head = idx;
    list->size--;
}

void dll_reverse(dll_t *list) {
    int cur = list->head;
    int tmp;
    while (cur != -1) {
        tmp = list->nodes[cur].next;
        list->nodes[cur].next = list->nodes[cur].prev;
        list->nodes[cur].prev = tmp;
        cur = tmp;
    }
    tmp = list->head;
    list->head = list->tail;
    list->tail = tmp;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C502: Doubly linked list - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C502: empty output");
    assert!(code.contains("fn dll_init"), "C502: Should contain dll_init");
    assert!(code.contains("fn dll_insert_front"), "C502: Should contain dll_insert_front");
    assert!(code.contains("fn dll_reverse"), "C502: Should contain dll_reverse");
}

#[test]
fn c503_stack_array_based() {
    let c_code = r#"
#define STACK_MAX 256

typedef struct {
    int data[STACK_MAX];
    int top;
} stack_t;

void stack_init(stack_t *s) {
    s->top = -1;
}

int stack_is_empty(const stack_t *s) {
    return s->top == -1;
}

int stack_is_full(const stack_t *s) {
    return s->top == STACK_MAX - 1;
}

int stack_push(stack_t *s, int value) {
    if (stack_is_full(s)) return -1;
    s->top++;
    s->data[s->top] = value;
    return 0;
}

int stack_pop(stack_t *s) {
    if (stack_is_empty(s)) return -1;
    int val = s->data[s->top];
    s->top--;
    return val;
}

int stack_peek(const stack_t *s) {
    if (stack_is_empty(s)) return -1;
    return s->data[s->top];
}

int stack_size(const stack_t *s) {
    return s->top + 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C503: Stack - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C503: empty output");
    assert!(code.contains("fn stack_push"), "C503: Should contain stack_push");
    assert!(code.contains("fn stack_pop"), "C503: Should contain stack_pop");
    assert!(code.contains("fn stack_peek"), "C503: Should contain stack_peek");
}

#[test]
fn c504_circular_queue() {
    let c_code = r#"
#define QUEUE_CAP 64

typedef struct {
    int data[QUEUE_CAP];
    int head;
    int tail;
    int count;
} cqueue_t;

void cqueue_init(cqueue_t *q) {
    q->head = 0;
    q->tail = 0;
    q->count = 0;
}

int cqueue_is_empty(const cqueue_t *q) {
    return q->count == 0;
}

int cqueue_is_full(const cqueue_t *q) {
    return q->count == QUEUE_CAP;
}

int cqueue_enqueue(cqueue_t *q, int value) {
    if (cqueue_is_full(q)) return -1;
    q->data[q->tail] = value;
    q->tail = (q->tail + 1) % QUEUE_CAP;
    q->count++;
    return 0;
}

int cqueue_dequeue(cqueue_t *q) {
    if (cqueue_is_empty(q)) return -1;
    int val = q->data[q->head];
    q->head = (q->head + 1) % QUEUE_CAP;
    q->count--;
    return val;
}

int cqueue_front(const cqueue_t *q) {
    if (cqueue_is_empty(q)) return -1;
    return q->data[q->head];
}

int cqueue_back(const cqueue_t *q) {
    if (cqueue_is_empty(q)) return -1;
    int idx = (q->tail - 1 + QUEUE_CAP) % QUEUE_CAP;
    return q->data[idx];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C504: Circular queue - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C504: empty output");
    assert!(code.contains("fn cqueue_enqueue"), "C504: Should contain cqueue_enqueue");
    assert!(code.contains("fn cqueue_dequeue"), "C504: Should contain cqueue_dequeue");
}

#[test]
fn c505_binary_search_tree_array_based() {
    let c_code = r#"
#define BST_MAX 256

typedef struct {
    int key;
    int left;
    int right;
    int used;
} bst_node_t;

typedef struct {
    bst_node_t nodes[BST_MAX];
    int root;
    int next_free;
} bst_t;

void bst_init(bst_t *tree) {
    int i;
    tree->root = -1;
    tree->next_free = 0;
    for (i = 0; i < BST_MAX; i++) {
        tree->nodes[i].used = 0;
        tree->nodes[i].left = -1;
        tree->nodes[i].right = -1;
    }
}

int bst_insert(bst_t *tree, int key) {
    if (tree->next_free >= BST_MAX) return -1;
    int idx = tree->next_free;
    tree->next_free++;
    tree->nodes[idx].key = key;
    tree->nodes[idx].left = -1;
    tree->nodes[idx].right = -1;
    tree->nodes[idx].used = 1;
    if (tree->root == -1) {
        tree->root = idx;
        return idx;
    }
    int cur = tree->root;
    while (1) {
        if (key < tree->nodes[cur].key) {
            if (tree->nodes[cur].left == -1) {
                tree->nodes[cur].left = idx;
                return idx;
            }
            cur = tree->nodes[cur].left;
        } else {
            if (tree->nodes[cur].right == -1) {
                tree->nodes[cur].right = idx;
                return idx;
            }
            cur = tree->nodes[cur].right;
        }
    }
}

int bst_search(const bst_t *tree, int key) {
    int cur = tree->root;
    while (cur != -1) {
        if (key == tree->nodes[cur].key) return cur;
        if (key < tree->nodes[cur].key) cur = tree->nodes[cur].left;
        else cur = tree->nodes[cur].right;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C505: BST - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C505: empty output");
    assert!(code.contains("fn bst_init"), "C505: Should contain bst_init");
    assert!(code.contains("fn bst_insert"), "C505: Should contain bst_insert");
    assert!(code.contains("fn bst_search"), "C505: Should contain bst_search");
}

// ============================================================================
// C506-C510: Balanced Trees and Heaps
// ============================================================================

#[test]
fn c506_avl_tree_rotations() {
    let c_code = r#"
#define AVL_MAX 128

typedef struct {
    int key;
    int left;
    int right;
    int height;
} avl_node_t;

typedef struct {
    avl_node_t nodes[AVL_MAX];
    int root;
    int next_free;
} avl_t;

void avl_init(avl_t *tree) {
    tree->root = -1;
    tree->next_free = 0;
}

int avl_height(const avl_t *tree, int idx) {
    if (idx == -1) return 0;
    return tree->nodes[idx].height;
}

int avl_balance_factor(const avl_t *tree, int idx) {
    if (idx == -1) return 0;
    return avl_height(tree, tree->nodes[idx].left) -
           avl_height(tree, tree->nodes[idx].right);
}

void avl_update_height(avl_t *tree, int idx) {
    int lh = avl_height(tree, tree->nodes[idx].left);
    int rh = avl_height(tree, tree->nodes[idx].right);
    tree->nodes[idx].height = (lh > rh ? lh : rh) + 1;
}

int avl_rotate_right(avl_t *tree, int y) {
    int x = tree->nodes[y].left;
    int t2 = tree->nodes[x].right;
    tree->nodes[x].right = y;
    tree->nodes[y].left = t2;
    avl_update_height(tree, y);
    avl_update_height(tree, x);
    return x;
}

int avl_rotate_left(avl_t *tree, int x) {
    int y = tree->nodes[x].right;
    int t2 = tree->nodes[y].left;
    tree->nodes[y].left = x;
    tree->nodes[x].right = t2;
    avl_update_height(tree, x);
    avl_update_height(tree, y);
    return y;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C506: AVL tree - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C506: empty output");
    assert!(code.contains("fn avl_rotate_right"), "C506: Should contain avl_rotate_right");
    assert!(code.contains("fn avl_rotate_left"), "C506: Should contain avl_rotate_left");
    assert!(code.contains("fn avl_balance_factor"), "C506: Should contain avl_balance_factor");
}

#[test]
fn c507_red_black_tree_insert() {
    let c_code = r#"
#define RB_MAX 128
#define RB_RED 0
#define RB_BLACK 1

typedef struct {
    int key;
    int color;
    int left;
    int right;
    int parent;
} rb_node_t;

typedef struct {
    rb_node_t nodes[RB_MAX];
    int root;
    int nil;
    int next_free;
} rbtree_t;

void rb_init(rbtree_t *tree) {
    tree->nil = 0;
    tree->nodes[0].color = RB_BLACK;
    tree->nodes[0].left = 0;
    tree->nodes[0].right = 0;
    tree->nodes[0].parent = 0;
    tree->nodes[0].key = 0;
    tree->root = 0;
    tree->next_free = 1;
}

int rb_alloc(rbtree_t *tree, int key) {
    if (tree->next_free >= RB_MAX) return tree->nil;
    int idx = tree->next_free;
    tree->next_free++;
    tree->nodes[idx].key = key;
    tree->nodes[idx].color = RB_RED;
    tree->nodes[idx].left = tree->nil;
    tree->nodes[idx].right = tree->nil;
    tree->nodes[idx].parent = tree->nil;
    return idx;
}

void rb_left_rotate(rbtree_t *tree, int x) {
    int y = tree->nodes[x].right;
    tree->nodes[x].right = tree->nodes[y].left;
    if (tree->nodes[y].left != tree->nil)
        tree->nodes[tree->nodes[y].left].parent = x;
    tree->nodes[y].parent = tree->nodes[x].parent;
    if (tree->nodes[x].parent == tree->nil)
        tree->root = y;
    else if (x == tree->nodes[tree->nodes[x].parent].left)
        tree->nodes[tree->nodes[x].parent].left = y;
    else
        tree->nodes[tree->nodes[x].parent].right = y;
    tree->nodes[y].left = x;
    tree->nodes[x].parent = y;
}

int rb_grandparent(const rbtree_t *tree, int n) {
    return tree->nodes[tree->nodes[n].parent].parent;
}

int rb_uncle(const rbtree_t *tree, int n) {
    int gp = rb_grandparent(tree, n);
    if (tree->nodes[n].parent == tree->nodes[gp].left)
        return tree->nodes[gp].right;
    return tree->nodes[gp].left;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C507: Red-black tree - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C507: empty output");
    assert!(code.contains("fn rb_init"), "C507: Should contain rb_init");
    assert!(code.contains("fn rb_left_rotate"), "C507: Should contain rb_left_rotate");
}

#[test]
fn c508_min_heap_priority_queue() {
    let c_code = r#"
#define HEAP_MAX 256

typedef struct {
    int data[HEAP_MAX];
    int size;
} min_heap_t;

void heap_init(min_heap_t *h) {
    h->size = 0;
}

void heap_swap(int *a, int *b) {
    int tmp = *a;
    *a = *b;
    *b = tmp;
}

void heap_sift_up(min_heap_t *h, int idx) {
    while (idx > 0) {
        int parent = (idx - 1) / 2;
        if (h->data[parent] > h->data[idx]) {
            heap_swap(&h->data[parent], &h->data[idx]);
            idx = parent;
        } else {
            break;
        }
    }
}

void heap_sift_down(min_heap_t *h, int idx) {
    while (1) {
        int smallest = idx;
        int left = 2 * idx + 1;
        int right = 2 * idx + 2;
        if (left < h->size && h->data[left] < h->data[smallest])
            smallest = left;
        if (right < h->size && h->data[right] < h->data[smallest])
            smallest = right;
        if (smallest != idx) {
            heap_swap(&h->data[idx], &h->data[smallest]);
            idx = smallest;
        } else {
            break;
        }
    }
}

int heap_push(min_heap_t *h, int value) {
    if (h->size >= HEAP_MAX) return -1;
    h->data[h->size] = value;
    heap_sift_up(h, h->size);
    h->size++;
    return 0;
}

int heap_pop(min_heap_t *h) {
    if (h->size == 0) return -1;
    int val = h->data[0];
    h->size--;
    h->data[0] = h->data[h->size];
    heap_sift_down(h, 0);
    return val;
}

int heap_peek(const min_heap_t *h) {
    if (h->size == 0) return -1;
    return h->data[0];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C508: Min-heap - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C508: empty output");
    assert!(code.contains("fn heap_push"), "C508: Should contain heap_push");
    assert!(code.contains("fn heap_pop"), "C508: Should contain heap_pop");
    assert!(code.contains("fn heap_sift_down"), "C508: Should contain heap_sift_down");
}

#[test]
fn c509_max_heap_heapsort() {
    let c_code = r#"
void hsort_swap(int *a, int *b) {
    int tmp = *a;
    *a = *b;
    *b = tmp;
}

void hsort_heapify(int *arr, int n, int i) {
    int largest = i;
    int left = 2 * i + 1;
    int right = 2 * i + 2;
    if (left < n && arr[left] > arr[largest])
        largest = left;
    if (right < n && arr[right] > arr[largest])
        largest = right;
    if (largest != i) {
        hsort_swap(&arr[i], &arr[largest]);
        hsort_heapify(arr, n, largest);
    }
}

void hsort_build_heap(int *arr, int n) {
    int i;
    for (i = n / 2 - 1; i >= 0; i--) {
        hsort_heapify(arr, n, i);
    }
}

void heapsort(int *arr, int n) {
    int i;
    hsort_build_heap(arr, n);
    for (i = n - 1; i > 0; i--) {
        hsort_swap(&arr[0], &arr[i]);
        hsort_heapify(arr, i, 0);
    }
}

int hsort_is_sorted(const int *arr, int n) {
    int i;
    for (i = 0; i < n - 1; i++) {
        if (arr[i] > arr[i + 1]) return 0;
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C509: Heapsort - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C509: empty output");
    assert!(code.contains("fn heapsort"), "C509: Should contain heapsort");
    assert!(code.contains("fn hsort_heapify"), "C509: Should contain hsort_heapify");
}

#[test]
fn c510_hash_map_open_addressing() {
    let c_code = r#"
#define HM_CAP 64
#define HM_EMPTY -1
#define HM_DELETED -2

typedef struct {
    int keys[HM_CAP];
    int values[HM_CAP];
    int states[HM_CAP];
    int count;
} hashmap_t;

void hm_init(hashmap_t *map) {
    int i;
    map->count = 0;
    for (i = 0; i < HM_CAP; i++) {
        map->states[i] = HM_EMPTY;
    }
}

int hm_hash(int key) {
    unsigned int k = (unsigned int)key;
    k = ((k >> 16) ^ k) * 0x45d9f3b;
    k = ((k >> 16) ^ k) * 0x45d9f3b;
    k = (k >> 16) ^ k;
    return (int)(k % HM_CAP);
}

int hm_put(hashmap_t *map, int key, int value) {
    if (map->count >= HM_CAP / 2) return -1;
    int idx = hm_hash(key);
    int dist = 0;
    while (map->states[idx] != HM_EMPTY && map->states[idx] != HM_DELETED) {
        if (map->keys[idx] == key) {
            map->values[idx] = value;
            return 0;
        }
        idx = (idx + 1) % HM_CAP;
        dist++;
        if (dist >= HM_CAP) return -1;
    }
    map->keys[idx] = key;
    map->values[idx] = value;
    map->states[idx] = 1;
    map->count++;
    return 0;
}

int hm_get(const hashmap_t *map, int key, int *out_value) {
    int idx = hm_hash(key);
    int dist = 0;
    while (map->states[idx] != HM_EMPTY) {
        if (map->states[idx] != HM_DELETED && map->keys[idx] == key) {
            *out_value = map->values[idx];
            return 1;
        }
        idx = (idx + 1) % HM_CAP;
        dist++;
        if (dist >= HM_CAP) return 0;
    }
    return 0;
}

int hm_remove(hashmap_t *map, int key) {
    int idx = hm_hash(key);
    int dist = 0;
    while (map->states[idx] != HM_EMPTY) {
        if (map->states[idx] != HM_DELETED && map->keys[idx] == key) {
            map->states[idx] = HM_DELETED;
            map->count--;
            return 1;
        }
        idx = (idx + 1) % HM_CAP;
        dist++;
        if (dist >= HM_CAP) return 0;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C510: Hash map - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C510: empty output");
    assert!(code.contains("fn hm_init"), "C510: Should contain hm_init");
    assert!(code.contains("fn hm_put"), "C510: Should contain hm_put");
    assert!(code.contains("fn hm_get"), "C510: Should contain hm_get");
}

// ============================================================================
// C511-C515: Tree Variants and Graphs
// ============================================================================

#[test]
fn c511_trie_prefix_tree() {
    let c_code = r#"
#define TRIE_ALPHA 26
#define TRIE_MAX 256

typedef struct {
    int children[TRIE_ALPHA];
    int is_end;
    int count;
} trie_node_t;

typedef struct {
    trie_node_t nodes[TRIE_MAX];
    int next_free;
} trie_t;

void trie_init(trie_t *t) {
    int i, j;
    t->next_free = 1;
    for (j = 0; j < TRIE_ALPHA; j++) {
        t->nodes[0].children[j] = -1;
    }
    t->nodes[0].is_end = 0;
    t->nodes[0].count = 0;
}

int trie_alloc(trie_t *t) {
    if (t->next_free >= TRIE_MAX) return -1;
    int idx = t->next_free;
    t->next_free++;
    int j;
    for (j = 0; j < TRIE_ALPHA; j++) {
        t->nodes[idx].children[j] = -1;
    }
    t->nodes[idx].is_end = 0;
    t->nodes[idx].count = 0;
    return idx;
}

int trie_insert(trie_t *t, const char *word, int len) {
    int cur = 0;
    int i;
    for (i = 0; i < len; i++) {
        int c = word[i] - 'a';
        if (c < 0 || c >= TRIE_ALPHA) return -1;
        if (t->nodes[cur].children[c] == -1) {
            int node = trie_alloc(t);
            if (node == -1) return -1;
            t->nodes[cur].children[c] = node;
        }
        cur = t->nodes[cur].children[c];
        t->nodes[cur].count++;
    }
    t->nodes[cur].is_end = 1;
    return 0;
}

int trie_search(const trie_t *t, const char *word, int len) {
    int cur = 0;
    int i;
    for (i = 0; i < len; i++) {
        int c = word[i] - 'a';
        if (c < 0 || c >= TRIE_ALPHA) return 0;
        if (t->nodes[cur].children[c] == -1) return 0;
        cur = t->nodes[cur].children[c];
    }
    return t->nodes[cur].is_end;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C511: Trie - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C511: empty output");
    assert!(code.contains("fn trie_init"), "C511: Should contain trie_init");
    assert!(code.contains("fn trie_insert"), "C511: Should contain trie_insert");
    assert!(code.contains("fn trie_search"), "C511: Should contain trie_search");
}

#[test]
fn c512_segment_tree_range_query() {
    let c_code = r#"
#define SEG_MAX 256
#define SEG_TREE_SIZE 1024

typedef struct {
    int tree[SEG_TREE_SIZE];
    int n;
} segtree_t;

void seg_build(segtree_t *st, const int *arr, int node, int start, int end) {
    if (start == end) {
        st->tree[node] = arr[start];
        return;
    }
    int mid = (start + end) / 2;
    seg_build(st, arr, 2 * node, start, mid);
    seg_build(st, arr, 2 * node + 1, mid + 1, end);
    st->tree[node] = st->tree[2 * node] + st->tree[2 * node + 1];
}

void seg_init(segtree_t *st, const int *arr, int n) {
    st->n = n;
    seg_build(st, arr, 1, 0, n - 1);
}

int seg_query(const segtree_t *st, int node, int start, int end, int l, int r) {
    if (r < start || end < l) return 0;
    if (l <= start && end <= r) return st->tree[node];
    int mid = (start + end) / 2;
    int left_sum = seg_query(st, 2 * node, start, mid, l, r);
    int right_sum = seg_query(st, 2 * node + 1, mid + 1, end, l, r);
    return left_sum + right_sum;
}

void seg_update(segtree_t *st, int node, int start, int end, int idx, int val) {
    if (start == end) {
        st->tree[node] = val;
        return;
    }
    int mid = (start + end) / 2;
    if (idx <= mid)
        seg_update(st, 2 * node, start, mid, idx, val);
    else
        seg_update(st, 2 * node + 1, mid + 1, end, idx, val);
    st->tree[node] = st->tree[2 * node] + st->tree[2 * node + 1];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C512: Segment tree - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C512: empty output");
    assert!(code.contains("fn seg_init"), "C512: Should contain seg_init");
    assert!(code.contains("fn seg_query"), "C512: Should contain seg_query");
    assert!(code.contains("fn seg_update"), "C512: Should contain seg_update");
}

#[test]
fn c513_fenwick_tree_prefix_sums() {
    let c_code = r#"
#define BIT_MAX 256

typedef struct {
    int tree[BIT_MAX + 1];
    int n;
} fenwick_t;

void bit_init(fenwick_t *ft, int n) {
    int i;
    ft->n = n;
    for (i = 0; i <= n; i++) {
        ft->tree[i] = 0;
    }
}

void bit_update(fenwick_t *ft, int idx, int delta) {
    idx++;
    while (idx <= ft->n) {
        ft->tree[idx] += delta;
        idx += idx & (-idx);
    }
}

int bit_prefix_sum(const fenwick_t *ft, int idx) {
    int sum = 0;
    idx++;
    while (idx > 0) {
        sum += ft->tree[idx];
        idx -= idx & (-idx);
    }
    return sum;
}

int bit_range_sum(const fenwick_t *ft, int l, int r) {
    if (l == 0) return bit_prefix_sum(ft, r);
    return bit_prefix_sum(ft, r) - bit_prefix_sum(ft, l - 1);
}

void bit_build(fenwick_t *ft, const int *arr, int n) {
    int i;
    bit_init(ft, n);
    for (i = 0; i < n; i++) {
        bit_update(ft, i, arr[i]);
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C513: Fenwick tree - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C513: empty output");
    assert!(code.contains("fn bit_update"), "C513: Should contain bit_update");
    assert!(code.contains("fn bit_prefix_sum"), "C513: Should contain bit_prefix_sum");
    assert!(code.contains("fn bit_range_sum"), "C513: Should contain bit_range_sum");
}

#[test]
fn c514_disjoint_set_union_find() {
    let c_code = r#"
#define DSU_MAX 256

typedef struct {
    int parent[DSU_MAX];
    int rank[DSU_MAX];
    int size[DSU_MAX];
    int num_sets;
} dsu_t;

void dsu_init(dsu_t *dsu, int n) {
    int i;
    dsu->num_sets = n;
    for (i = 0; i < n; i++) {
        dsu->parent[i] = i;
        dsu->rank[i] = 0;
        dsu->size[i] = 1;
    }
}

int dsu_find(dsu_t *dsu, int x) {
    while (dsu->parent[x] != x) {
        dsu->parent[x] = dsu->parent[dsu->parent[x]];
        x = dsu->parent[x];
    }
    return x;
}

int dsu_union(dsu_t *dsu, int x, int y) {
    int rx = dsu_find(dsu, x);
    int ry = dsu_find(dsu, y);
    if (rx == ry) return 0;
    if (dsu->rank[rx] < dsu->rank[ry]) {
        int tmp = rx; rx = ry; ry = tmp;
    }
    dsu->parent[ry] = rx;
    dsu->size[rx] += dsu->size[ry];
    if (dsu->rank[rx] == dsu->rank[ry]) {
        dsu->rank[rx]++;
    }
    dsu->num_sets--;
    return 1;
}

int dsu_connected(dsu_t *dsu, int x, int y) {
    return dsu_find(dsu, x) == dsu_find(dsu, y);
}

int dsu_set_size(dsu_t *dsu, int x) {
    return dsu->size[dsu_find(dsu, x)];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C514: Union-Find - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C514: empty output");
    assert!(code.contains("fn dsu_init"), "C514: Should contain dsu_init");
    assert!(code.contains("fn dsu_find"), "C514: Should contain dsu_find");
    assert!(code.contains("fn dsu_union"), "C514: Should contain dsu_union");
}

#[test]
fn c515_graph_adjacency_matrix_bfs_dfs() {
    let c_code = r#"
#define GRAPH_MAX 32

typedef struct {
    int adj[GRAPH_MAX][GRAPH_MAX];
    int n;
} graph_t;

void graph_init(graph_t *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            g->adj[i][j] = 0;
        }
    }
}

void graph_add_edge(graph_t *g, int u, int v) {
    g->adj[u][v] = 1;
    g->adj[v][u] = 1;
}

void graph_bfs(const graph_t *g, int start, int *visited, int *order, int *count) {
    int queue[GRAPH_MAX];
    int head = 0;
    int tail = 0;
    int i;
    *count = 0;
    for (i = 0; i < g->n; i++) visited[i] = 0;
    visited[start] = 1;
    queue[tail] = start;
    tail++;
    while (head < tail) {
        int cur = queue[head];
        head++;
        order[*count] = cur;
        (*count)++;
        for (i = 0; i < g->n; i++) {
            if (g->adj[cur][i] && !visited[i]) {
                visited[i] = 1;
                queue[tail] = i;
                tail++;
            }
        }
    }
}

void graph_dfs_helper(const graph_t *g, int v, int *visited, int *order, int *count) {
    visited[v] = 1;
    order[*count] = v;
    (*count)++;
    int i;
    for (i = 0; i < g->n; i++) {
        if (g->adj[v][i] && !visited[i]) {
            graph_dfs_helper(g, i, visited, order, count);
        }
    }
}

void graph_dfs(const graph_t *g, int start, int *visited, int *order, int *count) {
    int i;
    *count = 0;
    for (i = 0; i < g->n; i++) visited[i] = 0;
    graph_dfs_helper(g, start, visited, order, count);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C515: Graph BFS/DFS - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C515: empty output");
    assert!(code.contains("fn graph_init"), "C515: Should contain graph_init");
    assert!(code.contains("fn graph_bfs"), "C515: Should contain graph_bfs");
    assert!(code.contains("fn graph_dfs"), "C515: Should contain graph_dfs");
}

// ============================================================================
// C516-C520: Advanced Structures
// ============================================================================

#[test]
fn c516_graph_adjacency_list_topological_sort() {
    let c_code = r#"
#define GLIST_MAX 32
#define GLIST_EDGES 128

typedef struct {
    int dest[GLIST_EDGES];
    int next[GLIST_EDGES];
    int head[GLIST_MAX];
    int in_degree[GLIST_MAX];
    int n;
    int edge_count;
} glist_t;

void glist_init(glist_t *g, int n) {
    int i;
    g->n = n;
    g->edge_count = 0;
    for (i = 0; i < n; i++) {
        g->head[i] = -1;
        g->in_degree[i] = 0;
    }
}

void glist_add_edge(glist_t *g, int u, int v) {
    int e = g->edge_count;
    g->dest[e] = v;
    g->next[e] = g->head[u];
    g->head[u] = e;
    g->in_degree[v]++;
    g->edge_count++;
}

int glist_topo_sort(glist_t *g, int *result) {
    int queue[GLIST_MAX];
    int head = 0;
    int tail = 0;
    int count = 0;
    int i, e;
    int deg[GLIST_MAX];
    for (i = 0; i < g->n; i++) {
        deg[i] = g->in_degree[i];
        if (deg[i] == 0) {
            queue[tail] = i;
            tail++;
        }
    }
    while (head < tail) {
        int u = queue[head];
        head++;
        result[count] = u;
        count++;
        e = g->head[u];
        while (e != -1) {
            int v = g->dest[e];
            deg[v]--;
            if (deg[v] == 0) {
                queue[tail] = v;
                tail++;
            }
            e = g->next[e];
        }
    }
    return count == g->n ? 1 : 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C516: Graph topological sort - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C516: empty output");
    assert!(code.contains("fn glist_init"), "C516: Should contain glist_init");
    assert!(code.contains("fn glist_topo_sort"), "C516: Should contain glist_topo_sort");
}

#[test]
fn c517_deque_double_ended_queue() {
    let c_code = r#"
#define DEQUE_CAP 128

typedef struct {
    int data[DEQUE_CAP];
    int head;
    int tail;
    int count;
} deque_t;

void deque_init(deque_t *dq) {
    dq->head = 0;
    dq->tail = 0;
    dq->count = 0;
}

int deque_is_empty(const deque_t *dq) {
    return dq->count == 0;
}

int deque_is_full(const deque_t *dq) {
    return dq->count == DEQUE_CAP;
}

int deque_push_front(deque_t *dq, int value) {
    if (deque_is_full(dq)) return -1;
    dq->head = (dq->head - 1 + DEQUE_CAP) % DEQUE_CAP;
    dq->data[dq->head] = value;
    dq->count++;
    return 0;
}

int deque_push_back(deque_t *dq, int value) {
    if (deque_is_full(dq)) return -1;
    dq->data[dq->tail] = value;
    dq->tail = (dq->tail + 1) % DEQUE_CAP;
    dq->count++;
    return 0;
}

int deque_pop_front(deque_t *dq) {
    if (deque_is_empty(dq)) return -1;
    int val = dq->data[dq->head];
    dq->head = (dq->head + 1) % DEQUE_CAP;
    dq->count--;
    return val;
}

int deque_pop_back(deque_t *dq) {
    if (deque_is_empty(dq)) return -1;
    dq->tail = (dq->tail - 1 + DEQUE_CAP) % DEQUE_CAP;
    dq->count--;
    return dq->data[dq->tail];
}

int deque_front(const deque_t *dq) {
    if (deque_is_empty(dq)) return -1;
    return dq->data[dq->head];
}

int deque_back(const deque_t *dq) {
    if (deque_is_empty(dq)) return -1;
    int idx = (dq->tail - 1 + DEQUE_CAP) % DEQUE_CAP;
    return dq->data[idx];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C517: Deque - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C517: empty output");
    assert!(code.contains("fn deque_push_front"), "C517: Should contain deque_push_front");
    assert!(code.contains("fn deque_push_back"), "C517: Should contain deque_push_back");
    assert!(code.contains("fn deque_pop_front"), "C517: Should contain deque_pop_front");
}

#[test]
fn c518_treap_randomized_bst() {
    let c_code = r#"
#define TREAP_MAX 128

typedef struct {
    int key;
    int priority;
    int left;
    int right;
} treap_node_t;

typedef struct {
    treap_node_t nodes[TREAP_MAX];
    int root;
    int next_free;
    unsigned int seed;
} treap_t;

unsigned int treap_rand(treap_t *t) {
    t->seed = t->seed * 1103515245 + 12345;
    return (t->seed >> 16) & 0x7fff;
}

void treap_init(treap_t *t) {
    t->root = -1;
    t->next_free = 0;
    t->seed = 42;
}

int treap_rotate_right(treap_t *t, int p) {
    int q = t->nodes[p].left;
    t->nodes[p].left = t->nodes[q].right;
    t->nodes[q].right = p;
    return q;
}

int treap_rotate_left(treap_t *t, int p) {
    int q = t->nodes[p].right;
    t->nodes[p].right = t->nodes[q].left;
    t->nodes[q].left = p;
    return q;
}

int treap_insert_rec(treap_t *t, int node, int key) {
    if (node == -1) {
        if (t->next_free >= TREAP_MAX) return -1;
        int idx = t->next_free;
        t->next_free++;
        t->nodes[idx].key = key;
        t->nodes[idx].priority = (int)treap_rand(t);
        t->nodes[idx].left = -1;
        t->nodes[idx].right = -1;
        return idx;
    }
    if (key < t->nodes[node].key) {
        t->nodes[node].left = treap_insert_rec(t, t->nodes[node].left, key);
        if (t->nodes[t->nodes[node].left].priority > t->nodes[node].priority)
            node = treap_rotate_right(t, node);
    } else {
        t->nodes[node].right = treap_insert_rec(t, t->nodes[node].right, key);
        if (t->nodes[t->nodes[node].right].priority > t->nodes[node].priority)
            node = treap_rotate_left(t, node);
    }
    return node;
}

void treap_insert(treap_t *t, int key) {
    t->root = treap_insert_rec(t, t->root, key);
}

int treap_search(const treap_t *t, int node, int key) {
    if (node == -1) return 0;
    if (key == t->nodes[node].key) return 1;
    if (key < t->nodes[node].key)
        return treap_search(t, t->nodes[node].left, key);
    return treap_search(t, t->nodes[node].right, key);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C518: Treap - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C518: empty output");
    assert!(code.contains("fn treap_init"), "C518: Should contain treap_init");
    assert!(code.contains("fn treap_insert"), "C518: Should contain treap_insert");
    assert!(code.contains("fn treap_search"), "C518: Should contain treap_search");
}

#[test]
fn c519_btree_node_operations() {
    let c_code = r#"
#define BT_ORDER 3
#define BT_MAX_KEYS (2 * BT_ORDER - 1)
#define BT_MAX_NODES 64

typedef struct {
    int keys[BT_MAX_KEYS];
    int children[BT_MAX_KEYS + 1];
    int num_keys;
    int is_leaf;
} btnode_t;

typedef struct {
    btnode_t nodes[BT_MAX_NODES];
    int root;
    int next_free;
} btree_t;

void bt_init(btree_t *bt) {
    bt->root = 0;
    bt->next_free = 1;
    bt->nodes[0].num_keys = 0;
    bt->nodes[0].is_leaf = 1;
    int i;
    for (i = 0; i <= BT_MAX_KEYS; i++) {
        bt->nodes[0].children[i] = -1;
    }
}

int bt_search_node(const btnode_t *node, int key) {
    int lo = 0;
    int hi = node->num_keys - 1;
    while (lo <= hi) {
        int mid = (lo + hi) / 2;
        if (node->keys[mid] == key) return mid;
        if (node->keys[mid] < key) lo = mid + 1;
        else hi = mid - 1;
    }
    return -1;
}

int bt_find_child(const btnode_t *node, int key) {
    int i = 0;
    while (i < node->num_keys && key > node->keys[i]) {
        i++;
    }
    return i;
}

void bt_insert_into_node(btnode_t *node, int key, int right_child) {
    int i = node->num_keys - 1;
    while (i >= 0 && node->keys[i] > key) {
        node->keys[i + 1] = node->keys[i];
        node->children[i + 2] = node->children[i + 1];
        i--;
    }
    node->keys[i + 1] = key;
    node->children[i + 2] = right_child;
    node->num_keys++;
}

int bt_split_child(btree_t *bt, int parent_idx, int child_pos) {
    int child_idx = bt->nodes[parent_idx].children[child_pos];
    int new_idx = bt->next_free;
    bt->next_free++;
    int mid = BT_ORDER - 1;
    int median_key = bt->nodes[child_idx].keys[mid];
    bt->nodes[new_idx].is_leaf = bt->nodes[child_idx].is_leaf;
    bt->nodes[new_idx].num_keys = 0;
    int i;
    for (i = mid + 1; i < bt->nodes[child_idx].num_keys; i++) {
        bt->nodes[new_idx].keys[bt->nodes[new_idx].num_keys] = bt->nodes[child_idx].keys[i];
        bt->nodes[new_idx].children[bt->nodes[new_idx].num_keys] = bt->nodes[child_idx].children[i];
        bt->nodes[new_idx].num_keys++;
    }
    bt->nodes[new_idx].children[bt->nodes[new_idx].num_keys] = bt->nodes[child_idx].children[i];
    bt->nodes[child_idx].num_keys = mid;
    bt_insert_into_node(&bt->nodes[parent_idx], median_key, new_idx);
    return median_key;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C519: B-tree - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C519: empty output");
    assert!(code.contains("fn bt_init"), "C519: Should contain bt_init");
    assert!(code.contains("fn bt_search_node"), "C519: Should contain bt_search_node");
    assert!(code.contains("fn bt_split_child"), "C519: Should contain bt_split_child");
}

#[test]
fn c520_bloom_filter() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

#define BLOOM_BITS 256
#define BLOOM_BYTES (BLOOM_BITS / 8)

typedef struct {
    uint8_t bits[BLOOM_BYTES];
    int num_hashes;
} bloom_t;

void bloom_init(bloom_t *bf, int num_hashes) {
    int i;
    for (i = 0; i < BLOOM_BYTES; i++) {
        bf->bits[i] = 0;
    }
    bf->num_hashes = num_hashes;
}

uint32_t bloom_hash1(uint32_t key) {
    key = ((key >> 16) ^ key) * 0x45d9f3b;
    key = ((key >> 16) ^ key) * 0x45d9f3b;
    key = (key >> 16) ^ key;
    return key;
}

uint32_t bloom_hash2(uint32_t key) {
    key = ((key >> 16) ^ key) * 0x119de1f3;
    key = ((key >> 16) ^ key) * 0x119de1f3;
    key = (key >> 16) ^ key;
    return key;
}

void bloom_set_bit(bloom_t *bf, uint32_t pos) {
    uint32_t byte_idx = (pos % BLOOM_BITS) / 8;
    uint32_t bit_idx = (pos % BLOOM_BITS) % 8;
    bf->bits[byte_idx] |= (uint8_t)(1 << bit_idx);
}

int bloom_get_bit(const bloom_t *bf, uint32_t pos) {
    uint32_t byte_idx = (pos % BLOOM_BITS) / 8;
    uint32_t bit_idx = (pos % BLOOM_BITS) % 8;
    return (bf->bits[byte_idx] >> bit_idx) & 1;
}

void bloom_add(bloom_t *bf, uint32_t key) {
    uint32_t h1 = bloom_hash1(key);
    uint32_t h2 = bloom_hash2(key);
    int i;
    for (i = 0; i < bf->num_hashes; i++) {
        uint32_t h = h1 + (uint32_t)i * h2;
        bloom_set_bit(bf, h);
    }
}

int bloom_query(const bloom_t *bf, uint32_t key) {
    uint32_t h1 = bloom_hash1(key);
    uint32_t h2 = bloom_hash2(key);
    int i;
    for (i = 0; i < bf->num_hashes; i++) {
        uint32_t h = h1 + (uint32_t)i * h2;
        if (!bloom_get_bit(bf, h)) return 0;
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C520: Bloom filter - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C520: empty output");
    assert!(code.contains("fn bloom_init"), "C520: Should contain bloom_init");
    assert!(code.contains("fn bloom_add"), "C520: Should contain bloom_add");
    assert!(code.contains("fn bloom_query"), "C520: Should contain bloom_query");
}

// ============================================================================
// C521-C525: Specialized Structures
// ============================================================================

#[test]
fn c521_lru_cache_hash_and_list() {
    let c_code = r#"
#define LRU_CAP 32

typedef struct {
    int key;
    int value;
    int prev;
    int next;
    int used;
} lru_entry_t;

typedef struct {
    lru_entry_t entries[LRU_CAP];
    int hash_table[LRU_CAP];
    int head;
    int tail;
    int count;
} lru_t;

void lru_init(lru_t *cache) {
    int i;
    cache->head = -1;
    cache->tail = -1;
    cache->count = 0;
    for (i = 0; i < LRU_CAP; i++) {
        cache->entries[i].used = 0;
        cache->hash_table[i] = -1;
    }
}

int lru_hash(int key) {
    return ((unsigned int)key * 2654435761u) % LRU_CAP;
}

void lru_detach(lru_t *cache, int idx) {
    int p = cache->entries[idx].prev;
    int n = cache->entries[idx].next;
    if (p != -1) cache->entries[p].next = n;
    else cache->head = n;
    if (n != -1) cache->entries[n].prev = p;
    else cache->tail = p;
}

void lru_attach_front(lru_t *cache, int idx) {
    cache->entries[idx].prev = -1;
    cache->entries[idx].next = cache->head;
    if (cache->head != -1) cache->entries[cache->head].prev = idx;
    cache->head = idx;
    if (cache->tail == -1) cache->tail = idx;
}

int lru_get(lru_t *cache, int key) {
    int h = lru_hash(key);
    int idx = cache->hash_table[h];
    while (idx != -1 && cache->entries[idx].key != key) {
        idx = -1;
    }
    if (idx == -1) return -1;
    lru_detach(cache, idx);
    lru_attach_front(cache, idx);
    return cache->entries[idx].value;
}

void lru_put(lru_t *cache, int key, int value) {
    int h = lru_hash(key);
    int idx;
    if (cache->count < LRU_CAP) {
        idx = cache->count;
        cache->count++;
    } else {
        idx = cache->tail;
        lru_detach(cache, idx);
        int old_h = lru_hash(cache->entries[idx].key);
        cache->hash_table[old_h] = -1;
    }
    cache->entries[idx].key = key;
    cache->entries[idx].value = value;
    cache->entries[idx].used = 1;
    cache->hash_table[h] = idx;
    lru_attach_front(cache, idx);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C521: LRU cache - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C521: empty output");
    assert!(code.contains("fn lru_init"), "C521: Should contain lru_init");
    assert!(code.contains("fn lru_get"), "C521: Should contain lru_get");
    assert!(code.contains("fn lru_put"), "C521: Should contain lru_put");
}

#[test]
fn c522_skip_list_probabilistic() {
    let c_code = r#"
#define SKIP_MAX_LEVEL 8
#define SKIP_MAX_NODES 128

typedef struct {
    int key;
    int value;
    int forward[SKIP_MAX_LEVEL];
    int level;
} skip_node_t;

typedef struct {
    skip_node_t nodes[SKIP_MAX_NODES];
    int head;
    int level;
    int next_free;
    unsigned int seed;
} skiplist_t;

unsigned int skip_rand(skiplist_t *sl) {
    sl->seed = sl->seed * 1103515245 + 12345;
    return (sl->seed >> 16) & 0x7fff;
}

int skip_random_level(skiplist_t *sl) {
    int lvl = 1;
    while (lvl < SKIP_MAX_LEVEL && (skip_rand(sl) % 4) == 0) {
        lvl++;
    }
    return lvl;
}

void skip_init(skiplist_t *sl) {
    int i;
    sl->head = 0;
    sl->level = 1;
    sl->next_free = 1;
    sl->seed = 12345;
    sl->nodes[0].key = -1;
    sl->nodes[0].value = 0;
    sl->nodes[0].level = SKIP_MAX_LEVEL;
    for (i = 0; i < SKIP_MAX_LEVEL; i++) {
        sl->nodes[0].forward[i] = -1;
    }
}

int skip_search(const skiplist_t *sl, int key) {
    int cur = sl->head;
    int i;
    for (i = sl->level - 1; i >= 0; i--) {
        while (sl->nodes[cur].forward[i] != -1 &&
               sl->nodes[sl->nodes[cur].forward[i]].key < key) {
            cur = sl->nodes[cur].forward[i];
        }
    }
    cur = sl->nodes[cur].forward[0];
    if (cur != -1 && sl->nodes[cur].key == key)
        return sl->nodes[cur].value;
    return -1;
}

int skip_insert(skiplist_t *sl, int key, int value) {
    int update[SKIP_MAX_LEVEL];
    int cur = sl->head;
    int i;
    for (i = sl->level - 1; i >= 0; i--) {
        while (sl->nodes[cur].forward[i] != -1 &&
               sl->nodes[sl->nodes[cur].forward[i]].key < key) {
            cur = sl->nodes[cur].forward[i];
        }
        update[i] = cur;
    }
    if (sl->next_free >= SKIP_MAX_NODES) return -1;
    int lvl = skip_random_level(sl);
    int idx = sl->next_free;
    sl->next_free++;
    sl->nodes[idx].key = key;
    sl->nodes[idx].value = value;
    sl->nodes[idx].level = lvl;
    if (lvl > sl->level) {
        for (i = sl->level; i < lvl; i++) {
            update[i] = sl->head;
        }
        sl->level = lvl;
    }
    for (i = 0; i < lvl; i++) {
        sl->nodes[idx].forward[i] = sl->nodes[update[i]].forward[i];
        sl->nodes[update[i]].forward[i] = idx;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C522: Skip list - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C522: empty output");
    assert!(code.contains("fn skip_init"), "C522: Should contain skip_init");
    assert!(code.contains("fn skip_search"), "C522: Should contain skip_search");
    assert!(code.contains("fn skip_insert"), "C522: Should contain skip_insert");
}

#[test]
fn c523_circular_buffer_producer_consumer() {
    let c_code = r#"
#define CBUF_SIZE 64

typedef struct {
    int buffer[CBUF_SIZE];
    int read_pos;
    int write_pos;
    int count;
    int total_produced;
    int total_consumed;
} cbuf_t;

void cbuf_init(cbuf_t *cb) {
    cb->read_pos = 0;
    cb->write_pos = 0;
    cb->count = 0;
    cb->total_produced = 0;
    cb->total_consumed = 0;
}

int cbuf_is_full(const cbuf_t *cb) {
    return cb->count == CBUF_SIZE;
}

int cbuf_is_empty(const cbuf_t *cb) {
    return cb->count == 0;
}

int cbuf_produce(cbuf_t *cb, int value) {
    if (cbuf_is_full(cb)) return -1;
    cb->buffer[cb->write_pos] = value;
    cb->write_pos = (cb->write_pos + 1) % CBUF_SIZE;
    cb->count++;
    cb->total_produced++;
    return 0;
}

int cbuf_consume(cbuf_t *cb) {
    if (cbuf_is_empty(cb)) return -1;
    int val = cb->buffer[cb->read_pos];
    cb->read_pos = (cb->read_pos + 1) % CBUF_SIZE;
    cb->count--;
    cb->total_consumed++;
    return val;
}

int cbuf_available(const cbuf_t *cb) {
    return cb->count;
}

int cbuf_free_space(const cbuf_t *cb) {
    return CBUF_SIZE - cb->count;
}

int cbuf_peek(const cbuf_t *cb, int offset) {
    if (offset >= cb->count) return -1;
    int pos = (cb->read_pos + offset) % CBUF_SIZE;
    return cb->buffer[pos];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C523: Circular buffer - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C523: empty output");
    assert!(code.contains("fn cbuf_init"), "C523: Should contain cbuf_init");
    assert!(code.contains("fn cbuf_produce"), "C523: Should contain cbuf_produce");
    assert!(code.contains("fn cbuf_consume"), "C523: Should contain cbuf_consume");
}

#[test]
fn c524_suffix_array_construction() {
    let c_code = r#"
#define SA_MAX 128

typedef struct {
    int sa[SA_MAX];
    int rank[SA_MAX];
    int tmp[SA_MAX];
    int lcp[SA_MAX];
    int n;
} suffix_array_t;

void sa_init(suffix_array_t *s, const char *text, int n) {
    int i;
    s->n = n;
    for (i = 0; i < n; i++) {
        s->sa[i] = i;
        s->rank[i] = text[i];
    }
}

void sa_simple_sort(suffix_array_t *s, const char *text) {
    int i, j;
    for (i = 0; i < s->n - 1; i++) {
        for (j = i + 1; j < s->n; j++) {
            int a = s->sa[i];
            int b = s->sa[j];
            int k;
            int swapped = 0;
            for (k = 0; a + k < s->n && b + k < s->n; k++) {
                if (text[a + k] < text[b + k]) break;
                if (text[a + k] > text[b + k]) {
                    int tmp = s->sa[i];
                    s->sa[i] = s->sa[j];
                    s->sa[j] = tmp;
                    swapped = 1;
                    break;
                }
            }
            if (!swapped && a + k >= s->n && b + k < s->n) {
                int tmp = s->sa[i];
                s->sa[i] = s->sa[j];
                s->sa[j] = tmp;
            }
        }
    }
}

void sa_compute_lcp(suffix_array_t *s, const char *text) {
    int i;
    for (i = 0; i < s->n - 1; i++) {
        int a = s->sa[i];
        int b = s->sa[i + 1];
        int k = 0;
        while (a + k < s->n && b + k < s->n && text[a + k] == text[b + k]) {
            k++;
        }
        s->lcp[i] = k;
    }
    s->lcp[s->n - 1] = 0;
}

int sa_search(const suffix_array_t *s, const char *text, const char *pattern, int plen) {
    int lo = 0;
    int hi = s->n - 1;
    while (lo <= hi) {
        int mid = (lo + hi) / 2;
        int pos = s->sa[mid];
        int k;
        int cmp = 0;
        for (k = 0; k < plen && pos + k < s->n; k++) {
            if (text[pos + k] < pattern[k]) { cmp = -1; break; }
            if (text[pos + k] > pattern[k]) { cmp = 1; break; }
        }
        if (cmp == 0 && k < plen) cmp = -1;
        if (cmp == 0) return s->sa[mid];
        if (cmp < 0) lo = mid + 1;
        else hi = mid - 1;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C524: Suffix array - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C524: empty output");
    assert!(code.contains("fn sa_init"), "C524: Should contain sa_init");
    assert!(code.contains("fn sa_compute_lcp"), "C524: Should contain sa_compute_lcp");
    assert!(code.contains("fn sa_search"), "C524: Should contain sa_search");
}

#[test]
fn c525_persistent_stack_immutable() {
    let c_code = r#"
#define PSTACK_MAX 256

typedef struct {
    int value;
    int prev;
} pstack_node_t;

typedef struct {
    pstack_node_t nodes[PSTACK_MAX];
    int tops[64];
    int next_free;
    int num_versions;
} pstack_t;

void pstack_init(pstack_t *ps) {
    ps->next_free = 0;
    ps->num_versions = 1;
    ps->tops[0] = -1;
}

int pstack_push(pstack_t *ps, int version, int value) {
    if (ps->next_free >= PSTACK_MAX) return -1;
    if (ps->num_versions >= 64) return -1;
    int idx = ps->next_free;
    ps->next_free++;
    ps->nodes[idx].value = value;
    ps->nodes[idx].prev = ps->tops[version];
    int new_version = ps->num_versions;
    ps->tops[new_version] = idx;
    ps->num_versions++;
    return new_version;
}

int pstack_pop(pstack_t *ps, int version) {
    if (ps->tops[version] == -1) return -1;
    if (ps->num_versions >= 64) return -1;
    int new_version = ps->num_versions;
    ps->tops[new_version] = ps->nodes[ps->tops[version]].prev;
    ps->num_versions++;
    return new_version;
}

int pstack_top(const pstack_t *ps, int version) {
    if (ps->tops[version] == -1) return -1;
    return ps->nodes[ps->tops[version]].value;
}

int pstack_is_empty(const pstack_t *ps, int version) {
    return ps->tops[version] == -1;
}

int pstack_size(const pstack_t *ps, int version) {
    int count = 0;
    int cur = ps->tops[version];
    while (cur != -1) {
        count++;
        cur = ps->nodes[cur].prev;
    }
    return count;
}

int pstack_to_array(const pstack_t *ps, int version, int *out, int max_len) {
    int count = 0;
    int cur = ps->tops[version];
    while (cur != -1 && count < max_len) {
        out[count] = ps->nodes[cur].value;
        count++;
        cur = ps->nodes[cur].prev;
    }
    int i, j;
    for (i = 0, j = count - 1; i < j; i++, j--) {
        int tmp = out[i];
        out[i] = out[j];
        out[j] = tmp;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C525: Persistent stack - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C525: empty output");
    assert!(code.contains("fn pstack_init"), "C525: Should contain pstack_init");
    assert!(code.contains("fn pstack_push"), "C525: Should contain pstack_push");
    assert!(code.contains("fn pstack_top"), "C525: Should contain pstack_top");
}

