// Binary search tree implementation
// Transpiled to safe Rust using Box<TreeNode> for heap allocation

#[derive(Debug)]
struct TreeNode {
    value: i32,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
}

impl TreeNode {
    fn create_node(value: i32) -> Box<TreeNode> {
        Box::new(TreeNode {
            value,
            left: None,
            right: None,
        })
    }

    fn insert(root: Option<Box<TreeNode>>, value: i32) -> Option<Box<TreeNode>> {
        match root {
            None => Some(TreeNode::create_node(value)),
            Some(mut node) => {
                if value < node.value {
                    node.left = TreeNode::insert(node.left, value);
                } else if value > node.value {
                    node.right = TreeNode::insert(node.right, value);
                }
                Some(node)
            }
        }
    }

    fn search(root: &Option<Box<TreeNode>>, value: i32) -> bool {
        match root {
            None => false,
            Some(node) => {
                if value == node.value {
                    true
                } else if value < node.value {
                    TreeNode::search(&node.left, value)
                } else {
                    TreeNode::search(&node.right, value)
                }
            }
        }
    }
}

fn main() {
    let mut root: Option<Box<TreeNode>> = None;

    root = TreeNode::insert(root, 50);
    root = TreeNode::insert(root, 30);
    root = TreeNode::insert(root, 70);
    root = TreeNode::insert(root, 20);
    root = TreeNode::insert(root, 40);

    let found = TreeNode::search(&root, 40);
    println!("Found 40: {}", found);

    let found = TreeNode::search(&root, 100);
    println!("Found 100: {}", found);

    // No explicit free needed - Rust automatically drops the tree
}
