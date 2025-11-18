/* K&R C Chapter 5: Doubly-Linked List
 * K&R §6.5: Pointer-based data structures
 * Transpiled to safe Rust (using VecDeque as idiomatic alternative)
 */

use std::collections::VecDeque;

// Note: Doubly-linked lists with prev/next pointers are challenging in safe Rust
// due to circular references. VecDeque is the idiomatic replacement.
// This example shows both approaches.

// ============================================================================
// Approach 1: VecDeque (Idiomatic Rust)
// ============================================================================

struct DoublyLinkedList {
    data: VecDeque<i32>,
}

impl DoublyLinkedList {
    fn new() -> Self {
        DoublyLinkedList {
            data: VecDeque::new(),
        }
    }

    fn push_front(&mut self, value: i32) {
        self.data.push_front(value);
    }

    fn push_back(&mut self, value: i32) {
        self.data.push_back(value);
    }

    fn pop_front(&mut self) -> Option<i32> {
        self.data.pop_front()
    }

    fn pop_back(&mut self) -> Option<i32> {
        self.data.pop_back()
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn print_forward(&self) {
        print!("[");
        for (i, &value) in self.data.iter().enumerate() {
            print!("{}", value);
            if i < self.data.len() - 1 {
                print!(" <-> ");
            }
        }
        println!("]");
    }

    fn print_backward(&self) {
        print!("[");
        for (i, &value) in self.data.iter().rev().enumerate() {
            print!("{}", value);
            if i < self.data.len() - 1 {
                print!(" <-> ");
            }
        }
        println!("]");
    }
}

fn main() {
    println!("=== Doubly-Linked List ===\n");

    let mut list = DoublyLinkedList::new();

    println!("Push back: 10, 20, 30");
    list.push_back(10);
    list.push_back(20);
    list.push_back(30);
    print!("Forward:  ");
    list.print_forward();
    print!("Backward: ");
    list.print_backward();
    println!();

    println!("Push front: 5");
    list.push_front(5);
    print!("Forward:  ");
    list.print_forward();
    println!();

    if let Some(value) = list.pop_front() {
        println!("Pop front: {}", value);
    }
    if let Some(value) = list.pop_back() {
        println!("Pop back:  {}", value);
    }
    print!("Forward:  ");
    list.print_forward();

    println!("\nList will be automatically freed (RAII)");
}

// ============================================================================
// Approach 2: Unsafe doubly-linked list (for educational purposes)
// ============================================================================
// Note: This requires unsafe code and is NOT recommended for production.
// Shown here to demonstrate the challenge of doubly-linked lists in Rust.

#[allow(dead_code)]
mod unsafe_dll {
    use std::ptr::NonNull;

    struct Node {
        data: i32,
        prev: Option<NonNull<Node>>,
        next: Option<NonNull<Node>>,
    }

    pub struct UnsafeDoublyLinkedList {
        head: Option<NonNull<Node>>,
        tail: Option<NonNull<Node>>,
        size: usize,
    }

    impl UnsafeDoublyLinkedList {
        pub fn new() -> Self {
            UnsafeDoublyLinkedList {
                head: None,
                tail: None,
                size: 0,
            }
        }

        pub fn push_front(&mut self, data: i32) {
            let mut node = Box::new(Node {
                data,
                prev: None,
                next: self.head,
            });

            let node_ptr = NonNull::new(Box::into_raw(node));

            match self.head {
                Some(mut head) => unsafe {
                    head.as_mut().prev = node_ptr;
                },
                None => self.tail = node_ptr,
            }

            self.head = node_ptr;
            self.size += 1;
        }

        pub fn len(&self) -> usize {
            self.size
        }
    }

    impl Drop for UnsafeDoublyLinkedList {
        fn drop(&mut self) {
            let mut current = self.head;
            while let Some(node_ptr) = current {
                unsafe {
                    let node = Box::from_raw(node_ptr.as_ptr());
                    current = node.next;
                }
            }
        }
    }
}

// ============================================================================
// Approach 3: Index-based doubly-linked list (safe, using Vec)
// ============================================================================

#[allow(dead_code)]
mod index_based {
    struct Node {
        data: i32,
        prev: Option<usize>,
        next: Option<usize>,
    }

    pub struct IndexBasedDLL {
        nodes: Vec<Node>,
        head: Option<usize>,
        tail: Option<usize>,
    }

    impl IndexBasedDLL {
        pub fn new() -> Self {
            IndexBasedDLL {
                nodes: Vec::new(),
                head: None,
                tail: None,
            }
        }

        pub fn push_back(&mut self, data: i32) {
            let new_index = self.nodes.len();
            let node = Node {
                data,
                prev: self.tail,
                next: None,
            };

            self.nodes.push(node);

            if let Some(old_tail) = self.tail {
                self.nodes[old_tail].next = Some(new_index);
            } else {
                self.head = Some(new_index);
            }

            self.tail = Some(new_index);
        }

        pub fn print_forward(&self) {
            print!("[");
            let mut current = self.head;
            while let Some(index) = current {
                print!("{}", self.nodes[index].data);
                current = self.nodes[index].next;
                if current.is_some() {
                    print!(" <-> ");
                }
            }
            println!("]");
        }
    }
}

// Demonstrate VecDeque features
#[allow(dead_code)]
fn demonstrate_vecdeque() {
    let mut deque = VecDeque::new();

    // Push to both ends
    deque.push_back(2);
    deque.push_back(3);
    deque.push_front(1);
    deque.push_back(4);

    // Pop from both ends
    deque.pop_front();
    deque.pop_back();

    // Indexing (O(1))
    if let Some(&value) = deque.get(0) {
        println!("First: {}", value);
    }

    // Iteration
    for &value in &deque {
        println!("{}", value);
    }

    // Rotate
    deque.rotate_left(1);
    deque.rotate_right(1);

    // Convert to/from Vec
    let v: Vec<i32> = deque.iter().copied().collect();
    let deque2: VecDeque<i32> = v.into_iter().collect();
}

// Key differences from C:
// 1. VecDeque instead of manual doubly-linked list
// 2. O(1) operations at both ends
// 3. Random access in O(1) time
// 4. No unsafe pointer manipulation
// 5. RAII: automatic cleanup
// 6. Doubly-linked lists rare in Rust (VecDeque preferred)
// 7. If needed: use indices instead of pointers
// 8. std::collections::LinkedList exists but rarely used
