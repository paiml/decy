/* K&R C Chapter 5: Pointer-Based String Manipulation
 * Advanced string operations using pointers
 * Transpiled to safe Rust (using iterators and String methods)
 */

// String reverse in-place (on Vec<char> or String)
fn str_reverse(s: &mut String) {
    let chars: Vec<char> = s.chars().rev().collect();
    s.clear();
    s.extend(chars);
}

// String to uppercase
fn str_upper(s: &mut String) {
    *s = s.to_uppercase();
}

// String to lowercase
fn str_lower(s: &mut String) {
    *s = s.to_lowercase();
}

// Count words in string
fn count_words(s: &str) -> usize {
    s.split_whitespace().count()
}

// Find substring (returns Option instead of pointer)
fn str_find<'a>(haystack: &'a str, needle: &str) -> Option<&'a str> {
    haystack.find(needle).map(|pos| &haystack[pos..])
}

// Remove leading/trailing whitespace
fn str_trim(s: &str) -> &str {
    s.trim()
}

fn main() {
    let mut str1 = String::from("Hello, World!");
    let str2 = "  spaces around  ";
    let str3 = "The quick brown fox";
    let text = "Find the needle in the haystack";

    println!("Original: \"{}\"", str1);
    str_reverse(&mut str1);
    println!("Reversed: \"{}\"", str1);
    str_reverse(&mut str1);  // Reverse back

    println!("\nOriginal: \"{}\"", str1);
    str_upper(&mut str1);
    println!("Uppercase: \"{}\"", str1);
    str_lower(&mut str1);
    println!("Lowercase: \"{}\"", str1);

    println!("\nOriginal: \"{}\"", str2);
    let trimmed = str_trim(str2);
    println!("Trimmed: \"{}\"", trimmed);

    println!("\nCounting words in: \"{}\"", str3);
    println!("Word count: {}", count_words(str3));

    println!("\nSearching in: \"{}\"", text);
    if let Some(found) = str_find(text, "needle") {
        println!("Found \"needle\" at: \"{}\"", found);
    } else {
        println!("Not found");
    }

    match str_find(text, "xyz") {
        Some(found) => println!("Found \"xyz\": {}", found),
        None => println!("\"xyz\" not found"),
    }
}

// Demonstrate iterator-based string operations
fn demonstrate_iterators() {
    let text = "Hello, World!";

    // Character iteration
    let uppercase: String = text.chars().map(|c| c.to_ascii_uppercase()).collect();
    println!("Uppercase: {}", uppercase);

    // Filter characters
    let letters: String = text.chars().filter(|c| c.is_alphabetic()).collect();
    println!("Letters only: {}", letters);

    // Count specific characters
    let count = text.chars().filter(|&c| c == 'l').count();
    println!("Count of 'l': {}", count);

    // Word iteration
    for word in text.split_whitespace() {
        println!("Word: {}", word);
    }
}

// Manual character-by-character processing (closer to C pointer style)
fn manual_processing(s: &str) -> String {
    let mut result = String::new();

    for ch in s.chars() {
        if ch.is_alphabetic() {
            result.push(ch.to_ascii_uppercase());
        }
    }

    result
}

// Key differences from C:
// 1. String/&str instead of char*
// 2. Iterators instead of pointer traversal
// 3. Option<&str> instead of NULL return
// 4. Built-in methods: .to_uppercase(), .trim(), .find()
// 5. UTF-8 safe by default
// 6. No null terminator needed
// 7. Bounds checking automatic
