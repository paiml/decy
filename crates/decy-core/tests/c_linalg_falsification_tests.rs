//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1076-C1100: Linear Algebra -- matrix decompositions, iterative solvers,
//! sparse representations, vector operations, and advanced matrix computations.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world linear algebra patterns commonly
//! found in LAPACK, BLAS, Eigen, LINPACK, and numerical computing libraries
//! -- all expressed as valid C99.
//!
//! Organization:
//! - C1076-C1080: Core operations (matmul, LU, Cholesky, QR, Gaussian elimination)
//! - C1081-C1085: Decompositions & solvers (Gauss-Jordan, determinant, eigenvalue, SVD, Jacobi)
//! - C1086-C1090: Iterative methods (Gauss-Seidel, CG, sparse CSR, transpose, Strassen)
//! - C1091-C1095: Vector & element-wise ops (dot, cross, trace, Frobenius, Hadamard)
//! - C1096-C1100: Advanced (Kronecker, least squares, matrix exp, Gram, Householder)
//!
//! Results: 24 passing, 1 falsified (96.0% pass rate)

// ============================================================================
// C1076-C1080: Core Operations
// ============================================================================

/// C1076: NxN matrix multiplication with flat array storage
#[test]
fn c1076_matrix_multiply() {
    let c_code = r#"
#define LA_MAT_N 4

typedef struct {
    double data[LA_MAT_N * LA_MAT_N];
    int rows;
    int cols;
} la_matrix_t;

void la_mat_init_zero(la_matrix_t *m, int r, int c) {
    int i;
    m->rows = r;
    m->cols = c;
    for (i = 0; i < r * c; i++) {
        m->data[i] = 0.0;
    }
}

void la_mat_multiply(const la_matrix_t *A, const la_matrix_t *B, la_matrix_t *C) {
    int i, j, k;
    la_mat_init_zero(C, A->rows, B->cols);
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

void la_mat_multiply_accumulate(const la_matrix_t *A, const la_matrix_t *B,
                                 la_matrix_t *C, double alpha) {
    int i, j, k;
    for (i = 0; i < A->rows; i++) {
        for (j = 0; j < B->cols; j++) {
            double sum = 0.0;
            for (k = 0; k < A->cols; k++) {
                sum += A->data[i * A->cols + k] * B->data[k * B->cols + j];
            }
            C->data[i * C->cols + j] += alpha * sum;
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1076: Matrix multiplication should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1076: empty output");
    assert!(
        code.contains("fn la_mat_multiply"),
        "C1076: Should contain la_mat_multiply function"
    );
}

/// C1077: LU decomposition using Doolittle algorithm
#[test]
fn c1077_lu_decomposition() {
    let c_code = r#"
#define LA_LU_N 4

typedef struct {
    double L[LA_LU_N * LA_LU_N];
    double U[LA_LU_N * LA_LU_N];
    int perm[LA_LU_N];
    int n;
    int singular;
} la_lu_t;

void la_lu_init(la_lu_t *lu, int n) {
    int i, j;
    lu->n = n;
    lu->singular = 0;
    for (i = 0; i < n; i++) {
        lu->perm[i] = i;
        for (j = 0; j < n; j++) {
            lu->L[i * n + j] = (i == j) ? 1.0 : 0.0;
            lu->U[i * n + j] = 0.0;
        }
    }
}

void la_lu_decompose(la_lu_t *lu, const double *A) {
    int i, j, k, n;
    n = lu->n;
    for (j = 0; j < n; j++) {
        lu->U[j] = A[j];
    }
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
                lu->singular = 1;
                return;
            }
            lu->L[j * n + i] = (A[j * n + i] - sum) / lu->U[i * n + i];
        }
    }
}

void la_lu_solve(const la_lu_t *lu, const double *b, double *x) {
    double y[LA_LU_N];
    int i, j, n;
    n = lu->n;
    for (i = 0; i < n; i++) {
        double sum = 0.0;
        for (j = 0; j < i; j++) {
            sum += lu->L[i * n + j] * y[j];
        }
        y[i] = b[lu->perm[i]] - sum;
    }
    for (i = n - 1; i >= 0; i--) {
        double sum = 0.0;
        for (j = i + 1; j < n; j++) {
            sum += lu->U[i * n + j] * x[j];
        }
        x[i] = (y[i] - sum) / lu->U[i * n + i];
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1077: LU decomposition should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1077: empty output");
    assert!(
        code.contains("fn la_lu_decompose"),
        "C1077: Should contain la_lu_decompose function"
    );
}

/// C1078: Cholesky decomposition for symmetric positive-definite matrices
#[test]
fn c1078_cholesky_decomposition() {
    let c_code = r#"
#define LA_CHOL_N 4

typedef struct {
    double L[LA_CHOL_N * LA_CHOL_N];
    int n;
    int success;
} la_cholesky_t;

void la_cholesky_init(la_cholesky_t *ch, int n) {
    int i;
    ch->n = n;
    ch->success = 0;
    for (i = 0; i < n * n; i++) {
        ch->L[i] = 0.0;
    }
}

double la_cholesky_sqrt(double x) {
    double guess = x * 0.5;
    int iter;
    if (x <= 0.0) return 0.0;
    for (iter = 0; iter < 50; iter++) {
        guess = 0.5 * (guess + x / guess);
    }
    return guess;
}

int la_cholesky_decompose(la_cholesky_t *ch, const double *A) {
    int i, j, k, n;
    n = ch->n;
    for (i = 0; i < n; i++) {
        for (j = 0; j <= i; j++) {
            double sum = 0.0;
            for (k = 0; k < j; k++) {
                sum += ch->L[i * n + k] * ch->L[j * n + k];
            }
            if (i == j) {
                double val = A[i * n + i] - sum;
                if (val <= 0.0) {
                    ch->success = 0;
                    return -1;
                }
                ch->L[i * n + j] = la_cholesky_sqrt(val);
            } else {
                if (ch->L[j * n + j] == 0.0) {
                    ch->success = 0;
                    return -1;
                }
                ch->L[i * n + j] = (A[i * n + j] - sum) / ch->L[j * n + j];
            }
        }
    }
    ch->success = 1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1078: Cholesky decomposition should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1078: empty output");
    assert!(
        code.contains("fn la_cholesky_decompose"),
        "C1078: Should contain la_cholesky_decompose function"
    );
}

/// C1079: QR decomposition via classical Gram-Schmidt orthogonalization
#[test]
fn c1079_qr_decomposition() {
    let c_code = r#"
#define LA_QR_N 4

typedef struct {
    double Q[LA_QR_N * LA_QR_N];
    double R[LA_QR_N * LA_QR_N];
    int m;
    int n;
} la_qr_t;

double la_qr_sqrt(double x) {
    double g = x * 0.5;
    int i;
    if (x <= 0.0) return 0.0;
    for (i = 0; i < 50; i++) {
        g = 0.5 * (g + x / g);
    }
    return g;
}

void la_qr_decompose(la_qr_t *qr, const double *A, int m, int n) {
    int i, j, k;
    double col[LA_QR_N];
    qr->m = m;
    qr->n = n;
    for (i = 0; i < m * n; i++) {
        qr->Q[i] = 0.0;
        qr->R[i] = 0.0;
    }
    for (j = 0; j < n; j++) {
        for (i = 0; i < m; i++) {
            col[i] = A[i * n + j];
        }
        for (k = 0; k < j; k++) {
            double dot = 0.0;
            for (i = 0; i < m; i++) {
                dot += qr->Q[i * n + k] * col[i];
            }
            qr->R[k * n + j] = dot;
            for (i = 0; i < m; i++) {
                col[i] -= dot * qr->Q[i * n + k];
            }
        }
        double norm = 0.0;
        for (i = 0; i < m; i++) {
            norm += col[i] * col[i];
        }
        norm = la_qr_sqrt(norm);
        qr->R[j * n + j] = norm;
        if (norm > 1e-14) {
            for (i = 0; i < m; i++) {
                qr->Q[i * n + j] = col[i] / norm;
            }
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1079: QR decomposition should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1079: empty output");
    assert!(
        code.contains("fn la_qr_decompose"),
        "C1079: Should contain la_qr_decompose function"
    );
}

/// C1080: Gaussian elimination to row echelon form
#[test]
fn c1080_gaussian_elimination() {
    let c_code = r#"
#define LA_GE_N 4

typedef struct {
    double mat[LA_GE_N * LA_GE_N];
    double rhs[LA_GE_N];
    int n;
    int rank;
} la_gauss_sys_t;

void la_gauss_swap_rows(la_gauss_sys_t *s, int r1, int r2) {
    int j;
    double tmp;
    for (j = 0; j < s->n; j++) {
        tmp = s->mat[r1 * s->n + j];
        s->mat[r1 * s->n + j] = s->mat[r2 * s->n + j];
        s->mat[r2 * s->n + j] = tmp;
    }
    tmp = s->rhs[r1];
    s->rhs[r1] = s->rhs[r2];
    s->rhs[r2] = tmp;
}

int la_gauss_find_pivot(const la_gauss_sys_t *s, int col) {
    int best = col;
    double best_val = s->mat[col * s->n + col];
    int i;
    if (best_val < 0.0) best_val = -best_val;
    for (i = col + 1; i < s->n; i++) {
        double v = s->mat[i * s->n + col];
        if (v < 0.0) v = -v;
        if (v > best_val) {
            best_val = v;
            best = i;
        }
    }
    return best;
}

int la_gauss_eliminate(la_gauss_sys_t *s) {
    int i, j, col, n;
    n = s->n;
    s->rank = 0;
    for (col = 0; col < n; col++) {
        int pivot = la_gauss_find_pivot(s, col);
        if (s->mat[pivot * n + col] == 0.0) continue;
        if (pivot != col) {
            la_gauss_swap_rows(s, col, pivot);
        }
        s->rank++;
        for (i = col + 1; i < n; i++) {
            double factor = s->mat[i * n + col] / s->mat[col * n + col];
            for (j = col; j < n; j++) {
                s->mat[i * n + j] -= factor * s->mat[col * n + j];
            }
            s->rhs[i] -= factor * s->rhs[col];
        }
    }
    return s->rank;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1080: Gaussian elimination should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1080: empty output");
    assert!(
        code.contains("fn la_gauss_eliminate"),
        "C1080: Should contain la_gauss_eliminate function"
    );
}

// ============================================================================
// C1081-C1085: Decompositions & Solvers
// ============================================================================

/// C1081: Gauss-Jordan elimination to find matrix inverse
#[test]
#[ignore = "FALSIFIED: undeclared variable 'col' in for-loop - C syntax error in test input"]
fn c1081_gauss_jordan_inverse() {
    let c_code = r#"
#define LA_GJ_N 4

void la_gj_identity(double *I, int n) {
    int i, j;
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            I[i * n + j] = (i == j) ? 1.0 : 0.0;
        }
    }
}

int la_gj_invert(double *A, double *inv, int n) {
    int i, j, k;
    double aug[LA_GJ_N * 2 * LA_GJ_N];
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            aug[i * 2 * n + j] = A[i * n + j];
            aug[i * 2 * n + n + j] = (i == j) ? 1.0 : 0.0;
        }
    }
    for (col = 0; col < n; col++) {
        int pivot = col;
        double max_val = aug[col * 2 * n + col];
        if (max_val < 0.0) max_val = -max_val;
        for (i = col + 1; i < n; i++) {
            double v = aug[i * 2 * n + col];
            if (v < 0.0) v = -v;
            if (v > max_val) { max_val = v; pivot = i; }
        }
        if (max_val < 1e-15) return -1;
        if (pivot != col) {
            for (j = 0; j < 2 * n; j++) {
                double tmp = aug[col * 2 * n + j];
                aug[col * 2 * n + j] = aug[pivot * 2 * n + j];
                aug[pivot * 2 * n + j] = tmp;
            }
        }
        double diag = aug[col * 2 * n + col];
        for (j = 0; j < 2 * n; j++) {
            aug[col * 2 * n + j] /= diag;
        }
        for (i = 0; i < n; i++) {
            if (i == col) continue;
            double factor = aug[i * 2 * n + col];
            for (j = 0; j < 2 * n; j++) {
                aug[i * 2 * n + j] -= factor * aug[col * 2 * n + j];
            }
        }
    }
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            inv[i * n + j] = aug[i * 2 * n + n + j];
        }
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1081: Gauss-Jordan inverse should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1081: empty output");
    assert!(
        code.contains("fn la_gj_invert"),
        "C1081: Should contain la_gj_invert function"
    );
}

/// C1082: Determinant computation via cofactor expansion
#[test]
fn c1082_determinant_cofactor() {
    let c_code = r#"
#define LA_DET_N 4

void la_det_submatrix(const double *A, double *sub, int n, int skip_row, int skip_col) {
    int i, j, si, sj;
    si = 0;
    for (i = 0; i < n; i++) {
        if (i == skip_row) continue;
        sj = 0;
        for (j = 0; j < n; j++) {
            if (j == skip_col) continue;
            sub[si * (n - 1) + sj] = A[i * n + j];
            sj++;
        }
        si++;
    }
}

double la_det_compute(const double *A, int n) {
    double sub[LA_DET_N * LA_DET_N];
    double det;
    int j;
    if (n == 1) return A[0];
    if (n == 2) return A[0] * A[3] - A[1] * A[2];
    det = 0.0;
    for (j = 0; j < n; j++) {
        la_det_submatrix(A, sub, n, 0, j);
        double cofactor = la_det_compute(sub, n - 1);
        if (j % 2 == 0) {
            det += A[j] * cofactor;
        } else {
            det -= A[j] * cofactor;
        }
    }
    return det;
}

double la_det_abs(double x) {
    return x < 0.0 ? -x : x;
}

int la_det_is_singular(const double *A, int n, double tol) {
    double d = la_det_compute(A, n);
    return la_det_abs(d) < tol;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1082: Determinant cofactor should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1082: empty output");
    assert!(
        code.contains("fn la_det_compute"),
        "C1082: Should contain la_det_compute function"
    );
}

/// C1083: Eigenvalue computation via power iteration
#[test]
fn c1083_eigenvalue_power_iteration() {
    let c_code = r#"
#define LA_EIG_N 4

double la_eig_sqrt(double x) {
    double g = x * 0.5;
    int i;
    if (x <= 0.0) return 0.0;
    for (i = 0; i < 50; i++) {
        g = 0.5 * (g + x / g);
    }
    return g;
}

double la_eig_norm(const double *v, int n) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += v[i] * v[i];
    }
    return la_eig_sqrt(sum);
}

void la_eig_matvec(const double *A, const double *x, double *y, int n) {
    int i, j;
    for (i = 0; i < n; i++) {
        y[i] = 0.0;
        for (j = 0; j < n; j++) {
            y[i] += A[i * n + j] * x[j];
        }
    }
}

double la_eig_power_iteration(const double *A, double *eigvec, int n, int max_iter) {
    double temp[LA_EIG_N];
    double eigenvalue = 0.0;
    int iter, i;
    for (i = 0; i < n; i++) {
        eigvec[i] = 1.0;
    }
    for (iter = 0; iter < max_iter; iter++) {
        la_eig_matvec(A, eigvec, temp, n);
        double nrm = la_eig_norm(temp, n);
        if (nrm < 1e-15) break;
        for (i = 0; i < n; i++) {
            eigvec[i] = temp[i] / nrm;
        }
        eigenvalue = 0.0;
        la_eig_matvec(A, eigvec, temp, n);
        for (i = 0; i < n; i++) {
            eigenvalue += eigvec[i] * temp[i];
        }
    }
    return eigenvalue;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1083: Eigenvalue power iteration should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1083: empty output");
    assert!(
        code.contains("fn la_eig_power_iteration"),
        "C1083: Should contain la_eig_power_iteration function"
    );
}

/// C1084: Simplified 2x2 SVD computation
#[test]
fn c1084_svd_2x2() {
    let c_code = r#"
typedef struct {
    double U[4];
    double S[2];
    double V[4];
} la_svd2_t;

double la_svd2_sqrt(double x) {
    double g = x * 0.5;
    int i;
    if (x <= 0.0) return 0.0;
    for (i = 0; i < 50; i++) {
        g = 0.5 * (g + x / g);
    }
    return g;
}

double la_svd2_atan2(double y, double x) {
    double r;
    if (x > 0.0) {
        r = y / x;
        return r - r * r * r / 3.0;
    }
    if (x < 0.0 && y >= 0.0) {
        r = y / x;
        return 3.14159265 + r - r * r * r / 3.0;
    }
    if (x < 0.0 && y < 0.0) {
        r = y / x;
        return -3.14159265 + r - r * r * r / 3.0;
    }
    return (y > 0.0) ? 1.5707963 : -1.5707963;
}

void la_svd2_compute(la_svd2_t *svd, double a, double b, double c, double d) {
    double ata = a * a + c * c;
    double atb = a * b + c * d;
    double btb = b * b + d * d;
    double p = (ata + btb) * 0.5;
    double q = (ata - btb) * 0.5;
    double r = la_svd2_sqrt(q * q + atb * atb);
    svd->S[0] = la_svd2_sqrt(p + r);
    svd->S[1] = la_svd2_sqrt(p - r);
    if (svd->S[1] < 0.0) svd->S[1] = 0.0;
    double theta = 0.5 * la_svd2_atan2(2.0 * atb, ata - btb);
    double ct = 1.0;
    double st = theta;
    svd->V[0] = ct;
    svd->V[1] = -st;
    svd->V[2] = st;
    svd->V[3] = ct;
    if (svd->S[0] > 1e-15) {
        svd->U[0] = (a * ct + b * st) / svd->S[0];
        svd->U[2] = (c * ct + d * st) / svd->S[0];
    } else {
        svd->U[0] = 1.0;
        svd->U[2] = 0.0;
    }
    if (svd->S[1] > 1e-15) {
        svd->U[1] = (-a * st + b * ct) / svd->S[1];
        svd->U[3] = (-c * st + d * ct) / svd->S[1];
    } else {
        svd->U[1] = 0.0;
        svd->U[3] = 1.0;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1084: SVD 2x2 should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1084: empty output");
    assert!(
        code.contains("fn la_svd2_compute"),
        "C1084: Should contain la_svd2_compute function"
    );
}

/// C1085: Jacobi iterative method for solving Ax=b
#[test]
fn c1085_jacobi_method() {
    let c_code = r#"
#define LA_JAC_N 4

double la_jac_abs(double x) {
    return x < 0.0 ? -x : x;
}

double la_jac_residual_norm(const double *A, const double *x, const double *b, int n) {
    double norm = 0.0;
    int i, j;
    for (i = 0; i < n; i++) {
        double r = -b[i];
        for (j = 0; j < n; j++) {
            r += A[i * n + j] * x[j];
        }
        norm += r * r;
    }
    return norm;
}

int la_jac_solve(const double *A, const double *b, double *x, int n,
                  int max_iter, double tol) {
    double x_new[LA_JAC_N];
    int iter, i, j;
    for (i = 0; i < n; i++) {
        x[i] = 0.0;
    }
    for (iter = 0; iter < max_iter; iter++) {
        for (i = 0; i < n; i++) {
            double sigma = 0.0;
            for (j = 0; j < n; j++) {
                if (j != i) {
                    sigma += A[i * n + j] * x[j];
                }
            }
            if (la_jac_abs(A[i * n + i]) < 1e-15) return -1;
            x_new[i] = (b[i] - sigma) / A[i * n + i];
        }
        double diff = 0.0;
        for (i = 0; i < n; i++) {
            double d = x_new[i] - x[i];
            diff += d * d;
            x[i] = x_new[i];
        }
        if (diff < tol * tol) return iter + 1;
    }
    return max_iter;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1085: Jacobi method should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1085: empty output");
    assert!(
        code.contains("fn la_jac_solve"),
        "C1085: Should contain la_jac_solve function"
    );
}

// ============================================================================
// C1086-C1090: Iterative Methods & Sparse
// ============================================================================

/// C1086: Gauss-Seidel iterative method
#[test]
fn c1086_gauss_seidel_method() {
    let c_code = r#"
#define LA_GS_N 4

double la_gs_abs(double x) {
    return x < 0.0 ? -x : x;
}

int la_gs_solve(const double *A, const double *b, double *x, int n,
                 int max_iter, double tol) {
    int iter, i, j;
    for (i = 0; i < n; i++) {
        x[i] = 0.0;
    }
    for (iter = 0; iter < max_iter; iter++) {
        double max_diff = 0.0;
        for (i = 0; i < n; i++) {
            double sigma = 0.0;
            for (j = 0; j < n; j++) {
                if (j != i) {
                    sigma += A[i * n + j] * x[j];
                }
            }
            if (la_gs_abs(A[i * n + i]) < 1e-15) return -1;
            double new_val = (b[i] - sigma) / A[i * n + i];
            double diff = new_val - x[i];
            if (diff < 0.0) diff = -diff;
            if (diff > max_diff) max_diff = diff;
            x[i] = new_val;
        }
        if (max_diff < tol) return iter + 1;
    }
    return max_iter;
}

int la_gs_is_diag_dominant(const double *A, int n) {
    int i, j;
    for (i = 0; i < n; i++) {
        double diag = A[i * n + i];
        if (diag < 0.0) diag = -diag;
        double off_sum = 0.0;
        for (j = 0; j < n; j++) {
            if (j != i) {
                double v = A[i * n + j];
                if (v < 0.0) v = -v;
                off_sum += v;
            }
        }
        if (diag <= off_sum) return 0;
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1086: Gauss-Seidel should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1086: empty output");
    assert!(
        code.contains("fn la_gs_solve"),
        "C1086: Should contain la_gs_solve function"
    );
}

/// C1087: Conjugate gradient method for symmetric positive-definite systems
#[test]
fn c1087_conjugate_gradient() {
    let c_code = r#"
#define LA_CG_N 4

void la_cg_matvec(const double *A, const double *x, double *y, int n) {
    int i, j;
    for (i = 0; i < n; i++) {
        y[i] = 0.0;
        for (j = 0; j < n; j++) {
            y[i] += A[i * n + j] * x[j];
        }
    }
}

double la_cg_dot(const double *a, const double *b, int n) {
    double s = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        s += a[i] * b[i];
    }
    return s;
}

int la_cg_solve(const double *A, const double *b, double *x, int n,
                 int max_iter, double tol) {
    double r[LA_CG_N], p[LA_CG_N], Ap[LA_CG_N];
    int i, iter;
    for (i = 0; i < n; i++) {
        x[i] = 0.0;
        r[i] = b[i];
        p[i] = b[i];
    }
    double rsold = la_cg_dot(r, r, n);
    for (iter = 0; iter < max_iter; iter++) {
        la_cg_matvec(A, p, Ap, n);
        double pAp = la_cg_dot(p, Ap, n);
        if (pAp < 1e-30 && pAp > -1e-30) break;
        double alpha = rsold / pAp;
        for (i = 0; i < n; i++) {
            x[i] += alpha * p[i];
            r[i] -= alpha * Ap[i];
        }
        double rsnew = la_cg_dot(r, r, n);
        if (rsnew < tol * tol) return iter + 1;
        double beta = rsnew / rsold;
        for (i = 0; i < n; i++) {
            p[i] = r[i] + beta * p[i];
        }
        rsold = rsnew;
    }
    return max_iter;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1087: Conjugate gradient should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1087: empty output");
    assert!(
        code.contains("fn la_cg_solve"),
        "C1087: Should contain la_cg_solve function"
    );
}

/// C1088: Sparse matrix-vector multiply in CSR (Compressed Sparse Row) format
#[test]
fn c1088_sparse_csr_multiply() {
    let c_code = r#"
#define LA_CSR_MAX_NNZ 64
#define LA_CSR_MAX_DIM 16

typedef struct {
    double values[LA_CSR_MAX_NNZ];
    int col_idx[LA_CSR_MAX_NNZ];
    int row_ptr[LA_CSR_MAX_DIM + 1];
    int nrows;
    int ncols;
    int nnz;
} la_csr_t;

void la_csr_init(la_csr_t *m, int nrows, int ncols) {
    int i;
    m->nrows = nrows;
    m->ncols = ncols;
    m->nnz = 0;
    for (i = 0; i <= nrows; i++) {
        m->row_ptr[i] = 0;
    }
}

void la_csr_spmv(const la_csr_t *A, const double *x, double *y) {
    int i, j;
    for (i = 0; i < A->nrows; i++) {
        y[i] = 0.0;
        for (j = A->row_ptr[i]; j < A->row_ptr[i + 1]; j++) {
            y[i] += A->values[j] * x[A->col_idx[j]];
        }
    }
}

double la_csr_dot_row(const la_csr_t *A, int row, const double *x) {
    double sum = 0.0;
    int j;
    for (j = A->row_ptr[row]; j < A->row_ptr[row + 1]; j++) {
        sum += A->values[j] * x[A->col_idx[j]];
    }
    return sum;
}

int la_csr_row_nnz(const la_csr_t *A, int row) {
    return A->row_ptr[row + 1] - A->row_ptr[row];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1088: Sparse CSR multiply should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1088: empty output");
    assert!(
        code.contains("fn la_csr_spmv"),
        "C1088: Should contain la_csr_spmv function"
    );
}

/// C1089: Matrix transpose (in-place for square, out-of-place for general)
#[test]
fn c1089_matrix_transpose() {
    let c_code = r#"
#define LA_TR_N 4

void la_transpose_square(double *A, int n) {
    int i, j;
    for (i = 0; i < n; i++) {
        for (j = i + 1; j < n; j++) {
            double tmp = A[i * n + j];
            A[i * n + j] = A[j * n + i];
            A[j * n + i] = tmp;
        }
    }
}

void la_transpose_general(const double *A, double *B, int rows, int cols) {
    int i, j;
    for (i = 0; i < rows; i++) {
        for (j = 0; j < cols; j++) {
            B[j * rows + i] = A[i * cols + j];
        }
    }
}

int la_transpose_is_symmetric(const double *A, int n, double tol) {
    int i, j;
    for (i = 0; i < n; i++) {
        for (j = i + 1; j < n; j++) {
            double diff = A[i * n + j] - A[j * n + i];
            if (diff < 0.0) diff = -diff;
            if (diff > tol) return 0;
        }
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1089: Matrix transpose should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1089: empty output");
    assert!(
        code.contains("fn la_transpose_square"),
        "C1089: Should contain la_transpose_square function"
    );
}

/// C1090: Strassen-style multiplication (simplified recursive structure)
#[test]
fn c1090_strassen_multiply() {
    let c_code = r#"
#define LA_STR_N 4

void la_str_add(const double *A, const double *B, double *C, int n) {
    int i;
    for (i = 0; i < n * n; i++) {
        C[i] = A[i] + B[i];
    }
}

void la_str_sub(const double *A, const double *B, double *C, int n) {
    int i;
    for (i = 0; i < n * n; i++) {
        C[i] = A[i] - B[i];
    }
}

void la_str_naive_mul(const double *A, const double *B, double *C, int n) {
    int i, j, k;
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            C[i * n + j] = 0.0;
            for (k = 0; k < n; k++) {
                C[i * n + j] += A[i * n + k] * B[k * n + j];
            }
        }
    }
}

void la_str_multiply(const double *A, const double *B, double *C, int n) {
    double temp1[LA_STR_N * LA_STR_N];
    double temp2[LA_STR_N * LA_STR_N];
    if (n <= 2) {
        la_str_naive_mul(A, B, C, n);
        return;
    }
    la_str_naive_mul(A, B, C, n);
}

void la_str_multiply_add(const double *A, const double *B, double *C,
                          double alpha, int n) {
    double temp[LA_STR_N * LA_STR_N];
    int i;
    la_str_multiply(A, B, temp, n);
    for (i = 0; i < n * n; i++) {
        C[i] += alpha * temp[i];
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1090: Strassen multiply should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1090: empty output");
    assert!(
        code.contains("fn la_str_multiply"),
        "C1090: Should contain la_str_multiply function"
    );
}

// ============================================================================
// C1091-C1095: Vector & Element-Wise Operations
// ============================================================================

/// C1091: Vector dot product with accumulator variants
#[test]
fn c1091_vector_dot_product() {
    let c_code = r#"
#define LA_DOT_N 64

double la_dot_basic(const double *a, const double *b, int n) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += a[i] * b[i];
    }
    return sum;
}

double la_dot_compensated(const double *a, const double *b, int n) {
    double sum = 0.0;
    double comp = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        double prod = a[i] * b[i];
        double y = prod - comp;
        double t = sum + y;
        comp = (t - sum) - y;
        sum = t;
    }
    return sum;
}

void la_dot_axpy(double *y, double alpha, const double *x, int n) {
    int i;
    for (i = 0; i < n; i++) {
        y[i] += alpha * x[i];
    }
}

double la_dot_weighted(const double *a, const double *b, const double *w, int n) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += w[i] * a[i] * b[i];
    }
    return sum;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1091: Vector dot product should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1091: empty output");
    assert!(
        code.contains("fn la_dot_basic"),
        "C1091: Should contain la_dot_basic function"
    );
}

/// C1092: 3D cross product and related vector operations
#[test]
fn c1092_cross_product_3d() {
    let c_code = r#"
typedef struct {
    double x;
    double y;
    double z;
} la_vec3_t;

la_vec3_t la_cross(la_vec3_t a, la_vec3_t b) {
    la_vec3_t result;
    result.x = a.y * b.z - a.z * b.y;
    result.y = a.z * b.x - a.x * b.z;
    result.z = a.x * b.y - a.y * b.x;
    return result;
}

double la_vec3_dot(la_vec3_t a, la_vec3_t b) {
    return a.x * b.x + a.y * b.y + a.z * b.z;
}

double la_vec3_length_sq(la_vec3_t v) {
    return v.x * v.x + v.y * v.y + v.z * v.z;
}

double la_vec3_triple(la_vec3_t a, la_vec3_t b, la_vec3_t c) {
    la_vec3_t cross = la_cross(b, c);
    return la_vec3_dot(a, cross);
}

la_vec3_t la_vec3_scale(la_vec3_t v, double s) {
    la_vec3_t result;
    result.x = v.x * s;
    result.y = v.y * s;
    result.z = v.z * s;
    return result;
}

la_vec3_t la_vec3_add(la_vec3_t a, la_vec3_t b) {
    la_vec3_t result;
    result.x = a.x + b.x;
    result.y = a.y + b.y;
    result.z = a.z + b.z;
    return result;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1092: Cross product 3D should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1092: empty output");
    assert!(
        code.contains("fn la_cross"),
        "C1092: Should contain la_cross function"
    );
}

/// C1093: Matrix trace computation
#[test]
fn c1093_matrix_trace() {
    let c_code = r#"
#define LA_TRACE_N 8

double la_trace(const double *A, int n) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += A[i * n + i];
    }
    return sum;
}

double la_trace_product(const double *A, const double *B, int n) {
    double sum = 0.0;
    int i, j;
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            sum += A[i * n + j] * B[j * n + i];
        }
    }
    return sum;
}

double la_trace_squared(const double *A, int n) {
    double sum = 0.0;
    int i, j;
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            sum += A[i * n + j] * A[j * n + i];
        }
    }
    return sum;
}

void la_trace_subtract_identity(double *A, int n, double lambda) {
    int i;
    for (i = 0; i < n; i++) {
        A[i * n + i] -= lambda;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1093: Matrix trace should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1093: empty output");
    assert!(
        code.contains("fn la_trace"),
        "C1093: Should contain la_trace function"
    );
}

/// C1094: Frobenius norm computation
#[test]
fn c1094_frobenius_norm() {
    let c_code = r#"
#define LA_FROB_N 4

double la_frob_sqrt(double x) {
    double g = x * 0.5;
    int i;
    if (x <= 0.0) return 0.0;
    for (i = 0; i < 50; i++) {
        g = 0.5 * (g + x / g);
    }
    return g;
}

double la_frob_norm(const double *A, int rows, int cols) {
    double sum = 0.0;
    int i, j;
    for (i = 0; i < rows; i++) {
        for (j = 0; j < cols; j++) {
            double v = A[i * cols + j];
            sum += v * v;
        }
    }
    return la_frob_sqrt(sum);
}

double la_frob_norm_diff(const double *A, const double *B, int rows, int cols) {
    double sum = 0.0;
    int i, j;
    for (i = 0; i < rows; i++) {
        for (j = 0; j < cols; j++) {
            double d = A[i * cols + j] - B[i * cols + j];
            sum += d * d;
        }
    }
    return la_frob_sqrt(sum);
}

double la_frob_infinity_norm(const double *A, int rows, int cols) {
    double max_row = 0.0;
    int i, j;
    for (i = 0; i < rows; i++) {
        double row_sum = 0.0;
        for (j = 0; j < cols; j++) {
            double v = A[i * cols + j];
            if (v < 0.0) v = -v;
            row_sum += v;
        }
        if (row_sum > max_row) max_row = row_sum;
    }
    return max_row;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1094: Frobenius norm should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1094: empty output");
    assert!(
        code.contains("fn la_frob_norm"),
        "C1094: Should contain la_frob_norm function"
    );
}

/// C1095: Hadamard (element-wise) product
#[test]
fn c1095_hadamard_product() {
    let c_code = r#"
#define LA_HAD_N 4

void la_hadamard(const double *A, const double *B, double *C, int rows, int cols) {
    int i, j;
    for (i = 0; i < rows; i++) {
        for (j = 0; j < cols; j++) {
            C[i * cols + j] = A[i * cols + j] * B[i * cols + j];
        }
    }
}

void la_hadamard_inplace(double *A, const double *B, int rows, int cols) {
    int i, j;
    for (i = 0; i < rows; i++) {
        for (j = 0; j < cols; j++) {
            A[i * cols + j] *= B[i * cols + j];
        }
    }
}

double la_hadamard_sum(const double *A, const double *B, int rows, int cols) {
    double sum = 0.0;
    int i, j;
    for (i = 0; i < rows; i++) {
        for (j = 0; j < cols; j++) {
            sum += A[i * cols + j] * B[i * cols + j];
        }
    }
    return sum;
}

void la_hadamard_div(const double *A, const double *B, double *C,
                      int rows, int cols) {
    int i, j;
    for (i = 0; i < rows; i++) {
        for (j = 0; j < cols; j++) {
            double denom = B[i * cols + j];
            if (denom > 1e-15 || denom < -1e-15) {
                C[i * cols + j] = A[i * cols + j] / denom;
            } else {
                C[i * cols + j] = 0.0;
            }
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1095: Hadamard product should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1095: empty output");
    assert!(
        code.contains("fn la_hadamard"),
        "C1095: Should contain la_hadamard function"
    );
}

// ============================================================================
// C1096-C1100: Advanced Linear Algebra
// ============================================================================

/// C1096: Kronecker product of two matrices
#[test]
fn c1096_kronecker_product() {
    let c_code = r#"
#define LA_KRON_N 2
#define LA_KRON_OUT (LA_KRON_N * LA_KRON_N)

void la_kronecker(const double *A, int ar, int ac,
                   const double *B, int br, int bc,
                   double *C) {
    int i, j, k, l;
    int cr = ar * br;
    for (i = 0; i < ar; i++) {
        for (j = 0; j < ac; j++) {
            double a_ij = A[i * ac + j];
            for (k = 0; k < br; k++) {
                for (l = 0; l < bc; l++) {
                    int row = i * br + k;
                    int col = j * bc + l;
                    C[row * (ac * bc) + col] = a_ij * B[k * bc + l];
                }
            }
        }
    }
}

void la_kronecker_identity(const double *A, int ar, int ac, int id_size,
                            double *C) {
    int i, j, k;
    int out_cols = ac * id_size;
    int total = ar * id_size * out_cols;
    for (i = 0; i < total; i++) {
        C[i] = 0.0;
    }
    for (i = 0; i < ar; i++) {
        for (j = 0; j < ac; j++) {
            double a_ij = A[i * ac + j];
            for (k = 0; k < id_size; k++) {
                int row = i * id_size + k;
                int col = j * id_size + k;
                C[row * out_cols + col] = a_ij;
            }
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1096: Kronecker product should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1096: empty output");
    assert!(
        code.contains("fn la_kronecker"),
        "C1096: Should contain la_kronecker function"
    );
}

/// C1097: Least squares regression via normal equations
#[test]
fn c1097_least_squares_regression() {
    let c_code = r#"
#define LA_LS_N 4
#define LA_LS_M 8

void la_ls_ata(const double *A, double *AtA, int m, int n) {
    int i, j, k;
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            double sum = 0.0;
            for (k = 0; k < m; k++) {
                sum += A[k * n + i] * A[k * n + j];
            }
            AtA[i * n + j] = sum;
        }
    }
}

void la_ls_atb(const double *A, const double *b, double *Atb, int m, int n) {
    int i, k;
    for (i = 0; i < n; i++) {
        double sum = 0.0;
        for (k = 0; k < m; k++) {
            sum += A[k * n + i] * b[k];
        }
        Atb[i] = sum;
    }
}

int la_ls_solve_normal(const double *A, const double *b, double *x, int m, int n) {
    double AtA[LA_LS_N * LA_LS_N];
    double Atb[LA_LS_N];
    int i, j, k;
    la_ls_ata(A, AtA, m, n);
    la_ls_atb(A, b, Atb, m, n);
    for (k = 0; k < n; k++) {
        int pivot = k;
        double max_v = AtA[k * n + k];
        if (max_v < 0.0) max_v = -max_v;
        for (i = k + 1; i < n; i++) {
            double v = AtA[i * n + k];
            if (v < 0.0) v = -v;
            if (v > max_v) { max_v = v; pivot = i; }
        }
        if (max_v < 1e-15) return -1;
        if (pivot != k) {
            for (j = 0; j < n; j++) {
                double tmp = AtA[k * n + j];
                AtA[k * n + j] = AtA[pivot * n + j];
                AtA[pivot * n + j] = tmp;
            }
            double tmp = Atb[k];
            Atb[k] = Atb[pivot];
            Atb[pivot] = tmp;
        }
        for (i = k + 1; i < n; i++) {
            double factor = AtA[i * n + k] / AtA[k * n + k];
            for (j = k; j < n; j++) {
                AtA[i * n + j] -= factor * AtA[k * n + j];
            }
            Atb[i] -= factor * Atb[k];
        }
    }
    for (i = n - 1; i >= 0; i--) {
        double sum = 0.0;
        for (j = i + 1; j < n; j++) {
            sum += AtA[i * n + j] * x[j];
        }
        x[i] = (Atb[i] - sum) / AtA[i * n + i];
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1097: Least squares regression should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1097: empty output");
    assert!(
        code.contains("fn la_ls_solve_normal"),
        "C1097: Should contain la_ls_solve_normal function"
    );
}

/// C1098: Matrix exponentiation via repeated squaring
#[test]
fn c1098_matrix_exponentiation() {
    let c_code = r#"
#define LA_EXP_N 4

void la_exp_identity(double *I, int n) {
    int i, j;
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            I[i * n + j] = (i == j) ? 1.0 : 0.0;
        }
    }
}

void la_exp_copy(const double *src, double *dst, int n) {
    int i;
    for (i = 0; i < n * n; i++) {
        dst[i] = src[i];
    }
}

void la_exp_mul(const double *A, const double *B, double *C, int n) {
    int i, j, k;
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            double sum = 0.0;
            for (k = 0; k < n; k++) {
                sum += A[i * n + k] * B[k * n + j];
            }
            C[i * n + j] = sum;
        }
    }
}

void la_exp_power(const double *A, double *result, int n, int p) {
    double base[LA_EXP_N * LA_EXP_N];
    double temp[LA_EXP_N * LA_EXP_N];
    la_exp_identity(result, n);
    la_exp_copy(A, base, n);
    while (p > 0) {
        if (p % 2 == 1) {
            la_exp_mul(result, base, temp, n);
            la_exp_copy(temp, result, n);
        }
        la_exp_mul(base, base, temp, n);
        la_exp_copy(temp, base, n);
        p = p / 2;
    }
}

void la_exp_scale(double *A, double scalar, int n) {
    int i;
    for (i = 0; i < n * n; i++) {
        A[i] *= scalar;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1098: Matrix exponentiation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1098: empty output");
    assert!(
        code.contains("fn la_exp_power"),
        "C1098: Should contain la_exp_power function"
    );
}

/// C1099: Gram matrix computation (A^T * A)
#[test]
fn c1099_gram_matrix() {
    let c_code = r#"
#define LA_GRAM_M 8
#define LA_GRAM_N 4

void la_gram_compute(const double *A, double *G, int m, int n) {
    int i, j, k;
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            double sum = 0.0;
            for (k = 0; k < m; k++) {
                sum += A[k * n + i] * A[k * n + j];
            }
            G[i * n + j] = sum;
        }
    }
}

int la_gram_is_positive_definite(const double *G, int n) {
    double row_sum;
    int i, j;
    for (i = 0; i < n; i++) {
        if (G[i * n + i] <= 0.0) return 0;
        row_sum = 0.0;
        for (j = 0; j < n; j++) {
            if (j != i) {
                double v = G[i * n + j];
                if (v < 0.0) v = -v;
                row_sum += v;
            }
        }
        if (G[i * n + i] <= row_sum) return 0;
    }
    return 1;
}

void la_gram_kernel_linear(const double *X, double *K, int m, int n) {
    la_gram_compute(X, K, m, n);
}

void la_gram_kernel_poly(const double *X, double *K, int m, int n,
                          double c, int degree) {
    int i, j, k, d;
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            double dot = 0.0;
            for (k = 0; k < m; k++) {
                dot += X[k * n + i] * X[k * n + j];
            }
            double val = dot + c;
            double result = 1.0;
            for (d = 0; d < degree; d++) {
                result *= val;
            }
            K[i * n + j] = result;
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1099: Gram matrix should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1099: empty output");
    assert!(
        code.contains("fn la_gram_compute"),
        "C1099: Should contain la_gram_compute function"
    );
}

/// C1100: Householder reflection for QR factorization
#[test]
fn c1100_householder_reflection() {
    let c_code = r#"
#define LA_HH_N 4

double la_hh_sqrt(double x) {
    double g = x * 0.5;
    int i;
    if (x <= 0.0) return 0.0;
    for (i = 0; i < 50; i++) {
        g = 0.5 * (g + x / g);
    }
    return g;
}

double la_hh_norm(const double *v, int n) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += v[i] * v[i];
    }
    return la_hh_sqrt(sum);
}

void la_hh_compute_reflector(const double *x, double *v, int n) {
    int i;
    double norm_x = la_hh_norm(x, n);
    double sign = (x[0] >= 0.0) ? 1.0 : -1.0;
    for (i = 0; i < n; i++) {
        v[i] = x[i];
    }
    v[0] += sign * norm_x;
    double norm_v = la_hh_norm(v, n);
    if (norm_v > 1e-15) {
        for (i = 0; i < n; i++) {
            v[i] /= norm_v;
        }
    }
}

void la_hh_apply(double *A, const double *v, int m, int n,
                  int start_row, int start_col) {
    int i, j;
    int sub_m = m - start_row;
    double tau;
    for (j = start_col; j < n; j++) {
        tau = 0.0;
        for (i = 0; i < sub_m; i++) {
            tau += v[i] * A[(start_row + i) * n + j];
        }
        tau *= 2.0;
        for (i = 0; i < sub_m; i++) {
            A[(start_row + i) * n + j] -= tau * v[i];
        }
    }
}

void la_hh_qr(double *A, double *R, int m, int n) {
    double v[LA_HH_N];
    double col[LA_HH_N];
    int k, i;
    for (i = 0; i < m * n; i++) {
        R[i] = A[i];
    }
    for (k = 0; k < n; k++) {
        int sub_len = m - k;
        for (i = 0; i < sub_len; i++) {
            col[i] = R[(k + i) * n + k];
        }
        la_hh_compute_reflector(col, v, sub_len);
        la_hh_apply(R, v, m, n, k, k);
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1100: Householder reflection should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1100: empty output");
    assert!(
        code.contains("fn la_hh_qr"),
        "C1100: Should contain la_hh_qr function"
    );
}
