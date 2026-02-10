//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C801-C825: Signal Processing domain -- the kind of C code found in
//! DSP libraries, software-defined radios, audio effects, communications
//! systems, and real-time signal analysis.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world signal processing patterns commonly
//! found in GNU Radio, liquid-dsp, CMSIS-DSP, SigPack, and similar
//! signal processing libraries -- all expressed as valid C99 with
//! Taylor-series approximations for math functions (no libm dependency).
//!
//! Organization:
//! - C801-C805: Core filters and transforms (FIR, IIR biquad, FFT, IFFT, windowing)
//! - C806-C810: Correlation and spectral (autocorrelation, cross-correlation, PSD, resonator, comb)
//! - C811-C815: Utility filters and detectors (all-pass, resampler, envelope, zero-crossing, peak)
//! - C816-C820: Smoothing and analysis (moving avg, median, Goertzel, PLL, noise gate)
//! - C821-C825: Audio processing (compressor, pitch detect, equalizer, polyphase resampler, Haar wavelet)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C801-C805: Core Filters and Transforms
// ============================================================================

/// C801: FIR filter -- direct form convolution with coefficient array
#[test]
fn c801_fir_filter_direct_convolution() {
    let c_code = r#"
typedef struct {
    float coeffs[32];
    float delay[32];
    int num_taps;
    int write_idx;
} sp_fir_t;

void sp_fir_init(sp_fir_t *f, const float *coeffs, int taps) {
    int i;
    f->num_taps = taps < 32 ? taps : 32;
    f->write_idx = 0;
    for (i = 0; i < f->num_taps; i++) {
        f->coeffs[i] = coeffs[i];
        f->delay[i] = 0.0f;
    }
}

float sp_fir_process(sp_fir_t *f, float input) {
    float output = 0.0f;
    int i, idx;
    f->delay[f->write_idx] = input;
    for (i = 0; i < f->num_taps; i++) {
        idx = f->write_idx - i;
        if (idx < 0) idx += f->num_taps;
        output += f->coeffs[i] * f->delay[idx];
    }
    f->write_idx++;
    if (f->write_idx >= f->num_taps) f->write_idx = 0;
    return output;
}

void sp_fir_reset(sp_fir_t *f) {
    int i;
    f->write_idx = 0;
    for (i = 0; i < f->num_taps; i++) {
        f->delay[i] = 0.0f;
    }
}

float sp_fir_group_delay(const sp_fir_t *f) {
    return (float)(f->num_taps - 1) / 2.0f;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C801: FIR filter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C801: Output should not be empty");
    assert!(
        code.contains("fn sp_fir_init"),
        "C801: Should contain sp_fir_init function"
    );
    assert!(
        code.contains("fn sp_fir_process"),
        "C801: Should contain sp_fir_process function"
    );
}

/// C802: IIR biquad filter -- second-order section with feedback
#[test]
fn c802_iir_biquad_filter() {
    let c_code = r#"
typedef struct {
    float b0, b1, b2;
    float a1, a2;
    float x1, x2;
    float y1, y2;
} sp_biquad_t;

void sp_biquad_init(sp_biquad_t *bq, float b0, float b1, float b2,
                     float a1, float a2) {
    bq->b0 = b0;
    bq->b1 = b1;
    bq->b2 = b2;
    bq->a1 = a1;
    bq->a2 = a2;
    bq->x1 = 0.0f;
    bq->x2 = 0.0f;
    bq->y1 = 0.0f;
    bq->y2 = 0.0f;
}

float sp_biquad_process(sp_biquad_t *bq, float x0) {
    float y0 = bq->b0 * x0 + bq->b1 * bq->x1 + bq->b2 * bq->x2
              - bq->a1 * bq->y1 - bq->a2 * bq->y2;
    bq->x2 = bq->x1;
    bq->x1 = x0;
    bq->y2 = bq->y1;
    bq->y1 = y0;
    return y0;
}

void sp_biquad_reset(sp_biquad_t *bq) {
    bq->x1 = 0.0f;
    bq->x2 = 0.0f;
    bq->y1 = 0.0f;
    bq->y2 = 0.0f;
}

void sp_biquad_set_lowpass(sp_biquad_t *bq, float freq, float q, float sr) {
    float w0 = 2.0f * 3.14159265f * freq / sr;
    float w0_sq = w0 * w0;
    float alpha_approx = w0 / (2.0f * q);
    float norm = 1.0f + alpha_approx;
    bq->b0 = (1.0f - (1.0f - w0_sq / 2.0f)) / (2.0f * norm);
    bq->b1 = 2.0f * bq->b0;
    bq->b2 = bq->b0;
    bq->a1 = -2.0f * (1.0f - w0_sq / 2.0f) / norm;
    bq->a2 = (1.0f - alpha_approx) / norm;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C802: IIR biquad filter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C802: Output should not be empty");
    assert!(
        code.contains("fn sp_biquad_process"),
        "C802: Should contain sp_biquad_process function"
    );
    assert!(
        code.contains("fn sp_biquad_set_lowpass"),
        "C802: Should contain sp_biquad_set_lowpass function"
    );
}

/// C803: FFT radix-2 Cooley-Tukey -- in-place butterfly with bit-reversal
#[test]
fn c803_fft_radix2_cooley_tukey() {
    let c_code = r#"
typedef struct {
    float real;
    float imag;
} sp_complex_t;

void sp_fft_bitreverse(sp_complex_t *buf, int n) {
    int i, j, k;
    j = 0;
    for (i = 0; i < n - 1; i++) {
        if (i < j) {
            sp_complex_t tmp = buf[i];
            buf[i] = buf[j];
            buf[j] = tmp;
        }
        k = n >> 1;
        while (k <= j) {
            j -= k;
            k >>= 1;
        }
        j += k;
    }
}

static float sp_taylor_sin(float x) {
    float x2 = x * x;
    float x3 = x2 * x;
    float x5 = x3 * x2;
    float x7 = x5 * x2;
    return x - x3 / 6.0f + x5 / 120.0f - x7 / 5040.0f;
}

static float sp_taylor_cos(float x) {
    float x2 = x * x;
    float x4 = x2 * x2;
    float x6 = x4 * x2;
    return 1.0f - x2 / 2.0f + x4 / 24.0f - x6 / 720.0f;
}

void sp_fft_compute(sp_complex_t *buf, int n) {
    int stage, bfly, pair;
    float pi = 3.14159265f;
    sp_fft_bitreverse(buf, n);
    for (stage = 1; stage < n; stage <<= 1) {
        float angle = -pi / (float)stage;
        sp_complex_t w_step;
        w_step.real = sp_taylor_cos(angle);
        w_step.imag = sp_taylor_sin(angle);
        for (bfly = 0; bfly < n; bfly += stage << 1) {
            sp_complex_t w;
            w.real = 1.0f;
            w.imag = 0.0f;
            for (pair = 0; pair < stage; pair++) {
                sp_complex_t t;
                int top = bfly + pair;
                int bot = top + stage;
                t.real = w.real * buf[bot].real - w.imag * buf[bot].imag;
                t.imag = w.real * buf[bot].imag + w.imag * buf[bot].real;
                buf[bot].real = buf[top].real - t.real;
                buf[bot].imag = buf[top].imag - t.imag;
                buf[top].real = buf[top].real + t.real;
                buf[top].imag = buf[top].imag + t.imag;
                float w_new_real = w.real * w_step.real - w.imag * w_step.imag;
                float w_new_imag = w.real * w_step.imag + w.imag * w_step.real;
                w.real = w_new_real;
                w.imag = w_new_imag;
            }
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C803: FFT radix-2 should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C803: Output should not be empty");
    assert!(
        code.contains("fn sp_fft_compute"),
        "C803: Should contain sp_fft_compute function"
    );
    assert!(
        code.contains("fn sp_fft_bitreverse"),
        "C803: Should contain sp_fft_bitreverse function"
    );
}

/// C804: Inverse FFT -- conjugate-multiply-conjugate approach
#[test]
fn c804_inverse_fft() {
    let c_code = r#"
typedef struct {
    float real;
    float imag;
} sp_cplx_t;

void sp_ifft_conjugate(sp_cplx_t *buf, int n) {
    int i;
    for (i = 0; i < n; i++) {
        buf[i].imag = -buf[i].imag;
    }
}

void sp_ifft_scale(sp_cplx_t *buf, int n) {
    int i;
    float inv_n = 1.0f / (float)n;
    for (i = 0; i < n; i++) {
        buf[i].real *= inv_n;
        buf[i].imag *= inv_n;
    }
}

void sp_ifft_bitreverse(sp_cplx_t *buf, int n) {
    int i, j, k;
    j = 0;
    for (i = 0; i < n - 1; i++) {
        if (i < j) {
            sp_cplx_t tmp = buf[i];
            buf[i] = buf[j];
            buf[j] = tmp;
        }
        k = n >> 1;
        while (k <= j) {
            j -= k;
            k >>= 1;
        }
        j += k;
    }
}

static float sp_ifft_cos(float x) {
    float x2 = x * x;
    float x4 = x2 * x2;
    return 1.0f - x2 / 2.0f + x4 / 24.0f;
}

static float sp_ifft_sin(float x) {
    float x3 = x * x * x;
    float x5 = x3 * x * x;
    return x - x3 / 6.0f + x5 / 120.0f;
}

void sp_ifft_forward(sp_cplx_t *buf, int n) {
    int stage, bfly, pair;
    float pi = 3.14159265f;
    sp_ifft_bitreverse(buf, n);
    for (stage = 1; stage < n; stage <<= 1) {
        float angle = -pi / (float)stage;
        float wr = sp_ifft_cos(angle);
        float wi = sp_ifft_sin(angle);
        for (bfly = 0; bfly < n; bfly += stage << 1) {
            float cr = 1.0f, ci = 0.0f;
            for (pair = 0; pair < stage; pair++) {
                int top = bfly + pair;
                int bot = top + stage;
                float tr = cr * buf[bot].real - ci * buf[bot].imag;
                float ti = cr * buf[bot].imag + ci * buf[bot].real;
                buf[bot].real = buf[top].real - tr;
                buf[bot].imag = buf[top].imag - ti;
                buf[top].real = buf[top].real + tr;
                buf[top].imag = buf[top].imag + ti;
                float new_cr = cr * wr - ci * wi;
                ci = cr * wi + ci * wr;
                cr = new_cr;
            }
        }
    }
}

void sp_ifft_compute(sp_cplx_t *buf, int n) {
    sp_ifft_conjugate(buf, n);
    sp_ifft_forward(buf, n);
    sp_ifft_conjugate(buf, n);
    sp_ifft_scale(buf, n);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C804: Inverse FFT should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C804: Output should not be empty");
    assert!(
        code.contains("fn sp_ifft_compute"),
        "C804: Should contain sp_ifft_compute function"
    );
    assert!(
        code.contains("fn sp_ifft_conjugate"),
        "C804: Should contain sp_ifft_conjugate function"
    );
}

/// C805: Windowing functions -- Hamming, Hanning, Blackman with Taylor approx
#[test]
fn c805_windowing_functions() {
    let c_code = r#"
static float sp_win_cos(float x) {
    float x2 = x * x;
    float x4 = x2 * x2;
    float x6 = x4 * x2;
    return 1.0f - x2 / 2.0f + x4 / 24.0f - x6 / 720.0f;
}

void sp_hamming_window(float *w, int n) {
    int i;
    float pi2 = 2.0f * 3.14159265f;
    for (i = 0; i < n; i++) {
        float phase = pi2 * (float)i / (float)(n - 1);
        w[i] = 0.54f - 0.46f * sp_win_cos(phase);
    }
}

void sp_hanning_window(float *w, int n) {
    int i;
    float pi2 = 2.0f * 3.14159265f;
    for (i = 0; i < n; i++) {
        float phase = pi2 * (float)i / (float)(n - 1);
        w[i] = 0.5f * (1.0f - sp_win_cos(phase));
    }
}

void sp_blackman_window(float *w, int n) {
    int i;
    float pi2 = 2.0f * 3.14159265f;
    float pi4 = 4.0f * 3.14159265f;
    for (i = 0; i < n; i++) {
        float phase1 = pi2 * (float)i / (float)(n - 1);
        float phase2 = pi4 * (float)i / (float)(n - 1);
        w[i] = 0.42f - 0.5f * sp_win_cos(phase1) + 0.08f * sp_win_cos(phase2);
    }
}

void sp_apply_window(float *signal, const float *window, int n) {
    int i;
    for (i = 0; i < n; i++) {
        signal[i] *= window[i];
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C805: Windowing functions should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C805: Output should not be empty");
    assert!(
        code.contains("fn sp_hamming_window"),
        "C805: Should contain sp_hamming_window"
    );
    assert!(
        code.contains("fn sp_hanning_window"),
        "C805: Should contain sp_hanning_window"
    );
    assert!(
        code.contains("fn sp_blackman_window"),
        "C805: Should contain sp_blackman_window"
    );
}

// ============================================================================
// C806-C810: Correlation and Spectral Analysis
// ============================================================================

/// C806: Autocorrelation -- used for pitch detection and spectral estimation
#[test]
fn c806_autocorrelation() {
    let c_code = r#"
void sp_autocorrelate(const float *x, int n, float *r, int max_lag) {
    int lag, i;
    for (lag = 0; lag < max_lag && lag < n; lag++) {
        r[lag] = 0.0f;
        for (i = 0; i < n - lag; i++) {
            r[lag] += x[i] * x[i + lag];
        }
    }
}

void sp_autocorrelate_normalized(const float *x, int n, float *r, int max_lag) {
    int lag, i;
    float energy = 0.0f;
    for (i = 0; i < n; i++) {
        energy += x[i] * x[i];
    }
    if (energy < 1e-10f) {
        for (lag = 0; lag < max_lag; lag++) r[lag] = 0.0f;
        return;
    }
    for (lag = 0; lag < max_lag && lag < n; lag++) {
        r[lag] = 0.0f;
        for (i = 0; i < n - lag; i++) {
            r[lag] += x[i] * x[i + lag];
        }
        r[lag] /= energy;
    }
}

int sp_find_first_peak(const float *r, int max_lag, int min_lag) {
    int i;
    for (i = min_lag; i < max_lag - 1; i++) {
        if (r[i] > r[i - 1] && r[i] > r[i + 1] && r[i] > 0.2f) {
            return i;
        }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C806: Autocorrelation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C806: Output should not be empty");
    assert!(
        code.contains("fn sp_autocorrelate"),
        "C806: Should contain sp_autocorrelate function"
    );
    assert!(
        code.contains("fn sp_find_first_peak"),
        "C806: Should contain sp_find_first_peak function"
    );
}

/// C807: Cross-correlation -- measures similarity between two signals
#[test]
fn c807_cross_correlation() {
    let c_code = r#"
void sp_xcorrelate(const float *x, int nx, const float *y, int ny,
                    float *result, int max_lag) {
    int lag, i;
    int min_len = nx < ny ? nx : ny;
    for (lag = 0; lag < max_lag; lag++) {
        result[lag] = 0.0f;
        for (i = 0; i < min_len - lag; i++) {
            result[lag] += x[i] * y[i + lag];
        }
    }
}

float sp_xcorrelate_energy(const float *x, int n) {
    float e = 0.0f;
    int i;
    for (i = 0; i < n; i++) {
        e += x[i] * x[i];
    }
    return e;
}

static float sp_xcorr_sqrt(float x) {
    float guess = x * 0.5f;
    int i;
    for (i = 0; i < 10; i++) {
        if (guess <= 0.0f) return 0.0f;
        guess = 0.5f * (guess + x / guess);
    }
    return guess;
}

float sp_xcorrelate_coeff(const float *x, const float *y, int n) {
    float sum_xy = 0.0f;
    float sum_xx = 0.0f;
    float sum_yy = 0.0f;
    int i;
    for (i = 0; i < n; i++) {
        sum_xy += x[i] * y[i];
        sum_xx += x[i] * x[i];
        sum_yy += y[i] * y[i];
    }
    float denom = sp_xcorr_sqrt(sum_xx * sum_yy);
    if (denom < 1e-10f) return 0.0f;
    return sum_xy / denom;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C807: Cross-correlation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C807: Output should not be empty");
    assert!(
        code.contains("fn sp_xcorrelate"),
        "C807: Should contain sp_xcorrelate function"
    );
    assert!(
        code.contains("fn sp_xcorrelate_coeff"),
        "C807: Should contain sp_xcorrelate_coeff function"
    );
}

/// C808: Power spectral density -- magnitude squared of FFT bins
#[test]
fn c808_power_spectral_density() {
    let c_code = r#"
typedef struct {
    float real;
    float imag;
} sp_psd_complex_t;

void sp_psd_compute(const sp_psd_complex_t *fft_bins, int n, float *psd) {
    int i;
    for (i = 0; i < n; i++) {
        psd[i] = fft_bins[i].real * fft_bins[i].real
                + fft_bins[i].imag * fft_bins[i].imag;
    }
}

void sp_psd_to_db(const float *psd, int n, float *db, float ref_power) {
    int i;
    for (i = 0; i < n; i++) {
        float ratio = psd[i] / (ref_power + 1e-20f);
        float log_approx = 0.0f;
        if (ratio > 0.0f) {
            float y = (ratio - 1.0f) / (ratio + 1.0f);
            float y2 = y * y;
            log_approx = 2.0f * y * (1.0f + y2 / 3.0f + y2 * y2 / 5.0f);
        }
        db[i] = 10.0f * log_approx / 2.302585f;
    }
}

float sp_psd_total_power(const float *psd, int n) {
    float total = 0.0f;
    int i;
    for (i = 0; i < n; i++) {
        total += psd[i];
    }
    return total / (float)n;
}

int sp_psd_peak_bin(const float *psd, int n) {
    int peak = 0;
    float max_val = psd[0];
    int i;
    for (i = 1; i < n; i++) {
        if (psd[i] > max_val) {
            max_val = psd[i];
            peak = i;
        }
    }
    return peak;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C808: Power spectral density should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C808: Output should not be empty");
    assert!(
        code.contains("fn sp_psd_compute"),
        "C808: Should contain sp_psd_compute function"
    );
    assert!(
        code.contains("fn sp_psd_to_db"),
        "C808: Should contain sp_psd_to_db function"
    );
}

/// C809: Digital resonator -- second-order oscillator with feedback
#[test]
fn c809_digital_resonator() {
    let c_code = r#"
static float sp_res_cos(float x) {
    float x2 = x * x;
    float x4 = x2 * x2;
    return 1.0f - x2 / 2.0f + x4 / 24.0f;
}

typedef struct {
    float freq;
    float radius;
    float y1, y2;
    float coeff;
} sp_resonator_t;

void sp_resonator_init(sp_resonator_t *r, float freq, float radius, float sr) {
    float w = 2.0f * 3.14159265f * freq / sr;
    r->freq = freq;
    r->radius = radius;
    r->coeff = 2.0f * radius * sp_res_cos(w);
    r->y1 = 0.0f;
    r->y2 = 0.0f;
}

float sp_resonator_process(sp_resonator_t *r, float input) {
    float y0 = input + r->coeff * r->y1 - r->radius * r->radius * r->y2;
    r->y2 = r->y1;
    r->y1 = y0;
    return y0;
}

void sp_resonator_set_freq(sp_resonator_t *r, float freq, float sr) {
    float w = 2.0f * 3.14159265f * freq / sr;
    r->freq = freq;
    r->coeff = 2.0f * r->radius * sp_res_cos(w);
}

float sp_resonator_magnitude(const sp_resonator_t *r) {
    float y_sq = r->y1 * r->y1;
    float guess = y_sq * 0.5f;
    int i;
    if (y_sq < 1e-10f) return 0.0f;
    for (i = 0; i < 8; i++) {
        guess = 0.5f * (guess + y_sq / guess);
    }
    return guess;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C809: Digital resonator should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C809: Output should not be empty");
    assert!(
        code.contains("fn sp_resonator_init"),
        "C809: Should contain sp_resonator_init function"
    );
    assert!(
        code.contains("fn sp_resonator_process"),
        "C809: Should contain sp_resonator_process function"
    );
}

/// C810: Comb filter -- feedforward and feedback delay line
#[test]
fn c810_comb_filter() {
    let c_code = r#"
typedef struct {
    float buffer[2048];
    int delay;
    float feedback;
    int write_pos;
    int buf_size;
} sp_comb_t;

void sp_comb_init(sp_comb_t *c, int delay, float feedback) {
    int i;
    c->delay = delay < 2048 ? delay : 2047;
    c->feedback = feedback;
    c->write_pos = 0;
    c->buf_size = 2048;
    for (i = 0; i < 2048; i++) {
        c->buffer[i] = 0.0f;
    }
}

float sp_comb_ff_process(sp_comb_t *c, float input) {
    int read_pos = c->write_pos - c->delay;
    if (read_pos < 0) read_pos += c->buf_size;
    float delayed = c->buffer[read_pos];
    c->buffer[c->write_pos] = input;
    c->write_pos++;
    if (c->write_pos >= c->buf_size) c->write_pos = 0;
    return input + c->feedback * delayed;
}

float sp_comb_fb_process(sp_comb_t *c, float input) {
    int read_pos = c->write_pos - c->delay;
    if (read_pos < 0) read_pos += c->buf_size;
    float delayed = c->buffer[read_pos];
    float output = input + c->feedback * delayed;
    c->buffer[c->write_pos] = output;
    c->write_pos++;
    if (c->write_pos >= c->buf_size) c->write_pos = 0;
    return output;
}

void sp_comb_set_delay(sp_comb_t *c, int delay) {
    c->delay = delay < c->buf_size ? delay : c->buf_size - 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C810: Comb filter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C810: Output should not be empty");
    assert!(
        code.contains("fn sp_comb_init"),
        "C810: Should contain sp_comb_init function"
    );
    assert!(
        code.contains("fn sp_comb_ff_process"),
        "C810: Should contain sp_comb_ff_process function"
    );
    assert!(
        code.contains("fn sp_comb_fb_process"),
        "C810: Should contain sp_comb_fb_process function"
    );
}

// ============================================================================
// C811-C815: Utility Filters and Detectors
// ============================================================================

/// C811: All-pass filter -- phase-shifting without magnitude change
#[test]
fn c811_allpass_filter() {
    let c_code = r#"
typedef struct {
    float buffer[1024];
    int delay;
    float coeff;
    int write_pos;
    int buf_size;
} sp_allpass_t;

void sp_allpass_init(sp_allpass_t *ap, int delay, float coeff) {
    int i;
    ap->delay = delay < 1024 ? delay : 1023;
    ap->coeff = coeff;
    ap->write_pos = 0;
    ap->buf_size = 1024;
    for (i = 0; i < 1024; i++) {
        ap->buffer[i] = 0.0f;
    }
}

float sp_allpass_process(sp_allpass_t *ap, float input) {
    int read_pos = ap->write_pos - ap->delay;
    if (read_pos < 0) read_pos += ap->buf_size;
    float delayed = ap->buffer[read_pos];
    float output = -ap->coeff * input + delayed
                   + ap->coeff * delayed;
    ap->buffer[ap->write_pos] = input + ap->coeff * delayed;
    ap->write_pos++;
    if (ap->write_pos >= ap->buf_size) ap->write_pos = 0;
    return output;
}

void sp_allpass_reset(sp_allpass_t *ap) {
    int i;
    ap->write_pos = 0;
    for (i = 0; i < ap->buf_size; i++) {
        ap->buffer[i] = 0.0f;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C811: All-pass filter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C811: Output should not be empty");
    assert!(
        code.contains("fn sp_allpass_init"),
        "C811: Should contain sp_allpass_init function"
    );
    assert!(
        code.contains("fn sp_allpass_process"),
        "C811: Should contain sp_allpass_process function"
    );
}

/// C812: Sample rate converter -- linear interpolation resampler
#[test]
fn c812_sample_rate_converter() {
    let c_code = r#"
typedef struct {
    float ratio;
    float phase;
    float last_sample;
} sp_resampler_t;

void sp_resampler_init(sp_resampler_t *r, float in_rate, float out_rate) {
    r->ratio = in_rate / out_rate;
    r->phase = 0.0f;
    r->last_sample = 0.0f;
}

int sp_resample_linear(sp_resampler_t *r, const float *input, int in_len,
                        float *output, int max_out) {
    int in_idx = 0;
    int out_idx = 0;
    float prev = r->last_sample;
    while (in_idx < in_len && out_idx < max_out) {
        while (r->phase < 1.0f && out_idx < max_out) {
            float curr = input[in_idx < in_len ? in_idx : in_len - 1];
            output[out_idx] = prev + r->phase * (curr - prev);
            out_idx++;
            r->phase += r->ratio;
        }
        r->phase -= 1.0f;
        prev = input[in_idx];
        in_idx++;
    }
    r->last_sample = prev;
    return out_idx;
}

void sp_resampler_reset(sp_resampler_t *r) {
    r->phase = 0.0f;
    r->last_sample = 0.0f;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C812: Sample rate converter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C812: Output should not be empty");
    assert!(
        code.contains("fn sp_resampler_init"),
        "C812: Should contain sp_resampler_init function"
    );
    assert!(
        code.contains("fn sp_resample_linear"),
        "C812: Should contain sp_resample_linear function"
    );
}

/// C813: Envelope detector -- peak hold with exponential decay
#[test]
fn c813_envelope_detector() {
    let c_code = r#"
typedef struct {
    float attack_coeff;
    float release_coeff;
    float envelope;
} sp_envelope_t;

void sp_envelope_init(sp_envelope_t *e, float attack_ms, float release_ms,
                       float sample_rate) {
    float attack_samples = attack_ms * sample_rate / 1000.0f;
    float release_samples = release_ms * sample_rate / 1000.0f;
    e->attack_coeff = attack_samples > 1.0f ? 1.0f / attack_samples : 1.0f;
    e->release_coeff = release_samples > 1.0f ? 1.0f / release_samples : 1.0f;
    e->envelope = 0.0f;
}

float sp_envelope_process(sp_envelope_t *e, float input) {
    float abs_input = input >= 0.0f ? input : -input;
    if (abs_input > e->envelope) {
        e->envelope += e->attack_coeff * (abs_input - e->envelope);
    } else {
        e->envelope += e->release_coeff * (abs_input - e->envelope);
    }
    return e->envelope;
}

void sp_envelope_process_block(sp_envelope_t *e, const float *input,
                                float *output, int n) {
    int i;
    for (i = 0; i < n; i++) {
        output[i] = sp_envelope_process(e, input[i]);
    }
}

void sp_envelope_reset(sp_envelope_t *e) {
    e->envelope = 0.0f;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C813: Envelope detector should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C813: Output should not be empty");
    assert!(
        code.contains("fn sp_envelope_init"),
        "C813: Should contain sp_envelope_init function"
    );
    assert!(
        code.contains("fn sp_envelope_process"),
        "C813: Should contain sp_envelope_process function"
    );
}

/// C814: Zero-crossing detector -- counts sign changes in a signal
#[test]
fn c814_zero_crossing_detector() {
    let c_code = r#"
int sp_zero_crossings(const float *signal, int n) {
    int count = 0;
    int i;
    for (i = 1; i < n; i++) {
        if ((signal[i - 1] >= 0.0f && signal[i] < 0.0f) ||
            (signal[i - 1] < 0.0f && signal[i] >= 0.0f)) {
            count++;
        }
    }
    return count;
}

float sp_zero_crossing_rate(const float *signal, int n, float sample_rate) {
    int crossings = sp_zero_crossings(signal, n);
    if (n <= 1) return 0.0f;
    return (float)crossings * sample_rate / (float)(n - 1);
}

float sp_zero_crossing_position(float y0, float y1) {
    if (y0 == y1) return 0.5f;
    return y0 / (y0 - y1);
}

void sp_zero_crossing_indices(const float *signal, int n, int *indices,
                               int max_idx, int *count) {
    int i;
    *count = 0;
    for (i = 1; i < n && *count < max_idx; i++) {
        if ((signal[i - 1] >= 0.0f && signal[i] < 0.0f) ||
            (signal[i - 1] < 0.0f && signal[i] >= 0.0f)) {
            indices[*count] = i;
            (*count)++;
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C814: Zero-crossing detector should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C814: Output should not be empty");
    assert!(
        code.contains("fn sp_zero_crossings"),
        "C814: Should contain sp_zero_crossings function"
    );
    assert!(
        code.contains("fn sp_zero_crossing_rate"),
        "C814: Should contain sp_zero_crossing_rate function"
    );
}

/// C815: Peak detector -- finds local maxima in a signal
#[test]
fn c815_peak_detector() {
    let c_code = r#"
typedef struct {
    int index;
    float value;
} sp_peak_t;

int sp_find_peaks(const float *signal, int n, sp_peak_t *peaks,
                   int max_peaks, float threshold) {
    int count = 0;
    int i;
    for (i = 1; i < n - 1 && count < max_peaks; i++) {
        if (signal[i] > signal[i - 1] && signal[i] > signal[i + 1]
            && signal[i] > threshold) {
            peaks[count].index = i;
            peaks[count].value = signal[i];
            count++;
        }
    }
    return count;
}

float sp_peak_interpolate(const float *signal, int peak_idx) {
    float y0 = signal[peak_idx - 1];
    float y1 = signal[peak_idx];
    float y2 = signal[peak_idx + 1];
    float delta = 0.5f * (y2 - y0) / (2.0f * y1 - y0 - y2);
    return (float)peak_idx + delta;
}

int sp_find_highest_peak(const sp_peak_t *peaks, int n) {
    int best = 0;
    int i;
    float max_val = peaks[0].value;
    for (i = 1; i < n; i++) {
        if (peaks[i].value > max_val) {
            max_val = peaks[i].value;
            best = i;
        }
    }
    return best;
}

void sp_sort_peaks_by_value(sp_peak_t *peaks, int n) {
    int i, j;
    for (i = 0; i < n - 1; i++) {
        for (j = 0; j < n - i - 1; j++) {
            if (peaks[j].value < peaks[j + 1].value) {
                sp_peak_t tmp = peaks[j];
                peaks[j] = peaks[j + 1];
                peaks[j + 1] = tmp;
            }
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C815: Peak detector should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C815: Output should not be empty");
    assert!(
        code.contains("fn sp_find_peaks"),
        "C815: Should contain sp_find_peaks function"
    );
    assert!(
        code.contains("fn sp_peak_interpolate"),
        "C815: Should contain sp_peak_interpolate function"
    );
}

// ============================================================================
// C816-C820: Smoothing and Analysis
// ============================================================================

/// C816: Moving average filter -- efficient circular buffer implementation
#[test]
fn c816_moving_average_filter() {
    let c_code = r#"
typedef struct {
    float buffer[256];
    float sum;
    int window_size;
    int write_pos;
    int count;
} sp_movavg_t;

void sp_movavg_init(sp_movavg_t *ma, int window_size) {
    int i;
    ma->window_size = window_size < 256 ? window_size : 256;
    ma->sum = 0.0f;
    ma->write_pos = 0;
    ma->count = 0;
    for (i = 0; i < 256; i++) {
        ma->buffer[i] = 0.0f;
    }
}

float sp_movavg_process(sp_movavg_t *ma, float input) {
    ma->sum -= ma->buffer[ma->write_pos];
    ma->buffer[ma->write_pos] = input;
    ma->sum += input;
    ma->write_pos++;
    if (ma->write_pos >= ma->window_size) ma->write_pos = 0;
    if (ma->count < ma->window_size) ma->count++;
    return ma->sum / (float)ma->count;
}

void sp_movavg_process_block(sp_movavg_t *ma, const float *input,
                              float *output, int n) {
    int i;
    for (i = 0; i < n; i++) {
        output[i] = sp_movavg_process(ma, input[i]);
    }
}

void sp_movavg_reset(sp_movavg_t *ma) {
    int i;
    ma->sum = 0.0f;
    ma->write_pos = 0;
    ma->count = 0;
    for (i = 0; i < ma->window_size; i++) {
        ma->buffer[i] = 0.0f;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C816: Moving average filter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C816: Output should not be empty");
    assert!(
        code.contains("fn sp_movavg_init"),
        "C816: Should contain sp_movavg_init function"
    );
    assert!(
        code.contains("fn sp_movavg_process"),
        "C816: Should contain sp_movavg_process function"
    );
}

/// C817: Median filter -- bubble sort on small window
#[test]
fn c817_median_filter() {
    let c_code = r#"
typedef struct {
    float buffer[32];
    int window_size;
    int write_pos;
    int count;
} sp_median_t;

void sp_median_init(sp_median_t *mf, int window_size) {
    int i;
    mf->window_size = window_size < 32 ? window_size : 31;
    mf->write_pos = 0;
    mf->count = 0;
    for (i = 0; i < 32; i++) {
        mf->buffer[i] = 0.0f;
    }
}

static void sp_median_sort(float *arr, int n) {
    int i, j;
    for (i = 0; i < n - 1; i++) {
        for (j = 0; j < n - i - 1; j++) {
            if (arr[j] > arr[j + 1]) {
                float tmp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = tmp;
            }
        }
    }
}

float sp_median_process(sp_median_t *mf, float input) {
    float sorted[32];
    int i, n;
    mf->buffer[mf->write_pos] = input;
    mf->write_pos++;
    if (mf->write_pos >= mf->window_size) mf->write_pos = 0;
    if (mf->count < mf->window_size) mf->count++;
    n = mf->count;
    for (i = 0; i < n; i++) {
        sorted[i] = mf->buffer[i];
    }
    sp_median_sort(sorted, n);
    if (n % 2 == 1) {
        return sorted[n / 2];
    } else {
        return (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0f;
    }
}

void sp_median_reset(sp_median_t *mf) {
    int i;
    mf->write_pos = 0;
    mf->count = 0;
    for (i = 0; i < 32; i++) {
        mf->buffer[i] = 0.0f;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C817: Median filter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C817: Output should not be empty");
    assert!(
        code.contains("fn sp_median_init"),
        "C817: Should contain sp_median_init function"
    );
    assert!(
        code.contains("fn sp_median_process"),
        "C817: Should contain sp_median_process function"
    );
}

/// C818: Goertzel algorithm -- efficient single-bin DFT computation
#[test]
fn c818_goertzel_algorithm() {
    let c_code = r#"
static float sp_goertzel_cos(float x) {
    float x2 = x * x;
    float x4 = x2 * x2;
    float x6 = x4 * x2;
    return 1.0f - x2 / 2.0f + x4 / 24.0f - x6 / 720.0f;
}

typedef struct {
    float coeff;
    float s1, s2;
    int n;
} sp_goertzel_t;

void sp_goertzel_init(sp_goertzel_t *g, float target_freq, float sample_rate,
                       int block_size) {
    float k = target_freq * (float)block_size / sample_rate;
    float w = 2.0f * 3.14159265f * k / (float)block_size;
    g->coeff = 2.0f * sp_goertzel_cos(w);
    g->s1 = 0.0f;
    g->s2 = 0.0f;
    g->n = block_size;
}

void sp_goertzel_process_sample(sp_goertzel_t *g, float sample) {
    float s0 = sample + g->coeff * g->s1 - g->s2;
    g->s2 = g->s1;
    g->s1 = s0;
}

float sp_goertzel_magnitude_sq(const sp_goertzel_t *g) {
    return g->s1 * g->s1 + g->s2 * g->s2 - g->coeff * g->s1 * g->s2;
}

float sp_goertzel_detect(const float *signal, int n, float target_freq,
                          float sample_rate) {
    sp_goertzel_t g;
    int i;
    sp_goertzel_init(&g, target_freq, sample_rate, n);
    for (i = 0; i < n; i++) {
        sp_goertzel_process_sample(&g, signal[i]);
    }
    return sp_goertzel_magnitude_sq(&g);
}

void sp_goertzel_reset(sp_goertzel_t *g) {
    g->s1 = 0.0f;
    g->s2 = 0.0f;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C818: Goertzel algorithm should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C818: Output should not be empty");
    assert!(
        code.contains("fn sp_goertzel_init"),
        "C818: Should contain sp_goertzel_init function"
    );
    assert!(
        code.contains("fn sp_goertzel_detect"),
        "C818: Should contain sp_goertzel_detect function"
    );
}

/// C819: Phase-locked loop -- tracks phase of incoming signal
#[test]
fn c819_phase_locked_loop() {
    let c_code = r#"
static float sp_pll_sin(float x) {
    float x3 = x * x * x;
    float x5 = x3 * x * x;
    float x7 = x5 * x * x;
    return x - x3 / 6.0f + x5 / 120.0f - x7 / 5040.0f;
}

static float sp_pll_cos(float x) {
    float x2 = x * x;
    float x4 = x2 * x2;
    float x6 = x4 * x2;
    return 1.0f - x2 / 2.0f + x4 / 24.0f - x6 / 720.0f;
}

typedef struct {
    float phase;
    float freq;
    float alpha;
    float beta;
    float pi2;
} sp_pll_t;

void sp_pll_init(sp_pll_t *pll, float center_freq, float bandwidth,
                  float sample_rate) {
    pll->phase = 0.0f;
    pll->freq = 2.0f * 3.14159265f * center_freq / sample_rate;
    pll->alpha = 2.0f * bandwidth / sample_rate;
    pll->beta = pll->alpha * pll->alpha / 4.0f;
    pll->pi2 = 2.0f * 3.14159265f;
}

float sp_pll_process(sp_pll_t *pll, float input) {
    float ref_signal = sp_pll_cos(pll->phase);
    float error = input * sp_pll_sin(pll->phase);
    pll->freq += pll->beta * error;
    pll->phase += pll->freq + pll->alpha * error;
    while (pll->phase > pll->pi2) pll->phase -= pll->pi2;
    while (pll->phase < 0.0f) pll->phase += pll->pi2;
    return ref_signal;
}

float sp_pll_get_phase(const sp_pll_t *pll) {
    return pll->phase;
}

float sp_pll_get_frequency(const sp_pll_t *pll) {
    return pll->freq;
}

void sp_pll_reset(sp_pll_t *pll) {
    pll->phase = 0.0f;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C819: Phase-locked loop should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C819: Output should not be empty");
    assert!(
        code.contains("fn sp_pll_init"),
        "C819: Should contain sp_pll_init function"
    );
    assert!(
        code.contains("fn sp_pll_process"),
        "C819: Should contain sp_pll_process function"
    );
}

/// C820: Noise gate -- attenuates signal below threshold
#[test]
fn c820_noise_gate() {
    let c_code = r#"
typedef struct {
    float threshold;
    float attack_coeff;
    float release_coeff;
    float gain;
    float hold_time;
    float hold_counter;
    float sample_rate;
} sp_noisegate_t;

void sp_noisegate_init(sp_noisegate_t *ng, float threshold_db,
                        float attack_ms, float release_ms,
                        float hold_ms, float sample_rate) {
    float t_linear = 1.0f;
    float db = threshold_db;
    int i;
    for (i = 0; i < 3; i++) {
        t_linear *= (1.0f + db / 20.0f);
    }
    if (t_linear < 0.0f) t_linear = 0.001f;
    ng->threshold = t_linear;
    ng->attack_coeff = 1.0f / (attack_ms * sample_rate / 1000.0f + 1.0f);
    ng->release_coeff = 1.0f / (release_ms * sample_rate / 1000.0f + 1.0f);
    ng->hold_time = hold_ms * sample_rate / 1000.0f;
    ng->hold_counter = 0.0f;
    ng->gain = 0.0f;
    ng->sample_rate = sample_rate;
}

float sp_noisegate_process(sp_noisegate_t *ng, float input) {
    float abs_in = input >= 0.0f ? input : -input;
    float target_gain;
    if (abs_in > ng->threshold) {
        target_gain = 1.0f;
        ng->hold_counter = ng->hold_time;
    } else if (ng->hold_counter > 0.0f) {
        target_gain = 1.0f;
        ng->hold_counter -= 1.0f;
    } else {
        target_gain = 0.0f;
    }
    if (target_gain > ng->gain) {
        ng->gain += ng->attack_coeff * (target_gain - ng->gain);
    } else {
        ng->gain += ng->release_coeff * (target_gain - ng->gain);
    }
    return input * ng->gain;
}

void sp_noisegate_process_block(sp_noisegate_t *ng, const float *input,
                                 float *output, int n) {
    int i;
    for (i = 0; i < n; i++) {
        output[i] = sp_noisegate_process(ng, input[i]);
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C820: Noise gate should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C820: Output should not be empty");
    assert!(
        code.contains("fn sp_noisegate_init"),
        "C820: Should contain sp_noisegate_init function"
    );
    assert!(
        code.contains("fn sp_noisegate_process"),
        "C820: Should contain sp_noisegate_process function"
    );
}

// ============================================================================
// C821-C825: Audio Processing
// ============================================================================

/// C821: Compressor/limiter -- dynamic range compression with knee
#[test]
fn c821_compressor_limiter() {
    let c_code = r#"
typedef struct {
    float threshold;
    float ratio;
    float attack_coeff;
    float release_coeff;
    float envelope;
    float makeup_gain;
} sp_compressor_t;

void sp_compressor_init(sp_compressor_t *c, float threshold, float ratio,
                         float attack_ms, float release_ms,
                         float makeup_db, float sample_rate) {
    c->threshold = threshold;
    c->ratio = ratio;
    c->attack_coeff = 1.0f / (attack_ms * sample_rate / 1000.0f + 1.0f);
    c->release_coeff = 1.0f / (release_ms * sample_rate / 1000.0f + 1.0f);
    c->envelope = 0.0f;
    c->makeup_gain = 1.0f + makeup_db / 20.0f;
}

float sp_compressor_process(sp_compressor_t *c, float input) {
    float abs_in = input >= 0.0f ? input : -input;
    float gain;
    if (abs_in > c->envelope) {
        c->envelope += c->attack_coeff * (abs_in - c->envelope);
    } else {
        c->envelope += c->release_coeff * (abs_in - c->envelope);
    }
    if (c->envelope > c->threshold) {
        float over = c->envelope - c->threshold;
        float compressed_over = over / c->ratio;
        gain = (c->threshold + compressed_over) / c->envelope;
    } else {
        gain = 1.0f;
    }
    return input * gain * c->makeup_gain;
}

void sp_compressor_process_block(sp_compressor_t *c, const float *input,
                                  float *output, int n) {
    int i;
    for (i = 0; i < n; i++) {
        output[i] = sp_compressor_process(c, input[i]);
    }
}

void sp_compressor_reset(sp_compressor_t *c) {
    c->envelope = 0.0f;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C821: Compressor/limiter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C821: Output should not be empty");
    assert!(
        code.contains("fn sp_compressor_init"),
        "C821: Should contain sp_compressor_init function"
    );
    assert!(
        code.contains("fn sp_compressor_process"),
        "C821: Should contain sp_compressor_process function"
    );
}

/// C822: Pitch detector -- autocorrelation-based fundamental frequency estimation
#[test]
fn c822_pitch_detector_autocorrelation() {
    let c_code = r#"
typedef struct {
    float sample_rate;
    float min_freq;
    float max_freq;
    int min_lag;
    int max_lag;
} sp_pitchdet_t;

void sp_pitchdet_init(sp_pitchdet_t *pd, float sample_rate,
                       float min_freq, float max_freq) {
    pd->sample_rate = sample_rate;
    pd->min_freq = min_freq;
    pd->max_freq = max_freq;
    pd->min_lag = (int)(sample_rate / max_freq);
    pd->max_lag = (int)(sample_rate / min_freq);
    if (pd->min_lag < 1) pd->min_lag = 1;
    if (pd->max_lag > 512) pd->max_lag = 512;
}

static void sp_pitchdet_autocorr(const float *x, int n, float *r,
                                  int min_lag, int max_lag) {
    int lag, i;
    for (lag = min_lag; lag <= max_lag && lag < n; lag++) {
        r[lag] = 0.0f;
        for (i = 0; i < n - lag; i++) {
            r[lag] += x[i] * x[i + lag];
        }
    }
}

float sp_pitchdet_detect(sp_pitchdet_t *pd, const float *signal, int n) {
    float acf[513];
    int lag;
    int best_lag = 0;
    float best_val = -1.0f;
    float energy = 0.0f;
    int i;
    for (i = 0; i < n; i++) energy += signal[i] * signal[i];
    if (energy < 1e-6f) return 0.0f;
    sp_pitchdet_autocorr(signal, n, acf, pd->min_lag, pd->max_lag);
    for (lag = pd->min_lag; lag <= pd->max_lag && lag < n; lag++) {
        float normalized = acf[lag] / energy;
        if (normalized > best_val) {
            best_val = normalized;
            best_lag = lag;
        }
    }
    if (best_val < 0.3f || best_lag == 0) return 0.0f;
    return pd->sample_rate / (float)best_lag;
}

int sp_pitchdet_is_voiced(const float *signal, int n, float threshold) {
    float energy = 0.0f;
    int crossings = 0;
    int i;
    for (i = 0; i < n; i++) energy += signal[i] * signal[i];
    for (i = 1; i < n; i++) {
        if ((signal[i-1] >= 0.0f && signal[i] < 0.0f) ||
            (signal[i-1] < 0.0f && signal[i] >= 0.0f))
            crossings++;
    }
    return (energy / (float)n > threshold && crossings < n / 4) ? 1 : 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C822: Pitch detector should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C822: Output should not be empty");
    assert!(
        code.contains("fn sp_pitchdet_init"),
        "C822: Should contain sp_pitchdet_init function"
    );
    assert!(
        code.contains("fn sp_pitchdet_detect"),
        "C822: Should contain sp_pitchdet_detect function"
    );
}

/// C823: Parametric equalizer -- cascaded biquad sections for frequency shaping
#[test]
fn c823_parametric_equalizer() {
    let c_code = r#"
static float sp_eq_cos(float x) {
    float x2 = x * x;
    float x4 = x2 * x2;
    return 1.0f - x2 / 2.0f + x4 / 24.0f;
}

static float sp_eq_sin(float x) {
    float x3 = x * x * x;
    float x5 = x3 * x * x;
    return x - x3 / 6.0f + x5 / 120.0f;
}

typedef struct {
    float b0, b1, b2;
    float a1, a2;
    float z1, z2;
} sp_eq_band_t;

typedef struct {
    sp_eq_band_t bands[8];
    int num_bands;
} sp_equalizer_t;

void sp_eq_init(sp_equalizer_t *eq) {
    int i;
    eq->num_bands = 0;
    for (i = 0; i < 8; i++) {
        eq->bands[i].b0 = 1.0f;
        eq->bands[i].b1 = 0.0f;
        eq->bands[i].b2 = 0.0f;
        eq->bands[i].a1 = 0.0f;
        eq->bands[i].a2 = 0.0f;
        eq->bands[i].z1 = 0.0f;
        eq->bands[i].z2 = 0.0f;
    }
}

void sp_eq_add_band(sp_equalizer_t *eq, float freq, float gain_db,
                     float q, float sample_rate) {
    if (eq->num_bands >= 8) return;
    float w0 = 2.0f * 3.14159265f * freq / sample_rate;
    float A = 1.0f + gain_db / 40.0f;
    float alpha = sp_eq_sin(w0) / (2.0f * q);
    float cos_w0 = sp_eq_cos(w0);
    float norm = 1.0f + alpha / A;
    sp_eq_band_t *b = &eq->bands[eq->num_bands];
    b->b0 = (1.0f + alpha * A) / norm;
    b->b1 = (-2.0f * cos_w0) / norm;
    b->b2 = (1.0f - alpha * A) / norm;
    b->a1 = b->b1;
    b->a2 = (1.0f - alpha / A) / norm;
    b->z1 = 0.0f;
    b->z2 = 0.0f;
    eq->num_bands++;
}

float sp_eq_process(sp_equalizer_t *eq, float input) {
    float sample = input;
    int i;
    for (i = 0; i < eq->num_bands; i++) {
        sp_eq_band_t *b = &eq->bands[i];
        float out = b->b0 * sample + b->z1;
        b->z1 = b->b1 * sample - b->a1 * out + b->z2;
        b->z2 = b->b2 * sample - b->a2 * out;
        sample = out;
    }
    return sample;
}

void sp_eq_reset(sp_equalizer_t *eq) {
    int i;
    for (i = 0; i < eq->num_bands; i++) {
        eq->bands[i].z1 = 0.0f;
        eq->bands[i].z2 = 0.0f;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C823: Parametric equalizer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C823: Output should not be empty");
    assert!(
        code.contains("fn sp_eq_init"),
        "C823: Should contain sp_eq_init function"
    );
    assert!(
        code.contains("fn sp_eq_process"),
        "C823: Should contain sp_eq_process function"
    );
    assert!(
        code.contains("fn sp_eq_add_band"),
        "C823: Should contain sp_eq_add_band function"
    );
}

/// C824: Polyphase resampler -- efficient multi-rate conversion
#[test]
fn c824_polyphase_resampler() {
    let c_code = r#"
typedef struct {
    float coeffs[128];
    float delay[32];
    int num_phases;
    int taps_per_phase;
    int write_pos;
} sp_polyphase_t;

void sp_polyphase_init(sp_polyphase_t *pp, int num_phases, int taps_per_phase) {
    int i;
    int total;
    pp->num_phases = num_phases < 16 ? num_phases : 16;
    pp->taps_per_phase = taps_per_phase < 8 ? taps_per_phase : 8;
    pp->write_pos = 0;
    total = pp->num_phases * pp->taps_per_phase;
    for (i = 0; i < 128; i++) {
        pp->coeffs[i] = (i < total) ? 1.0f / (float)total : 0.0f;
    }
    for (i = 0; i < 32; i++) {
        pp->delay[i] = 0.0f;
    }
}

void sp_polyphase_set_coeffs(sp_polyphase_t *pp, const float *coeffs) {
    int i;
    int total = pp->num_phases * pp->taps_per_phase;
    for (i = 0; i < total && i < 128; i++) {
        pp->coeffs[i] = coeffs[i];
    }
}

int sp_polyphase_interpolate(sp_polyphase_t *pp, float input,
                              float *output, int max_out) {
    int phase, tap;
    int out_count = 0;
    pp->delay[pp->write_pos] = input;
    for (phase = 0; phase < pp->num_phases && out_count < max_out; phase++) {
        float sum = 0.0f;
        int coeff_base = phase * pp->taps_per_phase;
        for (tap = 0; tap < pp->taps_per_phase; tap++) {
            int idx = pp->write_pos - tap;
            if (idx < 0) idx += 32;
            sum += pp->coeffs[coeff_base + tap] * pp->delay[idx % 32];
        }
        output[out_count] = sum;
        out_count++;
    }
    pp->write_pos++;
    if (pp->write_pos >= 32) pp->write_pos = 0;
    return out_count;
}

void sp_polyphase_reset(sp_polyphase_t *pp) {
    int i;
    pp->write_pos = 0;
    for (i = 0; i < 32; i++) {
        pp->delay[i] = 0.0f;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C824: Polyphase resampler should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C824: Output should not be empty");
    assert!(
        code.contains("fn sp_polyphase_init"),
        "C824: Should contain sp_polyphase_init function"
    );
    assert!(
        code.contains("fn sp_polyphase_interpolate"),
        "C824: Should contain sp_polyphase_interpolate function"
    );
}

/// C825: Haar wavelet transform -- simplest wavelet for signal decomposition
#[test]
fn c825_haar_wavelet_transform() {
    let c_code = r#"
void sp_haar_forward(float *data, int n) {
    float temp[256];
    int len;
    int i;
    for (len = n; len >= 2; len >>= 1) {
        int half = len >> 1;
        for (i = 0; i < half; i++) {
            temp[i] = (data[2 * i] + data[2 * i + 1]) / 2.0f;
            temp[half + i] = (data[2 * i] - data[2 * i + 1]) / 2.0f;
        }
        for (i = 0; i < len && i < 256; i++) {
            data[i] = temp[i];
        }
    }
}

void sp_haar_inverse(float *data, int n) {
    float temp[256];
    int len;
    int i;
    for (len = 2; len <= n; len <<= 1) {
        int half = len >> 1;
        for (i = 0; i < half; i++) {
            temp[2 * i] = data[i] + data[half + i];
            temp[2 * i + 1] = data[i] - data[half + i];
        }
        for (i = 0; i < len && i < 256; i++) {
            data[i] = temp[i];
        }
    }
}

void sp_haar_threshold(float *coeffs, int n, float threshold) {
    int i;
    for (i = 0; i < n; i++) {
        if (coeffs[i] > -threshold && coeffs[i] < threshold) {
            coeffs[i] = 0.0f;
        }
    }
}

int sp_haar_nonzero_count(const float *coeffs, int n) {
    int count = 0;
    int i;
    for (i = 0; i < n; i++) {
        if (coeffs[i] != 0.0f) count++;
    }
    return count;
}

float sp_haar_energy(const float *coeffs, int n) {
    float energy = 0.0f;
    int i;
    for (i = 0; i < n; i++) {
        energy += coeffs[i] * coeffs[i];
    }
    return energy;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C825: Haar wavelet transform should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C825: Output should not be empty");
    assert!(
        code.contains("fn sp_haar_forward"),
        "C825: Should contain sp_haar_forward function"
    );
    assert!(
        code.contains("fn sp_haar_inverse"),
        "C825: Should contain sp_haar_inverse function"
    );
    assert!(
        code.contains("fn sp_haar_threshold"),
        "C825: Should contain sp_haar_threshold function"
    );
}
