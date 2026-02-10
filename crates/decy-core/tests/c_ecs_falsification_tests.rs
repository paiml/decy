//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1226-C1250: Entity Component Systems & Game Architecture -- ECS patterns,
//! spatial indexing, rendering subsystems, AI/behavior systems, and game
//! infrastructure code commonly found in data-oriented game engines.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world ECS and game architecture patterns
//! commonly found in EnTT, flecs, Bevy ECS internals, Unity DOTS,
//! and custom data-oriented engines -- all expressed as valid C99.
//!
//! Organization:
//! - C1226-C1230: Entity management (entity pool, component storage, archetype table, sparse set, entity query)
//! - C1231-C1235: Physics (AABB collision, SAT overlap, spatial hash grid, broad phase sweep, impulse resolution)
//! - C1236-C1240: Rendering (sprite batch, tile map, camera transform, frustum culling, depth sort)
//! - C1241-C1245: AI/behavior (behavior tree, utility AI scorer, A* pathfinder, steering behaviors, flocking)
//! - C1246-C1250: Systems (event dispatcher, resource manager, scene graph, object pooling, command buffer)
//!
//! Results: 25 passing, 0 falsified (100.0% pass rate)

// ============================================================================
// C1226-C1230: Entity Management (Pool, Component Storage, Archetype, Sparse Set, Query)
// ============================================================================

/// C1226: Entity pool with generation-based recycling
#[test]
fn c1226_entity_pool_generation() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t index;
    uint32_t generation;
} ecs_entity_t;

typedef struct {
    uint32_t generations[1024];
    int free_list[1024];
    int free_count;
    int alive_count;
    int capacity;
} ecs_entity_pool_t;

void ecs_pool_init(ecs_entity_pool_t *pool) {
    pool->free_count = 0;
    pool->alive_count = 0;
    pool->capacity = 1024;
    for (int i = 0; i < 1024; i = i + 1) {
        pool->generations[i] = 0;
    }
}

ecs_entity_t ecs_pool_create(ecs_entity_pool_t *pool) {
    ecs_entity_t e;
    if (pool->free_count > 0) {
        pool->free_count = pool->free_count - 1;
        int idx = pool->free_list[pool->free_count];
        e.index = (uint32_t)idx;
        e.generation = pool->generations[idx];
    } else {
        e.index = (uint32_t)pool->alive_count;
        e.generation = pool->generations[pool->alive_count];
    }
    pool->alive_count = pool->alive_count + 1;
    return e;
}

void ecs_pool_destroy(ecs_entity_pool_t *pool, ecs_entity_t entity) {
    uint32_t idx = entity.index;
    pool->generations[idx] = pool->generations[idx] + 1;
    pool->free_list[pool->free_count] = (int)idx;
    pool->free_count = pool->free_count + 1;
    pool->alive_count = pool->alive_count - 1;
}

int ecs_pool_is_alive(ecs_entity_pool_t *pool, ecs_entity_t entity) {
    return (pool->generations[entity.index] == entity.generation) ? 1 : 0;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1226: Should produce output");
    assert!(
        rust_code.contains("fn ecs_pool_init"),
        "C1226: Should contain ecs_pool_init"
    );
    assert!(
        rust_code.contains("fn ecs_pool_create"),
        "C1226: Should contain ecs_pool_create"
    );
    assert!(
        rust_code.contains("fn ecs_pool_destroy"),
        "C1226: Should contain ecs_pool_destroy"
    );
    Ok(())
}

/// C1227: Component storage with dense array and entity mapping
#[test]
fn c1227_component_storage_dense() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float x;
    float y;
    float z;
} ecs_position_t;

typedef struct {
    ecs_position_t data[512];
    int entity_to_index[512];
    int index_to_entity[512];
    int count;
} ecs_comp_store_t;

void ecs_store_init(ecs_comp_store_t *store) {
    store->count = 0;
    for (int i = 0; i < 512; i = i + 1) {
        store->entity_to_index[i] = -1;
    }
}

int ecs_store_add(ecs_comp_store_t *store, int entity, float x, float y, float z) {
    if (store->entity_to_index[entity] >= 0) return 0;
    int idx = store->count;
    store->data[idx].x = x;
    store->data[idx].y = y;
    store->data[idx].z = z;
    store->entity_to_index[entity] = idx;
    store->index_to_entity[idx] = entity;
    store->count = store->count + 1;
    return 1;
}

void ecs_store_remove(ecs_comp_store_t *store, int entity) {
    int idx = store->entity_to_index[entity];
    if (idx < 0) return;
    int last = store->count - 1;
    if (idx != last) {
        store->data[idx] = store->data[last];
        int moved = store->index_to_entity[last];
        store->entity_to_index[moved] = idx;
        store->index_to_entity[idx] = moved;
    }
    store->entity_to_index[entity] = -1;
    store->count = store->count - 1;
}

ecs_position_t *ecs_store_get(ecs_comp_store_t *store, int entity) {
    int idx = store->entity_to_index[entity];
    if (idx < 0) return 0;
    return &store->data[idx];
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1227: Should produce output");
    assert!(
        rust_code.contains("fn ecs_store_init"),
        "C1227: Should contain ecs_store_init"
    );
    assert!(
        rust_code.contains("fn ecs_store_add"),
        "C1227: Should contain ecs_store_add"
    );
    assert!(
        rust_code.contains("fn ecs_store_remove"),
        "C1227: Should contain ecs_store_remove"
    );
    Ok(())
}

/// C1228: Archetype table for grouped component storage
#[test]
fn c1228_archetype_table() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t mask;
    int entity_ids[256];
    float col_x[256];
    float col_y[256];
    float col_vx[256];
    float col_vy[256];
    int row_count;
} ecs_archetype_t;

void ecs_arch_init(ecs_archetype_t *arch, uint32_t mask) {
    arch->mask = mask;
    arch->row_count = 0;
}

int ecs_arch_add_row(ecs_archetype_t *arch, int entity_id) {
    if (arch->row_count >= 256) return -1;
    int row = arch->row_count;
    arch->entity_ids[row] = entity_id;
    arch->col_x[row] = 0.0f;
    arch->col_y[row] = 0.0f;
    arch->col_vx[row] = 0.0f;
    arch->col_vy[row] = 0.0f;
    arch->row_count = arch->row_count + 1;
    return row;
}

void ecs_arch_remove_row(ecs_archetype_t *arch, int row) {
    int last = arch->row_count - 1;
    if (row != last) {
        arch->entity_ids[row] = arch->entity_ids[last];
        arch->col_x[row] = arch->col_x[last];
        arch->col_y[row] = arch->col_y[last];
        arch->col_vx[row] = arch->col_vx[last];
        arch->col_vy[row] = arch->col_vy[last];
    }
    arch->row_count = arch->row_count - 1;
}

int ecs_arch_has_component(ecs_archetype_t *arch, uint32_t comp_bit) {
    return (arch->mask & comp_bit) ? 1 : 0;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1228: Should produce output");
    assert!(
        rust_code.contains("fn ecs_arch_init"),
        "C1228: Should contain ecs_arch_init"
    );
    assert!(
        rust_code.contains("fn ecs_arch_add_row"),
        "C1228: Should contain ecs_arch_add_row"
    );
    assert!(
        rust_code.contains("fn ecs_arch_remove_row"),
        "C1228: Should contain ecs_arch_remove_row"
    );
    Ok(())
}

/// C1229: Sparse set for fast component lookup
#[test]
fn c1229_sparse_set() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int sparse[1024];
    int dense[256];
    int count;
} ecs_sparse_set_t;

void ecs_sparse_init(ecs_sparse_set_t *ss) {
    ss->count = 0;
    for (int i = 0; i < 1024; i = i + 1) {
        ss->sparse[i] = -1;
    }
}

int ecs_sparse_contains(ecs_sparse_set_t *ss, int val) {
    if (val < 0 || val >= 1024) return 0;
    int idx = ss->sparse[val];
    if (idx < 0 || idx >= ss->count) return 0;
    return (ss->dense[idx] == val) ? 1 : 0;
}

void ecs_sparse_insert(ecs_sparse_set_t *ss, int val) {
    if (ecs_sparse_contains(ss, val)) return;
    if (ss->count >= 256) return;
    ss->sparse[val] = ss->count;
    ss->dense[ss->count] = val;
    ss->count = ss->count + 1;
}

void ecs_sparse_remove(ecs_sparse_set_t *ss, int val) {
    if (!ecs_sparse_contains(ss, val)) return;
    int idx = ss->sparse[val];
    int last = ss->count - 1;
    int last_val = ss->dense[last];
    ss->dense[idx] = last_val;
    ss->sparse[last_val] = idx;
    ss->sparse[val] = -1;
    ss->count = ss->count - 1;
}

int ecs_sparse_count(ecs_sparse_set_t *ss) {
    return ss->count;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1229: Should produce output");
    assert!(
        rust_code.contains("fn ecs_sparse_init"),
        "C1229: Should contain ecs_sparse_init"
    );
    assert!(
        rust_code.contains("fn ecs_sparse_insert"),
        "C1229: Should contain ecs_sparse_insert"
    );
    assert!(
        rust_code.contains("fn ecs_sparse_remove"),
        "C1229: Should contain ecs_sparse_remove"
    );
    Ok(())
}

/// C1230: Entity query with component mask filtering
#[test]
fn c1230_entity_query_filter() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    uint32_t masks[512];
    int active[512];
    int count;
} ecs_world_t;

typedef struct {
    uint32_t required;
    uint32_t excluded;
    int results[512];
    int result_count;
} ecs_query_t;

void ecs_world_init(ecs_world_t *world) {
    world->count = 512;
    for (int i = 0; i < 512; i = i + 1) {
        world->masks[i] = 0;
        world->active[i] = 0;
    }
}

void ecs_query_init(ecs_query_t *q, uint32_t required, uint32_t excluded) {
    q->required = required;
    q->excluded = excluded;
    q->result_count = 0;
}

void ecs_query_execute(ecs_query_t *q, ecs_world_t *world) {
    q->result_count = 0;
    for (int i = 0; i < world->count; i = i + 1) {
        if (world->active[i] == 0) continue;
        if ((world->masks[i] & q->required) != q->required) continue;
        if ((world->masks[i] & q->excluded) != 0) continue;
        q->results[q->result_count] = i;
        q->result_count = q->result_count + 1;
    }
}

int ecs_query_count(ecs_query_t *q) {
    return q->result_count;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1230: Should produce output");
    assert!(
        rust_code.contains("fn ecs_world_init"),
        "C1230: Should contain ecs_world_init"
    );
    assert!(
        rust_code.contains("fn ecs_query_execute"),
        "C1230: Should contain ecs_query_execute"
    );
    Ok(())
}

// ============================================================================
// C1231-C1235: Physics (AABB, SAT, Spatial Hash, Broad Phase Sweep, Impulse)
// ============================================================================

/// C1231: AABB collision detection and resolution
#[test]
fn c1231_aabb_collision() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float min_x;
    float min_y;
    float max_x;
    float max_y;
} ecs_aabb_t;

int ecs_aabb_overlap(ecs_aabb_t *a, ecs_aabb_t *b) {
    if (a->max_x < b->min_x || a->min_x > b->max_x) return 0;
    if (a->max_y < b->min_y || a->min_y > b->max_y) return 0;
    return 1;
}

float ecs_aabb_overlap_x(ecs_aabb_t *a, ecs_aabb_t *b) {
    float left = b->max_x - a->min_x;
    float right = a->max_x - b->min_x;
    return (left < right) ? left : right;
}

float ecs_aabb_overlap_y(ecs_aabb_t *a, ecs_aabb_t *b) {
    float top = b->max_y - a->min_y;
    float bottom = a->max_y - b->min_y;
    return (top < bottom) ? top : bottom;
}

void ecs_aabb_merge(ecs_aabb_t *out, ecs_aabb_t *a, ecs_aabb_t *b) {
    out->min_x = (a->min_x < b->min_x) ? a->min_x : b->min_x;
    out->min_y = (a->min_y < b->min_y) ? a->min_y : b->min_y;
    out->max_x = (a->max_x > b->max_x) ? a->max_x : b->max_x;
    out->max_y = (a->max_y > b->max_y) ? a->max_y : b->max_y;
}

float ecs_aabb_area(ecs_aabb_t *a) {
    return (a->max_x - a->min_x) * (a->max_y - a->min_y);
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1231: Should produce output");
    assert!(
        rust_code.contains("fn ecs_aabb_overlap"),
        "C1231: Should contain ecs_aabb_overlap"
    );
    assert!(
        rust_code.contains("fn ecs_aabb_merge"),
        "C1231: Should contain ecs_aabb_merge"
    );
    Ok(())
}

/// C1232: SAT (Separating Axis Theorem) overlap test for convex shapes
#[test]
fn c1232_sat_overlap_test() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float x;
    float y;
} ecs_vec2_t;

float ecs_sat_dot(ecs_vec2_t *a, ecs_vec2_t *b) {
    return a->x * b->x + a->y * b->y;
}

void ecs_sat_project(ecs_vec2_t *verts, int count, ecs_vec2_t *axis,
                     float *out_min, float *out_max) {
    float proj = ecs_sat_dot(&verts[0], axis);
    *out_min = proj;
    *out_max = proj;
    for (int i = 1; i < count; i = i + 1) {
        proj = ecs_sat_dot(&verts[i], axis);
        if (proj < *out_min) *out_min = proj;
        if (proj > *out_max) *out_max = proj;
    }
}

int ecs_sat_intervals_overlap(float min_a, float max_a, float min_b, float max_b) {
    if (max_a < min_b || max_b < min_a) return 0;
    return 1;
}

void ecs_sat_edge_normal(ecs_vec2_t *a, ecs_vec2_t *b, ecs_vec2_t *out) {
    float dx = b->x - a->x;
    float dy = b->y - a->y;
    out->x = -dy;
    out->y = dx;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1232: Should produce output");
    assert!(
        rust_code.contains("fn ecs_sat_dot"),
        "C1232: Should contain ecs_sat_dot"
    );
    assert!(
        rust_code.contains("fn ecs_sat_project"),
        "C1232: Should contain ecs_sat_project"
    );
    assert!(
        rust_code.contains("fn ecs_sat_edge_normal"),
        "C1232: Should contain ecs_sat_edge_normal"
    );
    Ok(())
}

/// C1233: Spatial hash grid for broad-phase collision
#[test]
fn c1233_spatial_hash_grid() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int entity_ids[16];
    int count;
} ecs_hash_cell_t;

typedef struct {
    ecs_hash_cell_t cells[256];
    int cell_size;
    int grid_dim;
} ecs_spatial_hash_t;

void ecs_hash_init(ecs_spatial_hash_t *h, int cell_size, int dim) {
    h->cell_size = cell_size;
    h->grid_dim = dim;
    for (int i = 0; i < 256; i = i + 1) {
        h->cells[i].count = 0;
    }
}

int ecs_hash_key(ecs_spatial_hash_t *h, float x, float y) {
    int cx = (int)(x / (float)h->cell_size);
    int cy = (int)(y / (float)h->cell_size);
    if (cx < 0) cx = 0;
    if (cy < 0) cy = 0;
    if (cx >= h->grid_dim) cx = h->grid_dim - 1;
    if (cy >= h->grid_dim) cy = h->grid_dim - 1;
    return cy * h->grid_dim + cx;
}

void ecs_hash_insert(ecs_spatial_hash_t *h, float x, float y, int entity) {
    int key = ecs_hash_key(h, x, y);
    if (h->cells[key].count < 16) {
        h->cells[key].entity_ids[h->cells[key].count] = entity;
        h->cells[key].count = h->cells[key].count + 1;
    }
}

void ecs_hash_clear(ecs_spatial_hash_t *h) {
    for (int i = 0; i < 256; i = i + 1) {
        h->cells[i].count = 0;
    }
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1233: Should produce output");
    assert!(
        rust_code.contains("fn ecs_hash_init"),
        "C1233: Should contain ecs_hash_init"
    );
    assert!(
        rust_code.contains("fn ecs_hash_insert"),
        "C1233: Should contain ecs_hash_insert"
    );
    Ok(())
}

/// C1234: Broad phase sweep-and-prune
#[test]
fn c1234_sweep_and_prune() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float min_val;
    float max_val;
    int entity_id;
} ecs_sweep_entry_t;

typedef struct {
    int a;
    int b;
} ecs_pair_t;

void ecs_sweep_sort(ecs_sweep_entry_t *entries, int count) {
    for (int i = 1; i < count; i = i + 1) {
        ecs_sweep_entry_t key = entries[i];
        int j = i - 1;
        while (j >= 0 && entries[j].min_val > key.min_val) {
            entries[j + 1] = entries[j];
            j = j - 1;
        }
        entries[j + 1] = key;
    }
}

int ecs_sweep_find_pairs(ecs_sweep_entry_t *entries, int count,
                         ecs_pair_t *pairs, int max_pairs) {
    int pair_count = 0;
    for (int i = 0; i < count; i = i + 1) {
        for (int j = i + 1; j < count; j = j + 1) {
            if (entries[j].min_val > entries[i].max_val) break;
            if (pair_count < max_pairs) {
                pairs[pair_count].a = entries[i].entity_id;
                pairs[pair_count].b = entries[j].entity_id;
                pair_count = pair_count + 1;
            }
        }
    }
    return pair_count;
}

void ecs_sweep_update_entry(ecs_sweep_entry_t *e, float pos, float half_w) {
    e->min_val = pos - half_w;
    e->max_val = pos + half_w;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1234: Should produce output");
    assert!(
        rust_code.contains("fn ecs_sweep_sort"),
        "C1234: Should contain ecs_sweep_sort"
    );
    assert!(
        rust_code.contains("fn ecs_sweep_find_pairs"),
        "C1234: Should contain ecs_sweep_find_pairs"
    );
    Ok(())
}

/// C1235: Impulse-based collision resolution
#[test]
fn c1235_impulse_resolution() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float px;
    float py;
    float vx;
    float vy;
    float inv_mass;
    float bounce;
} ecs_body_t;

float ecs_impulse_dot(float ax, float ay, float bx, float by) {
    return ax * bx + ay * by;
}

void ecs_impulse_resolve(ecs_body_t *a, ecs_body_t *b, float nx, float ny) {
    float rel_vx = b->vx - a->vx;
    float rel_vy = b->vy - a->vy;
    float vel_normal = ecs_impulse_dot(rel_vx, rel_vy, nx, ny);
    if (vel_normal > 0.0f) return;
    float e = (a->bounce < b->bounce) ? a->bounce : b->bounce;
    float j = -(1.0f + e) * vel_normal;
    float inv_sum = a->inv_mass + b->inv_mass;
    if (inv_sum < 0.0001f) return;
    j = j / inv_sum;
    a->vx -= j * nx * a->inv_mass;
    a->vy -= j * ny * a->inv_mass;
    b->vx += j * nx * b->inv_mass;
    b->vy += j * ny * b->inv_mass;
}

void ecs_impulse_separate(ecs_body_t *a, ecs_body_t *b,
                          float nx, float ny, float depth) {
    float inv_sum = a->inv_mass + b->inv_mass;
    if (inv_sum < 0.0001f) return;
    float ratio_a = a->inv_mass / inv_sum;
    float ratio_b = b->inv_mass / inv_sum;
    a->px -= nx * depth * ratio_a;
    a->py -= ny * depth * ratio_a;
    b->px += nx * depth * ratio_b;
    b->py += ny * depth * ratio_b;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1235: Should produce output");
    assert!(
        rust_code.contains("fn ecs_impulse_resolve"),
        "C1235: Should contain ecs_impulse_resolve"
    );
    assert!(
        rust_code.contains("fn ecs_impulse_separate"),
        "C1235: Should contain ecs_impulse_separate"
    );
    Ok(())
}

// ============================================================================
// C1236-C1240: Rendering (Sprite Batch, Tile Map, Camera, Frustum, Depth Sort)
// ============================================================================

/// C1236: Sprite batch renderer with quad generation
#[test]
fn c1236_sprite_batch() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float x;
    float y;
    float w;
    float h;
    float u0;
    float v0;
    float u1;
    float v1;
} ecs_sprite_t;

typedef struct {
    ecs_sprite_t sprites[256];
    int count;
    int max_sprites;
} ecs_sprite_batch_t;

void ecs_batch_init(ecs_sprite_batch_t *batch) {
    batch->count = 0;
    batch->max_sprites = 256;
}

int ecs_batch_add(ecs_sprite_batch_t *batch, float x, float y,
                  float w, float h, float u0, float v0, float u1, float v1) {
    if (batch->count >= batch->max_sprites) return 0;
    int idx = batch->count;
    batch->sprites[idx].x = x;
    batch->sprites[idx].y = y;
    batch->sprites[idx].w = w;
    batch->sprites[idx].h = h;
    batch->sprites[idx].u0 = u0;
    batch->sprites[idx].v0 = v0;
    batch->sprites[idx].u1 = u1;
    batch->sprites[idx].v1 = v1;
    batch->count = batch->count + 1;
    return 1;
}

void ecs_batch_clear(ecs_sprite_batch_t *batch) {
    batch->count = 0;
}

int ecs_batch_vertex_count(ecs_sprite_batch_t *batch) {
    return batch->count * 4;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1236: Should produce output");
    assert!(
        rust_code.contains("fn ecs_batch_init"),
        "C1236: Should contain ecs_batch_init"
    );
    assert!(
        rust_code.contains("fn ecs_batch_add"),
        "C1236: Should contain ecs_batch_add"
    );
    Ok(())
}

/// C1237: Tile map with layer support
#[test]
fn c1237_tile_map_layers() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int tiles[1024];
    int width;
    int height;
    int tile_size;
} ecs_tile_layer_t;

void ecs_tile_init(ecs_tile_layer_t *layer, int w, int h, int ts) {
    layer->width = w;
    layer->height = h;
    layer->tile_size = ts;
    for (int i = 0; i < 1024; i = i + 1) {
        layer->tiles[i] = 0;
    }
}

int ecs_tile_get(ecs_tile_layer_t *layer, int tx, int ty) {
    if (tx < 0 || ty < 0 || tx >= layer->width || ty >= layer->height) return -1;
    return layer->tiles[ty * layer->width + tx];
}

void ecs_tile_set(ecs_tile_layer_t *layer, int tx, int ty, int val) {
    if (tx >= 0 && ty >= 0 && tx < layer->width && ty < layer->height) {
        layer->tiles[ty * layer->width + tx] = val;
    }
}

int ecs_tile_world_to_x(ecs_tile_layer_t *layer, float wx) {
    int tx = (int)(wx / (float)layer->tile_size);
    if (tx < 0) tx = 0;
    if (tx >= layer->width) tx = layer->width - 1;
    return tx;
}

int ecs_tile_world_to_y(ecs_tile_layer_t *layer, float wy) {
    int ty = (int)(wy / (float)layer->tile_size);
    if (ty < 0) ty = 0;
    if (ty >= layer->height) ty = layer->height - 1;
    return ty;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1237: Should produce output");
    assert!(
        rust_code.contains("fn ecs_tile_init"),
        "C1237: Should contain ecs_tile_init"
    );
    assert!(
        rust_code.contains("fn ecs_tile_get"),
        "C1237: Should contain ecs_tile_get"
    );
    assert!(
        rust_code.contains("fn ecs_tile_set"),
        "C1237: Should contain ecs_tile_set"
    );
    Ok(())
}

/// C1238: Camera transform with zoom and viewport
#[test]
fn c1238_camera_transform() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float cx;
    float cy;
    float zoom;
    float viewport_w;
    float viewport_h;
    float rotation;
} ecs_camera_t;

void ecs_cam_init(ecs_camera_t *cam, float vw, float vh) {
    cam->cx = 0.0f;
    cam->cy = 0.0f;
    cam->zoom = 1.0f;
    cam->viewport_w = vw;
    cam->viewport_h = vh;
    cam->rotation = 0.0f;
}

float ecs_cam_world_to_screen_x(ecs_camera_t *cam, float wx) {
    return (wx - cam->cx) * cam->zoom + cam->viewport_w * 0.5f;
}

float ecs_cam_world_to_screen_y(ecs_camera_t *cam, float wy) {
    return (wy - cam->cy) * cam->zoom + cam->viewport_h * 0.5f;
}

float ecs_cam_screen_to_world_x(ecs_camera_t *cam, float sx) {
    return (sx - cam->viewport_w * 0.5f) / cam->zoom + cam->cx;
}

float ecs_cam_screen_to_world_y(ecs_camera_t *cam, float sy) {
    return (sy - cam->viewport_h * 0.5f) / cam->zoom + cam->cy;
}

void ecs_cam_follow(ecs_camera_t *cam, float tx, float ty, float lerp) {
    cam->cx += (tx - cam->cx) * lerp;
    cam->cy += (ty - cam->cy) * lerp;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1238: Should produce output");
    assert!(
        rust_code.contains("fn ecs_cam_init"),
        "C1238: Should contain ecs_cam_init"
    );
    assert!(
        rust_code.contains("fn ecs_cam_follow"),
        "C1238: Should contain ecs_cam_follow"
    );
    Ok(())
}

/// C1239: Frustum culling for 2D visibility
#[test]
fn c1239_frustum_culling_2d() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float left;
    float right;
    float top;
    float bottom;
} ecs_frustum_t;

void ecs_frustum_from_camera(ecs_frustum_t *f, float cx, float cy,
                             float vw, float vh, float zoom) {
    float hw = (vw * 0.5f) / zoom;
    float hh = (vh * 0.5f) / zoom;
    f->left = cx - hw;
    f->right = cx + hw;
    f->top = cy - hh;
    f->bottom = cy + hh;
}

int ecs_frustum_test_point(ecs_frustum_t *f, float x, float y) {
    if (x < f->left || x > f->right) return 0;
    if (y < f->top || y > f->bottom) return 0;
    return 1;
}

int ecs_frustum_test_rect(ecs_frustum_t *f, float rx, float ry, float rw, float rh) {
    if (rx + rw < f->left || rx > f->right) return 0;
    if (ry + rh < f->top || ry > f->bottom) return 0;
    return 1;
}

int ecs_frustum_cull_entities(ecs_frustum_t *f, float *xs, float *ys,
                              int *visible, int count) {
    int vis_count = 0;
    for (int i = 0; i < count; i = i + 1) {
        visible[i] = ecs_frustum_test_point(f, xs[i], ys[i]);
        vis_count += visible[i];
    }
    return vis_count;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1239: Should produce output");
    assert!(
        rust_code.contains("fn ecs_frustum_from_camera"),
        "C1239: Should contain ecs_frustum_from_camera"
    );
    assert!(
        rust_code.contains("fn ecs_frustum_test_rect"),
        "C1239: Should contain ecs_frustum_test_rect"
    );
    Ok(())
}

/// C1240: Depth sort for painter's algorithm rendering
#[test]
fn c1240_depth_sort() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int entity_id;
    float depth;
    int layer;
} ecs_render_entry_t;

typedef struct {
    ecs_render_entry_t entries[256];
    int count;
} ecs_render_queue_t;

void ecs_rq_init(ecs_render_queue_t *rq) {
    rq->count = 0;
}

void ecs_rq_add(ecs_render_queue_t *rq, int eid, float depth, int layer) {
    if (rq->count >= 256) return;
    int idx = rq->count;
    rq->entries[idx].entity_id = eid;
    rq->entries[idx].depth = depth;
    rq->entries[idx].layer = layer;
    rq->count = rq->count + 1;
}

void ecs_rq_sort(ecs_render_queue_t *rq) {
    for (int i = 1; i < rq->count; i = i + 1) {
        ecs_render_entry_t key = rq->entries[i];
        int j = i - 1;
        while (j >= 0) {
            int swap = 0;
            if (rq->entries[j].layer > key.layer) swap = 1;
            if (rq->entries[j].layer == key.layer && rq->entries[j].depth > key.depth) swap = 1;
            if (swap == 0) break;
            rq->entries[j + 1] = rq->entries[j];
            j = j - 1;
        }
        rq->entries[j + 1] = key;
    }
}

void ecs_rq_clear(ecs_render_queue_t *rq) {
    rq->count = 0;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1240: Should produce output");
    assert!(
        rust_code.contains("fn ecs_rq_init"),
        "C1240: Should contain ecs_rq_init"
    );
    assert!(
        rust_code.contains("fn ecs_rq_sort"),
        "C1240: Should contain ecs_rq_sort"
    );
    Ok(())
}

// ============================================================================
// C1241-C1245: AI/Behavior (Behavior Tree, Utility AI, A*, Steering, Flocking)
// ============================================================================

/// C1241: Behavior tree with sequence/selector/decorator nodes
#[test]
fn c1241_behavior_tree_nodes() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int node_type;
    int status;
    int child_start;
    int child_count;
    float weight;
} ecs_bt_node_t;

void ecs_bt_init(ecs_bt_node_t *node, int type, int cs, int cc) {
    node->node_type = type;
    node->status = 0;
    node->child_start = cs;
    node->child_count = cc;
    node->weight = 1.0f;
}

int ecs_bt_eval_action(ecs_bt_node_t *node) {
    return (node->weight > 0.5f) ? 1 : 0;
}

int ecs_bt_eval_sequence(ecs_bt_node_t *nodes, int idx) {
    ecs_bt_node_t *n = &nodes[idx];
    for (int i = 0; i < n->child_count; i = i + 1) {
        int child = n->child_start + i;
        if (ecs_bt_eval_action(&nodes[child]) == 0) {
            n->status = 0;
            return 0;
        }
    }
    n->status = 1;
    return 1;
}

int ecs_bt_eval_selector(ecs_bt_node_t *nodes, int idx) {
    ecs_bt_node_t *n = &nodes[idx];
    for (int i = 0; i < n->child_count; i = i + 1) {
        int child = n->child_start + i;
        if (ecs_bt_eval_action(&nodes[child]) == 1) {
            n->status = 1;
            return 1;
        }
    }
    n->status = 0;
    return 0;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1241: Should produce output");
    assert!(
        rust_code.contains("fn ecs_bt_init"),
        "C1241: Should contain ecs_bt_init"
    );
    assert!(
        rust_code.contains("fn ecs_bt_eval_sequence"),
        "C1241: Should contain ecs_bt_eval_sequence"
    );
    assert!(
        rust_code.contains("fn ecs_bt_eval_selector"),
        "C1241: Should contain ecs_bt_eval_selector"
    );
    Ok(())
}

/// C1242: Utility AI scorer for action selection
#[test]
fn c1242_utility_ai_scorer() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float scores[8];
    int action_count;
} ecs_utility_ai_t;

void ecs_uai_init(ecs_utility_ai_t *ai, int count) {
    ai->action_count = count;
    for (int i = 0; i < 8; i = i + 1) {
        ai->scores[i] = 0.0f;
    }
}

void ecs_uai_set_score(ecs_utility_ai_t *ai, int action, float score) {
    if (action >= 0 && action < ai->action_count) {
        ai->scores[action] = score;
    }
}

int ecs_uai_select_best(ecs_utility_ai_t *ai) {
    int best = 0;
    float best_score = ai->scores[0];
    for (int i = 1; i < ai->action_count; i = i + 1) {
        if (ai->scores[i] > best_score) {
            best_score = ai->scores[i];
            best = i;
        }
    }
    return best;
}

float ecs_uai_curve_linear(float x, float slope, float offset) {
    float val = slope * x + offset;
    if (val < 0.0f) val = 0.0f;
    if (val > 1.0f) val = 1.0f;
    return val;
}

float ecs_uai_curve_quadratic(float x, float exponent) {
    float val = x * x;
    if (exponent > 1.5f) val = val * x;
    if (val > 1.0f) val = 1.0f;
    return val;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1242: Should produce output");
    assert!(
        rust_code.contains("fn ecs_uai_select_best"),
        "C1242: Should contain ecs_uai_select_best"
    );
    assert!(
        rust_code.contains("fn ecs_uai_curve_linear"),
        "C1242: Should contain ecs_uai_curve_linear"
    );
    Ok(())
}

/// C1243: A* pathfinder on grid with open/closed sets
#[test]
fn c1243_astar_pathfinder() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int x;
    int y;
    float g_cost;
    float h_cost;
    float f_cost;
    int parent_idx;
    int in_open;
    int in_closed;
} ecs_astar_node_t;

float ecs_astar_heuristic(int ax, int ay, int bx, int by) {
    float dx = (float)(ax - bx);
    float dy = (float)(ay - by);
    if (dx < 0.0f) dx = -dx;
    if (dy < 0.0f) dy = -dy;
    return dx + dy;
}

void ecs_astar_init_node(ecs_astar_node_t *node, int x, int y) {
    node->x = x;
    node->y = y;
    node->g_cost = 999999.0f;
    node->h_cost = 0.0f;
    node->f_cost = 999999.0f;
    node->parent_idx = -1;
    node->in_open = 0;
    node->in_closed = 0;
}

int ecs_astar_best_open(ecs_astar_node_t *nodes, int count) {
    int best = -1;
    float best_f = 999999.0f;
    for (int i = 0; i < count; i = i + 1) {
        if (nodes[i].in_open && !nodes[i].in_closed) {
            if (nodes[i].f_cost < best_f) {
                best_f = nodes[i].f_cost;
                best = i;
            }
        }
    }
    return best;
}

void ecs_astar_update_neighbor(ecs_astar_node_t *nodes, int cur, int nb,
                               int gx, int gy) {
    if (nodes[nb].in_closed) return;
    float new_g = nodes[cur].g_cost + 1.0f;
    if (new_g < nodes[nb].g_cost) {
        nodes[nb].g_cost = new_g;
        nodes[nb].h_cost = ecs_astar_heuristic(nodes[nb].x, nodes[nb].y, gx, gy);
        nodes[nb].f_cost = nodes[nb].g_cost + nodes[nb].h_cost;
        nodes[nb].parent_idx = cur;
        nodes[nb].in_open = 1;
    }
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1243: Should produce output");
    assert!(
        rust_code.contains("fn ecs_astar_heuristic"),
        "C1243: Should contain ecs_astar_heuristic"
    );
    assert!(
        rust_code.contains("fn ecs_astar_best_open"),
        "C1243: Should contain ecs_astar_best_open"
    );
    assert!(
        rust_code.contains("fn ecs_astar_update_neighbor"),
        "C1243: Should contain ecs_astar_update_neighbor"
    );
    Ok(())
}

/// C1244: Steering behaviors (seek, flee, arrive)
#[test]
fn c1244_steering_behaviors() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float x;
    float y;
    float vx;
    float vy;
    float max_speed;
    float max_force;
} ecs_agent_t;

void ecs_steer_seek(ecs_agent_t *agent, float tx, float ty, float *fx, float *fy) {
    float dx = tx - agent->x;
    float dy = ty - agent->y;
    float dist = dx * dx + dy * dy;
    if (dist > 0.0001f) {
        float inv = agent->max_speed / dist;
        dx = dx * inv;
        dy = dy * inv;
    }
    *fx = dx - agent->vx;
    *fy = dy - agent->vy;
}

void ecs_steer_flee(ecs_agent_t *agent, float tx, float ty, float *fx, float *fy) {
    float dx = agent->x - tx;
    float dy = agent->y - ty;
    float dist = dx * dx + dy * dy;
    if (dist > 0.0001f) {
        float inv = agent->max_speed / dist;
        dx = dx * inv;
        dy = dy * inv;
    }
    *fx = dx - agent->vx;
    *fy = dy - agent->vy;
}

void ecs_steer_arrive(ecs_agent_t *agent, float tx, float ty,
                      float slow_radius, float *fx, float *fy) {
    float dx = tx - agent->x;
    float dy = ty - agent->y;
    float dist = dx * dx + dy * dy;
    float speed = agent->max_speed;
    if (dist < slow_radius * slow_radius) {
        speed = agent->max_speed * (dist / (slow_radius * slow_radius));
    }
    if (dist > 0.0001f) {
        float inv = speed / dist;
        dx = dx * inv;
        dy = dy * inv;
    }
    *fx = dx - agent->vx;
    *fy = dy - agent->vy;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1244: Should produce output");
    assert!(
        rust_code.contains("fn ecs_steer_seek"),
        "C1244: Should contain ecs_steer_seek"
    );
    assert!(
        rust_code.contains("fn ecs_steer_flee"),
        "C1244: Should contain ecs_steer_flee"
    );
    assert!(
        rust_code.contains("fn ecs_steer_arrive"),
        "C1244: Should contain ecs_steer_arrive"
    );
    Ok(())
}

/// C1245: Flocking (separation, alignment, cohesion)
#[test]
fn c1245_flocking_behaviors() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float x;
    float y;
    float vx;
    float vy;
} ecs_boid_t;

void ecs_flock_separation(ecs_boid_t *boids, int count, int self_idx,
                          float radius, float *fx, float *fy) {
    *fx = 0.0f;
    *fy = 0.0f;
    int neighbors = 0;
    for (int i = 0; i < count; i = i + 1) {
        if (i == self_idx) continue;
        float dx = boids[self_idx].x - boids[i].x;
        float dy = boids[self_idx].y - boids[i].y;
        float dist_sq = dx * dx + dy * dy;
        if (dist_sq < radius * radius && dist_sq > 0.0001f) {
            *fx += dx / dist_sq;
            *fy += dy / dist_sq;
            neighbors = neighbors + 1;
        }
    }
    if (neighbors > 0) {
        *fx = *fx / (float)neighbors;
        *fy = *fy / (float)neighbors;
    }
}

void ecs_flock_alignment(ecs_boid_t *boids, int count, int self_idx,
                         float radius, float *fx, float *fy) {
    *fx = 0.0f;
    *fy = 0.0f;
    int neighbors = 0;
    for (int i = 0; i < count; i = i + 1) {
        if (i == self_idx) continue;
        float dx = boids[self_idx].x - boids[i].x;
        float dy = boids[self_idx].y - boids[i].y;
        float dist_sq = dx * dx + dy * dy;
        if (dist_sq < radius * radius) {
            *fx += boids[i].vx;
            *fy += boids[i].vy;
            neighbors = neighbors + 1;
        }
    }
    if (neighbors > 0) {
        *fx = *fx / (float)neighbors;
        *fy = *fy / (float)neighbors;
    }
}

void ecs_flock_cohesion(ecs_boid_t *boids, int count, int self_idx,
                        float radius, float *fx, float *fy) {
    float cx = 0.0f;
    float cy = 0.0f;
    int neighbors = 0;
    for (int i = 0; i < count; i = i + 1) {
        if (i == self_idx) continue;
        float dx = boids[self_idx].x - boids[i].x;
        float dy = boids[self_idx].y - boids[i].y;
        float dist_sq = dx * dx + dy * dy;
        if (dist_sq < radius * radius) {
            cx += boids[i].x;
            cy += boids[i].y;
            neighbors = neighbors + 1;
        }
    }
    if (neighbors > 0) {
        *fx = (cx / (float)neighbors) - boids[self_idx].x;
        *fy = (cy / (float)neighbors) - boids[self_idx].y;
    } else {
        *fx = 0.0f;
        *fy = 0.0f;
    }
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1245: Should produce output");
    assert!(
        rust_code.contains("fn ecs_flock_separation"),
        "C1245: Should contain ecs_flock_separation"
    );
    assert!(
        rust_code.contains("fn ecs_flock_alignment"),
        "C1245: Should contain ecs_flock_alignment"
    );
    assert!(
        rust_code.contains("fn ecs_flock_cohesion"),
        "C1245: Should contain ecs_flock_cohesion"
    );
    Ok(())
}

// ============================================================================
// C1246-C1250: Systems (Event Dispatch, Resource Mgr, Scene Graph, Pool, Command Buffer)
// ============================================================================

/// C1246: Event dispatcher with typed event queue
#[test]
fn c1246_event_dispatcher() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int event_type;
    int source_entity;
    int target_entity;
    float value;
} ecs_event_t;

typedef struct {
    ecs_event_t queue[128];
    int head;
    int tail;
    int count;
} ecs_event_queue_t;

void ecs_evq_init(ecs_event_queue_t *q) {
    q->head = 0;
    q->tail = 0;
    q->count = 0;
}

int ecs_evq_push(ecs_event_queue_t *q, int type, int src, int tgt, float val) {
    if (q->count >= 128) return 0;
    q->queue[q->tail].event_type = type;
    q->queue[q->tail].source_entity = src;
    q->queue[q->tail].target_entity = tgt;
    q->queue[q->tail].value = val;
    q->tail = (q->tail + 1) % 128;
    q->count = q->count + 1;
    return 1;
}

int ecs_evq_pop(ecs_event_queue_t *q, ecs_event_t *out) {
    if (q->count <= 0) return 0;
    *out = q->queue[q->head];
    q->head = (q->head + 1) % 128;
    q->count = q->count - 1;
    return 1;
}

int ecs_evq_peek_type(ecs_event_queue_t *q) {
    if (q->count <= 0) return -1;
    return q->queue[q->head].event_type;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1246: Should produce output");
    assert!(
        rust_code.contains("fn ecs_evq_init"),
        "C1246: Should contain ecs_evq_init"
    );
    assert!(
        rust_code.contains("fn ecs_evq_push"),
        "C1246: Should contain ecs_evq_push"
    );
    assert!(
        rust_code.contains("fn ecs_evq_pop"),
        "C1246: Should contain ecs_evq_pop"
    );
    Ok(())
}

/// C1247: Resource manager with reference counting
#[test]
fn c1247_resource_manager() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef unsigned int uint32_t;

typedef struct {
    int resource_type;
    int ref_count;
    int loaded;
    uint32_t byte_size;
} ecs_resource_t;

typedef struct {
    ecs_resource_t slots[256];
    int used_count;
    uint32_t total_bytes;
} ecs_res_mgr_t;

void ecs_resmgr_init(ecs_res_mgr_t *mgr) {
    mgr->used_count = 0;
    mgr->total_bytes = 0;
    for (int i = 0; i < 256; i = i + 1) {
        mgr->slots[i].loaded = 0;
        mgr->slots[i].ref_count = 0;
    }
}

int ecs_resmgr_load(ecs_res_mgr_t *mgr, int type, uint32_t size) {
    for (int i = 0; i < 256; i = i + 1) {
        if (mgr->slots[i].loaded == 0) {
            mgr->slots[i].resource_type = type;
            mgr->slots[i].ref_count = 1;
            mgr->slots[i].loaded = 1;
            mgr->slots[i].byte_size = size;
            mgr->used_count = mgr->used_count + 1;
            mgr->total_bytes += size;
            return i;
        }
    }
    return -1;
}

void ecs_resmgr_addref(ecs_res_mgr_t *mgr, int handle) {
    if (handle >= 0 && handle < 256 && mgr->slots[handle].loaded) {
        mgr->slots[handle].ref_count = mgr->slots[handle].ref_count + 1;
    }
}

void ecs_resmgr_release(ecs_res_mgr_t *mgr, int handle) {
    if (handle < 0 || handle >= 256) return;
    if (mgr->slots[handle].loaded == 0) return;
    mgr->slots[handle].ref_count = mgr->slots[handle].ref_count - 1;
    if (mgr->slots[handle].ref_count <= 0) {
        mgr->total_bytes -= mgr->slots[handle].byte_size;
        mgr->slots[handle].loaded = 0;
        mgr->used_count = mgr->used_count - 1;
    }
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1247: Should produce output");
    assert!(
        rust_code.contains("fn ecs_resmgr_init"),
        "C1247: Should contain ecs_resmgr_init"
    );
    assert!(
        rust_code.contains("fn ecs_resmgr_load"),
        "C1247: Should contain ecs_resmgr_load"
    );
    assert!(
        rust_code.contains("fn ecs_resmgr_release"),
        "C1247: Should contain ecs_resmgr_release"
    );
    Ok(())
}

/// C1248: Scene graph with parent-child transform propagation
#[test]
fn c1248_scene_graph() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float lx;
    float ly;
    float l_rot;
    float l_scale;
    float wx;
    float wy;
    float w_rot;
    float w_scale;
    int parent;
    int active;
} ecs_scene_node_t;

void ecs_sg_init(ecs_scene_node_t *nodes, int count) {
    for (int i = 0; i < count; i = i + 1) {
        nodes[i].lx = 0.0f;
        nodes[i].ly = 0.0f;
        nodes[i].l_rot = 0.0f;
        nodes[i].l_scale = 1.0f;
        nodes[i].wx = 0.0f;
        nodes[i].wy = 0.0f;
        nodes[i].w_rot = 0.0f;
        nodes[i].w_scale = 1.0f;
        nodes[i].parent = -1;
        nodes[i].active = 0;
    }
}

void ecs_sg_propagate(ecs_scene_node_t *nodes, int count) {
    for (int i = 0; i < count; i = i + 1) {
        if (nodes[i].active == 0) continue;
        if (nodes[i].parent < 0) {
            nodes[i].wx = nodes[i].lx;
            nodes[i].wy = nodes[i].ly;
            nodes[i].w_rot = nodes[i].l_rot;
            nodes[i].w_scale = nodes[i].l_scale;
        } else {
            int p = nodes[i].parent;
            nodes[i].w_scale = nodes[p].w_scale * nodes[i].l_scale;
            nodes[i].w_rot = nodes[p].w_rot + nodes[i].l_rot;
            nodes[i].wx = nodes[p].wx + nodes[i].lx * nodes[p].w_scale;
            nodes[i].wy = nodes[p].wy + nodes[i].ly * nodes[p].w_scale;
        }
    }
}

int ecs_sg_create(ecs_scene_node_t *nodes, int max_nodes, int parent) {
    for (int i = 0; i < max_nodes; i = i + 1) {
        if (nodes[i].active == 0) {
            nodes[i].active = 1;
            nodes[i].parent = parent;
            nodes[i].l_scale = 1.0f;
            return i;
        }
    }
    return -1;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1248: Should produce output");
    assert!(
        rust_code.contains("fn ecs_sg_init"),
        "C1248: Should contain ecs_sg_init"
    );
    assert!(
        rust_code.contains("fn ecs_sg_propagate"),
        "C1248: Should contain ecs_sg_propagate"
    );
    assert!(
        rust_code.contains("fn ecs_sg_create"),
        "C1248: Should contain ecs_sg_create"
    );
    Ok(())
}

/// C1249: Object pooling with free list
#[test]
fn c1249_object_pooling() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    float data[4];
    int active;
    int next_free;
} ecs_pool_obj_t;

typedef struct {
    ecs_pool_obj_t objects[512];
    int first_free;
    int active_count;
    int capacity;
} ecs_obj_pool_t;

void ecs_objpool_init(ecs_obj_pool_t *pool) {
    pool->first_free = 0;
    pool->active_count = 0;
    pool->capacity = 512;
    for (int i = 0; i < 512; i = i + 1) {
        pool->objects[i].active = 0;
        pool->objects[i].next_free = i + 1;
    }
    pool->objects[511].next_free = -1;
}

int ecs_objpool_alloc(ecs_obj_pool_t *pool) {
    if (pool->first_free < 0) return -1;
    int idx = pool->first_free;
    pool->first_free = pool->objects[idx].next_free;
    pool->objects[idx].active = 1;
    pool->objects[idx].next_free = -1;
    pool->active_count = pool->active_count + 1;
    return idx;
}

void ecs_objpool_free(ecs_obj_pool_t *pool, int idx) {
    if (idx < 0 || idx >= pool->capacity) return;
    if (pool->objects[idx].active == 0) return;
    pool->objects[idx].active = 0;
    pool->objects[idx].next_free = pool->first_free;
    pool->first_free = idx;
    pool->active_count = pool->active_count - 1;
}

int ecs_objpool_active(ecs_obj_pool_t *pool) {
    return pool->active_count;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1249: Should produce output");
    assert!(
        rust_code.contains("fn ecs_objpool_init"),
        "C1249: Should contain ecs_objpool_init"
    );
    assert!(
        rust_code.contains("fn ecs_objpool_alloc"),
        "C1249: Should contain ecs_objpool_alloc"
    );
    assert!(
        rust_code.contains("fn ecs_objpool_free"),
        "C1249: Should contain ecs_objpool_free"
    );
    Ok(())
}

/// C1250: Command buffer for deferred operations
#[test]
fn c1250_command_buffer() -> Result<(), Box<dyn std::error::Error>> {
    let c_code = r#"
typedef struct {
    int cmd_type;
    int entity_id;
    int component_id;
    float value;
} ecs_command_t;

typedef struct {
    ecs_command_t commands[256];
    int count;
    int flushed;
} ecs_cmd_buffer_t;

void ecs_cmdbuf_init(ecs_cmd_buffer_t *buf) {
    buf->count = 0;
    buf->flushed = 0;
}

int ecs_cmdbuf_add(ecs_cmd_buffer_t *buf, int type, int entity,
                   int component, float value) {
    if (buf->count >= 256) return 0;
    int idx = buf->count;
    buf->commands[idx].cmd_type = type;
    buf->commands[idx].entity_id = entity;
    buf->commands[idx].component_id = component;
    buf->commands[idx].value = value;
    buf->count = buf->count + 1;
    return 1;
}

int ecs_cmdbuf_count(ecs_cmd_buffer_t *buf) {
    return buf->count;
}

void ecs_cmdbuf_clear(ecs_cmd_buffer_t *buf) {
    buf->count = 0;
    buf->flushed = buf->flushed + 1;
}

int ecs_cmdbuf_get_type(ecs_cmd_buffer_t *buf, int idx) {
    if (idx < 0 || idx >= buf->count) return -1;
    return buf->commands[idx].cmd_type;
}
"#;
    let rust_code = decy_core::transpile(c_code)?;
    assert!(!rust_code.is_empty(), "C1250: Should produce output");
    assert!(
        rust_code.contains("fn ecs_cmdbuf_init"),
        "C1250: Should contain ecs_cmdbuf_init"
    );
    assert!(
        rust_code.contains("fn ecs_cmdbuf_add"),
        "C1250: Should contain ecs_cmdbuf_add"
    );
    assert!(
        rust_code.contains("fn ecs_cmdbuf_clear"),
        "C1250: Should contain ecs_cmdbuf_clear"
    );
    Ok(())
}
