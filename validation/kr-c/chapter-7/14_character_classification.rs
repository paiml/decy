/* K&R C Chapter 7: Character Classification
 * K&R §7.2, Appendix B: ctype.h functions
 * Transpiled to safe Rust (using char methods)
 */

fn main() {
    println!("=== Character Classification ===\n");

    // Individual character classification
    println!("Individual character classification:");
    classify_char('A');
    println!();
    classify_char('5');
    println!();
    classify_char(' ');
    println!();

    // Analyze strings
    println!("String analysis:");
    analyze_string("Hello, World! 123");
    println!();
    analyze_string("Test@Example.com");
    println!();

    // Case conversion
    println!("Case conversion:");
    let str1 = "Hello, World!";
    println!("  Original: {}", str1);
    println!("  Upper:    {}", str_to_upper(str1));

    let str2 = "Hello, World!";
    println!("  Lower:    {}", str_to_lower(str2));

    let str3 = "hello world from rust programming";
    println!("  Title:    {}", str_to_title(str3));
    println!();

    // Remove non-alphanumeric
    println!("Remove non-alphanumeric:");
    let str4 = "abc-123-xyz!@#";
    println!("  Original: {}", str4);
    println!("  Cleaned:  {}", remove_non_alnum(str4));
    println!();

    // Password validation
    println!("Password validation:");
    let passwords = vec!["weak", "Weak123", "Strong@123", "VeryStrong!2024"];
    for password in &passwords {
        let valid = validate_password(password);
        println!(
            "  '{}': {}",
            password,
            if valid { "VALID" } else { "INVALID" }
        );
    }
    println!();

    // Extract words
    println!("Word extraction:");
    extract_words("The quick brown fox jumps over the lazy dog!");
    println!();

    // Hexadecimal digit check
    println!("Hexadecimal digit check:");
    let hex_chars = "0123456789abcdefABCDEFxyzXYZ";
    for ch in hex_chars.chars() {
        println!(
            "  '{}': {}",
            ch,
            if ch.is_ascii_hexdigit() { "YES" } else { "NO" }
        );
    }

    println!("\nchar methods in Rust:");
    println!("  - is_alphabetic, is_numeric, is_alphanumeric");
    println!("  - is_whitespace, is_ascii");
    println!("  - to_uppercase, to_lowercase");
    println!("  - Built-in Unicode support");
}

fn classify_char(c: char) {
    println!("Character '{}' (U+{:04X}):", c, c as u32);
    println!("  is_alphabetic: {}", c.is_alphabetic());
    println!("  is_numeric: {}", c.is_numeric());
    println!("  is_alphanumeric: {}", c.is_alphanumeric());
    println!("  is_whitespace: {}", c.is_whitespace());
    println!("  is_uppercase: {}", c.is_uppercase());
    println!("  is_lowercase: {}", c.is_lowercase());
    println!("  is_ascii_punctuation: {}", c.is_ascii_punctuation());
    println!("  is_ascii: {}", c.is_ascii());
    println!("  is_control: {}", c.is_control());
    println!("  to_uppercase: '{}'", c.to_uppercase().next().unwrap_or(c));
    println!("  to_lowercase: '{}'", c.to_lowercase().next().unwrap_or(c));
}

fn analyze_string(s: &str) {
    let alpha = s.chars().filter(|c| c.is_alphabetic()).count();
    let digit = s.chars().filter(|c| c.is_numeric()).count();
    let space = s.chars().filter(|c| c.is_whitespace()).count();
    let punct = s.chars().filter(|c| c.is_ascii_punctuation()).count();
    let other = s.len() - alpha - digit - space - punct;

    println!("String: \"{}\"", s);
    println!("  Alphabetic: {}", alpha);
    println!("  Digits:     {}", digit);
    println!("  Whitespace: {}", space);
    println!("  Punctuation:{}", punct);
    println!("  Other:      {}", other);
    println!("  Total:      {}", s.chars().count());
}

fn str_to_upper(s: &str) -> String {
    s.to_uppercase()
}

fn str_to_lower(s: &str) -> String {
    s.to_lowercase()
}

fn str_to_title(s: &str) -> String {
    let mut result = String::new();
    let mut new_word = true;

    for ch in s.chars() {
        if ch.is_whitespace() {
            result.push(ch);
            new_word = true;
        } else if new_word {
            result.push(ch.to_uppercase().next().unwrap_or(ch));
            new_word = false;
        } else {
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        }
    }

    result
}

fn remove_non_alnum(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).collect()
}

fn validate_password(password: &str) -> bool {
    if password.len() < 8 {
        return false;
    }

    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| c.is_ascii_punctuation());

    has_upper && has_lower && has_digit && has_special
}

fn extract_words(text: &str) {
    let words: Vec<&str> = text.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect();

    println!("Words extracted from text:");
    for (i, word) in words.iter().enumerate() {
        println!("  Word {}: '{}'", i + 1, word);
    }
    println!("Total words: {}", words.len());
}

// Key differences from C:
// 1. char methods instead of ctype.h functions
// 2. Unicode-aware by default
// 3. Iterator methods (filter, any, count)
// 4. to_uppercase/to_lowercase return iterators
// 5. is_alphabetic vs isalpha (Unicode-aware)
// 6. is_numeric vs isdigit
// 7. is_alphanumeric vs isalnum
// 8. No locale dependencies for basic operations
