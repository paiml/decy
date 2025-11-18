/* K&R C Chapter 5: LRU Cache
 * K&R §5.10: Least Recently Used cache with hash map + doubly-linked list
 * Transpiled to safe Rust (using HashMap + VecDeque)
 */

use std::collections::{HashMap, VecDeque};

struct LRUCache {
    cache: HashMap<i32, i32>,
    order: VecDeque<i32>,
    capacity: usize,
}

impl LRUCache {
    fn new(capacity: usize) -> Self {
        LRUCache {
            cache: HashMap::new(),
            order: VecDeque::new(),
            capacity,
        }
    }

    fn get(&mut self, key: i32) -> Option<i32> {
        if let Some(&value) = self.cache.get(&key) {
            // Move to front (most recently used)
            self.order.retain(|&k| k != key);
            self.order.push_front(key);
            Some(value)
        } else {
            None
        }
    }

    fn put(&mut self, key: i32, value: i32) {
        if self.cache.contains_key(&key) {
            // Update existing key
            self.cache.insert(key, value);
            self.order.retain(|&k| k != key);
            self.order.push_front(key);
        } else {
            // Insert new key
            if self.cache.len() >= self.capacity {
                // Evict LRU
                if let Some(lru_key) = self.order.pop_back() {
                    self.cache.remove(&lru_key);
                    println!("  Evicted key {}", lru_key);
                }
            }

            self.cache.insert(key, value);
            self.order.push_front(key);
        }
    }

    fn print(&self) {
        print!("Cache (MRU -> LRU): [");
        for (i, &key) in self.order.iter().enumerate() {
            if let Some(&value) = self.cache.get(&key) {
                print!("({}:{})", key, value);
                if i < self.order.len() - 1 {
                    print!(", ");
                }
            }
        }
        println!("]");
    }

    fn len(&self) -> usize {
        self.cache.len()
    }
}

fn main() {
    println!("=== LRU Cache ===\n");

    let mut cache = LRUCache::new(3);

    println!("Put(1, 10):");
    cache.put(1, 10);
    cache.print();

    println!("\nPut(2, 20):");
    cache.put(2, 20);
    cache.print();

    println!("\nPut(3, 30):");
    cache.put(3, 30);
    cache.print();

    println!("\nGet(1): {}", cache.get(1).unwrap());
    cache.print();

    println!("\nPut(4, 40) - should evict key 2:");
    cache.put(4, 40);
    cache.print();

    println!("\nGet(2): {} (should be -1, evicted)",
             cache.get(2).unwrap_or(-1));
}

// Production-ready LRU using lru crate
#[allow(dead_code)]
fn demonstrate_lru_crate() {
    // In Cargo.toml: lru = "0.12"
    // use lru::LruCache;
    //
    // let mut cache = LruCache::new(std::num::NonZeroUsize::new(3).unwrap());
    //
    // cache.put(1, 10);
    // cache.put(2, 20);
    //
    // if let Some(&value) = cache.get(&1) {
    //     println!("Found: {}", value);
    // }
}

// Advanced: LRU with custom eviction callback
#[allow(dead_code)]
struct LRUCacheWithCallback<F>
where
    F: FnMut(i32, i32),
{
    cache: HashMap<i32, i32>,
    order: VecDeque<i32>,
    capacity: usize,
    on_evict: F,
}

impl<F> LRUCacheWithCallback<F>
where
    F: FnMut(i32, i32),
{
    fn new(capacity: usize, on_evict: F) -> Self {
        LRUCacheWithCallback {
            cache: HashMap::new(),
            order: VecDeque::new(),
            capacity,
            on_evict,
        }
    }

    fn put(&mut self, key: i32, value: i32) {
        if self.cache.contains_key(&key) {
            self.cache.insert(key, value);
            self.order.retain(|&k| k != key);
            self.order.push_front(key);
        } else {
            if self.cache.len() >= self.capacity {
                if let Some(lru_key) = self.order.pop_back() {
                    if let Some(lru_value) = self.cache.remove(&lru_key) {
                        (self.on_evict)(lru_key, lru_value);
                    }
                }
            }

            self.cache.insert(key, value);
            self.order.push_front(key);
        }
    }
}

// Key differences from C:
// 1. HashMap instead of manual hash table
// 2. VecDeque instead of doubly-linked list
// 3. No manual malloc/free
// 4. RAII: automatic cleanup
// 5. Option<i32> for get operations
// 6. retain() for removing elements
// 7. Simpler implementation than C version
// 8. Consider lru crate for production
