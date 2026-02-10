//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C901-C925: Sorting Algorithm implementations -- the kind of C code found
//! in textbooks (CLRS, Sedgewick), competitive programming, and systems software.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise classic and advanced sorting patterns commonly
//! found in real-world C codebases -- all expressed as valid C99 without #include.
//!
//! Organization:
//! - C901-C905: Comparison-based classics (quicksort, mergesort, heapsort, insertion, selection)
//! - C906-C910: Gap/distribution sorts (bubble, shell, counting, radix, bucket)
//! - C911-C915: Hybrid/tree sorts (timsort, introsort, treesort, patience, cocktail)
//! - C916-C920: Novelty sorts (comb, gnome, cycle, bitonic, pancake)
//! - C921-C925: Exotic sorts (stooge, bogosort, flashsort, blocksort, strandsort)

use decy_core::transpile;

// ============================================================================
// C901-C905: Comparison-Based Classics
// ============================================================================

#[test]
fn c901_quicksort_lomuto() {
    let c_code = r#"
void sort_quicksort_swap(int *a, int *b) {
    int tmp = *a;
    *a = *b;
    *b = tmp;
}

int sort_quicksort_partition(int *arr, int low, int high) {
    int pivot = arr[high];
    int i = low - 1;
    int j;
    for (j = low; j < high; j++) {
        if (arr[j] <= pivot) {
            i++;
            sort_quicksort_swap(&arr[i], &arr[j]);
        }
    }
    sort_quicksort_swap(&arr[i + 1], &arr[high]);
    return i + 1;
}

void sort_quicksort(int *arr, int low, int high) {
    if (low < high) {
        int pi = sort_quicksort_partition(arr, low, high);
        sort_quicksort(arr, low, pi - 1);
        sort_quicksort(arr, pi + 1, high);
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C901 quicksort failed: {:?}", result.err());
}

#[test]
fn c902_mergesort_topdown() {
    let c_code = r#"
void sort_merge(int *arr, int *tmp, int left, int mid, int right) {
    int i = left;
    int j = mid + 1;
    int k = left;
    while (i <= mid && j <= right) {
        if (arr[i] <= arr[j]) {
            tmp[k++] = arr[i++];
        } else {
            tmp[k++] = arr[j++];
        }
    }
    while (i <= mid) {
        tmp[k++] = arr[i++];
    }
    while (j <= right) {
        tmp[k++] = arr[j++];
    }
    for (i = left; i <= right; i++) {
        arr[i] = tmp[i];
    }
}

void sort_mergesort(int *arr, int *tmp, int left, int right) {
    if (left < right) {
        int mid = left + (right - left) / 2;
        sort_mergesort(arr, tmp, left, mid);
        sort_mergesort(arr, tmp, mid + 1, right);
        sort_merge(arr, tmp, left, mid, right);
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C902 mergesort failed: {:?}", result.err());
}

#[test]
fn c903_heapsort_maxheap() {
    let c_code = r#"
void sort_heapsort_swap(int *a, int *b) {
    int t = *a;
    *a = *b;
    *b = t;
}

void sort_heapify(int *arr, int n, int i) {
    int largest = i;
    int left = 2 * i + 1;
    int right = 2 * i + 2;
    if (left < n && arr[left] > arr[largest])
        largest = left;
    if (right < n && arr[right] > arr[largest])
        largest = right;
    if (largest != i) {
        sort_heapsort_swap(&arr[i], &arr[largest]);
        sort_heapify(arr, n, largest);
    }
}

void sort_heapsort(int *arr, int n) {
    int i;
    for (i = n / 2 - 1; i >= 0; i--)
        sort_heapify(arr, n, i);
    for (i = n - 1; i > 0; i--) {
        sort_heapsort_swap(&arr[0], &arr[i]);
        sort_heapify(arr, i, 0);
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C903 heapsort failed: {:?}", result.err());
}

#[test]
fn c904_insertion_sort() {
    let c_code = r#"
void sort_insertion(int *arr, int n) {
    int i, j, key;
    for (i = 1; i < n; i++) {
        key = arr[i];
        j = i - 1;
        while (j >= 0 && arr[j] > key) {
            arr[j + 1] = arr[j];
            j--;
        }
        arr[j + 1] = key;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C904 insertion sort failed: {:?}", result.err());
}

#[test]
fn c905_selection_sort() {
    let c_code = r#"
void sort_selection(int *arr, int n) {
    int i, j, min_idx, tmp;
    for (i = 0; i < n - 1; i++) {
        min_idx = i;
        for (j = i + 1; j < n; j++) {
            if (arr[j] < arr[min_idx])
                min_idx = j;
        }
        if (min_idx != i) {
            tmp = arr[i];
            arr[i] = arr[min_idx];
            arr[min_idx] = tmp;
        }
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C905 selection sort failed: {:?}", result.err());
}

// ============================================================================
// C906-C910: Gap/Distribution Sorts
// ============================================================================

#[test]
fn c906_bubble_sort_optimized() {
    let c_code = r#"
void sort_bubble(int *arr, int n) {
    int i, j, tmp, swapped;
    for (i = 0; i < n - 1; i++) {
        swapped = 0;
        for (j = 0; j < n - i - 1; j++) {
            if (arr[j] > arr[j + 1]) {
                tmp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = tmp;
                swapped = 1;
            }
        }
        if (!swapped) break;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C906 bubble sort failed: {:?}", result.err());
}

#[test]
fn c907_shell_sort_knuth() {
    let c_code = r#"
void sort_shell(int *arr, int n) {
    int gap = 1;
    int i, j, tmp;
    while (gap < n / 3)
        gap = gap * 3 + 1;
    while (gap >= 1) {
        for (i = gap; i < n; i++) {
            tmp = arr[i];
            j = i;
            while (j >= gap && arr[j - gap] > tmp) {
                arr[j] = arr[j - gap];
                j -= gap;
            }
            arr[j] = tmp;
        }
        gap /= 3;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C907 shell sort failed: {:?}", result.err());
}

#[test]
fn c908_counting_sort() {
    let c_code = r#"
void sort_counting(int *arr, int n, int max_val) {
    int count[1024];
    int output[1024];
    int i;
    for (i = 0; i <= max_val; i++)
        count[i] = 0;
    for (i = 0; i < n; i++)
        count[arr[i]]++;
    for (i = 1; i <= max_val; i++)
        count[i] += count[i - 1];
    for (i = n - 1; i >= 0; i--) {
        output[count[arr[i]] - 1] = arr[i];
        count[arr[i]]--;
    }
    for (i = 0; i < n; i++)
        arr[i] = output[i];
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C908 counting sort failed: {:?}", result.err());
}

#[test]
fn c909_radix_sort_lsd() {
    let c_code = r#"
int sort_radix_get_max(int *arr, int n) {
    int max = arr[0];
    int i;
    for (i = 1; i < n; i++) {
        if (arr[i] > max)
            max = arr[i];
    }
    return max;
}

void sort_radix_count_sort(int *arr, int n, int exp) {
    int output[1024];
    int count[10];
    int i;
    for (i = 0; i < 10; i++)
        count[i] = 0;
    for (i = 0; i < n; i++)
        count[(arr[i] / exp) % 10]++;
    for (i = 1; i < 10; i++)
        count[i] += count[i - 1];
    for (i = n - 1; i >= 0; i--) {
        output[count[(arr[i] / exp) % 10] - 1] = arr[i];
        count[(arr[i] / exp) % 10]--;
    }
    for (i = 0; i < n; i++)
        arr[i] = output[i];
}

void sort_radix(int *arr, int n) {
    int max = sort_radix_get_max(arr, n);
    int exp;
    for (exp = 1; max / exp > 0; exp *= 10)
        sort_radix_count_sort(arr, n, exp);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C909 radix sort failed: {:?}", result.err());
}

#[test]
fn c910_bucket_sort() {
    let c_code = r#"
void sort_bucket_insert(int *bucket, int *bsize, int val) {
    int i = *bsize - 1;
    bucket[*bsize] = val;
    while (i >= 0 && bucket[i] > val) {
        bucket[i + 1] = bucket[i];
        i--;
    }
    bucket[i + 1] = val;
    (*bsize)++;
}

void sort_bucket(int *arr, int n, int max_val) {
    int buckets[10][128];
    int bsizes[10];
    int i, j, idx;
    int num_buckets = 10;
    for (i = 0; i < num_buckets; i++)
        bsizes[i] = 0;
    for (i = 0; i < n; i++) {
        idx = arr[i] * num_buckets / (max_val + 1);
        if (idx >= num_buckets) idx = num_buckets - 1;
        sort_bucket_insert(buckets[idx], &bsizes[idx], arr[i]);
    }
    idx = 0;
    for (i = 0; i < num_buckets; i++) {
        for (j = 0; j < bsizes[i]; j++) {
            arr[idx++] = buckets[i][j];
        }
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C910 bucket sort failed: {:?}", result.err());
}

// ============================================================================
// C911-C915: Hybrid/Tree Sorts
// ============================================================================

#[test]
fn c911_timsort_simplified() {
    let c_code = r#"
void sort_tim_insertion(int *arr, int left, int right) {
    int i, j, key;
    for (i = left + 1; i <= right; i++) {
        key = arr[i];
        j = i - 1;
        while (j >= left && arr[j] > key) {
            arr[j + 1] = arr[j];
            j--;
        }
        arr[j + 1] = key;
    }
}

void sort_tim_merge(int *arr, int l, int m, int r) {
    int left_arr[512];
    int right_arr[512];
    int len1 = m - l + 1;
    int len2 = r - m;
    int i, j, k;
    for (i = 0; i < len1; i++)
        left_arr[i] = arr[l + i];
    for (j = 0; j < len2; j++)
        right_arr[j] = arr[m + 1 + j];
    i = 0;
    j = 0;
    k = l;
    while (i < len1 && j < len2) {
        if (left_arr[i] <= right_arr[j])
            arr[k++] = left_arr[i++];
        else
            arr[k++] = right_arr[j++];
    }
    while (i < len1)
        arr[k++] = left_arr[i++];
    while (j < len2)
        arr[k++] = right_arr[j++];
}

void sort_timsort(int *arr, int n) {
    int run = 32;
    int i, size, left, mid, right;
    for (i = 0; i < n; i += run) {
        int end = i + run - 1;
        if (end >= n) end = n - 1;
        sort_tim_insertion(arr, i, end);
    }
    for (size = run; size < n; size = 2 * size) {
        for (left = 0; left < n; left += 2 * size) {
            mid = left + size - 1;
            right = left + 2 * size - 1;
            if (mid >= n) mid = n - 1;
            if (right >= n) right = n - 1;
            if (mid < right)
                sort_tim_merge(arr, left, mid, right);
        }
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C911 timsort failed: {:?}", result.err());
}

#[test]
fn c912_introsort() {
    let c_code = r#"
void sort_intro_swap(int *a, int *b) {
    int t = *a;
    *a = *b;
    *b = t;
}

void sort_intro_sift_down(int *arr, int start, int end) {
    int root = start;
    while (2 * root + 1 <= end) {
        int child = 2 * root + 1;
        int swap_idx = root;
        if (arr[swap_idx] < arr[child])
            swap_idx = child;
        if (child + 1 <= end && arr[swap_idx] < arr[child + 1])
            swap_idx = child + 1;
        if (swap_idx == root) return;
        sort_intro_swap(&arr[root], &arr[swap_idx]);
        root = swap_idx;
    }
}

void sort_intro_heapsort(int *arr, int n) {
    int i;
    for (i = (n - 2) / 2; i >= 0; i--)
        sort_intro_sift_down(arr, i, n - 1);
    for (i = n - 1; i > 0; i--) {
        sort_intro_swap(&arr[0], &arr[i]);
        sort_intro_sift_down(arr, 0, i - 1);
    }
}

void sort_intro_insertion(int *arr, int n) {
    int i, j, key;
    for (i = 1; i < n; i++) {
        key = arr[i];
        j = i - 1;
        while (j >= 0 && arr[j] > key) {
            arr[j + 1] = arr[j];
            j--;
        }
        arr[j + 1] = key;
    }
}

int sort_intro_partition(int *arr, int low, int high) {
    int pivot = arr[high];
    int i = low - 1;
    int j;
    for (j = low; j < high; j++) {
        if (arr[j] <= pivot) {
            i++;
            sort_intro_swap(&arr[i], &arr[j]);
        }
    }
    sort_intro_swap(&arr[i + 1], &arr[high]);
    return i + 1;
}

int sort_intro_log2(int n) {
    int log = 0;
    while (n > 1) {
        n /= 2;
        log++;
    }
    return log;
}

void sort_introsort_impl(int *arr, int low, int high, int depth_limit) {
    int size = high - low + 1;
    if (size <= 16) {
        sort_intro_insertion(arr + low, size);
        return;
    }
    if (depth_limit == 0) {
        sort_intro_heapsort(arr + low, size);
        return;
    }
    {
        int pi = sort_intro_partition(arr, low, high);
        sort_introsort_impl(arr, low, pi - 1, depth_limit - 1);
        sort_introsort_impl(arr, pi + 1, high, depth_limit - 1);
    }
}

void sort_introsort(int *arr, int n) {
    int depth_limit = 2 * sort_intro_log2(n);
    sort_introsort_impl(arr, 0, n - 1, depth_limit);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C912 introsort failed: {:?}", result.err());
}

#[test]
fn c913_tree_sort_bst() {
    let c_code = r#"
typedef struct {
    int key;
    int left;
    int right;
} sort_tree_node_t;

typedef struct {
    sort_tree_node_t nodes[1024];
    int count;
    int root;
} sort_tree_bst_t;

void sort_tree_init(sort_tree_bst_t *tree) {
    tree->count = 0;
    tree->root = -1;
}

int sort_tree_new_node(sort_tree_bst_t *tree, int key) {
    int idx = tree->count++;
    tree->nodes[idx].key = key;
    tree->nodes[idx].left = -1;
    tree->nodes[idx].right = -1;
    return idx;
}

int sort_tree_insert(sort_tree_bst_t *tree, int root, int key) {
    if (root == -1)
        return sort_tree_new_node(tree, key);
    if (key < tree->nodes[root].key)
        tree->nodes[root].left = sort_tree_insert(tree, tree->nodes[root].left, key);
    else
        tree->nodes[root].right = sort_tree_insert(tree, tree->nodes[root].right, key);
    return root;
}

void sort_tree_inorder(sort_tree_bst_t *tree, int root, int *arr, int *idx) {
    if (root == -1) return;
    sort_tree_inorder(tree, tree->nodes[root].left, arr, idx);
    arr[*idx] = tree->nodes[root].key;
    (*idx)++;
    sort_tree_inorder(tree, tree->nodes[root].right, arr, idx);
}

void sort_treesort(int *arr, int n) {
    sort_tree_bst_t tree;
    int i, idx;
    sort_tree_init(&tree);
    for (i = 0; i < n; i++)
        tree.root = sort_tree_insert(&tree, tree.root, arr[i]);
    idx = 0;
    sort_tree_inorder(&tree, tree.root, arr, &idx);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C913 tree sort failed: {:?}", result.err());
}

#[test]
fn c914_patience_sort() {
    let c_code = r#"
typedef struct {
    int cards[1024];
    int top;
} sort_patience_pile_t;

void sort_patience_pile_init(sort_patience_pile_t *pile) {
    pile->top = -1;
}

void sort_patience_pile_push(sort_patience_pile_t *pile, int val) {
    pile->top++;
    pile->cards[pile->top] = val;
}

int sort_patience_pile_peek(sort_patience_pile_t *pile) {
    return pile->cards[pile->top];
}

int sort_patience_pile_pop(sort_patience_pile_t *pile) {
    return pile->cards[pile->top--];
}

void sort_patience(int *arr, int n) {
    sort_patience_pile_t piles[256];
    int num_piles = 0;
    int i, j, min_idx, min_val;

    for (i = 0; i < n; i++) {
        int placed = 0;
        for (j = 0; j < num_piles; j++) {
            if (sort_patience_pile_peek(&piles[j]) >= arr[i]) {
                sort_patience_pile_push(&piles[j], arr[i]);
                placed = 1;
                break;
            }
        }
        if (!placed) {
            sort_patience_pile_init(&piles[num_piles]);
            sort_patience_pile_push(&piles[num_piles], arr[i]);
            num_piles++;
        }
    }

    for (i = 0; i < n; i++) {
        min_idx = -1;
        min_val = 0;
        for (j = 0; j < num_piles; j++) {
            if (piles[j].top >= 0) {
                if (min_idx == -1 || sort_patience_pile_peek(&piles[j]) < min_val) {
                    min_idx = j;
                    min_val = sort_patience_pile_peek(&piles[j]);
                }
            }
        }
        arr[i] = sort_patience_pile_pop(&piles[min_idx]);
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C914 patience sort failed: {:?}", result.err());
}

#[test]
fn c915_cocktail_shaker_sort() {
    let c_code = r#"
void sort_cocktail(int *arr, int n) {
    int swapped = 1;
    int start = 0;
    int end = n - 1;
    int i, tmp;
    while (swapped) {
        swapped = 0;
        for (i = start; i < end; i++) {
            if (arr[i] > arr[i + 1]) {
                tmp = arr[i];
                arr[i] = arr[i + 1];
                arr[i + 1] = tmp;
                swapped = 1;
            }
        }
        if (!swapped) break;
        swapped = 0;
        end--;
        for (i = end - 1; i >= start; i--) {
            if (arr[i] > arr[i + 1]) {
                tmp = arr[i];
                arr[i] = arr[i + 1];
                arr[i + 1] = tmp;
                swapped = 1;
            }
        }
        start++;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C915 cocktail shaker sort failed: {:?}", result.err());
}

// ============================================================================
// C916-C920: Novelty Sorts
// ============================================================================

#[test]
fn c916_comb_sort() {
    let c_code = r#"
int sort_comb_next_gap(int gap) {
    gap = (gap * 10) / 13;
    if (gap < 1) return 1;
    return gap;
}

void sort_comb(int *arr, int n) {
    int gap = n;
    int swapped = 1;
    int i, tmp;
    while (gap != 1 || swapped) {
        gap = sort_comb_next_gap(gap);
        swapped = 0;
        for (i = 0; i < n - gap; i++) {
            if (arr[i] > arr[i + gap]) {
                tmp = arr[i];
                arr[i] = arr[i + gap];
                arr[i + gap] = tmp;
                swapped = 1;
            }
        }
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C916 comb sort failed: {:?}", result.err());
}

#[test]
fn c917_gnome_sort() {
    let c_code = r#"
void sort_gnome(int *arr, int n) {
    int index = 0;
    int tmp;
    while (index < n) {
        if (index == 0 || arr[index] >= arr[index - 1]) {
            index++;
        } else {
            tmp = arr[index];
            arr[index] = arr[index - 1];
            arr[index - 1] = tmp;
            index--;
        }
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C917 gnome sort failed: {:?}", result.err());
}

#[test]
fn c918_cycle_sort() {
    let c_code = r#"
void sort_cycle(int *arr, int n) {
    int cycle_start, item, pos, tmp;
    int i;
    for (cycle_start = 0; cycle_start < n - 1; cycle_start++) {
        item = arr[cycle_start];
        pos = cycle_start;
        for (i = cycle_start + 1; i < n; i++) {
            if (arr[i] < item)
                pos++;
        }
        if (pos == cycle_start)
            continue;
        while (item == arr[pos])
            pos++;
        if (pos != cycle_start) {
            tmp = arr[pos];
            arr[pos] = item;
            item = tmp;
        }
        while (pos != cycle_start) {
            pos = cycle_start;
            for (i = cycle_start + 1; i < n; i++) {
                if (arr[i] < item)
                    pos++;
            }
            while (item == arr[pos])
                pos++;
            if (item != arr[pos]) {
                tmp = arr[pos];
                arr[pos] = item;
                item = tmp;
            }
        }
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C918 cycle sort failed: {:?}", result.err());
}

#[test]
fn c919_bitonic_sort() {
    let c_code = r#"
void sort_bitonic_swap(int *arr, int i, int j, int dir) {
    int tmp;
    if ((arr[i] > arr[j] && dir == 1) || (arr[i] < arr[j] && dir == 0)) {
        tmp = arr[i];
        arr[i] = arr[j];
        arr[j] = tmp;
    }
}

void sort_bitonic_merge(int *arr, int low, int cnt, int dir) {
    int k, i;
    if (cnt > 1) {
        k = cnt / 2;
        for (i = low; i < low + k; i++)
            sort_bitonic_swap(arr, i, i + k, dir);
        sort_bitonic_merge(arr, low, k, dir);
        sort_bitonic_merge(arr, low + k, k, dir);
    }
}

void sort_bitonic_impl(int *arr, int low, int cnt, int dir) {
    int k;
    if (cnt > 1) {
        k = cnt / 2;
        sort_bitonic_impl(arr, low, k, 1);
        sort_bitonic_impl(arr, low + k, k, 0);
        sort_bitonic_merge(arr, low, cnt, dir);
    }
}

void sort_bitonic(int *arr, int n) {
    sort_bitonic_impl(arr, 0, n, 1);
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C919 bitonic sort failed: {:?}", result.err());
}

#[test]
fn c920_pancake_sort() {
    let c_code = r#"
void sort_pancake_flip(int *arr, int k) {
    int i = 0;
    int j = k;
    int tmp;
    while (i < j) {
        tmp = arr[i];
        arr[i] = arr[j];
        arr[j] = tmp;
        i++;
        j--;
    }
}

int sort_pancake_find_max(int *arr, int n) {
    int max_idx = 0;
    int i;
    for (i = 1; i <= n; i++) {
        if (arr[i] > arr[max_idx])
            max_idx = i;
    }
    return max_idx;
}

void sort_pancake(int *arr, int n) {
    int curr_size, mi;
    for (curr_size = n - 1; curr_size > 0; curr_size--) {
        mi = sort_pancake_find_max(arr, curr_size);
        if (mi != curr_size) {
            if (mi != 0)
                sort_pancake_flip(arr, mi);
            sort_pancake_flip(arr, curr_size);
        }
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C920 pancake sort failed: {:?}", result.err());
}

// ============================================================================
// C921-C925: Exotic Sorts
// ============================================================================

#[test]
fn c921_stooge_sort() {
    let c_code = r#"
void sort_stooge(int *arr, int l, int h) {
    int tmp, t;
    if (arr[l] > arr[h]) {
        tmp = arr[l];
        arr[l] = arr[h];
        arr[h] = tmp;
    }
    if (h - l + 1 > 2) {
        t = (h - l + 1) / 3;
        sort_stooge(arr, l, h - t);
        sort_stooge(arr, l + t, h);
        sort_stooge(arr, l, h - t);
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C921 stooge sort failed: {:?}", result.err());
}

#[test]
fn c922_bogosort_bounded() {
    let c_code = r#"
int sort_bogo_is_sorted(int *arr, int n) {
    int i;
    for (i = 0; i < n - 1; i++) {
        if (arr[i] > arr[i + 1])
            return 0;
    }
    return 1;
}

unsigned int sort_bogo_rand_state = 42;

unsigned int sort_bogo_rand(void) {
    sort_bogo_rand_state = sort_bogo_rand_state * 1103515245 + 12345;
    return (sort_bogo_rand_state >> 16) & 0x7fff;
}

void sort_bogo_shuffle(int *arr, int n) {
    int i, j, tmp;
    for (i = n - 1; i > 0; i--) {
        j = sort_bogo_rand() % (i + 1);
        tmp = arr[i];
        arr[i] = arr[j];
        arr[j] = tmp;
    }
}

void sort_bogosort(int *arr, int n) {
    int max_iterations = 10000;
    int iter = 0;
    while (!sort_bogo_is_sorted(arr, n) && iter < max_iterations) {
        sort_bogo_shuffle(arr, n);
        iter++;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C922 bogosort failed: {:?}", result.err());
}

#[test]
fn c923_flash_sort() {
    let c_code = r#"
void sort_flash(int *arr, int n) {
    int class_arr[256];
    int m = (int)(0.45 * n);
    int i, j, k, t, nmove;
    int min_val, max_val, max_idx;
    double c1;

    if (m <= 0) m = 1;
    if (n <= 1) return;

    min_val = arr[0];
    max_val = arr[0];
    max_idx = 0;
    for (i = 1; i < n; i++) {
        if (arr[i] < min_val) min_val = arr[i];
        if (arr[i] > max_val) {
            max_val = arr[i];
            max_idx = i;
        }
    }
    if (max_val == min_val) return;

    c1 = (double)(m - 1) / (double)(max_val - min_val);
    for (i = 0; i < m; i++)
        class_arr[i] = 0;
    for (i = 0; i < n; i++) {
        k = (int)(c1 * (arr[i] - min_val));
        class_arr[k]++;
    }
    for (i = 1; i < m; i++)
        class_arr[i] += class_arr[i - 1];

    t = arr[max_idx];
    arr[max_idx] = arr[0];
    arr[0] = t;

    nmove = 0;
    j = 0;
    k = m - 1;
    while (nmove < n - 1) {
        while (j >= class_arr[k]) {
            j++;
            k = (int)(c1 * (arr[j] - min_val));
        }
        t = arr[j];
        while (j != class_arr[k]) {
            k = (int)(c1 * (t - min_val));
            class_arr[k]--;
            {
                int hold = arr[class_arr[k]];
                arr[class_arr[k]] = t;
                t = hold;
            }
            nmove++;
        }
    }

    for (i = 1; i < n; i++) {
        t = arr[i];
        j = i - 1;
        while (j >= 0 && arr[j] > t) {
            arr[j + 1] = arr[j];
            j--;
        }
        arr[j + 1] = t;
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C923 flash sort failed: {:?}", result.err());
}

#[test]
fn c924_block_sort_simplified() {
    let c_code = r#"
void sort_block_insertion(int *arr, int left, int right) {
    int i, j, key;
    for (i = left + 1; i <= right; i++) {
        key = arr[i];
        j = i - 1;
        while (j >= left && arr[j] > key) {
            arr[j + 1] = arr[j];
            j--;
        }
        arr[j + 1] = key;
    }
}

void sort_block_merge(int *arr, int *buf, int left, int mid, int right) {
    int i, j, k;
    int len1 = mid - left + 1;
    for (i = 0; i < len1; i++)
        buf[i] = arr[left + i];
    i = 0;
    j = mid + 1;
    k = left;
    while (i < len1 && j <= right) {
        if (buf[i] <= arr[j])
            arr[k++] = buf[i++];
        else
            arr[k++] = arr[j++];
    }
    while (i < len1)
        arr[k++] = buf[i++];
}

void sort_blocksort(int *arr, int n) {
    int block_size = 16;
    int buffer[1024];
    int i, left, mid, right, size;
    for (i = 0; i < n; i += block_size) {
        int end = i + block_size - 1;
        if (end >= n) end = n - 1;
        sort_block_insertion(arr, i, end);
    }
    for (size = block_size; size < n; size *= 2) {
        for (left = 0; left < n - size; left += 2 * size) {
            mid = left + size - 1;
            right = left + 2 * size - 1;
            if (right >= n) right = n - 1;
            sort_block_merge(arr, buffer, left, mid, right);
        }
    }
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C924 block sort failed: {:?}", result.err());
}

#[test]
fn c925_strand_sort() {
    let c_code = r#"
typedef struct {
    int data[1024];
    int size;
} sort_strand_list_t;

void sort_strand_list_init(sort_strand_list_t *list) {
    list->size = 0;
}

void sort_strand_list_append(sort_strand_list_t *list, int val) {
    list->data[list->size++] = val;
}

int sort_strand_list_remove_at(sort_strand_list_t *list, int idx) {
    int val = list->data[idx];
    int i;
    for (i = idx; i < list->size - 1; i++)
        list->data[i] = list->data[i + 1];
    list->size--;
    return val;
}

void sort_strand_merge_lists(sort_strand_list_t *out, sort_strand_list_t *sub) {
    sort_strand_list_t merged;
    int i = 0, j = 0, k = 0;
    sort_strand_list_init(&merged);
    while (i < out->size && j < sub->size) {
        if (out->data[i] <= sub->data[j])
            merged.data[k++] = out->data[i++];
        else
            merged.data[k++] = sub->data[j++];
    }
    while (i < out->size)
        merged.data[k++] = out->data[i++];
    while (j < sub->size)
        merged.data[k++] = sub->data[j++];
    merged.size = k;
    *out = merged;
}

void sort_strand(int *arr, int n) {
    sort_strand_list_t input;
    sort_strand_list_t output;
    sort_strand_list_t sublist;
    int i;

    sort_strand_list_init(&input);
    sort_strand_list_init(&output);

    for (i = 0; i < n; i++)
        sort_strand_list_append(&input, arr[i]);

    while (input.size > 0) {
        sort_strand_list_init(&sublist);
        sort_strand_list_append(&sublist, sort_strand_list_remove_at(&input, 0));
        i = 0;
        while (i < input.size) {
            if (input.data[i] >= sublist.data[sublist.size - 1]) {
                sort_strand_list_append(&sublist, sort_strand_list_remove_at(&input, i));
            } else {
                i++;
            }
        }
        sort_strand_merge_lists(&output, &sublist);
    }

    for (i = 0; i < n; i++)
        arr[i] = output.data[i];
}
"#;
    let result = transpile(c_code);
    assert!(result.is_ok(), "C925 strand sort failed: {:?}", result.err());
}
