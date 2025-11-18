/* K&R C Chapter 5: Circular Buffer (Ring Buffer)
 * K&R §5.10: Efficient fixed-size buffer
 * Transpiled to safe Rust (using Vec with modulo indexing)
 */

struct CircularBuffer {
    data: Vec<i32>,
    capacity: usize,
    head: usize,  // Write position
    tail: usize,  // Read position
    count: usize, // Number of elements
}

impl CircularBuffer {
    fn new(capacity: usize) -> Self {
        CircularBuffer {
            data: vec![0; capacity],
            capacity,
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    fn is_full(&self) -> bool {
        self.count == self.capacity
    }

    fn is_empty(&self) -> bool {
        self.count == 0
    }

    fn push(&mut self, value: i32) {
        if self.is_full() {
            println!("  Buffer full, overwriting oldest");
            self.tail = (self.tail + 1) % self.capacity;
        } else {
            self.count += 1;
        }

        self.data[self.head] = value;
        self.head = (self.head + 1) % self.capacity;
    }

    fn pop(&mut self) -> Option<i32> {
        if self.is_empty() {
            return None;
        }

        let value = self.data[self.tail];
        self.tail = (self.tail + 1) % self.capacity;
        self.count -= 1;

        Some(value)
    }

    fn peek(&self) -> Option<i32> {
        if self.is_empty() {
            None
        } else {
            Some(self.data[self.tail])
        }
    }

    fn len(&self) -> usize {
        self.count
    }

    fn print(&self) {
        print!("[");
        for i in 0..self.count {
            let index = (self.tail + i) % self.capacity;
            print!("{}", self.data[index]);
            if i < self.count - 1 {
                print!(", ");
            }
        }
        println!("] (count={}/{})", self.count, self.capacity);
    }
}

fn main() {
    println!("=== Circular Buffer ===\n");

    let mut cb = CircularBuffer::new(5);

    println!("Push 10, 20, 30:");
    cb.push(10);
    cb.push(20);
    cb.push(30);
    cb.print();

    println!("\nPop: {}", cb.pop().unwrap_or(-1));
    cb.print();

    println!("\nPush 40, 50, 60:");
    cb.push(40);
    cb.push(50);
    cb.push(60);
    cb.print();

    println!("\nPush 70 (buffer full, overwrite):");
    cb.push(70);
    cb.print();

    println!("\nCircular buffer automatically freed (RAII)");
}

// Idiomatic alternative: use VecDeque
#[allow(dead_code)]
fn demonstrate_vecdeque() {
    use std::collections::VecDeque;

    let mut buffer: VecDeque<i32> = VecDeque::with_capacity(5);

    // Push/pop from both ends
    buffer.push_back(10);
    buffer.push_back(20);
    buffer.pop_front();

    // Fixed-size ring buffer behavior
    if buffer.len() == buffer.capacity() {
        buffer.pop_front(); // Remove oldest
        buffer.push_back(30); // Add newest
    }

    println!("{:?}", buffer);
}

// Key differences from C:
// 1. Vec<i32> instead of int*
// 2. No manual malloc/free
// 3. RAII: automatic cleanup
// 4. Option<i32> instead of error codes
// 5. Bounds checking automatic
// 6. Modulo operator % for wrapping
// 7. Prefer VecDeque for ring buffer behavior
