//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1001-C1025: Dynamic Programming implementations -- the kind of C code found
//! in CLRS, Sedgewick, competitive programming, and real-world optimization.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise classic DP algorithm patterns expressed as
//! valid C99 with array-based representations (no malloc/free, no #include).
//!
//! Organization:
//! - C1001-C1005: Classic DP (0/1 knapsack, coin change, LIS, matrix chain, rod cutting)
//! - C1006-C1010: Combinatorial DP (egg drop, subset sum, partition, palindrome subseq, edit distance)
//! - C1011-C1015: Tree/number DP (optimal BST, Catalan, Bell, Kadane, max sum rectangle)
//! - C1016-C1020: String/sequence DP (LCS 3-string, wildcard, interleaving, palindrome partition, word wrap)
//! - C1021-C1025: Advanced DP (TSP bitmask, bitonic subseq, box stacking, assembly line, min cost path)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C1001-C1005: Classic DP
// ============================================================================

#[test]
fn c1001_knapsack_01() {
    let c_code = r#"
int dp_knapsack_01(int W, int n, int weights[], int values[]) {
    int dp[101][1001];
    int i, w;

    for (i = 0; i <= n; i++) {
        for (w = 0; w <= W; w++) {
            if (i == 0 || w == 0) {
                dp[i][w] = 0;
            } else if (weights[i - 1] <= w) {
                int include = values[i - 1] + dp[i - 1][w - weights[i - 1]];
                int exclude = dp[i - 1][w];
                dp[i][w] = include > exclude ? include : exclude;
            } else {
                dp[i][w] = dp[i - 1][w];
            }
        }
    }
    return dp[n][W];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1001: 0/1 Knapsack - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1001: empty output");
    assert!(code.contains("fn dp_knapsack_01"), "C1001: Should contain dp_knapsack_01");
}

#[test]
fn c1002_coin_change() {
    let c_code = r#"
int dp_coin_change(int coins[], int n, int amount) {
    int dp[10001];
    int i, j;
    int max_val = amount + 1;

    for (i = 0; i <= amount; i++)
        dp[i] = max_val;
    dp[0] = 0;

    for (i = 0; i < n; i++) {
        for (j = coins[i]; j <= amount; j++) {
            if (dp[j - coins[i]] + 1 < dp[j]) {
                dp[j] = dp[j - coins[i]] + 1;
            }
        }
    }
    return dp[amount] > amount ? -1 : dp[amount];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1002: Coin change - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1002: empty output");
    assert!(code.contains("fn dp_coin_change"), "C1002: Should contain dp_coin_change");
}

#[test]
fn c1003_longest_increasing_subsequence() {
    let c_code = r#"
int dp_lis(int arr[], int n) {
    int dp[1000];
    int i, j;
    int max_len = 1;

    for (i = 0; i < n; i++)
        dp[i] = 1;

    for (i = 1; i < n; i++) {
        for (j = 0; j < i; j++) {
            if (arr[j] < arr[i] && dp[j] + 1 > dp[i]) {
                dp[i] = dp[j] + 1;
            }
        }
        if (dp[i] > max_len)
            max_len = dp[i];
    }
    return max_len;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1003: LIS - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1003: empty output");
    assert!(code.contains("fn dp_lis"), "C1003: Should contain dp_lis");
}

#[test]
fn c1004_matrix_chain_multiplication() {
    let c_code = r#"
int dp_matrix_chain(int dims[], int n) {
    int dp[50][50];
    int i, j, k, L;

    for (i = 0; i < n; i++)
        dp[i][i] = 0;

    for (L = 2; L < n; L++) {
        for (i = 1; i < n - L + 1; i++) {
            j = i + L - 1;
            dp[i][j] = 2147483647;
            for (k = i; k < j; k++) {
                int cost = dp[i][k] + dp[k + 1][j] + dims[i - 1] * dims[k] * dims[j];
                if (cost < dp[i][j])
                    dp[i][j] = cost;
            }
        }
    }
    return dp[1][n - 1];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1004: Matrix chain - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1004: empty output");
    assert!(code.contains("fn dp_matrix_chain"), "C1004: Should contain dp_matrix_chain");
}

#[test]
fn c1005_rod_cutting() {
    let c_code = r#"
int dp_rod_cutting(int prices[], int n) {
    int dp[1001];
    int i, j;

    dp[0] = 0;

    for (i = 1; i <= n; i++) {
        dp[i] = -1;
        for (j = 1; j <= i; j++) {
            int val = prices[j - 1] + dp[i - j];
            if (val > dp[i])
                dp[i] = val;
        }
    }
    return dp[n];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1005: Rod cutting - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1005: empty output");
    assert!(code.contains("fn dp_rod_cutting"), "C1005: Should contain dp_rod_cutting");
}

// ============================================================================
// C1006-C1010: Combinatorial DP
// ============================================================================

#[test]
fn c1006_egg_drop() {
    let c_code = r#"
int dp_egg_drop(int eggs, int floors) {
    int dp[11][101];
    int i, j, x;

    for (i = 0; i <= eggs; i++) {
        dp[i][0] = 0;
        dp[i][1] = 1;
    }
    for (j = 0; j <= floors; j++)
        dp[1][j] = j;

    for (i = 2; i <= eggs; i++) {
        for (j = 2; j <= floors; j++) {
            dp[i][j] = 2147483647;
            for (x = 1; x <= j; x++) {
                int breaks = dp[i - 1][x - 1];
                int survives = dp[i][j - x];
                int worst = (breaks > survives ? breaks : survives) + 1;
                if (worst < dp[i][j])
                    dp[i][j] = worst;
            }
        }
    }
    return dp[eggs][floors];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1006: Egg drop - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1006: empty output");
    assert!(code.contains("fn dp_egg_drop"), "C1006: Should contain dp_egg_drop");
}

#[test]
fn c1007_subset_sum() {
    let c_code = r#"
int dp_subset_sum(int set[], int n, int target) {
    int dp[101][10001];
    int i, j;

    for (i = 0; i <= n; i++)
        dp[i][0] = 1;
    for (j = 1; j <= target; j++)
        dp[0][j] = 0;

    for (i = 1; i <= n; i++) {
        for (j = 1; j <= target; j++) {
            dp[i][j] = dp[i - 1][j];
            if (j >= set[i - 1]) {
                dp[i][j] = dp[i][j] || dp[i - 1][j - set[i - 1]];
            }
        }
    }
    return dp[n][target];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1007: Subset sum - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1007: empty output");
    assert!(code.contains("fn dp_subset_sum"), "C1007: Should contain dp_subset_sum");
}

#[test]
fn c1008_partition_equal_subset_sum() {
    let c_code = r#"
int dp_can_partition(int nums[], int n) {
    int total = 0;
    int i, j;
    int dp[20001];

    for (i = 0; i < n; i++)
        total += nums[i];

    if (total % 2 != 0)
        return 0;

    int half = total / 2;

    for (j = 0; j <= half; j++)
        dp[j] = 0;
    dp[0] = 1;

    for (i = 0; i < n; i++) {
        for (j = half; j >= nums[i]; j--) {
            if (dp[j - nums[i]])
                dp[j] = 1;
        }
    }
    return dp[half];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1008: Partition equal subset - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1008: empty output");
    assert!(code.contains("fn dp_can_partition"), "C1008: Should contain dp_can_partition");
}

#[test]
fn c1009_longest_palindromic_subsequence() {
    let c_code = r#"
int dp_lps(const char *s, int n) {
    int dp[500][500];
    int i, j, L;

    for (i = 0; i < n; i++)
        dp[i][i] = 1;

    for (L = 2; L <= n; L++) {
        for (i = 0; i < n - L + 1; i++) {
            j = i + L - 1;
            if (s[i] == s[j] && L == 2) {
                dp[i][j] = 2;
            } else if (s[i] == s[j]) {
                dp[i][j] = dp[i + 1][j - 1] + 2;
            } else {
                dp[i][j] = dp[i + 1][j] > dp[i][j - 1] ? dp[i + 1][j] : dp[i][j - 1];
            }
        }
    }
    return dp[0][n - 1];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1009: Longest palindromic subseq - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1009: empty output");
    assert!(code.contains("fn dp_lps"), "C1009: Should contain dp_lps");
}

#[test]
fn c1010_edit_distance() {
    let c_code = r#"
int dp_edit_distance(const char *s1, int m, const char *s2, int n) {
    int dp[501][501];
    int i, j;

    for (i = 0; i <= m; i++)
        dp[i][0] = i;
    for (j = 0; j <= n; j++)
        dp[0][j] = j;

    for (i = 1; i <= m; i++) {
        for (j = 1; j <= n; j++) {
            if (s1[i - 1] == s2[j - 1]) {
                dp[i][j] = dp[i - 1][j - 1];
            } else {
                int ins = dp[i][j - 1];
                int del = dp[i - 1][j];
                int rep = dp[i - 1][j - 1];
                int min_val = ins < del ? ins : del;
                min_val = min_val < rep ? min_val : rep;
                dp[i][j] = min_val + 1;
            }
        }
    }
    return dp[m][n];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1010: Edit distance - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1010: empty output");
    assert!(code.contains("fn dp_edit_distance"), "C1010: Should contain dp_edit_distance");
}

// ============================================================================
// C1011-C1015: Tree/Number DP
// ============================================================================

#[test]
fn c1011_optimal_bst() {
    let c_code = r#"
int dp_optimal_bst(int keys[], int freq[], int n) {
    int cost[50][50];
    int sum[50][50];
    int i, j, L, r;

    for (i = 0; i < n; i++) {
        cost[i][i] = freq[i];
        sum[i][i] = freq[i];
    }

    for (L = 2; L <= n; L++) {
        for (i = 0; i <= n - L; i++) {
            j = i + L - 1;
            cost[i][j] = 2147483647;
            sum[i][j] = sum[i][j - 1] + freq[j];

            for (r = i; r <= j; r++) {
                int left = (r > i) ? cost[i][r - 1] : 0;
                int right = (r < j) ? cost[r + 1][j] : 0;
                int c = left + right + sum[i][j];
                if (c < cost[i][j])
                    cost[i][j] = c;
            }
        }
    }
    return cost[0][n - 1];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1011: Optimal BST - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1011: empty output");
    assert!(code.contains("fn dp_optimal_bst"), "C1011: Should contain dp_optimal_bst");
}

#[test]
fn c1012_catalan_numbers() {
    let c_code = r#"
typedef unsigned long dp_u64;

dp_u64 dp_catalan(int n) {
    dp_u64 cat[100];
    int i, j;

    cat[0] = 1;
    cat[1] = 1;

    for (i = 2; i <= n; i++) {
        cat[i] = 0;
        for (j = 0; j < i; j++) {
            cat[i] += cat[j] * cat[i - 1 - j];
        }
    }
    return cat[n];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1012: Catalan numbers - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1012: empty output");
    assert!(code.contains("fn dp_catalan"), "C1012: Should contain dp_catalan");
}

#[test]
fn c1013_bell_numbers() {
    let c_code = r#"
typedef unsigned long dp_bell_u64;

dp_bell_u64 dp_bell(int n) {
    dp_bell_u64 tri[50][50];
    int i, j;

    tri[0][0] = 1;

    for (i = 1; i <= n; i++) {
        tri[i][0] = tri[i - 1][i - 1];
        for (j = 1; j <= i; j++) {
            tri[i][j] = tri[i - 1][j - 1] + tri[i][j - 1];
        }
    }
    return tri[n][0];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1013: Bell numbers - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1013: empty output");
    assert!(code.contains("fn dp_bell"), "C1013: Should contain dp_bell");
}

#[test]
fn c1014_kadane_max_subarray() {
    let c_code = r#"
int dp_kadane(int arr[], int n) {
    int max_ending = arr[0];
    int max_so_far = arr[0];
    int i;

    for (i = 1; i < n; i++) {
        max_ending = max_ending + arr[i];
        if (arr[i] > max_ending)
            max_ending = arr[i];
        if (max_ending > max_so_far)
            max_so_far = max_ending;
    }
    return max_so_far;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1014: Kadane max subarray - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1014: empty output");
    assert!(code.contains("fn dp_kadane"), "C1014: Should contain dp_kadane");
}

#[test]
fn c1015_max_sum_rectangle() {
    let c_code = r#"
#define DP_MSR_ROWS 100
#define DP_MSR_COLS 100

int dp_kadane_1d(int arr[], int n, int *start_out, int *end_out) {
    int max_sum = -2147483647;
    int cur_sum = 0;
    int local_start = 0;
    int i;

    *start_out = 0;
    *end_out = 0;

    for (i = 0; i < n; i++) {
        cur_sum += arr[i];
        if (cur_sum > max_sum) {
            max_sum = cur_sum;
            *start_out = local_start;
            *end_out = i;
        }
        if (cur_sum < 0) {
            cur_sum = 0;
            local_start = i + 1;
        }
    }
    return max_sum;
}

int dp_max_sum_rectangle(int matrix[][DP_MSR_COLS], int rows, int cols) {
    int temp[DP_MSR_COLS];
    int max_sum = -2147483647;
    int left, right, i;
    int start_row, end_row;

    for (left = 0; left < cols; left++) {
        for (i = 0; i < rows; i++)
            temp[i] = 0;

        for (right = left; right < cols; right++) {
            for (i = 0; i < rows; i++)
                temp[i] += matrix[i][right];

            int sum = dp_kadane_1d(temp, rows, &start_row, &end_row);
            if (sum > max_sum)
                max_sum = sum;
        }
    }
    return max_sum;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1015: Max sum rectangle - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1015: empty output");
    assert!(code.contains("fn dp_max_sum_rectangle"), "C1015: Should contain dp_max_sum_rectangle");
}

// ============================================================================
// C1016-C1020: String/Sequence DP
// ============================================================================

#[test]
fn c1016_lcs_three_strings() {
    let c_code = r#"
int dp_lcs3(const char *a, int la, const char *b, int lb, const char *c, int lc) {
    int dp[51][51][51];
    int i, j, k;

    for (i = 0; i <= la; i++)
        for (j = 0; j <= lb; j++)
            for (k = 0; k <= lc; k++)
                dp[i][j][k] = 0;

    for (i = 1; i <= la; i++) {
        for (j = 1; j <= lb; j++) {
            for (k = 1; k <= lc; k++) {
                if (a[i - 1] == b[j - 1] && b[j - 1] == c[k - 1]) {
                    dp[i][j][k] = dp[i - 1][j - 1][k - 1] + 1;
                } else {
                    int ab = dp[i - 1][j][k];
                    int bc = dp[i][j - 1][k];
                    int ac = dp[i][j][k - 1];
                    int m = ab > bc ? ab : bc;
                    dp[i][j][k] = m > ac ? m : ac;
                }
            }
        }
    }
    return dp[la][lb][lc];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1016: LCS 3 strings - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1016: empty output");
    assert!(code.contains("fn dp_lcs3"), "C1016: Should contain dp_lcs3");
}

#[test]
fn c1017_wildcard_matching() {
    let c_code = r#"
int dp_wildcard_match(const char *s, int sn, const char *p, int pn) {
    int dp[201][201];
    int i, j;

    dp[0][0] = 1;
    for (j = 1; j <= pn; j++)
        dp[0][j] = (p[j - 1] == '*') ? dp[0][j - 1] : 0;
    for (i = 1; i <= sn; i++)
        dp[i][0] = 0;

    for (i = 1; i <= sn; i++) {
        for (j = 1; j <= pn; j++) {
            if (p[j - 1] == '*') {
                dp[i][j] = dp[i - 1][j] || dp[i][j - 1];
            } else if (p[j - 1] == '?' || s[i - 1] == p[j - 1]) {
                dp[i][j] = dp[i - 1][j - 1];
            } else {
                dp[i][j] = 0;
            }
        }
    }
    return dp[sn][pn];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1017: Wildcard matching - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1017: empty output");
    assert!(code.contains("fn dp_wildcard_match"), "C1017: Should contain dp_wildcard_match");
}

#[test]
fn c1018_interleaving_strings() {
    let c_code = r#"
int dp_is_interleave(const char *s1, int m, const char *s2, int n, const char *s3) {
    int dp[201][201];
    int i, j;

    if (m + n == 0)
        return 1;

    dp[0][0] = 1;
    for (i = 1; i <= m; i++)
        dp[i][0] = dp[i - 1][0] && (s1[i - 1] == s3[i - 1]);
    for (j = 1; j <= n; j++)
        dp[0][j] = dp[0][j - 1] && (s2[j - 1] == s3[j - 1]);

    for (i = 1; i <= m; i++) {
        for (j = 1; j <= n; j++) {
            dp[i][j] = (dp[i - 1][j] && s1[i - 1] == s3[i + j - 1]) ||
                        (dp[i][j - 1] && s2[j - 1] == s3[i + j - 1]);
        }
    }
    return dp[m][n];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1018: Interleaving strings - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1018: empty output");
    assert!(code.contains("fn dp_is_interleave"), "C1018: Should contain dp_is_interleave");
}

#[test]
fn c1019_palindrome_partitioning_min_cuts() {
    let c_code = r#"
int dp_min_palindrome_cuts(const char *s, int n) {
    int is_pal[500][500];
    int cuts[500];
    int i, j, L;

    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++)
            is_pal[i][j] = 0;

    for (i = 0; i < n; i++)
        is_pal[i][i] = 1;

    for (L = 2; L <= n; L++) {
        for (i = 0; i < n - L + 1; i++) {
            j = i + L - 1;
            if (L == 2) {
                is_pal[i][j] = (s[i] == s[j]);
            } else {
                is_pal[i][j] = (s[i] == s[j]) && is_pal[i + 1][j - 1];
            }
        }
    }

    for (i = 0; i < n; i++) {
        if (is_pal[0][i]) {
            cuts[i] = 0;
        } else {
            cuts[i] = 2147483647;
            for (j = 0; j < i; j++) {
                if (is_pal[j + 1][i] && cuts[j] + 1 < cuts[i]) {
                    cuts[i] = cuts[j] + 1;
                }
            }
        }
    }
    return cuts[n - 1];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1019: Palindrome partitioning - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1019: empty output");
    assert!(code.contains("fn dp_min_palindrome_cuts"), "C1019: Should contain dp_min_palindrome_cuts");
}

#[test]
fn c1020_word_wrap() {
    let c_code = r#"
#define DP_WW_INF 2147483647

int dp_word_wrap(int words[], int n, int line_width) {
    int extras[100][100];
    int lc[100][100];
    int cost[100];
    int i, j;

    for (i = 1; i <= n; i++) {
        extras[i][i] = line_width - words[i - 1];
        for (j = i + 1; j <= n; j++) {
            extras[i][j] = extras[i][j - 1] - words[j - 1] - 1;
        }
    }

    for (i = 1; i <= n; i++) {
        for (j = i; j <= n; j++) {
            if (extras[i][j] < 0) {
                lc[i][j] = DP_WW_INF;
            } else if (j == n && extras[i][j] >= 0) {
                lc[i][j] = 0;
            } else {
                lc[i][j] = extras[i][j] * extras[i][j];
            }
        }
    }

    cost[0] = 0;
    for (j = 1; j <= n; j++) {
        cost[j] = DP_WW_INF;
        for (i = 1; i <= j; i++) {
            if (cost[i - 1] != DP_WW_INF && lc[i][j] != DP_WW_INF) {
                int val = cost[i - 1] + lc[i][j];
                if (val < cost[j])
                    cost[j] = val;
            }
        }
    }
    return cost[n];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1020: Word wrap - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1020: empty output");
    assert!(code.contains("fn dp_word_wrap"), "C1020: Should contain dp_word_wrap");
}

// ============================================================================
// C1021-C1025: Advanced DP
// ============================================================================

#[test]
fn c1021_tsp_bitmask() {
    let c_code = r#"
#define DP_TSP_MAX 15
#define DP_TSP_INF 2147483647

int dp_tsp(int dist[][DP_TSP_MAX], int n) {
    int dp[1 << DP_TSP_MAX][DP_TSP_MAX];
    int mask, pos, next;
    int full_mask;

    full_mask = (1 << n) - 1;

    for (mask = 0; mask <= full_mask; mask++)
        for (pos = 0; pos < n; pos++)
            dp[mask][pos] = DP_TSP_INF;

    dp[1][0] = 0;

    for (mask = 1; mask <= full_mask; mask++) {
        for (pos = 0; pos < n; pos++) {
            if (dp[mask][pos] == DP_TSP_INF) continue;
            if (!(mask & (1 << pos))) continue;

            for (next = 0; next < n; next++) {
                if (mask & (1 << next)) continue;
                int new_mask = mask | (1 << next);
                int new_cost = dp[mask][pos] + dist[pos][next];
                if (new_cost < dp[new_mask][next])
                    dp[new_mask][next] = new_cost;
            }
        }
    }

    int result = DP_TSP_INF;
    for (pos = 0; pos < n; pos++) {
        if (dp[full_mask][pos] != DP_TSP_INF) {
            int total = dp[full_mask][pos] + dist[pos][0];
            if (total < result)
                result = total;
        }
    }
    return result;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1021: TSP bitmask - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1021: empty output");
    assert!(code.contains("fn dp_tsp"), "C1021: Should contain dp_tsp");
}

#[test]
fn c1022_longest_bitonic_subsequence() {
    let c_code = r#"
int dp_longest_bitonic(int arr[], int n) {
    int lis[1000];
    int lds[1000];
    int i, j;
    int max_len;

    for (i = 0; i < n; i++)
        lis[i] = 1;
    for (i = 1; i < n; i++) {
        for (j = 0; j < i; j++) {
            if (arr[j] < arr[i] && lis[j] + 1 > lis[i])
                lis[i] = lis[j] + 1;
        }
    }

    for (i = 0; i < n; i++)
        lds[i] = 1;
    for (i = n - 2; i >= 0; i--) {
        for (j = n - 1; j > i; j--) {
            if (arr[j] < arr[i] && lds[j] + 1 > lds[i])
                lds[i] = lds[j] + 1;
        }
    }

    max_len = lis[0] + lds[0] - 1;
    for (i = 1; i < n; i++) {
        int cur = lis[i] + lds[i] - 1;
        if (cur > max_len)
            max_len = cur;
    }
    return max_len;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1022: Longest bitonic subseq - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1022: empty output");
    assert!(code.contains("fn dp_longest_bitonic"), "C1022: Should contain dp_longest_bitonic");
}

#[test]
fn c1023_box_stacking() {
    let c_code = r#"
typedef struct {
    int h;
    int w;
    int d;
    int area;
} dp_Box;

void dp_box_swap(dp_Box *a, dp_Box *b) {
    dp_Box temp = *a;
    *a = *b;
    *b = temp;
}

void dp_box_sort(dp_Box boxes[], int n) {
    int i, j;
    for (i = 0; i < n - 1; i++) {
        for (j = 0; j < n - 1 - i; j++) {
            if (boxes[j].area < boxes[j + 1].area) {
                dp_box_swap(&boxes[j], &boxes[j + 1]);
            }
        }
    }
}

int dp_box_stacking(int dims[][3], int n) {
    dp_Box rot[300];
    int count = 0;
    int i, j;
    int msh[300];
    int max_height;

    for (i = 0; i < n; i++) {
        int h = dims[i][0], w = dims[i][1], d = dims[i][2];

        rot[count].h = h;
        rot[count].w = w > d ? w : d;
        rot[count].d = w > d ? d : w;
        rot[count].area = rot[count].w * rot[count].d;
        count++;

        rot[count].h = w;
        rot[count].w = h > d ? h : d;
        rot[count].d = h > d ? d : h;
        rot[count].area = rot[count].w * rot[count].d;
        count++;

        rot[count].h = d;
        rot[count].w = h > w ? h : w;
        rot[count].d = h > w ? w : h;
        rot[count].area = rot[count].w * rot[count].d;
        count++;
    }

    dp_box_sort(rot, count);

    for (i = 0; i < count; i++)
        msh[i] = rot[i].h;

    for (i = 1; i < count; i++) {
        for (j = 0; j < i; j++) {
            if (rot[j].w > rot[i].w && rot[j].d > rot[i].d) {
                if (msh[j] + rot[i].h > msh[i])
                    msh[i] = msh[j] + rot[i].h;
            }
        }
    }

    max_height = msh[0];
    for (i = 1; i < count; i++) {
        if (msh[i] > max_height)
            max_height = msh[i];
    }
    return max_height;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1023: Box stacking - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1023: empty output");
    assert!(code.contains("fn dp_box_stacking"), "C1023: Should contain dp_box_stacking");
}

#[test]
fn c1024_assembly_line_scheduling() {
    let c_code = r#"
#define DP_ALS_STATIONS 100

int dp_assembly_line(int a[][DP_ALS_STATIONS], int t[][DP_ALS_STATIONS],
                     int e[], int x[], int n) {
    int f1[DP_ALS_STATIONS];
    int f2[DP_ALS_STATIONS];
    int i;
    int result;

    f1[0] = e[0] + a[0][0];
    f2[0] = e[1] + a[1][0];

    for (i = 1; i < n; i++) {
        int stay1 = f1[i - 1] + a[0][i];
        int transfer1 = f2[i - 1] + t[1][i] + a[0][i];
        f1[i] = stay1 < transfer1 ? stay1 : transfer1;

        int stay2 = f2[i - 1] + a[1][i];
        int transfer2 = f1[i - 1] + t[0][i] + a[1][i];
        f2[i] = stay2 < transfer2 ? stay2 : transfer2;
    }

    int total1 = f1[n - 1] + x[0];
    int total2 = f2[n - 1] + x[1];
    result = total1 < total2 ? total1 : total2;
    return result;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1024: Assembly line scheduling - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1024: empty output");
    assert!(code.contains("fn dp_assembly_line"), "C1024: Should contain dp_assembly_line");
}

#[test]
fn c1025_minimum_cost_path() {
    let c_code = r#"
#define DP_MCP_SIZE 100

int dp_min_cost_path(int grid[][DP_MCP_SIZE], int m, int n) {
    int cost[DP_MCP_SIZE][DP_MCP_SIZE];
    int i, j;

    cost[0][0] = grid[0][0];

    for (i = 1; i < m; i++)
        cost[i][0] = cost[i - 1][0] + grid[i][0];

    for (j = 1; j < n; j++)
        cost[0][j] = cost[0][j - 1] + grid[0][j];

    for (i = 1; i < m; i++) {
        for (j = 1; j < n; j++) {
            int from_left = cost[i][j - 1];
            int from_above = cost[i - 1][j];
            int from_diag = cost[i - 1][j - 1];
            int min_prev = from_left < from_above ? from_left : from_above;
            min_prev = min_prev < from_diag ? min_prev : from_diag;
            cost[i][j] = min_prev + grid[i][j];
        }
    }
    return cost[m - 1][n - 1];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1025: Min cost path - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1025: empty output");
    assert!(code.contains("fn dp_min_cost_path"), "C1025: Should contain dp_min_cost_path");
}
