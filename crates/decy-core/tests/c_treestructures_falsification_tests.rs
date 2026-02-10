//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1051-C1075: Tree Structure implementations -- the kind of C code found
//! in textbooks (CLRS, Sedgewick), competitive programming, and systems software.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise classic and advanced tree data structure patterns commonly
//! found in real-world C codebases -- all expressed as valid C99 without #include.
//! All structures use array-based node pools to avoid malloc/free.
//!
//! Organization:
//! - C1051-C1055: Self-balancing trees (AVL, red-black, B-tree, binary heap, Fibonacci heap)
//! - C1056-C1060: Query/string trees (segment tree, Fenwick tree, trie, radix tree, suffix tree)
//! - C1061-C1065: Randomized/spatial trees (splay, treap, k-d tree, interval tree, Van Emde Boas)
//! - C1066-C1070: Specialized trees (Merkle, quadtree, octree, rope, persistent segment tree)
//! - C1071-C1075: Exotic trees (Cartesian, scapegoat, AA tree, weight-balanced, link-cut)

use decy_core::transpile;

// ============================================================================
// C1051-C1055: Self-Balancing Trees
// ============================================================================

#[test]
fn c1051_avl_tree() {
    let c_code = r#"
typedef struct {
    int key;
    int left;
    int right;
    int height;
} tree_AvlNode;

typedef struct {
    tree_AvlNode nodes[1024];
    int root;
    int next_free;
} tree_Avl;

int tree_avl_height(tree_Avl *t, int idx) {
    if (idx == -1) return 0;
    return t->nodes[idx].height;
}

int tree_avl_max(int a, int b) {
    return a > b ? a : b;
}

void tree_avl_update_height(tree_Avl *t, int idx) {
    int lh = tree_avl_height(t, t->nodes[idx].left);
    int rh = tree_avl_height(t, t->nodes[idx].right);
    t->nodes[idx].height = 1 + tree_avl_max(lh, rh);
}

int tree_avl_balance_factor(tree_Avl *t, int idx) {
    if (idx == -1) return 0;
    return tree_avl_height(t, t->nodes[idx].left) - tree_avl_height(t, t->nodes[idx].right);
}

int tree_avl_rotate_right(tree_Avl *t, int y) {
    int x = t->nodes[y].left;
    int T2 = t->nodes[x].right;
    t->nodes[x].right = y;
    t->nodes[y].left = T2;
    tree_avl_update_height(t, y);
    tree_avl_update_height(t, x);
    return x;
}

int tree_avl_rotate_left(tree_Avl *t, int x) {
    int y = t->nodes[x].right;
    int T2 = t->nodes[y].left;
    t->nodes[y].left = x;
    t->nodes[x].right = T2;
    tree_avl_update_height(t, x);
    tree_avl_update_height(t, y);
    return y;
}

int tree_avl_alloc(tree_Avl *t, int key) {
    int idx = t->next_free;
    t->next_free++;
    t->nodes[idx].key = key;
    t->nodes[idx].left = -1;
    t->nodes[idx].right = -1;
    t->nodes[idx].height = 1;
    return idx;
}

int tree_avl_insert(tree_Avl *t, int node, int key) {
    int bf;
    if (node == -1) return tree_avl_alloc(t, key);
    if (key < t->nodes[node].key) {
        t->nodes[node].left = tree_avl_insert(t, t->nodes[node].left, key);
    } else if (key > t->nodes[node].key) {
        t->nodes[node].right = tree_avl_insert(t, t->nodes[node].right, key);
    } else {
        return node;
    }
    tree_avl_update_height(t, node);
    bf = tree_avl_balance_factor(t, node);
    if (bf > 1 && key < t->nodes[t->nodes[node].left].key)
        return tree_avl_rotate_right(t, node);
    if (bf < -1 && key > t->nodes[t->nodes[node].right].key)
        return tree_avl_rotate_left(t, node);
    if (bf > 1 && key > t->nodes[t->nodes[node].left].key) {
        t->nodes[node].left = tree_avl_rotate_left(t, t->nodes[node].left);
        return tree_avl_rotate_right(t, node);
    }
    if (bf < -1 && key < t->nodes[t->nodes[node].right].key) {
        t->nodes[node].right = tree_avl_rotate_right(t, t->nodes[node].right);
        return tree_avl_rotate_left(t, node);
    }
    return node;
}

void tree_avl_init(tree_Avl *t) {
    t->root = -1;
    t->next_free = 0;
}

void tree_avl_add(tree_Avl *t, int key) {
    t->root = tree_avl_insert(t, t->root, key);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1051 AVL tree failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1051: empty output");
    assert!(code.contains("fn tree_avl_insert"), "C1051: Should contain tree_avl_insert");
    assert!(code.contains("fn tree_avl_rotate_right"), "C1051: Should contain tree_avl_rotate_right");
}

#[test]
fn c1052_red_black_tree() {
    let c_code = r#"
typedef struct {
    int key;
    int left;
    int right;
    int parent;
    int color;
} tree_RbNode;

typedef struct {
    tree_RbNode nodes[1024];
    int root;
    int nil;
    int next_free;
} tree_Rbt;

void tree_rbt_init(tree_Rbt *t) {
    t->nil = 0;
    t->nodes[0].color = 0;
    t->nodes[0].left = 0;
    t->nodes[0].right = 0;
    t->nodes[0].parent = 0;
    t->nodes[0].key = 0;
    t->root = 0;
    t->next_free = 1;
}

int tree_rbt_alloc(tree_Rbt *t, int key) {
    int idx = t->next_free;
    t->next_free++;
    t->nodes[idx].key = key;
    t->nodes[idx].left = t->nil;
    t->nodes[idx].right = t->nil;
    t->nodes[idx].parent = t->nil;
    t->nodes[idx].color = 1;
    return idx;
}

void tree_rbt_rotate_left(tree_Rbt *t, int x) {
    int y = t->nodes[x].right;
    t->nodes[x].right = t->nodes[y].left;
    if (t->nodes[y].left != t->nil)
        t->nodes[t->nodes[y].left].parent = x;
    t->nodes[y].parent = t->nodes[x].parent;
    if (t->nodes[x].parent == t->nil)
        t->root = y;
    else if (x == t->nodes[t->nodes[x].parent].left)
        t->nodes[t->nodes[x].parent].left = y;
    else
        t->nodes[t->nodes[x].parent].right = y;
    t->nodes[y].left = x;
    t->nodes[x].parent = y;
}

void tree_rbt_rotate_right(tree_Rbt *t, int x) {
    int y = t->nodes[x].left;
    t->nodes[x].left = t->nodes[y].right;
    if (t->nodes[y].right != t->nil)
        t->nodes[t->nodes[y].right].parent = x;
    t->nodes[y].parent = t->nodes[x].parent;
    if (t->nodes[x].parent == t->nil)
        t->root = y;
    else if (x == t->nodes[t->nodes[x].parent].right)
        t->nodes[t->nodes[x].parent].right = y;
    else
        t->nodes[t->nodes[x].parent].left = y;
    t->nodes[y].right = x;
    t->nodes[x].parent = y;
}

void tree_rbt_insert_fixup(tree_Rbt *t, int z) {
    int y;
    while (t->nodes[t->nodes[z].parent].color == 1) {
        if (t->nodes[z].parent == t->nodes[t->nodes[t->nodes[z].parent].parent].left) {
            y = t->nodes[t->nodes[t->nodes[z].parent].parent].right;
            if (t->nodes[y].color == 1) {
                t->nodes[t->nodes[z].parent].color = 0;
                t->nodes[y].color = 0;
                t->nodes[t->nodes[t->nodes[z].parent].parent].color = 1;
                z = t->nodes[t->nodes[z].parent].parent;
            } else {
                if (z == t->nodes[t->nodes[z].parent].right) {
                    z = t->nodes[z].parent;
                    tree_rbt_rotate_left(t, z);
                }
                t->nodes[t->nodes[z].parent].color = 0;
                t->nodes[t->nodes[t->nodes[z].parent].parent].color = 1;
                tree_rbt_rotate_right(t, t->nodes[t->nodes[z].parent].parent);
            }
        } else {
            y = t->nodes[t->nodes[t->nodes[z].parent].parent].left;
            if (t->nodes[y].color == 1) {
                t->nodes[t->nodes[z].parent].color = 0;
                t->nodes[y].color = 0;
                t->nodes[t->nodes[t->nodes[z].parent].parent].color = 1;
                z = t->nodes[t->nodes[z].parent].parent;
            } else {
                if (z == t->nodes[t->nodes[z].parent].left) {
                    z = t->nodes[z].parent;
                    tree_rbt_rotate_right(t, z);
                }
                t->nodes[t->nodes[z].parent].color = 0;
                t->nodes[t->nodes[t->nodes[z].parent].parent].color = 1;
                tree_rbt_rotate_left(t, t->nodes[t->nodes[z].parent].parent);
            }
        }
    }
    t->nodes[t->root].color = 0;
}

void tree_rbt_insert(tree_Rbt *t, int key) {
    int z = tree_rbt_alloc(t, key);
    int y = t->nil;
    int x = t->root;
    while (x != t->nil) {
        y = x;
        if (t->nodes[z].key < t->nodes[x].key)
            x = t->nodes[x].left;
        else
            x = t->nodes[x].right;
    }
    t->nodes[z].parent = y;
    if (y == t->nil)
        t->root = z;
    else if (t->nodes[z].key < t->nodes[y].key)
        t->nodes[y].left = z;
    else
        t->nodes[y].right = z;
    tree_rbt_insert_fixup(t, z);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1052 red-black tree failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1052: empty output");
    assert!(code.contains("fn tree_rbt_insert"), "C1052: Should contain tree_rbt_insert");
}

#[test]
fn c1053_btree_order3() {
    let c_code = r#"
typedef struct {
    int keys[2];
    int children[3];
    int num_keys;
    int is_leaf;
} tree_BtNode;

typedef struct {
    tree_BtNode nodes[1024];
    int root;
    int next_free;
} tree_Bt;

void tree_bt_init(tree_Bt *t) {
    t->root = -1;
    t->next_free = 0;
}

int tree_bt_alloc(tree_Bt *t) {
    int idx = t->next_free;
    t->next_free++;
    t->nodes[idx].num_keys = 0;
    t->nodes[idx].is_leaf = 1;
    t->nodes[idx].children[0] = -1;
    t->nodes[idx].children[1] = -1;
    t->nodes[idx].children[2] = -1;
    return idx;
}

int tree_bt_search(tree_Bt *t, int node, int key) {
    int i = 0;
    if (node == -1) return -1;
    while (i < t->nodes[node].num_keys && key > t->nodes[node].keys[i])
        i++;
    if (i < t->nodes[node].num_keys && key == t->nodes[node].keys[i])
        return node;
    if (t->nodes[node].is_leaf)
        return -1;
    return tree_bt_search(t, t->nodes[node].children[i], key);
}

void tree_bt_insert_nonfull(tree_Bt *t, int node, int key) {
    int i = t->nodes[node].num_keys - 1;
    if (t->nodes[node].is_leaf) {
        while (i >= 0 && t->nodes[node].keys[i] > key) {
            t->nodes[node].keys[i + 1] = t->nodes[node].keys[i];
            i--;
        }
        t->nodes[node].keys[i + 1] = key;
        t->nodes[node].num_keys++;
    } else {
        while (i >= 0 && t->nodes[node].keys[i] > key)
            i--;
        i++;
        if (t->nodes[t->nodes[node].children[i]].num_keys == 2) {
            int child = t->nodes[node].children[i];
            int new_node = tree_bt_alloc(t);
            int mid_key = t->nodes[child].keys[1];
            t->nodes[new_node].keys[0] = t->nodes[child].keys[1];
            t->nodes[new_node].num_keys = 0;
            t->nodes[child].num_keys = 1;
            t->nodes[new_node].is_leaf = t->nodes[child].is_leaf;
            if (!t->nodes[child].is_leaf) {
                t->nodes[new_node].children[0] = t->nodes[child].children[2];
                t->nodes[child].children[2] = -1;
            }
            t->nodes[node].keys[1] = t->nodes[node].keys[0];
            t->nodes[node].children[2] = t->nodes[node].children[1];
            if (i == 0) {
                t->nodes[node].keys[0] = mid_key;
                t->nodes[node].children[1] = new_node;
            } else {
                t->nodes[node].keys[0] = t->nodes[node].keys[0];
                t->nodes[node].children[1] = new_node;
            }
            t->nodes[node].num_keys++;
            if (key < mid_key)
                tree_bt_insert_nonfull(t, child, key);
            else
                tree_bt_insert_nonfull(t, new_node, key);
        } else {
            tree_bt_insert_nonfull(t, t->nodes[node].children[i], key);
        }
    }
}

void tree_bt_insert(tree_Bt *t, int key) {
    if (t->root == -1) {
        t->root = tree_bt_alloc(t);
        t->nodes[t->root].keys[0] = key;
        t->nodes[t->root].num_keys = 1;
        return;
    }
    if (t->nodes[t->root].num_keys == 2) {
        int new_root = tree_bt_alloc(t);
        t->nodes[new_root].is_leaf = 0;
        t->nodes[new_root].children[0] = t->root;
        t->root = new_root;
        tree_bt_insert_nonfull(t, t->root, key);
    } else {
        tree_bt_insert_nonfull(t, t->root, key);
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1053 B-tree failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1053: empty output");
    assert!(code.contains("fn tree_bt_insert"), "C1053: Should contain tree_bt_insert");
}

#[test]
fn c1054_binary_heap() {
    let c_code = r#"
typedef struct {
    int data[1024];
    int size;
} tree_MinHeap;

void tree_heap_init(tree_MinHeap *h) {
    h->size = 0;
}

void tree_heap_swap(int *a, int *b) {
    int tmp = *a;
    *a = *b;
    *b = tmp;
}

void tree_heap_sift_up(tree_MinHeap *h, int idx) {
    int parent;
    while (idx > 0) {
        parent = (idx - 1) / 2;
        if (h->data[parent] > h->data[idx]) {
            tree_heap_swap(&h->data[parent], &h->data[idx]);
            idx = parent;
        } else {
            break;
        }
    }
}

void tree_heap_sift_down(tree_MinHeap *h, int idx) {
    int smallest = idx;
    int left = 2 * idx + 1;
    int right = 2 * idx + 2;
    if (left < h->size && h->data[left] < h->data[smallest])
        smallest = left;
    if (right < h->size && h->data[right] < h->data[smallest])
        smallest = right;
    if (smallest != idx) {
        tree_heap_swap(&h->data[idx], &h->data[smallest]);
        tree_heap_sift_down(h, smallest);
    }
}

void tree_heap_push(tree_MinHeap *h, int val) {
    if (h->size >= 1024) return;
    h->data[h->size] = val;
    h->size++;
    tree_heap_sift_up(h, h->size - 1);
}

int tree_heap_pop(tree_MinHeap *h) {
    int min_val;
    if (h->size == 0) return -1;
    min_val = h->data[0];
    h->size--;
    h->data[0] = h->data[h->size];
    tree_heap_sift_down(h, 0);
    return min_val;
}

int tree_heap_peek(tree_MinHeap *h) {
    if (h->size == 0) return -1;
    return h->data[0];
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1054 binary heap failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1054: empty output");
    assert!(code.contains("fn tree_heap_push"), "C1054: Should contain tree_heap_push");
    assert!(code.contains("fn tree_heap_pop"), "C1054: Should contain tree_heap_pop");
}

#[test]
fn c1055_fibonacci_heap() {
    let c_code = r#"
typedef struct {
    int key;
    int degree;
    int parent;
    int child;
    int left;
    int right;
    int marked;
} tree_FibNode;

typedef struct {
    tree_FibNode nodes[1024];
    int min_idx;
    int num_nodes;
    int next_free;
} tree_FibHeap;

void tree_fib_init(tree_FibHeap *h) {
    h->min_idx = -1;
    h->num_nodes = 0;
    h->next_free = 0;
}

int tree_fib_alloc(tree_FibHeap *h, int key) {
    int idx = h->next_free;
    h->next_free++;
    h->nodes[idx].key = key;
    h->nodes[idx].degree = 0;
    h->nodes[idx].parent = -1;
    h->nodes[idx].child = -1;
    h->nodes[idx].left = idx;
    h->nodes[idx].right = idx;
    h->nodes[idx].marked = 0;
    return idx;
}

void tree_fib_list_insert(tree_FibHeap *h, int a, int b) {
    int a_right = h->nodes[a].right;
    h->nodes[a].right = b;
    h->nodes[b].left = a;
    h->nodes[b].right = a_right;
    h->nodes[a_right].left = b;
}

void tree_fib_insert(tree_FibHeap *h, int key) {
    int idx = tree_fib_alloc(h, key);
    if (h->min_idx == -1) {
        h->min_idx = idx;
    } else {
        tree_fib_list_insert(h, h->min_idx, idx);
        if (h->nodes[idx].key < h->nodes[h->min_idx].key) {
            h->min_idx = idx;
        }
    }
    h->num_nodes++;
}

int tree_fib_find_min(tree_FibHeap *h) {
    if (h->min_idx == -1) return -1;
    return h->nodes[h->min_idx].key;
}

void tree_fib_decrease_key(tree_FibHeap *h, int idx, int new_key) {
    if (new_key > h->nodes[idx].key) return;
    h->nodes[idx].key = new_key;
    if (h->nodes[idx].key < h->nodes[h->min_idx].key) {
        h->min_idx = idx;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1055 Fibonacci heap failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1055: empty output");
    assert!(code.contains("fn tree_fib_insert"), "C1055: Should contain tree_fib_insert");
}

// ============================================================================
// C1056-C1060: Query and String Trees
// ============================================================================

#[test]
fn c1056_segment_tree() {
    let c_code = r#"
typedef struct {
    int tree[4096];
    int n;
} tree_SegTree;

void tree_seg_build(tree_SegTree *st, int *arr, int node, int start, int end) {
    if (start == end) {
        st->tree[node] = arr[start];
    } else {
        int mid = (start + end) / 2;
        tree_seg_build(st, arr, 2 * node, start, mid);
        tree_seg_build(st, arr, 2 * node + 1, mid + 1, end);
        st->tree[node] = st->tree[2 * node] + st->tree[2 * node + 1];
    }
}

int tree_seg_query(tree_SegTree *st, int node, int start, int end, int l, int r) {
    if (r < start || end < l) return 0;
    if (l <= start && end <= r) return st->tree[node];
    {
        int mid = (start + end) / 2;
        int left_sum = tree_seg_query(st, 2 * node, start, mid, l, r);
        int right_sum = tree_seg_query(st, 2 * node + 1, mid + 1, end, l, r);
        return left_sum + right_sum;
    }
}

void tree_seg_update(tree_SegTree *st, int node, int start, int end, int idx, int val) {
    if (start == end) {
        st->tree[node] = val;
    } else {
        int mid = (start + end) / 2;
        if (idx <= mid)
            tree_seg_update(st, 2 * node, start, mid, idx, val);
        else
            tree_seg_update(st, 2 * node + 1, mid + 1, end, idx, val);
        st->tree[node] = st->tree[2 * node] + st->tree[2 * node + 1];
    }
}

void tree_seg_init(tree_SegTree *st, int *arr, int n) {
    int i;
    st->n = n;
    for (i = 0; i < 4096; i++) st->tree[i] = 0;
    tree_seg_build(st, arr, 1, 0, n - 1);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1056 segment tree failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1056: empty output");
    assert!(code.contains("fn tree_seg_query"), "C1056: Should contain tree_seg_query");
    assert!(code.contains("fn tree_seg_update"), "C1056: Should contain tree_seg_update");
}

#[test]
fn c1057_fenwick_tree() {
    let c_code = r#"
typedef struct {
    int bit[1025];
    int n;
} tree_Fenwick;

void tree_fenwick_init(tree_Fenwick *ft, int n) {
    int i;
    ft->n = n;
    for (i = 0; i <= n; i++) ft->bit[i] = 0;
}

void tree_fenwick_update(tree_Fenwick *ft, int idx, int delta) {
    while (idx <= ft->n) {
        ft->bit[idx] = ft->bit[idx] + delta;
        idx = idx + (idx & (-idx));
    }
}

int tree_fenwick_query(tree_Fenwick *ft, int idx) {
    int sum = 0;
    while (idx > 0) {
        sum = sum + ft->bit[idx];
        idx = idx - (idx & (-idx));
    }
    return sum;
}

int tree_fenwick_range_query(tree_Fenwick *ft, int l, int r) {
    return tree_fenwick_query(ft, r) - tree_fenwick_query(ft, l - 1);
}

void tree_fenwick_build(tree_Fenwick *ft, int *arr, int n) {
    int i;
    tree_fenwick_init(ft, n);
    for (i = 1; i <= n; i++) {
        tree_fenwick_update(ft, i, arr[i - 1]);
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1057 Fenwick tree failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1057: empty output");
    assert!(code.contains("fn tree_fenwick_update"), "C1057: Should contain tree_fenwick_update");
    assert!(code.contains("fn tree_fenwick_query"), "C1057: Should contain tree_fenwick_query");
}

#[test]
fn c1058_trie_with_deletion() {
    let c_code = r#"
typedef struct {
    int children[26];
    int is_end;
    int count;
} tree_TrieNode;

typedef struct {
    tree_TrieNode nodes[4096];
    int root;
    int next_free;
} tree_Trie;

void tree_trie_init(tree_Trie *t) {
    int i;
    t->root = 0;
    t->next_free = 1;
    t->nodes[0].is_end = 0;
    t->nodes[0].count = 0;
    for (i = 0; i < 26; i++) t->nodes[0].children[i] = -1;
}

int tree_trie_alloc(tree_Trie *t) {
    int idx = t->next_free;
    int i;
    t->next_free++;
    t->nodes[idx].is_end = 0;
    t->nodes[idx].count = 0;
    for (i = 0; i < 26; i++) t->nodes[idx].children[i] = -1;
    return idx;
}

void tree_trie_insert(tree_Trie *t, const char *word) {
    int cur = t->root;
    int i = 0;
    while (word[i] != '\0') {
        int c = word[i] - 'a';
        if (t->nodes[cur].children[c] == -1) {
            t->nodes[cur].children[c] = tree_trie_alloc(t);
        }
        cur = t->nodes[cur].children[c];
        t->nodes[cur].count++;
        i++;
    }
    t->nodes[cur].is_end = 1;
}

int tree_trie_search(tree_Trie *t, const char *word) {
    int cur = t->root;
    int i = 0;
    while (word[i] != '\0') {
        int c = word[i] - 'a';
        if (t->nodes[cur].children[c] == -1) return 0;
        cur = t->nodes[cur].children[c];
        i++;
    }
    return t->nodes[cur].is_end;
}

int tree_trie_delete_helper(tree_Trie *t, int node, const char *word, int depth) {
    int c;
    int child_empty;
    int i;
    if (word[depth] == '\0') {
        if (!t->nodes[node].is_end) return 0;
        t->nodes[node].is_end = 0;
        for (i = 0; i < 26; i++) {
            if (t->nodes[node].children[i] != -1) return 0;
        }
        return 1;
    }
    c = word[depth] - 'a';
    if (t->nodes[node].children[c] == -1) return 0;
    child_empty = tree_trie_delete_helper(t, t->nodes[node].children[c], word, depth + 1);
    if (child_empty) {
        t->nodes[node].children[c] = -1;
        t->nodes[node].count--;
        if (!t->nodes[node].is_end && t->nodes[node].count == 0)
            return 1;
    }
    return 0;
}

void tree_trie_delete(tree_Trie *t, const char *word) {
    tree_trie_delete_helper(t, t->root, word, 0);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1058 trie with deletion failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1058: empty output");
    assert!(code.contains("fn tree_trie_insert"), "C1058: Should contain tree_trie_insert");
    assert!(code.contains("fn tree_trie_delete"), "C1058: Should contain tree_trie_delete");
}

#[test]
fn c1059_radix_tree() {
    let c_code = r#"
typedef struct {
    int children[26];
    int edge_start;
    int edge_len;
    int is_end;
    int parent;
} tree_RadixNode;

typedef struct {
    tree_RadixNode nodes[1024];
    char text_pool[4096];
    int text_used;
    int root;
    int next_free;
} tree_RadixTree;

void tree_radix_init(tree_RadixTree *t) {
    int i;
    t->root = 0;
    t->next_free = 1;
    t->text_used = 0;
    t->nodes[0].is_end = 0;
    t->nodes[0].edge_start = -1;
    t->nodes[0].edge_len = 0;
    t->nodes[0].parent = -1;
    for (i = 0; i < 26; i++) t->nodes[0].children[i] = -1;
}

int tree_radix_alloc(tree_RadixTree *t) {
    int idx = t->next_free;
    int i;
    t->next_free++;
    t->nodes[idx].is_end = 0;
    t->nodes[idx].edge_start = -1;
    t->nodes[idx].edge_len = 0;
    t->nodes[idx].parent = -1;
    for (i = 0; i < 26; i++) t->nodes[idx].children[i] = -1;
    return idx;
}

int tree_radix_store_text(tree_RadixTree *t, const char *s, int len) {
    int start = t->text_used;
    int i;
    for (i = 0; i < len; i++) {
        t->text_pool[t->text_used] = s[i];
        t->text_used++;
    }
    return start;
}

void tree_radix_insert(tree_RadixTree *t, const char *key) {
    int cur = t->root;
    int pos = 0;
    int c;
    while (key[pos] != '\0') {
        c = key[pos] - 'a';
        if (t->nodes[cur].children[c] == -1) {
            int new_node = tree_radix_alloc(t);
            int len = 0;
            const char *start = &key[pos];
            while (key[pos + len] != '\0') len++;
            t->nodes[new_node].edge_start = tree_radix_store_text(t, start, len);
            t->nodes[new_node].edge_len = len;
            t->nodes[new_node].is_end = 1;
            t->nodes[new_node].parent = cur;
            t->nodes[cur].children[c] = new_node;
            return;
        }
        cur = t->nodes[cur].children[c];
        pos++;
    }
    t->nodes[cur].is_end = 1;
}

int tree_radix_search(tree_RadixTree *t, const char *key) {
    int cur = t->root;
    int pos = 0;
    int c;
    while (key[pos] != '\0') {
        c = key[pos] - 'a';
        if (t->nodes[cur].children[c] == -1) return 0;
        cur = t->nodes[cur].children[c];
        pos++;
    }
    return t->nodes[cur].is_end;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1059 radix tree failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1059: empty output");
    assert!(code.contains("fn tree_radix_insert"), "C1059: Should contain tree_radix_insert");
}

#[test]
fn c1060_suffix_tree() {
    let c_code = r#"
typedef struct {
    int children[27];
    int start;
    int end;
    int suffix_link;
} tree_SuffixNode;

typedef struct {
    tree_SuffixNode nodes[2048];
    int root;
    int next_free;
    int text_len;
} tree_SuffixTree;

void tree_suffix_init(tree_SuffixTree *t) {
    int i;
    t->root = 0;
    t->next_free = 1;
    t->text_len = 0;
    t->nodes[0].start = -1;
    t->nodes[0].end = -1;
    t->nodes[0].suffix_link = -1;
    for (i = 0; i < 27; i++) t->nodes[0].children[i] = -1;
}

int tree_suffix_alloc(tree_SuffixTree *t, int start, int end) {
    int idx = t->next_free;
    int i;
    t->next_free++;
    t->nodes[idx].start = start;
    t->nodes[idx].end = end;
    t->nodes[idx].suffix_link = -1;
    for (i = 0; i < 27; i++) t->nodes[idx].children[i] = -1;
    return idx;
}

void tree_suffix_build_naive(tree_SuffixTree *t, const char *text) {
    int i;
    int len = 0;
    while (text[len] != '\0') len++;
    t->text_len = len;
    for (i = 0; i < len; i++) {
        int cur = t->root;
        int j = i;
        while (j < len) {
            int c = text[j] - 'a';
            if (c < 0 || c >= 26) c = 26;
            if (t->nodes[cur].children[c] == -1) {
                int new_node = tree_suffix_alloc(t, j, len - 1);
                t->nodes[cur].children[c] = new_node;
                break;
            }
            cur = t->nodes[cur].children[c];
            j++;
        }
    }
}

int tree_suffix_search(tree_SuffixTree *t, const char *text, const char *pattern) {
    int cur = t->root;
    int i = 0;
    int c;
    while (pattern[i] != '\0') {
        c = pattern[i] - 'a';
        if (c < 0 || c >= 26) c = 26;
        if (t->nodes[cur].children[c] == -1) return 0;
        cur = t->nodes[cur].children[c];
        i++;
    }
    return 1;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1060 suffix tree failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1060: empty output");
    assert!(code.contains("fn tree_suffix_build_naive"), "C1060: Should contain tree_suffix_build_naive");
}

// ============================================================================
// C1061-C1065: Randomized and Spatial Trees
// ============================================================================

#[test]
fn c1061_splay_tree() {
    let c_code = r#"
typedef struct {
    int key;
    int left;
    int right;
    int parent;
} tree_SplayNode;

typedef struct {
    tree_SplayNode nodes[1024];
    int root;
    int next_free;
} tree_SplayTree;

void tree_splay_init(tree_SplayTree *t) {
    t->root = -1;
    t->next_free = 0;
}

int tree_splay_alloc(tree_SplayTree *t, int key) {
    int idx = t->next_free;
    t->next_free++;
    t->nodes[idx].key = key;
    t->nodes[idx].left = -1;
    t->nodes[idx].right = -1;
    t->nodes[idx].parent = -1;
    return idx;
}

void tree_splay_rotate_right(tree_SplayTree *t, int x) {
    int y = t->nodes[x].parent;
    int g = t->nodes[y].parent;
    t->nodes[y].left = t->nodes[x].right;
    if (t->nodes[x].right != -1)
        t->nodes[t->nodes[x].right].parent = y;
    t->nodes[x].right = y;
    t->nodes[y].parent = x;
    t->nodes[x].parent = g;
    if (g != -1) {
        if (t->nodes[g].left == y)
            t->nodes[g].left = x;
        else
            t->nodes[g].right = x;
    }
}

void tree_splay_rotate_left(tree_SplayTree *t, int x) {
    int y = t->nodes[x].parent;
    int g = t->nodes[y].parent;
    t->nodes[y].right = t->nodes[x].left;
    if (t->nodes[x].left != -1)
        t->nodes[t->nodes[x].left].parent = y;
    t->nodes[x].left = y;
    t->nodes[y].parent = x;
    t->nodes[x].parent = g;
    if (g != -1) {
        if (t->nodes[g].left == y)
            t->nodes[g].left = x;
        else
            t->nodes[g].right = x;
    }
}

void tree_splay(tree_SplayTree *t, int x) {
    int p;
    int g;
    while (t->nodes[x].parent != -1) {
        p = t->nodes[x].parent;
        g = t->nodes[p].parent;
        if (g == -1) {
            if (t->nodes[p].left == x)
                tree_splay_rotate_right(t, x);
            else
                tree_splay_rotate_left(t, x);
        } else if (t->nodes[g].left == p && t->nodes[p].left == x) {
            tree_splay_rotate_right(t, p);
            tree_splay_rotate_right(t, x);
        } else if (t->nodes[g].right == p && t->nodes[p].right == x) {
            tree_splay_rotate_left(t, p);
            tree_splay_rotate_left(t, x);
        } else if (t->nodes[g].left == p && t->nodes[p].right == x) {
            tree_splay_rotate_left(t, x);
            tree_splay_rotate_right(t, x);
        } else {
            tree_splay_rotate_right(t, x);
            tree_splay_rotate_left(t, x);
        }
    }
    t->root = x;
}

void tree_splay_insert(tree_SplayTree *t, int key) {
    int cur = t->root;
    int par = -1;
    int node;
    while (cur != -1) {
        par = cur;
        if (key < t->nodes[cur].key)
            cur = t->nodes[cur].left;
        else if (key > t->nodes[cur].key)
            cur = t->nodes[cur].right;
        else {
            tree_splay(t, cur);
            return;
        }
    }
    node = tree_splay_alloc(t, key);
    t->nodes[node].parent = par;
    if (par == -1) {
        t->root = node;
    } else if (key < t->nodes[par].key) {
        t->nodes[par].left = node;
    } else {
        t->nodes[par].right = node;
    }
    tree_splay(t, node);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1061 splay tree failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1061: empty output");
    assert!(code.contains("fn tree_splay_insert"), "C1061: Should contain tree_splay_insert");
}

#[test]
fn c1062_treap() {
    let c_code = r#"
typedef struct {
    int key;
    int priority;
    int left;
    int right;
} tree_TreapNode;

typedef struct {
    tree_TreapNode nodes[1024];
    int root;
    int next_free;
    int rng_state;
} tree_Treap;

int tree_treap_rand(tree_Treap *t) {
    t->rng_state = t->rng_state * 1103515245 + 12345;
    return (t->rng_state >> 16) & 0x7FFF;
}

void tree_treap_init(tree_Treap *t) {
    t->root = -1;
    t->next_free = 0;
    t->rng_state = 42;
}

int tree_treap_alloc(tree_Treap *t, int key) {
    int idx = t->next_free;
    t->next_free++;
    t->nodes[idx].key = key;
    t->nodes[idx].priority = tree_treap_rand(t);
    t->nodes[idx].left = -1;
    t->nodes[idx].right = -1;
    return idx;
}

int tree_treap_rotate_right(tree_Treap *t, int node) {
    int left = t->nodes[node].left;
    t->nodes[node].left = t->nodes[left].right;
    t->nodes[left].right = node;
    return left;
}

int tree_treap_rotate_left(tree_Treap *t, int node) {
    int right = t->nodes[node].right;
    t->nodes[node].right = t->nodes[right].left;
    t->nodes[right].left = node;
    return right;
}

int tree_treap_insert_rec(tree_Treap *t, int node, int key) {
    int new_node;
    if (node == -1) return tree_treap_alloc(t, key);
    if (key < t->nodes[node].key) {
        t->nodes[node].left = tree_treap_insert_rec(t, t->nodes[node].left, key);
        if (t->nodes[t->nodes[node].left].priority > t->nodes[node].priority)
            node = tree_treap_rotate_right(t, node);
    } else if (key > t->nodes[node].key) {
        t->nodes[node].right = tree_treap_insert_rec(t, t->nodes[node].right, key);
        if (t->nodes[t->nodes[node].right].priority > t->nodes[node].priority)
            node = tree_treap_rotate_left(t, node);
    }
    return node;
}

void tree_treap_insert(tree_Treap *t, int key) {
    t->root = tree_treap_insert_rec(t, t->root, key);
}

int tree_treap_search(tree_Treap *t, int node, int key) {
    if (node == -1) return 0;
    if (key == t->nodes[node].key) return 1;
    if (key < t->nodes[node].key)
        return tree_treap_search(t, t->nodes[node].left, key);
    return tree_treap_search(t, t->nodes[node].right, key);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1062 treap failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1062: empty output");
    assert!(code.contains("fn tree_treap_insert"), "C1062: Should contain tree_treap_insert");
}

#[test]
fn c1063_kd_tree() {
    let c_code = r#"
typedef struct {
    int coords[2];
    int left;
    int right;
} tree_KdNode;

typedef struct {
    tree_KdNode nodes[1024];
    int root;
    int next_free;
} tree_KdTree;

void tree_kd_init(tree_KdTree *t) {
    t->root = -1;
    t->next_free = 0;
}

int tree_kd_alloc(tree_KdTree *t, int x, int y) {
    int idx = t->next_free;
    t->next_free++;
    t->nodes[idx].coords[0] = x;
    t->nodes[idx].coords[1] = y;
    t->nodes[idx].left = -1;
    t->nodes[idx].right = -1;
    return idx;
}

int tree_kd_insert_rec(tree_KdTree *t, int node, int x, int y, int depth) {
    int axis;
    if (node == -1) return tree_kd_alloc(t, x, y);
    axis = depth % 2;
    if (axis == 0) {
        if (x < t->nodes[node].coords[0])
            t->nodes[node].left = tree_kd_insert_rec(t, t->nodes[node].left, x, y, depth + 1);
        else
            t->nodes[node].right = tree_kd_insert_rec(t, t->nodes[node].right, x, y, depth + 1);
    } else {
        if (y < t->nodes[node].coords[1])
            t->nodes[node].left = tree_kd_insert_rec(t, t->nodes[node].left, x, y, depth + 1);
        else
            t->nodes[node].right = tree_kd_insert_rec(t, t->nodes[node].right, x, y, depth + 1);
    }
    return node;
}

void tree_kd_insert(tree_KdTree *t, int x, int y) {
    t->root = tree_kd_insert_rec(t, t->root, x, y, 0);
}

int tree_kd_abs(int x) {
    return x < 0 ? -x : x;
}

int tree_kd_dist_sq(tree_KdTree *t, int node, int x, int y) {
    int dx = t->nodes[node].coords[0] - x;
    int dy = t->nodes[node].coords[1] - y;
    return dx * dx + dy * dy;
}

int tree_kd_nearest(tree_KdTree *t, int node, int x, int y, int depth, int best) {
    int axis;
    int diff;
    int near_child;
    int far_child;
    int d;
    if (node == -1) return best;
    d = tree_kd_dist_sq(t, node, x, y);
    if (best == -1 || d < tree_kd_dist_sq(t, best, x, y))
        best = node;
    axis = depth % 2;
    if (axis == 0) diff = x - t->nodes[node].coords[0];
    else diff = y - t->nodes[node].coords[1];
    if (diff < 0) {
        near_child = t->nodes[node].left;
        far_child = t->nodes[node].right;
    } else {
        near_child = t->nodes[node].right;
        far_child = t->nodes[node].left;
    }
    best = tree_kd_nearest(t, near_child, x, y, depth + 1, best);
    if (diff * diff < tree_kd_dist_sq(t, best, x, y))
        best = tree_kd_nearest(t, far_child, x, y, depth + 1, best);
    return best;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1063 k-d tree failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1063: empty output");
    assert!(code.contains("fn tree_kd_insert"), "C1063: Should contain tree_kd_insert");
    assert!(code.contains("fn tree_kd_nearest"), "C1063: Should contain tree_kd_nearest");
}

#[test]
fn c1064_interval_tree() {
    let c_code = r#"
typedef struct {
    int low;
    int high;
    int max;
    int left;
    int right;
} tree_IntervalNode;

typedef struct {
    tree_IntervalNode nodes[1024];
    int root;
    int next_free;
} tree_IntervalTree;

void tree_interval_init(tree_IntervalTree *t) {
    t->root = -1;
    t->next_free = 0;
}

int tree_interval_max(int a, int b) {
    return a > b ? a : b;
}

int tree_interval_get_max(tree_IntervalTree *t, int node) {
    if (node == -1) return -1;
    return t->nodes[node].max;
}

void tree_interval_update_max(tree_IntervalTree *t, int node) {
    int lm = tree_interval_get_max(t, t->nodes[node].left);
    int rm = tree_interval_get_max(t, t->nodes[node].right);
    t->nodes[node].max = t->nodes[node].high;
    if (lm > t->nodes[node].max) t->nodes[node].max = lm;
    if (rm > t->nodes[node].max) t->nodes[node].max = rm;
}

int tree_interval_alloc(tree_IntervalTree *t, int low, int high) {
    int idx = t->next_free;
    t->next_free++;
    t->nodes[idx].low = low;
    t->nodes[idx].high = high;
    t->nodes[idx].max = high;
    t->nodes[idx].left = -1;
    t->nodes[idx].right = -1;
    return idx;
}

int tree_interval_insert(tree_IntervalTree *t, int node, int low, int high) {
    if (node == -1) return tree_interval_alloc(t, low, high);
    if (low < t->nodes[node].low)
        t->nodes[node].left = tree_interval_insert(t, t->nodes[node].left, low, high);
    else
        t->nodes[node].right = tree_interval_insert(t, t->nodes[node].right, low, high);
    tree_interval_update_max(t, node);
    return node;
}

void tree_interval_add(tree_IntervalTree *t, int low, int high) {
    t->root = tree_interval_insert(t, t->root, low, high);
}

int tree_interval_overlaps(int l1, int h1, int l2, int h2) {
    return l1 <= h2 && l2 <= h1;
}

int tree_interval_search(tree_IntervalTree *t, int node, int low, int high) {
    if (node == -1) return -1;
    if (tree_interval_overlaps(t->nodes[node].low, t->nodes[node].high, low, high))
        return node;
    if (t->nodes[node].left != -1 && tree_interval_get_max(t, t->nodes[node].left) >= low)
        return tree_interval_search(t, t->nodes[node].left, low, high);
    return tree_interval_search(t, t->nodes[node].right, low, high);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1064 interval tree failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1064: empty output");
    assert!(code.contains("fn tree_interval_add"), "C1064: Should contain tree_interval_add");
    assert!(code.contains("fn tree_interval_search"), "C1064: Should contain tree_interval_search");
}

#[test]
fn c1065_van_emde_boas() {
    let c_code = r#"
typedef struct {
    int min_val;
    int max_val;
    int universe;
    int summary[16];
    int clusters[16][16];
    int cluster_min[16];
    int cluster_max[16];
} tree_Veb;

int tree_veb_high(int x, int sqrt_u) {
    return x / sqrt_u;
}

int tree_veb_low(int x, int sqrt_u) {
    return x % sqrt_u;
}

int tree_veb_index(int h, int l, int sqrt_u) {
    return h * sqrt_u + l;
}

void tree_veb_init(tree_Veb *v, int universe) {
    int i;
    int j;
    v->universe = universe;
    v->min_val = -1;
    v->max_val = -1;
    for (i = 0; i < 16; i++) {
        v->summary[i] = 0;
        v->cluster_min[i] = -1;
        v->cluster_max[i] = -1;
        for (j = 0; j < 16; j++) {
            v->clusters[i][j] = 0;
        }
    }
}

int tree_veb_member(tree_Veb *v, int x) {
    int sqrt_u = 16;
    int h;
    int l;
    if (x == v->min_val || x == v->max_val) return 1;
    if (v->universe <= 2) return 0;
    h = tree_veb_high(x, sqrt_u);
    l = tree_veb_low(x, sqrt_u);
    return v->clusters[h][l];
}

void tree_veb_insert(tree_Veb *v, int x) {
    int sqrt_u = 16;
    int h;
    int l;
    int tmp;
    if (v->min_val == -1) {
        v->min_val = x;
        v->max_val = x;
        return;
    }
    if (x < v->min_val) {
        tmp = v->min_val;
        v->min_val = x;
        x = tmp;
    }
    h = tree_veb_high(x, sqrt_u);
    l = tree_veb_low(x, sqrt_u);
    if (v->cluster_min[h] == -1) {
        v->summary[h] = 1;
        v->cluster_min[h] = l;
        v->cluster_max[h] = l;
    }
    v->clusters[h][l] = 1;
    if (l < v->cluster_min[h]) v->cluster_min[h] = l;
    if (l > v->cluster_max[h]) v->cluster_max[h] = l;
    if (x > v->max_val) v->max_val = x;
}

int tree_veb_successor(tree_Veb *v, int x) {
    int sqrt_u = 16;
    int h;
    int l;
    int i;
    if (v->min_val != -1 && x < v->min_val) return v->min_val;
    h = tree_veb_high(x, sqrt_u);
    l = tree_veb_low(x, sqrt_u);
    if (v->cluster_max[h] != -1 && l < v->cluster_max[h]) {
        i = l + 1;
        while (i < 16 && !v->clusters[h][i]) i++;
        if (i < 16) return tree_veb_index(h, i, sqrt_u);
    }
    h = h + 1;
    while (h < 16 && !v->summary[h]) h++;
    if (h >= 16) return -1;
    return tree_veb_index(h, v->cluster_min[h], sqrt_u);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1065 Van Emde Boas failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1065: empty output");
    assert!(code.contains("fn tree_veb_insert"), "C1065: Should contain tree_veb_insert");
    assert!(code.contains("fn tree_veb_member"), "C1065: Should contain tree_veb_member");
}

// ============================================================================
// C1066-C1070: Specialized Trees
// ============================================================================

#[test]
fn c1066_merkle_tree() {
    let c_code = r#"
typedef unsigned int tree_uint32;

typedef struct {
    tree_uint32 hash;
    int left;
    int right;
    int data;
    int is_leaf;
} tree_MerkleNode;

typedef struct {
    tree_MerkleNode nodes[2048];
    int root;
    int next_free;
} tree_MerkleTree;

tree_uint32 tree_merkle_hash_combine(tree_uint32 a, tree_uint32 b) {
    tree_uint32 h = a;
    h = h ^ (b + 0x9e3779b9 + (h << 6) + (h >> 2));
    return h;
}

tree_uint32 tree_merkle_hash_data(int data) {
    tree_uint32 h = (tree_uint32)data;
    h = ((h >> 16) ^ h) * 0x45d9f3b;
    h = ((h >> 16) ^ h) * 0x45d9f3b;
    h = (h >> 16) ^ h;
    return h;
}

void tree_merkle_init(tree_MerkleTree *t) {
    t->root = -1;
    t->next_free = 0;
}

int tree_merkle_alloc_leaf(tree_MerkleTree *t, int data) {
    int idx = t->next_free;
    t->next_free++;
    t->nodes[idx].data = data;
    t->nodes[idx].hash = tree_merkle_hash_data(data);
    t->nodes[idx].left = -1;
    t->nodes[idx].right = -1;
    t->nodes[idx].is_leaf = 1;
    return idx;
}

int tree_merkle_alloc_internal(tree_MerkleTree *t, int left, int right) {
    int idx = t->next_free;
    t->next_free++;
    t->nodes[idx].left = left;
    t->nodes[idx].right = right;
    t->nodes[idx].is_leaf = 0;
    t->nodes[idx].data = 0;
    t->nodes[idx].hash = tree_merkle_hash_combine(t->nodes[left].hash, t->nodes[right].hash);
    return idx;
}

int tree_merkle_build(tree_MerkleTree *t, int *data, int start, int end) {
    int mid;
    int left;
    int right;
    if (start == end)
        return tree_merkle_alloc_leaf(t, data[start]);
    mid = (start + end) / 2;
    left = tree_merkle_build(t, data, start, mid);
    right = tree_merkle_build(t, data, mid + 1, end);
    return tree_merkle_alloc_internal(t, left, right);
}

void tree_merkle_construct(tree_MerkleTree *t, int *data, int n) {
    tree_merkle_init(t);
    if (n > 0)
        t->root = tree_merkle_build(t, data, 0, n - 1);
}

int tree_merkle_verify(tree_MerkleTree *t, int node) {
    tree_uint32 expected;
    if (node == -1) return 1;
    if (t->nodes[node].is_leaf) {
        return t->nodes[node].hash == tree_merkle_hash_data(t->nodes[node].data);
    }
    expected = tree_merkle_hash_combine(
        t->nodes[t->nodes[node].left].hash,
        t->nodes[t->nodes[node].right].hash);
    if (t->nodes[node].hash != expected) return 0;
    return tree_merkle_verify(t, t->nodes[node].left) &&
           tree_merkle_verify(t, t->nodes[node].right);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1066 Merkle tree failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1066: empty output");
    assert!(code.contains("fn tree_merkle_construct"), "C1066: Should contain tree_merkle_construct");
    assert!(code.contains("fn tree_merkle_verify"), "C1066: Should contain tree_merkle_verify");
}

#[test]
fn c1067_quadtree() {
    let c_code = r#"
typedef struct {
    int x;
    int y;
    int data;
} tree_QPoint;

typedef struct {
    int cx;
    int cy;
    int half;
    int children[4];
    tree_QPoint points[4];
    int num_points;
    int subdivided;
} tree_QuadNode;

typedef struct {
    tree_QuadNode nodes[1024];
    int root;
    int next_free;
} tree_Quadtree;

void tree_quad_init_node(tree_Quadtree *qt, int idx, int cx, int cy, int half) {
    int i;
    qt->nodes[idx].cx = cx;
    qt->nodes[idx].cy = cy;
    qt->nodes[idx].half = half;
    qt->nodes[idx].num_points = 0;
    qt->nodes[idx].subdivided = 0;
    for (i = 0; i < 4; i++) qt->nodes[idx].children[i] = -1;
}

void tree_quad_init(tree_Quadtree *qt, int size) {
    qt->next_free = 1;
    qt->root = 0;
    tree_quad_init_node(qt, 0, size / 2, size / 2, size / 2);
}

int tree_quad_contains(tree_Quadtree *qt, int node, int x, int y) {
    int cx = qt->nodes[node].cx;
    int cy = qt->nodes[node].cy;
    int h = qt->nodes[node].half;
    return x >= cx - h && x < cx + h && y >= cy - h && y < cy + h;
}

void tree_quad_subdivide(tree_Quadtree *qt, int node) {
    int cx = qt->nodes[node].cx;
    int cy = qt->nodes[node].cy;
    int h = qt->nodes[node].half / 2;
    int ne = qt->next_free++;
    int nw = qt->next_free++;
    int sw = qt->next_free++;
    int se = qt->next_free++;
    tree_quad_init_node(qt, ne, cx + h, cy - h, h);
    tree_quad_init_node(qt, nw, cx - h, cy - h, h);
    tree_quad_init_node(qt, sw, cx - h, cy + h, h);
    tree_quad_init_node(qt, se, cx + h, cy + h, h);
    qt->nodes[node].children[0] = ne;
    qt->nodes[node].children[1] = nw;
    qt->nodes[node].children[2] = sw;
    qt->nodes[node].children[3] = se;
    qt->nodes[node].subdivided = 1;
}

int tree_quad_insert(tree_Quadtree *qt, int node, int x, int y, int data) {
    int i;
    if (!tree_quad_contains(qt, node, x, y)) return 0;
    if (qt->nodes[node].num_points < 4 && !qt->nodes[node].subdivided) {
        int n = qt->nodes[node].num_points;
        qt->nodes[node].points[n].x = x;
        qt->nodes[node].points[n].y = y;
        qt->nodes[node].points[n].data = data;
        qt->nodes[node].num_points++;
        return 1;
    }
    if (!qt->nodes[node].subdivided)
        tree_quad_subdivide(qt, node);
    for (i = 0; i < 4; i++) {
        if (tree_quad_insert(qt, qt->nodes[node].children[i], x, y, data))
            return 1;
    }
    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1067 quadtree failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1067: empty output");
    assert!(code.contains("fn tree_quad_insert"), "C1067: Should contain tree_quad_insert");
}

#[test]
fn c1068_octree() {
    let c_code = r#"
typedef struct {
    int x;
    int y;
    int z;
    int data;
} tree_OctPoint;

typedef struct {
    int cx;
    int cy;
    int cz;
    int half;
    int children[8];
    tree_OctPoint point;
    int has_point;
    int subdivided;
} tree_OctNode;

typedef struct {
    tree_OctNode nodes[1024];
    int root;
    int next_free;
} tree_Octree;

void tree_oct_init_node(tree_Octree *ot, int idx, int cx, int cy, int cz, int half) {
    int i;
    ot->nodes[idx].cx = cx;
    ot->nodes[idx].cy = cy;
    ot->nodes[idx].cz = cz;
    ot->nodes[idx].half = half;
    ot->nodes[idx].has_point = 0;
    ot->nodes[idx].subdivided = 0;
    for (i = 0; i < 8; i++) ot->nodes[idx].children[i] = -1;
}

void tree_oct_init(tree_Octree *ot, int size) {
    ot->next_free = 1;
    ot->root = 0;
    tree_oct_init_node(ot, 0, size / 2, size / 2, size / 2, size / 2);
}

int tree_oct_contains(tree_Octree *ot, int node, int x, int y, int z) {
    int cx = ot->nodes[node].cx;
    int cy = ot->nodes[node].cy;
    int cz = ot->nodes[node].cz;
    int h = ot->nodes[node].half;
    return x >= cx - h && x < cx + h &&
           y >= cy - h && y < cy + h &&
           z >= cz - h && z < cz + h;
}

int tree_oct_get_octant(tree_Octree *ot, int node, int x, int y, int z) {
    int octant = 0;
    if (x >= ot->nodes[node].cx) octant = octant | 1;
    if (y >= ot->nodes[node].cy) octant = octant | 2;
    if (z >= ot->nodes[node].cz) octant = octant | 4;
    return octant;
}

void tree_oct_subdivide(tree_Octree *ot, int node) {
    int h = ot->nodes[node].half / 2;
    int cx = ot->nodes[node].cx;
    int cy = ot->nodes[node].cy;
    int cz = ot->nodes[node].cz;
    int i;
    for (i = 0; i < 8; i++) {
        int child = ot->next_free++;
        int dx = (i & 1) ? h : -h;
        int dy = (i & 2) ? h : -h;
        int dz = (i & 4) ? h : -h;
        tree_oct_init_node(ot, child, cx + dx, cy + dy, cz + dz, h);
        ot->nodes[node].children[i] = child;
    }
    ot->nodes[node].subdivided = 1;
}

int tree_oct_insert(tree_Octree *ot, int node, int x, int y, int z, int data) {
    int octant;
    if (!tree_oct_contains(ot, node, x, y, z)) return 0;
    if (!ot->nodes[node].has_point && !ot->nodes[node].subdivided) {
        ot->nodes[node].point.x = x;
        ot->nodes[node].point.y = y;
        ot->nodes[node].point.z = z;
        ot->nodes[node].point.data = data;
        ot->nodes[node].has_point = 1;
        return 1;
    }
    if (!ot->nodes[node].subdivided) {
        tree_oct_subdivide(ot, node);
        if (ot->nodes[node].has_point) {
            octant = tree_oct_get_octant(ot, node,
                ot->nodes[node].point.x,
                ot->nodes[node].point.y,
                ot->nodes[node].point.z);
            tree_oct_insert(ot, ot->nodes[node].children[octant],
                ot->nodes[node].point.x,
                ot->nodes[node].point.y,
                ot->nodes[node].point.z,
                ot->nodes[node].point.data);
            ot->nodes[node].has_point = 0;
        }
    }
    octant = tree_oct_get_octant(ot, node, x, y, z);
    return tree_oct_insert(ot, ot->nodes[node].children[octant], x, y, z, data);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1068 octree failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1068: empty output");
    assert!(code.contains("fn tree_oct_insert"), "C1068: Should contain tree_oct_insert");
}

#[test]
fn c1069_rope() {
    let c_code = r#"
typedef struct {
    char text[64];
    int text_len;
    int left;
    int right;
    int weight;
    int is_leaf;
} tree_RopeNode;

typedef struct {
    tree_RopeNode nodes[1024];
    int root;
    int next_free;
} tree_Rope;

void tree_rope_init(tree_Rope *r) {
    r->root = -1;
    r->next_free = 0;
}

int tree_rope_alloc_leaf(tree_Rope *r, const char *text, int len) {
    int idx = r->next_free;
    int i;
    r->next_free++;
    for (i = 0; i < len && i < 63; i++)
        r->nodes[idx].text[i] = text[i];
    r->nodes[idx].text[len] = '\0';
    r->nodes[idx].text_len = len;
    r->nodes[idx].weight = len;
    r->nodes[idx].left = -1;
    r->nodes[idx].right = -1;
    r->nodes[idx].is_leaf = 1;
    return idx;
}

int tree_rope_alloc_internal(tree_Rope *r, int left, int right) {
    int idx = r->next_free;
    r->next_free++;
    r->nodes[idx].left = left;
    r->nodes[idx].right = right;
    r->nodes[idx].is_leaf = 0;
    r->nodes[idx].text_len = 0;
    r->nodes[idx].weight = tree_rope_length(r, left);
    return idx;
}

int tree_rope_length(tree_Rope *r, int node) {
    if (node == -1) return 0;
    if (r->nodes[node].is_leaf) return r->nodes[node].text_len;
    return tree_rope_length(r, r->nodes[node].left) +
           tree_rope_length(r, r->nodes[node].right);
}

char tree_rope_index(tree_Rope *r, int node, int idx) {
    if (node == -1) return '\0';
    if (r->nodes[node].is_leaf) {
        if (idx < r->nodes[node].text_len)
            return r->nodes[node].text[idx];
        return '\0';
    }
    if (idx < r->nodes[node].weight)
        return tree_rope_index(r, r->nodes[node].left, idx);
    return tree_rope_index(r, r->nodes[node].right, idx - r->nodes[node].weight);
}

int tree_rope_concat(tree_Rope *r, int left, int right) {
    if (left == -1) return right;
    if (right == -1) return left;
    return tree_rope_alloc_internal(r, left, right);
}

void tree_rope_build(tree_Rope *r, const char *text) {
    int len = 0;
    int i;
    int left;
    int right;
    while (text[len] != '\0') len++;
    if (len <= 64) {
        r->root = tree_rope_alloc_leaf(r, text, len);
    } else {
        int mid = len / 2;
        left = tree_rope_alloc_leaf(r, text, mid);
        right = tree_rope_alloc_leaf(r, text + mid, len - mid);
        r->root = tree_rope_concat(r, left, right);
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1069 rope failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1069: empty output");
    assert!(code.contains("fn tree_rope_index"), "C1069: Should contain tree_rope_index");
    assert!(code.contains("fn tree_rope_concat"), "C1069: Should contain tree_rope_concat");
}

#[test]
fn c1070_persistent_segment_tree() {
    let c_code = r#"
typedef struct {
    int left;
    int right;
    int sum;
} tree_PersistNode;

typedef struct {
    tree_PersistNode nodes[8192];
    int roots[256];
    int num_versions;
    int next_free;
} tree_PersistSegTree;

void tree_persist_init(tree_PersistSegTree *t) {
    t->num_versions = 0;
    t->next_free = 0;
}

int tree_persist_alloc(tree_PersistSegTree *t, int left, int right, int sum) {
    int idx = t->next_free;
    t->next_free++;
    t->nodes[idx].left = left;
    t->nodes[idx].right = right;
    t->nodes[idx].sum = sum;
    return idx;
}

int tree_persist_build(tree_PersistSegTree *t, int *arr, int lo, int hi) {
    int mid;
    int left;
    int right;
    if (lo == hi)
        return tree_persist_alloc(t, -1, -1, arr[lo]);
    mid = (lo + hi) / 2;
    left = tree_persist_build(t, arr, lo, mid);
    right = tree_persist_build(t, arr, mid + 1, hi);
    return tree_persist_alloc(t, left, right,
        t->nodes[left].sum + t->nodes[right].sum);
}

int tree_persist_update(tree_PersistSegTree *t, int prev, int lo, int hi, int idx, int val) {
    int mid;
    int new_left;
    int new_right;
    if (lo == hi)
        return tree_persist_alloc(t, -1, -1, val);
    mid = (lo + hi) / 2;
    if (idx <= mid) {
        new_left = tree_persist_update(t, t->nodes[prev].left, lo, mid, idx, val);
        return tree_persist_alloc(t, new_left, t->nodes[prev].right,
            t->nodes[new_left].sum + t->nodes[t->nodes[prev].right].sum);
    } else {
        new_right = tree_persist_update(t, t->nodes[prev].right, mid + 1, hi, idx, val);
        return tree_persist_alloc(t, t->nodes[prev].left, new_right,
            t->nodes[t->nodes[prev].left].sum + t->nodes[new_right].sum);
    }
}

int tree_persist_query(tree_PersistSegTree *t, int node, int lo, int hi, int l, int r) {
    int mid;
    if (node == -1 || r < lo || hi < l) return 0;
    if (l <= lo && hi <= r) return t->nodes[node].sum;
    mid = (lo + hi) / 2;
    return tree_persist_query(t, t->nodes[node].left, lo, mid, l, r) +
           tree_persist_query(t, t->nodes[node].right, mid + 1, hi, l, r);
}

void tree_persist_create_version(tree_PersistSegTree *t, int *arr, int n) {
    t->roots[t->num_versions] = tree_persist_build(t, arr, 0, n - 1);
    t->num_versions++;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1070 persistent segment tree failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1070: empty output");
    assert!(code.contains("fn tree_persist_update"), "C1070: Should contain tree_persist_update");
    assert!(code.contains("fn tree_persist_query"), "C1070: Should contain tree_persist_query");
}

// ============================================================================
// C1071-C1075: Exotic Trees
// ============================================================================

#[test]
fn c1071_cartesian_tree() {
    let c_code = r#"
typedef struct {
    int value;
    int left;
    int right;
    int parent;
} tree_CartNode;

typedef struct {
    tree_CartNode nodes[1024];
    int root;
    int next_free;
} tree_CartTree;

void tree_cart_init(tree_CartTree *t) {
    t->root = -1;
    t->next_free = 0;
}

int tree_cart_alloc(tree_CartTree *t, int value) {
    int idx = t->next_free;
    t->next_free++;
    t->nodes[idx].value = value;
    t->nodes[idx].left = -1;
    t->nodes[idx].right = -1;
    t->nodes[idx].parent = -1;
    return idx;
}

void tree_cart_build(tree_CartTree *t, int *arr, int n) {
    int stack[1024];
    int top = -1;
    int i;
    tree_cart_init(t);
    for (i = 0; i < n; i++) {
        int node = tree_cart_alloc(t, arr[i]);
        int last_popped = -1;
        while (top >= 0 && t->nodes[stack[top]].value > arr[i]) {
            last_popped = stack[top];
            top--;
        }
        if (last_popped != -1) {
            t->nodes[node].left = last_popped;
            t->nodes[last_popped].parent = node;
        }
        if (top >= 0) {
            t->nodes[stack[top]].right = node;
            t->nodes[node].parent = stack[top];
        } else {
            t->root = node;
        }
        top++;
        stack[top] = node;
    }
}

int tree_cart_is_min_heap(tree_CartTree *t, int node) {
    if (node == -1) return 1;
    if (t->nodes[node].left != -1 &&
        t->nodes[t->nodes[node].left].value < t->nodes[node].value)
        return 0;
    if (t->nodes[node].right != -1 &&
        t->nodes[t->nodes[node].right].value < t->nodes[node].value)
        return 0;
    return tree_cart_is_min_heap(t, t->nodes[node].left) &&
           tree_cart_is_min_heap(t, t->nodes[node].right);
}

void tree_cart_inorder(tree_CartTree *t, int node, int *out, int *idx) {
    if (node == -1) return;
    tree_cart_inorder(t, t->nodes[node].left, out, idx);
    out[*idx] = t->nodes[node].value;
    (*idx)++;
    tree_cart_inorder(t, t->nodes[node].right, out, idx);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1071 Cartesian tree failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1071: empty output");
    assert!(code.contains("fn tree_cart_build"), "C1071: Should contain tree_cart_build");
}

#[test]
fn c1072_scapegoat_tree() {
    let c_code = r#"
typedef struct {
    int key;
    int left;
    int right;
    int size;
} tree_SgNode;

typedef struct {
    tree_SgNode nodes[1024];
    int root;
    int next_free;
    int total_nodes;
    int max_nodes;
} tree_SgTree;

void tree_sg_init(tree_SgTree *t) {
    t->root = -1;
    t->next_free = 0;
    t->total_nodes = 0;
    t->max_nodes = 0;
}

int tree_sg_alloc(tree_SgTree *t, int key) {
    int idx = t->next_free;
    t->next_free++;
    t->nodes[idx].key = key;
    t->nodes[idx].left = -1;
    t->nodes[idx].right = -1;
    t->nodes[idx].size = 1;
    t->total_nodes++;
    if (t->total_nodes > t->max_nodes)
        t->max_nodes = t->total_nodes;
    return idx;
}

int tree_sg_size(tree_SgTree *t, int node) {
    if (node == -1) return 0;
    return t->nodes[node].size;
}

void tree_sg_update_size(tree_SgTree *t, int node) {
    if (node == -1) return;
    t->nodes[node].size = 1 + tree_sg_size(t, t->nodes[node].left) +
                               tree_sg_size(t, t->nodes[node].right);
}

int tree_sg_height(tree_SgTree *t, int node) {
    int lh;
    int rh;
    if (node == -1) return 0;
    lh = tree_sg_height(t, t->nodes[node].left);
    rh = tree_sg_height(t, t->nodes[node].right);
    return 1 + (lh > rh ? lh : rh);
}

int tree_sg_is_scapegoat(tree_SgTree *t, int node) {
    int sz = tree_sg_size(t, node);
    int lsz = tree_sg_size(t, t->nodes[node].left);
    int rsz = tree_sg_size(t, t->nodes[node].right);
    /* alpha = 2/3 approximation: child > 2/3 * parent */
    return lsz * 3 > sz * 2 || rsz * 3 > sz * 2;
}

void tree_sg_flatten(tree_SgTree *t, int node, int *arr, int *idx) {
    if (node == -1) return;
    tree_sg_flatten(t, t->nodes[node].left, arr, idx);
    arr[*idx] = node;
    (*idx)++;
    tree_sg_flatten(t, t->nodes[node].right, arr, idx);
}

int tree_sg_build_balanced(tree_SgTree *t, int *arr, int lo, int hi) {
    int mid;
    if (lo > hi) return -1;
    mid = (lo + hi) / 2;
    t->nodes[arr[mid]].left = tree_sg_build_balanced(t, arr, lo, mid - 1);
    t->nodes[arr[mid]].right = tree_sg_build_balanced(t, arr, mid + 1, hi);
    tree_sg_update_size(t, arr[mid]);
    return arr[mid];
}

int tree_sg_rebuild(tree_SgTree *t, int node) {
    int arr[1024];
    int count = 0;
    tree_sg_flatten(t, node, arr, &count);
    return tree_sg_build_balanced(t, arr, 0, count - 1);
}

int tree_sg_insert_rec(tree_SgTree *t, int node, int key, int depth) {
    if (node == -1) return tree_sg_alloc(t, key);
    if (key < t->nodes[node].key)
        t->nodes[node].left = tree_sg_insert_rec(t, t->nodes[node].left, key, depth + 1);
    else
        t->nodes[node].right = tree_sg_insert_rec(t, t->nodes[node].right, key, depth + 1);
    tree_sg_update_size(t, node);
    if (tree_sg_is_scapegoat(t, node))
        node = tree_sg_rebuild(t, node);
    return node;
}

void tree_sg_insert(tree_SgTree *t, int key) {
    t->root = tree_sg_insert_rec(t, t->root, key, 0);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1072 scapegoat tree failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1072: empty output");
    assert!(code.contains("fn tree_sg_insert"), "C1072: Should contain tree_sg_insert");
    assert!(code.contains("fn tree_sg_rebuild"), "C1072: Should contain tree_sg_rebuild");
}

#[test]
fn c1073_aa_tree() {
    let c_code = r#"
typedef struct {
    int key;
    int level;
    int left;
    int right;
} tree_AaNode;

typedef struct {
    tree_AaNode nodes[1024];
    int root;
    int nil;
    int next_free;
} tree_AaTree;

void tree_aa_init(tree_AaTree *t) {
    t->nil = 0;
    t->nodes[0].key = 0;
    t->nodes[0].level = 0;
    t->nodes[0].left = 0;
    t->nodes[0].right = 0;
    t->root = 0;
    t->next_free = 1;
}

int tree_aa_alloc(tree_AaTree *t, int key) {
    int idx = t->next_free;
    t->next_free++;
    t->nodes[idx].key = key;
    t->nodes[idx].level = 1;
    t->nodes[idx].left = t->nil;
    t->nodes[idx].right = t->nil;
    return idx;
}

int tree_aa_skew(tree_AaTree *t, int node) {
    int left;
    if (node == t->nil) return node;
    left = t->nodes[node].left;
    if (left == t->nil) return node;
    if (t->nodes[left].level == t->nodes[node].level) {
        t->nodes[node].left = t->nodes[left].right;
        t->nodes[left].right = node;
        return left;
    }
    return node;
}

int tree_aa_split(tree_AaTree *t, int node) {
    int right;
    int rr;
    if (node == t->nil) return node;
    right = t->nodes[node].right;
    if (right == t->nil) return node;
    rr = t->nodes[right].right;
    if (rr != t->nil && t->nodes[rr].level == t->nodes[node].level) {
        t->nodes[node].right = t->nodes[right].left;
        t->nodes[right].left = node;
        t->nodes[right].level++;
        return right;
    }
    return node;
}

int tree_aa_insert_rec(tree_AaTree *t, int node, int key) {
    if (node == t->nil) return tree_aa_alloc(t, key);
    if (key < t->nodes[node].key)
        t->nodes[node].left = tree_aa_insert_rec(t, t->nodes[node].left, key);
    else if (key > t->nodes[node].key)
        t->nodes[node].right = tree_aa_insert_rec(t, t->nodes[node].right, key);
    else
        return node;
    node = tree_aa_skew(t, node);
    node = tree_aa_split(t, node);
    return node;
}

void tree_aa_insert(tree_AaTree *t, int key) {
    t->root = tree_aa_insert_rec(t, t->root, key);
}

int tree_aa_search(tree_AaTree *t, int node, int key) {
    if (node == t->nil) return 0;
    if (key == t->nodes[node].key) return 1;
    if (key < t->nodes[node].key)
        return tree_aa_search(t, t->nodes[node].left, key);
    return tree_aa_search(t, t->nodes[node].right, key);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1073 AA tree failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1073: empty output");
    assert!(code.contains("fn tree_aa_insert"), "C1073: Should contain tree_aa_insert");
    assert!(code.contains("fn tree_aa_skew"), "C1073: Should contain tree_aa_skew");
    assert!(code.contains("fn tree_aa_split"), "C1073: Should contain tree_aa_split");
}

#[test]
fn c1074_weight_balanced_tree() {
    let c_code = r#"
typedef struct {
    int key;
    int left;
    int right;
    int size;
} tree_WbNode;

typedef struct {
    tree_WbNode nodes[1024];
    int root;
    int next_free;
} tree_WbTree;

void tree_wb_init(tree_WbTree *t) {
    t->root = -1;
    t->next_free = 0;
}

int tree_wb_alloc(tree_WbTree *t, int key) {
    int idx = t->next_free;
    t->next_free++;
    t->nodes[idx].key = key;
    t->nodes[idx].left = -1;
    t->nodes[idx].right = -1;
    t->nodes[idx].size = 1;
    return idx;
}

int tree_wb_size(tree_WbTree *t, int node) {
    if (node == -1) return 0;
    return t->nodes[node].size;
}

void tree_wb_update_size(tree_WbTree *t, int node) {
    if (node == -1) return;
    t->nodes[node].size = 1 + tree_wb_size(t, t->nodes[node].left) +
                               tree_wb_size(t, t->nodes[node].right);
}

int tree_wb_rotate_right(tree_WbTree *t, int node) {
    int left = t->nodes[node].left;
    t->nodes[node].left = t->nodes[left].right;
    t->nodes[left].right = node;
    tree_wb_update_size(t, node);
    tree_wb_update_size(t, left);
    return left;
}

int tree_wb_rotate_left(tree_WbTree *t, int node) {
    int right = t->nodes[node].right;
    t->nodes[node].right = t->nodes[right].left;
    t->nodes[right].left = node;
    tree_wb_update_size(t, node);
    tree_wb_update_size(t, right);
    return right;
}

int tree_wb_is_balanced(tree_WbTree *t, int node) {
    int total;
    int ls;
    int rs;
    if (node == -1) return 1;
    total = t->nodes[node].size;
    ls = tree_wb_size(t, t->nodes[node].left);
    rs = tree_wb_size(t, t->nodes[node].right);
    /* Weight balance: each subtree is at most 70% of total (alpha ~ 0.29) */
    /* Using integer arithmetic: size * 10 <= total * 7 */
    return ls * 10 <= total * 7 && rs * 10 <= total * 7;
}

int tree_wb_balance(tree_WbTree *t, int node) {
    int ls;
    int rs;
    int total;
    if (node == -1) return node;
    tree_wb_update_size(t, node);
    total = t->nodes[node].size;
    ls = tree_wb_size(t, t->nodes[node].left);
    rs = tree_wb_size(t, t->nodes[node].right);
    if (ls * 10 > total * 7) {
        int ll = tree_wb_size(t, t->nodes[t->nodes[node].left].left);
        int lr = tree_wb_size(t, t->nodes[t->nodes[node].left].right);
        if (lr > ll)
            t->nodes[node].left = tree_wb_rotate_left(t, t->nodes[node].left);
        return tree_wb_rotate_right(t, node);
    }
    if (rs * 10 > total * 7) {
        int rl = tree_wb_size(t, t->nodes[t->nodes[node].right].left);
        int rr = tree_wb_size(t, t->nodes[t->nodes[node].right].right);
        if (rl > rr)
            t->nodes[node].right = tree_wb_rotate_right(t, t->nodes[node].right);
        return tree_wb_rotate_left(t, node);
    }
    return node;
}

int tree_wb_insert_rec(tree_WbTree *t, int node, int key) {
    if (node == -1) return tree_wb_alloc(t, key);
    if (key < t->nodes[node].key)
        t->nodes[node].left = tree_wb_insert_rec(t, t->nodes[node].left, key);
    else if (key > t->nodes[node].key)
        t->nodes[node].right = tree_wb_insert_rec(t, t->nodes[node].right, key);
    else
        return node;
    return tree_wb_balance(t, node);
}

void tree_wb_insert(tree_WbTree *t, int key) {
    t->root = tree_wb_insert_rec(t, t->root, key);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1074 weight-balanced tree failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1074: empty output");
    assert!(code.contains("fn tree_wb_insert"), "C1074: Should contain tree_wb_insert");
    assert!(code.contains("fn tree_wb_balance"), "C1074: Should contain tree_wb_balance");
}

#[test]
fn c1075_link_cut_tree() {
    let c_code = r#"
typedef struct {
    int left;
    int right;
    int parent;
    int val;
    int sum;
    int reversed;
} tree_LctNode;

typedef struct {
    tree_LctNode nodes[1024];
    int next_free;
} tree_Lct;

void tree_lct_init(tree_Lct *t) {
    t->next_free = 0;
}

int tree_lct_alloc(tree_Lct *t, int val) {
    int idx = t->next_free;
    t->next_free++;
    t->nodes[idx].left = -1;
    t->nodes[idx].right = -1;
    t->nodes[idx].parent = -1;
    t->nodes[idx].val = val;
    t->nodes[idx].sum = val;
    t->nodes[idx].reversed = 0;
    return idx;
}

void tree_lct_push_up(tree_Lct *t, int x) {
    int ls = 0;
    int rs = 0;
    if (x == -1) return;
    if (t->nodes[x].left != -1) ls = t->nodes[t->nodes[x].left].sum;
    if (t->nodes[x].right != -1) rs = t->nodes[t->nodes[x].right].sum;
    t->nodes[x].sum = ls + rs + t->nodes[x].val;
}

void tree_lct_push_down(tree_Lct *t, int x) {
    int tmp;
    if (x == -1) return;
    if (t->nodes[x].reversed) {
        tmp = t->nodes[x].left;
        t->nodes[x].left = t->nodes[x].right;
        t->nodes[x].right = tmp;
        if (t->nodes[x].left != -1)
            t->nodes[t->nodes[x].left].reversed = !t->nodes[t->nodes[x].left].reversed;
        if (t->nodes[x].right != -1)
            t->nodes[t->nodes[x].right].reversed = !t->nodes[t->nodes[x].right].reversed;
        t->nodes[x].reversed = 0;
    }
}

int tree_lct_is_root(tree_Lct *t, int x) {
    int p = t->nodes[x].parent;
    if (p == -1) return 1;
    if (t->nodes[p].left != x && t->nodes[p].right != x) return 1;
    return 0;
}

void tree_lct_rotate(tree_Lct *t, int x) {
    int y = t->nodes[x].parent;
    int z = t->nodes[y].parent;
    int is_left = (t->nodes[y].left == x);
    int child;
    if (is_left) {
        child = t->nodes[x].right;
        t->nodes[y].left = child;
        t->nodes[x].right = y;
    } else {
        child = t->nodes[x].left;
        t->nodes[y].right = child;
        t->nodes[x].left = y;
    }
    if (child != -1) t->nodes[child].parent = y;
    t->nodes[y].parent = x;
    t->nodes[x].parent = z;
    if (z != -1) {
        if (t->nodes[z].left == y) t->nodes[z].left = x;
        else if (t->nodes[z].right == y) t->nodes[z].right = x;
    }
    tree_lct_push_up(t, y);
    tree_lct_push_up(t, x);
}

void tree_lct_splay(tree_Lct *t, int x) {
    int y;
    int z;
    tree_lct_push_down(t, x);
    while (!tree_lct_is_root(t, x)) {
        y = t->nodes[x].parent;
        if (!tree_lct_is_root(t, y)) {
            z = t->nodes[y].parent;
            tree_lct_push_down(t, z);
            tree_lct_push_down(t, y);
            tree_lct_push_down(t, x);
            if ((t->nodes[z].left == y) == (t->nodes[y].left == x))
                tree_lct_rotate(t, y);
            else
                tree_lct_rotate(t, x);
        } else {
            tree_lct_push_down(t, y);
            tree_lct_push_down(t, x);
        }
        tree_lct_rotate(t, x);
    }
}

void tree_lct_access(tree_Lct *t, int x) {
    int last = -1;
    int cur = x;
    while (cur != -1) {
        tree_lct_splay(t, cur);
        t->nodes[cur].right = last;
        tree_lct_push_up(t, cur);
        last = cur;
        cur = t->nodes[cur].parent;
    }
    tree_lct_splay(t, x);
}

void tree_lct_make_root(tree_Lct *t, int x) {
    tree_lct_access(t, x);
    t->nodes[x].reversed = !t->nodes[x].reversed;
    tree_lct_push_down(t, x);
}

void tree_lct_link(tree_Lct *t, int x, int y) {
    tree_lct_make_root(t, x);
    t->nodes[x].parent = y;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1075 link-cut tree failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1075: empty output");
    assert!(code.contains("fn tree_lct_splay"), "C1075: Should contain tree_lct_splay");
    assert!(code.contains("fn tree_lct_link"), "C1075: Should contain tree_lct_link");
}
