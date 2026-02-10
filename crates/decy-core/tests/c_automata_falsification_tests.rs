//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C876-C900: Finite Automata & Formal Languages -- the kind of C code found
//! in compiler frontends, protocol validators, regex engines, model checkers,
//! and formal verification tools.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise finite automata, pushdown automata, Turing machines,
//! parser generators, and related formal language constructs -- all expressed
//! as valid C99 with array-based representations (no malloc/free, no includes).
//!
//! Organization:
//! - C876-C880: Classical automata (DFA, NFA, NFA→DFA, DFA minimization, regex→NFA)
//! - C881-C885: Parsing machines (PDA, Turing machine, CYK, LL(1), LR(0))
//! - C886-C890: Grammar analysis & machines (FIRST/FOLLOW, guarded SM, Petri net, Mealy, Moore)
//! - C891-C895: Extended automata (ε-NFA→NFA, regex backtrack, transducer, Büchi, tree automaton)
//! - C896-C900: Advanced models (weighted, timed, counter, cellular, alternating)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C876-C880: Classical Automata
// ============================================================================

/// C876: DFA simulator -- classic deterministic finite automaton
#[test]
fn c876_dfa_simulator() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int num_states;
    int num_symbols;
    int transitions[32][4];
    int accepting[32];
    int start_state;
} fa_dfa_t;

void fa_dfa_init(fa_dfa_t *dfa, int states, int symbols, int start) {
    int i, j;
    dfa->num_states = states;
    dfa->num_symbols = symbols;
    dfa->start_state = start;
    for (i = 0; i < 32; i++) {
        dfa->accepting[i] = 0;
        for (j = 0; j < 4; j++) {
            dfa->transitions[i][j] = -1;
        }
    }
}

void fa_dfa_set_transition(fa_dfa_t *dfa, int from, int symbol, int to) {
    if (from >= 0 && from < 32 && symbol >= 0 && symbol < 4) {
        dfa->transitions[from][symbol] = to;
    }
}

void fa_dfa_set_accepting(fa_dfa_t *dfa, int state) {
    if (state >= 0 && state < 32) {
        dfa->accepting[state] = 1;
    }
}

int fa_dfa_run(const fa_dfa_t *dfa, const int *input, int len) {
    int state = dfa->start_state;
    int i;
    for (i = 0; i < len; i++) {
        int sym = input[i];
        if (sym < 0 || sym >= dfa->num_symbols) return 0;
        state = dfa->transitions[state][sym];
        if (state < 0) return 0;
    }
    return dfa->accepting[state];
}

int fa_dfa_test(void) {
    fa_dfa_t dfa;
    fa_dfa_init(&dfa, 3, 2, 0);
    fa_dfa_set_transition(&dfa, 0, 0, 1);
    fa_dfa_set_transition(&dfa, 0, 1, 0);
    fa_dfa_set_transition(&dfa, 1, 0, 1);
    fa_dfa_set_transition(&dfa, 1, 1, 2);
    fa_dfa_set_transition(&dfa, 2, 0, 2);
    fa_dfa_set_transition(&dfa, 2, 1, 2);
    fa_dfa_set_accepting(&dfa, 2);
    int input1[3] = {0, 1, 0};
    int input2[2] = {1, 1};
    int r1 = fa_dfa_run(&dfa, input1, 3);
    int r2 = fa_dfa_run(&dfa, input2, 2);
    if (r1 != 1) return -1;
    if (r2 != 0) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C876: DFA simulator should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C876: empty output");
    assert!(code.contains("fn fa_dfa_init"), "C876: Should contain fa_dfa_init");
    assert!(code.contains("fn fa_dfa_run"), "C876: Should contain fa_dfa_run");
    assert!(code.contains("fn fa_dfa_test"), "C876: Should contain fa_dfa_test");
    Ok(())
}

/// C877: NFA simulator with subset construction state sets
#[test]
fn c877_nfa_simulator() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    int num_states;
    int num_symbols;
    uint32_t transitions[16][4];
    uint32_t accepting_mask;
    int start_state;
} fa_nfa_t;

void fa_nfa_init(fa_nfa_t *nfa, int states, int symbols, int start) {
    int i, j;
    nfa->num_states = states;
    nfa->num_symbols = symbols;
    nfa->start_state = start;
    nfa->accepting_mask = 0;
    for (i = 0; i < 16; i++) {
        for (j = 0; j < 4; j++) {
            nfa->transitions[i][j] = 0;
        }
    }
}

void fa_nfa_add_transition(fa_nfa_t *nfa, int from, int symbol, int to) {
    if (from >= 0 && from < 16 && symbol >= 0 && symbol < 4 && to >= 0 && to < 16) {
        nfa->transitions[from][symbol] |= (1u << to);
    }
}

void fa_nfa_set_accepting(fa_nfa_t *nfa, int state) {
    if (state >= 0 && state < 16) {
        nfa->accepting_mask |= (1u << state);
    }
}

uint32_t fa_nfa_step(const fa_nfa_t *nfa, uint32_t state_set, int symbol) {
    uint32_t next = 0;
    int i;
    for (i = 0; i < nfa->num_states; i++) {
        if (state_set & (1u << i)) {
            next |= nfa->transitions[i][symbol];
        }
    }
    return next;
}

int fa_nfa_run(const fa_nfa_t *nfa, const int *input, int len) {
    uint32_t current = (1u << nfa->start_state);
    int i;
    for (i = 0; i < len; i++) {
        int sym = input[i];
        if (sym < 0 || sym >= nfa->num_symbols) return 0;
        current = fa_nfa_step(nfa, current, sym);
        if (current == 0) return 0;
    }
    return (current & nfa->accepting_mask) != 0;
}

int fa_nfa_test(void) {
    fa_nfa_t nfa;
    fa_nfa_init(&nfa, 4, 2, 0);
    fa_nfa_add_transition(&nfa, 0, 0, 0);
    fa_nfa_add_transition(&nfa, 0, 0, 1);
    fa_nfa_add_transition(&nfa, 0, 1, 0);
    fa_nfa_add_transition(&nfa, 1, 1, 2);
    fa_nfa_add_transition(&nfa, 2, 0, 3);
    fa_nfa_set_accepting(&nfa, 3);
    int input1[3] = {0, 1, 0};
    int r = fa_nfa_run(&nfa, input1, 3);
    if (r != 1) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C877: NFA simulator should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C877: empty output");
    assert!(code.contains("fn fa_nfa_init"), "C877: Should contain fa_nfa_init");
    assert!(code.contains("fn fa_nfa_run"), "C877: Should contain fa_nfa_run");
    assert!(code.contains("fn fa_nfa_step"), "C877: Should contain fa_nfa_step");
    Ok(())
}

/// C878: NFA to DFA conversion via subset construction
#[test]
fn c878_nfa_to_dfa_conversion() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t nfa_trans[8][2];
    uint32_t nfa_accept;
    int nfa_states;
    int dfa_trans[64][2];
    int dfa_accept[64];
    uint32_t dfa_state_sets[64];
    int dfa_count;
} fa_converter_t;

void fa_conv_init(fa_converter_t *conv, int nfa_states) {
    int i, j;
    conv->nfa_states = nfa_states;
    conv->nfa_accept = 0;
    conv->dfa_count = 0;
    for (i = 0; i < 8; i++) {
        for (j = 0; j < 2; j++) {
            conv->nfa_trans[i][j] = 0;
        }
    }
    for (i = 0; i < 64; i++) {
        conv->dfa_accept[i] = 0;
        conv->dfa_state_sets[i] = 0;
        for (j = 0; j < 2; j++) {
            conv->dfa_trans[i][j] = -1;
        }
    }
}

static int fa_conv_find_or_add(fa_converter_t *conv, uint32_t state_set) {
    int i;
    for (i = 0; i < conv->dfa_count; i++) {
        if (conv->dfa_state_sets[i] == state_set) return i;
    }
    if (conv->dfa_count >= 64) return -1;
    int idx = conv->dfa_count;
    conv->dfa_state_sets[idx] = state_set;
    conv->dfa_accept[idx] = (state_set & conv->nfa_accept) != 0;
    conv->dfa_count++;
    return idx;
}

static uint32_t fa_conv_move(const fa_converter_t *conv, uint32_t set, int sym) {
    uint32_t result = 0;
    int i;
    for (i = 0; i < conv->nfa_states; i++) {
        if (set & (1u << i)) {
            result |= conv->nfa_trans[i][sym];
        }
    }
    return result;
}

int fa_conv_convert(fa_converter_t *conv, int start) {
    uint32_t start_set = (1u << start);
    int queue[64];
    int head = 0;
    int tail = 0;
    int s0 = fa_conv_find_or_add(conv, start_set);
    if (s0 < 0) return -1;
    queue[tail] = s0;
    tail++;
    while (head < tail) {
        int cur = queue[head];
        head++;
        int sym;
        for (sym = 0; sym < 2; sym++) {
            uint32_t next_set = fa_conv_move(conv, conv->dfa_state_sets[cur], sym);
            if (next_set == 0) continue;
            int next = fa_conv_find_or_add(conv, next_set);
            if (next < 0) return -2;
            conv->dfa_trans[cur][sym] = next;
            if (next >= head + (tail - head)) {
                queue[tail] = next;
                tail++;
            }
        }
    }
    return conv->dfa_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C878: NFA→DFA conversion should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C878: empty output");
    assert!(code.contains("fn fa_conv_init"), "C878: Should contain fa_conv_init");
    assert!(code.contains("fn fa_conv_convert"), "C878: Should contain fa_conv_convert");
    Ok(())
}

/// C879: DFA minimization via Hopcroft's algorithm (partition refinement)
#[test]
fn c879_dfa_minimization_hopcroft() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int trans[16][2];
    int accepting[16];
    int num_states;
    int partition[16];
    int num_groups;
} fa_minidfa_t;

void fa_mini_init(fa_minidfa_t *m, int states) {
    int i, j;
    m->num_states = states;
    m->num_groups = 0;
    for (i = 0; i < 16; i++) {
        m->accepting[i] = 0;
        m->partition[i] = 0;
        for (j = 0; j < 2; j++) {
            m->trans[i][j] = -1;
        }
    }
}

void fa_mini_initial_partition(fa_minidfa_t *m) {
    int i;
    m->num_groups = 2;
    for (i = 0; i < m->num_states; i++) {
        m->partition[i] = m->accepting[i] ? 1 : 0;
    }
}

static int fa_mini_target_group(const fa_minidfa_t *m, int state, int symbol) {
    int target = m->trans[state][symbol];
    if (target < 0) return -1;
    return m->partition[target];
}

int fa_mini_refine(fa_minidfa_t *m) {
    int changed = 1;
    int iterations = 0;
    while (changed && iterations < 100) {
        changed = 0;
        iterations++;
        int g;
        for (g = 0; g < m->num_groups; g++) {
            int first = -1;
            int i;
            for (i = 0; i < m->num_states; i++) {
                if (m->partition[i] != g) continue;
                if (first == -1) {
                    first = i;
                    continue;
                }
                int differs = 0;
                int s;
                for (s = 0; s < 2; s++) {
                    if (fa_mini_target_group(m, i, s) !=
                        fa_mini_target_group(m, first, s)) {
                        differs = 1;
                        break;
                    }
                }
                if (differs) {
                    m->partition[i] = m->num_groups;
                    changed = 1;
                }
            }
            if (changed) {
                m->num_groups++;
                break;
            }
        }
    }
    return m->num_groups;
}

int fa_mini_test(void) {
    fa_minidfa_t m;
    fa_mini_init(&m, 5);
    m.trans[0][0] = 1; m.trans[0][1] = 2;
    m.trans[1][0] = 3; m.trans[1][1] = 4;
    m.trans[2][0] = 3; m.trans[2][1] = 4;
    m.trans[3][0] = 3; m.trans[3][1] = 4;
    m.trans[4][0] = 3; m.trans[4][1] = 4;
    m.accepting[3] = 1;
    m.accepting[4] = 1;
    fa_mini_initial_partition(&m);
    int groups = fa_mini_refine(&m);
    if (groups < 2) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C879: DFA minimization should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C879: empty output");
    assert!(code.contains("fn fa_mini_init"), "C879: Should contain fa_mini_init");
    assert!(code.contains("fn fa_mini_refine"), "C879: Should contain fa_mini_refine");
    Ok(())
}

/// C880: Regular expression to NFA (Thompson's construction)
#[test]
fn c880_regex_to_nfa_thompson() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int from;
    int to;
    int symbol;
} fa_nfa_edge_t;

typedef struct {
    fa_nfa_edge_t edges[128];
    int num_edges;
    int num_states;
    int start;
    int accept;
} fa_thompson_t;

void fa_thompson_init(fa_thompson_t *t) {
    t->num_edges = 0;
    t->num_states = 0;
    t->start = -1;
    t->accept = -1;
}

static int fa_thompson_new_state(fa_thompson_t *t) {
    int s = t->num_states;
    t->num_states++;
    return s;
}

static void fa_thompson_add_edge(fa_thompson_t *t, int from, int to, int sym) {
    if (t->num_edges >= 128) return;
    t->edges[t->num_edges].from = from;
    t->edges[t->num_edges].to = to;
    t->edges[t->num_edges].symbol = sym;
    t->num_edges++;
}

void fa_thompson_literal(fa_thompson_t *t, int symbol) {
    int s0 = fa_thompson_new_state(t);
    int s1 = fa_thompson_new_state(t);
    fa_thompson_add_edge(t, s0, s1, symbol);
    t->start = s0;
    t->accept = s1;
}

void fa_thompson_concat(fa_thompson_t *t, int mid_accept, int mid_start) {
    fa_thompson_add_edge(t, mid_accept, mid_start, -1);
}

void fa_thompson_alternation(fa_thompson_t *t, int s1, int a1, int s2, int a2) {
    int new_start = fa_thompson_new_state(t);
    int new_accept = fa_thompson_new_state(t);
    fa_thompson_add_edge(t, new_start, s1, -1);
    fa_thompson_add_edge(t, new_start, s2, -1);
    fa_thompson_add_edge(t, a1, new_accept, -1);
    fa_thompson_add_edge(t, a2, new_accept, -1);
    t->start = new_start;
    t->accept = new_accept;
}

void fa_thompson_kleene_star(fa_thompson_t *t, int old_start, int old_accept) {
    int new_start = fa_thompson_new_state(t);
    int new_accept = fa_thompson_new_state(t);
    fa_thompson_add_edge(t, new_start, old_start, -1);
    fa_thompson_add_edge(t, new_start, new_accept, -1);
    fa_thompson_add_edge(t, old_accept, old_start, -1);
    fa_thompson_add_edge(t, old_accept, new_accept, -1);
    t->start = new_start;
    t->accept = new_accept;
}

int fa_thompson_test(void) {
    fa_thompson_t t;
    fa_thompson_init(&t);
    fa_thompson_literal(&t, 0);
    int s_a = t.start;
    int a_a = t.accept;
    fa_thompson_literal(&t, 1);
    int s_b = t.start;
    int a_b = t.accept;
    fa_thompson_concat(&t, a_a, s_b);
    fa_thompson_kleene_star(&t, s_a, a_b);
    if (t.num_states < 4) return -1;
    if (t.num_edges < 4) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C880: Thompson's construction should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C880: empty output");
    assert!(code.contains("fn fa_thompson_init"), "C880: Should contain fa_thompson_init");
    assert!(code.contains("fn fa_thompson_literal"), "C880: Should contain fa_thompson_literal");
    assert!(code.contains("fn fa_thompson_kleene_star"), "C880: Should contain fa_thompson_kleene_star");
    Ok(())
}

// ============================================================================
// C881-C885: Parsing Machines
// ============================================================================

/// C881: Pushdown automaton (PDA) for balanced parentheses
#[test]
fn c881_pushdown_automaton() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int stack[256];
    int top;
    int state;
    int num_states;
} fa_pda_t;

void fa_pda_init(fa_pda_t *pda, int states) {
    pda->top = -1;
    pda->state = 0;
    pda->num_states = states;
}

void fa_pda_push(fa_pda_t *pda, int symbol) {
    if (pda->top < 255) {
        pda->top++;
        pda->stack[pda->top] = symbol;
    }
}

int fa_pda_pop(fa_pda_t *pda) {
    if (pda->top < 0) return -1;
    int val = pda->stack[pda->top];
    pda->top--;
    return val;
}

int fa_pda_peek(const fa_pda_t *pda) {
    if (pda->top < 0) return -1;
    return pda->stack[pda->top];
}

int fa_pda_check_balanced(const int *input, int len) {
    fa_pda_t pda;
    fa_pda_init(&pda, 2);
    int i;
    for (i = 0; i < len; i++) {
        if (input[i] == 1) {
            fa_pda_push(&pda, 1);
        } else if (input[i] == 2) {
            int top = fa_pda_pop(&pda);
            if (top != 1) return 0;
        } else {
            return 0;
        }
    }
    return pda.top == -1;
}

int fa_pda_test(void) {
    int good[4] = {1, 1, 2, 2};
    int bad[4] = {1, 2, 2, 1};
    if (!fa_pda_check_balanced(good, 4)) return -1;
    if (fa_pda_check_balanced(bad, 4)) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C881: PDA should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C881: empty output");
    assert!(code.contains("fn fa_pda_init"), "C881: Should contain fa_pda_init");
    assert!(code.contains("fn fa_pda_push"), "C881: Should contain fa_pda_push");
    assert!(code.contains("fn fa_pda_check_balanced"), "C881: Should contain fa_pda_check_balanced");
    Ok(())
}

/// C882: Turing machine simulator (single tape)
#[test]
fn c882_turing_machine_simulator() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int state;
    int read_sym;
    int write_sym;
    int move_dir;
    int next_state;
} fa_tm_rule_t;

typedef struct {
    int tape[512];
    int head;
    int state;
    int num_rules;
    fa_tm_rule_t rules[64];
    int halt_accept;
    int halt_reject;
} fa_tm_t;

void fa_tm_init(fa_tm_t *tm, int accept, int reject) {
    int i;
    tm->head = 256;
    tm->state = 0;
    tm->num_rules = 0;
    tm->halt_accept = accept;
    tm->halt_reject = reject;
    for (i = 0; i < 512; i++) {
        tm->tape[i] = 0;
    }
}

void fa_tm_add_rule(fa_tm_t *tm, int st, int rd, int wr, int mv, int ns) {
    if (tm->num_rules >= 64) return;
    fa_tm_rule_t *r = &tm->rules[tm->num_rules];
    r->state = st;
    r->read_sym = rd;
    r->write_sym = wr;
    r->move_dir = mv;
    r->next_state = ns;
    tm->num_rules++;
}

int fa_tm_step(fa_tm_t *tm) {
    if (tm->state == tm->halt_accept) return 1;
    if (tm->state == tm->halt_reject) return -1;
    int sym = tm->tape[tm->head];
    int i;
    for (i = 0; i < tm->num_rules; i++) {
        if (tm->rules[i].state == tm->state &&
            tm->rules[i].read_sym == sym) {
            tm->tape[tm->head] = tm->rules[i].write_sym;
            tm->head += tm->rules[i].move_dir;
            if (tm->head < 0) tm->head = 0;
            if (tm->head >= 512) tm->head = 511;
            tm->state = tm->rules[i].next_state;
            return 0;
        }
    }
    return -1;
}

int fa_tm_run(fa_tm_t *tm, int max_steps) {
    int steps = 0;
    while (steps < max_steps) {
        int result = fa_tm_step(tm);
        if (result != 0) return result;
        steps++;
    }
    return 0;
}

int fa_tm_test(void) {
    fa_tm_t tm;
    fa_tm_init(&tm, 2, 3);
    fa_tm_add_rule(&tm, 0, 1, 1, 1, 0);
    fa_tm_add_rule(&tm, 0, 0, 0, 0, 2);
    tm.tape[256] = 1;
    tm.tape[257] = 1;
    tm.tape[258] = 1;
    int result = fa_tm_run(&tm, 100);
    if (result != 1) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C882: Turing machine should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C882: empty output");
    assert!(code.contains("fn fa_tm_init"), "C882: Should contain fa_tm_init");
    assert!(code.contains("fn fa_tm_step"), "C882: Should contain fa_tm_step");
    assert!(code.contains("fn fa_tm_run"), "C882: Should contain fa_tm_run");
    Ok(())
}

/// C883: CYK algorithm for context-free grammar parsing
#[test]
fn c883_cyk_parser() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    int lhs;
    int rhs1;
    int rhs2;
} fa_cyk_rule_t;

typedef struct {
    int lhs;
    int terminal;
} fa_cyk_term_t;

typedef struct {
    fa_cyk_rule_t rules[32];
    int num_rules;
    fa_cyk_term_t terminals[16];
    int num_terminals;
    uint32_t table[16][16];
    int start_symbol;
} fa_cyk_t;

void fa_cyk_init(fa_cyk_t *cyk, int start) {
    int i, j;
    cyk->num_rules = 0;
    cyk->num_terminals = 0;
    cyk->start_symbol = start;
    for (i = 0; i < 16; i++) {
        for (j = 0; j < 16; j++) {
            cyk->table[i][j] = 0;
        }
    }
}

void fa_cyk_add_rule(fa_cyk_t *cyk, int lhs, int rhs1, int rhs2) {
    if (cyk->num_rules >= 32) return;
    cyk->rules[cyk->num_rules].lhs = lhs;
    cyk->rules[cyk->num_rules].rhs1 = rhs1;
    cyk->rules[cyk->num_rules].rhs2 = rhs2;
    cyk->num_rules++;
}

void fa_cyk_add_terminal(fa_cyk_t *cyk, int lhs, int terminal) {
    if (cyk->num_terminals >= 16) return;
    cyk->terminals[cyk->num_terminals].lhs = lhs;
    cyk->terminals[cyk->num_terminals].terminal = terminal;
    cyk->num_terminals++;
}

int fa_cyk_parse(fa_cyk_t *cyk, const int *input, int n) {
    int i, j, k, r;
    for (i = 0; i < n; i++) {
        cyk->table[i][i] = 0;
        for (r = 0; r < cyk->num_terminals; r++) {
            if (cyk->terminals[r].terminal == input[i]) {
                cyk->table[i][i] |= (1u << cyk->terminals[r].lhs);
            }
        }
    }
    int len;
    for (len = 2; len <= n; len++) {
        for (i = 0; i <= n - len; i++) {
            j = i + len - 1;
            for (k = i; k < j; k++) {
                for (r = 0; r < cyk->num_rules; r++) {
                    if ((cyk->table[i][k] & (1u << cyk->rules[r].rhs1)) &&
                        (cyk->table[k + 1][j] & (1u << cyk->rules[r].rhs2))) {
                        cyk->table[i][j] |= (1u << cyk->rules[r].lhs);
                    }
                }
            }
        }
    }
    return (cyk->table[0][n - 1] & (1u << cyk->start_symbol)) != 0;
}

int fa_cyk_test(void) {
    fa_cyk_t cyk;
    fa_cyk_init(&cyk, 0);
    fa_cyk_add_rule(&cyk, 0, 1, 2);
    fa_cyk_add_terminal(&cyk, 1, 10);
    fa_cyk_add_terminal(&cyk, 2, 20);
    int input[2] = {10, 20};
    int result = fa_cyk_parse(&cyk, input, 2);
    if (result != 1) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C883: CYK parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C883: empty output");
    assert!(code.contains("fn fa_cyk_init"), "C883: Should contain fa_cyk_init");
    assert!(code.contains("fn fa_cyk_parse"), "C883: Should contain fa_cyk_parse");
    Ok(())
}

/// C884: LL(1) parser table construction
#[test]
fn c884_ll1_parser_table() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int lhs;
    int rhs[8];
    int rhs_len;
} fa_ll1_production_t;

typedef struct {
    fa_ll1_production_t productions[32];
    int num_productions;
    int parse_table[8][8];
    int num_nonterminals;
    int num_terminals;
} fa_ll1_t;

void fa_ll1_init(fa_ll1_t *parser, int nonterms, int terms) {
    int i, j;
    parser->num_productions = 0;
    parser->num_nonterminals = nonterms;
    parser->num_terminals = terms;
    for (i = 0; i < 8; i++) {
        for (j = 0; j < 8; j++) {
            parser->parse_table[i][j] = -1;
        }
    }
}

int fa_ll1_add_production(fa_ll1_t *parser, int lhs, const int *rhs, int len) {
    if (parser->num_productions >= 32) return -1;
    int idx = parser->num_productions;
    parser->productions[idx].lhs = lhs;
    parser->productions[idx].rhs_len = len;
    int i;
    for (i = 0; i < len && i < 8; i++) {
        parser->productions[idx].rhs[i] = rhs[i];
    }
    parser->num_productions++;
    return idx;
}

void fa_ll1_set_entry(fa_ll1_t *parser, int nonterm, int terminal, int prod) {
    if (nonterm >= 0 && nonterm < 8 && terminal >= 0 && terminal < 8) {
        parser->parse_table[nonterm][terminal] = prod;
    }
}

int fa_ll1_parse(const fa_ll1_t *parser, const int *input, int len) {
    int stack[64];
    int sp = 0;
    stack[sp] = 0;
    int pos = 0;
    int steps = 0;
    while (sp >= 0 && steps < 1000) {
        steps++;
        int top = stack[sp];
        if (top < parser->num_nonterminals) {
            int lookahead = (pos < len) ? input[pos] : parser->num_terminals - 1;
            int prod = parser->parse_table[top][lookahead];
            if (prod < 0) return 0;
            sp--;
            int i;
            for (i = parser->productions[prod].rhs_len - 1; i >= 0; i--) {
                sp++;
                if (sp >= 64) return 0;
                stack[sp] = parser->productions[prod].rhs[i];
            }
        } else {
            int terminal = top - parser->num_nonterminals;
            if (pos >= len) return 0;
            if (input[pos] != terminal) return 0;
            sp--;
            pos++;
        }
    }
    return (sp < 0 && pos == len);
}

int fa_ll1_test(void) {
    fa_ll1_t parser;
    fa_ll1_init(&parser, 2, 4);
    int rhs0[2] = {2, 3};
    int p0 = fa_ll1_add_production(&parser, 0, rhs0, 2);
    fa_ll1_set_entry(&parser, 0, 0, p0);
    int input[2] = {0, 1};
    int ok = fa_ll1_parse(&parser, input, 2);
    return ok ? 0 : -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C884: LL(1) parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C884: empty output");
    assert!(code.contains("fn fa_ll1_init"), "C884: Should contain fa_ll1_init");
    assert!(code.contains("fn fa_ll1_parse"), "C884: Should contain fa_ll1_parse");
    Ok(())
}

/// C885: LR(0) parser table with shift/reduce actions
#[test]
fn c885_lr0_parser_table() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int action_type;
    int value;
} fa_lr_action_t;

typedef struct {
    fa_lr_action_t action[16][8];
    int goto_table[16][4];
    int num_states;
    int num_terminals;
    int num_nonterminals;
    int prod_lhs[16];
    int prod_len[16];
    int num_productions;
} fa_lr0_t;

void fa_lr0_init(fa_lr0_t *lr, int states, int terms, int nonterms) {
    int i, j;
    lr->num_states = states;
    lr->num_terminals = terms;
    lr->num_nonterminals = nonterms;
    lr->num_productions = 0;
    for (i = 0; i < 16; i++) {
        for (j = 0; j < 8; j++) {
            lr->action[i][j].action_type = 0;
            lr->action[i][j].value = -1;
        }
        for (j = 0; j < 4; j++) {
            lr->goto_table[i][j] = -1;
        }
    }
}

void fa_lr0_set_shift(fa_lr0_t *lr, int state, int terminal, int next) {
    lr->action[state][terminal].action_type = 1;
    lr->action[state][terminal].value = next;
}

void fa_lr0_set_reduce(fa_lr0_t *lr, int state, int terminal, int prod) {
    lr->action[state][terminal].action_type = 2;
    lr->action[state][terminal].value = prod;
}

void fa_lr0_set_accept(fa_lr0_t *lr, int state, int terminal) {
    lr->action[state][terminal].action_type = 3;
    lr->action[state][terminal].value = 0;
}

int fa_lr0_parse(const fa_lr0_t *lr, const int *input, int len) {
    int stack[128];
    int sp = 0;
    stack[0] = 0;
    int pos = 0;
    int steps = 0;
    while (steps < 1000) {
        steps++;
        int state = stack[sp];
        int sym = (pos < len) ? input[pos] : lr->num_terminals - 1;
        fa_lr_action_t act = lr->action[state][sym];
        if (act.action_type == 1) {
            sp++;
            if (sp >= 128) return 0;
            stack[sp] = act.value;
            pos++;
        } else if (act.action_type == 2) {
            int prod = act.value;
            sp -= lr->prod_len[prod];
            if (sp < 0) return 0;
            int lhs = lr->prod_lhs[prod];
            int goto_state = lr->goto_table[stack[sp]][lhs];
            if (goto_state < 0) return 0;
            sp++;
            stack[sp] = goto_state;
        } else if (act.action_type == 3) {
            return 1;
        } else {
            return 0;
        }
    }
    return 0;
}

int fa_lr0_test(void) {
    fa_lr0_t lr;
    fa_lr0_init(&lr, 4, 3, 1);
    lr.prod_lhs[0] = 0;
    lr.prod_len[0] = 2;
    lr.num_productions = 1;
    fa_lr0_set_shift(&lr, 0, 0, 1);
    fa_lr0_set_shift(&lr, 1, 1, 2);
    fa_lr0_set_reduce(&lr, 2, 2, 0);
    lr.goto_table[0][0] = 3;
    fa_lr0_set_accept(&lr, 3, 2);
    int input[2] = {0, 1};
    return fa_lr0_parse(&lr, input, 2) ? 0 : -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C885: LR(0) parser should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C885: empty output");
    assert!(code.contains("fn fa_lr0_init"), "C885: Should contain fa_lr0_init");
    assert!(code.contains("fn fa_lr0_parse"), "C885: Should contain fa_lr0_parse");
    Ok(())
}

// ============================================================================
// C886-C890: Grammar Analysis & Machines
// ============================================================================

/// C886: FIRST/FOLLOW set computation for grammar analysis
#[test]
fn c886_first_follow_sets() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    int lhs;
    int rhs[8];
    int rhs_len;
} fa_grammar_prod_t;

typedef struct {
    fa_grammar_prod_t prods[32];
    int num_prods;
    int num_symbols;
    int num_nonterminals;
    uint32_t first_set[16];
    uint32_t follow_set[16];
    int nullable[16];
} fa_grammar_t;

void fa_grammar_init(fa_grammar_t *g, int nonterms, int total) {
    int i;
    g->num_prods = 0;
    g->num_nonterminals = nonterms;
    g->num_symbols = total;
    for (i = 0; i < 16; i++) {
        g->first_set[i] = 0;
        g->follow_set[i] = 0;
        g->nullable[i] = 0;
    }
}

void fa_grammar_add_prod(fa_grammar_t *g, int lhs, const int *rhs, int len) {
    if (g->num_prods >= 32) return;
    int idx = g->num_prods;
    g->prods[idx].lhs = lhs;
    g->prods[idx].rhs_len = len;
    int i;
    for (i = 0; i < len && i < 8; i++) {
        g->prods[idx].rhs[i] = rhs[i];
    }
    g->num_prods++;
}

void fa_compute_nullable(fa_grammar_t *g) {
    int changed = 1;
    while (changed) {
        changed = 0;
        int p;
        for (p = 0; p < g->num_prods; p++) {
            int lhs = g->prods[p].lhs;
            if (g->nullable[lhs]) continue;
            if (g->prods[p].rhs_len == 0) {
                g->nullable[lhs] = 1;
                changed = 1;
                continue;
            }
            int all_nullable = 1;
            int i;
            for (i = 0; i < g->prods[p].rhs_len; i++) {
                int sym = g->prods[p].rhs[i];
                if (sym >= g->num_nonterminals || !g->nullable[sym]) {
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

void fa_compute_first(fa_grammar_t *g) {
    int i;
    for (i = g->num_nonterminals; i < g->num_symbols; i++) {
        g->first_set[i] = (1u << i);
    }
    int changed = 1;
    while (changed) {
        changed = 0;
        int p;
        for (p = 0; p < g->num_prods; p++) {
            int lhs = g->prods[p].lhs;
            uint32_t old = g->first_set[lhs];
            int j;
            for (j = 0; j < g->prods[p].rhs_len; j++) {
                int sym = g->prods[p].rhs[j];
                g->first_set[lhs] |= g->first_set[sym];
                if (sym >= g->num_nonterminals || !g->nullable[sym]) break;
            }
            if (g->first_set[lhs] != old) changed = 1;
        }
    }
}

int fa_first_follow_test(void) {
    fa_grammar_t g;
    fa_grammar_init(&g, 2, 5);
    int rhs0[2] = {2, 1};
    fa_grammar_add_prod(&g, 0, rhs0, 2);
    int rhs1[1] = {3};
    fa_grammar_add_prod(&g, 1, rhs1, 1);
    fa_compute_nullable(&g);
    fa_compute_first(&g);
    if (g.first_set[0] == 0) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C886: FIRST/FOLLOW should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C886: empty output");
    assert!(code.contains("fn fa_grammar_init"), "C886: Should contain fa_grammar_init");
    assert!(code.contains("fn fa_compute_first"), "C886: Should contain fa_compute_first");
    assert!(code.contains("fn fa_compute_nullable"), "C886: Should contain fa_compute_nullable");
    Ok(())
}

/// C887: State machine with guards and actions
#[test]
fn c887_guarded_state_machine() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int current_state;
    int variables[8];
} fa_gsm_context_t;

typedef struct {
    int from_state;
    int event;
    int guard_var;
    int guard_op;
    int guard_val;
    int action_var;
    int action_val;
    int to_state;
} fa_gsm_transition_t;

typedef struct {
    fa_gsm_transition_t transitions[64];
    int num_transitions;
    int num_states;
} fa_gsm_t;

void fa_gsm_init(fa_gsm_t *sm, int states) {
    sm->num_transitions = 0;
    sm->num_states = states;
}

void fa_gsm_add_transition(fa_gsm_t *sm, int from, int ev, int gvar,
                           int gop, int gval, int avar, int aval, int to) {
    if (sm->num_transitions >= 64) return;
    fa_gsm_transition_t *t = &sm->transitions[sm->num_transitions];
    t->from_state = from;
    t->event = ev;
    t->guard_var = gvar;
    t->guard_op = gop;
    t->guard_val = gval;
    t->action_var = avar;
    t->action_val = aval;
    t->to_state = to;
    sm->num_transitions++;
}

static int fa_gsm_check_guard(const fa_gsm_context_t *ctx, const fa_gsm_transition_t *t) {
    int val = ctx->variables[t->guard_var];
    if (t->guard_op == 0) return val == t->guard_val;
    if (t->guard_op == 1) return val != t->guard_val;
    if (t->guard_op == 2) return val < t->guard_val;
    if (t->guard_op == 3) return val > t->guard_val;
    return 1;
}

int fa_gsm_process_event(const fa_gsm_t *sm, fa_gsm_context_t *ctx, int event) {
    int i;
    for (i = 0; i < sm->num_transitions; i++) {
        const fa_gsm_transition_t *t = &sm->transitions[i];
        if (t->from_state == ctx->current_state && t->event == event) {
            if (fa_gsm_check_guard(ctx, t)) {
                ctx->variables[t->action_var] = t->action_val;
                ctx->current_state = t->to_state;
                return 1;
            }
        }
    }
    return 0;
}

int fa_gsm_test(void) {
    fa_gsm_t sm;
    fa_gsm_init(&sm, 3);
    fa_gsm_add_transition(&sm, 0, 1, 0, 0, 0, 0, 1, 1);
    fa_gsm_add_transition(&sm, 1, 2, 0, 0, 1, 1, 10, 2);
    fa_gsm_context_t ctx;
    ctx.current_state = 0;
    int i;
    for (i = 0; i < 8; i++) ctx.variables[i] = 0;
    fa_gsm_process_event(&sm, &ctx, 1);
    if (ctx.current_state != 1) return -1;
    fa_gsm_process_event(&sm, &ctx, 2);
    if (ctx.current_state != 2) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C887: Guarded SM should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C887: empty output");
    assert!(code.contains("fn fa_gsm_init"), "C887: Should contain fa_gsm_init");
    assert!(code.contains("fn fa_gsm_process_event"), "C887: Should contain fa_gsm_process_event");
    Ok(())
}

/// C888: Petri net simulator
#[test]
fn c888_petri_net_simulator() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int places[16];
    int pre[16][16];
    int post[16][16];
    int num_places;
    int num_transitions;
} fa_petri_t;

void fa_petri_init(fa_petri_t *net, int places, int transitions) {
    int i, j;
    net->num_places = places;
    net->num_transitions = transitions;
    for (i = 0; i < 16; i++) {
        net->places[i] = 0;
        for (j = 0; j < 16; j++) {
            net->pre[i][j] = 0;
            net->post[i][j] = 0;
        }
    }
}

void fa_petri_set_tokens(fa_petri_t *net, int place, int tokens) {
    if (place >= 0 && place < net->num_places) {
        net->places[place] = tokens;
    }
}

void fa_petri_set_arc(fa_petri_t *net, int place, int trans, int pre, int post) {
    if (place >= 0 && place < 16 && trans >= 0 && trans < 16) {
        net->pre[trans][place] = pre;
        net->post[trans][place] = post;
    }
}

int fa_petri_is_enabled(const fa_petri_t *net, int trans) {
    int p;
    for (p = 0; p < net->num_places; p++) {
        if (net->places[p] < net->pre[trans][p]) return 0;
    }
    return 1;
}

int fa_petri_fire(fa_petri_t *net, int trans) {
    if (!fa_petri_is_enabled(net, trans)) return 0;
    int p;
    for (p = 0; p < net->num_places; p++) {
        net->places[p] -= net->pre[trans][p];
        net->places[p] += net->post[trans][p];
    }
    return 1;
}

int fa_petri_test(void) {
    fa_petri_t net;
    fa_petri_init(&net, 3, 2);
    fa_petri_set_tokens(&net, 0, 2);
    fa_petri_set_arc(&net, 0, 0, 1, 0);
    fa_petri_set_arc(&net, 1, 0, 0, 1);
    fa_petri_set_arc(&net, 1, 1, 1, 0);
    fa_petri_set_arc(&net, 2, 1, 0, 1);
    if (!fa_petri_is_enabled(&net, 0)) return -1;
    fa_petri_fire(&net, 0);
    if (net.places[0] != 1) return -2;
    if (net.places[1] != 1) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C888: Petri net should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C888: empty output");
    assert!(code.contains("fn fa_petri_init"), "C888: Should contain fa_petri_init");
    assert!(code.contains("fn fa_petri_fire"), "C888: Should contain fa_petri_fire");
    assert!(code.contains("fn fa_petri_is_enabled"), "C888: Should contain fa_petri_is_enabled");
    Ok(())
}

/// C889: Mealy machine (output depends on state + input)
#[test]
fn c889_mealy_machine() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int next_state[16][4];
    int output[16][4];
    int num_states;
    int num_inputs;
    int current_state;
} fa_mealy_t;

void fa_mealy_init(fa_mealy_t *m, int states, int inputs) {
    int i, j;
    m->num_states = states;
    m->num_inputs = inputs;
    m->current_state = 0;
    for (i = 0; i < 16; i++) {
        for (j = 0; j < 4; j++) {
            m->next_state[i][j] = 0;
            m->output[i][j] = 0;
        }
    }
}

void fa_mealy_set_transition(fa_mealy_t *m, int from, int input, int to, int out) {
    if (from >= 0 && from < 16 && input >= 0 && input < 4) {
        m->next_state[from][input] = to;
        m->output[from][input] = out;
    }
}

int fa_mealy_step(fa_mealy_t *m, int input) {
    if (input < 0 || input >= m->num_inputs) return -1;
    int out = m->output[m->current_state][input];
    m->current_state = m->next_state[m->current_state][input];
    return out;
}

int fa_mealy_run(fa_mealy_t *m, const int *inputs, int *outputs, int len) {
    int i;
    for (i = 0; i < len; i++) {
        outputs[i] = fa_mealy_step(m, inputs[i]);
        if (outputs[i] < 0) return i;
    }
    return len;
}

int fa_mealy_test(void) {
    fa_mealy_t m;
    fa_mealy_init(&m, 2, 2);
    fa_mealy_set_transition(&m, 0, 0, 0, 0);
    fa_mealy_set_transition(&m, 0, 1, 1, 1);
    fa_mealy_set_transition(&m, 1, 0, 0, 1);
    fa_mealy_set_transition(&m, 1, 1, 1, 0);
    int inputs[4] = {0, 1, 0, 1};
    int outputs[4];
    fa_mealy_run(&m, inputs, outputs, 4);
    if (outputs[0] != 0) return -1;
    if (outputs[1] != 1) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C889: Mealy machine should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C889: empty output");
    assert!(code.contains("fn fa_mealy_init"), "C889: Should contain fa_mealy_init");
    assert!(code.contains("fn fa_mealy_step"), "C889: Should contain fa_mealy_step");
    assert!(code.contains("fn fa_mealy_run"), "C889: Should contain fa_mealy_run");
    Ok(())
}

/// C890: Moore machine (output depends on state only)
#[test]
fn c890_moore_machine() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int next_state[16][4];
    int state_output[16];
    int num_states;
    int num_inputs;
    int current_state;
} fa_moore_t;

void fa_moore_init(fa_moore_t *m, int states, int inputs) {
    int i, j;
    m->num_states = states;
    m->num_inputs = inputs;
    m->current_state = 0;
    for (i = 0; i < 16; i++) {
        m->state_output[i] = 0;
        for (j = 0; j < 4; j++) {
            m->next_state[i][j] = 0;
        }
    }
}

void fa_moore_set_output(fa_moore_t *m, int state, int output) {
    if (state >= 0 && state < 16) {
        m->state_output[state] = output;
    }
}

void fa_moore_set_transition(fa_moore_t *m, int from, int input, int to) {
    if (from >= 0 && from < 16 && input >= 0 && input < 4) {
        m->next_state[from][input] = to;
    }
}

int fa_moore_step(fa_moore_t *m, int input) {
    if (input < 0 || input >= m->num_inputs) return -1;
    m->current_state = m->next_state[m->current_state][input];
    return m->state_output[m->current_state];
}

int fa_moore_get_output(const fa_moore_t *m) {
    return m->state_output[m->current_state];
}

int fa_moore_run(fa_moore_t *m, const int *inputs, int *outputs, int len) {
    int i;
    outputs[0] = fa_moore_get_output(m);
    for (i = 0; i < len; i++) {
        int out = fa_moore_step(m, inputs[i]);
        if (out < 0) return i;
        outputs[i + 1] = out;
    }
    return len;
}

int fa_moore_test(void) {
    fa_moore_t m;
    fa_moore_init(&m, 3, 2);
    fa_moore_set_output(&m, 0, 0);
    fa_moore_set_output(&m, 1, 1);
    fa_moore_set_output(&m, 2, 0);
    fa_moore_set_transition(&m, 0, 0, 0);
    fa_moore_set_transition(&m, 0, 1, 1);
    fa_moore_set_transition(&m, 1, 0, 2);
    fa_moore_set_transition(&m, 1, 1, 1);
    fa_moore_set_transition(&m, 2, 0, 0);
    fa_moore_set_transition(&m, 2, 1, 1);
    int inputs[3] = {1, 0, 1};
    int outputs[4];
    fa_moore_run(&m, inputs, outputs, 3);
    if (outputs[0] != 0) return -1;
    if (outputs[1] != 1) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C890: Moore machine should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C890: empty output");
    assert!(code.contains("fn fa_moore_init"), "C890: Should contain fa_moore_init");
    assert!(code.contains("fn fa_moore_step"), "C890: Should contain fa_moore_step");
    assert!(code.contains("fn fa_moore_run"), "C890: Should contain fa_moore_run");
    Ok(())
}

// ============================================================================
// C891-C895: Extended Automata
// ============================================================================

/// C891: Epsilon-NFA to NFA conversion (epsilon closure computation)
#[test]
fn c891_epsilon_nfa_to_nfa() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t eps_trans[16];
    uint32_t sym_trans[16][4];
    int num_states;
    int num_symbols;
    uint32_t accepting;
} fa_enfa_t;

void fa_enfa_init(fa_enfa_t *enfa, int states, int symbols) {
    int i, j;
    enfa->num_states = states;
    enfa->num_symbols = symbols;
    enfa->accepting = 0;
    for (i = 0; i < 16; i++) {
        enfa->eps_trans[i] = 0;
        for (j = 0; j < 4; j++) {
            enfa->sym_trans[i][j] = 0;
        }
    }
}

void fa_enfa_add_eps(fa_enfa_t *enfa, int from, int to) {
    if (from >= 0 && from < 16 && to >= 0 && to < 16) {
        enfa->eps_trans[from] |= (1u << to);
    }
}

void fa_enfa_add_sym(fa_enfa_t *enfa, int from, int sym, int to) {
    if (from >= 0 && from < 16 && sym >= 0 && sym < 4 && to >= 0 && to < 16) {
        enfa->sym_trans[from][sym] |= (1u << to);
    }
}

uint32_t fa_enfa_eps_closure(const fa_enfa_t *enfa, uint32_t states) {
    uint32_t closure = states;
    uint32_t prev = 0;
    while (closure != prev) {
        prev = closure;
        int i;
        for (i = 0; i < enfa->num_states; i++) {
            if (closure & (1u << i)) {
                closure |= enfa->eps_trans[i];
            }
        }
    }
    return closure;
}

void fa_enfa_to_nfa(const fa_enfa_t *enfa, uint32_t nfa_trans[16][4],
                     uint32_t *nfa_accept) {
    int s, sym;
    *nfa_accept = 0;
    for (s = 0; s < enfa->num_states; s++) {
        uint32_t closure_s = fa_enfa_eps_closure(enfa, (1u << s));
        if (closure_s & enfa->accepting) {
            *nfa_accept |= (1u << s);
        }
        for (sym = 0; sym < enfa->num_symbols; sym++) {
            uint32_t move = 0;
            int i;
            for (i = 0; i < enfa->num_states; i++) {
                if (closure_s & (1u << i)) {
                    move |= enfa->sym_trans[i][sym];
                }
            }
            nfa_trans[s][sym] = fa_enfa_eps_closure(enfa, move);
        }
    }
}

int fa_enfa_test(void) {
    fa_enfa_t enfa;
    fa_enfa_init(&enfa, 4, 2);
    fa_enfa_add_eps(&enfa, 0, 1);
    fa_enfa_add_sym(&enfa, 1, 0, 2);
    fa_enfa_add_eps(&enfa, 2, 3);
    enfa.accepting = (1u << 3);
    uint32_t nfa_trans[16][4];
    uint32_t nfa_accept;
    fa_enfa_to_nfa(&enfa, nfa_trans, &nfa_accept);
    if (!(nfa_accept & (1u << 0)) == 0 && (nfa_trans[0][0] == 0)) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C891: Epsilon-NFA should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C891: empty output");
    assert!(code.contains("fn fa_enfa_init"), "C891: Should contain fa_enfa_init");
    assert!(code.contains("fn fa_enfa_eps_closure"), "C891: Should contain fa_enfa_eps_closure");
    assert!(code.contains("fn fa_enfa_to_nfa"), "C891: Should contain fa_enfa_to_nfa");
    Ok(())
}

/// C892: Regular expression matcher with backtracking
#[test]
fn c892_regex_backtracking_matcher() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int type;
    int value;
    int quantifier;
} fa_regex_node_t;

typedef struct {
    fa_regex_node_t nodes[64];
    int num_nodes;
} fa_regex_pattern_t;

void fa_regex_init(fa_regex_pattern_t *pat) {
    pat->num_nodes = 0;
}

void fa_regex_add_literal(fa_regex_pattern_t *pat, int ch, int quant) {
    if (pat->num_nodes >= 64) return;
    pat->nodes[pat->num_nodes].type = 0;
    pat->nodes[pat->num_nodes].value = ch;
    pat->nodes[pat->num_nodes].quantifier = quant;
    pat->num_nodes++;
}

void fa_regex_add_dot(fa_regex_pattern_t *pat, int quant) {
    if (pat->num_nodes >= 64) return;
    pat->nodes[pat->num_nodes].type = 1;
    pat->nodes[pat->num_nodes].value = 0;
    pat->nodes[pat->num_nodes].quantifier = quant;
    pat->num_nodes++;
}

static int fa_regex_match_node(const fa_regex_node_t *node, int ch) {
    if (node->type == 1) return 1;
    return node->value == ch;
}

static int fa_regex_match_from(const fa_regex_pattern_t *pat, int ni,
                                const int *text, int ti, int tlen, int depth) {
    if (depth > 200) return 0;
    if (ni >= pat->num_nodes) return (ti == tlen);
    const fa_regex_node_t *node = &pat->nodes[ni];
    if (node->quantifier == 0) {
        if (ti >= tlen) return 0;
        if (!fa_regex_match_node(node, text[ti])) return 0;
        return fa_regex_match_from(pat, ni + 1, text, ti + 1, tlen, depth + 1);
    }
    if (node->quantifier == 1) {
        if (fa_regex_match_from(pat, ni + 1, text, ti, tlen, depth + 1)) return 1;
        int i;
        for (i = ti; i < tlen; i++) {
            if (!fa_regex_match_node(node, text[i])) break;
            if (fa_regex_match_from(pat, ni + 1, text, i + 1, tlen, depth + 1)) return 1;
        }
        return 0;
    }
    if (node->quantifier == 2) {
        if (ti >= tlen || !fa_regex_match_node(node, text[ti])) return 0;
        int i;
        for (i = ti + 1; i <= tlen; i++) {
            if (fa_regex_match_from(pat, ni + 1, text, i, tlen, depth + 1)) return 1;
            if (i < tlen && !fa_regex_match_node(node, text[i])) break;
        }
        return 0;
    }
    return 0;
}

int fa_regex_match(const fa_regex_pattern_t *pat, const int *text, int len) {
    return fa_regex_match_from(pat, 0, text, 0, len, 0);
}

int fa_regex_test(void) {
    fa_regex_pattern_t pat;
    fa_regex_init(&pat);
    fa_regex_add_literal(&pat, 1, 0);
    fa_regex_add_dot(&pat, 1);
    fa_regex_add_literal(&pat, 2, 0);
    int text1[3] = {1, 3, 2};
    int text2[2] = {1, 2};
    if (!fa_regex_match(&pat, text1, 3)) return -1;
    if (!fa_regex_match(&pat, text2, 2)) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C892: Regex backtracking should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C892: empty output");
    assert!(code.contains("fn fa_regex_init"), "C892: Should contain fa_regex_init");
    assert!(code.contains("fn fa_regex_match"), "C892: Should contain fa_regex_match");
    Ok(())
}

/// C893: Finite state transducer
#[test]
fn c893_finite_state_transducer() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int next_state[16][4];
    int output_symbol[16][4];
    int output_len[16][4];
    int num_states;
    int num_inputs;
    int current_state;
} fa_transducer_t;

void fa_trans_init(fa_transducer_t *t, int states, int inputs) {
    int i, j;
    t->num_states = states;
    t->num_inputs = inputs;
    t->current_state = 0;
    for (i = 0; i < 16; i++) {
        for (j = 0; j < 4; j++) {
            t->next_state[i][j] = -1;
            t->output_symbol[i][j] = -1;
            t->output_len[i][j] = 0;
        }
    }
}

void fa_trans_set(fa_transducer_t *t, int from, int in_sym, int to, int out_sym) {
    if (from >= 0 && from < 16 && in_sym >= 0 && in_sym < 4) {
        t->next_state[from][in_sym] = to;
        t->output_symbol[from][in_sym] = out_sym;
        t->output_len[from][in_sym] = 1;
    }
}

int fa_trans_run(fa_transducer_t *t, const int *input, int in_len,
                 int *output, int max_out) {
    int out_pos = 0;
    int i;
    t->current_state = 0;
    for (i = 0; i < in_len; i++) {
        int sym = input[i];
        if (sym < 0 || sym >= t->num_inputs) return -1;
        int ns = t->next_state[t->current_state][sym];
        if (ns < 0) return -1;
        if (t->output_len[t->current_state][sym] > 0 && out_pos < max_out) {
            output[out_pos] = t->output_symbol[t->current_state][sym];
            out_pos++;
        }
        t->current_state = ns;
    }
    return out_pos;
}

int fa_trans_test(void) {
    fa_transducer_t t;
    fa_trans_init(&t, 2, 2);
    fa_trans_set(&t, 0, 0, 0, 10);
    fa_trans_set(&t, 0, 1, 1, 20);
    fa_trans_set(&t, 1, 0, 0, 30);
    fa_trans_set(&t, 1, 1, 1, 40);
    int input[3] = {0, 1, 0};
    int output[3];
    int n = fa_trans_run(&t, input, 3, output, 3);
    if (n != 3) return -1;
    if (output[0] != 10) return -2;
    if (output[1] != 20) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C893: Transducer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C893: empty output");
    assert!(code.contains("fn fa_trans_init"), "C893: Should contain fa_trans_init");
    assert!(code.contains("fn fa_trans_run"), "C893: Should contain fa_trans_run");
    Ok(())
}

/// C894: Buchi automaton for infinite word acceptance (bounded prefix check)
#[test]
fn c894_buchi_automaton() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int transitions[8][4];
    int accepting[8];
    int num_states;
    int num_symbols;
    int start_state;
} fa_buchi_t;

void fa_buchi_init(fa_buchi_t *b, int states, int symbols, int start) {
    int i, j;
    b->num_states = states;
    b->num_symbols = symbols;
    b->start_state = start;
    for (i = 0; i < 8; i++) {
        b->accepting[i] = 0;
        for (j = 0; j < 4; j++) {
            b->transitions[i][j] = -1;
        }
    }
}

void fa_buchi_set_accepting(fa_buchi_t *b, int state) {
    if (state >= 0 && state < 8) b->accepting[state] = 1;
}

int fa_buchi_check_prefix(const fa_buchi_t *b, const int *word, int len) {
    int state = b->start_state;
    int accepting_visits = 0;
    int i;
    for (i = 0; i < len; i++) {
        int sym = word[i];
        if (sym < 0 || sym >= b->num_symbols) return -1;
        state = b->transitions[state][sym];
        if (state < 0) return -1;
        if (b->accepting[state]) accepting_visits++;
    }
    return accepting_visits;
}

int fa_buchi_check_lasso(const fa_buchi_t *b, const int *stem, int stem_len,
                          const int *loop, int loop_len) {
    int state = b->start_state;
    int i;
    for (i = 0; i < stem_len; i++) {
        state = b->transitions[state][stem[i]];
        if (state < 0) return 0;
    }
    int loop_start_state = state;
    int found_accepting = 0;
    for (i = 0; i < loop_len; i++) {
        state = b->transitions[state][loop[i]];
        if (state < 0) return 0;
        if (b->accepting[state]) found_accepting = 1;
    }
    if (state != loop_start_state) return 0;
    return found_accepting;
}

int fa_buchi_test(void) {
    fa_buchi_t b;
    fa_buchi_init(&b, 2, 2, 0);
    b.transitions[0][0] = 0;
    b.transitions[0][1] = 1;
    b.transitions[1][0] = 0;
    b.transitions[1][1] = 1;
    fa_buchi_set_accepting(&b, 1);
    int stem[1] = {1};
    int loop[1] = {1};
    int result = fa_buchi_check_lasso(&b, stem, 1, loop, 1);
    if (result != 1) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C894: Buchi automaton should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C894: empty output");
    assert!(code.contains("fn fa_buchi_init"), "C894: Should contain fa_buchi_init");
    assert!(code.contains("fn fa_buchi_check_lasso"), "C894: Should contain fa_buchi_check_lasso");
    Ok(())
}

/// C895: Tree automaton (bottom-up on binary trees stored in arrays)
#[test]
fn c895_tree_automaton() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int label[63];
    int left[63];
    int right[63];
    int num_nodes;
} fa_tree_t;

typedef struct {
    int leaf_state[8];
    int branch_state[8][8][8];
    int accepting[8];
    int num_states;
    int num_labels;
} fa_tree_aut_t;

void fa_tree_init(fa_tree_t *t) {
    int i;
    t->num_nodes = 0;
    for (i = 0; i < 63; i++) {
        t->label[i] = -1;
        t->left[i] = -1;
        t->right[i] = -1;
    }
}

int fa_tree_add_leaf(fa_tree_t *t, int lab) {
    if (t->num_nodes >= 63) return -1;
    int idx = t->num_nodes;
    t->label[idx] = lab;
    t->left[idx] = -1;
    t->right[idx] = -1;
    t->num_nodes++;
    return idx;
}

int fa_tree_add_branch(fa_tree_t *t, int lab, int l, int r) {
    if (t->num_nodes >= 63) return -1;
    int idx = t->num_nodes;
    t->label[idx] = lab;
    t->left[idx] = l;
    t->right[idx] = r;
    t->num_nodes++;
    return idx;
}

void fa_tree_aut_init(fa_tree_aut_t *a, int states, int labels) {
    int i, j, k;
    a->num_states = states;
    a->num_labels = labels;
    for (i = 0; i < 8; i++) {
        a->leaf_state[i] = -1;
        a->accepting[i] = 0;
        for (j = 0; j < 8; j++) {
            for (k = 0; k < 8; k++) {
                a->branch_state[i][j][k] = -1;
            }
        }
    }
}

int fa_tree_aut_run(const fa_tree_aut_t *a, const fa_tree_t *t, int node) {
    if (node < 0 || node >= t->num_nodes) return -1;
    int lab = t->label[node];
    if (t->left[node] == -1 && t->right[node] == -1) {
        if (lab >= 0 && lab < 8) return a->leaf_state[lab];
        return -1;
    }
    int ls = fa_tree_aut_run(a, t, t->left[node]);
    int rs = fa_tree_aut_run(a, t, t->right[node]);
    if (ls < 0 || rs < 0 || lab < 0 || lab >= 8) return -1;
    if (ls >= 8 || rs >= 8) return -1;
    return a->branch_state[lab][ls][rs];
}

int fa_tree_aut_accepts(const fa_tree_aut_t *a, const fa_tree_t *t, int root) {
    int state = fa_tree_aut_run(a, t, root);
    if (state < 0 || state >= 8) return 0;
    return a->accepting[state];
}

int fa_tree_aut_test(void) {
    fa_tree_t tree;
    fa_tree_init(&tree);
    int l0 = fa_tree_add_leaf(&tree, 0);
    int l1 = fa_tree_add_leaf(&tree, 1);
    int root = fa_tree_add_branch(&tree, 2, l0, l1);
    fa_tree_aut_t aut;
    fa_tree_aut_init(&aut, 3, 3);
    aut.leaf_state[0] = 0;
    aut.leaf_state[1] = 1;
    aut.branch_state[2][0][1] = 2;
    aut.accepting[2] = 1;
    int result = fa_tree_aut_accepts(&aut, &tree, root);
    if (result != 1) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C895: Tree automaton should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C895: empty output");
    assert!(code.contains("fn fa_tree_init"), "C895: Should contain fa_tree_init");
    assert!(code.contains("fn fa_tree_aut_run"), "C895: Should contain fa_tree_aut_run");
    assert!(code.contains("fn fa_tree_aut_accepts"), "C895: Should contain fa_tree_aut_accepts");
    Ok(())
}

// ============================================================================
// C896-C900: Advanced Models
// ============================================================================

/// C896: Weighted automaton (semiring over tropical weights)
#[test]
fn c896_weighted_automaton() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int next_state;
    int weight;
} fa_weighted_edge_t;

typedef struct {
    fa_weighted_edge_t trans[16][4];
    int initial_weight[16];
    int final_weight[16];
    int num_states;
    int num_symbols;
} fa_weighted_t;

void fa_weighted_init(fa_weighted_t *w, int states, int symbols) {
    int i, j;
    w->num_states = states;
    w->num_symbols = symbols;
    for (i = 0; i < 16; i++) {
        w->initial_weight[i] = 999999;
        w->final_weight[i] = 999999;
        for (j = 0; j < 4; j++) {
            w->trans[i][j].next_state = -1;
            w->trans[i][j].weight = 999999;
        }
    }
}

static int fa_weighted_min(int a, int b) {
    return (a < b) ? a : b;
}

int fa_weighted_shortest_path(const fa_weighted_t *w, const int *input, int len) {
    int dist[16];
    int next_dist[16];
    int i, j;
    for (i = 0; i < w->num_states; i++) {
        dist[i] = w->initial_weight[i];
    }
    for (i = 0; i < len; i++) {
        int sym = input[i];
        if (sym < 0 || sym >= w->num_symbols) return 999999;
        for (j = 0; j < w->num_states; j++) {
            next_dist[j] = 999999;
        }
        for (j = 0; j < w->num_states; j++) {
            if (dist[j] >= 999999) continue;
            int ns = w->trans[j][sym].next_state;
            if (ns < 0) continue;
            int cost = dist[j] + w->trans[j][sym].weight;
            next_dist[ns] = fa_weighted_min(next_dist[ns], cost);
        }
        for (j = 0; j < w->num_states; j++) {
            dist[j] = next_dist[j];
        }
    }
    int best = 999999;
    for (i = 0; i < w->num_states; i++) {
        if (dist[i] < 999999 && w->final_weight[i] < 999999) {
            int total = dist[i] + w->final_weight[i];
            best = fa_weighted_min(best, total);
        }
    }
    return best;
}

int fa_weighted_test(void) {
    fa_weighted_t w;
    fa_weighted_init(&w, 3, 2);
    w.initial_weight[0] = 0;
    w.final_weight[2] = 0;
    w.trans[0][0].next_state = 1; w.trans[0][0].weight = 3;
    w.trans[0][1].next_state = 2; w.trans[0][1].weight = 10;
    w.trans[1][0].next_state = 2; w.trans[1][0].weight = 2;
    w.trans[1][1].next_state = 1; w.trans[1][1].weight = 1;
    int input[2] = {0, 0};
    int cost = fa_weighted_shortest_path(&w, input, 2);
    if (cost != 5) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C896: Weighted automaton should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C896: empty output");
    assert!(code.contains("fn fa_weighted_init"), "C896: Should contain fa_weighted_init");
    assert!(code.contains("fn fa_weighted_shortest_path"), "C896: Should contain fa_weighted_shortest_path");
    Ok(())
}

/// C897: Timed automaton with clock constraints
#[test]
fn c897_timed_automaton() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int from_state;
    int to_state;
    int symbol;
    int clock_idx;
    int guard_op;
    int guard_val;
    int reset_clock;
} fa_timed_trans_t;

typedef struct {
    fa_timed_trans_t transitions[32];
    int num_transitions;
    int clocks[4];
    int num_clocks;
    int state;
    int accepting[8];
    int num_states;
} fa_timed_t;

void fa_timed_init(fa_timed_t *ta, int states, int clocks) {
    int i;
    ta->num_transitions = 0;
    ta->num_clocks = clocks;
    ta->num_states = states;
    ta->state = 0;
    for (i = 0; i < 4; i++) ta->clocks[i] = 0;
    for (i = 0; i < 8; i++) ta->accepting[i] = 0;
}

void fa_timed_add_trans(fa_timed_t *ta, int from, int to, int sym,
                        int clk, int op, int val, int reset) {
    if (ta->num_transitions >= 32) return;
    fa_timed_trans_t *t = &ta->transitions[ta->num_transitions];
    t->from_state = from;
    t->to_state = to;
    t->symbol = sym;
    t->clock_idx = clk;
    t->guard_op = op;
    t->guard_val = val;
    t->reset_clock = reset;
    ta->num_transitions++;
}

static int fa_timed_check_guard(int clock_val, int op, int val) {
    if (op == 0) return clock_val < val;
    if (op == 1) return clock_val <= val;
    if (op == 2) return clock_val == val;
    if (op == 3) return clock_val >= val;
    if (op == 4) return clock_val > val;
    return 1;
}

int fa_timed_step(fa_timed_t *ta, int symbol, int time_delta) {
    int i;
    for (i = 0; i < ta->num_clocks; i++) {
        ta->clocks[i] += time_delta;
    }
    for (i = 0; i < ta->num_transitions; i++) {
        fa_timed_trans_t *t = &ta->transitions[i];
        if (t->from_state != ta->state) continue;
        if (t->symbol != symbol) continue;
        if (!fa_timed_check_guard(ta->clocks[t->clock_idx], t->guard_op, t->guard_val))
            continue;
        ta->state = t->to_state;
        if (t->reset_clock >= 0 && t->reset_clock < ta->num_clocks) {
            ta->clocks[t->reset_clock] = 0;
        }
        return 1;
    }
    return 0;
}

int fa_timed_test(void) {
    fa_timed_t ta;
    fa_timed_init(&ta, 3, 1);
    ta.accepting[2] = 1;
    fa_timed_add_trans(&ta, 0, 1, 0, 0, 0, 10, 0);
    fa_timed_add_trans(&ta, 1, 2, 1, 0, 0, 5, -1);
    if (!fa_timed_step(&ta, 0, 3)) return -1;
    if (ta.state != 1) return -2;
    if (!fa_timed_step(&ta, 1, 2)) return -3;
    if (ta.state != 2) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C897: Timed automaton should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C897: empty output");
    assert!(code.contains("fn fa_timed_init"), "C897: Should contain fa_timed_init");
    assert!(code.contains("fn fa_timed_step"), "C897: Should contain fa_timed_step");
    Ok(())
}

/// C898: Counter machine (2-counter Minsky machine)
#[test]
fn c898_counter_machine() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int type;
    int counter;
    int next_if_nonzero;
    int next_if_zero;
    int next;
} fa_cm_instr_t;

typedef struct {
    fa_cm_instr_t program[32];
    int num_instructions;
    int counters[2];
    int pc;
    int halted;
} fa_counter_machine_t;

void fa_cm_init(fa_counter_machine_t *cm) {
    int i;
    cm->num_instructions = 0;
    cm->counters[0] = 0;
    cm->counters[1] = 0;
    cm->pc = 0;
    cm->halted = 0;
    for (i = 0; i < 32; i++) {
        cm->program[i].type = 0;
        cm->program[i].counter = 0;
        cm->program[i].next = 0;
        cm->program[i].next_if_nonzero = 0;
        cm->program[i].next_if_zero = 0;
    }
}

void fa_cm_add_inc(fa_counter_machine_t *cm, int counter, int next) {
    if (cm->num_instructions >= 32) return;
    int idx = cm->num_instructions;
    cm->program[idx].type = 1;
    cm->program[idx].counter = counter;
    cm->program[idx].next = next;
    cm->num_instructions++;
}

void fa_cm_add_dec_or_jump(fa_counter_machine_t *cm, int counter,
                           int next_nz, int next_z) {
    if (cm->num_instructions >= 32) return;
    int idx = cm->num_instructions;
    cm->program[idx].type = 2;
    cm->program[idx].counter = counter;
    cm->program[idx].next_if_nonzero = next_nz;
    cm->program[idx].next_if_zero = next_z;
    cm->num_instructions++;
}

void fa_cm_add_halt(fa_counter_machine_t *cm) {
    if (cm->num_instructions >= 32) return;
    cm->program[cm->num_instructions].type = 0;
    cm->num_instructions++;
}

int fa_cm_step(fa_counter_machine_t *cm) {
    if (cm->halted) return 0;
    if (cm->pc < 0 || cm->pc >= cm->num_instructions) {
        cm->halted = 1;
        return 0;
    }
    fa_cm_instr_t *instr = &cm->program[cm->pc];
    if (instr->type == 0) {
        cm->halted = 1;
        return 0;
    }
    if (instr->type == 1) {
        cm->counters[instr->counter]++;
        cm->pc = instr->next;
        return 1;
    }
    if (instr->type == 2) {
        if (cm->counters[instr->counter] > 0) {
            cm->counters[instr->counter]--;
            cm->pc = instr->next_if_nonzero;
        } else {
            cm->pc = instr->next_if_zero;
        }
        return 1;
    }
    cm->halted = 1;
    return 0;
}

int fa_cm_run(fa_counter_machine_t *cm, int max_steps) {
    int steps = 0;
    while (!cm->halted && steps < max_steps) {
        fa_cm_step(cm);
        steps++;
    }
    return steps;
}

int fa_cm_test(void) {
    fa_counter_machine_t cm;
    fa_cm_init(&cm);
    fa_cm_add_inc(&cm, 0, 1);
    fa_cm_add_inc(&cm, 0, 2);
    fa_cm_add_inc(&cm, 0, 3);
    fa_cm_add_dec_or_jump(&cm, 0, 4, 5);
    fa_cm_add_inc(&cm, 1, 3);
    fa_cm_add_halt(&cm);
    fa_cm_run(&cm, 100);
    if (!cm.halted) return -1;
    if (cm.counters[1] != 3) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C898: Counter machine should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C898: empty output");
    assert!(code.contains("fn fa_cm_init"), "C898: Should contain fa_cm_init");
    assert!(code.contains("fn fa_cm_step"), "C898: Should contain fa_cm_step");
    assert!(code.contains("fn fa_cm_run"), "C898: Should contain fa_cm_run");
    Ok(())
}

/// C899: Cellular automaton (1D Rule 110)
#[test]
fn c899_cellular_automaton_rule110() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int cells[128];
    int next_cells[128];
    int width;
    int rule;
} fa_cellular_t;

void fa_cell_init(fa_cellular_t *ca, int width, int rule) {
    int i;
    ca->width = width;
    ca->rule = rule;
    for (i = 0; i < 128; i++) {
        ca->cells[i] = 0;
        ca->next_cells[i] = 0;
    }
}

void fa_cell_set(fa_cellular_t *ca, int pos, int val) {
    if (pos >= 0 && pos < ca->width) {
        ca->cells[pos] = val;
    }
}

static int fa_cell_get(const fa_cellular_t *ca, int pos) {
    if (pos < 0 || pos >= ca->width) return 0;
    return ca->cells[pos];
}

void fa_cell_step(fa_cellular_t *ca) {
    int i;
    for (i = 0; i < ca->width; i++) {
        int left = fa_cell_get(ca, i - 1);
        int center = fa_cell_get(ca, i);
        int right = fa_cell_get(ca, i + 1);
        int pattern = (left << 2) | (center << 1) | right;
        ca->next_cells[i] = (ca->rule >> pattern) & 1;
    }
    for (i = 0; i < ca->width; i++) {
        ca->cells[i] = ca->next_cells[i];
    }
}

int fa_cell_count_alive(const fa_cellular_t *ca) {
    int count = 0;
    int i;
    for (i = 0; i < ca->width; i++) {
        if (ca->cells[i]) count++;
    }
    return count;
}

void fa_cell_run(fa_cellular_t *ca, int steps) {
    int i;
    for (i = 0; i < steps; i++) {
        fa_cell_step(ca);
    }
}

int fa_cell_test(void) {
    fa_cellular_t ca;
    fa_cell_init(&ca, 32, 110);
    fa_cell_set(&ca, 16, 1);
    fa_cell_run(&ca, 10);
    int alive = fa_cell_count_alive(&ca);
    if (alive == 0) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C899: Cellular automaton should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C899: empty output");
    assert!(code.contains("fn fa_cell_init"), "C899: Should contain fa_cell_init");
    assert!(code.contains("fn fa_cell_step"), "C899: Should contain fa_cell_step");
    assert!(code.contains("fn fa_cell_run"), "C899: Should contain fa_cell_run");
    Ok(())
}

/// C900: Alternating automaton (universal + existential states)
#[test]
fn c900_alternating_automaton() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t transitions[8][4];
    int state_type[8];
    int accepting[8];
    int num_states;
    int num_symbols;
    int start_state;
} fa_alt_t;

void fa_alt_init(fa_alt_t *a, int states, int symbols, int start) {
    int i, j;
    a->num_states = states;
    a->num_symbols = symbols;
    a->start_state = start;
    for (i = 0; i < 8; i++) {
        a->state_type[i] = 0;
        a->accepting[i] = 0;
        for (j = 0; j < 4; j++) {
            a->transitions[i][j] = 0;
        }
    }
}

void fa_alt_set_existential(fa_alt_t *a, int state) {
    if (state >= 0 && state < 8) a->state_type[state] = 0;
}

void fa_alt_set_universal(fa_alt_t *a, int state) {
    if (state >= 0 && state < 8) a->state_type[state] = 1;
}

static int fa_alt_eval(const fa_alt_t *a, const int *input, int pos, int len,
                        int state, int depth) {
    if (depth > 100) return 0;
    if (pos >= len) return a->accepting[state];
    int sym = input[pos];
    if (sym < 0 || sym >= a->num_symbols) return 0;
    uint32_t succs = a->transitions[state][sym];
    if (succs == 0) return 0;
    if (a->state_type[state] == 0) {
        int s;
        for (s = 0; s < a->num_states; s++) {
            if (succs & (1u << s)) {
                if (fa_alt_eval(a, input, pos + 1, len, s, depth + 1)) return 1;
            }
        }
        return 0;
    } else {
        int s;
        for (s = 0; s < a->num_states; s++) {
            if (succs & (1u << s)) {
                if (!fa_alt_eval(a, input, pos + 1, len, s, depth + 1)) return 0;
            }
        }
        return 1;
    }
}

int fa_alt_accepts(const fa_alt_t *a, const int *input, int len) {
    return fa_alt_eval(a, input, 0, len, a->start_state, 0);
}

int fa_alt_test(void) {
    fa_alt_t a;
    fa_alt_init(&a, 3, 2, 0);
    fa_alt_set_universal(&a, 0);
    fa_alt_set_existential(&a, 1);
    fa_alt_set_existential(&a, 2);
    a.transitions[0][0] = (1u << 1) | (1u << 2);
    a.transitions[1][0] = (1u << 1);
    a.transitions[1][1] = (1u << 1);
    a.transitions[2][0] = (1u << 2);
    a.transitions[2][1] = (1u << 2);
    a.accepting[1] = 1;
    a.accepting[2] = 1;
    int input[2] = {0, 0};
    int result = fa_alt_accepts(&a, input, 2);
    if (result != 1) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C900: Alternating automaton should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C900: empty output");
    assert!(code.contains("fn fa_alt_init"), "C900: Should contain fa_alt_init");
    assert!(code.contains("fn fa_alt_accepts"), "C900: Should contain fa_alt_accepts");
    Ok(())
}
