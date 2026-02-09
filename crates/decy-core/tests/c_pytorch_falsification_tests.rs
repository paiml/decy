//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C226-C250: PyTorch/ML-framework-inspired pure C patterns.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise complex, real-world C patterns commonly found
//! in ML framework internals (memory pools, tensor operations, custom
//! allocators, data structures for graph computation, etc.) -- all
//! expressed as valid C99.
//!
//! Organization:
//! - C226-C230: Core computation and data structure patterns
//! - C231-C235: Memory management and allocation patterns
//! - C236-C240: System-level and error handling patterns
//! - C241-C245: Advanced data structure patterns
//! - C246-C250: ML-specific numeric and graph patterns
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C226-C230: Core Computation and Data Structure Patterns
// ============================================================================

#[test]
fn c226_dense_matrix_multiply() {
    let c_code = r#"
void matmul(const float *a, const float *b, float *c, int m, int n, int k) {
    for (int i = 0; i < m; i++) {
        for (int j = 0; j < n; j++) {
            float sum = 0.0f;
            for (int p = 0; p < k; p++) {
                sum += a[i * k + p] * b[p * n + j];
            }
            c[i * n + j] = sum;
        }
    }
}

float dot_product(const float *x, const float *y, int len) {
    float result = 0.0f;
    for (int i = 0; i < len; i++) {
        result += x[i] * y[i];
    }
    return result;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C226: Dense matrix multiply should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C226: Output should not be empty");
    assert!(
        code.contains("fn matmul"),
        "C226: Should contain transpiled function matmul"
    );
    assert!(
        code.contains("fn dot_product"),
        "C226: Should contain transpiled function dot_product"
    );
}

#[test]
fn c227_tensor_shape_dynamic_indexing() {
    let c_code = r#"
#define MAX_DIMS 8

typedef struct {
    int dims[MAX_DIMS];
    int ndim;
} TensorShape;

int tensor_numel(const TensorShape *shape) {
    int total = 1;
    for (int i = 0; i < shape->ndim; i++) {
        total *= shape->dims[i];
    }
    return total;
}

int tensor_offset(const TensorShape *shape, const int *indices) {
    int offset = 0;
    int stride = 1;
    for (int i = shape->ndim - 1; i >= 0; i--) {
        offset += indices[i] * stride;
        stride *= shape->dims[i];
    }
    return offset;
}

int tensor_broadcast_compatible(const TensorShape *a, const TensorShape *b) {
    int max_ndim = a->ndim > b->ndim ? a->ndim : b->ndim;
    for (int i = 0; i < max_ndim; i++) {
        int da = (i < a->ndim) ? a->dims[a->ndim - 1 - i] : 1;
        int db = (i < b->ndim) ? b->dims[b->ndim - 1 - i] : 1;
        if (da != db && da != 1 && db != 1) {
            return 0;
        }
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C227: Tensor shape with dynamic indexing should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C227: Output should not be empty");
    assert!(
        code.contains("fn tensor_numel"),
        "C227: Should contain transpiled function tensor_numel"
    );
    assert!(
        code.contains("fn tensor_offset"),
        "C227: Should contain transpiled function tensor_offset"
    );
}

#[test]
fn c228_memory_pool_allocator() {
    let c_code = r#"
#include <stdlib.h>
#include <string.h>

#define POOL_BLOCK_SIZE 4096

typedef struct PoolBlock {
    char data[POOL_BLOCK_SIZE];
    int used;
    struct PoolBlock *next;
} PoolBlock;

typedef struct {
    PoolBlock *head;
    int total_allocated;
} MemoryPool;

MemoryPool *pool_create(void) {
    MemoryPool *pool = (MemoryPool *)malloc(sizeof(MemoryPool));
    if (!pool) return NULL;
    pool->head = NULL;
    pool->total_allocated = 0;
    return pool;
}

void *pool_alloc(MemoryPool *pool, int size) {
    if (size > POOL_BLOCK_SIZE) return NULL;
    PoolBlock *block = pool->head;
    while (block) {
        if (block->used + size <= POOL_BLOCK_SIZE) {
            void *ptr = &block->data[block->used];
            block->used += size;
            pool->total_allocated += size;
            return ptr;
        }
        block = block->next;
    }
    PoolBlock *new_block = (PoolBlock *)malloc(sizeof(PoolBlock));
    if (!new_block) return NULL;
    new_block->used = size;
    new_block->next = pool->head;
    pool->head = new_block;
    pool->total_allocated += size;
    return &new_block->data[0];
}

void pool_destroy(MemoryPool *pool) {
    PoolBlock *block = pool->head;
    while (block) {
        PoolBlock *next = block->next;
        free(block);
        block = next;
    }
    free(pool);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C228: Memory pool allocator should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C228: Output should not be empty");
    assert!(
        code.contains("fn pool_create"),
        "C228: Should contain transpiled function pool_create"
    );
    assert!(
        code.contains("fn pool_alloc"),
        "C228: Should contain transpiled function pool_alloc"
    );
    assert!(
        code.contains("fn pool_destroy"),
        "C228: Should contain transpiled function pool_destroy"
    );
}

#[test]
fn c229_ring_buffer() {
    let c_code = r#"
#define RING_CAPACITY 256

typedef struct {
    float data[RING_CAPACITY];
    int head;
    int tail;
    int count;
} RingBuffer;

void ring_init(RingBuffer *rb) {
    rb->head = 0;
    rb->tail = 0;
    rb->count = 0;
}

int ring_push(RingBuffer *rb, float value) {
    if (rb->count >= RING_CAPACITY) return -1;
    rb->data[rb->tail] = value;
    rb->tail = (rb->tail + 1) % RING_CAPACITY;
    rb->count++;
    return 0;
}

int ring_pop(RingBuffer *rb, float *out) {
    if (rb->count <= 0) return -1;
    *out = rb->data[rb->head];
    rb->head = (rb->head + 1) % RING_CAPACITY;
    rb->count--;
    return 0;
}

float ring_peek(const RingBuffer *rb, int offset) {
    if (offset < 0 || offset >= rb->count) return 0.0f;
    int idx = (rb->head + offset) % RING_CAPACITY;
    return rb->data[idx];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C229: Ring buffer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C229: Output should not be empty");
    assert!(
        code.contains("fn ring_init"),
        "C229: Should contain transpiled function ring_init"
    );
    assert!(
        code.contains("fn ring_push"),
        "C229: Should contain transpiled function ring_push"
    );
    assert!(
        code.contains("fn ring_pop"),
        "C229: Should contain transpiled function ring_pop"
    );
}

#[test]
fn c230_hash_table_with_chaining() {
    let c_code = r#"
#include <stdlib.h>
#include <string.h>

#define TABLE_SIZE 64

typedef struct Entry {
    int key;
    float value;
    struct Entry *next;
} Entry;

typedef struct {
    Entry *buckets[TABLE_SIZE];
    int size;
} HashTable;

int hash_func(int key) {
    unsigned int h = (unsigned int)key;
    h = ((h >> 16) ^ h) * 0x45d9f3b;
    h = ((h >> 16) ^ h) * 0x45d9f3b;
    h = (h >> 16) ^ h;
    return (int)(h % TABLE_SIZE);
}

HashTable *ht_create(void) {
    HashTable *ht = (HashTable *)malloc(sizeof(HashTable));
    if (!ht) return NULL;
    ht->size = 0;
    for (int i = 0; i < TABLE_SIZE; i++) {
        ht->buckets[i] = NULL;
    }
    return ht;
}

int ht_insert(HashTable *ht, int key, float value) {
    int idx = hash_func(key);
    Entry *e = ht->buckets[idx];
    while (e) {
        if (e->key == key) {
            e->value = value;
            return 0;
        }
        e = e->next;
    }
    Entry *new_entry = (Entry *)malloc(sizeof(Entry));
    if (!new_entry) return -1;
    new_entry->key = key;
    new_entry->value = value;
    new_entry->next = ht->buckets[idx];
    ht->buckets[idx] = new_entry;
    ht->size++;
    return 0;
}

float ht_get(const HashTable *ht, int key, int *found) {
    int idx = hash_func(key);
    Entry *e = ht->buckets[idx];
    while (e) {
        if (e->key == key) {
            *found = 1;
            return e->value;
        }
        e = e->next;
    }
    *found = 0;
    return 0.0f;
}

void ht_destroy(HashTable *ht) {
    for (int i = 0; i < TABLE_SIZE; i++) {
        Entry *e = ht->buckets[i];
        while (e) {
            Entry *next = e->next;
            free(e);
            e = next;
        }
    }
    free(ht);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C230: Hash table with chaining should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C230: Output should not be empty");
    assert!(
        code.contains("fn hash_func"),
        "C230: Should contain transpiled function hash_func"
    );
    assert!(
        code.contains("fn ht_create"),
        "C230: Should contain transpiled function ht_create"
    );
    assert!(
        code.contains("fn ht_insert"),
        "C230: Should contain transpiled function ht_insert"
    );
}

// ============================================================================
// C231-C235: Memory Management and Allocation Patterns
// ============================================================================

#[test]
fn c231_dynamic_array_realloc() {
    let c_code = r#"
#include <stdlib.h>
#include <string.h>

typedef struct {
    float *data;
    int len;
    int cap;
} DynArray;

DynArray *dynarray_new(int initial_cap) {
    DynArray *arr = (DynArray *)malloc(sizeof(DynArray));
    if (!arr) return NULL;
    arr->data = (float *)malloc(initial_cap * sizeof(float));
    if (!arr->data) { free(arr); return NULL; }
    arr->len = 0;
    arr->cap = initial_cap;
    return arr;
}

int dynarray_push(DynArray *arr, float value) {
    if (arr->len >= arr->cap) {
        int new_cap = arr->cap * 2;
        float *new_data = (float *)realloc(arr->data, new_cap * sizeof(float));
        if (!new_data) return -1;
        arr->data = new_data;
        arr->cap = new_cap;
    }
    arr->data[arr->len++] = value;
    return 0;
}

float dynarray_get(const DynArray *arr, int idx) {
    if (idx < 0 || idx >= arr->len) return 0.0f;
    return arr->data[idx];
}

void dynarray_free(DynArray *arr) {
    if (arr) {
        free(arr->data);
        free(arr);
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C231: Dynamic array (realloc) should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C231: Output should not be empty");
    assert!(
        code.contains("fn dynarray_new"),
        "C231: Should contain transpiled function dynarray_new"
    );
    assert!(
        code.contains("fn dynarray_push"),
        "C231: Should contain transpiled function dynarray_push"
    );
}

#[test]
fn c232_callback_function_pointer_table() {
    let c_code = r#"
typedef float (*activation_fn)(float);

float relu(float x) {
    return x > 0.0f ? x : 0.0f;
}

float sigmoid(float x) {
    return 1.0f / (1.0f + 1.0f);
}

float tanh_approx(float x) {
    if (x > 3.0f) return 1.0f;
    if (x < -3.0f) return -1.0f;
    return x * (27.0f + x * x) / (27.0f + 9.0f * x * x);
}

float leaky_relu(float x) {
    return x > 0.0f ? x : 0.01f * x;
}

typedef struct {
    activation_fn funcs[4];
    int count;
} ActivationTable;

void init_activations(ActivationTable *table) {
    table->funcs[0] = relu;
    table->funcs[1] = sigmoid;
    table->funcs[2] = tanh_approx;
    table->funcs[3] = leaky_relu;
    table->count = 4;
}

float apply_activation(const ActivationTable *table, int idx, float x) {
    if (idx >= 0 && idx < table->count) {
        return table->funcs[idx](x);
    }
    return x;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C232: Callback function pointer table should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C232: Output should not be empty");
    assert!(
        code.contains("fn relu"),
        "C232: Should contain transpiled function relu"
    );
    assert!(
        code.contains("fn apply_activation"),
        "C232: Should contain transpiled function apply_activation"
    );
}

#[test]
fn c233_bit_manipulation_packed_storage() {
    let c_code = r#"
typedef unsigned int uint32;

uint32 pack_4x8(int a, int b, int c, int d) {
    return ((uint32)(a & 0xFF) << 24) |
           ((uint32)(b & 0xFF) << 16) |
           ((uint32)(c & 0xFF) << 8)  |
           ((uint32)(d & 0xFF));
}

int unpack_byte(uint32 packed, int position) {
    return (int)((packed >> (position * 8)) & 0xFF);
}

uint32 set_bit(uint32 value, int bit) {
    return value | (1u << bit);
}

uint32 clear_bit(uint32 value, int bit) {
    return value & ~(1u << bit);
}

int test_bit(uint32 value, int bit) {
    return (value >> bit) & 1;
}

int popcount(uint32 x) {
    int count = 0;
    while (x) {
        count += x & 1;
        x >>= 1;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C233: Bit manipulation for packed storage should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C233: Output should not be empty");
    assert!(
        code.contains("fn pack_4x8"),
        "C233: Should contain transpiled function pack_4x8"
    );
    assert!(
        code.contains("fn popcount"),
        "C233: Should contain transpiled function popcount"
    );
}

#[test]
fn c234_manual_loop_unrolling() {
    let c_code = r#"
void vector_add_unrolled(const float *a, const float *b, float *c, int n) {
    int i = 0;
    int n4 = n - (n % 4);
    for (; i < n4; i += 4) {
        c[i]     = a[i]     + b[i];
        c[i + 1] = a[i + 1] + b[i + 1];
        c[i + 2] = a[i + 2] + b[i + 2];
        c[i + 3] = a[i + 3] + b[i + 3];
    }
    for (; i < n; i++) {
        c[i] = a[i] + b[i];
    }
}

float sum_unrolled(const float *data, int n) {
    float s0 = 0.0f, s1 = 0.0f, s2 = 0.0f, s3 = 0.0f;
    int i = 0;
    int n4 = n - (n % 4);
    for (; i < n4; i += 4) {
        s0 += data[i];
        s1 += data[i + 1];
        s2 += data[i + 2];
        s3 += data[i + 3];
    }
    float total = s0 + s1 + s2 + s3;
    for (; i < n; i++) {
        total += data[i];
    }
    return total;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C234: Manual loop unrolling should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C234: Output should not be empty");
    assert!(
        code.contains("fn vector_add_unrolled"),
        "C234: Should contain transpiled function vector_add_unrolled"
    );
    assert!(
        code.contains("fn sum_unrolled"),
        "C234: Should contain transpiled function sum_unrolled"
    );
}

#[test]
fn c235_reference_counting() {
    let c_code = r#"
#include <stdlib.h>
#include <string.h>

typedef struct {
    int refcount;
    int size;
    float data[1];
} RefCounted;

RefCounted *rc_create(int size) {
    RefCounted *rc = (RefCounted *)malloc(
        sizeof(RefCounted) + (size - 1) * sizeof(float)
    );
    if (!rc) return NULL;
    rc->refcount = 1;
    rc->size = size;
    memset(rc->data, 0, size * sizeof(float));
    return rc;
}

RefCounted *rc_retain(RefCounted *rc) {
    if (rc) {
        rc->refcount++;
    }
    return rc;
}

void rc_release(RefCounted *rc) {
    if (rc) {
        rc->refcount--;
        if (rc->refcount <= 0) {
            free(rc);
        }
    }
}

int rc_is_unique(const RefCounted *rc) {
    return rc && rc->refcount == 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C235: Reference counting should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C235: Output should not be empty");
    assert!(
        code.contains("fn rc_create"),
        "C235: Should contain transpiled function rc_create"
    );
    assert!(
        code.contains("fn rc_retain"),
        "C235: Should contain transpiled function rc_retain"
    );
    assert!(
        code.contains("fn rc_release"),
        "C235: Should contain transpiled function rc_release"
    );
}

// ============================================================================
// C236-C240: System-Level and Error Handling Patterns
// ============================================================================

#[test]
fn c236_thread_local_static_variables() {
    let c_code = r#"
static int call_count = 0;
static float running_sum = 0.0f;

void accumulate(float value) {
    call_count++;
    running_sum += value;
}

float get_average(void) {
    if (call_count == 0) return 0.0f;
    return running_sum / (float)call_count;
}

void reset_accumulator(void) {
    call_count = 0;
    running_sum = 0.0f;
}

int get_call_count(void) {
    return call_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C236: Thread-local static variables should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C236: Output should not be empty");
    assert!(
        code.contains("fn accumulate"),
        "C236: Should contain transpiled function accumulate"
    );
    assert!(
        code.contains("fn get_average"),
        "C236: Should contain transpiled function get_average"
    );
}

#[test]
fn c237_variadic_function_tensor_creation() {
    let c_code = r#"
#include <stdarg.h>

int sum_ints(int count, ...) {
    va_list args;
    va_start(args, count);
    int total = 0;
    for (int i = 0; i < count; i++) {
        total += va_arg(args, int);
    }
    va_end(args);
    return total;
}

float max_floats(int count, ...) {
    va_list args;
    va_start(args, count);
    float max_val = 0.0f;
    for (int i = 0; i < count; i++) {
        double v = va_arg(args, double);
        if (i == 0 || (float)v > max_val) {
            max_val = (float)v;
        }
    }
    va_end(args);
    return max_val;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C237: Variadic functions should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C237: Output should not be empty");
    assert!(
        code.contains("fn sum_ints"),
        "C237: Should contain transpiled function sum_ints"
    );
}

#[test]
fn c238_opaque_type_void_ptr() {
    let c_code = r#"
#include <stdlib.h>

typedef struct TensorImpl TensorImpl;

typedef struct {
    void *impl_ptr;
    int dtype;
} Tensor;

Tensor tensor_create(int dtype, int size) {
    Tensor t;
    t.impl_ptr = malloc(size * sizeof(float));
    t.dtype = dtype;
    return t;
}

void tensor_destroy(Tensor *t) {
    if (t->impl_ptr) {
        free(t->impl_ptr);
        t->impl_ptr = NULL;
    }
}

int tensor_dtype(const Tensor *t) {
    return t->dtype;
}

void *tensor_data_ptr(const Tensor *t) {
    return t->impl_ptr;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C238: Opaque type with void* should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C238: Output should not be empty");
    assert!(
        code.contains("fn tensor_create"),
        "C238: Should contain transpiled function tensor_create"
    );
    assert!(
        code.contains("fn tensor_destroy"),
        "C238: Should contain transpiled function tensor_destroy"
    );
}

#[test]
fn c239_error_code_goto_cleanup() {
    let c_code = r#"
#include <stdlib.h>

typedef struct {
    float *weights;
    float *bias;
    int input_size;
    int output_size;
} Layer;

int layer_init(Layer *layer, int in_size, int out_size) {
    layer->weights = (float *)malloc(in_size * out_size * sizeof(float));
    if (!layer->weights) goto fail_weights;

    layer->bias = (float *)malloc(out_size * sizeof(float));
    if (!layer->bias) goto fail_bias;

    layer->input_size = in_size;
    layer->output_size = out_size;

    for (int i = 0; i < in_size * out_size; i++) {
        layer->weights[i] = 0.01f;
    }
    for (int i = 0; i < out_size; i++) {
        layer->bias[i] = 0.0f;
    }

    return 0;

fail_bias:
    free(layer->weights);
    layer->weights = NULL;
fail_weights:
    return -1;
}

void layer_free(Layer *layer) {
    free(layer->weights);
    free(layer->bias);
    layer->weights = NULL;
    layer->bias = NULL;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C239: Error code with goto cleanup should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C239: Output should not be empty");
    assert!(
        code.contains("fn layer_init"),
        "C239: Should contain transpiled function layer_init"
    );
    assert!(
        code.contains("fn layer_free"),
        "C239: Should contain transpiled function layer_free"
    );
}

#[test]
fn c240_volatile_memory_mapped_io() {
    let c_code = r#"
void write_register(volatile int *reg, int value) {
    *reg = value;
}

int read_register(volatile int *reg) {
    return *reg;
}

void poll_until_ready(volatile int *status_reg) {
    while ((*status_reg & 0x01) == 0) {
        /* spin */
    }
}

void memory_barrier_store(volatile int *dest, int value) {
    *dest = value;
}

int atomic_cas_sim(volatile int *ptr, int expected, int desired) {
    if (*ptr == expected) {
        *ptr = desired;
        return 1;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C240: Volatile memory-mapped I/O should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C240: Output should not be empty");
    assert!(
        code.contains("fn write_register"),
        "C240: Should contain transpiled function write_register"
    );
    assert!(
        code.contains("fn read_register"),
        "C240: Should contain transpiled function read_register"
    );
}

// ============================================================================
// C241-C245: Advanced Data Structure Patterns
// ============================================================================

#[test]
fn c241_alignment_padding_calculation() {
    let c_code = r#"
int align_up(int value, int alignment) {
    return (value + alignment - 1) & ~(alignment - 1);
}

int calc_padding(int offset, int alignment) {
    int aligned = align_up(offset, alignment);
    return aligned - offset;
}

typedef struct {
    char a;
    int b;
    char c;
    double d;
} PaddedStruct;

int padded_struct_size(void) {
    return (int)sizeof(PaddedStruct);
}

int calc_stride(int width, int element_size, int alignment) {
    int row_bytes = width * element_size;
    return align_up(row_bytes, alignment);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C241: Alignment and padding calculation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C241: Output should not be empty");
    assert!(
        code.contains("fn align_up"),
        "C241: Should contain transpiled function align_up"
    );
    assert!(
        code.contains("fn calc_padding"),
        "C241: Should contain transpiled function calc_padding"
    );
}

#[test]
fn c242_string_interning_table() {
    let c_code = r#"
#include <stdlib.h>
#include <string.h>

#define INTERN_TABLE_SIZE 128
#define MAX_STRING_LEN 64

typedef struct {
    char strings[INTERN_TABLE_SIZE][MAX_STRING_LEN];
    int count;
} InternTable;

void intern_init(InternTable *table) {
    table->count = 0;
}

int intern_find(const InternTable *table, const char *str) {
    for (int i = 0; i < table->count; i++) {
        if (strcmp(table->strings[i], str) == 0) {
            return i;
        }
    }
    return -1;
}

int intern_insert(InternTable *table, const char *str) {
    int existing = intern_find(table, str);
    if (existing >= 0) return existing;
    if (table->count >= INTERN_TABLE_SIZE) return -1;
    strncpy(table->strings[table->count], str, MAX_STRING_LEN - 1);
    table->strings[table->count][MAX_STRING_LEN - 1] = '\0';
    return table->count++;
}

const char *intern_get(const InternTable *table, int id) {
    if (id < 0 || id >= table->count) return NULL;
    return table->strings[id];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C242: String interning table should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C242: Output should not be empty");
    assert!(
        code.contains("fn intern_init"),
        "C242: Should contain transpiled function intern_init"
    );
    assert!(
        code.contains("fn intern_find"),
        "C242: Should contain transpiled function intern_find"
    );
    assert!(
        code.contains("fn intern_insert"),
        "C242: Should contain transpiled function intern_insert"
    );
}

#[test]
fn c243_graph_adjacency_list() {
    let c_code = r#"
#include <stdlib.h>

#define MAX_NODES 128
#define MAX_EDGES_PER_NODE 16

typedef struct {
    int neighbors[MAX_NODES][MAX_EDGES_PER_NODE];
    int edge_count[MAX_NODES];
    int num_nodes;
} Graph;

void graph_init(Graph *g, int num_nodes) {
    g->num_nodes = num_nodes;
    for (int i = 0; i < num_nodes; i++) {
        g->edge_count[i] = 0;
    }
}

int graph_add_edge(Graph *g, int from, int to) {
    if (from < 0 || from >= g->num_nodes) return -1;
    if (to < 0 || to >= g->num_nodes) return -1;
    if (g->edge_count[from] >= MAX_EDGES_PER_NODE) return -1;
    g->neighbors[from][g->edge_count[from]] = to;
    g->edge_count[from]++;
    return 0;
}

int graph_has_edge(const Graph *g, int from, int to) {
    if (from < 0 || from >= g->num_nodes) return 0;
    for (int i = 0; i < g->edge_count[from]; i++) {
        if (g->neighbors[from][i] == to) return 1;
    }
    return 0;
}

int graph_degree(const Graph *g, int node) {
    if (node < 0 || node >= g->num_nodes) return 0;
    return g->edge_count[node];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C243: Graph adjacency list should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C243: Output should not be empty");
    assert!(
        code.contains("fn graph_init"),
        "C243: Should contain transpiled function graph_init"
    );
    assert!(
        code.contains("fn graph_add_edge"),
        "C243: Should contain transpiled function graph_add_edge"
    );
    assert!(
        code.contains("fn graph_has_edge"),
        "C243: Should contain transpiled function graph_has_edge"
    );
}

#[test]
fn c244_priority_queue_heap() {
    let c_code = r#"
#define HEAP_MAX 512

typedef struct {
    float priorities[HEAP_MAX];
    int values[HEAP_MAX];
    int size;
} MinHeap;

void heap_init(MinHeap *h) {
    h->size = 0;
}

static void heap_swap(MinHeap *h, int i, int j) {
    float tp = h->priorities[i];
    h->priorities[i] = h->priorities[j];
    h->priorities[j] = tp;
    int tv = h->values[i];
    h->values[i] = h->values[j];
    h->values[j] = tv;
}

int heap_push(MinHeap *h, float priority, int value) {
    if (h->size >= HEAP_MAX) return -1;
    int i = h->size;
    h->priorities[i] = priority;
    h->values[i] = value;
    h->size++;
    while (i > 0) {
        int parent = (i - 1) / 2;
        if (h->priorities[i] < h->priorities[parent]) {
            heap_swap(h, i, parent);
            i = parent;
        } else {
            break;
        }
    }
    return 0;
}

int heap_pop(MinHeap *h, float *out_priority, int *out_value) {
    if (h->size <= 0) return -1;
    *out_priority = h->priorities[0];
    *out_value = h->values[0];
    h->size--;
    if (h->size > 0) {
        h->priorities[0] = h->priorities[h->size];
        h->values[0] = h->values[h->size];
        int i = 0;
        while (1) {
            int left = 2 * i + 1;
            int right = 2 * i + 2;
            int smallest = i;
            if (left < h->size && h->priorities[left] < h->priorities[smallest])
                smallest = left;
            if (right < h->size && h->priorities[right] < h->priorities[smallest])
                smallest = right;
            if (smallest != i) {
                heap_swap(h, i, smallest);
                i = smallest;
            } else {
                break;
            }
        }
    }
    return 0;
}

int heap_size(const MinHeap *h) {
    return h->size;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C244: Priority queue with heap should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C244: Output should not be empty");
    assert!(
        code.contains("fn heap_init"),
        "C244: Should contain transpiled function heap_init"
    );
    assert!(
        code.contains("fn heap_push"),
        "C244: Should contain transpiled function heap_push"
    );
    assert!(
        code.contains("fn heap_pop"),
        "C244: Should contain transpiled function heap_pop"
    );
}

#[test]
fn c245_btree_node_variable_keys() {
    let c_code = r#"
#define BTREE_ORDER 4
#define MAX_KEYS (BTREE_ORDER - 1)
#define MAX_CHILDREN BTREE_ORDER

typedef struct BTreeNode {
    int keys[MAX_KEYS];
    float values[MAX_KEYS];
    struct BTreeNode *children[MAX_CHILDREN];
    int num_keys;
    int is_leaf;
} BTreeNode;

BTreeNode *btree_create_node(int is_leaf) {
    BTreeNode *node = (BTreeNode *)malloc(sizeof(BTreeNode));
    if (!node) return NULL;
    node->num_keys = 0;
    node->is_leaf = is_leaf;
    for (int i = 0; i < MAX_CHILDREN; i++) {
        node->children[i] = NULL;
    }
    return node;
}

int btree_search(const BTreeNode *node, int key, float *out_value) {
    if (!node) return 0;
    int i = 0;
    while (i < node->num_keys && key > node->keys[i]) {
        i++;
    }
    if (i < node->num_keys && key == node->keys[i]) {
        *out_value = node->values[i];
        return 1;
    }
    if (node->is_leaf) return 0;
    return btree_search(node->children[i], key, out_value);
}

int btree_insert_nonfull(BTreeNode *node, int key, float value) {
    int i = node->num_keys - 1;
    if (node->is_leaf) {
        while (i >= 0 && node->keys[i] > key) {
            node->keys[i + 1] = node->keys[i];
            node->values[i + 1] = node->values[i];
            i--;
        }
        node->keys[i + 1] = key;
        node->values[i + 1] = value;
        node->num_keys++;
        return 0;
    }
    while (i >= 0 && node->keys[i] > key) {
        i--;
    }
    i++;
    return btree_insert_nonfull(node->children[i], key, value);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C245: B-tree node should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C245: Output should not be empty");
    assert!(
        code.contains("fn btree_create_node") || code.contains("fn btree_search"),
        "C245: Should contain transpiled btree functions"
    );
}

// ============================================================================
// C246-C250: ML-Specific Numeric and Graph Patterns
// ============================================================================

#[test]
fn c246_sparse_matrix_csr() {
    let c_code = r#"
typedef struct {
    float *values;
    int *col_indices;
    int *row_ptr;
    int num_rows;
    int num_cols;
    int nnz;
} CSRMatrix;

float csr_get(const CSRMatrix *mat, int row, int col) {
    if (row < 0 || row >= mat->num_rows) return 0.0f;
    int start = mat->row_ptr[row];
    int end = mat->row_ptr[row + 1];
    for (int i = start; i < end; i++) {
        if (mat->col_indices[i] == col) {
            return mat->values[i];
        }
    }
    return 0.0f;
}

void csr_matvec(const CSRMatrix *mat, const float *x, float *y) {
    for (int row = 0; row < mat->num_rows; row++) {
        float sum = 0.0f;
        int start = mat->row_ptr[row];
        int end = mat->row_ptr[row + 1];
        for (int i = start; i < end; i++) {
            sum += mat->values[i] * x[mat->col_indices[i]];
        }
        y[row] = sum;
    }
}

int csr_nnz_in_row(const CSRMatrix *mat, int row) {
    if (row < 0 || row >= mat->num_rows) return 0;
    return mat->row_ptr[row + 1] - mat->row_ptr[row];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C246: Sparse matrix CSR format should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C246: Output should not be empty");
    assert!(
        code.contains("fn csr_get"),
        "C246: Should contain transpiled function csr_get"
    );
    assert!(
        code.contains("fn csr_matvec"),
        "C246: Should contain transpiled function csr_matvec"
    );
}

#[test]
fn c247_double_ended_queue() {
    let c_code = r#"
#define DEQUE_CAP 256

typedef struct {
    int data[DEQUE_CAP];
    int front;
    int back;
    int size;
} Deque;

void deque_init(Deque *d) {
    d->front = 0;
    d->back = 0;
    d->size = 0;
}

int deque_push_back(Deque *d, int value) {
    if (d->size >= DEQUE_CAP) return -1;
    d->data[d->back] = value;
    d->back = (d->back + 1) % DEQUE_CAP;
    d->size++;
    return 0;
}

int deque_push_front(Deque *d, int value) {
    if (d->size >= DEQUE_CAP) return -1;
    d->front = (d->front - 1 + DEQUE_CAP) % DEQUE_CAP;
    d->data[d->front] = value;
    d->size++;
    return 0;
}

int deque_pop_back(Deque *d, int *out) {
    if (d->size <= 0) return -1;
    d->back = (d->back - 1 + DEQUE_CAP) % DEQUE_CAP;
    *out = d->data[d->back];
    d->size--;
    return 0;
}

int deque_pop_front(Deque *d, int *out) {
    if (d->size <= 0) return -1;
    *out = d->data[d->front];
    d->front = (d->front + 1) % DEQUE_CAP;
    d->size--;
    return 0;
}

int deque_is_empty(const Deque *d) {
    return d->size == 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C247: Double-ended queue should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C247: Output should not be empty");
    assert!(
        code.contains("fn deque_init"),
        "C247: Should contain transpiled function deque_init"
    );
    assert!(
        code.contains("fn deque_push_back"),
        "C247: Should contain transpiled function deque_push_back"
    );
    assert!(
        code.contains("fn deque_pop_front"),
        "C247: Should contain transpiled function deque_pop_front"
    );
}

#[test]
fn c248_lru_cache_doubly_linked_list() {
    let c_code = r#"
#include <stdlib.h>

#define LRU_CAPACITY 32

typedef struct LRUNode {
    int key;
    float value;
    struct LRUNode *prev;
    struct LRUNode *next;
} LRUNode;

typedef struct {
    LRUNode *head;
    LRUNode *tail;
    LRUNode *map[LRU_CAPACITY];
    int size;
} LRUCache;

void lru_init(LRUCache *cache) {
    cache->head = NULL;
    cache->tail = NULL;
    cache->size = 0;
    for (int i = 0; i < LRU_CAPACITY; i++) {
        cache->map[i] = NULL;
    }
}

static void lru_remove_node(LRUCache *cache, LRUNode *node) {
    if (node->prev) node->prev->next = node->next;
    else cache->head = node->next;
    if (node->next) node->next->prev = node->prev;
    else cache->tail = node->prev;
}

static void lru_push_front(LRUCache *cache, LRUNode *node) {
    node->prev = NULL;
    node->next = cache->head;
    if (cache->head) cache->head->prev = node;
    cache->head = node;
    if (!cache->tail) cache->tail = node;
}

int lru_get(LRUCache *cache, int key, float *out_value) {
    int idx = key % LRU_CAPACITY;
    LRUNode *node = cache->map[idx];
    if (!node || node->key != key) return 0;
    lru_remove_node(cache, node);
    lru_push_front(cache, node);
    *out_value = node->value;
    return 1;
}

void lru_put(LRUCache *cache, int key, float value) {
    int idx = key % LRU_CAPACITY;
    LRUNode *existing = cache->map[idx];
    if (existing && existing->key == key) {
        existing->value = value;
        lru_remove_node(cache, existing);
        lru_push_front(cache, existing);
        return;
    }
    LRUNode *node = (LRUNode *)malloc(sizeof(LRUNode));
    if (!node) return;
    node->key = key;
    node->value = value;
    cache->map[idx] = node;
    lru_push_front(cache, node);
    cache->size++;
    if (cache->size > LRU_CAPACITY && cache->tail) {
        LRUNode *evict = cache->tail;
        lru_remove_node(cache, evict);
        cache->map[evict->key % LRU_CAPACITY] = NULL;
        free(evict);
        cache->size--;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C248: LRU cache with doubly-linked list should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C248: Output should not be empty");
    assert!(
        code.contains("fn lru_init"),
        "C248: Should contain transpiled function lru_init"
    );
    assert!(
        code.contains("fn lru_get"),
        "C248: Should contain transpiled function lru_get"
    );
    assert!(
        code.contains("fn lru_put"),
        "C248: Should contain transpiled function lru_put"
    );
}

#[test]
fn c249_red_black_tree_node_coloring() {
    let c_code = r#"
#include <stdlib.h>

#define RB_RED 0
#define RB_BLACK 1

typedef struct RBNode {
    int key;
    float value;
    int color;
    struct RBNode *left;
    struct RBNode *right;
    struct RBNode *parent;
} RBNode;

RBNode *rb_create_node(int key, float value) {
    RBNode *node = (RBNode *)malloc(sizeof(RBNode));
    if (!node) return NULL;
    node->key = key;
    node->value = value;
    node->color = RB_RED;
    node->left = NULL;
    node->right = NULL;
    node->parent = NULL;
    return node;
}

int rb_is_red(const RBNode *node) {
    if (!node) return 0;
    return node->color == RB_RED;
}

RBNode *rb_rotate_left(RBNode *node) {
    RBNode *right = node->right;
    node->right = right->left;
    if (right->left) right->left->parent = node;
    right->parent = node->parent;
    right->left = node;
    node->parent = right;
    right->color = node->color;
    node->color = RB_RED;
    return right;
}

RBNode *rb_rotate_right(RBNode *node) {
    RBNode *left = node->left;
    node->left = left->right;
    if (left->right) left->right->parent = node;
    left->parent = node->parent;
    left->right = node;
    node->parent = left;
    left->color = node->color;
    node->color = RB_RED;
    return left;
}

void rb_flip_colors(RBNode *node) {
    node->color = RB_RED;
    if (node->left) node->left->color = RB_BLACK;
    if (node->right) node->right->color = RB_BLACK;
}

RBNode *rb_find(RBNode *root, int key) {
    RBNode *curr = root;
    while (curr) {
        if (key < curr->key) curr = curr->left;
        else if (key > curr->key) curr = curr->right;
        else return curr;
    }
    return NULL;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C249: Red-black tree node coloring should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C249: Output should not be empty");
    assert!(
        code.contains("fn rb_create_node"),
        "C249: Should contain transpiled function rb_create_node"
    );
    assert!(
        code.contains("fn rb_is_red"),
        "C249: Should contain transpiled function rb_is_red"
    );
    assert!(
        code.contains("fn rb_find"),
        "C249: Should contain transpiled function rb_find"
    );
}

#[test]
fn c250_neural_network_layer() {
    let c_code = r#"
typedef struct {
    float *weights;
    float *bias;
    float *output;
    int input_size;
    int output_size;
} DenseLayer;

float relu_activation(float x) {
    return x > 0.0f ? x : 0.0f;
}

void dense_forward(const DenseLayer *layer, const float *input) {
    for (int o = 0; o < layer->output_size; o++) {
        float sum = layer->bias[o];
        for (int i = 0; i < layer->input_size; i++) {
            sum += input[i] * layer->weights[o * layer->input_size + i];
        }
        layer->output[o] = relu_activation(sum);
    }
}

void dense_backward_weights(const float *input, const float *grad_output,
                            float *grad_weights, int input_size, int output_size) {
    for (int o = 0; o < output_size; o++) {
        for (int i = 0; i < input_size; i++) {
            grad_weights[o * input_size + i] += grad_output[o] * input[i];
        }
    }
}

void sgd_update(float *params, const float *grads, int n, float learning_rate) {
    for (int i = 0; i < n; i++) {
        params[i] -= learning_rate * grads[i];
    }
}

float mse_loss(const float *predicted, const float *target, int n) {
    float loss = 0.0f;
    for (int i = 0; i < n; i++) {
        float diff = predicted[i] - target[i];
        loss += diff * diff;
    }
    return loss / (float)n;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C250: Neural network layer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C250: Output should not be empty");
    assert!(
        code.contains("fn relu_activation") || code.contains("fn dense_forward"),
        "C250: Should contain transpiled neural network functions"
    );
    assert!(
        code.contains("fn sgd_update") || code.contains("fn mse_loss"),
        "C250: Should contain transpiled training functions"
    );
}
