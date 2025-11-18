/* K&R C Chapter 5: Trie (Prefix Tree)
 * K&R §5.10: String search tree
 * Transpiled to safe Rust (using Box and Vec)
 */

const ALPHABET_SIZE: usize = 26;

struct TrieNode {
    children: Vec<Option<Box<TrieNode>>>,
    is_end_of_word: bool,
}

impl TrieNode {
    fn new() -> Self {
        TrieNode {
            children: vec![None; ALPHABET_SIZE],
            is_end_of_word: false,
        }
    }
}

struct Trie {
    root: Box<TrieNode>,
}

impl Trie {
    fn new() -> Self {
        Trie {
            root: Box::new(TrieNode::new()),
        }
    }

    fn insert(&mut self, word: &str) {
        let mut current = &mut *self.root;

        for ch in word.chars() {
            let index = (ch as usize) - ('a' as usize);
            if index >= ALPHABET_SIZE {
                continue; // Skip non-lowercase letters
            }

            current = current.children[index]
                .get_or_insert_with(|| Box::new(TrieNode::new()));
        }

        current.is_end_of_word = true;
    }

    fn search(&self, word: &str) -> bool {
        let mut current = &*self.root;

        for ch in word.chars() {
            let index = (ch as usize) - ('a' as usize);
            if index >= ALPHABET_SIZE {
                return false;
            }

            match &current.children[index] {
                Some(node) => current = node,
                None => return false,
            }
        }

        current.is_end_of_word
    }

    fn starts_with(&self, prefix: &str) -> bool {
        let mut current = &*self.root;

        for ch in prefix.chars() {
            let index = (ch as usize) - ('a' as usize);
            if index >= ALPHABET_SIZE {
                return false;
            }

            match &current.children[index] {
                Some(node) => current = node,
                None => return false,
            }
        }

        true
    }

    fn print_all(&self) {
        let mut prefix = String::new();
        self.print_helper(&*self.root, &mut prefix);
    }

    fn print_helper(&self, node: &TrieNode, prefix: &mut String) {
        if node.is_end_of_word {
            println!("  {}", prefix);
        }

        for i in 0..ALPHABET_SIZE {
            if let Some(ref child) = node.children[i] {
                prefix.push((b'a' + i as u8) as char);
                self.print_helper(child, prefix);
                prefix.pop();
            }
        }
    }

    fn collect_words(&self) -> Vec<String> {
        let mut words = Vec::new();
        let mut prefix = String::new();
        self.collect_helper(&*self.root, &mut prefix, &mut words);
        words
    }

    fn collect_helper(&self, node: &TrieNode, prefix: &mut String, words: &mut Vec<String>) {
        if node.is_end_of_word {
            words.push(prefix.clone());
        }

        for i in 0..ALPHABET_SIZE {
            if let Some(ref child) = node.children[i] {
                prefix.push((b'a' + i as u8) as char);
                self.collect_helper(child, prefix, words);
                prefix.pop();
            }
        }
    }
}

fn main() {
    println!("=== Trie (Prefix Tree) ===\n");

    let mut trie = Trie::new();

    println!("Insert: cat, car, card, care, careful, dog, dodge, door");
    trie.insert("cat");
    trie.insert("car");
    trie.insert("card");
    trie.insert("care");
    trie.insert("careful");
    trie.insert("dog");
    trie.insert("dodge");
    trie.insert("door");

    println!("\nAll words:");
    trie.print_all();

    println!("\nSearch 'car': {}", if trie.search("car") { "Found" } else { "Not found" });
    println!("Search 'can': {}", if trie.search("can") { "Found" } else { "Not found" });

    println!("\nStarts with 'car': {}", if trie.starts_with("car") { "Yes" } else { "No" });
    println!("Starts with 'do': {}", if trie.starts_with("do") { "Yes" } else { "No" });
}

// Idiomatic alternative: use HashSet or BTreeSet
#[allow(dead_code)]
fn demonstrate_alternatives() {
    use std::collections::{HashSet, BTreeSet};

    // HashSet for exact matching
    let mut words: HashSet<String> = HashSet::new();
    words.insert("cat".to_string());
    words.insert("car".to_string());

    if words.contains("cat") {
        println!("Found 'cat'");
    }

    // For prefix matching, filter
    let words_vec = vec!["cat", "car", "card", "dog"];
    let with_prefix: Vec<&str> = words_vec.iter()
        .filter(|w| w.starts_with("car"))
        .copied()
        .collect();
    println!("Words starting with 'car': {:?}", with_prefix);

    // BTreeSet for sorted iteration
    let mut sorted: BTreeSet<String> = BTreeSet::new();
    sorted.insert("cat".to_string());
    sorted.insert("car".to_string());

    for word in &sorted {
        println!("{}", word);
    }
}

// Key differences from C:
// 1. Vec<Option<Box<TrieNode>>> instead of TrieNode*[]
// 2. Box for heap allocation
// 3. Option for nullable pointers
// 4. String and &str instead of char*
// 5. RAII: automatic cleanup
// 6. get_or_insert_with for lazy initialization
// 7. chars() iterator for UTF-8 safe traversal
// 8. For simple cases: HashSet or prefix filtering
