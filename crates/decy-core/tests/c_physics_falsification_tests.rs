//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1476-C1500: Physics Simulation patterns -- the kind of C code found in
//! physics engines, game engines, and scientific simulation software.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world physics simulation patterns commonly
//! found in Box2D, Bullet, PhysX, SPH solvers, cloth simulators, and
//! spatial acceleration structures -- all expressed as valid C99.
//!
//! Organization:
//! - C1476-C1480: Particle systems (init/step, emitter, force accumulator, collision, lifespan)
//! - C1481-C1485: Rigid body dynamics (state, force/torque, AABB collision, impulse, constraint)
//! - C1486-C1490: Fluid simulation (SPH density, pressure, viscosity, advection, boundary)
//! - C1491-C1495: Cloth/soft body (spring-mass, distance constraint, bending, self-collision, wind)
//! - C1496-C1500: Spatial structures (uniform grid, BVH, k-d tree, octree, broad phase)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C1476-C1480: Particle Systems
// ============================================================================

#[test]
fn c1476_particle_init_step() {
    let c_code = r#"
typedef struct { float x; float y; float vx; float vy; float mass; int alive; } phys_particle_t;

void phys_particle_init(phys_particle_t *p, float x, float y) {
    p->x = x; p->y = y;
    p->vx = 0.0f; p->vy = 0.0f;
    p->mass = 1.0f; p->alive = 1;
}

void phys_particle_step(phys_particle_t *p, float dt) {
    if (!p->alive) return;
    p->x += p->vx * dt;
    p->y += p->vy * dt;
}

void phys_particle_apply_gravity(phys_particle_t *p, float g, float dt) {
    if (!p->alive) return;
    p->vy += g * dt;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1476 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1476: empty output");
    assert!(
        code.contains("fn phys_particle_init"),
        "C1476: should contain phys_particle_init"
    );
}

#[test]
fn c1477_particle_emitter() {
    let c_code = r#"
typedef struct { float x; float y; float vx; float vy; int alive; } phys_part_t;
typedef struct { phys_part_t particles[256]; int count; int max_count; float emit_rate; float timer; } phys_emitter_t;

void phys_emitter_init(phys_emitter_t *e, int max) {
    e->count = 0;
    e->max_count = max;
    e->emit_rate = 10.0f;
    e->timer = 0.0f;
}

void phys_emitter_emit(phys_emitter_t *e, float x, float y, float vx, float vy) {
    if (e->count >= e->max_count) return;
    phys_part_t *p = &e->particles[e->count];
    p->x = x; p->y = y;
    p->vx = vx; p->vy = vy;
    p->alive = 1;
    e->count++;
}

void phys_emitter_update(phys_emitter_t *e, float dt) {
    e->timer += dt;
    while (e->timer >= 1.0f / e->emit_rate) {
        phys_emitter_emit(e, 0.0f, 0.0f, 0.0f, 1.0f);
        e->timer -= 1.0f / e->emit_rate;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1477 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1477: empty output");
    assert!(
        code.contains("fn phys_emitter_init"),
        "C1477: should contain phys_emitter_init"
    );
}

#[test]
fn c1478_force_accumulator() {
    let c_code = r#"
typedef struct { float fx; float fy; float fz; } phys_vec3_t;
typedef struct { phys_vec3_t forces[16]; int count; } phys_force_accum_t;

void phys_accum_reset(phys_force_accum_t *a) {
    a->count = 0;
}

void phys_accum_add(phys_force_accum_t *a, float fx, float fy, float fz) {
    if (a->count >= 16) return;
    a->forces[a->count].fx = fx;
    a->forces[a->count].fy = fy;
    a->forces[a->count].fz = fz;
    a->count++;
}

phys_vec3_t phys_accum_total(const phys_force_accum_t *a) {
    phys_vec3_t total;
    total.fx = 0.0f; total.fy = 0.0f; total.fz = 0.0f;
    int i;
    for (i = 0; i < a->count; i++) {
        total.fx += a->forces[i].fx;
        total.fy += a->forces[i].fy;
        total.fz += a->forces[i].fz;
    }
    return total;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1478 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1478: empty output");
    assert!(
        code.contains("fn phys_accum_total"),
        "C1478: should contain phys_accum_total"
    );
}

#[test]
fn c1479_particle_collision() {
    let c_code = r#"
typedef struct { float x; float y; float vx; float vy; float radius; } phys_ball_t;

float phys_distance_sq(float x1, float y1, float x2, float y2) {
    float dx = x2 - x1;
    float dy = y2 - y1;
    return dx * dx + dy * dy;
}

int phys_balls_collide(const phys_ball_t *a, const phys_ball_t *b) {
    float r = a->radius + b->radius;
    return phys_distance_sq(a->x, a->y, b->x, b->y) < r * r;
}

void phys_resolve_collision(phys_ball_t *a, phys_ball_t *b) {
    if (!phys_balls_collide(a, b)) return;
    float tmpvx = a->vx; float tmpvy = a->vy;
    a->vx = b->vx; a->vy = b->vy;
    b->vx = tmpvx; b->vy = tmpvy;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1479 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1479: empty output");
    assert!(
        code.contains("fn phys_balls_collide"),
        "C1479: should contain phys_balls_collide"
    );
}

#[test]
fn c1480_particle_lifespan() {
    let c_code = r#"
typedef struct { float x; float y; float vx; float vy; float life; float max_life; int active; } phys_lpart_t;

void phys_lpart_spawn(phys_lpart_t *p, float x, float y, float max_life) {
    p->x = x; p->y = y;
    p->vx = 0.0f; p->vy = 0.0f;
    p->life = 0.0f;
    p->max_life = max_life;
    p->active = 1;
}

void phys_lpart_tick(phys_lpart_t *p, float dt) {
    if (!p->active) return;
    p->life += dt;
    if (p->life >= p->max_life) { p->active = 0; return; }
    p->x += p->vx * dt;
    p->y += p->vy * dt;
}

float phys_lpart_alpha(const phys_lpart_t *p) {
    if (!p->active || p->max_life <= 0.0f) return 0.0f;
    return 1.0f - p->life / p->max_life;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1480 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1480: empty output");
    assert!(
        code.contains("fn phys_lpart_alpha"),
        "C1480: should contain phys_lpart_alpha"
    );
}

// ============================================================================
// C1481-C1485: Rigid Body Dynamics
// ============================================================================

#[test]
fn c1481_rigid_body_state() {
    let c_code = r#"
typedef struct { float x; float y; float angle; float vx; float vy; float omega; float mass; float inertia; } phys_rbody_t;

void phys_rbody_init(phys_rbody_t *b, float x, float y, float mass) {
    b->x = x; b->y = y;
    b->angle = 0.0f;
    b->vx = 0.0f; b->vy = 0.0f;
    b->omega = 0.0f;
    b->mass = mass;
    b->inertia = mass * 0.5f;
}

void phys_rbody_integrate(phys_rbody_t *b, float dt) {
    b->x += b->vx * dt;
    b->y += b->vy * dt;
    b->angle += b->omega * dt;
}

float phys_rbody_kinetic_energy(const phys_rbody_t *b) {
    float lin = 0.5f * b->mass * (b->vx * b->vx + b->vy * b->vy);
    float rot = 0.5f * b->inertia * b->omega * b->omega;
    return lin + rot;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1481 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1481: empty output");
    assert!(
        code.contains("fn phys_rbody_init"),
        "C1481: should contain phys_rbody_init"
    );
}

#[test]
fn c1482_force_and_torque() {
    let c_code = r#"
typedef struct { float x; float y; float vx; float vy; float omega; float mass; float inertia; float fx; float fy; float torque; } phys_body2d_t;

void phys_body2d_clear_forces(phys_body2d_t *b) {
    b->fx = 0.0f; b->fy = 0.0f; b->torque = 0.0f;
}

void phys_body2d_apply_force(phys_body2d_t *b, float fx, float fy) {
    b->fx += fx;
    b->fy += fy;
}

void phys_body2d_apply_torque_at(phys_body2d_t *b, float fx, float fy, float rx, float ry) {
    b->fx += fx;
    b->fy += fy;
    b->torque += rx * fy - ry * fx;
}

void phys_body2d_step(phys_body2d_t *b, float dt) {
    float inv_m = 1.0f / b->mass;
    b->vx += b->fx * inv_m * dt;
    b->vy += b->fy * inv_m * dt;
    b->omega += b->torque / b->inertia * dt;
    b->x += b->vx * dt;
    b->y += b->vy * dt;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1482 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1482: empty output");
    assert!(
        code.contains("fn phys_body2d_apply_torque_at"),
        "C1482: should contain phys_body2d_apply_torque_at"
    );
}

#[test]
fn c1483_aabb_collision() {
    let c_code = r#"
typedef struct { float min_x; float min_y; float max_x; float max_y; } phys_aabb_t;

int phys_aabb_overlap(const phys_aabb_t *a, const phys_aabb_t *b) {
    if (a->max_x < b->min_x || b->max_x < a->min_x) return 0;
    if (a->max_y < b->min_y || b->max_y < a->min_y) return 0;
    return 1;
}

phys_aabb_t phys_aabb_merge(const phys_aabb_t *a, const phys_aabb_t *b) {
    phys_aabb_t r;
    r.min_x = a->min_x < b->min_x ? a->min_x : b->min_x;
    r.min_y = a->min_y < b->min_y ? a->min_y : b->min_y;
    r.max_x = a->max_x > b->max_x ? a->max_x : b->max_x;
    r.max_y = a->max_y > b->max_y ? a->max_y : b->max_y;
    return r;
}

float phys_aabb_area(const phys_aabb_t *a) {
    return (a->max_x - a->min_x) * (a->max_y - a->min_y);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1483 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1483: empty output");
    assert!(
        code.contains("fn phys_aabb_overlap"),
        "C1483: should contain phys_aabb_overlap"
    );
}

#[test]
fn c1484_impulse_response() {
    let c_code = r#"
typedef struct { float vx; float vy; float mass; float restitution; } phys_impbody_t;

void phys_impulse_apply(phys_impbody_t *a, phys_impbody_t *b, float nx, float ny) {
    float rel_vx = b->vx - a->vx;
    float rel_vy = b->vy - a->vy;
    float vel_along = rel_vx * nx + rel_vy * ny;
    if (vel_along > 0.0f) return;
    float e = a->restitution < b->restitution ? a->restitution : b->restitution;
    float j = -(1.0f + e) * vel_along;
    float inv_sum = 1.0f / a->mass + 1.0f / b->mass;
    j = j / inv_sum;
    a->vx -= j / a->mass * nx;
    a->vy -= j / a->mass * ny;
    b->vx += j / b->mass * nx;
    b->vy += j / b->mass * ny;
}

float phys_impulse_magnitude(float v_rel, float e, float inv_mass_sum) {
    return -(1.0f + e) * v_rel / inv_mass_sum;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1484 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1484: empty output");
    assert!(
        code.contains("fn phys_impulse_apply"),
        "C1484: should contain phys_impulse_apply"
    );
}

#[test]
fn c1485_constraint_solver() {
    let c_code = r#"
typedef struct { float x; float y; float inv_mass; } phys_cpoint_t;
typedef struct { int a; int b; float rest_length; } phys_constraint_t;

void phys_solve_distance(phys_cpoint_t *pts, const phys_constraint_t *c) {
    float dx = pts[c->b].x - pts[c->a].x;
    float dy = pts[c->b].y - pts[c->a].y;
    float dist_sq = dx * dx + dy * dy;
    if (dist_sq < 0.0001f) return;
    float dist = dist_sq;
    int i;
    for (i = 0; i < 5; i++) dist = 0.5f * (dist + dist_sq / dist);
    float diff = (dist - c->rest_length) / dist;
    float w = pts[c->a].inv_mass + pts[c->b].inv_mass;
    if (w < 0.0001f) return;
    float s1 = pts[c->a].inv_mass / w * diff;
    float s2 = pts[c->b].inv_mass / w * diff;
    pts[c->a].x += dx * s1;
    pts[c->a].y += dy * s1;
    pts[c->b].x -= dx * s2;
    pts[c->b].y -= dy * s2;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1485 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1485: empty output");
    assert!(
        code.contains("fn phys_solve_distance"),
        "C1485: should contain phys_solve_distance"
    );
}

// ============================================================================
// C1486-C1490: Fluid Simulation
// ============================================================================

#[test]
fn c1486_sph_density() {
    let c_code = r#"
typedef struct { float x; float y; float density; float pressure; float mass; } phys_sph_t;

float phys_sph_kernel(float r, float h) {
    if (r >= h) return 0.0f;
    float q = 1.0f - r / h;
    return 315.0f / (64.0f * 3.14159f * h * h) * q * q * q;
}

void phys_sph_compute_density(phys_sph_t *particles, int n, float h) {
    int i, j;
    for (i = 0; i < n; i++) {
        particles[i].density = 0.0f;
        for (j = 0; j < n; j++) {
            float dx = particles[i].x - particles[j].x;
            float dy = particles[i].y - particles[j].y;
            float r = dx * dx + dy * dy;
            int k; float sq = r;
            for (k = 0; k < 5; k++) sq = 0.5f * (sq + r / sq);
            particles[i].density += particles[j].mass * phys_sph_kernel(sq, h);
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1486 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1486: empty output");
    assert!(
        code.contains("fn phys_sph_kernel"),
        "C1486: should contain phys_sph_kernel"
    );
}

#[test]
fn c1487_pressure_force() {
    let c_code = r#"
typedef struct { float x; float y; float vx; float vy; float density; float pressure; float mass; } phys_fluid_t;

void phys_compute_pressure(phys_fluid_t *p, int n, float k, float rho0) {
    int i;
    for (i = 0; i < n; i++) {
        p[i].pressure = k * (p[i].density - rho0);
    }
}

void phys_pressure_force(phys_fluid_t *particles, int n, float h) {
    int i, j;
    for (i = 0; i < n; i++) {
        float fpx = 0.0f, fpy = 0.0f;
        for (j = 0; j < n; j++) {
            if (i == j) continue;
            float dx = particles[j].x - particles[i].x;
            float dy = particles[j].y - particles[i].y;
            float r2 = dx * dx + dy * dy;
            if (r2 > h * h || r2 < 0.0001f) continue;
            float avg_p = (particles[i].pressure + particles[j].pressure) * 0.5f;
            float w = particles[j].mass / particles[j].density;
            fpx += avg_p * w * dx;
            fpy += avg_p * w * dy;
        }
        particles[i].vx += fpx / particles[i].density;
        particles[i].vy += fpy / particles[i].density;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1487 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1487: empty output");
    assert!(
        code.contains("fn phys_pressure_force"),
        "C1487: should contain phys_pressure_force"
    );
}

#[test]
fn c1488_viscosity() {
    let c_code = r#"
typedef struct { float x; float y; float vx; float vy; float density; float mass; } phys_visc_t;

float phys_visc_kernel(float r, float h) {
    if (r >= h) return 0.0f;
    return 45.0f / (3.14159f * h * h * h * h * h * h) * (h - r);
}

void phys_apply_viscosity(phys_visc_t *p, int n, float h, float mu) {
    int i, j;
    for (i = 0; i < n; i++) {
        float ax = 0.0f, ay = 0.0f;
        for (j = 0; j < n; j++) {
            if (i == j) continue;
            float dx = p[j].x - p[i].x;
            float dy = p[j].y - p[i].y;
            float r2 = dx * dx + dy * dy;
            if (r2 > h * h) continue;
            float r = r2; int k;
            for (k = 0; k < 5; k++) r = 0.5f * (r + r2 / r);
            float w = phys_visc_kernel(r, h) * p[j].mass / p[j].density;
            ax += (p[j].vx - p[i].vx) * w;
            ay += (p[j].vy - p[i].vy) * w;
        }
        p[i].vx += mu * ax; p[i].vy += mu * ay;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1488 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1488: empty output");
    assert!(
        code.contains("fn phys_apply_viscosity"),
        "C1488: should contain phys_apply_viscosity"
    );
}

#[test]
fn c1489_grid_advection() {
    let c_code = r#"
typedef struct { float u[64]; float v[64]; int nx; int ny; } phys_grid_t;

void phys_grid_init(phys_grid_t *g, int nx, int ny) {
    g->nx = nx; g->ny = ny;
    int i;
    for (i = 0; i < 64; i++) { g->u[i] = 0.0f; g->v[i] = 0.0f; }
}

float phys_grid_sample(const float *field, int nx, int ny, float x, float y) {
    int ix = (int)x; int iy = (int)y;
    if (ix < 0) ix = 0; if (ix >= nx - 1) ix = nx - 2;
    if (iy < 0) iy = 0; if (iy >= ny - 1) iy = ny - 2;
    float fx = x - ix; float fy = y - iy;
    float v00 = field[iy * nx + ix];
    float v10 = field[iy * nx + ix + 1];
    float v01 = field[(iy + 1) * nx + ix];
    float v11 = field[(iy + 1) * nx + ix + 1];
    return v00 * (1 - fx) * (1 - fy) + v10 * fx * (1 - fy) + v01 * (1 - fx) * fy + v11 * fx * fy;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1489 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1489: empty output");
    assert!(
        code.contains("fn phys_grid_sample"),
        "C1489: should contain phys_grid_sample"
    );
}

#[test]
fn c1490_boundary_conditions() {
    let c_code = r#"
typedef struct { float x; float y; float vx; float vy; } phys_bpart_t;
typedef struct { float xmin; float ymin; float xmax; float ymax; float damping; } phys_bounds_t;

void phys_bounds_init(phys_bounds_t *b, float w, float h) {
    b->xmin = 0.0f; b->ymin = 0.0f;
    b->xmax = w; b->ymax = h;
    b->damping = 0.8f;
}

void phys_enforce_bounds(phys_bpart_t *p, const phys_bounds_t *b) {
    if (p->x < b->xmin) { p->x = b->xmin; p->vx = -p->vx * b->damping; }
    if (p->x > b->xmax) { p->x = b->xmax; p->vx = -p->vx * b->damping; }
    if (p->y < b->ymin) { p->y = b->ymin; p->vy = -p->vy * b->damping; }
    if (p->y > b->ymax) { p->y = b->ymax; p->vy = -p->vy * b->damping; }
}

int phys_is_in_bounds(const phys_bpart_t *p, const phys_bounds_t *b) {
    return p->x >= b->xmin && p->x <= b->xmax && p->y >= b->ymin && p->y <= b->ymax;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1490 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1490: empty output");
    assert!(
        code.contains("fn phys_enforce_bounds"),
        "C1490: should contain phys_enforce_bounds"
    );
}

// ============================================================================
// C1491-C1495: Cloth / Soft Body Simulation
// ============================================================================

#[test]
fn c1491_spring_mass_system() {
    let c_code = r#"
typedef struct { float x; float y; float ox; float oy; float mass; int pinned; } phys_node_t;

void phys_node_init(phys_node_t *n, float x, float y) {
    n->x = x; n->y = y;
    n->ox = x; n->oy = y;
    n->mass = 1.0f; n->pinned = 0;
}

void phys_verlet_step(phys_node_t *n, float gx, float gy, float dt) {
    if (n->pinned) return;
    float nx = 2.0f * n->x - n->ox + gx * dt * dt;
    float ny = 2.0f * n->y - n->oy + gy * dt * dt;
    n->ox = n->x; n->oy = n->y;
    n->x = nx; n->y = ny;
}

void phys_spring_step(phys_node_t *a, phys_node_t *b, float rest) {
    float dx = b->x - a->x;
    float dy = b->y - a->y;
    float d2 = dx * dx + dy * dy;
    if (d2 < 0.0001f) return;
    float d = d2; int i;
    for (i = 0; i < 5; i++) d = 0.5f * (d + d2 / d);
    float diff = (d - rest) / d * 0.5f;
    if (!a->pinned) { a->x += dx * diff; a->y += dy * diff; }
    if (!b->pinned) { b->x -= dx * diff; b->y -= dy * diff; }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1491 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1491: empty output");
    assert!(
        code.contains("fn phys_verlet_step"),
        "C1491: should contain phys_verlet_step"
    );
}

#[test]
fn c1492_distance_constraint() {
    let c_code = r#"
typedef struct { float x; float y; } phys_dpt_t;
typedef struct { int i0; int i1; float rest; float stiffness; } phys_dconstraint_t;

void phys_solve_dconstraint(phys_dpt_t *pts, const phys_dconstraint_t *c) {
    float dx = pts[c->i1].x - pts[c->i0].x;
    float dy = pts[c->i1].y - pts[c->i0].y;
    float d2 = dx * dx + dy * dy;
    if (d2 < 0.0001f) return;
    float d = d2; int k;
    for (k = 0; k < 5; k++) d = 0.5f * (d + d2 / d);
    float correction = c->stiffness * (d - c->rest) / d;
    float hc = correction * 0.5f;
    pts[c->i0].x += dx * hc;
    pts[c->i0].y += dy * hc;
    pts[c->i1].x -= dx * hc;
    pts[c->i1].y -= dy * hc;
}

void phys_solve_iterations(phys_dpt_t *pts, const phys_dconstraint_t *constraints, int nc, int iters) {
    int it, c;
    for (it = 0; it < iters; it++) {
        for (c = 0; c < nc; c++) {
            phys_solve_dconstraint(pts, &constraints[c]);
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1492 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1492: empty output");
    assert!(
        code.contains("fn phys_solve_dconstraint"),
        "C1492: should contain phys_solve_dconstraint"
    );
}

#[test]
fn c1493_bending_constraint() {
    let c_code = r#"
typedef struct { float x; float y; } phys_bpt_t;

float phys_triangle_area(const phys_bpt_t *a, const phys_bpt_t *b, const phys_bpt_t *c) {
    float ax = b->x - a->x, ay = b->y - a->y;
    float bx = c->x - a->x, by = c->y - a->y;
    float cross = ax * by - ay * bx;
    return cross < 0 ? -cross * 0.5f : cross * 0.5f;
}

void phys_bend_constraint(phys_bpt_t *p0, phys_bpt_t *p1, phys_bpt_t *p2, float rest_angle, float stiffness) {
    float d1x = p1->x - p0->x, d1y = p1->y - p0->y;
    float d2x = p2->x - p0->x, d2y = p2->y - p0->y;
    float dot = d1x * d2x + d1y * d2y;
    float cross = d1x * d2y - d1y * d2x;
    float angle = 0.0f;
    if (dot > 0.0001f || dot < -0.0001f) {
        angle = cross / dot;
    }
    float diff = stiffness * (angle - rest_angle);
    p1->x -= diff * d1y * 0.5f;
    p1->y += diff * d1x * 0.5f;
    p2->x -= diff * d2y * 0.5f;
    p2->y += diff * d2x * 0.5f;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1493 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1493: empty output");
    assert!(
        code.contains("fn phys_bend_constraint"),
        "C1493: should contain phys_bend_constraint"
    );
}

#[test]
fn c1494_self_collision() {
    let c_code = r#"
typedef struct { float x; float y; float radius; } phys_snode_t;

int phys_self_collide_check(const phys_snode_t *a, const phys_snode_t *b) {
    float dx = b->x - a->x;
    float dy = b->y - a->y;
    float min_d = a->radius + b->radius;
    return (dx * dx + dy * dy) < (min_d * min_d);
}

void phys_self_collide_resolve(phys_snode_t *a, phys_snode_t *b) {
    float dx = b->x - a->x;
    float dy = b->y - a->y;
    float d2 = dx * dx + dy * dy;
    float min_d = a->radius + b->radius;
    if (d2 >= min_d * min_d || d2 < 0.0001f) return;
    float d = d2; int k;
    for (k = 0; k < 5; k++) d = 0.5f * (d + d2 / d);
    float overlap = (min_d - d) * 0.5f / d;
    a->x -= dx * overlap;
    a->y -= dy * overlap;
    b->x += dx * overlap;
    b->y += dy * overlap;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1494 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1494: empty output");
    assert!(
        code.contains("fn phys_self_collide_resolve"),
        "C1494: should contain phys_self_collide_resolve"
    );
}

#[test]
fn c1495_wind_force() {
    let c_code = r#"
typedef struct { float x; float y; float vx; float vy; float area; } phys_sail_t;

void phys_wind_apply(phys_sail_t *s, float wx, float wy, float drag) {
    float rel_x = wx - s->vx;
    float rel_y = wy - s->vy;
    float speed2 = rel_x * rel_x + rel_y * rel_y;
    float force = drag * s->area * speed2;
    float sp = speed2; int i;
    for (i = 0; i < 5; i++) sp = 0.5f * (sp + speed2 / sp);
    if (sp < 0.0001f) return;
    s->vx += force * rel_x / sp;
    s->vy += force * rel_y / sp;
}

void phys_turbulent_wind(float *wx, float *wy, float base_x, float base_y, float t) {
    float phase = t * 2.0f;
    float variation = 0.3f;
    *wx = base_x + variation * (phase - (int)phase);
    *wy = base_y + variation * (1.0f - (phase - (int)phase));
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1495 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1495: empty output");
    assert!(
        code.contains("fn phys_wind_apply"),
        "C1495: should contain phys_wind_apply"
    );
}

// ============================================================================
// C1496-C1500: Spatial Acceleration Structures
// ============================================================================

#[test]
fn c1496_uniform_grid() {
    let c_code = r#"
typedef struct { int cells[256]; int counts[64]; int nx; int ny; float cell_size; } phys_ugrid_t;

void phys_ugrid_init(phys_ugrid_t *g, int nx, int ny, float cs) {
    g->nx = nx; g->ny = ny; g->cell_size = cs;
    int i;
    for (i = 0; i < 64; i++) g->counts[i] = 0;
    for (i = 0; i < 256; i++) g->cells[i] = -1;
}

int phys_ugrid_cell(const phys_ugrid_t *g, float x, float y) {
    int cx = (int)(x / g->cell_size);
    int cy = (int)(y / g->cell_size);
    if (cx < 0) cx = 0; if (cx >= g->nx) cx = g->nx - 1;
    if (cy < 0) cy = 0; if (cy >= g->ny) cy = g->ny - 1;
    return cy * g->nx + cx;
}

void phys_ugrid_insert(phys_ugrid_t *g, float x, float y, int id) {
    int cell = phys_ugrid_cell(g, x, y);
    int base = cell * 4;
    if (g->counts[cell] >= 4) return;
    g->cells[base + g->counts[cell]] = id;
    g->counts[cell]++;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1496 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1496: empty output");
    assert!(
        code.contains("fn phys_ugrid_insert"),
        "C1496: should contain phys_ugrid_insert"
    );
}

#[test]
fn c1497_bvh_construction() {
    let c_code = r#"
typedef struct { float min_x; float min_y; float max_x; float max_y; } phys_bbox_t;
typedef struct { phys_bbox_t bounds; int left; int right; int obj_id; } phys_bvh_node_t;

phys_bbox_t phys_bbox_union(const phys_bbox_t *a, const phys_bbox_t *b) {
    phys_bbox_t r;
    r.min_x = a->min_x < b->min_x ? a->min_x : b->min_x;
    r.min_y = a->min_y < b->min_y ? a->min_y : b->min_y;
    r.max_x = a->max_x > b->max_x ? a->max_x : b->max_x;
    r.max_y = a->max_y > b->max_y ? a->max_y : b->max_y;
    return r;
}

float phys_bbox_cost(const phys_bbox_t *b) {
    return (b->max_x - b->min_x) * (b->max_y - b->min_y);
}

int phys_bvh_is_leaf(const phys_bvh_node_t *n) {
    return n->left == -1 && n->right == -1;
}

int phys_bbox_overlaps(const phys_bbox_t *a, const phys_bbox_t *b) {
    if (a->max_x < b->min_x || b->max_x < a->min_x) return 0;
    if (a->max_y < b->min_y || b->max_y < a->min_y) return 0;
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1497 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1497: empty output");
    assert!(
        code.contains("fn phys_bvh_is_leaf"),
        "C1497: should contain phys_bvh_is_leaf"
    );
}

#[test]
fn c1498_kd_tree_nearest() {
    let c_code = r#"
typedef struct { float x; float y; int left; int right; int axis; } phys_kd_node_t;

float phys_kd_dist2(float x1, float y1, float x2, float y2) {
    float dx = x2 - x1; float dy = y2 - y1;
    return dx * dx + dy * dy;
}

int phys_kd_closer(const phys_kd_node_t *nodes, int a, int b, float qx, float qy) {
    if (a == -1) return b;
    if (b == -1) return a;
    float da = phys_kd_dist2(nodes[a].x, nodes[a].y, qx, qy);
    float db = phys_kd_dist2(nodes[b].x, nodes[b].y, qx, qy);
    return da < db ? a : b;
}

float phys_kd_axis_dist(const phys_kd_node_t *n, float qx, float qy) {
    if (n->axis == 0) { float d = qx - n->x; return d * d; }
    else { float d = qy - n->y; return d * d; }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1498 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1498: empty output");
    assert!(
        code.contains("fn phys_kd_closer"),
        "C1498: should contain phys_kd_closer"
    );
}

#[test]
fn c1499_octree_insert() {
    let c_code = r#"
typedef struct { float cx; float cy; float half; int children[4]; int obj_id; int count; } phys_qtree_t;

void phys_qtree_init(phys_qtree_t *q, float cx, float cy, float half) {
    q->cx = cx; q->cy = cy; q->half = half;
    q->obj_id = -1; q->count = 0;
    int i; for (i = 0; i < 4; i++) q->children[i] = -1;
}

int phys_qtree_quadrant(const phys_qtree_t *q, float x, float y) {
    int idx = 0;
    if (x >= q->cx) idx |= 1;
    if (y >= q->cy) idx |= 2;
    return idx;
}

int phys_qtree_contains(const phys_qtree_t *q, float x, float y) {
    return x >= q->cx - q->half && x <= q->cx + q->half &&
           y >= q->cy - q->half && y <= q->cy + q->half;
}

int phys_qtree_is_leaf(const phys_qtree_t *q) {
    return q->children[0] == -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1499 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1499: empty output");
    assert!(
        code.contains("fn phys_qtree_quadrant"),
        "C1499: should contain phys_qtree_quadrant"
    );
}

#[test]
fn c1500_broad_phase_collision_pairs() {
    let c_code = r#"
typedef struct { float min_x; float max_x; int id; } phys_interval_t;
typedef struct { int a; int b; } phys_pair_t;

void phys_sort_intervals(phys_interval_t *arr, int n) {
    int i, j;
    for (i = 1; i < n; i++) {
        phys_interval_t key = arr[i];
        j = i - 1;
        while (j >= 0 && arr[j].min_x > key.min_x) {
            arr[j + 1] = arr[j];
            j--;
        }
        arr[j + 1] = key;
    }
}

int phys_sweep_and_prune(phys_interval_t *intervals, int n, phys_pair_t *pairs, int max_pairs) {
    phys_sort_intervals(intervals, n);
    int count = 0;
    int i, j;
    for (i = 0; i < n && count < max_pairs; i++) {
        for (j = i + 1; j < n && intervals[j].min_x <= intervals[i].max_x; j++) {
            pairs[count].a = intervals[i].id;
            pairs[count].b = intervals[j].id;
            count++;
            if (count >= max_pairs) break;
        }
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1500 failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1500: empty output");
    assert!(
        code.contains("fn phys_sweep_and_prune"),
        "C1500: should contain phys_sweep_and_prune"
    );
}
