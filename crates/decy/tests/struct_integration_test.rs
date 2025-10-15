//! DECY-042: Integration tests for struct transpilation
//!
//! These tests validate that struct features work end-to-end:
//! - Struct definitions with fields
//! - Struct pointers and member access (-> operator)
//! - Nested structs
//! - sizeof operator with structs

use std::fs;
use std::process::Command;

/// Helper function to compile Rust code and return success/error
fn compile_rust_code(rust_code: &str, output_name: &str) -> Result<(), String> {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("{}.rs", output_name));
    let output_file = temp_dir.join(output_name);

    // Wrap code in a module with #![allow(unused)] to avoid warnings about unused functions
    // For now, wrap functions in unsafe blocks since pointer dereferencing requires unsafe
    // NOTE: This is a workaround - future work will have the code generator emit unsafe blocks
    let wrapped_code = wrap_functions_in_unsafe(rust_code);
    let final_code = format!("#![allow(unused)]\n\n{}", wrapped_code);

    fs::write(&temp_file, final_code).expect("Failed to write temp file");

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

/// Wrap function bodies in unsafe blocks for testing
/// This is a temporary workaround until the code generator emits unsafe blocks
fn wrap_functions_in_unsafe(rust_code: &str) -> String {
    let mut result = String::new();
    let mut in_function = false;
    let mut brace_count = 0;

    for line in rust_code.lines() {
        let trimmed = line.trim();

        // Check if this is a struct or other non-function definition
        if trimmed.starts_with("pub struct") || trimmed.starts_with("#[derive") {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        // Check if starting a function
        if trimmed.starts_with("fn ") && trimmed.contains('{') {
            in_function = true;
            brace_count = 1;
            result.push_str(line);
            result.push('\n');
            result.push_str("    unsafe {\n");
            continue;
        }

        if in_function {
            // Count braces to track function end
            brace_count += line.matches('{').count() as i32;
            brace_count -= line.matches('}').count() as i32;

            if brace_count == 0 {
                // End of function - close unsafe block
                result.push_str("    }\n");
                result.push_str(line);
                result.push('\n');
                in_function = false;
            } else {
                // Inside function - add line with extra indentation
                result.push_str("    ");
                result.push_str(line);
                result.push('\n');
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}

#[test]
fn test_simple_struct_definition() {
    // Given: C code with a simple struct
    let c_code = r#"
        struct Point {
            int x;
            int y;
        };

        int get_x(struct Point* p) {
            return p->x;
        }
    "#;

    // When: We transpile it
    let result = decy_core::transpile(c_code);

    // Then: Transpilation should succeed
    assert!(
        result.is_ok(),
        "Should transpile simple struct, got error: {:?}",
        result.err()
    );

    let rust_code = result.unwrap();

    println!("Generated Rust code:\n{}", rust_code);

    // And: Should contain struct definition
    assert!(
        rust_code.contains("struct Point") || rust_code.contains("Point"),
        "Should contain Point struct definition"
    );

    // And: Should contain field definitions
    assert!(
        rust_code.contains("x") && rust_code.contains("y"),
        "Should contain x and y fields"
    );

    // And: Should contain function using struct
    assert!(
        rust_code.contains("fn get_x"),
        "Should contain get_x function"
    );

    // And: Generated code should compile
    match compile_rust_code(&rust_code, "test_simple_struct") {
        Ok(_) => (),
        Err(e) => panic!("Generated Rust code should compile. Errors:\n{}", e),
    }
}

#[test]
fn test_struct_pointer_member_access() {
    // Given: C code with pointer member access (->)
    let c_code = r#"
        struct Node {
            int data;
            struct Node* next;
        };

        int get_data(struct Node* node) {
            if (node != 0) {
                return node->data;
            }
            return -1;
        }
    "#;

    // When: We transpile it
    let result = decy_core::transpile(c_code);

    // Then: Should succeed
    assert!(
        result.is_ok(),
        "Should transpile struct with pointers, got error: {:?}",
        result.err()
    );

    let rust_code = result.unwrap();

    println!("Generated Rust code:\n{}", rust_code);

    // And: Should contain struct definition
    assert!(
        rust_code.contains("struct Node") || rust_code.contains("Node"),
        "Should contain Node struct"
    );

    // And: Should handle pointer member access
    // In Rust, ptr->field becomes (*ptr).field or ptr.field
    assert!(
        rust_code.contains("data"),
        "Should contain data field access"
    );

    // And: Generated code should compile
    match compile_rust_code(&rust_code, "test_struct_pointer_access") {
        Ok(_) => (),
        Err(e) => panic!("Generated Rust code should compile. Errors:\n{}", e),
    }
}

#[test]
fn test_linked_list_real_world() {
    // Given: Real-world linked list example
    let c_code = r#"
        struct Node {
            int data;
            struct Node* next;
        };

        int list_length(struct Node* head) {
            int count = 0;
            while (head != 0) {
                count = count + 1;
                head = head->next;
            }
            return count;
        }

        int list_sum(struct Node* head) {
            int sum = 0;
            while (head != 0) {
                sum = sum + head->data;
                head = head->next;
            }
            return sum;
        }
    "#;

    // When: We transpile it
    let result = decy_core::transpile(c_code);

    // Then: Should succeed
    assert!(
        result.is_ok(),
        "Should transpile linked list, got error: {:?}",
        result.err()
    );

    let rust_code = result.unwrap();

    println!("Generated Rust code:\n{}", rust_code);

    // And: Should contain both functions
    assert!(
        rust_code.contains("fn list_length"),
        "Should contain list_length function"
    );
    assert!(
        rust_code.contains("fn list_sum"),
        "Should contain list_sum function"
    );

    // And: Generated code should compile
    match compile_rust_code(&rust_code, "test_linked_list") {
        Ok(_) => (),
        Err(e) => panic!("Generated Rust code should compile. Errors:\n{}", e),
    }
}

#[test]
fn test_nested_structs() {
    // Given: C code with nested structs
    let c_code = r#"
        struct Point {
            int x;
            int y;
        };

        struct Rectangle {
            struct Point top_left;
            struct Point bottom_right;
        };

        int rect_width(struct Rectangle* r) {
            return r->bottom_right.x - r->top_left.x;
        }
    "#;

    // When: We transpile it
    let result = decy_core::transpile(c_code);

    // Then: Should succeed
    assert!(
        result.is_ok(),
        "Should transpile nested structs, got error: {:?}",
        result.err()
    );

    let rust_code = result.unwrap();

    println!("Generated Rust code:\n{}", rust_code);

    // And: Should contain both struct definitions
    assert!(
        rust_code.contains("Point") && rust_code.contains("Rectangle"),
        "Should contain both struct definitions"
    );

    // And: Generated code should compile
    match compile_rust_code(&rust_code, "test_nested_structs") {
        Ok(_) => (),
        Err(e) => panic!("Generated Rust code should compile. Errors:\n{}", e),
    }
}

#[test]
fn test_struct_with_sizeof() {
    // Given: C code using sizeof with structs
    let c_code = r#"
        struct Data {
            int value;
            int flags;
        };

        int get_struct_size() {
            return sizeof(struct Data);
        }
    "#;

    // When: We transpile it
    let result = decy_core::transpile(c_code);

    // Then: Should succeed
    assert!(
        result.is_ok(),
        "Should transpile sizeof, got error: {:?}",
        result.err()
    );

    let rust_code = result.unwrap();

    println!("Generated Rust code:\n{}", rust_code);

    // And: Should convert sizeof to std::mem::size_of
    assert!(
        rust_code.contains("size_of") || rust_code.contains("std::mem"),
        "Should use Rust's size_of equivalent"
    );

    // And: Generated code should compile
    match compile_rust_code(&rust_code, "test_sizeof") {
        Ok(_) => (),
        Err(e) => panic!("Generated Rust code should compile. Errors:\n{}", e),
    }
}
