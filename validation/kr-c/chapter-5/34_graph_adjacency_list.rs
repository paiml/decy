/* K&R C Chapter 5: Graph with Adjacency List
 * K&R §5.10: Pointer-based graph representation
 * Transpiled to safe Rust (using Vec<Vec<Edge>>)
 */

use std::collections::VecDeque;

#[derive(Clone)]
struct Edge {
    vertex: usize,
    weight: i32,
}

struct Graph {
    adj_lists: Vec<Vec<Edge>>,
    num_vertices: usize,
}

impl Graph {
    fn new(num_vertices: usize) -> Self {
        Graph {
            adj_lists: vec![Vec::new(); num_vertices],
            num_vertices,
        }
    }

    fn add_edge(&mut self, src: usize, dest: usize, weight: i32) {
        self.adj_lists[src].push(Edge {
            vertex: dest,
            weight,
        });
    }

    fn add_undirected_edge(&mut self, v1: usize, v2: usize, weight: i32) {
        self.add_edge(v1, v2, weight);
        self.add_edge(v2, v1, weight);
    }

    fn print(&self) {
        println!("Graph adjacency lists:");
        for (i, edges) in self.adj_lists.iter().enumerate() {
            print!("  {}: ", i);
            for edge in edges {
                print!("-> {}(w={}) ", edge.vertex, edge.weight);
            }
            println!();
        }
    }

    fn dfs(&self, start: usize) {
        let mut visited = vec![false; self.num_vertices];
        print!("DFS from {}: ", start);
        self.dfs_helper(start, &mut visited);
        println!();
    }

    fn dfs_helper(&self, vertex: usize, visited: &mut Vec<bool>) {
        visited[vertex] = true;
        print!("{} ", vertex);

        for edge in &self.adj_lists[vertex] {
            if !visited[edge.vertex] {
                self.dfs_helper(edge.vertex, visited);
            }
        }
    }

    fn bfs(&self, start: usize) {
        let mut visited = vec![false; self.num_vertices];
        let mut queue = VecDeque::new();

        visited[start] = true;
        queue.push_back(start);

        print!("BFS from {}: ", start);

        while let Some(vertex) = queue.pop_front() {
            print!("{} ", vertex);

            for edge in &self.adj_lists[vertex] {
                if !visited[edge.vertex] {
                    visited[edge.vertex] = true;
                    queue.push_back(edge.vertex);
                }
            }
        }

        println!();
    }

    fn has_path(&self, from: usize, to: usize) -> bool {
        let mut visited = vec![false; self.num_vertices];
        self.has_path_helper(from, to, &mut visited)
    }

    fn has_path_helper(&self, current: usize, target: usize, visited: &mut Vec<bool>) -> bool {
        if current == target {
            return true;
        }

        visited[current] = true;

        for edge in &self.adj_lists[current] {
            if !visited[edge.vertex] && self.has_path_helper(edge.vertex, target, visited) {
                return true;
            }
        }

        false
    }
}

fn main() {
    println!("=== Graph with Adjacency List ===\n");

    let mut graph = Graph::new(6);

    println!("Adding edges:");
    graph.add_undirected_edge(0, 1, 5);
    graph.add_undirected_edge(0, 2, 3);
    graph.add_undirected_edge(1, 3, 7);
    graph.add_undirected_edge(2, 3, 2);
    graph.add_undirected_edge(3, 4, 6);
    graph.add_undirected_edge(4, 5, 1);

    graph.print();
    println!();

    graph.dfs(0);
    graph.bfs(0);

    println!("\nPath queries:");
    println!("  Has path 0->5: {}", graph.has_path(0, 5));
    println!("  Has path 0->0: {}", graph.has_path(0, 0));
}

// Advanced graph operations
#[allow(dead_code)]
fn demonstrate_advanced_graphs() {
    use std::collections::{HashMap, HashSet};

    // Weighted graph with HashMap (for sparse graphs)
    let mut graph: HashMap<usize, Vec<(usize, i32)>> = HashMap::new();
    graph.entry(0).or_insert_with(Vec::new).push((1, 5));
    graph.entry(0).or_insert_with(Vec::new).push((2, 3));

    // Find all neighbors
    if let Some(neighbors) = graph.get(&0) {
        for (vertex, weight) in neighbors {
            println!("0 -> {} (weight: {})", vertex, weight);
        }
    }

    // Topological sort (for DAG)
    fn topological_sort(graph: &Graph) -> Vec<usize> {
        let mut visited = vec![false; graph.num_vertices];
        let mut stack = Vec::new();

        fn dfs_topo(v: usize, graph: &Graph, visited: &mut Vec<bool>, stack: &mut Vec<usize>) {
            visited[v] = true;
            for edge in &graph.adj_lists[v] {
                if !visited[edge.vertex] {
                    dfs_topo(edge.vertex, graph, visited, stack);
                }
            }
            stack.push(v);
        }

        for v in 0..graph.num_vertices {
            if !visited[v] {
                dfs_topo(v, graph, &mut visited, &mut stack);
            }
        }

        stack.reverse();
        stack
    }
}

// Key differences from C:
// 1. Vec<Vec<Edge>> instead of AdjNode**
// 2. No manual malloc/free for nodes
// 3. RAII: automatic cleanup
// 4. VecDeque for BFS queue
// 5. Vec<bool> for visited tracking
// 6. Bounds checking automatic
// 7. Clone trait for Edge copies
// 8. For advanced graphs: use petgraph crate
