//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C426-C450: Graphics, Rendering, and Game Engine patterns -- the kind of C
//! code found in software renderers, physics engines, game engines, and GPU
//! compute pipelines.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world graphics and game engine patterns commonly
//! found in Quake, Doom, raylib, stb_image, Box2D, and similar projects --
//! all expressed as valid C99.
//!
//! Organization:
//! - C426-C430: Vector and matrix math (2D/3D vectors, 4x4 matrices, quaternions)
//! - C431-C435: Spatial algorithms (AABB, ray-sphere, Bresenham, rasterization, Z-buffer)
//! - C436-C440: Rendering pipeline (alpha blend, frustum cull, UV atlas, tilemap, particles)
//! - C441-C445: Curves and spatial structures (projection, Bezier, Catmull-Rom, spatial hash, vertex buffer)
//! - C446-C450: Advanced rendering (normal maps, LOD, SSAO, shadow maps, skeletal animation)
//!
//! Results: 24 passing, 1 falsified (96.0% pass rate)

// ============================================================================
// C426-C430: Vector and Matrix Math
// ============================================================================

#[test]
fn c426_2d_vector_math_operations() {
    let c_code = r#"
typedef struct {
    float x;
    float y;
} vec2_t;

vec2_t vec2_add(vec2_t a, vec2_t b) {
    vec2_t r;
    r.x = a.x + b.x;
    r.y = a.y + b.y;
    return r;
}

vec2_t vec2_sub(vec2_t a, vec2_t b) {
    vec2_t r;
    r.x = a.x - b.x;
    r.y = a.y - b.y;
    return r;
}

vec2_t vec2_scale(vec2_t v, float s) {
    vec2_t r;
    r.x = v.x * s;
    r.y = v.y * s;
    return r;
}

float vec2_dot(vec2_t a, vec2_t b) {
    return a.x * b.x + a.y * b.y;
}

float vec2_length_sq(vec2_t v) {
    return v.x * v.x + v.y * v.y;
}

float vec2_cross(vec2_t a, vec2_t b) {
    return a.x * b.y - a.y * b.x;
}

vec2_t vec2_perp(vec2_t v) {
    vec2_t r;
    r.x = -v.y;
    r.y = v.x;
    return r;
}

vec2_t vec2_lerp(vec2_t a, vec2_t b, float t) {
    vec2_t r;
    r.x = a.x + (b.x - a.x) * t;
    r.y = a.y + (b.y - a.y) * t;
    return r;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C426: 2D vector math operations should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C426: Output should not be empty");
    assert!(
        code.contains("fn vec2_add"),
        "C426: Should contain vec2_add function"
    );
    assert!(
        code.contains("fn vec2_dot"),
        "C426: Should contain vec2_dot function"
    );
    assert!(
        code.contains("fn vec2_lerp"),
        "C426: Should contain vec2_lerp function"
    );
}

#[test]
fn c427_3d_vector_math_cross_product_and_reflect() {
    let c_code = r#"
typedef struct {
    float x;
    float y;
    float z;
} vec3_t;

vec3_t vec3_add(vec3_t a, vec3_t b) {
    vec3_t r;
    r.x = a.x + b.x;
    r.y = a.y + b.y;
    r.z = a.z + b.z;
    return r;
}

vec3_t vec3_sub(vec3_t a, vec3_t b) {
    vec3_t r;
    r.x = a.x - b.x;
    r.y = a.y - b.y;
    r.z = a.z - b.z;
    return r;
}

vec3_t vec3_scale(vec3_t v, float s) {
    vec3_t r;
    r.x = v.x * s;
    r.y = v.y * s;
    r.z = v.z * s;
    return r;
}

float vec3_dot(vec3_t a, vec3_t b) {
    return a.x * b.x + a.y * b.y + a.z * b.z;
}

vec3_t vec3_cross(vec3_t a, vec3_t b) {
    vec3_t r;
    r.x = a.y * b.z - a.z * b.y;
    r.y = a.z * b.x - a.x * b.z;
    r.z = a.x * b.y - a.y * b.x;
    return r;
}

float vec3_length_sq(vec3_t v) {
    return v.x * v.x + v.y * v.y + v.z * v.z;
}

vec3_t vec3_reflect(vec3_t incident, vec3_t normal) {
    float d = 2.0f * vec3_dot(incident, normal);
    vec3_t r;
    r.x = incident.x - d * normal.x;
    r.y = incident.y - d * normal.y;
    r.z = incident.z - d * normal.z;
    return r;
}

vec3_t vec3_negate(vec3_t v) {
    vec3_t r;
    r.x = -v.x;
    r.y = -v.y;
    r.z = -v.z;
    return r;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C427: 3D vector math should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C427: Output should not be empty");
    assert!(
        code.contains("fn vec3_cross"),
        "C427: Should contain vec3_cross function"
    );
    assert!(
        code.contains("fn vec3_reflect"),
        "C427: Should contain vec3_reflect function"
    );
    assert!(
        code.contains("fn vec3_dot"),
        "C427: Should contain vec3_dot function"
    );
}

#[test]
fn c428_4x4_matrix_multiply() {
    let c_code = r#"
typedef struct {
    float m[4][4];
} mat4_t;

void mat4_identity(mat4_t *out) {
    int i;
    int j;
    for (i = 0; i < 4; i++) {
        for (j = 0; j < 4; j++) {
            if (i == j) {
                out->m[i][j] = 1.0f;
            } else {
                out->m[i][j] = 0.0f;
            }
        }
    }
}

void mat4_multiply(mat4_t *out, const mat4_t *a, const mat4_t *b) {
    int i;
    int j;
    int k;
    for (i = 0; i < 4; i++) {
        for (j = 0; j < 4; j++) {
            float sum = 0.0f;
            for (k = 0; k < 4; k++) {
                sum += a->m[i][k] * b->m[k][j];
            }
            out->m[i][j] = sum;
        }
    }
}

void mat4_transpose(mat4_t *out, const mat4_t *in) {
    int i;
    int j;
    for (i = 0; i < 4; i++) {
        for (j = 0; j < 4; j++) {
            out->m[i][j] = in->m[j][i];
        }
    }
}

void mat4_scale(mat4_t *out, float sx, float sy, float sz) {
    mat4_identity(out);
    out->m[0][0] = sx;
    out->m[1][1] = sy;
    out->m[2][2] = sz;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C428: 4x4 matrix multiply should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C428: Output should not be empty");
    assert!(
        code.contains("fn mat4_multiply"),
        "C428: Should contain mat4_multiply function"
    );
    assert!(
        code.contains("fn mat4_identity"),
        "C428: Should contain mat4_identity function"
    );
}

#[test]
fn c429_matrix_vector_transform() {
    let c_code = r#"
typedef struct {
    float x;
    float y;
    float z;
    float w;
} vec4_t;

typedef struct {
    float m[4][4];
} mat4_t;

vec4_t mat4_mul_vec4(const mat4_t *mat, vec4_t v) {
    vec4_t r;
    r.x = mat->m[0][0] * v.x + mat->m[0][1] * v.y + mat->m[0][2] * v.z + mat->m[0][3] * v.w;
    r.y = mat->m[1][0] * v.x + mat->m[1][1] * v.y + mat->m[1][2] * v.z + mat->m[1][3] * v.w;
    r.z = mat->m[2][0] * v.x + mat->m[2][1] * v.y + mat->m[2][2] * v.z + mat->m[2][3] * v.w;
    r.w = mat->m[3][0] * v.x + mat->m[3][1] * v.y + mat->m[3][2] * v.z + mat->m[3][3] * v.w;
    return r;
}

void mat4_translate(mat4_t *out, float tx, float ty, float tz) {
    int i;
    int j;
    for (i = 0; i < 4; i++) {
        for (j = 0; j < 4; j++) {
            if (i == j) {
                out->m[i][j] = 1.0f;
            } else {
                out->m[i][j] = 0.0f;
            }
        }
    }
    out->m[0][3] = tx;
    out->m[1][3] = ty;
    out->m[2][3] = tz;
}

vec4_t transform_point(const mat4_t *mvp, float x, float y, float z) {
    vec4_t point;
    point.x = x;
    point.y = y;
    point.z = z;
    point.w = 1.0f;
    return mat4_mul_vec4(mvp, point);
}

float vec4_dot(vec4_t a, vec4_t b) {
    return a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C429: Matrix-vector transform should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C429: Output should not be empty");
    assert!(
        code.contains("fn mat4_mul_vec4"),
        "C429: Should contain mat4_mul_vec4 function"
    );
    assert!(
        code.contains("fn transform_point"),
        "C429: Should contain transform_point function"
    );
}

#[test]
fn c430_quaternion_operations() {
    let c_code = r#"
typedef struct {
    float w;
    float x;
    float y;
    float z;
} quat_t;

quat_t quat_identity(void) {
    quat_t q;
    q.w = 1.0f;
    q.x = 0.0f;
    q.y = 0.0f;
    q.z = 0.0f;
    return q;
}

quat_t quat_multiply(quat_t a, quat_t b) {
    quat_t r;
    r.w = a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z;
    r.x = a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y;
    r.y = a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x;
    r.z = a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w;
    return r;
}

float quat_length_sq(quat_t q) {
    return q.w * q.w + q.x * q.x + q.y * q.y + q.z * q.z;
}

quat_t quat_conjugate(quat_t q) {
    quat_t r;
    r.w = q.w;
    r.x = -q.x;
    r.y = -q.y;
    r.z = -q.z;
    return r;
}

float quat_dot(quat_t a, quat_t b) {
    return a.w * b.w + a.x * b.x + a.y * b.y + a.z * b.z;
}

quat_t quat_lerp(quat_t a, quat_t b, float t) {
    quat_t r;
    float one_minus_t = 1.0f - t;
    r.w = one_minus_t * a.w + t * b.w;
    r.x = one_minus_t * a.x + t * b.x;
    r.y = one_minus_t * a.y + t * b.y;
    r.z = one_minus_t * a.z + t * b.z;
    return r;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C430: Quaternion operations should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C430: Output should not be empty");
    assert!(
        code.contains("fn quat_multiply"),
        "C430: Should contain quat_multiply function"
    );
    assert!(
        code.contains("fn quat_conjugate"),
        "C430: Should contain quat_conjugate function"
    );
    assert!(
        code.contains("fn quat_dot"),
        "C430: Should contain quat_dot function"
    );
}

// ============================================================================
// C431-C435: Spatial Algorithms (AABB, Ray-Sphere, Bresenham, Raster, Z-Buffer)
// ============================================================================

#[test]
fn c431_aabb_collision_detection() {
    let c_code = r#"
typedef struct {
    float min_x;
    float min_y;
    float min_z;
    float max_x;
    float max_y;
    float max_z;
} aabb_t;

int aabb_intersects(const aabb_t *a, const aabb_t *b) {
    if (a->max_x < b->min_x || a->min_x > b->max_x) return 0;
    if (a->max_y < b->min_y || a->min_y > b->max_y) return 0;
    if (a->max_z < b->min_z || a->min_z > b->max_z) return 0;
    return 1;
}

int aabb_contains_point(const aabb_t *box, float px, float py, float pz) {
    if (px < box->min_x || px > box->max_x) return 0;
    if (py < box->min_y || py > box->max_y) return 0;
    if (pz < box->min_z || pz > box->max_z) return 0;
    return 1;
}

void aabb_merge(aabb_t *out, const aabb_t *a, const aabb_t *b) {
    out->min_x = (a->min_x < b->min_x) ? a->min_x : b->min_x;
    out->min_y = (a->min_y < b->min_y) ? a->min_y : b->min_y;
    out->min_z = (a->min_z < b->min_z) ? a->min_z : b->min_z;
    out->max_x = (a->max_x > b->max_x) ? a->max_x : b->max_x;
    out->max_y = (a->max_y > b->max_y) ? a->max_y : b->max_y;
    out->max_z = (a->max_z > b->max_z) ? a->max_z : b->max_z;
}

float aabb_volume(const aabb_t *box) {
    float dx = box->max_x - box->min_x;
    float dy = box->max_y - box->min_y;
    float dz = box->max_z - box->min_z;
    return dx * dy * dz;
}

float aabb_surface_area(const aabb_t *box) {
    float dx = box->max_x - box->min_x;
    float dy = box->max_y - box->min_y;
    float dz = box->max_z - box->min_z;
    return 2.0f * (dx * dy + dy * dz + dz * dx);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C431: AABB collision detection should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C431: Output should not be empty");
    assert!(
        code.contains("fn aabb_intersects"),
        "C431: Should contain aabb_intersects function"
    );
    assert!(
        code.contains("fn aabb_merge"),
        "C431: Should contain aabb_merge function"
    );
}

#[test]
fn c432_ray_sphere_intersection() {
    let c_code = r#"
typedef struct {
    float x;
    float y;
    float z;
} vec3_t;

typedef struct {
    vec3_t origin;
    vec3_t direction;
} ray_t;

typedef struct {
    vec3_t center;
    float radius;
} sphere_t;

float vec3_dot_rs(vec3_t a, vec3_t b) {
    return a.x * b.x + a.y * b.y + a.z * b.z;
}

vec3_t vec3_sub_rs(vec3_t a, vec3_t b) {
    vec3_t r;
    r.x = a.x - b.x;
    r.y = a.y - b.y;
    r.z = a.z - b.z;
    return r;
}

int ray_sphere_intersect(const ray_t *ray, const sphere_t *sphere, float *t_near, float *t_far) {
    vec3_t oc = vec3_sub_rs(ray->origin, sphere->center);
    float a = vec3_dot_rs(ray->direction, ray->direction);
    float b = 2.0f * vec3_dot_rs(oc, ray->direction);
    float c = vec3_dot_rs(oc, oc) - sphere->radius * sphere->radius;
    float discriminant = b * b - 4.0f * a * c;

    if (discriminant < 0.0f) return 0;

    float inv_2a = 1.0f / (2.0f * a);
    float t0;
    float t1;

    if (discriminant == 0.0f) {
        t0 = -b * inv_2a;
        t1 = t0;
    } else {
        float sqrt_disc = discriminant;
        int i;
        for (i = 0; i < 8; i++) {
            sqrt_disc = 0.5f * (sqrt_disc + discriminant / sqrt_disc);
        }
        t0 = (-b - sqrt_disc) * inv_2a;
        t1 = (-b + sqrt_disc) * inv_2a;
    }

    if (t0 > t1) {
        float tmp = t0;
        t0 = t1;
        t1 = tmp;
    }

    *t_near = t0;
    *t_far = t1;
    return 1;
}

vec3_t ray_at(const ray_t *ray, float t) {
    vec3_t r;
    r.x = ray->origin.x + t * ray->direction.x;
    r.y = ray->origin.y + t * ray->direction.y;
    r.z = ray->origin.z + t * ray->direction.z;
    return r;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C432: Ray-sphere intersection should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C432: Output should not be empty");
    assert!(
        code.contains("fn ray_sphere_intersect"),
        "C432: Should contain ray_sphere_intersect function"
    );
    assert!(
        code.contains("fn ray_at"),
        "C432: Should contain ray_at function"
    );
}

#[test]
fn c433_bresenham_line_drawing() {
    let c_code = r#"
#define SCREEN_W 320
#define SCREEN_H 240

typedef unsigned char uint8_t;

typedef struct {
    uint8_t pixels[SCREEN_H][SCREEN_W];
    int width;
    int height;
} framebuffer_t;

void fb_clear(framebuffer_t *fb, uint8_t color) {
    int y;
    int x;
    for (y = 0; y < fb->height; y++) {
        for (x = 0; x < fb->width; x++) {
            fb->pixels[y][x] = color;
        }
    }
}

void fb_set_pixel(framebuffer_t *fb, int x, int y, uint8_t color) {
    if (x >= 0 && x < fb->width && y >= 0 && y < fb->height) {
        fb->pixels[y][x] = color;
    }
}

static int abs_val(int x) {
    return (x < 0) ? -x : x;
}

void bresenham_line(framebuffer_t *fb, int x0, int y0, int x1, int y1, uint8_t color) {
    int dx = abs_val(x1 - x0);
    int dy = -abs_val(y1 - y0);
    int sx = (x0 < x1) ? 1 : -1;
    int sy = (y0 < y1) ? 1 : -1;
    int err = dx + dy;

    while (1) {
        fb_set_pixel(fb, x0, y0, color);
        if (x0 == x1 && y0 == y1) break;
        int e2 = 2 * err;
        if (e2 >= dy) {
            err += dy;
            x0 += sx;
        }
        if (e2 <= dx) {
            err += dx;
            y0 += sy;
        }
    }
}

void draw_rect(framebuffer_t *fb, int x, int y, int w, int h, uint8_t color) {
    bresenham_line(fb, x, y, x + w, y, color);
    bresenham_line(fb, x + w, y, x + w, y + h, color);
    bresenham_line(fb, x + w, y + h, x, y + h, color);
    bresenham_line(fb, x, y + h, x, y, color);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C433: Bresenham line drawing should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C433: Output should not be empty");
    assert!(
        code.contains("fn bresenham_line"),
        "C433: Should contain bresenham_line function"
    );
    assert!(
        code.contains("fn fb_set_pixel"),
        "C433: Should contain fb_set_pixel function"
    );
}

#[test]
fn c434_triangle_rasterization_barycentric() {
    let c_code = r#"
typedef struct {
    float x;
    float y;
} point2d_t;

typedef struct {
    int width;
    int height;
    float depth[256][256];
    unsigned char color[256][256];
} canvas_t;

float edge_function(point2d_t a, point2d_t b, point2d_t c) {
    return (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x);
}

static int min_int(int a, int b) {
    return (a < b) ? a : b;
}

static int max_int(int a, int b) {
    return (a > b) ? a : b;
}

static int clamp_int(int v, int lo, int hi) {
    if (v < lo) return lo;
    if (v > hi) return hi;
    return v;
}

void rasterize_triangle(canvas_t *canvas, point2d_t v0, point2d_t v1, point2d_t v2, unsigned char shade) {
    float area = edge_function(v0, v1, v2);
    if (area <= 0.0f) return;
    float inv_area = 1.0f / area;

    int min_x = clamp_int((int)v0.x, 0, canvas->width - 1);
    int min_y = clamp_int((int)v0.y, 0, canvas->height - 1);
    int max_x = clamp_int((int)v0.x, 0, canvas->width - 1);
    int max_y = clamp_int((int)v0.y, 0, canvas->height - 1);

    int tmp_x = clamp_int((int)v1.x, 0, canvas->width - 1);
    int tmp_y = clamp_int((int)v1.y, 0, canvas->height - 1);
    min_x = min_int(min_x, tmp_x);
    min_y = min_int(min_y, tmp_y);
    max_x = max_int(max_x, tmp_x);
    max_y = max_int(max_y, tmp_y);

    tmp_x = clamp_int((int)v2.x, 0, canvas->width - 1);
    tmp_y = clamp_int((int)v2.y, 0, canvas->height - 1);
    min_x = min_int(min_x, tmp_x);
    min_y = min_int(min_y, tmp_y);
    max_x = max_int(max_x, tmp_x);
    max_y = max_int(max_y, tmp_y);

    int y;
    int x;
    for (y = min_y; y <= max_y; y++) {
        for (x = min_x; x <= max_x; x++) {
            point2d_t p;
            p.x = (float)x + 0.5f;
            p.y = (float)y + 0.5f;
            float w0 = edge_function(v1, v2, p) * inv_area;
            float w1 = edge_function(v2, v0, p) * inv_area;
            float w2 = edge_function(v0, v1, p) * inv_area;
            if (w0 >= 0.0f && w1 >= 0.0f && w2 >= 0.0f) {
                canvas->color[y][x] = shade;
            }
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C434: Triangle rasterization should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C434: Output should not be empty");
    assert!(
        code.contains("fn rasterize_triangle"),
        "C434: Should contain rasterize_triangle function"
    );
    assert!(
        code.contains("fn edge_function"),
        "C434: Should contain edge_function function"
    );
}

#[test]
fn c435_zbuffer_depth_testing() {
    let c_code = r#"
#define ZB_WIDTH 128
#define ZB_HEIGHT 128

typedef unsigned char uint8_t;

typedef struct {
    float depth[ZB_HEIGHT][ZB_WIDTH];
    uint8_t color_r[ZB_HEIGHT][ZB_WIDTH];
    uint8_t color_g[ZB_HEIGHT][ZB_WIDTH];
    uint8_t color_b[ZB_HEIGHT][ZB_WIDTH];
    int width;
    int height;
} zbuffer_t;

void zbuffer_clear(zbuffer_t *zb) {
    int y;
    int x;
    for (y = 0; y < zb->height; y++) {
        for (x = 0; x < zb->width; x++) {
            zb->depth[y][x] = 1.0f;
            zb->color_r[y][x] = 0;
            zb->color_g[y][x] = 0;
            zb->color_b[y][x] = 0;
        }
    }
}

int zbuffer_test_and_write(zbuffer_t *zb, int x, int y, float z,
                            uint8_t r, uint8_t g, uint8_t b) {
    if (x < 0 || x >= zb->width || y < 0 || y >= zb->height) return 0;
    if (z >= zb->depth[y][x]) return 0;

    zb->depth[y][x] = z;
    zb->color_r[y][x] = r;
    zb->color_g[y][x] = g;
    zb->color_b[y][x] = b;
    return 1;
}

float zbuffer_read_depth(const zbuffer_t *zb, int x, int y) {
    if (x < 0 || x >= zb->width || y < 0 || y >= zb->height) return 1.0f;
    return zb->depth[y][x];
}

int zbuffer_is_visible(const zbuffer_t *zb, int x, int y, float z) {
    if (x < 0 || x >= zb->width || y < 0 || y >= zb->height) return 0;
    return z < zb->depth[y][x];
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C435: Z-buffer depth testing should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C435: Output should not be empty");
    assert!(
        code.contains("fn zbuffer_test_and_write"),
        "C435: Should contain zbuffer_test_and_write function"
    );
    assert!(
        code.contains("fn zbuffer_clear"),
        "C435: Should contain zbuffer_clear function"
    );
}

// ============================================================================
// C436-C440: Rendering Pipeline (Alpha Blend, Frustum, UV Atlas, Tilemap, Particles)
// ============================================================================

#[test]
fn c436_color_blending_alpha_compositing() {
    let c_code = r#"
typedef unsigned char uint8_t;

typedef struct {
    uint8_t r;
    uint8_t g;
    uint8_t b;
    uint8_t a;
} color_t;

static uint8_t blend_channel(uint8_t src, uint8_t dst, uint8_t alpha) {
    int result = ((int)src * (int)alpha + (int)dst * (255 - (int)alpha)) / 255;
    if (result > 255) result = 255;
    if (result < 0) result = 0;
    return (uint8_t)result;
}

color_t color_alpha_blend(color_t src, color_t dst) {
    color_t out;
    out.r = blend_channel(src.r, dst.r, src.a);
    out.g = blend_channel(src.g, dst.g, src.a);
    out.b = blend_channel(src.b, dst.b, src.a);
    out.a = (uint8_t)(((int)src.a * 255 + (int)dst.a * (255 - (int)src.a)) / 255);
    return out;
}

color_t color_additive_blend(color_t src, color_t dst) {
    color_t out;
    int r = (int)src.r + (int)dst.r;
    int g = (int)src.g + (int)dst.g;
    int b = (int)src.b + (int)dst.b;
    out.r = (uint8_t)((r > 255) ? 255 : r);
    out.g = (uint8_t)((g > 255) ? 255 : g);
    out.b = (uint8_t)((b > 255) ? 255 : b);
    out.a = src.a;
    return out;
}

color_t color_multiply(color_t a, color_t b) {
    color_t out;
    out.r = (uint8_t)(((int)a.r * (int)b.r) / 255);
    out.g = (uint8_t)(((int)a.g * (int)b.g) / 255);
    out.b = (uint8_t)(((int)a.b * (int)b.b) / 255);
    out.a = (uint8_t)(((int)a.a * (int)b.a) / 255);
    return out;
}

color_t color_lerp(color_t a, color_t b, int t) {
    color_t out;
    out.r = (uint8_t)(((int)a.r * (255 - t) + (int)b.r * t) / 255);
    out.g = (uint8_t)(((int)a.g * (255 - t) + (int)b.g * t) / 255);
    out.b = (uint8_t)(((int)a.b * (255 - t) + (int)b.b * t) / 255);
    out.a = (uint8_t)(((int)a.a * (255 - t) + (int)b.a * t) / 255);
    return out;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C436: Color blending alpha compositing should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C436: Output should not be empty");
    assert!(
        code.contains("fn color_alpha_blend"),
        "C436: Should contain color_alpha_blend function"
    );
    assert!(
        code.contains("fn color_additive_blend"),
        "C436: Should contain color_additive_blend function"
    );
}

#[test]
fn c437_frustum_culling_plane_point_test() {
    let c_code = r#"
typedef struct {
    float x;
    float y;
    float z;
} vec3f_t;

typedef struct {
    float a;
    float b;
    float c;
    float d;
} plane_t;

#define NUM_FRUSTUM_PLANES 6

typedef struct {
    plane_t planes[NUM_FRUSTUM_PLANES];
} frustum_t;

float plane_distance_to_point(const plane_t *plane, vec3f_t point) {
    return plane->a * point.x + plane->b * point.y + plane->c * point.z + plane->d;
}

int point_in_frustum(const frustum_t *frustum, vec3f_t point) {
    int i;
    for (i = 0; i < NUM_FRUSTUM_PLANES; i++) {
        if (plane_distance_to_point(&frustum->planes[i], point) < 0.0f) {
            return 0;
        }
    }
    return 1;
}

int sphere_in_frustum(const frustum_t *frustum, vec3f_t center, float radius) {
    int i;
    for (i = 0; i < NUM_FRUSTUM_PLANES; i++) {
        float dist = plane_distance_to_point(&frustum->planes[i], center);
        if (dist < -radius) {
            return 0;
        }
    }
    return 1;
}

void plane_normalize(plane_t *p) {
    float len_sq = p->a * p->a + p->b * p->b + p->c * p->c;
    float inv_len;
    int i;
    float approx = len_sq;
    for (i = 0; i < 5; i++) {
        approx = 0.5f * (approx + len_sq / approx);
    }
    inv_len = 1.0f / approx;
    p->a *= inv_len;
    p->b *= inv_len;
    p->c *= inv_len;
    p->d *= inv_len;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C437: Frustum culling should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C437: Output should not be empty");
    assert!(
        code.contains("fn point_in_frustum"),
        "C437: Should contain point_in_frustum function"
    );
    assert!(
        code.contains("fn sphere_in_frustum"),
        "C437: Should contain sphere_in_frustum function"
    );
}

#[test]
fn c438_sprite_atlas_uv_coordinate_mapping() {
    let c_code = r#"
typedef struct {
    float u0;
    float v0;
    float u1;
    float v1;
} uv_rect_t;

typedef struct {
    int atlas_width;
    int atlas_height;
    int sprite_width;
    int sprite_height;
    int columns;
    int rows;
} sprite_atlas_t;

void atlas_init(sprite_atlas_t *atlas, int aw, int ah, int sw, int sh) {
    atlas->atlas_width = aw;
    atlas->atlas_height = ah;
    atlas->sprite_width = sw;
    atlas->sprite_height = sh;
    atlas->columns = aw / sw;
    atlas->rows = ah / sh;
}

uv_rect_t atlas_get_uv(const sprite_atlas_t *atlas, int index) {
    uv_rect_t uv;
    int col = index % atlas->columns;
    int row = index / atlas->columns;
    float inv_w = 1.0f / (float)atlas->atlas_width;
    float inv_h = 1.0f / (float)atlas->atlas_height;
    uv.u0 = (float)(col * atlas->sprite_width) * inv_w;
    uv.v0 = (float)(row * atlas->sprite_height) * inv_h;
    uv.u1 = (float)((col + 1) * atlas->sprite_width) * inv_w;
    uv.v1 = (float)((row + 1) * atlas->sprite_height) * inv_h;
    return uv;
}

int atlas_total_sprites(const sprite_atlas_t *atlas) {
    return atlas->columns * atlas->rows;
}

uv_rect_t atlas_get_uv_by_rowcol(const sprite_atlas_t *atlas, int row, int col) {
    int index = row * atlas->columns + col;
    return atlas_get_uv(atlas, index);
}

float uv_width(const uv_rect_t *uv) {
    return uv->u1 - uv->u0;
}

float uv_height(const uv_rect_t *uv) {
    return uv->v1 - uv->v0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C438: Sprite atlas UV mapping should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C438: Output should not be empty");
    assert!(
        code.contains("fn atlas_get_uv"),
        "C438: Should contain atlas_get_uv function"
    );
    assert!(
        code.contains("fn atlas_init"),
        "C438: Should contain atlas_init function"
    );
}

#[test]
fn c439_tile_map_renderer_2d_grid() {
    let c_code = r#"
#define TILE_SIZE 16
#define MAP_WIDTH 32
#define MAP_HEIGHT 32
#define TILE_EMPTY 0
#define TILE_WALL 1
#define TILE_FLOOR 2
#define TILE_WATER 3

typedef struct {
    int tiles[MAP_HEIGHT][MAP_WIDTH];
    int width;
    int height;
} tilemap_t;

void tilemap_init(tilemap_t *map) {
    int y;
    int x;
    map->width = MAP_WIDTH;
    map->height = MAP_HEIGHT;
    for (y = 0; y < MAP_HEIGHT; y++) {
        for (x = 0; x < MAP_WIDTH; x++) {
            map->tiles[y][x] = TILE_EMPTY;
        }
    }
}

void tilemap_set(tilemap_t *map, int x, int y, int tile_id) {
    if (x >= 0 && x < map->width && y >= 0 && y < map->height) {
        map->tiles[y][x] = tile_id;
    }
}

int tilemap_get(const tilemap_t *map, int x, int y) {
    if (x < 0 || x >= map->width || y < 0 || y >= map->height) {
        return TILE_EMPTY;
    }
    return map->tiles[y][x];
}

int tilemap_is_solid(const tilemap_t *map, int x, int y) {
    int tile = tilemap_get(map, x, y);
    return tile == TILE_WALL;
}

int tilemap_pixel_to_tile_x(int pixel_x) {
    return pixel_x / TILE_SIZE;
}

int tilemap_pixel_to_tile_y(int pixel_y) {
    return pixel_y / TILE_SIZE;
}

int tilemap_count_neighbors(const tilemap_t *map, int x, int y, int tile_id) {
    int count = 0;
    if (tilemap_get(map, x - 1, y) == tile_id) count++;
    if (tilemap_get(map, x + 1, y) == tile_id) count++;
    if (tilemap_get(map, x, y - 1) == tile_id) count++;
    if (tilemap_get(map, x, y + 1) == tile_id) count++;
    return count;
}

void tilemap_fill_rect(tilemap_t *map, int x0, int y0, int w, int h, int tile_id) {
    int y;
    int x;
    for (y = y0; y < y0 + h; y++) {
        for (x = x0; x < x0 + w; x++) {
            tilemap_set(map, x, y, tile_id);
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C439: Tile map renderer should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C439: Output should not be empty");
    assert!(
        code.contains("fn tilemap_get"),
        "C439: Should contain tilemap_get function"
    );
    assert!(
        code.contains("fn tilemap_is_solid"),
        "C439: Should contain tilemap_is_solid function"
    );
}

#[test]
fn c440_particle_system_spawn_update_age() {
    let c_code = r#"
#define MAX_PARTICLES 256

typedef struct {
    float x;
    float y;
    float vx;
    float vy;
    float life;
    float max_life;
    int active;
} particle_t;

typedef struct {
    particle_t particles[MAX_PARTICLES];
    int count;
    float spawn_rate;
    float gravity;
} particle_system_t;

void psys_init(particle_system_t *ps, float rate, float grav) {
    int i;
    ps->count = 0;
    ps->spawn_rate = rate;
    ps->gravity = grav;
    for (i = 0; i < MAX_PARTICLES; i++) {
        ps->particles[i].active = 0;
    }
}

int psys_find_inactive(const particle_system_t *ps) {
    int i;
    for (i = 0; i < MAX_PARTICLES; i++) {
        if (!ps->particles[i].active) return i;
    }
    return -1;
}

int psys_spawn(particle_system_t *ps, float x, float y, float vx, float vy, float life) {
    int idx = psys_find_inactive(ps);
    if (idx < 0) return 0;

    ps->particles[idx].x = x;
    ps->particles[idx].y = y;
    ps->particles[idx].vx = vx;
    ps->particles[idx].vy = vy;
    ps->particles[idx].life = life;
    ps->particles[idx].max_life = life;
    ps->particles[idx].active = 1;
    ps->count++;
    return 1;
}

void psys_update(particle_system_t *ps, float dt) {
    int i;
    for (i = 0; i < MAX_PARTICLES; i++) {
        if (!ps->particles[i].active) continue;
        ps->particles[i].vy += ps->gravity * dt;
        ps->particles[i].x += ps->particles[i].vx * dt;
        ps->particles[i].y += ps->particles[i].vy * dt;
        ps->particles[i].life -= dt;
        if (ps->particles[i].life <= 0.0f) {
            ps->particles[i].active = 0;
            ps->count--;
        }
    }
}

int psys_active_count(const particle_system_t *ps) {
    return ps->count;
}

float psys_particle_alpha(const particle_t *p) {
    if (!p->active || p->max_life <= 0.0f) return 0.0f;
    return p->life / p->max_life;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C440: Particle system should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C440: Output should not be empty");
    assert!(
        code.contains("fn psys_spawn"),
        "C440: Should contain psys_spawn function"
    );
    assert!(
        code.contains("fn psys_update"),
        "C440: Should contain psys_update function"
    );
}

// ============================================================================
// C441-C445: Curves and Spatial Structures
// ============================================================================

#[test]
fn c441_camera_perspective_projection() {
    let c_code = r#"
typedef struct {
    float m[4][4];
} mat4_t;

void mat4_zero(mat4_t *out) {
    int i;
    int j;
    for (i = 0; i < 4; i++) {
        for (j = 0; j < 4; j++) {
            out->m[i][j] = 0.0f;
        }
    }
}

void perspective_projection(mat4_t *out, float fov_rad, float aspect,
                             float near_plane, float far_plane) {
    float half_fov;
    float top;
    float right;
    float depth;
    int i;
    float approx;

    mat4_zero(out);

    approx = fov_rad * 0.5f;
    half_fov = approx - (approx * approx * approx) / 6.0f;
    top = near_plane * half_fov;
    right = top * aspect;
    depth = far_plane - near_plane;

    if (right != 0.0f) {
        out->m[0][0] = near_plane / right;
    }
    if (top != 0.0f) {
        out->m[1][1] = near_plane / top;
    }
    if (depth != 0.0f) {
        out->m[2][2] = -(far_plane + near_plane) / depth;
        out->m[2][3] = -2.0f * far_plane * near_plane / depth;
    }
    out->m[3][2] = -1.0f;
}

void orthographic_projection(mat4_t *out, float left, float right_val,
                               float bottom, float top,
                               float near_plane, float far_plane) {
    float rl = right_val - left;
    float tb = top - bottom;
    float fn = far_plane - near_plane;
    mat4_zero(out);
    if (rl != 0.0f) out->m[0][0] = 2.0f / rl;
    if (tb != 0.0f) out->m[1][1] = 2.0f / tb;
    if (fn != 0.0f) out->m[2][2] = -2.0f / fn;
    out->m[0][3] = -(right_val + left) / rl;
    out->m[1][3] = -(top + bottom) / tb;
    out->m[2][3] = -(far_plane + near_plane) / fn;
    out->m[3][3] = 1.0f;
}

void viewport_transform(float *screen_x, float *screen_y,
                         float ndc_x, float ndc_y, int width, int height) {
    *screen_x = (ndc_x + 1.0f) * 0.5f * (float)width;
    *screen_y = (1.0f - ndc_y) * 0.5f * (float)height;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C441: Camera perspective projection should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C441: Output should not be empty");
    assert!(
        code.contains("fn perspective_projection"),
        "C441: Should contain perspective_projection function"
    );
    assert!(
        code.contains("fn viewport_transform"),
        "C441: Should contain viewport_transform function"
    );
}

#[test]
fn c442_bezier_curve_evaluation_cubic() {
    let c_code = r#"
typedef struct {
    float x;
    float y;
} point2d_t;

point2d_t bezier_cubic(point2d_t p0, point2d_t p1, point2d_t p2, point2d_t p3, float t) {
    float u = 1.0f - t;
    float uu = u * u;
    float uuu = uu * u;
    float tt = t * t;
    float ttt = tt * t;

    point2d_t result;
    result.x = uuu * p0.x + 3.0f * uu * t * p1.x + 3.0f * u * tt * p2.x + ttt * p3.x;
    result.y = uuu * p0.y + 3.0f * uu * t * p1.y + 3.0f * u * tt * p2.y + ttt * p3.y;
    return result;
}

point2d_t bezier_cubic_tangent(point2d_t p0, point2d_t p1, point2d_t p2, point2d_t p3, float t) {
    float u = 1.0f - t;
    float uu = u * u;
    float tt = t * t;

    point2d_t tangent;
    tangent.x = 3.0f * uu * (p1.x - p0.x) + 6.0f * u * t * (p2.x - p1.x) + 3.0f * tt * (p3.x - p2.x);
    tangent.y = 3.0f * uu * (p1.y - p0.y) + 6.0f * u * t * (p2.y - p1.y) + 3.0f * tt * (p3.y - p2.y);
    return tangent;
}

point2d_t bezier_quadratic(point2d_t p0, point2d_t p1, point2d_t p2, float t) {
    float u = 1.0f - t;
    point2d_t result;
    result.x = u * u * p0.x + 2.0f * u * t * p1.x + t * t * p2.x;
    result.y = u * u * p0.y + 2.0f * u * t * p1.y + t * t * p2.y;
    return result;
}

float bezier_arc_length_approx(point2d_t p0, point2d_t p1, point2d_t p2, point2d_t p3, int segments) {
    float length = 0.0f;
    point2d_t prev = p0;
    int i;
    for (i = 1; i <= segments; i++) {
        float t = (float)i / (float)segments;
        point2d_t cur = bezier_cubic(p0, p1, p2, p3, t);
        float dx = cur.x - prev.x;
        float dy = cur.y - prev.y;
        float dist_sq = dx * dx + dy * dy;
        float approx = dist_sq;
        int j;
        for (j = 0; j < 5; j++) {
            approx = 0.5f * (approx + dist_sq / approx);
        }
        length += approx;
        prev = cur;
    }
    return length;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C442: Bezier curve evaluation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C442: Output should not be empty");
    assert!(
        code.contains("fn bezier_cubic"),
        "C442: Should contain bezier_cubic function"
    );
    assert!(
        code.contains("fn bezier_cubic_tangent"),
        "C442: Should contain bezier_cubic_tangent function"
    );
}

#[test]
fn c443_catmull_rom_spline_interpolation() {
    let c_code = r#"
typedef struct {
    float x;
    float y;
} spline_point_t;

spline_point_t catmull_rom(spline_point_t p0, spline_point_t p1,
                           spline_point_t p2, spline_point_t p3, float t) {
    float t2 = t * t;
    float t3 = t2 * t;

    spline_point_t result;
    result.x = 0.5f * ((2.0f * p1.x) +
               (-p0.x + p2.x) * t +
               (2.0f * p0.x - 5.0f * p1.x + 4.0f * p2.x - p3.x) * t2 +
               (-p0.x + 3.0f * p1.x - 3.0f * p2.x + p3.x) * t3);
    result.y = 0.5f * ((2.0f * p1.y) +
               (-p0.y + p2.y) * t +
               (2.0f * p0.y - 5.0f * p1.y + 4.0f * p2.y - p3.y) * t2 +
               (-p0.y + 3.0f * p1.y - 3.0f * p2.y + p3.y) * t3);
    return result;
}

spline_point_t catmull_rom_tangent(spline_point_t p0, spline_point_t p1,
                                    spline_point_t p2, spline_point_t p3, float t) {
    float t2 = t * t;
    spline_point_t tangent;
    tangent.x = 0.5f * ((-p0.x + p2.x) +
                2.0f * (2.0f * p0.x - 5.0f * p1.x + 4.0f * p2.x - p3.x) * t +
                3.0f * (-p0.x + 3.0f * p1.x - 3.0f * p2.x + p3.x) * t2);
    tangent.y = 0.5f * ((-p0.y + p2.y) +
                2.0f * (2.0f * p0.y - 5.0f * p1.y + 4.0f * p2.y - p3.y) * t +
                3.0f * (-p0.y + 3.0f * p1.y - 3.0f * p2.y + p3.y) * t2);
    return tangent;
}

float spline_length_approx(spline_point_t p0, spline_point_t p1,
                            spline_point_t p2, spline_point_t p3, int segments) {
    float length = 0.0f;
    spline_point_t prev = catmull_rom(p0, p1, p2, p3, 0.0f);
    int i;
    for (i = 1; i <= segments; i++) {
        float t = (float)i / (float)segments;
        spline_point_t cur = catmull_rom(p0, p1, p2, p3, t);
        float dx = cur.x - prev.x;
        float dy = cur.y - prev.y;
        length += dx * dx + dy * dy;
        prev = cur;
    }
    return length;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C443: Catmull-Rom spline interpolation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C443: Output should not be empty");
    assert!(
        code.contains("fn catmull_rom"),
        "C443: Should contain catmull_rom function"
    );
    assert!(
        code.contains("fn catmull_rom_tangent"),
        "C443: Should contain catmull_rom_tangent function"
    );
}

#[test]
fn c444_spatial_hash_grid_broad_phase_collision() {
    let c_code = r#"
#define GRID_SIZE 16
#define GRID_CELLS (GRID_SIZE * GRID_SIZE)
#define MAX_OBJECTS_PER_CELL 8
#define MAX_OBJECTS 128

typedef struct {
    float x;
    float y;
    float half_w;
    float half_h;
    int id;
} spatial_object_t;

typedef struct {
    int object_ids[MAX_OBJECTS_PER_CELL];
    int count;
} grid_cell_t;

typedef struct {
    grid_cell_t cells[GRID_CELLS];
    float cell_size;
    float inv_cell_size;
} spatial_hash_t;

void spatial_hash_init(spatial_hash_t *grid, float cell_size) {
    int i;
    grid->cell_size = cell_size;
    grid->inv_cell_size = 1.0f / cell_size;
    for (i = 0; i < GRID_CELLS; i++) {
        grid->cells[i].count = 0;
    }
}

void spatial_hash_clear(spatial_hash_t *grid) {
    int i;
    for (i = 0; i < GRID_CELLS; i++) {
        grid->cells[i].count = 0;
    }
}

static int grid_cell_index(const spatial_hash_t *grid, float x, float y) {
    int cx = (int)(x * grid->inv_cell_size);
    int cy = (int)(y * grid->inv_cell_size);
    if (cx < 0) cx = 0;
    if (cx >= GRID_SIZE) cx = GRID_SIZE - 1;
    if (cy < 0) cy = 0;
    if (cy >= GRID_SIZE) cy = GRID_SIZE - 1;
    return cy * GRID_SIZE + cx;
}

void spatial_hash_insert(spatial_hash_t *grid, const spatial_object_t *obj) {
    int idx = grid_cell_index(grid, obj->x, obj->y);
    grid_cell_t *cell = &grid->cells[idx];
    if (cell->count < MAX_OBJECTS_PER_CELL) {
        cell->object_ids[cell->count] = obj->id;
        cell->count++;
    }
}

int spatial_hash_query(const spatial_hash_t *grid, float x, float y,
                       int *results, int max_results) {
    int idx = grid_cell_index(grid, x, y);
    const grid_cell_t *cell = &grid->cells[idx];
    int count = cell->count;
    if (count > max_results) count = max_results;
    int i;
    for (i = 0; i < count; i++) {
        results[i] = cell->object_ids[i];
    }
    return count;
}

int spatial_hash_cell_count(const spatial_hash_t *grid, int cell_idx) {
    if (cell_idx < 0 || cell_idx >= GRID_CELLS) return 0;
    return grid->cells[cell_idx].count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C444: Spatial hash grid should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C444: Output should not be empty");
    assert!(
        code.contains("fn spatial_hash_insert"),
        "C444: Should contain spatial_hash_insert function"
    );
    assert!(
        code.contains("fn spatial_hash_query"),
        "C444: Should contain spatial_hash_query function"
    );
}

#[test]
fn c445_vertex_buffer_interleaved_attributes() {
    let c_code = r#"
#define MAX_VERTICES 1024
#define ATTR_POSITION 0
#define ATTR_NORMAL 1
#define ATTR_TEXCOORD 2

typedef struct {
    float px;
    float py;
    float pz;
    float nx;
    float ny;
    float nz;
    float u;
    float v;
} vertex_t;

typedef struct {
    vertex_t data[MAX_VERTICES];
    int count;
    int capacity;
} vertex_buffer_t;

void vbuf_init(vertex_buffer_t *vb) {
    vb->count = 0;
    vb->capacity = MAX_VERTICES;
}

int vbuf_push(vertex_buffer_t *vb, float px, float py, float pz,
              float nx, float ny, float nz, float u, float v) {
    if (vb->count >= vb->capacity) return -1;
    int idx = vb->count;
    vb->data[idx].px = px;
    vb->data[idx].py = py;
    vb->data[idx].pz = pz;
    vb->data[idx].nx = nx;
    vb->data[idx].ny = ny;
    vb->data[idx].nz = nz;
    vb->data[idx].u = u;
    vb->data[idx].v = v;
    vb->count++;
    return idx;
}

float vbuf_get_position_x(const vertex_buffer_t *vb, int index) {
    if (index < 0 || index >= vb->count) return 0.0f;
    return vb->data[index].px;
}

float vbuf_get_position_y(const vertex_buffer_t *vb, int index) {
    if (index < 0 || index >= vb->count) return 0.0f;
    return vb->data[index].py;
}

float vbuf_get_position_z(const vertex_buffer_t *vb, int index) {
    if (index < 0 || index >= vb->count) return 0.0f;
    return vb->data[index].pz;
}

int vbuf_size(const vertex_buffer_t *vb) {
    return vb->count;
}

int vbuf_stride(void) {
    return 8;
}

void vbuf_clear(vertex_buffer_t *vb) {
    vb->count = 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C445: Vertex buffer layout should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C445: Output should not be empty");
    assert!(
        code.contains("fn vbuf_push"),
        "C445: Should contain vbuf_push function"
    );
    assert!(
        code.contains("fn vbuf_init"),
        "C445: Should contain vbuf_init function"
    );
}

// ============================================================================
// C446-C450: Advanced Rendering
// ============================================================================

#[test]
fn c446_normal_map_tangent_space_calculation() {
    let c_code = r#"
typedef struct {
    float x;
    float y;
    float z;
} vec3n_t;

typedef struct {
    float x;
    float y;
} vec2n_t;

typedef struct {
    vec3n_t tangent;
    vec3n_t bitangent;
} tangent_result_t;

vec3n_t vec3n_sub(vec3n_t a, vec3n_t b) {
    vec3n_t r;
    r.x = a.x - b.x;
    r.y = a.y - b.y;
    r.z = a.z - b.z;
    return r;
}

vec2n_t vec2n_sub(vec2n_t a, vec2n_t b) {
    vec2n_t r;
    r.x = a.x - b.x;
    r.y = a.y - b.y;
    return r;
}

tangent_result_t compute_tangent_space(vec3n_t pos0, vec3n_t pos1, vec3n_t pos2,
                                       vec2n_t uv0, vec2n_t uv1, vec2n_t uv2) {
    tangent_result_t result;
    vec3n_t edge1 = vec3n_sub(pos1, pos0);
    vec3n_t edge2 = vec3n_sub(pos2, pos0);
    vec2n_t duv1 = vec2n_sub(uv1, uv0);
    vec2n_t duv2 = vec2n_sub(uv2, uv0);

    float det = duv1.x * duv2.y - duv2.x * duv1.y;
    float inv_det;
    if (det != 0.0f) {
        inv_det = 1.0f / det;
    } else {
        inv_det = 0.0f;
    }

    result.tangent.x = inv_det * (duv2.y * edge1.x - duv1.y * edge2.x);
    result.tangent.y = inv_det * (duv2.y * edge1.y - duv1.y * edge2.y);
    result.tangent.z = inv_det * (duv2.y * edge1.z - duv1.y * edge2.z);

    result.bitangent.x = inv_det * (-duv2.x * edge1.x + duv1.x * edge2.x);
    result.bitangent.y = inv_det * (-duv2.x * edge1.y + duv1.x * edge2.y);
    result.bitangent.z = inv_det * (-duv2.x * edge1.z + duv1.x * edge2.z);

    return result;
}

float vec3n_dot(vec3n_t a, vec3n_t b) {
    return a.x * b.x + a.y * b.y + a.z * b.z;
}

vec3n_t vec3n_scale(vec3n_t v, float s) {
    vec3n_t r;
    r.x = v.x * s;
    r.y = v.y * s;
    r.z = v.z * s;
    return r;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C446: Normal map tangent space should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C446: Output should not be empty");
    assert!(
        code.contains("fn compute_tangent_space"),
        "C446: Should contain compute_tangent_space function"
    );
    assert!(
        code.contains("fn vec3n_dot"),
        "C446: Should contain vec3n_dot function"
    );
}

#[test]
fn c447_lod_level_of_detail_distance_selector() {
    let c_code = r#"
#define LOD_LEVELS 4
#define MAX_LOD_OBJECTS 64

typedef struct {
    float x;
    float y;
    float z;
} vec3_lod_t;

typedef struct {
    float distance_thresholds[LOD_LEVELS];
    int triangle_counts[LOD_LEVELS];
    int num_levels;
} lod_config_t;

typedef struct {
    vec3_lod_t position;
    float bounding_radius;
    int current_lod;
    int visible;
} lod_object_t;

void lod_config_init(lod_config_t *config) {
    config->num_levels = LOD_LEVELS;
    config->distance_thresholds[0] = 10.0f;
    config->distance_thresholds[1] = 30.0f;
    config->distance_thresholds[2] = 80.0f;
    config->distance_thresholds[3] = 200.0f;
    config->triangle_counts[0] = 5000;
    config->triangle_counts[1] = 2000;
    config->triangle_counts[2] = 500;
    config->triangle_counts[3] = 100;
}

float vec3_lod_dist_sq(vec3_lod_t a, vec3_lod_t b) {
    float dx = a.x - b.x;
    float dy = a.y - b.y;
    float dz = a.z - b.z;
    return dx * dx + dy * dy + dz * dz;
}

int lod_select_level(const lod_config_t *config, float distance_sq) {
    int i;
    for (i = 0; i < config->num_levels; i++) {
        float threshold = config->distance_thresholds[i];
        if (distance_sq < threshold * threshold) {
            return i;
        }
    }
    return config->num_levels - 1;
}

void lod_update_object(lod_object_t *obj, const lod_config_t *config, vec3_lod_t camera_pos) {
    float dist_sq = vec3_lod_dist_sq(obj->position, camera_pos);
    obj->current_lod = lod_select_level(config, dist_sq);
    float max_dist = config->distance_thresholds[config->num_levels - 1];
    obj->visible = (dist_sq < max_dist * max_dist) ? 1 : 0;
}

int lod_total_triangles(const lod_object_t *objects, int count, const lod_config_t *config) {
    int total = 0;
    int i;
    for (i = 0; i < count; i++) {
        if (objects[i].visible) {
            total += config->triangle_counts[objects[i].current_lod];
        }
    }
    return total;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C447: LOD distance selector should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C447: Output should not be empty");
    assert!(
        code.contains("fn lod_select_level"),
        "C447: Should contain lod_select_level function"
    );
    assert!(
        code.contains("fn lod_update_object"),
        "C447: Should contain lod_update_object function"
    );
}

#[test]
fn c448_ssao_kernel_generation() {
    let c_code = r#"
#define SSAO_KERNEL_SIZE 16

typedef struct {
    float x;
    float y;
    float z;
} ssao_sample_t;

typedef struct {
    ssao_sample_t samples[SSAO_KERNEL_SIZE];
    int count;
    float radius;
    float bias;
} ssao_kernel_t;

static float pseudo_random(int seed) {
    int x = seed;
    x = ((x >> 16) ^ x) * 0x45d9f3b;
    x = ((x >> 16) ^ x) * 0x45d9f3b;
    x = (x >> 16) ^ x;
    float result = (float)(x & 0x7FFFFFFF) / (float)0x7FFFFFFF;
    return result;
}

void ssao_kernel_init(ssao_kernel_t *kernel, float radius, float bias) {
    int i;
    kernel->count = SSAO_KERNEL_SIZE;
    kernel->radius = radius;
    kernel->bias = bias;

    for (i = 0; i < SSAO_KERNEL_SIZE; i++) {
        float rx = pseudo_random(i * 3) * 2.0f - 1.0f;
        float ry = pseudo_random(i * 3 + 1) * 2.0f - 1.0f;
        float rz = pseudo_random(i * 3 + 2);

        float len_sq = rx * rx + ry * ry + rz * rz;
        float inv_len = 1.0f;
        if (len_sq > 0.001f) {
            float approx = len_sq;
            int j;
            for (j = 0; j < 5; j++) {
                approx = 0.5f * (approx + len_sq / approx);
            }
            inv_len = 1.0f / approx;
        }

        kernel->samples[i].x = rx * inv_len;
        kernel->samples[i].y = ry * inv_len;
        kernel->samples[i].z = rz * inv_len;

        float scale = (float)i / (float)SSAO_KERNEL_SIZE;
        scale = 0.1f + scale * scale * 0.9f;
        kernel->samples[i].x *= scale * radius;
        kernel->samples[i].y *= scale * radius;
        kernel->samples[i].z *= scale * radius;
    }
}

float ssao_compute_occlusion(const ssao_kernel_t *kernel, float depth_at_sample,
                              float sample_depth, float range_check_sq) {
    float diff = depth_at_sample - sample_depth;
    if (diff < kernel->bias) return 0.0f;
    if (diff * diff > range_check_sq) return 0.0f;
    return 1.0f;
}

float ssao_average_occlusion(const float *occlusion_values, int count) {
    float sum = 0.0f;
    int i;
    for (i = 0; i < count; i++) {
        sum += occlusion_values[i];
    }
    if (count > 0) {
        return 1.0f - (sum / (float)count);
    }
    return 1.0f;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C448: SSAO kernel generation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C448: Output should not be empty");
    assert!(
        code.contains("fn ssao_kernel_init"),
        "C448: Should contain ssao_kernel_init function"
    );
    assert!(
        code.contains("fn ssao_compute_occlusion"),
        "C448: Should contain ssao_compute_occlusion function"
    );
}

#[test]
fn c449_shadow_map_depth_comparison() {
    let c_code = r#"
#define SHADOW_MAP_SIZE 256

typedef struct {
    float depth[SHADOW_MAP_SIZE][SHADOW_MAP_SIZE];
    int width;
    int height;
    float bias;
} shadow_map_t;

void shadow_map_init(shadow_map_t *sm) {
    int y;
    int x;
    sm->width = SHADOW_MAP_SIZE;
    sm->height = SHADOW_MAP_SIZE;
    sm->bias = 0.005f;
    for (y = 0; y < SHADOW_MAP_SIZE; y++) {
        for (x = 0; x < SHADOW_MAP_SIZE; x++) {
            sm->depth[y][x] = 1.0f;
        }
    }
}

void shadow_map_write(shadow_map_t *sm, int x, int y, float depth) {
    if (x >= 0 && x < sm->width && y >= 0 && y < sm->height) {
        if (depth < sm->depth[y][x]) {
            sm->depth[y][x] = depth;
        }
    }
}

float shadow_map_read(const shadow_map_t *sm, int x, int y) {
    if (x < 0 || x >= sm->width || y < 0 || y >= sm->height) return 1.0f;
    return sm->depth[y][x];
}

float shadow_test(const shadow_map_t *sm, float light_x, float light_y, float frag_depth) {
    int sx = (int)(light_x * (float)sm->width);
    int sy = (int)(light_y * (float)sm->height);
    float stored_depth = shadow_map_read(sm, sx, sy);
    if (frag_depth - sm->bias > stored_depth) {
        return 0.3f;
    }
    return 1.0f;
}

float shadow_pcf_3x3(const shadow_map_t *sm, float light_x, float light_y, float frag_depth) {
    float shadow = 0.0f;
    int sx = (int)(light_x * (float)sm->width);
    int sy = (int)(light_y * (float)sm->height);
    int dy;
    int dx;
    int sample_count = 0;
    for (dy = -1; dy <= 1; dy++) {
        for (dx = -1; dx <= 1; dx++) {
            float stored = shadow_map_read(sm, sx + dx, sy + dy);
            if (frag_depth - sm->bias > stored) {
                shadow += 0.3f;
            } else {
                shadow += 1.0f;
            }
            sample_count++;
        }
    }
    return shadow / (float)sample_count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C449: Shadow map depth comparison should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C449: Output should not be empty");
    assert!(
        code.contains("fn shadow_test"),
        "C449: Should contain shadow_test function"
    );
    assert!(
        code.contains("fn shadow_pcf_3x3"),
        "C449: Should contain shadow_pcf_3x3 function"
    );
}

#[test]
fn c450_skeletal_animation_bone_transform() {
    let c_code = r#"
#define MAX_BONES 32

typedef struct {
    float m[4][4];
} bone_mat_t;

typedef struct {
    float x;
    float y;
    float z;
} bone_vec3_t;

typedef struct {
    bone_mat_t local_transform;
    bone_mat_t world_transform;
    int parent_index;
    int id;
} bone_t;

typedef struct {
    bone_t bones[MAX_BONES];
    int num_bones;
} skeleton_t;

void bone_mat_identity(bone_mat_t *out) {
    int i;
    int j;
    for (i = 0; i < 4; i++) {
        for (j = 0; j < 4; j++) {
            out->m[i][j] = (i == j) ? 1.0f : 0.0f;
        }
    }
}

void bone_mat_multiply(bone_mat_t *out, const bone_mat_t *a, const bone_mat_t *b) {
    int i;
    int j;
    int k;
    bone_mat_t tmp;
    for (i = 0; i < 4; i++) {
        for (j = 0; j < 4; j++) {
            float sum = 0.0f;
            for (k = 0; k < 4; k++) {
                sum += a->m[i][k] * b->m[k][j];
            }
            tmp.m[i][j] = sum;
        }
    }
    int ii;
    int jj;
    for (ii = 0; ii < 4; ii++) {
        for (jj = 0; jj < 4; jj++) {
            out->m[ii][jj] = tmp.m[ii][jj];
        }
    }
}

void skeleton_init(skeleton_t *skel) {
    int i;
    skel->num_bones = 0;
    for (i = 0; i < MAX_BONES; i++) {
        bone_mat_identity(&skel->bones[i].local_transform);
        bone_mat_identity(&skel->bones[i].world_transform);
        skel->bones[i].parent_index = -1;
        skel->bones[i].id = i;
    }
}

int skeleton_add_bone(skeleton_t *skel, int parent_idx) {
    if (skel->num_bones >= MAX_BONES) return -1;
    int idx = skel->num_bones;
    skel->bones[idx].parent_index = parent_idx;
    skel->bones[idx].id = idx;
    bone_mat_identity(&skel->bones[idx].local_transform);
    bone_mat_identity(&skel->bones[idx].world_transform);
    skel->num_bones++;
    return idx;
}

void skeleton_update_world_transforms(skeleton_t *skel) {
    int i;
    for (i = 0; i < skel->num_bones; i++) {
        if (skel->bones[i].parent_index < 0) {
            int r;
            int c;
            for (r = 0; r < 4; r++) {
                for (c = 0; c < 4; c++) {
                    skel->bones[i].world_transform.m[r][c] =
                        skel->bones[i].local_transform.m[r][c];
                }
            }
        } else {
            bone_mat_multiply(
                &skel->bones[i].world_transform,
                &skel->bones[skel->bones[i].parent_index].world_transform,
                &skel->bones[i].local_transform
            );
        }
    }
}

bone_vec3_t skeleton_transform_point(const skeleton_t *skel, int bone_idx,
                                      float x, float y, float z) {
    bone_vec3_t result;
    const bone_mat_t *m = &skel->bones[bone_idx].world_transform;
    result.x = m->m[0][0] * x + m->m[0][1] * y + m->m[0][2] * z + m->m[0][3];
    result.y = m->m[1][0] * x + m->m[1][1] * y + m->m[1][2] * z + m->m[1][3];
    result.z = m->m[2][0] * x + m->m[2][1] * y + m->m[2][2] * z + m->m[2][3];
    return result;
}

void bone_set_translation(bone_t *bone, float tx, float ty, float tz) {
    bone->local_transform.m[0][3] = tx;
    bone->local_transform.m[1][3] = ty;
    bone->local_transform.m[2][3] = tz;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C450: Skeletal animation bone transform should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C450: Output should not be empty");
    assert!(
        code.contains("fn skeleton_update_world_transforms"),
        "C450: Should contain skeleton_update_world_transforms function"
    );
    assert!(
        code.contains("fn skeleton_transform_point"),
        "C450: Should contain skeleton_transform_point function"
    );
    assert!(
        code.contains("fn bone_mat_multiply"),
        "C450: Should contain bone_mat_multiply function"
    );
}
