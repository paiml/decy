/* K&R C Chapter 5: Union-Find (Disjoint Set)
 * K&R §5.10: Path compression and union by rank
 * Transpiled to safe Rust (using Vec for parent and rank arrays)
 */

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
    size: usize,
}

impl UnionFind {
    fn new(size: usize) -> Self {
        UnionFind {
            parent: (0..size).collect(),
            rank: vec![0; size],
            size,
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]); // Path compression
        }
        self.parent[x]
    }

    fn union(&mut self, x: usize, y: usize) {
        let root_x = self.find(x);
        let root_y = self.find(y);

        if root_x == root_y {
            return;
        }

        // Union by rank
        match self.rank[root_x].cmp(&self.rank[root_y]) {
            std::cmp::Ordering::Less => {
                self.parent[root_x] = root_y;
            }
            std::cmp::Ordering::Greater => {
                self.parent[root_y] = root_x;
            }
            std::cmp::Ordering::Equal => {
                self.parent[root_y] = root_x;
                self.rank[root_x] += 1;
            }
        }
    }

    fn connected(&mut self, x: usize, y: usize) -> bool {
        self.find(x) == self.find(y)
    }

    fn count_components(&mut self) -> usize {
        let mut count = 0;
        for i in 0..self.size {
            if self.find(i) == i {
                count += 1;
            }
        }
        count
    }

    fn print(&mut self) {
        println!("Disjoint sets:");
        for i in 0..self.size {
            let root = self.find(i);
            println!("  {} -> root {}", i, root);
        }
    }
}

fn main() {
    println!("=== Union-Find (Disjoint Set) ===\n");

    let mut uf = UnionFind::new(10);

    println!("Initial: 10 disjoint elements");
    println!("Components: {}\n", uf.count_components());

    println!("Union operations:");
    println!("  Union(0, 1)");
    uf.union(0, 1);
    println!("  Union(2, 3)");
    uf.union(2, 3);
    println!("  Union(0, 2)");
    uf.union(0, 2);
    println!("  Union(4, 5)");
    uf.union(4, 5);
    println!("  Union(6, 7)");
    uf.union(6, 7);
    println!();

    uf.print();
    println!("\nComponents: {}", uf.count_components());

    println!("\nConnectivity queries:");
    println!("  Connected(0, 3): {}", if uf.connected(0, 3) { "Yes" } else { "No" });
    println!("  Connected(0, 4): {}", if uf.connected(0, 4) { "Yes" } else { "No" });
    println!("  Connected(6, 7): {}", if uf.connected(6, 7) { "Yes" } else { "No" });
}

// Advanced union-find with size tracking
#[allow(dead_code)]
struct UnionFindWithSize {
    parent: Vec<usize>,
    size: Vec<usize>,
}

impl UnionFindWithSize {
    fn new(n: usize) -> Self {
        UnionFindWithSize {
            parent: (0..n).collect(),
            size: vec![1; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, x: usize, y: usize) -> bool {
        let root_x = self.find(x);
        let root_y = self.find(y);

        if root_x == root_y {
            return false;
        }

        // Union by size
        if self.size[root_x] < self.size[root_y] {
            self.parent[root_x] = root_y;
            self.size[root_y] += self.size[root_x];
        } else {
            self.parent[root_y] = root_x;
            self.size[root_x] += self.size[root_y];
        }

        true
    }

    fn get_size(&mut self, x: usize) -> usize {
        let root = self.find(x);
        self.size[root]
    }
}

// Key differences from C:
// 1. Vec<usize> instead of int*
// 2. No manual malloc/free
// 3. RAII: automatic cleanup
// 4. (0..size).collect() for initialization
// 5. Path compression in find()
// 6. Union by rank for balance
// 7. cmp() for three-way comparison
// 8. Time complexity: O(α(n)) amortized (nearly constant)
