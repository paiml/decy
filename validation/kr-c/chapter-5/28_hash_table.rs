/* K&R C Chapter 5: Pointer-Based Hash Table
 * Simple hash table with chaining
 * Transpiled to safe Rust (using Vec and String)
 */

const TABLE_SIZE: usize = 10;

struct Entry {
    key: String,
    value: i32,
}

struct HashTable {
    buckets: Vec<Vec<Entry>>,
    size: usize,
}

impl HashTable {
    fn new() -> Self {
        let mut buckets = Vec::with_capacity(TABLE_SIZE);
        for _ in 0..TABLE_SIZE {
            buckets.push(Vec::new());
        }

        HashTable { buckets, size: 0 }
    }

    fn hash(key: &str) -> usize {
        let mut hash: u32 = 0;
        for ch in key.chars() {
            hash = hash.wrapping_mul(31).wrapping_add(ch as u32);
        }
        (hash as usize) % TABLE_SIZE
    }

    fn insert(&mut self, key: &str, value: i32) {
        let index = Self::hash(key);
        let bucket = &mut self.buckets[index];

        // Check if key exists (update)
        for entry in bucket.iter_mut() {
            if entry.key == key {
                entry.value = value;
                return;
            }
        }

        // Create new entry
        bucket.push(Entry {
            key: key.to_string(),
            value,
        });
        self.size += 1;
    }

    fn get(&self, key: &str) -> Option<i32> {
        let index = Self::hash(key);
        let bucket = &self.buckets[index];

        for entry in bucket {
            if entry.key == key {
                return Some(entry.value);
            }
        }

        None
    }

    fn remove(&mut self, key: &str) -> Option<i32> {
        let index = Self::hash(key);
        let bucket = &mut self.buckets[index];

        if let Some(pos) = bucket.iter().position(|entry| entry.key == key) {
            let entry = bucket.remove(pos);
            self.size -= 1;
            Some(entry.value)
        } else {
            None
        }
    }

    fn print(&self) {
        println!("Hash Table [{} entries]:", self.size);
        for (i, bucket) in self.buckets.iter().enumerate() {
            if !bucket.is_empty() {
                print!("  Bucket {}: ", i);
                for entry in bucket {
                    print!("({}: {}) ", entry.key, entry.value);
                }
                println!();
            }
        }
    }

    fn len(&self) -> usize {
        self.size
    }

    fn is_empty(&self) -> bool {
        self.size == 0
    }
}

fn main() {
    let mut ht = HashTable::new();

    println!("=== Hash Table Demo ===\n");

    // Insert entries
    println!("Inserting entries...");
    ht.insert("apple", 100);
    ht.insert("banana", 200);
    ht.insert("cherry", 300);
    ht.insert("date", 400);
    ht.insert("elderberry", 500);
    ht.insert("fig", 600);

    ht.print();

    // Lookup entries
    println!("\nLookup:");
    let keys = ["apple", "banana", "grape"];
    for key in &keys {
        match ht.get(key) {
            Some(value) => println!("  {} = {}", key, value),
            None => println!("  {} not found", key),
        }
    }

    // Update entry
    println!("\nUpdating 'apple' to 999...");
    ht.insert("apple", 999);

    if let Some(value) = ht.get("apple") {
        println!("  apple = {}", value);
    }

    // Show collisions
    println!("\nHash distribution:");
    for (i, bucket) in ht.buckets.iter().enumerate() {
        let count = bucket.len();
        if count > 0 {
            println!("  Bucket {}: {} entries", i, count);
        }
    }

    // Remove entry
    println!("\nRemoving 'banana'...");
    if let Some(value) = ht.remove("banana") {
        println!("  Removed: banana = {}", value);
    }
    ht.print();

    // Cleanup (automatic via Drop)
    println!("\nHash table will be automatically freed (RAII)");
}

// Idiomatic alternative: use HashMap
#[allow(dead_code)]
fn demonstrate_hashmap() {
    use std::collections::HashMap;

    let mut map = HashMap::new();

    // Insert
    map.insert("apple", 100);
    map.insert("banana", 200);

    // Get
    if let Some(&value) = map.get("apple") {
        println!("apple = {}", value);
    }

    // Update
    map.insert("apple", 999);

    // Remove
    map.remove("banana");

    // Iteration
    for (key, value) in &map {
        println!("{}: {}", key, value);
    }

    // Entry API for insert-or-update
    map.entry("cherry").or_insert(300);
    *map.entry("apple").or_insert(0) += 1;
}

// Key differences from C:
// 1. Vec<Vec<Entry>> instead of Entry*[]
// 2. String instead of char*
// 3. No manual malloc/free/strdup
// 4. RAII: automatic cleanup
// 5. Option<i32> instead of pointer + return code
// 6. to_string() instead of strdup()
// 7. wrapping_mul/wrapping_add for overflow safety
// 8. Prefer HashMap from std::collections
// 9. Entry API for advanced operations
