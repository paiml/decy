/* K&R C Chapter 6.5: Self-referential Structures
 * Page 139-140
 * Binary tree with self-referential structure
 * Transpiled to safe Rust using Box
 */

struct TNode {
    word: &'static str,
    count: i32,
    left: Option<Box<TNode>>,
    right: Option<Box<TNode>>,
}

impl TNode {
    fn new(word: &'static str, count: i32) -> Box<TNode> {
        Box::new(TNode {
            word,
            count,
            left: None,
            right: None,
        })
    }
}

fn main() {
    // Create simple tree: root with two children
    let mut root = TNode::new("hello", 1);

    root.left = Some(TNode::new("apple", 1));
    root.right = Some(TNode::new("world", 1));

    // Print tree
    println!("Root: {} ({})", root.word, root.count);
    if let Some(ref left) = root.left {
        println!("Left: {} ({})", left.word, left.count);
    }
    if let Some(ref right) = root.right {
        println!("Right: {} ({})", right.word, right.count);
    }

    // Memory cleanup automatic with Box
}
