/* K&R C Chapter 3.8: Goto and Labels
 * Page 65-66
 * Error handling with goto
 * Transpiled to safe Rust (using Result and RAII)
 */

// Rust doesn't need goto for cleanup - uses RAII and Result
fn process_data(data: &[i32]) -> Result<(), String> {
    let size = data.len();

    // Vec automatically cleans up when dropped (RAII)
    let mut temp1 = Vec::with_capacity(size);
    let mut temp2 = Vec::with_capacity(size);

    // Process data
    for &val in data {
        temp1.push(val * 2);
        temp2.push(val + 10);
    }

    // Success
    println!("Data processed successfully");
    Ok(())

    // temp1 and temp2 automatically freed when function returns
    // No need for goto cleanup label
}

fn main() {
    let data = vec![1, 2, 3, 4, 5];

    println!("Processing with sufficient memory:");
    match process_data(&data) {
        Ok(()) => println!("Result: 0 (success)\n"),
        Err(e) => println!("Error: {}\n", e),
    }
}

// C version with goto requires manual cleanup
// Rust version uses RAII: automatic cleanup on scope exit

// Demonstrating error handling without goto
fn process_with_errors(data: &[i32]) -> Result<(), String> {
    // Early return pattern instead of goto
    if data.is_empty() {
        return Err("Empty data".to_string());
    }

    let temp1: Result<Vec<i32>, String> = Ok(
        data.iter().map(|&x| x * 2).collect()
    );

    temp1?;  // Early return if error (? operator)

    println!("Processing complete");
    Ok(())
}

// Key differences from C:
// 1. No goto needed - RAII handles cleanup
// 2. Result type for error handling
// 3. ? operator for early returns
// 4. Automatic memory cleanup (Drop trait)
// 5. Compiler ensures all paths clean up properly
