//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1451-C1475: Compiler Backend / Code Generation Patterns -- register
//! allocators, instruction selectors, control flow graph manipulations,
//! code emitters, and backend optimization passes.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise real-world compiler backend patterns commonly
//! found in LLVM, GCC, Cranelift, and similar code generators -- all
//! expressed as valid C99.
//!
//! Organization:
//! - C1451-C1455: Register allocation (linear scan, graph coloring, spill, live range, coalescing)
//! - C1456-C1460: Instruction selection (pattern match, tree tiling, strength reduce, addr mode, imm fold)
//! - C1461-C1465: Control flow (basic block layout, branch opt, loop detect, phi insert, dominator tree)
//! - C1466-C1470: Code emission (instruction encoder, relocation, symbol table, section, alignment)
//! - C1471-C1475: Optimization (DCE, CSE, LICM, inlining heuristic, tail call optimization)

// ============================================================================
// C1451-C1455: Register Allocation
// ============================================================================

/// C1451: Linear scan register allocator with interval tracking
#[test]
fn c1451_linear_scan_regalloc() {
    let c_code = r#"
#define CB_MAX_INTERVALS 64
#define CB_NUM_REGS 16

typedef struct {
    int vreg;
    int start;
    int end;
    int phys_reg;
} cb_interval_t;

typedef struct {
    cb_interval_t intervals[CB_MAX_INTERVALS];
    int count;
    int reg_busy_until[CB_NUM_REGS];
} cb_linscan_t;

void cb_linscan_init(cb_linscan_t *ls) {
    int i;
    ls->count = 0;
    for (i = 0; i < CB_NUM_REGS; i++)
        ls->reg_busy_until[i] = -1;
}

int cb_linscan_add(cb_linscan_t *ls, int vreg, int start, int end) {
    if (ls->count >= CB_MAX_INTERVALS) return -1;
    ls->intervals[ls->count].vreg = vreg;
    ls->intervals[ls->count].start = start;
    ls->intervals[ls->count].end = end;
    ls->intervals[ls->count].phys_reg = -1;
    ls->count++;
    return 0;
}

int cb_linscan_alloc(cb_linscan_t *ls, int idx) {
    int i;
    int best = -1;
    int best_end = 999999;
    for (i = 0; i < CB_NUM_REGS; i++) {
        if (ls->reg_busy_until[i] < ls->intervals[idx].start) {
            if (best < 0 || ls->reg_busy_until[i] < best_end) {
                best = i;
                best_end = ls->reg_busy_until[i];
            }
        }
    }
    if (best >= 0) {
        ls->intervals[idx].phys_reg = best;
        ls->reg_busy_until[best] = ls->intervals[idx].end;
    }
    return best;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1451: Linear scan regalloc should transpile: {:?}", result.err());
}

/// C1452: Graph coloring register allocator with interference
#[test]
fn c1452_graph_coloring_regalloc() {
    let c_code = r#"
#define CB_GC_MAX 32
#define CB_GC_REGS 8

typedef struct {
    int adj[CB_GC_MAX][CB_GC_MAX];
    int color[CB_GC_MAX];
    int degree[CB_GC_MAX];
    int node_count;
} cb_coloring_t;

void cb_coloring_init(cb_coloring_t *g, int n) {
    int i; int j;
    g->node_count = n;
    for (i = 0; i < n; i++) {
        g->color[i] = -1;
        g->degree[i] = 0;
        for (j = 0; j < n; j++)
            g->adj[i][j] = 0;
    }
}

void cb_coloring_add_edge(cb_coloring_t *g, int u, int v) {
    if (!g->adj[u][v]) {
        g->adj[u][v] = 1;
        g->adj[v][u] = 1;
        g->degree[u]++;
        g->degree[v]++;
    }
}

int cb_coloring_assign(cb_coloring_t *g, int node) {
    int used[CB_GC_REGS];
    int i;
    for (i = 0; i < CB_GC_REGS; i++) used[i] = 0;
    for (i = 0; i < g->node_count; i++) {
        if (g->adj[node][i] && g->color[i] >= 0 && g->color[i] < CB_GC_REGS)
            used[g->color[i]] = 1;
    }
    for (i = 0; i < CB_GC_REGS; i++) {
        if (!used[i]) { g->color[node] = i; return i; }
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1452: Graph coloring regalloc should transpile: {:?}", result.err());
}

/// C1453: Spill slot management for register pressure
#[test]
fn c1453_spill_management() {
    let c_code = r#"
#define CB_MAX_SPILLS 128

typedef struct {
    int offset;
    int size;
    int vreg;
    int in_use;
} cb_spill_slot_t;

typedef struct {
    cb_spill_slot_t slots[CB_MAX_SPILLS];
    int count;
    int frame_size;
} cb_spill_mgr_t;

void cb_spill_init(cb_spill_mgr_t *m) {
    m->count = 0;
    m->frame_size = 0;
}

int cb_spill_allocate(cb_spill_mgr_t *m, int vreg, int size) {
    int i;
    for (i = 0; i < m->count; i++) {
        if (!m->slots[i].in_use && m->slots[i].size >= size) {
            m->slots[i].in_use = 1;
            m->slots[i].vreg = vreg;
            return m->slots[i].offset;
        }
    }
    if (m->count >= CB_MAX_SPILLS) return -1;
    m->slots[m->count].offset = m->frame_size;
    m->slots[m->count].size = (size + 7) & ~7;
    m->slots[m->count].vreg = vreg;
    m->slots[m->count].in_use = 1;
    m->frame_size += m->slots[m->count].size;
    m->count++;
    return m->slots[m->count - 1].offset;
}

void cb_spill_release(cb_spill_mgr_t *m, int vreg) {
    int i;
    for (i = 0; i < m->count; i++) {
        if (m->slots[i].vreg == vreg) {
            m->slots[i].in_use = 0;
            m->slots[i].vreg = -1;
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1453: Spill management should transpile: {:?}", result.err());
}

/// C1454: Live range analysis with def-use chains
#[test]
fn c1454_live_range_analysis() {
    let c_code = r#"
#define CB_LR_MAX 64

typedef struct {
    int vreg;
    int def_point;
    int last_use;
    int num_uses;
    int is_loop_carried;
} cb_live_range_t;

typedef struct {
    cb_live_range_t ranges[CB_LR_MAX];
    int count;
} cb_liveness_t;

void cb_liveness_init(cb_liveness_t *lv) {
    lv->count = 0;
}

int cb_liveness_define(cb_liveness_t *lv, int vreg, int point) {
    if (lv->count >= CB_LR_MAX) return -1;
    lv->ranges[lv->count].vreg = vreg;
    lv->ranges[lv->count].def_point = point;
    lv->ranges[lv->count].last_use = point;
    lv->ranges[lv->count].num_uses = 0;
    lv->ranges[lv->count].is_loop_carried = 0;
    lv->count++;
    return lv->count - 1;
}

void cb_liveness_use(cb_liveness_t *lv, int vreg, int point) {
    int i;
    for (i = lv->count - 1; i >= 0; i--) {
        if (lv->ranges[i].vreg == vreg) {
            if (point > lv->ranges[i].last_use)
                lv->ranges[i].last_use = point;
            lv->ranges[i].num_uses++;
            if (point < lv->ranges[i].def_point)
                lv->ranges[i].is_loop_carried = 1;
            return;
        }
    }
}

int cb_liveness_weight(cb_liveness_t *lv, int idx) {
    int len = lv->ranges[idx].last_use - lv->ranges[idx].def_point + 1;
    int uses = lv->ranges[idx].num_uses;
    if (len <= 0) return 0;
    return (uses * 100) / len;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1454: Live range analysis should transpile: {:?}", result.err());
}

/// C1455: Register coalescing for move elimination
#[test]
fn c1455_register_coalescing() {
    let c_code = r#"
#define CB_COAL_MAX 64

typedef struct {
    int src;
    int dst;
    int weight;
    int coalesced;
} cb_move_t;

typedef struct {
    cb_move_t moves[CB_COAL_MAX];
    int count;
    int alias[CB_COAL_MAX];
} cb_coalesce_t;

void cb_coalesce_init(cb_coalesce_t *c) {
    int i;
    c->count = 0;
    for (i = 0; i < CB_COAL_MAX; i++)
        c->alias[i] = i;
}

int cb_coalesce_find(cb_coalesce_t *c, int reg) {
    while (c->alias[reg] != reg)
        reg = c->alias[reg];
    return reg;
}

void cb_coalesce_add_move(cb_coalesce_t *c, int src, int dst, int weight) {
    if (c->count >= CB_COAL_MAX) return;
    c->moves[c->count].src = src;
    c->moves[c->count].dst = dst;
    c->moves[c->count].weight = weight;
    c->moves[c->count].coalesced = 0;
    c->count++;
}

int cb_coalesce_try(cb_coalesce_t *c, int idx) {
    int s = cb_coalesce_find(c, c->moves[idx].src);
    int d = cb_coalesce_find(c, c->moves[idx].dst);
    if (s == d) { c->moves[idx].coalesced = 1; return 1; }
    c->alias[d] = s;
    c->moves[idx].coalesced = 1;
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1455: Register coalescing should transpile: {:?}", result.err());
}

// ============================================================================
// C1456-C1460: Instruction Selection
// ============================================================================

/// C1456: Pattern matching instruction selector
#[test]
fn c1456_pattern_matching_isel() {
    let c_code = r#"
#define CB_OP_ADD  0
#define CB_OP_MUL  1
#define CB_OP_LOAD 2
#define CB_OP_STORE 3
#define CB_OP_SHIFT 4
#define CB_OP_MADD 5

typedef struct {
    int opcode;
    int dst;
    int src1;
    int src2;
    int imm;
} cb_isel_node_t;

typedef struct {
    int opcode;
    int dst;
    int src1;
    int src2;
    int imm;
} cb_machine_inst_t;

int cb_isel_match_madd(cb_isel_node_t *a, cb_isel_node_t *b) {
    if (a->opcode == CB_OP_MUL && b->opcode == CB_OP_ADD)
        if (b->src1 == a->dst || b->src2 == a->dst)
            return 1;
    return 0;
}

cb_machine_inst_t cb_isel_select(cb_isel_node_t *node) {
    cb_machine_inst_t mi;
    mi.dst = node->dst;
    mi.src1 = node->src1;
    mi.src2 = node->src2;
    mi.imm = node->imm;
    if (node->opcode == CB_OP_ADD && node->imm != 0)
        mi.opcode = 10;
    else if (node->opcode == CB_OP_MUL && (node->src2 & (node->src2 - 1)) == 0)
        mi.opcode = CB_OP_SHIFT;
    else
        mi.opcode = node->opcode;
    return mi;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1456: Pattern matching isel should transpile: {:?}", result.err());
}

/// C1457: Tree tiling instruction selector with cost model
#[test]
#[ignore = "FALSIFIED: invalid C - struct member access ctx->nodes[idx].nodes is not valid (no nested nodes member on cb_tree_node_t)"]
fn c1457_tree_tiling_isel() {
    let c_code = r#"
#define CB_TILE_MAX 32
#define CB_NODE_ADD 0
#define CB_NODE_MUL 1
#define CB_NODE_LOAD 2
#define CB_NODE_CONST 3

typedef struct cb_tree_node {
    int kind;
    int value;
    int left;
    int right;
    int cost;
    int tile_id;
} cb_tree_node_t;

typedef struct {
    cb_tree_node_t nodes[CB_TILE_MAX];
    int count;
} cb_tile_ctx_t;

int cb_tile_add_node(cb_tile_ctx_t *ctx, int kind, int val, int l, int r) {
    if (ctx->count >= CB_TILE_MAX) return -1;
    ctx->nodes[ctx->count].kind = kind;
    ctx->nodes[ctx->count].value = val;
    ctx->nodes[ctx->count].left = l;
    ctx->nodes[ctx->count].right = r;
    ctx->nodes[ctx->count].cost = 999;
    ctx->nodes[ctx->count].tile_id = -1;
    return ctx->count++;
}

int cb_tile_cost(cb_tile_ctx_t *ctx, int idx) {
    int lc = 0; int rc = 0;
    if (idx < 0 || idx >= ctx->count) return 999;
    if (ctx->nodes[idx].left >= 0)
        lc = cb_tile_cost(ctx, ctx->nodes[idx].left);
    if (ctx->nodes[idx].right >= 0)
        rc = cb_tile_cost(ctx, ctx->nodes[idx].right);
    ctx->nodes[idx].cost = 1 + lc + rc;
    if (ctx->nodes[idx].kind == CB_NODE_LOAD && ctx->nodes[idx].left >= 0 &&
        ctx->nodes[idx].nodes[ctx->nodes[idx].left].kind == CB_NODE_ADD)
        ctx->nodes[idx].cost = lc + rc;
    return ctx->nodes[idx].cost;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1457: Tree tiling isel should transpile: {:?}", result.err());
}

/// C1458: Strength reduction (multiply to shift, division to reciprocal)
#[test]
fn c1458_strength_reduction() {
    let c_code = r#"
typedef unsigned int uint32_t;

int cb_is_power_of_two(uint32_t x) {
    return x > 0 && (x & (x - 1)) == 0;
}

int cb_log2_floor(uint32_t x) {
    int r = 0;
    while (x > 1) { x >>= 1; r++; }
    return r;
}

uint32_t cb_strength_reduce_mul(uint32_t a, uint32_t b) {
    if (b == 0) return 0;
    if (b == 1) return a;
    if (cb_is_power_of_two(b))
        return a << cb_log2_floor(b);
    if (cb_is_power_of_two(b - 1))
        return (a << cb_log2_floor(b - 1)) + a;
    if (cb_is_power_of_two(b + 1))
        return (a << cb_log2_floor(b + 1)) - a;
    return a * b;
}

uint32_t cb_strength_reduce_div(uint32_t a, uint32_t b) {
    if (b == 1) return a;
    if (cb_is_power_of_two(b))
        return a >> cb_log2_floor(b);
    return a / b;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1458: Strength reduction should transpile: {:?}", result.err());
}

/// C1459: Addressing mode selection for memory ops
#[test]
fn c1459_addressing_mode_selection() {
    let c_code = r#"
#define CB_ADDR_DIRECT   0
#define CB_ADDR_INDEXED  1
#define CB_ADDR_BASED    2
#define CB_ADDR_SCALED   3

typedef struct {
    int mode;
    int base_reg;
    int index_reg;
    int scale;
    int offset;
} cb_addr_mode_t;

cb_addr_mode_t cb_addr_select(int base, int index, int scale, int off) {
    cb_addr_mode_t am;
    am.base_reg = base;
    am.index_reg = index;
    am.scale = scale;
    am.offset = off;
    if (index < 0 && off == 0)
        am.mode = CB_ADDR_DIRECT;
    else if (index >= 0 && scale > 1)
        am.mode = CB_ADDR_SCALED;
    else if (index >= 0)
        am.mode = CB_ADDR_INDEXED;
    else
        am.mode = CB_ADDR_BASED;
    return am;
}

int cb_addr_encode(cb_addr_mode_t *am) {
    int enc = am->mode << 24;
    enc |= (am->base_reg & 0xF) << 20;
    enc |= (am->index_reg & 0xF) << 16;
    enc |= (am->scale & 0x3) << 14;
    enc |= am->offset & 0x3FFF;
    return enc;
}

int cb_addr_is_simple(cb_addr_mode_t *am) {
    return am->mode == CB_ADDR_DIRECT || am->mode == CB_ADDR_BASED;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1459: Addressing mode selection should transpile: {:?}", result.err());
}

/// C1460: Immediate folding and constant propagation in isel
#[test]
fn c1460_immediate_folding() {
    let c_code = r#"
#define CB_IMM_NONE   0
#define CB_IMM_SMALL  1
#define CB_IMM_LARGE  2
#define CB_IMM_SPLIT  3

typedef struct {
    int op;
    int reg;
    int imm_kind;
    int imm_lo;
    int imm_hi;
} cb_imm_inst_t;

int cb_imm_fits_12(int val) {
    return val >= -2048 && val <= 2047;
}

int cb_imm_fits_20(int val) {
    return val >= -(1 << 19) && val < (1 << 19);
}

cb_imm_inst_t cb_imm_fold(int op, int reg, int imm) {
    cb_imm_inst_t inst;
    inst.op = op;
    inst.reg = reg;
    if (imm == 0) {
        inst.imm_kind = CB_IMM_NONE;
        inst.imm_lo = 0;
        inst.imm_hi = 0;
    } else if (cb_imm_fits_12(imm)) {
        inst.imm_kind = CB_IMM_SMALL;
        inst.imm_lo = imm;
        inst.imm_hi = 0;
    } else if (cb_imm_fits_20(imm)) {
        inst.imm_kind = CB_IMM_LARGE;
        inst.imm_lo = imm;
        inst.imm_hi = 0;
    } else {
        inst.imm_kind = CB_IMM_SPLIT;
        inst.imm_lo = imm & 0xFFF;
        inst.imm_hi = (imm >> 12) & 0xFFFFF;
    }
    return inst;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1460: Immediate folding should transpile: {:?}", result.err());
}

// ============================================================================
// C1461-C1465: Control Flow
// ============================================================================

/// C1461: Basic block layout with fall-through optimization
#[test]
fn c1461_basic_block_layout() {
    let c_code = r#"
#define CB_BB_MAX 64
#define CB_SUCC_MAX 4

typedef struct {
    int id;
    int inst_start;
    int inst_end;
    int succs[CB_SUCC_MAX];
    int num_succs;
    int exec_count;
    int placed;
} cb_basic_block_t;

typedef struct {
    cb_basic_block_t blocks[CB_BB_MAX];
    int count;
    int layout[CB_BB_MAX];
    int layout_len;
} cb_layout_t;

void cb_layout_init(cb_layout_t *l) {
    l->count = 0;
    l->layout_len = 0;
}

int cb_layout_add_bb(cb_layout_t *l, int inst_s, int inst_e) {
    if (l->count >= CB_BB_MAX) return -1;
    l->blocks[l->count].id = l->count;
    l->blocks[l->count].inst_start = inst_s;
    l->blocks[l->count].inst_end = inst_e;
    l->blocks[l->count].num_succs = 0;
    l->blocks[l->count].exec_count = 0;
    l->blocks[l->count].placed = 0;
    return l->count++;
}

void cb_layout_place(cb_layout_t *l) {
    int i; int best; int best_count;
    int current = 0;
    l->blocks[0].placed = 1;
    l->layout[l->layout_len++] = 0;
    while (l->layout_len < l->count) {
        best = -1;
        best_count = -1;
        for (i = 0; i < l->blocks[current].num_succs; i++) {
            int s = l->blocks[current].succs[i];
            if (!l->blocks[s].placed && l->blocks[s].exec_count > best_count) {
                best = s;
                best_count = l->blocks[s].exec_count;
            }
        }
        if (best < 0) {
            for (i = 0; i < l->count; i++) {
                if (!l->blocks[i].placed) { best = i; break; }
            }
        }
        if (best < 0) break;
        l->blocks[best].placed = 1;
        l->layout[l->layout_len++] = best;
        current = best;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1461: Basic block layout should transpile: {:?}", result.err());
}

/// C1462: Branch optimization (conditional to unconditional, inversion)
#[test]
fn c1462_branch_optimization() {
    let c_code = r#"
#define CB_BR_COND   0
#define CB_BR_UNCOND 1
#define CB_BR_NEVER  2

typedef struct {
    int kind;
    int cond_reg;
    int target;
    int fallthrough;
    int inverted;
} cb_branch_t;

cb_branch_t cb_branch_optimize(int kind, int cond, int tgt, int ft) {
    cb_branch_t br;
    br.kind = kind;
    br.cond_reg = cond;
    br.target = tgt;
    br.fallthrough = ft;
    br.inverted = 0;
    if (kind == CB_BR_COND && tgt == ft) {
        br.kind = CB_BR_NEVER;
    } else if (kind == CB_BR_COND && ft == tgt + 1) {
        br.kind = CB_BR_UNCOND;
        br.target = tgt;
    }
    return br;
}

cb_branch_t cb_branch_invert(cb_branch_t *br) {
    cb_branch_t inv;
    inv.kind = br->kind;
    inv.cond_reg = br->cond_reg;
    inv.target = br->fallthrough;
    inv.fallthrough = br->target;
    inv.inverted = !br->inverted;
    return inv;
}

int cb_branch_can_eliminate(cb_branch_t *br) {
    return br->kind == CB_BR_NEVER || (br->kind == CB_BR_UNCOND && br->target == br->fallthrough);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1462: Branch optimization should transpile: {:?}", result.err());
}

/// C1463: Loop detection via back-edge identification
#[test]
fn c1463_loop_detection() {
    let c_code = r#"
#define CB_LOOP_MAX 32
#define CB_CFG_MAX 64

typedef struct {
    int header;
    int back_edge_src;
    int depth;
    int body[CB_CFG_MAX];
    int body_count;
} cb_loop_t;

typedef struct {
    int succs[CB_CFG_MAX][4];
    int num_succs[CB_CFG_MAX];
    int visited[CB_CFG_MAX];
    int in_stack[CB_CFG_MAX];
    int num_nodes;
    cb_loop_t loops[CB_LOOP_MAX];
    int loop_count;
} cb_loop_ctx_t;

void cb_loop_init(cb_loop_ctx_t *ctx, int n) {
    int i;
    ctx->num_nodes = n;
    ctx->loop_count = 0;
    for (i = 0; i < n; i++) {
        ctx->num_succs[i] = 0;
        ctx->visited[i] = 0;
        ctx->in_stack[i] = 0;
    }
}

void cb_loop_dfs(cb_loop_ctx_t *ctx, int node) {
    int i;
    ctx->visited[node] = 1;
    ctx->in_stack[node] = 1;
    for (i = 0; i < ctx->num_succs[node]; i++) {
        int s = ctx->succs[node][i];
        if (!ctx->visited[s])
            cb_loop_dfs(ctx, s);
        else if (ctx->in_stack[s] && ctx->loop_count < CB_LOOP_MAX) {
            ctx->loops[ctx->loop_count].header = s;
            ctx->loops[ctx->loop_count].back_edge_src = node;
            ctx->loops[ctx->loop_count].depth = 1;
            ctx->loops[ctx->loop_count].body_count = 0;
            ctx->loop_count++;
        }
    }
    ctx->in_stack[node] = 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1463: Loop detection should transpile: {:?}", result.err());
}

/// C1464: Phi node insertion for SSA construction
#[test]
fn c1464_phi_node_insertion() {
    let c_code = r#"
#define CB_PHI_MAX 8
#define CB_SSA_MAX 64

typedef struct {
    int var;
    int sources[CB_PHI_MAX];
    int blocks[CB_PHI_MAX];
    int num_sources;
    int result_ver;
} cb_phi_node_t;

typedef struct {
    cb_phi_node_t phis[CB_SSA_MAX];
    int phi_count;
    int version_counter[CB_SSA_MAX];
} cb_ssa_ctx_t;

void cb_ssa_init(cb_ssa_ctx_t *ctx) {
    int i;
    ctx->phi_count = 0;
    for (i = 0; i < CB_SSA_MAX; i++)
        ctx->version_counter[i] = 0;
}

int cb_ssa_new_version(cb_ssa_ctx_t *ctx, int var) {
    return ctx->version_counter[var]++;
}

int cb_ssa_insert_phi(cb_ssa_ctx_t *ctx, int var, int block) {
    int idx;
    if (ctx->phi_count >= CB_SSA_MAX) return -1;
    idx = ctx->phi_count++;
    ctx->phis[idx].var = var;
    ctx->phis[idx].num_sources = 0;
    ctx->phis[idx].result_ver = cb_ssa_new_version(ctx, var);
    return idx;
}

void cb_ssa_add_phi_source(cb_ssa_ctx_t *ctx, int phi_idx, int ver, int blk) {
    int n;
    if (phi_idx < 0 || phi_idx >= ctx->phi_count) return;
    n = ctx->phis[phi_idx].num_sources;
    if (n >= CB_PHI_MAX) return;
    ctx->phis[phi_idx].sources[n] = ver;
    ctx->phis[phi_idx].blocks[n] = blk;
    ctx->phis[phi_idx].num_sources++;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1464: Phi node insertion should transpile: {:?}", result.err());
}

/// C1465: Dominator tree computation
#[test]
fn c1465_dominator_tree() {
    let c_code = r#"
#define CB_DOM_MAX 64

typedef struct {
    int idom[CB_DOM_MAX];
    int preds[CB_DOM_MAX][4];
    int num_preds[CB_DOM_MAX];
    int rpo[CB_DOM_MAX];
    int num_nodes;
} cb_dom_tree_t;

void cb_dom_init(cb_dom_tree_t *d, int n) {
    int i;
    d->num_nodes = n;
    for (i = 0; i < n; i++) {
        d->idom[i] = -1;
        d->num_preds[i] = 0;
    }
    d->idom[0] = 0;
}

int cb_dom_intersect(cb_dom_tree_t *d, int a, int b) {
    while (a != b) {
        while (a > b) a = d->idom[a];
        while (b > a) b = d->idom[b];
    }
    return a;
}

void cb_dom_compute(cb_dom_tree_t *d) {
    int changed = 1;
    int i; int j;
    while (changed) {
        changed = 0;
        for (i = 1; i < d->num_nodes; i++) {
            int new_idom = -1;
            for (j = 0; j < d->num_preds[i]; j++) {
                int p = d->preds[i][j];
                if (d->idom[p] >= 0) {
                    if (new_idom < 0)
                        new_idom = p;
                    else
                        new_idom = cb_dom_intersect(d, new_idom, p);
                }
            }
            if (new_idom >= 0 && d->idom[i] != new_idom) {
                d->idom[i] = new_idom;
                changed = 1;
            }
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1465: Dominator tree should transpile: {:?}", result.err());
}

// ============================================================================
// C1466-C1470: Code Emission
// ============================================================================

/// C1466: Instruction encoder for fixed-width ISA
#[test]
fn c1466_instruction_encoder() {
    let c_code = r#"
typedef unsigned int uint32_t;

#define CB_ENC_R 0
#define CB_ENC_I 1
#define CB_ENC_S 2
#define CB_ENC_B 3

typedef struct {
    int format;
    int opcode;
    int rd;
    int rs1;
    int rs2;
    int imm;
} cb_enc_inst_t;

uint32_t cb_encode_r(cb_enc_inst_t *inst) {
    uint32_t w = 0;
    w |= (inst->opcode & 0x7F);
    w |= (inst->rd & 0x1F) << 7;
    w |= (inst->rs1 & 0x1F) << 15;
    w |= (inst->rs2 & 0x1F) << 20;
    return w;
}

uint32_t cb_encode_i(cb_enc_inst_t *inst) {
    uint32_t w = 0;
    w |= (inst->opcode & 0x7F);
    w |= (inst->rd & 0x1F) << 7;
    w |= (inst->rs1 & 0x1F) << 15;
    w |= (inst->imm & 0xFFF) << 20;
    return w;
}

uint32_t cb_encode(cb_enc_inst_t *inst) {
    if (inst->format == CB_ENC_R)
        return cb_encode_r(inst);
    else if (inst->format == CB_ENC_I)
        return cb_encode_i(inst);
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1466: Instruction encoder should transpile: {:?}", result.err());
}

/// C1467: Relocation table for link-time fixups
#[test]
fn c1467_relocation_table() {
    let c_code = r#"
#define CB_RELOC_MAX 256
#define CB_REL_ABS32  0
#define CB_REL_REL32  1
#define CB_REL_PCREL  2

typedef struct {
    int offset;
    int type;
    int symbol_idx;
    int addend;
} cb_reloc_t;

typedef struct {
    cb_reloc_t entries[CB_RELOC_MAX];
    int count;
} cb_reloc_table_t;

void cb_reloc_init(cb_reloc_table_t *rt) {
    rt->count = 0;
}

int cb_reloc_add(cb_reloc_table_t *rt, int off, int type, int sym, int add) {
    if (rt->count >= CB_RELOC_MAX) return -1;
    rt->entries[rt->count].offset = off;
    rt->entries[rt->count].type = type;
    rt->entries[rt->count].symbol_idx = sym;
    rt->entries[rt->count].addend = add;
    rt->count++;
    return 0;
}

int cb_reloc_apply(cb_reloc_table_t *rt, int idx, int sym_addr, int pc) {
    int val = 0;
    if (idx < 0 || idx >= rt->count) return 0;
    if (rt->entries[idx].type == CB_REL_ABS32)
        val = sym_addr + rt->entries[idx].addend;
    else if (rt->entries[idx].type == CB_REL_REL32)
        val = sym_addr - rt->entries[idx].offset + rt->entries[idx].addend;
    else if (rt->entries[idx].type == CB_REL_PCREL)
        val = sym_addr - pc + rt->entries[idx].addend;
    return val;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1467: Relocation table should transpile: {:?}", result.err());
}

/// C1468: Symbol table with name hashing
#[test]
fn c1468_symbol_table() {
    let c_code = r#"
#define CB_SYM_MAX 128
#define CB_SYM_LOCAL  0
#define CB_SYM_GLOBAL 1
#define CB_SYM_EXTERN 2

typedef struct {
    int name_hash;
    int value;
    int size;
    int binding;
    int section;
    int defined;
} cb_symbol_t;

typedef struct {
    cb_symbol_t syms[CB_SYM_MAX];
    int count;
} cb_symtab_t;

void cb_symtab_init(cb_symtab_t *st) {
    st->count = 0;
}

int cb_symtab_hash(const char *name) {
    int h = 5381;
    while (*name) {
        h = ((h << 5) + h) + *name;
        name++;
    }
    return h;
}

int cb_symtab_add(cb_symtab_t *st, const char *name, int val, int bind) {
    int h;
    if (st->count >= CB_SYM_MAX) return -1;
    h = cb_symtab_hash(name);
    st->syms[st->count].name_hash = h;
    st->syms[st->count].value = val;
    st->syms[st->count].size = 0;
    st->syms[st->count].binding = bind;
    st->syms[st->count].section = 0;
    st->syms[st->count].defined = (bind != CB_SYM_EXTERN);
    return st->count++;
}

int cb_symtab_find(cb_symtab_t *st, const char *name) {
    int h = cb_symtab_hash(name);
    int i;
    for (i = 0; i < st->count; i++) {
        if (st->syms[i].name_hash == h)
            return i;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1468: Symbol table should transpile: {:?}", result.err());
}

/// C1469: Section builder for object file layout
#[test]
fn c1469_section_builder() {
    let c_code = r#"
#define CB_SEC_MAX 16
#define CB_SEC_CODE 0
#define CB_SEC_DATA 1
#define CB_SEC_RODATA 2
#define CB_SEC_BSS 3

typedef struct {
    int type;
    int offset;
    int size;
    int align;
    int flags;
} cb_section_t;

typedef struct {
    cb_section_t sections[CB_SEC_MAX];
    int count;
    int total_size;
} cb_sec_builder_t;

void cb_sec_init(cb_sec_builder_t *sb) {
    sb->count = 0;
    sb->total_size = 0;
}

int cb_sec_add(cb_sec_builder_t *sb, int type, int size, int align, int flags) {
    int pad;
    if (sb->count >= CB_SEC_MAX) return -1;
    pad = (align - (sb->total_size % align)) % align;
    sb->total_size += pad;
    sb->sections[sb->count].type = type;
    sb->sections[sb->count].offset = sb->total_size;
    sb->sections[sb->count].size = size;
    sb->sections[sb->count].align = align;
    sb->sections[sb->count].flags = flags;
    sb->total_size += size;
    return sb->count++;
}

int cb_sec_find(cb_sec_builder_t *sb, int type) {
    int i;
    for (i = 0; i < sb->count; i++) {
        if (sb->sections[i].type == type) return i;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1469: Section builder should transpile: {:?}", result.err());
}

/// C1470: Alignment and padding for code emission
#[test]
fn c1470_alignment_padding() {
    let c_code = r#"
typedef unsigned char uint8_t;
typedef unsigned int uint32_t;

#define CB_NOP_1BYTE  0x90
#define CB_BUF_MAX    4096

typedef struct {
    uint8_t buf[CB_BUF_MAX];
    int pos;
} cb_emit_buf_t;

void cb_emit_init(cb_emit_buf_t *eb) {
    eb->pos = 0;
}

void cb_emit_byte(cb_emit_buf_t *eb, uint8_t b) {
    if (eb->pos < CB_BUF_MAX)
        eb->buf[eb->pos++] = b;
}

void cb_emit_align(cb_emit_buf_t *eb, int align) {
    while (eb->pos % align != 0)
        cb_emit_byte(eb, CB_NOP_1BYTE);
}

void cb_emit_word(cb_emit_buf_t *eb, uint32_t w) {
    cb_emit_byte(eb, (uint8_t)(w & 0xFF));
    cb_emit_byte(eb, (uint8_t)((w >> 8) & 0xFF));
    cb_emit_byte(eb, (uint8_t)((w >> 16) & 0xFF));
    cb_emit_byte(eb, (uint8_t)((w >> 24) & 0xFF));
}

void cb_emit_pad(cb_emit_buf_t *eb, int n) {
    int i;
    for (i = 0; i < n && eb->pos < CB_BUF_MAX; i++)
        cb_emit_byte(eb, 0);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1470: Alignment padding should transpile: {:?}", result.err());
}

// ============================================================================
// C1471-C1475: Optimization
// ============================================================================

/// C1471: Dead code elimination via liveness
#[test]
fn c1471_dead_code_elimination() {
    let c_code = r#"
#define CB_DCE_MAX 128

typedef struct {
    int opcode;
    int dst;
    int src1;
    int src2;
    int live;
    int has_side_effect;
} cb_dce_inst_t;

typedef struct {
    cb_dce_inst_t insts[CB_DCE_MAX];
    int count;
    int used[CB_DCE_MAX];
} cb_dce_ctx_t;

void cb_dce_init(cb_dce_ctx_t *ctx) {
    int i;
    ctx->count = 0;
    for (i = 0; i < CB_DCE_MAX; i++)
        ctx->used[i] = 0;
}

void cb_dce_mark_used(cb_dce_ctx_t *ctx, int reg) {
    if (reg >= 0 && reg < CB_DCE_MAX)
        ctx->used[reg] = 1;
}

int cb_dce_sweep(cb_dce_ctx_t *ctx) {
    int i;
    int eliminated = 0;
    for (i = ctx->count - 1; i >= 0; i--) {
        if (ctx->insts[i].has_side_effect) {
            cb_dce_mark_used(ctx, ctx->insts[i].src1);
            cb_dce_mark_used(ctx, ctx->insts[i].src2);
            ctx->insts[i].live = 1;
        } else if (ctx->used[ctx->insts[i].dst]) {
            cb_dce_mark_used(ctx, ctx->insts[i].src1);
            cb_dce_mark_used(ctx, ctx->insts[i].src2);
            ctx->insts[i].live = 1;
        } else {
            ctx->insts[i].live = 0;
            eliminated++;
        }
    }
    return eliminated;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1471: Dead code elimination should transpile: {:?}", result.err());
}

/// C1472: Common subexpression elimination with value numbering
#[test]
fn c1472_common_subexpression_elimination() {
    let c_code = r#"
#define CB_CSE_MAX 64

typedef struct {
    int opcode;
    int src1;
    int src2;
    int result_reg;
} cb_cse_entry_t;

typedef struct {
    cb_cse_entry_t table[CB_CSE_MAX];
    int count;
} cb_cse_ctx_t;

void cb_cse_init(cb_cse_ctx_t *ctx) {
    ctx->count = 0;
}

int cb_cse_lookup(cb_cse_ctx_t *ctx, int op, int s1, int s2) {
    int i;
    for (i = 0; i < ctx->count; i++) {
        if (ctx->table[i].opcode == op &&
            ctx->table[i].src1 == s1 &&
            ctx->table[i].src2 == s2)
            return ctx->table[i].result_reg;
        if (ctx->table[i].opcode == op &&
            ctx->table[i].src1 == s2 &&
            ctx->table[i].src2 == s1)
            return ctx->table[i].result_reg;
    }
    return -1;
}

int cb_cse_insert(cb_cse_ctx_t *ctx, int op, int s1, int s2, int res) {
    if (ctx->count >= CB_CSE_MAX) return -1;
    ctx->table[ctx->count].opcode = op;
    ctx->table[ctx->count].src1 = s1;
    ctx->table[ctx->count].src2 = s2;
    ctx->table[ctx->count].result_reg = res;
    ctx->count++;
    return 0;
}

void cb_cse_invalidate(cb_cse_ctx_t *ctx, int reg) {
    int i;
    for (i = 0; i < ctx->count; i++) {
        if (ctx->table[i].src1 == reg || ctx->table[i].src2 == reg ||
            ctx->table[i].result_reg == reg) {
            ctx->table[i] = ctx->table[ctx->count - 1];
            ctx->count--;
            i--;
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1472: CSE should transpile: {:?}", result.err());
}

/// C1473: Loop invariant code motion
#[test]
fn c1473_loop_invariant_motion() {
    let c_code = r#"
#define CB_LICM_MAX 64

typedef struct {
    int opcode;
    int dst;
    int src1;
    int src2;
    int in_loop;
    int is_invariant;
    int hoisted;
} cb_licm_inst_t;

typedef struct {
    cb_licm_inst_t insts[CB_LICM_MAX];
    int count;
    int loop_defs[CB_LICM_MAX];
} cb_licm_ctx_t;

void cb_licm_init(cb_licm_ctx_t *ctx) {
    int i;
    ctx->count = 0;
    for (i = 0; i < CB_LICM_MAX; i++)
        ctx->loop_defs[i] = 0;
}

void cb_licm_mark_loop_defs(cb_licm_ctx_t *ctx) {
    int i;
    for (i = 0; i < ctx->count; i++) {
        if (ctx->insts[i].in_loop)
            ctx->loop_defs[ctx->insts[i].dst] = 1;
    }
}

int cb_licm_is_invariant(cb_licm_ctx_t *ctx, int idx) {
    int s1 = ctx->insts[idx].src1;
    int s2 = ctx->insts[idx].src2;
    if (!ctx->insts[idx].in_loop) return 0;
    if (s1 >= 0 && ctx->loop_defs[s1]) return 0;
    if (s2 >= 0 && ctx->loop_defs[s2]) return 0;
    return 1;
}

int cb_licm_hoist(cb_licm_ctx_t *ctx) {
    int i;
    int hoisted = 0;
    cb_licm_mark_loop_defs(ctx);
    for (i = 0; i < ctx->count; i++) {
        if (cb_licm_is_invariant(ctx, i)) {
            ctx->insts[i].hoisted = 1;
            ctx->insts[i].in_loop = 0;
            ctx->loop_defs[ctx->insts[i].dst] = 0;
            hoisted++;
        }
    }
    return hoisted;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1473: LICM should transpile: {:?}", result.err());
}

/// C1474: Inlining heuristic based on call-site cost model
#[test]
fn c1474_inlining_heuristic() {
    let c_code = r#"
#define CB_INL_MAX 64
#define CB_INL_THRESHOLD 50

typedef struct {
    int func_id;
    int inst_count;
    int call_count;
    int loop_depth;
    int has_recursion;
    int is_leaf;
} cb_func_info_t;

typedef struct {
    cb_func_info_t funcs[CB_INL_MAX];
    int count;
} cb_inline_ctx_t;

void cb_inline_init(cb_inline_ctx_t *ctx) {
    ctx->count = 0;
}

int cb_inline_cost(cb_func_info_t *f) {
    int cost = f->inst_count;
    if (f->has_recursion) cost += 1000;
    if (!f->is_leaf) cost += 10;
    return cost;
}

int cb_inline_benefit(cb_func_info_t *f, int call_loop_depth) {
    int ben = f->call_count * 5;
    ben += call_loop_depth * 20;
    if (f->is_leaf) ben += 15;
    if (f->inst_count < 10) ben += 30;
    return ben;
}

int cb_inline_should_inline(cb_func_info_t *f, int call_loop_depth) {
    int cost = cb_inline_cost(f);
    int benefit = cb_inline_benefit(f, call_loop_depth);
    if (f->has_recursion) return 0;
    return benefit > cost || f->inst_count <= 3;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1474: Inlining heuristic should transpile: {:?}", result.err());
}

/// C1475: Tail call optimization detection
#[test]
fn c1475_tail_call_optimization() {
    let c_code = r#"
#define CB_TCO_MAX 32

typedef struct {
    int func_id;
    int caller_id;
    int is_tail_pos;
    int args_match;
    int no_cleanup;
} cb_tco_call_t;

typedef struct {
    cb_tco_call_t calls[CB_TCO_MAX];
    int count;
    int optimized;
} cb_tco_ctx_t;

void cb_tco_init(cb_tco_ctx_t *ctx) {
    ctx->count = 0;
    ctx->optimized = 0;
}

int cb_tco_add_call(cb_tco_ctx_t *ctx, int func, int caller, int tail, int args_ok, int no_cl) {
    if (ctx->count >= CB_TCO_MAX) return -1;
    ctx->calls[ctx->count].func_id = func;
    ctx->calls[ctx->count].caller_id = caller;
    ctx->calls[ctx->count].is_tail_pos = tail;
    ctx->calls[ctx->count].args_match = args_ok;
    ctx->calls[ctx->count].no_cleanup = no_cl;
    return ctx->count++;
}

int cb_tco_can_optimize(cb_tco_call_t *call) {
    return call->is_tail_pos && call->no_cleanup;
}

int cb_tco_is_self_recursive(cb_tco_call_t *call) {
    return call->func_id == call->caller_id;
}

int cb_tco_optimize(cb_tco_ctx_t *ctx) {
    int i;
    int count = 0;
    for (i = 0; i < ctx->count; i++) {
        if (cb_tco_can_optimize(&ctx->calls[i])) {
            if (cb_tco_is_self_recursive(&ctx->calls[i]) || ctx->calls[i].args_match) {
                count++;
            }
        }
    }
    ctx->optimized = count;
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C1475: Tail call optimization should transpile: {:?}", result.err());
}
