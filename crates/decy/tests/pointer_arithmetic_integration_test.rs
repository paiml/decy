//! DECY-041: Integration tests for pointer arithmetic transpilation
//!
//! These tests validate that pointer arithmetic features work end-to-end:
//! - Compound assignment operators (+=, -=, *=, /=, %=)
//! - Increment/decrement operators (++, --)
//! - Real-world pointer arithmetic patterns

use std::fs;
use std::path::Path;
use std::process::Command;

/// Helper function to compile Rust code and return success/error
fn compile_rust_code(rust_code: &str, output_name: &str) -> Result<(), String> {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("{}.rs", output_name));
    let output_file = temp_dir.join(output_name);

    fs::write(&temp_file, rust_code).expect("Failed to write temp file");

    let compile_output = Command::new("rustc")
        .arg(&temp_file)
        .arg("--crate-type")
        .arg("lib")
        .arg("-o")
        .arg(&output_file)
        .output()
        .expect("Failed to run rustc");

    // Clean up
    let _ = fs::remove_file(&temp_file);
    let _ = fs::remove_file(&output_file);

    if compile_output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&compile_output.stderr).to_string())
    }
}

#[test]
fn test_transpile_compound_assignments() {
    // Given: C code with compound assignment operators
    let example_path = "../../examples/pointer_arithmetic/compound_assignments.c";
    assert!(
        Path::new(example_path).exists(),
        "Example file {} should exist",
        example_path
    );

    let c_code = fs::read_to_string(example_path).expect("Failed to read example file");

    // Verify example contains expected operators
    assert!(c_code.contains("+="), "Should contain += operator");
    assert!(c_code.contains("-="), "Should contain -= operator");
    assert!(c_code.contains("*="), "Should contain *= operator");
    assert!(c_code.contains("/="), "Should contain /= operator");
    assert!(c_code.contains("%="), "Should contain %= operator");

    // When: We transpile it
    let result = decy_core::transpile(&c_code);

    // Then: Transpilation should succeed
    assert!(
        result.is_ok(),
        "Should transpile compound assignments, got error: {:?}",
        result.err()
    );

    let rust_code = result.unwrap();

    // And: Generated Rust code should contain all functions
    assert!(
        rust_code.contains("fn increment_by"),
        "Should contain increment_by function"
    );
    assert!(
        rust_code.contains("fn decrement_by"),
        "Should contain decrement_by function"
    );
    assert!(
        rust_code.contains("fn multiply_by"),
        "Should contain multiply_by function"
    );
    assert!(
        rust_code.contains("fn divide_by"),
        "Should contain divide_by function"
    );
    assert!(
        rust_code.contains("fn modulo_by"),
        "Should contain modulo_by function"
    );
    assert!(
        rust_code.contains("fn advance_pointer"),
        "Should contain advance_pointer function"
    );

    // And: Should use i32 for int types
    assert!(
        rust_code.contains("i32"),
        "Should use i32 for int types, got: {}",
        rust_code
    );

    // DECY-041: Verify compound assignments are transpiled correctly
    // They should be converted to expanded form: x += y becomes x = x + y
    println!("Generated Rust code:\n{}", rust_code);

    // And: Generated code should compile
    match compile_rust_code(&rust_code, "test_compound_assignments") {
        Ok(_) => (),
        Err(e) => panic!("Generated Rust code should compile. Errors:\n{}", e),
    }
}

#[test]
#[ignore = "DECY-TBD: Codegen produces non-mut parameters that are then mutated (n = n - 1 with immutable n)"]
fn test_transpile_increment_decrement() {
    // Given: C code with increment/decrement operators
    let example_path = "../../examples/pointer_arithmetic/increment_decrement.c";
    assert!(
        Path::new(example_path).exists(),
        "Example file {} should exist",
        example_path
    );

    let c_code = fs::read_to_string(example_path).expect("Failed to read example file");

    // Verify example contains expected operators
    assert!(c_code.contains("++"), "Should contain ++ operator");
    assert!(c_code.contains("--"), "Should contain -- operator");

    // When: We transpile it
    let result = decy_core::transpile(&c_code);

    // Then: Transpilation should succeed
    assert!(
        result.is_ok(),
        "Should transpile increment/decrement, got error: {:?}",
        result.err()
    );

    let rust_code = result.unwrap();

    // And: Generated Rust code should contain all test functions
    assert!(
        rust_code.contains("fn post_increment_test"),
        "Should contain post_increment_test function"
    );
    assert!(
        rust_code.contains("fn pre_increment_test"),
        "Should contain pre_increment_test function"
    );
    assert!(
        rust_code.contains("fn post_decrement_test"),
        "Should contain post_decrement_test function"
    );
    assert!(
        rust_code.contains("fn pre_decrement_test"),
        "Should contain pre_decrement_test function"
    );
    assert!(
        rust_code.contains("fn sum_to_n"),
        "Should contain sum_to_n function with for loop"
    );
    assert!(
        rust_code.contains("fn countdown_sum"),
        "Should contain countdown_sum function"
    );

    // DECY-041: Verify increment/decrement are transpiled correctly
    // They should be converted to: x++ becomes x = x + 1
    println!("Generated Rust code:\n{}", rust_code);

    // And: Generated code should compile
    match compile_rust_code(&rust_code, "test_increment_decrement") {
        Ok(_) => (),
        Err(e) => panic!("Generated Rust code should compile. Errors:\n{}", e),
    }
}

#[test]
#[ignore = "DECY-TBD: Codegen produces invalid pointer arithmetic on slices (arr + arr.len())"]
fn test_transpile_real_world_patterns() {
    // Given: C code with real-world pointer arithmetic patterns
    let example_path = "../../examples/pointer_arithmetic/real_world_patterns.c";
    assert!(
        Path::new(example_path).exists(),
        "Example file {} should exist",
        example_path
    );

    let c_code = fs::read_to_string(example_path).expect("Failed to read example file");

    // Verify example contains expected patterns
    assert!(
        c_code.contains("arr + size"),
        "Should contain pointer arithmetic"
    );
    assert!(c_code.contains("arr++"), "Should contain pointer increment");
    assert!(c_code.contains("break"), "Should contain break statement");
    assert!(
        c_code.contains("continue"),
        "Should contain continue statement"
    );

    // When: We transpile it
    let result = decy_core::transpile(&c_code);

    // Then: Transpilation should succeed
    assert!(
        result.is_ok(),
        "Should transpile real-world patterns, got error: {:?}",
        result.err()
    );

    let rust_code = result.unwrap();

    // And: Generated Rust code should contain all functions
    assert!(
        rust_code.contains("fn sum_array"),
        "Should contain sum_array function"
    );
    assert!(
        rust_code.contains("fn find_first"),
        "Should contain find_first function"
    );
    assert!(
        rust_code.contains("fn count_even"),
        "Should contain count_even function with continue"
    );
    assert!(
        rust_code.contains("fn linear_search"),
        "Should contain linear_search function with break"
    );
    assert!(
        rust_code.contains("fn string_length"),
        "Should contain string_length function"
    );

    // And: Should contain break and continue statements
    assert!(
        rust_code.contains("break"),
        "Should preserve break statements"
    );
    assert!(
        rust_code.contains("continue"),
        "Should preserve continue statements"
    );

    println!("Generated Rust code:\n{}", rust_code);

    // And: Generated code should compile
    match compile_rust_code(&rust_code, "test_real_world_patterns") {
        Ok(_) => (),
        Err(e) => panic!("Generated Rust code should compile. Errors:\n{}", e),
    }
}

#[test]
fn test_for_loop_with_increment() {
    // Given: Simple for loop with i++ increment
    let c_code = r#"
        int sum_range(int n) {
            int sum = 0;
            int i;
            for (i = 0; i < n; i++) {
                sum += i;
            }
            return sum;
        }
    "#;

    // When: We transpile it
    let result = decy_core::transpile(c_code);

    // Then: Should succeed
    assert!(
        result.is_ok(),
        "Should transpile for loop with i++, got error: {:?}",
        result.err()
    );

    let rust_code = result.unwrap();

    // And: Should convert for loop to while loop with increment
    assert!(
        rust_code.contains("while"),
        "For loop should convert to while loop"
    );

    println!("Generated Rust code:\n{}", rust_code);

    // And: Generated code should compile
    match compile_rust_code(&rust_code, "test_for_loop_increment") {
        Ok(_) => (),
        Err(e) => panic!("Generated Rust code should compile. Errors:\n{}", e),
    }
}

#[test]
fn test_nested_loops_with_break_continue() {
    // Given: Nested loops with break and continue (from DECY-042)
    let c_code = r#"
        int find_pair_sum(int* arr, int size, int target) {
            int i;
            int j;
            for (i = 0; i < size; i++) {
                for (j = i + 1; j < size; j++) {
                    if (arr[i] + arr[j] == target) {
                        return 1;  // Found
                    }
                }
            }
            return 0;  // Not found
        }

        int count_valid(int* arr, int size) {
            int count = 0;
            int i;
            for (i = 0; i < size; i++) {
                if (arr[i] < 0) {
                    continue;  // Skip negative
                }
                count++;
            }
            return count;
        }
    "#;

    // When: We transpile it
    let result = decy_core::transpile(c_code);

    // Then: Should succeed
    assert!(
        result.is_ok(),
        "Should transpile nested loops, got error: {:?}",
        result.err()
    );

    let rust_code = result.unwrap();

    println!("Generated Rust code:\n{}", rust_code);

    // And: Generated code should compile
    match compile_rust_code(&rust_code, "test_nested_loops") {
        Ok(_) => (),
        Err(e) => panic!("Generated Rust code should compile. Errors:\n{}", e),
    }
}
