//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C401-C425: Audio/DSP/Signal Processing patterns -- the kind of C code found
//! in audio engines, synthesizers, effects processors, and digital signal
//! processing libraries.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world audio and DSP programming patterns commonly
//! found in PortAudio, JUCE, libsndfile, FFTW, Sox, and similar audio
//! libraries -- all expressed as valid C99.
//!
//! Organization:
//! - C401-C405: Buffers, FIR/IIR filters, FFT, sample rate conversion
//! - C406-C410: Synthesis (ADSR, wavetable, mixer, level meter, compressor)
//! - C411-C415: Effects (delay, noise gate, EQ, crossfade, MIDI)
//! - C416-C420: Analysis (window functions, pitch detect, dithering, convolution, spectral)
//! - C421-C425: Polyphony, envelope follower, format conversion, click detect, beat detect
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C401-C405: Buffers, FIR/IIR Filters, FFT, Sample Rate Conversion
// ============================================================================

#[test]
fn c401_ring_buffer_for_audio_samples() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    float buffer[4096];
    uint32_t write_pos;
    uint32_t read_pos;
    uint32_t size;
    uint32_t mask;
} ring_buffer_t;

void ring_buffer_init(ring_buffer_t *rb, uint32_t size) {
    uint32_t i;
    rb->size = size;
    rb->mask = size - 1;
    rb->write_pos = 0;
    rb->read_pos = 0;
    for (i = 0; i < size && i < 4096; i++) {
        rb->buffer[i] = 0.0f;
    }
}

void ring_buffer_write(ring_buffer_t *rb, float sample) {
    rb->buffer[rb->write_pos & rb->mask] = sample;
    rb->write_pos++;
}

float ring_buffer_read(ring_buffer_t *rb) {
    float val = rb->buffer[rb->read_pos & rb->mask];
    rb->read_pos++;
    return val;
}

uint32_t ring_buffer_available(const ring_buffer_t *rb) {
    return rb->write_pos - rb->read_pos;
}

int ring_buffer_is_empty(const ring_buffer_t *rb) {
    return rb->write_pos == rb->read_pos;
}

int ring_buffer_is_full(const ring_buffer_t *rb) {
    return (rb->write_pos - rb->read_pos) >= rb->size;
}

float ring_buffer_peek(const ring_buffer_t *rb, uint32_t offset) {
    return rb->buffer[(rb->read_pos + offset) & rb->mask];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C401: Ring buffer for audio samples should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C401: Output should not be empty");
    assert!(
        code.contains("fn ring_buffer_init"),
        "C401: Should contain ring_buffer_init function"
    );
    assert!(
        code.contains("fn ring_buffer_write"),
        "C401: Should contain ring_buffer_write function"
    );
    assert!(
        code.contains("fn ring_buffer_read"),
        "C401: Should contain ring_buffer_read function"
    );
}

#[test]
fn c402_fir_filter_with_coefficient_array() {
    let c_code = r#"
typedef struct {
    float coeffs[64];
    float delay_line[64];
    int num_taps;
    int write_idx;
} fir_filter_t;

void fir_init(fir_filter_t *f, const float *coeffs, int num_taps) {
    int i;
    f->num_taps = num_taps < 64 ? num_taps : 64;
    f->write_idx = 0;
    for (i = 0; i < f->num_taps; i++) {
        f->coeffs[i] = coeffs[i];
        f->delay_line[i] = 0.0f;
    }
}

float fir_process_sample(fir_filter_t *f, float input) {
    float output = 0.0f;
    int i;
    int idx;
    f->delay_line[f->write_idx] = input;
    for (i = 0; i < f->num_taps; i++) {
        idx = f->write_idx - i;
        if (idx < 0) idx += f->num_taps;
        output += f->coeffs[i] * f->delay_line[idx];
    }
    f->write_idx = (f->write_idx + 1) % f->num_taps;
    return output;
}

void fir_process_block(fir_filter_t *f, float *data, int len) {
    int i;
    for (i = 0; i < len; i++) {
        data[i] = fir_process_sample(f, data[i]);
    }
}

void fir_reset(fir_filter_t *f) {
    int i;
    f->write_idx = 0;
    for (i = 0; i < f->num_taps; i++) {
        f->delay_line[i] = 0.0f;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C402: FIR filter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C402: Output should not be empty");
    assert!(
        code.contains("fn fir_init"),
        "C402: Should contain fir_init function"
    );
    assert!(
        code.contains("fn fir_process_sample"),
        "C402: Should contain fir_process_sample function"
    );
}

#[test]
fn c403_iir_biquad_filter_with_state() {
    let c_code = r#"
typedef struct {
    float b0;
    float b1;
    float b2;
    float a1;
    float a2;
    float x1;
    float x2;
    float y1;
    float y2;
} biquad_t;

void biquad_init(biquad_t *bq) {
    bq->b0 = 1.0f;
    bq->b1 = 0.0f;
    bq->b2 = 0.0f;
    bq->a1 = 0.0f;
    bq->a2 = 0.0f;
    bq->x1 = 0.0f;
    bq->x2 = 0.0f;
    bq->y1 = 0.0f;
    bq->y2 = 0.0f;
}

void biquad_set_lowpass(biquad_t *bq, float cutoff, float q, float sample_rate) {
    float w0 = 2.0f * 3.14159265f * cutoff / sample_rate;
    float alpha = w0 / (2.0f * q);
    float cos_w0 = 1.0f - w0 * w0 / 2.0f;
    float a0_inv = 1.0f / (1.0f + alpha);
    bq->b0 = ((1.0f - cos_w0) / 2.0f) * a0_inv;
    bq->b1 = (1.0f - cos_w0) * a0_inv;
    bq->b2 = bq->b0;
    bq->a1 = (-2.0f * cos_w0) * a0_inv;
    bq->a2 = (1.0f - alpha) * a0_inv;
}

float biquad_process(biquad_t *bq, float input) {
    float output = bq->b0 * input + bq->b1 * bq->x1 + bq->b2 * bq->x2
                 - bq->a1 * bq->y1 - bq->a2 * bq->y2;
    bq->x2 = bq->x1;
    bq->x1 = input;
    bq->y2 = bq->y1;
    bq->y1 = output;
    return output;
}

void biquad_process_block(biquad_t *bq, float *data, int len) {
    int i;
    for (i = 0; i < len; i++) {
        data[i] = biquad_process(bq, data[i]);
    }
}

void biquad_reset(biquad_t *bq) {
    bq->x1 = 0.0f;
    bq->x2 = 0.0f;
    bq->y1 = 0.0f;
    bq->y2 = 0.0f;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C403: IIR biquad filter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C403: Output should not be empty");
    assert!(
        code.contains("fn biquad_init"),
        "C403: Should contain biquad_init function"
    );
    assert!(
        code.contains("fn biquad_process"),
        "C403: Should contain biquad_process function"
    );
    assert!(
        code.contains("fn biquad_set_lowpass"),
        "C403: Should contain biquad_set_lowpass function"
    );
}

#[test]
fn c404_fft_butterfly_radix2() {
    let c_code = r#"
typedef struct {
    float re;
    float im;
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

void fft_butterfly(complex_t *data, int n, int stride) {
    int i;
    complex_t twiddle;
    complex_t temp;
    float angle;
    int half = n / 2;
    for (i = 0; i < half; i++) {
        angle = -2.0f * 3.14159265f * (float)i / (float)n;
        twiddle.re = 1.0f - angle * angle / 2.0f;
        twiddle.im = angle;
        temp = complex_mul(twiddle, data[(i + half) * stride]);
        data[(i + half) * stride] = complex_sub(data[i * stride], temp);
        data[i * stride] = complex_add(data[i * stride], temp);
    }
}

float complex_magnitude(complex_t c) {
    return c.re * c.re + c.im * c.im;
}

float complex_phase(complex_t c) {
    if (c.re == 0.0f && c.im == 0.0f) return 0.0f;
    return c.im / (c.re + 0.0001f);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C404: FFT butterfly operation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C404: Output should not be empty");
    assert!(
        code.contains("fn fft_butterfly"),
        "C404: Should contain fft_butterfly function"
    );
    assert!(
        code.contains("fn complex_mul"),
        "C404: Should contain complex_mul function"
    );
}

#[test]
fn c405_sample_rate_converter_linear_interp() {
    let c_code = r#"
typedef struct {
    float phase;
    float phase_inc;
    float prev_sample;
    float curr_sample;
} src_linear_t;

void src_linear_init(src_linear_t *src, float ratio) {
    src->phase = 0.0f;
    src->phase_inc = ratio;
    src->prev_sample = 0.0f;
    src->curr_sample = 0.0f;
}

float src_linear_lerp(float a, float b, float t) {
    return a + (b - a) * t;
}

int src_linear_process(src_linear_t *src, const float *input, int in_len,
                       float *output, int out_max) {
    int in_idx = 0;
    int out_idx = 0;
    while (out_idx < out_max && in_idx < in_len) {
        if (src->phase >= 1.0f) {
            src->phase -= 1.0f;
            src->prev_sample = src->curr_sample;
            in_idx++;
            if (in_idx < in_len) {
                src->curr_sample = input[in_idx];
            }
        }
        output[out_idx] = src_linear_lerp(src->prev_sample, src->curr_sample, src->phase);
        src->phase += src->phase_inc;
        out_idx++;
    }
    return out_idx;
}

void src_linear_reset(src_linear_t *src) {
    src->phase = 0.0f;
    src->prev_sample = 0.0f;
    src->curr_sample = 0.0f;
}

float src_compute_ratio(float src_rate, float dst_rate) {
    if (dst_rate <= 0.0f) return 1.0f;
    return src_rate / dst_rate;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C405: Sample rate converter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C405: Output should not be empty");
    assert!(
        code.contains("fn src_linear_init"),
        "C405: Should contain src_linear_init function"
    );
    assert!(
        code.contains("fn src_linear_process"),
        "C405: Should contain src_linear_process function"
    );
}

// ============================================================================
// C406-C410: Synthesis (ADSR, Wavetable, Mixer, Level Meter, Compressor)
// ============================================================================

#[test]
fn c406_adsr_envelope_generator() {
    let c_code = r#"
enum adsr_stage {
    ADSR_IDLE = 0,
    ADSR_ATTACK,
    ADSR_DECAY,
    ADSR_SUSTAIN,
    ADSR_RELEASE
};

typedef struct {
    enum adsr_stage stage;
    float level;
    float attack_rate;
    float decay_rate;
    float sustain_level;
    float release_rate;
    float sample_rate;
} adsr_t;

void adsr_init(adsr_t *env, float sample_rate) {
    env->stage = ADSR_IDLE;
    env->level = 0.0f;
    env->attack_rate = 0.01f;
    env->decay_rate = 0.001f;
    env->sustain_level = 0.7f;
    env->release_rate = 0.0005f;
    env->sample_rate = sample_rate;
}

void adsr_set_params(adsr_t *env, float attack_ms, float decay_ms,
                     float sustain, float release_ms) {
    env->attack_rate = 1.0f / (attack_ms * env->sample_rate / 1000.0f);
    env->decay_rate = 1.0f / (decay_ms * env->sample_rate / 1000.0f);
    env->sustain_level = sustain;
    env->release_rate = 1.0f / (release_ms * env->sample_rate / 1000.0f);
}

void adsr_gate_on(adsr_t *env) {
    env->stage = ADSR_ATTACK;
}

void adsr_gate_off(adsr_t *env) {
    env->stage = ADSR_RELEASE;
}

float adsr_process(adsr_t *env) {
    if (env->stage == ADSR_ATTACK) {
        env->level += env->attack_rate;
        if (env->level >= 1.0f) {
            env->level = 1.0f;
            env->stage = ADSR_DECAY;
        }
    } else if (env->stage == ADSR_DECAY) {
        env->level -= env->decay_rate;
        if (env->level <= env->sustain_level) {
            env->level = env->sustain_level;
            env->stage = ADSR_SUSTAIN;
        }
    } else if (env->stage == ADSR_RELEASE) {
        env->level -= env->release_rate;
        if (env->level <= 0.0f) {
            env->level = 0.0f;
            env->stage = ADSR_IDLE;
        }
    }
    return env->level;
}

int adsr_is_active(const adsr_t *env) {
    return env->stage != ADSR_IDLE;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C406: ADSR envelope generator should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C406: Output should not be empty");
    assert!(
        code.contains("fn adsr_init"),
        "C406: Should contain adsr_init function"
    );
    assert!(
        code.contains("fn adsr_process"),
        "C406: Should contain adsr_process function"
    );
}

#[test]
fn c407_wavetable_oscillator_with_phase_accumulator() {
    let c_code = r#"
typedef struct {
    float table[256];
    float phase;
    float phase_inc;
    int table_size;
} wavetable_osc_t;

void wavetable_init_sine(wavetable_osc_t *osc) {
    int i;
    float pi2 = 2.0f * 3.14159265f;
    osc->table_size = 256;
    osc->phase = 0.0f;
    osc->phase_inc = 0.0f;
    for (i = 0; i < 256; i++) {
        float angle = pi2 * (float)i / 256.0f;
        osc->table[i] = angle - (angle * angle * angle) / 6.0f;
    }
}

void wavetable_set_freq(wavetable_osc_t *osc, float freq, float sample_rate) {
    osc->phase_inc = freq * (float)osc->table_size / sample_rate;
}

float wavetable_process(wavetable_osc_t *osc) {
    int idx0;
    int idx1;
    float frac;
    float s0;
    float s1;
    float result;
    idx0 = (int)osc->phase;
    idx1 = (idx0 + 1) % osc->table_size;
    frac = osc->phase - (float)idx0;
    s0 = osc->table[idx0 % osc->table_size];
    s1 = osc->table[idx1];
    result = s0 + (s1 - s0) * frac;
    osc->phase += osc->phase_inc;
    if (osc->phase >= (float)osc->table_size) {
        osc->phase -= (float)osc->table_size;
    }
    return result;
}

void wavetable_process_block(wavetable_osc_t *osc, float *output, int len) {
    int i;
    for (i = 0; i < len; i++) {
        output[i] = wavetable_process(osc);
    }
}

void wavetable_reset_phase(wavetable_osc_t *osc) {
    osc->phase = 0.0f;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C407: Wavetable oscillator should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C407: Output should not be empty");
    assert!(
        code.contains("fn wavetable_init_sine"),
        "C407: Should contain wavetable_init_sine function"
    );
    assert!(
        code.contains("fn wavetable_process"),
        "C407: Should contain wavetable_process function"
    );
}

#[test]
fn c408_audio_mixer_multichannel_sum() {
    let c_code = r#"
#define MAX_CHANNELS 16

typedef struct {
    float gain[MAX_CHANNELS];
    float pan[MAX_CHANNELS];
    int active[MAX_CHANNELS];
    int num_channels;
} mixer_t;

void mixer_init(mixer_t *m, int num_channels) {
    int i;
    m->num_channels = num_channels < MAX_CHANNELS ? num_channels : MAX_CHANNELS;
    for (i = 0; i < MAX_CHANNELS; i++) {
        m->gain[i] = 1.0f;
        m->pan[i] = 0.5f;
        m->active[i] = 0;
    }
}

void mixer_set_gain(mixer_t *m, int channel, float gain) {
    if (channel >= 0 && channel < m->num_channels) {
        m->gain[channel] = gain;
    }
}

void mixer_set_pan(mixer_t *m, int channel, float pan) {
    if (channel >= 0 && channel < m->num_channels) {
        m->pan[channel] = pan < 0.0f ? 0.0f : (pan > 1.0f ? 1.0f : pan);
    }
}

void mixer_set_active(mixer_t *m, int channel, int active) {
    if (channel >= 0 && channel < m->num_channels) {
        m->active[channel] = active;
    }
}

void mixer_process_stereo(const mixer_t *m, const float *inputs,
                          int samples_per_channel, float *left, float *right) {
    int ch;
    int i;
    for (i = 0; i < samples_per_channel; i++) {
        left[i] = 0.0f;
        right[i] = 0.0f;
    }
    for (ch = 0; ch < m->num_channels; ch++) {
        if (!m->active[ch]) continue;
        float g = m->gain[ch];
        float pl = 1.0f - m->pan[ch];
        float pr = m->pan[ch];
        for (i = 0; i < samples_per_channel; i++) {
            float sample = inputs[ch * samples_per_channel + i] * g;
            left[i] += sample * pl;
            right[i] += sample * pr;
        }
    }
}

float mixer_get_peak(const float *buffer, int len) {
    float peak = 0.0f;
    int i;
    for (i = 0; i < len; i++) {
        float abs_val = buffer[i] < 0.0f ? -buffer[i] : buffer[i];
        if (abs_val > peak) peak = abs_val;
    }
    return peak;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C408: Audio mixer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C408: Output should not be empty");
    assert!(
        code.contains("fn mixer_init"),
        "C408: Should contain mixer_init function"
    );
    assert!(
        code.contains("fn mixer_process_stereo"),
        "C408: Should contain mixer_process_stereo function"
    );
}

#[test]
fn c409_peak_rms_level_meter() {
    let c_code = r#"
typedef struct {
    float peak;
    float rms_sum;
    int rms_count;
    float decay_rate;
    float peak_hold;
    int hold_counter;
    int hold_samples;
} level_meter_t;

void level_meter_init(level_meter_t *lm, float decay_per_sample, int hold_samples) {
    lm->peak = 0.0f;
    lm->rms_sum = 0.0f;
    lm->rms_count = 0;
    lm->decay_rate = decay_per_sample;
    lm->peak_hold = 0.0f;
    lm->hold_counter = 0;
    lm->hold_samples = hold_samples;
}

void level_meter_process(level_meter_t *lm, const float *samples, int len) {
    int i;
    for (i = 0; i < len; i++) {
        float abs_val = samples[i] < 0.0f ? -samples[i] : samples[i];
        if (abs_val > lm->peak) {
            lm->peak = abs_val;
        }
        lm->rms_sum += samples[i] * samples[i];
        lm->rms_count++;
    }
    lm->peak *= lm->decay_rate;
    if (lm->peak > lm->peak_hold) {
        lm->peak_hold = lm->peak;
        lm->hold_counter = lm->hold_samples;
    } else if (lm->hold_counter > 0) {
        lm->hold_counter--;
    } else {
        lm->peak_hold *= lm->decay_rate;
    }
}

float level_meter_get_peak(const level_meter_t *lm) {
    return lm->peak;
}

float level_meter_get_rms(const level_meter_t *lm) {
    if (lm->rms_count == 0) return 0.0f;
    return lm->rms_sum / (float)lm->rms_count;
}

float level_meter_get_peak_hold(const level_meter_t *lm) {
    return lm->peak_hold;
}

void level_meter_reset(level_meter_t *lm) {
    lm->peak = 0.0f;
    lm->rms_sum = 0.0f;
    lm->rms_count = 0;
    lm->peak_hold = 0.0f;
    lm->hold_counter = 0;
}

float level_to_db(float level) {
    if (level <= 0.0001f) return -80.0f;
    float db = 20.0f * (level - 1.0f);
    return db < -80.0f ? -80.0f : db;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C409: Peak/RMS level meter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C409: Output should not be empty");
    assert!(
        code.contains("fn level_meter_init"),
        "C409: Should contain level_meter_init function"
    );
    assert!(
        code.contains("fn level_meter_process"),
        "C409: Should contain level_meter_process function"
    );
    assert!(
        code.contains("fn level_to_db"),
        "C409: Should contain level_to_db function"
    );
}

#[test]
fn c410_compressor_limiter() {
    let c_code = r#"
typedef struct {
    float threshold;
    float ratio;
    float attack_coeff;
    float release_coeff;
    float env;
    float makeup_gain;
    float knee_width;
} compressor_t;

void compressor_init(compressor_t *c, float sample_rate) {
    c->threshold = 0.5f;
    c->ratio = 4.0f;
    c->attack_coeff = 1.0f - (1.0f / (0.01f * sample_rate));
    c->release_coeff = 1.0f - (1.0f / (0.1f * sample_rate));
    c->env = 0.0f;
    c->makeup_gain = 1.0f;
    c->knee_width = 0.1f;
}

void compressor_set_params(compressor_t *c, float threshold, float ratio,
                           float attack_ms, float release_ms, float sample_rate) {
    c->threshold = threshold;
    c->ratio = ratio;
    c->attack_coeff = 1.0f - (1.0f / (attack_ms * 0.001f * sample_rate));
    c->release_coeff = 1.0f - (1.0f / (release_ms * 0.001f * sample_rate));
}

float compressor_compute_gain(const compressor_t *c, float input_level) {
    float over;
    float gain;
    if (input_level < c->threshold) {
        return 1.0f;
    }
    over = input_level - c->threshold;
    gain = c->threshold + over / c->ratio;
    return gain / input_level;
}

void compressor_process(compressor_t *c, float *data, int len) {
    int i;
    for (i = 0; i < len; i++) {
        float abs_val = data[i] < 0.0f ? -data[i] : data[i];
        if (abs_val > c->env) {
            c->env = c->attack_coeff * c->env + (1.0f - c->attack_coeff) * abs_val;
        } else {
            c->env = c->release_coeff * c->env + (1.0f - c->release_coeff) * abs_val;
        }
        float gain = compressor_compute_gain(c, c->env);
        data[i] *= gain * c->makeup_gain;
    }
}

void compressor_set_makeup(compressor_t *c, float gain) {
    c->makeup_gain = gain;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C410: Compressor/limiter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C410: Output should not be empty");
    assert!(
        code.contains("fn compressor_init"),
        "C410: Should contain compressor_init function"
    );
    assert!(
        code.contains("fn compressor_process"),
        "C410: Should contain compressor_process function"
    );
}

// ============================================================================
// C411-C415: Effects (Delay, Noise Gate, EQ, Crossfade, MIDI)
// ============================================================================

#[test]
fn c411_delay_line_circular_buffer_feedback() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define DELAY_MAX_SAMPLES 48000

typedef struct {
    float buffer[DELAY_MAX_SAMPLES];
    uint32_t write_pos;
    uint32_t delay_samples;
    float feedback;
    float wet_mix;
    float dry_mix;
} delay_line_t;

void delay_init(delay_line_t *dl, uint32_t delay_samples, float feedback) {
    uint32_t i;
    dl->write_pos = 0;
    dl->delay_samples = delay_samples < DELAY_MAX_SAMPLES ? delay_samples : DELAY_MAX_SAMPLES;
    dl->feedback = feedback;
    dl->wet_mix = 0.5f;
    dl->dry_mix = 0.5f;
    for (i = 0; i < DELAY_MAX_SAMPLES; i++) {
        dl->buffer[i] = 0.0f;
    }
}

float delay_process(delay_line_t *dl, float input) {
    uint32_t read_pos;
    float delayed;
    float output;
    read_pos = (dl->write_pos + DELAY_MAX_SAMPLES - dl->delay_samples) % DELAY_MAX_SAMPLES;
    delayed = dl->buffer[read_pos];
    output = input * dl->dry_mix + delayed * dl->wet_mix;
    dl->buffer[dl->write_pos] = input + delayed * dl->feedback;
    dl->write_pos = (dl->write_pos + 1) % DELAY_MAX_SAMPLES;
    return output;
}

void delay_process_block(delay_line_t *dl, float *data, int len) {
    int i;
    for (i = 0; i < len; i++) {
        data[i] = delay_process(dl, data[i]);
    }
}

void delay_set_time(delay_line_t *dl, uint32_t samples) {
    dl->delay_samples = samples < DELAY_MAX_SAMPLES ? samples : DELAY_MAX_SAMPLES;
}

void delay_set_feedback(delay_line_t *dl, float fb) {
    dl->feedback = fb < 0.0f ? 0.0f : (fb > 0.99f ? 0.99f : fb);
}

void delay_clear(delay_line_t *dl) {
    uint32_t i;
    for (i = 0; i < DELAY_MAX_SAMPLES; i++) {
        dl->buffer[i] = 0.0f;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C411: Delay line should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C411: Output should not be empty");
    assert!(
        code.contains("fn delay_init"),
        "C411: Should contain delay_init function"
    );
    assert!(
        code.contains("fn delay_process"),
        "C411: Should contain delay_process function"
    );
}

#[test]
fn c412_noise_gate_with_hysteresis() {
    let c_code = r#"
enum gate_state {
    GATE_CLOSED = 0,
    GATE_OPENING,
    GATE_OPEN,
    GATE_CLOSING
};

typedef struct {
    enum gate_state state;
    float threshold_open;
    float threshold_close;
    float attack_coeff;
    float release_coeff;
    float env;
    float gain;
} noise_gate_t;

void gate_init(noise_gate_t *g, float sample_rate) {
    g->state = GATE_CLOSED;
    g->threshold_open = 0.1f;
    g->threshold_close = 0.05f;
    g->attack_coeff = 1.0f - (1.0f / (0.001f * sample_rate));
    g->release_coeff = 1.0f - (1.0f / (0.05f * sample_rate));
    g->env = 0.0f;
    g->gain = 0.0f;
}

void gate_set_thresholds(noise_gate_t *g, float open_thresh, float close_thresh) {
    g->threshold_open = open_thresh;
    g->threshold_close = close_thresh;
}

float gate_process(noise_gate_t *g, float input) {
    float abs_val = input < 0.0f ? -input : input;
    if (abs_val > g->env) {
        g->env = g->attack_coeff * g->env + (1.0f - g->attack_coeff) * abs_val;
    } else {
        g->env = g->release_coeff * g->env + (1.0f - g->release_coeff) * abs_val;
    }
    if (g->state == GATE_CLOSED || g->state == GATE_CLOSING) {
        if (g->env > g->threshold_open) {
            g->state = GATE_OPENING;
        }
    }
    if (g->state == GATE_OPEN || g->state == GATE_OPENING) {
        if (g->env < g->threshold_close) {
            g->state = GATE_CLOSING;
        }
    }
    if (g->state == GATE_OPENING) {
        g->gain += 0.01f;
        if (g->gain >= 1.0f) {
            g->gain = 1.0f;
            g->state = GATE_OPEN;
        }
    } else if (g->state == GATE_CLOSING) {
        g->gain -= 0.001f;
        if (g->gain <= 0.0f) {
            g->gain = 0.0f;
            g->state = GATE_CLOSED;
        }
    }
    return input * g->gain;
}

void gate_process_block(noise_gate_t *g, float *data, int len) {
    int i;
    for (i = 0; i < len; i++) {
        data[i] = gate_process(g, data[i]);
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C412: Noise gate with hysteresis should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C412: Output should not be empty");
    assert!(
        code.contains("fn gate_init"),
        "C412: Should contain gate_init function"
    );
    assert!(
        code.contains("fn gate_process"),
        "C412: Should contain gate_process function"
    );
}

#[test]
fn c413_parametric_eq_single_band() {
    let c_code = r#"
typedef struct {
    float b0;
    float b1;
    float b2;
    float a1;
    float a2;
    float z1;
    float z2;
} eq_band_t;

void eq_band_init(eq_band_t *eq) {
    eq->b0 = 1.0f;
    eq->b1 = 0.0f;
    eq->b2 = 0.0f;
    eq->a1 = 0.0f;
    eq->a2 = 0.0f;
    eq->z1 = 0.0f;
    eq->z2 = 0.0f;
}

void eq_band_set_peaking(eq_band_t *eq, float freq, float gain_db, float q,
                         float sample_rate) {
    float A = 1.0f + gain_db / 40.0f;
    float w0 = 2.0f * 3.14159265f * freq / sample_rate;
    float cos_w0 = 1.0f - w0 * w0 / 2.0f;
    float alpha = w0 / (2.0f * q);
    float a0_inv = 1.0f / (1.0f + alpha / A);
    eq->b0 = (1.0f + alpha * A) * a0_inv;
    eq->b1 = (-2.0f * cos_w0) * a0_inv;
    eq->b2 = (1.0f - alpha * A) * a0_inv;
    eq->a1 = eq->b1;
    eq->a2 = (1.0f - alpha / A) * a0_inv;
}

float eq_band_process(eq_band_t *eq, float input) {
    float output = eq->b0 * input + eq->z1;
    eq->z1 = eq->b1 * input - eq->a1 * output + eq->z2;
    eq->z2 = eq->b2 * input - eq->a2 * output;
    return output;
}

void eq_band_process_block(eq_band_t *eq, float *data, int len) {
    int i;
    for (i = 0; i < len; i++) {
        data[i] = eq_band_process(eq, data[i]);
    }
}

void eq_band_reset(eq_band_t *eq) {
    eq->z1 = 0.0f;
    eq->z2 = 0.0f;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C413: Parametric EQ single band should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C413: Output should not be empty");
    assert!(
        code.contains("fn eq_band_init"),
        "C413: Should contain eq_band_init function"
    );
    assert!(
        code.contains("fn eq_band_process"),
        "C413: Should contain eq_band_process function"
    );
    assert!(
        code.contains("fn eq_band_set_peaking"),
        "C413: Should contain eq_band_set_peaking function"
    );
}

#[test]
fn c414_crossfade_interpolation_between_buffers() {
    let c_code = r#"
void crossfade_linear(const float *buf_a, const float *buf_b,
                      float *output, int len, float mix) {
    int i;
    float a_gain = 1.0f - mix;
    float b_gain = mix;
    for (i = 0; i < len; i++) {
        output[i] = buf_a[i] * a_gain + buf_b[i] * b_gain;
    }
}

void crossfade_equal_power(const float *buf_a, const float *buf_b,
                           float *output, int len, float mix) {
    int i;
    float a_gain = 1.0f - mix * mix;
    float b_gain = mix * (2.0f - mix);
    for (i = 0; i < len; i++) {
        output[i] = buf_a[i] * a_gain + buf_b[i] * b_gain;
    }
}

void crossfade_block(const float *buf_a, const float *buf_b,
                     float *output, int len) {
    int i;
    for (i = 0; i < len; i++) {
        float t = (float)i / (float)(len - 1);
        float a_gain = 1.0f - t;
        float b_gain = t;
        output[i] = buf_a[i] * a_gain + buf_b[i] * b_gain;
    }
}

void buffer_copy(float *dst, const float *src, int len) {
    int i;
    for (i = 0; i < len; i++) {
        dst[i] = src[i];
    }
}

void buffer_scale(float *data, int len, float scale) {
    int i;
    for (i = 0; i < len; i++) {
        data[i] *= scale;
    }
}

void buffer_add(float *dst, const float *src, int len) {
    int i;
    for (i = 0; i < len; i++) {
        dst[i] += src[i];
    }
}

float buffer_sum_squares(const float *data, int len) {
    float sum = 0.0f;
    int i;
    for (i = 0; i < len; i++) {
        sum += data[i] * data[i];
    }
    return sum;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C414: Crossfade/interpolation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C414: Output should not be empty");
    assert!(
        code.contains("fn crossfade_linear"),
        "C414: Should contain crossfade_linear function"
    );
    assert!(
        code.contains("fn crossfade_equal_power"),
        "C414: Should contain crossfade_equal_power function"
    );
    assert!(
        code.contains("fn buffer_copy"),
        "C414: Should contain buffer_copy function"
    );
}

#[test]
fn c415_midi_note_to_frequency() {
    let c_code = r#"
float midi_note_to_freq(int note) {
    float base = 440.0f;
    float diff = (float)(note - 69);
    float semitone_ratio = 1.05946309f;
    float freq = base;
    int i;
    if (diff > 0) {
        for (i = 0; i < (int)diff; i++) {
            freq *= semitone_ratio;
        }
    } else if (diff < 0) {
        for (i = 0; i < (int)(-diff); i++) {
            freq /= semitone_ratio;
        }
    }
    return freq;
}

int freq_to_midi_note(float freq) {
    float ratio = freq / 440.0f;
    int semitones = 0;
    if (ratio >= 1.0f) {
        while (ratio > 1.05946309f) {
            ratio /= 1.05946309f;
            semitones++;
        }
    } else {
        while (ratio < 1.0f / 1.05946309f) {
            ratio *= 1.05946309f;
            semitones--;
        }
    }
    return 69 + semitones;
}

int midi_note_to_octave(int note) {
    return (note / 12) - 1;
}

int midi_note_to_semitone(int note) {
    return note % 12;
}

float midi_velocity_to_gain(int velocity) {
    if (velocity <= 0) return 0.0f;
    if (velocity >= 127) return 1.0f;
    return (float)velocity / 127.0f;
}

int midi_clamp(int value, int min_val, int max_val) {
    if (value < min_val) return min_val;
    if (value > max_val) return max_val;
    return value;
}

float midi_pitch_bend_to_semitones(int bend, int range) {
    float normalized = (float)(bend - 8192) / 8192.0f;
    return normalized * (float)range;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C415: MIDI note to frequency converter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C415: Output should not be empty");
    assert!(
        code.contains("fn midi_note_to_freq"),
        "C415: Should contain midi_note_to_freq function"
    );
    assert!(
        code.contains("fn freq_to_midi_note"),
        "C415: Should contain freq_to_midi_note function"
    );
    assert!(
        code.contains("fn midi_velocity_to_gain"),
        "C415: Should contain midi_velocity_to_gain function"
    );
}

// ============================================================================
// C416-C420: Analysis (Window Functions, Pitch Detect, Dithering, Convolution, Spectral)
// ============================================================================

#[test]
fn c416_window_function_generator() {
    let c_code = r#"
void window_hamming(float *window, int len) {
    int i;
    float pi2 = 2.0f * 3.14159265f;
    for (i = 0; i < len; i++) {
        float t = pi2 * (float)i / (float)(len - 1);
        window[i] = 0.54f - 0.46f * (1.0f - t * t / 2.0f);
    }
}

void window_hann(float *window, int len) {
    int i;
    float pi2 = 2.0f * 3.14159265f;
    for (i = 0; i < len; i++) {
        float t = pi2 * (float)i / (float)(len - 1);
        window[i] = 0.5f * (1.0f - (1.0f - t * t / 2.0f));
    }
}

void window_blackman(float *window, int len) {
    int i;
    float pi2 = 2.0f * 3.14159265f;
    float pi4 = 4.0f * 3.14159265f;
    for (i = 0; i < len; i++) {
        float t1 = pi2 * (float)i / (float)(len - 1);
        float t2 = pi4 * (float)i / (float)(len - 1);
        float cos1 = 1.0f - t1 * t1 / 2.0f;
        float cos2 = 1.0f - t2 * t2 / 2.0f;
        window[i] = 0.42f - 0.5f * cos1 + 0.08f * cos2;
    }
}

void window_apply(float *data, const float *window, int len) {
    int i;
    for (i = 0; i < len; i++) {
        data[i] *= window[i];
    }
}

float window_coherent_gain(const float *window, int len) {
    float sum = 0.0f;
    int i;
    for (i = 0; i < len; i++) {
        sum += window[i];
    }
    return sum / (float)len;
}

void window_rectangular(float *window, int len) {
    int i;
    for (i = 0; i < len; i++) {
        window[i] = 1.0f;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C416: Window function generator should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C416: Output should not be empty");
    assert!(
        code.contains("fn window_hamming"),
        "C416: Should contain window_hamming function"
    );
    assert!(
        code.contains("fn window_hann"),
        "C416: Should contain window_hann function"
    );
    assert!(
        code.contains("fn window_apply"),
        "C416: Should contain window_apply function"
    );
}

#[test]
fn c417_pitch_detection_zero_crossing_rate() {
    let c_code = r#"
int count_zero_crossings(const float *data, int len) {
    int count = 0;
    int i;
    for (i = 1; i < len; i++) {
        if ((data[i - 1] >= 0.0f && data[i] < 0.0f) ||
            (data[i - 1] < 0.0f && data[i] >= 0.0f)) {
            count++;
        }
    }
    return count;
}

float zero_crossing_rate(const float *data, int len) {
    if (len <= 1) return 0.0f;
    int crossings = count_zero_crossings(data, len);
    return (float)crossings / (float)(len - 1);
}

float estimate_pitch_zcr(const float *data, int len, float sample_rate) {
    int crossings = count_zero_crossings(data, len);
    if (crossings < 2) return 0.0f;
    float period_samples = 2.0f * (float)len / (float)crossings;
    return sample_rate / period_samples;
}

int find_first_positive_crossing(const float *data, int len) {
    int i;
    for (i = 1; i < len; i++) {
        if (data[i - 1] < 0.0f && data[i] >= 0.0f) {
            return i;
        }
    }
    return -1;
}

float signal_energy(const float *data, int len) {
    float energy = 0.0f;
    int i;
    for (i = 0; i < len; i++) {
        energy += data[i] * data[i];
    }
    return energy / (float)len;
}

int is_voiced(const float *data, int len, float energy_threshold) {
    float energy = signal_energy(data, len);
    return energy > energy_threshold;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C417: Pitch detection via zero-crossing rate should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C417: Output should not be empty");
    assert!(
        code.contains("fn count_zero_crossings"),
        "C417: Should contain count_zero_crossings function"
    );
    assert!(
        code.contains("fn estimate_pitch_zcr"),
        "C417: Should contain estimate_pitch_zcr function"
    );
}

#[test]
fn c418_audio_dithering_tpdf() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t state;
    float amplitude;
} dither_t;

void dither_init(dither_t *d, uint32_t seed, int target_bits) {
    d->state = seed;
    d->amplitude = 1.0f;
    if (target_bits > 0 && target_bits < 32) {
        int i;
        d->amplitude = 1.0f;
        for (i = 0; i < target_bits; i++) {
            d->amplitude *= 0.5f;
        }
    }
}

uint32_t dither_lcg(dither_t *d) {
    d->state = d->state * 1664525U + 1013904223U;
    return d->state;
}

float dither_uniform(dither_t *d) {
    uint32_t r = dither_lcg(d);
    return (float)(r >> 8) / 16777216.0f - 0.5f;
}

float dither_tpdf(dither_t *d) {
    float a = dither_uniform(d);
    float b = dither_uniform(d);
    return (a + b) * d->amplitude;
}

void dither_process(dither_t *d, float *data, int len) {
    int i;
    for (i = 0; i < len; i++) {
        data[i] += dither_tpdf(d);
    }
}

float quantize_to_bits(float sample, int bits) {
    float scale = 1.0f;
    int i;
    for (i = 0; i < bits - 1; i++) {
        scale *= 2.0f;
    }
    float quantized = (float)((int)(sample * scale)) / scale;
    return quantized;
}

void dither_and_quantize(dither_t *d, float *data, int len, int bits) {
    int i;
    for (i = 0; i < len; i++) {
        data[i] += dither_tpdf(d);
        data[i] = quantize_to_bits(data[i], bits);
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C418: Audio dithering TPDF should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C418: Output should not be empty");
    assert!(
        code.contains("fn dither_init"),
        "C418: Should contain dither_init function"
    );
    assert!(
        code.contains("fn dither_tpdf"),
        "C418: Should contain dither_tpdf function"
    );
    assert!(
        code.contains("fn dither_process"),
        "C418: Should contain dither_process function"
    );
}

#[test]
fn c419_convolution_reverb_time_domain() {
    let c_code = r#"
void convolve_direct(const float *signal, int sig_len,
                     const float *impulse, int imp_len,
                     float *output) {
    int n;
    int k;
    int out_len = sig_len + imp_len - 1;
    for (n = 0; n < out_len; n++) {
        output[n] = 0.0f;
    }
    for (n = 0; n < sig_len; n++) {
        for (k = 0; k < imp_len; k++) {
            output[n + k] += signal[n] * impulse[k];
        }
    }
}

void convolve_overlap_add(const float *signal, int sig_len,
                          const float *impulse, int imp_len,
                          float *output, int block_size) {
    int pos;
    int i;
    int k;
    int out_len = sig_len + imp_len - 1;
    for (i = 0; i < out_len; i++) {
        output[i] = 0.0f;
    }
    for (pos = 0; pos < sig_len; pos += block_size) {
        int chunk = sig_len - pos;
        if (chunk > block_size) chunk = block_size;
        for (i = 0; i < chunk; i++) {
            for (k = 0; k < imp_len; k++) {
                output[pos + i + k] += signal[pos + i] * impulse[k];
            }
        }
    }
}

void normalize_impulse(float *impulse, int len) {
    float sum = 0.0f;
    int i;
    for (i = 0; i < len; i++) {
        float abs_val = impulse[i] < 0.0f ? -impulse[i] : impulse[i];
        sum += abs_val;
    }
    if (sum > 0.0001f) {
        float inv_sum = 1.0f / sum;
        for (i = 0; i < len; i++) {
            impulse[i] *= inv_sum;
        }
    }
}

float compute_rt60_samples(const float *impulse, int len) {
    float max_val = 0.0f;
    float threshold;
    int i;
    for (i = 0; i < len; i++) {
        float abs_val = impulse[i] < 0.0f ? -impulse[i] : impulse[i];
        if (abs_val > max_val) max_val = abs_val;
    }
    threshold = max_val * 0.001f;
    for (i = len - 1; i >= 0; i--) {
        float abs_val = impulse[i] < 0.0f ? -impulse[i] : impulse[i];
        if (abs_val > threshold) {
            return (float)i;
        }
    }
    return 0.0f;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C419: Convolution reverb should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C419: Output should not be empty");
    assert!(
        code.contains("fn convolve_direct"),
        "C419: Should contain convolve_direct function"
    );
    assert!(
        code.contains("fn normalize_impulse"),
        "C419: Should contain normalize_impulse function"
    );
}

#[test]
fn c420_spectral_magnitude_phase() {
    let c_code = r#"
typedef struct {
    float re;
    float im;
} spectral_bin_t;

float spectral_magnitude(spectral_bin_t bin) {
    return bin.re * bin.re + bin.im * bin.im;
}

float spectral_phase(spectral_bin_t bin) {
    if (bin.re == 0.0f && bin.im == 0.0f) return 0.0f;
    return bin.im / (bin.re + 0.00001f);
}

void compute_magnitude_spectrum(const spectral_bin_t *bins, float *magnitudes, int len) {
    int i;
    for (i = 0; i < len; i++) {
        magnitudes[i] = spectral_magnitude(bins[i]);
    }
}

void compute_phase_spectrum(const spectral_bin_t *bins, float *phases, int len) {
    int i;
    for (i = 0; i < len; i++) {
        phases[i] = spectral_phase(bins[i]);
    }
}

void compute_power_spectrum(const spectral_bin_t *bins, float *power, int len) {
    int i;
    for (i = 0; i < len; i++) {
        float mag = spectral_magnitude(bins[i]);
        power[i] = mag;
    }
}

float spectral_centroid(const float *magnitudes, int len, float sample_rate) {
    float num = 0.0f;
    float den = 0.0f;
    int i;
    float bin_freq;
    for (i = 0; i < len; i++) {
        bin_freq = (float)i * sample_rate / (float)(2 * len);
        num += bin_freq * magnitudes[i];
        den += magnitudes[i];
    }
    if (den < 0.0001f) return 0.0f;
    return num / den;
}

float spectral_flatness(const float *magnitudes, int len) {
    float geo_sum = 0.0f;
    float arith_sum = 0.0f;
    int i;
    int count = 0;
    for (i = 0; i < len; i++) {
        if (magnitudes[i] > 0.0001f) {
            geo_sum += magnitudes[i];
            arith_sum += magnitudes[i];
            count++;
        }
    }
    if (count == 0) return 0.0f;
    return geo_sum / (arith_sum + 0.0001f);
}

spectral_bin_t spectral_bin_from_polar(float magnitude, float phase) {
    spectral_bin_t bin;
    bin.re = magnitude * (1.0f - phase * phase / 2.0f);
    bin.im = magnitude * phase;
    return bin;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C420: Spectral analysis should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C420: Output should not be empty");
    assert!(
        code.contains("fn spectral_magnitude"),
        "C420: Should contain spectral_magnitude function"
    );
    assert!(
        code.contains("fn spectral_centroid"),
        "C420: Should contain spectral_centroid function"
    );
    assert!(
        code.contains("fn spectral_flatness"),
        "C420: Should contain spectral_flatness function"
    );
}

// ============================================================================
// C421-C425: Polyphony, Envelope Follower, Format Conversion, Click Detect, Beat Detect
// ============================================================================

#[test]
fn c421_polyphonic_voice_allocator() {
    let c_code = r#"
#define MAX_VOICES 16

enum voice_state {
    VOICE_FREE = 0,
    VOICE_ACTIVE,
    VOICE_RELEASING
};

typedef struct {
    enum voice_state state;
    int note;
    int velocity;
    int age;
} voice_t;

typedef struct {
    voice_t voices[MAX_VOICES];
    int num_voices;
    int next_age;
} voice_allocator_t;

void allocator_init(voice_allocator_t *va, int num_voices) {
    int i;
    va->num_voices = num_voices < MAX_VOICES ? num_voices : MAX_VOICES;
    va->next_age = 0;
    for (i = 0; i < MAX_VOICES; i++) {
        va->voices[i].state = VOICE_FREE;
        va->voices[i].note = -1;
        va->voices[i].velocity = 0;
        va->voices[i].age = 0;
    }
}

int allocator_find_free(const voice_allocator_t *va) {
    int i;
    for (i = 0; i < va->num_voices; i++) {
        if (va->voices[i].state == VOICE_FREE) {
            return i;
        }
    }
    return -1;
}

int allocator_find_oldest(const voice_allocator_t *va) {
    int oldest = 0;
    int oldest_age = va->voices[0].age;
    int i;
    for (i = 1; i < va->num_voices; i++) {
        if (va->voices[i].age < oldest_age) {
            oldest_age = va->voices[i].age;
            oldest = i;
        }
    }
    return oldest;
}

int allocator_note_on(voice_allocator_t *va, int note, int velocity) {
    int slot = allocator_find_free(va);
    if (slot < 0) {
        slot = allocator_find_oldest(va);
    }
    va->voices[slot].state = VOICE_ACTIVE;
    va->voices[slot].note = note;
    va->voices[slot].velocity = velocity;
    va->voices[slot].age = va->next_age++;
    return slot;
}

void allocator_note_off(voice_allocator_t *va, int note) {
    int i;
    for (i = 0; i < va->num_voices; i++) {
        if (va->voices[i].note == note && va->voices[i].state == VOICE_ACTIVE) {
            va->voices[i].state = VOICE_RELEASING;
        }
    }
}

void allocator_free_voice(voice_allocator_t *va, int slot) {
    if (slot >= 0 && slot < va->num_voices) {
        va->voices[slot].state = VOICE_FREE;
        va->voices[slot].note = -1;
    }
}

int allocator_active_count(const voice_allocator_t *va) {
    int count = 0;
    int i;
    for (i = 0; i < va->num_voices; i++) {
        if (va->voices[i].state != VOICE_FREE) {
            count++;
        }
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C421: Polyphonic voice allocator should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C421: Output should not be empty");
    assert!(
        code.contains("fn allocator_init"),
        "C421: Should contain allocator_init function"
    );
    assert!(
        code.contains("fn allocator_note_on"),
        "C421: Should contain allocator_note_on function"
    );
    assert!(
        code.contains("fn allocator_note_off"),
        "C421: Should contain allocator_note_off function"
    );
}

#[test]
fn c422_envelope_follower_peak_detection() {
    let c_code = r#"
typedef struct {
    float peak;
    float attack_coeff;
    float release_coeff;
    float smoothed;
} env_follower_t;

void env_follower_init(env_follower_t *ef, float attack_ms, float release_ms,
                       float sample_rate) {
    ef->peak = 0.0f;
    ef->smoothed = 0.0f;
    if (attack_ms > 0.0f) {
        ef->attack_coeff = 1.0f - (1.0f / (attack_ms * 0.001f * sample_rate));
    } else {
        ef->attack_coeff = 0.0f;
    }
    if (release_ms > 0.0f) {
        ef->release_coeff = 1.0f - (1.0f / (release_ms * 0.001f * sample_rate));
    } else {
        ef->release_coeff = 0.0f;
    }
}

float env_follower_process(env_follower_t *ef, float input) {
    float abs_val = input < 0.0f ? -input : input;
    if (abs_val > ef->peak) {
        ef->peak = ef->attack_coeff * ef->peak + (1.0f - ef->attack_coeff) * abs_val;
    } else {
        ef->peak = ef->release_coeff * ef->peak;
    }
    return ef->peak;
}

void env_follower_process_block(env_follower_t *ef, const float *input,
                                float *output, int len) {
    int i;
    for (i = 0; i < len; i++) {
        output[i] = env_follower_process(ef, input[i]);
    }
}

float env_follower_get_peak(const env_follower_t *ef) {
    return ef->peak;
}

void env_follower_reset(env_follower_t *ef) {
    ef->peak = 0.0f;
    ef->smoothed = 0.0f;
}

float env_follower_smooth(env_follower_t *ef, float input, float coeff) {
    ef->smoothed = coeff * ef->smoothed + (1.0f - coeff) * input;
    return ef->smoothed;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C422: Envelope follower should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C422: Output should not be empty");
    assert!(
        code.contains("fn env_follower_init"),
        "C422: Should contain env_follower_init function"
    );
    assert!(
        code.contains("fn env_follower_process"),
        "C422: Should contain env_follower_process function"
    );
}

#[test]
fn c423_audio_format_converter_int16_float() {
    let c_code = r#"
typedef short int16_t;
typedef unsigned short uint16_t;

void int16_to_float(const int16_t *input, float *output, int len) {
    int i;
    for (i = 0; i < len; i++) {
        output[i] = (float)input[i] / 32768.0f;
    }
}

void float_to_int16(const float *input, int16_t *output, int len) {
    int i;
    for (i = 0; i < len; i++) {
        float clamped = input[i];
        if (clamped > 1.0f) clamped = 1.0f;
        if (clamped < -1.0f) clamped = -1.0f;
        output[i] = (int16_t)(clamped * 32767.0f);
    }
}

void interleave_stereo(const float *left, const float *right,
                       float *interleaved, int frames) {
    int i;
    for (i = 0; i < frames; i++) {
        interleaved[i * 2] = left[i];
        interleaved[i * 2 + 1] = right[i];
    }
}

void deinterleave_stereo(const float *interleaved, float *left,
                         float *right, int frames) {
    int i;
    for (i = 0; i < frames; i++) {
        left[i] = interleaved[i * 2];
        right[i] = interleaved[i * 2 + 1];
    }
}

void mono_to_stereo(const float *mono, float *left, float *right, int frames) {
    int i;
    for (i = 0; i < frames; i++) {
        left[i] = mono[i];
        right[i] = mono[i];
    }
}

void stereo_to_mono(const float *left, const float *right,
                    float *mono, int frames) {
    int i;
    for (i = 0; i < frames; i++) {
        mono[i] = (left[i] + right[i]) * 0.5f;
    }
}

float sample_peak_db(const float *data, int len) {
    float peak = 0.0f;
    int i;
    for (i = 0; i < len; i++) {
        float abs_val = data[i] < 0.0f ? -data[i] : data[i];
        if (abs_val > peak) peak = abs_val;
    }
    if (peak < 0.0001f) return -80.0f;
    return 20.0f * (peak - 1.0f);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C423: Audio format converter should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C423: Output should not be empty");
    assert!(
        code.contains("fn int16_to_float"),
        "C423: Should contain int16_to_float function"
    );
    assert!(
        code.contains("fn float_to_int16"),
        "C423: Should contain float_to_int16 function"
    );
    assert!(
        code.contains("fn interleave_stereo"),
        "C423: Should contain interleave_stereo function"
    );
}

#[test]
fn c424_click_pop_detection() {
    let c_code = r#"
typedef struct {
    float prev_sample;
    float prev_delta;
    float threshold;
    int click_count;
    int consecutive_large;
    int min_consecutive;
} click_detector_t;

void click_detector_init(click_detector_t *cd, float threshold) {
    cd->prev_sample = 0.0f;
    cd->prev_delta = 0.0f;
    cd->threshold = threshold;
    cd->click_count = 0;
    cd->consecutive_large = 0;
    cd->min_consecutive = 3;
}

int click_detector_process(click_detector_t *cd, float sample) {
    float delta = sample - cd->prev_sample;
    float delta_delta = delta - cd->prev_delta;
    float abs_dd = delta_delta < 0.0f ? -delta_delta : delta_delta;
    int click_detected = 0;
    if (abs_dd > cd->threshold) {
        cd->consecutive_large++;
        if (cd->consecutive_large >= cd->min_consecutive) {
            click_detected = 1;
            cd->click_count++;
            cd->consecutive_large = 0;
        }
    } else {
        cd->consecutive_large = 0;
    }
    cd->prev_delta = delta;
    cd->prev_sample = sample;
    return click_detected;
}

void click_detector_scan(click_detector_t *cd, const float *data, int len,
                         int *click_positions, int max_clicks, int *num_found) {
    int i;
    int found = 0;
    for (i = 0; i < len; i++) {
        if (click_detector_process(cd, data[i])) {
            if (found < max_clicks) {
                click_positions[found] = i;
                found++;
            }
        }
    }
    *num_found = found;
}

int click_detector_get_count(const click_detector_t *cd) {
    return cd->click_count;
}

void click_detector_reset(click_detector_t *cd) {
    cd->prev_sample = 0.0f;
    cd->prev_delta = 0.0f;
    cd->click_count = 0;
    cd->consecutive_large = 0;
}

void click_repair_linear(float *data, int pos, int width) {
    int start = pos - width;
    int end = pos + width;
    float start_val;
    float end_val;
    int i;
    if (start < 0) start = 0;
    start_val = data[start];
    end_val = data[end];
    for (i = start; i <= end; i++) {
        float t = (float)(i - start) / (float)(end - start);
        data[i] = start_val + (end_val - start_val) * t;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C424: Click/pop detection should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C424: Output should not be empty");
    assert!(
        code.contains("fn click_detector_init"),
        "C424: Should contain click_detector_init function"
    );
    assert!(
        code.contains("fn click_detector_process"),
        "C424: Should contain click_detector_process function"
    );
    assert!(
        code.contains("fn click_repair_linear"),
        "C424: Should contain click_repair_linear function"
    );
}

#[test]
fn c425_beat_detection_energy_onset() {
    let c_code = r#"
#define BEAT_HISTORY 64

typedef struct {
    float energy_history[BEAT_HISTORY];
    int history_pos;
    int history_count;
    float threshold_multiplier;
    float min_interval_samples;
    float samples_since_last;
    int beat_count;
} beat_detector_t;

void beat_detector_init(beat_detector_t *bd, float threshold, float min_bpm,
                        float sample_rate) {
    int i;
    bd->history_pos = 0;
    bd->history_count = 0;
    bd->threshold_multiplier = threshold;
    bd->min_interval_samples = 60.0f * sample_rate / min_bpm;
    bd->samples_since_last = bd->min_interval_samples;
    bd->beat_count = 0;
    for (i = 0; i < BEAT_HISTORY; i++) {
        bd->energy_history[i] = 0.0f;
    }
}

float beat_compute_energy(const float *block, int block_size) {
    float energy = 0.0f;
    int i;
    for (i = 0; i < block_size; i++) {
        energy += block[i] * block[i];
    }
    return energy / (float)block_size;
}

float beat_average_energy(const beat_detector_t *bd) {
    float sum = 0.0f;
    int i;
    int count = bd->history_count < BEAT_HISTORY ? bd->history_count : BEAT_HISTORY;
    if (count == 0) return 0.0f;
    for (i = 0; i < count; i++) {
        sum += bd->energy_history[i];
    }
    return sum / (float)count;
}

int beat_detect_block(beat_detector_t *bd, const float *block, int block_size) {
    float energy = beat_compute_energy(block, block_size);
    float avg = beat_average_energy(bd);
    int is_beat = 0;
    bd->energy_history[bd->history_pos] = energy;
    bd->history_pos = (bd->history_pos + 1) % BEAT_HISTORY;
    if (bd->history_count < BEAT_HISTORY) {
        bd->history_count++;
    }
    bd->samples_since_last += (float)block_size;
    if (energy > avg * bd->threshold_multiplier &&
        bd->samples_since_last >= bd->min_interval_samples) {
        is_beat = 1;
        bd->beat_count++;
        bd->samples_since_last = 0.0f;
    }
    return is_beat;
}

float beat_estimate_bpm(const beat_detector_t *bd, float sample_rate) {
    if (bd->beat_count < 2) return 0.0f;
    float total = bd->samples_since_last;
    if (total <= 0.0f) return 0.0f;
    return 60.0f * sample_rate / total;
}

int beat_get_count(const beat_detector_t *bd) {
    return bd->beat_count;
}

void beat_detector_reset(beat_detector_t *bd) {
    int i;
    bd->history_pos = 0;
    bd->history_count = 0;
    bd->samples_since_last = bd->min_interval_samples;
    bd->beat_count = 0;
    for (i = 0; i < BEAT_HISTORY; i++) {
        bd->energy_history[i] = 0.0f;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C425: Beat detection should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C425: Output should not be empty");
    assert!(
        code.contains("fn beat_detector_init"),
        "C425: Should contain beat_detector_init function"
    );
    assert!(
        code.contains("fn beat_detect_block"),
        "C425: Should contain beat_detect_block function"
    );
    assert!(
        code.contains("fn beat_compute_energy"),
        "C425: Should contain beat_compute_energy function"
    );
}
