//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1301-C1325: Regex Engine Patterns -- the kind of C code found in regex
//! libraries, text search engines, pattern matchers, and string processing
//! utilities.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise NFA/DFA construction, pattern matching, regex
//! compilation, execution engines, and specialized matching -- all expressed
//! as valid C99 with array-based representations (no malloc/free, no includes).
//!
//! Organization:
//! - C1301-C1305: NFA/DFA basics (state, epsilon closure, subset construction, minimization)
//! - C1306-C1310: Pattern matching (literal, char class, quantifiers, alternation, anchors)
//! - C1311-C1315: Regex compilation (postfix, Thompson, char range, escape, grouping)
//! - C1316-C1320: Regex execution (backtracking, NFA sim, match extract, greedy/lazy, lookahead)
//! - C1321-C1325: Specialized (glob, wildcard, replace, split, pattern cache)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

use decy_core::transpile;

// ============================================================================
// C1301-C1305: NFA/DFA Basics
// ============================================================================

/// C1301: NFA state -- basic non-deterministic finite automaton state with transition table
#[test]
fn c1301_nfa_state() {
    let c_code = r#"
typedef struct { int id; int accepting; int transitions[128]; } re_nfa_state;

void re_nfa_init(re_nfa_state *s, int id) {
    s->id = id;
    s->accepting = 0;
    int i;
    for (i = 0; i < 128; i++) s->transitions[i] = -1;
}

int re_nfa_accepts(re_nfa_state *s) { return s->accepting; }

void re_nfa_add_transition(re_nfa_state *s, int ch, int target) {
    if (ch >= 0 && ch < 128) s->transitions[ch] = target;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1301 failed: {:?}", result.err());
}

/// C1302: DFA state -- deterministic finite automaton with fixed alphabet
#[test]
fn c1302_dfa_state() {
    let c_code = r#"
typedef struct {
    int num_states;
    int transitions[64][256];
    int accepting[64];
    int start;
} re_dfa_t;

void re_dfa_init(re_dfa_t *d, int start) {
    int i, j;
    d->num_states = 0;
    d->start = start;
    for (i = 0; i < 64; i++) {
        d->accepting[i] = 0;
        for (j = 0; j < 256; j++) d->transitions[i][j] = -1;
    }
}

int re_dfa_step(re_dfa_t *d, int state, int ch) {
    if (state < 0 || state >= 64 || ch < 0 || ch >= 256) return -1;
    return d->transitions[state][ch];
}

int re_dfa_match(re_dfa_t *d, const char *s, int len) {
    int state = d->start;
    int i;
    for (i = 0; i < len; i++) {
        state = re_dfa_step(d, state, (unsigned char)s[i]);
        if (state < 0) return 0;
    }
    return d->accepting[state];
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1302 failed: {:?}", result.err());
}

/// C1303: Epsilon closure -- compute set of NFA states reachable via epsilon transitions
#[test]
fn c1303_epsilon_closure() {
    let c_code = r#"
typedef struct {
    int eps[32][8];
    int eps_count[32];
} re_eps_nfa_t;

void re_eps_init(re_eps_nfa_t *n) {
    int i, j;
    for (i = 0; i < 32; i++) {
        n->eps_count[i] = 0;
        for (j = 0; j < 8; j++) n->eps[i][j] = -1;
    }
}

void re_eps_add(re_eps_nfa_t *n, int from, int to) {
    if (from >= 0 && from < 32 && n->eps_count[from] < 8) {
        n->eps[from][n->eps_count[from]] = to;
        n->eps_count[from] = n->eps_count[from] + 1;
    }
}

void re_eps_closure(re_eps_nfa_t *n, int *set, int *set_size) {
    int stack[64];
    int top = *set_size;
    int visited[32];
    int i;
    for (i = 0; i < 32; i++) visited[i] = 0;
    for (i = 0; i < *set_size; i++) {
        stack[i] = set[i];
        visited[set[i]] = 1;
    }
    while (top > 0) {
        int s = stack[top - 1];
        top = top - 1;
        for (i = 0; i < n->eps_count[s]; i++) {
            int t = n->eps[s][i];
            if (t >= 0 && !visited[t]) {
                visited[t] = 1;
                set[*set_size] = t;
                *set_size = *set_size + 1;
                stack[top] = t;
                top = top + 1;
            }
        }
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1303 failed: {:?}", result.err());
}

/// C1304: NFA-to-DFA subset construction -- powerset construction algorithm
#[test]
fn c1304_subset_construction() {
    let c_code = r#"
typedef struct {
    int nfa_trans[16][4];
    int dfa_trans[32][4];
    int dfa_sets[32][16];
    int dfa_set_sizes[32];
    int dfa_count;
} re_subset_t;

void re_subset_init(re_subset_t *sc) {
    int i, j;
    sc->dfa_count = 0;
    for (i = 0; i < 16; i++)
        for (j = 0; j < 4; j++) sc->nfa_trans[i][j] = -1;
    for (i = 0; i < 32; i++) {
        sc->dfa_set_sizes[i] = 0;
        for (j = 0; j < 4; j++) sc->dfa_trans[i][j] = -1;
    }
}

int re_subset_find(re_subset_t *sc, int *set, int size) {
    int i, j, match;
    for (i = 0; i < sc->dfa_count; i++) {
        if (sc->dfa_set_sizes[i] != size) continue;
        match = 1;
        for (j = 0; j < size; j++) {
            if (sc->dfa_sets[i][j] != set[j]) { match = 0; break; }
        }
        if (match) return i;
    }
    return -1;
}

int re_subset_add_state(re_subset_t *sc, int *set, int size) {
    int idx = sc->dfa_count;
    int j;
    if (idx >= 32) return -1;
    sc->dfa_set_sizes[idx] = size;
    for (j = 0; j < size; j++) sc->dfa_sets[idx][j] = set[j];
    sc->dfa_count = sc->dfa_count + 1;
    return idx;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1304 failed: {:?}", result.err());
}

/// C1305: DFA minimization -- Hopcroft-style partition refinement
#[test]
fn c1305_dfa_minimization() {
    let c_code = r#"
typedef struct {
    int partition[32];
    int num_states;
    int num_groups;
} re_dfa_min_t;

void re_dfa_min_init(re_dfa_min_t *m, int states, const int *accepting) {
    int i;
    m->num_states = states;
    m->num_groups = 2;
    for (i = 0; i < states; i++) {
        m->partition[i] = accepting[i] ? 1 : 0;
    }
}

int re_dfa_min_refine(re_dfa_min_t *m, const int trans[][4], int sym) {
    int new_part[32];
    int changed = 0;
    int i, j, next_group;
    next_group = m->num_groups;
    for (i = 0; i < m->num_states; i++) new_part[i] = m->partition[i];
    for (i = 0; i < m->num_states; i++) {
        for (j = i + 1; j < m->num_states; j++) {
            if (m->partition[i] != m->partition[j]) continue;
            int ti = trans[i][sym];
            int tj = trans[j][sym];
            int gi = (ti >= 0) ? m->partition[ti] : -1;
            int gj = (tj >= 0) ? m->partition[tj] : -1;
            if (gi != gj) {
                new_part[j] = next_group;
                changed = 1;
            }
        }
    }
    if (changed) {
        for (i = 0; i < m->num_states; i++) m->partition[i] = new_part[i];
        m->num_groups = next_group + 1;
    }
    return changed;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1305 failed: {:?}", result.err());
}

// ============================================================================
// C1306-C1310: Pattern Matching
// ============================================================================

/// C1306: Literal string match -- brute-force exact substring search
#[test]
fn c1306_literal_match() {
    let c_code = r#"
int re_literal_match(const char *text, int tlen, const char *pat, int plen) {
    int i, j;
    for (i = 0; i <= tlen - plen; i++) {
        int ok = 1;
        for (j = 0; j < plen; j++) {
            if (text[i + j] != pat[j]) { ok = 0; break; }
        }
        if (ok) return i;
    }
    return -1;
}

int re_literal_count(const char *text, int tlen, const char *pat, int plen) {
    int count = 0;
    int i, j;
    for (i = 0; i <= tlen - plen; i++) {
        int ok = 1;
        for (j = 0; j < plen; j++) {
            if (text[i + j] != pat[j]) { ok = 0; break; }
        }
        if (ok) count = count + 1;
    }
    return count;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1306 failed: {:?}", result.err());
}

/// C1307: Character class -- bitmap-based character set matching [a-z], [^0-9]
#[test]
fn c1307_character_class() {
    let c_code = r#"
typedef struct { unsigned char bits[32]; int negated; } re_charclass_t;

void re_cc_init(re_charclass_t *cc, int negated) {
    int i;
    for (i = 0; i < 32; i++) cc->bits[i] = 0;
    cc->negated = negated;
}

void re_cc_add(re_charclass_t *cc, int ch) {
    if (ch >= 0 && ch < 256)
        cc->bits[ch / 8] = cc->bits[ch / 8] | (1 << (ch % 8));
}

void re_cc_add_range(re_charclass_t *cc, int lo, int hi) {
    int i;
    for (i = lo; i <= hi && i < 256; i++) re_cc_add(cc, i);
}

int re_cc_test(re_charclass_t *cc, int ch) {
    if (ch < 0 || ch >= 256) return 0;
    int bit = (cc->bits[ch / 8] >> (ch % 8)) & 1;
    return cc->negated ? !bit : bit;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1307 failed: {:?}", result.err());
}

/// C1308: Quantifiers -- star (*), plus (+), question (?) matching
#[test]
fn c1308_quantifiers() {
    let c_code = r#"
int re_match_star(char c, const char *pat, const char *text) {
    do {
        if (re_match_here(pat, text)) return 1;
    } while (*text != '\0' && (*text == c || c == '.') && ++text);
    return 0;
}

int re_match_plus(char c, const char *pat, const char *text) {
    if (*text == '\0') return 0;
    if (*text != c && c != '.') return 0;
    text = text + 1;
    return re_match_star(c, pat, text);
}

int re_match_question(char c, const char *pat, const char *text) {
    if (re_match_here(pat, text)) return 1;
    if (*text != '\0' && (*text == c || c == '.'))
        return re_match_here(pat, text + 1);
    return 0;
}

int re_match_here(const char *pat, const char *text) {
    if (pat[0] == '\0') return 1;
    if (pat[1] == '*') return re_match_star(pat[0], pat + 2, text);
    if (pat[1] == '+') return re_match_plus(pat[0], pat + 2, text);
    if (pat[1] == '?') return re_match_question(pat[0], pat + 2, text);
    if (*text != '\0' && (pat[0] == '.' || pat[0] == *text))
        return re_match_here(pat + 1, text + 1);
    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1308 failed: {:?}", result.err());
}

/// C1309: Alternation -- matching a|b patterns via simple split
#[test]
fn c1309_alternation() {
    let c_code = r#"
typedef struct { char branches[8][32]; int count; } re_alt_t;

void re_alt_init(re_alt_t *a) { a->count = 0; }

void re_alt_add(re_alt_t *a, const char *branch, int len) {
    int i;
    if (a->count >= 8 || len >= 32) return;
    for (i = 0; i < len; i++) a->branches[a->count][i] = branch[i];
    a->branches[a->count][len] = '\0';
    a->count = a->count + 1;
}

int re_alt_match(re_alt_t *a, const char *text, int tlen) {
    int b, i, ok;
    for (b = 0; b < a->count; b++) {
        ok = 1;
        for (i = 0; a->branches[b][i] != '\0'; i++) {
            if (i >= tlen || text[i] != a->branches[b][i]) { ok = 0; break; }
        }
        if (ok) return b;
    }
    return -1;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1309 failed: {:?}", result.err());
}

/// C1310: Anchors -- matching ^ (start) and $ (end) anchors
#[test]
fn c1310_anchors() {
    let c_code = r#"
int re_match_here(const char *pat, const char *text);

int re_anchor_match(const char *pat, const char *text, int len) {
    int anchored_start = 0;
    int anchored_end = 0;
    int plen = 0;
    const char *p = pat;
    while (p[plen] != '\0') plen = plen + 1;
    if (plen > 0 && pat[0] == '^') { anchored_start = 1; pat = pat + 1; plen = plen - 1; }
    if (plen > 0 && pat[plen - 1] == '$') { anchored_end = 1; plen = plen - 1; }
    if (anchored_start) {
        return re_anchor_try(pat, plen, text, len, 0, anchored_end);
    }
    int i;
    for (i = 0; i < len; i++) {
        if (re_anchor_try(pat, plen, text, len, i, anchored_end)) return 1;
    }
    return 0;
}

int re_anchor_try(const char *pat, int plen, const char *text, int tlen, int pos, int anchored_end) {
    int j;
    if (pos + plen > tlen) return 0;
    for (j = 0; j < plen; j++) {
        if (pat[j] != '.' && pat[j] != text[pos + j]) return 0;
    }
    if (anchored_end && (pos + plen != tlen)) return 0;
    return 1;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1310 failed: {:?}", result.err());
}

// ============================================================================
// C1311-C1315: Regex Compilation
// ============================================================================

/// C1311: Postfix conversion -- infix regex to postfix (reverse Polish) notation
#[test]
fn c1311_postfix_conversion() {
    let c_code = r#"
int re_to_postfix(const char *re, char *out, int maxlen) {
    int nalt = 0;
    int natom = 0;
    int paren[16];
    int ptop = 0;
    int olen = 0;
    int i = 0;
    while (re[i] != '\0') {
        char c = re[i];
        if (c == '(') {
            if (natom > 1) { natom = natom - 1; if (olen < maxlen) out[olen++] = '.'; }
            if (ptop < 16) { paren[ptop] = nalt; ptop = ptop + 1; paren[ptop] = natom; ptop = ptop + 1; }
            nalt = 0; natom = 0;
        } else if (c == ')') {
            while (nalt > 0) { nalt = nalt - 1; if (olen < maxlen) out[olen++] = '|'; }
            ptop = ptop - 1; natom = paren[ptop]; ptop = ptop - 1; nalt = paren[ptop];
            natom = natom + 1;
        } else if (c == '|') {
            while (natom > 1) { natom = natom - 1; if (olen < maxlen) out[olen++] = '.'; }
            nalt = nalt + 1;
        } else if (c == '*' || c == '+' || c == '?') {
            if (olen < maxlen) out[olen++] = c;
        } else {
            if (natom > 1) { natom = natom - 1; if (olen < maxlen) out[olen++] = '.'; }
            if (olen < maxlen) out[olen++] = c;
            natom = natom + 1;
        }
        i = i + 1;
    }
    while (natom > 1) { natom = natom - 1; if (olen < maxlen) out[olen++] = '.'; }
    while (nalt > 0) { nalt = nalt - 1; if (olen < maxlen) out[olen++] = '|'; }
    if (olen < maxlen) out[olen] = '\0';
    return olen;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1311 failed: {:?}", result.err());
}

/// C1312: Thompson construction -- build NFA fragment from postfix regex
#[test]
fn c1312_thompson_construction() {
    let c_code = r#"
typedef struct { int start; int out; int ch; } re_frag_t;
typedef struct { int from; int to; int ch; } re_edge_t;

typedef struct {
    re_edge_t edges[128];
    int edge_count;
    int next_state;
} re_thompson_t;

void re_thompson_init(re_thompson_t *t) { t->edge_count = 0; t->next_state = 0; }

int re_thompson_new_state(re_thompson_t *t) { return t->next_state++; }

void re_thompson_add_edge(re_thompson_t *t, int from, int to, int ch) {
    if (t->edge_count < 128) {
        t->edges[t->edge_count].from = from;
        t->edges[t->edge_count].to = to;
        t->edges[t->edge_count].ch = ch;
        t->edge_count = t->edge_count + 1;
    }
}

re_frag_t re_thompson_literal(re_thompson_t *t, int ch) {
    re_frag_t f;
    f.start = re_thompson_new_state(t);
    f.out = re_thompson_new_state(t);
    f.ch = ch;
    re_thompson_add_edge(t, f.start, f.out, ch);
    return f;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1312 failed: {:?}", result.err());
}

/// C1313: Character range compilation -- compile [a-z] style ranges into bitmap
#[test]
fn c1313_char_range() {
    let c_code = r#"
typedef struct { unsigned char bitmap[32]; } re_range_t;

void re_range_clear(re_range_t *r) { int i; for (i = 0; i < 32; i++) r->bitmap[i] = 0; }

void re_range_set(re_range_t *r, int ch) {
    if (ch >= 0 && ch < 256) r->bitmap[ch / 8] = r->bitmap[ch / 8] | (unsigned char)(1 << (ch % 8));
}

int re_range_parse(re_range_t *r, const char *pat, int start) {
    int pos = start;
    re_range_clear(r);
    if (pat[pos] == '[') pos = pos + 1;
    while (pat[pos] != '\0' && pat[pos] != ']') {
        if (pat[pos + 1] == '-' && pat[pos + 2] != ']' && pat[pos + 2] != '\0') {
            int lo = (unsigned char)pat[pos];
            int hi = (unsigned char)pat[pos + 2];
            int c;
            for (c = lo; c <= hi; c++) re_range_set(r, c);
            pos = pos + 3;
        } else {
            re_range_set(r, (unsigned char)pat[pos]);
            pos = pos + 1;
        }
    }
    if (pat[pos] == ']') pos = pos + 1;
    return pos;
}

int re_range_test(re_range_t *r, int ch) {
    if (ch < 0 || ch >= 256) return 0;
    return (r->bitmap[ch / 8] >> (ch % 8)) & 1;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1313 failed: {:?}", result.err());
}

/// C1314: Escape sequences -- handle \d \w \s \n \t etc. in regex patterns
#[test]
fn c1314_escape_sequences() {
    let c_code = r#"
int re_is_digit(int ch) { return ch >= '0' && ch <= '9'; }
int re_is_word(int ch) { return (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || (ch >= '0' && ch <= '9') || ch == '_'; }
int re_is_space(int ch) { return ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r'; }

int re_escape_match(char esc, int ch) {
    if (esc == 'd') return re_is_digit(ch);
    if (esc == 'D') return !re_is_digit(ch);
    if (esc == 'w') return re_is_word(ch);
    if (esc == 'W') return !re_is_word(ch);
    if (esc == 's') return re_is_space(ch);
    if (esc == 'S') return !re_is_space(ch);
    if (esc == 'n') return ch == '\n';
    if (esc == 't') return ch == '\t';
    return ch == esc;
}

int re_parse_escape(const char *pat, int pos, char *out) {
    if (pat[pos] == '\\' && pat[pos + 1] != '\0') {
        *out = pat[pos + 1];
        return pos + 2;
    }
    *out = pat[pos];
    return pos + 1;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1314 failed: {:?}", result.err());
}

/// C1315: Grouping and capture -- parenthesized groups with capture extraction
#[test]
fn c1315_grouping_capture() {
    let c_code = r#"
typedef struct { int start; int end; int valid; } re_capture_t;

typedef struct {
    re_capture_t groups[16];
    int count;
    int depth;
} re_captures_t;

void re_cap_init(re_captures_t *c) {
    int i;
    c->count = 0;
    c->depth = 0;
    for (i = 0; i < 16; i++) { c->groups[i].start = -1; c->groups[i].end = -1; c->groups[i].valid = 0; }
}

void re_cap_open(re_captures_t *c, int pos) {
    if (c->count < 16) {
        c->groups[c->count].start = pos;
        c->groups[c->count].valid = 0;
        c->count = c->count + 1;
        c->depth = c->depth + 1;
    }
}

void re_cap_close(re_captures_t *c, int pos) {
    int i;
    if (c->depth <= 0) return;
    for (i = c->count - 1; i >= 0; i--) {
        if (c->groups[i].start >= 0 && !c->groups[i].valid) {
            c->groups[i].end = pos;
            c->groups[i].valid = 1;
            c->depth = c->depth - 1;
            return;
        }
    }
}

int re_cap_length(re_captures_t *c, int group) {
    if (group < 0 || group >= c->count || !c->groups[group].valid) return -1;
    return c->groups[group].end - c->groups[group].start;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1315 failed: {:?}", result.err());
}

// ============================================================================
// C1316-C1320: Regex Execution
// ============================================================================

/// C1316: Backtracking engine -- recursive descent regex matcher with backtracking
#[test]
fn c1316_backtracking_engine() {
    let c_code = r#"
int re_bt_match_char(char p, char t) { return p == '.' || p == t; }

int re_bt_match(const char *pat, int pi, int plen, const char *text, int ti, int tlen) {
    if (pi >= plen) return ti;
    if (pi + 1 < plen && pat[pi + 1] == '*') {
        int result = re_bt_match(pat, pi + 2, plen, text, ti, tlen);
        if (result >= 0) return result;
        while (ti < tlen && re_bt_match_char(pat[pi], text[ti])) {
            ti = ti + 1;
            result = re_bt_match(pat, pi + 2, plen, text, ti, tlen);
            if (result >= 0) return result;
        }
        return -1;
    }
    if (ti >= tlen) return -1;
    if (re_bt_match_char(pat[pi], text[ti]))
        return re_bt_match(pat, pi + 1, plen, text, ti + 1, tlen);
    return -1;
}

int re_bt_search(const char *pat, int plen, const char *text, int tlen) {
    int i;
    for (i = 0; i < tlen; i++) {
        int r = re_bt_match(pat, 0, plen, text, i, tlen);
        if (r >= 0) return i;
    }
    return -1;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1316 failed: {:?}", result.err());
}

/// C1317: NFA simulation -- parallel NFA execution via state set tracking
#[test]
fn c1317_nfa_simulation() {
    let c_code = r#"
typedef struct {
    int trans[32][128];
    int eps[32][4];
    int eps_cnt[32];
    int accepting[32];
    int nstates;
} re_nfa_t;

void re_nfa_sim_init(re_nfa_t *n) {
    int i, j;
    n->nstates = 0;
    for (i = 0; i < 32; i++) {
        n->accepting[i] = 0;
        n->eps_cnt[i] = 0;
        for (j = 0; j < 128; j++) n->trans[i][j] = -1;
        for (j = 0; j < 4; j++) n->eps[i][j] = -1;
    }
}

int re_nfa_sim_run(re_nfa_t *n, int start, const char *input, int len) {
    int curr[32], next[32];
    int csz = 1, nsz, i, j;
    curr[0] = start;
    for (i = 0; i < len; i++) {
        nsz = 0;
        int ch = (unsigned char)input[i];
        for (j = 0; j < csz; j++) {
            int s = curr[j];
            int t = (ch < 128) ? n->trans[s][ch] : -1;
            if (t >= 0 && nsz < 32) { next[nsz] = t; nsz = nsz + 1; }
        }
        csz = nsz;
        for (j = 0; j < csz; j++) curr[j] = next[j];
    }
    for (i = 0; i < csz; i++) {
        if (n->accepting[curr[i]]) return 1;
    }
    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1317 failed: {:?}", result.err());
}

/// C1318: Match extraction -- extract matched substrings and positions
#[test]
fn c1318_match_extraction() {
    let c_code = r#"
typedef struct { int start; int end; } re_match_t;

typedef struct {
    re_match_t matches[32];
    int count;
} re_match_result_t;

void re_match_init(re_match_result_t *r) { r->count = 0; }

void re_match_add(re_match_result_t *r, int start, int end) {
    if (r->count < 32) {
        r->matches[r->count].start = start;
        r->matches[r->count].end = end;
        r->count = r->count + 1;
    }
}

int re_match_length(re_match_result_t *r, int idx) {
    if (idx < 0 || idx >= r->count) return -1;
    return r->matches[idx].end - r->matches[idx].start;
}

int re_find_all(const char *text, int tlen, char pat, re_match_result_t *out) {
    int i, start;
    re_match_init(out);
    i = 0;
    while (i < tlen) {
        if (text[i] == pat) {
            start = i;
            while (i < tlen && text[i] == pat) i = i + 1;
            re_match_add(out, start, i);
        } else {
            i = i + 1;
        }
    }
    return out->count;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1318 failed: {:?}", result.err());
}

/// C1319: Greedy vs lazy matching -- greedy (.*) vs lazy (.*?) quantifier behavior
#[test]
fn c1319_greedy_vs_lazy() {
    let c_code = r#"
int re_greedy_star(const char *text, int pos, int len, char ch) {
    int end = pos;
    while (end < len && (ch == '.' || text[end] == ch)) end = end + 1;
    return end;
}

int re_lazy_star(const char *text, int pos, int len, char ch, char next) {
    while (pos < len) {
        if (text[pos] == next) return pos;
        if (ch != '.' && text[pos] != ch) return -1;
        pos = pos + 1;
    }
    return -1;
}

int re_greedy_match(const char *text, int tlen, char before, char after) {
    int start = re_greedy_star(text, 0, tlen, before);
    if (start <= 0) return -1;
    int i;
    for (i = start; i >= 0; i--) {
        if (text[i] == after) return i;
    }
    return -1;
}

int re_lazy_match(const char *text, int tlen, char before, char after) {
    return re_lazy_star(text, 0, tlen, before, after);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1319 failed: {:?}", result.err());
}

/// C1320: Lookahead -- positive and negative lookahead assertions
#[test]
fn c1320_lookahead() {
    let c_code = r#"
int re_str_eq(const char *a, const char *b, int len) {
    int i;
    for (i = 0; i < len; i++) { if (a[i] != b[i]) return 0; }
    return 1;
}

int re_lookahead_pos(const char *text, int pos, int tlen, const char *look, int llen) {
    if (pos + llen > tlen) return 0;
    return re_str_eq(text + pos, look, llen);
}

int re_lookahead_neg(const char *text, int pos, int tlen, const char *look, int llen) {
    return !re_lookahead_pos(text, pos, tlen, look, llen);
}

int re_match_with_lookahead(const char *text, int tlen, char pat, const char *look, int llen, int positive) {
    int i;
    for (i = 0; i < tlen; i++) {
        if (text[i] == pat || pat == '.') {
            int ahead = positive ? re_lookahead_pos(text, i + 1, tlen, look, llen)
                                 : re_lookahead_neg(text, i + 1, tlen, look, llen);
            if (ahead) return i;
        }
    }
    return -1;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1320 failed: {:?}", result.err());
}

// ============================================================================
// C1321-C1325: Specialized
// ============================================================================

/// C1321: Glob pattern -- shell-style glob matching with * and ?
#[test]
fn c1321_glob_pattern() {
    let c_code = r#"
int re_glob_match(const char *pat, const char *str) {
    while (*pat != '\0') {
        if (*pat == '*') {
            pat = pat + 1;
            if (*pat == '\0') return 1;
            while (*str != '\0') {
                if (re_glob_match(pat, str)) return 1;
                str = str + 1;
            }
            return 0;
        }
        if (*str == '\0') return 0;
        if (*pat == '?' || *pat == *str) {
            pat = pat + 1;
            str = str + 1;
        } else {
            return 0;
        }
    }
    return *str == '\0';
}

int re_glob_has_magic(const char *pat) {
    while (*pat != '\0') {
        if (*pat == '*' || *pat == '?' || *pat == '[') return 1;
        pat = pat + 1;
    }
    return 0;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1321 failed: {:?}", result.err());
}

/// C1322: Wildcard match -- simple wildcard with ? and * support
#[test]
fn c1322_wildcard_match() {
    let c_code = r#"
int re_wild_match_dp(const char *pat, int plen, const char *str, int slen) {
    int dp[33][33];
    int i, j;
    if (plen > 32 || slen > 32) return 0;
    dp[0][0] = 1;
    for (i = 1; i <= slen; i++) dp[0][i] = 0;
    for (i = 1; i <= plen; i++) {
        dp[i][0] = (pat[i - 1] == '*') ? dp[i - 1][0] : 0;
    }
    for (i = 1; i <= plen; i++) {
        for (j = 1; j <= slen; j++) {
            if (pat[i - 1] == '*') {
                dp[i][j] = dp[i - 1][j] || dp[i][j - 1];
            } else if (pat[i - 1] == '?' || pat[i - 1] == str[j - 1]) {
                dp[i][j] = dp[i - 1][j - 1];
            } else {
                dp[i][j] = 0;
            }
        }
    }
    return dp[plen][slen];
}

int re_wild_is_match(const char *pat, int plen, const char *str, int slen) {
    return re_wild_match_dp(pat, plen, str, slen);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1322 failed: {:?}", result.err());
}

/// C1323: String replace -- find and replace pattern occurrences in text
#[test]
fn c1323_string_replace() {
    let c_code = r#"
int re_str_find(const char *text, int tlen, const char *pat, int plen) {
    int i, j, ok;
    for (i = 0; i <= tlen - plen; i++) {
        ok = 1;
        for (j = 0; j < plen; j++) {
            if (text[i + j] != pat[j]) { ok = 0; break; }
        }
        if (ok) return i;
    }
    return -1;
}

int re_replace(const char *text, int tlen, const char *pat, int plen,
               const char *rep, int rlen, char *out, int maxout) {
    int olen = 0;
    int pos = 0;
    while (pos < tlen) {
        int found = re_str_find(text + pos, tlen - pos, pat, plen);
        if (found < 0) {
            while (pos < tlen && olen < maxout) { out[olen] = text[pos]; olen = olen + 1; pos = pos + 1; }
            break;
        }
        int i;
        for (i = 0; i < found && olen < maxout; i++) { out[olen] = text[pos + i]; olen = olen + 1; }
        for (i = 0; i < rlen && olen < maxout; i++) { out[olen] = rep[i]; olen = olen + 1; }
        pos = pos + found + plen;
    }
    if (olen < maxout) out[olen] = '\0';
    return olen;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1323 failed: {:?}", result.err());
}

/// C1324: Regex split -- split string by pattern delimiter into tokens
#[test]
fn c1324_regex_split() {
    let c_code = r#"
typedef struct { int start; int len; } re_token_t;

typedef struct {
    re_token_t tokens[32];
    int count;
} re_split_result_t;

void re_split_init(re_split_result_t *r) { r->count = 0; }

int re_split(const char *text, int tlen, char delim, re_split_result_t *out) {
    int i, start;
    re_split_init(out);
    start = 0;
    for (i = 0; i <= tlen; i++) {
        if (i == tlen || text[i] == delim) {
            if (i > start && out->count < 32) {
                out->tokens[out->count].start = start;
                out->tokens[out->count].len = i - start;
                out->count = out->count + 1;
            }
            start = i + 1;
        }
    }
    return out->count;
}

int re_split_multi(const char *text, int tlen, const char *delims, int dlen, re_split_result_t *out) {
    int i, j, start, is_delim;
    re_split_init(out);
    start = 0;
    for (i = 0; i <= tlen; i++) {
        is_delim = (i == tlen);
        for (j = 0; j < dlen && !is_delim; j++) {
            if (text[i] == delims[j]) is_delim = 1;
        }
        if (is_delim) {
            if (i > start && out->count < 32) {
                out->tokens[out->count].start = start;
                out->tokens[out->count].len = i - start;
                out->count = out->count + 1;
            }
            start = i + 1;
        }
    }
    return out->count;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1324 failed: {:?}", result.err());
}

/// C1325: Pattern cache/memoization -- compiled pattern cache with LRU eviction
#[test]
fn c1325_pattern_cache() {
    let c_code = r#"
typedef struct {
    char pattern[64];
    int plen;
    int compiled_id;
    int use_count;
    int last_used;
} re_cache_entry_t;

typedef struct {
    re_cache_entry_t entries[16];
    int count;
    int clock;
} re_pattern_cache_t;

void re_cache_init(re_pattern_cache_t *c) { c->count = 0; c->clock = 0; }

int re_cache_find(re_pattern_cache_t *c, const char *pat, int plen) {
    int i, j;
    for (i = 0; i < c->count; i++) {
        if (c->entries[i].plen != plen) continue;
        int eq = 1;
        for (j = 0; j < plen; j++) {
            if (c->entries[i].pattern[j] != pat[j]) { eq = 0; break; }
        }
        if (eq) {
            c->entries[i].use_count = c->entries[i].use_count + 1;
            c->entries[i].last_used = c->clock;
            c->clock = c->clock + 1;
            return c->entries[i].compiled_id;
        }
    }
    return -1;
}

int re_cache_evict_lru(re_pattern_cache_t *c) {
    int min_used = c->entries[0].last_used;
    int min_idx = 0;
    int i;
    for (i = 1; i < c->count; i++) {
        if (c->entries[i].last_used < min_used) { min_used = c->entries[i].last_used; min_idx = i; }
    }
    return min_idx;
}

int re_cache_insert(re_pattern_cache_t *c, const char *pat, int plen, int compiled_id) {
    int idx;
    if (plen >= 64) return -1;
    if (c->count < 16) {
        idx = c->count;
        c->count = c->count + 1;
    } else {
        idx = re_cache_evict_lru(c);
    }
    int i;
    for (i = 0; i < plen; i++) c->entries[idx].pattern[i] = pat[i];
    c->entries[idx].pattern[plen] = '\0';
    c->entries[idx].plen = plen;
    c->entries[idx].compiled_id = compiled_id;
    c->entries[idx].use_count = 1;
    c->entries[idx].last_used = c->clock;
    c->clock = c->clock + 1;
    return idx;
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C1325 failed: {:?}", result.err());
}
