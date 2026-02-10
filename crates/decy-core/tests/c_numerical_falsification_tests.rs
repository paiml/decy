//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1276-C1300: Numerical Methods & Scientific Computing -- ODE solvers,
//! interpolation, quadrature, root finding, PDE discretization, and FFT
//! patterns commonly found in GSL, SUNDIALS, FFTW, and numerical recipes.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world numerical programming patterns commonly
//! found in scientific computing libraries -- all expressed as valid C99.
//!
//! Organization:
//! - C1276-C1280: ODE solvers (Euler, RK4, adaptive step, leapfrog, Verlet)
//! - C1281-C1285: Interpolation (linear, Lagrange, cubic spline, Chebyshev, polynomial eval)
//! - C1286-C1290: Integration (trapezoidal, Simpson, Gauss-Legendre, Romberg, Monte Carlo)
//! - C1291-C1295: Root finding (Newton-Raphson, Brent, fixed point, Muller, Durand-Kerner)
//! - C1296-C1300: PDE/FFT (heat equation FD, wave equation FD, FFT butterfly, DFT direct, spectral)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C1276-C1280: ODE Solvers
// ============================================================================

/// C1276: Forward Euler method for ODE integration
#[test]
fn c1276_euler_method_ode() {
    let c_code = r#"
typedef struct {
    double t;
    double y;
    double h;
    double t_end;
} num_euler_state_t;

double num_euler_dydt(double t, double y) {
    return -2.0 * y + t * t;
}

void num_euler_step(num_euler_state_t *st) {
    double dy = num_euler_dydt(st->t, st->y);
    st->y += st->h * dy;
    st->t += st->h;
}

int num_euler_integrate(num_euler_state_t *st, double *results, int max_steps) {
    int steps = 0;
    results[0] = st->y;
    while (st->t < st->t_end && steps < max_steps) {
        num_euler_step(st);
        steps++;
        results[steps] = st->y;
    }
    return steps;
}

void num_euler_init(num_euler_state_t *st, double y0, double h, double t_end) {
    st->t = 0.0;
    st->y = y0;
    st->h = h;
    st->t_end = t_end;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1276: Euler method ODE should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1276: empty output");
    assert!(
        code.contains("fn num_euler_step"),
        "C1276: Should contain num_euler_step function"
    );
    assert!(
        code.contains("fn num_euler_integrate"),
        "C1276: Should contain num_euler_integrate function"
    );
}

/// C1277: Classical 4th-order Runge-Kutta integrator
#[test]
fn c1277_rk4_ode_solver() {
    let c_code = r#"
typedef struct {
    double t;
    double y;
    double h;
} num_rk4_state_t;

double num_rk4_f(double t, double y) {
    return -0.5 * y + 2.0 * t;
}

void num_rk4_step(num_rk4_state_t *st) {
    double h = st->h;
    double t = st->t;
    double y = st->y;
    double k1 = h * num_rk4_f(t, y);
    double k2 = h * num_rk4_f(t + h / 2.0, y + k1 / 2.0);
    double k3 = h * num_rk4_f(t + h / 2.0, y + k2 / 2.0);
    double k4 = h * num_rk4_f(t + h, y + k3);
    st->y = y + (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0;
    st->t = t + h;
}

int num_rk4_solve(num_rk4_state_t *st, double t_end, double *out, int max_n) {
    int n = 0;
    out[0] = st->y;
    while (st->t < t_end && n < max_n - 1) {
        num_rk4_step(st);
        n++;
        out[n] = st->y;
    }
    return n;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1277: RK4 ODE solver should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1277: empty output");
    assert!(
        code.contains("fn num_rk4_step"),
        "C1277: Should contain num_rk4_step function"
    );
    assert!(
        code.contains("fn num_rk4_solve"),
        "C1277: Should contain num_rk4_solve function"
    );
}

/// C1278: Adaptive step-size RK4 with error estimation
#[test]
fn c1278_adaptive_step_rk4() {
    let c_code = r#"
typedef struct {
    double t;
    double y;
    double h;
    double tol;
    double h_min;
    double h_max;
} num_ark_state_t;

double num_ark_f(double t, double y) {
    return -3.0 * y + t;
}

double num_ark_abs(double x) {
    return x < 0.0 ? -x : x;
}

double num_ark_rk4_step(double t, double y, double h) {
    double k1 = h * num_ark_f(t, y);
    double k2 = h * num_ark_f(t + h / 2.0, y + k1 / 2.0);
    double k3 = h * num_ark_f(t + h / 2.0, y + k2 / 2.0);
    double k4 = h * num_ark_f(t + h, y + k3);
    return y + (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0;
}

int num_ark_step(num_ark_state_t *st) {
    double y_full = num_ark_rk4_step(st->t, st->y, st->h);
    double y_half1 = num_ark_rk4_step(st->t, st->y, st->h / 2.0);
    double y_half2 = num_ark_rk4_step(st->t + st->h / 2.0, y_half1, st->h / 2.0);
    double err = num_ark_abs(y_half2 - y_full);
    if (err < st->tol) {
        st->y = y_half2;
        st->t += st->h;
        if (st->h * 2.0 < st->h_max) st->h *= 2.0;
        return 1;
    }
    st->h /= 2.0;
    if (st->h < st->h_min) return -1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1278: Adaptive step RK4 should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1278: empty output");
    assert!(
        code.contains("fn num_ark_step"),
        "C1278: Should contain num_ark_step function"
    );
    assert!(
        code.contains("fn num_ark_rk4_step"),
        "C1278: Should contain num_ark_rk4_step function"
    );
}

/// C1279: Leapfrog (Stormer-Verlet) integrator for Hamiltonian systems
#[test]
fn c1279_leapfrog_integrator() {
    let c_code = r#"
typedef struct {
    double x;
    double v;
    double dt;
    double mass;
} num_leap_state_t;

double num_leap_force(double x) {
    return -x - 0.1 * x * x * x;
}

void num_leap_init(num_leap_state_t *st, double x0, double v0, double dt) {
    st->x = x0;
    st->v = v0;
    st->dt = dt;
    st->mass = 1.0;
}

void num_leap_step(num_leap_state_t *st) {
    double a = num_leap_force(st->x) / st->mass;
    double v_half = st->v + 0.5 * st->dt * a;
    st->x += st->dt * v_half;
    double a_new = num_leap_force(st->x) / st->mass;
    st->v = v_half + 0.5 * st->dt * a_new;
}

double num_leap_energy(const num_leap_state_t *st) {
    double ke = 0.5 * st->mass * st->v * st->v;
    double pe = 0.5 * st->x * st->x + 0.025 * st->x * st->x * st->x * st->x;
    return ke + pe;
}

int num_leap_evolve(num_leap_state_t *st, int steps) {
    int i;
    for (i = 0; i < steps; i++) {
        num_leap_step(st);
    }
    return steps;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1279: Leapfrog integrator should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1279: empty output");
    assert!(
        code.contains("fn num_leap_step"),
        "C1279: Should contain num_leap_step function"
    );
    assert!(
        code.contains("fn num_leap_energy"),
        "C1279: Should contain num_leap_energy function"
    );
}

/// C1280: Velocity Verlet integration for molecular dynamics
#[test]
fn c1280_verlet_integration() {
    let c_code = r#"
typedef struct {
    double pos[32];
    double vel[32];
    double acc[32];
    int n;
    double dt;
} num_verlet_t;

double num_verlet_force(double x, int idx) {
    return -x * (1.0 + 0.01 * idx);
}

void num_verlet_init(num_verlet_t *v, int n, double dt) {
    int i;
    v->n = n;
    v->dt = dt;
    for (i = 0; i < n; i++) {
        v->pos[i] = 0.0;
        v->vel[i] = 0.0;
        v->acc[i] = 0.0;
    }
}

void num_verlet_step(num_verlet_t *v) {
    int i;
    double dt = v->dt;
    for (i = 0; i < v->n; i++) {
        v->pos[i] += v->vel[i] * dt + 0.5 * v->acc[i] * dt * dt;
    }
    for (i = 0; i < v->n; i++) {
        double new_acc = num_verlet_force(v->pos[i], i);
        v->vel[i] += 0.5 * (v->acc[i] + new_acc) * dt;
        v->acc[i] = new_acc;
    }
}

double num_verlet_kinetic(const num_verlet_t *v) {
    double ke = 0.0;
    int i;
    for (i = 0; i < v->n; i++) {
        ke += 0.5 * v->vel[i] * v->vel[i];
    }
    return ke;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1280: Verlet integration should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1280: empty output");
    assert!(
        code.contains("fn num_verlet_step"),
        "C1280: Should contain num_verlet_step function"
    );
    assert!(
        code.contains("fn num_verlet_kinetic"),
        "C1280: Should contain num_verlet_kinetic function"
    );
}

// ============================================================================
// C1281-C1285: Interpolation
// ============================================================================

/// C1281: Piecewise linear interpolation
#[test]
fn c1281_linear_interpolation() {
    let c_code = r#"
typedef struct {
    double x[32];
    double y[32];
    int n;
} num_linterp_t;

void num_linterp_init(num_linterp_t *li) {
    li->n = 0;
}

void num_linterp_add(num_linterp_t *li, double x, double y) {
    if (li->n < 32) {
        li->x[li->n] = x;
        li->y[li->n] = y;
        li->n++;
    }
}

int num_linterp_find_segment(const num_linterp_t *li, double x) {
    int i;
    for (i = 0; i < li->n - 1; i++) {
        if (x >= li->x[i] && x <= li->x[i + 1]) return i;
    }
    return li->n - 2;
}

double num_linterp_eval(const num_linterp_t *li, double x) {
    int seg = num_linterp_find_segment(li, x);
    double x0 = li->x[seg];
    double x1 = li->x[seg + 1];
    double y0 = li->y[seg];
    double y1 = li->y[seg + 1];
    if (x1 - x0 == 0.0) return y0;
    double t = (x - x0) / (x1 - x0);
    return y0 + t * (y1 - y0);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1281: Linear interpolation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1281: empty output");
    assert!(
        code.contains("fn num_linterp_eval"),
        "C1281: Should contain num_linterp_eval function"
    );
    assert!(
        code.contains("fn num_linterp_find_segment"),
        "C1281: Should contain num_linterp_find_segment function"
    );
}

/// C1282: Lagrange interpolation polynomial
#[test]
fn c1282_lagrange_interpolation() {
    let c_code = r#"
typedef struct {
    double x[16];
    double y[16];
    int n;
} num_lagr_data_t;

double num_lagr_basis(const num_lagr_data_t *d, int j, double x) {
    double L = 1.0;
    int i;
    for (i = 0; i < d->n; i++) {
        if (i != j) {
            L *= (x - d->x[i]) / (d->x[j] - d->x[i]);
        }
    }
    return L;
}

double num_lagr_eval(const num_lagr_data_t *d, double x) {
    double result = 0.0;
    int j;
    for (j = 0; j < d->n; j++) {
        result += d->y[j] * num_lagr_basis(d, j, x);
    }
    return result;
}

double num_lagr_error_bound(const num_lagr_data_t *d, double x, double max_deriv) {
    double omega = 1.0;
    double factorial = 1.0;
    int i;
    for (i = 0; i < d->n; i++) {
        omega *= (x - d->x[i]);
        factorial *= (i + 1);
    }
    if (omega < 0.0) omega = -omega;
    return max_deriv * omega / factorial;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1282: Lagrange interpolation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1282: empty output");
    assert!(
        code.contains("fn num_lagr_eval"),
        "C1282: Should contain num_lagr_eval function"
    );
    assert!(
        code.contains("fn num_lagr_basis"),
        "C1282: Should contain num_lagr_basis function"
    );
}

/// C1283: Natural cubic spline interpolation
#[test]
fn c1283_cubic_spline_interpolation() {
    let c_code = r#"
typedef struct {
    double x[16];
    double y[16];
    double a[16];
    double b[16];
    double c[16];
    double d[16];
    int n;
} num_cspline_t;

void num_cspline_init(num_cspline_t *sp) {
    sp->n = 0;
}

void num_cspline_add(num_cspline_t *sp, double x, double y) {
    if (sp->n < 16) {
        sp->x[sp->n] = x;
        sp->y[sp->n] = y;
        sp->n++;
    }
}

void num_cspline_build(num_cspline_t *sp) {
    int i, n;
    double h[15], alpha[15], l[16], mu[16], z[16];
    n = sp->n - 1;
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

double num_cspline_eval(const num_cspline_t *sp, double x) {
    int seg = 0;
    int i;
    for (i = 0; i < sp->n - 1; i++) {
        if (x >= sp->x[i] && x <= sp->x[i + 1]) { seg = i; break; }
    }
    double dx = x - sp->x[seg];
    return sp->a[seg] + sp->b[seg] * dx + sp->c[seg] * dx * dx
         + sp->d[seg] * dx * dx * dx;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1283: Cubic spline interpolation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1283: empty output");
    assert!(
        code.contains("fn num_cspline_build"),
        "C1283: Should contain num_cspline_build function"
    );
    assert!(
        code.contains("fn num_cspline_eval"),
        "C1283: Should contain num_cspline_eval function"
    );
}

/// C1284: Chebyshev polynomial approximation
#[test]
fn c1284_chebyshev_approximation() {
    let c_code = r#"
typedef struct {
    double coeffs[16];
    int n;
    double a;
    double b;
} num_cheb_t;

double num_cheb_func(double x) {
    return 1.0 / (1.0 + 25.0 * x * x);
}

double num_cheb_cos_approx(double x) {
    double x2 = x * x;
    return 1.0 - x2 / 2.0 + x2 * x2 / 24.0 - x2 * x2 * x2 / 720.0;
}

void num_cheb_fit(num_cheb_t *ch, int n, double a, double b) {
    int j, k;
    double pi = 3.14159265358979;
    ch->n = n;
    ch->a = a;
    ch->b = b;
    for (j = 0; j < n; j++) {
        double sum = 0.0;
        for (k = 0; k < n; k++) {
            double theta = pi * (k + 0.5) / n;
            double xk = num_cheb_cos_approx(theta);
            double x = 0.5 * (b + a) + 0.5 * (b - a) * xk;
            double tk = num_cheb_cos_approx(j * theta);
            sum += num_cheb_func(x) * tk;
        }
        ch->coeffs[j] = 2.0 * sum / n;
    }
}

double num_cheb_eval(const num_cheb_t *ch, double x) {
    double xn = (2.0 * x - ch->a - ch->b) / (ch->b - ch->a);
    double d0 = 0.0, d1 = 0.0;
    int j;
    for (j = ch->n - 1; j >= 1; j--) {
        double tmp = 2.0 * xn * d0 - d1 + ch->coeffs[j];
        d1 = d0;
        d0 = tmp;
    }
    return xn * d0 - d1 + 0.5 * ch->coeffs[0];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1284: Chebyshev approximation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1284: empty output");
    assert!(
        code.contains("fn num_cheb_fit"),
        "C1284: Should contain num_cheb_fit function"
    );
    assert!(
        code.contains("fn num_cheb_eval"),
        "C1284: Should contain num_cheb_eval function"
    );
}

/// C1285: Horner's method polynomial evaluation with derivative
#[test]
fn c1285_polynomial_evaluation() {
    let c_code = r#"
typedef struct {
    double c[16];
    int deg;
} num_poly_t;

void num_poly_init(num_poly_t *p, int deg) {
    int i;
    p->deg = deg;
    for (i = 0; i <= deg; i++) {
        p->c[i] = 0.0;
    }
}

double num_poly_horner(const num_poly_t *p, double x) {
    double val = p->c[p->deg];
    int i;
    for (i = p->deg - 1; i >= 0; i--) {
        val = val * x + p->c[i];
    }
    return val;
}

double num_poly_deriv(const num_poly_t *p, double x) {
    double val = p->c[p->deg] * p->deg;
    int i;
    for (i = p->deg - 1; i >= 1; i--) {
        val = val * x + p->c[i] * i;
    }
    return val;
}

void num_poly_multiply(const num_poly_t *a, const num_poly_t *b, num_poly_t *r) {
    int i, j;
    num_poly_init(r, a->deg + b->deg);
    for (i = 0; i <= a->deg; i++) {
        for (j = 0; j <= b->deg; j++) {
            r->c[i + j] += a->c[i] * b->c[j];
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1285: Polynomial evaluation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1285: empty output");
    assert!(
        code.contains("fn num_poly_horner"),
        "C1285: Should contain num_poly_horner function"
    );
    assert!(
        code.contains("fn num_poly_deriv"),
        "C1285: Should contain num_poly_deriv function"
    );
}

// ============================================================================
// C1286-C1290: Integration (Quadrature)
// ============================================================================

/// C1286: Composite trapezoidal rule with Richardson extrapolation
#[test]
fn c1286_trapezoidal_rule() {
    let c_code = r#"
double num_trap_f(double x) {
    return x * x * x - 2.0 * x + 1.0;
}

double num_trap_integrate(double a, double b, int n) {
    double h = (b - a) / n;
    double sum = (num_trap_f(a) + num_trap_f(b)) / 2.0;
    int i;
    for (i = 1; i < n; i++) {
        sum += num_trap_f(a + i * h);
    }
    return sum * h;
}

double num_trap_richardson(double a, double b, int n) {
    double t1 = num_trap_integrate(a, b, n);
    double t2 = num_trap_integrate(a, b, 2 * n);
    return (4.0 * t2 - t1) / 3.0;
}

double num_trap_error(double a, double b, int n, double exact) {
    double approx = num_trap_integrate(a, b, n);
    double err = approx - exact;
    if (err < 0.0) err = -err;
    return err;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1286: Trapezoidal rule should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1286: empty output");
    assert!(
        code.contains("fn num_trap_integrate"),
        "C1286: Should contain num_trap_integrate function"
    );
    assert!(
        code.contains("fn num_trap_richardson"),
        "C1286: Should contain num_trap_richardson function"
    );
}

/// C1287: Composite Simpson's 1/3 rule
#[test]
fn c1287_simpsons_rule() {
    let c_code = r#"
double num_simp_f(double x) {
    return 1.0 / (1.0 + x * x);
}

double num_simp_integrate(double a, double b, int n) {
    if (n % 2 != 0) n++;
    double h = (b - a) / n;
    double sum = num_simp_f(a) + num_simp_f(b);
    int i;
    for (i = 1; i < n; i++) {
        double xi = a + i * h;
        if (i % 2 == 0) {
            sum += 2.0 * num_simp_f(xi);
        } else {
            sum += 4.0 * num_simp_f(xi);
        }
    }
    return sum * h / 3.0;
}

double num_simp_adaptive(double a, double b, double tol) {
    int n = 2;
    double prev = 0.0;
    int iter;
    for (iter = 0; iter < 20; iter++) {
        double curr = num_simp_integrate(a, b, n);
        if (iter > 0) {
            double diff = curr - prev;
            if (diff < 0.0) diff = -diff;
            if (diff < tol) return curr;
        }
        prev = curr;
        n *= 2;
    }
    return prev;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1287: Simpson's rule should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1287: empty output");
    assert!(
        code.contains("fn num_simp_integrate"),
        "C1287: Should contain num_simp_integrate function"
    );
    assert!(
        code.contains("fn num_simp_adaptive"),
        "C1287: Should contain num_simp_adaptive function"
    );
}

/// C1288: Gauss-Legendre 3-point quadrature
#[test]
fn c1288_gauss_legendre_quadrature() {
    let c_code = r#"
double num_gl_f(double x) {
    return x * x * x * x - x * x + 1.0;
}

double num_gl_3pt(double a, double b) {
    double mid = 0.5 * (a + b);
    double half = 0.5 * (b - a);
    double x1 = mid - half * 0.7745966692;
    double x2 = mid;
    double x3 = mid + half * 0.7745966692;
    double w1 = 0.5555555556;
    double w2 = 0.8888888889;
    double w3 = 0.5555555556;
    return half * (w1 * num_gl_f(x1) + w2 * num_gl_f(x2) + w3 * num_gl_f(x3));
}

double num_gl_composite(double a, double b, int panels) {
    double h = (b - a) / panels;
    double sum = 0.0;
    int i;
    for (i = 0; i < panels; i++) {
        double ai = a + i * h;
        double bi = ai + h;
        sum += num_gl_3pt(ai, bi);
    }
    return sum;
}

double num_gl_error(double a, double b, int panels, double exact) {
    double approx = num_gl_composite(a, b, panels);
    double err = approx - exact;
    if (err < 0.0) err = -err;
    return err;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1288: Gauss-Legendre quadrature should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1288: empty output");
    assert!(
        code.contains("fn num_gl_3pt"),
        "C1288: Should contain num_gl_3pt function"
    );
    assert!(
        code.contains("fn num_gl_composite"),
        "C1288: Should contain num_gl_composite function"
    );
}

/// C1289: Romberg integration via Richardson extrapolation table
#[test]
fn c1289_romberg_integration() {
    let c_code = r#"
double num_romb_f(double x) {
    return x * x * x + 2.0 * x - 1.0;
}

double num_romb_trap(double a, double b, int n) {
    double h = (b - a) / n;
    double sum = (num_romb_f(a) + num_romb_f(b)) / 2.0;
    int i;
    for (i = 1; i < n; i++) {
        sum += num_romb_f(a + i * h);
    }
    return sum * h;
}

double num_romb_integrate(double a, double b, int max_k, double tol) {
    double R[8][8];
    int k, j;
    R[0][0] = num_romb_trap(a, b, 1);
    for (k = 1; k < max_k; k++) {
        int n = 1;
        int i;
        for (i = 0; i < k; i++) n *= 2;
        R[k][0] = num_romb_trap(a, b, n);
        double power = 1.0;
        for (j = 1; j <= k; j++) {
            power *= 4.0;
            R[k][j] = (power * R[k][j - 1] - R[k - 1][j - 1]) / (power - 1.0);
        }
        double diff = R[k][k] - R[k - 1][k - 1];
        if (diff < 0.0) diff = -diff;
        if (diff < tol) return R[k][k];
    }
    return R[max_k - 1][max_k - 1];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1289: Romberg integration should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1289: empty output");
    assert!(
        code.contains("fn num_romb_integrate"),
        "C1289: Should contain num_romb_integrate function"
    );
    assert!(
        code.contains("fn num_romb_trap"),
        "C1289: Should contain num_romb_trap function"
    );
}

/// C1290: Monte Carlo integration with simple LCG
#[test]
fn c1290_monte_carlo_integration() {
    let c_code = r#"
typedef unsigned int num_mc_uint32;

typedef struct {
    num_mc_uint32 state;
} num_mc_rng_t;

void num_mc_rng_init(num_mc_rng_t *r, num_mc_uint32 seed) {
    r->state = seed;
}

num_mc_uint32 num_mc_rng_next(num_mc_rng_t *r) {
    r->state = r->state * 1103515245 + 12345;
    return (r->state >> 16) & 0x7FFF;
}

double num_mc_rng_uniform(num_mc_rng_t *r, double a, double b) {
    double u = (double)num_mc_rng_next(r) / 32767.0;
    return a + u * (b - a);
}

double num_mc_func(double x) {
    return x * x * x + 1.0;
}

double num_mc_integrate(double a, double b, int n, num_mc_uint32 seed) {
    num_mc_rng_t rng;
    double sum = 0.0;
    int i;
    num_mc_rng_init(&rng, seed);
    for (i = 0; i < n; i++) {
        double x = num_mc_rng_uniform(&rng, a, b);
        sum += num_mc_func(x);
    }
    return (b - a) * sum / n;
}

double num_mc_variance(double a, double b, int n, num_mc_uint32 seed) {
    num_mc_rng_t rng;
    double sum = 0.0, sum_sq = 0.0;
    int i;
    num_mc_rng_init(&rng, seed);
    for (i = 0; i < n; i++) {
        double x = num_mc_rng_uniform(&rng, a, b);
        double fx = num_mc_func(x);
        sum += fx;
        sum_sq += fx * fx;
    }
    double mean = sum / n;
    return (sum_sq / n - mean * mean) * (b - a) * (b - a) / n;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1290: Monte Carlo integration should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1290: empty output");
    assert!(
        code.contains("fn num_mc_integrate"),
        "C1290: Should contain num_mc_integrate function"
    );
    assert!(
        code.contains("fn num_mc_variance"),
        "C1290: Should contain num_mc_variance function"
    );
}

// ============================================================================
// C1291-C1295: Root Finding
// ============================================================================

/// C1291: Newton-Raphson root finding with convergence tracking
#[test]
fn c1291_newton_raphson() {
    let c_code = r#"
typedef struct {
    double root;
    int iterations;
    int converged;
    double tol;
    int max_iter;
} num_nr_result_t;

double num_nr_f(double x) {
    return x * x * x - 2.0 * x - 5.0;
}

double num_nr_fp(double x) {
    return 3.0 * x * x - 2.0;
}

void num_nr_solve(num_nr_result_t *res, double x0) {
    double x = x0;
    int i;
    res->converged = 0;
    for (i = 0; i < res->max_iter; i++) {
        double fx = num_nr_f(x);
        double fpx = num_nr_fp(x);
        if (fpx == 0.0) break;
        double dx = fx / fpx;
        x -= dx;
        if (dx < 0.0) dx = -dx;
        if (dx < res->tol) {
            res->root = x;
            res->iterations = i + 1;
            res->converged = 1;
            return;
        }
    }
    res->root = x;
    res->iterations = res->max_iter;
}

void num_nr_init(num_nr_result_t *res) {
    res->tol = 1e-12;
    res->max_iter = 100;
    res->root = 0.0;
    res->iterations = 0;
    res->converged = 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1291: Newton-Raphson should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1291: empty output");
    assert!(
        code.contains("fn num_nr_solve"),
        "C1291: Should contain num_nr_solve function"
    );
    assert!(
        code.contains("fn num_nr_f"),
        "C1291: Should contain num_nr_f function"
    );
}

/// C1292: Brent's method for bracketed root finding
#[test]
fn c1292_brents_method() {
    let c_code = r#"
double num_brent_f(double x) {
    return x * x * x - x - 1.0;
}

double num_brent_abs(double x) {
    return x < 0.0 ? -x : x;
}

double num_brent_solve(double a, double b, double tol, int max_iter) {
    double fa = num_brent_f(a);
    double fb = num_brent_f(b);
    double c = a, fc = fa;
    double d = b - a, e = d;
    int i;
    if (fa * fb > 0.0) return a;
    for (i = 0; i < max_iter; i++) {
        if (fb * fc > 0.0) { c = a; fc = fa; d = b - a; e = d; }
        if (num_brent_abs(fc) < num_brent_abs(fb)) {
            a = b; b = c; c = a;
            fa = fb; fb = fc; fc = fa;
        }
        double m = 0.5 * (c - b);
        if (num_brent_abs(m) <= tol || fb == 0.0) return b;
        if (num_brent_abs(e) >= tol && num_brent_abs(fa) > num_brent_abs(fb)) {
            double s = fb / fa;
            double p, q;
            if (a == c) {
                p = 2.0 * m * s;
                q = 1.0 - s;
            } else {
                q = fa / fc;
                double r = fb / fc;
                p = s * (2.0 * m * q * (q - r) - (b - a) * (r - 1.0));
                q = (q - 1.0) * (r - 1.0) * (s - 1.0);
            }
            if (p > 0.0) q = -q; else p = -p;
            if (2.0 * p < 3.0 * m * q - num_brent_abs(tol * q) && 2.0 * p < num_brent_abs(e * q)) {
                e = d; d = p / q;
            } else { d = m; e = m; }
        } else { d = m; e = m; }
        a = b; fa = fb;
        if (num_brent_abs(d) > tol) b += d;
        else b += (m > 0.0) ? tol : -tol;
        fb = num_brent_f(b);
    }
    return b;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1292: Brent's method should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1292: empty output");
    assert!(
        code.contains("fn num_brent_solve"),
        "C1292: Should contain num_brent_solve function"
    );
    assert!(
        code.contains("fn num_brent_f"),
        "C1292: Should contain num_brent_f function"
    );
}

/// C1293: Fixed-point iteration with convergence check
#[test]
fn c1293_fixed_point_iteration() {
    let c_code = r#"
typedef struct {
    double x;
    double tol;
    int max_iter;
    int converged;
    int iterations;
} num_fpi_state_t;

double num_fpi_g(double x) {
    return (x + 2.0 / (x * x)) / 3.0;
}

double num_fpi_abs(double x) {
    return x < 0.0 ? -x : x;
}

void num_fpi_init(num_fpi_state_t *st, double x0) {
    st->x = x0;
    st->tol = 1e-10;
    st->max_iter = 200;
    st->converged = 0;
    st->iterations = 0;
}

int num_fpi_solve(num_fpi_state_t *st) {
    int i;
    for (i = 0; i < st->max_iter; i++) {
        double x_new = num_fpi_g(st->x);
        double diff = num_fpi_abs(x_new - st->x);
        st->x = x_new;
        if (diff < st->tol) {
            st->converged = 1;
            st->iterations = i + 1;
            return 1;
        }
    }
    st->iterations = st->max_iter;
    return 0;
}

double num_fpi_residual(double x) {
    double gx = num_fpi_g(x);
    double diff = gx - x;
    if (diff < 0.0) diff = -diff;
    return diff;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1293: Fixed point iteration should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1293: empty output");
    assert!(
        code.contains("fn num_fpi_solve"),
        "C1293: Should contain num_fpi_solve function"
    );
    assert!(
        code.contains("fn num_fpi_g"),
        "C1293: Should contain num_fpi_g function"
    );
}

/// C1294: Muller's method for polynomial root finding
#[test]
fn c1294_mullers_method() {
    let c_code = r#"
double num_muller_f(double x) {
    return x * x * x - x * x + 2.0 * x - 2.0;
}

double num_muller_abs(double x) {
    return x < 0.0 ? -x : x;
}

double num_muller_solve(double x0, double x1, double x2, double tol, int max_iter) {
    int i;
    for (i = 0; i < max_iter; i++) {
        double f0 = num_muller_f(x0);
        double f1 = num_muller_f(x1);
        double f2 = num_muller_f(x2);
        double h1 = x1 - x0;
        double h2 = x2 - x1;
        double d1 = (f1 - f0) / h1;
        double d2 = (f2 - f1) / h2;
        double a = (d2 - d1) / (h2 + h1);
        double b = a * h2 + d2;
        double c = f2;
        double disc = b * b - 4.0 * a * c;
        if (disc < 0.0) disc = 0.0;
        double sq = disc;
        int j;
        for (j = 0; j < 30; j++) {
            if (sq > 0.0) sq = (sq + disc / sq) / 2.0;
        }
        double denom;
        if (num_muller_abs(b + sq) > num_muller_abs(b - sq)) {
            denom = b + sq;
        } else {
            denom = b - sq;
        }
        if (denom == 0.0) break;
        double dx = -2.0 * c / denom;
        double x3 = x2 + dx;
        if (num_muller_abs(dx) < tol) return x3;
        x0 = x1; x1 = x2; x2 = x3;
    }
    return x2;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1294: Muller's method should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1294: empty output");
    assert!(
        code.contains("fn num_muller_solve"),
        "C1294: Should contain num_muller_solve function"
    );
    assert!(
        code.contains("fn num_muller_f"),
        "C1294: Should contain num_muller_f function"
    );
}

/// C1295: Durand-Kerner method for all polynomial roots simultaneously
#[test]
fn c1295_durand_kerner() {
    let c_code = r#"
typedef struct {
    double real;
    double imag;
} num_dk_complex_t;

num_dk_complex_t num_dk_cmul(num_dk_complex_t a, num_dk_complex_t b) {
    num_dk_complex_t r;
    r.real = a.real * b.real - a.imag * b.imag;
    r.imag = a.real * b.imag + a.imag * b.real;
    return r;
}

num_dk_complex_t num_dk_cdiv(num_dk_complex_t a, num_dk_complex_t b) {
    num_dk_complex_t r;
    double denom = b.real * b.real + b.imag * b.imag;
    if (denom < 1e-30 && denom > -1e-30) { r.real = 0.0; r.imag = 0.0; return r; }
    r.real = (a.real * b.real + a.imag * b.imag) / denom;
    r.imag = (a.imag * b.real - a.real * b.imag) / denom;
    return r;
}

num_dk_complex_t num_dk_csub(num_dk_complex_t a, num_dk_complex_t b) {
    num_dk_complex_t r;
    r.real = a.real - b.real;
    r.imag = a.imag - b.imag;
    return r;
}

num_dk_complex_t num_dk_poly_eval(const double *coeffs, int deg, num_dk_complex_t z) {
    num_dk_complex_t result;
    int i;
    result.real = coeffs[deg];
    result.imag = 0.0;
    for (i = deg - 1; i >= 0; i--) {
        result = num_dk_cmul(result, z);
        result.real += coeffs[i];
    }
    return result;
}

double num_dk_cabs(num_dk_complex_t c) {
    double sq = c.real * c.real + c.imag * c.imag;
    double r = sq;
    int i;
    if (r <= 0.0) return 0.0;
    for (i = 0; i < 30; i++) {
        r = (r + sq / r) / 2.0;
    }
    return r;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1295: Durand-Kerner should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1295: empty output");
    assert!(
        code.contains("fn num_dk_cmul"),
        "C1295: Should contain num_dk_cmul function"
    );
    assert!(
        code.contains("fn num_dk_poly_eval"),
        "C1295: Should contain num_dk_poly_eval function"
    );
}

// ============================================================================
// C1296-C1300: PDE/FFT
// ============================================================================

/// C1296: 1D heat equation via explicit finite differences
#[test]
fn c1296_heat_equation_fd() {
    let c_code = r#"
typedef struct {
    double u[64];
    double u_new[64];
    int nx;
    double dx;
    double dt;
    double alpha;
} num_heat_t;

void num_heat_init(num_heat_t *h, int nx, double alpha) {
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

void num_heat_step(num_heat_t *h) {
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

void num_heat_evolve(num_heat_t *h, int steps) {
    int s;
    for (s = 0; s < steps; s++) {
        num_heat_step(h);
    }
}

double num_heat_max(const num_heat_t *h) {
    double m = h->u[0];
    int i;
    for (i = 1; i < h->nx; i++) {
        if (h->u[i] > m) m = h->u[i];
    }
    return m;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1296: Heat equation FD should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1296: empty output");
    assert!(
        code.contains("fn num_heat_step"),
        "C1296: Should contain num_heat_step function"
    );
    assert!(
        code.contains("fn num_heat_evolve"),
        "C1296: Should contain num_heat_evolve function"
    );
}

/// C1297: 1D wave equation via finite differences
#[test]
fn c1297_wave_equation_fd() {
    let c_code = r#"
typedef struct {
    double u[64];
    double u_prev[64];
    double u_next[64];
    int nx;
    double dx;
    double dt;
    double c;
} num_wave_t;

void num_wave_init(num_wave_t *w, int nx, double c) {
    int i;
    w->nx = nx;
    w->c = c;
    w->dx = 1.0 / (nx - 1);
    w->dt = 0.9 * w->dx / c;
    for (i = 0; i < nx; i++) {
        w->u[i] = 0.0;
        w->u_prev[i] = 0.0;
        w->u_next[i] = 0.0;
    }
}

void num_wave_step(num_wave_t *w) {
    int i;
    double r2 = (w->c * w->dt / w->dx) * (w->c * w->dt / w->dx);
    w->u_next[0] = 0.0;
    w->u_next[w->nx - 1] = 0.0;
    for (i = 1; i < w->nx - 1; i++) {
        w->u_next[i] = 2.0 * w->u[i] - w->u_prev[i]
                      + r2 * (w->u[i + 1] - 2.0 * w->u[i] + w->u[i - 1]);
    }
    for (i = 0; i < w->nx; i++) {
        w->u_prev[i] = w->u[i];
        w->u[i] = w->u_next[i];
    }
}

double num_wave_energy(const num_wave_t *w) {
    double e = 0.0;
    int i;
    for (i = 1; i < w->nx - 1; i++) {
        double du = (w->u[i] - w->u_prev[i]) / w->dt;
        double dx = (w->u[i + 1] - w->u[i - 1]) / (2.0 * w->dx);
        e += 0.5 * (du * du + w->c * w->c * dx * dx) * w->dx;
    }
    return e;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1297: Wave equation FD should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1297: empty output");
    assert!(
        code.contains("fn num_wave_step"),
        "C1297: Should contain num_wave_step function"
    );
    assert!(
        code.contains("fn num_wave_energy"),
        "C1297: Should contain num_wave_energy function"
    );
}

/// C1298: FFT butterfly operations with complex arithmetic
#[test]
fn c1298_fft_butterfly() {
    let c_code = r#"
typedef struct {
    double re;
    double im;
} num_fft_c_t;

num_fft_c_t num_fft_cadd(num_fft_c_t a, num_fft_c_t b) {
    num_fft_c_t r;
    r.re = a.re + b.re;
    r.im = a.im + b.im;
    return r;
}

num_fft_c_t num_fft_csub(num_fft_c_t a, num_fft_c_t b) {
    num_fft_c_t r;
    r.re = a.re - b.re;
    r.im = a.im - b.im;
    return r;
}

num_fft_c_t num_fft_cmul(num_fft_c_t a, num_fft_c_t b) {
    num_fft_c_t r;
    r.re = a.re * b.re - a.im * b.im;
    r.im = a.re * b.im + a.im * b.re;
    return r;
}

void num_fft_butterfly(num_fft_c_t *data, int n) {
    int s, k, j;
    int log2n = 0;
    int temp = n;
    while (temp > 1) { log2n++; temp /= 2; }
    for (s = 1; s <= log2n; s++) {
        int m = 1 << s;
        num_fft_c_t wm;
        wm.re = 1.0;
        wm.im = -6.28318530718 / m;
        for (k = 0; k < n; k += m) {
            num_fft_c_t w;
            w.re = 1.0;
            w.im = 0.0;
            for (j = 0; j < m / 2; j++) {
                num_fft_c_t t = num_fft_cmul(w, data[k + j + m / 2]);
                num_fft_c_t u = data[k + j];
                data[k + j] = num_fft_cadd(u, t);
                data[k + j + m / 2] = num_fft_csub(u, t);
                w = num_fft_cmul(w, wm);
            }
        }
    }
}

double num_fft_magnitude(num_fft_c_t c) {
    return c.re * c.re + c.im * c.im;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1298: FFT butterfly should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1298: empty output");
    assert!(
        code.contains("fn num_fft_butterfly"),
        "C1298: Should contain num_fft_butterfly function"
    );
    assert!(
        code.contains("fn num_fft_cmul"),
        "C1298: Should contain num_fft_cmul function"
    );
}

/// C1299: Direct DFT computation (O(N^2) reference)
#[test]
fn c1299_dft_direct() {
    let c_code = r#"
typedef struct {
    double re;
    double im;
} num_dft_c_t;

double num_dft_cos_approx(double x) {
    double x2 = x * x;
    return 1.0 - x2 / 2.0 + x2 * x2 / 24.0 - x2 * x2 * x2 / 720.0;
}

double num_dft_sin_approx(double x) {
    double x2 = x * x;
    return x - x * x2 / 6.0 + x * x2 * x2 / 120.0 - x * x2 * x2 * x2 / 5040.0;
}

void num_dft_forward(const double *input, num_dft_c_t *output, int n) {
    int k, j;
    double pi2 = 6.28318530718;
    for (k = 0; k < n; k++) {
        output[k].re = 0.0;
        output[k].im = 0.0;
        for (j = 0; j < n; j++) {
            double angle = pi2 * k * j / n;
            output[k].re += input[j] * num_dft_cos_approx(angle);
            output[k].im -= input[j] * num_dft_sin_approx(angle);
        }
    }
}

void num_dft_power_spectrum(const num_dft_c_t *dft, double *power, int n) {
    int k;
    for (k = 0; k < n; k++) {
        power[k] = dft[k].re * dft[k].re + dft[k].im * dft[k].im;
    }
}

double num_dft_dominant_freq(const double *power, int n) {
    double max_p = power[1];
    int max_k = 1;
    int k;
    for (k = 2; k < n / 2; k++) {
        if (power[k] > max_p) {
            max_p = power[k];
            max_k = k;
        }
    }
    return (double)max_k;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1299: DFT direct should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1299: empty output");
    assert!(
        code.contains("fn num_dft_forward"),
        "C1299: Should contain num_dft_forward function"
    );
    assert!(
        code.contains("fn num_dft_power_spectrum"),
        "C1299: Should contain num_dft_power_spectrum function"
    );
}

/// C1300: Spectral method for Poisson equation using DFT
#[test]
fn c1300_spectral_method() {
    let c_code = r#"
typedef struct {
    double u[32];
    double rhs[32];
    int n;
    double dx;
} num_spec_t;

void num_spec_init(num_spec_t *sp, int n) {
    int i;
    sp->n = n;
    sp->dx = 1.0 / (n - 1);
    for (i = 0; i < n; i++) {
        sp->u[i] = 0.0;
        sp->rhs[i] = 0.0;
    }
}

double num_spec_sin_approx(double x) {
    double x2 = x * x;
    return x - x * x2 / 6.0 + x * x2 * x2 / 120.0;
}

void num_spec_dst_forward(const double *f, double *F, int n) {
    int k, j;
    double pi = 3.14159265358979;
    for (k = 1; k < n - 1; k++) {
        F[k] = 0.0;
        for (j = 1; j < n - 1; j++) {
            double angle = pi * k * j / (n - 1);
            F[k] += f[j] * num_spec_sin_approx(angle);
        }
        F[k] *= 2.0 / (n - 1);
    }
}

void num_spec_dst_inverse(const double *F, double *f, int n) {
    int k, j;
    double pi = 3.14159265358979;
    for (j = 1; j < n - 1; j++) {
        f[j] = 0.0;
        for (k = 1; k < n - 1; k++) {
            double angle = pi * k * j / (n - 1);
            f[j] += F[k] * num_spec_sin_approx(angle);
        }
    }
}

void num_spec_poisson_solve(num_spec_t *sp) {
    double F_rhs[32], F_u[32];
    int k;
    double pi = 3.14159265358979;
    double dx2 = sp->dx * sp->dx;
    num_spec_dst_forward(sp->rhs, F_rhs, sp->n);
    for (k = 1; k < sp->n - 1; k++) {
        double lambda = 2.0 * (1.0 - num_spec_sin_approx(pi * k / (sp->n - 1)));
        if (lambda > 1e-14) {
            F_u[k] = -F_rhs[k] * dx2 / lambda;
        } else {
            F_u[k] = 0.0;
        }
    }
    num_spec_dst_inverse(F_u, sp->u, sp->n);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1300: Spectral method should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1300: empty output");
    assert!(
        code.contains("fn num_spec_poisson_solve"),
        "C1300: Should contain num_spec_poisson_solve function"
    );
    assert!(
        code.contains("fn num_spec_dst_forward"),
        "C1300: Should contain num_spec_dst_forward function"
    );
}
