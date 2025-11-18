/* K&R C Chapter 5: Skip List
 * K&R §5.10: Probabilistic data structure
 * Transpiled to safe Rust (using Vec<Option<Box<Node>>>)
 */

use rand::Rng;

const MAX_LEVEL: usize = 4;

struct SkipNode {
    key: i32,
    value: i32,
    forward: Vec<Option<Box<SkipNode>>>,
}

impl SkipNode {
    fn new(key: i32, value: i32, level: usize) -> Self {
        SkipNode {
            key,
            value,
            forward: vec![None; level + 1],
        }
    }
}

struct SkipList {
    level: usize,
    header: Box<SkipNode>,
}

impl SkipList {
    fn new() -> Self {
        SkipList {
            level: 0,
            header: Box::new(SkipNode::new(-1, 0, MAX_LEVEL)),
        }
    }

    fn random_level() -> usize {
        let mut rng = rand::thread_rng();
        let mut level = 0;
        while rng.gen::<bool>() && level < MAX_LEVEL {
            level += 1;
        }
        level
    }

    fn insert(&mut self, key: i32, value: i32) {
        let mut update: Vec<*mut SkipNode> = vec![std::ptr::null_mut(); MAX_LEVEL + 1];
        let mut current = &mut *self.header as *mut SkipNode;

        unsafe {
            for i in (0..=self.level).rev() {
                while let Some(ref next) = (*current).forward[i] {
                    if next.key < key {
                        current = (*current).forward[i].as_mut().unwrap().as_mut() as *mut SkipNode;
                    } else {
                        break;
                    }
                }
                update[i] = current;
            }
        }

        let level = Self::random_level();
        if level > self.level {
            for i in (self.level + 1)..=level {
                update[i] = &mut *self.header as *mut SkipNode;
            }
            self.level = level;
        }

        let mut new_node = Box::new(SkipNode::new(key, value, level));

        unsafe {
            for i in 0..=level {
                new_node.forward[i] = (*update[i]).forward[i].take();
                (*update[i]).forward[i] = Some(new_node);
                new_node = (*update[i]).forward[i].take().unwrap();
            }
            (*update[0]).forward[0] = Some(new_node);
        }
    }

    fn search(&self, key: i32) -> Option<i32> {
        let mut current = &*self.header;

        for i in (0..=self.level).rev() {
            while let Some(ref next) = current.forward[i] {
                if next.key < key {
                    current = next;
                } else {
                    break;
                }
            }
        }

        current.forward[0].as_ref().and_then(|node| {
            if node.key == key {
                Some(node.value)
            } else {
                None
            }
        })
    }

    fn print(&self) {
        println!("Skip List:");
        for i in (0..=self.level).rev() {
            print!("Level {}: ", i);
            let mut current = &self.header.forward[i];
            while let Some(ref node) = current {
                print!("{}->", node.key);
                current = &node.forward[i];
            }
            println!("NULL");
        }
    }
}

fn main() {
    println!("=== Skip List ===\n");

    let mut list = SkipList::new();

    println!("Insert: 3, 6, 7, 9, 12, 19, 17");
    list.insert(3, 30);
    list.insert(6, 60);
    list.insert(7, 70);
    list.insert(9, 90);
    list.insert(12, 120);
    list.insert(19, 190);
    list.insert(17, 170);

    list.print();

    println!("\nSearch 7: {}",
             if let Some(value) = list.search(7) {
                 format!("Found (value={})", value)
             } else {
                 "Not found".to_string()
             });

    println!("Search 15: {}",
             if list.search(15).is_some() {
                 "Found"
             } else {
                 "Not found"
             });
}

// Safe alternative: use BTreeMap
#[allow(dead_code)]
fn demonstrate_btreemap() {
    use std::collections::BTreeMap;

    let mut map = BTreeMap::new();

    // Insert (O(log n))
    map.insert(3, 30);
    map.insert(6, 60);
    map.insert(7, 70);

    // Search (O(log n))
    if let Some(&value) = map.get(&7) {
        println!("Found: {}", value);
    }

    // Ordered iteration
    for (key, value) in &map {
        println!("{}: {}", key, value);
    }
}

// Key differences from C:
// 1. Vec<Option<Box<SkipNode>>> instead of SkipNode**
// 2. Box for heap allocation
// 3. Option for nullable pointers
// 4. RAII: automatic cleanup
// 5. Unsafe block for mutable pointer manipulation
// 6. rand crate for random numbers
// 7. Prefer BTreeMap for production (guaranteed O(log n))
