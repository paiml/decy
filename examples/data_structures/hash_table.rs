// Simple hash table with chaining
// Transpiled to safe Rust using Vec and Box

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const TABLE_SIZE: usize = 100;

#[derive(Debug)]
struct Entry {
    key: String,
    value: i32,
    next: Option<Box<Entry>>,
}

struct HashTable {
    buckets: Vec<Option<Box<Entry>>>,
}

impl HashTable {
    fn hash(key: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % TABLE_SIZE
    }

    fn create_table() -> HashTable {
        let mut buckets = Vec::with_capacity(TABLE_SIZE);
        for _ in 0..TABLE_SIZE {
            buckets.push(None);
        }
        HashTable { buckets }
    }

    fn insert(&mut self, key: &str, value: i32) {
        let index = HashTable::hash(key);
        let new_entry = Box::new(Entry {
            key: key.to_string(),
            value,
            next: self.buckets[index].take(),
        });
        self.buckets[index] = Some(new_entry);
    }

    fn get(&self, key: &str) -> Option<i32> {
        let index = HashTable::hash(key);
        let mut entry = &self.buckets[index];

        while let Some(ref node) = entry {
            if node.key == key {
                return Some(node.value);
            }
            entry = &node.next;
        }

        None
    }
}

fn main() {
    let mut table = HashTable::create_table();

    table.insert("apple", 5);
    table.insert("banana", 7);
    table.insert("cherry", 3);

    if let Some(value) = table.get("banana") {
        println!("banana: {}", value);
    }

    if table.get("grape").is_none() {
        println!("grape not found");
    }

    // No explicit free needed - Rust automatically drops the hash table
}
