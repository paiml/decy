//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C551-C575: Robotics and Control Systems patterns -- the kind of C code found
//! in embedded controllers, robot firmware, sensor processing, and motion planning
//! libraries.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world robotics programming patterns commonly
//! found in ROS drivers, ArduPilot, PX4, MoveIt, and similar robotics
//! frameworks -- all expressed as valid C99.
//!
//! Organization:
//! - C551-C555: Controllers and filters (PID, Kalman, EKF, complementary, PWM)
//! - C556-C560: Kinematics and planning (servo interp, FK, IK, trajectory, A*)
//! - C561-C565: Navigation and behavior (obstacle avoid, SLAM, sensor fusion, FSM, PD+FF)
//! - C566-C570: Signal processing (low-pass, high-pass, moving avg, quaternion, DH)
//! - C571-C575: Advanced control (Jacobian, velocity ctrl, F/T sensor, odometry, line follow)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C551-C555: Controllers and Filters
// ============================================================================

/// C551: PID controller with anti-windup
#[test]
fn c551_pid_controller() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float kp;
    float ki;
    float kd;
    float integral;
    float prev_error;
    float output_min;
    float output_max;
} pid_t;

void pid_init(pid_t *pid, float kp, float ki, float kd) {
    pid->kp = kp;
    pid->ki = ki;
    pid->kd = kd;
    pid->integral = 0.0f;
    pid->prev_error = 0.0f;
    pid->output_min = -1.0f;
    pid->output_max = 1.0f;
}

float pid_update(pid_t *pid, float setpoint, float measurement, float dt) {
    float error = setpoint - measurement;
    float p_term = pid->kp * error;
    pid->integral += error * dt;
    float i_term = pid->ki * pid->integral;
    float d_term = 0.0f;
    float output;
    if (dt > 0.0f) {
        d_term = pid->kd * (error - pid->prev_error) / dt;
    }
    output = p_term + i_term + d_term;
    if (output > pid->output_max) {
        output = pid->output_max;
        pid->integral -= error * dt;
    }
    if (output < pid->output_min) {
        output = pid->output_min;
        pid->integral -= error * dt;
    }
    pid->prev_error = error;
    return output;
}

void pid_reset(pid_t *pid) {
    pid->integral = 0.0f;
    pid->prev_error = 0.0f;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C551: Should produce output");
    assert!(rust_code.contains("fn pid_init"), "C551: Should contain pid_init");
    assert!(rust_code.contains("fn pid_update"), "C551: Should contain pid_update");
    Ok(())
}

/// C552: 1D Kalman filter
#[test]
fn c552_kalman_filter_1d() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float x;
    float p;
    float q;
    float r;
    float k;
} kalman1d_t;

void kalman_init(kalman1d_t *kf, float initial_x, float initial_p, float q, float r) {
    kf->x = initial_x;
    kf->p = initial_p;
    kf->q = q;
    kf->r = r;
    kf->k = 0.0f;
}

void kalman_predict(kalman1d_t *kf, float u) {
    kf->x = kf->x + u;
    kf->p = kf->p + kf->q;
}

void kalman_update(kalman1d_t *kf, float z) {
    float innovation = z - kf->x;
    float s = kf->p + kf->r;
    if (s > 0.0001f) {
        kf->k = kf->p / s;
    } else {
        kf->k = 0.0f;
    }
    kf->x = kf->x + kf->k * innovation;
    kf->p = (1.0f - kf->k) * kf->p;
}

float kalman_get_state(const kalman1d_t *kf) {
    return kf->x;
}

float kalman_get_uncertainty(const kalman1d_t *kf) {
    return kf->p;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C552: Should produce output");
    assert!(rust_code.contains("fn kalman_predict"), "C552: Should contain kalman_predict");
    assert!(rust_code.contains("fn kalman_update"), "C552: Should contain kalman_update");
    Ok(())
}

/// C553: Extended Kalman filter state prediction
#[test]
fn c553_extended_kalman_filter_prediction() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float x[3];
    float P[9];
    float Q[9];
    float R[4];
} ekf_state_t;

void ekf_init(ekf_state_t *ekf) {
    int i;
    for (i = 0; i < 3; i++) {
        ekf->x[i] = 0.0f;
    }
    for (i = 0; i < 9; i++) {
        ekf->P[i] = 0.0f;
        ekf->Q[i] = 0.0f;
    }
    for (i = 0; i < 4; i++) {
        ekf->R[i] = 0.0f;
    }
    ekf->P[0] = 1.0f;
    ekf->P[4] = 1.0f;
    ekf->P[8] = 1.0f;
    ekf->Q[0] = 0.01f;
    ekf->Q[4] = 0.01f;
    ekf->Q[8] = 0.01f;
    ekf->R[0] = 0.1f;
    ekf->R[3] = 0.1f;
}

void ekf_predict(ekf_state_t *ekf, float v, float w, float dt) {
    float theta = ekf->x[2];
    float ct = 1.0f;
    float st = 0.0f;
    float F[9];
    float Pnew[9];
    int i, j, k;

    if (theta > -3.15f && theta < 3.15f) {
        ct = 1.0f - theta * theta * 0.5f;
        st = theta - theta * theta * theta / 6.0f;
    }

    ekf->x[0] += v * ct * dt;
    ekf->x[1] += v * st * dt;
    ekf->x[2] += w * dt;

    for (i = 0; i < 9; i++) {
        F[i] = 0.0f;
    }
    F[0] = 1.0f;
    F[2] = -v * st * dt;
    F[4] = 1.0f;
    F[5] = v * ct * dt;
    F[8] = 1.0f;

    for (i = 0; i < 3; i++) {
        for (j = 0; j < 3; j++) {
            Pnew[i * 3 + j] = ekf->Q[i * 3 + j];
            for (k = 0; k < 3; k++) {
                Pnew[i * 3 + j] += F[i * 3 + k] * ekf->P[k * 3 + j];
            }
        }
    }
    for (i = 0; i < 9; i++) {
        ekf->P[i] = Pnew[i];
    }
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C553: Should produce output");
    assert!(rust_code.contains("fn ekf_init"), "C553: Should contain ekf_init");
    assert!(rust_code.contains("fn ekf_predict"), "C553: Should contain ekf_predict");
    Ok(())
}

/// C554: Complementary filter for IMU sensor fusion
#[test]
fn c554_complementary_filter_imu_fusion() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float angle;
    float alpha;
    float bias;
} comp_filter_t;

void comp_filter_init(comp_filter_t *cf, float alpha) {
    cf->angle = 0.0f;
    cf->alpha = alpha;
    cf->bias = 0.0f;
}

float comp_filter_update(comp_filter_t *cf, float gyro_rate, float accel_angle, float dt) {
    float gyro_corrected = gyro_rate - cf->bias;
    float gyro_angle = cf->angle + gyro_corrected * dt;
    cf->angle = cf->alpha * gyro_angle + (1.0f - cf->alpha) * accel_angle;
    return cf->angle;
}

void comp_filter_set_bias(comp_filter_t *cf, float bias) {
    cf->bias = bias;
}

float comp_filter_get_angle(const comp_filter_t *cf) {
    return cf->angle;
}

void comp_filter_calibrate(comp_filter_t *cf, float *gyro_samples, int n) {
    float sum = 0.0f;
    int i;
    for (i = 0; i < n; i++) {
        sum += gyro_samples[i];
    }
    if (n > 0) {
        cf->bias = sum / (float)n;
    }
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C554: Should produce output");
    assert!(
        rust_code.contains("fn comp_filter_init"),
        "C554: Should contain comp_filter_init"
    );
    assert!(
        rust_code.contains("fn comp_filter_update"),
        "C554: Should contain comp_filter_update"
    );
    Ok(())
}

/// C555: Motor PWM control with ramp limiting
#[test]
fn c555_motor_pwm_control() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    float duty_cycle;
    float target_duty;
    float max_ramp_rate;
    uint32_t pwm_period;
    int enabled;
} motor_pwm_t;

void motor_init(motor_pwm_t *m, uint32_t period, float max_ramp) {
    m->duty_cycle = 0.0f;
    m->target_duty = 0.0f;
    m->max_ramp_rate = max_ramp;
    m->pwm_period = period;
    m->enabled = 0;
}

void motor_set_target(motor_pwm_t *m, float target) {
    if (target > 1.0f) {
        m->target_duty = 1.0f;
    } else if (target < -1.0f) {
        m->target_duty = -1.0f;
    } else {
        m->target_duty = target;
    }
}

void motor_update(motor_pwm_t *m, float dt) {
    float diff;
    float step;
    if (m->enabled == 0) {
        m->duty_cycle = 0.0f;
        return;
    }
    diff = m->target_duty - m->duty_cycle;
    step = m->max_ramp_rate * dt;
    if (diff > step) {
        m->duty_cycle += step;
    } else if (diff < -step) {
        m->duty_cycle -= step;
    } else {
        m->duty_cycle = m->target_duty;
    }
}

uint32_t motor_get_compare(const motor_pwm_t *m) {
    float abs_duty = m->duty_cycle;
    if (abs_duty < 0.0f) {
        abs_duty = -abs_duty;
    }
    return (uint32_t)(abs_duty * (float)m->pwm_period);
}

void motor_enable(motor_pwm_t *m) {
    m->enabled = 1;
}

void motor_disable(motor_pwm_t *m) {
    m->enabled = 0;
    m->duty_cycle = 0.0f;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C555: Should produce output");
    assert!(rust_code.contains("fn motor_init"), "C555: Should contain motor_init");
    assert!(rust_code.contains("fn motor_update"), "C555: Should contain motor_update");
    Ok(())
}

// ============================================================================
// C556-C560: Kinematics and Planning
// ============================================================================

/// C556: Servo position interpolation
#[test]
fn c556_servo_position_interpolation() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float current_pos;
    float target_pos;
    float speed;
    float min_pos;
    float max_pos;
} servo_t;

void servo_init(servo_t *s, float min_p, float max_p) {
    s->current_pos = (min_p + max_p) * 0.5f;
    s->target_pos = s->current_pos;
    s->speed = 1.0f;
    s->min_pos = min_p;
    s->max_pos = max_p;
}

void servo_set_target(servo_t *s, float target) {
    if (target < s->min_pos) {
        s->target_pos = s->min_pos;
    } else if (target > s->max_pos) {
        s->target_pos = s->max_pos;
    } else {
        s->target_pos = target;
    }
}

void servo_update(servo_t *s, float dt) {
    float diff = s->target_pos - s->current_pos;
    float step = s->speed * dt;
    if (diff > step) {
        s->current_pos += step;
    } else if (diff < -step) {
        s->current_pos -= step;
    } else {
        s->current_pos = s->target_pos;
    }
}

float servo_get_position(const servo_t *s) {
    return s->current_pos;
}

int servo_at_target(const servo_t *s) {
    float diff = s->target_pos - s->current_pos;
    if (diff < 0.0f) {
        diff = -diff;
    }
    return diff < 0.001f;
}

float servo_lerp(float a, float b, float t) {
    return a + (b - a) * t;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C556: Should produce output");
    assert!(rust_code.contains("fn servo_init"), "C556: Should contain servo_init");
    assert!(rust_code.contains("fn servo_update"), "C556: Should contain servo_update");
    Ok(())
}

/// C557: Forward kinematics for 2-link planar arm
#[test]
fn c557_forward_kinematics_2link() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float x;
    float y;
} point2d_t;

typedef struct {
    float l1;
    float l2;
    float theta1;
    float theta2;
} arm2link_t;

float fk_cos_approx(float x) {
    float x2 = x * x;
    return 1.0f - x2 * 0.5f + x2 * x2 * 0.041667f;
}

float fk_sin_approx(float x) {
    float x2 = x * x;
    return x - x * x2 * 0.166667f + x * x2 * x2 * 0.008333f;
}

void fk_compute(const arm2link_t *arm, point2d_t *elbow, point2d_t *endeff) {
    float c1 = fk_cos_approx(arm->theta1);
    float s1 = fk_sin_approx(arm->theta1);
    float c12 = fk_cos_approx(arm->theta1 + arm->theta2);
    float s12 = fk_sin_approx(arm->theta1 + arm->theta2);
    elbow->x = arm->l1 * c1;
    elbow->y = arm->l1 * s1;
    endeff->x = elbow->x + arm->l2 * c12;
    endeff->y = elbow->y + arm->l2 * s12;
}

void arm_init(arm2link_t *arm, float l1, float l2) {
    arm->l1 = l1;
    arm->l2 = l2;
    arm->theta1 = 0.0f;
    arm->theta2 = 0.0f;
}

void arm_set_angles(arm2link_t *arm, float t1, float t2) {
    arm->theta1 = t1;
    arm->theta2 = t2;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C557: Should produce output");
    assert!(rust_code.contains("fn fk_compute"), "C557: Should contain fk_compute");
    assert!(rust_code.contains("fn arm_init"), "C557: Should contain arm_init");
    Ok(())
}

/// C558: Inverse kinematics for 2-link planar arm
#[test]
fn c558_inverse_kinematics_2link() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float l1;
    float l2;
} ik_arm_t;

typedef struct {
    float theta1;
    float theta2;
    int valid;
} ik_solution_t;

float ik_sqrt_approx(float x) {
    float guess = x * 0.5f;
    int i;
    for (i = 0; i < 10; i++) {
        if (guess > 0.0001f) {
            guess = (guess + x / guess) * 0.5f;
        }
    }
    return guess;
}

float ik_atan2_approx(float y, float x) {
    float abs_x = x;
    float abs_y = y;
    float r;
    float angle;
    if (abs_x < 0.0f) { abs_x = -abs_x; }
    if (abs_y < 0.0f) { abs_y = -abs_y; }
    if (abs_x < 0.0001f && abs_y < 0.0001f) { return 0.0f; }
    if (abs_x > abs_y) {
        r = y / x;
        angle = r - r * r * r * 0.333f;
    } else {
        r = x / y;
        angle = 1.5708f - r + r * r * r * 0.333f;
        if (y < 0.0f) { angle = -angle; }
    }
    if (x < 0.0f) {
        if (y >= 0.0f) { angle += 3.14159f; }
        else { angle -= 3.14159f; }
    }
    return angle;
}

float ik_acos_approx(float x) {
    if (x > 1.0f) { x = 1.0f; }
    if (x < -1.0f) { x = -1.0f; }
    return 1.5708f - x - x * x * x * 0.1667f;
}

void ik_solve(const ik_arm_t *arm, float tx, float ty, ik_solution_t *sol) {
    float dist_sq = tx * tx + ty * ty;
    float dist = ik_sqrt_approx(dist_sq);
    float l1 = arm->l1;
    float l2 = arm->l2;
    float cos_q2;

    if (dist > l1 + l2 || dist < 0.001f) {
        sol->valid = 0;
        sol->theta1 = 0.0f;
        sol->theta2 = 0.0f;
        return;
    }

    cos_q2 = (dist_sq - l1 * l1 - l2 * l2) / (2.0f * l1 * l2);
    sol->theta2 = ik_acos_approx(cos_q2);
    sol->theta1 = ik_atan2_approx(ty, tx) - ik_atan2_approx(l2 * (sol->theta2 - sol->theta2 * sol->theta2 * sol->theta2 * 0.1667f), l1 + l2 * cos_q2);
    sol->valid = 1;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C558: Should produce output");
    assert!(rust_code.contains("fn ik_solve"), "C558: Should contain ik_solve");
    Ok(())
}

/// C559: Trapezoidal velocity profile trajectory planning
#[test]
fn c559_trajectory_trapezoidal_velocity() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float start_pos;
    float end_pos;
    float max_vel;
    float accel;
    float t_accel;
    float t_cruise;
    float t_total;
} traj_trap_t;

float traj_sqrt(float x) {
    float g = x * 0.5f;
    int i;
    for (i = 0; i < 8; i++) {
        if (g > 0.0001f) {
            g = (g + x / g) * 0.5f;
        }
    }
    return g;
}

void traj_plan(traj_trap_t *t, float start, float end, float v_max, float a) {
    float dist = end - start;
    float t_acc;
    float dist_accel;
    if (dist < 0.0f) {
        dist = -dist;
    }
    t->start_pos = start;
    t->end_pos = end;
    t->max_vel = v_max;
    t->accel = a;
    t_acc = v_max / a;
    dist_accel = a * t_acc * t_acc;
    if (dist_accel > dist) {
        t_acc = traj_sqrt(dist / a);
        t->t_accel = t_acc;
        t->t_cruise = 0.0f;
    } else {
        t->t_accel = t_acc;
        t->t_cruise = (dist - dist_accel) / v_max;
    }
    t->t_total = 2.0f * t->t_accel + t->t_cruise;
}

float traj_position(const traj_trap_t *t, float time) {
    float dir = 1.0f;
    float pos;
    if (t->end_pos < t->start_pos) {
        dir = -1.0f;
    }
    if (time <= 0.0f) {
        return t->start_pos;
    }
    if (time >= t->t_total) {
        return t->end_pos;
    }
    if (time < t->t_accel) {
        pos = 0.5f * t->accel * time * time;
    } else if (time < t->t_accel + t->t_cruise) {
        pos = 0.5f * t->accel * t->t_accel * t->t_accel + t->max_vel * (time - t->t_accel);
    } else {
        float td = t->t_total - time;
        float total_dist = t->end_pos - t->start_pos;
        if (total_dist < 0.0f) { total_dist = -total_dist; }
        pos = total_dist - 0.5f * t->accel * td * td;
    }
    return t->start_pos + dir * pos;
}

float traj_velocity(const traj_trap_t *t, float time) {
    float dir = 1.0f;
    if (t->end_pos < t->start_pos) {
        dir = -1.0f;
    }
    if (time <= 0.0f || time >= t->t_total) {
        return 0.0f;
    }
    if (time < t->t_accel) {
        return dir * t->accel * time;
    }
    if (time < t->t_accel + t->t_cruise) {
        return dir * t->max_vel;
    }
    return dir * t->accel * (t->t_total - time);
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C559: Should produce output");
    assert!(rust_code.contains("fn traj_plan"), "C559: Should contain traj_plan");
    assert!(rust_code.contains("fn traj_position"), "C559: Should contain traj_position");
    Ok(())
}

/// C560: A* path planning on grid
#[test]
fn c560_astar_grid_path_planning() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
#define GRID_SIZE 16
#define MAX_OPEN 256

typedef struct {
    int x;
    int y;
    int g_cost;
    int h_cost;
    int f_cost;
    int parent_x;
    int parent_y;
    int closed;
} astar_node_t;

typedef struct {
    astar_node_t nodes[GRID_SIZE * GRID_SIZE];
    int grid[GRID_SIZE * GRID_SIZE];
    int open_list[MAX_OPEN];
    int open_count;
} astar_t;

int astar_heuristic(int x1, int y1, int x2, int y2) {
    int dx = x2 - x1;
    int dy = y2 - y1;
    if (dx < 0) { dx = -dx; }
    if (dy < 0) { dy = -dy; }
    return dx + dy;
}

void astar_init(astar_t *a) {
    int i;
    for (i = 0; i < GRID_SIZE * GRID_SIZE; i++) {
        a->nodes[i].x = i % GRID_SIZE;
        a->nodes[i].y = i / GRID_SIZE;
        a->nodes[i].g_cost = 99999;
        a->nodes[i].h_cost = 0;
        a->nodes[i].f_cost = 99999;
        a->nodes[i].parent_x = -1;
        a->nodes[i].parent_y = -1;
        a->nodes[i].closed = 0;
        a->grid[i] = 0;
    }
    a->open_count = 0;
}

void astar_set_obstacle(astar_t *a, int x, int y) {
    if (x >= 0 && x < GRID_SIZE && y >= 0 && y < GRID_SIZE) {
        a->grid[y * GRID_SIZE + x] = 1;
    }
}

int astar_find_lowest_f(const astar_t *a) {
    int best = -1;
    int best_f = 999999;
    int i;
    for (i = 0; i < a->open_count; i++) {
        int idx = a->open_list[i];
        if (a->nodes[idx].f_cost < best_f) {
            best_f = a->nodes[idx].f_cost;
            best = i;
        }
    }
    return best;
}

int astar_is_valid(const astar_t *a, int x, int y) {
    if (x < 0 || x >= GRID_SIZE || y < 0 || y >= GRID_SIZE) {
        return 0;
    }
    return a->grid[y * GRID_SIZE + x] == 0;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C560: Should produce output");
    assert!(rust_code.contains("fn astar_init"), "C560: Should contain astar_init");
    assert!(
        rust_code.contains("fn astar_heuristic"),
        "C560: Should contain astar_heuristic"
    );
    Ok(())
}

// ============================================================================
// C561-C565: Navigation and Behavior
// ============================================================================

/// C561: Obstacle avoidance using potential field method
#[test]
fn c561_obstacle_avoidance_potential_field() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
#define MAX_OBSTACLES 16

typedef struct {
    float x;
    float y;
} vec2_t;

typedef struct {
    vec2_t obstacles[MAX_OBSTACLES];
    float radii[MAX_OBSTACLES];
    int count;
    float attract_gain;
    float repulse_gain;
    float repulse_dist;
} potential_field_t;

void pf_init(potential_field_t *pf, float attract, float repulse, float dist) {
    pf->count = 0;
    pf->attract_gain = attract;
    pf->repulse_gain = repulse;
    pf->repulse_dist = dist;
}

void pf_add_obstacle(potential_field_t *pf, float ox, float oy, float r) {
    if (pf->count < MAX_OBSTACLES) {
        pf->obstacles[pf->count].x = ox;
        pf->obstacles[pf->count].y = oy;
        pf->radii[pf->count] = r;
        pf->count++;
    }
}

float pf_sqrt(float x) {
    float g = x * 0.5f;
    int i;
    for (i = 0; i < 8; i++) {
        if (g > 0.0001f) {
            g = (g + x / g) * 0.5f;
        }
    }
    return g;
}

void pf_compute(const potential_field_t *pf, float rx, float ry, float gx, float gy, vec2_t *force) {
    int i;
    float dx, dy, dist, rep_f;
    force->x = pf->attract_gain * (gx - rx);
    force->y = pf->attract_gain * (gy - ry);
    for (i = 0; i < pf->count; i++) {
        dx = rx - pf->obstacles[i].x;
        dy = ry - pf->obstacles[i].y;
        dist = pf_sqrt(dx * dx + dy * dy);
        if (dist < pf->repulse_dist && dist > 0.001f) {
            rep_f = pf->repulse_gain * (1.0f / dist - 1.0f / pf->repulse_dist);
            force->x += rep_f * dx / dist;
            force->y += rep_f * dy / dist;
        }
    }
}

float pf_magnitude(const vec2_t *v) {
    return pf_sqrt(v->x * v->x + v->y * v->y);
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C561: Should produce output");
    assert!(rust_code.contains("fn pf_compute"), "C561: Should contain pf_compute");
    assert!(
        rust_code.contains("fn pf_add_obstacle"),
        "C561: Should contain pf_add_obstacle"
    );
    Ok(())
}

/// C562: Simple SLAM landmark update
#[test]
fn c562_slam_landmark_update() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
#define MAX_LANDMARKS 32

typedef struct {
    float x;
    float y;
    float covariance;
    int observed;
} landmark_t;

typedef struct {
    float robot_x;
    float robot_y;
    float robot_theta;
    landmark_t landmarks[MAX_LANDMARKS];
    int landmark_count;
} slam_state_t;

void slam_init(slam_state_t *s) {
    int i;
    s->robot_x = 0.0f;
    s->robot_y = 0.0f;
    s->robot_theta = 0.0f;
    s->landmark_count = 0;
    for (i = 0; i < MAX_LANDMARKS; i++) {
        s->landmarks[i].x = 0.0f;
        s->landmarks[i].y = 0.0f;
        s->landmarks[i].covariance = 100.0f;
        s->landmarks[i].observed = 0;
    }
}

float slam_sqrt(float x) {
    float g = x * 0.5f;
    int i;
    for (i = 0; i < 8; i++) {
        if (g > 0.0001f) {
            g = (g + x / g) * 0.5f;
        }
    }
    return g;
}

int slam_find_nearest(const slam_state_t *s, float mx, float my, float threshold) {
    int i;
    int best = -1;
    float best_dist = threshold;
    for (i = 0; i < s->landmark_count; i++) {
        float dx = s->landmarks[i].x - mx;
        float dy = s->landmarks[i].y - my;
        float dist = slam_sqrt(dx * dx + dy * dy);
        if (dist < best_dist) {
            best_dist = dist;
            best = i;
        }
    }
    return best;
}

void slam_update_landmark(slam_state_t *s, int idx, float mx, float my, float meas_cov) {
    float k;
    float total_cov = s->landmarks[idx].covariance + meas_cov;
    if (total_cov > 0.001f) {
        k = s->landmarks[idx].covariance / total_cov;
    } else {
        k = 0.5f;
    }
    s->landmarks[idx].x += k * (mx - s->landmarks[idx].x);
    s->landmarks[idx].y += k * (my - s->landmarks[idx].y);
    s->landmarks[idx].covariance *= (1.0f - k);
    s->landmarks[idx].observed++;
}

void slam_add_landmark(slam_state_t *s, float mx, float my) {
    if (s->landmark_count < MAX_LANDMARKS) {
        s->landmarks[s->landmark_count].x = mx;
        s->landmarks[s->landmark_count].y = my;
        s->landmarks[s->landmark_count].covariance = 10.0f;
        s->landmarks[s->landmark_count].observed = 1;
        s->landmark_count++;
    }
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C562: Should produce output");
    assert!(rust_code.contains("fn slam_init"), "C562: Should contain slam_init");
    assert!(
        rust_code.contains("fn slam_update_landmark"),
        "C562: Should contain slam_update_landmark"
    );
    Ok(())
}

/// C563: Sensor fusion with weighted average
#[test]
fn c563_sensor_fusion_weighted_average() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
#define MAX_SENSORS 8

typedef struct {
    float value;
    float variance;
    int active;
} sensor_reading_t;

typedef struct {
    sensor_reading_t sensors[MAX_SENSORS];
    int count;
    float fused_value;
    float fused_variance;
} sensor_fusion_t;

void fusion_init(sensor_fusion_t *sf) {
    int i;
    sf->count = 0;
    sf->fused_value = 0.0f;
    sf->fused_variance = 100.0f;
    for (i = 0; i < MAX_SENSORS; i++) {
        sf->sensors[i].value = 0.0f;
        sf->sensors[i].variance = 1.0f;
        sf->sensors[i].active = 0;
    }
}

void fusion_add_sensor(sensor_fusion_t *sf, float variance) {
    if (sf->count < MAX_SENSORS) {
        sf->sensors[sf->count].variance = variance;
        sf->sensors[sf->count].active = 1;
        sf->count++;
    }
}

void fusion_update_reading(sensor_fusion_t *sf, int idx, float value) {
    if (idx >= 0 && idx < sf->count) {
        sf->sensors[idx].value = value;
    }
}

void fusion_compute(sensor_fusion_t *sf) {
    float weight_sum = 0.0f;
    float weighted_val = 0.0f;
    int i;
    for (i = 0; i < sf->count; i++) {
        if (sf->sensors[i].active != 0 && sf->sensors[i].variance > 0.0001f) {
            float w = 1.0f / sf->sensors[i].variance;
            weight_sum += w;
            weighted_val += w * sf->sensors[i].value;
        }
    }
    if (weight_sum > 0.0001f) {
        sf->fused_value = weighted_val / weight_sum;
        sf->fused_variance = 1.0f / weight_sum;
    }
}

float fusion_get_value(const sensor_fusion_t *sf) {
    return sf->fused_value;
}

float fusion_get_confidence(const sensor_fusion_t *sf) {
    if (sf->fused_variance > 0.0001f) {
        return 1.0f / sf->fused_variance;
    }
    return 0.0f;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C563: Should produce output");
    assert!(rust_code.contains("fn fusion_init"), "C563: Should contain fusion_init");
    assert!(rust_code.contains("fn fusion_compute"), "C563: Should contain fusion_compute");
    Ok(())
}

/// C564: Robot behavior state machine
#[test]
fn c564_robot_state_machine() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef enum {
    STATE_IDLE,
    STATE_EXPLORE,
    STATE_GOTO,
    STATE_AVOID,
    STATE_CHARGE,
    STATE_ERROR
} robot_state_e;

typedef struct {
    robot_state_e state;
    float battery_level;
    float obstacle_dist;
    float goal_x;
    float goal_y;
    float pos_x;
    float pos_y;
    int error_code;
} robot_fsm_t;

void fsm_init(robot_fsm_t *r) {
    r->state = STATE_IDLE;
    r->battery_level = 100.0f;
    r->obstacle_dist = 999.0f;
    r->goal_x = 0.0f;
    r->goal_y = 0.0f;
    r->pos_x = 0.0f;
    r->pos_y = 0.0f;
    r->error_code = 0;
}

void fsm_update(robot_fsm_t *r) {
    if (r->battery_level < 10.0f) {
        r->state = STATE_CHARGE;
        return;
    }
    if (r->error_code != 0) {
        r->state = STATE_ERROR;
        return;
    }
    if (r->state == STATE_IDLE) {
        r->state = STATE_EXPLORE;
    } else if (r->state == STATE_EXPLORE) {
        if (r->obstacle_dist < 0.5f) {
            r->state = STATE_AVOID;
        }
    } else if (r->state == STATE_GOTO) {
        float dx = r->goal_x - r->pos_x;
        float dy = r->goal_y - r->pos_y;
        if (dx * dx + dy * dy < 0.01f) {
            r->state = STATE_IDLE;
        }
        if (r->obstacle_dist < 0.5f) {
            r->state = STATE_AVOID;
        }
    } else if (r->state == STATE_AVOID) {
        if (r->obstacle_dist > 1.0f) {
            r->state = STATE_EXPLORE;
        }
    } else if (r->state == STATE_CHARGE) {
        if (r->battery_level > 90.0f) {
            r->state = STATE_IDLE;
        }
    }
}

int fsm_is_moving(const robot_fsm_t *r) {
    return r->state == STATE_EXPLORE || r->state == STATE_GOTO || r->state == STATE_AVOID;
}

void fsm_set_goal(robot_fsm_t *r, float gx, float gy) {
    r->goal_x = gx;
    r->goal_y = gy;
    if (r->state == STATE_IDLE || r->state == STATE_EXPLORE) {
        r->state = STATE_GOTO;
    }
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C564: Should produce output");
    assert!(rust_code.contains("fn fsm_init"), "C564: Should contain fsm_init");
    assert!(rust_code.contains("fn fsm_update"), "C564: Should contain fsm_update");
    Ok(())
}

/// C565: PD controller with feedforward term
#[test]
fn c565_pd_controller_with_feedforward() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float kp;
    float kd;
    float kff;
    float prev_error;
    float output_limit;
} pd_ff_t;

void pd_ff_init(pd_ff_t *c, float kp, float kd, float kff, float limit) {
    c->kp = kp;
    c->kd = kd;
    c->kff = kff;
    c->prev_error = 0.0f;
    c->output_limit = limit;
}

float pd_ff_update(pd_ff_t *c, float setpoint, float measurement, float ff_input, float dt) {
    float error = setpoint - measurement;
    float p_term = c->kp * error;
    float d_term = 0.0f;
    float ff_term = c->kff * ff_input;
    float output;
    if (dt > 0.0001f) {
        d_term = c->kd * (error - c->prev_error) / dt;
    }
    output = p_term + d_term + ff_term;
    if (output > c->output_limit) {
        output = c->output_limit;
    }
    if (output < -c->output_limit) {
        output = -c->output_limit;
    }
    c->prev_error = error;
    return output;
}

void pd_ff_reset(pd_ff_t *c) {
    c->prev_error = 0.0f;
}

float pd_ff_get_error(const pd_ff_t *c) {
    return c->prev_error;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C565: Should produce output");
    assert!(rust_code.contains("fn pd_ff_init"), "C565: Should contain pd_ff_init");
    assert!(rust_code.contains("fn pd_ff_update"), "C565: Should contain pd_ff_update");
    Ok(())
}

// ============================================================================
// C566-C570: Signal Processing
// ============================================================================

/// C566: Digital low-pass filter (first order IIR)
#[test]
fn c566_lowpass_filter_digital() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float alpha;
    float output;
    int initialized;
} lowpass_t;

void lowpass_init(lowpass_t *lp, float cutoff_hz, float sample_hz) {
    float rc;
    float dt;
    if (sample_hz < 1.0f) {
        sample_hz = 1.0f;
    }
    dt = 1.0f / sample_hz;
    if (cutoff_hz < 0.01f) {
        cutoff_hz = 0.01f;
    }
    rc = 1.0f / (2.0f * 3.14159f * cutoff_hz);
    lp->alpha = dt / (rc + dt);
    lp->output = 0.0f;
    lp->initialized = 0;
}

float lowpass_update(lowpass_t *lp, float input) {
    if (lp->initialized == 0) {
        lp->output = input;
        lp->initialized = 1;
        return input;
    }
    lp->output = lp->alpha * input + (1.0f - lp->alpha) * lp->output;
    return lp->output;
}

void lowpass_reset(lowpass_t *lp) {
    lp->output = 0.0f;
    lp->initialized = 0;
}

float lowpass_get(const lowpass_t *lp) {
    return lp->output;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C566: Should produce output");
    assert!(rust_code.contains("fn lowpass_init"), "C566: Should contain lowpass_init");
    assert!(rust_code.contains("fn lowpass_update"), "C566: Should contain lowpass_update");
    Ok(())
}

/// C567: Digital high-pass filter (first order)
#[test]
fn c567_highpass_filter_digital() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float alpha;
    float prev_input;
    float output;
    int initialized;
} highpass_t;

void highpass_init(highpass_t *hp, float cutoff_hz, float sample_hz) {
    float rc;
    float dt;
    if (sample_hz < 1.0f) {
        sample_hz = 1.0f;
    }
    dt = 1.0f / sample_hz;
    if (cutoff_hz < 0.01f) {
        cutoff_hz = 0.01f;
    }
    rc = 1.0f / (2.0f * 3.14159f * cutoff_hz);
    hp->alpha = rc / (rc + dt);
    hp->prev_input = 0.0f;
    hp->output = 0.0f;
    hp->initialized = 0;
}

float highpass_update(highpass_t *hp, float input) {
    if (hp->initialized == 0) {
        hp->prev_input = input;
        hp->output = 0.0f;
        hp->initialized = 1;
        return 0.0f;
    }
    hp->output = hp->alpha * (hp->output + input - hp->prev_input);
    hp->prev_input = input;
    return hp->output;
}

void highpass_reset(highpass_t *hp) {
    hp->prev_input = 0.0f;
    hp->output = 0.0f;
    hp->initialized = 0;
}

float highpass_get(const highpass_t *hp) {
    return hp->output;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C567: Should produce output");
    assert!(rust_code.contains("fn highpass_init"), "C567: Should contain highpass_init");
    assert!(
        rust_code.contains("fn highpass_update"),
        "C567: Should contain highpass_update"
    );
    Ok(())
}

/// C568: Moving average filter
#[test]
fn c568_moving_average_filter() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
#define MA_MAX_SIZE 64

typedef struct {
    float buffer[MA_MAX_SIZE];
    int size;
    int index;
    float sum;
    int count;
} moving_avg_t;

void ma_init(moving_avg_t *ma, int size) {
    int i;
    if (size > MA_MAX_SIZE) {
        size = MA_MAX_SIZE;
    }
    if (size < 1) {
        size = 1;
    }
    ma->size = size;
    ma->index = 0;
    ma->sum = 0.0f;
    ma->count = 0;
    for (i = 0; i < MA_MAX_SIZE; i++) {
        ma->buffer[i] = 0.0f;
    }
}

float ma_update(moving_avg_t *ma, float input) {
    if (ma->count >= ma->size) {
        ma->sum -= ma->buffer[ma->index];
    }
    ma->buffer[ma->index] = input;
    ma->sum += input;
    ma->index++;
    if (ma->index >= ma->size) {
        ma->index = 0;
    }
    if (ma->count < ma->size) {
        ma->count++;
    }
    return ma->sum / (float)ma->count;
}

float ma_get_average(const moving_avg_t *ma) {
    if (ma->count > 0) {
        return ma->sum / (float)ma->count;
    }
    return 0.0f;
}

void ma_reset(moving_avg_t *ma) {
    int i;
    ma->index = 0;
    ma->sum = 0.0f;
    ma->count = 0;
    for (i = 0; i < MA_MAX_SIZE; i++) {
        ma->buffer[i] = 0.0f;
    }
}

int ma_is_full(const moving_avg_t *ma) {
    return ma->count >= ma->size;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C568: Should produce output");
    assert!(rust_code.contains("fn ma_init"), "C568: Should contain ma_init");
    assert!(rust_code.contains("fn ma_update"), "C568: Should contain ma_update");
    Ok(())
}

/// C569: Quaternion rotation
#[test]
fn c569_quaternion_rotation() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float w;
    float x;
    float y;
    float z;
} quat_t;

typedef struct {
    float x;
    float y;
    float z;
} vec3_t;

void quat_identity(quat_t *q) {
    q->w = 1.0f;
    q->x = 0.0f;
    q->y = 0.0f;
    q->z = 0.0f;
}

float quat_sqrt(float v) {
    float g = v * 0.5f;
    int i;
    for (i = 0; i < 8; i++) {
        if (g > 0.0001f) {
            g = (g + v / g) * 0.5f;
        }
    }
    return g;
}

void quat_normalize(quat_t *q) {
    float mag = quat_sqrt(q->w * q->w + q->x * q->x + q->y * q->y + q->z * q->z);
    if (mag > 0.0001f) {
        float inv = 1.0f / mag;
        q->w *= inv;
        q->x *= inv;
        q->y *= inv;
        q->z *= inv;
    }
}

void quat_multiply(const quat_t *a, const quat_t *b, quat_t *out) {
    out->w = a->w * b->w - a->x * b->x - a->y * b->y - a->z * b->z;
    out->x = a->w * b->x + a->x * b->w + a->y * b->z - a->z * b->y;
    out->y = a->w * b->y - a->x * b->z + a->y * b->w + a->z * b->x;
    out->z = a->w * b->z + a->x * b->y - a->y * b->x + a->z * b->w;
}

void quat_rotate_vec(const quat_t *q, const vec3_t *v, vec3_t *out) {
    float qv_w = -(q->x * v->x + q->y * v->y + q->z * v->z);
    float qv_x = q->w * v->x + q->y * v->z - q->z * v->y;
    float qv_y = q->w * v->y + q->z * v->x - q->x * v->z;
    float qv_z = q->w * v->z + q->x * v->y - q->y * v->x;
    out->x = -qv_w * q->x + qv_x * q->w - qv_y * q->z + qv_z * q->y;
    out->y = -qv_w * q->y + qv_x * q->z + qv_y * q->w - qv_z * q->x;
    out->z = -qv_w * q->z - qv_x * q->y + qv_y * q->x + qv_z * q->w;
}

void quat_conjugate(const quat_t *q, quat_t *out) {
    out->w = q->w;
    out->x = -q->x;
    out->y = -q->y;
    out->z = -q->z;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C569: Should produce output");
    assert!(
        rust_code.contains("fn quat_normalize"),
        "C569: Should contain quat_normalize"
    );
    assert!(
        rust_code.contains("fn quat_rotate_vec"),
        "C569: Should contain quat_rotate_vec"
    );
    Ok(())
}

/// C570: Denavit-Hartenberg parameter transform
#[test]
fn c570_dh_parameter_transform() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float m[16];
} mat4_t;

typedef struct {
    float theta;
    float d;
    float a;
    float alpha;
} dh_params_t;

float dh_cos(float x) {
    float x2 = x * x;
    return 1.0f - x2 * 0.5f + x2 * x2 * 0.041667f;
}

float dh_sin(float x) {
    float x2 = x * x;
    return x - x * x2 * 0.166667f + x * x2 * x2 * 0.008333f;
}

void dh_transform(const dh_params_t *dh, mat4_t *T) {
    float ct = dh_cos(dh->theta);
    float st = dh_sin(dh->theta);
    float ca = dh_cos(dh->alpha);
    float sa = dh_sin(dh->alpha);
    T->m[0] = ct;
    T->m[1] = -st * ca;
    T->m[2] = st * sa;
    T->m[3] = dh->a * ct;
    T->m[4] = st;
    T->m[5] = ct * ca;
    T->m[6] = -ct * sa;
    T->m[7] = dh->a * st;
    T->m[8] = 0.0f;
    T->m[9] = sa;
    T->m[10] = ca;
    T->m[11] = dh->d;
    T->m[12] = 0.0f;
    T->m[13] = 0.0f;
    T->m[14] = 0.0f;
    T->m[15] = 1.0f;
}

void mat4_multiply(const mat4_t *A, const mat4_t *B, mat4_t *C) {
    int i, j, k;
    for (i = 0; i < 4; i++) {
        for (j = 0; j < 4; j++) {
            C->m[i * 4 + j] = 0.0f;
            for (k = 0; k < 4; k++) {
                C->m[i * 4 + j] += A->m[i * 4 + k] * B->m[k * 4 + j];
            }
        }
    }
}

void mat4_identity(mat4_t *m) {
    int i;
    for (i = 0; i < 16; i++) {
        m->m[i] = 0.0f;
    }
    m->m[0] = 1.0f;
    m->m[5] = 1.0f;
    m->m[10] = 1.0f;
    m->m[15] = 1.0f;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C570: Should produce output");
    assert!(rust_code.contains("fn dh_transform"), "C570: Should contain dh_transform");
    assert!(
        rust_code.contains("fn mat4_multiply"),
        "C570: Should contain mat4_multiply"
    );
    Ok(())
}

// ============================================================================
// C571-C575: Advanced Control
// ============================================================================

/// C571: Jacobian computation for 2-link arm
#[test]
fn c571_jacobian_2link() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float j[4];
} jacobian2x2_t;

float jac_cos(float x) {
    float x2 = x * x;
    return 1.0f - x2 * 0.5f + x2 * x2 * 0.041667f;
}

float jac_sin(float x) {
    float x2 = x * x;
    return x - x * x2 * 0.166667f + x * x2 * x2 * 0.008333f;
}

void jacobian_compute(float l1, float l2, float t1, float t2, jacobian2x2_t *J) {
    float c1 = jac_cos(t1);
    float s1 = jac_sin(t1);
    float c12 = jac_cos(t1 + t2);
    float s12 = jac_sin(t1 + t2);
    J->j[0] = -l1 * s1 - l2 * s12;
    J->j[1] = -l2 * s12;
    J->j[2] = l1 * c1 + l2 * c12;
    J->j[3] = l2 * c12;
}

float jacobian_det(const jacobian2x2_t *J) {
    return J->j[0] * J->j[3] - J->j[1] * J->j[2];
}

int jacobian_invert(const jacobian2x2_t *J, jacobian2x2_t *Jinv) {
    float det = jacobian_det(J);
    float abs_det = det;
    if (abs_det < 0.0f) {
        abs_det = -abs_det;
    }
    if (abs_det < 0.0001f) {
        return 0;
    }
    float inv_det = 1.0f / det;
    Jinv->j[0] = J->j[3] * inv_det;
    Jinv->j[1] = -J->j[1] * inv_det;
    Jinv->j[2] = -J->j[2] * inv_det;
    Jinv->j[3] = J->j[0] * inv_det;
    return 1;
}

void jacobian_multiply_vec(const jacobian2x2_t *J, float vx, float vy, float *ox, float *oy) {
    *ox = J->j[0] * vx + J->j[1] * vy;
    *oy = J->j[2] * vx + J->j[3] * vy;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C571: Should produce output");
    assert!(
        rust_code.contains("fn jacobian_compute"),
        "C571: Should contain jacobian_compute"
    );
    assert!(
        rust_code.contains("fn jacobian_det"),
        "C571: Should contain jacobian_det"
    );
    Ok(())
}

/// C572: Velocity controller for differential drive
#[test]
fn c572_velocity_controller() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float kp_linear;
    float kp_angular;
    float max_linear;
    float max_angular;
    float wheel_base;
} vel_ctrl_t;

typedef struct {
    float left_speed;
    float right_speed;
} wheel_speeds_t;

void vel_ctrl_init(vel_ctrl_t *vc, float kpl, float kpa, float ml, float ma, float wb) {
    vc->kp_linear = kpl;
    vc->kp_angular = kpa;
    vc->max_linear = ml;
    vc->max_angular = ma;
    vc->wheel_base = wb;
}

float vel_clamp(float v, float limit) {
    if (v > limit) { return limit; }
    if (v < -limit) { return -limit; }
    return v;
}

void vel_ctrl_compute(const vel_ctrl_t *vc, float v_desired, float w_desired,
                      float v_current, float w_current, wheel_speeds_t *out) {
    float v_cmd = vc->kp_linear * (v_desired - v_current);
    float w_cmd = vc->kp_angular * (w_desired - w_current);
    float v;
    float w;
    v_cmd = vel_clamp(v_cmd, vc->max_linear);
    w_cmd = vel_clamp(w_cmd, vc->max_angular);
    v = v_cmd;
    w = w_cmd;
    out->left_speed = v - w * vc->wheel_base * 0.5f;
    out->right_speed = v + w * vc->wheel_base * 0.5f;
}

void vel_ctrl_stop(wheel_speeds_t *out) {
    out->left_speed = 0.0f;
    out->right_speed = 0.0f;
}

float vel_ctrl_speed(const wheel_speeds_t *ws) {
    return (ws->left_speed + ws->right_speed) * 0.5f;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C572: Should produce output");
    assert!(
        rust_code.contains("fn vel_ctrl_init"),
        "C572: Should contain vel_ctrl_init"
    );
    assert!(
        rust_code.contains("fn vel_ctrl_compute"),
        "C572: Should contain vel_ctrl_compute"
    );
    Ok(())
}

/// C573: Force-torque sensor data processing
#[test]
fn c573_force_torque_sensor() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float fx;
    float fy;
    float fz;
    float tx;
    float ty;
    float tz;
} ft_reading_t;

typedef struct {
    ft_reading_t bias;
    ft_reading_t filtered;
    float alpha;
    int calibrated;
} ft_sensor_t;

void ft_init(ft_sensor_t *s, float filter_alpha) {
    s->bias.fx = 0.0f;
    s->bias.fy = 0.0f;
    s->bias.fz = 0.0f;
    s->bias.tx = 0.0f;
    s->bias.ty = 0.0f;
    s->bias.tz = 0.0f;
    s->filtered.fx = 0.0f;
    s->filtered.fy = 0.0f;
    s->filtered.fz = 0.0f;
    s->filtered.tx = 0.0f;
    s->filtered.ty = 0.0f;
    s->filtered.tz = 0.0f;
    s->alpha = filter_alpha;
    s->calibrated = 0;
}

void ft_calibrate(ft_sensor_t *s, const ft_reading_t *samples, int n) {
    int i;
    s->bias.fx = 0.0f;
    s->bias.fy = 0.0f;
    s->bias.fz = 0.0f;
    s->bias.tx = 0.0f;
    s->bias.ty = 0.0f;
    s->bias.tz = 0.0f;
    for (i = 0; i < n; i++) {
        s->bias.fx += samples[i].fx;
        s->bias.fy += samples[i].fy;
        s->bias.fz += samples[i].fz;
        s->bias.tx += samples[i].tx;
        s->bias.ty += samples[i].ty;
        s->bias.tz += samples[i].tz;
    }
    if (n > 0) {
        float inv_n = 1.0f / (float)n;
        s->bias.fx *= inv_n;
        s->bias.fy *= inv_n;
        s->bias.fz *= inv_n;
        s->bias.tx *= inv_n;
        s->bias.ty *= inv_n;
        s->bias.tz *= inv_n;
    }
    s->calibrated = 1;
}

void ft_process(ft_sensor_t *s, const ft_reading_t *raw) {
    float a = s->alpha;
    float b = 1.0f - a;
    float cx = raw->fx - s->bias.fx;
    float cy = raw->fy - s->bias.fy;
    float cz = raw->fz - s->bias.fz;
    s->filtered.fx = a * cx + b * s->filtered.fx;
    s->filtered.fy = a * cy + b * s->filtered.fy;
    s->filtered.fz = a * cz + b * s->filtered.fz;
    s->filtered.tx = a * (raw->tx - s->bias.tx) + b * s->filtered.tx;
    s->filtered.ty = a * (raw->ty - s->bias.ty) + b * s->filtered.ty;
    s->filtered.tz = a * (raw->tz - s->bias.tz) + b * s->filtered.tz;
}

float ft_force_magnitude(const ft_sensor_t *s) {
    float fx = s->filtered.fx;
    float fy = s->filtered.fy;
    float fz = s->filtered.fz;
    float sum = fx * fx + fy * fy + fz * fz;
    float g = sum * 0.5f;
    int i;
    for (i = 0; i < 8; i++) {
        if (g > 0.0001f) {
            g = (g + sum / g) * 0.5f;
        }
    }
    return g;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C573: Should produce output");
    assert!(rust_code.contains("fn ft_init"), "C573: Should contain ft_init");
    assert!(rust_code.contains("fn ft_process"), "C573: Should contain ft_process");
    Ok(())
}

/// C574: Differential drive odometry calculation
#[test]
fn c574_odometry_calculation() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float x;
    float y;
    float theta;
    float wheel_base;
    float wheel_radius;
    int left_ticks;
    int right_ticks;
    int prev_left;
    int prev_right;
    float ticks_per_rev;
} odometry_t;

void odom_init(odometry_t *o, float wb, float wr, float tpr) {
    o->x = 0.0f;
    o->y = 0.0f;
    o->theta = 0.0f;
    o->wheel_base = wb;
    o->wheel_radius = wr;
    o->left_ticks = 0;
    o->right_ticks = 0;
    o->prev_left = 0;
    o->prev_right = 0;
    o->ticks_per_rev = tpr;
}

float odom_cos(float x) {
    float x2 = x * x;
    return 1.0f - x2 * 0.5f + x2 * x2 * 0.041667f;
}

float odom_sin(float x) {
    float x2 = x * x;
    return x - x * x2 * 0.166667f + x * x2 * x2 * 0.008333f;
}

void odom_update(odometry_t *o, int left_enc, int right_enc) {
    int dl = left_enc - o->prev_left;
    int dr = right_enc - o->prev_right;
    float dist_per_tick = 2.0f * 3.14159f * o->wheel_radius / o->ticks_per_rev;
    float left_dist = (float)dl * dist_per_tick;
    float right_dist = (float)dr * dist_per_tick;
    float center_dist = (left_dist + right_dist) * 0.5f;
    float dtheta = (right_dist - left_dist) / o->wheel_base;
    float mid_theta = o->theta + dtheta * 0.5f;
    o->x += center_dist * odom_cos(mid_theta);
    o->y += center_dist * odom_sin(mid_theta);
    o->theta += dtheta;
    o->prev_left = left_enc;
    o->prev_right = right_enc;
}

float odom_distance_to(const odometry_t *o, float gx, float gy) {
    float dx = gx - o->x;
    float dy = gy - o->y;
    float sum = dx * dx + dy * dy;
    float g = sum * 0.5f;
    int i;
    for (i = 0; i < 8; i++) {
        if (g > 0.0001f) {
            g = (g + sum / g) * 0.5f;
        }
    }
    return g;
}

void odom_reset(odometry_t *o) {
    o->x = 0.0f;
    o->y = 0.0f;
    o->theta = 0.0f;
    o->prev_left = o->left_ticks;
    o->prev_right = o->right_ticks;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C574: Should produce output");
    assert!(rust_code.contains("fn odom_init"), "C574: Should contain odom_init");
    assert!(rust_code.contains("fn odom_update"), "C574: Should contain odom_update");
    Ok(())
}

/// C575: Line following PID controller
#[test]
fn c575_line_following_pid() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
#define NUM_LINE_SENSORS 5

typedef struct {
    float kp;
    float ki;
    float kd;
    float integral;
    float prev_error;
    float base_speed;
    float max_correction;
} line_pid_t;

typedef struct {
    int raw[NUM_LINE_SENSORS];
    float weights[NUM_LINE_SENSORS];
    float threshold;
} line_sensors_t;

void line_pid_init(line_pid_t *lp, float kp, float ki, float kd, float speed) {
    lp->kp = kp;
    lp->ki = ki;
    lp->kd = kd;
    lp->integral = 0.0f;
    lp->prev_error = 0.0f;
    lp->base_speed = speed;
    lp->max_correction = speed * 0.8f;
}

void line_sensors_init(line_sensors_t *ls, float thresh) {
    int i;
    ls->threshold = thresh;
    for (i = 0; i < NUM_LINE_SENSORS; i++) {
        ls->raw[i] = 0;
        ls->weights[i] = (float)(i - NUM_LINE_SENSORS / 2);
    }
}

float line_compute_error(const line_sensors_t *ls) {
    float weighted_sum = 0.0f;
    float active_sum = 0.0f;
    int i;
    for (i = 0; i < NUM_LINE_SENSORS; i++) {
        if ((float)ls->raw[i] > ls->threshold) {
            weighted_sum += ls->weights[i] * (float)ls->raw[i];
            active_sum += (float)ls->raw[i];
        }
    }
    if (active_sum > 0.001f) {
        return weighted_sum / active_sum;
    }
    return 0.0f;
}

void line_pid_update(line_pid_t *lp, float error, float dt, float *left_motor, float *right_motor) {
    float p_term = lp->kp * error;
    float d_term = 0.0f;
    float correction;
    lp->integral += error * dt;
    if (lp->integral > 10.0f) { lp->integral = 10.0f; }
    if (lp->integral < -10.0f) { lp->integral = -10.0f; }
    if (dt > 0.0001f) {
        d_term = lp->kd * (error - lp->prev_error) / dt;
    }
    correction = p_term + lp->ki * lp->integral + d_term;
    if (correction > lp->max_correction) {
        correction = lp->max_correction;
    }
    if (correction < -lp->max_correction) {
        correction = -lp->max_correction;
    }
    *left_motor = lp->base_speed + correction;
    *right_motor = lp->base_speed - correction;
    lp->prev_error = error;
}

int line_detect_junction(const line_sensors_t *ls) {
    int count = 0;
    int i;
    for (i = 0; i < NUM_LINE_SENSORS; i++) {
        if ((float)ls->raw[i] > ls->threshold) {
            count++;
        }
    }
    return count >= 4;
}

int line_lost(const line_sensors_t *ls) {
    int i;
    for (i = 0; i < NUM_LINE_SENSORS; i++) {
        if ((float)ls->raw[i] > ls->threshold) {
            return 0;
        }
    }
    return 1;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C575: Should produce output");
    assert!(
        rust_code.contains("fn line_pid_init"),
        "C575: Should contain line_pid_init"
    );
    assert!(
        rust_code.contains("fn line_pid_update"),
        "C575: Should contain line_pid_update"
    );
    Ok(())
}
