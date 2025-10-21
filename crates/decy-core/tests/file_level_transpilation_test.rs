//! File-level transpilation tests (DECY-047 RED phase)
//!
//! These tests define the API for file-by-file transpilation to support
//! incremental C→Rust migration and large project handling.
//!
//! Goal: Enable transpiling C files independently with cross-file reference tracking.

use decy_core::{transpile_file, ProjectContext};
use std::path::Path;
use tempfile::TempDir;

/// Helper: Create temporary C file with content
fn create_temp_c_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).expect("Failed to write temp file");
    path
}

#[test]
fn test_transpile_single_file_without_dependencies() {
    // Test: Transpile a single C file that doesn't depend on other files
    let temp = TempDir::new().unwrap();
    let c_file = create_temp_c_file(&temp, "simple.c", "int add(int a, int b) { return a + b; }");

    let context = ProjectContext::new();
    let result = transpile_file(&c_file, &context);

    assert!(result.is_ok(), "Should transpile single file successfully");

    let transpiled = result.unwrap();
    assert_eq!(transpiled.source_path, c_file);
    assert!(
        transpiled.rust_code.contains("fn add"),
        "Should contain function"
    );
    assert!(
        transpiled.dependencies.is_empty(),
        "Should have no dependencies"
    );
}

#[test]
fn test_transpiled_file_has_metadata() {
    // Test: TranspiledFile struct contains all required metadata
    let temp = TempDir::new().unwrap();
    let c_file = create_temp_c_file(&temp, "test.c", "int get_value() { return 42; }");

    let context = ProjectContext::new();
    let result = transpile_file(&c_file, &context);

    assert!(result.is_ok());

    let transpiled = result.unwrap();
    assert!(
        transpiled.source_path.exists(),
        "Source path should be valid"
    );
    assert!(
        !transpiled.rust_code.is_empty(),
        "Should have generated Rust code"
    );
    assert!(
        !transpiled.functions_exported.is_empty(),
        "Should track exported functions"
    );
}

#[test]
fn test_transpile_file_with_header_dependency() {
    // Test: Transpile files where one references header via #include detection
    // Note: For simplicity, we test dependency detection without actual #include preprocessing
    let temp = TempDir::new().unwrap();

    // Create header file (declarations only)
    let header_content = r#"
        // utils.h - function declarations
        int utility_function(int x);
    "#;
    let header_file = create_temp_c_file(&temp, "utils.h", header_content);

    // Create implementation file (implementation without #include, since clang-sys needs preprocessed C)
    let impl_content = r#"
        int utility_function(int x) {
            return x * 2;
        }
    "#;
    let impl_file = create_temp_c_file(&temp, "utils.c", impl_content);

    let context = ProjectContext::new();

    // Transpile header first
    let _header_result = transpile_file(&header_file, &context);
    // Header has no function bodies, so may not parse as full C - that's ok for this test

    // Transpile implementation
    let impl_result = transpile_file(&impl_file, &context);
    assert!(impl_result.is_ok(), "Should transpile implementation file");

    let impl_transpiled = impl_result.unwrap();
    assert!(
        !impl_transpiled.rust_code.is_empty(),
        "Should generate Rust code"
    );
    assert!(impl_transpiled
        .functions_exported
        .contains(&"utility_function".to_string()));
}

#[test]
fn test_project_context_tracks_types() {
    // Test: ProjectContext tracks struct/enum types across files
    let temp = TempDir::new().unwrap();

    // Header defines struct
    let header = create_temp_c_file(
        &temp,
        "types.h",
        r#"
            struct Point {
                int x;
                int y;
            };
        "#,
    );

    let context = ProjectContext::new();
    let result = transpile_file(&header, &context);
    assert!(result.is_ok());

    let transpiled = result.unwrap();

    // Context should track the Point struct
    let mut context_with_types = ProjectContext::new();
    context_with_types.add_transpiled_file(&transpiled);

    assert!(
        context_with_types.has_type("Point"),
        "Context should track Point struct"
    );
}

#[test]
fn test_project_context_tracks_functions() {
    // Test: ProjectContext tracks function declarations across files
    let temp = TempDir::new().unwrap();

    let file = create_temp_c_file(&temp, "funcs.c", "int calculate(int n) { return n + 1; }");

    let context = ProjectContext::new();
    let result = transpile_file(&file, &context);
    assert!(result.is_ok());

    let transpiled = result.unwrap();

    let mut context_with_funcs = ProjectContext::new();
    context_with_funcs.add_transpiled_file(&transpiled);

    assert!(
        context_with_funcs.has_function("calculate"),
        "Context should track calculate function"
    );
}

#[test]
fn test_ffi_boundary_generation_for_c_functions() {
    // Test: Generate extern "C" declarations for C functions that need FFI
    let temp = TempDir::new().unwrap();

    // File with function that will be called from C
    let file = create_temp_c_file(
        &temp,
        "api.c",
        "int public_api(int value) { return value * 2; }",
    );

    let context = ProjectContext::new();
    let result = transpile_file(&file, &context);
    assert!(result.is_ok());

    let transpiled = result.unwrap();

    // Should generate FFI boundary for public_api
    assert!(
        transpiled.ffi_declarations.contains("extern \"C\""),
        "Should generate extern C declarations"
    );
    assert!(
        transpiled.ffi_declarations.contains("public_api"),
        "Should include function name in FFI"
    );
}

#[test]
fn test_transpile_multiple_files_with_cross_references() {
    // Integration test: Transpile two files where one references the other
    let temp = TempDir::new().unwrap();

    // utils.c defines a function
    let utils = create_temp_c_file(&temp, "utils.c", "int helper(int x) { return x + 10; }");

    // main.c calls helper
    let main_c = create_temp_c_file(
        &temp,
        "main.c",
        r#"
            extern int helper(int x);
            int main() {
                int result;
                result = helper(5);
                return 0;
            }
        "#,
    );

    let mut context = ProjectContext::new();

    // Transpile utils.c first
    let utils_result = transpile_file(&utils, &context);
    assert!(utils_result.is_ok());

    let utils_transpiled = utils_result.unwrap();
    context.add_transpiled_file(&utils_transpiled);

    // Transpile main.c with utils in context
    let main_result = transpile_file(&main_c, &context);
    assert!(main_result.is_ok());

    let main_transpiled = main_result.unwrap();

    // main should reference helper from utils
    assert!(
        main_transpiled.rust_code.contains("helper"),
        "Should call helper function"
    );
}

#[test]
fn test_transpile_header_only_file() {
    // Test: Transpile a header file (declarations only, no implementations)
    let temp = TempDir::new().unwrap();

    let header = create_temp_c_file(
        &temp,
        "api.h",
        r#"
            #ifndef API_H
            #define API_H

            int compute(int a, int b);
            void process(int* data, int count);

            #endif
        "#,
    );

    let context = ProjectContext::new();
    let result = transpile_file(&header, &context);

    assert!(result.is_ok(), "Should transpile header file");

    let transpiled = result.unwrap();

    // Header should track function declarations for context
    assert!(
        transpiled
            .functions_exported
            .contains(&"compute".to_string()),
        "Should track compute declaration"
    );
    assert!(
        transpiled
            .functions_exported
            .contains(&"process".to_string()),
        "Should track process declaration"
    );
}

#[test]
fn test_transpile_nonexistent_file_returns_error() {
    // Test: Attempting to transpile a file that doesn't exist returns error
    let nonexistent_path = Path::new("/nonexistent/file.c");
    let context = ProjectContext::new();

    let result = transpile_file(nonexistent_path, &context);

    assert!(result.is_err(), "Should error for nonexistent file");
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("not found")
            || error_msg.contains("No such file")
            || error_msg.contains("Failed to read file"),
        "Error should mention file not found, got: {}",
        error_msg
    );
}
