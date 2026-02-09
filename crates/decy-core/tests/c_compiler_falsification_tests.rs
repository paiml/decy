//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C376-C400: Compilers, Interpreters, and Language Runtimes -- the kind of
//! C code found in parsers, lexers, type checkers, bytecode interpreters,
//! and garbage collectors.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world compiler and runtime patterns commonly
//! found in GCC, LLVM, CPython, Lua, V8, and similar projects -- all
//! expressed as valid C99.
//!
//! Organization:
//! - C376-C380: Frontend (lexer, parser, AST, symbol table, type checker)
//! - C381-C385: Backend and GC (bytecode VM, register alloc, SSA, mark-sweep, copying GC)
//! - C386-C390: Optimization (refcounting, instruction encoding, const fold, DCE, closures)
//! - C391-C395: Advanced (pattern match, TCO, exceptions, string interning, opcode dispatch)
//! - C396-C400: Runtime (JIT buffer, debug info, modules, peephole, type tags)

// ============================================================================
// C376-C380: Frontend (Lexer, Parser, AST, Symbol Table, Type Checker)
// ============================================================================

#[test]
fn c376_lexer_with_token_types() {
    let c_code = r#"
typedef unsigned char uint8_t;

enum token_kind {
    TOK_EOF = 0,
    TOK_IDENT,
    TOK_NUMBER,
    TOK_STRING,
    TOK_PLUS,
    TOK_MINUS,
    TOK_STAR,
    TOK_SLASH,
    TOK_LPAREN,
    TOK_RPAREN,
    TOK_SEMI,
    TOK_ASSIGN,
    TOK_EQ,
    TOK_NEQ,
    TOK_LT,
    TOK_GT
};

struct token {
    enum token_kind kind;
    int start;
    int length;
    int line;
};

struct lexer {
    const char *source;
    int pos;
    int line;
    int length;
};

void lexer_init(struct lexer *lex, const char *src, int len) {
    lex->source = src;
    lex->pos = 0;
    lex->line = 1;
    lex->length = len;
}

static int is_alpha(char c) {
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_';
}

static int is_digit(char c) {
    return c >= '0' && c <= '9';
}

static char lexer_peek(struct lexer *lex) {
    if (lex->pos >= lex->length) return '\0';
    return lex->source[lex->pos];
}

static char lexer_advance(struct lexer *lex) {
    char c = lex->source[lex->pos];
    lex->pos++;
    if (c == '\n') lex->line++;
    return c;
}

static void skip_whitespace(struct lexer *lex) {
    while (lex->pos < lex->length) {
        char c = lex->source[lex->pos];
        if (c == ' ' || c == '\t' || c == '\r' || c == '\n') {
            lexer_advance(lex);
        } else {
            break;
        }
    }
}

struct token lexer_next(struct lexer *lex) {
    struct token tok;
    skip_whitespace(lex);

    tok.line = lex->line;
    tok.start = lex->pos;

    if (lex->pos >= lex->length) {
        tok.kind = TOK_EOF;
        tok.length = 0;
        return tok;
    }

    char c = lexer_advance(lex);

    if (is_alpha(c)) {
        while (lex->pos < lex->length &&
               (is_alpha(lexer_peek(lex)) || is_digit(lexer_peek(lex)))) {
            lexer_advance(lex);
        }
        tok.kind = TOK_IDENT;
        tok.length = lex->pos - tok.start;
        return tok;
    }

    if (is_digit(c)) {
        while (lex->pos < lex->length && is_digit(lexer_peek(lex))) {
            lexer_advance(lex);
        }
        tok.kind = TOK_NUMBER;
        tok.length = lex->pos - tok.start;
        return tok;
    }

    tok.length = 1;
    switch (c) {
        case '+': tok.kind = TOK_PLUS; break;
        case '-': tok.kind = TOK_MINUS; break;
        case '*': tok.kind = TOK_STAR; break;
        case '/': tok.kind = TOK_SLASH; break;
        case '(': tok.kind = TOK_LPAREN; break;
        case ')': tok.kind = TOK_RPAREN; break;
        case ';': tok.kind = TOK_SEMI; break;
        case '=':
            if (lexer_peek(lex) == '=') {
                lexer_advance(lex);
                tok.kind = TOK_EQ;
                tok.length = 2;
            } else {
                tok.kind = TOK_ASSIGN;
            }
            break;
        case '<': tok.kind = TOK_LT; break;
        case '>': tok.kind = TOK_GT; break;
        default: tok.kind = TOK_EOF; break;
    }
    return tok;
}

int count_tokens(const char *src, int len) {
    struct lexer lex;
    int count = 0;
    lexer_init(&lex, src, len);
    while (1) {
        struct token tok = lexer_next(&lex);
        if (tok.kind == TOK_EOF) break;
        count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C376: Lexer with token types - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C376: empty output");
    assert!(
        code.contains("fn lexer_next"),
        "C376: Should contain lexer_next function"
    );
}

#[test]
fn c377_recursive_descent_parser() {
    let c_code = r#"
enum node_type {
    NODE_NUM = 0,
    NODE_ADD,
    NODE_SUB,
    NODE_MUL,
    NODE_DIV,
    NODE_NEG
};

struct ast_node {
    enum node_type type;
    int value;
    int left;
    int right;
};

struct parser {
    const char *input;
    int pos;
    int length;
    struct ast_node nodes[256];
    int node_count;
};

void parser_init(struct parser *p, const char *input, int len) {
    p->input = input;
    p->pos = 0;
    p->length = len;
    p->node_count = 0;
}

static int alloc_node(struct parser *p, enum node_type type, int val, int left, int right) {
    if (p->node_count >= 256) return -1;
    int idx = p->node_count;
    p->nodes[idx].type = type;
    p->nodes[idx].value = val;
    p->nodes[idx].left = left;
    p->nodes[idx].right = right;
    p->node_count++;
    return idx;
}

static char parser_peek(struct parser *p) {
    while (p->pos < p->length && p->input[p->pos] == ' ') p->pos++;
    if (p->pos >= p->length) return '\0';
    return p->input[p->pos];
}

static int parse_number(struct parser *p) {
    int val = 0;
    while (p->pos < p->length && p->input[p->pos] >= '0' && p->input[p->pos] <= '9') {
        val = val * 10 + (p->input[p->pos] - '0');
        p->pos++;
    }
    return alloc_node(p, NODE_NUM, val, -1, -1);
}

static int parse_expr(struct parser *p);

static int parse_primary(struct parser *p) {
    char c = parser_peek(p);
    if (c == '(') {
        p->pos++;
        int inner = parse_expr(p);
        if (parser_peek(p) == ')') p->pos++;
        return inner;
    }
    if (c == '-') {
        p->pos++;
        int operand = parse_primary(p);
        return alloc_node(p, NODE_NEG, 0, operand, -1);
    }
    return parse_number(p);
}

static int parse_term(struct parser *p) {
    int left = parse_primary(p);
    while (1) {
        char c = parser_peek(p);
        if (c == '*') {
            p->pos++;
            int right = parse_primary(p);
            left = alloc_node(p, NODE_MUL, 0, left, right);
        } else if (c == '/') {
            p->pos++;
            int right = parse_primary(p);
            left = alloc_node(p, NODE_DIV, 0, left, right);
        } else {
            break;
        }
    }
    return left;
}

static int parse_expr(struct parser *p) {
    int left = parse_term(p);
    while (1) {
        char c = parser_peek(p);
        if (c == '+') {
            p->pos++;
            int right = parse_term(p);
            left = alloc_node(p, NODE_ADD, 0, left, right);
        } else if (c == '-') {
            p->pos++;
            int right = parse_term(p);
            left = alloc_node(p, NODE_SUB, 0, left, right);
        } else {
            break;
        }
    }
    return left;
}

int eval_node(const struct parser *p, int idx) {
    if (idx < 0) return 0;
    const struct ast_node *n = &p->nodes[idx];
    switch (n->type) {
        case NODE_NUM: return n->value;
        case NODE_ADD: return eval_node(p, n->left) + eval_node(p, n->right);
        case NODE_SUB: return eval_node(p, n->left) - eval_node(p, n->right);
        case NODE_MUL: return eval_node(p, n->left) * eval_node(p, n->right);
        case NODE_DIV: {
            int d = eval_node(p, n->right);
            if (d == 0) return 0;
            return eval_node(p, n->left) / d;
        }
        case NODE_NEG: return -eval_node(p, n->left);
        default: return 0;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C377: Recursive descent parser - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C377: empty output");
    assert!(
        code.contains("fn eval_node"),
        "C377: Should contain eval_node function"
    );
}

#[test]
fn c378_ast_node_allocation() {
    let c_code = r#"
typedef unsigned int uint32_t;

enum ast_kind {
    AST_LITERAL = 0,
    AST_IDENT,
    AST_BINOP,
    AST_UNARYOP,
    AST_CALL,
    AST_IF,
    AST_WHILE,
    AST_BLOCK,
    AST_RETURN
};

struct ast_node {
    enum ast_kind kind;
    int data;
    int children[4];
    int num_children;
    int line;
    int col;
};

struct ast_pool {
    struct ast_node nodes[1024];
    int count;
    int capacity;
};

void pool_init(struct ast_pool *pool) {
    pool->count = 0;
    pool->capacity = 1024;
}

int pool_alloc(struct ast_pool *pool, enum ast_kind kind, int data, int line, int col) {
    if (pool->count >= pool->capacity) return -1;
    int idx = pool->count;
    pool->nodes[idx].kind = kind;
    pool->nodes[idx].data = data;
    pool->nodes[idx].num_children = 0;
    pool->nodes[idx].line = line;
    pool->nodes[idx].col = col;
    int i;
    for (i = 0; i < 4; i++) pool->nodes[idx].children[i] = -1;
    pool->count++;
    return idx;
}

int pool_add_child(struct ast_pool *pool, int parent, int child) {
    if (parent < 0 || parent >= pool->count) return -1;
    struct ast_node *node = &pool->nodes[parent];
    if (node->num_children >= 4) return -2;
    node->children[node->num_children] = child;
    node->num_children++;
    return 0;
}

int pool_node_depth(const struct ast_pool *pool, int idx) {
    if (idx < 0 || idx >= pool->count) return 0;
    const struct ast_node *node = &pool->nodes[idx];
    int max_depth = 0;
    int i;
    for (i = 0; i < node->num_children; i++) {
        int d = pool_node_depth(pool, node->children[i]);
        if (d > max_depth) max_depth = d;
    }
    return max_depth + 1;
}

int pool_count_kind(const struct ast_pool *pool, enum ast_kind kind) {
    int count = 0;
    int i;
    for (i = 0; i < pool->count; i++) {
        if (pool->nodes[i].kind == kind) count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C378: AST node allocation - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C378: empty output");
    assert!(
        code.contains("fn pool_alloc"),
        "C378: Should contain pool_alloc function"
    );
}

#[test]
fn c379_symbol_table_with_scope_chaining() {
    let c_code = r#"
enum sym_kind {
    SYM_VAR = 0,
    SYM_FUNC,
    SYM_TYPE,
    SYM_CONST
};

struct symbol {
    char name[64];
    enum sym_kind kind;
    int type_id;
    int scope_depth;
    int is_defined;
};

struct scope {
    struct symbol symbols[64];
    int count;
    int parent_scope;
    int depth;
};

struct sym_table {
    struct scope scopes[32];
    int scope_count;
    int current_scope;
};

void symtab_init(struct sym_table *st) {
    st->scope_count = 1;
    st->current_scope = 0;
    st->scopes[0].count = 0;
    st->scopes[0].parent_scope = -1;
    st->scopes[0].depth = 0;
}

int symtab_push_scope(struct sym_table *st) {
    if (st->scope_count >= 32) return -1;
    int idx = st->scope_count;
    st->scopes[idx].count = 0;
    st->scopes[idx].parent_scope = st->current_scope;
    st->scopes[idx].depth = st->scopes[st->current_scope].depth + 1;
    st->scope_count++;
    st->current_scope = idx;
    return idx;
}

int symtab_pop_scope(struct sym_table *st) {
    if (st->current_scope == 0) return -1;
    int parent = st->scopes[st->current_scope].parent_scope;
    st->current_scope = parent;
    return parent;
}

static int str_eq(const char *a, const char *b) {
    int i;
    for (i = 0; a[i] && b[i]; i++) {
        if (a[i] != b[i]) return 0;
    }
    return a[i] == b[i];
}

static void str_copy(char *dst, const char *src, int max) {
    int i;
    for (i = 0; i < max - 1 && src[i]; i++) {
        dst[i] = src[i];
    }
    dst[i] = '\0';
}

int symtab_insert(struct sym_table *st, const char *name, enum sym_kind kind, int type_id) {
    struct scope *sc = &st->scopes[st->current_scope];
    if (sc->count >= 64) return -1;
    int idx = sc->count;
    str_copy(sc->symbols[idx].name, name, 64);
    sc->symbols[idx].kind = kind;
    sc->symbols[idx].type_id = type_id;
    sc->symbols[idx].scope_depth = sc->depth;
    sc->symbols[idx].is_defined = 1;
    sc->count++;
    return 0;
}

int symtab_lookup(const struct sym_table *st, const char *name) {
    int scope_idx = st->current_scope;
    while (scope_idx >= 0) {
        const struct scope *sc = &st->scopes[scope_idx];
        int i;
        for (i = 0; i < sc->count; i++) {
            if (str_eq(sc->symbols[i].name, name)) return 1;
        }
        scope_idx = sc->parent_scope;
    }
    return 0;
}

int symtab_current_depth(const struct sym_table *st) {
    return st->scopes[st->current_scope].depth;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C379: Symbol table with scope chaining - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C379: empty output");
    assert!(
        code.contains("fn symtab_lookup"),
        "C379: Should contain symtab_lookup function"
    );
}

#[test]
fn c380_type_checker_unification() {
    let c_code = r#"
enum type_kind {
    TYPE_INT = 0,
    TYPE_FLOAT,
    TYPE_BOOL,
    TYPE_STRING,
    TYPE_FUNC,
    TYPE_VAR,
    TYPE_ERROR
};

struct type_node {
    enum type_kind kind;
    int param_type;
    int return_type;
    int binding;
};

struct type_env {
    struct type_node types[256];
    int count;
};

void type_env_init(struct type_env *env) {
    env->count = 0;
}

int type_new(struct type_env *env, enum type_kind kind) {
    if (env->count >= 256) return -1;
    int idx = env->count;
    env->types[idx].kind = kind;
    env->types[idx].param_type = -1;
    env->types[idx].return_type = -1;
    env->types[idx].binding = -1;
    env->count++;
    return idx;
}

int type_new_func(struct type_env *env, int param, int ret) {
    int idx = type_new(env, TYPE_FUNC);
    if (idx < 0) return -1;
    env->types[idx].param_type = param;
    env->types[idx].return_type = ret;
    return idx;
}

int type_new_var(struct type_env *env) {
    return type_new(env, TYPE_VAR);
}

static int find_root(struct type_env *env, int idx) {
    if (idx < 0 || idx >= env->count) return idx;
    while (env->types[idx].kind == TYPE_VAR && env->types[idx].binding >= 0) {
        idx = env->types[idx].binding;
    }
    return idx;
}

int type_unify(struct type_env *env, int a, int b) {
    a = find_root(env, a);
    b = find_root(env, b);
    if (a == b) return 0;
    if (a < 0 || b < 0) return -1;

    struct type_node *ta = &env->types[a];
    struct type_node *tb = &env->types[b];

    if (ta->kind == TYPE_VAR) {
        ta->binding = b;
        return 0;
    }
    if (tb->kind == TYPE_VAR) {
        tb->binding = a;
        return 0;
    }
    if (ta->kind != tb->kind) return -1;

    if (ta->kind == TYPE_FUNC) {
        if (type_unify(env, ta->param_type, tb->param_type) != 0) return -1;
        if (type_unify(env, ta->return_type, tb->return_type) != 0) return -1;
    }
    return 0;
}

int type_resolve(struct type_env *env, int idx) {
    idx = find_root(env, idx);
    if (idx < 0 || idx >= env->count) return -1;
    return env->types[idx].kind;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C380: Type checker unification - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C380: empty output");
    assert!(
        code.contains("fn type_unify"),
        "C380: Should contain type_unify function"
    );
}

// ============================================================================
// C381-C385: Backend and GC (Bytecode VM, Register Alloc, SSA, GC)
// ============================================================================

#[test]
fn c381_stack_based_bytecode_interpreter() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

enum opcode {
    OP_CONST = 0,
    OP_ADD,
    OP_SUB,
    OP_MUL,
    OP_DIV,
    OP_NEG,
    OP_LOAD,
    OP_STORE,
    OP_JUMP,
    OP_JUMP_IF_ZERO,
    OP_CALL,
    OP_RET,
    OP_PRINT,
    OP_HALT
};

struct vm {
    uint8_t code[4096];
    int code_len;
    int stack[256];
    int sp;
    int locals[64];
    int ip;
    int running;
};

void vm_init(struct vm *vm, const uint8_t *bytecode, int len) {
    int i;
    for (i = 0; i < len && i < 4096; i++) {
        vm->code[i] = bytecode[i];
    }
    vm->code_len = len;
    vm->sp = 0;
    vm->ip = 0;
    vm->running = 1;
    for (i = 0; i < 64; i++) vm->locals[i] = 0;
}

static void vm_push(struct vm *vm, int val) {
    if (vm->sp < 256) {
        vm->stack[vm->sp] = val;
        vm->sp++;
    }
}

static int vm_pop(struct vm *vm) {
    if (vm->sp > 0) {
        vm->sp--;
        return vm->stack[vm->sp];
    }
    return 0;
}

static uint16_t read_u16(struct vm *vm) {
    uint16_t val = ((uint16_t)vm->code[vm->ip] << 8) | (uint16_t)vm->code[vm->ip + 1];
    vm->ip += 2;
    return val;
}

void vm_step(struct vm *vm) {
    if (vm->ip >= vm->code_len || !vm->running) {
        vm->running = 0;
        return;
    }

    uint8_t op = vm->code[vm->ip];
    vm->ip++;

    switch (op) {
        case OP_CONST: {
            int val = (int)read_u16(vm);
            vm_push(vm, val);
            break;
        }
        case OP_ADD: {
            int b = vm_pop(vm);
            int a = vm_pop(vm);
            vm_push(vm, a + b);
            break;
        }
        case OP_SUB: {
            int b = vm_pop(vm);
            int a = vm_pop(vm);
            vm_push(vm, a - b);
            break;
        }
        case OP_MUL: {
            int b = vm_pop(vm);
            int a = vm_pop(vm);
            vm_push(vm, a * b);
            break;
        }
        case OP_DIV: {
            int b = vm_pop(vm);
            int a = vm_pop(vm);
            if (b != 0) vm_push(vm, a / b);
            else vm_push(vm, 0);
            break;
        }
        case OP_NEG: {
            int a = vm_pop(vm);
            vm_push(vm, -a);
            break;
        }
        case OP_LOAD: {
            int idx = vm->code[vm->ip];
            vm->ip++;
            if (idx < 64) vm_push(vm, vm->locals[idx]);
            break;
        }
        case OP_STORE: {
            int idx = vm->code[vm->ip];
            vm->ip++;
            if (idx < 64) vm->locals[idx] = vm_pop(vm);
            break;
        }
        case OP_JUMP: {
            uint16_t target = read_u16(vm);
            vm->ip = target;
            break;
        }
        case OP_JUMP_IF_ZERO: {
            uint16_t target = read_u16(vm);
            int cond = vm_pop(vm);
            if (cond == 0) vm->ip = target;
            break;
        }
        case OP_HALT:
            vm->running = 0;
            break;
        default:
            vm->running = 0;
            break;
    }
}

int vm_run(struct vm *vm) {
    int steps = 0;
    while (vm->running && steps < 10000) {
        vm_step(vm);
        steps++;
    }
    return steps;
}

int vm_top(const struct vm *vm) {
    if (vm->sp > 0) return vm->stack[vm->sp - 1];
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C381: Stack-based bytecode interpreter - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C381: empty output");
    assert!(
        code.contains("fn vm_run"),
        "C381: Should contain vm_run function"
    );
}

#[test]
fn c382_register_allocation_linear_scan() {
    let c_code = r#"
struct live_interval {
    int vreg;
    int start;
    int end;
    int assigned_reg;
    int spilled;
};

struct reg_allocator {
    struct live_interval intervals[128];
    int num_intervals;
    int active[16];
    int num_active;
    int num_phys_regs;
    int spill_count;
};

void regalloc_init(struct reg_allocator *ra, int num_regs) {
    ra->num_intervals = 0;
    ra->num_active = 0;
    ra->num_phys_regs = num_regs;
    ra->spill_count = 0;
}

void regalloc_add_interval(struct reg_allocator *ra, int vreg, int start, int end) {
    if (ra->num_intervals >= 128) return;
    int idx = ra->num_intervals;
    ra->intervals[idx].vreg = vreg;
    ra->intervals[idx].start = start;
    ra->intervals[idx].end = end;
    ra->intervals[idx].assigned_reg = -1;
    ra->intervals[idx].spilled = 0;
    ra->num_intervals++;
}

static void sort_by_start(struct reg_allocator *ra) {
    int i, j;
    for (i = 0; i < ra->num_intervals - 1; i++) {
        for (j = 0; j < ra->num_intervals - i - 1; j++) {
            if (ra->intervals[j].start > ra->intervals[j + 1].start) {
                struct live_interval tmp = ra->intervals[j];
                ra->intervals[j] = ra->intervals[j + 1];
                ra->intervals[j + 1] = tmp;
            }
        }
    }
}

static void expire_old(struct reg_allocator *ra, int pos) {
    int i = 0;
    while (i < ra->num_active) {
        int idx = ra->active[i];
        if (ra->intervals[idx].end <= pos) {
            int j;
            for (j = i; j < ra->num_active - 1; j++) {
                ra->active[j] = ra->active[j + 1];
            }
            ra->num_active--;
        } else {
            i++;
        }
    }
}

static int find_free_reg(const struct reg_allocator *ra) {
    int used[16];
    int i;
    for (i = 0; i < 16; i++) used[i] = 0;
    for (i = 0; i < ra->num_active; i++) {
        int reg = ra->intervals[ra->active[i]].assigned_reg;
        if (reg >= 0 && reg < 16) used[reg] = 1;
    }
    for (i = 0; i < ra->num_phys_regs; i++) {
        if (!used[i]) return i;
    }
    return -1;
}

void regalloc_run(struct reg_allocator *ra) {
    sort_by_start(ra);
    int i;
    for (i = 0; i < ra->num_intervals; i++) {
        expire_old(ra, ra->intervals[i].start);
        int reg = find_free_reg(ra);
        if (reg >= 0) {
            ra->intervals[i].assigned_reg = reg;
            ra->active[ra->num_active] = i;
            ra->num_active++;
        } else {
            ra->intervals[i].spilled = 1;
            ra->spill_count++;
        }
    }
}

int regalloc_spill_count(const struct reg_allocator *ra) {
    return ra->spill_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C382: Register allocation linear scan - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C382: empty output");
    assert!(
        code.contains("fn regalloc_run"),
        "C382: Should contain regalloc_run function"
    );
}

#[test]
fn c383_ssa_phi_node_insertion() {
    let c_code = r#"
struct phi_operand {
    int value;
    int block_id;
};

struct phi_node {
    int dest_var;
    struct phi_operand operands[8];
    int num_operands;
};

struct basic_block {
    int id;
    int instructions[32];
    int num_instructions;
    struct phi_node phis[8];
    int num_phis;
    int successors[4];
    int num_successors;
    int predecessors[4];
    int num_predecessors;
    int dom_frontier[8];
    int num_dom_frontier;
    int idom;
};

struct ssa_builder {
    struct basic_block blocks[64];
    int num_blocks;
    int var_defs[32];
    int num_vars;
};

void ssa_init(struct ssa_builder *ssa) {
    ssa->num_blocks = 0;
    ssa->num_vars = 0;
}

int ssa_add_block(struct ssa_builder *ssa) {
    if (ssa->num_blocks >= 64) return -1;
    int idx = ssa->num_blocks;
    ssa->blocks[idx].id = idx;
    ssa->blocks[idx].num_instructions = 0;
    ssa->blocks[idx].num_phis = 0;
    ssa->blocks[idx].num_successors = 0;
    ssa->blocks[idx].num_predecessors = 0;
    ssa->blocks[idx].num_dom_frontier = 0;
    ssa->blocks[idx].idom = -1;
    ssa->num_blocks++;
    return idx;
}

void ssa_add_edge(struct ssa_builder *ssa, int from, int to) {
    if (from < 0 || to < 0 || from >= ssa->num_blocks || to >= ssa->num_blocks) return;
    struct basic_block *src = &ssa->blocks[from];
    struct basic_block *dst = &ssa->blocks[to];
    if (src->num_successors < 4) {
        src->successors[src->num_successors] = to;
        src->num_successors++;
    }
    if (dst->num_predecessors < 4) {
        dst->predecessors[dst->num_predecessors] = from;
        dst->num_predecessors++;
    }
}

void ssa_insert_phi(struct ssa_builder *ssa, int block_id, int var) {
    if (block_id < 0 || block_id >= ssa->num_blocks) return;
    struct basic_block *blk = &ssa->blocks[block_id];
    if (blk->num_phis >= 8) return;
    int i;
    for (i = 0; i < blk->num_phis; i++) {
        if (blk->phis[i].dest_var == var) return;
    }
    int idx = blk->num_phis;
    blk->phis[idx].dest_var = var;
    blk->phis[idx].num_operands = 0;
    blk->num_phis++;
}

void ssa_add_phi_operand(struct ssa_builder *ssa, int block_id, int var, int value, int from_block) {
    if (block_id < 0 || block_id >= ssa->num_blocks) return;
    struct basic_block *blk = &ssa->blocks[block_id];
    int i;
    for (i = 0; i < blk->num_phis; i++) {
        if (blk->phis[i].dest_var == var && blk->phis[i].num_operands < 8) {
            int idx = blk->phis[i].num_operands;
            blk->phis[i].operands[idx].value = value;
            blk->phis[i].operands[idx].block_id = from_block;
            blk->phis[i].num_operands++;
            return;
        }
    }
}

int ssa_count_phis(const struct ssa_builder *ssa) {
    int total = 0;
    int i;
    for (i = 0; i < ssa->num_blocks; i++) {
        total += ssa->blocks[i].num_phis;
    }
    return total;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C383: SSA phi node insertion - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C383: empty output");
    assert!(
        code.contains("fn ssa_insert_phi"),
        "C383: Should contain ssa_insert_phi function"
    );
}

#[test]
fn c384_mark_and_sweep_garbage_collector() {
    let c_code = r#"
enum gc_obj_type {
    GC_INT = 0,
    GC_PAIR,
    GC_STRING
};

struct gc_object {
    enum gc_obj_type type;
    int marked;
    int value;
    int car;
    int cdr;
    int next;
};

struct gc_heap {
    struct gc_object objects[512];
    int count;
    int free_list;
    int roots[32];
    int num_roots;
    int collections;
    int freed_count;
};

void gc_init(struct gc_heap *heap) {
    heap->count = 0;
    heap->free_list = -1;
    heap->num_roots = 0;
    heap->collections = 0;
    heap->freed_count = 0;
}

int gc_alloc(struct gc_heap *heap, enum gc_obj_type type) {
    int idx;
    if (heap->free_list >= 0) {
        idx = heap->free_list;
        heap->free_list = heap->objects[idx].next;
    } else {
        if (heap->count >= 512) return -1;
        idx = heap->count;
        heap->count++;
    }
    heap->objects[idx].type = type;
    heap->objects[idx].marked = 0;
    heap->objects[idx].value = 0;
    heap->objects[idx].car = -1;
    heap->objects[idx].cdr = -1;
    heap->objects[idx].next = -1;
    return idx;
}

int gc_make_int(struct gc_heap *heap, int val) {
    int idx = gc_alloc(heap, GC_INT);
    if (idx >= 0) heap->objects[idx].value = val;
    return idx;
}

int gc_make_pair(struct gc_heap *heap, int car, int cdr) {
    int idx = gc_alloc(heap, GC_PAIR);
    if (idx >= 0) {
        heap->objects[idx].car = car;
        heap->objects[idx].cdr = cdr;
    }
    return idx;
}

void gc_add_root(struct gc_heap *heap, int obj) {
    if (heap->num_roots < 32) {
        heap->roots[heap->num_roots] = obj;
        heap->num_roots++;
    }
}

static void gc_mark(struct gc_heap *heap, int idx) {
    if (idx < 0 || idx >= heap->count) return;
    if (heap->objects[idx].marked) return;
    heap->objects[idx].marked = 1;
    if (heap->objects[idx].type == GC_PAIR) {
        gc_mark(heap, heap->objects[idx].car);
        gc_mark(heap, heap->objects[idx].cdr);
    }
}

void gc_collect(struct gc_heap *heap) {
    int i;
    for (i = 0; i < heap->count; i++) {
        heap->objects[i].marked = 0;
    }
    for (i = 0; i < heap->num_roots; i++) {
        gc_mark(heap, heap->roots[i]);
    }
    for (i = 0; i < heap->count; i++) {
        if (!heap->objects[i].marked) {
            heap->objects[i].next = heap->free_list;
            heap->free_list = i;
            heap->freed_count++;
        }
    }
    heap->collections++;
}

int gc_live_count(const struct gc_heap *heap) {
    int count = 0;
    int i;
    for (i = 0; i < heap->count; i++) {
        if (heap->objects[i].marked) count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C384: Mark-and-sweep garbage collector - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C384: empty output");
    assert!(
        code.contains("fn gc_collect"),
        "C384: Should contain gc_collect function"
    );
}

#[test]
fn c385_copying_compacting_gc_semispace() {
    let c_code = r#"
struct gc_obj {
    int tag;
    int size;
    int forwarded;
    int forward_addr;
    int fields[4];
};

struct semispace_gc {
    struct gc_obj from_space[256];
    struct gc_obj to_space[256];
    int from_count;
    int to_count;
    int capacity;
    int total_copies;
};

void semi_gc_init(struct semispace_gc *gc) {
    gc->from_count = 0;
    gc->to_count = 0;
    gc->capacity = 256;
    gc->total_copies = 0;
}

int semi_gc_alloc(struct semispace_gc *gc, int tag, int size) {
    if (gc->from_count >= gc->capacity) return -1;
    int idx = gc->from_count;
    gc->from_space[idx].tag = tag;
    gc->from_space[idx].size = size;
    gc->from_space[idx].forwarded = 0;
    gc->from_space[idx].forward_addr = -1;
    int i;
    for (i = 0; i < 4; i++) gc->from_space[idx].fields[i] = -1;
    gc->from_count++;
    return idx;
}

static int copy_object(struct semispace_gc *gc, int idx) {
    if (idx < 0 || idx >= gc->from_count) return -1;
    if (gc->from_space[idx].forwarded) {
        return gc->from_space[idx].forward_addr;
    }
    if (gc->to_count >= gc->capacity) return -1;
    int new_idx = gc->to_count;
    gc->to_space[new_idx] = gc->from_space[idx];
    gc->to_space[new_idx].forwarded = 0;
    gc->to_space[new_idx].forward_addr = -1;
    gc->from_space[idx].forwarded = 1;
    gc->from_space[idx].forward_addr = new_idx;
    gc->to_count++;
    gc->total_copies++;
    return new_idx;
}

void semi_gc_collect(struct semispace_gc *gc, int *roots, int num_roots) {
    gc->to_count = 0;
    int i;
    for (i = 0; i < num_roots; i++) {
        roots[i] = copy_object(gc, roots[i]);
    }
    int scan = 0;
    while (scan < gc->to_count) {
        int j;
        for (j = 0; j < 4; j++) {
            int ref = gc->to_space[scan].fields[j];
            if (ref >= 0) {
                gc->to_space[scan].fields[j] = copy_object(gc, ref);
            }
        }
        scan++;
    }
    for (i = 0; i < gc->to_count; i++) {
        gc->from_space[i] = gc->to_space[i];
    }
    gc->from_count = gc->to_count;
}

int semi_gc_used(const struct semispace_gc *gc) {
    return gc->from_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C385: Copying/compacting GC semispace - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C385: empty output");
    assert!(
        code.contains("fn semi_gc_collect"),
        "C385: Should contain semi_gc_collect function"
    );
}

// ============================================================================
// C386-C390: Optimization (Refcounting, Instruction Encoding, Const Fold, DCE, Closures)
// ============================================================================

#[test]
fn c386_reference_counting_with_cycle_detection() {
    let c_code = r#"
struct rc_object {
    int ref_count;
    int type_tag;
    int value;
    int refs[4];
    int num_refs;
    int color;
    int buffered;
};

struct rc_collector {
    struct rc_object objects[256];
    int count;
    int buffer[64];
    int buffer_count;
};

void rc_init(struct rc_collector *rc) {
    rc->count = 0;
    rc->buffer_count = 0;
}

int rc_alloc(struct rc_collector *rc, int type_tag, int value) {
    if (rc->count >= 256) return -1;
    int idx = rc->count;
    rc->objects[idx].ref_count = 1;
    rc->objects[idx].type_tag = type_tag;
    rc->objects[idx].value = value;
    rc->objects[idx].num_refs = 0;
    rc->objects[idx].color = 0;
    rc->objects[idx].buffered = 0;
    int i;
    for (i = 0; i < 4; i++) rc->objects[idx].refs[i] = -1;
    rc->count++;
    return idx;
}

void rc_add_ref(struct rc_collector *rc, int from, int to) {
    if (from < 0 || from >= rc->count) return;
    if (to < 0 || to >= rc->count) return;
    struct rc_object *obj = &rc->objects[from];
    if (obj->num_refs >= 4) return;
    obj->refs[obj->num_refs] = to;
    obj->num_refs++;
    rc->objects[to].ref_count++;
}

void rc_release(struct rc_collector *rc, int idx) {
    if (idx < 0 || idx >= rc->count) return;
    rc->objects[idx].ref_count--;
    if (rc->objects[idx].ref_count == 0) {
        int i;
        for (i = 0; i < rc->objects[idx].num_refs; i++) {
            rc_release(rc, rc->objects[idx].refs[i]);
        }
    } else if (!rc->objects[idx].buffered && rc->buffer_count < 64) {
        rc->objects[idx].buffered = 1;
        rc->buffer[rc->buffer_count] = idx;
        rc->buffer_count++;
    }
}

static void mark_gray(struct rc_collector *rc, int idx) {
    if (idx < 0 || idx >= rc->count) return;
    if (rc->objects[idx].color == 1) return;
    rc->objects[idx].color = 1;
    int i;
    for (i = 0; i < rc->objects[idx].num_refs; i++) {
        int child = rc->objects[idx].refs[i];
        if (child >= 0 && child < rc->count) {
            rc->objects[child].ref_count--;
            mark_gray(rc, child);
        }
    }
}

static void scan_roots(struct rc_collector *rc, int idx) {
    if (idx < 0 || idx >= rc->count) return;
    if (rc->objects[idx].color != 1) return;
    if (rc->objects[idx].ref_count > 0) {
        rc->objects[idx].color = 0;
    } else {
        rc->objects[idx].color = 2;
        int i;
        for (i = 0; i < rc->objects[idx].num_refs; i++) {
            scan_roots(rc, rc->objects[idx].refs[i]);
        }
    }
}

void rc_collect_cycles(struct rc_collector *rc) {
    int i;
    for (i = 0; i < rc->buffer_count; i++) {
        mark_gray(rc, rc->buffer[i]);
    }
    for (i = 0; i < rc->buffer_count; i++) {
        scan_roots(rc, rc->buffer[i]);
    }
    rc->buffer_count = 0;
}

int rc_get_refcount(const struct rc_collector *rc, int idx) {
    if (idx < 0 || idx >= rc->count) return -1;
    return rc->objects[idx].ref_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C386: Reference counting with cycle detection - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C386: empty output");
    assert!(
        code.contains("fn rc_collect_cycles"),
        "C386: Should contain rc_collect_cycles function"
    );
}

#[test]
fn c387_instruction_encoding_decoding() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

struct instruction {
    uint8_t opcode;
    uint8_t rd;
    uint8_t rs1;
    uint8_t rs2;
    int immediate;
};

uint32_t encode_r_type(uint8_t opcode, uint8_t rd, uint8_t rs1, uint8_t rs2) {
    uint32_t instr = 0;
    instr |= ((uint32_t)opcode & 0x7F);
    instr |= ((uint32_t)rd & 0x1F) << 7;
    instr |= ((uint32_t)rs1 & 0x1F) << 15;
    instr |= ((uint32_t)rs2 & 0x1F) << 20;
    return instr;
}

uint32_t encode_i_type(uint8_t opcode, uint8_t rd, uint8_t rs1, int imm) {
    uint32_t instr = 0;
    instr |= ((uint32_t)opcode & 0x7F);
    instr |= ((uint32_t)rd & 0x1F) << 7;
    instr |= ((uint32_t)rs1 & 0x1F) << 15;
    instr |= ((uint32_t)(imm & 0xFFF)) << 20;
    return instr;
}

void decode_instruction(uint32_t encoded, struct instruction *instr) {
    instr->opcode = (uint8_t)(encoded & 0x7F);
    instr->rd = (uint8_t)((encoded >> 7) & 0x1F);
    instr->rs1 = (uint8_t)((encoded >> 15) & 0x1F);
    instr->rs2 = (uint8_t)((encoded >> 20) & 0x1F);
    instr->immediate = (int)((encoded >> 20) & 0xFFF);
    if (instr->immediate & 0x800) {
        instr->immediate |= 0xFFFFF000;
    }
}

int instruction_uses_reg(const struct instruction *instr, uint8_t reg) {
    return instr->rs1 == reg || instr->rs2 == reg;
}

int instruction_defines_reg(const struct instruction *instr) {
    return instr->rd;
}

uint32_t encode_nop(void) {
    return encode_r_type(0, 0, 0, 0);
}

int is_nop(uint32_t encoded) {
    return encoded == 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C387: Instruction encoding/decoding - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C387: empty output");
    assert!(
        code.contains("fn decode_instruction"),
        "C387: Should contain decode_instruction function"
    );
}

#[test]
fn c388_constant_folding_optimizer() {
    let c_code = r#"
enum ir_op {
    IR_CONST = 0,
    IR_ADD,
    IR_SUB,
    IR_MUL,
    IR_DIV,
    IR_NEG,
    IR_LOAD,
    IR_STORE,
    IR_PHI,
    IR_NOP
};

struct ir_instr {
    enum ir_op op;
    int dest;
    int src1;
    int src2;
    int const_val;
    int is_const;
};

struct optimizer {
    struct ir_instr instrs[256];
    int count;
    int folded;
};

void opt_init(struct optimizer *opt) {
    opt->count = 0;
    opt->folded = 0;
}

int opt_add_instr(struct optimizer *opt, enum ir_op op, int dest, int s1, int s2, int cval) {
    if (opt->count >= 256) return -1;
    int idx = opt->count;
    opt->instrs[idx].op = op;
    opt->instrs[idx].dest = dest;
    opt->instrs[idx].src1 = s1;
    opt->instrs[idx].src2 = s2;
    opt->instrs[idx].const_val = cval;
    opt->instrs[idx].is_const = (op == IR_CONST) ? 1 : 0;
    opt->count++;
    return idx;
}

static int find_instr_by_dest(const struct optimizer *opt, int dest) {
    int i;
    for (i = opt->count - 1; i >= 0; i--) {
        if (opt->instrs[i].dest == dest && opt->instrs[i].op != IR_NOP) return i;
    }
    return -1;
}

void opt_fold_constants(struct optimizer *opt) {
    int changed = 1;
    while (changed) {
        changed = 0;
        int i;
        for (i = 0; i < opt->count; i++) {
            struct ir_instr *instr = &opt->instrs[i];
            if (instr->op == IR_NOP || instr->is_const) continue;

            if (instr->op >= IR_ADD && instr->op <= IR_DIV) {
                int s1 = find_instr_by_dest(opt, instr->src1);
                int s2 = find_instr_by_dest(opt, instr->src2);
                if (s1 >= 0 && s2 >= 0 &&
                    opt->instrs[s1].is_const && opt->instrs[s2].is_const) {
                    int a = opt->instrs[s1].const_val;
                    int b = opt->instrs[s2].const_val;
                    int result = 0;
                    switch (instr->op) {
                        case IR_ADD: result = a + b; break;
                        case IR_SUB: result = a - b; break;
                        case IR_MUL: result = a * b; break;
                        case IR_DIV: result = (b != 0) ? a / b : 0; break;
                        default: break;
                    }
                    instr->op = IR_CONST;
                    instr->const_val = result;
                    instr->is_const = 1;
                    opt->folded++;
                    changed = 1;
                }
            }

            if (instr->op == IR_NEG) {
                int s1 = find_instr_by_dest(opt, instr->src1);
                if (s1 >= 0 && opt->instrs[s1].is_const) {
                    instr->op = IR_CONST;
                    instr->const_val = -opt->instrs[s1].const_val;
                    instr->is_const = 1;
                    opt->folded++;
                    changed = 1;
                }
            }
        }
    }
}

int opt_folded_count(const struct optimizer *opt) {
    return opt->folded;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C388: Constant folding optimizer - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C388: empty output");
    assert!(
        code.contains("fn opt_fold_constants"),
        "C388: Should contain opt_fold_constants function"
    );
}

#[test]
fn c389_dead_code_elimination() {
    let c_code = r#"
struct dce_instr {
    int opcode;
    int dest;
    int src1;
    int src2;
    int is_dead;
    int has_side_effect;
};

struct dce_pass {
    struct dce_instr instrs[256];
    int count;
    int eliminated;
};

void dce_init(struct dce_pass *dce) {
    dce->count = 0;
    dce->eliminated = 0;
}

int dce_add_instr(struct dce_pass *dce, int opcode, int dest, int s1, int s2, int side_effect) {
    if (dce->count >= 256) return -1;
    int idx = dce->count;
    dce->instrs[idx].opcode = opcode;
    dce->instrs[idx].dest = dest;
    dce->instrs[idx].src1 = s1;
    dce->instrs[idx].src2 = s2;
    dce->instrs[idx].is_dead = 0;
    dce->instrs[idx].has_side_effect = side_effect;
    dce->count++;
    return idx;
}

static int is_used(const struct dce_pass *dce, int def_idx) {
    int dest = dce->instrs[def_idx].dest;
    int i;
    for (i = def_idx + 1; i < dce->count; i++) {
        if (dce->instrs[i].is_dead) continue;
        if (dce->instrs[i].src1 == dest || dce->instrs[i].src2 == dest) {
            return 1;
        }
    }
    return 0;
}

void dce_eliminate(struct dce_pass *dce) {
    int changed = 1;
    while (changed) {
        changed = 0;
        int i;
        for (i = dce->count - 1; i >= 0; i--) {
            if (dce->instrs[i].is_dead) continue;
            if (dce->instrs[i].has_side_effect) continue;
            if (!is_used(dce, i)) {
                dce->instrs[i].is_dead = 1;
                dce->eliminated++;
                changed = 1;
            }
        }
    }
}

int dce_live_count(const struct dce_pass *dce) {
    int live = 0;
    int i;
    for (i = 0; i < dce->count; i++) {
        if (!dce->instrs[i].is_dead) live++;
    }
    return live;
}

int dce_eliminated_count(const struct dce_pass *dce) {
    return dce->eliminated;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C389: Dead code elimination - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C389: empty output");
    assert!(
        code.contains("fn dce_eliminate"),
        "C389: Should contain dce_eliminate function"
    );
}

#[test]
fn c390_closure_environment_capture() {
    let c_code = r#"
struct captured_var {
    int index;
    int value;
    int is_mutable;
};

struct closure_env {
    struct captured_var captures[16];
    int num_captures;
    int parent_env;
};

struct closure {
    int func_id;
    int env_id;
    int arity;
};

struct closure_runtime {
    struct closure_env envs[64];
    int env_count;
    struct closure closures[64];
    int closure_count;
};

void closure_rt_init(struct closure_runtime *rt) {
    rt->env_count = 0;
    rt->closure_count = 0;
}

int closure_create_env(struct closure_runtime *rt, int parent) {
    if (rt->env_count >= 64) return -1;
    int idx = rt->env_count;
    rt->envs[idx].num_captures = 0;
    rt->envs[idx].parent_env = parent;
    rt->env_count++;
    return idx;
}

int closure_env_capture(struct closure_runtime *rt, int env_id, int var_index, int value, int is_mut) {
    if (env_id < 0 || env_id >= rt->env_count) return -1;
    struct closure_env *env = &rt->envs[env_id];
    if (env->num_captures >= 16) return -2;
    int idx = env->num_captures;
    env->captures[idx].index = var_index;
    env->captures[idx].value = value;
    env->captures[idx].is_mutable = is_mut;
    env->num_captures++;
    return idx;
}

int closure_env_get(const struct closure_runtime *rt, int env_id, int var_index) {
    if (env_id < 0 || env_id >= rt->env_count) return 0;
    const struct closure_env *env = &rt->envs[env_id];
    int i;
    for (i = 0; i < env->num_captures; i++) {
        if (env->captures[i].index == var_index) {
            return env->captures[i].value;
        }
    }
    if (env->parent_env >= 0) {
        return closure_env_get(rt, env->parent_env, var_index);
    }
    return 0;
}

int closure_env_set(struct closure_runtime *rt, int env_id, int var_index, int value) {
    if (env_id < 0 || env_id >= rt->env_count) return -1;
    struct closure_env *env = &rt->envs[env_id];
    int i;
    for (i = 0; i < env->num_captures; i++) {
        if (env->captures[i].index == var_index && env->captures[i].is_mutable) {
            env->captures[i].value = value;
            return 0;
        }
    }
    return -1;
}

int closure_create(struct closure_runtime *rt, int func_id, int env_id, int arity) {
    if (rt->closure_count >= 64) return -1;
    int idx = rt->closure_count;
    rt->closures[idx].func_id = func_id;
    rt->closures[idx].env_id = env_id;
    rt->closures[idx].arity = arity;
    rt->closure_count++;
    return idx;
}

int closure_get_env(const struct closure_runtime *rt, int closure_id) {
    if (closure_id < 0 || closure_id >= rt->closure_count) return -1;
    return rt->closures[closure_id].env_id;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C390: Closure environment capture - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C390: empty output");
    assert!(
        code.contains("fn closure_env_get"),
        "C390: Should contain closure_env_get function"
    );
}

// ============================================================================
// C391-C395: Advanced (Pattern Match, TCO, Exceptions, Interning, Dispatch)
// ============================================================================

#[test]
fn c391_pattern_matching_compiler() {
    let c_code = r#"
enum pat_kind {
    PAT_WILDCARD = 0,
    PAT_LITERAL,
    PAT_VARIABLE,
    PAT_CONSTRUCTOR,
    PAT_TUPLE
};

struct pattern {
    enum pat_kind kind;
    int value;
    int tag;
    int sub_patterns[4];
    int num_sub;
};

struct match_arm {
    struct pattern pat;
    int body_label;
    int guard_expr;
};

struct match_compiler {
    struct match_arm arms[32];
    int num_arms;
    int output_labels[32];
    int num_outputs;
    int default_label;
};

void match_init(struct match_compiler *mc, int default_lbl) {
    mc->num_arms = 0;
    mc->num_outputs = 0;
    mc->default_label = default_lbl;
}

int match_add_arm(struct match_compiler *mc, enum pat_kind kind, int value, int tag, int body_label) {
    if (mc->num_arms >= 32) return -1;
    int idx = mc->num_arms;
    mc->arms[idx].pat.kind = kind;
    mc->arms[idx].pat.value = value;
    mc->arms[idx].pat.tag = tag;
    mc->arms[idx].pat.num_sub = 0;
    mc->arms[idx].body_label = body_label;
    mc->arms[idx].guard_expr = -1;
    mc->num_arms++;
    return idx;
}

static int pattern_matches(const struct pattern *pat, int value, int tag) {
    switch (pat->kind) {
        case PAT_WILDCARD:
            return 1;
        case PAT_LITERAL:
            return pat->value == value;
        case PAT_VARIABLE:
            return 1;
        case PAT_CONSTRUCTOR:
            return pat->tag == tag;
        case PAT_TUPLE:
            return 1;
        default:
            return 0;
    }
}

int match_execute(const struct match_compiler *mc, int value, int tag) {
    int i;
    for (i = 0; i < mc->num_arms; i++) {
        if (pattern_matches(&mc->arms[i].pat, value, tag)) {
            return mc->arms[i].body_label;
        }
    }
    return mc->default_label;
}

int match_is_exhaustive(const struct match_compiler *mc) {
    int i;
    for (i = 0; i < mc->num_arms; i++) {
        if (mc->arms[i].pat.kind == PAT_WILDCARD ||
            mc->arms[i].pat.kind == PAT_VARIABLE) {
            return 1;
        }
    }
    return 0;
}

int match_count_wildcards(const struct match_compiler *mc) {
    int count = 0;
    int i;
    for (i = 0; i < mc->num_arms; i++) {
        if (mc->arms[i].pat.kind == PAT_WILDCARD) count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C391: Pattern matching compiler - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C391: empty output");
    assert!(
        code.contains("fn match_execute"),
        "C391: Should contain match_execute function"
    );
}

#[test]
fn c392_tail_call_optimization_detector() {
    let c_code = r#"
enum call_type {
    CALL_NORMAL = 0,
    CALL_TAIL,
    CALL_SELF_TAIL,
    CALL_NOT_A_CALL
};

struct call_site {
    int func_id;
    int callee_id;
    int position;
    enum call_type type;
    int is_in_return_position;
    int num_args;
};

struct tco_analyzer {
    struct call_site sites[128];
    int num_sites;
    int tail_calls_found;
    int self_tail_calls_found;
};

void tco_init(struct tco_analyzer *tco) {
    tco->num_sites = 0;
    tco->tail_calls_found = 0;
    tco->self_tail_calls_found = 0;
}

int tco_add_call(struct tco_analyzer *tco, int func_id, int callee_id, int pos, int in_return, int nargs) {
    if (tco->num_sites >= 128) return -1;
    int idx = tco->num_sites;
    tco->sites[idx].func_id = func_id;
    tco->sites[idx].callee_id = callee_id;
    tco->sites[idx].position = pos;
    tco->sites[idx].is_in_return_position = in_return;
    tco->sites[idx].num_args = nargs;
    tco->sites[idx].type = CALL_NORMAL;
    tco->num_sites++;
    return idx;
}

void tco_analyze(struct tco_analyzer *tco) {
    int i;
    for (i = 0; i < tco->num_sites; i++) {
        struct call_site *site = &tco->sites[i];
        if (site->is_in_return_position) {
            if (site->func_id == site->callee_id) {
                site->type = CALL_SELF_TAIL;
                tco->self_tail_calls_found++;
                tco->tail_calls_found++;
            } else {
                site->type = CALL_TAIL;
                tco->tail_calls_found++;
            }
        } else {
            site->type = CALL_NORMAL;
        }
    }
}

int tco_can_optimize(const struct tco_analyzer *tco, int site_idx) {
    if (site_idx < 0 || site_idx >= tco->num_sites) return 0;
    return tco->sites[site_idx].type == CALL_SELF_TAIL;
}

int tco_count_optimizable(const struct tco_analyzer *tco) {
    int count = 0;
    int i;
    for (i = 0; i < tco->num_sites; i++) {
        if (tco->sites[i].type == CALL_SELF_TAIL) count++;
    }
    return count;
}

int tco_tail_call_ratio(const struct tco_analyzer *tco) {
    if (tco->num_sites == 0) return 0;
    return (tco->tail_calls_found * 100) / tco->num_sites;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C392: Tail call optimization detector - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C392: empty output");
    assert!(
        code.contains("fn tco_analyze"),
        "C392: Should contain tco_analyze function"
    );
}

#[test]
fn c393_exception_handling_setjmp_longjmp_table() {
    let c_code = r#"
enum exception_type {
    EX_NONE = 0,
    EX_RUNTIME,
    EX_TYPE_ERROR,
    EX_DIVIDE_BY_ZERO,
    EX_NULL_POINTER,
    EX_OUT_OF_BOUNDS,
    EX_STACK_OVERFLOW
};

struct exception_frame {
    int frame_id;
    int handler_label;
    int finally_label;
    enum exception_type catch_type;
    int parent_frame;
    int is_active;
};

struct exception_state {
    enum exception_type current_exception;
    int exception_value;
    int exception_line;
};

struct exception_table {
    struct exception_frame frames[32];
    int num_frames;
    int current_frame;
    struct exception_state state;
};

void exc_init(struct exception_table *et) {
    et->num_frames = 0;
    et->current_frame = -1;
    et->state.current_exception = EX_NONE;
    et->state.exception_value = 0;
    et->state.exception_line = 0;
}

int exc_push_frame(struct exception_table *et, int handler_label, int finally_label, enum exception_type catch_type) {
    if (et->num_frames >= 32) return -1;
    int idx = et->num_frames;
    et->frames[idx].frame_id = idx;
    et->frames[idx].handler_label = handler_label;
    et->frames[idx].finally_label = finally_label;
    et->frames[idx].catch_type = catch_type;
    et->frames[idx].parent_frame = et->current_frame;
    et->frames[idx].is_active = 1;
    et->num_frames++;
    et->current_frame = idx;
    return idx;
}

int exc_pop_frame(struct exception_table *et) {
    if (et->current_frame < 0) return -1;
    et->frames[et->current_frame].is_active = 0;
    int parent = et->frames[et->current_frame].parent_frame;
    et->current_frame = parent;
    return parent;
}

int exc_throw(struct exception_table *et, enum exception_type type, int value, int line) {
    et->state.current_exception = type;
    et->state.exception_value = value;
    et->state.exception_line = line;

    int frame = et->current_frame;
    while (frame >= 0) {
        if (et->frames[frame].is_active) {
            if (et->frames[frame].catch_type == type ||
                et->frames[frame].catch_type == EX_RUNTIME) {
                return et->frames[frame].handler_label;
            }
        }
        frame = et->frames[frame].parent_frame;
    }
    return -1;
}

void exc_clear(struct exception_table *et) {
    et->state.current_exception = EX_NONE;
    et->state.exception_value = 0;
    et->state.exception_line = 0;
}

int exc_is_pending(const struct exception_table *et) {
    return et->state.current_exception != EX_NONE;
}

int exc_depth(const struct exception_table *et) {
    int depth = 0;
    int frame = et->current_frame;
    while (frame >= 0) {
        depth++;
        frame = et->frames[frame].parent_frame;
    }
    return depth;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C393: Exception handling table - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C393: empty output");
    assert!(
        code.contains("fn exc_throw"),
        "C393: Should contain exc_throw function"
    );
}

#[test]
fn c394_string_interning_table() {
    let c_code = r#"
struct intern_entry {
    char data[128];
    int length;
    unsigned int hash;
    int next;
};

struct intern_table {
    struct intern_entry entries[256];
    int count;
    int buckets[64];
    int num_buckets;
};

static unsigned int hash_string(const char *str, int len) {
    unsigned int h = 2166136261u;
    int i;
    for (i = 0; i < len; i++) {
        h ^= (unsigned int)(unsigned char)str[i];
        h *= 16777619u;
    }
    return h;
}

void intern_init(struct intern_table *table) {
    table->count = 0;
    table->num_buckets = 64;
    int i;
    for (i = 0; i < 64; i++) table->buckets[i] = -1;
}

static int str_equal(const char *a, int alen, const char *b, int blen) {
    if (alen != blen) return 0;
    int i;
    for (i = 0; i < alen; i++) {
        if (a[i] != b[i]) return 0;
    }
    return 1;
}

int intern_lookup(const struct intern_table *table, const char *str, int len) {
    unsigned int h = hash_string(str, len);
    int bucket = (int)(h % (unsigned int)table->num_buckets);
    int idx = table->buckets[bucket];
    while (idx >= 0) {
        if (table->entries[idx].hash == h &&
            str_equal(table->entries[idx].data, table->entries[idx].length, str, len)) {
            return idx;
        }
        idx = table->entries[idx].next;
    }
    return -1;
}

int intern_insert(struct intern_table *table, const char *str, int len) {
    int existing = intern_lookup(table, str, len);
    if (existing >= 0) return existing;
    if (table->count >= 256) return -1;
    if (len >= 128) return -2;

    int idx = table->count;
    unsigned int h = hash_string(str, len);
    int bucket = (int)(h % (unsigned int)table->num_buckets);

    int i;
    for (i = 0; i < len; i++) {
        table->entries[idx].data[i] = str[i];
    }
    table->entries[idx].data[len] = '\0';
    table->entries[idx].length = len;
    table->entries[idx].hash = h;
    table->entries[idx].next = table->buckets[bucket];
    table->buckets[bucket] = idx;
    table->count++;
    return idx;
}

int intern_count(const struct intern_table *table) {
    return table->count;
}

int intern_same(const struct intern_table *table, int a, int b) {
    return a == b;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C394: String interning table - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C394: empty output");
    assert!(
        code.contains("fn intern_insert"),
        "C394: Should contain intern_insert function"
    );
}

#[test]
fn c395_opcode_dispatch_table() {
    let c_code = r#"
typedef unsigned char uint8_t;

enum vm_op {
    VM_NOP = 0,
    VM_PUSH,
    VM_POP,
    VM_ADD,
    VM_SUB,
    VM_MUL,
    VM_DIV,
    VM_AND,
    VM_OR,
    VM_NOT,
    VM_CMP,
    VM_JMP,
    VM_JEQ,
    VM_JNE,
    VM_CALL,
    VM_RET,
    VM_HALT,
    VM_NUM_OPS
};

struct dispatch_entry {
    enum vm_op opcode;
    int operand_count;
    int stack_effect;
    int has_jump_target;
    int is_terminator;
};

struct dispatch_table {
    struct dispatch_entry entries[32];
    int count;
};

void dispatch_init(struct dispatch_table *dt) {
    dt->count = 0;
}

int dispatch_register(struct dispatch_table *dt, enum vm_op op, int operands, int stack_eff, int jump, int term) {
    if (dt->count >= 32) return -1;
    int idx = dt->count;
    dt->entries[idx].opcode = op;
    dt->entries[idx].operand_count = operands;
    dt->entries[idx].stack_effect = stack_eff;
    dt->entries[idx].has_jump_target = jump;
    dt->entries[idx].is_terminator = term;
    dt->count++;
    return idx;
}

int dispatch_lookup(const struct dispatch_table *dt, enum vm_op op) {
    int i;
    for (i = 0; i < dt->count; i++) {
        if (dt->entries[i].opcode == op) return i;
    }
    return -1;
}

int dispatch_stack_effect(const struct dispatch_table *dt, enum vm_op op) {
    int idx = dispatch_lookup(dt, op);
    if (idx < 0) return 0;
    return dt->entries[idx].stack_effect;
}

int dispatch_is_terminator(const struct dispatch_table *dt, enum vm_op op) {
    int idx = dispatch_lookup(dt, op);
    if (idx < 0) return 0;
    return dt->entries[idx].is_terminator;
}

int dispatch_operand_size(const struct dispatch_table *dt, enum vm_op op) {
    int idx = dispatch_lookup(dt, op);
    if (idx < 0) return 0;
    return dt->entries[idx].operand_count;
}

int dispatch_verify_stack(const struct dispatch_table *dt, const uint8_t *bytecode, int len) {
    int stack_depth = 0;
    int i = 0;
    while (i < len) {
        enum vm_op op = (enum vm_op)bytecode[i];
        int effect = dispatch_stack_effect(dt, op);
        stack_depth += effect;
        if (stack_depth < 0) return -1;
        i += 1 + dispatch_operand_size(dt, op);
        if (dispatch_is_terminator(dt, op)) break;
    }
    return stack_depth;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C395: Opcode dispatch table - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C395: empty output");
    assert!(
        code.contains("fn dispatch_verify_stack"),
        "C395: Should contain dispatch_verify_stack function"
    );
}

// ============================================================================
// C396-C400: Runtime (JIT Buffer, Debug Info, Modules, Peephole, Type Tags)
// ============================================================================

#[test]
fn c396_jit_code_buffer_allocation() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

struct code_buffer {
    uint8_t code[4096];
    int offset;
    int capacity;
    int patch_sites[64];
    int patch_targets[64];
    int num_patches;
};

void codebuf_init(struct code_buffer *buf) {
    buf->offset = 0;
    buf->capacity = 4096;
    buf->num_patches = 0;
}

int codebuf_emit_byte(struct code_buffer *buf, uint8_t byte) {
    if (buf->offset >= buf->capacity) return -1;
    buf->code[buf->offset] = byte;
    buf->offset++;
    return 0;
}

int codebuf_emit_u32(struct code_buffer *buf, uint32_t value) {
    if (buf->offset + 4 > buf->capacity) return -1;
    buf->code[buf->offset] = (uint8_t)(value & 0xFF);
    buf->code[buf->offset + 1] = (uint8_t)((value >> 8) & 0xFF);
    buf->code[buf->offset + 2] = (uint8_t)((value >> 16) & 0xFF);
    buf->code[buf->offset + 3] = (uint8_t)((value >> 24) & 0xFF);
    buf->offset += 4;
    return 0;
}

int codebuf_current_offset(const struct code_buffer *buf) {
    return buf->offset;
}

int codebuf_add_patch(struct code_buffer *buf, int site, int target) {
    if (buf->num_patches >= 64) return -1;
    buf->patch_sites[buf->num_patches] = site;
    buf->patch_targets[buf->num_patches] = target;
    buf->num_patches++;
    return 0;
}

void codebuf_resolve_patches(struct code_buffer *buf) {
    int i;
    for (i = 0; i < buf->num_patches; i++) {
        int site = buf->patch_sites[i];
        int target = buf->patch_targets[i];
        int rel = target - (site + 4);
        if (site + 4 <= buf->capacity) {
            buf->code[site] = (uint8_t)(rel & 0xFF);
            buf->code[site + 1] = (uint8_t)((rel >> 8) & 0xFF);
            buf->code[site + 2] = (uint8_t)((rel >> 16) & 0xFF);
            buf->code[site + 3] = (uint8_t)((rel >> 24) & 0xFF);
        }
    }
}

int codebuf_size(const struct code_buffer *buf) {
    return buf->offset;
}

int codebuf_remaining(const struct code_buffer *buf) {
    return buf->capacity - buf->offset;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C396: JIT code buffer allocation - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C396: empty output");
    assert!(
        code.contains("fn codebuf_resolve_patches"),
        "C396: Should contain codebuf_resolve_patches function"
    );
}

#[test]
fn c397_debug_info_source_map_generation() {
    let c_code = r#"
struct source_location {
    int file_id;
    int line;
    int column;
};

struct debug_entry {
    int code_offset;
    struct source_location loc;
    int scope_depth;
    int is_statement;
};

struct local_var_info {
    char name[32];
    int type_id;
    int stack_slot;
    int scope_start;
    int scope_end;
};

struct debug_info {
    struct debug_entry entries[256];
    int num_entries;
    struct local_var_info locals[64];
    int num_locals;
    char file_names[16][64];
    int num_files;
};

void debug_init(struct debug_info *di) {
    di->num_entries = 0;
    di->num_locals = 0;
    di->num_files = 0;
}

static void copy_str(char *dst, const char *src, int max) {
    int i;
    for (i = 0; i < max - 1 && src[i]; i++) dst[i] = src[i];
    dst[i] = '\0';
}

int debug_add_file(struct debug_info *di, const char *name) {
    if (di->num_files >= 16) return -1;
    int idx = di->num_files;
    copy_str(di->file_names[idx], name, 64);
    di->num_files++;
    return idx;
}

int debug_add_entry(struct debug_info *di, int offset, int file, int line, int col, int depth, int is_stmt) {
    if (di->num_entries >= 256) return -1;
    int idx = di->num_entries;
    di->entries[idx].code_offset = offset;
    di->entries[idx].loc.file_id = file;
    di->entries[idx].loc.line = line;
    di->entries[idx].loc.column = col;
    di->entries[idx].scope_depth = depth;
    di->entries[idx].is_statement = is_stmt;
    di->num_entries++;
    return idx;
}

int debug_add_local(struct debug_info *di, const char *name, int type_id, int slot, int start, int end) {
    if (di->num_locals >= 64) return -1;
    int idx = di->num_locals;
    copy_str(di->locals[idx].name, name, 32);
    di->locals[idx].type_id = type_id;
    di->locals[idx].stack_slot = slot;
    di->locals[idx].scope_start = start;
    di->locals[idx].scope_end = end;
    di->num_locals++;
    return idx;
}

int debug_find_location(const struct debug_info *di, int code_offset) {
    int best = -1;
    int i;
    for (i = 0; i < di->num_entries; i++) {
        if (di->entries[i].code_offset <= code_offset) {
            if (best < 0 || di->entries[i].code_offset > di->entries[best].code_offset) {
                best = i;
            }
        }
    }
    return best;
}

int debug_locals_in_scope(const struct debug_info *di, int code_offset) {
    int count = 0;
    int i;
    for (i = 0; i < di->num_locals; i++) {
        if (di->locals[i].scope_start <= code_offset &&
            di->locals[i].scope_end > code_offset) {
            count++;
        }
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C397: Debug info source map generation - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C397: empty output");
    assert!(
        code.contains("fn debug_find_location"),
        "C397: Should contain debug_find_location function"
    );
}

#[test]
fn c398_module_import_resolution() {
    let c_code = r#"
enum module_state {
    MOD_UNLOADED = 0,
    MOD_LOADING,
    MOD_LOADED,
    MOD_ERROR
};

struct module_export {
    char name[32];
    int symbol_id;
    int is_public;
};

struct module {
    char name[64];
    enum module_state state;
    struct module_export exports[32];
    int num_exports;
    int dependencies[16];
    int num_deps;
};

struct module_registry {
    struct module modules[32];
    int count;
};

static void mod_str_copy(char *dst, const char *src, int max) {
    int i;
    for (i = 0; i < max - 1 && src[i]; i++) dst[i] = src[i];
    dst[i] = '\0';
}

static int mod_str_eq(const char *a, const char *b) {
    int i;
    for (i = 0; a[i] && b[i]; i++) {
        if (a[i] != b[i]) return 0;
    }
    return a[i] == b[i];
}

void registry_init(struct module_registry *reg) {
    reg->count = 0;
}

int registry_add_module(struct module_registry *reg, const char *name) {
    if (reg->count >= 32) return -1;
    int idx = reg->count;
    mod_str_copy(reg->modules[idx].name, name, 64);
    reg->modules[idx].state = MOD_UNLOADED;
    reg->modules[idx].num_exports = 0;
    reg->modules[idx].num_deps = 0;
    reg->count++;
    return idx;
}

int registry_find_module(const struct module_registry *reg, const char *name) {
    int i;
    for (i = 0; i < reg->count; i++) {
        if (mod_str_eq(reg->modules[i].name, name)) return i;
    }
    return -1;
}

int module_add_export(struct module_registry *reg, int mod_id, const char *name, int sym_id, int is_public) {
    if (mod_id < 0 || mod_id >= reg->count) return -1;
    struct module *mod = &reg->modules[mod_id];
    if (mod->num_exports >= 32) return -2;
    int idx = mod->num_exports;
    mod_str_copy(mod->exports[idx].name, name, 32);
    mod->exports[idx].symbol_id = sym_id;
    mod->exports[idx].is_public = is_public;
    mod->num_exports++;
    return idx;
}

int module_add_dep(struct module_registry *reg, int mod_id, int dep_id) {
    if (mod_id < 0 || mod_id >= reg->count) return -1;
    struct module *mod = &reg->modules[mod_id];
    if (mod->num_deps >= 16) return -2;
    mod->dependencies[mod->num_deps] = dep_id;
    mod->num_deps++;
    return 0;
}

int module_resolve_import(const struct module_registry *reg, int mod_id, const char *name) {
    if (mod_id < 0 || mod_id >= reg->count) return -1;
    const struct module *mod = &reg->modules[mod_id];
    int i;
    for (i = 0; i < mod->num_deps; i++) {
        int dep = mod->dependencies[i];
        if (dep < 0 || dep >= reg->count) continue;
        const struct module *dep_mod = &reg->modules[dep];
        int j;
        for (j = 0; j < dep_mod->num_exports; j++) {
            if (dep_mod->exports[j].is_public && mod_str_eq(dep_mod->exports[j].name, name)) {
                return dep_mod->exports[j].symbol_id;
            }
        }
    }
    return -1;
}

static int has_cycle_helper(const struct module_registry *reg, int mod_id, int *visited, int *stack) {
    if (mod_id < 0 || mod_id >= reg->count) return 0;
    if (stack[mod_id]) return 1;
    if (visited[mod_id]) return 0;
    visited[mod_id] = 1;
    stack[mod_id] = 1;
    const struct module *mod = &reg->modules[mod_id];
    int i;
    for (i = 0; i < mod->num_deps; i++) {
        if (has_cycle_helper(reg, mod->dependencies[i], visited, stack)) return 1;
    }
    stack[mod_id] = 0;
    return 0;
}

int registry_has_cycle(const struct module_registry *reg) {
    int visited[32];
    int stack[32];
    int i;
    for (i = 0; i < 32; i++) { visited[i] = 0; stack[i] = 0; }
    for (i = 0; i < reg->count; i++) {
        if (has_cycle_helper(reg, i, visited, stack)) return 1;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C398: Module/import resolution - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C398: empty output");
    assert!(
        code.contains("fn module_resolve_import"),
        "C398: Should contain module_resolve_import function"
    );
}

#[test]
fn c399_peephole_optimizer() {
    let c_code = r#"
typedef unsigned char uint8_t;

enum peep_op {
    PEEP_NOP = 0,
    PEEP_LOAD,
    PEEP_STORE,
    PEEP_ADD,
    PEEP_SUB,
    PEEP_MUL,
    PEEP_PUSH,
    PEEP_POP,
    PEEP_MOV,
    PEEP_JUMP,
    PEEP_RET
};

struct peep_instr {
    enum peep_op op;
    int arg1;
    int arg2;
    int deleted;
};

struct peephole {
    struct peep_instr instrs[512];
    int count;
    int optimized;
};

void peep_init(struct peephole *ph) {
    ph->count = 0;
    ph->optimized = 0;
}

int peep_add(struct peephole *ph, enum peep_op op, int a1, int a2) {
    if (ph->count >= 512) return -1;
    int idx = ph->count;
    ph->instrs[idx].op = op;
    ph->instrs[idx].arg1 = a1;
    ph->instrs[idx].arg2 = a2;
    ph->instrs[idx].deleted = 0;
    ph->count++;
    return idx;
}

static int next_live(const struct peephole *ph, int idx) {
    int i;
    for (i = idx + 1; i < ph->count; i++) {
        if (!ph->instrs[i].deleted) return i;
    }
    return -1;
}

void peep_optimize(struct peephole *ph) {
    int changed = 1;
    while (changed) {
        changed = 0;
        int i;
        for (i = 0; i < ph->count; i++) {
            if (ph->instrs[i].deleted) continue;
            int j = next_live(ph, i);
            if (j < 0) continue;

            /* push X; pop X => NOP NOP */
            if (ph->instrs[i].op == PEEP_PUSH && ph->instrs[j].op == PEEP_POP &&
                ph->instrs[i].arg1 == ph->instrs[j].arg1) {
                ph->instrs[i].deleted = 1;
                ph->instrs[j].deleted = 1;
                ph->optimized += 2;
                changed = 1;
                continue;
            }

            /* store X; load X => store X (keep value on stack) */
            if (ph->instrs[i].op == PEEP_STORE && ph->instrs[j].op == PEEP_LOAD &&
                ph->instrs[i].arg1 == ph->instrs[j].arg1) {
                ph->instrs[j].deleted = 1;
                ph->optimized++;
                changed = 1;
                continue;
            }

            /* add 0 or sub 0 => NOP */
            if ((ph->instrs[i].op == PEEP_ADD || ph->instrs[i].op == PEEP_SUB) &&
                ph->instrs[i].arg2 == 0) {
                ph->instrs[i].deleted = 1;
                ph->optimized++;
                changed = 1;
                continue;
            }

            /* mul 1 => NOP */
            if (ph->instrs[i].op == PEEP_MUL && ph->instrs[i].arg2 == 1) {
                ph->instrs[i].deleted = 1;
                ph->optimized++;
                changed = 1;
                continue;
            }

            /* mov X X => NOP */
            if (ph->instrs[i].op == PEEP_MOV && ph->instrs[i].arg1 == ph->instrs[i].arg2) {
                ph->instrs[i].deleted = 1;
                ph->optimized++;
                changed = 1;
                continue;
            }
        }
    }
}

int peep_live_count(const struct peephole *ph) {
    int count = 0;
    int i;
    for (i = 0; i < ph->count; i++) {
        if (!ph->instrs[i].deleted) count++;
    }
    return count;
}

int peep_optimized_count(const struct peephole *ph) {
    return ph->optimized;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C399: Peephole optimizer - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C399: empty output");
    assert!(
        code.contains("fn peep_optimize"),
        "C399: Should contain peep_optimize function"
    );
}

#[test]
fn c400_runtime_type_tag_dispatch() {
    let c_code = r#"
enum value_tag {
    TAG_INT = 0,
    TAG_FLOAT,
    TAG_BOOL,
    TAG_STRING,
    TAG_ARRAY,
    TAG_OBJECT,
    TAG_NULL,
    TAG_FUNCTION
};

struct tagged_value {
    enum value_tag tag;
    int int_val;
    int float_bits;
    int bool_val;
    int str_id;
    int array_id;
    int obj_id;
    int func_id;
};

struct type_dispatch {
    struct tagged_value values[256];
    int count;
};

void td_init(struct type_dispatch *td) {
    td->count = 0;
}

int td_make_int(struct type_dispatch *td, int val) {
    if (td->count >= 256) return -1;
    int idx = td->count;
    td->values[idx].tag = TAG_INT;
    td->values[idx].int_val = val;
    td->count++;
    return idx;
}

int td_make_bool(struct type_dispatch *td, int val) {
    if (td->count >= 256) return -1;
    int idx = td->count;
    td->values[idx].tag = TAG_BOOL;
    td->values[idx].bool_val = val;
    td->count++;
    return idx;
}

int td_make_null(struct type_dispatch *td) {
    if (td->count >= 256) return -1;
    int idx = td->count;
    td->values[idx].tag = TAG_NULL;
    td->count++;
    return idx;
}

int td_make_func(struct type_dispatch *td, int func_id) {
    if (td->count >= 256) return -1;
    int idx = td->count;
    td->values[idx].tag = TAG_FUNCTION;
    td->values[idx].func_id = func_id;
    td->count++;
    return idx;
}

int td_is_truthy(const struct type_dispatch *td, int idx) {
    if (idx < 0 || idx >= td->count) return 0;
    const struct tagged_value *v = &td->values[idx];
    switch (v->tag) {
        case TAG_INT: return v->int_val != 0;
        case TAG_FLOAT: return v->float_bits != 0;
        case TAG_BOOL: return v->bool_val;
        case TAG_STRING: return v->str_id >= 0;
        case TAG_ARRAY: return v->array_id >= 0;
        case TAG_OBJECT: return v->obj_id >= 0;
        case TAG_NULL: return 0;
        case TAG_FUNCTION: return 1;
        default: return 0;
    }
}

int td_add(struct type_dispatch *td, int a, int b) {
    if (a < 0 || a >= td->count || b < 0 || b >= td->count) return -1;
    const struct tagged_value *va = &td->values[a];
    const struct tagged_value *vb = &td->values[b];

    if (va->tag == TAG_INT && vb->tag == TAG_INT) {
        return td_make_int(td, va->int_val + vb->int_val);
    }
    return -1;
}

int td_equal(const struct type_dispatch *td, int a, int b) {
    if (a < 0 || a >= td->count || b < 0 || b >= td->count) return 0;
    const struct tagged_value *va = &td->values[a];
    const struct tagged_value *vb = &td->values[b];
    if (va->tag != vb->tag) return 0;
    switch (va->tag) {
        case TAG_INT: return va->int_val == vb->int_val;
        case TAG_BOOL: return va->bool_val == vb->bool_val;
        case TAG_NULL: return 1;
        default: return 0;
    }
}

int td_type_name_len(const struct type_dispatch *td, int idx) {
    if (idx < 0 || idx >= td->count) return 0;
    switch (td->values[idx].tag) {
        case TAG_INT: return 3;
        case TAG_FLOAT: return 5;
        case TAG_BOOL: return 4;
        case TAG_STRING: return 6;
        case TAG_ARRAY: return 5;
        case TAG_OBJECT: return 6;
        case TAG_NULL: return 4;
        case TAG_FUNCTION: return 8;
        default: return 7;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C400: Runtime type tag dispatch - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C400: empty output");
    assert!(
        code.contains("fn td_is_truthy"),
        "C400: Should contain td_is_truthy function"
    );
}
