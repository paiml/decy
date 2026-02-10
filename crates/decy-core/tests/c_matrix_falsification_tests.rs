//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1776-C1800: Matrix and Tensor Operations -- basic matrix ops,
//! decompositions, sparse matrices, vector operations, and tensor computations.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world matrix/tensor patterns commonly
//! found in BLAS, LAPACK, Eigen, NumPy internals, and scientific
//! computing libraries -- all expressed as valid C99.
//!
//! Organization:
//! - C1776-C1780: Basic matrix ops (add, multiply, transpose, identity, scalar multiply)
//! - C1781-C1785: Matrix decomposition (LU, forward sub, back sub, determinant, inverse)
//! - C1786-C1790: Sparse matrices (COO, CSR, sparse-dense multiply, sparse add, sparse transpose)
//! - C1791-C1795: Vector operations (dot product, cross product, normalize, Gram-Schmidt, projection)
//! - C1796-C1800: Tensor operations (3D indexing, reshape, slice, batch matmul, element-wise)

// ============================================================================
// C1776-C1780: Basic Matrix Operations
// ============================================================================

/// C1776: Element-wise matrix addition with flat array storage
#[test]
fn c1776_matrix_add() {
    let c_code = r##"
#define MAT_ADD_N 4

typedef struct {
    double data[MAT_ADD_N * MAT_ADD_N];
    int rows;
    int cols;
} mat_add_matrix_t;

void mat_add_init(mat_add_matrix_t *m, int r, int c) {
    int i;
    m->rows = r;
    m->cols = c;
    for (i = 0; i < r * c; i++) {
        m->data[i] = 0.0;
    }
}

void mat_add(const mat_add_matrix_t *A, const mat_add_matrix_t *B, mat_add_matrix_t *C) {
    int i, j;
    C->rows = A->rows;
    C->cols = A->cols;
    for (i = 0; i < A->rows; i++) {
        for (j = 0; j < A->cols; j++) {
            C->data[i * A->cols + j] = A->data[i * A->cols + j] + B->data[i * A->cols + j];
        }
    }
}

void mat_add_inplace(mat_add_matrix_t *A, const mat_add_matrix_t *B) {
    int i;
    for (i = 0; i < A->rows * A->cols; i++) {
        A->data[i] += B->data[i];
    }
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1776: Matrix add should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1776: Output should not be empty");
    assert!(
        code.contains("fn mat_add"),
        "C1776: Should contain mat_add function"
    );
}

/// C1777: NxN matrix multiplication with triple nested loops
#[test]
fn c1777_matrix_multiply() {
    let c_code = r##"
#define MAT_MUL_N 4

typedef struct {
    double data[MAT_MUL_N * MAT_MUL_N];
    int rows;
    int cols;
} mat_mul_matrix_t;

void mat_mul_zero(mat_mul_matrix_t *m, int r, int c) {
    int i;
    m->rows = r;
    m->cols = c;
    for (i = 0; i < r * c; i++) {
        m->data[i] = 0.0;
    }
}

void mat_mul(const mat_mul_matrix_t *A, const mat_mul_matrix_t *B, mat_mul_matrix_t *C) {
    int i, j, k;
    mat_mul_zero(C, A->rows, B->cols);
    for (i = 0; i < A->rows; i++) {
        for (j = 0; j < B->cols; j++) {
            double sum = 0.0;
            for (k = 0; k < A->cols; k++) {
                sum += A->data[i * A->cols + k] * B->data[k * B->cols + j];
            }
            C->data[i * C->cols + j] = sum;
        }
    }
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1777: Matrix multiply should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1777: Output should not be empty");
    assert!(
        code.contains("fn mat_mul"),
        "C1777: Should contain mat_mul function"
    );
}

/// C1778: Matrix transpose for rectangular matrices
#[test]
fn c1778_matrix_transpose() {
    let c_code = r##"
#define MAT_TRANS_N 4

typedef struct {
    double data[MAT_TRANS_N * MAT_TRANS_N];
    int rows;
    int cols;
} mat_trans_matrix_t;

void mat_transpose(const mat_trans_matrix_t *src, mat_trans_matrix_t *dst) {
    int i, j;
    dst->rows = src->cols;
    dst->cols = src->rows;
    for (i = 0; i < src->rows; i++) {
        for (j = 0; j < src->cols; j++) {
            dst->data[j * src->rows + i] = src->data[i * src->cols + j];
        }
    }
}

int mat_transpose_is_symmetric(const mat_trans_matrix_t *m) {
    int i, j;
    if (m->rows != m->cols) return 0;
    for (i = 0; i < m->rows; i++) {
        for (j = i + 1; j < m->cols; j++) {
            double diff = m->data[i * m->cols + j] - m->data[j * m->cols + i];
            if (diff > 1e-9 || diff < -1e-9) return 0;
        }
    }
    return 1;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1778: Matrix transpose should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1778: Output should not be empty");
    assert!(
        code.contains("fn mat_transpose"),
        "C1778: Should contain mat_transpose function"
    );
}

/// C1779: Identity matrix generation
#[test]
fn c1779_identity_matrix() {
    let c_code = r##"
#define MAT_ID_N 4

typedef struct {
    double data[MAT_ID_N * MAT_ID_N];
    int size;
} mat_identity_t;

void mat_identity_gen(mat_identity_t *m, int n) {
    int i, j;
    m->size = n;
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            m->data[i * n + j] = (i == j) ? 1.0 : 0.0;
        }
    }
}

int mat_identity_check(const mat_identity_t *m) {
    int i, j;
    for (i = 0; i < m->size; i++) {
        for (j = 0; j < m->size; j++) {
            double expected = (i == j) ? 1.0 : 0.0;
            double diff = m->data[i * m->size + j] - expected;
            if (diff > 1e-12 || diff < -1e-12) return 0;
        }
    }
    return 1;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1779: Identity matrix should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1779: Output should not be empty");
    assert!(
        code.contains("fn mat_identity_gen"),
        "C1779: Should contain mat_identity_gen function"
    );
}

/// C1780: Scalar multiplication of matrix
#[test]
fn c1780_scalar_multiply() {
    let c_code = r##"
#define MAT_SCAL_N 4

typedef struct {
    double data[MAT_SCAL_N * MAT_SCAL_N];
    int rows;
    int cols;
} mat_scal_matrix_t;

void mat_scalar_mul(mat_scal_matrix_t *m, double scalar) {
    int i;
    for (i = 0; i < m->rows * m->cols; i++) {
        m->data[i] *= scalar;
    }
}

void mat_scalar_mul_out(const mat_scal_matrix_t *src, double scalar, mat_scal_matrix_t *dst) {
    int i;
    dst->rows = src->rows;
    dst->cols = src->cols;
    for (i = 0; i < src->rows * src->cols; i++) {
        dst->data[i] = src->data[i] * scalar;
    }
}

double mat_scalar_trace(const mat_scal_matrix_t *m) {
    double sum = 0.0;
    int i;
    int n = m->rows;
    if (m->rows != m->cols) return 0.0;
    for (i = 0; i < n; i++) {
        sum += m->data[i * n + i];
    }
    return sum;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1780: Scalar multiply should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1780: Output should not be empty");
    assert!(
        code.contains("fn mat_scalar_mul"),
        "C1780: Should contain mat_scalar_mul function"
    );
}

// ============================================================================
// C1781-C1785: Matrix Decomposition
// ============================================================================

/// C1781: LU decomposition using Doolittle algorithm
#[test]
fn c1781_lu_decomposition() {
    let c_code = r##"
#define MAT_LU_N 4

typedef struct {
    double L[MAT_LU_N * MAT_LU_N];
    double U[MAT_LU_N * MAT_LU_N];
    int n;
    int valid;
} mat_lu_t;

void mat_lu_init(mat_lu_t *lu, int n) {
    int i, j;
    lu->n = n;
    lu->valid = 1;
    for (i = 0; i < n * n; i++) {
        lu->L[i] = 0.0;
        lu->U[i] = 0.0;
    }
    for (i = 0; i < n; i++) {
        lu->L[i * n + i] = 1.0;
    }
}

void mat_lu_decompose(mat_lu_t *lu, const double *A) {
    int i, j, k, n;
    n = lu->n;
    for (i = 0; i < n; i++) {
        for (j = i; j < n; j++) {
            double sum = 0.0;
            for (k = 0; k < i; k++) {
                sum += lu->L[i * n + k] * lu->U[k * n + j];
            }
            lu->U[i * n + j] = A[i * n + j] - sum;
        }
        for (j = i + 1; j < n; j++) {
            double sum = 0.0;
            for (k = 0; k < i; k++) {
                sum += lu->L[j * n + k] * lu->U[k * n + i];
            }
            if (lu->U[i * n + i] == 0.0) {
                lu->valid = 0;
                return;
            }
            lu->L[j * n + i] = (A[j * n + i] - sum) / lu->U[i * n + i];
        }
    }
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1781: LU decomposition should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1781: Output should not be empty");
    assert!(
        code.contains("fn mat_lu_decompose"),
        "C1781: Should contain mat_lu_decompose function"
    );
}

/// C1782: Forward substitution (Ly = b)
#[test]
fn c1782_forward_substitution() {
    let c_code = r##"
#define MAT_FWD_N 4

void mat_forward_sub(const double *L, const double *b, double *y, int n) {
    int i, j;
    for (i = 0; i < n; i++) {
        double sum = 0.0;
        for (j = 0; j < i; j++) {
            sum += L[i * n + j] * y[j];
        }
        y[i] = b[i] - sum;
    }
}

void mat_forward_sub_unit(const double *L, const double *b, double *y, int n) {
    int i, j;
    for (i = 0; i < n; i++) {
        double sum = 0.0;
        for (j = 0; j < i; j++) {
            sum += L[i * n + j] * y[j];
        }
        y[i] = (b[i] - sum) / L[i * n + i];
    }
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1782: Forward substitution should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1782: Output should not be empty");
    assert!(
        code.contains("fn mat_forward_sub"),
        "C1782: Should contain mat_forward_sub function"
    );
}

/// C1783: Back substitution (Ux = y)
#[test]
fn c1783_back_substitution() {
    let c_code = r##"
#define MAT_BACK_N 4

void mat_back_sub(const double *U, const double *y, double *x, int n) {
    int i, j;
    for (i = n - 1; i >= 0; i--) {
        double sum = 0.0;
        for (j = i + 1; j < n; j++) {
            sum += U[i * n + j] * x[j];
        }
        if (U[i * n + i] == 0.0) {
            x[i] = 0.0;
        } else {
            x[i] = (y[i] - sum) / U[i * n + i];
        }
    }
}

int mat_back_sub_check(const double *U, int n) {
    int i;
    for (i = 0; i < n; i++) {
        if (U[i * n + i] == 0.0) return 0;
    }
    return 1;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1783: Back substitution should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1783: Output should not be empty");
    assert!(
        code.contains("fn mat_back_sub"),
        "C1783: Should contain mat_back_sub function"
    );
}

/// C1784: Matrix determinant via cofactor expansion (small matrices)
#[test]
fn c1784_determinant() {
    let c_code = r##"
double mat_det_2x2(const double *m) {
    return m[0] * m[3] - m[1] * m[2];
}

double mat_det_3x3(const double *m) {
    double a = m[0] * (m[4] * m[8] - m[5] * m[7]);
    double b = m[1] * (m[3] * m[8] - m[5] * m[6]);
    double c = m[2] * (m[3] * m[7] - m[4] * m[6]);
    return a - b + c;
}

void mat_det_minor(const double *src, double *dst, int n, int row, int col) {
    int di, dj, si, sj;
    di = 0;
    for (si = 0; si < n; si++) {
        if (si == row) continue;
        dj = 0;
        for (sj = 0; sj < n; sj++) {
            if (sj == col) continue;
            dst[di * (n - 1) + dj] = src[si * n + sj];
            dj++;
        }
        di++;
    }
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1784: Determinant should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1784: Output should not be empty");
    assert!(
        code.contains("fn mat_det_2x2"),
        "C1784: Should contain mat_det_2x2 function"
    );
}

/// C1785: Small matrix inverse (3x3) using adjugate method
#[test]
fn c1785_matrix_inverse_3x3() {
    let c_code = r##"
int mat_inverse_3x3(const double *m, double *inv) {
    double det;
    double cofactors[9];
    int i;

    cofactors[0] = m[4] * m[8] - m[5] * m[7];
    cofactors[1] = -(m[3] * m[8] - m[5] * m[6]);
    cofactors[2] = m[3] * m[7] - m[4] * m[6];
    cofactors[3] = -(m[1] * m[8] - m[2] * m[7]);
    cofactors[4] = m[0] * m[8] - m[2] * m[6];
    cofactors[5] = -(m[0] * m[7] - m[1] * m[6]);
    cofactors[6] = m[1] * m[5] - m[2] * m[4];
    cofactors[7] = -(m[0] * m[5] - m[2] * m[3]);
    cofactors[8] = m[0] * m[4] - m[1] * m[3];

    det = m[0] * cofactors[0] + m[1] * cofactors[1] + m[2] * cofactors[2];

    if (det == 0.0) return 0;

    for (i = 0; i < 9; i++) {
        inv[i] = cofactors[i] / det;
    }

    return 1;
}

double mat_inverse_det_3x3(const double *m) {
    return m[0] * (m[4] * m[8] - m[5] * m[7])
         - m[1] * (m[3] * m[8] - m[5] * m[6])
         + m[2] * (m[3] * m[7] - m[4] * m[6]);
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1785: Matrix inverse 3x3 should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1785: Output should not be empty");
    assert!(
        code.contains("fn mat_inverse_3x3"),
        "C1785: Should contain mat_inverse_3x3 function"
    );
}

// ============================================================================
// C1786-C1790: Sparse Matrices
// ============================================================================

/// C1786: COO (Coordinate) sparse format
#[test]
fn c1786_sparse_coo() {
    let c_code = r##"
#define MAT_COO_MAX 64

typedef struct {
    int row[MAT_COO_MAX];
    int col[MAT_COO_MAX];
    double val[MAT_COO_MAX];
    int nnz;
    int nrows;
    int ncols;
} mat_coo_t;

void mat_coo_init(mat_coo_t *s, int nrows, int ncols) {
    s->nnz = 0;
    s->nrows = nrows;
    s->ncols = ncols;
}

int mat_coo_insert(mat_coo_t *s, int r, int c, double v) {
    if (s->nnz >= MAT_COO_MAX) return -1;
    s->row[s->nnz] = r;
    s->col[s->nnz] = c;
    s->val[s->nnz] = v;
    s->nnz++;
    return 0;
}

double mat_coo_get(const mat_coo_t *s, int r, int c) {
    int i;
    for (i = 0; i < s->nnz; i++) {
        if (s->row[i] == r && s->col[i] == c) {
            return s->val[i];
        }
    }
    return 0.0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1786: Sparse COO should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1786: Output should not be empty");
    assert!(
        code.contains("fn mat_coo_init"),
        "C1786: Should contain mat_coo_init function"
    );
}

/// C1787: CSR (Compressed Sparse Row) format
#[test]
fn c1787_sparse_csr() {
    let c_code = r##"
#define MAT_CSR_MAX_NNZ 64
#define MAT_CSR_MAX_ROWS 16

typedef struct {
    double values[MAT_CSR_MAX_NNZ];
    int col_idx[MAT_CSR_MAX_NNZ];
    int row_ptr[MAT_CSR_MAX_ROWS + 1];
    int nrows;
    int ncols;
    int nnz;
} mat_csr_t;

void mat_csr_init(mat_csr_t *s, int nrows, int ncols) {
    int i;
    s->nrows = nrows;
    s->ncols = ncols;
    s->nnz = 0;
    for (i = 0; i <= nrows; i++) {
        s->row_ptr[i] = 0;
    }
}

double mat_csr_get(const mat_csr_t *s, int r, int c) {
    int i;
    for (i = s->row_ptr[r]; i < s->row_ptr[r + 1]; i++) {
        if (s->col_idx[i] == c) {
            return s->values[i];
        }
    }
    return 0.0;
}

void mat_csr_row_sum(const mat_csr_t *s, double *sums) {
    int r, i;
    for (r = 0; r < s->nrows; r++) {
        sums[r] = 0.0;
        for (i = s->row_ptr[r]; i < s->row_ptr[r + 1]; i++) {
            sums[r] += s->values[i];
        }
    }
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1787: Sparse CSR should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1787: Output should not be empty");
    assert!(
        code.contains("fn mat_csr_init"),
        "C1787: Should contain mat_csr_init function"
    );
}

/// C1788: Sparse matrix-dense vector multiplication (CSR format)
#[test]
fn c1788_sparse_dense_multiply() {
    let c_code = r##"
#define MAT_SPMV_MAX_NNZ 64
#define MAT_SPMV_MAX_ROWS 16

typedef struct {
    double values[MAT_SPMV_MAX_NNZ];
    int col_idx[MAT_SPMV_MAX_NNZ];
    int row_ptr[MAT_SPMV_MAX_ROWS + 1];
    int nrows;
    int ncols;
    int nnz;
} mat_spmv_csr_t;

void mat_spmv_multiply(const mat_spmv_csr_t *A, const double *x, double *y) {
    int r, i;
    for (r = 0; r < A->nrows; r++) {
        y[r] = 0.0;
        for (i = A->row_ptr[r]; i < A->row_ptr[r + 1]; i++) {
            y[r] += A->values[i] * x[A->col_idx[i]];
        }
    }
}

double mat_spmv_dot(const double *a, const double *b, int n) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += a[i] * b[i];
    }
    return sum;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1788: Sparse-dense multiply should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1788: Output should not be empty");
    assert!(
        code.contains("fn mat_spmv_multiply"),
        "C1788: Should contain mat_spmv_multiply function"
    );
}

/// C1789: Sparse matrix addition in COO format
#[test]
fn c1789_sparse_add() {
    let c_code = r##"
#define MAT_SPADD_MAX 128

typedef struct {
    int row[MAT_SPADD_MAX];
    int col[MAT_SPADD_MAX];
    double val[MAT_SPADD_MAX];
    int nnz;
} mat_spadd_coo_t;

void mat_spadd_init(mat_spadd_coo_t *s) {
    s->nnz = 0;
}

int mat_spadd_push(mat_spadd_coo_t *s, int r, int c, double v) {
    if (s->nnz >= MAT_SPADD_MAX) return -1;
    s->row[s->nnz] = r;
    s->col[s->nnz] = c;
    s->val[s->nnz] = v;
    s->nnz++;
    return 0;
}

void mat_spadd(const mat_spadd_coo_t *A, const mat_spadd_coo_t *B, mat_spadd_coo_t *C) {
    int i;
    mat_spadd_init(C);
    for (i = 0; i < A->nnz; i++) {
        mat_spadd_push(C, A->row[i], A->col[i], A->val[i]);
    }
    for (i = 0; i < B->nnz; i++) {
        mat_spadd_push(C, B->row[i], B->col[i], B->val[i]);
    }
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1789: Sparse add should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1789: Output should not be empty");
    assert!(
        code.contains("fn mat_spadd"),
        "C1789: Should contain mat_spadd function"
    );
}

/// C1790: Sparse matrix transpose in COO format
#[test]
fn c1790_sparse_transpose() {
    let c_code = r##"
#define MAT_SPT_MAX 64

typedef struct {
    int row[MAT_SPT_MAX];
    int col[MAT_SPT_MAX];
    double val[MAT_SPT_MAX];
    int nnz;
    int nrows;
    int ncols;
} mat_spt_coo_t;

void mat_spt_init(mat_spt_coo_t *s, int nrows, int ncols) {
    s->nnz = 0;
    s->nrows = nrows;
    s->ncols = ncols;
}

void mat_spt_transpose(const mat_spt_coo_t *src, mat_spt_coo_t *dst) {
    int i;
    dst->nrows = src->ncols;
    dst->ncols = src->nrows;
    dst->nnz = src->nnz;
    for (i = 0; i < src->nnz; i++) {
        dst->row[i] = src->col[i];
        dst->col[i] = src->row[i];
        dst->val[i] = src->val[i];
    }
}

int mat_spt_count_row(const mat_spt_coo_t *s, int r) {
    int i, count;
    count = 0;
    for (i = 0; i < s->nnz; i++) {
        if (s->row[i] == r) count++;
    }
    return count;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1790: Sparse transpose should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1790: Output should not be empty");
    assert!(
        code.contains("fn mat_spt_transpose"),
        "C1790: Should contain mat_spt_transpose function"
    );
}

// ============================================================================
// C1791-C1795: Vector Operations
// ============================================================================

/// C1791: Vector dot product
#[test]
fn c1791_dot_product() {
    let c_code = r##"
double mat_dot_product(const double *a, const double *b, int n) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += a[i] * b[i];
    }
    return sum;
}

double mat_dot_weighted(const double *a, const double *b, const double *w, int n) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += a[i] * b[i] * w[i];
    }
    return sum;
}

double mat_dot_self(const double *a, int n) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += a[i] * a[i];
    }
    return sum;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1791: Dot product should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1791: Output should not be empty");
    assert!(
        code.contains("fn mat_dot_product"),
        "C1791: Should contain mat_dot_product function"
    );
}

/// C1792: 3D cross product
#[test]
fn c1792_cross_product() {
    let c_code = r##"
typedef struct {
    double x;
    double y;
    double z;
} mat_vec3_t;

mat_vec3_t mat_cross_product(mat_vec3_t a, mat_vec3_t b) {
    mat_vec3_t result;
    result.x = a.y * b.z - a.z * b.y;
    result.y = a.z * b.x - a.x * b.z;
    result.z = a.x * b.y - a.y * b.x;
    return result;
}

double mat_cross_magnitude(mat_vec3_t a, mat_vec3_t b) {
    mat_vec3_t c;
    c = mat_cross_product(a, b);
    return c.x * c.x + c.y * c.y + c.z * c.z;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1792: Cross product should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1792: Output should not be empty");
    assert!(
        code.contains("fn mat_cross_product"),
        "C1792: Should contain mat_cross_product function"
    );
}

/// C1793: Vector normalization
#[test]
fn c1793_vector_normalize() {
    let c_code = r##"
double mat_vec_norm_squared(const double *v, int n) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += v[i] * v[i];
    }
    return sum;
}

double mat_vec_norm_approx(double x) {
    double guess = x * 0.5;
    int i;
    if (x <= 0.0) return 0.0;
    for (i = 0; i < 20; i++) {
        guess = 0.5 * (guess + x / guess);
    }
    return guess;
}

void mat_vec_normalize(double *v, int n) {
    double norm_sq = mat_vec_norm_squared(v, n);
    double norm = mat_vec_norm_approx(norm_sq);
    int i;
    if (norm > 1e-15) {
        for (i = 0; i < n; i++) {
            v[i] /= norm;
        }
    }
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1793: Vector normalize should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1793: Output should not be empty");
    assert!(
        code.contains("fn mat_vec_normalize"),
        "C1793: Should contain mat_vec_normalize function"
    );
}

/// C1794: Gram-Schmidt orthogonalization
#[test]
fn c1794_gram_schmidt() {
    let c_code = r##"
#define MAT_GS_DIM 4
#define MAT_GS_VECS 4

double mat_gs_dot(const double *a, const double *b, int n) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += a[i] * b[i];
    }
    return sum;
}

double mat_gs_sqrt(double x) {
    double g = x * 0.5;
    int i;
    if (x <= 0.0) return 0.0;
    for (i = 0; i < 20; i++) {
        g = 0.5 * (g + x / g);
    }
    return g;
}

void mat_gs_normalize(double *v, int n) {
    double norm = mat_gs_sqrt(mat_gs_dot(v, v, n));
    int i;
    if (norm > 1e-15) {
        for (i = 0; i < n; i++) {
            v[i] /= norm;
        }
    }
}

void mat_gs_project(const double *u, const double *v, double *result, int n) {
    double dot_uv = mat_gs_dot(u, v, n);
    double dot_uu = mat_gs_dot(u, u, n);
    int i;
    if (dot_uu > 1e-15) {
        double scale = dot_uv / dot_uu;
        for (i = 0; i < n; i++) {
            result[i] = scale * u[i];
        }
    } else {
        for (i = 0; i < n; i++) {
            result[i] = 0.0;
        }
    }
}

void mat_gs_orthogonalize(double *vecs, int nvecs, int dim) {
    int i, j, k;
    double proj[MAT_GS_DIM];
    for (i = 1; i < nvecs; i++) {
        for (j = 0; j < i; j++) {
            mat_gs_project(&vecs[j * dim], &vecs[i * dim], proj, dim);
            for (k = 0; k < dim; k++) {
                vecs[i * dim + k] -= proj[k];
            }
        }
        mat_gs_normalize(&vecs[i * dim], dim);
    }
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1794: Gram-Schmidt should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1794: Output should not be empty");
    assert!(
        code.contains("fn mat_gs_orthogonalize"),
        "C1794: Should contain mat_gs_orthogonalize function"
    );
}

/// C1795: Vector projection onto another vector
#[test]
fn c1795_vector_projection() {
    let c_code = r##"
double mat_proj_dot(const double *a, const double *b, int n) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += a[i] * b[i];
    }
    return sum;
}

void mat_proj_vector(const double *v, const double *onto, double *result, int n) {
    double dot_vo = mat_proj_dot(v, onto, n);
    double dot_oo = mat_proj_dot(onto, onto, n);
    int i;
    if (dot_oo > 1e-15) {
        double scale = dot_vo / dot_oo;
        for (i = 0; i < n; i++) {
            result[i] = scale * onto[i];
        }
    } else {
        for (i = 0; i < n; i++) {
            result[i] = 0.0;
        }
    }
}

void mat_proj_reject(const double *v, const double *onto, double *result, int n) {
    double proj[16];
    int i;
    mat_proj_vector(v, onto, proj, n);
    for (i = 0; i < n; i++) {
        result[i] = v[i] - proj[i];
    }
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1795: Vector projection should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1795: Output should not be empty");
    assert!(
        code.contains("fn mat_proj_vector"),
        "C1795: Should contain mat_proj_vector function"
    );
}

// ============================================================================
// C1796-C1800: Tensor Operations
// ============================================================================

/// C1796: 3D tensor indexing with flat array storage
#[test]
fn c1796_tensor_3d_indexing() {
    let c_code = r##"
#define MAT_TENS_D0 4
#define MAT_TENS_D1 4
#define MAT_TENS_D2 4
#define MAT_TENS_SIZE (MAT_TENS_D0 * MAT_TENS_D1 * MAT_TENS_D2)

typedef struct {
    double data[MAT_TENS_SIZE];
    int dim0;
    int dim1;
    int dim2;
} mat_tensor3d_t;

void mat_tensor3d_init(mat_tensor3d_t *t, int d0, int d1, int d2) {
    int i;
    t->dim0 = d0;
    t->dim1 = d1;
    t->dim2 = d2;
    for (i = 0; i < d0 * d1 * d2; i++) {
        t->data[i] = 0.0;
    }
}

double mat_tensor3d_get(const mat_tensor3d_t *t, int i, int j, int k) {
    return t->data[i * t->dim1 * t->dim2 + j * t->dim2 + k];
}

void mat_tensor3d_set(mat_tensor3d_t *t, int i, int j, int k, double val) {
    t->data[i * t->dim1 * t->dim2 + j * t->dim2 + k] = val;
}

void mat_tensor3d_fill(mat_tensor3d_t *t, double val) {
    int i;
    for (i = 0; i < t->dim0 * t->dim1 * t->dim2; i++) {
        t->data[i] = val;
    }
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1796: Tensor 3D indexing should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1796: Output should not be empty");
    assert!(
        code.contains("fn mat_tensor3d_get"),
        "C1796: Should contain mat_tensor3d_get function"
    );
}

/// C1797: Tensor reshape (reinterpret dimensions without moving data)
#[test]
fn c1797_tensor_reshape() {
    let c_code = r##"
#define MAT_RESH_MAX 256

typedef struct {
    double data[MAT_RESH_MAX];
    int dims[4];
    int ndims;
    int total;
} mat_reshape_tensor_t;

void mat_reshape_init(mat_reshape_tensor_t *t, int total) {
    int i;
    t->total = total;
    t->ndims = 1;
    t->dims[0] = total;
    for (i = 0; i < total; i++) {
        t->data[i] = 0.0;
    }
}

int mat_reshape(mat_reshape_tensor_t *t, int ndims, const int *new_dims) {
    int product = 1;
    int i;
    for (i = 0; i < ndims; i++) {
        product *= new_dims[i];
    }
    if (product != t->total) return -1;
    t->ndims = ndims;
    for (i = 0; i < ndims; i++) {
        t->dims[i] = new_dims[i];
    }
    return 0;
}

int mat_reshape_flat_idx(const mat_reshape_tensor_t *t, const int *indices) {
    int idx = 0;
    int stride = 1;
    int i;
    for (i = t->ndims - 1; i >= 0; i--) {
        idx += indices[i] * stride;
        stride *= t->dims[i];
    }
    return idx;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1797: Tensor reshape should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1797: Output should not be empty");
    assert!(
        code.contains("fn mat_reshape"),
        "C1797: Should contain mat_reshape function"
    );
}

/// C1798: Tensor slice (extract sub-tensor along a dimension)
#[test]
fn c1798_tensor_slice() {
    let c_code = r##"
#define MAT_SLICE_D 4

typedef struct {
    double data[MAT_SLICE_D * MAT_SLICE_D * MAT_SLICE_D];
    int d0;
    int d1;
    int d2;
} mat_slice_tensor_t;

typedef struct {
    double data[MAT_SLICE_D * MAT_SLICE_D];
    int rows;
    int cols;
} mat_slice_matrix_t;

void mat_slice_along_d0(const mat_slice_tensor_t *t, int idx, mat_slice_matrix_t *out) {
    int j, k;
    out->rows = t->d1;
    out->cols = t->d2;
    for (j = 0; j < t->d1; j++) {
        for (k = 0; k < t->d2; k++) {
            out->data[j * t->d2 + k] = t->data[idx * t->d1 * t->d2 + j * t->d2 + k];
        }
    }
}

void mat_slice_along_d2(const mat_slice_tensor_t *t, int idx, mat_slice_matrix_t *out) {
    int i, j;
    out->rows = t->d0;
    out->cols = t->d1;
    for (i = 0; i < t->d0; i++) {
        for (j = 0; j < t->d1; j++) {
            out->data[i * t->d1 + j] = t->data[i * t->d1 * t->d2 + j * t->d2 + idx];
        }
    }
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1798: Tensor slice should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1798: Output should not be empty");
    assert!(
        code.contains("fn mat_slice_along_d0"),
        "C1798: Should contain mat_slice_along_d0 function"
    );
}

/// C1799: Batch matrix multiply (multiply N pairs of matrices)
#[test]
fn c1799_batch_matrix_multiply() {
    let c_code = r##"
#define MAT_BATCH_DIM 4
#define MAT_BATCH_COUNT 4

typedef struct {
    double data[MAT_BATCH_COUNT * MAT_BATCH_DIM * MAT_BATCH_DIM];
    int batch_size;
    int rows;
    int cols;
} mat_batch_t;

void mat_batch_init(mat_batch_t *b, int batch, int rows, int cols) {
    int i;
    b->batch_size = batch;
    b->rows = rows;
    b->cols = cols;
    for (i = 0; i < batch * rows * cols; i++) {
        b->data[i] = 0.0;
    }
}

void mat_batch_get_matrix(const mat_batch_t *b, int idx, double *out) {
    int i;
    int offset = idx * b->rows * b->cols;
    for (i = 0; i < b->rows * b->cols; i++) {
        out[i] = b->data[offset + i];
    }
}

void mat_batch_multiply(const mat_batch_t *A, const mat_batch_t *B, mat_batch_t *C) {
    int n, i, j, k;
    int dim = A->rows;
    mat_batch_init(C, A->batch_size, A->rows, B->cols);
    for (n = 0; n < A->batch_size; n++) {
        int off_a = n * dim * dim;
        int off_b = n * dim * dim;
        int off_c = n * dim * dim;
        for (i = 0; i < dim; i++) {
            for (j = 0; j < dim; j++) {
                double sum = 0.0;
                for (k = 0; k < dim; k++) {
                    sum += A->data[off_a + i * dim + k] * B->data[off_b + k * dim + j];
                }
                C->data[off_c + i * dim + j] = sum;
            }
        }
    }
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1799: Batch matrix multiply should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1799: Output should not be empty");
    assert!(
        code.contains("fn mat_batch_multiply"),
        "C1799: Should contain mat_batch_multiply function"
    );
}

/// C1800: Element-wise tensor operations (add, multiply, scale)
#[test]
fn c1800_elementwise_tensor_ops() {
    let c_code = r##"
#define MAT_EW_MAX 256

typedef struct {
    double data[MAT_EW_MAX];
    int size;
} mat_ew_tensor_t;

void mat_ew_init(mat_ew_tensor_t *t, int size) {
    int i;
    t->size = size;
    for (i = 0; i < size; i++) {
        t->data[i] = 0.0;
    }
}

void mat_ew_add(const mat_ew_tensor_t *a, const mat_ew_tensor_t *b, mat_ew_tensor_t *c) {
    int i;
    c->size = a->size;
    for (i = 0; i < a->size; i++) {
        c->data[i] = a->data[i] + b->data[i];
    }
}

void mat_ew_multiply(const mat_ew_tensor_t *a, const mat_ew_tensor_t *b, mat_ew_tensor_t *c) {
    int i;
    c->size = a->size;
    for (i = 0; i < a->size; i++) {
        c->data[i] = a->data[i] * b->data[i];
    }
}

void mat_ew_scale(mat_ew_tensor_t *t, double s) {
    int i;
    for (i = 0; i < t->size; i++) {
        t->data[i] *= s;
    }
}

double mat_ew_sum(const mat_ew_tensor_t *t) {
    double sum = 0.0;
    int i;
    for (i = 0; i < t->size; i++) {
        sum += t->data[i];
    }
    return sum;
}

double mat_ew_max(const mat_ew_tensor_t *t) {
    double max_val;
    int i;
    if (t->size == 0) return 0.0;
    max_val = t->data[0];
    for (i = 1; i < t->size; i++) {
        if (t->data[i] > max_val) {
            max_val = t->data[i];
        }
    }
    return max_val;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1800: Element-wise tensor ops should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1800: Output should not be empty");
    assert!(
        code.contains("fn mat_ew_add"),
        "C1800: Should contain mat_ew_add function"
    );
}
