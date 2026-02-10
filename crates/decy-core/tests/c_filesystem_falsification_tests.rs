//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1826-C1850: Filesystem Operation Patterns -- path manipulation,
//! directory traversal, file metadata, I/O simulation, and filesystem
//! utilities commonly found in systems programming.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! All functions use `fs_` prefix to avoid symbol conflicts.
//! C code is self-contained with no #include directives.
//!
//! Organization:
//! - C1826-C1830: Path operations (join, normalize, split extension, parent, basename)
//! - C1831-C1835: Directory operations (list entries, recursive walk, directory tree, filtered listing)
//! - C1836-C1840: File metadata (file size, permissions, modification time, type detection)
//! - C1841-C1845: File I/O simulation (read buffer, write buffer, seek, copy, truncate)
//! - C1846-C1850: Filesystem utilities (temp file, file locking, glob match, disk usage, inode tracking)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C1826-C1830: Path Operations
// ============================================================================

/// C1826: Path join -- concatenate directory and filename with separator
#[test]
fn c1826_path_join() {
    let c_code = r##"
typedef unsigned long size_t;

#define FS_PATH_MAX 512
#define FS_SEP '/'

typedef struct {
    char buf[FS_PATH_MAX];
    size_t len;
} fs_path_t;

void fs_path_init(fs_path_t *p) {
    p->len = 0;
    p->buf[0] = '\0';
}

static size_t fs_strlen(const char *s) {
    size_t n = 0;
    while (s[n] != '\0') n++;
    return n;
}

static void fs_strcpy(char *dst, const char *src, size_t max) {
    size_t i = 0;
    while (i < max - 1 && src[i] != '\0') {
        dst[i] = src[i];
        i++;
    }
    dst[i] = '\0';
}

int fs_path_join(fs_path_t *result, const char *dir, const char *name) {
    size_t dlen = fs_strlen(dir);
    size_t nlen = fs_strlen(name);
    if (dlen + nlen + 2 > FS_PATH_MAX) return -1;
    fs_strcpy(result->buf, dir, FS_PATH_MAX);
    if (dlen > 0 && dir[dlen - 1] != FS_SEP) {
        result->buf[dlen] = FS_SEP;
        dlen++;
    }
    fs_strcpy(result->buf + dlen, name, FS_PATH_MAX - dlen);
    result->len = dlen + nlen;
    return 0;
}

int fs_path_join_test(void) {
    fs_path_t p;
    fs_path_init(&p);
    int rc = fs_path_join(&p, "/home/user", "file.txt");
    if (rc != 0) return -1;
    if (p.len != 20) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1826: Path join should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1826: Output should not be empty");
    assert!(code.contains("fn fs_path_join"), "C1826: Should contain fn fs_path_join");
}

/// C1827: Path normalize -- resolve . and .. components
#[test]
fn c1827_path_normalize() {
    let c_code = r##"
typedef unsigned long size_t;

#define FS_PATH_MAX 512
#define FS_MAX_COMPONENTS 64

typedef struct {
    char buf[FS_PATH_MAX];
    size_t len;
} fs_norm_path_t;

static int fs_norm_is_sep(char c) {
    return c == '/';
}

int fs_path_normalize(fs_norm_path_t *out, const char *input) {
    char components[FS_MAX_COMPONENTS][128];
    int count = 0;
    int i = 0;
    int is_absolute = 0;

    if (input[0] == '/') {
        is_absolute = 1;
        i = 1;
    }

    while (input[i] != '\0' && count < FS_MAX_COMPONENTS) {
        int start = i;
        while (input[i] != '\0' && !fs_norm_is_sep(input[i])) i++;
        int clen = i - start;
        if (clen == 1 && input[start] == '.') {
            /* skip current dir */
        } else if (clen == 2 && input[start] == '.' && input[start + 1] == '.') {
            if (count > 0) count--;
        } else if (clen > 0) {
            int j;
            for (j = 0; j < clen && j < 127; j++) {
                components[count][j] = input[start + j];
            }
            components[count][j] = '\0';
            count++;
        }
        if (input[i] == '/') i++;
    }

    int pos = 0;
    if (is_absolute) {
        out->buf[pos++] = '/';
    }
    int c;
    for (c = 0; c < count; c++) {
        if (c > 0) out->buf[pos++] = '/';
        int k = 0;
        while (components[c][k] != '\0' && pos < FS_PATH_MAX - 1) {
            out->buf[pos++] = components[c][k++];
        }
    }
    out->buf[pos] = '\0';
    out->len = pos;
    return 0;
}

int fs_normalize_test(void) {
    fs_norm_path_t p;
    fs_path_normalize(&p, "/home/user/../user/./docs");
    if (p.buf[0] != '/') return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1827: Path normalize should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1827: Output should not be empty");
    assert!(code.contains("fn fs_path_normalize"), "C1827: Should contain fn fs_path_normalize");
}

/// C1828: Split path into stem and extension
#[test]
fn c1828_path_split_extension() {
    let c_code = r##"
typedef unsigned long size_t;

#define FS_NAME_MAX 256

typedef struct {
    char stem[FS_NAME_MAX];
    char ext[FS_NAME_MAX];
    int has_ext;
} fs_split_ext_t;

int fs_path_split_ext(fs_split_ext_t *out, const char *filename) {
    int last_dot = -1;
    int last_sep = -1;
    int i = 0;

    while (filename[i] != '\0') {
        if (filename[i] == '.') last_dot = i;
        if (filename[i] == '/') last_sep = i;
        i++;
    }
    int total_len = i;

    if (last_dot <= last_sep + 1 || last_dot < 0) {
        int j;
        for (j = 0; j < total_len && j < FS_NAME_MAX - 1; j++)
            out->stem[j] = filename[j];
        out->stem[j] = '\0';
        out->ext[0] = '\0';
        out->has_ext = 0;
        return 0;
    }

    int s;
    for (s = 0; s < last_dot && s < FS_NAME_MAX - 1; s++)
        out->stem[s] = filename[s];
    out->stem[s] = '\0';

    int e = 0;
    int k;
    for (k = last_dot + 1; k < total_len && e < FS_NAME_MAX - 1; k++)
        out->ext[e++] = filename[k];
    out->ext[e] = '\0';
    out->has_ext = 1;
    return 0;
}

int fs_split_ext_test(void) {
    fs_split_ext_t r;
    fs_path_split_ext(&r, "/home/user/file.tar.gz");
    if (!r.has_ext) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1828: Split extension should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1828: Output should not be empty");
    assert!(code.contains("fn fs_path_split_ext"), "C1828: Should contain fn fs_path_split_ext");
}

/// C1829: Extract parent directory from path
#[test]
fn c1829_parent_directory() {
    let c_code = r##"
typedef unsigned long size_t;

#define FS_PATH_MAX 512

typedef struct {
    char path[FS_PATH_MAX];
    size_t len;
} fs_parent_result_t;

int fs_path_parent(fs_parent_result_t *out, const char *path) {
    int last_sep = -1;
    int i = 0;

    while (path[i] != '\0') {
        if (path[i] == '/') last_sep = i;
        i++;
    }

    if (last_sep <= 0) {
        if (path[0] == '/') {
            out->path[0] = '/';
            out->path[1] = '\0';
            out->len = 1;
        } else {
            out->path[0] = '.';
            out->path[1] = '\0';
            out->len = 1;
        }
        return 0;
    }

    int j;
    for (j = 0; j < last_sep && j < FS_PATH_MAX - 1; j++)
        out->path[j] = path[j];
    out->path[j] = '\0';
    out->len = j;
    return 0;
}

int fs_parent_test(void) {
    fs_parent_result_t r;
    fs_path_parent(&r, "/home/user/docs/file.txt");
    if (r.len == 0) return -1;
    if (r.path[0] != '/') return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1829: Parent directory should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1829: Output should not be empty");
    assert!(code.contains("fn fs_path_parent"), "C1829: Should contain fn fs_path_parent");
}

/// C1830: Extract basename (filename) from full path
#[test]
fn c1830_basename() {
    let c_code = r##"
typedef unsigned long size_t;

#define FS_NAME_MAX 256

typedef struct {
    char name[FS_NAME_MAX];
    size_t len;
} fs_basename_result_t;

int fs_path_basename(fs_basename_result_t *out, const char *path) {
    int last_sep = -1;
    int i = 0;

    while (path[i] != '\0') {
        if (path[i] == '/') last_sep = i;
        i++;
    }
    int total = i;

    if (total == 0) {
        out->name[0] = '\0';
        out->len = 0;
        return -1;
    }

    /* Handle trailing slash */
    int end = total;
    if (end > 1 && path[end - 1] == '/') end--;

    int start = last_sep + 1;
    if (last_sep == end - 1) {
        /* Recalculate for trailing slash case */
        int k = end - 2;
        while (k >= 0 && path[k] != '/') k--;
        start = k + 1;
    }

    int j = 0;
    int m;
    for (m = start; m < end && j < FS_NAME_MAX - 1; m++)
        out->name[j++] = path[m];
    out->name[j] = '\0';
    out->len = j;
    return 0;
}

int fs_basename_test(void) {
    fs_basename_result_t r;
    fs_path_basename(&r, "/home/user/document.pdf");
    if (r.len == 0) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1830: Basename should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1830: Output should not be empty");
    assert!(code.contains("fn fs_path_basename"), "C1830: Should contain fn fs_path_basename");
}

// ============================================================================
// C1831-C1835: Directory Operations
// ============================================================================

/// C1831: Directory entry listing with simulated readdir
#[test]
fn c1831_list_directory_entries() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define FS_MAX_ENTRIES 128
#define FS_NAME_MAX 256
#define FS_TYPE_FILE 0
#define FS_TYPE_DIR  1
#define FS_TYPE_LINK 2

typedef struct {
    char name[FS_NAME_MAX];
    int entry_type;
    uint32_t inode;
    size_t size;
} fs_dir_entry_t;

typedef struct {
    fs_dir_entry_t entries[FS_MAX_ENTRIES];
    int count;
    int position;
} fs_dir_listing_t;

void fs_dir_init(fs_dir_listing_t *d) {
    d->count = 0;
    d->position = 0;
}

int fs_dir_add_entry(fs_dir_listing_t *d, const char *name, int etype, uint32_t ino, size_t sz) {
    if (d->count >= FS_MAX_ENTRIES) return -1;
    int i = 0;
    while (name[i] != '\0' && i < FS_NAME_MAX - 1) {
        d->entries[d->count].name[i] = name[i];
        i++;
    }
    d->entries[d->count].name[i] = '\0';
    d->entries[d->count].entry_type = etype;
    d->entries[d->count].inode = ino;
    d->entries[d->count].size = sz;
    d->count++;
    return 0;
}

int fs_dir_next(fs_dir_listing_t *d, fs_dir_entry_t *out) {
    if (d->position >= d->count) return -1;
    int i = 0;
    while (d->entries[d->position].name[i] != '\0') {
        out->name[i] = d->entries[d->position].name[i];
        i++;
    }
    out->name[i] = '\0';
    out->entry_type = d->entries[d->position].entry_type;
    out->inode = d->entries[d->position].inode;
    out->size = d->entries[d->position].size;
    d->position++;
    return 0;
}

int fs_dir_list_test(void) {
    fs_dir_listing_t dir;
    fs_dir_init(&dir);
    fs_dir_add_entry(&dir, "readme.md", FS_TYPE_FILE, 100, 4096);
    fs_dir_add_entry(&dir, "src", FS_TYPE_DIR, 101, 0);
    fs_dir_entry_t e;
    int rc = fs_dir_next(&dir, &e);
    if (rc != 0) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1831: Directory listing should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1831: Output should not be empty");
    assert!(code.contains("fn fs_dir_init"), "C1831: Should contain fn fs_dir_init");
    assert!(code.contains("fn fs_dir_next"), "C1831: Should contain fn fs_dir_next");
}

/// C1832: Recursive directory walk with depth tracking
#[test]
fn c1832_recursive_directory_walk() {
    let c_code = r##"
typedef unsigned long size_t;

#define FS_WALK_MAX_DEPTH 32
#define FS_WALK_MAX_NODES 256
#define FS_NAME_LEN 64

typedef struct {
    char name[FS_NAME_LEN];
    int is_dir;
    int parent_idx;
    int depth;
} fs_walk_node_t;

typedef struct {
    fs_walk_node_t nodes[FS_WALK_MAX_NODES];
    int node_count;
    int stack[FS_WALK_MAX_DEPTH];
    int stack_top;
    int max_depth;
    size_t total_size;
} fs_walk_state_t;

void fs_walk_init(fs_walk_state_t *w, int max_depth) {
    w->node_count = 0;
    w->stack_top = 0;
    w->max_depth = max_depth;
    w->total_size = 0;
}

int fs_walk_push(fs_walk_state_t *w, const char *name, int is_dir, int parent) {
    if (w->node_count >= FS_WALK_MAX_NODES) return -1;
    int depth = 0;
    if (parent >= 0) depth = w->nodes[parent].depth + 1;
    if (depth > w->max_depth) return -2;

    int idx = w->node_count;
    int i = 0;
    while (name[i] != '\0' && i < FS_NAME_LEN - 1) {
        w->nodes[idx].name[i] = name[i];
        i++;
    }
    w->nodes[idx].name[i] = '\0';
    w->nodes[idx].is_dir = is_dir;
    w->nodes[idx].parent_idx = parent;
    w->nodes[idx].depth = depth;
    w->node_count++;

    if (is_dir && w->stack_top < FS_WALK_MAX_DEPTH) {
        w->stack[w->stack_top++] = idx;
    }
    return idx;
}

int fs_walk_count_at_depth(fs_walk_state_t *w, int depth) {
    int count = 0;
    int i;
    for (i = 0; i < w->node_count; i++) {
        if (w->nodes[i].depth == depth) count++;
    }
    return count;
}

int fs_walk_test(void) {
    fs_walk_state_t w;
    fs_walk_init(&w, 10);
    int root = fs_walk_push(&w, "root", 1, -1);
    fs_walk_push(&w, "src", 1, root);
    fs_walk_push(&w, "main.c", 0, root);
    int at_depth_1 = fs_walk_count_at_depth(&w, 1);
    if (at_depth_1 != 2) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1832: Recursive walk should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1832: Output should not be empty");
    assert!(code.contains("fn fs_walk_init"), "C1832: Should contain fn fs_walk_init");
    assert!(code.contains("fn fs_walk_push"), "C1832: Should contain fn fs_walk_push");
}

/// C1833: Directory tree builder with indented display
#[test]
fn c1833_directory_tree() {
    let c_code = r##"
typedef unsigned long size_t;

#define FS_TREE_MAX 128
#define FS_TREE_NAME_LEN 64

typedef struct {
    char name[FS_TREE_NAME_LEN];
    int is_directory;
    int parent;
    int child_count;
} fs_tree_node_t;

typedef struct {
    fs_tree_node_t nodes[FS_TREE_MAX];
    int count;
    int root;
} fs_tree_t;

void fs_tree_init(fs_tree_t *t) {
    t->count = 0;
    t->root = -1;
}

int fs_tree_add(fs_tree_t *t, const char *name, int is_dir, int parent) {
    if (t->count >= FS_TREE_MAX) return -1;
    int idx = t->count;
    int i = 0;
    while (name[i] != '\0' && i < FS_TREE_NAME_LEN - 1) {
        t->nodes[idx].name[i] = name[i];
        i++;
    }
    t->nodes[idx].name[i] = '\0';
    t->nodes[idx].is_directory = is_dir;
    t->nodes[idx].parent = parent;
    t->nodes[idx].child_count = 0;
    if (parent >= 0 && parent < t->count) {
        t->nodes[parent].child_count++;
    }
    if (t->count == 0) t->root = idx;
    t->count++;
    return idx;
}

int fs_tree_depth(fs_tree_t *t, int idx) {
    int depth = 0;
    int cur = idx;
    while (cur >= 0 && t->nodes[cur].parent >= 0) {
        cur = t->nodes[cur].parent;
        depth++;
    }
    return depth;
}

int fs_tree_count_dirs(fs_tree_t *t) {
    int dirs = 0;
    int i;
    for (i = 0; i < t->count; i++) {
        if (t->nodes[i].is_directory) dirs++;
    }
    return dirs;
}

int fs_tree_test(void) {
    fs_tree_t tree;
    fs_tree_init(&tree);
    int root = fs_tree_add(&tree, "project", 1, -1);
    int src = fs_tree_add(&tree, "src", 1, root);
    fs_tree_add(&tree, "main.c", 0, src);
    fs_tree_add(&tree, "utils.c", 0, src);
    if (fs_tree_count_dirs(&tree) != 2) return -1;
    if (fs_tree_depth(&tree, 2) != 2) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1833: Directory tree should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1833: Output should not be empty");
    assert!(code.contains("fn fs_tree_init"), "C1833: Should contain fn fs_tree_init");
    assert!(code.contains("fn fs_tree_add"), "C1833: Should contain fn fs_tree_add");
}

/// C1834: Filtered directory listing by extension
#[test]
fn c1834_filtered_listing() {
    let c_code = r##"
typedef unsigned long size_t;

#define FS_FILT_MAX 64
#define FS_FILT_NAME_LEN 128

typedef struct {
    char name[FS_FILT_NAME_LEN];
    size_t size;
} fs_filt_entry_t;

typedef struct {
    fs_filt_entry_t entries[FS_FILT_MAX];
    int count;
} fs_filt_result_t;

static int fs_filt_ends_with(const char *str, const char *suffix) {
    int slen = 0, xlen = 0;
    while (str[slen] != '\0') slen++;
    while (suffix[xlen] != '\0') xlen++;
    if (xlen > slen) return 0;
    int i;
    for (i = 0; i < xlen; i++) {
        if (str[slen - xlen + i] != suffix[i]) return 0;
    }
    return 1;
}

int fs_filter_by_ext(fs_filt_result_t *out, const char names[][FS_FILT_NAME_LEN],
                     const size_t sizes[], int total, const char *ext) {
    out->count = 0;
    int i;
    for (i = 0; i < total && out->count < FS_FILT_MAX; i++) {
        if (fs_filt_ends_with(names[i], ext)) {
            int j = 0;
            while (names[i][j] != '\0' && j < FS_FILT_NAME_LEN - 1) {
                out->entries[out->count].name[j] = names[i][j];
                j++;
            }
            out->entries[out->count].name[j] = '\0';
            out->entries[out->count].size = sizes[i];
            out->count++;
        }
    }
    return out->count;
}

int fs_filter_test(void) {
    fs_filt_result_t r;
    char names[4][FS_FILT_NAME_LEN];
    size_t sizes[4];
    int k;
    for (k = 0; k < 4; k++) {
        names[k][0] = 'a' + k;
        names[k][1] = '.';
        names[k][2] = (k < 2) ? 'c' : 'h';
        names[k][3] = '\0';
        sizes[k] = 100 + k;
    }
    int count = fs_filter_by_ext(&r, names, sizes, 4, ".c");
    if (count != 2) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1834: Filtered listing should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1834: Output should not be empty");
    assert!(code.contains("fn fs_filter_by_ext"), "C1834: Should contain fn fs_filter_by_ext");
}

/// C1835: Directory size calculation with recursive accumulation
#[test]
fn c1835_directory_size_calc() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define FS_DSIZE_MAX 256

typedef struct {
    int parent;
    int is_dir;
    size_t file_size;
} fs_dsize_entry_t;

typedef struct {
    fs_dsize_entry_t entries[FS_DSIZE_MAX];
    int count;
} fs_dsize_tree_t;

void fs_dsize_init(fs_dsize_tree_t *t) {
    t->count = 0;
}

int fs_dsize_add(fs_dsize_tree_t *t, int parent, int is_dir, size_t sz) {
    if (t->count >= FS_DSIZE_MAX) return -1;
    int idx = t->count;
    t->entries[idx].parent = parent;
    t->entries[idx].is_dir = is_dir;
    t->entries[idx].file_size = sz;
    t->count++;
    return idx;
}

size_t fs_dsize_compute(fs_dsize_tree_t *t, int dir_idx) {
    size_t total = 0;
    int i;
    for (i = 0; i < t->count; i++) {
        if (t->entries[i].parent == dir_idx) {
            if (t->entries[i].is_dir) {
                total += fs_dsize_compute(t, i);
            } else {
                total += t->entries[i].file_size;
            }
        }
    }
    return total;
}

int fs_dsize_test(void) {
    fs_dsize_tree_t tree;
    fs_dsize_init(&tree);
    int root = fs_dsize_add(&tree, -1, 1, 0);
    fs_dsize_add(&tree, root, 0, 1024);
    fs_dsize_add(&tree, root, 0, 2048);
    int sub = fs_dsize_add(&tree, root, 1, 0);
    fs_dsize_add(&tree, sub, 0, 512);
    size_t total = fs_dsize_compute(&tree, root);
    if (total != 3584) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1835: Directory size calc should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1835: Output should not be empty");
    assert!(code.contains("fn fs_dsize_compute"), "C1835: Should contain fn fs_dsize_compute");
}

// ============================================================================
// C1836-C1840: File Metadata
// ============================================================================

/// C1836: File size tracking with block-level accounting
#[test]
fn c1836_file_size_tracker() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define FS_BLOCK_SIZE 4096
#define FS_MAX_FILES 64

typedef struct {
    uint32_t inode;
    size_t logical_size;
    size_t block_count;
    size_t allocated_bytes;
} fs_size_info_t;

typedef struct {
    fs_size_info_t files[FS_MAX_FILES];
    int count;
    size_t total_allocated;
} fs_size_tracker_t;

void fs_size_tracker_init(fs_size_tracker_t *t) {
    t->count = 0;
    t->total_allocated = 0;
}

int fs_size_track(fs_size_tracker_t *t, uint32_t inode, size_t logical) {
    if (t->count >= FS_MAX_FILES) return -1;
    size_t blocks = (logical + FS_BLOCK_SIZE - 1) / FS_BLOCK_SIZE;
    size_t allocated = blocks * FS_BLOCK_SIZE;
    int idx = t->count;
    t->files[idx].inode = inode;
    t->files[idx].logical_size = logical;
    t->files[idx].block_count = blocks;
    t->files[idx].allocated_bytes = allocated;
    t->total_allocated += allocated;
    t->count++;
    return idx;
}

size_t fs_size_wasted(fs_size_tracker_t *t) {
    size_t wasted = 0;
    int i;
    for (i = 0; i < t->count; i++) {
        wasted += t->files[i].allocated_bytes - t->files[i].logical_size;
    }
    return wasted;
}

int fs_size_test(void) {
    fs_size_tracker_t tracker;
    fs_size_tracker_init(&tracker);
    fs_size_track(&tracker, 1, 100);
    fs_size_track(&tracker, 2, 5000);
    size_t w = fs_size_wasted(&tracker);
    if (w == 0) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1836: File size tracker should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1836: Output should not be empty");
    assert!(code.contains("fn fs_size_track"), "C1836: Should contain fn fs_size_track");
    assert!(code.contains("fn fs_size_wasted"), "C1836: Should contain fn fs_size_wasted");
}

/// C1837: File permissions check with mode bits
#[test]
fn c1837_permissions_check() {
    let c_code = r##"
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;

#define FS_PERM_READ    0x04
#define FS_PERM_WRITE   0x02
#define FS_PERM_EXEC    0x01
#define FS_PERM_OWNER_SHIFT 6
#define FS_PERM_GROUP_SHIFT 3
#define FS_PERM_OTHER_SHIFT 0

typedef struct {
    uint16_t mode;
    uint32_t uid;
    uint32_t gid;
} fs_perm_info_t;

int fs_perm_check_owner(fs_perm_info_t *info, int flag) {
    int bits = (info->mode >> FS_PERM_OWNER_SHIFT) & 0x07;
    return (bits & flag) != 0;
}

int fs_perm_check_group(fs_perm_info_t *info, int flag) {
    int bits = (info->mode >> FS_PERM_GROUP_SHIFT) & 0x07;
    return (bits & flag) != 0;
}

int fs_perm_check_other(fs_perm_info_t *info, int flag) {
    int bits = (info->mode >> FS_PERM_OTHER_SHIFT) & 0x07;
    return (bits & flag) != 0;
}

int fs_perm_can_access(fs_perm_info_t *info, uint32_t uid, uint32_t gid, int flag) {
    if (uid == info->uid) return fs_perm_check_owner(info, flag);
    if (gid == info->gid) return fs_perm_check_group(info, flag);
    return fs_perm_check_other(info, flag);
}

int fs_perm_test(void) {
    fs_perm_info_t info;
    info.mode = 0x1ED; /* 0755 in octal = rwxr-xr-x */
    info.uid = 1000;
    info.gid = 100;
    if (!fs_perm_can_access(&info, 1000, 100, FS_PERM_WRITE)) return -1;
    if (fs_perm_can_access(&info, 2000, 200, FS_PERM_WRITE)) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1837: Permissions check should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1837: Output should not be empty");
    assert!(code.contains("fn fs_perm_can_access"), "C1837: Should contain fn fs_perm_can_access");
}

/// C1838: File modification time tracking with epoch timestamps
#[test]
fn c1838_modification_time() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;
typedef long int64_t;

#define FS_TIME_MAX_FILES 32

typedef struct {
    uint32_t inode;
    int64_t created_at;
    int64_t modified_at;
    int64_t accessed_at;
} fs_time_entry_t;

typedef struct {
    fs_time_entry_t files[FS_TIME_MAX_FILES];
    int count;
} fs_time_table_t;

void fs_time_init(fs_time_table_t *t) {
    t->count = 0;
}

int fs_time_register(fs_time_table_t *t, uint32_t ino, int64_t now) {
    if (t->count >= FS_TIME_MAX_FILES) return -1;
    int idx = t->count;
    t->files[idx].inode = ino;
    t->files[idx].created_at = now;
    t->files[idx].modified_at = now;
    t->files[idx].accessed_at = now;
    t->count++;
    return idx;
}

int fs_time_touch(fs_time_table_t *t, uint32_t ino, int64_t now) {
    int i;
    for (i = 0; i < t->count; i++) {
        if (t->files[i].inode == ino) {
            t->files[i].modified_at = now;
            t->files[i].accessed_at = now;
            return 0;
        }
    }
    return -1;
}

int fs_time_find_newest(fs_time_table_t *t) {
    if (t->count == 0) return -1;
    int newest = 0;
    int i;
    for (i = 1; i < t->count; i++) {
        if (t->files[i].modified_at > t->files[newest].modified_at)
            newest = i;
    }
    return newest;
}

int fs_time_test(void) {
    fs_time_table_t tbl;
    fs_time_init(&tbl);
    fs_time_register(&tbl, 10, 1000);
    fs_time_register(&tbl, 20, 2000);
    fs_time_touch(&tbl, 10, 3000);
    int newest = fs_time_find_newest(&tbl);
    if (newest != 0) return -1;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1838: Modification time should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1838: Output should not be empty");
    assert!(code.contains("fn fs_time_register"), "C1838: Should contain fn fs_time_register");
    assert!(code.contains("fn fs_time_touch"), "C1838: Should contain fn fs_time_touch");
}

/// C1839: File type detection from magic bytes
#[test]
fn c1839_file_type_detection() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

#define FS_FTYPE_UNKNOWN 0
#define FS_FTYPE_PNG     1
#define FS_FTYPE_JPEG    2
#define FS_FTYPE_PDF     3
#define FS_FTYPE_ELF     4
#define FS_FTYPE_ZIP     5

typedef struct {
    int file_type;
    int confidence;
} fs_ftype_result_t;

int fs_ftype_detect(fs_ftype_result_t *out, const uint8_t *header, size_t len) {
    out->file_type = FS_FTYPE_UNKNOWN;
    out->confidence = 0;

    if (len < 4) return -1;

    /* PNG: 0x89 0x50 0x4E 0x47 */
    if (header[0] == 0x89 && header[1] == 0x50 &&
        header[2] == 0x4E && header[3] == 0x47) {
        out->file_type = FS_FTYPE_PNG;
        out->confidence = 100;
        return 0;
    }

    /* JPEG: 0xFF 0xD8 0xFF */
    if (header[0] == 0xFF && header[1] == 0xD8 && header[2] == 0xFF) {
        out->file_type = FS_FTYPE_JPEG;
        out->confidence = 95;
        return 0;
    }

    /* PDF: %PDF */
    if (header[0] == 0x25 && header[1] == 0x50 &&
        header[2] == 0x44 && header[3] == 0x46) {
        out->file_type = FS_FTYPE_PDF;
        out->confidence = 100;
        return 0;
    }

    /* ELF: 0x7F E L F */
    if (header[0] == 0x7F && header[1] == 0x45 &&
        header[2] == 0x4C && header[3] == 0x46) {
        out->file_type = FS_FTYPE_ELF;
        out->confidence = 100;
        return 0;
    }

    /* ZIP: PK 0x03 0x04 */
    if (header[0] == 0x50 && header[1] == 0x4B &&
        header[2] == 0x03 && header[3] == 0x04) {
        out->file_type = FS_FTYPE_ZIP;
        out->confidence = 90;
        return 0;
    }

    return -1;
}

int fs_ftype_test(void) {
    uint8_t png_hdr[4];
    png_hdr[0] = 0x89;
    png_hdr[1] = 0x50;
    png_hdr[2] = 0x4E;
    png_hdr[3] = 0x47;
    fs_ftype_result_t r;
    int rc = fs_ftype_detect(&r, png_hdr, 4);
    if (rc != 0) return -1;
    if (r.file_type != FS_FTYPE_PNG) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1839: File type detection should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1839: Output should not be empty");
    assert!(code.contains("fn fs_ftype_detect"), "C1839: Should contain fn fs_ftype_detect");
}

/// C1840: File attribute flags with bitmask operations
#[test]
fn c1840_file_attributes() {
    let c_code = r##"
typedef unsigned int uint32_t;

#define FS_ATTR_READONLY  0x01
#define FS_ATTR_HIDDEN    0x02
#define FS_ATTR_SYSTEM    0x04
#define FS_ATTR_ARCHIVE   0x08
#define FS_ATTR_DIRECTORY 0x10
#define FS_ATTR_SYMLINK   0x20

typedef struct {
    uint32_t flags;
    uint32_t inode;
} fs_attr_t;

void fs_attr_init(fs_attr_t *a, uint32_t ino) {
    a->flags = 0;
    a->inode = ino;
}

void fs_attr_set(fs_attr_t *a, uint32_t flag) {
    a->flags = a->flags | flag;
}

void fs_attr_clear(fs_attr_t *a, uint32_t flag) {
    a->flags = a->flags & (~flag);
}

int fs_attr_has(fs_attr_t *a, uint32_t flag) {
    return (a->flags & flag) != 0;
}

int fs_attr_is_writable(fs_attr_t *a) {
    return !fs_attr_has(a, FS_ATTR_READONLY);
}

int fs_attr_is_visible(fs_attr_t *a) {
    return !fs_attr_has(a, FS_ATTR_HIDDEN) && !fs_attr_has(a, FS_ATTR_SYSTEM);
}

int fs_attr_count_set(fs_attr_t *a) {
    uint32_t v = a->flags;
    int count = 0;
    while (v != 0) {
        count += v & 1;
        v = v >> 1;
    }
    return count;
}

int fs_attr_test(void) {
    fs_attr_t a;
    fs_attr_init(&a, 42);
    fs_attr_set(&a, FS_ATTR_READONLY);
    fs_attr_set(&a, FS_ATTR_ARCHIVE);
    if (!fs_attr_has(&a, FS_ATTR_READONLY)) return -1;
    if (fs_attr_is_writable(&a)) return -2;
    if (fs_attr_count_set(&a) != 2) return -3;
    fs_attr_clear(&a, FS_ATTR_READONLY);
    if (!fs_attr_is_writable(&a)) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1840: File attributes should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1840: Output should not be empty");
    assert!(code.contains("fn fs_attr_init"), "C1840: Should contain fn fs_attr_init");
    assert!(code.contains("fn fs_attr_has"), "C1840: Should contain fn fs_attr_has");
}

// ============================================================================
// C1841-C1845: File I/O Simulation
// ============================================================================

/// C1841: Read buffer simulation with position tracking
#[test]
fn c1841_read_buffer() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

#define FS_RBUF_SIZE 1024

typedef struct {
    uint8_t data[FS_RBUF_SIZE];
    size_t size;
    size_t pos;
    int eof_flag;
} fs_read_buf_t;

void fs_rbuf_init(fs_read_buf_t *rb, const uint8_t *src, size_t len) {
    size_t i;
    size_t copy_len = len < FS_RBUF_SIZE ? len : FS_RBUF_SIZE;
    for (i = 0; i < copy_len; i++) {
        rb->data[i] = src[i];
    }
    rb->size = copy_len;
    rb->pos = 0;
    rb->eof_flag = 0;
}

int fs_rbuf_read(fs_read_buf_t *rb, uint8_t *dst, size_t count, size_t *bytes_read) {
    if (rb->pos >= rb->size) {
        rb->eof_flag = 1;
        *bytes_read = 0;
        return -1;
    }
    size_t avail = rb->size - rb->pos;
    size_t to_read = count < avail ? count : avail;
    size_t i;
    for (i = 0; i < to_read; i++) {
        dst[i] = rb->data[rb->pos + i];
    }
    rb->pos += to_read;
    *bytes_read = to_read;
    if (rb->pos >= rb->size) rb->eof_flag = 1;
    return 0;
}

int fs_rbuf_is_eof(fs_read_buf_t *rb) {
    return rb->eof_flag;
}

size_t fs_rbuf_remaining(fs_read_buf_t *rb) {
    return rb->size - rb->pos;
}

int fs_rbuf_test(void) {
    uint8_t src[16];
    int i;
    for (i = 0; i < 16; i++) src[i] = i;
    fs_read_buf_t rb;
    fs_rbuf_init(&rb, src, 16);
    uint8_t dst[8];
    size_t got;
    fs_rbuf_read(&rb, dst, 8, &got);
    if (got != 8) return -1;
    if (fs_rbuf_remaining(&rb) != 8) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1841: Read buffer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1841: Output should not be empty");
    assert!(code.contains("fn fs_rbuf_init"), "C1841: Should contain fn fs_rbuf_init");
    assert!(code.contains("fn fs_rbuf_read"), "C1841: Should contain fn fs_rbuf_read");
}

/// C1842: Write buffer with flush threshold
#[test]
fn c1842_write_buffer() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

#define FS_WBUF_SIZE 512
#define FS_WBUF_FLUSH_THRESHOLD 384

typedef struct {
    uint8_t buffer[FS_WBUF_SIZE];
    size_t used;
    int flush_count;
    size_t total_written;
} fs_write_buf_t;

void fs_wbuf_init(fs_write_buf_t *wb) {
    wb->used = 0;
    wb->flush_count = 0;
    wb->total_written = 0;
}

int fs_wbuf_flush(fs_write_buf_t *wb) {
    if (wb->used == 0) return 0;
    wb->total_written += wb->used;
    wb->used = 0;
    wb->flush_count++;
    return 0;
}

int fs_wbuf_write(fs_write_buf_t *wb, const uint8_t *data, size_t len) {
    size_t remaining = len;
    size_t offset = 0;
    while (remaining > 0) {
        size_t space = FS_WBUF_SIZE - wb->used;
        size_t chunk = remaining < space ? remaining : space;
        size_t i;
        for (i = 0; i < chunk; i++) {
            wb->buffer[wb->used + i] = data[offset + i];
        }
        wb->used += chunk;
        offset += chunk;
        remaining -= chunk;
        if (wb->used >= FS_WBUF_FLUSH_THRESHOLD) {
            fs_wbuf_flush(wb);
        }
    }
    return 0;
}

size_t fs_wbuf_pending(fs_write_buf_t *wb) {
    return wb->used;
}

int fs_wbuf_test(void) {
    fs_write_buf_t wb;
    fs_wbuf_init(&wb);
    uint8_t data[256];
    int i;
    for (i = 0; i < 256; i++) data[i] = i;
    fs_wbuf_write(&wb, data, 256);
    fs_wbuf_write(&wb, data, 200);
    if (wb.flush_count == 0) return -1;
    fs_wbuf_flush(&wb);
    if (wb.total_written != 456) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1842: Write buffer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1842: Output should not be empty");
    assert!(code.contains("fn fs_wbuf_init"), "C1842: Should contain fn fs_wbuf_init");
    assert!(code.contains("fn fs_wbuf_write"), "C1842: Should contain fn fs_wbuf_write");
}

/// C1843: Seek operations on simulated file descriptor
#[test]
fn c1843_seek_operations() {
    let c_code = r##"
typedef unsigned long size_t;
typedef long int64_t;

#define FS_SEEK_SET 0
#define FS_SEEK_CUR 1
#define FS_SEEK_END 2

typedef struct {
    size_t file_size;
    int64_t position;
    int open;
} fs_seek_fd_t;

int fs_seek_open(fs_seek_fd_t *fd, size_t size) {
    fd->file_size = size;
    fd->position = 0;
    fd->open = 1;
    return 0;
}

int fs_seek(fs_seek_fd_t *fd, int64_t offset, int whence) {
    if (!fd->open) return -1;
    int64_t new_pos;

    if (whence == FS_SEEK_SET) {
        new_pos = offset;
    } else if (whence == FS_SEEK_CUR) {
        new_pos = fd->position + offset;
    } else if (whence == FS_SEEK_END) {
        new_pos = (int64_t)fd->file_size + offset;
    } else {
        return -2;
    }

    if (new_pos < 0) return -3;
    fd->position = new_pos;
    return 0;
}

int64_t fs_seek_tell(fs_seek_fd_t *fd) {
    if (!fd->open) return -1;
    return fd->position;
}

void fs_seek_close(fs_seek_fd_t *fd) {
    fd->open = 0;
}

int fs_seek_test(void) {
    fs_seek_fd_t fd;
    fs_seek_open(&fd, 1000);
    fs_seek(&fd, 100, FS_SEEK_SET);
    if (fs_seek_tell(&fd) != 100) return -1;
    fs_seek(&fd, 50, FS_SEEK_CUR);
    if (fs_seek_tell(&fd) != 150) return -2;
    fs_seek(&fd, -10, FS_SEEK_END);
    if (fs_seek_tell(&fd) != 990) return -3;
    fs_seek_close(&fd);
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1843: Seek operations should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1843: Output should not be empty");
    assert!(code.contains("fn fs_seek_open"), "C1843: Should contain fn fs_seek_open");
    assert!(code.contains("fn fs_seek"), "C1843: Should contain fn fs_seek");
}

/// C1844: File copy with chunked transfer
#[test]
fn c1844_file_copy() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

#define FS_COPY_CHUNK 256
#define FS_COPY_MAX 2048

typedef struct {
    uint8_t data[FS_COPY_MAX];
    size_t size;
} fs_copy_file_t;

typedef struct {
    size_t bytes_copied;
    int chunks_transferred;
    int success;
} fs_copy_result_t;

int fs_copy_file(fs_copy_result_t *result, fs_copy_file_t *dst,
                 const fs_copy_file_t *src) {
    result->bytes_copied = 0;
    result->chunks_transferred = 0;
    result->success = 0;

    if (src->size > FS_COPY_MAX) return -1;

    size_t remaining = src->size;
    size_t offset = 0;

    while (remaining > 0) {
        size_t chunk = remaining < FS_COPY_CHUNK ? remaining : FS_COPY_CHUNK;
        size_t i;
        for (i = 0; i < chunk; i++) {
            dst->data[offset + i] = src->data[offset + i];
        }
        offset += chunk;
        remaining -= chunk;
        result->bytes_copied += chunk;
        result->chunks_transferred++;
    }

    dst->size = src->size;
    result->success = 1;
    return 0;
}

int fs_copy_verify(const fs_copy_file_t *a, const fs_copy_file_t *b) {
    if (a->size != b->size) return -1;
    size_t i;
    for (i = 0; i < a->size; i++) {
        if (a->data[i] != b->data[i]) return -2;
    }
    return 0;
}

int fs_copy_test(void) {
    fs_copy_file_t src;
    fs_copy_file_t dst;
    size_t i;
    src.size = 600;
    for (i = 0; i < 600; i++) src.data[i] = (uint8_t)(i & 0xFF);
    dst.size = 0;
    fs_copy_result_t r;
    fs_copy_file(&r, &dst, &src);
    if (!r.success) return -1;
    if (r.chunks_transferred != 3) return -2;
    if (fs_copy_verify(&src, &dst) != 0) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1844: File copy should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1844: Output should not be empty");
    assert!(code.contains("fn fs_copy_file"), "C1844: Should contain fn fs_copy_file");
    assert!(code.contains("fn fs_copy_verify"), "C1844: Should contain fn fs_copy_verify");
}

/// C1845: File truncate with size validation
#[test]
fn c1845_file_truncate() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

#define FS_TRUNC_MAX 1024

typedef struct {
    uint8_t data[FS_TRUNC_MAX];
    size_t size;
    size_t capacity;
} fs_trunc_file_t;

void fs_trunc_init(fs_trunc_file_t *f) {
    f->size = 0;
    f->capacity = FS_TRUNC_MAX;
}

int fs_trunc_append(fs_trunc_file_t *f, uint8_t val, size_t count) {
    if (f->size + count > f->capacity) return -1;
    size_t i;
    for (i = 0; i < count; i++) {
        f->data[f->size + i] = val;
    }
    f->size += count;
    return 0;
}

int fs_trunc_truncate(fs_trunc_file_t *f, size_t new_size) {
    if (new_size > f->capacity) return -1;
    if (new_size > f->size) {
        /* Extend with zeros */
        size_t i;
        for (i = f->size; i < new_size; i++) {
            f->data[i] = 0;
        }
    }
    f->size = new_size;
    return 0;
}

int fs_trunc_is_zero_filled(fs_trunc_file_t *f, size_t from, size_t to) {
    size_t i;
    for (i = from; i < to && i < f->size; i++) {
        if (f->data[i] != 0) return 0;
    }
    return 1;
}

int fs_trunc_test(void) {
    fs_trunc_file_t f;
    fs_trunc_init(&f);
    fs_trunc_append(&f, 0xAA, 100);
    if (f.size != 100) return -1;
    fs_trunc_truncate(&f, 50);
    if (f.size != 50) return -2;
    fs_trunc_truncate(&f, 80);
    if (f.size != 80) return -3;
    if (!fs_trunc_is_zero_filled(&f, 50, 80)) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1845: File truncate should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1845: Output should not be empty");
    assert!(code.contains("fn fs_trunc_init"), "C1845: Should contain fn fs_trunc_init");
    assert!(code.contains("fn fs_trunc_truncate"), "C1845: Should contain fn fs_trunc_truncate");
}

// ============================================================================
// C1846-C1850: Filesystem Utilities
// ============================================================================

/// C1846: Temp file name generation with pseudo-random suffix
#[test]
fn c1846_temp_file_creation() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define FS_TEMP_PREFIX_MAX 64
#define FS_TEMP_NAME_MAX 128

typedef struct {
    uint32_t state;
} fs_temp_rng_t;

void fs_temp_rng_init(fs_temp_rng_t *rng, uint32_t seed) {
    rng->state = seed;
}

uint32_t fs_temp_rng_next(fs_temp_rng_t *rng) {
    rng->state = rng->state * 1103515245 + 12345;
    return (rng->state >> 16) & 0x7FFF;
}

int fs_temp_generate(char *out, size_t out_max, const char *prefix,
                     const char *dir, fs_temp_rng_t *rng) {
    int pos = 0;
    int i = 0;

    /* Copy directory */
    while (dir[i] != '\0' && pos < (int)out_max - 2) {
        out[pos++] = dir[i++];
    }
    if (pos > 0 && out[pos - 1] != '/') {
        out[pos++] = '/';
    }

    /* Copy prefix */
    i = 0;
    while (prefix[i] != '\0' && pos < (int)out_max - 10) {
        out[pos++] = prefix[i++];
    }

    /* Add random suffix */
    int d;
    for (d = 0; d < 6 && pos < (int)out_max - 1; d++) {
        uint32_t r = fs_temp_rng_next(rng) % 36;
        if (r < 10) {
            out[pos++] = '0' + r;
        } else {
            out[pos++] = 'a' + (r - 10);
        }
    }

    out[pos] = '\0';
    return pos;
}

int fs_temp_test(void) {
    char name[FS_TEMP_NAME_MAX];
    fs_temp_rng_t rng;
    fs_temp_rng_init(&rng, 42);
    int len = fs_temp_generate(name, FS_TEMP_NAME_MAX, "tmp_", "/tmp", &rng);
    if (len <= 0) return -1;
    if (name[0] != '/') return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1846: Temp file creation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1846: Output should not be empty");
    assert!(code.contains("fn fs_temp_generate"), "C1846: Should contain fn fs_temp_generate");
}

/// C1847: File locking with lock table
#[test]
fn c1847_file_locking() {
    let c_code = r##"
typedef unsigned int uint32_t;

#define FS_LOCK_MAX 32
#define FS_LOCK_NONE     0
#define FS_LOCK_SHARED   1
#define FS_LOCK_EXCLUSIVE 2

typedef struct {
    uint32_t inode;
    int lock_type;
    int holder_pid;
    int ref_count;
} fs_lock_entry_t;

typedef struct {
    fs_lock_entry_t locks[FS_LOCK_MAX];
    int count;
} fs_lock_table_t;

void fs_lock_init(fs_lock_table_t *lt) {
    lt->count = 0;
}

static int fs_lock_find(fs_lock_table_t *lt, uint32_t inode) {
    int i;
    for (i = 0; i < lt->count; i++) {
        if (lt->locks[i].inode == inode) return i;
    }
    return -1;
}

int fs_lock_acquire(fs_lock_table_t *lt, uint32_t inode, int pid, int lock_type) {
    int idx = fs_lock_find(lt, inode);
    if (idx >= 0) {
        if (lt->locks[idx].lock_type == FS_LOCK_EXCLUSIVE) return -1;
        if (lock_type == FS_LOCK_EXCLUSIVE) return -2;
        if (lt->locks[idx].lock_type == FS_LOCK_SHARED && lock_type == FS_LOCK_SHARED) {
            lt->locks[idx].ref_count++;
            return 0;
        }
        return -3;
    }

    if (lt->count >= FS_LOCK_MAX) return -4;
    idx = lt->count;
    lt->locks[idx].inode = inode;
    lt->locks[idx].lock_type = lock_type;
    lt->locks[idx].holder_pid = pid;
    lt->locks[idx].ref_count = 1;
    lt->count++;
    return 0;
}

int fs_lock_release(fs_lock_table_t *lt, uint32_t inode, int pid) {
    int idx = fs_lock_find(lt, inode);
    if (idx < 0) return -1;
    lt->locks[idx].ref_count--;
    if (lt->locks[idx].ref_count <= 0) {
        lt->locks[idx] = lt->locks[lt->count - 1];
        lt->count--;
    }
    return 0;
}

int fs_lock_test(void) {
    fs_lock_table_t lt;
    fs_lock_init(&lt);
    int rc = fs_lock_acquire(&lt, 100, 1, FS_LOCK_EXCLUSIVE);
    if (rc != 0) return -1;
    rc = fs_lock_acquire(&lt, 100, 2, FS_LOCK_SHARED);
    if (rc == 0) return -2; /* Should fail: exclusive held */
    fs_lock_release(&lt, 100, 1);
    rc = fs_lock_acquire(&lt, 100, 2, FS_LOCK_SHARED);
    if (rc != 0) return -3;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1847: File locking should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1847: Output should not be empty");
    assert!(code.contains("fn fs_lock_init"), "C1847: Should contain fn fs_lock_init");
    assert!(code.contains("fn fs_lock_acquire"), "C1847: Should contain fn fs_lock_acquire");
}

/// C1848: Glob pattern matching for filenames
#[test]
fn c1848_glob_matching() {
    let c_code = r##"
typedef unsigned long size_t;

int fs_glob_match(const char *pattern, const char *str) {
    int pi = 0;
    int si = 0;
    int star_p = -1;
    int star_s = -1;

    while (str[si] != '\0') {
        if (pattern[pi] == '?' || pattern[pi] == str[si]) {
            pi++;
            si++;
        } else if (pattern[pi] == '*') {
            star_p = pi;
            star_s = si;
            pi++;
        } else if (star_p >= 0) {
            pi = star_p + 1;
            star_s++;
            si = star_s;
        } else {
            return 0;
        }
    }

    while (pattern[pi] == '*') pi++;
    return pattern[pi] == '\0';
}

int fs_glob_filter_count(const char *pattern, const char names[][64],
                         int total) {
    int count = 0;
    int i;
    for (i = 0; i < total; i++) {
        if (fs_glob_match(pattern, names[i])) count++;
    }
    return count;
}

int fs_glob_test(void) {
    if (!fs_glob_match("*.c", "main.c")) return -1;
    if (!fs_glob_match("test_??.c", "test_01.c")) return -2;
    if (fs_glob_match("*.h", "main.c")) return -3;
    if (!fs_glob_match("src/**/main.*", "src/**/main.*")) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1848: Glob matching should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1848: Output should not be empty");
    assert!(code.contains("fn fs_glob_match"), "C1848: Should contain fn fs_glob_match");
}

/// C1849: Disk usage calculator with block accounting
#[test]
fn c1849_disk_usage() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define FS_DU_BLOCK_SIZE 4096
#define FS_DU_MAX_PARTS 8

typedef struct {
    size_t total_blocks;
    size_t used_blocks;
    size_t reserved_blocks;
    uint32_t partition_id;
} fs_du_partition_t;

typedef struct {
    fs_du_partition_t parts[FS_DU_MAX_PARTS];
    int part_count;
} fs_du_system_t;

void fs_du_init(fs_du_system_t *sys) {
    sys->part_count = 0;
}

int fs_du_add_partition(fs_du_system_t *sys, uint32_t id, size_t total,
                        size_t used, size_t reserved) {
    if (sys->part_count >= FS_DU_MAX_PARTS) return -1;
    if (used + reserved > total) return -2;
    int idx = sys->part_count;
    sys->parts[idx].partition_id = id;
    sys->parts[idx].total_blocks = total;
    sys->parts[idx].used_blocks = used;
    sys->parts[idx].reserved_blocks = reserved;
    sys->part_count++;
    return idx;
}

size_t fs_du_available(fs_du_system_t *sys, int idx) {
    if (idx < 0 || idx >= sys->part_count) return 0;
    return sys->parts[idx].total_blocks - sys->parts[idx].used_blocks
           - sys->parts[idx].reserved_blocks;
}

int fs_du_usage_percent(fs_du_system_t *sys, int idx) {
    if (idx < 0 || idx >= sys->part_count) return -1;
    if (sys->parts[idx].total_blocks == 0) return 0;
    return (int)((sys->parts[idx].used_blocks * 100)
                 / sys->parts[idx].total_blocks);
}

size_t fs_du_total_free(fs_du_system_t *sys) {
    size_t total = 0;
    int i;
    for (i = 0; i < sys->part_count; i++) {
        total += fs_du_available(sys, i);
    }
    return total;
}

int fs_du_test(void) {
    fs_du_system_t sys;
    fs_du_init(&sys);
    fs_du_add_partition(&sys, 1, 1000, 600, 50);
    fs_du_add_partition(&sys, 2, 2000, 500, 100);
    if (fs_du_usage_percent(&sys, 0) != 60) return -1;
    size_t free_total = fs_du_total_free(&sys);
    if (free_total != 1750) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1849: Disk usage should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1849: Output should not be empty");
    assert!(code.contains("fn fs_du_init"), "C1849: Should contain fn fs_du_init");
    assert!(code.contains("fn fs_du_available"), "C1849: Should contain fn fs_du_available");
}

/// C1850: Inode tracking with allocation bitmap
#[test]
fn c1850_inode_tracking() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define FS_INODE_BITMAP_SIZE 32
#define FS_INODE_TOTAL (FS_INODE_BITMAP_SIZE * 32)

typedef struct {
    uint32_t bitmap[FS_INODE_BITMAP_SIZE];
    int allocated_count;
    int total_inodes;
} fs_inode_table_t;

void fs_inode_init(fs_inode_table_t *it) {
    int i;
    for (i = 0; i < FS_INODE_BITMAP_SIZE; i++) {
        it->bitmap[i] = 0;
    }
    it->allocated_count = 0;
    it->total_inodes = FS_INODE_TOTAL;
}

int fs_inode_alloc(fs_inode_table_t *it) {
    int i;
    for (i = 0; i < FS_INODE_BITMAP_SIZE; i++) {
        if (it->bitmap[i] != 0xFFFFFFFF) {
            int bit;
            for (bit = 0; bit < 32; bit++) {
                if (!(it->bitmap[i] & (1u << bit))) {
                    it->bitmap[i] |= (1u << bit);
                    it->allocated_count++;
                    return i * 32 + bit;
                }
            }
        }
    }
    return -1;
}

int fs_inode_free(fs_inode_table_t *it, int inode) {
    if (inode < 0 || inode >= it->total_inodes) return -1;
    int word = inode / 32;
    int bit = inode % 32;
    if (!(it->bitmap[word] & (1u << bit))) return -2;
    it->bitmap[word] &= ~(1u << bit);
    it->allocated_count--;
    return 0;
}

int fs_inode_is_used(fs_inode_table_t *it, int inode) {
    if (inode < 0 || inode >= it->total_inodes) return -1;
    int word = inode / 32;
    int bit = inode % 32;
    return (it->bitmap[word] & (1u << bit)) != 0;
}

int fs_inode_free_count(fs_inode_table_t *it) {
    return it->total_inodes - it->allocated_count;
}

int fs_inode_test(void) {
    fs_inode_table_t tbl;
    fs_inode_init(&tbl);
    int a = fs_inode_alloc(&tbl);
    int b = fs_inode_alloc(&tbl);
    if (a != 0 || b != 1) return -1;
    if (!fs_inode_is_used(&tbl, 0)) return -2;
    fs_inode_free(&tbl, 0);
    if (fs_inode_is_used(&tbl, 0)) return -3;
    if (fs_inode_free_count(&tbl) != FS_INODE_TOTAL - 1) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1850: Inode tracking should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1850: Output should not be empty");
    assert!(code.contains("fn fs_inode_init"), "C1850: Should contain fn fs_inode_init");
    assert!(code.contains("fn fs_inode_alloc"), "C1850: Should contain fn fs_inode_alloc");
}
