/* K&R C Chapter 7.7: Line Input and Output
 * Page 164-165
 * Transpiled to safe Rust (using BufRead trait and lines iterator)
 */

use std::io::{self, BufRead};

fn main() {
    println!("=== Line Input and Output ===\n");

    println!("Enter lines of text (Ctrl+D on Unix, Ctrl+Z on Windows to end):");

    let stdin = io::stdin();
    let mut count = 0;

    // Read and echo lines using lines() iterator
    for line in stdin.lock().lines() {
        match line {
            Ok(text) => {
                count += 1;
                println!("{:3}: {}", count, text);
            }
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                break;
            }
        }
    }

    println!("\nTotal lines: {}", count);

    // Demonstrate other line reading methods
    demo_read_line();
}

fn demo_read_line() {
    println!("\n=== Alternative: read_line() ===\n");

    let stdin = io::stdin();
    let mut buffer = String::new();
    let mut line_count = 0;

    println!("Enter one line:");

    match stdin.read_line(&mut buffer) {
        Ok(bytes) => {
            line_count += 1;
            println!("Read {} bytes: {}", bytes, buffer.trim());
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("Lines read with read_line: {}", line_count);
}

// Demonstrate collecting all lines into a Vec
#[allow(dead_code)]
fn demo_collect_lines() -> io::Result<Vec<String>> {
    let stdin = io::stdin();
    let lines: Vec<String> = stdin.lock().lines().collect::<Result<_, _>>()?;

    println!("Collected {} lines", lines.len());
    for (i, line) in lines.iter().enumerate() {
        println!("{}: {}", i + 1, line);
    }

    Ok(lines)
}

// Key differences from C:
// 1. BufRead trait for line-oriented input
// 2. lines() returns Iterator<Item = Result<String>>
// 3. No fixed buffer size (String grows dynamically)
// 4. UTF-8 validation built-in
// 5. No need for fgets/fputs
// 6. println! instead of fputs(line, stdout)
// 7. Iterator methods (for line in lines)
// 8. No manual newline handling
