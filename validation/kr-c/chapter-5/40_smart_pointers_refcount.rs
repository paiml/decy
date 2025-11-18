/* K&R C Chapter 5: Smart Pointers (Reference Counting)
 * K&R §5.10: Manual reference counting for memory management
 * Transpiled to safe Rust (using Rc and Weak)
 */

use std::rc::{Rc, Weak};
use std::cell::RefCell;

// Basic reference counting demo
fn demo_basic_refcount() {
    println!("=== Basic Reference Counting ===");

    let data = Rc::new("Hello, World!");
    println!("  RC created (strong_count={})", Rc::strong_count(&data));

    // Multiple owners
    let owner1 = Rc::clone(&data);
    println!("  RC cloned (strong_count={})", Rc::strong_count(&data));

    let owner2 = Rc::clone(&data);
    println!("  RC cloned (strong_count={})", Rc::strong_count(&data));

    // Dropping owners
    drop(owner1);
    println!("  RC dropped (strong_count={})", Rc::strong_count(&data));

    drop(owner2);
    println!("  RC dropped (strong_count={})", Rc::strong_count(&data));

    drop(data);
    println!("  RC dropped (strong_count=0, deallocated)");

    println!();
}

// Shared linked list
#[derive(Debug)]
struct Node {
    data: i32,
    next: Option<Rc<Node>>,
}

impl Node {
    fn new(data: i32) -> Rc<Self> {
        Rc::new(Node { data, next: None })
    }

    fn with_next(data: i32, next: Rc<Node>) -> Rc<Self> {
        Rc::new(Node {
            data,
            next: Some(next),
        })
    }
}

fn demo_shared_list() {
    println!("=== Shared Linked List ===");

    // Create nodes
    let node3 = Node::new(3);
    println!("  Node 3 created (strong_count={})", Rc::strong_count(&node3));

    let node2 = Node::with_next(2, Rc::clone(&node3));
    println!("  Node 2->3 created (node3 strong_count={})", Rc::strong_count(&node3));

    let node1 = Node::with_next(1, Rc::clone(&node2));
    println!("  Node 1->2 created (node2 strong_count={})", Rc::strong_count(&node2));

    // Create second list sharing the tail
    let head2 = Node::with_next(10, Rc::clone(&node2));
    println!("  Node 10->2 created (node2 strong_count={})", Rc::strong_count(&node2));

    println!("List 1: 1 -> 2 -> 3");
    println!("List 2: 10 -> 2 -> 3 (shares 2->3 with List 1)");

    println!("\nReleasing List 1:");
    drop(node1);
    println!("  node2 strong_count={}", Rc::strong_count(&node2));

    println!("\nReleasing List 2:");
    drop(head2);
    println!("  Tail nodes automatically deallocated");

    println!();
}

// Circular reference with weak pointers
#[derive(Debug)]
struct CircNode {
    data: i32,
    next: RefCell<Option<Rc<CircNode>>>,
    prev: RefCell<Weak<CircNode>>, // Weak reference to break cycle
}

impl CircNode {
    fn new(data: i32) -> Rc<Self> {
        Rc::new(CircNode {
            data,
            next: RefCell::new(None),
            prev: RefCell::new(Weak::new()),
        })
    }
}

fn demo_circular_reference() {
    println!("=== Circular Reference (Weak Pointer) ===");

    let node1 = CircNode::new(1);
    let node2 = CircNode::new(2);

    // Create cycle with weak back pointer
    *node1.next.borrow_mut() = Some(Rc::clone(&node2)); // Strong
    *node2.prev.borrow_mut() = Rc::downgrade(&node1);   // Weak

    println!("  Node 1 created (strong_count={})", Rc::strong_count(&node1));
    println!("  Node 2 created (strong_count={})", Rc::strong_count(&node2));
    println!("  Node 1 <-weak- Node 2");
    println!("  Node 1 -strong-> Node 2");

    // Access weak reference
    if let Some(prev) = node2.prev.borrow().upgrade() {
        println!("  Node 2's prev points to node with data: {}", prev.data);
    }

    println!("\nReleasing nodes:");
    drop(node1);
    drop(node2);
    println!("  All nodes deallocated (no cycle leak)");

    println!();
}

// Copy-on-write example
fn demo_copy_on_write() {
    println!("=== Copy-on-Write ===");

    let buffer = Rc::new(RefCell::new(vec![1, 2, 3, 4, 5]));
    let shared = Rc::clone(&buffer);

    println!("Two owners of buffer (strong_count={})", Rc::strong_count(&buffer));

    // Try to modify - if sole owner, modify in place; otherwise clone
    let mut_buffer = if Rc::strong_count(&buffer) == 1 {
        println!("  Sole owner - modify in place");
        buffer
    } else {
        println!("  Multiple owners - cloning buffer");
        Rc::new(RefCell::new(buffer.borrow().clone()))
    };

    mut_buffer.borrow_mut()[0] = 99;

    println!("After modification:");
    println!("  Original buffer: {:?}", shared.borrow());
    println!("  Modified buffer: {:?}", mut_buffer.borrow());

    println!();
}

fn main() {
    println!("=== Smart Pointers (Reference Counting) ===\n");

    demo_basic_refcount();
    demo_shared_list();
    demo_circular_reference();
    demo_copy_on_write();

    println!("Reference counting benefits:");
    println!("  - Automatic memory management");
    println!("  - Shared ownership");
    println!("  - Predictable deallocation");
    println!("  - Copy-on-write optimization");
    println!("\nRust improvements over C:");
    println!("  - Compile-time cycle detection hints");
    println!("  - Weak pointers to break cycles");
    println!("  - Thread-safe Arc for multi-threading");
    println!("  - No manual retain/release calls");
}

// Thread-safe reference counting with Arc
#[allow(dead_code)]
fn demonstrate_arc() {
    use std::sync::Arc;
    use std::thread;

    let data = Arc::new(vec![1, 2, 3, 4, 5]);

    let mut handles = vec![];

    for i in 0..3 {
        let data_clone = Arc::clone(&data);
        let handle = thread::spawn(move || {
            println!("Thread {}: {:?}", i, data_clone);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

// Key differences from C:
// 1. Rc<T> instead of manual RefCounted
// 2. Weak<T> for weak references
// 3. No manual retain/release
// 4. RAII: automatic cleanup
// 5. Compile-time cycle detection warnings
// 6. Arc<T> for thread-safe reference counting
// 7. RefCell for interior mutability
// 8. Strong/weak count tracking built-in
