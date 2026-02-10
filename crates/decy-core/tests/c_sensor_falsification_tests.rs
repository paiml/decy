//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1426-C1450: Sensor/IoT Processing patterns -- signal filters, sensor fusion,
//! data acquisition, protocol handling, and IoT primitives.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world sensor and IoT patterns commonly
//! found in embedded sensor systems, IoT gateways, data acquisition
//! systems, and industrial monitoring -- all expressed as valid C99.
//!
//! Organization:
//! - C1426-C1430: Signal filters (moving average, exponential, median, IIR, Kalman)
//! - C1431-C1435: Sensor fusion (complementary, accel+gyro, mag cal, quaternion, weighting)
//! - C1436-C1440: Data acquisition (ADC, ring buffer, threshold, peak, zero-crossing)
//! - C1441-C1445: Protocol handling (I2C, SPI, MODBUS, CAN, 1-Wire)
//! - C1446-C1450: IoT primitives (telemetry, registry, watchdog, power FSM, OTA)

// ============================================================================
// C1426-C1430: Signal Filters
// ============================================================================

#[test]
fn c1426_moving_average_filter() {
    let c_code = r#"
typedef struct { int buf[16]; int idx; int count; int sum; } sens_avg_t;

void sens_avg_init(sens_avg_t *a) {
    a->idx = 0; a->count = 0; a->sum = 0;
}

int sens_avg_add(sens_avg_t *a, int val) {
    if (a->count >= 16) a->sum -= a->buf[a->idx];
    else a->count++;
    a->buf[a->idx] = val;
    a->sum += val;
    a->idx = (a->idx + 1) % 16;
    return a->sum / a->count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1426 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1426: Output should not be empty");
    assert!(
        code.contains("sens_avg_init"),
        "C1426: Should contain sens_avg_init"
    );
}

#[test]
fn c1427_exponential_smoothing() {
    let c_code = r#"
typedef struct { int value; int alpha_num; int alpha_den; int initialized; } sens_ema_t;

void sens_ema_init(sens_ema_t *e, int alpha_num, int alpha_den) {
    e->value = 0; e->alpha_num = alpha_num;
    e->alpha_den = alpha_den; e->initialized = 0;
}

int sens_ema_update(sens_ema_t *e, int sample) {
    if (!e->initialized) { e->value = sample * e->alpha_den; e->initialized = 1; }
    else { e->value = e->alpha_num * sample + (e->alpha_den - e->alpha_num) * (e->value / e->alpha_den); }
    return e->value / e->alpha_den;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1427 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1427: Output should not be empty");
    assert!(
        code.contains("sens_ema_update"),
        "C1427: Should contain sens_ema_update"
    );
}

#[test]
fn c1428_median_filter() {
    let c_code = r#"
typedef struct { int window[5]; int count; } sens_median_t;

void sens_median_init(sens_median_t *m) { m->count = 0; }

static void sens_sort3(int *a, int *b, int *c) {
    int t;
    if (*a > *b) { t = *a; *a = *b; *b = t; }
    if (*b > *c) { t = *b; *b = *c; *c = t; }
    if (*a > *b) { t = *a; *a = *b; *b = t; }
}

int sens_median_add(sens_median_t *m, int val) {
    int i;
    if (m->count < 5) { m->window[m->count++] = val; }
    else { for (i = 0; i < 4; i++) m->window[i] = m->window[i+1]; m->window[4] = val; }
    if (m->count < 3) return val;
    int a = m->window[m->count-3], b = m->window[m->count-2], c = m->window[m->count-1];
    sens_sort3(&a, &b, &c);
    return b;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1428 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1428: Output should not be empty");
    assert!(
        code.contains("sens_median_add"),
        "C1428: Should contain sens_median_add"
    );
}

#[test]
fn c1429_lowpass_iir_filter() {
    let c_code = r#"
typedef struct { int y_prev; int coeff_a; int coeff_b; int scale; } sens_iir_t;

void sens_iir_init(sens_iir_t *f, int a, int b, int scale) {
    f->y_prev = 0; f->coeff_a = a; f->coeff_b = b; f->scale = scale;
}

int sens_iir_apply(sens_iir_t *f, int x) {
    int y = (f->coeff_a * x + f->coeff_b * f->y_prev) / f->scale;
    f->y_prev = y;
    return y;
}

void sens_iir_reset(sens_iir_t *f) { f->y_prev = 0; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1429 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1429: Output should not be empty");
    assert!(
        code.contains("sens_iir_apply"),
        "C1429: Should contain sens_iir_apply"
    );
}

#[test]
fn c1430_kalman_filter_1d() {
    let c_code = r#"
typedef struct { int x_est; int p_est; int q; int r; int k_num; int k_den; } sens_kalman_t;

void sens_kalman_init(sens_kalman_t *kf, int init_x, int init_p, int q, int r) {
    kf->x_est = init_x; kf->p_est = init_p;
    kf->q = q; kf->r = r; kf->k_num = 0; kf->k_den = 1;
}

int sens_kalman_update(sens_kalman_t *kf, int measurement) {
    int p_pred = kf->p_est + kf->q;
    kf->k_den = p_pred + kf->r;
    if (kf->k_den == 0) kf->k_den = 1;
    kf->k_num = p_pred;
    kf->x_est = kf->x_est + kf->k_num * (measurement - kf->x_est) / kf->k_den;
    kf->p_est = (kf->r * p_pred) / kf->k_den;
    return kf->x_est;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1430 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1430: Output should not be empty");
    assert!(
        code.contains("sens_kalman_update"),
        "C1430: Should contain sens_kalman_update"
    );
}

// ============================================================================
// C1431-C1435: Sensor Fusion
// ============================================================================

#[test]
fn c1431_complementary_filter() {
    let c_code = r#"
typedef struct { int angle; int alpha; int scale; } sens_comp_t;

void sens_comp_init(sens_comp_t *cf, int alpha, int scale) {
    cf->angle = 0; cf->alpha = alpha; cf->scale = scale;
}

int sens_comp_update(sens_comp_t *cf, int gyro_rate, int accel_angle, int dt_ms) {
    int gyro_angle = cf->angle + gyro_rate * dt_ms / 1000;
    cf->angle = (cf->alpha * gyro_angle + (cf->scale - cf->alpha) * accel_angle) / cf->scale;
    return cf->angle;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1431 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1431: Output should not be empty");
    assert!(
        code.contains("sens_comp_update"),
        "C1431: Should contain sens_comp_update"
    );
}

#[test]
fn c1432_accel_gyro_fusion() {
    let c_code = r#"
typedef struct { int pitch; int roll; int dt_us; } sens_imu_t;

void sens_imu_init(sens_imu_t *imu) { imu->pitch = 0; imu->roll = 0; imu->dt_us = 10000; }

void sens_imu_fuse(sens_imu_t *imu, int ax, int ay, int az, int gx, int gy) {
    int accel_pitch = 0, accel_roll = 0;
    if (az != 0) { accel_pitch = (ax * 1000) / az; accel_roll = (ay * 1000) / az; }
    int gyro_pitch = imu->pitch + gx * imu->dt_us / 1000000;
    int gyro_roll = imu->roll + gy * imu->dt_us / 1000000;
    imu->pitch = (98 * gyro_pitch + 2 * accel_pitch) / 100;
    imu->roll = (98 * gyro_roll + 2 * accel_roll) / 100;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1432 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1432: Output should not be empty");
    assert!(
        code.contains("sens_imu_fuse"),
        "C1432: Should contain sens_imu_fuse"
    );
}

#[test]
fn c1433_magnetometer_calibration() {
    let c_code = r#"
typedef struct { int offset_x; int offset_y; int offset_z; int scale; } sens_mag_cal_t;

void sens_mag_cal_init(sens_mag_cal_t *cal) {
    cal->offset_x = 0; cal->offset_y = 0; cal->offset_z = 0; cal->scale = 1000;
}

void sens_mag_calibrate(sens_mag_cal_t *cal, int min_x, int max_x, int min_y, int max_y, int min_z, int max_z) {
    cal->offset_x = (min_x + max_x) / 2;
    cal->offset_y = (min_y + max_y) / 2;
    cal->offset_z = (min_z + max_z) / 2;
    int range_x = max_x - min_x; if (range_x == 0) range_x = 1;
    cal->scale = range_x;
}

int sens_mag_apply(sens_mag_cal_t *cal, int raw_x) {
    return (raw_x - cal->offset_x) * 1000 / cal->scale;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1433 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1433: Output should not be empty");
    assert!(
        code.contains("sens_mag_calibrate"),
        "C1433: Should contain sens_mag_calibrate"
    );
}

#[test]
fn c1434_quaternion_rotation() {
    let c_code = r#"
typedef struct { int w; int x; int y; int z; } sens_quat_t;

void sens_quat_identity(sens_quat_t *q) { q->w = 1000; q->x = 0; q->y = 0; q->z = 0; }

void sens_quat_multiply(sens_quat_t *r, const sens_quat_t *a, const sens_quat_t *b) {
    r->w = (a->w * b->w - a->x * b->x - a->y * b->y - a->z * b->z) / 1000;
    r->x = (a->w * b->x + a->x * b->w + a->y * b->z - a->z * b->y) / 1000;
    r->y = (a->w * b->y - a->x * b->z + a->y * b->w + a->z * b->x) / 1000;
    r->z = (a->w * b->z + a->x * b->y - a->y * b->x + a->z * b->w) / 1000;
}

int sens_quat_norm_sq(const sens_quat_t *q) {
    return (q->w * q->w + q->x * q->x + q->y * q->y + q->z * q->z) / 1000;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1434 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1434: Output should not be empty");
    assert!(
        code.contains("sens_quat_multiply"),
        "C1434: Should contain sens_quat_multiply"
    );
}

#[test]
fn c1435_sensor_weighting() {
    let c_code = r#"
typedef struct { int value; int weight; int variance; } sens_source_t;

int sens_weighted_avg(const sens_source_t *sources, int n) {
    int wsum = 0, wtotal = 0, i;
    for (i = 0; i < n; i++) {
        if (sources[i].variance > 0) {
            int w = 1000 / sources[i].variance;
            wsum += sources[i].value * w;
            wtotal += w;
        }
    }
    if (wtotal == 0) return 0;
    return wsum / wtotal;
}

int sens_best_source(const sens_source_t *sources, int n) {
    int best = 0, min_var = 2147483647, i;
    for (i = 0; i < n; i++) {
        if (sources[i].variance < min_var) { min_var = sources[i].variance; best = i; }
    }
    return best;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1435 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1435: Output should not be empty");
    assert!(
        code.contains("sens_weighted_avg"),
        "C1435: Should contain sens_weighted_avg"
    );
}

// ============================================================================
// C1436-C1440: Data Acquisition
// ============================================================================

#[test]
fn c1436_adc_converter() {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;

typedef struct { uint16_t raw; int vref_mv; int bits; } sens_adc_t;

void sens_adc_init(sens_adc_t *adc, int vref_mv, int bits) {
    adc->raw = 0; adc->vref_mv = vref_mv; adc->bits = bits;
}

int sens_adc_to_mv(const sens_adc_t *adc) {
    uint32_t max_val = (1u << adc->bits) - 1;
    if (max_val == 0) return 0;
    return (int)((uint32_t)adc->raw * (uint32_t)adc->vref_mv / max_val);
}

int sens_adc_to_temp_c(const sens_adc_t *adc) {
    int mv = sens_adc_to_mv(adc);
    return (mv - 500) / 10;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1436 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1436: Output should not be empty");
    assert!(
        code.contains("sens_adc_to_mv"),
        "C1436: Should contain sens_adc_to_mv"
    );
}

#[test]
fn c1437_sample_ring_buffer() {
    let c_code = r#"
typedef struct { int samples[256]; int head; int tail; int full; } sens_ring_t;

void sens_ring_init(sens_ring_t *r) { r->head = 0; r->tail = 0; r->full = 0; }

int sens_ring_push(sens_ring_t *r, int val) {
    r->samples[r->head] = val;
    if (r->full) r->tail = (r->tail + 1) % 256;
    r->head = (r->head + 1) % 256;
    r->full = (r->head == r->tail);
    return 1;
}

int sens_ring_pop(sens_ring_t *r, int *val) {
    if (r->head == r->tail && !r->full) return 0;
    *val = r->samples[r->tail];
    r->tail = (r->tail + 1) % 256;
    r->full = 0;
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1437 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1437: Output should not be empty");
    assert!(
        code.contains("sens_ring_push"),
        "C1437: Should contain sens_ring_push"
    );
}

#[test]
fn c1438_threshold_detector() {
    let c_code = r#"
typedef struct { int high_thresh; int low_thresh; int state; int debounce; int count; } sens_thresh_t;

void sens_thresh_init(sens_thresh_t *t, int high, int low, int debounce) {
    t->high_thresh = high; t->low_thresh = low;
    t->state = 0; t->debounce = debounce; t->count = 0;
}

int sens_thresh_check(sens_thresh_t *t, int value) {
    if (t->state == 0 && value >= t->high_thresh) {
        t->count++;
        if (t->count >= t->debounce) { t->state = 1; t->count = 0; return 1; }
    } else if (t->state == 1 && value <= t->low_thresh) {
        t->count++;
        if (t->count >= t->debounce) { t->state = 0; t->count = 0; return -1; }
    } else { t->count = 0; }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1438 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1438: Output should not be empty");
    assert!(
        code.contains("sens_thresh_check"),
        "C1438: Should contain sens_thresh_check"
    );
}

#[test]
fn c1439_peak_detector() {
    let c_code = r#"
typedef struct { int peak; int valley; int last; int rising; } sens_peak_t;

void sens_peak_init(sens_peak_t *p) {
    p->peak = -2147483647; p->valley = 2147483647; p->last = 0; p->rising = 1;
}

int sens_peak_feed(sens_peak_t *p, int val) {
    int event = 0;
    if (p->rising) {
        if (val > p->peak) p->peak = val;
        if (val < p->peak - 10) { event = 1; p->rising = 0; p->valley = val; }
    } else {
        if (val < p->valley) p->valley = val;
        if (val > p->valley + 10) { event = -1; p->rising = 1; p->peak = val; }
    }
    p->last = val;
    return event;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1439 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1439: Output should not be empty");
    assert!(
        code.contains("sens_peak_feed"),
        "C1439: Should contain sens_peak_feed"
    );
}

#[test]
fn c1440_zero_crossing_detector() {
    let c_code = r#"
typedef struct { int prev; int crossings; int last_dir; } sens_zcd_t;

void sens_zcd_init(sens_zcd_t *z) { z->prev = 0; z->crossings = 0; z->last_dir = 0; }

int sens_zcd_feed(sens_zcd_t *z, int val) {
    int crossed = 0;
    if (z->prev < 0 && val >= 0) { crossed = 1; z->last_dir = 1; z->crossings++; }
    else if (z->prev >= 0 && val < 0) { crossed = 1; z->last_dir = -1; z->crossings++; }
    z->prev = val;
    return crossed;
}

int sens_zcd_frequency(const sens_zcd_t *z, int sample_period_us) {
    if (z->crossings < 2 || sample_period_us == 0) return 0;
    return 1000000 / (sample_period_us * z->crossings / 2);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1440 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1440: Output should not be empty");
    assert!(
        code.contains("sens_zcd_feed"),
        "C1440: Should contain sens_zcd_feed"
    );
}

// ============================================================================
// C1441-C1445: Protocol Handling
// ============================================================================

#[test]
fn c1441_i2c_message() {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct { uint8_t addr; uint8_t reg; uint8_t data[8]; int len; int is_read; } sens_i2c_msg_t;

void sens_i2c_write_msg(sens_i2c_msg_t *msg, uint8_t addr, uint8_t reg, const uint8_t *data, int len) {
    int i;
    msg->addr = addr; msg->reg = reg; msg->is_read = 0;
    msg->len = len > 8 ? 8 : len;
    for (i = 0; i < msg->len; i++) msg->data[i] = data[i];
}

void sens_i2c_read_msg(sens_i2c_msg_t *msg, uint8_t addr, uint8_t reg, int len) {
    msg->addr = addr; msg->reg = reg; msg->is_read = 1;
    msg->len = len > 8 ? 8 : len;
}

uint8_t sens_i2c_checksum(const sens_i2c_msg_t *msg) {
    uint8_t sum = msg->addr + msg->reg;
    int i;
    for (i = 0; i < msg->len; i++) sum += msg->data[i];
    return sum;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1441 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1441: Output should not be empty");
    assert!(
        code.contains("sens_i2c_write_msg"),
        "C1441: Should contain sens_i2c_write_msg"
    );
}

#[test]
fn c1442_spi_frame() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

typedef struct { uint8_t cmd; uint8_t addr; uint8_t payload[16]; int plen; } sens_spi_frame_t;

void sens_spi_build(sens_spi_frame_t *f, uint8_t cmd, uint8_t addr, const uint8_t *data, int len) {
    int i;
    f->cmd = cmd; f->addr = addr;
    f->plen = len > 16 ? 16 : len;
    for (i = 0; i < f->plen; i++) f->payload[i] = data[i];
}

uint8_t sens_spi_crc8(const sens_spi_frame_t *f) {
    uint8_t crc = 0xFF;
    int i;
    crc ^= f->cmd; crc ^= f->addr;
    for (i = 0; i < f->plen; i++) crc ^= f->payload[i];
    return crc;
}

int sens_spi_frame_size(const sens_spi_frame_t *f) {
    return 2 + f->plen + 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1442 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1442: Output should not be empty");
    assert!(
        code.contains("sens_spi_build"),
        "C1442: Should contain sens_spi_build"
    );
}

#[test]
fn c1443_modbus_rtu() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;

typedef struct { uint8_t slave_id; uint8_t func; uint16_t reg_addr; uint16_t reg_count; uint16_t crc; } sens_modbus_t;

void sens_modbus_build_read(sens_modbus_t *m, uint8_t slave, uint16_t addr, uint16_t count) {
    m->slave_id = slave; m->func = 0x03;
    m->reg_addr = addr; m->reg_count = count; m->crc = 0;
}

uint16_t sens_modbus_crc16(const uint8_t *buf, int len) {
    uint16_t crc = 0xFFFF;
    int i, j;
    for (i = 0; i < len; i++) {
        crc ^= (uint16_t)buf[i];
        for (j = 0; j < 8; j++) {
            if (crc & 1) crc = (crc >> 1) ^ 0xA001;
            else crc >>= 1;
        }
    }
    return crc;
}

int sens_modbus_valid_func(uint8_t func) {
    return func == 0x01 || func == 0x02 || func == 0x03 || func == 0x04 || func == 0x06 || func == 0x10;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1443 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1443: Output should not be empty");
    assert!(
        code.contains("sens_modbus_crc16"),
        "C1443: Should contain sens_modbus_crc16"
    );
}

#[test]
fn c1444_can_bus_message() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

typedef struct { uint32_t id; uint8_t dlc; uint8_t data[8]; int is_extended; } sens_can_msg_t;

void sens_can_build(sens_can_msg_t *msg, uint32_t id, int extended) {
    msg->id = extended ? (id & 0x1FFFFFFF) : (id & 0x7FF);
    msg->dlc = 0; msg->is_extended = extended;
}

int sens_can_set_data(sens_can_msg_t *msg, const uint8_t *data, int len) {
    int i;
    if (len > 8) return -1;
    msg->dlc = (uint8_t)len;
    for (i = 0; i < len; i++) msg->data[i] = data[i];
    return 0;
}

int sens_can_match_filter(const sens_can_msg_t *msg, uint32_t mask, uint32_t filter) {
    return (msg->id & mask) == (filter & mask);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1444 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1444: Output should not be empty");
    assert!(
        code.contains("sens_can_build"),
        "C1444: Should contain sens_can_build"
    );
}

#[test]
fn c1445_onewire_timing() {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct { int pin; int parasitic_power; uint8_t rom[8]; } sens_ow_t;

void sens_ow_init(sens_ow_t *ow, int pin) {
    int i;
    ow->pin = pin; ow->parasitic_power = 0;
    for (i = 0; i < 8; i++) ow->rom[i] = 0;
}

int sens_ow_reset_timing(int us_low, int us_wait) {
    return (us_low >= 480 && us_low <= 640) && (us_wait >= 60 && us_wait <= 75);
}

uint8_t sens_ow_crc8(const uint8_t *data, int len) {
    uint8_t crc = 0;
    int i, j;
    for (i = 0; i < len; i++) {
        uint8_t byte = data[i];
        for (j = 0; j < 8; j++) {
            uint8_t mix = (crc ^ byte) & 0x01;
            crc >>= 1;
            if (mix) crc ^= 0x8C;
            byte >>= 1;
        }
    }
    return crc;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1445 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1445: Output should not be empty");
    assert!(
        code.contains("sens_ow_crc8"),
        "C1445: Should contain sens_ow_crc8"
    );
}

// ============================================================================
// C1446-C1450: IoT Primitives
// ============================================================================

#[test]
fn c1446_telemetry_packet() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;
typedef unsigned short uint16_t;

typedef struct { uint8_t type_id; uint32_t timestamp; int value; uint16_t seq; uint8_t flags; } sens_telem_t;

void sens_telem_build(sens_telem_t *t, uint8_t type_id, uint32_t ts, int val, uint16_t seq) {
    t->type_id = type_id; t->timestamp = ts; t->value = val; t->seq = seq; t->flags = 0;
}

int sens_telem_serialize(const sens_telem_t *t, uint8_t *buf, int buflen) {
    if (buflen < 10) return -1;
    buf[0] = t->type_id;
    buf[1] = (uint8_t)(t->timestamp >> 24);
    buf[2] = (uint8_t)(t->timestamp >> 16);
    buf[3] = (uint8_t)(t->timestamp >> 8);
    buf[4] = (uint8_t)(t->timestamp);
    buf[5] = (uint8_t)(t->value >> 8);
    buf[6] = (uint8_t)(t->value);
    buf[7] = (uint8_t)(t->seq >> 8);
    buf[8] = (uint8_t)(t->seq);
    buf[9] = t->flags;
    return 10;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1446 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1446: Output should not be empty");
    assert!(
        code.contains("sens_telem_build"),
        "C1446: Should contain sens_telem_build"
    );
}

#[test]
fn c1447_device_registry() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct { uint32_t id; int active; int last_seen; int sensor_type; } sens_dev_t;
typedef struct { sens_dev_t devices[32]; int count; } sens_registry_t;

void sens_reg_init(sens_registry_t *r) { r->count = 0; }

int sens_reg_add(sens_registry_t *r, uint32_t id, int sensor_type) {
    if (r->count >= 32) return -1;
    r->devices[r->count].id = id;
    r->devices[r->count].active = 1;
    r->devices[r->count].last_seen = 0;
    r->devices[r->count].sensor_type = sensor_type;
    r->count++;
    return 0;
}

int sens_reg_find(const sens_registry_t *r, uint32_t id) {
    int i;
    for (i = 0; i < r->count; i++) {
        if (r->devices[i].id == id) return i;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1447 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1447: Output should not be empty");
    assert!(
        code.contains("sens_reg_add"),
        "C1447: Should contain sens_reg_add"
    );
}

#[test]
fn c1448_watchdog_timer() {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct { uint32_t timeout_ms; uint32_t last_kick; uint32_t now; int expired; } sens_wdt_t;

void sens_wdt_init(sens_wdt_t *w, uint32_t timeout_ms) {
    w->timeout_ms = timeout_ms; w->last_kick = 0; w->now = 0; w->expired = 0;
}

void sens_wdt_kick(sens_wdt_t *w, uint32_t now) {
    w->last_kick = now; w->now = now; w->expired = 0;
}

int sens_wdt_check(sens_wdt_t *w, uint32_t now) {
    w->now = now;
    if (now - w->last_kick > w->timeout_ms) { w->expired = 1; return 1; }
    return 0;
}

uint32_t sens_wdt_remaining(const sens_wdt_t *w) {
    uint32_t elapsed = w->now - w->last_kick;
    if (elapsed >= w->timeout_ms) return 0;
    return w->timeout_ms - elapsed;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1448 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1448: Output should not be empty");
    assert!(
        code.contains("sens_wdt_kick"),
        "C1448: Should contain sens_wdt_kick"
    );
}

#[test]
fn c1449_power_state_machine() {
    let c_code = r#"
typedef enum { SENS_PWR_OFF = 0, SENS_PWR_SLEEP = 1, SENS_PWR_IDLE = 2, SENS_PWR_ACTIVE = 3 } sens_pwr_state_t;

typedef struct { int state; int wake_count; int sleep_count; } sens_pwr_t;

void sens_pwr_init(sens_pwr_t *p) { p->state = SENS_PWR_OFF; p->wake_count = 0; p->sleep_count = 0; }

int sens_pwr_transition(sens_pwr_t *p, int target) {
    if (target < SENS_PWR_OFF || target > SENS_PWR_ACTIVE) return -1;
    if (p->state == SENS_PWR_OFF && target > SENS_PWR_SLEEP) return -1;
    if (target > p->state) p->wake_count++;
    else if (target < p->state) p->sleep_count++;
    p->state = target;
    return 0;
}

int sens_pwr_is_low_power(const sens_pwr_t *p) {
    return p->state <= SENS_PWR_SLEEP;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1449 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1449: Output should not be empty");
    assert!(
        code.contains("sens_pwr_transition"),
        "C1449: Should contain sens_pwr_transition"
    );
}

#[test]
fn c1450_ota_update_chunk() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

typedef struct { uint32_t total_size; uint32_t received; uint32_t chunk_size; uint32_t crc; int complete; } sens_ota_t;

void sens_ota_init(sens_ota_t *o, uint32_t total_size, uint32_t chunk_size) {
    o->total_size = total_size; o->chunk_size = chunk_size;
    o->received = 0; o->crc = 0xFFFFFFFF; o->complete = 0;
}

int sens_ota_feed(sens_ota_t *o, const uint8_t *data, uint32_t len) {
    uint32_t i;
    if (o->complete) return -1;
    if (o->received + len > o->total_size) return -2;
    for (i = 0; i < len; i++) o->crc ^= (uint32_t)data[i];
    o->received += len;
    if (o->received == o->total_size) o->complete = 1;
    return 0;
}

int sens_ota_progress(const sens_ota_t *o) {
    if (o->total_size == 0) return 0;
    return (int)((o->received * 100) / o->total_size);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1450 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1450: Output should not be empty");
    assert!(
        code.contains("sens_ota_feed"),
        "C1450: Should contain sens_ota_feed"
    );
}
