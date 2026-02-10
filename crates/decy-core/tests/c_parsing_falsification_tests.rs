//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1276-C1300: Parsing & Grammar Algorithms -- the kind of C code found
//! in compilers, interpreters, text processors, configuration loaders,
//! network protocol handlers, and developer tools.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise expression parsers, lexical analyzers, grammar
//! parsers, specialized format parsers, and applied parsing systems --
//! all expressed as valid C99 with no includes, using the prs_ prefix
//! for all function and type names.
//!
//! Organization:
//! - C1276-C1280: Expression parsing (recursive descent, Pratt parser, shunting yard, postfix eval, infix to postfix)
//! - C1281-C1285: Lexical analysis (scanner/tokenizer, keyword recognition, number literal parser, string escape, comment stripper)
//! - C1286-C1290: Grammar parsing (LL(1) table-driven, LR(0) automaton, FIRST set computation, FOLLOW set, nullable)
//! - C1291-C1295: Specialized parsers (regex NFA builder, glob matcher, printf format parser, date/time parser, semantic version)
//! - C1296-C1300: Applied parsing (calculator with variables, config file parser, command-line parser, URI parser, MIME type parser)

// ============================================================================
// C1276-C1280: Expression Parsing
// ============================================================================

/// C1276: Recursive descent expression parser for arithmetic
#[test]
fn c1276_recursive_descent_expression_parser() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    const char *input;
    int pos;
    int length;
    int error;
} prs_rd_parser_t;

void prs_rd_init(prs_rd_parser_t *p, const char *input) {
    p->input = input;
    p->pos = 0;
    p->length = 0;
    p->error = 0;
    while (input[p->length] != '\0') {
        p->length++;
    }
}

char prs_rd_peek(prs_rd_parser_t *p) {
    if (p->pos >= p->length) return '\0';
    return p->input[p->pos];
}

char prs_rd_advance(prs_rd_parser_t *p) {
    char c = prs_rd_peek(p);
    if (c != '\0') p->pos++;
    return c;
}

void prs_rd_skip_spaces(prs_rd_parser_t *p) {
    while (prs_rd_peek(p) == ' ') prs_rd_advance(p);
}

int prs_rd_parse_expr(prs_rd_parser_t *p);
int prs_rd_parse_term(prs_rd_parser_t *p);
int prs_rd_parse_factor(prs_rd_parser_t *p);

int prs_rd_parse_factor(prs_rd_parser_t *p) {
    prs_rd_skip_spaces(p);
    char c = prs_rd_peek(p);
    if (c == '(') {
        prs_rd_advance(p);
        int val = prs_rd_parse_expr(p);
        prs_rd_skip_spaces(p);
        if (prs_rd_peek(p) == ')') prs_rd_advance(p);
        else p->error = 1;
        return val;
    }
    int sign = 1;
    if (c == '-') { sign = -1; prs_rd_advance(p); }
    int num = 0;
    while (prs_rd_peek(p) >= '0' && prs_rd_peek(p) <= '9') {
        num = num * 10 + (prs_rd_advance(p) - '0');
    }
    return sign * num;
}

int prs_rd_parse_term(prs_rd_parser_t *p) {
    int left = prs_rd_parse_factor(p);
    prs_rd_skip_spaces(p);
    while (prs_rd_peek(p) == '*' || prs_rd_peek(p) == '/') {
        char op = prs_rd_advance(p);
        int right = prs_rd_parse_factor(p);
        if (op == '*') left = left * right;
        else if (right != 0) left = left / right;
        else { p->error = 1; return 0; }
        prs_rd_skip_spaces(p);
    }
    return left;
}

int prs_rd_parse_expr(prs_rd_parser_t *p) {
    int left = prs_rd_parse_term(p);
    prs_rd_skip_spaces(p);
    while (prs_rd_peek(p) == '+' || prs_rd_peek(p) == '-') {
        char op = prs_rd_advance(p);
        int right = prs_rd_parse_term(p);
        if (op == '+') left = left + right;
        else left = left - right;
        prs_rd_skip_spaces(p);
    }
    return left;
}

int prs_rd_test(void) {
    prs_rd_parser_t p;
    prs_rd_init(&p, "3 + 4 * (2 - 1)");
    int result = prs_rd_parse_expr(&p);
    if (p.error) return -1;
    if (result != 7) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1276: recursive descent parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1276: empty output");
    Ok(())
}

/// C1277: Pratt parser for operator precedence with binding power
#[test]
fn c1277_pratt_parser_binding_power() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef enum {
    PRS_TOK_NUM,
    PRS_TOK_PLUS,
    PRS_TOK_MINUS,
    PRS_TOK_STAR,
    PRS_TOK_SLASH,
    PRS_TOK_CARET,
    PRS_TOK_LPAREN,
    PRS_TOK_RPAREN,
    PRS_TOK_EOF
} prs_pratt_tok_t;

typedef struct {
    prs_pratt_tok_t type;
    int value;
} prs_pratt_token_t;

typedef struct {
    prs_pratt_token_t tokens[64];
    int count;
    int pos;
} prs_pratt_lexer_t;

int prs_pratt_lbp(prs_pratt_tok_t type) {
    switch (type) {
        case PRS_TOK_PLUS: case PRS_TOK_MINUS: return 10;
        case PRS_TOK_STAR: case PRS_TOK_SLASH: return 20;
        case PRS_TOK_CARET: return 30;
        default: return 0;
    }
}

prs_pratt_token_t prs_pratt_peek(prs_pratt_lexer_t *lex) {
    if (lex->pos < lex->count) return lex->tokens[lex->pos];
    prs_pratt_token_t eof;
    eof.type = PRS_TOK_EOF;
    eof.value = 0;
    return eof;
}

prs_pratt_token_t prs_pratt_next(prs_pratt_lexer_t *lex) {
    prs_pratt_token_t t = prs_pratt_peek(lex);
    if (lex->pos < lex->count) lex->pos++;
    return t;
}

int prs_pratt_expr(prs_pratt_lexer_t *lex, int min_bp);

int prs_pratt_nud(prs_pratt_lexer_t *lex, prs_pratt_token_t t) {
    if (t.type == PRS_TOK_NUM) return t.value;
    if (t.type == PRS_TOK_MINUS) return -prs_pratt_expr(lex, 25);
    if (t.type == PRS_TOK_LPAREN) {
        int v = prs_pratt_expr(lex, 0);
        prs_pratt_next(lex);
        return v;
    }
    return 0;
}

int prs_pratt_expr(prs_pratt_lexer_t *lex, int min_bp) {
    prs_pratt_token_t t = prs_pratt_next(lex);
    int left = prs_pratt_nud(lex, t);
    while (prs_pratt_lbp(prs_pratt_peek(lex).type) > min_bp) {
        prs_pratt_token_t op = prs_pratt_next(lex);
        int right = prs_pratt_expr(lex, prs_pratt_lbp(op.type));
        switch (op.type) {
            case PRS_TOK_PLUS:  left = left + right; break;
            case PRS_TOK_MINUS: left = left - right; break;
            case PRS_TOK_STAR:  left = left * right; break;
            case PRS_TOK_SLASH: if (right != 0) left = left / right; break;
            default: break;
        }
    }
    return left;
}

int prs_pratt_test(void) {
    prs_pratt_lexer_t lex;
    lex.count = 5;
    lex.pos = 0;
    lex.tokens[0].type = PRS_TOK_NUM; lex.tokens[0].value = 2;
    lex.tokens[1].type = PRS_TOK_PLUS; lex.tokens[1].value = 0;
    lex.tokens[2].type = PRS_TOK_NUM; lex.tokens[2].value = 3;
    lex.tokens[3].type = PRS_TOK_STAR; lex.tokens[3].value = 0;
    lex.tokens[4].type = PRS_TOK_NUM; lex.tokens[4].value = 4;
    int result = prs_pratt_expr(&lex, 0);
    if (result != 14) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1277: Pratt parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1277: empty output");
    Ok(())
}

/// C1278: Shunting-yard algorithm for infix to postfix conversion
#[test]
fn c1278_shunting_yard_algorithm() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    int data[64];
    int top;
} prs_sy_stack_t;

void prs_sy_stack_init(prs_sy_stack_t *s) {
    s->top = -1;
}

void prs_sy_push(prs_sy_stack_t *s, int val) {
    if (s->top < 63) {
        s->top++;
        s->data[s->top] = val;
    }
}

int prs_sy_pop(prs_sy_stack_t *s) {
    if (s->top >= 0) return s->data[s->top--];
    return -1;
}

int prs_sy_peek(prs_sy_stack_t *s) {
    if (s->top >= 0) return s->data[s->top];
    return -1;
}

int prs_sy_precedence(char op) {
    if (op == '+' || op == '-') return 1;
    if (op == '*' || op == '/') return 2;
    if (op == '^') return 3;
    return 0;
}

int prs_sy_is_left_assoc(char op) {
    return op != '^';
}

int prs_sy_convert(const char *infix, int *output, int *out_len) {
    prs_sy_stack_t ops;
    prs_sy_stack_init(&ops);
    *out_len = 0;
    int i = 0;
    while (infix[i] != '\0') {
        char c = infix[i];
        if (c >= '0' && c <= '9') {
            output[(*out_len)++] = c - '0';
        } else if (c == '(') {
            prs_sy_push(&ops, c);
        } else if (c == ')') {
            while (ops.top >= 0 && prs_sy_peek(&ops) != '(') {
                output[(*out_len)++] = prs_sy_pop(&ops) + 256;
            }
            if (ops.top >= 0) prs_sy_pop(&ops);
        } else if (c == '+' || c == '-' || c == '*' || c == '/') {
            while (ops.top >= 0 && prs_sy_peek(&ops) != '(' &&
                   (prs_sy_precedence(prs_sy_peek(&ops)) > prs_sy_precedence(c) ||
                    (prs_sy_precedence(prs_sy_peek(&ops)) == prs_sy_precedence(c) &&
                     prs_sy_is_left_assoc(c)))) {
                output[(*out_len)++] = prs_sy_pop(&ops) + 256;
            }
            prs_sy_push(&ops, c);
        }
        i++;
    }
    while (ops.top >= 0) {
        output[(*out_len)++] = prs_sy_pop(&ops) + 256;
    }
    return *out_len;
}

int prs_sy_test(void) {
    int output[64];
    int len;
    prs_sy_convert("3+4*2", output, &len);
    if (len != 5) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1278: shunting-yard should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1278: empty output");
    Ok(())
}

/// C1279: Postfix (RPN) expression evaluator with stack
#[test]
fn c1279_postfix_expression_evaluator() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    double data[128];
    int top;
} prs_rpn_stack_t;

void prs_rpn_init(prs_rpn_stack_t *s) {
    s->top = -1;
}

void prs_rpn_push(prs_rpn_stack_t *s, double val) {
    if (s->top < 127) {
        s->top++;
        s->data[s->top] = val;
    }
}

double prs_rpn_pop(prs_rpn_stack_t *s) {
    if (s->top >= 0) return s->data[s->top--];
    return 0.0;
}

int prs_rpn_is_digit(char c) {
    return c >= '0' && c <= '9';
}

double prs_rpn_evaluate(const char *expr) {
    prs_rpn_stack_t stack;
    prs_rpn_init(&stack);
    int i = 0;
    while (expr[i] != '\0') {
        char c = expr[i];
        if (prs_rpn_is_digit(c)) {
            double num = 0;
            while (prs_rpn_is_digit(expr[i])) {
                num = num * 10 + (expr[i] - '0');
                i++;
            }
            prs_rpn_push(&stack, num);
            continue;
        }
        if (c == ' ') { i++; continue; }
        double b = prs_rpn_pop(&stack);
        double a = prs_rpn_pop(&stack);
        double r = 0;
        switch (c) {
            case '+': r = a + b; break;
            case '-': r = a - b; break;
            case '*': r = a * b; break;
            case '/': r = (b != 0.0) ? a / b : 0.0; break;
            default: break;
        }
        prs_rpn_push(&stack, r);
        i++;
    }
    return prs_rpn_pop(&stack);
}

int prs_rpn_test(void) {
    double r1 = prs_rpn_evaluate("3 4 + 2 *");
    if (r1 < 13.9 || r1 > 14.1) return -1;
    double r2 = prs_rpn_evaluate("5 1 2 + 4 * + 3 -");
    if (r2 < 13.9 || r2 > 14.1) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1279: postfix evaluator should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1279: empty output");
    Ok(())
}

/// C1280: Infix to postfix converter with parentheses and unary minus
#[test]
fn c1280_infix_to_postfix_converter() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    char output[256];
    int out_pos;
    char ops[64];
    int ops_top;
} prs_i2p_ctx_t;

void prs_i2p_init(prs_i2p_ctx_t *ctx) {
    ctx->out_pos = 0;
    ctx->ops_top = -1;
    ctx->output[0] = '\0';
}

void prs_i2p_emit_char(prs_i2p_ctx_t *ctx, char c) {
    if (ctx->out_pos < 255) {
        ctx->output[ctx->out_pos++] = c;
        ctx->output[ctx->out_pos] = '\0';
    }
}

void prs_i2p_push_op(prs_i2p_ctx_t *ctx, char op) {
    if (ctx->ops_top < 63) ctx->ops[++ctx->ops_top] = op;
}

char prs_i2p_pop_op(prs_i2p_ctx_t *ctx) {
    if (ctx->ops_top >= 0) return ctx->ops[ctx->ops_top--];
    return '\0';
}

char prs_i2p_peek_op(prs_i2p_ctx_t *ctx) {
    if (ctx->ops_top >= 0) return ctx->ops[ctx->ops_top];
    return '\0';
}

int prs_i2p_prec(char op) {
    if (op == '+' || op == '-') return 1;
    if (op == '*' || op == '/') return 2;
    if (op == '~') return 3;
    return 0;
}

void prs_i2p_convert(prs_i2p_ctx_t *ctx, const char *infix) {
    int i = 0;
    int expect_operand = 1;
    while (infix[i] != '\0') {
        char c = infix[i];
        if (c == ' ') { i++; continue; }
        if (c >= '0' && c <= '9') {
            while (infix[i] >= '0' && infix[i] <= '9') {
                prs_i2p_emit_char(ctx, infix[i]);
                i++;
            }
            prs_i2p_emit_char(ctx, ' ');
            expect_operand = 0;
            continue;
        }
        if (c == '(') {
            prs_i2p_push_op(ctx, c);
            expect_operand = 1;
            i++;
            continue;
        }
        if (c == ')') {
            while (ctx->ops_top >= 0 && prs_i2p_peek_op(ctx) != '(') {
                prs_i2p_emit_char(ctx, prs_i2p_pop_op(ctx));
                prs_i2p_emit_char(ctx, ' ');
            }
            if (ctx->ops_top >= 0) prs_i2p_pop_op(ctx);
            expect_operand = 0;
            i++;
            continue;
        }
        if (c == '-' && expect_operand) {
            prs_i2p_push_op(ctx, '~');
            i++;
            continue;
        }
        while (ctx->ops_top >= 0 && prs_i2p_peek_op(ctx) != '(' &&
               prs_i2p_prec(prs_i2p_peek_op(ctx)) >= prs_i2p_prec(c)) {
            prs_i2p_emit_char(ctx, prs_i2p_pop_op(ctx));
            prs_i2p_emit_char(ctx, ' ');
        }
        prs_i2p_push_op(ctx, c);
        expect_operand = 1;
        i++;
    }
    while (ctx->ops_top >= 0) {
        prs_i2p_emit_char(ctx, prs_i2p_pop_op(ctx));
        prs_i2p_emit_char(ctx, ' ');
    }
}

int prs_i2p_test(void) {
    prs_i2p_ctx_t ctx;
    prs_i2p_init(&ctx);
    prs_i2p_convert(&ctx, "3 + 4 * 2");
    if (ctx.out_pos < 1) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1280: infix to postfix should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1280: empty output");
    Ok(())
}

// ============================================================================
// C1281-C1285: Lexical Analysis
// ============================================================================

/// C1281: Scanner/tokenizer for a simple expression language
#[test]
fn c1281_scanner_tokenizer() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef enum {
    PRS_SC_INT,
    PRS_SC_IDENT,
    PRS_SC_OP,
    PRS_SC_LPAREN,
    PRS_SC_RPAREN,
    PRS_SC_SEMI,
    PRS_SC_EOF,
    PRS_SC_ERROR
} prs_sc_tok_type_t;

typedef struct {
    prs_sc_tok_type_t type;
    char text[64];
    int text_len;
    int line;
    int col;
} prs_sc_token_t;

typedef struct {
    const char *src;
    int pos;
    int line;
    int col;
} prs_sc_scanner_t;

void prs_sc_init(prs_sc_scanner_t *sc, const char *src) {
    sc->src = src;
    sc->pos = 0;
    sc->line = 1;
    sc->col = 1;
}

char prs_sc_current(prs_sc_scanner_t *sc) {
    return sc->src[sc->pos];
}

char prs_sc_advance_char(prs_sc_scanner_t *sc) {
    char c = sc->src[sc->pos];
    if (c == '\n') { sc->line++; sc->col = 1; }
    else { sc->col++; }
    sc->pos++;
    return c;
}

int prs_sc_is_alpha(char c) {
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_';
}

int prs_sc_is_digit(char c) {
    return c >= '0' && c <= '9';
}

prs_sc_token_t prs_sc_next_token(prs_sc_scanner_t *sc) {
    prs_sc_token_t tok;
    tok.text_len = 0;
    while (prs_sc_current(sc) == ' ' || prs_sc_current(sc) == '\n' ||
           prs_sc_current(sc) == '\t' || prs_sc_current(sc) == '\r') {
        prs_sc_advance_char(sc);
    }
    tok.line = sc->line;
    tok.col = sc->col;
    char c = prs_sc_current(sc);
    if (c == '\0') { tok.type = PRS_SC_EOF; return tok; }
    if (prs_sc_is_digit(c)) {
        tok.type = PRS_SC_INT;
        while (prs_sc_is_digit(prs_sc_current(sc))) {
            tok.text[tok.text_len++] = prs_sc_advance_char(sc);
        }
        tok.text[tok.text_len] = '\0';
        return tok;
    }
    if (prs_sc_is_alpha(c)) {
        tok.type = PRS_SC_IDENT;
        while (prs_sc_is_alpha(prs_sc_current(sc)) || prs_sc_is_digit(prs_sc_current(sc))) {
            tok.text[tok.text_len++] = prs_sc_advance_char(sc);
        }
        tok.text[tok.text_len] = '\0';
        return tok;
    }
    if (c == '(') { tok.type = PRS_SC_LPAREN; prs_sc_advance_char(sc); return tok; }
    if (c == ')') { tok.type = PRS_SC_RPAREN; prs_sc_advance_char(sc); return tok; }
    if (c == ';') { tok.type = PRS_SC_SEMI; prs_sc_advance_char(sc); return tok; }
    if (c == '+' || c == '-' || c == '*' || c == '/' || c == '=') {
        tok.type = PRS_SC_OP;
        tok.text[0] = prs_sc_advance_char(sc);
        tok.text_len = 1;
        tok.text[1] = '\0';
        return tok;
    }
    tok.type = PRS_SC_ERROR;
    prs_sc_advance_char(sc);
    return tok;
}

int prs_sc_test(void) {
    prs_sc_scanner_t sc;
    prs_sc_init(&sc, "x = 42 + y;");
    int count = 0;
    prs_sc_token_t tok;
    do {
        tok = prs_sc_next_token(&sc);
        count++;
    } while (tok.type != PRS_SC_EOF);
    if (count < 6) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1281: scanner/tokenizer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1281: empty output");
    Ok(())
}

/// C1282: Keyword recognition with perfect hash lookup
#[test]
fn c1282_keyword_recognition() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef enum {
    PRS_KW_IF,
    PRS_KW_ELSE,
    PRS_KW_WHILE,
    PRS_KW_FOR,
    PRS_KW_RETURN,
    PRS_KW_INT,
    PRS_KW_VOID,
    PRS_KW_STRUCT,
    PRS_KW_NONE
} prs_kw_type_t;

typedef struct {
    const char *word;
    prs_kw_type_t type;
} prs_kw_entry_t;

int prs_kw_strlen(const char *s) {
    int n = 0;
    while (s[n] != '\0') n++;
    return n;
}

int prs_kw_strcmp(const char *a, const char *b) {
    int i = 0;
    while (a[i] != '\0' && b[i] != '\0') {
        if (a[i] != b[i]) return a[i] - b[i];
        i++;
    }
    return a[i] - b[i];
}

unsigned int prs_kw_hash(const char *s) {
    unsigned int h = 5381;
    int i = 0;
    while (s[i] != '\0') {
        h = ((h << 5) + h) + (unsigned char)s[i];
        i++;
    }
    return h;
}

prs_kw_type_t prs_kw_lookup(const char *word) {
    prs_kw_entry_t table[8];
    table[0].word = "if";      table[0].type = PRS_KW_IF;
    table[1].word = "else";    table[1].type = PRS_KW_ELSE;
    table[2].word = "while";   table[2].type = PRS_KW_WHILE;
    table[3].word = "for";     table[3].type = PRS_KW_FOR;
    table[4].word = "return";  table[4].type = PRS_KW_RETURN;
    table[5].word = "int";     table[5].type = PRS_KW_INT;
    table[6].word = "void";    table[6].type = PRS_KW_VOID;
    table[7].word = "struct";  table[7].type = PRS_KW_STRUCT;
    int i;
    for (i = 0; i < 8; i++) {
        if (prs_kw_strcmp(word, table[i].word) == 0) {
            return table[i].type;
        }
    }
    return PRS_KW_NONE;
}

int prs_kw_test(void) {
    if (prs_kw_lookup("if") != PRS_KW_IF) return -1;
    if (prs_kw_lookup("while") != PRS_KW_WHILE) return -2;
    if (prs_kw_lookup("return") != PRS_KW_RETURN) return -3;
    if (prs_kw_lookup("foo") != PRS_KW_NONE) return -4;
    if (prs_kw_lookup("struct") != PRS_KW_STRUCT) return -5;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1282: keyword recognition should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1282: empty output");
    Ok(())
}

/// C1283: Number literal parser supporting decimal, hex, octal, and binary
#[test]
fn c1283_number_literal_parser() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef enum {
    PRS_NL_DECIMAL,
    PRS_NL_HEX,
    PRS_NL_OCTAL,
    PRS_NL_BINARY,
    PRS_NL_FLOAT,
    PRS_NL_INVALID
} prs_nl_type_t;

typedef struct {
    prs_nl_type_t type;
    long long int_val;
    double float_val;
    int chars_consumed;
} prs_nl_result_t;

int prs_nl_is_hex_digit(char c) {
    return (c >= '0' && c <= '9') || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F');
}

int prs_nl_hex_value(char c) {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'f') return 10 + c - 'a';
    if (c >= 'A' && c <= 'F') return 10 + c - 'A';
    return 0;
}

prs_nl_result_t prs_nl_parse(const char *s) {
    prs_nl_result_t r;
    r.int_val = 0;
    r.float_val = 0.0;
    r.chars_consumed = 0;
    r.type = PRS_NL_INVALID;
    int i = 0;
    if (s[0] == '\0') return r;
    if (s[0] == '0' && (s[1] == 'x' || s[1] == 'X')) {
        r.type = PRS_NL_HEX;
        i = 2;
        while (prs_nl_is_hex_digit(s[i])) {
            r.int_val = r.int_val * 16 + prs_nl_hex_value(s[i]);
            i++;
        }
        r.chars_consumed = i;
        return r;
    }
    if (s[0] == '0' && (s[1] == 'b' || s[1] == 'B')) {
        r.type = PRS_NL_BINARY;
        i = 2;
        while (s[i] == '0' || s[i] == '1') {
            r.int_val = r.int_val * 2 + (s[i] - '0');
            i++;
        }
        r.chars_consumed = i;
        return r;
    }
    if (s[0] == '0' && s[1] >= '0' && s[1] <= '7') {
        r.type = PRS_NL_OCTAL;
        i = 1;
        while (s[i] >= '0' && s[i] <= '7') {
            r.int_val = r.int_val * 8 + (s[i] - '0');
            i++;
        }
        r.chars_consumed = i;
        return r;
    }
    r.type = PRS_NL_DECIMAL;
    while (s[i] >= '0' && s[i] <= '9') {
        r.int_val = r.int_val * 10 + (s[i] - '0');
        i++;
    }
    if (s[i] == '.') {
        r.type = PRS_NL_FLOAT;
        r.float_val = (double)r.int_val;
        i++;
        double frac = 0.1;
        while (s[i] >= '0' && s[i] <= '9') {
            r.float_val = r.float_val + (s[i] - '0') * frac;
            frac = frac * 0.1;
            i++;
        }
    }
    r.chars_consumed = i;
    return r;
}

int prs_nl_test(void) {
    prs_nl_result_t r;
    r = prs_nl_parse("0xFF");
    if (r.type != PRS_NL_HEX || r.int_val != 255) return -1;
    r = prs_nl_parse("0b1010");
    if (r.type != PRS_NL_BINARY || r.int_val != 10) return -2;
    r = prs_nl_parse("077");
    if (r.type != PRS_NL_OCTAL || r.int_val != 63) return -3;
    r = prs_nl_parse("42");
    if (r.type != PRS_NL_DECIMAL || r.int_val != 42) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1283: number literal parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1283: empty output");
    Ok(())
}

/// C1284: String escape sequence processor
#[test]
fn c1284_string_escape_processor() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    char buffer[256];
    int length;
    int error;
} prs_esc_result_t;

int prs_esc_hex_val(char c) {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'f') return 10 + c - 'a';
    if (c >= 'A' && c <= 'F') return 10 + c - 'A';
    return -1;
}

prs_esc_result_t prs_esc_process(const char *input) {
    prs_esc_result_t res;
    res.length = 0;
    res.error = 0;
    int i = 0;
    while (input[i] != '\0' && res.length < 255) {
        if (input[i] != '\\') {
            res.buffer[res.length++] = input[i++];
            continue;
        }
        i++;
        switch (input[i]) {
            case 'n': res.buffer[res.length++] = '\n'; i++; break;
            case 't': res.buffer[res.length++] = '\t'; i++; break;
            case 'r': res.buffer[res.length++] = '\r'; i++; break;
            case '\\': res.buffer[res.length++] = '\\'; i++; break;
            case '0': res.buffer[res.length++] = '\0'; i++; break;
            case 'x': {
                i++;
                int h = prs_esc_hex_val(input[i]);
                int l = prs_esc_hex_val(input[i + 1]);
                if (h >= 0 && l >= 0) {
                    res.buffer[res.length++] = (char)(h * 16 + l);
                    i += 2;
                } else {
                    res.error = 1;
                    i++;
                }
                break;
            }
            default:
                res.buffer[res.length++] = input[i++];
                break;
        }
    }
    res.buffer[res.length] = '\0';
    return res;
}

int prs_esc_test(void) {
    prs_esc_result_t r;
    r = prs_esc_process("hello\\nworld");
    if (r.length != 11) return -1;
    r = prs_esc_process("tab\\there");
    if (r.length != 8) return -2;
    r = prs_esc_process("hex\\x41B");
    if (r.buffer[3] != 'A') return -3;
    if (r.error) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1284: string escape processor should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1284: empty output");
    Ok(())
}

/// C1285: Comment stripper for C-style single-line and multi-line comments
#[test]
fn c1285_comment_stripper() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef enum {
    PRS_CS_NORMAL,
    PRS_CS_SLASH,
    PRS_CS_LINE_COMMENT,
    PRS_CS_BLOCK_COMMENT,
    PRS_CS_BLOCK_STAR,
    PRS_CS_STRING,
    PRS_CS_STRING_ESC
} prs_cs_state_t;

int prs_cs_strip(const char *input, char *output) {
    prs_cs_state_t state = PRS_CS_NORMAL;
    int out = 0;
    int i = 0;
    while (input[i] != '\0') {
        char c = input[i];
        switch (state) {
            case PRS_CS_NORMAL:
                if (c == '/') state = PRS_CS_SLASH;
                else if (c == '"') { output[out++] = c; state = PRS_CS_STRING; }
                else output[out++] = c;
                break;
            case PRS_CS_SLASH:
                if (c == '/') state = PRS_CS_LINE_COMMENT;
                else if (c == '*') state = PRS_CS_BLOCK_COMMENT;
                else { output[out++] = '/'; output[out++] = c; state = PRS_CS_NORMAL; }
                break;
            case PRS_CS_LINE_COMMENT:
                if (c == '\n') { output[out++] = '\n'; state = PRS_CS_NORMAL; }
                break;
            case PRS_CS_BLOCK_COMMENT:
                if (c == '*') state = PRS_CS_BLOCK_STAR;
                break;
            case PRS_CS_BLOCK_STAR:
                if (c == '/') { output[out++] = ' '; state = PRS_CS_NORMAL; }
                else if (c != '*') state = PRS_CS_BLOCK_COMMENT;
                break;
            case PRS_CS_STRING:
                output[out++] = c;
                if (c == '\\') state = PRS_CS_STRING_ESC;
                else if (c == '"') state = PRS_CS_NORMAL;
                break;
            case PRS_CS_STRING_ESC:
                output[out++] = c;
                state = PRS_CS_STRING;
                break;
        }
        i++;
    }
    output[out] = '\0';
    return out;
}

int prs_cs_test(void) {
    char out[512];
    int len = prs_cs_strip("int x = 5; // comment\nint y = 6;", out);
    if (len < 20) return -1;
    len = prs_cs_strip("int /* block */ z;", out);
    if (len < 8) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1285: comment stripper should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1285: empty output");
    Ok(())
}

// ============================================================================
// C1286-C1290: Grammar Parsing
// ============================================================================

/// C1286: LL(1) table-driven parser for a simple grammar
#[test]
fn c1286_ll1_table_driven_parser() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef enum {
    PRS_LL_S, PRS_LL_A, PRS_LL_B,
    PRS_LL_NT_COUNT
} prs_ll_nonterminal_t;

typedef enum {
    PRS_LL_TERM_A, PRS_LL_TERM_B, PRS_LL_TERM_C, PRS_LL_TERM_EOF,
    PRS_LL_TERM_COUNT
} prs_ll_terminal_t;

typedef struct {
    int production[8];
    int length;
} prs_ll_rule_t;

typedef struct {
    int stack[64];
    int top;
    int parse_table[3][4];
    prs_ll_rule_t rules[8];
    int rule_count;
    int error;
} prs_ll_parser_t;

void prs_ll_init(prs_ll_parser_t *p) {
    p->top = -1;
    p->error = 0;
    p->rule_count = 4;
    int i, j;
    for (i = 0; i < 3; i++)
        for (j = 0; j < 4; j++)
            p->parse_table[i][j] = -1;
    p->rules[0].production[0] = 100 + PRS_LL_TERM_A;
    p->rules[0].production[1] = PRS_LL_A;
    p->rules[0].length = 2;
    p->rules[1].production[0] = 100 + PRS_LL_TERM_B;
    p->rules[1].production[1] = PRS_LL_B;
    p->rules[1].length = 2;
    p->rules[2].production[0] = 100 + PRS_LL_TERM_C;
    p->rules[2].length = 1;
    p->rules[3].length = 0;
    p->parse_table[PRS_LL_S][PRS_LL_TERM_A] = 0;
    p->parse_table[PRS_LL_A][PRS_LL_TERM_B] = 1;
    p->parse_table[PRS_LL_A][PRS_LL_TERM_EOF] = 3;
    p->parse_table[PRS_LL_B][PRS_LL_TERM_C] = 2;
    p->parse_table[PRS_LL_B][PRS_LL_TERM_EOF] = 3;
}

void prs_ll_push(prs_ll_parser_t *p, int sym) {
    if (p->top < 63) p->stack[++p->top] = sym;
}

int prs_ll_pop(prs_ll_parser_t *p) {
    if (p->top >= 0) return p->stack[p->top--];
    return -1;
}

int prs_ll_parse(prs_ll_parser_t *p, const int *tokens, int len) {
    prs_ll_push(p, PRS_LL_S);
    int pos = 0;
    while (p->top >= 0 && !p->error) {
        int top_sym = prs_ll_pop(p);
        int tok = (pos < len) ? tokens[pos] : PRS_LL_TERM_EOF;
        if (top_sym >= 100) {
            if (top_sym - 100 == tok) { pos++; }
            else { p->error = 1; }
        } else {
            int rule_idx = p->parse_table[top_sym][tok];
            if (rule_idx < 0) { p->error = 1; }
            else {
                int k;
                for (k = p->rules[rule_idx].length - 1; k >= 0; k--) {
                    prs_ll_push(p, p->rules[rule_idx].production[k]);
                }
            }
        }
    }
    return p->error ? -1 : 0;
}

int prs_ll_test(void) {
    prs_ll_parser_t p;
    prs_ll_init(&p);
    int tokens[] = {PRS_LL_TERM_A, PRS_LL_TERM_B, PRS_LL_TERM_C};
    int result = prs_ll_parse(&p, tokens, 3);
    if (result != 0) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1286: LL(1) parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1286: empty output");
    Ok(())
}

/// C1287: LR(0) automaton state builder for shift-reduce parsing
#[test]
fn c1287_lr0_automaton() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    int lhs;
    int rhs[8];
    int rhs_len;
} prs_lr_rule_t;

typedef struct {
    int rule_idx;
    int dot_pos;
} prs_lr_item_t;

typedef struct {
    prs_lr_item_t items[32];
    int item_count;
    int transitions[16];
    int trans_symbols[16];
    int trans_count;
} prs_lr_state_t;

typedef struct {
    prs_lr_rule_t rules[16];
    int rule_count;
    prs_lr_state_t states[32];
    int state_count;
} prs_lr_automaton_t;

void prs_lr_init(prs_lr_automaton_t *a) {
    a->rule_count = 0;
    a->state_count = 0;
}

void prs_lr_add_rule(prs_lr_automaton_t *a, int lhs, int *rhs, int rhs_len) {
    if (a->rule_count < 16) {
        prs_lr_rule_t *r = &a->rules[a->rule_count];
        r->lhs = lhs;
        r->rhs_len = rhs_len;
        int i;
        for (i = 0; i < rhs_len && i < 8; i++) r->rhs[i] = rhs[i];
        a->rule_count++;
    }
}

int prs_lr_items_equal(prs_lr_item_t a, prs_lr_item_t b) {
    return a.rule_idx == b.rule_idx && a.dot_pos == b.dot_pos;
}

int prs_lr_closure(prs_lr_automaton_t *a, prs_lr_state_t *state) {
    int changed = 1;
    while (changed) {
        changed = 0;
        int i;
        for (i = 0; i < state->item_count; i++) {
            prs_lr_item_t *it = &state->items[i];
            prs_lr_rule_t *rule = &a->rules[it->rule_idx];
            if (it->dot_pos < rule->rhs_len) {
                int sym = rule->rhs[it->dot_pos];
                int j;
                for (j = 0; j < a->rule_count; j++) {
                    if (a->rules[j].lhs == sym) {
                        prs_lr_item_t new_item;
                        new_item.rule_idx = j;
                        new_item.dot_pos = 0;
                        int dup = 0;
                        int k;
                        for (k = 0; k < state->item_count; k++) {
                            if (prs_lr_items_equal(state->items[k], new_item)) { dup = 1; break; }
                        }
                        if (!dup && state->item_count < 32) {
                            state->items[state->item_count++] = new_item;
                            changed = 1;
                        }
                    }
                }
            }
        }
    }
    return state->item_count;
}

int prs_lr_test(void) {
    prs_lr_automaton_t a;
    prs_lr_init(&a);
    int rhs1[] = {10, 11};
    prs_lr_add_rule(&a, 0, rhs1, 2);
    int rhs2[] = {12};
    prs_lr_add_rule(&a, 10, rhs2, 1);
    prs_lr_state_t s0;
    s0.item_count = 1;
    s0.trans_count = 0;
    s0.items[0].rule_idx = 0;
    s0.items[0].dot_pos = 0;
    int count = prs_lr_closure(&a, &s0);
    if (count < 2) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1287: LR(0) automaton should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1287: empty output");
    Ok(())
}

/// C1288: FIRST set computation for context-free grammar
#[test]
fn c1288_first_set_computation() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    int lhs;
    int rhs[8];
    int rhs_len;
} prs_fs_rule_t;

typedef struct {
    int elements[32];
    int count;
} prs_fs_set_t;

typedef struct {
    prs_fs_rule_t rules[16];
    int rule_count;
    prs_fs_set_t first_sets[16];
    int symbol_count;
    int terminal_count;
} prs_fs_grammar_t;

void prs_fs_set_init(prs_fs_set_t *s) {
    s->count = 0;
}

int prs_fs_set_contains(prs_fs_set_t *s, int elem) {
    int i;
    for (i = 0; i < s->count; i++) {
        if (s->elements[i] == elem) return 1;
    }
    return 0;
}

int prs_fs_set_add(prs_fs_set_t *s, int elem) {
    if (!prs_fs_set_contains(s, elem) && s->count < 32) {
        s->elements[s->count++] = elem;
        return 1;
    }
    return 0;
}

int prs_fs_is_terminal(prs_fs_grammar_t *g, int sym) {
    return sym < g->terminal_count;
}

void prs_fs_compute(prs_fs_grammar_t *g) {
    int i;
    for (i = 0; i < g->symbol_count; i++) {
        prs_fs_set_init(&g->first_sets[i]);
    }
    for (i = 0; i < g->terminal_count; i++) {
        prs_fs_set_add(&g->first_sets[i], i);
    }
    int changed = 1;
    while (changed) {
        changed = 0;
        int r;
        for (r = 0; r < g->rule_count; r++) {
            int lhs = g->rules[r].lhs;
            if (g->rules[r].rhs_len == 0) {
                changed |= prs_fs_set_add(&g->first_sets[lhs], -1);
                continue;
            }
            int k;
            int all_nullable = 1;
            for (k = 0; k < g->rules[r].rhs_len; k++) {
                int sym = g->rules[r].rhs[k];
                int j;
                for (j = 0; j < g->first_sets[sym].count; j++) {
                    int e = g->first_sets[sym].elements[j];
                    if (e != -1) {
                        changed |= prs_fs_set_add(&g->first_sets[lhs], e);
                    }
                }
                if (!prs_fs_set_contains(&g->first_sets[sym], -1)) {
                    all_nullable = 0;
                    break;
                }
            }
            if (all_nullable) {
                changed |= prs_fs_set_add(&g->first_sets[lhs], -1);
            }
        }
    }
}

int prs_fs_test(void) {
    prs_fs_grammar_t g;
    g.terminal_count = 3;
    g.symbol_count = 5;
    g.rule_count = 2;
    g.rules[0].lhs = 3;
    g.rules[0].rhs[0] = 0;
    g.rules[0].rhs[1] = 4;
    g.rules[0].rhs_len = 2;
    g.rules[1].lhs = 4;
    g.rules[1].rhs[0] = 1;
    g.rules[1].rhs_len = 1;
    prs_fs_compute(&g);
    if (!prs_fs_set_contains(&g.first_sets[3], 0)) return -1;
    if (!prs_fs_set_contains(&g.first_sets[4], 1)) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1288: FIRST set computation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1288: empty output");
    Ok(())
}

/// C1289: FOLLOW set computation for context-free grammar
#[test]
fn c1289_follow_set_computation() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    int lhs;
    int rhs[8];
    int rhs_len;
} prs_fo_rule_t;

typedef struct {
    int elements[32];
    int count;
} prs_fo_set_t;

void prs_fo_set_init(prs_fo_set_t *s) { s->count = 0; }

int prs_fo_set_contains(prs_fo_set_t *s, int elem) {
    int i;
    for (i = 0; i < s->count; i++) {
        if (s->elements[i] == elem) return 1;
    }
    return 0;
}

int prs_fo_set_add(prs_fo_set_t *s, int elem) {
    if (!prs_fo_set_contains(s, elem) && s->count < 32) {
        s->elements[s->count++] = elem;
        return 1;
    }
    return 0;
}

int prs_fo_set_union(prs_fo_set_t *dst, prs_fo_set_t *src, int skip_epsilon) {
    int changed = 0;
    int i;
    for (i = 0; i < src->count; i++) {
        if (skip_epsilon && src->elements[i] == -1) continue;
        changed |= prs_fo_set_add(dst, src->elements[i]);
    }
    return changed;
}

typedef struct {
    prs_fo_rule_t rules[16];
    int rule_count;
    prs_fo_set_t first_sets[16];
    prs_fo_set_t follow_sets[16];
    int terminal_count;
    int symbol_count;
    int start_symbol;
} prs_fo_grammar_t;

void prs_fo_compute(prs_fo_grammar_t *g) {
    int i;
    for (i = 0; i < g->symbol_count; i++) {
        prs_fo_set_init(&g->follow_sets[i]);
    }
    prs_fo_set_add(&g->follow_sets[g->start_symbol], -2);
    int changed = 1;
    while (changed) {
        changed = 0;
        int r;
        for (r = 0; r < g->rule_count; r++) {
            int lhs = g->rules[r].lhs;
            int k;
            for (k = 0; k < g->rules[r].rhs_len; k++) {
                int B = g->rules[r].rhs[k];
                if (B < g->terminal_count) continue;
                if (k + 1 < g->rules[r].rhs_len) {
                    int beta = g->rules[r].rhs[k + 1];
                    changed |= prs_fo_set_union(&g->follow_sets[B],
                                                 &g->first_sets[beta], 1);
                    if (prs_fo_set_contains(&g->first_sets[beta], -1)) {
                        changed |= prs_fo_set_union(&g->follow_sets[B],
                                                     &g->follow_sets[lhs], 0);
                    }
                } else {
                    changed |= prs_fo_set_union(&g->follow_sets[B],
                                                 &g->follow_sets[lhs], 0);
                }
            }
        }
    }
}

int prs_fo_test(void) {
    prs_fo_grammar_t g;
    g.terminal_count = 3;
    g.symbol_count = 5;
    g.start_symbol = 3;
    g.rule_count = 2;
    g.rules[0].lhs = 3;
    g.rules[0].rhs[0] = 0;
    g.rules[0].rhs[1] = 4;
    g.rules[0].rhs_len = 2;
    g.rules[1].lhs = 4;
    g.rules[1].rhs[0] = 1;
    g.rules[1].rhs_len = 1;
    int i;
    for (i = 0; i < 5; i++) prs_fo_set_init(&g.first_sets[i]);
    prs_fo_set_add(&g.first_sets[0], 0);
    prs_fo_set_add(&g.first_sets[1], 1);
    prs_fo_set_add(&g.first_sets[2], 2);
    prs_fo_set_add(&g.first_sets[3], 0);
    prs_fo_set_add(&g.first_sets[4], 1);
    prs_fo_compute(&g);
    if (!prs_fo_set_contains(&g.follow_sets[3], -2)) return -1;
    if (!prs_fo_set_contains(&g.follow_sets[4], -2)) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1289: FOLLOW set computation should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1289: empty output");
    Ok(())
}

/// C1290: Nullable symbol detection for context-free grammars
#[test]
fn c1290_nullable_detection() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    int lhs;
    int rhs[8];
    int rhs_len;
} prs_nu_rule_t;

typedef struct {
    prs_nu_rule_t rules[16];
    int rule_count;
    int nullable[16];
    int symbol_count;
    int terminal_count;
} prs_nu_grammar_t;

void prs_nu_init(prs_nu_grammar_t *g) {
    int i;
    for (i = 0; i < 16; i++) g->nullable[i] = 0;
}

void prs_nu_compute(prs_nu_grammar_t *g) {
    prs_nu_init(g);
    int changed = 1;
    while (changed) {
        changed = 0;
        int r;
        for (r = 0; r < g->rule_count; r++) {
            int lhs = g->rules[r].lhs;
            if (g->nullable[lhs]) continue;
            if (g->rules[r].rhs_len == 0) {
                g->nullable[lhs] = 1;
                changed = 1;
                continue;
            }
            int all_nullable = 1;
            int k;
            for (k = 0; k < g->rules[r].rhs_len; k++) {
                int sym = g->rules[r].rhs[k];
                if (sym < g->terminal_count || !g->nullable[sym]) {
                    all_nullable = 0;
                    break;
                }
            }
            if (all_nullable) {
                g->nullable[lhs] = 1;
                changed = 1;
            }
        }
    }
}

int prs_nu_count_nullable(prs_nu_grammar_t *g) {
    int count = 0;
    int i;
    for (i = g->terminal_count; i < g->symbol_count; i++) {
        if (g->nullable[i]) count++;
    }
    return count;
}

int prs_nu_test(void) {
    prs_nu_grammar_t g;
    g.terminal_count = 3;
    g.symbol_count = 6;
    g.rule_count = 4;
    g.rules[0].lhs = 3; g.rules[0].rhs[0] = 4; g.rules[0].rhs[1] = 5;
    g.rules[0].rhs_len = 2;
    g.rules[1].lhs = 4; g.rules[1].rhs[0] = 0;
    g.rules[1].rhs_len = 1;
    g.rules[2].lhs = 4; g.rules[2].rhs_len = 0;
    g.rules[3].lhs = 5; g.rules[3].rhs_len = 0;
    prs_nu_compute(&g);
    if (!g.nullable[4]) return -1;
    if (!g.nullable[5]) return -2;
    if (!g.nullable[3]) return -3;
    int count = prs_nu_count_nullable(&g);
    if (count != 3) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1290: nullable detection should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1290: empty output");
    Ok(())
}

// ============================================================================
// C1291-C1295: Specialized Parsers
// ============================================================================

/// C1291: Regex NFA builder from pattern string
#[test]
fn c1291_regex_nfa_builder() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    int from;
    int to;
    char symbol;
    int epsilon;
} prs_nfa_edge_t;

typedef struct {
    prs_nfa_edge_t edges[128];
    int edge_count;
    int state_count;
    int start;
    int accept;
} prs_nfa_t;

void prs_nfa_init(prs_nfa_t *nfa) {
    nfa->edge_count = 0;
    nfa->state_count = 0;
    nfa->start = 0;
    nfa->accept = 0;
}

int prs_nfa_new_state(prs_nfa_t *nfa) {
    return nfa->state_count++;
}

void prs_nfa_add_edge(prs_nfa_t *nfa, int from, int to, char sym, int eps) {
    if (nfa->edge_count < 128) {
        prs_nfa_edge_t *e = &nfa->edges[nfa->edge_count++];
        e->from = from;
        e->to = to;
        e->symbol = sym;
        e->epsilon = eps;
    }
}

typedef struct { int start; int accept; } prs_nfa_frag_t;

prs_nfa_frag_t prs_nfa_build_char(prs_nfa_t *nfa, char c) {
    prs_nfa_frag_t f;
    f.start = prs_nfa_new_state(nfa);
    f.accept = prs_nfa_new_state(nfa);
    prs_nfa_add_edge(nfa, f.start, f.accept, c, 0);
    return f;
}

prs_nfa_frag_t prs_nfa_build_concat(prs_nfa_t *nfa, prs_nfa_frag_t a, prs_nfa_frag_t b) {
    prs_nfa_frag_t f;
    prs_nfa_add_edge(nfa, a.accept, b.start, '\0', 1);
    f.start = a.start;
    f.accept = b.accept;
    return f;
}

prs_nfa_frag_t prs_nfa_build_star(prs_nfa_t *nfa, prs_nfa_frag_t a) {
    prs_nfa_frag_t f;
    f.start = prs_nfa_new_state(nfa);
    f.accept = prs_nfa_new_state(nfa);
    prs_nfa_add_edge(nfa, f.start, a.start, '\0', 1);
    prs_nfa_add_edge(nfa, f.start, f.accept, '\0', 1);
    prs_nfa_add_edge(nfa, a.accept, a.start, '\0', 1);
    prs_nfa_add_edge(nfa, a.accept, f.accept, '\0', 1);
    return f;
}

prs_nfa_frag_t prs_nfa_compile(prs_nfa_t *nfa, const char *pattern) {
    prs_nfa_frag_t stack[32];
    int sp = 0;
    int i = 0;
    while (pattern[i] != '\0') {
        char c = pattern[i];
        if (c == '*' && sp > 0) {
            stack[sp - 1] = prs_nfa_build_star(nfa, stack[sp - 1]);
        } else if (c == '.') {
            if (sp >= 2) {
                prs_nfa_frag_t b = stack[--sp];
                prs_nfa_frag_t a = stack[--sp];
                stack[sp++] = prs_nfa_build_concat(nfa, a, b);
            }
        } else {
            stack[sp++] = prs_nfa_build_char(nfa, c);
        }
        i++;
    }
    if (sp > 0) {
        nfa->start = stack[0].start;
        nfa->accept = stack[0].accept;
        return stack[0];
    }
    prs_nfa_frag_t empty;
    empty.start = prs_nfa_new_state(nfa);
    empty.accept = empty.start;
    return empty;
}

int prs_nfa_test(void) {
    prs_nfa_t nfa;
    prs_nfa_init(&nfa);
    prs_nfa_compile(&nfa, "ab.c.");
    if (nfa.state_count < 6) return -1;
    if (nfa.edge_count < 4) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1291: regex NFA builder should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1291: empty output");
    Ok(())
}

/// C1292: Glob pattern matcher with *, ?, and character classes
#[test]
fn c1292_glob_pattern_matcher() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

int prs_glob_match(const char *pattern, const char *text) {
    int pp = 0, tp = 0;
    int star_p = -1, star_t = -1;
    while (text[tp] != '\0') {
        if (pattern[pp] == '?') {
            pp++;
            tp++;
        } else if (pattern[pp] == '*') {
            star_p = pp;
            star_t = tp;
            pp++;
        } else if (pattern[pp] == '[') {
            pp++;
            int negate = 0;
            if (pattern[pp] == '!') { negate = 1; pp++; }
            int matched = 0;
            while (pattern[pp] != '\0' && pattern[pp] != ']') {
                if (pattern[pp + 1] == '-' && pattern[pp + 2] != ']') {
                    if (text[tp] >= pattern[pp] && text[tp] <= pattern[pp + 2]) {
                        matched = 1;
                    }
                    pp += 3;
                } else {
                    if (text[tp] == pattern[pp]) matched = 1;
                    pp++;
                }
            }
            if (pattern[pp] == ']') pp++;
            if (negate) matched = !matched;
            if (!matched) {
                if (star_p >= 0) {
                    pp = star_p + 1;
                    tp = ++star_t;
                } else {
                    return 0;
                }
            } else {
                tp++;
            }
        } else if (pattern[pp] == text[tp]) {
            pp++;
            tp++;
        } else if (star_p >= 0) {
            pp = star_p + 1;
            tp = ++star_t;
        } else {
            return 0;
        }
    }
    while (pattern[pp] == '*') pp++;
    return pattern[pp] == '\0';
}

int prs_glob_test(void) {
    if (!prs_glob_match("*.txt", "file.txt")) return -1;
    if (!prs_glob_match("test?", "test1")) return -2;
    if (prs_glob_match("test?", "test12")) return -3;
    if (!prs_glob_match("[a-z]*.c", "main.c")) return -4;
    if (!prs_glob_match("**", "anything")) return -5;
    if (prs_glob_match("[!0-9]*", "1bad")) return -6;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1292: glob matcher should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1292: empty output");
    Ok(())
}

/// C1293: printf format string parser
#[test]
fn c1293_printf_format_parser() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef enum {
    PRS_FMT_INT,
    PRS_FMT_UINT,
    PRS_FMT_FLOAT,
    PRS_FMT_CHAR,
    PRS_FMT_STRING,
    PRS_FMT_HEX,
    PRS_FMT_OCTAL,
    PRS_FMT_POINTER,
    PRS_FMT_PERCENT,
    PRS_FMT_UNKNOWN
} prs_fmt_type_t;

typedef struct {
    prs_fmt_type_t type;
    int width;
    int precision;
    int left_align;
    int zero_pad;
    int has_width;
    int has_precision;
    char length_mod;
} prs_fmt_spec_t;

typedef struct {
    prs_fmt_spec_t specs[32];
    int count;
} prs_fmt_result_t;

int prs_fmt_parse_int(const char *s, int *pos) {
    int val = 0;
    while (s[*pos] >= '0' && s[*pos] <= '9') {
        val = val * 10 + (s[*pos] - '0');
        (*pos)++;
    }
    return val;
}

prs_fmt_result_t prs_fmt_parse(const char *fmt) {
    prs_fmt_result_t result;
    result.count = 0;
    int i = 0;
    while (fmt[i] != '\0') {
        if (fmt[i] != '%') { i++; continue; }
        i++;
        if (fmt[i] == '%') { i++; continue; }
        prs_fmt_spec_t *spec = &result.specs[result.count];
        spec->left_align = 0;
        spec->zero_pad = 0;
        spec->has_width = 0;
        spec->has_precision = 0;
        spec->width = 0;
        spec->precision = 0;
        spec->length_mod = '\0';
        while (fmt[i] == '-' || fmt[i] == '0' || fmt[i] == '+' || fmt[i] == ' ') {
            if (fmt[i] == '-') spec->left_align = 1;
            if (fmt[i] == '0') spec->zero_pad = 1;
            i++;
        }
        if (fmt[i] >= '1' && fmt[i] <= '9') {
            spec->has_width = 1;
            spec->width = prs_fmt_parse_int(fmt, &i);
        }
        if (fmt[i] == '.') {
            i++;
            spec->has_precision = 1;
            spec->precision = prs_fmt_parse_int(fmt, &i);
        }
        if (fmt[i] == 'l' || fmt[i] == 'h') {
            spec->length_mod = fmt[i];
            i++;
        }
        switch (fmt[i]) {
            case 'd': case 'i': spec->type = PRS_FMT_INT; break;
            case 'u': spec->type = PRS_FMT_UINT; break;
            case 'f': case 'F': spec->type = PRS_FMT_FLOAT; break;
            case 'c': spec->type = PRS_FMT_CHAR; break;
            case 's': spec->type = PRS_FMT_STRING; break;
            case 'x': case 'X': spec->type = PRS_FMT_HEX; break;
            case 'o': spec->type = PRS_FMT_OCTAL; break;
            case 'p': spec->type = PRS_FMT_POINTER; break;
            default: spec->type = PRS_FMT_UNKNOWN; break;
        }
        i++;
        if (result.count < 31) result.count++;
    }
    return result;
}

int prs_fmt_test(void) {
    prs_fmt_result_t r = prs_fmt_parse("Hello %s, you are %d years old, score: %08.2f%%");
    if (r.count != 3) return -1;
    if (r.specs[0].type != PRS_FMT_STRING) return -2;
    if (r.specs[1].type != PRS_FMT_INT) return -3;
    if (r.specs[2].type != PRS_FMT_FLOAT) return -4;
    if (r.specs[2].zero_pad != 1) return -5;
    if (r.specs[2].has_precision != 1) return -6;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1293: printf format parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1293: empty output");
    Ok(())
}

/// C1294: Date/time string parser (ISO 8601 subset)
#[test]
fn c1294_datetime_parser() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    int year;
    int month;
    int day;
    int hour;
    int minute;
    int second;
    int has_time;
    int valid;
} prs_dt_datetime_t;

int prs_dt_parse_2digit(const char *s, int pos) {
    if (s[pos] >= '0' && s[pos] <= '9' &&
        s[pos + 1] >= '0' && s[pos + 1] <= '9') {
        return (s[pos] - '0') * 10 + (s[pos + 1] - '0');
    }
    return -1;
}

int prs_dt_parse_4digit(const char *s, int pos) {
    int v = 0;
    int i;
    for (i = 0; i < 4; i++) {
        if (s[pos + i] < '0' || s[pos + i] > '9') return -1;
        v = v * 10 + (s[pos + i] - '0');
    }
    return v;
}

int prs_dt_days_in_month(int year, int month) {
    int days[] = {31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31};
    if (month < 1 || month > 12) return 0;
    if (month == 2) {
        if ((year % 4 == 0 && year % 100 != 0) || year % 400 == 0) return 29;
    }
    return days[month - 1];
}

prs_dt_datetime_t prs_dt_parse(const char *s) {
    prs_dt_datetime_t dt;
    dt.valid = 0;
    dt.has_time = 0;
    dt.hour = 0;
    dt.minute = 0;
    dt.second = 0;
    dt.year = prs_dt_parse_4digit(s, 0);
    if (dt.year < 0 || s[4] != '-') return dt;
    dt.month = prs_dt_parse_2digit(s, 5);
    if (dt.month < 1 || dt.month > 12 || s[7] != '-') return dt;
    dt.day = prs_dt_parse_2digit(s, 8);
    if (dt.day < 1 || dt.day > prs_dt_days_in_month(dt.year, dt.month)) return dt;
    dt.valid = 1;
    if (s[10] == 'T' || s[10] == ' ') {
        dt.hour = prs_dt_parse_2digit(s, 11);
        if (dt.hour < 0 || dt.hour > 23 || s[13] != ':') return dt;
        dt.minute = prs_dt_parse_2digit(s, 14);
        if (dt.minute < 0 || dt.minute > 59 || s[16] != ':') return dt;
        dt.second = prs_dt_parse_2digit(s, 17);
        if (dt.second < 0 || dt.second > 59) return dt;
        dt.has_time = 1;
    }
    return dt;
}

int prs_dt_test(void) {
    prs_dt_datetime_t d;
    d = prs_dt_parse("2024-01-15");
    if (!d.valid || d.year != 2024 || d.month != 1 || d.day != 15) return -1;
    d = prs_dt_parse("2024-02-29T10:30:45");
    if (!d.valid || !d.has_time || d.hour != 10) return -2;
    d = prs_dt_parse("2023-02-29");
    if (d.valid) return -3;
    d = prs_dt_parse("2024-13-01");
    if (d.valid) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1294: datetime parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1294: empty output");
    Ok(())
}

/// C1295: Semantic version parser (major.minor.patch with optional pre-release)
#[test]
fn c1295_semver_parser() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    int major;
    int minor;
    int patch;
    char pre_release[32];
    int pre_len;
    int valid;
} prs_sv_version_t;

int prs_sv_parse_num(const char *s, int *pos) {
    if (s[*pos] < '0' || s[*pos] > '9') return -1;
    int val = 0;
    while (s[*pos] >= '0' && s[*pos] <= '9') {
        val = val * 10 + (s[*pos] - '0');
        (*pos)++;
    }
    return val;
}

prs_sv_version_t prs_sv_parse(const char *s) {
    prs_sv_version_t v;
    v.valid = 0;
    v.pre_len = 0;
    v.pre_release[0] = '\0';
    int pos = 0;
    if (s[0] == 'v' || s[0] == 'V') pos++;
    v.major = prs_sv_parse_num(s, &pos);
    if (v.major < 0 || s[pos] != '.') return v;
    pos++;
    v.minor = prs_sv_parse_num(s, &pos);
    if (v.minor < 0 || s[pos] != '.') return v;
    pos++;
    v.patch = prs_sv_parse_num(s, &pos);
    if (v.patch < 0) return v;
    v.valid = 1;
    if (s[pos] == '-') {
        pos++;
        while (s[pos] != '\0' && s[pos] != '+' && v.pre_len < 31) {
            v.pre_release[v.pre_len++] = s[pos++];
        }
        v.pre_release[v.pre_len] = '\0';
    }
    return v;
}

int prs_sv_compare(prs_sv_version_t a, prs_sv_version_t b) {
    if (a.major != b.major) return a.major - b.major;
    if (a.minor != b.minor) return a.minor - b.minor;
    if (a.patch != b.patch) return a.patch - b.patch;
    if (a.pre_len == 0 && b.pre_len == 0) return 0;
    if (a.pre_len == 0) return 1;
    if (b.pre_len == 0) return -1;
    int i = 0;
    while (i < a.pre_len && i < b.pre_len) {
        if (a.pre_release[i] != b.pre_release[i])
            return a.pre_release[i] - b.pre_release[i];
        i++;
    }
    return a.pre_len - b.pre_len;
}

int prs_sv_test(void) {
    prs_sv_version_t v1 = prs_sv_parse("1.2.3");
    if (!v1.valid || v1.major != 1 || v1.minor != 2 || v1.patch != 3) return -1;
    prs_sv_version_t v2 = prs_sv_parse("v2.0.0-beta.1");
    if (!v2.valid || v2.major != 2 || v2.pre_len == 0) return -2;
    prs_sv_version_t v3 = prs_sv_parse("1.0.0");
    prs_sv_version_t v4 = prs_sv_parse("1.0.1");
    if (prs_sv_compare(v3, v4) >= 0) return -3;
    prs_sv_version_t v5 = prs_sv_parse("bad");
    if (v5.valid) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1295: semver parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1295: empty output");
    Ok(())
}

// ============================================================================
// C1296-C1300: Applied Parsing
// ============================================================================

/// C1296: Calculator with named variables and assignment
#[test]
fn c1296_calculator_with_variables() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    char name[16];
    double value;
} prs_cv_var_t;

typedef struct {
    prs_cv_var_t vars[32];
    int var_count;
    const char *input;
    int pos;
    int error;
} prs_cv_calc_t;

void prs_cv_init(prs_cv_calc_t *c) {
    c->var_count = 0;
    c->error = 0;
}

double *prs_cv_get_var(prs_cv_calc_t *c, const char *name, int name_len) {
    int i;
    for (i = 0; i < c->var_count; i++) {
        int j = 0;
        int match = 1;
        while (j < name_len && c->vars[i].name[j] != '\0') {
            if (c->vars[i].name[j] != name[j]) { match = 0; break; }
            j++;
        }
        if (match && j == name_len && c->vars[i].name[j] == '\0')
            return &c->vars[i].value;
    }
    if (c->var_count < 32) {
        int j;
        for (j = 0; j < name_len && j < 15; j++)
            c->vars[c->var_count].name[j] = name[j];
        c->vars[c->var_count].name[j] = '\0';
        c->vars[c->var_count].value = 0.0;
        return &c->vars[c->var_count++].value;
    }
    return 0;
}

void prs_cv_skip_ws(prs_cv_calc_t *c) {
    while (c->input[c->pos] == ' ') c->pos++;
}

double prs_cv_parse_expr(prs_cv_calc_t *c);
double prs_cv_parse_term(prs_cv_calc_t *c);
double prs_cv_parse_atom(prs_cv_calc_t *c);

double prs_cv_parse_atom(prs_cv_calc_t *c) {
    prs_cv_skip_ws(c);
    char ch = c->input[c->pos];
    if (ch == '(') {
        c->pos++;
        double v = prs_cv_parse_expr(c);
        prs_cv_skip_ws(c);
        if (c->input[c->pos] == ')') c->pos++;
        return v;
    }
    if ((ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z')) {
        int start = c->pos;
        while ((c->input[c->pos] >= 'a' && c->input[c->pos] <= 'z') ||
               (c->input[c->pos] >= 'A' && c->input[c->pos] <= 'Z') ||
               (c->input[c->pos] >= '0' && c->input[c->pos] <= '9'))
            c->pos++;
        double *var = prs_cv_get_var(c, c->input + start, c->pos - start);
        if (!var) { c->error = 1; return 0; }
        return *var;
    }
    double num = 0;
    int sign = 1;
    if (ch == '-') { sign = -1; c->pos++; }
    while (c->input[c->pos] >= '0' && c->input[c->pos] <= '9') {
        num = num * 10 + (c->input[c->pos] - '0');
        c->pos++;
    }
    return sign * num;
}

double prs_cv_parse_term(prs_cv_calc_t *c) {
    double left = prs_cv_parse_atom(c);
    prs_cv_skip_ws(c);
    while (c->input[c->pos] == '*' || c->input[c->pos] == '/') {
        char op = c->input[c->pos++];
        double right = prs_cv_parse_atom(c);
        if (op == '*') left *= right;
        else if (right != 0) left /= right;
        prs_cv_skip_ws(c);
    }
    return left;
}

double prs_cv_parse_expr(prs_cv_calc_t *c) {
    double left = prs_cv_parse_term(c);
    prs_cv_skip_ws(c);
    while (c->input[c->pos] == '+' || c->input[c->pos] == '-') {
        char op = c->input[c->pos++];
        double right = prs_cv_parse_term(c);
        if (op == '+') left += right;
        else left -= right;
        prs_cv_skip_ws(c);
    }
    return left;
}

int prs_cv_test(void) {
    prs_cv_calc_t calc;
    prs_cv_init(&calc);
    double *x = prs_cv_get_var(&calc, "x", 1);
    if (!x) return -1;
    *x = 10.0;
    calc.input = "x * 2 + 3";
    calc.pos = 0;
    double r = prs_cv_parse_expr(&calc);
    if (r < 22.9 || r > 23.1) return -2;
    if (calc.error) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1296: calculator with variables should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1296: empty output");
    Ok(())
}

/// C1297: INI-style config file parser with sections and key=value pairs
#[test]
fn c1297_config_file_parser() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r##"
typedef unsigned long size_t;

typedef struct {
    char section[32];
    char key[32];
    char value[64];
} prs_cfg_entry_t;

typedef struct {
    prs_cfg_entry_t entries[64];
    int count;
    int error;
    int error_line;
} prs_cfg_result_t;

void prs_cfg_trim(char *s, int len) {
    int start = 0;
    while (start < len && (s[start] == ' ' || s[start] == '\t')) start++;
    int end = len - 1;
    while (end >= start && (s[end] == ' ' || s[end] == '\t' || s[end] == '\r')) end--;
    int i;
    int j = 0;
    for (i = start; i <= end; i++) s[j++] = s[i];
    s[j] = '\0';
}

void prs_cfg_copy(char *dst, const char *src, int start, int end, int max) {
    int i;
    int len = end - start;
    if (len > max - 1) len = max - 1;
    for (i = 0; i < len; i++) dst[i] = src[start + i];
    dst[len] = '\0';
}

prs_cfg_result_t prs_cfg_parse(const char *text) {
    prs_cfg_result_t result;
    result.count = 0;
    result.error = 0;
    result.error_line = 0;
    char current_section[32];
    current_section[0] = '\0';
    int line = 1;
    int i = 0;
    while (text[i] != '\0') {
        int line_start = i;
        while (text[i] != '\0' && text[i] != '\n') i++;
        int line_end = i;
        if (text[i] == '\n') i++;
        while (line_start < line_end && (text[line_start] == ' ' || text[line_start] == '\t'))
            line_start++;
        if (line_start >= line_end || text[line_start] == '#' || text[line_start] == ';') {
            line++;
            continue;
        }
        if (text[line_start] == '[') {
            int end = line_start + 1;
            while (end < line_end && text[end] != ']') end++;
            if (text[end] == ']') {
                prs_cfg_copy(current_section, text, line_start + 1, end, 32);
            } else {
                result.error = 1;
                result.error_line = line;
            }
        } else {
            int eq = line_start;
            while (eq < line_end && text[eq] != '=') eq++;
            if (eq < line_end && result.count < 64) {
                prs_cfg_entry_t *e = &result.entries[result.count];
                prs_cfg_copy(e->section, current_section, 0, 32, 32);
                prs_cfg_copy(e->key, text, line_start, eq, 32);
                prs_cfg_trim(e->key, eq - line_start);
                prs_cfg_copy(e->value, text, eq + 1, line_end, 64);
                prs_cfg_trim(e->value, line_end - eq - 1);
                result.count++;
            }
        }
        line++;
    }
    return result;
}

int prs_cfg_test(void) {
    const char *ini =
        "[server]\n"
        "host = localhost\n"
        "port = 8080\n"
        "; a comment line\n"
        "\n"
        "[database]\n"
        "name = mydb\n";
    prs_cfg_result_t r = prs_cfg_parse(ini);
    if (r.count != 3) return -1;
    if (r.error) return -2;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1297: config file parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1297: empty output");
    Ok(())
}

/// C1298: Command-line argument parser with flags, options, and positional args
#[test]
fn c1298_cmdline_parser() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef enum {
    PRS_ARG_FLAG,
    PRS_ARG_OPTION,
    PRS_ARG_POSITIONAL
} prs_arg_type_t;

typedef struct {
    prs_arg_type_t type;
    char name[32];
    char value[64];
    int present;
} prs_arg_entry_t;

typedef struct {
    prs_arg_entry_t args[32];
    int arg_count;
    char positional[16][64];
    int pos_count;
    int error;
} prs_arg_result_t;

int prs_arg_strcmp(const char *a, const char *b) {
    int i = 0;
    while (a[i] != '\0' && b[i] != '\0') {
        if (a[i] != b[i]) return a[i] - b[i];
        i++;
    }
    return a[i] - b[i];
}

void prs_arg_strcpy(char *dst, const char *src, int max) {
    int i = 0;
    while (src[i] != '\0' && i < max - 1) {
        dst[i] = src[i];
        i++;
    }
    dst[i] = '\0';
}

int prs_arg_starts_with_dd(const char *s) {
    return s[0] == '-' && s[1] == '-' && s[2] != '\0';
}

int prs_arg_starts_with_d(const char *s) {
    return s[0] == '-' && s[1] != '-' && s[1] != '\0';
}

prs_arg_result_t prs_arg_parse(int argc, const char *argv[]) {
    prs_arg_result_t result;
    result.arg_count = 0;
    result.pos_count = 0;
    result.error = 0;
    int i = 1;
    int positional_only = 0;
    while (i < argc) {
        if (prs_arg_strcmp(argv[i], "--") == 0) {
            positional_only = 1;
            i++;
            continue;
        }
        if (!positional_only && prs_arg_starts_with_dd(argv[i])) {
            prs_arg_entry_t *a = &result.args[result.arg_count];
            const char *name = argv[i] + 2;
            int eq = 0;
            while (name[eq] != '\0' && name[eq] != '=') eq++;
            if (name[eq] == '=') {
                a->type = PRS_ARG_OPTION;
                int k;
                for (k = 0; k < eq && k < 31; k++) a->name[k] = name[k];
                a->name[k] = '\0';
                prs_arg_strcpy(a->value, name + eq + 1, 64);
            } else {
                a->type = PRS_ARG_FLAG;
                prs_arg_strcpy(a->name, name, 32);
                a->value[0] = '\0';
            }
            a->present = 1;
            if (result.arg_count < 31) result.arg_count++;
        } else if (!positional_only && prs_arg_starts_with_d(argv[i])) {
            int j = 1;
            while (argv[i][j] != '\0') {
                prs_arg_entry_t *a = &result.args[result.arg_count];
                a->type = PRS_ARG_FLAG;
                a->name[0] = argv[i][j];
                a->name[1] = '\0';
                a->value[0] = '\0';
                a->present = 1;
                if (result.arg_count < 31) result.arg_count++;
                j++;
            }
        } else {
            if (result.pos_count < 16) {
                prs_arg_strcpy(result.positional[result.pos_count], argv[i], 64);
                result.pos_count++;
            }
        }
        i++;
    }
    return result;
}

int prs_arg_test(void) {
    const char *argv[] = {"prog", "--verbose", "--output=file.txt", "-abc", "input.c"};
    prs_arg_result_t r = prs_arg_parse(5, argv);
    if (r.arg_count != 5) return -1;
    if (r.pos_count != 1) return -2;
    if (r.error) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1298: cmdline parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1298: empty output");
    Ok(())
}

/// C1299: URI parser splitting scheme, host, port, path, query, fragment
#[test]
fn c1299_uri_parser() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    char scheme[16];
    char host[64];
    int port;
    char path[128];
    char query[128];
    char fragment[64];
    int valid;
} prs_uri_t;

void prs_uri_copy_range(char *dst, const char *src, int start, int end, int max) {
    int len = end - start;
    if (len > max - 1) len = max - 1;
    int i;
    for (i = 0; i < len; i++) dst[i] = src[start + i];
    dst[i] = '\0';
}

prs_uri_t prs_uri_parse(const char *uri) {
    prs_uri_t r;
    r.scheme[0] = '\0';
    r.host[0] = '\0';
    r.port = -1;
    r.path[0] = '\0';
    r.query[0] = '\0';
    r.fragment[0] = '\0';
    r.valid = 0;
    int i = 0;
    int scheme_end = -1;
    int j = 0;
    while (uri[j] != '\0') {
        if (uri[j] == ':' && uri[j + 1] == '/' && uri[j + 2] == '/') {
            scheme_end = j;
            break;
        }
        j++;
    }
    if (scheme_end > 0) {
        prs_uri_copy_range(r.scheme, uri, 0, scheme_end, 16);
        i = scheme_end + 3;
    }
    int host_start = i;
    while (uri[i] != '\0' && uri[i] != '/' && uri[i] != ':' &&
           uri[i] != '?' && uri[i] != '#') i++;
    prs_uri_copy_range(r.host, uri, host_start, i, 64);
    if (uri[i] == ':') {
        i++;
        r.port = 0;
        while (uri[i] >= '0' && uri[i] <= '9') {
            r.port = r.port * 10 + (uri[i] - '0');
            i++;
        }
    }
    if (uri[i] == '/') {
        int path_start = i;
        while (uri[i] != '\0' && uri[i] != '?' && uri[i] != '#') i++;
        prs_uri_copy_range(r.path, uri, path_start, i, 128);
    }
    if (uri[i] == '?') {
        i++;
        int q_start = i;
        while (uri[i] != '\0' && uri[i] != '#') i++;
        prs_uri_copy_range(r.query, uri, q_start, i, 128);
    }
    if (uri[i] == '#') {
        i++;
        int f_start = i;
        while (uri[i] != '\0') i++;
        prs_uri_copy_range(r.fragment, uri, f_start, i, 64);
    }
    if (r.host[0] != '\0') r.valid = 1;
    return r;
}

int prs_uri_test(void) {
    prs_uri_t u = prs_uri_parse("https://example.com:8080/path/to?key=val#sec");
    if (!u.valid) return -1;
    if (u.port != 8080) return -2;
    if (u.scheme[0] != 'h') return -3;
    if (u.path[0] != '/') return -4;
    if (u.fragment[0] != 's') return -5;
    prs_uri_t u2 = prs_uri_parse("http://localhost/");
    if (!u2.valid) return -6;
    if (u2.port != -1) return -7;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1299: URI parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1299: empty output");
    Ok(())
}

/// C1300: MIME type parser splitting type, subtype, and parameters
#[test]
fn c1300_mime_type_parser() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long size_t;

typedef struct {
    char key[32];
    char value[64];
} prs_mime_param_t;

typedef struct {
    char type_str[32];
    char subtype[32];
    prs_mime_param_t params[8];
    int param_count;
    int valid;
} prs_mime_t;

void prs_mime_copy(char *dst, const char *src, int start, int end, int max) {
    int len = end - start;
    if (len > max - 1) len = max - 1;
    int i;
    for (i = 0; i < len; i++) dst[i] = src[start + i];
    dst[i] = '\0';
}

void prs_mime_lower(char *s) {
    int i = 0;
    while (s[i] != '\0') {
        if (s[i] >= 'A' && s[i] <= 'Z') s[i] = s[i] + 32;
        i++;
    }
}

void prs_mime_trim_spaces(char *s) {
    int i = 0;
    while (s[i] == ' ') i++;
    if (i > 0) {
        int j = 0;
        while (s[i] != '\0') s[j++] = s[i++];
        s[j] = '\0';
    }
    int len = 0;
    while (s[len] != '\0') len++;
    while (len > 0 && s[len - 1] == ' ') { s[--len] = '\0'; }
}

prs_mime_t prs_mime_parse(const char *input) {
    prs_mime_t m;
    m.valid = 0;
    m.param_count = 0;
    m.type_str[0] = '\0';
    m.subtype[0] = '\0';
    int i = 0;
    while (input[i] == ' ') i++;
    int type_start = i;
    while (input[i] != '\0' && input[i] != '/') i++;
    if (input[i] != '/') return m;
    prs_mime_copy(m.type_str, input, type_start, i, 32);
    prs_mime_lower(m.type_str);
    prs_mime_trim_spaces(m.type_str);
    i++;
    int sub_start = i;
    while (input[i] != '\0' && input[i] != ';') i++;
    prs_mime_copy(m.subtype, input, sub_start, i, 32);
    prs_mime_lower(m.subtype);
    prs_mime_trim_spaces(m.subtype);
    m.valid = 1;
    while (input[i] == ';') {
        i++;
        while (input[i] == ' ') i++;
        int key_start = i;
        while (input[i] != '\0' && input[i] != '=') i++;
        if (input[i] != '=' || m.param_count >= 8) break;
        prs_mime_param_t *p = &m.params[m.param_count];
        prs_mime_copy(p->key, input, key_start, i, 32);
        prs_mime_lower(p->key);
        prs_mime_trim_spaces(p->key);
        i++;
        int val_start = i;
        int in_quote = 0;
        if (input[i] == '"') { in_quote = 1; i++; val_start = i; }
        if (in_quote) {
            while (input[i] != '\0' && input[i] != '"') i++;
            prs_mime_copy(p->value, input, val_start, i, 64);
            if (input[i] == '"') i++;
        } else {
            while (input[i] != '\0' && input[i] != ';') i++;
            prs_mime_copy(p->value, input, val_start, i, 64);
            prs_mime_trim_spaces(p->value);
        }
        m.param_count++;
    }
    return m;
}

int prs_mime_test(void) {
    prs_mime_t m = prs_mime_parse("text/html; charset=utf-8");
    if (!m.valid) return -1;
    if (m.type_str[0] != 't') return -2;
    if (m.param_count != 1) return -3;
    prs_mime_t m2 = prs_mime_parse("application/json");
    if (!m2.valid || m2.param_count != 0) return -4;
    prs_mime_t m3 = prs_mime_parse("multipart/form-data; boundary=\"----abc\"");
    if (!m3.valid || m3.param_count != 1) return -5;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1300: MIME type parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1300: empty output");
    Ok(())
}
