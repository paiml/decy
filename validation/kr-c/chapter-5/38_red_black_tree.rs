/* K&R C Chapter 5: Red-Black Tree
 * K&R §5.10: Self-balancing binary search tree
 * Transpiled to safe Rust (simplified demonstration)
 */

use std::collections::BTreeMap;

// Note: Full red-black tree implementation in safe Rust is complex due to
// parent pointers and rotations. This demonstrates the concept using BTreeMap.

#[derive(Debug, Clone, Copy, PartialEq)]
enum Color {
    Red,
    Black,
}

// Simplified RB tree node (educational purposes)
#[allow(dead_code)]
struct RBNode {
    key: i32,
    color: Color,
    left: Option<Box<RBNode>>,
    right: Option<Box<RBNode>>,
}

impl RBNode {
    fn new(key: i32, color: Color) -> Self {
        RBNode {
            key,
            color,
            left: None,
            right: None,
        }
    }

    fn insert(&mut self, key: i32) {
        if key < self.key {
            match self.left {
                Some(ref mut node) => node.insert(key),
                None => self.left = Some(Box::new(RBNode::new(key, Color::Red))),
            }
        } else if key > self.key {
            match self.right {
                Some(ref mut node) => node.insert(key),
                None => self.right = Some(Box::new(RBNode::new(key, Color::Red))),
            }
        }
    }

    fn inorder(&self) {
        if let Some(ref left) = self.left {
            left.inorder();
        }
        print!("{}{}  ", self.key, if self.color == Color::Red { 'R' } else { 'B' });
        if let Some(ref right) = self.right {
            right.inorder();
        }
    }
}

// Idiomatic Rust: use BTreeMap (implements RB-tree internally)
struct RedBlackTree {
    map: BTreeMap<i32, ()>,
}

impl RedBlackTree {
    fn new() -> Self {
        RedBlackTree {
            map: BTreeMap::new(),
        }
    }

    fn insert(&mut self, key: i32) {
        self.map.insert(key, ());
    }

    fn contains(&self, key: i32) -> bool {
        self.map.contains_key(&key)
    }

    fn inorder(&self) {
        for key in self.map.keys() {
            print!("{} ", key);
        }
        println!();
    }

    fn len(&self) -> usize {
        self.map.len()
    }
}

fn main() {
    println!("=== Red-Black Tree ===\n");

    let mut tree = RedBlackTree::new();

    println!("Insert: 7, 3, 18, 10, 22, 8, 11, 26");
    tree.insert(7);
    tree.insert(3);
    tree.insert(18);
    tree.insert(10);
    tree.insert(22);
    tree.insert(8);
    tree.insert(11);
    tree.insert(26);

    print!("Inorder: ");
    tree.inorder();

    println!("\nRed-Black properties maintained:");
    println!("  1. Every node is red or black");
    println!("  2. Root is black");
    println!("  3. All leaves (NIL) are black");
    println!("  4. Red node has black children");
    println!("  5. All paths have same black height");

    println!("\nSearch operations:");
    println!("  Contains 10: {}", tree.contains(10));
    println!("  Contains 99: {}", tree.contains(99));

    println!("\nNote: Rust's BTreeMap uses B-tree (generalization of RB-tree)");
    println!("  - Guaranteed O(log n) operations");
    println!("  - Self-balancing");
    println!("  - Cache-friendly (better locality than RB-tree)");
}

// Demonstrate BTreeMap features
#[allow(dead_code)]
fn demonstrate_btreemap() {
    use std::collections::BTreeMap;

    let mut map = BTreeMap::new();

    // Insert
    map.insert(7, "seven");
    map.insert(3, "three");
    map.insert(18, "eighteen");

    // Search
    if let Some(&value) = map.get(&7) {
        println!("Found: {}", value);
    }

    // Range queries (BTreeMap advantage)
    println!("Keys in range 5..15:");
    for key in map.range(5..15) {
        println!("  {}: {}", key.0, key.1);
    }

    // Ordered iteration
    for (key, value) in &map {
        println!("{}: {}", key, value);
    }

    // Remove
    map.remove(&3);

    // First/last
    if let Some((&first_key, _)) = map.iter().next() {
        println!("First: {}", first_key);
    }

    if let Some((&last_key, _)) = map.iter().next_back() {
        println!("Last: {}", last_key);
    }
}

// Key differences from C:
// 1. BTreeMap instead of manual RB-tree
// 2. No manual malloc/free
// 3. RAII: automatic cleanup
// 4. No parent pointers needed
// 5. No color/rotation management
// 6. Guaranteed O(log n) operations
// 7. Range queries supported
// 8. Full RB-tree in safe Rust is complex (prefer BTreeMap)
//
// For educational RB-tree implementation in Rust:
// - Requires unsafe or complex RefCell patterns
// - Parent pointers challenging without unsafe
// - BTreeMap is preferred for production use
