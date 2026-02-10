//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C651-C675: Game Engine and Simulation patterns -- the kind of C code found
//! in game engines, physics simulations, spatial partitioning, AI systems, and
//! real-time interactive applications.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world game engine patterns commonly found in
//! Quake, Doom, Unity internals, Godot, Box2D, Recast/Detour, and similar
//! game engines -- all expressed as valid C99.
//!
//! Organization:
//! - C651-C655: Core systems (ECS, quadtree, collision, physics, particles)
//! - C656-C660: Rendering & input (tilemap, sprite anim, input FSM, camera, pathfinding)
//! - C661-C665: AI & architecture (behavior tree, FSM, events, resource mgr, scene graph)
//! - C666-C670: 3D spatial & animation (octree, frustum cull, LOD, skeletal, navmesh)
//! - C671-C675: Physics & utilities (rigid body, broadphase, audio mixer, RNG, command pattern)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C651-C655: Core Systems (ECS, Quadtree, Collision, Physics, Particles)
// ============================================================================

/// C651: Entity-component system (basic ECS)
#[test]
fn c651_entity_component_system() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    float x;
    float y;
} position_t;

typedef struct {
    float vx;
    float vy;
} velocity_t;

typedef struct {
    uint32_t mask;
    position_t pos;
    velocity_t vel;
    int active;
} entity_t;

void ecs_init(entity_t *entities, int count) {
    for (int i = 0; i < count; i = i + 1) {
        entities[i].mask = 0;
        entities[i].active = 0;
        entities[i].pos.x = 0.0f;
        entities[i].pos.y = 0.0f;
        entities[i].vel.vx = 0.0f;
        entities[i].vel.vy = 0.0f;
    }
}

int ecs_create(entity_t *entities, int max_entities) {
    for (int i = 0; i < max_entities; i = i + 1) {
        if (entities[i].active == 0) {
            entities[i].active = 1;
            entities[i].mask = 0;
            return i;
        }
    }
    return -1;
}

void ecs_destroy(entity_t *entities, int id) {
    entities[id].active = 0;
    entities[id].mask = 0;
}

void ecs_add_position(entity_t *entities, int id, float x, float y) {
    entities[id].pos.x = x;
    entities[id].pos.y = y;
    entities[id].mask = entities[id].mask | 1;
}

void ecs_add_velocity(entity_t *entities, int id, float vx, float vy) {
    entities[id].vel.vx = vx;
    entities[id].vel.vy = vy;
    entities[id].mask = entities[id].mask | 2;
}

void ecs_movement_system(entity_t *entities, int count, float dt) {
    for (int i = 0; i < count; i = i + 1) {
        if (entities[i].active && (entities[i].mask & 3) == 3) {
            entities[i].pos.x += entities[i].vel.vx * dt;
            entities[i].pos.y += entities[i].vel.vy * dt;
        }
    }
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C651: Should produce output");
    assert!(rust_code.contains("fn ecs_init"), "C651: Should contain ecs_init");
    assert!(rust_code.contains("fn ecs_create"), "C651: Should contain ecs_create");
    assert!(
        rust_code.contains("fn ecs_movement_system"),
        "C651: Should contain ecs_movement_system"
    );
    Ok(())
}

/// C652: Spatial partitioning (quadtree insert and query)
#[test]
fn c652_quadtree_spatial_partitioning() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float x;
    float y;
    float w;
    float h;
} rect_t;

typedef struct {
    float x;
    float y;
    int id;
} point_t;

int rect_contains(rect_t *r, float px, float py) {
    if (px >= r->x && px < r->x + r->w &&
        py >= r->y && py < r->y + r->h) {
        return 1;
    }
    return 0;
}

int rect_intersects(rect_t *a, rect_t *b) {
    if (a->x + a->w < b->x) return 0;
    if (a->x > b->x + b->w) return 0;
    if (a->y + a->h < b->y) return 0;
    if (a->y > b->y + b->h) return 0;
    return 1;
}

void quadtree_subdivide(rect_t *parent, rect_t *children) {
    float hw = parent->w / 2.0f;
    float hh = parent->h / 2.0f;
    children[0].x = parent->x;
    children[0].y = parent->y;
    children[0].w = hw;
    children[0].h = hh;
    children[1].x = parent->x + hw;
    children[1].y = parent->y;
    children[1].w = hw;
    children[1].h = hh;
    children[2].x = parent->x;
    children[2].y = parent->y + hh;
    children[2].w = hw;
    children[2].h = hh;
    children[3].x = parent->x + hw;
    children[3].y = parent->y + hh;
    children[3].w = hw;
    children[3].h = hh;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C652: Should produce output");
    assert!(
        rust_code.contains("fn rect_contains"),
        "C652: Should contain rect_contains"
    );
    assert!(
        rust_code.contains("fn quadtree_subdivide"),
        "C652: Should contain quadtree_subdivide"
    );
    Ok(())
}

/// C653: Collision detection (circle-circle)
#[test]
fn c653_circle_collision_detection() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float x;
    float y;
    float radius;
} circle_t;

typedef struct {
    float nx;
    float ny;
    float depth;
    int collided;
} collision_result_t;

float circle_dist_sq(circle_t *a, circle_t *b) {
    float dx = b->x - a->x;
    float dy = b->y - a->y;
    return dx * dx + dy * dy;
}

collision_result_t circle_vs_circle(circle_t *a, circle_t *b) {
    collision_result_t result;
    float dx = b->x - a->x;
    float dy = b->y - a->y;
    float dist_sq = dx * dx + dy * dy;
    float radius_sum = a->radius + b->radius;
    result.collided = 0;
    result.depth = 0.0f;
    result.nx = 0.0f;
    result.ny = 0.0f;
    if (dist_sq < radius_sum * radius_sum) {
        float dist = dist_sq;
        if (dist > 0.0001f) {
            result.nx = dx / dist;
            result.ny = dy / dist;
        } else {
            result.nx = 1.0f;
            result.ny = 0.0f;
        }
        result.depth = radius_sum - dist;
        result.collided = 1;
    }
    return result;
}

void circle_resolve(circle_t *a, circle_t *b, collision_result_t *col) {
    float half = col->depth / 2.0f;
    a->x -= col->nx * half;
    a->y -= col->ny * half;
    b->x += col->nx * half;
    b->y += col->ny * half;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C653: Should produce output");
    assert!(
        rust_code.contains("fn circle_vs_circle"),
        "C653: Should contain circle_vs_circle"
    );
    assert!(
        rust_code.contains("fn circle_resolve"),
        "C653: Should contain circle_resolve"
    );
    Ok(())
}

/// C654: Physics simulation (Verlet integration)
#[test]
fn c654_verlet_integration_physics() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float x;
    float y;
    float prev_x;
    float prev_y;
    float accel_x;
    float accel_y;
    float damping;
} verlet_body_t;

void verlet_init(verlet_body_t *body, float x, float y) {
    body->x = x;
    body->y = y;
    body->prev_x = x;
    body->prev_y = y;
    body->accel_x = 0.0f;
    body->accel_y = 0.0f;
    body->damping = 0.99f;
}

void verlet_apply_force(verlet_body_t *body, float fx, float fy) {
    body->accel_x += fx;
    body->accel_y += fy;
}

void verlet_integrate(verlet_body_t *body, float dt) {
    float vx = (body->x - body->prev_x) * body->damping;
    float vy = (body->y - body->prev_y) * body->damping;
    body->prev_x = body->x;
    body->prev_y = body->y;
    body->x += vx + body->accel_x * dt * dt;
    body->y += vy + body->accel_y * dt * dt;
    body->accel_x = 0.0f;
    body->accel_y = 0.0f;
}

void verlet_constrain_distance(verlet_body_t *a, verlet_body_t *b, float rest_len) {
    float dx = b->x - a->x;
    float dy = b->y - a->y;
    float dist_sq = dx * dx + dy * dy;
    float diff = 0.0f;
    if (dist_sq > 0.0001f) {
        diff = (rest_len * rest_len - dist_sq) / (dist_sq * 2.0f);
    }
    float ox = dx * diff;
    float oy = dy * diff;
    a->x -= ox;
    a->y -= oy;
    b->x += ox;
    b->y += oy;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C654: Should produce output");
    assert!(
        rust_code.contains("fn verlet_init"),
        "C654: Should contain verlet_init"
    );
    assert!(
        rust_code.contains("fn verlet_integrate"),
        "C654: Should contain verlet_integrate"
    );
    assert!(
        rust_code.contains("fn verlet_constrain_distance"),
        "C654: Should contain verlet_constrain_distance"
    );
    Ok(())
}

/// C655: Particle system (basic emitter)
#[test]
fn c655_particle_system_emitter() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
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
    float emit_x;
    float emit_y;
    float spread;
    float speed;
    float lifetime;
} emitter_t;

void particle_init(particle_t *p) {
    p->x = 0.0f;
    p->y = 0.0f;
    p->vx = 0.0f;
    p->vy = 0.0f;
    p->life = 0.0f;
    p->max_life = 1.0f;
    p->active = 0;
}

void particle_emit(particle_t *p, emitter_t *e, float angle) {
    p->x = e->emit_x;
    p->y = e->emit_y;
    p->vx = e->speed * angle;
    p->vy = e->speed * (1.0f - angle);
    p->life = e->lifetime;
    p->max_life = e->lifetime;
    p->active = 1;
}

void particle_update(particle_t *particles, int count, float dt, float gravity) {
    for (int i = 0; i < count; i = i + 1) {
        if (particles[i].active) {
            particles[i].vy += gravity * dt;
            particles[i].x += particles[i].vx * dt;
            particles[i].y += particles[i].vy * dt;
            particles[i].life -= dt;
            if (particles[i].life <= 0.0f) {
                particles[i].active = 0;
            }
        }
    }
}

int particle_count_active(particle_t *particles, int count) {
    int active = 0;
    for (int i = 0; i < count; i = i + 1) {
        if (particles[i].active) {
            active = active + 1;
        }
    }
    return active;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C655: Should produce output");
    assert!(
        rust_code.contains("fn particle_emit"),
        "C655: Should contain particle_emit"
    );
    assert!(
        rust_code.contains("fn particle_update"),
        "C655: Should contain particle_update"
    );
    Ok(())
}

// ============================================================================
// C656-C660: Rendering & Input (Tilemap, Sprite, Input FSM, Camera, A*)
// ============================================================================

/// C656: Tilemap renderer
#[test]
fn c656_tilemap_renderer() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int tile_width;
    int tile_height;
    int map_width;
    int map_height;
} tilemap_t;

int tilemap_get_tile(int *tiles, int map_w, int tx, int ty) {
    if (tx < 0 || ty < 0 || tx >= map_w) return -1;
    return tiles[ty * map_w + tx];
}

void tilemap_set_tile(int *tiles, int map_w, int tx, int ty, int value) {
    if (tx >= 0 && ty >= 0 && tx < map_w) {
        tiles[ty * map_w + tx] = value;
    }
}

int tilemap_world_to_tile_x(tilemap_t *tm, float wx) {
    int tx = (int)(wx / (float)tm->tile_width);
    if (tx < 0) tx = 0;
    if (tx >= tm->map_width) tx = tm->map_width - 1;
    return tx;
}

int tilemap_world_to_tile_y(tilemap_t *tm, float wy) {
    int ty = (int)(wy / (float)tm->tile_height);
    if (ty < 0) ty = 0;
    if (ty >= tm->map_height) ty = tm->map_height - 1;
    return ty;
}

int tilemap_is_solid(int *tiles, int map_w, int tx, int ty) {
    int tile = tilemap_get_tile(tiles, map_w, tx, ty);
    return (tile > 0) ? 1 : 0;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C656: Should produce output");
    assert!(
        rust_code.contains("fn tilemap_get_tile"),
        "C656: Should contain tilemap_get_tile"
    );
    assert!(
        rust_code.contains("fn tilemap_world_to_tile_x"),
        "C656: Should contain tilemap_world_to_tile_x"
    );
    Ok(())
}

/// C657: Sprite animation controller
#[test]
fn c657_sprite_animation_controller() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int start_frame;
    int end_frame;
    float frame_duration;
    int looping;
} animation_t;

typedef struct {
    int current_frame;
    float timer;
    int playing;
    animation_t anim;
} sprite_animator_t;

void animator_play(sprite_animator_t *sa, int start, int end, float dur, int loop_flag) {
    sa->anim.start_frame = start;
    sa->anim.end_frame = end;
    sa->anim.frame_duration = dur;
    sa->anim.looping = loop_flag;
    sa->current_frame = start;
    sa->timer = 0.0f;
    sa->playing = 1;
}

void animator_update(sprite_animator_t *sa, float dt) {
    if (sa->playing == 0) return;
    sa->timer += dt;
    if (sa->timer >= sa->anim.frame_duration) {
        sa->timer -= sa->anim.frame_duration;
        sa->current_frame = sa->current_frame + 1;
        if (sa->current_frame > sa->anim.end_frame) {
            if (sa->anim.looping) {
                sa->current_frame = sa->anim.start_frame;
            } else {
                sa->current_frame = sa->anim.end_frame;
                sa->playing = 0;
            }
        }
    }
}

void animator_stop(sprite_animator_t *sa) {
    sa->playing = 0;
}

int animator_is_finished(sprite_animator_t *sa) {
    if (sa->playing == 0 && sa->current_frame == sa->anim.end_frame) {
        return 1;
    }
    return 0;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C657: Should produce output");
    assert!(
        rust_code.contains("fn animator_play"),
        "C657: Should contain animator_play"
    );
    assert!(
        rust_code.contains("fn animator_update"),
        "C657: Should contain animator_update"
    );
    Ok(())
}

/// C658: Input state machine
#[test]
fn c658_input_state_machine() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int current;
    int previous;
} key_state_t;

typedef struct {
    key_state_t keys[256];
    int mouse_x;
    int mouse_y;
    int mouse_btn;
    int prev_mouse_btn;
} input_state_t;

void input_init(input_state_t *inp) {
    for (int i = 0; i < 256; i = i + 1) {
        inp->keys[i].current = 0;
        inp->keys[i].previous = 0;
    }
    inp->mouse_x = 0;
    inp->mouse_y = 0;
    inp->mouse_btn = 0;
    inp->prev_mouse_btn = 0;
}

void input_update(input_state_t *inp) {
    for (int i = 0; i < 256; i = i + 1) {
        inp->keys[i].previous = inp->keys[i].current;
    }
    inp->prev_mouse_btn = inp->mouse_btn;
}

int input_key_pressed(input_state_t *inp, int key) {
    return (inp->keys[key].current && !inp->keys[key].previous) ? 1 : 0;
}

int input_key_released(input_state_t *inp, int key) {
    return (!inp->keys[key].current && inp->keys[key].previous) ? 1 : 0;
}

int input_key_held(input_state_t *inp, int key) {
    return inp->keys[key].current;
}

int input_mouse_clicked(input_state_t *inp) {
    return (inp->mouse_btn && !inp->prev_mouse_btn) ? 1 : 0;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C658: Should produce output");
    assert!(
        rust_code.contains("fn input_init"),
        "C658: Should contain input_init"
    );
    assert!(
        rust_code.contains("fn input_key_pressed"),
        "C658: Should contain input_key_pressed"
    );
    Ok(())
}

/// C659: Camera follow (smooth)
#[test]
fn c659_smooth_camera_follow() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float x;
    float y;
    float zoom;
    float target_x;
    float target_y;
    float smoothness;
    float bounds_min_x;
    float bounds_min_y;
    float bounds_max_x;
    float bounds_max_y;
} camera_t;

void camera_init(camera_t *cam, float x, float y) {
    cam->x = x;
    cam->y = y;
    cam->zoom = 1.0f;
    cam->target_x = x;
    cam->target_y = y;
    cam->smoothness = 0.1f;
    cam->bounds_min_x = -10000.0f;
    cam->bounds_min_y = -10000.0f;
    cam->bounds_max_x = 10000.0f;
    cam->bounds_max_y = 10000.0f;
}

void camera_set_target(camera_t *cam, float tx, float ty) {
    cam->target_x = tx;
    cam->target_y = ty;
}

void camera_update(camera_t *cam, float dt) {
    float lerp = cam->smoothness;
    cam->x += (cam->target_x - cam->x) * lerp;
    cam->y += (cam->target_y - cam->y) * lerp;
    if (cam->x < cam->bounds_min_x) cam->x = cam->bounds_min_x;
    if (cam->x > cam->bounds_max_x) cam->x = cam->bounds_max_x;
    if (cam->y < cam->bounds_min_y) cam->y = cam->bounds_min_y;
    if (cam->y > cam->bounds_max_y) cam->y = cam->bounds_max_y;
}

float camera_screen_to_world_x(camera_t *cam, float sx, float screen_w) {
    return (sx - screen_w / 2.0f) / cam->zoom + cam->x;
}

float camera_screen_to_world_y(camera_t *cam, float sy, float screen_h) {
    return (sy - screen_h / 2.0f) / cam->zoom + cam->y;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C659: Should produce output");
    assert!(
        rust_code.contains("fn camera_init"),
        "C659: Should contain camera_init"
    );
    assert!(
        rust_code.contains("fn camera_update"),
        "C659: Should contain camera_update"
    );
    Ok(())
}

/// C660: A* pathfinding on tilemap
#[test]
fn c660_astar_pathfinding() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int x;
    int y;
    float g;
    float h;
    float f;
    int parent_x;
    int parent_y;
    int open;
    int closed;
} astar_node_t;

float astar_heuristic(int x1, int y1, int x2, int y2) {
    float dx = (float)(x1 - x2);
    float dy = (float)(y1 - y2);
    if (dx < 0.0f) dx = -dx;
    if (dy < 0.0f) dy = -dy;
    return dx + dy;
}

void astar_init_grid(astar_node_t *grid, int w, int h) {
    for (int y = 0; y < h; y = y + 1) {
        for (int x = 0; x < w; x = x + 1) {
            int idx = y * w + x;
            grid[idx].x = x;
            grid[idx].y = y;
            grid[idx].g = 999999.0f;
            grid[idx].h = 0.0f;
            grid[idx].f = 999999.0f;
            grid[idx].parent_x = -1;
            grid[idx].parent_y = -1;
            grid[idx].open = 0;
            grid[idx].closed = 0;
        }
    }
}

int astar_find_lowest_f(astar_node_t *grid, int total) {
    int best = -1;
    float best_f = 999999.0f;
    for (int i = 0; i < total; i = i + 1) {
        if (grid[i].open && !grid[i].closed) {
            if (grid[i].f < best_f) {
                best_f = grid[i].f;
                best = i;
            }
        }
    }
    return best;
}

void astar_consider_neighbor(astar_node_t *grid, int w, int cur, int nx, int ny, int gx, int gy) {
    int nidx = ny * w + nx;
    if (grid[nidx].closed) return;
    float new_g = grid[cur].g + 1.0f;
    if (new_g < grid[nidx].g) {
        grid[nidx].g = new_g;
        grid[nidx].h = astar_heuristic(nx, ny, gx, gy);
        grid[nidx].f = grid[nidx].g + grid[nidx].h;
        grid[nidx].parent_x = grid[cur].x;
        grid[nidx].parent_y = grid[cur].y;
        grid[nidx].open = 1;
    }
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C660: Should produce output");
    assert!(
        rust_code.contains("fn astar_heuristic"),
        "C660: Should contain astar_heuristic"
    );
    assert!(
        rust_code.contains("fn astar_init_grid"),
        "C660: Should contain astar_init_grid"
    );
    assert!(
        rust_code.contains("fn astar_find_lowest_f"),
        "C660: Should contain astar_find_lowest_f"
    );
    Ok(())
}

// ============================================================================
// C661-C665: AI & Architecture (Behavior Tree, FSM, Events, Resources, Scene Graph)
// ============================================================================

/// C661: Behavior tree (node evaluation)
#[test]
fn c661_behavior_tree_evaluation() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int type;
    int state;
    int child_start;
    int child_count;
    int data;
} bt_node_t;

void bt_init_node(bt_node_t *node, int type, int child_start, int child_count) {
    node->type = type;
    node->state = 0;
    node->child_start = child_start;
    node->child_count = child_count;
    node->data = 0;
}

int bt_eval_leaf(bt_node_t *node) {
    if (node->data > 0) return 1;
    return 0;
}

int bt_eval_sequence(bt_node_t *nodes, int node_idx) {
    bt_node_t *node = &nodes[node_idx];
    for (int i = 0; i < node->child_count; i = i + 1) {
        int child = node->child_start + i;
        int result = bt_eval_leaf(&nodes[child]);
        if (result == 0) {
            node->state = 0;
            return 0;
        }
    }
    node->state = 1;
    return 1;
}

int bt_eval_selector(bt_node_t *nodes, int node_idx) {
    bt_node_t *node = &nodes[node_idx];
    for (int i = 0; i < node->child_count; i = i + 1) {
        int child = node->child_start + i;
        int result = bt_eval_leaf(&nodes[child]);
        if (result == 1) {
            node->state = 1;
            return 1;
        }
    }
    node->state = 0;
    return 0;
}

void bt_reset(bt_node_t *nodes, int count) {
    for (int i = 0; i < count; i = i + 1) {
        nodes[i].state = 0;
    }
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C661: Should produce output");
    assert!(
        rust_code.contains("fn bt_eval_sequence"),
        "C661: Should contain bt_eval_sequence"
    );
    assert!(
        rust_code.contains("fn bt_eval_selector"),
        "C661: Should contain bt_eval_selector"
    );
    Ok(())
}

/// C662: Finite state machine (transitions)
#[test]
fn c662_finite_state_machine() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int from_state;
    int to_state;
    int event;
} transition_t;

typedef struct {
    int current_state;
    transition_t transitions[64];
    int transition_count;
    float state_time;
} fsm_t;

void fsm_init(fsm_t *fsm, int initial_state) {
    fsm->current_state = initial_state;
    fsm->transition_count = 0;
    fsm->state_time = 0.0f;
}

void fsm_add_transition(fsm_t *fsm, int from, int to, int event) {
    if (fsm->transition_count < 64) {
        int idx = fsm->transition_count;
        fsm->transitions[idx].from_state = from;
        fsm->transitions[idx].to_state = to;
        fsm->transitions[idx].event = event;
        fsm->transition_count = fsm->transition_count + 1;
    }
}

int fsm_fire_event(fsm_t *fsm, int event) {
    for (int i = 0; i < fsm->transition_count; i = i + 1) {
        if (fsm->transitions[i].from_state == fsm->current_state &&
            fsm->transitions[i].event == event) {
            fsm->current_state = fsm->transitions[i].to_state;
            fsm->state_time = 0.0f;
            return 1;
        }
    }
    return 0;
}

void fsm_update(fsm_t *fsm, float dt) {
    fsm->state_time += dt;
}

int fsm_get_state(fsm_t *fsm) {
    return fsm->current_state;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C662: Should produce output");
    assert!(
        rust_code.contains("fn fsm_init"),
        "C662: Should contain fsm_init"
    );
    assert!(
        rust_code.contains("fn fsm_fire_event"),
        "C662: Should contain fsm_fire_event"
    );
    Ok(())
}

/// C663: Event system (pub/sub)
#[test]
fn c663_event_system_pubsub() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int event_type;
    int data_a;
    int data_b;
    float data_f;
} event_t;

typedef struct {
    event_t buffer[256];
    int head;
    int tail;
    int count;
} event_queue_t;

void evq_init(event_queue_t *q) {
    q->head = 0;
    q->tail = 0;
    q->count = 0;
}

int evq_push(event_queue_t *q, int type, int da, int db, float df) {
    if (q->count >= 256) return 0;
    int idx = q->tail;
    q->buffer[idx].event_type = type;
    q->buffer[idx].data_a = da;
    q->buffer[idx].data_b = db;
    q->buffer[idx].data_f = df;
    q->tail = (q->tail + 1) % 256;
    q->count = q->count + 1;
    return 1;
}

int evq_pop(event_queue_t *q, event_t *out) {
    if (q->count <= 0) return 0;
    out->event_type = q->buffer[q->head].event_type;
    out->data_a = q->buffer[q->head].data_a;
    out->data_b = q->buffer[q->head].data_b;
    out->data_f = q->buffer[q->head].data_f;
    q->head = (q->head + 1) % 256;
    q->count = q->count - 1;
    return 1;
}

int evq_is_empty(event_queue_t *q) {
    return (q->count == 0) ? 1 : 0;
}

void evq_clear(event_queue_t *q) {
    q->head = 0;
    q->tail = 0;
    q->count = 0;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C663: Should produce output");
    assert!(
        rust_code.contains("fn evq_init"),
        "C663: Should contain evq_init"
    );
    assert!(
        rust_code.contains("fn evq_push"),
        "C663: Should contain evq_push"
    );
    assert!(
        rust_code.contains("fn evq_pop"),
        "C663: Should contain evq_pop"
    );
    Ok(())
}

/// C664: Resource manager (handle-based)
#[test]
fn c664_resource_manager_handles() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    int type;
    int ref_count;
    int loaded;
    uint32_t size;
} resource_t;

typedef struct {
    resource_t slots[512];
    int slot_count;
    uint32_t total_memory;
} resource_mgr_t;

void resmgr_init(resource_mgr_t *mgr) {
    for (int i = 0; i < 512; i = i + 1) {
        mgr->slots[i].type = 0;
        mgr->slots[i].ref_count = 0;
        mgr->slots[i].loaded = 0;
        mgr->slots[i].size = 0;
    }
    mgr->slot_count = 0;
    mgr->total_memory = 0;
}

int resmgr_alloc(resource_mgr_t *mgr, int type, uint32_t size) {
    for (int i = 0; i < 512; i = i + 1) {
        if (mgr->slots[i].loaded == 0) {
            mgr->slots[i].type = type;
            mgr->slots[i].ref_count = 1;
            mgr->slots[i].loaded = 1;
            mgr->slots[i].size = size;
            mgr->slot_count = mgr->slot_count + 1;
            mgr->total_memory += size;
            return i;
        }
    }
    return -1;
}

void resmgr_addref(resource_mgr_t *mgr, int handle) {
    if (handle >= 0 && handle < 512 && mgr->slots[handle].loaded) {
        mgr->slots[handle].ref_count = mgr->slots[handle].ref_count + 1;
    }
}

void resmgr_release(resource_mgr_t *mgr, int handle) {
    if (handle >= 0 && handle < 512 && mgr->slots[handle].loaded) {
        mgr->slots[handle].ref_count = mgr->slots[handle].ref_count - 1;
        if (mgr->slots[handle].ref_count <= 0) {
            mgr->total_memory -= mgr->slots[handle].size;
            mgr->slots[handle].loaded = 0;
            mgr->slots[handle].type = 0;
            mgr->slots[handle].size = 0;
            mgr->slot_count = mgr->slot_count - 1;
        }
    }
}

int resmgr_is_valid(resource_mgr_t *mgr, int handle) {
    if (handle >= 0 && handle < 512 && mgr->slots[handle].loaded) return 1;
    return 0;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C664: Should produce output");
    assert!(
        rust_code.contains("fn resmgr_init"),
        "C664: Should contain resmgr_init"
    );
    assert!(
        rust_code.contains("fn resmgr_alloc"),
        "C664: Should contain resmgr_alloc"
    );
    assert!(
        rust_code.contains("fn resmgr_release"),
        "C664: Should contain resmgr_release"
    );
    Ok(())
}

/// C665: Scene graph (transform hierarchy)
#[test]
fn c665_scene_graph_transform_hierarchy() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float local_x;
    float local_y;
    float local_rotation;
    float local_scale;
    float world_x;
    float world_y;
    float world_rotation;
    float world_scale;
    int parent;
    int active;
} scene_node_t;

void scene_init(scene_node_t *nodes, int count) {
    for (int i = 0; i < count; i = i + 1) {
        nodes[i].local_x = 0.0f;
        nodes[i].local_y = 0.0f;
        nodes[i].local_rotation = 0.0f;
        nodes[i].local_scale = 1.0f;
        nodes[i].world_x = 0.0f;
        nodes[i].world_y = 0.0f;
        nodes[i].world_rotation = 0.0f;
        nodes[i].world_scale = 1.0f;
        nodes[i].parent = -1;
        nodes[i].active = 0;
    }
}

void scene_set_parent(scene_node_t *nodes, int child, int parent) {
    nodes[child].parent = parent;
}

void scene_update_world(scene_node_t *nodes, int count) {
    for (int i = 0; i < count; i = i + 1) {
        if (nodes[i].active == 0) continue;
        if (nodes[i].parent < 0) {
            nodes[i].world_x = nodes[i].local_x;
            nodes[i].world_y = nodes[i].local_y;
            nodes[i].world_rotation = nodes[i].local_rotation;
            nodes[i].world_scale = nodes[i].local_scale;
        } else {
            int p = nodes[i].parent;
            nodes[i].world_x = nodes[p].world_x + nodes[i].local_x * nodes[p].world_scale;
            nodes[i].world_y = nodes[p].world_y + nodes[i].local_y * nodes[p].world_scale;
            nodes[i].world_rotation = nodes[p].world_rotation + nodes[i].local_rotation;
            nodes[i].world_scale = nodes[p].world_scale * nodes[i].local_scale;
        }
    }
}

int scene_create_node(scene_node_t *nodes, int max_nodes) {
    for (int i = 0; i < max_nodes; i = i + 1) {
        if (nodes[i].active == 0) {
            nodes[i].active = 1;
            nodes[i].parent = -1;
            nodes[i].local_scale = 1.0f;
            nodes[i].world_scale = 1.0f;
            return i;
        }
    }
    return -1;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C665: Should produce output");
    assert!(
        rust_code.contains("fn scene_init"),
        "C665: Should contain scene_init"
    );
    assert!(
        rust_code.contains("fn scene_update_world"),
        "C665: Should contain scene_update_world"
    );
    Ok(())
}

// ============================================================================
// C666-C670: 3D Spatial & Animation (Octree, Frustum, LOD, Skeletal, Navmesh)
// ============================================================================

/// C666: Octree (3D spatial partitioning)
#[test]
fn c666_octree_3d_spatial() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float x;
    float y;
    float z;
} vec3_t;

typedef struct {
    float cx;
    float cy;
    float cz;
    float half_size;
} octree_bounds_t;

int octree_contains(octree_bounds_t *b, float px, float py, float pz) {
    if (px < b->cx - b->half_size) return 0;
    if (px > b->cx + b->half_size) return 0;
    if (py < b->cy - b->half_size) return 0;
    if (py > b->cy + b->half_size) return 0;
    if (pz < b->cz - b->half_size) return 0;
    if (pz > b->cz + b->half_size) return 0;
    return 1;
}

int octree_get_child_index(octree_bounds_t *b, float px, float py, float pz) {
    int idx = 0;
    if (px > b->cx) idx = idx | 1;
    if (py > b->cy) idx = idx | 2;
    if (pz > b->cz) idx = idx | 4;
    return idx;
}

void octree_child_bounds(octree_bounds_t *parent, int child_idx, octree_bounds_t *out) {
    float quarter = parent->half_size / 2.0f;
    out->half_size = quarter;
    out->cx = parent->cx + ((child_idx & 1) ? quarter : -quarter);
    out->cy = parent->cy + ((child_idx & 2) ? quarter : -quarter);
    out->cz = parent->cz + ((child_idx & 4) ? quarter : -quarter);
}

float octree_distance_sq(float ax, float ay, float az, float bx, float by, float bz) {
    float dx = ax - bx;
    float dy = ay - by;
    float dz = az - bz;
    return dx * dx + dy * dy + dz * dz;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C666: Should produce output");
    assert!(
        rust_code.contains("fn octree_contains"),
        "C666: Should contain octree_contains"
    );
    assert!(
        rust_code.contains("fn octree_get_child_index"),
        "C666: Should contain octree_get_child_index"
    );
    Ok(())
}

/// C667: Frustum culling
#[test]
fn c667_frustum_culling() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float a;
    float b;
    float c;
    float d;
} plane_t;

typedef struct {
    plane_t planes[6];
} frustum_t;

float plane_dot_point(plane_t *p, float x, float y, float z) {
    return p->a * x + p->b * y + p->c * z + p->d;
}

void plane_normalize(plane_t *p) {
    float len = p->a * p->a + p->b * p->b + p->c * p->c;
    if (len > 0.0001f) {
        float inv = 1.0f / len;
        p->a = p->a * inv;
        p->b = p->b * inv;
        p->c = p->c * inv;
        p->d = p->d * inv;
    }
}

int frustum_test_sphere(frustum_t *f, float cx, float cy, float cz, float radius) {
    for (int i = 0; i < 6; i = i + 1) {
        float dist = plane_dot_point(&f->planes[i], cx, cy, cz);
        if (dist < -radius) return 0;
    }
    return 1;
}

int frustum_test_aabb(frustum_t *f, float minx, float miny, float minz,
                      float maxx, float maxy, float maxz) {
    for (int i = 0; i < 6; i = i + 1) {
        float px = (f->planes[i].a > 0.0f) ? maxx : minx;
        float py = (f->planes[i].b > 0.0f) ? maxy : miny;
        float pz = (f->planes[i].c > 0.0f) ? maxz : minz;
        float d = plane_dot_point(&f->planes[i], px, py, pz);
        if (d < 0.0f) return 0;
    }
    return 1;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C667: Should produce output");
    assert!(
        rust_code.contains("fn frustum_test_sphere"),
        "C667: Should contain frustum_test_sphere"
    );
    assert!(
        rust_code.contains("fn frustum_test_aabb"),
        "C667: Should contain frustum_test_aabb"
    );
    Ok(())
}

/// C668: LOD (level of detail) selection
#[test]
fn c668_lod_level_of_detail() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float distances[4];
    int levels;
    int current_lod;
} lod_selector_t;

void lod_init(lod_selector_t *lod, int levels) {
    lod->levels = levels;
    lod->current_lod = 0;
    lod->distances[0] = 50.0f;
    lod->distances[1] = 150.0f;
    lod->distances[2] = 400.0f;
    lod->distances[3] = 1000.0f;
}

int lod_select(lod_selector_t *lod, float distance) {
    for (int i = 0; i < lod->levels; i = i + 1) {
        if (distance < lod->distances[i]) {
            lod->current_lod = i;
            return i;
        }
    }
    lod->current_lod = lod->levels - 1;
    return lod->levels - 1;
}

float lod_compute_distance(float cx, float cy, float cz,
                           float px, float py, float pz) {
    float dx = cx - px;
    float dy = cy - py;
    float dz = cz - pz;
    return dx * dx + dy * dy + dz * dz;
}

int lod_should_transition(lod_selector_t *lod, int new_lod) {
    if (new_lod != lod->current_lod) return 1;
    return 0;
}

float lod_blend_factor(lod_selector_t *lod, float distance) {
    int level = lod->current_lod;
    if (level >= lod->levels - 1) return 1.0f;
    float near = lod->distances[level];
    float far = lod->distances[level + 1];
    float range = far - near;
    if (range < 0.001f) return 0.0f;
    float t = (distance - near) / range;
    if (t < 0.0f) t = 0.0f;
    if (t > 1.0f) t = 1.0f;
    return t;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C668: Should produce output");
    assert!(
        rust_code.contains("fn lod_init"),
        "C668: Should contain lod_init"
    );
    assert!(
        rust_code.contains("fn lod_select"),
        "C668: Should contain lod_select"
    );
    assert!(
        rust_code.contains("fn lod_blend_factor"),
        "C668: Should contain lod_blend_factor"
    );
    Ok(())
}

/// C669: Skeletal animation (bone hierarchy)
#[test]
fn c669_skeletal_animation_bones() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float x;
    float y;
    float rotation;
    float scale;
    int parent;
} bone_t;

typedef struct {
    float x;
    float y;
    float rotation;
    float scale;
} bone_pose_t;

void bone_init(bone_t *bones, int count) {
    for (int i = 0; i < count; i = i + 1) {
        bones[i].x = 0.0f;
        bones[i].y = 0.0f;
        bones[i].rotation = 0.0f;
        bones[i].scale = 1.0f;
        bones[i].parent = -1;
    }
}

void bone_set_parent(bone_t *bones, int child, int parent) {
    bones[child].parent = parent;
}

void bone_compute_world(bone_t *bones, bone_pose_t *world, int count) {
    for (int i = 0; i < count; i = i + 1) {
        if (bones[i].parent < 0) {
            world[i].x = bones[i].x;
            world[i].y = bones[i].y;
            world[i].rotation = bones[i].rotation;
            world[i].scale = bones[i].scale;
        } else {
            int p = bones[i].parent;
            world[i].scale = world[p].scale * bones[i].scale;
            world[i].rotation = world[p].rotation + bones[i].rotation;
            world[i].x = world[p].x + bones[i].x * world[p].scale;
            world[i].y = world[p].y + bones[i].y * world[p].scale;
        }
    }
}

void bone_lerp_pose(bone_pose_t *out, bone_pose_t *a, bone_pose_t *b, int count, float t) {
    for (int i = 0; i < count; i = i + 1) {
        out[i].x = a[i].x + (b[i].x - a[i].x) * t;
        out[i].y = a[i].y + (b[i].y - a[i].y) * t;
        out[i].rotation = a[i].rotation + (b[i].rotation - a[i].rotation) * t;
        out[i].scale = a[i].scale + (b[i].scale - a[i].scale) * t;
    }
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C669: Should produce output");
    assert!(
        rust_code.contains("fn bone_init"),
        "C669: Should contain bone_init"
    );
    assert!(
        rust_code.contains("fn bone_compute_world"),
        "C669: Should contain bone_compute_world"
    );
    assert!(
        rust_code.contains("fn bone_lerp_pose"),
        "C669: Should contain bone_lerp_pose"
    );
    Ok(())
}

/// C670: Navmesh (triangle-based navigation)
#[test]
fn c670_navmesh_triangle_navigation() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float x;
    float y;
} nav_point_t;

typedef struct {
    int v0;
    int v1;
    int v2;
    int adj0;
    int adj1;
    int adj2;
    float cx;
    float cy;
} nav_tri_t;

float nav_cross_2d(float ax, float ay, float bx, float by) {
    return ax * by - ay * bx;
}

int nav_point_in_triangle(nav_point_t *verts, int v0, int v1, int v2,
                          float px, float py) {
    float d0 = nav_cross_2d(verts[v1].x - verts[v0].x, verts[v1].y - verts[v0].y,
                            px - verts[v0].x, py - verts[v0].y);
    float d1 = nav_cross_2d(verts[v2].x - verts[v1].x, verts[v2].y - verts[v1].y,
                            px - verts[v1].x, py - verts[v1].y);
    float d2 = nav_cross_2d(verts[v0].x - verts[v2].x, verts[v0].y - verts[v2].y,
                            px - verts[v2].x, py - verts[v2].y);
    int has_neg = (d0 < 0.0f) || (d1 < 0.0f) || (d2 < 0.0f);
    int has_pos = (d0 > 0.0f) || (d1 > 0.0f) || (d2 > 0.0f);
    return !(has_neg && has_pos);
}

void nav_compute_centroid(nav_point_t *verts, nav_tri_t *tri) {
    tri->cx = (verts[tri->v0].x + verts[tri->v1].x + verts[tri->v2].x) / 3.0f;
    tri->cy = (verts[tri->v0].y + verts[tri->v1].y + verts[tri->v2].y) / 3.0f;
}

int nav_find_triangle(nav_point_t *verts, nav_tri_t *tris, int tri_count,
                      float px, float py) {
    for (int i = 0; i < tri_count; i = i + 1) {
        if (nav_point_in_triangle(verts, tris[i].v0, tris[i].v1, tris[i].v2, px, py)) {
            return i;
        }
    }
    return -1;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C670: Should produce output");
    assert!(
        rust_code.contains("fn nav_point_in_triangle"),
        "C670: Should contain nav_point_in_triangle"
    );
    assert!(
        rust_code.contains("fn nav_find_triangle"),
        "C670: Should contain nav_find_triangle"
    );
    Ok(())
}

// ============================================================================
// C671-C675: Physics & Utilities (Rigid Body, Broadphase, Audio, RNG, Commands)
// ============================================================================

/// C671: Rigid body dynamics
#[test]
fn c671_rigid_body_dynamics() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float x;
    float y;
    float vx;
    float vy;
    float angle;
    float angular_vel;
    float mass;
    float inv_mass;
    float restitution;
} rigid_body_t;

void rb_init(rigid_body_t *rb, float x, float y, float mass) {
    rb->x = x;
    rb->y = y;
    rb->vx = 0.0f;
    rb->vy = 0.0f;
    rb->angle = 0.0f;
    rb->angular_vel = 0.0f;
    rb->mass = mass;
    rb->inv_mass = (mass > 0.0f) ? 1.0f / mass : 0.0f;
    rb->restitution = 0.5f;
}

void rb_apply_force(rigid_body_t *rb, float fx, float fy) {
    rb->vx += fx * rb->inv_mass;
    rb->vy += fy * rb->inv_mass;
}

void rb_apply_impulse(rigid_body_t *rb, float ix, float iy) {
    rb->vx += ix * rb->inv_mass;
    rb->vy += iy * rb->inv_mass;
}

void rb_integrate(rigid_body_t *rb, float dt) {
    rb->x += rb->vx * dt;
    rb->y += rb->vy * dt;
    rb->angle += rb->angular_vel * dt;
}

void rb_resolve_collision(rigid_body_t *a, rigid_body_t *b,
                          float nx, float ny) {
    float rvx = b->vx - a->vx;
    float rvy = b->vy - a->vy;
    float vel_along_normal = rvx * nx + rvy * ny;
    if (vel_along_normal > 0.0f) return;
    float e = a->restitution;
    if (b->restitution < e) e = b->restitution;
    float j = -(1.0f + e) * vel_along_normal;
    j = j / (a->inv_mass + b->inv_mass);
    a->vx -= j * nx * a->inv_mass;
    a->vy -= j * ny * a->inv_mass;
    b->vx += j * nx * b->inv_mass;
    b->vy += j * ny * b->inv_mass;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C671: Should produce output");
    assert!(
        rust_code.contains("fn rb_init"),
        "C671: Should contain rb_init"
    );
    assert!(
        rust_code.contains("fn rb_resolve_collision"),
        "C671: Should contain rb_resolve_collision"
    );
    Ok(())
}

/// C672: Broadphase collision (grid-based)
#[test]
fn c672_broadphase_collision_grid() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int cell_size;
    int grid_width;
    int grid_height;
    int cells[4096];
    int counts[4096];
} spatial_grid_t;

void grid_init(spatial_grid_t *g, int cell_size, int width, int height) {
    g->cell_size = cell_size;
    g->grid_width = width / cell_size;
    g->grid_height = height / cell_size;
    for (int i = 0; i < 4096; i = i + 1) {
        g->cells[i] = -1;
        g->counts[i] = 0;
    }
}

void grid_clear(spatial_grid_t *g) {
    int total = g->grid_width * g->grid_height;
    for (int i = 0; i < total; i = i + 1) {
        g->cells[i] = -1;
        g->counts[i] = 0;
    }
}

int grid_hash(spatial_grid_t *g, float x, float y) {
    int cx = (int)(x / (float)g->cell_size);
    int cy = (int)(y / (float)g->cell_size);
    if (cx < 0) cx = 0;
    if (cy < 0) cy = 0;
    if (cx >= g->grid_width) cx = g->grid_width - 1;
    if (cy >= g->grid_height) cy = g->grid_height - 1;
    return cy * g->grid_width + cx;
}

void grid_insert(spatial_grid_t *g, float x, float y, int entity_id) {
    int cell = grid_hash(g, x, y);
    if (g->counts[cell] == 0) {
        g->cells[cell] = entity_id;
    }
    g->counts[cell] = g->counts[cell] + 1;
}

int grid_query(spatial_grid_t *g, float x, float y) {
    int cell = grid_hash(g, x, y);
    return g->counts[cell];
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C672: Should produce output");
    assert!(
        rust_code.contains("fn grid_init"),
        "C672: Should contain grid_init"
    );
    assert!(
        rust_code.contains("fn grid_hash"),
        "C672: Should contain grid_hash"
    );
    assert!(
        rust_code.contains("fn grid_insert"),
        "C672: Should contain grid_insert"
    );
    Ok(())
}

/// C673: Audio mixer (channel-based)
#[test]
fn c673_audio_mixer_channels() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float volume;
    float pan;
    int playing;
    int looping;
    float position;
    float pitch;
} audio_channel_t;

typedef struct {
    audio_channel_t channels[16];
    float master_volume;
    int active_channels;
} audio_mixer_t;

void mixer_init(audio_mixer_t *m) {
    m->master_volume = 1.0f;
    m->active_channels = 0;
    for (int i = 0; i < 16; i = i + 1) {
        m->channels[i].volume = 1.0f;
        m->channels[i].pan = 0.0f;
        m->channels[i].playing = 0;
        m->channels[i].looping = 0;
        m->channels[i].position = 0.0f;
        m->channels[i].pitch = 1.0f;
    }
}

int mixer_play(audio_mixer_t *m, float vol, float pan) {
    for (int i = 0; i < 16; i = i + 1) {
        if (m->channels[i].playing == 0) {
            m->channels[i].volume = vol;
            m->channels[i].pan = pan;
            m->channels[i].playing = 1;
            m->channels[i].position = 0.0f;
            m->channels[i].pitch = 1.0f;
            m->active_channels = m->active_channels + 1;
            return i;
        }
    }
    return -1;
}

void mixer_stop(audio_mixer_t *m, int channel) {
    if (channel >= 0 && channel < 16 && m->channels[channel].playing) {
        m->channels[channel].playing = 0;
        m->active_channels = m->active_channels - 1;
    }
}

void mixer_update(audio_mixer_t *m, float dt) {
    for (int i = 0; i < 16; i = i + 1) {
        if (m->channels[i].playing) {
            m->channels[i].position += dt * m->channels[i].pitch;
        }
    }
}

float mixer_get_left(audio_mixer_t *m, int ch) {
    float p = m->channels[ch].pan;
    float left = (1.0f - p) * 0.5f + 0.5f;
    return m->channels[ch].volume * left * m->master_volume;
}

float mixer_get_right(audio_mixer_t *m, int ch) {
    float p = m->channels[ch].pan;
    float right = (1.0f + p) * 0.5f + 0.5f;
    return m->channels[ch].volume * right * m->master_volume;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C673: Should produce output");
    assert!(
        rust_code.contains("fn mixer_init"),
        "C673: Should contain mixer_init"
    );
    assert!(
        rust_code.contains("fn mixer_play"),
        "C673: Should contain mixer_play"
    );
    assert!(
        rust_code.contains("fn mixer_update"),
        "C673: Should contain mixer_update"
    );
    Ok(())
}

/// C674: Random number generator (LCG + PCG)
#[test]
fn c674_random_number_generators() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

typedef struct {
    uint32_t state;
} lcg_rng_t;

typedef struct {
    uint64_t state;
    uint64_t inc;
} pcg_rng_t;

void lcg_seed(lcg_rng_t *rng, uint32_t seed) {
    rng->state = seed;
}

uint32_t lcg_next(lcg_rng_t *rng) {
    rng->state = rng->state * 1664525 + 1013904223;
    return rng->state;
}

float lcg_float(lcg_rng_t *rng) {
    uint32_t val = lcg_next(rng);
    return (float)(val & 0x7FFFFF) / (float)0x7FFFFF;
}

int lcg_range(lcg_rng_t *rng, int min, int max) {
    uint32_t val = lcg_next(rng);
    int range = max - min + 1;
    return min + (int)(val % (uint32_t)range);
}

void pcg_seed(pcg_rng_t *rng, uint64_t seed, uint64_t seq) {
    rng->state = 0;
    rng->inc = (seq << 1) | 1;
    rng->state = rng->state + seed;
}

uint32_t pcg_next(pcg_rng_t *rng) {
    uint64_t old = rng->state;
    rng->state = old * 6364136223846793005ULL + rng->inc;
    uint32_t xorshifted = (uint32_t)(((old >> 18) ^ old) >> 27);
    uint32_t rot = (uint32_t)(old >> 59);
    return (xorshifted >> rot) | (xorshifted << (32 - rot));
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C674: Should produce output");
    assert!(
        rust_code.contains("fn lcg_seed"),
        "C674: Should contain lcg_seed"
    );
    assert!(
        rust_code.contains("fn lcg_next"),
        "C674: Should contain lcg_next"
    );
    assert!(
        rust_code.contains("fn pcg_next"),
        "C674: Should contain pcg_next"
    );
    Ok(())
}

/// C675: Command pattern (undo/redo)
#[test]
fn c675_command_pattern_undo_redo() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int type;
    int target_id;
    int old_value;
    int new_value;
} command_t;

typedef struct {
    command_t history[256];
    int count;
    int current;
} command_stack_t;

void cmd_init(command_stack_t *stack) {
    stack->count = 0;
    stack->current = -1;
}

void cmd_execute(command_stack_t *stack, int type, int target, int old_val, int new_val) {
    stack->current = stack->current + 1;
    stack->history[stack->current].type = type;
    stack->history[stack->current].target_id = target;
    stack->history[stack->current].old_value = old_val;
    stack->history[stack->current].new_value = new_val;
    stack->count = stack->current + 1;
}

int cmd_can_undo(command_stack_t *stack) {
    return (stack->current >= 0) ? 1 : 0;
}

int cmd_can_redo(command_stack_t *stack) {
    return (stack->current < stack->count - 1) ? 1 : 0;
}

int cmd_undo(command_stack_t *stack) {
    if (stack->current < 0) return -1;
    int val = stack->history[stack->current].old_value;
    stack->current = stack->current - 1;
    return val;
}

int cmd_redo(command_stack_t *stack) {
    if (stack->current >= stack->count - 1) return -1;
    stack->current = stack->current + 1;
    return stack->history[stack->current].new_value;
}

int cmd_get_undo_count(command_stack_t *stack) {
    return stack->current + 1;
}

int cmd_get_redo_count(command_stack_t *stack) {
    return stack->count - stack->current - 1;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C675: Should produce output");
    assert!(
        rust_code.contains("fn cmd_init"),
        "C675: Should contain cmd_init"
    );
    assert!(
        rust_code.contains("fn cmd_execute"),
        "C675: Should contain cmd_execute"
    );
    assert!(
        rust_code.contains("fn cmd_undo"),
        "C675: Should contain cmd_undo"
    );
    assert!(
        rust_code.contains("fn cmd_redo"),
        "C675: Should contain cmd_redo"
    );
    Ok(())
}
