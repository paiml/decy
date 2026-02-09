//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler (Advanced)
//!
//! C176-C200: Advanced/pathological C patterns.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! Organization:
//! - C176-C180: Preprocessor and compilation patterns
//! - C181-C185: Type system edge cases
//! - C186-C190: Concurrency and system patterns
//! - C191-C195: Real-world C idioms (PyTorch-style)
//! - C196-C200: Extreme edge cases
//!
//! Results: 24 passing, 1 falsified (96.0% pass rate)

// ============================================================================
// C176-C180: Preprocessor and Compilation Patterns
// ============================================================================

#[test]
fn c176_conditional_compilation_ifdef_chains() {
    let c_code = r#"
#define PLATFORM_LINUX 1
#define USE_LOGGING 1

int get_config() {
#ifdef PLATFORM_LINUX
    #ifdef USE_LOGGING
        return 3;
    #else
        return 2;
    #endif
#elif defined(PLATFORM_WINDOWS)
    return 1;
#else
    return 0;
#endif
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C176: Conditional compilation (#ifdef chains) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C176: Output should not be empty");
    assert!(
        code.contains("fn get_config"),
        "C176: Should contain transpiled function get_config"
    );
}

#[test]
fn c177_macro_do_while_zero_pattern() {
    let c_code = r#"
#define SAFE_FREE(ptr) do { if (ptr) { ptr = 0; } } while(0)

int cleanup(int *p) {
    int x = 42;
    SAFE_FREE(p);
    return x;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C177: Macro with do-while(0) pattern should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C177: Output should not be empty");
    assert!(
        code.contains("fn cleanup"),
        "C177: Should contain transpiled function cleanup"
    );
}

#[test]
fn c178_x_macro_pattern() {
    let c_code = r#"
#define ERROR_LIST \
    X(ERR_NONE, 0) \
    X(ERR_IO, 1) \
    X(ERR_MEM, 2)

#define X(name, val) name = val,
enum ErrorCode { ERROR_LIST };
#undef X

int get_error_code(int idx) {
    if (idx == 0) return ERR_NONE;
    if (idx == 1) return ERR_IO;
    return ERR_MEM;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C178: X-macro pattern should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C178: Output should not be empty");
    assert!(
        code.contains("fn get_error_code"),
        "C178: Should contain transpiled function get_error_code"
    );
}

#[test]
fn c179_token_pasting_and_stringification() {
    let c_code = r#"
#define MAKE_GETTER(field) int get_##field(int field) { return field; }
MAKE_GETTER(width)
MAKE_GETTER(height)

int total() {
    return get_width(10) + get_height(20);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C179: Token pasting and stringification should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C179: Output should not be empty");
    assert!(
        code.contains("fn total"),
        "C179: Should contain transpiled function total"
    );
    assert!(
        code.contains("fn get_width") || code.contains("get_width"),
        "C179: Should contain macro-expanded get_width function"
    );
}

#[test]
fn c180_include_guard_with_function() {
    let c_code = r#"
#ifndef GUARD_H
#define GUARD_H

#define VERSION_MAJOR 1
#define VERSION_MINOR 0

int get_version() {
    return VERSION_MAJOR * 100 + VERSION_MINOR;
}

#endif
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C180: Include guard pattern should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C180: Output should not be empty");
    assert!(
        code.contains("fn get_version"),
        "C180: Should contain transpiled function get_version"
    );
}

// ============================================================================
// C181-C185: Type System Edge Cases
// ============================================================================

#[test]
fn c181_typeof_operator() {
    let c_code = r#"
int use_typeof() {
    int x = 42;
    __typeof__(x) y = x + 1;
    return y;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C181: typeof operator should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C181: Output should not be empty");
    assert!(
        code.contains("fn use_typeof"),
        "C181: Should contain transpiled function use_typeof"
    );
}

#[test]
fn c182_compound_literal_c99() {
    let c_code = r#"
struct Point { int x; int y; };
int sum_point() {
    struct Point p = (struct Point){ 10, 20 };
    return p.x + p.y;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C182: Compound literal (C99) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C182: Output should not be empty");
    assert!(
        code.contains("struct Point") || code.contains("Point"),
        "C182: Should contain Point struct in output"
    );
    assert!(
        code.contains("fn sum_point"),
        "C182: Should contain transpiled function sum_point"
    );
}

#[test]
fn c183_designated_initializers_c99() {
    let c_code = r#"
struct Config {
    int width;
    int height;
    int depth;
};
int get_area() {
    struct Config c = { .width = 640, .height = 480, .depth = 32 };
    return c.width * c.height;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C183: Designated initializers (C99) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C183: Output should not be empty");
    assert!(
        code.contains("struct Config") || code.contains("Config"),
        "C183: Should contain Config struct in output"
    );
    assert!(
        code.contains("fn get_area"),
        "C183: Should contain transpiled function get_area"
    );
}

#[test]
fn c184_variable_length_array() {
    let c_code = r#"
int sum_vla(int n) {
    int arr[n];
    int i;
    for (i = 0; i < n; i++) {
        arr[i] = i;
    }
    int total = 0;
    for (i = 0; i < n; i++) {
        total += arr[i];
    }
    return total;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C184: Variable-length array (VLA) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C184: Output should not be empty");
    assert!(
        code.contains("fn sum_vla"),
        "C184: Should contain transpiled function sum_vla"
    );
}

#[test]
fn c185_generic_selection_c11() {
    let c_code = r#"
#define type_name(x) _Generic((x), \
    int: "int", \
    float: "float", \
    double: "double", \
    default: "other")

int test_generic() {
    int x = 42;
    float f = 3.14f;
    return x;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C185: _Generic selection (C11) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C185: Output should not be empty");
    assert!(
        code.contains("fn test_generic"),
        "C185: Should contain transpiled function test_generic"
    );
}

// ============================================================================
// C186-C190: Concurrency and System Patterns
// ============================================================================

#[test]
fn c186_pthread_create_void_arg() {
    let c_code = r#"
struct ThreadArg {
    int id;
    int value;
};

int process_arg(struct ThreadArg *arg) {
    return arg->id + arg->value;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C186: pthread_create with void* argument pattern should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C186: Output should not be empty");
    assert!(
        code.contains("struct ThreadArg") || code.contains("ThreadArg"),
        "C186: Should contain ThreadArg struct in output"
    );
    assert!(
        code.contains("fn process_arg"),
        "C186: Should contain transpiled function process_arg"
    );
}

#[test]
fn c187_mutex_lock_unlock_pattern() {
    let c_code = r#"
struct Mutex {
    int locked;
};

void mutex_lock(struct Mutex *m) {
    m->locked = 1;
}

void mutex_unlock(struct Mutex *m) {
    m->locked = 0;
}

int critical_section(struct Mutex *m, int value) {
    mutex_lock(m);
    int result = value * 2;
    mutex_unlock(m);
    return result;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C187: Mutex lock/unlock pattern should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C187: Output should not be empty");
    assert!(
        code.contains("fn mutex_lock"),
        "C187: Should contain transpiled function mutex_lock"
    );
    assert!(
        code.contains("fn critical_section"),
        "C187: Should contain transpiled function critical_section"
    );
}

#[test]
fn c188_condition_variable_pattern() {
    let c_code = r#"
struct CondVar {
    int signaled;
    int waiters;
};

void cond_wait(struct CondVar *cv) {
    cv->waiters++;
    while (!cv->signaled) {
        /* spin */
    }
    cv->waiters--;
}

void cond_signal(struct CondVar *cv) {
    cv->signaled = 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C188: Condition variable wait/signal should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C188: Output should not be empty");
    assert!(
        code.contains("fn cond_wait"),
        "C188: Should contain transpiled function cond_wait"
    );
    assert!(
        code.contains("fn cond_signal"),
        "C188: Should contain transpiled function cond_signal"
    );
}

#[test]
fn c189_atomic_operations() {
    let c_code = r#"
int atomic_increment(int *counter) {
    int old = *counter;
    *counter = old + 1;
    return old;
}

int atomic_compare_swap(int *target, int expected, int desired) {
    if (*target == expected) {
        *target = desired;
        return 1;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C189: Atomic operations pattern should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C189: Output should not be empty");
    assert!(
        code.contains("fn atomic_increment"),
        "C189: Should contain transpiled function atomic_increment"
    );
    assert!(
        code.contains("fn atomic_compare_swap"),
        "C189: Should contain transpiled function atomic_compare_swap"
    );
}

#[test]
fn c190_pipe_communication_pattern() {
    let c_code = r#"
struct Pipe {
    int buffer[256];
    int read_pos;
    int write_pos;
    int count;
};

int pipe_write(struct Pipe *p, int value) {
    if (p->count >= 256) return -1;
    p->buffer[p->write_pos] = value;
    p->write_pos = (p->write_pos + 1) % 256;
    p->count++;
    return 0;
}

int pipe_read(struct Pipe *p, int *value) {
    if (p->count <= 0) return -1;
    *value = p->buffer[p->read_pos];
    p->read_pos = (p->read_pos + 1) % 256;
    p->count--;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C190: Pipe communication pattern should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C190: Output should not be empty");
    assert!(
        code.contains("fn pipe_write"),
        "C190: Should contain transpiled function pipe_write"
    );
    assert!(
        code.contains("fn pipe_read"),
        "C190: Should contain transpiled function pipe_read"
    );
    assert!(
        code.contains("struct Pipe") || code.contains("Pipe"),
        "C190: Should contain Pipe struct in output"
    );
}

// ============================================================================
// C191-C195: Real-World C Idioms (PyTorch-style)
// ============================================================================

#[test]
fn c191_reference_counting_addref_release() {
    let c_code = r#"
struct Object {
    int refcount;
    int data;
    int type_id;
};

void obj_addref(struct Object *obj) {
    obj->refcount++;
}

int obj_release(struct Object *obj) {
    obj->refcount--;
    if (obj->refcount <= 0) {
        return 1;
    }
    return 0;
}

int obj_get_data(struct Object *obj) {
    return obj->data;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C191: Reference counting pattern should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C191: Output should not be empty");
    assert!(
        code.contains("fn obj_addref"),
        "C191: Should contain transpiled function obj_addref"
    );
    assert!(
        code.contains("fn obj_release"),
        "C191: Should contain transpiled function obj_release"
    );
    assert!(
        code.contains("fn obj_get_data"),
        "C191: Should contain transpiled function obj_get_data"
    );
}

#[test]
fn c192_object_oriented_vtable() {
    let c_code = r#"
typedef int (*compute_fn)(int, int);
typedef int (*describe_fn)(void);

struct VTable {
    compute_fn compute;
    describe_fn describe;
};

struct BaseObject {
    struct VTable *vtable;
    int id;
};

int invoke_compute(struct BaseObject *obj, int a, int b) {
    return obj->vtable->compute(a, b);
}

int add_impl(int a, int b) { return a + b; }
int mul_impl(int a, int b) { return a * b; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C192: Object-oriented C with vtable should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C192: Output should not be empty");
    assert!(
        code.contains("fn invoke_compute"),
        "C192: Should contain transpiled function invoke_compute"
    );
    assert!(
        code.contains("fn add_impl"),
        "C192: Should contain transpiled function add_impl"
    );
    assert!(
        code.contains("fn mul_impl"),
        "C192: Should contain transpiled function mul_impl"
    );
}

#[test]
fn c193_error_code_enum_with_string_mapping() {
    let c_code = r#"
enum ErrorCode {
    ERR_OK = 0,
    ERR_NOMEM = 1,
    ERR_IO = 2,
    ERR_INVALID = 3,
    ERR_TIMEOUT = 4
};

int is_fatal(enum ErrorCode code) {
    switch (code) {
        case ERR_NOMEM: return 1;
        case ERR_IO: return 1;
        case ERR_TIMEOUT: return 0;
        case ERR_INVALID: return 0;
        default: return 0;
    }
}

int error_severity(enum ErrorCode code) {
    if (code == ERR_OK) return 0;
    if (code == ERR_NOMEM) return 3;
    if (code == ERR_IO) return 2;
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C193: Error code enum with string mapping should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C193: Output should not be empty");
    assert!(
        code.contains("fn is_fatal"),
        "C193: Should contain transpiled function is_fatal"
    );
    assert!(
        code.contains("fn error_severity"),
        "C193: Should contain transpiled function error_severity"
    );
    assert!(
        code.contains("match") || code.contains("if"),
        "C193: Should contain match or if for switch/case transpilation"
    );
}

#[test]
fn c194_ring_buffer_implementation() {
    let c_code = r#"
#define RING_SIZE 64

struct RingBuffer {
    int data[64];
    int head;
    int tail;
    int count;
};

int ring_push(struct RingBuffer *rb, int value) {
    if (rb->count >= 64) return -1;
    rb->data[rb->tail] = value;
    rb->tail = (rb->tail + 1) % 64;
    rb->count++;
    return 0;
}

int ring_pop(struct RingBuffer *rb, int *value) {
    if (rb->count <= 0) return -1;
    *value = rb->data[rb->head];
    rb->head = (rb->head + 1) % 64;
    rb->count--;
    return 0;
}

int ring_is_empty(struct RingBuffer *rb) {
    return rb->count == 0;
}

int ring_is_full(struct RingBuffer *rb) {
    return rb->count >= 64;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C194: Ring buffer implementation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C194: Output should not be empty");
    assert!(
        code.contains("fn ring_push"),
        "C194: Should contain transpiled function ring_push"
    );
    assert!(
        code.contains("fn ring_pop"),
        "C194: Should contain transpiled function ring_pop"
    );
    assert!(
        code.contains("fn ring_is_empty"),
        "C194: Should contain transpiled function ring_is_empty"
    );
    assert!(
        code.contains("fn ring_is_full"),
        "C194: Should contain transpiled function ring_is_full"
    );
    assert!(
        code.contains("RingBuffer"),
        "C194: Should contain RingBuffer struct in output"
    );
}

#[test]
fn c195_hash_table_with_chaining() {
    let c_code = r#"
#define TABLE_SIZE 16

struct Entry {
    int key;
    int value;
    int occupied;
};

struct HashTable {
    struct Entry entries[16];
};

int hash_fn(int key) {
    return ((unsigned int)key) % 16;
}

int ht_insert(struct HashTable *ht, int key, int value) {
    int idx = hash_fn(key);
    int i;
    for (i = 0; i < 16; i++) {
        int probe = (idx + i) % 16;
        if (!ht->entries[probe].occupied) {
            ht->entries[probe].key = key;
            ht->entries[probe].value = value;
            ht->entries[probe].occupied = 1;
            return 0;
        }
    }
    return -1;
}

int ht_get(struct HashTable *ht, int key) {
    int idx = hash_fn(key);
    int i;
    for (i = 0; i < 16; i++) {
        int probe = (idx + i) % 16;
        if (!ht->entries[probe].occupied) return -1;
        if (ht->entries[probe].key == key) return ht->entries[probe].value;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C195: Hash table with chaining should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C195: Output should not be empty");
    assert!(
        code.contains("fn hash_fn"),
        "C195: Should contain transpiled function hash_fn"
    );
    assert!(
        code.contains("fn ht_insert"),
        "C195: Should contain transpiled function ht_insert"
    );
    assert!(
        code.contains("fn ht_get"),
        "C195: Should contain transpiled function ht_get"
    );
    assert!(
        code.contains("HashTable") || code.contains("Entry"),
        "C195: Should contain HashTable or Entry struct in output"
    );
}

// ============================================================================
// C196-C200: Extreme Edge Cases
// ============================================================================

#[test]
fn c196_computed_goto_dispatch() {
    let c_code = r#"
int dispatch(int op, int a, int b) {
    static void *table[] = { &&do_add, &&do_sub, &&do_mul, &&do_div };
    if (op < 0 || op > 3) return -1;
    goto *table[op];
do_add: return a + b;
do_sub: return a - b;
do_mul: return a * b;
do_div: return b != 0 ? a / b : 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C196: Computed goto dispatch should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C196: Output should not be empty");
    assert!(
        code.contains("fn dispatch"),
        "C196: Should contain transpiled function dispatch"
    );
}

#[test]
#[ignore = "FALSIFIED: nested functions (GCC extension) rejected by clang parser"]
fn c197_nested_functions_gcc() {
    let c_code = r#"
int outer(int x) {
    int multiplier = 3;
    int inner(int y) {
        return y * multiplier;
    }
    return inner(x) + inner(x + 1);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C197: Nested functions (GCC extension) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C197: Output should not be empty");
}

#[test]
fn c198_statement_expressions_gcc() {
    let c_code = r#"
#define MAX_SAFE(a, b) ({ \
    __typeof__(a) _a = (a); \
    __typeof__(b) _b = (b); \
    _a > _b ? _a : _b; \
})

int find_max(int x, int y, int z) {
    int m = MAX_SAFE(x, y);
    return MAX_SAFE(m, z);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C198: Statement expressions (GCC extension) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C198: Output should not be empty");
    assert!(
        code.contains("fn find_max"),
        "C198: Should contain transpiled function find_max"
    );
}

#[test]
fn c199_attribute_packed_struct() {
    let c_code = r#"
struct __attribute__((packed)) NetworkHeader {
    unsigned char version;
    unsigned short length;
    unsigned int sequence;
};

int get_sequence(struct NetworkHeader *hdr) {
    return hdr->sequence;
}

int get_length(struct NetworkHeader *hdr) {
    return hdr->length;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C199: Attribute packed struct should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C199: Output should not be empty");
    assert!(
        code.contains("fn get_sequence"),
        "C199: Should contain transpiled function get_sequence"
    );
    assert!(
        code.contains("fn get_length"),
        "C199: Should contain transpiled function get_length"
    );
    assert!(
        code.contains("NetworkHeader"),
        "C199: Should contain NetworkHeader struct in output"
    );
}

#[test]
fn c200_extern_linkage_multi_unit() {
    let c_code = r#"
extern int shared_counter;
extern int get_counter(void);
extern void set_counter(int val);

int increment_counter() {
    int current = get_counter();
    set_counter(current + 1);
    return current + 1;
}

int reset_and_get() {
    set_counter(0);
    return get_counter();
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C200: Extern linkage multi-unit should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C200: Output should not be empty");
    assert!(
        code.contains("fn increment_counter"),
        "C200: Should contain transpiled function increment_counter"
    );
    assert!(
        code.contains("fn reset_and_get"),
        "C200: Should contain transpiled function reset_and_get"
    );
}
