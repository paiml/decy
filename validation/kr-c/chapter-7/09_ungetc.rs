/* K&R C Chapter 7.8: Miscellaneous Functions - ungetc
 * Page 166
 * Transpiled to safe Rust (using Peekable iterator)
 */

use std::io::{self, BufRead};

fn main() {
    println!("=== Pushback/Peek (ungetc alternative) ===\n");

    println!("Enter a number followed by any character: ");

    let stdin = io::stdin();
    let mut chars = stdin.lock().bytes().peekable();

    // Skip whitespace
    while let Some(Ok(b)) = chars.peek() {
        if b.is_ascii_whitespace() {
            chars.next();
        } else {
            break;
        }
    }

    // Read number
    let number = read_number(&mut chars);
    println!("Number read: {}", number);

    // The non-digit was "peeked" but not consumed
    // Read the next character
    if let Some(Ok(ch)) = chars.next() {
        println!("Next character: {}", ch as char);
    }

    // Demonstrate with string parsing
    demo_string_parsing();
}

// Read number using peek (like ungetc pattern)
fn read_number<I>(chars: &mut std::iter::Peekable<I>) -> i32
where
    I: Iterator<Item = io::Result<u8>>,
{
    let mut n = 0;

    while let Some(&Ok(b)) = chars.peek() {
        if b.is_ascii_digit() {
            n = 10 * n + (b - b'0') as i32;
            chars.next();  // Consume the digit
        } else {
            // Don't consume - leave it for next read (like ungetc)
            break;
        }
    }

    n
}

fn demo_string_parsing() {
    println!("\n=== String Parsing with Peek ===\n");

    let input = "123abc456def";
    let mut iter = input.chars().peekable();

    println!("Input: {}", input);
    println!("Parsing numbers and letters:\n");

    while iter.peek().is_some() {
        // Try to read a number
        let mut num = String::new();
        while let Some(&ch) = iter.peek() {
            if ch.is_ascii_digit() {
                num.push(ch);
                iter.next();
            } else {
                break;
            }
        }

        if !num.is_empty() {
            println!("  Number: {}", num);
        }

        // Try to read letters
        let mut word = String::new();
        while let Some(&ch) = iter.peek() {
            if ch.is_ascii_alphabetic() {
                word.push(ch);
                iter.next();
            } else {
                break;
            }
        }

        if !word.is_empty() {
            println!("  Word: {}", word);
        }
    }
}

// Advanced: Parse tokens with lookahead
#[allow(dead_code)]
fn parse_expression(input: &str) {
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            '0'..='9' => {
                let num = read_number_from_chars(&mut chars);
                println!("Number: {}", num);
            }
            '+' | '-' | '*' | '/' => {
                println!("Operator: {}", ch);
                chars.next();
            }
            ' ' => {
                chars.next();  // Skip whitespace
            }
            _ => {
                println!("Unknown: {}", ch);
                chars.next();
            }
        }
    }
}

fn read_number_from_chars<I>(chars: &mut std::iter::Peekable<I>) -> i32
where
    I: Iterator<Item = char>,
{
    let mut n = 0;
    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() {
            n = 10 * n + ch.to_digit(10).unwrap() as i32;
            chars.next();
        } else {
            break;
        }
    }
    n
}

// Key differences from C:
// 1. Peekable iterator instead of ungetc
// 2. peek() looks at next item without consuming
// 3. Type-safe iteration
// 4. No buffer limitations
// 5. Iterator combinators
// 6. No manual pushback required
// 7. chars() or bytes() for iteration
// 8. Pattern matching for parsing
