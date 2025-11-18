/* K&R C Chapter 7.2, 7.4: String Formatting
 * Page 154, 159
 * Transpiled to safe Rust (using format! macro and parse)
 */

fn main() {
    println!("=== String Formatting (sprintf/sscanf alternatives) ===\n");

    // sprintf equivalent: format! macro
    let buffer = format!("Date: {} {}, {}", "January", 15, 2024);
    println!("Formatted string: {}", buffer);

    // sscanf equivalent: split and parse
    let date_str = "January 15 2024";
    let parts: Vec<&str> = date_str.split_whitespace().collect();

    if parts.len() >= 3 {
        let month = parts[0];
        let day: i32 = parts[1].parse().unwrap_or(0);
        let year: i32 = parts[2].parse().unwrap_or(0);

        println!("Parsed: month={} day={} year={}", month, day, year);
    }

    // Number parsing from string
    let temp_str = "Temperature: 72.5";
    if let Some(temp_part) = temp_str.split(':').nth(1) {
        let temp: f32 = temp_part.trim().parse().unwrap_or(0.0);
        println!("Temperature: {:.1}", temp);
    }

    // Demonstrate various formatting options
    demo_formatting();

    // Demonstrate parsing with error handling
    demo_parsing();
}

fn demo_formatting() {
    println!("\n=== Format Specifiers ===\n");

    let n = 42;
    let x = 255;
    let f = 3.14159;

    println!("Decimal:       {}", n);
    println!("Hex:           {:x}", x);
    println!("Hex uppercase: {:X}", x);
    println!("Octal:         {:o}", x);
    println!("Binary:        {:b}", x);
    println!("Float:         {:.2}", f);
    println!("Scientific:    {:e}", f);
    println!("Padded:        {:05}", n);
    println!("Aligned:       |{:>10}|", n);
    println!("Left aligned:  |{:<10}|", n);

    // Format into String
    let s = format!("{:#x}", x);  // 0xff
    println!("Formatted hex: {}", s);
}

fn demo_parsing() {
    println!("\n=== Parsing with Error Handling ===\n");

    let inputs = vec!["42", "3.14", "invalid", "0xFF"];

    for input in inputs {
        // Parse as integer
        match input.parse::<i32>() {
            Ok(n) => println!("{:10} -> i32:  {}", input, n),
            Err(_) => {
                // Try parsing as float
                match input.parse::<f32>() {
                    Ok(f) => println!("{:10} -> f32:  {:.2}", input, f),
                    Err(_) => println!("{:10} -> Error: invalid number", input),
                }
            }
        }
    }

    // Parse hex
    println!("\n=== Hex Parsing ===\n");
    let hex_str = "FF";
    match i32::from_str_radix(hex_str, 16) {
        Ok(n) => println!("0x{} = {}", hex_str, n),
        Err(e) => println!("Parse error: {}", e),
    }
}

// Demonstrate complex parsing
#[allow(dead_code)]
fn parse_complex(input: &str) -> Option<(String, i32, i32)> {
    let parts: Vec<&str> = input.split_whitespace().collect();

    if parts.len() >= 3 {
        let month = parts[0].to_string();
        let day = parts[1].parse().ok()?;
        let year = parts[2].parse().ok()?;
        Some((month, day, year))
    } else {
        None
    }
}

// Key differences from C:
// 1. format! macro instead of sprintf
// 2. No buffer overflow risk
// 3. parse() instead of sscanf
// 4. Type inference with turbofish ::<T>
// 5. from_str_radix for non-decimal parsing
// 6. Result<T, E> for parse errors
// 7. No format string vulnerabilities
// 8. Compile-time format checking
