/* K&R C Chapter 7: Wide Character I/O
 * Transpiled to safe Rust (native Unicode support)
 */

fn main() {
    println!("=== Wide Character I/O ===\n");
    
    println!("Unicode strings (native in Rust):");
    println!("  English: Hello, World!");
    println!("  Spanish: ¡Hola, Mundo!");
    println!("  French: Bonjour, le Monde!");
    println!("  German: Hallo, Welt!");
    println!("  Russian: Привет, мир!");
    println!("  Japanese: こんにちは世界!");
    println!("  Chinese: 你好世界");
    println!("  Arabic: مرحبا بالعالم!");
    println!("  Emoji: 😀 🚀 ⭐ 💚");
    println!();
    
    wide_string_operations();
    unicode_operations();
    
    println!("\nUnicode in Rust:");
    println!("  - String is always UTF-8");
    println!("  - char is a Unicode scalar value");
    println!("  - Full Unicode support built-in");
    println!("  - No separate wide char types needed");
}

fn wide_string_operations() {
    println!("=== String Operations ===");
    
    let str1 = "Hello";
    let str2 = "World";
    let result = format!("{} {}", str1, str2);
    
    println!("  Length of '{}': {}", str1, str1.len());
    println!("  Concatenated: {}", result);
    println!("  Compare: {}", str1.cmp(str2) as i8);
    
    if let Some(pos) = result.find('o') {
        println!("  Found 'o' at position: {}", pos);
    }
    println!();
}

fn unicode_operations() {
    println!("=== Unicode Operations ===");
    
    let text = "The quick brown 狐 jumps";
    
    println!("  Text: {}", text);
    println!("  Byte length: {}", text.len());
    println!("  Char count: {}", text.chars().count());
    
    print!("  Characters: ");
    for ch in text.chars() {
        print!("'{}' ", ch);
    }
    println!("\n");
    
    let emoji = "😀🚀⭐";
    println!("  Emoji: {}", emoji);
    println!("  Emoji count: {}", emoji.chars().count());
}

// Key differences from C:
// 1. No wchar_t - String is UTF-8
// 2. char is Unicode scalar value (4 bytes)
// 3. No wprintf/wscanf needed
// 4. Unicode support is default
// 5. chars() iterator for Unicode iteration
// 6. Full grapheme cluster support available
// 7. No locale dependencies
// 8. Type-safe Unicode handling
