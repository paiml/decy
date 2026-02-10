//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C1551-C1575: Type System Implementations -- patterns found in compilers,
//! type checkers, and language runtimes that implement type inference, type
//! checking, generic instantiation, type representation, and advanced type
//! system features.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! All C code is self-contained C99 without #include directives.
//! All functions use the `ts_` prefix.
//!
//! Organization:
//! - C1551-C1555: Type inference (HM unification, type var substitution,
//!                constraint gen, occur check, let-polymorphism)
//! - C1556-C1560: Type checking (structural subtyping, record types, variant types,
//!                function types, recursive types)
//! - C1561-C1565: Generic instantiation (monomorphization, type erasure,
//!                dictionary passing, specialization, coercion)
//! - C1566-C1570: Type representation (tagged union, vtable dispatch, type descriptor,
//!                reflection metadata, type ID hashing)
//! - C1571-C1575: Advanced types (dependent type index, linear type tracker,
//!                affine type checker, effect system, row polymorphism)

// ============================================================================
// C1551-C1555: Type Inference
// ============================================================================

/// C1551: Hindley-Milner unification with union-find for type variables
#[test]
fn c1551_hm_unification() {
    let c_code = r#"
enum ts_type_kind {
    TS_TVAR = 0,
    TS_TINT = 1,
    TS_TBOOL = 2,
    TS_TARROW = 3,
    TS_TPAIR = 4
};

struct ts_type_node {
    enum ts_type_kind kind;
    int var_id;
    int left;
    int right;
    int parent;
};

struct ts_unify_ctx {
    struct ts_type_node nodes[128];
    int count;
};

void ts_unify_init(struct ts_unify_ctx *ctx) {
    ctx->count = 0;
}

int ts_unify_new_var(struct ts_unify_ctx *ctx, int var_id) {
    if (ctx->count >= 128) return -1;
    int idx = ctx->count;
    ctx->nodes[idx].kind = TS_TVAR;
    ctx->nodes[idx].var_id = var_id;
    ctx->nodes[idx].left = -1;
    ctx->nodes[idx].right = -1;
    ctx->nodes[idx].parent = idx;
    ctx->count++;
    return idx;
}

int ts_unify_new_concrete(struct ts_unify_ctx *ctx, enum ts_type_kind kind) {
    if (ctx->count >= 128) return -1;
    int idx = ctx->count;
    ctx->nodes[idx].kind = kind;
    ctx->nodes[idx].var_id = -1;
    ctx->nodes[idx].left = -1;
    ctx->nodes[idx].right = -1;
    ctx->nodes[idx].parent = idx;
    ctx->count++;
    return idx;
}

int ts_unify_find(struct ts_unify_ctx *ctx, int idx) {
    if (idx < 0 || idx >= ctx->count) return -1;
    while (ctx->nodes[idx].parent != idx) {
        ctx->nodes[idx].parent = ctx->nodes[ctx->nodes[idx].parent].parent;
        idx = ctx->nodes[idx].parent;
    }
    return idx;
}

int ts_unify_union(struct ts_unify_ctx *ctx, int a, int b) {
    int ra = ts_unify_find(ctx, a);
    int rb = ts_unify_find(ctx, b);
    if (ra < 0 || rb < 0) return -1;
    if (ra == rb) return 0;
    if (ctx->nodes[ra].kind == TS_TVAR) {
        ctx->nodes[ra].parent = rb;
        return 0;
    }
    if (ctx->nodes[rb].kind == TS_TVAR) {
        ctx->nodes[rb].parent = ra;
        return 0;
    }
    if (ctx->nodes[ra].kind != ctx->nodes[rb].kind) return -1;
    ctx->nodes[rb].parent = ra;
    return 0;
}

int ts_unify_selftest(void) {
    struct ts_unify_ctx ctx;
    ts_unify_init(&ctx);
    int v0 = ts_unify_new_var(&ctx, 0);
    int t_int = ts_unify_new_concrete(&ctx, TS_TINT);
    if (ts_unify_union(&ctx, v0, t_int) != 0) return -1;
    int r = ts_unify_find(&ctx, v0);
    if (ctx.nodes[r].kind != TS_TINT) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1551: HM unification should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1551: Output should not be empty");
    assert!(
        code.contains("fn ts_unify_find"),
        "C1551: Should contain ts_unify_find function"
    );
}

/// C1552: Type variable substitution with walk-through and compose
#[test]
fn c1552_type_var_substitution() {
    let c_code = r#"
struct ts_subst_entry {
    int from_var;
    int to_type;
};

struct ts_subst {
    struct ts_subst_entry mappings[64];
    int count;
};

void ts_subst_init(struct ts_subst *s) {
    s->count = 0;
}

int ts_subst_add(struct ts_subst *s, int from, int to) {
    if (s->count >= 64) return -1;
    int i;
    for (i = 0; i < s->count; i++) {
        if (s->mappings[i].from_var == from) {
            s->mappings[i].to_type = to;
            return 0;
        }
    }
    int idx = s->count;
    s->mappings[idx].from_var = from;
    s->mappings[idx].to_type = to;
    s->count++;
    return 0;
}

int ts_subst_walk(struct ts_subst *s, int type_id) {
    int depth = 0;
    while (depth < 64) {
        int found = 0;
        int i;
        for (i = 0; i < s->count; i++) {
            if (s->mappings[i].from_var == type_id) {
                type_id = s->mappings[i].to_type;
                found = 1;
                break;
            }
        }
        if (!found) break;
        depth++;
    }
    return type_id;
}

int ts_subst_compose(struct ts_subst *dst, struct ts_subst *s1, struct ts_subst *s2) {
    int i;
    ts_subst_init(dst);
    for (i = 0; i < s1->count; i++) {
        int resolved = ts_subst_walk(s2, s1->mappings[i].to_type);
        ts_subst_add(dst, s1->mappings[i].from_var, resolved);
    }
    for (i = 0; i < s2->count; i++) {
        int already = 0;
        int j;
        for (j = 0; j < dst->count; j++) {
            if (dst->mappings[j].from_var == s2->mappings[i].from_var) {
                already = 1;
                break;
            }
        }
        if (!already) {
            ts_subst_add(dst, s2->mappings[i].from_var, s2->mappings[i].to_type);
        }
    }
    return 0;
}

int ts_subst_selftest(void) {
    struct ts_subst s;
    ts_subst_init(&s);
    ts_subst_add(&s, 0, 10);
    ts_subst_add(&s, 10, 20);
    if (ts_subst_walk(&s, 0) != 20) return -1;
    if (ts_subst_walk(&s, 99) != 99) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1552: Type var substitution should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1552: Output should not be empty");
    assert!(
        code.contains("fn ts_subst_walk"),
        "C1552: Should contain ts_subst_walk function"
    );
}

/// C1553: Constraint generation from expression types
#[test]
fn c1553_constraint_generation() {
    let c_code = r#"
enum ts_con_kind {
    TS_CON_EQ = 0,
    TS_CON_SUB = 1,
    TS_CON_INST = 2
};

struct ts_constraint {
    enum ts_con_kind kind;
    int left_type;
    int right_type;
    int source_line;
};

struct ts_con_set {
    struct ts_constraint items[256];
    int count;
    int next_var;
};

void ts_con_set_init(struct ts_con_set *cs) {
    cs->count = 0;
    cs->next_var = 1000;
}

int ts_con_fresh_var(struct ts_con_set *cs) {
    return cs->next_var++;
}

int ts_con_add(struct ts_con_set *cs, enum ts_con_kind kind, int left, int right, int line) {
    if (cs->count >= 256) return -1;
    int idx = cs->count;
    cs->items[idx].kind = kind;
    cs->items[idx].left_type = left;
    cs->items[idx].right_type = right;
    cs->items[idx].source_line = line;
    cs->count++;
    return 0;
}

int ts_con_has_eq_conflict(struct ts_con_set *cs) {
    int i, j;
    for (i = 0; i < cs->count; i++) {
        if (cs->items[i].kind != TS_CON_EQ) continue;
        for (j = i + 1; j < cs->count; j++) {
            if (cs->items[j].kind != TS_CON_EQ) continue;
            if (cs->items[i].left_type == cs->items[j].left_type &&
                cs->items[i].right_type != cs->items[j].right_type) {
                return 1;
            }
        }
    }
    return 0;
}

int ts_con_selftest(void) {
    struct ts_con_set cs;
    ts_con_set_init(&cs);
    int v = ts_con_fresh_var(&cs);
    ts_con_add(&cs, TS_CON_EQ, v, 1, 10);
    if (ts_con_has_eq_conflict(&cs)) return -1;
    ts_con_add(&cs, TS_CON_EQ, v, 2, 20);
    if (!ts_con_has_eq_conflict(&cs)) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1553: Constraint generation should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1553: Output should not be empty");
    assert!(
        code.contains("fn ts_con_add"),
        "C1553: Should contain ts_con_add function"
    );
}

/// C1554: Occurs check for infinite type detection
#[test]
fn c1554_occur_check() {
    let c_code = r#"
struct ts_occ_node {
    int kind;
    int var_id;
    int children[4];
    int child_count;
};

struct ts_occ_graph {
    struct ts_occ_node nodes[64];
    int count;
};

void ts_occ_graph_init(struct ts_occ_graph *g) {
    g->count = 0;
}

int ts_occ_add_var(struct ts_occ_graph *g, int var_id) {
    if (g->count >= 64) return -1;
    int idx = g->count;
    g->nodes[idx].kind = 0;
    g->nodes[idx].var_id = var_id;
    g->nodes[idx].child_count = 0;
    g->count++;
    return idx;
}

int ts_occ_add_constructor(struct ts_occ_graph *g, int kind) {
    if (g->count >= 64) return -1;
    int idx = g->count;
    g->nodes[idx].kind = kind;
    g->nodes[idx].var_id = -1;
    g->nodes[idx].child_count = 0;
    g->count++;
    return idx;
}

int ts_occ_add_child(struct ts_occ_graph *g, int parent, int child) {
    if (parent < 0 || parent >= g->count) return -1;
    if (g->nodes[parent].child_count >= 4) return -1;
    g->nodes[parent].children[g->nodes[parent].child_count] = child;
    g->nodes[parent].child_count++;
    return 0;
}

int ts_occ_check(struct ts_occ_graph *g, int var_id, int node_idx) {
    if (node_idx < 0 || node_idx >= g->count) return 0;
    struct ts_occ_node *n = &g->nodes[node_idx];
    if (n->kind == 0 && n->var_id == var_id) return 1;
    int i;
    for (i = 0; i < n->child_count; i++) {
        if (ts_occ_check(g, var_id, n->children[i])) return 1;
    }
    return 0;
}

int ts_occ_selftest(void) {
    struct ts_occ_graph g;
    ts_occ_graph_init(&g);
    int v0 = ts_occ_add_var(&g, 5);
    int arr = ts_occ_add_constructor(&g, 10);
    ts_occ_add_child(&g, arr, v0);
    if (!ts_occ_check(&g, 5, arr)) return -1;
    if (ts_occ_check(&g, 99, arr)) return -2;
    if (!ts_occ_check(&g, 5, v0)) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1554: Occur check should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1554: Output should not be empty");
    assert!(
        code.contains("fn ts_occ_check"),
        "C1554: Should contain ts_occ_check function"
    );
}

/// C1555: Let-polymorphism with generalization and instantiation
#[test]
fn c1555_let_polymorphism() {
    let c_code = r#"
struct ts_poly_var {
    int id;
    int level;
    int is_generic;
};

struct ts_poly_scheme {
    int quantified[8];
    int num_quantified;
    int body_type;
};

struct ts_poly_ctx {
    struct ts_poly_var vars[64];
    int var_count;
    int current_level;
    int next_fresh;
};

void ts_poly_ctx_init(struct ts_poly_ctx *ctx) {
    ctx->var_count = 0;
    ctx->current_level = 0;
    ctx->next_fresh = 100;
}

int ts_poly_add_var(struct ts_poly_ctx *ctx, int id, int level) {
    if (ctx->var_count >= 64) return -1;
    int idx = ctx->var_count;
    ctx->vars[idx].id = id;
    ctx->vars[idx].level = level;
    ctx->vars[idx].is_generic = 0;
    ctx->var_count++;
    return idx;
}

void ts_poly_enter_let(struct ts_poly_ctx *ctx) {
    ctx->current_level++;
}

void ts_poly_leave_let(struct ts_poly_ctx *ctx) {
    if (ctx->current_level > 0) ctx->current_level--;
}

int ts_poly_generalize(struct ts_poly_ctx *ctx, struct ts_poly_scheme *scheme, int body) {
    scheme->body_type = body;
    scheme->num_quantified = 0;
    int i;
    for (i = 0; i < ctx->var_count; i++) {
        if (ctx->vars[i].level > ctx->current_level) {
            ctx->vars[i].is_generic = 1;
            if (scheme->num_quantified < 8) {
                scheme->quantified[scheme->num_quantified] = ctx->vars[i].id;
                scheme->num_quantified++;
            }
        }
    }
    return scheme->num_quantified;
}

int ts_poly_instantiate(struct ts_poly_ctx *ctx, struct ts_poly_scheme *scheme) {
    int i;
    for (i = 0; i < scheme->num_quantified; i++) {
        ctx->next_fresh++;
    }
    return scheme->body_type;
}

int ts_poly_selftest(void) {
    struct ts_poly_ctx ctx;
    struct ts_poly_scheme scheme;
    ts_poly_ctx_init(&ctx);
    ts_poly_enter_let(&ctx);
    ts_poly_add_var(&ctx, 0, 1);
    ts_poly_add_var(&ctx, 1, 1);
    ts_poly_add_var(&ctx, 2, 0);
    ts_poly_leave_let(&ctx);
    int count = ts_poly_generalize(&ctx, &scheme, 42);
    if (count != 2) return -1;
    if (scheme.body_type != 42) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1555: Let-polymorphism should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1555: Output should not be empty");
    assert!(
        code.contains("fn ts_poly_generalize"),
        "C1555: Should contain ts_poly_generalize function"
    );
}

// ============================================================================
// C1556-C1560: Type Checking
// ============================================================================

/// C1556: Structural subtyping with width and depth checks
#[test]
fn c1556_structural_subtyping() {
    let c_code = r#"
struct ts_field_desc {
    int name_hash;
    int type_id;
};

struct ts_struct_type {
    struct ts_field_desc fields[16];
    int field_count;
};

void ts_struct_init(struct ts_struct_type *st) {
    st->field_count = 0;
}

int ts_struct_add_field(struct ts_struct_type *st, int name_hash, int type_id) {
    if (st->field_count >= 16) return -1;
    int idx = st->field_count;
    st->fields[idx].name_hash = name_hash;
    st->fields[idx].type_id = type_id;
    st->field_count++;
    return 0;
}

int ts_struct_has_field(struct ts_struct_type *st, int name_hash) {
    int i;
    for (i = 0; i < st->field_count; i++) {
        if (st->fields[i].name_hash == name_hash) return i;
    }
    return -1;
}

int ts_struct_is_subtype(struct ts_struct_type *sub, struct ts_struct_type *sup) {
    int i;
    for (i = 0; i < sup->field_count; i++) {
        int idx = ts_struct_has_field(sub, sup->fields[i].name_hash);
        if (idx < 0) return 0;
        if (sub->fields[idx].type_id != sup->fields[i].type_id) return 0;
    }
    return 1;
}

int ts_struct_sub_selftest(void) {
    struct ts_struct_type wide, narrow;
    ts_struct_init(&wide);
    ts_struct_init(&narrow);
    ts_struct_add_field(&wide, 100, 1);
    ts_struct_add_field(&wide, 200, 2);
    ts_struct_add_field(&wide, 300, 3);
    ts_struct_add_field(&narrow, 100, 1);
    ts_struct_add_field(&narrow, 200, 2);
    if (!ts_struct_is_subtype(&wide, &narrow)) return -1;
    if (ts_struct_is_subtype(&narrow, &wide)) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1556: Structural subtyping should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1556: Output should not be empty");
    assert!(
        code.contains("fn ts_struct_is_subtype"),
        "C1556: Should contain ts_struct_is_subtype function"
    );
}

/// C1557: Record type operations with field projection and extension
#[test]
fn c1557_record_types() {
    let c_code = r#"
struct ts_rec_field {
    int name_hash;
    int type_id;
    int mutable_flag;
};

struct ts_record {
    struct ts_rec_field fields[16];
    int count;
    int sealed;
};

void ts_rec_init(struct ts_record *r) {
    r->count = 0;
    r->sealed = 0;
}

int ts_rec_add_field(struct ts_record *r, int name_hash, int type_id, int mutable_flag) {
    if (r->sealed || r->count >= 16) return -1;
    int i;
    for (i = 0; i < r->count; i++) {
        if (r->fields[i].name_hash == name_hash) return -2;
    }
    int idx = r->count;
    r->fields[idx].name_hash = name_hash;
    r->fields[idx].type_id = type_id;
    r->fields[idx].mutable_flag = mutable_flag;
    r->count++;
    return 0;
}

int ts_rec_project(struct ts_record *r, int name_hash) {
    int i;
    for (i = 0; i < r->count; i++) {
        if (r->fields[i].name_hash == name_hash) return r->fields[i].type_id;
    }
    return -1;
}

int ts_rec_extend(struct ts_record *dst, struct ts_record *base, int name_hash, int type_id) {
    int i;
    ts_rec_init(dst);
    for (i = 0; i < base->count; i++) {
        ts_rec_add_field(dst, base->fields[i].name_hash,
                         base->fields[i].type_id, base->fields[i].mutable_flag);
    }
    return ts_rec_add_field(dst, name_hash, type_id, 0);
}

int ts_rec_selftest(void) {
    struct ts_record r;
    ts_rec_init(&r);
    ts_rec_add_field(&r, 10, 1, 0);
    ts_rec_add_field(&r, 20, 2, 1);
    if (ts_rec_project(&r, 10) != 1) return -1;
    if (ts_rec_project(&r, 99) != -1) return -2;
    struct ts_record ext;
    ts_rec_extend(&ext, &r, 30, 3);
    if (ext.count != 3) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1557: Record types should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1557: Output should not be empty");
    assert!(
        code.contains("fn ts_rec_project"),
        "C1557: Should contain ts_rec_project function"
    );
}

/// C1558: Variant (sum) types with case analysis and exhaustiveness
#[test]
fn c1558_variant_types() {
    let c_code = r#"
struct ts_variant_case {
    int tag;
    int payload_type;
    int covered;
};

struct ts_variant {
    struct ts_variant_case cases[16];
    int case_count;
};

void ts_variant_init(struct ts_variant *v) {
    v->case_count = 0;
}

int ts_variant_add_case(struct ts_variant *v, int tag, int payload_type) {
    if (v->case_count >= 16) return -1;
    int idx = v->case_count;
    v->cases[idx].tag = tag;
    v->cases[idx].payload_type = payload_type;
    v->cases[idx].covered = 0;
    v->case_count++;
    return 0;
}

int ts_variant_tag_type(struct ts_variant *v, int tag) {
    int i;
    for (i = 0; i < v->case_count; i++) {
        if (v->cases[i].tag == tag) return v->cases[i].payload_type;
    }
    return -1;
}

int ts_variant_mark_covered(struct ts_variant *v, int tag) {
    int i;
    for (i = 0; i < v->case_count; i++) {
        if (v->cases[i].tag == tag) {
            v->cases[i].covered = 1;
            return 0;
        }
    }
    return -1;
}

int ts_variant_is_exhaustive(struct ts_variant *v) {
    int i;
    for (i = 0; i < v->case_count; i++) {
        if (!v->cases[i].covered) return 0;
    }
    return 1;
}

int ts_variant_selftest(void) {
    struct ts_variant v;
    ts_variant_init(&v);
    ts_variant_add_case(&v, 0, 10);
    ts_variant_add_case(&v, 1, 20);
    ts_variant_add_case(&v, 2, 30);
    if (ts_variant_tag_type(&v, 1) != 20) return -1;
    ts_variant_mark_covered(&v, 0);
    ts_variant_mark_covered(&v, 1);
    if (ts_variant_is_exhaustive(&v)) return -2;
    ts_variant_mark_covered(&v, 2);
    if (!ts_variant_is_exhaustive(&v)) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1558: Variant types should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1558: Output should not be empty");
    assert!(
        code.contains("fn ts_variant_is_exhaustive"),
        "C1558: Should contain ts_variant_is_exhaustive function"
    );
}

/// C1559: Function type checking with arity and return type compatibility
#[test]
fn c1559_function_types() {
    let c_code = r#"
struct ts_fn_param {
    int type_id;
    int is_const;
};

struct ts_fn_type {
    struct ts_fn_param params[8];
    int param_count;
    int return_type;
    int is_variadic;
};

void ts_fn_init(struct ts_fn_type *ft, int return_type) {
    ft->param_count = 0;
    ft->return_type = return_type;
    ft->is_variadic = 0;
}

int ts_fn_add_param(struct ts_fn_type *ft, int type_id, int is_const) {
    if (ft->param_count >= 8) return -1;
    int idx = ft->param_count;
    ft->params[idx].type_id = type_id;
    ft->params[idx].is_const = is_const;
    ft->param_count++;
    return 0;
}

int ts_fn_compatible(struct ts_fn_type *expected, struct ts_fn_type *actual) {
    if (expected->return_type != actual->return_type) return 0;
    if (expected->param_count != actual->param_count) {
        if (!expected->is_variadic) return 0;
        if (actual->param_count < expected->param_count) return 0;
    }
    int i;
    int check_count = expected->param_count;
    for (i = 0; i < check_count; i++) {
        if (expected->params[i].type_id != actual->params[i].type_id) return 0;
    }
    return 1;
}

int ts_fn_arity_check(struct ts_fn_type *ft, int arg_count) {
    if (ft->is_variadic) return arg_count >= ft->param_count;
    return arg_count == ft->param_count;
}

int ts_fn_selftest(void) {
    struct ts_fn_type f1, f2;
    ts_fn_init(&f1, 1);
    ts_fn_init(&f2, 1);
    ts_fn_add_param(&f1, 10, 0);
    ts_fn_add_param(&f1, 20, 1);
    ts_fn_add_param(&f2, 10, 0);
    ts_fn_add_param(&f2, 20, 1);
    if (!ts_fn_compatible(&f1, &f2)) return -1;
    ts_fn_add_param(&f2, 30, 0);
    if (ts_fn_compatible(&f1, &f2)) return -2;
    if (!ts_fn_arity_check(&f1, 2)) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1559: Function types should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1559: Output should not be empty");
    assert!(
        code.contains("fn ts_fn_compatible"),
        "C1559: Should contain ts_fn_compatible function"
    );
}

/// C1560: Recursive types with mu-types and contractiveness check
#[test]
fn c1560_recursive_types() {
    let c_code = r#"
enum ts_rec_kind {
    TS_RK_BASE = 0,
    TS_RK_MU = 1,
    TS_RK_VAR = 2,
    TS_RK_APP = 3
};

struct ts_rec_type {
    enum ts_rec_kind kind;
    int mu_var;
    int body;
    int arg;
};

struct ts_rec_env {
    struct ts_rec_type types[64];
    int count;
    int unfolding_depth;
};

void ts_rec_env_init(struct ts_rec_env *env) {
    env->count = 0;
    env->unfolding_depth = 0;
}

int ts_rec_add_base(struct ts_rec_env *env) {
    if (env->count >= 64) return -1;
    int idx = env->count;
    env->types[idx].kind = TS_RK_BASE;
    env->types[idx].mu_var = -1;
    env->types[idx].body = -1;
    env->types[idx].arg = -1;
    env->count++;
    return idx;
}

int ts_rec_add_mu(struct ts_rec_env *env, int mu_var, int body) {
    if (env->count >= 64) return -1;
    int idx = env->count;
    env->types[idx].kind = TS_RK_MU;
    env->types[idx].mu_var = mu_var;
    env->types[idx].body = body;
    env->types[idx].arg = -1;
    env->count++;
    return idx;
}

int ts_rec_unfold(struct ts_rec_env *env, int type_idx) {
    if (type_idx < 0 || type_idx >= env->count) return -1;
    if (env->types[type_idx].kind != TS_RK_MU) return type_idx;
    if (env->unfolding_depth > 10) return -1;
    env->unfolding_depth++;
    int result = env->types[type_idx].body;
    env->unfolding_depth--;
    return result;
}

int ts_rec_is_contractive(struct ts_rec_env *env, int type_idx) {
    if (type_idx < 0 || type_idx >= env->count) return 0;
    if (env->types[type_idx].kind == TS_RK_BASE) return 1;
    if (env->types[type_idx].kind == TS_RK_MU) {
        int body = env->types[type_idx].body;
        if (body >= 0 && body < env->count) {
            return env->types[body].kind != TS_RK_VAR;
        }
    }
    return 1;
}

int ts_rec_selftest(void) {
    struct ts_rec_env env;
    ts_rec_env_init(&env);
    int base = ts_rec_add_base(&env);
    int mu = ts_rec_add_mu(&env, 0, base);
    int unfolded = ts_rec_unfold(&env, mu);
    if (unfolded != base) return -1;
    if (!ts_rec_is_contractive(&env, mu)) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1560: Recursive types should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1560: Output should not be empty");
    assert!(
        code.contains("fn ts_rec_unfold"),
        "C1560: Should contain ts_rec_unfold function"
    );
}

// ============================================================================
// C1561-C1565: Generic Instantiation
// ============================================================================

/// C1561: Monomorphization cache with type argument keying
#[test]
fn c1561_monomorphization() {
    let c_code = r#"
struct ts_mono_key {
    int generic_id;
    int type_args[4];
    int num_args;
};

struct ts_mono_entry {
    struct ts_mono_key key;
    int specialized_id;
};

struct ts_mono_cache {
    struct ts_mono_entry entries[128];
    int count;
};

void ts_mono_cache_init(struct ts_mono_cache *mc) {
    mc->count = 0;
}

int ts_mono_key_eq(struct ts_mono_key *a, struct ts_mono_key *b) {
    if (a->generic_id != b->generic_id) return 0;
    if (a->num_args != b->num_args) return 0;
    int i;
    for (i = 0; i < a->num_args; i++) {
        if (a->type_args[i] != b->type_args[i]) return 0;
    }
    return 1;
}

int ts_mono_lookup(struct ts_mono_cache *mc, struct ts_mono_key *key) {
    int i;
    for (i = 0; i < mc->count; i++) {
        if (ts_mono_key_eq(&mc->entries[i].key, key)) {
            return mc->entries[i].specialized_id;
        }
    }
    return -1;
}

int ts_mono_insert(struct ts_mono_cache *mc, struct ts_mono_key *key, int spec_id) {
    if (mc->count >= 128) return -1;
    int idx = mc->count;
    mc->entries[idx].key = *key;
    mc->entries[idx].specialized_id = spec_id;
    mc->count++;
    return 0;
}

int ts_mono_selftest(void) {
    struct ts_mono_cache mc;
    ts_mono_cache_init(&mc);
    struct ts_mono_key k1;
    k1.generic_id = 1;
    k1.type_args[0] = 10;
    k1.type_args[1] = 20;
    k1.num_args = 2;
    ts_mono_insert(&mc, &k1, 100);
    if (ts_mono_lookup(&mc, &k1) != 100) return -1;
    struct ts_mono_key k2;
    k2.generic_id = 1;
    k2.type_args[0] = 10;
    k2.type_args[1] = 30;
    k2.num_args = 2;
    if (ts_mono_lookup(&mc, &k2) != -1) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1561: Monomorphization should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1561: Output should not be empty");
    assert!(
        code.contains("fn ts_mono_lookup"),
        "C1561: Should contain ts_mono_lookup function"
    );
}

/// C1562: Type erasure with runtime tag preservation
#[test]
fn c1562_type_erasure() {
    let c_code = r#"
enum ts_erased_tag {
    TS_ERASE_INT = 0,
    TS_ERASE_FLOAT = 1,
    TS_ERASE_PTR = 2,
    TS_ERASE_STRUCT = 3,
    TS_ERASE_VOID = 4
};

struct ts_erased_type {
    enum ts_erased_tag tag;
    int size;
    int align;
    int original_id;
};

struct ts_erasure_map {
    struct ts_erased_type entries[64];
    int count;
};

void ts_erasure_init(struct ts_erasure_map *em) {
    em->count = 0;
}

int ts_erasure_erase(struct ts_erasure_map *em, int original_id, enum ts_erased_tag tag, int size, int align) {
    if (em->count >= 64) return -1;
    int idx = em->count;
    em->entries[idx].tag = tag;
    em->entries[idx].size = size;
    em->entries[idx].align = align;
    em->entries[idx].original_id = original_id;
    em->count++;
    return idx;
}

enum ts_erased_tag ts_erasure_get_tag(struct ts_erasure_map *em, int idx) {
    if (idx < 0 || idx >= em->count) return TS_ERASE_VOID;
    return em->entries[idx].tag;
}

int ts_erasure_same_layout(struct ts_erasure_map *em, int a, int b) {
    if (a < 0 || a >= em->count || b < 0 || b >= em->count) return 0;
    return em->entries[a].size == em->entries[b].size &&
           em->entries[a].align == em->entries[b].align;
}

int ts_erasure_selftest(void) {
    struct ts_erasure_map em;
    ts_erasure_init(&em);
    int i = ts_erasure_erase(&em, 100, TS_ERASE_INT, 4, 4);
    int f = ts_erasure_erase(&em, 200, TS_ERASE_FLOAT, 4, 4);
    if (ts_erasure_get_tag(&em, i) != TS_ERASE_INT) return -1;
    if (!ts_erasure_same_layout(&em, i, f)) return -2;
    int p = ts_erasure_erase(&em, 300, TS_ERASE_PTR, 8, 8);
    if (ts_erasure_same_layout(&em, i, p)) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1562: Type erasure should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1562: Output should not be empty");
    assert!(
        code.contains("fn ts_erasure_erase"),
        "C1562: Should contain ts_erasure_erase function"
    );
}

/// C1563: Dictionary passing for typeclass method dispatch
#[test]
fn c1563_dictionary_passing() {
    let c_code = r#"
struct ts_dict_method {
    int method_id;
    int impl_fn_id;
};

struct ts_dict {
    int typeclass_id;
    int impl_type;
    struct ts_dict_method methods[8];
    int method_count;
};

struct ts_dict_table {
    struct ts_dict dicts[64];
    int count;
};

void ts_dict_table_init(struct ts_dict_table *dt) {
    dt->count = 0;
}

int ts_dict_register(struct ts_dict_table *dt, int typeclass_id, int impl_type) {
    if (dt->count >= 64) return -1;
    int idx = dt->count;
    dt->dicts[idx].typeclass_id = typeclass_id;
    dt->dicts[idx].impl_type = impl_type;
    dt->dicts[idx].method_count = 0;
    dt->count++;
    return idx;
}

int ts_dict_add_method(struct ts_dict_table *dt, int dict_idx, int method_id, int impl_fn) {
    if (dict_idx < 0 || dict_idx >= dt->count) return -1;
    struct ts_dict *d = &dt->dicts[dict_idx];
    if (d->method_count >= 8) return -1;
    int idx = d->method_count;
    d->methods[idx].method_id = method_id;
    d->methods[idx].impl_fn_id = impl_fn;
    d->method_count++;
    return 0;
}

int ts_dict_dispatch(struct ts_dict_table *dt, int typeclass_id, int impl_type, int method_id) {
    int i, j;
    for (i = 0; i < dt->count; i++) {
        if (dt->dicts[i].typeclass_id == typeclass_id &&
            dt->dicts[i].impl_type == impl_type) {
            for (j = 0; j < dt->dicts[i].method_count; j++) {
                if (dt->dicts[i].methods[j].method_id == method_id) {
                    return dt->dicts[i].methods[j].impl_fn_id;
                }
            }
        }
    }
    return -1;
}

int ts_dict_selftest(void) {
    struct ts_dict_table dt;
    ts_dict_table_init(&dt);
    int d = ts_dict_register(&dt, 1, 10);
    ts_dict_add_method(&dt, d, 100, 500);
    ts_dict_add_method(&dt, d, 200, 600);
    if (ts_dict_dispatch(&dt, 1, 10, 100) != 500) return -1;
    if (ts_dict_dispatch(&dt, 1, 10, 200) != 600) return -2;
    if (ts_dict_dispatch(&dt, 1, 10, 999) != -1) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1563: Dictionary passing should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1563: Output should not be empty");
    assert!(
        code.contains("fn ts_dict_dispatch"),
        "C1563: Should contain ts_dict_dispatch function"
    );
}

/// C1564: Type specialization with priority-ordered rules and default fallback
#[test]
fn c1564_specialization() {
    let c_code = r#"
struct ts_spec_rule {
    int generic_id;
    int pattern_type;
    int impl_id;
    int priority;
    int is_default;
};

struct ts_spec_table {
    struct ts_spec_rule rules[64];
    int count;
};

void ts_spec_init(struct ts_spec_table *st) {
    st->count = 0;
}

int ts_spec_add_rule(struct ts_spec_table *st, int gen_id, int pat, int impl_id, int prio, int is_default) {
    if (st->count >= 64) return -1;
    int idx = st->count;
    st->rules[idx].generic_id = gen_id;
    st->rules[idx].pattern_type = pat;
    st->rules[idx].impl_id = impl_id;
    st->rules[idx].priority = prio;
    st->rules[idx].is_default = is_default;
    st->count++;
    return 0;
}

int ts_spec_resolve(struct ts_spec_table *st, int gen_id, int concrete) {
    int best = -1;
    int best_prio = -1;
    int default_impl = -1;
    int i;
    for (i = 0; i < st->count; i++) {
        if (st->rules[i].generic_id != gen_id) continue;
        if (st->rules[i].is_default) {
            default_impl = st->rules[i].impl_id;
            continue;
        }
        if (st->rules[i].pattern_type == concrete &&
            st->rules[i].priority > best_prio) {
            best = st->rules[i].impl_id;
            best_prio = st->rules[i].priority;
        }
    }
    return best >= 0 ? best : default_impl;
}

int ts_spec_selftest(void) {
    struct ts_spec_table st;
    ts_spec_init(&st);
    ts_spec_add_rule(&st, 1, -1, 999, 0, 1);
    ts_spec_add_rule(&st, 1, 10, 100, 5, 0);
    ts_spec_add_rule(&st, 1, 10, 200, 10, 0);
    if (ts_spec_resolve(&st, 1, 10) != 200) return -1;
    if (ts_spec_resolve(&st, 1, 99) != 999) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1564: Specialization should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1564: Output should not be empty");
    assert!(
        code.contains("fn ts_spec_resolve"),
        "C1564: Should contain ts_spec_resolve function"
    );
}

/// C1565: Implicit type coercion with cost-based selection
#[test]
fn c1565_coercion() {
    let c_code = r#"
struct ts_coercion {
    int from_type;
    int to_type;
    int cost;
    int lossy;
};

struct ts_coercion_table {
    struct ts_coercion rules[64];
    int count;
};

void ts_coercion_init(struct ts_coercion_table *ct) {
    ct->count = 0;
}

int ts_coercion_add(struct ts_coercion_table *ct, int from, int to, int cost, int lossy) {
    if (ct->count >= 64) return -1;
    int idx = ct->count;
    ct->rules[idx].from_type = from;
    ct->rules[idx].to_type = to;
    ct->rules[idx].cost = cost;
    ct->rules[idx].lossy = lossy;
    ct->count++;
    return 0;
}

int ts_coercion_find(struct ts_coercion_table *ct, int from, int to) {
    int best_cost = 9999;
    int best_idx = -1;
    int i;
    for (i = 0; i < ct->count; i++) {
        if (ct->rules[i].from_type == from && ct->rules[i].to_type == to) {
            if (ct->rules[i].cost < best_cost) {
                best_cost = ct->rules[i].cost;
                best_idx = i;
            }
        }
    }
    return best_idx;
}

int ts_coercion_is_safe(struct ts_coercion_table *ct, int from, int to) {
    int idx = ts_coercion_find(ct, from, to);
    if (idx < 0) return 0;
    return !ct->rules[idx].lossy;
}

int ts_coercion_cost(struct ts_coercion_table *ct, int from, int to) {
    int idx = ts_coercion_find(ct, from, to);
    if (idx < 0) return -1;
    return ct->rules[idx].cost;
}

int ts_coercion_selftest(void) {
    struct ts_coercion_table ct;
    ts_coercion_init(&ct);
    ts_coercion_add(&ct, 1, 2, 1, 0);
    ts_coercion_add(&ct, 2, 3, 2, 1);
    ts_coercion_add(&ct, 1, 3, 5, 0);
    if (!ts_coercion_is_safe(&ct, 1, 2)) return -1;
    if (ts_coercion_is_safe(&ct, 2, 3)) return -2;
    if (ts_coercion_cost(&ct, 1, 2) != 1) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1565: Coercion should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1565: Output should not be empty");
    assert!(
        code.contains("fn ts_coercion_find"),
        "C1565: Should contain ts_coercion_find function"
    );
}

// ============================================================================
// C1566-C1570: Type Representation
// ============================================================================

/// C1566: Tagged union runtime representation with discriminant and accessors
#[test]
fn c1566_tagged_union() {
    let c_code = r#"
enum ts_tag_kind {
    TS_TAG_INT = 0,
    TS_TAG_FLOAT = 1,
    TS_TAG_STRING = 2,
    TS_TAG_BOOL = 3,
    TS_TAG_NIL = 4
};

struct ts_tagged_value {
    enum ts_tag_kind tag;
    int int_val;
    int extra;
};

struct ts_tagged_value ts_tagged_int(int v) {
    struct ts_tagged_value tv;
    tv.tag = TS_TAG_INT;
    tv.int_val = v;
    tv.extra = 0;
    return tv;
}

struct ts_tagged_value ts_tagged_nil(void) {
    struct ts_tagged_value tv;
    tv.tag = TS_TAG_NIL;
    tv.int_val = 0;
    tv.extra = 0;
    return tv;
}

int ts_tagged_is_truthy(struct ts_tagged_value v) {
    switch (v.tag) {
        case TS_TAG_NIL: return 0;
        case TS_TAG_BOOL: return v.int_val != 0;
        case TS_TAG_INT: return v.int_val != 0;
        default: return 1;
    }
}

int ts_tagged_eq(struct ts_tagged_value a, struct ts_tagged_value b) {
    if (a.tag != b.tag) return 0;
    if (a.tag == TS_TAG_NIL) return 1;
    return a.int_val == b.int_val;
}

int ts_tagged_selftest(void) {
    struct ts_tagged_value i = ts_tagged_int(42);
    struct ts_tagged_value n = ts_tagged_nil();
    if (!ts_tagged_is_truthy(i)) return -1;
    if (ts_tagged_is_truthy(n)) return -2;
    if (ts_tagged_eq(i, n)) return -3;
    struct ts_tagged_value i2 = ts_tagged_int(42);
    if (!ts_tagged_eq(i, i2)) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1566: Tagged union should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1566: Output should not be empty");
    assert!(
        code.contains("fn ts_tagged_is_truthy"),
        "C1566: Should contain ts_tagged_is_truthy function"
    );
}

/// C1567: Vtable dispatch for dynamic method resolution
#[test]
fn c1567_vtable_dispatch() {
    let c_code = r#"
struct ts_vtable_entry {
    int method_id;
    int fn_ptr_id;
};

struct ts_vtable {
    struct ts_vtable_entry entries[16];
    int count;
    int type_id;
};

struct ts_vtable_registry {
    struct ts_vtable vtables[32];
    int count;
};

void ts_vtable_reg_init(struct ts_vtable_registry *reg) {
    reg->count = 0;
}

int ts_vtable_create(struct ts_vtable_registry *reg, int type_id) {
    if (reg->count >= 32) return -1;
    int idx = reg->count;
    reg->vtables[idx].type_id = type_id;
    reg->vtables[idx].count = 0;
    reg->count++;
    return idx;
}

int ts_vtable_add_method(struct ts_vtable_registry *reg, int vt_idx, int method_id, int fn_ptr_id) {
    if (vt_idx < 0 || vt_idx >= reg->count) return -1;
    struct ts_vtable *vt = &reg->vtables[vt_idx];
    if (vt->count >= 16) return -1;
    int idx = vt->count;
    vt->entries[idx].method_id = method_id;
    vt->entries[idx].fn_ptr_id = fn_ptr_id;
    vt->count++;
    return 0;
}

int ts_vtable_dispatch(struct ts_vtable_registry *reg, int type_id, int method_id) {
    int i, j;
    for (i = 0; i < reg->count; i++) {
        if (reg->vtables[i].type_id == type_id) {
            for (j = 0; j < reg->vtables[i].count; j++) {
                if (reg->vtables[i].entries[j].method_id == method_id) {
                    return reg->vtables[i].entries[j].fn_ptr_id;
                }
            }
        }
    }
    return -1;
}

int ts_vtable_selftest(void) {
    struct ts_vtable_registry reg;
    ts_vtable_reg_init(&reg);
    int vt = ts_vtable_create(&reg, 10);
    ts_vtable_add_method(&reg, vt, 1, 100);
    ts_vtable_add_method(&reg, vt, 2, 200);
    if (ts_vtable_dispatch(&reg, 10, 1) != 100) return -1;
    if (ts_vtable_dispatch(&reg, 10, 2) != 200) return -2;
    if (ts_vtable_dispatch(&reg, 10, 99) != -1) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1567: Vtable dispatch should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1567: Output should not be empty");
    assert!(
        code.contains("fn ts_vtable_dispatch"),
        "C1567: Should contain ts_vtable_dispatch function"
    );
}

/// C1568: Type descriptor with layout and trait information
#[test]
fn c1568_type_descriptor() {
    let c_code = r#"
struct ts_type_desc {
    int type_id;
    int size;
    int align;
    int num_fields;
    int traits_mask;
    int is_copy;
    int is_drop;
};

struct ts_desc_registry {
    struct ts_type_desc descs[64];
    int count;
};

void ts_desc_reg_init(struct ts_desc_registry *reg) {
    reg->count = 0;
}

int ts_desc_register(struct ts_desc_registry *reg, int type_id, int size, int align, int num_fields) {
    if (reg->count >= 64) return -1;
    int idx = reg->count;
    reg->descs[idx].type_id = type_id;
    reg->descs[idx].size = size;
    reg->descs[idx].align = align;
    reg->descs[idx].num_fields = num_fields;
    reg->descs[idx].traits_mask = 0;
    reg->descs[idx].is_copy = 0;
    reg->descs[idx].is_drop = 0;
    reg->count++;
    return idx;
}

int ts_desc_find(struct ts_desc_registry *reg, int type_id) {
    int i;
    for (i = 0; i < reg->count; i++) {
        if (reg->descs[i].type_id == type_id) return i;
    }
    return -1;
}

int ts_desc_set_traits(struct ts_desc_registry *reg, int type_id, int mask, int is_copy, int is_drop) {
    int idx = ts_desc_find(reg, type_id);
    if (idx < 0) return -1;
    reg->descs[idx].traits_mask = mask;
    reg->descs[idx].is_copy = is_copy;
    reg->descs[idx].is_drop = is_drop;
    return 0;
}

int ts_desc_needs_drop(struct ts_desc_registry *reg, int type_id) {
    int idx = ts_desc_find(reg, type_id);
    if (idx < 0) return 0;
    return reg->descs[idx].is_drop;
}

int ts_desc_selftest(void) {
    struct ts_desc_registry reg;
    ts_desc_reg_init(&reg);
    ts_desc_register(&reg, 10, 4, 4, 0);
    ts_desc_register(&reg, 20, 16, 8, 3);
    ts_desc_set_traits(&reg, 20, 7, 0, 1);
    if (ts_desc_needs_drop(&reg, 10)) return -1;
    if (!ts_desc_needs_drop(&reg, 20)) return -2;
    int idx = ts_desc_find(&reg, 20);
    if (idx < 0) return -3;
    if (reg.descs[idx].size != 16) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1568: Type descriptor should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1568: Output should not be empty");
    assert!(
        code.contains("fn ts_desc_register"),
        "C1568: Should contain ts_desc_register function"
    );
}

/// C1569: Reflection metadata for runtime type introspection
#[test]
fn c1569_reflection_metadata() {
    let c_code = r#"
struct ts_field_meta {
    int name_hash;
    int type_id;
    int offset;
    int size;
};

struct ts_type_meta {
    int type_id;
    int name_hash;
    struct ts_field_meta fields[16];
    int field_count;
    int total_size;
};

struct ts_meta_store {
    struct ts_type_meta types[32];
    int count;
};

void ts_meta_store_init(struct ts_meta_store *ms) {
    ms->count = 0;
}

int ts_meta_register_type(struct ts_meta_store *ms, int type_id, int name_hash, int total_size) {
    if (ms->count >= 32) return -1;
    int idx = ms->count;
    ms->types[idx].type_id = type_id;
    ms->types[idx].name_hash = name_hash;
    ms->types[idx].field_count = 0;
    ms->types[idx].total_size = total_size;
    ms->count++;
    return idx;
}

int ts_meta_add_field(struct ts_meta_store *ms, int type_idx, int name_hash, int type_id, int offset, int size) {
    if (type_idx < 0 || type_idx >= ms->count) return -1;
    struct ts_type_meta *tm = &ms->types[type_idx];
    if (tm->field_count >= 16) return -1;
    int idx = tm->field_count;
    tm->fields[idx].name_hash = name_hash;
    tm->fields[idx].type_id = type_id;
    tm->fields[idx].offset = offset;
    tm->fields[idx].size = size;
    tm->field_count++;
    return 0;
}

int ts_meta_get_field_offset(struct ts_meta_store *ms, int type_id, int field_name_hash) {
    int i, j;
    for (i = 0; i < ms->count; i++) {
        if (ms->types[i].type_id == type_id) {
            for (j = 0; j < ms->types[i].field_count; j++) {
                if (ms->types[i].fields[j].name_hash == field_name_hash) {
                    return ms->types[i].fields[j].offset;
                }
            }
        }
    }
    return -1;
}

int ts_meta_selftest(void) {
    struct ts_meta_store ms;
    ts_meta_store_init(&ms);
    int idx = ts_meta_register_type(&ms, 10, 1000, 12);
    ts_meta_add_field(&ms, idx, 100, 1, 0, 4);
    ts_meta_add_field(&ms, idx, 200, 2, 4, 4);
    ts_meta_add_field(&ms, idx, 300, 1, 8, 4);
    if (ts_meta_get_field_offset(&ms, 10, 200) != 4) return -1;
    if (ts_meta_get_field_offset(&ms, 10, 999) != -1) return -2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1569: Reflection metadata should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1569: Output should not be empty");
    assert!(
        code.contains("fn ts_meta_add_field"),
        "C1569: Should contain ts_meta_add_field function"
    );
}

/// C1570: Type ID hashing for fast structural type identity comparison
#[test]
fn c1570_type_id_hashing() {
    let c_code = r##"
typedef unsigned int uint32_t;

struct ts_type_id {
    uint32_t hash;
    int kind;
    int depth;
};

uint32_t ts_hash_combine(uint32_t seed, uint32_t value) {
    seed ^= value + 0x9e3779b9 + (seed << 6) + (seed >> 2);
    return seed;
}

struct ts_type_id ts_type_id_base(int kind) {
    struct ts_type_id tid;
    tid.kind = kind;
    tid.depth = 0;
    tid.hash = ts_hash_combine(0, (uint32_t)kind);
    return tid;
}

struct ts_type_id ts_type_id_compound(int kind, struct ts_type_id inner) {
    struct ts_type_id tid;
    tid.kind = kind;
    tid.depth = inner.depth + 1;
    tid.hash = ts_hash_combine(inner.hash, (uint32_t)kind);
    return tid;
}

int ts_type_id_eq(struct ts_type_id a, struct ts_type_id b) {
    return a.hash == b.hash && a.kind == b.kind && a.depth == b.depth;
}

int ts_type_id_selftest(void) {
    struct ts_type_id base_int = ts_type_id_base(1);
    struct ts_type_id base_int2 = ts_type_id_base(1);
    struct ts_type_id base_float = ts_type_id_base(2);
    if (!ts_type_id_eq(base_int, base_int2)) return -1;
    if (ts_type_id_eq(base_int, base_float)) return -2;
    struct ts_type_id ptr_int = ts_type_id_compound(3, base_int);
    if (ptr_int.depth != 1) return -3;
    if (ts_type_id_eq(ptr_int, base_int)) return -4;
    return 0;
}
"##;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1570: Type ID hashing should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1570: Output should not be empty");
    assert!(
        code.contains("fn ts_hash_combine"),
        "C1570: Should contain ts_hash_combine function"
    );
}

// ============================================================================
// C1571-C1575: Advanced Types
// ============================================================================

/// C1571: Dependent type index with bounds verification
#[test]
fn c1571_dependent_type_index() {
    let c_code = r#"
struct ts_dep_dim {
    int bound;
    int stride;
};

struct ts_dep_index {
    struct ts_dep_dim dims[4];
    int ndims;
};

void ts_dep_index_init(struct ts_dep_index *di, int ndims) {
    di->ndims = ndims;
    int i;
    for (i = 0; i < 4; i++) {
        di->dims[i].bound = 0;
        di->dims[i].stride = 1;
    }
}

void ts_dep_set_dim(struct ts_dep_index *di, int axis, int bound) {
    if (axis >= 0 && axis < di->ndims) {
        di->dims[axis].bound = bound;
    }
}

void ts_dep_compute_strides(struct ts_dep_index *di) {
    int stride = 1;
    int i;
    for (i = di->ndims - 1; i >= 0; i--) {
        di->dims[i].stride = stride;
        stride *= di->dims[i].bound;
    }
}

int ts_dep_in_bounds(struct ts_dep_index *di, int indices[], int num) {
    if (num != di->ndims) return 0;
    int i;
    for (i = 0; i < di->ndims; i++) {
        if (indices[i] < 0 || indices[i] >= di->dims[i].bound) return 0;
    }
    return 1;
}

int ts_dep_linearize(struct ts_dep_index *di, int indices[]) {
    int flat = 0;
    int i;
    for (i = 0; i < di->ndims; i++) {
        flat += indices[i] * di->dims[i].stride;
    }
    return flat;
}

int ts_dep_total_size(struct ts_dep_index *di) {
    int total = 1;
    int i;
    for (i = 0; i < di->ndims; i++) {
        total *= di->dims[i].bound;
    }
    return total;
}

int ts_dep_selftest(void) {
    struct ts_dep_index di;
    ts_dep_index_init(&di, 2);
    ts_dep_set_dim(&di, 0, 3);
    ts_dep_set_dim(&di, 1, 4);
    ts_dep_compute_strides(&di);
    int idx[2];
    idx[0] = 1; idx[1] = 2;
    if (!ts_dep_in_bounds(&di, idx, 2)) return -1;
    if (ts_dep_linearize(&di, idx) != 6) return -2;
    if (ts_dep_total_size(&di) != 12) return -3;
    idx[0] = 3;
    if (ts_dep_in_bounds(&di, idx, 2)) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1571: Dependent type index should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1571: Output should not be empty");
    assert!(
        code.contains("fn ts_dep_in_bounds"),
        "C1571: Should contain ts_dep_in_bounds function"
    );
}

/// C1572: Linear type tracker ensuring exactly-once use
#[test]
fn c1572_linear_type_tracker() {
    let c_code = r#"
enum ts_lin_state {
    TS_LIN_UNUSED = 0,
    TS_LIN_USED = 1,
    TS_LIN_CONSUMED = 2
};

struct ts_lin_resource {
    int resource_id;
    enum ts_lin_state state;
    int creation_scope;
};

struct ts_lin_tracker {
    struct ts_lin_resource resources[64];
    int count;
    int current_scope;
    int violation_count;
};

void ts_lin_tracker_init(struct ts_lin_tracker *lt) {
    lt->count = 0;
    lt->current_scope = 0;
    lt->violation_count = 0;
}

int ts_lin_create(struct ts_lin_tracker *lt, int resource_id) {
    if (lt->count >= 64) return -1;
    int idx = lt->count;
    lt->resources[idx].resource_id = resource_id;
    lt->resources[idx].state = TS_LIN_UNUSED;
    lt->resources[idx].creation_scope = lt->current_scope;
    lt->count++;
    return idx;
}

int ts_lin_use(struct ts_lin_tracker *lt, int resource_id) {
    int i;
    for (i = 0; i < lt->count; i++) {
        if (lt->resources[i].resource_id == resource_id) {
            if (lt->resources[i].state == TS_LIN_CONSUMED) {
                lt->violation_count++;
                return -1;
            }
            lt->resources[i].state = TS_LIN_USED;
            return 0;
        }
    }
    return -2;
}

int ts_lin_consume(struct ts_lin_tracker *lt, int resource_id) {
    int i;
    for (i = 0; i < lt->count; i++) {
        if (lt->resources[i].resource_id == resource_id) {
            if (lt->resources[i].state == TS_LIN_CONSUMED) {
                lt->violation_count++;
                return -1;
            }
            lt->resources[i].state = TS_LIN_CONSUMED;
            return 0;
        }
    }
    return -2;
}

int ts_lin_check_all_consumed(struct ts_lin_tracker *lt) {
    int i;
    int unconsumed = 0;
    for (i = 0; i < lt->count; i++) {
        if (lt->resources[i].state != TS_LIN_CONSUMED) unconsumed++;
    }
    return unconsumed;
}

int ts_lin_selftest(void) {
    struct ts_lin_tracker lt;
    ts_lin_tracker_init(&lt);
    ts_lin_create(&lt, 10);
    ts_lin_create(&lt, 20);
    ts_lin_use(&lt, 10);
    ts_lin_consume(&lt, 10);
    if (ts_lin_consume(&lt, 10) != -1) return -1;
    if (ts_lin_check_all_consumed(&lt) != 1) return -2;
    ts_lin_consume(&lt, 20);
    if (ts_lin_check_all_consumed(&lt) != 0) return -3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1572: Linear type tracker should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1572: Output should not be empty");
    assert!(
        code.contains("fn ts_lin_consume"),
        "C1572: Should contain ts_lin_consume function"
    );
}

/// C1573: Affine type checker allowing at-most-once use
#[test]
fn c1573_affine_type_checker() {
    let c_code = r#"
enum ts_aff_status {
    TS_AFF_LIVE = 0,
    TS_AFF_MOVED = 1,
    TS_AFF_DROPPED = 2
};

struct ts_aff_binding {
    int var_id;
    enum ts_aff_status status;
    int type_id;
    int scope;
};

struct ts_aff_checker {
    struct ts_aff_binding bindings[64];
    int count;
    int scope;
    int errors;
};

void ts_aff_checker_init(struct ts_aff_checker *ac) {
    ac->count = 0;
    ac->scope = 0;
    ac->errors = 0;
}

int ts_aff_bind(struct ts_aff_checker *ac, int var_id, int type_id) {
    if (ac->count >= 64) return -1;
    int idx = ac->count;
    ac->bindings[idx].var_id = var_id;
    ac->bindings[idx].status = TS_AFF_LIVE;
    ac->bindings[idx].type_id = type_id;
    ac->bindings[idx].scope = ac->scope;
    ac->count++;
    return idx;
}

int ts_aff_move(struct ts_aff_checker *ac, int var_id) {
    int i;
    for (i = ac->count - 1; i >= 0; i--) {
        if (ac->bindings[i].var_id == var_id) {
            if (ac->bindings[i].status != TS_AFF_LIVE) {
                ac->errors++;
                return -1;
            }
            ac->bindings[i].status = TS_AFF_MOVED;
            return 0;
        }
    }
    ac->errors++;
    return -2;
}

int ts_aff_is_live(struct ts_aff_checker *ac, int var_id) {
    int i;
    for (i = ac->count - 1; i >= 0; i--) {
        if (ac->bindings[i].var_id == var_id) {
            return ac->bindings[i].status == TS_AFF_LIVE;
        }
    }
    return 0;
}

void ts_aff_enter_scope(struct ts_aff_checker *ac) {
    ac->scope++;
}

void ts_aff_leave_scope(struct ts_aff_checker *ac) {
    int i;
    for (i = ac->count - 1; i >= 0; i--) {
        if (ac->bindings[i].scope == ac->scope) {
            if (ac->bindings[i].status == TS_AFF_LIVE) {
                ac->bindings[i].status = TS_AFF_DROPPED;
            }
        }
    }
    ac->scope--;
}

int ts_aff_selftest(void) {
    struct ts_aff_checker ac;
    ts_aff_checker_init(&ac);
    ts_aff_bind(&ac, 1, 100);
    if (!ts_aff_is_live(&ac, 1)) return -1;
    ts_aff_move(&ac, 1);
    if (ts_aff_is_live(&ac, 1)) return -2;
    if (ts_aff_move(&ac, 1) != -1) return -3;
    if (ac.errors != 1) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1573: Affine type checker should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1573: Output should not be empty");
    assert!(
        code.contains("fn ts_aff_move"),
        "C1573: Should contain ts_aff_move function"
    );
}

/// C1574: Effect system with effect bitmask tracking and subeffect checks
#[test]
fn c1574_effect_system() {
    let c_code = r#"
enum ts_effect {
    TS_EFF_PURE = 0,
    TS_EFF_READ = 1,
    TS_EFF_WRITE = 2,
    TS_EFF_ALLOC = 4,
    TS_EFF_IO = 8,
    TS_EFF_EXCEPT = 16,
    TS_EFF_DIVERGE = 32
};

struct ts_effect_set {
    int mask;
};

struct ts_effect_set ts_eff_empty(void) {
    struct ts_effect_set e;
    e.mask = TS_EFF_PURE;
    return e;
}

struct ts_effect_set ts_eff_add(struct ts_effect_set e, enum ts_effect eff) {
    e.mask |= (int)eff;
    return e;
}

struct ts_effect_set ts_eff_remove(struct ts_effect_set e, enum ts_effect eff) {
    e.mask &= ~(int)eff;
    return e;
}

int ts_eff_has(struct ts_effect_set e, enum ts_effect eff) {
    return (e.mask & (int)eff) != 0;
}

int ts_eff_is_pure(struct ts_effect_set e) {
    return e.mask == TS_EFF_PURE;
}

struct ts_effect_set ts_eff_join(struct ts_effect_set a, struct ts_effect_set b) {
    struct ts_effect_set r;
    r.mask = a.mask | b.mask;
    return r;
}

int ts_eff_subeffect(struct ts_effect_set sub, struct ts_effect_set sup) {
    return (sub.mask & ~sup.mask) == 0;
}

int ts_eff_count(struct ts_effect_set e) {
    int count = 0;
    int m = e.mask;
    while (m) {
        count += m & 1;
        m >>= 1;
    }
    return count;
}

int ts_eff_selftest(void) {
    struct ts_effect_set p = ts_eff_empty();
    if (!ts_eff_is_pure(p)) return -1;
    struct ts_effect_set rw = ts_eff_add(ts_eff_add(p, TS_EFF_READ), TS_EFF_WRITE);
    if (ts_eff_is_pure(rw)) return -2;
    if (!ts_eff_has(rw, TS_EFF_READ)) return -3;
    if (!ts_eff_subeffect(p, rw)) return -4;
    if (ts_eff_subeffect(rw, p)) return -5;
    if (ts_eff_count(rw) != 2) return -6;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1574: Effect system should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1574: Output should not be empty");
    assert!(
        code.contains("fn ts_eff_join"),
        "C1574: Should contain ts_eff_join function"
    );
}

/// C1575: Row polymorphism for extensible records with restriction and compatibility
#[test]
fn c1575_row_polymorphism() {
    let c_code = r#"
struct ts_row_field {
    int label_hash;
    int type_id;
};

struct ts_row_type {
    struct ts_row_field fields[16];
    int field_count;
    int has_tail;
    int tail_var;
};

void ts_row_init(struct ts_row_type *r) {
    r->field_count = 0;
    r->has_tail = 0;
    r->tail_var = -1;
}

void ts_row_set_tail(struct ts_row_type *r, int var) {
    r->has_tail = 1;
    r->tail_var = var;
}

int ts_row_add_field(struct ts_row_type *r, int label_hash, int type_id) {
    if (r->field_count >= 16) return -1;
    int idx = r->field_count;
    r->fields[idx].label_hash = label_hash;
    r->fields[idx].type_id = type_id;
    r->field_count++;
    return 0;
}

int ts_row_project(struct ts_row_type *r, int label_hash) {
    int i;
    for (i = 0; i < r->field_count; i++) {
        if (r->fields[i].label_hash == label_hash) return r->fields[i].type_id;
    }
    return -1;
}

int ts_row_restrict(struct ts_row_type *dst, struct ts_row_type *src, int exclude_label) {
    ts_row_init(dst);
    dst->has_tail = src->has_tail;
    dst->tail_var = src->tail_var;
    int i;
    for (i = 0; i < src->field_count; i++) {
        if (src->fields[i].label_hash != exclude_label) {
            ts_row_add_field(dst, src->fields[i].label_hash, src->fields[i].type_id);
        }
    }
    return dst->field_count;
}

int ts_row_compatible(struct ts_row_type *needed, struct ts_row_type *provided) {
    int i;
    for (i = 0; i < needed->field_count; i++) {
        int found = ts_row_project(provided, needed->fields[i].label_hash);
        if (found < 0) {
            if (!provided->has_tail) return 0;
        } else if (found != needed->fields[i].type_id) {
            return 0;
        }
    }
    return 1;
}

int ts_row_selftest(void) {
    struct ts_row_type r;
    ts_row_init(&r);
    ts_row_add_field(&r, 10, 1);
    ts_row_add_field(&r, 20, 2);
    ts_row_add_field(&r, 30, 3);
    if (ts_row_project(&r, 20) != 2) return -1;
    struct ts_row_type restricted;
    ts_row_restrict(&restricted, &r, 20);
    if (restricted.field_count != 2) return -2;
    if (ts_row_project(&restricted, 20) != -1) return -3;
    struct ts_row_type needed;
    ts_row_init(&needed);
    ts_row_add_field(&needed, 10, 1);
    if (!ts_row_compatible(&needed, &r)) return -4;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(
        result.is_ok(),
        "C1575: Row polymorphism should transpile: {:?}",
        result.err()
    );
    let code = result.unwrap();
    assert!(!code.is_empty(), "C1575: Output should not be empty");
    assert!(
        code.contains("fn ts_row_restrict"),
        "C1575: Should contain ts_row_restrict function"
    );
}
