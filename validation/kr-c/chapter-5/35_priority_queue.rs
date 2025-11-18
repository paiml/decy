/* K&R C Chapter 5: Priority Queue (Binary Heap)
 * K&R §5.10: Heap-based priority queue
 * Transpiled to safe Rust (using Vec with heap property)
 */

struct PriorityQueue {
    data: Vec<i32>,
}

impl PriorityQueue {
    fn new() -> Self {
        PriorityQueue { data: Vec::new() }
    }

    fn with_capacity(capacity: usize) -> Self {
        PriorityQueue {
            data: Vec::with_capacity(capacity),
        }
    }

    fn parent(i: usize) -> usize {
        (i - 1) / 2
    }

    fn left(i: usize) -> usize {
        2 * i + 1
    }

    fn right(i: usize) -> usize {
        2 * i + 2
    }

    fn bubble_up(&mut self, mut index: usize) {
        while index > 0 && self.data[Self::parent(index)] > self.data[index] {
            let parent = Self::parent(index);
            self.data.swap(index, parent);
            index = parent;
        }
    }

    fn bubble_down(&mut self, mut index: usize) {
        loop {
            let mut min_index = index;
            let left = Self::left(index);
            let right = Self::right(index);

            if left < self.data.len() && self.data[left] < self.data[min_index] {
                min_index = left;
            }

            if right < self.data.len() && self.data[right] < self.data[min_index] {
                min_index = right;
            }

            if min_index == index {
                break;
            }

            self.data.swap(index, min_index);
            index = min_index;
        }
    }

    fn insert(&mut self, value: i32) {
        self.data.push(value);
        self.bubble_up(self.data.len() - 1);
    }

    fn extract_min(&mut self) -> Option<i32> {
        if self.data.is_empty() {
            return None;
        }

        let min = self.data[0];
        let last = self.data.pop().unwrap();

        if !self.data.is_empty() {
            self.data[0] = last;
            self.bubble_down(0);
        }

        Some(min)
    }

    fn peek(&self) -> Option<i32> {
        self.data.first().copied()
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn print(&self) {
        print!("[");
        for (i, &value) in self.data.iter().enumerate() {
            print!("{}", value);
            if i < self.data.len() - 1 {
                print!(", ");
            }
        }
        println!("] (size={})", self.len());
    }
}

fn main() {
    println!("=== Priority Queue (Min-Heap) ===\n");

    let mut pq = PriorityQueue::with_capacity(10);

    println!("Insert: 15, 10, 20, 8, 12, 25, 6");
    pq.insert(15);
    pq.insert(10);
    pq.insert(20);
    pq.insert(8);
    pq.insert(12);
    pq.insert(25);
    pq.insert(6);

    print!("Heap: ");
    pq.print();

    println!("\nPeek min: {}", pq.peek().unwrap());

    print!("\nExtract min: ");
    while let Some(value) = pq.extract_min() {
        print!("{} ", value);
    }
    println!();
}

// Idiomatic alternative: use BinaryHeap
#[allow(dead_code)]
fn demonstrate_binary_heap() {
    use std::collections::BinaryHeap;
    use std::cmp::Reverse;

    // BinaryHeap is max-heap by default
    let mut max_heap = BinaryHeap::new();
    max_heap.push(10);
    max_heap.push(20);
    max_heap.push(5);

    println!("Max: {}", max_heap.peek().unwrap());

    // For min-heap, use Reverse
    let mut min_heap = BinaryHeap::new();
    min_heap.push(Reverse(10));
    min_heap.push(Reverse(20));
    min_heap.push(Reverse(5));

    if let Some(Reverse(min)) = min_heap.peek() {
        println!("Min: {}", min);
    }

    // Extract all in sorted order
    while let Some(Reverse(value)) = min_heap.pop() {
        println!("{}", value);
    }
}

// Generic priority queue
#[allow(dead_code)]
struct GenericPriorityQueue<T: Ord> {
    data: Vec<T>,
}

impl<T: Ord> GenericPriorityQueue<T> {
    fn new() -> Self {
        GenericPriorityQueue { data: Vec::new() }
    }

    fn insert(&mut self, value: T) {
        self.data.push(value);
        self.bubble_up(self.data.len() - 1);
    }

    fn bubble_up(&mut self, mut index: usize) {
        while index > 0 {
            let parent = (index - 1) / 2;
            if self.data[parent] <= self.data[index] {
                break;
            }
            self.data.swap(index, parent);
            index = parent;
        }
    }

    fn extract_min(&mut self) -> Option<T> {
        if self.data.is_empty() {
            return None;
        }

        Some(self.data.swap_remove(0))
    }
}

// Key differences from C:
// 1. Vec<i32> instead of int*
// 2. No manual malloc/free
// 3. RAII: automatic cleanup
// 4. Option<i32> for extract/peek
// 5. swap() method instead of manual swap
// 6. Bounds checking automatic
// 7. Prefer std::collections::BinaryHeap
// 8. Generic version with Ord trait
