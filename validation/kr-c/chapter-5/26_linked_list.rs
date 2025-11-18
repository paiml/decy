/* K&R C Chapter 5: Pointer-Based Linked List
 * Simple singly-linked list implementation
 * Transpiled to safe Rust (using Box<Node> and Option)
 */

type Link = Option<Box<Node>>;

struct Node {
    data: i32,
    next: Link,
}

struct List {
    head: Link,
    size: usize,
}

impl List {
    fn new() -> Self {
        List {
            head: None,
            size: 0,
        }
    }

    fn push_front(&mut self, value: i32) {
        let new_node = Box::new(Node {
            data: value,
            next: self.head.take(),
        });
        self.head = Some(new_node);
        self.size += 1;
    }

    fn push_back(&mut self, value: i32) {
        let new_node = Box::new(Node {
            data: value,
            next: None,
        });

        if self.head.is_none() {
            self.head = Some(new_node);
        } else {
            let mut current = self.head.as_mut().unwrap();
            while current.next.is_some() {
                current = current.next.as_mut().unwrap();
            }
            current.next = Some(new_node);
        }
        self.size += 1;
    }

    fn pop_front(&mut self) -> Option<i32> {
        self.head.take().map(|node| {
            self.head = node.next;
            self.size -= 1;
            node.data
        })
    }

    fn print(&self) {
        print!("List [{}]: ", self.size);
        let mut current = &self.head;
        while let Some(node) = current {
            print!("{} ", node.data);
            current = &node.next;
        }
        println!();
    }

    fn find(&self, value: i32) -> Option<usize> {
        let mut current = &self.head;
        let mut index = 0;

        while let Some(node) = current {
            if node.data == value {
                return Some(index);
            }
            current = &node.next;
            index += 1;
        }

        None
    }

    fn reverse(&mut self) {
        let mut prev: Link = None;
        let mut current = self.head.take();

        while let Some(mut node) = current {
            let next = node.next.take();
            node.next = prev;
            prev = Some(node);
            current = next;
        }

        self.head = prev;
    }

    fn len(&self) -> usize {
        self.size
    }
}

// RAII: Drop automatically frees all nodes
impl Drop for List {
    fn drop(&mut self) {
        let mut current = self.head.take();
        while let Some(mut node) = current {
            current = node.next.take();
        }
    }
}

fn main() {
    let mut list = List::new();

    println!("=== Linked List Demo ===\n");

    // Push elements
    println!("Pushing to front: 3, 2, 1");
    list.push_front(3);
    list.push_front(2);
    list.push_front(1);
    list.print();

    println!("\nPushing to back: 4, 5, 6");
    list.push_back(4);
    list.push_back(5);
    list.push_back(6);
    list.print();

    // Find elements
    println!("\nSearching for 5: ");
    if let Some(index) = list.find(5) {
        println!("found at index {}", index);
    } else {
        println!("not found");
    }

    println!("Searching for 99: ");
    if let Some(index) = list.find(99) {
        println!("found at index {}", index);
    } else {
        println!("not found");
    }

    // Reverse list
    println!("\nReversing list...");
    list.reverse();
    list.print();

    // Pop elements
    println!("\nPopping from front:");
    for _ in 0..3 {
        if let Some(value) = list.pop_front() {
            println!("  Popped: {}", value);
            list.print();
        }
    }

    // Cleanup (automatic via Drop)
    println!("\nList will be automatically freed (RAII)");
    drop(list);
    println!("List freed");
}

// Idiomatic alternative: use Vec<i32> or VecDeque<i32>
#[allow(dead_code)]
fn demonstrate_vec_alternative() {
    let mut list = vec![1, 2, 3, 4, 5, 6];

    // Push/pop operations
    list.insert(0, 0);  // push_front
    list.push(7);       // push_back
    list.remove(0);     // pop_front

    // Find
    if let Some(index) = list.iter().position(|&x| x == 5) {
        println!("Found at index {}", index);
    }

    // Reverse
    list.reverse();

    println!("{:?}", list);
}

// Key differences from C:
// 1. Option<Box<Node>> instead of Node*
// 2. No manual malloc/free - Box handles allocation
// 3. RAII: Drop trait for automatic cleanup
// 4. Pattern matching with if let, while let
// 5. take() for moving ownership from Option
// 6. No NULL - use None
// 7. Bounds checking automatic
// 8. Prefer Vec<T> for most use cases
