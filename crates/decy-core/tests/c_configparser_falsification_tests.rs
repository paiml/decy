//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1726-C1750: Configuration Parser Systems -- INI-style key-value stores,
//! structured/hierarchical configs, format parsing, config operations (merge,
//! override, defaults, validation), and dynamic runtime configuration.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world configuration parsing and management patterns
//! commonly found in application servers, operating systems, build tools, and
//! deployment systems -- all expressed as valid C99.
//!
//! Organization:
//! - C1726-C1730: Key-value config stores (INI parsing, key lookup, section handling, multi-value, write-back)
//! - C1731-C1735: Structured config (nested configs, config trees, hierarchical settings, schema, flatten)
//! - C1736-C1740: Format parsing (line parsing, delimiter handling, comment stripping, escape sequences, multiline)
//! - C1741-C1745: Config operations (merge, override, default values, validation, snapshot)
//! - C1746-C1750: Dynamic config (runtime reload, observer pattern, config diff, hot swap, versioning)

// ============================================================================
// C1726-C1730: Key-Value Config Stores
// ============================================================================

/// C1726: INI-style key-value parser that splits on '=' delimiter
#[test]
fn c1726_ini_key_value_parser() {
    let c_code = r##"
typedef unsigned long size_t;

int cfg_parse_key_value(const char *line, char *key, char *value) {
    int i = 0;
    int key_len = 0;
    int val_start = 0;

    while (line[i] != '\0' && line[i] == ' ') {
        i++;
    }

    while (line[i] != '\0' && line[i] != '=' && line[i] != '\n') {
        key[key_len] = line[i];
        key_len++;
        i++;
    }

    while (key_len > 0 && key[key_len - 1] == ' ') {
        key_len--;
    }
    key[key_len] = '\0';

    if (line[i] != '=') {
        value[0] = '\0';
        return 0;
    }
    i++;

    while (line[i] != '\0' && line[i] == ' ') {
        i++;
    }

    val_start = 0;
    while (line[i] != '\0' && line[i] != '\n') {
        value[val_start] = line[i];
        val_start++;
        i++;
    }
    value[val_start] = '\0';
    return 1;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1726: INI key-value parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1726: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1726: Should contain cfg_ functions");
}

/// C1727: Key lookup in a flat config store with linear search
#[test]
fn c1727_config_key_lookup() {
    let c_code = r##"
typedef unsigned long size_t;

#define CFG_MAX_ENTRIES 64
#define CFG_KEY_LEN 64
#define CFG_VAL_LEN 256

typedef struct {
    char keys[CFG_MAX_ENTRIES][CFG_KEY_LEN];
    char values[CFG_MAX_ENTRIES][CFG_VAL_LEN];
    int count;
} cfg_store_t;

void cfg_store_init(cfg_store_t *store) {
    store->count = 0;
}

int cfg_store_set(cfg_store_t *store, const char *key, const char *val) {
    int i;
    for (i = 0; i < store->count; i++) {
        int match = 1;
        int j = 0;
        while (key[j] != '\0' && store->keys[i][j] != '\0') {
            if (key[j] != store->keys[i][j]) {
                match = 0;
                break;
            }
            j++;
        }
        if (match && key[j] == '\0' && store->keys[i][j] == '\0') {
            j = 0;
            while (val[j] != '\0' && j < CFG_VAL_LEN - 1) {
                store->values[i][j] = val[j];
                j++;
            }
            store->values[i][j] = '\0';
            return 1;
        }
    }
    if (store->count >= CFG_MAX_ENTRIES) return -1;
    i = 0;
    while (key[i] != '\0' && i < CFG_KEY_LEN - 1) {
        store->keys[store->count][i] = key[i];
        i++;
    }
    store->keys[store->count][i] = '\0';
    i = 0;
    while (val[i] != '\0' && i < CFG_VAL_LEN - 1) {
        store->values[store->count][i] = val[i];
        i++;
    }
    store->values[store->count][i] = '\0';
    store->count++;
    return 0;
}

const char *cfg_store_get(cfg_store_t *store, const char *key) {
    int i;
    for (i = 0; i < store->count; i++) {
        int match = 1;
        int j = 0;
        while (key[j] != '\0' && store->keys[i][j] != '\0') {
            if (key[j] != store->keys[i][j]) {
                match = 0;
                break;
            }
            j++;
        }
        if (match && key[j] == '\0' && store->keys[i][j] == '\0') {
            return store->values[i];
        }
    }
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1727: Config key lookup should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1727: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1727: Should contain cfg_ functions");
}

/// C1728: INI section parser that tracks [section] headers
#[test]
fn c1728_ini_section_handler() {
    let c_code = r##"
typedef unsigned long size_t;

#define CFG_SECTION_MAX 32
#define CFG_NAME_LEN 64

typedef struct {
    char section_names[CFG_SECTION_MAX][CFG_NAME_LEN];
    int entry_start[CFG_SECTION_MAX];
    int entry_count[CFG_SECTION_MAX];
    int section_count;
    int current_section;
} cfg_sections_t;

void cfg_sections_init(cfg_sections_t *s) {
    s->section_count = 0;
    s->current_section = -1;
    int i;
    for (i = 0; i < CFG_SECTION_MAX; i++) {
        s->entry_start[i] = 0;
        s->entry_count[i] = 0;
    }
}

int cfg_parse_section_header(cfg_sections_t *s, const char *line) {
    if (line[0] != '[') return 0;

    int i = 1;
    int name_len = 0;
    while (line[i] != '\0' && line[i] != ']' && name_len < CFG_NAME_LEN - 1) {
        s->section_names[s->section_count][name_len] = line[i];
        name_len++;
        i++;
    }
    if (line[i] != ']') return -1;
    s->section_names[s->section_count][name_len] = '\0';
    s->current_section = s->section_count;
    s->section_count++;
    return 1;
}

int cfg_find_section(cfg_sections_t *s, const char *name) {
    int i;
    for (i = 0; i < s->section_count; i++) {
        int j = 0;
        int match = 1;
        while (name[j] != '\0') {
            if (s->section_names[i][j] != name[j]) {
                match = 0;
                break;
            }
            j++;
        }
        if (match && s->section_names[i][j] == '\0') {
            return i;
        }
    }
    return -1;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1728: INI section handler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1728: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1728: Should contain cfg_ functions");
}

/// C1729: Multi-value config entry supporting comma-separated lists
#[test]
fn c1729_multi_value_config() {
    let c_code = r##"
typedef unsigned long size_t;

#define CFG_LIST_MAX 16
#define CFG_ITEM_LEN 128

typedef struct {
    char items[CFG_LIST_MAX][CFG_ITEM_LEN];
    int count;
} cfg_value_list_t;

int cfg_parse_list_value(const char *value, cfg_value_list_t *list) {
    list->count = 0;
    int i = 0;
    int item_pos = 0;

    while (value[i] != '\0' && list->count < CFG_LIST_MAX) {
        if (value[i] == ',') {
            list->items[list->count][item_pos] = '\0';
            if (item_pos > 0) {
                list->count++;
            }
            item_pos = 0;
            i++;
            while (value[i] == ' ') i++;
            continue;
        }
        if (item_pos < CFG_ITEM_LEN - 1) {
            list->items[list->count][item_pos] = value[i];
            item_pos++;
        }
        i++;
    }
    if (item_pos > 0) {
        list->items[list->count][item_pos] = '\0';
        list->count++;
    }
    return list->count;
}

int cfg_list_contains(cfg_value_list_t *list, const char *item) {
    int i;
    for (i = 0; i < list->count; i++) {
        int j = 0;
        int match = 1;
        while (item[j] != '\0') {
            if (list->items[i][j] != item[j]) {
                match = 0;
                break;
            }
            j++;
        }
        if (match && list->items[i][j] == '\0') return 1;
    }
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1729: Multi-value config should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1729: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1729: Should contain cfg_ functions");
}

/// C1730: Config write-back that serializes a store to INI format
#[test]
fn c1730_config_writeback() {
    let c_code = r##"
typedef unsigned long size_t;

#define CFG_WB_MAX 32
#define CFG_WB_KEYLEN 64
#define CFG_WB_VALLEN 128

typedef struct {
    char keys[CFG_WB_MAX][CFG_WB_KEYLEN];
    char vals[CFG_WB_MAX][CFG_WB_VALLEN];
    int count;
} cfg_writeback_t;

void cfg_writeback_init(cfg_writeback_t *wb) {
    wb->count = 0;
}

int cfg_writeback_add(cfg_writeback_t *wb, const char *key, const char *val) {
    if (wb->count >= CFG_WB_MAX) return -1;
    int i = 0;
    while (key[i] != '\0' && i < CFG_WB_KEYLEN - 1) {
        wb->keys[wb->count][i] = key[i];
        i++;
    }
    wb->keys[wb->count][i] = '\0';
    i = 0;
    while (val[i] != '\0' && i < CFG_WB_VALLEN - 1) {
        wb->vals[wb->count][i] = val[i];
        i++;
    }
    wb->vals[wb->count][i] = '\0';
    wb->count++;
    return 0;
}

int cfg_writeback_serialize(cfg_writeback_t *wb, char *output, int max_len) {
    int pos = 0;
    int i;
    for (i = 0; i < wb->count && pos < max_len - 4; i++) {
        int j = 0;
        while (wb->keys[i][j] != '\0' && pos < max_len - 3) {
            output[pos++] = wb->keys[i][j++];
        }
        output[pos++] = '=';
        j = 0;
        while (wb->vals[i][j] != '\0' && pos < max_len - 2) {
            output[pos++] = wb->vals[i][j++];
        }
        output[pos++] = '\n';
    }
    output[pos] = '\0';
    return pos;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1730: Config write-back should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1730: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1730: Should contain cfg_ functions");
}

// ============================================================================
// C1731-C1735: Structured Config
// ============================================================================

/// C1731: Nested config with parent-child section relationships
#[test]
fn c1731_nested_config() {
    let c_code = r##"
typedef unsigned long size_t;

#define CFG_NEST_MAX 32
#define CFG_NEST_NAME 64

typedef struct {
    char names[CFG_NEST_MAX][CFG_NEST_NAME];
    int parent[CFG_NEST_MAX];
    int depth[CFG_NEST_MAX];
    int count;
} cfg_nested_t;

void cfg_nested_init(cfg_nested_t *n) {
    n->count = 0;
    int i;
    for (i = 0; i < CFG_NEST_MAX; i++) {
        n->parent[i] = -1;
        n->depth[i] = 0;
    }
}

int cfg_nested_add(cfg_nested_t *n, const char *name, int parent_idx) {
    if (n->count >= CFG_NEST_MAX) return -1;
    int i = 0;
    while (name[i] != '\0' && i < CFG_NEST_NAME - 1) {
        n->names[n->count][i] = name[i];
        i++;
    }
    n->names[n->count][i] = '\0';
    n->parent[n->count] = parent_idx;
    if (parent_idx >= 0 && parent_idx < n->count) {
        n->depth[n->count] = n->depth[parent_idx] + 1;
    } else {
        n->depth[n->count] = 0;
    }
    n->count++;
    return n->count - 1;
}

int cfg_nested_find_children(cfg_nested_t *n, int parent_idx, int *children, int max) {
    int found = 0;
    int i;
    for (i = 0; i < n->count && found < max; i++) {
        if (n->parent[i] == parent_idx) {
            children[found] = i;
            found++;
        }
    }
    return found;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1731: Nested config should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1731: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1731: Should contain cfg_ functions");
}

/// C1732: Config tree with dot-separated path traversal (e.g., "server.http.port")
#[test]
fn c1732_config_tree_path() {
    let c_code = r##"
typedef unsigned long size_t;

#define CFG_TREE_MAX 64
#define CFG_PATH_LEN 256
#define CFG_NODE_NAME 64

typedef struct {
    char node_name[CFG_TREE_MAX][CFG_NODE_NAME];
    char node_value[CFG_TREE_MAX][CFG_PATH_LEN];
    int node_parent[CFG_TREE_MAX];
    int node_count;
} cfg_tree_t;

void cfg_tree_init(cfg_tree_t *t) {
    t->node_count = 0;
}

int cfg_tree_resolve_path(cfg_tree_t *t, const char *path) {
    int current = -1;
    int seg_start = 0;
    int i = 0;
    char segment[CFG_NODE_NAME];

    while (1) {
        if (path[i] == '.' || path[i] == '\0') {
            int seg_len = i - seg_start;
            int j;
            for (j = 0; j < seg_len && j < CFG_NODE_NAME - 1; j++) {
                segment[j] = path[seg_start + j];
            }
            segment[j] = '\0';

            int found = -1;
            for (j = 0; j < t->node_count; j++) {
                if (t->node_parent[j] == current) {
                    int k = 0;
                    int match = 1;
                    while (segment[k] != '\0') {
                        if (t->node_name[j][k] != segment[k]) {
                            match = 0;
                            break;
                        }
                        k++;
                    }
                    if (match && t->node_name[j][k] == '\0') {
                        found = j;
                        break;
                    }
                }
            }
            if (found < 0) return -1;
            current = found;

            if (path[i] == '\0') break;
            seg_start = i + 1;
        }
        i++;
    }
    return current;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1732: Config tree path should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1732: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1732: Should contain cfg_ functions");
}

/// C1733: Hierarchical settings with inheritance from parent sections
#[test]
fn c1733_hierarchical_settings() {
    let c_code = r##"
typedef unsigned long size_t;

#define CFG_HIER_NODES 32
#define CFG_HIER_PROPS 8

typedef struct {
    int parent[CFG_HIER_NODES];
    int prop_values[CFG_HIER_NODES][CFG_HIER_PROPS];
    int prop_set[CFG_HIER_NODES][CFG_HIER_PROPS];
    int count;
} cfg_hierarchy_t;

void cfg_hierarchy_init(cfg_hierarchy_t *h) {
    h->count = 0;
    int i, j;
    for (i = 0; i < CFG_HIER_NODES; i++) {
        h->parent[i] = -1;
        for (j = 0; j < CFG_HIER_PROPS; j++) {
            h->prop_values[i][j] = 0;
            h->prop_set[i][j] = 0;
        }
    }
}

int cfg_hierarchy_set_prop(cfg_hierarchy_t *h, int node, int prop, int value) {
    if (node < 0 || node >= h->count) return -1;
    if (prop < 0 || prop >= CFG_HIER_PROPS) return -1;
    h->prop_values[node][prop] = value;
    h->prop_set[node][prop] = 1;
    return 0;
}

int cfg_hierarchy_resolve(cfg_hierarchy_t *h, int node, int prop) {
    if (prop < 0 || prop >= CFG_HIER_PROPS) return -1;
    int current = node;
    while (current >= 0 && current < h->count) {
        if (h->prop_set[current][prop]) {
            return h->prop_values[current][prop];
        }
        current = h->parent[current];
    }
    return -1;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1733: Hierarchical settings should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1733: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1733: Should contain cfg_ functions");
}

/// C1734: Config schema that validates types and constraints
#[test]
fn c1734_config_schema() {
    let c_code = r##"
typedef unsigned long size_t;

#define CFG_SCHEMA_MAX 32
#define CFG_SCHEMA_NAME 64

#define CFG_TYPE_INT 0
#define CFG_TYPE_BOOL 1
#define CFG_TYPE_STRING 2

typedef struct {
    char field_names[CFG_SCHEMA_MAX][CFG_SCHEMA_NAME];
    int field_types[CFG_SCHEMA_MAX];
    int field_required[CFG_SCHEMA_MAX];
    int int_min[CFG_SCHEMA_MAX];
    int int_max[CFG_SCHEMA_MAX];
    int field_count;
} cfg_schema_t;

void cfg_schema_init(cfg_schema_t *s) {
    s->field_count = 0;
}

int cfg_schema_add_int(cfg_schema_t *s, const char *name, int required, int min, int max) {
    if (s->field_count >= CFG_SCHEMA_MAX) return -1;
    int i = 0;
    while (name[i] != '\0' && i < CFG_SCHEMA_NAME - 1) {
        s->field_names[s->field_count][i] = name[i];
        i++;
    }
    s->field_names[s->field_count][i] = '\0';
    s->field_types[s->field_count] = CFG_TYPE_INT;
    s->field_required[s->field_count] = required;
    s->int_min[s->field_count] = min;
    s->int_max[s->field_count] = max;
    s->field_count++;
    return 0;
}

int cfg_schema_validate_int(cfg_schema_t *s, const char *name, int value) {
    int i;
    for (i = 0; i < s->field_count; i++) {
        int j = 0;
        int match = 1;
        while (name[j] != '\0') {
            if (s->field_names[i][j] != name[j]) {
                match = 0;
                break;
            }
            j++;
        }
        if (match && s->field_names[i][j] == '\0') {
            if (s->field_types[i] != CFG_TYPE_INT) return -2;
            if (value < s->int_min[i] || value > s->int_max[i]) return -1;
            return 0;
        }
    }
    return -3;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1734: Config schema should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1734: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1734: Should contain cfg_ functions");
}

/// C1735: Config flattener that converts nested tree to flat dot-separated keys
#[test]
fn c1735_config_flatten() {
    let c_code = r##"
typedef unsigned long size_t;

#define CFG_FLAT_MAX 64
#define CFG_FLAT_KEYLEN 256
#define CFG_FLAT_VALLEN 128

typedef struct {
    char flat_keys[CFG_FLAT_MAX][CFG_FLAT_KEYLEN];
    char flat_vals[CFG_FLAT_MAX][CFG_FLAT_VALLEN];
    int flat_count;
} cfg_flat_t;

void cfg_flat_init(cfg_flat_t *f) {
    f->flat_count = 0;
}

int cfg_flat_add(cfg_flat_t *f, const char *prefix, const char *key, const char *val) {
    if (f->flat_count >= CFG_FLAT_MAX) return -1;
    int pos = 0;
    int i = 0;

    if (prefix[0] != '\0') {
        while (prefix[i] != '\0' && pos < CFG_FLAT_KEYLEN - 2) {
            f->flat_keys[f->flat_count][pos++] = prefix[i++];
        }
        f->flat_keys[f->flat_count][pos++] = '.';
    }

    i = 0;
    while (key[i] != '\0' && pos < CFG_FLAT_KEYLEN - 1) {
        f->flat_keys[f->flat_count][pos++] = key[i++];
    }
    f->flat_keys[f->flat_count][pos] = '\0';

    i = 0;
    while (val[i] != '\0' && i < CFG_FLAT_VALLEN - 1) {
        f->flat_vals[f->flat_count][i] = val[i];
        i++;
    }
    f->flat_vals[f->flat_count][i] = '\0';
    f->flat_count++;
    return 0;
}

int cfg_flat_count_prefix(cfg_flat_t *f, const char *prefix) {
    int count = 0;
    int plen = 0;
    while (prefix[plen] != '\0') plen++;

    int i;
    for (i = 0; i < f->flat_count; i++) {
        int j;
        int match = 1;
        for (j = 0; j < plen; j++) {
            if (f->flat_keys[i][j] != prefix[j]) {
                match = 0;
                break;
            }
        }
        if (match && (f->flat_keys[i][plen] == '.' || f->flat_keys[i][plen] == '\0')) {
            count++;
        }
    }
    return count;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1735: Config flatten should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1735: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1735: Should contain cfg_ functions");
}

// ============================================================================
// C1736-C1740: Format Parsing
// ============================================================================

/// C1736: Line parser that handles different line endings (LF, CRLF)
#[test]
fn c1736_line_parser() {
    let c_code = r##"
typedef unsigned long size_t;

#define CFG_LINE_MAX 512

typedef struct {
    const char *data;
    int pos;
    int length;
} cfg_line_reader_t;

void cfg_line_reader_init(cfg_line_reader_t *r, const char *data, int length) {
    r->data = data;
    r->pos = 0;
    r->length = length;
}

int cfg_line_reader_next(cfg_line_reader_t *r, char *line_buf, int buf_size) {
    if (r->pos >= r->length) return -1;

    int out = 0;
    while (r->pos < r->length && out < buf_size - 1) {
        char c = r->data[r->pos];
        r->pos++;

        if (c == '\n') {
            break;
        }
        if (c == '\r') {
            if (r->pos < r->length && r->data[r->pos] == '\n') {
                r->pos++;
            }
            break;
        }
        line_buf[out++] = c;
    }
    line_buf[out] = '\0';
    return out;
}

int cfg_line_reader_remaining(cfg_line_reader_t *r) {
    return r->length - r->pos;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1736: Line parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1736: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1736: Should contain cfg_ functions");
}

/// C1737: Delimiter handler supporting multiple separator characters
#[test]
fn c1737_delimiter_handler() {
    let c_code = r##"
typedef unsigned long size_t;

#define CFG_DELIM_MAX 8
#define CFG_TOKEN_LEN 128

typedef struct {
    char delimiters[CFG_DELIM_MAX];
    int delim_count;
} cfg_delim_set_t;

void cfg_delim_init(cfg_delim_set_t *ds) {
    ds->delim_count = 0;
}

void cfg_delim_add(cfg_delim_set_t *ds, char delim) {
    if (ds->delim_count < CFG_DELIM_MAX) {
        ds->delimiters[ds->delim_count++] = delim;
    }
}

int cfg_delim_is_delim(cfg_delim_set_t *ds, char c) {
    int i;
    for (i = 0; i < ds->delim_count; i++) {
        if (ds->delimiters[i] == c) return 1;
    }
    return 0;
}

int cfg_delim_split_count(cfg_delim_set_t *ds, const char *input) {
    int count = 0;
    int in_token = 0;
    int i = 0;

    while (input[i] != '\0') {
        if (cfg_delim_is_delim(ds, input[i])) {
            if (in_token) {
                count++;
                in_token = 0;
            }
        } else {
            in_token = 1;
        }
        i++;
    }
    if (in_token) count++;
    return count;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1737: Delimiter handler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1737: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1737: Should contain cfg_ functions");
}

/// C1738: Comment stripping that handles # and ; style comments
#[test]
fn c1738_comment_stripping() {
    let c_code = r##"
typedef unsigned long size_t;

int cfg_strip_comment(char *line) {
    int i = 0;
    int in_quotes = 0;
    int stripped_pos = -1;

    while (line[i] != '\0') {
        if (line[i] == '"') {
            in_quotes = !in_quotes;
        }
        if (!in_quotes && (line[i] == '#' || line[i] == ';')) {
            stripped_pos = i;
            break;
        }
        i++;
    }

    if (stripped_pos >= 0) {
        while (stripped_pos > 0 && line[stripped_pos - 1] == ' ') {
            stripped_pos--;
        }
        line[stripped_pos] = '\0';
        return 1;
    }
    return 0;
}

int cfg_strip_whitespace(char *line) {
    int start = 0;
    while (line[start] == ' ' || line[start] == '\t') {
        start++;
    }

    int end = start;
    while (line[end] != '\0') end++;
    end--;
    while (end > start && (line[end] == ' ' || line[end] == '\t' || line[end] == '\n' || line[end] == '\r')) {
        end--;
    }

    int len = end - start + 1;
    if (len < 0) len = 0;
    int i;
    for (i = 0; i < len; i++) {
        line[i] = line[start + i];
    }
    line[len] = '\0';
    return len;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1738: Comment stripping should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1738: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1738: Should contain cfg_ functions");
}

/// C1739: Escape sequence handler for quoted config values
#[test]
fn c1739_escape_sequence_handler() {
    let c_code = r##"
typedef unsigned long size_t;

int cfg_unescape_value(const char *input, char *output, int max_len) {
    int i = 0;
    int o = 0;
    int in_quote = 0;

    if (input[0] == '"') {
        in_quote = 1;
        i = 1;
    }

    while (input[i] != '\0' && o < max_len - 1) {
        if (in_quote && input[i] == '"') {
            break;
        }
        if (input[i] == '\\' && input[i + 1] != '\0') {
            i++;
            switch (input[i]) {
                case 'n': output[o++] = '\n'; break;
                case 't': output[o++] = '\t'; break;
                case 'r': output[o++] = '\r'; break;
                case '\\': output[o++] = '\\'; break;
                case '"': output[o++] = '"'; break;
                default: output[o++] = '\\'; output[o++] = input[i]; break;
            }
        } else {
            output[o++] = input[i];
        }
        i++;
    }
    output[o] = '\0';
    return o;
}

int cfg_needs_escaping(const char *value) {
    int i = 0;
    while (value[i] != '\0') {
        if (value[i] == '\\' || value[i] == '"' || value[i] == '\n' ||
            value[i] == '\t' || value[i] == '#' || value[i] == ';') {
            return 1;
        }
        i++;
    }
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1739: Escape sequence handler should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1739: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1739: Should contain cfg_ functions");
}

/// C1740: Multiline value continuation with backslash line ending
#[test]
fn c1740_multiline_continuation() {
    let c_code = r##"
typedef unsigned long size_t;

#define CFG_ML_BUFSIZE 1024

typedef struct {
    char buffer[CFG_ML_BUFSIZE];
    int length;
    int complete;
} cfg_multiline_t;

void cfg_multiline_init(cfg_multiline_t *ml) {
    ml->length = 0;
    ml->complete = 0;
    ml->buffer[0] = '\0';
}

int cfg_multiline_append(cfg_multiline_t *ml, const char *line) {
    int line_len = 0;
    while (line[line_len] != '\0') line_len++;

    int ends_with_backslash = 0;
    int effective_len = line_len;
    if (line_len > 0 && line[line_len - 1] == '\\') {
        ends_with_backslash = 1;
        effective_len = line_len - 1;
    }

    if (ml->length + effective_len >= CFG_ML_BUFSIZE) {
        return -1;
    }

    int i;
    for (i = 0; i < effective_len; i++) {
        ml->buffer[ml->length + i] = line[i];
    }
    ml->length += effective_len;
    ml->buffer[ml->length] = '\0';

    if (!ends_with_backslash) {
        ml->complete = 1;
    }

    return ml->length;
}

int cfg_multiline_is_complete(cfg_multiline_t *ml) {
    return ml->complete;
}

void cfg_multiline_reset(cfg_multiline_t *ml) {
    ml->length = 0;
    ml->complete = 0;
    ml->buffer[0] = '\0';
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1740: Multiline continuation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1740: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1740: Should contain cfg_ functions");
}

// ============================================================================
// C1741-C1745: Config Operations
// ============================================================================

/// C1741: Config merge that combines two config stores with conflict resolution
#[test]
fn c1741_config_merge() {
    let c_code = r##"
typedef unsigned long size_t;

#define CFG_MERGE_MAX 32
#define CFG_MERGE_KLEN 64
#define CFG_MERGE_VLEN 128

#define CFG_MERGE_KEEP_LEFT 0
#define CFG_MERGE_KEEP_RIGHT 1

typedef struct {
    char keys[CFG_MERGE_MAX][CFG_MERGE_KLEN];
    char vals[CFG_MERGE_MAX][CFG_MERGE_VLEN];
    int count;
} cfg_merge_store_t;

void cfg_merge_store_init(cfg_merge_store_t *s) {
    s->count = 0;
}

int cfg_merge_find(cfg_merge_store_t *s, const char *key) {
    int i;
    for (i = 0; i < s->count; i++) {
        int j = 0;
        int match = 1;
        while (key[j] != '\0') {
            if (s->keys[i][j] != key[j]) { match = 0; break; }
            j++;
        }
        if (match && s->keys[i][j] == '\0') return i;
    }
    return -1;
}

int cfg_merge_stores(cfg_merge_store_t *dest, cfg_merge_store_t *src, int strategy) {
    int merged = 0;
    int i;
    for (i = 0; i < src->count; i++) {
        int existing = cfg_merge_find(dest, src->keys[i]);
        if (existing >= 0) {
            if (strategy == CFG_MERGE_KEEP_RIGHT) {
                int j = 0;
                while (src->vals[i][j] != '\0' && j < CFG_MERGE_VLEN - 1) {
                    dest->vals[existing][j] = src->vals[i][j];
                    j++;
                }
                dest->vals[existing][j] = '\0';
                merged++;
            }
        } else if (dest->count < CFG_MERGE_MAX) {
            int j = 0;
            while (src->keys[i][j] != '\0' && j < CFG_MERGE_KLEN - 1) {
                dest->keys[dest->count][j] = src->keys[i][j];
                j++;
            }
            dest->keys[dest->count][j] = '\0';
            j = 0;
            while (src->vals[i][j] != '\0' && j < CFG_MERGE_VLEN - 1) {
                dest->vals[dest->count][j] = src->vals[i][j];
                j++;
            }
            dest->vals[dest->count][j] = '\0';
            dest->count++;
            merged++;
        }
    }
    return merged;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1741: Config merge should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1741: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1741: Should contain cfg_ functions");
}

/// C1742: Config override with environment variable precedence
#[test]
fn c1742_config_override() {
    let c_code = r##"
typedef unsigned long size_t;

#define CFG_OVR_MAX 32
#define CFG_OVR_KLEN 64
#define CFG_OVR_VLEN 128

#define CFG_SRC_DEFAULT 0
#define CFG_SRC_FILE 1
#define CFG_SRC_ENV 2
#define CFG_SRC_CLI 3

typedef struct {
    char keys[CFG_OVR_MAX][CFG_OVR_KLEN];
    char vals[CFG_OVR_MAX][CFG_OVR_VLEN];
    int sources[CFG_OVR_MAX];
    int count;
} cfg_override_t;

void cfg_override_init(cfg_override_t *o) {
    o->count = 0;
}

int cfg_override_set(cfg_override_t *o, const char *key, const char *val, int source) {
    int i;
    for (i = 0; i < o->count; i++) {
        int j = 0;
        int match = 1;
        while (key[j] != '\0') {
            if (o->keys[i][j] != key[j]) { match = 0; break; }
            j++;
        }
        if (match && o->keys[i][j] == '\0') {
            if (source >= o->sources[i]) {
                j = 0;
                while (val[j] != '\0' && j < CFG_OVR_VLEN - 1) {
                    o->vals[i][j] = val[j];
                    j++;
                }
                o->vals[i][j] = '\0';
                o->sources[i] = source;
                return 1;
            }
            return 0;
        }
    }
    if (o->count >= CFG_OVR_MAX) return -1;
    i = 0;
    while (key[i] != '\0' && i < CFG_OVR_KLEN - 1) {
        o->keys[o->count][i] = key[i];
        i++;
    }
    o->keys[o->count][i] = '\0';
    i = 0;
    while (val[i] != '\0' && i < CFG_OVR_VLEN - 1) {
        o->vals[o->count][i] = val[i];
        i++;
    }
    o->vals[o->count][i] = '\0';
    o->sources[o->count] = source;
    o->count++;
    return 1;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1742: Config override should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1742: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1742: Should contain cfg_ functions");
}

/// C1743: Default value provider with fallback chains
#[test]
fn c1743_config_defaults() {
    let c_code = r##"
typedef unsigned long size_t;

#define CFG_DEF_MAX 32
#define CFG_DEF_KLEN 64

typedef struct {
    char keys[CFG_DEF_MAX][CFG_DEF_KLEN];
    int int_vals[CFG_DEF_MAX];
    int has_value[CFG_DEF_MAX];
    int count;
} cfg_defaults_t;

void cfg_defaults_init(cfg_defaults_t *d) {
    d->count = 0;
    int i;
    for (i = 0; i < CFG_DEF_MAX; i++) {
        d->has_value[i] = 0;
        d->int_vals[i] = 0;
    }
}

int cfg_defaults_register(cfg_defaults_t *d, const char *key, int default_val) {
    if (d->count >= CFG_DEF_MAX) return -1;
    int i = 0;
    while (key[i] != '\0' && i < CFG_DEF_KLEN - 1) {
        d->keys[d->count][i] = key[i];
        i++;
    }
    d->keys[d->count][i] = '\0';
    d->int_vals[d->count] = default_val;
    d->has_value[d->count] = 0;
    d->count++;
    return 0;
}

int cfg_defaults_get(cfg_defaults_t *d, const char *key, int *out_val) {
    int i;
    for (i = 0; i < d->count; i++) {
        int j = 0;
        int match = 1;
        while (key[j] != '\0') {
            if (d->keys[i][j] != key[j]) { match = 0; break; }
            j++;
        }
        if (match && d->keys[i][j] == '\0') {
            if (d->has_value[i]) {
                *out_val = d->int_vals[i];
                return 1;
            }
            *out_val = d->int_vals[i];
            return 0;
        }
    }
    return -1;
}

void cfg_defaults_set(cfg_defaults_t *d, int idx, int value) {
    if (idx >= 0 && idx < d->count) {
        d->int_vals[idx] = value;
        d->has_value[idx] = 1;
    }
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1743: Config defaults should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1743: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1743: Should contain cfg_ functions");
}

/// C1744: Config validator that checks required fields and constraints
#[test]
fn c1744_config_validation() {
    let c_code = r##"
typedef unsigned long size_t;

#define CFG_VALID_MAX 16
#define CFG_VALID_KLEN 64
#define CFG_VALID_ERRORS 32
#define CFG_VALID_ERRLEN 128

typedef struct {
    char required_keys[CFG_VALID_MAX][CFG_VALID_KLEN];
    int required_count;
    char present_keys[CFG_VALID_MAX][CFG_VALID_KLEN];
    int present_count;
    char errors[CFG_VALID_ERRORS][CFG_VALID_ERRLEN];
    int error_count;
} cfg_validator_t;

void cfg_validator_init(cfg_validator_t *v) {
    v->required_count = 0;
    v->present_count = 0;
    v->error_count = 0;
}

void cfg_validator_require(cfg_validator_t *v, const char *key) {
    if (v->required_count >= CFG_VALID_MAX) return;
    int i = 0;
    while (key[i] != '\0' && i < CFG_VALID_KLEN - 1) {
        v->required_keys[v->required_count][i] = key[i];
        i++;
    }
    v->required_keys[v->required_count][i] = '\0';
    v->required_count++;
}

void cfg_validator_mark_present(cfg_validator_t *v, const char *key) {
    if (v->present_count >= CFG_VALID_MAX) return;
    int i = 0;
    while (key[i] != '\0' && i < CFG_VALID_KLEN - 1) {
        v->present_keys[v->present_count][i] = key[i];
        i++;
    }
    v->present_keys[v->present_count][i] = '\0';
    v->present_count++;
}

int cfg_validator_check(cfg_validator_t *v) {
    v->error_count = 0;
    int r;
    for (r = 0; r < v->required_count; r++) {
        int found = 0;
        int p;
        for (p = 0; p < v->present_count; p++) {
            int j = 0;
            int match = 1;
            while (v->required_keys[r][j] != '\0') {
                if (v->present_keys[p][j] != v->required_keys[r][j]) {
                    match = 0;
                    break;
                }
                j++;
            }
            if (match && v->present_keys[p][j] == '\0') {
                found = 1;
                break;
            }
        }
        if (!found && v->error_count < CFG_VALID_ERRORS) {
            int i = 0;
            while (v->required_keys[r][i] != '\0' && i < CFG_VALID_ERRLEN - 1) {
                v->errors[v->error_count][i] = v->required_keys[r][i];
                i++;
            }
            v->errors[v->error_count][i] = '\0';
            v->error_count++;
        }
    }
    return v->error_count;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1744: Config validation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1744: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1744: Should contain cfg_ functions");
}

/// C1745: Config snapshot that captures and restores config state
#[test]
fn c1745_config_snapshot() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define CFG_SNAP_MAX 16
#define CFG_SNAP_KLEN 64
#define CFG_SNAP_VLEN 128

typedef struct {
    char keys[CFG_SNAP_MAX][CFG_SNAP_KLEN];
    char vals[CFG_SNAP_MAX][CFG_SNAP_VLEN];
    int count;
    uint32_t version;
} cfg_snapshot_t;

void cfg_snapshot_init(cfg_snapshot_t *snap) {
    snap->count = 0;
    snap->version = 0;
}

int cfg_snapshot_capture(cfg_snapshot_t *snap, const char keys[][64],
                         const char vals[][128], int count) {
    if (count > CFG_SNAP_MAX) count = CFG_SNAP_MAX;
    int i;
    for (i = 0; i < count; i++) {
        int j = 0;
        while (keys[i][j] != '\0' && j < CFG_SNAP_KLEN - 1) {
            snap->keys[i][j] = keys[i][j];
            j++;
        }
        snap->keys[i][j] = '\0';
        j = 0;
        while (vals[i][j] != '\0' && j < CFG_SNAP_VLEN - 1) {
            snap->vals[i][j] = vals[i][j];
            j++;
        }
        snap->vals[i][j] = '\0';
    }
    snap->count = count;
    snap->version++;
    return count;
}

int cfg_snapshot_restore(cfg_snapshot_t *snap, char keys[][64],
                         char vals[][128], int max_count) {
    int count = snap->count;
    if (count > max_count) count = max_count;
    int i;
    for (i = 0; i < count; i++) {
        int j = 0;
        while (snap->keys[i][j] != '\0' && j < 63) {
            keys[i][j] = snap->keys[i][j];
            j++;
        }
        keys[i][j] = '\0';
        j = 0;
        while (snap->vals[i][j] != '\0' && j < 127) {
            vals[i][j] = snap->vals[i][j];
            j++;
        }
        vals[i][j] = '\0';
    }
    return count;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1745: Config snapshot should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1745: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1745: Should contain cfg_ functions");
}

// ============================================================================
// C1746-C1750: Dynamic Config
// ============================================================================

/// C1746: Runtime config reload with generation counter for stale detection
#[test]
fn c1746_runtime_reload() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define CFG_RELOAD_MAX 32
#define CFG_RELOAD_KLEN 64
#define CFG_RELOAD_VLEN 128

typedef struct {
    char keys[CFG_RELOAD_MAX][CFG_RELOAD_KLEN];
    char vals[CFG_RELOAD_MAX][CFG_RELOAD_VLEN];
    int count;
    uint32_t generation;
    int dirty;
} cfg_reloadable_t;

void cfg_reloadable_init(cfg_reloadable_t *r) {
    r->count = 0;
    r->generation = 0;
    r->dirty = 0;
}

int cfg_reloadable_load(cfg_reloadable_t *r, const char *data) {
    r->count = 0;
    int pos = 0;

    while (data[pos] != '\0' && r->count < CFG_RELOAD_MAX) {
        int kstart = pos;
        while (data[pos] != '\0' && data[pos] != '=') pos++;
        if (data[pos] != '=') break;

        int klen = pos - kstart;
        if (klen >= CFG_RELOAD_KLEN) klen = CFG_RELOAD_KLEN - 1;
        int i;
        for (i = 0; i < klen; i++) {
            r->keys[r->count][i] = data[kstart + i];
        }
        r->keys[r->count][klen] = '\0';

        pos++;
        int vstart = pos;
        while (data[pos] != '\0' && data[pos] != '\n') pos++;

        int vlen = pos - vstart;
        if (vlen >= CFG_RELOAD_VLEN) vlen = CFG_RELOAD_VLEN - 1;
        for (i = 0; i < vlen; i++) {
            r->vals[r->count][i] = data[vstart + i];
        }
        r->vals[r->count][vlen] = '\0';

        r->count++;
        if (data[pos] == '\n') pos++;
    }

    r->generation++;
    r->dirty = 0;
    return r->count;
}

uint32_t cfg_reloadable_generation(cfg_reloadable_t *r) {
    return r->generation;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1746: Runtime reload should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1746: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1746: Should contain cfg_ functions");
}

/// C1747: Observer pattern for config change notifications
#[test]
fn c1747_config_observer() {
    let c_code = r##"
typedef unsigned long size_t;

#define CFG_OBS_MAX 8
#define CFG_OBS_KLEN 64

typedef void (*cfg_change_fn)(const char *key, const char *old_val, const char *new_val);

typedef struct {
    char watched_keys[CFG_OBS_MAX][CFG_OBS_KLEN];
    cfg_change_fn callbacks[CFG_OBS_MAX];
    int observer_count;
} cfg_observer_t;

void cfg_observer_init(cfg_observer_t *obs) {
    obs->observer_count = 0;
    int i;
    for (i = 0; i < CFG_OBS_MAX; i++) {
        obs->callbacks[i] = 0;
    }
}

int cfg_observer_watch(cfg_observer_t *obs, const char *key, cfg_change_fn callback) {
    if (obs->observer_count >= CFG_OBS_MAX) return -1;
    int i = 0;
    while (key[i] != '\0' && i < CFG_OBS_KLEN - 1) {
        obs->watched_keys[obs->observer_count][i] = key[i];
        i++;
    }
    obs->watched_keys[obs->observer_count][i] = '\0';
    obs->callbacks[obs->observer_count] = callback;
    obs->observer_count++;
    return 0;
}

void cfg_observer_notify(cfg_observer_t *obs, const char *key,
                         const char *old_val, const char *new_val) {
    int i;
    for (i = 0; i < obs->observer_count; i++) {
        int j = 0;
        int match = 1;
        while (key[j] != '\0') {
            if (obs->watched_keys[i][j] != key[j]) {
                match = 0;
                break;
            }
            j++;
        }
        if (match && obs->watched_keys[i][j] == '\0') {
            if (obs->callbacks[i]) {
                obs->callbacks[i](key, old_val, new_val);
            }
        }
    }
}

int cfg_observer_unwatch(cfg_observer_t *obs, const char *key) {
    int i;
    for (i = 0; i < obs->observer_count; i++) {
        int j = 0;
        int match = 1;
        while (key[j] != '\0') {
            if (obs->watched_keys[i][j] != key[j]) { match = 0; break; }
            j++;
        }
        if (match && obs->watched_keys[i][j] == '\0') {
            int k;
            for (k = i; k < obs->observer_count - 1; k++) {
                int m = 0;
                while (obs->watched_keys[k + 1][m] != '\0') {
                    obs->watched_keys[k][m] = obs->watched_keys[k + 1][m];
                    m++;
                }
                obs->watched_keys[k][m] = '\0';
                obs->callbacks[k] = obs->callbacks[k + 1];
            }
            obs->observer_count--;
            return 1;
        }
    }
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1747: Config observer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1747: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1747: Should contain cfg_ functions");
}

/// C1748: Config diff that computes changes between two config states
#[test]
fn c1748_config_diff() {
    let c_code = r##"
typedef unsigned long size_t;

#define CFG_DIFF_MAX 32
#define CFG_DIFF_KLEN 64
#define CFG_DIFF_VLEN 128

#define CFG_DIFF_ADDED 0
#define CFG_DIFF_REMOVED 1
#define CFG_DIFF_CHANGED 2

typedef struct {
    char keys[CFG_DIFF_MAX][CFG_DIFF_KLEN];
    char old_vals[CFG_DIFF_MAX][CFG_DIFF_VLEN];
    char new_vals[CFG_DIFF_MAX][CFG_DIFF_VLEN];
    int change_types[CFG_DIFF_MAX];
    int change_count;
} cfg_diff_t;

void cfg_diff_init(cfg_diff_t *d) {
    d->change_count = 0;
}

static int cfg_diff_streq(const char *a, const char *b) {
    int i = 0;
    while (a[i] != '\0' && b[i] != '\0') {
        if (a[i] != b[i]) return 0;
        i++;
    }
    return a[i] == b[i];
}

int cfg_diff_compute(cfg_diff_t *d,
                     const char old_keys[][64], const char old_vals[][128], int old_count,
                     const char new_keys[][64], const char new_vals[][128], int new_count) {
    d->change_count = 0;
    int i, j;

    for (i = 0; i < old_count && d->change_count < CFG_DIFF_MAX; i++) {
        int found = 0;
        for (j = 0; j < new_count; j++) {
            if (cfg_diff_streq(old_keys[i], new_keys[j])) {
                found = 1;
                if (!cfg_diff_streq(old_vals[i], new_vals[j])) {
                    int k = 0;
                    while (old_keys[i][k] != '\0' && k < CFG_DIFF_KLEN - 1) {
                        d->keys[d->change_count][k] = old_keys[i][k];
                        k++;
                    }
                    d->keys[d->change_count][k] = '\0';
                    d->change_types[d->change_count] = CFG_DIFF_CHANGED;
                    d->change_count++;
                }
                break;
            }
        }
        if (!found && d->change_count < CFG_DIFF_MAX) {
            int k = 0;
            while (old_keys[i][k] != '\0' && k < CFG_DIFF_KLEN - 1) {
                d->keys[d->change_count][k] = old_keys[i][k];
                k++;
            }
            d->keys[d->change_count][k] = '\0';
            d->change_types[d->change_count] = CFG_DIFF_REMOVED;
            d->change_count++;
        }
    }

    for (j = 0; j < new_count && d->change_count < CFG_DIFF_MAX; j++) {
        int found = 0;
        for (i = 0; i < old_count; i++) {
            if (cfg_diff_streq(old_keys[i], new_keys[j])) {
                found = 1;
                break;
            }
        }
        if (!found) {
            int k = 0;
            while (new_keys[j][k] != '\0' && k < CFG_DIFF_KLEN - 1) {
                d->keys[d->change_count][k] = new_keys[j][k];
                k++;
            }
            d->keys[d->change_count][k] = '\0';
            d->change_types[d->change_count] = CFG_DIFF_ADDED;
            d->change_count++;
        }
    }
    return d->change_count;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1748: Config diff should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1748: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1748: Should contain cfg_ functions");
}

/// C1749: Hot-swap config with double-buffering for lock-free reads
#[test]
fn c1749_config_hot_swap() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define CFG_SWAP_MAX 16
#define CFG_SWAP_KLEN 64
#define CFG_SWAP_VLEN 128

typedef struct {
    char keys[CFG_SWAP_MAX][CFG_SWAP_KLEN];
    char vals[CFG_SWAP_MAX][CFG_SWAP_VLEN];
    int count;
} cfg_buffer_t;

typedef struct {
    cfg_buffer_t buffers[2];
    int active;
    uint32_t swap_count;
} cfg_hot_swap_t;

void cfg_hot_swap_init(cfg_hot_swap_t *hs) {
    hs->active = 0;
    hs->swap_count = 0;
    hs->buffers[0].count = 0;
    hs->buffers[1].count = 0;
}

cfg_buffer_t *cfg_hot_swap_read(cfg_hot_swap_t *hs) {
    return &hs->buffers[hs->active];
}

cfg_buffer_t *cfg_hot_swap_write_buffer(cfg_hot_swap_t *hs) {
    return &hs->buffers[1 - hs->active];
}

void cfg_hot_swap_commit(cfg_hot_swap_t *hs) {
    hs->active = 1 - hs->active;
    hs->swap_count++;
}

int cfg_hot_swap_load_into_back(cfg_hot_swap_t *hs, const char keys[][64],
                                const char vals[][128], int count) {
    cfg_buffer_t *back = cfg_hot_swap_write_buffer(hs);
    if (count > CFG_SWAP_MAX) count = CFG_SWAP_MAX;
    int i;
    for (i = 0; i < count; i++) {
        int j = 0;
        while (keys[i][j] != '\0' && j < CFG_SWAP_KLEN - 1) {
            back->keys[i][j] = keys[i][j];
            j++;
        }
        back->keys[i][j] = '\0';
        j = 0;
        while (vals[i][j] != '\0' && j < CFG_SWAP_VLEN - 1) {
            back->vals[i][j] = vals[i][j];
            j++;
        }
        back->vals[i][j] = '\0';
    }
    back->count = count;
    return count;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1749: Config hot swap should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1749: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1749: Should contain cfg_ functions");
}

/// C1750: Config versioning with history of changes and rollback
#[test]
fn c1750_config_versioning() {
    let c_code = r##"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

#define CFG_VER_MAX 16
#define CFG_VER_KLEN 64
#define CFG_VER_VLEN 128
#define CFG_VER_HISTORY 8

typedef struct {
    char keys[CFG_VER_MAX][CFG_VER_KLEN];
    char vals[CFG_VER_MAX][CFG_VER_VLEN];
    int count;
    uint32_t version;
} cfg_version_snap_t;

typedef struct {
    cfg_version_snap_t history[CFG_VER_HISTORY];
    int history_count;
    int current_idx;
    uint32_t next_version;
} cfg_versioned_t;

void cfg_versioned_init(cfg_versioned_t *v) {
    v->history_count = 0;
    v->current_idx = -1;
    v->next_version = 1;
}

int cfg_versioned_commit(cfg_versioned_t *v, const char keys[][64],
                         const char vals[][128], int count) {
    if (v->history_count >= CFG_VER_HISTORY) {
        int i;
        for (i = 0; i < CFG_VER_HISTORY - 1; i++) {
            v->history[i] = v->history[i + 1];
        }
        v->history_count = CFG_VER_HISTORY - 1;
    }

    int idx = v->history_count;
    if (count > CFG_VER_MAX) count = CFG_VER_MAX;
    int i;
    for (i = 0; i < count; i++) {
        int j = 0;
        while (keys[i][j] != '\0' && j < CFG_VER_KLEN - 1) {
            v->history[idx].keys[i][j] = keys[i][j];
            j++;
        }
        v->history[idx].keys[i][j] = '\0';
        j = 0;
        while (vals[i][j] != '\0' && j < CFG_VER_VLEN - 1) {
            v->history[idx].vals[i][j] = vals[i][j];
            j++;
        }
        v->history[idx].vals[i][j] = '\0';
    }
    v->history[idx].count = count;
    v->history[idx].version = v->next_version++;
    v->current_idx = idx;
    v->history_count++;
    return v->history[idx].version;
}

int cfg_versioned_rollback(cfg_versioned_t *v) {
    if (v->current_idx <= 0) return -1;
    v->current_idx--;
    return v->history[v->current_idx].version;
}

int cfg_versioned_current_version(cfg_versioned_t *v) {
    if (v->current_idx < 0) return 0;
    return v->history[v->current_idx].version;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1750: Config versioning should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1750: Output should not be empty");
    assert!(code.contains("fn cfg_"), "C1750: Should contain cfg_ functions");
}
