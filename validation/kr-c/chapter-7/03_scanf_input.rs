/* K&R C Chapter 7.4: Formatted Input - scanf
 * Page 157-159
 * Transpiled to safe Rust (using stdin and parse)
 */

use std::io::{self, Write};

fn main() {
    println!("=== Formatted Input (scanf alternative) ===\n");

    // Read date components
    print!("Enter date (e.g., January 15 2024): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");

    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.len() >= 3 {
        let month = parts[0];
        let day: i32 = parts[1].parse().unwrap_or(0);
        let year: i32 = parts[2].parse().unwrap_or(0);

        println!("Parsed: month={} day={} year={}", month, day, year);
    }

    // Read temperature
    print!("Enter temperature: ");
    io::stdout().flush().unwrap();

    let mut temp_input = String::new();
    io::stdin().read_line(&mut temp_input).expect("Failed to read line");

    let temperature: f32 = temp_input.trim().parse().unwrap_or(0.0);

    println!("Date: {} {}, {}", parts[0], parts[1], parts[2]);
    println!("Temperature: {:.1} degrees", temperature);

    // Demonstrate better error handling
    println!("\n=== With Better Error Handling ===\n");

    print!("Enter a number: ");
    io::stdout().flush().unwrap();

    let mut num_input = String::new();
    io::stdin().read_line(&mut num_input).expect("Failed to read line");

    match num_input.trim().parse::<i32>() {
        Ok(num) => println!("You entered: {}", num),
        Err(e) => println!("Invalid number: {}", e),
    }

    println!("\nKey differences from C scanf:");
    println!("  - read_line() returns Result");
    println!("  - parse() for type conversion");
    println!("  - No format string vulnerabilities");
    println!("  - Explicit error handling");
    println!("  - UTF-8 safe");
}

// Key differences from C:
// 1. stdin() instead of scanf
// 2. read_line() for input
// 3. parse() for type conversion
// 4. Result<T, E> for error handling
// 5. No format string parsing
// 6. split_whitespace() for parsing
// 7. Type inference with turbofish ::<T>
// 8. UTF-8 by default (String vs char*)
