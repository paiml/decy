//! Popperian Falsification Test Suite for Decy C-to-Rust Transpiler
//!
//! C926-C950: Graph Algorithm implementations -- the kind of C code found
//! in CLRS, Sedgewick, competitive programming, and real-world systems.
//! Tests are APPEND-ONLY per Popperian methodology.
//! Falsified tests are marked #[ignore = "FALSIFIED: reason"].
//!
//! These tests exercise classic graph algorithm patterns expressed as
//! valid C99 with array-based representations (no malloc/free, no #include).
//!
//! Organization:
//! - C926-C930: Traversal and shortest paths (BFS, DFS, Dijkstra, Bellman-Ford, Floyd-Warshall)
//! - C931-C935: MST and connectivity (Kruskal, Prim, topological sort, Kosaraju SCC, Tarjan SCC)
//! - C936-C940: Structure analysis (articulation points, bipartite, Euler path, Hamiltonian, A*)
//! - C941-C945: Flow and coloring (max flow, min cut, graph coloring, cycle detection, transitive closure)
//! - C946-C950: Advanced (longest path DAG, vertex cover, max independent set, PageRank, community detection)

// ============================================================================
// C926-C930: Traversal and Shortest Paths
// ============================================================================

#[test]
fn c926_bfs() {
    let c_code = r#"
typedef struct {
    int adj[100][100];
    int n;
} graph_Graph;

void graph_init(graph_Graph *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++)
            g->adj[i][j] = 0;
}

void graph_add_edge(graph_Graph *g, int u, int v) {
    g->adj[u][v] = 1;
    g->adj[v][u] = 1;
}

void graph_bfs(const graph_Graph *g, int start, int *visited, int *order, int *count) {
    int queue[100];
    int front = 0, back = 0;
    int i;

    for (i = 0; i < g->n; i++)
        visited[i] = 0;
    *count = 0;

    visited[start] = 1;
    queue[back++] = start;

    while (front < back) {
        int u = queue[front++];
        order[(*count)++] = u;

        for (i = 0; i < g->n; i++) {
            if (g->adj[u][i] && !visited[i]) {
                visited[i] = 1;
                queue[back++] = i;
            }
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C926: BFS - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C926: empty output");
    assert!(code.contains("fn graph_bfs"), "C926: Should contain graph_bfs");
}

#[test]
fn c927_dfs() {
    let c_code = r#"
typedef struct {
    int adj[100][100];
    int n;
} graph_DfsGraph;

void graph_dfs_init(graph_DfsGraph *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++)
            g->adj[i][j] = 0;
}

void graph_dfs_iterative(const graph_DfsGraph *g, int start, int *visited, int *order, int *count) {
    int stack[100];
    int top = -1;
    int i;

    for (i = 0; i < g->n; i++)
        visited[i] = 0;
    *count = 0;

    stack[++top] = start;

    while (top >= 0) {
        int u = stack[top--];

        if (visited[u])
            continue;

        visited[u] = 1;
        order[(*count)++] = u;

        for (i = g->n - 1; i >= 0; i--) {
            if (g->adj[u][i] && !visited[i]) {
                stack[++top] = i;
            }
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C927: DFS iterative - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C927: empty output");
    assert!(code.contains("fn graph_dfs_iterative"), "C927: Should contain graph_dfs_iterative");
}

#[test]
fn c928_dijkstra() {
    let c_code = r#"
typedef struct {
    int weight[100][100];
    int adj[100][100];
    int n;
} graph_WeightedGraph;

void graph_dijkstra_init(graph_WeightedGraph *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++) {
            g->adj[i][j] = 0;
            g->weight[i][j] = 0;
        }
}

void graph_dijkstra(const graph_WeightedGraph *g, int src, int *dist, int *prev) {
    int visited[100];
    int i, u, v;

    for (i = 0; i < g->n; i++) {
        dist[i] = 999999;
        prev[i] = -1;
        visited[i] = 0;
    }
    dist[src] = 0;

    for (i = 0; i < g->n; i++) {
        int min_dist = 999999;
        u = -1;
        for (v = 0; v < g->n; v++) {
            if (!visited[v] && dist[v] < min_dist) {
                min_dist = dist[v];
                u = v;
            }
        }
        if (u == -1) break;
        visited[u] = 1;

        for (v = 0; v < g->n; v++) {
            if (g->adj[u][v] && !visited[v]) {
                int new_dist = dist[u] + g->weight[u][v];
                if (new_dist < dist[v]) {
                    dist[v] = new_dist;
                    prev[v] = u;
                }
            }
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C928: Dijkstra - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C928: empty output");
    assert!(code.contains("fn graph_dijkstra"), "C928: Should contain graph_dijkstra");
}

#[test]
fn c929_bellman_ford() {
    let c_code = r#"
typedef struct {
    int src;
    int dst;
    int weight;
} graph_Edge;

typedef struct {
    graph_Edge edges[500];
    int num_edges;
    int num_vertices;
} graph_EdgeList;

void graph_edgelist_init(graph_EdgeList *g, int nv) {
    g->num_edges = 0;
    g->num_vertices = nv;
}

void graph_edgelist_add(graph_EdgeList *g, int s, int d, int w) {
    g->edges[g->num_edges].src = s;
    g->edges[g->num_edges].dst = d;
    g->edges[g->num_edges].weight = w;
    g->num_edges++;
}

int graph_bellman_ford(const graph_EdgeList *g, int src, int *dist) {
    int i, j;
    int changed;

    for (i = 0; i < g->num_vertices; i++)
        dist[i] = 999999;
    dist[src] = 0;

    for (i = 0; i < g->num_vertices - 1; i++) {
        changed = 0;
        for (j = 0; j < g->num_edges; j++) {
            int u = g->edges[j].src;
            int v = g->edges[j].dst;
            int w = g->edges[j].weight;
            if (dist[u] != 999999 && dist[u] + w < dist[v]) {
                dist[v] = dist[u] + w;
                changed = 1;
            }
        }
        if (!changed) break;
    }

    for (j = 0; j < g->num_edges; j++) {
        int u = g->edges[j].src;
        int v = g->edges[j].dst;
        int w = g->edges[j].weight;
        if (dist[u] != 999999 && dist[u] + w < dist[v])
            return 0;
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C929: Bellman-Ford - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C929: empty output");
    assert!(code.contains("fn graph_bellman_ford"), "C929: Should contain graph_bellman_ford");
}

#[test]
fn c930_floyd_warshall() {
    let c_code = r#"
typedef struct {
    int dist[50][50];
    int next[50][50];
    int n;
} graph_FWGraph;

void graph_fw_init(graph_FWGraph *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++) {
            if (i == j)
                g->dist[i][j] = 0;
            else
                g->dist[i][j] = 999999;
            g->next[i][j] = -1;
        }
}

void graph_fw_add_edge(graph_FWGraph *g, int u, int v, int w) {
    g->dist[u][v] = w;
    g->next[u][v] = v;
}

void graph_floyd_warshall(graph_FWGraph *g) {
    int k, i, j;
    for (k = 0; k < g->n; k++) {
        for (i = 0; i < g->n; i++) {
            for (j = 0; j < g->n; j++) {
                if (g->dist[i][k] != 999999 && g->dist[k][j] != 999999) {
                    int through_k = g->dist[i][k] + g->dist[k][j];
                    if (through_k < g->dist[i][j]) {
                        g->dist[i][j] = through_k;
                        g->next[i][j] = g->next[i][k];
                    }
                }
            }
        }
    }
}

int graph_fw_get_path(const graph_FWGraph *g, int u, int v, int *path) {
    int len = 0;
    if (g->next[u][v] == -1)
        return 0;
    path[len++] = u;
    while (u != v) {
        u = g->next[u][v];
        path[len++] = u;
    }
    return len;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C930: Floyd-Warshall - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C930: empty output");
    assert!(code.contains("fn graph_floyd_warshall"), "C930: Should contain graph_floyd_warshall");
}

// ============================================================================
// C931-C935: MST and Connectivity
// ============================================================================

#[test]
fn c931_kruskal_mst() {
    let c_code = r#"
typedef struct {
    int u;
    int v;
    int weight;
} graph_KEdge;

typedef struct {
    int parent[100];
    int rank[100];
} graph_UnionFind;

void graph_uf_init(graph_UnionFind *uf, int n) {
    int i;
    for (i = 0; i < n; i++) {
        uf->parent[i] = i;
        uf->rank[i] = 0;
    }
}

int graph_uf_find(graph_UnionFind *uf, int x) {
    while (uf->parent[x] != x)
        x = uf->parent[x];
    return x;
}

int graph_uf_union(graph_UnionFind *uf, int x, int y) {
    int rx = graph_uf_find(uf, x);
    int ry = graph_uf_find(uf, y);
    if (rx == ry) return 0;
    if (uf->rank[rx] < uf->rank[ry]) {
        uf->parent[rx] = ry;
    } else if (uf->rank[rx] > uf->rank[ry]) {
        uf->parent[ry] = rx;
    } else {
        uf->parent[ry] = rx;
        uf->rank[rx]++;
    }
    return 1;
}

void graph_sort_edges(graph_KEdge *edges, int n) {
    int i, j;
    for (i = 0; i < n - 1; i++) {
        for (j = 0; j < n - i - 1; j++) {
            if (edges[j].weight > edges[j + 1].weight) {
                graph_KEdge tmp = edges[j];
                edges[j] = edges[j + 1];
                edges[j + 1] = tmp;
            }
        }
    }
}

int graph_kruskal(graph_KEdge *edges, int num_edges, int num_vertices,
                  graph_KEdge *mst) {
    graph_UnionFind uf;
    int mst_size = 0;
    int i;

    graph_uf_init(&uf, num_vertices);
    graph_sort_edges(edges, num_edges);

    for (i = 0; i < num_edges && mst_size < num_vertices - 1; i++) {
        if (graph_uf_union(&uf, edges[i].u, edges[i].v)) {
            mst[mst_size++] = edges[i];
        }
    }
    return mst_size;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C931: Kruskal MST - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C931: empty output");
    assert!(code.contains("fn graph_kruskal"), "C931: Should contain graph_kruskal");
}

#[test]
fn c932_prim_mst() {
    let c_code = r#"
typedef struct {
    int adj[100][100];
    int weight[100][100];
    int n;
} graph_PrimGraph;

void graph_prim_init(graph_PrimGraph *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++) {
            g->adj[i][j] = 0;
            g->weight[i][j] = 0;
        }
}

int graph_prim(const graph_PrimGraph *g, int *parent) {
    int key[100];
    int in_mst[100];
    int i, u, v;
    int total_weight = 0;

    for (i = 0; i < g->n; i++) {
        key[i] = 999999;
        in_mst[i] = 0;
        parent[i] = -1;
    }
    key[0] = 0;

    for (i = 0; i < g->n; i++) {
        int min_key = 999999;
        u = -1;
        for (v = 0; v < g->n; v++) {
            if (!in_mst[v] && key[v] < min_key) {
                min_key = key[v];
                u = v;
            }
        }
        if (u == -1) break;
        in_mst[u] = 1;
        total_weight += key[u];

        for (v = 0; v < g->n; v++) {
            if (g->adj[u][v] && !in_mst[v] && g->weight[u][v] < key[v]) {
                key[v] = g->weight[u][v];
                parent[v] = u;
            }
        }
    }
    return total_weight;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C932: Prim MST - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C932: empty output");
    assert!(code.contains("fn graph_prim"), "C932: Should contain graph_prim");
}

#[test]
fn c933_topological_sort_kahn() {
    let c_code = r#"
typedef struct {
    int adj[100][100];
    int n;
} graph_DagGraph;

void graph_dag_init(graph_DagGraph *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++)
            g->adj[i][j] = 0;
}

void graph_dag_add_edge(graph_DagGraph *g, int u, int v) {
    g->adj[u][v] = 1;
}

int graph_topo_sort_kahn(const graph_DagGraph *g, int *order) {
    int in_degree[100];
    int queue[100];
    int front = 0, back = 0;
    int count = 0;
    int i, j;

    for (i = 0; i < g->n; i++) {
        in_degree[i] = 0;
    }
    for (i = 0; i < g->n; i++) {
        for (j = 0; j < g->n; j++) {
            if (g->adj[i][j])
                in_degree[j]++;
        }
    }

    for (i = 0; i < g->n; i++) {
        if (in_degree[i] == 0)
            queue[back++] = i;
    }

    while (front < back) {
        int u = queue[front++];
        order[count++] = u;

        for (j = 0; j < g->n; j++) {
            if (g->adj[u][j]) {
                in_degree[j]--;
                if (in_degree[j] == 0)
                    queue[back++] = j;
            }
        }
    }

    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C933: Topological sort (Kahn) - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C933: empty output");
    assert!(code.contains("fn graph_topo_sort_kahn"), "C933: Should contain graph_topo_sort_kahn");
}

#[test]
fn c934_kosaraju_scc() {
    let c_code = r#"
typedef struct {
    int adj[50][50];
    int radj[50][50];
    int n;
} graph_SccGraph;

void graph_scc_init(graph_SccGraph *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++) {
            g->adj[i][j] = 0;
            g->radj[i][j] = 0;
        }
}

void graph_scc_add_edge(graph_SccGraph *g, int u, int v) {
    g->adj[u][v] = 1;
    g->radj[v][u] = 1;
}

void graph_kosaraju_dfs1(const graph_SccGraph *g, int u, int *visited,
                          int *finish_stack, int *top) {
    int i;
    visited[u] = 1;
    for (i = 0; i < g->n; i++) {
        if (g->adj[u][i] && !visited[i])
            graph_kosaraju_dfs1(g, i, visited, finish_stack, top);
    }
    finish_stack[(*top)++] = u;
}

void graph_kosaraju_dfs2(const graph_SccGraph *g, int u, int *visited,
                          int *component, int comp_id) {
    int i;
    visited[u] = 1;
    component[u] = comp_id;
    for (i = 0; i < g->n; i++) {
        if (g->radj[u][i] && !visited[i])
            graph_kosaraju_dfs2(g, i, visited, component, comp_id);
    }
}

int graph_kosaraju(const graph_SccGraph *g, int *component) {
    int visited[50];
    int finish_stack[50];
    int top = 0;
    int num_scc = 0;
    int i;

    for (i = 0; i < g->n; i++)
        visited[i] = 0;

    for (i = 0; i < g->n; i++) {
        if (!visited[i])
            graph_kosaraju_dfs1(g, i, visited, finish_stack, &top);
    }

    for (i = 0; i < g->n; i++)
        visited[i] = 0;

    while (top > 0) {
        int u = finish_stack[--top];
        if (!visited[u]) {
            graph_kosaraju_dfs2(g, u, visited, component, num_scc);
            num_scc++;
        }
    }
    return num_scc;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C934: Kosaraju SCC - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C934: empty output");
    assert!(code.contains("fn graph_kosaraju"), "C934: Should contain graph_kosaraju");
}

#[test]
fn c935_tarjan_scc() {
    let c_code = r#"
typedef struct {
    int adj[50][50];
    int n;
} graph_TarjanGraph;

typedef struct {
    int disc[50];
    int low[50];
    int on_stack[50];
    int stack[50];
    int top;
    int timer;
    int comp[50];
    int num_scc;
} graph_TarjanState;

void graph_tarjan_init(graph_TarjanState *st, int n) {
    int i;
    st->top = -1;
    st->timer = 0;
    st->num_scc = 0;
    for (i = 0; i < n; i++) {
        st->disc[i] = -1;
        st->low[i] = -1;
        st->on_stack[i] = 0;
        st->comp[i] = -1;
    }
}

void graph_tarjan_dfs(const graph_TarjanGraph *g, int u, graph_TarjanState *st) {
    int i, v;
    st->disc[u] = st->low[u] = st->timer++;
    st->stack[++(st->top)] = u;
    st->on_stack[u] = 1;

    for (i = 0; i < g->n; i++) {
        if (!g->adj[u][i]) continue;
        v = i;
        if (st->disc[v] == -1) {
            graph_tarjan_dfs(g, v, st);
            if (st->low[v] < st->low[u])
                st->low[u] = st->low[v];
        } else if (st->on_stack[v]) {
            if (st->disc[v] < st->low[u])
                st->low[u] = st->disc[v];
        }
    }

    if (st->low[u] == st->disc[u]) {
        int w;
        do {
            w = st->stack[(st->top)--];
            st->on_stack[w] = 0;
            st->comp[w] = st->num_scc;
        } while (w != u);
        st->num_scc++;
    }
}

int graph_tarjan_scc(const graph_TarjanGraph *g, int *comp) {
    graph_TarjanState st;
    int i;
    graph_tarjan_init(&st, g->n);
    for (i = 0; i < g->n; i++) {
        if (st.disc[i] == -1)
            graph_tarjan_dfs(g, i, &st);
    }
    for (i = 0; i < g->n; i++)
        comp[i] = st.comp[i];
    return st.num_scc;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C935: Tarjan SCC - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C935: empty output");
    assert!(code.contains("fn graph_tarjan_scc"), "C935: Should contain graph_tarjan_scc");
}

// ============================================================================
// C936-C940: Structure Analysis
// ============================================================================

#[test]
fn c936_articulation_points() {
    let c_code = r#"
typedef struct {
    int adj[50][50];
    int n;
} graph_BridgeGraph;

typedef struct {
    int disc[50];
    int low[50];
    int parent[50];
    int is_ap[50];
    int timer;
} graph_APState;

void graph_ap_init(graph_APState *st, int n) {
    int i;
    st->timer = 0;
    for (i = 0; i < n; i++) {
        st->disc[i] = -1;
        st->low[i] = -1;
        st->parent[i] = -1;
        st->is_ap[i] = 0;
    }
}

void graph_ap_dfs(const graph_BridgeGraph *g, int u, graph_APState *st) {
    int children = 0;
    int i;
    st->disc[u] = st->low[u] = st->timer++;

    for (i = 0; i < g->n; i++) {
        if (!g->adj[u][i]) continue;
        if (st->disc[i] == -1) {
            children++;
            st->parent[i] = u;
            graph_ap_dfs(g, i, st);
            if (st->low[i] < st->low[u])
                st->low[u] = st->low[i];

            if (st->parent[u] == -1 && children > 1)
                st->is_ap[u] = 1;
            if (st->parent[u] != -1 && st->low[i] >= st->disc[u])
                st->is_ap[u] = 1;
        } else if (i != st->parent[u]) {
            if (st->disc[i] < st->low[u])
                st->low[u] = st->disc[i];
        }
    }
}

int graph_find_articulation_points(const graph_BridgeGraph *g, int *ap_list) {
    graph_APState st;
    int count = 0;
    int i;
    graph_ap_init(&st, g->n);
    for (i = 0; i < g->n; i++) {
        if (st.disc[i] == -1)
            graph_ap_dfs(g, i, &st);
    }
    for (i = 0; i < g->n; i++) {
        if (st.is_ap[i])
            ap_list[count++] = i;
    }
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C936: Articulation points - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C936: empty output");
    assert!(code.contains("fn graph_find_articulation_points"), "C936: Should contain graph_find_articulation_points");
}

#[test]
fn c937_bipartite_check() {
    let c_code = r#"
typedef struct {
    int adj[100][100];
    int n;
} graph_BiGraph;

void graph_bi_init(graph_BiGraph *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++)
            g->adj[i][j] = 0;
}

int graph_is_bipartite(const graph_BiGraph *g, int *color) {
    int queue[100];
    int front, back;
    int i, u;

    for (i = 0; i < g->n; i++)
        color[i] = -1;

    for (i = 0; i < g->n; i++) {
        if (color[i] != -1) continue;

        color[i] = 0;
        front = 0;
        back = 0;
        queue[back++] = i;

        while (front < back) {
            u = queue[front++];
            int v;
            for (v = 0; v < g->n; v++) {
                if (!g->adj[u][v]) continue;
                if (color[v] == -1) {
                    color[v] = 1 - color[u];
                    queue[back++] = v;
                } else if (color[v] == color[u]) {
                    return 0;
                }
            }
        }
    }
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C937: Bipartite check - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C937: empty output");
    assert!(code.contains("fn graph_is_bipartite"), "C937: Should contain graph_is_bipartite");
}

#[test]
fn c938_euler_path() {
    let c_code = r#"
typedef struct {
    int adj[50][50];
    int degree[50];
    int n;
} graph_EulerGraph;

void graph_euler_init(graph_EulerGraph *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++) {
        g->degree[i] = 0;
        for (j = 0; j < n; j++)
            g->adj[i][j] = 0;
    }
}

void graph_euler_add_edge(graph_EulerGraph *g, int u, int v) {
    g->adj[u][v]++;
    g->adj[v][u]++;
    g->degree[u]++;
    g->degree[v]++;
}

int graph_euler_find_start(const graph_EulerGraph *g) {
    int i;
    int odd_start = -1;
    for (i = 0; i < g->n; i++) {
        if (g->degree[i] % 2 != 0) {
            if (odd_start == -1)
                odd_start = i;
        }
    }
    if (odd_start != -1) return odd_start;
    for (i = 0; i < g->n; i++) {
        if (g->degree[i] > 0)
            return i;
    }
    return 0;
}

int graph_euler_path(graph_EulerGraph *g, int *path) {
    int stack[500];
    int top = -1;
    int path_len = 0;
    int start;
    int i;

    start = graph_euler_find_start(g);
    stack[++top] = start;

    while (top >= 0) {
        int u = stack[top];
        int found = 0;
        for (i = 0; i < g->n; i++) {
            if (g->adj[u][i] > 0) {
                g->adj[u][i]--;
                g->adj[i][u]--;
                stack[++top] = i;
                found = 1;
                break;
            }
        }
        if (!found) {
            path[path_len++] = stack[top--];
        }
    }
    return path_len;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C938: Euler path (Hierholzer) - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C938: empty output");
    assert!(code.contains("fn graph_euler_path"), "C938: Should contain graph_euler_path");
}

#[test]
fn c939_hamiltonian_path() {
    let c_code = r#"
typedef struct {
    int adj[20][20];
    int n;
} graph_HamGraph;

void graph_ham_init(graph_HamGraph *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++)
            g->adj[i][j] = 0;
}

int graph_ham_safe(const graph_HamGraph *g, int v, int *path, int pos) {
    int i;
    if (!g->adj[path[pos - 1]][v])
        return 0;
    for (i = 0; i < pos; i++) {
        if (path[i] == v)
            return 0;
    }
    return 1;
}

int graph_ham_solve(const graph_HamGraph *g, int *path, int pos) {
    int v;
    if (pos == g->n)
        return 1;

    for (v = 1; v < g->n; v++) {
        if (graph_ham_safe(g, v, path, pos)) {
            path[pos] = v;
            if (graph_ham_solve(g, path, pos + 1))
                return 1;
            path[pos] = -1;
        }
    }
    return 0;
}

int graph_hamiltonian_path(const graph_HamGraph *g, int *path) {
    int i;
    for (i = 0; i < g->n; i++)
        path[i] = -1;
    path[0] = 0;
    return graph_ham_solve(g, path, 1);
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C939: Hamiltonian path - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C939: empty output");
    assert!(code.contains("fn graph_hamiltonian_path"), "C939: Should contain graph_hamiltonian_path");
}

#[test]
fn c940_astar_pathfinding() {
    let c_code = r#"
typedef struct {
    int grid[50][50];
    int rows;
    int cols;
} graph_AStarGrid;

void graph_astar_init(graph_AStarGrid *g, int rows, int cols) {
    int i, j;
    g->rows = rows;
    g->cols = cols;
    for (i = 0; i < rows; i++)
        for (j = 0; j < cols; j++)
            g->grid[i][j] = 0;
}

int graph_astar_abs(int x) {
    return x < 0 ? -x : x;
}

int graph_astar_heuristic(int r1, int c1, int r2, int c2) {
    return graph_astar_abs(r1 - r2) + graph_astar_abs(c1 - c2);
}

int graph_astar_search(const graph_AStarGrid *g, int sr, int sc, int er, int ec,
                       int *path_r, int *path_c) {
    int g_score[50][50];
    int f_score[50][50];
    int came_from_r[50][50];
    int came_from_c[50][50];
    int closed[50][50];
    int open_r[2500], open_c[2500];
    int open_size = 0;
    int dr[4], dc[4];
    int i, j;

    dr[0] = -1; dr[1] = 1; dr[2] = 0; dr[3] = 0;
    dc[0] = 0; dc[1] = 0; dc[2] = -1; dc[3] = 1;

    for (i = 0; i < g->rows; i++)
        for (j = 0; j < g->cols; j++) {
            g_score[i][j] = 999999;
            f_score[i][j] = 999999;
            came_from_r[i][j] = -1;
            came_from_c[i][j] = -1;
            closed[i][j] = 0;
        }

    g_score[sr][sc] = 0;
    f_score[sr][sc] = graph_astar_heuristic(sr, sc, er, ec);
    open_r[open_size] = sr;
    open_c[open_size] = sc;
    open_size++;

    while (open_size > 0) {
        int best = 0;
        int cr, cc;
        for (i = 1; i < open_size; i++) {
            if (f_score[open_r[i]][open_c[i]] < f_score[open_r[best]][open_c[best]])
                best = i;
        }
        cr = open_r[best];
        cc = open_c[best];
        open_r[best] = open_r[open_size - 1];
        open_c[best] = open_c[open_size - 1];
        open_size--;

        if (cr == er && cc == ec) {
            int len = 0;
            int tr = er, tc = ec;
            while (tr != -1) {
                path_r[len] = tr;
                path_c[len] = tc;
                len++;
                int pr = came_from_r[tr][tc];
                int pc = came_from_c[tr][tc];
                tr = pr;
                tc = pc;
            }
            return len;
        }

        closed[cr][cc] = 1;

        for (i = 0; i < 4; i++) {
            int nr = cr + dr[i];
            int nc = cc + dc[i];
            if (nr < 0 || nr >= g->rows || nc < 0 || nc >= g->cols)
                continue;
            if (g->grid[nr][nc] || closed[nr][nc])
                continue;
            int tentative = g_score[cr][cc] + 1;
            if (tentative < g_score[nr][nc]) {
                came_from_r[nr][nc] = cr;
                came_from_c[nr][nc] = cc;
                g_score[nr][nc] = tentative;
                f_score[nr][nc] = tentative + graph_astar_heuristic(nr, nc, er, ec);
                open_r[open_size] = nr;
                open_c[open_size] = nc;
                open_size++;
            }
        }
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C940: A* pathfinding - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C940: empty output");
    assert!(code.contains("fn graph_astar_search"), "C940: Should contain graph_astar_search");
}

// ============================================================================
// C941-C945: Flow and Coloring
// ============================================================================

#[test]
fn c941_max_flow_ford_fulkerson() {
    let c_code = r#"
typedef struct {
    int cap[50][50];
    int flow[50][50];
    int n;
} graph_FlowNetwork;

void graph_flow_init(graph_FlowNetwork *net, int n) {
    int i, j;
    net->n = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++) {
            net->cap[i][j] = 0;
            net->flow[i][j] = 0;
        }
}

void graph_flow_add_edge(graph_FlowNetwork *net, int u, int v, int c) {
    net->cap[u][v] = c;
}

int graph_flow_bfs(const graph_FlowNetwork *net, int s, int t, int *parent) {
    int visited[50];
    int queue[50];
    int front = 0, back = 0;
    int i;

    for (i = 0; i < net->n; i++) {
        visited[i] = 0;
        parent[i] = -1;
    }
    visited[s] = 1;
    queue[back++] = s;

    while (front < back) {
        int u = queue[front++];
        for (i = 0; i < net->n; i++) {
            int residual = net->cap[u][i] - net->flow[u][i];
            if (!visited[i] && residual > 0) {
                visited[i] = 1;
                parent[i] = u;
                if (i == t) return 1;
                queue[back++] = i;
            }
        }
    }
    return 0;
}

int graph_max_flow(graph_FlowNetwork *net, int s, int t) {
    int parent[50];
    int max_flow_val = 0;

    while (graph_flow_bfs(net, s, t, parent)) {
        int path_flow = 999999;
        int v = t;
        while (v != s) {
            int u = parent[v];
            int residual = net->cap[u][v] - net->flow[u][v];
            if (residual < path_flow)
                path_flow = residual;
            v = u;
        }

        v = t;
        while (v != s) {
            int u = parent[v];
            net->flow[u][v] += path_flow;
            net->flow[v][u] -= path_flow;
            v = u;
        }

        max_flow_val += path_flow;
    }
    return max_flow_val;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C941: Max flow (Ford-Fulkerson) - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C941: empty output");
    assert!(code.contains("fn graph_max_flow"), "C941: Should contain graph_max_flow");
}

#[test]
fn c942_minimum_cut() {
    let c_code = r#"
typedef struct {
    int cap[50][50];
    int n;
} graph_CutGraph;

void graph_cut_init(graph_CutGraph *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++)
            g->cap[i][j] = 0;
}

int graph_cut_bfs(int cap[50][50], int n, int s, int t, int *parent) {
    int visited[50];
    int queue[50];
    int front = 0, back = 0;
    int i;

    for (i = 0; i < n; i++) {
        visited[i] = 0;
        parent[i] = -1;
    }
    visited[s] = 1;
    queue[back++] = s;

    while (front < back) {
        int u = queue[front++];
        for (i = 0; i < n; i++) {
            if (!visited[i] && cap[u][i] > 0) {
                visited[i] = 1;
                parent[i] = u;
                if (i == t) return 1;
                queue[back++] = i;
            }
        }
    }
    return 0;
}

int graph_min_cut(graph_CutGraph *g, int s, int t, int *cut_s, int *cut_t,
                  int *cut_size) {
    int rg[50][50];
    int parent[50];
    int visited[50];
    int queue[50];
    int front, back;
    int max_flow = 0;
    int i, j;

    for (i = 0; i < g->n; i++)
        for (j = 0; j < g->n; j++)
            rg[i][j] = g->cap[i][j];

    while (graph_cut_bfs(rg, g->n, s, t, parent)) {
        int path_flow = 999999;
        int v = t;
        while (v != s) {
            int u = parent[v];
            if (rg[u][v] < path_flow) path_flow = rg[u][v];
            v = u;
        }
        v = t;
        while (v != s) {
            int u = parent[v];
            rg[u][v] -= path_flow;
            rg[v][u] += path_flow;
            v = u;
        }
        max_flow += path_flow;
    }

    for (i = 0; i < g->n; i++) visited[i] = 0;
    front = 0; back = 0;
    visited[s] = 1;
    queue[back++] = s;
    while (front < back) {
        int u = queue[front++];
        for (i = 0; i < g->n; i++) {
            if (!visited[i] && rg[u][i] > 0) {
                visited[i] = 1;
                queue[back++] = i;
            }
        }
    }

    *cut_size = 0;
    for (i = 0; i < g->n; i++) {
        if (visited[i])
            cut_s[(*cut_size)++] = i;
    }
    int t_size = 0;
    for (i = 0; i < g->n; i++) {
        if (!visited[i])
            cut_t[t_size++] = i;
    }

    return max_flow;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C942: Minimum cut - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C942: empty output");
    assert!(code.contains("fn graph_min_cut"), "C942: Should contain graph_min_cut");
}

#[test]
fn c943_graph_coloring_greedy() {
    let c_code = r#"
typedef struct {
    int adj[100][100];
    int n;
} graph_ColorGraph;

void graph_color_init(graph_ColorGraph *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++)
            g->adj[i][j] = 0;
}

int graph_greedy_coloring(const graph_ColorGraph *g, int *colors) {
    int available[100];
    int i, u, v;
    int num_colors = 0;

    for (i = 0; i < g->n; i++)
        colors[i] = -1;

    colors[0] = 0;
    num_colors = 1;

    for (u = 1; u < g->n; u++) {
        for (i = 0; i < g->n; i++)
            available[i] = 1;

        for (v = 0; v < g->n; v++) {
            if (g->adj[u][v] && colors[v] != -1)
                available[colors[v]] = 0;
        }

        int c;
        for (c = 0; c < g->n; c++) {
            if (available[c]) {
                colors[u] = c;
                if (c + 1 > num_colors)
                    num_colors = c + 1;
                break;
            }
        }
    }
    return num_colors;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C943: Graph coloring (greedy) - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C943: empty output");
    assert!(code.contains("fn graph_greedy_coloring"), "C943: Should contain graph_greedy_coloring");
}

#[test]
fn c944_cycle_detection_directed() {
    let c_code = r#"
typedef struct {
    int adj[100][100];
    int n;
} graph_CycleGraph;

void graph_cycle_init(graph_CycleGraph *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++)
            g->adj[i][j] = 0;
}

int graph_cycle_dfs(const graph_CycleGraph *g, int u, int *color) {
    int i;
    color[u] = 1;
    for (i = 0; i < g->n; i++) {
        if (!g->adj[u][i]) continue;
        if (color[i] == 1) return 1;
        if (color[i] == 0 && graph_cycle_dfs(g, i, color))
            return 1;
    }
    color[u] = 2;
    return 0;
}

int graph_has_cycle(const graph_CycleGraph *g) {
    int color[100];
    int i;

    for (i = 0; i < g->n; i++)
        color[i] = 0;

    for (i = 0; i < g->n; i++) {
        if (color[i] == 0) {
            if (graph_cycle_dfs(g, i, color))
                return 1;
        }
    }
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C944: Cycle detection (directed) - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C944: empty output");
    assert!(code.contains("fn graph_has_cycle"), "C944: Should contain graph_has_cycle");
}

#[test]
fn c945_transitive_closure() {
    let c_code = r#"
typedef struct {
    int adj[50][50];
    int reach[50][50];
    int n;
} graph_TCGraph;

void graph_tc_init(graph_TCGraph *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++) {
            g->adj[i][j] = 0;
            g->reach[i][j] = 0;
        }
}

void graph_tc_add_edge(graph_TCGraph *g, int u, int v) {
    g->adj[u][v] = 1;
}

void graph_transitive_closure(graph_TCGraph *g) {
    int i, j, k;

    for (i = 0; i < g->n; i++)
        for (j = 0; j < g->n; j++)
            g->reach[i][j] = g->adj[i][j];

    for (i = 0; i < g->n; i++)
        g->reach[i][i] = 1;

    for (k = 0; k < g->n; k++) {
        for (i = 0; i < g->n; i++) {
            for (j = 0; j < g->n; j++) {
                if (g->reach[i][k] && g->reach[k][j])
                    g->reach[i][j] = 1;
            }
        }
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C945: Transitive closure - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C945: empty output");
    assert!(code.contains("fn graph_transitive_closure"), "C945: Should contain graph_transitive_closure");
}

// ============================================================================
// C946-C950: Advanced Graph Algorithms
// ============================================================================

#[test]
fn c946_longest_path_dag() {
    let c_code = r#"
typedef struct {
    int adj[50][50];
    int weight[50][50];
    int n;
} graph_DagWGraph;

void graph_dagw_init(graph_DagWGraph *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++) {
            g->adj[i][j] = 0;
            g->weight[i][j] = 0;
        }
}

void graph_dagw_topo(const graph_DagWGraph *g, int *order, int *count) {
    int in_deg[50];
    int queue[50];
    int front = 0, back = 0;
    int i, j;

    *count = 0;
    for (i = 0; i < g->n; i++) in_deg[i] = 0;
    for (i = 0; i < g->n; i++)
        for (j = 0; j < g->n; j++)
            if (g->adj[i][j]) in_deg[j]++;

    for (i = 0; i < g->n; i++)
        if (in_deg[i] == 0) queue[back++] = i;

    while (front < back) {
        int u = queue[front++];
        order[(*count)++] = u;
        for (j = 0; j < g->n; j++) {
            if (g->adj[u][j]) {
                in_deg[j]--;
                if (in_deg[j] == 0) queue[back++] = j;
            }
        }
    }
}

int graph_longest_path_dag(const graph_DagWGraph *g, int src, int *dist) {
    int order[50];
    int count = 0;
    int i, j, u;
    int max_dist = 0;

    graph_dagw_topo(g, order, &count);

    for (i = 0; i < g->n; i++)
        dist[i] = -999999;
    dist[src] = 0;

    for (i = 0; i < count; i++) {
        u = order[i];
        if (dist[u] == -999999) continue;
        for (j = 0; j < g->n; j++) {
            if (g->adj[u][j]) {
                int d = dist[u] + g->weight[u][j];
                if (d > dist[j])
                    dist[j] = d;
            }
        }
    }

    for (i = 0; i < g->n; i++) {
        if (dist[i] > max_dist)
            max_dist = dist[i];
    }
    return max_dist;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C946: Longest path in DAG - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C946: empty output");
    assert!(code.contains("fn graph_longest_path_dag"), "C946: Should contain graph_longest_path_dag");
}

#[test]
fn c947_vertex_cover_approx() {
    let c_code = r#"
typedef struct {
    int adj[100][100];
    int n;
} graph_VCGraph;

void graph_vc_init(graph_VCGraph *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++)
            g->adj[i][j] = 0;
}

int graph_vertex_cover_approx(const graph_VCGraph *g, int *cover) {
    int matched[100];
    int cover_size = 0;
    int i, j;

    for (i = 0; i < g->n; i++)
        matched[i] = 0;

    for (i = 0; i < g->n; i++) {
        if (matched[i]) continue;
        for (j = i + 1; j < g->n; j++) {
            if (matched[j]) continue;
            if (g->adj[i][j]) {
                cover[cover_size++] = i;
                cover[cover_size++] = j;
                matched[i] = 1;
                matched[j] = 1;
                break;
            }
        }
    }
    return cover_size;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C947: Vertex cover (approx) - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C947: empty output");
    assert!(code.contains("fn graph_vertex_cover_approx"), "C947: Should contain graph_vertex_cover_approx");
}

#[test]
fn c948_max_independent_set_greedy() {
    let c_code = r#"
typedef struct {
    int adj[100][100];
    int n;
} graph_MISGraph;

void graph_mis_init(graph_MISGraph *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++)
            g->adj[i][j] = 0;
}

int graph_degree(const graph_MISGraph *g, int v, const int *removed) {
    int deg = 0;
    int i;
    for (i = 0; i < g->n; i++) {
        if (!removed[i] && g->adj[v][i])
            deg++;
    }
    return deg;
}

int graph_max_independent_set(const graph_MISGraph *g, int *mis) {
    int removed[100];
    int mis_size = 0;
    int i, remaining;

    for (i = 0; i < g->n; i++)
        removed[i] = 0;

    remaining = g->n;
    while (remaining > 0) {
        int min_deg = g->n + 1;
        int best = -1;

        for (i = 0; i < g->n; i++) {
            if (removed[i]) continue;
            int d = graph_degree(g, i, removed);
            if (d < min_deg) {
                min_deg = d;
                best = i;
            }
        }

        if (best == -1) break;

        mis[mis_size++] = best;
        removed[best] = 1;
        remaining--;

        for (i = 0; i < g->n; i++) {
            if (!removed[i] && g->adj[best][i]) {
                removed[i] = 1;
                remaining--;
            }
        }
    }
    return mis_size;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C948: Max independent set (greedy) - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C948: empty output");
    assert!(code.contains("fn graph_max_independent_set"), "C948: Should contain graph_max_independent_set");
}

#[test]
fn c949_pagerank_iterative() {
    let c_code = r#"
typedef struct {
    int adj[50][50];
    int out_degree[50];
    int n;
} graph_PRGraph;

void graph_pr_init(graph_PRGraph *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++) {
        g->out_degree[i] = 0;
        for (j = 0; j < n; j++)
            g->adj[i][j] = 0;
    }
}

void graph_pr_add_edge(graph_PRGraph *g, int u, int v) {
    g->adj[u][v] = 1;
    g->out_degree[u]++;
}

void graph_pagerank(const graph_PRGraph *g, double *rank, int iterations,
                    double damping) {
    double new_rank[50];
    int iter, i, j;
    double base = (1.0 - damping) / g->n;

    for (i = 0; i < g->n; i++)
        rank[i] = 1.0 / g->n;

    for (iter = 0; iter < iterations; iter++) {
        for (i = 0; i < g->n; i++)
            new_rank[i] = base;

        for (i = 0; i < g->n; i++) {
            if (g->out_degree[i] == 0) {
                double share = damping * rank[i] / g->n;
                for (j = 0; j < g->n; j++)
                    new_rank[j] += share;
            } else {
                double share = damping * rank[i] / g->out_degree[i];
                for (j = 0; j < g->n; j++) {
                    if (g->adj[i][j])
                        new_rank[j] += share;
                }
            }
        }

        for (i = 0; i < g->n; i++)
            rank[i] = new_rank[i];
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C949: PageRank (iterative) - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C949: empty output");
    assert!(code.contains("fn graph_pagerank"), "C949: Should contain graph_pagerank");
}

#[test]
fn c950_community_detection_label_propagation() {
    let c_code = r#"
typedef struct {
    int adj[100][100];
    int n;
} graph_CDGraph;

void graph_cd_init(graph_CDGraph *g, int n) {
    int i, j;
    g->n = n;
    for (i = 0; i < n; i++)
        for (j = 0; j < n; j++)
            g->adj[i][j] = 0;
}

int graph_most_frequent_label(const graph_CDGraph *g, int v, const int *labels) {
    int counts[100];
    int i;
    int best_label = labels[v];
    int best_count = 0;

    for (i = 0; i < g->n; i++)
        counts[i] = 0;

    for (i = 0; i < g->n; i++) {
        if (g->adj[v][i])
            counts[labels[i]]++;
    }

    for (i = 0; i < g->n; i++) {
        if (counts[i] > best_count) {
            best_count = counts[i];
            best_label = i;
        }
    }
    return best_label;
}

int graph_label_propagation(const graph_CDGraph *g, int *labels, int max_iter) {
    int order[100];
    int iter, i, j;
    int changed;

    for (i = 0; i < g->n; i++) {
        labels[i] = i;
        order[i] = i;
    }

    for (iter = 0; iter < max_iter; iter++) {
        changed = 0;

        for (i = g->n - 1; i > 0; i--) {
            j = i / 2;
            int tmp = order[i];
            order[i] = order[j];
            order[j] = tmp;
        }

        for (i = 0; i < g->n; i++) {
            int v = order[i];
            int new_label = graph_most_frequent_label(g, v, labels);
            if (new_label != labels[v]) {
                labels[v] = new_label;
                changed = 1;
            }
        }

        if (!changed) break;
    }

    int num_communities = 0;
    int seen[100];
    for (i = 0; i < g->n; i++) seen[i] = 0;
    for (i = 0; i < g->n; i++) {
        if (!seen[labels[i]]) {
            seen[labels[i]] = 1;
            num_communities++;
        }
    }
    return num_communities;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "C950: Community detection (label propagation) - failed: {:?}", result.err());
    let code = result.unwrap();
    assert!(!code.is_empty(), "C950: empty output");
    assert!(code.contains("fn graph_label_propagation"), "C950: Should contain graph_label_propagation");
}
