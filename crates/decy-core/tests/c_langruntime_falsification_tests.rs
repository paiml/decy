//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C751-C775: Language Runtime domain -- bytecode VMs, lexers, parsers,
//! type checkers, register allocators, garbage collectors, JIT infrastructure,
//! and other patterns found in language implementation codebases.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise language runtime patterns commonly found in
//! CPython, Lua, V8, Ruby MRI, and similar projects -- all expressed
//! as valid C99 without #include.
//!
//! Organization:
//! - C751-C755: Core VM (bytecode VM, lexer, parser, AST builder, symbol table)
//! - C756-C760: Analysis/Optimization (type checker, register alloc, const fold, DCE, TCO)
//! - C761-C763: Runtime (closures, mark-compact GC, string interning)
//! - C764-C768: Dispatch/Control (vtable, exception, coroutine, JIT buffer, profiler)
//! - C769-C775: Infrastructure (debug info, module loader, regex NFA, pattern match,
//!              instruction codec, peephole opt, stack frame layout)

// ============================================================================
// C751-C755: Core VM (Bytecode VM, Lexer, Parser, AST Builder, Symbol Table)
// ============================================================================

/// C751: Bytecode virtual machine (stack-based) with arithmetic operations
#[test]
fn c751_bytecode_virtual_machine_stack_based() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef int int32_t;

enum lr_opcode {
    LR_OP_NOP = 0,
    LR_OP_PUSH,
    LR_OP_POP,
    LR_OP_ADD,
    LR_OP_SUB,
    LR_OP_MUL,
    LR_OP_DIV,
    LR_OP_NEG,
    LR_OP_DUP,
    LR_OP_SWAP,
    LR_OP_LOAD,
    LR_OP_STORE,
    LR_OP_HALT
};

struct lr_vm {
    int32_t stack[256];
    int sp;
    int32_t locals[64];
    uint8_t code[1024];
    int32_t constants[128];
    int pc;
    int running;
};

void lr_vm_init(struct lr_vm *vm) {
    int i;
    vm->sp = 0;
    vm->pc = 0;
    vm->running = 1;
    for (i = 0; i < 64; i++) vm->locals[i] = 0;
}

static int lr_vm_push(struct lr_vm *vm, int32_t val) {
    if (vm->sp >= 256) return -1;
    vm->stack[vm->sp] = val;
    vm->sp++;
    return 0;
}

static int32_t lr_vm_pop(struct lr_vm *vm) {
    if (vm->sp <= 0) {
        vm->running = 0;
        return 0;
    }
    vm->sp--;
    return vm->stack[vm->sp];
}

int lr_vm_execute(struct lr_vm *vm) {
    int cycles = 0;
    while (vm->running && vm->pc < 1024) {
        uint8_t op = vm->code[vm->pc];
        vm->pc++;
        cycles++;

        switch (op) {
            case LR_OP_NOP:
                break;
            case LR_OP_PUSH: {
                uint8_t idx = vm->code[vm->pc];
                vm->pc++;
                lr_vm_push(vm, vm->constants[idx]);
                break;
            }
            case LR_OP_POP:
                lr_vm_pop(vm);
                break;
            case LR_OP_ADD: {
                int32_t b = lr_vm_pop(vm);
                int32_t a = lr_vm_pop(vm);
                lr_vm_push(vm, a + b);
                break;
            }
            case LR_OP_SUB: {
                int32_t b = lr_vm_pop(vm);
                int32_t a = lr_vm_pop(vm);
                lr_vm_push(vm, a - b);
                break;
            }
            case LR_OP_MUL: {
                int32_t b = lr_vm_pop(vm);
                int32_t a = lr_vm_pop(vm);
                lr_vm_push(vm, a * b);
                break;
            }
            case LR_OP_DIV: {
                int32_t b = lr_vm_pop(vm);
                int32_t a = lr_vm_pop(vm);
                if (b == 0) {
                    vm->running = 0;
                } else {
                    lr_vm_push(vm, a / b);
                }
                break;
            }
            case LR_OP_NEG: {
                int32_t a = lr_vm_pop(vm);
                lr_vm_push(vm, -a);
                break;
            }
            case LR_OP_DUP: {
                if (vm->sp > 0) {
                    lr_vm_push(vm, vm->stack[vm->sp - 1]);
                }
                break;
            }
            case LR_OP_SWAP: {
                if (vm->sp >= 2) {
                    int32_t tmp = vm->stack[vm->sp - 1];
                    vm->stack[vm->sp - 1] = vm->stack[vm->sp - 2];
                    vm->stack[vm->sp - 2] = tmp;
                }
                break;
            }
            case LR_OP_LOAD: {
                uint8_t slot = vm->code[vm->pc];
                vm->pc++;
                if (slot < 64) lr_vm_push(vm, vm->locals[slot]);
                break;
            }
            case LR_OP_STORE: {
                uint8_t slot = vm->code[vm->pc];
                vm->pc++;
                if (slot < 64) vm->locals[slot] = lr_vm_pop(vm);
                break;
            }
            case LR_OP_HALT:
                vm->running = 0;
                break;
            default:
                vm->running = 0;
                break;
        }
    }
    return cycles;
}

int32_t lr_vm_top(struct lr_vm *vm) {
    if (vm->sp > 0) return vm->stack[vm->sp - 1];
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C751: Bytecode VM (stack-based) - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C751: empty output");
    assert!(
        code.contains("fn lr_vm_execute"),
        "C751: Should contain lr_vm_execute function"
    );
}

/// C752: Lexer/tokenizer for simple expression language
#[test]
fn c752_lexer_tokenizer_simple_language() {
    let c_code = r#"
typedef unsigned char uint8_t;

enum lr_tok_kind {
    LR_TOK_EOF = 0,
    LR_TOK_INT,
    LR_TOK_FLOAT,
    LR_TOK_IDENT,
    LR_TOK_STRING,
    LR_TOK_PLUS,
    LR_TOK_MINUS,
    LR_TOK_STAR,
    LR_TOK_SLASH,
    LR_TOK_PERCENT,
    LR_TOK_LPAREN,
    LR_TOK_RPAREN,
    LR_TOK_LBRACE,
    LR_TOK_RBRACE,
    LR_TOK_COMMA,
    LR_TOK_SEMI,
    LR_TOK_ASSIGN,
    LR_TOK_EQ,
    LR_TOK_BANG,
    LR_TOK_NEQ,
    LR_TOK_LT,
    LR_TOK_GT,
    LR_TOK_LTE,
    LR_TOK_GTE,
    LR_TOK_ERROR
};

struct lr_token {
    enum lr_tok_kind kind;
    int start;
    int length;
    int line;
    int column;
};

struct lr_lexer {
    const char *source;
    int pos;
    int length;
    int line;
    int col;
    int token_count;
};

void lr_lexer_init(struct lr_lexer *lex, const char *src, int len) {
    lex->source = src;
    lex->pos = 0;
    lex->length = len;
    lex->line = 1;
    lex->col = 1;
    lex->token_count = 0;
}

static int lr_is_alpha(char c) {
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_';
}

static int lr_is_digit(char c) {
    return c >= '0' && c <= '9';
}

static char lr_lex_peek(struct lr_lexer *lex) {
    if (lex->pos >= lex->length) return '\0';
    return lex->source[lex->pos];
}

static char lr_lex_advance(struct lr_lexer *lex) {
    char c = lex->source[lex->pos];
    lex->pos++;
    if (c == '\n') {
        lex->line++;
        lex->col = 1;
    } else {
        lex->col++;
    }
    return c;
}

static void lr_skip_ws(struct lr_lexer *lex) {
    while (lex->pos < lex->length) {
        char c = lex->source[lex->pos];
        if (c == ' ' || c == '\t' || c == '\r' || c == '\n') {
            lr_lex_advance(lex);
        } else if (c == '/' && (lex->pos + 1) < lex->length &&
                   lex->source[lex->pos + 1] == '/') {
            while (lex->pos < lex->length && lex->source[lex->pos] != '\n') {
                lr_lex_advance(lex);
            }
        } else {
            break;
        }
    }
}

struct lr_token lr_lex_next(struct lr_lexer *lex) {
    struct lr_token tok;
    lr_skip_ws(lex);

    tok.line = lex->line;
    tok.column = lex->col;
    tok.start = lex->pos;

    if (lex->pos >= lex->length) {
        tok.kind = LR_TOK_EOF;
        tok.length = 0;
        return tok;
    }

    char c = lr_lex_advance(lex);

    if (lr_is_alpha(c)) {
        while (lex->pos < lex->length &&
               (lr_is_alpha(lr_lex_peek(lex)) || lr_is_digit(lr_lex_peek(lex)))) {
            lr_lex_advance(lex);
        }
        tok.kind = LR_TOK_IDENT;
        tok.length = lex->pos - tok.start;
        lex->token_count++;
        return tok;
    }

    if (lr_is_digit(c)) {
        int has_dot = 0;
        while (lex->pos < lex->length) {
            char p = lr_lex_peek(lex);
            if (lr_is_digit(p)) {
                lr_lex_advance(lex);
            } else if (p == '.' && !has_dot) {
                has_dot = 1;
                lr_lex_advance(lex);
            } else {
                break;
            }
        }
        tok.kind = has_dot ? LR_TOK_FLOAT : LR_TOK_INT;
        tok.length = lex->pos - tok.start;
        lex->token_count++;
        return tok;
    }

    tok.length = 1;
    lex->token_count++;

    switch (c) {
        case '+': tok.kind = LR_TOK_PLUS; break;
        case '-': tok.kind = LR_TOK_MINUS; break;
        case '*': tok.kind = LR_TOK_STAR; break;
        case '/': tok.kind = LR_TOK_SLASH; break;
        case '%': tok.kind = LR_TOK_PERCENT; break;
        case '(': tok.kind = LR_TOK_LPAREN; break;
        case ')': tok.kind = LR_TOK_RPAREN; break;
        case '{': tok.kind = LR_TOK_LBRACE; break;
        case '}': tok.kind = LR_TOK_RBRACE; break;
        case ',': tok.kind = LR_TOK_COMMA; break;
        case ';': tok.kind = LR_TOK_SEMI; break;
        case '=':
            if (lr_lex_peek(lex) == '=') {
                lr_lex_advance(lex);
                tok.kind = LR_TOK_EQ;
                tok.length = 2;
            } else {
                tok.kind = LR_TOK_ASSIGN;
            }
            break;
        case '!':
            if (lr_lex_peek(lex) == '=') {
                lr_lex_advance(lex);
                tok.kind = LR_TOK_NEQ;
                tok.length = 2;
            } else {
                tok.kind = LR_TOK_BANG;
            }
            break;
        case '<':
            if (lr_lex_peek(lex) == '=') {
                lr_lex_advance(lex);
                tok.kind = LR_TOK_LTE;
                tok.length = 2;
            } else {
                tok.kind = LR_TOK_LT;
            }
            break;
        case '>':
            if (lr_lex_peek(lex) == '=') {
                lr_lex_advance(lex);
                tok.kind = LR_TOK_GTE;
                tok.length = 2;
            } else {
                tok.kind = LR_TOK_GT;
            }
            break;
        default:
            tok.kind = LR_TOK_ERROR;
            break;
    }
    return tok;
}

int lr_count_tokens(const char *src, int len) {
    struct lr_lexer lex;
    int count = 0;
    lr_lexer_init(&lex, src, len);
    while (1) {
        struct lr_token tok = lr_lex_next(&lex);
        if (tok.kind == LR_TOK_EOF) break;
        count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C752: Lexer/tokenizer - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C752: empty output");
    assert!(
        code.contains("fn lr_lex_next"),
        "C752: Should contain lr_lex_next function"
    );
}

/// C753: Recursive descent parser for arithmetic expressions
#[test]
fn c753_recursive_descent_parser() {
    let c_code = r#"
enum lr_parse_node_type {
    LR_PN_NUM = 0,
    LR_PN_ADD,
    LR_PN_SUB,
    LR_PN_MUL,
    LR_PN_DIV,
    LR_PN_NEG,
    LR_PN_VAR
};

struct lr_parse_node {
    enum lr_parse_node_type ntype;
    int value;
    int left;
    int right;
};

struct lr_parser {
    const char *input;
    int pos;
    int length;
    struct lr_parse_node nodes[512];
    int node_count;
    int error;
};

void lr_parser_init(struct lr_parser *p, const char *input, int len) {
    p->input = input;
    p->pos = 0;
    p->length = len;
    p->node_count = 0;
    p->error = 0;
}

static int lr_parser_alloc(struct lr_parser *p, enum lr_parse_node_type t, int val, int l, int r) {
    if (p->node_count >= 512) {
        p->error = 1;
        return -1;
    }
    int idx = p->node_count;
    p->nodes[idx].ntype = t;
    p->nodes[idx].value = val;
    p->nodes[idx].left = l;
    p->nodes[idx].right = r;
    p->node_count++;
    return idx;
}

static void lr_parser_skip_ws(struct lr_parser *p) {
    while (p->pos < p->length) {
        char c = p->input[p->pos];
        if (c == ' ' || c == '\t' || c == '\n') {
            p->pos++;
        } else {
            break;
        }
    }
}

static char lr_parser_peek(struct lr_parser *p) {
    lr_parser_skip_ws(p);
    if (p->pos >= p->length) return '\0';
    return p->input[p->pos];
}

static char lr_parser_eat(struct lr_parser *p) {
    lr_parser_skip_ws(p);
    if (p->pos >= p->length) return '\0';
    char c = p->input[p->pos];
    p->pos++;
    return c;
}

static int lr_parse_expr(struct lr_parser *p);
static int lr_parse_term(struct lr_parser *p);
static int lr_parse_factor(struct lr_parser *p);

static int lr_parse_factor(struct lr_parser *p) {
    char c = lr_parser_peek(p);
    if (c == '(') {
        lr_parser_eat(p);
        int node = lr_parse_expr(p);
        if (lr_parser_peek(p) == ')') {
            lr_parser_eat(p);
        } else {
            p->error = 1;
        }
        return node;
    }
    if (c == '-') {
        lr_parser_eat(p);
        int operand = lr_parse_factor(p);
        return lr_parser_alloc(p, LR_PN_NEG, 0, operand, -1);
    }
    if (c >= '0' && c <= '9') {
        int val = 0;
        while (p->pos < p->length && p->input[p->pos] >= '0' && p->input[p->pos] <= '9') {
            val = val * 10 + (p->input[p->pos] - '0');
            p->pos++;
        }
        return lr_parser_alloc(p, LR_PN_NUM, val, -1, -1);
    }
    if (c >= 'a' && c <= 'z') {
        int var_id = c - 'a';
        p->pos++;
        return lr_parser_alloc(p, LR_PN_VAR, var_id, -1, -1);
    }
    p->error = 1;
    return -1;
}

static int lr_parse_term(struct lr_parser *p) {
    int left = lr_parse_factor(p);
    while (!p->error) {
        char c = lr_parser_peek(p);
        if (c == '*') {
            lr_parser_eat(p);
            int right = lr_parse_factor(p);
            left = lr_parser_alloc(p, LR_PN_MUL, 0, left, right);
        } else if (c == '/') {
            lr_parser_eat(p);
            int right = lr_parse_factor(p);
            left = lr_parser_alloc(p, LR_PN_DIV, 0, left, right);
        } else {
            break;
        }
    }
    return left;
}

static int lr_parse_expr(struct lr_parser *p) {
    int left = lr_parse_term(p);
    while (!p->error) {
        char c = lr_parser_peek(p);
        if (c == '+') {
            lr_parser_eat(p);
            int right = lr_parse_term(p);
            left = lr_parser_alloc(p, LR_PN_ADD, 0, left, right);
        } else if (c == '-') {
            lr_parser_eat(p);
            int right = lr_parse_term(p);
            left = lr_parser_alloc(p, LR_PN_SUB, 0, left, right);
        } else {
            break;
        }
    }
    return left;
}

int lr_parse(struct lr_parser *p) {
    return lr_parse_expr(p);
}

int lr_eval_node(struct lr_parser *p, int idx, int *vars) {
    if (idx < 0 || idx >= p->node_count) return 0;
    struct lr_parse_node *n = &p->nodes[idx];
    switch (n->ntype) {
        case LR_PN_NUM: return n->value;
        case LR_PN_VAR: return vars[n->value];
        case LR_PN_ADD: return lr_eval_node(p, n->left, vars) + lr_eval_node(p, n->right, vars);
        case LR_PN_SUB: return lr_eval_node(p, n->left, vars) - lr_eval_node(p, n->right, vars);
        case LR_PN_MUL: return lr_eval_node(p, n->left, vars) * lr_eval_node(p, n->right, vars);
        case LR_PN_DIV: {
            int denom = lr_eval_node(p, n->right, vars);
            if (denom == 0) return 0;
            return lr_eval_node(p, n->left, vars) / denom;
        }
        case LR_PN_NEG: return -lr_eval_node(p, n->left, vars);
        default: return 0;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C753: Recursive descent parser - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C753: empty output");
    assert!(
        code.contains("fn lr_parse_expr"),
        "C753: Should contain lr_parse_expr function"
    );
}

/// C754: Abstract syntax tree builder with node pool
#[test]
fn c754_ast_builder_node_pool() {
    let c_code = r#"
enum lr_ast_kind {
    LR_AST_LITERAL = 0,
    LR_AST_IDENT,
    LR_AST_BINOP,
    LR_AST_UNOP,
    LR_AST_CALL,
    LR_AST_IF,
    LR_AST_WHILE,
    LR_AST_BLOCK,
    LR_AST_ASSIGN,
    LR_AST_RETURN
};

enum lr_binop {
    LR_BIN_ADD = 0,
    LR_BIN_SUB,
    LR_BIN_MUL,
    LR_BIN_DIV,
    LR_BIN_EQ,
    LR_BIN_NEQ,
    LR_BIN_LT,
    LR_BIN_GT
};

struct lr_ast_node {
    enum lr_ast_kind kind;
    int int_val;
    enum lr_binop op;
    int child0;
    int child1;
    int child2;
    int next_sibling;
    int line;
};

struct lr_ast_pool {
    struct lr_ast_node nodes[1024];
    int count;
    int root;
};

void lr_ast_pool_init(struct lr_ast_pool *pool) {
    pool->count = 0;
    pool->root = -1;
}

int lr_ast_alloc(struct lr_ast_pool *pool, enum lr_ast_kind kind) {
    if (pool->count >= 1024) return -1;
    int idx = pool->count;
    pool->nodes[idx].kind = kind;
    pool->nodes[idx].int_val = 0;
    pool->nodes[idx].op = LR_BIN_ADD;
    pool->nodes[idx].child0 = -1;
    pool->nodes[idx].child1 = -1;
    pool->nodes[idx].child2 = -1;
    pool->nodes[idx].next_sibling = -1;
    pool->nodes[idx].line = 0;
    pool->count++;
    return idx;
}

int lr_ast_make_literal(struct lr_ast_pool *pool, int value, int line) {
    int idx = lr_ast_alloc(pool, LR_AST_LITERAL);
    if (idx >= 0) {
        pool->nodes[idx].int_val = value;
        pool->nodes[idx].line = line;
    }
    return idx;
}

int lr_ast_make_binop(struct lr_ast_pool *pool, enum lr_binop op, int left, int right, int line) {
    int idx = lr_ast_alloc(pool, LR_AST_BINOP);
    if (idx >= 0) {
        pool->nodes[idx].op = op;
        pool->nodes[idx].child0 = left;
        pool->nodes[idx].child1 = right;
        pool->nodes[idx].line = line;
    }
    return idx;
}

int lr_ast_make_if(struct lr_ast_pool *pool, int cond, int then_body, int else_body, int line) {
    int idx = lr_ast_alloc(pool, LR_AST_IF);
    if (idx >= 0) {
        pool->nodes[idx].child0 = cond;
        pool->nodes[idx].child1 = then_body;
        pool->nodes[idx].child2 = else_body;
        pool->nodes[idx].line = line;
    }
    return idx;
}

int lr_ast_make_while(struct lr_ast_pool *pool, int cond, int body, int line) {
    int idx = lr_ast_alloc(pool, LR_AST_WHILE);
    if (idx >= 0) {
        pool->nodes[idx].child0 = cond;
        pool->nodes[idx].child1 = body;
        pool->nodes[idx].line = line;
    }
    return idx;
}

int lr_ast_count_nodes(struct lr_ast_pool *pool, int idx) {
    if (idx < 0 || idx >= pool->count) return 0;
    int total = 1;
    struct lr_ast_node *n = &pool->nodes[idx];
    total += lr_ast_count_nodes(pool, n->child0);
    total += lr_ast_count_nodes(pool, n->child1);
    total += lr_ast_count_nodes(pool, n->child2);
    total += lr_ast_count_nodes(pool, n->next_sibling);
    return total;
}

int lr_ast_depth(struct lr_ast_pool *pool, int idx) {
    if (idx < 0 || idx >= pool->count) return 0;
    struct lr_ast_node *n = &pool->nodes[idx];
    int d0 = lr_ast_depth(pool, n->child0);
    int d1 = lr_ast_depth(pool, n->child1);
    int d2 = lr_ast_depth(pool, n->child2);
    int max = d0;
    if (d1 > max) max = d1;
    if (d2 > max) max = d2;
    return max + 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C754: AST builder - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C754: empty output");
    assert!(
        code.contains("fn lr_ast_alloc"),
        "C754: Should contain lr_ast_alloc function"
    );
}

/// C755: Symbol table with scope chains
#[test]
fn c755_symbol_table_scope_chains() {
    let c_code = r#"
enum lr_sym_kind {
    LR_SYM_VAR = 0,
    LR_SYM_FUNC,
    LR_SYM_PARAM,
    LR_SYM_CONST
};

enum lr_sym_type {
    LR_TYPE_INT = 0,
    LR_TYPE_FLOAT,
    LR_TYPE_BOOL,
    LR_TYPE_STRING,
    LR_TYPE_VOID
};

struct lr_symbol {
    char name[32];
    enum lr_sym_kind kind;
    enum lr_sym_type stype;
    int scope_level;
    int slot;
    int is_initialized;
    int is_mutable;
};

struct lr_scope {
    int parent;
    int sym_start;
    int sym_count;
    int level;
};

struct lr_symtab {
    struct lr_symbol symbols[256];
    int sym_count;
    struct lr_scope scopes[32];
    int scope_count;
    int current_scope;
};

void lr_symtab_init(struct lr_symtab *tab) {
    tab->sym_count = 0;
    tab->scope_count = 0;
    tab->current_scope = -1;
}

int lr_symtab_push_scope(struct lr_symtab *tab) {
    if (tab->scope_count >= 32) return -1;
    int idx = tab->scope_count;
    tab->scopes[idx].parent = tab->current_scope;
    tab->scopes[idx].sym_start = tab->sym_count;
    tab->scopes[idx].sym_count = 0;
    tab->scopes[idx].level = (tab->current_scope >= 0)
        ? tab->scopes[tab->current_scope].level + 1
        : 0;
    tab->current_scope = idx;
    tab->scope_count++;
    return idx;
}

void lr_symtab_pop_scope(struct lr_symtab *tab) {
    if (tab->current_scope >= 0) {
        tab->current_scope = tab->scopes[tab->current_scope].parent;
    }
}

static int lr_str_eq(const char *a, const char *b) {
    int i = 0;
    while (a[i] != '\0' && b[i] != '\0') {
        if (a[i] != b[i]) return 0;
        i++;
    }
    return a[i] == b[i];
}

static void lr_str_copy(char *dst, const char *src, int max) {
    int i = 0;
    while (i < max - 1 && src[i] != '\0') {
        dst[i] = src[i];
        i++;
    }
    dst[i] = '\0';
}

int lr_symtab_add(struct lr_symtab *tab, const char *name, enum lr_sym_kind kind,
                  enum lr_sym_type stype, int is_mutable) {
    if (tab->sym_count >= 256 || tab->current_scope < 0) return -1;
    int idx = tab->sym_count;
    lr_str_copy(tab->symbols[idx].name, name, 32);
    tab->symbols[idx].kind = kind;
    tab->symbols[idx].stype = stype;
    tab->symbols[idx].scope_level = tab->scopes[tab->current_scope].level;
    tab->symbols[idx].slot = tab->scopes[tab->current_scope].sym_count;
    tab->symbols[idx].is_initialized = 0;
    tab->symbols[idx].is_mutable = is_mutable;
    tab->scopes[tab->current_scope].sym_count++;
    tab->sym_count++;
    return idx;
}

int lr_symtab_lookup(struct lr_symtab *tab, const char *name) {
    int scope = tab->current_scope;
    while (scope >= 0) {
        int start = tab->scopes[scope].sym_start;
        int count = tab->scopes[scope].sym_count;
        int i;
        for (i = start + count - 1; i >= start; i--) {
            if (lr_str_eq(tab->symbols[i].name, name)) {
                return i;
            }
        }
        scope = tab->scopes[scope].parent;
    }
    return -1;
}

int lr_symtab_lookup_local(struct lr_symtab *tab, const char *name) {
    if (tab->current_scope < 0) return -1;
    int start = tab->scopes[tab->current_scope].sym_start;
    int count = tab->scopes[tab->current_scope].sym_count;
    int i;
    for (i = start; i < start + count; i++) {
        if (lr_str_eq(tab->symbols[i].name, name)) {
            return i;
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C755: Symbol table with scope chains - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C755: empty output");
    assert!(
        code.contains("fn lr_symtab_lookup"),
        "C755: Should contain lr_symtab_lookup function"
    );
}

// ============================================================================
// C756-C760: Analysis/Optimization (Type Checker, Register Alloc, Const Fold,
//            DCE, TCO)
// ============================================================================

/// C756: Type checker for simple types
#[test]
fn c756_type_checker_simple_types() {
    let c_code = r#"
enum lr_tc_type {
    LR_TC_INT = 0,
    LR_TC_FLOAT,
    LR_TC_BOOL,
    LR_TC_STRING,
    LR_TC_VOID,
    LR_TC_ERROR
};

enum lr_tc_expr {
    LR_TC_LIT_INT = 0,
    LR_TC_LIT_FLOAT,
    LR_TC_LIT_BOOL,
    LR_TC_LIT_STR,
    LR_TC_ADD,
    LR_TC_SUB,
    LR_TC_MUL,
    LR_TC_DIV,
    LR_TC_EQ,
    LR_TC_LT,
    LR_TC_AND,
    LR_TC_OR,
    LR_TC_NOT,
    LR_TC_NEGATE,
    LR_TC_CALL,
    LR_TC_VAR_REF
};

struct lr_tc_node {
    enum lr_tc_expr expr;
    enum lr_tc_type resolved_type;
    int child0;
    int child1;
    int int_val;
};

struct lr_type_checker {
    struct lr_tc_node nodes[256];
    int node_count;
    enum lr_tc_type var_types[64];
    int var_count;
    int error_count;
};

void lr_tc_init(struct lr_type_checker *tc) {
    tc->node_count = 0;
    tc->var_count = 0;
    tc->error_count = 0;
}

static int lr_tc_is_numeric(enum lr_tc_type t) {
    return t == LR_TC_INT || t == LR_TC_FLOAT;
}

static enum lr_tc_type lr_tc_promote(enum lr_tc_type a, enum lr_tc_type b) {
    if (a == LR_TC_FLOAT || b == LR_TC_FLOAT) return LR_TC_FLOAT;
    if (a == LR_TC_INT && b == LR_TC_INT) return LR_TC_INT;
    return LR_TC_ERROR;
}

enum lr_tc_type lr_tc_check(struct lr_type_checker *tc, int idx) {
    if (idx < 0 || idx >= tc->node_count) return LR_TC_ERROR;
    struct lr_tc_node *n = &tc->nodes[idx];

    switch (n->expr) {
        case LR_TC_LIT_INT:
            n->resolved_type = LR_TC_INT;
            return LR_TC_INT;
        case LR_TC_LIT_FLOAT:
            n->resolved_type = LR_TC_FLOAT;
            return LR_TC_FLOAT;
        case LR_TC_LIT_BOOL:
            n->resolved_type = LR_TC_BOOL;
            return LR_TC_BOOL;
        case LR_TC_LIT_STR:
            n->resolved_type = LR_TC_STRING;
            return LR_TC_STRING;

        case LR_TC_ADD:
        case LR_TC_SUB:
        case LR_TC_MUL:
        case LR_TC_DIV: {
            enum lr_tc_type lt = lr_tc_check(tc, n->child0);
            enum lr_tc_type rt = lr_tc_check(tc, n->child1);
            if (!lr_tc_is_numeric(lt) || !lr_tc_is_numeric(rt)) {
                tc->error_count++;
                n->resolved_type = LR_TC_ERROR;
                return LR_TC_ERROR;
            }
            enum lr_tc_type result = lr_tc_promote(lt, rt);
            n->resolved_type = result;
            return result;
        }

        case LR_TC_EQ: {
            enum lr_tc_type lt = lr_tc_check(tc, n->child0);
            enum lr_tc_type rt = lr_tc_check(tc, n->child1);
            if (lt != rt) {
                tc->error_count++;
                n->resolved_type = LR_TC_ERROR;
                return LR_TC_ERROR;
            }
            n->resolved_type = LR_TC_BOOL;
            return LR_TC_BOOL;
        }

        case LR_TC_LT: {
            enum lr_tc_type lt = lr_tc_check(tc, n->child0);
            enum lr_tc_type rt = lr_tc_check(tc, n->child1);
            if (!lr_tc_is_numeric(lt) || !lr_tc_is_numeric(rt)) {
                tc->error_count++;
                n->resolved_type = LR_TC_ERROR;
                return LR_TC_ERROR;
            }
            n->resolved_type = LR_TC_BOOL;
            return LR_TC_BOOL;
        }

        case LR_TC_AND:
        case LR_TC_OR: {
            enum lr_tc_type lt = lr_tc_check(tc, n->child0);
            enum lr_tc_type rt = lr_tc_check(tc, n->child1);
            if (lt != LR_TC_BOOL || rt != LR_TC_BOOL) {
                tc->error_count++;
                n->resolved_type = LR_TC_ERROR;
                return LR_TC_ERROR;
            }
            n->resolved_type = LR_TC_BOOL;
            return LR_TC_BOOL;
        }

        case LR_TC_NOT: {
            enum lr_tc_type operand = lr_tc_check(tc, n->child0);
            if (operand != LR_TC_BOOL) {
                tc->error_count++;
                n->resolved_type = LR_TC_ERROR;
                return LR_TC_ERROR;
            }
            n->resolved_type = LR_TC_BOOL;
            return LR_TC_BOOL;
        }

        case LR_TC_NEGATE: {
            enum lr_tc_type operand = lr_tc_check(tc, n->child0);
            if (!lr_tc_is_numeric(operand)) {
                tc->error_count++;
                n->resolved_type = LR_TC_ERROR;
                return LR_TC_ERROR;
            }
            n->resolved_type = operand;
            return operand;
        }

        case LR_TC_VAR_REF: {
            if (n->int_val >= 0 && n->int_val < tc->var_count) {
                n->resolved_type = tc->var_types[n->int_val];
                return tc->var_types[n->int_val];
            }
            tc->error_count++;
            n->resolved_type = LR_TC_ERROR;
            return LR_TC_ERROR;
        }

        default:
            n->resolved_type = LR_TC_ERROR;
            return LR_TC_ERROR;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C756: Type checker - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C756: empty output");
    assert!(
        code.contains("fn lr_tc_check"),
        "C756: Should contain lr_tc_check function"
    );
}

/// C757: Register allocator (linear scan)
#[test]
fn c757_register_allocator_linear_scan() {
    let c_code = r#"
struct lr_interval {
    int vreg;
    int start;
    int end;
    int assigned_reg;
    int spilled;
};

struct lr_regalloc {
    struct lr_interval intervals[128];
    int interval_count;
    int reg_available[16];
    int num_regs;
    int spill_slot;
    int total_spills;
};

void lr_regalloc_init(struct lr_regalloc *ra, int num_regs) {
    int i;
    ra->interval_count = 0;
    ra->num_regs = num_regs;
    if (num_regs > 16) ra->num_regs = 16;
    ra->spill_slot = 0;
    ra->total_spills = 0;
    for (i = 0; i < 16; i++) {
        ra->reg_available[i] = 1;
    }
}

int lr_regalloc_add_interval(struct lr_regalloc *ra, int vreg, int start, int end) {
    if (ra->interval_count >= 128) return -1;
    int idx = ra->interval_count;
    ra->intervals[idx].vreg = vreg;
    ra->intervals[idx].start = start;
    ra->intervals[idx].end = end;
    ra->intervals[idx].assigned_reg = -1;
    ra->intervals[idx].spilled = 0;
    ra->interval_count++;
    return idx;
}

static void lr_regalloc_sort(struct lr_regalloc *ra) {
    int i, j;
    for (i = 0; i < ra->interval_count - 1; i++) {
        for (j = 0; j < ra->interval_count - i - 1; j++) {
            if (ra->intervals[j].start > ra->intervals[j + 1].start) {
                struct lr_interval tmp = ra->intervals[j];
                ra->intervals[j] = ra->intervals[j + 1];
                ra->intervals[j + 1] = tmp;
            }
        }
    }
}

static int lr_regalloc_find_free(struct lr_regalloc *ra) {
    int i;
    for (i = 0; i < ra->num_regs; i++) {
        if (ra->reg_available[i]) return i;
    }
    return -1;
}

static void lr_regalloc_expire(struct lr_regalloc *ra, int current_start) {
    int i;
    for (i = 0; i < ra->interval_count; i++) {
        if (ra->intervals[i].assigned_reg >= 0 &&
            ra->intervals[i].end <= current_start) {
            ra->reg_available[ra->intervals[i].assigned_reg] = 1;
            ra->intervals[i].assigned_reg = -2;
        }
    }
}

void lr_regalloc_allocate(struct lr_regalloc *ra) {
    int i;
    lr_regalloc_sort(ra);

    for (i = 0; i < ra->interval_count; i++) {
        lr_regalloc_expire(ra, ra->intervals[i].start);
        int reg = lr_regalloc_find_free(ra);
        if (reg >= 0) {
            ra->intervals[i].assigned_reg = reg;
            ra->reg_available[reg] = 0;
        } else {
            ra->intervals[i].spilled = 1;
            ra->intervals[i].assigned_reg = -(ra->spill_slot + 100);
            ra->spill_slot++;
            ra->total_spills++;
        }
    }
}

int lr_regalloc_spill_count(struct lr_regalloc *ra) {
    return ra->total_spills;
}

int lr_regalloc_get_reg(struct lr_regalloc *ra, int vreg) {
    int i;
    for (i = 0; i < ra->interval_count; i++) {
        if (ra->intervals[i].vreg == vreg) {
            return ra->intervals[i].assigned_reg;
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C757: Register allocator - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C757: empty output");
    assert!(
        code.contains("fn lr_regalloc_allocate"),
        "C757: Should contain lr_regalloc_allocate function"
    );
}

/// C758: Constant folding optimizer
#[test]
fn c758_constant_folding_optimizer() {
    let c_code = r#"
enum lr_cf_op {
    LR_CF_CONST = 0,
    LR_CF_ADD,
    LR_CF_SUB,
    LR_CF_MUL,
    LR_CF_DIV,
    LR_CF_NEG,
    LR_CF_EQ,
    LR_CF_LT,
    LR_CF_AND,
    LR_CF_OR,
    LR_CF_NOT,
    LR_CF_VAR
};

struct lr_cf_node {
    enum lr_cf_op op;
    int value;
    int is_const;
    int left;
    int right;
};

struct lr_const_folder {
    struct lr_cf_node nodes[256];
    int count;
    int folds_performed;
};

void lr_cf_init(struct lr_const_folder *cf) {
    cf->count = 0;
    cf->folds_performed = 0;
}

int lr_cf_add_node(struct lr_const_folder *cf, enum lr_cf_op op, int val, int left, int right) {
    if (cf->count >= 256) return -1;
    int idx = cf->count;
    cf->nodes[idx].op = op;
    cf->nodes[idx].value = val;
    cf->nodes[idx].is_const = (op == LR_CF_CONST) ? 1 : 0;
    cf->nodes[idx].left = left;
    cf->nodes[idx].right = right;
    cf->count++;
    return idx;
}

int lr_cf_fold(struct lr_const_folder *cf, int idx) {
    if (idx < 0 || idx >= cf->count) return 0;
    struct lr_cf_node *n = &cf->nodes[idx];

    if (n->is_const) return 1;
    if (n->op == LR_CF_VAR) return 0;

    int left_const = (n->left >= 0) ? lr_cf_fold(cf, n->left) : 0;
    int right_const = (n->right >= 0) ? lr_cf_fold(cf, n->right) : 0;

    if (n->op == LR_CF_NEG || n->op == LR_CF_NOT) {
        if (left_const) {
            int lv = cf->nodes[n->left].value;
            if (n->op == LR_CF_NEG) {
                n->value = -lv;
            } else {
                n->value = lv ? 0 : 1;
            }
            n->is_const = 1;
            n->op = LR_CF_CONST;
            cf->folds_performed++;
            return 1;
        }
        return 0;
    }

    if (!left_const || !right_const) {
        if (left_const && n->op == LR_CF_MUL &&
            cf->nodes[n->left].value == 0) {
            n->value = 0;
            n->is_const = 1;
            n->op = LR_CF_CONST;
            cf->folds_performed++;
            return 1;
        }
        if (right_const && n->op == LR_CF_MUL &&
            cf->nodes[n->right].value == 0) {
            n->value = 0;
            n->is_const = 1;
            n->op = LR_CF_CONST;
            cf->folds_performed++;
            return 1;
        }
        if (right_const && n->op == LR_CF_MUL &&
            cf->nodes[n->right].value == 1) {
            n->value = cf->nodes[n->left].value;
            n->op = cf->nodes[n->left].op;
            n->is_const = cf->nodes[n->left].is_const;
            cf->folds_performed++;
            return n->is_const;
        }
        if (right_const && n->op == LR_CF_ADD &&
            cf->nodes[n->right].value == 0) {
            n->value = cf->nodes[n->left].value;
            n->op = cf->nodes[n->left].op;
            n->is_const = cf->nodes[n->left].is_const;
            cf->folds_performed++;
            return n->is_const;
        }
        return 0;
    }

    int lv = cf->nodes[n->left].value;
    int rv = cf->nodes[n->right].value;

    switch (n->op) {
        case LR_CF_ADD: n->value = lv + rv; break;
        case LR_CF_SUB: n->value = lv - rv; break;
        case LR_CF_MUL: n->value = lv * rv; break;
        case LR_CF_DIV:
            if (rv == 0) return 0;
            n->value = lv / rv;
            break;
        case LR_CF_EQ: n->value = (lv == rv) ? 1 : 0; break;
        case LR_CF_LT: n->value = (lv < rv) ? 1 : 0; break;
        case LR_CF_AND: n->value = (lv && rv) ? 1 : 0; break;
        case LR_CF_OR: n->value = (lv || rv) ? 1 : 0; break;
        default: return 0;
    }

    n->is_const = 1;
    n->op = LR_CF_CONST;
    cf->folds_performed++;
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C758: Constant folding optimizer - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C758: empty output");
    assert!(
        code.contains("fn lr_cf_fold"),
        "C758: Should contain lr_cf_fold function"
    );
}

/// C759: Dead code eliminator
#[test]
fn c759_dead_code_eliminator() {
    let c_code = r#"
enum lr_dce_inst {
    LR_DCE_NOP = 0,
    LR_DCE_ASSIGN,
    LR_DCE_ADD,
    LR_DCE_SUB,
    LR_DCE_MUL,
    LR_DCE_LOAD,
    LR_DCE_STORE,
    LR_DCE_BRANCH,
    LR_DCE_RET,
    LR_DCE_CALL
};

struct lr_dce_op {
    enum lr_dce_inst inst;
    int dest;
    int src1;
    int src2;
    int is_dead;
};

struct lr_dce {
    struct lr_dce_op ops[256];
    int op_count;
    int var_used[128];
    int eliminated;
};

void lr_dce_init(struct lr_dce *dce) {
    int i;
    dce->op_count = 0;
    dce->eliminated = 0;
    for (i = 0; i < 128; i++) dce->var_used[i] = 0;
}

int lr_dce_add_op(struct lr_dce *dce, enum lr_dce_inst inst, int dest, int s1, int s2) {
    if (dce->op_count >= 256) return -1;
    int idx = dce->op_count;
    dce->ops[idx].inst = inst;
    dce->ops[idx].dest = dest;
    dce->ops[idx].src1 = s1;
    dce->ops[idx].src2 = s2;
    dce->ops[idx].is_dead = 0;
    dce->op_count++;
    return idx;
}

static void lr_dce_mark_used(struct lr_dce *dce, int var) {
    if (var >= 0 && var < 128) {
        dce->var_used[var] = 1;
    }
}

static int lr_dce_has_side_effects(enum lr_dce_inst inst) {
    return inst == LR_DCE_STORE || inst == LR_DCE_CALL ||
           inst == LR_DCE_BRANCH || inst == LR_DCE_RET;
}

void lr_dce_analyze(struct lr_dce *dce) {
    int i;
    int changed = 1;

    for (i = 0; i < 128; i++) dce->var_used[i] = 0;

    for (i = dce->op_count - 1; i >= 0; i--) {
        if (lr_dce_has_side_effects(dce->ops[i].inst)) {
            lr_dce_mark_used(dce, dce->ops[i].src1);
            lr_dce_mark_used(dce, dce->ops[i].src2);
        }
    }

    while (changed) {
        changed = 0;
        for (i = dce->op_count - 1; i >= 0; i--) {
            int dest = dce->ops[i].dest;
            if (dest >= 0 && dest < 128 && dce->var_used[dest]) {
                int s1 = dce->ops[i].src1;
                int s2 = dce->ops[i].src2;
                if (s1 >= 0 && s1 < 128 && !dce->var_used[s1]) {
                    dce->var_used[s1] = 1;
                    changed = 1;
                }
                if (s2 >= 0 && s2 < 128 && !dce->var_used[s2]) {
                    dce->var_used[s2] = 1;
                    changed = 1;
                }
            }
        }
    }
}

int lr_dce_eliminate(struct lr_dce *dce) {
    int i;
    dce->eliminated = 0;
    lr_dce_analyze(dce);

    for (i = 0; i < dce->op_count; i++) {
        if (!lr_dce_has_side_effects(dce->ops[i].inst)) {
            int dest = dce->ops[i].dest;
            if (dest >= 0 && dest < 128 && !dce->var_used[dest]) {
                dce->ops[i].is_dead = 1;
                dce->ops[i].inst = LR_DCE_NOP;
                dce->eliminated++;
            }
        }
    }
    return dce->eliminated;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C759: Dead code eliminator - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C759: empty output");
    assert!(
        code.contains("fn lr_dce_eliminate"),
        "C759: Should contain lr_dce_eliminate function"
    );
}

/// C760: Tail call optimizer
#[test]
fn c760_tail_call_optimizer() {
    let c_code = r#"
enum lr_tco_inst {
    LR_TCO_NOP = 0,
    LR_TCO_PUSH,
    LR_TCO_POP,
    LR_TCO_CALL,
    LR_TCO_TAIL_CALL,
    LR_TCO_RET,
    LR_TCO_JUMP,
    LR_TCO_ADD,
    LR_TCO_LOAD
};

struct lr_tco_op {
    enum lr_tco_inst inst;
    int arg0;
    int arg1;
};

struct lr_tco_func {
    struct lr_tco_op ops[64];
    int op_count;
    int self_id;
    int param_count;
};

struct lr_tco {
    struct lr_tco_func funcs[32];
    int func_count;
    int optimized_count;
};

void lr_tco_init(struct lr_tco *tco) {
    tco->func_count = 0;
    tco->optimized_count = 0;
}

int lr_tco_add_func(struct lr_tco *tco, int self_id, int params) {
    if (tco->func_count >= 32) return -1;
    int idx = tco->func_count;
    tco->funcs[idx].op_count = 0;
    tco->funcs[idx].self_id = self_id;
    tco->funcs[idx].param_count = params;
    tco->func_count++;
    return idx;
}

int lr_tco_add_op(struct lr_tco *tco, int func_idx, enum lr_tco_inst inst, int a0, int a1) {
    if (func_idx < 0 || func_idx >= tco->func_count) return -1;
    struct lr_tco_func *f = &tco->funcs[func_idx];
    if (f->op_count >= 64) return -1;
    int idx = f->op_count;
    f->ops[idx].inst = inst;
    f->ops[idx].arg0 = a0;
    f->ops[idx].arg1 = a1;
    f->op_count++;
    return idx;
}

static int lr_tco_is_tail_position(struct lr_tco_func *f, int op_idx) {
    if (op_idx < 0 || op_idx >= f->op_count) return 0;
    if (f->ops[op_idx].inst != LR_TCO_CALL) return 0;

    int next = op_idx + 1;
    while (next < f->op_count) {
        if (f->ops[next].inst == LR_TCO_RET) return 1;
        if (f->ops[next].inst != LR_TCO_NOP) return 0;
        next++;
    }
    return 0;
}

static int lr_tco_is_self_call(struct lr_tco_func *f, int op_idx) {
    if (f->ops[op_idx].inst != LR_TCO_CALL) return 0;
    return f->ops[op_idx].arg0 == f->self_id;
}

int lr_tco_optimize_func(struct lr_tco *tco, int func_idx) {
    if (func_idx < 0 || func_idx >= tco->func_count) return 0;
    struct lr_tco_func *f = &tco->funcs[func_idx];
    int count = 0;
    int i;

    for (i = 0; i < f->op_count; i++) {
        if (lr_tco_is_tail_position(f, i) && lr_tco_is_self_call(f, i)) {
            f->ops[i].inst = LR_TCO_TAIL_CALL;
            count++;
        }
    }

    tco->optimized_count += count;
    return count;
}

int lr_tco_optimize_all(struct lr_tco *tco) {
    int total = 0;
    int i;
    for (i = 0; i < tco->func_count; i++) {
        total += lr_tco_optimize_func(tco, i);
    }
    return total;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C760: Tail call optimizer - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C760: empty output");
    assert!(
        code.contains("fn lr_tco_optimize_all"),
        "C760: Should contain lr_tco_optimize_all function"
    );
}

// ============================================================================
// C761-C763: Runtime (Closures, Mark-Compact GC, String Interning)
// ============================================================================

/// C761: Closure/upvalue capture
#[test]
fn c761_closure_upvalue_capture() {
    let c_code = r#"
enum lr_upval_state {
    LR_UPVAL_OPEN = 0,
    LR_UPVAL_CLOSED
};

struct lr_upvalue {
    enum lr_upval_state state;
    int stack_index;
    int closed_value;
    int next;
};

struct lr_closure {
    int func_id;
    int upvalue_indices[16];
    int upvalue_count;
    int arity;
};

struct lr_upval_pool {
    struct lr_upvalue upvalues[256];
    int count;
    int open_list_head;
    struct lr_closure closures[64];
    int closure_count;
    int stack[256];
    int sp;
};

void lr_upval_pool_init(struct lr_upval_pool *pool) {
    pool->count = 0;
    pool->open_list_head = -1;
    pool->closure_count = 0;
    pool->sp = 0;
}

int lr_upval_capture(struct lr_upval_pool *pool, int stack_idx) {
    int cur = pool->open_list_head;
    while (cur >= 0) {
        if (pool->upvalues[cur].stack_index == stack_idx) {
            return cur;
        }
        cur = pool->upvalues[cur].next;
    }

    if (pool->count >= 256) return -1;
    int idx = pool->count;
    pool->upvalues[idx].state = LR_UPVAL_OPEN;
    pool->upvalues[idx].stack_index = stack_idx;
    pool->upvalues[idx].closed_value = 0;
    pool->upvalues[idx].next = pool->open_list_head;
    pool->open_list_head = idx;
    pool->count++;
    return idx;
}

void lr_upval_close_at(struct lr_upval_pool *pool, int stack_level) {
    int prev = -1;
    int cur = pool->open_list_head;

    while (cur >= 0) {
        int next = pool->upvalues[cur].next;
        if (pool->upvalues[cur].stack_index >= stack_level) {
            pool->upvalues[cur].state = LR_UPVAL_CLOSED;
            pool->upvalues[cur].closed_value =
                pool->stack[pool->upvalues[cur].stack_index];

            if (prev >= 0) {
                pool->upvalues[prev].next = next;
            } else {
                pool->open_list_head = next;
            }
        } else {
            prev = cur;
        }
        cur = next;
    }
}

int lr_upval_get(struct lr_upval_pool *pool, int upval_idx) {
    if (upval_idx < 0 || upval_idx >= pool->count) return 0;
    struct lr_upvalue *uv = &pool->upvalues[upval_idx];
    if (uv->state == LR_UPVAL_OPEN) {
        return pool->stack[uv->stack_index];
    }
    return uv->closed_value;
}

void lr_upval_set(struct lr_upval_pool *pool, int upval_idx, int value) {
    if (upval_idx < 0 || upval_idx >= pool->count) return;
    struct lr_upvalue *uv = &pool->upvalues[upval_idx];
    if (uv->state == LR_UPVAL_OPEN) {
        pool->stack[uv->stack_index] = value;
    } else {
        uv->closed_value = value;
    }
}

int lr_closure_create(struct lr_upval_pool *pool, int func_id, int arity) {
    if (pool->closure_count >= 64) return -1;
    int idx = pool->closure_count;
    pool->closures[idx].func_id = func_id;
    pool->closures[idx].arity = arity;
    pool->closures[idx].upvalue_count = 0;
    pool->closure_count++;
    return idx;
}

int lr_closure_add_upvalue(struct lr_upval_pool *pool, int closure_idx, int upval_idx) {
    if (closure_idx < 0 || closure_idx >= pool->closure_count) return -1;
    struct lr_closure *cl = &pool->closures[closure_idx];
    if (cl->upvalue_count >= 16) return -1;
    cl->upvalue_indices[cl->upvalue_count] = upval_idx;
    cl->upvalue_count++;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C761: Closure/upvalue capture - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C761: empty output");
    assert!(
        code.contains("fn lr_upval_capture"),
        "C761: Should contain lr_upval_capture function"
    );
}

/// C762: Mark-compact garbage collector for VM
#[test]
fn c762_mark_compact_garbage_collector() {
    let c_code = r#"
typedef unsigned char uint8_t;

enum lr_gc_obj_type {
    LR_GC_INT = 0,
    LR_GC_PAIR,
    LR_GC_STRING,
    LR_GC_ARRAY,
    LR_GC_CLOSURE
};

struct lr_gc_object {
    enum lr_gc_obj_type otype;
    uint8_t marked;
    uint8_t forwarded;
    int forward_addr;
    int field0;
    int field1;
    int size;
};

struct lr_gc_heap {
    struct lr_gc_object objects[512];
    int count;
    int roots[32];
    int root_count;
    int live_count;
    int collected;
};

void lr_gc_init(struct lr_gc_heap *heap) {
    heap->count = 0;
    heap->root_count = 0;
    heap->live_count = 0;
    heap->collected = 0;
}

int lr_gc_alloc(struct lr_gc_heap *heap, enum lr_gc_obj_type otype, int f0, int f1) {
    if (heap->count >= 512) return -1;
    int idx = heap->count;
    heap->objects[idx].otype = otype;
    heap->objects[idx].marked = 0;
    heap->objects[idx].forwarded = 0;
    heap->objects[idx].forward_addr = -1;
    heap->objects[idx].field0 = f0;
    heap->objects[idx].field1 = f1;
    heap->objects[idx].size = 1;
    heap->count++;
    return idx;
}

int lr_gc_add_root(struct lr_gc_heap *heap, int obj_idx) {
    if (heap->root_count >= 32) return -1;
    heap->roots[heap->root_count] = obj_idx;
    heap->root_count++;
    return 0;
}

static void lr_gc_mark(struct lr_gc_heap *heap, int idx) {
    if (idx < 0 || idx >= heap->count) return;
    if (heap->objects[idx].marked) return;

    heap->objects[idx].marked = 1;

    if (heap->objects[idx].otype == LR_GC_PAIR ||
        heap->objects[idx].otype == LR_GC_CLOSURE) {
        lr_gc_mark(heap, heap->objects[idx].field0);
        lr_gc_mark(heap, heap->objects[idx].field1);
    }
    if (heap->objects[idx].otype == LR_GC_ARRAY) {
        lr_gc_mark(heap, heap->objects[idx].field0);
    }
}

static void lr_gc_mark_roots(struct lr_gc_heap *heap) {
    int i;
    for (i = 0; i < heap->root_count; i++) {
        lr_gc_mark(heap, heap->roots[i]);
    }
}

static void lr_gc_compute_forwards(struct lr_gc_heap *heap) {
    int dest = 0;
    int i;
    for (i = 0; i < heap->count; i++) {
        if (heap->objects[i].marked) {
            heap->objects[i].forward_addr = dest;
            dest++;
        }
    }
    heap->live_count = dest;
}

static int lr_gc_remap(struct lr_gc_heap *heap, int idx) {
    if (idx < 0 || idx >= heap->count) return idx;
    if (heap->objects[idx].marked) {
        return heap->objects[idx].forward_addr;
    }
    return -1;
}

static void lr_gc_update_refs(struct lr_gc_heap *heap) {
    int i;
    for (i = 0; i < heap->count; i++) {
        if (!heap->objects[i].marked) continue;
        struct lr_gc_object *obj = &heap->objects[i];
        if (obj->otype == LR_GC_PAIR || obj->otype == LR_GC_CLOSURE) {
            obj->field0 = lr_gc_remap(heap, obj->field0);
            obj->field1 = lr_gc_remap(heap, obj->field1);
        }
        if (obj->otype == LR_GC_ARRAY) {
            obj->field0 = lr_gc_remap(heap, obj->field0);
        }
    }
    for (i = 0; i < heap->root_count; i++) {
        heap->roots[i] = lr_gc_remap(heap, heap->roots[i]);
    }
}

static void lr_gc_compact(struct lr_gc_heap *heap) {
    int dest = 0;
    int i;
    for (i = 0; i < heap->count; i++) {
        if (heap->objects[i].marked) {
            if (dest != i) {
                heap->objects[dest] = heap->objects[i];
            }
            heap->objects[dest].marked = 0;
            heap->objects[dest].forwarded = 0;
            heap->objects[dest].forward_addr = -1;
            dest++;
        }
    }
    heap->collected = heap->count - dest;
    heap->count = dest;
}

int lr_gc_collect(struct lr_gc_heap *heap) {
    int i;
    for (i = 0; i < heap->count; i++) {
        heap->objects[i].marked = 0;
        heap->objects[i].forwarded = 0;
    }

    lr_gc_mark_roots(heap);
    lr_gc_compute_forwards(heap);
    lr_gc_update_refs(heap);
    lr_gc_compact(heap);

    return heap->collected;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C762: Mark-compact GC - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C762: empty output");
    assert!(
        code.contains("fn lr_gc_collect"),
        "C762: Should contain lr_gc_collect function"
    );
}

/// C763: String interning table
#[test]
fn c763_string_interning_table() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned int uint32_t;

struct lr_interned_str {
    char data[64];
    int length;
    uint32_t hash;
    int next;
};

struct lr_intern_table {
    struct lr_interned_str strings[256];
    int str_count;
    int buckets[64];
    int bucket_count;
    int total_lookups;
    int total_hits;
};

static uint32_t lr_intern_hash(const char *str, int len) {
    uint32_t h = 2166136261u;
    int i;
    for (i = 0; i < len; i++) {
        h = h ^ (uint32_t)(unsigned char)str[i];
        h = h * 16777619u;
    }
    return h;
}

void lr_intern_init(struct lr_intern_table *tab) {
    int i;
    tab->str_count = 0;
    tab->bucket_count = 64;
    tab->total_lookups = 0;
    tab->total_hits = 0;
    for (i = 0; i < 64; i++) {
        tab->buckets[i] = -1;
    }
}

static int lr_intern_str_eq(const char *a, int alen, const char *b, int blen) {
    int i;
    if (alen != blen) return 0;
    for (i = 0; i < alen; i++) {
        if (a[i] != b[i]) return 0;
    }
    return 1;
}

int lr_intern(struct lr_intern_table *tab, const char *str, int len) {
    uint32_t h = lr_intern_hash(str, len);
    int bucket = (int)(h % (uint32_t)tab->bucket_count);

    tab->total_lookups++;

    int cur = tab->buckets[bucket];
    while (cur >= 0) {
        if (tab->strings[cur].hash == h &&
            lr_intern_str_eq(tab->strings[cur].data, tab->strings[cur].length, str, len)) {
            tab->total_hits++;
            return cur;
        }
        cur = tab->strings[cur].next;
    }

    if (tab->str_count >= 256) return -1;
    if (len >= 64) return -1;

    int idx = tab->str_count;
    int i;
    for (i = 0; i < len; i++) {
        tab->strings[idx].data[i] = str[i];
    }
    tab->strings[idx].data[len] = '\0';
    tab->strings[idx].length = len;
    tab->strings[idx].hash = h;
    tab->strings[idx].next = tab->buckets[bucket];
    tab->buckets[bucket] = idx;
    tab->str_count++;
    return idx;
}

int lr_intern_lookup(struct lr_intern_table *tab, const char *str, int len) {
    uint32_t h = lr_intern_hash(str, len);
    int bucket = (int)(h % (uint32_t)tab->bucket_count);

    int cur = tab->buckets[bucket];
    while (cur >= 0) {
        if (tab->strings[cur].hash == h &&
            lr_intern_str_eq(tab->strings[cur].data, tab->strings[cur].length, str, len)) {
            return cur;
        }
        cur = tab->strings[cur].next;
    }
    return -1;
}

int lr_intern_count(struct lr_intern_table *tab) {
    return tab->str_count;
}

int lr_intern_hit_rate_pct(struct lr_intern_table *tab) {
    if (tab->total_lookups == 0) return 0;
    return (tab->total_hits * 100) / tab->total_lookups;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C763: String interning table - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C763: empty output");
    assert!(
        code.contains("fn lr_intern"),
        "C763: Should contain lr_intern function"
    );
}

// ============================================================================
// C764-C768: Dispatch/Control (VTable, Exception, Coroutine, JIT Buffer,
//            Profiler)
// ============================================================================

/// C764: Method dispatch (vtable)
#[test]
fn c764_method_dispatch_vtable() {
    let c_code = r#"
typedef int (*lr_method_fn)(int self_id, int arg);

struct lr_vtable {
    lr_method_fn methods[16];
    int method_count;
    int parent_vtable;
};

struct lr_obj_header {
    int vtable_id;
    int fields[8];
    int field_count;
    int ref_count;
};

struct lr_dispatch {
    struct lr_vtable vtables[32];
    int vtable_count;
    struct lr_obj_header objects[128];
    int obj_count;
};

void lr_dispatch_init(struct lr_dispatch *d) {
    int i;
    d->vtable_count = 0;
    d->obj_count = 0;
    for (i = 0; i < 32; i++) {
        int j;
        for (j = 0; j < 16; j++) {
            d->vtables[i].methods[j] = 0;
        }
        d->vtables[i].method_count = 0;
        d->vtables[i].parent_vtable = -1;
    }
}

int lr_dispatch_new_vtable(struct lr_dispatch *d, int parent) {
    if (d->vtable_count >= 32) return -1;
    int idx = d->vtable_count;
    d->vtables[idx].method_count = 0;
    d->vtables[idx].parent_vtable = parent;

    if (parent >= 0 && parent < d->vtable_count) {
        int i;
        for (i = 0; i < d->vtables[parent].method_count; i++) {
            d->vtables[idx].methods[i] = d->vtables[parent].methods[i];
        }
        d->vtables[idx].method_count = d->vtables[parent].method_count;
    }

    d->vtable_count++;
    return idx;
}

int lr_dispatch_set_method(struct lr_dispatch *d, int vt_id, int slot, lr_method_fn fn) {
    if (vt_id < 0 || vt_id >= d->vtable_count) return -1;
    if (slot < 0 || slot >= 16) return -1;
    d->vtables[vt_id].methods[slot] = fn;
    if (slot >= d->vtables[vt_id].method_count) {
        d->vtables[vt_id].method_count = slot + 1;
    }
    return 0;
}

int lr_dispatch_new_obj(struct lr_dispatch *d, int vt_id) {
    if (d->obj_count >= 128) return -1;
    int idx = d->obj_count;
    d->objects[idx].vtable_id = vt_id;
    d->objects[idx].field_count = 0;
    d->objects[idx].ref_count = 1;
    int i;
    for (i = 0; i < 8; i++) d->objects[idx].fields[i] = 0;
    d->obj_count++;
    return idx;
}

int lr_dispatch_call(struct lr_dispatch *d, int obj_id, int method_slot, int arg) {
    if (obj_id < 0 || obj_id >= d->obj_count) return -1;
    int vt = d->objects[obj_id].vtable_id;
    if (vt < 0 || vt >= d->vtable_count) return -1;
    if (method_slot < 0 || method_slot >= d->vtables[vt].method_count) return -1;
    lr_method_fn fn = d->vtables[vt].methods[method_slot];
    if (fn == 0) return -1;
    return fn(obj_id, arg);
}

int lr_dispatch_lookup(struct lr_dispatch *d, int vt_id, int method_slot) {
    while (vt_id >= 0 && vt_id < d->vtable_count) {
        if (method_slot < d->vtables[vt_id].method_count &&
            d->vtables[vt_id].methods[method_slot] != 0) {
            return vt_id;
        }
        vt_id = d->vtables[vt_id].parent_vtable;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C764: Method dispatch (vtable) - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C764: empty output");
    assert!(
        code.contains("fn lr_dispatch_call"),
        "C764: Should contain lr_dispatch_call function"
    );
}

/// C765: Exception/longjmp handler simulation
#[test]
fn c765_exception_handler() {
    let c_code = r#"
enum lr_exc_type {
    LR_EXC_NONE = 0,
    LR_EXC_RUNTIME,
    LR_EXC_TYPE_ERROR,
    LR_EXC_OVERFLOW,
    LR_EXC_DIV_ZERO,
    LR_EXC_INDEX_OOB,
    LR_EXC_NULL_PTR
};

struct lr_exc_frame {
    int handler_pc;
    int stack_level;
    int scope_level;
    int prev_frame;
};

struct lr_exc_state {
    struct lr_exc_frame frames[32];
    int frame_count;
    int current_frame;
    enum lr_exc_type pending_type;
    int pending_value;
    int is_unwinding;
    int caught_count;
    int uncaught_count;
};

void lr_exc_init(struct lr_exc_state *es) {
    es->frame_count = 0;
    es->current_frame = -1;
    es->pending_type = LR_EXC_NONE;
    es->pending_value = 0;
    es->is_unwinding = 0;
    es->caught_count = 0;
    es->uncaught_count = 0;
}

int lr_exc_push_handler(struct lr_exc_state *es, int handler_pc, int stack_lvl, int scope_lvl) {
    if (es->frame_count >= 32) return -1;
    int idx = es->frame_count;
    es->frames[idx].handler_pc = handler_pc;
    es->frames[idx].stack_level = stack_lvl;
    es->frames[idx].scope_level = scope_lvl;
    es->frames[idx].prev_frame = es->current_frame;
    es->current_frame = idx;
    es->frame_count++;
    return idx;
}

void lr_exc_pop_handler(struct lr_exc_state *es) {
    if (es->current_frame >= 0) {
        es->current_frame = es->frames[es->current_frame].prev_frame;
    }
}

int lr_exc_throw(struct lr_exc_state *es, enum lr_exc_type etype, int value) {
    es->pending_type = etype;
    es->pending_value = value;
    es->is_unwinding = 1;

    if (es->current_frame >= 0) {
        int frame = es->current_frame;
        int handler_pc = es->frames[frame].handler_pc;
        es->current_frame = es->frames[frame].prev_frame;
        es->is_unwinding = 0;
        es->caught_count++;
        return handler_pc;
    }

    es->uncaught_count++;
    return -1;
}

int lr_exc_is_pending(struct lr_exc_state *es) {
    return es->pending_type != LR_EXC_NONE;
}

void lr_exc_clear(struct lr_exc_state *es) {
    es->pending_type = LR_EXC_NONE;
    es->pending_value = 0;
    es->is_unwinding = 0;
}

int lr_exc_rethrow(struct lr_exc_state *es) {
    if (es->pending_type == LR_EXC_NONE) return -1;
    return lr_exc_throw(es, es->pending_type, es->pending_value);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C765: Exception handler - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C765: empty output");
    assert!(
        code.contains("fn lr_exc_throw"),
        "C765: Should contain lr_exc_throw function"
    );
}

/// C766: Coroutine/fiber scheduler
#[test]
fn c766_coroutine_fiber_scheduler() {
    let c_code = r#"
enum lr_fiber_state {
    LR_FIBER_READY = 0,
    LR_FIBER_RUNNING,
    LR_FIBER_SUSPENDED,
    LR_FIBER_FINISHED,
    LR_FIBER_ERROR
};

struct lr_fiber {
    int id;
    enum lr_fiber_state state;
    int pc;
    int sp;
    int stack[64];
    int yield_value;
    int priority;
    int ticks_remaining;
};

struct lr_scheduler {
    struct lr_fiber fibers[32];
    int fiber_count;
    int current_fiber;
    int run_queue[32];
    int run_queue_len;
    int total_context_switches;
    int time_slice;
};

void lr_sched_init(struct lr_scheduler *sched, int time_slice) {
    sched->fiber_count = 0;
    sched->current_fiber = -1;
    sched->run_queue_len = 0;
    sched->total_context_switches = 0;
    sched->time_slice = time_slice;
}

int lr_sched_create_fiber(struct lr_scheduler *sched, int start_pc, int priority) {
    if (sched->fiber_count >= 32) return -1;
    int idx = sched->fiber_count;
    sched->fibers[idx].id = idx;
    sched->fibers[idx].state = LR_FIBER_READY;
    sched->fibers[idx].pc = start_pc;
    sched->fibers[idx].sp = 0;
    sched->fibers[idx].yield_value = 0;
    sched->fibers[idx].priority = priority;
    sched->fibers[idx].ticks_remaining = sched->time_slice;

    sched->run_queue[sched->run_queue_len] = idx;
    sched->run_queue_len++;
    sched->fiber_count++;
    return idx;
}

static int lr_sched_pick_next(struct lr_scheduler *sched) {
    int best = -1;
    int best_pri = -1;
    int i;
    for (i = 0; i < sched->run_queue_len; i++) {
        int fib = sched->run_queue[i];
        if (sched->fibers[fib].state == LR_FIBER_READY ||
            sched->fibers[fib].state == LR_FIBER_SUSPENDED) {
            if (sched->fibers[fib].priority > best_pri) {
                best_pri = sched->fibers[fib].priority;
                best = i;
            }
        }
    }
    return best;
}

int lr_sched_yield(struct lr_scheduler *sched, int value) {
    if (sched->current_fiber < 0) return -1;
    struct lr_fiber *f = &sched->fibers[sched->current_fiber];
    f->state = LR_FIBER_SUSPENDED;
    f->yield_value = value;
    return 0;
}

int lr_sched_resume(struct lr_scheduler *sched, int fiber_id) {
    if (fiber_id < 0 || fiber_id >= sched->fiber_count) return -1;
    if (sched->fibers[fiber_id].state != LR_FIBER_SUSPENDED) return -1;
    sched->fibers[fiber_id].state = LR_FIBER_READY;
    sched->fibers[fiber_id].ticks_remaining = sched->time_slice;
    return 0;
}

int lr_sched_step(struct lr_scheduler *sched) {
    int next_idx = lr_sched_pick_next(sched);
    if (next_idx < 0) return -1;

    int fiber_id = sched->run_queue[next_idx];

    if (sched->current_fiber >= 0 && sched->current_fiber != fiber_id) {
        sched->total_context_switches++;
    }

    sched->current_fiber = fiber_id;
    sched->fibers[fiber_id].state = LR_FIBER_RUNNING;
    sched->fibers[fiber_id].ticks_remaining--;

    if (sched->fibers[fiber_id].ticks_remaining <= 0) {
        sched->fibers[fiber_id].state = LR_FIBER_SUSPENDED;
        sched->fibers[fiber_id].ticks_remaining = sched->time_slice;
    }

    return fiber_id;
}

int lr_sched_finish(struct lr_scheduler *sched, int fiber_id) {
    if (fiber_id < 0 || fiber_id >= sched->fiber_count) return -1;
    sched->fibers[fiber_id].state = LR_FIBER_FINISHED;
    return 0;
}

int lr_sched_active_count(struct lr_scheduler *sched) {
    int count = 0;
    int i;
    for (i = 0; i < sched->fiber_count; i++) {
        if (sched->fibers[i].state != LR_FIBER_FINISHED &&
            sched->fibers[i].state != LR_FIBER_ERROR) {
            count++;
        }
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C766: Coroutine/fiber scheduler - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C766: empty output");
    assert!(
        code.contains("fn lr_sched_step"),
        "C766: Should contain lr_sched_step function"
    );
}

/// C767: JIT code buffer manager
#[test]
fn c767_jit_code_buffer_manager() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef unsigned long size_t;

struct lr_jit_block {
    uint8_t code[256];
    int size;
    int capacity;
    int func_id;
    int is_valid;
    int exec_count;
};

struct lr_jit_buffer {
    struct lr_jit_block blocks[64];
    int block_count;
    int total_bytes_emitted;
    int total_patches;
};

void lr_jit_init(struct lr_jit_buffer *jit) {
    jit->block_count = 0;
    jit->total_bytes_emitted = 0;
    jit->total_patches = 0;
}

int lr_jit_new_block(struct lr_jit_buffer *jit, int func_id) {
    if (jit->block_count >= 64) return -1;
    int idx = jit->block_count;
    jit->blocks[idx].size = 0;
    jit->blocks[idx].capacity = 256;
    jit->blocks[idx].func_id = func_id;
    jit->blocks[idx].is_valid = 1;
    jit->blocks[idx].exec_count = 0;
    jit->block_count++;
    return idx;
}

int lr_jit_emit_byte(struct lr_jit_buffer *jit, int block_id, uint8_t byte) {
    if (block_id < 0 || block_id >= jit->block_count) return -1;
    struct lr_jit_block *b = &jit->blocks[block_id];
    if (b->size >= b->capacity) return -1;
    b->code[b->size] = byte;
    b->size++;
    jit->total_bytes_emitted++;
    return 0;
}

int lr_jit_emit_u32(struct lr_jit_buffer *jit, int block_id, uint32_t val) {
    int r = 0;
    r |= lr_jit_emit_byte(jit, block_id, (uint8_t)(val & 0xFF));
    r |= lr_jit_emit_byte(jit, block_id, (uint8_t)((val >> 8) & 0xFF));
    r |= lr_jit_emit_byte(jit, block_id, (uint8_t)((val >> 16) & 0xFF));
    r |= lr_jit_emit_byte(jit, block_id, (uint8_t)((val >> 24) & 0xFF));
    return r;
}

int lr_jit_patch_u32(struct lr_jit_buffer *jit, int block_id, int offset, uint32_t val) {
    if (block_id < 0 || block_id >= jit->block_count) return -1;
    struct lr_jit_block *b = &jit->blocks[block_id];
    if (offset < 0 || offset + 4 > b->size) return -1;
    b->code[offset] = (uint8_t)(val & 0xFF);
    b->code[offset + 1] = (uint8_t)((val >> 8) & 0xFF);
    b->code[offset + 2] = (uint8_t)((val >> 16) & 0xFF);
    b->code[offset + 3] = (uint8_t)((val >> 24) & 0xFF);
    jit->total_patches++;
    return 0;
}

void lr_jit_invalidate(struct lr_jit_buffer *jit, int block_id) {
    if (block_id >= 0 && block_id < jit->block_count) {
        jit->blocks[block_id].is_valid = 0;
    }
}

int lr_jit_find_block(struct lr_jit_buffer *jit, int func_id) {
    int i;
    for (i = 0; i < jit->block_count; i++) {
        if (jit->blocks[i].func_id == func_id && jit->blocks[i].is_valid) {
            return i;
        }
    }
    return -1;
}

int lr_jit_total_code_size(struct lr_jit_buffer *jit) {
    int total = 0;
    int i;
    for (i = 0; i < jit->block_count; i++) {
        if (jit->blocks[i].is_valid) {
            total += jit->blocks[i].size;
        }
    }
    return total;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C767: JIT code buffer manager - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C767: empty output");
    assert!(
        code.contains("fn lr_jit_emit_byte"),
        "C767: Should contain lr_jit_emit_byte function"
    );
}

/// C768: Profiler/call graph builder
#[test]
fn c768_profiler_call_graph_builder() {
    let c_code = r#"
typedef unsigned long uint64_t;
typedef unsigned int uint32_t;

struct lr_prof_entry {
    int func_id;
    uint64_t call_count;
    uint64_t total_ticks;
    uint64_t self_ticks;
    int callers[8];
    int caller_count;
    int callees[8];
    int callee_count;
};

struct lr_profiler {
    struct lr_prof_entry entries[128];
    int entry_count;
    int call_stack[64];
    int call_stack_depth;
    uint64_t current_tick;
    uint64_t enter_tick[64];
};

void lr_prof_init(struct lr_profiler *prof) {
    prof->entry_count = 0;
    prof->call_stack_depth = 0;
    prof->current_tick = 0;
}

static int lr_prof_find_or_create(struct lr_profiler *prof, int func_id) {
    int i;
    for (i = 0; i < prof->entry_count; i++) {
        if (prof->entries[i].func_id == func_id) return i;
    }
    if (prof->entry_count >= 128) return -1;
    int idx = prof->entry_count;
    prof->entries[idx].func_id = func_id;
    prof->entries[idx].call_count = 0;
    prof->entries[idx].total_ticks = 0;
    prof->entries[idx].self_ticks = 0;
    prof->entries[idx].caller_count = 0;
    prof->entries[idx].callee_count = 0;
    prof->entry_count++;
    return idx;
}

static void lr_prof_add_edge(int *arr, int *count, int max, int val) {
    int i;
    for (i = 0; i < *count; i++) {
        if (arr[i] == val) return;
    }
    if (*count < max) {
        arr[*count] = val;
        (*count)++;
    }
}

void lr_prof_enter(struct lr_profiler *prof, int func_id) {
    int eidx = lr_prof_find_or_create(prof, func_id);
    if (eidx < 0) return;

    prof->entries[eidx].call_count++;

    if (prof->call_stack_depth > 0) {
        int caller = prof->call_stack[prof->call_stack_depth - 1];
        int cidx = lr_prof_find_or_create(prof, caller);
        if (cidx >= 0) {
            lr_prof_add_edge(prof->entries[cidx].callees,
                           &prof->entries[cidx].callee_count, 8, func_id);
            lr_prof_add_edge(prof->entries[eidx].callers,
                           &prof->entries[eidx].caller_count, 8, caller);
        }
    }

    if (prof->call_stack_depth < 64) {
        prof->call_stack[prof->call_stack_depth] = func_id;
        prof->enter_tick[prof->call_stack_depth] = prof->current_tick;
        prof->call_stack_depth++;
    }
}

void lr_prof_exit(struct lr_profiler *prof, int func_id) {
    if (prof->call_stack_depth <= 0) return;

    prof->call_stack_depth--;
    uint64_t elapsed = prof->current_tick - prof->enter_tick[prof->call_stack_depth];

    int eidx = lr_prof_find_or_create(prof, func_id);
    if (eidx >= 0) {
        prof->entries[eidx].total_ticks += elapsed;
        prof->entries[eidx].self_ticks += elapsed;
    }
}

void lr_prof_tick(struct lr_profiler *prof, uint64_t ticks) {
    prof->current_tick += ticks;
}

int lr_prof_hottest(struct lr_profiler *prof) {
    int best = -1;
    uint64_t best_ticks = 0;
    int i;
    for (i = 0; i < prof->entry_count; i++) {
        if (prof->entries[i].self_ticks > best_ticks) {
            best_ticks = prof->entries[i].self_ticks;
            best = prof->entries[i].func_id;
        }
    }
    return best;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C768: Profiler/call graph builder - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C768: empty output");
    assert!(
        code.contains("fn lr_prof_enter"),
        "C768: Should contain lr_prof_enter function"
    );
}

// ============================================================================
// C769-C775: Infrastructure (Debug Info, Module Loader, Regex NFA,
//            Pattern Match, Instruction Codec, Peephole Opt, Stack Frame)
// ============================================================================

/// C769: Debug info table
#[test]
fn c769_debug_info_table() {
    let c_code = r#"
struct lr_debug_line {
    int pc;
    int source_line;
    int source_col;
    int file_id;
};

struct lr_debug_local {
    char name[32];
    int scope_start_pc;
    int scope_end_pc;
    int slot;
    int type_id;
};

struct lr_debug_info {
    struct lr_debug_line lines[512];
    int line_count;
    struct lr_debug_local locals[128];
    int local_count;
    int func_start_pc;
    int func_end_pc;
    int func_name_id;
};

struct lr_debug_table {
    struct lr_debug_info funcs[32];
    int func_count;
};

void lr_debug_init(struct lr_debug_table *dt) {
    dt->func_count = 0;
}

int lr_debug_add_func(struct lr_debug_table *dt, int name_id, int start_pc) {
    if (dt->func_count >= 32) return -1;
    int idx = dt->func_count;
    dt->funcs[idx].func_name_id = name_id;
    dt->funcs[idx].func_start_pc = start_pc;
    dt->funcs[idx].func_end_pc = start_pc;
    dt->funcs[idx].line_count = 0;
    dt->funcs[idx].local_count = 0;
    dt->func_count++;
    return idx;
}

int lr_debug_add_line(struct lr_debug_table *dt, int func_idx, int pc, int line, int col, int file) {
    if (func_idx < 0 || func_idx >= dt->func_count) return -1;
    struct lr_debug_info *di = &dt->funcs[func_idx];
    if (di->line_count >= 512) return -1;
    int idx = di->line_count;
    di->lines[idx].pc = pc;
    di->lines[idx].source_line = line;
    di->lines[idx].source_col = col;
    di->lines[idx].file_id = file;
    di->line_count++;
    if (pc > di->func_end_pc) di->func_end_pc = pc;
    return idx;
}

static void lr_debug_str_copy(char *dst, const char *src, int max) {
    int i = 0;
    while (i < max - 1 && src[i] != '\0') {
        dst[i] = src[i];
        i++;
    }
    dst[i] = '\0';
}

int lr_debug_add_local(struct lr_debug_table *dt, int func_idx, const char *name,
                       int start_pc, int end_pc, int slot, int type_id) {
    if (func_idx < 0 || func_idx >= dt->func_count) return -1;
    struct lr_debug_info *di = &dt->funcs[func_idx];
    if (di->local_count >= 128) return -1;
    int idx = di->local_count;
    lr_debug_str_copy(di->locals[idx].name, name, 32);
    di->locals[idx].scope_start_pc = start_pc;
    di->locals[idx].scope_end_pc = end_pc;
    di->locals[idx].slot = slot;
    di->locals[idx].type_id = type_id;
    di->local_count++;
    return idx;
}

int lr_debug_find_line(struct lr_debug_table *dt, int pc) {
    int i, j;
    for (i = 0; i < dt->func_count; i++) {
        struct lr_debug_info *di = &dt->funcs[i];
        if (pc < di->func_start_pc || pc > di->func_end_pc) continue;
        int best_line = -1;
        int best_pc = -1;
        for (j = 0; j < di->line_count; j++) {
            if (di->lines[j].pc <= pc && di->lines[j].pc > best_pc) {
                best_pc = di->lines[j].pc;
                best_line = di->lines[j].source_line;
            }
        }
        if (best_line >= 0) return best_line;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C769: Debug info table - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C769: empty output");
    assert!(
        code.contains("fn lr_debug_find_line"),
        "C769: Should contain lr_debug_find_line function"
    );
}

/// C770: Module loader with dependency resolution
#[test]
fn c770_module_loader_dependency_resolution() {
    let c_code = r#"
enum lr_mod_state {
    LR_MOD_UNLOADED = 0,
    LR_MOD_LOADING,
    LR_MOD_LOADED,
    LR_MOD_ERROR
};

struct lr_module {
    char name[32];
    enum lr_mod_state state;
    int deps[8];
    int dep_count;
    int export_count;
    int load_order;
};

struct lr_mod_loader {
    struct lr_module modules[64];
    int mod_count;
    int load_order_counter;
    int circular_detected;
};

static int lr_mod_str_eq(const char *a, const char *b) {
    int i = 0;
    while (a[i] != '\0' && b[i] != '\0') {
        if (a[i] != b[i]) return 0;
        i++;
    }
    return a[i] == b[i];
}

static void lr_mod_str_copy(char *dst, const char *src, int max) {
    int i = 0;
    while (i < max - 1 && src[i] != '\0') {
        dst[i] = src[i];
        i++;
    }
    dst[i] = '\0';
}

void lr_mod_init(struct lr_mod_loader *loader) {
    loader->mod_count = 0;
    loader->load_order_counter = 0;
    loader->circular_detected = 0;
}

int lr_mod_register(struct lr_mod_loader *loader, const char *name) {
    int i;
    for (i = 0; i < loader->mod_count; i++) {
        if (lr_mod_str_eq(loader->modules[i].name, name)) return i;
    }
    if (loader->mod_count >= 64) return -1;
    int idx = loader->mod_count;
    lr_mod_str_copy(loader->modules[idx].name, name, 32);
    loader->modules[idx].state = LR_MOD_UNLOADED;
    loader->modules[idx].dep_count = 0;
    loader->modules[idx].export_count = 0;
    loader->modules[idx].load_order = -1;
    loader->mod_count++;
    return idx;
}

int lr_mod_add_dep(struct lr_mod_loader *loader, int mod_id, int dep_id) {
    if (mod_id < 0 || mod_id >= loader->mod_count) return -1;
    if (dep_id < 0 || dep_id >= loader->mod_count) return -1;
    struct lr_module *m = &loader->modules[mod_id];
    if (m->dep_count >= 8) return -1;
    m->deps[m->dep_count] = dep_id;
    m->dep_count++;
    return 0;
}

static int lr_mod_load_recursive(struct lr_mod_loader *loader, int mod_id) {
    if (mod_id < 0 || mod_id >= loader->mod_count) return -1;
    struct lr_module *m = &loader->modules[mod_id];

    if (m->state == LR_MOD_LOADED) return 0;
    if (m->state == LR_MOD_LOADING) {
        loader->circular_detected = 1;
        m->state = LR_MOD_ERROR;
        return -2;
    }

    m->state = LR_MOD_LOADING;

    int i;
    for (i = 0; i < m->dep_count; i++) {
        int result = lr_mod_load_recursive(loader, m->deps[i]);
        if (result < 0) {
            m->state = LR_MOD_ERROR;
            return result;
        }
    }

    m->state = LR_MOD_LOADED;
    m->load_order = loader->load_order_counter;
    loader->load_order_counter++;
    return 0;
}

int lr_mod_load(struct lr_mod_loader *loader, int mod_id) {
    return lr_mod_load_recursive(loader, mod_id);
}

int lr_mod_load_all(struct lr_mod_loader *loader) {
    int i;
    int failures = 0;
    for (i = 0; i < loader->mod_count; i++) {
        if (loader->modules[i].state == LR_MOD_UNLOADED) {
            int r = lr_mod_load_recursive(loader, i);
            if (r < 0) failures++;
        }
    }
    return failures;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C770: Module loader - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C770: empty output");
    assert!(
        code.contains("fn lr_mod_load"),
        "C770: Should contain lr_mod_load function"
    );
}

/// C771: Regular expression compiler to NFA
#[test]
fn c771_regex_compiler_to_nfa() {
    let c_code = r#"
enum lr_nfa_type {
    LR_NFA_MATCH = 0,
    LR_NFA_LITERAL,
    LR_NFA_DOT,
    LR_NFA_SPLIT,
    LR_NFA_JUMP
};

struct lr_nfa_state {
    enum lr_nfa_type stype;
    char literal;
    int out1;
    int out2;
};

struct lr_nfa {
    struct lr_nfa_state states[256];
    int state_count;
    int start;
};

void lr_nfa_init(struct lr_nfa *nfa) {
    nfa->state_count = 0;
    nfa->start = -1;
}

static int lr_nfa_add_state(struct lr_nfa *nfa, enum lr_nfa_type t, char lit, int o1, int o2) {
    if (nfa->state_count >= 256) return -1;
    int idx = nfa->state_count;
    nfa->states[idx].stype = t;
    nfa->states[idx].literal = lit;
    nfa->states[idx].out1 = o1;
    nfa->states[idx].out2 = o2;
    nfa->state_count++;
    return idx;
}

int lr_nfa_compile(struct lr_nfa *nfa, const char *pattern, int len) {
    int stack[64];
    int stack_top = 0;
    int i;

    int match_state = lr_nfa_add_state(nfa, LR_NFA_MATCH, '\0', -1, -1);

    for (i = len - 1; i >= 0; i--) {
        char c = pattern[i];
        int next = (stack_top > 0) ? stack[stack_top - 1] : match_state;

        if (c == '*' && i > 0) {
            i--;
            char prev = pattern[i];
            int split = lr_nfa_add_state(nfa, LR_NFA_SPLIT, '\0', -1, next);
            int body;
            if (prev == '.') {
                body = lr_nfa_add_state(nfa, LR_NFA_DOT, '\0', split, -1);
            } else {
                body = lr_nfa_add_state(nfa, LR_NFA_LITERAL, prev, split, -1);
            }
            nfa->states[split].out1 = body;
            if (stack_top > 0) stack_top--;
            stack[stack_top] = split;
            stack_top++;
        } else if (c == '?') {
            if (i > 0) {
                i--;
                char prev = pattern[i];
                int split = lr_nfa_add_state(nfa, LR_NFA_SPLIT, '\0', -1, next);
                int body;
                if (prev == '.') {
                    body = lr_nfa_add_state(nfa, LR_NFA_DOT, '\0', next, -1);
                } else {
                    body = lr_nfa_add_state(nfa, LR_NFA_LITERAL, prev, next, -1);
                }
                nfa->states[split].out1 = body;
                if (stack_top > 0) stack_top--;
                stack[stack_top] = split;
                stack_top++;
            }
        } else if (c == '.') {
            int s = lr_nfa_add_state(nfa, LR_NFA_DOT, '\0', next, -1);
            if (stack_top > 0) stack_top--;
            stack[stack_top] = s;
            stack_top++;
        } else {
            int s = lr_nfa_add_state(nfa, LR_NFA_LITERAL, c, next, -1);
            if (stack_top > 0) stack_top--;
            stack[stack_top] = s;
            stack_top++;
        }
    }

    nfa->start = (stack_top > 0) ? stack[stack_top - 1] : match_state;
    return nfa->start;
}

static int lr_nfa_simulate_step(struct lr_nfa *nfa, int state, const char *input, int pos, int len) {
    if (state < 0 || state >= nfa->state_count) return 0;
    struct lr_nfa_state *s = &nfa->states[state];

    if (s->stype == LR_NFA_MATCH) return 1;
    if (pos >= len) return (s->stype == LR_NFA_MATCH) ? 1 : 0;

    switch (s->stype) {
        case LR_NFA_LITERAL:
            if (input[pos] == s->literal) {
                return lr_nfa_simulate_step(nfa, s->out1, input, pos + 1, len);
            }
            return 0;
        case LR_NFA_DOT:
            return lr_nfa_simulate_step(nfa, s->out1, input, pos + 1, len);
        case LR_NFA_SPLIT:
            if (lr_nfa_simulate_step(nfa, s->out1, input, pos, len)) return 1;
            return lr_nfa_simulate_step(nfa, s->out2, input, pos, len);
        case LR_NFA_JUMP:
            return lr_nfa_simulate_step(nfa, s->out1, input, pos, len);
        default:
            return 0;
    }
}

int lr_nfa_match(struct lr_nfa *nfa, const char *input, int len) {
    return lr_nfa_simulate_step(nfa, nfa->start, input, 0, len);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C771: Regex compiler to NFA - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C771: empty output");
    assert!(
        code.contains("fn lr_nfa_compile"),
        "C771: Should contain lr_nfa_compile function"
    );
}

/// C772: Pattern matching compiler
#[test]
fn c772_pattern_matching_compiler() {
    let c_code = r#"
enum lr_pat_kind {
    LR_PAT_WILDCARD = 0,
    LR_PAT_INT_LIT,
    LR_PAT_VAR_BIND,
    LR_PAT_TUPLE,
    LR_PAT_CTOR,
    LR_PAT_OR,
    LR_PAT_GUARD
};

struct lr_pattern {
    enum lr_pat_kind kind;
    int int_val;
    int var_slot;
    int children[4];
    int child_count;
    int ctor_tag;
    int guard_expr;
};

struct lr_match_arm {
    int pattern_idx;
    int body_pc;
    int is_reachable;
};

struct lr_pattern_compiler {
    struct lr_pattern patterns[128];
    int pattern_count;
    struct lr_match_arm arms[32];
    int arm_count;
    int exhaustive;
    int redundant_count;
};

void lr_patc_init(struct lr_pattern_compiler *pc) {
    pc->pattern_count = 0;
    pc->arm_count = 0;
    pc->exhaustive = 0;
    pc->redundant_count = 0;
}

int lr_patc_add_pattern(struct lr_pattern_compiler *pc, enum lr_pat_kind kind) {
    if (pc->pattern_count >= 128) return -1;
    int idx = pc->pattern_count;
    pc->patterns[idx].kind = kind;
    pc->patterns[idx].int_val = 0;
    pc->patterns[idx].var_slot = -1;
    pc->patterns[idx].child_count = 0;
    pc->patterns[idx].ctor_tag = -1;
    pc->patterns[idx].guard_expr = -1;
    pc->pattern_count++;
    return idx;
}

int lr_patc_add_child(struct lr_pattern_compiler *pc, int pat_idx, int child_idx) {
    if (pat_idx < 0 || pat_idx >= pc->pattern_count) return -1;
    struct lr_pattern *p = &pc->patterns[pat_idx];
    if (p->child_count >= 4) return -1;
    p->children[p->child_count] = child_idx;
    p->child_count++;
    return 0;
}

int lr_patc_add_arm(struct lr_pattern_compiler *pc, int pat_idx, int body_pc) {
    if (pc->arm_count >= 32) return -1;
    int idx = pc->arm_count;
    pc->arms[idx].pattern_idx = pat_idx;
    pc->arms[idx].body_pc = body_pc;
    pc->arms[idx].is_reachable = 1;
    pc->arm_count++;
    return idx;
}

static int lr_patc_subsumes(struct lr_pattern_compiler *pc, int a_idx, int b_idx) {
    if (a_idx < 0 || b_idx < 0) return 0;
    struct lr_pattern *a = &pc->patterns[a_idx];
    struct lr_pattern *b = &pc->patterns[b_idx];

    if (a->kind == LR_PAT_WILDCARD) return 1;
    if (a->kind == LR_PAT_VAR_BIND) return 1;

    if (a->kind != b->kind) return 0;

    if (a->kind == LR_PAT_INT_LIT) {
        return a->int_val == b->int_val;
    }

    if (a->kind == LR_PAT_CTOR) {
        if (a->ctor_tag != b->ctor_tag) return 0;
        if (a->child_count != b->child_count) return 0;
        int i;
        for (i = 0; i < a->child_count; i++) {
            if (!lr_patc_subsumes(pc, a->children[i], b->children[i])) return 0;
        }
        return 1;
    }

    return 0;
}

void lr_patc_check_redundancy(struct lr_pattern_compiler *pc) {
    int i, j;
    pc->redundant_count = 0;
    for (i = 1; i < pc->arm_count; i++) {
        for (j = 0; j < i; j++) {
            if (pc->arms[j].is_reachable &&
                lr_patc_subsumes(pc, pc->arms[j].pattern_idx, pc->arms[i].pattern_idx)) {
                pc->arms[i].is_reachable = 0;
                pc->redundant_count++;
                break;
            }
        }
    }
}

int lr_patc_has_wildcard(struct lr_pattern_compiler *pc) {
    int i;
    for (i = 0; i < pc->arm_count; i++) {
        int pidx = pc->arms[i].pattern_idx;
        if (pidx >= 0 && pidx < pc->pattern_count) {
            if (pc->patterns[pidx].kind == LR_PAT_WILDCARD) return 1;
        }
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C772: Pattern matching compiler - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C772: empty output");
    assert!(
        code.contains("fn lr_patc_check_redundancy"),
        "C772: Should contain lr_patc_check_redundancy function"
    );
}

/// C773: Instruction decoder/encoder
#[test]
fn c773_instruction_decoder_encoder() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;

enum lr_enc_op {
    LR_ENC_NOP = 0,
    LR_ENC_LOAD_CONST,
    LR_ENC_LOAD_LOCAL,
    LR_ENC_STORE_LOCAL,
    LR_ENC_ADD,
    LR_ENC_SUB,
    LR_ENC_MUL,
    LR_ENC_DIV,
    LR_ENC_JUMP,
    LR_ENC_JUMP_IF,
    LR_ENC_CALL,
    LR_ENC_RET,
    LR_ENC_CMP_EQ,
    LR_ENC_CMP_LT,
    LR_ENC_PUSH,
    LR_ENC_POP
};

struct lr_instruction {
    enum lr_enc_op opcode;
    uint16_t arg1;
    uint16_t arg2;
    uint8_t width;
};

struct lr_encoder {
    uint8_t buffer[1024];
    int pos;
    int instruction_count;
};

void lr_enc_init(struct lr_encoder *enc) {
    enc->pos = 0;
    enc->instruction_count = 0;
}

static int lr_enc_needs_wide(enum lr_enc_op op) {
    return op == LR_ENC_JUMP || op == LR_ENC_JUMP_IF || op == LR_ENC_CALL;
}

int lr_enc_encode(struct lr_encoder *enc, struct lr_instruction *inst) {
    if (enc->pos >= 1020) return -1;

    enc->buffer[enc->pos] = (uint8_t)inst->opcode;
    enc->pos++;

    if (inst->opcode == LR_ENC_NOP || inst->opcode == LR_ENC_ADD ||
        inst->opcode == LR_ENC_SUB || inst->opcode == LR_ENC_MUL ||
        inst->opcode == LR_ENC_DIV || inst->opcode == LR_ENC_RET ||
        inst->opcode == LR_ENC_POP || inst->opcode == LR_ENC_CMP_EQ ||
        inst->opcode == LR_ENC_CMP_LT) {
        inst->width = 1;
    } else if (lr_enc_needs_wide(inst->opcode)) {
        enc->buffer[enc->pos] = (uint8_t)(inst->arg1 & 0xFF);
        enc->buffer[enc->pos + 1] = (uint8_t)((inst->arg1 >> 8) & 0xFF);
        enc->buffer[enc->pos + 2] = (uint8_t)(inst->arg2 & 0xFF);
        enc->buffer[enc->pos + 3] = (uint8_t)((inst->arg2 >> 8) & 0xFF);
        enc->pos += 4;
        inst->width = 5;
    } else {
        enc->buffer[enc->pos] = (uint8_t)(inst->arg1 & 0xFF);
        enc->pos++;
        inst->width = 2;
    }

    enc->instruction_count++;
    return 0;
}

int lr_enc_decode(uint8_t *buffer, int pos, int len, struct lr_instruction *inst) {
    if (pos >= len) return -1;
    inst->opcode = (enum lr_enc_op)buffer[pos];
    inst->arg1 = 0;
    inst->arg2 = 0;
    pos++;

    if (inst->opcode == LR_ENC_NOP || inst->opcode == LR_ENC_ADD ||
        inst->opcode == LR_ENC_SUB || inst->opcode == LR_ENC_MUL ||
        inst->opcode == LR_ENC_DIV || inst->opcode == LR_ENC_RET ||
        inst->opcode == LR_ENC_POP || inst->opcode == LR_ENC_CMP_EQ ||
        inst->opcode == LR_ENC_CMP_LT) {
        inst->width = 1;
    } else if (lr_enc_needs_wide(inst->opcode)) {
        if (pos + 4 > len) return -1;
        inst->arg1 = (uint16_t)buffer[pos] | ((uint16_t)buffer[pos + 1] << 8);
        inst->arg2 = (uint16_t)buffer[pos + 2] | ((uint16_t)buffer[pos + 3] << 8);
        inst->width = 5;
    } else {
        if (pos >= len) return -1;
        inst->arg1 = (uint16_t)buffer[pos];
        inst->width = 2;
    }

    return inst->width;
}

int lr_enc_count_instructions(uint8_t *buffer, int len) {
    int count = 0;
    int pos = 0;
    while (pos < len) {
        struct lr_instruction inst;
        int w = lr_enc_decode(buffer, pos, len, &inst);
        if (w < 0) break;
        pos += w;
        count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C773: Instruction decoder/encoder - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C773: empty output");
    assert!(
        code.contains("fn lr_enc_encode"),
        "C773: Should contain lr_enc_encode function"
    );
}

/// C774: Peephole optimizer
#[test]
fn c774_peephole_optimizer() {
    let c_code = r#"
enum lr_peep_op {
    LR_PEEP_NOP = 0,
    LR_PEEP_LOAD,
    LR_PEEP_STORE,
    LR_PEEP_ADD,
    LR_PEEP_SUB,
    LR_PEEP_MUL,
    LR_PEEP_DIV,
    LR_PEEP_PUSH,
    LR_PEEP_POP,
    LR_PEEP_DUP,
    LR_PEEP_SWAP,
    LR_PEEP_JUMP,
    LR_PEEP_RET,
    LR_PEEP_CONST
};

struct lr_peep_inst {
    enum lr_peep_op op;
    int arg;
};

struct lr_peephole {
    struct lr_peep_inst input[512];
    int input_count;
    struct lr_peep_inst output[512];
    int output_count;
    int optimizations_applied;
};

void lr_peep_init(struct lr_peephole *ph) {
    ph->input_count = 0;
    ph->output_count = 0;
    ph->optimizations_applied = 0;
}

int lr_peep_add(struct lr_peephole *ph, enum lr_peep_op op, int arg) {
    if (ph->input_count >= 512) return -1;
    ph->input[ph->input_count].op = op;
    ph->input[ph->input_count].arg = arg;
    ph->input_count++;
    return 0;
}

static void lr_peep_emit(struct lr_peephole *ph, enum lr_peep_op op, int arg) {
    if (ph->output_count < 512) {
        ph->output[ph->output_count].op = op;
        ph->output[ph->output_count].arg = arg;
        ph->output_count++;
    }
}

void lr_peep_optimize(struct lr_peephole *ph) {
    int i = 0;
    ph->output_count = 0;
    ph->optimizations_applied = 0;

    while (i < ph->input_count) {
        if (i + 1 < ph->input_count &&
            ph->input[i].op == LR_PEEP_PUSH &&
            ph->input[i + 1].op == LR_PEEP_POP) {
            ph->optimizations_applied++;
            i += 2;
            continue;
        }

        if (i + 1 < ph->input_count &&
            ph->input[i].op == LR_PEEP_LOAD &&
            ph->input[i + 1].op == LR_PEEP_STORE &&
            ph->input[i].arg == ph->input[i + 1].arg) {
            lr_peep_emit(ph, LR_PEEP_LOAD, ph->input[i].arg);
            ph->optimizations_applied++;
            i += 2;
            continue;
        }

        if (i + 1 < ph->input_count &&
            ph->input[i].op == LR_PEEP_STORE &&
            ph->input[i + 1].op == LR_PEEP_LOAD &&
            ph->input[i].arg == ph->input[i + 1].arg) {
            lr_peep_emit(ph, LR_PEEP_STORE, ph->input[i].arg);
            lr_peep_emit(ph, LR_PEEP_DUP, 0);
            ph->optimizations_applied++;
            i += 2;
            continue;
        }

        if (ph->input[i].op == LR_PEEP_ADD && ph->input[i].arg == 0) {
            ph->optimizations_applied++;
            i++;
            continue;
        }

        if (ph->input[i].op == LR_PEEP_MUL && ph->input[i].arg == 1) {
            ph->optimizations_applied++;
            i++;
            continue;
        }

        if (ph->input[i].op == LR_PEEP_MUL && ph->input[i].arg == 0) {
            lr_peep_emit(ph, LR_PEEP_CONST, 0);
            ph->optimizations_applied++;
            i++;
            continue;
        }

        if (i + 1 < ph->input_count &&
            ph->input[i].op == LR_PEEP_JUMP &&
            ph->input[i + 1].op != LR_PEEP_NOP) {
            int target = ph->input[i].arg;
            if (target == i + 1) {
                ph->optimizations_applied++;
                i++;
                continue;
            }
        }

        if (ph->input[i].op == LR_PEEP_NOP) {
            ph->optimizations_applied++;
            i++;
            continue;
        }

        lr_peep_emit(ph, ph->input[i].op, ph->input[i].arg);
        i++;
    }
}

int lr_peep_reduction_pct(struct lr_peephole *ph) {
    if (ph->input_count == 0) return 0;
    int removed = ph->input_count - ph->output_count;
    return (removed * 100) / ph->input_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C774: Peephole optimizer - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C774: empty output");
    assert!(
        code.contains("fn lr_peep_optimize"),
        "C774: Should contain lr_peep_optimize function"
    );
}

/// C775: Stack frame layout manager
#[test]
fn c775_stack_frame_layout_manager() {
    let c_code = r#"
enum lr_frame_slot_kind {
    LR_SLOT_LOCAL = 0,
    LR_SLOT_PARAM,
    LR_SLOT_TEMP,
    LR_SLOT_SPILL,
    LR_SLOT_RETURN_ADDR,
    LR_SLOT_SAVED_REG
};

struct lr_frame_slot {
    enum lr_frame_slot_kind kind;
    int offset;
    int size;
    int alignment;
    int var_id;
    int is_live;
};

struct lr_frame_layout {
    struct lr_frame_slot slots[128];
    int slot_count;
    int total_size;
    int param_area_size;
    int local_area_size;
    int spill_area_size;
    int max_alignment;
};

void lr_frame_init(struct lr_frame_layout *fl) {
    fl->slot_count = 0;
    fl->total_size = 0;
    fl->param_area_size = 0;
    fl->local_area_size = 0;
    fl->spill_area_size = 0;
    fl->max_alignment = 4;
}

static int lr_frame_align_up(int offset, int alignment) {
    if (alignment <= 0) alignment = 1;
    return (offset + alignment - 1) & ~(alignment - 1);
}

int lr_frame_add_slot(struct lr_frame_layout *fl, enum lr_frame_slot_kind kind,
                      int size, int alignment, int var_id) {
    if (fl->slot_count >= 128) return -1;
    if (alignment > fl->max_alignment) fl->max_alignment = alignment;

    int idx = fl->slot_count;
    fl->slots[idx].kind = kind;
    fl->slots[idx].size = size;
    fl->slots[idx].alignment = alignment;
    fl->slots[idx].var_id = var_id;
    fl->slots[idx].is_live = 1;
    fl->slots[idx].offset = -1;
    fl->slot_count++;
    return idx;
}

void lr_frame_compute_layout(struct lr_frame_layout *fl) {
    int offset = 0;
    int i;

    offset = lr_frame_align_up(offset, fl->max_alignment);
    fl->slots[fl->slot_count - 1].offset = offset;

    int param_start = offset;
    for (i = 0; i < fl->slot_count; i++) {
        if (fl->slots[i].kind == LR_SLOT_PARAM) {
            offset = lr_frame_align_up(offset, fl->slots[i].alignment);
            fl->slots[i].offset = offset;
            offset += fl->slots[i].size;
        }
    }
    fl->param_area_size = offset - param_start;

    offset = lr_frame_align_up(offset, 4);
    for (i = 0; i < fl->slot_count; i++) {
        if (fl->slots[i].kind == LR_SLOT_SAVED_REG) {
            offset = lr_frame_align_up(offset, fl->slots[i].alignment);
            fl->slots[i].offset = offset;
            offset += fl->slots[i].size;
        }
    }

    int local_start = offset;
    for (i = 0; i < fl->slot_count; i++) {
        if (fl->slots[i].kind == LR_SLOT_LOCAL ||
            fl->slots[i].kind == LR_SLOT_TEMP) {
            offset = lr_frame_align_up(offset, fl->slots[i].alignment);
            fl->slots[i].offset = offset;
            offset += fl->slots[i].size;
        }
    }
    fl->local_area_size = offset - local_start;

    int spill_start = offset;
    for (i = 0; i < fl->slot_count; i++) {
        if (fl->slots[i].kind == LR_SLOT_SPILL) {
            offset = lr_frame_align_up(offset, fl->slots[i].alignment);
            fl->slots[i].offset = offset;
            offset += fl->slots[i].size;
        }
    }
    fl->spill_area_size = offset - spill_start;

    fl->total_size = lr_frame_align_up(offset, fl->max_alignment);
}

int lr_frame_find_slot(struct lr_frame_layout *fl, int var_id) {
    int i;
    for (i = 0; i < fl->slot_count; i++) {
        if (fl->slots[i].var_id == var_id) {
            return fl->slots[i].offset;
        }
    }
    return -1;
}

int lr_frame_live_size(struct lr_frame_layout *fl) {
    int total = 0;
    int i;
    for (i = 0; i < fl->slot_count; i++) {
        if (fl->slots[i].is_live) {
            total += fl->slots[i].size;
        }
    }
    return total;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C775: Stack frame layout manager - failed: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C775: empty output");
    assert!(
        code.contains("fn lr_frame_compute_layout"),
        "C775: Should contain lr_frame_compute_layout function"
    );
}
