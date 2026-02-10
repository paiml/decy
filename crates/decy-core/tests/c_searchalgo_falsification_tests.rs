//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1151-C1175: Search Algorithm implementations -- the kind of C code found
//! in textbooks (CLRS, Sedgewick), competitive programming, and systems software.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise classic and advanced search patterns commonly
//! found in real-world C codebases -- all expressed as valid C99 without #include.
//!
//! Organization:
//! - C1151-C1155: Basic search (binary search, interpolation search, exponential search, jump search, ternary search)
//! - C1156-C1160: Tree search (BST lookup, AVL search, B-tree search, trie search, red-black lookup)
//! - C1161-C1165: Graph search (BFS pathfinding, DFS cycle finder, bidirectional BFS, iterative deepening, beam search)
//! - C1166-C1170: Heuristic search (A* grid, hill climbing, simulated annealing, tabu search, genetic algorithm selection)
//! - C1171-C1175: Specialized search (binary search on answer, two pointer search, sliding window max, median of medians, kth element quickselect)

use decy_core::transpile;

// ============================================================================
// C1151-C1155: Basic Search
// ============================================================================

#[test]
fn c1151_binary_search() {
    let c_code = r#"
int srch_binary(int *arr, int n, int target) {
    int low = 0, high = n - 1;
    while (low <= high) {
        int mid = low + (high - low) / 2;
        if (arr[mid] == target) return mid;
        if (arr[mid] < target) low = mid + 1;
        else high = mid - 1;
    }
    return -1;
}

int srch_binary_first(int *arr, int n, int target) {
    int low = 0, high = n - 1, result = -1;
    while (low <= high) {
        int mid = low + (high - low) / 2;
        if (arr[mid] == target) {
            result = mid;
            high = mid - 1;
        } else if (arr[mid] < target) {
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }
    return result;
}

int srch_binary_last(int *arr, int n, int target) {
    int low = 0, high = n - 1, result = -1;
    while (low <= high) {
        int mid = low + (high - low) / 2;
        if (arr[mid] == target) {
            result = mid;
            low = mid + 1;
        } else if (arr[mid] < target) {
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }
    return result;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1151 binary search failed: {:?}", result.err());
}

#[test]
fn c1152_interpolation_search() {
    let c_code = r#"
int srch_interpolation(int *arr, int n, int target) {
    int low = 0, high = n - 1;
    while (low <= high && target >= arr[low] && target <= arr[high]) {
        if (low == high) {
            if (arr[low] == target) return low;
            return -1;
        }
        int range = arr[high] - arr[low];
        int pos;
        if (range == 0) {
            pos = low;
        } else {
            pos = low + ((target - arr[low]) * (high - low)) / range;
        }
        if (pos < low || pos > high) return -1;
        if (arr[pos] == target) return pos;
        if (arr[pos] < target) low = pos + 1;
        else high = pos - 1;
    }
    return -1;
}

int srch_interpolation_count(int *arr, int n, int target) {
    int first = -1, last = -1;
    int low = 0, high = n - 1;
    while (low <= high) {
        int mid = low + (high - low) / 2;
        if (arr[mid] == target) {
            first = mid;
            high = mid - 1;
        } else if (arr[mid] < target) low = mid + 1;
        else high = mid - 1;
    }
    if (first == -1) return 0;
    low = first;
    high = n - 1;
    while (low <= high) {
        int mid = low + (high - low) / 2;
        if (arr[mid] == target) {
            last = mid;
            low = mid + 1;
        } else if (arr[mid] < target) low = mid + 1;
        else high = mid - 1;
    }
    return last - first + 1;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1152 interpolation search failed: {:?}", result.err());
}

#[test]
fn c1153_exponential_search() {
    let c_code = r#"
int srch_exp_binary(int *arr, int low, int high, int target) {
    while (low <= high) {
        int mid = low + (high - low) / 2;
        if (arr[mid] == target) return mid;
        if (arr[mid] < target) low = mid + 1;
        else high = mid - 1;
    }
    return -1;
}

int srch_exponential(int *arr, int n, int target) {
    if (n == 0) return -1;
    if (arr[0] == target) return 0;
    int bound = 1;
    while (bound < n && arr[bound] <= target) {
        bound = bound * 2;
    }
    int low = bound / 2;
    int high = bound;
    if (high >= n) high = n - 1;
    return srch_exp_binary(arr, low, high, target);
}

int srch_exp_lower_bound(int *arr, int n, int target) {
    if (n == 0) return -1;
    int bound = 1;
    while (bound < n && arr[bound] < target) {
        bound = bound * 2;
    }
    int low = bound / 2;
    int high = bound;
    if (high >= n) high = n - 1;
    int result = -1;
    while (low <= high) {
        int mid = low + (high - low) / 2;
        if (arr[mid] >= target) {
            result = mid;
            high = mid - 1;
        } else {
            low = mid + 1;
        }
    }
    return result;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1153 exponential search failed: {:?}", result.err());
}

#[test]
fn c1154_jump_search() {
    let c_code = r#"
int srch_jump_sqrt(int n) {
    int s = 1;
    while (s * s <= n) s++;
    return s - 1;
}

int srch_jump(int *arr, int n, int target) {
    int step = srch_jump_sqrt(n);
    if (step == 0) step = 1;
    int prev = 0;
    int curr = step;
    while (curr < n && arr[curr] < target) {
        prev = curr;
        curr = curr + step;
    }
    if (curr >= n) curr = n - 1;
    int i;
    for (i = prev; i <= curr; i++) {
        if (arr[i] == target) return i;
    }
    return -1;
}

int srch_jump_block(int *arr, int n, int target, int block_size) {
    int block_start = 0;
    while (block_start < n && arr[block_start + block_size - 1 < n - 1 ? block_start + block_size - 1 : n - 1] < target) {
        block_start = block_start + block_size;
    }
    int i;
    int end = block_start + block_size;
    if (end > n) end = n;
    for (i = block_start; i < end; i++) {
        if (arr[i] == target) return i;
    }
    return -1;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1154 jump search failed: {:?}", result.err());
}

#[test]
fn c1155_ternary_search() {
    let c_code = r#"
int srch_ternary(int *arr, int n, int target) {
    int low = 0, high = n - 1;
    while (low <= high) {
        int third = (high - low) / 3;
        int mid1 = low + third;
        int mid2 = high - third;
        if (arr[mid1] == target) return mid1;
        if (arr[mid2] == target) return mid2;
        if (target < arr[mid1]) {
            high = mid1 - 1;
        } else if (target > arr[mid2]) {
            low = mid2 + 1;
        } else {
            low = mid1 + 1;
            high = mid2 - 1;
        }
    }
    return -1;
}

int srch_ternary_max(int *arr, int low, int high) {
    while (high - low > 2) {
        int third = (high - low) / 3;
        int m1 = low + third;
        int m2 = high - third;
        if (arr[m1] < arr[m2]) {
            low = m1 + 1;
        } else {
            high = m2 - 1;
        }
    }
    int max_val = arr[low];
    int max_idx = low;
    int i;
    for (i = low + 1; i <= high; i++) {
        if (arr[i] > max_val) {
            max_val = arr[i];
            max_idx = i;
        }
    }
    return max_idx;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1155 ternary search failed: {:?}", result.err());
}

// ============================================================================
// C1156-C1160: Tree Search
// ============================================================================

#[test]
fn c1156_bst_lookup() {
    let c_code = r#"
typedef struct {
    int key;
    int value;
    int left;
    int right;
} srch_bst_node_t;

typedef struct {
    srch_bst_node_t nodes[512];
    int root;
    int count;
} srch_bst_t;

void srch_bst_init(srch_bst_t *tree) {
    tree->root = -1;
    tree->count = 0;
}

int srch_bst_insert(srch_bst_t *tree, int key, int value) {
    int idx = tree->count;
    if (idx >= 512) return -1;
    tree->nodes[idx].key = key;
    tree->nodes[idx].value = value;
    tree->nodes[idx].left = -1;
    tree->nodes[idx].right = -1;
    tree->count++;
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
        } else if (key > tree->nodes[cur].key) {
            if (tree->nodes[cur].right == -1) {
                tree->nodes[cur].right = idx;
                return idx;
            }
            cur = tree->nodes[cur].right;
        } else {
            tree->nodes[cur].value = value;
            tree->count--;
            return cur;
        }
    }
}

int srch_bst_search(const srch_bst_t *tree, int key) {
    int cur = tree->root;
    while (cur != -1) {
        if (key == tree->nodes[cur].key) return tree->nodes[cur].value;
        if (key < tree->nodes[cur].key) cur = tree->nodes[cur].left;
        else cur = tree->nodes[cur].right;
    }
    return -1;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1156 BST lookup failed: {:?}", result.err());
}

#[test]
fn c1157_avl_search() {
    let c_code = r#"
typedef struct {
    int key;
    int height;
    int left;
    int right;
} srch_avl_node_t;

typedef struct {
    srch_avl_node_t nodes[512];
    int root;
    int count;
} srch_avl_t;

void srch_avl_init(srch_avl_t *tree) {
    tree->root = -1;
    tree->count = 0;
}

int srch_avl_height(const srch_avl_t *tree, int idx) {
    if (idx == -1) return 0;
    return tree->nodes[idx].height;
}

int srch_avl_balance(const srch_avl_t *tree, int idx) {
    if (idx == -1) return 0;
    return srch_avl_height(tree, tree->nodes[idx].left) -
           srch_avl_height(tree, tree->nodes[idx].right);
}

int srch_avl_update_height(srch_avl_t *tree, int idx) {
    int lh = srch_avl_height(tree, tree->nodes[idx].left);
    int rh = srch_avl_height(tree, tree->nodes[idx].right);
    tree->nodes[idx].height = (lh > rh ? lh : rh) + 1;
    return tree->nodes[idx].height;
}

int srch_avl_search(const srch_avl_t *tree, int key) {
    int cur = tree->root;
    while (cur != -1) {
        if (key == tree->nodes[cur].key) return cur;
        if (key < tree->nodes[cur].key) cur = tree->nodes[cur].left;
        else cur = tree->nodes[cur].right;
    }
    return -1;
}

int srch_avl_find_min(const srch_avl_t *tree, int idx) {
    if (idx == -1) return -1;
    while (tree->nodes[idx].left != -1) {
        idx = tree->nodes[idx].left;
    }
    return idx;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1157 AVL search failed: {:?}", result.err());
}

#[test]
fn c1158_btree_search() {
    let c_code = r#"
#define SRCH_BT_ORDER 4
#define SRCH_BT_MAX_KEYS (SRCH_BT_ORDER - 1)

typedef struct {
    int keys[SRCH_BT_MAX_KEYS];
    int children[SRCH_BT_ORDER];
    int num_keys;
    int is_leaf;
} srch_btree_node_t;

typedef struct {
    srch_btree_node_t nodes[256];
    int root;
    int count;
} srch_btree_t;

void srch_btree_init(srch_btree_t *tree) {
    tree->root = -1;
    tree->count = 0;
}

int srch_btree_search_node(const srch_btree_t *tree, int node_idx, int key) {
    int i;
    if (node_idx == -1) return -1;
    for (i = 0; i < tree->nodes[node_idx].num_keys; i++) {
        if (key == tree->nodes[node_idx].keys[i]) return node_idx;
        if (key < tree->nodes[node_idx].keys[i]) {
            if (tree->nodes[node_idx].is_leaf) return -1;
            return srch_btree_search_node(tree, tree->nodes[node_idx].children[i], key);
        }
    }
    if (tree->nodes[node_idx].is_leaf) return -1;
    return srch_btree_search_node(tree, tree->nodes[node_idx].children[i], key);
}

int srch_btree_search(const srch_btree_t *tree, int key) {
    return srch_btree_search_node(tree, tree->root, key);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1158 B-tree search failed: {:?}", result.err());
}

#[test]
fn c1159_trie_search() {
    let c_code = r#"
#define SRCH_TRIE_ALPHA 26
#define SRCH_TRIE_MAX 1024

typedef struct {
    int children[SRCH_TRIE_ALPHA];
    int is_end;
    int count;
} srch_trie_node_t;

typedef struct {
    srch_trie_node_t nodes[SRCH_TRIE_MAX];
    int num_nodes;
} srch_trie_t;

void srch_trie_init(srch_trie_t *trie) {
    int i;
    trie->num_nodes = 1;
    trie->nodes[0].is_end = 0;
    trie->nodes[0].count = 0;
    for (i = 0; i < SRCH_TRIE_ALPHA; i++) {
        trie->nodes[0].children[i] = -1;
    }
}

void srch_trie_insert(srch_trie_t *trie, const char *word) {
    int cur = 0;
    int i = 0;
    while (word[i] != '\0') {
        int c = word[i] - 'a';
        if (c < 0 || c >= SRCH_TRIE_ALPHA) { i++; continue; }
        if (trie->nodes[cur].children[c] == -1) {
            int new_node = trie->num_nodes++;
            if (new_node >= SRCH_TRIE_MAX) return;
            int j;
            for (j = 0; j < SRCH_TRIE_ALPHA; j++) {
                trie->nodes[new_node].children[j] = -1;
            }
            trie->nodes[new_node].is_end = 0;
            trie->nodes[new_node].count = 0;
            trie->nodes[cur].children[c] = new_node;
        }
        cur = trie->nodes[cur].children[c];
        trie->nodes[cur].count++;
        i++;
    }
    trie->nodes[cur].is_end = 1;
}

int srch_trie_search(const srch_trie_t *trie, const char *word) {
    int cur = 0;
    int i = 0;
    while (word[i] != '\0') {
        int c = word[i] - 'a';
        if (c < 0 || c >= SRCH_TRIE_ALPHA) return 0;
        if (trie->nodes[cur].children[c] == -1) return 0;
        cur = trie->nodes[cur].children[c];
        i++;
    }
    return trie->nodes[cur].is_end;
}

int srch_trie_prefix_count(const srch_trie_t *trie, const char *prefix) {
    int cur = 0;
    int i = 0;
    while (prefix[i] != '\0') {
        int c = prefix[i] - 'a';
        if (c < 0 || c >= SRCH_TRIE_ALPHA) return 0;
        if (trie->nodes[cur].children[c] == -1) return 0;
        cur = trie->nodes[cur].children[c];
        i++;
    }
    return trie->nodes[cur].count;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1159 trie search failed: {:?}", result.err());
}

#[test]
fn c1160_redblack_lookup() {
    let c_code = r#"
#define SRCH_RB_RED 0
#define SRCH_RB_BLACK 1

typedef struct {
    int key;
    int value;
    int left;
    int right;
    int parent;
    int color;
} srch_rb_node_t;

typedef struct {
    srch_rb_node_t nodes[512];
    int root;
    int count;
} srch_rb_tree_t;

void srch_rb_init(srch_rb_tree_t *tree) {
    tree->root = -1;
    tree->count = 0;
}

int srch_rb_search(const srch_rb_tree_t *tree, int key) {
    int cur = tree->root;
    while (cur != -1) {
        if (key == tree->nodes[cur].key) return tree->nodes[cur].value;
        if (key < tree->nodes[cur].key) cur = tree->nodes[cur].left;
        else cur = tree->nodes[cur].right;
    }
    return -1;
}

int srch_rb_minimum(const srch_rb_tree_t *tree, int node) {
    if (node == -1) return -1;
    while (tree->nodes[node].left != -1) {
        node = tree->nodes[node].left;
    }
    return node;
}

int srch_rb_successor(const srch_rb_tree_t *tree, int node) {
    if (node == -1) return -1;
    if (tree->nodes[node].right != -1) {
        return srch_rb_minimum(tree, tree->nodes[node].right);
    }
    int parent = tree->nodes[node].parent;
    while (parent != -1 && node == tree->nodes[parent].right) {
        node = parent;
        parent = tree->nodes[parent].parent;
    }
    return parent;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1160 red-black lookup failed: {:?}", result.err());
}

// ============================================================================
// C1161-C1165: Graph Search
// ============================================================================

#[test]
fn c1161_bfs_pathfinding() {
    let c_code = r#"
#define SRCH_BFS_MAX 256

typedef struct {
    int adj[SRCH_BFS_MAX][SRCH_BFS_MAX];
    int num_vertices;
} srch_bfs_graph_t;

void srch_bfs_init(srch_bfs_graph_t *g, int n) {
    int i, j;
    g->num_vertices = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++)
            g->adj[i][j] = 0;
}

void srch_bfs_add_edge(srch_bfs_graph_t *g, int u, int v) {
    g->adj[u][v] = 1;
    g->adj[v][u] = 1;
}

int srch_bfs_shortest_path(const srch_bfs_graph_t *g, int src, int dst, int *path, int *path_len) {
    int visited[SRCH_BFS_MAX];
    int parent[SRCH_BFS_MAX];
    int queue[SRCH_BFS_MAX];
    int front = 0, rear = 0;
    int i;
    for (i = 0; i < g->num_vertices; i++) {
        visited[i] = 0;
        parent[i] = -1;
    }
    visited[src] = 1;
    queue[rear++] = src;
    while (front < rear) {
        int cur = queue[front++];
        if (cur == dst) {
            int count = 0;
            int trace = dst;
            while (trace != -1) {
                path[count++] = trace;
                trace = parent[trace];
            }
            *path_len = count;
            return 1;
        }
        for (i = 0; i < g->num_vertices; i++) {
            if (g->adj[cur][i] && !visited[i]) {
                visited[i] = 1;
                parent[i] = cur;
                queue[rear++] = i;
            }
        }
    }
    *path_len = 0;
    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1161 BFS pathfinding failed: {:?}", result.err());
}

#[test]
fn c1162_dfs_cycle_finder() {
    let c_code = r#"
#define SRCH_DFS_MAX 128

typedef struct {
    int adj[SRCH_DFS_MAX][SRCH_DFS_MAX];
    int num_vertices;
} srch_dfs_graph_t;

void srch_dfs_init(srch_dfs_graph_t *g, int n) {
    int i, j;
    g->num_vertices = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++)
            g->adj[i][j] = 0;
}

void srch_dfs_add_edge(srch_dfs_graph_t *g, int u, int v) {
    g->adj[u][v] = 1;
}

int srch_dfs_visit(const srch_dfs_graph_t *g, int node, int *color, int *parent, int *cycle_start) {
    color[node] = 1;
    int i;
    for (i = 0; i < g->num_vertices; i++) {
        if (g->adj[node][i]) {
            if (color[i] == 1) {
                *cycle_start = i;
                parent[i] = node;
                return 1;
            }
            if (color[i] == 0) {
                parent[i] = node;
                if (srch_dfs_visit(g, i, color, parent, cycle_start)) return 1;
            }
        }
    }
    color[node] = 2;
    return 0;
}

int srch_dfs_has_cycle(const srch_dfs_graph_t *g) {
    int color[SRCH_DFS_MAX];
    int parent[SRCH_DFS_MAX];
    int cycle_start = -1;
    int i;
    for (i = 0; i < g->num_vertices; i++) {
        color[i] = 0;
        parent[i] = -1;
    }
    for (i = 0; i < g->num_vertices; i++) {
        if (color[i] == 0) {
            if (srch_dfs_visit(g, i, color, parent, &cycle_start)) return 1;
        }
    }
    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1162 DFS cycle finder failed: {:?}", result.err());
}

#[test]
fn c1163_bidirectional_bfs() {
    let c_code = r#"
#define SRCH_BIDIR_MAX 128

typedef struct {
    int adj[SRCH_BIDIR_MAX][SRCH_BIDIR_MAX];
    int n;
} srch_bidir_graph_t;

void srch_bidir_init(srch_bidir_graph_t *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++)
            g->adj[i][j] = 0;
}

void srch_bidir_add_edge(srch_bidir_graph_t *g, int u, int v) {
    g->adj[u][v] = 1;
    g->adj[v][u] = 1;
}

int srch_bidir_bfs(const srch_bidir_graph_t *g, int src, int dst) {
    if (src == dst) return 0;
    int visited_s[SRCH_BIDIR_MAX];
    int visited_d[SRCH_BIDIR_MAX];
    int dist_s[SRCH_BIDIR_MAX];
    int dist_d[SRCH_BIDIR_MAX];
    int queue_s[SRCH_BIDIR_MAX];
    int queue_d[SRCH_BIDIR_MAX];
    int fs = 0, rs = 0, fd = 0, rd = 0;
    int i;
    for (i = 0; i < g->n; i++) {
        visited_s[i] = 0;
        visited_d[i] = 0;
        dist_s[i] = -1;
        dist_d[i] = -1;
    }
    visited_s[src] = 1;
    dist_s[src] = 0;
    queue_s[rs++] = src;
    visited_d[dst] = 1;
    dist_d[dst] = 0;
    queue_d[rd++] = dst;
    while (fs < rs && fd < rd) {
        int cur = queue_s[fs++];
        for (i = 0; i < g->n; i++) {
            if (g->adj[cur][i] && !visited_s[i]) {
                visited_s[i] = 1;
                dist_s[i] = dist_s[cur] + 1;
                queue_s[rs++] = i;
                if (visited_d[i]) return dist_s[i] + dist_d[i];
            }
        }
        cur = queue_d[fd++];
        for (i = 0; i < g->n; i++) {
            if (g->adj[cur][i] && !visited_d[i]) {
                visited_d[i] = 1;
                dist_d[i] = dist_d[cur] + 1;
                queue_d[rd++] = i;
                if (visited_s[i]) return dist_s[i] + dist_d[i];
            }
        }
    }
    return -1;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1163 bidirectional BFS failed: {:?}", result.err());
}

#[test]
fn c1164_iterative_deepening() {
    let c_code = r#"
#define SRCH_IDS_MAX 128

typedef struct {
    int adj[SRCH_IDS_MAX][SRCH_IDS_MAX];
    int n;
} srch_ids_graph_t;

void srch_ids_init(srch_ids_graph_t *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++)
            g->adj[i][j] = 0;
}

void srch_ids_add_edge(srch_ids_graph_t *g, int u, int v) {
    g->adj[u][v] = 1;
}

int srch_ids_dls(const srch_ids_graph_t *g, int node, int target, int depth, int *visited) {
    if (node == target) return 1;
    if (depth <= 0) return 0;
    visited[node] = 1;
    int i;
    for (i = 0; i < g->n; i++) {
        if (g->adj[node][i] && !visited[i]) {
            if (srch_ids_dls(g, i, target, depth - 1, visited)) return 1;
        }
    }
    visited[node] = 0;
    return 0;
}

int srch_ids_search(const srch_ids_graph_t *g, int src, int target, int max_depth) {
    int depth;
    int visited[SRCH_IDS_MAX];
    for (depth = 0; depth <= max_depth; depth++) {
        int i;
        for (i = 0; i < g->n; i++) visited[i] = 0;
        if (srch_ids_dls(g, src, target, depth, visited)) return depth;
    }
    return -1;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1164 iterative deepening failed: {:?}", result.err());
}

#[test]
fn c1165_beam_search() {
    let c_code = r#"
#define SRCH_BEAM_MAX 256

typedef struct {
    int state;
    int score;
    int parent;
    int depth;
} srch_beam_node_t;

typedef struct {
    srch_beam_node_t nodes[SRCH_BEAM_MAX];
    int count;
} srch_beam_t;

void srch_beam_init(srch_beam_t *beam) {
    beam->count = 0;
}

void srch_beam_insert(srch_beam_t *beam, int state, int score, int parent, int depth) {
    if (beam->count >= SRCH_BEAM_MAX) return;
    int idx = beam->count++;
    beam->nodes[idx].state = state;
    beam->nodes[idx].score = score;
    beam->nodes[idx].parent = parent;
    beam->nodes[idx].depth = depth;
}

void srch_beam_sort_by_score(srch_beam_t *beam) {
    int i, j;
    for (i = 0; i < beam->count - 1; i++) {
        for (j = 0; j < beam->count - i - 1; j++) {
            if (beam->nodes[j].score < beam->nodes[j + 1].score) {
                srch_beam_node_t tmp = beam->nodes[j];
                beam->nodes[j] = beam->nodes[j + 1];
                beam->nodes[j + 1] = tmp;
            }
        }
    }
}

int srch_beam_select_top(srch_beam_t *beam, int beam_width, int *selected) {
    srch_beam_sort_by_score(beam);
    int count = beam->count;
    if (count > beam_width) count = beam_width;
    int i;
    for (i = 0; i < count; i++) {
        selected[i] = beam->nodes[i].state;
    }
    return count;
}

int srch_beam_search(int start, int goal, int beam_width, int max_depth) {
    srch_beam_t current;
    srch_beam_t next;
    srch_beam_init(&current);
    srch_beam_insert(&current, start, 0, -1, 0);
    int depth;
    for (depth = 0; depth < max_depth; depth++) {
        srch_beam_init(&next);
        int selected[SRCH_BEAM_MAX];
        int num_sel = srch_beam_select_top(&current, beam_width, selected);
        int i;
        for (i = 0; i < num_sel; i++) {
            if (selected[i] == goal) return depth;
            int neighbor;
            for (neighbor = 0; neighbor < 4; neighbor++) {
                int ns = selected[i] + neighbor - 1;
                int score = -(ns - goal) * (ns - goal);
                srch_beam_insert(&next, ns, score, selected[i], depth + 1);
            }
        }
        current = next;
        if (current.count == 0) break;
    }
    return -1;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1165 beam search failed: {:?}", result.err());
}

// ============================================================================
// C1166-C1170: Heuristic Search
// ============================================================================

#[test]
fn c1166_astar_grid() {
    let c_code = r#"
#define SRCH_ASTAR_ROWS 32
#define SRCH_ASTAR_COLS 32
#define SRCH_ASTAR_MAX (SRCH_ASTAR_ROWS * SRCH_ASTAR_COLS)

typedef struct {
    int row;
    int col;
    int g;
    int h;
    int f;
    int parent_row;
    int parent_col;
    int closed;
    int open;
} srch_astar_cell_t;

int srch_astar_abs(int x) {
    return x < 0 ? -x : x;
}

int srch_astar_heuristic(int r1, int c1, int r2, int c2) {
    return srch_astar_abs(r1 - r2) + srch_astar_abs(c1 - c2);
}

int srch_astar_find_min_open(srch_astar_cell_t grid[SRCH_ASTAR_ROWS][SRCH_ASTAR_COLS], int rows, int cols, int *out_r, int *out_c) {
    int min_f = 999999;
    int found = 0;
    int r, c;
    for (r = 0; r < rows; r++) {
        for (c = 0; c < cols; c++) {
            if (grid[r][c].open && !grid[r][c].closed && grid[r][c].f < min_f) {
                min_f = grid[r][c].f;
                *out_r = r;
                *out_c = c;
                found = 1;
            }
        }
    }
    return found;
}

int srch_astar_search(int walls[SRCH_ASTAR_ROWS][SRCH_ASTAR_COLS],
                      int rows, int cols,
                      int sr, int sc, int er, int ec) {
    srch_astar_cell_t grid[SRCH_ASTAR_ROWS][SRCH_ASTAR_COLS];
    int r, c;
    for (r = 0; r < rows; r++) {
        for (c = 0; c < cols; c++) {
            grid[r][c].g = 999999;
            grid[r][c].h = 0;
            grid[r][c].f = 999999;
            grid[r][c].parent_row = -1;
            grid[r][c].parent_col = -1;
            grid[r][c].closed = 0;
            grid[r][c].open = 0;
        }
    }
    grid[sr][sc].g = 0;
    grid[sr][sc].h = srch_astar_heuristic(sr, sc, er, ec);
    grid[sr][sc].f = grid[sr][sc].h;
    grid[sr][sc].open = 1;
    int dx[4] = {-1, 1, 0, 0};
    int dy[4] = {0, 0, -1, 1};
    while (1) {
        int cur_r, cur_c;
        if (!srch_astar_find_min_open(grid, rows, cols, &cur_r, &cur_c)) return -1;
        if (cur_r == er && cur_c == ec) return grid[er][ec].g;
        grid[cur_r][cur_c].closed = 1;
        int d;
        for (d = 0; d < 4; d++) {
            int nr = cur_r + dx[d];
            int nc = cur_c + dy[d];
            if (nr < 0 || nr >= rows || nc < 0 || nc >= cols) continue;
            if (walls[nr][nc] || grid[nr][nc].closed) continue;
            int new_g = grid[cur_r][cur_c].g + 1;
            if (new_g < grid[nr][nc].g) {
                grid[nr][nc].g = new_g;
                grid[nr][nc].h = srch_astar_heuristic(nr, nc, er, ec);
                grid[nr][nc].f = new_g + grid[nr][nc].h;
                grid[nr][nc].parent_row = cur_r;
                grid[nr][nc].parent_col = cur_c;
                grid[nr][nc].open = 1;
            }
        }
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1166 A* grid search failed: {:?}", result.err());
}

#[test]
fn c1167_hill_climbing() {
    let c_code = r#"
typedef struct {
    int x;
    int y;
    int value;
} srch_hill_point_t;

int srch_hill_evaluate(int *landscape, int width, int height, int x, int y) {
    if (x < 0 || x >= width || y < 0 || y >= height) return -999999;
    return landscape[y * width + x];
}

srch_hill_point_t srch_hill_climb(int *landscape, int width, int height, int start_x, int start_y) {
    srch_hill_point_t current;
    current.x = start_x;
    current.y = start_y;
    current.value = srch_hill_evaluate(landscape, width, height, start_x, start_y);
    int dx[4] = {-1, 1, 0, 0};
    int dy[4] = {0, 0, -1, 1};
    int improved = 1;
    while (improved) {
        improved = 0;
        int d;
        for (d = 0; d < 4; d++) {
            int nx = current.x + dx[d];
            int ny = current.y + dy[d];
            int nv = srch_hill_evaluate(landscape, width, height, nx, ny);
            if (nv > current.value) {
                current.x = nx;
                current.y = ny;
                current.value = nv;
                improved = 1;
            }
        }
    }
    return current;
}

srch_hill_point_t srch_hill_steepest(int *landscape, int width, int height, int start_x, int start_y) {
    srch_hill_point_t current;
    current.x = start_x;
    current.y = start_y;
    current.value = srch_hill_evaluate(landscape, width, height, start_x, start_y);
    int dx[8] = {-1, -1, -1, 0, 0, 1, 1, 1};
    int dy[8] = {-1, 0, 1, -1, 1, -1, 0, 1};
    int improved = 1;
    while (improved) {
        improved = 0;
        int best_val = current.value;
        int best_x = current.x;
        int best_y = current.y;
        int d;
        for (d = 0; d < 8; d++) {
            int nx = current.x + dx[d];
            int ny = current.y + dy[d];
            int nv = srch_hill_evaluate(landscape, width, height, nx, ny);
            if (nv > best_val) {
                best_val = nv;
                best_x = nx;
                best_y = ny;
            }
        }
        if (best_val > current.value) {
            current.x = best_x;
            current.y = best_y;
            current.value = best_val;
            improved = 1;
        }
    }
    return current;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1167 hill climbing failed: {:?}", result.err());
}

#[test]
fn c1168_simulated_annealing() {
    let c_code = r#"
typedef struct {
    int state[64];
    int n;
    int cost;
} srch_sa_solution_t;

int srch_sa_compute_cost(const int *state, int n) {
    int cost = 0;
    int i;
    for (i = 0; i < n - 1; i++) {
        int diff = state[i + 1] - state[i];
        if (diff < 0) diff = -diff;
        cost += diff;
    }
    return cost;
}

void srch_sa_copy_state(int *dst, const int *src, int n) {
    int i;
    for (i = 0; i < n; i++) dst[i] = src[i];
}

int srch_sa_pseudo_random(int seed) {
    return (seed * 1103515245 + 12345) & 0x7FFFFFFF;
}

srch_sa_solution_t srch_sa_search(int *initial, int n, int max_iter) {
    srch_sa_solution_t best;
    srch_sa_solution_t current;
    int i;
    best.n = n;
    current.n = n;
    srch_sa_copy_state(best.state, initial, n);
    srch_sa_copy_state(current.state, initial, n);
    best.cost = srch_sa_compute_cost(initial, n);
    current.cost = best.cost;
    int seed = 42;
    for (i = 0; i < max_iter; i++) {
        int temperature = max_iter - i;
        seed = srch_sa_pseudo_random(seed);
        int idx1 = (seed & 0x7FFFFFFF) % n;
        seed = srch_sa_pseudo_random(seed);
        int idx2 = (seed & 0x7FFFFFFF) % n;
        int tmp = current.state[idx1];
        current.state[idx1] = current.state[idx2];
        current.state[idx2] = tmp;
        int new_cost = srch_sa_compute_cost(current.state, n);
        int delta = new_cost - current.cost;
        seed = srch_sa_pseudo_random(seed);
        int accept = 0;
        if (delta < 0) {
            accept = 1;
        } else if (temperature > 0) {
            int threshold = (seed & 0x7FFFFFFF) % (temperature + 1);
            if (threshold > delta) accept = 1;
        }
        if (accept) {
            current.cost = new_cost;
            if (current.cost < best.cost) {
                srch_sa_copy_state(best.state, current.state, n);
                best.cost = current.cost;
            }
        } else {
            tmp = current.state[idx1];
            current.state[idx1] = current.state[idx2];
            current.state[idx2] = tmp;
        }
    }
    return best;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1168 simulated annealing failed: {:?}", result.err());
}

#[test]
fn c1169_tabu_search() {
    let c_code = r#"
#define SRCH_TABU_MAX 64
#define SRCH_TABU_LIST_MAX 32

typedef struct {
    int solution[SRCH_TABU_MAX];
    int n;
    int cost;
} srch_tabu_state_t;

typedef struct {
    int moves[SRCH_TABU_LIST_MAX][2];
    int count;
    int head;
} srch_tabu_list_t;

void srch_tabu_list_init(srch_tabu_list_t *tl) {
    tl->count = 0;
    tl->head = 0;
}

void srch_tabu_list_add(srch_tabu_list_t *tl, int i, int j) {
    int idx = tl->head;
    tl->moves[idx][0] = i;
    tl->moves[idx][1] = j;
    tl->head = (tl->head + 1) % SRCH_TABU_LIST_MAX;
    if (tl->count < SRCH_TABU_LIST_MAX) tl->count++;
}

int srch_tabu_list_contains(const srch_tabu_list_t *tl, int i, int j) {
    int k;
    for (k = 0; k < tl->count; k++) {
        int idx = (tl->head - 1 - k + SRCH_TABU_LIST_MAX) % SRCH_TABU_LIST_MAX;
        if (tl->moves[idx][0] == i && tl->moves[idx][1] == j) return 1;
        if (tl->moves[idx][0] == j && tl->moves[idx][1] == i) return 1;
    }
    return 0;
}

int srch_tabu_cost(const int *sol, int n) {
    int cost = 0;
    int i;
    for (i = 0; i < n - 1; i++) {
        int d = sol[i + 1] - sol[i];
        if (d < 0) d = -d;
        cost += d;
    }
    return cost;
}

srch_tabu_state_t srch_tabu_search(int *initial, int n, int max_iter) {
    srch_tabu_state_t best;
    srch_tabu_state_t current;
    srch_tabu_list_t tabu;
    int i, iter;
    srch_tabu_list_init(&tabu);
    best.n = n;
    current.n = n;
    for (i = 0; i < n; i++) {
        best.solution[i] = initial[i];
        current.solution[i] = initial[i];
    }
    best.cost = srch_tabu_cost(initial, n);
    current.cost = best.cost;
    for (iter = 0; iter < max_iter; iter++) {
        int best_i = -1, best_j = -1;
        int best_delta = 999999;
        for (i = 0; i < n - 1; i++) {
            int j;
            for (j = i + 1; j < n; j++) {
                if (srch_tabu_list_contains(&tabu, i, j)) continue;
                int tmp = current.solution[i];
                current.solution[i] = current.solution[j];
                current.solution[j] = tmp;
                int nc = srch_tabu_cost(current.solution, n);
                int delta = nc - current.cost;
                if (delta < best_delta) {
                    best_delta = delta;
                    best_i = i;
                    best_j = j;
                }
                tmp = current.solution[i];
                current.solution[i] = current.solution[j];
                current.solution[j] = tmp;
            }
        }
        if (best_i == -1) break;
        int tmp = current.solution[best_i];
        current.solution[best_i] = current.solution[best_j];
        current.solution[best_j] = tmp;
        current.cost = current.cost + best_delta;
        srch_tabu_list_add(&tabu, best_i, best_j);
        if (current.cost < best.cost) {
            for (i = 0; i < n; i++) best.solution[i] = current.solution[i];
            best.cost = current.cost;
        }
    }
    return best;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1169 tabu search failed: {:?}", result.err());
}

#[test]
fn c1170_genetic_selection() {
    let c_code = r#"
#define SRCH_GA_POP 32
#define SRCH_GA_GENES 16

typedef struct {
    int genes[SRCH_GA_GENES];
    int fitness;
} srch_ga_individual_t;

typedef struct {
    srch_ga_individual_t pop[SRCH_GA_POP];
    int size;
} srch_ga_population_t;

int srch_ga_evaluate(const int *genes, int n) {
    int sum = 0;
    int i;
    for (i = 0; i < n; i++) {
        sum += genes[i] * genes[i];
    }
    return -sum;
}

void srch_ga_evaluate_all(srch_ga_population_t *pop, int gene_len) {
    int i;
    for (i = 0; i < pop->size; i++) {
        pop->pop[i].fitness = srch_ga_evaluate(pop->pop[i].genes, gene_len);
    }
}

int srch_ga_tournament_select(const srch_ga_population_t *pop, int seed) {
    int a = ((seed * 1103515245 + 12345) & 0x7FFFFFFF) % pop->size;
    int b = ((seed * 6364136223846793005 + 1) & 0x7FFFFFFF) % pop->size;
    if (pop->pop[a].fitness > pop->pop[b].fitness) return a;
    return b;
}

void srch_ga_crossover(const int *parent1, const int *parent2, int *child, int n, int crosspoint) {
    int i;
    for (i = 0; i < crosspoint; i++) {
        child[i] = parent1[i];
    }
    for (i = crosspoint; i < n; i++) {
        child[i] = parent2[i];
    }
}

void srch_ga_mutate(int *genes, int n, int seed) {
    int idx = ((seed * 1103515245 + 12345) & 0x7FFFFFFF) % n;
    genes[idx] = genes[idx] + 1;
}

int srch_ga_find_best(const srch_ga_population_t *pop) {
    int best = 0;
    int i;
    for (i = 1; i < pop->size; i++) {
        if (pop->pop[i].fitness > pop->pop[best].fitness) {
            best = i;
        }
    }
    return best;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1170 genetic algorithm selection failed: {:?}", result.err());
}

// ============================================================================
// C1171-C1175: Specialized Search
// ============================================================================

#[test]
fn c1171_binary_search_on_answer() {
    let c_code = r#"
int srch_bsa_can_split(int *arr, int n, int max_sum, int k) {
    int pieces = 1;
    int current_sum = 0;
    int i;
    for (i = 0; i < n; i++) {
        if (arr[i] > max_sum) return 0;
        if (current_sum + arr[i] > max_sum) {
            pieces++;
            current_sum = arr[i];
            if (pieces > k) return 0;
        } else {
            current_sum += arr[i];
        }
    }
    return 1;
}

int srch_bsa_min_max_partition(int *arr, int n, int k) {
    int low = 0, high = 0;
    int i;
    for (i = 0; i < n; i++) {
        if (arr[i] > low) low = arr[i];
        high += arr[i];
    }
    int result = high;
    while (low <= high) {
        int mid = low + (high - low) / 2;
        if (srch_bsa_can_split(arr, n, mid, k)) {
            result = mid;
            high = mid - 1;
        } else {
            low = mid + 1;
        }
    }
    return result;
}

int srch_bsa_min_pages(int *pages, int n, int students) {
    if (students > n) return -1;
    return srch_bsa_min_max_partition(pages, n, students);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1171 binary search on answer failed: {:?}", result.err());
}

#[test]
fn c1172_two_pointer_search() {
    let c_code = r#"
int srch_two_ptr_pair_sum(int *arr, int n, int target, int *out_i, int *out_j) {
    int left = 0, right = n - 1;
    while (left < right) {
        int sum = arr[left] + arr[right];
        if (sum == target) {
            *out_i = left;
            *out_j = right;
            return 1;
        }
        if (sum < target) left++;
        else right--;
    }
    *out_i = -1;
    *out_j = -1;
    return 0;
}

int srch_two_ptr_three_sum(int *arr, int n, int target, int *result) {
    int count = 0;
    int i;
    for (i = 0; i < n - 2; i++) {
        if (i > 0 && arr[i] == arr[i - 1]) continue;
        int left = i + 1, right = n - 1;
        while (left < right) {
            int sum = arr[i] + arr[left] + arr[right];
            if (sum == target) {
                result[count * 3] = arr[i];
                result[count * 3 + 1] = arr[left];
                result[count * 3 + 2] = arr[right];
                count++;
                while (left < right && arr[left] == arr[left + 1]) left++;
                while (left < right && arr[right] == arr[right - 1]) right--;
                left++;
                right--;
            } else if (sum < target) {
                left++;
            } else {
                right--;
            }
        }
    }
    return count;
}

int srch_two_ptr_container_water(int *heights, int n) {
    int left = 0, right = n - 1;
    int max_water = 0;
    while (left < right) {
        int h = heights[left] < heights[right] ? heights[left] : heights[right];
        int water = h * (right - left);
        if (water > max_water) max_water = water;
        if (heights[left] < heights[right]) left++;
        else right--;
    }
    return max_water;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1172 two pointer search failed: {:?}", result.err());
}

#[test]
fn c1173_sliding_window_max() {
    let c_code = r#"
typedef struct {
    int data[256];
    int front;
    int rear;
    int size;
} srch_deque_t;

void srch_deque_init(srch_deque_t *dq) {
    dq->front = 0;
    dq->rear = -1;
    dq->size = 0;
}

void srch_deque_push_back(srch_deque_t *dq, int val) {
    dq->rear = (dq->rear + 1) % 256;
    dq->data[dq->rear] = val;
    dq->size++;
}

void srch_deque_pop_front(srch_deque_t *dq) {
    dq->front = (dq->front + 1) % 256;
    dq->size--;
}

void srch_deque_pop_back(srch_deque_t *dq) {
    dq->rear = (dq->rear - 1 + 256) % 256;
    dq->size--;
}

int srch_deque_front(const srch_deque_t *dq) {
    return dq->data[dq->front];
}

int srch_deque_back(const srch_deque_t *dq) {
    return dq->data[dq->rear];
}

int srch_deque_empty(const srch_deque_t *dq) {
    return dq->size == 0;
}

void srch_sliding_window_max(int *arr, int n, int k, int *result, int *result_len) {
    srch_deque_t dq;
    srch_deque_init(&dq);
    int i;
    *result_len = 0;
    for (i = 0; i < n; i++) {
        while (!srch_deque_empty(&dq) && srch_deque_front(&dq) <= i - k) {
            srch_deque_pop_front(&dq);
        }
        while (!srch_deque_empty(&dq) && arr[srch_deque_back(&dq)] <= arr[i]) {
            srch_deque_pop_back(&dq);
        }
        srch_deque_push_back(&dq, i);
        if (i >= k - 1) {
            result[(*result_len)++] = arr[srch_deque_front(&dq)];
        }
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1173 sliding window max failed: {:?}", result.err());
}

#[test]
fn c1174_median_of_medians() {
    let c_code = r#"
void srch_mom_swap(int *a, int *b) {
    int tmp = *a;
    *a = *b;
    *b = tmp;
}

void srch_mom_insertion_sort(int *arr, int left, int right) {
    int i, j;
    for (i = left + 1; i <= right; i++) {
        int key = arr[i];
        j = i - 1;
        while (j >= left && arr[j] > key) {
            arr[j + 1] = arr[j];
            j--;
        }
        arr[j + 1] = key;
    }
}

int srch_mom_partition(int *arr, int left, int right, int pivot_val) {
    int i;
    for (i = left; i <= right; i++) {
        if (arr[i] == pivot_val) {
            srch_mom_swap(&arr[i], &arr[right]);
            break;
        }
    }
    int store = left;
    for (i = left; i < right; i++) {
        if (arr[i] < pivot_val) {
            srch_mom_swap(&arr[i], &arr[store]);
            store++;
        }
    }
    srch_mom_swap(&arr[store], &arr[right]);
    return store;
}

int srch_mom_select(int *arr, int left, int right, int k) {
    if (left == right) return arr[left];
    int n = right - left + 1;
    int num_groups = (n + 4) / 5;
    int medians[128];
    int i;
    for (i = 0; i < num_groups; i++) {
        int gl = left + i * 5;
        int gr = gl + 4;
        if (gr > right) gr = right;
        srch_mom_insertion_sort(arr, gl, gr);
        medians[i] = arr[gl + (gr - gl) / 2];
    }
    int pivot;
    if (num_groups == 1) {
        pivot = medians[0];
    } else {
        int tmp[128];
        for (i = 0; i < num_groups; i++) tmp[i] = medians[i];
        pivot = srch_mom_select(tmp, 0, num_groups - 1, num_groups / 2);
    }
    int pos = srch_mom_partition(arr, left, right, pivot);
    if (k == pos) return arr[pos];
    if (k < pos) return srch_mom_select(arr, left, pos - 1, k);
    return srch_mom_select(arr, pos + 1, right, k);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1174 median of medians failed: {:?}", result.err());
}

#[test]
fn c1175_kth_element_quickselect() {
    let c_code = r#"
void srch_qs_swap(int *a, int *b) {
    int tmp = *a;
    *a = *b;
    *b = tmp;
}

int srch_qs_partition(int *arr, int left, int right) {
    int pivot = arr[right];
    int i = left - 1;
    int j;
    for (j = left; j < right; j++) {
        if (arr[j] <= pivot) {
            i++;
            srch_qs_swap(&arr[i], &arr[j]);
        }
    }
    srch_qs_swap(&arr[i + 1], &arr[right]);
    return i + 1;
}

int srch_qs_median3(int *arr, int left, int right) {
    int mid = left + (right - left) / 2;
    if (arr[left] > arr[mid]) srch_qs_swap(&arr[left], &arr[mid]);
    if (arr[left] > arr[right]) srch_qs_swap(&arr[left], &arr[right]);
    if (arr[mid] > arr[right]) srch_qs_swap(&arr[mid], &arr[right]);
    srch_qs_swap(&arr[mid], &arr[right]);
    return srch_qs_partition(arr, left, right);
}

int srch_qs_quickselect(int *arr, int left, int right, int k) {
    while (left <= right) {
        if (left == right) return arr[left];
        int pivot_idx = srch_qs_median3(arr, left, right);
        if (k == pivot_idx) return arr[k];
        if (k < pivot_idx) right = pivot_idx - 1;
        else left = pivot_idx + 1;
    }
    return -1;
}

int srch_qs_kth_smallest(int *arr, int n, int k) {
    if (k < 0 || k >= n) return -1;
    return srch_qs_quickselect(arr, 0, n - 1, k);
}

int srch_qs_kth_largest(int *arr, int n, int k) {
    if (k < 0 || k >= n) return -1;
    return srch_qs_quickselect(arr, 0, n - 1, n - 1 - k);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1175 kth element quickselect failed: {:?}", result.err());
}
