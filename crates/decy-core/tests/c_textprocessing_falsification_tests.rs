//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C601-C625: Text Processing and String Algorithm implementations -- the kind
//! of C code found in search engines, text editors, NLP systems, and
//! compilers. Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world text processing patterns commonly found in
//! GNU grep, sed, awk, libpcre, ICU, and similar text-processing libraries --
//! all expressed as valid C99 with array-based representations (no malloc/free).
//!
//! Organization:
//! - C601-C605: String search and comparison algorithms (KMP, Boyer-Moore, Rabin-Karp, Levenshtein, LCS)
//! - C606-C610: Pattern matching and indexing (regex NFA construction, NFA matching, trie, Aho-Corasick, suffix array)
//! - C611-C615: Encoding and phonetics (UTF-8 decode, UTF-8 encode, string intern, wildcard match, Soundex)
//! - C616-C620: NLP primitives (Metaphone, Porter stemmer, word frequency, n-gram, sentence tokenizer)
//! - C621-C625: Compression and ciphers (RLE, Caesar, Vigenere, ROT13, Morse)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C601-C605: String Search and Comparison Algorithms
// ============================================================================

#[test]
fn c601_kmp_string_search() {
    let c_code = r#"
void kmp_build_failure(const char *pattern, int pat_len, int *failure) {
    int i;
    int j;
    failure[0] = 0;
    j = 0;
    for (i = 1; i < pat_len; i++) {
        while (j > 0 && pattern[i] != pattern[j]) {
            j = failure[j - 1];
        }
        if (pattern[i] == pattern[j]) {
            j++;
        }
        failure[i] = j;
    }
}

int kmp_search(const char *text, int text_len, const char *pattern, int pat_len) {
    int failure[256];
    int i;
    int j;
    if (pat_len == 0) return 0;
    if (pat_len > 256) return -1;
    kmp_build_failure(pattern, pat_len, failure);
    j = 0;
    for (i = 0; i < text_len; i++) {
        while (j > 0 && text[i] != pattern[j]) {
            j = failure[j - 1];
        }
        if (text[i] == pattern[j]) {
            j++;
        }
        if (j == pat_len) {
            return i - pat_len + 1;
        }
    }
    return -1;
}

int kmp_count_occurrences(const char *text, int text_len, const char *pattern, int pat_len) {
    int failure[256];
    int i;
    int j;
    int count;
    if (pat_len == 0 || pat_len > 256) return 0;
    kmp_build_failure(pattern, pat_len, failure);
    j = 0;
    count = 0;
    for (i = 0; i < text_len; i++) {
        while (j > 0 && text[i] != pattern[j]) {
            j = failure[j - 1];
        }
        if (text[i] == pattern[j]) {
            j++;
        }
        if (j == pat_len) {
            count++;
            j = failure[j - 1];
        }
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C601: KMP string search should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C601: Output should not be empty");
    assert!(code.contains("fn kmp_build_failure"), "C601: Should contain kmp_build_failure");
    assert!(code.contains("fn kmp_search"), "C601: Should contain kmp_search");
}

#[test]
fn c602_boyer_moore_string_search() {
    let c_code = r#"
typedef unsigned char uint8_t;

void bm_bad_char_table(const char *pattern, int pat_len, int *bad_char) {
    int i;
    for (i = 0; i < 256; i++) {
        bad_char[i] = pat_len;
    }
    for (i = 0; i < pat_len - 1; i++) {
        bad_char[(uint8_t)pattern[i]] = pat_len - 1 - i;
    }
}

int bm_search(const char *text, int text_len, const char *pattern, int pat_len) {
    int bad_char[256];
    int i;
    int j;
    int shift;
    if (pat_len == 0) return 0;
    bm_bad_char_table(pattern, pat_len, bad_char);
    i = pat_len - 1;
    while (i < text_len) {
        j = pat_len - 1;
        while (j >= 0 && text[i] == pattern[j]) {
            i--;
            j--;
        }
        if (j < 0) {
            return i + 1;
        }
        shift = bad_char[(uint8_t)text[i]];
        if (shift < pat_len - j) {
            shift = pat_len - j;
        }
        i = i + shift;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C602: Boyer-Moore should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C602: Output should not be empty");
    assert!(code.contains("fn bm_bad_char_table"), "C602: Should contain bm_bad_char_table");
    assert!(code.contains("fn bm_search"), "C602: Should contain bm_search");
}

#[test]
fn c603_rabin_karp_string_search() {
    let c_code = r#"
typedef unsigned long uint32_t;

int rk_search(const char *text, int text_len, const char *pattern, int pat_len) {
    uint32_t base;
    uint32_t modulus;
    uint32_t pat_hash;
    uint32_t txt_hash;
    uint32_t h;
    int i;
    int j;
    int match_found;
    if (pat_len == 0) return 0;
    if (pat_len > text_len) return -1;
    base = 256;
    modulus = 1000000007;
    h = 1;
    for (i = 0; i < pat_len - 1; i++) {
        h = (h * base) % modulus;
    }
    pat_hash = 0;
    txt_hash = 0;
    for (i = 0; i < pat_len; i++) {
        pat_hash = (pat_hash * base + (uint32_t)pattern[i]) % modulus;
        txt_hash = (txt_hash * base + (uint32_t)text[i]) % modulus;
    }
    for (i = 0; i <= text_len - pat_len; i++) {
        if (pat_hash == txt_hash) {
            match_found = 1;
            for (j = 0; j < pat_len; j++) {
                if (text[i + j] != pattern[j]) {
                    match_found = 0;
                    break;
                }
            }
            if (match_found) return i;
        }
        if (i < text_len - pat_len) {
            txt_hash = (base * (txt_hash + modulus - h * (uint32_t)text[i] % modulus) + (uint32_t)text[i + pat_len]) % modulus;
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C603: Rabin-Karp should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C603: Output should not be empty");
    assert!(code.contains("fn rk_search"), "C603: Should contain rk_search");
}

#[test]
fn c604_levenshtein_distance() {
    let c_code = r#"
int lev_min3(int a, int b, int c) {
    int min;
    min = a;
    if (b < min) min = b;
    if (c < min) min = c;
    return min;
}

int levenshtein(const char *s, int s_len, const char *t, int t_len) {
    int dp[128][128];
    int i;
    int j;
    int cost;
    if (s_len > 127 || t_len > 127) return -1;
    for (i = 0; i <= s_len; i++) {
        dp[i][0] = i;
    }
    for (j = 0; j <= t_len; j++) {
        dp[0][j] = j;
    }
    for (i = 1; i <= s_len; i++) {
        for (j = 1; j <= t_len; j++) {
            if (s[i - 1] == t[j - 1]) {
                cost = 0;
            } else {
                cost = 1;
            }
            dp[i][j] = lev_min3(
                dp[i - 1][j] + 1,
                dp[i][j - 1] + 1,
                dp[i - 1][j - 1] + cost
            );
        }
    }
    return dp[s_len][t_len];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C604: Levenshtein distance should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C604: Output should not be empty");
    assert!(code.contains("fn levenshtein"), "C604: Should contain levenshtein");
    assert!(code.contains("fn lev_min3"), "C604: Should contain lev_min3");
}

#[test]
fn c605_longest_common_subsequence() {
    let c_code = r#"
int lcs_length(const char *a, int a_len, const char *b, int b_len) {
    int dp[128][128];
    int i;
    int j;
    if (a_len > 127 || b_len > 127) return -1;
    for (i = 0; i <= a_len; i++) {
        dp[i][0] = 0;
    }
    for (j = 0; j <= b_len; j++) {
        dp[0][j] = 0;
    }
    for (i = 1; i <= a_len; i++) {
        for (j = 1; j <= b_len; j++) {
            if (a[i - 1] == b[j - 1]) {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                if (dp[i - 1][j] > dp[i][j - 1]) {
                    dp[i][j] = dp[i - 1][j];
                } else {
                    dp[i][j] = dp[i][j - 1];
                }
            }
        }
    }
    return dp[a_len][b_len];
}

void lcs_recover(const char *a, int a_len, const char *b, int b_len, char *out, int *out_len) {
    int dp[128][128];
    int i;
    int j;
    int idx;
    if (a_len > 127 || b_len > 127) {
        *out_len = 0;
        return;
    }
    for (i = 0; i <= a_len; i++) {
        dp[i][0] = 0;
    }
    for (j = 0; j <= b_len; j++) {
        dp[0][j] = 0;
    }
    for (i = 1; i <= a_len; i++) {
        for (j = 1; j <= b_len; j++) {
            if (a[i - 1] == b[j - 1]) {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                if (dp[i - 1][j] > dp[i][j - 1]) {
                    dp[i][j] = dp[i - 1][j];
                } else {
                    dp[i][j] = dp[i][j - 1];
                }
            }
        }
    }
    idx = dp[a_len][b_len];
    *out_len = idx;
    i = a_len;
    j = b_len;
    while (i > 0 && j > 0) {
        if (a[i - 1] == b[j - 1]) {
            idx--;
            out[idx] = a[i - 1];
            i--;
            j--;
        } else if (dp[i - 1][j] > dp[i][j - 1]) {
            i--;
        } else {
            j--;
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C605: LCS should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C605: Output should not be empty");
    assert!(code.contains("fn lcs_length"), "C605: Should contain lcs_length");
    assert!(code.contains("fn lcs_recover"), "C605: Should contain lcs_recover");
}

// ============================================================================
// C606-C610: Pattern Matching and Indexing
// ============================================================================

#[test]
fn c606_regex_nfa_construction() {
    let c_code = r#"
typedef struct {
    int from;
    int to;
    char ch;
    int is_epsilon;
} nfa_edge_t;

typedef struct {
    nfa_edge_t edges[512];
    int edge_count;
    int state_count;
    int start;
    int accept;
} nfa_t;

void nfa_init(nfa_t *nfa) {
    nfa->edge_count = 0;
    nfa->state_count = 0;
    nfa->start = 0;
    nfa->accept = 0;
}

int nfa_new_state(nfa_t *nfa) {
    int s;
    s = nfa->state_count;
    nfa->state_count++;
    return s;
}

void nfa_add_edge(nfa_t *nfa, int from, int to, char ch, int is_epsilon) {
    if (nfa->edge_count >= 512) return;
    nfa->edges[nfa->edge_count].from = from;
    nfa->edges[nfa->edge_count].to = to;
    nfa->edges[nfa->edge_count].ch = ch;
    nfa->edges[nfa->edge_count].is_epsilon = is_epsilon;
    nfa->edge_count++;
}

void nfa_build_char(nfa_t *nfa, char c) {
    int s0;
    int s1;
    s0 = nfa_new_state(nfa);
    s1 = nfa_new_state(nfa);
    nfa_add_edge(nfa, s0, s1, c, 0);
    nfa->start = s0;
    nfa->accept = s1;
}

void nfa_build_concat(nfa_t *nfa, int s1_start, int s1_accept, int s2_start, int s2_accept) {
    nfa_add_edge(nfa, s1_accept, s2_start, 0, 1);
    nfa->start = s1_start;
    nfa->accept = s2_accept;
}

void nfa_build_union(nfa_t *nfa, int s1_start, int s1_accept, int s2_start, int s2_accept) {
    int new_start;
    int new_accept;
    new_start = nfa_new_state(nfa);
    new_accept = nfa_new_state(nfa);
    nfa_add_edge(nfa, new_start, s1_start, 0, 1);
    nfa_add_edge(nfa, new_start, s2_start, 0, 1);
    nfa_add_edge(nfa, s1_accept, new_accept, 0, 1);
    nfa_add_edge(nfa, s2_accept, new_accept, 0, 1);
    nfa->start = new_start;
    nfa->accept = new_accept;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C606: Regex NFA construction should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C606: Output should not be empty");
    assert!(code.contains("fn nfa_init"), "C606: Should contain nfa_init");
    assert!(code.contains("fn nfa_add_edge"), "C606: Should contain nfa_add_edge");
}

#[test]
fn c607_regex_nfa_matching() {
    let c_code = r#"
typedef struct {
    int from;
    int to;
    char ch;
    int is_epsilon;
} nfa2_edge_t;

typedef struct {
    nfa2_edge_t edges[256];
    int edge_count;
    int state_count;
    int start;
    int accept;
} nfa2_t;

void nfa2_epsilon_closure(const nfa2_t *nfa, const int *states, int count, int *out, int *out_count) {
    int stack[64];
    int visited[64];
    int sp;
    int i;
    int s;
    int e;
    for (i = 0; i < 64; i++) {
        visited[i] = 0;
    }
    sp = 0;
    for (i = 0; i < count; i++) {
        stack[sp] = states[i];
        sp++;
        visited[states[i]] = 1;
    }
    *out_count = 0;
    while (sp > 0) {
        sp--;
        s = stack[sp];
        out[*out_count] = s;
        (*out_count)++;
        for (e = 0; e < nfa->edge_count; e++) {
            if (nfa->edges[e].from == s && nfa->edges[e].is_epsilon == 1) {
                if (visited[nfa->edges[e].to] == 0) {
                    visited[nfa->edges[e].to] = 1;
                    stack[sp] = nfa->edges[e].to;
                    sp++;
                }
            }
        }
    }
}

int nfa2_match(const nfa2_t *nfa, const char *input, int input_len) {
    int current[64];
    int next[64];
    int temp[64];
    int cur_count;
    int next_count;
    int i;
    int e;
    int start_arr[1];
    start_arr[0] = nfa->start;
    nfa2_epsilon_closure(nfa, start_arr, 1, current, &cur_count);
    for (i = 0; i < input_len; i++) {
        next_count = 0;
        for (e = 0; e < nfa->edge_count; e++) {
            int s;
            for (s = 0; s < cur_count; s++) {
                if (nfa->edges[e].from == current[s] && nfa->edges[e].is_epsilon == 0 && nfa->edges[e].ch == input[i]) {
                    next[next_count] = nfa->edges[e].to;
                    next_count++;
                }
            }
        }
        nfa2_epsilon_closure(nfa, next, next_count, temp, &cur_count);
        for (e = 0; e < cur_count; e++) {
            current[e] = temp[e];
        }
    }
    for (i = 0; i < cur_count; i++) {
        if (current[i] == nfa->accept) return 1;
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C607: Regex NFA matching should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C607: Output should not be empty");
    assert!(code.contains("fn nfa2_epsilon_closure"), "C607: Should contain nfa2_epsilon_closure");
    assert!(code.contains("fn nfa2_match"), "C607: Should contain nfa2_match");
}

#[test]
fn c608_trie_autocomplete() {
    let c_code = r#"
typedef struct {
    int children[26];
    int is_end;
    int count;
} trie_node_t;

typedef struct {
    trie_node_t nodes[1024];
    int node_count;
} trie_t;

void trie_init(trie_t *trie) {
    int i;
    trie->node_count = 1;
    for (i = 0; i < 26; i++) {
        trie->nodes[0].children[i] = -1;
    }
    trie->nodes[0].is_end = 0;
    trie->nodes[0].count = 0;
}

int trie_alloc_node(trie_t *trie) {
    int idx;
    int i;
    if (trie->node_count >= 1024) return -1;
    idx = trie->node_count;
    trie->node_count++;
    for (i = 0; i < 26; i++) {
        trie->nodes[idx].children[i] = -1;
    }
    trie->nodes[idx].is_end = 0;
    trie->nodes[idx].count = 0;
    return idx;
}

void trie_insert(trie_t *trie, const char *word, int len) {
    int node;
    int i;
    int c;
    node = 0;
    for (i = 0; i < len; i++) {
        c = word[i] - 'a';
        if (c < 0 || c >= 26) return;
        if (trie->nodes[node].children[c] == -1) {
            trie->nodes[node].children[c] = trie_alloc_node(trie);
            if (trie->nodes[node].children[c] == -1) return;
        }
        node = trie->nodes[node].children[c];
        trie->nodes[node].count++;
    }
    trie->nodes[node].is_end = 1;
}

int trie_search(const trie_t *trie, const char *word, int len) {
    int node;
    int i;
    int c;
    node = 0;
    for (i = 0; i < len; i++) {
        c = word[i] - 'a';
        if (c < 0 || c >= 26) return 0;
        if (trie->nodes[node].children[c] == -1) return 0;
        node = trie->nodes[node].children[c];
    }
    return trie->nodes[node].is_end;
}

int trie_count_prefix(const trie_t *trie, const char *prefix, int len) {
    int node;
    int i;
    int c;
    node = 0;
    for (i = 0; i < len; i++) {
        c = prefix[i] - 'a';
        if (c < 0 || c >= 26) return 0;
        if (trie->nodes[node].children[c] == -1) return 0;
        node = trie->nodes[node].children[c];
    }
    return trie->nodes[node].count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C608: Trie autocomplete should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C608: Output should not be empty");
    assert!(code.contains("fn trie_init"), "C608: Should contain trie_init");
    assert!(code.contains("fn trie_insert"), "C608: Should contain trie_insert");
    assert!(code.contains("fn trie_search"), "C608: Should contain trie_search");
}

#[test]
fn c609_aho_corasick_multi_pattern_search() {
    let c_code = r#"
typedef struct {
    int go[26];
    int fail;
    int output;
    int depth;
} ac_node_t;

typedef struct {
    ac_node_t nodes[512];
    int node_count;
} ac_t;

void ac_init(ac_t *ac) {
    int i;
    ac->node_count = 1;
    for (i = 0; i < 26; i++) {
        ac->nodes[0].go[i] = -1;
    }
    ac->nodes[0].fail = 0;
    ac->nodes[0].output = 0;
    ac->nodes[0].depth = 0;
}

int ac_alloc(ac_t *ac) {
    int idx;
    int i;
    if (ac->node_count >= 512) return -1;
    idx = ac->node_count;
    ac->node_count++;
    for (i = 0; i < 26; i++) {
        ac->nodes[idx].go[i] = -1;
    }
    ac->nodes[idx].fail = 0;
    ac->nodes[idx].output = 0;
    ac->nodes[idx].depth = 0;
    return idx;
}

void ac_add_pattern(ac_t *ac, const char *pat, int pat_len, int pat_id) {
    int node;
    int i;
    int c;
    node = 0;
    for (i = 0; i < pat_len; i++) {
        c = pat[i] - 'a';
        if (c < 0 || c >= 26) return;
        if (ac->nodes[node].go[c] == -1) {
            ac->nodes[node].go[c] = ac_alloc(ac);
            if (ac->nodes[node].go[c] == -1) return;
        }
        node = ac->nodes[node].go[c];
        ac->nodes[node].depth = i + 1;
    }
    ac->nodes[node].output = pat_id + 1;
}

void ac_build_fail(ac_t *ac) {
    int queue[512];
    int front;
    int back;
    int u;
    int c;
    int v;
    int f;
    front = 0;
    back = 0;
    for (c = 0; c < 26; c++) {
        if (ac->nodes[0].go[c] != -1) {
            ac->nodes[ac->nodes[0].go[c]].fail = 0;
            queue[back] = ac->nodes[0].go[c];
            back++;
        } else {
            ac->nodes[0].go[c] = 0;
        }
    }
    while (front < back) {
        u = queue[front];
        front++;
        for (c = 0; c < 26; c++) {
            v = ac->nodes[u].go[c];
            if (v != -1) {
                f = ac->nodes[u].fail;
                while (f != 0 && ac->nodes[f].go[c] == -1) {
                    f = ac->nodes[f].fail;
                }
                ac->nodes[v].fail = ac->nodes[f].go[c];
                if (ac->nodes[v].fail == v) {
                    ac->nodes[v].fail = 0;
                }
                queue[back] = v;
                back++;
            }
        }
    }
}

int ac_search(const ac_t *ac, const char *text, int text_len, int *matches, int max_matches) {
    int state;
    int match_count;
    int i;
    int c;
    int temp;
    state = 0;
    match_count = 0;
    for (i = 0; i < text_len; i++) {
        c = text[i] - 'a';
        if (c < 0 || c >= 26) {
            state = 0;
            continue;
        }
        while (state != 0 && ac->nodes[state].go[c] == -1) {
            state = ac->nodes[state].fail;
        }
        if (ac->nodes[state].go[c] != -1) {
            state = ac->nodes[state].go[c];
        }
        temp = state;
        while (temp != 0) {
            if (ac->nodes[temp].output > 0 && match_count < max_matches) {
                matches[match_count] = i - ac->nodes[temp].depth + 1;
                match_count++;
            }
            temp = ac->nodes[temp].fail;
        }
    }
    return match_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C609: Aho-Corasick should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C609: Output should not be empty");
    assert!(code.contains("fn ac_init"), "C609: Should contain ac_init");
    assert!(code.contains("fn ac_build_fail"), "C609: Should contain ac_build_fail");
    assert!(code.contains("fn ac_search"), "C609: Should contain ac_search");
}

#[test]
fn c610_suffix_array_construction() {
    let c_code = r#"
void sa_build_naive(const char *text, int n, int *sa) {
    int i;
    int j;
    int temp;
    int k;
    for (i = 0; i < n; i++) {
        sa[i] = i;
    }
    for (i = 0; i < n - 1; i++) {
        for (j = i + 1; j < n; j++) {
            k = 0;
            while (sa[i] + k < n && sa[j] + k < n) {
                if (text[sa[i] + k] < text[sa[j] + k]) {
                    break;
                }
                if (text[sa[i] + k] > text[sa[j] + k]) {
                    temp = sa[i];
                    sa[i] = sa[j];
                    sa[j] = temp;
                    break;
                }
                k++;
            }
            if (sa[i] + k >= n && sa[j] + k < n) {
                temp = sa[i];
                sa[i] = sa[j];
                sa[j] = temp;
            }
        }
    }
}

int sa_binary_search(const char *text, int n, const int *sa, const char *pattern, int pat_len) {
    int lo;
    int hi;
    int mid;
    int k;
    int cmp;
    lo = 0;
    hi = n - 1;
    while (lo <= hi) {
        mid = lo + (hi - lo) / 2;
        cmp = 0;
        for (k = 0; k < pat_len; k++) {
            if (sa[mid] + k >= n) {
                cmp = -1;
                break;
            }
            if (text[sa[mid] + k] < pattern[k]) {
                cmp = -1;
                break;
            }
            if (text[sa[mid] + k] > pattern[k]) {
                cmp = 1;
                break;
            }
        }
        if (cmp == 0) return sa[mid];
        if (cmp < 0) {
            lo = mid + 1;
        } else {
            hi = mid - 1;
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C610: Suffix array should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C610: Output should not be empty");
    assert!(code.contains("fn sa_build_naive"), "C610: Should contain sa_build_naive");
    assert!(code.contains("fn sa_binary_search"), "C610: Should contain sa_binary_search");
}

// ============================================================================
// C611-C615: Encoding and Phonetics
// ============================================================================

#[test]
fn c611_utf8_byte_decoder() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

int utf8_decode(const uint8_t *buf, int buf_len, uint32_t *codepoint) {
    uint8_t b0;
    if (buf_len < 1) return 0;
    b0 = buf[0];
    if (b0 < 0x80) {
        *codepoint = b0;
        return 1;
    }
    if ((b0 & 0xE0) == 0xC0) {
        if (buf_len < 2) return 0;
        if ((buf[1] & 0xC0) != 0x80) return -1;
        *codepoint = ((uint32_t)(b0 & 0x1F) << 6) | (uint32_t)(buf[1] & 0x3F);
        if (*codepoint < 0x80) return -1;
        return 2;
    }
    if ((b0 & 0xF0) == 0xE0) {
        if (buf_len < 3) return 0;
        if ((buf[1] & 0xC0) != 0x80) return -1;
        if ((buf[2] & 0xC0) != 0x80) return -1;
        *codepoint = ((uint32_t)(b0 & 0x0F) << 12)
                   | ((uint32_t)(buf[1] & 0x3F) << 6)
                   | (uint32_t)(buf[2] & 0x3F);
        if (*codepoint < 0x800) return -1;
        return 3;
    }
    if ((b0 & 0xF8) == 0xF0) {
        if (buf_len < 4) return 0;
        if ((buf[1] & 0xC0) != 0x80) return -1;
        if ((buf[2] & 0xC0) != 0x80) return -1;
        if ((buf[3] & 0xC0) != 0x80) return -1;
        *codepoint = ((uint32_t)(b0 & 0x07) << 18)
                   | ((uint32_t)(buf[1] & 0x3F) << 12)
                   | ((uint32_t)(buf[2] & 0x3F) << 6)
                   | (uint32_t)(buf[3] & 0x3F);
        if (*codepoint < 0x10000) return -1;
        if (*codepoint > 0x10FFFF) return -1;
        return 4;
    }
    return -1;
}

int utf8_string_len(const uint8_t *buf, int byte_len) {
    int pos;
    int count;
    uint32_t cp;
    int consumed;
    pos = 0;
    count = 0;
    while (pos < byte_len) {
        consumed = utf8_decode(buf + pos, byte_len - pos, &cp);
        if (consumed <= 0) return -1;
        pos = pos + consumed;
        count++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C611: UTF-8 decoder should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C611: Output should not be empty");
    assert!(code.contains("fn utf8_decode"), "C611: Should contain utf8_decode");
    assert!(code.contains("fn utf8_string_len"), "C611: Should contain utf8_string_len");
}

#[test]
fn c612_utf8_codepoint_encoder() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

int utf8_encode(uint32_t cp, uint8_t *buf, int buf_len) {
    if (cp < 0x80) {
        if (buf_len < 1) return 0;
        buf[0] = (uint8_t)cp;
        return 1;
    }
    if (cp < 0x800) {
        if (buf_len < 2) return 0;
        buf[0] = (uint8_t)(0xC0 | (cp >> 6));
        buf[1] = (uint8_t)(0x80 | (cp & 0x3F));
        return 2;
    }
    if (cp < 0x10000) {
        if (buf_len < 3) return 0;
        if (cp >= 0xD800 && cp <= 0xDFFF) return -1;
        buf[0] = (uint8_t)(0xE0 | (cp >> 12));
        buf[1] = (uint8_t)(0x80 | ((cp >> 6) & 0x3F));
        buf[2] = (uint8_t)(0x80 | (cp & 0x3F));
        return 3;
    }
    if (cp <= 0x10FFFF) {
        if (buf_len < 4) return 0;
        buf[0] = (uint8_t)(0xF0 | (cp >> 18));
        buf[1] = (uint8_t)(0x80 | ((cp >> 12) & 0x3F));
        buf[2] = (uint8_t)(0x80 | ((cp >> 6) & 0x3F));
        buf[3] = (uint8_t)(0x80 | (cp & 0x3F));
        return 4;
    }
    return -1;
}

int utf8_encode_string(const uint32_t *codepoints, int cp_count, uint8_t *buf, int buf_len) {
    int pos;
    int i;
    int written;
    pos = 0;
    for (i = 0; i < cp_count; i++) {
        written = utf8_encode(codepoints[i], buf + pos, buf_len - pos);
        if (written <= 0) return -1;
        pos = pos + written;
    }
    return pos;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C612: UTF-8 encoder should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C612: Output should not be empty");
    assert!(code.contains("fn utf8_encode"), "C612: Should contain utf8_encode");
}

#[test]
fn c613_string_interning_hash_table() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    char data[64];
    int len;
    uint32_t hash;
    int occupied;
} intern_entry_t;

typedef struct {
    intern_entry_t table[256];
    int count;
} intern_pool_t;

uint32_t intern_hash(const char *s, int len) {
    uint32_t h;
    int i;
    h = 5381;
    for (i = 0; i < len; i++) {
        h = ((h << 5) + h) + (uint32_t)s[i];
    }
    return h;
}

void intern_init(intern_pool_t *pool) {
    int i;
    pool->count = 0;
    for (i = 0; i < 256; i++) {
        pool->table[i].occupied = 0;
    }
}

int intern_str_eq(const char *a, int a_len, const char *b, int b_len) {
    int i;
    if (a_len != b_len) return 0;
    for (i = 0; i < a_len; i++) {
        if (a[i] != b[i]) return 0;
    }
    return 1;
}

int intern_lookup(const intern_pool_t *pool, const char *s, int len) {
    uint32_t h;
    int idx;
    int i;
    h = intern_hash(s, len);
    idx = (int)(h % 256);
    for (i = 0; i < 256; i++) {
        int probe;
        probe = (idx + i) % 256;
        if (pool->table[probe].occupied == 0) return -1;
        if (pool->table[probe].hash == h && intern_str_eq(pool->table[probe].data, pool->table[probe].len, s, len)) {
            return probe;
        }
    }
    return -1;
}

int intern_insert(intern_pool_t *pool, const char *s, int len) {
    uint32_t h;
    int idx;
    int i;
    int probe;
    if (len >= 64 || pool->count >= 200) return -1;
    h = intern_hash(s, len);
    idx = (int)(h % 256);
    for (i = 0; i < 256; i++) {
        probe = (idx + i) % 256;
        if (pool->table[probe].occupied == 0) {
            int j;
            for (j = 0; j < len; j++) {
                pool->table[probe].data[j] = s[j];
            }
            pool->table[probe].len = len;
            pool->table[probe].hash = h;
            pool->table[probe].occupied = 1;
            pool->count++;
            return probe;
        }
        if (pool->table[probe].hash == h && intern_str_eq(pool->table[probe].data, pool->table[probe].len, s, len)) {
            return probe;
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C613: String interning should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C613: Output should not be empty");
    assert!(code.contains("fn intern_init"), "C613: Should contain intern_init");
    assert!(code.contains("fn intern_lookup"), "C613: Should contain intern_lookup");
    assert!(code.contains("fn intern_insert"), "C613: Should contain intern_insert");
}

#[test]
fn c614_wildcard_pattern_matching() {
    let c_code = r#"
int wildcard_match(const char *pattern, int pat_len, const char *text, int text_len) {
    int dp[128][128];
    int i;
    int j;
    if (pat_len > 127 || text_len > 127) return 0;
    dp[0][0] = 1;
    for (j = 1; j <= text_len; j++) {
        dp[0][j] = 0;
    }
    for (i = 1; i <= pat_len; i++) {
        if (pattern[i - 1] == '*') {
            dp[i][0] = dp[i - 1][0];
        } else {
            dp[i][0] = 0;
        }
    }
    for (i = 1; i <= pat_len; i++) {
        for (j = 1; j <= text_len; j++) {
            if (pattern[i - 1] == '*') {
                dp[i][j] = dp[i - 1][j] || dp[i][j - 1];
            } else if (pattern[i - 1] == '?' || pattern[i - 1] == text[j - 1]) {
                dp[i][j] = dp[i - 1][j - 1];
            } else {
                dp[i][j] = 0;
            }
        }
    }
    return dp[pat_len][text_len];
}

int wildcard_match_recursive(const char *pat, int pi, int plen, const char *txt, int ti, int tlen) {
    if (pi == plen && ti == tlen) return 1;
    if (pi == plen) return 0;
    if (pat[pi] == '*') {
        if (ti < tlen && wildcard_match_recursive(pat, pi, plen, txt, ti + 1, tlen)) return 1;
        return wildcard_match_recursive(pat, pi + 1, plen, txt, ti, tlen);
    }
    if (ti == tlen) return 0;
    if (pat[pi] == '?' || pat[pi] == txt[ti]) {
        return wildcard_match_recursive(pat, pi + 1, plen, txt, ti + 1, tlen);
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C614: Wildcard matching should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C614: Output should not be empty");
    assert!(code.contains("fn wildcard_match"), "C614: Should contain wildcard_match");
}

#[test]
fn c615_soundex_phonetic_encoding() {
    let c_code = r#"
int soundex_code(char c) {
    if (c >= 'a' && c <= 'z') {
        c = c - 32;
    }
    if (c == 'B' || c == 'F' || c == 'P' || c == 'V') return 1;
    if (c == 'C' || c == 'G' || c == 'J' || c == 'K' || c == 'Q' || c == 'S' || c == 'X' || c == 'Z') return 2;
    if (c == 'D' || c == 'T') return 3;
    if (c == 'L') return 4;
    if (c == 'M' || c == 'N') return 5;
    if (c == 'R') return 6;
    return 0;
}

void soundex_encode(const char *name, int name_len, char *out) {
    int i;
    int idx;
    int code;
    int last_code;
    char first;
    if (name_len == 0) {
        out[0] = '0';
        out[1] = '0';
        out[2] = '0';
        out[3] = '0';
        out[4] = 0;
        return;
    }
    first = name[0];
    if (first >= 'a' && first <= 'z') {
        first = first - 32;
    }
    out[0] = first;
    idx = 1;
    last_code = soundex_code(name[0]);
    for (i = 1; i < name_len && idx < 4; i++) {
        code = soundex_code(name[i]);
        if (code > 0 && code != last_code) {
            out[idx] = '0' + code;
            idx++;
        }
        if (code > 0) {
            last_code = code;
        }
    }
    while (idx < 4) {
        out[idx] = '0';
        idx++;
    }
    out[4] = 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C615: Soundex should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C615: Output should not be empty");
    assert!(code.contains("fn soundex_code"), "C615: Should contain soundex_code");
    assert!(code.contains("fn soundex_encode"), "C615: Should contain soundex_encode");
}

// ============================================================================
// C616-C620: NLP Primitives
// ============================================================================

#[test]
fn c616_metaphone_encoding() {
    let c_code = r#"
int meta_is_vowel(char c) {
    if (c == 'A' || c == 'E' || c == 'I' || c == 'O' || c == 'U') return 1;
    return 0;
}

char meta_to_upper(char c) {
    if (c >= 'a' && c <= 'z') return c - 32;
    return c;
}

void metaphone_encode(const char *word, int word_len, char *out, int max_out) {
    char upper[64];
    int i;
    int oi;
    char c;
    char prev;
    char next;
    if (word_len > 63) word_len = 63;
    for (i = 0; i < word_len; i++) {
        upper[i] = meta_to_upper(word[i]);
    }
    upper[word_len] = 0;
    oi = 0;
    i = 0;
    if (word_len >= 2) {
        if ((upper[0] == 'A' && upper[1] == 'E') ||
            (upper[0] == 'G' && upper[1] == 'N') ||
            (upper[0] == 'K' && upper[1] == 'N') ||
            (upper[0] == 'P' && upper[1] == 'N') ||
            (upper[0] == 'W' && upper[1] == 'R')) {
            i = 1;
        }
    }
    prev = 0;
    while (i < word_len && oi < max_out - 1) {
        c = upper[i];
        if (i + 1 < word_len) {
            next = upper[i + 1];
        } else {
            next = 0;
        }
        if (c == 'B') {
            if (prev != 'M') {
                out[oi] = 'B';
                oi++;
            }
        } else if (c == 'C') {
            if (next == 'I' || next == 'E' || next == 'Y') {
                out[oi] = 'S';
                oi++;
            } else {
                out[oi] = 'K';
                oi++;
            }
        } else if (c == 'D') {
            if (next == 'G') {
                out[oi] = 'J';
                oi++;
                i++;
            } else {
                out[oi] = 'T';
                oi++;
            }
        } else if (c == 'F') {
            out[oi] = 'F';
            oi++;
        } else if (c == 'G') {
            if (next != 'H' && !meta_is_vowel(next)) {
                out[oi] = 'K';
                oi++;
            }
        } else if (c == 'H') {
            if (meta_is_vowel(next) && !meta_is_vowel(prev)) {
                out[oi] = 'H';
                oi++;
            }
        } else if (c == 'J') {
            out[oi] = 'J';
            oi++;
        } else if (c == 'K') {
            if (prev != 'C') {
                out[oi] = 'K';
                oi++;
            }
        } else if (c == 'L') {
            out[oi] = 'L';
            oi++;
        } else if (c == 'M') {
            out[oi] = 'M';
            oi++;
        } else if (c == 'N') {
            out[oi] = 'N';
            oi++;
        } else if (c == 'P') {
            if (next == 'H') {
                out[oi] = 'F';
                oi++;
                i++;
            } else {
                out[oi] = 'P';
                oi++;
            }
        } else if (c == 'R') {
            out[oi] = 'R';
            oi++;
        } else if (c == 'S') {
            if (next == 'H') {
                out[oi] = 'X';
                oi++;
                i++;
            } else {
                out[oi] = 'S';
                oi++;
            }
        } else if (c == 'T') {
            if (next == 'H') {
                out[oi] = '0';
                oi++;
                i++;
            } else {
                out[oi] = 'T';
                oi++;
            }
        } else if (c == 'V') {
            out[oi] = 'F';
            oi++;
        } else if (c == 'W' || c == 'Y') {
            if (meta_is_vowel(next)) {
                out[oi] = c;
                oi++;
            }
        } else if (c == 'X') {
            out[oi] = 'K';
            oi++;
            if (oi < max_out - 1) {
                out[oi] = 'S';
                oi++;
            }
        } else if (c == 'Z') {
            out[oi] = 'S';
            oi++;
        }
        prev = c;
        i++;
    }
    out[oi] = 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C616: Metaphone should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C616: Output should not be empty");
    assert!(code.contains("fn metaphone_encode"), "C616: Should contain metaphone_encode");
}

#[test]
fn c617_porter_stemmer_step1() {
    let c_code = r#"
int stem_is_consonant(const char *word, int i) {
    char c;
    c = word[i];
    if (c == 'a' || c == 'e' || c == 'i' || c == 'o' || c == 'u') return 0;
    if (c == 'y') {
        if (i == 0) return 1;
        return !stem_is_consonant(word, i - 1);
    }
    return 1;
}

int stem_measure(const char *word, int len) {
    int i;
    int m;
    int in_vowel;
    m = 0;
    i = 0;
    while (i < len && stem_is_consonant(word, i)) {
        i++;
    }
    if (i >= len) return 0;
    in_vowel = 1;
    while (i < len) {
        if (stem_is_consonant(word, i)) {
            if (in_vowel) {
                m++;
                in_vowel = 0;
            }
        } else {
            in_vowel = 1;
        }
        i++;
    }
    return m;
}

int stem_ends_with(const char *word, int len, const char *suffix, int suf_len) {
    int i;
    if (suf_len > len) return 0;
    for (i = 0; i < suf_len; i++) {
        if (word[len - suf_len + i] != suffix[i]) return 0;
    }
    return 1;
}

int stem_step1a(char *word, int len) {
    if (stem_ends_with(word, len, "sses", 4)) {
        len = len - 2;
    } else if (stem_ends_with(word, len, "ies", 3)) {
        len = len - 2;
    } else if (!stem_ends_with(word, len, "ss", 2) && stem_ends_with(word, len, "s", 1)) {
        len = len - 1;
    }
    word[len] = 0;
    return len;
}

int stem_has_vowel(const char *word, int len) {
    int i;
    for (i = 0; i < len; i++) {
        if (!stem_is_consonant(word, i)) return 1;
    }
    return 0;
}

int stem_step1b(char *word, int len) {
    int stem_len;
    if (stem_ends_with(word, len, "eed", 3)) {
        stem_len = len - 3;
        if (stem_measure(word, stem_len) > 0) {
            len = len - 1;
        }
    } else if (stem_ends_with(word, len, "ed", 2)) {
        stem_len = len - 2;
        if (stem_has_vowel(word, stem_len)) {
            len = stem_len;
        }
    } else if (stem_ends_with(word, len, "ing", 3)) {
        stem_len = len - 3;
        if (stem_has_vowel(word, stem_len)) {
            len = stem_len;
        }
    }
    word[len] = 0;
    return len;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C617: Porter stemmer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C617: Output should not be empty");
    assert!(code.contains("fn stem_measure"), "C617: Should contain stem_measure");
    assert!(code.contains("fn stem_step1a"), "C617: Should contain stem_step1a");
}

#[test]
fn c618_word_frequency_counter() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    char word[32];
    int word_len;
    int count;
    int occupied;
} wf_entry_t;

typedef struct {
    wf_entry_t entries[256];
    int unique_count;
    int total_words;
} wf_counter_t;

uint32_t wf_hash(const char *s, int len) {
    uint32_t h;
    int i;
    h = 0;
    for (i = 0; i < len; i++) {
        h = h * 31 + (uint32_t)s[i];
    }
    return h;
}

void wf_init(wf_counter_t *wf) {
    int i;
    wf->unique_count = 0;
    wf->total_words = 0;
    for (i = 0; i < 256; i++) {
        wf->entries[i].occupied = 0;
    }
}

int wf_str_eq(const char *a, int a_len, const char *b, int b_len) {
    int i;
    if (a_len != b_len) return 0;
    for (i = 0; i < a_len; i++) {
        if (a[i] != b[i]) return 0;
    }
    return 1;
}

void wf_add_word(wf_counter_t *wf, const char *word, int word_len) {
    uint32_t h;
    int idx;
    int i;
    int probe;
    if (word_len >= 32) return;
    h = wf_hash(word, word_len);
    idx = (int)(h % 256);
    wf->total_words++;
    for (i = 0; i < 256; i++) {
        probe = (idx + i) % 256;
        if (wf->entries[probe].occupied == 0) {
            int j;
            for (j = 0; j < word_len; j++) {
                wf->entries[probe].word[j] = word[j];
            }
            wf->entries[probe].word_len = word_len;
            wf->entries[probe].count = 1;
            wf->entries[probe].occupied = 1;
            wf->unique_count++;
            return;
        }
        if (wf_str_eq(wf->entries[probe].word, wf->entries[probe].word_len, word, word_len)) {
            wf->entries[probe].count++;
            return;
        }
    }
}

void wf_count_text(wf_counter_t *wf, const char *text, int text_len) {
    int i;
    int word_start;
    int in_word;
    in_word = 0;
    word_start = 0;
    for (i = 0; i < text_len; i++) {
        if (text[i] == ' ' || text[i] == '\n' || text[i] == '\t') {
            if (in_word) {
                wf_add_word(wf, text + word_start, i - word_start);
                in_word = 0;
            }
        } else {
            if (!in_word) {
                word_start = i;
                in_word = 1;
            }
        }
    }
    if (in_word) {
        wf_add_word(wf, text + word_start, text_len - word_start);
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C618: Word frequency should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C618: Output should not be empty");
    assert!(code.contains("fn wf_init"), "C618: Should contain wf_init");
    assert!(code.contains("fn wf_add_word"), "C618: Should contain wf_add_word");
    assert!(code.contains("fn wf_count_text"), "C618: Should contain wf_count_text");
}

#[test]
fn c619_ngram_generator() {
    let c_code = r#"
typedef struct {
    char grams[256][8];
    int gram_lens[256];
    int count;
} ngram_result_t;

void ngram_init(ngram_result_t *res) {
    res->count = 0;
}

void ngram_char_ngrams(const char *text, int text_len, int n, ngram_result_t *res) {
    int i;
    int j;
    res->count = 0;
    if (n <= 0 || n > 7 || text_len < n) return;
    for (i = 0; i <= text_len - n; i++) {
        if (res->count >= 256) break;
        for (j = 0; j < n; j++) {
            res->grams[res->count][j] = text[i + j];
        }
        res->grams[res->count][n] = 0;
        res->gram_lens[res->count] = n;
        res->count++;
    }
}

int ngram_is_separator(char c) {
    if (c == ' ' || c == '\t' || c == '\n' || c == '\r') return 1;
    return 0;
}

void ngram_word_ngrams(const char *text, int text_len, int n, ngram_result_t *res) {
    int word_starts[128];
    int word_lens[128];
    int word_count;
    int i;
    int j;
    int k;
    int in_word;
    int ws;
    res->count = 0;
    word_count = 0;
    in_word = 0;
    ws = 0;
    for (i = 0; i < text_len; i++) {
        if (ngram_is_separator(text[i])) {
            if (in_word && word_count < 128) {
                word_starts[word_count] = ws;
                word_lens[word_count] = i - ws;
                word_count++;
                in_word = 0;
            }
        } else {
            if (!in_word) {
                ws = i;
                in_word = 1;
            }
        }
    }
    if (in_word && word_count < 128) {
        word_starts[word_count] = ws;
        word_lens[word_count] = text_len - ws;
        word_count++;
    }
    if (word_count < n) return;
    for (i = 0; i <= word_count - n; i++) {
        int pos;
        if (res->count >= 256) break;
        pos = 0;
        for (j = 0; j < n; j++) {
            if (j > 0 && pos < 7) {
                res->grams[res->count][pos] = ' ';
                pos++;
            }
            for (k = 0; k < word_lens[i + j] && pos < 7; k++) {
                res->grams[res->count][pos] = text[word_starts[i + j] + k];
                pos++;
            }
        }
        res->grams[res->count][pos] = 0;
        res->gram_lens[res->count] = pos;
        res->count++;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C619: N-gram generator should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C619: Output should not be empty");
    assert!(code.contains("fn ngram_char_ngrams"), "C619: Should contain ngram_char_ngrams");
    assert!(code.contains("fn ngram_word_ngrams"), "C619: Should contain ngram_word_ngrams");
}

#[test]
fn c620_sentence_tokenizer() {
    let c_code = r#"
typedef struct {
    int starts[128];
    int lens[128];
    int count;
} sentence_list_t;

int sent_is_alpha(char c) {
    if ((c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z')) return 1;
    return 0;
}

int sent_is_upper(char c) {
    if (c >= 'A' && c <= 'Z') return 1;
    return 0;
}

int sent_is_terminator(char c) {
    if (c == '.' || c == '!' || c == '?') return 1;
    return 0;
}

void sent_tokenize(const char *text, int text_len, sentence_list_t *result) {
    int i;
    int sent_start;
    int in_sentence;
    result->count = 0;
    sent_start = 0;
    in_sentence = 0;
    for (i = 0; i < text_len; i++) {
        if (!in_sentence && sent_is_alpha(text[i])) {
            sent_start = i;
            in_sentence = 1;
        }
        if (in_sentence && sent_is_terminator(text[i])) {
            int next_idx;
            next_idx = i + 1;
            while (next_idx < text_len && text[next_idx] == ' ') {
                next_idx++;
            }
            if (next_idx >= text_len || sent_is_upper(text[next_idx])) {
                if (result->count < 128) {
                    result->starts[result->count] = sent_start;
                    result->lens[result->count] = i - sent_start + 1;
                    result->count++;
                }
                in_sentence = 0;
            }
        }
    }
    if (in_sentence && result->count < 128) {
        result->starts[result->count] = sent_start;
        result->lens[result->count] = text_len - sent_start;
        result->count++;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C620: Sentence tokenizer should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C620: Output should not be empty");
    assert!(code.contains("fn sent_tokenize"), "C620: Should contain sent_tokenize");
    assert!(code.contains("fn sent_is_terminator"), "C620: Should contain sent_is_terminator");
}

// ============================================================================
// C621-C625: Compression and Ciphers
// ============================================================================

#[test]
fn c621_run_length_string_compression() {
    let c_code = r#"
int rle_encode(const char *input, int in_len, char *output, int out_max) {
    int i;
    int oi;
    int run;
    char c;
    oi = 0;
    i = 0;
    while (i < in_len) {
        c = input[i];
        run = 1;
        while (i + run < in_len && input[i + run] == c && run < 255) {
            run++;
        }
        if (run >= 3) {
            if (oi + 3 > out_max) return -1;
            output[oi] = '#';
            oi++;
            output[oi] = c;
            oi++;
            output[oi] = (char)run;
            oi++;
        } else {
            int j;
            for (j = 0; j < run; j++) {
                if (oi >= out_max) return -1;
                if (c == '#') {
                    if (oi + 2 > out_max) return -1;
                    output[oi] = '#';
                    oi++;
                    output[oi] = '#';
                    oi++;
                } else {
                    output[oi] = c;
                    oi++;
                }
            }
        }
        i = i + run;
    }
    return oi;
}

int rle_decode(const char *input, int in_len, char *output, int out_max) {
    int i;
    int oi;
    oi = 0;
    i = 0;
    while (i < in_len) {
        if (input[i] == '#') {
            i++;
            if (i >= in_len) return -1;
            if (input[i] == '#') {
                if (oi >= out_max) return -1;
                output[oi] = '#';
                oi++;
                i++;
            } else {
                char c;
                int run;
                int j;
                c = input[i];
                i++;
                if (i >= in_len) return -1;
                run = (int)(unsigned char)input[i];
                i++;
                for (j = 0; j < run; j++) {
                    if (oi >= out_max) return -1;
                    output[oi] = c;
                    oi++;
                }
            }
        } else {
            if (oi >= out_max) return -1;
            output[oi] = input[i];
            oi++;
            i++;
        }
    }
    return oi;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C621: RLE compression should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C621: Output should not be empty");
    assert!(code.contains("fn rle_encode"), "C621: Should contain rle_encode");
    assert!(code.contains("fn rle_decode"), "C621: Should contain rle_decode");
}

#[test]
fn c622_caesar_cipher() {
    let c_code = r#"
void caesar_encrypt(char *text, int len, int shift) {
    int i;
    int s;
    s = shift % 26;
    if (s < 0) s = s + 26;
    for (i = 0; i < len; i++) {
        if (text[i] >= 'A' && text[i] <= 'Z') {
            text[i] = 'A' + (text[i] - 'A' + s) % 26;
        } else if (text[i] >= 'a' && text[i] <= 'z') {
            text[i] = 'a' + (text[i] - 'a' + s) % 26;
        }
    }
}

void caesar_decrypt(char *text, int len, int shift) {
    caesar_encrypt(text, len, 26 - (shift % 26));
}

int caesar_crack_frequency(const char *cipher, int len, char *plain, int plain_max) {
    int freq[26];
    int i;
    int best_shift;
    int max_freq;
    int shift;
    int idx;
    for (i = 0; i < 26; i++) {
        freq[i] = 0;
    }
    for (i = 0; i < len; i++) {
        if (cipher[i] >= 'a' && cipher[i] <= 'z') {
            freq[cipher[i] - 'a']++;
        } else if (cipher[i] >= 'A' && cipher[i] <= 'Z') {
            freq[cipher[i] - 'A']++;
        }
    }
    max_freq = 0;
    best_shift = 0;
    for (i = 0; i < 26; i++) {
        if (freq[i] > max_freq) {
            max_freq = freq[i];
            best_shift = i;
        }
    }
    shift = (best_shift - 4 + 26) % 26;
    if (len > plain_max) len = plain_max;
    for (i = 0; i < len; i++) {
        plain[i] = cipher[i];
    }
    caesar_encrypt(plain, len, 26 - shift);
    return shift;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C622: Caesar cipher should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C622: Output should not be empty");
    assert!(code.contains("fn caesar_encrypt"), "C622: Should contain caesar_encrypt");
    assert!(code.contains("fn caesar_decrypt"), "C622: Should contain caesar_decrypt");
}

#[test]
fn c623_vigenere_cipher() {
    let c_code = r#"
void vigenere_encrypt(char *text, int text_len, const char *key, int key_len) {
    int i;
    int ki;
    int shift;
    ki = 0;
    for (i = 0; i < text_len; i++) {
        if (key_len == 0) return;
        if (text[i] >= 'A' && text[i] <= 'Z') {
            shift = key[ki % key_len];
            if (shift >= 'a' && shift <= 'z') shift = shift - 'a';
            else if (shift >= 'A' && shift <= 'Z') shift = shift - 'A';
            else shift = 0;
            text[i] = 'A' + (text[i] - 'A' + shift) % 26;
            ki++;
        } else if (text[i] >= 'a' && text[i] <= 'z') {
            shift = key[ki % key_len];
            if (shift >= 'a' && shift <= 'z') shift = shift - 'a';
            else if (shift >= 'A' && shift <= 'Z') shift = shift - 'A';
            else shift = 0;
            text[i] = 'a' + (text[i] - 'a' + shift) % 26;
            ki++;
        }
    }
}

void vigenere_decrypt(char *text, int text_len, const char *key, int key_len) {
    int i;
    int ki;
    int shift;
    ki = 0;
    for (i = 0; i < text_len; i++) {
        if (key_len == 0) return;
        if (text[i] >= 'A' && text[i] <= 'Z') {
            shift = key[ki % key_len];
            if (shift >= 'a' && shift <= 'z') shift = shift - 'a';
            else if (shift >= 'A' && shift <= 'Z') shift = shift - 'A';
            else shift = 0;
            text[i] = 'A' + (text[i] - 'A' - shift + 26) % 26;
            ki++;
        } else if (text[i] >= 'a' && text[i] <= 'z') {
            shift = key[ki % key_len];
            if (shift >= 'a' && shift <= 'z') shift = shift - 'a';
            else if (shift >= 'A' && shift <= 'Z') shift = shift - 'A';
            else shift = 0;
            text[i] = 'a' + (text[i] - 'a' - shift + 26) % 26;
            ki++;
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C623: Vigenere cipher should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C623: Output should not be empty");
    assert!(code.contains("fn vigenere_encrypt"), "C623: Should contain vigenere_encrypt");
    assert!(code.contains("fn vigenere_decrypt"), "C623: Should contain vigenere_decrypt");
}

#[test]
fn c624_rot13_encoding() {
    let c_code = r#"
void rot13_encode(char *text, int len) {
    int i;
    for (i = 0; i < len; i++) {
        if (text[i] >= 'A' && text[i] <= 'Z') {
            text[i] = 'A' + (text[i] - 'A' + 13) % 26;
        } else if (text[i] >= 'a' && text[i] <= 'z') {
            text[i] = 'a' + (text[i] - 'a' + 13) % 26;
        }
    }
}

void rot13_encode_buf(const char *input, int len, char *output) {
    int i;
    for (i = 0; i < len; i++) {
        if (input[i] >= 'A' && input[i] <= 'Z') {
            output[i] = 'A' + (input[i] - 'A' + 13) % 26;
        } else if (input[i] >= 'a' && input[i] <= 'z') {
            output[i] = 'a' + (input[i] - 'a' + 13) % 26;
        } else {
            output[i] = input[i];
        }
    }
    output[len] = 0;
}

int rot13_selftest(void) {
    char buf[16];
    char orig[16];
    int i;
    int len;
    len = 5;
    orig[0] = 'H'; orig[1] = 'e'; orig[2] = 'l'; orig[3] = 'l'; orig[4] = 'o';
    for (i = 0; i < len; i++) {
        buf[i] = orig[i];
    }
    rot13_encode(buf, len);
    rot13_encode(buf, len);
    for (i = 0; i < len; i++) {
        if (buf[i] != orig[i]) return -1;
    }
    return 0;
}

int rot47_encode(char *text, int len) {
    int i;
    int count;
    count = 0;
    for (i = 0; i < len; i++) {
        if (text[i] >= 33 && text[i] <= 126) {
            text[i] = 33 + (text[i] - 33 + 47) % 94;
            count++;
        }
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C624: ROT13 should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C624: Output should not be empty");
    assert!(code.contains("fn rot13_encode"), "C624: Should contain rot13_encode");
    assert!(code.contains("fn rot13_selftest"), "C624: Should contain rot13_selftest");
}

#[test]
fn c625_morse_code_encoder() {
    let c_code = r#"
typedef struct {
    char letter;
    char code[8];
} morse_entry_t;

int morse_table_size(void) {
    return 36;
}

void morse_get_entry(int idx, char *letter, char *code) {
    char letters[36];
    int i;
    letters[0] = 'A'; letters[1] = 'B'; letters[2] = 'C'; letters[3] = 'D';
    letters[4] = 'E'; letters[5] = 'F'; letters[6] = 'G'; letters[7] = 'H';
    letters[8] = 'I'; letters[9] = 'J'; letters[10] = 'K'; letters[11] = 'L';
    letters[12] = 'M'; letters[13] = 'N'; letters[14] = 'O'; letters[15] = 'P';
    letters[16] = 'Q'; letters[17] = 'R'; letters[18] = 'S'; letters[19] = 'T';
    letters[20] = 'U'; letters[21] = 'V'; letters[22] = 'W'; letters[23] = 'X';
    letters[24] = 'Y'; letters[25] = 'Z'; letters[26] = '0'; letters[27] = '1';
    letters[28] = '2'; letters[29] = '3'; letters[30] = '4'; letters[31] = '5';
    letters[32] = '6'; letters[33] = '7'; letters[34] = '8'; letters[35] = '9';
    *letter = letters[idx];
    code[0] = 0;
}

char morse_to_upper(char c) {
    if (c >= 'a' && c <= 'z') return c - 32;
    return c;
}

int morse_lookup(char c, char *code) {
    char ch;
    char entry_code[8];
    int i;
    int tsize;
    ch = morse_to_upper(c);
    tsize = morse_table_size();
    for (i = 0; i < tsize; i++) {
        char letter;
        morse_get_entry(i, &letter, entry_code);
        if (letter == ch) {
            int j;
            for (j = 0; j < 8; j++) {
                code[j] = entry_code[j];
                if (entry_code[j] == 0) break;
            }
            return 1;
        }
    }
    return 0;
}

int morse_encode(const char *text, int text_len, char *output, int out_max) {
    int i;
    int oi;
    char code[8];
    oi = 0;
    for (i = 0; i < text_len; i++) {
        if (text[i] == ' ') {
            if (oi + 3 > out_max) return -1;
            output[oi] = ' ';
            oi++;
            output[oi] = '/';
            oi++;
            output[oi] = ' ';
            oi++;
        } else if (morse_lookup(text[i], code)) {
            int j;
            if (i > 0 && text[i - 1] != ' ' && oi > 0) {
                if (oi >= out_max) return -1;
                output[oi] = ' ';
                oi++;
            }
            for (j = 0; code[j] != 0 && j < 8; j++) {
                if (oi >= out_max) return -1;
                output[oi] = code[j];
                oi++;
            }
        }
    }
    if (oi < out_max) {
        output[oi] = 0;
    }
    return oi;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C625: Morse code should transpile: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C625: Output should not be empty");
    assert!(code.contains("fn morse_encode"), "C625: Should contain morse_encode");
    assert!(code.contains("fn morse_lookup"), "C625: Should contain morse_lookup");
}
