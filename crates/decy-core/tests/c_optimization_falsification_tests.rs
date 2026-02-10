//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1176-C1200: Optimization Algorithm implementations -- the kind of C code
//! found in numerical optimization libraries, operations research solvers,
//! and scientific computing frameworks.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world optimization patterns commonly found in
//! NLopt, GLPK, OR-Tools (C backend), GSL optimization, and similar
//! mathematical optimization libraries -- all expressed as valid C99.
//!
//! Organization:
//! - C1176-C1180: Classical optimization (gradient descent, Newton, golden section, bisection, secant)
//! - C1181-C1185: Linear programming (simplex, dual simplex, Hungarian, assignment, transportation)
//! - C1186-C1190: Metaheuristics (particle swarm, ant colony, differential evolution, harmony search, firefly)
//! - C1191-C1195: Constrained optimization (Lagrange, penalty, barrier, augmented Lagrangian, SQP)
//! - C1196-C1200: Combinatorial optimization (branch and bound, local search, 2-opt TSP, bin packing, job scheduling)

// ============================================================================
// C1176-C1180: Classical Optimization
// ============================================================================

#[test]
fn c1176_gradient_descent() {
    let c_code = r#"
void opt_gradient_descent(double *x, double *grad, int n, double lr, int max_iters, double tol) {
    int iter, j;
    double norm_sq;
    for (iter = 0; iter < max_iters; iter++) {
        norm_sq = 0.0;
        for (j = 0; j < n; j++) {
            grad[j] = 2.0 * x[j];
            norm_sq += grad[j] * grad[j];
        }
        if (norm_sq < tol * tol) {
            break;
        }
        for (j = 0; j < n; j++) {
            x[j] = x[j] - lr * grad[j];
        }
        lr *= 0.999;
    }
}

double opt_rosenbrock(double x, double y) {
    double a = 1.0 - x;
    double b = y - x * x;
    return a * a + 100.0 * b * b;
}

void opt_rosenbrock_grad(double x, double y, double *gx, double *gy) {
    *gx = -2.0 * (1.0 - x) + 200.0 * (y - x * x) * (-2.0 * x);
    *gy = 200.0 * (y - x * x);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1176: Gradient descent should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1177_newtons_method_optimization() {
    let c_code = r#"
double opt_newton_1d(double x0, int max_iters, double tol) {
    double x = x0;
    int i;
    for (i = 0; i < max_iters; i++) {
        double fx = x * x * x - 2.0 * x - 5.0;
        double fpx = 3.0 * x * x - 2.0;
        if (fpx == 0.0) {
            break;
        }
        double dx = fx / fpx;
        x = x - dx;
        if (dx > -tol && dx < tol) {
            break;
        }
    }
    return x;
}

void opt_newton_multidim(double *x, double *hessian, double *grad, int n, int max_iters, double tol) {
    int iter, i, j, k;
    double h_inv[16];
    double step[4];
    double norm;

    for (iter = 0; iter < max_iters; iter++) {
        for (i = 0; i < n; i++) {
            grad[i] = 2.0 * x[i] + 0.1 * x[i] * x[i] * x[i];
        }
        for (i = 0; i < n; i++) {
            for (j = 0; j < n; j++) {
                hessian[i * n + j] = (i == j) ? (2.0 + 0.3 * x[i] * x[i]) : 0.0;
            }
        }
        for (i = 0; i < n; i++) {
            step[i] = 0.0;
            for (j = 0; j < n; j++) {
                double inv_val = (i == j) ? 1.0 / hessian[i * n + j] : 0.0;
                step[i] += inv_val * grad[j];
            }
        }
        norm = 0.0;
        for (i = 0; i < n; i++) {
            x[i] -= step[i];
            norm += step[i] * step[i];
        }
        if (norm < tol * tol) {
            break;
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1177: Newton's method optimization should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1178_golden_section_search() {
    let c_code = r#"
double opt_golden_eval(double x) {
    return (x - 2.0) * (x - 2.0) + 3.0 * x + 1.0;
}

double opt_golden_section(double a, double b, double tol, int max_iters) {
    double phi = 1.6180339887;
    double resphi = 2.0 - phi;
    double x1 = a + resphi * (b - a);
    double x2 = b - resphi * (b - a);
    double f1 = opt_golden_eval(x1);
    double f2 = opt_golden_eval(x2);
    int i;

    for (i = 0; i < max_iters; i++) {
        if ((b - a) < tol) {
            break;
        }
        if (f1 < f2) {
            b = x2;
            x2 = x1;
            f2 = f1;
            x1 = a + resphi * (b - a);
            f1 = opt_golden_eval(x1);
        } else {
            a = x1;
            x1 = x2;
            f1 = f2;
            x2 = b - resphi * (b - a);
            f2 = opt_golden_eval(x2);
        }
    }
    return (a + b) / 2.0;
}

double opt_golden_section_general(double a, double b, double tol) {
    double gr = 0.6180339887;
    double c = b - gr * (b - a);
    double d = a + gr * (b - a);
    while ((b - a) > tol) {
        if (opt_golden_eval(c) < opt_golden_eval(d)) {
            b = d;
        } else {
            a = c;
        }
        c = b - gr * (b - a);
        d = a + gr * (b - a);
    }
    return (b + a) / 2.0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1178: Golden section search should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1179_bisection_root_finding() {
    let c_code = r#"
double opt_bisect_func(double x) {
    return x * x * x - x - 2.0;
}

double opt_bisection(double a, double b, double tol, int max_iters) {
    double fa = opt_bisect_func(a);
    double fb = opt_bisect_func(b);
    double mid, fmid;
    int i;

    if (fa * fb > 0.0) {
        return a;
    }

    for (i = 0; i < max_iters; i++) {
        mid = (a + b) / 2.0;
        fmid = opt_bisect_func(mid);

        if (fmid == 0.0 || (b - a) / 2.0 < tol) {
            return mid;
        }

        if (fa * fmid < 0.0) {
            b = mid;
            fb = fmid;
        } else {
            a = mid;
            fa = fmid;
        }
    }
    return (a + b) / 2.0;
}

int opt_count_roots(double start, double end, double step) {
    int count = 0;
    double x = start;
    double prev = opt_bisect_func(start);
    while (x < end) {
        x += step;
        double curr = opt_bisect_func(x);
        if (prev * curr < 0.0) {
            count++;
        }
        prev = curr;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1179: Bisection root finding should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1180_secant_method() {
    let c_code = r#"
double opt_secant_func(double x) {
    return x * x - 612.0;
}

double opt_secant_method(double x0, double x1, double tol, int max_iters) {
    double f0 = opt_secant_func(x0);
    double f1 = opt_secant_func(x1);
    double x2;
    int i;

    for (i = 0; i < max_iters; i++) {
        if (f1 - f0 == 0.0) {
            break;
        }
        x2 = x1 - f1 * (x1 - x0) / (f1 - f0);
        x0 = x1;
        f0 = f1;
        x1 = x2;
        f1 = opt_secant_func(x2);
        if (f1 > -tol && f1 < tol) {
            break;
        }
    }
    return x1;
}

double opt_regula_falsi(double a, double b, double tol, int max_iters) {
    double fa = opt_secant_func(a);
    double fb = opt_secant_func(b);
    double c, fc;
    int i;

    for (i = 0; i < max_iters; i++) {
        c = (a * fb - b * fa) / (fb - fa);
        fc = opt_secant_func(c);
        if (fc > -tol && fc < tol) {
            return c;
        }
        if (fa * fc < 0.0) {
            b = c;
            fb = fc;
        } else {
            a = c;
            fa = fc;
        }
    }
    return c;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1180: Secant method should transpile: {:?}",
        result.err()
    );
}

// ============================================================================
// C1181-C1185: Linear Programming
// ============================================================================

#[test]
fn c1181_simplex_method() {
    let c_code = r#"
void opt_simplex_pivot(double *tableau, int rows, int cols, int pivot_row, int pivot_col) {
    int i, j;
    double pivot_val = tableau[pivot_row * cols + pivot_col];
    for (j = 0; j < cols; j++) {
        tableau[pivot_row * cols + j] /= pivot_val;
    }
    for (i = 0; i < rows; i++) {
        if (i != pivot_row) {
            double factor = tableau[i * cols + pivot_col];
            for (j = 0; j < cols; j++) {
                tableau[i * cols + j] -= factor * tableau[pivot_row * cols + j];
            }
        }
    }
}

int opt_simplex_solve(double *tableau, int m, int n) {
    int cols = n + m + 2;
    int rows = m + 1;
    int iter;

    for (iter = 0; iter < 100; iter++) {
        int pivot_col = -1;
        double min_val = 0.0;
        int j;
        for (j = 0; j < cols - 1; j++) {
            if (tableau[(rows - 1) * cols + j] < min_val) {
                min_val = tableau[(rows - 1) * cols + j];
                pivot_col = j;
            }
        }
        if (pivot_col == -1) {
            return 1;
        }

        int pivot_row = -1;
        double min_ratio = 1e30;
        int i;
        for (i = 0; i < rows - 1; i++) {
            if (tableau[i * cols + pivot_col] > 0.0) {
                double ratio = tableau[i * cols + cols - 1] / tableau[i * cols + pivot_col];
                if (ratio < min_ratio) {
                    min_ratio = ratio;
                    pivot_row = i;
                }
            }
        }
        if (pivot_row == -1) {
            return -1;
        }
        opt_simplex_pivot(tableau, rows, cols, pivot_row, pivot_col);
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1181: Simplex method should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1182_dual_simplex() {
    let c_code = r#"
int opt_dual_simplex_find_leaving(double *tableau, int rows, int cols) {
    int i;
    int leaving = -1;
    double min_rhs = 0.0;
    for (i = 0; i < rows - 1; i++) {
        double rhs = tableau[i * cols + cols - 1];
        if (rhs < min_rhs) {
            min_rhs = rhs;
            leaving = i;
        }
    }
    return leaving;
}

int opt_dual_simplex_find_entering(double *tableau, int rows, int cols, int leaving) {
    int j;
    int entering = -1;
    double min_ratio = 1e30;
    for (j = 0; j < cols - 1; j++) {
        double aij = tableau[leaving * cols + j];
        if (aij < 0.0) {
            double cj = tableau[(rows - 1) * cols + j];
            double ratio = -cj / aij;
            if (ratio < min_ratio) {
                min_ratio = ratio;
                entering = j;
            }
        }
    }
    return entering;
}

int opt_dual_simplex(double *tableau, int m, int n) {
    int cols = n + 1;
    int rows = m + 1;
    int iter;
    for (iter = 0; iter < 200; iter++) {
        int leaving = opt_dual_simplex_find_leaving(tableau, rows, cols);
        if (leaving == -1) {
            return 1;
        }
        int entering = opt_dual_simplex_find_entering(tableau, rows, cols, leaving);
        if (entering == -1) {
            return -1;
        }
        double pivot = tableau[leaving * cols + entering];
        int i, j;
        for (j = 0; j < cols; j++) {
            tableau[leaving * cols + j] /= pivot;
        }
        for (i = 0; i < rows; i++) {
            if (i != leaving) {
                double factor = tableau[i * cols + entering];
                for (j = 0; j < cols; j++) {
                    tableau[i * cols + j] -= factor * tableau[leaving * cols + j];
                }
            }
        }
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1182: Dual simplex should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1183_hungarian_algorithm() {
    let c_code = r#"
void opt_hungarian_subtract_row_min(double *cost, int n) {
    int i, j;
    for (i = 0; i < n; i++) {
        double row_min = cost[i * n];
        for (j = 1; j < n; j++) {
            if (cost[i * n + j] < row_min) {
                row_min = cost[i * n + j];
            }
        }
        for (j = 0; j < n; j++) {
            cost[i * n + j] -= row_min;
        }
    }
}

void opt_hungarian_subtract_col_min(double *cost, int n) {
    int i, j;
    for (j = 0; j < n; j++) {
        double col_min = cost[j];
        for (i = 1; i < n; i++) {
            if (cost[i * n + j] < col_min) {
                col_min = cost[i * n + j];
            }
        }
        for (i = 0; i < n; i++) {
            cost[i * n + j] -= col_min;
        }
    }
}

int opt_hungarian_count_zeros(double *cost, int n, int *assignment) {
    int covered_rows[8];
    int covered_cols[8];
    int i, j;
    int count = 0;

    for (i = 0; i < n; i++) {
        covered_rows[i] = 0;
        covered_cols[i] = 0;
        assignment[i] = -1;
    }
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            if (cost[i * n + j] == 0.0 && covered_rows[i] == 0 && covered_cols[j] == 0) {
                assignment[i] = j;
                covered_rows[i] = 1;
                covered_cols[j] = 1;
                count++;
            }
        }
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1183: Hungarian algorithm should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1184_assignment_problem() {
    let c_code = r#"
double opt_assignment_greedy(double *cost, int n, int *assignment) {
    int assigned_cols[8];
    int i, j;
    double total_cost = 0.0;

    for (j = 0; j < n; j++) {
        assigned_cols[j] = 0;
    }
    for (i = 0; i < n; i++) {
        assignment[i] = -1;
    }

    for (i = 0; i < n; i++) {
        double best = 1e30;
        int best_j = -1;
        for (j = 0; j < n; j++) {
            if (assigned_cols[j] == 0 && cost[i * n + j] < best) {
                best = cost[i * n + j];
                best_j = j;
            }
        }
        if (best_j >= 0) {
            assignment[i] = best_j;
            assigned_cols[best_j] = 1;
            total_cost += best;
        }
    }
    return total_cost;
}

double opt_assignment_evaluate(double *cost, int n, int *assignment) {
    double total = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        if (assignment[i] >= 0 && assignment[i] < n) {
            total += cost[i * n + assignment[i]];
        }
    }
    return total;
}

int opt_assignment_is_valid(int *assignment, int n) {
    int used[8];
    int i;
    for (i = 0; i < n; i++) {
        used[i] = 0;
    }
    for (i = 0; i < n; i++) {
        if (assignment[i] < 0 || assignment[i] >= n) {
            return 0;
        }
        if (used[assignment[i]]) {
            return 0;
        }
        used[assignment[i]] = 1;
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1184: Assignment problem should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1185_transportation_problem() {
    let c_code = r#"
void opt_transport_northwest(double *cost, double *supply, double *demand,
                              double *alloc, int m, int n) {
    int i = 0;
    int j = 0;
    double s[8];
    double d[8];
    int k;

    for (k = 0; k < m; k++) s[k] = supply[k];
    for (k = 0; k < n; k++) d[k] = demand[k];
    for (k = 0; k < m * n; k++) alloc[k] = 0.0;

    while (i < m && j < n) {
        double qty;
        if (s[i] < d[j]) {
            qty = s[i];
        } else {
            qty = d[j];
        }
        alloc[i * n + j] = qty;
        s[i] -= qty;
        d[j] -= qty;
        if (s[i] == 0.0) {
            i++;
        } else {
            j++;
        }
    }
}

double opt_transport_cost(double *cost, double *alloc, int m, int n) {
    double total = 0.0;
    int i, j;
    for (i = 0; i < m; i++) {
        for (j = 0; j < n; j++) {
            total += cost[i * n + j] * alloc[i * n + j];
        }
    }
    return total;
}

int opt_transport_is_feasible(double *alloc, double *supply, double *demand, int m, int n) {
    int i, j;
    for (i = 0; i < m; i++) {
        double row_sum = 0.0;
        for (j = 0; j < n; j++) {
            row_sum += alloc[i * n + j];
        }
        double diff = row_sum - supply[i];
        if (diff > 0.001 || diff < -0.001) {
            return 0;
        }
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1185: Transportation problem should transpile: {:?}",
        result.err()
    );
}

// ============================================================================
// C1186-C1190: Metaheuristics
// ============================================================================

#[test]
fn c1186_particle_swarm_optimization() {
    let c_code = r#"
typedef struct {
    double pos[4];
    double vel[4];
    double best_pos[4];
    double best_val;
} opt_particle_t;

double opt_pso_objective(double *x, int dim) {
    double sum = 0.0;
    int i;
    for (i = 0; i < dim; i++) {
        sum += x[i] * x[i];
    }
    return sum;
}

void opt_pso_update(opt_particle_t *swarm, int n_particles, int dim,
                     double *global_best, double w, double c1, double c2) {
    int i, d;
    double r1 = 0.5;
    double r2 = 0.3;

    for (i = 0; i < n_particles; i++) {
        for (d = 0; d < dim; d++) {
            swarm[i].vel[d] = w * swarm[i].vel[d]
                + c1 * r1 * (swarm[i].best_pos[d] - swarm[i].pos[d])
                + c2 * r2 * (global_best[d] - swarm[i].pos[d]);
            swarm[i].pos[d] += swarm[i].vel[d];
        }
        double val = opt_pso_objective(swarm[i].pos, dim);
        if (val < swarm[i].best_val) {
            swarm[i].best_val = val;
            for (d = 0; d < dim; d++) {
                swarm[i].best_pos[d] = swarm[i].pos[d];
            }
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1186: Particle swarm optimization should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1187_ant_colony_optimization() {
    let c_code = r#"
void opt_aco_update_pheromone(double *pheromone, double *delta, int n,
                               double evap_rate) {
    int i, j;
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            pheromone[i * n + j] = (1.0 - evap_rate) * pheromone[i * n + j]
                                     + delta[i * n + j];
        }
    }
}

int opt_aco_select_next(double *pheromone, double *distance, int *visited,
                         int current, int n, double alpha, double beta) {
    double probs[16];
    double total = 0.0;
    int j;

    for (j = 0; j < n; j++) {
        if (visited[j]) {
            probs[j] = 0.0;
        } else {
            double tau = pheromone[current * n + j];
            double eta = 1.0 / (distance[current * n + j] + 0.001);
            double p = 1.0;
            int k;
            for (k = 0; k < (int)alpha; k++) p *= tau;
            double q = 1.0;
            for (k = 0; k < (int)beta; k++) q *= eta;
            probs[j] = p * q;
            total += probs[j];
        }
    }

    if (total == 0.0) {
        for (j = 0; j < n; j++) {
            if (!visited[j]) return j;
        }
        return -1;
    }

    double cumulative = 0.0;
    double threshold = total * 0.5;
    for (j = 0; j < n; j++) {
        cumulative += probs[j];
        if (cumulative >= threshold) {
            return j;
        }
    }
    return n - 1;
}

double opt_aco_tour_length(int *tour, double *distance, int n) {
    double total = 0.0;
    int i;
    for (i = 0; i < n - 1; i++) {
        total += distance[tour[i] * n + tour[i + 1]];
    }
    total += distance[tour[n - 1] * n + tour[0]];
    return total;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1187: Ant colony optimization should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1188_differential_evolution() {
    let c_code = r#"
double opt_de_sphere(double *x, int dim) {
    double sum = 0.0;
    int i;
    for (i = 0; i < dim; i++) {
        sum += x[i] * x[i];
    }
    return sum;
}

void opt_de_mutate(double *population, double *mutant, int pop_size, int dim,
                    int target, double F) {
    int r1 = (target + 1) % pop_size;
    int r2 = (target + 2) % pop_size;
    int r3 = (target + 3) % pop_size;
    int j;

    for (j = 0; j < dim; j++) {
        mutant[j] = population[r1 * dim + j]
                   + F * (population[r2 * dim + j] - population[r3 * dim + j]);
    }
}

void opt_de_crossover(double *target, double *mutant, double *trial,
                       int dim, double CR) {
    int j;
    int jrand = dim / 2;
    for (j = 0; j < dim; j++) {
        double rand_val = (double)j / (double)dim;
        if (rand_val < CR || j == jrand) {
            trial[j] = mutant[j];
        } else {
            trial[j] = target[j];
        }
    }
}

void opt_de_iterate(double *population, double *fitness, int pop_size, int dim,
                     double F, double CR) {
    double mutant[8];
    double trial[8];
    int i;

    for (i = 0; i < pop_size; i++) {
        opt_de_mutate(population, mutant, pop_size, dim, i, F);
        opt_de_crossover(&population[i * dim], mutant, trial, dim, CR);
        double trial_fit = opt_de_sphere(trial, dim);
        if (trial_fit < fitness[i]) {
            int j;
            for (j = 0; j < dim; j++) {
                population[i * dim + j] = trial[j];
            }
            fitness[i] = trial_fit;
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1188: Differential evolution should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1189_harmony_search() {
    let c_code = r#"
double opt_hs_objective(double *x, int dim) {
    double sum = 0.0;
    int i;
    for (i = 0; i < dim; i++) {
        sum += (x[i] - 1.0) * (x[i] - 1.0);
    }
    return sum;
}

void opt_hs_improvise(double *memory, double *fitness, int hms, int dim,
                       double hmcr, double par, double bw,
                       double *new_harmony) {
    int j, r;
    for (j = 0; j < dim; j++) {
        double test = (double)j / (double)(dim + 1);
        if (test < hmcr) {
            r = j % hms;
            new_harmony[j] = memory[r * dim + j];
            double test2 = (double)(j + 1) / (double)(dim + 2);
            if (test2 < par) {
                new_harmony[j] += bw * (0.5 - test2);
            }
        } else {
            new_harmony[j] = -5.0 + 10.0 * test;
        }
    }
}

void opt_hs_update_memory(double *memory, double *fitness, int hms, int dim,
                           double *new_harmony, double new_fit) {
    int worst = 0;
    int i;
    for (i = 1; i < hms; i++) {
        if (fitness[i] > fitness[worst]) {
            worst = i;
        }
    }
    if (new_fit < fitness[worst]) {
        int j;
        for (j = 0; j < dim; j++) {
            memory[worst * dim + j] = new_harmony[j];
        }
        fitness[worst] = new_fit;
    }
}

double opt_hs_best(double *fitness, int hms) {
    double best = fitness[0];
    int i;
    for (i = 1; i < hms; i++) {
        if (fitness[i] < best) {
            best = fitness[i];
        }
    }
    return best;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1189: Harmony search should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1190_firefly_algorithm() {
    let c_code = r#"
double opt_firefly_objective(double *x, int dim) {
    double sum = 0.0;
    int i;
    for (i = 0; i < dim; i++) {
        sum += x[i] * x[i] - 10.0 * 1.0 + 10.0;
    }
    return sum;
}

double opt_firefly_distance(double *a, double *b, int dim) {
    double sum = 0.0;
    int i;
    for (i = 0; i < dim; i++) {
        double d = a[i] - b[i];
        sum += d * d;
    }
    return sum;
}

void opt_firefly_move(double *xi, double *xj, int dim,
                       double beta0, double gamma_val, double alpha) {
    double r2 = opt_firefly_distance(xi, xj, dim);
    double beta = beta0 / (1.0 + gamma_val * r2);
    int d;

    for (d = 0; d < dim; d++) {
        double rand_term = alpha * (0.5 - (double)d / (double)(dim + 1));
        xi[d] = xi[d] + beta * (xj[d] - xi[d]) + rand_term;
    }
}

void opt_firefly_iterate(double *population, double *fitness, int n, int dim,
                          double beta0, double gamma_val, double alpha) {
    int i, j;
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            if (fitness[j] < fitness[i]) {
                opt_firefly_move(&population[i * dim], &population[j * dim],
                                  dim, beta0, gamma_val, alpha);
                fitness[i] = opt_firefly_objective(&population[i * dim], dim);
            }
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1190: Firefly algorithm should transpile: {:?}",
        result.err()
    );
}

// ============================================================================
// C1191-C1195: Constrained Optimization
// ============================================================================

#[test]
fn c1191_lagrange_multiplier() {
    let c_code = r#"
double opt_lagrange_objective(double x, double y) {
    return x * x + y * y;
}

double opt_lagrange_constraint(double x, double y) {
    return x + y - 1.0;
}

void opt_lagrange_solve(double *x, double *y, double *lambda,
                         double lr, int max_iters, double tol) {
    int iter;
    double lam = *lambda;

    for (iter = 0; iter < max_iters; iter++) {
        double grad_x = 2.0 * (*x) + lam;
        double grad_y = 2.0 * (*y) + lam;
        double g = opt_lagrange_constraint(*x, *y);

        *x -= lr * grad_x;
        *y -= lr * grad_y;
        lam += lr * g;

        double norm = grad_x * grad_x + grad_y * grad_y + g * g;
        if (norm < tol * tol) {
            break;
        }
    }
    *lambda = lam;
}

void opt_lagrange_kkt_check(double x, double y, double lambda,
                             double *stationarity, double *feasibility) {
    *stationarity = (2.0 * x + lambda) * (2.0 * x + lambda)
                  + (2.0 * y + lambda) * (2.0 * y + lambda);
    *feasibility = opt_lagrange_constraint(x, y);
    if (*feasibility < 0.0) *feasibility = -*feasibility;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1191: Lagrange multiplier should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1192_penalty_method() {
    let c_code = r#"
double opt_penalty_objective(double *x, int n) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += (x[i] - 1.0) * (x[i] - 1.0);
    }
    return sum;
}

double opt_penalty_constraint_violation(double *x, int n) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += x[i];
    }
    double violation = sum - 2.0;
    if (violation > 0.0) {
        return violation * violation;
    }
    return 0.0;
}

double opt_penalty_augmented(double *x, int n, double rho) {
    return opt_penalty_objective(x, n) + rho * opt_penalty_constraint_violation(x, n);
}

void opt_penalty_minimize(double *x, int n, double rho_init, double rho_mult,
                           int outer_iters, int inner_iters, double lr) {
    double rho = rho_init;
    int outer, inner, i;

    for (outer = 0; outer < outer_iters; outer++) {
        for (inner = 0; inner < inner_iters; inner++) {
            double sum = 0.0;
            for (i = 0; i < n; i++) sum += x[i];
            double g = sum - 2.0;
            double penalty_grad = (g > 0.0) ? 2.0 * rho * g : 0.0;

            for (i = 0; i < n; i++) {
                double grad = 2.0 * (x[i] - 1.0) + penalty_grad;
                x[i] -= lr * grad;
            }
        }
        rho *= rho_mult;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1192: Penalty method should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1193_barrier_method() {
    let c_code = r#"
double opt_barrier_log(double x) {
    if (x <= 0.0) return -1e30;
    double result = 0.0;
    double term = (x - 1.0);
    result = term - term * term / 2.0 + term * term * term / 3.0;
    return result;
}

double opt_barrier_objective(double *x, int n) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += x[i] * x[i];
    }
    return sum;
}

double opt_barrier_function(double *x, int n, double t) {
    double f = opt_barrier_objective(x, n);
    double barrier = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        double slack = 5.0 - x[i];
        if (slack <= 0.0) {
            return 1e30;
        }
        barrier += opt_barrier_log(slack);
    }
    return t * f - barrier;
}

void opt_barrier_minimize(double *x, int n, double t_init, double mu,
                           int outer_iters, int inner_iters, double lr) {
    double t = t_init;
    int outer, inner, i;

    for (outer = 0; outer < outer_iters; outer++) {
        for (inner = 0; inner < inner_iters; inner++) {
            for (i = 0; i < n; i++) {
                double slack = 5.0 - x[i];
                double grad_f = 2.0 * t * x[i];
                double grad_b = (slack > 0.001) ? 1.0 / slack : 1000.0;
                x[i] -= lr * (grad_f + grad_b);
            }
        }
        t *= mu;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1193: Barrier method should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1194_augmented_lagrangian() {
    let c_code = r#"
double opt_auglag_objective(double *x, int n) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += x[i] * x[i];
    }
    return sum;
}

double opt_auglag_constraint(double *x, int n) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += x[i];
    }
    return sum - 1.0;
}

double opt_auglag_function(double *x, int n, double lambda, double rho) {
    double f = opt_auglag_objective(x, n);
    double c = opt_auglag_constraint(x, n);
    return f + lambda * c + (rho / 2.0) * c * c;
}

void opt_auglag_solve(double *x, int n, double *lambda, double *rho,
                       int outer_iters, int inner_iters, double lr) {
    int outer, inner, i;
    double lam = *lambda;
    double r = *rho;

    for (outer = 0; outer < outer_iters; outer++) {
        for (inner = 0; inner < inner_iters; inner++) {
            double c = opt_auglag_constraint(x, n);
            for (i = 0; i < n; i++) {
                double grad_f = 2.0 * x[i];
                double grad_c = 1.0;
                double grad = grad_f + (lam + r * c) * grad_c;
                x[i] -= lr * grad;
            }
        }
        double c = opt_auglag_constraint(x, n);
        lam += r * c;
        r *= 1.5;
    }
    *lambda = lam;
    *rho = r;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1194: Augmented Lagrangian should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1195_sqp_step() {
    let c_code = r#"
void opt_sqp_gradient(double *x, double *grad, int n) {
    int i;
    for (i = 0; i < n; i++) {
        grad[i] = 2.0 * x[i] + 0.5;
    }
}

void opt_sqp_hessian_approx(double *x, double *H, int n) {
    int i, j;
    for (i = 0; i < n; i++) {
        for (j = 0; j < n; j++) {
            H[i * n + j] = (i == j) ? 2.0 : 0.0;
        }
    }
}

double opt_sqp_constraint(double *x, int n) {
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        sum += x[i] * x[i];
    }
    return sum - 1.0;
}

void opt_sqp_constraint_grad(double *x, double *gc, int n) {
    int i;
    for (i = 0; i < n; i++) {
        gc[i] = 2.0 * x[i];
    }
}

void opt_sqp_step(double *x, int n, double *lambda, double lr, int max_iters) {
    double grad[4];
    double H[16];
    double gc[4];
    double step[4];
    int iter, i;

    for (iter = 0; iter < max_iters; iter++) {
        opt_sqp_gradient(x, grad, n);
        opt_sqp_hessian_approx(x, H, n);
        double c = opt_sqp_constraint(x, n);
        opt_sqp_constraint_grad(x, gc, n);

        for (i = 0; i < n; i++) {
            double rhs = -(grad[i] + (*lambda) * gc[i]);
            step[i] = rhs / H[i * n + i];
        }

        double gc_dot_step = 0.0;
        for (i = 0; i < n; i++) {
            gc_dot_step += gc[i] * step[i];
        }
        double dlambda = -(c + gc_dot_step);

        for (i = 0; i < n; i++) {
            x[i] += lr * step[i];
        }
        *lambda += lr * dlambda;

        double norm = 0.0;
        for (i = 0; i < n; i++) norm += step[i] * step[i];
        if (norm < 1e-10) break;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1195: SQP step should transpile: {:?}",
        result.err()
    );
}

// ============================================================================
// C1196-C1200: Combinatorial Optimization
// ============================================================================

#[test]
fn c1196_branch_and_bound() {
    let c_code = r#"
typedef struct {
    int items[16];
    int count;
    double value;
    double weight;
} opt_bb_node_t;

double opt_bb_bound(double *values, double *weights, int n,
                     double capacity, int level, double current_val, double current_wt) {
    double bound = current_val;
    double remaining = capacity - current_wt;
    int i;

    for (i = level; i < n && remaining > 0.0; i++) {
        if (weights[i] <= remaining) {
            bound += values[i];
            remaining -= weights[i];
        } else {
            bound += values[i] * (remaining / weights[i]);
            remaining = 0.0;
        }
    }
    return bound;
}

double opt_bb_knapsack(double *values, double *weights, int n, double capacity,
                        int *best_items) {
    int stack_level[64];
    int stack_include[64];
    double stack_val[64];
    double stack_wt[64];
    int sp = 0;
    double best_val = 0.0;
    int i;

    for (i = 0; i < n; i++) best_items[i] = 0;

    stack_level[sp] = 0;
    stack_include[sp] = 0;
    stack_val[sp] = 0.0;
    stack_wt[sp] = 0.0;
    sp++;

    while (sp > 0) {
        sp--;
        int level = stack_level[sp];
        double val = stack_val[sp];
        double wt = stack_wt[sp];

        if (level >= n) {
            if (val > best_val) {
                best_val = val;
            }
            continue;
        }

        double bound = opt_bb_bound(values, weights, n, capacity, level, val, wt);
        if (bound <= best_val) {
            continue;
        }

        if (sp < 62 && wt + weights[level] <= capacity) {
            stack_level[sp] = level + 1;
            stack_include[sp] = 1;
            stack_val[sp] = val + values[level];
            stack_wt[sp] = wt + weights[level];
            sp++;
        }
        if (sp < 62) {
            stack_level[sp] = level + 1;
            stack_include[sp] = 0;
            stack_val[sp] = val;
            stack_wt[sp] = wt;
            sp++;
        }
    }
    return best_val;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1196: Branch and bound should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1197_local_search() {
    let c_code = r#"
double opt_local_eval(int *solution, double *cost_matrix, int n) {
    double total = 0.0;
    int i;
    for (i = 0; i < n - 1; i++) {
        total += cost_matrix[solution[i] * n + solution[i + 1]];
    }
    total += cost_matrix[solution[n - 1] * n + solution[0]];
    return total;
}

void opt_local_swap(int *solution, int i, int j) {
    int temp = solution[i];
    solution[i] = solution[j];
    solution[j] = temp;
}

int opt_local_search_step(int *solution, double *cost_matrix, int n) {
    double current_cost = opt_local_eval(solution, cost_matrix, n);
    int i, j;
    int improved = 0;

    for (i = 0; i < n - 1; i++) {
        for (j = i + 1; j < n; j++) {
            opt_local_swap(solution, i, j);
            double new_cost = opt_local_eval(solution, cost_matrix, n);
            if (new_cost < current_cost) {
                current_cost = new_cost;
                improved = 1;
            } else {
                opt_local_swap(solution, i, j);
            }
        }
    }
    return improved;
}

double opt_local_search(int *solution, double *cost_matrix, int n, int max_iters) {
    int iter;
    for (iter = 0; iter < max_iters; iter++) {
        if (!opt_local_search_step(solution, cost_matrix, n)) {
            break;
        }
    }
    return opt_local_eval(solution, cost_matrix, n);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1197: Local search should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1198_two_opt_tsp() {
    let c_code = r#"
double opt_tsp_distance(double *coords, int i, int j) {
    double dx = coords[2 * i] - coords[2 * j];
    double dy = coords[2 * i + 1] - coords[2 * j + 1];
    double dist_sq = dx * dx + dy * dy;
    double guess = dist_sq / 2.0;
    int k;
    for (k = 0; k < 10; k++) {
        if (guess <= 0.0) break;
        guess = (guess + dist_sq / guess) / 2.0;
    }
    return guess;
}

double opt_tsp_tour_length(int *tour, double *coords, int n) {
    double total = 0.0;
    int i;
    for (i = 0; i < n - 1; i++) {
        total += opt_tsp_distance(coords, tour[i], tour[i + 1]);
    }
    total += opt_tsp_distance(coords, tour[n - 1], tour[0]);
    return total;
}

void opt_tsp_reverse_segment(int *tour, int i, int j) {
    while (i < j) {
        int temp = tour[i];
        tour[i] = tour[j];
        tour[j] = temp;
        i++;
        j--;
    }
}

double opt_tsp_two_opt(int *tour, double *coords, int n, int max_iters) {
    int iter;
    int improved = 1;

    for (iter = 0; iter < max_iters && improved; iter++) {
        improved = 0;
        int i, j;
        for (i = 0; i < n - 1; i++) {
            for (j = i + 2; j < n; j++) {
                double d1 = opt_tsp_distance(coords, tour[i], tour[i + 1])
                          + opt_tsp_distance(coords, tour[j], tour[(j + 1) % n]);
                double d2 = opt_tsp_distance(coords, tour[i], tour[j])
                          + opt_tsp_distance(coords, tour[i + 1], tour[(j + 1) % n]);
                if (d2 < d1) {
                    opt_tsp_reverse_segment(tour, i + 1, j);
                    improved = 1;
                }
            }
        }
    }
    return opt_tsp_tour_length(tour, coords, n);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1198: 2-opt TSP should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1199_bin_packing_first_fit() {
    let c_code = r#"
typedef struct {
    double items[32];
    int count;
    double remaining;
} opt_bin_t;

int opt_bin_packing_ff(double *items, int n, double capacity, opt_bin_t *bins, int max_bins) {
    int num_bins = 0;
    int i, j;

    for (i = 0; i < max_bins; i++) {
        bins[i].count = 0;
        bins[i].remaining = capacity;
    }

    for (i = 0; i < n; i++) {
        int placed = 0;
        for (j = 0; j < num_bins; j++) {
            if (bins[j].remaining >= items[i]) {
                bins[j].items[bins[j].count] = items[i];
                bins[j].count++;
                bins[j].remaining -= items[i];
                placed = 1;
                break;
            }
        }
        if (!placed && num_bins < max_bins) {
            bins[num_bins].items[0] = items[i];
            bins[num_bins].count = 1;
            bins[num_bins].remaining = capacity - items[i];
            num_bins++;
        }
    }
    return num_bins;
}

int opt_bin_packing_ffd(double *items, int n, double capacity, opt_bin_t *bins, int max_bins) {
    double sorted[64];
    int i, j;
    for (i = 0; i < n; i++) sorted[i] = items[i];

    for (i = 0; i < n - 1; i++) {
        for (j = i + 1; j < n; j++) {
            if (sorted[j] > sorted[i]) {
                double temp = sorted[i];
                sorted[i] = sorted[j];
                sorted[j] = temp;
            }
        }
    }
    return opt_bin_packing_ff(sorted, n, capacity, bins, max_bins);
}

double opt_bin_utilization(opt_bin_t *bins, int num_bins, double capacity) {
    double total_used = 0.0;
    int i;
    for (i = 0; i < num_bins; i++) {
        total_used += (capacity - bins[i].remaining);
    }
    if (num_bins == 0) return 0.0;
    return total_used / (num_bins * capacity);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1199: Bin packing first fit should transpile: {:?}",
        result.err()
    );
}

#[test]
fn c1200_job_scheduling() {
    let c_code = r#"
typedef struct {
    int id;
    double processing_time;
    double due_date;
    double weight;
    double completion_time;
} opt_job_t;

void opt_schedule_spt(opt_job_t *jobs, int n) {
    int i, j;
    for (i = 0; i < n - 1; i++) {
        for (j = i + 1; j < n; j++) {
            if (jobs[j].processing_time < jobs[i].processing_time) {
                opt_job_t temp = jobs[i];
                jobs[i] = jobs[j];
                jobs[j] = temp;
            }
        }
    }
    double time = 0.0;
    for (i = 0; i < n; i++) {
        time += jobs[i].processing_time;
        jobs[i].completion_time = time;
    }
}

double opt_schedule_weighted_tardiness(opt_job_t *jobs, int n) {
    double total = 0.0;
    int i;
    for (i = 0; i < n; i++) {
        double tardiness = jobs[i].completion_time - jobs[i].due_date;
        if (tardiness > 0.0) {
            total += jobs[i].weight * tardiness;
        }
    }
    return total;
}

void opt_schedule_edd(opt_job_t *jobs, int n) {
    int i, j;
    for (i = 0; i < n - 1; i++) {
        for (j = i + 1; j < n; j++) {
            if (jobs[j].due_date < jobs[i].due_date) {
                opt_job_t temp = jobs[i];
                jobs[i] = jobs[j];
                jobs[j] = temp;
            }
        }
    }
    double time = 0.0;
    for (i = 0; i < n; i++) {
        time += jobs[i].processing_time;
        jobs[i].completion_time = time;
    }
}

double opt_schedule_makespan(opt_job_t *jobs, int n, int num_machines) {
    double machine_time[8];
    int m;
    int i;
    for (m = 0; m < num_machines; m++) {
        machine_time[m] = 0.0;
    }
    for (i = 0; i < n; i++) {
        int best_m = 0;
        for (m = 1; m < num_machines; m++) {
            if (machine_time[m] < machine_time[best_m]) {
                best_m = m;
            }
        }
        machine_time[best_m] += jobs[i].processing_time;
        jobs[i].completion_time = machine_time[best_m];
    }
    double makespan = machine_time[0];
    for (m = 1; m < num_machines; m++) {
        if (machine_time[m] > makespan) {
            makespan = machine_time[m];
        }
    }
    return makespan;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1200: Job scheduling should transpile: {:?}",
        result.err()
    );
}
