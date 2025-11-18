/* K&R C Chapter 5: Pointer-Based Binary Search Tree
 * BST implementation with insert, search, traversal
 * Transpiled to safe Rust (using Box<TreeNode> and Option)
 */

type TreeLink = Option<Box<TreeNode>>;

struct TreeNode {
    value: i32,
    left: TreeLink,
    right: TreeLink,
}

impl TreeNode {
    fn new(value: i32) -> Self {
        TreeNode {
            value,
            left: None,
            right: None,
        }
    }

    fn insert(&mut self, value: i32) {
        if value < self.value {
            match self.left {
                Some(ref mut node) => node.insert(value),
                None => self.left = Some(Box::new(TreeNode::new(value))),
            }
        } else if value > self.value {
            match self.right {
                Some(ref mut node) => node.insert(value),
                None => self.right = Some(Box::new(TreeNode::new(value))),
            }
        }
        // Equal values are ignored
    }

    fn search(&self, value: i32) -> Option<&TreeNode> {
        if value == self.value {
            Some(self)
        } else if value < self.value {
            self.left.as_ref()?.search(value)
        } else {
            self.right.as_ref()?.search(value)
        }
    }

    fn inorder(&self) {
        if let Some(ref left) = self.left {
            left.inorder();
        }
        print!("{} ", self.value);
        if let Some(ref right) = self.right {
            right.inorder();
        }
    }

    fn preorder(&self) {
        print!("{} ", self.value);
        if let Some(ref left) = self.left {
            left.preorder();
        }
        if let Some(ref right) = self.right {
            right.preorder();
        }
    }

    fn postorder(&self) {
        if let Some(ref left) = self.left {
            left.postorder();
        }
        if let Some(ref right) = self.right {
            right.postorder();
        }
        print!("{} ", self.value);
    }

    fn height(&self) -> usize {
        let left_height = self.left.as_ref().map_or(0, |n| n.height());
        let right_height = self.right.as_ref().map_or(0, |n| n.height());
        1 + left_height.max(right_height)
    }

    fn count(&self) -> usize {
        let left_count = self.left.as_ref().map_or(0, |n| n.count());
        let right_count = self.right.as_ref().map_or(0, |n| n.count());
        1 + left_count + right_count
    }
}

struct BinarySearchTree {
    root: TreeLink,
}

impl BinarySearchTree {
    fn new() -> Self {
        BinarySearchTree { root: None }
    }

    fn insert(&mut self, value: i32) {
        match self.root {
            Some(ref mut node) => node.insert(value),
            None => self.root = Some(Box::new(TreeNode::new(value))),
        }
    }

    fn search(&self, value: i32) -> bool {
        self.root.as_ref().map_or(false, |node| node.search(value).is_some())
    }

    fn inorder(&self) {
        if let Some(ref node) = self.root {
            node.inorder();
        }
    }

    fn preorder(&self) {
        if let Some(ref node) = self.root {
            node.preorder();
        }
    }

    fn postorder(&self) {
        if let Some(ref node) = self.root {
            node.postorder();
        }
    }

    fn height(&self) -> usize {
        self.root.as_ref().map_or(0, |node| node.height())
    }

    fn count(&self) -> usize {
        self.root.as_ref().map_or(0, |node| node.count())
    }
}

fn main() {
    let mut tree = BinarySearchTree::new();

    println!("=== Binary Search Tree Demo ===\n");

    // Build tree
    let values = [50, 30, 70, 20, 40, 60, 80, 10, 25, 35, 65];

    print!("Inserting values: ");
    for &value in &values {
        print!("{} ", value);
        tree.insert(value);
    }
    println!();

    // Tree statistics
    println!("\nTree statistics:");
    println!("  Nodes: {}", tree.count());
    println!("  Height: {}", tree.height());

    // Traversals
    println!("\nTraversals:");
    print!("  Inorder:   ");
    tree.inorder();
    println!();

    print!("  Preorder:  ");
    tree.preorder();
    println!();

    print!("  Postorder: ");
    tree.postorder();
    println!();

    // Search
    println!("\nSearching:");
    let search_vals = [35, 100];
    for &val in &search_vals {
        if tree.search(val) {
            println!("  Found {}", val);
        } else {
            println!("  {} not found", val);
        }
    }

    // Cleanup (automatic via Drop)
    println!("\nTree will be automatically freed (RAII)");
}

// Alternative: using BTreeMap (idiomatic Rust)
#[allow(dead_code)]
fn demonstrate_btreemap() {
    use std::collections::BTreeSet;

    let mut tree = BTreeSet::new();

    // Insert
    tree.insert(50);
    tree.insert(30);
    tree.insert(70);

    // Search
    if tree.contains(&50) {
        println!("Found 50");
    }

    // Iteration (sorted)
    for value in &tree {
        println!("{}", value);
    }
}

// Key differences from C:
// 1. Option<Box<TreeNode>> instead of TreeNode*
// 2. No manual malloc/free - Box handles allocation
// 3. RAII: automatic cleanup via Drop
// 4. Pattern matching for tree traversal
// 5. as_ref() for borrowing from Option
// 6. map_or() for Option with default value
// 7. No NULL checks - Option handles it
// 8. Prefer BTreeMap/BTreeSet for production
