//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C951-C975: String Algorithms -- the kind of C code found in text search
//! engines, bioinformatics tools, compilers, and data compression libraries.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world string algorithm patterns commonly
//! found in grep, awk, compiler lexers, DNA sequence analysis, and
//! information retrieval systems -- all expressed as valid C99.
//!
//! Organization:
//! - C951-C955: Pattern matching and suffix structures
//! - C956-C960: Substring problems and trie-based search
//! - C961-C965: Sequence comparison, hashing, encoding
//! - C966-C970: Transforms, automata, pattern matching
//! - C971-C975: Classic string DP problems

// ============================================================================
// C951-C955: Pattern Matching and Suffix Structures
// ============================================================================

#[test]
fn c951_kmp_search() {
    let c_code = r#"
typedef unsigned long size_t;

void str_kmp_build_failure(const char *pattern, int pat_len, int *failure) {
    int i, j;
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

int str_kmp_search(const char *text, int text_len, const char *pattern, int pat_len) {
    int failure[256];
    int i, j;
    if (pat_len == 0) return 0;
    if (pat_len > 256) return -1;
    str_kmp_build_failure(pattern, pat_len, failure);
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

int str_kmp_count_occurrences(const char *text, int text_len, const char *pattern, int pat_len) {
    int failure[256];
    int i, j, count;
    if (pat_len == 0 || pat_len > 256) return 0;
    str_kmp_build_failure(pattern, pat_len, failure);
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
    assert!(
        result.is_ok(),
        "C951: KMP pattern matching should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C951: Output should not be empty");
    assert!(
        code.contains("fn str_kmp_search"),
        "C951: Should contain str_kmp_search function"
    );
}

#[test]
fn c952_rabin_karp() {
    let c_code = r#"
typedef unsigned long size_t;
typedef unsigned long uint64_t;

static const uint64_t RK_BASE = 256;
static const uint64_t RK_MOD = 1000000007;

uint64_t str_rk_power(uint64_t base, int exp, uint64_t mod_val) {
    uint64_t result = 1;
    base = base % mod_val;
    while (exp > 0) {
        if (exp % 2 == 1) {
            result = (result * base) % mod_val;
        }
        exp = exp / 2;
        base = (base * base) % mod_val;
    }
    return result;
}

uint64_t str_rk_hash(const char *s, int len) {
    uint64_t h = 0;
    int i;
    for (i = 0; i < len; i++) {
        h = (h * RK_BASE + (uint64_t)s[i]) % RK_MOD;
    }
    return h;
}

int str_rabin_karp_search(const char *text, int text_len, const char *pattern, int pat_len) {
    uint64_t pat_hash, text_hash, high_pow;
    int i, j, match;
    if (pat_len > text_len) return -1;
    if (pat_len == 0) return 0;
    high_pow = str_rk_power(RK_BASE, pat_len - 1, RK_MOD);
    pat_hash = str_rk_hash(pattern, pat_len);
    text_hash = str_rk_hash(text, pat_len);
    for (i = 0; i <= text_len - pat_len; i++) {
        if (text_hash == pat_hash) {
            match = 1;
            for (j = 0; j < pat_len; j++) {
                if (text[i + j] != pattern[j]) {
                    match = 0;
                    break;
                }
            }
            if (match) return i;
        }
        if (i < text_len - pat_len) {
            text_hash = (text_hash + RK_MOD - (uint64_t)text[i] * high_pow % RK_MOD) % RK_MOD;
            text_hash = (text_hash * RK_BASE + (uint64_t)text[i + pat_len]) % RK_MOD;
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C952: Rabin-Karp string search should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C952: Output should not be empty");
    assert!(
        code.contains("fn str_rabin_karp_search"),
        "C952: Should contain str_rabin_karp_search function"
    );
}

#[test]
fn c953_boyer_moore_bad_char() {
    let c_code = r#"
void str_bm_build_bad_char(const char *pattern, int pat_len, int *bad_char) {
    int i;
    for (i = 0; i < 256; i++) {
        bad_char[i] = -1;
    }
    for (i = 0; i < pat_len; i++) {
        bad_char[(unsigned char)pattern[i]] = i;
    }
}

int str_boyer_moore_search(const char *text, int text_len, const char *pattern, int pat_len) {
    int bad_char[256];
    int shift, i, j;
    if (pat_len == 0) return 0;
    if (pat_len > text_len) return -1;
    str_bm_build_bad_char(pattern, pat_len, bad_char);
    shift = 0;
    while (shift <= text_len - pat_len) {
        j = pat_len - 1;
        while (j >= 0 && pattern[j] == text[shift + j]) {
            j--;
        }
        if (j < 0) {
            return shift;
        } else {
            int bc = bad_char[(unsigned char)text[shift + j]];
            int skip = j - bc;
            if (skip < 1) skip = 1;
            shift += skip;
        }
    }
    return -1;
}

int str_bm_count(const char *text, int text_len, const char *pattern, int pat_len) {
    int bad_char[256];
    int shift, j, count;
    if (pat_len == 0 || pat_len > text_len) return 0;
    str_bm_build_bad_char(pattern, pat_len, bad_char);
    shift = 0;
    count = 0;
    while (shift <= text_len - pat_len) {
        j = pat_len - 1;
        while (j >= 0 && pattern[j] == text[shift + j]) {
            j--;
        }
        if (j < 0) {
            count++;
            shift += pat_len;
        } else {
            int bc = bad_char[(unsigned char)text[shift + j]];
            int skip = j - bc;
            if (skip < 1) skip = 1;
            shift += skip;
        }
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C953: Boyer-Moore bad char rule should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C953: Output should not be empty");
    assert!(
        code.contains("fn str_boyer_moore_search"),
        "C953: Should contain str_boyer_moore_search function"
    );
}

#[test]
fn c954_z_algorithm() {
    let c_code = r#"
void str_z_array(const char *s, int n, int *z) {
    int l, r, k, i;
    z[0] = n;
    l = 0;
    r = 0;
    for (i = 1; i < n; i++) {
        if (i < r) {
            z[i] = r - i;
            if (z[i] > z[i - l]) {
                z[i] = z[i - l];
            }
        } else {
            z[i] = 0;
        }
        while (i + z[i] < n && s[z[i]] == s[i + z[i]]) {
            z[i]++;
        }
        if (i + z[i] > r) {
            l = i;
            r = i + z[i];
        }
    }
}

int str_z_search(const char *text, int text_len, const char *pattern, int pat_len) {
    char concat[1024];
    int z[1024];
    int total, i;
    if (pat_len == 0) return 0;
    total = pat_len + 1 + text_len;
    if (total > 1024) return -1;
    for (i = 0; i < pat_len; i++) concat[i] = pattern[i];
    concat[pat_len] = '$';
    for (i = 0; i < text_len; i++) concat[pat_len + 1 + i] = text[i];
    str_z_array(concat, total, z);
    for (i = pat_len + 1; i < total; i++) {
        if (z[i] == pat_len) {
            return i - pat_len - 1;
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C954: Z-algorithm should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C954: Output should not be empty");
    assert!(
        code.contains("fn str_z_array"),
        "C954: Should contain str_z_array function"
    );
}

#[test]
fn c955_suffix_array() {
    let c_code = r#"
static int str_sa_str_len(const char *s) {
    int len = 0;
    while (s[len] != '\0') len++;
    return len;
}

static const char *str_sa_global_str;

int str_sa_compare(const void *a, const void *b) {
    int ia = *(const int *)a;
    int ib = *(const int *)b;
    int i = 0;
    while (str_sa_global_str[ia + i] != '\0' && str_sa_global_str[ib + i] != '\0') {
        if (str_sa_global_str[ia + i] < str_sa_global_str[ib + i]) return -1;
        if (str_sa_global_str[ia + i] > str_sa_global_str[ib + i]) return 1;
        i++;
    }
    if (str_sa_global_str[ia + i] == '\0' && str_sa_global_str[ib + i] != '\0') return -1;
    if (str_sa_global_str[ia + i] != '\0' && str_sa_global_str[ib + i] == '\0') return 1;
    return 0;
}

void str_suffix_array_build(const char *s, int *sa, int n) {
    int i, j, min_idx, temp;
    str_sa_global_str = s;
    for (i = 0; i < n; i++) sa[i] = i;
    for (i = 0; i < n - 1; i++) {
        min_idx = i;
        for (j = i + 1; j < n; j++) {
            if (str_sa_compare(&sa[j], &sa[min_idx]) < 0) {
                min_idx = j;
            }
        }
        if (min_idx != i) {
            temp = sa[i];
            sa[i] = sa[min_idx];
            sa[min_idx] = temp;
        }
    }
}

int str_suffix_array_search(const char *text, int text_len, int *sa, const char *pattern, int pat_len) {
    int lo, hi, mid, cmp, k;
    lo = 0;
    hi = text_len - 1;
    while (lo <= hi) {
        mid = (lo + hi) / 2;
        cmp = 0;
        for (k = 0; k < pat_len && sa[mid] + k < text_len; k++) {
            if (text[sa[mid] + k] < pattern[k]) { cmp = -1; break; }
            if (text[sa[mid] + k] > pattern[k]) { cmp = 1; break; }
        }
        if (cmp == 0 && k < pat_len) cmp = -1;
        if (cmp == 0) return sa[mid];
        if (cmp < 0) lo = mid + 1;
        else hi = mid - 1;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C955: Suffix array construction should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C955: Output should not be empty");
    assert!(
        code.contains("fn str_suffix_array_build"),
        "C955: Should contain str_suffix_array_build function"
    );
}

// ============================================================================
// C956-C960: Substring Problems and Trie-Based Search
// ============================================================================

#[test]
fn c956_longest_common_substring() {
    let c_code = r#"
int str_lcs_substring(const char *a, int a_len, const char *b, int b_len, char *result) {
    int dp[128][128];
    int i, j, max_len, end_idx;
    max_len = 0;
    end_idx = 0;
    for (i = 0; i <= a_len; i++) dp[i][0] = 0;
    for (j = 0; j <= b_len; j++) dp[0][j] = 0;
    for (i = 1; i <= a_len; i++) {
        for (j = 1; j <= b_len; j++) {
            if (a[i - 1] == b[j - 1]) {
                dp[i][j] = dp[i - 1][j - 1] + 1;
                if (dp[i][j] > max_len) {
                    max_len = dp[i][j];
                    end_idx = i;
                }
            } else {
                dp[i][j] = 0;
            }
        }
    }
    for (i = 0; i < max_len; i++) {
        result[i] = a[end_idx - max_len + i];
    }
    result[max_len] = '\0';
    return max_len;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C956: Longest common substring should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C956: Output should not be empty");
    assert!(
        code.contains("fn str_lcs_substring"),
        "C956: Should contain str_lcs_substring function"
    );
}

#[test]
fn c957_levenshtein_distance() {
    let c_code = r#"
static int str_lev_min3(int a, int b, int c) {
    int m = a;
    if (b < m) m = b;
    if (c < m) m = c;
    return m;
}

int str_levenshtein(const char *s, int s_len, const char *t, int t_len) {
    int dp[256][256];
    int i, j, cost;
    if (s_len > 255 || t_len > 255) return -1;
    for (i = 0; i <= s_len; i++) dp[i][0] = i;
    for (j = 0; j <= t_len; j++) dp[0][j] = j;
    for (i = 1; i <= s_len; i++) {
        for (j = 1; j <= t_len; j++) {
            cost = (s[i - 1] == t[j - 1]) ? 0 : 1;
            dp[i][j] = str_lev_min3(
                dp[i - 1][j] + 1,
                dp[i][j - 1] + 1,
                dp[i - 1][j - 1] + cost
            );
        }
    }
    return dp[s_len][t_len];
}

int str_levenshtein_threshold(const char *s, int s_len, const char *t, int t_len, int max_dist) {
    int dist = str_levenshtein(s, s_len, t, t_len);
    return (dist >= 0 && dist <= max_dist) ? 1 : 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C957: Levenshtein edit distance should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C957: Output should not be empty");
    assert!(
        code.contains("fn str_levenshtein"),
        "C957: Should contain str_levenshtein function"
    );
}

#[test]
fn c958_manacher_palindrome() {
    let c_code = r#"
int str_manacher_longest_palindrome(const char *s, int n, int *center, int *radius) {
    int p[1024];
    int c_val, r, i, mirror, max_len, max_center;
    char t[1024];
    int t_len, k;
    if (n == 0) { *center = 0; *radius = 0; return 0; }
    t_len = 0;
    t[t_len++] = '#';
    for (k = 0; k < n && t_len < 1022; k++) {
        t[t_len++] = s[k];
        t[t_len++] = '#';
    }
    for (k = 0; k < t_len; k++) p[k] = 0;
    c_val = 0;
    r = 0;
    for (i = 0; i < t_len; i++) {
        mirror = 2 * c_val - i;
        if (i < r) {
            p[i] = r - i;
            if (mirror >= 0 && p[mirror] < p[i]) {
                p[i] = p[mirror];
            }
        }
        while (i + p[i] + 1 < t_len && i - p[i] - 1 >= 0
               && t[i + p[i] + 1] == t[i - p[i] - 1]) {
            p[i]++;
        }
        if (i + p[i] > r) {
            c_val = i;
            r = i + p[i];
        }
    }
    max_len = 0;
    max_center = 0;
    for (i = 0; i < t_len; i++) {
        if (p[i] > max_len) {
            max_len = p[i];
            max_center = i;
        }
    }
    *center = (max_center - max_len) / 2;
    *radius = max_len;
    return max_len;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C958: Manacher's palindrome algorithm should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C958: Output should not be empty");
    assert!(
        code.contains("fn str_manacher_longest_palindrome"),
        "C958: Should contain str_manacher_longest_palindrome function"
    );
}

#[test]
fn c959_trie_insert_search() {
    let c_code = r#"
typedef struct str_trie_node {
    int children[26];
    int is_end;
    int count;
} str_trie_node;

typedef struct {
    str_trie_node nodes[4096];
    int node_count;
} str_trie;

void str_trie_init(str_trie *t) {
    int i, j;
    t->node_count = 1;
    for (j = 0; j < 26; j++) {
        t->nodes[0].children[j] = -1;
    }
    t->nodes[0].is_end = 0;
    t->nodes[0].count = 0;
}

int str_trie_alloc_node(str_trie *t) {
    int idx, j;
    if (t->node_count >= 4096) return -1;
    idx = t->node_count++;
    for (j = 0; j < 26; j++) {
        t->nodes[idx].children[j] = -1;
    }
    t->nodes[idx].is_end = 0;
    t->nodes[idx].count = 0;
    return idx;
}

int str_trie_insert(str_trie *t, const char *word) {
    int node = 0;
    int i = 0;
    int c, child;
    while (word[i] != '\0') {
        c = word[i] - 'a';
        if (c < 0 || c >= 26) return -1;
        if (t->nodes[node].children[c] == -1) {
            child = str_trie_alloc_node(t);
            if (child == -1) return -1;
            t->nodes[node].children[c] = child;
        }
        node = t->nodes[node].children[c];
        t->nodes[node].count++;
        i++;
    }
    t->nodes[node].is_end = 1;
    return 0;
}

int str_trie_search(const str_trie *t, const char *word) {
    int node = 0;
    int i = 0;
    int c;
    while (word[i] != '\0') {
        c = word[i] - 'a';
        if (c < 0 || c >= 26) return 0;
        if (t->nodes[node].children[c] == -1) return 0;
        node = t->nodes[node].children[c];
        i++;
    }
    return t->nodes[node].is_end;
}

int str_trie_starts_with(const str_trie *t, const char *prefix) {
    int node = 0;
    int i = 0;
    int c;
    while (prefix[i] != '\0') {
        c = prefix[i] - 'a';
        if (c < 0 || c >= 26) return 0;
        if (t->nodes[node].children[c] == -1) return 0;
        node = t->nodes[node].children[c];
        i++;
    }
    return t->nodes[node].count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C959: Trie insert/search should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C959: Output should not be empty");
    assert!(
        code.contains("fn str_trie_insert"),
        "C959: Should contain str_trie_insert function"
    );
    assert!(
        code.contains("fn str_trie_search"),
        "C959: Should contain str_trie_search function"
    );
}

#[test]
fn c960_aho_corasick() {
    let c_code = r#"
typedef struct {
    int children[26];
    int fail;
    int output;
    int depth;
} str_ac_node;

typedef struct {
    str_ac_node nodes[2048];
    int node_count;
} str_ac_automaton;

void str_ac_init(str_ac_automaton *ac) {
    int j;
    ac->node_count = 1;
    for (j = 0; j < 26; j++) ac->nodes[0].children[j] = -1;
    ac->nodes[0].fail = 0;
    ac->nodes[0].output = -1;
    ac->nodes[0].depth = 0;
}

int str_ac_alloc(str_ac_automaton *ac) {
    int idx, j;
    if (ac->node_count >= 2048) return -1;
    idx = ac->node_count++;
    for (j = 0; j < 26; j++) ac->nodes[idx].children[j] = -1;
    ac->nodes[idx].fail = 0;
    ac->nodes[idx].output = -1;
    ac->nodes[idx].depth = 0;
    return idx;
}

int str_ac_add_pattern(str_ac_automaton *ac, const char *pattern, int pattern_id) {
    int node = 0;
    int i = 0;
    int c, child;
    while (pattern[i] != '\0') {
        c = pattern[i] - 'a';
        if (c < 0 || c >= 26) return -1;
        if (ac->nodes[node].children[c] == -1) {
            child = str_ac_alloc(ac);
            if (child == -1) return -1;
            ac->nodes[node].children[c] = child;
            ac->nodes[child].depth = ac->nodes[node].depth + 1;
        }
        node = ac->nodes[node].children[c];
        i++;
    }
    ac->nodes[node].output = pattern_id;
    return 0;
}

void str_ac_build_fail(str_ac_automaton *ac) {
    int queue[2048];
    int head, tail, u, c, v, f;
    head = 0;
    tail = 0;
    for (c = 0; c < 26; c++) {
        if (ac->nodes[0].children[c] != -1) {
            ac->nodes[ac->nodes[0].children[c]].fail = 0;
            queue[tail++] = ac->nodes[0].children[c];
        }
    }
    while (head < tail) {
        u = queue[head++];
        for (c = 0; c < 26; c++) {
            v = ac->nodes[u].children[c];
            if (v != -1) {
                f = ac->nodes[u].fail;
                while (f != 0 && ac->nodes[f].children[c] == -1) {
                    f = ac->nodes[f].fail;
                }
                if (ac->nodes[f].children[c] != -1 && ac->nodes[f].children[c] != v) {
                    ac->nodes[v].fail = ac->nodes[f].children[c];
                } else {
                    ac->nodes[v].fail = 0;
                }
                queue[tail++] = v;
            }
        }
    }
}

int str_ac_search(const str_ac_automaton *ac, const char *text, int *matches, int max_matches) {
    int state, i, c, count, temp;
    state = 0;
    count = 0;
    i = 0;
    while (text[i] != '\0' && count < max_matches) {
        c = text[i] - 'a';
        if (c < 0 || c >= 26) { state = 0; i++; continue; }
        while (state != 0 && ac->nodes[state].children[c] == -1) {
            state = ac->nodes[state].fail;
        }
        if (ac->nodes[state].children[c] != -1) {
            state = ac->nodes[state].children[c];
        }
        temp = state;
        while (temp != 0) {
            if (ac->nodes[temp].output >= 0) {
                matches[count++] = i - ac->nodes[temp].depth + 1;
            }
            temp = ac->nodes[temp].fail;
        }
        i++;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C960: Aho-Corasick multi-pattern search should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C960: Output should not be empty");
    assert!(
        code.contains("fn str_ac_search"),
        "C960: Should contain str_ac_search function"
    );
}

// ============================================================================
// C961-C965: Sequence Comparison, Hashing, Encoding
// ============================================================================

#[test]
fn c961_longest_common_subsequence() {
    let c_code = r#"
int str_lcs_length(const char *a, int a_len, const char *b, int b_len) {
    int dp[256][256];
    int i, j;
    if (a_len > 255 || b_len > 255) return -1;
    for (i = 0; i <= a_len; i++) dp[i][0] = 0;
    for (j = 0; j <= b_len; j++) dp[0][j] = 0;
    for (i = 1; i <= a_len; i++) {
        for (j = 1; j <= b_len; j++) {
            if (a[i - 1] == b[j - 1]) {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j] > dp[i][j - 1] ? dp[i - 1][j] : dp[i][j - 1];
            }
        }
    }
    return dp[a_len][b_len];
}

int str_lcs_recover(const char *a, int a_len, const char *b, int b_len, char *out) {
    int dp[256][256];
    int i, j, idx;
    if (a_len > 255 || b_len > 255) return -1;
    for (i = 0; i <= a_len; i++) dp[i][0] = 0;
    for (j = 0; j <= b_len; j++) dp[0][j] = 0;
    for (i = 1; i <= a_len; i++) {
        for (j = 1; j <= b_len; j++) {
            if (a[i - 1] == b[j - 1]) {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j] > dp[i][j - 1] ? dp[i - 1][j] : dp[i][j - 1];
            }
        }
    }
    idx = dp[a_len][b_len];
    out[idx] = '\0';
    i = a_len;
    j = b_len;
    while (i > 0 && j > 0) {
        if (a[i - 1] == b[j - 1]) {
            out[--idx] = a[i - 1];
            i--;
            j--;
        } else if (dp[i - 1][j] > dp[i][j - 1]) {
            i--;
        } else {
            j--;
        }
    }
    return dp[a_len][b_len];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C961: Longest common subsequence should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C961: Output should not be empty");
    assert!(
        code.contains("fn str_lcs_length"),
        "C961: Should contain str_lcs_length function"
    );
}

#[test]
fn c962_polynomial_rolling_hash() {
    let c_code = r#"
typedef unsigned long uint64_t;

typedef struct {
    uint64_t hash;
    uint64_t base;
    uint64_t mod_val;
    uint64_t high_pow;
    int window_size;
} str_rolling_hash;

void str_rhash_init(str_rolling_hash *rh, uint64_t base, uint64_t mod_val, int window) {
    uint64_t pw;
    int i;
    rh->hash = 0;
    rh->base = base;
    rh->mod_val = mod_val;
    rh->window_size = window;
    pw = 1;
    for (i = 0; i < window - 1; i++) {
        pw = (pw * base) % mod_val;
    }
    rh->high_pow = pw;
}

void str_rhash_add(str_rolling_hash *rh, char c) {
    rh->hash = (rh->hash * rh->base + (uint64_t)c) % rh->mod_val;
}

void str_rhash_slide(str_rolling_hash *rh, char old_c, char new_c) {
    rh->hash = (rh->hash + rh->mod_val - (uint64_t)old_c * rh->high_pow % rh->mod_val) % rh->mod_val;
    rh->hash = (rh->hash * rh->base + (uint64_t)new_c) % rh->mod_val;
}

uint64_t str_hash_string(const char *s, int len, uint64_t base, uint64_t mod_val) {
    uint64_t h = 0;
    int i;
    for (i = 0; i < len; i++) {
        h = (h * base + (uint64_t)s[i]) % mod_val;
    }
    return h;
}

int str_hash_match(const char *text, int text_len, const char *pattern, int pat_len) {
    str_rolling_hash rh;
    uint64_t pat_hash;
    int i, j, match_flag;
    if (pat_len > text_len) return -1;
    str_rhash_init(&rh, 31, 1000000007, pat_len);
    pat_hash = str_hash_string(pattern, pat_len, 31, 1000000007);
    for (i = 0; i < pat_len; i++) str_rhash_add(&rh, text[i]);
    if (rh.hash == pat_hash) {
        match_flag = 1;
        for (j = 0; j < pat_len; j++) {
            if (text[j] != pattern[j]) { match_flag = 0; break; }
        }
        if (match_flag) return 0;
    }
    for (i = 1; i <= text_len - pat_len; i++) {
        str_rhash_slide(&rh, text[i - 1], text[i + pat_len - 1]);
        if (rh.hash == pat_hash) {
            match_flag = 1;
            for (j = 0; j < pat_len; j++) {
                if (text[i + j] != pattern[j]) { match_flag = 0; break; }
            }
            if (match_flag) return i;
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C962: Polynomial rolling hash should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C962: Output should not be empty");
    assert!(
        code.contains("fn str_rhash_init"),
        "C962: Should contain str_rhash_init function"
    );
}

#[test]
fn c963_run_length_encoding() {
    let c_code = r#"
int str_rle_encode(const char *input, int in_len, char *output, int out_max) {
    int i, j, count, out_pos;
    out_pos = 0;
    i = 0;
    while (i < in_len) {
        count = 1;
        while (i + count < in_len && input[i] == input[i + count] && count < 255) {
            count++;
        }
        if (count > 1) {
            if (out_pos + 3 > out_max) return -1;
            output[out_pos++] = '#';
            output[out_pos++] = (char)(count + '0');
            output[out_pos++] = input[i];
        } else {
            if (out_pos + 1 > out_max) return -1;
            if (input[i] == '#') {
                if (out_pos + 3 > out_max) return -1;
                output[out_pos++] = '#';
                output[out_pos++] = '1';
                output[out_pos++] = '#';
            } else {
                output[out_pos++] = input[i];
            }
        }
        i += count;
    }
    if (out_pos < out_max) output[out_pos] = '\0';
    return out_pos;
}

int str_rle_decode(const char *input, int in_len, char *output, int out_max) {
    int i, count, out_pos;
    out_pos = 0;
    i = 0;
    while (i < in_len) {
        if (input[i] == '#' && i + 2 < in_len) {
            count = input[i + 1] - '0';
            while (count > 0 && out_pos < out_max) {
                output[out_pos++] = input[i + 2];
                count--;
            }
            i += 3;
        } else {
            if (out_pos >= out_max) return -1;
            output[out_pos++] = input[i];
            i++;
        }
    }
    if (out_pos < out_max) output[out_pos] = '\0';
    return out_pos;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C963: Run-length encoding should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C963: Output should not be empty");
    assert!(
        code.contains("fn str_rle_encode"),
        "C963: Should contain str_rle_encode function"
    );
}

#[test]
fn c964_huffman_coding() {
    let c_code = r#"
typedef struct {
    int freq;
    int left;
    int right;
    char ch;
    int is_leaf;
} str_huff_node;

typedef struct {
    str_huff_node nodes[512];
    int node_count;
    int root;
    int code_len[256];
    int codes[256];
} str_huffman_tree;

void str_huff_init(str_huffman_tree *ht) {
    int i;
    ht->node_count = 0;
    ht->root = -1;
    for (i = 0; i < 256; i++) {
        ht->code_len[i] = 0;
        ht->codes[i] = 0;
    }
}

int str_huff_add_leaf(str_huffman_tree *ht, char ch, int freq) {
    int idx;
    if (ht->node_count >= 512) return -1;
    idx = ht->node_count++;
    ht->nodes[idx].ch = ch;
    ht->nodes[idx].freq = freq;
    ht->nodes[idx].left = -1;
    ht->nodes[idx].right = -1;
    ht->nodes[idx].is_leaf = 1;
    return idx;
}

int str_huff_add_internal(str_huffman_tree *ht, int left, int right) {
    int idx;
    if (ht->node_count >= 512) return -1;
    idx = ht->node_count++;
    ht->nodes[idx].ch = 0;
    ht->nodes[idx].freq = ht->nodes[left].freq + ht->nodes[right].freq;
    ht->nodes[idx].left = left;
    ht->nodes[idx].right = right;
    ht->nodes[idx].is_leaf = 0;
    return idx;
}

void str_huff_build(str_huffman_tree *ht, const char *text, int text_len) {
    int freq[256];
    int queue[512];
    int qsize, i, min1, min2, min1_idx, min2_idx, j, new_node;
    for (i = 0; i < 256; i++) freq[i] = 0;
    for (i = 0; i < text_len; i++) freq[(unsigned char)text[i]]++;
    qsize = 0;
    for (i = 0; i < 256; i++) {
        if (freq[i] > 0) {
            queue[qsize++] = str_huff_add_leaf(ht, (char)i, freq[i]);
        }
    }
    while (qsize > 1) {
        min1 = 0;
        for (i = 1; i < qsize; i++) {
            if (ht->nodes[queue[i]].freq < ht->nodes[queue[min1]].freq) min1 = i;
        }
        min1_idx = queue[min1];
        queue[min1] = queue[--qsize];
        min2 = 0;
        for (i = 1; i < qsize; i++) {
            if (ht->nodes[queue[i]].freq < ht->nodes[queue[min2]].freq) min2 = i;
        }
        min2_idx = queue[min2];
        queue[min2] = queue[--qsize];
        new_node = str_huff_add_internal(ht, min1_idx, min2_idx);
        queue[qsize++] = new_node;
    }
    if (qsize == 1) ht->root = queue[0];
}

void str_huff_gen_codes(str_huffman_tree *ht, int node, int code, int depth) {
    if (node < 0) return;
    if (ht->nodes[node].is_leaf) {
        ht->codes[(unsigned char)ht->nodes[node].ch] = code;
        ht->code_len[(unsigned char)ht->nodes[node].ch] = depth > 0 ? depth : 1;
        return;
    }
    str_huff_gen_codes(ht, ht->nodes[node].left, code << 1, depth + 1);
    str_huff_gen_codes(ht, ht->nodes[node].right, (code << 1) | 1, depth + 1);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C964: Huffman coding should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C964: Output should not be empty");
    assert!(
        code.contains("fn str_huff_build"),
        "C964: Should contain str_huff_build function"
    );
}

#[test]
fn c965_base64_encode_decode() {
    let c_code = r#"
typedef unsigned char uint8_t;

static const char str_b64_table[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

int str_b64_char_to_val(char c) {
    if (c >= 'A' && c <= 'Z') return c - 'A';
    if (c >= 'a' && c <= 'z') return c - 'a' + 26;
    if (c >= '0' && c <= '9') return c - '0' + 52;
    if (c == '+') return 62;
    if (c == '/') return 63;
    return -1;
}

int str_base64_encode(const uint8_t *input, int in_len, char *output, int out_max) {
    int i, out_pos, val;
    out_pos = 0;
    for (i = 0; i < in_len; i += 3) {
        if (out_pos + 4 > out_max) return -1;
        val = (input[i] << 16);
        if (i + 1 < in_len) val |= (input[i + 1] << 8);
        if (i + 2 < in_len) val |= input[i + 2];
        output[out_pos++] = str_b64_table[(val >> 18) & 0x3F];
        output[out_pos++] = str_b64_table[(val >> 12) & 0x3F];
        output[out_pos++] = (i + 1 < in_len) ? str_b64_table[(val >> 6) & 0x3F] : '=';
        output[out_pos++] = (i + 2 < in_len) ? str_b64_table[val & 0x3F] : '=';
    }
    if (out_pos < out_max) output[out_pos] = '\0';
    return out_pos;
}

int str_base64_decode(const char *input, int in_len, uint8_t *output, int out_max) {
    int i, out_pos, v0, v1, v2, v3;
    out_pos = 0;
    for (i = 0; i + 3 < in_len; i += 4) {
        v0 = str_b64_char_to_val(input[i]);
        v1 = str_b64_char_to_val(input[i + 1]);
        v2 = (input[i + 2] != '=') ? str_b64_char_to_val(input[i + 2]) : 0;
        v3 = (input[i + 3] != '=') ? str_b64_char_to_val(input[i + 3]) : 0;
        if (v0 < 0 || v1 < 0) return -1;
        if (out_pos >= out_max) return -1;
        output[out_pos++] = (uint8_t)((v0 << 2) | (v1 >> 4));
        if (input[i + 2] != '=' && out_pos < out_max) {
            output[out_pos++] = (uint8_t)(((v1 & 0xF) << 4) | (v2 >> 2));
        }
        if (input[i + 3] != '=' && out_pos < out_max) {
            output[out_pos++] = (uint8_t)(((v2 & 0x3) << 6) | v3);
        }
    }
    return out_pos;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C965: Base64 encode/decode should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C965: Output should not be empty");
    assert!(
        code.contains("fn str_base64_encode"),
        "C965: Should contain str_base64_encode function"
    );
}

// ============================================================================
// C966-C970: Transforms, Automata, Pattern Matching
// ============================================================================

#[test]
fn c966_burrows_wheeler_transform() {
    let c_code = r#"
static int str_bwt_len;
static const char *str_bwt_text;

int str_bwt_compare(const void *a, const void *b) {
    int ia = *(const int *)a;
    int ib = *(const int *)b;
    int k;
    for (k = 0; k < str_bwt_len; k++) {
        int ca = str_bwt_text[(ia + k) % str_bwt_len];
        int cb = str_bwt_text[(ib + k) % str_bwt_len];
        if (ca < cb) return -1;
        if (ca > cb) return 1;
    }
    return 0;
}

int str_bwt_transform(const char *input, int len, char *output) {
    int indices[512];
    int i, j, min_idx, temp, primary;
    if (len > 512) return -1;
    str_bwt_len = len;
    str_bwt_text = input;
    for (i = 0; i < len; i++) indices[i] = i;
    for (i = 0; i < len - 1; i++) {
        min_idx = i;
        for (j = i + 1; j < len; j++) {
            if (str_bwt_compare(&indices[j], &indices[min_idx]) < 0) {
                min_idx = j;
            }
        }
        if (min_idx != i) {
            temp = indices[i];
            indices[i] = indices[min_idx];
            indices[min_idx] = temp;
        }
    }
    primary = -1;
    for (i = 0; i < len; i++) {
        output[i] = input[(indices[i] + len - 1) % len];
        if (indices[i] == 0) primary = i;
    }
    output[len] = '\0';
    return primary;
}

void str_bwt_inverse(const char *bwt, int len, int primary, char *output) {
    int count[256];
    int first_occ[256];
    int t_table[512];
    int i, c, idx;
    for (i = 0; i < 256; i++) count[i] = 0;
    for (i = 0; i < len; i++) count[(unsigned char)bwt[i]]++;
    first_occ[0] = 0;
    for (i = 1; i < 256; i++) first_occ[i] = first_occ[i - 1] + count[i - 1];
    for (i = 0; i < 256; i++) count[i] = 0;
    for (i = 0; i < len; i++) {
        c = (unsigned char)bwt[i];
        t_table[i] = first_occ[c] + count[c];
        count[c]++;
    }
    idx = primary;
    for (i = len - 1; i >= 0; i--) {
        output[i] = bwt[idx];
        idx = t_table[idx];
    }
    output[len] = '\0';
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C966: Burrows-Wheeler transform should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C966: Output should not be empty");
    assert!(
        code.contains("fn str_bwt_transform"),
        "C966: Should contain str_bwt_transform function"
    );
}

#[test]
fn c967_suffix_automaton() {
    let c_code = r#"
typedef struct {
    int len;
    int link;
    int transitions[26];
} str_sa_state;

typedef struct {
    str_sa_state states[2048];
    int state_count;
    int last;
} str_suffix_automaton;

void str_sam_init(str_suffix_automaton *sam) {
    int j;
    sam->state_count = 1;
    sam->states[0].len = 0;
    sam->states[0].link = -1;
    for (j = 0; j < 26; j++) sam->states[0].transitions[j] = -1;
    sam->last = 0;
}

void str_sam_extend(str_suffix_automaton *sam, char ch) {
    int c = ch - 'a';
    int cur, p, q, clone, j;
    if (c < 0 || c >= 26) return;
    if (sam->state_count >= 2047) return;
    cur = sam->state_count++;
    sam->states[cur].len = sam->states[sam->last].len + 1;
    sam->states[cur].link = -1;
    for (j = 0; j < 26; j++) sam->states[cur].transitions[j] = -1;
    p = sam->last;
    while (p != -1 && sam->states[p].transitions[c] == -1) {
        sam->states[p].transitions[c] = cur;
        p = sam->states[p].link;
    }
    if (p == -1) {
        sam->states[cur].link = 0;
    } else {
        q = sam->states[p].transitions[c];
        if (sam->states[p].len + 1 == sam->states[q].len) {
            sam->states[cur].link = q;
        } else {
            if (sam->state_count >= 2047) return;
            clone = sam->state_count++;
            sam->states[clone] = sam->states[q];
            sam->states[clone].len = sam->states[p].len + 1;
            while (p != -1 && sam->states[p].transitions[c] == q) {
                sam->states[p].transitions[c] = clone;
                p = sam->states[p].link;
            }
            sam->states[q].link = clone;
            sam->states[cur].link = clone;
        }
    }
    sam->last = cur;
}

int str_sam_contains(const str_suffix_automaton *sam, const char *pattern) {
    int state = 0;
    int i = 0;
    int c;
    while (pattern[i] != '\0') {
        c = pattern[i] - 'a';
        if (c < 0 || c >= 26) return 0;
        if (sam->states[state].transitions[c] == -1) return 0;
        state = sam->states[state].transitions[c];
        i++;
    }
    return 1;
}

long str_sam_count_distinct(const str_suffix_automaton *sam) {
    long total = 0;
    int i;
    for (i = 1; i < sam->state_count; i++) {
        total += sam->states[i].len - sam->states[sam->states[i].link].len;
    }
    return total;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C967: Suffix automaton should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C967: Output should not be empty");
    assert!(
        code.contains("fn str_sam_extend"),
        "C967: Should contain str_sam_extend function"
    );
}

#[test]
fn c968_palindrome_partitioning() {
    let c_code = r#"
static int str_is_palindrome(const char *s, int start, int end) {
    while (start < end) {
        if (s[start] != s[end]) return 0;
        start++;
        end--;
    }
    return 1;
}

int str_min_palindrome_cuts(const char *s, int n) {
    int dp[512];
    int pal[512][512];
    int i, j, k, min_val;
    if (n <= 1) return 0;
    if (n > 512) return -1;
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            pal[i][j] = 0;
        }
    }
    for (i = 0; i < n; i++) pal[i][i] = 1;
    for (j = 1; j < n; j++) {
        for (i = 0; i <= j; i++) {
            if (s[i] == s[j]) {
                if (j - i <= 2 || pal[i + 1][j - 1]) {
                    pal[i][j] = 1;
                }
            }
        }
    }
    for (i = 0; i < n; i++) {
        if (pal[0][i]) {
            dp[i] = 0;
        } else {
            dp[i] = i;
            for (k = 1; k <= i; k++) {
                if (pal[k][i] && dp[k - 1] + 1 < dp[i]) {
                    dp[i] = dp[k - 1] + 1;
                }
            }
        }
    }
    return dp[n - 1];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C968: Palindrome partitioning should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C968: Output should not be empty");
    assert!(
        code.contains("fn str_min_palindrome_cuts"),
        "C968: Should contain str_min_palindrome_cuts function"
    );
}

#[test]
fn c969_wildcard_matching() {
    let c_code = r#"
int str_wildcard_match(const char *text, const char *pattern) {
    int t = 0;
    int p = 0;
    int star_p = -1;
    int star_t = -1;
    while (text[t] != '\0') {
        if (pattern[p] == '?' || pattern[p] == text[t]) {
            t++;
            p++;
        } else if (pattern[p] == '*') {
            star_p = p;
            star_t = t;
            p++;
        } else if (star_p >= 0) {
            p = star_p + 1;
            star_t++;
            t = star_t;
        } else {
            return 0;
        }
    }
    while (pattern[p] == '*') p++;
    return pattern[p] == '\0' ? 1 : 0;
}

int str_wildcard_match_dp(const char *text, int t_len, const char *pattern, int p_len) {
    int dp[256][256];
    int i, j;
    if (t_len > 255 || p_len > 255) return -1;
    dp[0][0] = 1;
    for (j = 1; j <= p_len; j++) {
        dp[0][j] = (pattern[j - 1] == '*') ? dp[0][j - 1] : 0;
    }
    for (i = 1; i <= t_len; i++) dp[i][0] = 0;
    for (i = 1; i <= t_len; i++) {
        for (j = 1; j <= p_len; j++) {
            if (pattern[j - 1] == '*') {
                dp[i][j] = dp[i - 1][j] || dp[i][j - 1];
            } else if (pattern[j - 1] == '?' || pattern[j - 1] == text[i - 1]) {
                dp[i][j] = dp[i - 1][j - 1];
            } else {
                dp[i][j] = 0;
            }
        }
    }
    return dp[t_len][p_len];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C969: Wildcard pattern matching should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C969: Output should not be empty");
    assert!(
        code.contains("fn str_wildcard_match"),
        "C969: Should contain str_wildcard_match function"
    );
}

#[test]
fn c970_regex_matching() {
    let c_code = r#"
int str_regex_match_helper(const char *text, const char *pattern) {
    if (pattern[0] == '\0') {
        return text[0] == '\0' ? 1 : 0;
    }
    if (pattern[1] == '*') {
        if (str_regex_match_helper(text, pattern + 2)) return 1;
        while (text[0] != '\0' && (pattern[0] == '.' || pattern[0] == text[0])) {
            text++;
            if (str_regex_match_helper(text, pattern + 2)) return 1;
        }
        return 0;
    }
    if (text[0] != '\0' && (pattern[0] == '.' || pattern[0] == text[0])) {
        return str_regex_match_helper(text + 1, pattern + 1);
    }
    return 0;
}

int str_regex_match(const char *text, const char *pattern) {
    return str_regex_match_helper(text, pattern);
}

int str_regex_match_dp(const char *text, int t_len, const char *pattern, int p_len) {
    int dp[128][128];
    int i, j;
    if (t_len > 127 || p_len > 127) return -1;
    dp[0][0] = 1;
    for (j = 1; j <= p_len; j++) {
        dp[0][j] = (j >= 2 && pattern[j - 1] == '*') ? dp[0][j - 2] : 0;
    }
    for (i = 1; i <= t_len; i++) dp[i][0] = 0;
    for (i = 1; i <= t_len; i++) {
        for (j = 1; j <= p_len; j++) {
            if (pattern[j - 1] == '*') {
                dp[i][j] = dp[i][j - 2];
                if (pattern[j - 2] == '.' || pattern[j - 2] == text[i - 1]) {
                    dp[i][j] = dp[i][j] || dp[i - 1][j];
                }
            } else if (pattern[j - 1] == '.' || pattern[j - 1] == text[i - 1]) {
                dp[i][j] = dp[i - 1][j - 1];
            } else {
                dp[i][j] = 0;
            }
        }
    }
    return dp[t_len][p_len];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C970: Regex matching should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C970: Output should not be empty");
    assert!(
        code.contains("fn str_regex_match"),
        "C970: Should contain str_regex_match function"
    );
}

// ============================================================================
// C971-C975: Classic String DP Problems
// ============================================================================

#[test]
fn c971_string_rotation_check() {
    let c_code = r#"
static int str_rot_len(const char *s) {
    int len = 0;
    while (s[len] != '\0') len++;
    return len;
}

int str_is_rotation(const char *s1, const char *s2) {
    int len1, len2, i, j, match_flag;
    len1 = str_rot_len(s1);
    len2 = str_rot_len(s2);
    if (len1 != len2) return 0;
    if (len1 == 0) return 1;
    for (i = 0; i < len1; i++) {
        match_flag = 1;
        for (j = 0; j < len1; j++) {
            if (s1[(i + j) % len1] != s2[j]) {
                match_flag = 0;
                break;
            }
        }
        if (match_flag) return 1;
    }
    return 0;
}

int str_min_rotation(const char *s, int len) {
    int i, j, k, best;
    best = 0;
    i = 0;
    j = 1;
    k = 0;
    while (i < len && j < len && k < len) {
        int ci = s[(i + k) % len];
        int cj = s[(j + k) % len];
        if (ci == cj) {
            k++;
        } else if (ci > cj) {
            i = i + k + 1;
            if (i == j) i++;
            k = 0;
        } else {
            j = j + k + 1;
            if (j == i) j++;
            k = 0;
        }
    }
    best = i < j ? i : j;
    return best;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C971: String rotation check should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C971: Output should not be empty");
    assert!(
        code.contains("fn str_is_rotation"),
        "C971: Should contain str_is_rotation function"
    );
}

#[test]
fn c972_anagram_detection() {
    let c_code = r#"
int str_is_anagram(const char *s1, const char *s2) {
    int count[256];
    int i;
    for (i = 0; i < 256; i++) count[i] = 0;
    i = 0;
    while (s1[i] != '\0') {
        count[(unsigned char)s1[i]]++;
        i++;
    }
    i = 0;
    while (s2[i] != '\0') {
        count[(unsigned char)s2[i]]--;
        i++;
    }
    for (i = 0; i < 256; i++) {
        if (count[i] != 0) return 0;
    }
    return 1;
}

int str_count_anagram_substrings(const char *text, int text_len, const char *pattern, int pat_len) {
    int count_p[256];
    int count_w[256];
    int i, j, matches, matching;
    if (pat_len > text_len) return 0;
    for (i = 0; i < 256; i++) { count_p[i] = 0; count_w[i] = 0; }
    for (i = 0; i < pat_len; i++) {
        count_p[(unsigned char)pattern[i]]++;
        count_w[(unsigned char)text[i]]++;
    }
    matches = 0;
    for (i = 0; i <= text_len - pat_len; i++) {
        if (i > 0) {
            count_w[(unsigned char)text[i - 1]]--;
            count_w[(unsigned char)text[i + pat_len - 1]]++;
        }
        matching = 1;
        for (j = 0; j < 256; j++) {
            if (count_p[j] != count_w[j]) { matching = 0; break; }
        }
        if (matching) matches++;
    }
    return matches;
}

void str_group_anagrams(const char **words, int n, int *groups) {
    int sorted[64][128];
    int lens[64];
    int i, j, k, temp;
    int group_id;
    for (i = 0; i < n && i < 64; i++) {
        k = 0;
        while (words[i][k] != '\0' && k < 127) {
            sorted[i][k] = words[i][k];
            k++;
        }
        lens[i] = k;
        for (j = 0; j < k - 1; j++) {
            int m;
            for (m = 0; m < k - 1 - j; m++) {
                if (sorted[i][m] > sorted[i][m + 1]) {
                    temp = sorted[i][m];
                    sorted[i][m] = sorted[i][m + 1];
                    sorted[i][m + 1] = temp;
                }
            }
        }
        groups[i] = -1;
    }
    group_id = 0;
    for (i = 0; i < n && i < 64; i++) {
        if (groups[i] >= 0) continue;
        groups[i] = group_id;
        for (j = i + 1; j < n && j < 64; j++) {
            if (groups[j] >= 0) continue;
            if (lens[i] != lens[j]) continue;
            k = 0;
            while (k < lens[i] && sorted[i][k] == sorted[j][k]) k++;
            if (k == lens[i]) groups[j] = group_id;
        }
        group_id++;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C972: Anagram detection should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C972: Output should not be empty");
    assert!(
        code.contains("fn str_is_anagram"),
        "C972: Should contain str_is_anagram function"
    );
}

#[test]
fn c973_longest_repeating_subsequence() {
    let c_code = r#"
int str_longest_repeating_subseq(const char *s, int n) {
    int dp[256][256];
    int i, j;
    if (n > 255) return -1;
    for (i = 0; i <= n; i++) dp[i][0] = 0;
    for (j = 0; j <= n; j++) dp[0][j] = 0;
    for (i = 1; i <= n; i++) {
        for (j = 1; j <= n; j++) {
            if (s[i - 1] == s[j - 1] && i != j) {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j] > dp[i][j - 1] ? dp[i - 1][j] : dp[i][j - 1];
            }
        }
    }
    return dp[n][n];
}

int str_recover_repeating_subseq(const char *s, int n, char *out) {
    int dp[256][256];
    int i, j, idx;
    if (n > 255) return -1;
    for (i = 0; i <= n; i++) dp[i][0] = 0;
    for (j = 0; j <= n; j++) dp[0][j] = 0;
    for (i = 1; i <= n; i++) {
        for (j = 1; j <= n; j++) {
            if (s[i - 1] == s[j - 1] && i != j) {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j] > dp[i][j - 1] ? dp[i - 1][j] : dp[i][j - 1];
            }
        }
    }
    idx = dp[n][n];
    out[idx] = '\0';
    i = n;
    j = n;
    while (i > 0 && j > 0) {
        if (s[i - 1] == s[j - 1] && i != j) {
            out[--idx] = s[i - 1];
            i--;
            j--;
        } else if (dp[i - 1][j] > dp[i][j - 1]) {
            i--;
        } else {
            j--;
        }
    }
    return dp[n][n];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C973: Longest repeating subsequence should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C973: Output should not be empty");
    assert!(
        code.contains("fn str_longest_repeating_subseq"),
        "C973: Should contain str_longest_repeating_subseq function"
    );
}

#[test]
fn c974_word_break() {
    let c_code = r#"
static int str_wb_match(const char *s, int start, int end, const char *word) {
    int i = 0;
    int j = start;
    while (j < end && word[i] != '\0') {
        if (s[j] != word[i]) return 0;
        i++;
        j++;
    }
    return (j == end && word[i] == '\0') ? 1 : 0;
}

int str_word_break(const char *s, int s_len, const char **dict, int dict_size) {
    int dp[512];
    int i, j, k;
    if (s_len > 511) return 0;
    for (i = 0; i <= s_len; i++) dp[i] = 0;
    dp[0] = 1;
    for (i = 1; i <= s_len; i++) {
        for (j = 0; j < i; j++) {
            if (dp[j] == 0) continue;
            for (k = 0; k < dict_size; k++) {
                if (str_wb_match(s, j, i, dict[k])) {
                    dp[i] = 1;
                    break;
                }
            }
            if (dp[i]) break;
        }
    }
    return dp[s_len];
}

int str_word_break_count(const char *s, int s_len, const char **dict, int dict_size) {
    int dp[512];
    int i, j, k;
    if (s_len > 511) return 0;
    for (i = 0; i <= s_len; i++) dp[i] = 0;
    dp[0] = 1;
    for (i = 1; i <= s_len; i++) {
        for (j = 0; j < i; j++) {
            if (dp[j] == 0) continue;
            for (k = 0; k < dict_size; k++) {
                if (str_wb_match(s, j, i, dict[k])) {
                    dp[i] += dp[j];
                }
            }
        }
    }
    return dp[s_len];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C974: Word break problem should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C974: Output should not be empty");
    assert!(
        code.contains("fn str_word_break"),
        "C974: Should contain str_word_break function"
    );
}

#[test]
fn c975_minimum_window_substring() {
    let c_code = r#"
int str_min_window(const char *s, int s_len, const char *t, int t_len,
                   int *win_start, int *win_len) {
    int need[256];
    int have[256];
    int required, formed;
    int left, right, best_len, best_start;
    int i;
    if (t_len > s_len || t_len == 0) {
        *win_start = -1;
        *win_len = 0;
        return 0;
    }
    for (i = 0; i < 256; i++) { need[i] = 0; have[i] = 0; }
    required = 0;
    for (i = 0; i < t_len; i++) {
        if (need[(unsigned char)t[i]] == 0) required++;
        need[(unsigned char)t[i]]++;
    }
    formed = 0;
    left = 0;
    best_len = s_len + 1;
    best_start = 0;
    for (right = 0; right < s_len; right++) {
        unsigned char c = (unsigned char)s[right];
        have[c]++;
        if (need[c] > 0 && have[c] == need[c]) {
            formed++;
        }
        while (formed == required && left <= right) {
            int cur_len = right - left + 1;
            if (cur_len < best_len) {
                best_len = cur_len;
                best_start = left;
            }
            {
                unsigned char lc = (unsigned char)s[left];
                have[lc]--;
                if (need[lc] > 0 && have[lc] < need[lc]) {
                    formed--;
                }
            }
            left++;
        }
    }
    if (best_len > s_len) {
        *win_start = -1;
        *win_len = 0;
        return 0;
    }
    *win_start = best_start;
    *win_len = best_len;
    return 1;
}

int str_min_window_contains(const char *s, int s_len, const char *t, int t_len) {
    int start, len;
    return str_min_window(s, s_len, t, t_len, &start, &len);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C975: Minimum window substring should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C975: Output should not be empty");
    assert!(
        code.contains("fn str_min_window"),
        "C975: Should contain str_min_window function"
    );
}
