/* K&R C Chapter 7: String Conversion Functions
 * K&R §7.2, Appendix B: atoi, atof, strtol, strtod
 * Transpiled to safe Rust (using parse method)
 */

fn main() {
    println!("=== String Conversion Functions ===\n");

    demo_parse_int();
    demo_parse_float();
    demo_parse_with_radix();
    demo_csv_parsing();
    demo_hex_color();
}

fn demo_parse_int() {
    println!("=== parse::<i32>() - String to Integer ===");

    let test_cases = vec!["123", "-456", "  789", "42abc", "abc42", "0"];

    for case in &test_cases {
        match case.trim().parse::<i32>() {
            Ok(n) => println!("  parse(\"{}\") = {}", case, n),
            Err(e) => println!("  parse(\"{}\") = Error: {}", case, e),
        }
    }
    println!();
}

fn demo_parse_float() {
    println!("=== parse::<f64>() - String to Float ===");

    let test_cases = vec!["3.14159", "-2.71828", "  1.414", "1.23e2", "1e-3", "0.0"];

    for case in &test_cases {
        match case.trim().parse::<f64>() {
            Ok(n) => println!("  parse(\"{}\") = {}", case, n),
            Err(e) => println!("  parse(\"{}\") = Error: {}", case, e),
        }
    }
    println!();
}

fn demo_parse_with_radix() {
    println!("=== from_str_radix() - Parse with Base ===");

    println!("  from_str_radix(\"FF\", 16) = {}", i32::from_str_radix("FF", 16).unwrap());
    println!("  from_str_radix(\"77\", 8) = {}", i32::from_str_radix("77", 8).unwrap());
    println!("  from_str_radix(\"1010\", 2) = {}", i32::from_str_radix("1010", 2).unwrap());
    println!();
}

fn demo_csv_parsing() {
    println!("=== CSV Parsing ===");

    let line = "123,45.67,hello,3.14";
    println!("Parsing CSV: \"{}\"", line);

    for (i, field) in line.split(',').enumerate() {
        if let Ok(int_val) = field.parse::<i32>() {
            println!("  Field {} (int): {}", i, int_val);
        } else if let Ok(float_val) = field.parse::<f64>() {
            println!("  Field {} (float): {}", i, float_val);
        } else {
            println!("  Field {} (string): '{}'", i, field);
        }
    }
    println!();
}

fn demo_hex_color() {
    println!("=== Hex Color Parsing ===");

    let colors = vec!["#FF5733", "#00AAFF", "#123456"];

    for hex in &colors {
        if let Some(rgb) = parse_hex_color(hex) {
            println!("Color {}: RGB({}, {}, {})", hex, rgb.0, rgb.1, rgb.2);
        }
    }
    println!();
}

fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return None;
    }

    let color = u32::from_str_radix(hex, 16).ok()?;
    let r = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = (color & 0xFF) as u8;

    Some((r, g, b))
}

// Key differences from C:
// 1. parse() method instead of atoi/atof
// 2. from_str_radix for different bases
// 3. Result<T, E> for error handling
// 4. No errno or endptr needed
// 5. Type inference with turbofish ::<T>
// 6. Overflow checking built-in
// 7. UTF-8 safe
// 8. No undefined behavior
