//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C676-C700: Mathematical and Numerical Algorithms -- the kind of C code
//! found in numerical libraries, algebra systems, geometry engines, and
//! scientific computing frameworks.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world mathematical programming patterns commonly
//! found in GMP, NTL, FLINT, GSL, CGAL, and similar math libraries --
//! all expressed as valid C99.
//!
//! Organization:
//! - C676-C680: Fixed-point and big integer (Q16.16, bigint add/mul, mod exp, prime sieve)
//! - C681-C685: Classical algorithms (ext GCD, Horner, Newton-Raphson, bisection, Simpson)
//! - C686-C690: Statistical and noise (num diff, linear regression, EMA, Perlin, simplex)
//! - C691-C695: Curves and transforms (Bezier, B-spline, Catmull-Rom, quaternion, dual number)
//! - C696-C700: Number systems (complex, interval, rational, Montgomery, NTT)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C676-C680: Fixed-Point and Big Integer Arithmetic
// ============================================================================

#[test]
fn c676_fixed_point_q16_16_arithmetic() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef int int32_t;
typedef long long int64_t;

typedef int32_t fixed_t;

fixed_t fixed_from_int(int val) {
    return val << 16;
}

int fixed_to_int(fixed_t val) {
    return val >> 16;
}

fixed_t fixed_add(fixed_t a, fixed_t b) {
    return a + b;
}

fixed_t fixed_sub(fixed_t a, fixed_t b) {
    return a - b;
}

fixed_t fixed_mul(fixed_t a, fixed_t b) {
    int64_t temp = (int64_t)a * (int64_t)b;
    return (fixed_t)(temp >> 16);
}

fixed_t fixed_div(fixed_t a, fixed_t b) {
    int64_t temp = ((int64_t)a) << 16;
    return (fixed_t)(temp / b);
}

fixed_t fixed_lerp(fixed_t a, fixed_t b, fixed_t t) {
    return fixed_add(a, fixed_mul(fixed_sub(b, a), t));
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C676: Fixed-point Q16.16 arithmetic should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C676: empty output");
    assert!(
        code.contains("fn fixed_mul"),
        "C676: Should contain fixed_mul function"
    );
    assert!(
        code.contains("fn fixed_div"),
        "C676: Should contain fixed_div function"
    );
    Ok(())
}

#[test]
fn c677_big_integer_addition() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t digits[64];
    int len;
} bigint_t;

void bigint_zero(bigint_t *a) {
    int i;
    for (i = 0; i < 64; i = i + 1) {
        a->digits[i] = 0;
    }
    a->len = 1;
}

void bigint_add(const bigint_t *a, const bigint_t *b, bigint_t *result) {
    uint32_t carry = 0;
    int max_len = a->len;
    int i;
    if (b->len > max_len) {
        max_len = b->len;
    }
    for (i = 0; i < max_len; i = i + 1) {
        uint32_t sum = carry;
        if (i < a->len) {
            sum = sum + a->digits[i];
        }
        if (i < b->len) {
            sum = sum + b->digits[i];
        }
        result->digits[i] = sum & 0xFFFFFFFF;
        carry = (sum < a->digits[i]) ? 1 : 0;
    }
    if (carry) {
        result->digits[max_len] = carry;
        result->len = max_len + 1;
    } else {
        result->len = max_len;
    }
}

int bigint_compare(const bigint_t *a, const bigint_t *b) {
    int i;
    if (a->len != b->len) {
        return a->len - b->len;
    }
    for (i = a->len - 1; i >= 0; i = i - 1) {
        if (a->digits[i] != b->digits[i]) {
            if (a->digits[i] > b->digits[i]) return 1;
            return -1;
        }
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C677: Big integer addition should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C677: empty output");
    assert!(
        code.contains("fn bigint_add"),
        "C677: Should contain bigint_add function"
    );
    Ok(())
}

#[test]
fn c678_big_integer_multiplication() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

typedef struct {
    uint32_t digits[64];
    int len;
} bigint_t;

void bigint_mul(const bigint_t *a, const bigint_t *b, bigint_t *result) {
    int i, j;
    for (i = 0; i < 64; i = i + 1) {
        result->digits[i] = 0;
    }
    for (i = 0; i < a->len; i = i + 1) {
        uint64_t carry = 0;
        for (j = 0; j < b->len; j = j + 1) {
            uint64_t prod = (uint64_t)a->digits[i] * (uint64_t)b->digits[j];
            prod = prod + result->digits[i + j] + carry;
            result->digits[i + j] = (uint32_t)(prod & 0xFFFFFFFF);
            carry = prod >> 32;
        }
        if (carry) {
            result->digits[i + b->len] = (uint32_t)carry;
        }
    }
    result->len = a->len + b->len;
    while (result->len > 1 && result->digits[result->len - 1] == 0) {
        result->len = result->len - 1;
    }
}

void bigint_from_uint(bigint_t *a, uint32_t val) {
    int i;
    for (i = 0; i < 64; i = i + 1) {
        a->digits[i] = 0;
    }
    a->digits[0] = val;
    a->len = 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C678: Big integer multiplication should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C678: empty output");
    assert!(
        code.contains("fn bigint_mul"),
        "C678: Should contain bigint_mul function"
    );
    Ok(())
}

#[test]
fn c679_modular_exponentiation() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long long uint64_t;

uint64_t mod_mul(uint64_t a, uint64_t b, uint64_t mod) {
    uint64_t result = 0;
    a = a % mod;
    while (b > 0) {
        if (b & 1) {
            result = (result + a) % mod;
        }
        a = (a * 2) % mod;
        b = b >> 1;
    }
    return result;
}

uint64_t mod_pow(uint64_t base, uint64_t exp, uint64_t mod) {
    uint64_t result = 1;
    base = base % mod;
    while (exp > 0) {
        if (exp & 1) {
            result = mod_mul(result, base, mod);
        }
        exp = exp >> 1;
        base = mod_mul(base, base, mod);
    }
    return result;
}

int is_probable_prime(uint64_t n, int rounds) {
    uint64_t d;
    int r, i;
    if (n < 2) return 0;
    if (n == 2 || n == 3) return 1;
    if (n % 2 == 0) return 0;
    d = n - 1;
    r = 0;
    while (d % 2 == 0) {
        d = d / 2;
        r = r + 1;
    }
    for (i = 0; i < rounds; i = i + 1) {
        uint64_t a = 2 + (i * 1234567) % (n - 3);
        uint64_t x = mod_pow(a, d, n);
        int j;
        if (x == 1 || x == n - 1) continue;
        for (j = 0; j < r - 1; j = j + 1) {
            x = mod_mul(x, x, n);
            if (x == n - 1) break;
        }
        if (x != n - 1) return 0;
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C679: Modular exponentiation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C679: empty output");
    assert!(
        code.contains("fn mod_pow"),
        "C679: Should contain mod_pow function"
    );
    Ok(())
}

#[test]
fn c680_prime_sieve_eratosthenes() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned char uint8_t;

void sieve_eratosthenes(uint8_t *is_prime, int n) {
    int i, j;
    for (i = 0; i < n; i = i + 1) {
        is_prime[i] = 1;
    }
    is_prime[0] = 0;
    if (n > 1) {
        is_prime[1] = 0;
    }
    for (i = 2; i * i < n; i = i + 1) {
        if (is_prime[i]) {
            for (j = i * i; j < n; j = j + i) {
                is_prime[j] = 0;
            }
        }
    }
}

int count_primes(const uint8_t *is_prime, int n) {
    int count = 0;
    int i;
    for (i = 0; i < n; i = i + 1) {
        if (is_prime[i]) {
            count = count + 1;
        }
    }
    return count;
}

int nth_prime(const uint8_t *is_prime, int n, int k) {
    int count = 0;
    int i;
    for (i = 2; i < n; i = i + 1) {
        if (is_prime[i]) {
            count = count + 1;
            if (count == k) {
                return i;
            }
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C680: Prime sieve should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C680: empty output");
    assert!(
        code.contains("fn sieve_eratosthenes"),
        "C680: Should contain sieve_eratosthenes function"
    );
    Ok(())
}

// ============================================================================
// C681-C685: Classical Numerical Algorithms
// ============================================================================

#[test]
fn c681_extended_euclidean_gcd_lcm() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef long long int64_t;

int64_t gcd(int64_t a, int64_t b) {
    while (b != 0) {
        int64_t t = b;
        b = a % b;
        a = t;
    }
    return a;
}

int64_t lcm(int64_t a, int64_t b) {
    int64_t g = gcd(a, b);
    if (g == 0) return 0;
    return (a / g) * b;
}

int64_t ext_gcd(int64_t a, int64_t b, int64_t *x, int64_t *y) {
    if (a == 0) {
        *x = 0;
        *y = 1;
        return b;
    }
    int64_t x1, y1;
    int64_t g = ext_gcd(b % a, a, &x1, &y1);
    *x = y1 - (b / a) * x1;
    *y = x1;
    return g;
}

int64_t mod_inverse(int64_t a, int64_t m) {
    int64_t x, y;
    int64_t g = ext_gcd(a, m, &x, &y);
    if (g != 1) return -1;
    return (x % m + m) % m;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C681: Extended Euclidean GCD/LCM should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C681: empty output");
    assert!(
        code.contains("fn gcd") || code.contains("fn ext_gcd"),
        "C681: Should contain gcd or ext_gcd function"
    );
    Ok(())
}

#[test]
fn c682_polynomial_evaluation_horner() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
double horner_eval(const double *coeffs, int degree, double x) {
    double result = coeffs[degree];
    int i;
    for (i = degree - 1; i >= 0; i = i - 1) {
        result = result * x + coeffs[i];
    }
    return result;
}

void poly_add(const double *a, int deg_a, const double *b, int deg_b,
              double *result, int *deg_result) {
    int max_deg = deg_a;
    int i;
    if (deg_b > max_deg) max_deg = deg_b;
    for (i = 0; i <= max_deg; i = i + 1) {
        result[i] = 0.0;
        if (i <= deg_a) result[i] = result[i] + a[i];
        if (i <= deg_b) result[i] = result[i] + b[i];
    }
    *deg_result = max_deg;
}

void poly_derivative(const double *coeffs, int degree,
                     double *result, int *deg_result) {
    int i;
    if (degree == 0) {
        result[0] = 0.0;
        *deg_result = 0;
        return;
    }
    for (i = 1; i <= degree; i = i + 1) {
        result[i - 1] = coeffs[i] * i;
    }
    *deg_result = degree - 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C682: Polynomial Horner evaluation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C682: empty output");
    assert!(
        code.contains("fn horner_eval"),
        "C682: Should contain horner_eval function"
    );
    Ok(())
}

#[test]
fn c683_newton_raphson_root_finding() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
double newton_sqrt(double val, double tol, int max_iter) {
    double x = val;
    int i;
    if (val <= 0.0) return 0.0;
    for (i = 0; i < max_iter; i = i + 1) {
        double fx = x * x - val;
        double fpx = 2.0 * x;
        if (fpx == 0.0) break;
        double x_new = x - fx / fpx;
        double diff = x_new - x;
        if (diff < 0) diff = -diff;
        if (diff < tol) return x_new;
        x = x_new;
    }
    return x;
}

double newton_cbrt(double val, double tol, int max_iter) {
    double x = val;
    int i;
    if (val == 0.0) return 0.0;
    for (i = 0; i < max_iter; i = i + 1) {
        double fx = x * x * x - val;
        double fpx = 3.0 * x * x;
        if (fpx == 0.0) break;
        double x_new = x - fx / fpx;
        double diff = x_new - x;
        if (diff < 0) diff = -diff;
        if (diff < tol) return x_new;
        x = x_new;
    }
    return x;
}

double newton_inv_sqrt(double val, double tol, int max_iter) {
    double x = 1.0;
    int i;
    if (val <= 0.0) return 0.0;
    for (i = 0; i < max_iter; i = i + 1) {
        double x_new = x * (1.5 - 0.5 * val * x * x);
        double diff = x_new - x;
        if (diff < 0) diff = -diff;
        if (diff < tol) return x_new;
        x = x_new;
    }
    return x;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C683: Newton-Raphson root finding should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C683: empty output");
    assert!(
        code.contains("fn newton_sqrt"),
        "C683: Should contain newton_sqrt function"
    );
    Ok(())
}

#[test]
fn c684_bisection_method() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    double root;
    int iterations;
    int converged;
} bisect_result_t;

double test_func_poly(double x) {
    return x * x * x - 2.0 * x - 5.0;
}

double test_func_trig(double x) {
    return x * x - 2.0;
}

bisect_result_t bisect(double a, double b, double tol, int max_iter) {
    bisect_result_t res;
    double fa = test_func_poly(a);
    double mid;
    int i;
    res.converged = 0;
    res.iterations = 0;
    for (i = 0; i < max_iter; i = i + 1) {
        mid = (a + b) / 2.0;
        double fm = test_func_poly(mid);
        res.iterations = i + 1;
        double width = b - a;
        if (width < 0) width = -width;
        if (width < tol) {
            res.root = mid;
            res.converged = 1;
            return res;
        }
        if (fa * fm < 0.0) {
            b = mid;
        } else {
            a = mid;
            fa = fm;
        }
    }
    res.root = (a + b) / 2.0;
    return res;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C684: Bisection method should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C684: empty output");
    assert!(
        code.contains("fn bisect"),
        "C684: Should contain bisect function"
    );
    Ok(())
}

#[test]
fn c685_simpson_rule_integration() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
double simpson_rule(double a, double b, int n) {
    double h = (b - a) / n;
    double sum = 0.0;
    int i;

    sum = a * a + b * b;

    for (i = 1; i < n; i = i + 1) {
        double x = a + i * h;
        double fx = x * x;
        if (i % 2 == 0) {
            sum = sum + 2.0 * fx;
        } else {
            sum = sum + 4.0 * fx;
        }
    }

    return sum * h / 3.0;
}

double trapezoid_rule(double a, double b, int n) {
    double h = (b - a) / n;
    double sum = 0.0;
    int i;

    sum = (a * a + b * b) / 2.0;

    for (i = 1; i < n; i = i + 1) {
        double x = a + i * h;
        sum = sum + x * x;
    }

    return sum * h;
}

double midpoint_rule(double a, double b, int n) {
    double h = (b - a) / n;
    double sum = 0.0;
    int i;
    for (i = 0; i < n; i = i + 1) {
        double x = a + (i + 0.5) * h;
        sum = sum + x * x;
    }
    return sum * h;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C685: Simpson's rule integration should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C685: empty output");
    assert!(
        code.contains("fn simpson_rule"),
        "C685: Should contain simpson_rule function"
    );
    Ok(())
}

// ============================================================================
// C686-C690: Statistical and Noise Algorithms
// ============================================================================

#[test]
fn c686_numerical_differentiation() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
double forward_diff(double x, double h) {
    double fx = x * x;
    double fxh = (x + h) * (x + h);
    return (fxh - fx) / h;
}

double central_diff(double x, double h) {
    double fxph = (x + h) * (x + h);
    double fxmh = (x - h) * (x - h);
    return (fxph - fxmh) / (2.0 * h);
}

double second_derivative(double x, double h) {
    double fxph = (x + h) * (x + h);
    double fx = x * x;
    double fxmh = (x - h) * (x - h);
    return (fxph - 2.0 * fx + fxmh) / (h * h);
}

double richardson_extrap(double x, double h) {
    double d1 = central_diff(x, h);
    double d2 = central_diff(x, h / 2.0);
    return (4.0 * d2 - d1) / 3.0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C686: Numerical differentiation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C686: empty output");
    assert!(
        code.contains("fn central_diff"),
        "C686: Should contain central_diff function"
    );
    Ok(())
}

#[test]
fn c687_linear_regression_least_squares() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    double slope;
    double intercept;
    double r_squared;
} regression_t;

regression_t linear_regression(const double *x, const double *y, int n) {
    regression_t result;
    double sum_x = 0.0, sum_y = 0.0;
    double sum_xy = 0.0, sum_x2 = 0.0, sum_y2 = 0.0;
    int i;

    for (i = 0; i < n; i = i + 1) {
        sum_x = sum_x + x[i];
        sum_y = sum_y + y[i];
        sum_xy = sum_xy + x[i] * y[i];
        sum_x2 = sum_x2 + x[i] * x[i];
        sum_y2 = sum_y2 + y[i] * y[i];
    }

    double denom = n * sum_x2 - sum_x * sum_x;
    if (denom == 0.0) {
        result.slope = 0.0;
        result.intercept = sum_y / n;
        result.r_squared = 0.0;
        return result;
    }

    result.slope = (n * sum_xy - sum_x * sum_y) / denom;
    result.intercept = (sum_y - result.slope * sum_x) / n;

    double ss_tot = sum_y2 - (sum_y * sum_y) / n;
    double ss_res = 0.0;
    for (i = 0; i < n; i = i + 1) {
        double pred = result.slope * x[i] + result.intercept;
        double diff = y[i] - pred;
        ss_res = ss_res + diff * diff;
    }

    if (ss_tot > 0.0) {
        result.r_squared = 1.0 - ss_res / ss_tot;
    } else {
        result.r_squared = 0.0;
    }

    return result;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C687: Linear regression should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C687: empty output");
    assert!(
        code.contains("fn linear_regression"),
        "C687: Should contain linear_regression function"
    );
    Ok(())
}

#[test]
fn c688_exponential_moving_average() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    double alpha;
    double value;
    int initialized;
} ema_t;

void ema_init(ema_t *ema, double alpha) {
    ema->alpha = alpha;
    ema->value = 0.0;
    ema->initialized = 0;
}

double ema_update(ema_t *ema, double sample) {
    if (!ema->initialized) {
        ema->value = sample;
        ema->initialized = 1;
    } else {
        ema->value = ema->alpha * sample + (1.0 - ema->alpha) * ema->value;
    }
    return ema->value;
}

void ema_batch(ema_t *ema, const double *samples, double *output, int n) {
    int i;
    for (i = 0; i < n; i = i + 1) {
        output[i] = ema_update(ema, samples[i]);
    }
}

double simple_moving_avg(const double *data, int n, int window) {
    double sum = 0.0;
    int i;
    int start = n - window;
    if (start < 0) start = 0;
    for (i = start; i < n; i = i + 1) {
        sum = sum + data[i];
    }
    return sum / (n - start);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C688: Exponential moving average should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C688: empty output");
    assert!(
        code.contains("fn ema_update"),
        "C688: Should contain ema_update function"
    );
    Ok(())
}

#[test]
fn c689_perlin_noise_2d() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
static int perm[256];

void perlin_init(int seed) {
    int i, j, tmp;
    for (i = 0; i < 256; i = i + 1) {
        perm[i] = i;
    }
    for (i = 255; i > 0; i = i - 1) {
        j = (seed + i * 7 + 13) % (i + 1);
        tmp = perm[i];
        perm[i] = perm[j];
        perm[j] = tmp;
    }
}

double fade(double t) {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

double lerp_noise(double t, double a, double b) {
    return a + t * (b - a);
}

double grad2d(int hash, double x, double y) {
    int h = hash & 3;
    double u, v;
    if (h == 0) { u = x; v = y; }
    else if (h == 1) { u = -x; v = y; }
    else if (h == 2) { u = x; v = -y; }
    else { u = -x; v = -y; }
    return u + v;
}

double perlin2d(double x, double y) {
    int xi = (int)x & 255;
    int yi = (int)y & 255;
    double xf = x - (int)x;
    double yf = y - (int)y;
    double u = fade(xf);
    double v = fade(yf);

    int aa = perm[(perm[xi] + yi) & 255];
    int ab = perm[(perm[xi] + yi + 1) & 255];
    int ba = perm[(perm[(xi + 1) & 255] + yi) & 255];
    int bb = perm[(perm[(xi + 1) & 255] + yi + 1) & 255];

    double x1 = lerp_noise(u, grad2d(aa, xf, yf), grad2d(ba, xf - 1.0, yf));
    double x2 = lerp_noise(u, grad2d(ab, xf, yf - 1.0), grad2d(bb, xf - 1.0, yf - 1.0));

    return lerp_noise(v, x1, x2);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C689: Perlin noise 2D should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C689: empty output");
    assert!(
        code.contains("fn perlin2d"),
        "C689: Should contain perlin2d function"
    );
    Ok(())
}

#[test]
fn c690_simplex_noise() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
static double grad3[12][3] = {
    {1,1,0},{-1,1,0},{1,-1,0},{-1,-1,0},
    {1,0,1},{-1,0,1},{1,0,-1},{-1,0,-1},
    {0,1,1},{0,-1,1},{0,1,-1},{0,-1,-1}
};

static int simplex_perm[256];

void simplex_init(int seed) {
    int i, j, tmp;
    for (i = 0; i < 256; i = i + 1) {
        simplex_perm[i] = i;
    }
    for (i = 255; i > 0; i = i - 1) {
        j = (seed + i * 11 + 7) % (i + 1);
        tmp = simplex_perm[i];
        simplex_perm[i] = simplex_perm[j];
        simplex_perm[j] = tmp;
    }
}

double dot2(const double *g, double x, double y) {
    return g[0] * x + g[1] * y;
}

double simplex_noise2d(double xin, double yin) {
    double n0, n1, n2;
    double s = (xin + yin) * 0.3660254037844386;
    int i = (int)(xin + s);
    int j = (int)(yin + s);
    double t = (i + j) * 0.21132486540518713;
    double x0 = xin - (i - t);
    double y0 = yin - (j - t);

    int i1, j1;
    if (x0 > y0) { i1 = 1; j1 = 0; }
    else { i1 = 0; j1 = 1; }

    double x1 = x0 - i1 + 0.21132486540518713;
    double y1 = y0 - j1 + 0.21132486540518713;
    double x2 = x0 - 1.0 + 2.0 * 0.21132486540518713;
    double y2 = y0 - 1.0 + 2.0 * 0.21132486540518713;

    int ii = i & 255;
    int jj = j & 255;
    int gi0 = simplex_perm[(ii + simplex_perm[jj]) & 255] % 12;
    int gi1 = simplex_perm[(ii + i1 + simplex_perm[(jj + j1) & 255]) & 255] % 12;
    int gi2 = simplex_perm[(ii + 1 + simplex_perm[(jj + 1) & 255]) & 255] % 12;

    double t0 = 0.5 - x0 * x0 - y0 * y0;
    if (t0 < 0) n0 = 0.0;
    else { t0 = t0 * t0; n0 = t0 * t0 * dot2(grad3[gi0], x0, y0); }

    double t1 = 0.5 - x1 * x1 - y1 * y1;
    if (t1 < 0) n1 = 0.0;
    else { t1 = t1 * t1; n1 = t1 * t1 * dot2(grad3[gi1], x1, y1); }

    double t2 = 0.5 - x2 * x2 - y2 * y2;
    if (t2 < 0) n2 = 0.0;
    else { t2 = t2 * t2; n2 = t2 * t2 * dot2(grad3[gi2], x2, y2); }

    return 70.0 * (n0 + n1 + n2);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C690: Simplex noise should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C690: empty output");
    assert!(
        code.contains("fn simplex_noise2d"),
        "C690: Should contain simplex_noise2d function"
    );
    Ok(())
}

// ============================================================================
// C691-C695: Curves, Quaternions, and Automatic Differentiation
// ============================================================================

#[test]
fn c691_bezier_curve_evaluation() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    double x;
    double y;
} point_t;

point_t bezier_linear(point_t p0, point_t p1, double t) {
    point_t r;
    r.x = (1.0 - t) * p0.x + t * p1.x;
    r.y = (1.0 - t) * p0.y + t * p1.y;
    return r;
}

point_t bezier_quadratic(point_t p0, point_t p1, point_t p2, double t) {
    double u = 1.0 - t;
    point_t r;
    r.x = u * u * p0.x + 2.0 * u * t * p1.x + t * t * p2.x;
    r.y = u * u * p0.y + 2.0 * u * t * p1.y + t * t * p2.y;
    return r;
}

point_t bezier_cubic(point_t p0, point_t p1, point_t p2, point_t p3, double t) {
    double u = 1.0 - t;
    double uu = u * u;
    double uuu = uu * u;
    double tt = t * t;
    double ttt = tt * t;
    point_t r;
    r.x = uuu * p0.x + 3.0 * uu * t * p1.x + 3.0 * u * tt * p2.x + ttt * p3.x;
    r.y = uuu * p0.y + 3.0 * uu * t * p1.y + 3.0 * u * tt * p2.y + ttt * p3.y;
    return r;
}

double bezier_arc_length(point_t p0, point_t p1, point_t p2, point_t p3, int segments) {
    double length = 0.0;
    point_t prev = p0;
    int i;
    for (i = 1; i <= segments; i = i + 1) {
        double t = (double)i / segments;
        point_t cur = bezier_cubic(p0, p1, p2, p3, t);
        double dx = cur.x - prev.x;
        double dy = cur.y - prev.y;
        length = length + dx * dx + dy * dy;
        prev = cur;
    }
    return length;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C691: Bezier curve evaluation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C691: empty output");
    assert!(
        code.contains("fn bezier_cubic"),
        "C691: Should contain bezier_cubic function"
    );
    Ok(())
}

#[test]
fn c692_bspline_evaluation() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    double x;
    double y;
} vec2_t;

double bspline_basis(int i, int k, double t, const double *knots) {
    double d1, d2, n1, n2;
    if (k == 1) {
        if (t >= knots[i] && t < knots[i + 1]) return 1.0;
        return 0.0;
    }
    d1 = knots[i + k - 1] - knots[i];
    d2 = knots[i + k] - knots[i + 1];
    n1 = 0.0;
    n2 = 0.0;
    if (d1 > 0.0) {
        n1 = (t - knots[i]) / d1 * bspline_basis(i, k - 1, t, knots);
    }
    if (d2 > 0.0) {
        n2 = (knots[i + k] - t) / d2 * bspline_basis(i + 1, k - 1, t, knots);
    }
    return n1 + n2;
}

vec2_t bspline_eval(const vec2_t *points, int n, const double *knots, int order, double t) {
    vec2_t result;
    int i;
    result.x = 0.0;
    result.y = 0.0;
    for (i = 0; i < n; i = i + 1) {
        double b = bspline_basis(i, order, t, knots);
        result.x = result.x + b * points[i].x;
        result.y = result.y + b * points[i].y;
    }
    return result;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C692: B-spline evaluation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C692: empty output");
    assert!(
        code.contains("fn bspline_eval"),
        "C692: Should contain bspline_eval function"
    );
    Ok(())
}

#[test]
fn c693_catmull_rom_spline() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    double x;
    double y;
} point2d_t;

point2d_t catmull_rom(point2d_t p0, point2d_t p1, point2d_t p2, point2d_t p3, double t) {
    double t2 = t * t;
    double t3 = t2 * t;
    point2d_t r;

    r.x = 0.5 * ((2.0 * p1.x) +
                  (-p0.x + p2.x) * t +
                  (2.0 * p0.x - 5.0 * p1.x + 4.0 * p2.x - p3.x) * t2 +
                  (-p0.x + 3.0 * p1.x - 3.0 * p2.x + p3.x) * t3);

    r.y = 0.5 * ((2.0 * p1.y) +
                  (-p0.y + p2.y) * t +
                  (2.0 * p0.y - 5.0 * p1.y + 4.0 * p2.y - p3.y) * t2 +
                  (-p0.y + 3.0 * p1.y - 3.0 * p2.y + p3.y) * t3);

    return r;
}

point2d_t catmull_rom_tangent(point2d_t p0, point2d_t p1, point2d_t p2, point2d_t p3, double t) {
    double t2 = t * t;
    point2d_t r;

    r.x = 0.5 * ((-p0.x + p2.x) +
                  (4.0 * p0.x - 10.0 * p1.x + 8.0 * p2.x - 2.0 * p3.x) * t +
                  (-3.0 * p0.x + 9.0 * p1.x - 9.0 * p2.x + 3.0 * p3.x) * t2);

    r.y = 0.5 * ((-p0.y + p2.y) +
                  (4.0 * p0.y - 10.0 * p1.y + 8.0 * p2.y - 2.0 * p3.y) * t +
                  (-3.0 * p0.y + 9.0 * p1.y - 9.0 * p2.y + 3.0 * p3.y) * t2);

    return r;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C693: Catmull-Rom spline should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C693: empty output");
    assert!(
        code.contains("fn catmull_rom"),
        "C693: Should contain catmull_rom function"
    );
    Ok(())
}

#[test]
fn c694_quaternion_operations() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    double w;
    double x;
    double y;
    double z;
} quat_t;

quat_t quat_mul(quat_t a, quat_t b) {
    quat_t r;
    r.w = a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z;
    r.x = a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y;
    r.y = a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x;
    r.z = a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w;
    return r;
}

quat_t quat_conjugate(quat_t q) {
    quat_t r;
    r.w = q.w;
    r.x = -q.x;
    r.y = -q.y;
    r.z = -q.z;
    return r;
}

double quat_norm(quat_t q) {
    return q.w * q.w + q.x * q.x + q.y * q.y + q.z * q.z;
}

quat_t quat_normalize(quat_t q) {
    double n = quat_norm(q);
    quat_t r;
    if (n < 0.000001) {
        r.w = 1.0; r.x = 0.0; r.y = 0.0; r.z = 0.0;
        return r;
    }
    double inv = 1.0 / n;
    r.w = q.w * inv;
    r.x = q.x * inv;
    r.y = q.y * inv;
    r.z = q.z * inv;
    return r;
}

quat_t quat_slerp(quat_t a, quat_t b, double t) {
    double dot = a.w * b.w + a.x * b.x + a.y * b.y + a.z * b.z;
    quat_t r;
    double s0, s1;

    if (dot < 0.0) {
        b.w = -b.w; b.x = -b.x; b.y = -b.y; b.z = -b.z;
        dot = -dot;
    }

    if (dot > 0.9995) {
        s0 = 1.0 - t;
        s1 = t;
    } else {
        double theta = dot;
        if (theta > 1.0) theta = 1.0;
        if (theta < -1.0) theta = -1.0;
        double sin_t = 1.0 - theta * theta;
        if (sin_t < 0.0) sin_t = 0.0;
        s0 = (1.0 - t);
        s1 = t;
    }

    r.w = s0 * a.w + s1 * b.w;
    r.x = s0 * a.x + s1 * b.x;
    r.y = s0 * a.y + s1 * b.y;
    r.z = s0 * a.z + s1 * b.z;
    return r;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C694: Quaternion operations should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C694: empty output");
    assert!(
        code.contains("fn quat_mul"),
        "C694: Should contain quat_mul function"
    );
    Ok(())
}

#[test]
fn c695_dual_number_automatic_differentiation() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    double real;
    double dual;
} dual_t;

dual_t dual_const(double val) {
    dual_t d;
    d.real = val;
    d.dual = 0.0;
    return d;
}

dual_t dual_var(double val) {
    dual_t d;
    d.real = val;
    d.dual = 1.0;
    return d;
}

dual_t dual_add(dual_t a, dual_t b) {
    dual_t r;
    r.real = a.real + b.real;
    r.dual = a.dual + b.dual;
    return r;
}

dual_t dual_mul(dual_t a, dual_t b) {
    dual_t r;
    r.real = a.real * b.real;
    r.dual = a.real * b.dual + a.dual * b.real;
    return r;
}

dual_t dual_div(dual_t a, dual_t b) {
    dual_t r;
    r.real = a.real / b.real;
    r.dual = (a.dual * b.real - a.real * b.dual) / (b.real * b.real);
    return r;
}

dual_t dual_pow_int(dual_t base, int exp) {
    dual_t result = dual_const(1.0);
    int i;
    for (i = 0; i < exp; i = i + 1) {
        result = dual_mul(result, base);
    }
    return result;
}

double auto_diff(double x) {
    dual_t dx = dual_var(x);
    dual_t three = dual_const(3.0);
    dual_t two = dual_const(2.0);
    dual_t x2 = dual_mul(dx, dx);
    dual_t x3 = dual_mul(x2, dx);
    dual_t term1 = dual_mul(three, x2);
    dual_t term2 = dual_mul(two, dx);
    dual_t result = dual_add(dual_add(x3, term1), term2);
    return result.dual;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C695: Dual number automatic differentiation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C695: empty output");
    assert!(
        code.contains("fn dual_mul"),
        "C695: Should contain dual_mul function"
    );
    Ok(())
}

// ============================================================================
// C696-C700: Number Systems and Transforms
// ============================================================================

#[test]
fn c696_complex_number_arithmetic() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    double re;
    double im;
} complex_t;

complex_t complex_add(complex_t a, complex_t b) {
    complex_t r;
    r.re = a.re + b.re;
    r.im = a.im + b.im;
    return r;
}

complex_t complex_sub(complex_t a, complex_t b) {
    complex_t r;
    r.re = a.re - b.re;
    r.im = a.im - b.im;
    return r;
}

complex_t complex_mul(complex_t a, complex_t b) {
    complex_t r;
    r.re = a.re * b.re - a.im * b.im;
    r.im = a.re * b.im + a.im * b.re;
    return r;
}

complex_t complex_div(complex_t a, complex_t b) {
    complex_t r;
    double denom = b.re * b.re + b.im * b.im;
    r.re = (a.re * b.re + a.im * b.im) / denom;
    r.im = (a.im * b.re - a.re * b.im) / denom;
    return r;
}

double complex_abs(complex_t z) {
    return z.re * z.re + z.im * z.im;
}

complex_t complex_conj(complex_t z) {
    complex_t r;
    r.re = z.re;
    r.im = -z.im;
    return r;
}

complex_t complex_pow_int(complex_t z, int n) {
    complex_t result;
    int i;
    result.re = 1.0;
    result.im = 0.0;
    for (i = 0; i < n; i = i + 1) {
        result = complex_mul(result, z);
    }
    return result;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C696: Complex number arithmetic should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C696: empty output");
    assert!(
        code.contains("fn complex_mul"),
        "C696: Should contain complex_mul function"
    );
    Ok(())
}

#[test]
fn c697_interval_arithmetic() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    double lo;
    double hi;
} interval_t;

interval_t interval_new(double lo, double hi) {
    interval_t r;
    r.lo = lo;
    r.hi = hi;
    return r;
}

interval_t interval_add(interval_t a, interval_t b) {
    interval_t r;
    r.lo = a.lo + b.lo;
    r.hi = a.hi + b.hi;
    return r;
}

interval_t interval_sub(interval_t a, interval_t b) {
    interval_t r;
    r.lo = a.lo - b.hi;
    r.hi = a.hi - b.lo;
    return r;
}

interval_t interval_mul(interval_t a, interval_t b) {
    interval_t r;
    double p1 = a.lo * b.lo;
    double p2 = a.lo * b.hi;
    double p3 = a.hi * b.lo;
    double p4 = a.hi * b.hi;
    r.lo = p1;
    if (p2 < r.lo) r.lo = p2;
    if (p3 < r.lo) r.lo = p3;
    if (p4 < r.lo) r.lo = p4;
    r.hi = p1;
    if (p2 > r.hi) r.hi = p2;
    if (p3 > r.hi) r.hi = p3;
    if (p4 > r.hi) r.hi = p4;
    return r;
}

double interval_width(interval_t a) {
    return a.hi - a.lo;
}

double interval_midpoint(interval_t a) {
    return (a.lo + a.hi) / 2.0;
}

int interval_contains(interval_t a, double x) {
    return x >= a.lo && x <= a.hi;
}

int interval_overlaps(interval_t a, interval_t b) {
    return a.lo <= b.hi && b.lo <= a.hi;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C697: Interval arithmetic should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C697: empty output");
    assert!(
        code.contains("fn interval_mul"),
        "C697: Should contain interval_mul function"
    );
    Ok(())
}

#[test]
fn c698_rational_number_arithmetic() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef long long int64_t;

typedef struct {
    int64_t num;
    int64_t den;
} rational_t;

int64_t rat_gcd(int64_t a, int64_t b) {
    if (a < 0) a = -a;
    if (b < 0) b = -b;
    while (b != 0) {
        int64_t t = b;
        b = a % b;
        a = t;
    }
    return a;
}

rational_t rat_normalize(rational_t r) {
    rational_t result;
    int64_t g;
    if (r.den < 0) {
        r.num = -r.num;
        r.den = -r.den;
    }
    g = rat_gcd(r.num, r.den);
    if (g == 0) g = 1;
    result.num = r.num / g;
    result.den = r.den / g;
    return result;
}

rational_t rat_add(rational_t a, rational_t b) {
    rational_t r;
    r.num = a.num * b.den + b.num * a.den;
    r.den = a.den * b.den;
    return rat_normalize(r);
}

rational_t rat_sub(rational_t a, rational_t b) {
    rational_t r;
    r.num = a.num * b.den - b.num * a.den;
    r.den = a.den * b.den;
    return rat_normalize(r);
}

rational_t rat_mul(rational_t a, rational_t b) {
    rational_t r;
    r.num = a.num * b.num;
    r.den = a.den * b.den;
    return rat_normalize(r);
}

rational_t rat_div(rational_t a, rational_t b) {
    rational_t r;
    r.num = a.num * b.den;
    r.den = a.den * b.num;
    return rat_normalize(r);
}

int rat_compare(rational_t a, rational_t b) {
    int64_t lhs = a.num * b.den;
    int64_t rhs = b.num * a.den;
    if (lhs < rhs) return -1;
    if (lhs > rhs) return 1;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C698: Rational number arithmetic should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C698: empty output");
    assert!(
        code.contains("fn rat_mul") || code.contains("fn rat_add"),
        "C698: Should contain rational arithmetic functions"
    );
    Ok(())
}

#[test]
fn c699_montgomery_multiplication() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long long uint64_t;

typedef struct {
    uint64_t mod_val;
    uint64_t r;
    uint64_t r2;
    uint64_t mod_inv;
} mont_ctx_t;

uint64_t mont_reduce(const mont_ctx_t *ctx, uint64_t t) {
    uint64_t m = t * ctx->mod_inv;
    uint64_t u = t + m * ctx->mod_val;
    u = u >> 32;
    if (u >= ctx->mod_val) {
        u = u - ctx->mod_val;
    }
    return u;
}

uint64_t mont_mul(const mont_ctx_t *ctx, uint64_t a, uint64_t b) {
    uint64_t t = a * b;
    return mont_reduce(ctx, t);
}

uint64_t mont_to(const mont_ctx_t *ctx, uint64_t a) {
    return mont_mul(ctx, a, ctx->r2);
}

uint64_t mont_from(const mont_ctx_t *ctx, uint64_t a) {
    return mont_reduce(ctx, a);
}

uint64_t mont_pow(const mont_ctx_t *ctx, uint64_t base, uint64_t exp) {
    uint64_t result = mont_to(ctx, 1);
    uint64_t b = mont_to(ctx, base);
    while (exp > 0) {
        if (exp & 1) {
            result = mont_mul(ctx, result, b);
        }
        b = mont_mul(ctx, b, b);
        exp = exp >> 1;
    }
    return mont_from(ctx, result);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C699: Montgomery multiplication should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C699: empty output");
    assert!(
        code.contains("fn mont_mul"),
        "C699: Should contain mont_mul function"
    );
    Ok(())
}

#[test]
fn c700_number_theoretic_transform() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned long long uint64_t;

uint64_t ntt_mod_pow(uint64_t base, uint64_t exp, uint64_t mod) {
    uint64_t result = 1;
    base = base % mod;
    while (exp > 0) {
        if (exp & 1) {
            result = (result * base) % mod;
        }
        exp = exp >> 1;
        base = (base * base) % mod;
    }
    return result;
}

void ntt_forward(uint64_t *a, int n, uint64_t mod, uint64_t g) {
    int i, j, len;
    j = 0;
    for (i = 1; i < n; i = i + 1) {
        int bit = n >> 1;
        while (j & bit) {
            j = j ^ bit;
            bit = bit >> 1;
        }
        j = j ^ bit;
        if (i < j) {
            uint64_t tmp = a[i];
            a[i] = a[j];
            a[j] = tmp;
        }
    }

    for (len = 2; len <= n; len = len * 2) {
        uint64_t w = ntt_mod_pow(g, (mod - 1) / len, mod);
        for (i = 0; i < n; i = i + len) {
            uint64_t wn = 1;
            for (j = 0; j < len / 2; j = j + 1) {
                uint64_t u = a[i + j];
                uint64_t v = (a[i + j + len / 2] * wn) % mod;
                a[i + j] = (u + v) % mod;
                a[i + j + len / 2] = (u + mod - v) % mod;
                wn = (wn * w) % mod;
            }
        }
    }
}

void ntt_inverse(uint64_t *a, int n, uint64_t mod, uint64_t g) {
    uint64_t g_inv = ntt_mod_pow(g, mod - 2, mod);
    uint64_t n_inv = ntt_mod_pow(n, mod - 2, mod);
    int i;
    ntt_forward(a, n, mod, g_inv);
    for (i = 0; i < n; i = i + 1) {
        a[i] = (a[i] * n_inv) % mod;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C700: Number-theoretic transform should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C700: empty output");
    assert!(
        code.contains("fn ntt_forward"),
        "C700: Should contain ntt_forward function"
    );
    Ok(())
}
