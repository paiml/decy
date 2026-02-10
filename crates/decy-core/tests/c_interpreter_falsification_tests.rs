//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1251-C1275: Interpreters & Virtual Machines domain -- bytecode VMs,
//! interpreters, JIT helpers, and language runtimes in C.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise interpreter and VM patterns commonly found in
//! CPython, Lua, Ruby MRI, Wren, and similar projects -- all expressed
//! as valid C99 without #include.
//!
//! Organization:
//! - C1251-C1255: Stack machines (stack VM, operand stack, call frame, local variables, return handling)
//! - C1256-C1260: Bytecode (bytecode encoder, instruction decoder, opcode dispatch, constant pool, line number table)
//! - C1261-C1265: Memory (mark-sweep GC, ref counting, arena allocator, string interning, weak references)
//! - C1266-C1270: Types (tagged union, type checker, method dispatch, vtable lookup, generic instantiation)
//! - C1271-C1275: Optimization (peephole optimizer, constant folding, dead code elimination, register allocator, basic block)

// ============================================================================
// C1251-C1255: Stack Machines
// ============================================================================

/// C1251: Stack-based virtual machine with push/pop/arithmetic
#[test]
fn c1251_stack_vm_push_pop_arithmetic() {
    let c_code = r#"
typedef int int32_t;

struct vm_stack_machine {
    int32_t data[256];
    int sp;
    int halted;
};

void vm_stack_init(struct vm_stack_machine *sm) {
    sm->sp = 0;
    sm->halted = 0;
}

int vm_stack_push(struct vm_stack_machine *sm, int32_t val) {
    if (sm->sp >= 256) return -1;
    sm->data[sm->sp] = val;
    sm->sp++;
    return 0;
}

int32_t vm_stack_pop(struct vm_stack_machine *sm) {
    if (sm->sp <= 0) {
        sm->halted = 1;
        return 0;
    }
    sm->sp--;
    return sm->data[sm->sp];
}

int vm_stack_add(struct vm_stack_machine *sm) {
    int32_t b = vm_stack_pop(sm);
    int32_t a = vm_stack_pop(sm);
    if (sm->halted) return -1;
    return vm_stack_push(sm, a + b);
}

int vm_stack_sub(struct vm_stack_machine *sm) {
    int32_t b = vm_stack_pop(sm);
    int32_t a = vm_stack_pop(sm);
    if (sm->halted) return -1;
    return vm_stack_push(sm, a - b);
}

int vm_stack_mul(struct vm_stack_machine *sm) {
    int32_t b = vm_stack_pop(sm);
    int32_t a = vm_stack_pop(sm);
    if (sm->halted) return -1;
    return vm_stack_push(sm, a * b);
}

int32_t vm_stack_peek(struct vm_stack_machine *sm) {
    if (sm->sp <= 0) return 0;
    return sm->data[sm->sp - 1];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1251: Stack VM push/pop/arithmetic - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1251: empty output");
    assert!(
        code.contains("fn vm_stack_push"),
        "C1251: Should contain vm_stack_push function"
    );
}

/// C1252: Operand stack with type-tagged values
#[test]
fn c1252_operand_stack_typed_values() {
    let c_code = r#"
enum vm_val_type {
    VM_VAL_INT = 0,
    VM_VAL_FLOAT,
    VM_VAL_BOOL,
    VM_VAL_NIL
};

struct vm_operand {
    enum vm_val_type type;
    int int_val;
    double float_val;
};

struct vm_operand_stack {
    struct vm_operand items[128];
    int top;
};

void vm_opstack_init(struct vm_operand_stack *os) {
    os->top = 0;
}

int vm_opstack_push_int(struct vm_operand_stack *os, int val) {
    if (os->top >= 128) return -1;
    os->items[os->top].type = VM_VAL_INT;
    os->items[os->top].int_val = val;
    os->top++;
    return 0;
}

int vm_opstack_push_float(struct vm_operand_stack *os, double val) {
    if (os->top >= 128) return -1;
    os->items[os->top].type = VM_VAL_FLOAT;
    os->items[os->top].float_val = val;
    os->top++;
    return 0;
}

int vm_opstack_push_nil(struct vm_operand_stack *os) {
    if (os->top >= 128) return -1;
    os->items[os->top].type = VM_VAL_NIL;
    os->top++;
    return 0;
}

struct vm_operand vm_opstack_pop(struct vm_operand_stack *os) {
    struct vm_operand nil;
    nil.type = VM_VAL_NIL;
    nil.int_val = 0;
    nil.float_val = 0.0;
    if (os->top <= 0) return nil;
    os->top--;
    return os->items[os->top];
}

int vm_opstack_is_int(struct vm_operand *v) {
    return v->type == VM_VAL_INT;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1252: Operand stack typed values - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1252: empty output");
    assert!(
        code.contains("fn vm_opstack_push_int"),
        "C1252: Should contain vm_opstack_push_int function"
    );
}

/// C1253: Call frame management for function invocations
#[test]
fn c1253_call_frame_management() {
    let c_code = r#"
struct vm_call_frame {
    int return_addr;
    int base_pointer;
    int func_id;
    int arg_count;
};

struct vm_call_stack {
    struct vm_call_frame frames[64];
    int depth;
};

void vm_callstack_init(struct vm_call_stack *cs) {
    cs->depth = 0;
}

int vm_callstack_push(struct vm_call_stack *cs, int ret_addr, int bp, int func, int argc) {
    if (cs->depth >= 64) return -1;
    cs->frames[cs->depth].return_addr = ret_addr;
    cs->frames[cs->depth].base_pointer = bp;
    cs->frames[cs->depth].func_id = func;
    cs->frames[cs->depth].arg_count = argc;
    cs->depth++;
    return 0;
}

int vm_callstack_pop(struct vm_call_stack *cs, int *ret_addr, int *bp) {
    if (cs->depth <= 0) return -1;
    cs->depth--;
    *ret_addr = cs->frames[cs->depth].return_addr;
    *bp = cs->frames[cs->depth].base_pointer;
    return 0;
}

int vm_callstack_current_func(struct vm_call_stack *cs) {
    if (cs->depth <= 0) return -1;
    return cs->frames[cs->depth - 1].func_id;
}

int vm_callstack_depth(struct vm_call_stack *cs) {
    return cs->depth;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1253: Call frame management - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1253: empty output");
    assert!(
        code.contains("fn vm_callstack_push"),
        "C1253: Should contain vm_callstack_push function"
    );
}

/// C1254: Local variable slots within frames
#[test]
fn c1254_local_variable_slots() {
    let c_code = r#"
struct vm_local_slot {
    int value;
    int defined;
};

struct vm_locals {
    struct vm_local_slot slots[32];
    int count;
    int frame_base;
};

void vm_locals_init(struct vm_locals *loc, int base) {
    int i;
    loc->count = 0;
    loc->frame_base = base;
    for (i = 0; i < 32; i++) {
        loc->slots[i].value = 0;
        loc->slots[i].defined = 0;
    }
}

int vm_locals_set(struct vm_locals *loc, int idx, int val) {
    if (idx < 0 || idx >= 32) return -1;
    loc->slots[idx].value = val;
    loc->slots[idx].defined = 1;
    if (idx >= loc->count) loc->count = idx + 1;
    return 0;
}

int vm_locals_get(struct vm_locals *loc, int idx, int *out) {
    if (idx < 0 || idx >= 32) return -1;
    if (!loc->slots[idx].defined) return -2;
    *out = loc->slots[idx].value;
    return 0;
}

int vm_locals_is_defined(struct vm_locals *loc, int idx) {
    if (idx < 0 || idx >= 32) return 0;
    return loc->slots[idx].defined;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1254: Local variable slots - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1254: empty output");
    assert!(
        code.contains("fn vm_locals_set"),
        "C1254: Should contain vm_locals_set function"
    );
}

/// C1255: Return value handling across frames
#[test]
fn c1255_return_value_handling() {
    let c_code = r#"
struct vm_return_info {
    int has_value;
    int value;
    int from_func;
};

struct vm_return_stack {
    struct vm_return_info entries[32];
    int count;
};

void vm_retstack_init(struct vm_return_stack *rs) {
    rs->count = 0;
}

int vm_retstack_push(struct vm_return_stack *rs, int val, int func_id) {
    if (rs->count >= 32) return -1;
    rs->entries[rs->count].has_value = 1;
    rs->entries[rs->count].value = val;
    rs->entries[rs->count].from_func = func_id;
    rs->count++;
    return 0;
}

int vm_retstack_push_void(struct vm_return_stack *rs, int func_id) {
    if (rs->count >= 32) return -1;
    rs->entries[rs->count].has_value = 0;
    rs->entries[rs->count].value = 0;
    rs->entries[rs->count].from_func = func_id;
    rs->count++;
    return 0;
}

int vm_retstack_pop(struct vm_return_stack *rs, int *val, int *had_value) {
    if (rs->count <= 0) return -1;
    rs->count--;
    *val = rs->entries[rs->count].value;
    *had_value = rs->entries[rs->count].has_value;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1255: Return value handling - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1255: empty output");
    assert!(
        code.contains("fn vm_retstack_push"),
        "C1255: Should contain vm_retstack_push function"
    );
}

// ============================================================================
// C1256-C1260: Bytecode
// ============================================================================

/// C1256: Bytecode encoder that emits instructions to a buffer
#[test]
fn c1256_bytecode_encoder() {
    let c_code = r#"
typedef unsigned char uint8_t;

struct vm_bytecode_buf {
    uint8_t code[1024];
    int len;
    int capacity;
};

void vm_bytecode_init(struct vm_bytecode_buf *buf) {
    buf->len = 0;
    buf->capacity = 1024;
}

int vm_bytecode_emit_byte(struct vm_bytecode_buf *buf, uint8_t byte) {
    if (buf->len >= buf->capacity) return -1;
    buf->code[buf->len] = byte;
    buf->len++;
    return 0;
}

int vm_bytecode_emit_u16(struct vm_bytecode_buf *buf, int val) {
    if (buf->len + 2 > buf->capacity) return -1;
    buf->code[buf->len] = (uint8_t)(val >> 8);
    buf->code[buf->len + 1] = (uint8_t)(val & 0xFF);
    buf->len += 2;
    return 0;
}

int vm_bytecode_emit_op(struct vm_bytecode_buf *buf, uint8_t op, int arg) {
    int rc = vm_bytecode_emit_byte(buf, op);
    if (rc != 0) return rc;
    return vm_bytecode_emit_u16(buf, arg);
}

int vm_bytecode_size(struct vm_bytecode_buf *buf) {
    return buf->len;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1256: Bytecode encoder - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1256: empty output");
    assert!(
        code.contains("fn vm_bytecode_emit_byte"),
        "C1256: Should contain vm_bytecode_emit_byte function"
    );
}

/// C1257: Instruction decoder that reads opcode and operands
#[test]
fn c1257_instruction_decoder() {
    let c_code = r#"
typedef unsigned char uint8_t;

struct vm_instr {
    uint8_t opcode;
    int operand;
    int width;
};

struct vm_decoder {
    uint8_t *bytecode;
    int length;
    int offset;
};

void vm_decoder_init(struct vm_decoder *dec, uint8_t *code, int len) {
    dec->bytecode = code;
    dec->length = len;
    dec->offset = 0;
}

int vm_decoder_has_more(struct vm_decoder *dec) {
    return dec->offset < dec->length;
}

int vm_decode_next(struct vm_decoder *dec, struct vm_instr *out) {
    uint8_t op;
    if (dec->offset >= dec->length) return -1;
    op = dec->bytecode[dec->offset];
    out->opcode = op;
    dec->offset++;
    if (op >= 0x10 && op <= 0x3F) {
        if (dec->offset + 1 >= dec->length) return -2;
        out->operand = (dec->bytecode[dec->offset] << 8) | dec->bytecode[dec->offset + 1];
        dec->offset += 2;
        out->width = 3;
    } else {
        out->operand = 0;
        out->width = 1;
    }
    return 0;
}

void vm_decoder_reset(struct vm_decoder *dec) {
    dec->offset = 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1257: Instruction decoder - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1257: empty output");
    assert!(
        code.contains("fn vm_decode_next"),
        "C1257: Should contain vm_decode_next function"
    );
}

/// C1258: Opcode dispatch loop with switch-based execution
#[test]
fn c1258_opcode_dispatch_loop() {
    let c_code = r#"
typedef unsigned char uint8_t;

enum vm_op {
    VM_OP_NOP = 0,
    VM_OP_LOAD_CONST,
    VM_OP_ADD,
    VM_OP_SUB,
    VM_OP_PRINT,
    VM_OP_JUMP,
    VM_OP_JUMP_IF_ZERO,
    VM_OP_HALT
};

struct vm_dispatch_ctx {
    int stack[64];
    int sp;
    uint8_t *code;
    int pc;
    int running;
};

void vm_dispatch_init(struct vm_dispatch_ctx *ctx, uint8_t *code) {
    ctx->sp = 0;
    ctx->pc = 0;
    ctx->running = 1;
    ctx->code = code;
}

int vm_dispatch_step(struct vm_dispatch_ctx *ctx) {
    uint8_t op = ctx->code[ctx->pc];
    ctx->pc++;
    switch (op) {
        case VM_OP_NOP:
            break;
        case VM_OP_LOAD_CONST:
            if (ctx->sp < 64) {
                ctx->stack[ctx->sp] = ctx->code[ctx->pc];
                ctx->sp++;
                ctx->pc++;
            }
            break;
        case VM_OP_ADD:
            if (ctx->sp >= 2) {
                ctx->sp--;
                ctx->stack[ctx->sp - 1] += ctx->stack[ctx->sp];
            }
            break;
        case VM_OP_SUB:
            if (ctx->sp >= 2) {
                ctx->sp--;
                ctx->stack[ctx->sp - 1] -= ctx->stack[ctx->sp];
            }
            break;
        case VM_OP_HALT:
            ctx->running = 0;
            break;
        default:
            ctx->running = 0;
            return -1;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1258: Opcode dispatch loop - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1258: empty output");
    assert!(
        code.contains("fn vm_dispatch_step"),
        "C1258: Should contain vm_dispatch_step function"
    );
}

/// C1259: Constant pool for storing literal values
#[test]
fn c1259_constant_pool() {
    let c_code = r#"
struct vm_const_entry {
    int tag;
    int int_val;
    double float_val;
};

struct vm_const_pool {
    struct vm_const_entry entries[256];
    int count;
};

void vm_const_pool_init(struct vm_const_pool *pool) {
    pool->count = 0;
}

int vm_const_add_int(struct vm_const_pool *pool, int val) {
    int idx;
    if (pool->count >= 256) return -1;
    idx = pool->count;
    pool->entries[idx].tag = 1;
    pool->entries[idx].int_val = val;
    pool->entries[idx].float_val = 0.0;
    pool->count++;
    return idx;
}

int vm_const_add_float(struct vm_const_pool *pool, double val) {
    int idx;
    if (pool->count >= 256) return -1;
    idx = pool->count;
    pool->entries[idx].tag = 2;
    pool->entries[idx].int_val = 0;
    pool->entries[idx].float_val = val;
    pool->count++;
    return idx;
}

int vm_const_get_int(struct vm_const_pool *pool, int idx) {
    if (idx < 0 || idx >= pool->count) return 0;
    return pool->entries[idx].int_val;
}

double vm_const_get_float(struct vm_const_pool *pool, int idx) {
    if (idx < 0 || idx >= pool->count) return 0.0;
    return pool->entries[idx].float_val;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1259: Constant pool - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1259: empty output");
    assert!(
        code.contains("fn vm_const_add_int"),
        "C1259: Should contain vm_const_add_int function"
    );
}

/// C1260: Line number table mapping bytecode offsets to source lines
#[test]
fn c1260_line_number_table() {
    let c_code = r#"
struct vm_line_entry {
    int offset;
    int line;
};

struct vm_line_table {
    struct vm_line_entry entries[512];
    int count;
};

void vm_linetable_init(struct vm_line_table *lt) {
    lt->count = 0;
}

int vm_linetable_add(struct vm_line_table *lt, int offset, int line) {
    if (lt->count >= 512) return -1;
    lt->entries[lt->count].offset = offset;
    lt->entries[lt->count].line = line;
    lt->count++;
    return 0;
}

int vm_linetable_lookup(struct vm_line_table *lt, int offset) {
    int i;
    int best_line = 0;
    for (i = 0; i < lt->count; i++) {
        if (lt->entries[i].offset <= offset) {
            best_line = lt->entries[i].line;
        } else {
            break;
        }
    }
    return best_line;
}

int vm_linetable_count(struct vm_line_table *lt) {
    return lt->count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1260: Line number table - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1260: empty output");
    assert!(
        code.contains("fn vm_linetable_lookup"),
        "C1260: Should contain vm_linetable_lookup function"
    );
}

// ============================================================================
// C1261-C1265: Memory
// ============================================================================

/// C1261: Mark-sweep garbage collector with object graph traversal
#[test]
fn c1261_mark_sweep_gc() {
    let c_code = r#"
struct vm_gc_obj {
    int marked;
    int type_tag;
    int refs[8];
    int ref_count;
    int alive;
};

struct vm_gc_heap {
    struct vm_gc_obj objects[128];
    int count;
    int threshold;
};

void vm_gc_init(struct vm_gc_heap *heap) {
    heap->count = 0;
    heap->threshold = 64;
}

int vm_gc_alloc(struct vm_gc_heap *heap, int type_tag) {
    int idx;
    if (heap->count >= 128) return -1;
    idx = heap->count;
    heap->objects[idx].marked = 0;
    heap->objects[idx].type_tag = type_tag;
    heap->objects[idx].ref_count = 0;
    heap->objects[idx].alive = 1;
    heap->count++;
    return idx;
}

void vm_gc_add_ref(struct vm_gc_heap *heap, int obj, int ref) {
    struct vm_gc_obj *o;
    if (obj < 0 || obj >= heap->count) return;
    o = &heap->objects[obj];
    if (o->ref_count < 8) {
        o->refs[o->ref_count] = ref;
        o->ref_count++;
    }
}

static void vm_gc_mark(struct vm_gc_heap *heap, int idx) {
    int i;
    struct vm_gc_obj *o;
    if (idx < 0 || idx >= heap->count) return;
    o = &heap->objects[idx];
    if (o->marked || !o->alive) return;
    o->marked = 1;
    for (i = 0; i < o->ref_count; i++) {
        vm_gc_mark(heap, o->refs[i]);
    }
}

int vm_gc_sweep(struct vm_gc_heap *heap) {
    int i, freed = 0;
    for (i = 0; i < heap->count; i++) {
        if (!heap->objects[i].marked && heap->objects[i].alive) {
            heap->objects[i].alive = 0;
            freed++;
        }
        heap->objects[i].marked = 0;
    }
    return freed;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1261: Mark-sweep GC - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1261: empty output");
    assert!(
        code.contains("fn vm_gc_sweep"),
        "C1261: Should contain vm_gc_sweep function"
    );
}

/// C1262: Reference counting with increment/decrement and free
#[test]
fn c1262_ref_counting() {
    let c_code = r#"
struct vm_rc_obj {
    int refcount;
    int value;
    int alive;
    int type_id;
};

struct vm_rc_pool {
    struct vm_rc_obj objects[128];
    int count;
    int freed;
};

void vm_rc_pool_init(struct vm_rc_pool *pool) {
    pool->count = 0;
    pool->freed = 0;
}

int vm_rc_alloc(struct vm_rc_pool *pool, int value, int type_id) {
    int idx;
    if (pool->count >= 128) return -1;
    idx = pool->count;
    pool->objects[idx].refcount = 1;
    pool->objects[idx].value = value;
    pool->objects[idx].alive = 1;
    pool->objects[idx].type_id = type_id;
    pool->count++;
    return idx;
}

void vm_rc_incref(struct vm_rc_pool *pool, int idx) {
    if (idx < 0 || idx >= pool->count) return;
    if (pool->objects[idx].alive) {
        pool->objects[idx].refcount++;
    }
}

void vm_rc_decref(struct vm_rc_pool *pool, int idx) {
    if (idx < 0 || idx >= pool->count) return;
    if (!pool->objects[idx].alive) return;
    pool->objects[idx].refcount--;
    if (pool->objects[idx].refcount <= 0) {
        pool->objects[idx].alive = 0;
        pool->freed++;
    }
}

int vm_rc_get_refcount(struct vm_rc_pool *pool, int idx) {
    if (idx < 0 || idx >= pool->count) return -1;
    return pool->objects[idx].refcount;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1262: Reference counting - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1262: empty output");
    assert!(
        code.contains("fn vm_rc_decref"),
        "C1262: Should contain vm_rc_decref function"
    );
}

/// C1263: Arena allocator for fast bump-pointer allocation
#[test]
fn c1263_arena_allocator() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned char uint8_t;

struct vm_arena {
    uint8_t buffer[4096];
    int offset;
    int capacity;
    int alloc_count;
};

void vm_arena_init(struct vm_arena *arena) {
    arena->offset = 0;
    arena->capacity = 4096;
    arena->alloc_count = 0;
}

int vm_arena_alloc(struct vm_arena *arena, int size) {
    int start;
    int aligned = (size + 7) & ~7;
    if (arena->offset + aligned > arena->capacity) return -1;
    start = arena->offset;
    arena->offset += aligned;
    arena->alloc_count++;
    return start;
}

void vm_arena_reset(struct vm_arena *arena) {
    arena->offset = 0;
    arena->alloc_count = 0;
}

int vm_arena_used(struct vm_arena *arena) {
    return arena->offset;
}

int vm_arena_remaining(struct vm_arena *arena) {
    return arena->capacity - arena->offset;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1263: Arena allocator - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1263: empty output");
    assert!(
        code.contains("fn vm_arena_alloc"),
        "C1263: Should contain vm_arena_alloc function"
    );
}

/// C1264: String interning table for deduplication
#[test]
fn c1264_string_interning() {
    let c_code = r#"
struct vm_intern_entry {
    char data[64];
    int len;
    int hash;
    int used;
};

struct vm_intern_table {
    struct vm_intern_entry entries[128];
    int count;
};

void vm_intern_init(struct vm_intern_table *tbl) {
    int i;
    tbl->count = 0;
    for (i = 0; i < 128; i++) {
        tbl->entries[i].used = 0;
    }
}

static int vm_intern_hash(const char *str, int len) {
    int h = 0;
    int i;
    for (i = 0; i < len; i++) {
        h = h * 31 + str[i];
    }
    return h & 0x7FFFFFFF;
}

static int vm_intern_streq(const char *a, int alen, const char *b, int blen) {
    int i;
    if (alen != blen) return 0;
    for (i = 0; i < alen; i++) {
        if (a[i] != b[i]) return 0;
    }
    return 1;
}

int vm_intern_find_or_add(struct vm_intern_table *tbl, const char *str, int len) {
    int h = vm_intern_hash(str, len);
    int i;
    for (i = 0; i < tbl->count; i++) {
        if (tbl->entries[i].used && tbl->entries[i].hash == h) {
            if (vm_intern_streq(tbl->entries[i].data, tbl->entries[i].len, str, len)) {
                return i;
            }
        }
    }
    if (tbl->count >= 128 || len >= 64) return -1;
    i = tbl->count;
    tbl->entries[i].hash = h;
    tbl->entries[i].len = len;
    tbl->entries[i].used = 1;
    tbl->count++;
    return i;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1264: String interning - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1264: empty output");
    assert!(
        code.contains("fn vm_intern_find_or_add"),
        "C1264: Should contain vm_intern_find_or_add function"
    );
}

/// C1265: Weak reference tracking for cycle breaking
#[test]
fn c1265_weak_references() {
    let c_code = r#"
struct vm_weak_ref {
    int target_id;
    int alive;
    int generation;
};

struct vm_weak_table {
    struct vm_weak_ref refs[64];
    int count;
    int current_gen;
};

void vm_weak_init(struct vm_weak_table *wt) {
    wt->count = 0;
    wt->current_gen = 1;
}

int vm_weak_register(struct vm_weak_table *wt, int target) {
    int idx;
    if (wt->count >= 64) return -1;
    idx = wt->count;
    wt->refs[idx].target_id = target;
    wt->refs[idx].alive = 1;
    wt->refs[idx].generation = wt->current_gen;
    wt->count++;
    return idx;
}

int vm_weak_deref(struct vm_weak_table *wt, int idx) {
    if (idx < 0 || idx >= wt->count) return -1;
    if (!wt->refs[idx].alive) return -2;
    return wt->refs[idx].target_id;
}

void vm_weak_invalidate(struct vm_weak_table *wt, int target) {
    int i;
    for (i = 0; i < wt->count; i++) {
        if (wt->refs[i].target_id == target && wt->refs[i].alive) {
            wt->refs[i].alive = 0;
        }
    }
}

void vm_weak_advance_gen(struct vm_weak_table *wt) {
    wt->current_gen++;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1265: Weak references - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1265: empty output");
    assert!(
        code.contains("fn vm_weak_register"),
        "C1265: Should contain vm_weak_register function"
    );
}

// ============================================================================
// C1266-C1270: Types
// ============================================================================

/// C1266: Tagged union (NaN-boxed) value representation
#[test]
fn c1266_tagged_union_values() {
    let c_code = r#"
enum vm_tag {
    VM_TAG_INT = 0,
    VM_TAG_FLOAT,
    VM_TAG_BOOL,
    VM_TAG_NIL,
    VM_TAG_OBJ
};

struct vm_tagged_val {
    enum vm_tag tag;
    int int_val;
    double float_val;
    int obj_id;
};

struct vm_tagged_val vm_make_int(int v) {
    struct vm_tagged_val val;
    val.tag = VM_TAG_INT;
    val.int_val = v;
    val.float_val = 0.0;
    val.obj_id = -1;
    return val;
}

struct vm_tagged_val vm_make_float(double v) {
    struct vm_tagged_val val;
    val.tag = VM_TAG_FLOAT;
    val.int_val = 0;
    val.float_val = v;
    val.obj_id = -1;
    return val;
}

struct vm_tagged_val vm_make_nil(void) {
    struct vm_tagged_val val;
    val.tag = VM_TAG_NIL;
    val.int_val = 0;
    val.float_val = 0.0;
    val.obj_id = -1;
    return val;
}

int vm_is_truthy(struct vm_tagged_val *v) {
    switch (v->tag) {
        case VM_TAG_INT: return v->int_val != 0;
        case VM_TAG_FLOAT: return v->float_val != 0.0;
        case VM_TAG_BOOL: return v->int_val != 0;
        case VM_TAG_NIL: return 0;
        case VM_TAG_OBJ: return 1;
        default: return 0;
    }
}

int vm_values_equal(struct vm_tagged_val *a, struct vm_tagged_val *b) {
    if (a->tag != b->tag) return 0;
    switch (a->tag) {
        case VM_TAG_INT: return a->int_val == b->int_val;
        case VM_TAG_FLOAT: return a->float_val == b->float_val;
        case VM_TAG_NIL: return 1;
        default: return 0;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1266: Tagged union values - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1266: empty output");
    assert!(
        code.contains("fn vm_is_truthy"),
        "C1266: Should contain vm_is_truthy function"
    );
}

/// C1267: Runtime type checker for dynamic dispatch
#[test]
fn c1267_runtime_type_checker() {
    let c_code = r#"
enum vm_type_kind {
    VM_TY_INT = 0,
    VM_TY_FLOAT,
    VM_TY_STRING,
    VM_TY_ARRAY,
    VM_TY_FUNC,
    VM_TY_NULL
};

struct vm_type_info {
    enum vm_type_kind kind;
    int element_type;
    int param_count;
    int return_type;
};

struct vm_type_registry {
    struct vm_type_info types[64];
    int count;
};

void vm_typereg_init(struct vm_type_registry *reg) {
    reg->count = 0;
}

int vm_typereg_add(struct vm_type_registry *reg, enum vm_type_kind kind) {
    int idx;
    if (reg->count >= 64) return -1;
    idx = reg->count;
    reg->types[idx].kind = kind;
    reg->types[idx].element_type = -1;
    reg->types[idx].param_count = 0;
    reg->types[idx].return_type = -1;
    reg->count++;
    return idx;
}

int vm_typecheck_compatible(struct vm_type_registry *reg, int a, int b) {
    if (a < 0 || a >= reg->count || b < 0 || b >= reg->count) return 0;
    if (reg->types[a].kind == reg->types[b].kind) return 1;
    if (reg->types[a].kind == VM_TY_INT && reg->types[b].kind == VM_TY_FLOAT) return 1;
    if (reg->types[a].kind == VM_TY_FLOAT && reg->types[b].kind == VM_TY_INT) return 1;
    return 0;
}

int vm_typecheck_is_numeric(struct vm_type_registry *reg, int id) {
    if (id < 0 || id >= reg->count) return 0;
    return reg->types[id].kind == VM_TY_INT || reg->types[id].kind == VM_TY_FLOAT;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1267: Runtime type checker - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1267: empty output");
    assert!(
        code.contains("fn vm_typecheck_compatible"),
        "C1267: Should contain vm_typecheck_compatible function"
    );
}

/// C1268: Method dispatch table for object-oriented runtime
#[test]
fn c1268_method_dispatch() {
    let c_code = r#"
typedef int (*vm_method_fn)(int, int);

struct vm_method_entry {
    int name_hash;
    int class_id;
    int method_idx;
};

struct vm_method_table {
    struct vm_method_entry entries[128];
    int count;
};

void vm_mtable_init(struct vm_method_table *mt) {
    mt->count = 0;
}

int vm_mtable_register(struct vm_method_table *mt, int name_hash, int class_id, int method_idx) {
    if (mt->count >= 128) return -1;
    mt->entries[mt->count].name_hash = name_hash;
    mt->entries[mt->count].class_id = class_id;
    mt->entries[mt->count].method_idx = method_idx;
    mt->count++;
    return 0;
}

int vm_mtable_lookup(struct vm_method_table *mt, int name_hash, int class_id) {
    int i;
    for (i = 0; i < mt->count; i++) {
        if (mt->entries[i].name_hash == name_hash && mt->entries[i].class_id == class_id) {
            return mt->entries[i].method_idx;
        }
    }
    return -1;
}

int vm_mtable_has_method(struct vm_method_table *mt, int name_hash, int class_id) {
    return vm_mtable_lookup(mt, name_hash, class_id) >= 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1268: Method dispatch - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1268: empty output");
    assert!(
        code.contains("fn vm_mtable_lookup"),
        "C1268: Should contain vm_mtable_lookup function"
    );
}

/// C1269: Virtual table (vtable) lookup for class hierarchy
#[test]
fn c1269_vtable_lookup() {
    let c_code = r#"
struct vm_vtable_slot {
    int func_id;
    int overridden;
};

struct vm_vtable {
    struct vm_vtable_slot slots[32];
    int slot_count;
    int parent_class;
    int class_id;
};

struct vm_vtable_registry {
    struct vm_vtable tables[32];
    int count;
};

void vm_vtreg_init(struct vm_vtable_registry *reg) {
    reg->count = 0;
}

int vm_vtreg_add_class(struct vm_vtable_registry *reg, int class_id, int parent) {
    int idx;
    int i;
    if (reg->count >= 32) return -1;
    idx = reg->count;
    reg->tables[idx].class_id = class_id;
    reg->tables[idx].parent_class = parent;
    reg->tables[idx].slot_count = 0;
    for (i = 0; i < 32; i++) {
        reg->tables[idx].slots[i].func_id = -1;
        reg->tables[idx].slots[i].overridden = 0;
    }
    reg->count++;
    return idx;
}

int vm_vtreg_set_method(struct vm_vtable_registry *reg, int class_idx, int slot, int func_id) {
    if (class_idx < 0 || class_idx >= reg->count) return -1;
    if (slot < 0 || slot >= 32) return -1;
    reg->tables[class_idx].slots[slot].func_id = func_id;
    reg->tables[class_idx].slots[slot].overridden = 1;
    if (slot >= reg->tables[class_idx].slot_count) {
        reg->tables[class_idx].slot_count = slot + 1;
    }
    return 0;
}

int vm_vtreg_resolve(struct vm_vtable_registry *reg, int class_idx, int slot) {
    if (class_idx < 0 || class_idx >= reg->count) return -1;
    if (slot < 0 || slot >= 32) return -1;
    if (reg->tables[class_idx].slots[slot].func_id >= 0) {
        return reg->tables[class_idx].slots[slot].func_id;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1269: Vtable lookup - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1269: empty output");
    assert!(
        code.contains("fn vm_vtreg_resolve"),
        "C1269: Should contain vm_vtreg_resolve function"
    );
}

/// C1270: Generic/template instantiation registry
#[test]
fn c1270_generic_instantiation() {
    let c_code = r#"
struct vm_generic_param {
    int type_id;
    int constraint;
};

struct vm_generic_inst {
    int base_func;
    int params[4];
    int param_count;
    int specialized_func;
};

struct vm_generic_registry {
    struct vm_generic_inst instances[64];
    int count;
};

void vm_generics_init(struct vm_generic_registry *reg) {
    reg->count = 0;
}

int vm_generics_instantiate(struct vm_generic_registry *reg, int base_func, int *params, int pcount) {
    int idx, i, j, match;
    for (i = 0; i < reg->count; i++) {
        if (reg->instances[i].base_func != base_func) continue;
        if (reg->instances[i].param_count != pcount) continue;
        match = 1;
        for (j = 0; j < pcount; j++) {
            if (reg->instances[i].params[j] != params[j]) {
                match = 0;
                break;
            }
        }
        if (match) return reg->instances[i].specialized_func;
    }
    if (reg->count >= 64 || pcount > 4) return -1;
    idx = reg->count;
    reg->instances[idx].base_func = base_func;
    reg->instances[idx].param_count = pcount;
    for (i = 0; i < pcount; i++) {
        reg->instances[idx].params[i] = params[i];
    }
    reg->instances[idx].specialized_func = base_func + reg->count + 1000;
    reg->count++;
    return reg->instances[idx].specialized_func;
}

int vm_generics_count(struct vm_generic_registry *reg) {
    return reg->count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1270: Generic instantiation - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1270: empty output");
    assert!(
        code.contains("fn vm_generics_instantiate"),
        "C1270: Should contain vm_generics_instantiate function"
    );
}

// ============================================================================
// C1271-C1275: Optimization
// ============================================================================

/// C1271: Peephole optimizer for bytecode sequences
#[test]
fn c1271_peephole_optimizer() {
    let c_code = r#"
typedef unsigned char uint8_t;

enum vm_peep_op {
    VM_POP_NOP = 0,
    VM_POP_PUSH,
    VM_POP_POP,
    VM_POP_ADD,
    VM_POP_LOAD,
    VM_POP_STORE,
    VM_POP_DUP,
    VM_POP_SWAP
};

struct vm_peephole {
    uint8_t input[512];
    uint8_t output[512];
    int in_len;
    int out_len;
    int changes;
};

void vm_peep_init(struct vm_peephole *p, uint8_t *code, int len) {
    int i;
    p->in_len = len;
    p->out_len = 0;
    p->changes = 0;
    for (i = 0; i < len && i < 512; i++) {
        p->input[i] = code[i];
    }
}

static void vm_peep_emit(struct vm_peephole *p, uint8_t op) {
    if (p->out_len < 512) {
        p->output[p->out_len] = op;
        p->out_len++;
    }
}

int vm_peep_optimize(struct vm_peephole *p) {
    int i = 0;
    p->out_len = 0;
    p->changes = 0;
    while (i < p->in_len) {
        if (i + 1 < p->in_len && p->input[i] == VM_POP_PUSH && p->input[i + 1] == VM_POP_POP) {
            p->changes++;
            i += 2;
        } else if (i + 1 < p->in_len && p->input[i] == VM_POP_NOP && p->input[i + 1] == VM_POP_NOP) {
            p->changes++;
            i += 2;
        } else {
            vm_peep_emit(p, p->input[i]);
            i++;
        }
    }
    return p->changes;
}

int vm_peep_output_len(struct vm_peephole *p) {
    return p->out_len;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1271: Peephole optimizer - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1271: empty output");
    assert!(
        code.contains("fn vm_peep_optimize"),
        "C1271: Should contain vm_peep_optimize function"
    );
}

/// C1272: Constant folding pass for compile-time evaluation
#[test]
fn c1272_constant_folding() {
    let c_code = r#"
enum vm_fold_op {
    VM_FOLD_ADD = 0,
    VM_FOLD_SUB,
    VM_FOLD_MUL,
    VM_FOLD_DIV,
    VM_FOLD_NEG,
    VM_FOLD_NOT
};

struct vm_fold_expr {
    int is_const;
    int value;
    enum vm_fold_op op;
    int left;
    int right;
};

struct vm_folder {
    struct vm_fold_expr exprs[128];
    int count;
    int folded;
};

void vm_folder_init(struct vm_folder *f) {
    f->count = 0;
    f->folded = 0;
}

int vm_folder_add_const(struct vm_folder *f, int val) {
    int idx;
    if (f->count >= 128) return -1;
    idx = f->count;
    f->exprs[idx].is_const = 1;
    f->exprs[idx].value = val;
    f->exprs[idx].left = -1;
    f->exprs[idx].right = -1;
    f->count++;
    return idx;
}

int vm_folder_add_binop(struct vm_folder *f, enum vm_fold_op op, int left, int right) {
    int idx;
    if (f->count >= 128) return -1;
    idx = f->count;
    f->exprs[idx].is_const = 0;
    f->exprs[idx].op = op;
    f->exprs[idx].left = left;
    f->exprs[idx].right = right;
    f->count++;
    return idx;
}

int vm_folder_try_fold(struct vm_folder *f, int idx) {
    int lv, rv;
    struct vm_fold_expr *e;
    if (idx < 0 || idx >= f->count) return 0;
    e = &f->exprs[idx];
    if (e->is_const) return 1;
    if (e->left < 0 || e->right < 0) return 0;
    if (!f->exprs[e->left].is_const || !f->exprs[e->right].is_const) return 0;
    lv = f->exprs[e->left].value;
    rv = f->exprs[e->right].value;
    switch (e->op) {
        case VM_FOLD_ADD: e->value = lv + rv; break;
        case VM_FOLD_SUB: e->value = lv - rv; break;
        case VM_FOLD_MUL: e->value = lv * rv; break;
        case VM_FOLD_DIV:
            if (rv == 0) return 0;
            e->value = lv / rv;
            break;
        default: return 0;
    }
    e->is_const = 1;
    f->folded++;
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1272: Constant folding - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1272: empty output");
    assert!(
        code.contains("fn vm_folder_try_fold"),
        "C1272: Should contain vm_folder_try_fold function"
    );
}

/// C1273: Dead code elimination via reachability analysis
#[test]
fn c1273_dead_code_elimination() {
    let c_code = r#"
struct vm_dce_instr {
    int opcode;
    int dest;
    int src1;
    int src2;
    int alive;
};

struct vm_dce_pass {
    struct vm_dce_instr instrs[256];
    int count;
    int used[64];
    int eliminated;
};

void vm_dce_init(struct vm_dce_pass *dce) {
    int i;
    dce->count = 0;
    dce->eliminated = 0;
    for (i = 0; i < 64; i++) dce->used[i] = 0;
}

int vm_dce_add_instr(struct vm_dce_pass *dce, int opcode, int dest, int src1, int src2) {
    int idx;
    if (dce->count >= 256) return -1;
    idx = dce->count;
    dce->instrs[idx].opcode = opcode;
    dce->instrs[idx].dest = dest;
    dce->instrs[idx].src1 = src1;
    dce->instrs[idx].src2 = src2;
    dce->instrs[idx].alive = 1;
    dce->count++;
    return idx;
}

void vm_dce_mark_used(struct vm_dce_pass *dce, int reg) {
    if (reg >= 0 && reg < 64) dce->used[reg] = 1;
}

int vm_dce_eliminate(struct vm_dce_pass *dce) {
    int i;
    dce->eliminated = 0;
    for (i = 0; i < dce->count; i++) {
        int d = dce->instrs[i].dest;
        if (d >= 0 && d < 64 && !dce->used[d]) {
            dce->instrs[i].alive = 0;
            dce->eliminated++;
        }
        if (dce->instrs[i].alive) {
            if (dce->instrs[i].src1 >= 0 && dce->instrs[i].src1 < 64)
                dce->used[dce->instrs[i].src1] = 1;
            if (dce->instrs[i].src2 >= 0 && dce->instrs[i].src2 < 64)
                dce->used[dce->instrs[i].src2] = 1;
        }
    }
    return dce->eliminated;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1273: Dead code elimination - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1273: empty output");
    assert!(
        code.contains("fn vm_dce_eliminate"),
        "C1273: Should contain vm_dce_eliminate function"
    );
}

/// C1274: Linear scan register allocator
#[test]
fn c1274_register_allocator() {
    let c_code = r#"
struct vm_live_range {
    int vreg;
    int start;
    int end;
    int phys_reg;
    int spilled;
};

struct vm_regalloc {
    struct vm_live_range ranges[64];
    int range_count;
    int reg_available[8];
    int num_regs;
    int spill_count;
};

void vm_regalloc_init(struct vm_regalloc *ra, int num_regs) {
    int i;
    ra->range_count = 0;
    ra->num_regs = num_regs;
    ra->spill_count = 0;
    for (i = 0; i < 8 && i < num_regs; i++) {
        ra->reg_available[i] = 1;
    }
}

int vm_regalloc_add_range(struct vm_regalloc *ra, int vreg, int start, int end) {
    int idx;
    if (ra->range_count >= 64) return -1;
    idx = ra->range_count;
    ra->ranges[idx].vreg = vreg;
    ra->ranges[idx].start = start;
    ra->ranges[idx].end = end;
    ra->ranges[idx].phys_reg = -1;
    ra->ranges[idx].spilled = 0;
    ra->range_count++;
    return idx;
}

static int vm_regalloc_find_free(struct vm_regalloc *ra) {
    int i;
    for (i = 0; i < ra->num_regs && i < 8; i++) {
        if (ra->reg_available[i]) return i;
    }
    return -1;
}

int vm_regalloc_allocate(struct vm_regalloc *ra) {
    int i, reg;
    ra->spill_count = 0;
    for (i = 0; i < ra->range_count; i++) {
        reg = vm_regalloc_find_free(ra);
        if (reg >= 0) {
            ra->ranges[i].phys_reg = reg;
            ra->reg_available[reg] = 0;
        } else {
            ra->ranges[i].spilled = 1;
            ra->spill_count++;
        }
    }
    return ra->spill_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1274: Register allocator - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1274: empty output");
    assert!(
        code.contains("fn vm_regalloc_allocate"),
        "C1274: Should contain vm_regalloc_allocate function"
    );
}

/// C1275: Basic block construction from linear instruction stream
#[test]
fn c1275_basic_block_construction() {
    let c_code = r#"
struct vm_bb_instr {
    int opcode;
    int arg;
    int is_jump;
    int is_branch;
    int target;
};

struct vm_basic_block {
    int start;
    int end;
    int successor1;
    int successor2;
    int visited;
};

struct vm_cfg_builder {
    struct vm_bb_instr instrs[128];
    int instr_count;
    struct vm_basic_block blocks[32];
    int block_count;
};

void vm_cfg_init(struct vm_cfg_builder *cfg) {
    cfg->instr_count = 0;
    cfg->block_count = 0;
}

int vm_cfg_add_instr(struct vm_cfg_builder *cfg, int opcode, int arg, int is_jump, int target) {
    int idx;
    if (cfg->instr_count >= 128) return -1;
    idx = cfg->instr_count;
    cfg->instrs[idx].opcode = opcode;
    cfg->instrs[idx].arg = arg;
    cfg->instrs[idx].is_jump = is_jump;
    cfg->instrs[idx].is_branch = 0;
    cfg->instrs[idx].target = target;
    cfg->instr_count++;
    return idx;
}

int vm_cfg_build_blocks(struct vm_cfg_builder *cfg) {
    int i, block_start = 0;
    cfg->block_count = 0;
    for (i = 0; i < cfg->instr_count; i++) {
        if (cfg->instrs[i].is_jump || i == cfg->instr_count - 1) {
            if (cfg->block_count >= 32) return -1;
            cfg->blocks[cfg->block_count].start = block_start;
            cfg->blocks[cfg->block_count].end = i;
            cfg->blocks[cfg->block_count].successor1 = -1;
            cfg->blocks[cfg->block_count].successor2 = -1;
            cfg->blocks[cfg->block_count].visited = 0;
            if (cfg->instrs[i].is_jump) {
                cfg->blocks[cfg->block_count].successor1 = cfg->instrs[i].target;
            }
            cfg->block_count++;
            block_start = i + 1;
        }
    }
    return cfg->block_count;
}

int vm_cfg_block_count(struct vm_cfg_builder *cfg) {
    return cfg->block_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1275: Basic block construction - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1275: empty output");
    assert!(
        code.contains("fn vm_cfg_build_blocks"),
        "C1275: Should contain vm_cfg_build_blocks function"
    );
}
