/* K&R C Chapter 5: Dynamic Array (Vector)
 * K&R §5.10: Dynamic memory allocation
 * Transpiled to safe Rust (using Vec<T> directly)
 */

// Rust's Vec<T> is the idiomatic dynamic array
// This example demonstrates Vec usage + a custom wrapper

struct DynamicArray {
    data: Vec<i32>,
}

impl DynamicArray {
    fn new() -> Self {
        DynamicArray { data: Vec::new() }
    }

    fn with_capacity(capacity: usize) -> Self {
        DynamicArray {
            data: Vec::with_capacity(capacity),
        }
    }

    fn push(&mut self, value: i32) {
        self.data.push(value);
    }

    fn pop(&mut self) -> Option<i32> {
        self.data.pop()
    }

    fn get(&self, index: usize) -> Option<&i32> {
        self.data.get(index)
    }

    fn set(&mut self, index: usize, value: i32) -> bool {
        if index < self.data.len() {
            self.data[index] = value;
            true
        } else {
            false
        }
    }

    fn insert(&mut self, index: usize, value: i32) {
        if index <= self.data.len() {
            self.data.insert(index, value);
        }
    }

    fn remove(&mut self, index: usize) -> Option<i32> {
        if index < self.data.len() {
            Some(self.data.remove(index))
        } else {
            None
        }
    }

    fn clear(&mut self) {
        self.data.clear();
    }

    fn shrink_to_fit(&mut self) {
        let old_capacity = self.data.capacity();
        self.data.shrink_to_fit();
        let new_capacity = self.data.capacity();
        if old_capacity != new_capacity {
            println!("  Array shrunk to capacity {}", new_capacity);
        }
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn capacity(&self) -> usize {
        self.data.capacity()
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
        println!("] (size={}, capacity={})", self.len(), self.capacity());
    }
}

fn main() {
    println!("=== Dynamic Array (Vector) ===\n");

    // Create array with initial capacity
    println!("Creating array with capacity 2:");
    let mut arr = DynamicArray::with_capacity(2);
    arr.print();
    println!();

    // Push elements (automatic growth)
    println!("Pushing elements:");
    for i in 1..=5 {
        print!("  Push {}: ", i * 10);
        arr.push(i * 10);
        arr.print();
        if arr.len() > arr.capacity() / 2 && arr.len() == arr.capacity() {
            // Growth happened in previous push
        }
    }
    println!();

    // Pop element
    if let Some(value) = arr.pop() {
        println!("Pop element: {}", value);
    }
    arr.print();
    println!();

    // Get and set
    if let Some(&value) = arr.get(2) {
        println!("Get arr[2]: {}", value);
    }
    println!("Set arr[2] = 99:");
    arr.set(2, 99);
    arr.print();
    println!();

    // Insert
    println!("Insert 55 at index 1:");
    arr.insert(1, 55);
    arr.print();
    println!();

    // Remove
    println!("Remove element at index 2:");
    if let Some(value) = arr.remove(2) {
        println!("  Removed: {}", value);
    }
    arr.print();
    println!();

    // Shrink to fit
    println!("Shrink to fit:");
    arr.shrink_to_fit();
    arr.print();
    println!();

    // Clear
    println!("Clear array:");
    arr.clear();
    arr.print();

    println!("\nDynamic array characteristics:");
    println!("  - O(1) amortized push");
    println!("  - O(1) random access");
    println!("  - O(n) insert/remove (middle)");
    println!("  - Automatic growth (no manual realloc)");
}

// Direct Vec usage (most idiomatic)
#[allow(dead_code)]
fn demonstrate_vec_directly() {
    let mut v: Vec<i32> = Vec::new();

    // Push/pop
    v.push(10);
    v.push(20);
    v.push(30);

    if let Some(last) = v.pop() {
        println!("Popped: {}", last);
    }

    // Indexing
    v[0] = 100;
    println!("First: {}", v[0]);

    // Safe indexing
    if let Some(&value) = v.get(1) {
        println!("Second: {}", value);
    }

    // Insert/remove
    v.insert(1, 15);
    v.remove(0);

    // Iteration
    for &value in &v {
        println!("{}", value);
    }

    // Capacity management
    v.reserve(100);
    v.shrink_to_fit();

    println!("Length: {}, Capacity: {}", v.len(), v.capacity());
}

// Advanced Vec features
#[allow(dead_code)]
fn advanced_vec_features() {
    // Initialization
    let v1 = vec![1, 2, 3, 4, 5];
    let v2 = vec![0; 10];  // 10 zeros

    // Slicing
    let slice = &v1[1..4];
    println!("{:?}", slice);

    // Extend
    let mut v3 = vec![1, 2];
    v3.extend(&[3, 4, 5]);

    // Retain (filter in place)
    v3.retain(|&x| x % 2 == 0);

    // Dedup
    let mut v4 = vec![1, 1, 2, 2, 3, 3];
    v4.dedup();

    // Sorting
    let mut v5 = vec![5, 2, 8, 1, 9];
    v5.sort();
    v5.reverse();

    // Binary search (on sorted vec)
    if let Ok(index) = v5.binary_search(&5) {
        println!("Found at index {}", index);
    }
}

// Key differences from C:
// 1. Vec<T> instead of custom realloc-based array
// 2. No manual memory management
// 3. RAII: automatic cleanup
// 4. Growth strategy handled by Vec
// 5. Bounds checking automatic
// 6. Option<T> for fallible operations
// 7. Iterators for safe traversal
// 8. Rich API: extend, retain, dedup, sort, etc.
