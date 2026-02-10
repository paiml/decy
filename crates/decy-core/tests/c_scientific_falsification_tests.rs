//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C451-C475: Scientific and Numerical Computing patterns -- the kind of C code
//! found in numerical libraries, simulation software, and scientific computing.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world numerical programming patterns commonly
//! found in LAPACK, BLAS, GSL, FFTW, NumPy (C backend), and similar
//! scientific computing libraries -- all expressed as valid C99.
//!
//! Organization:
//! - C451-C455: Linear algebra fundamentals (matvec, Gauss elim, LU, Jacobi, CSR)
//! - C456-C460: Root finding and integration (Newton, RK4, trapezoid, Simpson, Horner)
//! - C461-C465: Interpolation and decomposition (Lagrange, least squares, QR, power iter, Monte Carlo)
//! - C466-C470: PDEs and transforms (heat eq, FFT, Cholesky, bisection, cubic spline)
//! - C471-C475: Advanced solvers (Gauss-Seidel, SVD 2x2, central diff, Adams-Bashforth, CG)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C451-C455: Linear Algebra Fundamentals
// ============================================================================

#[test]
fn c451_dense_matrix_vector_multiply() {
    let c_code = r#"
typedef struct {
    double data[16];
    int rows;
    int cols;
} matrix_t;

typedef struct {
    double data[4];
    int len;
} vector_t;

void mat_vec_mul(const matrix_t *A, const vector_t *x, vector_t *y) {
    int i, j;
    for (i = 0; i < A->rows; i++) {
        y->data[i] = 0.0;
        for (j = 0; j < A->cols; j++) {
            y->data[i] += A->data[i * A->cols + j] * x->data[j];
        }
    }
    y->len = A->rows;
}

double vec_dot(const vector_t *a, const vector_t *b) {
    double sum = 0.0;
    int i;
    for (i = 0; i < a->len; i++) {
        sum += a->data[i] * b->data[i];
    }
    return sum;
}

double vec_norm(const vector_t *v) {
    double sum = 0.0;
    int i;
    for (i = 0; i < v->len; i++) {
        sum += v->data[i] * v->data[i];
    }
    return sum;
}

void vec_scale(vector_t *v, double alpha) {
    int i;
    for (i = 0; i < v->len; i++) {
        v->data[i] *= alpha;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C451: Dense matrix-vector multiply should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C451: empty output");
    assert!(
        code.contains("fn mat_vec_mul"),
        "C451: Should contain mat_vec_mul function"
    );
    assert!(
        code.contains("fn vec_dot"),
        "C451: Should contain vec_dot function"
    );
}

#[test]
fn c452_gaussian_elimination_partial_pivoting() {
    let c_code = r#"
typedef struct {
    double a[16];
    double b[4];
    double x[4];
    int n;
} linear_system_t;

void gauss_swap_rows(linear_system_t *sys, int r1, int r2) {
    int j;
    double tmp;
    for (j = 0; j < sys->n; j++) {
        tmp = sys->a[r1 * sys->n + j];
        sys->a[r1 * sys->n + j] = sys->a[r2 * sys->n + j];
        sys->a[r2 * sys->n + j] = tmp;
    }
    tmp = sys->b[r1];
    sys->b[r1] = sys->b[r2];
    sys->b[r2] = tmp;
}

int gauss_find_pivot(const linear_system_t *sys, int col) {
    int max_row = col;
    double max_val = sys->a[col * sys->n + col];
    int i;
    if (max_val < 0.0) max_val = -max_val;
    for (i = col + 1; i < sys->n; i++) {
        double val = sys->a[i * sys->n + col];
        if (val < 0.0) val = -val;
        if (val > max_val) {
            max_val = val;
            max_row = i;
        }
    }
    return max_row;
}

int gauss_eliminate(linear_system_t *sys) {
    int i, j, k;
    for (k = 0; k < sys->n; k++) {
        int pivot = gauss_find_pivot(sys, k);
        if (pivot != k) {
            gauss_swap_rows(sys, k, pivot);
        }
        if (sys->a[k * sys->n + k] == 0.0) return -1;
        for (i = k + 1; i < sys->n; i++) {
            double factor = sys->a[i * sys->n + k] / sys->a[k * sys->n + k];
            for (j = k; j < sys->n; j++) {
                sys->a[i * sys->n + j] -= factor * sys->a[k * sys->n + j];
            }
            sys->b[i] -= factor * sys->b[k];
        }
    }
    return 0;
}

void gauss_back_substitute(linear_system_t *sys) {
    int i, j;
    for (i = sys->n - 1; i >= 0; i--) {
        sys->x[i] = sys->b[i];
        for (j = i + 1; j < sys->n; j++) {
            sys->x[i] -= sys->a[i * sys->n + j] * sys->x[j];
        }
        sys->x[i] /= sys->a[i * sys->n + i];
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C452: Gaussian elimination should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C452: empty output");
    assert!(
        code.contains("fn gauss_eliminate"),
        "C452: Should contain gauss_eliminate function"
    );
    assert!(
        code.contains("fn gauss_back_substitute"),
        "C452: Should contain gauss_back_substitute function"
    );
}

#[test]
fn c453_lu_decomposition_doolittle() {
    let c_code = r#"
typedef struct {
    double L[16];
    double U[16];
    int n;
} lu_decomp_t;

void lu_init(lu_decomp_t *lu, int n) {
    int i, j;
    lu->n = n;
    for (i = 0; i < n * n; i++) {
        lu->L[i] = 0.0;
        lu->U[i] = 0.0;
    }
    for (i = 0; i < n; i++) {
        lu->L[i * n + i] = 1.0;
    }
}

int lu_decompose(lu_decomp_t *lu, const double *A) {
    int i, j, k;
    int n = lu->n;
    for (j = 0; j < n; j++) {
        for (i = 0; i <= j; i++) {
            double sum = 0.0;
            for (k = 0; k < i; k++) {
                sum += lu->L[i * n + k] * lu->U[k * n + j];
            }
            lu->U[i * n + j] = A[i * n + j] - sum;
        }
        for (i = j + 1; i < n; i++) {
            double sum = 0.0;
            for (k = 0; k < j; k++) {
                sum += lu->L[i * n + k] * lu->U[k * n + j];
            }
            if (lu->U[j * n + j] == 0.0) return -1;
            lu->L[i * n + j] = (A[i * n + j] - sum) / lu->U[j * n + j];
        }
    }
    return 0;
}

void lu_solve(const lu_decomp_t *lu, const double *b, double *x) {
    double y[4];
    int i, j;
    int n = lu->n;
    for (i = 0; i < n; i++) {
        y[i] = b[i];
        for (j = 0; j < i; j++) {
            y[i] -= lu->L[i * n + j] * y[j];
        }
    }
    for (i = n - 1; i >= 0; i--) {
        x[i] = y[i];
        for (j = i + 1; j < n; j++) {
            x[i] -= lu->U[i * n + j] * x[j];
        }
        x[i] /= lu->U[i * n + i];
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C453: LU decomposition should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C453: empty output");
    assert!(
        code.contains("fn lu_decompose"),
        "C453: Should contain lu_decompose function"
    );
    assert!(
        code.contains("fn lu_solve"),
        "C453: Should contain lu_solve function"
    );
}

#[test]
fn c454_jacobi_iterative_solver() {
    let c_code = r#"
typedef struct {
    double A[16];
    double b[4];
    double x[4];
    double x_new[4];
    int n;
    int max_iter;
    double tol;
} jacobi_solver_t;

void jacobi_init(jacobi_solver_t *sol, int n) {
    int i;
    sol->n = n;
    sol->max_iter = 1000;
    sol->tol = 1e-8;
    for (i = 0; i < n; i++) {
        sol->x[i] = 0.0;
        sol->x_new[i] = 0.0;
    }
}

double jacobi_residual(const jacobi_solver_t *sol) {
    double max_diff = 0.0;
    int i;
    for (i = 0; i < sol->n; i++) {
        double diff = sol->x_new[i] - sol->x[i];
        if (diff < 0.0) diff = -diff;
        if (diff > max_diff) max_diff = diff;
    }
    return max_diff;
}

int jacobi_iterate(jacobi_solver_t *sol) {
    int i, j, iter;
    int n = sol->n;
    for (iter = 0; iter < sol->max_iter; iter++) {
        for (i = 0; i < n; i++) {
            double sigma = 0.0;
            for (j = 0; j < n; j++) {
                if (j != i) {
                    sigma += sol->A[i * n + j] * sol->x[j];
                }
            }
            sol->x_new[i] = (sol->b[i] - sigma) / sol->A[i * n + i];
        }
        if (jacobi_residual(sol) < sol->tol) return iter + 1;
        for (i = 0; i < n; i++) {
            sol->x[i] = sol->x_new[i];
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C454: Jacobi iterative solver should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C454: empty output");
    assert!(
        code.contains("fn jacobi_iterate"),
        "C454: Should contain jacobi_iterate function"
    );
    assert!(
        code.contains("fn jacobi_residual"),
        "C454: Should contain jacobi_residual function"
    );
}

#[test]
fn c455_sparse_matrix_csr_format() {
    let c_code = r#"
typedef struct {
    double values[32];
    int col_idx[32];
    int row_ptr[8];
    int nnz;
    int nrows;
    int ncols;
} csr_matrix_t;

void csr_init(csr_matrix_t *m, int nrows, int ncols) {
    int i;
    m->nrows = nrows;
    m->ncols = ncols;
    m->nnz = 0;
    for (i = 0; i <= nrows; i++) {
        m->row_ptr[i] = 0;
    }
}

void csr_matvec(const csr_matrix_t *A, const double *x, double *y) {
    int i, j;
    for (i = 0; i < A->nrows; i++) {
        y[i] = 0.0;
        for (j = A->row_ptr[i]; j < A->row_ptr[i + 1]; j++) {
            y[i] += A->values[j] * x[A->col_idx[j]];
        }
    }
}

double csr_row_norm(const csr_matrix_t *A, int row) {
    double sum = 0.0;
    int j;
    for (j = A->row_ptr[row]; j < A->row_ptr[row + 1]; j++) {
        double v = A->values[j];
        sum += v * v;
    }
    return sum;
}

int csr_count_row_nonzeros(const csr_matrix_t *A, int row) {
    return A->row_ptr[row + 1] - A->row_ptr[row];
}

double csr_get(const csr_matrix_t *A, int row, int col) {
    int j;
    for (j = A->row_ptr[row]; j < A->row_ptr[row + 1]; j++) {
        if (A->col_idx[j] == col) {
            return A->values[j];
        }
    }
    return 0.0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C455: Sparse CSR matrix should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C455: empty output");
    assert!(
        code.contains("fn csr_matvec"),
        "C455: Should contain csr_matvec function"
    );
    assert!(
        code.contains("fn csr_get"),
        "C455: Should contain csr_get function"
    );
}

// ============================================================================
// C456-C460: Root Finding and Integration
// ============================================================================

#[test]
fn c456_newton_raphson_root_finding() {
    let c_code = r#"
typedef struct {
    double x0;
    double tol;
    int max_iter;
    double root;
    int converged;
} newton_state_t;

double newton_func(double x) {
    return x * x * x - 2.0 * x - 5.0;
}

double newton_deriv(double x) {
    return 3.0 * x * x - 2.0;
}

void newton_init(newton_state_t *st, double x0) {
    st->x0 = x0;
    st->tol = 1e-10;
    st->max_iter = 100;
    st->root = 0.0;
    st->converged = 0;
}

int newton_solve(newton_state_t *st) {
    double x = st->x0;
    int i;
    for (i = 0; i < st->max_iter; i++) {
        double fx = newton_func(x);
        double fpx = newton_deriv(x);
        if (fpx == 0.0) return -1;
        double dx = fx / fpx;
        x = x - dx;
        if (dx < 0.0) dx = -dx;
        if (dx < st->tol) {
            st->root = x;
            st->converged = 1;
            return i + 1;
        }
    }
    st->root = x;
    st->converged = 0;
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C456: Newton-Raphson root finding should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C456: empty output");
    assert!(
        code.contains("fn newton_solve"),
        "C456: Should contain newton_solve function"
    );
    assert!(
        code.contains("fn newton_func"),
        "C456: Should contain newton_func function"
    );
}

#[test]
fn c457_runge_kutta_4th_order() {
    let c_code = r#"
typedef struct {
    double t;
    double y;
    double h;
    double t_end;
} rk4_state_t;

double rk4_dydt(double t, double y) {
    return -2.0 * t * y;
}

void rk4_step(rk4_state_t *st) {
    double k1, k2, k3, k4;
    double h = st->h;
    double t = st->t;
    double y = st->y;

    k1 = h * rk4_dydt(t, y);
    k2 = h * rk4_dydt(t + h / 2.0, y + k1 / 2.0);
    k3 = h * rk4_dydt(t + h / 2.0, y + k2 / 2.0);
    k4 = h * rk4_dydt(t + h, y + k3);

    st->y = y + (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0;
    st->t = t + h;
}

int rk4_integrate(rk4_state_t *st, double *results, int max_steps) {
    int steps = 0;
    results[0] = st->y;
    while (st->t < st->t_end && steps < max_steps) {
        rk4_step(st);
        steps++;
        results[steps] = st->y;
    }
    return steps;
}

void rk4_init(rk4_state_t *st, double y0, double h, double t_end) {
    st->t = 0.0;
    st->y = y0;
    st->h = h;
    st->t_end = t_end;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C457: Runge-Kutta 4th order should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C457: empty output");
    assert!(
        code.contains("fn rk4_step"),
        "C457: Should contain rk4_step function"
    );
    assert!(
        code.contains("fn rk4_integrate"),
        "C457: Should contain rk4_integrate function"
    );
}

#[test]
fn c458_trapezoidal_numerical_integration() {
    let c_code = r#"
typedef struct {
    double a;
    double b;
    int n;
} trap_params_t;

double trap_func(double x) {
    return x * x * x + 2.0 * x * x - x + 3.0;
}

double trap_integrate(const trap_params_t *p) {
    double h = (p->b - p->a) / p->n;
    double sum = (trap_func(p->a) + trap_func(p->b)) / 2.0;
    int i;
    for (i = 1; i < p->n; i++) {
        double xi = p->a + i * h;
        sum += trap_func(xi);
    }
    return sum * h;
}

double trap_error_estimate(const trap_params_t *p, double exact) {
    double approx = trap_integrate(p);
    double err = approx - exact;
    if (err < 0.0) err = -err;
    return err;
}

int trap_converge(double a, double b, double tol, double exact) {
    trap_params_t p;
    int n = 4;
    p.a = a;
    p.b = b;
    while (n < 1000000) {
        p.n = n;
        double err = trap_error_estimate(&p, exact);
        if (err < tol) return n;
        n *= 2;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C458: Trapezoidal integration should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C458: empty output");
    assert!(
        code.contains("fn trap_integrate"),
        "C458: Should contain trap_integrate function"
    );
    assert!(
        code.contains("fn trap_converge"),
        "C458: Should contain trap_converge function"
    );
}

#[test]
fn c459_simpsons_rule_integration() {
    let c_code = r#"
typedef struct {
    double a;
    double b;
    int n;
} simpson_params_t;

double simpson_func(double x) {
    return 1.0 / (1.0 + x * x);
}

double simpson_integrate(const simpson_params_t *p) {
    int n = p->n;
    if (n % 2 != 0) n++;
    double h = (p->b - p->a) / n;
    double sum = simpson_func(p->a) + simpson_func(p->b);
    int i;
    for (i = 1; i < n; i++) {
        double xi = p->a + i * h;
        if (i % 2 == 0) {
            sum += 2.0 * simpson_func(xi);
        } else {
            sum += 4.0 * simpson_func(xi);
        }
    }
    return sum * h / 3.0;
}

double simpson_adaptive(double a, double b, double tol) {
    simpson_params_t p;
    p.a = a;
    p.b = b;
    p.n = 2;
    double prev = 0.0;
    int iter;
    for (iter = 0; iter < 20; iter++) {
        double curr = simpson_integrate(&p);
        if (iter > 0) {
            double diff = curr - prev;
            if (diff < 0.0) diff = -diff;
            if (diff < tol) return curr;
        }
        prev = curr;
        p.n *= 2;
    }
    return prev;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C459: Simpson's rule integration should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C459: empty output");
    assert!(
        code.contains("fn simpson_integrate"),
        "C459: Should contain simpson_integrate function"
    );
    assert!(
        code.contains("fn simpson_adaptive"),
        "C459: Should contain simpson_adaptive function"
    );
}

#[test]
fn c460_polynomial_evaluation_horner() {
    let c_code = r#"
typedef struct {
    double coeffs[16];
    int degree;
} polynomial_t;

void poly_init(polynomial_t *p, int degree) {
    int i;
    p->degree = degree;
    for (i = 0; i <= degree; i++) {
        p->coeffs[i] = 0.0;
    }
}

double poly_eval_horner(const polynomial_t *p, double x) {
    double result = p->coeffs[p->degree];
    int i;
    for (i = p->degree - 1; i >= 0; i--) {
        result = result * x + p->coeffs[i];
    }
    return result;
}

double poly_eval_deriv(const polynomial_t *p, double x) {
    double result = p->coeffs[p->degree] * p->degree;
    int i;
    for (i = p->degree - 1; i >= 1; i--) {
        result = result * x + p->coeffs[i] * i;
    }
    return result;
}

void poly_add(const polynomial_t *a, const polynomial_t *b, polynomial_t *result) {
    int max_deg = a->degree > b->degree ? a->degree : b->degree;
    int i;
    poly_init(result, max_deg);
    for (i = 0; i <= a->degree; i++) {
        result->coeffs[i] += a->coeffs[i];
    }
    for (i = 0; i <= b->degree; i++) {
        result->coeffs[i] += b->coeffs[i];
    }
}

double poly_root_newton(const polynomial_t *p, double x0, double tol, int max_iter) {
    double x = x0;
    int i;
    for (i = 0; i < max_iter; i++) {
        double fx = poly_eval_horner(p, x);
        double fpx = poly_eval_deriv(p, x);
        if (fpx == 0.0) break;
        double dx = fx / fpx;
        x = x - dx;
        if (dx < 0.0) dx = -dx;
        if (dx < tol) break;
    }
    return x;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C460: Polynomial Horner's method should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C460: empty output");
    assert!(
        code.contains("fn poly_eval_horner"),
        "C460: Should contain poly_eval_horner function"
    );
    assert!(
        code.contains("fn poly_root_newton"),
        "C460: Should contain poly_root_newton function"
    );
}

// ============================================================================
// C461-C465: Interpolation and Decomposition
// ============================================================================

#[test]
fn c461_lagrange_interpolation() {
    let c_code = r#"
typedef struct {
    double x[16];
    double y[16];
    int n;
} interp_data_t;

void interp_init(interp_data_t *d) {
    d->n = 0;
}

void interp_add_point(interp_data_t *d, double x, double y) {
    if (d->n < 16) {
        d->x[d->n] = x;
        d->y[d->n] = y;
        d->n++;
    }
}

double lagrange_basis(const interp_data_t *d, int j, double x) {
    double basis = 1.0;
    int i;
    for (i = 0; i < d->n; i++) {
        if (i != j) {
            basis *= (x - d->x[i]) / (d->x[j] - d->x[i]);
        }
    }
    return basis;
}

double lagrange_eval(const interp_data_t *d, double x) {
    double result = 0.0;
    int j;
    for (j = 0; j < d->n; j++) {
        result += d->y[j] * lagrange_basis(d, j, x);
    }
    return result;
}

double lagrange_error_bound(const interp_data_t *d, double x, double max_deriv) {
    double product = 1.0;
    int i;
    double factorial = 1.0;
    for (i = 0; i < d->n; i++) {
        product *= (x - d->x[i]);
        factorial *= (i + 1);
    }
    if (product < 0.0) product = -product;
    return max_deriv * product / factorial;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C461: Lagrange interpolation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C461: empty output");
    assert!(
        code.contains("fn lagrange_eval"),
        "C461: Should contain lagrange_eval function"
    );
    assert!(
        code.contains("fn lagrange_basis"),
        "C461: Should contain lagrange_basis function"
    );
}

#[test]
fn c462_least_squares_linear_regression() {
    let c_code = r#"
typedef struct {
    double x[64];
    double y[64];
    int n;
    double slope;
    double intercept;
    double r_squared;
} regression_t;

void reg_init(regression_t *r) {
    r->n = 0;
    r->slope = 0.0;
    r->intercept = 0.0;
    r->r_squared = 0.0;
}

void reg_add_point(regression_t *r, double x, double y) {
    if (r->n < 64) {
        r->x[r->n] = x;
        r->y[r->n] = y;
        r->n++;
    }
}

void reg_fit(regression_t *r) {
    double sum_x = 0.0, sum_y = 0.0;
    double sum_xx = 0.0, sum_xy = 0.0;
    double sum_yy = 0.0;
    int i;
    int n = r->n;
    for (i = 0; i < n; i++) {
        sum_x += r->x[i];
        sum_y += r->y[i];
        sum_xx += r->x[i] * r->x[i];
        sum_xy += r->x[i] * r->y[i];
        sum_yy += r->y[i] * r->y[i];
    }
    double denom = n * sum_xx - sum_x * sum_x;
    if (denom == 0.0) return;
    r->slope = (n * sum_xy - sum_x * sum_y) / denom;
    r->intercept = (sum_y - r->slope * sum_x) / n;
    double ss_res = 0.0;
    double mean_y = sum_y / n;
    double ss_tot = 0.0;
    for (i = 0; i < n; i++) {
        double pred = r->slope * r->x[i] + r->intercept;
        double residual = r->y[i] - pred;
        ss_res += residual * residual;
        double dev = r->y[i] - mean_y;
        ss_tot += dev * dev;
    }
    if (ss_tot > 0.0) {
        r->r_squared = 1.0 - ss_res / ss_tot;
    }
}

double reg_predict(const regression_t *r, double x) {
    return r->slope * x + r->intercept;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C462: Least squares regression should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C462: empty output");
    assert!(
        code.contains("fn reg_fit"),
        "C462: Should contain reg_fit function"
    );
    assert!(
        code.contains("fn reg_predict"),
        "C462: Should contain reg_predict function"
    );
}

#[test]
fn c463_qr_decomposition_gram_schmidt() {
    let c_code = r#"
typedef struct {
    double Q[16];
    double R[16];
    int n;
} qr_decomp_t;

void qr_init(qr_decomp_t *qr, int n) {
    int i;
    qr->n = n;
    for (i = 0; i < n * n; i++) {
        qr->Q[i] = 0.0;
        qr->R[i] = 0.0;
    }
}

double qr_col_dot(const double *A, int n, int col1, int col2) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += A[i * n + col1] * A[i * n + col2];
    }
    return sum;
}

double qr_col_norm(const double *A, int n, int col) {
    return qr_col_dot(A, n, col, col);
}

void qr_decompose(qr_decomp_t *qr, const double *A) {
    int i, j, k;
    int n = qr->n;
    for (j = 0; j < n; j++) {
        for (i = 0; i < n; i++) {
            qr->Q[i * n + j] = A[i * n + j];
        }
        for (k = 0; k < j; k++) {
            double dot = 0.0;
            for (i = 0; i < n; i++) {
                dot += qr->Q[i * n + k] * A[i * n + j];
            }
            qr->R[k * n + j] = dot;
            for (i = 0; i < n; i++) {
                qr->Q[i * n + j] -= dot * qr->Q[i * n + k];
            }
        }
        double norm_sq = 0.0;
        for (i = 0; i < n; i++) {
            norm_sq += qr->Q[i * n + j] * qr->Q[i * n + j];
        }
        double norm = norm_sq;
        if (norm > 0.0) {
            int idx;
            for (idx = 0; idx < 10; idx++) {
                norm = (norm + norm_sq / norm) / 2.0;
            }
        }
        qr->R[j * n + j] = norm;
        if (norm > 1e-14) {
            for (i = 0; i < n; i++) {
                qr->Q[i * n + j] /= norm;
            }
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C463: QR decomposition should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C463: empty output");
    assert!(
        code.contains("fn qr_decompose"),
        "C463: Should contain qr_decompose function"
    );
    assert!(
        code.contains("fn qr_col_dot"),
        "C463: Should contain qr_col_dot function"
    );
}

#[test]
fn c464_power_iteration_eigenvalue() {
    let c_code = r#"
typedef struct {
    double matrix[16];
    double eigvec[4];
    double eigval;
    int n;
    int max_iter;
    double tol;
} power_iter_t;

void power_init(power_iter_t *pi, int n) {
    int i;
    pi->n = n;
    pi->max_iter = 1000;
    pi->tol = 1e-10;
    pi->eigval = 0.0;
    for (i = 0; i < n; i++) {
        pi->eigvec[i] = 1.0;
    }
}

double power_vec_norm(const double *v, int n) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += v[i] * v[i];
    }
    double norm = sum;
    int iter;
    for (iter = 0; iter < 20; iter++) {
        if (norm > 0.0) norm = (norm + sum / norm) / 2.0;
    }
    return norm;
}

void power_normalize(double *v, int n) {
    double norm = power_vec_norm(v, n);
    int i;
    if (norm > 1e-14) {
        for (i = 0; i < n; i++) {
            v[i] /= norm;
        }
    }
}

int power_iterate(power_iter_t *pi) {
    double temp[4];
    int i, j, iter;
    int n = pi->n;
    power_normalize(pi->eigvec, n);
    for (iter = 0; iter < pi->max_iter; iter++) {
        for (i = 0; i < n; i++) {
            temp[i] = 0.0;
            for (j = 0; j < n; j++) {
                temp[i] += pi->matrix[i * n + j] * pi->eigvec[j];
            }
        }
        double new_eigval = power_vec_norm(temp, n);
        power_normalize(temp, n);
        for (i = 0; i < n; i++) {
            pi->eigvec[i] = temp[i];
        }
        double diff = new_eigval - pi->eigval;
        if (diff < 0.0) diff = -diff;
        pi->eigval = new_eigval;
        if (diff < pi->tol) return iter + 1;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C464: Power iteration eigenvalue should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C464: empty output");
    assert!(
        code.contains("fn power_iterate"),
        "C464: Should contain power_iterate function"
    );
    assert!(
        code.contains("fn power_vec_norm"),
        "C464: Should contain power_vec_norm function"
    );
}

#[test]
fn c465_monte_carlo_pi_estimation() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t state;
} lcg_rng_t;

void rng_init(lcg_rng_t *rng, uint32_t seed) {
    rng->state = seed;
}

uint32_t rng_next(lcg_rng_t *rng) {
    rng->state = rng->state * 1103515245 + 12345;
    return (rng->state >> 16) & 0x7FFF;
}

double rng_uniform(lcg_rng_t *rng) {
    return (double)rng_next(rng) / 32767.0;
}

typedef struct {
    int inside;
    int total;
    double pi_estimate;
} monte_carlo_t;

void mc_init(monte_carlo_t *mc) {
    mc->inside = 0;
    mc->total = 0;
    mc->pi_estimate = 0.0;
}

void mc_sample(monte_carlo_t *mc, lcg_rng_t *rng, int num_samples) {
    int i;
    for (i = 0; i < num_samples; i++) {
        double x = rng_uniform(rng);
        double y = rng_uniform(rng);
        if (x * x + y * y <= 1.0) {
            mc->inside++;
        }
        mc->total++;
    }
    mc->pi_estimate = 4.0 * mc->inside / mc->total;
}

double mc_error(const monte_carlo_t *mc) {
    double err = mc->pi_estimate - 3.14159265358979;
    if (err < 0.0) err = -err;
    return err;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C465: Monte Carlo Pi estimation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C465: empty output");
    assert!(
        code.contains("fn mc_sample"),
        "C465: Should contain mc_sample function"
    );
    assert!(
        code.contains("fn rng_uniform"),
        "C465: Should contain rng_uniform function"
    );
}

// ============================================================================
// C466-C470: PDEs and Transforms
// ============================================================================

#[test]
fn c466_finite_difference_heat_equation_1d() {
    let c_code = r#"
typedef struct {
    double u[64];
    double u_new[64];
    int nx;
    double dx;
    double dt;
    double alpha;
} heat1d_t;

void heat1d_init(heat1d_t *h, int nx, double alpha) {
    int i;
    h->nx = nx;
    h->alpha = alpha;
    h->dx = 1.0 / (nx - 1);
    h->dt = 0.4 * h->dx * h->dx / alpha;
    for (i = 0; i < nx; i++) {
        h->u[i] = 0.0;
        h->u_new[i] = 0.0;
    }
}

void heat1d_set_ic(heat1d_t *h, double left, double right) {
    int i;
    h->u[0] = left;
    h->u[h->nx - 1] = right;
    for (i = 1; i < h->nx - 1; i++) {
        h->u[i] = left + (right - left) * i / (h->nx - 1);
    }
}

void heat1d_step(heat1d_t *h) {
    int i;
    double r = h->alpha * h->dt / (h->dx * h->dx);
    h->u_new[0] = h->u[0];
    h->u_new[h->nx - 1] = h->u[h->nx - 1];
    for (i = 1; i < h->nx - 1; i++) {
        h->u_new[i] = h->u[i] + r * (h->u[i + 1] - 2.0 * h->u[i] + h->u[i - 1]);
    }
    for (i = 0; i < h->nx; i++) {
        h->u[i] = h->u_new[i];
    }
}

double heat1d_max_temp(const heat1d_t *h) {
    double max_val = h->u[0];
    int i;
    for (i = 1; i < h->nx; i++) {
        if (h->u[i] > max_val) max_val = h->u[i];
    }
    return max_val;
}

void heat1d_evolve(heat1d_t *h, int num_steps) {
    int s;
    for (s = 0; s < num_steps; s++) {
        heat1d_step(h);
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C466: Finite difference heat equation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C466: empty output");
    assert!(
        code.contains("fn heat1d_step"),
        "C466: Should contain heat1d_step function"
    );
    assert!(
        code.contains("fn heat1d_evolve"),
        "C466: Should contain heat1d_evolve function"
    );
}

#[test]
fn c467_fft_simplified_radix2() {
    let c_code = r#"
typedef struct {
    double real;
    double imag;
} complex_t;

complex_t complex_add(complex_t a, complex_t b) {
    complex_t result;
    result.real = a.real + b.real;
    result.imag = a.imag + b.imag;
    return result;
}

complex_t complex_sub(complex_t a, complex_t b) {
    complex_t result;
    result.real = a.real - b.real;
    result.imag = a.imag - b.imag;
    return result;
}

complex_t complex_mul(complex_t a, complex_t b) {
    complex_t result;
    result.real = a.real * b.real - a.imag * b.imag;
    result.imag = a.real * b.imag + a.imag * b.real;
    return result;
}

double complex_mag(complex_t c) {
    return c.real * c.real + c.imag * c.imag;
}

void bit_reverse_copy(const complex_t *src, complex_t *dst, int n) {
    int i, j;
    for (i = 0; i < n; i++) {
        j = 0;
        int bits = i;
        int log2n = 0;
        int temp = n;
        while (temp > 1) { log2n++; temp /= 2; }
        int b;
        for (b = 0; b < log2n; b++) {
            j = (j << 1) | (bits & 1);
            bits >>= 1;
        }
        dst[j] = src[i];
    }
}

void fft_butterfly(complex_t *data, int n) {
    int s, k, j;
    int m = 1;
    int log2n = 0;
    int temp = n;
    while (temp > 1) { log2n++; temp /= 2; }
    for (s = 1; s <= log2n; s++) {
        m = 1 << s;
        complex_t wm;
        wm.real = 1.0;
        wm.imag = 0.0;
        double angle = -6.28318530718 / m;
        complex_t w_step;
        w_step.real = 1.0;
        w_step.imag = angle;
        for (k = 0; k < n; k += m) {
            complex_t w;
            w.real = 1.0;
            w.imag = 0.0;
            for (j = 0; j < m / 2; j++) {
                complex_t t = complex_mul(w, data[k + j + m / 2]);
                complex_t u = data[k + j];
                data[k + j] = complex_add(u, t);
                data[k + j + m / 2] = complex_sub(u, t);
                w = complex_mul(w, w_step);
            }
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C467: FFT radix-2 should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C467: empty output");
    assert!(
        code.contains("fn fft_butterfly"),
        "C467: Should contain fft_butterfly function"
    );
    assert!(
        code.contains("fn complex_mul"),
        "C467: Should contain complex_mul function"
    );
}

#[test]
fn c468_cholesky_decomposition() {
    let c_code = r#"
typedef struct {
    double L[16];
    int n;
} cholesky_t;

void cholesky_init(cholesky_t *ch, int n) {
    int i;
    ch->n = n;
    for (i = 0; i < n * n; i++) {
        ch->L[i] = 0.0;
    }
}

int cholesky_decompose(cholesky_t *ch, const double *A) {
    int i, j, k;
    int n = ch->n;
    for (j = 0; j < n; j++) {
        double sum = 0.0;
        for (k = 0; k < j; k++) {
            sum += ch->L[j * n + k] * ch->L[j * n + k];
        }
        double diag = A[j * n + j] - sum;
        if (diag <= 0.0) return -1;
        double sqrt_val = diag;
        int iter;
        for (iter = 0; iter < 30; iter++) {
            if (sqrt_val > 0.0) sqrt_val = (sqrt_val + diag / sqrt_val) / 2.0;
        }
        ch->L[j * n + j] = sqrt_val;
        for (i = j + 1; i < n; i++) {
            sum = 0.0;
            for (k = 0; k < j; k++) {
                sum += ch->L[i * n + k] * ch->L[j * n + k];
            }
            ch->L[i * n + j] = (A[i * n + j] - sum) / ch->L[j * n + j];
        }
    }
    return 0;
}

void cholesky_solve(const cholesky_t *ch, const double *b, double *x) {
    double y[4];
    int i, j;
    int n = ch->n;
    for (i = 0; i < n; i++) {
        double sum = 0.0;
        for (j = 0; j < i; j++) {
            sum += ch->L[i * n + j] * y[j];
        }
        y[i] = (b[i] - sum) / ch->L[i * n + i];
    }
    for (i = n - 1; i >= 0; i--) {
        double sum = 0.0;
        for (j = i + 1; j < n; j++) {
            sum += ch->L[j * n + i] * x[j];
        }
        x[i] = (y[i] - sum) / ch->L[i * n + i];
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C468: Cholesky decomposition should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C468: empty output");
    assert!(
        code.contains("fn cholesky_decompose"),
        "C468: Should contain cholesky_decompose function"
    );
    assert!(
        code.contains("fn cholesky_solve"),
        "C468: Should contain cholesky_solve function"
    );
}

#[test]
fn c469_bisection_method_root_finding() {
    let c_code = r#"
typedef struct {
    double a;
    double b;
    double tol;
    int max_iter;
    double root;
    int iterations;
} bisection_t;

double bisect_func(double x) {
    return x * x * x - x - 2.0;
}

void bisect_init(bisection_t *bs, double a, double b) {
    bs->a = a;
    bs->b = b;
    bs->tol = 1e-12;
    bs->max_iter = 100;
    bs->root = 0.0;
    bs->iterations = 0;
}

int bisect_check_bracket(const bisection_t *bs) {
    double fa = bisect_func(bs->a);
    double fb = bisect_func(bs->b);
    return (fa * fb < 0.0) ? 1 : 0;
}

int bisect_solve(bisection_t *bs) {
    double a = bs->a;
    double b = bs->b;
    double fa = bisect_func(a);
    int i;
    if (fa * bisect_func(b) >= 0.0) return -1;
    for (i = 0; i < bs->max_iter; i++) {
        double mid = (a + b) / 2.0;
        double fmid = bisect_func(mid);
        if (fmid == 0.0 || (b - a) / 2.0 < bs->tol) {
            bs->root = mid;
            bs->iterations = i + 1;
            return 0;
        }
        if (fa * fmid < 0.0) {
            b = mid;
        } else {
            a = mid;
            fa = fmid;
        }
    }
    bs->root = (a + b) / 2.0;
    bs->iterations = bs->max_iter;
    return -1;
}

double bisect_convergence_rate(const bisection_t *bs) {
    double interval = bs->b - bs->a;
    int i;
    for (i = 0; i < bs->iterations; i++) {
        interval /= 2.0;
    }
    return interval;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C469: Bisection method should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C469: empty output");
    assert!(
        code.contains("fn bisect_solve"),
        "C469: Should contain bisect_solve function"
    );
    assert!(
        code.contains("fn bisect_check_bracket"),
        "C469: Should contain bisect_check_bracket function"
    );
}

#[test]
fn c470_cubic_spline_interpolation() {
    let c_code = r#"
typedef struct {
    double x[16];
    double y[16];
    double a[16];
    double b[16];
    double c[16];
    double d[16];
    int n;
} cubic_spline_t;

void spline_init(cubic_spline_t *sp) {
    sp->n = 0;
}

void spline_add_point(cubic_spline_t *sp, double x, double y) {
    if (sp->n < 16) {
        sp->x[sp->n] = x;
        sp->y[sp->n] = y;
        sp->n++;
    }
}

void spline_compute(cubic_spline_t *sp) {
    int i;
    int n = sp->n - 1;
    double h[15];
    double alpha[15];
    double l[16];
    double mu[16];
    double z[16];
    for (i = 0; i < n; i++) {
        h[i] = sp->x[i + 1] - sp->x[i];
    }
    for (i = 1; i < n; i++) {
        alpha[i] = (3.0 / h[i]) * (sp->y[i + 1] - sp->y[i])
                 - (3.0 / h[i - 1]) * (sp->y[i] - sp->y[i - 1]);
    }
    l[0] = 1.0; mu[0] = 0.0; z[0] = 0.0;
    for (i = 1; i < n; i++) {
        l[i] = 2.0 * (sp->x[i + 1] - sp->x[i - 1]) - h[i - 1] * mu[i - 1];
        mu[i] = h[i] / l[i];
        z[i] = (alpha[i] - h[i - 1] * z[i - 1]) / l[i];
    }
    l[n] = 1.0; z[n] = 0.0; sp->c[n] = 0.0;
    for (i = n - 1; i >= 0; i--) {
        sp->c[i] = z[i] - mu[i] * sp->c[i + 1];
        sp->b[i] = (sp->y[i + 1] - sp->y[i]) / h[i]
                  - h[i] * (sp->c[i + 1] + 2.0 * sp->c[i]) / 3.0;
        sp->d[i] = (sp->c[i + 1] - sp->c[i]) / (3.0 * h[i]);
        sp->a[i] = sp->y[i];
    }
}

double spline_eval(const cubic_spline_t *sp, double x) {
    int i;
    int seg = 0;
    for (i = 0; i < sp->n - 1; i++) {
        if (x >= sp->x[i] && x <= sp->x[i + 1]) {
            seg = i;
            break;
        }
    }
    double dx = x - sp->x[seg];
    return sp->a[seg] + sp->b[seg] * dx + sp->c[seg] * dx * dx
         + sp->d[seg] * dx * dx * dx;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C470: Cubic spline interpolation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C470: empty output");
    assert!(
        code.contains("fn spline_compute"),
        "C470: Should contain spline_compute function"
    );
    assert!(
        code.contains("fn spline_eval"),
        "C470: Should contain spline_eval function"
    );
}

// ============================================================================
// C471-C475: Advanced Solvers
// ============================================================================

#[test]
fn c471_gauss_seidel_iterative_solver() {
    let c_code = r#"
typedef struct {
    double A[16];
    double b[4];
    double x[4];
    int n;
    double tol;
    int max_iter;
} gauss_seidel_t;

void gs_init(gauss_seidel_t *gs, int n) {
    int i;
    gs->n = n;
    gs->tol = 1e-10;
    gs->max_iter = 10000;
    for (i = 0; i < n; i++) {
        gs->x[i] = 0.0;
    }
}

double gs_compute_residual(const gauss_seidel_t *gs) {
    double max_res = 0.0;
    int i, j;
    int n = gs->n;
    for (i = 0; i < n; i++) {
        double row_sum = 0.0;
        for (j = 0; j < n; j++) {
            row_sum += gs->A[i * n + j] * gs->x[j];
        }
        double res = gs->b[i] - row_sum;
        if (res < 0.0) res = -res;
        if (res > max_res) max_res = res;
    }
    return max_res;
}

int gs_solve(gauss_seidel_t *gs) {
    int i, j, iter;
    int n = gs->n;
    for (iter = 0; iter < gs->max_iter; iter++) {
        for (i = 0; i < n; i++) {
            double sigma = 0.0;
            for (j = 0; j < n; j++) {
                if (j != i) {
                    sigma += gs->A[i * n + j] * gs->x[j];
                }
            }
            if (gs->A[i * n + i] == 0.0) return -2;
            gs->x[i] = (gs->b[i] - sigma) / gs->A[i * n + i];
        }
        if (gs_compute_residual(gs) < gs->tol) return iter + 1;
    }
    return -1;
}

int gs_check_diagonal_dominance(const gauss_seidel_t *gs) {
    int i, j;
    int n = gs->n;
    for (i = 0; i < n; i++) {
        double diag = gs->A[i * n + i];
        if (diag < 0.0) diag = -diag;
        double off_sum = 0.0;
        for (j = 0; j < n; j++) {
            if (j != i) {
                double val = gs->A[i * n + j];
                if (val < 0.0) val = -val;
                off_sum += val;
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
        "C471: Gauss-Seidel solver should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C471: empty output");
    assert!(
        code.contains("fn gs_solve"),
        "C471: Should contain gs_solve function"
    );
    assert!(
        code.contains("fn gs_compute_residual"),
        "C471: Should contain gs_compute_residual function"
    );
}

#[test]
fn c472_svd_2x2_simplified() {
    let c_code = r#"
typedef struct {
    double a11, a12, a21, a22;
} mat2x2_t;

typedef struct {
    mat2x2_t U;
    double s1;
    double s2;
    mat2x2_t V;
} svd2x2_t;

double svd_abs(double x) {
    return x < 0.0 ? -x : x;
}

double svd_sqrt_approx(double x) {
    double r = x;
    int i;
    if (x <= 0.0) return 0.0;
    for (i = 0; i < 30; i++) {
        r = (r + x / r) / 2.0;
    }
    return r;
}

double svd_atan2_approx(double y, double x) {
    if (x == 0.0 && y == 0.0) return 0.0;
    if (x > 0.0) {
        double r = y / x;
        return r / (1.0 + 0.28 * r * r);
    }
    if (y >= 0.0) return 1.5707963 - x / y / (1.0 + 0.28 * x * x / (y * y));
    return -1.5707963 - x / y / (1.0 + 0.28 * x * x / (y * y));
}

void svd2x2_compute(svd2x2_t *svd, const mat2x2_t *A) {
    double e = (A->a11 + A->a22) / 2.0;
    double f = (A->a11 - A->a22) / 2.0;
    double g = (A->a21 + A->a12) / 2.0;
    double h = (A->a21 - A->a12) / 2.0;
    double q = svd_sqrt_approx(e * e + h * h);
    double r = svd_sqrt_approx(f * f + g * g);
    svd->s1 = q + r;
    svd->s2 = q - r;
    if (svd->s2 < 0.0) svd->s2 = -svd->s2;
    double a1 = svd_atan2_approx(g, f);
    double a2 = svd_atan2_approx(h, e);
    double theta = (a2 - a1) / 2.0;
    double phi = (a2 + a1) / 2.0;
    double ct = 1.0 - theta * theta / 2.0;
    double st = theta;
    double cp = 1.0 - phi * phi / 2.0;
    double sp = phi;
    svd->U.a11 = ct * cp - st * sp;
    svd->U.a12 = -st * cp - ct * sp;
    svd->U.a21 = ct * sp + st * cp;
    svd->U.a22 = -st * sp + ct * cp;
    svd->V.a11 = ct * cp + st * sp;
    svd->V.a12 = st * cp - ct * sp;
    svd->V.a21 = -ct * sp + st * cp;
    svd->V.a22 = ct * cp - st * sp;
}

double svd2x2_condition_number(const svd2x2_t *svd) {
    if (svd->s2 < 1e-14) return 1e14;
    return svd->s1 / svd->s2;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C472: SVD 2x2 simplified should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C472: empty output");
    assert!(
        code.contains("fn svd2x2_compute"),
        "C472: Should contain svd2x2_compute function"
    );
    assert!(
        code.contains("fn svd_sqrt_approx"),
        "C472: Should contain svd_sqrt_approx function"
    );
}

#[test]
fn c473_numerical_differentiation_central_difference() {
    let c_code = r#"
typedef struct {
    double h;
    int order;
} diff_params_t;

double diff_test_func(double x) {
    return x * x * x * x - 3.0 * x * x + 2.0 * x - 1.0;
}

double diff_forward(double x, double h) {
    return (diff_test_func(x + h) - diff_test_func(x)) / h;
}

double diff_backward(double x, double h) {
    return (diff_test_func(x) - diff_test_func(x - h)) / h;
}

double diff_central(double x, double h) {
    return (diff_test_func(x + h) - diff_test_func(x - h)) / (2.0 * h);
}

double diff_central_second(double x, double h) {
    return (diff_test_func(x + h) - 2.0 * diff_test_func(x) + diff_test_func(x - h)) / (h * h);
}

double diff_richardson(double x, double h) {
    double d1 = diff_central(x, h);
    double d2 = diff_central(x, h / 2.0);
    return (4.0 * d2 - d1) / 3.0;
}

double diff_estimate_error(double x, double h, double exact_deriv) {
    double approx = diff_central(x, h);
    double err = approx - exact_deriv;
    if (err < 0.0) err = -err;
    return err;
}

double diff_optimal_h(double x) {
    double h = 1.0;
    double best_h = h;
    double best_err = 1e30;
    int i;
    double exact = 4.0 * x * x * x - 6.0 * x + 2.0;
    for (i = 0; i < 20; i++) {
        double err = diff_estimate_error(x, h, exact);
        if (err < best_err) {
            best_err = err;
            best_h = h;
        }
        h /= 2.0;
    }
    return best_h;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C473: Numerical differentiation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C473: empty output");
    assert!(
        code.contains("fn diff_central"),
        "C473: Should contain diff_central function"
    );
    assert!(
        code.contains("fn diff_richardson"),
        "C473: Should contain diff_richardson function"
    );
}

#[test]
fn c474_adams_bashforth_multistep_ode() {
    let c_code = r#"
typedef struct {
    double t[128];
    double y[128];
    double f[128];
    int n;
    double h;
} ab_solver_t;

double ab_ode_func(double t, double y) {
    return -0.5 * y + t;
}

void ab_init(ab_solver_t *sol, double y0, double h) {
    sol->h = h;
    sol->n = 0;
    sol->t[0] = 0.0;
    sol->y[0] = y0;
    sol->f[0] = ab_ode_func(0.0, y0);
    sol->n = 1;
}

void ab_bootstrap_rk4(ab_solver_t *sol) {
    int i;
    for (i = 0; i < 3; i++) {
        double t = sol->t[sol->n - 1];
        double y = sol->y[sol->n - 1];
        double h = sol->h;
        double k1 = h * ab_ode_func(t, y);
        double k2 = h * ab_ode_func(t + h / 2.0, y + k1 / 2.0);
        double k3 = h * ab_ode_func(t + h / 2.0, y + k2 / 2.0);
        double k4 = h * ab_ode_func(t + h, y + k3);
        double y_new = y + (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0;
        double t_new = t + h;
        sol->t[sol->n] = t_new;
        sol->y[sol->n] = y_new;
        sol->f[sol->n] = ab_ode_func(t_new, y_new);
        sol->n++;
    }
}

void ab4_step(ab_solver_t *sol) {
    int n = sol->n;
    double h = sol->h;
    double y_new = sol->y[n - 1] + h * (
        55.0 * sol->f[n - 1]
      - 59.0 * sol->f[n - 2]
      + 37.0 * sol->f[n - 3]
      -  9.0 * sol->f[n - 4]
    ) / 24.0;
    double t_new = sol->t[n - 1] + h;
    sol->t[n] = t_new;
    sol->y[n] = y_new;
    sol->f[n] = ab_ode_func(t_new, y_new);
    sol->n++;
}

int ab_solve(ab_solver_t *sol, double t_end) {
    ab_bootstrap_rk4(sol);
    while (sol->t[sol->n - 1] < t_end && sol->n < 127) {
        ab4_step(sol);
    }
    return sol->n;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C474: Adams-Bashforth ODE solver should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C474: empty output");
    assert!(
        code.contains("fn ab4_step"),
        "C474: Should contain ab4_step function"
    );
    assert!(
        code.contains("fn ab_solve"),
        "C474: Should contain ab_solve function"
    );
}

#[test]
fn c475_conjugate_gradient_solver() {
    let c_code = r#"
typedef struct {
    double A[16];
    double b[4];
    double x[4];
    double r[4];
    double p[4];
    double Ap[4];
    int n;
    double tol;
    int max_iter;
} cg_solver_t;

void cg_init(cg_solver_t *cg, int n) {
    int i;
    cg->n = n;
    cg->tol = 1e-10;
    cg->max_iter = 1000;
    for (i = 0; i < n; i++) {
        cg->x[i] = 0.0;
    }
}

double cg_dot(const double *a, const double *b, int n) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += a[i] * b[i];
    }
    return sum;
}

void cg_matvec(const double *A, const double *x, double *y, int n) {
    int i, j;
    for (i = 0; i < n; i++) {
        y[i] = 0.0;
        for (j = 0; j < n; j++) {
            y[i] += A[i * n + j] * x[j];
        }
    }
}

int cg_solve(cg_solver_t *cg) {
    int i, iter;
    int n = cg->n;
    cg_matvec(cg->A, cg->x, cg->Ap, n);
    for (i = 0; i < n; i++) {
        cg->r[i] = cg->b[i] - cg->Ap[i];
        cg->p[i] = cg->r[i];
    }
    double rs_old = cg_dot(cg->r, cg->r, n);
    for (iter = 0; iter < cg->max_iter; iter++) {
        cg_matvec(cg->A, cg->p, cg->Ap, n);
        double pAp = cg_dot(cg->p, cg->Ap, n);
        if (pAp == 0.0) return -2;
        double alpha = rs_old / pAp;
        for (i = 0; i < n; i++) {
            cg->x[i] += alpha * cg->p[i];
            cg->r[i] -= alpha * cg->Ap[i];
        }
        double rs_new = cg_dot(cg->r, cg->r, n);
        if (rs_new < cg->tol * cg->tol) return iter + 1;
        double beta = rs_new / rs_old;
        for (i = 0; i < n; i++) {
            cg->p[i] = cg->r[i] + beta * cg->p[i];
        }
        rs_old = rs_new;
    }
    return -1;
}

double cg_residual_norm(const cg_solver_t *cg) {
    double norm_sq = cg_dot(cg->r, cg->r, cg->n);
    double norm = norm_sq;
    int i;
    if (norm > 0.0) {
        for (i = 0; i < 20; i++) {
            norm = (norm + norm_sq / norm) / 2.0;
        }
    }
    return norm;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C475: Conjugate gradient solver should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C475: empty output");
    assert!(
        code.contains("fn cg_solve"),
        "C475: Should contain cg_solve function"
    );
    assert!(
        code.contains("fn cg_dot"),
        "C475: Should contain cg_dot function"
    );
}
