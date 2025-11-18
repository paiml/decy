/* K&R C Chapter 7: String Formatting
 * K&R §7.2: sprintf, snprintf
 * Transpiled to safe Rust (using format! macro)
 */

use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    println!("=== String Formatting ===\n");

    println!("Formatted reports:");
    build_report(42, "Alice", 1234.56);
    build_report(123, "Bob", 9876.54);
    println!();

    println!("SQL query building:");
    build_sql_query("users", "id", 42);
    println!();

    println!("JSON building:");
    build_json(101, "Alice", 30, 75000.00);
    println!();

    println!("File size formatting:");
    format_file_size(500);
    format_file_size(5000);
    format_file_size(5000000);
    println!();

    println!("Progress bar:");
    for percent in [0, 25, 50, 75, 100] {
        format_progress_bar(percent);
    }
    println!();
}

fn build_report(id: i32, name: &str, amount: f64) {
    let buffer = format!("Report #{:04}: {} - ${:.2}", id, name, amount);
    println!("Report: {}", buffer);
}

fn build_sql_query(table: &str, column: &str, value: i32) {
    let query = format!("SELECT * FROM {} WHERE {} = {};", table, column, value);
    println!("SQL Query: {}", query);
}

fn build_json(id: i32, name: &str, age: i32, salary: f64) {
    let json = format!(
        "{{\n  \"id\": {},\n  \"name\": \"{}\",\n  \"age\": {},\n  \"salary\": {:.2}\n}}",
        id, name, age, salary
    );
    println!("JSON:\n{}", json);
}

fn format_file_size(bytes: i64) {
    let size = if bytes < 1024 {
        format!("{} bytes", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    };
    println!("Size: {}", size);
}

fn format_progress_bar(percent: i32) {
    let bar_width = 50;
    let filled = (bar_width * percent) / 100;
    let empty = bar_width - filled;
    
    let bar = format!("[{}{}] {}%",
        "#".repeat(filled as usize),
        " ".repeat(empty as usize),
        percent);
    println!("{}", bar);
}

// Key differences from C:
// 1. format! macro instead of sprintf
// 2. No buffer overflow risk
// 3. String grows dynamically
// 4. Type-safe formatting
// 5. Compile-time format checking
// 6. {} placeholders instead of % format specifiers
// 7. No manual buffer size tracking
// 8. UTF-8 safe
