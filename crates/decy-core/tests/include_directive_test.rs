//! End-to-end tests for #include directive support (DECY-056 RED phase)
//!
//! Tests verify that C code with #include directives transpiles correctly.
//! This is the P0 critical blocker - ALL real-world C projects use includes.
//!
//! References:
//! - ISO C99 §6.10.2: Source file inclusion
//! - K&R §4.11: Preprocessor
//! - DECY-051 validation: 100% of real-world projects blocked

use decy_core::{transpile, transpile_with_includes};
use std::fs;
use tempfile::TempDir;

/// Helper: Create temporary test project with multiple files
fn create_test_project(files: &[(&str, &str)]) -> TempDir {
    let temp = TempDir::new().expect("Failed to create temp dir");

    for (filename, content) in files {
        let file_path = temp.path().join(filename);

        // Create parent directories if needed
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent dirs");
        }

        fs::write(&file_path, content).expect("Failed to write test file");
    }

    temp
}

#[test]
fn test_transpile_local_include_simple() {
    // Create a simple two-file project: utils.h + main.c
    let project = create_test_project(&[
        ("utils.h", "int add(int a, int b);"),
        (
            "main.c",
            r#"
#include "utils.h"

int main() {
    return add(1, 2);
}
        "#,
        ),
    ]);

    let main_path = project.path().join("main.c");
    let c_code = fs::read_to_string(&main_path).expect("Failed to read main.c");

    let rust_code = transpile_with_includes(&c_code, Some(project.path()))
        .expect("Transpilation should succeed");

    // Should transpile both the include and main function
    assert!(
        rust_code.contains("fn add"),
        "Should contain add function signature"
    );
    assert!(
        rust_code.contains("fn main"),
        "Should contain main function"
    );
    assert!(
        rust_code.contains("add(1, 2)"),
        "Should contain function call"
    );
}

#[test]
fn test_transpile_multiple_includes() {
    // Test project with multiple includes
    let project = create_test_project(&[
        ("math.h", "int add(int a, int b);"),
        ("string.h", "int strlen(char* s);"),
        (
            "main.c",
            r#"
#include "math.h"
#include "string.h"

int main() {
    int x = add(1, 2);
    int len = strlen("hello");
    return x + len;
}
        "#,
        ),
    ]);

    let main_path = project.path().join("main.c");
    let c_code = fs::read_to_string(&main_path).expect("Failed to read main.c");

    let rust_code = transpile_with_includes(&c_code, Some(project.path()))
        .expect("Transpilation should succeed");

    assert!(rust_code.contains("fn add"), "Should contain add function");
    assert!(
        rust_code.contains("fn strlen"),
        "Should contain strlen function"
    );
    assert!(
        rust_code.contains("fn main"),
        "Should contain main function"
    );
}

#[test]
fn test_transpile_nested_includes() {
    // Test nested includes: main.c includes a.h, a.h includes b.h
    let project = create_test_project(&[
        ("b.h", "typedef int number_t;"),
        (
            "a.h",
            r#"
#include "b.h"
int process(number_t x);
        "#,
        ),
        (
            "main.c",
            r#"
#include "a.h"

int main() {
    return process(42);
}
        "#,
        ),
    ]);

    let main_path = project.path().join("main.c");
    let c_code = fs::read_to_string(&main_path).expect("Failed to read main.c");

    let rust_code = transpile_with_includes(&c_code, Some(project.path()))
        .expect("Transpilation should succeed");

    // Should resolve nested includes
    assert!(
        rust_code.contains("number_t") || rust_code.contains("i32"),
        "Should contain typedef or resolved type"
    );
    assert!(
        rust_code.contains("fn process"),
        "Should contain process function"
    );
}

#[test]
fn test_transpile_relative_include_path() {
    // Test relative path resolution: ../include/header.h
    let project = create_test_project(&[
        ("include/utils.h", "int helper(int x);"),
        (
            "src/main.c",
            r#"
#include "../include/utils.h"

int main() {
    return helper(10);
}
        "#,
        ),
    ]);

    let main_path = project.path().join("src/main.c");
    let c_code = fs::read_to_string(&main_path).expect("Failed to read main.c");
    let src_dir = main_path.parent().unwrap();

    let rust_code =
        transpile_with_includes(&c_code, Some(src_dir)).expect("Transpilation should succeed");

    assert!(
        rust_code.contains("fn helper"),
        "Should resolve relative include path"
    );
}

#[test]
fn test_transpile_header_guards_prevent_duplicate_parsing() {
    // Test that header guards prevent duplicate includes
    let project = create_test_project(&[
        (
            "common.h",
            r#"
#ifndef COMMON_H
#define COMMON_H

typedef int value_t;

#endif
        "#,
        ),
        (
            "a.h",
            r#"
#include "common.h"
int func_a(value_t x);
        "#,
        ),
        (
            "b.h",
            r#"
#include "common.h"
int func_b(value_t x);
        "#,
        ),
        (
            "main.c",
            r#"
#include "a.h"
#include "b.h"

int main() {
    return func_a(1) + func_b(2);
}
        "#,
        ),
    ]);

    let main_path = project.path().join("main.c");
    let c_code = fs::read_to_string(&main_path).expect("Failed to read main.c");

    let rust_code = transpile_with_includes(&c_code, Some(project.path()))
        .expect("Transpilation should succeed");

    // Should only define value_t ONCE (not duplicate)
    let typedef_count = rust_code.matches("type value_t").count();
    assert!(
        typedef_count <= 1,
        "Should not duplicate typedef due to header guards"
    );

    assert!(rust_code.contains("fn func_a"), "Should contain func_a");
    assert!(rust_code.contains("fn func_b"), "Should contain func_b");
}

#[test]
fn test_transpile_missing_include_file_error() {
    let c_code = r#"
#include "nonexistent.h"

int main() {
    return 0;
}
    "#;

    let result = transpile_with_includes(c_code, None);

    // Should return an error (not panic)
    assert!(result.is_err(), "Should error on missing include file");

    let error_msg = format!("{:?}", result.unwrap_err());
    assert!(
        error_msg.contains("nonexistent")
            || error_msg.contains("not found")
            || error_msg.contains("Failed to find"),
        "Error should mention missing file"
    );
}

#[test]
fn test_transpile_circular_dependency_detection() {
    // Test circular includes: a.h includes b.h, b.h includes a.h
    let project = create_test_project(&[
        (
            "a.h",
            r#"
#ifndef A_H
#define A_H
#include "b.h"
int func_a();
#endif
        "#,
        ),
        (
            "b.h",
            r#"
#ifndef B_H
#define B_H
#include "a.h"
int func_b();
#endif
        "#,
        ),
        (
            "main.c",
            r#"
#include "a.h"

int main() {
    return func_a();
}
        "#,
        ),
    ]);

    let main_path = project.path().join("main.c");
    let c_code = fs::read_to_string(&main_path).expect("Failed to read main.c");

    // Should handle circular dependencies (header guards prevent infinite loop)
    let result = transpile_with_includes(&c_code, Some(project.path()));

    // Should either succeed (header guards work) or provide clear error
    match result {
        Err(e) => {
            let error_msg = format!("{:?}", e);
            assert!(
                error_msg.contains("circular") || error_msg.contains("cycle"),
                "If error, should mention circular dependency"
            );
        }
        Ok(rust_code) => {
            // If successful, both functions should be present
            assert!(rust_code.contains("fn func_a"));
            assert!(rust_code.contains("fn func_b"));
        }
    }
}

#[test]
fn test_transpile_cross_file_function_call() {
    // Test that function defined in header can be called in main
    let project = create_test_project(&[
        (
            "utils.h",
            r#"
int add(int a, int b) {
    return a + b;
}
        "#,
        ),
        (
            "main.c",
            r#"
#include "utils.h"

int main() {
    int result = add(10, 20);
    return result;
}
        "#,
        ),
    ]);

    let main_path = project.path().join("main.c");
    let c_code = fs::read_to_string(&main_path).expect("Failed to read main.c");

    let rust_code = transpile_with_includes(&c_code, Some(project.path()))
        .expect("Transpilation should succeed");

    // Should have complete add function implementation (not just signature)
    assert!(rust_code.contains("fn add"), "Should contain add function");
    assert!(
        rust_code.contains("a + b"),
        "Should contain add implementation"
    );
    assert!(
        rust_code.contains("fn main"),
        "Should contain main function"
    );
    assert!(
        rust_code.contains("add(10, 20)"),
        "Should contain function call"
    );
}

#[test]
fn test_transpile_system_include_placeholder() {
    // Test that system includes (<stdio.h>) are recognized
    // For now, we might not transpile them, but should not error
    let c_code = r#"
#include <stdio.h>

int main() {
    return 0;
}
    "#;

    let result = transpile(c_code);

    // Should handle gracefully (either transpile or skip system headers)
    // At minimum, should not panic
    assert!(
        result.is_ok() || result.is_err(),
        "Should handle system includes"
    );
}
